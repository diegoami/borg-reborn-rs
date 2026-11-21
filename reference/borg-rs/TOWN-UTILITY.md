# Trip-relative town utility (model v5)

> Current repair: [capability coverage and calibration](TOWN-CAPABILITIES.md). The schema-7 implementation builds on the combined M12 runtime; the studies below remain pinned to their original source.

Owner direction, 2026-11-14: replace fixed stock targets with utility that
responds to level and ambition; do not spend the purse merely to complete a
shopping list. Walking home from a shallow dive and storing Recall in the
Home must compete with buying and carrying it. This supersedes the fixed
v4 stock targets, its five-level default ambition, and the per-kind deficit
surrogate for mapped supply purchases. Execution is now **opt-in live** with `borg_town_plan=1`; zero keeps the
historical chooser, with optional comparison recording. See
[the combined runtime and trials](JOINT-POLICY-TRIALS.md).

## What the plan compares

A purchase adds **marginal final-kit utility**, less the opportunity cost of
pack room. The solver maximises that kit improvement **minus the utility
lost by spending gold**, subject to actual gold, slots, weight and the
Black Market caution. Urgent needs retain their existing precedence.

The curve is `C(w, x, d) = w*x/(x+d)`: diminishing returns, with no fixed
number of copies at which supplies abruptly become worthless. Calculations
use integer thousandths, including supply quantities, with final division
rounded down. Weights express priorities, not quantities to purchase.

Substitute foods, potions or scrolls are evaluated together in one supply
need group. For example, CLW and CSW no longer each earn the entire missing
Healing weight independently. Knowledge is deduplicated by kind inside a
shared supply option. Gear alternatives still compete per equipment slot.

## Declaring the journey

- `borg_town_target_depth`: destination, 1–100; zero uses the next frontier,
  `min(max_depth + depth_gain, 100)`, with depth_gain default 1. A first character therefore forecasts depth 1.
- `borg_town_trip_turns`: expected player actions, 1–1000000; zero estimates
  exploration plus travel. It can describe a short depth-2 sortie or a long
  trip at the same depth without changing an inventory quota.
- `borg_town_w_gold`: priority for retaining purchasing options, default 25;
  zero explicitly disables that value.

The default forecast uses 1000 actions per new floor and 50 actions of stair
travel per floor each way. With destination `D`, known maximum `M`, character
level `L` (at least 1), maximum HP `H`, and duration `T`:

```
advancement A = max(1, D-M)
walking W    = 2 * 50 * D
T (default)  = 1000 * A + W
exposure E   = (T / 1000) * (D / L)
```

These are **explicit provisional rates**, not empirical survival estimates.
They define an interpretable forecast whose inputs can be changed and whose
output scales. They are not proof that the new policy plays better. Actual
routing remains the existing dungeon Borg's: the shadow destination declares
a planning assumption, and does not order the character to reach it.

## Supply utility

Here `x` is the kit's aggregate supply in the row's units. The table uses real
numbers for clarity; the implementation uses integer fixed point.

| Need | Unit and scale | Total trip utility |
| --- | --- | --- |
| Light | Sources; `d = T/5000` | `C(w_light, x, d)` |
| Food | Ration-equivalents; `d = T/3000` | `d * C(w_food, x, d)` |
| Healing | HP; `d = H*E` | `E * C(w_healing, x, d)` |
| Escape | Uses; `d = E` | `E * C(w_escape, x, d)` |
| Recall / Trip | Uses | `min(1, W/T) * C(w_trip, x, 1)` |
| Poison | Available cure | Whole condition weight while poisoned, unless a curing spell already cures it |
| Gear power | Positive try-on/try-off difference | `C(w_power, gain, 40000*A)` |

Food and risk exposure scale both demand and the value of avoiding shortages:
longer/more dangerous trips do not dilute the first potion's value by spreading
an unchanged category weight over a larger quota. Recall's travel benefit is
small on a short shallow sortie and rises with the walking avoided. No rule
requires two scrolls, or any scroll, for an ordinary purchase plan.

CLW, CSW and CCW contribute 20, 40 and 60 HP; Healing contributes a conservative
300 HP, *Healing* 1200, and Life maximum HP. Each use is capped at maximum HP,
so a stronger potion cannot heal more than the character could lose. Missing-HP
percentage effects are conservatively omitted. These are healing **capacities**,
not predictions that every use occurs at the optimal HP deficit. Cure effects
also serve the urgent pass, independently of the primary HP utility.

