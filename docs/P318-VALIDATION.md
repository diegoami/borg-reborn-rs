# Qualified historical dossier imports (#318)

Native exports and canonical registry allocation remain owned by
borg-cemetery. The local importer packages completed validation evidence
with deterministic full SHA-256 IDs. It preserves the original record
byte for byte, changes only the canonical record ID, and retains every
payload byte. Receipts make the transformation auditable.

All 2,455 cases from the accepted #307 head are frozen in the legacy
inventory without rewriting their records or paths. Their records and
13,705 declared payload files verify. The ten new #317 cases become
qualified packages; P318-DOSSIER-MAP.json records every original path.
All 2,465 retained cases pass the new CI gate. Archive independence and
attempt counts remain 2,462 and 2,465 respectively; all other aggregate
index metadata is unchanged.

Nine new importer tests pass, including six concurrent subprocess imports,
idempotent re-import, duplicate encounter/result provenance, damaged data,
legacy mutation/missing cases, source symlinks and canonical-root refusal.
All 87 tool tests and 19 script tests pass.

As a real collision check, isolated copies of P D2446-rust-seed1011 at
`1710276038e78c01d29905beb177cc28d3870c54` and R D2446-rust-seed1084 at
`cee4c94d` import together with distinct qualified IDs. Original records,
identities, attempts and all payload bytes remain intact. No R case is
added to this branch and no external repository is written.

The ten #317 deaths reconcile against the qualified archive, using the
actual `/workspace/work/p317/build/rust/BUILD_ID`. An initial invocation
from this tools-only worktree omitted that build ID; its standalone
original-core fixture had no parent manifest, so reconciliation reported
one missing source identity. Supplying the actual build ID passes with
zero missing or unrecorded deaths. No record or assertion was changed.

Game/build inputs remain exactly #317 source SHA-256
`69c447072d3822bef7dc9f9998cadfc113452ff406858f98c81bd0f44a567726`.
Its full-game, native and Rust evidence applies (P317-VALIDATION.md);
this tools/workflow change does not require a new measured build.
Required task CI still gates track-p integration, followed by owner
milestone review before promotion to main.
