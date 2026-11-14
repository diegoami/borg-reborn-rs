# Provenance

*docs/SPEC.md §2.2: what was copied from another project, once. A copy is
ours from then on: it is changed or deleted as we need, and never kept in
sync. Licence of every source below: GPL-2.0.*

| Ours | Source | Commit | Path there | What we changed |
| --- | --- | --- | --- | --- |
| `engine/angband/` | angband/angband | `e473720` | the whole tree | nothing (byte for byte; `engine/UPSTREAM`) |
| `engine/patches/02`–`07` | diegoami/borg-rs | `286215c` | `angband/src/{game-world,z-rand,mon-make,message,ui-input,ui-game}.c` | only their `BORG_HARNESS` hunks; the `BORG_RECORDING` hunks are dropped |
| `engine/frontend/main-borgh.c`, `main-borgh.h` | diegoami/borg-rs | `286215c` | `angband/src/main-borgh.c`, `main-borgh.h` | trimmed to the contract of docs/SPEC.md §3.5: dropped `-states`, `-frames` and its diagnostics, `-make-fixture`, `-near-morgoth`, `-stop-rune-pattern`, `-cheat-live` (1,247 → 931 lines) |
| `engine/patches/01` | diegoami/borg-rs | `286215c` | `angband/src/main.c`, `main.h` | only the `USE_BORGH` hunks; the `main.h` declarations are guarded here |
| `engine/sources.cmake` | angband/angband | `e473720` | `CMakeLists.txt`, `add_library(OurCoreLib OBJECT …)` | the list only, without the 59 `src/borg/*.c`; `engine/tests/test_build.py` checks it is upstream's minus those |
| `harness/run.sh`, `suite.sh`, `diff`, `identity.sh`, `build.sh`, `suites/smoke`, `suites/full` | diegoami/borg-rs | `e5cb263` | `harness/` | copied in T1.2 (#5) with three adaptations (our crate names in `identity.sh`, the death export in `run.sh`, the Rust overlay); re-founded on our own build in milestone E (TE.4, TE.5): `build.sh` builds `engine/` only, `identity.sh` hashes our inputs, `suite.sh` and `diff` accept only `rust`. The borg-rs logic that remains (the suite runner, the binary checks, the diff of keys, messages and states) is ours to change |
| `borg-reborn-rs/src/engine.rs` | diegoami/borg-rs | `e5cb263` | `borg-rs/src/engine.rs` | copied in T1.5 (#8), then cut to the reads of docs/ENGINE-READS.md and re-based on our generated bindings in TE.3; perception (`perceive`, `analyze`) is ours |
| `borg-core/` | diegoami/borg-rs | `e6d13fc` (tag `4.2.6-impact-v12`; first vendored at `4ed3263`) | `borg-rs/` (the crate), `harness/borg.txt` (as `borg-core/borg.txt`), `angband/src/borg/borg-init.c` and `borg.txt` (as `borg-core/c-reference/`, test data) | vendored for the option-C pilot (#207, docs/REDESIGN.md): package renamed `borg-core` (the library keeps the name `borg_rs`), `rlib` only; its `#[no_mangle] do_cmd_borg` became a plain function the shell calls when `BORG_CORE=borg-rs` (`borg-core/src/engine.rs`, `lib.rs`); three test `include_str!` paths repointed. Unlike the copies above, this one is meant to be **refreshed** from borg-rs at a new pin while the pilot runs. Re-pinned to milestone 12 on 2026-11-14 by re-applying our changes as one patch (docs/PILOT.md *Carrying borg-rs's changes*); our files there: `src/shell.rs` and the hooks marked `borg-reborn-rs (#207)`/`(#208)` |
| `borg-reborn-rs/src/town.rs` | diegoami/borg-rs | `a7711a7` (`feature/town-session-tools2`) | `borg-rs/src/borg.rs:407-462` (the handover), `borg_town_dump.rs` (`dump_town_state`, `state_key`, the schema-3 record), `borg_object_log.rs` | rewritten on our observation (#191): the record's fields and order are borg-rs's; `traits` is `[]` (we have no C-Borg traits), a pack item's value is -1 (we have no value estimate), the quiver is added under `x`; the object log's purchases are paired with the next think's pack instead of a turn window, and it adds `eat`, `throw`, `fire`, `wield`, `sell`, `store` |
| `harness/play.sh` | diegoami/borg-rs | `a7711a7` | `harness/play.sh` | rewritten on our X11 build (`watch-borg.sh`): no `borg.txt` flags, no native recording, no decision dumps; the trace via `BORG_RUN_DIR` |
| `harness/feedback.sh` | diegoami/borg-rs | `a7711a7` | `harness/feedback.sh` | nothing |

`borg-reborn-rs/src/headers/` (c2rust bindings copied from borg-rs `e5cb263` in M1) was replaced in TE.3 by bindings we generate (`engine/bindings/generate.sh`); nothing of it remains.

`tools/death_registry.py` keeps the registry index of `tools/borg_deaths.py`
as T1.6 copied it from borg-rs `e5cb263` (`registry_lock`,
`canonical_identity`, `aggregate`; borg-rs took them from better-borg-c
`f3ced62`, `tools/aggregate-deaths.py`). `index.json` and the README's case
rows regenerate byte for byte; the README's closing instructions name our
tools instead of the removed ones. The rest of
the tools copied in T1.6 (recorded in `tools/IMPORTED.json` until then) were
removed in TE.6, and so were borg-rs's `angband/` tree, the `borg-rs/` shim
crate and `harness/measurement/`.
