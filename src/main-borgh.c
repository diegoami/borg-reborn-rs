/**
 * \file main-borgh.c
 * \brief Headless, deterministic frontend that runs the Borg ("borgh").
 *
 * Not part of upstream Angband; added by the borg-rs project to produce
 * reproducible Borg keypress logs.  Modelled on main-test.c.
 *
 * Copyright (c) 2011 Elly <elly+angband@leptoquark.net> (main-test.c)
 * Copyright (c) 2026 borg-rs contributors
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
 *    class, point-based stats, default name) and then sends ^Z z to start
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
 *   -seed N       seed the game RNG (also settable with $ANGBAND_SEED)
 *   -race N       race index (0-based, as in p_race.txt); -1 = random
 *   -class N      class index (0-based, as in class.txt); -1 = random
 *   -turns N      stop when the game turn reaches N (0 = no limit)
 *   -clev N       stop when the character level reaches N (0 = no limit)
 *   -depth N      stop when the max depth reaches N (0 = no limit)
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
 *   -cheat-live   turn on the game option cheat_live before the Borg starts
 *                 (with borg_cheat_death off, the Borg then answers "Die?"
 *                 with y and reincarnates as a new character), and do not
 *                 stop on death: the run goes on with the new character.
 *                 The summary counts the deaths.
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
#include "main.h"
#include "main-borgh.h"
#include "init.h"
#include "obj-knowledge.h"
#include "obj-gear.h"
#include "obj-make.h"
#include "obj-pile.h"
#include "obj-util.h"
#include "mon-util.h"
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
#include <sys/wait.h>
#include <unistd.h>
#include <errno.h>
#ifdef __linux__
#include <sys/prctl.h>
#endif

#if defined(USE_BORGH) && defined(BORG_HARNESS) && defined(ALLOW_BORG)

extern struct keypress (*inkey_hack)(int flush_first);

/* Options */
static int opt_race = 0;
static int opt_class = 0;
static long opt_turns = 0;
static long opt_keys = 0;
static bool opt_stop_rune_pattern = false;
static int opt_clev = 0;
static int opt_depth = 0;
static const char *opt_keylog = NULL;
static const char *opt_msglog = NULL;
static const char *opt_statelog = NULL;
static const char *opt_summary = NULL;
static bool opt_screen = false;
static bool opt_menu_letters = false;
static bool opt_cheat_live = false;
static const char *opt_load = NULL;
static const char *opt_save = NULL;
static bool opt_save_living_only = false;
static bool opt_keep_load_rng = false;
static bool opt_save_dead = false;
static const char *opt_frames = NULL;
static FILE *frames_fp = NULL;
static unsigned long frame_count;
static unsigned long long previous_details;
static wchar_t previous_chars[24][80];
static int previous_attrs[24][80];
static long loaded_turn = 0;
static int opt_fixture_depth = 0;
static bool opt_near_morgoth = false;
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
static int startup_acks = 0;	/* "-more-" prompts answered before the Borg's first key */
static int deaths = 0;		/* with -cheat-live: deaths seen after a key */
static bool was_dead = false;

/* Scripted keys (before the Borg is running) */
#define SCRIPT_MAX 64
#define SCRIPT_START_BORG 0x7fffffff /* marker: set options, then ^Z */
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

/* Engine-only diagnostics adapted from diegoami/better-borg-c 08a5cd0198b2a2a9ceb426d4cc4704fd5364d519
 * tools/native-replay.c; original dual GPL/Angband licensing is preserved.
 * C-only Borg fields are intentionally omitted; nothing is fed back to AI. */
static unsigned long long mix_number(unsigned long long hash, long long value)
{
    unsigned long long bits = (unsigned long long)value;
    int i;
    for (i = 0; i < 8; i++) {
        hash = (hash ^ (bits & 255)) * 1099511628211ULL;
        bits >>= 8;
    }
    return hash;
}

