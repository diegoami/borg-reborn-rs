/**
 * \file main-borgh.c
 * \brief Headless, deterministic frontend that runs the Borg ("borgh").
 *
 * Not part of upstream Angband. borg-reborn-rs's frontend (docs/SPEC.md
 * §3.5, task TE.2): copied from borg-rs 286215c's main-borgh.c and trimmed
 * to the contract (docs/PROVENANCE.md); it is ours from here on. Dropped:
 * the native-observer stream (-states), recorded frames (-frames) and their
 * diagnostics, fixture building (-make-fixture, -near-morgoth), the rune
 * stop (-stop-rune-pattern) and cheat_live (-cheat-live; D5 excludes cheat
 * death). Modelled on main-test.c.
 *
 * Copyright (c) 2011 Elly <elly+angband@leptoquark.net> (main-test.c)
 * Copyright (c) 2026 borg-rs contributors
 * Copyright (c) 2026 borg-reborn-rs contributors
 *
 * This work is free software; you can redistribute it and/or modify it
 * under the terms of either:
 *
 * a) the GNU General Public License as published by the Free Software
 *    Foundation, version 2, or
 *
 * b) the "Angband licence":
 *    This software may be copied and distributed for educational, research,
 *    and not for profit purposes provided that this copyright and statement
 *    are included in all such copies.  Other copyrights may also apply.
 *
 * What it does:
 *  - provides an 80x24 terminal that draws nothing and never sleeps;
 *  - feeds a scripted key sequence that creates a character (given race and
 *    class, point-based stats, default name) and then sends Ctrl-Z z to start
 *    the Borg;
 *  - logs every key returned by inkey_hack (one line per key:
 *    "<game turn> <event type> <key code hex> <mods hex>");
 *  - optionally logs every message added to the message log, and per key
 *    a digest of the engine state (read only: no RNG, no game state);
 *  - stops on a game-turn limit, a character level or max depth limit,
 *    player death, or when the game blocks waiting for a real key;
 *  - writes a JSON summary on exit.
 *
 * Usage: angband -mborgh -n -u<name> -d... -- [options]
 *   -cborg           switch to CBorg through the normal command prompt
 *   -seed N       seed the game RNG (also settable with $ANGBAND_SEED)
 *   -race N       race index (0-based, as in p_race.txt); explicit only
 *   -class N      class index (0-based, as in class.txt); explicit only
 *   -turns N      stop when the game turn reaches N (0 = no limit)
 *   -stall-decisions N  stop after N decisions without a world-turn advance
 *   -exact-turns  stop at the world-turn increment, before another key
 *   -random-birth choose race and class using the seeded birth menu's *
 *   -clev N       stop when the character level reaches N (0 = no limit)
 *   -depth N      stop when the max depth reaches N (0 = no limit)
 *   -start-depth N  synthetic birth at dungeon level N (no load/random birth)
 *                 one-way: no ascent or recall; further descent is allowed
 *   -start-monsters SPEC  with -start-depth (after the level is generated)
 *                 or -load (after the replay seed, the character in the
 *                 dungeon): place monsters near the player (#722, a breeder
 *                 test bed):
 *                 NAME:COUNT[:DIST] joined by '+', '_' for a space in the
 *                 name (yellow_worm_mass:6+green_worm_mass:4:8); DIST is the
 *                 scatter distance from the player (default 6)
 *   -keylog FILE  keypress log (default: none)
 *   -msglog FILE  message log: every message_add() call, Borg notes
 *                 included (default: none)
 *   -statelog FILE engine-state log: one line per logged key (default: none)
 *   -summary FILE summary JSON (default: stdout)
 *   -load FILE   load a living character instead of birth (omit -n);
 *                 reset RNG to -seed immediately before starting the Borg
 *   -save-on-stop FILE write a living character at the stop boundary
 *   -checkpoint-every N capture at the first game-command boundary at or
 *                 after each multiple of N game turns (requires directory)
 *   -checkpoint-dir DIR directory for independent engine savefiles and index
 *   -screen       dump the screen to stderr when the run ends unexpectedly
 *   -menu-letters pick race and class with the birth menu's own letters
 *                 (all_letters_nohjkl).  Without it the script sends
 *                 "abc..z"[index], which skips no letters, so an index >= 7
 *                 picks a different entry (kept as the default so the
 *                 existing reference logs stay valid).
 */

#include "angband.h"
#include "buildid.h"
#include "z-color.h"
#include "z-quark.h"
#include "monster.h"
#include "object.h"
#include "player-timed.h"
#include "cmds.h"
#include "generate.h"
#include "game-world.h"
#include "game-event.h"
#include "main.h"
#include "main-borgh.h"
#include "synthetic-build.h"
#include "ui-input.h"
#include "init.h"
#include "obj-knowledge.h"
#include "obj-gear.h"
#include "obj-make.h"
#include "obj-pile.h"
#include "obj-util.h"
#include "mon-util.h"
#include "mon-make.h"
#include "cave.h"
#include "object.h"
#include "player.h"
#include "player-calcs.h"
#include "player-timed.h"
#include "player-util.h"
#include "savefile.h"
#include "ui-game.h"
#include "ui-menu.h"
#include "ui-term.h"

#include <sys/time.h>
#ifndef _WIN32
#include <sys/wait.h>
#else
/* #823: Windows' C runtime names it _putenv_s; the one caller sets an
 * unset variable, so the no-overwrite flag makes no difference. */
#define setenv(name, value, overwrite) _putenv_s((name), (value))
#endif
#include <unistd.h>
#include <errno.h>
#ifdef __linux__
#include <sys/prctl.h>
#endif

#if defined(USE_BORGH) && defined(BORG_HARNESS) && defined(ALLOW_BORG)

extern struct keypress (*inkey_hack)(int flush_first);

/* Options */
static bool opt_exact_turns;
static bool opt_random_birth;
static long opt_stall_decisions;
static unsigned long decisions;
static long stall_turn;
static unsigned long stall_start_decisions;
static unsigned long stall_start_energy;
static uint32_t borg_rng;
#define BORGH_SETTINGS_MAX 256
static char setting_names[BORGH_SETTINGS_MAX][96];
static int setting_values[BORGH_SETTINGS_MAX];
static int setting_count;
static int opt_race = 0;
static int opt_class = 0;
static long opt_turns = 0;
static long opt_keys = 0;
static int opt_clev = 0;
static int opt_depth = 0;
static int opt_start_depth = 0;
static int initial_depth = 0;
static int initial_clevel, initial_max_hp, initial_max_sp;
static int initial_cave_depth;
static bool initial_generation_pending;
static const char *opt_start_build;
static const char *opt_start_monsters;
static int start_monsters_placed;
static int start_monsters_requested;
static char start_build_sha256[65];
static const char *opt_capture_build;
static int opt_capture_build_level;
static bool capture_level_seen;
static bool synthetic_start_authorized;
static long build_capture_turn;
static char build_capture_sha256[65];
static const char *opt_keylog = NULL;
static const char *opt_msglog = NULL;
static const char *opt_statelog = NULL;
static const char *opt_summary = NULL;
static bool opt_screen = false;
static bool opt_menu_letters = false;
static const char *opt_load = NULL;
static const char *opt_save = NULL;
static bool opt_save_living_only = false;
static bool opt_keep_load_rng = false;
static bool opt_save_dead = false;
static long loaded_turn = 0;
static long opt_checkpoint_every = 0;
static const char *opt_checkpoint_dir = NULL;
static FILE *checkpoint_fp = NULL;
static long checkpoint_next = 0;
static long checkpoint_last_seen = 0;
static unsigned long checkpoint_count = 0;

