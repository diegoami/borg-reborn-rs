# Review prompt: Tunable Borg Plan

Paste the block below into a fresh reviewer session. The reviewer reports;
it does not change any repository, issue or pull request.

````text
You are reviewing a plan, not code: docs/c-borg/TUNABLE-BORG-PLAN.md on the
branch task/tunable-borg-plan of diegoami/neo-angband-tracker. Decide whether it
is a sound basis for the owner to approve, and say plainly what is wrong or
missing. Do not fix it. Do not push, merge, open or edit issues or PRs in any
repository, and do not run anything in another session's worktree.

## What the plan is for

The owner wants a better Angband Borg: one whose parameters and behaviours can
be tuned and compared by experiment (survival and progress), built on what the
original C Borg knows rather than from scratch, but not bound to its decisions.
Two repositories share that goal:

- diegoami/better-borg-c: a native C Angband/Borg product at upstream
  e473720ed88a0f0c7ca66815cef3000eb4a38bc3, carrying verified bug fixes from
  neo-angband-tracker, with a native measurement runner and death dossiers.
- diegoami/borg-rs: a faithful Rust port of the C Borg, verified by
  byte-identical key streams against the C Borg on seed suites.

The plan recommends: first carry better-borg-c's fixes into borg-rs (step R1,
with a deliberately light check); then expose parameters with C-Borg defaults
(identity at defaults), tune by racing on frozen seed sets, and replace modules
one at a time behind switches; both repositories share one parameter table,
harness contract and result schema, and must agree key for key at the same
profile. The long-term base (C or Rust) is decided later from data.

## If this is a re-review

Revision 1 (`7e008cc93`) was reviewed in neo-angband-tracker issue #360. For
revision 2 and later, first check item by item that each error, omission and
judgement call in #360 is resolved or explicitly answered in the plan, then do
the full review below on what changed.

## Read first

- docs/c-borg/TUNABLE-BORG-PLAN.md (the plan under review)
- docs/c-borg/BORG_SPEC.md (the C Borg specification the plan relies on; on
  PR #317 if it is not yet on master)
- better-borg-c: README.md, PLAN.md, RULES.md, AGENTS.md, patches/series.json,
  docs/measurement/README.md, BASELINE-V1.md, DEEP-IMPACT-V2.md,
  IMPACT-MILESTONE2.md, docs/deaths/
  (gh api repos/diegoami/better-borg-c/contents/<path>, or a read-only clone
  in your own scratch directory)
- borg-rs: README.md, CLAUDE.md, docs/REVIEW.md, docs/STATUS.md,
  docs/harness.md, harness/suites/*, borg-rs/src/state.rs (the local checkout
  at ~/projects/borg-rs is read-only for you; or clone into scratch). The owner
  authorises this read for the review; it is not routine polling.
- Upstream source at the pin: neo-angband-tracker's origin/c-borg-reference
  (git archive it into scratch). Do not use reference/ (that is 4.2.6).

## 1. Check the facts the plan states

Verify each, and report accurate / inaccurate / arguable with the source:

- better-borg-c: number of carried fixes and their order (patches/series.json);
  runner behaviour (determinism, outcome classes, observer instrumentation);
  run cost (about 95–113 s per 400,000 engine turns, 8–15 s per 20,000);
  cohort sizes used so far; the scope rules the plan says must change
  (RULES 12, AGENTS.md).
- borg-rs: milestones M1–M3, module count and size, where `unsafe` remains,
  suites and their sizes, verification limits (deepest level reached, what is
  not exercised), the BorgState design, its "faithful port, no new tactics,
  C bugs kept" rules, and the 4-core machine.
- The C Borg: the formula section in src/borg/borg.txt and what
  borg_uses_dynamic_calcs replaces (borg-power.c, borg-prepared.c,
  borg-formulas.c); every row of the "first tuning surface" table against the
  named functions; the count of numeric literals (re-derive it and say how).
- The statistics: re-derive the sample sizes (two proportions, two-sided
  alpha 0.05, power 0.8) and the core-hour figures.

## 2. Judge the recommendation

- Is "parameterise in place, then race, then replace modules" right for this
  goal? Argue the strongest case for an alternative (rewrite from the spec, a
  learned policy, tuning only the existing formula section, or doing
  everything in borg-rs from the start) and say whether the plan answers it.
- Is the identity-at-defaults invariant achievable in C as described (many
  constants, compile-time folding, the Borg's private RNG), and is it the
  right gate?
- The objective: does "survival plus progress on a Pareto front" avoid the
  never-dive trap? Is dlvl D by turn T a good primary measure? What would you
  measure instead?
- The harness: are the four levels the right ones? Is the situation suite
  useful while the Borg's memory cannot be restored on load? Is borgh the
  right shared frontend, given that better-borg-c's runner uses the test
  frontend and an observer build?
- The search: are screening, then racing, then held-out confirmation
  appropriate for noisy, censored, expensive runs? What would you change?

## 3. Challenge R1, the first step

The owner asked for a light check: port all fixes one commit each, compare
the borg-rs suites once against a better-borg-c build of the same carries,
bisect only on divergence, no per-fix native scenarios, and a side-by-side
read for fixes the suites never reach.

- For each carried fix, say whether the existing borg-rs suites are likely to
  execute the changed code (look at what each patch changes and how deep the
  suites go). List the fixes that would rest on the side-by-side read alone.
- Is the single end-of-batch comparison enough, or is there a cheap addition
  that would catch a wrong translation the suites miss? Do not propose
  anything heavier than needed. The owner considers per-fix scenarios
  overkill.
- Does comparing against "a better-borg-c build of the same carries" need the
  borgh frontend in better-borg-c? If so, say what that costs.

## 4. Feasibility, scope and risk

- Compute: is the plan realistic on the stated machines, given the shared
  validation lock and other sessions' gates?
- Governance: which existing rules in each repository conflict with the plan,
  and does the plan's "decisions for the owner" list cover them all?
- Two codebases: is keeping C and Rust in step worth its cost? When should
  the plan stop doing it?
- Anything stale or likely to go stale (counts, pins, run times)?

## Report

- Verdict: approve / approve after the listed changes / do not approve.
- Facts checked: a table of claim, source, and accurate / inaccurate /
  arguable. Say plainly what you did not check.
- Findings, kept separate: errors (false statements), omissions, and
  judgement calls. For each: where it is, why it matters to the owner's goal,
  and what you would do instead.
- R1: the per-fix coverage list and your verdict on the light check.
- The most important thing: if the owner approved this plan as written, what
  would most likely go wrong first?
````
