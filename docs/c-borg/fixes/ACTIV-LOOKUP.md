# Activ lookup carry: verified product scenario, ready for adoption

Consumer [75](https://github.com/diegoami/better-borg-c/issues/75), canonical
source [260](https://github.com/diegoami/neo-angband-tracker/issues/260)
(activ-260). Fix applied as `7323c8df1506f5572c4cfbbc7348cd4f3fbe88da`
(import of source PR308 `891d0cb9cc9057a2004dfeba7b21b19913f5a2b6`, patch
`patches/0019-activ-lookup-ids.patch`).

## Defect

The `BF_ACT_ACID_BOLT3` maneuver passed `act_acid_bolt2` to the equips
check, so the Borg looked for the lesser bolt on its gear: the
greater-bolt item was never considered and the maneuver was valued at 0
(and the borrowing direction was real — a worn lesser bolt got fired at
the greater bolt's damage). The companion `BF_ACT_DISPEL_ALL` token looked
up `act_dispel_undead`, so an all-monsters dispel item was never
considered; it rides in the same two-token fix and is verified by reading
(its corrected valuation hits a separate dispel bolt-walk quirk documented
in the source README, out of scope).

## Pins and layered provenance

Before build: eighteen-carry `d29f373e5540a54c49fec10c10d48329a4bf26dc`
(retained tree `../detect-panel`, normal and ASan cores, gate at
`c3e8f60a`, source-identical to `d29f373e5` over src/lib).
`borg-fight-attack.c` already carries 0011/0013/0014/0016; the verifier
proves the before module + the source two-token diff equals the product
module (the dispel id line is unique; the acid id line appears twice and
only its second occurrence — inside `BF_ACT_ACID_BOLT3` — changes).
Process note: the first edit attempt matched nothing (the file's own
wrapping), was caught by its own assertion, never committed, and the
builds of that attempt ran on the unpatched tree; they were discarded and
rerun after the verified-context import.

## Verified results (`artifacts/a260/scenarios`, 2026-11-06)

- normal-before `miswired`: 0/1, `bolt3=0 bolt2=0` — the worn
  greater-bolt activator is invisible to the maneuver that describes it.
- normal-after `miswired`: 1/1, `bolt3=52 bolt2=0`.
- `control` (correctly-wired lesser-bolt maneuver): `bolt2=52`, 1/1 on
  every phase, byte-identical before/after.
- asan-before/after mirror the normal phases; no sanitizer report.
- `result.json` records `verified: true` with all commands and hashes.

Native gate on this core: **965/965 unit + 5/5 frontend**. Regression
matrix (`artifacts/a260-carry/regression.py`): all **83 prior native
fixture cases whole-log identical** to their baselines (81 + 2 detect).
Tooling 29/29.

## Scope and limits

Bounded native rating scenario: the maneuver values for the worn
greater-bolt activator and the correctly-wired lesser bolt, target list
filled the way the chooser fills it. The DISPEL_ALL token is corrected in
the same import but its valuation is not separately asserted (the source
README documents the separate dispel bolt-walk quirk its corrected
valuation reaches). Rating figures are the Borg's heuristic; no
combat-outcome, frequency or survival claim.

## Exact next work

1. PR from `task/carry-activ-lookup`, agent evidence review plus required
   hosted checks, then normal merge.
2. On merged main: frozen v1 measurement seeds 11/29/47, cap 20000, wall
   120, stall 30.
3. Mark 75 carried/open+patch-active; receipts on issue 75, source 260 and
   checkpoint 1; evidence archive under `artifacts/a260-carry/`.
4. Then reserve-258 (PR318 `7b4ecd6b`, one loop, pin 359882c25), then
   recheck source207 for newer demonstrations (source-ranked 259/262/261).
