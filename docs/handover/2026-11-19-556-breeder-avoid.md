## 2026-11-19: kill breeders first; flee only what cannot be stopped (#556)

Owner, 2026-11-19, after the first design (avoid breeders whenever known):
"the first choice should be to kill them first and only escape if it is not
possible, the flight should be from places where the breeding cannot be
stopped any more", and to check the existing breeder settings first.

**The starting case** was Half-Elf Priest 592068863 (borg-cemetery#73), measured on #616's commit `8a427494`, where it reproduced with the dossier's keys. It died to giant white mice on DL1 at turn 86,224:
- a flight to stairs across the colony, undone each think by a back-away;
- then a Phase Door that landed it inside the colony.

The case **no longer replays** on any current build: main's shipped values changed in `e0c4a527`. With every switch off the Priest lives, so the change cannot show this death avoided.

**What the existing settings did against "kill first":**
- `caution_breeder_fight` (shipped 1) stands only with **no** easy escape. A Phase Door in the pack, or stairs within 5 grids, makes the Borg flee breeders. Four flee triggers act:
  - the breeder-level flee below `caution_breeder_flee_max_level` (15);
  - `borg_check_breeders`' "no recall" flee;
  - the scary list's breeder lines;
  - Recall as an escape (`caution_flee_recall_breeders`).
- `caution_breeder_hunt_max_count` (4): below CL10 the kill flow skips breeders once more than 4 are known.

**What was done:**
- **`caution_breeder_stoppable`** (compiled 0, shipped 0), decided once per think after the monster update (`note_breeders_stoppable`):
  - The test: the known breeders' expected births a turn, times `caution_breeder_stoppable_births_percent` (200), must stay below the Borg's kills a turn.
    - **Births:** the engine's 1 in 8k while k < 4 (`monster_turn_multiply`), weighed by each breeder's energy against the character's.
    - **Kills:** one per ceil(HP / damage) turns. The damage is the larger of melee, a missile from the bow (`best_launch_damage`), and the true damage of the attack the last ranking chose (`recent_best`), within `caution_breeder_stoppable_attack_turns` (10).
  - **While stoppable:** `breeder_stand` and `breeder_hunt` are true at any count and level, escape or not. The four flees stand down, and #503/#558's supporting settings follow: stand radius, boosts, no rest while hunting, the acid fear. Danger at or above the HP stops the stand for that think.
  - **Once not stoppable:** the level is lost to the breeders until a new level (`breeders_lost`, noted `# Breeders cannot be stopped …`).
  - The engine's cap (100) counts **live** breeders (`num_repro` drops when one dies), so a capped colony is not a finished one.
- **The flight** (`caution_breeder_avoid` and companions, from the first design) now blocks only while fleeing or the level is scary. A mark is put on every grid a breeder is seen on, so a moving breeder leaves a trail:
  - the flows go around the remembered colony;
  - `caution_breeder_avoid_fight_through` takes the route through it when there is no other;
  - `caution_breeder_through_turns` stands back-away down while fighting through;
  - `caution_breeder_stuck_turns` lifts the marks when the Borg is stuck;
  - a Phase Door or teleport landing near a mark counts as scary.

  A staircase inside a colony still draws a fleeing Borg in; that is the "fight our way out".
- **The review of `3e5fb21b`, addressed:**
  - the pure `breeder_blocks` predicate, with a test of the first-step exemption;
  - a switch-off test (`with_the_switches_off_nothing_changes`);
  - `fight_through` restores `hunting` (`mem::replace`);
  - the block buffer is reused;
  - the far-corner clip is tested;
  - the stale smoke paragraph is gone.

**Owner, 2026-11-19, on the Druid's death (borg-cemetery#77): "count all attacks".** The first kill rate (melee, then a bow, then the last ranked attack) was a hand-picked subset. It missed throws: the Druid judged 3 mice unstoppable on its whip while carrying 18 Flasks of oil. Now:
- the kill rate is the attack planner's own dry run (`borg_attack_estimate`, every attack the Borg has), with the random state restored after it;
- the planner counts the kills of each aim (`kill_acc`/`kill_best`: for each monster hit, damage over its HP, at most 1);
- a think is judged only with something to attack, and the level is lost only after 5 judged thinks in a row (`caution_breeder_stoppable_confirm`);
- on a lost level with no staircase known, the Borg fights on.

**Checked** on this head (`37c5697e`, merged with main `3ba17e2c`):
- CI's commands, `check-param-citations.py` (2178), `borg-core-vendor.py check`.
- Switch off: the 18 classes-18 starts play keys identical to main (18/18).
- classes-18 with it on (100,000 turns; local leads, RULES 14):

| arm | deaths | levels lost | breeder kills | sum max CL | median CL change |
|---|---:|---:|---:|---:|---:|
| main | 1 | – | 106 | 101 | – |
| stoppable | 3 | 3 | 99 | 88 | 0 |
| stoppable + flight avoidance | 2 | 3 | 134 | 92 | 0 |

- **The Druid (102023799) lives in both arms.**
- **"Lost" verdicts** are few and come with real trouble: 5 worm masses at 235 danger against 31 HP; 8 breeders at 1.54 births against 1.00 kills a turn.
- **The deaths:**
  - the rock lizard (in every arm);
  - the Half-Elf Ranger 1739161339: it shot a mouse where main fled, and Fang killed it 5,000 turns later in a diverged game;
  - in the stoppable arm only, the Human Necromancer 300036923: a wild cat, not a breeder, in a diverged game.
- **Levels:** the CL sum falls because two control games were unusually good (CL12 and CL8); the median change is 0.


**The 18 local starts** (`harness/run.sh … SEED RACE CLASS 100000 exact,menu`; the zoo's `classes-18` cycles races differently, so its numbers differ, e.g. a reviewer on all-Half-Elf starts counted 4 "levels lost" where this set has 3):

| seed | race | class |
|---|---|---|
| 1085222502 | Human | Warrior |
| 1548306106 | Human | Mage |
| 102023799 | Human | Druid |
| 1569794094 | Human | Priest |
| 300036923 | Human | Necromancer |
| 766958386 | Human | Paladin |
| 1502695781 | Human | Rogue |
| 226104090 | Human | Ranger |
| 1472012981 | Human | Blackguard |
| 1019921749 | Half-Elf | Warrior |
| 801785300 | Half-Elf | Mage |
| 934061548 | Half-Elf | Druid |
| 592068863 | Half-Elf | Priest |
| 1574344917 | Half-Elf | Necromancer |
| 661409606 | Half-Elf | Paladin |
| 1381972077 | Half-Elf | Rogue |
| 1739161339 | Half-Elf | Ranger |
| 888896726 | Half-Elf | Blackguard |

**For the owner:**
- the zoo pilot (borg-zoo#411, re-pinned to this head);
- the births weight (200%) and the confirmation (5 thinks);
- whether to ship.

Left: the planner checks the mana for the next cast, not for a whole fight against the colony (#625's PR #657 does that for the Phase Door rule).
