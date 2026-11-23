# Historical import/provenance records

One file per CBUG ID preserves discovery/source correspondence and its
canonical source issue. evidence_at_import/status_at_import/proposals_at_import
and next_at_import describe the migration snapshot, not current workflow.
Edit the issue for live investigation, proposals, conclusions and source
resolution. Do not maintain another carry/status ledger in these records.

catalogue.json holds source/group/claim coverage; the original monolithic
Mandos JSON is replaced by these split audit records. tools/c-bugs.mjs checks
inventory, pinned contexts, exact discovery snapshot and coverage; --write
creates only the issue-list landing page and data-only watch-index.json.
Carry/fixed consumer state lives in linked Mandos integration issues only.
