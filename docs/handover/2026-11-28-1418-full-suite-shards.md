## 2026-11-28: Shard the full reborn suite (#1418)

Three matrix jobs draw disjoint subsets from `harness/suites/full` and each
diffs against the matching subset of the existing reference. The inventory
check requires the complete suite and reference to agree. Each job runs the
perception and no-time checks. The existing required `engine-full` check
reports the matrix result and accepts a skip only when scope allows it.

Each shard has its own suite name in the output manifest and runs on its
own runner. The required check starts after those runners finish. The
combined stack has seven validation workers at peak, plus the separate
PR base check. Scope and the two result jobs are short jobs that run
before or after their workers.
