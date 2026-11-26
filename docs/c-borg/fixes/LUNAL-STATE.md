# Lunal-state carry: verified product scenario, ready for adoption

Consumer [117](https://github.com/diegoami/better-borg-c/issues/117), canonical
source [314](https://github.com/diegoami/neo-angband-tracker/issues/314)
(lunal-314, priority medium). Fix applied as
`2e0593f6effcd211b565be3c589d8716de85c1ec` (import of source PR355
`a26da54d61be83236c2c5859141b801521d6c6d5`, patch
`patches/0032-lunal-flag-pair.patch`).

## Defect

The production town stair-scum bail cleared `lunal_mode` but left
`fleeing_lunal` set, producing the split state the caution phase misreads.
The correction clears the pair, matching the normal lunal exit. A full-pack
bail follows a different path and must leave both flags untouched.

## Pins and provenance

Before build: thirty-one-carry `b0cf47b1354918825d0c2c464b2460e85296e672`
(retained tree `../home-blows`, normal and ASan cores, gate at `668ac32c`,
source-identical to `b0cf47b13` over src/lib).
`borg-think-dungeon-util.c` had no earlier consumer patch. The verifier proves
that the source diff applied to the before module equals the product module.
The source evidence fixture at `d50baba7` is used unchanged.

## Verified results (`artifacts/l314/scenarios`, 2026-11-07)

- normal-before `town`: 0/1, `lunal_mode=0`, `fleeing_lunal=1` — split state.
- normal-after `town`: 1/1, `lunal_mode=0`, `fleeing_lunal=0` — pair cleared.
- `pack` (full-pack control): `lunal_mode=1`, `fleeing_lunal=1`, 1/1 on every
  phase, byte-identical before/after.
- asan-before/after mirror the normal phases; no sanitizer report.
- `result.json` records `verified: true` with all commands and hashes.

Native gate on this core: **965/965 unit + 5/5 frontend**. Regression matrix
(`artifacts/l314-carry/regression.py`): all **109 prior native fixture cases**
pass (107 identical plus the two documented losdispel interactions asserted),
and the two new lunal cases pass, **111 total**. Tooling 29/29.

## Scope and limits

Bounded native production bail-path scenario for the two flag states. It
proves the split-state correction and unchanged full-pack control, not a live
frequency or survival claim.

## Exact next work

1. PR from `task/carry-lunal-state`, agent evidence review plus required
   hosted checks, then normal merge.
2. On merged main: frozen v1 measurement seeds 11/29/47, cap 20000, wall
   120, stall 30.
3. Mark 117 carried/open+patch-active; receipts on issue 117, source 314 and
   checkpoint 1; evidence archive under `artifacts/l314-carry/`.
4. Recheck source207; when no higher-priority demonstration is ready, resume
   opblock-324 and calfree-323, then brand-240.
