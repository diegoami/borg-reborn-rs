# Home-blows carry: verified product scenario, ready for adoption

Consumer [113](https://github.com/diegoami/better-borg-c/issues/113), canonical
source [313](https://github.com/diegoami/neo-angband-tracker/issues/313)
(homeblows-313, priority medium). Fix applied as
`668ac32c4be91b773410e652d3913cbb12dfe199` (import of source PR353
`d7b42db02cc74473bc175d2c37dd6feba1a77811`, patch
`patches/0031-home-blows-single-weapon.patch`).

## Defect

The home-inventory blows calculation used the entire weapon stack's weight
where the engine uses one weapon's weight. For a stack of ten rapiers at
strength 17 and dexterity 16, the Borg computed one blow while the engine
computed two. The correction divides the stack weight by its count, restoring
agreement without changing single weapons.

## Pins and layered provenance

Before build: thirty-carry `1e09c1c7bd7379767df94eab9a157f8c88d3a11d`
(retained tree `../surrounded-traps`, normal and ASan cores, gate at
`dbdb5f7f`, source-identical to `1e09c1c7b` over src/lib). `borg-trait.c`
already carries 0017/0029; the verifier proves that the source diff applied
to the before module equals the product module. The source evidence fixture
at `d73c67ab` is used unchanged.

## Verified results (`artifacts/h313/scenarios`, 2026-11-07)

- normal-before `stack`: 0/1, `borg_blows=1`, `engine_blows=2`.
- normal-after `stack`: 1/1, `borg_blows=2`, `engine_blows=2`.
- `single` (one rapier): `borg_blows=2`, `engine_blows=2`, 1/1 on every
  phase, byte-identical before/after.
- asan-before/after mirror the normal phases; no sanitizer report.
- `result.json` records `verified: true` with all commands and hashes.

Native gate on this core: **965/965 unit + 5/5 frontend**. Regression matrix
(`artifacts/h313-carry/regression.py`): all **107 prior native fixture cases**
pass (105 identical plus the two documented losdispel interactions asserted),
and the two new blows cases pass, **109 total**. Tooling 29/29.

## Scope and limits

Bounded native production item-analyzer scenario comparing Borg and engine
blows for a weapon stack and a single weapon. It proves the valuation
divergence and its correction, not a combat-outcome or survival claim.

## Exact next work

1. PR from `task/carry-home-blows`, agent evidence review plus required
   hosted checks, then normal merge.
2. On merged main: frozen v1 measurement seeds 11/29/47, cap 20000, wall
   120, stall 30.
3. Mark 113 carried/open+patch-active; receipts on issue 113, source 313 and
   checkpoint 1; evidence archive under `artifacts/h313-carry/`.
4. Then lunal-314 (priority medium), followed by the low-priority
   opblock-324/calfree-323 and brand-240 queue after a source207 recheck.