/* State */
static FILE *keylog_fp = NULL;
static FILE *msglog_fp = NULL;
static FILE *statelog_fp = NULL;
static unsigned long keys_logged = 0;
static struct timeval start_tv;
static bool finished = false;
static bool borg_started = false;
static bool opt_cborg = false;
static bool test_map_export = false;
static int startup_acks = 0;	/* "-more-" prompts answered before the Borg's first key */

/* Scripted keys (before the Borg is running) */
#define SCRIPT_MAX 64
#define SCRIPT_START_BORG 0x7fffffff /* marker: set options, then Ctrl-Z */
static keycode_t script[SCRIPT_MAX];
static int script_len = 0;
static int script_pos = 0;

static const char all_letters_borgh[] = "abcdefghijklmnopqrstuvwxyz";

static void script_add(keycode_t k)
{
	if (script_len < SCRIPT_MAX) script[script_len++] = k;
}

static void json_str(FILE *fp, const char *s)
{
	fputc('"', fp);
	for (; s && *s; s++) {
		unsigned char c = (unsigned char)*s;
		if (c == '"' || c == '\\') fprintf(fp, "\\%c", c);
		else if (c < 0x20) fprintf(fp, "\\u%04x", c);
		else fputc(c, fp);
	}
	fputc('"', fp);
}

static void dump_screen(void)
{
	int x, y, a;
	wchar_t c;
	char buf[8];

	if (!Term) return;
	fprintf(stderr, "---- screen ----\n");
	for (y = 0; y < Term->hgt; y++) {
		for (x = 0; x < Term->wid; x++) {
			int n;
			(void)Term_what(x, y, &a, &c);
			n = wctomb(buf, c);
			if (n <= 0) { buf[0] = '?'; n = 1; }
			fwrite(buf, 1, n, stderr);
		}
		fputc('\n', stderr);
	}
	fprintf(stderr, "----------------\n");
}


/**
 * Write the summary and exit the process without saving.
 */
static void borgh_finish(const char *reason)
{
	struct timeval now;
	double secs;
	FILE *fp = stdout;
	bool io_error = false;
    /* Native saving may mutate state. Preserve the RNG at the stop. */
    bool stop_quick = Rand_quick;
    uint32_t stop_value = Rand_value, stop_index = state_i;
    uint32_t stop_rng[RAND_DEG];
    memcpy(stop_rng, STATE, sizeof(stop_rng));

	if (finished) return;
	if (opt_start_depth && !initial_depth) {
		dump_screen();
		quit_fmt("borgh: synthetic start was not verified before stop (%s)", reason);
	}
	finished = true;

	gettimeofday(&now, NULL);
	secs = (now.tv_sec - start_tv.tv_sec)
		+ (now.tv_usec - start_tv.tv_usec) / 1e6;

	if (keylog_fp) {
		if (ferror(keylog_fp)) io_error = true;
		if (fclose(keylog_fp) != 0) io_error = true;
		keylog_fp = NULL;
	}
	if (msglog_fp) {
		if (ferror(msglog_fp)) io_error = true;
		if (fclose(msglog_fp) != 0) io_error = true;
		msglog_fp = NULL;
	}
	if (statelog_fp) {
		if (ferror(statelog_fp)) io_error = true;
		if (fclose(statelog_fp) != 0) io_error = true;
		statelog_fp = NULL;
	}

	if (opt_screen && !streq(reason, "turns")) dump_screen();
	if (opt_save && !(opt_save_living_only && player && player->is_dead)
		&& (!player || (player->is_dead && !opt_save_dead) || !savefile_save(opt_save))) {
		fprintf(stderr, "borgh: cannot save living character to %s\n", opt_save);
		_exit(3);
	}

	if (opt_summary) {
		fp = fopen(opt_summary, "wb");
		if (!fp) fp = stdout;
	}
	fprintf(fp, "{\n");
	fprintf(fp, "  \"seed\": %lu,\n", (unsigned long)borgh_seed);
	/* Every summary says whether a save was loaded (docs/SPEC.md §3.5). */
	fprintf(fp, "  \"loaded\": %s,\n", opt_load ? "true" : "false");
	if (opt_start_monsters)
	{
		fprintf(fp, "  \"start_monsters\": ");
		json_str(fp, opt_start_monsters);
		fprintf(fp, ",\n  \"start_monsters_requested\": %d,\n  \"start_monsters_placed\": %d,\n",
			start_monsters_requested, start_monsters_placed);
	}
	if (opt_start_depth) {
		fprintf(fp, "  \"synthetic\": true,\n");
		fprintf(fp, "  \"synthetic_contract\": \"borg-reborn-rs/synthetic-start@2\",\n");
		fprintf(fp, "  \"start_depth\": %d,\n", opt_start_depth);
		fprintf(fp, "  \"initial_depth\": %d,\n", initial_depth);
		fprintf(fp, "  \"initial_cave_depth\": %d,\n  \"initial_generation_pending\": %s,\n", initial_cave_depth, initial_generation_pending ? "true" : "false");
		fprintf(fp, "  \"initial_clevel\": %d,\n  \"initial_max_hp\": %d,\n  \"initial_max_sp\": %d,\n", initial_clevel, initial_max_hp, initial_max_sp);
		fprintf(fp, "  \"start_build\": \"%s\",\n", opt_start_build ? "record" : "birth");
		if (opt_start_build) fprintf(fp, "  \"start_build_sha256\": \"%s\",\n", start_build_sha256);
		fprintf(fp, "  \"dungeon_mode\": \"one_way\",\n");
	}
	if (opt_load)
		fprintf(fp, "  \"loaded_turn\": %ld,\n", loaded_turn);
	if (opt_capture_build) {
		fprintf(fp, "  \"build_capture_status\": \"%s\",\n", build_capture_turn ? "captured" : "absent");
		fprintf(fp, "  \"build_capture_requested_level\": %d,\n", opt_capture_build_level);
	}
	if (build_capture_turn)
		fprintf(fp, "  \"build_capture_turn\": %ld,\n  \"build_capture_level\": %d,\n", build_capture_turn, opt_capture_build_level);
	if (build_capture_turn)
		fprintf(fp, "  \"build_capture_sha256\": \"%s\",\n", build_capture_sha256);
	fprintf(fp, "  \"race\": ");
	json_str(fp, (player && player->race) ? player->race->name : "");
	fprintf(fp, ",\n  \"race_idx\": %d,\n", opt_race);
	fprintf(fp, "  \"class\": ");
	json_str(fp, (player && player->class) ? player->class->name : "");
	fprintf(fp, ",\n  \"class_idx\": %d,\n", opt_class);
	if (opt_menu_letters) fprintf(fp, "  \"menu_letters\": true,\n");
	if (startup_acks)
		fprintf(fp, "  \"startup_acks\": %d,\n", startup_acks);
	fprintf(fp, "  \"stop_reason\": ");
	json_str(fp, reason);
	fprintf(fp, ",\n  \"game_turns\": %ld,\n", (long)turn);
	fprintf(fp, "  \"player_turns\": %lu,\n",
		player ? (unsigned long)(player->total_energy / 100) : 0UL);
	fprintf(fp, "  \"resting_turns\": %lu,\n",
		player ? (unsigned long)player->resting_turn : 0UL);
	fprintf(fp, "  \"clevel\": %d,\n", player ? player->lev : 0);
	fprintf(fp, "  \"max_depth\": %d,\n", player ? player->max_depth : 0);
	fprintf(fp, "  \"depth\": %d,\n", player ? player->depth : 0);
	fprintf(fp, "  \"dead\": %s,\n",
		(player && player->is_dead) ? "true" : "false");
	fprintf(fp, "  \"death_cause\": ");
	json_str(fp, (player && player->is_dead) ? player->died_from : "");
	fprintf(fp, ",\n  \"keys\": %lu,\n", keys_logged);
    fprintf(fp, "  \"calendar\": \"fixed\",\n");
    fprintf(fp, "  \"max_clevel\": %d,\n  \"hp\": %d,\n  \"max_hp\": %d,\n",
        player ? player->max_lev : 0, player ? player->chp : 0, player ? player->mhp : 0);
    fprintf(fp, "  \"borg_decisions\": %lu,\n  \"player_energy\": %lu,\n",
        decisions, player ? (unsigned long)player->total_energy : 0UL);
    fprintf(fp, "  \"borg_rng\": %lu,\n  \"game_rng\": {\"quick\": %s, \"quick_rng\": %lu, \"rng_index\": %lu, \"rng\": \"",
        (unsigned long)borg_rng, stop_quick ? "true" : "false",
        (unsigned long)stop_value, (unsigned long)stop_index);
    {
        int j;
        for (j = 0; j < RAND_DEG; ++j) fprintf(fp, "%08lx", (unsigned long)stop_rng[j]);
        fprintf(fp, "\"},\n  \"effective_settings\": {");
        for (j = 0; j < setting_count; ++j) {
            if (j) fprintf(fp, ",");
            json_str(fp, setting_names[j]);
            fprintf(fp, ": %d", setting_values[j]);
        }
        fprintf(fp, "},\n");
    }
    /* The game options in force at the stop, as the Borg set them: part of
     * the run record (docs/SPEC.md §4, docs/ENGINE.md). */
    if (player) {
        int j;
        bool first = true;
        fprintf(fp, "  \"game_options\": {");
        for (j = 1; j < OPT_MAX; ++j) {
            const char *name = option_name(j);
            if (!name) continue;
            fprintf(fp, "%s", first ? "" : ",");
            first = false;
            json_str(fp, name);
            fprintf(fp, ": %s", player->opts.opt[j] ? "true" : "false");
        }
        fprintf(fp, "},\n");
    }
    fprintf(fp, "  \"exact_turns\": %s,\n", opt_exact_turns ? "true" : "false");
    if (opt_random_birth) {
        fprintf(fp, "  \"random_birth\": true,\n  \"played_race_idx\": %u,\n  \"played_class_idx\": %u,\n",
            player->race->ridx, player->class->cidx);
    }
    if (streq(reason, "stalled"))
        fprintf(fp, "  \"stall_turn\": %ld,\n  \"stall_decisions\": %lu,\n  \"stall_energy_delta\": %lu,\n",
            stall_turn, decisions - stall_start_decisions,
            (unsigned long)player->total_energy - stall_start_energy);
	fprintf(fp, "  \"wall_seconds\": %.3f\n", secs);
	fprintf(fp, "}\n");
	if (fp != stdout) {
		if (ferror(fp)) io_error = true;
		if (fclose(fp) != 0) io_error = true;
	}
	fflush(stdout);

	/* A short key log or summary must not pass as a good run */
	if (io_error) {
		fprintf(stderr, "borgh: error writing a log or the summary\n");
		_exit(3);
	}

#ifdef BORG_RECORDING
    { extern void ar_finish(void); ar_finish(); }
#endif
    /* Do not save; do not run the normal quit hooks */
    _exit(0);
}

