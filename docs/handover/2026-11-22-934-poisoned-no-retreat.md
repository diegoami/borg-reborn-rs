## 2026-11-22: Poisoned without a cure: escapes before the one-grid retreat (#934)

**Done.** `caution_poisoned_no_cure_retreat_hp_percent`, compiled and shipped 0 (today's
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

**Review fixes (GPT-6 review of `d4c41a11`).** "No cure" now means no usable
poison cure of any kind: Neutralize Poison or curing food, Cure Critical Wounds
(spell included), Healing, *Healing*, Life, or a castable Cure Poison or Herbal
Curing (finding 1). The rule no longer vetoes the retreat: with an escape at
hand it tries the escapes first, and when they decline the retreat goes on, so
a safe step is never lost (findings 2 and 3). Unit test for the cure
predicate. Merged `track-p` (`7afbdd67`).
