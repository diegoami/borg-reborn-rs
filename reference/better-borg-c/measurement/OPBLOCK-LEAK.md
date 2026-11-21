# Operator-block leak carry: verified product scenario, ready for adoption

Consumer [121](https://github.com/diegoami/better-borg-c/issues/121), canonical
source [324](https://github.com/diegoami/neo-angband-tracker/issues/324)
(opblock-324, priority low). Fix applied as
`ced816c3c53ae1ad2346e0ab3064a32fb489ad1f` (import of source PR339
`02be7739db306897f2441562ee8d27dddc15d36c`, patch
`patches/0033-operator-block-arrays.patch`).

## Defect

Precedence adjustment freed each temporary operator-block struct without
freeing its backing pointer array. Every populated operator block of every
parsed formula, including startup parsing, could therefore leak its array.
The correction releases the array with the struct. Formula precedence and
values are unchanged.

## Pins and layered provenance

Before build: thirty-two-carry `56f9d49595f1cd5504e6f1d92edb144f543c88ff`
(retained tree `../lunal-state`, normal and ASan cores, gate at `2e0593f6`,
source-identical to `56f9d4959` over src/lib). `borg-formulas-calc.c` already
carries 0026; the verifier proves that the source diff applied to the before
module equals the product module. The source evidence fixture at `6cbb0347`
is used unchanged.

## Verified results (`artifacts/o324/scenarios`, 2026-11-07)

- normal-before and normal-after `operators`: both 1/1,
  `formula_value=7`, `calc_count=1`, byte-identical logs.
- `control` (bare constant): `control_value=42`, `control_count=1`, 1/1 in
  both builds, byte-identical before/after.
- asan-before: functional case passes, then LeakSanitizer reports **160
  bytes in 2 allocations**, grouped into one Direct-leak report block whose
  stack contains `adjust_order_ops_depth_block`; exit 1 is retained.
- asan-after: functional case passes and **no leaks are reported**.
- `result.json` records `verified: true` with all commands and hashes.

The first verifier run incorrectly expected two matching report blocks rather
than the one block grouping two allocations. That tooling failure is retained
under `artifacts/o324/verifier-group-count-error`; only the verifier's grouping
assertion was corrected, then the unchanged fixture was rerun successfully.
No product or fixture assertion was weakened.

Native gate on this core: **965/965 unit + 5/5 frontend**. Regression matrix
(`artifacts/o324-carry/regression.py`): all **111 prior native fixture cases**
pass (109 identical plus the two documented losdispel interactions asserted),
and the two new operator cases pass, **113 total**. Tooling **41/41**; the
increase from 29 is the independently merged PR116 `task/named-headless`
coverage present on this main baseline.

## Scope and limits

Minimal native formula-parse scenario with stack-specific LeakSanitizer
attribution. It proves the temporary-array ownership defect and its correction,
not a formula-value, gameplay or survival change.

## Exact next work

1. PR from `task/carry-opblock-leak`, agent evidence review plus required
   hosted checks, then normal merge.
2. On merged main: frozen v1 measurement seeds 11/29/47, cap 20000, wall
   120, stall 30.
3. Mark 121 carried/open+patch-active; receipts on issue 121, source 324 and
   checkpoint 1; evidence archive under `artifacts/o324-carry/`.
4. Then calfree-323 and brand-240, after a source207 recheck for newer
   high/medium demonstrations.
