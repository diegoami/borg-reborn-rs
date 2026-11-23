# Phase Door: where the Borg decides it

*#584. A map of every place where the vendored core (`borg-core/`, the
borg-rs core) reads, casts or activates Phase Door, what makes each one fire,
and which settings it reads. Gameplay evidence about these decisions belongs
in borg-cemetery; this page describes the code.*

## One decision point

Every site calls `phase_door(st, site, check, danger, cast)`
(`borg-core/src/borg_escape.rs`):

- **`site`** (`PhaseSite`) names the place. It appears in every note.
- **`check`** (`PhaseCheck`) is the test the site always ran before phasing
  (kept as it was, so the refactor plays the same keys):
  - `Landing { tolerance, horizon }`, the old `borg_caution_phase`: the Borg
    has a Phase Door (`BI_APHASE`); #539's "a monster next to me"
    (`phase_is_pointless`, with `escape_phase_adjacent_only`); and the landing
    test below.
  - `MonsterNear`: only #539's test.
  - `ShootScoot`: `borg_shoot_scoot_safe` (missiles or mana to shoot with, a
    landing test of its own).
  - `None`: no test at all.
- **`cast`** reads, casts or activates in the site's own order. Some sites'
  lists also hold a teleport, as C's did.

The notes in `messages.log`:

