# R434 power boundary preparation

Track R source `ba460186678ee07f6a3aad7053e27cf1e5b2c2b9` includes completed
item-use PR #442. Its exact-head CI670 passed all jobs, including the final
canonical archive gate; #435 is closed. Cemetery PR51 and supplemental PR60
are merged. Existing R435 receipts remain unchanged by this task.

Work is on `task/T434-power-port`, from `feature/track-r`, targeting that
feature only. [Boundary design](R434-BOUNDARY.md) and its source-hash inventory
are preparation only: no power wrappers, API additions, protocol changes,
parameter moves, new games or performance claims. All runtime bytes remain
the validated R435 input95fb364b.

#433 is now code-complete on Track P, not accepted/promoted. Its closure and
review PR #477 explicitly retain the owner review gate. Do not consume its
implementation head, merge #477 or implement P extraction in this R task.
After the accepted promotion and a separate audited carry, refresh this
inventory before writing the owned power policy. Source refresh was read-only;
newer M13 attack/town changes observed in references were not imported.

Local verification checks every inventoried source hash and lexical trait
mapping against the receiving tree, runs the existing R4 surface check and
confirms the runtime input hash is unchanged. No gameplay campaign is needed
for this preparation; future runtime validation requirements are in the
boundary document. Keep #434 and its PR in progress/draft until the full port
and all gates complete. Future gameplay evidence belongs in borg-cemetery.
