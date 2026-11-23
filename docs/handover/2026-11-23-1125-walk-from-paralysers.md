## 2026-11-23: Walk away from paralysing monsters (#1125)

**Done.** Four switches, one per point of #1125, all compiled and shipped 0
(today's play). They come from the floating-eye death of Half-Elf
Necromancer 453254028 (borg-cemetery#141). `docs/CALIBRATION.md` has the
section and the rows, and `harness/borg.txt` the shipped lines. Citations were
re-pointed with `scripts/check-param-citations.py --fix`.

- `escape_paralyser_fight_lost`: next to an awake paralyser without Free
  Action, the Phase rule's fight is lost whatever the danger says.
- `caution_paralyser_step_away`: next to a never-moving paralyser without
  Free Action, the think's first action is a step out of its reach.
- `caution_paralyser_no_reach`: without Free Action, no flow enters a known
  paralyser's grid or the eight around it, stair flights included.
- `caution_remember_never_move`: a `NEVER_MOVE` record survives the
  anti-bounce reset and the 2000-turn expiry.

**Found.** The eye was not forgotten because it left view. The anti-bounce
reset (`borg_check_bounciness`, counts 500-503) wiped every monster record
after a 400-turn loop between `dark-bfs (ring 3)` and `targetable position`.
That same high count made `bouncing()` refuse every grid of a step away. So
the paralyser's step skips the bounce test: a step out of a never-mover's
reach cannot ping-pong.

**Checked.** CI's commands: fmt, clippy, both test suites, the vendor check
and the citation check. Smoke on the default core is identical to
`smoke-borg-rs` (4/4). Unit tests cover the decisions with the race facts
given (unit tests have no engine tables). The replay is seed 453254028 from
birth, 100,000 turns, on the death's build `d8b55d64` plus this change (a
local tree, never pushed). Off plays the death key for key. 1 alone, 4
alone, 2+3 and all four live to the cap. 2 alone and 1+2 die: the step
comes first, then the next flow walks straight back beside the eye. 3 alone
plays the same game, because the eye had already been forgotten. This is one
start, not a rate (RULES 14).

**Left.**
- The population measure: a borg-zoo request, drafted in the PR, not filed
  yet (see Owner).
- The 400-turn flow loop next to the eye (`dark-bfs` against `targetable
  position`) is not addressed here. It is its own issue, to be filed.
- #1099, the same step for an immobile confuser after the floor's cure, is
  still the owner's decision. `best_step_away_from` can serve it.
- #852 (the ranged-only scary entries) is unchanged.

**Owner.**
- Should the zoo request be filed?
- Switch 2 is only useful with 3 (and 3 needs 4 when the record was
  dropped). Should the zoo arms be bundles rather than singles?
- The two dying replay arms (2, and 1+2) are counterfactual trials of a
  recorded case. They are listed in the PR, and no Cemetery package was
  made.
- Merge into `track-p` after the review (RULES 1a).
