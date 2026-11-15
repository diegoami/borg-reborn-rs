# Dossier identity and publication IDs

Owner decision, 2026-11-15 (#338): borg-cemetery assigns final numeric IDs
centrally when it accepts records. IDs in isolated branch/CI registries are
provisional display labels, not globally unique identities. No per-track
numeric range is reserved by a session.

For cross-branch references, qualify the local ID with the repository and an
exact registry snapshot commit containing that record, for example:
`diegoami/borg-reborn-rs@5cebb4c1aeb96bcade94f3001429ebeb1945131a:docs/deaths/D2449-rust-seed1011`.
A mutable branch name, or the game's build commit before the record existed,
is not an adequate registry qualifier. References inside one repository
snapshot are relative to that snapshot. When copying a report elsewhere,
retain its original registry namespace or an explicit mapping.

Before records are committed, use the CI run/registry namespace plus source
commit and provisional ID, e.g. a workflow run's core-deaths versus deaths
artifact. Two isolated registries at the same source commit may both start
at D0001. Record the exact containing registry commit when one becomes
available; do not introduce a circular self-commit field into a record.

The existing encounter_family and attempt result_sha256 provenance keys
support deduplication across branches and repeated exports. A new source
identity or display ID does not make a repeated game an independent sample.
Canonical publication must deduplicate first, serialize final allocation,
and return the canonical ID and provisional-to-canonical mapping. A local
export into a cemetery checkout is not proof that publication was accepted.

Preserve already published record bytes, original IDs, file hashes and
review references. Import/promotion must detect numeric collisions even
when folder suffixes differ; retain origin repository/registry commit/ID
and explicit mappings for any remap. Never mass-renumber historical data
or silently rewrite evidence to hide a collision.

`python3 scripts/check-death-ids.py docs/deaths` is this repository's consumer
archive check, run in CI. It rejects duplicate numeric IDs and numeric
folder/record disagreement; it does not allocate IDs, publish, deduplicate
statistical samples or mutate records. The existing native toolkit remains
cemetery-owned. Canonical publication protocol implementation/confirmation
is a separate owner handoff; this policy does not claim it is already changed.
