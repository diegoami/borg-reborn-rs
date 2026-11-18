# R528: accepted P4 inventory contract carry

Owner accepted #477. Its current synchronized head
`ac293ef700e0704ae3971cac161165d12054f330` passed all Rust/engine/recorder
and security checks and was promoted to main through #477 at
`435b07e7583581e91993bafda425d18378c91d5b`.
The earlier clean re-review at `7078bb1d2eaf5ac24f4396b1ef14ebbbf45f72be` remains historical; #524 synchronized
accepted main before this promotion. No new P milestone is selected.

This separate carry starts at Track R `ba460186` and targets only
`feature/track-r`, before the owned power implementation #434/#478.
The receiving runtime donor remains M13 `1443c2d1`; this is a neutral local
parameter-contract carry, not a re-vendor or adoption of unrelated main
combat, town, purchasing-quantity or shipped-default changes. Native state
derives and all R shadow facades remain intact.

The six native inventory files receive exactly the P4 delta from accepted
main base `a8ac685a` to P4 `ac293ef7`. The [carry inventory](R528-CARRY.json) preserves
all 1,729 receiving specs and IDs and appends the exact 362 P4 specs, for 2,091.
Accepted P4 has 2,191 because its main prefix is different. The ID mapping
is explicit; never apply numeric IDs from another specimen's registry.
The copied M13 parameter contract is byte-identical to accepted P4 and its
source/line citations describe that source, not a new R validation campaign.

`scripts/check-p4-carry.py` works offline: it protects the old registry,
all new definitions/order, accepted contract bytes and the ID map. The
non-spec remainder of `params.rs` is fingerprinted too, protecting parsing,
ordering validation and its bounds regressions. Reversing
the exact accepted patch in a temporary copy must reproduce each receiving
baseline module byte for byte. No external checkout, Git fetch or metadata
rewrite runs during this check. Five carry tests cover the exact import and reject extra policy edits,
existing-default changes, altered contract bytes, incorrect ID mappings and
changes to parameter validation or its bounds regression.

The carry also adopts the already accepted Cemetery pin `09225018` and its
native-byte attestation from main, retaining R's pre-dispatch shadow replay
setting extension. This implements the owner's current same-start/key-stream
reuse rule across builds/settings; it is not a new locally designed matcher.
The owner implementation still rejects missing local exports, different
seeds/races/classes/saves/keys and unknown starts; exact-family mode remains.
New tests exercise those cases and retain R's replay-setting regression.
No historical packet, manifest or immutable evidence is rewritten.

Local checks pass: 339 core tests (four existing ignores), workspace tests,
clippy, formatting, 52 source tests and 116 tooling tests. The source inventory
and exact donor hooks pass. The clean native build and real shopping-budget
override test pass. All three receiving smoke modes match the retained R435
baseline, and each constructed mode passes 55 expected-value checks. Full
off and check modes each match all 45 baseline games across keys, messages,
state, core traces and endpoints. Full switch mode is still running; its
strict gate, final canonical reconciliation and new-head CI remain required.
Repeated modes and reused baselines are not independent trials.
Retain every original run, build identity, failure and raw death packet.

RULES 1a is synchronized while retaining the owner's Track R branch exception.
Every new carry/runtime PR waits for the owner to say its review is done.
Acceptance of #477 does not approve this separate carry PR. Do not merge it
or claim full #434 completion before its own review and green validation.

The three added donor blobs were independently checked against the exact
M13 donor with `borg-core-vendor.py check`, in addition to the offline
recorded-byte gate. Runtime inputs remain unchanged by these review fixes.
