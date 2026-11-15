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

Runtime acceptance is pending the predecessor #343 baseline, a separate
exact accessor build, donor/off/check/switch comparisons and strict entry
coverage in both modes. Every natural death needs a qualified retained
package and reconciliation. Historical M13/R2 results are predecessor
measurements, not accessor proof. No speed or survival improvement is claimed.
