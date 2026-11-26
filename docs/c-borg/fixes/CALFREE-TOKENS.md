# Calculation token-array carry: verified product scenario, ready for adoption

Consumer [124](https://github.com/diegoami/better-borg-c/issues/124), canonical
source [323](https://github.com/diegoami/neo-angband-tracker/issues/323)
(calfree-323, priority low). Fix applied as
`dc1ea0c3a3554b69b9a70920cd8981351b685389` (import of source PR341
`2b98c831ba7b3f53b05ddcf620b8b2c54f5ed6a3`, patch
`patches/0034-calculation-token-arrays.patch`).

## Defect

`calculations_free()` released each calculation struct without releasing its
token list's backing pointer array. A borg reinitialize therefore leaked that
array even though the visible count returned to zero. The correction frees
the token array with the struct. Formula values and surface counts are
unchanged.

## Pins, layered provenance and interaction

Before build: thirty-three-carry `4ea7f7614d60bb2d47c53c608f7c552cf41fda05`
(retained tree `../opblock-leak`, normal and ASan cores, gate at `ced816c3`,
source-identical to `4ea7f7614` over src/lib). `borg-formulas-calc.c` already
carries 0026/0033; the verifier proves that the source diff applied to the
before module equals the product module. The source evidence fixture at
`0799d463` is used unchanged.

The pristine source evidence reports 160 bytes/2 allocations before and 80/1
after (the remainder was the then-unfixed operator-block lead). The consumer
before build already contains opblock-324, so the required product contract
is **80 bytes/1 token allocation before and no leaks after**, with zero
operator-block leaks in both phases. This interaction is asserted explicitly;
no source expectation is copied blindly.

## Verified results (`artifacts/c323/scenarios`, 2026-11-07)

- normal-before and normal-after `free`: both 1/1, `freed_value=3`,
  `freed_count=0`, byte-identical logs.
- `control` (unfreed constant): `control_value=42`, `control_count=1`, 1/1 in
  both builds, byte-identical before/after.
- asan-before: functional case passes, then LeakSanitizer reports **80 bytes
  in 1 allocation**, attributed to `tokenize_math`; zero operator-block leaks.
- asan-after: functional case passes and **no leaks are reported**; token and
  operator counts are both zero.
- `result.json` records `verified: true` with all commands and hashes.

Native gate on this core: **965/965 unit + 5/5 frontend**. Regression matrix
(`artifacts/c323-carry/regression.py`, local-only): the preceding 113 cases
comprise 110 whole-log identical cases, one dispel-248 case passing with an
asserted changed log, and two dispel-259 cases that are asserted-red at
pinned post-losdispel values. With the two new byte-identical free cases,
the final matrix records **113 passing and two asserted-red interaction
cases (115 total)**. Tooling **41/41**.

A post-review final-runtime receipt (`artifacts/formula-leaks-final/`)
links the unchanged calc-reload-325, opblock-324 and calfree-323 fixtures
against the ASan build source-identical to `702f091f6`: all three pass with
no LeakSanitizer report. The retained pre-fix calc-reload binary still
reports 320 bytes in 4 allocations in the same environment, confirming leak
detection. An initial verifier-only suite-spelling failure is retained at
`artifacts/formula-leaks-final-verifier-suite-name-error`.

## Scope and limits

Minimal native parse/evaluate/free scenario with stack-specific
LeakSanitizer attribution. It proves the token-array ownership defect and its
correction, not a formula-value, gameplay or survival change. Together with
0026/0033 it clears the three demonstrated formula-memory leak paths in their
respective fixtures.

## Exact next work

1. PR from `task/carry-calfree-tokens`, agent evidence review plus required
   hosted checks, then normal merge.
2. On merged main: frozen v1 measurement seeds 11/29/47, cap 20000, wall
   120, stall 30.
3. Mark 124 carried/open+patch-active; receipts on issue 124, source 323 and
   checkpoint 1; evidence archive under `artifacts/c323-carry/`.
4. Pause as the owner requested. Resume by rechecking source207, then
   brand-240 and stairtrack-232 if no higher-priority demonstration is ready.
