# M13 town/trip donor integration (#328)

Donor tag `4.2.6-rs-town-trip-m13-v1` peels to
`1443c2d1b97d110ffbf5ae420d6c6b97ac15f7ea` (verified before refresh).
The previous core was `e6d13fc9baff0ef7f06de6333fbda436c961e0bf`.
The receiving task starts at main `b58e7b44`; it does not promote Track R
or additional Track P work. The three-way vendor tool preserves OURS files
and audits every other deviation via HOOKED. Existing Zoo pins are unchanged.

## Conflict decisions

The refresh reported nine files:

- `Cargo.toml`: keep the receiving rlib/package name, serde dependencies and
  vendored Clippy policy. Keep donor test features. The receiver links the
  recorder through its own staticlib, so do not add a second recording feature
  or dependency to borg-core.
- `lib.rs`: preserve shell, parameters, core trace and the receiving entry
  point. Retain all new town/trip modules; omit the donor recorder re-export.
- `borg_caution.rs`: retain named healing estimates, changing their defaults
  to M13's fixed potion healing: 0 percent and 20/40/60 HP. Other thresholds
  remain named. Remove retired fleeing-mode fields.
- `borg_danger.rs`: retain named fear weights; use M13's BI_PSTUN checks for
  gravity and ice instead of the old sound-resistance checks.
- `borg_fight_attack.rs`: retain the named sleeping-monster danger divisor;
  remove the retired munchkin exception.
- `borg_fight_defend.rs`: retain the middle-danger fail adjustment; remove
  M13's deleted, unreachable low-danger branch and its three parameters.
- `borg_power.rs`: preserve named limits/weights while removing mode gates.
  Retire the old munchkin cure-food level threshold.
- `borg_recover.rs`: preserve the food threshold; remove retired-mode rest
  branches and their obsolete parameter.
- `borg_think_dungeon.rs`: remove retired modes; preserve trace choice labels
  while taking the new nonnegative shop-index guards and town/trip wiring.

Five retired parameter names now fail as unknown rather than silently doing
nothing: `recover_munchkin_rest_min_food`,
`power_inventory_maxclevel_threshold_2`, `defend_shield_low_danger_numerator`,
`defend_shield_low_danger_divisor`, `defend_shield_low_fail_adjustment`.
Historical profiles/results retain their original meaning. Source citations
for unchanged parameters retain their documented historical origin; the six
changed healing defaults cite the M13 donor below in the registry.

## Native and binding adaptations

The receiving engine remains pristine upstream `42cd156ac`. Patch
`08-town-queries.patch` adds the donor's exact read-only `store_sale_quote`,
`inven_merge_num` and `inven_quiver_stack_room` implementations/declarations
under BORG_HARNESS. They neither perform a transaction nor mutate objects.
The guards-off engine check verifies that the upstream files are unchanged.

The receiver keeps its standard store UI, without donor `store_buy_direct`.
Its native player_options has 46 boolean options, followed by four bytes;
blindly taking the donor's 47-option struct would move those fields and
potentially later player fields. Keep the receiver's exact option_h layout
as an audited HOOKED file. engine::player_opt returns false for the donor-only
option before touching player memory. All existing option indices are stable;
M13's store buyer therefore sends the standard purchase prefix here. A Rust
regression checks native size/offset and calls that adapter without an engine.
The receiver's own generated bindings and recorder hooks are unchanged.

The donor build.rs recorder integrity check uses the receiving checker path;
it is declared HOOKED. The recorder release and cemetery-tool pin are unchanged.

## Configuration and serialization

Both `borg_town_plan` and `borg_trip_policy` remain zero. The receiving
harness config is refreshed from the donor config, retaining the reborn core,
errand and fear settings and disabling self-scumming. Retired mode keys are
removed from the delivered config; donor parsing still rejects explicit mode
activation and accepts inert historical settings with diagnostics. Errands
remain independent of those retired modes.

Town traits, schema11 producer/replay mappings, Confuse Monster wand support,
unsupported confusion staff/stun devices, selling-pressure backstops and
incremental-return observations are taken intact. Unknown threshold zero
still disables learning returns; pack slack zero still waits for a full pack.
The donor's tests cover these contracts; bounded receiving runs test wiring.

This is a default-off integration, with no survival or performance claim.
Fresh exact-tag donor comparisons replace v12 references for this acceptance.
Issue #293 remains separate: a pin refresh alone does not establish that its
reported shopping behavior is fixed.