static unsigned long long diagnostic_hash(void)
{
    const struct object *obj;
    unsigned long long hash = 14695981039346656037ULL;
    int i;
    for (i = 0; i < TMD_MAX; i++) hash = mix_number(hash, player->timed[i]);
    for (obj = player->gear; obj; obj = obj->next) {
        hash = mix_number(hash, obj->kind ? obj->kind->kidx : 0);
        hash = mix_number(hash, obj->known && obj->known->kind ? obj->known->kind->kidx : 0);
        hash = mix_number(hash, obj->kind ? obj->kind->aware : 0);
        hash = mix_number(hash, obj->number); hash = mix_number(hash, obj->weight);
        hash = mix_number(hash, obj->pval); hash = mix_number(hash, obj->timeout);
        hash = mix_number(hash, obj->to_h); hash = mix_number(hash, obj->to_d);
        hash = mix_number(hash, obj->to_a); hash = mix_number(hash, obj->note);
    }
    if (cave) for (i = 1; i < cave->mon_max; i++) {
        const struct monster *mon = &cave->monsters[i];
        if (!mon->race) continue;
        hash = mix_number(hash, i); hash = mix_number(hash, mon->race->ridx);
        hash = mix_number(hash, mon->grid.x); hash = mix_number(hash, mon->grid.y);
        hash = mix_number(hash, mon->hp); hash = mix_number(hash, mon->maxhp);
        hash = mix_number(hash, mon->mspeed);
        hash = mix_number(hash, mflag_has(mon->mflag, MFLAG_VISIBLE));
    }
    return hash;
}

static void json_string(const char *text)
{
    const unsigned char *p = (const unsigned char *)(text ? text : "");
    fputc('"', frames_fp);
    for (; *p; p++) {
        if (*p == '"' || *p == '\\') fprintf(frames_fp, "\\%c", *p);
        else if (*p < 32) fprintf(frames_fp, "\\u%04x", *p);
        else fputc(*p, frames_fp);
    }
    fputc('"', frames_fp);
}

static void diagnostics(void)
{
    const struct object *obj;
    int comma = 0, i;
    fprintf(frames_fp, "{\"scope\":\"diagnostic actual state; never supplied to Borg inputs\",\"gear\":[");
    for (obj = player->gear; obj; obj = obj->next) {
        fprintf(frames_fp, "%s{\"actual_kind\":", comma ? "," : ""); comma = 1;
        json_string(obj->kind ? obj->kind->name : "");
        fprintf(frames_fp, ",\"known_kind\":");
        json_string(obj->known && obj->known->kind ? obj->known->kind->name : "");
        fprintf(frames_fp, ",\"aware\":%d,\"quantity\":%d,\"weight\":%d,\"pval\":%d,\"timeout\":%d,\"to_h\":%d,\"to_d\":%d,\"to_a\":%d,\"inscription\":",
            obj->kind ? obj->kind->aware : 0, obj->number, obj->weight, obj->pval,
            obj->timeout, obj->to_h, obj->to_d, obj->to_a);
        json_string(quark_str(obj->note)); fprintf(frames_fp, "}");
    }
    fprintf(frames_fp, "],\"monsters\":["); comma = 0;
    if (cave) for (i = 1; i < cave->mon_max; i++) {
        const struct monster *mon = &cave->monsters[i];
        if (!mon->race) continue;
        fprintf(frames_fp, "%s{\"id\":%d,\"name\":", comma ? "," : "", i); comma = 1;
        json_string(mon->race->name);
        fprintf(frames_fp, ",\"x\":%d,\"y\":%d,\"hp\":%d,\"max_hp\":%d,\"speed\":%d,\"visible\":%d}",
            mon->grid.x, mon->grid.y, mon->hp, mon->maxhp, mon->mspeed,
            mflag_has(mon->mflag, MFLAG_VISIBLE));
    }
    fprintf(frames_fp, "],\"timed\":{"); comma = 0;
    for (i = 0; i < TMD_MAX; i++) if (player->timed[i]) {
        if (comma) fprintf(frames_fp, ","); comma = 1;
        json_string(timed_effects[i].name);
        fprintf(frames_fp, ":%d", player->timed[i]);
    }
    fprintf(frames_fp, "},\"borg\":{}}");
}

