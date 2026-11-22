# R4 immutable milestone review prompt

The review PR supplies SOURCE_SHA, BASE_SHA, canonical cemetery commit and
manifest SHA, source task CI run and exact build identity. Missing pins fail
preflight; do not substitute latest. Implementation family is Codex/GPT-6;
the owner selects/runs the reviewer and accepts the milestone.

1. Check out SOURCE_SHA; read AGENTS.md, RULES.md, R4-PLAN.md,
   R4-EXIT-REPORT.md, R438-HANDOVER.md and every slice's ownership manifest.
2. Run formatting, workspace clippy/tests, core release tests, scripts/tests,
   tools/tests, source/vendor/P4 guards and each power/equipment/store audit.
   Run `python3 scripts/r4-code-metrics.py` and reproduce every published count.
3. Verify the canonical receipt and manifest byte inventory, original build
   lineage and qualified registries. Distinguish existing games, reused runs,
   copied scenarios and independent games. Reconcile every natural death.
4. Reproduce all three smoke/full45 comparisons against their exact named
   baseline, parameters/endpoints, zero mismatches, complete public-entry and
   immutable-root coverage. Check strict expected modes and source/build pins.
5. Inspect positive and early-return controls, quantities/charges/restoration,
   output parameters, nested callback/effect/RNG-once and literal mutation/repair.
   Queued commands do not establish engine consumption or natural branch coverage.
6. Audit shared M13/dynamic/notice/stock/maintenance boundaries and explicit
   #293 non-delivery. Compare scope/cost claims with evidence; reject unsupported
   performance, survival, independent-game or complete-branch claims.
7. Verify the immutable review branch and base are exactly those in this PR;
   no main/global-reviewed promotion or R5 implementation is authorized.

Report each check as reproduced/failed/not checked with file:line or immutable
artifact citations. List blockers, limitations and model family; provide a
verdict without treating CI success or this review as owner acceptance.