void borgh_setting(int index, const char *name, int value)
{
    if (!borg_started) return;
    if (index < 0 || index >= BORGH_SETTINGS_MAX) quit("borgh: too many settings");
    my_strcpy(setting_names[index], name, sizeof(setting_names[index]));
    setting_values[index] = value;
    if (setting_count <= index) setting_count = index + 1;
}

void borgh_rng(uint32_t rng) { borg_rng = rng; }

void borgh_decision(uint32_t rng)
{
    if (!borg_started) return;
    borg_rng = rng;
    ++decisions;
    if (decisions == 1 || turn != stall_turn) {
        stall_turn = turn;
        stall_start_decisions = decisions;
        stall_start_energy = player->total_energy;
    }
    if (opt_stall_decisions > 0 && decisions - stall_start_decisions >= (unsigned long)opt_stall_decisions)
        borgh_finish("stalled");
}

static void borgh_tick(void)
{
    if (borg_started && opt_exact_turns && opt_turns > 0 && turn >= opt_turns)
        borgh_finish("turns");
}

/* ---- message and engine-state logs (read only) ---- */

/**
 * Observer for every message_add() call (repeats included).
 * Line: "<keys logged so far> <game turn> <message type> <text>", with
 * bytes below 0x20, 0x7f and backslash written as \xNN.
 */
static void borgh_message(const char *str, uint16_t type)
{
	const unsigned char *c;

	if (!msglog_fp) return;
	fprintf(msglog_fp, "%lu %ld %u ", keys_logged, (long)turn,
		(unsigned)type);
	for (c = (const unsigned char *)str; c && *c; c++) {
		if (*c < 0x20 || *c == 0x7f || *c == '\\')
			fprintf(msglog_fp, "\\x%02x", *c);
		else
			fputc(*c, msglog_fp);
	}
	fputc('\n', msglog_fp);
}

/* FNV-1a, 64 bit, over values fed as little-endian 32-bit words */
#define FNV_OFFSET 0xcbf29ce484222325ULL
#define FNV_PRIME 0x100000001b3ULL

static void fnv_int(uint64_t *h, long v)
{
	uint32_t u = (uint32_t)v;
	int i;

	for (i = 0; i < 4; i++) {
		*h ^= (u >> (8 * i)) & 0xff;
		*h *= FNV_PRIME;
	}
}

/**
 * Feed the fields of one object (or of its known version) into a hash.
 */
