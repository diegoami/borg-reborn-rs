# The engine (`engine/`)

*docs/SPEC.md §3. Task TE.1 (#71) makes the tree and the patch series; the
frontend (TE.2), bindings (TE.3) and build (TE.4) follow.*

## The tree: `engine/angband/`

Upstream Angband at `e473720ed88a0f0c7ca66815cef3000eb4a38bc3`,
**byte for byte** (`engine/UPSTREAM`). It was extracted from the upstream
repository (`git archive` of that commit), not from borg-rs. Its tree id,
`99a0549aaba851809eaf2c9117351b85d3788945`, equals the commit's tree.
`engine/tests/test_engine.py` checks that at `HEAD` and refuses local
changes. The tree is never edited: changes are patches applied to a copy at
build time (`engine/apply-patches.sh DEST`).

One file, `lib/user/save/Makefile`, is ignored by upstream's own
`.gitignore` but tracked upstream, so it is force-added here too.

Today's `angband/` (borg-rs's tree) stays until TE.4 switches the build to
`engine/`. Then it goes.

## The patch series: `engine/patches/`

Each patch says why it exists, and each guard is `BORG_HARNESS` (the
harness hooks) or `USE_BORGH` (the frontend mode). With both undefined,
every patched file is upstream's code. There is one declared exception, and
the test checks the rest:

| Patch | Files | What |
| --- | --- | --- |
| `01-frontend-registration` | `main.c`, `main.h` | registers mode `borgh` in upstream's module table and declares `init_borgh`, `help_borgh` |
| `02-turn-callback` | `game-world.c` | `borgh_turn_hook()` after every game turn: the exact turn boundary |
| `03-seeding` | `z-rand.c` | `Rand_init()` seeds from `borgh_seed` when the harness sets one |
| `04-fixed-date` | `mon-make.c` | the time holiday monsters read is 0 when seeded |
| `05-message-log` | `message.c` | `borgh_message_hook()` sees every `message_add()`, repeats included |
| `06-key-log` | `ui-input.c` | `borgh_key_hook()` sees every key `inkey_hack` returns |
| `07-command-boundary` | `ui-game.c` | `borgh_command_hook()` at each game-command boundary |

**The declared exception:** `04-fixed-date` splits
`struct tm *date = localtime(&cur_time);` into a declaration and an
assignment, so that the guarded line can go between them. With the guards
off, the code is the same.

The hook variables and the header the patches include (`main-borgh.h`)
belong to our frontend (TE.2). The hooks come from borg-rs's tree
(docs/PROVENANCE.md). Its other changes to the engine are not carried: the
recording hooks (`BORG_RECORDING` in 8 files), the wizard cure-all
correction (`cmd-wizard.c`, reachable only by cheat death, which D5
excludes), its CMake changes, its recording frontends, and its C Borg
carries in `src/borg/`.

## Determinism

The seed, the fixed date and the environment (`TZ=UTC`, `LANG` and
`LC_ALL` set to `C.UTF-8`: the date is turned into a calendar date in the
run's time zone) fix a run (docs/SPEC.md §4). The game options the Borg
sets are listed here once they are settled (TE.5).
