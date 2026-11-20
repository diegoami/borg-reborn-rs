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

Left: complete candidate/native override checks, retain canonical evidence
links, obtain owner-selected task review and green CI, then integrate #680
into track-p. Stop after that for the pinned P6 milestone review; no main
promotion or R4 carry is authorized here. All limitations and validation
identities belong in the PR/canonical receipt, not an improvement claim.
