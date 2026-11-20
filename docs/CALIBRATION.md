# Calibrating borg-rs's core (track P)

P1 exposes603 caution/danger/escape/recovery parameters. P2 adds167
attack,218 defence,19 item-use and727 power values: **1734 total**.
Defaults retain the vendored v12 policy. Input bounds support experiments;
no value is recommended or claimed to improve play. Each section below
identifies its pre-extraction source and retained mechanics/interface data.


Each section below is a snapshot: its "registry now holds N" is the count
right after that change, so older sections show older counts.

## Flight tuning after the Ranger deaths (#747, #749, #750, #751, #753)

Owner (2026-11-20), after the analysis of borg-zoo#488's 12 Ranger deaths
(main fde8025a): "we must fix the flight. My idea is to stop flight when
other monsters are IN BETWEEN the stairs, not close."

The registry now holds **2240** parameters (2234 on main, which carries #755
and #754; six added: five here and #743's `caution_stairs_clean_path_flee`,
section below; `caution_stairs_clean_path` gains modes 2 and 3). Six
switches, all compiled and shipped 0 = today (`caution_stairs_clean_path`
stays shipped 1):
- **`caution_stairs_clean_path` 2** (#751): the path test (`path_blocker`,
  `borg_flow_stairs.rs`) counts only a monster **on** the path, the staircase
  included. Mode 1 (#736) also counted one next to it, which made the
  monster next to the Borg "in the way" of an adjacent staircase
  (2095979189, wild cat). The mode is read inside `path_blocker`, so it
  applies to the desperate walk and to #743's check of every flight.
- **`caution_stairs_clean_path` 3** (#751, owner on 93034700: "he has to
  actually walk past his opponent"): on the path, or next to it (one of the
  8 grids around a grid of the path) and ahead of the Borg: nearer the
  staircase at the path's end, in grid steps (king moves), than the Borg.
  A monster standing on the walk always blocks (the review of #757 found a
  winding path where it did not; fixed, with a test). Mode 2 let a
  Necromancer at 11/11 HP walk past an adjacent apprentice to stairs 8
  steps beyond it, and it died on the way with 9 Cure Light Wounds, 6 Phase
  Door and 27 Flasks of oil. At mode 1 the same start reads Phase Door and
  kills the apprentice with two flasks. A monster next to the Borg but
  behind it, away from the stairs, does not block.
- **`caution_desperate_ranged_first_reach`** (#751, owner on 247597266: "he
  starts moving towards the cutpurse even if it has much better options,
  such as flask of oil, magic missile"): above 0, while the Borg has a ranged
  attack (`ranged_means`: a bow with ammunition or something to throw;
  spells are not counted) and no awake monster is adjacent, the desperate
  walk does not start toward an awake dangerous monster within this many
  grids of its path and nearer the stairs than the Borg (mode 3's test with
  a wider reach); "Stairs not reachable: X ahead, fighting at range", and
  the attack at range decides. The Mage had cast one Magic Missile, failed,
  and the lower danger estimate after it let the walk win.
- **`caution_desperate_fast_danger_percent`** (#747): "Desperate for Stairs
  (three)" doesn't walk while a faster monster is adjacent. Above 0, only
  when that monster's danger is at least this percent of the HP: a fruit bat
  (432972050) no longer cancels a walk to stairs 6 steps away.
- **`caution_recall_cure_min_loss_percent`** (#749): C's buy-time cure while
  waiting for Recall (`borg-caution.c:2046`: less than 100 HP lost) is true
  at full HP for a low-level character, which drank 4 Cure Light Wounds at
  15/15 HP (432972050). Above 0, a cure needs at least this percent of the
  max HP lost.
- **`caution_rising_hold_min_hp_percent`** (#753): with #557's
  `caution_rising_hold_flee_descent`, walking home (rising) holds a flight
  off a down staircase unless the danger there is at least the HP. In
  214960633 the Borg stood on a down staircase at 2 of 47 HP, the danger on
  the staircase 0 (a grey mold next to it), and was held. Above 0, a
  fleeing Borg below this percent of its max HP is not held.
- **`caution_stairs_flight_ignore_danger`** (#750): a flight's stair flow
  (`GOAL_FLEE`, or any `GOAL_BORE` stair flow while fleeing or leaving: the
  bored walk up, the bored walks down and the powerdive) refuses no grid for
  its danger, as the desperate walk. (Review of #757: the gate checks the
  goal and the flight, not the direction; the measured runs had this
  scope.) In 807683971 a black naga two grids
  beside a clear corridor made the corridor's first grid "too dangerous" and
  the flight never reached the up staircase 8 grids away (a logging replay).

**On the 12 Ranger deaths** (fde8025a with the four switches; off replays the
zoo's keys 12/12): each alone saves two games (#747: 1222599014, 432972050;
#749: 1583605217, 432972050; #750: 225452245, 807683971; #751 changes only
2095979189, which still dies); all four save 4. Six games are untouched by
any of them (see #753, #754).

**On 600 local starts** (fde8025a plus the flight branch with #743 merged;
"on" = `caution_stairs_clean_path` 2, `caution_desperate_fast_danger_percent`
50, `caution_recall_cure_min_loss_percent` 30,
`caution_stairs_flight_ignore_danger` 1, `caution_stairs_clean_path_flee` 1;
#753's switch was not in these runs):

| starts | off deaths | on, mode 2 | on, mode 3 | mode 3 saved | mode 3 newly killed |
|---|---|---|---|---|---|
| 200 Rangers | 9 | 10 | 12 | 1 | 4 |
| 400 others (50 per class) | 3 | 8 | 4 | 1 | 2 |

With mode 2, five deaths of the "on" arm walk to the stairs at 0–7 HP with
the attacker next to them (93034700, 122163533, 2095979189, 247597266,
4971587). Mode 3 removes four of them; 2095979189's wild cat is level with
the Borg and still kills it. Mode 3's six new deaths split from "off" at the
other switches (#749's Recall cure in 249140769, #750's flow) or after a
refused walk, and die 1,500 to 38,000 game turns later of other things
(142490685, 902565616). 12 against 16 deaths is too few for a sign test. The
mode 3 numbers are from before the review fixes to #743's fallback. Nothing here is a claim: the zoo's measurement is
borg-zoo#501, and every switch stays at 0 until it reports.

| parameter | default | unit | range | source |
|---|---|---|---|---|
| `caution_stairs_clean_path` | 0 (shipped 1) | choice | 0–3 | `borg_caution.rs:967` |
| `caution_desperate_fast_danger_percent` | 0 | percent of HP | 0–10000 | `borg_caution.rs:4213` |
| `caution_desperate_ranged_first_reach` | 0 | grids | 0–10 | `borg_caution.rs:981` |
| `caution_recall_cure_min_loss_percent` | 0 | percent of max HP | 0–100 | `borg_caution.rs:4485` |
| `caution_stairs_flight_ignore_danger` | 0 | flag | 0–1 | `borg_caution.rs:4192` |
| `caution_rising_hold_min_hp_percent` | 0 | percent of max HP | 0–100 | `borg_caution.rs:3661` |

## A clean path for every flight to the stairs (#743)

The registry now holds **2223** parameters (2222 before; one added). Owner,
2026-11-20, closing #689: the follow-up of #736's clean path, for the
ordinary flight to the stairs.

- **`caution_stairs_clean_path_flee`** (compiled 0, shipped 0): every flight
  to the stairs (`GOAL_FLEE` through `borg_flow_stair_both`, `_less` and
  `_more`: a scary level, too much danger, "Looking for any stair") takes
  a staircase only by a clean path. The flight's own flow is walked before it
  is committed (`commit_and_step`, #736's `path_blocker`; no second spread,
  so a clean path changes nothing). An awake monster on, or next to, the
  path, whose danger is at least `caution_stairs_block_danger_percent` of the
  HP, refuses it; a flight to any stairs then tries the up and the down
  stairs on their own. With none clean the flight does not start, and the
  escapes and the attack decide. Noted once per level as #736's "# Stairs not reachable".
  The desperate walk keeps its own check (`caution_stairs_clean_path`).

| parameter | default | unit | range | source |
|---|---|---|---|---|
| `caution_stairs_clean_path_flee` | 0 | flag | 0–1 | `borg_caution.rs:949` |

## The C Borg's leash as parameters (#732)

The registry now holds **2232** parameters (2222 before; ten added). Owner,
2026-11-20: "remove our leash completely and expose the leash of C with
configuration parameters". The owner's own leash (PR #734: 15/25/40/60 steps
by level) is dropped; the zoo found it neutral (borg-zoo#475).

- C's leash keeps the Borg within `CL × per_level + base` steps of the stairs
  (`borg_get_leash`): for picking up items below `pickup_max_level`; for
  exploring while the depth is at least the level less `depth_margin` (so a
  Borg well above its depth explores freely); `unbounded` otherwise. Once
  bored (more than `twitch_after` twitches) it grows by the twitch count.
- The same numbers drive the distance-to-stairs check
  (`borg_flow_far_from_stairs_dist`, below `stairs_max_level`) and the two
  item checks of `take_select` (below `take_near_max_level` near the stairs;
  below `take_max_level` past `take_nearness`).
- Every default is C's literal, so play is unchanged; a zoo profile can move
  any of them. Read in one place, `c_leash` in `borg_flow.rs`, on each use
  (not cached).

| parameter | default | unit | range | source |
|---|---|---|---|---|
| `flow_leash_per_level` | 3 | steps per level | 0–100 | `borg_flow.rs:1662` |
| `flow_leash_base` | 9 | steps | 0–250 | `borg_flow.rs:1663` |
| `flow_leash_unbounded` | 250 | steps | 0–250 | `borg_flow.rs:1659` |
| `flow_leash_pickup_max_level` | 20 | level | 0–51 | `borg_flow.rs:1660` |
| `flow_leash_depth_margin` | 5 | levels | 0–127 | `borg_flow.rs:1661` |
| `flow_leash_twitch_after` | 21 | twitches | 0–1000 | `borg_flow.rs:1662` |
| `flow_leash_stairs_max_level` | 20 | level | 0–51 | `borg_flow.rs:1663` |
| `flow_leash_take_near_max_level` | 10 | level | 0–51 | `borg_flow.rs:1664` |
| `flow_leash_take_nearness` | 5 | steps | 0–250 | `borg_flow.rs:1665` |
| `flow_leash_take_max_level` | 20 | level | 0–51 | `borg_flow.rs:1666` |

## Immobile monsters and blocked stairs (#736)

The registry now holds **2221** parameters (2216 after #716; five added).
Owner, 2026-11-20, from the key-by-key replays of #717's Druid deaths: "the
path to the stairs is not clean, so the 'desperate for stairs' should see
that the stairs are not reachable"; "it stepped away in the wrong direction
... towards the exit"; "choosing the enemy you can take down is the wrong
idea ... target the mobile one"; "I would put them already on".

- **`caution_stairs_clean_path`** (flag, compiled 0 = C, **shipped 1**): the
  desperate walks to the stairs in `borg_caution` ("Desperate for Stairs"
  one to four) first walk the desperate flow's path from the Borg
  (`borg_flow_stairs::stair_path_blocker`). An awake monster on a grid of
  the path, or next to one, whose danger to the Borg is at least
  **`caution_stairs_block_danger_percent`** (50) of the HP makes the stairs
  not reachable (`# Stairs not reachable: <monster> in the way`). The
  escapes and the attack decide instead.
  - The probe puts the flow state back as it was, so it only observes.
- **`caution_immobile_step_to_exit`** (flag, compiled 0, **shipped 1**): the
  step away from an immobile attacker (`immobile_step_away`, only with
  `caution_ranged_vs_immobile` 1–3, shipped 0) takes, among the grids it may
  step to, the one with the fewest steps to a staircase, up or down
  (`stair_steps_around`), then the least danger. C's step takes the least
  danger, so ties go to the first direction tried.
- **`attack_mobile_threat_first`** (flag, compiled 0, **shipped 1**): while a
  mobile monster among `borg_attack`'s targets is a threat (danger at least
  **`attack_mobile_threat_percent`**, 50, of the HP), immobile monsters are
  not targets (`# Targeting the mobile threat, not the immobile monsters`).
- **Evidence:** the three Druid deaths of borg-zoo#461 that only modes 2 and
  3 have, replayed on #717's head with mode 2, all live with the three on.
  On classes-18 (main with mode 0, A and C on against off), deaths 0 and 0,
  sum of max CL 104 → 106, sum of max DL 37 → 41, 11 of 18 identical. A zoo
  pilot follows.

| parameter | default | unit | range | source |
|---|---|---|---|---|
| `caution_stairs_clean_path` | 0 (shipped 1) | flag | 0–1 | `borg_caution.rs:967` |
| `caution_stairs_block_danger_percent` | 50 | percent | 0–10000 | `borg_caution.rs:968` |
| `caution_immobile_step_to_exit` | 0 (shipped 1) | flag | 0–1 | `borg_caution.rs:1729` |
| `attack_mobile_threat_first` | 0 (shipped 1) | flag | 0–1 | `borg_fight_attack.rs:4881` |
| `attack_mobile_threat_percent` | 50 | percent | 0–10000 | `borg_fight_attack.rs:4882` |

## Ranger arrows: a soft cap, then a diminishing curve (#716)

The registry now holds **2216** parameters (2214 after #712; two added).
Owner, 2026-11-20: "they do not need that many arrows"; "a soft cap at 50
and then a curve for additional arrows utility"; "a curve coefficient like
we defined for healing ... can take infinite arrows, but the more he carries
the less value they have".

- **`town_ranger_arrows_max`** (0 = C): the town planner's shot reserve
  ("Ranger usable shots") becomes a requirement. Each shot up to this many
  is worth the same, the reserve's weight divided by the cap. C's curve
  grows with the trip and never stops paying, and arrows cost 1 gold, so
  Rangers bought about 300 a game and carried 150–200. Their weight sent
  Phase Door to the Home.
- **`town_ranger_arrows_curve_scale_percent`** (0): shots past the cap count
  on a capability of their own, "extra shots". It is C's shot curve (the
  reserve's weight, the trip's demand) with weight and demand scaled by this
  percent, as `town_heal_curve_scale_percent` does healing. Every shot is
  worth something, each less than the last. 0 means nothing past the cap.
- **Shipped** (owner): the Ranger, 50 and 50; every other class 0 (only the
  Ranger has a shot reserve, `town_missiles_percent` -1). A zoo experiment
  calibrates the scale.
- **Not changed:** pack power still keeps up to 180 shots that the Ranger
  picks up in the dungeon (`power_inventory_amissiles_limit_3`).

| parameter | default | unit | range | source |
|---|---|---|---|---|
| `town_ranger_arrows_max` | 0 | count | 0–1000 | `borg_town_capability.rs:684` |
| `town_ranger_arrows_curve_scale_percent` | 0 | percent | 0–10000 | `borg_town_capability.rs:722` |

## Thrown oil, a town-planner curve of its own (#712)

The registry now holds **2214** parameters (2212 on main before it; two added).

- **`town_oil_apart`** (flag, 0 = C): the town planner values each flask of
  oil kept to throw on a capability of its own, "thrown oil", beside its
  damage on the shared `ATTACK` curve (as an arrow counts in "Ranger usable
  shots"). With C's planner only, a stack of sling shots saturates `ATTACK`,
  so a flask's marginal damage is worth almost nothing, and the weight
  repair stores the flasks at the Home, one at a time. The curve is weighted
  as `ATTACK` and by the class's `town_oil_curve_scale_percent` (#682).
- **`town_oil_flasks`** (10): the thrown-oil curve is at half its value with
  this many flasks kept to throw.
- **Shipped** (owner, 2026-11-20): on for every class except the Ranger,
  which has arrows (`town_oil_apart__ranger = 0`).

| parameter | default | unit | range | source |
|---|---|---|---|---|
| `town_oil_apart` | 0 | flag | 0–1 | `borg_town_capability.rs:392` |
| `town_oil_flasks` | 10 | count | 1–255 | `borg_town_capability.rs:698` |



## Step-away from immobile creatures turned off (#736)

The registry still holds **2211** parameters. Owner, 2026-11-20: "stepaway
flag is not a solution, I do not like it"; "why do we build 'step away',
isn't it better 'do not get closer'?".

- **`caution_ranged_vs_immobile`** ships **0** (C). In borg-zoo#461's big
  tier, modes 2 and 3 lost games (22 deaths off, 24 and 27; Druids 5 → 9 and
  10). Mode 3 crashed (#726).
  - Replays on PR #717's head: the step-away loops against a flight whose
    path passes the monster, and it steps instead of reading Phase Door while
    a mobile attacker hits the Borg.
- **Until #736:** the owner's reading of the replays: the step-away is not
  the fault, its direction is (toward the exit, not the first safe grid), and
  the stairs, the target and Phase Door around it. #736 fixes those, then
  the step-away can come back.

## Upstream C Borg fixes carried (#724)

The registry now holds **2211** parameters (2210 before; one added). Owner,
2026-11-20: "carry fixes from upstream Borg in Angband in borg-rs".

| upstream | fix | in borg-core |
|---|---|---|
| `b5747369` | the healing-chance adjustments go the way their comments say (`borg_heal`) | carried before, borg-rs patch 0092 (`borg_caution.rs`) |
| `007bea95` | keep the **largest** slay multiplier across equipment slots | carried now as upstream's maximum (`borg_trait.rs`); borg-rs used `\|=`, the same wherever a slay is read as on or off, which every read is |
| `95387b06` | check the teleport landing with **either** kind of teleport | `escape_teleport_landing_either`, below |

- **`escape_teleport_landing_either`** (compiled 0, shipped 0): 1 =
  `borg_caution_teleport` runs with a teleport scroll or spell
  (`BI_ATELEPORT`) **or** a teleport staff or activation (`BI_AESCAPE`), as
  upstream; 0 = C and borg-rs (both needed).

| parameter | default | unit | range | source |
|---|---|---|---|---|
| `escape_teleport_landing_either` | 0 | flag | 0–1 | `#724 (upstream Angband 95387b06): 1 = borg_caution_teleport checks the teleport landing with a teleport scroll or spell (BI_ATELEPORT) or a teleport staff or activation (BI_AESCAPE); 0 = C and borg-rs (both needed, so with only one the check never ran and the Destruction and Teleport Level defences never saw a safe landing).` |



## Immobile creatures kept at range (#713)

The registry still holds **2210** parameters (after #697): #646's
`caution_ranged_vs_immobile` grows from a flag to a mode, and ships at 2.
Owner, 2026-11-20, on #713 (a Paladin meleed a yellow jelly for 120 game
turns and died of its poison): "step away from immobile creature with
annoying effects and only attack them from a distance, if you have to";
"consolidate them"; "they can be in the way, so kill them with ranged
attack"; "do not block the path through their reach if they need to go to
their stairs"; "set the switch to on".

- **`caution_ranged_vs_immobile`** (mode 0–3, compiled 0 = C, shipped 2, then **shipped 0** by #736's
  owner decision below):
  - **1** (#646): next to an immobile attacker (`NEVER_MOVE` with a blow),
    step away when confused (up to 3 steps) or carrying a ranged attack (1),
    and hold the confusion cure next to it.
  - **2** (#713): 1, and keep at range one whose blows do more than plain
    damage (any effect but `HURT`: poison, confusion, paralysis, fear,
    blindness, hallucination, stat or experience drain, theft, elemental
    damage). That is 23 of the 25 in `monster.txt`: 15 at DL10 or shallower and 8
    deeper, from the grape jelly (DL12) to Wiruin (DL80). Only the clear
    mushroom patch (DL10) and the old forest tree (DL11) are left out.
  - **3:** as 2 for every immobile attacker.
- **What "kept at range" does** (in the dungeon):
  - **No melee:** `borg_thrust_damage_one` scores 0.
  - **No hunt walk:** `borg_flow_kill` skips it.
  - **Flows avoid its reach:** `spread_blocked` blocks its grid and the 8 next
    to it (`in_reach_of_kept`), never the Borg's own grid. `borg_flow_old` never
    walks into it. A flow to stairs (`borg_flow_spread_stairs`) may pass
    through the reach, never onto its grid.
  - **Step away:** next to one, the Borg steps out of reach
    (`immobile_step_away`), while fleeing too, up to 3 times per creature and
    level.
  - Throws and shots are unchanged. From two grids away a throw has no
    point-blank divisor, so one in the way is killed from afar.
- **The case** (766958386 Human Paladin, the dying build `e5cc5469` with the
  change applied): at 0 it dies of poison at game turn 5,100 (the dossier). At
  2 (the earlier two-switch version, same behaviour) it steps away, throws 45
  flasks and lives to the 100,000-turn cap at CL6.
- **Found while measuring, and fixed:**
  - **Fleeing:** #646's step did not run while fleeing, and the case was
    fleeing; its exploring flow walked into the jelly 12 times.
  - **The reach:** an item under a white jelly drew the Borg back beside it
    about 60 times.
  - **The Borg's own grid:** a reach block on it would have left no flow to
    start from.
- **The merged switch on the case:** at 1, 2 and 3 the game takes another path
  earlier and never meets the jelly; it lives in all three (CL4, CL4, CL5).
- **classes-18 plus `809881907` Human Priest, 100,000 game turns, local, base
  `dbc92d18`:**

| mode | deaths | sum of max CL | sum of max DL | steps away | touches by immobile creatures | differ from 0 |
|---|---:|---:|---:|---:|---:|---:|
| 0 | 1 | 111 | 40 | 0 | 0 | — |
| 1 | 1 | 116 | 44 | 81 | 7 | 13 |
| 2 | 1 | 109 | 43 | 88 | 15 | 14 |
| 3 | 1 | 107 | 44 | 99 | 0 | 16 |

  Mode 0's keys are identical to main in 19 of 19. The one death in every
  mode is the same: a blue worm mass kills the CL7 Necromancer 1574344917.
  The differences in CL are within the noise of 19 games. Not a measure of
  benefit (RULES 14); the zoo experiment decides.
- **Re-measured on main `b59ccbb6`** (review of #717: main now ships
  `attack_throw_as_melee = 1`, the throw rating the case turned on), after
  the review's fix (a kept attacker is chosen first): mode 0 keys-identical
  to main in 19 of 19. Mode 2 against main: deaths 0 and 0; sum of max CL
  110 to 112; sum of max DL 41 to 44; 73 steps away; blows taken from
  immobile creatures 15 to 3; 15 of 19 games differ.
- **Left by the review:** the Phase Door rule's "fight lost" test
  (`borg_escape.rs`, `adjacent_foes`) counts a kept neighbour as a foe the
  Borg cannot hurt (its melee is 0), so that escape fires a little more
  eagerly next to one. Not changed here.
## An unseen attacker (#697)

The registry now holds **2210** parameters (2207 after #722; three added).
Owner, 2026-11-19: "it leaves if it senses an invisible thing and goes rest
somewhere else, and use healing potions". The default core's version of
#462.

- **Why:** "Something touches you" makes no monster (`borg_locate_kill`
  returns none), so the danger at the Borg's grid stays about 0: the rest is
  "safe" and `borg_heal` never fires. The only trace was regional fear of 4
  at DL1–4. In #691's measurement (`kill_flow_mode` 3) a CL3 Rogue rested
  through 23 touches of a clear icky thing and died with 2 Cure Light Wounds.
- **`recover_unseen_attack_turns`** (game turns, 0 = C): an unseen attack
  hits or misses, recorded in `borg_update`) counts this long. The engine
  names an unseen monster's blow "Something" (neuter) or "Someone" (male or
  female, `mon-desc.c:123-152`); both count. No HP, rod or mana rest within
  **`recover_unseen_attack_distance`** (10) grids of where it struck (noted:
  "# No rest: an unseen attacker struck N game turns ago."). Hurt or short
  of mana there, the Borg walks to a known grid at least that far, a
  corridor or doorway first (#544's test), and rests there. The flow counts
  danger (not #544's desperate flow) and stays off unknown grids.
  - "Followed": struck again at least that far from the last attack within
    the window. It stays set until the window lapses (or a new level), and
    the Borg leaves the level.
  - "Nowhere to walk" means no *known* grid that far: on a little-explored
    level the Borg leaves rather than explores.
- **`recover_unseen_heal_hp_percent`** (0 = off): while the attack is
  recent, below this share of maximum HP drink Cure Light, Cure Serious,
  then Cure Critical Wounds (the plain quaff: a last Cure Critical is drunk
  too). The walk away comes first; the Borg drinks once it is clear of the
  attack, or when it has nowhere to walk.
- **The case** (1502695781 Human Rogue, `kill_flow_mode` 3): on base
  `a87b4c96` (before the review fixes), with 500 game turns and 50%, struck
  once at game turn 22,631, the Borg walks 10 steps away and is not touched
  there again; it lives to the 100,000-turn cap at CL6 (off: dead at
  23,285). On main `dbc92d18` the game takes another path and lives with the
  switches off too; on, it walks away 15 steps after an unseen touch. With
  the distance at 1 and the heal at 100% it drinks a Cure Light Wounds after
  each of two unseen touches (the heal path, after the review's reordering).
- **classes-18 plus `809881907` Human Priest, 100,000 game turns, local,
  base `dbc92d18`** (after the review of #698): switches off: keys identical
  to main in 19 of 19, smoke identical. On (500, 10, 50): 9 of 19 differ;
  deaths 1 and 1 (the same blue worm mass death in both); sum of max CL 111
  to 116; sum of max DL 40 to 41; 308 steps away, 21 rests held, 9 leaves;
  unseen blows 49 "Something" and 1 "Someone" off, 47 and 3 on, and the
  rule acts on "Someone" too (a walk away, a leave with nowhere known to
  walk). The heal never fired at 50% (once clear, no game was below it).
  Not a measure of benefit (RULES 14).
- **What bounds the rule** (re-review of #698): the C Borg's own reaction to an
  unseen blow is regional fear (`borg_update.rs`, `HIT_BY` adds
  `4 * (depth / 5 + 1)`, `MISS_BY` half), which `borg_danger` counts at the
  Borg's grid. `borg_recover` does nothing while that danger is above
  `avoidance / recover_danger_divisor` (HP / 4). So the walk away and the heal
  act from the first unseen touch until the regional fear crosses HP / 4;
  after that the Borg is back to C's behaviour there. The fear decays by one
  every 10 thinks and covers an 11x11 region. The rule is therefore a
  first-touch rule: it saves the case by walking away at once.
## One breeder policy (#722)

The registry now holds **2207** parameters (2205 after #693; two added). Owner,
2026-11-20: "so many breeder settings"; "assume the level is lost if you could
not kill all the breeders"; "search for stairs first, but if paths are blocked
by breeders, recall at some point".

- **`caution_breeder_policy`** (mode -1..5, compiled -1, **shipped 3**). From
  0 up it sets the six breeder switches by layer (`params::value`, so every
  read is unchanged; `params-core.json` shows the effective values with
  origin `caution_breeder_policy`):
  - **-1:** the switches as set.
  - **0:** C, all off.
  - **1:** `caution_breeder_fight` and `danger_hunt_breeder_no_acid_fear`.
    Stand without an easy escape, hunt few breeders below CL10.
  - **2:** and `recover_rest_breeders`. No rest while breeders are known.
  - **3:** and `caution_breeder_stoppable`. Kill them first while the kills
    outpace the breeding, at any count and level. In addition, at 3 and
    above:
    - **full:** the level is lost at once when more breeders are known than
      `caution_breeder_hunt_max_count` (per class: Warrior 7, Blackguard 8,
      Paladin, Rogue and Ranger 6, others 4; `breeder_level_full`);
    - **search:** a lost level is left even with no stairs known; the flight
      searches for them (`stand_against_breeders`);
    - **Recall:** a flight's Recall waits; **`caution_breeder_lost_recall_turns`**
      (500 game turns) after the loss, Recall is read, stairs known or not
      (`breeder_recall_due`, from `borg_check_breeders`).
  - **4:** and `caution_breeder_avoid`. Paths avoid colonies.
  - **5:** and `caution_breeder_exit`. Walk out of a breeder room.
- **What the analysis on #722 found dead** under the old shipped switches with
  stoppable on: `caution_breeder_hunt_max_count` (now the "full" line),
  `caution_breeder_fight`, the avoidance and exit sub-settings while their
  switches are 0, and `recover_breeder_hunt_hp_percent` with
  `recover_rest_breeders` on.
- **The breeder test bed** (`run.sh ... start-depth=N,start-monsters=...`, the
  frontend's `-start-monsters`): monsters placed near the player after the
  synthetic level is generated, as the engine's debug summon does.
  Deterministic.
- **classes-18 plus `809881907` Human Priest, 100,000 game turns, local, base
  `6f03fea9`:**

| policy | deaths | sum of max CL | sum of max DL | levels lost (full / race) | Recalls |
|---|---:|---:|---:|---:|---:|
| main | 1 | 111 | 40 | — | — |
| -1 | 1 | 111 | 40 | 0 / 0 | 0 |
| 3 (shipped) | 2 | 111 | 43 | 2 / 1 | 0 |
| 5 | 1 | 109 | 44 | 2 / 2 | 0 |

  - -1 is keys-identical to main in 19 of 19.
  - Neither of policy 3's deaths involves breeders: a CL1 Necromancer walked
    into the dark near a floating eye, and a CL5 Priest fought an acolyte and
    a soldier at 10/40 HP.
- **The test bed as tried** (the 18 starts, CL1 fresh characters, DL2 with 11
  worm masses and DL4 with 9 lice and mice, 30,000 turns): 15–18 of 18 die
  under every policy, mostly to plain DL4 monsters and poison, and only part
  of the monsters are placed (in view within 6 grids). It needs a real
  character (`start-build` from a captured CL8–10 build, #382) to compare the
  policies. As tried, it shows no difference between 3, 4 and 5. Not a
  measure of benefit (RULES 14).
## Store at home only when overweight (#711)

The registry now holds **2201** parameters (2200 before; one added).
Owner, 2026-11-20: "only when it weighs too much they should stash the
object with less value at home, untile they are not overcarrying again".

- **`town_store_only_overweight`** (compiled 0, **shipped 1**: owner,
  2026-11-20, "it is a no brainer, turn the switch on"). With it on, the
  live town planner (`dispositions`, `borg_town_utility.rs`) moves a pack
  stack to the Home only while the pack is over its weight allowance. The
  pack is visited lowest keep value per weight first, so the least valuable
  stack goes first, and the storing stops once the pack is under.
- **0 (today):** a stack also goes home when its value kept for later
  (times `borg_town_home_discount` / 1000, shipped 500) minus a home slot's
  cost beats its value carried now minus its carrying cost. Next to a few
  Cure Light Wounds the healing curve is nearly full, so a Cure Serious
  Wounds went home with the pack light (#711; the pack power rule,
  `power_inventory_maxclevel_threshold`, plays no part with
  `borg_town_plan = 1`).
- Taking stacks back from the Home is unchanged.

| parameter | default | unit | range | source |
|---|---|---|---|---|
| `town_store_only_overweight` | 0 | flag | 0–1 | `borg_town_utility.rs:256` |

## No detection prayer by panel at low level (#685)

The registry now holds **2200** parameters (2199 after #625; one added).

- **`light_detect_evil_spell_min_level`** (0 = C): below this character
  level, `borg_check_light`'s panel check casts no Detect Evil, Detect
  Monsters, Read Minds or Seek Battle. Items, rods and Sense Surroundings
  still detect. C casts the prayer on every new panel, so a low-level
  Priest or Paladin, with 2–6 SP, spends them there and then rests for
  mana (`recover_mana_rest_*`): 40% of game time for Priests and 46% for
  Paladins. Detect Evil also misses the animals that kill these classes on
  DL1. Per class with `light_detect_evil_spell_min_level@<class>`. Shipped
  (owner): 10 for Priest, Paladin, Mage, Necromancer and Druid; on
  2026-11-20 10 for every class (#723), then **reverted to 0 for every class**
  (owner, 2026-11-20: "turning on call light and detect monsters on casters
  low level seems a bad call"). A CL1 character sees only the 8 grids next to
  it in a dark room (unlight `2 + lev/6`, or a torch's `LIGHT_2`, radius 1,
  `cave-view.c:658,777`), so it meets a floating eye only when adjacent, and
  the eye's gaze paralyses it before it acts again (165 of 8,702 zoo deaths,
  117 at CL1). Detection shows it first. The zoo measures 0 against 10.
  **Then 10 again for the five casters** (Priest, Paladin, Mage,
  Necromancer, Druid; owner, 2026-11-20, #744; the global line stays 0):
  borg-zoo#480 (main b895e352, classes-1800 starts 1–360, 100k) measured
  11 deaths against 11, CL +0.53 (±0.09), DL +0.13, with no floating-eye
  death in 2,160 games. By class: Mage +1.62 CL, Priest +1.27, Paladin
  +1.10, Necromancer +0.72; the Druid's games were identical. It was the
  fourth measurement with that result.
  Shipped with #568 (PR #723, the same run as there): detection casts on
  the 18 classes-18 starts 62 to 14, most of them the Blackguards' Seek
  Battle (37 to 2 and 20 to 1). The 40% and 46% figures above are the
  Priests' and Paladins' (#685's case); they were not re-measured for the
  classes the global value newly covers.

| parameter | default | unit | range | source |
|---|---|---|---|---|
| `light_detect_evil_spell_min_level` | 0 | level | 0–51 | `borg_light.rs:365` |

## The Phase Door rule's fight counts mana and fail rates (#625)

The registry now holds **2199** parameters (2198 before; one added).
Owner, 2026-11-19: the "can I win" test should check that a caster still has
the mana, and that its spells are likely enough to work.

- `escape_phase_lost_mana` (compiled 1; the rule ships with
  `escape_phase_lost_test` 0, the danger test, so this is not used in
  shipped play, #661): with `escape_phase_lost_test` 1 or 2, the attack `borg_attack` would choose counts at its true damage
  times its success chance: a spell's fail rate (`borg_spell_fail_rate`,
  noted when it passes `borg_spell_okay_fail` in the ranking,
  `spell_okay_fail_noted`), a wand's or rod's `borg_activate_failure` per
  mille / 10. It is used only for the uses the Borg can pay for: casts =
  current SP / the spell's mana, **all of it** (owner, 2026-11-19: "count
  all the mana for fighting, at the beginning we have no choice, and later
  on we will get potions to recover it": the class's escape reserve is not
  set aside); the reserve cast's real mana, not the faked one; a wand's
  charges; one zap for a rod (it recharges over many turns). After that, the
  best true damage of another attack that needs no mana, never the winner
  itself (`est_fallback`), or melee against that monster (`fight_turns`).
  0 = #612's test (the chosen attack at full damage every turn). Staves and
  activations are counted at full damage, with no limit. Melee damage is
  expected (the Borg's hit estimate), a missile's nominal: compared as they
  are (docs/PHASE-DOOR.md). Whirlwind Attack, Maim Foe and Vampire Strike
  now report their true damage (`dmg_best`).

| parameter | default | unit | range | source |
|---|---|---|---|---|
| `escape_phase_lost_mana` | 1 | flag | 0–1 | `borg_escape.rs:940` |


## Throws on melee's footing (#568)

The registry now holds **2198** parameters (2197 after #691; one added).
From #539 (`priest-809881907`): cornered by Grip with 19 Flasks of oil, the
Borg chose melee (44) over oil (15); the owner, from the 1,721 save, threw
one flask and Grip fled.

- **`attack_throw_as_melee`** (flag, 0 = C; **shipped 1**, owner 2026-11-20). With 1, `borg_attack_aux_object`
  rates a throw on melee's footing:
  - **damage**: the engine's throw (`thrown_damage`, `player-attack.c:554-582`
    and `:1296`): dice average plus to-dam, times `2 + weight / 12` for a
    THROWING object, times 3 for EXPLODE. A flask of oil: 22, not 2.
    `best_throw` ranks the objects by it; its gates (worth money, no damage)
    keep the dice average, so `ranged_means` is unchanged;
  - **hit chance**: the engine's (`throw_hit_percent`,
    `player-attack.c:121-219`): the throwing skill (`BI_THT`, 3/2 of it for a
    non-THROWING object), the object's to-hit (and the player's for a
    THROWING one), less the distance; 12% always hit, 5% always miss. It is
    the base chance: the engine halves it against a target that is not
    obvious (`chance_of_missile_hit`), which this leaves out (review of #687).
    The
    damage is scaled by it, as melee scales by its own chance. C used the bow
    skill, the launcher's to-hit and its multiplier, and zeroed the damage
    below 80% of the armour;
  - **danger term**: melee's (`melee_score`): `attack_melee_danger_horizon`
    (2 turns) at full damage, divided by `attack_melee_partial_danger_divisor`
    for a partial kill above `attack_melee_partial_min_level`. C gave a throw
    the bolt's (`attack_bolt_danger_horizon`, 1 turn, partial damage). This is
    where melee's bigger bonus came from.
  - So "melee's footing" holds for the damage, the hit chance and the danger
    term; the point-blank divisor and the kill multiplier below stay the
    bolt's (review of #687).
  - Unchanged: the point-blank divisor for an adjacent non-unique
    (`attack_arrow_point_blank_divisor`, 5; Rogue 3, Ranger 2), the bolt's
    kill multiplier, the missile conservation above CL20. No separate term for
    a monster fleeing: monsters flee from damage, which the engine's damage
    now carries.
- **classes-18, 100,000 game turns, local, plus `809881907` Human Priest**
  (19 games), base `e5cc5469`: switch off plays keys identical to main in
  19 of 19. Switch on: 17 of 19 play differently; flasks thrown 456 to 630,
  melee attacks 1,025 to 1,000; deaths 1 to 2 (the shared poison death, and
  the Priest walking in the dark next to an unseen floating eye at CL2,
  paralysed to death: no attack choice involved); sum of max CL 110 to 104;
  sum of max DL 41 to 40. Not a measure of benefit (RULES 14).
- **Shipped together with #685's global 10** (owner, 2026-11-20, PR #723):
  classes-18 through `harness/run.sh … SEED RACE CLASS 100000 exact,menu`
  (races Human for the first nine, Half-Elf for the second nine, as listed in
  `docs/handover/2026-11-20-723-ship-568-685.md`), base main `4c678c09`, both
  values off against both on: deaths 0 and 0; sum of max CL 109 to 113, sum
  of max DL 40 to 43; flasks thrown 855 to 1,043; all 18 games change. A
  review on another set of starts measured flasks going down (1,413 to
  1,214): the direction depends on the starts. Not a measure of benefit
  (RULES 14); no zoo run.
- **The scene** (the owner's 1,721 save, 6,000 turns): Grip adjacent at game
  turn 1,881, main melees (44); with the switch on the Borg throws a flask
  (80). Both arms survive the scene; with `keep-load-rng` too.
## The hunt removed in steps (#691)

The registry now holds **2197** parameters (2196 before; one added).
Owner, 2026-11-19, on #691: "what if we got rid of it?" and "for me this
flow kill is a candidate for rewrite".

- **`kill_flow_mode`** (mode, 0 = C): what `borg_flow_kill` may walk to.
  `borg_attack` (melee, throws, spells, shots at what is in reach) is not
  affected at any value, nor `borg_flow_kill_aim`, `_corridor`, `_direct`.
  - 0: as C.
  - 1: only the targeted hunts: breeders while a hunt for them applies
    (#503, #558), dangerous town monsters (#619, #637). Their own settings
    are unchanged, but #503's melee hunt, which walks to any monster at C,
    walks only to breeders at 1 (as #558's caster hunt already did): for
    Warriors, Paladins, Rogues and Blackguards mode 1 narrows the targets,
    not only the calls (review of #695).
  - 2: no out-of-view call (the "chase old monsters" calls: `nearness` 250
    and `max_range + 1`, `viewable` false).
  - 3: never.
- **The analysis behind it** is on #691: in 19 games below CL10 the hunt
  walks 263 times against about 22,000 exploring steps; the out-of-view
  call finds a target in 516 calls and walks in 27.
- **classes-18 plus `809881907` Human Priest, 100,000 game turns, local,
  base `a87b4c96`** (19 games):

| mode | deaths | sum of max CL | sum of max DL | walks to a fight | kills | games that differ from 0 |
|---|---:|---:|---:|---:|---:|---:|
| 0 | 0 | 108 | 40 | 305 | 1,427 | — |
| 1 | 2 | 100 | 39 | 26 | 1,234 | 15 |
| 2 | 0 | 111 | 41 | 302 | 1,475 | 11 |
| 3 | 1 | 104 | 41 | 0 | 1,268 | 16 |

  The deaths are not hunting decisions:
  - `1502695781` (modes 1 and 3, same turn): the Borg rests again and again
    while an invisible clear icky thing touches it ("# Invisible monster
    nearby", "Resting to recover HP/SP"), #462's pattern on the borg-rs core;
  - `1472012981` (mode 1): at HP 12/46, unable to flow to an up stair, it
    searches for secret doors until a soldier arrives.
  Below CL10 the hunt is worth about 10% of the kills and a few levels. Mode
  2 loses nothing here. CL10 and above are not measured locally (no game gets
  there); that is the zoo pilot's question. Not a measure of benefit (RULES
  14).

## Phase Door and flasks of oil by class (#682)

The registry now holds **2196** parameters (2194 before; two added). Owner,
2026-11-19: a class-based curve for Phase Door and flasks of oil, "in the
same way are healing objects"; no reserve, only the curve.

- **`town_phase_curve_scale_percent`** (100 = before #682): stretches the
  live town planner's Phase Door curve as `town_heal_curve_scale_percent`
  does healing's: weight and demand scale together, so the first scroll is
  worth the same and more scrolls stay worth buying.
- **`town_oil_curve_scale_percent`** (100): weights each flask of oil's
  thrown damage on the planner's attack curve. A flask's throw has no curve
  of its own (it shares `ATTACK` with missiles, wands and spells; a curve of
  its own would be a new capability and a plan-record schema change, and
  would change the play of every class at 100). Flasks kept as lantern fuel
  are not changed.
- **Per class** (`<name>@<class>`, the class files, owner's values):

| class | Phase Door % | oil % | healing curve % / reserve % (unchanged unless noted) |
|---|---:|---:|---|
| Warrior | 75 | 100 | 50 / 100 |
| Mage | 175 | 100 | 150 / 300 |
| Druid | 175 | 150 | 125 / 250 |
| Priest | 125 | 150 | 125 / 250 |
| Necromancer | 175 | 100 | 150 / 300 |
| Paladin | 100 | 125 | 75 / 150 |
| Rogue | 125 | 125 | 100 / 200 |
| Ranger | 150 | 50 | 100 / 200 |
| Blackguard | 50 | 100 | **100 / 200** (was 75 / 150; owner: as Rogue and Ranger) |

| parameter | default | unit | range | source |
|---|---|---|---|---|
| `town_phase_curve_scale_percent` | 100 | percent | 1–10000 | `borg_town_capability.rs:639` |
| `town_oil_curve_scale_percent` | 100 | percent | 1–10000 | `borg_town_capability.rs:529` |

## No rest while breeders are known (#644)

The registry now holds **2194** parameters (2193 after #556; one added).
Owner, 2026-11-19, on #615: "no rest if aware of any breeders, clear the
breeders or leave the level asap".

- **`recover_rest_breeders`** (flag, 0 = off): in the dungeon, while any
  breeder is known on the level (`mon.breeders`), a rod, HP/SP or mana rest
  that is due is held. If #558's hunt applies (`breeder_hunt_now`), the Borg
  hunts; otherwise it starts leaving the level (`goal.leaving`, which takes
  the stairs out and, with no stairs known, explores for them; note `#
  Leaving (breeders on the level, no rest)`). Note, once a think: `# No
  rest: N breeders known on the level, hunting them|leaving.`
- **The hunt resumes** (review of #656): `breeder_engaged` reads
  `goal.leaving` as "not engaged", so a leave set by this rule first stopped
  the hunt for the rest of the level. The hunt is now judged without the
  rule's own leave (`breeder_engaged_but_own_leave`), and a hunt that
  applies again clears it (`# Not leaving: the breeder hunt applies
  again.`). A leave set by another rule, ignoring the breeders (#503) or
  fleeing still stops the hunt. The withdrawal only takes the rule's own
  leave while the Borg is not fleeing (re-review of #656): the low-hit-points
  and bleeding/poison leaves also set `goal.fleeing`, and may have joined the
  rule's leave after it was set. The ownership flag resets on a new level.
- **The code:** a decision (`breeder_rest_decision`: rest, hunt or leave)
  and its effect (`apply_breeder_rest`), taken only when a rest is due.
- **Not changed:** the Recall rests (a Borg waiting for Recall is leaving
  already); #558's own skip of the rest for the hunt, though with this rule
  on its note (`# Not resting: breeders to hunt.`) gives way to this one.
  #556's avoidance of known colonies (#632) is its own switch.
- **Kept when impaired:** confused, blind, hallucinating or stunned, the rest
  is kept (#615 rule a: a confused Borg that walked instead died in lava).

Evidence, a lead (the 18 classes-18 starts, 100,000 turns, borg-rs core,
main `592c6984`, after the review of #656):

| arm | deaths | sum of CL | keys as main | held rests | hunting / leaving / hunt resumed |
|---|---:|---:|---:|---:|---:|
| off | 0 | 97 | 18/18 | | |
| 1 | 0 | 100 | 5/18 | 561 (13 games) | 126 / 25 / 1 |

Before the fix (main `3ba17e2c`, the hunt stopped by the rule's own leave):
1 death in both arms, sum of CL 96 against 101.

| parameter | default | unit | range | source |
|---|---|---|---|---|
| `recover_rest_breeders` | 0 | flag | 0–1 | `borg_recover.rs:679` |

## Kill breeders first; flee only what cannot be stopped (#556)

Owner, 2026-11-19: "the first choice should be to kill them first and only
escape if it is not possible, the flight should be from places where the
breeding cannot be stopped any more". This replaces the first design below
(avoid breeders whenever known), which borg-zoo#364 measured neutral
(1,440 games, p = 1.0).

- `caution_breeder_stoppable` (compiled 0, shipped 0): each think after the
  monster update (`note_breeders_stoppable`), the known breeders' expected
  births a character turn are summed: the engine's 1 in 8k while the 3x3
  block holds k < 4 monsters, itself included (`monster_turn_multiply`,
  mon-move.c; `mon-play:mult-rate:8`), times the monster's energy over the
  character's. Against them, the Borg's kills a turn with **every attack it
  has** (owner, 2026-11-19: "count all attacks"): the attack planner's own
  dry run (`borg_attack_estimate`: melee, missiles, throws, every spell with
  its mana and fail checks, wands, rods, staves, activations) and the kills
  of the attack it would choose (`kill_best`, `est_kills`): for each monster
  hit, its true damage over that monster's HP, at most 1
  (`kill_fraction`), so a finishing blow is a kill and a ball over three
  mice is three. The dry run draws the Borg's random numbers; their state is
  restored after it. A think is judged only when the planner has something
  to attack, and the level is lost only after
  `caution_breeder_stoppable_confirm` (5) judged thinks in a row. Human Druid 102023799 (borg-cemetery#77) judged 3 mice unstoppable
  on its whip alone while carrying 18 Flasks of oil. Stoppable while births x the percent below < kills.
  - While stoppable, `breeder_stand` and `breeder_hunt` are true at any
    count and character level, escape items or not, so the breeder flees
    (breeder level, "no recall", the scary list's breeder lines, Recall
    as an escape) stand down, and #503/#558's supporting settings
    (stand radius, boosts, no rest while hunting, the acid fear) apply.
  - The danger where the Borg stands at or above its HP pauses the stand,
    and with it the hunt (`breeders_stoppable` is false that think), for
    that think only. The usual caution still acts that think: the Borg
    retreats and escapes. The three level-long effects of not standing (the
    "ignoring breeders" leave of `borg_check_breeders`, the breeder flee,
    the breeder scary level) do not follow from a pause
    (`breeder_stand_paused`, #722's lead b; before it, one dangerous think
    latched the flight for the level).
  - Once the breeding outpaces the kills, the level is lost to them until
    a new level (`breeders_lost`, noted `# Breeders cannot be stopped`);
    the flees act as before, and with `caution_breeder_avoid` the flight
    avoids the colony. While no staircase is known the Borg still stands
    (`stand_against_breeders`): a flight with nowhere to go only wanders
    through the colony (borg-cemetery#77).
  - The engine's cap (`repro-max:100`) counts live breeders
    (`num_repro` drops when one dies, mon-make.c), so a capped colony is
    not a finished one and the test ignores it.
- `caution_breeder_stoppable_births_percent` (200): the births' weight
  (the Borg walks between breeders and does not see them all).
- `caution_breeder_avoid` now blocks only during a flight (fleeing, or the
  level scary). A staircase inside a colony still draws a fleeing Borg in
  (the flow starts from it): that is the owner's "fight our way out", and
  with the stand-down the Borg fights along the way.

| parameter | default | unit | range | source |
|---|---|---|---|---|
| `caution_breeder_stoppable` | 0 | flag | 0–1 | `borg_caution.rs:3296` |
| `caution_breeder_stoppable_births_percent` | 200 | percent | 1–10000 | `borg_caution.rs:3362` |

## Avoid breeder colonies (#556)

The registry now holds **2193** parameters (2184 before; nine added).
Owner, 2026-11-18: "maybe in that scenario one should not walk into a room
with breeders then". The flows that move the Borg ignored known breeders, and
a breeder out of view is forgotten, so the next think's flow walked back into
the colony; a Phase Door could land in it (Half-Elf Priest 592068863,
borg-cemetery#73).

- `caution_breeder_avoid` (compiled 0, shipped 0): each think the grid of
  every known breeder is remembered with the kill's `when`
  (`note_breeders`, after the monster update; cleared on a new level; no
  random numbers). Every flow spread except a kill flow's
  (`borg_flow_spread_hunting`: the kill flows and their digging, so #503's
  hunt still reaches the breeders) refuses a grid within the radius of a
  remembered breeder, except the grids next to the Borg (its first step).
  So the stair and flee flows, exploration, the item flows and the breeder
  exit route around the colony, or find no route. A Phase Door or teleport
  landing grid there counts as scary (`count_scary`).
- `caution_breeder_avoid_radius` (1): the engine's distance from a mark.
- `caution_breeder_avoid_turns` (50): a mark is kept this many Borg turns
  (`time.now`, one per think) after the breeder was last known there.
- `caution_breeder_avoid_fight_through` (1; owner, 2026-11-19: "fallback
  to fight our way out"): a flight (`GOAL_FLEE`) whose stair flow finds no
  route around the marks takes the plain route through them
  (`fight_through`, borg_flow_stairs.rs; noted `# Fighting through the
  breeders to the stairs`); 0 = no route, the flight looks elsewhere.
- `caution_breeder_through_turns` (5): for this many Borg turns after the
  fight-through, retreat and back-away stand down while the danger here is
  below the HP (`breeder_fighting_through`), so the Borg kills what blocks
  the way and goes on instead of stepping back (the Priest's ping-pong).
- `caution_breeder_stuck_turns` (20): when the Borg is stuck (the dungeon
  think reaches its bravery boosts) while marks block its flows, the marks
  are ignored for this many Borg turns (`suspend_breeder_marks_when_stuck`;
  noted `# Stuck behind remembered breeders`), so it fights its way out
  instead of staying boxed in; 0 = never. Human Druid 102023799 (classes-18,
  current main): no staircase known, every exploration route marked, stuck
  for 1,200 turns until the mice killed it.

| parameter | default | unit | range | source |
|---|---|---|---|---|
| `caution_breeder_avoid` | 0 | flag | 0–1 | `borg_caution.rs:3259` |
| `caution_breeder_avoid_radius` | 1 | grids | 0–10 | `borg_caution.rs:3280` |
| `caution_breeder_avoid_turns` | 50 | Borg turns | 1–100000 | `borg_caution.rs:3263` |
| `caution_breeder_avoid_fight_through` | 1 | flag | 0–1 | `borg_caution.rs:3517` |
| `caution_breeder_through_turns` | 5 | Borg turns | 0–1000 | `borg_caution.rs:3526` |
| `caution_breeder_stuck_turns` | 20 | Borg turns | 0–1000 | `borg_caution.rs:3500` |

## Ranged first against immobile attackers (#646)

The registry now holds **2184** parameters (2183 after #647; one added). Owner, 2026-11-19: "always choose ranged against immobile but annoying monsters". `caution_ranged_vs_immobile` (0, off). At 1, next to a known monster that never moves and attacks with its blows (`NEVER_MOVE` with a blow and without `NEVER_BLOW`: mushroom patches, molds, a floating eye), while confused or while carrying a ranged attack (`BI_AMISSILES`: flasks below CL15, ammunition), the Borg steps to a free grid out of its reach ("# Stepping away from immobile …"; to a less dangerous grid that is not a bounce; with a ranged attack once per attacker per level, confused up to three times, remembered by the attacker's grid, and a full memory refuses the step), and it does not cure confusion next to it ("# Not fixing Confusion next to an immobile attacker"). Neither applies when the moving monsters within two grids are as dangerous as the HP left. The cure is held at both confusion-cure sites (`borg_heal` and the cures at the end of `borg_caution`). A confused step goes astray 75% of the time (`player_confuse_dir`, `player-util.c:1358`) and is tried again. From borg-cemetery#75: a CL1 Priest drank three cures next to a grey mushroom patch, which confused it again each time. Not here: walking up to an immobile monster in the first place, and buying something to throw (#649, #652).

## A flight in town takes a step no worse than standing (#647)

The registry now holds **2183** parameters (2182 on main; one added).

#647: a CL4 Druid (start 738785, race 4, on #620's branch) twitched in town
for 70,000 turns. The cause, from a replay with logging: the Borg knew the
staircase, four steps away, and the stair flow queued it; but the flow
refuses every grid whose danger is above its fear (`flow_fear`: 90, the
avoidance 30 x 3 on a scary level). A filthy street urchin next to the Borg
made its own grid 406 and the grid toward the stairs 203 (C's "avoid them
stupid urchins": `danger_monblow_eat_gold_fear` below
`danger_gold_theft_low_level`). No path left the Borg's grid, Phase Door
was spent, and a scary level allows no attack, so only twitching was left.

With `caution_town_flee_no_worse = 1`, in town while the level is scary or
the Borg flees or leaves, the flow's fear is at least the danger at the
Borg's own grid (`borg_caution::town_flee_fear_floor`, in
`borg_flow_spread` and `borg_flow_enqueue_grid`): standing still is no
safer. It applies to every flow in town while scary or fleeing; a grid
already marked icky under a lower floor stays so until the level changes.
On the stalled game's build (#620's branch; main does not reproduce the
stall): 7,002 "Twitchy!" notes to 0, and the Borg leaves town. On main the
switch is insurance against a rare deadlock.

On main `3ba17e2c` (100,000 turns, the zoo adapter's steps) no game stalls
with the switch off:

| starts | arm | keys as off | deaths | sum of CL | game turns in town |
|---|---|---:|---:|---:|---:|
| classes-18 | off | | 1 | 101 | 347,559 |
| classes-18 | on | 18/18 | 1 | 101 | 347,559 |
| 16 more Druids | off | | 2 | 104 | 199,714 |
| 16 more Druids | on | 14/16 | 2 | 103 | 200,867 |

| parameter | default | unit | range | source |
|---|---|---|---|---|
| `caution_town_flee_no_worse` | 0 | flag | 0–1 | `borg_caution.rs` (`town_flee_fear_floor`, read by `borg_flow_spread` and `borg_flow_enqueue_grid`) |

## Projectiles and devices by class (#666)

The registry now holds **2182** parameters (2179 after #615; three added). Owner, 2026-11-19: "weighing buying projectile and magic by class. Not only rangers, but other classes should buy more projectiles"; by shooting skill and by device skill; "projectile buying is more important if they cannot get flasks of oil or wands".
- `town_missiles_percent` (compiled -1 = C, the Ranger's reserve only): the town plan's reserve of shots for the class's launcher, in percent of `borg_town_w_ranger_missiles`, for any class. Shipped -1: measured with the shooting-skill values (Rogue 90, Warrior 75, Paladin 70, Druid 55, Priest and Blackguard 50, Mage and Necromancer 30; Ranger 72 = 100) over classes-18, the extra ammunition took gold from healing and Phase Door (sum of CL 97 → 95, max DL 37 → 34), and flasks, always in the General Store, come first (owner).
- `town_missiles_other_ranged_percent` (compiled and shipped 100; for when the shots are set per class; owner: "I would make them buy it mostly when they cannot find flasks of oil at all", and the General Store always stocks flasks, `store.txt`): with 5 or more flasks of oil and attack-wand charges (Magic Missile, Stinking Cloud) in the pack, the shots weigh this percent.
- `town_devices_percent` (compiled 100): a wand, rod or staff is worth this percent of what it supplies. The class files set it by device skill (24 = 100): Mage and Necromancer 150, Rogue 135, Priest 125, Ranger 115, Druid, Paladin and Blackguard 100, Warrior 75.

## No rest with a monster near (#615, rule a)

The registry now holds **2179** parameters (2177 after #567 and #637; two added).
Owner, 2026-11-19, on the rest analysis (borg-cemetery#74): "ok do a". In 108
games on main, 63% of town rests and 13% of dungeon rests lasted one player
turn, mostly with an awake monster in view: the game breaks the rest and the
Borg starts another, up to 32 in a row. `borg_check_rest` refuses a rest for
monsters close by (within 1 grid, 2 for a mover, 5 for a scary one, 10 for a
breeder, a ranged attacker in view); it has no memory of a monster that was
near a moment ago, and none of an awake one further off that may come. The
game breaks a rest when a monster comes into view (`mon-util.c`, "Disturb on
appearance").

- **`recover_rest_monster_turns`** (game turns, 0 = off): no rod, HP/SP or
  mana rest in `borg_recover` while an awake monster that can move was known
  within `recover_rest_monster_distance` (20 grids) in the last N game turns
  on this level. The Borg does something else that think. Note: `# No rest:
  a monster was near N game turns ago.` (once a think).
- **The memory:** #567's scan (`nearest_mover_seen`), run each dungeon think
  in `borg_caution` and before each rest. The clock is set while an awake
  monster that can move is **on screen** (`kill.seen`), and then holds rests
  for N game turns after it leaves the screen or is forgotten. Review of
  #645: it records which monster set it, and that monster's death
  (`borg_count_death`) clears it, so a dead yeek no longer holds a rest.
- **Not when impaired:** confused, blind, hallucinating, stunned or afraid,
  the rest is kept. In a first local run a confused Ranger held its rest in
  town, walked toward its home into lava and died; an afraid Borg twitches
  in a random direction (review of #645).
- **The walk to a recovery grid** (`borg_flow_recover`, `GOAL_RECOVER`) is
  held too (`holding_for_monster`, review of #645), as #614 does: the Borg
  could not rest there either.
- The Recall rests (all three, #567) keep their own switch,
  `caution_recall_rest_monster_turns`.

Evidence, a lead (the 18 classes-18 starts, 100,000 turns, borg-rs core,
branch with main `13e1187c`):

| arm | deaths | sum of CL | rests started | held-rest notes |
|---|---:|---:|---:|---:|
| off (keys = #640's off, 18/18) | 1 (Priest, game turn 8,158) | 101 | 1,893 | |
| 100 | 2 (the same Priest; a Druid at 90,700, 9,000 game turns after its last held rest, walking to the stairs at 2 HP) | 96 | 1,203 | 1,172 |
| 200 | 1 (the same Priest) | 97 | 1,246 | 1,684 |

Re-taken after the review of #645 (the clock clears on the monster's death,
afraid is impaired, the recovery-grid walk waits). The reviewer's own 66
games (6 races x 11 classes) gave 3 deaths at 100 and at 200 against 6 off,
and a higher mean minimum HP. Direction only: the zoo decides.

| parameter | default | unit | range | source |
|---|---|---|---|---|
| `recover_rest_monster_turns` | 0 | game turns | 0–100000 | `borg_recover.rs:56` |
| `recover_rest_monster_distance` | 20 | grids | 1–100 | `borg_recover.rs:57` |

## No Recall rest right after a monster was near (#567)

The registry now holds **2177** parameters (2175 after #637; two added). From
`priest-809881907` (borg-zoo#227, #539): at game turn 1,761 the Priest saw
Grip 4 grids away and read its only Word of Recall. A think later, at 1,771,
Grip was out of view and forgotten (its kill record deleted, danger 0), so
the Borg rested for Recall. Grip came back at 1,791 and bit (13 to 10 HP).

- **`caution_recall_rest_monster_turns`** (game turns, 0 = off): while Recall
  is pending, if an awake monster was known within
  `caution_recall_rest_monster_distance` (20 grids) in the last N game turns
  on this level, neither "Resting for Recall" (`caution_recall_rest_turns`)
  nor "Waiting for Recall" (9 turns) runs. The Borg stays still one turn
  (`,`, the engine's "stay still"; a rest count of 1 repeats the previous
  count, `cmd-cave.c` `do_cmd_rest`), so every think sees a monster that
  comes back. Note: `# Watching for Recall: a monster was near N game turns
  ago.` The Borg thinks about once a player turn (about 10 game turns at
  normal speed), so the threshold moves in steps of about 10: 200 is about 20
  thinks.
- **A wait, not a rest:** staying still does not heal the way resting does.
  Where "Resting for Recall" would also have healed, the switch trades that
  healing for a watch, for up to N game turns.
- **Three Recall rests:** "Resting for Recall" (`borg_caution`), "Waiting
  for Recall" (`borg_think_dungeon`) and the wait with no light
  (`borg_think_dungeon_light`, review of #640), which runs before
  `borg_caution`; the hold refreshes the memory itself, so all three see the
  monsters of this think (re-review of #640).
- **The memory:** each dungeon think, `borg_caution` records the game turn
  and depth while such a monster is **seen** in this update (`kill.seen`).
  It outlives the forgetting of a monster out of view. Review of #640: a
  kill record keeps its last position for up to 2,000 ticks after the
  monster is lost, and without `kill.seen` it refreshed the watch every
  think (192 of 420 watches at "0 game turns ago"; one Ranger stood still
  for about 330 game turns for a kobold it could not see). A monster that never moves (a floating eye, a mold)
  does not count: in a first local run a floating eye held every Recall wait.
- **With #603:** `caution_recall_wait_as_usual` keeps the rest in place when
  safe; this switch decides whether that rest is a long one.

| parameter | default | unit | range | source |
|---|---|---|---|---|
| `caution_recall_rest_monster_turns` | 0 | game turns | 0–100000 | `borg_caution.rs:4351` |
| `caution_recall_rest_monster_distance` | 20 | grids | 1–100 | `borg_caution.rs:4354` |

## Kill on sight in town: caster walk, wands, a sure preference (#637)

The registry now holds **2175** parameters (2172 after #620; three added).
From the review of #624: three limits of `attack_town_dangerous_on_sight`,
each now a switch (compiled and written 0 = #619's behaviour), each needing
`attack_town_dangerous_on_sight = 1`:

- `attack_town_dangerous_caster_walk`: a Mage or Necromancer below
  `borg_flow_kill`'s caster level (CL25 in town; not only below the town
  hunt's CL20 gate) walks to a dangerous town monster too, with at least
  half its mana (#558's condition); otherwise its town hunt is breeders
  only, and no town race breeds.
- `attack_town_dangerous_wand`: in town a wand (not Wonder) may be aimed
  when a dangerous town monster was seen within `attack_max_target_age`,
  and only those are worth its charge (`town_wand` zeroes the others in
  `borg_launch_damage_one`). In town the wand's presence is checked before
  C's paranoia roll, which draws the Borg's own RNG (carried across thinks):
  a first version rolled it with no wand and changed 17 of 18 games without
  aiming one.
- `attack_town_dangerous_bonus_whole`: `attack_town_dangerous_bonus_percent`
  multiplies the danger term of the attack's value too (`melee_score`, the
  bolt value in `borg_launch_bolt_aux_hack`), not only the damage; in melee
  after the partial-kill division.

On borg-zoo's 18 classes-18 starts (100,000 turns, main `3ba17e2c`, the zoo
adapter's steps), with `attack_town_dangerous_on_sight = 1`: each switch alone
plays all 18 games as without it (the town scare still sends the Borg away
from the four at low level, the Borg has no wand in town, and the damage term
already won every choice). With `caution_town_never_scary = 1` as well, all
three together change 2 of 18 games, both Necromancers (the caster walk), with
the same levels; deaths 4 and 4, sum of CL 83 and 83.

With #622's `caution_town_scary_levels_less` raised from the shipped 3 (the
town's list scares end sooner; owner: test kill on sight with the town
scary until a level), the same starts:

| N | arm | keys as the row above | deaths | sum of CL | game turns in town | kills of the four | thefts |
|---|---|---:|---:|---:|---:|---:|---:|
| 6 | on_sight 0 | | 1 | 97 | 261,010 | 17 | 1 |
| 6 | on_sight 1 | 14/18 | 1 | 97 | 256,359 | 13 | 0 |
| 6 | + the three | 18/18 | 1 | 97 | 256,359 | 13 | 0 |
| 11 | on_sight 0 | | 1 | 94 | 212,355 | 23 | 11 |
| 11 | on_sight 1 | 8/18 | 1 | 94 | 209,471 | 22 | 15 |
| 11 | + the three | 13/18 | 1 | 98 | 218,887 | 27 | 13 |

At N = 6 the three still change nothing. At N = 11 (no list scare left in
town) they change 5 games: 4 levels more, 5 more kills of the four, 2 thefts
fewer, the same death. Kill on sight alone kills fewer of the four than
no kill on sight at N = 6 (13 against 17) and N = 11 (22 against 23):
unexplained, 18 games. Direction only: a lead for a zoo arm, not a claim
(RULES 14). Rerun after the review's fixes (the caster walk also reaches
CL20-24): the N = 11 row with the three is unchanged (13/18, CL 98, 1 death).

| parameter | default | unit | range | source |
|---|---|---|---|---|
| `attack_town_dangerous_caster_walk` | 0 | flag | 0–1 | `borg_fight_attack.rs` (`town_caster_walk`, read by `borg_flow_kill`) |
| `attack_town_dangerous_wand` | 0 | flag | 0–1 | `borg_fight_attack.rs` (`town_wand_allowed`, `borg_attack_aux_wand_bolt`) |
| `attack_town_dangerous_bonus_whole` | 0 | flag | 0–1 | `borg_fight_attack.rs` (`town_danger_bonus`) |

## Fox Form for a Druid's walks in town (#620)

The registry now holds **2172** parameters (2169 after #619; three added).
Owner, 2026-11-19 (#613): "for the druid what about using fox at lower levels
for moving faster and ignore the town mobs for the most time?"

With `escape_fox_form_town = 1`, a Druid at most
`escape_fox_form_town_max_level` (20), about to take a step of a walk in
town (to a shop, home, a staircase, or a flight; not a kill, a take or a
rest) at
least `escape_fox_form_town_min_steps` (10) from its goal, casts Fox Form
(`MOVES[1]` in `shape.txt`: a step costs half the energy) if its fail rate
is at most `escape_fox_form_fail`. It casts only when the mana left stays
at or above `borg_recover`'s mana rest (`recover_mana_rest_sp_tenths`
below `recover_mana_rest_max_level`), and not while standing on a
staircase. It keeps the shape while the walk lasts, changes back on a
staircase (before the stairs are taken), at the shop door (a shapechanged
player is refused: `player-util.c` `player_handle_post_move`, "the door
slams shut"; the home refuses silently) and before a blow (#551's
`revert_fox_to_fight`); the change-back prompt is answered as with
`escape_fox_form`. A step into an unseen monster is still struck as a fox
(#551's gap). Harmless town monsters are left to #621's ignore list. The
fail rate is #551's `escape_fox_form_fail`, shared on purpose (same spell);
tuning the two apart would need a parameter of its own.

On 18 Druid starts (the two classes-18 Druids and 16 more, 100,000 turns,
main `3ba17e2c`, the zoo adapter's steps):

| arm | keys as off | deaths | sum of CL | game turns in town | casts |
|---|---:|---:|---:|---:|---:|
| off | | 2 | 117 | 233,996 | |
| on | 3/18 | 2 (the same two) | 107 | 276,229 | 153 |
| on, review fixes | 3/18 | 2 (the same two) | 114 | 245,316 | |

One game accounts for the rise in town time: it went another way and stuck
in town for 70,000 turns, fleeing an adjacent filthy street urchin with no
staircase found, unrelated to the fox (no cast in its last 400 turns
before; #647). Without it, town time falls by 10% (194,236 against
214,651), lower in 10 games, higher in 5. The levels fall in 8 games and
rise in 2. A first version without the mana condition doubled the town
time: each cast sent the Druid to rest for its mana. With the review's
fixes (no fox on a staircase: the first version took the stairs as a fox 42
times in 13 games, now 0; no fox for a rest) the levels are 114 against
117 and the town time +5%, with no stall. Direction only. On the 6-game
measure of `harness/borg.txt` the switch changes the Druid's game (seed
102023799) from its first cast at about turn 37,000, and none of the other
five (re-review of #648). The change back on a staircase writes "# Fox
Form: changing back on the staircase".

| parameter | default | unit | range | source |
|---|---|---|---|---|
| `escape_fox_form_town` | 0 | flag | 0–1 | `borg_escape.rs` (`any_fox_form_on`, `town_fox_now`; the cast in `fox_form_before_town_step`, called by `borg_flow_old`; the keep in `keep_fox_form`; the shop door in `revert_fox_at_shop`, called by `borg_play_step`) |
| `escape_fox_form_town_max_level` | 20 | level | 1–51 | `borg_escape.rs` (`town_fox_now`) |
| `escape_fox_form_town_min_steps` | 10 | steps of flow | 1–250 | `borg_escape.rs` (`fox_form_before_town_step`) |

## Dangerous town monsters killed on sight (#619)

The registry now holds **2169** parameters (2166 after #603; three added). Owner, 2026-11-19:
"I would also kill the dangerous ones on sight with the best attacks that we
have, if possible", and "test it together when town is scary until a certain
level, also dependent on the level of the monsters that are scary".

A town monster (race level 0) is dangerous when a blow steals (`EAT_GOLD`,
`EAT_ITEM`) or hurts for at least `attack_town_dangerous_min_damage` (6,
dice x sides): the battle-scarred veteran (2d6), the mean-looking mercenary
(1d10), the squint-eyed rogue (1d6, steals) and the filthy street urchin
(steals). With `attack_town_dangerous_on_sight = 1`, in town, its attack
value is multiplied by `attack_town_dangerous_bonus_percent` / 100 (300), so
`borg_attack` picks it first with its best attack (not a wand: C never aims
one in town), and the hunt walks to it below CL20 (C never hunts in town
then); a Mage or Necromancer below CL25 still doesn't walk to it, since its
town hunt is breeders only and no town race breeds. The bonus multiplies
the damage term only, not the danger term `melee_score` adds: that it makes
the four the preferred target was measured through their kill counts, not
checked per target choice. The scare is not touched: while the
scary list makes the town scary the Borg flees as before.

On borg-zoo's 18 classes-18 starts (100,000 turns, main `38858846`):

| arm | keys as main | deaths | sum of CL |
|---|---:|---:|---:|
| main | | 0 | 99 |
| switch 0 | 18/18 | 0 | 99 |
| switch 1, shipped list | 16/18 | 0 | 99 |
| town-calm list (borg-zoo#328) | 0/18 | 2 | 102 |
| switch 1, town-calm list | 0/18 | 2 | 101 |
| `caution_town_never_scary = 1` | 0/18 | 2 | 97 |
| switch 1, town never scary | 0/18 | 3 | 89 |

With #622's `caution_town_scary_levels_less` (a build of both): N = 3 gives
0 deaths and 101 CL, N = 6 1 and 102, N = 11 3 and 90. Killing them on sight
raises the kills of the four (20 to 53 at N = 3) but not the thefts. Without
the scare it costs deaths. Direction only: 18 games, no zoo run, and all
before #621's ignore list (none of the four is ignored: their blows are
not MOAN, BEG or DROOL) and #595.

| parameter | default | unit | range | source |
|---|---|---|---|---|
| `attack_town_dangerous_on_sight` | 0 | flag | 0–1 | `borg_fight_attack.rs` (`town_kill_on_sight`; the bonus in `borg_thrust_damage_one`, `borg_launch_damage_one`; the hunt in `borg_flow_kill`) |
| `attack_town_dangerous_min_damage` | 6 | damage | 1–1000 | `borg_fight_attack.rs` (`dangerous_town_race`) |
| `attack_town_dangerous_bonus_percent` | 300 | percent | 100–10000 | `borg_fight_attack.rs` (`town_kill_bonus`) |

## Play by the ordinary rules while Recall counts down (#603)

The registry now holds **2166** parameters (2165 after #614; one added). `caution_recall_wait_as_usual` (compiled 0, shipped 1 since #694): while Word of Recall is pending (`goal.recalling`), the Borg plays by its ordinary rules. The engine keeps the countdown through a level change (`player->word_recall`, `effect-handler-general.c`), so none of the Recall-only rules is needed for the jump. Under the switch, `recall_restricts` is false and seven places act as without Recall: the flee and leave goals (`borg_think_dungeon.rs`), taking stairs down and the staircase as an escape (`borg_caution.rs`), the stair flows (`borg_flow_stairs.rs`), Teleport Level (`borg_escape.rs`), and "fight because recalling" on a scary level (`borg_fight_attack.rs`); `buy_time_for_recall` is skipped. Kept: resting or waiting in place when safe, the town buffs before the jump, and "not bored". Only on DL2 and deeper: in town and on DL1 a staircase crosses between town and dungeon, and the Recall would fire on the other side and take the Borg back (owner, #627).

| parameter | default | unit | range | source |
|---|---|---|---|---|
| `caution_recall_wait_as_usual` | 0 | flag | 0–1 | `borg_caution.rs:3004` |

## Rest at the stairs (#614)

The registry now holds **2165** parameters (2160 before; five added). Owner,
2026-11-19, from the death of Elf Druid 1678765969 in borg-zoo#312's small
tier: "once committed to a stair, it must at least reach that stair before
stopping to rest ... He can rest on the stairs, and if a mob appears he can
climb up/down immediately." That Druid, fleeing as a fox to stairs 11 steps
away, stopped for `borg_recover`'s low-level mana rest. A white harpy and an
apprentice arrived during the rest, and it never reached the stairs.

- **`recover_rest_reach_stairs`** (steps of flow, 0 = off):
  - **The rule:** on a flight (`GOAL_FLEE`) to a staircase at most this many
    steps away, the Borg holds every rest in `borg_recover` (rod, HP, mana)
    until it stands on a staircase. It also skips `borg_think_dungeon`'s flow
    to a recovery grid (`borg_flow_recover` and `GOAL_RECOVER`, review of
    #616), which could walk it off the flight. Potions and cures are not
    rests. Note: `# No rest: committed to the stairs, N steps to go.`
  - **Why a distance:** a first version held rests at any distance. A hurt
    Druid then walked a 138-step flight without resting and died to a crow.
  - **Surviving a re-plan:** a new monster wipes the flow goal for a think.
    The commitment still holds while the flight is on (fleeing, leaving or a
    scary level), on the same level, if its last step was within
    `recover_rest_reach_stairs_memory` (10 Borg-time units). Each flight step
    sets `borg.flight_steps`, `flight_time` and `flight_depth`.
- **`caution_stairs_leave_on_monster`** (flag): in the dungeon, a Borg
  standing on a staircase that still needs to recover (HP or SP below the
  maximum) takes the stairs when a monster threatens it. Note: `# Leaving by
  the stairs: a monster while recovering (danger N, up|down).`
  - **Not in town** (review of #616): the town's down staircase made it dive
    into DL1, 38 of 72 firings.
  - **A real threat:** the danger here must be at least
    `caution_stairs_leave_danger_percent` (10) of the current HP, and at least
    1. Before, it fired at any danger above 0, 59 of 121 times at danger 1.
  - **The direction:** `caution_stairs_leave_down` (1) takes the staircase
    under the Borg, up or down (the owner's "climb up/down"); 0 takes an up
    staircase only. The staircase sets `stair_less` or `stair_more` itself,
    and `choose_stairs` is skipped then, so nothing cancels a noted
    departure. While rising, a down staircase is held as in `choose_stairs`
    (#483, #557; re-review of #616).
- **Only a flight to stairs:** the breeder exit (#544) and the flow to light
  are `GOAL_FLEE` flows too. `borg.flight_to_stairs`, set by the stair flows
  and cleared by every other flow commit, keeps them from holding rests
  (re-review of #616).

Compiled and shipped 0 (C), with the companions at 10, 10 and 1. Evidence, a
lead (the 18 classes-18 starts, 100,000 turns, branch merged with main
`951eecbd`, borg-rs core):

| arm | deaths | sum of CL | sum of max depth | keys as off | held rests | stairs taken (up/down) |
|---|---:|---:|---:|---:|---:|---:|
| off | 0 | 99 | 41 | | | |
| reach 20 | 2 (Mage, an apprentice; Druid, a crow) | 92 | 39 | 5/18 | 228 | |
| leave 1 | 1 (Priest, a giant white mouse) | 97 | 39 | 10/18 | | 20/3 |
| leave 1, up only | 1 (the same Priest) | 98 | 38 | 10/18 | | 22/0 |
| both | 2 (the Mage and the Druid) | 93 | 37 | 3/18 | 219 | 17/4 |

Every death comes thousands of game turns after its game first differs. The
Mage's last held rest was at 23,595 and it died at 28,905. The Druid's was at
4,663 and it died at 13,826. The Priest's last leave was at 63,020 and it died
at 86,224, among breeders. So the deaths are leads. The review of #616
reported 45 starts of the `full` suite for the first version: reach 20 had 2
deaths against 3 off, with higher levels and depth.

**The scene itself:** replayed from the zoo cell with this branch. Off, it
replays the zoo death byte for byte. With reach 20 the Druid holds the rest
11 steps out and keeps its flight when the harpy appears. It kills the harpy
with Stinking Cloud, reaches the up staircase at game turn 67,268, rests on
the stairs of the new level, and lives to 100,000 (CL7). The zoo is the
measurement.

| parameter | default | unit | range | source |
|---|---|---|---|---|
| `recover_rest_reach_stairs` | 0 | steps of flow | 0–250 | `borg_recover.rs:143` |
| `recover_rest_reach_stairs_memory` | 10 | borg time | 0–1000 | `borg_recover.rs:164` |
| `caution_stairs_leave_on_monster` | 0 | flag | 0–1 | `borg_caution.rs:4249` |
| `caution_stairs_leave_danger_percent` | 10 | percent of current HP | 0–1000 | `borg_caution.rs:4248` |
| `caution_stairs_leave_down` | 1 | flag | 0–1 | `borg_caution.rs:4255` |

## Ignored town monsters (#618, #621)

The registry still holds **2160** parameters (none added). Owner, 2026-11-19
(#613): leave Farmer Maggot alone; then "should we not have an ignore list?",
"yes reuse it, ignore 10-20", and "add the first ones too, unless they are
in the way".

`harness/scary_monsters.txt` gets a flag, `ignore`: an ignore line is not a
scare. Up to its level (by class, as the scary levels) the Borg doesn't
target or hunt the monster (`borg_fight_attack.rs` `ignored`: its attack
value is 0 in melee and at range; `borg_flow_kill` skips it), but one
standing in its way is still attacked: the walk step is unchanged. A scare
line and an ignore line for the same monster are separate entries; a later
line replaces only its own kind, and `monster | none | | ignore` removes the
ignore line. The compiled C list (`borg_scary_list = none`) has no ignore
entries. Ignore lines apply whatever `caution_scary_named_monsters` says
(that switch is for the scares), and an ignore line takes no `breeders`
flag (it would do nothing, so the parser refuses it).

Shipped, at 10-20 (CL10 for Warriors, Paladins and Blackguards up to CL20
for Mages and Necromancers): Farmer Maggot (MOAN, 350 HP) and the six
townsfolk that do no damage (BEG or DROOL in `monster.txt`): the
pitiful-looking beggar, the singing, happy drunk, the blubbering idiot, the
village idiot, the mangy-looking leper and the boil-covered wretch.

On borg-zoo's 18 classes-18 starts (100,000 turns, main `4acf071f`, the zoo
adapter's steps, the borg-rs core):

| | main | ignore list |
|---|---:|---:|
| keys as main | | 0/18 |
| deaths | 1 (a wild cat, CL1) | 1 (a rock lizard, CL1) |
| sum of CL | 91 | 101 |
| attacks aimed at the seven (`# Facing`/`# Targeting`) | 509 | 0 |
| the seven killed | 220 | 70 |
| walk-ins on one in the way | 8 | 121 |
| game turns in town | 441,954 | 347,559 |

Game turns in town: the sum, over `state.log` rows at depth 0, of the turn
gap to the next row. Shipped play changes, so `harness/reference/smoke-borg-rs`
is re-recorded (#606). Direction only: 18 games, no zoo run.
## The scary flag clears when the scary monster is hurt or afraid (#593)

The registry now holds **2160** parameters (2159 after #612; one added). `caution_scary_unlatch_calmed` (compiled 0, shipped 1 since #671): with `caution_scary_stand`, the level's scary flag clears once no scary monster is known and a named-list or damage-rule scary monster is known hurt or afraid in this scan, or one was killed, with the latch on or off. Only a race that made the level scary counts (#590's record): a monster hurt or afraid from first sight never made it scary, so it clears nothing (second review of #595). A monster that recovers is scary again at the next scan.

## The owner's Phase Door rule (#584)

The registry now holds **2159** parameters (2156 after #622; three added).
Owner, 2026-11-18: "to use Phase Door, enemies must be on the square next to
you"; otherwise escape if possible, else attack with magic or missiles, else
boost (Speed included). docs/PHASE-DOOR.md maps the 16 Phase Door sites.

- `escape_phase_adjacent_rule` (compiled 0, shipped 1 since #661): at every site, before
  its own check, Phase Door only with an awake enemy adjacent and the fight
  judged lost. The rule only refuses the Phase Door; the escape levels go on
  with their other escapes, and if none acts the Borg attacks (its best
  attack by its own ranking) or, with an awake monster in view, boosts
  (`phase_rule_act`, once per think). Decided and noted once per think.
- `escape_phase_lost_test` (compiled 1, shipped 0 since #661; owner 2026-11-19: both comparisons behind a
  setting): how the adjacent fight is judged lost. 0 = the threshold below;
  1 = the fight compared: killing the adjacent awake monsters one after
  another (their HP left) takes more turns than the danger where it stands
  needs to kill it; its damage a turn is the larger of its melee and the true
  damage of the attack it would choose now, of any kind
  (`borg_attack_estimate`, without the kill, danger and unique bonuses);
  2 = as 1, and the mean danger of the sampled landing grids is below the
  danger here.
- `escape_phase_lost_danger_percent` (100): with test 0, the fight is lost
  when the danger where the Borg stands is at or above this percent of the
  current HP.
- With the rule on, the boost falls back to `borg_defend` (resistances,
  Protection from Evil, shields, staves), and #539's
  `escape_phase_adjacent_only` check is skipped (the rule decides).

| parameter | default | unit | range | source |
|---|---|---|---|---|
| `escape_phase_adjacent_rule` | 0 | flag | 0–1 | `borg_escape.rs:855` |
| `escape_phase_lost_test` | 1 | choice | 0–2 | `borg_escape.rs:936` |
| `escape_phase_lost_danger_percent` | 100 | percent of HP | 0–1000 | `borg_escape.rs:935` |

## Town less scary (#622)

The registry now holds **2156** parameters (one added). Owner, 2026-11-19:
"rather than having a flag 'town not scary' maybe let us have a
'town_less_scary' that decreases the scaryness level of town monsters".

`caution_town_scary_levels_less` (levels, compiled 0, **shipped 3**, owner 2026-11-19: "N=3 sounds good"): for
a town race (race level 0) on the data-file scary list (`borg_scary_list`,
shipped `scary_monsters.txt`), the listed level is lowered by this many
character levels (`clevel <= up_to - N`, `scary_list::is_scary`); the
compiled C list (`borg_scary_list = none`) is not changed. A level-0 race
is only ever generated in town (`mon-make.c`: "No town monsters in
dungeon"). The damage rule and the resist scares are not
changed. In the vendored core: it plays with the borg-rs core, the default
since #606.

On borg-zoo's 18 classes-18 starts (100,000 turns, main `38858846`, the zoo
adapter's steps). With #619's `attack_town_dangerous_on_sight` on top, from
a build of both branches:

| N | deaths | sum of CL | town scares | town <-> DL1 | thefts in town | dangerous town monsters killed |
|---|---:|---:|---:|---:|---:|---:|
| 0 (= main, 18/18) | 0 | 99 | 2,260 | 3,012 | 0 | 6 |
| 3 | 0 | 102 | 2,090 | 2,908 | 1 | 20 |
| 6 | 1 | 99 | 850 | 2,076 | 13 | 37 |
| 11 | 2 | 98 | 0 | 1,152 | 16 | 15 |
| 3, kill on sight | 0 | 101 | 2,058 | 2,884 | 1 | 53 |
| 6, kill on sight | 1 | 102 | 907 | 2,108 | 16 | 38 |
| 11, kill on sight | 3 | 90 | 0 | 1,138 | 11 | 52 |

All the deaths are early in the dungeon (wild cats, Grip, a floating eye,
CL1-4), none in town. Messages are placed in town when every `state.log` row
in force at their game turn says depth 0. Direction only: 18 games, no zoo
run.

| parameter | default | unit | range | source |
|---|---|---|---|---|
| `caution_town_scary_levels_less` | 0 | levels | 0–51 | `borg_caution.rs` (`town_scary_levels_less`, read in `borg_flow_kill.rs` `borg_near_monster_type` through `scary_list::Situation::levels_less`) |

## Flight Recall from a danger share (#587)

The registry now holds **2155** parameters (one added). `caution_flee_recall_danger_percent` (0, off): where `caution_flee_recall = 0` or `caution_flee_recall_gate_below_depth` holds a flight's Word of Recall back, the danger here at or above this percent of the current HP allows it, beside the breeder and chaser exceptions. Only the flight's read uses it; the stand tests ask whether an escape from the level exists and keep the old exceptions. At the shipped values (gate 1, off; `caution_flee_recall = 1`) nothing is held back, so it acts only with the gate on or `caution_flee_recall = 0`.

## Town never scary (#604)

The registry now holds **2154** parameters (2153 after #592; one added). Owner, 2026-11-18:
"Could we try a setting where we always fight monsters in town, I have never
got killed by monsters in town." All of it is in the vendored core, so it
plays with the borg-rs core (`borg_reborn_core = borg-rs`, shipped since #479,
or `BORG_CORE=borg-rs`; since #606 also a harness run without `BORG_CORE`);
the reborn core (`BORG_CORE=reborn`) has no scare and is not affected.

The measurement: 18 starts, the 9 classes x seeds 2001 and 2002, race 0,
100,000 game turns (not borg-zoo's classes-18 starts). Each was run as
`angband -mborgh -n -- -seed S -race 0 -class C -turns 100000 -keylog ...
-summary ... -menu-letters -exact-turns -statelog ... -msglog ...`, with the
shipped `harness/borg.txt` copied as the run's `user/borg.txt` (so the
borg-rs core) and `scary_monsters.txt` beside it; `BORG_CORE` unset.
Counted from the logs:
- **town scares, attacks, thefts:** messages placed at depth 0 by the
  `# Loc:... Dep:N` notes around them: counted when the previous and the
  next position note both say town, and "at a level change" when they
  disagree. The Borg can arrive on DL1 and leave again within one game turn
  without a position note, so a DL1 message can sit between two town notes:
  the remaining "town scares" with the switch on are of that kind (the ones
  checked by hand were DL1 arrivals), and the code sets no scare in town;
- **town <-> DL1 moves:** `state.log` depth changes between 0 and 1;
- **game turns in town:** the sum, over `state.log` rows at depth 0, of the
  turn-column gap to the next row;

On main `ed3ffc04` the town was scary at least 697 times (up to 2,740 with
the notes at a level change). The monsters known in town when a scare was
noted are, most often first, five town residents on the scary list: the
battle-scarred veteran, the mean-looking mercenary, the filthy street
urchin, the squint-eyed rogue and Farmer Maggot (who never attacks). Each
scare sends the Borg down to DL1, from where "finish shopping" brings it
back up, so it shuttled between town and DL1 2,831 times.

`caution_town_never_scary` (compiled 0, written out at 0): in town no
monster makes the level scary; the danger rules still apply.

| on the 18 starts | main | switch on |
|---|---:|---:|
| deaths | 3 | 2 |
| sum of character levels | 93 | 105 |
| sum of max depth | 35 | 39 |
| town scares | 697 (+2,043 at a level change) | 0 by construction (4 + 398 at a level change; see below) |
| town <-> DL1 moves | 2,831 | 1,286 |
| game turns in town | 484,073 | 298,362 |
| attacks from town monsters | 47 (+42 at a level change) | 73 (+30) |
| thefts in town | 0 (+1 at a level change) | 0 (+2) |
| deaths in town | 0 | 0 |

With the switch at 0 the 18 games are identical to main. Direction only: 18
games, no zoo run. The review of #605 reran borg-zoo's classes-18 starts with
`BORG_CORE=borg-rs`: the same directions (town scares 3,215 to 0, town <->
DL1 moves 3,012 to 1,151, no death in town), with 0 deaths on main and 2 with
the switch, the sum of CL 99 against 97.

The switch covers every scare decided in `borg_near_monster_type`. The door
scare ("You hear a door burst open!", `borg_messages.rs`) is not gated by it;
the town has no doors (its layout places only floor, walls and shop
entrances), so it never fires there.

| parameter | default | unit | range | source |
|---|---|---|---|---|
| `caution_town_never_scary` | 0 | flag | 0–1 | `borg_caution.rs` (`town_never_scary`, read in `borg_flow_kill.rs` `borg_near_monster_type`) |

## The scary flag clears after the kill (#590)

The registry now holds **2153** parameters (one added). `caution_scary_unlatch_kill` (0, off): with `caution_scary_stand`, the level's scary flag clears once no scary monster is known and the last one known was killed, with the latch on or off; forgetting the monster is not enough. The scan records the races that made the level scary (a bit per race, no cap), a death message for one of them (both passes) marks the kill, and a scan that still finds a scary monster known spends an earlier kill.

## The twitch and the parked staircase (#597)

The registry now holds **2152** parameters (two added). priest-809881907 died in lava in town: the only known staircase had a monster marked on it, so every stair flow had no goal and the Borg twitched until a random step went into lava (100 + 1d100 fire damage).
- `caution_twitch_avoid_lava` (compiled 0, shipped 1): the twitch refuses lava without fire immunity (`lava_forbids_step`, #458's rule).
- `caution_stair_parked_fallback` (compiled 0, shipped 1): when no staircase entered a stair flow and a monster is marked on known ones, the flow takes those. A staircase the flow refuses as icky or dangerous does not count as entered. In `borg_flow_stair_both` the decision is across both kinds.

## A fleeing Druid takes Fox Form (#551)

The registry now holds **2150** parameters (2145 before; five added). Owner,
2026-11-18: Druids die most (borg-zoo#227), and Fox Form, a level-1 spell,
is the strategy manual's CL1-20 escape (`14-class-playbooks.md:453`). The
fox shape has `MOVES[1]` (each step costs half the energy, `player-util.c`
`energy_per_move`) and Free Action (`shape.txt`). The owner's rules: use
the new speed, make the step threshold a parameter, and once a fox, escape,
or change back with an action before fighting.

With `escape_fox_form = 1`:
- **The cast:** a Druid about to take a flight step (`borg_flow_old` with
  `GOAL_FLEE`) at least `escape_fox_form_min_steps` flow steps from its goal
  casts Fox Form first, if its fail rate is at most `escape_fox_form_fail`.
  Note: `# Fox Form: fleeing (N steps to go)`.
- **Keeping the shape:** `borg_think` keeps it in the dungeon (the game's
  depth) while the Borg flees (the flight flow, or the flee or leave goal)
  or the level is scary, standing included. Otherwise the Borg changes back.
  Keeping it on a scary level stops a flip-flop with #530's stand mode (110
  casts in one game without it).
- **A fox does not fight:** before a melee blow, `borg_attack` (`BF_THRUST`)
  or a flow step into a known monster (`borg_play_step`), the Borg changes
  back (no energy) and fights in its own shape from the next think. Note:
  `# Fox Form: reverting to fight`. Every other attack is an item or a spell
  command. For those the game asks "Change back and <verb> (y/n) or
  (r)eturn to normal?" (`player_get_resume_normal_shape`), and the Borg
  answers `y`: it changes back and acts in one command, with no extra
  energy (`# Fox Form: reverting for an item or a spell`). Its own change
  back sends `v` and answers `r` (`fox_revert`).
- **The speed:** `escape_fox_form_speed` (10: halving a step's energy is +10
  at normal speed, `extract_energy` 10 to 20) is added to `borg_danger`'s
  speed while the Borg is a fox (a fox only moves), and to the player speed
  of #530's stand decision while it is a fox. A Druid that is not yet a fox
  but can cast Fox Form (it will at its first flight step) gets
  `escape_fox_form_stand_speed` (10) there instead. That bonus is granted on
  the ability, so it has a parameter of its own (review of #594). More speed
  means fewer pursuers, so fewer stands and more flights. The flat +10 is
  exact only at speed 110 (normal). At speed S a fox has
  `2 × extract_energy[S]`, which is worth +2 at 100, +7 at 105, +15 at 115,
  +24 at 120 (review of #594).
- **The change-back flag:** `fox_revert` lives as long as its queued `v`. It
  is consumed by the prompt and cleared by `borg_flush` (review of #594:
  otherwise a flushed `v` left it set, and the next item command was
  answered `r`, so the Borg changed back without acting).

Compiled and shipped 0 (C). Evidence, a lead (branch of main `8e7dbad4`,
100,000 turns): the 18 classes-18 starts and 27 Druid starts from
classes-1800 (29 Druids in all):

| arm | Druid deaths | Druid sum of CL | Druid sum of max depth | other classes |
|---|---:|---:|---:|---|
| off | 3 | 161 | 54 | |
| on | 3 | 156 | 56 | 16/16 keys identical |

With `escape_fox_form_stand_speed = 0`, 1 game of 29 changes: 36
stands against 35, and the same deaths, sum of CL and depth.

On, the Druids cast Fox Form 431 times in all. They changed back for an
item or a spell 120 times, and to fight 67 times. A fox's step takes 5 game
turns, not 10. Seven blows were still struck as a fox, all steps into an
unseen monster ("You claw something"): the game turns the step into an
attack the Borg did not know of. The 3 deaths are the same in both arms, at
CL1-2 and before any flight, so these games say nothing about survival: that
needs the zoo (Druid starts).

| parameter | default | unit | range | source |
|---|---|---|---|---|
| `escape_fox_form` | 0 | flag | 0–1 | `borg_escape.rs:493` |
| `escape_fox_form_fail` | 25 | percent | 0–100 | `borg_escape.rs:531` |
| `escape_fox_form_min_steps` | 3 | steps of flow | 1–250 | `borg_escape.rs:626` |
| `escape_fox_form_speed` | 10 | speed | 0–30 | `borg_escape.rs:682` |
| `escape_fox_form_stand_speed` | 10 | speed | 0–30 | `borg_escape.rs:684` |

## Breeders hunted by every class (#558)

The registry now holds **2139** parameters (2137 on main before #558; two
added). Owner, 2026-11-18, from `~/borg-play/priest-1055640847`: a CL6 Priest
with 70 HP never killed a green worm mass it could solo. The breeder hunt
(#503) now takes every class: `caution_breeder_hunt_max_count` decides, with
the common 4 for every class (#561 shipped Warrior 6 and Priest 3; #574,
owner, removed them until borg-zoo#277 is read; a class value of 0 turns a
class's hunt off).
The count is the switch for the new classes (owner): there is no separate
flag, so the expansion plays even with the two switches below at 0. The
configuration that plays as main is the count at 0 for Mage, Druid, Priest,
Necromancer and Ranger and 4 for Warrior, with both switches at 0.
Low-level Mages and Necromancers: the hunt walk (`borg_flow_kill`) targets
breeders only, and only while they have half their mana, so they cast rather
than melee (`borg_flow_kill_direct`, the dig to the closest monster after
3000 turns on a level, is unchanged and has no such filter). All of it is in
the vendored core, so it plays with the borg-rs core (`borg_reborn_core =
borg-rs`, shipped since #479, or `BORG_CORE=borg-rs`); the reborn core
(`BORG_CORE=reborn`) is unchanged. Two new
switches, compiled 0, shipped 1 and 50:

| parameter | default | unit | range | source |
|---|---|---|---|---|
| `recover_breeder_hunt_hp_percent` | 0 | percent | 0–100 | `borg_recover.rs` (`hunt_before_rest`, `borg_flow_kill::breeder_hunt_now`: when a rest is due, no rest while HP is at least this percent of max HP and the Borg is fit to hunt now: the hunt applies, the Borg is not ignoring the breeders, fleeing or leaving, no scary monster is on the level, a low-level caster has half its mana, it is not hungry, confused, blind, hallucinating, afraid or stunned (a rest clears those), and a known breeder passes the walk's danger test (only that test: not the walk's leash to the stairs or reachability); breeders multiply while the Borg rests. The HP rest is skipped only when it is due and those hold; the mana rest is skipped whenever they hold. For acid breeders it depends on the next switch: with the acid fear a green worm mass reads 507 against a limit of 23 at CL6, so the gate never opens) |
| `danger_hunt_breeder_no_acid_fear` | 0 | flag | 0–1 | `borg_danger.rs` (`hunted_breeder`, `borg_caution::breeder_engaged`: while the hunt applies and the Borg is not ignoring the breeders, fleeing, leaving or facing a scary monster, every breeder on the level is judged without `danger_monblow_acid_fear`, the flat 200 per acid blow that put a green worm mass's 1d3 bite at 630. Level-wide, not per monster: the regional fear map and retreat grids see it too) |

## A hurt or afraid monster is not scary (#539)

The registry now holds **2137** parameters (2135 after #562; two added).
Owner, 2026-11-18, from the zoo death `priest-809881907` (Grip): "The dog
stopped being scary as soon as I threw to him a flask of oil, can it stop
being scary when it is low in hp?"

In `borg_near_monster_type`, a monster that has lost at least
`caution_scary_hurt_percent` of its HP (`kill.injury`, from the health bar),
or that is afraid (`kill.afraid`, with `caution_scary_afraid`), is no
named-list or damage-rule scare. Resist-based scares and door scares are not
affected. A calmed monster still counts as a pursuer in stand mode
(`caution_scary_stand`) when it is awake and at least as fast as the
character. Without that, a monster turning afraid while other scary
monsters slept made a stand into a flight from the level: a first version
had two deaths on the 18 starts (a cave spider, a poisoning). With
`caution_scary_latch = 1` (shipped) a calmed monster only keeps a level from
becoming scary; with 0 the level's scare also ends once every scary monster
is calmed, dead or forgotten.

Compiled 0 and 0 (C); `harness/borg.txt` ships 50 and 1. On the 18
classes-18 starts at 100,000 turns, branch of main `162f96af` (a lead, 18
games show direction only):

| arm | deaths | sum of CL | sum of max depth | keys as off |
|---|---:|---:|---:|---:|
| off (0, 0; latch 1) | 0 | 107 | 43 | |
| shipped (50, 1; latch 1) | 0 | 107 | 43 | 17/18 |
| latch 0 alone | 0 | 100 | 39 | 1/18 |
| 50, 1 and latch 0 | 0 | 104 | 41 | 1/18 |

The one shipped game that changes is Ranger 1739161339: Grip appears with 4
HP and the Borg shoots it instead of fleeing the level.

With the owner's `borg.txt` of main `fba1c6e0` (`borg_trip_unknown_stacks
= 2`, `caution_stair_scum_threat = 1`), merged into this branch: off and
shipped play identical keys in 18/18 games. In both arms, 1 death
(Blackguard 1472012981, a cave spider), sum of CL 101 and max depth 38. The
death is the new `borg.txt`'s, not this change's: it comes from
`borg_trip_unknown_stacks = 2` (with 5 the game lives).

| parameter | default | unit | range | source |
|---|---|---|---|---|
| `caution_scary_hurt_percent` | 0 | percent of HP | 0–100 | `borg_caution.rs:3792` |
| `caution_scary_afraid` | 0 | flag | 0–1 | `borg_caution.rs:3792` |

## Fear cures and the see-invisible clock (#516)

The registry now holds **2134** parameters. Owner, 2026-11-17 (purchase audit
of #513). On 18 classes-18 starts on main `aa9ebf49`:
- **Boldness:** 35 bought, 7 drunk. The planner values a Potion of
  Boldness as a fear cure on the same diminishing curve as the confusion
  and blindness cures. With `town_fear_cure_count` > 0 and the character
  not afraid, fear cures are valued linearly up to that many units (one per
  potion while not afraid) and not beyond. Afraid, the cure demand is as
  before. The cap does not make a cure cheaper: with the cap at 2, one
  potion is worth half the support weight and two all of it, far more than
  on the curve (about 495 each), and a third is worth nothing.
- **Detect Invisible:** 187 reads, 57 in town, 171 with no unseen monster
  on the level. After every rest `borg_recover` (C's `borg-recover.c:385`,
  "reset the inviso clock to avoid loops") stamps `need_see_invis` with the
  current time, which `borg_defend_aux_inviso` reads as "hit by an unseen
  monster recently", so every rest armed the defence. With
  `recover_rest_arms_see_invis = 0` a rest leaves the clock alone; only an
  unseen attack (`borg_locate_kill`) arms it. The same clock has two other readers,
  which read it the other way round: the Light Beam defence
  (`borg_defend_aux_lbeam`, skipped while the stamp is recent) and the
  removal of regional fear when a monster appears (`borg_flow_kill`, kept
  while recent). At 0, both can now act after a rest where C did not.

Compiled defaults are C's (0 and 1); `harness/borg.txt` ships 2 and 0. On
the same 18 starts: Detect Invisible read 95 times (0 in town) and bought
81 (1,245 gold against 1,830); Boldness bought 8 (64 gold against 280) and
drunk 10; no deaths in either; summed level 107 against 95, depth 43
against 38 (18 games: direction only).

| parameter | default | unit | range | source |
|---|---|---|---|---|
| `town_fear_cure_count` | 0 | count | 0–100 | `borg_town_capability.rs:649` |
| `recover_rest_arms_see_invis` | 1 | flag | 0–1 | `borg_recover.rs:450` |


## A living monster keeps some hit points (#546)

The registry now holds **2123** parameters (2122 after #544, which added three; one added). Owner,
2026-11-18, from the floating-eye Mage 2028599180, which stopped attacking an
eye it had left at 1/9 HP and walked next to it: the HP estimate truncated to
0. Compiled 0 (C's estimate); `harness/borg.txt` ships 1.

| parameter | default | unit | range | source |
|---|---|---|---|---|
| `kill_live_min_power` | 0 | hp | 0–10 | `borg_flow_kill.rs` (`set_health`/`live_power`: a monster with hp above 0 is estimated at least this many HP; the attack overkill caps use the estimate) |

## Fleeing: stairs, range, no pointless Phase Door (#539)

The registry now holds **2119** parameters (2116 after #520; three added). Owner,
2026-11-18, from the Priest 809881907 (borg-zoo#227), who read its only Word of
Recall with the up staircase 8 steps away, rested while Grip closed in, phased
twice from a dog twice its speed and died meleeing with 17 flasks of oil: "use
recall only if a stair is not reachable at all", "do not phase door when a
monster is still distant", and with no stairs "the best available ranged
attack" before Recall and Phase Door. Compiled 0 (C's behaviour); `harness/borg.txt`
ships all three at 1.

| parameter | default | unit | range | source |
|---|---|---|---|---|
| `caution_flee_stairs_before_recall` | 0 | flag | 0–1 | `borg_caution.rs` (`stairs_before_recall`: no Recall while the flee flows' stair flow reaches a free staircase) |
| `escape_phase_adjacent_only` | 0 | flag | 0–1 | `borg_escape.rs` (`phase_is_pointless`: a Phase Door only with an awake monster adjacent, or one in view with a ranged attack) |
| `caution_scary_ranged_no_stairs` | 0 | flag | 0–1 | `borg_caution.rs` (`scary_ranged_no_stairs`: no staircase reachable, a scary level lets `borg_attack` target an awake monster in view at range; Recall and Phase Door wait while there is one and the attack planner has a missile or a throwable object) |

## Class values and class files (#520)

The registry now holds **2116** parameters (2191 before; 81 per-class
parameters folded into 6 attack-preference families). Owner, 2026-11-17: one
common `borg.txt`, per-class files that can override any setting, and no
per-class parameter families in the code. `params::value` returns the current
character's class value when it has one (`set_class`, from
`params::initialize` and `borg_prepare_race_class_info`), else the common one.
The families `attack_preference_<class>_<family>_permille` (54),
`attack_arrow_point_blank_divisor_<class>`, `town_heal_curve_scale_percent_<class>`
and `town_heal_reserve_hp_percent_<class>` (9 each) are gone from the registry;
their names remain as aliases of the class values (docs/ZOO-INTERFACE.md,
"Class values"). New: `attack_preference_melee_permille`, `_projectile_`,
`_thrown_`, `_spell_`, `_device_`, `_activation_` (1000, 500–2000,
`borg_fight_attack.rs:4575`). The shipped class values live in
`harness/borg_<class>.txt`, folded into `harness/borg.txt` by
`scripts/borg-class-files.py`.

## M13 inventory gap closure (#433 / P4)

The synchronized registry contains **2,191** parameters. This adds 362 neutral
inventory parameters to the 1,829-entry accepted main registry at `a8ac685a`: 48 power,
23 wear, 14 buy, 34 sell, 6 store-choice and 237 home-valuation entries.
The [M13 inventory contract](M13-INVENTORY-PARAMETERS.md) records scope,
units, bounds, retained mechanics, #293 separation and the P-before-R4 gate;
its JSON companion lists every new default and original source location.
The extraction changes no default decision policy. Historical counts below
remain receipts of their own snapshots.

## Class/family attack preference extension (#403)

The historical P1–P3 counts above describe the original extraction. #403 adds
54 neutral-default parameters to the current M13 registry. The full arithmetic,
family inventory, configuration and trace@2 contract is in
[ATTACK-PREFERENCES.md](ATTACK-PREFERENCES.md). `utility_permille` means a
multiplier of returned utility with neutral 1000, bounds 500..2000; it is distinct
from the existing device-failure `permille` unit. No existing default changes.

## Gold reserve (#493)

The registry now holds **1797** parameters (1795 after #485's follow-up; two added). Before,
the live town planner kept gold only through the soft price
`borg_town_w_gold` (`borg_town_solve::solve_cash`: spending `c` costs
`w x c / (gold left + 1)`), which made the last coins nearly unspendable. The
reserve is now explicit: `Budget::of` (`borg_town_budget.rs`,
`build_with_reserve`) takes the larger of the two below from the gold left
after the urgent needs, which may still use the whole purse. The reserve
bounds the purse at arrival: the proceeds of the visit's sales join the gold
at the purchases stage (`borg_town_record`, `limits.gold + freed.gold`) and
are spendable on top of it. The shipped `harness/borg.txt` sets both to 0 and
`borg_town_w_gold = 0` (owner, 2026-11-17), and the compiled
`borg_town_w_gold` default is 0 too (`borg_init.rs`, a declared deviation from
borg-rs's C table, `OUR_DEFAULTS`; borg-rs's pristine `borg-core/borg.txt`
keeps 25). The soft price came from borg-rs `785f5064` (2026-11-14,
`docs/TOWN-UTILITY.md` "Retaining gold"), vendored with borg-rs #87.

| parameter | default | unit | range | source |
|---|---|---|---|---|
| `town_gold_reserve` | 0 | gold | 0–10000000 | `borg_town_budget.rs:250` |
| `town_gold_reserve_percent` | 0 | percent of the purse | 0–100 | `borg_town_budget.rs:251` |

## Point-blank divisor by class (#485)

The registry now holds **1795** parameters (1786 after #491; nine added). The
damage of a shot or throw at an adjacent non-unique is divided by
`attack_arrow_point_blank_divisor` (compiled 5, borg-rs's); since #485 a class
overrides it with `attack_arrow_point_blank_divisor_<class>` (0, the default,
means the general divisor), resolved by `borg_fight_attack::point_blank_divisor`.
The shipped `harness/borg.txt` sets the general divisor to 5, the Ranger's to
2 and the Rogue's to 3 (owner, 2026-11-17: 2 suits Rangers but not Rogues, the
second missile class); #495 had shipped 2 for every class.

| parameter | default | unit | range | source |
|---|---|---|---|---|
| `attack_arrow_point_blank_divisor_<class>` (warrior, mage, druid, priest, necromancer, paladin, rogue, ranger, blackguard) | 0 (= `attack_arrow_point_blank_divisor`) | divisor override (0 = the general divisor) | 0–1000 | `borg_fight_attack.rs:3009` |

## Healing reserve (#491)

The registry now holds **1786** parameters (1783 after #483; three added). Owner
(2026-11-17): "we can never have enough healing", as configurable values, and
"budget healing speed as well". All act on the live town planner's healing curve
(`borg_town_capability::observe`), where carrying `x` HP of healing was worth
`w x x / (x + d)` with `w = borg_town_w_bleeding x exposure` and
`d = max HP x exposure`, so each potion was worth less than the one before.

| parameter | default | unit | range | source |
|---|---|---|---|---|
| `town_heal_reserve_hp_percent` | 0 (off) | percent of max HP | 0–10000 | `borg_town_capability.rs:454` |
| `town_heal_speed_percent` | 0 (HP alone) | percent of max HP | 0–100 | `borg_town_capability.rs:309` |
| `town_heal_curve_scale_percent` | 100 (historical) | percent | 1–10000 | `borg_town_capability.rs:445` |

- **Reserve:** a new capability row, "healing reserve" (id 58), fed by the
  same HP as healing (`EF_HEAL_HP`, capped at max HP). Up to the reserve,
  each HP is worth the curve's first HP, `w / d = borg_town_w_bleeding /
  max HP`, independent of the trip's exposure, on top of the curve. It is a
  requirement (negative demand, like food's floor), so the planner buys it
  first and never sells below it; past it only the curve remains.
- **Curve scale:** multiplies `w` and `d` together. The first HP keeps its
  value; the curve reaches half its maximum at `scale x d` instead of `d`.
  Its maximum, the most healing can ever be worth, is multiplied by
  `scale / 100` too, so the scale changes the total value as well as the
  shape (#496 review F4).
- **Healing speed:** the HP one HEAL_HP use supplies, to the curve and the
  reserve alike, are weighted by how fast it brings the character back to
  max HP. A use that heals at least `town_heal_speed_percent` of max HP
  counts in full; a smaller one counts `heal x heal / (percent x max HP)`.
  At 66, Cure Light Wounds (20 HP) counts in full up to 30 max HP. The
  planner's best healing per gold, by the character's max HP: CLW up to
  about 30 max HP, Cure Serious Wounds (40 HP) at 60–100 max HP, Healing
  (300 HP) from about 200 max HP. With HP alone it is CLW at every max HP.
  Cure Critical Wounds (60 HP for 200 gold) never wins on healing alone
  (`engine/angband/lib/gamedata/object.txt`: CLW 20 HP / 20 gold, CSW
  40 / 70, CCW 60 / 200, Healing 300+m35 / 800).
  The factor also shrinks each potion's HP on the curve while the curve's
  demand stays in max-HP units: each weak potion is worth less on the curve,
  and more of them are needed to reach half its value. So speed changes the
  curve's shape as well as the ranking of potions (#496 review F4).
- The shipped `harness/borg.txt` sets 400, 100 and 66 (owner, #491;
  measurement in #491's entry of docs/HANDOVER.md, the archive). At 0, 100 and 0 play is byte-identical
  to before.

## Rising to town explores for an up staircase (#483)

The registry now holds **1783** parameters (1781 before; two added). While
the Borg is rising to town (`goal.rising`: a trip or town return), a descent
that is not a desperate escape is held, so that it explores the level for an
up staircase. Before, with connected stairs, it arrived on a down staircase,
took it (the level was "scary", or no up staircase was known), climbed back
onto another down staircase and never reached town (borg-zoo#170 run
44054277431, Ranger 1739161339: DL2/DL3 to the end). The gate is
`borg_caution::rising_holds_descent`, applied in `choose_stairs`, in the flee
block's flow to a down staircase and in `borg_leave_level`'s "no up stairs
found, going down", and since the review every descent `borg_leave_level` decides
(its "Go Down" block: bored, "spent too long on level", the trip policy).

The budget counts Borg ticks, the same clock as "Spent too long on level";
how many game turns a tick spans depends on what the Borg does (a move, a
rest, a search). It is per level, not per return: it bounds the search for an
up staircase on one level.

| parameter | default | unit | range | source |
|---|---|---|---|---|
| `caution_rising_explore_ticks` | 300 | Borg ticks (one per think, `borg_think`; restarted on each new level) | 0–100000 | `borg_caution.rs:4178` (0 = off, C's behaviour) |
| `caution_rising_descend_danger_tenths` | 10 | tenths | 0–1000 | `borg_caution.rs:4183` (danger here above avoidance x value / 10 lets the descent through) |

## Cornered mode (#504)

The registry now holds **1809** parameters (1803 after #505's scary damage, #514; six added). Owner, 2026-11-17 (#488 slice B). `borg_defend`'s buffs fire only in "some
danger but not much" (10–50% of avoidance), so a Borg that is losing a fight
it cannot leave used none: a Blackguard died at 1/41 HP with its Berserk
Strength spell unused. With `caution_cornered`, cornered means: in the
dungeon, no Phase Door, teleport or escape (in the pack or as a spell), a
known monster within the radius, and danger here at or above the HP x tenths /
10. Then, before any retreat, in order and each while not active: a fear cure
when afraid, Speed, Heroism, Berserk Strength, Blessing, and Monster Confusion
against an adjacent unique or higher-level monster that is not immune to confusion.
Buff spells (Haste Self for Druids and Rangers, Heroism, Berserk Strength,
Bless) are cast only at or below `caution_cornered_spell_fail` (25) percent
failure. A melee class (Warrior,
Paladin, Rogue, Blackguard) neither retreats nor backs away. The ordinary heal tiers run earlier in `borg_caution`; #457's outmatched
heal stays at the end, after retreat and back-away (tried first, it kept a
cornered Borg drinking in place where a step away saved it). A monster that
never moves (a mold) does not corner the Borg. Teleport Level counts as an escape; Word of Recall does not (unlike
#503's easy escapes: 15–35 turns is too slow for a fight already lost). Monster Confusion is off the town
planner's worthless list while the mode is on. The compiled defaults leave it
off; `harness/borg.txt` ships 1, 10, 2 and 0. Owner: the same boosts in a
hard breeder fight, `caution_breeder_boost_danger_tenths` (compiled 0 = off,
shipped 5): while standing against breeders with no easy escape (#503's
`breeder_stand`) and danger here at or above half the HP. Blessing is the
Bless spell, a Scroll of Holy Chant or a Scroll of Blessing.

| parameter | default | unit | range | source |
|---|---|---|---|---|
| `caution_cornered` | 0 | flag | 0–1 | `borg_caution.rs:3956` (1 = on) |
| `caution_cornered_danger_tenths` | 10 | tenths | 0–1000 | `borg_caution.rs:3957` |
| `caution_cornered_radius` | 2 | grids | 0–100 | `borg_caution.rs:3959` |
| `caution_cornered_big_level_margin` | 0 | levels | 0–100 | `borg_caution.rs:4016` |
| `caution_cornered_spell_fail` | 25 | percent | 0–100 | `borg_caution.rs:4080` (buff spells only at or below this failure rate) |
| `caution_breeder_boost_danger_tenths` | 0 | tenths | 0–1000 | `borg_caution.rs:4004` (0 = off; owner: the same boosts in a hard breeder fight) |

## Healing by class; no Shovel while poor (#487)

The registry now holds **1829** parameters (1809 after #504; twenty added).
Owner, 2026-11-17, from #487's analysis (no melee class bought a weapon; the
gold went to dozens of Cure Light Wounds for #491's 400% reserve and to a
Shovel at the first visit):

- `town_heal_curve_scale_percent_<class>` (`borg_town_capability.rs:368`,
  0 = `town_heal_curve_scale_percent`) and `town_heal_reserve_hp_percent_<class>`
  (`borg_town_capability.rs:387`, -1 = `town_heal_reserve_hp_percent`, 0 = no
  reserve), for warrior, mage, druid, priest, necromancer, paladin, rogue,
  ranger and blackguard. Unit `percent_override`; compiled defaults fall back
  to the general values.
- `town_digger_min_gold` (`borg_town_capability.rs:635`, 0 = off): below this
  gold the live planner values no digging. A Shovel only clears rubble (the
  engine swaps it in, "with your swap digger", 2–6 piles a game); a weapon
  digs rubble too.

- `town_heal_reserve_real_hp` (`borg_town_capability.rs:239`, 0 = #491 as
  first shipped): the healing reserve counts the real HP a potion restores.
  `town_heal_speed_percent` then weights only the healing curve (which kind is
  worth more), not how many potions fill the reserve. Speed-weighted, a Cure
  Light Wounds counted about 5.6 HP at 108 max HP and the reserve took dozens.

Shipped in `harness/borg.txt`: the general reserve 100 (was 400) and the
general curve scale 125 (owner); the reserve counts real HP; scale
Warrior 50, Paladin and Blackguard 75, Rogue and Ranger 100, Priest and Druid
125, Necromancer and Mage 150; reserve Warrior 100, Paladin and Blackguard
150, Rogue and Ranger 200, Priest and Druid 250, Necromancer and Mage 300;
digger 500 gold.

| parameter | default | unit | range | source |
|---|---|---|---|---|
| `town_heal_curve_scale_percent_<class>` (nine) | 0 (= `town_heal_curve_scale_percent`) | percent_override | 0–10000 | `borg_town_capability.rs:368` |
| `town_heal_reserve_hp_percent_<class>` (nine) | -1 (= `town_heal_reserve_hp_percent`) | percent_override | -1–10000 | `borg_town_capability.rs:387` |
| `town_heal_reserve_real_hp` | 0 | flag | 0–1 | `borg_town_capability.rs:345` |
| `town_digger_min_gold` | 0 | gold | 0–10000000 | `borg_town_capability.rs:635` |
| `town_missiles_percent` | -1 | percent_override | -1–1000 | `borg_town_capability.rs:407` |
| `town_missiles_other_ranged_percent` | 100 | percent | 0–100 | `borg_town_capability.rs:657` |
| `town_devices_percent` | 100 | percent | 0–1000 | `borg_town_capability.rs:562` |
| `town_recall_reserve` | 0 | count | 0–5 | `borg_town_capability.rs:694` (#557: this many Word of Recall are a requirement at the full trip weight each, at any depth; also in `borg_town_utility.rs`'s need model) |
| `town_recall_value_percent` | 100 | percent | 0–10000 | `borg_town_capability.rs:1371` (#557: a Word of Recall's value in percent of the walk it saves, in the capability and need models) |
| `prepared_recall_depth` | 5 | depth | 0–127 | `borg_prepared.rs:202` (#557: C's "1 recall" for a descent from this depth; 0 = never; gates descents only) |

## Breeders and named scary monsters (#503)

The registry now holds **1802** parameters (1797 after #493; five added). Owner, 2026-11-17 (#488 slice A). The C Borg flees breeders at low level three
ways (the breeder flee of `borg_caution`, the breeder scary trigger of
`borg_near_monster_type`, the "no recall" flee of `borg_check_breeders`), backs
away from their danger, and below clevel 10 never flows to them. With
`caution_breeder_fight` and no easy escape (a free staircase within the steps,
by the engine's `distance`; a Phase Door, teleport, Teleport Level, escape or
Word of Recall) it stands and fights: none of the three flees, and no retreat
or back-away while a breeder is within the radius and no other known monster
is within the danger horizon (`danger_monster_max_distance`, on the larger axis
distance like `borg_danger`); being surrounded
still moves it. The generic low-HP and bleeding/poison flees stay (a Borg
losing the fight is #504's cornered mode). With the named list on
(`caution_scary_named_monsters = 1`) a level made scary by breeders before the
stand engaged (while the Borg still had an escape) stays scary until a new
level: the latch is a known limit of that intermediate arm. Warriors, Paladins,
Rogues and Blackguards hunt breeders while at most the hunt count are known.

`caution_scary_named_monsters` turns off the C Borg's hard-coded scary monsters
by character level (squint-eyed rogue; Grip, Fang, small kobold; soldier,
cutpurse, acolyte, apprentice, kobold, shrieker, Farmer Maggot, street urchin,
veteran, mercenary; Bullroarer and the breeders; cave spider, red naga, giant
red frog, radiation eye; gravity and inertia hounds, ancient dragons, Beorn,
Dread) and the low-level door scares. A scary level stays scary until a new
level: one small kobold seen once kept a Blackguard fleeing for 54,000 game
turns. The resist-based scares (nether, blindness, chaos, confusion, poison)
stay. The compiled defaults are C's; `harness/borg.txt` ships 1, 5, 5, 4 and,
since the owner's decision after borg-zoo#216 (with the breeder stand on: 1
death with the list, 4 without), 1.

| parameter | default | unit | range | source |
|---|---|---|---|---|
| `caution_breeder_fight` | 0 | flag | 0–1 | `borg_caution.rs:2771` (1 = stand and fight breeders without an easy escape, melee classes hunt them) |
| `caution_breeder_easy_escape_steps` | 5 | grids | 0–100 | `borg_caution.rs:2772` |
| `caution_breeder_stand_radius` | 5 | grids | 0–100 | `borg_caution.rs:3133` |
| `caution_breeder_hunt_max_count` | 4 | count | 0–1000 | `borg_caution.rs:3320` |
| `caution_breeder_exit` | 0 | flag | 0–1 | `borg_caution.rs:3147` (#544 B: 1 = only breeders near and outmatched or surrounded, walk to a known chokepoint (corridor or doorway: its passable neighbours form two or more groups) away from every breeder) |
| `caution_breeder_exit_distance` | 6 | grids | 1–100 | `borg_caution.rs:3150` (#544 B: the exit's distance from every breeder) |
| `caution_flee_known_down_stairs` | 0 | flag | 0–1 | `borg_caution.rs:1043` (#544 A, narrowed in #548: 1 = fleeing with no up staircase known, a known down staircase, only cornered and within the cost cap below) |
| `caution_flee_known_down_cornered` | 1 | flag | 0–1 | `borg_caution.rs:1048` (#548: 1 = only cornered: a moving monster adjacent (distance 1; #504 uses `caution_cornered_radius`), no Phase Door, teleport, Teleport Level or escape item in the pack (spells not counted), danger here >= HP or surrounded; 0 = any flight, #545 as merged) |
| `caution_flee_known_down_max_cost` | 40 | steps of flow | 0–250 | `borg_caution.rs:1049` (#548: the staircase within this many steps of flow, the walk around walls, not the straight-line distance; 0 = any distance) |
| `caution_jumpstart_keep_stairs` | 0 | flag | 0–1 | `borg_caution.rs:4566` (#548, #544's K: 1 = the stuck jumpstart in `borg_think_dungeon.rs` keeps the known stairs; usually wanted with `caution_flee_known_down_stairs`) |
| `caution_twitch_avoid_lava` | 0 (shipped 1) | flag | 0–1 | `borg_caution.rs:2979` (#597: the twitch never steps into lava without fire immunity) |
| `caution_stair_parked_fallback` | 0 (shipped 1) | flag | 0–1 | `borg_caution.rs:2986` (#597: a monster on every known staircase no longer hides them from the stair flows) |
| `caution_stair_scum_threat` | 0 | flag | 0–1 | `borg_caution.rs:3624` (#549: 1 = after fleeing town, no DL1 stair scum while threatened: scary level, danger here, or hurt; the HP condition is the broad one: any injury stops the scum, and the danger limit only adds grids that are dangerous at full HP. A threat also cancels the scum's own `GOAL_TAKE` flow) |
| `caution_stair_scum_danger` | 0 | danger | 0–10000 | `borg_caution.rs:3628` (#549: the danger here above which the stair scum stops) |
| `caution_rising_hold_flee_descent` | 0 | flag | 0–1 | `borg_caution.rs:3655` (#557: 1 = while rising in the dungeon, no flight down a staircase unless danger here >= HP: `choose_stairs`, `borg_flow_stair_both`, `borg_flow_stair_more(GOAL_FLEE)`; the held staircase is no easy escape in #530's stand) |
| `caution_flee_recall` | 1 (shipped 1; borg-zoo#255) | flag | 0–1 | `borg_caution.rs:2658` (#557: 0 = a flight reads Word of Recall only on a breeder-swamped level or when chased; Recall then is no easy escape in the stand tests) |
| `caution_flee_recall_breeders` | 5 | monsters | 0–100 | `borg_caution.rs:2694` |
| `caution_flee_recall_chasers` | 3 | monsters | 0–100 | `borg_caution.rs:2695` |
| `caution_flee_recall_chase_distance` | 6 | grids | 1–30 | `borg_caution.rs:2678` |
| `caution_flee_recall_gate_below_depth` | 0 (shipped 1, i.e. off) | dungeon level | 0–127 | `borg_caution.rs:2660` (#557: a ceiling; with `caution_flee_recall` 1, at depths shallower than this a flight's Recall needs the breeder or chaser exception; 2 = DL1 only; 0 or 1 = off) |
| `caution_flee_recall_danger_percent` | 0 | percent of HP | 0–10000 | `borg_caution.rs:2669` (#587: where the gate holds a flight's Recall, danger here at this share of HP allows it; 0 = off; inert at the shipped gate) |
| `caution_scary_named_monsters` | 1 | flag | 0–1 | `borg_caution.rs:3704` (0 = off; read in `borg_flow_kill.rs`, `borg_update.rs`, `borg_messages.rs`) |

## Flee only while the escape is real (#530)

The registry now holds **1830** parameters. Owner, 2026-11-17: the Borg
should flee scary monsters while it has the chance and, once it sees it
does not, stand and fight. With `caution_scary_stand = 1`, in the dungeon,
`borg_near_monster_type` decides the level's scary flag after its scan
(`borg_caution::apply_scary_stand`):
- a monster that never moves (`RF_NEVER_MOVE`) is never scary: it cannot
  follow;
- the level is fled (C) while the escape is real: a free staircase within
  `caution_breeder_easy_escape_steps` or an escape item (#503's easy-escape
  test, `easy_escape_with`), or no scary monster awake and at least as fast
  as the character;
- otherwise the Borg stands: the scary flag is cleared, the leave, flee and
  town flight the scary check itself set (`mon.scary_leaving`,
  `scary_fleeing`, `scary_town`: each was off before it) are withdrawn, and
  the note
  `# Scary guy on level, no escape: standing and fighting.` marks the
  change. #504's cornered mode supplies the boosts when the danger is high.
  The decision is remade at every scan. A door scare (an unseen monster
  opened or burst a door, with the named list on) enters the decision as a
  scary pursuer (`mon.scary_door`). A new level and the jumpstarts forget
  every scary fact (`forget_scary`).

Compiled 0 (C), shipped 1. Evidence: of 1,301 deaths in today's 12,456 zoo
games about 43% were monsters on the list, with it on; in 48 replayed
poison deaths (`e0aa2e0a`) the Borg was fleeing at the end of every one
(25 white worm masses, 11 white jellies). Local, 18 classes-18 starts:
setting 0 plays the same keys as before; at 1, 2 deaths against 3, summed
level 93 against 89, depth 41 against 38, 45 stands. On those 48 poison
starts on the current build: 1 death against 2, 81 stands.

| parameter | default | unit | range | source |
|---|---|---|---|---|
| `caution_scary_stand` | 0 | flag | 0–1 | `borg_caution.rs:2809` (read in `borg_flow_kill.rs`) |

### The scary flag stops latching (#515)

With `caution_scary_stand = 1`, a scan that finds no scary monster left the
flag as it was, so a level stayed scary until a new level even after its
scary monsters were dead or forgotten. With `caution_scary_latch = 0`,
`apply_scary_stand` then clears the flag and withdraws only the goals the
scare set (`unlatch_scare`), with the note
`# Scary guy gone: the level is no longer scary.` Door scares stay latched.
The unlatch also ends a stand, so the next stand prints its note again.
"Gone" includes forgotten: a monster not seen for 2000 game turns leaves the
known monsters (`expire_kills`, `borg_update.rs:1566`), so with latch 0
walking away from a scary monster ends its scare too, not only killing it.
Read an arm's results with that in mind (review of #562).

Compiled and shipped 1 (the latch), for a zoo pilot. Evidence (a lead):
from the turn-88,147 save of `priest-1055640847`, 5 seeds to 120,000
turns: latch on, 2 deaths to the worm mass and 1 of 5 in town by 120,000;
latch off, 5 alive and 2 in town. The six borg-play deaths from birth with
stand mode on: the same 3 deaths either way.

| parameter | default | unit | range | source |
|---|---|---|---|---|
| `caution_scary_latch` | 1 | flag | 0–1 | `borg_caution.rs:2836` |
| `caution_scary_unlatch_kill` | 0 | flag | 0–1 | `borg_caution.rs:2837` (#590: with `caution_scary_stand`, clear once the last scary monster known was killed, latch on or off; forgetting is not enough) |
| `caution_scary_unlatch_calmed` | 0 | flag | 0–1 | `borg_caution.rs:2838` (#593: with `caution_scary_stand`, clear once a named-list or damage-rule scary monster is known hurt or afraid, or killed, latch on or off) |

## The named scary list as data (#518)

Owner, 2026-11-17: the named list should be flexible, by monster, character
level and class. With `borg_scary_list = FILE` in `borg.txt` (a relative name
is resolved under the user directory), `borg_near_monster_type` reads the
named list from that file instead of the compiled C block
(`borg-core/src/scary_list.rs`); without the key the compiled list stays.
`caution_scary_named_monsters` still switches named scares on and off.

`harness/scary_monsters.txt` gives each monster a range, `tough-fragile`,
and each class a fragility from 0 to 100 (owner, 2026-11-17): a class fears
a monster while its level is at most
`tough + (fragile - tough) x fragility / 100`, rounded. One line per
monster: `monster | tough-fragile | class exceptions | flags`:
- the monster's name as in `monster.txt`, or `base:` and the start of a
  monster base name;
- a single number is the same level for every class;
- `class level` pairs set that class's level for this monster outright;
- the flag `breeders`: only with at least as many known breeders as the
  character's level, and not while standing against breeders (#503).

The shipped fragilities are Warrior, Paladin, Blackguard 0; Ranger, Rogue 33;
Priest, Druid 67; Mage, Necromancer 100. Each range is C's level to C's
level + 3, so Fang (C: 5) is 5-8: 5, 6, 7 and 8 for those four groups. C's
only class exception (the squint-eyed rogue scares Mages and Priests up to 6)
lies inside its range. `harness/scary_monsters_c.txt` is C's list exactly
(single levels plus that exception), a control arm. The borg-core test
`the_c_file_matches_the_compiled_list_on_every_race` checks it against the
compiled condition (`borg_flow_kill::c_named_scary`) over every race in
`monster.txt`, every class, levels 0-60 and the breeder cases, and checks
that the shipped ranges match it for the classes at fragility 0; it also
plays byte-identical keys to the compiled list on the 18 classes-18 starts. With the shipped ranges, 2 of the 18 games change (the
Rogue and the Ranger): the frail casters never passed C's levels within
100,000 turns.

A later line for the same monster replaces the earlier one, `monster | none`
removes it, and a later `fragility:` value replaces an earlier one, so an
experiment can append its changes to the shipped list (borg-zoo#228).

An unknown monster, class or flag, a bad level or a backwards range stops
the run before play
with exit 2 and the line. The shipped `borg.txt` selects `scary_monsters.txt`
(owner, 2026-11-17); `borg_scary_list = none` keeps the compiled C list.
`harness/run.sh` takes `BORG_SCARY_LIST=FILE` first (under its own file
name), otherwise the list the `borg.txt` names from `harness/`, and with
`none` runs the compiled list with no file and no hash. A copied list is
pinned by its SHA-256 in the run's `borg.txt` and recorded in `config.txt`
(`borg_scary_list`, `borg_scary_list_sha256`); the zoo copies and pins it as
well (docs/ZOO-INTERFACE.md).

## Scary by damage (#505)

The registry now holds **1803** parameters (1802 after #503; one added).
Owner, 2026-11-17: scary is a monster that can one- or two-shot you, as a
percentage of HP. Turning the named list off (#503) killed characters in
borg-zoo#216 (run 44147930313): 5 and 4 deaths of 18 against 1 with the
list, two of them at character level 1. The list stood in for this rule:
each name had a character-level cap because Grip, Fang or a kobold is
deadly only to a weak character.

A monster on the level (`borg_near_monster_type`, every known monster, as
the named list) makes the level scary when one character turn of its melee
takes at least `caution_scary_damage_percent` of max HP. The damage is the
monster's own: the average of each blow's `random_value` as the engine
rolls it (`randcalc` AVERAGE: base, dice and level bonus; no blow in
`monster.txt` has a base or bonus today), times its moves per
character turn (`extract_energy` of both speeds, at least one). It is not
`borg_danger`: a first try with it at 50 rated a street urchin (it steals)
at 245 and a floating eye (it paralyses) at 276 against a 10-HP character,
and made a level scary 14,507 times in 18 games. Spells are not counted.
Max HP, not current HP: a wounded character is already the
danger-against-avoidance flight's business. The scary level then works as
before (leave and flee, latched until a new level). The resist-based scares
and the named list's switch stay. Compiled 0 (off); `harness/borg.txt`
ships 60 (owner), with the named list off.

Local check, 18 class starts (borg-zoo's classes-18), 100,000 turns, main
`dd5f026e`:

| arm | deaths | sum of CL | sum of max depth | "Scary guy" notes |
|---|---:|---:|---:|---:|
| list off (main) | 2 | 109 | 42 | 0 |
| named list on | 2 | 83 | 36 | 11,739 |
| damage 30 | 2 | 111 | 43 | 415 |
| damage 50 | 3 | 111 | 44 | 81 |
| damage 60 (shipped) | 3 | 108 | 43 | 4 (16 of 18 games keep list off's keys) |
| damage 80 | 2 | 109 | 42 | 0 (keys identical to list off) |

At 30 the scares come from battle-scarred veterans, mercenaries, grey molds,
centipedes and once Grip; at 50 almost only veterans, against low-HP
characters. 18 games cannot separate the arms' deaths (borg-zoo#216 saw the
list help: 1 death against 5); the zoo decides.

| parameter | default | unit | range | source |
|---|---|---|---|---|
| `caution_scary_damage_percent` | 0 | percent of max HP | 0–1000 | `borg_caution.rs:3768` (0 = off; read in `borg_flow_kill.rs`) |

## No unknown items (#470)

The registry now holds **1781** parameters: five removed here, and one added by #469 (`attack_speed_fight_danger_tenths`).
`caution_unknown_danger_multiplier` and `caution_unknown_max_level` set C's
last-resort rule in `borg_caution` that used unknown devices, scrolls,
potions and mushrooms; `attack_unknown_rod_damage` and
`attack_unknown_wand_damage` priced aiming an unknown rod or wand at a
monster, and `attack_staff_paranoia_chance` was that attack's paranoia roll. Both uses are gone (an unidentified item is never used), so the
parameters are too.

## Outmatched healing flag (#457)

The registry now holds **1785** parameters (1783 before; two added). Two
parameters control one study branch, off in the compiled registry (default 0);
since #479 the shipped `harness/borg.txt` sets `caution_heal_when_outmatched = 1`
(#479's entry in docs/HANDOVER.md, the archive):

| parameter | default | unit | range | source |
|---|---|---|---|---|
| `caution_heal_when_outmatched` | 0 | flag | 0-1 | `borg_caution.rs:229` |
| `caution_heal_outmatched_min_loss` | 50 | percent | 0-100 | `borg_caution.rs:233` |

`borg_heal` drinks a cure potion only when `danger < curhp + heal`, so when
the danger estimate reaches the Borg's maximum HP no potion qualifies, and
with no escape left the Borg falls through to `borg_attack` holding potions.
With the flag at 1, at the very end of `borg_caution` (after every stair,
retreat, escape and back-away option declined, so escapes keep precedence), a
Borg that has lost at least `caution_heal_outmatched_min_loss` percent of its
maximum HP and whose danger is at least its current HP drinks Cure Light, else
Serious, else Critical Wounds ("# Healing (outmatched).") instead of attacking.
With the flag at 0 the code path is not entered and play is byte-identical.

## Configuration and records

At core initialization, values are loaded once for the process, including
reincarnations. Precedence is `BORG_PARAM_<UPPERCASE_NAME>`, then
`borg_param_<name>` in the run's `ANGBAND_DIR_USER/borg.txt`, then the default.
Names are case-sensitive: file keys use lowercase, for example
`borg_param_recover_rest_sp_tenths`; environment keys use uppercase,
`BORG_PARAM_RECOVER_REST_SP_TENTHS`. Mixed-case names are rejected.
Other `borg.txt` settings are unchanged. For example:

```text
borg_param_caution_clw_min_loss = 20
```

```sh
BORG_CORE=borg-rs BORG_PARAM_CAUTION_CLW_MIN_LOSS=20 \
  harness/run.sh harness/out/calibration 1010 0 0 100000 menu
```

Values must be integers in the inclusive range. Invalid numbers, unknown
parameter names and duplicate file settings stop initialization with a
specific error on stderr and exit status 2; they are never clamped. An
environment override replaces the file value before numeric validation.

`params-core.json` in `BORG_RUN_DIR` records every name, default, effective
value, origin (`default`, `borg.txt`, `environment`), unit, bounds, source
location and description. A failed write aborts initialization. The harness
hashes this file into `config.txt` as `params_core_sha256`. Death dossiers
keep the file and hash; encounter families include the hash so environment
arms cannot silently merge into one family. Runs of our original core do
not load these settings. Zoo profile wiring is P3.

The parameter table and all marked hooks are ours. Source locations below
refer to **borg_caution.rs at reborn commit `726e04fa`**, before extraction,
not to the file's shifting line numbers after the hooks.

The core selector accepts exactly `reborn`/`0` and `borg-rs`/`1`.
`BORG_CORE` takes precedence over `borg_reborn_core` in borg.txt; absent
both, the compiled fallback is the original reborn core. Since #479 the shipped
`harness/borg.txt` selects `borg-rs`, so a game reading it plays borg-rs's core;
since #606 `harness/run.sh` plays it too when `BORG_CORE` is unset (`BORG_CORE=reborn` selects the reborn core). Invalid selectors,
including an empty environment value, fail with source/value diagnostics
and exit status 2 instead of silently selecting another measured arm.

## Units and arithmetic

- `hp`, `sp`, `gold`, `level`, `depth`, `count`, `grids` and `votes` use
  their literal units. `danger` uses borg-rs's existing danger score.
- `percent` is on the original 0–100 scale; `permille` is the device
  failure scale, 0–1000. Loss means missing HP, not remaining HP.
- `tenths` is the numerator of `x * parameter / 10`. `multiplier` is the
  factor of `x * parameter`; `divisor` is the denominator of `x / parameter`
  and is always positive. `denominator multiplier` means a factor inside
  a denominator: `danger_melee_hit_power_scale` multiplies `(level + power)`
  before division, so its lower bound remains 1. Integer operations keep
  their original order.
- `turns` uses the existing caller's clock: Borg-clock ticks for memory
  and delays, danger horizons in Borg turns, engine rest count for
  `caution_recall_rest_turns` and `recover_rod_rest_turns`, and the original fuel timeout unit for
  `caution_nasty_light_timeout`. There is no clock conversion.
- The two numeric rest parameters queue `R`, decimal digits, Enter.
  These request engine turns (manual `command.rst:186–194`;
  `ui-command.c:textui_cmd_rest:191–226`). Engine
  `cmd-cave.c:do_cmd_rest:1639–1645` treats 1 as the previous rest count,
  so explicit duration bounds are 2–9999. Defaults 500/100 are unchanged;
  disturbances can interrupt rest before that requested duration.
- Protected-summon power citations with default 1 point to the original
  `p += spell_power` branch: its multiplier was implicit. The nearby
  literal `* 3` belongs to a different branch and is not that default.
- Healing band endpoints remain separate parameters. Changing them can
  intentionally create gaps or overlaps; tune the bands as a policy.

## Constants retained, and why

The following numbers in caution remain literals because they describe
representation or game structure rather than an adjustable decision:

- Class IDs 0–5; `GOAL_FLEE = 7`; inventory light offset 5; key encoding
  (`b'0'`, direction 5); raw engine enums and item/spell IDs.
- Zero tests for a boolean/status, town depth, empty inventory, no danger,
  dead/missing monsters or no goal; -1 sentinels; index zero, loop
  increments and one-based monster-table boundaries.
- Eight neighbor directions, adjacency at one grid, the first retreat step and the 2/3 history offsets
  required to recognize the same bounce pattern. Fast-monster safety's
  two-grid reach is tunable separately.
- 100 for percent conversion and random-number domains, 10 for tenths,
  11 for the regional fear-grid indexing. The RNG domain stays fixed. At defaults the number/order of RNG calls
  is preserved; changing landing samples/trials changes draw counts and
  the subsequent stream. Chance cutoffs are parameters.
- `near.unique >= 10` is the encoded questor category. Depth 100 is
  Morgoth's level, the `< 100` questor exclusion is that boundary, and
  `CDEPTH + 1` is the next dungeon level. The strategy's 98/85/80 depth
  gates are parameters.
- The first point per drained stat counts distinct drains; class-specific
  extra votes and HP/SP-dependent weights are parameters. Checks against
  actual current/max HP and SP or computed danger retain their relationships.
- The diagnostic-only danger cutoff `avoidance / 10` controls a printed
  note, not an action. Comment examples (healing dice and an unused C
  estimate) are documentation, not executed decisions.

## Parameter catalogue

The description and original expression are also available in the registry
and at each source location. The prefix `caution_` is part of every name.

| Name | Default | Unit | Range | Original location |
| --- | ---: | --- | --- | --- |
| `caution_bounce_healthy_tenths` | 7 | tenths | 0–100 | `borg_caution.rs:105` |
| `caution_bounce_limit` | 300 | turns | 0–100000 | `borg_caution.rs:111` |
| `caution_caster_stat_vote` | 1 | votes | 0–100 | `borg_caution.rs:166` |
| `caution_priest_stat_vote` | 1 | votes | 0–100 | `borg_caution.rs:169` |
| `caution_druid_stat_vote` | 1 | votes | 0–100 | `borg_caution.rs:172` |
| `caution_necromancer_stat_vote` | 1 | votes | 0–100 | `borg_caution.rs:175` |
| `caution_warrior_stat_vote` | 1 | votes | 0–100 | `borg_caution.rs:178` |
| `caution_stat_fix_hp_high` | 850 | hp | 0–10000 | `borg_caution.rs:180` |
| `caution_stat_fix_vote_high` | 1 | votes | 0–100 | `borg_caution.rs:181` |
| `caution_stat_fix_hp_low` | 700 | hp | 0–10000 | `borg_caution.rs:183` |
| `caution_stat_fix_vote_low` | 3 | votes | 0–100 | `borg_caution.rs:184` |
| `caution_priest_stat_fix_sp` | 100 | sp | 0–10000 | `borg_caution.rs:187` |
| `caution_priest_stat_fix_votes` | 5 | votes | 0–100 | `borg_caution.rs:190` |
| `caution_mage_stat_fix_sp` | 100 | sp | 0–10000 | `borg_caution.rs:193` |
| `caution_mage_stat_fix_votes` | 5 | votes | 0–100 | `borg_caution.rs:196` |
| `caution_heal_spell_fail` | 15 | percent | 0–100 | `borg_caution.rs:257` |
| `caution_clw_estimate_percent` | 15 | percent | 0–100 | `borg_caution.rs:264` |
| `caution_clw_estimate_min` | 15 | hp | 0–10000 | `borg_caution.rs:265` |
| `caution_csw_estimate_percent` | 20 | percent | 0–100 | `borg_caution.rs:269` |
| `caution_csw_estimate_min` | 25 | hp | 0–10000 | `borg_caution.rs:270` |
| `caution_ccw_estimate_percent` | 25 | percent | 0–100 | `borg_caution.rs:274` |
| `caution_ccw_estimate_min` | 30 | hp | 0–10000 | `borg_caution.rs:275` |
| `caution_heal_estimate_percent` | 35 | percent | 0–100 | `borg_caution.rs:280` |
| `caution_heal_estimate_min` | 300 | hp | 0–10000 | `borg_caution.rs:281` |
| `caution_healing_rod_fail` | 500 | permille | 0–1000 | `borg_caution.rs:289` |
| `caution_ranged_vs_immobile` | 0 (shipped 0, #736) | mode | 0–3 | `borg_caution.rs:1558` |
| `caution_confused_heal_loss` | 80 | percent | 0–100 | `borg_caution.rs:304` |
| `caution_confused_star_loss` | 85 | percent | 0–100 | `borg_caution.rs:311` |
| `caution_confused_star_danger_multiplier` | 2 | multiplier | 0–20 | `borg_caution.rs:312` |
| `caution_confused_staff_fail` | 650 | permille | 0–1000 | `borg_caution.rs:338` |
| `caution_confused_cure_danger_tenths` | 15 | tenths | 0–100 | `borg_caution.rs:340` |
| `caution_confused_escape_danger_multiplier` | 2 | multiplier | 0–20 | `borg_caution.rs:351` |
| `caution_blind_cure_chance` | 85 | percent | 0–100 | `borg_caution.rs:368` |
| `caution_blind_escape_danger_tenths` | 25 | tenths | 0–100 | `borg_caution.rs:372` |
| `caution_blind_heal_loss` | 300 | hp | 0–10000 | `borg_caution.rs:377` |
| `caution_blind_warrior_hp_divisor` | 4 | divisor | 1–100 | `borg_caution.rs:385` |
| `caution_blind_confused_star_loss` | 400 | hp | 0–10000 | `borg_caution.rs:402` |
| `caution_blind_confused_danger_multiplier` | 5 | multiplier | 0–20 | `borg_caution.rs:403` |
| `caution_blind_confused_min_loss` | 100 | hp | 0–10000 | `borg_caution.rs:404` |
| `caution_questor_heal_hp` | 700 | hp | 0–10000 | `borg_caution.rs:413` |
| `caution_questor_holy_word_hp` | 250 | hp | 0–10000 | `borg_caution.rs:414` |
| `caution_questor_holy_word_fail` | 14 | percent | 0–100 | `borg_caution.rs:414` |
| `caution_questor_life_stat_votes` | 5 | votes | 0–100 | `borg_caution.rs:416` |
| `caution_questor_life_loss` | 500 | hp | 0–10000 | `borg_caution.rs:418` |
| `caution_questor_emergency_holy_hp` | 250 | hp | 0–10000 | `borg_caution.rs:430` |
| `caution_questor_emergency_holy_fail` | 5 | percent | 0–100 | `borg_caution.rs:430` |
| `caution_questor_safe_holy_hp` | 550 | hp | 0–10000 | `borg_caution.rs:431` |
| `caution_questor_safe_holy_fail` | 15 | percent | 0–100 | `borg_caution.rs:431` |
| `caution_questor_healing_fail` | 15 | percent | 0–100 | `borg_caution.rs:432` |
| `caution_magi_sp_divisor` | 5 | divisor | 1–100 | `borg_caution.rs:442` |
| `caution_magi_chance` | 50 | percent | 0–100 | `borg_caution.rs:443` |
| `caution_restore_mana_sp_divisor` | 10 | divisor | 1–100 | `borg_caution.rs:452` |
| `caution_restore_mana_sp` | 70 | sp | 0–10000 | `borg_caution.rs:453` |
| `caution_restore_mana_maxsp` | 200 | sp | 0–10000 | `borg_caution.rs:453` |
| `caution_restore_mana_danger_multiplier` | 2 | multiplier | 0–20 | `borg_caution.rs:456` |
| `caution_restore_stat_votes` | 5 | votes | 0–100 | `borg_caution.rs:479` |
| `caution_restore_stat_hp` | 650 | hp | 0–10000 | `borg_caution.rs:481` |
| `caution_heal_unique_chance_bonus` | 10 | percent | 0–100 | `borg_caution.rs:500` |
| `caution_heal_lethal_chance_bonus` | 75 | percent | 0–100 | `borg_caution.rs:505` |
| `caution_heal_nonpriest_chance_penalty` | 25 | percent | 0–100 | `borg_caution.rs:507` |
| `caution_heal_risky_chance_penalty` | 5 | percent | 0–100 | `borg_caution.rs:512` |
| `caution_heal_band1_max_loss` | 15 | percent | 0–100 | `borg_caution.rs:515` |
| `caution_heal_band1_skip_chance` | 98 | percent | 0–100 | `borg_caution.rs:516` |
| `caution_heal_band2_min_loss` | 16 | percent | 0–100 | `borg_caution.rs:517` |
| `caution_heal_band2_max_loss` | 25 | percent | 0–100 | `borg_caution.rs:517` |
| `caution_heal_band2_skip_chance` | 95 | percent | 0–100 | `borg_caution.rs:519` |
| `caution_heal_band3_min_loss` | 26 | percent | 0–100 | `borg_caution.rs:520` |
| `caution_heal_band3_max_loss` | 50 | percent | 0–100 | `borg_caution.rs:520` |
| `caution_heal_band3_skip_chance` | 80 | percent | 0–100 | `borg_caution.rs:522` |
| `caution_heal_band4_min_loss` | 51 | percent | 0–100 | `borg_caution.rs:523` |
| `caution_heal_band4_max_loss` | 65 | percent | 0–100 | `borg_caution.rs:523` |
| `caution_heal_band4_skip_chance` | 50 | percent | 0–100 | `borg_caution.rs:525` |
| `caution_heal_band5_min_loss` | 66 | percent | 0–100 | `borg_caution.rs:526` |
| `caution_heal_band5_max_loss` | 74 | percent | 0–100 | `borg_caution.rs:526` |
| `caution_heal_band5_skip_chance` | 25 | percent | 0–100 | `borg_caution.rs:528` |
| `caution_heal_band6_min_loss` | 75 | percent | 0–100 | `borg_caution.rs:529` |
| `caution_heal_band6_skip_chance` | 1 | percent | 0–100 | `borg_caution.rs:530` |
| `caution_clw_min_loss` | 30 | percent | 0–100 | `borg_caution.rs:540` |
| `caution_clw_max_loss` | 40 | percent | 0–100 | `borg_caution.rs:541` |
| `caution_clw_max_level` | 10 | level | 1–51 | `borg_caution.rs:541` |
| `caution_clw_danger_divisor` | 3 | divisor | 1–100 | `borg_caution.rs:543` |
| `caution_csw_min_loss` | 40 | percent | 0–100 | `borg_caution.rs:552` |
| `caution_csw_max_loss` | 50 | percent | 0–100 | `borg_caution.rs:553` |
| `caution_csw_max_level` | 20 | level | 1–51 | `borg_caution.rs:553` |
| `caution_csw_danger_divisor` | 3 | divisor | 1–100 | `borg_caution.rs:555` |
| `caution_ccw_min_loss` | 50 | percent | 0–100 | `borg_caution.rs:564` |
| `caution_ccw_max_loss` | 55 | percent | 0–100 | `borg_caution.rs:564` |
| `caution_ccw_danger_divisor` | 3 | divisor | 1–100 | `borg_caution.rs:566` |
| `caution_emergency_ccw_hp` | 50 | hp | 0–10000 | `borg_caution.rs:576` |
| `caution_deep_ccw_depth` | 80 | depth | 0–127 | `borg_caution.rs:586` |
| `caution_deep_ccw_danger` | 50 | danger | 0–10000 | `borg_caution.rs:587` |
| `caution_deep_ccw_loss` | 20 | percent | 0–100 | `borg_caution.rs:588` |
| `caution_heal_device_min_loss` | 55 | percent | 0–100 | `borg_caution.rs:596` |
| `caution_conserve_heal_depth` | 98 | depth | 0–127 | `borg_caution.rs:616` |
| `caution_heal_potion_min_loss` | 50 | percent | 0–100 | `borg_caution.rs:626` |
| `caution_heal_all_min_loss` | 60 | percent | 0–100 | `borg_caution.rs:640` |
| `caution_heal_unique_reserve_min_loss` | 65 | percent | 0–100 | `borg_caution.rs:659` |
| `caution_heal_last_resort_loss` | 75 | percent | 0–100 | `borg_caution.rs:685` |
| `caution_cure_safe_danger_tenths` | 2 | tenths | 0–100 | `borg_caution.rs:699` |
| `caution_poison_cure_hp_divisor` | 2 | divisor | 1–100 | `borg_caution.rs:706` |
| `caution_poison_cure_fail` | 60 | percent | 0–100 | `borg_caution.rs:708` |
| `caution_poison_herbal_fail` | 60 | percent | 0–100 | `borg_caution.rs:709` |
| `caution_poison_healing_fail` | 60 | percent | 0–100 | `borg_caution.rs:724` |
| `caution_poison_holy_fail` | 60 | percent | 0–100 | `borg_caution.rs:725` |
| `caution_poison_critical_hp` | 2 | hp | 0–10000 | `borg_caution.rs:744` |
| `caution_poison_critical_hp_divisor` | 20 | divisor | 1–100 | `borg_caution.rs:746` |
| `caution_poison_reserve_hp_divisor` | 20 | divisor | 1–100 | `borg_caution.rs:784` |
| `caution_cut_cure_hp_divisor` | 3 | divisor | 1–100 | `borg_caution.rs:799` |
| `caution_cut_cure_chance` | 20 | percent | 0–100 | `borg_caution.rs:800` |
| `caution_cut_critical_ccw_hp` | 10 | hp | 0–10000 | `borg_caution.rs:805` |
| `caution_cut_critical_hp` | 2 | hp | 0–10000 | `borg_caution.rs:814` |
| `caution_cut_critical_hp_divisor` | 20 | divisor | 1–100 | `borg_caution.rs:816` |
| `caution_cut_reserve_hp_divisor` | 20 | divisor | 1–100 | `borg_caution.rs:839` |
| `caution_desperate_stair_distance` | 3 | grids | 1–100 | `borg_caution.rs:1021` |
| `caution_desperate_adjacent_up_distance` | 2 | grids | 1–100 | `borg_caution.rs:1030` |
| `caution_desperate_scary_distance` | 20 | grids | 1–100 | `borg_caution.rs:1038` |
| `caution_desperate_scary_level` | 20 | level | 1–51 | `borg_caution.rs:1041` |
| `caution_desperate_unique_depth` | 5 | depth | 0–127 | `borg_caution.rs:1019` |
| `caution_desperate_adjacent_down_distance` | 2 | grids | 1–100 | `borg_caution.rs:1031` |
| `caution_retreat_max_distance` | 10 | grids | 1–100 | `borg_caution.rs:1136` |
| `caution_retreat_step_hp_tenths` | 4 | tenths | 0–100 | `borg_caution.rs:1158` |
| `caution_retreat_step_danger_tenths` | 5 | tenths | 0–100 | `borg_caution.rs:1163` |
| `caution_retreat_surrounded_level` | 20 | level | 1–51 | `borg_caution.rs:1168` |
| `caution_retreat_surrounded_distance` | 5 | grids | 1–100 | `borg_caution.rs:1169` |
| `caution_retreat_surrounded_hp_tenths` | 15 | tenths | 0–100 | `borg_caution.rs:1170` |
| `caution_retreat_low_level_hp_multiplier` | 4 | multiplier | 0–20 | `borg_caution.rs:1179` |
| `caution_retreat_monster_age` | 2 | turns | 0–100000 | `borg_caution.rs:1197` |
| `caution_back_surrounded_danger_tenths` | 12 | tenths | 0–100 | `borg_caution.rs:1340` |
| `caution_back_monster_age` | 2 | turns | 0–100000 | `borg_caution.rs:1348` |
| `caution_back_fast_monster_distance` | 2 | grids | 1–100 | `borg_caution.rs:1359` |
| `caution_back_high_level` | 35 | level | 1–51 | `borg_caution.rs:1385` |
| `caution_back_high_level_danger_tenths` | 6 | tenths | 0–100 | `borg_caution.rs:1386` |
| `caution_back_low_level` | 35 | level | 1–51 | `borg_caution.rs:1395` |
| `caution_back_low_level_danger_tenths` | 8 | tenths | 0–100 | `borg_caution.rs:1397` |
| `caution_back_injured_level` | 10 | level | 1–51 | `borg_caution.rs:1419` |
| `caution_back_freedom_depth` | 85 | depth | 0–127 | `borg_caution.rs:1428` |
| `caution_move_antisummon_delay` | 50 | turns | 0–100000 | `borg_caution.rs:1790` |
| `caution_move_max_hp` | 500 | hp | 0–10000 | `borg_caution.rs:1793` |
| `caution_one_blow_monster_age` | 2 | turns | 0–100000 | `borg_caution.rs:1825` |
| `caution_nasty_light_timeout` | 250 | turns | 0–100000 | `borg_caution.rs:1887` |
| `caution_questor_arrival_turns` | 10 | turns | 0–100000 | `borg_caution.rs:1901` |
| `caution_escape_limit` | 3 | count | 0–1000 | `borg_caution.rs:1912` |
| `caution_escape_absolute_limit` | 55 | count | 0–1000 | `borg_caution.rs:1936` |
| `caution_escape_limit_max_depth` | 98 | depth | 0–127 | `borg_caution.rs:1938` |
| `caution_escape_flee_limit` | 3 | count | 0–1000 | `borg_caution.rs:1944` |
| `caution_swap_danger_divisor` | 3 | divisor | 1–100 | `borg_caution.rs:2021` |
| `caution_recall_prep_turns` | 2 | turns | 0–100000 | `borg_caution.rs:2081` |
| `caution_restock_town_delay` | 200 | turns | 0–100000 | `borg_caution.rs:2104` |
| `caution_restock_ccw_count` | 2 | count | 0–1000 | `borg_caution.rs:2131` |
| `caution_restock_food_count` | 3 | count | 0–1000 | `borg_caution.rs:2132` |
| `caution_restock_fuel_count` | 2 | count | 0–1000 | `borg_caution.rs:2133` |
| `caution_restock_level_delay` | 400 | turns | 0–100000 | `borg_caution.rs:2135` |
| `caution_flee_danger_hp_multiplier` | 2 | multiplier | 0–20 | `borg_caution.rs:2176` |
| `caution_flee_max_level` | 50 | level | 1–51 | `borg_caution.rs:2181` |
| `caution_town_flee_max_level` | 50 | level | 1–51 | `borg_caution.rs:2195` |
| `caution_stair_low_level` | 5 | level | 1–51 | `borg_caution.rs:1892` |
| `caution_stair_danger_divisor` | 2 | divisor | 1–100 | `borg_caution.rs:1893` |
| `caution_breeder_flee_max_level` | 15 | level | 1–51 | `borg_caution.rs:1980` |
| `caution_desperate_max_level` | 35 | level | 1–51 | `borg_caution.rs:1993` |
| `caution_retreat_danger_divisor` | 3 | divisor | 1–100 | `borg_caution.rs:2028` |
| `caution_back_danger_tenths` | 4 | tenths | 0–100 | `borg_caution.rs:2066` |
| `caution_confused_emergency_loss` | 300 | hp | 0–10000 | `borg_caution.rs:2079` |
| `caution_fear_cure_chance` | 70 | percent | 0–100 | `borg_caution.rs:2101` |
| `caution_fear_berserk_fail` | 25 | percent | 0–100 | `borg_caution.rs:2107` |
| `caution_fear_hero_fail` | 25 | percent | 0–100 | `borg_caution.rs:2108` |
| `caution_fear_holy_fail` | 25 | percent | 0–100 | `borg_caution.rs:2115` |
| `caution_flee_low_hp_divisor` | 3 | divisor | 1–100 | `borg_caution.rs:2123` |
| `caution_flee_moderate_hp_divisor` | 2 | divisor | 1–100 | `borg_caution.rs:2124` |
| `caution_flee_hp_per_level` | 3 | hp_per_level | 0–100 | `borg_caution.rs:2125` |
| `caution_flee_ccw_count` | 3 | count | 0–1000 | `borg_caution.rs:2126` |
| `caution_flee_heal_count` | 1 | count | 0–1000 | `borg_caution.rs:2127` |
| `caution_flee_low_hp_chance` | 25 | percent | 0–100 | `borg_caution.rs:2130` |
| `caution_flee_status_hp_divisor` | 2 | divisor | 1–100 | `borg_caution.rs:2138` |
| `caution_flee_status_chance` | 25 | percent | 0–100 | `borg_caution.rs:2141` |
| `caution_reserve_critical_hp_divisor` | 10 | divisor | 1–100 | `borg_caution.rs:2150` |
| `caution_reserve_escape_count` | 2 | count | 0–1000 | `borg_caution.rs:2152` |
| `caution_reserve_danger_hp_divisor` | 4 | divisor | 1–100 | `borg_caution.rs:2153` |
| `caution_reserve_heal_count` | 5 | count | 0–1000 | `borg_caution.rs:2155` |
| `caution_reserve_surplus_hp_divisor` | 4 | divisor | 1–100 | `borg_caution.rs:2155` |
| `caution_reserve_unique_loss` | 600 | hp | 0–10000 | `borg_caution.rs:2157` |
| `caution_reserve_unique_depth` | 85 | depth | 0–127 | `borg_caution.rs:2159` |
| `caution_recall_danger_multiplier` | 2 | multiplier | 0–20 | `borg_caution.rs:2193` |
| `caution_stair_danger_max_level` | 49 | level | 1–51 | `borg_caution.rs:4284` |
| `caution_stair_min_food` | 2 | count | 0–1000 | `borg_caution.rs:4309` |
| `caution_sell_max_level` | 25 | level | 1–51 | `borg_caution.rs:4317` |
| `caution_sell_max_gold` | 25000 | gold | 0–1000000 | `borg_caution.rs:4318` |
| `caution_sell_min_count` | 13 | count | 0–1000 | `borg_caution.rs:4319` |
| `caution_recall_mana_maxsp` | 60 | sp | 0–10000 | `borg_caution.rs:4450` |
| `caution_recall_mana_divisor` | 4 | divisor | 1–100 | `borg_caution.rs:4451` |
| `caution_recall_phase_tolerance` | 50 | percent | 0–100 | `borg_caution.rs:4464` |
| `caution_recall_phase_horizon` | 1 | turns | 1–100000 | `borg_caution.rs:4465` |
| `caution_recall_phase_fail` | 30 | percent | 0–100 | `borg_caution.rs:4470` |
| `caution_recall_portal_fail` | 30 | percent | 0–100 | `borg_caution.rs:4471` |
| `caution_recall_cure_loss` | 100 | hp | 0–10000 | `borg_caution.rs:4484` |
| `caution_recall_heal_loss` | 150 | hp | 0–10000 | `borg_caution.rs:4494` |
| `caution_retreat_danger_horizon` | 1 | turns | 1–10 | `borg_caution.rs:1145` |
| `caution_back_danger_horizon` | 1 | turns | 1–10 | `borg_caution.rs:1373` |
| `caution_danger_horizon` | 1 | turns | 1–10 | `borg_caution.rs:1950` |
| `caution_recall_rest_turns` | 500 | turns | 2–9999 | `borg_caution.rs:2056` |
| `caution_heal_when_outmatched` | 0 | flag | 0–1 | `borg_caution.rs:229` |
| `caution_heal_outmatched_min_loss` | 50 | percent | 0–100 | `borg_caution.rs:233` |


## P1.2: danger and escape (#220)

The source locations in this section refer to **reborn `30ab9d66`**, before
these hooks. They are separate from caution's original locations above.
The registry's single generated enum/table keeps indices, names and defaults
in step. Existing configuration precedence, recording and dossier hashes
apply unchanged to the added names.

The new controls cover per-blow hit-power estimates and fear supplements;
per-spell status/inventory/summon risk weights; danger caps; low-level,
sleep/confusion and stuck-monster adjustments; regional fear; the escape
priority ladder; spell-failure tolerances; HP/SP/depth/count cutoffs; and
Phase Door/teleport sampling budgets and danger horizons. These are the
vendored policy's existing values, not recommended tuning settings.

`hit_power`, `save_points`, `sleep_rating`, `energy_tenths`, `ac`,
`coefficient` and `freedom_score` are native model units. The stair-freedom
penalty multiplies doubled distance (the existing `double_distance`), not
ordinary grids. The hit-luck bias is in percentage points and can exceed
100 before the existing probability cap; its default is 150. In contrast,
ordinary probability thresholds stay in 0–100. Level offsets use `levels`.

Sampling tolerances count scary samples, not a newly normalized percentage.
The default sample budget is 100. If that budget changes, choose the
landing tolerances on the same count scale. Sampling budgets and danger
horizons must be positive; divisions have positive denominators. No RNG
call or search order changes when the defaults are used.

The fast-player threshold is bounded at its original 135 or below. Assumed
player speeds are 110–189, leaving room for the unchanged +10 haste in the
200-entry energy table. The low-HP monster-speed bonus is 0–3, so an
override cannot increase it beyond the original adjustment. These bounds
protect table indexing; they are not a measurement of gameplay benefit.
The non-questor unique-count cutoffs stop at 9, below the questor encoding
of 10 (`borg_flow_kill.rs`, the nearby-monster scan).

### Retained constants and mechanics-model data

These remain literals. Extracting policy values does not correct or retune
the vendored mechanics model:

- Engine IDs (classes, spells, blow effects, terrain, curses), feature
  masks, table boundaries, array dimensions, the regional `/ 11` geometry,
  inventory offset 5, the eight adjacent grids, unit adjacency and loop
  counters/sentinels. Zero indicates absent/dead/no damage, or initializes
  an accumulator. A zero protected-summon fear branch remains a categorical
  exemption, not a knob that enables a new risk term.
- Actual blow dice, HP-based breath damage/caps, elemental resistance and
  immunity reductions, and spell-power damage expressions stay with the
  copied attack model. See upstream `lib/gamedata/monster.txt`,
  `monster_spell.txt`, `src/list-elements.h` and the corresponding handlers
  at engine `42cd156ac`. The model is legacy data, **not a claim that every
  expression matches today's engine**. Supplementary fear and status
  penalties are parameters; the direct damage formulas are not.
- A shield's +50 AC is engine data (`src/player-calcs.c:2160–2162`). The
  protection-from-evil divisor is now a parameter: the legacy factor 2 is
  an approximation, while `src/mon-attack.c:597–609` uses the player's level
  in `randint0(100) + level > 50`. Haste/slow's 10 speed,
  the percentage denominator 100, the game's 100 energy per action and
  the danger model's tenth-action conversion/ceiling remain units and
  arithmetic, not tunable thresholds.
- The 10/20 energy branch boundaries delimit one action and move-then-cast
  in the one-step movement simulator. Its cap of 20 belongs to that
  algorithm. `danger_minimum_monster_energy` is the separate policy floor.
- Stat 3 is the minimum drainable stat; saving/theft escape checks at 100
  represent a guaranteed roll. Crush's strict `HP < 4 * clevel` condition
  is game damage (`lib/gamedata/class.txt:1038–1044`,
  `src/project-mon.c:1004–1013`), not a safety margin. The hit-power values
  are deliberately named: for example, the legacy HURT 60 and POISON 5
  differ from `blow_effects.txt`'s current 40 and 20. Defaults preserve the
  old model rather than silently carrying a mechanics correction.
- Questor encoding 10 and game levels 99/100 remain categorical game data.
  Town depth 0 and the starvation check on the first dungeon level remain
  locations, not policy cutoffs. The 80/95/97 depth gates are parameters.
- Phase's radius 10, the legacy teleport radius 100 and half-radius sampling
  interval remain with the copied landing model, as do lash/spit reach
  checks at 3/4. The fixed lash/spit damage estimates themselves are named.
  The landing model is not today's exact teleport distribution: the manual
  ch. 13 and engine `effect_handler_TELEPORT` describe span-based long
  teleports and ±25% distance. Source carry #595 is already flagged by the
  refresh report; that correction would change default play and is not
  part of this extraction.
- Raw keys, direction 5, recall reply characters, local-map boundaries and
  panel offsets are engine interface data. The freedom score's distance
  subtraction is the definition of that score; its base values, close
  range and distance penalty are parameters.

### Additional parameter catalogue

| Name | Default | Unit | Range | Original location |
| --- | ---: | --- | --- | --- |
| `danger_monblow_hurt_fear` | 400 | danger | 0–5000 | `borg_danger.rs:208` |
| `danger_monblow_hurt_hit_power` | 60 | hit_power | 0–1000 | `borg_danger.rs:210` |
| `danger_monblow_poison_hit_power` | 5 | hit_power | 0–1000 | `borg_danger.rs:218` |
| `danger_monblow_poison_fear` | 10 | danger | 0–5000 | `borg_danger.rs:226` |
| `danger_monblow_disenchant_hit_power` | 20 | hit_power | 0–1000 | `borg_danger.rs:234` |
| `danger_monblow_disenchant_fear` | 500 | danger | 0–5000 | `borg_danger.rs:239` |
| `danger_monblow_drain_charges_fear` | 20 | danger | 0–5000 | `borg_danger.rs:248` |
| `danger_monblow_drain_charges_hit_power` | 15 | hit_power | 0–1000 | `borg_danger.rs:249` |
| `danger_monblow_eat_gold_fear` | 50 | danger | 0–5000 | `borg_danger.rs:259` |
| `danger_monblow_eat_gold_hit_power` | 5 | hit_power | 0–1000 | `borg_danger.rs:261` |
| `danger_monblow_eat_gold_fear_2` | 5 | danger | 0–5000 | `borg_danger.rs:274` |
| `danger_monblow_eat_item_hit_power` | 5 | hit_power | 0–1000 | `borg_danger.rs:282` |
| `danger_monblow_eat_item_fear` | 5 | danger | 0–5000 | `borg_danger.rs:289` |
| `danger_monblow_eat_food_hit_power` | 5 | hit_power | 0–1000 | `borg_danger.rs:297` |
| `danger_monblow_eat_food_fear` | 5 | danger | 0–5000 | `borg_danger.rs:302` |
| `danger_monblow_eat_light_hit_power` | 5 | hit_power | 0–1000 | `borg_danger.rs:310` |
| `danger_monblow_eat_light_fear` | 5 | danger | 0–5000 | `borg_danger.rs:319` |
| `danger_monblow_acid_fear` | 200 | danger | 0–5000 | `borg_danger.rs:359` |
| `danger_monblow_elec_hit_power` | 10 | hit_power | 0–1000 | `borg_danger.rs:348` |
| `danger_monblow_elec_damage_multiplier` | 2 | multiplier | 0–50 | `borg_danger.rs:356` |
| `danger_monblow_fire_hit_power` | 10 | hit_power | 0–1000 | `borg_danger.rs:367` |
| `danger_monblow_fire_damage_multiplier` | 2 | multiplier | 0–50 | `borg_danger.rs:375` |
| `danger_monblow_cold_hit_power` | 10 | hit_power | 0–1000 | `borg_danger.rs:386` |
| `danger_monblow_cold_damage_multiplier` | 2 | multiplier | 0–50 | `borg_danger.rs:394` |
| `danger_monblow_blind_hit_power` | 2 | hit_power | 0–1000 | `borg_danger.rs:402` |
| `danger_monblow_blind_fear` | 10 | danger | 0–5000 | `borg_danger.rs:407` |
| `danger_monblow_blind_fear_2` | 75 | danger | 0–5000 | `borg_danger.rs:409` |
| `danger_monblow_confuse_hit_power` | 10 | hit_power | 0–1000 | `borg_danger.rs:418` |
| `danger_monblow_confuse_fear` | 200 | danger | 0–5000 | `borg_danger.rs:423` |
| `danger_monblow_confuse_fear_2` | 200 | danger | 0–5000 | `borg_danger.rs:425` |
| `danger_monblow_terrify_hit_power` | 10 | hit_power | 0–1000 | `borg_danger.rs:434` |
| `danger_monblow_terrify_damage_multiplier` | 2 | multiplier | 0–50 | `borg_danger.rs:439` |
| `danger_monblow_paralyze_hit_power` | 2 | hit_power | 0–1000 | `borg_danger.rs:447` |
| `danger_monblow_paralyze_fear` | 200 | danger | 0–5000 | `borg_danger.rs:451` |
| `danger_monblow_lose_str_fear` | 150 | danger | 0–5000 | `borg_danger.rs:474` |
| `danger_monblow_lose_str_fear_2` | 100 | danger | 0–5000 | `borg_danger.rs:477` |
| `danger_monblow_lose_dex_fear` | 150 | danger | 0–5000 | `borg_danger.rs:498` |
| `danger_monblow_lose_dex_fear_2` | 100 | danger | 0–5000 | `borg_danger.rs:501` |
| `danger_monblow_lose_con_fear` | 150 | danger | 0–5000 | `borg_danger.rs:526` |
| `danger_monblow_lose_con_fear_2` | 100 | danger | 0–5000 | `borg_danger.rs:529` |
| `danger_monblow_lose_int_fear` | 150 | danger | 0–5000 | `borg_danger.rs:553` |
| `danger_monblow_lose_int_fear_2` | 50 | danger | 0–5000 | `borg_danger.rs:556` |
| `danger_monblow_lose_wis_fear` | 150 | danger | 0–5000 | `borg_danger.rs:577` |
| `danger_monblow_lose_wis_fear_2` | 50 | danger | 0–5000 | `borg_danger.rs:580` |
| `danger_monblow_lose_all_hit_power` | 2 | hit_power | 0–1000 | `borg_danger.rs:589` |
| `danger_monblow_shatter_hit_power` | 60 | hit_power | 0–1000 | `borg_danger.rs:596` |
| `danger_monblow_shatter_fear` | 150 | danger | 0–5000 | `borg_danger.rs:598` |
| `danger_monblow_hallu_fear` | 250 | danger | 0–5000 | `borg_danger.rs:636` |
| `danger_rsf_shriek_fear` | 5 | danger | 0–5000 | `borg_danger.rs:795` |
| `danger_rsf_br_acid_fear` | 40 | danger | 0–5000 | `borg_danger.rs:838` |
| `danger_rsf_br_elec_fear` | 20 | danger | 0–5000 | `borg_danger.rs:857` |
| `danger_rsf_br_fire_fear` | 40 | danger | 0–5000 | `borg_danger.rs:876` |
| `danger_rsf_br_cold_fear` | 20 | danger | 0–5000 | `borg_danger.rs:895` |
| `danger_rsf_br_pois_fear` | 20 | danger | 0–5000 | `borg_danger.rs:917` |
| `danger_rsf_br_neth_fear` | 125 | danger | 0–5000 | `borg_danger.rs:931` |
| `danger_rsf_br_light_fear` | 20 | danger | 0–5000 | `borg_danger.rs:947` |
| `danger_rsf_br_light_fear_2` | 20 | danger | 0–5000 | `borg_danger.rs:949` |
| `danger_rsf_br_dark_fear` | 20 | danger | 0–5000 | `borg_danger.rs:968` |
| `danger_rsf_br_dark_fear_2` | 20 | danger | 0–5000 | `borg_danger.rs:970` |
| `danger_rsf_br_soun_fear` | 500 | danger | 0–5000 | `borg_danger.rs:988` |
| `danger_rsf_br_soun_fear_2` | 1000 | danger | 0–5000 | `borg_danger.rs:991` |
| `danger_rsf_br_soun_fear_3` | 50 | danger | 0–5000 | `borg_danger.rs:994` |
| `danger_rsf_br_chao_fear` | 100 | danger | 0–5000 | `borg_danger.rs:1007` |
| `danger_rsf_br_chao_fear_2` | 200 | danger | 0–5000 | `borg_danger.rs:1011` |
| `danger_rsf_br_dise_fear` | 500 | danger | 0–5000 | `borg_danger.rs:1026` |
| `danger_rsf_br_nexu_fear` | 100 | danger | 0–5000 | `borg_danger.rs:1042` |
| `danger_rsf_br_time_fear` | 250 | danger | 0–5000 | `borg_danger.rs:1052` |
| `danger_rsf_br_iner_fear` | 100 | danger | 0–5000 | `borg_danger.rs:1062` |
| `danger_rsf_br_grav_fear` | 100 | danger | 0–5000 | `borg_danger.rs:1072` |
| `danger_rsf_br_grav_fear_2` | 500 | danger | 0–5000 | `borg_danger.rs:1078` |
| `danger_rsf_br_grav_fear_3` | 1000 | danger | 0–5000 | `borg_danger.rs:1081` |
| `danger_rsf_br_shar_fear` | 50 | danger | 0–5000 | `borg_danger.rs:1098` |
| `danger_rsf_br_plas_fear` | 100 | danger | 0–5000 | `borg_danger.rs:1112` |
| `danger_rsf_br_plas_fear_2` | 500 | danger | 0–5000 | `borg_danger.rs:1115` |
| `danger_rsf_br_plas_fear_3` | 1000 | danger | 0–5000 | `borg_danger.rs:1118` |
| `danger_rsf_br_wall_fear` | 100 | danger | 0–5000 | `borg_danger.rs:1133` |
| `danger_rsf_br_wall_fear_2` | 500 | danger | 0–5000 | `borg_danger.rs:1136` |
| `danger_rsf_br_wall_fear_3` | 50 | danger | 0–5000 | `borg_danger.rs:1139` |
| `danger_rsf_ba_acid_fear` | 40 | danger | 0–5000 | `borg_danger.rs:1172` |
| `danger_rsf_ba_elec_fear` | 20 | danger | 0–5000 | `borg_danger.rs:1187` |
| `danger_rsf_ba_fire_fear` | 40 | danger | 0–5000 | `borg_danger.rs:1202` |
| `danger_rsf_ba_cold_fear` | 20 | danger | 0–5000 | `borg_danger.rs:1217` |
| `danger_rsf_ba_pois_fear` | 20 | danger | 0–5000 | `borg_danger.rs:1235` |
| `danger_rsf_ba_shar_fear` | 20 | danger | 0–5000 | `borg_danger.rs:1247` |
| `danger_rsf_ba_neth_fear` | 250 | danger | 0–5000 | `borg_danger.rs:1259` |
| `danger_rsf_ba_wate_fear` | 500 | danger | 0–5000 | `borg_danger.rs:1269` |
| `danger_rsf_ba_wate_fear_2` | 1000 | danger | 0–5000 | `borg_danger.rs:1272` |
| `danger_rsf_ba_wate_fear_3` | 50 | danger | 0–5000 | `borg_danger.rs:1278` |
| `danger_rsf_ba_wate_fear_4` | 20 | danger | 0–5000 | `borg_danger.rs:1280` |
| `danger_rsf_ba_mana_fear` | 50 | danger | 0–5000 | `borg_danger.rs:1287` |
| `danger_rsf_ba_holy_fear` | 50 | danger | 0–5000 | `borg_danger.rs:1293` |
| `danger_rsf_ba_dark_fear` | 20 | danger | 0–5000 | `borg_danger.rs:1308` |
| `danger_rsf_ba_dark_fear_2` | 20 | danger | 0–5000 | `borg_danger.rs:1310` |
| `danger_rsf_ba_light_fear` | 20 | danger | 0–5000 | `borg_danger.rs:1326` |
| `danger_rsf_ba_light_fear_2` | 20 | danger | 0–5000 | `borg_danger.rs:1328` |
| `danger_rsf_storm_fear` | 500 | danger | 0–5000 | `borg_danger.rs:1339` |
| `danger_rsf_storm_fear_2` | 1000 | danger | 0–5000 | `borg_danger.rs:1342` |
| `danger_rsf_drain_mana_fear` | 100 | danger | 0–5000 | `borg_danger.rs:1350` |
| `danger_rsf_brain_smash_fear` | 200 | danger | 0–5000 | `borg_danger.rs:1362` |
| `danger_rsf_bo_acid_fear` | 40 | danger | 0–5000 | `borg_danger.rs:1390` |
| `danger_rsf_bo_elec_fear` | 20 | danger | 0–5000 | `borg_danger.rs:1406` |
| `danger_rsf_bo_fire_fear` | 40 | danger | 0–5000 | `borg_danger.rs:1422` |
| `danger_rsf_bo_cold_fear` | 20 | danger | 0–5000 | `borg_danger.rs:1438` |
| `danger_rsf_bo_neth_fear` | 200 | danger | 0–5000 | `borg_danger.rs:1465` |
| `danger_rsf_bo_wate_fear` | 500 | danger | 0–5000 | `borg_danger.rs:1476` |
| `danger_rsf_bo_wate_fear_2` | 1000 | danger | 0–5000 | `borg_danger.rs:1479` |
| `danger_rsf_bo_wate_fear_3` | 20 | danger | 0–5000 | `borg_danger.rs:1485` |
| `danger_rsf_bo_wate_fear_4` | 20 | danger | 0–5000 | `borg_danger.rs:1487` |
| `danger_rsf_bo_mana_fear` | 50 | danger | 0–5000 | `borg_danger.rs:1495` |
| `danger_rsf_bo_plas_fear` | 500 | danger | 0–5000 | `borg_danger.rs:1506` |
| `danger_rsf_bo_plas_fear_2` | 1000 | danger | 0–5000 | `borg_danger.rs:1509` |
| `danger_rsf_bo_ice_fear` | 20 | danger | 0–5000 | `borg_danger.rs:1517` |
| `danger_rsf_bo_ice_fear_2` | 50 | danger | 0–5000 | `borg_danger.rs:1523` |
| `danger_rsf_bo_ice_fear_3` | 1000 | danger | 0–5000 | `borg_danger.rs:1526` |
| `danger_rsf_scare_fear` | 10 | danger | 0–5000 | `borg_danger.rs:1560` |
| `danger_rsf_blind_fear` | 10 | danger | 0–5000 | `borg_danger.rs:1572` |
| `danger_rsf_conf_fear` | 10 | danger | 0–5000 | `borg_danger.rs:1583` |
| `danger_rsf_slow_fear` | 5 | danger | 0–5000 | `borg_danger.rs:1594` |
| `danger_rsf_hold_fear` | 150 | danger | 0–5000 | `borg_danger.rs:1604` |
| `danger_rsf_haste_fear` | 10 | danger | 0–5000 | `borg_danger.rs:1609` |
| `danger_rsf_heal_fear` | 10 | danger | 0–5000 | `borg_danger.rs:1614` |
| `danger_rsf_tport_fear` | 10 | danger | 0–5000 | `borg_danger.rs:1623` |
| `danger_rsf_tele_to_fear` | 20 | danger | 0–5000 | `borg_danger.rs:1628` |
| `danger_rsf_tele_self_to_fear` | 20 | danger | 0–5000 | `borg_danger.rs:1633` |
| `danger_rsf_tele_away_fear` | 10 | danger | 0–5000 | `borg_danger.rs:1638` |
| `danger_rsf_tele_level_fear` | 50 | danger | 0–5000 | `borg_danger.rs:1644` |
| `danger_rsf_darkness_fear` | 5 | danger | 0–5000 | `borg_danger.rs:1650` |
| `danger_rsf_traps_fear` | 50 | danger | 0–5000 | `borg_danger.rs:1655` |
| `danger_rsf_forget_fear` | 500 | danger | 0–5000 | `borg_danger.rs:1662` |
| `danger_rsf_forget_fear_2` | 30 | danger | 0–5000 | `borg_danger.rs:1664` |
| `danger_rsf_shapechange_fear` | 200 | danger | 0–5000 | `borg_danger.rs:1671` |
| `danger_rsf_s_kin_power_multiplier` | 3 | multiplier | 0–50 | `borg_danger.rs:1688` |
| `danger_rsf_s_kin_power_multiplier_2` | 7 | multiplier | 0–50 | `borg_danger.rs:1691` |
| `danger_summon_unique_percent` | 75 | percent | 0–100 | `borg_danger.rs:1696` |
| `danger_rsf_s_hi_demon_power_multiplier` | 6 | multiplier | 0–50 | `borg_danger.rs:1706` |
| `danger_rsf_s_hi_demon_power_multiplier_2` | 12 | multiplier | 0–50 | `borg_danger.rs:1709` |
| `danger_rsf_s_monster_power_multiplier` | 5 | multiplier | 0–50 | `borg_danger.rs:1723` |
| `danger_rsf_s_monsters_power_multiplier` | 7 | multiplier | 0–50 | `borg_danger.rs:1733` |
| `danger_rsf_s_animal_power_multiplier` | 5 | multiplier | 0–50 | `borg_danger.rs:1747` |
| `danger_rsf_s_hydra_power_multiplier` | 2 | multiplier | 0–50 | `borg_danger.rs:1762` |
| `danger_rsf_s_hydra_power_multiplier_2` | 5 | multiplier | 0–50 | `borg_danger.rs:1765` |
| `danger_rsf_s_ainu_power_multiplier` | 3 | multiplier | 0–50 | `borg_danger.rs:1780` |
| `danger_rsf_s_ainu_power_multiplier_2` | 7 | multiplier | 0–50 | `borg_danger.rs:1783` |
| `danger_rsf_s_demon_power_multiplier` | 3 | multiplier | 0–50 | `borg_danger.rs:1798` |
| `danger_rsf_s_demon_power_multiplier_2` | 7 | multiplier | 0–50 | `borg_danger.rs:1801` |
| `danger_rsf_s_hi_undead_power_multiplier` | 6 | multiplier | 0–50 | `borg_danger.rs:1816` |
| `danger_rsf_s_hi_undead_power_multiplier_2` | 12 | multiplier | 0–50 | `borg_danger.rs:1819` |
| `danger_rsf_s_hi_dragon_power_multiplier` | 6 | multiplier | 0–50 | `borg_danger.rs:1834` |
| `danger_rsf_s_hi_dragon_power_multiplier_2` | 12 | multiplier | 0–50 | `borg_danger.rs:1837` |
| `danger_rsf_s_unique_power_multiplier` | 3 | multiplier | 0–50 | `borg_danger.rs:1852` |
| `danger_rsf_s_unique_power_multiplier_2` | 6 | multiplier | 0–50 | `borg_danger.rs:1855` |
| `danger_unknown_physical` | 1000 | danger | 0–5000 | `borg_danger.rs:175` |
| `danger_knockout_max_sides` | 3 | count | 0–1000 | `borg_danger.rs:207` |
| `danger_knockout_min_dice` | 5 | count | 0–1000 | `borg_danger.rs:207` |
| `danger_gold_theft_low_level` | 5 | level | 1–51 | `borg_danger.rs:258` |
| `danger_gold_theft_min_gold` | 100 | gold | 0–1000000 | `borg_danger.rs:267` |
| `danger_gold_theft_max_gold` | 100000 | gold | 0–1000000 | `borg_danger.rs:270` |
| `danger_food_theft_reserve` | 5 | count | 0–1000 | `borg_danger.rs:298` |
| `danger_fuel_theft_reserve` | 5 | count | 0–1000 | `borg_danger.rs:315` |
| `danger_str_drain_low_stat` | 10 | stat | 3–238 | `borg_danger.rs:476` |
| `danger_dex_drain_low_stat` | 10 | stat | 3–238 | `borg_danger.rs:500` |
| `danger_con_drain_low_stat` | 8 | stat | 3–238 | `borg_danger.rs:528` |
| `danger_shatter_ac_cap` | 150 | ac | 0–1000 | `borg_danger.rs:595` |
| `danger_shatter_ac_divisor` | 250 | divisor | 1–1000 | `borg_danger.rs:595` |
| `danger_exp10_fear` | 100 | danger | 0–5000 | `borg_danger.rs:608` |
| `danger_exp_ignore_level` | 50 | level | 1–51 | `borg_danger.rs:608` |
| `danger_exp20_fear` | 150 | danger | 0–5000 | `borg_danger.rs:609` |
| `danger_exp40_fear` | 200 | danger | 0–5000 | `borg_danger.rs:610` |
| `danger_exp80_fear` | 250 | danger | 0–5000 | `borg_danger.rs:611` |
| `danger_melee_hit_luck_percent` | 150 | percent | 100–300 | `borg_danger.rs:657` |
| `danger_melee_hit_ac_scale` | 300 | coefficient | 0–1000 | `borg_danger.rs:657` |
| `danger_melee_hit_ac_divisor` | 4 | divisor | 1–1000 | `borg_danger.rs:657` |
| `danger_melee_hit_power_scale` | 3 | denominator multiplier | 1–1000 | `borg_danger.rs:657` |
| `danger_melee_hit_floor_percent` | 5 | percent | 0–100 | `borg_danger.rs:663` |
| `danger_summon_protected_divisor` | 100 | divisor | 1–1000 | `borg_danger.rs:739` |
| `danger_summon_corridor_divisor` | 1000 | divisor | 1–10000 | `borg_danger.rs:742` |
| `danger_unknown_spell` | 1000 | danger | 0–5000 | `borg_danger.rs:766` |
| `danger_brain_smash_save_weight` | 2 | multiplier | 0–50 | `borg_danger.rs:1362` |
| `danger_cause4_save_bias` | 120 | save_points | 0–1000 | `borg_danger.rs:1374` |
| `danger_forget_low_sp` | 15 | sp | 0–10000 | `borg_danger.rs:1661` |
| `danger_ignore_side_effects_depth` | 75 | depth | 0–127 | `borg_danger.rs:1876` |
| `danger_corridor_spell_tenths` | 7 | tenths | 0–30 | `borg_danger.rs:1901` |
| `danger_worst_spell_average_tenths` | 15 | tenths | 0–30 | `borg_danger.rs:1948` |
| `danger_worst_spell_hp_tenths` | 8 | tenths | 0–30 | `borg_danger.rs:1948` |
| `danger_sleep_level_margin` | 15 | level | 1–51 | `borg_danger.rs:2043` |
| `danger_ghost_monster` | 100 | danger | 0–5000 | `borg_danger.rs:2106` |
| `danger_monster_max_distance` | 20 | grids | 1–100 | `borg_danger.rs:2138` |
| `danger_fast_player_threshold` | 135 | speed | 110–135 | `borg_danger.rs:2143` |
| `danger_fast_player_unique_speed` | 120 | speed | 110–189 | `borg_danger.rs:2144` |
| `danger_fast_player_normal_speed` | 125 | speed | 110–189 | `borg_danger.rs:2144` |
| `danger_low_hp_speed_limit` | 20 | hp | 0–10000 | `borg_danger.rs:2085` |
| `danger_low_hp_monster_speed_bonus` | 3 | speed | 0–3 | `borg_danger.rs:2086` |
| `danger_minimum_monster_energy` | 10 | energy_tenths | 1–100 | `borg_danger.rs:2122` |
| `danger_stuck_bounce_limit` | 1200 | count | 0–10000 | `borg_danger.rs:2135` |
| `danger_stuck_level_turns` | 25000 | turns | 0–100000 | `borg_danger.rs:2135` |
| `danger_stuck_melee_divisor` | 5 | divisor | 1–1000 | `borg_danger.rs:2136` |
| `danger_breeder_melee_max_level` | 20 | level | 1–51 | `borg_danger.rs:2150` |
| `danger_breeder_melee_extra_tenths` | 15 | tenths | 0–30 | `borg_danger.rs:2152` |
| `danger_friends_melee_max_level` | 20 | level | 1–51 | `borg_danger.rs:2158` |
| `danger_friends_melee_low_level` | 15 | level | 1–51 | `borg_danger.rs:2159` |
| `danger_friends_melee_low_extra_tenths` | 18 | tenths | 0–30 | `borg_danger.rs:2161` |
| `danger_friends_melee_extra_tenths` | 13 | tenths | 0–30 | `borg_danger.rs:2164` |
| `danger_sleeping_melee_alertness` | 5 | sleep_rating | 0–1000 | `borg_danger.rs:2170` |
| `danger_sleeping_melee_min_level` | 25 | level | 1–51 | `borg_danger.rs:2172` |
| `danger_sleeping_melee_divisor` | 2 | divisor | 1–1000 | `borg_danger.rs:2173` |
| `danger_sleep2_melee_weak_level` | 20 | level | 1–51 | `borg_danger.rs:2183` |
| `danger_sleep2_melee_weak_hp_divisor` | 2 | divisor | 1–1000 | `borg_danger.rs:2183` |
| `danger_sleep2_melee_divisor` | 3 | divisor | 1–1000 | `borg_danger.rs:2186` |
| `danger_sleep_melee_weak_level` | 20 | level | 1–51 | `borg_danger.rs:2192` |
| `danger_sleep_melee_weak_hp_divisor` | 2 | divisor | 1–1000 | `borg_danger.rs:2192` |
| `danger_sleep_melee_distance_offset` | 2 | count | 0–1000 | `borg_danger.rs:2195` |
| `danger_confused_melee_divisor` | 2 | divisor | 1–1000 | `borg_danger.rs:2204` |
| `danger_stunned_melee_numerator` | 10 | coefficient | 0–1000 | `borg_danger.rs:2207` |
| `danger_stunned_melee_divisor` | 13 | divisor | 1–1000 | `borg_danger.rs:2207` |
| `danger_confuse_spell_weak_level` | 20 | level | 1–51 | `borg_danger.rs:2211` |
| `danger_confuse_spell_weak_hp_divisor` | 2 | divisor | 1–1000 | `borg_danger.rs:2211` |
| `danger_confuse_spell_distance_offset` | 2 | count | 0–1000 | `borg_danger.rs:2214` |
| `danger_stuck_spell_divisor` | 5 | divisor | 1–1000 | `borg_danger.rs:2401` |
| `danger_breeder_spell_max_level` | 20 | level | 1–51 | `borg_danger.rs:2405` |
| `danger_breeder_spell_extra_tenths` | 12 | tenths | 0–30 | `borg_danger.rs:2406` |
| `danger_friends_spell_max_level` | 20 | level | 1–51 | `borg_danger.rs:2410` |
| `danger_friends_spell_extra_tenths` | 12 | tenths | 0–30 | `borg_danger.rs:2411` |
| `danger_sleeping_spell_alertness` | 5 | sleep_rating | 0–1000 | `borg_danger.rs:2416` |
| `danger_sleeping_spell_min_level` | 25 | level | 1–51 | `borg_danger.rs:2418` |
| `danger_sleeping_spell_divisor` | 2 | divisor | 1–1000 | `borg_danger.rs:2419` |
| `danger_sleep2_spell_linear_level` | 15 | level | 1–51 | `borg_danger.rs:2430` |
| `danger_sleep2_spell_level_subtract` | 10 | levels | 0–1000 | `borg_danger.rs:2430` |
| `danger_sleep2_spell_level_divisor` | 4 | divisor | 1–1000 | `borg_danger.rs:2430` |
| `danger_sleep2_spell_level_multiplier` | 3 | multiplier | 0–50 | `borg_danger.rs:2430` |
| `danger_sleep2_spell_level_add` | 10 | levels | 0–1000 | `borg_danger.rs:2430` |
| `danger_sleep2_spell_divisor` | 3 | divisor | 1–1000 | `borg_danger.rs:2437` |
| `danger_sleep_spell_distance_offset` | 2 | count | 0–1000 | `borg_danger.rs:2448` |
| `danger_confused_spell_divisor` | 2 | divisor | 1–1000 | `borg_danger.rs:2452` |
| `danger_stunned_spell_numerator` | 10 | coefficient | 0–1000 | `borg_danger.rs:2456` |
| `danger_stunned_spell_divisor` | 13 | divisor | 1–1000 | `borg_danger.rs:2456` |
| `danger_confuse_spell_divisor` | 6 | divisor | 1–1000 | `borg_danger.rs:2459` |
| `danger_frequency_low_percent` | 11 | percent | 0–100 | `borg_danger.rs:2467` |
| `danger_frequency_low_tenths` | 4 | tenths | 0–30 | `borg_danger.rs:2468` |
| `danger_frequency_mid_percent` | 26 | percent | 0–100 | `borg_danger.rs:2469` |
| `danger_frequency_mid_tenths` | 6 | tenths | 0–30 | `borg_danger.rs:2470` |
| `danger_frequency_high_percent` | 51 | percent | 0–100 | `borg_danger.rs:2471` |
| `danger_frequency_high_tenths` | 8 | tenths | 0–30 | `borg_danger.rs:2472` |
| `danger_maximum` | 2000 | danger | 0–5000 | `borg_danger.rs:2487` |
| `danger_regional_fear_max_depth` | 80 | depth | 0–127 | `borg_danger.rs:2618` |
| `danger_morgoth_regional_fear_cap` | 300 | danger | 0–5000 | `borg_danger.rs:2623` |
| `danger_monster_fear_bounce_limit` | 200 | count | 0–1000 | `borg_danger.rs:2630` |
| `danger_rsf_s_kin_protected_power_multiplier` | 1 | multiplier | 0–50 | `borg_danger.rs:1685` |
| `danger_rsf_s_hi_demon_protected_power_multiplier` | 1 | multiplier | 0–50 | `borg_danger.rs:1703` |
| `danger_rsf_s_hydra_protected_power_multiplier` | 1 | multiplier | 0–50 | `borg_danger.rs:1759` |
| `danger_rsf_s_ainu_protected_power_multiplier` | 1 | multiplier | 0–50 | `borg_danger.rs:1777` |
| `danger_rsf_s_demon_protected_power_multiplier` | 1 | multiplier | 0–50 | `borg_danger.rs:1795` |
| `danger_rsf_s_hi_undead_protected_power_multiplier` | 1 | multiplier | 0–50 | `borg_danger.rs:1813` |
| `danger_rsf_s_unique_protected_power_multiplier` | 1 | multiplier | 0–50 | `borg_danger.rs:1849` |
| `escape_antisummon_reset_turns` | 50 | turns | 0–100000 | `borg_escape.rs:87` |
| `escape_recall_fail` | 60 | percent | 0–100 | `borg_escape.rs:100` |
| `escape_recall_depth_window` | 5 | depth | 0–127 | `borg_escape.rs:112` |
| `escape_recall_reset_min_depth` | 80 | depth | 0–127 | `borg_escape.rs:116` |
| `escape_recall_reset_unique_count` | 5 | count | 0–1000 | `borg_escape.rs:128` |
| `escape_surround_distance` | 3 | grids | 1–100 | `borg_escape.rs:198` |
| `escape_surround_single_exit_count` | 1 | count | 1–8 | `borg_escape.rs:266` |
| `escape_surround_single_monster_count` | 1 | count | 1–8 | `borg_escape.rs:266` |
| `escape_town_stair_value` | 1000 | freedom_score | 0–5000 | `borg_escape.rs:317` |
| `escape_town_stair_close_distance` | 4 | grids | 1–100 | `borg_escape.rs:320` |
| `escape_town_stair_close_value` | 2000 | freedom_score | 0–5000 | `borg_escape.rs:321` |
| `escape_town_stair_distance_penalty` | 500 | freedom_score | 0–5000 | `borg_escape.rs:321` |
| `escape_dungeon_stair_value` | 1000 | freedom_score | 0–5000 | `borg_escape.rs:331` |
| `escape_dungeon_stair_close_distance` | 4 | grids | 1–100 | `borg_escape.rs:334` |
| `escape_dungeon_stair_close_value` | 2000 | freedom_score | 0–5000 | `borg_escape.rs:335` |
| `escape_dungeon_stair_distance_penalty` | 500 | freedom_score | 0–5000 | `borg_escape.rs:335` |
| `escape_landing_samples` | 100 | count | 1–1000 | `borg_escape.rs:363` |
| `escape_landing_trials` | 100 | count | 1–1000 | `borg_escape.rs:369` |
| `escape_unknown_landing_max_hp` | 30 | hp | 0–10000 | `borg_escape.rs:419` |
| `escape_unknown_teleport_level_turns` | 2000 | turns | 0–100000 | `borg_escape.rs:1266` |
| `escape_shadow_shift_min_hp` | 12 | hp | 0–10000 | `borg_escape.rs:1344` |
| `escape_dimension_door_range` | 50 | grids | 1–100 | `borg_escape.rs:1353` |
| `escape_dimension_door_horizon` | 2 | turns | 1–10 | `borg_escape.rs:1380` |
| `escape_near_unique_max_count` | 8 | count | 1–9 | `borg_escape.rs:1444` |
| `escape_normal_fail` | 25 | percent | 0–100 | `borg_escape.rs:1455` |
| `escape_healthy_hp_percent` | 70 | percent | 0–100 | `borg_escape.rs:1452` |
| `escape_healthy_fail` | 10 | percent | 0–100 | `borg_escape.rs:1453` |
| `escape_stunned_fail` | 35 | percent | 0–100 | `borg_escape.rs:1464` |
| `escape_morgoth_hold_hp_tenths` | 5 | tenths | 0–100 | `borg_escape.rs:1469` |
| `escape_morgoth_hold_glyph_count` | 3 | count | 0–1000 | `borg_escape.rs:1487` |
| `escape_risky_danger_extra_tenths` | 3 | tenths | 0–100 | `borg_escape.rs:1505` |
| `escape_emergency_danger_tenths` | 45 | tenths | 0–100 | `borg_escape.rs:1513` |
| `escape_morgoth_danger_tenths` | 40 | tenths | 0–100 | `borg_escape.rs:1514` |
| `escape_morgoth_max_hp` | 600 | hp | 0–10000 | `borg_escape.rs:1517` |
| `escape_sauron_danger_tenths` | 30 | tenths | 0–100 | `borg_escape.rs:1518` |
| `escape_sauron_max_hp` | 600 | hp | 0–10000 | `borg_escape.rs:1521` |
| `escape_deep_unique_danger_tenths` | 25 | tenths | 0–100 | `borg_escape.rs:1522` |
| `escape_deep_unique_depth` | 95 | depth | 0–127 | `borg_escape.rs:1524` |
| `escape_deep_unique_max_hp` | 550 | hp | 0–10000 | `borg_escape.rs:1525` |
| `escape_unique_danger_tenths` | 17 | tenths | 0–100 | `borg_escape.rs:1526` |
| `escape_ordinary_danger_tenths` | 15 | tenths | 0–100 | `borg_escape.rs:1527` |
| `escape_emergency_fail` | 15 | percent | 0–100 | `borg_escape.rs:1529` |
| `escape_initial_fail_discount` | 10 | percent | 0–100 | `borg_escape.rs:1533` |
| `escape_emergency_fail_extra` | 9 | percent | 0–100 | `borg_escape.rs:1542` |
| `escape_emergency_phase_tolerance` | 75 | count | 0–1000 | `borg_escape.rs:1464` |
| `escape_emergency_phase_horizon` | 2 | turns | 1–10 | `borg_escape.rs:1464` |
| `escape_last_ditch_max_level` | 10 | level | 1–51 | `borg_escape.rs:1571` |
| `escape_last_ditch_hp_divisor` | 10 | divisor | 1–1000 | `borg_escape.rs:1572` |
| `escape_last_ditch_dimension_fail` | 90 | percent | 0–100 | `borg_escape.rs:1574` |
| `escape_critical_phase_hp_divisor` | 10 | divisor | 1–1000 | `borg_escape.rs:1586` |
| `escape_last_ditch_phase_max_level` | 10 | level | 1–51 | `borg_escape.rs:1600` |
| `escape_last_ditch_phase_hp_divisor` | 10 | divisor | 1–1000 | `borg_escape.rs:1601` |
| `escape_last_ditch_phase_fail` | 15 | percent | 0–100 | `borg_escape.rs:1603` |
| `escape_endgame_stay_danger_tenths` | 25 | tenths | 0–100 | `borg_escape.rs:1618` |
| `escape_endgame_stay_unique_count` | 3 | count | 1–9 | `borg_escape.rs:1620` |
| `escape_endgame_stay_min_depth` | 97 | depth | 0–127 | `borg_escape.rs:1621` |
| `escape_mage_danger_tenths` | 3 | tenths | 0–100 | `borg_escape.rs:1630` |
| `escape_mage_max_sp` | 20 | sp | 0–10000 | `borg_escape.rs:1632` |
| `escape_mage_min_level` | 45 | level | 1–51 | `borg_escape.rs:1633` |
| `escape_teleport_unique_danger_tenths` | 13 | tenths | 0–100 | `borg_escape.rs:1634` |
| `escape_teleport_ordinary_danger_tenths` | 11 | tenths | 0–100 | `borg_escape.rs:1635` |
| `escape_teleport_phase_tolerance` | 50 | count | 0–1000 | `borg_escape.rs:1671` |
| `escape_teleport_phase_horizon` | 2 | turns | 1–10 | `borg_escape.rs:1671` |
| `escape_phase_antisummon_delay` | 50 | turns | 0–100000 | `borg_escape.rs:1678` |
| `escape_phase_unique_danger_tenths` | 13 | tenths | 0–100 | `borg_escape.rs:1462` |
| `escape_phase_unique_min_count` | 2 | count | 1–9 | `borg_escape.rs:1462` |
| `escape_phase_ordinary_danger_tenths` | 10 | tenths | 0–100 | `borg_escape.rs:1463` |
| `escape_afraid_warrior_danger_tenths` | 10 | tenths | 0–100 | `borg_escape.rs:1464` |
| `escape_phase_tolerance` | 25 | count | 0–1000 | `borg_escape.rs:1470` |
| `escape_phase_horizon` | 2 | turns | 1–10 | `borg_escape.rs:1470` |
| `escape_fallback_phase_tolerance` | 75 | count | 0–1000 | `borg_escape.rs:1505` |
| `escape_fallback_phase_horizon` | 2 | turns | 1–10 | `borg_escape.rs:1505` |
| `escape_failed_teleport_flee_max_level` | 35 | level | 1–51 | `borg_escape.rs:1535` |
| `escape_failed_teleport_leave_max_level` | 35 | level | 1–51 | `borg_escape.rs:1547` |
| `escape_compromised_danger_tenths` | 8 | tenths | 0–100 | `borg_escape.rs:1559` |
| `escape_compromised_max_level` | 35 | level | 1–51 | `borg_escape.rs:1560` |
| `escape_compromised_hp_divisor` | 3 | divisor | 1–1000 | `borg_escape.rs:1560` |
| `escape_compromised_unique_danger_tenths` | 9 | tenths | 0–100 | `borg_escape.rs:1561` |
| `escape_compromised_unique_max_level` | 35 | level | 1–51 | `borg_escape.rs:1563` |
| `escape_compromised_unique_hp_divisor` | 3 | divisor | 1–1000 | `borg_escape.rs:1563` |
| `escape_compromised_ordinary_danger_tenths` | 6 | tenths | 0–100 | `borg_escape.rs:1564` |
| `escape_compromised_ordinary_max_level` | 20 | level | 1–51 | `borg_escape.rs:1564` |
| `escape_compromised_general_danger_tenths` | 6 | tenths | 0–100 | `borg_escape.rs:1565` |
| `escape_compromised_general_max_level` | 35 | level | 1–51 | `borg_escape.rs:1565` |
| `escape_compromised_phase_tolerance` | 20 | count | 0–1000 | `borg_escape.rs:1569` |
| `escape_compromised_phase_horizon` | 2 | turns | 1–10 | `borg_escape.rs:1569` |
| `escape_failed_compromised_flee_max_level` | 25 | level | 1–51 | `borg_escape.rs:1608` |
| `escape_weak_caster_max_level` | 35 | level | 1–51 | `borg_escape.rs:1628` |
| `escape_weak_caster_phase_tolerance` | 65 | count | 0–1000 | `borg_escape.rs:1629` |
| `escape_weak_caster_phase_horizon` | 2 | turns | 1–10 | `borg_escape.rs:1629` |
| `escape_low_level_max_level` | 10 | level | 1–51 | `borg_escape.rs:1647` |
| `escape_low_level_danger_tenths` | 5 | tenths | 0–100 | `borg_escape.rs:1648` |
| `escape_low_level_unique_danger_tenths` | 7 | tenths | 0–100 | `borg_escape.rs:1648` |
| `escape_low_level_phase_tolerance` | 20 | count | 0–1000 | `borg_escape.rs:1652` |
| `escape_low_level_phase_horizon` | 2 | turns | 1–10 | `borg_escape.rs:1652` |
| `escape_low_caster_max_level` | 8 | level | 1–51 | `borg_escape.rs:1706` |
| `escape_low_caster_phase_tolerance` | 65 | count | 0–1000 | `borg_escape.rs:1707` |
| `escape_low_caster_phase_horizon` | 2 | turns | 1–10 | `borg_escape.rs:1707` |
| `escape_low_mana_danger_tenths` | 6 | tenths | 0–100 | `borg_escape.rs:1728` |
| `escape_low_mana_unique_danger_tenths` | 8 | tenths | 0–100 | `borg_escape.rs:1728` |
| `escape_low_mana_sp_divisor` | 10 | divisor | 1–1000 | `borg_escape.rs:1729` |
| `escape_low_mana_min_maxsp` | 100 | sp | 0–10000 | `borg_escape.rs:1729` |
| `escape_low_mana_phase_tolerance` | 20 | count | 0–1000 | `borg_escape.rs:1733` |
| `escape_low_mana_phase_horizon` | 2 | turns | 1–10 | `borg_escape.rs:1733` |
| `escape_scoot_tolerance` | 20 | count | 0–1000 | `borg_escape.rs:1773` |
| `escape_scoot_horizon` | 2 | turns | 1–10 | `borg_escape.rs:1773` |
| `escape_twitch_limit` | 50 | count | 0–1000 | `borg_escape.rs:1788` |
| `escape_twitch_phase_tolerance` | 20 | count | 0–1000 | `borg_escape.rs:1791` |
| `escape_twitch_phase_horizon` | 2 | turns | 1–10 | `borg_escape.rs:1791` |
| `danger_default_hit_power` | 0 | hit_power | 0–1000 | `borg_danger.rs:197` |
| `danger_rsf_lash_damage` | 100 | danger | 0–5000 | `borg_danger.rs:800` |
| `danger_rsf_spit_damage` | 100 | danger | 0–5000 | `borg_danger.rs:806` |
| `danger_pfe_melee_divisor` | 2 | divisor | 1–1000 | `borg_danger.rs:212` |


## P1.3: recovery (#221)

Recovery's source locations refer to **`30ab9d66`**, before extraction.
The controls cover its danger horizon and avoidance fraction; the injured
and critical-HP chance bonuses; each cheap/expensive cure's chance;
HP/SP gates; Unholy Reprieve's HP cutoff; rod-rest duration; and SP/depth,
food and danger gates for ordinary and munchkin rest.

Fractions retain their original multiply/divide order. Chance thresholds
stay in 0–100; SP tenths in 0–10; denominators/horizons and rod-rest counts
must be positive. The rod rest defaults to the same `R100<Enter>` keys.
The zero-danger requirement for cheap healing remains a categorical
no-danger guard. Status booleans, full-health/full-SP comparisons, rod
presence/charging state, goal and terrain enums, regional fear geometry,
RNG domain 100, tenths denominator 10, anti-bounce resets and the `R&`/`R*`
rest protocol remain representation or game/interface data. No rest order,
item preference, random roll or effect was changed.

| Name | Default | Unit | Range | Original location |
| --- | ---: | --- | --- | --- |
| `recover_danger_horizon` | 1 | turns | 1–10 | `borg_recover.rs:233` |
| `recover_danger_divisor` | 4 | divisor | 1–1000 | `borg_recover.rs:236` |
| `recover_hurt_hp_divisor` | 2 | divisor | 1–1000 | `borg_recover.rs:229` |
| `recover_hurt_chance_bonus` | 10 | percent | 0–100 | `borg_recover.rs:230` |
| `recover_critical_hp_divisor` | 4 | divisor | 1–1000 | `borg_recover.rs:234` |
| `recover_critical_chance_bonus` | 10 | percent | 0–100 | `borg_recover.rs:235` |
| `recover_cheap_stun_chance` | 75 | percent | 0–100 | `borg_recover.rs:242` |
| `recover_cheap_cut_chance` | 75 | percent | 0–100 | `borg_recover.rs:278` |
| `recover_cheap_poison_chance` | 75 | percent | 0–100 | `borg_recover.rs:291` |
| `recover_cheap_fear_chance` | 75 | percent | 0–100 | `borg_recover.rs:304` |
| `recover_cheap_hunger_chance` | 75 | percent | 0–100 | `borg_recover.rs:317` |
| `recover_cheap_image_chance` | 75 | percent | 0–100 | `borg_recover.rs:342` |
| `recover_cheap_heal_hp_divisor` | 2 | divisor | 1–1000 | `borg_recover.rs:349` |
| `recover_cheap_heal_chance` | 75 | percent | 0–100 | `borg_recover.rs:350` |
| `recover_cheap_heal_sp_divisor` | 4 | divisor | 1–1000 | `borg_recover.rs:352` |
| `recover_unholy_min_hp` | 90 | hp | 0–10000 | `borg_recover.rs:237` |
| `recover_costly_stun_chance` | 25 | percent | 0–100 | `borg_recover.rs:266` |
| `recover_costly_heavy_stun_chance` | 95 | percent | 0–100 | `borg_recover.rs:279` |
| `recover_costly_cut_chance` | 25 | percent | 0–100 | `borg_recover.rs:292` |
| `recover_critical_ccw_hp` | 10 | hp | 0–10000 | `borg_recover.rs:298` |
| `recover_costly_poison_chance` | 25 | percent | 0–100 | `borg_recover.rs:305` |
| `recover_costly_blind_chance` | 25 | percent | 0–100 | `borg_recover.rs:321` |
| `recover_costly_confusion_chance` | 25 | percent | 0–100 | `borg_recover.rs:336` |
| `recover_costly_fear_chance` | 25 | percent | 0–100 | `borg_recover.rs:350` |
| `recover_costly_hunger_chance` | 25 | percent | 0–100 | `borg_recover.rs:362` |
| `recover_costly_heal_hp_divisor` | 2 | divisor | 1–1000 | `borg_recover.rs:370` |
| `recover_costly_heal_chance` | 25 | percent | 0–100 | `borg_recover.rs:371` |
| `recover_rest_high_depth` | 85 | depth | 0–127 | `borg_recover.rs:418` |
| `recover_rest_high_sp_tenths` | 7 | tenths | 0–10 | `borg_recover.rs:418` |
| `recover_rest_sp_tenths` | 6 | tenths | 0–10 | `borg_recover.rs:418` |
| `recover_mana_rest_max_level` | 40 | level | 1–51 | `borg_recover.rs:461` |
| `recover_mana_rest_deep_depth` | 85 | depth | 0–127 | `borg_recover.rs:461` |
| `recover_mana_rest_sp_tenths` | 8 | tenths | 0–10 | `borg_recover.rs:462` |
| `recover_mana_rest_danger_divisor` | 10 | divisor | 1–1000 | `borg_recover.rs:463` |
| `recover_mana_rest_min_food` | 2 | count | 0–1000 | `borg_recover.rs:469` |
| `recover_munchkin_rest_min_food` | 2 | count | 0–1000 | `borg_recover.rs:492` |
| `recover_rod_rest_turns` | 100 | turns | 2–9999 | `borg_recover.rs:404` |

## Named decision trace (P1.4)

The vendored core writes `trace-core.jsonl` and death dossiers retain its
last 200 rows. See [CORE-TRACE.md](CORE-TRACE.md) for the schema, sampling
point and opt-out, and [P1.4-VALIDATION.md](P1.4-VALIDATION.md) for default
parity and the recovery override's named divergence.

## Attack (P2.1, #240)

167 attack parameters bring the registry to 770 entries. Source lines below
refer to `borg_fight_attack.rs` at `5d9abb8f`. The existing policy and damage
model are retained; these are not corrections to that model. Reviewed the
combat manual (chapter 4) and Borg notes §§15.5/15.11 before extraction.

Retained data: class/attack/effect identifiers; keycodes/directions; array
sizes, loop/index increments and missing/invalid sentinels (-1, -2, -12,
-500, -999, -9999); zero/status guards; adjacency and projection geometry
(including bolt/beam/dispel radius sentinels, panel/fear-region sizes, angle
table bounds); percentage/decimal units and dice means; slay/brand and
resistance multipliers; copied spell/device/activation damage and range
formulas, including crush self-cost and special strike blow counts.
The item-destruction penalties, mana reserves/failure allowances, paranoia
chances and strategic scoring weights are configurable.

Engine authority: `src/player-attack.c` (melee/ranged damage and hit tests),
`src/project.c` (projection geometry), `lib/gamedata/class.txt`,
`object.txt`, `activation.txt` (spell/item effects). Copied estimates may
differ from these current mechanics; no claim of exact engine modelling is
made. In particular the legacy spell-damage dispatch remains frozen.
Bounds require positive divisors/horizons, restrict leap distance to the
spell's existing four-grid geometry, and keep the multiple-unique cutoff
below questor rank 10. Numeric bounds are exploratory input limits, not
recommended values.

| Name | Default | Unit | Source line |
| --- | ---: | --- | ---: |
| `attack_melee_min_hit_tenths` | 8 | tenths | 380 |
| `attack_melee_bonus_min_level` | 15 | level | 388 |
| `attack_melee_hit_bonus` | 10 | percent | 389 |
| `attack_mage_melee_min_sp` | 1 | mana | 394 |
| `attack_mage_hit_penalty` | 10 | percent | 396 |
| `attack_melee_overkill_multiplier` | 2 | multiplier | 408 |
| `attack_mage_melee_max_level` | 40 | level | 416 |
| `attack_mage_melee_damage_tenths` | 8 | tenths | 419 |
| `attack_melee_unique_bonus` | 5 | multiplier | 430 |
| `attack_melee_town_unique_numerator` | 2 | multiplier | 438 |
| `attack_melee_town_unique_divisor` | 3 | divisor | 438 |
| `attack_town_unique_min_level` | 5 | level | 441 |
| `attack_melee_breeder_numerator` | 3 | multiplier | 448 |
| `attack_melee_breeder_divisor` | 2 | divisor | 448 |
| `attack_melee_summoner_numerator` | 3 | multiplier | 455 |
| `attack_melee_summoner_divisor` | 2 | divisor | 455 |
| `attack_melee_questor_bonus` | 5 | multiplier | 463 |
| `attack_melee_sleep_danger_horizon` | 1 | turns | 478 |
| `attack_melee_sleep_danger_multiplier` | 2 | multiplier | 480 |
| `attack_melee_danger_horizon` | 2 | turns | 491 |
| `attack_melee_partial_min_level` | 15 | level | 494 |
| `attack_melee_partial_danger_divisor` | 10 | divisor | 495 |
| `attack_status_low_level_cutoff` | 13 | level | 660 |
| `attack_status_low_level_limit` | 10 | level | 660 |
| `attack_status_level_offset` | 10 | level | 660 |
| `attack_status_level_divisor` | 4 | divisor | 660 |
| `attack_status_level_numerator` | 3 | multiplier | 660 |
| `attack_status_level_base` | 10 | level | 660 |
| `attack_status_danger_horizon` | 1 | turns | 691 |
| `attack_fear_bonus_max_level` | 10 | level | 694 |
| `attack_fear_danger_bonus` | 20 | danger | 695 |
| `attack_arrow_point_blank_divisor` | 5 | divisor | 766 |
| `attack_arrow_min_hit_tenths` | 8 | tenths | 769 |
| `attack_mana_unique_min_potions` | 3 | count | 777 |
| `attack_mana_unique_multiplier` | 2 | multiplier | 779 |
| `attack_nether_point_blank_divisor` | 5 | divisor | 887 |
| `attack_teleport_unique_injury` | 60 | percent | 995 |
| `attack_teleport_unique_danger_tenths` | 13 | tenths | 1001 |
| `attack_teleport_unique_max_depth` | 98 | depth | 1001 |
| `attack_teleport_unique_min_count` | 2 | count | 1004 |
| `attack_teleport_unique_max_count` | 8 | count | 1004 |
| `attack_teleport_mage_unique_distance` | 2 | grids | 1007 |
| `attack_teleport_vault_min_walls` | 2 | count | 1032 |
| `attack_teleport_glyph_value` | 300 | score | 1062 |
| `attack_teleport_morgoth_value` | 100 | score | 1070 |
| `attack_teleport_low_sp` | 35 | mana | 1076 |
| `attack_teleport_low_sp_value` | 150 | score | 1079 |
| `attack_turn_danger_horizon` | 1 | turns | 1090 |
| `attack_status_slow_speed_margin` | 5 | speed | 1098 |
| `attack_fear_danger_horizon` | 2 | turns | 1160 |
| `attack_fear_danger_multiplier` | 2 | multiplier | 1162 |
| `attack_crush_monster_level_margin` | 5 | level | 1175 |
| `attack_sleep_danger_horizon` | 1 | turns | 1198 |
| `attack_missile_danger_horizon` | 1 | turns | 1236 |
| `attack_missile_danger_tenths` | 2 | tenths | 1236 |
| `attack_missile_level_margin` | 5 | level | 1238 |
| `attack_missile_weak_max_level` | 20 | level | 1244 |
| `attack_ranged_overkill_multiplier` | 2 | multiplier | 1258 |
| `attack_ranged_unique_multiplier` | 3 | multiplier | 1265 |
| `attack_ranged_town_unique_numerator` | 2 | multiplier | 1273 |
| `attack_ranged_town_unique_divisor` | 3 | divisor | 1273 |
| `attack_ranged_breeder_numerator` | 3 | multiplier | 1283 |
| `attack_ranged_breeder_divisor` | 2 | divisor | 1283 |
| `attack_ranged_summoner_numerator` | 3 | multiplier | 1290 |
| `attack_ranged_summoner_divisor` | 2 | divisor | 1290 |
| `attack_ranged_questor_bonus` | 9 | multiplier | 1298 |
| `attack_bolt_max_target_age` | 2 | turns | 1347 |
| `attack_bolt_unknown_min_walls` | 2 | count | 1388 |
| `attack_bolt_unknown_min_grids` | 1 | count | 1388 |
| `attack_bolt_sleep_danger_horizon` | 1 | turns | 1415 |
| `attack_bolt_sleep_danger_divisor` | 2 | divisor | 1418 |
| `attack_bolt_danger_horizon` | 1 | turns | 1428 |
| `attack_bolt_kill_multiplier` | 2 | multiplier | 1432 |
| `attack_destroy_unknown_boots_value` | 20 | score | 1450 |
| `attack_destroy_unknown_ring_value` | 20 | score | 1456 |
| `attack_destroy_speed_ring_value` | 2000 | score | 1459 |
| `attack_destroy_valuable_potion_value` | 2000 | score | 1482 |
| `attack_destroy_unknown_potion_value` | 20 | score | 1485 |
| `attack_unmapped_fear_divisor` | 20 | divisor | 1531 |
| `attack_ego_ammo_multiplier` | 99 | multiplier | 2193 |
| `attack_throw_max_target_age` | 10 | turns | 2307 |
| `attack_throw_fast_monster_margin` | 5 | speed | 2317 |
| `attack_throw_min_item_value` | 100 | gold | 2408 |
| `attack_throw_low_damage` | 5 | damage | 2408 |
| `attack_reserve_mage_sp` | 6 | mana | 2503 |
| `attack_reserve_ranger_sp` | 22 | mana | 2504 |
| `attack_reserve_rogue_sp` | 20 | mana | 2505 |
| `attack_reserve_priest_sp` | 8 | mana | 2506 |
| `attack_reserve_paladin_sp` | 20 | mana | 2507 |
| `attack_reserve_necromancer_sp` | 10 | mana | 2508 |
| `attack_spell_paranoia_max_level` | 2 | level | 2534 |
| `attack_spell_paranoia_chance` | 1 | percent | 2535 |
| `attack_spell_unique_fail` | 40 | percent | 2546 |
| `attack_spell_fail` | 25 | percent | 2546 |
| `attack_primary_teleport_fail` | 15 | percent | 2562 |
| `attack_primary_max_level` | 30 | level | 2562 |
| `attack_spell_low_max_sp` | 50 | mana | 2575 |
| `attack_spell_reserve_divisor` | 2 | divisor | 2580 |
| `attack_spell_reserve_penalty` | 3 | multiplier | 2581 |
| `attack_spell_deep_reserve_divisor` | 3 | divisor | 2585 |
| `attack_spell_deep_reserve_penalty` | 5 | multiplier | 2586 |
| `attack_escape_reserve_min_max_sp` | 30 | mana | 2592 |
| `attack_escape_reserve_penalty` | 750 | multiplier | 2593 |
| `attack_reserve_spell_max_level` | 15 | level | 2631 |
| `attack_reserve_spell_fail` | 25 | percent | 2646 |
| `attack_reserve_spell_danger_horizon` | 1 | turns | 2651 |
| `attack_reserve_spell_danger_multiplier` | 2 | multiplier | 2651 |
| `attack_reserve_spell_near_distance` | 7 | grids | 2679 |
| `attack_reserve_spell_damage_margin` | 4 | damage | 2686 |
| `attack_dispel_paranoia_chance` | 2 | percent | 2757 |
| `attack_dispel_fail` | 25 | percent | 2762 |
| `attack_dispel_reserve_divisor` | 2 | divisor | 2775 |
| `attack_dispel_reserve_penalty` | 3 | multiplier | 2776 |
| `attack_dispel_deep_reserve_divisor` | 3 | divisor | 2780 |
| `attack_dispel_deep_reserve_penalty` | 5 | multiplier | 2781 |
| `attack_dispel_escape_min_max_sp` | 30 | mana | 2786 |
| `attack_dispel_escape_penalty` | 750 | multiplier | 2787 |
| `attack_wand_paranoia_chance` | 2 | percent | 2818 |
| `attack_wand_charge_penalty` | 50 | score | 2831 |
| `attack_rod_paranoia_chance` | 2 | percent | 2854 |
| `attack_rod_max_failure` | 500 | permille | 2859 |
| `attack_unknown_device_paranoia_chance` | 2 | percent | 2911 |
| `attack_unknown_wand_max_failure` | 500 | permille | 2929 |
| `attack_unknown_wand_penalty_min_level` | 5 | level | 2937 |
| `attack_unknown_wand_penalty` | 5 | score | 2938 |
| `attack_unknown_wand_danger_horizon` | 1 | turns | 2949 |
| `attack_unknown_wand_danger_tenths` | 7 | tenths | 2949 |
| `attack_unknown_wand_emergency_value` | 999 | score | 2951 |
| `attack_activation_paranoia_chance` | 2 | percent | 3088 |
| `attack_ring_paranoia_chance` | 2 | percent | 3132 |
| `attack_dragon_paranoia_chance` | 2 | percent | 3171 |
| `attack_leap_min_target_score` | 10 | score | 3284 |
| `attack_leap_max_distance` | 4 | grids | 3296 |
| `attack_vampire_max_distance` | 20 | grids | 3513 |
| `attack_vampire_sleep_danger_horizon` | 1 | turns | 3577 |
| `attack_vampire_sleep_danger_multiplier` | 2 | multiplier | 3579 |
| `attack_vampire_danger_horizon` | 2 | turns | 3585 |
| `attack_vampire_partial_min_level` | 15 | level | 3588 |
| `attack_vampire_partial_danger_divisor` | 10 | divisor | 3589 |
| `attack_flag_danger_horizon` | 4 | turns | 3618 |
| `attack_crush_hp_margin` | 10 | hp | 3636 |
| `attack_crush_hp_level_multiplier` | 4 | multiplier | 3636 |
| `attack_crush_post_danger_multiplier` | 2 | multiplier | 3648 |
| `attack_crush_min_post_hp` | 50 | hp | 3648 |
| `attack_crush_reserve_divisor` | 2 | divisor | 3658 |
| `attack_crush_reserve_penalty` | 10 | multiplier | 3659 |
| `attack_trance_reserve_divisor` | 2 | divisor | 3704 |
| `attack_trance_reserve_penalty` | 10 | multiplier | 3705 |
| `attack_holy_word_min_loss` | 300 | hp | 3917 |
| `attack_curse_min_hp` | 120 | hp | 4030 |
| `attack_holy_staff_hp_divisor` | 2 | divisor | 4164 |
| `attack_holy_staff_hurt_value` | 500 | score | 4164 |
| `attack_holy_staff_value` | 120 | score | 4164 |
| `attack_holy_activation_hp_divisor` | 2 | divisor | 4283 |
| `attack_holy_activation_hurt_value` | 500 | score | 4283 |
| `attack_holy_activation_value` | 120 | score | 4283 |
| `attack_max_target_age` | 2 | turns | 4376 |
| `attack_town_mage_max_level` | 10 | level | 4457 |
| `attack_fast_monster_distance` | 2 | grids | 4466 |
| `attack_scary_breeder_max_depth` | 5 | depth | 4476 |
| `attack_scary_max_level_time` | 2000 | turns | 4478 |
| `attack_scary_max_town_time` | 3000 | turns | 4478 |
| `attack_scary_depth_multiplier` | 4 | multiplier | 4487 |
| `attack_scary_min_level` | 10 | level | 4487 |

## Defence (P2.2) (#241)

218 named parameters. Source locations and validated input ranges
are also recorded in `params-core.json` for each run. Defaults retain the
existing policy; no tuning recommendation is implied.

| Name | Default | Unit | Source line |
| --- | ---: | --- | ---: |
| `defend_bless_fail` | 25 | percent | 291 |
| `defend_bless_max_target_age` | 5 | turns | 332 |
| `defend_bless_no_rest_duration` | 11000 | duration | 357 |
| `defend_speed_fail_base` | 25 | percent | 393 |
| `defend_speed_fail_mid_adjustment` | 10 | percent | 393 |
| `defend_speed_fail_low_adjustment` | 10 | percent | 393 |
| `defend_grim_purpose_fail_base` | 25 | percent | 535 |
| `defend_grim_purpose_fail_mid_adjustment` | 10 | percent | 535 |
| `defend_grim_purpose_fail_low_adjustment` | 10 | percent | 535 |
| `defend_grim_purpose_improvement_divisor` | 7 | divisor | 554 |
| `defend_grim_purpose_score_bonus` | 2 | score | 557 |
| `defend_grim_purpose_no_rest_duration` | 13000 | duration | 565 |
| `defend_resist_fecap_improvement_divisor` | 7 | divisor | 630 |
| `defend_resist_fecap_score_bonus` | 2 | score | 633 |
| `defend_resist_fecap_no_rest_duration` | 21000 | duration | 643 |
| `defend_resist_f_fail_base` | 25 | percent | 668 |
| `defend_resist_f_fail_mid_adjustment` | 10 | percent | 668 |
| `defend_resist_f_fail_low_adjustment` | 10 | percent | 668 |
| `defend_resist_f_improvement_divisor` | 7 | divisor | 698 |
| `defend_resist_f_no_rest_duration` | 21000 | duration | 721 |
| `defend_resist_c_fail_base` | 25 | percent | 744 |
| `defend_resist_c_fail_mid_adjustment` | 10 | percent | 744 |
| `defend_resist_c_fail_low_adjustment` | 10 | percent | 744 |
| `defend_resist_c_improvement_divisor` | 7 | divisor | 775 |
| `defend_resist_c_no_rest_duration` | 21000 | duration | 798 |
| `defend_resist_a_fail_base` | 25 | percent | 821 |
| `defend_resist_a_fail_mid_adjustment` | 10 | percent | 821 |
| `defend_resist_a_fail_low_adjustment` | 10 | percent | 821 |
| `defend_resist_a_improvement_divisor` | 7 | divisor | 844 |
| `defend_resist_a_no_rest_duration` | 21000 | duration | 871 |
| `defend_resist_e_fail_base` | 25 | percent | 893 |
| `defend_resist_e_fail_mid_adjustment` | 10 | percent | 893 |
| `defend_resist_e_fail_low_adjustment` | 10 | percent | 893 |
| `defend_resist_e_improvement_divisor` | 7 | divisor | 917 |
| `defend_resist_e_no_rest_duration` | 21000 | duration | 945 |
| `defend_resist_p_fail_base` | 25 | percent | 967 |
| `defend_resist_p_fail_mid_adjustment` | 10 | percent | 967 |
| `defend_resist_p_fail_low_adjustment` | 10 | percent | 967 |
| `defend_resist_p_improvement_divisor` | 7 | divisor | 990 |
| `defend_resist_p_no_rest_duration` | 21000 | duration | 1006 |
| `defend_prot_evil_fail_base` | 25 | percent | 1032 |
| `defend_prot_evil_fail_mid_adjustment` | 5 | percent | 1032 |
| `defend_prot_evil_fail_low_adjustment` | 10 | percent | 1032 |
| `defend_prot_evil_improvement_divisor` | 7 | divisor | 1069 |
| `defend_shield_improvement_divisor` | 7 | divisor | 1126 |
| `defend_shield_no_rest_duration` | 2000 | duration | 1139 |
| `defend_tele_away_fail` | 50 | percent | 1152 |
| `defend_tele_away_max_target_age` | 2 | turns | 1235 |
| `defend_hero_fail` | 15 | percent | 1336 |
| `defend_hero_no_rest_duration` | 10000 | duration | 1374 |
| `defend_regen_fail` | 15 | percent | 1384 |
| `defend_regen_no_rest_duration` | 10000 | duration | 1416 |
| `defend_berserk_fail` | 15 | percent | 1427 |
| `defend_near_evil_max_target_age` | 2 | turns | 1488 |
| `defend_smite_evil_fail` | 15 | percent | 1516 |
| `defend_glyph_fail_base` | 25 | percent | 1587 |
| `defend_glyph_fail_mid_adjustment` | 5 | percent | 1587 |
| `defend_glyph_fail_low_adjustment` | 20 | percent | 1587 |
| `defend_glyph_improvement_divisor` | 7 | divisor | 1622 |
| `defend_create_door_fail_base` | 30 | percent | 1672 |
| `defend_create_door_fail_mid_adjustment` | 5 | percent | 1672 |
| `defend_create_door_fail_low_adjustment` | 20 | percent | 1672 |
| `defend_create_door_improvement_divisor` | 7 | divisor | 1725 |
| `defend_genocide_fail_base` | 25 | percent | 1901 |
| `defend_genocide_fail_mid_adjustment` | 10 | percent | 1901 |
| `defend_genocide_fail_low_adjustment` | 10 | percent | 1901 |
| `defend_earthquake_improvement_divisor` | 5 | divisor | 2359 |
| `defend_banishment_fail` | 15 | percent | 2557 |
| `defend_inviso_fail` | 25 | percent | 2723 |
| `defend_inviso_no_rest_duration` | 18000 | duration | 2789 |
| `defend_inviso_potion_no_rest_duration` | 16000 | duration | 2795 |
| `defend_tele_away_morgoth_fail` | 40 | percent | 3097 |
| `defend_tele_away_morgoth_max_target_age` | 2 | turns | 3151 |
| `defend_banishment_morgoth_fail` | 50 | percent | 3247 |
| `defend_banishment_morgoth_max_target_age` | 2 | turns | 3291 |
| `defend_light_morgoth_fail` | 50 | percent | 3368 |
| `defend_light_morgoth_max_target_age` | 2 | turns | 3418 |
| `defend_scaled_fail_high_adjustment` | 19 | percent | 170 |
| `defend_scaled_fail_mid_numerator` | 2 | multiplier | 171 |
| `defend_scaled_fail_mid_divisor` | 3 | divisor | 171 |
| `defend_scaled_fail_low_divisor` | 3 | divisor | 173 |
| `defend_limit_unique_numerator` | 2 | multiplier | 183 |
| `defend_limit_unique_divisor` | 3 | divisor | 183 |
| `defend_limit_divisor` | 2 | divisor | 185 |
| `defend_some_danger_divisor` | 10 | divisor | 199 |
| `defend_some_danger_unique_tenths` | 7 | tenths | 199 |
| `defend_some_danger_tenths` | 5 | tenths | 199 |
| `defend_danger_horizon` | 1 | turns | 222 |
| `defend_bless_near_distance` | 3 | grids | 337 |
| `defend_bless_danger_divisor` | 12 | divisor | 347 |
| `defend_bless_max_level` | 15 | level | 347 |
| `defend_bless_max_danger_divisor` | 2 | divisor | 347 |
| `defend_speed_morgoth_tenths` | 3 | tenths | 443 |
| `defend_speed_breeder_tenths` | 7 | tenths | 448 |
| `defend_speed_questor_tenths` | 7 | tenths | 452 |
| `defend_speed_sauron_tenths` | 6 | tenths | 456 |
| `defend_speed_unique_tenths` | 6 | tenths | 468 |
| `defend_speed_morgoth_fight_tenths` | 5 | tenths | 473 |
| `defend_speed_no_unique_min_depth` | 97 | depth | 477 |
| `defend_speed_good_unique_numerator` | 2 | multiplier | 485 |
| `defend_speed_good_unique_divisor` | 3 | divisor | 485 |
| `defend_speed_good_divisor` | 2 | divisor | 485 |
| `defend_speed_good_min_danger_divisor` | 5 | divisor | 485 |
| `defend_speed_unique_numerator` | 2 | multiplier | 486 |
| `defend_speed_unique_divisor` | 3 | divisor | 486 |
| `defend_speed_divisor` | 3 | divisor | 486 |
| `defend_speed_min_danger_divisor` | 7 | divisor | 486 |
| `defend_speed_no_rest_per_level` | 1000 | turns | 496 |
| `defend_prayer_unique_tenths` | 8 | tenths | 619 |
| `defend_prayer_bonus_min_level` | 45 | level | 624 |
| `defend_prayer_level_tenths` | 8 | tenths | 625 |
| `defend_resist_fire_unique_tenths` | 8 | tenths | 693 |
| `defend_resist_cold_unique_tenths` | 8 | tenths | 770 |
| `defend_resist_all_no_rest_per_level` | 1000 | turns | 1085 |
| `defend_protection_evil_unique_tenths` | 7 | tenths | 1121 |
| `defend_shield_saving_danger_tenths` | 7 | tenths | 1163 |
| `defend_shield_saving_min_sp` | 30 | mana | 1163 |
| `defend_shield_low_danger_tenths` | 5 | tenths | 1166 |
| `defend_shield_min_danger_tenths` | 4 | tenths | 1171 |
| `defend_shield_high_danger_multiplier` | 3 | multiplier | 1176 |
| `defend_shield_high_fail_adjustment` | 10 | percent | 1177 |
| `defend_shield_mid_danger_multiplier` | 2 | multiplier | 1178 |
| `defend_shield_mid_fail_adjustment` | 5 | percent | 1180 |
| `defend_shield_low_danger_numerator` | 5 | multiplier | 1181 |
| `defend_shield_low_danger_divisor` | 2 | divisor | 1181 |
| `defend_shield_low_fail_adjustment` | 5 | percent | 1183 |
| `defend_shield_remnant_danger_divisor` | 2 | divisor | 1291 |
| `defend_berserk_min_max_hp` | 100 | hp | 1397 |
| `defend_hero_value` | 5 | score | 1453 |
| `defend_regeneration_near_distance` | 3 | grids | 1501 |
| `defend_regeneration_value` | 5 | score | 1542 |
| `defend_create_doors_max_bad_grids` | 6 | count | 1713 |
| `defend_mass_banish_fail` | 40 | percent | 1760 |
| `defend_mass_banish_min_danger_tenths` | 12 | tenths | 1769 |
| `defend_mass_banish_distance` | 20 | grids | 1789 |
| `defend_mass_banish_self_harm_tenths` | 12 | tenths | 1815 |
| `defend_mass_banish_morgoth_min_count` | 8 | count | 1823 |
| `defend_mass_banish_morgoth_tenths` | 6 | tenths | 1824 |
| `defend_banish_min_hp` | 75 | hp | 1933 |
| `defend_banish_threat_min_hp` | 375 | hp | 2045 |
| `defend_banish_threat_hp_multiplier` | 3 | multiplier | 2050 |
| `defend_banish_threat_self_harm_tenths` | 12 | tenths | 2056 |
| `defend_banish_threat_min_count` | 75 | count | 2062 |
| `defend_banish_threat_danger_divisor` | 5 | divisor | 2067 |
| `defend_banish_danger_self_harm_tenths` | 12 | tenths | 2094 |
| `defend_banish_min_danger_tenths` | 7 | tenths | 2102 |
| `defend_banish_deep_depth` | 75 | depth | 2102 |
| `defend_banish_deep_danger_tenths` | 6 | tenths | 2102 |
| `defend_banish_remnant_divisor` | 2 | divisor | 2107 |
| `defend_banish_nasties_min_hp_tenths` | 7 | tenths | 2210 |
| `defend_banish_nasties_min_hp` | 250 | hp | 2210 |
| `defend_banish_nasties_min_depth` | 25 | depth | 2215 |
| `defend_banish_nasties_danger_divisor` | 4 | divisor | 2225 |
| `defend_banish_nasties_fail` | 35 | percent | 2229 |
| `defend_banish_nasties_value` | 10 | score | 2250 |
| `defend_quake_fail` | 35 | percent | 2323 |
| `defend_quake_min_danger_tenths` | 6 | tenths | 2332 |
| `defend_quake_min_monster_distance` | 2 | grids | 2343 |
| `defend_quake_high_threat_count` | 4 | count | 2352 |
| `defend_quake_high_danger_tenths` | 7 | tenths | 2352 |
| `defend_quake_high_reduction_divisor` | 3 | divisor | 2353 |
| `defend_quake_mid_threat_count` | 3 | count | 2355 |
| `defend_quake_mid_danger_tenths` | 7 | tenths | 2355 |
| `defend_quake_mid_reduction_tenths` | 6 | tenths | 2356 |
| `defend_destruction_value` | 500 | score | 2393 |
| `defend_destruction_min_danger_tenths` | 8 | tenths | 2406 |
| `defend_destruction_low_hp_min_depth` | 90 | depth | 2406 |
| `defend_destruction_deep_low_hp` | 300 | hp | 2406 |
| `defend_destruction_unique_max_count` | 4 | count | 2427 |
| `defend_destruction_teleport_min_hp` | 275 | hp | 2428 |
| `defend_destruction_teleport_chance` | 75 | percent | 2429 |
| `defend_destruction_teleport_horizon` | 2 | turns | 2429 |
| `defend_destruction_teleport_min_escapes` | 2 | count | 2435 |
| `defend_destruction_fail` | 55 | percent | 2440 |
| `defend_destruction_emergency_danger_multiplier` | 4 | multiplier | 2448 |
| `defend_destruction_emergency_hp` | 150 | hp | 2448 |
| `defend_destruction_few_unique_count` | 2 | count | 2465 |
| `defend_destruction_few_unique_danger_multiplier` | 2 | multiplier | 2465 |
| `defend_teleport_level_danger_multiplier` | 2 | multiplier | 2496 |
| `defend_teleport_level_safe_chance` | 65 | percent | 2513 |
| `defend_teleport_level_safe_horizon` | 2 | turns | 2513 |
| `defend_teleport_level_min_escapes` | 2 | count | 2519 |
| `defend_teleport_level_fail` | 55 | percent | 2524 |
| `defend_banish_evil_min_danger_divisor` | 10 | divisor | 2563 |
| `defend_banish_evil_high_danger_multiplier` | 4 | multiplier | 2568 |
| `defend_banish_evil_fail_adjustment` | 10 | percent | 2569 |
| `defend_banish_evil_artifact_min_hp` | 100 | hp | 2577 |
| `defend_banish_evil_unique_injury` | 60 | percent | 2649 |
| `defend_banish_evil_sauron_hp` | 250 | hp | 2703 |
| `defend_banish_evil_morgoth_hp` | 350 | hp | 2706 |
| `defend_see_invis_max_age` | 45 | turns | 2735 |
| `defend_see_invis_max_danger_multiplier` | 2 | multiplier | 2740 |
| `defend_see_invis_value` | 10 | score | 2767 |
| `defend_see_invis_short_duration` | 3000 | duration | 2784 |
| `defend_see_invis_spell_duration` | 18000 | duration | 2788 |
| `defend_see_invis_potion_duration` | 30000 | duration | 2794 |
| `defend_light_beam_max_invis_age` | 48 | turns | 2817 |
| `defend_light_beam_danger_numerator` | 3 | multiplier | 2870 |
| `defend_light_beam_danger_divisor` | 4 | divisor | 2870 |
| `defend_light_beam_value` | 10 | score | 2881 |
| `defend_panel_shift_min_depth` | 70 | depth | 2917 |
| `defend_panel_shift_edge_distance` | 5 | grids | 2940 |
| `defend_panel_shift_max_age` | 10 | turns | 2977 |
| `defend_panel_shift_morgoth_max_age` | 10 | turns | 2978 |
| `defend_panel_shift_antisummon_min_age` | 50 | turns | 3020 |
| `defend_panel_shift_teleport_fail` | 30 | percent | 3038 |
| `defend_panel_shift_value` | 200 | score | 3082 |
| `defend_teleport_morgoth_min_glyphs` | 10 | count | 3121 |
| `defend_teleport_morgoth_value_cap` | 1000 | score | 3199 |
| `defend_teleport_distance` | 7 | grids | 3313 |
| `defend_teleport_min_count` | 7 | count | 3322 |
| `defend_teleport_simulated_value` | 1500 | score | 3328 |
| `defend_teleport_value` | 1000 | score | 3358 |
| `defend_smite_max_distance` | 5 | grids | 3430 |
| `defend_smite_simulated_value` | 500 | score | 3452 |
| `defend_smite_value` | 200 | score | 3469 |
| `defend_refresh_resistance_horizon` | 2 | turns | 3532 |
| `defend_refresh_resistance_duration` | 25000 | duration | 3553 |

Defence source lines refer to `5d9abb8f`; total 988 parameters. Retained
data: BD/class/effect identifiers, status and zero guards, unit success
scores (1), invalid score 9999, direction/keycodes, adjacency and panel
geometry, fear-region width 11, percentage units, questor rank 10 and
Sauron/Morgoth floors 99/100, banishment's self-damage model (3 for mass,
4 for selected race), single-monster danger's one-turn primitive.
Timers written to i16 have inclusive range 0..32767 and narrow only after
input validation; divisors and horizons are positive; unique-count
cutoffs stay below questor rank 10. Chapter 7 and Borg notes on defence,
healing and escapes distinguish mechanics from policy. Engine authority:
`src/effect-handler-general.c:effect_handler_BANISH/MASS_BANISH`
(self damage), `src/effect-handler-attack.c:effect_handler_DESTRUCTION`,
and `lib/gamedata/class.txt`/`activation.txt` (effects and durations).
Legacy predicted effects/durations and redundant branch order are retained,
not asserted to be accurate engine models.

## Item use (P2.3) (#242)

19 named parameters. Source locations and validated input ranges
are also recorded in `params-core.json` for each run. Defaults retain the
existing policy; no tuning recommendation is implied.

| Name | Default | Unit | Source line |
| --- | ---: | --- | ---: |
| `item_use_ccw_recent_window` | 4 | turns | 148 |
| `item_use_ccw_recent_skip_chance` | 75 | percent | 148 |
| `item_use_ccw_min_stock` | 2 | count | 153 |
| `item_use_food_heal_critical_hp` | 4 | hp | 423 |
| `item_use_rod_equip_max_failure` | 500 | permille | 461 |
| `item_use_rod_use_max_failure` | 500 | permille | 494 |
| `item_use_staff_use_max_failure` | 500 | permille | 586 |
| `item_use_staff_equip_max_failure` | 500 | permille | 641 |
| `item_use_staff_teleport_max_failure` | 650 | permille | 650 |
| `item_use_ring_equip_max_failure` | 500 | permille | 748 |
| `item_use_dragon_equip_max_failure` | 500 | permille | 867 |
| `item_use_unholy_min_hp` | 90 | hp | 1010 |
| `item_use_recharge_wand_max_charges` | 1 | count | 1179 |
| `item_use_recharge_staff_max_charges` | 1 | count | 1196 |
| `item_use_recharge_stacked_min_charges` | 1 | count | 1208 |
| `item_use_recharge_low_charges` | 2 | count | 1188 |
| `item_use_recharge_fail` | 96 | percent | 1189 |
| `item_use_teleport_staff_min_charges` | 3 | count | 1201 |
| `item_use_recharge_min_total_stock` | 4 | count | 1208 |

Item-use source lines refer to `5d9abb8f`; total 1007 entries. Retained
mechanics/interface data: 75% confusion skill modifier, the copied device
failure formula (including its documented zero-divisor case), percentage
and permille scales, EF_NOURISH subtype identifiers 0/2/3, inventory slot
offsets, keys/effect IDs, zero charge/readiness and status guards. Recharge
stock thresholds of one are policy and have names. Rod recall bypass,
activation ordering and unknown-item scans retain their existing behavior.
Read manual chapter7 and its device/recharge sections; engine authority is
`src/player-util.c:get_use_device_chance`, `src/effect-handler-general.c`
(recharge/nourishment), and `lib/gamedata/object.txt`/`activation.txt`. The
legacy failure estimate is retained, not presented as an exact modern
formula or silently corrected.

## Power valuation (P2.4) (#243)

727 named parameters. Source locations and validated input ranges
are also recorded in `params-core.json` for each run. Defaults retain the
existing policy; no tuning recommendation is implied.

| Name | Default | Unit | Source line |
| --- | ---: | --- | ---: |
| `power_activation_tele_other_mage_base` | 500 | score | 109 |
| `power_activation_tele_other_base` | 500 | score | 109 |
| `power_activation_tele_other_nonmage_bonus` | 500 | score | 109 |
| `power_activation_remove_fear_caster_base` | 500 | score | 111 |
| `power_activation_remove_fear_base` | 500 | score | 113 |
| `power_activation_remove_fear_noncaster_bonus` | 200 | score | 113 |
| `power_activation_resist_all_base` | 500 | score | 115 |
| `power_activation_resist_all_extra` | 150 | score | 115 |
| `power_activation_resist_all_nonmage_bonus` | 25000 | score | 115 |
| `power_activation_sleep_base` | 500 | score | 116 |
| `power_activation_sleep_class_bonus` | 200 | score | 118 |
| `power_activation_recharge_base` | 500 | score | 122 |
| `power_activation_recharge_class_bonus` | 100 | score | 124 |
| `power_activation_rage_bless_resist_base` | 500 | score | 128 |
| `power_activation_rage_bless_resist_extra` | 150 | score | 128 |
| `power_activation_rage_bless_resist_nonmage_bonus` | 25000 | score | 128 |
| `power_activation_dispel_evil_base` | 500 | score | 129 |
| `power_activation_illumination_base` | 500 | score | 132 |
| `power_activation_mapping_base` | 550 | score | 133 |
| `power_activation_clairvoyance_base` | 600 | score | 134 |
| `power_activation_fire_bolt_base` | 500 | score | 135 |
| `power_activation_cold_bolt_base` | 500 | score | 136 |
| `power_activation_elec_bolt_base` | 500 | score | 137 |
| `power_activation_acid_bolt_base` | 500 | score | 138 |
| `power_activation_mana_bolt_base` | 500 | score | 139 |
| `power_activation_stinking_cloud_base` | 500 | score | 140 |
| `power_activation_cold_ball50_base` | 500 | score | 141 |
| `power_activation_cold_ball100_base` | 500 | score | 142 |
| `power_activation_fire_ball72_base` | 500 | score | 143 |
| `power_activation_cold_bolt2_base` | 500 | score | 144 |
| `power_activation_fire_ball_base` | 500 | score | 145 |
| `power_activation_confuse2_base` | 0 | score | 147 |
| `power_activation_haste_base` | 0 | score | 148 |
| `power_activation_haste1_base` | 0 | score | 149 |
| `power_activation_haste2_base` | 0 | score | 150 |
| `power_activation_detect_objects_base` | 10 | score | 151 |
| `power_activation_probing_base` | 0 | score | 152 |
| `power_activation_stone_to_mud_base` | 0 | score | 153 |
| `power_activation_drain_life1_base` | 500 | score | 155 |
| `power_activation_drain_life2_base` | 500 | score | 156 |
| `power_activation_berserker_base` | 500 | score | 157 |
| `power_activation_cure_light_base` | 0 | score | 158 |
| `power_activation_cure_serious_base` | 0 | score | 159 |
| `power_activation_cure_critical_base` | 0 | score | 160 |
| `power_activation_cure_full2_base` | 0 | score | 161 |
| `power_activation_loskill_base` | 500 | score | 162 |
| `power_activation_recall_base` | 0 | score | 163 |
| `power_activation_arrow_base` | 500 | score | 164 |
| `power_activation_tele_phase_base` | 500 | score | 166 |
| `power_activation_detect_all_base` | 0 | score | 167 |
| `power_activation_cure_full_base` | 0 | score | 168 |
| `power_activation_heal1_base` | 0 | score | 169 |
| `power_activation_heal2_base` | 0 | score | 170 |
| `power_activation_heal3_base` | 0 | score | 171 |
| `power_activation_cure_nonorlybig_base` | 0 | score | 172 |
| `power_activation_protevil_base` | 0 | score | 173 |
| `power_activation_destroy_doors_base` | 50 | score | 174 |
| `power_activation_banishment_base` | 1000 | score | 175 |
| `power_activation_tele_long_base` | 300 | score | 179 |
| `power_activation_missile_base` | 500 | score | 180 |
| `power_activation_cure_temp_base` | 500 | score | 181 |
| `power_activation_starlight2_base` | 100 | score | 182 |
| `power_activation_bizarre_base` | 999999 | score | 183 |
| `power_activation_star_ball_base` | 500 | score | 184 |
| `power_activation_polymorph_base` | 0 | score | 186 |
| `power_activation_starlight_base` | 100 | score | 187 |
| `power_activation_light_base` | 0 | score | 188 |
| `power_activation_firebrand_base` | 500 | score | 189 |
| `power_activation_restore_life_base` | 0 | score | 190 |
| `power_activation_restore_exp_base` | 0 | score | 191 |
| `power_activation_restore_st_lev_base` | 0 | score | 192 |
| `power_activation_enlightenment_base` | 500 | score | 193 |
| `power_activation_hero_base` | 500 | score | 194 |
| `power_activation_shero_base` | 500 | score | 195 |
| `power_activation_cure_paranoia_base` | 100 | score | 196 |
| `power_activation_cure_mind_base` | 1000 | score | 197 |
| `power_activation_cure_body_base` | 1000 | score | 198 |
| `power_activation_mon_slow_base` | 500 | score | 199 |
| `power_activation_mon_confuse_base` | 500 | score | 200 |
| `power_activation_sleep_all_base` | 500 | score | 201 |
| `power_activation_mon_scare_base` | 500 | score | 202 |
| `power_activation_light_line_base` | 50 | score | 203 |
| `power_activation_disable_traps_base` | 50 | score | 204 |
| `power_activation_drain_life3_base` | 500 | score | 205 |
| `power_activation_drain_life4_base` | 500 | score | 206 |
| `power_activation_elec_ball_base` | 500 | score | 207 |
| `power_activation_elec_ball2_base` | 500 | score | 208 |
| `power_activation_acid_bolt2_base` | 500 | score | 209 |
| `power_activation_acid_bolt3_base` | 500 | score | 210 |
| `power_activation_acid_ball_base` | 500 | score | 211 |
| `power_activation_cold_ball160_base` | 500 | score | 212 |
| `power_activation_cold_ball2_base` | 500 | score | 213 |
| `power_activation_fire_ball2_base` | 500 | score | 214 |
| `power_activation_fire_ball200_base` | 500 | score | 215 |
| `power_activation_fire_bolt2_base` | 500 | score | 216 |
| `power_activation_fire_bolt3_base` | 500 | score | 217 |
| `power_activation_dispel_evil60_base` | 500 | score | 218 |
| `power_activation_dispel_undead_base` | 500 | score | 219 |
| `power_activation_dispel_all_base` | 500 | score | 220 |
| `power_activation_deep_descent_base` | 0 | score | 221 |
| `power_activation_earthquakes_base` | 500 | score | 222 |
| `power_activation_destruction2_base` | 500 | score | 223 |
| `power_activation_losslow_base` | 50 | score | 224 |
| `power_activation_lossleep_base` | 100 | score | 225 |
| `power_activation_losconf_base` | 100 | score | 226 |
| `power_activation_satisfy_base` | 50 | score | 227 |
| `power_activation_blessing_base` | 50 | score | 228 |
| `power_activation_blessing2_base` | 50 | score | 229 |
| `power_activation_blessing3_base` | 50 | score | 230 |
| `power_activation_glyph_base` | 0 | score | 231 |
| `power_activation_tele_level_base` | 5000 | score | 232 |
| `power_activation_confusing_base` | 0 | score | 233 |
| `power_activation_enchant_tohit_base` | 0 | score | 234 |
| `power_activation_enchant_todam_base` | 0 | score | 235 |
| `power_activation_enchant_weapon_base` | 0 | score | 236 |
| `power_activation_enchant_armor_base` | 0 | score | 237 |
| `power_activation_enchant_armor2_base` | 0 | score | 238 |
| `power_activation_remove_curse_base` | 9000 | score | 239 |
| `power_activation_remove_curse2_base` | 10000 | score | 240 |
| `power_activation_detect_treasure_base` | 0 | score | 241 |
| `power_activation_detect_invis_base` | 50 | score | 242 |
| `power_activation_detect_evil_base` | 50 | score | 243 |
| `power_activation_restore_mana_base` | 5000 | score | 244 |
| `power_activation_brawn_base` | 0 | score | 245 |
| `power_activation_intellect_base` | 0 | score | 246 |
| `power_activation_contemplation_base` | 0 | score | 247 |
| `power_activation_toughness_base` | 0 | score | 248 |
| `power_activation_nimbleness_base` | 0 | score | 249 |
| `power_activation_restore_str_base` | 50 | score | 250 |
| `power_activation_restore_int_base` | 50 | score | 251 |
| `power_activation_restore_wis_base` | 50 | score | 252 |
| `power_activation_restore_dex_base` | 50 | score | 253 |
| `power_activation_restore_con_base` | 50 | score | 254 |
| `power_activation_restore_all_base` | 150 | score | 255 |
| `power_activation_tmd_free_act_base` | 0 | score | 256 |
| `power_activation_tmd_infra_base` | 0 | score | 257 |
| `power_activation_tmd_sinvis_base` | 50 | score | 258 |
| `power_activation_tmd_esp_base` | 50 | score | 259 |
| `power_activation_resist_acid_base` | 100 | score | 260 |
| `power_activation_resist_elec_base` | 100 | score | 261 |
| `power_activation_resist_fire_base` | 100 | score | 262 |
| `power_activation_resist_cold_base` | 100 | score | 263 |
| `power_activation_resist_pois_base` | 150 | score | 264 |
| `power_activation_cure_confusion_base` | 50 | score | 265 |
| `power_activation_wonder_base` | 300 | score | 266 |
| `power_activation_wand_breath_base` | 0 | score | 267 |
| `power_activation_staff_magi_base` | 0 | score | 268 |
| `power_activation_staff_holy_base` | 1000 | score | 269 |
| `power_activation_drink_breath_base` | 0 | score | 270 |
| `power_activation_food_waybread_base` | 50 | score | 271 |
| `power_activation_shroom_emergency_base` | 0 | score | 272 |
| `power_activation_shroom_terror_base` | 0 | score | 273 |
| `power_activation_shroom_stone_base` | 50 | score | 274 |
| `power_activation_shroom_debility_base` | 0 | score | 275 |
| `power_activation_shroom_sprinting_base` | 0 | score | 276 |
| `power_activation_shroom_purging_base` | 50 | score | 277 |
| `power_activation_ring_acid_base` | 10000 | score | 278 |
| `power_activation_ring_flames_base` | 25000 | score | 279 |
| `power_activation_ring_ice_base` | 15000 | score | 280 |
| `power_activation_ring_lightning_base` | 10000 | score | 281 |
| `power_activation_dragon_blue_base` | 1100 | score | 282 |
| `power_activation_dragon_green_base` | 2750 | score | 283 |
| `power_activation_dragon_red_base` | 1100 | score | 284 |
| `power_activation_dragon_multihued_base` | 3250 | score | 285 |
| `power_activation_dragon_gold_base` | 5150 | score | 286 |
| `power_activation_dragon_chaos_base` | 5150 | score | 287 |
| `power_activation_dragon_law_base` | 5150 | score | 288 |
| `power_activation_dragon_balance_base` | 5150 | score | 289 |
| `power_activation_dragon_shining_base` | 5150 | score | 290 |
| `power_activation_dragon_power_base` | 5150 | score | 291 |
| `power_equipment_unidentified_weapon_bonus` | 1000000 | score | 321 |
| `power_equipment_melee_dice_weight` | 20 | score | 327 |
| `power_equipment_blows_weight` | 1 | score | 330 |
| `power_equipment_tohit_weight` | 100 | score | 333 |
| `power_equipment_todam_weight` | 30 | score | 336 |
| `power_equipment_tohit_weight_2` | 15 | score | 340 |
| `power_equipment_maxdepth_threshold` | 75 | depth | 344 |
| `power_equipment_tohit_weight_3` | 15 | score | 345 |
| `power_equipment_blows_weight_2` | 20 | score | 347 |
| `power_equipment_blows_weight_3` | 2 | score | 347 |
| `power_equipment_ws_animal_weight` | 2 | score | 354 |
| `power_equipment_ws_evil_weight` | 7 | score | 357 |
| `power_equipment_wk_undead_weight` | 5 | score | 369 |
| `power_equipment_wk_demon_weight` | 3 | score | 372 |
| `power_equipment_wk_dragon_weight` | 6 | score | 375 |
| `power_equipment_ws_giant_weight` | 4 | score | 378 |
| `power_equipment_wb_acid_weight` | 4 | score | 381 |
| `power_equipment_wb_elec_weight` | 5 | score | 384 |
| `power_equipment_wb_fire_weight` | 3 | score | 387 |
| `power_equipment_wb_cold_weight` | 3 | score | 390 |
| `power_equipment_ws_orc_weight` | 1 | score | 394 |
| `power_equipment_ws_troll_weight` | 2 | score | 397 |
| `power_equipment_ws_evil_weight_2` | 1 | score | 401 |
| `power_equipment_ws_evil_weight_3` | 1 | score | 404 |
| `power_equipment_wk_undead_weight_2` | 5 | score | 416 |
| `power_equipment_wk_demon_weight_2` | 5 | score | 419 |
| `power_equipment_wk_dragon_weight_2` | 5 | score | 422 |
| `power_equipment_w_impact_weight` | 50 | score | 431 |
| `power_equipment_heavywepon_weight` | 500000 | score | 436 |
| `power_equipment_clevel_threshold` | 10 | level | 440 |
| `power_equipment_blows_weight_4` | 45000 | score | 441 |
| `power_equipment_unidentified_bow_bonus` | 6000000 | score | 454 |
| `power_equipment_reward_damage_weight` | 11 | score | 467 |
| `power_equipment_reward_damage_weight_2` | 9 | score | 469 |
| `power_equipment_clevel_threshold_2` | 15 | level | 473 |
| `power_equipment_clevel_weight` | 200 | score | 474 |
| `power_equipment_str_threshold` | 9 | stat | 479 |
| `power_equipment_str_weight` | 5000 | score | 480 |
| `power_equipment_clevel_threshold_3` | 1 | level | 484 |
| `power_equipment_str_threshold_2` | 9 | stat | 484 |
| `power_equipment_str_weight_2` | 8000 | score | 485 |
| `power_equipment_tohit_weight_4` | 100 | score | 489 |
| `power_equipment_tohit_weight_5` | 25 | score | 493 |
| `power_equipment_ammo_tval_weight` | 30000 | score | 498 |
| `power_equipment_it_is_hard_to_hold_a_heavy_weapon_weight` | 500000 | score | 503 |
| `power_equipment_light_threshold` | 3 | count | 512 |
| `power_equipment_light_weight` | 10000 | score | 513 |
| `power_equipment_light_threshold_2` | 3 | count | 514 |
| `power_equipment_light_weight_2` | 30000 | score | 515 |
| `power_equipment_light_weight_3` | 1000 | score | 515 |
| `power_equipment_mod_moves_weight` | 3000 | score | 518 |
| `power_equipment_dam_red_weight` | 10000 | score | 519 |
| `power_equipment_reward_higher_for_10_than_50_speed_dec_threshold` | 150 | speed | 528 |
| `power_equipment_speed_tier_base` | 120 | speed | 529 |
| `power_equipment_speed_tier_slope` | 1500 | score | 529 |
| `power_equipment_speed_tier_offset` | 185000 | score | 529 |
| `power_equipment_reward_higher_for_10_than_50_speed_dec_threshold_2` | 145 | speed | 531 |
| `power_equipment_reward_higher_for_10_than_50_speed_dec_threshold_3` | 149 | speed | 531 |
| `power_equipment_speed_tier_base_2` | 120 | speed | 532 |
| `power_equipment_speed_tier_slope_2` | 1500 | score | 532 |
| `power_equipment_speed_tier_offset_2` | 180000 | score | 532 |
| `power_equipment_reward_higher_for_10_than_50_speed_dec_threshold_4` | 140 | speed | 534 |
| `power_equipment_reward_higher_for_10_than_50_speed_dec_threshold_5` | 144 | speed | 534 |
| `power_equipment_speed_tier_base_3` | 120 | speed | 535 |
| `power_equipment_speed_tier_slope_3` | 1500 | score | 535 |
| `power_equipment_speed_tier_offset_3` | 175000 | score | 535 |
| `power_equipment_reward_higher_for_10_than_50_speed_dec_threshold_6` | 135 | speed | 537 |
| `power_equipment_reward_higher_for_10_than_50_speed_dec_threshold_7` | 139 | speed | 537 |
| `power_equipment_speed_tier_base_4` | 120 | speed | 538 |
| `power_equipment_speed_tier_slope_4` | 1500 | score | 538 |
| `power_equipment_speed_tier_offset_4` | 175000 | score | 538 |
| `power_equipment_reward_higher_for_10_than_50_speed_dec_threshold_8` | 130 | speed | 540 |
| `power_equipment_reward_higher_for_10_than_50_speed_dec_threshold_9` | 134 | speed | 540 |
| `power_equipment_speed_tier_base_5` | 120 | speed | 541 |
| `power_equipment_speed_tier_slope_5` | 1500 | score | 541 |
| `power_equipment_speed_tier_offset_5` | 160000 | score | 541 |
| `power_equipment_reward_higher_for_10_than_50_speed_dec_threshold_10` | 125 | speed | 543 |
| `power_equipment_reward_higher_for_10_than_50_speed_dec_threshold_11` | 129 | speed | 543 |
| `power_equipment_speed_tier_base_6` | 110 | speed | 544 |
| `power_equipment_speed_tier_slope_6` | 1500 | score | 544 |
| `power_equipment_speed_tier_offset_6` | 135000 | score | 544 |
| `power_equipment_reward_higher_for_10_than_50_speed_dec_threshold_12` | 120 | speed | 546 |
| `power_equipment_reward_higher_for_10_than_50_speed_dec_threshold_13` | 124 | speed | 546 |
| `power_equipment_speed_tier_base_7` | 110 | speed | 547 |
| `power_equipment_speed_tier_slope_7` | 1500 | score | 547 |
| `power_equipment_speed_tier_offset_7` | 110000 | score | 547 |
| `power_equipment_reward_higher_for_10_than_50_speed_dec_threshold_14` | 115 | speed | 549 |
| `power_equipment_reward_higher_for_10_than_50_speed_dec_threshold_15` | 119 | speed | 549 |
| `power_equipment_speed_tier_base_8` | 110 | speed | 550 |
| `power_equipment_speed_tier_slope_8` | 1500 | score | 550 |
| `power_equipment_speed_tier_offset_8` | 85000 | score | 550 |
| `power_equipment_reward_higher_for_10_than_50_speed_dec_threshold_16` | 110 | speed | 552 |
| `power_equipment_reward_higher_for_10_than_50_speed_dec_threshold_17` | 114 | speed | 552 |
| `power_equipment_speed_tier_base_9` | 110 | speed | 553 |
| `power_equipment_speed_tier_slope_9` | 1500 | score | 553 |
| `power_equipment_speed_tier_offset_9` | 65000 | score | 553 |
| `power_equipment_reward_higher_for_10_than_50_speed_dec_threshold_18` | 110 | speed | 555 |
| `power_equipment_speed_tier_base_10` | 110 | speed | 556 |
| `power_equipment_speed_tier_slope_10` | 2500 | score | 556 |
| `power_equipment_speed_tier_offset_10` | 0 | score | 556 |
| `power_equipment_reward_higher_for_10_than_50_speed_dec_threshold_19` | 140 | speed | 559 |
| `power_equipment_speed_tier_base_11` | 120 | speed | 560 |
| `power_equipment_speed_tier_slope_11` | 1000 | score | 560 |
| `power_equipment_speed_tier_offset_11` | 175000 | score | 560 |
| `power_equipment_reward_higher_for_10_than_50_speed_dec_threshold_20` | 135 | speed | 562 |
| `power_equipment_reward_higher_for_10_than_50_speed_dec_threshold_21` | 139 | speed | 562 |
| `power_equipment_speed_tier_base_12` | 120 | speed | 563 |
| `power_equipment_speed_tier_slope_12` | 1000 | score | 563 |
| `power_equipment_speed_tier_offset_12` | 165000 | score | 563 |
| `power_equipment_reward_higher_for_10_than_50_speed_dec_threshold_22` | 130 | speed | 565 |
| `power_equipment_reward_higher_for_10_than_50_speed_dec_threshold_23` | 134 | speed | 565 |
| `power_equipment_speed_tier_base_13` | 120 | speed | 566 |
| `power_equipment_speed_tier_slope_13` | 1000 | score | 566 |
| `power_equipment_speed_tier_offset_13` | 150000 | score | 566 |
| `power_equipment_reward_higher_for_10_than_50_speed_dec_threshold_24` | 125 | speed | 568 |
| `power_equipment_reward_higher_for_10_than_50_speed_dec_threshold_25` | 129 | speed | 568 |
| `power_equipment_speed_tier_base_14` | 110 | speed | 569 |
| `power_equipment_speed_tier_slope_14` | 1000 | score | 569 |
| `power_equipment_speed_tier_offset_14` | 125000 | score | 569 |
| `power_equipment_reward_higher_for_10_than_50_speed_dec_threshold_26` | 120 | speed | 571 |
| `power_equipment_reward_higher_for_10_than_50_speed_dec_threshold_27` | 124 | speed | 571 |
| `power_equipment_speed_tier_base_15` | 110 | speed | 572 |
| `power_equipment_speed_tier_slope_15` | 1000 | score | 572 |
| `power_equipment_speed_tier_offset_15` | 100000 | score | 572 |
| `power_equipment_reward_higher_for_10_than_50_speed_dec_threshold_28` | 115 | speed | 574 |
| `power_equipment_reward_higher_for_10_than_50_speed_dec_threshold_29` | 119 | speed | 574 |
| `power_equipment_speed_tier_base_16` | 110 | speed | 575 |
| `power_equipment_speed_tier_slope_16` | 1000 | score | 575 |
| `power_equipment_speed_tier_offset_16` | 75000 | score | 575 |
| `power_equipment_reward_higher_for_10_than_50_speed_dec_threshold_30` | 110 | speed | 577 |
| `power_equipment_reward_higher_for_10_than_50_speed_dec_threshold_31` | 114 | speed | 577 |
| `power_equipment_speed_tier_base_17` | 110 | speed | 578 |
| `power_equipment_speed_tier_slope_17` | 1000 | score | 578 |
| `power_equipment_speed_tier_offset_17` | 55000 | score | 578 |
| `power_equipment_reward_higher_for_10_than_50_speed_dec_threshold_32` | 110 | speed | 580 |
| `power_equipment_speed_tier_base_18` | 110 | speed | 581 |
| `power_equipment_speed_tier_slope_18` | 2500 | score | 581 |
| `power_equipment_speed_tier_offset_18` | 0 | score | 581 |
| `power_equipment_str_index_weight` | 100 | score | 586 |
| `power_equipment_str_index_weight_2` | 500 | score | 591 |
| `power_equipment_other_fail_rates_threshold` | 1 | count | 609 |
| `power_equipment_other_fail_rates_weight` | 30000 | score | 610 |
| `power_equipment_dex_index_weight` | 120 | score | 618 |
| `power_equipment_con_index_weight` | 250 | score | 624 |
| `power_equipment_hp_adj_threshold` | 800 | hp | 629 |
| `power_equipment_hp_adj_weight` | 450 | score | 630 |
| `power_equipment_hp_adj_weight_2` | 800 | score | 632 |
| `power_equipment_hp_adj_weight_3` | 100 | score | 632 |
| `power_equipment_hp_adj_weight_4` | 350 | score | 632 |
| `power_equipment_hp_adj_weight_5` | 500 | score | 632 |
| `power_equipment_con_index_weight_2` | 150 | score | 636 |
| `power_equipment_hp_adj_threshold_2` | 500 | hp | 641 |
| `power_equipment_hp_adj_weight_6` | 350 | score | 642 |
| `power_equipment_hp_adj_weight_7` | 500 | score | 644 |
| `power_equipment_hp_adj_weight_8` | 100 | score | 644 |
| `power_equipment_hp_adj_weight_9` | 350 | score | 644 |
| `power_equipment_hp_adj_weight_10` | 500 | score | 644 |
| `power_equipment_disp_weight` | 2 | score | 660 |
| `power_equipment_dism_weight` | 2 | score | 661 |
| `power_equipment_dev_weight` | 25 | score | 662 |
| `power_equipment_sav_weight` | 25 | score | 663 |
| `power_equipment_sav_threshold` | 99 | count | 665 |
| `power_equipment_sav_weight_2` | 10000 | score | 666 |
| `power_equipment_stl_weight` | 2 | score | 668 |
| `power_equipment_thn_weight` | 5 | score | 670 |
| `power_equipment_thb_weight` | 35 | score | 671 |
| `power_equipment_tht_weight` | 2 | score | 672 |
| `power_equipment_dig_weight` | 2 | score | 673 |
| `power_equipment_sdig_weight` | 750 | score | 679 |
| `power_equipment_ishungry_weight` | 7500 | score | 682 |
| `power_equipment_isweak_weight` | 7500 | score | 685 |
| `power_equipment_maxdepth_threshold_2` | 20 | depth | 689 |
| `power_equipment_feath_weight` | 500 | score | 691 |
| `power_equipment_feath_weight_2` | 50 | score | 694 |
| `power_equipment_light_weight_4` | 2000 | score | 698 |
| `power_equipment_sinv_weight` | 500 | score | 702 |
| `power_equipment_sinv_weight_2` | 5000 | score | 706 |
| `power_equipment_fract_weight` | 10000 | score | 710 |
| `power_equipment_hlife_weight` | 2000 | score | 716 |
| `power_equipment_hlife_weight_2` | 200 | score | 719 |
| `power_equipment_reg_weight` | 2000 | score | 722 |
| `power_equipment_esp_weight` | 80000 | score | 725 |
| `power_equipment_icold_weight` | 65000 | score | 730 |
| `power_equipment_ielec_weight` | 40000 | score | 733 |
| `power_equipment_ifire_weight` | 80000 | score | 736 |
| `power_equipment_iacid_weight` | 50000 | score | 739 |
| `power_equipment_rcold_weight` | 3000 | score | 744 |
| `power_equipment_relec_weight` | 4000 | score | 747 |
| `power_equipment_racid_weight` | 6000 | score | 750 |
| `power_equipment_rfire_weight` | 8000 | score | 753 |
| `power_equipment_rcold_weight_2` | 10000 | score | 757 |
| `power_equipment_rpois_weight` | 20000 | score | 760 |
| `power_equipment_rsnd_weight` | 3500 | score | 763 |
| `power_equipment_rlite_weight` | 800 | score | 766 |
| `power_equipment_rdark_weight` | 800 | score | 769 |
| `power_equipment_rkaos_weight` | 5000 | score | 772 |
| `power_equipment_rconf_weight` | 80000 | score | 777 |
| `power_equipment_rconf_weight_2` | 2000 | score | 782 |
| `power_equipment_rdis_weight` | 5000 | score | 785 |
| `power_equipment_rshrd_weight` | 100 | score | 788 |
| `power_equipment_rnxus_weight` | 100 | score | 791 |
| `power_equipment_rblind_weight` | 5000 | score | 794 |
| `power_equipment_rnthr_weight` | 5500 | score | 797 |
| `power_equipment_rfear_weight` | 2000 | score | 800 |
| `power_equipment_sstr_weight` | 50 | score | 805 |
| `power_equipment_sint_weight` | 50 | score | 808 |
| `power_equipment_swis_weight` | 50 | score | 811 |
| `power_equipment_scon_weight` | 50 | score | 814 |
| `power_equipment_sdex_weight` | 50 | score | 817 |
| `power_equipment_sstr_weight_2` | 1000 | score | 826 |
| `power_equipment_esp_threshold` | 10 | depth | 835 |
| `power_equipment_esp_weight_2` | 100000 | score | 836 |
| `power_equipment_fract_threshold` | 20 | depth | 840 |
| `power_equipment_fract_weight_2` | 100000 | score | 841 |
| `power_equipment_rfire_threshold` | 25 | depth | 845 |
| `power_equipment_rfire_weight_2` | 100000 | score | 846 |
| `power_equipment_rpois_threshold` | 40 | depth | 850 |
| `power_equipment_rpois_weight_2` | 100000 | score | 851 |
| `power_equipment_relec_threshold` | 40 | depth | 853 |
| `power_equipment_relec_weight_2` | 100000 | score | 854 |
| `power_equipment_racid_threshold` | 40 | depth | 856 |
| `power_equipment_racid_weight_2` | 100000 | score | 857 |
| `power_equipment_rcold_threshold` | 40 | depth | 859 |
| `power_equipment_rcold_weight_3` | 100000 | score | 860 |
| `power_equipment_maxclevel_weight` | 100000 | score | 865 |
| `power_equipment_speed_threshold` | 115 | speed | 867 |
| `power_equipment_speed_threshold_2` | 46 | speed | 867 |
| `power_equipment_speed_weight` | 100000 | score | 868 |
| `power_equipment_rconf_threshold` | 46 | depth | 870 |
| `power_equipment_rconf_weight_3` | 100000 | score | 871 |
| `power_equipment_rnthr_threshold` | 50 | depth | 875 |
| `power_equipment_rnthr_weight_2` | 55000 | score | 876 |
| `power_equipment_rsnd_threshold` | 50 | depth | 880 |
| `power_equipment_rsnd_weight_2` | 100000 | score | 881 |
| `power_equipment_rblind_threshold` | 55 | depth | 885 |
| `power_equipment_rblind_weight_2` | 100000 | score | 886 |
| `power_equipment_esp_threshold_2` | 55 | depth | 888 |
| `power_equipment_esp_weight_3` | 100000 | score | 889 |
| `power_equipment_rnthr_threshold_2` | 60 | depth | 891 |
| `power_equipment_rnthr_weight_3` | 55000 | score | 892 |
| `power_equipment_rkaos_threshold` | 60 | depth | 896 |
| `power_equipment_rkaos_weight_2` | 104000 | score | 897 |
| `power_equipment_rdis_threshold` | 60 | depth | 899 |
| `power_equipment_rdis_weight_2` | 90000 | score | 900 |
| `power_equipment_speed_threshold_3` | 120 | speed | 902 |
| `power_equipment_speed_threshold_4` | 60 | speed | 902 |
| `power_equipment_speed_weight_2` | 100000 | score | 903 |
| `power_equipment_speed_threshold_5` | 130 | speed | 907 |
| `power_equipment_speed_threshold_6` | 80 | speed | 907 |
| `power_equipment_speed_weight_3` | 100000 | score | 908 |
| `power_equipment_rnthr_threshold_3` | 80 | depth | 915 |
| `power_equipment_rnthr_weight_4` | 15000 | score | 916 |
| `power_equipment_rdark_threshold` | 80 | depth | 918 |
| `power_equipment_rdark_weight_2` | 25000 | score | 919 |
| `power_equipment_speed_threshold_7` | 140 | speed | 921 |
| `power_equipment_speed_threshold_8` | 80 | speed | 921 |
| `power_equipment_class_weight` | 100000 | score | 922 |
| `power_equipment_reward_armor_threshold` | 15 | armor | 930 |
| `power_equipment_reward_armor_weight` | 2500 | score | 931 |
| `power_equipment_reward_armor_threshold_2` | 15 | armor | 933 |
| `power_equipment_reward_armor_threshold_3` | 75 | armor | 933 |
| `power_equipment_reward_armor_weight_2` | 2000 | score | 934 |
| `power_equipment_reward_armor_weight_3` | 28250 | score | 934 |
| `power_equipment_reward_armor_threshold_4` | 75 | armor | 936 |
| `power_equipment_reward_armor_weight_4` | 1500 | score | 937 |
| `power_equipment_reward_armor_weight_5` | 73750 | score | 937 |
| `power_equipment_reward_armor_threshold_5` | 15 | armor | 940 |
| `power_equipment_reward_armor_weight_6` | 2000 | score | 941 |
| `power_equipment_reward_armor_threshold_6` | 15 | armor | 943 |
| `power_equipment_reward_armor_threshold_7` | 75 | armor | 943 |
| `power_equipment_reward_armor_weight_7` | 500 | score | 944 |
| `power_equipment_reward_armor_weight_8` | 28350 | score | 944 |
| `power_equipment_reward_armor_threshold_8` | 75 | armor | 946 |
| `power_equipment_reward_armor_weight_9` | 100 | score | 947 |
| `power_equipment_reward_armor_weight_10` | 73750 | score | 947 |
| `power_equipment_crsagrv_weight` | 800000 | score | 955 |
| `power_equipment_crshpimp_weight` | 35000 | score | 958 |
| `power_equipment_crsmpimp_weight` | 15000 | score | 961 |
| `power_equipment_class_weight_2` | 15000 | score | 969 |
| `power_equipment_crsfear_weight` | 400000 | score | 972 |
| `power_equipment_class_weight_3` | 200000 | score | 975 |
| `power_equipment_crsdrain_xp_weight` | 400000 | score | 978 |
| `power_equipment_crsfvuln_weight` | 30000 | score | 981 |
| `power_equipment_crsevuln_weight` | 30000 | score | 984 |
| `power_equipment_crscvuln_weight` | 30000 | score | 987 |
| `power_equipment_crsavuln_weight` | 30000 | score | 990 |
| `power_equipment_crstele_weight` | 100000 | score | 994 |
| `power_equipment_crsenveloping_weight` | 50000 | score | 997 |
| `power_equipment_crsirritation_weight` | 20000 | score | 1000 |
| `power_equipment_crspois_weight` | 10000 | score | 1003 |
| `power_equipment_crssiren_weight` | 800000 | score | 1006 |
| `power_equipment_crshalu_weight` | 100000 | score | 1009 |
| `power_equipment_crspara_weight` | 800000 | score | 1012 |
| `power_equipment_crssdem_weight` | 100000 | score | 1015 |
| `power_equipment_crssdra_weight` | 100000 | score | 1018 |
| `power_equipment_crssund_weight` | 100000 | score | 1021 |
| `power_equipment_speed_threshold_9` | 140 | speed | 1023 |
| `power_equipment_speed_weight_4` | 10000 | score | 1024 |
| `power_equipment_speed_threshold_10` | 140 | speed | 1026 |
| `power_equipment_speed_weight_5` | 10000 | score | 1027 |
| `power_equipment_crsnotel_weight` | 700000 | score | 1030 |
| `power_equipment_crstwep_weight` | 100000 | score | 1033 |
| `power_equipment_crsairswing_weight` | 10000 | score | 1036 |
| `power_equipment_crsunkno_weight` | 9999999 | score | 1039 |
| `power_equipment_multiple_bonuses_weight` | 3000 | score | 1045 |
| `power_equipment_worn_need_id_weight` | 10000 | score | 1048 |
| `power_equipment_powers_weight` | 999999 | score | 1067 |
| `power_equipment_str_index_threshold` | 15 | count | 1075 |
| `power_equipment_powers_threshold` | 200 | count | 1076 |
| `power_equipment_str_index_weight_3` | 200 | score | 1077 |
| `power_equipment_str_index_weight_4` | 15 | score | 1077 |
| `power_equipment_powers_threshold_2` | 30 | count | 1079 |
| `power_equipment_str_index_weight_5` | 250 | score | 1080 |
| `power_equipment_powers_threshold_3` | 10 | count | 1082 |
| `power_equipment_str_index_weight_6` | 250 | score | 1083 |
| `power_equipment_powers_threshold_4` | 50 | count | 1085 |
| `power_equipment_str_index_weight_7` | 250 | score | 1086 |
| `power_equipment_mega_hack_penalize_heavy_armor_which_h_threshold` | 300 | mana | 1109 |
| `power_equipment_mega_hack_penalize_heavy_armor_which_h_threshold_2` | 350 | mana | 1109 |
| `power_equipment_mega_hack_penalize_heavy_armor_which_h_weight` | 400 | score | 1110 |
| `power_equipment_mega_hack_penalize_heavy_armor_which_h_threshold_3` | 200 | mana | 1112 |
| `power_equipment_mega_hack_penalize_heavy_armor_which_h_threshold_4` | 299 | mana | 1112 |
| `power_equipment_mega_hack_penalize_heavy_armor_which_h_weight_2` | 800 | score | 1113 |
| `power_equipment_mega_hack_penalize_heavy_armor_which_h_threshold_5` | 100 | mana | 1115 |
| `power_equipment_mega_hack_penalize_heavy_armor_which_h_threshold_6` | 199 | mana | 1115 |
| `power_equipment_mega_hack_penalize_heavy_armor_which_h_weight_3` | 1600 | score | 1116 |
| `power_equipment_mega_hack_penalize_heavy_armor_which_h_threshold_7` | 99 | mana | 1118 |
| `power_equipment_mega_hack_penalize_heavy_armor_which_h_weight_4` | 3200 | score | 1119 |
| `power_inventory_afuel_limit` | 6 | count | 1146 |
| `power_inventory_afuel_weight` | 60000 | score | 1146 |
| `power_inventory_str_threshold` | 15 | stat | 1147 |
| `power_inventory_reward_fuel_threshold` | 10 | count | 1148 |
| `power_inventory_food_limit` | 7 | count | 1160 |
| `power_inventory_food_weight` | 50000 | score | 1160 |
| `power_inventory_str_threshold_2` | 15 | stat | 1161 |
| `power_inventory_food_limit_2` | 10 | count | 1162 |
| `power_inventory_food_weight_2` | 200 | score | 1162 |
| `power_inventory_clevel_threshold` | 15 | level | 1164 |
| `power_inventory_food_limit_3` | 15 | count | 1166 |
| `power_inventory_food_weight_3` | 700 | score | 1166 |
| `power_inventory_food_hi_limit` | 7 | count | 1171 |
| `power_inventory_food_hi_weight` | 52 | score | 1171 |
| `power_inventory_food_lo_limit` | 15 | count | 1173 |
| `power_inventory_food_lo_weight` | 2 | score | 1173 |
| `power_inventory_acsw_limit` | 5 | count | 1186 |
| `power_inventory_acsw_weight` | 25000 | score | 1186 |
| `power_inventory_acurepois_threshold` | 20 | count | 1193 |
| `power_inventory_arespois_limit` | 4 | count | 1195 |
| `power_inventory_arespois_weight` | 300 | score | 1195 |
| `power_inventory_maxdepth_threshold` | 20 | depth | 1199 |
| `power_inventory_aresheat_limit` | 4 | count | 1204 |
| `power_inventory_aresheat_weight` | 500 | score | 1204 |
| `power_inventory_arescold_limit` | 4 | count | 1209 |
| `power_inventory_arescold_weight` | 500 | score | 1209 |
| `power_inventory_arespois_limit_2` | 4 | count | 1214 |
| `power_inventory_arespois_weight_2` | 500 | score | 1214 |
| `power_inventory_clevel_threshold_2` | 10 | level | 1221 |
| `power_inventory_aid_limit` | 5 | count | 1222 |
| `power_inventory_aid_weight` | 6000 | score | 1222 |
| `power_inventory_str_threshold_3` | 15 | stat | 1223 |
| `power_inventory_aid_limit_2` | 15 | count | 1224 |
| `power_inventory_aid_weight_2` | 600 | score | 1224 |
| `power_inventory_apfe_limit` | 10 | count | 1239 |
| `power_inventory_apfe_weight` | 10000 | score | 1239 |
| `power_inventory_apfe_limit_2` | 25 | count | 1240 |
| `power_inventory_apfe_weight_2` | 2000 | score | 1240 |
| `power_inventory_aglyph_limit` | 10 | count | 1244 |
| `power_inventory_aglyph_weight` | 10000 | score | 1244 |
| `power_inventory_aglyph_limit_2` | 25 | count | 1245 |
| `power_inventory_aglyph_weight_2` | 2000 | score | 1245 |
| `power_inventory_maxdepth_threshold_2` | 100 | depth | 1246 |
| `power_inventory_aglyph_limit_3` | 10 | count | 1248 |
| `power_inventory_aglyph_weight_3` | 2500 | score | 1248 |
| `power_inventory_aglyph_limit_4` | 75 | count | 1249 |
| `power_inventory_aglyph_weight_4` | 2500 | score | 1249 |
| `power_inventory_maxdepth_threshold_3` | 100 | depth | 1253 |
| `power_inventory_amassban_limit` | 99 | count | 1255 |
| `power_inventory_amassban_weight` | 2500 | score | 1255 |
| `power_inventory_clevel_threshold_3` | 7 | level | 1260 |
| `power_inventory_recall_limit` | 3 | count | 1261 |
| `power_inventory_recall_weight` | 50000 | score | 1261 |
| `power_inventory_str_threshold_4` | 15 | stat | 1262 |
| `power_inventory_recall_limit_2` | 7 | count | 1263 |
| `power_inventory_recall_weight_2` | 5000 | score | 1263 |
| `power_inventory_maxdepth_threshold_4` | 50 | depth | 1266 |
| `power_inventory_aphase_limit` | 8 | count | 1278 |
| `power_inventory_aphase_weight` | 500 | score | 1278 |
| `power_inventory_str_threshold_5` | 15 | stat | 1279 |
| `power_inventory_aphase_limit_2` | 15 | count | 1280 |
| `power_inventory_aphase_weight_2` | 500 | score | 1280 |
| `power_inventory_aescape_limit` | 2 | count | 1287 |
| `power_inventory_aescape_weight` | 10000 | score | 1287 |
| `power_inventory_maxdepth_threshold_5` | 70 | depth | 1288 |
| `power_inventory_aescape_limit_2` | 3 | count | 1290 |
| `power_inventory_aescape_weight_2` | 10000 | score | 1290 |
| `power_inventory_clevel_threshold_4` | 3 | level | 1296 |
| `power_inventory_clevel_threshold_5` | 7 | level | 1299 |
| `power_inventory_ateleport_limit` | 3 | count | 1300 |
| `power_inventory_ateleport_weight` | 10000 | score | 1300 |
| `power_inventory_clevel_threshold_6` | 30 | level | 1302 |
| `power_inventory_ateleport_limit_2` | 10 | count | 1303 |
| `power_inventory_ateleport_weight_2` | 10000 | score | 1303 |
| `power_inventory_clevel_threshold_7` | 15 | level | 1308 |
| `power_inventory_ateleportlvl_limit` | 5 | count | 1309 |
| `power_inventory_ateleportlvl_weight` | 5000 | score | 1309 |
| `power_inventory_maxdepth_threshold_6` | 46 | depth | 1315 |
| `power_inventory_aezheal_limit` | 1 | count | 1317 |
| `power_inventory_aezheal_weight` | 10000 | score | 1317 |
| `power_inventory_aezheal_limit_2` | 2 | count | 1319 |
| `power_inventory_aezheal_weight_2` | 10000 | score | 1319 |
| `power_inventory_aheal_limit` | 15 | count | 1329 |
| `power_inventory_aheal_weight` | 8000 | score | 1329 |
| `power_inventory_rod_healing_limit` | 6 | count | 1335 |
| `power_inventory_rod_healing_weight` | 20000 | score | 1335 |
| `power_inventory_aheal_limit_2` | 10 | count | 1342 |
| `power_inventory_aheal_weight_2` | 4000 | score | 1342 |
| `power_inventory_potion_healing_limit` | 3 | count | 1349 |
| `power_inventory_potion_healing_weight` | 5000 | score | 1349 |
| `power_inventory_rod_healing_limit_2` | 4 | count | 1354 |
| `power_inventory_rod_healing_weight_2` | 20000 | score | 1354 |
| `power_inventory_clevel_threshold_8` | 1 | level | 1359 |
| `power_inventory_potion_healing_limit_2` | 10 | count | 1361 |
| `power_inventory_potion_healing_weight_2` | 2000 | score | 1361 |
| `power_inventory_potion_healing_limit_3` | 5 | count | 1365 |
| `power_inventory_potion_healing_weight_3` | 2000 | score | 1365 |
| `power_inventory_maxdepth_threshold_7` | 99 | depth | 1373 |
| `power_inventory_potion_healing_limit_4` | 99 | count | 1375 |
| `power_inventory_potion_healing_weight_4` | 8000 | score | 1375 |
| `power_inventory_aezheal_limit_3` | 99 | count | 1377 |
| `power_inventory_aezheal_weight_3` | 10000 | score | 1377 |
| `power_inventory_aspeed_limit` | 99 | count | 1379 |
| `power_inventory_aspeed_weight` | 8000 | score | 1379 |
| `power_inventory_alife_limit` | 99 | count | 1381 |
| `power_inventory_alife_weight` | 10000 | score | 1381 |
| `power_inventory_potion_restore_mana_limit` | 99 | count | 1384 |
| `power_inventory_potion_restore_mana_weight` | 5000 | score | 1384 |
| `power_inventory_mush_stoneskin_limit` | 40 | count | 1387 |
| `power_inventory_mush_stoneskin_weight` | 5000 | score | 1387 |
| `power_inventory_amassban_limit_2` | 99 | count | 1393 |
| `power_inventory_amassban_weight_2` | 2500 | score | 1393 |
| `power_inventory_maxsp_threshold` | 100 | mana | 1398 |
| `power_inventory_potion_restore_mana_limit_2` | 10 | count | 1400 |
| `power_inventory_potion_restore_mana_weight_2` | 4000 | score | 1400 |
| `power_inventory_astfmagi_limit` | 100 | count | 1402 |
| `power_inventory_astfmagi_weight` | 4000 | score | 1402 |
| `power_inventory_clevel_threshold_9` | 35 | level | 1406 |
| `power_inventory_clevel_threshold_10` | 10 | level | 1406 |
| `power_inventory_accw_limit` | 10 | count | 1408 |
| `power_inventory_accw_weight` | 5000 | score | 1408 |
| `power_inventory_clevel_threshold_11` | 35 | level | 1409 |
| `power_inventory_accw_limit_2` | 10 | count | 1412 |
| `power_inventory_accw_weight_2` | 5000 | score | 1412 |
| `power_inventory_str_threshold_6` | 15 | stat | 1413 |
| `power_inventory_accw_limit_3` | 15 | count | 1414 |
| `power_inventory_accw_weight_3` | 500 | score | 1414 |
| `power_inventory_accw_threshold` | 5 | count | 1419 |
| `power_inventory_maxclevel_threshold` | 10 | level | 1419 |
| `power_inventory_clevel_threshold_12` | 35 | level | 1419 |
| `power_inventory_acsw_limit_2` | 7 | count | 1421 |
| `power_inventory_acsw_weight_2` | 50 | score | 1421 |
| `power_inventory_str_threshold_7` | 15 | stat | 1422 |
| `power_inventory_acsw_limit_3` | 10 | count | 1423 |
| `power_inventory_acsw_weight_3` | 5 | score | 1423 |
| `power_inventory_acsw_threshold` | 5 | count | 1428 |
| `power_inventory_clevel_threshold_13` | 8 | level | 1428 |
| `power_inventory_aclw_limit` | 5 | count | 1430 |
| `power_inventory_aclw_weight` | 550 | score | 1430 |
| `power_inventory_maxclevel_threshold_2` | 10 | level | 1435 |
| `power_inventory_food_cure_conf_limit` | 10 | count | 1439 |
| `power_inventory_food_cure_conf_weight` | 400 | score | 1439 |
| `power_inventory_food_cure_blind_limit` | 5 | count | 1445 |
| `power_inventory_food_cure_blind_weight` | 300 | score | 1445 |
| `power_inventory_acurepois_limit` | 5 | count | 1451 |
| `power_inventory_acurepois_weight` | 250 | score | 1451 |
| `power_inventory_adettrap_limit` | 1 | count | 1459 |
| `power_inventory_adettrap_weight` | 4000 | score | 1459 |
| `power_inventory_adetdoor_limit` | 1 | count | 1463 |
| `power_inventory_adetdoor_weight` | 2000 | score | 1463 |
| `power_inventory_adetevil_limit` | 1 | count | 1468 |
| `power_inventory_adetevil_weight` | 1000 | score | 1468 |
| `power_inventory_amagicmap_limit` | 1 | count | 1473 |
| `power_inventory_amagicmap_weight` | 4000 | score | 1473 |
| `power_inventory_alite_limit` | 1 | count | 1478 |
| `power_inventory_alite_weight` | 1000 | score | 1478 |
| `power_inventory_maxdepth_threshold_8` | 100 | depth | 1482 |
| `power_inventory_scroll_mass_banishment_limit` | 10 | count | 1484 |
| `power_inventory_scroll_mass_banishment_weight` | 10000 | score | 1484 |
| `power_inventory_scroll_mass_banishment_limit_2` | 25 | count | 1485 |
| `power_inventory_scroll_mass_banishment_weight_2` | 2000 | score | 1485 |
| `power_inventory_aspeed_limit_2` | 20 | count | 1491 |
| `power_inventory_aspeed_weight_2` | 5000 | score | 1491 |
| `power_inventory_maxdepth_threshold_9` | 99 | depth | 1495 |
| `power_inventory_amissiles_limit` | 40 | count | 1504 |
| `power_inventory_amissiles_weight` | 100 | score | 1504 |
| `power_inventory_str_threshold_8` | 15 | stat | 1505 |
| `power_inventory_str_threshold_9` | 18 | stat | 1505 |
| `power_inventory_amissiles_limit_2` | 80 | count | 1506 |
| `power_inventory_amissiles_weight_2` | 10 | score | 1506 |
| `power_inventory_str_threshold_10` | 18 | stat | 1508 |
| `power_inventory_amissiles_limit_3` | 180 | count | 1509 |
| `power_inventory_amissiles_weight_3` | 8 | score | 1509 |
| `power_inventory_penalize_use_of_too_many_quiver_slots_threshold` | 4 | count | 1513 |
| `power_inventory_amissiles_limit_4` | 20 | count | 1520 |
| `power_inventory_amissiles_weight_4` | 100 | score | 1520 |
| `power_inventory_str_threshold_11` | 15 | stat | 1521 |
| `power_inventory_amissiles_limit_5` | 50 | count | 1522 |
| `power_inventory_amissiles_weight_5` | 10 | score | 1522 |
| `power_inventory_str_threshold_12` | 15 | stat | 1525 |
| `power_inventory_amissiles_threshold` | 20 | count | 1525 |
| `power_inventory_penalize_use_of_too_many_quiver_slots_threshold_2` | 2 | count | 1529 |
| `power_inventory_astfdest_limit` | 9 | count | 1549 |
| `power_inventory_astfdest_weight` | 200 | score | 1549 |
| `power_inventory_atportother_limit` | 15 | count | 1561 |
| `power_inventory_atportother_weight` | 5000 | score | 1561 |
| `power_inventory_maxdepth_threshold_10` | 30 | depth | 1565 |
| `power_inventory_cdepth_threshold` | 30 | depth | 1569 |
| `power_inventory_clevel_threshold_14` | 20 | level | 1573 |
| `power_inventory_good_s_chg_limit` | 3 | count | 1589 |
| `power_inventory_good_s_chg_weight` | 500 | score | 1589 |
| `power_inventory_arod1_limit` | 6 | count | 1593 |
| `power_inventory_arod1_weight` | 8000 | score | 1593 |
| `power_inventory_arod2_limit` | 6 | count | 1595 |
| `power_inventory_arod2_weight` | 12000 | score | 1595 |
| `power_inventory_aench_arm_threshold` | 1000 | count | 1662 |
| `power_inventory_aench_toh_threshold` | 1000 | count | 1667 |
| `power_inventory_aench_tod_threshold` | 1000 | count | 1672 |
| `power_inventory_empty_limit` | 6 | count | 1688 |
| `power_inventory_empty_weight` | 40 | score | 1688 |
| `power_inventory_maxdepth_threshold_11` | 40 | depth | 1696 |
| `power_inventory_maxdepth_threshold_12` | 25 | depth | 1697 |
| `power_inventory_gold_threshold` | 100000 | gold | 1698 |
| `power_inventory_ignore_difficult_normal_books_threshold` | 5 | level | 1759 |
| `power_inventory_maxclevel_threshold_3` | 2 | level | 1759 |
| `power_inventory_reward_the_book_limit` | 1 | count | 1766 |
| `power_inventory_reward_the_book_weight` | 500000 | score | 1766 |
| `power_inventory_str_threshold_13` | 5 | stat | 1767 |
| `power_inventory_reward_the_book_limit_2` | 2 | count | 1768 |
| `power_inventory_reward_the_book_weight_2` | 10000 | score | 1768 |
| `power_inventory_aench_arm_threshold_2` | 1000 | count | 1800 |
| `power_inventory_aench_toh_threshold_2` | 1000 | count | 1803 |
| `power_inventory_aench_tod_threshold_2` | 1000 | count | 1806 |
| `power_prepared_depth_weight` | 40000 | score | 1856 |
| `power_equipment_unidentified_weapon_max_depth` | 10 | depth | 320 |
| `power_equipment_unidentified_weapon_max_level` | 15 | level | 320 |
| `power_equipment_unidentified_bow_max_depth` | 10 | depth | 453 |
| `power_equipment_unidentified_bow_max_level` | 15 | level | 453 |
| `power_equipment_bow_enchant_floor` | 8 | damage | 461 |
| `power_equipment_light_penalty` | 10000 | score | 511 |
| `power_equipment_sp_warrior_divisor` | 2 | divisor | 595 |
| `power_equipment_sp_warrior_weight` | 255 | score | 595 |
| `power_equipment_sp_caster_divisor` | 2 | divisor | 597 |
| `power_equipment_sp_caster_weight` | 155 | score | 597 |
| `power_equipment_bow_enchant_threshold` | 8 | damage | 458 |
| `power_equipment_bow_enchant_min_level` | 25 | level | 458 |
| `power_equipment_hold_life_depth` | 46 | depth | 864 |
| `power_prepared_depth_lookahead` | 50 | depth | 1849 |
| `power_equipment_spell_reliability_weight` | 100 | score | 603 |

Power adds 727 parameters for **1734 total**. Source lines refer to
`borg_power.rs` at `5d9abb8f`; every selected literal has its own stable
registry entry. Suffixes distinguish separate existing tiers/branches; the
source line identifies the exact term. Activation entries with an existing
zero valuation can also be calibrated, without changing effect IDs.

Retained data: equipment slots, class/trait/activation identifiers, array
sizes and loop/index increments, absent/invalid sentinels, raw-stat encoding
18+100 and final stat index37, level50's completed-experience condition,
slay/brand damage multipliers and fixed half-damage normalization, copied
activation effect damage formulas, percentage complement100, book/spell
iteration sizes and unknown-level99, item weight units (10), spell-point
units (100), and the engine's half-carry encumbrance rule with tenth-carry
steps. First-item/index baselines remain categorical; tier quantity limits,
weights, speed/armour breakpoints and strategic readiness bonuses are named.
The legacy C int/long wrapping arithmetic is retained, including overflow
semantics; no new arithmetic simplification or sorting changes were made.
Input bounds require positive divisors, cap readiness lookahead at the
existing50 levels and speed inputs at199, and bound count loops.

Read equipment chapter6 (§§2,19), combat chapter4 and Borg notes §15.10.
Engine authority: `src/player-calcs.c` (carry, mana/weight and stat indexing),
`src/player-attack.c` (slays/brands), `lib/gamedata/class.txt`, `object.txt`,
`activation.txt` and `slay.txt`/`brand.txt`. Reward magnitudes and creature
frequency weights are hypotheses, not game facts or recommendations. The
copied activation damage estimates remain legacy model data, not a claim
of exact agreement with every current effect.

The power weights govern static valuation (`borg_uses_dynamic_calcs = FALSE`).
The existing dynamic formula subsystem and its configuration switch are
unchanged; enabling it delegates valuation to the borg.txt formula rules.

## M13 parameter migration (#328)

The earlier tables describe the historical v12 registry. M13 changes potion
healing estimate defaults to zero percent plus fixed 20/40/60 HP for
CLW/CSW/CCW and removes five parameters whose branches no longer exist.
See [M13 integration](M13-INTEGRATION.md) for the complete list and sources.
The live registry remains borg-core/src/params.rs; historical profiles are
not rewritten or repinned. Both optional town/trip policies default off.

## Class/family preference catalogue (#403)

| Name | Default | Unit | Range | Source |
| --- | ---: | --- | --- | --- |
| `attack_preference_warrior_melee_permille` | 1000 | utility_permille | 500–2000 | `borg_fight_attack.rs:4575` (#403 ranking seam) |
| `attack_preference_warrior_projectile_permille` | 1000 | utility_permille | 500–2000 | `borg_fight_attack.rs:4575` (#403 ranking seam) |
| `attack_preference_warrior_thrown_permille` | 1000 | utility_permille | 500–2000 | `borg_fight_attack.rs:4575` (#403 ranking seam) |
| `attack_preference_warrior_spell_permille` | 1000 | utility_permille | 500–2000 | `borg_fight_attack.rs:4575` (#403 ranking seam) |
| `attack_preference_warrior_device_permille` | 1000 | utility_permille | 500–2000 | `borg_fight_attack.rs:4575` (#403 ranking seam) |
| `attack_preference_warrior_activation_permille` | 1000 | utility_permille | 500–2000 | `borg_fight_attack.rs:4575` (#403 ranking seam) |
| `attack_preference_mage_melee_permille` | 1000 | utility_permille | 500–2000 | `borg_fight_attack.rs:4575` (#403 ranking seam) |
| `attack_preference_mage_projectile_permille` | 1000 | utility_permille | 500–2000 | `borg_fight_attack.rs:4575` (#403 ranking seam) |
| `attack_preference_mage_thrown_permille` | 1000 | utility_permille | 500–2000 | `borg_fight_attack.rs:4575` (#403 ranking seam) |
| `attack_preference_mage_spell_permille` | 1000 | utility_permille | 500–2000 | `borg_fight_attack.rs:4575` (#403 ranking seam) |
| `attack_preference_mage_device_permille` | 1000 | utility_permille | 500–2000 | `borg_fight_attack.rs:4575` (#403 ranking seam) |
| `attack_preference_mage_activation_permille` | 1000 | utility_permille | 500–2000 | `borg_fight_attack.rs:4575` (#403 ranking seam) |
| `attack_preference_druid_melee_permille` | 1000 | utility_permille | 500–2000 | `borg_fight_attack.rs:4575` (#403 ranking seam) |
| `attack_preference_druid_projectile_permille` | 1000 | utility_permille | 500–2000 | `borg_fight_attack.rs:4575` (#403 ranking seam) |
| `attack_preference_druid_thrown_permille` | 1000 | utility_permille | 500–2000 | `borg_fight_attack.rs:4575` (#403 ranking seam) |
| `attack_preference_druid_spell_permille` | 1000 | utility_permille | 500–2000 | `borg_fight_attack.rs:4575` (#403 ranking seam) |
| `attack_preference_druid_device_permille` | 1000 | utility_permille | 500–2000 | `borg_fight_attack.rs:4575` (#403 ranking seam) |
| `attack_preference_druid_activation_permille` | 1000 | utility_permille | 500–2000 | `borg_fight_attack.rs:4575` (#403 ranking seam) |
| `attack_preference_priest_melee_permille` | 1000 | utility_permille | 500–2000 | `borg_fight_attack.rs:4575` (#403 ranking seam) |
| `attack_preference_priest_projectile_permille` | 1000 | utility_permille | 500–2000 | `borg_fight_attack.rs:4575` (#403 ranking seam) |
| `attack_preference_priest_thrown_permille` | 1000 | utility_permille | 500–2000 | `borg_fight_attack.rs:4575` (#403 ranking seam) |
| `attack_preference_priest_spell_permille` | 1000 | utility_permille | 500–2000 | `borg_fight_attack.rs:4575` (#403 ranking seam) |
| `attack_preference_priest_device_permille` | 1000 | utility_permille | 500–2000 | `borg_fight_attack.rs:4575` (#403 ranking seam) |
| `attack_preference_priest_activation_permille` | 1000 | utility_permille | 500–2000 | `borg_fight_attack.rs:4575` (#403 ranking seam) |
| `attack_preference_necromancer_melee_permille` | 1000 | utility_permille | 500–2000 | `borg_fight_attack.rs:4575` (#403 ranking seam) |
| `attack_preference_necromancer_projectile_permille` | 1000 | utility_permille | 500–2000 | `borg_fight_attack.rs:4575` (#403 ranking seam) |
| `attack_preference_necromancer_thrown_permille` | 1000 | utility_permille | 500–2000 | `borg_fight_attack.rs:4575` (#403 ranking seam) |
| `attack_preference_necromancer_spell_permille` | 1000 | utility_permille | 500–2000 | `borg_fight_attack.rs:4575` (#403 ranking seam) |
| `attack_preference_necromancer_device_permille` | 1000 | utility_permille | 500–2000 | `borg_fight_attack.rs:4575` (#403 ranking seam) |
| `attack_preference_necromancer_activation_permille` | 1000 | utility_permille | 500–2000 | `borg_fight_attack.rs:4575` (#403 ranking seam) |
| `attack_preference_paladin_melee_permille` | 1000 | utility_permille | 500–2000 | `borg_fight_attack.rs:4575` (#403 ranking seam) |
| `attack_preference_paladin_projectile_permille` | 1000 | utility_permille | 500–2000 | `borg_fight_attack.rs:4575` (#403 ranking seam) |
| `attack_preference_paladin_thrown_permille` | 1000 | utility_permille | 500–2000 | `borg_fight_attack.rs:4575` (#403 ranking seam) |
| `attack_preference_paladin_spell_permille` | 1000 | utility_permille | 500–2000 | `borg_fight_attack.rs:4575` (#403 ranking seam) |
| `attack_preference_paladin_device_permille` | 1000 | utility_permille | 500–2000 | `borg_fight_attack.rs:4575` (#403 ranking seam) |
| `attack_preference_paladin_activation_permille` | 1000 | utility_permille | 500–2000 | `borg_fight_attack.rs:4575` (#403 ranking seam) |
| `attack_preference_rogue_melee_permille` | 1000 | utility_permille | 500–2000 | `borg_fight_attack.rs:4575` (#403 ranking seam) |
| `attack_preference_rogue_projectile_permille` | 1000 | utility_permille | 500–2000 | `borg_fight_attack.rs:4575` (#403 ranking seam) |
| `attack_preference_rogue_thrown_permille` | 1000 | utility_permille | 500–2000 | `borg_fight_attack.rs:4575` (#403 ranking seam) |
| `attack_preference_rogue_spell_permille` | 1000 | utility_permille | 500–2000 | `borg_fight_attack.rs:4575` (#403 ranking seam) |
| `attack_preference_rogue_device_permille` | 1000 | utility_permille | 500–2000 | `borg_fight_attack.rs:4575` (#403 ranking seam) |
| `attack_preference_rogue_activation_permille` | 1000 | utility_permille | 500–2000 | `borg_fight_attack.rs:4575` (#403 ranking seam) |
| `attack_preference_ranger_melee_permille` | 1000 | utility_permille | 500–2000 | `borg_fight_attack.rs:4575` (#403 ranking seam) |
| `attack_preference_ranger_projectile_permille` | 1000 | utility_permille | 500–2000 | `borg_fight_attack.rs:4575` (#403 ranking seam) |
| `attack_preference_ranger_thrown_permille` | 1000 | utility_permille | 500–2000 | `borg_fight_attack.rs:4575` (#403 ranking seam) |
| `attack_preference_ranger_spell_permille` | 1000 | utility_permille | 500–2000 | `borg_fight_attack.rs:4575` (#403 ranking seam) |
| `attack_preference_ranger_device_permille` | 1000 | utility_permille | 500–2000 | `borg_fight_attack.rs:4575` (#403 ranking seam) |
| `attack_preference_ranger_activation_permille` | 1000 | utility_permille | 500–2000 | `borg_fight_attack.rs:4575` (#403 ranking seam) |
| `attack_preference_blackguard_melee_permille` | 1000 | utility_permille | 500–2000 | `borg_fight_attack.rs:4575` (#403 ranking seam) |
| `attack_preference_blackguard_projectile_permille` | 1000 | utility_permille | 500–2000 | `borg_fight_attack.rs:4575` (#403 ranking seam) |
| `attack_preference_blackguard_thrown_permille` | 1000 | utility_permille | 500–2000 | `borg_fight_attack.rs:4575` (#403 ranking seam) |
| `attack_preference_blackguard_spell_permille` | 1000 | utility_permille | 500–2000 | `borg_fight_attack.rs:4575` (#403 ranking seam) |
| `attack_preference_blackguard_device_permille` | 1000 | utility_permille | 500–2000 | `borg_fight_attack.rs:4575` (#403 ranking seam) |
| `attack_preference_blackguard_activation_permille` | 1000 | utility_permille | 500–2000 | `borg_fight_attack.rs:4575` (#403 ranking seam) |

## Speed for a dangerous fight (#469)

Owner, 2026-11-17: when the danger is very high and the Borg fights rather
than retreats, drink a Potion of Speed. `borg_defend_aux_speed` drinks only
when being fast would bring the danger under a fraction of avoidance, so it
skips exactly the fights speed cannot make safe. In `borg_attack`, once a
positive best attack other than waiting is chosen (the Borg has committed to
fighting this turn: `borg_caution` took no action), the Borg drinks a carried
Potion of Speed first when it is in the dungeon, not already fast, and the
danger where it stands from the monsters that can move (`borg_danger_one_kill`
summed over every monster without NEVER_MOVE, `defend_danger_horizon`) is
above avoidance x value / 10. Never-moving monsters (mushroom patches, floating
eyes) are left out: the Borg can step away, and in the first 180-start trial
their spores or gaze pushed the danger past every threshold tried. The note is `# Speed for a dangerous fight (danger X of
avoidance Y)`; the attack follows next turn. The potion is `TIMED_INC:FAST`,
20+2d10 turns, +10 speed (`object.txt`, Potion of Speed).

| parameter | default | unit | range | source |
|---|---|---|---|---|
| `attack_speed_fight_danger_tenths` | 0 (off) | tenths | 0–1000 | `borg_fight_attack.rs:5056` |

## C Borg dive gates (#768)

`prepared_clevel_depth_margin` is the main dive-speed control. Below
character level 50, static preparedness requires maximum CL ≥ DL − margin.
Positive values let the Borg qualify for deeper levels. The check follows
restock and applies even with `borg_plays_risky` on. At CL50 it stops
applying. `borg-prepared.c:borg_prepared:560-562` at `574b618c`.

The class gates skip their checks in risky play. DL2 starts a maximum-HP
requirement. DL3 adds class HP and CL requirements, plus a combined minimum of
two CLW, CSW and CCW potions below CL30, including in risky play. DL5 adds
class HP and CL requirements when the Borg is already in the dungeon.
These checks still apply at later depths that reach their code.
`borg-prepared.c:borg_prepared_aux:82-200` at `574b618c`.

The offset check starts at DL10, after the light and escape checks. For
Mage and Necromancer, it applies through CL28, including checks for depths
past DL19. For the other classes, it applies through DL19. The DL21 class
gate has its own offset and inclusive CL bounds. Above their DL21 upper CL
bound, Mage and Necromancer require DL ≤ 30 + 2 × (CL − 38) through CL44.
DL34 adds a minimum of CL40. `borg-prepared.c:borg_prepared_aux:221-253`
and `304-377` at `574b618c`.

| Class | DL3 HP / CL | DL5 HP / CL | DL10 offset | DL21 offset / min CL / max CL |
|---|---:|---:|---:|---:|
| Warrior, Blackguard | 50 / 4 | 60 / 6 | −4 | +5 / 0 / 38 |
| Mage, Necromancer | 60 / 11 | 80 / 15 | +5 | +8 / 0 / 38 |
| Druid, Priest | 40 / 9 | 60 / 15 | 0 | +13 / 0 / 46 |
| Paladin | 50 / 4 | 60 / 6 | 0 | +7 / 0 / 40 |
| Rogue | 50 / 8 | 60 / 10 | 0 | +10 / 0 / 43 |
| Ranger | 50 / 4 | 60 / 6 | 0 | +8 / 29 / 41 |

Compiled and shipped values match the C checks. The files
`harness/borg_<class>.txt` supply the class values folded as
`borg_param_<name>__<class>` in `harness/borg.txt`. Set that class-specific
key to sweep an offset in the zoo. The optional `borg_plan` overlay and
`borg_no_deeper` still provide explicit depth limits. Profiles with the
retired #693 settings load with a warning and use these checks.

| Parameter | Default | Unit | Range | Source |
|---|---|---|---|---|
| `prepared_clevel_depth_margin` | 0 | levels | -50–50 | `borg_prepared.rs:675` |
| `prepared_dl2_min_hp` | 30 | hp | 0–10000 | `borg_prepared.rs:229` |
| `prepared_dl34_min_hp` | By class | hp | 0–10000 | `borg_prepared.rs:243` |
| `prepared_dl34_min_clevel` | By class | level | 0–50 | `borg_prepared.rs:247` |
| `prepared_cure_min` | 2 | potions | 0–99 | `borg_prepared.rs:255` |
| `prepared_cure_until_clevel` | 30 | level | 0–50 | `borg_prepared.rs:256` |
| `prepared_dl59_min_hp` | By class | hp | 0–10000 | `borg_prepared.rs:271` |
| `prepared_dl59_min_clevel` | By class | level | 0–50 | `borg_prepared.rs:275` |
| `prepared_mid_clevel_offset` | By class | levels | -50–50 | `borg_prepared.rs:318` |
| `prepared_mid_max_depth` | 19 | depth | 0–127 | `borg_prepared.rs:316` |
| `prepared_mid_caster_max_clevel` | 28 | level | 0–50 | `borg_prepared.rs:314` |
| `prepared_deep_clevel_offset` | By class | levels | -50–50 | `borg_prepared.rs:386` |
| `prepared_deep_min_clevel` | By class | level | 0–50 | `borg_prepared.rs:387` |
| `prepared_deep_max_clevel` | By class | level | 0–50 | `borg_prepared.rs:385` |
| `prepared_deep_caster_ramp_max_clevel` | 44 | level | 0–50 | `borg_prepared.rs:393` |
| `prepared_deep_caster_ramp_base_depth` | 30 | depth | 0–127 | `borg_prepared.rs:394` |
| `prepared_deep_caster_ramp_depth_per_level` | 2 | levels | 0–127 | `borg_prepared.rs:395` |
| `prepared_dl34_plus_min_clevel` | 40 | level | 0–50 | `borg_prepared.rs:438` |

## Fox Form's study rating (#694)

The registry now holds **2212** parameters (2211 on main before it; one added).

`magic_fox_form_study_rating` (compiled and shipped 0 = C's rating 5): above 0,
the rating Fox Form is studied by (`borg_magic::study_rating`). C's 5 is below
the 50 a spell needs unless the Borg is bored, so a Druid learns it once bored;
100 puts it above Detect Life and Stinking Cloud (95), learned first. On 6
Druid starts (100k turns) first cost levels: sum of max CL 45 → 33, one death.

## Cornered: attack at range before the boost (#719)

The registry now holds **2222** parameters (2221 on main before it; one added).

Owner (2026-11-20): "if I am cornered I should throw missile and then boost
myself". `cornered_boost` (#504) runs in `borg_caution`, before
`borg_attack`, so a cornered Borg always buffs first.

`caution_cornered_throw_first` (compiled 0, shipped 1, owner 2026-11-20):
- **1:** in the cornered boost, while no monster is adjacent and an attack
  from range would do damage (the attack planner's dry run,
  `borg_attack_estimate`, with the Borg's random numbers and the waiting flag
  restored), the Borg attacks first. It buffs once a monster is next to it,
  or when nothing reaches.
- **2:** the same for the hard breeder fight's boost. (The Phase Door rule's
  boost is reached only after its own `borg_attack` found nothing to do, so
  there mode 2 adds nothing: that site already attacks first.)

**Local check** (100,000 turns, main 2026-11-20):

| starts | mode | deaths | sum of max CL | sum of max DL | games as mode 0 | throws before the boost |
|---|---|---:|---:|---:|---:|---:|
| classes-18 | 0 | 0 | 109 | 40 | – | – |
| classes-18 | 1, 2 | 0 | 109 | 40 | 18 / 18 | 0 |
| classes-1800 starts 1–90 | 0 | 4 | 517 | 206 | – | – |
| classes-1800 starts 1–90 | 2 | 4 | 518 | 206 | 89 / 90 | 4 (two Blackguard games, one Druid) |

**The shipped mode 1** (classes-1800 starts 1–90, 100k, rebuilt on main
2026-11-20 after merging it): it never acts. Modes 0 and 1 play 90/90
identical games (5 deaths, sum of CL 544, DL 213). The 4 actions of mode 2
above were all at the cornered boost, on the earlier main.

The rule is rarely reached: when the Borg is cornered (a known monster
within 2 grids), the monster is almost always already adjacent. A first
version that didn't restore the dry run's random numbers changed games where
the rule never acted; the restore fixed it (18/18 identical).

## The wait for Word of Recall on a scary level (#754)

The registry now holds **2234** parameters (2232 on main before it; two added).
In the Ranger analysis (borg-zoo#488) two games read Word of Recall on DL1 and
then had the note "Scary guy on level, no escape: standing and fighting".
That is #530's stand: a Recall that is already counting down doesn't count as
an escape. One of the games (432972050) then explored while it waited and walked
back into the cutpurse.

- **`caution_recall_pending_escape`** (flag, compiled and shipped 0): 1 = a
  Recall already counting down (`goal.recalling`) counts as an escape in
  `apply_scary_stand`, so a scary level keeps its flight. #603's gates
  (`recall_restricts`) still hold the flee flows and the down stairs, but an
  up staircase the Borg walks onto is still taken (`choose_stairs`, "Take
  stairs up"). On DL1 that is town, where the pending Recall fires and takes
  the Borg back to its maximum depth (review of #762).
- **`caution_recall_wait_no_explore`** (flag, compiled and shipped 0): 1 =
  while Recall counts down in the dungeon, `borg_think_dungeon` stops before
  its "Explore the dungeon" block, and the think ends there. That block
  covers vaults, far kills, items and veins (250), and `borg_flow_dark`. What
  follows it is skipped too: `borg_leave_level`, the shops, and C's own late
  "Wait for recall", which this wait replaces. The Borg waits where it
  stands. It rests for Recall (`caution_recall_rest_turns`, like caution's
  own site) only when `borg_check_rest` allows it there and `borg_danger` is
  0 (#567's hold first). Otherwise it stays still one turn and keeps #567's
  watch current (review of #762). Everything above the hook still runs:
  caution, attack, recovery, the #603 flight and leave under
  `caution_recall_wait_as_usual`, and the near kill and item flows.

**DL1 for `caution_recall_wait_as_usual`** (the issue's third question): not
extended. On DL1 the up staircase leads to town, where the Recall then fires
and takes the Borg back down to its maximum depth. Only the down staircase
would be safe, which would need a DL1 rule of its own (down stairs only).

**Local check** (100,000 turns; literal seeds 1–18, Human 1–9 and Half-Elf
10–18, classes 0–8 in turn, which is not the zoo's `classes-18` seed set; plus
the two Ranger starts 432972050 Half-Elf and 2049259432 Gnome; the first
version of the switches, before the review's `borg_check_rest` gate):

| arm | deaths | sum of max CL | sum of max DL | games as off | "not exploring" notes |
|---|---:|---:|---:|---:|---:|
| off | 1 | 122 | 44 | – | 0 |
| pending escape | 1 | 124 | 44 | 18 / 20 | 0 |
| no explore | 1 | 123 | 45 | 14 / 20 | 89 |
| both | 1 | 124 | 46 | 13 / 20 | 93 |

**After the review's gate** (same starts; the switches as merged):

| arm | deaths | sum of max CL | sum of max DL | games as off | rests / stays still |
|---|---:|---:|---:|---:|---:|
| off | 1 | 122 | 44 | – | – |
| pending escape | 1 | 124 | 44 | 18 / 20 | – |
| no explore | 1 | 123 | 45 | 13 / 20 | 0 / 104 |
| both | 1 | 124 | 46 | 12 / 20 | 0 / 108 |

In practice the wait only stays still. Caution's own rest for Recall
(`borg_caution`, earlier in the think) already rests whenever
`borg_check_rest` passes, so the wait's rest is left for the rare states
caution's site skips (`BI_ISHUNGRY` with food left).

- **The death is Fang (2049259432) in every arm.** At CL1 with 13 HP, Fang is
  adjacent and twice as fast, and the Borg has no escape left. The switches
  change only the note.
- **432972050 no longer dies on main** (it did on fde8025a).
- **With both switches off,** smoke-borg-rs is 3/3 identical.

| name | default | unit | range | cited |
|---|---:|---|---|---|
| `caution_recall_pending_escape` | 0 | flag | 0–1 | `borg_caution.rs:2823` |
| `caution_recall_wait_no_explore` | 0 | flag | 0–1 | `borg_caution.rs:3040` |

| `attack_speed_fight_danger_tenths` | 0 (off) | tenths | 0–1000 | `borg_fight_attack.rs:5014` |

## Enchant/decurse policy (#680 / P6)

Receiving baseline: `760fae80d3daa5c007a26e10d80f0c18eb0000ca` on
`track-p`; engine `42cd156ac`, donor `8c5d000a`. This contract does not
adopt newer main, an engine update or donor v15. The source inventory’s
explicit deferrals still apply. R4 adoption remains a separate audited carry
after owner acceptance and P6 promotion.

Read the Strategy Manual ch. 6 §15 (curses), ch. 7 §9 (enchantment) and the
official command manual; checked against the receiving engine’s
`effect-handler-general.c:enchant_score:270-287`, `enchant:319-348` and
`uncurse_object:179-209` and the receiving `borg_magic.rs` availability/failure checks.
The engine’s enchant chance table, stack-resistance formula, curse strengths,
spell failure calculation and prompts are mechanics, unchanged here. The
Borg’s willingness to spend a resource at a failure rate, stack size, bonus,
launcher quality or elapsed time is policy. Defaults are inherited choices,
not measured optima or safety recommendations.

The registry exposes ten additional settings through the existing
`borg_param_<name> = <integer>` and `BORG_PARAM_<UPPERCASE_NAME>` forms,
including existing per-class forms. Environment overrides file overrides
compiled defaults; invalid bounds/names are refused. `params-core.json`
retains name/default/value/unit/bounds/origin/source and the effective profile
hash is recorded in run configuration. No shipped profile values change.

| Name | Default | Unit | Bounds | Source |
| --- | --- | --- | --- | --- |
| `decurse_spell_fail` | 40 | percent | 0–100 | `borg_item_decurse.rs:65` |
| `enchant_spell_fail` | 65 | percent | 0–100 | `borg_item_enchant.rs:98` |
| `enchant_brand_spell_fail` | 65 | percent | 0–100 | `borg_item_enchant.rs:486` |
| `enchant_plain_target` | 8 | bonus | 0–20 | `borg_item_enchant.rs:96` |
| `enchant_ammo_target` | 10 | bonus | 0–20 | `borg_item_enchant.rs:391` |
| `enchant_min_shooter_power` | 3 | multiplier | 0–10 | `borg_item_enchant.rs:318` |
| `enchant_ammo_min_quantity` | 5 | count | 1–255 | `borg_item_enchant.rs:373` |
| `enchant_brand_min_quantity` | 20 | count | 1–255 | `borg_item_enchant.rs:447` |
| `enchant_dungeon_max_turns` | 5500 | turns | 0–100000 | `borg_item_enchant.rs:507` |
| `enchant_town_max_turns` | 3501 | turns | 0–100000 | `borg_item_enchant.rs:508` |

All comparisons remain i32 except the original u8 quantity read, which is
losslessly widened to i32 before comparing the bounded quantity setting.
No narrowing/truncation is added. The `>` timer cuts retain the original
short-circuit ordering and depth predicates (`>= 1` vs `== 0`); no depth is
reclassified. The shared spell-failure setting controls both enchant
availability and the matching casts; branding and decursing keep distinct
allowances. Original resource precedence and RNG/call order remain unchanged.

Classification of the remaining literals in the two audited files:

| Literal / shape | Classification and retained meaning |
| --- | --- |
| `BORG_ENCHANT_LIMIT` | Existing user configuration; advanced armour/equipment target unchanged. Ammo retains a separate target, as before. |
| `0` / `1` presence checks, depth town/dungeon split, boolean equality | State/availability semantics, not new policy thresholds. |
| Equipment offsets `+1`, `+6`, `+12`; `-1` and slot conversions; quiver digits; selector bytes, Enter; inventory/equipment bit masks | UI/index bookkeeping, unchanged. |
| Candidate sentinels `-1`, `99`, `0`; `<`/`>` best-item tie handling | Search initialization/order, unchanged; not enchant targets. |
| Spell/activation IDs, tvals and ego/artifact identity checks | Game/resource type and identity semantics, unchanged. |
| C duplicate `TV_HELM` entry and ineffective repeated shooter comment | Inherited ordering/comment retained; no cleanup mixed into extraction. |
| Literal values in comments, licenses, tests | Documentation / test oracles, not runtime knobs. |

`borg_enchanting` still refuses blind/confused state, performs elapsed-level
checks, tries swap-armour curse removal, swap-weapon removal and general
removal in order, and only then requires town for branding, weapon-to-hit,
weapon-to-damage and armour enchantment, in that order. `start_decurse`
still tries scroll, staff, spell, star-scroll and activations in its original
order. No direct curse-strength or engine enchant-success knob is introduced.

The existing trace@2 selection `borg_enchanting` and its queued keys remain
the decision witness; adding registry entries needs no trace schema bump.
Parameters/provenance are in params-core.json, not inferred from a missing
trace selection. The registry has boundary/precedence/provenance tests, while the bounded
native override check records an actual changed command and trace; these demonstrate control, not better play. Default
compatibility and all failures/limits are recorded in the PR against exact
receiving builds. Gameplay reports stay in Cemetery, not this document.


Validation receipt for this task: [immutable Cemetery bundle](https://github.com/diegoami/borg-cemetery/tree/0529ff6f7208b197a5ee156be180395e7fbef38e/validation/borg-reborn-rs/enchant-decurse-680)
(Cemetery PR #90, pending owner review). Clean build at receiving 760fae80
versus implementation 710ba18f passes strict full45 key/message/state
comparison and all 45 complete trace@2 comparisons. A single environment
override changes a real queued command/selection; original failures and
paired-attempt accounting remain in the canonical bundle. This verifies the
named receiving baseline only. Required reconciliation with newer accepted
main is tracked separately in #774 before P6 promotion.
