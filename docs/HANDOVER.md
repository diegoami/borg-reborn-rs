# Handover

*Where the work stands, for the next session. Update it in every PR.*

## 2026-11-18: no stair scum while threatened, a flag for a zoo arm (#549)

Owner, from the replayed zoo death ranger-2107852048 (Grip, game turn 276):
after fleeing town the Borg landed on DL1 on the up staircase home, and the
C Borg's stair scum (`borg_think_stair_scum`, grab DL1 items near the stairs,
then go back up) picked up items while Grip bit it from 14 to 6 HP. It runs
before `borg_caution` and checks only light. The same bug killed
mage-1594647951 (the soldier). #539's switches do not help: the scum returns
first.

- `caution_stair_scum_threat` (0 compiled and in `harness/borg.txt`): 1 skips
  the stair scum while the level is scary, the danger here is above
  `caution_stair_scum_danger` (0), or the HP is below the maximum.
- Flag off: smoke 3/3 identical; the six borg-play deaths replay key for key.
  Flag on: the Ranger returns to town at turn 201 (alive at 20,000), the Mage
  lives (85,841); the other four play identical keys.

Left: a zoo arm against the control.

## 2026-11-18: a monster left with 1 HP is still worth attacking (#546)

The owner replayed the floating-eye Mage 2028599180 (`~/borg-play`) and killed
the eye with Magic Missile or a flask. The Borg's first Magic Missile left the
eye at 1/9 HP. `set_health` estimates HP as `maxhp * (100*hp/maxhp) / 100`,
truncating twice: 9 × 11 / 100 = 0. The overkill caps (damage at most twice
the estimate) then score every attack 0, and `borg_flow_old` walked the Borg
next to the eye, whose gaze paralysed it until it died. New switch
`kill_live_min_power` (compiled 0, shipped 1): a living monster is estimated at
least that many HP.

- The Mage lives: it casts again, then throws oil, and reaches the turn cap at
  CL4. Five of the six `borg-play` deaths now live; the turn-276 Ranger (Grip)
  plays the same as before.
- 18 classes-18 starts on main `9ecbfd1e`: off identical to main (18/18). On,
  16/18 games change (1-HP monsters are common); the same 2 deaths, sum of CL
  95 → 92. No measured gain beyond the Mage; the zoo would measure it.

## 2026-11-18: escapes from a breeder room, two flags for a zoo arm (#544)

Owner, from the replayed zoo death priest-1055640847 (`~/borg-play`): the
Borg dies in a dead-end worm room the owner leaves in six moves from turn
89,000. Found: the stuck jumpstart (bravery boost 3) forgets the known
staircases, so "flee the level" has nowhere to go; and the danger of a worm
crowd saturates at `danger_maximum` on every grid, so retreat and back-away
never move it (236 attacks on one grid). The arm's #503 breeder stand is not
the cause.

- `caution_breeder_exit` (B): only breeders near, one adjacent, outmatched or
  surrounded: the desperate flow to the nearest known chokepoint (corridor or
  doorway: its passable neighbours form two or more groups; review of #545
  replaced a neighbour count that took room corners for corridors) at least
  `caution_breeder_exit_distance` (6) grids from every breeder.
- `caution_flee_known_down_stairs` (A+K): in the dungeon, fleeing with no up
  staircase known, the desperate flow to a known down staircase at any
  distance; the jumpstart keeps the known stairs.
- Both 0 in `harness/borg.txt`: smoke 3/3 identical, and the zoo game
  replays key for key. With either on from turn 88,600 the Borg leaves the
  room and is alive at 120,000. From 89,000 only B does: by then the stairs
  are already forgotten. On from birth, the game diverges early (B first at
  game turn 17,756 on DL1; A+K at 41,761), so this case alone says nothing about
  the population.

Left: a zoo arm against the control (owner's choice of size and runs); the
track for #544 is the owner's to name.

## 2026-11-17: the scary list file shipped on (#518)

Owner: switch to our list. The shipped `borg.txt` sets
`borg_scary_list = scary_monsters.txt` beside
`caution_scary_named_monsters = 1`; both are needed. `borg_scary_list =
none` keeps the compiled C list.
- **`harness/run.sh`:** `BORG_SCARY_LIST=FILE` wins under its own name;
  otherwise the list the `borg.txt` names is copied from `harness/`; `none`
  runs the compiled list with no file and no hash. A copied list's one
  selecting line is rewritten with its SHA-256, and `config.txt` records
  `borg_scary_list` and `borg_scary_list_sha256`
  (`engine/tests/test_scary_list_harness.py`).
- **Harness runs without `BORG_CORE=borg-rs`** play the original reborn
  core, which ignores the list: smoke 3/3 identical to its references.
- **The merge waits for the zoo:** its adapter must copy a list named by the
  shipped `borg.txt` (borg-zoo#228 follow-up). Otherwise zoo cells would
  stop with exit 2.

## 2026-11-18: fleeing: stairs, range, no pointless Phase Door (#539)

The owner replayed borg-zoo#227's Priest 809881907 (`~/borg-play`): at 13/13 HP
it read its only Recall with the up staircase 8 steps away, rested while Grip
closed in, phased twice from a dog twice its speed and died meleeing with 17
flasks of oil. Three switches (compiled 0, shipped 1):
`caution_flee_stairs_before_recall`, `escape_phase_adjacent_only`,
`caution_scary_ranged_no_stairs`.

- The Priest lives (walks back to the stairs). Of the six `borg-play` deaths,
  four live with the switches on; the floating-eye Mage and the turn-276
  Ranger play identical keys (no rule engaged).
- 18 classes-18 starts on main `3727505f`: off identical (18/18); on 5 → 2
  deaths, sum of CL 87 → 90; each switch alone: stairs 5 deaths, phase 3,
  ranged 3.
- Review of #543: "known" was not "reachable". Recall now waits only when the
  flee flows' own stair flow (up: `borg_flow_stair_less`'s; down: its gates,
  or `borg_flow_stair_both`'s on a scary level) reaches the Borg from a free
  staircase, probed without committing. "Something to throw or shoot" is the
  attack planner's own choice (`ranged_means`: `best_throw`, `launch_damage`),
  not any flask; on a scary level without stairs `borg_attack` takes only an
  awake monster in view at range (`ranged_candidate`, shared with the Recall
  and Phase waits). Rerun: off still identical to main (18/18); on 5 → 2
  deaths, sum of CL 87 → 95; alone: stairs 3 deaths / 95, phase 3 / 89,
  ranged 2 / 94. Priest and the same four `borg-play` deaths live; the two
  others still play identical keys.
- A first version walked to the stairs inside `borg_caution` and returned, so a
  fleeing Borg never reached `borg_attack` (sum of CL 87 → 69); the shipped one
  only declines to read Recall.
- Left on #539: a hurt or afraid monster stops being scary (needs the scary
  flag recomputed each think, with door/resist scares kept latched).

## 2026-11-18: contract tests run the environment overrides directly (#540)

Moving the borg-zoo pin (needed for #535's shipped scary list) broke
`engine/tests/test_core_profile_contract.py`: the current adapter clears every
ambient `BORG_*` variable, so the tests' `BORG_CORE`, `BORG_ERRAND`,
`BORG_ERRAND_BUDGET` and `BORG_STOPSCARED` overrides never reached the core.
Those tests now run the game directly (`direct`), with the pinned adapter's
own `config_text` and `command`. The profile tests still go through
`run_cell`. 13/13 pass with the old pin (`c5f1b04`) and with `7b373a70`; the
rest of the engine suite is unchanged between the two pins. Next: the pin
move in its own PR, then #535.

## 2026-11-17: class values and per-class borg files (#520)

Owner: one common borg.txt, per-class files overriding any setting, no
per-class flags in the code; decisions A–D "yes to all" (on #520).

- `params.rs`: every parameter has a common value and optional class values
  (`borg_param_<name>@<class>`, `BORG_PARAM_<NAME>__<CLASS>`, or the pre-#520
  per-class names as aliases). `value()` follows the character's class
  (`set_class`). 81 per-class parameters became 6 (registry 2,116).
- `harness/borg_<class>.txt` hold the 36 shipped class values; the generated
  CLASS FILES block in `harness/borg.txt` carries them (legacy names, so zoo
  profiles keep rewriting them); `scripts/tests/test_borg_class_files.py`
  keeps the block in sync. The zoo adapter needs no change.
- `params-core.json` adds `class_values`; the trace's attack `parameter_name`
  is the family's parameter.
- Checked: the 18 classes-18 starts play identical keys to main `435b07e7`.
- Left: the cfg `borg_*` settings (decision C: later); #518's scary list by
  class can use class values.
## 2026-11-17: borg.txt's formula section removed (#519)

Owner: "Borg.txt is becoming a monster"; the formulas are not used, so remove
them. borg-rs's formula section (Power, Depth Requirement, Restock
Requirement: 790 of `harness/borg.txt`'s 1,362 lines) is deleted; the file is
now 576 lines. `borg-core/borg.txt` (borg-rs's copy, the parity check) keeps
it.

- The shipped `borg_uses_dynamic_calcs = FALSE`: the section was read but never
  processed, and `borg_power`/`borg_prepared` use their built-in calculations.
  Checked: the 18 classes-18 starts play identical keys to main `a8ac685a`.
- Even with TRUE the formulas never took effect: a formula line fails to parse
  and `borg_load_formulas` switches dynamic calculations off (#526, closed as
  moot). With the section gone, TRUE warns "no formulas found" and stays off.
## 2026-11-17: the named scary list as data (#518)

Owner: a flexible scary list, by monster, level and class.
- **Code:** `borg-core/src/scary_list.rs` (ours, declared in the vendor
  check). `borg_scary_list = FILE` in `borg.txt` replaces the compiled
  block in `borg_near_monster_type`; the list loads at `borg_init`.
- **Data:** `harness/scary_monsters.txt`. Each monster has a range
  `tough-fragile`, each class a fragility 0-100 (owner): Fang 5-8 is 5 for
  Warrior, Paladin and Blackguard, 6 for Ranger and Rogue, 7 for Priest and
  Druid, 8 for Mage and Necromancer. Ranges are C's level to C's level + 3.
  Format: `monster | tough-fragile | class exceptions | flags`.
  C's list as a file plays byte-identical keys to the compiled one on 18
  starts; the shipped ranges change 2 of them (Rogue, Ranger).
- **Harness:** `BORG_SCARY_LIST=FILE harness/run.sh ...` copies and pins
  the file.
- **Not shipped yet:** `harness/borg.txt` doesn't name the file, so the zoo
  plays the compiled list (identical) until its adapter copies the file.
  The borg-zoo issue is linked from the PR.
- **Next:** per-class edits, starting where the list costs most (Blackguard
  and Druid lose about 7 levels, #518 evaluation), measured by zoo pilot.
- **The latch (#515) is parked**, on branch `task/T515-scary-latch`
  (`e5fddf0d`, no PR): with the latch re-checked every scan, the Borg still
  reached stairs within about ten turns, so 912 of 985 scares ended by
  leaving the level and the switch changed almost nothing.

## 2026-11-17: the named scary list is on again

Owner, after borg-zoo#216's pilot (with the breeder stand on, 1 death with
the named list against 4 without, with or without the errands plan):
`harness/borg.txt` ships `caution_scary_named_monsters = 1` again, alongside
#514's scary-by-damage rule (60). The owner wants the list flexible: #518
moves it to a data file by class and level (designed, on hold: the owner
first evaluates a per-class layout, possibly with #520's class files).

## 2026-11-17: cornered mode (#504, #488 slice B)

- `caution_cornered` (compiled 0, shipped 1): cornered (no Phase Door,
  teleport or escape, a monster within `caution_cornered_radius` 2, danger
  at or above HP x `caution_cornered_danger_tenths` / 10) the Borg uses every
  buff regardless of `borg_defend`'s "some danger but not much" window:
  fear cure, Speed, Heroism, Berserk Strength, Blessing, then Monster
  Confusion against an adjacent unique or higher-level monster
  (`caution_cornered_big_level_margin` 0). Melee classes then neither retreat
  nor back away. Monster Confusion leaves the planner's worthless list only
  while the mode is on (the planner still gives it no value: no core
  supply curve reads TMD_ATT_CONF).
- On 193d9076 (run 44054277431's death starts): Blackguard 888896726
  (Berserk Strength cast 9 times) and Rogue 1502695781 survive; the Ranger
  (bleeding) plays identical keys (it has Phase Door); Ranger3 dies to a
  crow instead of a centipede; the Druid is unchanged.
- On main, 9 classes x 2 seeds (2001, 2002), 100k turns: 6 deaths off, 3 on
  (the same three, identical keys: Mage, Druid, Priest); rescued a Priest,
  a Paladin (lava) and a Blackguard. Too few games: a zoo arm is needed.
- Re-synced with main after #510 (#503) merged.
- Review (#511): buff spells only at or below `caution_cornered_spell_fail`
  (25%) failure, Haste Self before the Speed potion; no Monster Confusion
  against `RF_NO_CONF`; monsters that never move do not corner the Borg;
  the outmatched heal stays at the end (tried before the boosts, it made a
  Rogue drink in front of a grey mold instead of stepping away, and a Ranger
  heal in place against Bullroarer: two new deaths on main `dd5f026e`); Teleport Level is an escape (Recall is
  not); the boost order is a pure `boost_order`, unit-tested.
- Owner: in a hard breeder fight the same boosts.
  `caution_breeder_boost_danger_tenths` (compiled 0, shipped 5): standing
  against breeders with no easy escape and danger here at or above half the
  HP, the Borg uses `cornered_boost` (fear cure, Speed, Heroism, Berserk
  Strength, Blessing: the Bless spell, Holy Chant or Blessing scrolls).

## 2026-11-17: healing by class, no Shovel while poor (#487)

- #487's analysis (on the issue): no melee class bought a weapon in five
  games; for Warriors and Rogues the try-on rates every heavier weapon about
  -44,500 power per lost blow, so keeping the Dagger is right; Paladins,
  Blackguards and Rangers miss upgrades by a few gold. The gold went to
  Cure Light Wounds for the 400% reserve (a CL8 Warrior bought 26 in one
  visit and kept 712 gold) and to a Shovel (66 gold) at four of five first
  visits.
- Owner: per-class overrides of the healing curve scale and reserve
  (`town_heal_curve_scale_percent_<class>`, `town_heal_reserve_hp_percent_<class>`),
  shipped from 50 (Warrior) to 150 (Mage) and from 100 to 300; the general
  reserve 100 and curve scale 125; `town_digger_min_gold` 500. Weapon saving
  is not done.
- Owner: healing speed (#491) was not meant to set how many potions fill the
  reserve. `town_heal_reserve_real_hp` (shipped 1) counts real HP in the
  reserve; speed weights only the healing curve.
- On main `233ef23e`, 9 classes x 2 seeds: with the old values (reserve 400,
  scale 100, `town_heal_reserve_real_hp` 0, no class values, digger 0) the 18
  games play identical keys to main. With the shipped ones (real-HP reserve
  included) 4 deaths become 2: the four that died live (Mage, Necromancer,
  Ranger, Blackguard), and two others die (a Druid to an acolyte, a Priest to
  a green worm mass). Cure Light Wounds bought 212 -> 117, healing gold
  7190 -> 5430, Shovels 20 -> 8; Warriors' healing 590/1540 -> 130/380 gold.
  Too few games: a borg-zoo experiment follows at the merge.
## 2026-11-17: scary by damage (#505)

Owner: scary is a monster that can one- or two-shot you, as a percentage of
HP. borg-zoo#216 saw the named list's removal (#510) kill characters (5 and
4 deaths of 18 against 1).

**Change:** `caution_scary_damage_percent` (compiled 0, shipped 60, owner).
A known monster on the level makes it scary when one character turn of its
melee, on average, takes that percent of max HP:
`borg_caution::scary_by_damage`.
- The damage is the average of its blows (`randcalc` AVERAGE: base, dice
  and level bonus, review of #514) times its moves per
  character turn.
- Not `borg_danger`: a first try with it rated a street urchin at 245 and
  a floating eye at 276 against 10 HP, from the penalties for stealing and
  paralysis.
- Spells are not counted.
- A note `# Scary by damage: <monster>, <damage> of max HP <hp>` marks when
  it first makes a level scary.
- The named list stays shipped off; the resist-based scares and the latch
  stay.

**Local check:** 18 starts; the table is in docs/CALIBRATION.md. The rule
keeps the levels and depth of "list off", where the named list loses about
a quarter of them. Deaths can't be told apart at this size.

**Left:**
- a zoo pilot (list on, list off, the rule at 30/60/80) decides the value;
- whether spells should count;
- the latch, if the zoo shows it holds the Borg.

## 2026-11-17: a planned purchase buys its copies in one transaction (#507)

Found by the zoo on borg-zoo#198. Half-Troll Ranger 60792140 bought 120
Arrows one transaction at a time: ~1,080 keys at one game turn, which the
zoo counted as a stall. The zoo's half, borg-zoo#205, is closed: borg-zoo#207
now flags such runs as `purchase_burst`.

**Cause:** live mode set every action's quantity to 1, and `buy_selected`
accepted the prompt's default of 1. The planner then re-solved after
every copy.

**Fix:**
- A BUY keeps the copies the plan picked of the first ware of its chosen
  option (`borg_town_shadow::evaluate`). Copies of the same class at a
  second ware, and other kinds in the option, come on later re-solves.
- `execute_store` caps them by the stack, the pack's room
  (`inven_carry_num`) and the gold at the price of one (`price_item`):
  `borg_town_live::purchase_quantity`, unit-tested. Never above store.c's
  maximum, so 2 or more always meets the prompt.
- `buy_selected` types the quantity.
- The purchase note and object log name the copies bought, and the log's
  cost is the gold the purse lost (for an uneven stack of wands or staves,
  `price_item`'s total is not N times the price of one).
- With `store_buy_direct` set the engine buys one copy with no prompt, so
  the quantity is not typed. The shipped `harness/borg.txt` does not set it.
- TAKE_HOME and sales still move one copy per action.
- The fix is in the vendored core (`borg-core/`), so it plays only when
  the borg-rs core is selected: `borg_reborn_core = borg-rs` (shipped since
  #479) or `BORG_CORE=borg-rs`. The original reborn core is unaffected.

**Checked:** three Ranger starts, 100,000 turns, on main `28360152` and on
the branch:
- the largest key count at one game turn fell from 149–330 to 72–80;
- arrow purchases fell from 87–237 transactions to 14;
- no "not enough gold" or "cannot carry" message.

Play changes, and more arrows are bought in all (276–302 against 87–237),
so it needs a zoo pilot. Deaths on a specimen with it need new Cemetery
packages (RULES 12).

**Still open from #491:** the healing reserve can crowd out Phase Door for
poor characters. A local run of 18 starts found 114 departures without
Phase Door at reserve 400, against 38 with no reserve. The owner chooses
the fix after borg-zoo#192's small and medium runs: a cap on the
reserve's gold, or a reserve that grows with level. Not a Phase Door
floor: the owner rejected it, since Phase Door is not in the same league
as healing. Those runs should keep the surviving games' purchase records,
because deaths alone cannot show the crowding.

## 2026-11-17: stand and fight breeders; named scary monsters off (#503)

#488 (stand-and-fight) is split into #503 (A, breeders), #504 (B, the
cornered mode) and #505 (C, the scary-monster list). Owner: removing the
hard-coded scary monsters belongs in this PR.

- `caution_breeder_fight` (compiled 0, shipped 1): with breeders on the
  level and no easy escape (a free staircase within
  `caution_breeder_easy_escape_steps` 5, Phase Door, teleport, escape or
  Word of Recall) the three breeder flees are skipped, and while every
  known monster within `caution_breeder_stand_radius` 5 is a breeder the
  Borg neither retreats nor backs away (the Rogue's death ping-ponged
  between two grids). Warriors, Paladins, Rogues and Blackguards hunt
  breeders below CL10 while at most `caution_breeder_hunt_max_count` 4 are
  known.
- `caution_scary_named_monsters` (compiled 1, shipped 0): the C Borg's
  named scary monsters by character level and the low-level door scares.
  The scary flag latches until a new level: a small kobold seen once kept
  the Blackguard 888896726 fleeing for 54,000 game turns. Resist-based
  scares stay. With it off, breeders scare through nothing; their handling
  is the breeder flee / stand above.
- Replays (run 44054277431's death starts): on 193d9076 all five die; with
  the shipped settings all five reach the turn cap (CL5–8, DL1–3). On main
  all six starts reach the cap without the change; with it, five do, at
  CL6–8 instead of CL4–6, and the Druid 934061548 dies to a kobold shaman on
  DL2 (CL6), on a later path.
- Review (#510): Teleport Level counts as an easy escape; the no-retreat
  rule needs no non-breeder within the danger horizon (20 grids), not only
  within the radius; the generic low-HP and bleeding/poison flees are left to
  #504; with the named list on, a breeder scare latched before the stand
  engaged stays until a new level (a known limit of that intermediate arm). Too few games to claim anything: a zoo arm is
  needed (named list off / breeder fight off / both).
- `engine/tests/test_errands_plan.py` growth test: seed 1003 (the plan
  feature's test; the shipped default stays no plan).
- Left: #504 (cornered mode), #505 (now: whether the resist-based scares
  and the latch need changes; to be decided with the owner).

## 2026-11-17: CI cancels superseded pull-request runs (#499)

Owner: are we doing too much on CI? The repositories are public (no paid
minutes), but runner slots are shared with borg-zoo's experiments, and
2026-11-17 had 43 CI runs. `.github/workflows/ci.yml` now has a concurrency
group per pull request with `cancel-in-progress` for `pull_request` events:
a new push to a PR cancels its older run. Each push to main or track-p has a
group of its own (the run id), so those are never queued or cancelled. The
session's practice from now on: batch review fixes into one push, and re-sync
a PR with main only right before merging, when needed.

## 2026-11-17: explicit gold reserve, shipped 0 (#493)

Owner: the gold reserve is a borg.txt setting, fixed and percent, and it
should be 0. Before, the live town planner kept gold only through
`borg_town_w_gold` (25), a price in `solve_cash` (spending `c` costs
`w x c / (gold left + 1)`) that made the last coins nearly unspendable.

- New parameters `town_gold_reserve` and `town_gold_reserve_percent`
  (default 0): `Budget::of` takes the larger from the gold left after the
  urgent needs (which may still use the whole purse);
  `borg_town_budget::{gold_reserve, spendable}`, unit-tested.
- Shipped `harness/borg.txt`: both at 0 and `borg_town_w_gold = 0`; the
  compiled default is 0 too (owner), declared in `borg_init.rs`
  (`OUR_DEFAULTS`, `NEW_SETTINGS`). The soft price came from borg-rs
  `785f5064` (2026-11-14, its docs/TOWN-UTILITY.md "Retaining gold"),
  vendored with borg-rs #87.
- The reserve bounds the purse at arrival; the visit's sale proceeds are
  spendable on top of it (review F1, documented rather than changed).
- Shipping `borg_town_w_gold = 0` changes the borg-rs core's key stream, so
  zoo deaths on a specimen with it need new Cemetery packages (RULES 12).
- On borg-zoo run 44054277431's death starts (main's defaults): the median
  gold on leaving town fell from 143/64/48/96 to 7/9/46/6; a 50% reserve
  holds (median 138). Healing purchases moved only partly (#491 is the
  healing value).
## 2026-11-17: no merge before the owner says the review is done (RULES 1a)

Owner, after #496 was merged while it was under review: every PR waits
for the owner to say its review is done, on every track and into any
branch. Green CI is not enough. RULES 1a and AGENTS.md "At the end of a
task" step 4 say so; track P's `track-p` task merges follow it too.

## 2026-11-17: point-blank divisor by class (#485 follow-up)

Owner: point blank 2 (#495) suits Rangers but not Rogues; keep a default
divisor and a per-class override.

- `attack_arrow_point_blank_divisor` stays the general divisor (compiled 5).
- Nine new parameters, `attack_arrow_point_blank_divisor_<class>` (default 0 =
  the general divisor), resolved in `borg_fight_attack::point_blank_divisor`.
- Shipped `harness/borg.txt`: general 5, Ranger 2, Rogue 3 (the second
  missile class). Every other class is back to borg-rs's 5; Rangers keep
  #495's 2.
- Registry 1783 -> 1792; `docs/CALIBRATION.md` and `docs/ZOO-INTERFACE.md`.
## 2026-11-17: Boldness is no buff to the town planner (#486)

The planner bought 14-21 Pints of Fine Wine a game (borg-zoo run 44054277431's
death starts on main's defaults), and the core never drank them for what they
were bought for. Wine (`object.txt`: weight 10, cost 2, `NOURISH:INC_BY 3`,
`TIMED_INC:BOLD`) has been no food to the planner since #444, but `timed_cap`
mapped `TMD_BOLD` to `BUFF`, the cheapest buff on offer.

- A Bold timed effect is no buff the core uses. The core takes Boldness only
  to cure fear: a Potion of Boldness is `CURE:AFRAID`, priced through FEAR,
  and `borg_caution` / `borg_recover` quaff it when afraid. The hunger list
  drinks wine as food.
- `TMD_BOLD` no longer maps to `BUFF` (`borg_town_capability::timed_cap`).
  The rule is effect-wide on purpose: the only kind with `TIMED_INC:BOLD` in
  `object.txt` is the Pint of Fine Wine (no activation or class spell grants
  it), and Potions of Boldness keep their fear-cure value (tested).
- On the four starts: wine bought 21/20/16/14 -> 0, 14-21 lb less carried.
  The freed gold isn't spent on healing; the healing value is a diminishing
  curve (HP x exposure), which is borg-reborn-rs#491.
## 2026-11-17: review of #496 (post-merge), fixes

The owner's review of #496 found no behaviour defect: the maths, "off is
identical" and the mage's death at reserve 800 all reproduce. Fixed here:
- **F1:** `docs/ZOO-INTERFACE.md` gave the registry as 1783; it now says
  1795 (1786 after #496, plus #498's nine per-class divisors).
- **F2:** the CLW/CSW/Healing ranges in docs/CALIBRATION.md and below were
  max HP, written as if they were potion HP. Reworded, with
  `object.txt`'s figures.
- **F3:** `tools/shopping-audit.py` reads `borg_town_capability::NAMES`
  from the source instead of a hand-kept copy, which lacked "healing
  reserve". Tested (`tools/tests/test_shopping_audit.py`).
- **F4:** docs/CALIBRATION.md says that the curve scale also multiplies
  healing's total value, and that the speed factor also changes the curve's
  shape.

## 2026-11-17: a healing reserve for the town planner (#491)

Owner: "we can never have enough healing". The owner chose
configurable borg.txt values instead of the issue's three proposals: a
linear reserve as a percent of max HP (as in proposal 1), a value for the
diminishing curve past it, and then healing speed, with proposed defaults.

- **Code:** `borg_town_capability.rs`, details in docs/CALIBRATION.md.
  - `town_heal_reserve_hp_percent` (0 = off): up to this percent of max HP
    of healing, each HP is worth the curve's first HP (`w_bleeding / max HP`),
    on top of the curve. Bought first, never sold below.
  - `town_heal_curve_scale_percent` (100 = today): stretches the curve past
    the reserve, keeping its first HP's value.
  - `town_heal_speed_percent` (0 = HP alone), owner: "budget healing speed
    as well". A potion that heals at least this percent of max HP in one
    swallow counts in full; a smaller one counts `heal x heal / (percent x
    max HP)`. HP per gold alone made Cure Light Wounds the best buy at every
    level: 800% of a 300-HP character's max HP is 120 CLW, each healing
    20 HP. At 66, CLW counts in full up to 30 max HP; the best buy moves to
    CSW at 60–100 max HP, then Healing from about 200 max HP. CCW (60 HP for
    200 gold) never wins on healing alone.
- **Off is identical:** at 0, 100 and 0 the four starts of the issue play
  the same key streams as main `4d83dd26`.
- **Shipped (owner):** reserve 400, curve 100, speed 66, in
  `harness/borg.txt`.
  - 800 was proposed first. In CI, engine test
    `test_active_cap_is_replaced_when_level_changes` (mage, seed 1001)
    failed with it: at 10 max HP, 800% is 8 Cure Light Wounds, about 160
    gold, almost all the starting gold. The mage bought no oil and died at
    level 1 to a grey mold (turn 4,084).
  - With 400 + 66, or 66 alone, the mage reaches level 2 alive.
  - On the four starts, 400 + 66 bought 13 / 6 / 5 / 12 healing potions
    (main 6 / 4 / 4 / 5). The Blackguard died early (turn 15,735), as it
    did at 400 without speed. One game each, so an anecdote.
  - Left: a zoo arm for reserve {400, 800} x speed {0, 66}, and possibly
    a reserve that grows with level, so early characters keep gold for
    their attack.
- **Speed 66 on the four starts:** off still equals main and reserve 800
  is unchanged by the new code.
  - Rogue, Blackguard: unchanged, with or without the reserve.
  - Ranger (34 HP) with reserve 800: 21 potions bought, 570 gold (CLW and
    CSW), alive.
  - Druid with reserve 800: 30 potions, 650 gold, alive.
  - No deep character is measured.
- **Measurement:** the four starts, 100k turns, branch build. Healing
  potions bought, gold on healing / total, outcome. One game per cell,
  descriptive (RULES 14).

| setting | Rogue | Ranger | Blackguard | Druid | deaths |
|---|---|---|---|---|---|
| off (= main) | 6, 270/2337 | 4, 80/899 | 4, 80/322, **died** | 5, 100/1168 | 1 |
| reserve 200 | 2, 40/814 | 3, 60/864 | 4, 180/761 | 5, 100/596 | 0 |
| reserve 400 | 10, 400/1432 | 11, 370/1051 | 5, 100/192, **died** | 12, 240/650 | 1 |
| **reserve 800** | 11, 220/1517 | 17, 340/837 | 9, 180/571 | 29, 780/1928 | 0 |
| reserve 1200 | 31, 720/2059 | 28, 560/1198 | 10, 200/215 | 14, 280/537, **died** | 1 |
| curve 300 | 17, 390/959 | 2, 40/117, **died** | 4, 80/138, **died** | 11, 270/793 | 2 |
| reserve 400 + curve 300 | 7, 140/1088 | 10, 300/1188 | 5, 100/192, **died** | 13, 260/593 | 1 |
| reserve 800 + curve 200 | 10, 200/1395 | 12, 240/1122 | 10, 200/941 | 8, 160/560 | 0 |

- **Reading:**
  - Reserve 200 is at or below what these low-level characters already
    carry, so it buys little.
  - Reserve 800 multiplies the healing bought by 2–6 and still leaves
    gold for arrows, oil and Phase Door. 1200 starts to crowd them out.
  - Stretching the curve alone (curve 300) bought more healing on two
    starts but had two deaths against one; no sign it helps outcomes.
  - Every game diverges early, so deaths are anecdotes.
- **Left:**
  - #496 review: the reserve's total value grows with max HP squared (its
    weight and its size both grow with max HP). At 400%, a 300-HP character
    can put about 3,200 gold into it (4 Healing). A reserve that grows less
    than linearly in max HP, or with level, would keep early characters
    cheap.
  - No deep character is measured. With speed 66 the reserve fills with
    Healing potions from about 200 max HP, not Cure Light Wounds.
  - Zoo pilot requested: borg-zoo#192, reserve {0, 400, 800} x speed
    {0, 66} on main `7e08c920`.
  - #494 (gold reserve, `borg_town_w_gold = 0`) also changes what gold is
    left for healing. #492 (no wine) is merged; it freed 14–21 lb and a
    little gold.

## 2026-11-17: rising to town explores for an up staircase (#483)

borg-zoo#170 run 44054277431 (specimen 193d9076): Ranger 1739161339 decided a
trip return 16 times and never reached town. With connected stairs, each
descent (21 of them through the scary-level flee in `choose_stairs`, the rest
through "no up stairs found, going down") landed it on an up staircase, and
each climb on a down staircase.

- `borg_caution::rising_holds_descent` holds a descent that is not a
  desperate escape while all of these hold:
  - `goal.rising`;
  - below DL0;
  - within `caution_rising_explore_ticks` (300 Borg ticks, one per think,
    restarted on each new level);
  - danger here at most avoidance x `caution_rising_descend_danger_tenths`
    (10) / 10.

  The Borg then explores for an up staircase.
- Applied in `choose_stairs`, in the flee block's `borg_flow_stair_more`, and
  in `borg_leave_level`: its "no up stairs" branch (note `# no up stairs
  found, exploring for them (rising)`) and, after the review, its whole "Go
  Down" block (note `# Holding a descent while rising to town`): bored,
  "spent too long on level", the trip policy.
- Desperate stair flows are unchanged.
- The budget is per level, not per return: it bounds the search on one level.
- Default 300 is the owner's (2026-11-17), between the 250 and 500 tried.
- On 193d9076 plus the change: Rogue 1502695781 (unknown2) and Ranger
  1739161339 (unknown3) live to the turn cap (they died); the other three
  deaths play identical keys.
## 2026-11-17: point-blank shots keep half their damage (#485)

Investigation of the Ranger 1739161339 death (borg-zoo#170, unknown3): its
bow scored 4 (8 with the Ranger preference) against melee 22–64.

- **Cause:** not the launcher's damage. Bow and Main Gauche both score 4,
  and the engine agrees (1.2 shots, 1.17 blows, about 6 each per turn).
  The gap is the danger bonus. Melee adds the target's danger over 2 turns
  at full damage; `borg_launch_bolt_aux_hack` adds 1 turn weighted by the
  monster's hit chance, 0 for a monster 2 squares away (same as C). An
  adjacent non-unique could not be shot at all: damage / 5 = 0. The engine
  has no point-blank penalty; arrow breakage is the same at any distance.
  Per-term numbers are in the issue.
- **Change (owner):** shipped `harness/borg.txt` sets
  `borg_param_attack_arrow_point_blank_divisor = 2`. The compiled default
  stays 5. It applies to launcher ammo and thrown objects of every class.
- **Replay:** with 2, the Ranger is alive at the 100k-turn cap (CL8, max
  DL7) instead of dying at turn 74,797. It diverges at turn 6,573, so this is
  one game, not evidence (RULES 14); a zoo arm should measure it.
- **Left:** the danger-term asymmetry itself (a `full_damage` parameter for
  the launcher's term) is not changed. Owner's call after the zoo measure.

## 2026-11-17: the speed30 bundle is the default, apart from the errands (#479)

Owner request: the settings of borg-zoo#170's arm `reborn-bundle-plan-speed30`
become the defaults, apart from the errands, and the default core switches to
borg-rs ("reborn is not ready yet").

- **Shipped `harness/borg.txt`:**
  - `borg_reborn_core = borg-rs` (was reborn);
  - `borg_town_plan = 1`;
  - `borg_trip_policy = 1`, `borg_trip_unknown_stacks = 5`, `borg_trip_pack_slack = 1`;
  - 18 `borg_param_*` lines: outmatched healing 1, speed fight 30, and the 16 class attack preferences.
- **Not set:** `borg_plan` and `borg_reborn_errand` (none).
- **Compiled defaults are unchanged.** `params.rs` and `borg_init.rs` stay neutral; unit tests pin them as the neutral and C-parity tables. The bundle comes from the shipped file, which a player's game and every zoo arm read. A zoo profile that names a key rewrites the line.
- **`harness/run.sh`:**
  - With no `BORG_CORE`, it rewrites the run's copy to `borg_reborn_core = reborn`, so the harness's reborn-core references (smoke, full, observations) still test the reborn core.
  - New `BORG_TXT_SET="name=value;..."` pins an existing key.
- **`test_shopping`'s home-storage round** pins `borg_town_plan=0`, the historical chooser it tests.
- **Measurement:** borg-zoo#170's pilot was identical across speed 0/30/60 on 18 starts. The medium tier (run 43998795815) is the measurement of the bundle.

## 2026-11-17: never use unidentified items; Deep Descent discarded (#470)

Owner request after borg-zoo#147: 35 of 900 small-tier games ended below
their max-depth cap, mostly through Deep Descent (an unknown scroll read in
an emergency, or the known scroll as an escape).

- **Never use an unidentified item.** The three emergency gambles in
  `borg_caution` (poison/cure section, after the wound patch, and C's last
  resort) no longer quaff, eat, use or read unknown items;
  `borg_quaff_unknown`, `borg_read_unknown`, `borg_eat_unknown` and
  `borg_use_unknown` are deleted. Combat no longer aims an unknown wand or
  zaps an unknown rod (`BF_WAND_UNKNOWN`/`BF_ROD_UNKNOWN` score 0). Five
  parameters went with the code (registry 1785 to 1780; docs/CALIBRATION.md).
  `borg_test_stuff` (Identify Rune) is unchanged.
- **Sell one of each unknown kind.** Under the town plan an unaware wand,
  staff or rod is now sold like an unknown potion, scroll or mushroom
  (`borg_town_shadow::kit_of`). The live planner sells one copy per action;
  the sale reveals the kind (also under `birth_no_selling`), and the rest of
  the stack is then valued as the known kind. The historical chooser
  (`borg_town_plan = 0`) already sold unknown potions and scrolls one at a
  time; `borg_good_sell` now does the same for unaware mushrooms and devices.
- **Deep Descent.** Never read or activated (`borg_teleport_off_level`).
  Once its flavour is known, `borg_junk::borg_discard_deep_descent` ignores
  the kind: the engine drops every carried copy and hides floor copies, so
  none is picked up. The planner values it 0 (`WORTHLESS`), so no shop buys
  it. Its scoring value stays 0.
- Always on: no switch.
- Not changed: Teleport Level and fleeing down stairs, the other known ways
  below a cap (#470 excludes them).
## 2026-11-17: Speed before a very dangerous fight (#469)

- Owner: when the danger is very high and the Borg fights rather than
  retreats, drink a Potion of Speed. `attack_speed_fight_danger_tenths`
  (0 = off, default): once `borg_attack` has chosen a positive best attack
  other than waiting, a Borg in the dungeon, not fast, with a Speed potion,
  drinks it first when the danger from monsters that can move (no
  NEVER_MOVE) is above avoidance x value / 10. This is now the only change
  that drinks Speed: #465 merged as hoarding only (its last-resort quaff was
  dropped). The core's own `borg_defend_aux_speed` is unchanged.
- Found: counting every monster, grey mushroom patches (NEVER_MOVE, spore
  danger 204) set it off at any threshold. Moving monsters only, it fires 2
  (setting 20) and 1 (setting 50) times in 180 starts of the ambitious arm
  (4,2), at thieves (scruffy looking hobbit, cutpurse), with no rescue or
  new death; off is byte-identical (12 of 12). Descriptive (RULES 14).
- Left: those games end near DL2, where Speed rarely matters; the setting
  needs a zoo arm with deeper play. Thieves' danger is inflated by theft.

## 2026-11-17: a game's start leaves out borg.txt and flags (#464 follow-up)

- Owner-approved: a new borg.txt setting changed the file's hash and made
  every control game UNRECORDED on #465, with identical key streams. The
  start is now implementation, seed, race, class and save; a setting that
  changes play changes the key stream, which must still match.
- `death-tools-pin.txt` moves to `09225018` (borg-cemetery
  `tools/relaxed-death-match-on-1afd942`; the same commit is borg-cemetery
  #54 against main). RULES 12 and docs/DOSSIER-IMPORT.md say so.
## 2026-11-17: hoard Potions of Speed (#452)

Owner: keep Speed potions as an emergency reserve. The last-resort quaff that
was tried with them is removed (owner, 2026-11-17).
- **Hoard** (`borg_town_capability.rs`): a speed effect (`TIMED_INC:FAST`,
  20+2d10 turns, +10 speed, object.txt Potion of Speed) supplies one use,
  and the SPEED curve is escape-like (`borg_town_w_escape` x exposure, like
  PHASE/TELEPORT). It was `amount` turns against the whole trip, about 1% of
  the demand, under a slot's burden: sold or stored. Unknown flavours are
  still sold (unchanged, #445's question). Unit test: a level-25 character
  on a 100000-turn trip keeps a stack of four; the old curve does not.
  The core drinks Speed only as before (`borg_defend_aux_speed`).
- **Measured** (180 classes-1800 starts, ambitious arm (4,2), 100k turns;
  descriptive, RULES 14): Speed sold known 57 -> 0, stored 13 -> 0, quaffed
  43 -> 58, deaths 33 -> 29, deaths with Speed still in the pack about 4 -> 1.
  The planner now also buys Speed (9 potions in 180 games, one never used).
- **Tried and removed:** `borg_speed_last_resort` (drink Speed when no heal,
  phase or teleport is left). As first written it never fired (any Cure
  Light Wounds potion counted as a heal). Given precedence over CLW it fired
  in 6 of 180 games, every time at full HP against a harmless monster whose
  danger estimate exceeded the Borg's HP (floating eye 460, giant white
  mouse 200, scruffy looking hobbit 63): it wasted the potion and rescued
  nothing. The danger estimate alone does not mark a real emergency.

## 2026-11-16: no knowledge-only purchases (#460)

- Owner: "turn off the buying of useless unknown objects". Cause of the
  Scroll of Monster Confusion buys (replay 1285833235, game turn 13116,
  shop 4 slot 10): not a supply path. `WORTHLESS` does blank
  `capability::supplies()` for it, and the plan entry's value was exactly
  25000 = `borg_town_w_knowledge` x UNIT, the first-copy "learn" bonus of a
  ware whose `serves` carries the knowledge bit. #444 zeroed that weight
  under `borg_trip_policy = 1` in `Weights::of`, but `step()` rebuilds its
  settings from the 24 recorded town params (`state_of`), and the trip policy
  is not one of them, so the zero never reached the plan: the weight was 25 in
  every arm.
- Change: `borg_town_shadow::effective_params` zeroes `borg_town_w_knowledge`
  in the params the step runs on and records (`configured_params` still holds
  the configured value). Off in every arm, trip 0 and 1; no schema, parameter
  or default change; replays of new records reproduce the plan. The IDENTIFY
  capability (Identify Rune scrolls, only with unknown runes carried) still
  reads the configured weight. `borg_town_shadow.rs` is declared HOOKED.
  Tests: the effective params, and a knowledge-only ware is planned at the
  configured weight and not at the effective one (both planner modes).
- Local replays (before = main build, after = this change; flavour-unknown
  buys counted from messages.log): 1285833235 trip 1: 6 buys 155 gold with
  one Monster Confusion, died of poison at clvl 4 -> 4 buys 58 gold (CLW,
  Phase Door, Boldness, Detect Invisible), alive at the 100000-turn cap;
  2109297965 trip 0: 7 buys 210 gold, killed by a wild dog at clvl 1 -> 5
  buys 185 gold, alive at the cap (clvl 7); sanity cell 501/0/0 ambitious:
  main keys a80b9ac6... -> ba6181b9..., 9 buys 238 gold, died to Lagduf at
  clvl 8 -> 4 buys 113 gold, alive at the cap. Play diverges from the first
  changed purchase, so end states are not evidence of improvement (RULES 14).
  What remains unknown-flavour was bought for its kind: CLW, Phase Door,
  Magic Mapping, Cure Serious Wounds, Boldness, Detect Invisible.
- A `WORTHLESS` check cannot cover a flavour-unknown ware by itself: in a
  store the ware carries its real kind, and `supplies()` prices it by that
  (so a known-worthless kind in stock has no supply on any path), but the
  knowledge bonus does not look at the kind. With the weight zero no
  worthless kind has value on any planner path.
- Question for the owner: should trip 0 keep buy-to-identify behind
  `borg_town_w_knowledge`? The issue said to ask; this change turns it off
  there too, as the owner's words ask. To restore the knob, read the
  configured weight again in `effective_params` (one line). The historical
  chooser (`borg_town_plan = 0`) has its own `borg_buy_identify` (default 0 in
  harness/borg.txt, #27 learning list including Monster Confusion and Remove
  Hunger, `borg_buy_strategy.rs:142`); not touched.
- Low-value kinds still bought for their capability (use sites in the core):
  Boldness `borg_caution.rs:1987`, `borg_recover.rs:349` (cure fear);
  Resist Heat `borg_fight_defend.rs:720`, Resist Cold `:797`; Blessing and
  Holy Chant `:366-367`; Detect Invisible scroll `:2774`; Remove Hunger scroll
  `borg_recover.rs:360` (and `borg_trait.rs:1920` counts it, the planner does
  not count it as food). The planner prices them through `timed_cap`
  (`borg_town_capability.rs:161-166`: BOLD/BLESSED to BUFF, OPP_FIRE/COLD to
  FIRE/COLD, SINVIS and EF_DETECT_INVISIBLE_MONSTERS `:253` to INVISIBLE),
  with the ordinary weight (`:492`, `:502-507`); the owner decides whether
  those curves are too generous.
## 2026-11-16: Remove Hunger scroll: value, carry, read before food (#451)

Owner: the scroll should have value, be carried, and be used before any food.
- **Value** (`borg_town_capability.rs`): the scroll supplies FOOD again (#444
  had stripped it from every non-food kind). `NOURISH:INC_TO` fills to N
  percent (`effect_handler_NOURISH`, subtype 3; units: `food_value` 100 per
  percent, object.txt Remove Hunger 50), so read hungry (grade 15, 
  player_timed.txt) it adds 35: more than a Ration's `INC_BY` 30, not 50
  and not a flat ration. Applies to every INC_TO food (Honey-cake, Draught
  60 -> 45, Waybread 75 -> 60).
- **Food floor stays coherent**: under the town plan `borg_trait` now counts
  every scroll in `BI_FOOD` beside the rations (before: only with no other
  food), so planner and core count the same (`with_remove_hunger`). No
  store-sold food supplies more than 35, so the planner's floor (200
  nutrition) is never met with fewer than the core's five items. Not under
  the plan: unchanged.
- **Use first** (`borg_item_use.rs`, `borg_caution.rs`): under the plan a
  hungry or weak Borg tries the hunger spells, then reads the scroll, then
  eats. Blind, confused, forgetful or in the dark it cannot read and eats.
- Local, 180 starts, ambitious arm (4,2): Remove Hunger read 0 -> 47 notes,
  sold 33 -> 4, stored 130 -> 103 (89 games), rations eaten 65 -> 56,
  "You are getting hungry" 651 -> 512; deaths 32 -> 28, mean CL 4.73 -> 4.83,
  mean max depth 2.33 -> 2.46; restock-food loops 0 / 0; no stalls.
- **Left open:** the scroll is still bought once per game (155) as a
  flavour-learning purchase (the planner values unknown flavours, like Resist
  Heat/Cold) and stored when the food floor is already met by cheap Hard
  Biscuits (eleven carried in the sampled game). That is the planner doing
  its job on a spare scroll; fewer learning buys is #445's question.
- `scripts/borg-core-vendor.py`: `borg_trait.rs` is declared hooked (it
  already reported `borg_town_usefulness.rs` and `borg_town_utility.rs`
  from #444 and earlier; not declared here, not part of CI).
- `tools/shopping-audit.py` (from #445, `c2799e51`): `# Wearing/# Wielding`
  notes count as use, so torches are not reported as unused.


## 2026-11-16: an unchanged control game needs no new record (#464)

- Owner: relax RULES 12. A CI death is KNOWN when the Cemetery retains the
  same start (implementation, seed, race, class, flags, borg.txt, save) and
  the same key stream from any build; only a changed game is UNRECORDED.
- borg-cemetery #46 changes `reconcile-deaths.py` (and adds
  `--exact` for the old build-qualified match). `death-tools-pin.txt` moves
  to `586d0c13`: the old pin `1afd9423` plus only that change (branch
  `tools/relaxed-death-match-on-1afd942`). Cemetery main also carries
  `9b38f76` (canonical hash death IDs), which this repository's importer and
  engine checks do not support yet: adopting it is separate work. `--runs`
  (every CI death exported) is unchanged.
- Evidence: the native registry retained 22, 23 and 25 identical copies of
  the 1011, 102 and 1020 control games. On #455's merged head (source
  9fb9ad72) all four control deaths are KNOWN against main's pins before
  any new record (#44/#45 would not have been needed).
- Left: merge borg-cemetery #46 before or with this PR. The core parameter
  table is not part of the start (it changes when a parameter is added).
## 2026-11-16: outmatched healing flag (#457, borg-zoo#147)

- New parameters `caution_heal_when_outmatched` (0/1, default 0) and
  `caution_heal_outmatched_min_loss` (50 percent); registry 1783 to 1785.
  `borg_heal_outmatched` runs at the very end of `borg_caution` (after stairs,
  retreat, escape, back-away and the reserve/recall/unknown-item steps all
  declined, so escapes keep precedence) and, when hurt by the threshold with
  danger at least current HP, quaffs CLW, else CSW, else CCW instead of
  letting `borg_attack` run. Note: "# Healing (outmatched)." The potion order
  is weakest first (conserves stronger cures); the owner may prefer strongest
  first for the study.
- Flag 0 is byte-identical on this build: seed 1516096046 (race 4 class 2,
  errands_plan.txt, town 1, trip 0, 100000 turns) still dies at CL3 with keys
  `4c2ea4baf255`; seed 501 (race 0 class 0, ambitious plan) keys `a80b9ac6185c`.
- Flag 1, same cells: 1516096046 quaffs CLW twice in the acolyte fight and
  survives to the turn limit (CL4, max DL2); 979002123 (race 8 class 6,
  trip 1) drinks 17 times and survives to the turn limit (CL10, max DL13)
  where flag 0 dies (CL8, DL12). One cell each, not a measurement (RULES 14):
  the zoo control (flag 0 against 1 on the #147 arms) is the next step.
- Unit tests cover the pure decision; the potion path needs the engine's
  `z_info` and is exercised by the replays above.

## 2026-11-16: back-away no longer steps onto lava (#458)

- `back_away` (borg_caution.rs, "# Backing up to x,y") accepted any
  `borg_cave_floor_grid`, which includes `FEAT_LAVA`. Lava burns 100+1d100
  per turn (engine player-util.c:918), so low-level Borgs died at full HP in
  town (borg-zoo#147: 15 of 212 death cells). It now skips lava unless the
  Borg is fire-immune, the rule already used in borg_caution's
  flee-grid check, flow_dark, flow and rest. Plain fix, no flag.
- Audit of the other `borg_cave_floor_grid` callers: none else chooses a
  step. The rest are projection and ray paths (borg_projection, fight_attack,
  fight_defend), monster tracking (flow_kill), light and surround counts,
  trap assumption (update), and a search-target scan (flow_misc) whose cost
  map already blocks lava. Dimension door excludes lava already. Flow
  spreading blocks lava for a non-immune Borg. Phase door lands wherever the
  game puts the Borg; that is not a Borg choice. `borg_cave_floor_grid` is
  unchanged (map/view code depends on lava being a floor).
- Replays (errands_plan, town 1, trip 0, 100000 turns): 1356248624 race 2
  class 0 now survives to the turn limit (CL5); 584077430 race 1 class 3 now
  dies at turn 73713 to a battle-scarred veteran, not lava. Sanity cell 501
  with the ambitious plan keeps its keys sha256 (a80b9ac6...).
- The unit test covers the lava rule; `back_away` itself links only with the
  engine, so the replays are the integration check.

## 2026-11-16: reviewed preference promotion synchronization (#453)

- Owner asks to merge after #407's review at50c92ba0 reports no blockers.
  Prepare the combined build through a track-p task PR before promotion.
- Merge accepted main6d1bca10 into track-p50c92ba0. Preserve both branches'
  handover/provenance additions; those are the only conflicts. Main's newer
  throwable supply helper and all accepted donor/planner/survival/startup
  changes remain intact. The reviewed final-ranking seam merges unchanged.
- Review findings1/2: document core-trace@2 and the attacks array; explain
  fixed-neutral waiting against downweighted attacks. Confirm the documented
  invocation-family semantics without changing weights or mapping.
- The vendor audit exposes an existing accepted-main ownership omission:
  #400/#443 changed borg_town_usefulness.rs without declaring it HOOKED.
  Declare that receiving file; retain its accepted bytes without edits.
- Local formatting/Clippy,165 workspace checks,333 native checks/four ignores
  and the vendor audit pass. Candidate full45 and real weighted Mage/trace
  parity checks pass. Cemetery0f9f7cf7 independently retains98 verified
  packages, including this combined source290fd89e and initial CI failure.
  Pin it for fresh CI. Compare the full45 with accepted main separately. Earlier exact-build evidence remains historical;
  green old task CI does not validate this combined build. Repin #407 only
  after task CI/integration; promotion must preserve accepted main work.

## 2026-11-16: attack preference review follow-up (#424)

- #405 merged into track-p at 0edee14b; #403 is closed. Its post-merge
  review of 3f744ddb found no correctness bugs or blocking findings.
- Owner requested the suggested small follow-up (findings 1 and 6). Retain
  the winning simulation's raw utility for the diagnostic note. When it
  differs from the weighted utility, label both; otherwise preserve the
  existing note exactly, including at all default weights. No helper is
  rerun, and ranking, execution, RNG and parameter values are unchanged.
- Correct PLAN.md's stale task status. Other minor review observations
  remain nonblocking; invocation-family weighting includes emergency and
  control actions as documented. Any non-neutral tuning needs that scope
  considered explicitly; no default tuning is proposed here.
- Local formatting/Clippy,165 workspace tests,336 core tests/four ignores and
  vendor audit pass. Neutral Mage keys/messages/state exactly match pre-fix;
  weighted logging and trace-on/off equality pass; original Fang keys match.
  Source-qualified controls and the initial CI reconciliation failure are
  retained in Cemeteryf2e4b38c, class-attack-preferences/review-followup-424.
  The second CI passed full engine behavior/reference checks; final original-
  core reconciliation also needed the Druid seed1020 identity. Its independent
  local control matches exactly; all86 archive packages verify before fresh CI.
  Pin that independently retained archive before fresh CI; never weaken the
  reconciliation check or relabel earlier source identities.
- Required Rust checks and task CI gate integration into track-p. Preserve
  the original review pin; update #407 to the integrated follow-up pin after
  green CI. Main promotion still requires owner acceptance and delegation;
  R carry #406 remains gated on accepted promotion.
- Read-only source refresh ran; missing moving-source clones and unresolved
  origin/HEAD refs prevent a complete upstream report. No source pins move.

## 2026-11-16: preference task synchronization with recorder review (#403)

- Track-p advanced from cd411e55 to 46d5f2ae through #393 while preference
  validation ran. Preserve the complete recorder v0.2.0 adoption and both
  handover histories; only this document conflicted. No recorder code or pin
  is edited by the preference task beyond inheriting that integration commit.
- #407 now freezes track-p for the recorder milestone review. #405 remains
  unmerged while that review is active; owner authorization to implement
  preferences does not authorize moving its reviewed head or promotion to main.
- Fresh clean25e89494 versus clean46d5f2ae passes strict full45/45 native
  key/message/state parity.336 native tests (four existing ignored),165 workspace
  tests, formatting/Clippy and70 engine checks (two local skips, required pinned
  zoo and incident fixtures present) pass. The real weighted choice and tracing
  checks still pass; original-core Fang replay reproduces50491fa13924 exactly.
- Current Cemetery evidence pin:62039b2750d3b106062d5246e77fa3a84d60d1d8, recorder-sync/ under
  validation/borg-reborn-rs/class-attack-preferences. All73 qualified packages
  verify, including27 new exact-build records;12 current native neutral deaths,
  the weighted case and two original-core deaths reconcile. The new fixture
  reference supplies CI's known archive without weakening any check. Earlier
  cac37f59/cd411e55 evidence below remains historical, not relabeled.
- R carry remains separately blocked in #406. Native calibration replay tooling
  is Cemetery#13; #407's reviewed track-p head remains unchanged.

## 2026-11-16: class/family attack preferences (#403, design #396)

- Owner authorized native P implementation after the proposal discussion. Branch
  task/T403-class-attack-preferences starts from track-p cd411e55; task PR targets
  track-p only. Main promotion and the later R carry need their normal gates.
- Add 54 named neutral-default parameters (nine classes, six invocation families),
  explicit symbolic mapping of all170 BF entries, widened floor arithmetic and
  one final-ranking seam. Hybrids/controls retain invocation family; emergency
  eligibility and all helper internals remain unchanged. No danger correction.
- core-trace@2 adds every evaluated candidate's raw/adjusted score, parameter and
  provenance plus per-invocation winner. Nonpositive results carry no invented
  rejection cause. Nested frames/repeated attempts remain separate; tracing
  does not rerun simulation or feed decisions. Historical @1 evidence is intact.
- Native arithmetic/mapping/config/ranking/nesting tests, workspace tests,
  formatting, Clippy, vendor audit and Python tools/scripts pass locally.
  A bounded real Mage regression changes the selected attack under a non-neutral
  profile and proves trace-on/off keys/messages/states equality. Its unit fixtures
  for other classes are not claimed as native replay coverage.
- Clean cac37f59 versus clean cd411e55 passes strict full45/45 key/message/state
  parity.336 native tests (four existing ignored),165 workspace tests,70 engine
  tests (three skipped),13 mandatory pinned zoo-profile contracts and the existing
  four-save incident regression pass. Other checks above remain green. Initial
  baseline/candidate full attempts played all45 games but could not finalize the
  suite marker because bc was absent; preserve those attempts. Install tools
  locally and repeat unchanged suites, never forge completion or discard deaths.
- Cemetery evidence is pinned ate6a3698c113e9b6bf7ebec87bed8a5d30196a62a under
  validation/borg-reborn-rs/class-attack-preferences. It retains46 qualified
  packages, original records, the initial CI artifact, all failure receipts and
  full comparison hashes. The second CI passed all gameplay comparisons but
  needed the reference-identical seed102 Fang smoke dossier under its new source
  identity; both CI artifact archives are retained.12 final neutral deaths and the paired non-neutral case
  reconcile. The dedicated harness/fixtures/attack-preferences.json pin feeds CI's
  known registry; the original fixture/tool pins and historical archive stay fixed.
  Native preference test deaths use a separate uploaded/reconciled registry, so
  the existing original-core observation replay continues to select its own
  core's case. Shared rerun-death lacks calibration-override restoration; that
  pre-existing tooling limit is not silently treated as a native replay check.
  Local raw evidence is under /workspace/work/403-*. R must
  independently consume the accepted promoted contract with neutral/non-neutral
  check/switch, ordered requests/effects/RNG and restoration (#364/#366).

## 2026-11-16: town planner item values (#400, #443, food)

One PR, owner's request (supersedes #441):
- **#400** learned spells no longer zero an item's demand (all services,
  not only Phase Door).
- **Food counted like the core**: only food and mushrooms supply FOOD
  (`BI_FOOD`, which the core's readiness counts). A Remove Hunger scroll
  counted as food let the planner sell the starting rations; the core
  then bounced DL1/town on "restock food < 3" in 27-31 of 90 town-arm
  games. The core seldom reads the scroll (6 of 253 bought).
- **Food floor**: below five rations' nutrition the FOOD curve is a capped
  requirement (borg_prepared "5 Food"). Without it the loop stays in 17 of
  90 games. Nutrition, not items: a Mage may buy cheap Hard Biscuits.
- **#443 worthless kinds**: 45 consumables the core never uses on purpose
  (source audit; foods and use-up-is-benefit potions excluded) get no
  planner value until the core learns to use them.
- **Flavour learning** (`borg_town_w_knowledge`) is 0 with the trip policy,
  which returns to town to sell unknown items.
- **Wine churn fixed**: drinks with under 10 nutrition (wine, whisky,
  liquor, miruvor) are not food to the planner; counted, the floor bought
  wine then stored it at the Home every visit (1,532 stores in 180 games
  -> 42).
- **Sell less** moved to branch `task/sell-policy` (owner): a parameter
  `town_sell_without_gold`, still under investigation (town arm without a
  plan: deaths 30 -> 44, deeper dives). Next study: 2x2 of depth caps
  (errands_plan / ambitious, max-depth) x returns (trip policy 0/1).
- Local, town arm, 180 starts (classes-1800 1-180): deaths main 34 /
  this 30; caster deaths 19 / 19; restock-food loops 58 / 0. Starts 1-90
  alone showed caster deaths 5 -> 12, starts 91-180 14 -> 7: noise.

## 2026-11-16: learned spells no longer zero item value (#400)

- Owner: fix all of #400 in one PR, the way #380 fixed Phase Door.
  borg_town_capability.rs no longer sets a capability's demand to 0 when
  the character has a reliable spell for it (TRAPS, MONSTERS, INVISIBLE,
  MAP, ILLUMINATE, DIG, RECALL, FOOD). Mana is finite, spells fail or are
  unusable, and the broad classes conflate effects (treasure sensing is
  not mapping). The demand drives buying and keeping, so this also stops
  casters selling those items.
- `effect_supply` is test-only now; `engine::spell_effect_values` keeps an
  `#[allow(dead_code)]` hook (donor code, no caller).
- test_phase_stock: on the Mage town fixture TRAPS, MONSTERS and
  ILLUMINATE were 0 (Find Traps, Detect Monsters, Light Room); now every
  service keeps its demand. The assertion fails on 4ecb93d7.

## 2026-11-16: owner-authorized synthetic P1 landing (#378)

- Owner explicitly requested merging #378 after the independent milestone
  review/re-review and coordinated fixes. Cemetery #7 landed first at
  `0360b72cfa93c84ab41d3192b9bad3cc38e3ccaf` from reviewed/fixed `ec332c6`.
- Synchronize accepted main `4ecb93d7` into the #378 feature branch. Keep both
  handover histories, native recorder v0.2.0 and accepted #369/#380 survival
  changes. Harness conflict resolution retains BOTH synthetic receipt/build
  checks and the accepted errands-plan copy/load identity check.
- The canonical fixture/evidence checkout points to Cemetery `ec332c6`, which
  includes the unchanged incident save bundle and both parent control sets.
  The native-tool pin remains `1afd9423`; its byte verification stays enabled.
- Integrated clean runtime is `88a9437a7cbc96ceb7de16c1eecd9cc49c8b4bfb`,
  source `2d1420b704fa8faa4a1972fe9548fdd2ebaa2a5fde98c9ff7ce9f4ba51d2ecef`.
  Formatting, Clippy, workspace/core tests, 94 tools and 19 scripts pass;
  86 local engine tests pass with 18 existing fixture/dependency skips.
  Initial native checks rejected a stale API-1 sibling checkout as designed;
  after checking out the landed Cemetery code, rerun passes without test edits.
  Ordinary/exact smoke match stored references 3/3 across all streams.
- Three natural controls for the integrated source are retained immutably in
  Cemetery PR #20 at `9283d07c77724326970e18759d886cd15efd7d29`; all 82 scoped
  packages verify and all 869 prior package blobs remain intact. The source
  fixture now points to that canonical evidence, keeping the original save
  bundle/hash unchanged. No new validation dossier is added to this source
  repository. Final integrated-head CI must pass before #378 merges. Reviewed P1 runtime/bank
  evidence remains pinned to its original runtime; no new survival or
  population claim follows from integration. Per-kind charges (#411) and
  optional one-way policy (#404) remain separate follow-ups.

## 2026-11-16: coordinated M2 review boundary fix (#378 / Cemetery #7)

- Owner requested coordination with the Cemetery session. Work is isolated on
  `task/371-m2-forwarder-review`, stacked on reviewed #378 head `34cec1fc`.
  No direct pushes to the shared #378/Cemetery branches and no native-tool pin
  change. Ownership proposal is on Cemetery #7 comment 6913007931 and reborn
  #378 comment 6913031259; Cemetery owns merge/index regeneration, retained
  stream checks, process cleanup and packet disclosure. Acknowledgement and a
  fixed new Cemetery head are still needed before paired integration updates.
- Reborn review finding 3: all native tool execution now requires file bytes
  matching death-tools-pin.txt, including outside CI. Docs/evidence-only heads
  with identical native code work; stale API-1 guards, changed/missing files,
  missing pin objects and added source fail before executing tool code.
- Seven pointer tests and all 94 tool tests pass. The reviewer's stale Cemetery
  main `5abfdbe` is refused before executing export against a real retained
  synthetic death; pinned `1afd9423` correctly rejects it as synthetic. No
  scratch natural-registry directory is created. No new game was run.
  Source/script tests: 19 pass. Engine input digest remains the reviewed
  `bf9a451c46a07142b59525a9c9c581287930e3b333baa4a81dda4816ed2a025a`;
  there is no new runtime/bank identity. Fix tracking is #410; low-priority
  per-kind charge validation is explicitly tracked in #411, without altering
  the reviewed observed tier.
- Finding 2 is explicitly disclosed in harness/README.md: the P1 birth kit is
  an ordinary kit created before one-way options, retaining the option-
  conditional Recall scroll that a native one-way Warrior birth omits. The
  existing bank's ineffective consumption is disclosed, not a valid escape.
  Runtime/capture/catalog/bank bytes remain unchanged; no new gameplay report
  belongs in this repository. This is a review response, not acceptance.

## 2026-11-16: synthetic P1 evidence ownership synchronization

- Preserve current main's accepted ownership/CI changes. All six unmerged
  task-control packages and the two gameplay evidence packets are now in
  borg-cemetery, byte-for-byte verified in its M2 ownership receipt. No
  accepted historical source record or published Git history is rewritten.
- Source gameplay packet paths are immutable Cemetery references. The
  fixture/evidence checkout pin includes canonical controls and the unchanged
  incident save bundle, so CI checks against retained evidence rather than
  itself. Source-input hash remains 8a61938a; the runtime code did not change.
- Canonical milestone packet: borg-cemetery docs/reviews/M2/README.md;
  exact paired heads and cloud checks are in the PR descriptions. M1 and P/R
  acceptance gates remain independent; no milestone is marked accepted here.

## 2026-11-16: continued synthetic P1 milestone (#371 / cemetery #5)

- Owner instructed continued implementation with review at the milestone;
  the original generated-depth PR is an integration checkpoint, not a task
  review stop. Cemetery PLAN.md selects M2 / synthetic P1 independently of M1.
- Added a restricted declarative Human Warrior tier: real measured capture,
  engine SHA256, explicit native item/state validation, full-health normalization,
  and verified initial level/max HP/SP. Capture preserves the ordinary key/state/
  message streams; no save or Borg memory becomes a synthetic input.
- OpenSSL crypto is a build dependency for the native file digest. Verified generated-cave starts use contract @2 for both variants; @1 is
  superseded because requested player depth did not prove cave generation. The pinned capability declaration
  explicitly limits supported native state; no unsupported item is dropped.
- The #378 owner-supplied review led to a direct cave/pending-generation
  check and exposed the initial startup bug. Complete the native transition
  before either Borg activates; retain old results as superseded, not valid
  generated-cave evidence. Both cores receive repeated real startup checks.
- Narrow core birth-option authorization and the guarded lifecycle wrapper
  follow #386 at its recorded provenance pin; no strategy/tactics are copied.
  These are competing primitive implementations: do not merge both blindly.
- Synthetic one-way save reload is refused. Cemetery additionally excludes
  unmarked one-way native options from natural exports. Generic refusal-message
  counts are reported separately from terminal outcomes and hazard qualification.
- Source snapshots and exploratory attempts remain outside Git. The final P1
  catalog/suite must use a clean, pinned build, with source/binary/config/log
  hashes, declared normalization and a single-observation sampling limitation.
- Cemetery owns generation, isolated adapter execution, independent terminal and
  validity outcomes, provisional low-HP screens and reports. Qualified null-move
  counts remain unavailable in P1; wider catalogs/qualification are P2.

## 2026-11-16: synthetic-start prerequisite (#371, owner-directed first slice)

- Branch `task/371-synthetic-start` starts at main `92404c7e`; this is an
  execution/harness task, not a P calibration or R port milestone.
- `start-depth=N` / `-start-depth N` generates a birth-kit start without a
  save, rejects bad/conflicting requests, verifies the first logged depth,
  and records `borg-reborn-rs/synthetic-start@1` in summary/config.
- Semantics are native one-way descent: no ascent/recall, further descent
  allowed. This deliberately resolves the epic's conflicting single-level
  wording; it must be assessed in the owner's review.
- Harness skips synthetic death export; the exact cemetery toolkit pin
  supplies an independent direct-export and reconciliation guard.
- Review scope and runnable checks: `docs/reviews/SYNTHETIC-START.md`.
  Leave the implementation unmerged for the owner-requested review.
- Initial CI exposed missing source-specific ordinary control dossiers,
  not a synthetic-start failure. Retained three immutable natural validation
  packages using `tools/import-deaths.py`; 2,556 packages verify and both
  ordinary/core controls reconcile. No synthetic dossier was imported.
- #371 remains open: run-derived build application and independent
  null-move diagnostics are follow-up slices; cemetery #5's suite remains
  outside this change. No population/improvement claim is made.
## 2026-11-16: owner-approved recorder-only promotion

- Owner accepts the recorder review at46d5f2ae and explicitly requests merge.
  Concurrent #405 integration expanded track-p/#407 beyond that review.
  Promote the preserved recorder snapshot separately; leave attack preferences
  on track-p for their combined review. No preference policy is included here.
- Preserve accepted main8a0b57c8, including #354 harness work and #368 core
  refresh. Retain both handover histories and main's canonical evidence pin;
  fixture archive bytes/hashes are unchanged. Revalidate this combined tree.
- Original recorder implementation and vendor bytes remain exactly46d5f2ae.
  Its review provenance and nonblocking follow-ups (#409) remain intact.
- Clean combined build71978c7b/source1573e6ca: native1011, smoke102 and
  original-core1020 controls match accepted-main retained key hashes and
  terminal facts. Pin canonical sync receipts at Cemetery8d63abf5. Local
  91 tool/19 script checks and all2572 historical package checks pass.
  Combined CI gates the delegated recorder-only merge.

## 2026-11-16: recorder integration after accepted importer review (#351)

- #395 is merged. Sync current track-p cd411e55, retaining both histories
  and the accepted Cemetery ownership/fixture changes. Gameplay source remains
  7d2af920, identical to the measured current-base recorder task build.
- Cemetery PR11 preserves all 54 task packets and eight reports byte-for-byte
  (602 audited files). The source archive returns to the accepted 2,572
  historical cases; canonical native controls total64. Pin that evidence for
  CI reconciliation and retain brief source links in P351-VALIDATION.md/.json.
  Older entries below describe their original snapshots and paths.
- After green final CI, merge #393 only into track-p and close #351. Open
  the exact-pin track-p -> main recorder milestone review; the owner chooses
  the reviewer and accepts promotion. Do not merge that promotion here.

## 2026-11-16: recorder task current-base synchronization (#351)

- Track-p advanced to 79ec0591 during recorder work, including the owner-
  directed #373/#379 policy revert and #358/#359 importer fixes. Preserve
  that source and both histories. Only this handover conflicts. Keep
  #395's review head fixed; #393 remains a separate draft.
- Preserve the first completed comparison as pre-revert proof under
  P351-PRE-REVERT-* and freeze its actual three executable images. Do not
  relabel its #355-era policy or overwrite recordings/build identities.
- Refreshed current-base checks pass: all 45 play triples and deterministic
  summaries, 242,641 classified rows, fresh/loaded v2 replay and saves, 69
  native tests (one local binding skip), 91 tool/19 script tests and required
  Rust checks. Actual clean build receipts are in P351-VALIDATION.md/.json.
- Preserve 26 current-source packets as well as 28 first-epoch packets;
  both registries reconcile and all 2,632 archive cases verify. #393 needs
  CI and #395 disposition before integration; its V2 pin/consumer hooks
  remain confined to the task branch while that reviewed head is fixed.

## 2026-11-16: recorder v0.2.0 consumer adoption (#351)

- #372 is already owner-promoted; continue on task/P351-recorder-v020.
  Pin the exact v0.2.0 tag ee8b133a and vendor all 41 files verbatim.
  No recorder implementation changes are made in this repository.
- Regenerate consumer hooks from the actual patched tree: early startup,
  file-newer, directory enumeration and live save-path existence. Keep
  recorder-off builds off and preserve existing harness/snapshot hooks.
- Document the v1/v2 incompatibility and retain all old sessions/identities.
  Add captured-load replay/final-save coverage alongside fresh-game capture,
  forked checkpoints, bot streams and v2 CLI verification. GUI/branch-menu
  and migration coverage are not claimed.
- Local validation passes: full 45/45 recorder-off play parity and 241,403
  classified rows, original smoke/death controls, fresh/loaded native v2
  replay/final saves/checkpoints, 18 recorder Rust checks and required Rust
  checks. Native 69-test run had two local skips; the pinned incident test
  rerun passes all four positions, leaving local binding regeneration for CI.
- Preserve 28 new provenance packets with exact original bytes; both
  registries reconcile and all 2,606 archive packages verify. Build/log/hash
  receipts and limitations are in P351-VALIDATION.md/.json and the map.
  Required CI gates track-p; a separate owner-selected review gates main.
## 2026-11-16: plan phases become depth caps (`max-depth N`)

- Owner: a phase should be an additional check, not forced behaviour.
  New plan command `max-depth N` (borg-core/src/errands_plan.rs): no
  errand; the core plays as itself and is not prepared below DL N (the
  phase supplies a separate N+1 threshold, combined with the live configured
  value). Review follow-up #447 replaces the cfg snapshot/restore: reload
  and same-save reinitialization retain the overlay; cancellation releases
  it immediately, including during Recall. No stale cfg is restored.
  Phase changes preserve errands and core goals; parser/dive/trip tests
  removed with loop-only coverage are restored.
- Owner: remove the loop errand entirely. `/loop-levels` and the plan
  command `loop-levels` are gone (a plan using it exits 2 with "loop-levels
  was removed; use max-depth N"), with the level-loop code in shell.rs,
  including #421's hold, test_loop_levels.py and test_loop_livelock.py.
  #421 cannot recur: nothing forces a route any more.
- Both shipped plans rewritten: `loop-levels A B` -> `max-depth B` (new
  hashes).
- The core's own cap lets a "spent too long, leaving" move step onto the
  level below for one move and climb straight back; owner: not an issue.
- The published #440 90-start comparison is historical, descriptive and
  not independently verified here. Raw seed/config/run custody is still
  being recovered; it is not validation of #447. See the canonical
  [Cemetery archive](https://github.com/diegoami/borg-cemetery/tree/513d14f59a00fee6b6a5fa39c650d04eaea05650/validation/borg-reborn-rs/plan-cap-review-447/);
  missing raw custody is tracked in #448.
- CI deaths for reviewed source ef85f725 are retained on borg-cemetery
  `evidence/borg-reborn-rs-max-depth-ci-controls-v2`; its 77dd467e (after the review fix) also
  adds `checkpoints-max-depth.zip`, the wild-cat archive with the plan
  capped (saves and borg.txt unchanged; the original stays as evidence).

## 2026-11-16: level-loop stair livelock (#421)

- borg-zoo's 3x3 pilot and small tier found frozen-turn bursts with the
  ambitious plan and the live town planner: on DL1 the core refuses DL2
  for a reason town cannot change (e.g. "30 hp"), the loop goes home, and
  in town it restarts and takes the stairs straight down, with no game
  time passing. Still on main 4ecb93d7 (Half-Troll Rogue 926210318: 1,098
  keys at one game turn).
- Fix (borg-core/src/shell.rs, ours): the loop remembers the refusal that
  sent it home (level, reason, max character level). The same refusal
  again, with no descent in between, holds the loop: the core plays the
  level it is prepared for, and leaves a used-up level its own way. A
  descent clears it. The note now names the core's reason.
- Checks: engine/tests/test_loop_livelock.py (fails on main with 1,098
  keys at turn 48483, passes here). 90 small-tier starts, ambitious-town:
  stalls 1 -> 0, deaths 23 -> 17, max burst 725 (one-unit arrow buying).
  No-plan arms (control, town) byte-identical on 8 cells.
- Synchronized with main 83da6679 (#378; play unchanged on checked cells).
  CI deaths for source 6b3b9e99 are retained on borg-cemetery
  `evidence/borg-reborn-rs-421-ci-controls-v2` (da51ee44, based on
  9283d07); the wild-cat fixture points CI at that commit.
- Found while auditing (next PR): every phase change starts with a town
  trip (`loop-levels 0 N`), arrival in town restarts the loop, and any
  upward move sets "home" (46 such overrides in one Warrior game).

## 2026-11-16: narrow survival fixes together on main (#369, #380)

- Owner: both narrow proposals have the highest priority, in one PR for
  review. Carries #392's flee-healing change (borg_caution.rs: fleeing no
  longer skips borg_heal() merely because HP cannot absorb another round;
  borg_heal keeps its own wound/level/resource gates) and #390's Phase
  Door stock change (borg_town_capability.rs: a learned Phase Door no
  longer zeroes the scroll's supply value; HOOKED). Both rebased onto
  main 8837ece2 (borg-rs 8c5d000, recorder v0.2.0) without other changes.
- Source hash 9a4ea375 → 2317021c. Wild-cat saved-position tests: heals
  through the fight; with CLW/CSW tiers disabled still dies (control).
  Phase stock: Mage fixture buys a Scroll of Phase Door.
- CI deaths for the new source (core 1011, smoke 102, reborn 1020; the
  fixes do not save them) are retained in borg-cemetery
  `evidence/borg-reborn-rs-369-380-ci-controls` (2f9d823); the wild-cat
  fixture points CI at that commit. The owner merges that cemetery PR.
- Supersedes #392 and #390 (close them on merge). The zoo needs a new ref.

## 2026-11-16: borg-core refreshed to borg-rs 8c5d000 (#368)

- Owner request: carry borg-rs#92 (fix for borg-rs#90, the Home oil
  store/take livelock) and borg-rs#88 (additive trip policy). Three-way
  refresh from 1443c2d, 0 conflicts; vendor check passes. harness/borg.txt
  gets #88's trip comments only (settings unchanged).
- Source hash 87717140 → 9a4ea375; harness/borg.txt 88296dd1.
- Donor parity: 12/12 cells (seeds 501–506, classes 0–5, 30k turns,
  town/trip off and on) byte-identical keys against a fresh borg-rs build
  at 8c5d000.
- Off arms unchanged: with borg_town_plan = 0 and borg_trip_policy = 0,
  12/12 cells (no plan and errands_plan.txt) are byte-identical to main
  1c4b351b; smoke 3/3 identical to the reference. All shop cells change,
  as #88 intends.
- Oil case (Warrior seed 501, ambitious plan + shop, 100k): keys 69,838 →
  15,209, Home takes 3,994 → 0, worst burst 719 → 12 actions at one turn;
  keys sha256 f32abbde21da.
- CI death controls for the new source (core seed 1011 Mage, smoke seed
  102 Mage, reborn seed
  1020 Druid; same keys as before) are retained in borg-cemetery
  `evidence/borg-reborn-rs-368-ci-controls` (696065b, PR borg-cemetery#16);
  harness/fixtures/planmage-wild-cat.json points CI at that commit.
  The owner merges borg-cemetery#16.
- Left: a store/take across visits is still possible (costs game time);
  no no-progress detector. The zoo needs the new ref (borg-zoo#123).

## 2026-11-16: #354 synchronized with main 72a21620, without #355

- Owner decision: the zoo's five-arm specimen carries current main's game,
  not #355. #355 (7022017e) was reverted on main by #379 (#373), so the
  game inputs are the pre-355 ones again (source hash 87717140, as at
  2f74e1c5) and this branch adds only plan files, harness plan runs and
  the zoo contract. Plan hashes unchanged: errands_plan.txt 922372ba,
  errands_plan_ambitious.txt 9d7d8a39. The 72-game local pilot still
  applies to this game. borg-rs#90 (oil store/take loop) is in the
  vendored core and still present.

## 2026-11-16: reviewed importer sync after Cemetery ownership migration (#398)

- #395 review at 79ec0591 reports no blockers. Its remaining engine CI
  condition is satisfied by green run 43783576729. Nonblocking mode,
  fault-injection, Python-minimum and formatting observations are tracked
  in #397; none is implemented in this synchronization.
- Main advanced to 6c89d9a2 through #394. Preserve the accepted Cemetery
  ownership migration, six mapped package removals and pinned CI evidence.
  Resolve only the handover conflict by retaining both histories.
- Reviewed importer, importer tests and DOSSIER-IMPORT.md stay byte-identical
  to 79ec0591. Runtime, source pins and archive contents match accepted main.
  Required CI on this combined tree must pass before track-p integration.
- Update #395 with the new pin and exact scope/tree proof; retain the
  original review's provenance. Promotion remains pending owner acceptance
  and explicit delegation, not merely the reviewer's recommendation.

## 2026-11-16: Cemetery evidence ownership (#376)

- AGENTS/RULES now require gameplay reports, scenarios and trials in
  borg-cemetery; source PRs retain code/tests and concise validation links.
- Cemetery PR8 retains the PlanMage incident and all66 files of the six
  #355 control packages byte-for-byte. Remove only those mapped packages
  here, regenerate the index and redirect the current incident pages.
- CI reads the unchanged checkpoint archive from a separate pinned Cemetery
  evidence checkout. Reconciliation uses retained historical plus canonical
  cases, never the new temporary registry as its own expected baseline.
- Original public player recordings/hashes and other sessions' historical
  packages remain preserved. No policy change is part of this cleanup.


## 2026-11-16: importer follow-up main sync after #379

- Main advanced to58b11ff2 through owner-directed #379 while #381/#384
  were in CI. Synchronize it through a separate task PR into track-p;
  retain the pre-355 behavior and both handover histories without edits to
  accepted runtime code. The merge is conflict-free.
- Importer mode and failed-publication cleanup follow-ups remain intact;
  runtime inputs must equal this accepted main and all2,578 archived
  packages remain unchanged. No new gameplay or survival claim.
- CI is required for the synchronized tree. After integration, open the
  importer follow-up batch's pinned track-p -> main review and stop;
  prior #372 acceptance does not cover the new changes.

## 2026-11-16: failed import publication cleanup (#359)

- Stacked on #358/#381. Restore0700 only on this invocation's unpublished
  staging before removal, even if publication adopted a read-only source's
  mode. Never scan other importers' staging or chmod an existing package.
- Keep the original publication exception if cleanup also fails, attaching
  the cleanup error as a diagnostic. Reproduce both paths with injected
  rename/removal failures; assert read-only source mode, private cleanup,
  existing package bytes, another active staging directory and no final
  partial package. No evidence, runtime, defaults or pin changes.
- Task PR records local/CI checks; integrate into track-p after green CI,
  then stop for the next pinned owner-selected promotion review.

## 2026-11-16: published import directory access (#358)

- Owner accepted #372 and delegated its merge; the unchanged reviewed pin
  cf687bb2 is promoted as b7d45f17. Review follow-ups #374 (exact invocation
  receipts) and #375 (reference-receipt assessment) remain separate. The
  reported missing log hashes already exist for all 45 pairs in the pin.
- Bound newly published directory modes by source access and directory
  creation under the importing process's umask. Obtain creation permissions
  with an empty directory inside private staging, without changing the
  process-global umask; remove the probe before publishing. Drop special
  bits. Source0777/umask022 now gives0755; source0755/umask077 gives0700.
  An intentionally permissive umask000 still permits a source0777 directory.
- The mode matrix covers four umasks and five source modes (including
  special bits), private staging during every write, file/generated-metadata
  access, unchanged source modes, exact package bytes/IDs and re-import modes.
  #359 still owns failed-publication cleanup for read-only staging.
- No runtime, default, pin, immutable package or index changes. Source-refresh
  report ran read-only; this isolated worktree lacks six moving-source clones,
  so it cannot report their new commits. This narrow historical-import review
  follow-up neither updates pins nor claims a refreshed gameplay baseline.
- Local/CI validation is recorded in the task PR. Task targets track-p;
  a fresh pinned owner-selected review must precede promotion to main.

## 2026-11-15: fresh exact-tag M13 adoption audit (#341)

- #349 was owner-reviewed, synchronized through green #362 and promoted as
  92404c7e. Track P's M13 source adoption was already integrated by #331;
  this task supplies its remaining fresh donor build/run evidence.
- Rebuilt exact donor 1443c2d1 and accepted combined-P adoption 2f74e1c5 from
  clean separate worktrees. Actual source/binary/build/config receipts are
  retained, without relabelling older dirty measurements. Full 45/45 play
  triples and effective configs match, with 242,641 contiguous classified
  trace rows and no unclassified rows. All calibration params are default.
- Historical adoption parity is not claimed for later #355 policy source
  0f2e003f. Current production code remains unchanged; its intentional six-
  file safety delta and existing scenario evidence are explicitly separated.
  See P341-FRESH-DONOR-VALIDATION.md/.json.
- Three policy-on pairs match; 656 schema11 plans replay (500 with a plan).
  328 core tests (four existing ignored), 13 isolation, 4 shopping, 3 loop tests
  and original smoke 3/3 pass. Initial fixture harness failures from missing
  discoverable cemetery tools are retained; unchanged reruns pass after
  adding the ignored pointer to the exact tooling pin. No assertion changed.
- All 25 fresh natural-death packages are qualified in P341-DOSSIER-MAP.json;
  original records/payloads remain exact. All 2,578 archive packages verify,
  and both receiving 13 and donor 12 deaths reconcile with none missing.
  Repeated/paired controls are not additional independent population lives.
- No game, configuration/default, source pin, Zoo specimen/run or canonical
  cemetery content changes. Raw builds/attempts remain under the named
  /workspace/work/p341-* paths. Task PR targets track-p and requires green
  CI; a new pinned owner-selected milestone review precedes promotion.
  #358/#359 remain separate permission/cleanup follow-ups.

## 2026-11-15: reviewed P349 synchronization (#360)

- Owner review at 1a95a0a6 accepts #349 with follow-ups and no blockers.
  The review reports 88 tool tests, script tests, all 2,547 archive cases
  and real mode/umask imports. Required CI and P342 play evidence were
  taken on trust; do not describe the later combined head as independently
  re-reviewed. The accepted runtime changes in main remain separate evidence.
- Merge main 7022017e (#355 safety fix) into track-p, retaining both handover
  sections. This is the sole merge conflict. Runtime inputs and archive
  packets remain byte-identical to that accepted main; the reviewed P
  importer, tests, ignore rule and guidance remain intact.
- #358 tracks source/umask bounds for published directories (including the
  world-writable source case); #359 tracks read-only staging cleanup after
  failed rename. Both are nonblocking follow-ups after this promotion.
  No follow-up implementation is slipped into the reviewed pin.
- Local checks pass: 88 tool tests, 19 source-tool tests, all 2,553
  archive packages, vendor audit, formatting and Clippy. Required Rust
  tests and sync CI must pass before integration and promotion.
  #341's fresh-donor baseline audit follows this gate; it must distinguish
  historical M13 parity from #355's intentional later policy changes.


## 2026-11-16: owner-requested rollback of #355 behavior (#373)

- Restore the six native-core files to accepted pre-#355 main 2f74e1c5.
  This rollback contains no dossier/report payloads; preserved evidence and
  canonical Cemetery migration are separate. The saved-position tests now
  identify the restored death control honestly. Narrow replacements will be
  separate owner-reviewed changes before any zoo dispatch.

## 2026-11-15: owner-directed wild-cat survival fix (#350)

- The original PlanMage GUI recording was replayed with its frozen binary;
  all streams, final save and complete core trace verify. Native saves at
  411638/648/658/668 match the recorded world fingerprints. Reloading them
  loses Borg process memory; nevertheless all four original-source fresh
  Borg branches die to wild cats, making them useful scenario fixtures.
- Remove the return-to-town stair-looting path, reject known occupied flee
  steps, use ordinary cures deterministically at useful wound sizes even
  when fleeing at lethal HP, and consider usable Phase Door separately from
  the conservative APHASE preparedness trait. A trapped/lethal flee can
  try a short escape despite unknown landing grids; arena/curse restrictions
  and the configured emergency failure bound remain enforced.
- Diagnostic instrumentation found a 5% legal/affordable Phase Door excluded
  by APHASE's 3% trait gate, followed by 100 unknown landing samples rejected.
  The initial healing/path patch still lost two nearby-save starts; those
  failed controls remain retained. With the emergency escape correction,
  all four fixed starts survive the bounded follow-up at full HP. This is
  one incident, not four independent lives or a population improvement.
- Fixture archive is pinned to player-cemetery bec33f4, with SHA256 checks;
  CI requires the real four saves. The standard native suite passes 69 tests.
  Final Rust checks pass (156+4+5 workspace,328 core/four ignores),
  69 engine tests,87 tools/19 scripts, formatting/Clippy/vendor. Original
  smoke matches3/3 in keys/messages/states. Six validation-control deaths
  across intermediate/final sources are qualified;2553 archive cases
  verify and all final controls reconcile with zero missing deaths.
  Final source0f2e003f988f and binary27f564611416 were built from5e8454ab
  with clean game inputs; only retained non-runtime evidence was uncommitted.
  Actual identities are preserved in INCIDENT-350-VALIDATION.json.
- Recorder-owned rolling save retention and turn-based read-only viewing
  are developed separately in angband-recorder; do not silently replace the
  pinned vendor or claim live Borg memory is restored by a native save.


## 2026-11-15: import guidance and staging follow-up (#333)

- Generic AGENTS.md guidance now applies to every track. Clarify that the
  legacy guarantee covers the full record and declared payload files;
  undeclared analysis/extra files are outside it. Qualified packets remain
  immutable in every file. No evidence or inventory is rewritten.
- Keep staging private while incomplete; publish with the source directory's
  ordinary access bits, respecting private sources. Files keep process-umask
  limits as well as source-file access; generated metadata follows the source
  record's limits. Re-import does not alter existing access. Ignore abandoned
  historical staging directories without deleting another process's work.
- All 88 tool tests and 19 script tests pass, including publication modes
  0750/0700 and immutable payload verification. All 2,547 archive cases
  verify; staging ignore is checked. Game inputs remain exactly #342 source
  87717140, so its Rust/native/play evidence applies. Required CI is pending.
  Prepared separately from #331/#346; integrate only after that promotion,
  then open a separate pinned milestone review before main.

## 2026-11-15: measured plan runs, zoo contract, ambitious plan (#352)

- Owner request after borg-zoo#123. `harness/run.sh` now runs errands plans:
  `BORG_CORE=borg-rs BORG_ERRANDS_PLAN=FILE`. It copies the plan into
  `user/`, selects it in the run's borg.txt with a line naming its SHA-256,
  records `borg_errands_plan`/`_sha256` in config.txt, and fails when the
  game did not preserve exactly that plan. The plan's bytes are now in
  `borg_txt_sha256`, so the cemetery's encounter family distinguishes plans
  with death-tool API 1 unchanged. Plans selected in the harness borg.txt,
  missing files and the original core are refused.
- Open: cemetery `rerun-death.py` does not restore `BORG_ERRANDS_PLAN` yet
  (it restores `BORG_LADDERS` the same way); a plan death's rerun reports
  "does not reproduce" until it does. That is a cemetery-owned change.
- docs/ZOO-INTERFACE.md gains the `borg_plan` row (the zoo copies
  `harness/plans/NAME`); the adapter work is borg-zoo#123.
- New `harness/plans/errands_plan_ambitious.txt`: the Strategy Manual's depth
  curve (ch. 12.2), fighters ~one dungeon level per character level, casters
  slower, no town-start phases from CL3 (CL6 casters). An engine test
  checks it is never shallower than the first plan. A hypothesis only.
- harness runs still cannot set borg.txt settings such as `borg_town_plan`;
  the local plan × shopping pilot in the PR used a zoo-like scratch driver.
- Pilot finding (72 cells, PR #354 comment): the ambitious plan's requests
  are capped by the core's readiness ("not prepared for DL2; town overrides
  the loop"), so it bounces to town instead of diving. In one cell
  (Warrior seed 501, ambitious + shopping) the M13 town planner looped
  buy oil at the General Store / drop it at Home about 4,000 times over
  ~35k game turns: a vendored-core (borg-rs town planner) defect to report
  upstream, not a plan defect.
- The plan-file checks are engine tests (strict reader + native load), not
  release-core tests, so borg-core and the build's source hash are
  unchanged and the existing CI death controls still apply.

## 2026-11-15: second main sync, errands-plan reader consolidation (#342)

- Main advanced through #308 to 820140e0 while M13 sync CI was running.
  Preserve its plan implementation, qualified evidence and handover history.
  Regenerate indexes rather than selecting one conflicting lineage.
- Remove the duplicate/lossy plan file readers. Plan and shell settings now
  share P's strict reader and OS-byte path handling. Keep core environment
  selection lazy: explicit original-core overrides do not read borg.txt;
  environment plans on that core still fail, as do file plans when the
  core itself is selected from the file. Native suites cover both contracts.
- Combined validation passes: required Rust checks (328 core tests, four
  ignored), 68 native engine tests (one skip), 45/45 M13 play parity and
  deterministic summaries, 242,641 classified rows, original smoke/death
  controls, and three policy-on controls. Fourteen fresh deaths reconcile;
  2,547 archive cases verify. P342-PLAN-VALIDATION.md names exact identities
  and limitations. Required CI is pending; #333 stays a separate draft.

## 2026-11-15: synchronize M13 main with accepted P review (#342)

- Owner authorized #331 promotion. GitHub rejected it after main advanced
  through #332. Preserve both implementations through a checked sync;
  #333 waits until this promotion gate is resolved.
- M13 source merges automatically with P's shared strict file reader.
  Retain both handover histories. Regenerate conflicting indexes from all
  evidence; qualify main's 44 provisional dossiers, retaining original
  directories under validation/m13/source-deaths. No legacy record is
  replaced. P342-M13-DOSSIER-MAP.json records source and qualified paths.
- Combined validation passes: 45/45 M13 play-log triples and deterministic
  summaries, 242,641 classified rows, three policy-on controls, original
  smoke/save-load fixture, required Rust and 60 engine tests (one skip).
  Fourteen fresh deaths reconcile; 2,523 archive cases verify. Exact build
  and limitations: P342-VALIDATION.md. Required sync CI is pending.
- The prior review receipt names its old exact head; record a fresh pin
  and the integration evidence for the owner-authorized promotion.

## 2026-11-15: review documentation follow-up (#319)

- Reflow P295/P296/P299 reports, restore missing spaces and make immutable
  hashes copyable. Values, revisions and dossier refs remain unchanged.
  #307 PR prose was already corrected before its authorized promotion.
- Describe existing lexical behavior: stopscared trims its parsed argument;
  budget integers reject surrounding whitespace. Empty aliases differ.
  Rewrap the complete ERRANDS.md configuration block. No parser, profile,
  default or game/build input changes.
- Diff/whitespace and immutable-value checks pass. This documentation task
  reuses #317/#318 evidence; required CI gates track-p integration. Then
  stop for owner-selected milestone review before main.

## 2026-11-15: qualify imported validation dossiers (#318)

- Keep the accepted #307 archive unchanged. New historical imports use
  deterministic full package hashes; original record bytes and every payload
  remain retained, with receipts. Numeric IDs are provisional outside the
  cemetery-owned canonical registry. Native export/registry ownership stays
  in borg-cemetery; no external write or alternative exporter is introduced.
- Qualify the ten #317 packets using P318-DOSSIER-MAP.json. Legacy inventory
  freezes all 2,455 accepted cases. CI rejects new bare IDs, altered legacy
  records/payload hashes and damaged qualified packages. See DOSSIER-IMPORT.md.
- Importer tests cover concurrent imports, duplicate provenance, idempotence,
  source/payload integrity and independence counts. All 87 tool and 19
  script tests pass; 2,465 archive packets verify. Real overlapping P/R
  D2446 imports coexist in an isolated registry. Ten deaths reconcile with
  the actual #317 BUILD_ID; source hash is unchanged. See P318-VALIDATION.md.

## 2026-11-15: owner accepted #307; strict file reads (#317)

- Owner instructed "ok merge and do the follow ups". #307 merged into
  main at b58e7b44. Future #317–#319 promotion remains separately gated.
- Treat only a missing borg.txt as absent. Diagnose other I/O/UTF-8 errors
  with file context and exit status 2 when configuration is consulted.
  The parameter loader shares that policy; Unix paths retain OS bytes.
- Preserve lazy environment precedence and valid/missing-file defaults.
  Document fatal re-entry behavior rather than claiming safe live reload;
  subprocess reader re-entry and native startup paths are tested separately.
- Local validation passes: full 45-case v12 parity, 295,464 trace rows,
  original smoke/fixture, 59 engine tests (one skip), required Rust checks
  and script/tool tests. Ten deaths retained as D2476–D2485; reconciliation
  passes. Exact build and limitations: P317-VALIDATION.md. CI gates merge.
## 2026-11-15: #308 re-review accepted; integration conflicts tracked

- Re-review at exact 0030b101 accepts the POC in isolation and confirms CI.
  Report: https://github.com/diegoami/borg-reborn-rs/pull/308#issuecomment-6901041092.
- All six POC evidence packages are now content-qualified, with original
  record and payload bytes preserved. PR308-DOSSIER-MAP.json retains both
  the earlier numeric rename history and the full source-qualified import
  map. The numeric reservation was insufficient for concurrent task branches.
  No other branch's evidence is replaced. #335 tracks the recovery.
- Owner main-first sequencing remains. #331's second-merge synchronization
  must consolidate the two strict config readers with lazy environment
  precedence and run both native configuration suites. It must also import
  newly accepted #332 evidence before enabling its legacy-inventory gate.
  #317/#318 are closed on track-p, awaiting accepted #331 promotion; their
  code is not silently imported here. See PR308-REVIEW-FIXES.md.
- Accepted main 70b48cc1 (#332 M13) is synchronized; all its dossier
  bytes remain unchanged. Combined native source 49daaed675fa was built
  clean at 7ee2c80b; four new-source natural deaths are also qualified and
  reconciled. The full map retains the supplementary race variant too.
  Formatting/Clippy/vendor pass, workspace 156+4+5 and release core 325
  (four ignores) pass; engine 63 (one skip), scripts/tools 19/78 pass.
  Original smoke matches all three references in keys/messages/states.
- Draft remains for owner testing; no engine/recorder or gameplay changes
  are made for the evidence recovery, and the guided game remains untouched.

## 2026-11-15: #308 returned-review fixes (reviewed head 19f2025b)

- Owner chose main-first delivery for the independent POC; P/R synchronize
  through their own later PRs. Accepted main (including #307) was merged;
  no unreviewed R implementation was imported or rewritten.
- POC D2446–D2448 are renamed to D2476–D2478. PR308-DOSSIER-MAP.json
  records the exact mapping; source/build/family/attempt identities and
  evidence hashes are preserved. D2476–D2481 were reserved on #318 for
  this delivery's old and revised-source controls. The re-review found that
  concurrent unintegrated task branches had already used those numbers;
  the qualified recovery above supersedes this reservation.
- The shipped file is tested by Plan::parse(include_str!(...)) for every
  class/default and level 1–50. The Python grammar approximation is removed;
  native checks load the real file. Canonical lowercase loop-levels is
  enforced; reversed endpoints are documented.
- Class availability is checked at Borg initialization/reinitialization,
  before acknowledging the plan. Strict POC settings reads diagnose I/O
  and UTF-8 failures, with absent-file defaults; native startup tests prove
  no silent fallback. The wider legacy-reader/re-entry work remains #317.
- BOM/CRLF are supported; oversized input reports size before UTF-8 errors.
  Budget values match accepted main's unbounded zero/none semantics, including
  signed/leading-zero spellings but excluding whitespace. Our new
  module is formatted for readability; shared donor code was not reformatted.
  Plan examples moved to harness/plans to separate them from BORG_PROFILE.
- Initialization errors still exit 2; live unsaved-game recovery is not
  claimed. The POC stays diagnostic-only and the guided game is untouched.
- PR308-REVIEW-FIXES.md records all finding dispositions and limits.
  Revised source 8b9642d658f1 was built at e900d670; no-plan CI controls
  are D2479–D2481 within the reservation. Local engine checks pass 62
  tests (one environment skip), focused POC checks 7/7, and no-plan
  3,000-turn keys/messages/states match. Required CI is rerun on the new head.

## 2026-11-15: working class-specific errand-plan POC (#305, PR #308)

- Owner requested all-class candidate plans to level 50. The example now
  explicitly covers all nine classes plus a default, exactly once per level
  1–49, and hands back at 50. Early gates follow legacy class groups; later
  shared intervals are labelled hypotheses. They are not calibrated or a
  claim that any character will reach 50. Core readiness still overrides them.
- A new data test checks all classes/levels and the 50 handover; all five
  POC engine/data tests pass, and the real mage loaded the complete new file
  through the relative borg.txt setting. Native source hash stays 3b5503a881fe.
- Follow-up #320 covers post-50 stats/resists/equipment/readiness, town visit
  frequency, equipment policy and longer-run design. Zoo #114 already owns
  checkpoint/resume storage; linked it and added the plan/controller state
  required to resume a continued life without double-counting samples.

- Owner rescoped the draft to a POC in a separate errands_plan.txt. The
  implementation accepts [default] and class sections such as [mage],
  non-overlapping level ranges and only loop-levels. Ladder/trips aliases
  are not accepted in the new file; legacy slash paths remain outside the
  opt-in POC until migration/removal is agreed after trying it.
- BORG_ERRANDS_PLAN selects a file; alternatively borg_plan in user/borg.txt
  resolves a relative path under that user directory. Wrong core, bad or
  missing files, unsupported commands and automatic-errand conflicts fail
  visibly. File bytes are frozen/preserved; no reload or live parameters.
- Runtime replaces a phase once on level change, including an active loop,
  and defers during stores/Recall. Gaps stop; manual errand/stop cancels it.
  Four parser/controller tests and four native checks cover class/default
  selection, a real mage level-up changing loops, bad input/gap handover
  and refusing untracked plan inputs in the normal measured harness.
- tools/try-errands-plan.py is the bounded diagnostic driver and preserves
  exact file/binary/command provenance. Standard harness plan runs are
  intentionally refused until cemetery family/death-replay integration
  handles the extra input/state; do not call POC runs baseline measurements.
- No engine/recorder changes; the original guided game stays untouched.
- Local checks pass: formatting/Clippy, workspace and release-core tests,
  19 scripts and 78 tools tests, vendoring, and native POC tests. Disabled
  POC keys/messages/states match the control for 3,000 turns. Standard CI
  controls (no plan selected) were reproduced at source 3b5503a881fe and
  retained as D2446–D2448 in the explicit historical validation registry.

## 2026-11-15: M13 town/trip donor refresh (#328)

- Verified tag 4.2.6-rs-town-trip-m13-v1 peels to 1443c2d1b97d110ffbf5ae420d6c6b97ac15f7ea.
  Refreshed with the three-way vendor workflow. M13-INTEGRATION.md records
  all nine conflicts, five retired parameters, corrected potion-healing
  defaults, read-only native queries and the receiving option ABI adapter.
  Vendor check passes. Errands/parameters/trace and recorder pins are retained.
- Exact-tag donor full and receiving no-errand full: 45/45 byte-identical
  keys/messages/states and effective configs; 12 natural deaths per side.
  Complete cross-repository summary equality fails because the existing
  telemetry schemas differ; the checker and its failed output are retained.
  Original-core smoke3/3 and full45/45 strictly match stored references.
- Three policy-on class pairs match play logs; 656 schema11 plans replay
  (500 with a plan). Unit tests preserve old mappings; the external historical
  replay corpus is not rerun. No urgent-purchase branch coverage is claimed.
- 318 core release tests pass (four existing ignored), 165 workspace tests,
  Clippy, formatting, generated bindings, 19 source-tool and 78 tool tests.
  Native C/Rust option ABI checks and all 13 profile/isolation tests pass.
  Final engine suite passes55 with one upstream-checkout comparison skipped.
  CI results are attached to PR #332 and M13-VALIDATION.json.
- M13-VALIDATION.json retains exact source/build/config/log identities for
  all150 validation attempt summaries, including the earlier smoke reporting
  failure (missing bc) and its separate successful rerun. Failed initial
  build/tool setup attempts remain in the raw work logs. Source hash a585062c
  matches committed runtime inputs f67966a5; receipts retain the actual dirty
  precommit build identity rather than rewriting it as a later clean build.
- All44 receiving natural deaths are retained as D2476–D2519 in the local
  validation archive, with zero missing dossiers. Twelve donor control
  dossiers are retained separately under docs/validation/m13/donor-deaths.
  Repeated/paired attempts do not inflate independent counts. Raw logs are
  at /workspace/work/m13-validation. Canonical cemetery contents/pins are unchanged.
- Both policies default off. No Zoo repin, population study, Track R
  promotion or additional Track P promotion. #293 remains separate; this
  integration alone does not establish that its shopping report is fixed.


## 2026-11-15: sweep provenance and catalogue assembly (#299)

- Stacked on #296; documentation-only follow-up, excluded from #292's
  authorized promotion. Correct stale 1f5eb966 source hash for specimen
  2ec1f250 to the actual cb48d943 hash, verified in a clean detached tree.
- Name both catalogue fragments and the separate experiment manifest,
  their assembly order and shared handoff revision; distinguish that
  documentation pin from the runtime specimen pin. The profile filename
  was already present, so qualify review292 F5's stronger claim.
- Actual pinned-zoo static validation expands 384 cells with 128/900 disjoint
  assignments. No runtime/profile/endpoint/sample-size or source-pin
  change, measured run, external repository write or dispatch occurs.

## 2026-11-15: budget validation complete (#296)

- Combined source aed27276 passes full 45/45 v12 play parity, 295,464 named
  rows, original smoke 3/3/fixture, 55 engine tests (one skip), 156+4+5
  original / 143 core tests (one ignored), Clippy/formatting and 19/78 Python
  tests. Six paired valid-budget cases match #295 byte for byte.
- Preserve ten natural deaths at D2466–D2475; reconciliation passes with
  actual build identity. P296-VALIDATION.md records intentional invalid-
  input changes, finite-budget semantics and infrastructure/test retries.
- Integrate into track-p after required CI, then stop for separate owner
  review/promotion with #295/#299. The #292 authorization excludes this.

## 2026-11-15: strict errand budgets (#296), validation pending

- Stacked on #295. Invalid configured budgets now exit2 with source/value
  diagnostics instead of silently becoming unbounded. Accept integer
  0..i64::MAX; absent/0/exact none mean no bound. Negative, empty,
  non-integer, overflowing and non-UTF-8 budgets are rejected.
- Preserve positive budgets and original-core/inactive-errand isolation.
  Actual native tests exercise finite stopping, invalid values and the
  ignored inactive contexts. Interactive shell parsing is unchanged.
- The first native finite-budget assertion used the wrong zoo outcome:
  the pinned adapter maps runner borg_stopped to stalled. The actual
  budget stopped at turn11. Assert both layers and the native stop reason;
  no product or adapter change is needed.
- This task is excluded from the already-authorized #292 promotion.
  Required checks/gameplay/evidence validation and task CI are pending.
  No experiment or external repository write is authorized here.

## 2026-11-15: stopscared validation complete (#295)

- Source27d7f453 passes full 45/45 v12 play parity, 295,464 classified rows,
  original smoke 3/3/fixture,53 engine tests (one skip),155+4+5 original/
  143 core tests (one ignored), Clippy/formatting and 19/78 Python tests.
  Three valid fear-setting controls match synchronized base byte for byte.
- Preserve both source variants' ten deaths. Explicit ID map retains
  pre-loop8f45f46b records atD2446–D2455; final deaths areD2456–D2465.
  Both registries and identity/hash checks pass. P295-VALIDATION.md records
  actual builds, compatibility limits and the infrastructure retries.
- Task CI gates track-p merge. #292 promotion does not accept this later
  task; a separate owner review/promotion remains required.

## 2026-11-15: strict configured stopscared (#295), validation pending

- Stacked on main synchronization #298. This task is excluded from the
  owner-authorized #292 promotion and later integrates into track-p only.
- Reject invalid/non-UTF-8 fear values with source/value diagnostics and
  exit2. Preserve valid aliases, numeric1–1000, empty/none/default handling,
  environment precedence and original-core isolation. Interactive shell
  parsing is unchanged. Native/profile tests cover the real startup paths.
- Wrap the ERRANDS.md paragraph noted by review292 F4. Required validation,
  evidence retention and task CI are pending. No experiment is launched.

## 2026-11-15: final #292 synchronization validation complete (#298)

- Combined source15289917 passes full 45/45 v12 play parity, 295,464 named
  trace rows, original smoke 3/3,50 engine tests (one local environment
  skip), required Rust checks (154+4+5 original / 143 core),19/78 Python
  tests and vendoring. P292-SYNC-VALIDATION.md records exact builds/limits.
- D2436–D2445 retain the final ten natural deaths; earlier sourcec4a38683
  deathsD2423–D2432 also reconcile. Nine imported recorder/loop dossiers
  retain exact source/attempt identities under the explicit ID map.
- Required CI must pass before integration, then complete the owner-
  authorized #292 main promotion. #295/#296/#299 remain separate later
  work on track-p; no experiment or new reviewer is launched.

## 2026-11-15: complete loop-levels integration in #298

- Main advanced to2bb9b72f (#297) during validation. Preserve its loop-levels
  implementation, docs and tests, with no P change to that policy.
- Its three additional independently allocated dossiers are imported as
  D2433–D2435 with unchanged source/attempt identities. The same explicit
  mapping now covers all nine imported recorder/loop cases.
- Pre-loop sourcec4a38683 passed full 45/45 v12 play parity, 295,464 named
  rows, original smoke 3/3 and47 engine tests (one skip); ten deaths are
  D2423–D2432. Disk exhaustion prevented the first fixture launch, then a
  fresh retry passed. Only regenerable caches were cleared.
- Validate the combined source before #292's authorized promotion. Later
  strict fear/budget work stays excluded. No experiment is launched.

## 2026-11-15: owner authorizes #292 promotion; synchronize main (#298)

- The owner explicitly instructed "you can merge 292 and continue" after
  the independent review recommended acceptance. Preserve the standing
  track-p gate for later #295/#296/#299 work; those are excluded here.
- Synchronize main352cefeb, preserving recorder CMake dependency handling,
  guided-debugging rules/launcher changes and both sessions' handovers.
  Recorder/engine/dependency pins are unchanged. The consumer CMake file
  is a build input, so validate and retain evidence on the new source.
- Recorder PR268 independently allocated six dossier IDs already used by
  the P archive. Keep every existing P case unchanged; import its six new
  cases underD2417–D2422, with source/attempt identities unchanged.
  P292-DOSSIER-MAP.json records the exact old/new names and fingerprints.
- Archive REVIEW-292-STRICT-ERRAND.md unchanged from the review branch.
  Its base6acbf170 has the same tree as main0ed0e5de. Qualification: old
  apply_line already parsed the whole line before applying commands; the
  actual fixed defect was warning-and-continuing instead of failing.
- Required validation/CI are pending before track-p integration and the
  owner-authorized #292 merge. No population experiment is launched.

## 2026-11-15: strict configured errands (#269), validation complete

- Stacked on #289; targets track-p after the owner-authorized #260
  promotion. Valid commands, environment precedence, absent/empty/none disabling
  and original-core isolation are preserved. Invalid errand command
  lines now fail with source/value diagnostics and exit2 instead of
  continuing an unconstrained run. Parsing validates the entire command
  line before applying any prefix.
- Pure tests and actual pinned-zoo/native process cases exercise invalid
  file/environment values, valid overrides and original-core isolation.
  ERRANDS.md and ZOO-INTERFACE.md state the intentional input change.
- The first candidate (014e64a4/source77f040ff) passed pure/native-selector
  cases but failed the pre-existing shopping fixture, which uses an empty
  string to disable errands. Preserve that legacy empty alias; the test
  itself is unchanged. Its full-suite evidence remains with its own build.
- Corrected sourcecb48d943 atc259f69c passes full 45/45 v12 play parity,
  295,464 named rows, trace on/off, original smoke 3/3, all45 engine tests
  (one local environment skip), required Rust checks and 19/78 Python tests.
  P269-VALIDATION.md records both candidates and the compatibility limits.
- D2399–D2406 preserve the first candidate's eight full-suite deaths;
  D2407–D2416 preserve ten corrected-build deaths. Both temporary
  registries reconcile. Explicit historical/CI export destinations retain
  the existing missing-death guard; no canonical migration/write occurs.
- Sweep handoff/examples now pin2ec1f250 (same corrected sourcecb48d943),
  with pinned catalogue validation and no population dispatch.
- Required CI remains the task integration gate. The owner authorized
  #260's earlier scope; this later task stays on track-p for its separate
  review/promotion decision. No population experiment was launched.

## 2026-11-15: owner-requested repeating level interval (#294)

- `/loop-levels A B` explores every intermediate level in both directions:
  `3 5` means 3,4,5,4,3,...; 0 is town. Two distinct engine depths 0–127
  are accepted. The route never completes on its own; budget/stopscared
  and explicit stop retain their existing semantics.
- Town intent suspends the route until actual town arrival, then restarts
  the first endpoint. Unprepared descent returns home rather than forcing
  a level or cancelling the loop. Town endpoints include the core's town
  work. Existing inventory management (#293) is not replaced here.
- Return travel can Recall; deep departures use the core's usual town
  Recall/readiness/supply policy. Actual arrival is observed before travel
  to the first waypoint; an adjacent DL1 target uses stairs. Pending Recall
  is not reissued, and absent Recall falls back to stair travel.
- Pure planner tests cover intermediate/reverse cycles, planned/unplanned
  town, waiting for town despite cleared flags, unexpected rises and Recall
  landing outside the interval. Three fixed-start engine tests cover
  Recall plus restart/queue cancellation at budget, stairs-only repeated
  visits and town override without aborting the loop. The original guided
  session and frozen binary remain intact.
- Formatting, Clippy, 162 workspace tests and 143 release-core tests pass
  (one pre-existing core ignore); engine checks pass 47 tests with one
  environment skip. The vendoring check passes. Against the #268 control
  binary, no-errand keys/messages/states match over 3,000 turns each at
  seeds 1010 (warrior), 1001 (mage), 1002 (rogue); all survived. This is
  bounded regression evidence, not a population improvement claim.
- Standard CI natural-death controls were reproduced at the final native
  source hash `50de000425b6` and retained in the explicitly selected
  historical validation registry: D2402 (vendored core, seed 1011), D2403
  (reborn smoke, seed 102), D2404 (reborn rerun-source, seed 1020). No
  loop was active in those controls; no missing-death guard was weakened.
- #206 was closed as superseded; its three evaluation documents are
  preserved by exact-source links in the closing comment and local archive
  `/tmp/pr206-preserved-85429a831316`. #268's consumer dependency fix is a
  prerequisite here; no recorder implementation or engine gameplay changes
  are part of this errand.

## 2026-11-15: owner-requested completion of recorder dependency PR #268

- The F17 fix remains consumer CMake integration, with the recorder pin,
  adapter contents and upstream game unchanged. The owner requested its
  delivery after confirming this boundary.
- The latest engine CI had passed the suites and stopped at missing
  historical validation dossiers. At exact source `0dadea400855`, core
  seed 1011 repeats D2399; the smoke and rerun-source cases are retained
  as D2400 (seed 102) and D2401 (seed 1020). The explicit historical
  registry override preserves the existing CI guard; it was not weakened.

## 2026-11-15: owner accepts promotion #260; current main synchronization (#287)

- The owner explicitly instructed: "ok then merge it and continue with other
  tasks", after the current review/fix scope was explained. This accepts
  the P1–P3 retrospective/promotion scope and delegates merging #260.
  The standing track-p gate remains in force for future work.
- #285 passed CI and integrated at1949530b. Main then contained the
  death-tool ownership change811f338a (#282); synchronize it here before
  #260. Preserve both handovers and regenerate the historical index using
  local links, retaining all2378 local cases throughD2398.
- Gameplay/build source remains4182670d. No external repository write or
  migration occurs here. The pending errand-selector task #269 stays on
  track-p; #272's RNG qualification was already implemented in #280.
- This sync must pass required CI before integration; then perform the
  owner-authorized #260 merge and record its actual merge commit on GitHub.
## 2026-11-15: first guided shopping incident; inventory replacement #293

- Owner observed no purchases and requested replacing the inherited town
  inventory management. Follow-up #293 records the evidence and requested
  direction; no inventory/caution policy change has been made yet.
- The Dwarf Priest's round stored Recall and gave away both Cure Serious
  Wounds potions under the no-selling option before selecting torches.
  At turn 341 a distant battle-scarred veteran triggered the core's
  leaving/fleeing flags; `/shopping` aborted at full HP and danger 0.
  A second invocation aborted at the same turn. Inventory choices and
  town-threat interruption are separate concerns.
- Original evidence is preserved under the guided session's
  `incidents/shopping-20261115T122211Z/`. The original live game was not
  changed. `reproduction-from-last-frame-v3/` uses the frozen executable
  and a copied last-frame native snapshot with its stored RNG: it loads
  the same character and reproduces the abort at turn 341 without advancing
  a turn. This is a fresh-Borg-state reproduction, not a verified replay
  of the still-live original recording. No engine/recorder fix is implied.

## 2026-11-15: local reconciliation with PR #282

- Local main advanced to `811f338a`. Both incoming handover entries and
  the local guided-session/launcher work are preserved; the handover's
  concurrent insertions were combined. RULES 12's cemetery ownership and
  the local guided-debugging rule 26 coexist.
- Installed the exact `death-tools-pin.txt` revision, `33c375527a8e`, in
  ignored `vendor/borg-cemetery`, where the compatibility pointers discover
  it. No sibling cemetery checkout was present. The 78 tools tests and 19
  scripts tests pass; shell syntax and diff checks pass.
- Corrected tools/README.md's stale default export directory. The guided
  recording and frozen executable were preserved; no game was restarted.

## 2026-11-15: owner-requested death-tool ownership (#273)

- The four native death-tool implementations moved to borg-cemetery with their
  source provenance and original tests. Local command paths are API-checked thin
  pointers; the harness and Borg instrumentation remain here.
- Normal exports target the cemetery checkout. Explicit temporary destinations
  and the existing historical CI reconciliation guard remain unchanged. The
  cemetery owns catalogue/presentation updates; its historical index is not
  overwritten by the native registry formatter. No bulk dossier migration.
- CI engine tests clear policy BORG_* variables; pointer discovery also finds
  the pinned vendor/borg-cemetery checkout without that environment variable.
  The test and all its assertions remain unchanged.
- CI reads an immutable cemetery revision from death-tools-pin.txt and checks it
  out without a write credential. Local tools locate the sibling of the primary
  checkout or BORG_CEMETERY_DIR. No source/build inputs or gameplay changed.
- Reused logic preserves family/key-stream deduplication, repair, source-matched
  reruns, errand/ladders settings and missing-death failures. Existing tools tests
  pass against the external implementation; required CI remains the merge gate.
- This is owner-directed storage infrastructure, not new P/R strategy work or
  implementation of the diagnosis/counterfactual plans #153/#212.


## 2026-11-15: synchronize cemetery documentation (#284)

- Main advanced to `f441296f` (#276) after #280 passed CI. Preserve that
  documentation/tooling change on track-p through a CI-checked sync PR;
  main and the owner promotion remain unmerged by this session.
- Resolved handover by retaining both sessions' entries. Regenerated the
  death registry/index using the incoming generator: all2378 cases through
  D2398 and the canonical cemetery notice survive. No case is moved to
  another repository and the runtime source remains `4182670d`.
- The migration snapshot in #276 describes D2358/2338 cases as of its
  original pin; track-p has since added40 dossiers in #280. The migration
  session must include D2359–D2398 in its eventual identity-preserving move.
- #269's core-selector component and #270/#271 are addressed by #280;
  #269's additional errand-selector investigation remains open. Owner
  acceptance of #259/#260 remains pending. See P-REVIEW-VALIDATION.md.

## 2026-11-15: retrospective review #259 fixes complete on track-p (#275)

- Stacked on #277 into track-p. Strict core selector accepts only
  reborn/0 and borg-rs/1, preserving default and environment precedence;
  invalid values exit 2 with their source/value. Added pure and actual
  native-profile contract coverage.
- Engine/manual evidence confirms decimal rest turns, rather than the
  review's proposed key-digit interpretation. Preserve names/defaults,
  exclude engine repeat sentinel 1 with bounds 2–9999. The hit-power
  multiplier is inside a denominator: clarify its unit, keep zero invalid.
- Document implicit factor-1 source citations without moving them to the
  adjacent factor-3 branch, qualify RNG preservation at defaults, remove
  a stale retained-literal claim, and document strict name casing.
- Validated source `4182670d` at implementation pin `132aeb26`: full 45/45
  v12 play parity, 295,464 named rows, trace on/off, original smoke 3/3,
  native profile5/5, engine42 (one skip), required Rust and17/70 Python
  checks pass. P-REVIEW-VALIDATION.md records limits and input changes.
- Reproduced old-source original-core full 45/45; preserved30 new deaths
  D2359–D2388 plus existing D2355. Updated build's ten deaths are
  D2389–D2398; both registries reconcile. Actual build identities retained.
- Sweep prompt/examples now pin `132aeb26`, with pinned-catalogue validation.
  Required CI must pass before track-p integration. Owner acceptance of
  #259/#260 is still pending; no promotion or population run is authorized.

## 2026-11-15: review #260 follow-up (#274)

- Integrated main `83cf442a` into the task branch for `track-p`, preserving
  shopping #267, its engine checks and dossiers D2356–D2358. No main merge
  or force push is authorized by this task.
- Clarified in all four workflow entry documents that pre-rule P1–P3 need
  owner acceptance of retrospective review #259, not a second runtime
  promotion. New P work still needs reviewed promotion. Other tracks keep
  their existing authorization; reviewer recommendations are not acceptance.
- Archived the two P reports unchanged from review branch commit
  `ade599661355bd5556015c8cf0a50b54d661a703`; no R work was imported.
  The reports retain the reviewer's assertions; #275 records corrections
  where engine evidence disagrees. #259 and #260 remain owner review PRs.
- Review #260 F1: `gh pr diff 260 --name-only` showed six policy/CI files,
  not a shopping removal. Synchronizing main removes the head divergence;
  the task PR's CI checks the integrated tree before any track-p merge.

## 2026-11-15: earlier pointer documentation (#276; routing superseded by #273)

This entry records the preceding pointer-only change. The #273 entry above and
`docs/deaths/POINTER.md` describe the current ownership and routing. In particular,
the historical cases remain local and their index links were corrected to their
actual locations; reconciliation does not copy evidence.

## 2026-11-15: the recorder adapter is a configure dependency (#268)

- **M1 review F17** (Low) in the angband-recorder M1 review asked for two
  things: add the adapter to CMake's configure dependencies, and cover it
  with a generated-tree dependency test. Both are done.
- `engine/recorder-hooks.patch` is applied at **configure** time, by
  `execute_process` inside the `SUPPORT_BORG_RECORDING` block, so the
  only way to re-apply it is to re-configure. `CMAKE_CONFIGURE_DEPENDS`
  listed `engine/patches/*.patch` and `engine/UPSTREAM` but not the
  adapter, so a plain incremental `cmake --build` could keep stale
  hooks after the adapter changed. `engine/CMakeLists.txt:29` now
  declares it.
- `engine/tests/test_build.py` gains two tests: one asserts the
  declaration, and one is **functional** — it configures a throwaway
  build, records the generated tree's mtime, touches only the adapter
  patch, runs `make cmake_check_build_system` and asserts the tree was
  re-applied. The test was reproduced as failing on unfixed `main`
  (the tree's mtime did not move) and passing here.
- **The game is unchanged.** `engine/patches/`, `engine/frontend/`,
  `engine/sources.cmake`, `engine/apply-patches.sh` and
  `engine/recorder-hooks.patch` are byte-identical to `main`, so the
  generated tree's content is identical: smoke is **3/3** against the
  reference. The recorder-enabled build configures, builds and passes
  `harness/check-recorder.py` ("Consumer recorder round trip passed").
- The vendored recorder and `recorder-pin.json` are untouched (v0.1.0 /
  `c4986b26b9fd`); re-vendoring the recorder's own corrections is a
  separate step.
- The branch's new build identity produced three new deaths (D2359–D2361),
  retained with their trace tails and hashes. The two earlier red CI runs
  on this branch were the RULES 12 reconcile gate refusing exactly those
  unrecorded deaths.
- **Open:** the upstream tree's *content* is still not a configure
  dependency — `engine/UPSTREAM` is a marker file (the tree id), so an
  in-place edit under `engine/angband/` does not re-apply the generated
  tree. Filed as #288; benign today (a re-pin updates `UPSTREAM`, RULES 9
  forbids editing the tree, and `engine/tests/test_engine.py` catches
  it), and out of F17's scope.

## 2026-11-15: the death registry points at borg-cemetery (#276)

- The death dossiers are recorded in **borg-cemetery**, the canonical
  home for recorded Borg runs, written death investigations and playable
  saved scenarios. This repository keeps the code that writes and
  reconciles them; `docs/deaths/POINTER.md` says where the cases live
  and which seam serves both homes.
- The seam is `BORG_DEATH_RECORDS_DIR`, already honoured by
  `tools/export-death.py`, `tools/death_registry.py` and
  `tools/reconcile-deaths.py`. **The export path and its defaults are
  unchanged**, so no bulk migration happened and the CI reconcile gate
  (RULES 12) is the same gate. The routing of exports is #273's, and
  the move and the reconciliation are the other session's.
- Two hazards are documented for that session, both found here:
  - **the case ids collide.** Ours run `D0001`–`D2358`; the cemetery's
    run `D0001`–`D0007` and mean *different deaths* by each id. A
    directory copy would clobber the cemetery's seven cases. The
    mapping has to be written down before the move; `encounter_family`
    survives a renumbering.
  - **the registry generator would overwrite the cemetery's index.**
    `tools/death_registry.py` writes `index.json` and `README.md` into
    the registry root. Both index files carry the same schema
    (`borg-rs/death-index@1`) and key set, so a content check cannot
    tell the registries apart; the guard has to be about which
    directory the generator was pointed at.
- `reference/deaths/INDEX.md` links our 2,338 cases to the cemetery and
  keeps the copied reference registries local; `BORG_CEMETERY_URL` and
  `BORG_DEATH_RECORDS_DIR` override the targets.
- The registry's README notice lives in the generator's **template**,
  not in the generated file: the generator rewrites `README.md` from
  scratch, and a hand-written notice there was silently lost on the
  next run. `tools/tests/test_death_registry.py` pins that it survives
  a second regeneration.
- AGENTS.md, RULES.md 12, README.md, docs/SPEC.md, docs/SOURCES.md,
  reference/README.md and tools/README.md name the cemetery.
- Checks pass: formatting, Clippy, 161 workspace tests, 19 scripts, 74
  tools, 40 engine tests (16 environment skips). `tools/death_registry.py`
  is idempotent and the reconcile gate resolves the registry.
- The P1–P3 retrospective review, the track-p workflow review and the R1
  review were also completed: `docs/REVIEW-259-P1-P3.md`,
  `docs/REVIEW-260-WORKFLOW.md`, `docs/REVIEW-266-R1.md`, with the
  findings posted on #259, #260 and #266 and filed as #269–#272.

## 2026-11-15: owner-guided recorded debugging workflow

- Active guided session: `.angband-reborn/guided/20261115T105724Z-ErrandDebug/`.
  `session.json` records private save/recording paths, environment, frozen
  executable hash and PIDs (recorder 99085, game 99087 when launched).
  It starts at fresh birth; the owner chooses the character and errands.
  Preserve this session and inspect its manifest/process before restarting.
- A fresh X11 birth, `/town`, explicit save, save-and-quit (Escape at the
  final prompt) and isolated-profile replay verified all recorder streams.
  Receipt and original evidence are under the session's
  `evidence/gui-qualification/`. No new engine/recorder patches were made.
  Recorder changes, if needed, go through PRs in angband-recorder per owner.
- RULES 26 and GUIDED-DEBUGGING.md define the requested workflow: the agent
  launches a recorder-enabled private session, the owner reports problems
  and saves if possible, and the agent collects evidence and reproduces.
- Preserve the original build/configuration/starting save and recording,
  verify the actual incident save, and collect live evidence if saving or
  input fails. A save alone does not restore Borg process memory.
- The qualified route is a short fresh-birth GUI session; arbitrary loaded
  saves and long human sessions have not yet been qualified here. The
  per-session manifest plus frozen runtime preserve the setup for diagnosis.

## 2026-11-15: owner-requested /shopping (#208)

- Added `/shopping`: a town-only round of the vendored core's buying,
  selling and home decisions. It hands over outside the store UI when no
  useful transaction remains; it does not force purchases or change the
  game's no-selling birth option. `/town; /shopping` returns first.
- Shopping runs after caution and before generic exploration/departure.
  Caution's direct town descent aborts instead. Dungeon calls, an
  unreachable shop, the errand budget or the stall cap also abort/stop;
  failures drop the queued errands, normal completion advances them.
- Fixed-start engine tests cover purchases/home storage, two queued
  rounds, town danger cancelling the queue, a one-turn budget and a
  dungeon call that stops without advancing a turn. Mage seed 1001
  completed at turn 591; rogue seed 1002 aborted safely in town at turn 1.
- No-errand keys/messages/states match `8351f0a3` byte for byte for 3,000
  turns at seeds 1010 (warrior), 1001 (mage), 1002 (rogue); all survived.
  This is regression evidence, not a ranking of the errands' reliability.
- CI's standard natural-death runs are retained as D2356 (vendored core,
  seed 1011) and D2357 (reborn core, seed 1020), reproduced locally at
  source hash `bcd9f0a9404e`. D2356 reconciles with the downloaded CI
  registry; neither run used shopping. No death guard was weakened.
- Rust formatting, Clippy, workspace tests (excluding borg-core), release
  borg-core tests and the vendor check pass. The corner freeze remains
  undiagnosed; no population experiment or GUI shopping session was run.

## 2026-11-15: owner-requested watched-game save fix

- `watch-borg.sh` now passes the data directories before `-u`: upstream
  resolves the save filename immediately when it parses `-u`, so the old
  order selected the default build-tree save path instead of the persistent
  `.angband-reborn/save` directory. The owner reported a save failure and
  a new character after restarting; no recent matching save was found.
- Added `--name NAME` and `--help`; the positional name and default
  `RustBorg` still work. Use the same name to reload a character.
- Validation: `bash -n`, `git diff --check`, and isolated launcher checks
  with stubbed build/game commands pass for default, positional and named
  launches, paths/names with spaces, save-directory argument ordering,
  invalid arguments and build-only. A live GUI save/reload was not tested.
- This fixes launch arguments; the reported corner freeze remains
  undiagnosed. The nested-entry diagnostic alone does not prove its cause.


## 2026-11-15: owner changes P integration to track-p (#257)

- The owner requires `track-p` (explicit spelling correction), with task
  PRs merged there after CI and regular milestone reviews before any
  promotion to main. The owner chooses the reviewer and accepts each
  milestone; the session never treats green CI as review approval.
- Saved the rule in AGENTS.md, RULES.md, PLAN.md and GOALS.md. P review
  fixes use track-p too; other-track PRs keep their own bases/workflows.
  CI now runs on track-p pushes as well as main and pull requests.
- P1–P3 runtime changes were already on main8351f0a3. This is a historical
  exception needing retrospective review, not independently reviewed work.
  Do not rewrite/revert main. A full combined review uses the pre-P main
  baseline c2c4e61d; the owner promotion PR into main currently changes only
  workflow documents/CI because runtime code was already merged.
- Checked open PRs: no P task PRs are pending; #251/#255/#256 are review
  snapshots to supersede with the combined review. R PR #248 and the
  separate repivot PR #206 are untouched. No reviewer or experiment is run.

## 2026-11-15: P3.2 sweep handoff (#245)

- CORE-SWEEP.md is an exact-commit-pinned prompt for the owner’s zoo
  session, with sample catalogue/profile/experiment fragments. It specifies
  128-seed exploration, fixed candidate selection and900 fresh matched
  confirmation pairs, prespecified success/failure accounting and exact
  analysis. No population experiment or sealed-set inspection occurred.
- The actual pinned catalogue validates384 exploratory cells and the two
  draft suites’ disjoint assignments. The sizing script reproduces the
  exact boundary-model power. P3.2-VALIDATION.md records the checks.
- Build inputs are identical to P3.1; its45/45 play parity, profile contract,
  named trace and eleven preserved deaths apply. Task PRs merge only after
  green CI. All P1–P3 deliverables are implemented when this task merges;
  milestone review snapshots remain for the owner's chosen reviewer.
- P1 review #251 pins main33ebdc1e. That real merge is verified by its
  parents, although GitHub left #246 open after a502; the stale PR and
  issue were closed manually after verification. No source was reverted.
  P2/P3 review snapshots are opened when their final task merges.

## 2026-11-15: P3.1 validation complete

- The real pinned zoo profile contract passes3 cells with no BORG_* policy
  environment: numeric core1, resolved parameter values/origins, invalid
  rejection and original core0. Native diagnostics accompany the summary.
- Full45/45,295,464 named rows, current-shell ladder, original-core smoke
  and death fixture pass. Required checks and36 engine tests pass (two
  environment skips). D2345–D2355 preserve eleven natural deaths, reconciled.
- P3.1-VALIDATION.md and ZOO-INTERFACE.md document the remaining adapter
  record limitation: its config_checked field still names the written file;
  native parameter/trace files must be preserved and hashed for analysis.
- Owner sweep handoff #245 is next; no population run has been launched.

## 2026-11-15: P3.1 native zoo profile contract

- Numeric borg_reborn_core=1 selects the vendored core;0 retains the
  original core. This fits the pinned zoo's bool/int-only profile writer.
- Native borgh calls without BORG_RUN_DIR place diagnostics beside their
  existing -summary file, preserving explicit caller paths. No policy or
  RNG behavior changes; the upstream engine and zoo pin remain untouched.
- Three actual pinned-adapter contract cells pass with no BORG_* policy
  environment: parsed profile values/origins, invalid-value exit2 and the
  original-core selector. Example control/rest-SP7/8 profiles are provided.
  Full/harness validation is retained in the final task report.
## 2026-11-15: P2.4 power valuation complete

- 727 parameters (1734 total) cover activation values, equipment scores,
  speed/armour tiers, inventory quantities/weights and readiness rewards.
  Retained mechanics and the unchanged dynamic-formula switch are declared.
- Full45/45,295,464 named rows, current-shell ladder, original-core smoke
  and death fixture pass. A zero unidentified-weapon bonus changes queued
  keys at5656 on Human Warrior seed1000; no improvement claim.
- Default expansion reproduces original bytes with only necessary i64
  parameter promotions removed.137 core tests, required Rust checks,
  17 scripts/70 tools tests and vendoring pass. D2334–D2344 retain eleven
  natural deaths, fully reconciled (P2.4-VALIDATION.md).
- P1 review snapshot is #251, unmerged. P3 #244 tests integer zoo profiles
  and native diagnostics; #245 provides the owner's sweep prompt. Owner
  authorized continuation; no reviewer or population experiment launched.

## 2026-11-15: P2.3 item-use validation complete

- 19 parameters/20 sites,1007 total; default expansion restores original
  bytes. Full45/45,295,464 named rows, current-shell ladder, original-core
  smoke/death and required checks pass on both source identities.
- CCW-stock trial has no change; the rod-failure trial changes queued keys
  at90378 on Half-Elf Warrior seed1001, on both builds. No improvement claim.
- D2311–D2322 retain original evidence; D2323–D2333 preserve eleven new
  natural deaths. Both registries reconcile. P2.3-VALIDATION.md has details.

## 2026-11-15: P2.2 integrated validation complete

- Defence retains 45/45, named traces and current-shell ladder parity
  after the parent integration. The actual diagnostic character is Hobbit
  Priest (race index3 with menu letters); the report now states it correctly.
- Required checks pass again. D2289–D2299 retain original evidence and
  D2300–D2310 preserve eleven integrated-build deaths, reconciled.

## 2026-11-15: P2.1 integrated validation complete

- Attack parameters retain 45/45 default parity and the same diagnostic
  divergence after receiving current main through P1.4. Current-shell
  ladder, named trace, original-core smoke/death and required checks pass.
- D2265–D2276 retain original evidence; D2277–D2288 add the integrated
  build's twelve natural deaths. Both registries reconcile. CI reruns on
  the updated task head; downstream task branches receive these records.

## 2026-11-15: track P integration with the option-C fixes

- P1.4's first head passed CI, but its merge was rejected after main's
  option-C fixes landed (#238). The premature merged-status comment was
  corrected immediately. Integrated main e514e8b1, preserving its shell
  changes; the only conflicts were independently allocated dossier IDs.
- Our original P1.4 cases are now D2242–D2253; D2254–D2264 preserve the
  integrated build's eleven new deaths. Identities and evidence remain
  intact, indexes/hashes regenerated. Both registries reconcile.
- Integrated full **45/45**, 295,464 named trace rows, current-shell ladder
  trace on/off, original-core smoke **3/3**, death fixture and all required
  checks pass. P1.4-VALIDATION.md identifies both builds. CI reruns before
  merging; downstream P2 branches will receive the integrated parent and
  rerun game validation. No review PR has been merged or reviewer launched.

## 2026-11-15: the option-C review and re-review fixed (#227)

- **Review #227** (Kimi K3.1 Pro): 13 findings, fixed in #230, #231 and
  #233. **The re-review** found 3 blocking issues and 7 more, fixed in
  #236's PR. The errand shell now:
  - ends a level in one place (`move_on`) whether the core is done with it,
    or `/explored` or `/stall` says so; the thresholds are measured on
    every think, the core's own leaving and fleeing included;
  - never walks down to a level the core is not prepared for: a command
    stops ("not prepared for DLn"), a ladder trip goes home;
  - tells fear (`fleeing_to_town`, or rising while not prepared for this
    depth: the trip ends) from a town errand of the core's own (rising
    while prepared: restock, sell, fix its level; the plan waits);
  - gives up steering after `/stall` (at least 10,000 game turns) on one
    level without moving on: a command stops ("no way on"), a ladder lets
    the core play on. A ladder necromancer that sold and re-bought in town
    for 13,890 turns no longer freezes; the owner's "runs to a corner"
    case is not reproduced yet;
  - ties an errand to its character (birth key: HP rolls, height, weight,
    age, name).
- **Measured:** `/explored` and `/stall` fire in play (with low settings).
  The 90% default does not bind early: borg-rs is bored with a DL1 level
  after seeing about 3% of its floor. No errand: full 45/45 identical to
  borg-rs v12.
- **Open:** the owner's corner case (seed and class unknown); the
  blackguard who rests "to gain Mana" for 23,000 turns is borg-rs's own
  (blackguards do not regain mana by resting), for track P or an issue
  upstream; RULES 4 (no AI trailers) was broken by this session's earlier
  commits and PR bodies, which cannot be amended.

## 2026-11-15: track P, P2.4 (#243), power extraction in progress

- Audited and extracted 727 equipment/inventory/activation valuation
  parameters (1734 total). Numeric game encodings and mechanics are retained;
  default expansion restores the original source bytes after removing the
  import and necessary i64 parameter promotions. No vendored formatting.
- Release unit tests and Clippy pass; game validation follows integration
  of the concurrent main errand-shell fixes through the parent stack.
  A boundary test rejects zero divisors and out-of-storage-range timers.

## 2026-11-15: track P, P2.3 (#242), item-use extraction in progress

- Extracted 19 parameters at 20 sites (1007 entries total). Default full
  45/45 and ladder parity pass on the pre-integration branch; default
  expansion reproduces the original source bytes. Rust/Python checks pass.
- The CCW-stock diagnostic on seed1011 causes no change; the rod-failure
  diagnostic is pending. Every measured death is retained.
- Another session's errand-shell fixes landed on main while P1.4 CI ran.
  Integrating them through the stack and rerunning before final PR gates;
  the older reports remain evidence for their actual source identities.

## 2026-11-15: track P, P2.2 (#241), defence parameters

- Added 218 defence settings at 242 sites (988 total): danger/failure
  windows, scoring, timers, banishment/destruction/teleport decisions,
  visibility and panel-shift budgets. CALIBRATION.md lists retained data.
- Full **45/45**, named traces and ladder parity hold. Suppressing Bless
  changes the priest's selected check at turn36902 from borg_caution to
  borg_attack. See P2.2-VALIDATION.md; no improvement claim.
- All required Rust/Python/vendor checks and default-expansion audit pass.
  D2289–D2299 preserve eleven natural deaths, reconciled without omissions.
- Initially stacked on #247; pending CI gates will merge in task order.
  Remaining P work: item use #242, power #243, profile contract #244 and
  owner sweep prompt #245. Review PRs stay unmerged for the owner.

## 2026-11-15: track P, P2.1 (#240), attack parameters

- Added 167 attack settings at 188 sites (770 total): scoring, mana
  reserves, danger horizons, failure/paranoia limits and resource costs.
  Retained damage/geometry/interface data is declared in CALIBRATION.md.
- Full **45/45**, 295,464 named trace rows and ladder parity hold. A
  zero overkill multiplier changes borg_attack to borg_flow_old at turn
  141 and dies later; the earlier hit-threshold trial has no change.
  Both are retained without an improvement claim (P2.1-VALIDATION.md).
- Required Rust/Python/vendor checks pass. Defaults restore original
  source bytes apart from import and three comparison promotions.
  D2265–D2276 preserve twelve natural deaths; none are missing.
- Stacked on P1.4 #246 while its engine CI finishes. Remaining track P:
  defence #241, item use #242, power #243, zoo profiles #244, sweep #245.
  Owner authorized this continuation across milestone handoffs; review
  PRs remain unmerged and reviewers remain the owner's choice.

## 2026-11-15: track P, P1.4 (#222), named core decisions

- Added the independent `trace-core.jsonl` stream, successful root-check
  hooks, core state/danger/goals/errand/queued keys and immediate line writes.
  Nested thinks retain separate selection frames. CORE-TRACE.md specifies
  the sampling point, composite-root names and schema limitations.
- Full **45/45** matches P1.3 and pinned v12 play logs. All 295,464 rows
  parse with contiguous sequences and no unclassified decisions. Tracing
  off preserves play; default-profile ladder remains identical.
- The recovery override changes the named check at turn 561, from
  borg_flow_take to borg_recover, and retains the known negative outcome.
- Death dossiers retain the last 200 core rows and their hashes/counts.
  D2242–D2253 preserve all twelve measured natural deaths, reconciled.
  Rust/Python/format/vendor checks pass; P1.4-VALIDATION.md has evidence.
- P1.1–P1.3 merged (#234, #235, #237) after green CI. The owner explicitly
  authorized all remaining P tasks on 2026-11-15, including continuation
  beyond milestone review handoffs. Task PRs merge after CI; review PRs
  stay unmerged for the owner's chosen reviewer. P2 tasks #240–#243 and
  P3 tasks #244–#245 implement the remaining PLAN scope. No reviewer or
  population experiment is launched by this session.

## 2026-11-15: track P, P1.3 (#221), recovery parameters

- Added 37 recovery parameters (603 total): cure chances, HP/SP gates,
  danger limits/horizon, Unholy Reprieve's HP cutoff, rod-rest duration,
  and mana/food/depth rest thresholds. The zero-danger guard, full-state
  comparisons, status/charging flags and rest protocol remain categorical
  state/interface data; CALIBRATION.md lists them and every parameter.
- Full **45/45** matches P1.2 by the standard diff and pinned v12 by all
  play-log bytes. Original-core smoke is **3/3** and ladder parity holds.
  Default expansion reproduces the original executable tokens apart from
  the equivalent `R100` decimal-digit loop.
- `recover_rest_sp_tenths=10` changes Half-Elf Mage seed 1010 at turn 561.
  It dies of poison at 2600 while the default survives; the negative
  diagnostic result is retained, not recommended. See P1.3-VALIDATION.md.
- Core tests (133 passed), original-crate tests, Clippy, formatting,
  Python tests and vendoring check pass. D2202–D2213 retain every natural
  validation death; reconciliation finds none missing.
- This task is stacked on P1.2 PR #235; each task's PR retains its own CI
  merge gate. P1's remaining task is #222, the named decision trace.


## 2026-11-15: track P, P1.2 (#220), danger and escape parameters

- Added 366 parameters (566 total): per-blow/spell risk weights, fear
  adjustments, escape thresholds, spell-failure tolerances, sampling budgets
  and danger horizons. Defaults and operation order are preserved. Source
  locations are pinned to `30ab9d66`; CALIBRATION.md lists all names and
  the retained mechanics-model data.
- The source/engine audit distinguishes legacy estimates from facts: the
  melee hit powers differ from current blow_effects.txt, and PFE's factor
  2 approximates a level-dependent engine probability. Those weights are
  named, not corrected. The older landing-distance model remains frozen;
  source carry #595 is already flagged by the refresh report.
- Full **45/45** matches P1.1 by the standard diff and pinned v12 by every
  byte of the play logs; all shared game-summary fields agree. The known
  frontend counter/settings discrepancy is unchanged. Original-core smoke
  is **3/3**, and a 20,000-turn ladder matches P1.1 exactly.
- `danger_maximum=1` changes seed 1000's first key at turn 4430 and its
  state trace at the same turn. The escape-threshold trial on that seed
  has no play change; both results are recorded without an improvement
  claim. See [P1.2-VALIDATION.md](P1.2-VALIDATION.md).
- Rust tests, Clippy, formatting, Python tests and vendoring check pass.
  D2183–D2201 preserve the validation deaths; none are missing. The shared
  lock covers every build and game run. The read-only source refresh was
  rerun; no pins moved and no newer policy correction was carried.
- CI is the merge gate; its final result is recorded on the task PR.
  Remaining P1 work: #221 (recovery) and #222 (named decision trace).


## 2026-11-15: track P, P1.1 (#219), calibration foundation

- `borg-core/src/params.rs` defines 200 caution parameters with the v12
  defaults, units, inclusive ranges, descriptions and pre-extraction source
  lines (`726e04fa`). Initialization reads the environment over borg.txt
  once, rejects invalid input loudly, and writes `params-core.json`.
- The harness records its hash; dossiers preserve and hash the table, and
  include it in encounter identity. CALIBRATION.md lists the catalogue and
  the constants retained as game/representation data. Hooks are marked #219
  in `borg_caution.rs`, `borg_init.rs` and `lib.rs` for a future refresh.
- Main's errand fixes through `46d0b40e` were merged without changing their
  code; the later documentation/vendoring-tool update through `c2c4e61d`
  was also merged. The vendoring tool now recognizes params.rs as ours and
  caution as hooked; `check` passes.
- **Play parity:** full 45/45 against pinned borg-rs v12 `e6d13fc`, in
  keys/messages/states; all shared game-summary fields agree. The standard
  cross-repository diff still fails on three existing frontend telemetry
  fields (our frontend reports 0/0/{} for the vendored core), documented in
  [P1.1-VALIDATION.md](P1.1-VALIDATION.md). Full 45/45 against the earlier
  main baseline passes the standard diff. Original-core smoke is 3/3;
  a 20,000-turn ladder errand matches main exactly.
- **Override evidence:** seed 1000 Human Warrior, only
  `caution_flee_danger_hp_multiplier=0`: first key difference at turn 567,
  first state-trace difference at turn 577. The move_max_hp=0 trial did not
  change that seed and is retained as a negative functional check.
- **Dossiers:** D2141–D2182 retain the measured validation deaths, including
  intermediate builds and the donor suite's legacy-birth attempt. No natural
  death is missing from the final run outputs. The core table is preserved.
- **Remote CI:** Rust, engine and recorder passed on run `43658140254`;
  P1.1 merged in #234 as `30ab9d66`. The initial draft's missing CI run
  was resolved by the final push.
- Rust tests (the CI recipe: release for borg-core), Clippy, formatting,
  parser tests and Python tests pass. The initial baseline suite's timing
  aggregation failed because bc was missing; it was rerun intact after
  installing the utility locally. No suite condition was weakened.
- Remaining P1 tasks are #220 (danger/escape), #221 (recovery) and #222
  (the named decision trace). P2 and track R are not part of this change.

## 2026-11-14: two tracks, P (calibration) and R (the redesign epic)

- **Owner:** both threads, side by side: the calibration workaround in
  borg-rs's core (track P, epic #218) and the full redesign (track R, epic
  #217). PLAN.md has their milestones and exits; AGENTS.md *Choosing the
  work* says how a session takes a track (the owner names it; labels
  `track:P` / `track:R` and `P1` / `R1` ...).
- **In progress:** P1 (#219 parameter table and caution, #220 danger and
  escape, #221 recovery, #222 the decision trace) and R1 (#223 `borg-next`
  and the typed view, #224 the shadow harness, #225 the carry procedure).
  #209 (pilot C3) is now R2.1.
- **GitHub milestones** are not created for P and R (the session's tools
  cannot); labels stand in. M5–M7 are paused.
- **The review of option C as merged** (the vendored core, the errands, the
  re-pin) is with the owner: the prompt is pinned to `main` `ad428a39`.

## 2026-11-14: the engine re-pinned to upstream 42cd156ac (#215)

- **Every Borg plays upstream Angband** (owner). borg-rs v12's engine is
  upstream `42cd156ac`, 12 official commits after our `e473720`, plus one
  debug-mode fix (`cmd-wizard.c`); "borg-rs's engine corrections" in the
  entry below were mostly those upstream commits. `engine/angband/` is now
  `42cd156ac` byte for byte (docs/ENGINE.md).
- **Results.** borg-rs's core: 45/45 identical to borg-rs v12's build (it
  was 44/45). Our own core: smoke and smoke-exact unchanged, full 44/45;
  the druid of seed 1024 lives instead of dying at turn 41,864 (full
  reference regenerated, 32 dossiers of the new build committed).
- **PR #206's patch 08 and `tools/engine-patch.py` are obsolete.** #206 is
  to be trimmed to the evaluation documents.
- **Next:** the review of option C as merged (the vendored core, the
  errands, the contract changes, this re-pin); the owner picks the
  reviewer.

## 2026-11-14: errands on main, on e473720 (split from PR #206)

- **What came in.** borg-rs's decision core, vendored at milestone 12
  (`e6d13fc`), as `borg-core/`, and our errands over it
  (`borg-core/src/shell.rs`). docs/ERRANDS.md lists the commands, settings
  and rules; docs/SPEC.md §2 is amended for the vendored core only.
- **What did not.** `engine/patches/08-borg-rs-engine.patch` and
  `tools/engine-patch.py` stay in PR #206, with the evaluation
  (docs/REDESIGN.md, docs/ARCHITECTURE-COMPARISON.md, docs/PILOT.md). The
  engine here is still upstream `e473720`, so our own core and its
  references are unchanged.
- **The cost** (until #215 above). borg-rs's core plays on our engine, not its own: 44 of 45
  of the full suite's games are identical to a fresh build of borg-rs v12
  (keys, messages, states). The rest differ through engine differences
  (the end of a rest), not through the shell. When the shared engine
  repository is pinned (owner, another session), parity should be 45/45.
- **Open:** a shop errand; the casters' ladder pace; the zoo run (prompt
  with the owner); C3 (#209) not started; PR #206 still carries the
  evaluation and patch 08.

## 2026-11-14: the freeze, and the trip loop (#201)

- **"Alive" counted a Borg that had stopped playing.** On main (f0942d9a),
  38 of the full suite's 39 survivors walked between two grids on DL1 from
  about turn 15,000 to the cap. `tools/progress.py` now reports, per run and
  per suite: turns frozen, distinct grids, maximum depth and character
  level. **No survival figure is reported without it.**
- **The trip loop** (owner, 2026-11-14), replacing the HP gate:
  - in town, restock (and give unknown consumables to identify them);
  - take the stairs down to the target depth;
  - explore that level;
  - go home when it is explored or gives nothing for `level_stall_turns`
    (5,000);
  - repeat on a new level.
  The target depth is `max(1, clevel x depth_per_clevel_pct / 100)`
  (default 50). It replaces T5.1's readiness gate, which never opened DL2
  for a level-1 character. Class, race and power terms are for the zoo to
  sweep, as arms.
- **The travel guard is released** (e2575fc0, #183, owner): a step retreats
  only from half a hit point. Two loops it left are closed:
  - a refused grid is barred from the planner, target included, for
    `goal_cooldown_turns`, so the planner and the guard cannot take turns;
  - a stationary monster that one turn of melee is expected to kill (a grey
    mushroom patch) no longer turns travel back, unless the step risks
    death. A broader version, "any stationary monster", walked characters
    into floating eyes, molds and jellies (33 deaths) and was reverted.
- **Measured, full suite (45 runs):**

  | | main | this PR |
  |---|---|---|
  | alive | 39 | 13 |
  | alive and moving (frozen < 20,000) | 1 | 9 |
  | mean turns frozen | 72,000 | 7,667 |
  | mean max depth | 1.04 | 1.67 |
  | mean character level | 1.38 | 3.98 |
  | reached DL5 | 0 | 0 |

  An earlier arm of this PR reached DL5 once (warrior-0, level 8).
- **Deaths: 32 of 45 (main 6).** The owner chose to merge this as the
  baseline and work the deaths as their own issues. Causes: soldier 5,
  small kobold 3, giant white mouse 3, Fang 3, poison 3, floating eye 2,
  acolyte 2, and one each of Grip, cave spider, green naga, black naga,
  giant white louse, kobold, fatal wound, wild dog, battle-scarred veteran,
  apprentice and gallant.
- **Left:** four survivors still freeze, two of them `Alter(1)` digging
  loops. The references (smoke, smoke-exact, full) are regenerated at this
  build. Dossiers D1938-D2092.

## 2026-11-13: town sessions, the first step of the town plan (#191)

- **`borg-reborn-rs/src/town.rs`**, three switches, all off by default:
  - `BORG_TOWN_PLAYER=1`: in town the input hook yields (`Key::None`, the
    engine reads the keyboard) and the Borg decides nothing; outside the
    town it plays. Checked under Xvfb with xdotool: a new warrior, the
    player read Recall in town (`r`, Enter) and rested; the Borg's first
    think was on DL1 at turn 321, none in town, and it played 7,783 thinks.
  - `BORG_TOWN_STATE=FILE`: borg-rs's schema-3 state records, one per
    distinct town state. `traits` is `[]` and a pack item's value is -1
    (we have neither); the quiver is our addition under `x`.
  - `BORG_OBJECT_LOG=FILE`: numbered lines, borg-rs's format: buy, take,
    acquire, quaff, read, eat, throw, fire, wield, sell, store.
- **`harness/play.sh OUTDIR [N]`** (the X11 build, the three switches, the
  trace) and **`harness/feedback.sh`** (copied verbatim).
- **Play unchanged:** smoke and smoke-exact 3/3 identical with the switches
  unset, and smoke 3/3 identical with the state records and the object log
  on (the logs read, never decide).
- **Not checked:** the Borg arriving back in town and yielding (the session
  above did not last that long), and `feedback.sh`'s questions on a
  terminal (only its no-terminal path ran). The owner's first session is the
  check (#191).
- Dossier D1937 (CI's seed-1020 death, recorded for this build).
- **Next:** #192 (the curation and the state-purchase pairing) needs a
  session's files; #193 (the control baseline) and #194 (item use) do not.

## 2026-11-13: throw and wield a launcher (#127); the town plan (#191-#198)

- **Throw** (`v`, the item, `'`; `player-attack.c:do_cmd_throw:1363`): a
  known flask of oil is offered at the shot's target, scored like the shot,
  its damage the engine's for a THROWING item thrown by hand, dice x
  (2 + weight / 12) (`ranged_damage:568-579`): 7.5 on average. **An empty
  bow slot takes a pack launcher.** World gains `ammo_tval`,
  `has_launcher`. Measured: 7 throws chosen in the full suite, no launcher
  wielded (none found), totals unchanged (baseline 39/45, 43 of 45 runs
  identical; guard arm 35/45). The mechanics hardly fire because nothing
  buys the items: that is the town plan's job.
- **A hand-written ranged kit (buy 5 oil, a sling, 20 ammo) was written and
  dropped** (owner, 2026-11-13): a game visits the town hundreds of times,
  and fixed counts cannot serve every visit.
- **The town plan** (docs/TOWN-PLAN.md, issues #191-#198, for the owner's
  acceptance before any code): borg-rs's town inventory strategy (floors,
  a usefulness function, an exact solver, weight vectors as zoo arms; the
  control's and a human's choices as evidence), with **no lists**: every
  target is recomputed at every visit from the state through our threat
  model (the trip model, #195, validated against what runs consume before
  any solver uses it). The record schema is shared with borg-rs. Item-use
  mechanics for every usable kind (#194) run alongside.
- Dossiers D1931-D1936.

## 2026-11-13: Remove Hunger is food (#176, part); arm A's zoo run

- **Every remaining starvation ends with 1 gold** (a Ration costs 3): the
  character earns too little to buy food. The nature realm's Remove Hunger
  (`class.txt`, `NOURISH:INC_TO`; druid L2, ranger L3) feeds without gold.
  The observation marks a spell with `EF_NOURISH` (`Spell::nourish`). A
  learned one with its book counts as food (`has_food`), and the Eat step
  casts it when no ration is carried, resting for its mana first when
  resting is safe. A broke priest (no such spell) is still open in #176.
- **Measured:** guard arm 34 → **35** alive (starvation 2 → 1); baseline
  unchanged (39/45, 44 of 45 runs identical).
- Dossiers D1925–D1930.

**The zoo ran arm A** (`3113c2e2`, guard off) on classes-1800
(borg-zoo `results` branch, `results/reborn-g1-classes-1800/43501324055-1`).
Arm B has not run.

| (1,800 cells) | alive | reach D=5 | reach DL2 | mean max level |
| --- | --- | --- | --- | --- |
| pristine C control (e473720) | 66% | **6.9%** (Warrior 34%) | 38–93% by class | 2.6–6.6 |
| arm A | **86%** | **0.0%** in every class | 0–2% | 1.1–1.5 |

89% of arm A's survivors gain no level and no gold between turns 25,000
and 100,000: the travel guard's freeze (#183). No character is ready for
DL2 under the readiness gate (worst turn of DL0–7 natives under half max
HP; Grip and Fang are in the band). G1's verdict waits for the owner's D
and margin; at D = 5 arm A is below the control in every class where the
control is above the margin. The readiness gate is the next design
question, to be measured as its own arm.

## 2026-11-13: status blows deal their dice; no exploring under pursuit (#184)

- **Status and economic blows deal their dice** before their effect
  (`mon-blows.c:melee_effect_timed:507`, `EAT_GOLD:781`). The model gave
  them 0, so a grey mushroom patch's `SPORE:CONFUSE:1d4` killed level-1
  characters at p_death 0.
- **bbc-d0006 had the wrong monster.** It placed race 96, the brown mold
  (a never-mover), where its construction and the dossier's killer are a
  killer brown beetle (race 178, `overall-assessment.md:5-7`). It passed
  only through the retreat guard's reaction to the mold. With the pursuer
  corrected, the Borg walked north into the unknown, D0006's fatal move.
- **No exploring under pursuit:** a frontier step while an awake mover that
  can hurt could reach the Borg within two player turns (c15 DANGER-08's
  horizon) becomes the retreat step into known ground. D0006 passes on it.
- **Measured:** guard arm 32 → **34** alive (depth 47, level 99 → 110);
  baseline 40 → 39 (depth 46 → 47, level 64 → 62); smoke 3/3 (level
  3 → 5). Guard-arm killers: a small kobold ×3, starvation ×2, then single
  causes.
- Dossiers D1919–D1924.

**The zoo arms for #128 and #183** are fixed commits in main's history:
A (guard off) `49c48bbf`, B (guard on) `2f72e80e` (#187). The prompt is in
the session's scratchpad and in #183's comments.

## 2026-11-13: two starvations of the freed Borg (#183, part)

- **A stalled urgent goal is cooled.** The urgent rules (starving with no
  food, a breeder colony, not ready) returned Return before the
  hysteresis check, and only that check sets a cooldown: full-6-rogue-1
  walked between two grids in "starving with no food" for 7,000 turns,
  its goal unchanged since 78,804. `choose_mode` now cools a stalled goal
  (its budget, `budget_of`) before any rule picks its mode again.
- **A monster in the way is walked round.** A grid a visible monster
  stands on costs ten steps to path through (declared). full-4-necro-3
  bumped Farmer Maggot 400 times on its way to a shop and starved with 195
  gold.
- **Measured:** guard arm 34 → 32 alive (depth 47, level 104 → 99; starvation
  3 → 2, the rest reshuffled); baseline 40/45 (depth 46, level 62 → 64);
  smoke 3/3.
- **The tail is now below the suite's resolution.** The guard arm's deaths
  are ten causes of one or two each: a fatal wound ×2, starvation ×2, a
  small kobold ×2, then one each of poison, a giant yellow centipede, a
  soldier ant, Fang, a grey mushroom patch, a giant white mouse, a
  floating eye. ±2 on 45 runs is noise; the zoo's classes-1800 is the
  measurement (the prompt is in #128 and #183).
- **The grey mushroom patch is #184.** Its `SPORE:CONFUSE:1d4` deals 1d4 a
  hit in the engine and 0 in the model, so a confused level-1 character
  walks away at random beside it at p_death 0.0.
- Dossiers D1914–D1918. The zoo's guard-on arm commit (main + e2575fc0) is
  pushed and reverted on this branch for every arm the owner wants.

## 2026-11-13: what kills the freed Borg, first four fixes (#183, part)

**The owner's decision on #183: option 1.** Keep the travel guard's
freeze, fix what kills the freed Borg first, then release the guard. Each
fix is measured twice: on the baseline (the committed code, for the
references) and with the guard (e2575fc0) applied as a scratch arm
(`git show e2575fc0 -- borg-reborn-rs/src/strategy.rs | git apply`, run
into `/tmp`, deaths recorded outside `docs/deaths`).

| fix | freed Borg (guard arm) alive / depth / level | baseline |
| --- | --- | --- |
| trial (guard only) | 24 / 48 / 121 | 41 / 45 / 67 |
| floating eye: rest on the composite, the gaze's floor | 29 / 48 / 119 | 41 / 45 / 67 |
| breeder colony: leave the level | 33 / 46 / 89 | 40 / 45 / 66 |
| confused beside lava: wait it out | 33 / 46 / 91 | 41 / 45 / 67 |
| a harmless monster in reach is not a fight | **34 / 47 / 104** | **40 / 46 / 62** |

- **Floating eye** (D1885, D1891, D1893): `may_rest` read only the HP
  risk, and the diceless gaze counted 0. Resting reads the composite
  (disabling included), and an unprotected PARALYZE gaze costs at least 1
  a hit (`mon-blows.c:1019-1021`). Counting every status blow's dice is
  engine-correct but breaks `bbc-d0006`, which passes only through the
  retreat guard: filed as #184.
- **Breeder colony** (the giant white mouse deaths): `breeder_colony`
  (default 4, declared) live `MULTIPLY` tracks on the level and nothing in
  reach send the Borg home before it rests (c08 §8.10, c15 DANGER-08).
  It costs levels in the guard arm (leaving levels).
- **Lava:** confused, a step is random 75% of the time
  (`player_confuse_dir:1358`). Beside `FEAT_LAVA` (23) a walk outside a
  fight becomes standing still. full-1-mage-4 burnt 11 HP to −150.
- **Harmless contact:** the contact rule needs the monsters in reach to
  carry composite threat. full-4-necro-3 attacked Farmer Maggot 493 times
  and starved with 195 gold.
- **Remaining killers in the guard arm:** starvation ×3, grey mushroom
  patch ×2, giant yellow centipede ×2, a fatal wound, a soldier ant, a
  small kobold, a floating eye. One starvation is a Return that never
  reaches the stairs (rogue-1).
- Dossiers D1896–D1913 (the baseline runs of each commit).

## 2026-11-13: a frontier grid stood on is not walked back to (#176, part)

**Play change in fifteen full runs; new references (smoke runs identical,
only their identity fields change).**

- **The walk in place, explore half.** A frontier grid (known floor next to
  an unknown one) was excluded only while underfoot. In the dark, in the
  unlit town at night or for an `UNLIGHT` necromancer, standing on it does
  not reveal the unknown neighbour. So two such grids took turns as the
  nearest target, and the Borg walked between them: #172's ranger-0 (Walk
  9/1), #176's priest-4 (Walk 3/7 in the night town), #177's necromancer.
  The world now marks every grid the Borg stands on (`Remembered::stood`,
  per level), and the frontier skips them. A path test pins it; it fails
  without the exclusion.
- **Measured (full, 45 runs):** alive 40 → **41**, max depth sum 43 →
  **45**, max level sum 63 → **67**. No run is worse. necro-3 survives;
  mage-0, paladin-0/4, druid-4 and blackguard-3 gain a level or a depth.
  Smoke unchanged.
- **Not fixed: priest-4 still starves in town.** Its last 20,000 turns are
  now Descend, "reach the down staircase", flipping Walk(1)/Walk(9) between
  (22,18) and (23,17) with no monster in the observation and no threat.
  The town there has lava (feature 23) and passable rubble (24). The down
  staircase is not in the panel. A fixed distance map cannot flip with no
  threat, so either the map changes between the two grids (the night-town
  memory) or `travel_step`'s retreat guard fires. It needs a replay of the
  town's observations from arrival (75,531). #176 stays open for that and
  for the no-gold half.
- Dossiers D1870–D1873.

**The town flip, found and held back for the owner (#183).** A replay of the
night town with the engine's full knowledge shows the down-stair path
stable (Walk 9 at both grids). `travel_step`'s guard retreats whenever the
next grid's composite risk is higher at any size, and a squint-eyed rogue
last seen 8,000 turns earlier put 0.0004 on the next grid. A guard at the
path's own resolution (half a hit point) was tried (e2575fc0) and
reverted. It frees every character the flip held: full alive 41 → **24**,
max depth sum 45 → 48, max level sum 67 → **121** (a warrior reaches
clevel 10); smoke 3 → 2. Much of the suite's survival was this freeze.
Survival against progress at that scale is the owner's call. Dossiers
D1874–D1895 are that run’s deaths.

## 2026-11-13: a monster at 0 hit points is alive (#179)

**Play change in twelve full runs; new references for all three suites.**

- **The cause of #179** was not darkness or stale tracks. The observation
  of `full-6-rogue-2` at 4,154 shows two white worm masses adjacent, in view
  and lit, each at **0 HP**. The engine kills a monster only below 0
  (`mon-util.c:mon_take_hit:1299-1300`). Attack, shot and cast skipped
  `hp <= 0` as dead, so no attack was offered and the rogue held while the
  worms poisoned it. All three filters now read `hp < 0` as dead. A test
  pins it; it fails on the old filter.
- **Measured.**
  - **Full (45 runs):** alive 39 → **40**, max depth sum 44 → 43, max
    level sum 63 → 63. Gained: rogue-2. Changed: mage-4 now dies of a cut at
    3,070 (it melees a giant white mouse swarm to 0 HP; the bleeding-out
    rule waits for no monster in reach) and necro-2 of poison at 3,860.
    Paladin-2 reaches DL1 instead of DL2.
  - **Smoke:** unchanged, 3/3.
- Dossiers D1865–D1869.

**What #179's other questions found:** the stale low-confidence tracks
were not why no attack was offered; the hp filter was. Whether stale
breeder tracks should cost a step its full free hit stays open as a
question for the danger model. Melee against a breeder swarm at low HP
(mage-4) is the same family as #176/#177's oscillations: a decision made
one think at a time with no exit plan.

## 2026-11-13: poison and cuts bleed the hit points left (#175)

**Play change in four full runs and one smoke run; new references for all
three suites.**

- **`danger::pending_bleed`:** the HP the poison and cut timers already
  on the character will take. Each player turn costs 1 for poison and 1–3
  for a cut by grade (`game-world.c:513-541`); the timers fall by
  `adj_con_fix[CON] + 1` (`decrease_timeouts`, the table lifted from
  `player-calcs.c:686-725`); nothing regenerates meanwhile
  (`player-util.c:468-470`); a Mortal Wound never closes. `p_death` reads
  the HP left after it (`hp_of`).
- **A POISON blow costs its timer:** `5 + 1d(rlev)` turns
  (`mon-blows.c:683-684`) go into the expected and worst loss, not into
  this round's death chance, unless poison is resisted.
- **Bleeding out:** when the timers would take every HP left, the Borg
  drinks Neutralize Poison (when poisoned) or the strongest known Cure
  Wounds potion before any step but a monster in reach.
- **Cure Light Wounds heals 20** (`object.txt` `HEAL_HP` `dice:20`;
  `effect_handler_HEAL_HP:212-217`), not the 2d8 the tactics declared.
  The fear fixture now also accepts drinking it at 8/33 HP, with the reason
  in its `limits`. Its rule, no attack while afraid, is unchanged.
- **The status share is counted once:** attack, shot and cast added the
  struck monster's status part to `current_risk`, which already holds it
  (`composite`). Fixed, with a test that an adjacent breeder is attacked
  rather than ignored. It changed no suite run.
- **Measured.**
  - **Smoke:** 2 → **3** alive. smoke-mage-2 no longer melees itself into
    poison.
  - **Full (45 runs):** alive 41 → **39**, max depth sum 44 → 44, max
    level sum 60 → **63**. Lost: mage-4 (a giant white mouse at 7,893, now
    clevel 5) and rogue-2 (poison at 4,220, Hold among unseen worm masses,
    #179).
  - **The experiment:** with the poison-blow term switched off, the smoke
    mage dies as before and both full runs survive. The term is the part
    that matters, both ways. It follows the engine, so it stays. The Hold
    it exposes is filed as #179.
- Dossiers: D1853–D1858 (the first commit's run), D1859–D1864 (the
  baseline).

## 2026-11-13: study and cast attack spells (#127, T5.3, second slice)

**Play change in 30 full runs and one smoke run; new references for all
three suites.**

- **Observation:** each spell's failure chance as the spell menu shows it
  (`spell_chance`, the "Fail" column), whether it aims (`spell_needs_aim`),
  and its first damaging effect with the dice evaluated for the level
  (`dice_random_value`). The menu's "Info" column is **not** read:
  `get_spell_info` calls `dice_roll`, which draws from the game's generator
  (`z-dice.c:579-592`). The status gains `new_spells` (the "Study"
  indicator). The bindings add `spell_chance`, `spell_needs_aim`,
  `dice_random_value` and `effect_index` (bindgen 0.73.2 installed with
  `cargo install bindgen-cli --version 0.73.2 --locked`); docs/ENGINE-READS.md
  lists them.
- **Actions:** `Study { book, letter }` is `G`, the book, then the spell's
  menu letter for a `CHOOSE_SPELLS` class (`ui-spell.c:251`,
  `all_letters_nohjkl` by position in the book). A priest or paladin learns
  the engine's random pick (`do_cmd_study_book`). `Cast { book, letter, aim }`
  is `m`, the book, the letter, and `'` (the closest target) when it aims.
  Both use the turn.
- **`spells.rs`:** a chooser studies an attack spell first, then the lowest
  level. The castable attacks are the target-centred effects (BOLT,
  BOLT_OR_BEAM, BALL, STRIKE: full damage at the target; arcs, short beams
  and lines are not used yet). Expected damage is the dice mean × (1 −
  fail) × the target's resistance (`project-mon.c`: nether 0 on undead,
  0.33 on IM_NETHER, 0.5 on evil; poison 1/9 on IM_POIS). A cast is never
  made without the mana. `can_cast` is `player_can_cast`: grid seen, not
  blind or confused.
- **Tactics** offers the best spell at the shot's target, scored like the
  shot. **Strategy** studies on a quiet grid, after the light and before an
  equipment swap. **Readiness:** a learned attack spell whose book is in
  the pack is a ranged attack (the DL5 gate).
- **Measured (full, 45 runs):** alive 39 → **41**; max depth sum 44 → 44;
  max level sum 54 → **60**. 40 spells learned, 1,134 casts chosen, 11
  failed concentrations. Gains: mage-4 (was poison at 1,390), druid-0/1/2,
  priest-0 now survive, and several runs gain a level. Losses: necro-0 (a
  small kobold at 5,078), necro-2 (a cave spider at 3,347), necro-3 (Fang
  at 80,541), all fights the model scored at p_death 0 until the last turn.
  Priest-4 starves in town (#176). **Smoke:** smoke-mage-2 now dies of
  poison at 15,220 (rests poisoned at p_death 0, #175). Smoke 3 → 2 alive.
- Dossiers: D1848 (smoke mage), D1849–D1852.

**Filed:** #175 (poison over time missing from p_death), #176 (starving in
town at night with no gold, walking in place), #177 (an unlit
necromancer explores in place).

**Left in #127:** devices with failure chances, thrown oil, priest
detection timing, arcs and beams, D0014's Speed→CLW.

## 2026-11-13: eat when Hungry, light the way (#172)

**Play change in 39 full runs; new full baseline. Smoke and smoke-exact
unchanged.**

- **`eat_below_food` 300 → 1500.** The engine's grades (`player_timed.txt`
  × `food-value` 100): Faint 400, Weak 800, Hungry 1500, Fed 9000. 300 was
  below Faint, so the Borg could faint before it ate. The manual (c01
  §1.16) eats when Hungry appears. The return-home rule for "starving with
  no food" reads the same threshold, so it now starts with 1,500 turns of
  food rather than 300. The fixture default `fed()` was 500 (Weak); it is
  5000 now.
- **The starvations' root cause was darkness, not the food rule.** A
  burnt-out torch is deleted from the light slot
  (`player-util.c:player_update_light:713-726`, `OF_BURNS_OUT`) and nothing
  wielded the spares: **all 45 full runs lost their first torch**. The
  runs that starved (ranger-0, rogue-1) spent their last ~25,000 turns
  unlit, with two Wooden Torches and a Word of Recall in the pack. Unlit,
  `player_can_read` refuses the Recall, and the unknown grid next to the
  Borg is never seen, so Explore and Return flipped between two grids
  (Walk 9 / Walk 1) until the food ran out. Found from a rerun to turn
  75,340 with `BORG_OBSERVATION_LOG`: the light slot empty, the panel
  grid (26,39) unknown beside a floor.
- **`equipment::light_to_wield`:** an empty light slot takes the aware pack
  light with the most fuel, before every step but a monster in reach (not
  gated on a quiet grid: unlit is itself the danger). 44 such wields in
  the suite, none refused.
- **UNLIGHT is left unlit.** The necromancer (`class.txt:975`) sees nearby
  grids without light (`cave-view.c:777-779`) and pays +25 spell failure on
  a lit grid (`player-spell.c:416-419`). Lighting it cost all four
  necromancer runs depth or a level in a scratch run. `Class` gains the
  engine's `pflags` (`Class::has_pflag`, `PF_UNLIGHT` = 12).
- **Measured (full, 45 runs):** alive 34 → **39**; max depth sum 44 → 44;
  max level sum 52 → 54; no run worse. No starvations (was 2). The
  remaining deaths: mage-4 poison at 1,390; druid-0 and druid-1 a small
  kobold; druid-2 a crow; priest-0 a giant white mouse; priest-4 a
  battle-scarred veteran. All are early casters who do not cast yet (#127).
- Dossiers: D1842–D1847 for the baseline. D1828–D1841 are the deaths of the
  two intermediate commits of this PR (each a committed source, RULES 12).

**Defect 1 of #172** (Return must outrank a live explore goal) did not
reproduce once the light was fixed. The return-home rule fires in three
full runs (necro-3 at 76,810, rogue-1 at 75,801, ranger-0 at 75,302) and
each reaches town within 250 turns, where Town mode takes over. It is not
changed here.

## 2026-11-13: shooting and Speed (#127, T5.3, first slice)

**Play change in three full runs (the rangers); new full baseline.**

- **`Action::Shoot`** is `h`, `do_cmd_fire_at_nearest`
  (`player-attack.c:1412-1443`): the first quiver ammunition the launcher
  takes, at the closest visible monster, with no prompt.
  `Character::shot` reads the launcher's ammunition type (`KF_SHOOTS_*`,
  10/11/12) and multiplier (`pval`) from its kind
  (`player-calcs.c:2265-2275`) and the damage as `ranged_damage` does
  (`:554-582`). The tactical response scores the share of the target's
  threat one shot removes; at range, that is the round it deals once
  adjacent. Never-movers at range are not shot; blind, confused or
  paralysed, no shot. One shot a turn; the hit chance is not modelled
  (declared, as for melee).
- **A Potion of Speed** (+10, `player-calcs.c:2176-2177`) is offered at
  the threat the same grid has at speed + 10, for one consumable.
- **bbc D0010 is rebuilt from its dossier** (`overall-assessment.md:16`:
  Main Gauche wielded, Short Bow x2 in the bow slot, eleven 1d4 arrows in
  the quiver). The fixture had the bow as the weapon, the arrows in the
  pack and wrong kinds. It now uses the engine's kinds 34/77/81/247 (added
  to the fixture knowledge from a `BORG_KNOWLEDGE_LOG` dump), and is
  active: the Borg shoots the kobold. The fixture loader gains a quiver.
- **D0014** now offers Speed, but the dagger's one-hit kill of the 6-HP
  falcon outscores it, so it stays a gap (#127): the owner's Speed→CLW
  line is a sequencing question.
- The readiness ranged gate uses `Character::shot` (ammunition the
  launcher fires).
- **Measured:** 9 shots in the full suite, none refused. `full-7-ranger-4`
  survives (was a floating eye at 3,880); `full-7-ranger-3` survives;
  `full-7-ranger-0` now starves at 88,180 — the starvation residue,
  filed as **#172**. Survival 34/45. Full reference regenerated; dossiers
  D1817–D1827.

**Left in #127:** spells (study, cast, Magic Missile's evaluation, priest
prayer randomness, book priorities), devices with failure chances, thrown
oil, and D0014's Speed→CLW sequence.

## 2026-11-13: equipment valued by effect (#126, T5.2)

**Play change in two full runs; new full baseline.**

- **The engine's tvals** (`world::tv`, `list-tvals.h`). The observation
  carries the engine's own tval, but `tactics::better_weapon` filtered on
  23, the pre-4.2 sword, which in 4.2 is hard armour: no real weapon was
  ever offered. The test data and all 24 decision fixtures used the old
  numbers and are corrected (dagger 9; mace, maul, whip 7; bow 5;
  mushroom 29).
- **No wield ever worked in a game.** The prompt catcher escaped "Wear or
  wield which item?" unconditionally, the C Borg's rule for stumbling
  into it. Our Wield opens that prompt itself with the item letter
  queued, so the letter fell on the map as a pickup. The prompt is now
  escaped only when nothing is queued, and a refused wield is barred like
  any refused action (#137). Before this fix the scratch suite had two
  runs with about 17,600 refused wields each, both starving.
- **`equipment.rs`:** a pack item is worn when, on a copy wearing it,
  melee per turn, known AC and (for a caster) the armour-weight mana
  penalty (`calc_mana:1505-1531`) are all no worse and one is strictly
  better. A launcher is not a weapon; jewellery is worn only when fully
  known and only into an empty slot. The strategy wears it when the grid
  is quiet and nothing is adjacent (rule "a better item in the pack:
  wear it").
- **D0011 is active:** the fixture's Maul is the dossier's 4d4 (+7,+6)
  (`overall-assessment.md:26`), not 2d4. The loader carries a pack
  weapon's numbers, and the Borg wields the Maul. OWNERS drops #126.
- **Measured:** smoke 3/3 and smoke-exact 3/3 unchanged; full 43/45.
  Two wields in the whole suite, both worn. `full-0-warrior-3` wears a
  Cloak [1,+0] and survives as before; `full-2-druid-0` diverges after
  its swap and dies to a small kobold at 3,098 (was the mouse swarm at
  7,912). Survival 34/45. Full reference regenerated; dossiers
  D1806–D1816.
- **Not valued:** resistances, slays, brands. Pickups and pack weight are
  unchanged (a swap does not change total weight). The town buys no
  equipment, so reserves come first by construction.

**Next:** #127 (T5.3 item use, spells, ranged).

## 2026-11-13: depth readiness (#125, T5.1)

**Play change; new baseline.** One readiness policy for every way down
(`readiness.rs`, from the strategy manual):

- **DEPTH-01:** depth *d* is allowed when the worst single turn
  (`danger::worst_turn`, c15 DANGER-01's `T_max`: moves times the larger
  of the full melee round and the largest spell) of every race native to
  *d*−2 … *d*+5 stays below `depth_hp_frac_pct` (50) of maximum hit points.
- **DEPTH-02:** the DL5, DL15 and DL20 kit gates (Recall, three cure
  potions, a ranged attack; 100 max HP and ten Phase Doors; Free Action).
  Kinds are matched by name *and* tval: Cure Light Wounds is also a staff,
  and a flask is tval 27. The DL25+ rows are not checked yet and fail.
  DL1 is always allowed.
- **Strategy (DEPTH-04):** not ready for the current depth → Return (go
  up); ready for depth+5 → dive before exploring; a down staircase only
  when ready for the next depth, otherwise an explored level rerolls
  upward under the existing rule.
- **Not covered:** trapdoors (the observation has no trap kind; a fall to
  an unready depth is answered by "go up"); Deep Descent (target
  `max_depth` + 5, but the Borg reads none today); a recall down (never
  read).
- **Declared parameters:** `depth_hp_frac_pct` 50 (the manual's
  `K_DEPTH_HP_FRAC`; its 0.3/0.5/0.7 sweep is not run),
  `readiness_band_below` 2, `readiness_band_above` 5.

**Measured (the declared change), full suite:**

| | before | after |
| --- | ---: | ---: |
| alive at the cap | 30/45 | **34/45** |
| mean max depth | 1.20 | **0.98** |
| mean max character level | 1.49 | **1.20** |
| deepest level | 3 | 2 |

The gate trades progress for survival. With the band reaching *d*+5,
Fang (DL5, +20, a 1d8 bite: 16 in one turn) keeps DL2 closed until about
33 maximum HP. On DL1 the Borg gains little experience, so most
characters never get there. "Not ready: go up" fired once in the suite
and "dive" never; the whole effect is the down staircase not taken.
`full-6-rogue-1` now starves on DL1 (the T4.6 residue: a Return goal that
never finds the way up, 1,100 turns, with 69 gold). The smoke mage
survives (it was Fang at 18,376). References regenerated (smoke 1,
smoke-exact 1, full 13 runs changed); dossiers D1795–D1805.

**For the owner:** progress below DL1 now needs experience the Borg does
not yet seek. #126 (equipment) and #127 (item use, spells, ranged)
should raise it; `depth_hp_frac_pct` and the band are the dials if the
gate is too strict, to be measured, not tuned by hand (RULES 14).

**Next:** #126 (T5.2 equipment and pack valuation).

## 2026-11-13: M5 source refresh (RULES 21)

**No play change.** M5 starts with its `--update` PR.

- **Pins moved:** borg-rs `4ed3263`, better-borg-c `4603703`,
  neo-angband-tracker `c2ddf2c`, borg-zoo `c5f1b04` and borg-runner
  `e55ac6f`. The copies in `reference/` and the death index are refreshed.
  Upstream Angband stays at `e473720` (the engine contract).
- **borg-agent retired:** its pin is dropped, with its mentions in RULES 7
  and 21 and the refresh script's docstring (left for this PR by #152).
- **Read:** recorded in `docs/research/KNOWLEDGE.md` (reading-state rows
  and an "M5 source refresh" note).
  - **borg-runner edition 7:** readiness is the destination's worst
    round, not its experience (`mmf-mage.3`); Deep Descent is `max_depth`
    + 5 (`effect-handler-general.c:1166-1168`); class kit facts for
    Priest, Ranger and Mage.
  - **better-borg-c:** no new deaths. `borg_guess_race` moved to lines
    874-889, so `observation.rs` cites the new lines.
  - **borg-zoo:** our milestone-E specimen is catalogued, there is a
    pristine `classes-1800` rerun (a G1 control candidate, not yet read),
    and the `clevel` stop mapping (borg-zoo#48), so the stale note in
    `docs/ZOO-INTERFACE.md` is corrected.
  - **borg-rs:** v11R carry; munchkin default back to FALSE.
- References reproduce 3/3, 3/3, 45/45; dossiers D1779–D1794 (the new
  source hash).
- The issue sections of the refresh report fail from cloud sessions
  (GitHub GraphQL is refused there); issues were not read on this
  refresh.

**Next:** #125 (T5.1 depth readiness).

## 2026-11-13: the M4 review's notes (#165)

**No play change** (measured): smoke 3/3, smoke-exact 3/3, full 45/45
identical to the references.

- `danger.rs`: sleep and confusion set the rounds a monster spends
  attacking, and expected, worst, `p_death`, `disabling` and `economic`
  all read them. Before, confusion scaled only expected and worst, and
  sleep left `disabling` whole. Stun is applied where the engine applies
  it: the hit chance falls by `STUN_HIT_REDUCTION`
  (`mon-attack.c:353-356`) and the rolled dice by `STUN_DAM_REDUCTION`,
  before armour (`:621-624`). Test: `confusion_and_stun_scale_every_term`.
- `strategy.rs`: `has_food` and the Eat step require an aware item (test
  `only_known_food_is_food`). The refusal bar's documentation names its
  granularity: an exact `Action`, pack index included; bar by kind if
  item refusals recur.
- `tools/calibrate.py`: only the engine's melee blow phrases
  (`blow_methods.txt` `act:` lines) and "misses you" count as blows. The
  blow decomposition is unchanged on this baseline (no spell line had
  matched).
- Dossiers D1763–D1778; `check-no-time.py` 0 violations.

**Next:** the M5 source refresh (its own PR), then #125.

## 2026-11-13: a wide blow is a distribution (#164)

**No play change.** M4 review finding 2. `danger.rs:blow_pmf` returned
`[0, mean]` for blows with dice × sides > 64: probability mass `mean` at
damage 1, which is not a distribution, so `p_death` came out clamped to 1.
The declared two-point model is now a real one: P(max) = mean / max and the
rest on 0. It sums to 1 and keeps the mean; the spread is overstated, which
is conservative. Only depth ≥ 36 monsters have such blows (20d10 at most).
Test: `a_wide_blow_is_a_distribution`. References reproduce 3/3, 3/3,
45/45; dossiers D1747–D1762.

## 2026-11-13: M4 reviewed and merged; growth factors are declared (#163)

**M4 review.** GLM-5.3 reviewed #162 at `612fa97`. It reproduced every
reference, the G0 checks, all 16 dossiers and every calibration number,
and gave the verdict: contract met, nothing blocking. The owner merged it
(`reviewed` advances by merge commit `ae5aa11`). **The `m4` tag could not
be pushed from the cloud session** (the proxy refuses tag refs and the tag
API); the owner creates it at `ae5aa11`. The findings are M5 issues #163,
#164 and #165.

**#163 (no play change):** the growth factors' citations are corrected.
The C Borg's code at `borg-danger.c:borg_danger_one_kill:2420-2436,
2700-2710` computes ×2.5 (breeders), ×2.8 / ×2.3 (monsters with friends)
and ×2.2 (its second pass). Its comments say +50%, +80% and +30%. Our 1.5
and 1.8 follow the comments; the 1.5 for summoners from CL15 is ours; the
C Borg has no summoner factor. `danger.rs` and the calibration report's
threshold table now call them declared. References reproduce 3/3, 3/3,
45/45; dossiers D1731–D1746 (the new source hash).

## 2026-11-12: remembered casters fade with confidence (#157, T4.8)

**Play change in one full run; new full baseline.**

- `danger.rs:one_threat` scales the ranged half of a monster's threat by
  `Track::confidence`, as it already scaled the melee half. It also scales
  `p_death` in both halves: the chance the monster is there times the
  chance it kills if it is. The melee `p_death` was unscaled too, a
  sibling of the same defect, fixed together here. Test:
  `a_forgotten_caster_adds_almost_nothing`.
- **Measured:** smoke 3/3, smoke-exact 3/3; full 44/45 identical.
  `full-6-rogue-3` leaves its 75,000-turn loop around three forgotten
  apprentices, reaches character level 3 and dies to Fang at 25,237
  (p_death 0.93 at the fatal think; it fought on at 1 HP). Full survival
  is 30/45, the T4.7 rate. The full reference is regenerated; dossiers are
  D1715–D1730.
- **Calibration:** pooled ratio back to 0.083 (it was 0.017 with the loop);
  remembered out-of-view sources 893 windows at 0.015. Addendum in
  [docs/M4-CALIBRATION.md](M4-CALIBRATION.md).

**M4's tasks are all closed with this PR.** Next is the M4 review PR
(`main` → `reviewed`), opened by the session with a review prompt. The
owner chooses the reviewer (RULES 6).

## 2026-11-12: the M3 review's small findings (#139)

**Play change in three full runs; new full baseline.** Five cleanups:

- `tactics`: responses and the weapon swap sort with `total_cmp`, through
  `ord()`, which folds -0.0 into 0.0. Ties then fall through to risk, gain
  and action rank exactly as under `partial_cmp`.
- `strategy`: Engage's progress measure has its own sentinel
  (`ENGAGE_NO_CONTACT`) when no monster is adjacent. Before, the empty case
  was 0, which reads as a monster at 0 HP; a track's `hp` is 0 when its
  health is unknown, so "no contact" and "contact with an unmeasured
  monster" looked like no progress.
- `strategy`: Explore's fallback stands still through `still()`, which
  never opens a shop by holding on its entrance.
- `executor`: Sell sends quantity digits and the prompt's Enter only for a
  stacked item. A single item has no quantity prompt, and digits there
  would be read as commands.
- `strategy`: `retreat_step`'s documentation states its tie-break: the
  first direction in `DIRS`, keypad order.

**Measured:**
- Smoke 3/3 and smoke-exact 3/3 are unchanged. Full: 42/45 unchanged.
- Bisected on seed 1002, the Engage sentinel alone accounts for all three
  differences: `full-0-warrior-2`, `full-0-warrior-3` (both still survive)
  and `full-2-druid-0` (dies to the same giant white mouse swarm at 7,912
  instead of 6,943). An Engage goal now keeps running while monsters come
  and go, instead of stalling.
- Full survival stays 31/45. The full reference is regenerated; dossiers
  are D1700–D1714. `check-no-time.py` finds 0 violations and
  `reconcile-deaths.py` is clean.
- Regression tests: `scores_are_totally_ordered_and_zero_has_one_sign`,
  `engage_progress_without_contact_is_not_a_dead_monster`, and the
  single-item Sell keys.

**Next:** #157 (remembered casters never fade), then the M4 review.

## 2026-11-12: the trace says why, and only scored candidates (#138)

**No play change; same baseline.** The M3 review's finding 3. The first
strategy chooses its mode by a priority rule, so the trace's single
candidate at score 1.0 claimed an alternative that was never weighed.

- `candidates` now holds only scored alternatives, which are the tactical
  responses of a contact think (T4.3). A rule-based think records none
  (in `full-0-warrior-0`, 9,385 of 9,522 thinks).
- New trace field `why` (`trace::Why`): the `choose_mode` rule that fired
  (for example "contact: a monster in reach", "a live goal keeps its
  mode"), the modes cooling down, and the goal's `goal_started` and
  `goal_progressed`. `Goal.started` is now read: age is
  `turn - goal_started`, stall is `turn - goal_progressed`. Old traces
  parse (serde defaults).
- Checked: smoke 3/3, smoke-exact 3/3, full 45/45 identical to the
  references; `check-no-time.py` 0 violations. The new source hash gives
  the 15 deaths new dossiers, D1685–D1699 (same games as D1670–D1684).
- **Next:** #139 (small cleanups), #157 (remembered casters), then the M4
  review.

## 2026-11-12: a refused action changes the next decision (#137)

**Play change; new full baseline.** The M3 review's finding 2: contact
returns Engage before any cooldown, so an action the executor reported
refused was chosen again. In the M4 baseline the refused action was a Read
the engine forbids.

- **`danger::can_read`** mirrors `player_can_read`
  (`player-util.c:1166-1197`): blind (`TMD_BLIND` 2), confused (4), amnesiac
  (25), or no light. No light is `no_light`, `!square_isseen` at the
  character's grid (`cave-view.c:932-935`); the observation carries it as
  `in_view` (`cave-map.c:119`) and the world as `seen == Some(turn)`. It
  gates the tactical Phase Door, Teleportation and Recall, and the
  Return/Stuck recall read.
- **Refusals bar the action:** the strategy remembers its last action, and
  a refusal bars that action until the mode's cooldown ends. Engage then
  takes the next-best response. Contact still overrides Engage's cooldown,
  which is the justification the issue asked for.
- **Measured:** two full runs change, and smoke and smoke-exact are
  identical. `full-6-rogue-3` survives the cap (was D1667: confused, it
  re-read a refused Phase Door for forty turns before a cutpurse killed
  it). `full-7-ranger-0` no longer reads Recall in the dark 13 times, and
  still starves at the same turn. Refused-read violations: 23 → 0. Full
  survival is 31/45. The full reference was regenerated at `798239a`, and a
  fresh run reproduces it 45/45 (smoke 3/3 against the kept references).
  Dossiers D1670–D1684; `reconcile-deaths.py` is clean.
- **Found: #157.** The rogue's survival is a 75,000-turn walk loop around
  three remembered apprentices. `one_threat` scales melee by track
  confidence but not the ranged estimate, so a forgotten caster keeps
  `expected 1.8`, `worst 14.4` at confidence 0.00015. It is opened as T4.8
  in M4. The calibration report has an addendum for this baseline.
- **Next:** #138, #139, #157 (M4), then the M4 review.

## 2026-11-12: threat calibration and the M4 survival gate (#124)

**No play change; same baseline.** T4.7 measures the threat model on the
M4 baseline's games and checks the M4 exit. The report is
[docs/M4-CALIBRATION.md](M4-CALIBRATION.md).

- **The trace names its monsters** (`danger.rs:summary_detailed`):
  `threat.sources` holds one line per monster with any non-zero term
  (race, distance, speed, sleep, confusion/stun, in view, same-race group,
  rounds, expected, worst, disabling, ranged). A floating eye's gaze has
  no dice, only paralysis, so it counts. The trace is written after the
  decision: the build reproduces smoke 3/3, smoke-exact 3/3 and full 45/45
  byte for byte. The source hash changes, so the 16 deaths have new
  dossiers, D1654–D1669 (same games as D1638–D1653).
- **`tools/calibrate.py`** (with tests): windows cut on the key stream, not
  on turns (`game-world.c:run_game_loop` lets faster monsters act before
  the next key). It reports predicted against taken damage by
  expected-damage bin, range, speed, status, attack form, group and choice;
  p_death against deaths; disabling against onset messages (hunger faints
  apart); a blow decomposition from `monster.txt` and the engine's hit and
  miss messages; bootstrap intervals; named outliers.
- **Found:** expected damage is about 10× what is taken (0.087, 90% CI
  0.07–0.10). Per blow the model is right (1.70 HP predicted per blow
  against 1.77 observed, hit rate 0.92). The gap is blows that never come:
  the Borg's own blow kills or frightens the monster first in 48% of
  standing windows, and survivors land 0.58 of the predicted blows (the
  low-HP `+3` speed and erratic movers are candidates). Worst case is
  exceeded in 0.14% of windows, only by unmodelled damage. 11 of 16 deaths
  came with p_death ≥ 0.2 and the Borg fought. The other 5 were
  starvation ×2, poison, an unseen biter while resting, and a floating eye.
  For the eye, the engine deals ≥ 1 HP per paralysing blow to a character
  already paralysed (`mon-blows.c:1019-1021`), and the Borg rested beside
  it at 1/15 HP.
- **Gate:** fixtures pass (21 active; the 3 gaps belong to M5), replay is
  deterministic, G0 holds with a 10-second stall timeout, and every death
  has a dossier. Survival 30/45 (67%) beside the zoo's C Borgs at 66%
  (4.2.6) and 69% (bbc-m7) of `classes-1800`, descriptively. Our Borg gets
  there at median depth 1 and character level 1; theirs reach depth 1–4
  and level 1.5–7.
- `decision_fixtures.rs` OWNERS drops #124.
- **Refresh report:** angband +12, borg-rs +21 (2 copied docs changed),
  better-borg-c +10 (new tags `4.2.6-impact-v8`…`v11`, `v9.1`),
  **neo-angband-tracker +52** (the T4.6 entry said unmoved), borg-zoo +33
  (catalogue changed), borg-runner +26 (new records and knowledge files);
  borg-agent unmoved. The report's issue sections fail from cloud sessions
  (GitHub GraphQL is refused there; the script would need the REST API).
  Pin moves are their own PRs (RULES 21).

**For the owner:**
- **M4 is not finished.** The milestone still has the deferred M3 review
  findings **#137** (Engage cooldown after refused actions), **#138** (real
  decision candidates and goal timing) and **#139** (small cleanups). The
  last handover said #124 was M4's last task. These come next. #137 may
  change play, and then the gate's survival table needs a rerun (one
  command, in the report). The M4 review PR waits for them.
- The report's *What follows from it* lists four play changes the
  calibration supports: blow frequency, the paralysis floor, unseen
  attackers, and status damage. Each needs its own baseline. Decide
  whether they belong to the M4 review or to M5.
- This session pushed from `ccr-e0e99f46-go3y0w`, not `task/T4.7-…`: the
  cloud session can push only to its own branch.

## 2026-11-12: recovery, return home and progress under oscillation (#123)

**Play change; new baseline.** T4.6 closes M4's behaviour half: the Borg
recovers under pressure, goes home when the level cannot feed it, and digs
out of sealed rooms. Smoke, smoke-exact and full references are regenerated
in this PR; the diff against the old baseline is the declared change.
(T4.4's merge line, carried along as announced: PR #154, merge commit
f7f8a689; main is now 1915f091 after the docs commit.)

- **The `return` mode** (`strategy.rs`): starving with no known food at
  depth outranks recovery itself and bans descending (bbc D0009's full-pack
  starvation, rust D1420's descend-on-the-staircase). The goal follows the
  manual's recall-and-home routine (c15 §12.12): the up staircase underfoot
  is always taken; a known Word of Recall is read early and only in safety;
  otherwise travel to the up staircase, explore for one before any
  down-stair fallback, then dig.
- **Threatened travel** (`travel_step`): a path step that lands on a grid
  more dangerous than where the Borg stands is refused, on the tactical
  layer's composite scale (`tactics.rs:composite`, now shared) — the u32
  `danger_with` rounds a small risk to zero and has no disabling half, so
  bbc-d0006's NEVER_MOVE mold (confusion, no hit-point risk) read as a free
  walk north. The retreat step reads the same scale.
- **Heal-while-fleeing**: a live Recover goal with a monster about rests
  anyway when no next blow can kill (`worst < hp`), letting the engine's
  disturbance interrupt the rest (c15 REST-01, `src/player-util.c`; bbc
  #408/#409) — a retreat loop (rust D1330) makes no progress under
  oscillation; a rest does.
- **Stuck escalates** (`dig_step`): with no staircase and no frontier
  reachable anywhere, the stuck goal digs (`borg::diggable` — rubble,
  magma, quartz, granite carry HURT_ROCK, engine `list-terrain.h:29-34`),
  walks to a diggable wall, or reads a known recall scroll; it no longer
  waits to starve (the M3 re-review's four sealed-room runs).
- **Fixtures:** `bbc-d0006-deep-descent-readiness`,
  `bbc-d0009-full-pack-starvation`, `rust-d1420-starvation-no-food` and
  `rust-stuck-sealed-room` are **active** (the sealed-room fixture is now
  the 3x3 its construction note always described). #123 leaves the gap
  OWNERS: #124, #126, #127.
- **Measured (the declared change):** smoke — the mage lives forty times
  longer (451 → 18376 turns, reaches depth 2, dies to Fang; dossier
  D1638) and the priest now survives the cap (was a grey mushroom patch
  at 3586); smoke-exact diverges at the identical turns, so determinism
  holds across both profiles. Full — 30 of 45 runs survive the 10001
  player-turn cap (was 11 of 45); the 15 natural deaths are dossiered
  (D1639–D1653) and `reconcile-deaths.py` is clean. Two late
  **starvation** deaths (full-5-paladin-2 at 91190, full-7-ranger-0 at
  89750 game turns) are the honest residue: runs the old baseline lost
  to early monsters now live long enough to starve; the M4 survival
  gate (#124) reads this baseline.
- **Refresh report:** angband +12 (head 42cd156a), borg-rs +21 (2 copied
  docs changed), better-borg-c +10, borg-zoo +33 (catalogue changed),
  borg-runner +26 (new records and knowledge files); borg-agent and
  neo-angband-tracker unmoved. Pin moves are their own PRs (RULES 21) and
  are not in this one.
- **Next:** #124 (T4.7 threat calibration report and the M4 survival gate,
  closing M4). #126/#127 are M5 and stay untouched.

## 2026-11-12: the manuals named as analysis sources

The owner asked that the repository make its manual sources visible to every
session: the **Angband Strategy Manual** and the **official user manual** in
`reference/c-manual/` (S14). They are now named where a session starts and
where the rules live:

- `AGENTS.md` reading list, as an entry of its own, and a new *Game
  knowledge, and where analysis starts* section: engine first, manuals
  second, recorded play third;
- `RULES.md` 16: read the relevant chapter before designing a rule, then
  check every number in `engine/angband/` and cite the engine;
- `README.md`: a *Game knowledge* section, and the manuals in the
  neo-angband-tracker row;
- `docs/SOURCES.md` S14 title and `docs/research/KNOWLEDGE.md`'s reading
  state: the user manual is read with the strategy manual, per task.

No pin, copy or code changed. `cargo test --workspace` checks the documents'
links (`borg-reborn-rs/tests/repository.rs`).

## 2026-11-12: the escape ladder (#121)

**Play change; new baseline.** T4.4 replaces the staircase rule and the
Phase placeholder with the escape ladder: step, Phase Door, Teleportation,
Word of Recall and the stairs are all scored on the one hit-point scale by
cost and sufficiency, not by a fixed monster list. Smoke, smoke-exact and
full references are regenerated in this PR; the diff against the old
baseline is the declared change.

- **The ladder** (`tactics.rs`): a relocation's gain is the threat it
  removes — a step's exactly as an escape's. A blink's `risk_after` is the
  mean assessment over the remembered enterable grids of its landing band
  (engine distance 8-12 for Phase Door's 10±25%; 45-75% of the longest
  cardinal edge distance for Teleportation's 60%±25% —
  `effect-handler-general.c:effect_handler_TELEPORT`); grids a visible
  monster stands on are skipped, and an unestimated landing (an empty
  band) is not an offered escape. A stair's landing is the declared
  `arrival_risk_hp` — 0 for up-to-town — and Word of Recall's is the
  25-turn mean wait beside what stands there; an active word is not
  re-read (reading cancels it). Walking stays available after the last
  consumable.
- **Last-item protection:** a consumable escape pays `escape_item_hp` (1);
  the last known escape scroll pays `last_escape_hp` (5) more. The last
  Phase Door is kept beside a dying yeek and a never-moving eye, and still
  spent at 4/16 HP against Fang.
- **Never-movers give no parting blow** (`danger.rs:never_move`): a step
  out of `RF_NEVER_MOVE` reach ends the threat for good (bbc D0013).
- **Ties end threats:** score ties break by smaller `risk_after`, then
  larger gain, then Attack over Walk (GOALS principle 6: killing the mold
  beats stepping off it, bbc D0012).
- **New parameters** (`params.rs`): `escape_item_hp` 1, `last_escape_hp`
  5, `arrival_risk_hp` 3 — declared, to be measured (#124). Also fixed a
  latent panic: `from_profile`'s match lacked six table names
  (`eat_below_food`, `risk_aversion_pct`, `threat_horizon` among them), so
  a profile setting one hit `unreachable!()`.
- **Fixtures:** three new active ones — `bbc-d0013-eye-step-not-phase`,
  `bbc-d0017-faster-pursuer-walk`, `bbc-d0018-dying-yeek-phase`.
  `bbc-d0014` is re-pinned (Attack now; the fatal Read ended; Speed/CLW
  stay #127). `bbc-d0012`'s worm masses moved to distance 2, where its own
  construction note always said they were; `fang-ac36` and
  `fear-blocks-the-attack` were widened to 15 columns so the blink band
  has remembered floor (both construction notes amended). #122 is closed,
  so the gap OWNERS are the open tasks only: #123, #124, #126, #127.
- **Refresh report:** angband moved +12 commits past the pin, borg-rs +18
  (one copied doc changed), better-borg-c +10 with new milestone tags
  `4.2.6-impact-v8`…`v11` (v9.1 among them — new zoo specimens to
  consider). Pin moves are their own PRs (RULES 21) and are not in this
  one.
- **Next:** #123 (T4.6 recovery/oscillation, owns four gap fixtures), #124
  (T4.7 calibration and the survival gate — the smoke mage's death to a
  grey mold's spores at 9/9 HP is calibration's first exhibit), #127
  (T5.3, owns d0010/d0014's class answers).

## 2026-11-11: play mode for models; borg-agent retired (owner, D11/D12 amended)

The owner wants models to be able to **play** through `borg-mcp`, not only
query it, and says borg-agent was a failed experiment that would need a
rebuild. Documents only:

- **GOALS:** D12 now covers a query *and play* surface; constraint 2 is
  "the measured Borg plays without a model" (play mode is a separate arm,
  logged and replayable, bound by D3 and D5); new roadmap stage **Q5,
  play mode**, and Q4 (live) is no longer optional, since Q5 needs it.
  *(Superseded the same day by the reorder below: Q4 became the diagnosis
  workflow, live hosting moved into Q5, and play mode is optional.)*
  D11: borg-agent retired as an input, its roles move to Q; borg-runner
  stays. Option E and the non-goal reworded to match.
- **RULES 24–25:** the zoo specimen is always the model-free Borg and the
  Borg's code never calls a model; a model game is a separate arm with its
  model, effort and tool calls recorded; in play mode the model chooses
  only executor actions, and rules 11, 12 and 15 bind it.
- **PLAN:** Q5's row and exit; Q4 no longer optional; the Neo Angband
  line no longer mentions borg-agent.
- **docs/QUERY-SURFACE.md:** *Play (Q5)*: tools `observe`, `options`,
  `act`, `autopilot` (hand-off conditions), three arms (model, hybrid,
  advised), the run record (`model.json`, `calls.jsonl`); *Workflows*:
  explaining a death, triage, reviewing a change, hybrid play, and which
  model for each; two new checks (play replay, arm separation).
- **SOURCES S7, ARCHITECTURE L3, AGENTS:** borg-agent retired; its pin is
  dropped in the next `--update` PR (RULES 21: pin moves are their own PRs).

**For the owner:** nothing new to decide before Q starts. The hand-off
conditions' defaults are an open question for Q5.

**Same day, Q reordered around diagnosis (owner).** The MCP's purpose is
diagnosing the model-free Borg's deaths after the fact; play mode is
optional and last. New roadmap: **Q0** pilot (no code, #153: diagnose
deaths from the dossiers as they are, measure a hit rate on known deaths,
list the missing tools), **Q1** facade (#149–#151, unchanged), **Q2**
diagnosis tools, **Q3** counterfactuals with **`branch`** (rerun the
seed, force an alternative action at a think through a `branch` cargo
feature the zoo never gets, let the Borg continue, report what followed;
one sample, not a probability), **Q4** the diagnosis workflow and its
evaluation, **Q5** play mode, optional, decided after Q4. The primary plan
(M4–M7) is unchanged.

**For the owner:** create the GitHub milestones Q0–Q5 (this replaces the
earlier ask for Q1–Q4): #153 goes into Q0, #149–#151 into Q1; open Q0 when
you want the pilot to run (it can run beside M4: it changes no code); fix
the hit-rate threshold for Q4's evaluation, and which cases it counts,
ideally after seeing Q0's numbers.

**#152's review (owner and a second reviewer), fixed in the same PR:**
- Q0's ground truth: the reference dossiers hold no trace or parameter
  table, so calibration is by deciding *turn*, cause and alternative, on
  the set #153 names;
- our dossiers keep the last 200 thinks (`tools/export-death.py`), so the
  triage gives the deciding think where the trace reaches it and the turn
  otherwise, and a longer trace is a named Q0 output;
- `branch` builds are marked `features=branch` in `BUILD_ID`, since the
  source hash cannot tell feature builds apart; the zoo-contract refusal
  is a new assertion to add in Q3, not an existing check;
- RULES 24's scope restored to every measured game build (ours or the
  zoo's), matching GOALS constraint 3;
- "the server does not link the engine" replaces "engine-free";
- the Q4 threshold is marked open in GOALS and PLAN;
- QUERY-SURFACE's mode headings say *query mode* and *play mode*, as the
  rules do;
- draft fixtures are not committed as tests;
- README no longer lists borg-agent as an input.

Left for the pin PR (RULES 21): `pins.json`, RULES 7 and 21, and
`scripts/refresh-sources.py`'s docstring, which still list borg-agent.

## 2026-11-11: no control runs requested from the zoo (owner)

The owner: borg-zoo already runs the original C Borg and better-borg-c in
its other experiments, so this repository does not request runs of them.
GOALS (observation rule, G1 prerequisites, G1/G2, D8), SPEC §6 and the M4
and M5 exits now say that a zoo run we ask for is of our Borg only, and
comparisons use the zoo's existing control results. Issues #124 (T4.7),
#128 (T5.4) and #131 (T6.3) are updated to match.

## 2026-11-11: secondary goal Q, the query surface (owner decision D12)

The owner asked for the Borg's judgement to be exposed to language models
over MCP, as a secondary goal with a roadmap, and for the rules to change
to allow it. Documents only, no code:

- **GOALS.md:** new section *Secondary goal Q* with its five constraints
  and the Q1–Q4 roadmap; the non-goal "a model in the play loop" stays and
  is clarified; D11 amended; **D12** added.
- **RULES.md:** RULES 2 lets a session work on a Q milestone once the owner
  opens it, never mixed with primary work; new RULES 24 (the Borg plays
  without a model; the core never depends on `borg-mcp`) and 25 (no
  judgement in the server; the versioned `query` facade).
- **PLAN.md:** *Secondary track Q* with milestones Q1–Q4 and their exits.
- **docs/QUERY-SURFACE.md:** the architecture draft. Decision: same
  repository, separate crate (`borg-mcp` depends on the core's new
  `query` facade, never the reverse); replay mode first (the
  `BORG_OBSERVATION_LOG`, `BORG_WORLD_LOG`, `BORG_KNOWLEDGE_LOG` logs and
  `trace.jsonl` already hold what is needed); live mode optional, one game
  per process.
- **ARCHITECTURE.md:** a pointer to the draft.

**Review (#148, owner's comment):** three findings, all fixed. (1) Q1's
exit said "the harness build's identity is unchanged", but
`harness/identity.sh` hashes all of `borg-reborn-rs/` and `Cargo.lock`, so
adding `query` changes it; the exit and the draft's checks now ask for
the references to reproduce play, and say a changed source hash with
unchanged play is not a new baseline (RULES 22). (2) RULES 2's wrap. (3)
The replay-determinism check was in both TQ1.3 and Q3's exit, and the
draft said CI from Q2; it is now Q1's alone.

**Found:** a replay rebuilds `World` from observations, but not the
strategy's own state (mode, goal, `Memory`, RNG). The draft replays the
strategy too, with the run's seed and profile, and checks each choice
against the trace, which makes every replay a determinism check.

**Q1's tasks are filed** (#149 `run.sh --record-state`, #150 `query::State`,
#151 strategy replay), labelled `task` but in no GitHub milestone: this
session cannot create one.

**For the owner:** create the GitHub milestones Q1–Q4 (move #149–#151 into
Q1) when Q is to start (the plan says after M4 is merged); decide after Q3 whether
Q4 (live) is wanted. The draft's open questions (transport library,
snapshot interval, `what_if` look-ahead, live pause point) are settled in
the stage named there. M4 continues unchanged.

## 2026-11-10: item actions and the escape items (#122)

The executor now has **Quaff**, **Read**, **Eat** and **Wield**: the
pack letter, the command, and an effect checked next think (the kind's
count fell, or the kind is in an equipment slot). The tactical layer uses
them:

- a known Cure Light Wounds is scored by hit points restored (2d8 mean,
  declared) and chosen when it beats the alternatives; bbc D0002 and
  rs D0005 now quaff it;
- a known Phase Door is scored by the risk it removes, with landing risk
  half for now; Fang AC36 reads it, and fear accepts it;
- a better pack weapon is scored by the rounds it saves on the most
  threatening adjacent monster;
- a hungry character with a known ration enters `Mode::Eat` before the
  timer reaches starvation (`eat_below_food`, new parameter, declared).

**Fixtures:** 14 active, 7 gaps. New active: D0002/rs D0005 (known cure),
Fang AC36 (known Phase), `rust-hungry-eats` (food). Fear accepts the
Phase escape. D0014 now pins the **last Phase at full HP** for #121
(landing risk and last-item protection); D0011's Wield is still not
chosen over the attack; D0010/shoot and casting stay #127; the
unknown-food/starvation and return-home gaps stay #122/#123.

**New baseline (play change).** References: smoke 3/3, smoke-exact 3/3,
full 45/45; boundary 3/3. 38 natural deaths, 0 unexported; 36 new
dossiers D1566-D1601. Smoke has one 20,000-turn survivor and two deaths;
**the full suite is 34/45 deaths and 11 survivors at the 100,000-turn cap**,
deepest depth 3, with 3 starvation and 10 floating-eye deaths. That is a
real behavioural change, not a survival claim: #124's calibration and the
M4 gate judge it, and the zoo has not measured it. The floating-eye
deaths are the escape-ladder case (#121), not a cure failure.

**Sequencing.** #121 was started first and parked: every escape-ladder
fixture but walking needs Phase/teleport actions, so #122 went first by
owner decision. #121 resumes with real actions next.

**Next:** #121 (T4.4), escape ladder and landing risk: last-item
protection (D0014), the floating-eye case, and a real landing-risk
estimate for Phase/teleport and level changes. Then #123.

## 2026-11-10: tactical alternatives and the re-review (#120)

`tactics.rs` replaces the first-applicable fight-or-retreat rule. Every
think in contact generates and scores, on one hit-point scale:

- an attack on each adjacent monster (only when `can_attack`; fear and
  paralysis are refused by the engine), gain = the share of its threat the
  attack removes;
- a step to each of the eight neighbours, with the free hit every adjacent
  monster gets when the step leaves its reach;
- the up staircase underfoot when a threat is present (landing risk is
  #121; a down staircase under fire is still not taken);
- holding.

The score is `gain - risk_after`, using T4.2's assessment, the one
`risk_aversion_pct` and `threat_horizon`. A disabling or summoning risk
costs expected hit points in the score (a declared upper bound), so a
paralysis gaze is not free. Every response, including the losing ones,
goes to the trace; #138 narrows the remaining contract.

**Fixtures.** Ten active, ten gaps. Promoted: rs D0006 (kills the 2-HP
mould), rust D1358 (attacks the 2-HP mouse instead of taking its free
hit), rust D1413 (ascends with Fang adjacent and the stair underfoot).
D0013/breeder accept the scored fight/step. The re-review's four
sealed-room starvation runs are attached to #123; `Mode::Stuck` now names
the state (`stuck` / "no mode can act") and `rust-stuck-sealed-room` plus
`a_sealed_room_names_the_stuck_state` pin it. Search/dig/recall is #123.
D0010 and Fang AC36 are gaps for #127 (shoot/cast); the cures and the
return home are #122/#123.

**New baseline (play change).** References regenerated: smoke 3/3,
smoke-exact 3/3, full 45/45; boundary 3/3. 49 natural deaths, 0
unexported; 47 new dossiers D1519-D1565 from the clean checkout. Smoke has
one 20,000-turn survivor and two deaths; full is 45/45 deaths, deepest
depth 3, longest 89,380 game turns, **20 starvation and 8 floating-eye
deaths**. No survival claim: food is #122, the floating-eye/paralysis
escape ladder is #121/#122, and the stuck exits are #123. The re-review's
finding 2, G0's missing smoke-warrior death and the four sealed-room runs
are recorded; the owner's #135 scope choice remains the owner's.

**Next:** #121 (T4.4), the escape ladder with landing risk; it should take
the stair-under-contact response beyond "upward only" and give the
floating-eye deaths an escape. #122 then adds the actions; #123 the stuck
exits.

## 2026-11-10: one threat assessment (#119)

`danger.rs` is now L4's assessment rather than the C Borg's full-damage sum.
For each grid and horizon it returns:

- `expected` damage: the engine's hit chance (`chance_of_monster_hit_base`,
  `player-attack.c:hit_chance`) and armour (`adjust_dam_armor`) over the
  attack rounds from the energy model;
- `worst`: every blow hits at its largest roll;
- `p_death`: exact convolution for small fights, a declared normal
  approximation for large ones;
- `disabling`: fear/blind/confuse/paralyse from blows and spells;
- `summoning`: shrieks and `S_*` spells;
- `economic`: theft, destruction and drains, never added to lethal risk.

Ranged attacks use the C Borg's breath/shot estimates as a floor
(`borg-danger.c:619-760`); uncommon effects use a conservative generic and
are marked as estimates, because `Knowledge` does not carry the spell
effect dice or breath divisors. LOS is a conservative Bresenham over the
remembered level. Summoners/breeders use the C Borg's 1.5/1.8 growth
factors. Remembered-unseen monsters scale by `Track::confidence`.

One risk number is `expected + risk_aversion_pct * (worst - expected)`.
Paths, rest, retreat, contact and combat pass the same tolerance;
`threat_horizon` sets the window. Both are new parameter-table entries
(50%, 1 turn) with evidence, not tuned values. Every trace line now has the
six numbers (`threat`), which #124 reads. Fear and paralysis set
`can_attack`/`can_act`; the engine refuses the attack, so the strategy
cannot choose it.

**Fixtures.** Seven active now: bbc D0008, D0012, D0013, D0019 (one
apprentice does not cause whole-level flight), the breeder pack, fear
(refused attack), and rust D1330 (the cat is seen and the stair is taken).
Twelve gaps remain, all #120/#122: stairs under contact, retreat at 1 HP,
the 2-HP mould, quaff/eat/read/wield. `bbc-d0006` pins Walk(7). The
fixture catalogue records the promotions.

**New baseline (play change).** Smoke/smoke-exact/full references are
regenerated and match 3/3, 3/3 and 45/45; the exact-boundary check is 3/3.
The smoke suite has one 20,000-turn survivor and two deaths; the full suite
is 45/45 deaths, deepest depth 3, longest 89,440 game turns, with nine
starvation deaths. 49 natural deaths in the runs, 0 unexported; 47 new
dossiers D1472-D1518. **No survival claim**: starvation is #122, and the
tactical layer that could use the new numbers is #120. The calibration
report (#124) is where the estimates are checked against damage taken.

**Next:** #120 (T4.3), tactical alternatives on one scale. It must promote
the twelve gaps: cure/attack/stairs under contact, retreat versus the free
hit. #122 then supplies the actions. The trace has the numbers; no
threshold was tuned silently.

## 2026-11-10: danger race flags, the #142 fix

The T4.1 fixtures' first finding is fixed. `danger.rs` named `MULTIPLY`,
`NEVER_BLOW` and `NEVER_MOVE` one higher than the engine enum (`RF_NONE` is 0
and the first valid flag is 1), so a grey mould's `NEVER_MOVE` was read as
`NEVER_BLOW`: it scored zero danger and the Borg rested beside it. Breeders
lost the multiplier and `RAND_25` was misread as `NEVER_MOVE`.

- Constants are now 17, 20 and 21, with the counting rule in the comment.
- `race_flag_numbers_match_the_engine_enum` checks the real bytes: a grey
  mould's byte 2 bit 4 is `NEVER_MOVE` and not `NEVER_BLOW`; a silver mouse
  is `MULTIPLY | RAND_50`; an adjacent mould scores above zero.
- Fixture promotion: `rs-d0006-mold-at-two-hp` no longer rests (Walk is the
  recorded retreat) and moved from the #142 gap to #120; `rust-d1330-rest-
  with-pursuer` no longer rests either, and its remaining gap is the
  staircase under a seen threat (#120). `bbc-d0006`'s exploration path
  changed from Walk(7) to Walk(9); the fixture pins the new direction.
- This is a play change: smoke, smoke-exact and full references are
  regenerated from build `3a74b98`, and the deaths at the new build
  identity are re-recorded. The runs produced 51 natural deaths, 0
  unexported, and 48 new dossiers D1424-D1471 (three repeats).
- **The full suite is 45/45 deaths, deepest depth 2, longest 89,240 game
  turns** (the longest run is 8,925 player turns). The previous baseline
  reached depth 3. This is not evidence that the fix is worse: the old
  model was resting beside moulds it could not see, and the strategy still
  has no cure/escape action. It is a play change to be measured, not a
  survival claim, and T4.2/T4.3 must judge it.

**Checks.** `cargo test -p borg-reborn-rs` (including the fixtures),
smoke and smoke-exact runs, `harness/diff` 3/3 and 3/3, the exact-boundary
check 3/3, the full suite, and 45/45 reference diffs. The references are a
new baseline, not an oracle; no survival or population claim.

**Next:** #119 (T4.2), one threat assessment. The fixtures it must satisfy
are `fear-blocks-the-attack` (#119), `bbc-d0006-deep-descent-readiness`
(#119), and the #120 group (stairs under contact, retreat at 1 HP, the
2-HP mould). #142 is closed by this task.

## 2026-11-10: T4.1 death-derived decision fixtures (#118)

Eighteen constructed `Observation` fixtures now replay deterministically in
`borg-reborn-rs/tests/decision_fixtures.rs`; the catalogue and the M3
triage are in [docs/FIXTURES.md](FIXTURES.md), the schema in
`tests/fixtures/README.md`, and the engine data they read is the trimmed
`tests/fixtures/knowledge.json` (a `BORG_KNOWLEDGE_LOG` dump at
`e473720`).

- **Four are active**: bbc D0008 (attack the adjacent 2-HP mould), bbc
  D0012 (the owner branch's one-blow kill), bbc D0013 (no level flight
  over one kobold), and `mmx-war-1` (one action per think in a breeder
  pack). They must avoid the forbidden action and meet the alternative.
- **Fourteen are gaps**, not skips: each pins the action the Borg takes
  today and names the open M4 owner. When the action or decision changes,
  the test fails and the fixture must be promoted. Owners: #119 (threat
  horizon, fear), #120 (scored alternatives, stairs under contact), #122
  (quaff/eat/read/wield), #142 (flag numbering).
- **New finding, filed as #142**: `danger.rs` numbers `MULTIPLY`,
  `NEVER_BLOW` and `NEVER_MOVE` one higher than the engine enum. A grey
  mould's `NEVER_MOVE` is read as `NEVER_BLOW`, so it scores zero danger
  and the Borg rests beside it; breeders lose the multiplier. The evidence
  is in the issue; the fix needs a new reference baseline and belongs with
  #119, not this PR.
- **M3 triage** (D1326-D1420, 95 dossiers): 59 last actions were retreat
  walks, 21 rests, 12 attacks, 2 explore walks, 1 recover walk; 4
  starvation, 4 poison, 5 grey mould, 2 Fang. Four representative fixtures
  cover the families; the rest repeat them (28 causes, one independent
  death per encounter family).
- The fixtures are constructed, not recorded: the committed dossiers lack
  the original snapshots. Each file's `construction` says what was
  invented and `limits` says what it cannot claim. The owner-manual
  branches are diagnostics, not lives; their `verified` text quotes the
  turn range and end state.
- Doc state from the M3 review is corrected: PLAN says M4 is in progress
  (T4.1), and #136 no longer carries `in progress`.

**Checks.** `cargo fmt --all --check`, `cargo clippy --workspace
--all-targets -- -D warnings`, `cargo test --workspace` and the script
tests. No play code changed, so no engine build, smoke run or reference
regeneration was needed; the fixtures are pure crate tests.

**Re-recording.** `harness/identity.sh` hashes the whole `borg-reborn-rs/`
tree, tests included, so adding the fixture test changed the build identity
even though play is byte-identical. CI's smoke deaths therefore arrived as
a new encounter family; they are re-recorded as D1421-D1423 from the CI
registry (seed 102/103/101, the same deaths as D1373-D1375). This is the
existing process cost of a test-only change. Whether test files belong in
the build-input hash is a repo-level question for the owner; it does not
block M4.

**Next:** #119 (T4.2), one threat assessment. It must account for #142
first (the fixture `rs-d0006-mold-at-two-hp` pins the wrong decision), then
add status (fear), the group/breeder multiplier and the multi-step horizon
that `rust-d1330-rest-with-pursuer` pins. #118's fixtures are the check.

## 2026-11-10: M4 source refresh

The owner directed M4 to start while #135 remains open for re-review.
Pins moved for all five active sources; copied references and the death
index are refreshed, with the new reading in
[docs/research/KNOWLEDGE.md](research/KNOWLEDGE.md).

- better-borg-c `a4cc2d4`: no new death dossiers. M11 #471 has 18 carry
  units but no cohort/per-mode evidence and no v11 tag yet. Read the five
  `borg-danger.c` corrections (partial-hit armour/chance, CON drain,
  Crush's ranged value, rubble) and Shoot'N'Scoot's index fix.
- borg-runner `1440f70`: edition 4, including the fear/shrieker death,
  Fang AC36 versus Magic Missile, sparse sleeping monster placement,
  the whole-level M map and `mmx-war-1`'s batched-key breeder death.
  These become M4 fixture inputs.
- borg-zoo `415183c`: use explicit `normal` for controls; `default`
  changed meaning at the mode cutoff. reserve410's small pilot is mixed,
  not support for a buying claim.
- borg-rs `399c907`: the family now has an explicit buying-strategy seam;
  buy-to-identify remains open. Angband's 12 new commits are noted for
  M4's rest/executor work; our engine remains at `e473720` as specified.

**Next:** #118 (T4.1), death-derived decision fixtures. Build the
observation/scenario fixtures before changing the threat model, including
the PLAN seed cases, the new fear/shrieker and Fang cases, and reproducible
M3 deaths. No survival or tuning claim comes from the M11 package.

## 2026-11-10: M3 review blocker, bounded rerolls (#136)

PR #135's review rejected the first G0 baseline: T3.7's up/down reroll
could generate levels forever at one game turn, reset both no-time and
goal budgets, and select fresh levels (stair-scumming in substance).
The dedicated fix replaces those references and rewrites
[docs/G0.md](G0.md).

- Level generations are unique; travel goals die on generation change,
  so fresh levels Explore. Only an exhausted frontier or cooling Explore
  may reroll, once per generation and at most two level changes at one
  game turn. Stairs no longer reset the ordinary no-time budget.
- Unsafe staircases retreat instead of holding; remembered danger uses
  `Track::confidence` and decays out of view.
- `tools/check-no-time.py` checks same-turn actions and level caps in CI;
  final G0 runs use a real 10-second supervisor stall timeout.
- Review findings deferred to M4: #137 Engage cooldown, #138 real
  decision candidates, #139 small strategy/executor cleanups. Closed
  M3 task labels are cleaned; PLAN points at the review blocker.

**Corrected evidence:** 54/54 processes succeed with the 10-second
supervisor; smoke is deterministic and exact passes; 54 natural deaths,
0 unexported, with 48 valid new dossiers D1373–D1420; 51 traces have no
same-turn budget violation; perception is 0 differences over 50,047
thinks; tests are 81 crate + 5 repository + 56 tools. Full-suite stair
actions fall from the rejected baseline's 2,040 to **83**, with no turn
above two.

**Not hidden:** the new full suite has 546 Holds and four starvation
deaths, all 45 runs die, deepest is 2. One unreachable-exit case owns
400 Holds and belongs to M4/#123. The old “6 Holds / 0 starvation”
claim is withdrawn. G0 passes on smoke only; there is no full-suite
no-stall or survival claim.

**Next:** return PR #135 to the owner/reviewer for re-review. Do not
start #118 until the owner merges the milestone review.

## 2026-11-10: T3.8, M4–M7 are task issues (#24)

Created the dependency-ordered future work, with PLAN.md's death fixtures
and owner-manual alternatives attached to the tasks that must satisfy
them. No play code or reference change.

- **M4, danger and survival:** #118 death-derived fixtures; #119 one
  threat assessment; #120 tactical alternatives on one scale; #121 escape
  ladder and landing risk; #122 consumables/status/reserves; #123
  recovery and oscillation; #124 calibration report and M4 gate.
- **M5, readiness and progression:** #125 depth readiness; #126 equipment
  and pack valuation; #127 class item use/spells/ranged; #128 G1 on
  `classes-1800` (owner-run zoo, D and margin fixed in advance).
- **M6, the family:** #129 evidence-led class profiles; #130 screening
  and racing; #131 G2 against better-borg-c on fresh seeds.
- **M7+:** #132 deep suites and G3; #133 long games and G4.

Every issue has the `task` label, its milestone and a **Needs** chain;
T4.7 opens the M4 review rather than allowing an automatic M5 start.

**M3 is complete.** Next and last in this session: open the M3 review PR
(`reviewed` ← `main`) with the G0 evidence and review prompt, then stop.
The reviewer is the owner's choice; do not start #118.

## 2026-11-10: T3.7, G0 (#23)

**G0 passes; the report is [docs/G0.md](G0.md).** The first attempt did
not pass honestly: seven long runs held in Explore/cooldown loops until
starvation, 53,497 Holds in the full suite. T3.7 fixes the causes:
frontier-underfoot is finished, empty Explore falls to Descend, the
up/down reroll actually Ascends on reaching the up staircase, and checked
level changes do not spend the refused-action budget. Regression tests
cover the exhausted level and its only exit.

Final committed-tree evidence (`6318cd2`, references in `84a30fd`):

- 54/54 smoke, smoke-exact and full processes succeed with no stall,
  timeout, signal or harness failure; two smoke suites are byte-identical
  and exact boundaries pass. Extra-load passes at turn 1408.
- 51 natural deaths in run outputs, 0 unexported; the 47 new dossiers
  D1326–D1372 all have valid trace tails and hashes.
- Full suite: **6 Holds and 0 violations** (was 53,497 and 2), deepest
  **3** (was 2), **0 starvation deaths** (was 7). All 45 still die, so
  this is a machinery pass, not a survival claim.
- Perception: 0 differences over 24,079 thinks. Tests: 76 crate + 5
  repository + 54 tools; fmt and clippy clean. Smoke's warrior survives
  to CL7 at the turn cap.

**Next:** T3.8 (#24) — break M4–M7 into task issues, then stop for the
M3 review PR. Do not start M4.

## 2026-11-10: T3.6, traces in death dossiers (#22)

**Already implemented by T2.4 (`651c822`), now verified as a task; no
play-code or reference change.** Every Rust death export keeps
`trace-tail.jsonl` (the last 200 thinks), a `decision_trace` record with
`lines` and `of`, its SHA-256 in `files_sha256`, and a report link.
Untraced C runs record `decision_trace: null`. The exporter's module
documentation now states this contract in the present tense.

Validation: all **54 tool tests** pass, including the 250-line synthetic
trace that retains exactly thinks 51–250. An audit of the **48 T3.5
dossiers D1278–D1325** parsed every retained think and checked every
line count and trace SHA-256; examples range from all 51 thinks of
D1278 to the last 200 of D1325's 8,876.

**Next:** T3.7 (#23) — the G0 smoke/stall/determinism/death-trace report
for the M3 review.

## 2026-11-10: T3.5, town reserves (#21)

The first strategy now has a **Town** mode, checked after recovery and
contact but before any live travel goal:

- **Reserves are explicit parameters**: 3 rations, 2 spare torches, 3
  Cure Light Wounds, 4 Phase Door and 1 Word of Recall (c15 §12.4 and
  the owner's verified D0015 loadout). Only identified consumables
  count. Food/light, cures, escapes and the way home are bought in that
  order, one ware per observation so sold-out rows cannot leave stale
  letters. A full pack still permits a purchase into an existing stack;
  unavailable stock, unaffordable wares or no room for a new stack let
  the Borg leave town instead of looping.
- **Identification sales** follow the reserves: one unknown scroll or
  potion at a time goes to the Alchemist, which learns its flavour and
  runes before the hand-over (`store.c:1947-1956`). Selling earns 0 with
  `birth_no_selling`; this is identification, not income. There are no
  discretionary purchases yet.
- **Store mechanics fixed along the way:** stacked buys/sales check
  carried item counts, not only pack slots; free store actions no longer
  consume the refused-action budget; and paths can leave a shop entrance
  after shopping as well as arrive at it.

**Validation:** 74 crate tests and 5 repository tests, clippy and fmt;
smoke 3/3, smoke-exact 3/3, full 45/45 and extra-load from committed
`155c25a`; **0 perception differences over 72,623 thinks**. Seed 101
buys the intended kit, reaches **depth 2 / CL3** and dies to a soldier at
3696 with zero violations; its keys are byte-identical on rerun. All 45
full-suite runs shop. References regenerated; deaths D1278 onward.

**Honest state:** every full-suite run still dies and deepest remains 2;
seven starve (buying food is not eating it), and two unrelated refused
attacks are left for T3.7's G0 report. No final-suite run returned to
town carrying an unknown consumable, so the identification sale's live
key sequence was not exercised; its strategy, keys and stacked effect
are unit-tested.

**Next:** T3.6 (#22) — put the decision trace in every death dossier.

## 2026-11-10: T3.4, the playing behaviours (#20)

On top of T3.3's machinery, the first behaviours (references
regenerated; every game differs from T3.3's):

- **Engage answers contact**: a visible monster in reach makes the mode
  Engage. The exchange test (c15 FIGHT-01 through the danger sum): the
  monster's danger at our grid × the turns our melee per turn
  (`Character::melee_per_turn`) needs to kill it — fought when that
  costs less than every hit point (no margin constant; M4's tactical
  layer replaces the whole comparison), the *cheapest* adjacent fight
  chosen (bbc D0012's 2-HP mold). A losing exchange steps away
  (never onto a monster); a corner fights the least dangerous one
  anyway (bbc D0018: walking from a faster pursuer is death).
- **Pickup**: the object underfoot is taken (`Get`) whenever the pack
  has room and nothing is in contact — never under fire, and never
  before the threat is cleared (bbc D0019's dropped Phase Door, both
  orders of the same mistake).
- **Stairs under fire are not taken**: `Descend` requires zero danger at
  the grid; what presses is fought or escaped first.
- Rest unchanged (the strict zero-danger gate); the recovery invariant
  now also requires safety, so contact outranks resting.
- **The world follows the player's mutable memory.** `map_info` reports
  `player->cave`, and feature 0 means the player has forgotten the grid:
  nightfall in town wipes boring floors (`cave_illuminate`,
  cave-map.c:581-585; day length 10000, hence turn 50000). `World::see`
  now erases those grids instead of keeping a private map. CI caught the
  old behaviour diverging by 2.5 million grids in one town night; the
  full test suite now has **0 perception differences over 39,002
  thinks**.
- **Town is left, not mapped.** At depth 0 Descend outranks Explore
  (c15 TOWN-01): one earlier run starved after exploring town for
  50,000 turns. The final full suite's deepest run reaches depth 2.

**The honest M3 state:** the Borg fights, but remains fragile. All 45
full-suite runs still die at CL1–2; the deepest reaches depth 2, four
starve after long dungeon stays, and fruit bats, poison and ordinary
early monsters remain deadly. Final seed 101 dies of poison at turn
2490; its 252 thinks split explore 134 / engage 116 / descend 2, with
zero effect violations and byte-identical keys on rerun. The smoke
priest survives to the 20,000-turn cap at depth 1. T3.7's G0 report
should examine both the long starvation runs and any remaining Holds.

**Next:** T3.5 (#21) — town: sell, buy reserves (the owner's verified
D0015 loadout in the ledger is the target shape; the store actions and
the free identification-by-giveaway are already in the executor).

## 2026-11-10: T3.3, the first strategy (#19)

`strategy.rs` (L5) replaces the minimal Borg's placeholder policy — the
**first play change since M1**; the references are regenerated in this
PR and every difference is the strategy's.

- **Modes, goals, budgets, cooldowns** (GOALS principle 6):
  `Recover`/`Descend`/`Explore`, each goal with a budget in game turns
  and a progress measure (hit points for recover, depth for descend, the
  level's remembered grids for explore); a goal that stalls past its
  budget, or whose action the executor reports refused, fails and its
  mode cools down (`goal_cooldown_turns`, 100). Hysteresis is by rule,
  not by number: a live goal keeps its mode until it completes, becomes
  unsafe or exhausts its budget; the recovery invariant outranks all.
- **The M3 danger placeholder** (`danger.rs`): the C Borg's danger sum
  lifted with citations — `borg_danger`'s capped sum
  (`borg-danger.c:2824-2865`), `borg_danger_one_kill`'s energy model
  (`:2361-2395`), proximity model (`:2516-2593`) and wakefulness and
  condition scaling, and `borg_danger_physical`'s per-blow full-damage
  terms for the common early blows. Declared out, for M4: the spell
  term (a caster at range scores melee only), the friends multiplier,
  the fear maps, PFE and buffs. It now feeds both the flow's threat
  costs (replacing T3.2's hand-rolled shape) and the strategy's rest
  gate.
- **Rest safety** is the strict reading of c15 REST-01: rest only at
  zero danger — the C Borg's `avoidance` form rests beside a small
  monster and dies on the rest's own turns (bbc D0017), and ours did
  too in the first try (a rest-loop against a fruit bat).
- **Recovery under threat** steps to the least dangerous neighbour
  (never onto a monster's grid) — one step of L4 until M4's tactical
  layer.
- Two new parameters with declared-placeholder defaults:
  `explore_budget_turns` (2000) and `goal_cooldown_turns` (100).
- The trace's mode is the strategy's own ("explore", "descend",
  "recover"); the executor's effect failures reach the strategy as goal
  failures.

**The honest M3 state:** the Borg cannot fight (melee is T3.4), so it
retreats from everything — and a faster pursuer kills a retreating
character (bbc D0018's shape). The smoke runs die early: seed 101 to a
fruit bat at turn 2815, the mage to a grey mold at 2221, and the town
runs to dogs and cats (D1081–D1086 and the full suite's). No stall, no
crash, no harness failure, no effect violation; modes switch correctly
(explore 246 / descend 16 / recover 21 in the seed-101 trace); the same
seed twice gives byte-identical keys.

**Next:** T3.4 (#20) — melee (the exchange judged with the danger sum),
pickup, stairs and rest as real behaviours on top of the machinery.
Then T3.5 (town), T3.6 (traces in dossiers), T3.7 (G0).

## 2026-11-10: borg-runner's ACCUMULATED_KNOWLEDGE, copied (S15)

A pass over borg-runner and better-borg-c, asked for by the owner, found
one document we did not have: **`ACCUMULATED_KNOWLEDGE.md`** at
borg-runner's root — the export *written for this repository* (its header
names borg-reborn-rs as the consumer), only what no manual or data file
says: measurements, corrections of the manuals and of itself, numbers,
and the open questions. It is rewritten from time to time (edition 3
landed during this pass and **corrected edition 2's most load-bearing
claim: a level has three or four down staircases, not one** — the "one"
was the labyrinth generator; the classic/modified generators ask for
`rand_range(3, 4)`, `gen-cave.c:1291`).

This PR adds the copy rule (and a two-line amendment to
`refresh-sources.py` so a root-file rule works, with a test), moves the
borg-runner pin to `883c474` (four new commits today: a twelfth run
`minimaxflash-3`, a Warrior that died standing still for a Word of
Recall; a new INSTRUCTIONS section on constitution as the pace of
descent; edition 3; playbook folds), and records S15's new material in
SOURCES.md. The reading itself is consolidated in
`docs/research/KNOWLEDGE.md` (next PR), which also sets up the
read-marker process the owner asked for.

## 2026-11-10: the knowledge ledger (read markers for the moving sources)

`docs/research/KNOWLEDGE.md` (this PR) consolidates the pass:

- **The loop**: the refresh report's lines (new deaths, changed copies,
  issues updated since `read_on`, new tags, changed cited functions) are
  mapped to what to read; the ledger records the state. FACT lines are
  checked and cited before use (RULES 16); nothing is a claim until the
  zoo measures it (RULES 14).
- **Reading state**: borg-runner read whole at pin `883c474` (all
  manuals, every notebook and report, ACCUMULATED_KNOWLEDGE edition 3 and
  its diff); better-borg-c dossiers D0001–D0019 and the analysis issues
  (#400–#412, #420 on this pass); the zoo's catalogue.
- **Distillations by task**: T3.3 (depth is not the objective, 3–4 down
  staircases, explore-or-reroll, constitution as pace), T3.4 (the
  hit-chance formula, thrown oil, rest policy), T3.5 (the owner's
  reference loadout from D0015, the store identifying for free,
  restocking), M4 (the five analysis issues as the failure catalogue,
  breath/summon/speed facts, unknown pools by depth, escapes-bought-
  never-used), M5 (kit gates, power pricing, the mana penalty, class
  findings).
- **What borg-runner can additionally provide**: `decisions.jsonl` at the
  pin (scenario material), the replay site, and what *not* to take
  (`agents/` is model self-notes, not game knowledge).

AGENTS.md's entry reading order gains the ledger as step 7, so every
session knows the reading state before choosing work.

## 2026-11-10: T3.2, paths and exploration (#18)

`borg-reborn-rs/src/path.rs` is L4's paths, substrate for T3.3's strategy
(nothing calls it in play yet; T3.3 wires it into decisions):

- **Distance maps, not gate-kept floods.** One Dijkstra flood per goal
  over the remembered level, from the goal grids outward (the C Borg's
  shape, `borg-flow.c:borg_flow_spread`), but threat is a **cost** rather
  than the C Borg's fear refusal (borg-flow.c:442-474): a dangerous grid
  is paid for, and a route through it stays available when it is the only
  one. Goals: `Stairs{up}` (the known `<`/`>` grids), `Frontier` (known,
  enterable grids with an unknown neighbour — the C Borg enqueues the
  unknown itself, borg-flow-dark.c:46-58; a closed door by the unknown is
  frontier too, borg-flow-dark.c:61), and `Grids` (explicit targets).
- **Costs.** Floor 1; a closed door 2 (walked open, then entered);
  a web or a known trap +2; unknown grids are never transit (the C Borg's
  `avoid`); shops are goals but not roads (PASSABLE terrain — `,` opens
  the store — yet the C Borg's spread skips a shop that is not its
  target, borg-flow.c:371-374). Threat placeholder until T3.3 lifts the
  C Borg's danger sum: an awake tracked monster costs its grid 12 and its
  neighbours 4, weighted by its actions per player action
  (`Knowledge::turns_per_action`), by `Track::confidence` and by a
  quarter when asleep.
- **Stale by construction.** `Paths` records the world snapshot
  (`turn`, `thinks`) and refuses any other; it is built per think and
  dies with it, so the stale-flow reuse of the mold deaths (bbc D0008,
  rs D0006) cannot happen. Within a think each goal's map is computed
  once.
- **Determinism:** DIRS order breaks step ties; no unordered containers.

**Evidence:** 9 unit tests (maps built from ASCII worlds: corridors,
doors, webs, walls, the unknown, shops, threat paid as a cost, faster and
sleeping monsters, the stale-world refusal); and, on a real game, the
recorded run's **302 thinks replays with every reachable goal stepping
strictly downhill onto a passable grid** (`recorded_walks_are_strictly_
downhill`, the same `BORG_*_LOG_CHECK` wiring CI already records). Play
is unchanged — no live-loop change at all: smoke 3/3, smoke-exact 3/3
and full 45/45 identical to the references. The new source hash
re-records the suites' deaths: D1035–D1080 (46 cases).

**Not checked:** the maps' *quality* (do these costs make the Borg choose
well?) is T3.3's, on play; the threat shape is a placeholder with no
claim (RULES 14). `remaining()` at the player subtracts the stood-on
grid's own cost; if a future caller finds that surprising, rename then.

**Next:** T3.3 (#19), the first strategy: modes, goals, budgets, with the
C Borg's danger sum lifted as the M3 placeholder and the executor (#17)
and paths (#18) beneath it.

## 2026-11-10: T3.1, the executor and prompts (#17)

M3 started (PLAN's row moved to *in progress*; the source refresh PR #108
landed first, see below). `borg-reborn-rs/src/executor.rs` is L6:

- **Actions to keys.** The `Action` vocabulary M3's strategy needs
  (T3.4/T3.5): `Rest(n)`, `Descend`, `Ascend`, `Walk(d)`, `Attack(d)`
  (the same key as a walk, named for its intent), `Get`, `Hold`,
  `Alter(d)`, `Buy{ware}`, `Sell{item, qty}`. Every action is one
  complete key sequence. Item letters come from the engine's own lists,
  checked at the pin: the pack's `all_letters_nohjkl`
  (`ui-menu.c:44`, labels at `ui-object.c:292`), the store's selections
  with the original keyset (`ui-store.c:805`). The store sequences are
  the C Borg's, lifted with citations
  (`borg-store-buy.c:borg_think_shop_buy:1206-1244`,
  `borg-store-sell.c:borg_think_shop_sell:1101-1146`): the whole store
  visit runs inside one `,` (`do_cmd_hold`, cmd-cave.c), costs no game
  time, and ends with the C Borg's `ESCAPE`s — one purchase per visit,
  which also sidesteps the store list's renumbering on a sale-out
  (borg-runner's minimaxflash-2#91).
- **The prompt catcher** runs in `next_key` before the key queue, as the
  C Borg's does (`borg.c:304-308`): `borg_react_prompted`'s table and
  the `-more-` test (`borg-messages-react.c:95-324`,
  `borg-messages.c:183-294`), answers unchanged, `"Die?"` excluded
  (GOALS D5). Two declared departures, both in the module docs and
  `docs/ENGINE.md`: we do not press a space to clear a normal message
  (the C Borg does, `borg-messages.c:288-290`; it would change every
  recorded key stream), and `auto_more` stays on (the catcher answers a
  `-more-` if one ever appears; the cost — a message batch that scrolls
  between thinks is not parsed from the screen — is declared, and T3.3
  revisits it if the strategy needs the full message stream). A third,
  cosmetic: the C Borg compares sub-screen lines with a trailing space;
  our screen read trims, so the trimmed form is used.
- **Expected effects.** Each action carries one (`Step`, `Time`,
  `Depth`, `PackGrows`, `PackShrinks`), checked by the next think
  against the new observation; a mismatch is a trace violation naming
  the action, and prompts answered between thinks are a new `prompts`
  field of each trace line (additive; old lines parse). The engine reads
  the catcher needs joined the D3 list: `inkey_flag`, the message line's
  first-cell attribute and the terminal cursor (`docs/ENGINE-READS.md`;
  the bindings were regenerated).

**Evidence:**
- Play is byte-identical to the references: smoke 3/3, smoke-exact 3/3,
  **full 45/45** (keys, messages, states). The catcher answered **0**
  prompts in the full suite's 72,096 thinks (the placeholder meets no
  prompts, as expected), and the effect check recorded exactly 5
  violations — refused steps the same-turn guard already caught, now
  named in the trace.
- Unit tests cover every key sequence, the whole prompt table (and its
  negatives: messages, blanks, `"Type"`, `"Die?"`, queue-not-empty
  guards) and each effect's hold and fail. `cargo fmt`, `clippy
  -D warnings` and the tools' 54 tests pass.
- The suites' deaths are recorded at this source as D0989–D1034 (46
  cases: the full suite's 45 and the smoke mage; the smoke-exact mage
  dedupes into the smoke case). The plays equal the references', but the
  Rust source hash is new, so they are new encounter families (RULES 12).

**Found on the way:**
- `birth_no_selling` defaults **true** in 4.2.6 (`list-options.h`):
  stores pay nothing, the sale prompt says "Give", and one `Enter` still
  confirms the hand-over. Sell's effect is therefore pack-based, not
  gold-based.
- A store visit charges no energy (`do_cmd_hold`: "turn will be taken
  exiting the shop" sets `energy_use = 0`), so no store action can
  expect game time to pass.

**Not exercised:** no suite run enters a store, picks anything up or
meets a y/n prompt — the placeholder never does — so `Buy`, `Sell`,
`Get` and the catcher's answers are unit-tested only until T3.4/T3.5
play them for real. The perception checks (CI's test build) were not
run locally; CI runs them on this PR.

**Next:** T3.2 (#18), paths and exploration. T3.3 (#19) replaces the
placeholder and will be the first play change since M1; new references
come with it.

## 2026-11-10: M3 source refresh

The M3 `--update` (RULES 21) moved five pins:
- borg-rs `286215c` → `8a7579d`: docs only (the milestone 9 adoption,
  its 0098 NULL guard);
- better-borg-c `eb9a78e` → `06f5789`: new deaths D0017–D0019,
  REDESIGN-LESSONS updated to cover them;
- borg-zoo `e56d5fa` → `a092d1e`;
- borg-agent `77e48c9` → `57a480c`;
- borg-runner `e829a01` → `956171b`: the knowledge page grew by 155
  lines, and three minimaxflash runs were added (notebooks and write-ups).

angband and neo-angband-tracker are at their pins.

**New deaths** (better-borg-c, read first; all original-C mechanisms,
input for M3's strategy and M4's threat model):
- **D0017**: a CL25 Ranger on DL21 escaped Mirkwood spiders (speed +10,
  poison) by staff at 7 HP, then rested for Recall while poisoned, with
  9 CCW and the SP for Herbal Curing carried. While `goal.fleeing` is
  set, caution never calls healing. A mode chosen for the fight blocked
  recovery from an ongoing status (REDESIGN-LESSONS L2's new
  requirement: re-evaluate a mode when the situation changes).
- **D0018**: a CL2 Druid, gold spent on armour and jewellery, one Phase
  Door bought and used at low danger, no CLW, Recall at home, walked
  away from a speed +8 wild cat for eight steps while it clawed the
  character down. The walking retreat judges each step by the pursuer's
  current square; a faster pursuer that keeps attacking is not modelled.
  The owner's verified retry died too.
- **D0019**: a CL3 Necromancer, no cure bought, Recall at home, fled a
  whole level over one apprentice (the CL ≤ 8 name list), spent its last
  Phase Door seven squares from the stairs, then meleed cutpurses for
  ~190 turns next to a dropped Phase Door it never picked up: attack
  runs before pickup. The owner's verified takeover survived.

D0018 and D0019 carry `owner-manual-branch.json`; D0019's branch
survived, so it joins the scenario-test list (PLAN.md) with D0010,
D0012–D0014.

**borg-runner's page** (`reference/borg-runner/manuals/INSTRUCTIONS.md`),
the M3 reading, gained: key-handling facts for the executor (never a
real newline — the frontend drops a trailing one; leave look/targeting
mode with ESC; never two commands in one ask after a movement command;
the store list renumbers when an item sells out, so buy the last row
first; the rest prompt is pre-filled and one backspace clears it); a
"depth is not the objective" section (town-and-depth-1 shuttle, town
restock daily, Word of Recall heals on arrival); where you stand (the
game begins on the town's down staircase; the stair under you is never
drawn, only the status line names it; one down staircase per level,
uniformly placed); and the experience arithmetic (a kill pays
`mexp × mlev / clev`; the early ladder; first casts and first uses
pay). FACT lines are checked against `engine/angband/src/` and cited
before use (RULES 16).

**New better-borg-c tags** `4.2.6-impact-v9` and `v9.1` (v8 was already
known): the zoo's catalogue at the new pin already lists the specimens
`4.2.6-bbc-m8` and `4.2.6-bbc-m9`, and a development suite
`classes-1800-200k` (200k turns). Nothing for this repository to do:
comparisons are the zoo's (SPEC §6).

**Next:** T3.1 (#17), the executor and prompts. PLAN's M3 row goes to
"in progress" in that PR.

## 2026-11-10: M2 reviewed and tagged

Kimi K3.1 Pro's fourth pass on #99, at `7cec235`: **M2 done**, with no
findings left. The owner had #99 merged (`reviewed` at `4ea0bb0`); `m2`
is tagged there, and the GitHub milestone is closed. The four passes'
findings were all fixed in #100–#104, so none carry over to M3. #103
(borg-runner as source S15, below) was held while the review ran and was
merged after it.

**Next session: start M3** (GitHub milestone 3, tasks #17–#24; SPEC §10
keeps M3 as PLAN.md describes it).
1. **Refresh the sources first**, in its own PR (RULES 21):
   `python3 scripts/refresh-sources.py`, then `--update`. On 2026-11-10,
   borg-runner was already 13 commits past its pin (`e829a01`), and its
   knowledge page is what M3's strategy reads.
2. **Then T3.1, the executor and prompts (#17):** the lowest-numbered open
   task, its Needs (#13, #15) closed. The M3 row of PLAN.md goes to
   "in progress" when it starts.
3. **Game knowledge for M3:** S14 (`reference/c-manual/`, chapter 15 for
   an automatic player) and S15 (`reference/borg-runner/manuals/
   INSTRUCTIONS.md`). Lines from either are checked against
   `engine/angband/src/` and cited (RULES 16), and none is a claim until
   the zoo measures it (RULES 14).
4. **The perception check stays on:** CI runs smoke and full on the
   `test-hallucinate,test-perception` build, and `tools/check-perception.py`
   must report 0 differences. A new engine read needs a row in
   `docs/ENGINE-READS.md` (`test_reads.py` fails otherwise).

**Known limits, declared and not fixed:** no recorded game has an
overfull pack, mixed cursed and uncursed stacks of one kind, the
unseen-monster-1 hallucination case, or a shop or web on an unknown grid,
so those fixes are covered only by the per-think checks. Floor objects and
store stock have no second route to compare against. Few monsters are met
(3,533 monster-thinks in smoke and full) until M3's Borg fights.

## 2026-11-10: a new source, borg-runner (S15)

The owner added [borg-runner](https://github.com/diegoami/borg-runner) as a
source of game knowledge: the accumulated knowledge of agent players.
Language-model agents play upstream 4.2.6 (`e473720`, our engine) from the
screen and record every key; what they learn goes into one page,
`manuals/INSTRUCTIONS.md`.
- **Pin:** `borg-runner` at `e829a01`. At that commit LEARNED.md and
  MANUAL-GAPS.md had just been folded into the page.
- **Copies** (`reference/borg-runner/`):
  - `manuals/`: the page, volume 2 (depth 5–20, untested) and the jev
    manual;
  - `knowledge/`: each run's notebook;
  - the `.md` write-ups of `records/`.

  `refresh-sources.py` has a new `suffix` selection for the last one.
- **Recorded in:** SOURCES (S15, with what we take and what we leave),
  GOALS D11 (borg-agent and borg-runner, same terms), README, AGENTS,
  reference/README.
- **How to use it:** each FACT line is checked against `engine/angband/src/`
  and cited (RULES 16). An OBSERVED line rests on a few runs, some of which
  inherited each other's notebooks. Nothing is a claim until the zoo has
  measured it (RULES 14).
- **First use:** the strategies and profiles of M3 onward, alongside S14.

## 2026-11-10: M2's third review: not done; findings fixed

Kimi K3.1 Pro's third review of #99, at `fb069d8`: **not done**. R2 and R4
are resolved. One blocking finding and three should-fix ones were new, all
fixed in the PR after #102:
1. **Blocking:** perception read `pack_size` inventory entries, but the
   pack has `pack_size + 1` and may be overfull by one. It now reads them
   all.
2. **Slots and levels:** the equipment's slot count and types are checked
   against the race's static body, and the occupied slots against the
   engine's; `Item.level` is compared.
3. **Pairing:** each engine object is paired with the observed item at its
   place and position. Each pair is compared field by field from the object
   directly, and whole against `analyze()` of that very object: exact
   weight, flags, modifiers, elements, description, awareness, runes. The
   description, awareness and runes come from the same engine functions as
   perception, so that half checks pairing and completeness, not those
   functions.
4. **The stat floor:** `wearing` takes the bonus nearest the known one
   (race, class, the gear's known modifiers). That is exact unless
   something unseen changes the stat at the floor. The reviewer's
   counterexamples are tests.

**Evidence:**
- Smoke and full on the test build: 76,175 thinks, 0 differences.
- The reviewer's faults now fail: slot types off by one, 48,948
  differences; zeroed item levels, 20,395.
- The normal build plays the references' games (3/3, 3/3, 45/45). Its
  deaths are D0943–D0988.
- **Not exercised:** no suite run has an overfull pack or mixed cursed and
  uncursed stacks.

## 2026-11-10: M2's re-review: done with changes; R1–R4 fixed

Kimi K3.1 Pro re-reviewed #99 at `eef7a8c`: **done with changes**. Both
blocking findings and the first round are resolved. Four remained, fixed
in the PR after #101:
- **R1:** the gear check is now place-aware. The engine says where each
  object is (`equipped_item_slot`, `object_is_in_quiver`), and each item
  is compared on its identity and known combat values, and known flags
  against bounds (superseded by the third review's pairing, below).
- **R2:** the pack count is compared with a fresh `pack_slots_used` call.
- **R3:** at the stat floor of 3, `wearing` takes the lowest result of all
  the bonuses that fit, so no stat is invented.
- **R4:** monsters are compared as multisets, so duplicates count.

**Evidence:**
- Smoke and full on the test build: 76,175 thinks, 0 differences.
- The reviewer's faults now fail: weapon and launcher swapped, 8,158
  differences; every monster duplicated, 48.
- The normal build plays the references' games (3/3, 3/3, 45/45). Its
  deaths are D0897–D0942.

**Next:** the owner merges #99 (or asks for another pass). Then tag `m2`,
the remaining findings (none open) become M3 issues, and M3 starts.

## 2026-11-10: M2's review (#99): not done; findings fixed

Kimi K3.1 Pro reviewed M2 at `70cb9b5`: **not done**, with two blocking and
six should-fix findings. They are fixed in #100 (findings 7, 8, doc nits)
and the PR after it (findings 1–6):
- **Finding 1 (blocking):**
  - **Fix:** the shop number and the web flag are read only where the
    player knows a shop or a floor, as the C Borg guards them.
  - **Check:** the truth check fails on any other shop or web.
- **Finding 2 (blocking):**
  - **Unknown grids:** a grid the player knows on the panel but the world
    doesn't is a difference.
  - **Panel:** the observed panel is checked against the engine's own
    `panel_contains`.
- **Finding 3:** monsters are compared field by field, and the gear item
  by item against the engine's gear list.
- **Finding 4:** every run needs one truth line per Borg decision, and a
  lost truth log fails the test build.
- **Finding 5:** `term_win` and `random` are in the types table, and a
  field-level audit fails on fields of unlisted types.
- **Finding 6:**
  - **To-dam:** the weapon and the launcher are left out of the gear's
    to-hit and to-dam; melee adds the weapon's own.
  - **Stats:** `wearing` recomputes the stats in use (`modify_stat_value`).
- **Findings 7–8 (#100):** `--runs` matches deaths by (family, keys);
  `make-reference.sh` refuses before writing anything.

**Evidence:**
- Smoke and full on the test build: 76,175 thinks, 0 differences.
- Kimi's three faults, which passed before, now fail: 219,103, 223,230 and
  96 differences.
- The normal build still plays the references' games (3/3, 3/3, 45/45),
  so the references stay. Its deaths are D0851–D0896.

**Next:** a re-review of #99 is the owner's call (RULES 6). #99's prompt is
updated to the new head.

## 2026-11-10: T2.5, perception against the engine; M2 complete

The owner closed E's review without a re-review. borg-zoo#48 now asks to
move `4.2.6-reborn-rust` to `f0eeee2`; the zoo check there gives the same
smoke play hashes as M1, and its death is D0804 (#97).

**T2.5 (#16):**
- **The check:** a test feature, `test-perception`, compares every think
  with the engine through other routes (`engine.rs`, below `ENGINE TRUTH`):
  - the world's remembered map against `player->cave`;
  - the engine's visible monsters on the panel against the monsters seen;
  - the gear list against the observed equipment, quiver and pack;
  - our pack count against `pack_slots_used`.
  `tools/check-perception.py` fails on any difference. It also checks the
  status against the frontend's `state.log`.
- **Results:** smoke and full, 76,175 thinks, have 0 differences.
  - Declared, not failed: 220,350 grid-thinks the player knows but the
    world never saw (218,978 in full, 1,372 in smoke), and 96 visible
    monsters off the panel. The C Borg
    reads only the panel too (`borg-update.c:264`).
  - An injected fault gives 8,205 differences, so the check can fail.
- **One defect found:** while hallucinating, `map_info` puts `m_idx = 1`
  on an empty grid. When monster 1 stood there unseen, the Borg took it as
  seen; 11 of 45 full runs failed the hallucination assertion, and CI had
  run only smoke. Perception now also needs `monster_is_visible`
  (`map_info`'s own test, a B read). Our suites play the same games
  (references unchanged: 3/3, 3/3, 45/45), since none of their runs met
  that case.
- **D3:** reason T for the check's reads, used only below the marker
  (`test_reads.py`). The types read field by field are listed (E review,
  finding 10).
- **CI:** one test build with both features runs smoke and full; its play
  equals the references, and `check-perception.py` passes.
- **Deaths:** D0805–D0850.

**M2 is complete:** T2.1–T2.5. Next is the review PR (`main` → `reviewed`)
with its prompt; the owner chooses the reviewer (RULES 6). M3 doesn't start
before that.

## 2026-11-10: E's review is in; fixes in three PRs

Kimi's review of E (on #92) has ten findings. They are fixed in three PRs
into `main`:
1. **The frontend (#95, RULES 22):** `loaded` and `exact_turns` are in
   every summary (finding 5).
   - Play doesn't change: keys, messages and states equal the references
     (smoke 3/3, smoke-exact 3/3, full 45/45).
   - `harness/diff` compares the two fields when both sides have them, as
     it does `game_options`.
   - Its deaths are recorded at its source (D0712–D0757).
2. **The code (#96):** suites, perception while hallucinating, the death and
   reference tools.
   - References are regenerated: the full suite's `menu` fix changes
     exactly the 20 runs whose character changes.
   - Smoke, smoke-exact and the other 25 full runs are unchanged.
   - Its deaths are D0758–D0803.
3. **Documents, rules and CI (#94):**
   - findings 6, 7, 9 and 10's document nits;
   - RULES 12 narrowed by the owner: measured runs only, not watched games;
   - CI fetches upstream `e473720`, so the tree check no longer skips.

**Left:**
- ENGINE-READS' types table (`term`, `monster`, `player_class`, `store`):
  T2.5's D3 recheck (#16).
- borg-zoo#48: `4.2.6-reborn-rust`'s `ref` moves to the commit that merges
  #94, not to `e`, since #96 changes the full suite's play (the zoo's own
  cells pass `-race`/`-class` with `-menu-letters`; whether its play
  changed is for the zoo's check).
- Whether the fixes get a re-review is the owner's call (RULES 6).
- Then T2.5 (#16) and M3.

## 2026-11-09: E merged before its review; a review after the fact

The owner had the session merge #92 (`main` → `reviewed`). Tag `e`
(`0a92da0`) was pushed, milestone E closed and borg-zoo#48 filed (move
`4.2.6-reborn-rust` to `e`), all **before any review**. The owner then
asked for the review after the fact:
- the reviewer, the owner's choice, reviews `55d7476` (the tree of `e`) and
  posts on #92;
- findings are fixed in PRs into `main`;
- tags are not moved (RULES 5);
- borg-zoo#48's first request is on hold until the review is clean.

Other work (T2.5, #16) may go on meanwhile.

## 2026-11-09: TE.9 and milestone E complete; E's review

- **TE.9:** `engine/tests/test_independence.py` (CI, rust job) checks that
  no build, test, check or CI step uses borg-rs. That means no checkout,
  crate, staticlib or former `angband/` tree, and no clone or download;
  provenance comments are allowed. It also checks that the workspace holds
  only our crate. CI now proves every point of SPEC §10's exit:
  - the tree is upstream's, the patches apply, and the source list matches;
  - the frontend contract, the zoo contract and the D3 list hold;
  - the bindings regenerate unchanged;
  - runs are deterministic, match the references, and keep the boundary
    contract;
  - the death reconciliation passes, and a CI death reruns with the same
    keys;
  - nothing reads borg-rs.
- **Milestone E is complete** (#81–#91). The review PR `main` →
  `reviewed` is open. The owner chooses the reviewer (RULES 6); the prompt
  is in the review PR's description.
- **After the owner merges it:**
  - tag `e`;
  - ask borg-zoo (an issue, RULES 7) to move `4.2.6-reborn-rust`'s `ref`
    to `e^{}` and to decide on the two C specimens;
  - continue with the rest of M2 (T2.5, #16), then M3.

## 2026-11-09: S12 retired (pin PR, #78)

The owner retired S12, the draft strategy manual, in favour of S14, the
complete one. This pin PR (RULES 21) removes the `strategy-manual` pin, its
copy rule and `reference/strategy/`, and updates SOURCES, reference/README
and AGENTS.md. Its local clone, `vendor/neo-angband-tracker-workbench-c-borg`,
is no longer used.

## 2026-11-09: TE.8, removals, documents and rules (#78)

- **Removed:**
  - the C arms' references (`harness/reference/smoke-c`, `smoke-pristine`,
    `full-pristine`, `smoke-c-bbc-m7`);
  - `pins.json`'s `controls` and borg-rs's oracle role, `borg_dir` and
    `tag_of_interest`;
  - `docs/ENGINE-TREE.md` (replaced by `docs/ENGINE.md`);
  - `suite.sh`'s dead control-arm branch.
- **Rewritten for docs/SPEC.md:**
  - RULES 9 (the tree is upstream's bytes plus our patch series), RULES 16
    (copies are recorded in PROVENANCE and owned) and RULES 22 (comparisons
    are the zoo's; a change to the integration reproduces the references
    or explains every difference);
  - GOALS principle 4, D2, D4 and D8, plus a superseded note on the "same
    seam, same harness as borg-rs" principle;
  - ARCHITECTURE L0, *What comes from where*, and the settled open
    question;
  - SOURCES S5 (borg-rs is a reference), with a historical note on its
    section;
  - README (intro, status, SPEC first in the table);
  - AGENTS.md (intro, start commands, the `engine/` and `harness/` rows,
    the `borg-rs/` row gone);
  - `harness/README.md` (rewritten for our harness);
  - the refresh report's wording (a new better-borg-c tag is a specimen for
    the zoo to consider).
- History stays history: PLAN's M1 row, the handover's earlier entries,
  `docs/ZOO-CHANGE-REQUEST.md`, and the M1-era rows of SOURCES.
- **S12's retirement** is its own pin PR (RULES 21), next.

## 2026-11-09: TE.7, the zoo contract (#77)

- **`docs/ZOO-INTERFACE.md`:** the contract the zoo's borg-rs adapter
  (`zoo/borgrs.py` at our pin `e56d5fa`) calls. It covers the build
  (`harness/build.sh`, `build/rust/game/`, `BUILD_ID`'s `binary_sha256` and
  `rustc`, `harness/borg.txt`), the run (`-mborgh` and its flags,
  `TZ`/locale), and what the zoo reads (`summary.json`'s 10 fields,
  `keys.log`, `state.log`). One mismatch is on the zoo's side: its `STOP`
  map knows `clev`, while our frontend writes `clevel`. It's harmless,
  since the zoo never passes `-clev`.
- **`engine/tests/test_zoo_contract.py`:** runs the binary with exactly the
  zoo's command line and checks every field and log format, plus that
  `BUILD_ID` matches the binary and another `BORG_IMPL` fails loudly. CI's
  engine job runs it.
- **`harness/zoo-check.py`:** checks only the Rust arm. End to end with the
  zoo's adapter: 3/3 cells with no harness failure or config error, and
  **play hashes identical to M1's**. The seed-29 death is recorded as D0711.
- **Retired:** `tools/endpoint.py` and its test (borg-zoo computes every
  statistic). GOALS D7 is amended.
- **After E is merged:** ask borg-zoo (an issue, RULES 7) to move
  `4.2.6-reborn-rust`'s `ref` to E's tag and to decide on the two C
  specimens.

## 2026-11-09: TE.6, our death tools; the copied tooling and borg-rs's tree gone (#76)

- **Decision (owner):** the death tooling to keep imported borg-rs's whole
  recording toolkit (GUI, replay viewer, runtime: about 2,000 lines), so
  all copied tools are dropped and replaced by lean tools of ours.
- **New:**
  - `tools/rerun-death.py` reruns a dossier's game from its build and seed
    with the observation, world and knowledge logs and checkpoint saves,
    and checks that the keys are the recorded ones. D0664 reproduces, and
    CI reruns one of its own deaths.
  - `tools/death_registry.py` regenerates the registry index, taken from
    `borg_deaths.py` with the same output (byte for byte).
  - `watch-borg.sh` builds our integration with X11 and opens the game.
- **Removed:**
  - the copied tools and their tests (25 Python files, 2 C files, 2
    headers) and 7 root launchers;
  - borg-rs's `angband/` tree, the `borg-rs/` shim crate and
    `harness/measurement/`;
  - `pins.json`'s `engine_tree` and the re-sync report;
  - the repository test `engine_tree_matches_its_record`;
  - `run.sh`'s `frames` flag.

  No file of borg-rs's engine tree is left, and no build reads one.
- **Docs:** README's *Playing*, AGENTS.md, `tools/README.md`,
  `docs/PROVENANCE.md` and SPEC §8 and §11 are updated.
- **Deaths:** the Rust deaths are recorded again, as D0665–D0710
  (`Cargo.toml` is a build input and lost the shim member). Play still
  matches the references: full 45/45, smoke 3/3.

## 2026-11-09: TE.5, the harness and run contract on our build (#75)

- **`run.sh`:** the C Borgs' flags (`cheat`, `cheatlive`, `munchkin`,
  `lunal`, `selflunal`, borg.txt rewrites) and the dropped frontend options
  (`frames`, `rune-pattern`) are gone. `BORG_IMPL` defaults to `rust`, and
  the run's `user/borg.txt` is written as before (the zoo copies the file;
  its hash is in a death's family). The environment is fixed (`TZ=UTC`,
  locale).
- **Option set:** the frontend writes every game option in force into
  `summary.json` (`game_options`), and `docs/ENGINE.md` declares the Rust
  Borg's options against the C Borg's.
  - **Found:** 4.2.6 has no `easy_alter`. The Rust Borg had been setting it,
    and the engine ignored it silently. It's removed, and an unknown option
    now stops the Borg (`GAME_OPTIONS` in `borg.rs`). Play can't change,
    since the call never did anything.
- **Reproduction:** the Rust Borg on our build reproduces the borg-rs-tree
  outputs: full 45/45, smoke 3/3, smoke-exact 3/3, hallucination smoke 3/3.
  `harness/diff` compares `game_options` only when both sides have it.
- **References:** `harness/reference/smoke-rust`, `full-rust` and
  `smoke-exact-rust`, made by `harness/make-reference.sh` with a
  `REFERENCE` file (source, binary, profile, borg.txt hashes,
  environment). CI compares smoke and smoke-exact with them. The C arms'
  references go in TE.8.
- **Deaths:** the Rust deaths are recorded again at the new source, as
  D0619–D0664.

## 2026-11-09: TE.4, our build (#74)

- **The build:** `engine/CMakeLists.txt` builds the Rust Borg from the
  pristine tree, our patches, our frontend and the crate's own staticlib.
  `harness/build.sh` (`BORG_IMPL=rust`) uses it and is rewritten; the build
  takes about 10 s.
- **Reproduction:** the Rust Borg plays **byte-identical** keys, messages
  and states to its build on borg-rs's tree (smoke 3/3, full 45/45,
  smoke-exact 3/3, hallucination smoke 3/3). The boundary contract holds.
  So the patch series and our frontend are a complete harness.
- **Gone:** the C arms (`c`, `c-pristine`, `c-bbc-m7`) from the build and
  CI, and `harness/control-overlay.py`. `harness/identity.sh` knows only
  `rust`, and its inputs are `engine/`, the crate, `Cargo.toml` and
  `Cargo.lock`.
- **Kept until TE.6:** borg-rs's `angband/`, the `borg-rs/` shim crate and
  `harness/measurement/`. The copied death tooling still builds its own
  binary from them (#76 updated).
- **Tests:** `engine/tests/test_build.py` checks the source list against
  upstream's and the input list, and runs the binary through `run.sh` for
  the frontend contract. CI's engine job runs it.
- **Deaths:** the Rust deaths are recorded again at the new source, as
  D0573–D0618 (the build inputs changed).

## 2026-11-09: TE.3, generated bindings and the D3 list (#73)

- **Generated bindings:** `borg-reborn-rs/src/bindings.rs` is generated by
  bindgen 0.73.2 from the pristine headers (`engine/bindings/`), limited to
  the symbols the Borg uses. It has 66 compile-time layout assertions
  against the C structs. The c2rust `headers/` copied from borg-rs are
  gone.
- **`engine.rs`:** re-based on the bindings. Its hand-written declarations
  of engine symbols are gone, except `inkey_hack`, which no upstream header
  declares.
- **The D3 list:** `docs/ENGINE-READS.md` lists every symbol with its
  reason (screen, direct read, static data, declared extra, write, seam).
  `engine/tests/test_reads.py` checks it both ways, and that only
  `engine.rs` touches the bindings.
- **CI:** installs libclang and bindgen and runs `generate.sh --check`.
  Locally this needed `clang`, `libclang-dev` (owner, apt) and `cargo
  install bindgen-cli`.
- **Play:** unchanged (Rust smoke 3/3, full 45/45). The Rust deaths are
  recorded again at the new source, as D0527–D0572.

## 2026-11-09: TE.2, our headless frontend (#72)

- **The frontend:** `engine/frontend/main-borgh.c` and `.h` are copied
  from borg-rs's borgh and trimmed to the contract of SPEC §3.5 (1,247 →
  931 lines). Dropped:
  - `-states` (native observer);
  - `-frames` and its diagnostics;
  - fixture building (`-make-fixture`, `-near-morgoth`);
  - `-stop-rune-pattern`;
  - `-cheat-live` (D5).

  Kept: every contract flag, the stall and key limits, `-clev`,
  `-random-birth`, the save options and checkpoints (the death tooling may
  use them).
- **The test:** `engine/tests/test_frontend.py` checks the flags, the
  `summary.json` fields, the mode registration, and a compile against the
  patched tree.
- **Not yet:** no binary until TE.4. `harness/run.sh` flags that used the
  dropped options (`frames`, `cheatlive`, `rune-pattern`) are cleaned up
  in TE.5.

## 2026-11-09: TE.1, the upstream tree and our patch series (#71)

- **The tree:** `engine/angband/` is upstream `e473720`, byte for byte
  (tree `99a0549`, `engine/UPSTREAM`), extracted from upstream, not from
  borg-rs.
- **The patches:** `engine/patches/` holds 7 patches for our harness hooks,
  every changed line guarded by `BORG_HARNESS` or `USE_BORGH`, plus one
  declared split of a declaration in `mon-make.c`.
- **Docs and tests:** `docs/ENGINE.md` and `docs/PROVENANCE.md` are new.
  `engine/tests/test_engine.py` checks the tree, that the series applies,
  and that with the guards off the code is upstream's. It runs in CI.
- **What stays:** borg-rs's `angband/` stays until TE.4 switches the build;
  nothing builds from `engine/` yet.

## 2026-11-09: milestone E planned

PLAN.md follows docs/SPEC.md §10. **E. Our own integration** is in
progress: the GitHub milestone E, tasks #71 (TE.1) to #79 (TE.9), linked by
their Needs. M2 is paused (T2.1–T2.4 done; T2.5, #16, needs E). Order:
- TE.1 tree and patches, then TE.2 frontend and TE.3 bindings;
- TE.4 build, then TE.5 harness and the reproduction check;
- TE.6 death tooling, TE.7 zoo contract and TE.8 removals and documents;
- TE.9 CI and E's review (the owner chooses the reviewer).

## 2026-11-09: the spec is in effect; only our Borg

The owner merged #66 (`main` → `reviewed`, `37bcb97`), so `docs/SPEC.md`
is in effect. There is no tag: it is not the end of a milestone. The owner
then decided:
- **borg-zoo does every comparison.** This repository delivers only our
  Borg, through the zoo contract. The C arms (`c`, `c-pristine`,
  `c-bbc-m7`), their overlays and references, `tools/endpoint.py` and #56
  (`c-bbc-m8`, now borg-zoo#36) are dropped in milestone E. GOALS D7 and D8
  change accordingly.
- **§11:** keep the death tooling and drop the GUI, replay and takeover
  tools; keep the zoo's current interface, written down; the engine-change
  question goes with the controls.

The `borg_self_scum = TRUE` finding concerns the zoo's
`4.2.6-reborn-c-*` specimens, which were built from this repository. It
is reported on borg-zoo#30.

Next: plan milestone E as issues (PLAN.md re-planned), then build it.

## 2026-11-09: the owner chooses the reviewer (RULES 6)

The review of #66 was started by the session: MiniMax-M3, picked through
the quota service. The owner then ruled: "next time give me the prompt and
I will chose the reviewer". RULES 6 and AGENTS.md (*Models*) now say so.
For every review the session writes the prompt and the owner chooses and
runs the reviewer. A second pass on #66 (for #67 and #68) therefore waits for
the owner's choice.

## 2026-11-09: review of #66 (MiniMax-M3): spec findings

The spec findings S1–S10 and the missed risks are answered in
`docs/SPEC.md`:
- **S1:** the zoo contract is written down. It covers the build entry point
  and layout, `harness/borg.txt`, and the frontend's mode `borgh`, flags
  and `summary.json` fields. The harness may change everywhere else.
- **S2:** the frontend registration (`main.c`, `main.h`) joins the patch
  series, and the line counts are corrected.
- **S3:** our CMake carries a copy of upstream's source list, checked
  against it; upstream's file can't be used as it stands.
- **S4, C2:** the configuration (the C Borgs' `borg.txt` with every D5
  exploit off, the game option set) and the environment (`TZ`, locale)
  become part of the run contract.
- **S5:** E's exit requires the new integration to reproduce today's
  `c-pristine` and Rust smoke and full keys before any old reference is
  replaced.
- **S6:** `docs/ENGINE-READS.md` is normative; channel A gains the
  message line's attribute and cursor; the four extra reads are declared.
- **S7, S9:** the inventory and the document lists are completed.
- **S8:** `c-pristine` is a control, not an oracle.
- **S10 and the risks:** configuration hashes and what CI must prove are
  in E's exit; the death registry's two generations are described.
- The reviewer's answers to §11 are recorded as recommendations; the owner
  decides.

**A finding for the owner (S4):** `harness/borg.txt` has
`borg_self_scum = TRUE`, upstream's default too. So in every control run so
far, a C Borg could money-scum for a ware from character level 10
(`borg-store-buy.c:141-153`), which D5 excludes. Its effect on M1's
numbers (c-pristine 1/45 and c-bbc-m7 2/45 at the full-exact cap) has not
been measured. The configuration is corrected in milestone E.
## 2026-11-09: review of #66 (MiniMax-M3): code findings

MiniMax-M3 reviewed the spec and the work since `m1` on #66: the spec is
"yes, with changes", and the code is sound. Code findings:
- **C1, fixed here:** `Character::blows` now follows the engine for a
  weapon too heavy to wield (one blow, `calc_bonuses:2296-2318`, with
  `adj_str_hold` lifted and checked against the engine's copy), and adds
  bloodlust's `timed / 20`. No recorded run heavy-wields yet, so the rule
  is unit-tested only.
- **C5, fixed here:** `with_borg_rng` restores the game's RNG through a
  drop guard, on every path.
- **C2** (the Rust Borg's game options differ from the C Borg's) goes to
  the spec's run contract.
- **C3** (a key aborts the Borg in a shop, which the C Borg does not) is for
  M3, when the Rust Borg first enters shops.
- **C4** (flushing the trace each think) is kept: a dossier needs the trace
  on disk when the game ends, and one write costs microseconds against
  borgh's stall limit of seconds.

The Rust deaths are recorded again as D0480–D0525.

## 2026-11-09: the spec (docs/SPEC.md), and a stop

The owner decided to move completely away from borg-rs. The game stays
upstream C Angband. Our own integration (hooks, frontend, bindings, build,
harness) is built first, and borg-rs is only a reference to copy from.
Before anything more is implemented, `docs/SPEC.md` states that project:
- the parts it keeps, replaces or drops (§8);
- the documents and rules that change (§9);
- a re-plan with a new milestone E, our own integration, first (§10);
- four open questions (§11).

The owner accepted it and sent it to review with the work since `m1`. It
takes effect when the owner merges that review PR. Until then: **no
implementation**, and the M2 task list after T2.4 (T2.5, then M3+) is on
hold for the re-plan.

## 2026-11-09: owner decisions

Asked again and answered on 2026-11-09:
1. **c-bbc-m8** (#56) runs with `borg_munchkin_by_class = FALSE`, so every
   arm keeps D5. Its label must say so. GOALS D8 is amended.
2. **The `smoke-c-bbc-m7` reference** is accepted as a self-produced
   regression reference (harness/README.md).
3. **S12 is retired** in favour of S14. The pin, its copy rule and
   `reference/strategy/` go in their own pin PR (RULES 21).
4. **T2.5: the engine is the truth, not borg-rs** (option D). borg-rs's
   perception can't be run on its own: it lives inside its decision loop.
   Perception is checked against engine facts reached by other routes
   than the Observation's reads, and every engine read is listed (the D3
   audit). GOALS D8, ARCHITECTURE L1, AGENTS.md and #16 are amended.

## 2026-11-09: T2.4, decision trace and parameter table (#15)

- `params.rs`: the parameter table has name, unit, range, default and
  the evidence for the default. It holds the minimal Borg's three numbers:
  `max_tries_per_turn` 2, `rest_below_hp_pct` 70 (integer percent, so
  `hp * 100 < max * 70` is the old test exactly) and `rest_turns` 75
  (`Action::Rest(n)`, whose keys are still `R75` Enter).
- Profiles are `name = value` files in borg.txt's syntax, read at start
  (`BORG_PROFILE`; run.sh defaults to
  `harness/profiles/reborn-default.txt`). An unknown name, a malformed
  line or an out-of-range value stops the game.
- The profile's SHA-256 is in `config.txt` (run.sh), `params.json` and
  every trace line.
- `trace.rs` plus `decide_with`: the decision now carries its goal,
  budget, scored candidates and violations; the guard of principle 6 is
  recorded as one. The Borg writes `params.json` and `trace.jsonl` to
  the run directory (`BORG_RUN_DIR`, from run.sh, Rust arm only). The trace
  is written after the decision from what the decision made, so play is
  unchanged (full 45/45, smoke 3/3, hallucination smoke 3/3 identical).
- Dossiers keep `params.json` and `trace-tail.jsonl` (the last 200
  thinks), and `record.json` gains `identity.profile_sha256` and
  `decision_trace`. CI checks the trace and parameters in run outputs and
  in Rust dossiers.
- The Rust deaths are recorded again as D0434–D0479, now with traces. The
  C arms' outputs and dossiers are unchanged.

## 2026-11-09: T2.3, knowledge tables (#14)

`borg-reborn-rs/src/knowledge.rs` holds the L3 `Knowledge`. The Borg reads
it once from the engine's parsed data at its first think
(`engine::knowledge()`):
- monster races (spoiler data, GOALS D3: level, speed, HP, AC, sleep,
  frequencies, race and spell flags, and blows with method, effect, power
  and dice);
- object kinds, egos and artifacts;
- classes with their blows parameters and spell books;
- player races;
- what each store always and normally stocks;
- the engine's `extract_energy` table and `move_energy`.

Lifted, with citations at the pinned engine:
- `stat_index` (`calc_bonuses:2065-2073`);
- `blows` (`calc_blows:1703-1735`, with `adj_str_blow`, `adj_dex_blow` and
  `blows_table`). better-borg-c milestone 8's fix to the C Borg's copy
  (`668ac32`, weigh one weapon, not the stack) agrees with the engine, so
  the engine's formula is used.

Uses:
- `Knowledge::turns_per_action` replaces the placeholder speed rule in
  `Track::confidence`;
- `Character::blows` and `melee_per_turn` give expected damage per turn for
  any weapon (blows times mean dice plus known to-dam, before hit chance,
  criticals and slays, which belong to M4's threat model).

The Observation gained the engine's own `stat_ind`, `num_blows` and the
O-combat option, as checks. Tests:
- unit tests use the C values worked by hand;
- `BORG_KNOWLEDGE_LOG=FILE` dumps the knowledge, and CI checks it against
  monster.txt and class.txt values and against the engine's `adj_str_blow`;
- on every recorded think our stat indices and blows equal the engine's:
  locally 595 thinks over all 9 classes, and in CI the Warrior run.

Play is unchanged (smoke 3/3, full 45/45, hallucination smoke 3/3). The
Rust deaths are recorded again as D0388–D0433.

## 2026-11-09: T2.2, the world model (#13)

`borg-reborn-rs/src/world.rs` holds the L2 `World`. It changes only in
`World::update(&Observation)`, once per think. It has:
- the current level's map memory: the feature last shown, when the grid
  was last in view, the remembered and the real top object, trap and web.
  An unknown grid never erases what was known. Leaving the level (a depth
  or level-size change) forgets it, as the C Borg does;
- monster tracks keyed by the game's monster index: race, position, first
  and last seen, HP, timers and speed. An index reused for another race is
  a new monster. A track out of view is kept, and `Track::confidence`
  falls with the time since it was seen and with the monster's speed
  (asleep, a quarter as fast). Its shape is a placeholder for M4's
  calibration;
- the town's shop entrances (kept across levels) with the last visit (the
  store header "Stor"/"Home" while standing on the entrance), and every
  store's stock and the home's, as last read;
- the character model: the latest status and gear, plus `GearBonuses`
  (to-hit, to-dam, AC, known flags and modifiers, and resistances combined
  as `calc_bonuses` does). `Character::wearing` gives a changed copy for
  "try this on", and `pack_free` uses the engine's own count.

The Observation gained the engine's pack count (`pack_slots_used`,
`z_info->pack_size`) and every store's stock (`borg_notice_store`: each
ware through the item analysis, with the shop's `price_item`).
`BORG_WORLD_LOG=FILE` writes the world's digest after every think. CI
records a 3,000-turn run with both logs, and checks that replaying the
observations without the game rebuilds the live world think by think (302
of 302 locally). Play is unchanged (Rust smoke 3/3, full 45/45 and
hallucination smoke identical), but the store reads and the update make the
suites about 25% slower. The Rust deaths are recorded again as
D0342–D0387.

Left for later tasks: expected damage per turn needs blows and the other
formulas of L3 (T2.3). Item curses and activations are not observed yet.
The strategy doesn't read the world until M3.

## 2026-11-09: T2.1, the Observation (#12)

`borg-reborn-rs/src/observation.rs` holds the L1 `Observation`. It is
plain, serde-serialisable data, one per think, with what the C Borg's two
channels give it (spec §1.1, GOALS D3):
- from the screen: the message line (row 0, split into messages by
  `parse_messages`) and the store header (row 3, columns 1–4, raw: the
  C Borg compares it with "Stor"/"Home");
- from direct reads: status and timed effects, every equipment slot, the
  quiver and the pack through the C Borg's item analysis (`ODESC_FULL`
  descriptions, the known part unless the item is fully known), the
  class's spells (`spell_flags`, base fail), the map panel through
  `map_info()` (clipped to the level, with known trap, shop number and
  web), the monsters `map_info()` shows with their real race, hit points,
  timers and speed, and the real kind of the top object that is not
  ignored on each grid with a seen object.

`engine::perceive()` does the reads, so `engine.rs` is still the only
`unsafe` module. The Borg perceives on every think, after the minimal
Borg's own reads. The placeholder strategy does not use the observation
yet, and play is unchanged (Rust smoke 3/3, full 45/45 and hallucination
smoke identical). `BORG_OBSERVATION_LOG=FILE` writes one JSON line per
think, about 270 KB each. CI records a 500-turn run and checks that every
line round-trips canonically. New dependencies: `serde` and `serde_json`.
The Rust deaths are recorded again as D0296–D0341.

Not in it yet, for T2.2–T2.5: the store's own inventory (when the Borg is
in a shop), item curses and activations, monster messages beyond the
line, and the oracle comparison with borg-rs's `BorgState` (T2.5).

## 2026-11-09: engine tree re-synced from borg-rs 286215c

`angband/` is now borg-rs `286215c:angband` (tree `3fc0c9f`, split
`6581b4f`), merged as a subtree (docs/ENGINE-TREE.md). Only
`angband/src/borg/` changed: borg-rs carried better-borg-c milestones 7
and 8. So the tree's own C Borg (`BORG_IMPL=c`, never a control) goes
from impact v6 to v8. It includes `borg_munchkin_by_class`, TRUE by
default: Warriors, Druids and Rangers take the munchkin start. The engine
and borg-rs's generated headers are unchanged.
- `harness/reference/smoke-c/` is replaced by a smoke run of borg-rs's own
  C build at `286215c`. Ours reproduces it byte for byte (3/3), the same
  import check as T1.2.
- A re-sync starts a new measurement baseline (RULES 22). The source hash
  of every arm includes `angband/`, so deaths from runs after this PR
  are new encounter families.
- `harness/borg.txt` is unchanged. Whether a control should run with
  `borg_munchkin_by_class = FALSE` (GOALS D5) is asked on #56: better-borg-c
  v8 has the option too.

## 2026-11-09: M2 source refresh

The M2 `--update` (RULES 21) moved five pins:
- borg-rs `e5cb263` → `286215c`: two docs changed, and the engine tree
  re-sync is available (next PR);
- better-borg-c `b1588cc` → `eb9a78e`: new deaths D0015 and D0016;
- strategy-manual `e67fd7d` → `86349ca`;
- borg-zoo `3f6b76e` → `e56d5fa`;
- borg-agent `9c58125` → `77e48c9`.

angband and neo-angband-tracker are at their pins.
- **S12 repointed**: the pin `strategy-manual` now follows
  neo-angband-tracker `workbench/c-borg` (its own clone
  `vendor/neo-angband-tracker-workbench-c-borg`), since `c-borg-reference`
  is now pristine upstream. The manual is complete there: chapter 3
  (`03-magic.md`) is new. S12 stays the draft of S14 until the owner
  retires it.
- **New deaths** (better-borg-c, both CL1 Necromancers, original-C
  mechanisms):
  - D0015: Fang, in a dark dead end seven squares from an up staircase.
    The C Borg forbids the stair dash from Grip or Fang, while another rule
    lists Fang as a reason to leave the level.
  - D0016: a fast crow after a teleport trap. The starting gold went on
    armour and unidentified rings, Recall stayed at home, and two known
    Cure Light Wounds were never offered while fleeing.

  Both are input for M4 (escapes, the tactical layer: retreat to stairs,
  healing while fleeing) and M5 (readiness: a survival stock before
  descending).
- **borg-zoo** at `e56d5fa` has borg-zoo#30 done (borg-zoo#32): the
  specimens `4.2.6-reborn-rust`, `-c-pristine` and `-c-bbc-m7` use
  `ref` `2dd11fd`, with `exact_turns`, the depth stream and kept death
  evidence. A comment on borg-zoo#30 gives them the `m1` commit
  (`fab7155`), and says what changed since `2dd11fd`: no keys change in
  our suites, but the Rust source hash does.

## 2026-11-09: a new control arm, c-bbc-m8 (#56)

better-borg-c tagged milestone 8 (`4.2.6-impact-v8`, `017e038`) before
our better-borg-c pin moved past it. The refresh report listed only tags
after the pin, so it never showed the tag. It now lists tags after the
baseline in use (`baseline.commit`, else the pin), which flags v8. Under
RULES 22, v8 is a new arm, `c-bbc-m8`, filed as #56 in M4, the first
milestone that measures against the controls. `c-bbc-m7` stays as it is,
and the GOALS wording "`c-bbc-m7` today" changes when #56 lands.

## 2026-11-09: M1 done; M2 starts

The owner merged the M1 review PR #39 (`main` → `reviewed`, merge
`fab7155`) after four review passes by Kimi K3.1 Pro. Tag `m1` is
annotated and points to that commit. Its message says "merged 2026-11-10",
following the dates of the entries below, but the merge was on
2026-11-09; tags are not moved (RULES 5). Those entries, marked
2026-11-10, are from 2026-11-09 too. The GitHub milestone M1 is closed,
and PLAN.md marks M2 in progress.

After the review, as the owner decided:
- RULES 7 now lets a session open issues, and keep its own issues up to
  date, on borg-zoo, borg-rs, better-borg-c, neo-angband-tracker and
  borg-agent, when the owner asks or when this repository needs a change
  there. Nothing is pushed or merged there, and angband/angband stays
  off limits.
- The zoo change request is
  [diegoami/borg-zoo#30](https://github.com/diegoami/borg-zoo/issues/30).
  docs/ZOO-CHANGE-REQUEST.md names the `m1` commit `fab71554fd32`.
  The issue's prompt is updated to that exact commit after this PR is
  merged, because its old fallback (`4236081`, unpeeled tag) is obsolete.

Found while re-reading RULES: commits and PR bodies since M0 carried
`Co-Authored-By` and "Generated with" lines, which RULES 4 forbids. They
stay, since published commits are not amended (RULES 5), and no new
commit has them.

Next, in M2 (each its own PR):
1. The source refresh: `--update` for better-borg-c (new deaths D0015 and
   D0016) and the other moved pins. Repoint `strategy-manual` (S12) to
   neo-angband-tracker `workbench/c-borg` so the copy keeps a source; the
   owner may still retire S12 in favour of S14.
2. The borg-rs impact-v7 engine re-sync (docs/ENGINE-TREE.md).
3. Then #12 (T2.1, Observation) and the rest of M2 in order.

Still open for the owner: whether the self-produced `smoke-c-bbc-m7`
reference is acceptable, and S12's retirement.

## 2026-11-10: M1 fourth review pass, findings S1–S2 (#39)

The fourth pass (Kimi K3.1 Pro, at `eff27c9`) resolved R1, R2, R4, R5 and
R6, and partly resolved R3. It added two should-fix findings in R3's new
extra-option load path; nothing blocks. Both are fixed in #54. run.sh now
handles `-- -load FILE` like `save=NAME`: it copies the file to
`OUT/save/input.save`, passes that absolute path to borgh, and drops `-n`.
Before, borgh received `-n`, so it started a new character that the summary
called loaded (S1). A relative path also failed after run.sh's `cd` (S2).
`harness/check-extra-load.sh`, now in CI, checks that a relative extra
load resumes the saved game and plays the same keys as `save=NAME`.

## 2026-11-10: M1 third review pass, findings R1–R6 (#39)

The third pass (Kimi K3.1 Pro, at `42ee347`) resolved N1, N3, N4, N5 and
N8, and resolved N6 and N10 for their cases. It partly resolved N2, N7, N9
and N11, and added six should-fix findings, R1–R6. Nothing blocks. The fixes
go in three PRs into `main`:
- #51, R2: a step off a shop can be refused without time (an afraid player
  walking into a visible monster), so on a shop `stand_still()` now alters
  a neighbour (`+` and a direction), which always spends a move's energy.
  Rust play is unchanged (full 45/45, smoke 3/3, smoke-exact 3/3
  identical); the deaths are recorded again as D0188–D0233.
- #52, evidence: R1, R3 and R4. check-boundary compares `depths.json`'s
  `stop_turn` with the summary. The death exporter takes a loaded save
  from borgh's `loaded`, so `run.sh ... -- -load FILE` counts, and run.sh
  records that save's hash and an extra `-keep-load-rng` in `config.txt`.
  zoo-check imports the zoo from a `git archive` of the pin, not the
  checkout, so an ignored `.pyc` cannot replace pinned code.
- sources (this PR): R5 and R6. The update snapshot backs up into a fresh
  directory of its own and never touches an existing `.refresh-backup-*`.
  The zoo prompt peels annotated tag `m1` to its commit (`m1^{}`). The
  copy on borg-zoo#30 needs the same fix, with N11's, when its link is
  recorded.

## 2026-11-10: M1 re-review findings N1–N11 (#39)

The re-review (Kimi K3.1 Pro, at `2dd11fd` and `16af1ad`) resolved 12 of
the 19 findings, partly resolved 7, and added N1–N11. N1 blocks. The fixes
go in PRs into `main`:
- measurement: N1, N3, N5 and N6. `suite.sh` writes each run's
  `run_exit`. `tools/endpoint.py` scores no run of a suite whose binary
  changed (`binary_changed`), and in a `FAILED` suite only a run whose
  run.sh succeeded. A `depths.json` that contradicts `summary.json` is an
  `evidence_conflict`. It refuses outputs with no DONE or FAILED marker,
  and never guesses the class of a run when its index played two classes.
  `harness/check-boundary.py` checks the suite hash and fails survivors
  that stopped before the cap.
- #48, N2: on a shop entrance `,` enters the store and takes no time, so
  `stand_still()` steps off instead of holding. Rust play is unchanged
  (full 45/45, smoke 3/3 identical), but the source moved, so the Rust deaths
  are recorded again as D0142–D0187.
- #49, death exporter: N4, N7 and N8. `Interrupting` and empty causes are not
  natural; `keep-load-rng` drops the seed only with `save=`; a random
  birth's terminal record names the played indices.
- sources (this PR): N9, N10 and N11. zoo-check refuses a modified zoo at
  its pin; `--update` rolls back a failed phase two; the zoo prompt needs
  tag `m1` or an exact commit from the owner. The prompt already on
  borg-zoo#30 still has the old fallback, so update it when the link is
  recorded.

**Source moved (owner, 2026-11-10):** neo-angband-tracker's
`c-borg-reference` branch is now pristine upstream (`e473720`), with no
`docs/strategy/`. The manuals live on `workbench/c-borg` (`842dc5d`):
`docs/user-manual.md` and `docs/strategy/`. The S12 pin `e67fd7d` is still
reachable from `workbench/c-borg` (and from `docs/strategy-manual`, which
the owner may delete). Do not run `--update` on `strategy-manual` before
the pin is moved: it follows `c-borg-reference`, which no longer has the
directory. In the post-review refresh PR, repoint the pin to branch
`workbench/c-borg`, or retire S12 in favour of S14 (owner's call, still
open).

## 2026-11-10: source S14, the complete Angband Strategy Manual

The owner added neo-angband-tracker `docs/c-manual/` (default branch) as a
source. It is copied to `reference/c-manual/` by a new copy rule on the
`neo-angband-tracker` pin, moved to `777eea1` with `--update` (the C Borg
spec and plan copies are unchanged there). It is the finished version of
S12's draft: 17 chapters, appendices A1–A10, and the official manual. Chapter
15, *Notes for the borg*, is direct input for M2–M6 strategy and parameters.
S12 stays pinned until the owner retires it.

## 2026-11-10: M1 review findings (#39)

The M1 review (Kimi K3.1 Pro, on #39) had 4 blocking, 13 should-fix and 2
nit findings. They are answered in four PRs into `main`, each with a reply
on its comments:
- #42: RNG isolation of perception (`with_borg_rng`, hallucination
  regression in CI), and no loop without game time.
- #43: endpoint scoring (exact boundary, process receipts, per-class
  attempts, victory, pairing, zoo boundary by kind) and suite manifests.
- #44: death records (natural-death checks, family identity, index repair,
  attempt dedupe, registration on publication refusal, zoo deaths
  repatriated, CI death reconciliation).
- the sources PR: an atomic `--update`, and the zoo request (death
  evidence, boundary for every adapter, no stale fallback ref).

Rust play is unchanged by the fixes (full suite 45/45 identical), but #42
changed the Rust build inputs, so the Rust deaths are re-recorded at the new
source. The zoo change request was filed as borg-zoo#30 at the owner's
request; the issue needs the corrections of this PR (owner's call).

**After the re-review and the owner's merge of #39:** tag `m1`; amend
RULES 7 (owner decision 2026-11-10: sessions may open issues, not push,
on the family's repositories) and record borg-zoo#30 in this file and in
ZOO-CHANGE-REQUEST.md; then start M2 with a refresh PR (better-borg-c
D0015–D0016, borg-rs impact v7 engine re-sync as its own PR).

## 2026-11-09: two sources added (owner request)

- **S12, the Angband Strategy Manual:** neo-angband-tracker `docs/strategy/`
  on branch `c-borg-reference` (not the default branch), copied in
  `reference/strategy/` at `e67fd7d`. A new pin `strategy-manual` with two new
  pin fields: `branch` (the branch the refresh report and `--update` follow)
  and `vendor` (its own clone, `vendor/neo-angband-tracker-c-borg-reference`,
  so it never fights the default-branch clone over a checkout).
  `scripts/refresh-sources.py` and `scripts/setup.sh` support both; a new test
  covers a branch pin. The manual is a work in progress (chapter 3 missing)
  and moved twice on 2026-11-09: expect the refresh report to show it.
- **S13, the official command descriptions**
  (angband.readthedocs.io `command.html`): built from upstream
  `docs/command.rst`. Upstream `master` is at our engine pin `e473720`, so the
  page is `angband/docs/command.rst` in our tree; nothing copied.
- Not done: no strategy is taken from either yet. That is M3+ work; lifted
  knowledge cites the engine source, not the manual (docs/SOURCES.md S12).

## 2026-11-09, end of session: M1 under review

**State**
- M0 is reviewed (tag `m0`). **M1 is complete and in review:** PR #39
  (`main` → `reviewed`). All M1 issues (#3–#11) are closed. Do **not** start
  M2 (#12–#16) before the owner merges #39 (RULES 6).
- CI on `main` is green: the `rust` and `engine` jobs, with all four arms,
  controls against references, Rust determinism and exact boundary, and the
  tool tests.

**Waiting on the owner**
1. The M1 review on PR #39. Its body has the reviewer prompt.
2. Tag `m1` on the merge of #39. The zoo specimens should point at it.
3. Routing the zoo change request. The owner asked for a prompt for a
   borg-zoo session; it is the appendix of
   [ZOO-CHANGE-REQUEST.md](ZOO-CHANGE-REQUEST.md). The zoo work happens in
   borg-zoo, not here.
4. Whether the self-produced `harness/reference/smoke-c-bbc-m7` is enough as a
   regression reference.

**Next session, in order**
1. `git pull`, `bash scripts/setup.sh`, and `python3 scripts/refresh-sources.py`.
   Read new deaths first; better-borg-c moves daily (RULES 21).
2. If #39 has review findings: one fix PR into `main` per finding set, a
   reply on each comment, and the re-review. If it is merged: tag `m1` (if the
   owner has not), then start M2 with an `--update` refresh PR (PLAN: *How
   every milestone starts*), then #12 (T2.1 Observation).
3. Check whether borg-zoo has the reborn specimens and exact-boundary cells.
   If it does, the next comparison is G1, which comes after M5 (GOALS), so
   nothing to run yet.

**Local state (not in Git)**
- `build/` (all four arms, the GUI build `build/play-rust`, the death-workflow
  caches), `harness/out/` (all suite outputs, including `t18/`), `vendor/`
  (pinned clones) and the scratch play data. All of it can be regenerated
  with the commands in harness/README.md and AGENTS.md.


## 2026-11-09: T1.8 every-arm check, zoo compatibility and change request (#11) — M1 complete

- **Every arm** (`c`, `c-pristine`, `c-bbc-m7`, `rust`) × smoke and full:
  - no harness failure;
  - the exact-boundary contract holds (`harness/check-boundary.py`,
    192/192 runs);
  - a rerun gives byte-identical keys (192/192);
  - the oracles still hold (`c` = `smoke-c`, `c-pristine` = `full-pristine`
    45/45).
- **Zoo:** `harness/zoo-check.py` runs the pinned zoo's own borg-rs adapter
  (in-memory specimens, no zoo change). All three arms run on the zoo's
  smoke with no harness failure or config error. The gaps are survivors
  stopping after the cap and an empty `first_turn_at_depth`.
  [ZOO-CHANGE-REQUEST.md](ZOO-CHANGE-REQUEST.md) is **for the owner to
  route**: three catalogue specimens; the adapter's `-exact-turns`,
  boundary field and depth stream; optionally the endpoint in stats.
- Controls now build from a fresh checkout: the pinned sources are cloned on
  demand and missing blobs fetched in one batch (`c-bbc-m7` in 16 s). CI
  builds both controls, checks them against `smoke-pristine` and the new
  self-produced `smoke-c-bbc-m7`, and checks the Rust arm's exact boundary.
- The Rust arm's first full-suite deaths are recorded.

**Next:** the **M1 review PR** (`main` → `reviewed`). Do not start M2 before
the owner merges it (RULES 6).

## 2026-11-09: T1.7 endpoint calculator (#10)

- `tools/endpoint.py` computes the GOALS primary endpoint per arm and class:
  alive at T with max depth ≥ D, or a victory before T, over **every
  attempted run**. Operational outcomes are listed separately, with Wilson
  95% CIs and paired exact McNemar tests on shared (seed, class).
- Inputs are local suite outputs (runs listed in the suite but missing count
  as harness failures) and borg-zoo cells (filtered by specimen and profile).
  Survivors not observed at the exact boundary are refused unless
  `--allow-after-key`; zoo borg-rs-adapter cells are after-key until T1.8's
  request is done. Seven unit tests.
- **Demonstrations (descriptive, not claims):**
  - our full-exact suite at D=5, T=100k: `c` 2/45, `c-bbc-m7` 2/45,
    `c-pristine` 1/45;
  - zoo `classes-milestones` (observer, 1800 per specimen): pristine 3.0%
    [2.3, 3.9], bbc-m4 4.7% [3.8, 5.7]; Warrior 15.5% against 20.0%,
    Druid 0.5% against 0.0%. That is post hoc on existing data.

**Next:** #11 (T1.8: every-arm check including `rust`, zoo compatibility,
change request). Then the M1 review PR.

## 2026-11-09: T1.6 launchers, recording, replay, death tooling (#9)

- borg-rs's launchers and `tools/` are copied from `e5cb263`, with the same
  commands. The adaptations are listed at the top of `tools/README.md`:
  `.angband-reborn/`, build-input lists naming our crate, two tests on `HEAD`
  instead of `m5`, and the aggregator accepting our basic dossiers.
- `run.sh` calls the depth stream and the basic exporter only when `tools/`
  is present. The recording launchers run a frozen copy without it and record
  deaths in borg-rs's full format, so there is no double record.
- The Rust Borg now honours **any key stops it** (`Term_inkey`, as the C
  Borg's `user_abort`). Harness keys are unchanged (3/3 identical).
- **Checked:**
  - a headless recording (`run-borg-headless.py`) replays **VERIFIED**;
  - a GUI session under Xvfb (`run-rust-borg.sh`, Ctrl-Z, z, Escape stops,
    Ctrl-X saves and quits) records completely and replays **VERIFIED**;
  - borg-rs's 12 GUI-recording tests pass with that fixture;
  - the tool suite passes 87 tests (26 skip for lack of fixtures) and now
    runs in CI's engine job.
- `docs/deaths/` now has borg-rs's aggregated `index.json` and `README.md`,
  regenerated after every export.
- Observed during the GUI test: the placeholder Borg dies quickly (an
  apprentice on level 2), as expected until M3.

**Next:** #10 (T1.7, endpoint calculator), then #11 (T1.8).

## 2026-11-09: T1.5 engine boundary and minimal Borg (#8)

- `borg-reborn-rs/src/headers/` holds borg-rs's c2rust engine bindings,
  copied unchanged with attribution. `engine.rs` is our own small boundary:
  the player view, `map_info` terrain, the `^Z` command key, options, the
  `inkey_hack` hook, `do_cmd_borg`, and `borgh_decision` behind the `borgh`
  feature. It is the only module with `unsafe`.
- `borg.rs` is the control loop of ARCHITECTURE.md. Queued keys go first;
  rest and repeat get `EVT_NONE`; every action starts with `ESCAPE`; the
  Borg's own RNG is used and the game's never. On top of it sits a
  **placeholder** policy: take a down stair underfoot, rest when below 70%
  HP (never twice without time passing), else walk to a random neighbour
  not known to be a wall or shop. Six unit tests cover the policy.
- `^Z z` starts it, so borgh's start script and the GUI work unchanged. It
  sets `auto_more` and `easy_alter`.
- **Checked:** the Rust smoke suite runs with no harness failure, and two
  runs give byte-identical keys (3/3). The Elf Mage of seed 102 dies to a
  rock lizard at turn 743, and its dossier is recorded. CI now runs Rust
  smoke twice and compares.
- Not here: launchers (T1.6), and the Rust arm's exact-boundary check (T1.8).

## 2026-11-09: T1.4 run contract: exact boundary and depth stream (#7)

- **No engine-tree edit was needed.** borg-rs's borgh already stops at engine
  turn T from its per-turn hook (`-exact-turns`) and logs depth on every key.
  `run.sh` gains the flag `exact`. `tools/depth-stream.py` writes
  `depths.json` after every run (boundary rule, alive at stop, maximum depth
  at stop, first turn at each depth), and dossiers include it. There are new
  suites `smoke-exact` and `full-exact`; the existing suites keep their
  semantics and references.
- **Checked on every C arm** (`c`, `c-pristine`, `c-bbc-m7`), smoke and full
  (144 runs) with `harness/check-boundary.py`. Keys before the boundary are
  byte-identical to the normal runs, survivors stop exactly at the cap, and
  deaths before the cap are the same deaths. The `rust` arm gets the same
  check in T1.8.
- The `c` arm's first full-suite deaths are recorded, from a clean build.

**Next:** #10 (T1.7, endpoint calculator; needs only T1.4) and #8 (T1.5,
the minimal Borg).

## 2026-11-09: T1.3 control C Borgs (#6)

- `BORG_IMPL=c-pristine` and `c-bbc-m7` build our engine tree with a pinned C
  Borg overlaid in `build/<impl>.src/` (`harness/control-overlay.py`,
  `controls` in `pins.json`). The overlay is checked blob for blob, and the
  pinned source is part of the build-input hash.
- **`c-pristine` is byte-identical to borg-rs's pristine-C references** (the
  zoo's `4.2.6-rs-m0-c`): smoke 3/3 and full 45/45, in keys, messages and
  state. Our engine's only gameplay correction (#474) was not reached.
- **`c-bbc-m7`:** tag `4.2.6-impact-v7` changes nothing in the engine outside
  `src/borg/` (audited). It ran smoke and full with no harness failure. No
  byte oracle exists under borgh.
- **Deaths recorded (RULES 12):** 15 `c-pristine` and 16 `c-bbc-m7` deaths on
  the full suite are in `docs/deaths/`, unassessed. This is one run each, so
  it is descriptive, not a comparison (RULES 14).
- **Note for T1.8 (#11):** without the `menu` flag, borg-rs's full suite plays
  other classes for index 8; the zoo passes `-menu-letters`.
- Controls are not in CI yet: they need `vendor/` (scripts/setup.sh). The
  local commands are in harness/README.md.

**Next:** #7 (T1.4, run contract: exact boundary and depth stream; borgh
already has `opt_exact_turns`).

## 2026-11-09: REDESIGN-LESSONS.md read into the design

- better-borg-c wrote `docs/deaths/REDESIGN-LESSONS.md` for this repository:
  thirteen lessons from D0001–D0014 and four owner manual takeovers. It came
  in with refresh #31, together with death D0014.
- [LESSONS-TRACE.md](LESSONS-TRACE.md) maps each lesson to our principle,
  our architecture section and the task that builds it.
- GOALS principles 4–8 are extended, and principle 14 is new (equipment by
  what it does). D3 now states that the threat model uses full race data
  (spoiler knowledge), as the C Borg does.
- ARCHITECTURE adds monster-track confidence, one assessment and tolerance
  for every consumer, cache validity, the escape ladder, consumable value
  and destination-aware level changes.
- PLAN adds D0014 to the fixtures, and owner manual branches as scenario
  tests.
- The refresh report now names new and changed registry documents.

**Next:** #6 (T1.3, control C Borgs).

## 2026-11-09: T1.2 harness, build identity, death export (#5)

- `harness/` is borg-rs's at `e5cb263`, with three adaptations listed in
  [harness/README.md](../harness/README.md):
  - `identity.sh` names our crates;
  - `run.sh` ends with `tools/export-death.py`;
  - `measurement/rust-overlay.py` copies both workspace members.
- `BORG_IMPL=rust` links through the seam package `borg-rs/` (the name the
  imported CMake builds), which re-exports `borg-reborn-rs`. The engine
  boundary module `engine` has a no-op `do_cmd_borg` until T1.5.
- **Import proved:** the `c` smoke suite is byte-identical (keys, messages,
  state) to borg-rs's own C build at `e5cb263`. That output is
  `harness/reference/smoke-c`. borg-rs's committed smoke reference is the
  pristine-C one (unchanged since `aa4ad6c`) and is kept as
  `harness/reference/smoke-pristine` for T1.3.
- **First death:** smoke-mage-2 (seed 102, Elf Mage, `c` arm) dies to an
  acolyte at turn 7195. It is the same as in borg-rs. Its dossier is our first
  case in `docs/deaths/`.
- CI gains an `engine` job: build `c`, run smoke, diff against `smoke-c`,
  build `rust`, check that the binary reads as `rust`.

**Next:** #6 (T1.3, control C Borgs) and #7 (T1.4, which needs T1.3).

## 2026-11-09: T1.1 engine tree imported (#4)

- `angband/` is borg-rs `e5cb263` (impact v6) `angband/`, split with
  `git subtree --ignore-joins` into 29 commits and added unsquashed. The tree id
  `5cc1b1c…` is recorded in `pins.json` (`engine_tree`), and a repository test
  pins it.
- [ENGINE-TREE.md](ENGINE-TREE.md) lists what differs from upstream
  `e473720`:
  - 8 added files (borgh, recording);
  - 14 engine files, all behind `BORG_HARNESS`/`BORG_RECORDING` except
    CMake, `main.h` declarations and the #474 cure-all correction;
  - 26 C Borg files (borg-rs's adopted milestones 2–6).
- It also gives the re-sync procedure.
- borg-rs's `FORK-CORRECTIONS.md` and `adoption/status.json` are copied to
  `reference/borg-rs/` by a new copy rule. The refresh report flags when
  borg-rs's `angband/` tree moves.
- Nothing is built yet: building and the harness are T1.2 (#5).

**Next:** #5 (T1.2: harness, build identity, death export before the first
game).

## 2026-11-09: new deaths read into the evidence

better-borg-c D0011–D0013 (from the first refresh) are folded in. SOURCES
has three new death patterns (fear by list, unknown items left unknown,
wrong kit valuation) and extended rows. GOALS principles 4–8 cite them, and
PLAN's death fixtures include them. The research digest
(`docs/research/DEATHS-AND-ISSUES.md`) is a dated snapshot of 2026-11-08;
`reference/deaths/INDEX.md` is the live list.

**Next:** #4 (T1.1, import the engine tree from borg-rs at `e5cb263`).

## 2026-11-09: T1.0 source refresh routine (#3)

- `scripts/refresh-sources.py` reports, per pin: commits since the pin, new
  and changed death cases, new tags (new control arms), changed C Borg
  functions we cite, and issues updated since the pin was read. `--update`
  moves the pins, refreshes `reference/` (copy rules in `pins.json`) and
  regenerates `reference/deaths/INDEX.md`. Tests on synthetic repositories
  cover the report, update, dirty vendor and offline cases; CI runs them.
- RULES 21–23: every session starts with the report, every milestone with an
  `--update` PR; baselines are exact; lifted knowledge is watched.
- **First `--update` (PR closing #3):** borg-rs `6b58a46`→`e5cb263` (impact v6; its C Borg now differs from upstream in 26 files, issue #4 updated), better-borg-c `63fbc1f`→`8b26042` (deaths D0011–D0013 copied), borg-zoo `ffe5a05`→`9cd7423`, borg-agent `697abde`→`8f1dd7f`. The new deaths are read into SOURCES/GOALS/PLAN in a separate PR (RULES 21).
- **First real report (2026-11-09):** better-borg-c has three new deaths
  (D0011–D0013), 25 commits and 75 updated issues; borg-rs moved 7 commits,
  borg-zoo 3, borg-agent 9. The `--update` PR that brings them in is the
  last step of #3.

## 2026-11-09: M0 reviewed, M1 started, backlog moved to issues

- The owner merged the M0 review (#1); tag `m0` is on the merge
  (`ae64f12`). D1–D11 are accepted (GOALS.md).
- The backlog is now GitHub issues (owner request): GitHub milestones
  M1–M7+, labels `task`, `process`, `in progress` and `owner`, and issues
  #3–#24 for T1.0–T3.8, each with a **Needs** line linking its
  dependencies. PLAN.md keeps only milestones, exits and method.
- **Next:** #3 (T1.0, the source-refresh routine over every pin, death
  reports included; owner request), then #4 (T1.1, import the engine tree
  at refreshed pins).

## 2026-11-08: M0 review findings addressed (T0.4)

The owner's review of PR #1 had six findings, all verified and addressed
in the T0.4 PR:

| Finding | Change |
| --- | --- |
| Blocking: the primary endpoint has no tooling that computes it. borgh overshoots the cap, the adapter leaves `first_turn_at_depth` empty, and zoo stats exclude operational outcomes | GOALS *Observation and scoring rule*; PLAN T1.4 (exact boundary, depth stream), T1.7 (`tools/endpoint.py`), T1.8 (zoo change request); G1 prerequisites stated |
| Blocking: the imported `BORG_IMPL=c` is borg-rs's adopted C Borg (22 files differ from upstream), and observer-harness zoo cells cannot be paired with borgh | PLAN T1.1 corrected; new T1.3: `c-pristine` and `c-bbc-m7` controls built here on the same engine and `borgh`, `c-pristine` checked against the zoo's `4.2.6-rs-m0-c`; GOALS D2/D8/G1/G2 |
| setup `--strict` did not check that checkouts were at the pins | `scripts/setup.sh` checks out pins, reports WRONG REVISION, never moves dirty checkouts, `--checkout` moves clean ones (all three paths tested) |
| Death copies missed `overall-assessment.md` and other small evidence files | All root-level `.md`/`.json` of each case copied; external links listed in `reference/README.md`; a test fails on dangling internal links (checked that it fails when an assessment is removed) |
| Death export came after the first playing run | Minimal death export is part of T1.2, before its first smoke run; RULES 12 applies from the first game. Follow-up review: T1.4 checks only the C arms, and the every-arm check (`rust` included) is in T1.8 and the M1 exit |
| Nit: `BORG_IMPL=reborn` | `rust` everywhere, as borg-rs |

Follow-up review on #2: borg-rs is reclassified everywhere as engine and
tooling source and perception oracle, not a control. Its zoo results are
context (GOALS recommendation, non-goals and D8; AGENTS).

Still open for the owner: D1–D11, and routing the zoo change request once
T1.8 writes it.

## 2026-11-08: milestone 0 (foundation), ready for review

**Done**
- T0.1: sources read and evaluated ([SOURCES.md](SOURCES.md)). Specification,
  plan, curated death dossiers and zoo catalogue copied to `reference/`
  with provenance. Prior art outside Angband surveyed (AutoAscend, BotHack,
  TAEB/Saiph, qw, Rog-O-Matic, NetPlay). A digest of deaths and issues
  by subsystem is in [research/DEATHS-AND-ISSUES.md](research/DEATHS-AND-ISSUES.md).
- T0.2: goals, principles, staged targets and eleven owner decisions,
  all *proposed* ([GOALS.md](GOALS.md)). The design direction is in
  [ARCHITECTURE.md](ARCHITECTURE.md).
- T0.3: [PLAN.md](../PLAN.md) (M0–M7, tasks for M1–M3),
  [RULES.md](../RULES.md), [AGENTS.md](../AGENTS.md) (model-agnostic entry;
  `CLAUDE.md` only includes it), `pins.json`, `scripts/setup.sh`, the
  empty crate `borg-reborn-rs/` (`staticlib` + `rlib`, as borg-rs),
  repository consistency tests, CI.

**For the owner to decide (milestone 0 review)**
- D1–D11 in [GOALS.md](GOALS.md#owner-decisions). The ones that shape M1
  most: D2 (engine tree from borg-rs), D3 (information level), D4 (reuse
  of borg-rs code), D9 (workflow).
- Whether the `reviewed` branch starts at the root commit (so the M0 review
  PR shows everything), as set up here.

**Next**
- Done on 2026-11-09: see the newest entry above.

**Known limits**
- Nothing is built against the engine yet. The CI checks only the empty
  crate and the documents.
- The death digest was produced with a model's help from the issue bodies
  and dossiers at the pinned commits. Its classifications are leads, not
  findings. SOURCES.md's table of death patterns was checked by hand
  against the dossiers' `assessment.json`.

## 2026-11-14: optional standalone recorder (#210)

The owner separately authorized this integration during angband-recorder setup. The v0.1.0 crate is linked through the existing Rust library, hooks apply only to the generated engine copy, and the default harness remains recorder-off. Exact pin and design: docs/ANGBAND-RECORDER.md. The recorder-enabled short native capture/replay passed; no strategy work or pristine engine bytes were changed. New raw runs stay outside Git. In-game branching remains future recorder work.

Recorder integration CI passed the gameplay/perception comparisons but required
new build-identity death dossiers. D2093/D2094 reproduce CI controls 102/1020;
keys, messages and state channels match the CI artifact, whose original record
and build identities are retained in ci-evidence.json. Both remain unassessed
validation controls, not a strategy/performance claim. No death check is waived.

## Synthetic P1 review follow-ups (2026-11-16)

Owner-supplied reviews of #378 d406d2c9 and Cemetery #7 e750549 are input, not
acceptance. Every requested capture now reports captured/absent and its requested
level; an unreached/skipped level no longer leaves an ambiguous successful run.
A parity regression covers the absence path. The startup wrapper mirrors the
native character_dungeon leave guard and names lifecycle/arena pin-bump checks.
Cemetery fixes malformed-option reconciliation, normalizes marker flags and
prints exclusions; this source pins its reviewed-follow-up guard revision.
Keep the one-way policy limitation explicit; #386's strategy/tactics are separate.
OpenSSL remains an intentional documented runtime dependency. Full responses
and fresh gameplay evidence belong in Cemetery docs/reviews/M2, not here.

## 2026-11-16: #440 review follow-up (#447)

- Separate cap overlay fixes reload, reinitialization, stale restoration and
  Recall cancellation; the cap test permits the accepted one-step N+1 move.
- Release core checks: 329 pass, 4 ignored; workspace tests, fmt and clippy
  pass. Targeted native plan/survival checks: 13 pass; tools: 94 pass;
  scripts: 19 pass. Full engine suite: 84 pass/checks, 2 fixture-dependent
  skips. Vendor declaration also covers #444’s existing hook.
- Clean implementation/source/binary are pinned in Cemetery #36, with three
  canonical CI control deaths. The source fixture pins that evidence commit.
  Lifecycle reload/cancel behavior has unit coverage; an interactive GUI
  reload/Recall session is not claimed. Follow-up stays unmerged for review.

## #449 re-review fixes

- Apply the effective depth limit before dynamic/static preparedness selection.
- Initialization refreshes the cap from the loaded character and level before
  store/Recall ticks; same-run cancellation remains cancelled.
- Require the child cancellation test to report one executed test. Remove
  the redundant cancellation branch and correct the remaining path-intent text.
- Add initialization lifecycle and production preparedness-guard regression before calculation dispatch.
  Validation and refreshed evidence pins will be recorded on #449.

## #449 second re-review

- Guard preparedness with only the active plan limit. Restore the configured
  check at its original static-only location so no-plan play and reason order
  remain unchanged. The dispatch regression covers absent-plan pass-through.
- Completed reincarnation calls the same refresh as initialization; declare
  the receiving hook and clarify configuration semantics in the documentation.
- The PR runtime pin is already current; refresh it and validation after these
  runtime changes. #448 remains open for historical raw-evidence custody.

## 2026-11-17: Track P synchronization before inventory calibration (#474)

- Owner selected #433. Synchronize accepted main 68e39b7e into track-p
  through this CI-checked integration PR; prior track-p is an ancestor.
- Preserve all accepted runtime, recorder, death identity/matching, evidence
  references and plan-cap fixes. No new gameplay policy is introduced.
- Next: M13 inventory parameter audit/extraction under #433, with neutral
  defaults, parity checks and a pinned owner-selected promotion review.

## 2026-11-17: P4 M13 inventory parameter extraction (#433)

- Owner selected #433; #475 synchronized accepted main into track-p at
  9cf60aae after green CI. P4 is the bounded active milestone in PLAN.md.
- Append 362 neutral parameters (2,143 total): power48, wear23, buy14,
  sell34, shop6, home237. Existing IDs/defaults, C-long/i32 arithmetic,
  tie/RNG order, quantities, no-selling and disabled exploit paths stay.
- M13-INVENTORY-PARAMETERS.md/.json classifies policy versus retained
  mechanics and gives original source references. #293 stays separate;
  R4 #434 waits for owner-reviewed promotion of an exact P4 pin.
- Divisors/narrow counts are bounded; invalid greed/tier/depth ordering
  fails before play. Add a native non-neutral town-budget regression.
- Validation/evidence receipts are recorded on the task PR before merge.
  Task integration targets track-p; the subsequent milestone review and
  owner acceptance gate promotion to main. No tuning/population claim.

## 2026-11-17: P4 milestone review fixes (#481)

- Owner-selected review #477 at 105f7947 confirms neutral extraction and
  independently matches all 45 native streams, with an exporter checkout
  mismatch limiting that reviewer's harness/reconciliation status.
- F1: cap mapping-rod reserve at 255 and test 256/300 rejection plus the
  valid endpoints and all new u8/i16 operand bounds. Default remains 2.
- F2: rename the three unpromoted curing bonuses for their actual CCW,
  Healing and poison-cure guards; synchronize descriptions and JSON.
- Preserve old parameter positions/defaults and immutable Cemetery #61
  evidence; no aliases or policy changes. Integrate only through track-p
  and update the exact #477 pin for owner re-review before main promotion.


## 2026-11-17: P4 conflict resolution before promotion (#522)

- Merge accepted main a8ac685a into task/T522-p4-main-sync from track-p
  7078bb1d, preserving all main bug fixes, configuration and review rules.
- Resolve params.rs test conflict by retaining both town and P4 source-module
  validation; retain both calibration snapshots and correct the current count.
- Main's 1829 specs stay byte-identical in the same order; append the same
  362 reviewed P4 specs (2191 total). Unpromoted P4 absolute IDs move;
  historical evidence remains pinned to its original registry and build.
- The buying-code auto-merge keeps #507's quantity fix and neutral P4 lookups.
  All other runtime modules equal their accepted parent versions exactly.
- Required checks/CI and owner review precede task integration under RULES 1a.
  #477 stays unmerged; update its pin after approved track-p integration and
  re-review the synchronized result before promotion.