static void fnv_object_fields(uint64_t *h, const struct object *obj)
{
	int i;

	if (!obj) {
		fnv_int(h, -1);
		return;
	}
	fnv_int(h, obj->kind ? (long)obj->kind->kidx : -1);
	fnv_int(h, obj->tval);
	fnv_int(h, obj->sval);
	fnv_int(h, obj->number);
	fnv_int(h, obj->pval);
	fnv_int(h, obj->ac);
	fnv_int(h, obj->to_h);
	fnv_int(h, obj->to_d);
	fnv_int(h, obj->to_a);
	/* obj->timeout is not hashed: it is logged on its own ("o" field) */
	fnv_int(h, obj->notice);
	for (i = 0; i < (int)OF_SIZE; i++) fnv_int(h, obj->flags[i]);
	for (i = 0; i < OBJ_MOD_MAX; i++) fnv_int(h, obj->modifiers[i]);
	for (i = 0; i < ELEM_MAX; i++) {
		fnv_int(h, obj->el_info[i].res_level);
		fnv_int(h, obj->el_info[i].flags);
	}
	fnv_int(h, obj->curses ? 1 : 0);
	if (obj->curses) {
		for (i = 0; i < z_info->curse_max; i++) {
			fnv_int(h, obj->curses[i].power);
			fnv_int(h, obj->curses[i].timeout);
		}
	}
}

/**
 * Hashes of the gear, in gear-list order: pack and quiver (objects not
 * equipped) and equipment (equipped, with the slot). Each object counts
 * with its own fields and those of its known version (what the player
 * knows of it: the runes learnt on it), except the timeout (light fuel,
 * recharge), which changes nearly every turn and is logged as a number.
 * The third hash is the player's rune knowledge (which runes are known
 * at all).
 */
static void gear_hashes(uint64_t *pack, uint64_t *equip, uint64_t *runes)
{
	struct object *obj;
	int i, n;

	*pack = FNV_OFFSET;
	*equip = FNV_OFFSET;
	*runes = FNV_OFFSET;
	for (obj = player->gear; obj; obj = obj->next) {
		int slot = equipped_item_slot(player->body, obj);
		uint64_t *h = (slot < player->body.count) ? equip : pack;

		fnv_int(h, slot);
		fnv_object_fields(h, obj);
		fnv_object_fields(h, obj->known);
	}
	if (player->obj_k) {
		n = max_runes();
		for (i = 0; i < n; i++)
			fnv_int(runes, player_knows_rune(player, i) ? 1 : 0);
	}
}

/**
 * One line of the state log, read when a key is returned:
 * "<turn> <x>,<y> d<depth>/<max depth> hp<chp>/<mhp> sp<csp>/<msp>
 *  f<food> au<gold> xp<exp> lv<level> t<index>:<value>,...|t-
 *  o<gear index>:<timeout>,...|o- p<pack hash> e<equipment hash>
 *  r<rune knowledge hash>".
 * Timed effects are listed by index (list-player-timed.h order) when set;
 * food has its own field. The o field lists the gear objects (gear-list
 * position from 0) whose timeout is not zero.
 */
static void borgh_state(void)
{
	uint64_t ph, eh, rh;
	const struct object *obj;
	int i;
	bool any = false;

	if (!statelog_fp) return;
	if (!player || !player->timed) {
		fprintf(statelog_fp, "%ld -\n", (long)turn);
		return;
	}
	gear_hashes(&ph, &eh, &rh);
	fprintf(statelog_fp, "%ld %d,%d d%d/%d hp%d/%d sp%d/%d f%d au%ld xp%ld"
		" lv%d t", (long)turn, player->grid.x, player->grid.y,
		player->depth, player->max_depth, player->chp, player->mhp,
		player->csp, player->msp, player->timed[TMD_FOOD],
		(long)player->au, (long)player->exp, player->lev);
	for (i = 0; i < TMD_MAX; i++) {
		if (i == TMD_FOOD || !player->timed[i]) continue;
		fprintf(statelog_fp, "%s%d:%d", any ? "," : "", i,
			player->timed[i]);
		any = true;
	}
	if (!any) fputc('-', statelog_fp);
	fputs(" o", statelog_fp);
	any = false;
	for (obj = player->gear, i = 0; obj; obj = obj->next, i++) {
		if (!obj->timeout) continue;
		fprintf(statelog_fp, "%s%d:%d", any ? "," : "", i, obj->timeout);
		any = true;
	}
	if (!any) fputc('-', statelog_fp);
	fprintf(statelog_fp, " p%016llx e%016llx r%016llx\n",
		(unsigned long long)ph, (unsigned long long)eh,
		(unsigned long long)rh);
}

/**
 * Observer for every key returned by inkey_hack.
 */
static void borgh_key(struct keypress k)
{
	if (opt_start_depth && !initial_depth) {
		/* This is after generation, before the first Borg key is played.
		 * Reject a town fallback or overwritten birth options as a failure. */
		if (!player || !cave || cave->depth != opt_start_depth ||
			player->upkeep->generate_level || player->depth != opt_start_depth ||
			player->max_depth != opt_start_depth ||
			!player->race || !player->class ||
			player->race->ridx != (unsigned)opt_race ||
			player->class->cidx != (unsigned)opt_class ||
			!OPT(player, birth_force_descend) || !OPT(player, birth_no_recall))
			quit("borgh: synthetic start identity mismatch");
		initial_depth = player->depth;
		initial_cave_depth = cave->depth;
		initial_generation_pending = player->upkeep->generate_level;
		initial_clevel = player->lev;
		initial_max_hp = player->mhp;
		initial_max_sp = player->msp;
	}
	keys_logged++;
	if (opt_capture_build && !build_capture_turn && player->lev == opt_capture_build_level) {
		synthetic_build_capture(opt_capture_build);
		synthetic_build_digest(opt_capture_build, build_capture_sha256);
		build_capture_turn = (long)turn;
	}
	if (keylog_fp) {
		fprintf(keylog_fp, "%ld %d %lx %x\n", (long)turn, (int)k.type,
			(unsigned long)k.code, (unsigned)k.mods);
	}
	borgh_state();

	/* Stop conditions */
	if (opt_keys > 0 && keys_logged >= opt_keys) borgh_finish("keys");
	if (opt_turns > 0 && turn >= opt_turns) borgh_finish("turns");
	if (player && player->is_dead) borgh_finish("death");
	if (opt_clev > 0 && player && player->lev >= opt_clev)
		borgh_finish("clevel");
	if (opt_depth > 0 && player && player->max_depth >= opt_depth)
		borgh_finish("depth");
}

/* Capture only at a top-level command boundary, never inside a prompt or
 * a Borg callback. A replay starts a NEW Borg; its private memory is not
 * serialised. Capturing must not send keys, reseed RNG or add messages. */