Low foods currently contribute one third of a ration-equivalent; ordinary
rations and mapped nourishing mushrooms contribute one. Remove Hunger now maps
to Food in both regular utility and the urgent pass. These food equivalents and
the 3000-action coverage estimate are approximate: individual nutrition,
current stomach contents, regeneration and speed are not fully modelled yet.
Light likewise uses source counts and an approximate fresh-torch lifetime,
without current fuel or permanent-light distinctions.

Trait values of 1000 denote reusable capabilities, **not** a thousand
physical consumables. Sustainable food/escape/Recall suppress duplicate stock
utility; wound devices do not masquerade as a thousand mana-free potions.
Physical healing stock retains its value despite healing spells. Consumables
receive **no inventory-power bonus** from `borg_power`, including its CLW and
Recall level gates. Durable equipment/tools retain power evaluation.

## Retaining gold

For purse `G` after urgent purchases and dispositions and total planned spend
`c`, the loss of future purchasing utility is:

```
cash_loss(c) = w_gold * c / (G-c+1)
objective   = kit_gain - cash_loss(c)
```

The marginal cost rises as the remaining purse shrinks. There is no hard
reserve of a particular fraction or amount. This cost applies to the **whole
kit**, including knowledge purchases; each kind cannot independently assume
it can spend the same untouched purse. Emergency purchases may still consume
the purse under the existing urgent-pass contract.

The gold DP compares all expenditure points after constructing its rows.
Slot sweeps, weight repairs and fills use the same cash-aware objective. The
record's `solve[0]` is net improvement; individual `plan` entries contain gross
option utility, so summing entries does not reconstruct the net score without
the cash loss.

## Carrying versus the Home

A stored supply retains discounted future option utility:
`(home_discount/1000)*C(w, home_supply, one_use)`, with one use measured in maximum HP for
Healing and one supply unit otherwise. The default factor 0.5 discounts delayed use;
it is an explicit policy coefficient. Additional stored reserves have
less marginal value. Poison has no standing reserve value.

Carrying incurs the opportunity cost of occupying pack capacity, using the
independent space priority (`borg_town_w_space`) as the value of leaving room for a newly found item:

```
pack_burden = w_space * opened_slots / (free_slots+1)
            + w_space * item_weight / (carry_allowance)
home_slot_cost = w_space / (free_home_slots+1)
```

The plan compares the current marginal loss of removing a stack, its burden,
and the marginal future benefit of storing it. Storage can win with room in
the pack: an early Recall is an example. A valuable reserve can later come
back when its current journey utility exceeds its stored option value and
carrying cost. An only cure for a current condition is protected, including
CCW's secondary poison cure. Unknown durable items remain protected from sale. Unaware carried potions and
scrolls keep the accepted sell/drop policy; their diagnostic hidden kind is
not counted as known healing or food. Unknown Home flavours are not taken as
known supplies. When storage needs room, a worthless Home stack may be sold
even for zero proceeds. Pack storage candidates are ordered by marginal carry
utility per weight; valued gear can be stored when overweight too.

Supply totals are updated after **each** disposition. Withdrawals cannot all
claim the same missing supply. When a departure removes a possible pack join,
subsequent purchases conservatively charge a fresh stack slot. Home stacks
are not sold merely because they have no immediate value under no-selling.

## Solver limits and evidence

The shared-need option set contains every pure-kind option generated by the
existing builder, plus a deterministic mixed-kind path chosen by marginal
utility per gold. Every option's aggregate coverage is evaluated exactly.
The mixed path is **not exhaustive**: there is no claim of the global joint
optimum over every possible mixture, gear interaction and disposition. Solver
`optimal` describes the generated multiple-choice problem, as the original
solver's certificate described its own surrogate.

The journey forecast is held fixed during a solve. Buying Recall does not
recursively shorten the supply-exposure forecast; an explicit duration can
state a Recall-based itinerary. Route/destination selection is not jointly
optimised here.

The two-stage order remains: dispositions, then purchases. Stacks are currently
moved whole, and the Home remains absent from the urgent-source pass. Known
unmapped devices, books, ammunition and fuel remain listed in the catalogue.
Neither this change nor shadow comparisons establish improved survival.

Records use **schema 5**, with 24 parameters and physical supply units. Historical
schema-4 records retain their old units and code path; replay still reproduces
the original fixed-target decisions rather than applying the new model to them.
See [the item catalogue](TOWN-ITEM-CATALOGUE.md) and [the validation receipt](validation/2026-11-14-town-utility/README.md) for executable examples and runtime comparisons.

