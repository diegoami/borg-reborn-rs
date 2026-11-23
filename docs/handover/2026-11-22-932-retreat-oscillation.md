## 2026-11-22: No retreat after a run of retreats that the next step undoes (#932)

**Done.** `caution_retreat_oscillation_limit` (shipped 0, today's play) and
`caution_retreat_oscillation_thinks` (shipped 3, read only when the limit is on), from the white jelly death of seed 829474344 ([Cemetery case](https://github.com/diegoami/borg-cemetery/blob/claude/kind-brown-leqftg/studies/fastfrag-white-jelly-829474344/README.md),
borg-cemetery#116). `docs/CALIBRATION.md` has the section and the rows;
`harness/borg.txt` the shipped lines; citations re-pointed with
`scripts/check-param-citations.py --fix`.

**Checked.** CI's commands; smoke on the default core identical to
`smoke-borg-rs` (3/3) at the shipped 0; the zoo's own game (seed 829474344 on
`7f8c6b80` plus this change, a local branch never pushed): switch off, key for
key the zoo game (dead at 96,820); switch on, see the PR.

**Left.** The population measure is borg-zoo#654 (one arm per switch of
#931-#934, specimen `172bd671`, the four together on `track-p` `e2d8919e`).
Thresholds are hypotheses (RULES 14). #930 (the ghost record that kept the level
scary) and #935 (out of arrows, no Recall) are not part of this task.

**Owner.** Merge into `track-p` (P8) after the review (RULES 1a). #932-#934 each
add a line to the same retreat condition in `borg_caution`; the ones merged
later need a one-line rebase there and in the `CALIBRATION.md` head.

**Later commit (#1023).** The issue was split (owner, 2026-11-22): this PR now
closes #1023, the general breaker; #932 keeps the stair-flight side (#964).
Added `caution_retreat_oscillation_backaway` (shipped 0): with it, back-aways
join the run and are vetoed too (seed 1512540769, white worm masses: back-aways
undone by the stair flight). The zoo request #654 measures the retreat-only
form (its specimen `172bd671` has no back-away flag); a back-away arm needs a
new specimen.

**Review fixes (GPT-6 review of `429a286d`).** A move away now continues the
run only when its landing was confirmed (a later think found the Borg on the
grid it stepped to) and the Borg then came back to, or next to, the grid it
had left without standing on that landing: steps that keep going (A to B, B to
C) no longer count (review finding 1, P1). One veto note per think (finding 2).
This note's defaults corrected (finding 3). Decision-sequence tests:
go-and-return counts; going on, an unconfirmed landing, another level and an
expired window do not. Merged `track-p` (`7afbdd67`).

**Second review fixes (MiniMax review of `71d2e9c1`).** The citations of
`caution_retreat_oscillation_thinks` and `_backaway` pointed at a line near
the read, not at it (the checker accepts any read in the file); they now cite
the reads (`borg_caution.rs:2294`, `3887`), in `params.rs` and
`CALIBRATION.md`. No code change.
