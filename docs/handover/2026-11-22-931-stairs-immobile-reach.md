## 2026-11-22: Stair flight keeps out of an immobile attacker's reach (#931)

**Done.** `caution_stairs_immobile_reach`, compiled and shipped 0 (today's
play), from the white jelly death of seed 829474344 ([Cemetery case](https://github.com/diegoami/borg-cemetery/blob/claude/kind-brown-leqftg/studies/fastfrag-white-jelly-829474344/README.md),
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

**Review fixes (GPT-6 review of `9cb2e644`).** The reach-free preference now
applies to whole routes that pass the clean-path check; when none is usable the
ordinary search runs, so a clean route past the jelly is no longer lost to a
blocked detour (finding 1). The policy on sleeping and stale records is stated
in the parameter and here: both count, and the fallback covers an impossible
detour (finding 2). The note fires only for a flight step that avoids a square
the ordinary route would enter, never from the reachability probe or a zero-step
path (finding 3). The choice is unit-tested (`prefer_reach_free`). Merged
`track-p` (`7afbdd67`). The rebase note above understated the conflicts: they
are in `params.rs` and `CALIBRATION.md`, chiefly citation lines.