#define REPLAY_OBSERVATION_STREAM frames_fp
#define REPLAY_ADD_VITALS
#include "replay-observations.h"
#undef REPLAY_ADD_VITALS
#undef REPLAY_OBSERVATION_STREAM

/* Observation only. Compatible terminal-frame schema with the imported viewer;
 * recorded headless runs replay the original Rust Borg from their origin. */
static void borgh_frame(const char *phase)
{
	int x, y, attr, color;
	wchar_t ch;
	bool full, first = true;
	if (!frames_fp || !Term || !player) return;
	if (Term->wid != 80 || Term->hgt != 24) quit("borgh: unsupported recording geometry");
	frame_count++;
	full = frame_count == 1 || frame_count % 100 == 0;
	fprintf(frames_fp, "{\"index\":%lu,\"turn\":%ld,\"phase\":", frame_count, (long)turn);
	json_str(frames_fp, phase);
	fprintf(frames_fp, ",\"width\":80,\"height\":24,\"full\":%s,\"level\":%d,\"depth\":%d,\"hp\":%d,\"sp\":%d,\"x\":%d,\"y\":%d,\"cells\":[",
		full ? "true" : "false", player->lev, player->depth, player->chp, player->csp, player->grid.x, player->grid.y);
	for (y = 0; y < 24; y++) for (x = 0; x < 80; x++) {
		(void)Term_what(x, y, &attr, &ch);
		if (full || ch != previous_chars[y][x] || attr != previous_attrs[y][x]) {
			fprintf(frames_fp, "%s[%d,%lu,%d]", first ? "" : ",", y * 80 + x, (unsigned long)ch, attr);
			first = false;
		}
		previous_chars[y][x] = ch;
		previous_attrs[y][x] = attr;
	}
	fputs("],\"details\":", frames_fp);
	{
		unsigned long long hash = diagnostic_hash();
		if (full || hash != previous_details) diagnostics();
		else fputs("null", frames_fp);
		previous_details = hash;
	}
	fputs(",\"palette\":", frames_fp);
	if (full) {
		fputc('[', frames_fp);
		for (color = 0; color < MAX_COLORS; color++) fprintf(frames_fp, "%s[%d,%d,%d]", color ? "," : "",
			(int)angband_color_table[color][1], (int)angband_color_table[color][2], (int)angband_color_table[color][3]);
		fputc(']', frames_fp);
	} else fputs("null", frames_fp);
	replay_extra_observations(full);
	fputs("}\n", frames_fp);
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

	if (finished) return;
	finished = true;
	borgh_frame(reason);
	if (frames_fp) {
		if (ferror(frames_fp)) io_error = true;
		if (fclose(frames_fp)) io_error = true;
		frames_fp = NULL;
	}

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
		fp = fopen(opt_summary, "w");
		if (!fp) fp = stdout;
	}
	fprintf(fp, "{\n");
	fprintf(fp, "  \"seed\": %lu,\n", (unsigned long)borgh_seed);
	if (opt_load)
		fprintf(fp, "  \"loaded\": true,\n  \"loaded_turn\": %ld,\n", loaded_turn);
	fprintf(fp, "  \"race\": ");
	json_str(fp, (player && player->race) ? player->race->name : "");
	fprintf(fp, ",\n  \"race_idx\": %d,\n", opt_race);
	fprintf(fp, "  \"class\": ");
	json_str(fp, (player && player->class) ? player->class->name : "");
	fprintf(fp, ",\n  \"class_idx\": %d,\n", opt_class);
	if (opt_menu_letters) fprintf(fp, "  \"menu_letters\": true,\n");
	if (opt_cheat_live)
		fprintf(fp, "  \"cheat_live\": true,\n  \"deaths\": %d,\n", deaths);
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

	/* Do not save; do not run the normal quit hooks */
	_exit(0);
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
	keys_logged++;
	if (keylog_fp) {
		fprintf(keylog_fp, "%ld %d %lx %x\n", (long)turn, (int)k.type,
			(unsigned long)k.code, (unsigned)k.mods);
	}
	borgh_state();
	borgh_frame("key");

	/* Stop conditions */
	if (opt_keys > 0 && keys_logged >= opt_keys) borgh_finish("keys");
	if (opt_stop_rune_pattern && player && cave && !player->is_dead) {
		int dx, dy, runes = 0;
		for (dy = -2; dy <= 2; dy++) for (dx = -2; dx <= 2; dx++) {
			struct loc grid = loc(player->grid.x + dx, player->grid.y + dy);
			if ((dx || dy) && square_in_bounds(cave, grid) && square_iswarded(cave, grid)) runes++;
		}
		if (runes == 24) borgh_finish("rune_pattern");
	}
	if (opt_turns > 0 && turn >= opt_turns) borgh_finish("turns");
	if (opt_cheat_live) {
		/* Count each death once; the Borg plays on as a new character */
		bool dead = player && player->is_dead;
		if (dead && !was_dead) deaths++;
		was_dead = dead;
	} else if (player && player->is_dead) borgh_finish("death");
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

