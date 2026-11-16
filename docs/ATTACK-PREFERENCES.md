# Class/family attack preferences (#403)

Owner-authorized implementation of the proposal in #396. This is Track P native
policy only, on a task branch from `track-p` at cd411e55. The alternate review's
half-up rounding, mechanism-independent control family and emergency bonus cap
are not adopted. Invocation determines family, including weapon spells and
controls; floor rounding and existing emergency eligibility are retained.

## Contract

`attack_preference_<class>_<family>_permille` has default 1000, inclusive bounds
500..2000, unit `utility_permille`. Nine classes × six families = 54 parameters.
Classes are engine class.txt order at 42cd156ac: Warrior, Mage, Druid, Priest,
Necromancer, Paladin, Rogue, Ranger, Blackguard. Wait is fixed at neutral; unknown
class IDs use neutral and are exposed as a numeric ID with null class name.
Unknown attack IDs are explicitly rejected. BF_MAX is not an action.

For a known attack whose helper returns positive raw utility, use
`clamp(i64(raw) * weight / 1000, 1, i32::MAX)`. Positive integer division floors.
Nonpositive raw values (including negative sentinels) are unchanged and remain
unselectable. The widened product cannot overflow for i32 utility and validated
weights. The positive floor preserves tiny positive candidates; high results
saturate. Strict greater-than preserves the first candidate in a tie. At 1000
the original score is returned unchanged, including negative sentinels.

The multiplier scales the complete returned utility, including existing danger
bonuses and cost penalties. Hit/damage computations, danger horizons/modes,
resource costs and availability checks stay in the original helpers. This is a
preference, not a damage estimate, a danger correction or a proven survival gain.

The only policy hook is in borg_attack after each original simulated helper
call and before comparing it with the incumbent. Execution calls the original
helper once as before, without preference. No preference reaches the helpers'
internal target, ammo, throwable or multi-effect searches. Candidate order,
RNG calls, simulation restoration and queued execution remain native.

Emergency reserve Magic Missile gets the same spell preference as normal Magic
Missile, only after its own existing checks. It is an insufficient-mana emergency
path, separate from ordinary reserve-mana cost penalties. Wand of Wonder's
intentional positive emergency utility remains utility, not a rejection sentinel.
Useful zero-direct-damage controls keep their invocation family and may compete.
The final seam cannot infer an immunity/mana/charge reason from an aggregate
integer; it never reevaluates availability or invents a rejection reason.

## Configuration

These are experimental examples, not recommended defaults:

```ini
borg_param_attack_preference_mage_spell_permille = 1600
borg_param_attack_preference_mage_device_permille = 1250
borg_param_attack_preference_warrior_melee_permille = 1250
borg_param_attack_preference_ranger_projectile_permille = 1250
```

Precedence and validation remain those of the existing registry: environment
`BORG_PARAM_ATTACK_PREFERENCE_MAGE_SPELL_PERMILLE`, then borg.txt, then default.
Unknown/misspelled keys, duplicate file keys, nonintegers and out-of-range values
fail at initialization with exit 2. Values are immutable for the process.
Zoo profiles use the same integer `borg_param_*` settings; no adapter/parser change.
`params-core.json` includes names, values, defaults, bounds, units and origins;
the harness records its hash. Clear ambient overrides for profile-only runs.

