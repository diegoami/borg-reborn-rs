## 2026-11-22: No retreat from a lone immobile neighbour melee kills quickly (#933)

**Done.** `caution_retreat_immobile_kill_turns` (shipped 0, today's play) and
`caution_retreat_immobile_danger_percent` (shipped 100 since the review fixes; read only when the turns are on), from the white jelly death of seed 829474344 ([Cemetery case](https://github.com/diegoami/borg-cemetery/blob/claude/kind-brown-leqftg/studies/fastfrag-white-jelly-829474344/README.md),
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

**Review fixes (GPT-6 review of `acf21c80`).** The kill estimate uses the
expected melee damage without targeting bonuses (`melee_milli_per_turn`,
finding 1). No veto when cursed with fear or when the record is older than the
attack's own `attack_max_target_age` (finding 2). The whole fight must be
affordable: the danger here from every monster times the turns of the kill below
the percent of the HP, now 100 (finding 3). This note's defaults corrected
(finding 4). Merged `track-p` (`7afbdd67`).
