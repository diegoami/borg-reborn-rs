# Calculation-reload carry: verified product scenario, ready for adoption

Consumer [96](https://github.com/diegoami/better-borg-c/issues/96), canonical
source [325](https://github.com/diegoami/neo-angband-tracker/issues/325)
(calcreload-325). Fix applied as `c528eb81e0651bad91f28c896594cf56ad6fd3de`
(import of source PR337 `253ab11468bd91f47f2a458ecc3f333a00eaa677`, patch
`patches/0026-calculation-reload-array.patch`).

## Defect

`calculations_free()` zeroed the calculation count but retained the backing
pointer array. `borg_array_add()` allocates a fresh array whenever the count
is zero, so parsing after a borg reinitialize overwrote the only pointer to
the previous allocation. The overwritten 80-byte array was lost on every
such reload.

The correction releases the array with the calculations and resets its
pointer and capacity. The reloaded formula still evaluates correctly.

## Pins and provenance

Before build: twenty-five-carry `7f4c9e7ae92aeed5add7ab672f6c2717dc925189`
(retained tree `../text-multibyte`, normal and ASan cores, gate at
`c6c51d4d`, source-identical to `7f4c9e7ae` over src/lib).
`borg-formulas-calc.c` had no earlier consumer patch. The verifier proves
that the source diff applied to the before module equals the product module.
The source evidence fixture at `8764d94b8` is used unchanged.

## Verified results (`artifacts/c325/scenarios`, 2026-11-06)

- normal-before `reload`: 0/1, `retained=1`, `reload_value=7`,
  `reload_count=1` — the stale backing pointer survives free.
- normal-after `reload`: 1/1, `retained=0`, `reload_value=7`,
  `reload_count=1` — the array is released and reload remains functional.
- `control` (single parse): `control_value=3`, `control_count=1`, 1/1 in
  both builds, byte-identical before/after.
- asan-before: LeakSanitizer **320 bytes in 4 allocations**, including the
  calculation-array allocation identified by the adjacent
  `borg_array_add`/`parse_calculation_line` stack.
- asan-after: **240 bytes in 3 allocations**; that calculation-array leak
  is gone. The separately owned operator-block (2) and token-array (1)
  leaks remain present at identical counts, as required. LeakSanitizer's
  exit 1 is retained as evidence, not suppressed.
- `result.json` records `verified: true` with all commands and hashes.

Native gate on this core: **965/965 unit + 5/5 frontend**. Regression matrix
(`artifacts/c325-carry/regression.py`): all **97 prior native fixture cases**
pass (95 identical plus the two documented losdispel interactions asserted),
and the two new reload cases pass, **99 total**. Tooling 29/29.

## Scope and limits

Minimal native calculation parse/free/reparse scenario with stack-specific
LeakSanitizer allocation attribution. It proves the 80-byte-per-reload
ownership defect and its correction. It does not claim corrupted calculations,
a gameplay behavior change, or a measurable survival effect. The other two
formula-memory leaks remain explicitly open until their own carries.

## Exact next work

1. PR from `task/carry-calc-reload`, agent evidence review plus required
   hosted checks, then normal merge.
2. On merged main: frozen v1 measurement seeds 11/29/47, cap 20000, wall
   120, stall 30.
3. Mark 96 carried/open+patch-active; receipts on issue 96, source 325 and
   checkpoint 1; evidence archive under `artifacts/c325-carry/`.
4. Then opblock-324 and calfree-323, followed by brand-240; recheck
   source207 for newer demonstrations.
