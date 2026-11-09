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

`borg-reborn-rs/src/headers/` (c2rust bindings copied from borg-rs `e5cb263` in M1) was replaced in TE.3 by bindings we generate (`engine/bindings/generate.sh`); nothing of it remains.

`tools/death_registry.py` keeps borg-rs's `tools/borg_deaths.py`'s
registry index (`registry_lock`, `canonical_identity`, `aggregate`), with the
same output, so the existing index regenerates byte for byte. The rest of
the tools copied in T1.6 (recorded in `tools/IMPORTED.json` until then) were
removed in TE.6, and so were borg-rs's `angband/` tree, the `borg-rs/` shim
crate and `harness/measurement/`.