| note | meaning |
|---|---|
| `# Phase (<site>): no, no monster next to me` | #539's test refused it |
| `# Phase (<site>): no, teleport is blocked` | a teleport curse (known from the item, or learned from "Teleportation forbidden!", #829) refused it before the site's own check |
| `# Phase (<site>): no, N scary landing grids of 100 (at most T)` | the landing test refused it |
| `# Phase (<site>): landing ok, N scary landing grids of 100 (at most T)` | the landing test passed |
| `# Phase (<site>): done, HP h/H, danger d, nearest awake monster n` | the Borg phased (or used the site's teleport) |

**A teleport curse stops it first (#829).** `phase_door` asks
`teleport_stopped` (which asks `teleport_curse` what the curse is) before
anything else, so no site can skip it. The whole check is behind
`escape_teleport_block` (compiled 0 = as before, shipped 1;
docs/CALIBRATION.md): with it at 0 neither the learning from messages nor
this asking happens, and only the old `BI_CRSNOTEL` check at the sites that
ask `borg_allow_teleport` remains. The curse is
known from `BI_CRSNOTEL` (an item whose anti-teleportation curse the Borg
reads), or learned from the engine's messages: "Your X prevents you
teleporting." and "Teleportation forbidden!" (`effect_handler_TELEPORT`,
`_TELEPORT_TO` and `_TELEPORT_LEVEL`, `OF_NO_TELEPORT`). It lasts until the
item it was learned on is no longer worn, or "The anti-teleportation curse is
removed!". A no-teleport *grid* is not asked here: Phase Door (distance 10)
is not stopped by one (`dis > 10` in `effect_handler_TELEPORT`). The same
test sits in `borg_read_scroll` (Phase Door, Teleportation, Teleport Level),
`borg_use_staff*` (Teleportation), `borg_activate_item` (the teleport
activations) and `borg_spell` (Phase Door, Teleport Self, Teleport Level,
Portal, Shadow Shift, Dimension Door), so a site that does not go through
`phase_door` cannot teleport either.

The old notes (`# Safe to Phase. scary squares`, `# No Phase …`) are gone;
the `# Danger Level N` notes after an escape are unchanged.

## Words used below

- **Danger** (`b_q` in `borg_escape`): the Borg's estimate of the damage it can
  take where it stands (`borg_caution`'s `pos_danger`, `borg_danger` over
  `caution_danger_horizon` turns). C's comment on `borg_escape` says "the
  least dangerous square around us", but C passes the same `pos_danger`.
- **Avoidance**: the danger the Borg accepts. It is the current HP, unless the
  Borg raised it to get unstuck (`boost_bravery`). A threshold written as
  `tenths` is `avoidance × tenths / 10`: 10 means "danger above the HP", 5
  "above half the HP".
- **Landing test** (`count_scary`): 100 random grids 5 to 10 steps away (where
  a Phase Door lands). A grid is *scary* if it is unknown (only while max HP is
  below `escape_unknown_landing_max_hp`, 30), if no floor was found after 100
  tries, or if the danger there over `horizon` turns is above the current HP.
  The Borg phases only if at most `tolerance` of the 100 are scary.
- **Anti-summon delay** (`escape_phase_antisummon_delay`, 50 turns): no Phase
  Door for 50 turns after the Borg finished a digging flow (`GOAL_DIGGING`,
  `borg_flow.rs`), at the seven sites marked "anti-summon" below. The timer
  is named for the anti-summon corridor (`borg_flow_kill_corridor`), but a
  tunnel dug straight to a monster (`borg_flow_kill_direct`, e.g. #558's
  breeder hunt) sets it too.

## The sites

`borg_escape(st, danger)` is tried from `borg_caution` after retreat and
back-away. Its levels run in order; each fires on its own condition, tries
teleports and/or a Phase Door, and returns at the first that acts. The
settings' values are the compiled defaults; `harness/borg.txt` ships them
unchanged except `escape_phase_adjacent_only = 1`.

| site (note) | fires when | check | settings |
|---|---|---|---|
| **escape 1 emergency** | danger > 1.5 × avoidance with no unique near (1.7 with one; special Morgoth/Sauron/deep-unique cases), after every teleport failed | landing, at most 75 of 100 | `escape_ordinary_danger_tenths` 15, `escape_unique_danger_tenths` 17, `escape_emergency_phase_tolerance` 75, `_horizon` 2 |
| **escape 1.2 critical** | in the dungeon, HP below 1/10 of max or danger > 4.5 × avoidance | monster near only | `escape_critical_phase_hp_divisor` 10, `escape_emergency_danger_tenths` 45 |
| **escape 1.3 last ditch** | below CL 10, HP below 1/10 of max (spell only) | monster near only | `escape_last_ditch_phase_max_level` 10, `_hp_divisor` 10, `_fail` 15 |
| **escape 2.2 after teleport** | danger > 1.1 × avoidance (1.3 with a unique; a low-mana Mage; heavy stun), teleports failed | landing, 50, anti-summon | `escape_teleport_ordinary_danger_tenths` 11, `_unique_` 13, `escape_teleport_phase_tolerance` 50, `_horizon` 2 |
| **escape 3.1 danger** | danger > 1.0 × avoidance with no unique (1.3 with 2–8 uniques; an afraid Warrior without missiles; heavy stun) | landing, 25, anti-summon | `escape_phase_ordinary_danger_tenths` 10, `escape_phase_unique_danger_tenths` 13, `escape_phase_unique_min_count` 2, `escape_phase_tolerance` 25, `escape_phase_horizon` 2, `escape_phase_antisummon_delay` 50 |
| **escape 3.3 danger fallback** | level 3, its Phase Door and teleports failed | landing, 75, anti-summon | `escape_fallback_phase_tolerance` 75, `_horizon` 2 |
| **escape 4.1 compromised** | danger > 0.8 × avoidance below CL 35 (or HP below 1/3), > 0.9 with a unique, **> 0.6 at CL ≤ 20 with no unique**, > 0.6 at CL ≤ 35 | landing, 20, anti-summon | `escape_compromised_danger_tenths` 8, `_max_level` 35, `_hp_divisor` 3, `_unique_danger_tenths` 9, `_ordinary_danger_tenths` 6, `_ordinary_max_level` 20, `_general_danger_tenths` 6, `_general_max_level` 35, `escape_compromised_phase_tolerance` 20 |
| **escape 4.3 weak caster** | a Mage or Necromancer up to CL 35, level 4's teleports failed | landing, 65, anti-summon | `escape_weak_caster_max_level` 35, `_phase_tolerance` 65 |
| **escape 5.1 low level** | **below CL 10 and danger > 0.5 × avoidance** (0.7 with a unique) | landing, 20 | `escape_low_level_max_level` 10, `_danger_tenths` 5, `_unique_danger_tenths` 7, `_phase_tolerance` 20, `_horizon` 2 |
| **escape 5.3 low caster** | a Mage or Necromancer up to CL 8, level 5's teleports failed | landing, 65 | `escape_low_caster_max_level` 8, `_phase_tolerance` 65 |
| **escape 6.1 low mana** | a Mage, Priest or Necromancer with SP ≤ 1/10, danger > 0.6 × avoidance | landing, 20, anti-summon | `escape_low_mana_danger_tenths` 6, `_sp_divisor` 10, `_phase_tolerance` 20 |
| **escape 7.1 shoot n scoot** | the Phase Door spell is castable and Shoot N Scoot is safe: phase away from a monster that came close, then shoot it | Shoot N Scoot | `escape_scoot_tolerance` 20, `_horizon` 2 |
| **escape 8 twitch** | the Borg twitched (got stuck) more than 50 times | landing, 20, anti-summon | `escape_twitch_limit` 50, `_phase_tolerance` 20 |
| **recall wait** (`borg_caution`) | Word of Recall is counting down and the Borg buys time | landing, 50 | `caution_recall_phase_tolerance` 50, `_horizon` 1, `_phase_fail`, `_portal_fail` |
| **twitchy move** (`borg_flow_misc::borg_twitchy`) | stuck, before moving at random | landing, 15 (fixed) | none |
| **twitchy think** (`borg_think_dungeon`) | stuck, up to 3 times a level, before boosting bravery | **none** (as in C) | none |
| **fast-fragile** (`borg_fast_fragile.rs`, #842) | `fast_fragile_policy = 1`: a marked monster, the expected damage of the marked monsters within two grids reaches `fast_fragile_escape_percent` of the HP and the kill needs more than one more turn of the best attack, no staircase underfoot; the owner's rule above also applies (an awake enemy adjacent, the fight lost) | landing at most 75 of 100 only with `fast_fragile_phase_landing = 1` (shipped 0: none; it refuses every Phase Door at HP 3–6 against a crow), anti-summon | `fast_fragile_phase_landing`, `escape_emergency_phase_tolerance` 75, `_horizon` 2, `escape_phase_antisummon_delay`, `escape_last_ditch_phase_fail` (the spell), `fast_fragile_escape_percent` |
| **unseen bump** (`borg_caution`) | confirmed adjacent unseen occupancy, after detection and the oil budget; after ordinary healing/defence/Recall, before stair walking | emergency landing test; bump supplies adjacency; counts as an escape | `caution_unseen_bump_turns`, `_response`, `_throw_tries`, `_phase_turns` and `_phase_min_danger` (#878), `escape_emergency_phase_tolerance`, `_horizon` |

Spell fail rates: `escape_normal_fail` 25%, `escape_healthy_fail` 10% when HP
is above `escape_healthy_hp_percent` (70%); the emergency levels have their
own.

## What this shows

Counts from the 18 local classes-18 games of #602's check (borg-rs core,
100,000 turns, shipped `borg.txt`, main `dbe99509`; per site in #602's PR
body and `docs/handover/2026-11-18-584-phase-decision.md`): 150 Phase Doors.

- **Each site decides from a danger threshold alone.** None compares Phase
  Door with attacking or with boosting.
- **Levels 4.1 and 5.1 phase below the HP:** at 0.6 × and 0.5 × avoidance.
  They made 71 of the 150 (40 and 31), often at full HP.
- **The six `escape_phase_*` settings are two kinds.** Five
  (`escape_phase_ordinary_danger_tenths`, `_unique_danger_tenths`,
  `_unique_min_count`, `_tolerance`, `_horizon`) gate only level 3.1 (1 of
  the 150). The sixth, `escape_phase_antisummon_delay`, is a shared timer
  read at seven sites (2.2, 3.1, 3.3, 4.1, 4.3, 6.1, 8), which made 56 of
  the 150; it only blocks them for 50 turns after a digging flow.
- **Twitchy think has no check** (25 of the 150), so #539's "a monster next
  to me" does not apply to it.

## The owner's rule (#584, `escape_phase_adjacent_rule`)

Owner, 2026-11-18: "to use Phase Door, enemies must be on the square next to
you"; otherwise, if the Borg cannot escape, attack with magic or missiles,
and if that is not possible, boost (Speed included); only in close quarters,
when the Borg thinks it cannot win, Phase Door.

With `escape_phase_adjacent_rule = 1` (compiled 0, shipped 1 since #661), `phase_door`
asks one rule at every site above, before the site's own check, if the Borg
can Phase Door at all (`BI_APHASE`):

| situation | verdict | at the site | after `borg_escape` found no escape |
|---|---|---|---|
| no awake enemy on an adjacent square | no Phase Door | the site goes on with its other escapes (stairs, teleports) | its best attack (`borg_attack`, the Borg's own ranking; with nothing adjacent only a ranged attack can win it); if none and an awake monster is in view, a boost (`cornered_boost`: Speed first, then Heroism, Berserk Strength, Bless; else `borg_defend`: resistances, Protection from Evil, shields, staves) |
| an enemy adjacent, the fight not lost | no Phase Door | the same | counter-attack (`borg_attack`), else the boost |
| an enemy adjacent, the fight lost | Phase Door allowed | the site's own check (landing test, …) and cast, as before | – |

"Lost" is chosen by `escape_phase_lost_test` (owner, 2026-11-19: both
comparisons behind a setting):

| value | the adjacent fight is lost when |
|---|---|
| 0 (shipped, #661) | the danger where the Borg stands is at or above `escape_phase_lost_danger_percent` (100) % of the current HP |
| **1** (compiled) | the fight compared: killing the adjacent awake monsters one after another (their HP left: `kill.power` less `kill.injury` %) takes more turns than the danger where it stands needs to kill the Borg. Its damage a turn is the larger of its melee against that monster (`borg_thrust_damage_one`: blows, slays, the chance to hit) and the true damage of the attack it would choose now, of any kind (`borg_attack_estimate`, a dry run of `borg_attack`: a Mage casts, an archer shoots, even next to an enemy). True damage: without the kill bonus, the target's danger and the unique targeting bonus that `borg_attack`'s ranking adds. |
| 2 | as 1, and the expected landing is safer: the mean one-turn danger of the sampled landing grids (`landing_danger`, drawn as the landing test draws them) is below the danger here |

**Danger without fear** (`escape_phase_lost_damage_only`, compiled and
shipped 0; #878). With 1, every test above reads the danger here without its
fear prices (`borg_danger_damage_only`: the `danger_*_fear` terms, the
breeder and friends bonuses and the fear maps at 0), and the site's own
danger is not used. One white worm mass next to the CL3 Mage 461220874 was
danger 32 with fear and 2 without.

**Mana and fail rates** (`escape_phase_lost_mana`, compiled 1; #625, owner).
With tests 1 and 2, the chosen attack counts at its true damage times its
success chance, and only for the uses the Borg can pay for:
- **Success chance:** 1 − the fail chance. That is a spell's fail rate, or a
  wand's or rod's `borg_activate_failure` / 10.
- **Uses:** a spell gets current SP / its mana. **All** the mana counts:
  owner, 2026-11-19, "at the beginning we have no choice, and later on we
  will get potions to recover it". So the class's escape reserve is not set
  aside. The reserve cast (`borg_attack_aux_spell_bolt_reserve`, which fakes
  full mana) counts the real mana. A wand gets its charges. A rod gets one
  use, since it recharges over many turns. Staves and activations have no
  limit.
- **After the uses run out:** the larger of melee against that monster and
  the best true damage of another attack that needs no mana (never the
  chosen attack itself) (`fight_turns`).

Melee damage is an expected damage (`borg_thrust_damage_one` applies the
Borg's hit estimate), while a missile's is nominal (`borg_launch_damage_one`
only refuses an arrow below its minimum hit chance). The two are compared as
they are; changing the missile figure would change the attack ranking
itself. With 0, the chosen attack counts at full damage every turn (#612).

Tests 1 and 2 note their numbers (`# Phase rule: the fight: <HP left> HP at
<damage> a turn, …; danger d, HP h: lost / not lost`, `… the landing: …`;
with `escape_phase_lost_mana`: `<HP left> HP (melee m), …; attack a at f% fail
(n uses), then b; danger d, HP h: lost / not lost`).
Test 2 draws random numbers (the landing samples). The rule is decided and noted once per think, at the first site
that asks (`# Phase rule (from <site>): …`); later sites reuse it. Its attack
or boost (`phase_rule_act`) runs once, in `borg_caution` right after
`borg_escape`, so after the emergency levels' borrowed mana is put back.

Shoot N Scoot's own check (`borg_shoot_scoot_safe`, which needs a monster
next to the Borg) answers before the rule there, so the rule is not asked
on its turns with nothing adjacent.

Two consequences of the order, with the rule on:
- `phase_rule_act` comes before `borg_caution`'s cures (fear, confusion)
  and later steps: the Borg attacks before it cures. Melee refuses while
  afraid, but a confused Borg may still swing; the boost cures fear first.
- The tests draw from the Borg's own random numbers (the attack simulation
  of `borg_attack_estimate`; test 2's landing samples; never the game's), so
  the attack the Borg then makes is drawn from a later point of that stream
  than without the rule. Two runs with the rule on play the same keys.

With the rule on, #539's `escape_phase_adjacent_only` check
(`phase_is_pointless`) is skipped at every site: the rule decides (the
owner's spec: it must not rely on #539).

Outside `borg_escape` (the Recall wait, the twitchy paths) the rule only
refuses the Phase Door; the Borg's ordinary flow fights later in the think.
With no enemy adjacent the Borg no longer phases to get unstuck.

Whether the rule saves or costs games is the zoo pilot's question
(borg-zoo#334, RULES 14): the 100 % threshold is a hypothesis.


#781 adds the off-by-default `unseen bump` site in the table. It bypasses
known-race fight and adjacency tests, since the observed bump supplies the
adjacent occupied grid. It retains the emergency landing test and counts
successful phases toward the level's ordinary escape accounting. Both
`caution_unseen_bump_turns` and `_response` must be enabled.
