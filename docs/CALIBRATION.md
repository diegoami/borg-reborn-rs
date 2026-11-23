# Calibrating borg-rs's core (track P)

P1 exposes603 caution/danger/escape/recovery parameters. P2 adds167
attack,218 defence,19 item-use and727 power values: **1734 total**.
Defaults retain the vendored v12 policy. Input bounds support experiments;
no value is recommended or claimed to improve play. Each section below
identifies its pre-extraction source and retained mechanics/interface data.


Each section below is a snapshot: its "registry now holds N" is the count
right after that change, so older sections show older counts.

## Walking away from paralysing monsters (#1125)

The registry now holds **2479** parameters (2475 before; four added). Owner,
2026-11-23: "from paralyzing monsters it is necessary to walk away". From the
death of Half-Elf Necromancer 453254028 (CL3, 29 HP, DL1, borg-cemetery#141):
a floating eye forgotten by the anti-bounce reset was walked next to, and its
gaze (`blow:GAZE:PARALYZE`, no dice, `NEVER_MOVE`, speed 110,
`monster.txt:694-708`) kept the Borg paralysed from 29 to 0 HP. At its one
decision the Phase rule read "danger 0 without fear: the fight is not lost"
and it attacked. `mon-blows.c:1017-1025`: 3 + 1d(rlev) turns, failed by Free
Action or a saving throw, at least 1 damage on a paralysed player;
`player_timed.txt:216-222`: `NONSTACKING`. Without Free Action, standing next
to a paralyser as fast as the Borg is a fight lost at any HP. All four are
compiled and shipped 0; their values are hypotheses for the zoo (RULES 14).

- **`escape_paralyser_fight_lost`**: next to an awake known monster with a
  `PARALYZE` blow (27 races, `race_paralyses`), without Free Action
  (`BI_FRACT`), the Phase rule's fight is lost whatever the danger says, and
  the Phase Door is allowed. The early fight (`fight_before_flight`: attack
  on sight, the lone slow monster, the winning fight) waits for caution
  meanwhile (review of #1129). Notes: `# Phase rule: next to the paralysing
  <race> without Free Action: the fight is lost (#1125)`, `# Next to the
  paralysing <race> without Free Action: no early fight, caution first
  (#1125)`.
- **`caution_paralyser_step_away`**: next to a known paralyser that never
  moves (awake or asleep), without Free Action, the dungeon think's first
  action is a step to a grid out of every immobile attacker's reach and not
  more dangerous than here (`best_step_away_from`; toward the stairs with
  `caution_immobile_step_to_exit`), before the early fight, rests, cures and
  Phase Door. The anti-bounce test does not apply to it: the death's
  400-turn flow loop had raised the count past `caution_bounce_limit`, which
  refused every grid. No step count. Note: `# Stepping out of the reach of
  the paralysing <race>, no Free Action (#1125)`.
- **`caution_paralyser_no_reach`**: without Free Action, `spread_blocked`
  closes a known paralyser's grid and its eight neighbours to every flow
  (exploration, kills, takes, stair flights), moving or not, awake or asleep,
  and `borg_flow_enqueue_grid` seeds no destination there (review of #1129:
  a staircase next to the eye was still a goal); the Borg's own grid stays
  open. Shots from two grids still reach it.
- **`caution_remember_never_move`**: a `NEVER_MOVE` monster's record survives
  the anti-bounce reset (`borg_check_bounciness`, count 500-503) and the
  2000-turn expiry (`expire_kills`). Note: `# Keeping the never-moving <race>
  at (y,x) through the bounce reset (#1125)`.

The replay of 453254028 from birth (100,000 turns, the death's build
`d8b55d64` with this change) is one start, not a rate: off dies at 77,923
with the same keys; 1 alone, 4 alone, 2+3 and all four live to the cap; 2
alone and 1+2 die (the step is undone by the next flow, which the closure of
3 prevents); 3 alone plays the same game (the eye was already forgotten).

| parameter | default | unit | range | source |
|---|---|---|---|---|
| `escape_paralyser_fight_lost` | 0 | flag | 0–1 | `borg_escape.rs:1209` |
| `caution_paralyser_step_away` | 0 | flag | 0–1 | `borg_caution.rs:2802` |
| `caution_paralyser_no_reach` | 0 | flag | 0–1 | `borg_caution.rs:2752` |
| `caution_remember_never_move` | 0 | flag | 0–1 | `borg_caution.rs:2763` |

## Waiting out a bad test potion (#1060)

The registry now holds **2445** parameters (2444 before; one added). Owner,
2026-11-22: "a character drank a potion of poison and then goes happily
exploring" (seed 1928291203: a Poison test, then a 76-step walk to the stairs
at 1 HP a turn, 26 -> 18). The test's HP floor, `item_test_unknown_min_hp`,
first added here, is #1063's section below (compiled 25, shipped 60, with the
known-cure exemption).

- **`item_test_unknown_hold_turns`** (compiled 180, **shipped 180**: Blindness, the longest supported potion, lasts up to 175 ticks): after a test,
  for at most this many world ticks (10 game turns each, the clock the status
  timers run on), on the test grid and level, while a harmful status it caused
  lasts (a Slowness or Surprise's stat scramble included, read from their
  timers) and the danger here is 0,
  the Borg stays where it tested (`potion_test_hold`, right after
  `borg_caution`): the ordinary in-place recovery (`borg_recover`) first, else
  it stands still (`,`). `potion_test_hold_validate`, before the early fight
  (`fight_before_flight`) and `borg_caution`, ends the hold on danger, a move
  or a new level. 0 = no hold. The cap bounds when a hold decision can start:
  a recovery rest (`R&`) that `borg_recover` starts inside the hold runs in the
  engine until the status clears or something disturbs it, past the cap if
  need be, as the same rest chosen by the ordinary order would.

| parameter | default | unit | range | source |
|---|---|---|---|---|
| `item_test_unknown_hold_turns` | 180 | turns | 0–1000 | `borg_item_use.rs:254` |

## Poisoned without a cure: escapes before the one-grid retreat (#934)

The registry now holds **2442** parameters (2441 before; one added). From the death of the
Half-Troll Ranger 829474344 (borg-zoo run 44950087466; Cemetery study
`studies/fastfrag-white-jelly-829474344`): a strategic retreat from a white jelly
and the next stair-flight step undid each other 74 times until poison killed it.
The zoo measures the switch (borg-zoo#654); its values are hypotheses.

- **`caution_poisoned_no_cure_retreat_hp_percent`** (percent of max HP, compiled
  and shipped 0). Poisoned, with no usable poison cure (Neutralize Poison or curing
  food, Cure Critical Wounds, Healing, *Healing*, Life, or a castable Cure Poison
  or Herbal Curing) and below that percent of the maximum HP, with an escape at
  hand (a staircase underfoot, or a Phase Door it can read or cast,
  `borg_fast_fragile::escape_available`): the escapes are tried before the
  strategic retreat; when they decline, the retreat goes on as usual, so no safe
  step is thrown away (review of #968). It uses only the Poisoned flag: the screen
  shows one grade (`player_timed.txt:255`). In 829474344 the Borg drank 12 CLW
  against poison (CLW cures none, `object.txt:3156-3173`) and never read its two
  Phase Doors. Notes: `# Poisoned, no cure, HP h of m (danger d): the escape
  before the retreat (#934)` and, when it declines, `... the escape declined, the
  retreat decides (#934)`. The estimate of the poison still to come, also asked in
  #934, is not built.

| parameter | default | unit | range | source |
|---|---|---|---|---|
| `caution_poisoned_no_cure_retreat_hp_percent` | 0 | percent of max HP | 0–100 | `borg_caution.rs:2353` |

## No retreat from a lone immobile neighbour melee kills quickly (#933)

The registry now holds **2441** parameters (2439 before; two added). From the death of the
Half-Troll Ranger 829474344 (borg-zoo run 44950087466; Cemetery study
`studies/fastfrag-white-jelly-829474344`): a strategic retreat from a white jelly
and the next stair-flight step undid each other 74 times until poison killed it.
The zoo measures the switch (borg-zoo#654); its values are hypotheses.

- **`caution_retreat_immobile_kill_turns`** (turns, compiled and shipped 0) and
  **`caution_retreat_immobile_danger_percent`** (100). The strategic retreat is not
  taken when the only known neighbour is an immobile attacker the Borg can melee
  (not afraid nor cursed with fear; the record young enough for this test and for
  the attack's own, `attack_max_target_age`), its expected melee damage a turn
  (the real hit chance and blows, without the targeting bonuses of
  `borg_thrust_damage_one`) kills it within that many turns, and the danger here
  from every monster over the fight stays below the percent of the HP (review of
  #967). It extends the one-blow exemption (`borg_adjacent_kill_in_one_blow`). In
  829474344 the 25-HP white jelly was never attacked: `borg_caution` retreats
  before `borg_attack` runs. Note: `# No retreat: immobile <race> dies in N
  turns, danger D a turn here (#933)`. Unknown wands (the other half of #933) are
  left to #918.

| parameter | default | unit | range | source |
|---|---|---|---|---|
| `caution_retreat_immobile_kill_turns` | 0 | turns | 0–100 | `borg_caution.rs:2426` |
| `caution_retreat_immobile_danger_percent` | 100 | percent of HP | 1–1000 | `borg_caution.rs:2449` |

## No retreat after a run of retreats that the next step undoes (#932)

The registry now holds **2439** parameters (2436 before; three added). From the death of the
Half-Troll Ranger 829474344 (borg-zoo run 44950087466; Cemetery study
`studies/fastfrag-white-jelly-829474344`): a strategic retreat from a white jelly
and the next stair-flight step undid each other 74 times until poison killed it.
The zoo measures the switch (borg-zoo#654); its values are hypotheses.

- **`caution_retreat_oscillation_limit`** (retreats, compiled and shipped 0) and
  **`caution_retreat_oscillation_thinks`** (3). After that many moves away in a row
  that were undone (the Borg landed where it stepped, then came back to the
  grid it left or a neighbour of it, each within the thinks of the last, on one
  level; steps that keep going never count), the next is not taken; the
  escapes, the outmatched cure and the attack decide. In 829474344 the retreat
  and the stair flight undid each other 74 times; `bouncing()` acts only at
  70% HP or after 300 thinks without a rest. Note (once per think): `# No retreat: N moves away in a
  row from here were undone (#932)`.
- **`caution_retreat_oscillation_backaway`** (flag, compiled and shipped 0;
  #1023). With the limit on, the back-away (the step to the adjacent grid of
  least danger) joins the same run and is vetoed by it too. Seed 1512540769, a
  Human Priest among white worm masses, backs up from one grid and the stair
  flight walks back, mixed with retreats, until it dies. At 0 only strategic
  retreats count, as in the arms of borg-zoo#654.

| parameter | default | unit | range | source |
|---|---|---|---|---|
| `caution_retreat_oscillation_limit` | 0 | retreats | 0–1000 | `borg_caution.rs:2502` |
| `caution_retreat_oscillation_thinks` | 3 | thinks | 1–100 | `borg_caution.rs:2510` |
| `caution_retreat_oscillation_backaway` | 0 | flag | 0–1 | `borg_caution.rs:4580` |

## Stair flight keeps out of an immobile attacker's reach (#931)

The registry now holds **2436** parameters (2434 before; two added). From the death of the
Half-Troll Ranger 829474344 (borg-zoo run 44950087466; Cemetery study
`studies/fastfrag-white-jelly-829474344`): a strategic retreat from a white jelly
and the next stair-flight step undid each other 74 times until poison killed it.
The zoo measures the switch (borg-zoo#654); its values are hypotheses.

- **`caution_stairs_immobile_reach`** (flag, compiled and shipped 0). The weighted
  stair flight (`weighted_route`, `caution_stairs_flight_threat_weight` > 0) first
  runs its whole search, the clean-path check included, refusing every square next
  to a known immobile attacker (a `NEVER_MOVE` monster with blows: it hits only
  its eight neighbours, `mon-move.c:1586`, `1621-1644`); when that finds no usable
  route, the ordinary search decides (review of #965). Sleeping attackers count,
  and a record counts until the usual bookkeeping drops it. The danger weight
  charged a white jelly beside the route 2.9 steps at full HP, cheaper than the
  detour, and `dangerous_path_blocker` rates a blocker from the Borg's own square.
  Note, only when a flight step avoids a square the ordinary route would enter:
  `# Stair flight keeps out of the reach of the immobile <race> (#931)`.
- **`caution_stairs_immobile_reach_extra_steps`** (steps, shipped 20, read only
  with the flag on). The reach-free route is taken only when its cost (the
  `StairPath` cost, weighted danger included) is at most this many steps above
  the ordinary route's; a costlier detour leaves the ordinary route (second review
  of #965: a 180-step detour against a 2-step route). 0 = any cost. 20 is a
  hypothesis.

| parameter | default | unit | range | source |
|---|---|---|---|---|
| `caution_stairs_immobile_reach` | 0 | flag | 0–1 | `borg_caution.rs:2641` |
| `caution_stairs_immobile_reach_extra_steps` | 20 | steps | 0–250 | `borg_caution.rs:2647` |

## The unknown-potion test's HP floor, and a known cure (#1063)

Two parameters added; the registry now holds 2433. Human Ranger 142490685
(CL2, 18/18 HP, no cure) drank its unknown "Cloudy Potions" as the #983 test on
DL1's up staircase: a Potion of Poison (`TIMED_INC:POISONED` 10+2d7,
object.txt), 1 HP a world tick with no regeneration (game-world.c:514-516,
`player_regen_hp`), and it died 180 game turns later. The engine kills below
0 HP (`player-util.c:228`, `chp < 0`), so without a cure a test at h HP kills
with probability at most P(10 + 2d7 > h): 12: 48/49 (98%), 15: 39/49 (80%),
18: 21/49 (43%), 20: 10/49 (20%), 24: 0. At most: a higher CON shortens the
timer faster (`player_timed_decrement_effects`, game-world.c:280-321) and the
fractional part of base regeneration is kept (`player_regen_hp`); review of
#1097.

- **`item_test_unknown_min_hp`** (compiled 25, **shipped 60**, owner
  2026-11-23: "put minimum hp at 60"): the test needs at least this many
  current HP, besides `item_test_unknown_min_hp_percent`. The compiled 25 is
  Poison's worst case (24) plus 1, the same parameter and default as #1060's
  PR #1071; 60 is the owner's margin, a hypothesis until borg-zoo measures it
  (RULES 14). The cost is fewer tests: a low-level character learns its
  potions later.
- **`item_test_unknown_min_hp_cure_exempt`** (compiled 0, **shipped 1**): a
  known potion of Neutralize Poison or Cure Critical Wounds in the pack
  (`CURE:POISONED`, object.txt; `carries_poison_cure_potion`, `borg_slot`
  skips unaware flavours) waives that floor, leaving only the percent floor
  (50). These are the two cures `borg_caution`'s poison cure drinks without a
  further gate (`borg_quaff_potion`, `borg_quaff_crit(st, true)`) once HP
  falls below max HP / `caution_poison_cure_hp_divisor`. Not counted (review
  of #1097): a poison-curing food such as Elvish Waybread (only `borg_recover`
  eats it, behind a random roll) and a Cure Critical Wounds activation
  (`borg_trait`'s +1000 does not check the charge). The engine's other cures
  (Healing, *Healing*, Life, Resist Poison, Curing staves and rods) are not
  counted either. 0 = the floor holds with a cure too.

#1063's second point (a poisoned Borg with no cure that the poison can kill
goes to the stairs or to town) cannot arise at the shipped values: without a
cure the test needs 60 HP, and Poison costs at most 24.

| parameter | default | unit | range | source |
|---|---|---|---|---|
| `item_test_unknown_min_hp` | 25 | hp | 0–10000 | `borg_item_use.rs:248` |
| `item_test_unknown_min_hp_cure_exempt` | 0 | flag | 0–1 | `borg_item_use.rs:235` |

## Phase Door into unexplored grids, and the newcomer at range (#1066)

The registry now holds **2425** parameters (2419 before; six added: four,
then two after the review of #1075). From
the level-1 death review of track-p `206cf4e7` (borg-cemetery study
`lvl1-deaths-trackp-206cf4e7`): the Half-Troll Priest 694417276 read Phase
Door at 41/55 HP ("escape 1 emergency", **0 scary landing grids of 100**)
into an unexplored room with a shrieker and four cave spiders; the Half-Elf
Blackguard 2025089708 read it at 43/60 ("escape 3.3 danger fallback", 74 of
100, limit 75) and landed farther from the staircase.

Engine: Phase Door is `effect:TELEPORT`, `dice:10`
(`engine/angband/lib/gamedata/object.txt:2571-2581`). `effect_handler_TELEPORT` lists every
passable grid without a player trap (`has_teleport_destination_prereqs`,
`engine/angband/src/effect-handler-general.c:132-146`) at the distance nearest the wanted
one, and picks one uniformly (`:2600-2650`, `:2675`). What the player knows
of a grid plays no part. C's landing check (`borg_caution_phase`) skips an
unknown grid and draws another, so the 100 samples come from the known
grids only: a Borg beside an unexplored room counts the room as safe.

| parameter | default | unit | range | source |
|---|---:|---|---|---|
| `escape_phase_unknown_scary` | 0 | flag | 0–1 | `borg_escape.rs:1477` |
| `escape_phase_unknown_hp_percent` | 50 | percent of max HP | 0–1000 | `borg_escape.rs:1478` |
| `escape_phase_newcomer_wait` | 0 | flag | 0–1 | `borg_escape.rs:1286` |
| `escape_phase_newcomer_hp_percent` | 50 | percent of max HP | 0–1000 | `borg_escape.rs:1290` |
| `escape_phase_unknown_tolerance` | 75 | count | 0–1000 | `borg_escape.rs:930` |
| `escape_phase_newcomer_turn_percent` | 100 | percent of current HP | 0–1000 | `borg_escape.rs:1295` |

- `escape_phase_unknown_scary` = 1: in the Phase Door landing check (every
  site that asks `caution_phase`), a sampled grid never seen (`FEAT_NONE`)
  ends its attempt and counts as scary. The note adds how many of the scary
  grids were never seen. Only while HP is at least
  `escape_phase_unknown_hp_percent` of the maximum; below it the unknown is
  skipped as before (a real emergency). The teleport check
  (`borg_caution_teleport`, range 100) is unchanged. An unknown grid may be
  rock the engine never lands on, so the count errs towards refusing: next
  to much unexplored rock, a Borg above half HP may find no Phase Door.
- The same policy (`phase_unknown_policy`) reaches the Phase Door paths
  that sample no landing (review of #1075, finding 2). The critical and
  last-ditch phases (`PhaseCheck::MonsterNear`), the twitchy ones and
  fast-fragile with `fast_fragile_phase_landing` 0 (`PhaseCheck::None`)
  sample 100 landings the same way (`never_seen_landings`) and refuse when
  more than `escape_phase_unknown_tolerance` first meet a never-seen grid.
  A landing check under the policy records its never-seen count
  (`phase_unknown_seen`); the gate reuses it in the same think at the same
  grid (`gate_count`), so a refused emergency cannot be undone by a lower
  redraw in the critical fall-through (re-review of #1075). The record also
  carries that site's tolerance, and a reuse judges by the stricter of the
  two (#1094). An emergency at 50 that refused 60 never-seen landings is
  not undone by a gate at 75.
  Shoot N Scoot's own sampler (`borg_shoot_scoot_safe`) counts a
  never-seen grid as scary.
- `escape_phase_newcomer_wait` = 1: with `escape_phase_adjacent_rule`, when
  the rule judged the adjacent fight lost, it is not lost if HP is at least
  `escape_phase_newcomer_hp_percent` of the maximum, an awake known monster
  farther than one grid endangers the Borg where it stands
  (`borg_danger_one_kill` > 0), and #759's fight race (`fight_race`, the
  computation of `winning_fight`, now shared) **against the adjacent
  monsters alone** is winning. The verdict is then Fight: other escapes may
  still act, else the rule's counter-attack. Every refusal of the race
  stands: the Borg's own confusion, blindness, hallucination, cut or poison,
  a town-hunt target, an unseen attacker anywhere, unknown damage or HP.
  The race is computed only on a lost verdict, and only with the switch on.
  The next turn must be survivable too (review of #1075, finding 1). The
  race's HP a turn from the adjacent monsters, plus the danger of the
  monsters at range, must stay below `escape_phase_newcomer_turn_percent`
  of HP. That danger is read without fear when
  `escape_phase_lost_damage_only` is on, as the rule reads it. It is also
  the full possible one-turn damage (`full_damage`): no spell-frequency
  discount (`spell_frequency_discount`, lifted from `borg_danger` with the
  same arithmetic) and whole monster turns (re-review of #1075).
- Both switches compiled **0**. **Shipped 1** in `harness/borg.txt` since
  #1094 (owner, 2026-11-23: "I would put both of escape_phase at 1 and
  merge"); the four tuning values ship at their compiled defaults.
  Hypotheses: no zoo measurement chose them (RULES 14; #1073).

## Attack on sight prices poison and drain blows (#1010)

The registry now holds **2415** parameters (2414 before; one added). A
follow-up of #878. `fight_incoming_with` already counts every blow's dice in
`hp_per_turn`, but any effect beyond HURT, ACID, ELEC, FIRE, COLD and NONE
sets `unknown_damage`, and attack on sight's target test
(`category_attack_safe`) refuses such a target. So the shipped entries
`white worm mass | 50 | | attack,breeders | breeder` (CRAWL:POISON:1d2) and the
yellow worm mass (CRAWL:LOSE_DEX:1d3) never act: in #878's named-case arms the
breeder entry fired for mice, lice and green worm masses, never for a white
worm mass.

| parameter | default | unit | range | source |
|---|---:|---|---|---|
| `caution_attack_price_status_blows` | 0 | flag | 0–1 | `borg_caution.rs:3969` |

- 1: the target test prices the blows that hurt without stopping the Borg
  fighting (`status_blow_priced`): POISON, LOSE_*, EXP_*, EAT_*, DISENCHANT,
  DRAIN_CHARGES, each dealing its dice (`monster_damage_target`,
  mon-blows.c:564-1080). POISON adds its timer: 1 HP a world tick
  (game-world.c:513-515, 1080-1081), each hit adding 5 + randint1(rlev) ticks
  (mon-blows.c:683-684), so a poisoner in contact costs `10 / energy` HP a
  Borg turn, none when poison is resisted. PARALYZE, CONFUSE, BLIND, TERRIFY,
  HALLU, SHATTER, BLACK_BREATH and spells of unknown damage still refuse the
  target. Only attack on sight's target test reads it: the winning fight and
  the town hunt also need a HURT-only certificate (`unknown_fight`) and are
  unchanged. Compiled and shipped **0**. A hypothesis (RULES 14).

## The breeder stand pause and boost on the danger without fear (#1009)

The registry now holds **2413** parameters (2411 before; two added). A
follow-up of #878. With `caution_breeder_stoppable` (shipped on through
`caution_breeder_policy = 3`), `note_breeders_stoppable` pauses the stand for
a think when the danger here reaches the HP (`breeders_in_danger`); a paused
stand lets `may_move_away`'s retreat and back-away act. The hard-breeder-fight
boost (`caution_breeder_boost_danger_tenths`, 5: danger ≥ HP / 2) reads the
danger too. Today both read it with its fear prices: one white worm mass next
to the CL3 Mage 461220874 is danger 32 against HP 20 (poison fear 10, then
+150% for a breeder, plus the fear maps). With these switches they read the
damage-only danger, 2. #878's two damage-only switches did not reach these
tests.

| parameter | default | unit | range | source |
|---|---:|---|---|---|
| `caution_breeder_danger_damage_only` | 0 | flag | 0–1 | `borg_caution.rs:5713` |
| `caution_breeder_boost_damage_only` | 0 | flag | 0–1 | `borg_caution.rs:6435` |

- `caution_breeder_danger_damage_only` = 1: the stand pause reads
  `borg_danger_damage_only` (#878) at horizon 1, as before. The stand holds
  where the fear priced it away, so the retreat and back-away stay off.
- `caution_breeder_boost_damage_only` = 1 (split off after the review of
  #1012): the boost reads it at `caution_danger_horizon`, the horizon of the
  `pos_danger` it replaces. The damage-only danger can only be lower, so the
  boost **fires less**: against one white worm mass it stops firing (2 x 10
  is below 20 x 5). Measured apart from the pause.
- Each site computes only the danger it reads (`pick_danger`); the boost's
  only once it is otherwise due (on, standing). The level-lost verdict
  (births against kills) does not read the danger and is unchanged. Both
  compiled and shipped **0**. Hypotheses (RULES 14).

## The lone-slow attack on a level lost to breeders (#1008)

The registry now holds **2411** parameters (2410 before; one added). A
follow-up of #878, from checking its switches against the shipped breeder
policy (`caution_breeder_policy = 3`). `lone_slow_attack` runs in
`fight_before_flight`, after `borg_check_breeders` has set the leave and flee
goals and before the flight, and read neither `breeders_lost` nor the stand;
the scary list's breeder attack entries read both (`attack_on_sight_reason`).

| parameter | default | unit | range | source |
|---|---:|---|---|---|
| `lone_slow_breeder_lost` | 0 | flag | 0–1 | `borg_lone_target.rs:110` |

- With `lone_slow_attack` on and this at 1, the lone rule does not attack a
  breeder (MULTIPLY) while the level is lost to breeders (`breeders_lost`:
  more known than `caution_breeder_hunt_max_count`, or the colony cannot be
  stopped, `note_breeders_stoppable`). Needs `caution_breeder_stoppable` = 1
  (shipped through `caution_breeder_policy = 3`): `breeders_lost` is set only
  there, so with it 0 the switch never fires. The breeder policy's leave decides
  (stairs, then Recall after `caution_breeder_lost_recall_turns`). Other
  monsters, and a level not lost, are unchanged. Compiled and shipped **0**.
  A hypothesis: no measurement chose it (RULES 14).

## Retreat and back-up refuse a dead end (#881 part 2, #1045)

The registry now holds **2405** parameters (2403 before; two added). Owner,
2026-11-22: "start on part 2, the dead-end retreat" (#881, now in P8).

`retreat` takes the nearest viewable "happy" grid of lower danger, and
`back_away` the adjacent grid of least danger. Neither has any notion of a
way out. In a one-exit lattice room every square is happy (the pillar cases
of `borg_happy_grid_bold`). Priest 399497898 (borg-zoo run 44924461733)
made 47 retreats and 27 back-ups after turn 62900 inside the room and never
left it. It reached the dead-end tip 25 times and died to white worm masses.

- **`caution_retreat_dead_end`** (mode, 0). It applies on a dungeon level
  with a known breeder within `danger_monster_max_distance` grids of the
  Borg. The **way out** is every known floor grid at least
  `caution_retreat_dead_end_distance` grids from every known breeder, plus
  every known staircase. Paths run over known floor only: unknown grids
  count as walls, and monsters, which move, are ignored.
  - 1: refuse a **dead end**, a grid whose every known path out passes
    through the Borg's own grid. This happens only where the Borg's grid is
    the only link: a corridor's end, or a pocket behind a single square
    (the whole branch behind it). In a lattice room every square has two
    or more links, so mode 1 refuses nothing there.
  - 2: also refuse a grid farther from the way out, by known path, than the
    Borg's own grid. In the priest's room this is the mode that acts. The
    review of #953 replayed the priest's 74 recorded decisions against the
    rule, using the study's maps: mode 1 refuses 0 of 47 retreats and 0 of
    27 back-ups; mode 2 refuses 25 and 11.
  - Both refuse a grid the Borg can see but has not mapped (`FEAT_NONE`):
    it is not known floor, so it has no known path out. This is the
    perception gap of #881 part 3.
  - The rule refuses a retreat's target and its first step, and each
    back-up grid. It refuses nothing when the Borg has no known path out,
    or already stands on the way out.
- **`caution_retreat_dead_end_distance`** (grids, 6). This is the same
  radius as `caution_breeder_exit_distance` (the study's `D_CLEAR`), set on
  its own.

Not changed: the flights (part 1, `caution_stairs_flight_unknown`), the
breeder exit, and slow crowds without breeders. A refused grid leaves the
retreat or back-up to its other candidates, or to the rest of
`borg_caution` (fight, escapes) when none is left. Hypotheses: no
measurement chose a mode or a distance (RULES 14).

| parameter | default | unit | range | source |
|---|---|---|---|---|
| `caution_retreat_dead_end` | 0 | mode | 0–2 | `borg_caution.rs:1418` |
| `caution_retreat_dead_end_distance` | 6 | grids | 1–100 | `borg_caution.rs:1436` |

## Stair flights through unknown grids (#881)

The registry now holds **2403** parameters (2401 before; two added). Owner,
2026-11-22: track P, part 1 of #881 first, shipped off (now P9: new
capabilities merge into `track-p` only shipped off).

`weighted_stair_path` (the route of every weighted stair flight, shipped
with `caution_stairs_flight_threat_weight = 5`) entered a grid the Borg's map
holds as unknown (`FEAT_NONE`) for one step, like floor: `spread_blocked`
refuses unknown grids only when `avoid` or desperate, and the down flights
pass `avoid: false`. Unseen rock between the Borg and the target looked like
a shortcut. Priest 399497898 (borg-zoo run 44924461733, [study](https://github.com/diegoami/borg-cemetery/blob/claude/modest-knuth-w7k1hg/studies/worm-pocket-399497898/README.md)):
in a one-exit lattice room 58 of 68 committed flight paths crossed granite
north of the room, so the priest reached the dead-end tip 25 times and died to
white worm masses.

- **`caution_stairs_flight_unknown`** (mode, 0). 0: one step (today's
  play; it adds exactly 0 to every grid's cost, so mode 0 cannot change
  play). 1: one step plus `caution_stairs_flight_unknown_steps`. 2: refused;
  the flight walks known grids only.
  - **Where it acts.** The rule acts on the **down** flight
    (`avoid: false`). The **up** flight below CL 36 in the light already
    refuses unknown grids through `avoid`, and the **desperate** walk
    refuses them through `spread_blocked` unless twitchy, so mode 2 does not
    change those two (review of #897, F6).
  - **Mode 1 only re-ranks routes.** The search has a move limit and no
    cost ceiling, so pricing never removes a route. The reachability test
    answers as under mode 0.
  - **Mode 2 also changes three flee decisions** (review of #897, F1). The
    flee rules' reachability test (`stair_reachable` → `stair_flow_reaches`
    → `weighted_stair_reachable`) asks the same search. A staircase
    reachable only through unknown grids is therefore "not reachable", and
    when no staircase is reachable through known grids:
    - `stairs_before_recall` (`caution_flee_stairs_before_recall`, shipped
      1) no longer holds Word of Recall back, so a fleeing Borg reads Recall
      instead of walking to the stairs;
    - `scary_ranged_no_stairs` (`caution_scary_ranged_no_stairs`, shipped 1)
      lets `borg_attack` shoot an awake monster in view on a scary level;
    - `ranged_before_recall` (same switch) prefers shooting to Recall.

    Mode 2 is therefore a route rule plus a change of when the Borg recalls
    and shoots. borg-zoo#642 measured it with this coupling.
- **`caution_stairs_flight_unknown_steps`** (steps, 5; the issue's 3–10).
  With the shipped `caution_stairs_flight_threat_weight = 5`, a grid whose
  danger equals the current HP costs 100 + 500 = 6 steps, and an unknown grid
  under mode 1 costs the same: 5 makes an unknown grid as expensive as the
  most dangerous grid the flight prices. The match holds only while the
  weight stays 5 (review of #897, F7).

Not changed: the unweighted stair flows (weight 0), exploration flows,
retreat and back-up (#1045, #881 part 2), and the perception gap at 28,147
(#1043).
Hypotheses: no measurement chose a mode or a step count (RULES 14).

| parameter | default | unit | range | source |
|---|---|---|---|---|
| `caution_stairs_flight_unknown` | 0 | mode | 0–2 | `borg_caution.rs:7053` |
| `caution_stairs_flight_unknown_steps` | 5 | steps | 0–100 | `borg_caution.rs:7053` |

## Testing unknown potions when safe (#983)

Four parameters added (the registry now holds **2380**). #983's Priest
(seed 17626119, zoo run 45028029063) died confused with four Cure Light
Wounds it had never learned: a flavour found on the floor, which #470 forbids
it to try. Owner, 2026-11-22: "test when safe".

- **`item_test_unknown_potions`** (compiled 0, **shipped 1**):
  `borg_quaff_unknown_when_safe`, right after `borg_test_stuff` in
  `borg_think_dungeon`, drinks one unknown potion when in the dungeon down to
  `item_test_unknown_max_depth` (shipped 30: Dragon Breath, DL40+, asks for
  an aim), with danger 0 from every monster that could reach it within
  `item_test_unknown_danger_turns` (10, `borg_danger`'s `c`: Sleep paralyses
  for 5+1d5, object.txt), at least
  `item_test_unknown_min_hp_percent` of max HP (50), no harmful status
  (paralysis included) and food in the pack (Salt Water). A bad potion costs
  turns or a stat; none kills by itself. #470 still forbids unknown scrolls,
  mushrooms, devices and any unknown item in an emergency.

| parameter | default | unit | range | source |
|---|---|---|---|---|
| `item_test_unknown_potions` | 0 | flag | 0–1 | `borg_item_use.rs:231` |
| `item_test_unknown_min_hp_percent` | 50 | percent | 0–100 | `borg_item_use.rs:238` |
| `item_test_unknown_danger_turns` | 10 | turns | 1–50 | `borg_item_use.rs:243` |
| `item_test_unknown_max_depth` | 30 | level | 0–128 | `borg_item_use.rs:250` |

## Kill-on-sight HP floor (#969)

One parameter added. Track-p's baseline at 9085c266 (zoo run 45028029063)
had 11 of 28 deaths by a grey mushroom patch: #869's `kill-on-sight` entry is
attacked before the cures and Phase Door with no test of the Borg's own HP,
and confused a swing reaches it 11/32 of the time (`player_confuse_dir`).

- **`attack_kill_on_sight_min_hp_percent`** (compiled 0, **shipped 50**):
  below this percent of max HP a `kill-on-sight` entry is not attacked on
  sight when a known Cure Light, Serious or Critical Wounds is in the pack or
  an escape is at hand (stairs underfoot, up or down, or a readable Phase
  Door: `borg_fast_fragile::has_cure`, `escape_available`). The floor only
  stops the attack on sight; the ordinary caution may still decline the cure
  (its own heal gates), so the Borg may neither attack nor drink. With
  neither at hand the attack goes on. 13 named replays: 13 deaths at 0, 1 at
  50 (borg-zoo#665 measures it).

| parameter | default | unit | range | source |
|---|---|---|---|---|
| `attack_kill_on_sight_min_hp_percent` | 0 | percent | 0–100 | `borg_fight_attack.rs:5992` |

## The cure the kill-on-sight floor counted (#1064)

One parameter added. With #969's floor alone, Druid 781905420 (CL1, 12 HP,
8 CLW; track-p `206cf4e7` and `7a466d73`, borg-zoo run 45070528012) was
confused next to a grey mushroom patch at HP 5/12: the floor declined the
attack on sight because a cure was carried, #867's hold then declined the
confusion cure because the attack should come first, and the Borg walked
confused until it died, two steps from the stairs.

- **`attack_kill_on_sight_floor_cure`** (compiled 0, **shipped 1**): when the
  floor declines the attack on sight, the cure it counted is drunk in that
  think, before the flight (confused: Cure Serious, then Critical, which cure
  the confusion, `object.txt`; then Cure Light, which heals 20 and shortens the confusion by 20 turns; not confused,
  the cheapest first), and #867's hold does not apply to a confuser the floor
  declines. The potion is not drunk when it does not outlive the next turn
  (the damage-only danger at the Borg's grid, #878, at or above its HP plus the heal, capped at
  the HP missing) and an escape is at hand: the floor takes the escape in the
  same think (the stairs underfoot, or Phase Door through `phase_door`'s own
  gates, site `kill-on-sight floor`), and drinks after all when neither can
  be used
  (review of #1082). With an escape only (no cure potion) the floor itself
  drinks nothing, and the caution re-evaluates its cures with the hold lifted:
  a cure the floor does not count, such as a Cure Mind mushroom, may be taken.
  A hypothesis until the zoo measures it (RULES 14).

| parameter | default | unit | range | source |
|---|---|---|---|---|
| `attack_kill_on_sight_floor_cure` | 0 | flag | 0–1 | `borg_fight_attack.rs:6009` |

## A lone slow monster, danger without fear, the reserve spell's faint (#878)

The registry now holds **2385** parameters (2376 before; nine added). From
the Human Mage 461220874's death (borg-zoo run 44936494812, csw arm,
da097a76; [borg-cemetery case](https://github.com/diegoami/borg-cemetery/blob/claude/trusting-turing-puk2no/runs/2026-11-22-reborn-confusion-csw-mage-461220874/README.md)):
one white worm mass (speed +0, 10 HP, `CRAWL:POISON:1d2`, MULTIPLY;
monster.txt:677-692) next to a CL3 Mage at HP 20/20, SP 5/5. The core ranked
Magic Missile first, but `borg_caution` retreated ("32 > 1") and the Phase
rule read Phase Door at "danger 32". Danger 32 is mostly fear: 1d2 gives 2,
`danger_monblow_poison_fear` adds 10, the breeder bonus
(`danger_breeder_melee_extra_tenths` 15, below CL20) +150%, the fear maps 2.
Later, at SP 0, the reserve spell fainted the Mage 11 times next to biting
worms. Every switch is compiled and shipped **off**: shipped play is
unchanged (smoke 3/3 identical). The values are hypotheses: no measurement
chose them (RULES 14).

| parameter | default | unit | range | source |
|---|---:|---|---|---|
| `lone_slow_attack` | 0 | flag | 0–1 | `borg_lone_target.rs:94` |
| `lone_slow_radius` | 3 | grids | 1–20 | `borg_lone_target.rs:71` |
| `lone_slow_casts` | 2 | uses | 1–10 | `borg_lone_target.rs:130` |
| `lone_slow_damage_percent` | 25 | percent of HP | 0–1000 | `borg_lone_target.rs:130` |
| `escape_phase_lost_damage_only` | 0 | flag | 0–1 | `borg_escape.rs:1203` |
| `attack_reserve_spell_damage_only` | 0 | flag | 0–1 | `borg_fight_attack.rs:3467` |
| `attack_reserve_faint_percent` | 0 | percent of HP | 0–1000 | `borg_fight_attack.rs:3374` |
| `caution_unseen_bump_phase_turns` | 0 | game turns | 0–100000 | `borg_caution.rs:8600` |
| `caution_unseen_bump_phase_min_danger` | 0 | danger | 0–100000 | `borg_caution.rs:8600` |

1. **Attack on sight, a lone slow monster** (`lone_slow_attack`, new module
   `borg_lone_target.rs`). In `fight_before_flight`, after attack on sight
   and before the winning-fight test, so before retreat, back-away and every
   Phase Door site: exactly one awake, recently seen monster within
   `lone_slow_radius` grids, in view, not faster than the Borg
   (`forecast_monster_speed` against `player_combat_speed`). Its best
   **ranged** attack (`focused_ranged_attack`: missiles, throws, spells,
   devices, activations, among the kinds its scary-list entry allows) must
   kill it (HP left over the expected damage a turn, fail rate included,
   rounded up) in at most `lone_slow_casts` uses that the mana, charges or
   missiles pay for (a cast beyond the mana is not counted), and the
   target's expected blow damage (`fight_incoming`: hit points, no fear
   prices) over those turns must stay below `lone_slow_damage_percent` of the
   current HP. The other monsters' danger gate of attack on sight still
   applies (`category_attack_safe_with(.., false)`). The spell counterpart of
   `borg_adjacent_kill_in_one_blow` (melee only). Poison taken over time is
   not counted.
2. **Danger without fear** (`params::damage_only`, `borg_danger_damage_only`).
   Inside the scope, `borg_danger.rs`'s parameter reads (`danger_value`) give 0
   for every `danger_*_fear` and `danger_*_fear_<n>` price (114; not the flag
   `danger_hunt_breeder_no_acid_fear`: a switch is never zeroed) and the five
   breeder and friends bonuses (`danger_breeder_*_extra_tenths`,
   `danger_friends_*_extra_tenths`), and `borg_danger` skips the regional and
   monster fear maps. Everything else (dice x sides, monster moves, the
   minimum energy, sleep and confusion) is unchanged: it is the danger's own
   hit-point part, not an expected-damage model. `params::value` itself is not
   touched (its size decides what the release unit tests link: a larger
   `value` made `easy_escape_with`'s test reach the engine).
   `escape_phase_lost_damage_only` uses it for the Phase rule's "lost"
   verdict (tests 0, 1 and 2); `attack_reserve_spell_damage_only` for the
   reserve spell's "must be dangerous" test.
3. **No faint next to what bites** (`attack_reserve_faint_percent`). Casting
   `oops` mana short faints the caster `randint1(5 x oops + 1)` world ticks
   (player-spell.c:541-553, player-util.c:835-842), mean `(5 x oops + 2) / 2`
   world ticks (the counter falls once every 10 game turns, game-world.c:651,
   :1080), that is `energy / 10` Borg turns a tick for the Borg's speed.
   The reserve spell is refused when the expected blow damage of the awake
   monsters over that mean reaches the percent of the current HP: each from
   the turn it can reach the Borg (distance − 1 steps at its speed, the
   distance the reserve spell's loop uses: the larger axis; a monster that
   never moves only when adjacent), the target (the one monster that loop
   found within `attack_reserve_spell_near_distance`) only by the cast's
   fail rate, since its kill is what the cast gains. The engine faints the
   caster whether the cast works or fails (the mana block follows the fail
   test in `cast_spell`). The same over-exertion also risks CON
   (`player_over_exert(.., PY_EXERT_CON, 50, 0)`, player-util.c:824-831: a
   50% chance to lose a point, a quarter of those permanent, about 12.5% a
   cast): the gate prices only the faint's HP damage, so refusing the cast
   also spares that, and `faint25` is read as an HP-damage gate.
4. **`danger_breeder_melee_extra_tenths`** is already a setting (15: +150%;
   the comment "extra 50%" in `danger_one_kill` and in C's
   borg-danger.c:2422-2425 does not match). Arms 5 and 15 need no code.
5. **The unseen-bump Phase Door, fresh and with danger**
   (`caution_unseen_bump_phase_turns`, `caution_unseen_bump_phase_min_danger`).
   `caution_unseen_bump_turns` (200 shipped) keeps a bumped grid occupied;
   the Mage read its last Phase Door 132 game turns after its bump, at danger
   0. With these, the Phase Door needs the bump at most that many game turns
   old, and the danger here at least that much; the detection and oil
   responses are unchanged.

## Confused attack on the grey mushroom patch, shipped (#901)

No parameter added; shipped values changed. Owner, 2026-11-22, after #867
and #869: "once you are close to him, you attack with your best attack";
then, for the point-blank divisor, "override that setting for classes that
are supposed to melee ... melee is infinite, flasks are not".

- **`attack_confused_by_direction`** 0 → **1** (compiled 0). Confused, the
  Borg also throws, shoots and uses wands, rods and staves at an adjacent
  monster, by direction key, at the 11/32 confused aim; the cure waits next
  to an immobile non-unique confuser while an attack is open. See "Confused
  attacks by direction (#867)" below.
- **`attack_throw_point_blank_divisor`** 0 → **1** in the common file, with
  **0** for Warrior, Blackguard, Paladin and Rogue (their class files). 0
  inherits `attack_arrow_point_blank_divisor` (5; Rogue 3).
  - At 1 an adjacent non-unique throw rated with `attack_throw_as_melee` is
    not divided in the ordinary attack chooser (#870), so a caster throws oil
    at point blank. Attack on sight already bypassed it.
  - The engine's flask: 1d4 x 3 (`THROWING`, 2 + weight/12) x 3 (`EXPLODE`,
    `player-attack.c:1296`), about 22 a hit.
  - At 1 for everyone, the smoke Warrior (CL2-3, the starting Dagger, 1d4)
    threw oil at an adjacent white worm mass, rated 84 against a full round
    of blows' 43. The melee classes keep the penalty: a workaround until
    flasks are spent only when needed (#912).
- Unchanged and kept 0: `caution_ranged_vs_immobile` (a confused step away
  would pre-empt the attack, #896) and `caution_ranged_only`.

Not measured: borg-zoo#636 (arms `confused` and `confused-pb`; the class
split is not one of its arms). Shipped on the owner's decision before the
measurement; a hypothesis until the zoo reports (RULES 14).

## Confused attacks by direction (#867)

The registry now holds **2374** parameters (2373 before; one added). Owner,
2026-11-21, from the play pack of borg-zoo#581's Gnome Necromancer
459443830: confused next to a grey mushroom patch, the owner threw flasks of
oil by direction and the second one killed it, where the Borg only swung and
walked into walls. Then: "attack must be dependent by class ... It should
choose the best attack available, and in case of mage without spells, that
would be the flask", and "if it has a wand/rod that must be an option too".

- **`attack_confused_by_direction`** (flag, compiled and shipped 0). The
  engine refuses a confused player only spells (`player_can_cast`,
  `player-util.c:1102`) and scrolls (`:1182`). Melee (`cmd-cave.c:1301`),
  missiles (`player-attack.c:1350`), throws (`:1393`) and aimed devices
  (`cmd-obj.c:440`) pass through `player_confuse_dir`
  (`player-util.c:1353-1377`): a direction key is kept 25% of the time,
  otherwise one of the 8 directions is drawn, so the aim holds 11/32 of the
  time; a target (direction 5) is always redrawn. C forbids every item attack
  while confused (`cant_aim`). On, while confused but neither blind nor
  hallucinating:
  - throws, missiles, wands and rods are rated against an **adjacent**
    monster only, aimed at it (no offset aim), and sent by **direction key**
    (`# Confused: aiming by direction N, not at a target`), not at a target;
  - every aimed attack, melee included, is rated at the aim chance (its
    value times 11/32; the item attacks' damage and kills too, see below),
    so the usual chooser compares them on damage: a weak-armed caster with no spell throws oil, a fighter swings;
  - a staff needs no aim (`cmd-obj.c:431-441`) and keeps its full value;
  - next to an immobile monster whose blow confuses (`CONFUSE`, the grey
    mushroom patch's `SPORE:CONFUSE`), the confusion cure waits and the
    attack goes first, at both cure sites (`cure_held_for_attack`; owner: "I
    threw flasks at the patch immediately"): the patch, speed +10, confuses
    again before a cure is worth anything. Not while the moving monsters near
    are as dangerous as the HP left, not next to a unique (Wiruin), and only
    while `borg_attack`'s estimate has an attack to make (review of #868: a
    magic mushroom patch scares and makes the Borg hallucinate, leaving none);
    a heal for HP still runs.
  The aim chance also reaches `borg_attack_estimate`, the fight-or-flee
  tests' damage, for the item attacks only: a blow's damage there stays C's
  (full), so a confused fight that melee wins is judged as in C (review of
  #868); the blow's value is still weighted when the chooser compares it. Not changed: `ranged_means` (still no ranged means while
  confused), activations, rings and dragon armour (still stopped while
  confused by `cant_aim`, as in C; review of #868).

| parameter | default | unit | range | source |
|---|---|---|---|---|
| `attack_confused_by_direction` | 0 | flag | 0–1 | `borg_fight_attack.rs:304` |

## Thrown objects at point blank (#870)

`attack_throw_point_blank_divisor` (0..1000, compiled **0**, no shipped
override) separates the native `attack_throw_as_melee` throw rating from
`attack_arrow_point_blank_divisor`. Zero inherits the resolved arrow divisor,
including class/profile overrides, so existing games retain their policy.
One removes the throw penalty; 2..1000 divide the damage before hit chance.
Only adjacent non-unique targets receive it. Arrows and legacy throws with
`attack_throw_as_melee = 0` retain their arrow divisor.

This is missile conservation policy, not an engine penalty: at `574b618c`,
`player-attack.c:156-165` subtracts distance from missile hit chance and
halves it against non-obvious targets; it has no adjacent damage divisor.
See the Strategy Manual, chapter 4 §4.11. The focused attack-on-sight /
fast-fragile planner already bypasses conservation policy, including this
penalty. This parameter does not change that planner's ranking of oil
against melee. A Zoo comparison is required before the owner chooses a
shipped override; no survival improvement is claimed.

## Kill on sight, with the attack kinds named (#869)

No parameter added (the registry still holds **2352**): a scary-list reason
and a field. Owner, 2026-11-21, after playing borg-zoo#581's Gnome
Necromancer 459443830 and Elf Necromancer 412975293 (both killed by a grey
mushroom patch; a flask of oil thrown while confused killed it in both):
"Always attack, confused or not. Unless there is something more dangerous",
"maybe should put it on another list", "add what type of attack are allowed:
melee, missiles, wands, picks the best one".

- **`kill-on-sight`** (a reason of an `attack` entry,
  `borg_fight_attack::KILL_ON_SIGHT`). `attack_on_sight` runs in
  `fight_before_flight`, before `borg_caution`'s cures and Phase Door; for
  this reason it drops the target's own damage test
  (`category_attack_safe_with(.., false)`, as #842 does for fast-fragile
  monsters), so it attacks confused or not; the other monsters' danger above
  HP x `caution_flee_danger_hp_multiplier` still stops it. Unlike
  fast-fragile, no heal ban and no fast-fragile escape.
- **Attack kinds** (an optional sixth field of an attack entry,
  `scary_list::ATTACK_KINDS`): `melee`, `missiles`, `throws`, `spells`,
  `devices` (wands, rods, staffs), `activations`, by
  `attack_preference::Family`; none named = all. `focused_attack` ranks only
  those, best first; it serves attack on sight, fast-fragile and the winning
  fight test (#759) for that monster.
- **Shipped:** `grey mushroom patch | 50 | | attack | kill-on-sight | melee,
  missiles, throws, devices` (monster.txt:545: speed +10, 2 HP,
  `SPORE:CONFUSE`, NEVER_MOVE), beside its inert #838 `ranged` entry. Confused,
  the attack is melee unless #867's `attack_confused_by_direction` allows the
  throw by direction.

## Ranged-only monsters (#838)

The registry now holds **2291** parameters (2290 before; one added). Owner,
2026-11-21: "a list of monsters to never melee ... and only attack from afar".
The named-list version of `caution_ranged_vs_immobile` 2, which borg-zoo#576
measured leaning worse (10 deaths against 6 on 540 starts).

- **`caution_ranged_only`** (flag, compiled and shipped 0). A scary-list
  entry `monster | up to CL | classes | ranged | reason` is a kind of its own
  beside `ignore` and `attack`: stored and replaced independently, removed
  with `monster | none | | ranged`, never a scare, and it takes a reason and
  no `breeders` flag. With the flag at 1, a listed monster, up to the level
  for the class, is **kept at range** as mode 2 keeps an immobile attacker:
  `borg_thrust_damage_one` gives no melee blow at it, the hunt skips it, and
  the flows keep out of its grid and the eight around it (a flow to the
  stairs may pass, never onto its grid; the Borg's own grid stays open so a
  flow can lead out). Missiles, spells, throws and devices still reach it,
  or it is left alone. Any listed monster qualifies, mobile ones too.
- **No step away.** The step away (`immobile_step_away`) is still the
  `caution_ranged_vs_immobile` switch's: the list is the "without step-away"
  arm of borg-zoo#576's comparison. A flow does not walk next to a listed
  monster it knows; one it first sees when adjacent is left by the next flow.
- **The shipped list** has the table of #838 (the 14 never-moving monsters to
  DL12 whose blows do more than damage: floating eye, grey, yellow, spotted
  and purple mushroom patches, the white, yellow, silver, rot, red and grape
  jellies, the radiation, disenchanter and bloodshot eyes), each up to CL 50
  for every class (a hypothesis, as #760's). The lines are inert at 0.
- **The floating eye is seen only when adjacent in a dark corridor.** A torch
  of light 2 lights radius `light - 1` = 1 (`cave-view.c:658`), so the eye
  appears in the grid the Borg has just stepped to, and its gaze (paralysis
  and damage) comes in the same game turn: no list can keep the Borg out of
  reach of an eye it has not seen (Ranger 1241679603 of borg-zoo#576). The
  list protects against an eye already known, on the level or in the
  detection the Borg cast.

| parameter | default | unit | range | source |
|---|---|---|---|---|
| `caution_ranged_only` | 0 | flag | 0–1 | `borg_caution.rs:1840` |

## Confusion (#837)

The registry now holds **2290** parameters (2283 before; seven added). Owner,
2026-11-21, from the three borg-zoo#566 deaths (a Mage and a Necromancer at a
grey mushroom patch, a Mage at a kobold shaman, each with Cure Light Wounds in
the pack). Every switch compiles and ships 0 (C); borg-zoo measures them.

- **`caution_confused_clw_cure`** (flag, 0). A Potion of Cure Light Wounds
  takes 20 turns off confusion (`object.txt`, `TIMED_DEC:CONFUSED`, dice 20),
  which clears a grey mushroom patch's 3 + 1d(level) turns
  (`mon-blows.c:1001`). On, it is a confusion cure after Clear Mind, Cure
  Serious and Critical Wounds, before a Healing potion, in `borg_heal`
  ("Fixing Confusion", levels 2 and 5) and in the cures at the end of
  `borg_caution`. C cures blindness with it only.
- **`caution_confused_cure_before_walk`** (mode, 0). 1: a confused Borg with a
  confusion cure in the pack does not start a "Desperate for Stairs" walk
  (75% of a confused Borg's steps go astray, `player-util.c:1353`); the cures
  at the end of `borg_caution` come first, unless the cure is held next to an
  immobile attacker (#646); "a cure in the pack" is what that chain would use
  (Clear Mind, Cure Serious Wounds, Cure Critical Wounds above
  `item_use_ccw_min_stock`, Cure Light Wounds with the CLW switch, a Healing
  potion, a Staff of Healing; not a Staff of Curing). 2: no such walk at all
  while confused, cure or not, held or not.
- **`caution_confusion_cure_when_stuck`** (flag, 0, with
  `caution_ranged_vs_immobile` >= 1). #646 holds the cure next to an
  immobile attacker whether or not a step out of its reach is possible; in a
  dead end, with every grid more dangerous or after the three tries, the Borg
  neither stepped nor cured and went on to melee confused. On, the cure is
  held only while `immobile_step_away` could still step (`# No step out of
  reach ... curing confusion anyway`).
- **`caution_immobile_step_mobile_first`** (flag, 0, with
  `caution_ranged_vs_immobile` >= 1). The step away ranks the grids out of
  the immobile attacker's reach by the danger from the monsters that move
  within two grids of the grid, then the stairs
  (`caution_immobile_step_to_exit`), then the least danger. Off, the stairs
  come first, so a grid next to a mobile monster can win for being a step
  nearer them.
- **`caution_immobile_no_return`** (mode, 0, with `caution_ranged_vs_immobile`
  >= 1; borg-zoo#576). After a step away from an immobile attacker, the flows
  (`spread_blocked`) do not enter its grid or the grids next to it while it is
  still known there (the grid still holds its kill and it is an immobile
  attacker). #576's two grey mold deaths (Necromancer 1203232169, Mage
  13040847) stepped away, then the next flow (`Flowing toward dark-3`) led back
  beside the mold; mode 2 does not keep a grey mold at range (its blows are
  plain damage), so `in_reach_of_kept` did not see it. 1: only while the Borg
  is below its full HP (it rests, then the usual rules decide again); 2: for the
  rest of the level. A flight to the stairs is not stopped.
- **`town_restore_life_reserve`** (count, 0). From maximum dungeon depth 12,
  a character without repeatable restoration protects this many known Restore
  Life Levels potions in town and in ordinary dungeon junking. A desperate
  full-pack clear can give up the stack. The town planner values each copy
  in the reserve at one quarter of its restore weight, with diminishing
  value for extra copies and separate value for a current drain. Depth 12
  is the grape jelly’s native depth. The reserve and its weights are
  hypotheses for measurement.
- **`town_confusion_cure_reserve`** (count, 0) and
  **`town_confusion_cure_reserve_percent`** (percent, 100). While the character
  is not confused, the first N confusion cures in the pack are a requirement
  as Recall's reserve is: each worth the percent of the restore weight,
  nothing past them. Only a full cure counts (Cure Serious or Critical
  Wounds, Clear Mind, a Curing staff: one unit each): not confused, the timer
  is 0, so a Cure Light Wounds (`TIMED_DEC`, clamped to the timer) supplies
  none, whatever `caution_confused_clw_cure` says (review of #843, F1; the
  Necromancer 2074919561 buying 9 CLW and no CSW is not explained by the
  reserve). 0 keeps the saturating curve, under which Cure
  Serious Wounds were bought, then stored at the Home (#710, #711).
- **The kobold shaman** (`monster.txt:1450`, DL3, 11 HP, speed +0, `WOUND |
  CONF | HEAL | MISSILE` 1 in 5): no setting, only two lines a zoo arm
  appends to the scary list (borg-zoo#228; `attack_on_sight` ships 1, so
  neither is in `harness/scary_monsters.txt`). **Attack:**
  `kobold shaman | 50 | | attack | confusing-caster`: a missile, spell or
  throw first (#760). **Leave:** `kobold shaman | 6-9`, a scare. A unit test
  (`kobold_shaman_arm_lines_parse_over_the_shipped_list`) parses both over
  the shipped list.

| parameter | default | unit | range | source |
|---|---|---|---|---|
| `caution_confused_clw_cure` | 0 | flag | 0–1 | `borg_caution.rs:2031` |
| `caution_confused_cure_before_walk` | 0 | mode | 0–2 | `borg_caution.rs:1046` |
| `caution_confusion_cure_when_stuck` | 0 | flag | 0–1 | `borg_caution.rs:2158` |
| `caution_immobile_no_return` | 0 | mode | 0–2 | `borg_caution.rs:1899` |
| `caution_immobile_step_mobile_first` | 0 | flag | 0–1 | `borg_caution.rs:2279` |
| `town_restore_life_spell_fail` | 20 | percent | 0–100 | `borg_town_capability.rs:2571` |
| `town_restore_life_reserve` | 0 | count | 0–10 | `borg_town_capability.rs:2564` |
| `town_confusion_cure_reserve` | 0 | count | 0–10 | `borg_town_capability.rs:1131` |
| `town_confusion_cure_reserve_percent` | 100 | percent | 0–1000 | `borg_town_capability.rs:1134` |

## Dark-entry detection exception (#796)

The registry now holds **2275** parameters (2273 before; two added).

- `light_hallway_repeat_guard=1` (shipped, compiled default 0). At 1,
  defense and exploration yield when the first eligible ray has the same
  observed terrain as the last beam queued from that square. At 0, historical
  fixtures can replay with the original clock and beam behavior.
- `light_detect_dark_entry=0` (compiled and shipped). At 1, the exception
  permits **Detect Monsters** and **Read Minds** below the regular level
  gate. A dungeon arrival or adjacent step must be on a non-glowing tile
  facing a ray of known floor into a grid still unknown beyond personal
  light. Existing detection coverage, cooldown and spell failure checks
  still apply. There is at most one exception cast attempt per panel per
  level, even if the spell fails.
- `light_detect_dark_entry_reserve_sp=2` (compiled and shipped). The spell
  must leave at least this many SP and enough for the cheapest learned,
  viable response among Phase Door, Teleport Self, Magic Missile and
  Nether Bolt. If no listed response is legal, there is no exception.
  The reserve also respects the two existing mana thresholds in
  `borg_recover` at the observed level and maximum SP, so the cast does
  not cross either threshold in that state.

Detect Evil and Seek Battle keep the level gate. The engine predicates
select non-invisible monsters for Detect Monsters, monsters with
`RF_SPIRIT` for Read Minds, evil monsters for Detect Evil, and
fear-susceptible monsters for Seek Battle
(`effect-handler-general.c:1858–1943`, `mon-predicate.c:83–172`).
The floating eye has neither `EVIL` nor `SPIRIT`, and is immune to fear
(`monster.txt:694–708`, `monster_base.txt:128–132`). Its CL1 Necromancer
also cannot know Read Minds before CL4 (`class.txt:1017–1025`).
The exception can protect a Mage or Rogue against a visible-species eye
while it is still unseen. The named Necromancer death cannot be prevented
by this spell eligibility change.

## Ranger Phase Door reserve regression (#797)

The Ranger check runs one fixed Half-Orc birth with reserve 0 and 2 and
weight 100 in both. It compares completed purchases before the first
departure, the Alchemist's stock and the first observed dungeon-contact
inventory. See [ZOO-INTERFACE.md](ZOO-INTERFACE.md#phase-door-observations-797)
for the additional receipt. The registry, compiled values and shipped
profile are unchanged.

## Resolved settings at birth (#798)

`settings-resolved.json` records the active class and every parameter
whose effective value differs from its compiled class default. The receipt
uses the decision core's parameter lookup after applying the class
override. See [ZOO-INTERFACE.md](ZOO-INTERFACE.md#resolved-settings-at-birth-798)
for the schema.

## Preparedness trace (#795)

The trace records each live preparedness check at dive and trip call sites.
A failed row carries the first rule evaluated that failed, its compared
values, and the original diagnostic text. The next observation links a
depth outcome to that check's think. No parameter changes. See
[ZOO-INTERFACE.md](ZOO-INTERFACE.md#preparedness-trace-795) for the schema
and trip-counting method.

## Walk out of a slower crowd (#690)

The registry now holds **2269** parameters (2266 before; three added).
Owner, 2026-11-19, on #675: file the general case the breeder work does not
cover. Surrounded by monsters slower than itself, with no escape item, the
Borg stood on one grid and meleed at the danger cap until it died
(borg-cemetery#78: a Blackguard in a yellow worm mass colony, speed 100
against its 110, with a grey mushroom patch adjacent, which kept #544's
breeder exit from applying).

- **`caution_slow_crowd_exit`** (compiled 0, shipped 0), tried in
  `borg_caution` right after #544's breeder exit: surrounded
  (`borg_surrounded`), the danger here at or above the HP, a monster
  adjacent, and every awake monster within `danger_monster_max_distance`
  either never moving (`NEVER_MOVE`) or slower than the Borg. The Borg then
  walks with the desperate flow (danger ignored, monster grids routed
  around) to the nearest known chokepoint (corridor or doorway, as the
  breeder exit's) **or staircase**, at least
  `caution_slow_crowd_exit_distance` grids from every monster within that
  reach; with none reachable, to the nearest other known open grid that
  far; with neither, the distance comes down by one, to 2. Noted
  `# Walking out of a slower crowd …`. (In the case, the nearest corridor
  was 115 grids away behind the worms, and by the end 19 worm masses left
  no grid 6 grids from all of them: without the open grids and the
  descending distance the rule found an exit but never walked.)
- **Slower** is energy per game turn (`extract_energy`, game-world.c), not
  the raw speed. The comparison uses base speeds. The threshold is the Borg's energy at least 100 +
  `caution_slow_crowd_exit_margin_percent` % of the monster's. Speed 110
  against 100 is 10 against 5; 110 against 105 is 10 against 7, slower at
  the default 25%, not at 50%.
- **Held** while an awake monster that moves fails the energy margin.
  The default danger horizon is 20 grids. Sleeping monsters do not veto
  the walk, but their positions still constrain exit grids.
- The breeder policy leaves this switch independent. At policy 3 the
  breeder hunt can pause for a dangerous turn, and the walk can act
  during that pause. Policy 4 can block paths near remembered breeders
  during flight. At policy 5 the breeder-only exit gets first try.
- A successful walk precedes the cornered fight and the escape block.
  It can replace a committed stair flow even when Phase Door is
  available. The owner kept this order for the comparison in #756.

| parameter | default | unit | range | source |
|---|---|---|---|---|
| `caution_slow_crowd_exit` | 0 | flag | 0–1 | `borg_caution.rs:5476` |
| `caution_slow_crowd_exit_distance` | 6 | grids | 1–100 | `borg_caution.rs:5487` |
| `caution_slow_crowd_exit_margin_percent` | 25 | percent | 0–1000 | `borg_caution.rs:5485` |

## Stair routes after a tactical retreat (#932)

`caution_retreat_reentry_ticks` (compiled default 0, range 0–128 Borg
thinks) keeps recently abandoned grids out of candidate staircase routes
after the Borg confirms each tactical retreat landed. The hold lasts only on the
same level while the abandoned grid still exceeds
the strategic retreat danger threshold. If the normal departure search
finds no route, the ordinary flight Recall gates still apply. A 128-think
window is a pilot hypothesis.

## Threat-weighted stair flights (#765)

The registry holds **2266** parameters. `GOAL_FLEE` stair
routes, including desperate walks and capped down-stair flights, and a
`GOAL_BORE` ascent while fleeing or leaving minimize the sum of
`1 + weight × danger / current HP` over the grids entered,
including the staircase. The route uses the Borg's one-move grid danger
estimate and blocks awake monster grids. Existing terrain, sneak and
breeder rules still apply. The down-stair cap counts moves.

The route is planned from the current observation. When a clean-path check
refuses the cheapest route, the flow searches the same stair set for the
cheapest route that passes the check. Mode 3 judges whether a monster is
ahead relative to each candidate staircase. Ranged-first reads the
selected weighted route on a desperate walk. A positive weight takes
precedence over the old grid danger gate and
`caution_stairs_flight_ignore_danger` on these flights. The compiled
weight is 0.

The shared profile ships weight 5 for every class (#786), with clean-path
mode 3, flight clean-path checks enabled, the fast-neighbor threshold at
50%, the Recall cure threshold at 30%, and the old flight danger gate
disabled. The mode 3 blocker threshold is 50% of current HP. Ranged-first
and the rising HP override ship at 0. [Zoo pilot](https://github.com/diegoami/borg-zoo/issues/524)

At a positive weight, only unoccupied staircases are eligible targets.
`caution_stair_parked_fallback` applies to legacy routing at weight 0.

| parameter | default | unit | range | source |
|---|---|---|---|---|
| `caution_stairs_flight_threat_weight` | 0 | extra steps at current HP | 0–100 | `borg_caution.rs:7027` |

## Phase Door first, unseen flavours, food and light (#761)

The registry now holds **2247** parameters (2240 before; seven added). Owner, 2026-11-20:
"we need to value phase doors much more, especially for rangers"; "increase
phase door and decrease value of flask all over the board"; "make sure it
picks it up". Every line ships at today's value (off or 100) — except
`town_phase_reserve` for the ranger, which ships 2 since #784 (owner,
2026-11-20, "for rangers put town_phase_reserve = 2"; the owner's
correction of an earlier paladin instruction, withdrawn before it merged);
the measuring
moved to borg-zoo, so the arms below are a request, not a recommendation.

| parameter | default | unit | range | source |
|---|---|---|---|---|
| `town_phase_reserve` | 0 (off) | count | 0–10 | `borg_town_capability.rs:1004` |
| `town_phase_reserve_percent` | 100 | percent | 0–10000 | `borg_town_capability.rs:1006` |
| `flow_take_phase_short_below` | 0 (off) | count | 0–20 | `borg_flow.rs:2362` |
| `flow_take_phase_first` | 0 (off) | flag | 0–1 | `borg_flow.rs:2362` |
| `town_unseen_percent` | 100 | percent | 0–10000 | `borg_town_capability.rs:727` |
| `town_food_percent` | 100 | percent | 0–10000 | `borg_town_capability.rs:978` |
| `town_light_percent` | 100 | percent | 0–10000 | `borg_town_capability.rs:959` |

- **Phase Door in town:** a capability row, "phase" (`PHASE`), on its own curve
  (weight `borg_town_w_escape` times exposure, scaled by
  `town_phase_curve_scale_percent`). A scroll's first copy is worth about a
  third of the trip's escape weight, so a 1-gold arrow or a 20-gold potion
  outbids it. `town_phase_reserve` adds a second row, "phase reserve": the
  first N scrolls are a requirement, each worth `town_phase_reserve_percent`
  of the escape weight, nothing past N (as Recall's reserve, #557).
- **Phase Door on the floor:** `take_select` (`borg_flow_take.rs:513`) skips an
  item far from the stairs twice (the `flow_leash_take_*` checks, lines 582
  and 631). While the Borg holds fewer than `flow_take_phase_short_below`
  Scrolls of Phase Door in the pack (`carried_phase_scrolls`: a Phase Door
  spell or an equipped activation is not a scroll), a Phase Door is exempt
  from both; with `flow_take_phase_first`, when one is in reach it is the
  only object flowed to — and when that Phase-only flow reaches nothing, the
  dropped candidates are restored and the flow is tried once more
  (`borg_flow_take`, review of #778).
- **Unknown flavours:** the planner does not buy a flavour to learn it. The
  buy-to-identify weight (`borg_town_w_knowledge`) is forced to 0 in the live
  plan (`borg_town_shadow.rs:51`, `effective_params`; `borg_town_usefulness.rs:271`
  with the trip policy), and a ware serving only knowledge is not bought
  (test `a_flavour_unknown_ware_serving_only_knowledge_is_not_bought`). A shop
  lists true names: the plan values the real kind (`supplies`,
  `borg_town_capability.rs:519`). `{unseen}` only means the player has not seen
  the flavour, and the message after the purchase then names the flavour
  ("You bought 3 Gold Potions"). At birth every potion and scroll is unseen, so
  in 200 Ranger games every one of the unknown-potion purchases was priced
  20 gold (Cure Light Wounds) and the unknown scrolls 15 gold (Phase Door)
  (`dump.jsonl` purchase records: "Potions of Cure Light Wounds {unseen}").
  `town_unseen_percent` scales what any unseen shop ware supplies, Phase Door
  excepted (it has `town_phase_*`); it lowers unseen Cure Light Wounds and
  Cure Serious Wounds as much as unknown Resist Heat, and a ware whose supply
  scales to zero drops out of the plan (`Model::valid` keeps no zero-unit
  supply row; review of #778). For healing alone use
  `town_heal_curve_scale_percent` and `town_heal_reserve_hp_percent`.
- **Food and light per class:** `borg_town_w_food` and `borg_town_w_light` are
  settings of the common file with no per-class spelling.
  `town_food_percent` and `town_light_percent` are their per-class percent.

**Local check** (100,000 turns, the 200 Rangers of the zoo pilot and 50 of each
other class; paired by start; a handful of deaths, so the death counts are
within chance): the tables are in `docs/handover/2026-11-20-761-ranger-phase.md`;
borg-zoo measures the arms listed there.

## The basic kit (#840)

The registry now holds **2314** parameters (31 added: every `kit_*` setting). All the switches
ship 0: play is byte-identical to before until a setting turns one on. The
numbers are hypotheses taken from the issue's examples; borg-zoo measures them
before any ships (RULES 14).

| parameter | default | unit | range | source |
|---|---:|---|---|---|
| `kit_leash` | 0 | flag | 0–1 | `borg_kit.rs:201` |
| `kit_leash_min` | 1 | steps | 0–20 | `borg_kit.rs:314` |
| `kit_leash_power` | 1 | power | 1–4 | `borg_kit.rs:314` |
| `kit_max_level` | 15 | level | 1–51 | `borg_kit.rs:279` |
| `kit_band_split_level` | 5 | level | 1–51 | `borg_kit.rs:178` |
| `kit_cure_target` | 3 | count | 0–20 | `borg_kit.rs:180` |
| `kit_phase_target` | 2 | count | 0–20 | `borg_kit.rs:181` |
| `kit_cure_target_high` | 5 | count | 0–20 | `borg_kit.rs:180` |
| `kit_phase_target_high` | 4 | count | 0–20 | `borg_kit.rs:181` |
| `kit_missile_target` | 20 | count | 0–200 | `borg_kit.rs:182` |
| `kit_flask_target` | 5 | count | 0–100 | `borg_kit.rs:183` |
| `kit_light_turns_target` | 1000 | turns | 0–100000 | `borg_kit.rs:184` |
| `kit_food_target` | 3 | count | 0–20 | `borg_kit.rs:185` |
| `kit_weight_cure` | 1 | weight | 0–100 | `borg_kit.rs:191` |
| `kit_weight_phase` | 1 | weight | 0–100 | `borg_kit.rs:192` |
| `kit_weight_ranged` | 1 | weight | 0–100 | `borg_kit.rs:193` |
| `kit_weight_light` | 1 | weight | 0–100 | `borg_kit.rs:194` |
| `kit_weight_food` | 1 | weight | 0–100 | `borg_kit.rs:195` |
| `kit_descent_min_percent` | 0 | percent | 0–100 | `borg_kit.rs:202` |
| `kit_leave_danger_percent` | 0 | percent of HP | 0–1000 | `borg_kit.rs:203` |
| `kit_restock` | 0 | flag | 0–1 | `borg_kit.rs:204` |
| `kit_restock_below_percent` | 75 | percent | 0–100 | `borg_kit.rs:385` |
| `kit_restock_min_gold` | 50 | gold | 0–100000 | `borg_kit.rs:365` |
| `kit_restock_light_turns` | 0 | turns | 0–100000 | `borg_kit.rs:384` |
| `kit_start_reserve` | 0 | flag | 0–1 | `borg_kit.rs:420` |
| `kit_start_cure_reserve` | 3 | count | 0–20 | `borg_kit.rs:424` |
| `kit_start_cure_hp` | 20 | hp | 1–1000 | `borg_kit.rs:424` |
| `kit_start_phase_reserve` | 2 | count | 0–10 | `borg_kit.rs:420` |
| `kit_money_first` | 0 | flag | 0–1 | `borg_kit.rs:205` |
| `kit_money_first_max_level` | 2 | level | 1–51 | `borg_kit.rs:298` |
| `kit_money_leash` | 6 | steps | 0–250 | `borg_kit.rs:317` |

**The score** (`borg_kit.rs`): the weighted mean of five shares, each the
amount held over its target and capped at 1: cures (CLW + CSW + CCW), Phase
Door, a ranged option (the larger of launcher missiles over `kit_missile_target`
and flasks of oil over `kit_flask_target`; a Mage's Magic Missile and a
Necromancer's Nether Bolt count as met), turns of light (the wielded light and
the spares) and food. The core's own counts are used (`BI_AC*`, `BI_APHASE`,
`BI_AMISSILES` less the flasks it adds, `BI_AFUEL`, `BI_FOOD`), so a cure
spell, a Phase Door spell or a light that needs no fuel count as plenty.
Permille integers throughout, `kit_leash_power` an integer, so a game replays
from its keys.

**The leash** (`kit_leash`): `kit_leash_min + (C's leash - kit_leash_min) x
score^kit_leash_power` while the score is below 1, in the dungeon, up to
`kit_max_level`. C's leash is CL x `flow_leash_per_level` + `flow_leash_base`;
where C has none (the depth is well above the level) the kit still uses that
formula. The stair check of the hunts and item detours
(`borg_flow_far_from_stairs_dist`, both checks of `take_select`) is on at any
depth and level while the leash follows the kit; the bored growth of C's leash
is added after the scaling. The "wandered too far" return to the up stair
(`borg_think_dungeon`) uses the scaled leash too, never below 3 (the "close
enough" test would flap).

**Curiosity**: `kit_descent_min_percent` refuses a descent below DL1 while the
score is below it (`borg_prepared`: a descent from DL1 to DL2 and deeper;
the way out of town is never refused). `borg_prepared` is asked about many
depths (the power dives at +5, the recall probes at +20 and 60% of the
deepest level, 99), so the floor refuses every depth below the Borg's own,
not only the next, while the mode applies (CL up to `kit_max_level`); a depth
at or above its own is never refused, so it never keeps a Borg from going up
or to town; `kit_leave_danger_percent` takes the up
stairs under the Borg at the first danger of that share of its HP, as #614's
rule does for a Borg that has to recover, and starts a flight to the stairs
when it is off them (`borg_caution`).

**Remedies**: a, `kit_restock`: the kit is part of `borg_restock`
at any depth, once the purse has grown by `kit_restock_min_gold` since the
Borg left town (a trip that bought nothing is not repeated), with a
light-turns check (`kit_restock_light_turns`); b, `kit_start_reserve`: the
town planner reserves cures (`kit_start_cure_reserve` x `kit_start_cure_hp`
hit points, as `town_heal_reserve_hp_percent` does) and Phase Door
(`kit_start_phase_reserve`, as `town_phase_reserve`); c, `kit_money_first`: at
CL up to `kit_money_first_max_level`, short of kit and too poor, no descent
below DL1 and a leash of at least `kit_money_leash`, then a trip to town once
it can pay. b changes the shopping list only while its reserve (60 HP of cures at the defaults) exceeds the class's own `town_heal_reserve_hp_percent` reserve (a Ranger's 200% of max HP): about CL1–3, where all 20 replay cases live; it prices each reserved hit point as the class reserve does (a larger total, not a dearer HP), so arm b is that reserve and nothing else. c needs `kit_leash` for its leash and the town hunt
(`town_hunt`) and money scum (`borg_money_scum_amount`) stay their own switches.

## Ranged reserve and town hunts (#789)

The registry now holds **2280** parameters (2276 before; four added).

| parameter | default | unit | range | source |
|---|---:|---|---|---|
| `town_ranged_reserve` | 0 | count | 0–100 | `borg_town_capability.rs:949` |
| `town_hunt` | 0 | flag | 0–1 | `borg_town_hunt.rs:22` |
| `town_hunt_ranged_bonus_percent` | 0 | percent | 0–1000 | `borg_town_hunt.rs:198` |
| `town_hunt_min_hp_percent` | 0 | percent of max HP | 0–100 | `borg_town_hunt.rs:140` |

The shopping reserve counts throw-ready oil and missiles for the equipped
launcher together. Oil allocated to lantern fuel stays available for light.
Purchases that fill the reserve come before other planned buys, after urgent
needs and inventory dispositions. Set `borg_param_town_ranged_reserve__<class>`
for a class override.

With `town_hunt=1`, the `town-melee-only` entries in the selected scary
list become hunt targets in town. The Borg checks the most dangerous
monster first. The #759 damage forecast governs attacks and approaches.
A ranged kill before contact can also qualify. The ranged bonus ranks
winnable attacks, and melee can still win the comparison. Below the HP floor
or in a losing close fight, the Borg tries the down staircase after emergency
caution and before the remaining flight actions. Farmer Maggot follows the
ignored-monster rule, including ordinary attacks when he blocks the way. See
`engine/angband/lib/gamedata/monster.txt:242–257,294–309,396–411,452–486` at
the engine pin for these monsters' attacks.

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
| `caution_stairs_clean_path` | 0 (shipped 1) | choice | 0–3 | `borg_caution.rs:990` |
| `caution_desperate_fast_danger_percent` | 0 | percent of HP | 0–10000 | `borg_caution.rs:6656` |
| `caution_desperate_ranged_first_reach` | 0 | grids | 0–10 | `borg_caution.rs:1004` |
| `caution_recall_cure_min_loss_percent` | 0 | percent of max HP | 0–100 | `borg_caution.rs:6918` |
| `caution_stairs_flight_ignore_danger` | 0 | flag | 0–1 | `borg_caution.rs:6634` |
| `caution_rising_hold_min_hp_percent` | 0 | percent of max HP | 0–100 | `borg_caution.rs:6079` |

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
| `caution_stairs_clean_path_flee` | 0 | flag | 0–1 | `borg_caution.rs:966` |

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
| `flow_leash_per_level` | 3 | steps per level | 0–100 | `borg_flow.rs:2369` |
| `flow_leash_base` | 9 | steps | 0–250 | `borg_flow.rs:2370` |
| `flow_leash_unbounded` | 250 | steps | 0–250 | `borg_flow.rs:2371` |
| `flow_leash_pickup_max_level` | 20 | level | 0–51 | `borg_flow.rs:2372` |
| `flow_leash_depth_margin` | 5 | levels | 0–127 | `borg_flow.rs:2373` |
| `flow_leash_twitch_after` | 21 | twitches | 0–1000 | `borg_flow.rs:2374` |
| `flow_leash_stairs_max_level` | 20 | level | 0–51 | `borg_flow.rs:2375` |
| `flow_leash_take_near_max_level` | 10 | level | 0–51 | `borg_flow.rs:2376` |
| `flow_leash_take_nearness` | 5 | steps | 0–250 | `borg_flow.rs:2377` |
| `flow_leash_take_max_level` | 20 | level | 0–51 | `borg_flow.rs:2378` |

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
| `caution_stairs_clean_path` | 0 (shipped 1) | flag | 0–1 | `borg_caution.rs:990` |
| `caution_stairs_block_danger_percent` | 50 | percent | 0–10000 | `borg_caution.rs:985` |
| `caution_immobile_step_to_exit` | 0 (shipped 1) | flag | 0–1 | `borg_caution.rs:2277` |
| `attack_mobile_threat_first` | 0 (shipped 1) | flag | 0–1 | `borg_fight_attack.rs:5433` |
| `attack_mobile_threat_percent` | 50 | percent | 0–10000 | `borg_fight_attack.rs:5434` |

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
| `town_ranger_arrows_max` | 0 | count | 0–1000 | `borg_town_capability.rs:1014` |
| `town_ranger_arrows_curve_scale_percent` | 0 | percent | 0–10000 | `borg_town_capability.rs:998` |

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
| `town_oil_apart` | 0 | flag | 0–1 | `borg_town_capability.rs:598` |
| `town_oil_flasks` | 10 | count | 1–255 | `borg_town_capability.rs:1104` |



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
## An invisible attacker near: a mode, and Detect Invisible (#831)

The registry now holds **2291** parameters (2283 before; eight added).
Owner, 2026-11-21: "when there is an invisible creature around, switch
mode and deal with it. Scroll of detect invisible are not so worthless after
all and make it that you can buy them if you have the spare money (or staff of
detect invisible), add some more value for them especially for blackguards and
those who do not have a spell to detect invisible, and if not available,
switch mode to knowing 'invisible creatures around' and stop resting until you
know that the invisible creature is gone and / or leave the level if
possible."

- **The cases** (borg-zoo#566, main `3b4924ba`): Human Blackguards 2078477958
  (CL2, DL1, game turn 14,317) and 2037495442 (CL5, DL1, 39,236) died to a
  clear icky thing (an invisible monster, touch 1d2) that "Something touches
  you" announced 23 and 44 times, each time followed by `# Resting to recover
  HP/SP...`. Blackguards have impaired HP regeneration (`class.txt`
  `IMPAIR_HP`), so no rest outpaces the touches. #697's switches existed but
  shipped off (`recover_unseen_attack_turns` 0).
- **The mode** is #697's state made explicit. `recover_unseen_attack_turns`
  (now **500** game turns) is its length: an unseen blow enters it
  (`# Mode: invisible attacker near (y,x)`), the last blow plus that many game
  turns, or a new level, ends it (`# Mode: invisible attacker gone ...`), and
  killing the attacker ends it too (`You have killed it.`, `# Mode: invisible
  attacker killed`); so does any death message of a monster the Borg may have
  seen (`You have slain ...`, `... dies.`, `... is destroyed.`) once a
  detection was read at some point in the episode (an attempt), the attacker
  then being probably revealed. The engine also gives those forms for an
  unseen monster shot at and for monsters killing each other; the episode then
  ends too and the next unseen blow starts a new one (`# Mode: over (a monster
  died after a detection was read)`, not `attacker killed`, is noted). While it lasts, every rest within
  `recover_unseen_attack_distance` grids of the blow is held (#697, unchanged).
  What is new is what the Borg does with a blow that has *just* struck
  (`recover_unseen_act_age`, **30** game turns: the attacker is next to it now):
  1. **Detect.** `recover_unseen_detect_reads` (**2** an episode): read Detect
     Invisible, quaff True Seeing, use a Staff of Detect Invisible or Detect
     Evil, an activation or cast Sense Invisible, Detection or Reveal Monsters
     (the list of `borg_defend_aux_inviso`, now shared:
     `detect_invisible_now`; not blind, confused, amnesic or in the dark). The
     revealed monster is a monster like any other: `borg_attack` fights it.
  2. **Strike where it must be.** `recover_unseen_probe_tries` (**10** an
     episode): `alter` (`+` and a direction) attacks a monster in the grid seen
     or not, and otherwise spends the turn without moving ("You spin around.",
     `cmd-cave.c:do_cmd_alter_aux:950-1001`, the monster branch first at 975-977,
     "You spin around." at 998). The candidates are the open adjacent
     grids (bare floor or a staircase, with no known object, trap or monster:
     alter would tunnel, open, close or disarm anything else), north first.
     One where a blow struck "something" is attacked again; one that spun is
     left out until the next blow (the attacker moves). Only while at most
     `recover_unseen_probe_max_grids` (**3**) grids are candidates (a corridor
     or a doorway; in the open it is guesswork and the Borg walks away
     instead); not afraid (the game refuses the blow), not confused, and not
     hurt enough to drink first.
  3. **Away, heal, leave** (#697's, `unseen_away` and `unseen_heal`, now
     **50%**): hurt and near the blow, walk at least 10 grids away and rest
     there; followed, or nowhere to walk, or **the blind attacks spent and it
     still strikes**, leave the level (`# Leaving (an unseen attacker, not
     found)`); below 50% of HP drink a cure.
- **Detect Invisible in the town plan.** #663 had put the scroll at 1% of the
  support weight ("never buy"). The first version made three reads a
  requirement behind a 250-gold floor; the owner found 3 too big a reserve and
  the floor flip-flopped (the smoke warrior had 401 gold, then 242, and sold the
  three scrolls it carried), so it is now a value curve and no reserve: with
  `town_invisible_count` (**2**) > 0 a read (scrolls at 15 gold, or a staff's
  charges) is worth a diminishing curve like Phase Door's and the flasks', the
  first value `town_invisible_percent` (**20**) of the trip's escape weight per
  read (a read's realised first value is that times count / (count + 1): 13.3%),
  half the most the curve pays at `town_invisible_count` reads, nothing a
  requirement, nothing kept aside; it competes with the rest of the purse and
  does not depend on the gold, so a carried read is never sold because the
  purse shrank. The percent depends on the class, by what it can already do
  about an invisible monster (`borg_magic`): **20** with nothing (Warriors,
  Blackguards, and others until they learn a spell); `town_invisible_evil_percent`
  (**10**) with Detect Evil (it reveals the evil ones, ghosts and a
  poltergeist, not a clear icky thing); `town_invisible_spell_percent` (**3**)
  with Sense Invisible, Detection or Reveal Monsters. 0 is #663. A Staff of
  Detect Invisible (200 gold, 8+d15 charges) gives the same units at far more
  gold, so it is bought only when scrolls are not stocked and the gold allows.
  In the dungeon it is not junk either: `junk_keep_some` (`borg_junk.rs`) keeps
  a known one whatever its price (the value tiers would crush a 15-gold item
  from depth 5 (the tier `depth < 10 && value > 15` fails at exactly 15) once dropping it raises `borg_power`; the pickup already walks
  to it, at the engine's price). Whether other cheap utility items suffer the
  same is borg-reborn-rs#846. A known Scroll of Detect Invisible is **never sold or dropped** by the town
  plan (`borg_town_utility.rs`, `keeps_detect_invisible`: it is kept, or stored
  at the Home while overweight), whatever the curve says; the one hard cap is
  the planner's own, `borg_town_max_non_missile_copies` (35) for every
  non-missile item, which the curve's diminishing value stays far below.
  - **First visits.** The old design needed a floor because a percent high
    enough to buy anything later also bought on the first visit: on the first
    visits of 2037495442 (151 gold) and 2078477958 (139 gold), CL1, 15% to 30%
    swapped a Cure Light Wounds or flasks for scrolls. With this curve, at 20%,
    both first visits buy exactly what the old plan bought (4 rations, flasks,
    3 Cure Light Wounds, 1 scroll).
- **Replay of the two cases** (from the zoo bundles of runs 44894233710 and
  44894253654: their `user/` files, `harness/borg.txt` of this tree with the
  cell's race and class and without its `borg_world` line; the old behaviour
  is these settings with the mode and the town plan off, and its key streams
  hash to the zoo's `b19d7b6a...` and `8acb6b08...` exactly):
  - **Old behaviour, shipped before the change:** both die as the zoo's did
    (game turns 14,317 and 39,236).
  - **This change:** both reach 100,000 game turns: 2078477958 at CL6 (max DL3),
    2037495442 at CL6 (max DL2). Both follow the old game exactly, first-visit
    purchases included, to the first unseen blow (game turns 5,384 and 3,532),
    then diverge. 2078477958 enters the mode three times, kills the attacker
    once (`# Mode: invisible attacker killed`), leaves the level once, and buys
    3 Scrolls of Detect Invisible at turn 50,982 (45 gold); 2037495442 enters it
    five times, strikes three times (`# Striking at ...`; one ends `killed`) and
    leaves once.
  - **#697's switches alone** also save both seeds (earlier local result:
    alive CL8 and CL6): the death was mainly a shipped-off setting.
  - **Not shown:** that 1,426 gold buys Detect Invisible in 2037495442's
    original game. That gold was found in the dungeon at about turn 38,000 and
    the original never returned to town; the harness has no way to give a
    Blackguard a purse. The valuation is checked by its unit tests and the
    identical first visits.
- **Local direction only, the first design (requirement and floor; not re-run)** (RULES 20): 12 Human Blackguards (seeds 7001..7012,
  zoo `reborn-head-shipped` settings, 100,000 game turns), the mode and the
  town plan off against on: deaths 0 and 0; sum of max CL 75 and 76; sum of
  max DL 34 and 34; HP rests 469 and 404; on, the mode was entered 29 times
  (32 blind attacks, 3 leaves, no detection: they carried none) and Detect
  Invisible was bought twice. No game stalled. This is no measure of benefit
  (RULES 14): in these 12 games the baseline did not die of an invisible
  monster either. A zoo arm against the shipped settings (the owner's) is the
  measurement: `borg_param_recover_unseen_attack_turns` 0 and 500 (with the
  mode's other settings), and `borg_param_town_invisible_count` 0 and 2.
- **Hypotheses, none measured:** all the numbers of this change. The mode's 500 is #697's example; the 30 game turns are three player
  turns at normal speed; 10 blind attacks cover a kill of a few hit points
  through the unseen penalty to hit.
- **Left:** an unseen caster ("Something mumbles", #699) is not a blow and
  does not enter the mode; the attack on the revealed monster is the ordinary
  fight's; Deep Descent and Recall as a way out are the level-leaving
  machinery's (`borg_leave_level`), not chosen here.

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
| `town_store_only_overweight` | 0 | flag | 0–1 | `borg_town_utility.rs:311` |

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
| `light_detect_evil_spell_min_level` | 0 | level | 0–51 | `borg_light.rs:550` |

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
| `escape_phase_lost_mana` | 1 | flag | 0–1 | `borg_escape.rs:1009` |


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
| `town_phase_curve_scale_percent` | 100 | percent | 1–10000 | `borg_town_capability.rs:998` |
| `town_oil_curve_scale_percent` | 100 | percent | 1–10000 | `borg_town_capability.rs:758` |

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
| `recover_rest_breeders` | 0 | flag | 0–1 | `borg_recover.rs:685` |

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
| `caution_breeder_stoppable` | 0 | flag | 0–1 | `borg_caution.rs:5707` |
| `caution_breeder_stoppable_births_percent` | 200 | percent | 1–10000 | `borg_caution.rs:5718` |

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
| `caution_breeder_avoid` | 0 | flag | 0–1 | `borg_caution.rs:5670` |
| `caution_breeder_avoid_radius` | 1 | grids | 0–10 | `borg_caution.rs:5671` |
| `caution_breeder_avoid_turns` | 50 | Borg turns | 1–100000 | `borg_caution.rs:5674` |
| `caution_breeder_avoid_fight_through` | 1 | flag | 0–1 | `borg_caution.rs:5935` |
| `caution_breeder_through_turns` | 5 | Borg turns | 0–1000 | `borg_caution.rs:5944` |
| `caution_breeder_stuck_turns` | 20 | Borg turns | 0–1000 | `borg_caution.rs:5918` |

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
| `caution_recall_rest_monster_turns` | 0 | game turns | 0–100000 | `borg_caution.rs:6804` |
| `caution_recall_rest_monster_distance` | 20 | grids | 1–100 | `borg_caution.rs:6807` |

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
| `caution_recall_wait_as_usual` | 0 | flag | 0–1 | `borg_caution.rs:5271` |

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
| `caution_stairs_leave_on_monster` | 0 | flag | 0–1 | `borg_caution.rs:6692` |
| `caution_stairs_leave_danger_percent` | 10 | percent of current HP | 0–1000 | `borg_caution.rs:6691` |
| `caution_stairs_leave_down` | 1 | flag | 0–1 | `borg_caution.rs:6698` |

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
| `escape_phase_adjacent_rule` | 0 | flag | 0–1 | `borg_escape.rs:902` |
| `escape_phase_lost_test` | 1 | choice | 0–2 | `borg_escape.rs:1009` |
| `escape_phase_lost_danger_percent` | 100 | percent of HP | 0–1000 | `borg_escape.rs:1010` |

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
| `escape_fox_form` | 0 | flag | 0–1 | `borg_escape.rs:524` |
| `escape_fox_form_fail` | 25 | percent | 0–100 | `borg_escape.rs:562` |
| `escape_fox_form_min_steps` | 3 | steps of flow | 1–250 | `borg_escape.rs:659` |
| `escape_fox_form_speed` | 10 | speed | 0–30 | `borg_escape.rs:715` |
| `escape_fox_form_stand_speed` | 10 | speed | 0–30 | `borg_escape.rs:717` |

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
| `caution_scary_hurt_percent` | 0 | percent of HP | 0–100 | `borg_caution.rs:6210` |
| `caution_scary_afraid` | 0 | flag | 0–1 | `borg_caution.rs:6210` |

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
  unseen attack (`borg_locate_kill`) arms it. The Light Beam defence
  (`borg_defend_aux_lbeam`) waits for the original 48-turn delay. With
  `light_hallway_repeat_guard=1` (shipped), it also requires a set stamp. Regional fear is kept while the stamp is recent
  (`borg_flow_kill`).

Compiled defaults are C's (0 and 1); `harness/borg.txt` ships 2 and 0. On
the same 18 starts: Detect Invisible read 95 times (0 in town) and bought
81 (1,245 gold against 1,830); Boldness bought 8 (64 gold against 280) and
drunk 10; no deaths in either; summed level 107 against 95, depth 43
against 38 (18 games: direction only).

| parameter | default | unit | range | source |
|---|---|---|---|---|
| `town_fear_cure_count` | 0 | count | 0–100 | `borg_town_capability.rs:1086` |
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
| `town_gold_reserve` | 0 | gold | 0–10000000 | `borg_town_budget.rs:268` |
| `town_gold_reserve_percent` | 0 | percent of the purse | 0–100 | `borg_town_budget.rs:264` |

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
| `town_heal_reserve_hp_percent` | 0 (off) | percent of max HP | 0–10000 | `borg_town_capability.rs:665` |
| `town_heal_speed_percent` | 0 (HP alone) | percent of max HP | 0–100 | `borg_town_capability.rs:522` |
| `town_heal_curve_scale_percent` | 100 (historical) | percent | 1–10000 | `borg_town_capability.rs:656` |

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
| `caution_rising_explore_ticks` | 300 | Borg ticks (one per think, `borg_think`; restarted on each new level) | 0–100000 | `borg_caution.rs:6620` (0 = off, C's behaviour) |
| `caution_rising_descend_danger_tenths` | 10 | tenths | 0–1000 | `borg_caution.rs:6625` (danger here above avoidance x value / 10 lets the descent through) |

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
| `caution_cornered` | 0 | flag | 0–1 | `borg_caution.rs:6374` (1 = on) |
| `caution_cornered_danger_tenths` | 10 | tenths | 0–1000 | `borg_caution.rs:6375` |
| `caution_cornered_radius` | 2 | grids | 0–100 | `borg_caution.rs:6377` |
| `caution_cornered_big_level_margin` | 0 | levels | 0–100 | `borg_caution.rs:6458` |
| `caution_cornered_spell_fail` | 25 | percent | 0–100 | `borg_caution.rs:6522` (buff spells only at or below this failure rate) |
| `caution_breeder_boost_danger_tenths` | 0 | tenths | 0–1000 | `borg_caution.rs:6429` (0 = off; owner: the same boosts in a hard breeder fight) |

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
| `town_heal_reserve_real_hp` | 0 | flag | 0–1 | `borg_town_capability.rs:573` |
| `town_digger_min_gold` | 0 | gold | 0–10000000 | `borg_town_capability.rs:1052` |
| `town_missiles_percent` | -1 | percent_override | -1–1000 | `borg_town_capability.rs:614` |
| `town_missiles_other_ranged_percent` | 100 | percent | 0–100 | `borg_town_capability.rs:985` |
| `town_devices_percent` | 100 | percent | 0–1000 | `borg_town_capability.rs:789` |
| `town_recall_reserve` | 0 | count | 0–5 | `borg_town_capability.rs:2524` (#557: this many Word of Recall are a requirement at the full trip weight each, at any depth; also in `borg_town_utility.rs`'s need model) |
| `town_recall_value_percent` | 100 | percent | 0–10000 | `borg_town_capability.rs:2395` (#557: a Word of Recall's value in percent of the walk it saves, in the capability and need models) |
| `prepared_recall_depth` | 5 | depth | 0–127 | `borg_prepared.rs:236` (#557: C's "1 recall" for a descent from this depth; 0 = never; gates descents only) |

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
| `caution_breeder_fight` | 0 | flag | 0–1 | `borg_caution.rs:5037` (1 = stand and fight breeders without an easy escape, melee classes hunt them) |
| `caution_breeder_easy_escape_steps` | 5 | grids | 0–100 | `borg_caution.rs:5038` |
| `caution_breeder_stand_radius` | 5 | grids | 0–100 | `borg_caution.rs:5379` |
| `caution_breeder_hunt_max_count` | 4 | count | 0–1000 | `borg_caution.rs:5711` |
| `caution_breeder_exit` | 0 | flag | 0–1 | `borg_caution.rs:5413` (#544 B: 1 = only breeders near and outmatched or surrounded, walk to a known chokepoint (corridor or doorway: its passable neighbours form two or more groups) away from every breeder) |
| `caution_breeder_exit_distance` | 6 | grids | 1–100 | `borg_caution.rs:5416` (#544 B: the exit's distance from every breeder) |
| `caution_flee_known_down_stairs` | 0 | flag | 0–1 | `borg_caution.rs:1043` (#544 A, narrowed in #548: 1 = fleeing with no up staircase known, a known down staircase, only cornered and within the cost cap below) |
| `caution_flee_known_down_cornered` | 1 | flag | 0–1 | `borg_caution.rs:1048` (#548: 1 = only cornered: a moving monster adjacent (distance 1; #504 uses `caution_cornered_radius`), no Phase Door, teleport, Teleport Level or escape item in the pack (spells not counted), danger here >= HP or surrounded; 0 = any flight, #545 as merged) |
| `caution_flee_known_down_max_cost` | 40 | steps of flow | 0–250 | `borg_caution.rs:1049` (#548: the staircase within this many steps of flow, the walk around walls, not the straight-line distance; 0 = any distance) |
| `caution_jumpstart_keep_stairs` | 0 | flag | 0–1 | `borg_caution.rs:7070` (#548, #544's K: 1 = the stuck jumpstart in `borg_think_dungeon.rs` keeps the known stairs; usually wanted with `caution_flee_known_down_stairs`) |
| `caution_twitch_avoid_lava` | 0 (shipped 1) | flag | 0–1 | `borg_caution.rs:5246` (#597: the twitch never steps into lava without fire immunity) |
| `caution_stair_parked_fallback` | 0 (shipped 1) | flag | 0–1 | `borg_caution.rs:5253` (#597: a monster on every known staircase no longer hides them from the stair flows) |
| `caution_stair_scum_threat` | 0 | flag | 0–1 | `borg_caution.rs:6042` (#549: 1 = after fleeing town, no DL1 stair scum while threatened: scary level, danger here, or hurt; the HP condition is the broad one: any injury stops the scum, and the danger limit only adds grids that are dangerous at full HP. A threat also cancels the scum's own `GOAL_TAKE` flow) |
| `caution_stair_scum_danger` | 0 | danger | 0–10000 | `borg_caution.rs:6046` (#549: the danger here above which the stair scum stops) |
| `caution_rising_hold_flee_descent` | 0 | flag | 0–1 | `borg_caution.rs:6073` (#557: 1 = while rising in the dungeon, no flight down a staircase unless danger here >= HP: `choose_stairs`, `borg_flow_stair_both`, `borg_flow_stair_more(GOAL_FLEE)`; the held staircase is no easy escape in #530's stand) |
| `caution_flee_recall` | 1 (shipped 1; borg-zoo#255) | flag | 0–1 | `borg_caution.rs:4924` (#557: 0 = a flight reads Word of Recall only on a breeder-swamped level or when chased; Recall then is no easy escape in the stand tests) |
| `caution_flee_recall_breeders` | 5 | monsters | 0–100 | `borg_caution.rs:4960` |
| `caution_flee_recall_chasers` | 3 | monsters | 0–100 | `borg_caution.rs:4961` |
| `caution_flee_recall_chase_distance` | 6 | grids | 1–30 | `borg_caution.rs:4944` |
| `caution_flee_recall_gate_below_depth` | 0 (shipped 1, i.e. off) | dungeon level | 0–127 | `borg_caution.rs:4926` (#557: a ceiling; with `caution_flee_recall` 1, at depths shallower than this a flight's Recall needs the breeder or chaser exception; 2 = DL1 only; 0 or 1 = off) |
| `caution_flee_recall_danger_percent` | 0 | percent of HP | 0–10000 | `borg_caution.rs:4935` (#587: where the gate holds a flight's Recall, danger here at this share of HP allows it; 0 = off; inert at the shipped gate) |
| `caution_scary_named_monsters` | 1 | flag | 0–1 | `borg_caution.rs:6122` (0 = off; read in `borg_flow_kill.rs`, `borg_update.rs`, `borg_messages.rs`) |

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
| `caution_scary_stand` | 0 | flag | 0–1 | `borg_caution.rs:5075` (read in `borg_flow_kill.rs`) |

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
| `caution_scary_latch` | 1 | flag | 0–1 | `borg_caution.rs:5102` |
| `caution_scary_unlatch_kill` | 0 | flag | 0–1 | `borg_caution.rs:5083` (#590: with `caution_scary_stand`, clear once the last scary monster known was killed, latch on or off; forgetting is not enough) |
| `caution_scary_unlatch_calmed` | 0 | flag | 0–1 | `borg_caution.rs:5084` (#593: with `caution_scary_stand`, clear once a named-list or damage-rule scary monster is known hurt or afraid, or killed, latch on or off) |

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
| `caution_scary_damage_percent` | 0 | percent of max HP | 0–1000 | `borg_caution.rs:6186` (0 = off; read in `borg_flow_kill.rs`) |

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
| `caution_heal_spell_fail` | 15 | percent | 0–100 | `borg_caution.rs:262` |
| `caution_clw_estimate_percent` | 15 | percent | 0–100 | `borg_caution.rs:264` |
| `caution_clw_estimate_min` | 15 | hp | 0–10000 | `borg_caution.rs:265` |
| `caution_csw_estimate_percent` | 20 | percent | 0–100 | `borg_caution.rs:269` |
| `caution_csw_estimate_min` | 25 | hp | 0–10000 | `borg_caution.rs:270` |
| `caution_ccw_estimate_percent` | 25 | percent | 0–100 | `borg_caution.rs:274` |
| `caution_ccw_estimate_min` | 30 | hp | 0–10000 | `borg_caution.rs:275` |
| `caution_heal_estimate_percent` | 35 | percent | 0–100 | `borg_caution.rs:280` |
| `caution_heal_estimate_min` | 300 | hp | 0–10000 | `borg_caution.rs:281` |
| `caution_healing_rod_fail` | 500 | permille | 0–1000 | `borg_caution.rs:289` |
| `caution_ranged_vs_immobile` | 0 (shipped 0, #736) | mode | 0–3 | `borg_caution.rs:1822` |
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
| `caution_desperate_stair_distance` | 3 | grids | 1–100 | `borg_caution.rs:1069` |
| `caution_desperate_adjacent_up_distance` | 2 | grids | 1–100 | `borg_caution.rs:1066` |
| `caution_desperate_scary_distance` | 20 | grids | 1–100 | `borg_caution.rs:1088` |
| `caution_desperate_scary_level` | 20 | level | 1–51 | `borg_caution.rs:1092` |
| `caution_desperate_unique_depth` | 5 | depth | 0–127 | `borg_caution.rs:1086` |
| `caution_desperate_adjacent_down_distance` | 2 | grids | 1–100 | `borg_caution.rs:1123` |
| `caution_retreat_max_distance` | 10 | grids | 1–100 | `borg_caution.rs:1229` |
| `caution_retreat_step_hp_tenths` | 4 | tenths | 0–100 | `borg_caution.rs:1276` |
| `caution_retreat_step_danger_tenths` | 5 | tenths | 0–100 | `borg_caution.rs:1163` |
| `caution_retreat_surrounded_level` | 20 | level | 1–51 | `borg_caution.rs:1168` |
| `caution_retreat_surrounded_distance` | 5 | grids | 1–100 | `borg_caution.rs:1169` |
| `caution_retreat_surrounded_hp_tenths` | 15 | tenths | 0–100 | `borg_caution.rs:1170` |
| `caution_retreat_low_level_hp_multiplier` | 4 | multiplier | 0–20 | `borg_caution.rs:1179` |
| `caution_retreat_monster_age` | 2 | turns | 0–100000 | `borg_caution.rs:1197` |
| `caution_back_surrounded_danger_tenths` | 12 | tenths | 0–100 | `borg_caution.rs:1588` |
| `caution_back_monster_age` | 2 | turns | 0–100000 | `borg_caution.rs:1658` |
| `caution_back_fast_monster_distance` | 2 | grids | 1–100 | `borg_caution.rs:1669` |
| `caution_back_high_level` | 35 | level | 1–51 | `borg_caution.rs:1695` |
| `caution_back_high_level_danger_tenths` | 6 | tenths | 0–100 | `borg_caution.rs:1696` |
| `caution_back_low_level` | 35 | level | 1–51 | `borg_caution.rs:1705` |
| `caution_back_low_level_danger_tenths` | 8 | tenths | 0–100 | `borg_caution.rs:1707` |
| `caution_back_injured_level` | 10 | level | 1–51 | `borg_caution.rs:1729` |
| `caution_back_freedom_depth` | 85 | depth | 0–127 | `borg_caution.rs:1738` |
| `caution_move_antisummon_delay` | 50 | turns | 0–100000 | `borg_caution.rs:2325` |
| `caution_move_max_hp` | 500 | hp | 0–10000 | `borg_caution.rs:2328` |
| `caution_one_blow_monster_age` | 2 | turns | 0–100000 | `borg_caution.rs:2430` |
| `caution_nasty_light_timeout` | 250 | turns | 0–100000 | `borg_caution.rs:4092` |
| `caution_questor_arrival_turns` | 10 | turns | 0–100000 | `borg_caution.rs:4086` |
| `caution_escape_limit` | 3 | count | 0–1000 | `borg_caution.rs:4140` |
| `caution_escape_absolute_limit` | 55 | count | 0–1000 | `borg_caution.rs:4141` |
| `caution_escape_limit_max_depth` | 98 | depth | 0–127 | `borg_caution.rs:4143` |
| `caution_escape_flee_limit` | 3 | count | 0–1000 | `borg_caution.rs:4087` |
| `caution_swap_danger_divisor` | 3 | divisor | 1–100 | `borg_caution.rs:4245` |
| `caution_recall_prep_turns` | 2 | turns | 0–100000 | `borg_caution.rs:4278` |
| `caution_restock_town_delay` | 200 | turns | 0–100000 | `borg_caution.rs:4334` |
| `caution_restock_ccw_count` | 2 | count | 0–1000 | `borg_caution.rs:4306` |
| `caution_restock_food_count` | 3 | count | 0–1000 | `borg_caution.rs:4307` |
| `caution_restock_fuel_count` | 2 | count | 0–1000 | `borg_caution.rs:4308` |
| `caution_restock_level_delay` | 400 | turns | 0–100000 | `borg_caution.rs:4309` |
| `caution_flee_danger_hp_multiplier` | 2 | multiplier | 0–20 | `borg_caution.rs:3960` |
| `caution_flee_max_level` | 50 | level | 1–51 | `borg_caution.rs:4325` |
| `caution_town_flee_max_level` | 50 | level | 1–51 | `borg_caution.rs:4339` |
| `caution_stair_low_level` | 5 | level | 1–51 | `borg_caution.rs:4371` |
| `caution_stair_danger_divisor` | 2 | divisor | 1–100 | `borg_caution.rs:4372` |
| `caution_breeder_flee_max_level` | 15 | level | 1–51 | `borg_caution.rs:4136` |
| `caution_desperate_max_level` | 35 | level | 1–51 | `borg_caution.rs:4478` |
| `caution_retreat_danger_divisor` | 3 | divisor | 1–100 | `borg_caution.rs:7039` |
| `caution_back_danger_tenths` | 4 | tenths | 0–100 | `borg_caution.rs:4299` |
| `caution_confused_emergency_loss` | 300 | hp | 0–10000 | `borg_caution.rs:4606` |
| `caution_fear_cure_chance` | 70 | percent | 0–100 | `borg_caution.rs:4691` |
| `caution_fear_berserk_fail` | 25 | percent | 0–100 | `borg_caution.rs:4083` |
| `caution_fear_hero_fail` | 25 | percent | 0–100 | `borg_caution.rs:4084` |
| `caution_fear_holy_fail` | 25 | percent | 0–100 | `borg_caution.rs:4091` |
| `caution_flee_low_hp_divisor` | 3 | divisor | 1–100 | `borg_caution.rs:4099` |
| `caution_flee_moderate_hp_divisor` | 2 | divisor | 1–100 | `borg_caution.rs:4100` |
| `caution_flee_hp_per_level` | 3 | hp_per_level | 0–100 | `borg_caution.rs:4101` |
| `caution_flee_ccw_count` | 3 | count | 0–1000 | `borg_caution.rs:4102` |
| `caution_flee_heal_count` | 1 | count | 0–1000 | `borg_caution.rs:4103` |
| `caution_flee_low_hp_chance` | 25 | percent | 0–100 | `borg_caution.rs:4106` |
| `caution_flee_status_hp_divisor` | 2 | divisor | 1–100 | `borg_caution.rs:4114` |
| `caution_flee_status_chance` | 25 | percent | 0–100 | `borg_caution.rs:4117` |
| `caution_reserve_critical_hp_divisor` | 10 | divisor | 1–100 | `borg_caution.rs:4210` |
| `caution_reserve_escape_count` | 2 | count | 0–1000 | `borg_caution.rs:4212` |
| `caution_reserve_danger_hp_divisor` | 4 | divisor | 1–100 | `borg_caution.rs:4213` |
| `caution_reserve_heal_count` | 5 | count | 0–1000 | `borg_caution.rs:4215` |
| `caution_reserve_surplus_hp_divisor` | 4 | divisor | 1–100 | `borg_caution.rs:4215` |
| `caution_reserve_unique_loss` | 600 | hp | 0–10000 | `borg_caution.rs:4217` |
| `caution_reserve_unique_depth` | 85 | depth | 0–127 | `borg_caution.rs:4219` |
| `caution_recall_danger_multiplier` | 2 | multiplier | 0–20 | `borg_caution.rs:4256` |
| `caution_stair_danger_max_level` | 49 | level | 1–51 | `borg_caution.rs:6737` |
| `caution_stair_min_food` | 2 | count | 0–1000 | `borg_caution.rs:6742` |
| `caution_sell_max_level` | 25 | level | 1–51 | `borg_caution.rs:6750` |
| `caution_sell_max_gold` | 25000 | gold | 0–1000000 | `borg_caution.rs:6757` |
| `caution_sell_min_count` | 13 | count | 0–1000 | `borg_caution.rs:6758` |
| `caution_recall_mana_maxsp` | 60 | sp | 0–10000 | `borg_caution.rs:6903` |
| `caution_recall_mana_divisor` | 4 | divisor | 1–100 | `borg_caution.rs:6904` |
| `caution_recall_phase_tolerance` | 50 | percent | 0–100 | `borg_caution.rs:6917` |
| `caution_recall_phase_horizon` | 1 | turns | 1–100000 | `borg_caution.rs:6898` |
| `caution_recall_phase_fail` | 30 | percent | 0–100 | `borg_caution.rs:6903` |
| `caution_recall_portal_fail` | 30 | percent | 0–100 | `borg_caution.rs:6904` |
| `caution_recall_cure_loss` | 100 | hp | 0–10000 | `borg_caution.rs:6917` |
| `caution_recall_heal_loss` | 150 | hp | 0–10000 | `borg_caution.rs:6947` |
| `caution_retreat_danger_horizon` | 1 | turns | 1–10 | `borg_caution.rs:1238` |
| `caution_back_danger_horizon` | 1 | turns | 1–10 | `borg_caution.rs:1683` |
| `caution_danger_horizon` | 1 | turns | 1–10 | `borg_caution.rs:4129` |
| `caution_recall_rest_turns` | 500 | turns | 2–9999 | `borg_caution.rs:4264` |
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
| `danger_low_hp_speed_limit` | 20 | hp | 0–10000 | `borg_danger.rs:2180` |
| `danger_low_hp_monster_speed_bonus` | 3 | speed | 0–3 | `borg_danger.rs:2181` |
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
| `danger_frequency_low_percent` | 11 | percent | 0–100 | `borg_danger.rs:2087` |
| `danger_frequency_low_tenths` | 4 | tenths | 0–30 | `borg_danger.rs:2088` |
| `danger_frequency_mid_percent` | 26 | percent | 0–100 | `borg_danger.rs:2089` |
| `danger_frequency_mid_tenths` | 6 | tenths | 0–30 | `borg_danger.rs:2090` |
| `danger_frequency_high_percent` | 51 | percent | 0–100 | `borg_danger.rs:2091` |
| `danger_frequency_high_tenths` | 8 | tenths | 0–30 | `borg_danger.rs:2092` |
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
| `escape_recall_fail` | 60 | percent | 0–100 | `borg_escape.rs:116` |
| `escape_recall_depth_window` | 5 | depth | 0–127 | `borg_escape.rs:112` |
| `escape_recall_reset_min_depth` | 80 | depth | 0–127 | `borg_escape.rs:116` |
| `escape_recall_reset_unique_count` | 5 | count | 0–1000 | `borg_escape.rs:128` |
| `escape_surround_distance` | 3 | grids | 1–100 | `borg_escape.rs:198` |
| `escape_surround_single_exit_count` | 1 | count | 1–8 | `borg_escape.rs:266` |
| `escape_surround_single_monster_count` | 1 | count | 1–8 | `borg_escape.rs:266` |
| `escape_town_stair_value` | 1000 | freedom_score | 0–5000 | `borg_escape.rs:333` |
| `escape_town_stair_close_distance` | 4 | grids | 1–100 | `borg_escape.rs:336` |
| `escape_town_stair_close_value` | 2000 | freedom_score | 0–5000 | `borg_escape.rs:337` |
| `escape_town_stair_distance_penalty` | 500 | freedom_score | 0–5000 | `borg_escape.rs:337` |
| `escape_dungeon_stair_value` | 1000 | freedom_score | 0–5000 | `borg_escape.rs:331` |
| `escape_dungeon_stair_close_distance` | 4 | grids | 1–100 | `borg_escape.rs:334` |
| `escape_dungeon_stair_close_value` | 2000 | freedom_score | 0–5000 | `borg_escape.rs:335` |
| `escape_dungeon_stair_distance_penalty` | 500 | freedom_score | 0–5000 | `borg_escape.rs:335` |
| `escape_landing_samples` | 100 | count | 1–1000 | `borg_escape.rs:379` |
| `escape_landing_trials` | 100 | count | 1–1000 | `borg_escape.rs:393` |
| `escape_unknown_landing_max_hp` | 30 | hp | 0–10000 | `borg_escape.rs:419` |
| `escape_unknown_teleport_level_turns` | 2000 | turns | 0–100000 | `borg_escape.rs:1574` |
| `escape_shadow_shift_min_hp` | 12 | hp | 0–10000 | `borg_escape.rs:1822` |
| `escape_dimension_door_range` | 50 | grids | 1–100 | `borg_escape.rs:1831` |
| `escape_dimension_door_horizon` | 2 | turns | 1–10 | `borg_escape.rs:1852` |
| `escape_near_unique_max_count` | 8 | count | 1–9 | `borg_escape.rs:1922` |
| `escape_normal_fail` | 25 | percent | 0–100 | `borg_escape.rs:1933` |
| `escape_healthy_hp_percent` | 70 | percent | 0–100 | `borg_escape.rs:1936` |
| `escape_healthy_fail` | 10 | percent | 0–100 | `borg_escape.rs:1937` |
| `escape_stunned_fail` | 35 | percent | 0–100 | `borg_escape.rs:1942` |
| `escape_morgoth_hold_hp_tenths` | 5 | tenths | 0–100 | `borg_escape.rs:1941` |
| `escape_morgoth_hold_glyph_count` | 3 | count | 0–1000 | `borg_escape.rs:1965` |
| `escape_risky_danger_extra_tenths` | 3 | tenths | 0–100 | `borg_escape.rs:1999` |
| `escape_emergency_danger_tenths` | 45 | tenths | 0–100 | `borg_escape.rs:1935` |
| `escape_morgoth_danger_tenths` | 40 | tenths | 0–100 | `borg_escape.rs:1936` |
| `escape_morgoth_max_hp` | 600 | hp | 0–10000 | `borg_escape.rs:1939` |
| `escape_sauron_danger_tenths` | 30 | tenths | 0–100 | `borg_escape.rs:1940` |
| `escape_sauron_max_hp` | 600 | hp | 0–10000 | `borg_escape.rs:1943` |
| `escape_deep_unique_danger_tenths` | 25 | tenths | 0–100 | `borg_escape.rs:1944` |
| `escape_deep_unique_depth` | 95 | depth | 0–127 | `borg_escape.rs:1946` |
| `escape_deep_unique_max_hp` | 550 | hp | 0–10000 | `borg_escape.rs:1947` |
| `escape_unique_danger_tenths` | 17 | tenths | 0–100 | `borg_escape.rs:1948` |
| `escape_ordinary_danger_tenths` | 15 | tenths | 0–100 | `borg_escape.rs:1949` |
| `escape_emergency_fail` | 15 | percent | 0–100 | `borg_escape.rs:1951` |
| `escape_initial_fail_discount` | 10 | percent | 0–100 | `borg_escape.rs:1955` |
| `escape_emergency_fail_extra` | 9 | percent | 0–100 | `borg_escape.rs:1964` |
| `escape_emergency_phase_tolerance` | 75 | count | 0–1000 | `borg_escape.rs:1974` |
| `escape_emergency_phase_horizon` | 2 | turns | 1–10 | `borg_escape.rs:1974` |
| `escape_last_ditch_max_level` | 10 | level | 1–51 | `borg_escape.rs:1993` |
| `escape_last_ditch_hp_divisor` | 10 | divisor | 1–1000 | `borg_escape.rs:1994` |
| `escape_last_ditch_dimension_fail` | 90 | percent | 0–100 | `borg_escape.rs:1996` |
| `escape_critical_phase_hp_divisor` | 10 | divisor | 1–1000 | `borg_escape.rs:2008` |
| `escape_last_ditch_phase_max_level` | 10 | level | 1–51 | `borg_escape.rs:2022` |
| `escape_last_ditch_phase_hp_divisor` | 10 | divisor | 1–1000 | `borg_escape.rs:2023` |
| `escape_last_ditch_phase_fail` | 15 | percent | 0–100 | `borg_escape.rs:2025` |
| `escape_endgame_stay_danger_tenths` | 25 | tenths | 0–100 | `borg_escape.rs:2040` |
| `escape_endgame_stay_unique_count` | 3 | count | 1–9 | `borg_escape.rs:2042` |
| `escape_endgame_stay_min_depth` | 97 | depth | 0–127 | `borg_escape.rs:2043` |
| `escape_mage_danger_tenths` | 3 | tenths | 0–100 | `borg_escape.rs:2052` |
| `escape_mage_max_sp` | 20 | sp | 0–10000 | `borg_escape.rs:2054` |
| `escape_mage_min_level` | 45 | level | 1–51 | `borg_escape.rs:2055` |
| `escape_teleport_unique_danger_tenths` | 13 | tenths | 0–100 | `borg_escape.rs:2056` |
| `escape_teleport_ordinary_danger_tenths` | 11 | tenths | 0–100 | `borg_escape.rs:2057` |
| `escape_teleport_phase_tolerance` | 50 | count | 0–1000 | `borg_escape.rs:2083` |
| `escape_teleport_phase_horizon` | 2 | turns | 1–10 | `borg_escape.rs:2083` |
| `escape_phase_antisummon_delay` | 50 | turns | 0–100000 | `borg_escape.rs:2084` |
| `escape_phase_unique_danger_tenths` | 13 | tenths | 0–100 | `borg_escape.rs:2101` |
| `escape_phase_unique_min_count` | 2 | count | 1–9 | `borg_escape.rs:2101` |
| `escape_phase_ordinary_danger_tenths` | 10 | tenths | 0–100 | `borg_escape.rs:2102` |
| `escape_afraid_warrior_danger_tenths` | 10 | tenths | 0–100 | `borg_escape.rs:2103` |
| `escape_phase_tolerance` | 25 | count | 0–1000 | `borg_escape.rs:2159` |
| `escape_phase_horizon` | 2 | turns | 1–10 | `borg_escape.rs:2159` |
| `escape_fallback_phase_tolerance` | 75 | count | 0–1000 | `borg_escape.rs:1929` |
| `escape_fallback_phase_horizon` | 2 | turns | 1–10 | `borg_escape.rs:1929` |
| `escape_failed_teleport_flee_max_level` | 35 | level | 1–51 | `borg_escape.rs:1959` |
| `escape_failed_teleport_leave_max_level` | 35 | level | 1–51 | `borg_escape.rs:1971` |
| `escape_compromised_danger_tenths` | 8 | tenths | 0–100 | `borg_escape.rs:1983` |
| `escape_compromised_max_level` | 35 | level | 1–51 | `borg_escape.rs:1984` |
| `escape_compromised_hp_divisor` | 3 | divisor | 1–1000 | `borg_escape.rs:1984` |
| `escape_compromised_unique_danger_tenths` | 9 | tenths | 0–100 | `borg_escape.rs:1985` |
| `escape_compromised_unique_max_level` | 35 | level | 1–51 | `borg_escape.rs:1987` |
| `escape_compromised_unique_hp_divisor` | 3 | divisor | 1–1000 | `borg_escape.rs:1987` |
| `escape_compromised_ordinary_danger_tenths` | 6 | tenths | 0–100 | `borg_escape.rs:1988` |
| `escape_compromised_ordinary_max_level` | 20 | level | 1–51 | `borg_escape.rs:1988` |
| `escape_compromised_general_danger_tenths` | 6 | tenths | 0–100 | `borg_escape.rs:1989` |
| `escape_compromised_general_max_level` | 35 | level | 1–51 | `borg_escape.rs:1989` |
| `escape_compromised_phase_tolerance` | 20 | count | 0–1000 | `borg_escape.rs:1993` |
| `escape_compromised_phase_horizon` | 2 | turns | 1–10 | `borg_escape.rs:1993` |
| `escape_failed_compromised_flee_max_level` | 25 | level | 1–51 | `borg_escape.rs:2032` |
| `escape_weak_caster_max_level` | 35 | level | 1–51 | `borg_escape.rs:2052` |
| `escape_weak_caster_phase_tolerance` | 65 | count | 0–1000 | `borg_escape.rs:2053` |
| `escape_weak_caster_phase_horizon` | 2 | turns | 1–10 | `borg_escape.rs:2053` |
| `escape_low_level_max_level` | 10 | level | 1–51 | `borg_escape.rs:2085` |
| `escape_low_level_danger_tenths` | 5 | tenths | 0–100 | `borg_escape.rs:2086` |
| `escape_low_level_unique_danger_tenths` | 7 | tenths | 0–100 | `borg_escape.rs:2086` |
| `escape_low_level_phase_tolerance` | 20 | count | 0–1000 | `borg_escape.rs:2090` |
| `escape_low_level_phase_horizon` | 2 | turns | 1–10 | `borg_escape.rs:2090` |
| `escape_low_caster_max_level` | 8 | level | 1–51 | `borg_escape.rs:2345` |
| `escape_low_caster_phase_tolerance` | 65 | count | 0–1000 | `borg_escape.rs:2346` |
| `escape_low_caster_phase_horizon` | 2 | turns | 1–10 | `borg_escape.rs:2346` |
| `escape_low_mana_danger_tenths` | 6 | tenths | 0–100 | `borg_escape.rs:2367` |
| `escape_low_mana_unique_danger_tenths` | 8 | tenths | 0–100 | `borg_escape.rs:2367` |
| `escape_low_mana_sp_divisor` | 10 | divisor | 1–1000 | `borg_escape.rs:2368` |
| `escape_low_mana_min_maxsp` | 100 | sp | 0–10000 | `borg_escape.rs:2368` |
| `escape_low_mana_phase_tolerance` | 20 | count | 0–1000 | `borg_escape.rs:2372` |
| `escape_low_mana_phase_horizon` | 2 | turns | 1–10 | `borg_escape.rs:2372` |
| `escape_scoot_tolerance` | 20 | count | 0–1000 | `borg_escape.rs:2412` |
| `escape_scoot_horizon` | 2 | turns | 1–10 | `borg_escape.rs:2412` |
| `escape_twitch_limit` | 50 | count | 0–1000 | `borg_escape.rs:2427` |
| `escape_twitch_phase_tolerance` | 20 | count | 0–1000 | `borg_escape.rs:2430` |
| `escape_twitch_phase_horizon` | 2 | turns | 1–10 | `borg_escape.rs:2430` |
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
| `recover_rest_high_depth` | 85 | depth | 0–127 | `borg_recover.rs:1315` |
| `recover_rest_high_sp_tenths` | 7 | tenths | 0–10 | `borg_recover.rs:1316` |
| `recover_rest_sp_tenths` | 6 | tenths | 0–10 | `borg_recover.rs:1315` |
| `recover_mana_rest_max_level` | 40 | level | 1–51 | `borg_recover.rs:1325` |
| `recover_mana_rest_deep_depth` | 85 | depth | 0–127 | `borg_recover.rs:1323` |
| `recover_mana_rest_sp_tenths` | 8 | tenths | 0–10 | `borg_recover.rs:1324` |
| `recover_mana_rest_danger_divisor` | 10 | divisor | 1–1000 | `borg_recover.rs:463` |
| `recover_mana_rest_min_food` | 2 | count | 0–1000 | `borg_recover.rs:469` |
| `recover_munchkin_rest_min_food` | 2 | count | 0–1000 | `borg_recover.rs:492` |
| `recover_rod_rest_turns` | 100 | turns | 2–9999 | `borg_recover.rs:404` |

## Named decision trace (P1.4)

The vendored core writes `trace-core.jsonl` and death dossiers retain its
last 200 rows. See [CORE-TRACE.md](CORE-TRACE.md) for the schema, sampling
point and opt-out, and [P1.4-VALIDATION.md](validation/P1.4-VALIDATION.md) for default
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
See [M13 integration](validation/M13-INTEGRATION.md) for the complete list and sources.
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
| `attack_speed_fight_danger_tenths` | 0 (off) | tenths | 0–1000 | `borg_fight_attack.rs:6276` |

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
| `prepared_clevel_depth_margin` | 0 | levels | -50–50 | `borg_prepared.rs:718` |
| `prepared_dl2_min_hp` | 30 | hp | 0–10000 | `borg_prepared.rs:257` |
| `prepared_dl34_min_hp` | By class | hp | 0–10000 | `borg_prepared.rs:271` |
| `prepared_dl34_min_clevel` | By class | level | 0–50 | `borg_prepared.rs:275` |
| `prepared_cure_min` | 2 | potions | 0–99 | `borg_prepared.rs:283` |
| `prepared_cure_until_clevel` | 30 | level | 0–50 | `borg_prepared.rs:284` |
| `prepared_dl59_min_hp` | By class | hp | 0–10000 | `borg_prepared.rs:299` |
| `prepared_dl59_min_clevel` | By class | level | 0–50 | `borg_prepared.rs:303` |
| `prepared_mid_clevel_offset` | By class | levels | -50–50 | `borg_prepared.rs:372` |
| `prepared_mid_max_depth` | 19 | depth | 0–127 | `borg_prepared.rs:370` |
| `prepared_mid_caster_max_clevel` | 28 | level | 0–50 | `borg_prepared.rs:368` |
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
| `caution_recall_pending_escape` | 0 | flag | 0–1 | `borg_caution.rs:5111` |
| `caution_recall_wait_no_explore` | 0 | flag | 0–1 | `borg_caution.rs:5306` |

## Attack categories and winning fights (#760, #759)

These independent switches compile to **0** and ship **1** by the owner's
2026-11-20 adoption decision. Their enabled
behavior and class ranges are hypotheses pending the paired validation;
no improvement is claimed from implementing the policy.

| Parameter | Compiled / shipped | Meaning |
| --- | --- | --- |
| `attack_on_sight` | 0 / 1 | Give an active attack category priority before flight, and exclude that monster from scare classification. |
| `caution_winning_fight` | 0 / 1 | Attack while turns to kill all attackers are strictly fewer than turns to die; otherwise use ordinary caution. |

`harness/scary_monsters.txt` remains the common database. Its new form is
`monster | level | class exceptions | attack[,breeders] | reason`.
Scare, ignore and attack entries replace/remove only their own kind. An
attack entry requires a reason; `monster | none | | attack` removes it.
The existing class interpolation and explicit class ceilings apply.
The new entries use CL50 for all classes, an unmeasured range rather than
an inferred class-specific cutoff. The reasons are `fast-fragile` (wild
cat, crow) and `breeder` (giant white mouse, giant white louse, giant fruit
fly, white/green worm masses). The file cites the engine data, including
MULTIPLY inherited from the insect base. Breeder entries defer to the
existing stand/stoppable-colony policy; a lost colony is still fled.
Spiders and crow of Durthang are deferred: groups, poison and WOUND need
separate evidence. This does not implement the YAML migration of #678.

The fight rung runs before light/countdown and depth-return flight rungs,
so they cannot preempt a winning fight. The focused planner considers every
native attack kind against the chosen
observed target, retaining availability and projection checks. Both the category and
winning-fight policies maximize expected damage across families. The requested
missile, spell/device, throw, melee ordering breaks damage ties; it never
puts a weak projectile ahead of a stronger available attack.
`caution_winning_fight=2` is a diagnostic logging mode: it evaluates the
original control fights without taking the priority attack. Its key/state
neutrality must be checked; it is not a measurement arm.
The ordinary attack preference weights are utility, not damage, and are
not used to certify a win. The dry run restores Borg RNG and temporary
simulation/target state before any real action.

The fight comparison retains exact HP/max HP from the same permitted
`set_health` observation, with no additional engine read. The health-bar
`kill.power` has already applied injury and can round down; it is not used
to certify a win. A monster at zero HP still needs one damage point
(`mon-util.c:1300`); the comparison therefore uses observed HP plus one.
Regeneration is charged at its continuous mean from the start of the fight
(`mon-move.c:1842-1853`), conservatively. Overkill is capped before hit
probability, so it cannot erase the chance of missing a finishing hit. Melee and missile
hit chances come from `player-attack.c:196-219` at `574b618c`; throws use
that file's throwing damage/hit mechanics. Critical bonuses are omitted.
The model reads the player's own computed `num_blows` (hundredths),
`num_shots` (tenths), and displayed speed through the engine boundary
(`player-attack.c:991-1021,1087`). It converts per-hit damage to a normal
100-energy Borg turn. A finite arrow stack pays for individual fractional
shots, not that many whole turns of fire. Fixed-blow weapon spells use a
guaranteed one-blow lower bound; unmodelled projections/self-costs cannot
certify a win. The early priority action still decrements `no_retreat`.

Mana, ammunition, thrown stack sizes and device uses are finite; after a
finite primary the estimate uses the available unlimited fallback. Uses
are consumed as whole actions across foes, with a shared mana budget.
The incoming sum is retained throughout the fight as foes die. Observed
monster Haste is added to base speed; Slow is not credited as persisting
throughout a forecast. The focused executor preserves the native free
fox-to-normal revert before melee and recomputes the plan before attacking. Plain HURT
blows use their dice mean, hit chance and monster/player energy ratio;
armor's damage reduction is omitted conservatively. Unmodelled status or
spell contributions prevent a winning certificate; native fear scores are
not quantities of HP and are not included in that race. Poison, bleeding,
confused commands, an attacker without a current observable target, unknown HP or an unmodelled damage projection cannot
certify a winning fight. These bounds can decline a fight that is actually
winnable; they are not a proof against a run of misses.

Validation starts from clean `08f94e1fed0b80fd854584d893209983079157ea`,
borg-rs core, shipped settings. All six owner-specified deaths reproduced
with the supplied off arm's exact key streams before implementation.
Named-case validation is complete; canonical details are
[borg-cemetery PR #88](https://github.com/diegoami/borg-cemetery/pull/88),
`studies/reborn-flight-fights-20261120/local-cases.md`. The seven selected
starts were replayed with individual switches/combinations. The corrected
all-three specimen reaches 100k in six; 1976035382 still dies. The Rogue
control was already alive. These are case
checks, not a population improvement claim. Non-acting mode 2 preserves
control keys/states on all seven and evaluates the original 425904011
finishing opportunity as winning (1.044 turns versus about 1.064).
Population measurement is [borg-zoo #511](https://github.com/diegoami/borg-zoo/issues/511),
whose initial pilot ran on `78b8f297`. Owner 2026-11-20: local builds/smoke/named replays only;
no local population arms. The source tree does not own gameplay dossiers.


The first zoo pilot on `78b8f297` found no survival-leading arm and exposed
an attack selector defect. Expected damage now sorts before attack family;
family preference resolves ties, covered by an oil/pebble regression test.
The corrected all-switch specimen is `5acb8104`, including possible-wake
Recall. Its local checks reach the cap in six of seven named starts;
1976035382 still dies. Full evidence and pilot investigation belong to
[Cemetery #88](https://github.com/diegoami/borg-cemetery/pull/88). The updated
specimen remains unmeasured; [Zoo #511](https://github.com/diegoami/borg-zoo/issues/511)
preserves the initial pilot provenance and pins the future specimen.


Review corrections on2379b3fb (historical; superseded by the unit separation below):

- Race time now rounds to complete commands, including the unlimited melee
  fallback and each attacker's finishing command. A shot retains its fractional
  energy duration. Booking a full melee command is conservative when an early
  kill saves unused blow energy (player-attack.c:990-1020). The earlier original
  425904011 estimate of 1.044 turns was continuous-rate arithmetic and does not
  certify a win under the corrected command model; earlier case observations
  above remain historical until the new replay ledger is published.
- Owner approved the real-threat exception: a category target remains no scare,
  but does not pre-empt ordinary caution if its awake one-turn damage reaches
  current HP, or other foes' danger exceeds today's excessive-danger threshold
  (`caution_flee_danger_hp_multiplier`, reused unchanged). A lone cat at full HP
  remains a first-sight target. Winning-fight comparison still follows a category
  refusal, so a separately certified win may attack.
- Execution restores the borrowed grid monster index. Fixed-blow weapon spells
  credit all their engine blows, sharing the HP cap before hit chance. Leap is
  only certified against an adjacent target; uncertain movement remains outside
  the forecast. Observed Haste also reaches native status/spell danger on a copy.
  Own blessed-weapon to-hit is included; monster stun is conservatively not
  assumed to persist, just as Slow is not assumed to persist.
- Category lookup already gates runtime scare suppression on the active range.
  Attack and scare rows are independent kinds; an attack-only row never creates
  a scare outside its range. `mage 3` overrides that class's ceiling rather than
  excluding every other class. Additional regressions cover these distinctions.

The shared reviewed specimen is `394f2aea1f7c218359f182b0e55a3f6ae56ddeca`,
after accepted main #768. New named-case checks reach100k in five of the
original seven all-switch lives;425 and151 still die. Both reviewer-named
pebble/oil starts reach100k, with actual spell/oil choices recorded. Full
before/after controls, all failures and old versions are in Cemetery #88's
`review-cases.md`. The original425 control-neutral diagnostic now charges two
commands at3650 and does not certify that finishing state as winning. The
future Zoo specimen is explicitly repinned after these corrections;5acb8104
is a previous unmeasured proposal. No population improvement is claimed.


Third-review units correction (#763/#764): incoming forecasts now carry
literal contact HP separately from unmodelled risks. Basic elemental blows
use their dice, actual static hit power and speed ratio, without AC or
resistance discounts (known vulnerability uses the engine4/3 factor).
Native fear scores never become HP/turn. Unknown spell/status HP makes a
fight uncertified. Recall explicitly refuses unmodelled risks without
printing a fabricated damage total; heal/scare and the other explicitly
non-damaging, non-moving spell effects do not add HP or generic fear prices.
The other-monster exception still uses today's threshold multiplier on the
other monsters' native danger sum, omitting the target and regional/map fear;
only the multiplier is shared, not the complete quantity.

The new bow finding was checked against the engine: ranged_damage does not
multiply an arrow by shooting rate (player-attack.c:554-582); the command's
energy is reduced instead (:1087). The focused value is HP per Borg turn,
not HP per arrow. An18HP arrow at3 shots/turn therefore gives54HP/turn and
kills30HP in2 arrows/2/3turn; the actual formula already books that. Tests
now exercise the real rate conversion and finite-ammo/fallback cases. Changing
to ceil(HP/rate) without duration would over-credit each arrow. Regeneration
is gated by turn%100, not by each monster attack (mon-move.c:1893-1897), so
its conversion through player energy remains unchanged. Older forecast
figures and specimen394f2aea are historical; the new specimen must be
explicitly pinned after verification. No switch is enabled for shipping.


Verified shared unit-separation specimen:
`a4e4d91546cd9a2e1b43a03c0321feaa14b9893f`, with accepted main #779.
42 new named attempts (nine explicitly named starts, repeated across flags
plus one diagnostic) have no harness failures. All-switch reaches100k in
five original lives;425 dies to Fang3621 and151 to rock lizard6299. Both
reviewer-added starts survive; Magic Missile/oil choices are in the canonical
Cemetery #88 `units-cases.md` at4672f20c. Original425's non-acting diagnostic
remains control-neutral. The sound naga calculation still gives155.61HP;
unmodelled risk is a separate refusal reason. This supersedes394f2aea as the
future Zoo proposal; the original78b8 pilot retains its own provenance.
Both implementation CI runs and the shipped-off smoke checks pass. No
population improvement or shipment is claimed; only the owner schedules
Zoo #511 and accepts re-review.

## Flight Recall survival gate (#758)

The registry now holds **2258** parameters after accepted main #768
(2256 in the parent fight branch; two Recall parameters added).

`caution_recall_survival` compiles to **0** and ships **1** by owner
decision (2026-11-20). At 1, only a
flight's Recall read is refused when modelled contact HP during the delay
reaches current HP, or a reachable spell/status contribution remains unmodelled.
Sleepers that may wake are included; unknown risk is reported separately,
not converted from a fear score into HP. The switch
does not change trip-home Recall or the handling of a pending Recall.

`caution_recall_survival_delay` is compiled/shipped **34 world ticks**,
a conservative, unmeasured horizon. The engine rolls `15 + randint0(20)`
(`effect-handler-general.c:1145` at `574b618c`) and decrements Recall in
`process_world`, once per ten game turns (`game-world.c:709-714,1080-1082`).
The guard converts world time through player/monster energy, so a hasted
Borg does not incorrectly halve the delay's total monster actions.

`caution_recall_blow_model` (#880) compiles to **0** and ships **1**
(owner, 2026-11-22: a POISON crawl and breeders must not veto Recall). At 1
the forecast prices every blow by its dice, since no status stops the Recall
countdown (`game-world.c:709-714`). A POISON blow adds the poison timer, one
HP per world tick (`game-world.c:513-515`), from the earliest tick a poisoner
can strike to the end of the delay, but no more than the timer the poisoners
can add: 5 + rlev ticks per action (`mon-blows.c:683-684`; POISONED stacks,
`player-timed.c:1059-1070`). Already poisoned, the whole delay counts (the
remaining duration is not observed); resist poison blocks it (`player_timed.txt`
POISONED `fail:2:POIS`). Only SHATTER (an earthquake, `mon-blows.c:1086-1110`)
and ranged spells stay unmodelled. A breeder's (`RF_MULTIPLY`) unmodelled risk
is waived (future-proofing: no breeder at `574b618c` has a dangerous spell or
SHATTER). Contact HP is capped at the k fastest attackers for the whole delay,
k being the neighbours of the Borg that are not known rubble or wall (8 if a reachable
attacker passes walls; `mon-move.c:1620-1632`), and a RAND_25/RAND_50 monster's
rate is scaled by its attack share, 1 − s + s·(1 − (7/8)^5) for stagger chance
s (`mon-move.c:1087-1103,1571-1632`). A hypothesis
until borg-zoo measures it (RULES 14).

Reachability uses the Borg's remembered terrain and race movement flags.
Known permanent walls block every route; other known walls and closed doors
respect pass/kill-wall and open/bash-door abilities. Unknown terrain is
allowed conservatively. Monsters occupying a route may move during the
wait. An immobile monster at range contributes only if it has a current
projectable ranged attack. Physical approach consumes time before damage
is counted. Current ranged attacks have no approach discount. A mobile caster with a
possible route also gets no approach discount: it can gain line of fire
before reaching melee. Observed Haste is included; Slow is not credited as
persisting throughout the delay.

Only literal modelled contact HP is summed over the delay. Native danger
contains fear prices and is not HP. Basic elemental blows use raw dice,
static hit power and speed; AC/resistance discounts are omitted and known
vulnerability uses the engine factor. Harmless non-damaging/non-moving spell
effects such as HEAL/SCARE do not add HP or remove melee travel time.
Unmodelled spell/status contributions refuse separately with an explicit
risk reason, never a score multiplied by time and printed as damage. This can decline a Recall
that would have succeeded through early engagement, cures, attacks or
repositioning; it does not assume those future actions will save the Borg.
No unobserved engine map or future Recall random roll is read.

On this base `borg_escape`'s “Leaving (failed to teleport)” sets flight
flags. It does not itself read Recall. Its subsequent read is the common
caution site, now routed through `borg_recall_for_flight`; the two stated
paths therefore share one survival gate. #582/#583/#754/#641 were read;
no countdown, light-wait or exploration policy from those issues is redone.

Unit checks cover world time at different player speeds, travel consuming
the whole delay, blocked/open routes and immobile contact. The seven named
before/after cases are complete at 100k engine turns. Recall-only reaches
the cap for 425904011, 1976035382 and 807683971; the other three Ranger
controls still die with this switch alone. All three switches together
historically reached the cap in six of seven on superseded specimen `5acb8104`;
1976035382 dies to a large white snake at engine turn 17873.
The Rogue control was already alive. These are selected case checks, not a population improvement claim.
Canonical results/builds/deaths are in [Cemetery #88](https://github.com/diegoami/borg-cemetery/pull/88).
Measurement is [borg-zoo #511](https://github.com/diegoami/borg-zoo/issues/511),
whose first pilot used `78b8f297`; later specimens and the default-on
adoption remain unmeasured. The current pre-adoption result below is five
of seven on `a4e4d915`, not the historical six-of-seven result. No local population comparison was started.


Owner clarification, 2026-11-20: include possible wake-up during Recall.
Seed 807683971's naga was asleep at the read at 1390, then woke and killed
the Borg at 1534. The guard therefore forecasts a reachable sleeper as
awake, using a copy of its observed record rather than mutating knowledge.
Sleep is not guaranteed to last throughout 34 world ticks. This is a
conservative bound; it can decline a Recall that would have finished before
that creature actually woke. Pending Recall is checked before forecasting,
so a read already underway is not falsely logged as a new blocked read.


Review follow-up: ranged reachability explicitly distinguishes a walking route
from a current line of fire. A projectable immobile caster must still contribute
when it has no route to melee. A new regression covers this case; the review
suggestion to drop the projectability branch would remove that exposure.
Any spell flag remains the conservative ranged proxy, not measured HP damage.
Status/spell danger remains a conservative risk estimate rather than literal
HP loss. The corrected shared fight model requires a new explicit Zoo pin.


Third-review correction: qualitative unknown risk and quantified contact HP
are separate fields and refusal reasons. A distant acolyte's HEAL/SCARE does
not give its melee blow a zero-time approach; a projectable immobile caster
with an unmodelled offensive/status spell still refuses on risk, even without
a walking route. The actual contact-path exposure is computed separately.
This deliberately conservative fallback can refuse a safe unknown case; it
is not a claim that the monster's unmodelled spell is lethal. The new shared
specimen must be explicitly repinned after named checks. No population
comparison or shipped adoption is implied.


Latest verification uses shared specimen a4e4d915:42 additional named attempts,
no harness failures, with fresh controls, all three switches individually/all,
two reviewer-named starts and the original425 diagnostic. Recall-only still
reaches100k for425,197 and807; all-switch reaches five original lives, with
Fang425 and rock-lizard151 retained as deaths. Both added starts survive;
actual Magic Missile/oil choices are retained in Cemetery #88 `units-cases.md`
at4672f20c. The naga's155.61HP exposure is preserved, while unknown spell/status
risk is qualitatively labelled. Both code-pin CI runs and3/3 off smoke pass.
The future Zoo proposal is explicitly repinned to a4e4d915 and remains
unmeasured. Earlier394/5acb proposals and78b8 pilot keep separate provenance.


Owner adoption, 2026-11-20 (#760/#759): ship `attack_on_sight=1` and
`caution_winning_fight=1`; compiled defaults remain zero. This explicitly
supersedes the earlier shipped-zero descriptions for these two switches.
The owner authorized adoption before population confirmation; it is not a
claim of measured improvement. Numerical roundoff at whole-command boundaries
is removed using a machine-epsilon error bound; finite-ammunition fallback
still books complete commands and genuine excess residuals round upward.
The conservative Recall unknown-risk policy is retained for the paired
experiments, with HP-based and qualitative refusals reported separately.


Owner decision after fourth review, 2026-11-20: retain the explicitly
conservative unknown-risk refusal and ship all three switches at1 (compiled0).
This includes reachable poison/status attacks whose total consequences are
not modelled; it means modelling uncertainty, not proven lethality. Zoo must
report quantified-HP refusals and qualitative-risk refusals separately,
including outcomes/new deaths and actual Recall initiations. The owner
adopts this policy before population confirmation; no measured benefit is
claimed. Known awake/asleep counts in notes cover all known monsters; the
reachable count is a subset. The note now states that relationship explicitly.


Default-on adoption checks use f827f844 (source e0451e3e), after accepted
main#778/#777:14 fresh named attempts, explicit off1/7 alive versus all1
5/7, no named harness failures.425Fang3621 and151rock-lizard6299 remain
deaths; the old a4 result is historical and agrees on these endpoints.
Full canonical versioned evidence is [Cemetery adopted-cases.md](https://github.com/diegoami/borg-cemetery/blob/eecd3a4345fa4abaadb59ca9ffaf17dd14983d44/studies/reborn-flight-fights-20261120/adopted-cases.md).
Both default-on smoke references were re-recorded with the repository script.
Required local Rust/vendor/citation checks pass (527 core tests,2265
parameters, zero wrong/disagreeing rows). Prepared population measurement is
[Zoo#511](https://github.com/diegoami/borg-zoo/issues/511) (five-arm ablation)
and [Zoo#528](https://github.com/diegoami/borg-zoo/issues/528) (original600,
explicit off versus shipped, nested owner-gated tiers). No Zoo dispatch or
population benefit claim; this known600 set is not independent confirmation.

## Unseen occupants inferred from a failed step (#781)

This section records 2273 parameters after synchronizing main 2b67266d
(2269 before this task; four added). Earlier section counts are historical
snapshots and are not rewritten.

The behavior switches compile off. The shipped borg-rs profile enables
200 game turns of memory, the response, and a cure threshold of 50% HP,
with up to three oil throws per grid (#802). A flow or twitch move
followed by unseen player-melee feedback and an unchanged observed position
marks its intended grid occupied. Incoming attacks alone do not locate a
source. Blind, confused and hallucinating steps supply no grid. All flow
seeds, spreads and old-flow steps refuse the remembered grid. Memory expires
in game turns, resets on a new level and clears on successful entry or a
confirmed directional oil kill. No race, HP, AC or monster index is invented.

| Parameter | Default | Range | Unit | Source | Meaning |
| --- | --- | --- | --- | --- | --- |
| `caution_unseen_bump_turns` | 0 | 0–100000 | game turns | `borg_caution.rs:8473` | #781: a queued step with unseen melee feedback and no movement marks its destination occupied for this many game turns. All flows refuse it. 0 = off. |
| `caution_unseen_bump_response` | 0 | 0–1 | flag | `borg_caution.rs:8598` | #781: after ordinary healing, defence, Recall and food, before stair walking, try one carried Detect Invisible scroll under the existing detection danger/darkness gates, bounded directional oil throws with normal fuel reserves, then Phase Door under the emergency landing test. Requires caution_unseen_bump_turns. Counts successful phases toward level escape accounting. 0 = off. |
| `caution_unseen_bump_heal_hp_percent` | 0 | 0–100 | percent | `borg_caution.rs:8590` | #781: below this HP percentage with a recent adjacent unseen bump, drink CLW, CSW, then CCW after ordinary healing/defence/Recall but before stair walking, even while fleeing. Intentionally ignores danger, including survivable-healing and no-rope-a-doping gates: regional fear does not quantify the unseen blow. Requires caution_unseen_bump_turns; independent of response. 0 = off. |
| `caution_unseen_bump_throw_tries` | 3 | 0–100 | throws per grid | `borg_caution.rs:8589` | #781: maximum queued oil attempts at one remembered bump grid before trying Phase Door. Cancels also spend an attempt; only one command is queued per response. Expiry or a confirmed kill clears that grid and its budget. 0 = phase after detection; hypothesis, not calibrated. |

Ordinary healing, defence, swaps, Recall handling and emergency food keep
priority. The bump cure and response follow those checks and precede stair
walking. Detection shares the existing See Invisible danger/darkness gates;
`borg_read_scroll` also refuses no-light, blindness, confusion and amnesia.
`detection_read` records a queued read, not an unsuccessful inventory query.
The new cure deliberately ignores both ordinary danger gates: it is an
independent experiment for regional fear without a located monster. It can
spend a potion that does not outheal actual damage; this is a stated risk,
not an assertion of safety.

Oil uses the normal fuel reserve and a direction, not a monster target. Each
queued attempt reserves one of the grid's throws, even if input is cancelled.
A pending shot survives intermediate updates until confirmed dead, resolved
by the oil-break message, cancelled, superseded or its grid expires/is entered.
A resolved miss cannot claim a later unrelated death. A kill deletes the grid
and its budget; a later new bump starts a new budget.

The Phase Door site bypasses the known-race fight/adjacency tests because
bump evidence supplies adjacency, but now keeps the ordinary emergency
landing tolerance/horizon. A successful phase increments `borg.escapes` and
cancels the old flow, as the ordinary escape call does. `caution_escape_limit`
is a threshold that starts leaving, not a hard cap prohibiting further escapes.
These settings are hypotheses for Zoo measurement. The owner chose to ship
the profile used in the #781 review trials.

Historical controls reproduce `8b21c6b94e80e969` and death at37,759. Current
main already survives after #768, so historical reconstruction is distinct
from current-main checks. Canonical prior attempts and reproduction patches
are pinned in the [Cemetery case](https://github.com/diegoami/borg-cemetery/tree/b20537c68830e415c79a8766135a1b32c7e539c2/validation/borg-reborn-rs/unseen-bump-781).
Review follow-up receipts are published with Cemetery PR #92. Those trials
did not measure a population effect.

## A teleport curse stops every teleport (#829)

One flag. Compiled 0 (the Borg plays as before); shipped 1 in `harness/borg.txt`.
A zoo comparison sets `borg_param_escape_teleport_block = 0` for its control
(see docs/PHASE-DOOR.md for where the check sits).

| Parameter | Default | Range | Unit | Source | Meaning |
| --- | --- | --- | --- | --- | --- |
| `escape_teleport_block` | 0 | 0–1 | flag | `borg_escape.rs:1714` | #829: 1 = a teleport curse stops every teleport of the player. The Borg learns it from "Your X prevents you teleporting." and "Teleportation forbidden!" (the engine's own words, whatever the item's identification state) and keeps it while the watched equipment is unchanged or until "The anti-teleportation curse is removed!"; Phase Door (every site, inside phase_door), Teleportation and Teleport Level scrolls, the staff of Teleportation, the teleport activations and the teleporting spells are then refused, so the site goes on with its next escape. A no-teleport grid is not asked for Phase Door (the engine does not stop it). 0 = as before (BI_CRSNOTEL only, at the sites that ask borg_allow_teleport). |

## Fast-fragile monsters (#842)

Owner, 2026-11-21: attack a fast, fragile monster "with all that we have got,
unless we want a phase door or a stair", and never drink a cure against it.
Nine parameters, all compiled **0** / their defaults. **Shipped** (owner,
2026-11-21, after the review of #847 and before the zoo result of borg-zoo#596):
`fast_fragile_policy = 1` and `fast_fragile_no_heal = 1`, on the named entries.
`fast_fragile_by_stats`, `fast_fragile_phase_landing` and
`fast_fragile_escape_percent` ship at their defaults (0, 0, 100). The numbers
(2 turns, 25%, 2 others, +5 speed, 100%) are hypotheses: no measurement chose
them.

| parameter | default | unit | range | source |
|---|---:|---|---|---|
| `fast_fragile_policy` | 0 | flag | 0–1 | `borg_fast_fragile.rs:47` |
| `fast_fragile_by_stats` | 0 | flag | 0–1 | `borg_fast_fragile.rs:340` |
| `fast_fragile_no_heal` | 0 | flag | 0–2 | `borg_fast_fragile.rs:536` |
| `fast_fragile_escape_percent` | 100 | percent of HP | 0–1000 | `borg_fast_fragile.rs:538` |
| `fast_fragile_phase_landing` | 0 | flag | 0–1 | `borg_fast_fragile.rs:573` |
| `fast_fragile_speed_margin` | 5 | speed points | 1–40 | `borg_fast_fragile.rs:353` |
| `fast_fragile_kill_turns` | 2 | turns | 1–10 | `borg_fast_fragile.rs:348` |
| `fast_fragile_damage_percent` | 25 | percent of HP | 1–1000 | `borg_fast_fragile.rs:349` |
| `fast_fragile_group_max` | 2 | monsters | 0–100 | `borg_fast_fragile.rs:349` |
| `fast_fragile_heal_floor` | 0 | percent of max HP | 0–100 | `borg_fast_fragile.rs:484` |
| `fast_fragile_escape_quantile` | 0 | percent | 0–100 | `borg_fast_fragile.rs:464` |
| `fast_fragile_count_dot` | 0 | flag | 0–1 | `borg_fast_fragile.rs:430` |

#926 added the last three, each 0 (today's play), none shipped:

- `fast_fragile_heal_floor`: at or below this share of the maximum HP the
  heal ban lifts, and attack on sight steps aside for a carried, *known* cure
  (after Phase Door and the stairs underfoot).
- `fast_fragile_escape_quantile`: "the next turn kills" weighs that quantile
  of the damage distribution before the Borg's next action (each blow's hit
  chance and dice; floor or ceil of the energy ratio for the number of
  actions; 100 = the maximum) instead of the mean, still against
  `fast_fragile_escape_percent` of HP. The wild cat against the Dwarf
  Necromancer 1398933929 at HP 8: mean 7.6, median 8, maximum 12; P(death)
  31.8%.
- `fast_fragile_count_dot`: adds the Borg's own poison and cut ticks
  (game-world.c:513-538) expected before its next action (10 / its energy
  world ticks). When they alone take it below 0 HP, it drinks the carried
  potion that stops them (a cut up to 20: CLW; deeper: CSW or CCW; poison:
  Neutralize Poison or CCW; both: CCW) before the attack, Phase Door and
  the stairs, and the heal ban lifts.
- #1028: "they alone take it below 0 HP" counts the world ticks of the
  coming window from the game turn and the Borg's energy (the player acts
  before the turn's tick, game-world.c:1016-1095), so it is exact, not an
  average: a hasted Borg's windows hold one tick and none in turn.

- `fast_fragile_policy=1`: a monster marked fast-fragile is attacked by the
  focused planner's best attack every turn, without attack-on-sight's
  `target damage < HP` gate (the other foes' danger test stays). When the
  expected damage of the fast-fragile monsters within two grids reaches the
  Borg's HP and the kill needs more than one more turn of its best attack,
  the turn goes to the staircase underfoot, else Phase Door (scroll, or the
  spell at `escape_last_ditch_phase_fail`); with neither, it attacks.
- `fast_fragile_by_stats=1`: besides the scary list's `fast-fragile` entries
  (crow, wild cat), a monster is marked when it is faster than the Borg by
  `fast_fragile_speed_margin` points (`forecast_monster_speed` against
  `player_combat_speed`; the engine spreads a race's speed by about a point,
  so +1 is no advantage: a small kobold was marked without a margin), its HP is at most
  `fast_fragile_kill_turns` turns of the best attack (hit chance, failure and
  the HP cap included), its expected damage per Borg turn is at least
  `fast_fragile_damage_percent` of the current HP, it moves, is not unique, is
  not a breeder (`MULTIPLY`) and has at most `fast_fragile_group_max` others of
  its race in view. A scary-list attack entry of any reason names its monster
  and overrides the stats, so a name can add or remove a monster. Each mark is
  noted with its numbers, e.g. (from a run) `# Fast-fragile (named): crow
  (speed +9 vs +0, 11 HP vs 8.9 per turn, 7.2 damage/turn = 66% of HP 11, 0
  others in view) (#842)`; monsters spread about one speed point and roll
  their HP, so the numbers are not the race's.
- **Dropped veto.** The ungated attack drops both of `category_attack_safe`'s
  tests of the target: `target damage < HP` (the point) and the veto on blows
  the core cannot model (`unknown_damage`: only HURT, ACID, ELEC, FIRE, COLD
  and NONE are modelled). A marked monster with a poison or other unmodelled
  blow (a wood spider, by stats) is attacked on a lower bound of its damage.
  The crow and wild cat are pure HURT. Read the `stats` arm with this in mind.
- The Phase Door goes through the one decision point (`phase_door`, site
  `fast-fragile`, docs/PHASE-DOOR.md): the owner's adjacent-enemy rule (#584)
  and the anti-summon delay apply. The landing test is behind
  `fast_fragile_phase_landing` (0 = skipped): at HP 3–6 against a crow every
  landing grid counts as scary (100 of 100) and the test refuses the Phase
  Door in the fights it is for. On the 8 crow cases: with the test 4 dead of
  8, without 1.
- `fast_fragile_no_heal=1`: `borg_heal` (past its confusion, blindness and
  Morgoth cases) and the outmatched heal return without a quaff while a marked
  monster is awake next to the Borg. `=2` makes the issue's one exception:
  when the next turn kills, the kill needs more than one more turn and
  neither Phase Door nor a staircase is at hand, attack-on-sight steps aside
  and a cure is allowed (a Borg with no cure attacks, as with 1).
- `fast_fragile_escape_percent`: "the next turn kills" is the expected damage
  of the marked monsters within two grids reaching this percent of the HP.
  100 is the mean; the crow's four bites spread more than that. 0 turns the
  Phase Door / stairs alternative off, so the policy only attacks.

The switches act through `attack_on_sight` (shipped 1). The zoo arms are
listed in the handover note.

## Carried fuel and spare torches (#810)

The registry holds **2302** parameters. The table gives the core fallback.
The shipped `harness/borg.txt` sets `town_light_reserve_turns = 20000`.

| parameter | default | unit | range | source |
|---|---:|---|---|---|
| `town_light_reserve_turns` | 0 | turns | 0–60000 | `borg_town_capability.rs:950` |
| `town_lantern_upgrade_power` | 0 | power | 0–100000 | `borg_town_shadow.rs:180` |

The reserve target is the larger of the configured amount and planned
journey length. The Borg counts the equipped light and usable pack fuel.
Each torch in a stack contributes its own timeout. Oil counts toward
light when using a fuel-burning lantern. Home stock counts once carried.
The reserve skips fuel selection for characters who can see without it.

While a torch is lit, the Borg replenishes from Home or the General
Store after immediate needs and within ordinary gold, weight, and pack
limits. Additional usable light has diminishing shopping value above
the reserve and competes with other supplies. `town_light_percent`
scales the shopping value. It does not change the fuel target. Zero
restores the original shopping curve and discard policy, including its
legacy flask valuation.

The lantern bonus adds power points to the normal try-on score before
the diminishing equipment curve. It defaults to zero in the shipped
profile.

## Enchant/decurse policy (#680 / P6)

The contract and receipts below describe the original #680 extraction.
#774 preserves this implementation while reconciling accepted main `4e9af99e`
and its engine `574b618c`. The original 45-case results are historical to
`760fae80`, not proof of the synchronized candidate. The synchronized merge
is checked locally with Rust/vendor/citations, smoke and one named override
case; the broader default comparison goes to Zoo under current RULES20.

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


### Synchronized receiving baseline (#774)

Accepted main `4e9af99e` and reviewed track-p `e9670cab` merge at
`27700983` (engine `574b618c`, donor still `8c5d000a`). The subject modules
are byte-exact reviewed #680 code, and subtracting those ten registry
entries/test/prefix map recovers main's exact parser/registry. All other
runtime inputs, settings, references and commit pins preserve main.
[Local immutable receipt](https://github.com/diegoami/borg-cemetery/tree/be61aeff63f19ad275de265a1550aba5d7d392b5/validation/borg-reborn-rs/p6-main-reconciliation-774)
(Cemetery #93) covers native smoke 3/3 and one named override case. Both
sides' complete traces are retained; the new witness is at turn80689, not
the historical turn23238. No natural death occurred in these five checks.
The stdout logs remain as captured; quiet success is explained in the README.

Broader new-engine/build parity remains pending [Zoo #520](https://github.com/diegoami/borg-zoo/issues/520)
under RULES20, pilot first and further starts only after owner-gated results.
Neither local smoke nor original 760fae80 full45 is proof of that comparison.
This note does not accept the reconciliation PR or the P6 milestone.


### Final receiving candidate (#814)

Accepted main07aa4b1e plus reviewed track-p221a2504 merge at a3635ad8.
Main's resolved-settings receipt and its test, parser, settings and native
pin46978936 are preserved, alongside the reviewed ten controls. Local smoke
matches3/3; the current native witness is dungeon-limit0 at turn19991,
not the old town-limit witness. The town-limit trials at100k and300k are
retained negative controls. This is local named-case coverage, not a cohort.
Core568/4 ignored, engine93/4 skipped,2293 citations, scripts39/tools96 pass.
The new broader comparison is Zoo571, pending; old Zoo520 full45 cannot be
relabelled as new07aa4b1e proof. This status does not accept P6 or main promotion.


## Departure room, the Recall curve and surplus-first storing (#859)

The registry now holds **2357** parameters (2352 on track-p `29c44cbc` before it; five added).
Owner, 2026-11-21: a Borg with 20,000 gold did not buy a Word of Recall, so
the dive gate (`prepared_recall_depth`, DL5) held it shallow: "it seems that
purchases are stopped because of carry weight". With #711's
`town_store_only_overweight` the planner stored stacks only while the pack was
over its allowance (`BI_CARRY / 2`) and stopped as soon as it was under, so
a pack at the allowance had no room for a 0.5 lb scroll (`object.txt`,
Word of Recall, weight 5).

- **`town_departure_free_weight`** (compiled 0, **shipped 50**: 5 lb) and
  **`town_departure_free_slots`** (compiled 0, **shipped 2**). The planner
  (`dispositions`, `borg_town_utility.rs`) stores at the Home until this
  room is free under the allowance, plus the room the Recall reserve still
  needs (`recall_room`, `borg_town_capability.rs`: the copies short of
  `town_recall_reserve`, the lightest offered copy, and a slot when none is
  carried). The purchases (`step`, `borg_town_record.rs`) and the Home
  withdrawals leave the departure room free, so the scroll fits on top of
  it. Short of slots, whole stacks go, lowest keep value per weight first.
  Each departure notes "# Town departure room: weight W free (keep 50),
  slots S free (keep 2)" once a visit.
- **`town_store_keep_min`** (compiled 0, **shipped 3**; owner: "prefer
  objects that they have an abundance of, typically healing potions and
  phase doors, ideally we want to have variety in the backpack"). Short of
  weight, stacks with copies past the minimum come first, priced by one copy
  on their curve, and only those copies count as freed. The live executor
  moves one copy per store action and solves again (`borg_town_shadow.rs`,
  `evaluate`: every live action but a purchase moves one copy, #507), so a
  stack goes down copy by copy while it has a surplus. The minimum is a
  floor for a stack that serves a need (potions, scrolls, torches, food,
  through the kit's `need` or `serves`) and for flasks of oil (by tval:
  oil serves no `need`; owner: "check that flasks are taken"): at the
  minimum it is kept whole, not stored, sold or dropped, so the live walk
  stops there and
  the stack frees no slot (the re-review of #864 found it went on down to
  none). An item that serves no need (a lone armour piece) has no floor,
  and short of slots whole stacks go. A pack whose only storable weight is
  floored stacks leaves with less room than the target. The
  plan's own action (`first_action`) moves exactly the surplus
  (review F1 of #864): twelve copies with three kept give a
  store of nine, and the whole stack when `town_store_keep_min` is 0 or the
  flavour is unknown. The live executor then cuts it to one copy
  (`executed_copies`).
- **`town_recall_curve_scale_percent`** (compiled 0, **shipped 110**;
  owner: "a curve for word of recall starting from 1 and make it be 110 by
  default, so that they buy a few"; "use the trip weight"). Above 0, every
  scroll is valued on a diminishing curve on the trip weight
  (`borg_town_w_trip`), weight and demand scaled by the percent as the
  healing and Phase Door scales do: each scroll is worth less than the last
  and never nothing, so a Borg with gold and room carries spares. The
  `town_recall_reserve` requirement stays on top of it, in a row of its own
  (`RECALL_RESERVE`, as `PHASE_RESERVE` is on top of `PHASE`;
  `recall_curve`). With the curve on, `town_recall_value_percent` (the
  walk a scroll saves) is not read: the trip weight replaces it. 0 = #557:
  the reserve alone, nothing past it, and below the gold floor the walk a
  scroll saves.
- **`town_recall_urgent`** (compiled 0, **shipped 1**; owner: "Word of
  recall should have a high value"). When the next trip reaches
  `prepared_recall_depth` and no scroll is carried (`recall_urgent`,
  `borg_town_utility.rs`), the first Word of Recall is an urgent need: the
  urgent pass (`Budget::of`) buys it before the solver weighs anything, as
  missing light or food, and `buy_recall_from_gold` does not hold that
  first scroll back (`execute_store`). The floor still caps the spares
  (the stock's quantity cap) and the `town_recall_reserve` requirement
  (`recall_reserve_for_budget`) in the same visit, as before. With the curve on, the Recall trip weight is also
  multiplied by the trip's exposure (never below the flat weight), as
  healing's and the escapes' are, so a scroll is worth more deeper.
- The shipped values are hypotheses. The zoo request on the PR measures
  them (RULES 14, 20).

| parameter | default | unit | range | source |
|---|---|---|---|---|
| `town_departure_free_weight` | 0 | tenths of a pound | 0–10000 | `borg_town_utility.rs:284` |
| `town_departure_free_slots` | 0 | slots | 0–23 | `borg_town_utility.rs:285` |
| `town_store_keep_min` | 0 | copies | 0–99 | `borg_town_utility.rs:313` |
| `town_recall_urgent` | 0 | flag | 0–1 | `borg_town_utility.rs:256` |
| `town_recall_curve_scale_percent` | 0 | percent | 0–10000 | `borg_town_capability.rs:812` |

## Word of Recall on shallow trips (#984)

Blackguard 312081895 (play `4aa99f20ea1ca7e6`) read its last Word of Recall
on DL1 as a flight. It then took the up staircase to town with the Recall
pending, and the Recall fired there 42 game turns later
(`game-world.c:707-731`; the delay is 15 to 34 world ticks,
`effect-handler-general.c:1145`). It was pulled to DL2 with no scroll. In
the plan records of that visit (`BORG_TOWN_PLAN_RECORD`), the urgent pass
held only a torch, and the floor left no Recall copy to buy: gold 241, less
the 6 for the torch, less `buy_recall_from_gold` 200, is below the scroll's
125 (`borg_town_shadow.rs`, the stock's quantity cap). With the floor at 0
the solve still spent the 235 gold on Phase Door, oil and potions. Both
switches below are off by default, which reproduces today's play.

- **`town_recall_urgent_depth`** (compiled 0, shipped 0). With
  `town_recall_urgent` on, this sets the trip depth from which the first
  Word of Recall is urgent when none is carried (`recall_urgent`,
  `urgent_recall_gate`). 0 keeps `prepared_recall_depth` (#859); 1 makes
  any trip into the dungeon qualify. The dive gate does not move.
- **`caution_recall_dl1_hold_up`** (compiled 0, shipped 0). While a Recall
  counts down on DL1, caution's "Take stairs up" is held unless the danger
  is above the avoidance (`recall_holds_up_stair`), the boundary of the
  stairs rule next to it (`pos_danger > avoidance`). The Recall then fires
  on DL1, and the Borg reaches town with nothing pending. The danger
  escapes (`borg_escape_stair`) are not held. 0 = C
  (`borg-caution.c:1171`). This is the up half of the crossing between
  town and DL1. The down half is already held by `recall_restricts` (#603,
  #627), and `recall_pending_escape`'s note names the gap. With both
  halves held, a Borg on DL1 waits out the countdown (15 to 34 ticks) or
  fights. An arena level has no staircase (`arena_gen`), so the hold never
  applies there.
- On this seed, either switch or both reach the 100,000-turn cap alive
  (one seed, no claim; RULES 14). The zoo measures them.

| parameter | default | unit | range | source |
|---|---|---|---|---|
| `town_recall_urgent_depth` | 0 | depth | 0–127 | `borg_town_utility.rs:255` |
| `caution_recall_dl1_hold_up` | 0 | flag | 0–1 | `borg_caution.rs:8928` |

## The Borg's own light radius (#877)

The C Borg marks as lit by the Borg's own light every grid up to `BI_LIGHT`
(`cur_light`) away (`borg_update_light`, borg-core/src/borg_cave_light.rs;
C src/borg/borg-cave-light.c:111–112, "Radius 2 -- lantern radius"). The
engine lights `radius = ABS(cur_light) - 1` (src/cave-view.c:658,
`add_light` :616–646): a Wooden Torch (`LIGHT_2`, lib/gamedata/object.txt:241)
lights distance 1 only. With C's radius a monster two grids away in the dark is
"expected visible" (`borg_follow_kill_aux`), not seen, and deleted
(`borg_follow_kill`, "C bug 24"); the Borg then walks back next to it (Human
Mage 461220874, borg-zoo run 44936494812: 44 of 50 worm forgets).

- **`cave_light_engine_radius`** (compiled 0, shipped 0). 1 = the engine's
  radius, `cur_light - 1` (never below 0), and the radius-2 and radius-3+
  blocks of `borg_update_light` follow it. A necromancer at `cur_light <= 1`
  sees by unlight (src/cave-view.c:777–779) and keeps C's radius. With 1, a
  non-necromancer whose light slot is empty still lights the map from other
  equipped gear (`BI_LIGHT > 0`, src/player-calcs.c:1624–1644; #888, kept by
  the owner on 2026-11-22). 0 = C's
  radius, today's play. The zoo measures it (RULES 14, 20) before it ships.
- With 1 the grids lit also take the engine's shape, `distance() <= radius`
  (#884): radius 2 lights the grids at distance 2, (2,0) and (2,1), each in
  line of sight for light on the Borg's map (`borg_los_light`, as `add_light`
  tests `los()` per grid, src/cave-view.c:625-640, through projectable grids
  only: not through passable rubble, a closed door or unknown terrain), where
  C lights (2,2) north and south (distance 3) and
  misses (2,1); C's floor-gated (3,3) corners (distance 4) go, and the
  radius-3+ box skips grids by distance <= 2, so (2,2) is lit at radius 3.
  The code that looks at the edge of the light reads the same radius
  (`own_light_radius`): `borg_flow_dark_2` (just outside the light,
  `borg_flow_dark.rs`), the dark frontier (#796; each ray's lit steps by
  engine distance, so a lantern's diagonal ray has one lit step) and Light Beam
  (`borg_light.rs`), and Call Light's radius-1 / radius-2+ branches
  (`borg_check_light_only`), so a torch takes the radius-1 corner test and
  a lantern the 5x5 lit-floor count. `BI_LIGHT` 1 (a Ring of Light,
  `object.txt:2132`, without a light source) is radius 0 and keeps C's corner
  test; only no light at all calls light without a test, as in C.
- Evidence, one run (review of #894, not a measurement): CI's death case,
  Mage seed 1016 to 100,000 turns (`.github/workflows/ci.yml`, the
  `core-trace-death` run), dies at turn 23,211 to an apprentice at 0 and
  survives to the cap (CL4, DL2) at 1. That run uses
  `BORG_CORE_OVERRIDES=off`, so it reads `borg-core/borg.txt` and the compiled
  default: shipping 1 in `harness/borg.txt` does not touch it, but changing the
  compiled default to 1 makes its `assert cases` fail. That job then needs
  another seed, or a check that tolerates a survivor.

| parameter | default | unit | range | source |
|---|---|---|---|---|
| `cave_light_engine_radius` | 0 | flag | 0–1 | `borg_cave_light.rs:130` |

## A miss is not a wall (#929)

After a ranged attack with no hit message the C Borg takes the first unknown
grid of the path for a wall (`borg_target_unknown_wall`,
borg-core/src/borg_projection.rs; C src/borg/borg-projection.c:739-830,
called from `borg_update`, borg-core/src/borg_update.rs:1735). Its walk starts
at the Borg's own grid (C :753-754), and with no unknown grid it writes the
step back from the target (C :821-826): for an adjacent target that step is
the Borg's grid. A thrown or fired miss prints nothing (`make_ranged_throw`,
src/player-attack.c:1278-1280, `test_hit` failed), and the missile still
reaches the monster's grid: `ranged_helper` stops it only before a grid that
is neither passable nor projectable (src/player-attack.c:1101-1108). So every
miss at an adjacent monster wrote granite under the Borg (Paladin 1194051723:
5 of 5 guesses; Druid 1024822556: 2 of 2; Necromancer 798380952).

- **`update_miss_wall_guess`** (compiled 0, **shipped 1**: owner, 2026-11-22, #1069, before a zoo measurement). 1 = (a) a miss at a
  monster on the targeted grid that is in view (`BORG_VIEW`) and was seen at
  the think that shot (`when` at most one think ago) is a miss: no grid is
  guessed and `successful_target` becomes 2, the value C's guess gives, so the
  next shot is allowed as today; (b) a guess never writes the Borg's grid, nor
  the target's own grid as an unknown one: the walk starts at its first step,
  skips the unknown test on the target (in the dark the first step of an
  adjacent target is the monster's grid, and granite there blocks the next
  shot, `borg_projectable`), and an adjacent target with no unknown grid
  between guesses nothing (`NextToBorg`). Every outcome sets
  `successful_target` to 2, the next shot allowed, as C after every guess;
  leaving -11 after a spell would lock ranged attacks out until the Borg
  moves (`missed_before` clears only -12 or less; review of #1005). 0 = C's
  guess.
- Evidence (replays, not a measurement): both Fang games and the three smoke
  games play the same key stream with 0 and 1 (Paladin `7fb11d46a041a04a`,
  Druid `3687ca84c5cdeb34`). The Borg's grid was remembered in all seven
  guesses, so the map pass of the same `borg_update` put the floor (or the
  open door) back before any decision read it. The granite lasts only where the
  player does not remember its own grid (light 0, the Necromancer): then no
  flow reaches the Borg (`spread_blocked`, borg_flow.rs:428) and every
  projection to the Borg reads as blocked (`borg_projectable`,
  borg_projection.rs:349), which hides ranged danger. Not replayed.

| parameter | default | unit | range | source |
|---|---|---|---|---|
| `update_miss_wall_guess` | 0 | flag | 0–1 | `borg_update.rs:1740` |

## Shooting at a monster believed in the dark (#879)

Owner, 2026-11-22: "we can teach the borg to shoot through a corridor, if they
think a monster is there". The C Borg aims only at monsters seen within
`attack_max_target_age` Borg turns (`borg_attack`, `borg_launch_bolt_aux_hack`).
`borg-core/src/borg_unseen_belief.rs` (ours) keeps a recently seen monster as a
ranged target at its last grid. The engine stops a bolt at the first monster on
its path (`PROJECT_STOP`, src/effect-handler-attack.c:130), so a monster that
closed in along the line is hit too. The Borg aims at the location (`borg_target`
in free mode, src/ui-target.c:1479–1494, src/target.c:180–191). Four
parameters, compiled at **0** / their defaults. `harness/borg.txt` ships
`unseen_belief = 1` with `unseen_decay_percent = 5` (owner, 2026-11-23,
together with `cave_light_engine_radius = 1`, #1078). That is the arm
`light+belief-decay5` of borg-zoo#702's belief full tier at track-p `f8725bc0`
(900 starts, 100,000 turns): 13 deaths against 23 for ctl (19 rescued, 9 newly
killed, p 0.087), 17 for `light`, and 15 for `light+belief` at decay 20.
Against `light` the step is 13 vs 17 (p 0.45), and DL3 is reached less often
in every light arm (365 → 326). The numbers remain hypotheses (RULES 14).
The decay grid 10 / 5 / 0 on an independent block (starts 1–900) is requested
on borg-zoo#702.

- **`unseen_belief`** (0): 1 = a monster seen within `unseen_max_age` Borg
  turns but not now, at least two grids away, whose last grid the Borg would not
  expect to see it on (`borg_follow_kill_aux`: dark, beyond its light), stays a
  target of the ranged attacks (bolts, balls, missiles, throws), each attack's
  value multiplied by the belief. Melee never takes it (`thrust_damage_with`).
  Each such shot is noted: `# Unseen target (#879): <race> believed at (y,x),
  seen N turns ago, belief W%`. It pairs with `cave_light_engine_radius = 1`
  (#877): with C's radius, the dark grid two away is expected visible and the
  record is deleted, so this switch alone changes nothing in the #877 case.
- **Belief**: 100% for `NEVER_MOVE` or a sleeper; otherwise `100 -
  unseen_decay_percent` per monster turn since it was seen (Borg turns × its
  energy / the Borg's, `extract_energy`), never below 0. After one unseen shot
  a sleeper counts as awake.
- **The effective window is the decay** (review of #890, F2). The age and the
  decay are one knob: at the shipped 20% and a Borg at speed 110 (energy 10,
  src/game-world.c:68–90), the belief at ages 3/4/5/10 is 40/20/0/0% for a
  speed-110 monster, 80/60/60/0% at speed 100, 100/80/80/40% at speed 90, and
  0 for anything faster than the Borg. `unseen_max_age` (10) is the window only
  for sleepers, `NEVER_MOVE` and slow monsters. A zoo arm with a lower decay
  measures the knob apart (owner, 2026-11-22; borg-zoo#702). The energy ratio
  over-counts the engine's actions a little at high speed
  (src/game-world.c:136–139); the integer division makes slow decay coarse.
- **What is aimed at** (review of #890, F3): `borg_attack` keeps only a record
  whose grid is on the panel (`BORG_OKAY`) and in the Borg's view shape
  (`BORG_VIEW`, lit or not), so an unlit corridor in line works and a monster
  around a corner does not. Infravision (`BI_INFRA`) raises the two-grid floor
  for warm monsters, since `borg_follow_kill_aux` expects them visible within
  it. A believed record never counts as the monster next to the Borg
  (`borg_attack`'s adjacent test, F1), so it cannot turn a scary level's
  flight into a fight.
- **`unseen_max_shots`** (3): at most this many unseen shots per sighting.
  After a launch, count each believed record in the projection, including
  arc and splash targets whose utility was negative. Update the sleeper's
  inferred wake state at the same time. Failed commands leave both alone.
  The cap stops the Borg from crediting a record after repeated shots.
- **Curse** requires a real monster target. Believed records are excluded
  from Curse scoring because they provide only a last-known location.

| parameter | default | unit | range | source |
|---|---|---|---|---|
| `unseen_belief` | 0 | flag | 0–1 | `borg_unseen_belief.rs:448` |
| `unseen_max_age` | 10 | Borg turns | 3–2000 | `borg_unseen_belief.rs:456` |
| `unseen_decay_percent` | 20 | percent per monster turn | 0–100 | `borg_unseen_belief.rs:471` |
| `unseen_max_shots` | 3 | shots | 1–100 | `borg_unseen_belief.rs:221` |

## Perception and belief follow-ups (#885, #917, #889)

Five switches, compiled **0**. `update_dark_clears_light` and
`view_rubble_blocks_sight` are **shipped 1** in `harness/borg.txt` (owner,
2026-11-23: "merge with both"), on borg-zoo#736's full tier at `7bfbe121`
(classes-1800 starts 901-1800, 900 per arm, paired against ctl):

| arm | deaths | rescued / newly killed | p | DL3 / DL5 reached | CL |
|---|---:|---|---:|---|---:|
| ctl | 9 | | | 221 / 19 | 5.48 |
| dark | 9 | 2 / 2 | 1.0 | 222 / 21 | 5.51 |
| rubble | 5 | 8 / 4 | 0.39 | 223 / 26 | 5.49 |
| both (shipped) | 5 | 8 / 4 | 0.39 | 216 / 33 | 5.48 |

A lead, not a result (RULES 14): 8 against 4 is not significant, and `dark`
alone changes nothing in deaths. No single forget or mis-aim has been traced
to a recorded game yet. The three #889 switches ship **0** (P9).
Two are perception fixes, where the Borg's map disagrees with the engine, and
both feed the position belief of #879: a grid `borg_follow_kill_aux` expects
to be visible both deletes an unseen record and refuses it a belief
(`believed_weight`).

- **`update_dark_clears_light`** (#885). A floor grid the engine shows neither
  glowing nor lit loses `BORG_LIGHT` when the map is read (`dark_floor_info`,
  `borg_update.rs`). C keeps the mark, and `borg_update_light` clears only the
  grids of the Borg's own light list, so a grid lit once another way (a
  monster's light) stays `BORG_LIGHT` after it goes dark. A cleared mark asks
  for `borg_update_light` in the same think, so the Borg's own light set is
  marked again (at `cave_light_engine_radius` 0 that list holds grids the
  engine leaves dark).
- **`view_rubble_blocks_sight`** (#917). Passable rubble stops the Borg's view
  (`borg_update_view`, through `sight_grid`) and the line of sight the view
  falls back on (`borg_los_sight`). The engine's passable rubble has neither
  `LOS` nor `PROJECT` (`lib/gamedata/terrain.txt:346-354`), so
  `update_view_one` (src/cave-view.c:765-830, `square_allowslos`) and `los()`
  (:85-248, `square_isprojectable`) stop at it. C's view asks the walk test
  (`borg_cave_floor_grid`, `borg_cave_floor_bold`), which counts it as floor,
  so grids behind it got `BORG_VIEW`. The rubble itself is still seen, as a
  wall is, and unknown grids stay see-through, as in C. A grid in view whose
  sight changes when the map is read (an unknown grid found to be passable
  rubble, which the walk test misses) redoes the view (`view_needs_update`;
  review of #1128, F2). The light map already
  had its own projectable test (#894, `borg_los_projectable`); attack
  forecasts through rubble are #1017 (P9).
- **`unseen_death_stops_shots`** (#889, item 2). "It dies." or "It is destroyed."
  among the messages of a ranged attack whose only believed target was one
  record spends that record's remaining unseen shots for this sighting
  (`believed_dead`, `spend_shots`). The record is kept (review of #1128, F1):
  the engine names a monster the player cannot see "It" (`get_subject`,
  src/mon-msg.c:320-330; a missile's note, src/player-attack.c:1122-1123), so
  the message cannot say which monster died, and a bolt can kill an untracked
  monster that stepped into its path, a ball another unseen one. With several
  believed targets, or "2 monsters die.", nothing changes. Without it the
  Borg shoots up to `unseen_max_shots` times.
- **`unseen_light_reveal`** (#889, item 3; mode 0–2). Before the attack choice
  (`borg_think_dungeon`), cast Light Room (fail 40% or less) at a believed
  monster inside its sphere: `SPHERE:LIGHT_WEAK:2` (lib/gamedata/class.txt:221-224),
  at most two grids and in line of sight for a projection, on a grid the Borg
  does not already count as lit, and once per sighting of the record (review
  of #1128, F3: lighting an invisible monster's grid again reveals nothing to
  a Borg without see-invisible). The sphere hurts
  `HURT_LIGHT` monsters and lights the grids it reaches, so the monster is seen
  or its record forgotten. 1 = only at a `HURT_LIGHT` monster, 2 = at any
  believed monster. Call Light's sphere is radius 1 (:727-730) and never reaches
  a believed target (at least two grids away). A separate step rather than an
  attack type: a new `BF_*` kind would renumber the attack kinds before
  `BF_ACT_STARLIGHT`.
- **`unseen_spread`** (#889, item 4). The belief spreads around the last grid
  (`belief_percent_spread`): an erratic mover's random steps do not count as
  leaving (RAND_25 25%, RAND_50 50%, both 75%: src/mon-move.c:1087-1101), and a
  ball still hits a monster that took no more steps than the ball covers in
  every direction from its last grid (`unseen_blast_slack`, `blast_slack`:
  the most steps `n` such that every grid `n` king's moves away is within the
  ball by the engine's `distance()`, radius 1 to 4). It is not the radius less
  the distance: one diagonal step can add 2 (from (1,1), distance 1, to
  (2,2), distance 3), so a radius-2 ball gives a monster last seen at (1,1)
  no slack (re-review of #1128 at `e0278dcf`). Bolts, beams, arcs,
  throws, Starlight's lines (radius 7) and the line-of-sight effects
  (dispels, mass sleep, slow, confuse: radius 10, which reach what the player
  sees, not a radius around the aim) take no slack (re-review of #1128, F1).
  The dragon breaths (`BF_DRAGON_*`, `BF_ACT_DRAGON_*`) are cones, not
  balls, and take none either (`ball_shaped`; Codex re-review of #1128, F2).
  So that such a ball is scored at all, `borg_attack` admits a record a ball
  could still reach (`believed_for_aim`, slack up to 3; review of #1128, F4),
  but only to the ball's own list (`blast_aims`), read by ball-shaped scoring
  alone. No other attack aims at it (Vampire Strike, melee, bolts), and it
  never makes `attack_mobile_threat_first` drop the immobile targets
  (`blast_only`; re-review F2, Codex re-review F1).
- **Beams and arcs** (#889, item 5) were already covered: every projection
  path (`borg_launch_bolt_at_location` for bolts, beams and balls,
  `borg_launch_arc_at_location` for arcs) scores a record through
  `borg_launch_bolt_aux_hack`, which asks `believed_weight`, and stages it with
  `ProjectedAttack::target`. Checked in the code, not on a recorded case.
- **Not done** (#889, item 1; now #1127): beliefs from unseen contact ("It bites you",
  `Unseen bump: occupied`) and from noises. A record needs a race, and an
  unseen attacker's is unknown; the unseen-bump response (#781,
  `caution_unseen_bump_*`) and the unseen mode of `borg_recover` (#831) already
  answer such contact apart from the belief.

| parameter | default | unit | range | source |
|---|---|---|---|---|
| `update_dark_clears_light` | 0 (shipped 1) | flag | 0–1 | `borg_update.rs:439` |
| `view_rubble_blocks_sight` | 0 (shipped 1) | flag | 0–1 | `borg_cave_view.rs:258` |
| `unseen_death_stops_shots` | 0 | flag | 0–1 | `borg_unseen_belief.rs:209` |
| `unseen_light_reveal` | 0 | mode | 0–2 | `borg_unseen_belief.rs:318` |
| `unseen_spread` | 0 | flag | 0–1 | `borg_unseen_belief.rs:475` |

## Home stock the town planner deliberately declines (#1049)

The live planner uses the same known-kind exclusions for purchase utility
and Home storage when `town_home_reject_unwanted` is enabled (the
default). A deliberately unwanted kind such as Treasure
Detection uses ordinary sale or drop paths. At Home these kinds go
before stack surplus so a known useless stack can free a slot even when
the character cannot earn gold by selling. The rule follows positive
knowledge of the kind and leaves unknown flavours to the learning
policy. A known unwanted Home stack with no buyer is withdrawn and
dropped. Temporary prices and shortages do not make a kind unwanted.

## Home stock variety (#1048)

`town_home_stack_limit` is a physical stack limit per known consumable
kind. The harness uses two, and zero keeps the previous policy. The live
plan applies it to known potions, scrolls, food, mushrooms and flasks.
It sends extra copies to a shop or the floor once the carried floor is
covered. The floor uses the higher of the configured minimum and the
planner's explicit reserve requirements. The plan retains the largest allowed Home stacks. Normal Home withdrawals can still cover a short
carried kit. Sufficiently stocked kinds yield their optional shopping
budget to other goods. The planner withdraws only the excess copies that
fit and credits a Home slot when the final copies leave. Schema 15
records the native room, and schema 16 enables partial surplus disposal.
Schema 17 lets the Borg finish clearing the surplus after withdrawing its
carried reserve from Home. Schema 18 defers cleanup if a recent shop
purchase would prevent selling the withdrawn surplus. Schema 19 leaves
protected Detect Invisible stock at Home unless every copy would exceed
the carried copy cap and can be sold through overflow.

## #863: variety in the backpack (town, prepared remedies)

Owner, 2026-11-21: "Better 4 cure poison, 4 detect invisible and 3 remove
curse in the pack and 20 light wounds in the pack than 40 light wound
potions"; "the point is being prepared to a variety of situations. You are
never going to use 35 phase doors in a trip, but a remove curse or cure
poison potion will help more." The valuation of every kind that is worth
nothing unless its condition is present now (`observe`,
`borg_town_capability.rs`) needed a redesign, not a per-curve patch (the
owner, on #863). This is its first step, for potions and scrolls; devices
(wands, staves, rods) and keeping a found device follow in a second PR.

Checked in this engine version (`object.txt` at `574b618c`): Cure Light
Wounds does not cure poison (HEAL_HP 20, CURE:BLIND, TIMED_DEC:CUT and
CONFUSED); Neutralize Poison (75 gold) and Cure Critical Wounds do
(`CURE:POISONED`). A pack of 40 Cure Light Wounds has no poison cure.

- **`town_prepared_percent`** (compiled 0, **shipped 8**; per class with
  `@<class>` or `__<class>`). Above 0, `observe` adds one row per
  situation's remedy (`PREP_POISON`, `PREP_CURSE`, `PREP_INVISIBLE`,
  `PREP_CONFUSION`, `PREP_FEAR`), fed one unit per copy by whatever supplies
  the situation's capability (`supplies_of_kind`; a device: one per charge),
  so Neutralize Poison and Cure Critical Wounds fill the poison row,
  Detect Invisible the invisible row, Remove Curse the curse row, and so on.
  Each row is a requirement (`prepared_row`, as Phase Door's reserve): each
  of the first N copies is worth this percent of the trip weight
  (`borg_town_w_trip` x exposure, as Recall's), a copy past them nothing; N is the remedy's own copies (`town_prepared_poison` 3, `_invisible` 4, `_curse` 2, `_confusion` 2, `_fear` 2). The condition-present
  curves (poisoned, cursed, afraid, ...) stay as they are, on top. 0 = the
  model has the shape it had before (no rows).
- **`town_prepared_poison`, `_invisible`, `_curse`, `_confusion`, `_fear`**
  (compiled and shipped 3, 4, 2, 2, 2; owner: "if I have 20 CLW I would
  have say 4 detect invisible and 2 remove curse"): the copies of each
  remedy wanted (nothing past them; a diminishing curve first tried let an
  8 gold Boldness pile up to 8 carried). Neutralize Poison, Resist Poison
  (also `CURE:POISONED`, 30 gold) and Cure Critical Wounds fill the poison
  row; Cure Serious and Critical Wounds the confusion row; Boldness, Heroism
  and Berserk Strength the fear row (`object.txt`). Remove Curse (100 gold)
  is always in the Alchemist's stock and Detect Invisible (15 gold) is a
  normal item there (`store.txt`): the planner buys them by value per gold
  after the urgent needs and the staples, so a richer Borg buys more of the
  dearer ones.
- Unit check (`the_first_remedies_are_worth_more_than_the_21st_cure_light_wounds`):
  with the numbers `observe` makes at CL5, DL3, 100 HP and the shipped 125
  percent healing scale, at 8 percent and 2 copies each target copy of each
  remedy is worth the same and more than twice the 21st Cure Light Wounds, a
  copy past the target nothing. Those numbers are an
  estimate, not a live reading.
- **Shipped 8** (`town_prepared_percent`: a copy is worth 8 percent of the trip weight, about what the first copy of the diminishing curve was at 25 percent), with the copies above: before it,
  the Borg never bought Detect Invisible or Remove Curse (owner: "that is what we want to change"). A hypothesis for the zoo (RULES 14, 20); candidates 25 and 50.
- The group mask (`Model::mask`, `shared_groups`) is 128 bits: capability ids
  reach 75, and an earlier 64-bit mask that folded every id from 63 up onto one
  bit merged unrelated groups (a fear remedy and a poison remedy, every
  healing potion with a Word of Recall), so the solver chose one option for
  all of them (review of #873, finding 1). Each id now has a bit of its own;
  bit 63 stays the packing-room sentinel, shared with the Recall reserve (id
  63) as in #864. Tests: `every_capability_has_its_own_mask_bit`,
  `different_remedies_are_different_groups`.
- **The staples' free copies** (`town_staple_clw`, `_csw`, `_ccw`, `_phase`,
  `_oil`, `_food`, `_light`; compiled 0 = no decay, **shipped common 6, 4, 2,
  6, 6, 4, 4**, the owner's: "Food and Light start decaying from 4, phase
  door, flasks and CLW from 6, CSW from 4, CCW 2, but again it might be class
  dependent") and **`town_staple_decay_percent`** (compiled and shipped 80).
  Owner: "make staple objects start losing value ... compared to the objects
  that are not supposed to be in the starter's shopping list"; "it must be a
  decaying curve"; "extrapolate values from what classes buy now and try to
  make it somewhat similar first, but then leaning on more variety". Past a
  kind's free copies each copy counts for the decay percent of the copy
  before it (7 free, 80: the 8th is worth 80 percent, the 9th 64, the 12th
  33, the 21st 4) and the curve sees that decayed amount (`decayed`), so the
  value keeps growing, ever more slowly, never flat. Carried copies count.
  Phase Door, oil, food and torches have one knee row on their capability
  (`STAPLE_PHASE`, `STAPLE_OIL`, `STAPLE_FOOD`, `STAPLE_LIGHT`); a copy of
  food is a Ration (30 nutrition; a Remove Hunger scroll counts as the food
  it adds), of light `borg_town_light_turns`; torches only without a lantern
  (with one the light row is off). The three healing potions share the HEAL
  curve, so each has a row of its own (`STAPLE_CLW`, `STAPLE_CSW`,
  `STAPLE_CCW`) that counts its copies and whose knee takes
  its healing units off the HEAL curve (`Model::heal_loss`: Cure Light
  Wounds 20 HP, Serious 40, Critical 60, `object.txt`). Rows hold the knee as a
  negative demand, the decay and one copy's units packed in the weight
  (`knee_row`), and carry no value of their own; `Model::gain` and
  `Model::reserve` skip the healing rows. The Home's stored healing is valued
  without the decay (`reserve`). Ammunition keeps its own caps
  (`town_ranger_arrows_max`).
- **Per class** (`harness/borg_<class>.txt`, folded into `harness/borg.txt`
  with `scripts/borg-class-files.py fold`; every parameter takes `@<class>`),
  Cure Light Wounds, Cure Serious Wounds, Phase Door and oil; food, light and
  Cure Critical Wounds are common. The values start from the owner's and move
  with what each class carries at a town departure now (36 games of 200,000
  game turns, shipped settings before this change, 4 seeds per class, Human,
  borg-zoo not used; median of the stack kept at each departure after the
  first, last count of "You have N ..." lines):

  | class | Cure Light | Cure Serious | Phase Door | Oil | Food | Torches |
  |---|---|---|---|---|---|---|
  | Warrior | 23 | 3 | 7 | 15 | 10 | 3 |
  | Mage | 18 | 2 | 13 | 10 | 8 | 0 |
  | Druid | 15 | 0 | 12 | 9 | 5 | 0 |
  | Priest | 18 | 0 | 3 | 8 | 10 | 2 |
  | Necromancer | 17 | 0 | 10 | 14 | 6 | 0 |
  | Paladin | 16 | 3 | 9 | 11 | 10 | 2 |
  | Rogue | 19 | 2 | 11 | 10 | 9 | 5 |
  | Ranger | 22 | 0 | 11 | 0 | 10 | 2 |
  | Blackguard | 22 | 2 | 5 | 14 | 11 | 3 |

  (A first count matched "Flasks of Oil" with a capital O; the game writes
  "Flasks of oil", so it saw none and this section said nobody carried oil.
  Corrected here: the table above has the real medians, and the first-visit
  purchases are 20-55 flasks for every class but the Ranger, who buys 63-90
  arrows.) Usage matters more than what is carried: per game (200,000
  turns) the Borg reads about 6-29 Phase Doors and throws 60-160 flasks, and
  the stack hit zero 0.2-2.5 times for Phase Door (Blackguard 2.5, Priest
  1.5) and 3-13 times for oil, so the decay must not cut what a trip uses.
  Cure Light and Serious Wounds are used 1-6 a game against 15-23 carried:
  the staples' decay is aimed at them.
  Cure Light Wounds = 6 x median / 19 (Warrior 7, Mage 6,
  Druid 5, Priest 6, Necromancer 5, Paladin 5, Rogue 6, Ranger 7,
  Blackguard 7); Cure Serious Wounds 4 for the classes that fight in
  melee or shoot (Warrior, Paladin, Rogue, Ranger, Blackguard), 3 for the
  casters; Cure Critical Wounds 2, food 4 and torches 4 (the owner's). **Phase
  Door and oil start above the carried medians**, near what a class uses
  (owner, 2026-11-21: "they tend to run out of them, especially rangers"):
  Phase Door Warrior 12, Mage 10, Druid 10, Priest 10, Necromancer 10,
  Paladin 12, Rogue 12, Ranger 15, Blackguard 12; oil = the class's median
  carried (Warrior 15, Mage 10, Druid 9, Priest 8, Necromancer 14, Paladin 11,
  Rogue 10, Ranger 6, Blackguard 14). All of it is a hypothesis for the zoo to
  measure (RULES 14).
- Same 36 games with the shipped values (staples only): the stacks at
  departure stay close to the baseline (Cure Light medians 11-23 against
  15-23, Phase Door 4-17 against 3-13); mean CL 8.6 against 8.8, mean max
  depth 3.58 against 3.64, 2 of 36 stopped early in both. That is a sanity
  check on 36 games, not a measurement.
- Same 36 games with the remedies shipped on (`town_prepared_percent` 8,
  copies 3, 4, 2, 2, 2) as well: bought per game, Detect Invisible 0 ->
  9.6 (the core reads them fast against invisible attackers, #831), Remove
  Curse 1.0 -> 1.0, Neutralize or Resist Poison 0 -> 0.4, a fear remedy 0.6
  -> 2.4; Cure Light Wounds 7.5 -> 6.9. Carried at a departure, the median
  is 2-4 Detect Invisible for seven classes (target 4), 0-3 poison cures,
  0 Remove Curse: it costs 100 gold, so by value per gold it comes after the
  staples and the cheaper remedies, and these games (mean CL 9) seldom have
  the gold. Mean CL 9.2, mean max depth 4.0, 1 of 36 stopped early (the
  baseline 8.6, 3.6, 2 of 36): within the noise of 36 games, no claim. A
  first version (a diminishing curve, 25 percent) let an 8 gold Boldness
  pile up (6.1 bought a game, 8 carried); the requirement shape caps it.
- Unit checks (`staples_decay_after_their_free_copies`): for Phase Door, oil,
  food and light the free copies are as before, the next keeps about 80
  percent, every later copy is worth something and less than the one
  before; each healing potion's first copy and last free copy are as before
  and the one past its knee is worth less; with 20 Cure Light Wounds carried
  the 21st is worth about a fifth of its plain value and the 3rd Remove
  Curse is worth more.
- Engine check: smoke at the shipped settings is re-recorded (the shipped
  values change every town visit); 80,000-turn games run the town plan
  without a panic.

- **Recall and Phase Door, bought more** (first measurement, on the earlier track-p and with the 64-bit mask of the first review finding; superseded by the re-measurement below) (owner, 2026-11-21: "current
  characters do not buy enough world of recall and should buy more, and
  probably more phase doors as well, they tend to run out of them, especially
  rangers"). Shipped: `town_recall_reserve` 1 -> 2, `buy_recall_from_gold`
  400 -> 200 (a scroll is bought only when that much gold remains after it),
  `town_phase_reserve` 0 -> 4 for every class (Ranger 2 -> 8, Blackguard 8,
  Priest 6, class files; raised again below). Neither Word of Recall nor missiles are touched by
  the staples' decay (Recall keeps its own curve, #864; missiles their own
  caps). Before (36 games, 200,000 turns, the share of town departures after
  the first that carry no Word of Recall): Warrior 39 percent, Mage 94,
  Druid 80, Priest 62, Necromancer 100, Paladin 31, Rogue 57, Ranger 68,
  Blackguard 75. After the reserve alone (2): 40, 78, 99, 61, 66, 55, 64, 57,
  81, no better, since the 400 gold floor held it back. With the floor at
  200: 57, 57, 62, 33, 40, 71, 29, 41, 85: better for the casters, Rogue and
  Ranger, worse for Warrior, Paladin and Blackguard (4 starts a class, noise
  or the floor; the zoo arm will say). Departures with 5 or fewer Phase
  Doors before / after: Warrior 24 / 22 percent, Mage 2 / 3, Druid 15 / 0,
  Priest 64 / 18, Necromancer 0 / 3, Paladin 15 / 10, Rogue 15 / 13, Ranger 5 /
  3, Blackguard 50 / 0; Phase Doors bought per game 3.5-12 before, 7.8-14.5
  after (Rogue 3.5 -> 9.0, Blackguard 3.8 -> 14.5); times the stack hit zero
  at most 1.5 a game against up to 2.5. Mean CL 8.8 / 8.75, max depth 3.64 /
  3.50, 2 / 1 of 36 stopped early: no change beyond noise.
- **Re-measured after the review of #873** (finding 1 fixed, `track-p` with
  #864's review fixes and #831's Detect Invisible curve merged in; 36 games,
  200,000 turns, 4 starts a class). The control is the same build with every
  #863 value set back (the zoo's `ctl` lines: staples, remedies and the depth
  gate off, Recall reserve 1 and floor 400, Phase Door reserve 0, Ranger 2), the
  earlier numbers having used an older baseline that lacked #831 and #864's
  fixes. Bought a game, control / shipped: Detect Invisible 6.2 / 8.4 (the
  control already buys them since #831), Remove Curse 0.1 / 0.4, a poison cure
  0.0 / 0.4, a fear remedy 0.3 / 1.7, Phase Door 7.8 / 10.9, Word of Recall 3.7 /
  4.7, Cure Light Wounds 10.4 / 6.8. The share of departures carrying no Word of
  Recall, control / shipped: Warrior 50 / 40 percent, Mage 84 / 74, Druid 66 / 60,
  Priest 56 / 65, Necromancer 88 / 20, Paladin 51 / 39, Rogue 64 / 38, Ranger 79 /
  59, Blackguard 66 / 59 (better in eight classes of nine). Departures with 5 or
  fewer Phase Doors, control / shipped: Warrior 46 / 32, Mage 17 / 25, Druid 1 /
  15, Priest 28 / 34, Necromancer 21 / 8, Paladin 57 / 39, Rogue 14 / 28, Ranger
  20 / 25, Blackguard 50 / 3: better in four classes, worse in five, so the Phase
  Door reserve did not fix running low in every class. Mean CL 9.11 / 9.11, mean
  max depth 3.78 / 3.83; stopped early 0 / 3 of 36 (a Druid at CL2 on DL1, a Druid at
  CL7, a Paladin at CL11; the first visit bought the same kit in both arms: eight
  Cure Light Wounds, 27 flasks, the same Phase Doors): within what 36 games can
  show, and what the zoo pilot is for.
- **Phase Door, raised again** (owner, 2026-11-22: "Ranger seems to run out
  more often of phase doors", "but the other ones too"). The first
  measurement counted every "Town departure room" line as a visit, but the
  Borg also bounces between DL1 and town at CL1 without shopping, so a Ranger
  "left with 3" nineteen visits running without ever shopping; the measure
  that follows counts only visits that bought three or more things, and the
  times the Phase Door stack hit zero in the dungeon. 36 games, 200,000
  turns, 4 starts a class, same build. Control (all #863 values back) /
  Phase Door reserve 4 (Ranger, Blackguard 8, Priest 6; knees 10-15) /
  **shipped now**: stack hit zero a game 0.58 / 0.28 / 0.14; after a shopping
  visit 5 or fewer Phase Doors: Warrior 50 / 33 / 0 percent, Mage 39 / 36 / 7,
  Druid 4 / 6 / 4, Priest 41 / 22 / 0, Necromancer 20 / 15 / 0, Paladin 50 / 37 /
  0, Rogue 19 / 38 / 0, Ranger 39 / 13 / 4, Blackguard 66 / 0 / 0; 8 or fewer:
  Ranger 78 / 50 / 8, Blackguard 91 / 50 / 4; median carried 5-9 / 6-12 / 6-16.
  Mean CL 9.11 / 9.11 / 8.92, mean max depth 3.78 / 3.83 / 3.78, stopped early
  0 / 3 / 2 of 36. Shipped: `town_phase_reserve` 6 (Ranger 10, Blackguard 10,
  Priest 8), `town_phase_reserve_percent` 150 (each reserved scroll worth 150
  percent of the trip's escape weight, to outbid arrows and flasks at a small
  purse), and the Phase Door free copies Warrior 16, Mage 12, Druid 12, Priest
  12, Necromancer 12, Paladin 14, Rogue 16, Ranger 20, Blackguard 16. A hypothesis
  for the zoo's `kit` arm. The reserve's range stops at 10.
- **`town_variety_min_depth`** (compiled 0, **shipped 2**; owner: "it is
  important that in the first level we have the staples"): the staples'
  decay and the remedies only apply once the deepest level reached
  (`BI_MAXDEPTH`) is this, so the first visits buy as before #863: a first
  count showed one to five Detect Invisible bought on the first visit
  (15 gold each, at starting gold) and the oil decay cutting first-visit
  flasks by a quarter to a third.

| parameter | default | unit | range | source |
|---|---|---|---|---|
| `town_prepared_percent` | 0 | percent | 0–10000 | `borg_town_capability.rs:1063` |
| `town_prepared_poison` | 3 | copies | 1–20 | `borg_town_capability.rs:1064` |
| `town_prepared_invisible` | 4 | copies | 1–20 | `borg_town_capability.rs:1065` |
| `town_prepared_curse` | 2 | copies | 1–20 | `borg_town_capability.rs:1064` |
| `town_prepared_confusion` | 2 | copies | 1–20 | `borg_town_capability.rs:1065` |
| `town_prepared_fear` | 2 | copies | 1–20 | `borg_town_capability.rs:1066` |
| `town_staple_decay_percent` | 80 | percent | 0–100 | `borg_town_capability.rs:1106` |
| `town_staple_clw` | 0 | copies | 0–99 | `borg_town_capability.rs:1110` |
| `town_staple_csw` | 0 | copies | 0–99 | `borg_town_capability.rs:1111` |
| `town_staple_ccw` | 0 | copies | 0–99 | `borg_town_capability.rs:1112` |
| `town_staple_phase` | 0 | copies | 0–99 | `borg_town_capability.rs:1145` |
| `town_staple_oil` | 0 | copies | 0–99 | `borg_town_capability.rs:1146` |
| `town_staple_food` | 0 | copies | 0–99 | `borg_town_capability.rs:1147` |
| `town_staple_light` | 0 | copies | 0–99 | `borg_town_capability.rs:1148` |
| `town_variety_min_depth` | 0 | depth | 0–127 | `borg_town_capability.rs:2436` |

## A monster at 0 HP is alive (#975)

The registry now holds **2388** parameters (2387 on track-p `2415bedf`
before it; one added). The engine kills a monster only when its hit points
fall below 0 (`mon_take_hit`, `engine/angband/src/mon-util.c:1300`), so a
monster hit down to exactly 0 lives. #546's floor (`kill_live_min_power`,
shipped 1) applies only to `hp > 0`, so such a monster is estimated at 0 HP.
The ranged and melee overkill caps (`borg_fight_attack.rs:694`, `:1629`)
then score every attack on it 0, and the Borg stops shooting: a 2-HP grey
mold hit for 2 by an arrow (Hobbit Ranger 282602274, turn 5444) was not
shot again, `borg_flow_old` stepped next to it and its spores killed the
Ranger. With `kill_live_zero_hp = 1` the floor also covers hp 0; the flag
needs `kill_live_min_power` above 0 (it ships at 1), and with the floor at 0
it changes nothing. The
focused fight planner (#759) already counts hp + 1 (`observed_kill_hp`).
Compiled 0; `harness/borg.txt` ships 1 since #1034 (owner, 2026-11-22: "the
flag should be turned on"); borg-zoo#670 measures it.

| parameter | default | unit | range | source |
|---|---|---|---|---|
| `kill_live_zero_hp` | 0 | flag | 0–1 | `borg_flow_kill.rs:387` |

## The emergency attack spell beyond the current mana (#980)

`borg_attack_aux_spell_bolt_reserve` (`borg_fight_attack.rs:3196`, the C
Borg's `borg-fight-attack.c:2520`) casts Magic Missile **only** when the Borg
lacks the mana for it, with its mana faked to the maximum for the fail check.
The engine always faints the caster of such a spell (`player-spell.c:540-553`:
`player_over_exert(PY_EXERT_FAINT, 100, 5 * oops + 1)`, a paralysis that free
action does not stop) and adds 5% fail per missing point (`player-spell.c:404-406`),
which the faked mana hides; it also damages CON half the time
(`player_over_exert(PY_EXERT_CON, 50, 0)`, permanent only at chance 50 or more
with a second roll, `player-util.c:825-832`). The kill test uses twice the missile's average
damage plus 4 (`(((CL-1)/5)+3)*(4+1)` against `power`), so a monster that
survives the cast gets free turns. The #842 attack-on-sight path chose it for
crows (592754909 on track-p, 1378676237 on main f57499d0).

| parameter | default | unit | range | source |
|---|---:|---|---|---|
| `attack_reserve_spell_overexert` | 1 | flag | 0–1 | `borg_fight_attack.rs:3456` |

1 is the C Borg's play (today's). 0 never casts the spell beyond the current SP.
Not shipped; the zoo measures it (a request issue). The four replays are
listed in PR #991; the games are the zoo report's (borg-zoo#668), not yet in
borg-cemetery. The `= 0` arm is the shipped `harness/borg.txt` plus the line
`borg_param_attack_reserve_spell_overexert = 0` in the run's `user/borg.txt`
(the key is not in the shipped file, so `BORG_TXT_SET` cannot reach it, and the
added line changes the run's `borg_txt_sha256`).

The registry now holds **2394** parameters (2393 on track-p `e26cb7b0` before
it; one added).

**With #878's guards** (`attack_reserve_faint_percent`,
`attack_reserve_spell_damage_only`, both compiled 0 and read only inside this
function). The #980 gate comes first, so at `attack_reserve_spell_overexert = 0`
the cast never happens and both #878 settings are inert. At 1 they work as
before. The two answer the same question differently. `= 0` removes the cast
outright. #878's percent keeps it when the damage expected during the faint is
small. That estimate weights the target by the spell's fail chance alone, so it
assumes a successful cast kills, and the doubled kill test makes a survivor
likely. On 592754909, `attack_reserve_faint_percent = 50` and `= 100` both
refuse the cast ("3.0 damage expected over a 3.5-turn faint, HP 3") and play
the same keys as `= 0` (`f115048321c12344`). At 100% that refusal is at the
margin: from 4 HP up, the same cast is allowed and the crow survives it.
Which one ships, and whether #878's settings retire with `= 0`, is the owner's
decision after the zoo (borg-zoo#675).

## #987: first cures before the Phase Door reserve

A 127-gold Elf Ranger (seed 681096527, track-p `4b2c2fcf`) bought 6 Phase
Door and no Cure Light Wounds, then died to a crow at 1 HP with nothing to
drink. In its plan record at game turn 541 each reserved Phase Door
(`town_phase_reserve` 10 for the Ranger, 150 percent of the 120,000 escape
weight: 180,000 a scroll, plus the curve) outbids the first CLW (70,588 on the
healing curve plus 100,000 on the healing reserve at 14 HP). The healing
reserve values a potion at most one full heal (`borg_town_w_bleeding`), so
the order holds at any HP. Low gold only decides whether anything is left
after the reserve. `town_first_cure` adds a row, "first cure": the first N
healing potions are a requirement, each worth `town_first_cure_percent` of
the escape weight, nothing past N. Off (0) the model keeps its shape: the
seed plays the same keys with byte-identical plan records. **Shipped: 2 for
the Ranger** (owner, 2026-11-22: "let us merge like that having the value 2
for clw for rangers"), 0 for every other class. Unmeasured by the zoo
(#1006): on the one seed the result flipped between two `track-p` builds (a
lead only, RULES 14).

| parameter | default | unit | range | source |
|---|---|---|---|---|
| `town_first_cure` | 0 (off) | count | 0–10 | `borg_town_capability.rs:2501` |
| `town_first_cure_percent` | 200 | percent | 0–10000 | `borg_town_capability.rs:1062` |

## Grouped monster messages (#915)

The engine writes the subject of a message about several monsters of one race
as "N <plural>" (`mon-msg.c:333-345`: "3 cave spiders start moving
faster."). `guess_race_name` took every name without "The " for the player
ghost, race `r_max - 1`, which in 4.2 is Morgoth (`mon-init.c:1792-1810`), so
those messages were lost (seed 2008390085, borg-zoo#596). At 1 the count and
the plural (monster.txt's `plural:`, or `plural_aux` undone, `mon-desc.c:27-36`)
name the race, and "N monsters" is an unseen group. A grouped death or blink
("3 cave spiders die.") removes up to N known monsters of the race, not one
(`grouped_count`, `borg_update.rs`; review of #943). With this section the
registry holds **2398** parameters (`track-p` `72651d25` plus
`kill_grouped_names`). Compiled 0; shipped 1
in `harness/borg.txt` (P8): every non-unique race of `monster.txt` (523)
reads back from its grouped subject, with a count of 2 or 15 and with
" (offscreen)" (`grouped_subjects_of_every_engine_race`), and seed 2008390085
plays the same keys, state and core trace (danger at every think) with the
switch off or on, both on the zoo's game (`7f8c6b80`) and on `track-p`.

| parameter | default | unit | range | source |
|---|---:|---|---|---|
| `kill_grouped_names` | 0 | flag | 0–1 | `borg_flow_kill.rs:1283` |

## Traps that cut the way back (#1067)

The Half-Elf Warrior 1868115537 (borg-zoo run 45070528012, track-p
`206cf4e7`, the same keys on `7a466d73`; diegoami/borg-cemetery#138, study
`lvl1-deaths-trackp-206cf4e7`): four steps from its up staircase, at 31/31 HP,
it set off a poison pit that filled the only corridor back. Below 60 HP every
known trap is a wall to the flows (`spread_blocked`: C's "Avoid Traps if low
level -- unless brave"), so the staircase became unreachable ("Bored but
unable to flow to up stairs.", 24 times), `borg_flow_cost_stair` went "off
leash" (#1062), and it explored on into a blinding gas trap and died 5 steps
from the stairs. New module `borg_trap_way.rs`, run in the dungeon think
after the attack and wear checks, before the item flows. All switches are
compiled 0. **Shipped** by the owner on 2026-11-23 before a zoo
measurement ("merge but set the values to try"): `trap_recover = 1`,
`trap_way_back = 1`, `trap_leave_level_count = 2` in `harness/borg.txt`
(`trap_leave_near_count` stays 0). Smoke plays the same keys and state as
before (one trap note added in smoke-mage-2). The values are hypotheses
until borg-zoo measures them (RULES 14).

A trap is **set off** when the engine prints one of trap.txt's activation
messages (`msg`, `msg-good`, `msg-bad`, `msg-xtra`; `hit_trap`, trap.c) or
"You set off the %s!" (`do_cmd_disarm_aux`, cmd-cave.c:842-845); a unit test
checks every trap.txt message is read. Messages another source also prints
(the fire blow's "You are enveloped in flames!", "The ground shakes.") count
only when the Borg then stands on a known trap. The kind it names (a pit, a
spiked or poison pit, a blinding or confusion gas trap) is remembered by grid
for the level.

1. **Recover at the stairs** (`trap_recover`). Below `trap_recover_max_hp`
   max HP, a trap that hurts (at the next think: HP below the maximum,
   poisoned, cut, blind, confused or stunned) stops exploration: the Borg
   flows to the nearest staircase it may take (up unless descent is forced;
   down only when not rising and not refused, `more_refused`; refusing traps
   as today), cures or rests there (`borg_recover` first, else `R&`), takes
   it at the first threat (danger above 0 on it), and explores again whole.
   No such staircase in reach: it rests where it stands (danger 0, not
   hungry), curing first. The hook runs after the attack and the wear
   checks, before the item flows, so an item cannot pull a hurt Borg away
   (review of #1086).
2. **Reopen the way back** (`trap_way_back`). When a spread from every known
   staircase that refuses traps does not reach the Borg and one through them
   does, the first trap of that route is cleared before anything else: from
   the adjacent grid `D` (disarm) when the engine's chance (the skill less
   depth/5, at least 2; physical for pits and gas, the lower skill for an
   unknown kind) is at least `trap_disarm_min_percent`; else, only while rule
   1 or 3 wants the stairs now (a walk leaves the trap where it is, and the
   Borg would cross it back and forth to explore), `W` (walk onto it,
   `CMD_JUMP`) when its kind is known and its worst cost (a pit 12 HP, a
   spiked pit 48, a poison pit 84 counting cut and poison turns, gas 0) is
   below the current HP; else, hurt, it rests first and disarms whole. Not
   adjacent, it follows the flow through the traps (`GOAL_MISC`). Blind,
   confused or hallucinating it rests first; without light it leaves the
   light check to act. After `trap_disarm_max_tries` attempts on one trap
   the rule gives up (play is then as today). With the switch on, a flow may
   leave a trap the Borg stands on (after a `W` onto a trap that stays) but
   enters no other (`trap_underfoot_open` in `spread_blocked`). The probe
   counts only the staircases rule 1 may take.
3. **Leave a trap-dense level** (`trap_leave_level_count`,
   `trap_leave_near_count`). Below `trap_recover_max_hp`, the level's n-th
   distinct trap set off, or the m-th within `trap_leave_radius` of the
   Borg's first grid on the level (a separate threshold: every trap near the
   arrival is also one of the level), makes it leave by the nearest staircase
   it may take once it reaches one, whole first when rule 1 is on (the owner:
   "the form of a cursed level"). A trap is placed at its own grid: a `D` or
   `W` at its target (this module's or `borg_play_step`'s disarm); a walk
   where the Borg then stands (a confused step may be turned,
   cmd-cave.c:1299-1312); a teleport rune at the step's grid (`goal.g`, one
   grid from the Borg when the message came) unless it was confused, so a
   rune set off twice is one trap. A trap it cannot place (a disarm message
   with no target, a confused teleport) is not counted. The cut-off probe and
   the walk back seed the free staircases first, an occupied one only as the
   stair flows' #597 fallback.

`trap_recover_max_hp` is 60, the core's own line for a known trap and above
every CL1-6 character; #1061's stair_scum threshold (25) would not cover this
Warrior (31, then 57 HP). Per class with `trap_recover_max_hp__<class>`.

| parameter | default | unit | range | source |
|---|---:|---|---|---|
| `trap_recover` | 0 | flag | 0–1 | `borg_trap_way.rs:341` |
| `trap_recover_max_hp` | 60 | max HP | 1–10000 | `borg_trap_way.rs:389` |
| `trap_way_back` | 0 | flag | 0–1 | `borg_trap_way.rs:285` |
| `trap_disarm_min_percent` | 50 | percent | 2–100 | `borg_trap_way.rs:625` |
| `trap_disarm_max_tries` | 10 | attempts | 1–1000 | `borg_trap_way.rs:610` |
| `trap_leave_level_count` | 0 | traps | 0–100 | `borg_trap_way.rs:343` |
| `trap_leave_near_count` | 0 | traps | 0–100 | `borg_trap_way.rs:344` |
| `trap_leave_radius` | 10 | grids | 0–250 | `borg_trap_way.rs:466` |

## #895: devices as variety (town)

Owner, 2026-11-21 (#863, #895): "we are going to need space for other devices
so it is already a good list"; a found wand is kept, not sold; a Staff of
Teleportation only with spare gold. Checked before changing anything: a wand,
staff or rod was valued only by what it adds to the shared curves
(`supplies_of_kind`: its effects times its charges, times
`town_devices_percent`), so an attack wand competed with arrows and oil, whose
curves a quiver saturates. In 36 games (9 classes, 4 seeds, 200,000 turns)
the Borg sold 42 unidentified devices (by design since #470: an unaware device
is never used, so it is sold to learn the flavour), and of the identified ones
it sold only empty wands (0 charges: Slow Monster, Confuse Monster, Light,
Magic Missile) and wands on the worthless list (Darkness, Wonder).

- **`town_device_percent`** (compiled 0, **shipped 24**; per class with
  `@<class>`), with **`town_device_attack`** 10, **`_escape`** 5, **`_detect`** 5
  and **`_control`** 5 (charges). Above 0, `observe` adds one requirement row
  per situation a learned device answers, counted in charges (`DEV_ATTACK`:
  Magic Missile, Stinking Cloud, damage rods; `DEV_ESCAPE`: a Staff of
  Teleportation; `DEV_DETECT`: Detect Evil, Mapping;
  `DEV_CONTROL`: Slow, Hold, Confuse Monster, and Teleport Other, which keeps a
  monster away and is counted with them). A Staff of Detect Invisible feeds the
  prepared Detect Invisible row (#863), one copy a charge up to
  `town_prepared_invisible`, and no device row: a second row would count it
  twice (review of #898, F2). A Wand of Light's damage is `LIGHT_WEAK`, which
  hurts light-sensitive monsters only, so it does not fill the attack row (F6);
  the shared ATTACK curve counts it as before. The first N
  charges together are worth `town_device_percent` of the trip weight (24
  percent of 50,000 is 12,000, each of ten attack charges 1,200), nothing past
  them. Charges are scaled by `town_devices_percent` (the device skill by class,
  Warrior 75, Mage 150, ...), the same scale the shared curves use. A learned,
  charged device is therefore kept and bought as variety; an empty one supplies
  no charges and no value, a recharge being the next slice. Like the remedies
  it starts with `town_variety_min_depth` (the first level buys the staples).
- **`town_device_escape_min_gold`** (compiled 0, **shipped 3000**): below this
  purse after the urgent needs, a **Staff of Teleportation on offer supplies
  nothing**: neither its device row nor the shared `TELEPORT` escape curve (the
  review of #898, F10, measured that closing the device row alone removed only
  23 to 30 percent of a staff's value, and the staff was still bought if nothing
  better fit). A staff already carried is never touched (its supplies come from
  the kit), so it is never sold because the purse fell during a visit
  (`escape_gated`, `without_escape`, `carried_escape_charges`). Other things that
  fill the escape curve, a Scroll of Teleportation among them, are unchanged. The
  threshold is on the purse, not on the price (a staff costs 2000 base): 3000
  means "keeps about 300 after buying one", and a Borg with 2999 buys none though
  it could afford it. What the Borg pays is the base cost plus `cost x charges /
  20` (`obj-power.c`), so a Staff of Teleportation (5+d4 charges) is about 2700
  gold and a Wand of Magic Missile (6+d10 charges) about 310. By value per gold
  paid a staff (12,000 over 2700, about 4 a gold) comes far behind the cheapest
  device that fills a row, a Wand of Magic Missile (about 39 a gold; a Wand of
  Stinking Cloud is dearer, 400 base), so even above the floor it comes last.
- **One device per row.** A row's demand caps its value, so a second device that
  answers the same situation adds nothing on that row: a Staff of Detect Evil
  already fills the five detection charges, and a Staff of Mapping then adds
  next to nothing there. The variety is at most one device per situation, four
  devices, not a long list.
- **Two settings one character apart:** `town_device_percent` is the device
  rows' weight (shipped 24, this section); `town_devices_percent` (#666) scales
  every device's charges by the class's device skill (100, Warrior 75, Mage
  150). Setting the second to 24 by mistake is accepted and means "every
  device's charges count for a quarter", with the rows still off.
- Unit check (`a_charged_device_is_kept_as_variety`): a ten-charge attack wand
  is worth the target (12,000) against next to nothing on an ATTACK curve
  saturated by shots and flasks; each charge up to ten is worth the same and an
  eleventh nothing; the gate switches off below the floor, not at it, and not
  with one carried.
- Engine check, 36 games with the device rows on against off on the same
  build: mean CL 8.33 and 8.33, max depth 3.44 and 3.44, 3 of 36 stopped early
  in both; devices bought differ by one wand (a Wand of Stinking Cloud); the
  devices sold are the same 42 unidentified flavours, the empty wands and the
  worthless kinds. In games this short the Borg finds and affords few learned
  devices, so the check shows the rows run without a panic and change little,
  not what they are worth: the zoo pilot at 300,000 turns is the measurement.
- **Not done:** an unidentified device is still sold to learn its flavour
  (#470); keeping it would need the Borg to use it (aiming an unknown wand was
  removed in #470), or to learn it from the Magic shop's stock. An empty device
  is sold, not recharged (a Scroll of Recharging is 200 gold).

| parameter | default | unit | range | source |
|---|---|---|---|---|
| `town_device_percent` | 0 | percent | 0–10000 | `borg_town_capability.rs:991` |
| `town_device_attack` | 10 | charges | 1–99 | `borg_town_capability.rs:992` |
| `town_device_escape` | 5 | charges | 1–99 | `borg_town_capability.rs:992` |
| `town_device_detect` | 5 | charges | 1–99 | `borg_town_capability.rs:993` |
| `town_device_control` | 5 | charges | 1–99 | `borg_town_capability.rs:993` |
| `town_device_escape_min_gold` | 0 | gold | 0–10000000 | `borg_town_capability.rs:993` |

## Breadth-first exploration from the staircases (#1080, #1085)

The owner (2026-11-23): "it seems to me that the borg explores depth-first,
but a safer way would be breadth-first." C's `borg_flow_dark` takes the first
of five methods that finds a target, and every one measures from the Borg:
in a corridor the nearest unknown grid is the next one ahead, so the Borg
follows the corridor to its end. At `explore_breadth_first = 1`, while the
Borg's maximum HP is below `explore_bfs_max_hp`, both exploration calls (near
and far) use one method instead (`borg_flow_dark_breadth_first`, the first
piece of #1080, split out as #1085):

1. one spread from every known staircase, up and down, over known grids
   gives each grid its **ring**, its flow steps from the nearest staircase;
   a grid the spread does not enter (an unknown grid, rubble, a vein, a
   known trap below 60 HP) is one more than its nearest entered neighbour;
2. the candidates are the interesting grids of the whole known map (methods
   3–5's, reachable, and method 1's torch-lit ones) within the leash
   (`borg_get_leash`, where C's distance test applies, measured from that
   nearest staircase, not from the farthest up one);
3. the nearest ring *d* is explored first (`plan`): all the candidates from
   *d* to *d* + `explore_bfs_slack` are goals of one spread, so the flow leads
   to the one nearest the Borg (the tie-break). A path may not cross a grid
   of a ring farther out than *d* + slack, or than the Borg's own when it
   stands farther out (its way back passes there): no target is reached
   around behind the Borg's back. With no path, the next ring (at most 16
   per think);
4. a staircase newly seen (`borg_update`), or a monster newly tracked, in
   view or not (`borg_flow_kill`), drops a committed exploration flow, so the
   next think plans again (reviews of #1088);
5. only the candidates the Borg can reach (one spread from the Borg, through
   no grid past the return leash, as the route) enter the plan, so rings it
   cannot walk to do not use up the 16 tries.

While the "wandered too far" return leash of `borg_think_dungeon` applies
(`return_leash_with`: 3 x CL + 14 from the nearest up staircase, by `distance`; with `flow_leash_any_stair`, #1062, the walking steps to the nearest staircase of either kind),
no target and no path grid lies past it. Without that, a target near a down
staircase but past the return leash sent the Borg out and back: 622 returns
in 20 games against 65 for the control, and one Necromancer at CL3 on DL1
after 100,000 turns.

Under boosted bravery (avoidance above HP: the flow then crosses unknown
grids and known traps), and when the Borg itself has no route to a known
staircase (none known, a
teleport into an unexplored region, a known trap across the only way back,
#1067), C's methods run as before (review of #1088: the stair side's
candidates no longer stop them). When no candidate is left within the
leashes, neither call explores, and the think is marked exhausted (#1098;
owner, 2026-11-23: "once there is nothing to explore, take the stairs"):
`borg_leave_level` takes the stairs then, as when the level timer is up,
before the secret-door search, with the same exit as the mixed walk (below). Behind
`explore_bfs_leave_when_done` (compiled 0, shipped 1); at 0 the Borg stays
until the level timer or another reason to leave, as before.

**C's own exploration** (#1098, owner 2026-11-23, option 2 of the
secret-door searches). In the 20 replays of the PR, after the breadth-first
exit, 578 of the 587 secret-door searches were C's: characters above the
breadth-first threshold, whose far exploration had found nothing while no
reason to leave applied yet. At `explore_c_leave_when_done = 1` (compiled 0,
shipped 1: owner, 2026-11-23, "merge it but with all settings on"), when the far methods (4 and 5; method 5 scans the whole known
map within the leash) find nothing, the think is marked exhausted.
`borg_think_dungeon` then calls `borg_leave_level` right after the far
exploration, before the door-finding spell and the secret-door search. The
exit is the same as above: the other direction if a random one is out of
reach. If no staircase can be taken, the level is released and the search
runs as before (a secret door can be the only way to an up staircase). The
mark waits while the Borg is impaired: confused, blind or hallucinating,
under boosted bravery, or in an invisible-attacker episode. Then exploration
and the stairs fail for its state, not the map. In the replays, a confusion
trap and an invisible attacker each released a whole level that way, and C
then stayed stuck for thousands of turns. The note is
`# Flowing toward dark-bfs (ring d) at cost n`.

**The threshold by level and class** (owner, 2026-11-23): breadth-first
applies while max HP < `explore_bfs_max_hp` + `explore_bfs_hp_per_level` x CL,
both values per class (`__<class>`). The engine gives a character its hit die
(race + class, `player-birth.c:999`) at CL1 and `randint1(hitdie)` more per
level (`:295`), so the delta sets how many levels the switch lasts. A delta
near a quarter of the class's mean gain keeps a level or two more for the
frail races (Hobbit, Kobold) than for the sturdy ones (Half-Troll). A delta
of half the gain makes the threshold grow almost as fast as the HP of a small
hit die: a Hobbit Mage would stay breadth-first to CL12.

**The shipped list** (owner, 2026-11-23: "ship it like that ... turn it on by
default": `explore_breadth_first = 1` in `harness/borg.txt`, these values in
the class files; compiled 0, so a profile without them plays C's order). It is
not measured yet (RULES 14): borg-zoo compares it with the control. For each class, the
target is the last CL with at least one DL1 death per 100 games that reached
it, in #1080's pooled table. That table pools every build, and recent runs are
5–10 times lower, so the cut at 1 per 100 is itself a hypothesis. The delta is
a quarter of the Human mean gain, and the base puts the threshold halfway
between the mean max HP at the target and the next CL (CON ignored). The owner
then raised every `explore_bfs_max_hp` by 10 (2026-11-23: "increase those
explore_bfs_max_hp by 10") and lowered them by 3 ("reduce those numbers ...
by 3", #1110's PR); the table gives the shipped values and the levels they
reach:

| class | hit die (Human) | target CL | `explore_bfs_max_hp` | `explore_bfs_hp_per_level` | Human through | all races through |
|---|---:|---:|---:|---:|---:|---:|
| warrior | 19 | 1 | 28 | 2 | CL2 | CL1–3 |
| mage | 10 | 4 | 32 | 1 | CL6 | CL4–9 |
| druid | 12 | 4 | 33 | 2 | CL6 | CL4–9 |
| priest | 12 | 3 | 28 | 2 | CL4 | CL3–7 |
| necromancer | 12 | 3 | 28 | 2 | CL4 | CL3–7 |
| paladin | 16 | 2 | 31 | 2 | CL3 | CL2–4 |
| rogue | 14 | 3 | 33 | 2 | CL4 | CL3–6 |
| ranger | 15 | 4 | 41 | 2 | CL5 | CL4–7 |
| blackguard | 18 | 1 | 27 | 2 | CL2 | CL2–3 |

The race delta of #1061's threshold (`__<race>`) is not part of this change:
the suffix is #1061's, and this threshold takes it when that lands.
borg-zoo measures the shipped list against `explore_breadth_first = 0`, and
the arms of #1085 (`explore_bfs_max_hp` 20, 25 and 30). With this
section the registry holds **2448** parameters (`track-p` `57d47863` plus the four
below).

| parameter | default | unit | range | source |
|---|---:|---|---|---|
| `explore_breadth_first` | 0 | flag | 0–1 | `borg_flow_dark.rs:878` |
| `explore_bfs_max_hp` | 25 | hp | 0–10000 | `borg_flow_dark.rs:881` |
| `explore_bfs_slack` | 0 | steps | 0–250 | `borg_flow_dark.rs:1228` |
| `explore_bfs_hp_per_level` | 0 | hp per level | -100–100 | `borg_flow_dark.rs:882` |

## More food and light while breadth-first exploration applies (#1110)

The owner (2026-11-23), after #1085 shipped breadth-first exploration on: "I
guess we should plan a little more food and light if we do that". The order
walks back toward the stairs between targets, so a trip of the same depth
takes more game turns. The live town planner (`borg_town_capability::observe`)
sizes food from the trip (`journey.turns / borg_town_food_turns` rations,
never below the five-ration floor) and light from the larger of the trip and
`town_light_reserve_turns` (20,000 shipped); both floors usually dominate, so
lengthening the trip alone would buy nothing more.

While breadth-first exploration would apply to the character
(`breadth_first_applies`: `explore_breadth_first` on and max HP below
`explore_bfs_max_hp` + `explore_bfs_hp_per_level` x CL, asked in town too),
the food target, floor included (`food_curve`), and the light goal, reserve
included, are `explore_bfs_food_percent` and `explore_bfs_light_percent` of
today's. Compiled 100; shipped 150 (owner: "a little more"): 7.5 rations
instead of 5, and 30,000 light turns instead of 20,000 (two more torches).
Not measured (RULES 14). The smoke Priest buys 3 torches and 7 rations.

One goal for every reader (review of #1113): `borg_light::fuel_goal` gives
the scaled light goal to the planner's curve, to the torch replenishment that
runs before other shopping (`borg_town_shadow`), and to the dungeon guards
that keep fuel from being thrown (`borg_fight_attack`) or junked
(`borg_light::keep_fuel_stack`). Without that, the replenishment stopped at
20,000 turns and combat could throw the oil bought for the extra 10,000.
Both percents are at least 100, so the five rations `borg_prepared` asks for
stay planned.

| parameter | default | unit | range | source |
|---|---:|---|---|---|
| `explore_bfs_food_percent` | 100 | percent | 100–10000 | `borg_flow_dark.rs:896` |
| `explore_bfs_light_percent` | 100 | percent | 100–10000 | `borg_flow_dark.rs:896` |

## stair_scum (#1061)

The owner's proposal from the level-1 death review of `track-p` `206cf4e7`
(borg-zoo run 45070528012; borg-cemetery study
`studies/lvl1-deaths-trackp-206cf4e7`): an exploration **mode** for a
character with few hit points, "they can go back to the stairs faster than
any character can come at them". With this section the registry holds
**2461** parameters (9 added to `track-p` `40c1c963`). `stair_scum` is compiled 0 and **shipped 1** in
`harness/borg.txt` (owner, 2026-11-23: "ok let us merge with those settings"),
with the owner's `byclass` arm of borg-zoo#721: `stair_scum_max_hp = 20`, slack
per class in `harness/borg_<class>.txt` (Druid and Necromancer 1, Mage and Priest
2, Ranger and Rogue 3, Paladin and Blackguard 4; the Warrior with the mode off),
the level rate at its default 50. It ships before the zoo's pilot reports; the
pilot (all arms explicit) is pinned to `0d09e318`, which ships it 0. Every number is a hypothesis (RULES 14): borg-zoo
measures arms at a threshold of 20, 25 and 30, then per class. The design
follows Codex's review on #1061 (the attacks on the way counted, the
engine's light, every step checked, the unplaced door sound).

| parameter | default | unit | range | source |
|---|---:|---|---|---|
| `stair_scum` | 0 | flag | 0–1 | `borg_stair_scum.rs:152` |
| `stair_scum_max_hp` | 25 | HP | 1–1000 | `borg_stair_scum.rs:159` |
| `stair_scum_max_hp_race_delta` | 0 | HP | -100–100 | `borg_stair_scum.rs:159` |
| `stair_scum_speed_ratio` | 2 | actions per player action | 1–10 | `borg_stair_scum.rs:559` |
| `stair_scum_contact_damage` | 4 | HP per attack | 1–1000 | `borg_stair_scum.rs:539` |
| `stair_scum_slack_steps` | 0 | steps | 0–50 | `borg_stair_scum.rs:541` |
| `stair_scum_slack_per_hp_tenths` | 0 | tenths of a step per max HP | 0–100 | `borg_stair_scum.rs:542` |
| `stair_scum_slack_per_level_percent` | 50 | percent of a step per level | 0–1000 | `borg_stair_scum.rs:544` |
| `stair_scum_alarm_turns` | 100 | game turns | 0–10000 | `borg_stair_scum.rs:905` |

**When.** In the dungeon, max HP below `stair_scum_max_hp` (the class's value)
plus `stair_scum_max_hp_race_delta` (the race's: the race suffix `__<race>`
is new with this task, docs/ZOO-INTERFACE.md), and no monster detection the
exploration uses (review of #1090): a Rod of Detection in the pack or the
Detection spell (`borg_light.rs` zaps and casts them with no level gate), or
Detect Monsters from `light_detect_evil_spell_min_level` on (the Mage's 10 in
the shipped `borg.txt`; or with `light_detect_dark_entry`). Reveal Monsters
(cast only against an invisible attacker), Detect Invisible and Detect Life do
not count.

**Safe grids** (`borg_stair_scum.rs`, `Map`). *s(g)* is the walk from *g* to
the nearest known staircase the Borg will take (up unless descent is forced;
down when prepared for the next depth and not rising: one rule for the area,
the walk back, the level change and the departure on a threat) over known
floor, no closed door (an open costs a turn and may fail) and no known trap
(below 60 HP the flows treat a known trap as a wall), 8-connected, through no
grid a monster stands on (so a monster in a shortcut leaves the open detour as
the walk); a staircase a monster stands on does not count;
an unknown grid or rubble counts one step past its nearest walkable
neighbour. *g* is safe when a monster first seen *D(g)* grids off survives
being outrun: `attacks_on_retreat` lets it act `stair_scum_speed_ratio` times
per player action, and `ratio - 1` times before the Borg's first action
(`game-world.c:1033-1067`: monsters with more energy act before the player);
each action closes a grid or attacks, each step opens one, and nothing acts
after the stair command. The walk is safe while those attacks times
`stair_scum_contact_damage` (per round of blows) stay below the current HP.
*D(g)* follows the engine's light whatever `cave_light_engine_radius` says:
`cur_light - 1` (`cave-view.c:658`), a Necromancer's unlight `1 + level / 6 -
cur_light` (`cave-view.c:777-779`); on a known lit (`BORG_GLOW`) floor grid,
one less than the Chebyshev distance to the nearest grid a monster can come
from unseen (unknown, dark or a door, next to the lit floor), when longer;
behind a closed door 1, and the open costs one action more. A Wooden Torch
(`LIGHT_2`) in a dark corridor at 12 HP against a +10 monster at 4 per attack:
1 step; a lit room grid 8 from its nearest dark entry: 7.

**The dial** (owner, 2026-11-23: "a tunable parameter to make it less
strict"): `stair_scum_slack_steps` lets a grid lie that many steps beyond the
strict safe walk (the survival test counts the walk that many steps shorter).
At 0 the count above; the torch at 12 HP: 1 step at 0, 2 at 1, 3 at 2 (the
owner's "two or three steps"). The zoo picks it (arms at 0 to 3). A rate
makes it grow with the character (owner, 2026-11-23: "a rate to make the scum
dependent on hp/class"): `stair_scum_slack_per_hp_tenths` adds a tenth of a
step per point of max HP (1: 12 HP one step, 24 HP two), per class and per
race like every parameter. And by level (owner: "slack grows by decile by level, say 50
means that druids at level 3 get to slack 2"):
`stair_scum_slack_per_level_percent` adds that percent of a step per character
level above 1, rounded down (50 with a Druid's slack 1: 1 at CL1, 2 at CL3, 3
at CL5). The three add up: steps + max HP × tenths / 10 + (CL − 1) × percent /
100. Its default is 50 (owner, 2026-11-23: "stair_scum_slack_per_level_percent:
50 as default"): with `stair_scum` 0 it does nothing, so play is unchanged;
with the mode on, the strict count of the review holds at CL1 and loosens by
a step every two levels.

**The stair side** (re-review of #1090): a grid is also unsafe when a monster
from the frontier nearest its staircase (an unknown grid or a closed door next
to the walk), acting `ratio - 1` times before the Borg and `ratio` times after
each of its steps but the last, reaches the staircase before the Borg does
(`cut_off`, timed on the real walk: the slack shortens only the damage
budget). The walk back steps over the same grids (`step_home`), never
through a closed door, and a committed departure walk resumes before the
return.

**The safe area** is every safe walkable grid joined to a staircase through
safe grids. The dark flows (`borg_flow_dark_1..5`) keep only the candidates in
it (unknown grids and doors next to it, safe themselves), nearest the
staircase first, and when the nearest lies farther out than the Borg it walks
back onto the staircase before taking it (no chaining outward along one
corridor); the item and monster flows (`take_select`, `scum_select`,
`borg_flow_kill`) skip targets outside it; `borg_flow_old` refuses any
voluntary step (flights excepted, resumed flows included) that leaves it
without coming nearer the staircase. After caution, a Borg outside it walks
back all the way onto a staircase.

**Doors.** A door it opens: one step back toward the staircase (or a turn's
wait, on it), then a turn's wait, then it goes on. A known closed door seen
open next to the area, which the Borg did not open: it walks onto a staircase
and stands there for `stair_scum_alarm_turns` game turns. "You hear a door
burst open!" names no grid (`mon-move.c:1269`): if no changed door is seen by
the next think, the same.

**Leaving.** Nothing left in the area (after the exploration flows): it takes
a staircase, on DL1 down first when prepared for DL2, deeper up first, down
only when prepared for the next depth. On a staircase it rests there (no walk
to a recovery grid), and a threat of `caution_stairs_leave_danger_percent` of
its HP that the fight race (`fight_before_flight`, before caution) did not
take is left by the staircase under it (`leave_stairs_on_monster`).

**No route.** No staircase known, or none the Borg can walk to over known
trap-free grids: the dark flows keep the candidates nearest a known staircase
as the crow flies (all of them when none is known), nothing is chased, no
step is refused, until a route exists (#1062, #1067).

**With breadth-first exploration** (#1080/#1085, `explore_breadth_first`,
shipped 1 on `track-p`): below its own threshold it replaces dark-1..5 with
`borg_flow_dark_breadth_first`, ring by ring from the staircases. With
`stair_scum` on too, that method gets the same map: its candidates must be in
the safe area (`Map::admits_target`), the return to the staircase between
targets applies (`return_before`), and `step_allowed` bounds its steps. The
two thresholds are separate (`explore_bfs_max_hp` and `stair_scum_max_hp`).

**Not done here:** the fight race on the damage quantile (#1077; the existing
race, on the mean, decides).

| `explore_bfs_food_percent` | 100 | percent | 100–10000 | `borg_flow_dark.rs:896` |
| `explore_bfs_light_percent` | 100 | percent | 100–10000 | `borg_flow_dark.rs:896` |

| `explore_bfs_hp_per_level` | 0 | hp per level | -100–100 | `borg_flow_dark.rs:882` |

## The holes in C's stair leash (#1062)

From the level-1 death review of track-p `206cf4e7` (borg-zoo run
45070528012; borg-cemetery#138). C's two stair leashes have three holes:
they count only up staircases, so a Borg that arrives on a down staircase
has no leash; with no staircase known or reachable there is no leash
(`borg_flow_cost_stair` returns 0, "go off leash"); and the exploration leash
measures the farthest up staircase (`borg_flow_dark`, C bug 31). A teleport
can also throw a CL1 Borg far from every staircase it knows. Six parameters
(the registry now holds **2475**, 2469 on track-p `91dd0d0a`), read in
`leash_holes` (`borg_flow.rs`) and used by `borg_leash.rs`. Every switch is
compiled 0, which is C's play. **Shipped on** in `harness/borg.txt` with the
values of the zoo arms (owner, 2026-11-23: "merge with the values to try"):
`flow_leash_any_stair = 1`, `flow_leash_unknown_stair_steps = 20`,
`flow_leash_teleported_recall = 1` (5, 30, 12). That is borg-zoo#719's
`leash3` arm, unmeasured (RULES 14), and smoke-borg-rs is re-recorded.
None acts in town, where C's leashes never act (town has no up staircase).

With a switch on, distances are **walking steps over known terrain**
(`terrain_steps`, a breadth-first search over the Borg's map, once a think
for each set of sources): no wall, rubble or unknown grid, and no monster,
trap or danger, which are what the Borg meets on the way, not the way. An
unknown target grid (the dark flow's) is one step past its nearest reached
neighbour. So a monster near a staircase, at the arrival grid or in the
corridor back does not hide it, and an unexplored gap is not a shortcut
(review and re-review of #1087).

- **`flow_leash_any_stair`**: the exploration leash (the far-from-stairs
  check of the dark, kill and take flows, take_select's own stair check and
  the vein flow's) takes the steps from the nearest known staircase of
  either kind, as do the early "far while near" filters of take_select and
  the kill flow. The return leash ("Return to Stair", 3 x CL + 14) starts on
  the steps to the nearest of them (a staircase across a wall is not near),
  ends ("Close enough", under 3) on them only for a return it started (the
  depth correction's climb ends on C's distance to an up staircase), starts
  only on a walk the stair flows can plan (at most 249 steps; one that
  becomes unwalkable is dropped), bounds breadth-first exploration (#1085)
  the same way, and its
  walk back flows to the nearest staircase of either kind
  (`borg_flow_stair_both`) before the up staircases.
- **`flow_leash_unknown_stair_steps`** = N: with no staircase the exploration
  leash counts, or none reachable, it is N steps from an anchor: the grid
  where the Borg arrived on the level while its area is connected to it, else
  the landing of a jump with no known route back that the area is connected
  to (at most 16, the newest always kept). The early "far while near"
  filters measure walking steps too. The return leash has no walk to that grid and is
  unchanged.
- **`flow_leash_teleported_recall`**: up to `flow_leash_recall_max_level`, a
  teleport, by its cause: a teleport trap (the message) or the Borg's own
  Teleportation scroll, staff or activation, Teleport Self, Portal or Shadow
  Shift, always; its own Phase Door or Dimension Door, never; a jump of
  unknown cause (a monster's teleport-to) from `flow_leash_recall_min_jump`
  grids (12: Phase Door asks for 9 to 11, `TELEPORT:10` randomised by a
  quarter in `effect_handler_TELEPORT`; the trap and the scroll are
  `TELEPORT:M80` and `:M60`), that leaves every known staircase more than
  `flow_leash_recall_steps` steps away (or none known) reads Word of Recall
  at the first dungeon think at danger 0, after the fights that own the
  action.

The four named seeds, replayed from birth on track-p `bc215cf8` (which ships
`explore_breadth_first = 1`, #1088) with this change (100,000 game turns,
local, a lead only, RULES 14; keys.log sha256 prefixes):

| seed | character | shipped | `any_stair` 1 | `unknown_stair_steps` 20 | `teleported_recall` 1 | all three |
|---|---|---|---|---|---|---|
| 592754909 | Kobold Mage | alive CL5 `5d2d6844` | alive CL5 `fbeaff39` | = shipped | = shipped | alive CL5 `fbeaff39` |
| 645909552 | Dwarf Mage | alive CL5 `0bc0bf21` | alive CL4 `e634cb27` | alive CL4 `6e3f27ac` | alive CL4 `ff452067` | alive CL5 `4acd96d2` |
| 526144893 | Human Ranger | alive CL4 `c1d0ce93` | alive CL5 `4ec45594` | alive CL5 `e3622e2f` | = shipped | alive CL5 `4ec45594` |
| 1024822556 | Dunadan Druid | alive CL6 `f3d29211` | alive CL6 `90c0127a` | alive CL6 `224787c1` | alive CL4 `c5311c09` | alive CL4 `16838cba` |

| parameter | default | unit | range | source |
|---|---|---|---|---|
| `flow_leash_any_stair` | 0 | flag | 0–1 | `borg_flow.rs:2398` |
| `flow_leash_unknown_stair_steps` | 0 | steps | 0–250 | `borg_flow.rs:2399` |
| `flow_leash_teleported_recall` | 0 | flag | 0–1 | `borg_flow.rs:2400` |
| `flow_leash_recall_max_level` | 5 | level | 0–51 | `borg_flow.rs:2401` |
| `flow_leash_recall_steps` | 30 | steps | 0–250 | `borg_flow.rs:2402` |
| `flow_leash_recall_min_jump` | 12 | grids | 2–250 | `borg_flow.rs:2403` |

## Mixed exploration from the staircases (#1080, #1098)

The owner (2026-11-23): "breadth-first, then mixed (fall back when they get
too far and explore another branch), then when they are on a level for which
they are fully prepared they can go depth-first"; "they do not get too far
from the stairs until they have covered everything that branches from them."
Breadth-first (#1085, above) is the first mode. At `explore_mixed = 1` the
second one, `borg_flow_dark_mixed`, applies to a Borg that breadth-first no
longer covers (its maximum HP at or above the class's threshold, or
`explore_breadth_first = 0`) and that is not yet prepared for its depth.
Both exploration calls (near and far) use it.

**Not prepared, for now.** #1080's readiness model is not written. Until it
is, the mixed walk applies where C's own exploring leash applies
(`c_leash_on`: the depth at least the character level less
`flow_leash_depth_margin`, below `flow_leash_stairs_max_level`, or the kit
short, #840). Elsewhere exploration stays C's depth-first order, as today.

**A cap by class** (owner, 2026-11-23: "explore_mix_max_clevel by class").
The walk applies only up to `explore_mix_max_clevel`; above it the class
counts as prepared. The per-class values are a hypothesis from #1080's pooled
table of DL1 deaths per 100 games that reached each CL: the level before the
first CL whose rate, and every later one, is at most 0.25. Priest and
Necromancer never get there within the table's CL1–7, so they take 7.
Compiled 50: no cap, and the walk ends with C's leash at CL20.

| class | DL1 deaths per 100 at CL3 / 4 / 5 / 6 / 7 | depth-first from | `explore_mix_max_clevel` |
|---|---|---:|---:|
| Warrior | 0.28 / 0.16 / 0.10 / 0.05 / 0.09 | CL4 | 3 |
| Mage | 2.18 / 1.51 / 0.45 / 0.26 / 0.16 | CL7 | 6 |
| Druid | 1.06 / 1.02 / 0.45 / 0.22 / 0.22 | CL6 | 5 |
| Priest | 1.61 / 0.70 / 0.31 / 0.29 / 0.46 | after CL7 | 7 |
| Necromancer | 2.17 / 0.45 / 0.95 / 0.37 / 0.59 | after CL7 | 7 |
| Paladin | 0.83 / 0.80 / 0.39 / 0.26 / 0.07 | CL7 | 6 |
| Rogue | 1.09 / 0.58 / 0.19 / 0.08 / 0.15 | CL5 | 4 |
| Ranger | 1.31 / 1.02 / 0.35 / 0.14 / 0.07 | CL6 | 5 |
| Blackguard | 0.67 / 0.63 / 0.11 / 0.14 / 0.23 | CL5 | 4 |

The table pools every build, and recent runs are 5–10 times lower, so the cut
at 0.25 is itself a hypothesis. borg-zoo#718's turns per (CL, DL) would turn
these into rates per turn.

**The walk.** It uses breadth-first's frame (`stair_frame`): each grid's ring
(flow steps from the nearest known staircase of either kind), the
interesting grids the Borg can reach, and the grids past the return leash.
1. **The radius.** *R* starts at `explore_mix_radius` and grows by
   `explore_mix_step`, up to a limit from the escape model of #1061:
   ⌊*D* / (*r* − 1)⌋ + ⌊max HP / *c*⌋ (`mix_limit`).
   - *D* = max(`explore_mix_sight`, the Borg's own light radius) is how far
     a monster is first seen.
   - *r* = `explore_mix_speed_ratio` is how many times as fast the depth's
     fast monsters are. It is 2 at DL1–5: Grip, Fang and cave spiders have
     `speed:120` (`monster.txt:1064,1083,1119`), 20 energy a game turn
     against 10 at normal speed (`game-world.c:68`). Such a monster first
     seen *D* grids away catches the Borg after it has walked
     *D* / (*r* − 1) grids toward the stairs.
   - *c* = `explore_mix_contact_damage` x the depth band (1 at DL1–5, 2 at
     DL6–10, ...) is the damage of one player turn in contact. The shipped 5
     is a 1d4 biter at speed 120, biting twice a turn (`monster.txt:1083,1119`).
     ⌊max HP / *c*⌋ is the turns of contact the Borg survives on the rest of
     the way.

   The limit is never below the start, never above `explore_mix_max_radius`,
   and never above C's leash (`borg_get_leash`) where that applies, so the
   walk never goes farther than today's. The shipped numbers give 8 steps at
   17 HP, 10 at 26 HP and 17 at 60 HP, against C's leash of 21 at CL4 and 27
   at CL6.
2. **Depth-first inside the radius** (`mixed_plan`). Every candidate within
   *R* is a goal of one spread, so the flow leads to the one nearest the Borg
   (C's choice). A path may not cross a ring beyond *R* or the Borg's own
   (`path_limit`), nor pass the return leash. A corridor is followed to its
   end while it stays inside *R*.
3. **Fall back and take another branch.** A branch whose frontier lies
   beyond *R* is parked: its grids are not goals, so the Borg walks back
   toward the stairs, to the nearest frontier still inside *R*.
4. **Grow only when the area is covered.** *R* is the smallest radius of the
   sequence that holds a candidate. It is not stored: each think takes it
   afresh, so a branch newly found near the stairs is covered before the
   walk goes deeper again. Larger radii follow as further tries, for goals
   the path limit cuts off (at most 16 tries a think; a radius whose goals
   and path limit repeat the last try's is skipped, review of #1117).
5. **When there is nothing more.** With no candidate within the limit, the
   walk is exhausted (`MixOutcome::Exhausted`, review of #1117), and so it
   is when there are tries but none gives a path (re-review). It marks the
   think, and `borg_leave_level` takes that as a reason to leave, like the
   level timer. The first leave call of the dungeon loop comes before the
   secret-door search and the direct monster flow. When the policy picked
   the direction at random (either allowed), a direction it cannot reach
   tries the other (re-review of #1117). If no staircase can be taken in
   any allowed direction (rising with no up staircase known, say), the walk
   stands down for that level and C explores for one
   (`release_after_failed_exit`).
6. **Edges.** With no route from the Borg to a known staircase (none known,
   a teleport, a known trap across the way back, #1067), or under boosted
   bravery, C's methods run, with C's leash (#1062). A staircase newly seen
   or a monster newly tracked drops a committed exploration flow, as with
   breadth-first (note `# New monster: planning the mixed exploration again
   (#1098)`).

The note is `# Flowing toward dark-mix (radius R of L) at cost n`. Not in
this change: #1098's point 6 (fight near the stairs or leave, and a Phase
Door that would land outside *R* or in never-seen grids is not an escape). It
is left to the fight race (#926), #1061 and the Phase Door switches of #1066.

The switch ships 1 (owner, 2026-11-23: "merge it but with all settings on"); its numbers are hypotheses (RULES 14). borg-zoo
measures `explore_mixed = 1` alone, with `explore_breadth_first = 1` (the
full ladder), and with two values of `explore_mix_contact_damage`. With this
section the registry holds **2489** parameters (`track-p` `a5cac1cd` plus
the ten below).

| parameter | default | unit | range | source |
|---|---:|---|---|---|
| `explore_bfs_leave_when_done` | 0 | flag | 0–1 | `borg_flow_dark.rs:1238` |
| `explore_c_leave_when_done` | 0 | flag | 0–1 | `borg_flow_dark.rs:1479` |
| `explore_mixed` | 0 | flag | 0–1 | `borg_flow_dark.rs:1291` |
| `explore_mix_radius` | 5 | steps | 1–250 | `borg_flow_dark.rs:1414` |
| `explore_mix_step` | 2 | steps | 1–250 | `borg_flow_dark.rs:1422` |
| `explore_mix_sight` | 5 | grids | 0–250 | `borg_flow_dark.rs:1417` |
| `explore_mix_speed_ratio` | 2 | ratio | 2–10 | `borg_flow_dark.rs:1413` |
| `explore_mix_contact_damage` | 5 | hp per turn | 1–1000 | `borg_flow_dark.rs:1421` |
| `explore_mix_max_radius` | 30 | steps | 1–250 | `borg_flow_dark.rs:1423` |
| `explore_mix_max_clevel` | 50 | level | 1–50 | `borg_flow_dark.rs:1295` |

## Committing to a brave stair walk (#1130)

The owner (2026-11-23), on a two-grid loop of the #1117 replays: "it must
definitely be fixed". Blackguard 312081895 (46 HP) stayed 60,000 turns
between two grids on a level whose staircases its ordinary flows could not
reach.
- **The stuck think.** It boosted bravery (`boost_bravery`: avoidance 92 for
  one think) and started a stair walk ("Goal_less, brave"), one step south.
- **The next think.** The avoidance was back at the HP. Continuing the walk
  failed: the shared flow buffer had been replaced by the think's other
  spreads, and the goal was cancelled. Exploration then took the step back
  north, to a target it could not step onto.

At `flow_brave_walk_commit = 1` (compiled 0, **shipped 1**), a stair walk
the brave think starts is committed (`Borg::brave_walk`):
- **Continuation.** `borg_flow_old` continues it with the avoidance it was
  spread with. When the old step does not go, it spreads the route again in
  the brave think's own order: any staircase while the return leash allows
  it (#1062), else an up one.
- **While:**
  - its last step is at most 3 thinks old;
  - it has steps left: the route's cost at the start, plus 10;
  - it is on its own level;
  - the danger here is at most half the HP. A monster that shows up is
    caution's.

Checked on the build where the loop reproduces (`433457a4`, with
`explore_c_leave_when_done = 1`):
- switch off: the loop, CL3 at 100,000 turns;
- switch on: the walk continued 159 times, "Spent too long" fell from 3,177
  to 11, levels visited rose from 76 to 180, and the game reached CL7 and
  DL3.

| parameter | default | unit | range | source |
|---|---:|---|---|---|
| `flow_brave_walk_commit` | 0 | flag | 0–1 | `borg_flow.rs:1838` |

