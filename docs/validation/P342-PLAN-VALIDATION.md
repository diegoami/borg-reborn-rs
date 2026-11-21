# Errands-plan/main synchronization (#342, second sync)

Main advanced through accepted #308 to
`820140e0409b34f0ee53cabfbac2b57a39f62e7e` while the preceding M13 sync
was in CI. Preserve that implementation and all ten qualified evidence
packages. Existing P and M13 evidence remains unchanged. Generated indexes
are rebuilt from both lineages and both handover histories are retained.

As required by PR308-REVIEW-FIXES.md, remove the duplicate/lossy plan-setting
readers and obsolete legacy-reader comment. All settings share the strict
P reader. A crate-private Result helper supplies the existing pure path
checks; the production entry point keeps fatal file-error diagnostics.

Core selection keeps environment priority lazy. An explicit original-core
environment override does not read borg.txt, including a file plan; an
explicit environment plan on that core still fails. An original core
selected through the file rejects a file plan. Vendored plan initialization
retains its strict file-selection contract. Native tests cover all three
isolation paths, both malformed-file routes, plan startup/transitions and
the parameter loader after a valid explicit plan selection. New plan file
errors can be diagnosed before parameter loading; tests distinguish these
paths rather than requiring the former diagnostic prefix in every route.

Measured revision: `0a0c23a1d52ad9da4923844b7fd899a83241e8f0`.
Source SHA-256:
`87717140e88805c99b2b8f5d036036cf21eb9b8e953b8cce25d07465a2c2a9a9`.
Binary SHA-256:
`0bd996e1cc05a16873b49760c171afc3c5b147305cd1307244eb09828cd3cce2`.
BUILD_ID records git_dirty=yes and inputs_dirty=no: native test edits and
a local target symlink were outside the game inputs. Retain that actual
identity; do not relabel it as a clean later revision. Removing the local
symlink and committing test/evidence files preserves the same source hash.

Required Rust checks pass: formatting, Clippy, 156 + 4 + 5 original tests,
and 328 core tests (four ignored). Vendor verification passes. All 68 engine
tests pass with one local binding-tool skip, including both native config
suites; pinned zoo and upstream checks are required. Tools (87) and scripts
(19) pass. No upstream game, recorder or donor-pin change is introduced.

All 45 default vendored-core play-log hash triples and deterministic
summaries match accepted M13 receiver-full evidence. Wall time is excluded.
The trace has 242,641 contiguous classified rows. Original smoke matches all
three references and its seed 1020 death control completes. Three M13
policy-on class controls reconstruct the accepted config hashes and match
all play logs. Existing plan growth/phase/no-match/malformed-input checks
pass. No new GUI drive, plan reload, historical replay corpus or population
experiment is claimed.

All 14 new measured natural deaths are retained with original source/build
identity and qualified paths in P342-PLAN-DOSSIER-MAP.json. Reconciliation
finds none missing or unrecorded. All 2,547 archive cases verify. Incoming
#308 packets and the accepted legacy inventory remain unchanged; all six
original POC imports were independently checked against 0030b101 byte for
byte. #335 was already resolved by the accepted #308 integration.

The old #331 independent review and the accepted #308 review remain tied
to their named heads. This integration implements #308's explicit shared-
reader reconciliation requirement; combined validation and required CI
support the new exact promotion pin. #333 remains a separate draft and
must not enter the owner-authorized #331 promotion.
