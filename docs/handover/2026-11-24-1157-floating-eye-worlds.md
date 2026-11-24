## 2026-11-24 — Floating-eye worlds (#1157)

Added `eyes-x4` and `eyes-x8`. The generator keeps the floating eye at
its original depth and rarity and multiplies the rarity of other races
by the configured weight. The engine computes allocation weights as
integer `100 / rarity` (`engine/angband/src/mon-make.c`).

The harness generates each world from the built game's `lib/` directory.
The generator reports one favoured and 623 other races. Using the source
`lib/` for this check includes `gamedata/Makefile`, which CMake does not
stage. The harness hashes are below.

| World | Config SHA-256 | Game-data SHA-256 |
|---|---|---|
| eyes-x4 | `eb794380d254dd12d33df88d134291cab94a4b9e97cea01638adfbe4dc4e07ef` | `bdc3d0e57e530977acdca3cdbf10689c64be9802ff495e37838c597134a392d9` |
| eyes-x8 | `e5adb0a0e197568cb08aa193df0794aa5c97df27c380ccb553f8b3ab7d450094` | `3919ce1b227d8c777b3882005e2982497973a7933e1a1c28a829c6a47cd463af` |

### What is left

[Zoo #760](https://github.com/diegoami/borg-zoo/issues/760) pairs the
shipped borg-rs profile with an arm that changes only
`escape_paralyser_fight_lost`, separately in eyes-x8 and the normal world.
Walk-away stays on. The pilot records known, awake paralyser contacts and
whether a contact changes the Phase decision.

The grey mold and other unlisted stationary races are downweighted in
both eye worlds. A separate comparison for `caution_remember_never_move`
in #1150 needs an unlisted stationary race and a reset or expiry. Use the
normal world or `molds-x8` if that comparison moves to Zoo.

### For the owner

Dispatch the paired pilot when ready. A retained changed action is the
evidence requested before sizing a broader comparison.
