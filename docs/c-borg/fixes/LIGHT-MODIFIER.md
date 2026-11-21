# Light-modifier carry: verified product scenario, ready for adoption

Consumer [105](https://github.com/diegoami/better-borg-c/issues/105), canonical
source [238](https://github.com/diegoami/neo-angband-tracker/issues/238)
(light-238, priority medium). Fix applied as
`497d92665d6d688c89faf1b58b0103442d651fb9` (import of source PR349
`66b183340a567811051a5745c8d3a53792963860`, patch
`patches/0029-light-modifier-once.patch`).

## Defect

The production equipment notice added an item's light modifier to the Borg's
light radius twice. The Phial of Galadriel's `LIGHT[4]` consequently produced
`bi_light=8`. Stock lights use `LIGHT_2`/`LIGHT_3` flags with a zero modifier,
so they never reach the duplicated modifier line. The correction drops the
second addition and leaves flag-based lights unchanged.

## Pins and layered provenance

Before build: twenty-eight-carry `df73d0fcad14d2f7335a91c4fb652f858fdc7527`
(retained tree `../glow-overflow`, normal and ASan cores, gate at `3cb61705`,
source-identical to `df73d0fca` over src/lib). `borg-trait.c` already carries
0017; the verifier proves that the source diff applied to the before module
equals the product module. The source evidence fixture at `f1e899b6` is used
unchanged.

## Verified results (`artifacts/l238/scenarios`, 2026-11-07)

- normal-before `phial`: 0/1, `bi_light=8` — the modifier of four is doubled.
- normal-after `phial`: 1/1, `bi_light=4` — the modifier is counted once.
- `torch` (stock flag-based Wooden Torch): `bi_light=2`, 1/1 on every phase,
  byte-identical before/after.
- asan-before/after mirror the normal phases; no sanitizer report.
- `result.json` records `verified: true` with all commands and hashes.

Native gate on this core: **965/965 unit + 5/5 frontend**. Regression matrix
(`artifacts/l238-carry/regression.py`): all **103 prior native fixture cases**
pass (101 identical plus the two documented losdispel interactions asserted),
and the two new light cases pass, **105 total**. Tooling 29/29.

## Scope and limits

Bounded native production equipment-notice scenario for the Phial and a stock
Wooden Torch. It proves the trait valuation and its correction, not a gameplay
frequency or survival claim.

## Exact next work

1. PR from `task/carry-light-modifier`, agent evidence review plus required
   hosted checks, then normal merge.
2. On merged main: frozen v1 measurement seeds 11/29/47, cap 20000, wall
   120, stall 30.
3. Mark 105 carried/open+patch-active; receipts on issue 105, source 238 and
   checkpoint 1; evidence archive under `artifacts/l238-carry/`.
4. Recheck source207; when no higher-priority demonstration is ready, resume
   opblock-324 and calfree-323, then brand-240.
