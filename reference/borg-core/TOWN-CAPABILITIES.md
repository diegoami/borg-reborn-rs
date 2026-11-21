# Town capability coverage repair

The initial live utility model removed consumable inventory-power bonuses but
replaced only a subset of the capabilities they represented. Its catalogue
already listed missing ammunition, devices, books and fuel, but those omissions
were not made a release gate. That was the implementation and validation error;
no parameter tuning could repair it.

The replacement uses one physical supply model for purchases, pack retention,
Home storage and withdrawals. It starts from the combined trip/town runtime
`d9a56f4`; it does not modify that frozen candidate or its zoo studies. Live
execution remains opt-in (`borg_town_plan = 1`), and the shared trip forecast
continues to come from the trip policy when enabled.

## Capabilities

- Ammunition supplies expected ranged damage, only for a matching launcher.
  Rangers additionally value usable shots as a separate trip-relative reserve;
  oil/devices cannot satisfy it. Existing quiver stock counts. Oil reserves whole bottles against the forecast fuel
  shortfall of a lantern; surplus bottles supply thrown damage. A bottle is
  never counted for both uses. Attack supplies share coverage with offensive devices.
- Known devices supply their native effects using actual charges or expected
  rod uses from the trip duration and native cooldown. Teleport Other and
  Stone to Mud are services, not damage; monster-healing/haste/clone/poly devices
  get no attack reward.
- Class books supply spell access when a spell is usable at the current level
  and maximum mana. An owned book closes the access requirement; copies cannot
  unlock the same spell access repeatedly.
- Native effects cover stronger escapes, cures, mana, restoration, curse
  removal, detection, mapping, illumination, temporary resistance, speed,
  combat preparation/control, enchantment, recharging, glyphs and banishment.
  Multi-effect supplies retain their secondary cures. Repair priorities depend
  on a relevant current deficit; mana has no value to a non-caster.
- Food uses native nutrition rather than a ten-kind whitelist. Harmless
  `CRUNCH` messages do not disqualify Hard Biscuits. Whisky's confusion does
  disqualify it as safe food. Fuel uses remaining lifetime, including the worn
  light, and a permanent light suppresses fuel utility.
- Reliable usable detection, mapping, illumination, food and Recall spells
  suppress redundant purchases for those services. Mana-free attack and healing supplies remain valuable
  alongside spells. Unknown carried flavours are not valued using hidden kinds.

Positive supply coverage has decreasing marginal returns. One-time access and
repairs saturate when the actual requirement is met; these are not stock quotas.
The solver merges overlapping capability groups, keeping pure options and a
bounded mixed path. This remains an explicitly approximate option set, not a
claim of a globally optimal kit. Gold, space, weight and Home-delay costs still
apply. The diagnostic catalogue includes possible Black Market kinds, not a
promise every kind can appear or that every special effect is purchased.

## Calibration

Existing trip, risk, gold and room parameters still apply. Six added settings:

| Setting | Default | Meaning |
| --- | ---: | --- |
| `borg_town_w_attack` | 100 | Ranged damage coverage priority |
| `borg_town_w_magic` | 100 | Class-book access and mana priority |
| `borg_town_w_support` | 50 | Detection, tools and combat support priority |
| `borg_town_w_restore` | 100 | Relevant restoration and stat-growth priority |
| `borg_town_attack_turns` | 100 | Forecast actions per ranged attack opportunity |
| `borg_town_support_turns` | 1000 | Forecast actions per support-service opportunity |

These rates are provisional and calibratable, not measured survival estimates.
Class differences come from the actual launcher, usable spells, class books,
mana, resistances and condition deficits, rather than nine separate stock lists.
Longer/deeper trips raise exposure; replenishment loses value as coverage grows.

## Records and the coverage gate

Schema 7 records contain `capability_curves` and `capability_supplies`: evaluated
weights/demands, carried/Home baselines and per-copy engine facts. Replay uses
those facts without loading game data. Historical schema 4/5/6 decisions retain
their original model. Native effective settings report the six new parameters.

`BORG_TOWN_CAPABILITY_AUDIT=<file>` writes one engine-backed catalogue per
process at the first town evaluation: hypothetical known single items, their
native supply vectors and marginal values in the current class context. It does
not insert items into the game or claim purchases. The CI-native coverage job joins this
with ordinary shop stock and require a capability, gear evaluation or an
explicit contextual exclusion. A matching launcher is verified from the native trait record before ammunition
can be contextually excluded. Actual purchases require native retail receipts
and wallet changes; neither this diagnostic nor a planner record is a receipt.

See [the item catalogue](TOWN-ITEM-CATALOGUE.md). Validation results and known
limits belong in the accompanying dated receipt before this is promoted.

The opt-in throwing path reserves the forecast fuel shortfall only for a
fuel-burning lantern, taking its remaining fuel and the planned duration into
account. The control retains its original gate, which could otherwise prevent
throwing bought oil with a torch and no spare fuel. Device replenishment and
recharging remain separate: recharging utility requires a depleted, still-carried
usable device; selling or storing that target removes the demand. Empty devices
are not automatically refurbished by this policy.

Snapshot information services use the configurable support interval as covered
time, so timed sight/telepathy and repeated detection are not mixed as cast
counts versus durations. Acquirement supplies discounted equipment opportunity
using the gear priority and trip exposure. Harmful monster-aiding projections
are excluded even when their native effect uses a status or area form.

Known consumer limits are explicit: this Borg has no known-use action for
Infravision potions or Disable Traps devices, and Deep Descent is outside the
trip router. These are excluded rather than assigned a fictitious benefit.

[Validation and confirmed shopping ledgers](validation/2026-11-15-town-capabilities/README.md)
cover the committed combined runtime. All 104 ordinary kinds are accounted for;
all 530 observed purchases pass native receipt/wallet checks. The nine-start
comparison has two deaths versus one baseline death, so these checks establish
coverage/execution, not improved survival or calibrated priorities.


## Trip-return integration

The integration branch builds on inventory commit `d489117`. It keeps this
capability model and extends learning-by-sale to unaware food and mushrooms as
well as potions/scrolls. Unknown carried stacks with no buyer remain protected.
Plan schema 8 uses the same capability facts as schema 7 and changes the
no-buyer learning disposition; historical schemas 4–7 retain their original
replay decisions. Trip record v3 reports actionable, exploratory and deferred
learning tasks, covering equipment and quiver as well as pack contents. See
[the return rule](TRIP-POLICY.md) for repeat suppression and its limits.

## PR83 correction candidate

The [combined review repairs](TOWN-PR83-FIXES.md) move new records to schema 9,
remove unsupported Monster Confusion combat value, correct branch/condition/rod
supply facts, and make gear/fuel and ammo capacity share their actual limits.
Known Home supplies participate in urgent planning; live purchase completion
requires observed native progress. Earlier receipts above describe their
original source and are preserved. The [new validation receipt](validation/2026-11-15-town-pr83-fixes/README.md)
separates correctness checks, selected class diagnostics and remaining policy
questions. No fresh zoo performance claim or promotion follows from it.