static void borgh_checkpoint(void)
{
#ifdef _WIN32
	/* #823: a checkpoint saves from a fork()ed copy of the engine, which
	 * Windows has not; -checkpoint-every is refused there. */
	if (checkpoint_fp) quit("borgh: checkpoints need fork(), not available on Windows");
}
#else
	char name[128], path[1024];
	pid_t snapshot, parent_pid;
	int status;
	long due;
	if (!checkpoint_fp || !borg_started || !keys_logged || !player ||
		player->is_dead || !character_dungeon || player->upkeep->generate_level)
		return;
	if (!checkpoint_next || turn < checkpoint_last_seen)
		checkpoint_next = ((long)turn / opt_checkpoint_every + 1) * opt_checkpoint_every;
	checkpoint_last_seen = (long)turn;
	if (turn < checkpoint_next) return;
	due = checkpoint_next;
	checkpoint_count++;
	strnfmt(name, sizeof(name), "checkpoint-%06lu-t%010ld.save", checkpoint_count, (long)turn);
	path_build(path, sizeof(path), opt_checkpoint_dir, name);
	if (file_exists(path)) quit("borgh: checkpoint already exists");
	/* The stock saver compacts monster indices and changes saved flags.
	 * Save in a private copy of this single-threaded engine instead of
	 * mutating the live simulation. The child never flushes inherited
	 * oracle streams; it does not execute another Borg action. */
	parent_pid = getpid();
	snapshot = fork();
	if (snapshot < 0) {
		fprintf(stderr, "borgh: checkpoint fork failed\n");
		_exit(3);
	}
	if (snapshot == 0) {
#ifdef BORG_RECORDING
        /* The saver child must never consume or append the parent's streams. */
        { extern void ar_snapshot_enter(void); ar_snapshot_enter(); }
#endif
#ifdef __linux__
		/* Do not leave a saving child behind if the supervised engine is
		 * killed. Check the race between fork and installing this guard. */
		if (prctl(PR_SET_PDEATHSIG, SIGKILL) != 0 || getppid() != parent_pid)
			_exit(3);
#else
		(void)parent_pid;
#endif
		if (keylog_fp) close(fileno(keylog_fp));
		if (msglog_fp) close(fileno(msglog_fp));
		if (statelog_fp) close(fileno(statelog_fp));
		close(fileno(checkpoint_fp));
		keylog_fp = msglog_fp = statelog_fp = checkpoint_fp = NULL;
		_exit(savefile_save(path) ? 0 : 3);
	}
	while (waitpid(snapshot, &status, 0) < 0) {
		if (errno == EINTR) continue;
		fprintf(stderr, "borgh: checkpoint wait failed\n");
		_exit(3);
	}
	if (!WIFEXITED(status) || WEXITSTATUS(status) != 0) {
		fprintf(stderr, "borgh: checkpoint child failed: %s\n", path);
		_exit(3);
	}
	fprintf(checkpoint_fp,
		"{\"file\":\"%s\",\"due_turn\":%ld,\"game_turn\":%ld,"
		"\"keys\":%lu,\"seed\":%lu,\"race_idx\":%u,\"class_idx\":%u,"
		"\"depth\":%d,\"max_depth\":%d,\"clevel\":%d,"
		"\"capture_method\":\"fork_private_engine\"}\n",
		name, due, (long)turn, keys_logged, (unsigned long)borgh_seed,
		player->race->ridx, player->class->cidx, player->depth,
		player->max_depth, player->lev);
	if (fflush(checkpoint_fp) != 0 || ferror(checkpoint_fp)) {
		fprintf(stderr, "borgh: checkpoint index write failed\n");
		_exit(3);
	}
	/* Fixed cadence, not crash-aware cut points. A long rest can cross
	 * several thresholds: capture once at the next safe boundary. */
	checkpoint_next = ((long)turn / opt_checkpoint_every + 1) * opt_checkpoint_every;
}
#endif


static bool junk_consumables_town_fixture(void)
{
#ifdef BORGH_TEST_JUNK_CONSUMABLES
	const char *fixture = getenv("BORGH_TEST_JUNK_CONSUMABLES");
	return fixture && streq(fixture, "town");
#else
	return false;
#endif
}

#ifdef BORGH_TEST_JUNK_CONSUMABLES
/* Give the attack a known line of fire and a target at an exact range. */
static void place_junk_consumables_target(struct monster_race *race, int distance)
{
	struct monster_group_info info = { 0, 0 };
	struct loc target = loc(player->grid.x + distance, player->grid.y);
	for (int step = 1; step <= distance; step++) {
		struct loc grid = loc(player->grid.x + step, player->grid.y);
		if (!square_in_bounds_fully(cave, grid) || square_monster(cave, grid))
			quit("borgh: junk consumables fixture line is obstructed");
		square_set_feat(cave, grid, FEAT_FLOOR);
		sqinfo_on(square(cave, grid)->info, SQUARE_GLOW);
		square_memorize(cave, grid);
	}
	if (!place_new_monster(cave, target, race, false, false, info, ORIGIN_DROP_WIZARD))
		quit("borgh: could not place junk consumables fixture monster");
	player->upkeep->update |= PU_UPDATE_VIEW | PU_MONSTERS;
	update_stuff(player);
	start_monsters_placed++;
}
#endif

/**
 * borg-reborn-rs (#722): place the monsters of -start-monsters near the
 * player, as the debug command's summon does (cmd-wizard.c,
 * do_cmd_wiz_summon_named): scatter_ext to an empty grid in view within
 * DIST, then place_new_monster (awake per monster.txt's sleep, no group).
 * The engine draws its own random numbers, so the run stays deterministic.
 */
static void place_start_monsters(const char *spec)
{
	char buf[512];
	char *entry, *save = NULL;

	if (strlen(spec) >= sizeof(buf)) quit("borgh: -start-monsters is too long");
	my_strcpy(buf, spec, sizeof(buf));
	for (entry = strtok_r(buf, "+", &save); entry; entry = strtok_r(NULL, "+", &save)) {
		char name[80];
		char *colon = strchr(entry, ':');
		char *p;
		long count, dist = 6;
		struct monster_race *race;
		struct monster_group_info info = { 0, 0 };

		if (!colon || colon == entry || (size_t)(colon - entry) >= sizeof(name))
			quit("borgh: -start-monsters entry must be NAME:COUNT[:DIST]");
		memcpy(name, entry, (size_t)(colon - entry));
		name[colon - entry] = '\0';
		for (p = name; *p; p++) if (*p == '_') *p = ' ';
		count = strtol(colon + 1, &p, 10);
		if (*p == ':') dist = strtol(p + 1, &p, 10);
		if (*p != '\0' || count < 1 || count > 100 || dist < 1 || dist > 40)
			quit("borgh: -start-monsters needs COUNT 1..100 and DIST 1..40");
		/* Review of #741: lookup_monster falls back to a substring match
		 * ("worm" is the white worm mass); the bed names its species
		 * exactly, so anything else is refused. */
		race = lookup_monster(name);
		if (!race || !race->name || my_stricmp(race->name, name) != 0)
			quit_fmt("borgh: -start-monsters: no monster named exactly '%s'", name);
		start_monsters_requested += (int)count;
#ifdef BORGH_TEST_JUNK_CONSUMABLES
		const char *fixture = getenv("BORGH_TEST_JUNK_CONSUMABLES");
		if (fixture && streq(fixture, "throw")) {
			if (count != 1 || dist > 11)
				quit("borgh: junk consumables fixture needs one target at range 1..11");
			place_junk_consumables_target(race, (int)dist);
			return;
		}
#endif
		while (count-- > 0) {
			int tries;
			for (tries = 0; tries < 10; tries++) {
				struct loc grid;
				if (scatter_ext(cave, &grid, 1, player->grid, (int)dist, true, square_isempty) == 0)
					break;
				if (place_new_monster(cave, grid, race, true, false, info, ORIGIN_DROP_WIZARD)) {
					start_monsters_placed++;
					break;
				}
			}
		}
	}
}

