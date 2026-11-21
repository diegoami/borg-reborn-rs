# Brand-need carry: verified product scenario, ready for adoption

Consumer [133](https://github.com/diegoami/better-borg-c/issues/133), canonical
source [240](https://github.com/diegoami/neo-angband-tracker/issues/240)
(brand-240, priority low). Fix applied as
`224c0d81f4aa37b4a54c868b1a248e092d280bf6` (import of source PR333
`b30afa1a67436d59b070e3509a7e92b932d01f75`, patch
`patches/0035-brand-need-single.patch`).

## Defect

The brand-the-ammo test in `borg_notice_ammo()` was pasted twice with
identical conditions (firebrand activation or the Brand Ammunition spell,
five or more shots, no ego or artifact, identified, the shooter's ammo
type): once inside the `BI_AMMO_POWER >= 3` block and once in a tail block
guarded by a `< 3` return, so the duplicate only ever ran in the same state
and every qualifying stack added twenty to `BI_NEED_BRAND_WEAPON` instead
of ten. The correction deletes the fifteen-line tail; nothing followed it,
and behavior below shooter power three is unchanged. The shipped
configuration formulas do not reference this trait; no memory error; no
death or survival claim.

## Pins and layered provenance

Before build: thirty-four-carry `dc1ea0c3a3554b69b9a70920cd8981351b685389`
(retained tree `../calfree-tokens`, normal and ASan cores, gate recorded at
that commit, source-identical to runtime snapshot `702f091f6` over
src/lib/CMakeLists.txt/cmake). `borg-trait.c` already carries 0017/0029/0031; the
verifier proves that the source diff applied to the before module equals the
product module, and the applied commit's diff is byte-identical to the
source commit's diff. The source evidence fixture at `d9a848a8e` (evidence
PR334) is used unchanged.

## Verified results (`artifacts/b240/scenarios`, 2026-11-07)

- normal-before `brand` (druid 50, long bow power 3, firebrand ring, five
  identified arrows): 0/1, `need_brand=20` — the need is counted twice.
- normal-after `brand`: 1/1, `need_brand=10` — counted once.
- `control` (two arrows, below the five-stack threshold): `need_brand=0`,
  1/1 on every phase, byte-identical before/after.
- asan-before/asan-after mirror the normal phases; no sanitizer report.
- `result.json` records `verified: true` with all commands and hashes.

Retained first attempt: `artifacts/b240/scenarios-attempt1-asan-helpers-missing`
— identical results through asan-before, then aborted because this tree's
ASan core lacked the two `OurUnitTestLib` helper objects; the target was
built and the fixture rerun in full. A v1 attempt against the gate build is
retained as `artifacts/b240-carry/invalid-v1-gate-build-attempt`: the frozen
cohort harness requires a prepared measurement build, so those three runs
are harness failures, not gameplay outcomes.

Native gate on this core at the fix commit: **965/965 unit + 5/5 frontend**
numerically read; tooling **41/41**. Regression matrix
(`artifacts/b240-carry/regression.py`, local-only): all **115 prior native
fixture cases** match the recorded thirty-four-carry tally — 112 whole-log
identical, the dispel-248 undead case passing with its asserted losdispel
log change, and the two dispel-259 cases asserted-red at the pinned
post-losdispel values.

## Scope and limits

Bounded native production inventory-scan scenario for one qualifying arrow
stack and a below-threshold control. It proves the trait valuation and its
correction, not a gameplay frequency or survival claim.

## Exact next work

1. PR from `task/carry-brand-need`, agent evidence review plus required
   hosted checks, then normal merge.
2. On merged main: prepare the measurement build
   (`tools/measure-native.py prepare --observe`), run frozen v1 seeds
   11/29/47, cap 20000, wall 120, stall 30, and compare outcomes and trace
   hashes with the recorded thirty-four-carry run.
3. Mark 133 carried/open+patch-active; receipts on issue 133, source 240
   and checkpoint 1; evidence archive under `artifacts/b240-carry/`.
4. Recheck source207; resume the recorded order (stairtrack-232 next) when
   no higher-priority demonstration is ready.
