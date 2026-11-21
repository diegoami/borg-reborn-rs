# Homebuyqty-19 carry: verified product scenario

Consumer [365](https://github.com/diegoami/better-borg-c/issues/365),
canonical source
[19](https://github.com/diegoami/neo-angband-tracker/issues/19)
(homebuyqty-19; source fix `e075f0ac7`, evidence fixture `40495b33a`).
Applied as a `cherry-pick -x` of source
`e075f0ac7c618fdd32eb5bd4b49d1936e8738da4`, patch
`patches/0093-home-buy-restore-ware.patch` (applied commit
`d649a36fbf`).

## Defect

`borg_think_home_buy_useful()` takes the trial quantity (and, for
staffs and wands, its charges) off its copy of the home ware before it
checks that the pack has room. The two "require two empty slots" checks
then skip the ware with a bare `continue`, so the copy is left reduced
instead of being restored from `borg_safe_shops` at the end of the loop
body. The later store steps read that copy: with exactly one free pack
slot and a single weapon in the home, the weapon is left with quantity
zero, and the backup-weapon step
(`borg_think_home_buy_swap_weapon`) skips it as an empty slot even
though it would buy it otherwise. The correction restores the ware
before each of those continues.

## Pins and provenance

Before: merged main `f8ecd11c7` (snapshotted build `build/native-before`,
gate result `artifacts/native-before/result.json`, 965/965 + 5/5 at that
commit). `borg-store-buy.c` is otherwise pristine on the product. The
verifier proves that the source diff applied to the before module equals
the product module and that the applied commit is the only `src`/`lib`
difference from the before commit. Fixture: kit `40495b33a` at head
`fbd96fa79`, asserted byte-identical to the run pin.

## Verified results (`artifacts/hb19/scenarios`, 2026-11-09)

- `single` — fail before, pass after: before the backup-weapon step
  skipped the home weapon left at quantity zero by the unreduced trial
  copy (`buys_home_ware0=0`), after it buys (`=1`).
- `pair` — **second flip on this product, disclosed**: the outcome row
  (`buys_home_ware0=1`) matches the kit's recorded row in both phases,
  but the scenario's own self-check fails before the fix on the
  consumer — the carried urgentshop-23/starvealch-18 changes in this
  module alter the scenario's secondary self-check (the kit's
  before-row passed on the pristine base). After the fix it passes.
- `roomy` / `stack` controls: 1/1 in both phases, logs byte-identical
  before/after (`1a338049…`, `659807d5…`).
- `result.json` records `verified: true` with all commands, pins and
  hashes. Harness notes recorded in the verifier commit: the pair
  reclassification above, and the byte-identity assertions for the
  controls were confirmed empirically on the first run.

## Scope and limits

Bounded home-purchase scenarios varying free pack slots and ware
stacking. They prove the restore and the backup-weapon purchase; no
survival claim.

## Exact next work

1. PR from `task/carry-homebuy`, agent evidence review plus required
   hosted checks, then normal merge.
2. Frozen v1 cohort at the merged tip against the healchance tip cohort.
3. Receipts on issue 365, source 19 and checkpoint 1.
