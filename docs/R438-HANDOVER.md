# R438 combined reporting and milestone review

Starts from accepted #437/#780 merge dc1491b9 on task/T438-r4-review-checkpoint.
R4-EXIT-REPORT.md records the combined scope, limits and remaining canonical
publication gate. R4-CODE-METRICS.json is generated from the accepted source;
no gameplay change or new measurement campaign belongs to this task.

The prepared combined canonical package retains power/wear/store reports,
original build identities, cumulative constructed controls, qualified original
death packages and per-run inputs/outcomes. Large raw streams remain preserved
locally and checksum-identified; the package is not a standalone replay archive.
The owner explicitly approved this exact runtime metadata publication. Cemetery
PR #114 retains it; R438-EVIDENCE-RECEIPT.json pins the immutable containing
commit and original manifest. The separate CI evidence job checks payload
hashes, qualified original record bytes, canonical sidecars and aliases using
the exact pinned Cemetery identity implementation. Historical R364/R365/R435
receipts and their reconciliation workflow remain unchanged.

The source PR must pass CI and wait for owner review completion before merge.
Then freeze an immutable review/R4-SHA into review/track-r-baseline with a pinned
review prompt and stop. No R5, main or global-reviewed promotion. #293 remains
open and shared maintenance #680 is not silently counted as ported.
