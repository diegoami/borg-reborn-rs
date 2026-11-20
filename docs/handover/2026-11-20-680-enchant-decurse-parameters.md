## 2026-11-20: neutral enchant/decurse parameters (#680)

Done: classified both subject files and extracted ten policy controls with
inherited defaults. Existing BORG_ENCHANT_LIMIT, mechanics/UI, operation
widths, resource order and trace@2 selection are preserved. New registry
entries use existing profile/environment/provenance support. Added bounded
profile/provenance checks; the native override exercises production decisions. See
[contract](../CALIBRATION.md#enchantdecurse-policy-680--p6).

Found: the accepted receiving baseline is track-p 760fae80 (engine 42cd156ac,
donor 8c5d000a). Newer main/donor source is deliberately not synchronized in
this task; eventual promotion must reconcile that gap explicitly. The first
local baseline attempt failed from disk exhaustion and missing bc; raw output
and failure log are preserved. The clean retry completed all 45 starts; a
post-suite compression rename race was resolved by verifying duplicate bytes,
without replaying or changing gameplay results.

Checks: strict native full45 defaults match keys/messages/states and all
45 complete trace@2 streams. Setting enchant_town_max_turns=0 changes the
queued command and native selection at turn 23238 (seed1001, race1, class0).
473 release-core tests (4 ignored), fmt/clippy/workspace/vendor/citations,
34 script and 96 tool tests pass. Original comparison inputs are clean.

Canonical evidence: [Cemetery bundle](https://github.com/diegoami/borg-cemetery/tree/0529ff6f7208b197a5ee156be180395e7fbef38e/validation/borg-reborn-rs/enchant-decurse-680),
publication PR diegoami/borg-cemetery#90, pending owner review. It retains
both exact binaries, paired run/config/parameter receipts, complete equal
streams, the changed override, failures and two immutable natural packages.
Paired controls/retries are not independent lives. No gameplay improvement
claim. Candidate compression emitted a rename warning, handled by byte checks.

Left: owner-selected review/green CI and #680 integration into track-p;
canonical evidence PR #90 review/integration; then #774 explicitly synchronizes
accepted newer main and revalidates before the P6 milestone review. No main
promotion or R4 carry is authorized here. The task's tested implementation
is 710ba18f; later completion-note changes do not change its runtime inputs.
