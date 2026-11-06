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
 *   -summary FILE summary JSON (default: stdout)
 *   -screen       dump the screen to stderr when the run ends unexpectedly
 *   -menu-letters pick race and class with the birth menu's own letters
 *                 (all_letters_nohjkl).  Without it the script sends
 *                 "abc..z"[index], which skips no letters, so an index >= 7
 *                 picks a different entry (kept as the default so the
 *                 existing reference logs stay valid).
 */

#include "angband.h"
#include "buildid.h"
#include "game-world.h"
#include "main.h"
#include "main-borgh.h"
#include "player.h"
#include "player-calcs.h"
#include "ui-game.h"
#include "ui-menu.h"
#include "ui-term.h"

#include <sys/time.h>

#if defined(USE_BORGH) && defined(BORG_HARNESS) && defined(ALLOW_BORG)

extern struct keypress (*inkey_hack)(int flush_first);

/* Options */
static int opt_race = 0;
static int opt_class = 0;
static long opt_turns = 0;
static int opt_clev = 0;
static int opt_depth = 0;
static const char *opt_keylog = NULL;
static const char *opt_summary = NULL;
static bool opt_screen = false;
static bool opt_menu_letters = false;

/* State */
static FILE *keylog_fp = NULL;
static unsigned long keys_logged = 0;
static struct timeval start_tv;
static bool finished = false;
static bool borg_started = false;
static int startup_acks = 0;	/* "-more-" prompts answered before the Borg's first key */

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

	gettimeofday(&now, NULL);
	secs = (now.tv_sec - start_tv.tv_sec)
		+ (now.tv_usec - start_tv.tv_usec) / 1e6;

	if (keylog_fp) {
		if (ferror(keylog_fp)) io_error = true;
		if (fclose(keylog_fp) != 0) io_error = true;
		keylog_fp = NULL;
	}

	if (opt_screen && !streq(reason, "turns")) dump_screen();

	if (opt_summary) {
		fp = fopen(opt_summary, "w");
		if (!fp) fp = stdout;
	}
	fprintf(fp, "{\n");
	fprintf(fp, "  \"seed\": %lu,\n", (unsigned long)borgh_seed);
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
	fprintf(fp, "  \"wall_seconds\": %.3f\n", secs);
	fprintf(fp, "}\n");
	if (fp != stdout) {
		if (ferror(fp)) io_error = true;
		if (fclose(fp) != 0) io_error = true;
	}
	fflush(stdout);

	/* A short key log or summary must not pass as a good run */
	if (io_error) {
		fprintf(stderr, "borgh: error writing the key log or summary\n");
		_exit(3);
	}

	/* Do not save; do not run the normal quit hooks */
	_exit(0);
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

	/* Stop conditions */
	if (opt_turns > 0 && turn >= opt_turns) borgh_finish("turns");
	if (player && player->is_dead) borgh_finish("death");
	if (opt_clev > 0 && player && player->lev >= opt_clev)
		borgh_finish("clevel");
	if (opt_depth > 0 && player && player->max_depth >= opt_depth)
		borgh_finish("depth");
}

/**
 * Set up the Borg start (called just before ^Z is fed to the game).
 */
static void prepare_borg_start(void)
{
	/* Skip the "are you sure" prompt for the borg commands */
	player->noscore |= NOSCORE_BORG;

	/* No delays (borg-think-dungeon.c sleeps delay^2 ms per step) */
	player->opts.delay_factor = 0;

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
		} else if (streq(argv[i], "-clev")) {
			opt_clev = (int)arg_long(argc, argv, &i);
		} else if (streq(argv[i], "-depth")) {
			opt_depth = (int)arg_long(argc, argv, &i);
		} else if (streq(argv[i], "-keylog")) {
			if (++i >= argc) quit("borgh: missing value for -keylog");
			opt_keylog = argv[i];
		} else if (streq(argv[i], "-summary")) {
			if (++i >= argc) quit("borgh: missing value for -summary");
			opt_summary = argv[i];
		} else if (streq(argv[i], "-screen")) {
			opt_screen = true;
		} else if (streq(argv[i], "-menu-letters")) {
			opt_menu_letters = true;
		} else {
			quit_fmt("borgh: bad argument '%s'", argv[i]);
		}
	}

	if (!borgh_seed_set)
		quit("borgh: a seed is required (-seed N or $ANGBAND_SEED)");
	if (opt_race < 0 || opt_race >= 26 || opt_class < 0 || opt_class >= 26)
		quit("borgh: -race and -class must be explicit indices");

	if (opt_keylog) {
		static char keylog_buf[1 << 16];
		keylog_fp = fopen(opt_keylog, "w");
		if (!keylog_fp) quit_fmt("borgh: cannot open %s", opt_keylog);
		setvbuf(keylog_fp, keylog_buf, _IOFBF, sizeof(keylog_buf));
	}

	/* No savefile from main.c's default: always a new character */
	savefile[0] = '\0';

	/*
	 * Key script.  Mirrors borg_reincarnate_start() for the birth screens
	 * (the first birth has no quickstart screen).
	 */
	script_add(ESCAPE);                       /* pause_line() splash */
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
	script_add(SCRIPT_START_BORG);            /* ^Z */
	script_add('z');                          /* activate borg */

	borgh_key_hook = borgh_key;

	term_data_link(0);
	return 0;
}

#endif /* USE_BORGH && BORG_HARNESS && ALLOW_BORG */
