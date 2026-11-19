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
| `# Phase (<site>): no, N scary landing grids of 100 (at most T)` | the landing test refused it |
| `# Phase (<site>): landing ok, N scary landing grids of 100 (at most T)` | the landing test passed |
| `# Phase (<site>): done, HP h/H, danger d, nearest awake monster n` | the Borg phased (or used the site's teleport) |

The old notes (`# Safe to Phase. scary squares`, `# No Phase …`) are gone;
the `# Danger Level N` notes after an escape are unchanged.

## Words used below

- **Danger** (`b_q` in `borg_escape`): the Borg's estimate of the damage it can
  take in the next turn on the least dangerous grid around it
  (`borg_danger`).
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

Spell fail rates: `escape_normal_fail` 25%, `escape_healthy_fail` 10% when HP
is above `escape_healthy_hp_percent` (70%); the emergency levels have their
own.

## What this shows

- **Each site decides from a danger threshold alone.** None compares Phase
  Door with attacking or with boosting.
- **Levels 4.1 and 5.1 phase below the HP:** at 0.6 × and 0.5 × avoidance.
  In 18 local classes-18 games (#584's investigation, main `8da9be99`) they
  made 67 of 171 Phase Doors, at a median danger of 0.6–0.7 × the HP, often
  at full HP.
- **The six `escape_phase_*` settings are two kinds.** Five
  (`escape_phase_ordinary_danger_tenths`, `_unique_danger_tenths`,
  `_unique_min_count`, `_tolerance`, `_horizon`) gate only level 3.1 (3 of
  those 171). The sixth, `escape_phase_antisummon_delay`, is a shared timer
  read at seven sites (2.2, 3.1, 3.3, 4.1, 4.3, 6.1, 8), which made 56 of
  the 150 Phase Doors in the 18 games of #602's check; it only blocks them
  for 50 turns after a digging flow.
- **Twitchy think has no check**, so #539's "a monster next to me" does not
  apply to it.

## The owner's rule (#584, `escape_phase_adjacent_rule`)

Owner, 2026-11-18: "to use Phase Door, enemies must be on the square next to
you"; otherwise attack with magic or missiles, and if that is not possible,
boost yourself (Speed included); only in close quarters, when the Borg
thinks it cannot win, Phase Door.

With `escape_phase_adjacent_rule = 1` (compiled 0, shipped 0), `phase_door`
applies one rule at every site above, before the site's own check:

| situation | verdict | what the Borg does |
|---|---|---|
| no awake enemy on an adjacent square | **no Phase Door** | its best attack (`borg_attack`, the Borg's own ranking; with nothing adjacent only a ranged attack can win it: missiles, throws, wands, rods, spells, activations); if it has none and an awake monster is in view, a boost (`cornered_boost`: Speed first, then Heroism, Berserk Strength, Bless, Monster Confusion against a big monster); else nothing here |
| an enemy adjacent, the fight not lost | **no Phase Door** | counter-attack (`borg_attack`), else the boost |
| an enemy adjacent, the fight lost | **Phase Door allowed** | the site's own check (landing test, …) and cast follow as before |

"Lost" is `danger ≥ escape_phase_lost_danger_percent` (100) % of the
current HP: the Borg could die this turn. The danger is the one the site
measured (in `borg_escape`, the least dangerous grid around the Borg;
elsewhere the danger where it stands). Notes: `# Phase (<site>): no, no
enemy on an adjacent square: attack at range or boost`, `…: no, enemy
adjacent but the fight is not lost (danger d, HP h): fight`, `…: enemy
adjacent, the fight looks lost (…): phase allowed`.

The stuck (twitchy) phases follow the rule too: with no enemy adjacent the
Borg no longer phases to get unstuck.

Whether the rule saves or costs games is the zoo pilot's question
(RULES 14): the threshold of 100 % is a hypothesis.
