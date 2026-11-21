# Death evidence to verified Borg improvements

Owner direction, 2026-11-07; workflow [155](https://github.com/diegoami/better-borg-c/issues/155).
Two processes share evidence but have separate decisions. Owner decision
2026-11-09: implementing approved improvements in this repo is allowed, as
labelled local improvement units with evidence, gates and before/after
measurement, never proposed upstream. Still not automatic: a named candidate
with its contract precedes implementation. Bugs retain priority. An implementation task must name the
candidate, change, budget and acceptance plan before work starts.

For the canonical architecture specification, revision3 tuning plan/reviews,
source-check limits and quick gains368–371/toolkit376, read the
[reference inventory in PLAN.md](../../PLAN.md#borg-design-tuning-proposal-and-evidence-references).
These references inform scoped candidates and do not start their implementation.

## Process 1: analyse a death and evaluate candidates

1. Record the natural death using tools/record-death.py and qualify its replay.
   Preserve the original version, settings, seed/start, native logs and failed
   replay attempts. Repeated runs of one encounter are not independent deaths.
2. Copy owner feedback to the linked investigation under docs/deaths/FEEDBACK.md.
   Inspect the earliest disputed decision, not only the fatal blow. Capture
   actual turn AND phase, knowledge before action, resources, legal alternatives,
   action energy and subsequent consequences. Separate hidden diagnostic state
   from what the Borg knew. Source reasoning is not runtime branch proof.
3. Every processed death report includes a brainstorming pass AND an overall
   assessment, using docs/deaths/REVIEW-TEMPLATE.md. Explain the whole trajectory:
   preparedness/stock → threat assessment → entry/escape → recovery → fatal
   sequence. List competing explanations and candidate improvements, including
   no-bug/bad-luck outcomes; distinguish evidence, inference and owner hypotheses.
   State confidence, missing observations, dependencies and the next discriminating
   experiment. Share this complete picture with the owner, not only one threshold
   or the final blow. A death may involve several interacting mechanisms.
   Explicitly examine attempted walking retreat, even without a guaranteed-safe
   route, especially after Phase Door is exhausted. Record why the current chooser
   considered/rejected it or whether it was actually blocked. Do not equate no
   scrolls with no escape option, or unknown terrain with an unavailable route.
4. Search existing issues and ownership. Reusable original-C bugs go first to
   neo-angband-tracker/source207, then source fix and separate consumer carry.
   Consumer investigations and improvement candidates live here. Link a reusable
   source enhancement both ways; the local issue owns product evaluation and the
   adoption decision. Do not label an enhancement carried or put a policy in the
   bug patch ledger; improvement units are tracked separately from bug carries
   and are never proposed upstream. Shared mechanisms use one action, not an issue per replay.
5. For each candidate specify the intended benefit, affected cases/classes,
   costs, adverse effects and a falsifiable experiment. Link it from assessment
   feedback/actions; reply with findings and regenerate tools/aggregate-deaths.py.
   Outcome: investigate further, propose an experiment, reject, or inconclusive.
   Acknowledging feedback does not resolve its investigation.

## Process 2: implement an experiment and test its benefit

1. Before implementation/results, attach the completed experiment contract below
   to the candidate issue. Pin the actual baseline and declare the minimal
   behavior change. Keep the bug integration lane separate; if baseline changes,
   record it and restart the comparison rather than mixing versions.
2. Implement one hypothesis in an isolated task worktree, preserving the default
   comparator. Check the intended decision and unaffected controls in native
   execution; pass normal required gates. Instrumentation must preserve decisions
   and RNG. A controlled rescue branch is useful diagnosis, not survival evidence.
3. Use a bounded pilot to estimate cost and event frequency, then freeze a
   prospective comparison. Pilot/development seeds used to choose the policy
   cannot be its untouched confirmation set. Derive sample size from the desired
   detectable effect and uncertainty; three all-survivor shallow runs are only
   regression evidence. No convenient seed/limit changes after seeing outcomes.
4. Run baseline and candidate with the same independent starting seeds/settings
   and equal declared budgets, balanced run order and class coverage appropriate
   to the claim. One seed pair is one independent comparison. Changed actions
   cause RNG divergence; identical seeds do not guarantee identical later fights.
   Use the shared validation lock and watched bounded runs under angband-suite.
5. Preserve every scheduled attempt: death, victory, cap survivor, stall, crash,
   timeout and harness failure, with explicit denominators and missingness.
   A cap survivor is right-censored, not a victory; a stall is not survival success.
   Record every natural death through the death protocol. Keep raw data/hashes
   outside Git and curated summaries/reproduction commands in the repository.
6. Evaluate survival AND progress. Declare the primary long-horizon endpoint
   (for example reaching a specified depth alive within a fixed turn budget),
   survival endpoint and acceptable regression margins before runs. Report
   paired effect sizes and uncertainty, event counts and per-class results,
   time/depth/level/experience reached, town trips, consumable use and stalls.
   Report milestone reach rates and time-to-milestone including failures, not
   only speed among successful survivors. Longer survival by refusing to dive
   fails a progress gate. A shallow result cannot prove late-game benefit.
7. Select/tune on development data only. Compare the fixed candidate on untouched
   confirmation seeds with the same declared acceptance rule. Keep all tested
   variants and negative results visible; account for repeated comparisons rather
   than choosing a lucky winner. If uncertainty spans a material harm or benefit,
   report inconclusive and plan a new prospective study; do not assert improvement.
8. Publish an explicit adopt/reject/inconclusive decision, exact pins, numerical
   evidence, limitations and rollback route. Promotion needs functional correctness,
   survival evidence and the progress gate, plus required checks/normal review.
   A merged implementation is not a confirmed benefit. Rejected experiments remain
   evidence. Follow-up monitoring uses separately declared cohorts and regression
   cases; no automatic learning, tuning or scheduler is introduced.

## Experiment contract (copy into the implementation issue)

- Candidate/local issue; linked source proposal; death cases and disputed windows.
- Hypothesis, alternatives, confidence; what result would disprove it.
- Baseline commit/build/data/config hashes; candidate commit and exact change.
- Functional scenario, unchanged controls and native verification commands.
- Development/pilot seeds and untouched confirmation seeds; classes/birth/options;
  independent starts, pair IDs, run order and reproducible seed-selection method.
- Turn/decision/wall budgets; sample-size justification and stopping rule;
  infrastructure-failure handling and all-outcome denominators.
- Primary long-term endpoint and horizon; survival measure; progress gate;
  minimum useful effect, uncertainty method and acceptable regression margins.
- Secondary measures and multiple-variant handling; per-class safety checks.
- Actual commands/tool/observer identities; evidence location and hashes.
- Results including failed/unfavorable attempts; effect estimates/uncertainty;
  confirmation results, limits, decision and rollback/follow-up.

Status vocabulary: proposed → investigating → ready for experiment → experimental
→ confirmed / rejected / inconclusive. A new experiment or meaningful policy
revision gets a new prospective contract; it does not overwrite the earlier result.

## First candidates and distinct questions

- [154](https://github.com/diegoami/better-borg-c/issues/154) links
  [source370](https://github.com/diegoami/neo-angband-tracker/issues/370): early
  Phase Door/cure restock gate. Compare one-scroll, two-scroll and cure changes
  separately or explicitly treat them as a combined policy. Measure early death,
  town trips, time/reach rate to depth5 and the declared longer-horizon endpoint.
- [41](https://github.com/diegoami/better-borg-c/issues/41): survival-stock reserve
  priority over spare gear. It is related to, but different from, the depth gate.
- [34](https://github.com/diegoami/better-borg-c/issues/34): first Phase Door,
  crowd landing, walking retreat and later healing eligibility. D0004 had four
  known Phase Door and twelve cures at20293. A departure reserve alone is not
  proved to prevent this death; stock availability and action selection differ.

[153](https://github.com/diegoami/better-borg-c/issues/153) records a future
viewer danger/decision pane. It is file-only and deferred; this process does not
implement it or alter Borg behavior.

## Owner brainstorming candidates (2026-11-07)

[158](https://github.com/diegoami/better-borg-c/issues/158): assess theft danger
weighting and whether economic penalties trigger harmful early escape decisions.
[159](https://github.com/diegoami/better-borg-c/issues/159): assess attempted
room/door/upstairs retreat before repeated phases AND after scroll exhaustion.
Both are focused children of34, linked to the same encounter evaluation rather
than new independent death claims. Sleep timing, safe fight and retreat outcomes
remain unproved. [Current overall assessment](../deaths/D0004-named-headless/overall-assessment.md).
