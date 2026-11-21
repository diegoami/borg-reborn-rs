# Trollswap-201 carry: verified product scenario

Consumer [289](https://github.com/diegoami/better-borg-c/issues/289),
canonical source
[201](https://github.com/diegoami/neo-angband-tracker/issues/201)
(trollswap-201; source PR454 `4fe83bc8`, evidence PR453). Fix applied as
`288556d2b` (`cherry-pick -x` of source
`4fe83bc81a2208e76481e09fa66a66481052cd44`, patch
`patches/0073-troll-slay-real-multiplier.patch`).

## Defect

The swap-weapon valuation added a flat `(dam * 2) / 2` for Slay Troll
instead of the weapon's real multiplier (every other slay trait uses its own
`weapon_swap_slay_*`). A Warrior holding a Slay Orc mace as the swap kept it
and offered the strictly better Slay Troll mace for sale. The correction
values the troll slay by `weapon_swap_slay_troll`: the troll mace is kept as
the swap and the orc mace is offered. Valuation only; no fight measured.

## Pins and provenance

Before: merged main `6002bfa4b` (snapshotted build `build/native-before`,
gate result `artifacts/native-before/result.json`, 965/965 + 5/5 at that
commit). The verifier proves that the source diff applied to the before
`borg-trait-swap.c` equals the product module (the file already carries the
0045 gloves correction in a different function) and that the applied commit
is the only `src`/`lib` difference from the before commit. Fixture: kit
`600adff3` at head `2f3a73f37`, asserted byte-identical to the run pin.

## Verified results (`artifacts/ts201/scenarios`, 2026-11-08)

- `troll-orc` — fail before, pass after: before 0/1 (`sells_other=0
  sells_troll=1`; the Slay Troll mace offered for sale), after 1/1
  (`sells_other=1 sells_troll=0`; the troll mace kept as the swap, the orc
  mace offered).
- `troll-plain` control: 1/1 in both phases, log byte-identical
  (`4c13aff2…`).
- `result.json` records `verified: true` with all commands, pins and
  hashes.

Native gate at the applied step tip: **965/965 unit + 5/5 frontend**
(before snapshot gate and product gate); tooling **129/129**. A harness
correction is recorded in the verifier commit: the first scope check
compared the whole PR tree (both stacked files) against this fix's file;
the fix commit's own scope and the documented stacked file are asserted
instead. Prior-fixture matrix not rerun (uncommitted runner; known gap
recorded on checkpoint 1 and the milestone 6 review).

## Scope and limits

Bounded valuation scenario driving the production notice and sell-useless
decision. It proves which weapon is kept as the swap and which is offered;
no combat effect is measured.

## Exact next work

1. Stacked in the same PR as drophole-136 (independent files); per-step
   gates; agent evidence review plus required hosted checks, then normal
   merge.
2. Frozen v1 cohort at the merged pair tip against the 0071-after cohort at
   `6002bfa4b`.
3. Receipts on issues 289/290, sources 201/136 and checkpoint 1.
