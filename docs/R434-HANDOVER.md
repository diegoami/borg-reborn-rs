# R434: owned power implementation

The owner authorized #529's merge into `feature/track-r` at
`99fa7cc5dafd301a6d74c2f5d18817c2a1998945`. Its accepted P4 carry is the
receiving baseline for this task. P4 promotion #477 and R carry #528 are
complete; their acceptance does not accept this new runtime port.

Work continues on `task/T434-power-port`, draft #478, targeting only
`feature/track-r`. The initial boundary design remains in
[R434-BOUNDARY.md](R434-BOUNDARY.md); its pre-carry inventory is historical.
The refreshed JSON pins the receiving sources after the carry.

`borg-next/src/stateful/power.rs` owns all 11 production bodies: equipment,
inventory, activation valuation, total-power ordering and arithmetic helpers.
Every BI read uses a named borrowed projection, including the three indexed
stat families. Spell entry lookup remains a taped memory query and the spell
view borrows current memory. Engine facts, item weight, random ego power and
spell-stat lookup use the closed API at their original call sites.

Equipment has a separate immutable `ReadDecision::PowerEquipment` entry;
total power uses mutable `Decision::Power`. Public native signatures are
preserved. Both private native entries and owned entries mark their policy
visits. `borg_power_dynamic` is an explicitly shared configured algorithm,
with complete state/result replay; no independent formula ownership is claimed.
Preparation remains a real mutable service; its buffer and other state
changes persist exactly. Full-state equality is enforced by the existing tape.

Shadow schema @6 adds power entries, its counter and the immutable equipment
root. `--r4-power` requires that exact surface; older @5 item-use evidence
still has its explicit reader. Historical evidence is never rewritten.

`audit-power-port.py` verifies all 11 bodies, constants and quoted literals
after declared API/Read/marker seams. Mutation tests reject arithmetic,
parameter and return drift. The exact native facade/dependency patch is
recorded separately as R434-NATIVE-SEAMS.patch; the P4 guard reverses it
before proving the unchanged accepted P4 delta. This does not permit arbitrary
native policy edits or rebaseline the accepted registry/contract.

The test-only `test-r4-power` feature includes existing item-use controls and
12 new synthetic cases: activation first-match, signed swap contributions and
wrapping, first fuel reward, preparation buffer persistence, explicit shared
dynamic formulas, nested effect-once replay, a rejected return fault and repair.
These use copied memory and real engine facts. They do not consume items or
count as natural games. The return-fault control is not an actual source-literal
mutation run; that runtime witness remains required separately.

Local checks pass: 54 source tests, 117 tools tests, workspace tests, 339 core
tests (four existing ignores), clippy with the power feature and formatting. Clean
native and test-feature builds, constructed execution, literal fault/repaired
runtime control, normal nested callbacks, off/check/switch smoke and full 45,
strict entry gates and canonical death reconciliation remain required. Do not
mark #434 complete, merge #478, or claim R4 accepted from compilation alone.
Keep all original attempts and actual source/build identities. Gameplay evidence
belongs in borg-cemetery; this source handover records implementation and limits.
