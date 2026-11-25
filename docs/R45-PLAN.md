# R4.5: catch up with the shipped core (#856)

The owner accepted corrected R4 snapshot
`40cbaf272193abb44dd80fad2cfeec24b627be22` on 2026-11-23 and selected
one exact receiving target:
`main@24be98732748230062882fe2b7d9601dba5e2cc0`.
R5 stays paused until this catch-up milestone is reviewed and accepted.

The target includes owner-promoted P6 (#822). GitHub's comparison confirms
it is 31 commits ahead of promotion `f57499d09ba30e84c2e06e5ac08a008c856f9a6d`
and not behind it. Unpromoted track-p commits are not a source for this carry;
changes already shipped on the named main pin are included regardless of their
origin. Later main changes wait for a later carry. The selected pin does not
move when either main or track-p advances.

## Measured scope

Run `python3 scripts/r45-preflight.py` with all three exact source commits
available. R45-PREFLIGHT.json pins its comparison inputs and complete per-file
counts. Historical common base `2f74e1c5d57ce2b41ded1b5b694174b91b51f265`
to the owner target changes 64 borg-core/src files: +20,750 / -2,616 lines.
The six R2/R3 primary modules account for +8,613 / -423; R4 primary modules
also changed and their caller, state and restoration contracts must be audited.
These are physical source differences, not independent policy or effort counts.
Direct R4-to-target counts additionally contain the absence of R's shadow seams
and must not be interpreted as all-new main policy.

The new native modules are attack_preference, borg_cursed, borg_fast_fragile,
borg_kit, borg_lightless, borg_town_hunt and scary_list. Each called dependency
must be ported or explicitly classified as shared behind a named boundary.
Neither a function-name watcher nor an opaque native result establishes owned
policy. Parameters, trait/state layouts and changed existing helpers need the
same audit as the six primary modules.

The selected main uses a newer engine/runtime donor than R4. Exact pins and
shipped harness settings hashes are recorded in the preflight JSON. R4's old
engine/build/parity receipts remain historical; they do not establish parity
on the receiving engine or shipped profile. A read-only three-way merge preview
against the historical common base found 35 conflicted paths, including native
policy/shadow seams, parameters, provenance, CI, tooling and archive indexes.
The preview changed no working-tree or runtime bytes and is not a receiving
merge or validation result.

## Carry sequence

1. Preserve accepted R4 source, evidence and both immutable review snapshots.
   Record the owner acceptance separately from pending review-baseline CI or
   integration. Update this track's local milestone plan before touching policy.
2. Pin and audit the named main tree, its shipped harness/profile/class files,
   native donor, engine, recorder, parameter registry and tools. Reconcile source
   integration without dropping newer main work or R's typed/shadow services.
   Changes are from this source repository's accepted main, not an arbitrary
   upstream refresh. Resolve conflicts explicitly and preserve both histories.
3. Establish the exact receiving native baseline under shipped settings. Preserve
   existing reference bytes; qualify any new reference or build epoch explicitly.
   No default override, old pristine configuration or profile from track-p may
   silently replace the selected shipped inputs.
4. Carry changed R2/R3 policy, and reconcile every affected already-owned R4
   function, trait/parameter projection, caller and dependency contract. Preserve
   native order, widths, side effects, state/output and charge/quantity restoration.
   Keep historical seam/parameter receipts immutable; new source anchors need
   their own explicit receiving receipts rather than reblessing old baselines.
5. Exercise literal fault/re-port and positive/missing-input/nested/effect-once
   controls as affected. Run check and switch gates against the selected native
   baseline with zero mismatches and exact required streams/outcomes. Separate
   natural and fixture-enabled run coverage. Preserve failures and all natural
   deaths with qualified canonical custody. Follow the selected integration's
   rules on Zoo comparisons; do not dispatch population experiments implicitly.
6. Report carried scope/cost, unresolved shared policy and backlog beyond the pin;
   reforecast R5 from measured carry work. Green CI and owner task review precede
   merges into feature/track-r. Finish R4.5 with its own immutable owner-selected
   milestone review; do not start R5 or promote R to main/global reviewed.

## Preflight state

Preflight, production-surface inventory and isolated receiving control setup
are complete. The exact main control builds and its native smoke passes 3/3
under shipped settings. The first smoke invocation's postprocessing failure
and successful repeat are retained separately (R856-HANDOVER.md). No receiving
merge, owned-policy carry or R4.5 shadow parity claim is made. The carry branch is
`task/T856-r45-main-carry`, based on accepted R4. Its PR stays draft until the
implementation and required verification are complete. Owner R4 acceptance
permits this catch-up work; it does not accept the unfinished R4.5 result.

R45-RECEIVING-SURFACE.json pins complete production definitions in all 14
primary native modules and the seven new modules, retaining production after
interleaved test modules. Eleven existing native entry signatures differ or
are retired. The inventory is not a dependency/ownership classification or
semantic parity proof. Its generator fails closed on unsupported test cfg
conditions instead of silently truncating receiving policy.

The [receiving rebuild handover](handover/2026-11-25-856-r45-rebuild.md)
records the recovered local branch, independent policy implementation,
source receipt and runtime results. The [carry report](R45-CARRY-REPORT.md)
gives the per-file counts, shared boundaries, and Zoo handoff. These
supersede the preflight status above.
