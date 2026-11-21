# Tradehist-20 carry: verified product scenario

Consumer [367](https://github.com/diegoami/better-borg-c/issues/367),
canonical source
[20](https://github.com/diegoami/neo-angband-tracker/issues/20)
(tradehist-20; source fix `9ae5e4585`, evidence fixture `942c80a50`).
Applied as a `cherry-pick -x` of source
`9ae5e4585599aa4f3c5dce7c2259fe7e2604cea1`, patch
`patches/0094-trade-history-counts.patch` (applied commit
`168a432522`).

## Defect

The Borg's recent-sale and recent-purchase lists prevent buy-backs and
sell-backs, but the writers stored the *index* of the just-written
entry in `sold_item_num` / `bought_item_num` while every reader scans
`p < num` — the most recent sale or purchase was never checked. After
storing three rations at home, a low-level Warrior took them straight
back out on its next store turn. The correction keeps the counters as
slot counts: advance the write index first and raise the count to it;
when the write index wraps, the count stays at the ring size
(`src/borg/borg-store-buy.c`, `src/borg/borg-store-sell.c`).

## Pins and provenance

Before: merged main `97c634a3e` (snapshotted build `build/native-before`,
gate result `artifacts/native-before/result.json`, 965/965 + 5/5 at that
commit). Both modules are otherwise pristine on the product except for
the carried urgentshop-23/starvealch-18/homebuyqty-19 changes in
`borg-store-buy.c`, which the source base predates: the 3-way merges
preserve them and the layered-module check proves the source diff still
reproduces the product modules. The verifier proves that the source diff
applied to the before modules equals the product modules and that the
applied commit is the only `src`/`lib` difference from the before
commit. Fixture: kit `942c80a50` at head `ddb06e523`, asserted
byte-identical to the run pin.

## Verified results (`artifacts/th20/scenarios`, 2026-11-09)

- `rations-only` — fail before, pass after: before two of the three
  stashed rations were retrieved straight back out
  (`retrieved_rations=2`; the newest entry was never checked), after
  none are (`=0`).
- `with-recall` control: 1/1 in both phases, identical rows and
  byte-identical log (`45623686…`).
- `result.json` records `verified: true` with all commands, pins and
  hashes. The verifier passed on its first complete run.

## Scope and limits

Bounded home-storage scenarios (with and without a recall scroll in
play). They prove the trade-history count semantics; no survival claim.

## Exact next work

1. PR from `task/carry-tradehist`, agent evidence review plus required
   hosted checks, then normal merge.
2. Frozen v1 cohort at the merged tip against the homebuy tip cohort.
3. Receipts on issue 367, source 20 and checkpoint 1.
