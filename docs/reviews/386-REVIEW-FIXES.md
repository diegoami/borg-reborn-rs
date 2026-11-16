# PR #386: response to the owner-supplied review

Reviewed boundary: `9f62ee6058e05d62f548d136fee430bb43cfa422`.
[Review comment](https://github.com/diegoami/borg-reborn-rs/pull/386#issuecomment-6904596402).
Runtime fix: `85021606646168799e9e121fc5b4f04c931385bd`.
The PR body names the exact publication head containing this evidence.
Re-review that diff; the owner still selects/runs the review. No merge or
acceptance is inferred from reviewer input or green CI.

1. **Starvation veto: fixed and reproduced.** A new controlled strategy
   fixture fails on the reviewed code: a foodless, starving one-way character
   facing reachable down stairs chooses Stuck/Hold. The fix applies the hunger
   descent veto only while Return is available. The fixture now requires both
   walking toward and taking the stairs. The existing ordinary-starvation
   fixture still requires Return; the normal target-depth gate is unchanged.
   This is a policy-state regression reproduction, not a claim of a live
   long-horizon starvation run or improved survival.
2. **Ordinary metadata: checked, retained.** Explicit false/zero metadata
   changes summary/config bytes. The promised byte identity is only the
   keys, messages and state streams; both ordinary and exact smoke again
   match 3/3. The cemetery exporter at `0216eb3` computes encounter family in
   `scripts/native/export-death.py:run_family/family_of` from named fields:
   implementation, seed/save RNG, actual race/class, source hash, Borg config
   hash, save hash and core-parameter hash. It does not hash the entire config
   for encounter identity. The new exporter regression verifies different
   config hashes but equal encounter families/repeat detection, and confirms
   repeat export preserves the original dossier's bytes. Payload hashes and
   immutable import package IDs **do** depend on exact bytes, intentionally.
   Historical records are preserved, and the compressed reference runs have
   no config snapshots requiring regeneration.
3. **Native enforcement: unchanged.** Engine forced descent and Recall's
   winner exception remain the rule; no policy bypass is added.
4. **Lifecycle provenance: clarified.** Patch 09 now names
   `src/game-world.c:run_game_loop`, full engine pin
   `42cd156ac9f95ddf4311e88dd7ad462292d6d1d6`, and the requirement to recheck
   departure/generation/arrival order when updating the engine pin. Its
   generated C behavior is unchanged; guarded-off/compile tests pass.
5. **Vendor/core checks: verified.** `scripts/borg-core-vendor.py check`
   passes against pinned donor `1443c2d1b97d`; PROVENANCE already declares
   #371's receiving hooks. Release core tests pass (328, four existing
   ignores), and real synthetic-start tests exercise the core arm as well
   as reborn. No new vendored-core edits are introduced by this review fix.
6. **Bad-depth partial output: retained.** As the reviewer suggested, keep
   the explicit engine-authoritative range check and diagnostic partial
   directory. Such failures never produce a successful natural-death record.

[386-REVIEW-FIX-EVIDENCE.json](386-REVIEW-FIX-EVIDENCE.json) records the clean
build, source/binary hashes, real depth-10/20 starts, initial states, stream
hashes and checks. Runtime source digest:
`3556b1a68e721330faa02909fe0e5012c3d0a58db115600e86da8950400c0d58`.
157+4+5 workspace tests, 90 tool tests, 19 script tests, formatting, Clippy,
vendor audit and 75 engine tests pass (two existing local dependency skips).
Original review prompt/evidence remains preserved for the earlier boundary.

Three new-source **natural** control dossiers are imported immutably to satisfy
the existing CI baseline gate; all earlier packages remain intact (2,559 total).
Controls reconcile with zero missing/unrecorded deaths. They are not synthetic
evidence or additional independent population samples. No synthetic registry
was created. Current publication-head CI, linked in the PR, is authoritative;
CI success on the earlier reviewed head does not validate this new head.

Scope and dependencies remain PR #377 / cemetery PR #6, birth builds only,
and follow-ups #382/#383. Independent PR #378 remains untouched.