static void prepare_borg_start(void)
{
	if (junk_consumables_town_fixture()) {
		synthetic_build_apply(opt_start_build, start_build_sha256);
		square_set_feat(cave, player->grid, FEAT_MORE);
		square_memorize(cave, player->grid);
	}
	if (opt_start_depth) {
		bool auto_more = OPT(player, auto_more);
		if (opt_start_depth >= z_info->max_depth)
			quit("borgh: -start-depth must be between 1 and max_depth-1");
		if (opt_start_build) synthetic_build_apply(opt_start_build, start_build_sha256);
		/* Complete the engine's normal level transition before any Borg
		 * initialization or key. Merely changing player->depth leaves
		 * the previous cave active until a later game-loop boundary. */
		player->max_depth = player->recall_depth = opt_start_depth;
		option_set("birth_force_descend", true);
		option_set("birth_no_recall", true);
		dungeon_change_level(player, opt_start_depth);
		option_set("auto_more", true);
		borgh_generate_start();
		event_signal(EVENT_MESSAGE_FLUSH);
		option_set("auto_more", auto_more);
		if (!cave || cave->depth != opt_start_depth || player->upkeep->generate_level)
			quit("borgh: synthetic cave generation did not complete");
		if (opt_start_monsters) place_start_monsters(opt_start_monsters);
		synthetic_start_authorized = true;
	}
	if (opt_load) {
		if (OPT(player, birth_force_descend) && OPT(player, birth_no_recall))
			quit("borgh: loading a synthetic one-way save is not supported");
		/* A missing/dead save must never silently become a birth run. */
		if (!player || player->is_dead || !player->race || !player->class ||
			player->race->ridx != (unsigned)opt_race || player->class->cidx != (unsigned)opt_class)
			quit("borgh: loaded character does not match requested race/class");
		loaded_turn = (long)turn;
		if (opt_turns > 0 && opt_turns <= loaded_turn)
			quit("borgh: turn limit must be after the loaded turn");
		/* load.c restores the saved RNG. Give both implementations the
		 * requested replay seed at exactly the same pre-Borg boundary. */
		if (!opt_keep_load_rng) Rand_state_init(borgh_seed);
		Rand_quick = false;
		/* borg-reborn-rs (#722): the breeder test bed on a real character
		 * (a living save): its monsters are placed after the replay seed,
		 * so the run stays deterministic; only on a dungeon level. */
		if (opt_start_monsters) {
			if (!cave || player->depth == 0)
				quit("borgh: -start-monsters needs the loaded character in the dungeon");
			place_start_monsters(opt_start_monsters);
		}
	}
	if (opt_checkpoint_every) {
		checkpoint_next = ((long)turn / opt_checkpoint_every + 1) * opt_checkpoint_every;
		checkpoint_last_seen = (long)turn;
	}
	/* Skip the "are you sure" prompt for the borg commands */
	player->noscore |= NOSCORE_BORG;

	/* No delays (borg-think-dungeon.c sleeps delay^2 ms per step) */
	player->opts.delay_factor = 0;

	borg_started = true;
}

/* Only this validated frontend request may authorize the core's two birth
 * option exceptions. GUI starts and non-harness core builds return false. */
bool borgh_synthetic_start(void) { return synthetic_start_authorized; }

/**
 * Whether the top line shows the "-more-" prompt of msg_flush().
 */
static bool more_prompt_shown(void)
{
	static const wchar_t more[] = L"-more-";
	int x, i, a;
	wchar_t c;

	if (!Term) return false;
	for (x = 0; x + 6 <= Term->wid; x++) {
		for (i = 0; i < 6; i++) {
			(void)Term_what(x + i, 0, &a, &c);
			if (c != more[i]) break;
		}
		if (i == 6) return true;
	}
	return false;
}

/* ---- terminal hooks ---- */

typedef struct term_data term_data;
struct term_data {
	term t;
};

static term_data td;

static errr term_xtra_borgh(int n, int v)
{
	switch (n) {
	case TERM_XTRA_EVENT:
		/*
		 * Scripted keys first, but only when the game blocks waiting
		 * for a key (polls, e.g. during level generation, would eat
		 * them).
		 */
		if (script_pos < script_len) {
			if (!v) return 0;
			/* Loading can trigger msg_flush() after the splash. Leave
			 * the start marker queued while any such prompt is open. */
			if (((opt_load && script[script_pos] == SCRIPT_START_BORG)
					|| (test_map_export && borg_started))
					&& !inkey_flag && more_prompt_shown()) {
				if (getenv("BORGH_DEBUG"))
					fprintf(stderr, "borgh: acknowledging scripted -more- prompt\n");
				Term_keypress(ESCAPE, 0);
				return 0;
			}
			keycode_t k = script[script_pos++];
			if (getenv("BORGH_DEBUG")) {
				fprintf(stderr, "borgh: feeding script key %d (0x%lx)\n",
					script_pos - 1, (unsigned long)k);
				dump_screen();
			}
			if (k == SCRIPT_START_BORG) {
				prepare_borg_start();
				k = KTRL('Z');
			}
			Term_keypress(k, 0);
			return 0;
		}

		/*
		 * A "-more-" prompt after "Ctrl-Z z" but before the Borg has
		 * returned any key: borg_init() shows it for a borg.txt
		 * warning (e.g. borg_uses_dynamic_calcs), before the keypress
		 * hook is installed.  Answer it with ESC, the same way for
		 * both Borgs.  In a run without such a warning the game never
		 * blocks here (it would have stopped as borg_stopped with no
		 * key), so other key logs are unchanged.
		 */
		if (v && borg_started && !inkey_hack && keys_logged == 0
				&& startup_acks < 16 && more_prompt_shown()) {
			startup_acks++;
			Term_keypress(ESCAPE, 0);
			return 0;
		}

		/* The game wants a real key: nothing more can happen */
		if (v) {
			if (player && player->is_dead)
				borgh_finish("death");
			else if (borg_started && !inkey_hack)
				borgh_finish("borg_stopped");
			else
				borgh_finish("blocked");
		}
		return 0;

	case TERM_XTRA_DELAY:
	case TERM_XTRA_CLEAR:
	case TERM_XTRA_NOISE:
	case TERM_XTRA_FRESH:
	case TERM_XTRA_SHAPE:
	case TERM_XTRA_ALIVE:
	case TERM_XTRA_FLUSH:
	case TERM_XTRA_REACT:
	default:
		return 0;
	}
}

