# Review #260: the track-p promotion gate

Reviewer: an independent session, a different model family from the
implementation family (OpenAI GPT-6 / Codex).

* Reviewed workflow: `f693e69ba48da35598ca7bc35ec5f13f5db9f564`
* Against main: `8351f0a394987f9dd8c2daa5e243e276b464c00d`
* Scope: the workflow recorded by #258 (issue #257), plus the routing
  rules that AGENTS.md, RULES.md, PLAN.md and docs/GOALS.md set.

The full retrospective code review is #259; this review covers only the
workflow and the promotion gate.

## Verdict

**Accept the workflow.** The gate is coherent, consistently recorded
across the four entry documents, enforced by CI, and it does not
retroactively rewrite `main` or claim a review that did not happen. One
presentational hazard in the PR's own diff view should be corrected
before the owner merges it (F1), and one gap in the rule's coverage of
the *other* track is worth closing (F2).

## What was checked

### The merge result is exactly what the PR body claims

The PR body states that "this PR's current diff contains only policy
documents and CI triggers". Verified by computing the merge tree:

```
$ git merge-tree --write-tree main origin/track-p
72155ac9cf09fbbfb6ad9493acd7a8a3927cb250

$ git diff --stat main 72155ac9cf09fbbfb6ad9493acd7a8a3927cb250
 .github/workflows/ci.yml |  2 +-
 AGENTS.md                    | 35 +++++++++++++++++++++++++++++---
 PLAN.md                      | 26 ++++++++++++++++++++++-------
 RULES.md                     | 26 ++++++++++++++++++++++-----
 docs/GOALS.md                |  2 +-
 docs/HANDOVER.md             | 18 ++++++++++++
 6 files changed, 96 insertions(+), 13 deletions(-)
```

Six files, all documents or the CI trigger. **No runtime code.**

### The CI trigger covers `track-p`

`.github/workflows/ci.yml`:

```yaml
  push:
    branches: [main, track-p]
```

A push to `track-p` runs the same job as a push to `main`, so a P task
merged into `track-p` cannot skip CI.

### The rule is recorded consistently in the four documents

* **AGENTS.md** — "Track P integration and review gate (owner,
  2026-11-15)": start P branches from `origin/track-p`, target P task
  PRs at `track-p`, merge there only after CI; never send a P task PR
  straight to `main` because CI passed; at each milestone stop and open a
  pinned `track-p` → `main` review PR; the owner selects and runs the
  review; fixes land through `track-p` task PRs; promotion stays unmerged
  until the owner accepts and CI passes; the owner merges or expressly
  delegates; the reviewed head stays fixed while the review is active,
  with a new exact pin recorded after fixes; `main` changes are
  synchronized through a CI-checked PR into `track-p`.
* **RULES.md 1** — the same rule, as the first numbered rule, with the
  explicit statement that "CI success is not review approval".
* **PLAN.md** — a "Track P exception" at the head of the milestone
  section and a Track P row giving the integration branch, the task PR
  base and the milestone review/promotion pair; the P1/P2/P3 rows' state
  changed from "in progress"/"not started" to "implementation on main;
  retrospective review pending".
* **docs/GOALS.md** — the owner decision recorded alongside the other
  decisions.

### The historical exception is stated honestly and in one place

Both AGENTS.md and PLAN.md say, in substance, that P1–P3 were already on
`main` at `8351f0a3` before the rule, that they need retrospective
review, and that this "does not certify prior independent review" and
that main is not rewritten or reverted to simulate a pre-merge review.
That is the honest description of the situation, and it is stated
identically in the two documents that a session reads before starting
work.

### The exit criteria of the milestones still hold

PLAN.md's P1/P2/P3 exit conditions are unchanged by the rule — every
threshold a parameter or listed game data, full 45/45 at the defaults, a
changed parameter changes play and the trace shows it, the trace in run
outputs and death dossiers, and the zoo contract check. The review
prompt's own claim that the policy-only change "no runtime inputs" is
confirmed: the six merged files contain no source, and the runtime
inputs are unchanged.

## Findings

### F1 — the PR's "Files changed" view is misleading, and the PR body should say so

`main` has four commits that `track-p` does not have (the `/shopping`
work of #267, merged after `track-p` was last synchronized). GitHub
renders a PR as a **two-dot** diff, so the PR page lists
`borg-core/src/shell.rs`, `borg-core/src/borg_caution.rs` and
`engine/tests/test_shopping.py` as *removed*, and lists the D2356–D2358
dossiers as deleted.

None of that would happen. The three-way merge keeps `main`'s version of
every file `track-p` never touched — verified above: the merged tree
still contains 20 shopping references in `shell.rs` and
`engine/tests/test_shopping.py` is present.

**Remedy**: add one sentence to the PR body saying that the diff view
shows `main`'s newer `/shopping` work as reversed because `track-p` has
not yet synchronized it, that the merge result is the six policy files
only, and that `track-p` should be synchronized with `main` through a
CI-checked PR before promotion (as the rule already requires for `main`
changes). This avoids a reviewer reading the file list and concluding the
promotion would revert a feature.

### F2 — the rule governs track P only, and the "other tracks keep their own workflow" clause is untested

RULES.md 1 and AGENTS.md both scope the gate to track P and say other
tracks keep their authorized workflow. Track R in fact works on
`feature/track-r` with its own review PR (#266), which is consistent.

The gap is that the rule says a track-R port may only begin "after the
relevant P milestone has been reviewed and promoted to `main`". P1–P3
have **not** been promoted — they are on `main` only because they were
merged before the rule. Track R's R1 work was carried out anyway, on the
strength of the "already on main" historical exception.

This is defensible, and the exception is stated in the documents rather
than hidden. But the dependency is written in terms of a promotion that
has not happened, so the rule as written does not describe what actually
happened. Two ways to close it:

* Have #259's retrospective review explicitly accept P1–P3 (which is
  what #259 proposes), and record in the handover that track R's
  dependency on P's thresholds is satisfied by that acceptance for the
  already-merged work — or
* Amend the dependency sentence to "after the relevant P milestone has
  been reviewed, and promoted to `main` for work done under the rule; the
  pre-rule work is covered by the retrospective review".

This is a documentation-precision point, not a defect in the gate.

### F3 — nothing to flag in the task-issue closure rule

The rule requires the task issue to be closed explicitly with its
integration PR or commit, and requires findings to be fixed through
`track-p` task PRs. Both are consistent with RULES 5's existing "one task
issue per PR" and "the PR says `Closes #N`" and add no ambiguity.

## Checks rerun

| Check | Result |
| --- | --- |
| Merge tree of `main` ← `track-p` | 6 files, documents and CI trigger only |
| CI trigger includes `track-p` | yes (`.github/workflows/ci.yml`) |
| AGENTS.md / RULES.md / PLAN.md / GOALS.md consistency | consistent; the same gate in all four |
| Runtime inputs changed | none |
| #259's retrospective code review | separate; see `docs/REVIEW-259-P1-P3.md` |

The PRs #259, #260 and #266 all show green CI on `rust`, `engine` and
`recorder` at the time of this review.

## Not checked

* The reviewer itself was not selected or launched by this session; the
  owner chooses and runs it (RULES 6). This document is an independent
  session's assessment of the workflow's internal consistency, not the
  milestone review the owner is commissioning.
* No population experiment was launched and no sealed suite was opened.
* #266 (track R's R1 review) was assessed separately; see
  `docs/REVIEW-266-R1.md`.
