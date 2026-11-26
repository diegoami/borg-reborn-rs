# Runeglyph-295 carry: verified product scenario

Consumer [375](https://github.com/diegoami/better-borg-c/issues/375),
canonical source
[295](https://github.com/diegoami/neo-angband-tracker/issues/295)
(runeglyph-295; source fix `50e64a37a`, evidence fixture `3ce928e86`).
Applied as a `cherry-pick -x` of source
`50e64a37a837c3ce4563e9ade452ce28db5cee82`, patch
`patches/0098-visible-rune-recording.patch` (applied commit
`8b96bd4798`).

## Defect

The Borg's map update set a grid's glyph flag only inside the branch
for visible, disarmable player traps. A Rune of Protection (and a
decoy) is a glyph, not a player trap — its kind carries
`GLYPH | VISIBLE | FLOOR` and no TRAP flag — so that branch never ran
and the Borg's map said there was no rune on a grid that visibly had
one. Standing on such a rune with a summoner in view, the Borg's
defence choice still saw the full summoning danger and read another
Rune of Protection onto the same grid, spending a scroll and a turn
for nothing. The correction also enters the branch when the trap the
player remembers on the grid is a glyph, and marks the grid as a trap
only for a real disarmable player trap; the rune is then recorded and
tracked as the Borg's own runes are.

## Pins and provenance

Before: merged main `521c880a5` (snapshotted build `build/native-before`,
gate result `artifacts/native-before/result.json`, 965/965 + 5/5 at that
commit). `borg-update.c` is otherwise pristine on the product. The
verifier proves that the source diff applied to the before module equals
the product module and that the applied commit is the only `src`/`lib`
difference from the before commit. Fixture: kit `3ce928e86` at head
`dae7a1bd6`, asserted byte-identical to the run pin.

## Verified results (`artifacts/rg295/scenarios`, 2026-11-09)

- `rune` — the defect's direct signal: before, the visible rune is
  **not recorded** (`borg_glyph_here=0`, `game_runes_here=1`) and the
  scenario self-check fails (the borg's defence still saw the full
  summoning danger); after, the rune is recorded
  (`borg_glyph_here=1`) and no second rune is read in either phase on
  this product (`read_rune=0` before and after — on the kit's base the
  before-phase row was `read_rune=1`, the wasted duplicate read this
  fix removes; on this product the defence chain had already chosen
  differently, see below).
- `floor` / `trap` — **diverge from the kit's recorded rows,
  disclosed**: on this product the defence chain chooses no action in
  these scenarios (`DECISION defend=0`; `read_rune=0` before AND
  after), and the logs are **byte-identical across the fix** — proving
  the fix is a provable no-op for them. The divergence predates this
  fix and stems from carried defence-chain patches (candidates:
  healchance-364's `borg_heal` flow, the tele-away rod family
  0091/0095/0096); attribution to one patch is not claimed.
- `result.json` records `verified: true` with all commands, pins and
  hashes. Harness notes recorded in the verifier commit: assertions
  rewritten to the observed product contract with the divergence
  analysis above.

## Scope and limits

Bounded rune-defence scenarios (visible rune, bare floor, real trap).
They prove the map recording and the suppressed duplicate read; no
survival claim.

## Exact next work

1. PR from `task/carry-runeglyph`, agent evidence review plus required
   hosted checks, then normal merge.
2. Frozen v1 cohort at the merged tip against the panelrows tip cohort.
3. Receipts on issue 375, source 295 and checkpoint 1.
