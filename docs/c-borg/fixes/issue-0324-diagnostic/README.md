# Issue #324 diagnostic evidence: grey-mold retreat/re-approach loop (D0012)

Curated evidence for the investigation in
[issue #324](https://github.com/diegoami/better-borg-c/issues/324), reviewed in
[#386](https://github.com/diegoami/better-borg-c/issues/386) (two verdicts:
"approve with corrections", findings R-01..R-07 and P2-01..P2-06) with its
re-review package [#387](https://github.com/diegoami/better-borg-c/issues/387).

## Subject

Recorded death D0012 (seed 1907608237, source `867c777845fd`, turns
27476-27770): the borg retreats from a stationary 2-HP grey mold every time it
is adjacent, and the flee flow re-approaches it every time it is one step away.
HP 21 -> 3 over five visits; worm-mass poison ends the life at turn 27770.

## Method (observation-only, non-interference verified)

Disposable copy of the recorded source, instrumented with env-gated hooks
(`BORG_DIAG324`, separate output file; no writes to decisions, RNG or recorded
streams). Two passes; each replayed through the standard verification
machinery with an engine override. Pass 1 hooks: per-tick caution context,
retreat-tick kill facts, `borg_attack` entry, scary-adjacent branch facts.
Pass 2 adds: kept destination list (`TEMP`/`TEMP-KILL`), per-type attack
values (`AUX`), both `borg_attack` exits, kill position/distance, and the
`borg_danger_one_kill` decomposition for the mold at the borg's own grid
(`D1K`). The cumulative instrumentation diff is `instrumentation.patch`
(three files, 285 lines); `diag_replay.py` reproduces the verified replay
(`verify()` takes the shared validation lock itself; do not wrap it in an
outer `flock`).

| replay | engine sha256 (first 16) | verified | first_difference |
|---|---|---|---|
| recorded original engine | `bd807757b2fb022a` | VERIFIED | none |
| pass 1 diagnostic engine | `2ac1200535ca8b41` | true | null |
| pass 2 diagnostic engine | `3fde3cff4a2bfe1a` | true | null |

Recording manifest sha256 `4e0e8ed48c9b9ab9` (identical across all three).

## Findings (final, post-corrections)

1. The cycle is strictly alternating. Adjacent ticks: `borg_caution` retreats
   before attack selection (`pos_danger=16` > `hp/3` at every observed HP,
   17->3; `avoidance == CURHP`). Distance-2 ticks: `borg_attack` keeps the mold
   as a destination (`temp_n=1, okay=1, view=1`), all 22 attack types evaluate
   to `n=0`, exits via `EXIT-NO-ATTACK b_n=0`; the flee flow then re-approaches.
   The scary-adjacent branch passes kills through (fall-through); the scary
   `continue` applies only at distance >= 3, outside the cycle.
2. The adjacent-square danger 16 is entirely the physical component:
   2 blows x max die side 4 = 8, doubled by the `full_damage` round-up in
   `borg_danger_one_kill` (`q = ((q+9)/10)*10`; `full_damage = true` is
   hard-coded in `borg_danger`, borg-danger.c). Any speed advantage >= 1 point
   (this mold rolled speed 111 via `rand_spread`, a true game value, not a
   borg estimate) trips the doubling. Observed damage per exchange is 3-5.
3. Counterfactual (arithmetic, not replayed): without the round-up v1 = 8 and
   the guard `8 > hp/3` still fires at every observed HP - the round-up is not
   causal to the loop.
4. Flee-flow tolerance differs from caution's threshold: `borg_flow_spread`
   marks grids icky when `danger > avoidance*3` (scary, CL<=5), so the same
   danger 16 is passable for the flow at HP >= 6 and icky at HP <= 5 -
   consistent with the recorded flow costs (4 at HP 17-9, 12 at HP 5/3).
   Source-derived, consistent with the trace; the flow's decision itself was
   not hooked.

## Disposition (per the #386 re-review verdict; closure is the owner's call)

No code defect found on this evidence: both halves behave as coded
(conservative max-danger retreat guard; no executable attack at distance 2;
flow tolerance wider than the retreat guard). Design lesson recorded in
`../DESIGN-LESSONS.md` (lesson 1). No neo-angband-tracker proposal, no borg-rs
defect routing; any fix shape (attack the one-blow kill despite the guard,
avoid routes adjacent to stationary monsters, change HP/3 vs 3xHP thresholds)
is gameplay tuning, out of this lane's scope. One recorded death: not
population evidence.
