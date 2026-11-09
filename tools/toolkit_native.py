"""Read-only instrumentation of disposable committed-source Rust exports.

Reference: better-borg-c a4084032485a7c7f29153d0b37ce63dede3e4d5a.
GPL-2.0-or-later or Angband licence. Never changes the checked-in engine.
"""
from pathlib import Path
import shutil

ROOT = Path(__file__).resolve().parents[1]
VERSION = 'borg-rs-toolkit-overlay@1'
FILES = ('tools/toolkit_native.py', 'tools/native-combat.h')


def replace(path, old, new):
    text = path.read_text()
    if text.count(old) != 1:
        raise ValueError('Unsupported frozen instrumentation site: ' + str(path) + ': ' + old[:70])
    path.write_text(text.replace(old, new))


def apply(source):
    src = source / 'angband/src'
    shutil.copy2(ROOT / 'tools/native-combat.h', src / 'native-combat.h')
    replace(src / 'native-replay.c', 'void record_init(void)',
            '#include "native-combat.h"\n\nvoid record_init(void)')
    replace(src / 'native-replay.c', '    if (!native_messages || snapshot_child) return;',
            '    if (!native_messages || snapshot_child) return;\n    combat_message(text, type);')
    replace(src / 'native-replay.c', '    ++frame_count;',
            '    if (screen->wid < 1 || screen->hgt < 1 || screen->wid > 4096 || screen->hgt > 4096 ||\n'
            '        (long)screen->wid * screen->hgt > 1048576) quit("Recording geometry outside resource bounds");\n'
            '    ++frame_count;\n    combat_speed();')
    replace(src / 'native-replay.c', 'width > 255 || height < 1 || height > 255',
            'width > 4096 || height < 1 || height > 4096 || (long)width * height > 1048576')
    replace(src / 'native-replay.c', '    if (!frames || !character_generated || !screen) return;',
            '    if (!frames || snapshot_child || !character_generated || !screen) return;')
    replace(src / 'native-replay.c', '    if (!directory) return;\n    if (native_messages',
            '    if (!directory || snapshot_child) return;\n'
            '    { FILE *streams[] = {inputs, clocks, actions, frames, file_inputs, native_messages, combat_stream};\n'
            '      unsigned i; for (i = 0; i < N_ELEMENTS(streams); i++)\n'
            '        if (streams[i] && (fflush(streams[i]) || ferror(streams[i]))) return; }\n'
            '    if (native_messages')
    replace(src / 'native-replay.h', 'void record_finish(void);',
            'void record_finish(void);\nvoid combat_damage(int requested, int before, int after, const char *cause);')
    replace(src / 'player-util.c', '\tp->chp -= dam;',
            '\tp->chp -= dam;\n#ifdef BORG_RECORDING\n'
            '\t{ extern void combat_damage(int, int, int, const char *); combat_damage(dam, old_chp, p->chp, kb_str); }\n#endif')
    # This optional observer is a process-local control only. A signal handler
    # sets a flag; saving/finalization waits for a living command boundary.
    replace(src / 'main-borgh.c', 'static void borgh_checkpoint(void)\n{',
            'static volatile sig_atomic_t toolkit_timeout;\n'
            'static void toolkit_signal(int sig) { (void)sig; toolkit_timeout = 1; }\n'
            'static void borgh_checkpoint(void)\n{\n'
            '\tif (toolkit_timeout && player && !player->is_dead) borgh_finish("wall_timeout");')
    replace(src / 'main-borgh.c', 'borgh_command_hook = opt_checkpoint_dir ? borgh_checkpoint : NULL;',
            'borgh_command_hook = borgh_checkpoint;\n\tsignal(SIGTERM, toolkit_signal);')
    replace(src / 'main-borgh.c', '\t_exit(0);\n}',
            '#ifdef BORG_RECORDING\n\t{ extern void record_finish(void); record_finish(); }\n#endif\n\t_exit(0);\n}')
    # Native terminal setup validates bounds BEFORE allocation (not only frames).
    replace(src / 'main-test.c', '    term_init(t, getenv("BORG_REPLAY_WIDTH")',
            '    { long w = getenv("BORG_REPLAY_WIDTH") ? atol(getenv("BORG_REPLAY_WIDTH")) : 80;\n'
            '      long h = getenv("BORG_REPLAY_HEIGHT") ? atol(getenv("BORG_REPLAY_HEIGHT")) : 24;\n'
            '      long k = getenv("BORG_REPLAY_KEY_SIZE") ? atol(getenv("BORG_REPLAY_KEY_SIZE")) : 256;\n'
            '      if (w < 1 || h < 1 || w > 4096 || h > 4096 || w*h > 1048576 || k < 1 || k > 65536)\n'
            '        quit("Replay terminal geometry/queue outside resource bounds"); }\n'
            '    term_init(t, getenv("BORG_REPLAY_WIDTH")')

    replace(src / 'main-borgh.c', '\tborg_started = true;',
            r'''
    if (getenv("BORG_TOOLKIT_REPORT")) {
        extern bool dump_save(const char *);
        const char *target = getenv("BORG_TOOLKIT_REPORT");
        long before_turn = turn;
        unsigned long energy = player->total_energy;
        uint32_t rng[RAND_DEG], index = state_i, quick = Rand_value;
        bool is_quick = Rand_quick;
        char path[2048]; FILE *proof;
        if (player->is_dead) quit("Cannot format a dead save");
        memcpy(rng, STATE, sizeof(rng));
        if (!dump_save(target)) quit("Cannot format native character");
        if (turn != before_turn || player->total_energy != energy ||
            state_i != index || Rand_value != quick || Rand_quick != is_quick ||
            memcmp(rng, STATE, sizeof(rng))) quit("Native formatting changed execution state");
        strnfmt(path, sizeof(path), "%s.json", target);
        proof = fopen(path, "w");
        if (!proof) quit("Cannot write formatter proof");
        fprintf(proof, "{\"turn\":%ld,\"energy\":%lu,\"unchanged\":true}\n", before_turn, energy);
        if (fclose(proof)) quit("Cannot finalize formatter proof");
        _exit(0);
    }
\tborg_started = true;'''.replace('\\t', '\t'))

    replace(src / 'main.c', '\tinit_angband();',
            '#ifdef BORG_HARNESS\n'
            '\tif (getenv("BORG_VISIBLE_SEED")) {\n'
            '\t    extern bool borgh_seed_set; extern uint32_t borgh_seed;\n'
            '\t    borgh_seed = (uint32_t)strtoul(getenv("BORG_VISIBLE_SEED"), NULL, 10); borgh_seed_set = true;\n'
            '\t}\n#endif\n\tinit_angband();')
    replace(src / 'native-replay.c', '    if (!inputs) return;',
            '    if (!inputs || snapshot_child) return;')
    replace(src / 'main-borgh.c', 'static uint32_t borg_rng;',
            'static uint32_t borg_rng;\nstatic uint32_t toolkit_rng;\nstatic unsigned long toolkit_decisions;\n'
            'void toolkit_borg_state(FILE *fp) { fprintf(fp, ",\\\"borg_rng\\\":%lu,\\\"borg_decisions\\\":%lu", (unsigned long)toolkit_rng, toolkit_decisions); }')
    replace(src / 'main-borgh.c', 'void borgh_rng(uint32_t rng) { borg_rng = rng; }',
            'void borgh_rng(uint32_t rng) { borg_rng = rng; toolkit_rng = rng; }')
    replace(src / 'main-borgh.c', 'void borgh_decision(uint32_t rng)\n{',
            'void borgh_decision(uint32_t rng)\n{\n    toolkit_rng = rng; ++toolkit_decisions;')
    replace(src / 'native-replay.c', '    replay_extra_observations(full);',
            '    replay_extra_observations(full);\n#ifdef BORG_HARNESS\n'
            '    { extern void toolkit_borg_state(FILE *); toolkit_borg_state(frames); }\n#endif')
    # Headless terminal events originate in the deterministic birth/Borg frontend,
    # which also activates the Borg. Recompute that frontend and compare every
    # event; injecting its keys would skip its required activation side effects.
    replace(src / 'native-replay.c', '        replay_inputs = open_record(original, "events.txt", "r");',
            '        if (!getenv("BORG_TOOLKIT_HEADLESS")) replay_inputs = open_record(original, "events.txt", "r");')
    replace(src / 'native-replay.c', '        setvbuf(replay_inputs, NULL, _IONBF, 0);',
            '        if (replay_inputs) setvbuf(replay_inputs, NULL, _IONBF, 0);')
    replace(src / 'native-replay.c', '    if (replay_inputs) {\n        int ch;',
            '    if (replay_clocks) {\n        int ch;')
    replace(src / 'native-replay.c', '            do { ch = fgetc(streams[i]); } while (ch != EOF && isspace(ch));',
            '            if (!streams[i]) continue;\n            do { ch = fgetc(streams[i]); } while (ch != EOF && isspace(ch));')

    replace(src / 'main-borgh.c', '\tborg_started = true;',
            '\tif (getenv("BORG_TOOLKIT_HEADLESS")) signal(SIGTERM, toolkit_signal);\n\tborg_started = true;')
    replace(src / 'game-world.c', '#include "game-world.h"',
            '#include "game-world.h"\n#ifdef BORG_RECORDING\n#include "native-replay.h"\n#endif')
    replace(src / 'game-world.c', '\t\t\tturn++;',
            '\t\t\tturn++;\n#ifdef BORG_RECORDING\n\t\t\trecord_frame("tick", record_key_count());\n#endif')
    replace(src / 'main-borgh.c', '\tborgh_frame(reason);',
            '\tborgh_frame(reason);\n#ifdef BORG_RECORDING\n'
            '\tif (!streq(reason, "human_takeover")) { extern void record_frame(const char *, long);\n'
            '\t    extern long record_key_count(void); record_frame(reason, record_key_count()); }\n#endif')
    # Format before harness startup changes scoring/delay options. The dump
    # operation checks ticks, energy and complete native RNG and exits here.
    report_path = src / 'main-borgh.c'
    report_text = report_path.read_text()
    begin = report_text.index('    if (getenv("BORG_TOOLKIT_REPORT")) {')
    end = report_text.index('\tif (getenv("BORG_TOOLKIT_HEADLESS")) signal(SIGTERM, toolkit_signal);', begin)
    formatter = report_text[begin:end]
    report_text = report_text[:begin] + report_text[end:]
    marker = '\t/* Skip the "are you sure" prompt for the borg commands */'
    if report_text.count(marker) != 1:
        raise ValueError('Historical formatter boundary changed')
    report_path.write_text(report_text.replace(marker, formatter + marker))
    replace(src / 'main-borgh.c',
            'void toolkit_borg_state(FILE *fp) { fprintf(fp, ",\\\"borg_rng\\\":%lu,\\\"borg_decisions\\\":%lu", (unsigned long)toolkit_rng, toolkit_decisions); }',
            'void toolkit_borg_state(FILE *fp) { fprintf(fp, ",\\\"borg_rng\\\":%lu,\\\"borg_decisions\\\":%lu,\\\"speed\\\":%d,\\\"energy_gain\\\":%d,\\\"move_cost\\\":%d,\\\"energy_available\\\":%d", (unsigned long)toolkit_rng, toolkit_decisions, player->state.speed, turn_energy(player->state.speed), z_info->move_energy, player->energy); }')
    # A native save must be published between top-level commands, never halfway
    # through an item/store prompt. Thresholds are prospective; complete a
    # pending command, then stop before asking the Borg for another action.
    replace(src / 'main-borgh.c', 'static uint32_t toolkit_rng;',
            'static uint32_t toolkit_rng;\nstatic bool toolkit_key_cap;')
    replace(src / 'main-borgh.c', 'if (opt_keys > 0 && keys_logged >= opt_keys) borgh_finish("keys");',
            'if (opt_keys > 0 && keys_logged >= opt_keys) {\n'
            '        if (getenv("BORG_TOOLKIT_HEADLESS")) toolkit_key_cap = true;\n'
            '        else borgh_finish("keys");\n    }')
    replace(src / 'main-borgh.c', 'if (opt_turns > 0 && turn >= opt_turns) borgh_finish("turns");',
            'if (!getenv("BORG_TOOLKIT_HEADLESS") && opt_turns > 0 && turn >= opt_turns) borgh_finish("turns");')
    replace(src / 'main-borgh.c', '\tif (toolkit_timeout && player && !player->is_dead) borgh_finish("wall_timeout");',
            '\tif (toolkit_timeout && player && !player->is_dead) borgh_finish("wall_timeout");\n'
            '\tif (getenv("BORG_TOOLKIT_HEADLESS") && borg_started && player && !player->is_dead) {\n'
            '\t    if (toolkit_key_cap) borgh_finish("keys");\n'
            '\t    if (opt_turns > 0 && turn >= opt_turns) borgh_finish("turns");\n\t}')
