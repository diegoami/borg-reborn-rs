# Power ownership boundary before the P4 carry (#434)

This is source design, not a runtime implementation or validation report.
Receiving source is Track R `ba460186678ee07f6a3aad7053e27cf1e5b2c2b9`;
the runtime input remains
`95fb364b150ad87a8330b45441d08f252096bedc0c766754e5eb3608b3e30293`.
The companion [inventory](R434-BOUNDARY.json) pins the files inspected.

## Dependency state

P task #433 closed after integration into `track-p` through #476 at
`105f79479df4631d7219eaa2d77c96a76e892238`. Its closure explicitly does
not accept P4 or authorize the R carry. Promotion/review #477 is open,
targets `main`, and requires owner-selected review and acceptance. Do not
merge it from an R session. After promotion, carry the exact accepted
contract separately into `feature/track-r`, preserving R facades and hooks.
Only then implement the complete power policy. Re-audit the receiving bytes,
named parameters and baseline after that carry; this pre-carry inventory
does not certify the future source or the 362 new parameter defaults.

## Owned entries and values

| Receiving entry | Proposed request | Comparison contract |
| --- | --- | --- |
| `borg_power_equipment(&BorgState) -> i32` (currently crate-visible) | `ReadDecision::PowerEquipment` returning `Value::Score` | Same immutable input, ordered query replay, return/order/request equality; no mutable state or engine-root capability |
| `borg_power(&mut BorgState) -> i32` | `Decision::Power` returning `Value::Score` | Independent full-state fork/replay, ordered dependencies, return and complete final-state equality |

These request names are proposals, not registered entries. The owned module
must own equipment/inventory/activation valuation and the arithmetic helpers.
The current file has 11 production bodies and three test bodies; the lexical
14-definition count includes tests. No native equipment/inventory/total-power
result may become an opaque dependency of its own purported port.

The immutable entry belongs in `read_decide`; the mutable entry belongs in
`decide`. Preserve the existing public native facade, shadow callback and
native-control path. Do not make a readonly wrapper accept `&mut BorgState`
just to reuse the mutable dispatcher. Never hold a borrowed `Read`, item or
spell across a mutable service. Obtain fresh named reads after effects.

## Dependencies, including deliberately shared policy

There are seven direct engine fact calls in the inspected file. `z_info`
already has a closed API adapter; pack size and activation count remain
engine facts. The other six need audited adapters for strength hold,
dungeon-book classification, class spell weight/count, player flags and
spell failure chance. Preserve argument types and return widths, including
the native `i16` spell chance. Request facts at the original call sites;
an eager snapshot would reorder queries and can admit unused requests.

Item weight, ego random-power classification, spell-stat selection and
spell-entry lookup also need named boundaries. Item weight is a pure
quantity-times-weight calculation truncated to `i16`; preserve that width.
Spell lookup returns an optional index into current memory: a named borrowed
spell view must retain unavailable results and actual level/power fields.
Activation-ID initialization remains shared engine-derived metadata, not an
independent activation chooser. All184 lexical BI names in the receiving
power source already have named Read-macro mappings; the missing-name list
is empty. Reuse those accessors. This does not establish non-trait item, spell,
configuration or swap-field coverage, nor is it a semantic call graph.

`borg_power_dynamic(&mut BorgState)` is a configured shared policy algorithm,
not an engine fact. If retained, declare it as an ordered mutable dependency
with complete state and result replay. Passing through it cannot establish
independently owned dynamic-formula valuation. `borg_prepared_c` already has
a taped mutable adapter; its reason and complete state changes must remain
observable. Neither service may be replaced by a cached scalar total.

## Mutation and arithmetic witnesses

`borg_power` calls equipment before inventory on the static path; the dynamic
path replaces those two calls. It then tests preparation depths in order,
stopping on the first reason. Preparation can write `prepared.buffer`; do
not restore it merely because the operation looks like a read-only valuation.
Re-read the loop bound at the same points after mutable calls. Add the depth
bonus and then weapon/armour swap contributions in the original order.
The complete-state comparator must cover all native side effects, not just
the known buffer or the selected item. Outer wear/store trials own their
restoration; a power facade must not invent extra rollback.

Preserve wrapping `i32` integer operations, `i64` long intermediates with
truncation back to `i32`, product/division order, count-loop `k` carry and
activation-chain first-match precedence. Do not clamp, saturate, reorder
accumulation or repair a native zero denominator within this migration.

Later validation needs positive valuations, activation collisions, count
limits, overflow/truncation/division ordering, static/dynamic selection,
preparation failure/buffer behavior and swap contributions. Include actual
nested item simulations and effect-once/tape restoration. Synthetic state
controls must be labelled and cannot stand in for normal engine execution.
Require a rejecting literal fault and repaired re-port, exact-mode full45
parity at the declared post-carry baseline, and canonical death custody.

## What remains before completion

Owner acceptance and P4 promotion, a separate exact carry, a refreshed source
and API audit, implementation, explicit protocol/entry registration, source
and literal correspondence, constructed and real nested controls, full
native/check/switch validation, canonical receipts and exact-head green CI
are all still required. #434 stays open. #293 shopping replacement and the
final immutable R4 review #438 remain separate. No main promotion, new
gameplay policy, parameter extraction or population experiment occurs here.