## Zoo calibration parameters

Every parameter below is an integer in the C/Rust settings tables and both
shipped `borg.txt` files. The C control reads and ignores town-plan parameters.
The v5 record stores all 24 town settings in order, including the legacy
`light_depth`/`food_trip` entries (inert in v5). Replay uses the recorded vector.

| Setting suffix (`borg_town_…`) | Default | Meaning / effective range |
| --- | --- | --- |
| `target_depth` | 0 | 0 infers destination, otherwise 1–100 |
| `depth_gain` | 1 | Auto destination advances this many floors; 1–100 |
| `trip_turns` | 0 | 0 forecasts duration; otherwise 1–1000000 player actions |
| `explore_turns` | 1000 | Forecast actions per new floor; 1–1000000 |
| `stair_turns` | 50 | Walking actions per floor per direction; 0–1000000 |
| `food_turns` | 3000 | Approximate actions covered by one ration-equivalent; 1–1000000 |
| `light_turns` | 5000 | Approximate actions covered by one fresh source; 1–1000000 |
| `risk` | 1000 | Exposure multiplier in thousandths; 0–1000000; zero disables standing Healing/Escape exposure, not current urgent cures |
| `home_discount` | 500 | Future supply option value in thousandths; 0–1000 |
| `w_gold` | 25 | Retained-cash priority; nonnegative, 0 disables |
| `w_space` | 25 | Pack/Home room priority, independent of learning; nonnegative, 0 disables |
| `power_scale` | 40000 | Gear-power normalisation per intended floor of advancement; 1–1000000000 |

All eight `w_*` need priorities and the Black Market `share` parameter remain
available. `w_bleeding` continues to mean Healing. Listed rate ranges are
clamped at use, leaving the raw declared parameter in the record. With calibrated
rates `e`, `s`, `f`, `l`, `r`, the default equations become `T=e*A+2*s*D`,
`E=(T/1000)*(D/L)*(r/1000)`, `d_food=T/f`, `d_light=T/l`, and gear scale
`power_scale*A`. HP per potion is game data, not a zoo tuning parameter.

[Portable sample arms](../tools/town-plan/zoo-profiles.json) declare normal mode
and all active utility parameters. They are dictionaries for configuring runs,
not a claim of compatibility with the zoo's native import schema. The bounded
runner now uses seeds 501–509, race 0, all nine classes, 30000 game turns each by default
(the original seven-class receipt remains frozen):

```sh
# Inspect the seven exact commands without starting games:
python3 tools/town-plan/calibrate.py --arm reference
# Execute an explicitly selected arm:
python3 tools/town-plan/calibrate.py --arm reference --run
# Compare one change on the same seeds; results go to a separate directory:
python3 tools/town-plan/calibrate.py --arm cash-cautious --run
```

Each run records the complete configured dictionary in `town-parameters.json`,
the ordinary harness build/config/seed identity, and the v5 plan JSONL. The
runner refuses to overwrite existing evidence. For zoo-native profiles, use
these same setting names/values in the specimen's `borg.txt` and keep the
normal/munchkin mode explicit under [the zoo contract](ZOO-INTERFACE.md).

**These arms measure shadow-plan choices, not different survival outcomes.**
Actual decisions still come from the current Borg, so gameplay and survival
should remain identical until live-mode integration is separately authorised.

## Class-aware calibration

Shared defaults, optional class baselines and per-arm class overrides are now
resolved before birth. The native export uses the zoo's existing `by_class`
and `otherwise` selector. The reference still uses one shared vector for all
nine classes; probe overrides are validation fixtures. See
[TOWN-CLASS-CALIBRATION](TOWN-CLASS-CALIBRATION.md) for precedence, commands,
validation and the live-testing/confirmation gates. New runner manifests are
reported per class, including failures and Ranger/Blackguard.

## Live execution

The owner has now requested the experimental executor and local full-suite
comparison. See [TOWN-LIVE](TOWN-LIVE.md). The default remains current-policy
mode 0; explicit `borg_town_plan = 1` executes the planner. Live exports use
separate `rs-town-live-*` names. The bounded class-calibration runner continues
to protect its shadow-only contract; the dedicated live-suite runner performs
the matched outcome comparison. New records are revision 6; old v5 evidence
and class-profile receipts remain historical and reproducible.
