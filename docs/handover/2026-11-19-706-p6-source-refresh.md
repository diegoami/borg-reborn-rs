## 2026-11-19: P6 source startup (#706)

Done: synchronized accepted main/P5 promotion 324697ed from track-p
9e0402d4 in an isolated task worktree. The original borg-rs --update
changed only read_on to 2026-11-19; commit pins, copies and the generated
index remain byte-identical to the accepted baseline.

Corrected on 2026-11-20 after #708 review 7030550773: the original report
missed already available borg-rs/zoo commits. A fresh direct upstream fetch
and final report observe borg-rs d75e17a5 (7 commits, 2 changed copied
documents) and zoo b4d9d6d3 (417 commits). New donor source is real;
it is explicitly deferred, not described as absent. Six mapped core files
changed, while both #680 subject-file blobs are unchanged.
Exact heads, report-time counts, copy semantics and core re-vendor deferral:
[source inventory](../research/P6-SOURCE-REFRESH.md).

Left: owner-selected re-review/CI and integration of this PR (#706), then
kickoff #707/#709 and neutral enchant/decurse contract #680. A behavior-changing
donor carry is separate work; none is smuggled into this inventory correction.
The owner chooses the reviewer. No task merge or main promotion is authorized
by CI alone; stop for the P6 milestone review after its tasks.
