# Stair-return-report carry: verified product scenario, ready for adoption

Consumer [167](https://github.com/diegoami/better-borg-c/issues/167), canonical
source [230](https://github.com/diegoami/neo-angband-tracker/issues/230)
(stairreturn-230). Fix applied as
`fb16560054b44899c50b8aec5ce1a2b0e35c0533` (import of source PR393
`db0d342ecf9fae2e129740d50d3bb42498f7ac0e`, patch
`patches/0041-stair-return-report.patch`).

## Defect

In `borg_think_dungeon_light()` the out-of-light "Testing for stairs"
branch queued `borg_keypress('<')` but fell through without reporting that
it acted, so the think loop stopped queueing further commands behind the
already-queued stair key (the key itself stays queued; nothing is
discarded). The correction adds the missing `return true;`. No memory
error; no death or survival claim.

## Pins and layered provenance

Before build: forty-carry `2c3d8172c38bd83090f6869f775893b31216bf13`
(retained tree `../enlightenment`, normal and ASan cores, gate recorded at
that commit, source-identical to merged main `b8095941f` over
src/lib/CMakeLists.txt/cmake — the concurrent documentation merges changed
nothing under the build inputs). `borg-think-dungeon-util.c` already
carries 0032/0036/0038, so the verifier proves that the source diff
applied to the before module equals the product module; the applied
commit's diff body is byte-identical to the source commit's diff. The
source evidence fixture at `484c534d` is used unchanged.

## Verified results (`artifacts/s230/scenarios`, 2026-11-07)

- normal-before `stairs-dead-torch` (burned-out worn torch, on a down
  staircase at depth 30): 0/1, `acted=0 keys=1 stairkey=1` — the `<` is
  queued but reported as "nothing done".
- normal-after `stairs-dead-torch`: 1/1, `acted=1 keys=1 stairkey=1`,
  sequence `<` — the routine reports acting, so the think loop stops on
  the stair key.
- `lit` (timeout 2000) and `dead-no-stairs` (dead torch, no staircase):
  skip on every phase, 1/1, byte-identical before/after.
- asan-before/asan-after mirror the normal phases; no sanitizer report.
- `result.json` records `verified: true` with all commands and hashes.

Native gate on this core at the fix commit: **965/965 unit + 5/5 frontend**
numerically read; tooling **66/66** (the current main tooling set;
note the suite now acquires the shared validation lock itself, so it must
run outside a lock wrapper). Regression matrix
(`artifacts/s230-carry/regression.py`, local-only): all **135 prior native
fixture cases** match the recorded forty-carry tally — 132 whole-log
identical (including the mapwall-212 and enlightenment cases), the
dispel-248 undead case passing with its asserted losdispel log change,
and the two dispel-259 cases asserted-red at the pinned post-losdispel
values.

## Frozen v1 measurement: behavior-neutral on the cohort

Post-merge frozen v1 measurement on a prepared observer build of merged
main (seeds 11/29/47, cap 20000, wall 120, stall 30): all three seeds
finish **turn_cap "(alive and well)"** with **whole-trace SHA256
identical to the forty-carry (enlightenment) run** — behavior-neutral on
the cohort (no v1 Warrior burned a torch to zero while standing on
stairs). The seeds' traces diverge from the older 34-carry recording
because mapwall-212 (patch 0039) is behavior-active; that divergence
predates this carry and is recorded in `MAP-WALL-CAPTURE.md`.

## Scope and limits

Bounded native production light-routine scenario with one staircase
situation and two controls. It proves the act report and its correction,
not a gameplay frequency or survival claim.

## Exact next work

1. PR from `task/carry-stairreturn`, agent evidence review plus required
   hosted checks, then normal merge.
2. On merged main: prepare the measurement build
   (`tools/measure-native.py prepare --observe`), run frozen v1 seeds
   11/29/47, cap 20000, wall 120, stall 30, and compare against the
   thirty-nine-carry (mapwall) run — the enlightenment carry was
   behavior-neutral on the cohort, so that comparison isolates this fix
   (expected effect: none unless a v1 Warrior burns a torch down to zero
   while standing on stairs).
3. Mark 167 carried/open+patch-active; receipts on issue 167, source 230
   and checkpoint 1; evidence archive under `artifacts/s230-carry/`.
4. Recheck source207 for newer demonstrations; tapunlife-249 remains
   blocked (see `TAP-UNLIFE.md`).
