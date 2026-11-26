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

Local validation passes: 505 release-core tests (4 ignored), fmt/clippy/
workspace/vendor, 2264 citations, 39 script tests, 96 tool tests and 92
engine/contract tests (4 skipped). Initial engine setup failed because a
shared Zoo checkout differed from the retained pin; a shared-clone attempt
also lacked cached blobs. A fresh direct checkout at exact Zoo 7b373a70
resolved the setup, without changing source or pins; failure logs are retained.

Native smoke is 3/3 identical keys/messages/states against the preserved
main reference. The named seed1001/race1/class0/menu/cap100000 pair stops
alive and proves the override still controls a real command: at key10604 /
turn80689 / think7928, enchant_town_max_turns=0 (environment) changes
borg_enchanting to borg_flow_shop_entry. The old turn23238 witness is not
claimed on this engine. Build source38947c1a / binary02bdfdea are pinned in
[canonical local evidence](https://github.com/diegoami/borg-cemetery/tree/be61aeff63f19ad275de265a1550aba5d7d392b5/validation/borg-reborn-rs/p6-main-reconciliation-774)
(Cemetery PR #93, pending review). Its default and override traces are each
retained independently; no natural deaths occurred in these five checks.
Build clean; smoke checkout had documentation-only dirtiness while the
citation conflict was corrected, inputs unchanged and overrides=none.

Broader default comparison is requested in [Zoo #520](https://github.com/diegoami/borg-zoo/issues/520)
at exact baseline4e9af99e versus published candidate d0657712 (runtime
identical to merge27700983; later completion edits documentation only).
Pilot first; any exact-native45 follow-up waits for its gate and owner
agreement. RULES20 routes sets of starts to Zoo: no new local full45 batch
was run. Original #680 receipts are preserved and are not new-main proof.

Left: #774 Zoo comparison receipt, owner-selected task review/green CI and
integration into track-p, then pinned P6 milestone review. No source main
promotion, R4 carry, independent-life inflation or population claim is implied.
Optional #773 evidence-presentation notes are addressed additively in
Cemetery #93's README, pointing at the already retained default trace in
0529ff6f and explaining quiet stdout; old canonical bytes stay fixed.
