# The engine tree (`angband/`)

*What is in `angband/`, where it came from, and how it is re-synced. Task
T1.1 (#4). The tree is never edited in this repository (RULES 9). Harness-only
hooks are added later as their own reviewed changes (T1.4).*

## Provenance

| | |
| --- | --- |
| Source | `angband/` of [diegoami/borg-rs](https://github.com/diegoami/borg-rs) at `e5cb263eed8e` (impact v6, pin of 2026-11-09) |
| Method | `git subtree split --ignore-joins --prefix=angband e5cb263eed8e` → `a2b6b4cbdd2f` (29 commits). `--ignore-joins` is needed because borg-rs added `angband/` as a squashed subtree of upstream `e473720` |
| Import | `git subtree add --prefix=angband a2b6b4cbdd2f` (not squashed, so the 29 commits are ancestors here) |
| Tree id | `5cc1b1cbc2024214dd2d833c928d494abeae1042`, equal to `git rev-parse e5cb263eed8e:angband` in borg-rs. Recorded in `pins.json` (`engine_tree`), and the repository test `engine_tree_matches_its_record` checks it |

## How it differs from upstream `e473720`

8 files added, 40 changed, none removed.

### Added (harness and recording frontends)

`src/main-borgh.c`, `src/main-borgh.h` (the headless `borgh` frontend),
`src/native-replay.c`, `src/native-replay.h`, `src/replay-native-inventory.h`,
`src/replay-native-spells.h`, `src/replay-observations.h` (recording and
replay), `src/tests/borgh-stubs.c`.

### Engine files changed

| File | Change | Guard |
| --- | --- | --- |
| `CMakeLists.txt` | `BORG_IMPL` (C or Rust Borg), sources moved to `target_sources`, harness options | unconditional, same files compiled in a default build (borg-rs `docs/harness.md`) |
| `src/main.h` | declarations of `init_borgh` and `help_borgh` | unconditional, declarations only |
| `src/cmd-wizard.c` | **gameplay correction:** the wizard cure-all also forgets known curses ([neo-angband-tracker#474](https://github.com/diegoami/neo-angband-tracker/issues/474)) | unconditional. It is reached only through wizard mode, which includes the C Borg's cheat-death path |
| `src/game-world.c`, `src/mon-make.c` (fixed date), `src/z-rand.c` (seeding), `src/message.c`, `src/ui-game.c`, `src/ui-input.c` | harness hooks | `#ifdef BORG_HARNESS` |
| `src/main.c`, `src/main-test.c`, `src/savefile.c`, `src/ui-term.c`, `src/z-file.c` (and parts of `message.c`, `ui-game.c`, `ui-input.c`, `z-rand.c`) | recording hooks | `#ifdef BORG_RECORDING` / `USE_BORGH` |

### C Borg files changed (`src/borg/`, 26 files)

These are borg-rs's adopted carries: better-borg-c milestones 2–6
(`docs/adoption/status.json` at the pin: `impact_v6_accepted`), plus the
narrow corrections listed in borg-rs `docs/FORK-CORRECTIONS.md`.
`reference/borg-rs/` has copies of both. The changed files are `borg.c`,
`borg-caution.c`, `borg-danger.c`, `borg-escape.c`, `borg-fight-attack.c`,
`borg-fight-perm.c`, `borg-flow.c`, `borg-flow-glyph.c`, `borg-flow-kill.c`,
`borg-flow-misc.c`, `borg-formulas-calc.c`, `borg-init.c`, `borg-io.c`,
`borg-io.h`, `borg-item-use.c`, `borg-item-wear.c`, `borg-messages.c`,
`borg-power.c`, `borg-projection.c`, `borg-store-sell.c`, `borg-think.c`,
`borg-think-dungeon.c`, `borg-think-dungeon-util.c`, `borg-trait.c`,
`borg-trait-swap.c` and `borg-update.c`.

**So the tree's own C Borg (`BORG_IMPL=c`) is borg-rs's adopted C Borg at
impact v6.** It is neither the original C Borg nor better-borg-c milestone 7,
and it is never a control. The controls `c-pristine` and `c-bbc-m7` overlay
their own pinned `src/borg/` at build time (T1.3, #6).

## Consequences for the controls

`c-pristine` is checked key for key against the zoo's `4.2.6-rs-m0-c`
(borg-rs `aa4ad6c`), whose engine predates #474 and the later harness hooks.
A divergence that reaches the wizard cure-all, which only cheat death uses,
is attributed to #474. Any other divergence is a defect (T1.3).

## Re-syncing

When the refresh report says borg-rs's `angband/` tree moved (RULES 22),
open a dedicated PR:

```sh
git clone https://github.com/diegoami/borg-rs.git /tmp/borg-rs-full   # full clone, not vendor/
git -C /tmp/borg-rs-full subtree split --ignore-joins --prefix=angband NEW_PIN   # prints SPLIT
git fetch /tmp/borg-rs-full SPLIT
git subtree merge --prefix=angband SPLIT -m "Re-sync angband/ from borg-rs NEW_PIN"
git rev-parse HEAD:angband   # must equal: git -C /tmp/borg-rs-full rev-parse NEW_PIN:angband
```

Then update `engine_tree` in `pins.json` and this file. A re-sync starts a new
measurement baseline: results stay with the build they ran on.
