# Provenance

*docs/SPEC.md §2.2: what was copied from another project, once. A copy is
ours from then on: it is changed or deleted as we need, and never kept in
sync. Licence of every source below: GPL-2.0.*

| Ours | Source | Commit | Path there | What we changed |
| --- | --- | --- | --- | --- |
| `engine/angband/` | angband/angband | `e473720` | the whole tree | nothing (byte for byte; `engine/UPSTREAM`) |
| `engine/patches/02`–`07` | diegoami/borg-rs | `286215c` | `angband/src/{game-world,z-rand,mon-make,message,ui-input,ui-game}.c` | only their `BORG_HARNESS` hunks; the `BORG_RECORDING` hunks are dropped |
| `engine/patches/01` | diegoami/borg-rs | `286215c` | `angband/src/main.c`, `main.h` | only the `USE_BORGH` hunks; the `main.h` declarations are guarded here |

The files copied before docs/SPEC.md took effect are listed in
`tools/IMPORTED.json` and in the T1.6 notes of `tools/README.md`. TE.6 moves
the ones kept into this table.
