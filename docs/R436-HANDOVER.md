# R436: wear and identification ownership

Power #434 is owner-accepted and merged through #478 into feature/track-r.
This task starts from that accepted feature baseline. The branch is
`task/T436-wear-maintenance`; every task PR targets feature/track-r.
The source contract and receiving hashes are in [R436-SOURCE.json](R436-SOURCE.json).

Independent scope is 37 production bodies: wear decisions/helpers,
identification/inscription and note-needs-ID, plus the small inventory
worth-ID helper. That helper is owned rather than treating its native policy
result as an opaque dependency. Wear metadata allocation/freeing, ammunition
classification, name/activation tables and slot classification remain shared.

Nine public mutable entries cover test/identify, ring reordering, ring wear,
backup swap, general wear, best equipment, clearing the best plan, recharge
and inscription. The clear-plan entry returns Unit; the other entries retain
the native boolean, which is not universally evidence of a queued action.
Every BI read uses a named projection. Native facades and private driving
implementations retain signatures, complete caller state and command order.

Notice/recomputation, flow/rest, danger, power and item use are named taped
services. Trials preserve the native order of safe-item copies, quantity
changes, repeated notice and restoration. Empty holes retain the native
residual bytes with quantity zero; trial-generated traits can persist until
the caller's recomputation. Do not invent a full rollback. Nonbinary critical
resists, first-match/tie rules, anti-bounce and wrapping arithmetic are retained.

Adjacent decurse/enchant still contain policy failure allowances and stack/
target limits without parameter coverage. They remain explicitly shared;
P dependency #680 requests classification and an accepted contract before
independent ownership. Their execution is not proof of an owned maintenance
chooser. No threshold extraction, tuning or new policy is done on this branch.

`audit-equipment-port.py` verifies all owned bodies, constants, string/character
literals and the exact native seam fold-back. The P4 guard independently
reverses only the wear portion of this patch before its previous accepted
carry proof. Native inventory, ID and shared note changes also fold back to
this receiving baseline, with no unrelated native edits.

Shadow protocol @7 adds the equipment entries and counter. The strict reader
requires every equipment entry and each immutable root from the older power/
item-use surface. Historical @6 and older readers stay explicit and unchanged.

Constructed controls use copied memory and queued keys. Only engine UI
callbacks of the shared note service are stubbed; its real anti-loop state
logic is shared between normal and constructed paths. Controls include
positive ring wear/removal, inscription and recharge; exact commands and note-state
changes; empty/early returns; nonbinary resistance loss; tie ordering; live
note origins; slot/quantity restoration; a real nested public wear callback,
a direct real-Borg-RNG draw/replay-once check, and a rejecting return fault
followed by repair. The RNG sample calls engine Rand_div while generate_keypress has installed
the Borg quick RNG (borg.rs); the fixture runs from borg_think::after_think
before generate_keypress restores the saved game RNG. A fixture assertion
requires quick mode. It advances the Borg stream in this test-only context
and is not an independent natural game. Simulations and queued keys
are not engine consumption or independent natural games.

At the original reviewed head, source/Rust checks, native builds, constructed
controls and exact-head CI passed. An actual literal threshold mutant was
rejected by both source audit and the native runtime control; restored-source
controls passed in check and switch. All smoke modes and full native/off
matched the accepted power baseline. The original full shadow campaign is
still running; its results belong to that original source.

Review corrections record first-empty-pack, home-full and inventory-full as
read-only queries, with shared immutable signatures rather than unnecessary
post-state copies. Fresh builds/controls and exact-head CI are required for
these corrections; fresh full shadow parity, strict coverage and canonical
reconciliation still gate completion. Owner review completion gates merge.

Natural smoke did not reach borg_object_fully_id: it is a user-command entry,
and constructed inscription cases supply its strict-entry coverage. Empty,
hungry and outside-home cases exercise early returns only. Positive ring
wear/removal, inscription, recharge and plan-clear controls do not establish
positive at-home borg_best_stuff_aux/borg_one_step_wearing_best coverage;
call counts alone cannot distinguish those branches. This remains an explicit
coverage limit. With test-r4-equipment enabled, item-use controls also execute
the real shared note anti-loop effects; only note UI callbacks are stubbed.
Retain every failed attempt and original build identity. New gameplay evidence
belongs in borg-cemetery; do not publish private runtime metadata to source PRs
without specific authorization. Source code and concise status stay here.

#437 follows the accepted wear task. The combined #438 checkpoint freezes
an exact review snapshot and stops for owner acceptance. R4 remains a feature
branch; neither task completion nor CI promotes it to main or reviewed.