For raw melee 25 and spell 17, spell 1500 yields 25 (first melee wins the tie);
1600 yields 27 (spell wins). These vectors come from the measured PlanMage
components but do not by themselves reconstruct that process's Borg memory.
Canonical evidence: [PlanMage analysis](https://github.com/diegoami/borg-cemetery/blob/main/deaths/recorded-planmage-20261115-wild-cat/magic-missile-analysis.md).

## Trace schema

New lines use `borg-reborn-rs/core-trace@2`; historical @1 evidence is unchanged.
All @1 fields remain. The additional `attacks` array holds each invocation of the
attack ranking loop in this think, including attempts with no positive winner.
Each invocation has `candidates` in original order and `winner` (BF ID or null).
Candidate fields are attack_id/name, family, class_id/name, eligibility,
raw_utility, parameter_name, effective_weight, parameter_origin, adjusted_utility
and incumbent_changed. Eligibility is `positive_return`, `nonpositive_return` or
`unknown_attack`, never an inferred cause. Null parameter and `fixed_neutral`
origin identify wait/unknown-class neutral behavior. No extra target/item reads.

Allocation is skipped with no active trace frame. Original simulation values are
captured without reexecution. Candidate vectors are local across effectful calls;
only the completed ranking is attached to the current think frame, before action
execution. Nested thinks use their own frame. Repeated selector attempts append
rather than overwrite. Trace data is never consumed by policy. Consumers retain
@1 support; new-output assertions use @2. R's shadow telemetry schema (#339) is a
separate contract, not silently renamed by this native trace expansion.

## Review and Track R dependency

Task PR targets track-p only. Required CI and native parity are task acceptance;
owner-selected pinned milestone review/acceptance still precedes promotion to
main. No R files or source pins change. R consumes the accepted promoted contract
through typed state/parameter access and independently implements classification,
arithmetic and ranking. Only spec/test vectors may be shared, never a native final
decision oracle. Coordinate #364/#366; if R3 freezes first, do an explicit later
carry against a new exact source pin. Validate neutral and chosen non-neutral
native/check/switch profiles with ordered effects/RNG, restoration and nesting.

Gameplay scenarios, tuning reports and natural-death evidence belong in
borg-cemetery. Source keeps tests and concise validation links; no zoo experiment
or survival claim is part of this implementation.

## Explicit BF inventory

This is the complete symbolic mapping compiled in attack_preference.rs. It is
checked against the current header declarations, including count, names and IDs.
The code uses no numeric ranges or runtime prefix classification. Without randarts,
the original loop remains `0..BF_ACT_STARLIGHT` (129 exclusive), unchanged.

| BF symbol | Family |
| --- | --- |
| `BF_REST` | wait |
| `BF_THRUST` | melee |
| `BF_OBJECT` | thrown |
| `BF_LAUNCH` | projectile |
| `BF_SPELL_MAGIC_MISSILE` | spell |
| `BF_SPELL_MAGIC_MISSILE_RESERVE` | spell |
| `BF_SPELL_STINK_CLOUD` | spell |
| `BF_SPELL_LIGHT_BEAM` | spell |
| `BF_SPELL_COLD_BOLT` | spell |
| `BF_SPELL_STONE_TO_MUD` | spell |
| `BF_SPELL_SLOW_MONSTER` | spell |
| `BF_SPELL_SLEEP_III` | spell |
| `BF_SPELL_FIRE_BALL` | spell |
| `BF_SPELL_SHOCK_WAVE` | spell |
| `BF_SPELL_EXPLOSION` | spell |
| `BF_SPELL_CONFUSE_MONSTER` | spell |
| `BF_SPELL_COLD_STORM` | spell |
| `BF_SPELL_METEOR_SWARM` | spell |
| `BF_SPELL_RIFT` | spell |
| `BF_SPELL_MANA_STORM` | spell |
| `BF_SPELL_BLIND_CREATURE` | spell |
| `BF_SPELL_TRANCE` | spell |
| `BF_PRAYER_HOLY_ORB_BALL` | spell |
| `BF_PRAYER_DISP_UNDEAD` | spell |
| `BF_PRAYER_DISP_EVIL` | spell |
| `BF_PRAYER_DISP_SPIRITS` | spell |
| `BF_PRAYER_HOLY_WORD` | spell |
| `BF_SPELL_ANNIHILATE` | spell |
| `BF_SPELL_ELECTRIC_ARC` | spell |
| `BF_SPELL_ACID_SPRAY` | spell |
| `BF_SPELL_MANA_BOLT` | spell |
| `BF_SPELL_THRUST_AWAY` | spell |
| `BF_SPELL_LIGHTNING_STRIKE` | spell |
| `BF_SPELL_EARTH_RISING` | spell |
| `BF_SPELL_VOLCANIC_ERUPTION` | spell |
| `BF_SPELL_RIVER_OF_LIGHTNING` | spell |
| `BF_SPELL_SPEAR_OF_OROME` | spell |
| `BF_SPELL_LIGHT_OF_VARDA` | spell |
| `BF_SPELL_NETHER_BOLT` | spell |
| `BF_SPELL_TAP_UNLIFE` | spell |
| `BF_SPELL_CRUSH` | spell |
| `BF_SPELL_SLEEP_EVIL` | spell |
| `BF_SPELL_DISENCHANT` | spell |
| `BF_SPELL_FRIGHTEN` | spell |
| `BF_SPELL_VAMPIRE_STRIKE` | spell |
| `BF_PRAYER_DISPEL_LIFE` | spell |
| `BF_SPELL_DARK_SPEAR` | spell |
| `BF_SPELL_UNLEASH_CHAOS` | spell |
| `BF_SPELL_STORM_OF_DARKNESS` | spell |
| `BF_SPELL_CURSE` | spell |
| `BF_SPELL_WHIRLWIND_ATTACK` | spell |
| `BF_SPELL_LEAP_INTO_BATTLE` | spell |
| `BF_SPELL_MAIM_FOE` | spell |
| `BF_SPELL_HOWL_OF_THE_DAMNED` | spell |
| `BF_ROD_ELEC_BOLT` | device |
| `BF_ROD_COLD_BOLT` | device |
| `BF_ROD_ACID_BOLT` | device |
| `BF_ROD_FIRE_BOLT` | device |
| `BF_ROD_LIGHT_BEAM` | device |
| `BF_ROD_DRAIN_LIFE` | device |
| `BF_ROD_ELEC_BALL` | device |
| `BF_ROD_COLD_BALL` | device |
| `BF_ROD_ACID_BALL` | device |
| `BF_ROD_FIRE_BALL` | device |
| `BF_ROD_SLOW_MONSTER` | device |
| `BF_ROD_SLEEP_MONSTER` | device |
| `BF_ROD_UNKNOWN` | device |
| `BF_STAFF_SLEEP_MONSTERS` | device |
| `BF_STAFF_SLOW_MONSTERS` | device |
| `BF_STAFF_DISPEL_EVIL` | device |
| `BF_STAFF_POWER` | device |
| `BF_STAFF_HOLINESS` | device |
| `BF_WAND_UNKNOWN` | device |
| `BF_WAND_MAGIC_MISSILE` | device |
| `BF_WAND_ELEC_BOLT` | device |
| `BF_WAND_COLD_BOLT` | device |
| `BF_WAND_ACID_BOLT` | device |
| `BF_WAND_FIRE_BOLT` | device |
| `BF_WAND_SLOW_MONSTER` | device |
| `BF_WAND_HOLD_MONSTER` | device |
| `BF_WAND_CONFUSE_MONSTER` | device |
| `BF_WAND_FEAR_MONSTER` | device |
| `BF_WAND_ANNIHILATION` | device |
| `BF_WAND_DRAIN_LIFE` | device |
| `BF_WAND_LIGHT_BEAM` | device |
| `BF_WAND_STINKING_CLOUD` | device |
| `BF_WAND_ELEC_BALL` | device |
| `BF_WAND_COLD_BALL` | device |
| `BF_WAND_ACID_BALL` | device |
| `BF_WAND_FIRE_BALL` | device |
| `BF_WAND_WONDER` | device |
| `BF_WAND_DRAGON_COLD` | device |
| `BF_WAND_DRAGON_FIRE` | device |
| `BF_RING_ACID` | activation |
| `BF_RING_FIRE` | activation |
| `BF_RING_ICE` | activation |
| `BF_RING_LIGHTNING` | activation |
| `BF_DRAGON_BLUE` | activation |
| `BF_DRAGON_WHITE` | activation |
| `BF_DRAGON_BLACK` | activation |
| `BF_DRAGON_GREEN` | activation |
| `BF_DRAGON_RED` | activation |
| `BF_DRAGON_MULTIHUED` | activation |
| `BF_DRAGON_GOLD` | activation |
| `BF_DRAGON_CHAOS` | activation |
| `BF_DRAGON_LAW` | activation |
| `BF_DRAGON_BALANCE` | activation |
| `BF_DRAGON_SHINING` | activation |
| `BF_DRAGON_POWER` | activation |
| `BF_ACT_FIRE_BOLT` | activation |
| `BF_ACT_FIRE_BALL72` | activation |
| `BF_ACT_FIRE_BALL` | activation |
| `BF_ACT_COLD_BOLT` | activation |
| `BF_ACT_COLD_BALL50` | activation |
| `BF_ACT_COLD_BALL100` | activation |
| `BF_ACT_COLD_BOLT2` | activation |
| `BF_ACT_DRAIN_LIFE1` | activation |
| `BF_ACT_DRAIN_LIFE2` | activation |
| `BF_ACT_STINKING_CLOUD` | activation |
| `BF_ACT_CONFUSE2` | activation |
| `BF_ACT_ARROW` | activation |
| `BF_ACT_MISSILE` | activation |
| `BF_ACT_SLEEPII` | activation |
| `BF_ACT_ELEC_BOLT` | activation |
| `BF_ACT_ACID_BOLT` | activation |
| `BF_ACT_DISPEL_EVIL` | activation |
| `BF_ACT_MANA_BOLT` | activation |
| `BF_ACT_STAR_BALL` | activation |
| `BF_ACT_STARLIGHT2` | activation |
| `BF_ACT_STARLIGHT` | activation |
| `BF_ACT_MON_SLOW` | activation |
| `BF_ACT_MON_CONFUSE` | activation |
| `BF_ACT_SLEEP_ALL` | activation |
| `BF_ACT_FEAR_MONSTER` | activation |
| `BF_ACT_LIGHT_BEAM` | activation |
| `BF_ACT_DRAIN_LIFE3` | activation |
| `BF_ACT_DRAIN_LIFE4` | activation |
| `BF_ACT_ELEC_BALL` | activation |
| `BF_ACT_ELEC_BALL2` | activation |
| `BF_ACT_ACID_BOLT2` | activation |
| `BF_ACT_ACID_BOLT3` | activation |
| `BF_ACT_ACID_BALL` | activation |
| `BF_ACT_COLD_BALL160` | activation |
| `BF_ACT_COLD_BALL2` | activation |
| `BF_ACT_FIRE_BALL2` | activation |
| `BF_ACT_FIRE_BALL200` | activation |
| `BF_ACT_FIRE_BOLT2` | activation |
| `BF_ACT_FIRE_BOLT3` | activation |
| `BF_ACT_DISPEL_EVIL60` | activation |
| `BF_ACT_DISPEL_UNDEAD` | activation |
| `BF_ACT_DISPEL_ALL` | activation |
| `BF_ACT_LOSSLOW` | activation |
| `BF_ACT_LOSSLEEP` | activation |
| `BF_ACT_LOSCONF` | activation |
| `BF_ACT_WONDER` | activation |
| `BF_ACT_STAFF_HOLY` | activation |
| `BF_ACT_RING_ACID` | activation |
| `BF_ACT_RING_FIRE` | activation |
| `BF_ACT_RING_ICE` | activation |
| `BF_ACT_RING_LIGHTNING` | activation |
| `BF_ACT_DRAGON_BLUE` | activation |
| `BF_ACT_DRAGON_GREEN` | activation |
| `BF_ACT_DRAGON_RED` | activation |
| `BF_ACT_DRAGON_MULTIHUED` | activation |
| `BF_ACT_DRAGON_GOLD` | activation |
| `BF_ACT_DRAGON_CHAOS` | activation |
| `BF_ACT_DRAGON_LAW` | activation |
| `BF_ACT_DRAGON_BALANCE` | activation |
| `BF_ACT_DRAGON_SHINING` | activation |
| `BF_ACT_DRAGON_POWER` | activation |

## Native validation pin

Clean implementation cac37f59 reproduces45/45 full baseline cd411e55 gameplay
streams, and the real Mage selection/trace parity regression passes. Canonical
[receipts and limits](https://github.com/diegoami/borg-cemetery/tree/e6a3698c113e9b6bf7ebec87bed8a5d30196a62a/validation/borg-reborn-rs/class-attack-preferences)
retain source/build/config identities, original CI/local records and all failed
attempts. harness/fixtures/attack-preferences.json pins the new evidence independently
of the old incident fixture; CI includes both retained archives in its known
registry. No runtime input is changed by this evidence-only follow-up. R parity
and native saved-position scenarios for every other class remain later coverage.

## Recorder integration synchronization

Track-p advanced to46d5f2ae through #393 and froze for review #407 during this
work. The preference branch merges that target as25e89494, preserving every
recorder change and both handovers. It does not move the frozen integration head.
Fresh strict full45/45 key/message/state parity against46d5f2ae passes; Rust checks
and70 engine checks pass again (two local skips). The original-core Fang replay
also reproduces exactly. Current source hash is
`0d05063ed05226f0a7d1b92c7aad844f1ce73994fd9b1615dda7c44a3aaf46e4`.

[Current canonical receipts](https://github.com/diegoami/borg-cemetery/tree/62039b2750d3b106062d5246e77fa3a84d60d1d8/validation/borg-reborn-rs/class-attack-preferences/recorder-sync)
preserve the earlier exact-build evidence and qualify the new source identities.
The candidate's recorder-off executable bytes match the earlier candidate, but
that fact does not replace the fresh source/build receipts. Native preference
validation has an independently reconciled registry, preserving the original-core
observation replay check. Native calibration replay restoration is a separate
Cemetery#13 tooling issue. #405 stays unmerged while #407's review is active;
R carry #406 and main promotion still wait for owner acceptance.