/**
 * Set up the Borg start (called just before ^Z is fed to the game).
 */
/* Wizard-created fixtures are setup data, never Borg behaviour changes.
 * Use the engine's ordinary object, equipment and level-generation APIs. */
static void fixture_carry(struct object *obj, bool equip)
{
	obj->origin = ORIGIN_CHEAT;
	obj->known = object_new();
	object_set_base_known(player, obj);
	object_flavor_aware(player, obj);
	obj->known->notice |= OBJ_NOTICE_ASSESSED;
	player_know_object(player, obj);
	obj->known->effect = obj->effect;
	inven_carry(player, obj, false, false);
	if (equip) inven_wield(obj, wield_slot(obj));
}

static void make_fixture(void)
{
	static const char *arts[] = {
		"'Ringil'", "of Fëanor", "of Fingolfin", "of Thorin",
		"of Dor-Lómin", "of Celeborn", "of Lúthien", "'Narya'",
		"'Elessar'", "of Elendil"
	};
	static const struct { int tval; const char *name; int number; } kit[] = {
		{ TV_POTION, "Healing", 30 }, { TV_POTION, "*Healing*", 20 },
		{ TV_POTION, "Life", 5 }, { TV_POTION, "Speed", 20 },
		{ TV_SCROLL, "Phase Door", 30 }, { TV_SCROLL, "Teleportation", 20 },
		{ TV_SCROLL, "Banishment", 10 }, { TV_SCROLL, "Mass Banishment", 5 },
		{ TV_SCROLL, "Word of Recall", 5 }, { TV_FOOD, "Ration of Food", 10 }
	};
	size_t i;
	bool more = OPT(player, auto_more);
	option_set("auto_more", true);
	player->noscore |= NOSCORE_WIZARD;
	do_cmd_wiz_advance(NULL);
	player_learn_all_runes(player);
	for (i = 0; i < N_ELEMENTS(arts); i++) {
		int j;
		struct artifact *art = NULL;
		struct object *obj;
		for (j = 1; j < z_info->a_max; j++)
			if (a_info[j].name && streq(a_info[j].name, arts[i])) { art = &a_info[j]; break; }
		if (!art) quit_fmt("borgh: fixture artifact not found: %s", arts[i]);
		obj = object_new();
		object_prep(obj, lookup_kind(art->tval, art->sval), art->alloc_min, RANDOMISE);
		obj->artifact = art;
		copy_artifact_data(obj, art);
		mark_artifact_created(art, true);
		fixture_carry(obj, true);
	}
	for (i = 0; i < N_ELEMENTS(kit); i++) {
		struct object *obj = object_new();
		int sval = lookup_sval(kit[i].tval, kit[i].name);
		struct object_kind *kind = lookup_kind(kit[i].tval, sval);
		if (!kind) quit_fmt("borgh: fixture kind not found: %s", kit[i].name);
		object_prep(obj, kind, opt_fixture_depth, RANDOMISE);
		obj->number = kit[i].number;
		fixture_carry(obj, false);
	}
	/* Sauron must be dead for the depth-100 quest to be reachable. */
	if (opt_fixture_depth == 100) lookup_monster("Sauron, the Sorcerer")->max_num = 0;
	dungeon_change_level(player, opt_fixture_depth);
	prepare_next_level(player);
	player->upkeep->generate_level = false;
	on_new_level();
	if (opt_near_morgoth) {
		struct monster *boss = NULL;
		int j, dx, dy;
		bool placed = false;
		for (j = 1; j < cave_monster_max(cave); j++) {
			struct monster *mon = cave_monster(cave, j);
			if (mon->race && streq(mon->race->name, "Morgoth, Lord of Darkness")) { boss = mon; break; }
		}
		if (!boss) quit("borgh: Morgoth not generated in fixture");
		for (dy = -1; dy <= 1 && !placed; dy++) for (dx = -1; dx <= 1 && !placed; dx++) {
			struct loc grid = loc(boss->grid.x + dx, boss->grid.y + dy);
			if (square_in_bounds_fully(cave, grid) && square_isempty(cave, grid)) {
				monster_swap(player->grid, grid);
				monster_wake(boss, false, 100);
				placed = true;
			}
		}
		if (!placed) quit("borgh: no empty square next to Morgoth");
	}
	player->upkeep->update |= PU_BONUS | PU_HP | PU_MANA | PU_INVEN | PU_UPDATE_VIEW | PU_MONSTERS;
	handle_stuff(player);
	player->chp = player->mhp;
	player->csp = player->msp;
	player->upkeep->energy_use = 0;
	option_set("auto_more", more);
	borgh_finish("fixture");
}

