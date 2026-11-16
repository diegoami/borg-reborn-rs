# Borrowed named reads for stateful policy (#363)

The R2 projection eagerly materialized every trait group at each stateful
read. M13 carries 373 constructor sites. This change supplies `Read`, a
borrow of the same native BorgState, with named groups and i32 scalar
methods. The existing trait declaration generates snapshot fields, lazy
methods and source-index documentation together. No cache, engine access,
interior mutability or lifetime extension is added.

`Read::new(st).character().curhp()` reads only BI_CURHP when that method is
called. Constructing Read or a group wraps one pointer and does not traverse
trait storage. Danger retains its one-assessment immutable View. Existing
native temporaries remain explicit temporaries; mutable dependencies end a
borrow before the following expression reconstructs one.

R363-SOURCE-AUDIT.json records byte hashes and a projection-only code-token
comparison against the M13 candidate. Policy expression order, branches,
requests, mutations and the named mana adapter remain unchanged. The source
gate now admits only Read for stateful policy, retaining its BI/native/engine
bypass restrictions. Tests cover all scalar mappings, successive-borrow
freshness, construction with empty storage, one-pointer representation and
compiler rejection of a retained borrow across a mutable write.

Local acceptance passes on runtime input
`3734726bb0f4dd8d3cf5717b3b86768df4debac1d9b0567403877b4a48225829`,
built at `9dc8ff9cf7ff48b8806e130130c0aaeb6f80951f`, both dirty flags false.
Off/check/switch each match all 45 exact-tag M13 donor games. Check/switch
have zero mismatches across 261,953,840 comparisons each, with all 11 public
entries covered. The strict rare-teleport fixture/flag/cap stay unchanged.
See R363-COMPARISON.json. The fresh donor from #343 is reused unchanged.

Native smoke is 3/3 equal across modes; same-receiver full diffs pass 45/45.
Strict cross-repository complete-summary equality retains the known telemetry
and schema differences; actual streams, effective configs and common gameplay
fields are checked independently without weakening the tool. Original smoke
matches its reference. The exact accessor build has four real native nested
callbacks per mode, equal play streams and zero mismatches. The engine suite
passes 68 checks with two optional local skips; its pinned profile contract
is required/enabled. CI supplies binding-generation/upstream-check capabilities.

All 36 full-run deaths reconcile to 12 repeated encounter/result identities;
original smoke/death regressions add two. The 14 new qualified packages and
all 2,683 archive packages verify. R363-DOSSIER-MAP.json explicitly scopes
provisional source IDs by content-hashed local artifact inventories. A build
commit is not a containing registry commit; the parent retained registry is
pinned separately. Final numeric allocation remains central at borg-cemetery.
Package and replay counts are not independent trials.

177 workspace tests plus a compile-fail doctest pass; direct compiler control
also gives E0502. Core: 335 pass, four upstream ignored. Source-tool: 34 pass.
Feature-enabled Clippy, vendor and archive checks pass. The source audit also
compares full normalized text and preserves quoted literals exactly.
No speed/survival gain is claimed. Final CI and feature-only merge remain;
then attack #364, defence #365 and combined owner-selected review #366.