static errr term_curs_borgh(int x, int y) { return 0; }
static errr term_wipe_borgh(int x, int y, int n) { return 0; }
static errr term_text_borgh(int x, int y, int n, int a, const wchar_t *s)
{
	return 0;
}

static void term_data_link(int i)
{
	term *t = &td.t;

	term_init(t, 80, 24, 256);

	t->xtra_hook = term_xtra_borgh;
	t->curs_hook = term_curs_borgh;
	t->wipe_hook = term_wipe_borgh;
	t->text_hook = term_text_borgh;
	t->data = &td;

	Term_activate(t);
	angband_term[i] = t;
}

const char help_borgh[] = "Headless deterministic Borg harness (borg-reborn-rs)";

static long arg_long(int argc, char *argv[], int *i)
{
	if (*i + 1 >= argc) quit_fmt("borgh: missing value for %s", argv[*i]);
	return strtol(argv[++(*i)], NULL, 0);
}

static int capture_level_arg(int argc, char *argv[], int *i)
{
	const char *p;
	int value = 0;
	if (capture_level_seen || ++(*i) >= argc)
		quit("borgh: duplicate/missing -capture-build-level");
	capture_level_seen = true;
	p = argv[*i];
	if (!*p) quit("borgh: invalid capture-build-level");
	for (; *p; ++p) {
		if (*p < '0' || *p > '9') quit("borgh: invalid capture-build-level");
		value = value * 10 + (*p - '0');
		if (value > 15) quit("borgh: capture-build-level must be 1..15");
	}
	if (!value) quit("borgh: capture-build-level must be 1..15");
	return value;
}

extern void borgh_check_pborg_settings(void);

