# Excavate-dark-flag carry: verified product scenario, ready for adoption

Consumer [188](https://github.com/diegoami/better-borg-c/issues/188), canonical
source [236](https://github.com/diegoami/neo-angband-tracker/issues/236)
(excavate-236). Fix applied as
`bdd3c2ea2c120c5200f182b1676b860942abc71e` (import of source PR400-head
`8e0f8448e9ebc6d164e4e621840acb0005094f81`, patch
`patches/0043-excavate-dark-flag.patch`).

## Defect

In `borg_excavate_vault()` the dug vault grid is remembered with the
comment "Dark" but the flag line sets `BORG_GLOW` — an exact copy of the
preceding "Not Lit" clear — so the excavated vault grid is remembered as
glowing where the routine intends darkness. The correction sets
`BORG_DARK` (the `&= ~BORG_GLOW` clear immediately above stays). No memory
error; no death or survival claim.

## Pins and layered provenance

Before build: forty-two-carry `64046743679e0925ae6698b458a7bbf9dfa6cb19`
(retained tree `../blowname`, normal and ASan cores, gate recorded at that
commit, source-identical to merged main `8c23ec0d4` over
src/lib/CMakeLists.txt/cmake). `borg-think-dungeon-util.c` already carries
0032/0036/0038/0041, so the verifier proves that the source diff applied
to the before module equals the product module; the applied commit's diff
body is byte-identical to the source commit's diff. The source evidence
fixture at `793f723eb` (evidence PR401 head; fixtures byte-identical to
the recorded fixture commit `4478e689c`; the head's artifact commit repairs
only the replay generator's panel wiring) is used unchanged.

## Verified results (`artifacts/e236/scenarios`, 2026-11-07)

- normal-before `excavate` (quartz dig grid beside a permanent vault wall,
  production `borg_excavate_vault(9)`): 0/1, `acted=1 dark_marked=0` — the
  dug grid is marked glowing where the comment says Dark.
- normal-after `excavate`: 1/1, `acted=1 dark_marked=1` — marked dark.
- `no-target` (no excavatable grid beside a permanent wall): skip on every
  phase, 1/1, byte-identical before/after.
- asan-before/asan-after mirror the normal phases; the sanitized runs
  report the pre-existing init-time `borg_sort_comp_hook` overflow
  (source finding #399, separately filed) in every phase — expected, and
  asserted as such rather than suppressed.
- `result.json` records `verified: true` with all commands and hashes.

Retained attempts: `scenarios-attempt1-init-halt` (plain ASan options
halted both sanitized runs at the pre-existing init overflow before the
scenario ran) and `scenarios-attempt2-asan-expected-report` /
`scenarios-attempt3-assert-loop` / `scenarios-attempt4-typo-before-path`
(verifier assertion and invocation mistakes, corrected without touching
any product assertion). The final attempt's normal-phase logs are
identical across all attempts.

Native gate on this core at the fix commit: **965/965 unit + 5/5 frontend**
numerically read; tooling **73/73** (the current main tooling set).
Regression matrix (`artifacts/e236-carry/regression.py`, local-only): all
**139 prior native fixture cases** match the recorded forty-two-carry
tally — 136 whole-log identical (including the blowname-247 case), the
dispel-248 undead case passing with its asserted losdispel log change,
and the two dispel-259 cases asserted-red at the pinned post-losdispel
values.

## Scope and limits

Bounded native production vault-excavation scenario with one dig site and
one control. It proves the flag marking and its correction, not a gameplay
frequency or survival claim.

## Exact next work

1. PR from `task/carry-excavate`, agent evidence review plus required
   hosted checks, then normal merge.
2. On merged main: prepare the measurement build
   (`tools/measure-native.py prepare --observe`), run frozen v1 seeds
   11/29/47, cap 20000, wall 120, stall 30, and compare against the
   forty-two-carry (blowname-lineage) run — expected behavior-neutral
   unless a v1 Warrior excavates a vault during the cohort.
3. Mark 188 carried/open+patch-active; receipts on issue 188, source 236
   and checkpoint 1; evidence archive under `artifacts/e236-carry/`.
4. Recheck source207 for newer demonstrations; tapunlife-249 remains
   blocked (see `TAP-UNLIFE.md`); source #399 remains a separately owned
   source finding.
