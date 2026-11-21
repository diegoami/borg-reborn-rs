# Surrounded-trap carry: verified product scenario, ready for adoption

Consumer [109](https://github.com/diegoami/better-borg-c/issues/109), canonical
source [312](https://github.com/diegoami/neo-angband-tracker/issues/312)
(surrounded-312, priority medium). Fix applied as
`dbdb5f7fa49963653abebfbce05537c502254c6f` (import of source PR351
`d366ef4e75e40f17fb293135c05fa74524f2e184`, patch
`patches/0030-surrounded-trap-chain.patch`).

## Defect

A trap-flagged wall matched both the wall branch and the independent trap
branch in `borg_surrounded()`, subtracting two safe squares. On the
demonstrated board—three trap-flagged walls, three trap-only floors and zero
monsters—the original drove the count below zero and returned surrounded.
The correction makes the trap test the fifth branch of the same exclusive
chain.

## Pins and provenance

Before build: twenty-nine-carry `127483bc4724c0f75ff7bbc4edddde9293e37c2c`
(retained tree `../light-modifier`, normal and ASan cores, gate at
`497d9266`, source-identical to `127483bc4` over src/lib). `borg-escape.c`
had no earlier consumer patch. The verifier proves that the source diff
applied to the before module equals the product module. The source evidence
fixture at `44334c6f` is used unchanged.

## Verified results (`artifacts/s312/scenarios`, 2026-11-07)

- normal-before `traps`: 0/1, `surrounded=1` with no monsters nearby.
- normal-after `traps`: 1/1, `surrounded=0`.
- `walls` (plain-wall control): `surrounded=0`, 1/1 on every phase,
  byte-identical before/after.
- asan-before/after mirror the normal phases; no sanitizer report.
- `result.json` records `verified: true` with all commands and hashes.

Native gate on this core: **965/965 unit + 5/5 frontend**. Regression matrix
(`artifacts/s312-carry/regression.py`): all **105 prior native fixture cases**
pass (103 identical plus the two documented losdispel interactions asserted),
and the two new surrounded cases pass, **107 total**. Tooling 29/29.

## Scope and limits

Bounded native production `borg_surrounded()` scenario on the reported board.
It proves the phantom verdict and its correction; it does not quantify how
often that verdict changed live behavior or claim a survival improvement.

## Exact next work

1. PR from `task/carry-surrounded-traps`, agent evidence review plus required
   hosted checks, then normal merge.
2. On merged main: frozen v1 measurement seeds 11/29/47, cap 20000, wall
   120, stall 30.
3. Mark 109 carried/open+patch-active; receipts on issue 109, source 312 and
   checkpoint 1; evidence archive under `artifacts/s312-carry/`.
4. Then homeblows-313 (priority medium), followed by the low-priority
   opblock-324/calfree-323 and brand-240 queue after a source207 recheck.
