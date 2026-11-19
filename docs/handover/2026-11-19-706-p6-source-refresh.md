## 2026-11-19: P6 source startup (#706)

Done: synchronized accepted main/P5 promotion 324697ed from track-p
9e0402d4 in an isolated task worktree. Ran the six-source report and
borg-rs --update in a private donor clone; only its read_on date changes.
Reference copies and the generated death index stay byte-identical.
Exact observed heads and deferrals: [source inventory](../research/P6-SOURCE-REFRESH.md).

Found: borg-rs has no new source. The eleven newly listed C-Borg reports
are unassessed. Other source advances are inventoried rather than adopted;
accepted main runtime, smoke references, tools and evidence are preserved.

Left: owner-selected review/CI and integration of this PR (#706), then a
separate P6 kickoff PR and neutral enchant/decurse contract (#680).
The owner chooses the reviewer. No P6 task merge or main promotion is
authorized by CI alone; stop for the P6 milestone review after its tasks.