static void prepare_borg_start(void)
{
	if (opt_fixture_depth) make_fixture();
	if (opt_load) {
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
	}
	if (opt_checkpoint_every) {
		checkpoint_next = ((long)turn / opt_checkpoint_every + 1) * opt_checkpoint_every;
		checkpoint_last_seen = (long)turn;
	}
	/* Skip the "are you sure" prompt for the borg commands */
	player->noscore |= NOSCORE_BORG;

	/* No delays (borg-think-dungeon.c sleeps delay^2 ms per step) */
	player->opts.delay_factor = 0;

	if (opt_cheat_live) option_set("cheat_live", true);

	borg_started = true;
}

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
		 * A "-more-" prompt after "^Z z" but before the Borg has
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
			if (player && player->is_dead && !opt_cheat_live)
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

const char help_borgh[] = "Headless deterministic Borg harness (borg-rs)";

static long arg_long(int argc, char *argv[], int *i)
{
	if (*i + 1 >= argc) quit_fmt("borgh: missing value for %s", argv[*i]);
	return strtol(argv[++(*i)], NULL, 0);
}

errr init_borgh(int argc, char *argv[])
{
	int i;
	const char *env;
	const char *letters = all_letters_borgh;

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
		} else if (streq(argv[i], "-race")) {
			opt_race = (int)arg_long(argc, argv, &i);
		} else if (streq(argv[i], "-class")) {
			opt_class = (int)arg_long(argc, argv, &i);
		} else if (streq(argv[i], "-turns")) {
			opt_turns = arg_long(argc, argv, &i);
		} else if (streq(argv[i], "-keys")) {
			opt_keys = arg_long(argc, argv, &i);
			if (opt_keys <= 0) quit("borgh: key limit must be positive");
		} else if (streq(argv[i], "-stop-rune-pattern")) {
			opt_stop_rune_pattern = true;
		} else if (streq(argv[i], "-clev")) {
			opt_clev = (int)arg_long(argc, argv, &i);
		} else if (streq(argv[i], "-depth")) {
			opt_depth = (int)arg_long(argc, argv, &i);
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
		} else if (streq(argv[i], "-cheat-live")) {
			opt_cheat_live = true;
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
		} else if (streq(argv[i], "-frames")) {
			if (++i >= argc) quit("borgh: missing value for -frames");
			opt_frames = argv[i];
		} else if (streq(argv[i], "-make-fixture")) {
			opt_fixture_depth = (int)arg_long(argc, argv, &i);
		} else if (streq(argv[i], "-near-morgoth")) {
			opt_near_morgoth = true;
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

	if (!borgh_seed_set)
		quit("borgh: a seed is required (-seed N or $ANGBAND_SEED)");
	if (opt_frames) {
		frames_fp = fopen(opt_frames, "w");
		if (!frames_fp) quit("borgh: cannot open frame log");
	}
	if (opt_fixture_depth && (opt_fixture_depth < 1 || opt_fixture_depth > 100 || !opt_save || opt_load))
		quit("borgh: fixture requires depth 1-100, -save-on-stop, and no -load");
	if (opt_near_morgoth && opt_fixture_depth != 100)
		quit("borgh: -near-morgoth requires -make-fixture 100");
	if (opt_race < 0 || opt_race >= 26 || opt_class < 0 || opt_class >= 26)
		quit("borgh: -race and -class must be explicit indices");
	if (!!opt_checkpoint_every != !!opt_checkpoint_dir)
		quit("borgh: checkpoint interval and directory are both required");
	if (opt_checkpoint_dir) {
		char path[1024];
		if (strlen(opt_checkpoint_dir) > sizeof(path) - 128)
			quit("borgh: checkpoint directory path too long");
		path_build(path, sizeof(path), opt_checkpoint_dir, "index.jsonl");
		if (file_exists(path)) quit("borgh: checkpoint index already exists");
		checkpoint_fp = fopen(path, "w");
		if (!checkpoint_fp) quit("borgh: cannot open checkpoint index");
	}

	if (opt_keylog) {
		static char keylog_buf[1 << 16];
		keylog_fp = fopen(opt_keylog, "w");
		if (!keylog_fp) quit_fmt("borgh: cannot open %s", opt_keylog);
		setvbuf(keylog_fp, keylog_buf, _IOFBF, sizeof(keylog_buf));
	}
	if (opt_msglog) {
		static char msglog_buf[1 << 16];
		msglog_fp = fopen(opt_msglog, "w");
		if (!msglog_fp) quit_fmt("borgh: cannot open %s", opt_msglog);
		setvbuf(msglog_fp, msglog_buf, _IOFBF, sizeof(msglog_buf));
	}
	if (opt_statelog) {
		static char statelog_buf[1 << 16];
		statelog_fp = fopen(opt_statelog, "w");
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
	if (opt_race) script_add(letters[opt_race]);
	script_add(KC_ENTER);                     /* race */
	if (opt_class) script_add(letters[opt_class]);
	script_add(KC_ENTER);                     /* class */
	script_add(KC_ENTER);                     /* point based */
	script_add(KC_ENTER);                     /* accept points */
	script_add(KC_ENTER);                     /* default name */
	script_add('y');                          /* accept history */
	script_add(KC_ENTER);                     /* final confirm */
	}
	script_add(SCRIPT_START_BORG);            /* ^Z */
	script_add('z');                          /* activate borg */

	borgh_key_hook = borgh_key;
	borgh_message_hook = borgh_message;
	borgh_command_hook = opt_checkpoint_dir ? borgh_checkpoint : NULL;

	term_data_link(0);
	return 0;
}

#endif /* USE_BORGH && BORG_HARNESS && ALLOW_BORG */