errr init_borgh(int argc, char *argv[])
{
	int i;
	const char *env;
	const char *letters = all_letters_borgh;

	if (argc == 2 && streq(argv[1], "-check-pborg-settings")) {
		borgh_check_pborg_settings();
		quit(NULL);
	}

	gettimeofday(&start_tv, NULL);

	env = getenv("ANGBAND_SEED");
	if (env && *env) {
		borgh_seed = (uint32_t)strtoul(env, NULL, 0);
		borgh_seed_set = true;
	}

	for (i = 1; i < argc; i++) {
		if (streq(argv[i], "-seed")) {
			borgh_seed = (uint32_t)arg_long(argc, argv, &i);
			borgh_seed_set = true;
		} else if (streq(argv[i], "-cborg")) {
			opt_cborg = true;
		} else if (streq(argv[i], "-race")) {
			opt_race = (int)arg_long(argc, argv, &i);
		} else if (streq(argv[i], "-class")) {
			opt_class = (int)arg_long(argc, argv, &i);
		} else if (streq(argv[i], "-turns")) {
			opt_turns = arg_long(argc, argv, &i);
		} else if (streq(argv[i], "-exact-turns")) {
            opt_exact_turns = true;
        } else if (streq(argv[i], "-random-birth")) {
            opt_random_birth = true;
        } else if (streq(argv[i], "-stall-decisions")) {
            opt_stall_decisions = arg_long(argc, argv, &i);
            if (opt_stall_decisions <= 0) quit("borgh: stall decisions must be positive");
        } else if (streq(argv[i], "-keys")) {
			opt_keys = arg_long(argc, argv, &i);
			if (opt_keys <= 0) quit("borgh: key limit must be positive");
		} else if (streq(argv[i], "-clev")) {
			opt_clev = (int)arg_long(argc, argv, &i);
		} else if (streq(argv[i], "-depth")) {
			opt_depth = (int)arg_long(argc, argv, &i);
		} else if (streq(argv[i], "-start-depth")) {
			const char *value;
			unsigned long depth = 0;
			if (opt_start_depth) quit("borgh: duplicate -start-depth");
			if (++i >= argc) quit("borgh: missing value for -start-depth");
			value = argv[i];
			if (!*value) quit("borgh: -start-depth requires a positive decimal integer");
			/* Bound before multiplication: no strtol truncation/overflow or
			 * zero sentinel silently turning a bad request into town birth. */
			for (; *value; ++value) {
				if (*value < '0' || *value > '9' || depth > 999)
					quit("borgh: invalid -start-depth");
				depth = depth * 10 + (*value - '0');
			}
			if (!depth || depth > 999) quit("borgh: invalid -start-depth");
			opt_start_depth = (int)depth;
		} else if (streq(argv[i], "-start-monsters")) {
			if (opt_start_monsters || ++i >= argc) quit("borgh: duplicate/missing -start-monsters");
			opt_start_monsters = argv[i];
		} else if (streq(argv[i], "-start-build")) {
			if (opt_start_build || ++i >= argc) quit("borgh: duplicate/missing -start-build");
			opt_start_build = argv[i];
		} else if (streq(argv[i], "-capture-build")) {
			if (opt_capture_build || ++i >= argc) quit("borgh: duplicate/missing -capture-build");
			opt_capture_build = argv[i];
		} else if (streq(argv[i], "-capture-build-level")) {
			opt_capture_build_level = capture_level_arg(argc, argv, &i);
		} else if (streq(argv[i], "-keylog")) {
			if (++i >= argc) quit("borgh: missing value for -keylog");
			opt_keylog = argv[i];
		} else if (streq(argv[i], "-msglog")) {
			if (++i >= argc) quit("borgh: missing value for -msglog");
			opt_msglog = argv[i];
		} else if (streq(argv[i], "-statelog")) {
			if (++i >= argc) quit("borgh: missing value for -statelog");
			opt_statelog = argv[i];
		} else if (streq(argv[i], "-summary")) {
			if (++i >= argc) quit("borgh: missing value for -summary");
			opt_summary = argv[i];
		} else if (streq(argv[i], "-screen")) {
			opt_screen = true;
		} else if (streq(argv[i], "-menu-letters")) {
			opt_menu_letters = true;
		} else if (streq(argv[i], "-load")) {
			if (++i >= argc) quit("borgh: missing value for -load");
			opt_load = argv[i];
		} else if (streq(argv[i], "-keep-load-rng")) {
			opt_keep_load_rng = true;
		} else if (streq(argv[i], "-save-on-stop")) {
			if (++i >= argc) quit("borgh: missing value for -save-on-stop");
			opt_save = argv[i];
			opt_save_living_only = false;
			opt_save_dead = false;
		} else if (streq(argv[i], "-save-living-on-stop")) {
			if (++i >= argc) quit("borgh: missing value for -save-living-on-stop");
			opt_save = argv[i];
			opt_save_living_only = true;
			opt_save_dead = false;
		} else if (streq(argv[i], "-save-final-state")) {
			if (++i >= argc) quit("borgh: missing value for -save-final-state");
			opt_save = argv[i];
			opt_save_living_only = false;
			opt_save_dead = true;
		} else if (streq(argv[i], "-checkpoint-every")) {
			opt_checkpoint_every = arg_long(argc, argv, &i);
			if (opt_checkpoint_every <= 0 || opt_checkpoint_every > 1000000000L)
				quit("borgh: checkpoint interval must be 1..1000000000");
		} else if (streq(argv[i], "-checkpoint-dir")) {
			if (++i >= argc) quit("borgh: missing value for -checkpoint-dir");
			opt_checkpoint_dir = argv[i];
		} else {
			quit_fmt("borgh: bad argument '%s'", argv[i]);
		}
	}

	/* P3 (#244): native zoo runs have -summary but no run.sh environment.
	 * Put optional parameter/decision diagnostics beside that summary.
	 * Explicit callers keep their chosen BORG_RUN_DIR. No game state/RNG. */
	if (opt_summary && !getenv("BORG_RUN_DIR")) {
		char *dir = string_make(opt_summary);
		char *slash = strrchr(dir, '/');
		if (slash == dir) slash[1] = '\0';
		else if (slash) *slash = '\0';
		else {
			string_free(dir);
			dir = string_make(".");
		}
		if (setenv("BORG_RUN_DIR", dir, 0) != 0)
			quit("borgh: cannot configure run diagnostics directory");
		string_free(dir);
	}

	if (!borgh_seed_set)
		quit("borgh: a seed is required (-seed N or $ANGBAND_SEED)");
	if (opt_race < 0 || opt_race >= 26 || opt_class < 0 || opt_class >= 26)
		quit("borgh: -race and -class must be explicit indices");
	if (opt_random_birth && opt_load) quit("borgh: random birth cannot load a save");
	if (opt_start_depth && (opt_load || opt_random_birth))
		quit("borgh: -start-depth cannot load a save or use random birth");
	if (junk_consumables_town_fixture() && (!opt_start_build || opt_start_depth || opt_load || opt_random_birth))
		quit("borgh: junk consumables town fixture requires a build and explicit town birth");
	if (opt_start_build && !opt_start_depth && !junk_consumables_town_fixture())
		quit("borgh: -start-build requires -start-depth");
	if (opt_start_monsters && !opt_start_depth && !opt_load)
		quit("borgh: -start-monsters requires -start-depth or -load");
	if (!!opt_capture_build != !!opt_capture_build_level || opt_capture_build_level < 0 || opt_capture_build_level > 15)
		quit("borgh: capture-build requires a level between 1 and 15");
	if (opt_capture_build && (opt_start_depth || opt_load || opt_random_birth))
		quit("borgh: build capture requires an ordinary explicit birth run");
	if (opt_exact_turns && opt_turns <= 0) quit("borgh: exact turns requires a positive turn limit");
	if (!!opt_checkpoint_every != !!opt_checkpoint_dir)
		quit("borgh: checkpoint interval and directory are both required");
#ifdef _WIN32
	/* #823 (review of #851): refuse at startup, not at the first capture. */
	if (opt_checkpoint_dir)
		quit("borgh: checkpoints need fork(), not available on Windows");
#endif
	if (opt_checkpoint_dir) {
		char path[1024];
		if (strlen(opt_checkpoint_dir) > sizeof(path) - 128)
			quit("borgh: checkpoint directory path too long");
		path_build(path, sizeof(path), opt_checkpoint_dir, "index.jsonl");
		if (file_exists(path)) quit("borgh: checkpoint index already exists");
		checkpoint_fp = fopen(path, "wb");
		if (!checkpoint_fp) quit("borgh: cannot open checkpoint index");
	}

	if (opt_keylog) {
		static char keylog_buf[1 << 16];
		/* #823: binary, so Windows writes the same bytes (no CRLF). */
		keylog_fp = fopen(opt_keylog, "wb");
		if (!keylog_fp) quit_fmt("borgh: cannot open %s", opt_keylog);
		setvbuf(keylog_fp, keylog_buf, _IOFBF, sizeof(keylog_buf));
	}
	if (opt_msglog) {
		static char msglog_buf[1 << 16];
		msglog_fp = fopen(opt_msglog, "wb");
		if (!msglog_fp) quit_fmt("borgh: cannot open %s", opt_msglog);
		setvbuf(msglog_fp, msglog_buf, _IOFBF, sizeof(msglog_buf));
	}
	if (opt_statelog) {
		static char statelog_buf[1 << 16];
		statelog_fp = fopen(opt_statelog, "wb");
		if (!statelog_fp) quit_fmt("borgh: cannot open %s", opt_statelog);
		setvbuf(statelog_fp, statelog_buf, _IOFBF, sizeof(statelog_buf));
	}

	/* run.sh omits -n for a replay. Never accept a missing fixture. */
	if (opt_load) {
		if (!file_exists(opt_load)) quit("borgh: savefile does not exist");
		if (strlen(opt_load) >= sizeof(savefile)) quit("borgh: savefile path too long");
		my_strcpy(savefile, opt_load, sizeof(savefile));
	} else {
		savefile[0] = '\0';
	}

	/*
	 * Key script.  Mirrors borg_reincarnate_start() for the birth screens
	 * (the first birth has no quickstart screen).
	 */
	script_add(ESCAPE);                       /* pause_line() splash */
	if (!opt_load) {
	if (opt_menu_letters) {
		/* The birth menus' own tags (they skip h, j, k, l) */
		letters = all_letters_nohjkl;
	}
	if (opt_random_birth) script_add('*');
	else if (opt_race) script_add(letters[opt_race]);
	if (!opt_random_birth) script_add(KC_ENTER); /* race: * selects immediately */
	if (opt_random_birth) script_add('*');
	else if (opt_class) script_add(letters[opt_class]);
	if (!opt_random_birth) script_add(KC_ENTER); /* class */
	script_add(KC_ENTER);                     /* point based */
	script_add(KC_ENTER);                     /* accept points */
	script_add(KC_ENTER);                     /* default name */
	script_add('y');                          /* accept history */
	script_add(KC_ENTER);                     /* final confirm */
	}
	script_add(SCRIPT_START_BORG);            /* Ctrl-Z */
	if (opt_cborg) {
		script_add('B');                      /* switch PBorg to CBorg */
		script_add(KTRL('Z'));                /* open CBorg's commands */
	}
	/* #1303: drive the real P-Borg filename prompt before starting play. */
	const char *map_export = getenv("BORGH_TEST_MAP_EXPORT");
	if (map_export) {
		if (opt_cborg) quit("borgh: the map export test requires P-Borg");
		test_map_export = true;
		script_add('l');
		if (streq(map_export, "cancel")) {
			script_add('\\');
			script_add(ESCAPE);
		} else if (streq(map_export, "open_failure")) {
			script_add('.'); /* A directory cannot be opened for writing. */
			script_add(KC_ENTER);
		} else if (streq(map_export, "success")) {
			script_add(KC_ENTER); /* Accept the default archive path. */
		} else {
			quit("borgh: unknown map export test case");
		}
		script_add(KTRL('Z'));
	}
	script_add('z');                          /* activate selected Borg */

	borgh_turn_hook = borgh_tick;
	borgh_key_hook = borgh_key;
	borgh_message_hook = borgh_message;
	borgh_command_hook = opt_checkpoint_dir ? borgh_checkpoint : NULL;

	term_data_link(0);
	return 0;
}

#endif /* USE_BORGH && BORG_HARNESS && ALLOW_BORG */
