## 2026-11-21: final P6 main reconciliation (#814)

Done: merged accepted main07aa4b1e92642b615e162773800979a97baf0dd6
into track-p221a2504779d6b5fb449dbf01907aeef1a505b10 at
a3635ad80ceb04165aa1d465acd908c158e50478 (treec960c16fb810b15d6e7b49538204f96ed2961078).
Resolved three conflicts: preserve both the P6 and main resolved-settings
receipt tests in params.rs; preserve main docs plus the P6 contract/provenance.
Subject modules stay exact reviewed #680 bytes. Subtracting reviewed P6
entries/test/prefix lines recovers main's exact registry/parser/tests.
All other runtime/settings/engine/references/commit pins preserve main,
including native tools46978936. No further donor carry or tuning.

Checks: core568/4 ignored, engine/contract93/4 skipped, required fmt/clippy/
workspace/vendor,2293 citations,39 script and96 tool tests pass. Native smoke
is3/3 identical keys/messages/states against main's reference. The original
named town-limit0 controls at100k and extended300k had no effect and remain
retained as negative coverage. The observed action is dungeon curse removal:
dungeon-limit0 at the original100k cap changes key2711/turn19991/think1944,
from borg_enchanting to borg_flow_dark, with environment provenance.
Eight local attempts reuse one named start plus smoke; all alive, no new deaths.
No broader local cohort. Source-refresh Zoo fetch was stopped to avoid disk
exhaustion; its error and direct upstream HEAD observation are retained,
not interpreted as no source advances. No source/pin change.

Canonical local evidence: [immutable packet](https://github.com/diegoami/borg-cemetery/tree/274b9dfea1fd3075f9f8e1cdf12b2745a2045f09/validation/borg-reborn-rs/p6-final-reconciliation-814/),
Cemetery PR #97 (196 verified artifact hashes and exact executable). Source
PR #818 carries the exact review prompt; runtime remains a3635ad8.

Left: source/evidence owner review and CI, plus Zoo571's new pinned
comparison disposition, then integration into track-p and pinned
P6 milestone review. Zoo520's successful full45 remains historical to its
own4e9af99e/d0657712 refs. No main promotion or R4 carry/closure authorized.
Old handovers and immutable0529ff6f/be61aeff packets remain unchanged.
