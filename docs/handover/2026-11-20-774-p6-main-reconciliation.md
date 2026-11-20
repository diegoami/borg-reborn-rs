## 2026-11-20: P6 pre-promotion reconciliation (#774)

Done: task branch starts at reviewed track-p e9670cabbc0e36bcfcfdfa84a2bc878a6134e72e
and merges exact accepted main 4e9af99ea51b531bdcf8c916ee6a37be72bef047.
Initial merge 27700983c9e6ec1427cccfd7ac39b42009963967 has those two parents;
its tree is cb2eff4ed89ce9baa2d6c06df74f002189f1a1de. The only conflict was
CALIBRATION.md append/citation context. Both documentation sections are
retained, with main's current Speed-for-fight citation kept once; an initial
duplicate stale row was removed before validation completion.

Found: the two subject modules on main are unchanged from the #680 receiving
source. Merged subject modules are byte-exact reviewed #680 code. Subtracting
the reviewed ten entries/test/prefix map from params.rs recovers main's exact
registry/parser bytes. Main's newer engine 574b618c, settings, other runtime
files, harness/suites/references, tools/fixtures, all commit pins and RULES/AGENTS
are preserved. Only the reviewed four-file P6 implementation delta remains.
The read_on metadata and P6 docs are retained; no further donor carry or tuning.
Historical #680 notes and immutable Cemetery 0529ff6f are unchanged.

Validation in progress: required Rust/vendor/citation/engine checks and native
smoke, plus the named seed1001/race1/class0 override case. Current RULES20
routes comparisons over sets of starts to Zoo: no local full45 batch is run.
The exact-build broader comparison request, result disposition and any new
canonical evidence will be linked in this task's PR/note. Original source
receipts cannot be relabelled as current-main proof.

Left: #774 Zoo comparison receipt, owner-selected task review/green CI and
integration into track-p, then pinned P6 milestone review. No source main
promotion, R4 carry, independent-life inflation or population claim is implied.
Optional #773 evidence-presentation notes are addressed additively in the new
evidence documentation, pointing at the already retained default trace and
explaining the quiet stdout log; old canonical bytes stay fixed.
