## 2026-11-21: Confused attacks by direction, shipped 0 (#867)

**Done.** One new setting, `attack_confused_by_direction` (flag, compiled
and shipped 0 = C), in `borg-core/src/borg_fight_attack.rs`; its row and
description are in `docs/CALIBRATION.md` ("Confused attacks by direction
(#867)") and `harness/borg.txt`.

From the owner's play of borg-zoo#581's Gnome Necromancer 459443830 (oil
thrown by direction while confused killed the grey mushroom patch), then:
"attack must be dependent by class ... choose the best attack available",
"if it has a wand/rod that must be an option too".

- **The engine.** Confused, the player may not cast (`player_can_cast`,
  `player-util.c:1102`) or read (`:1182`), but melee, missiles, throws and
  aimed devices all pass through `player_confuse_dir`
  (`player-util.c:1353-1377`): a direction key holds 11/32 of the time; a
  target (direction 5) is always redrawn. A staff needs no aim.
- **C** blocks every item attack while confused (`cant_aim`), and aims at a
  target (`5`) when it does attack.
- **On (1)**, confused but not blind or hallucinating: throws, missiles, wands
  and rods rate only an adjacent monster, aimed at it, and are sent by
  direction key (`aim_key`; note `# Confused: aiming by direction N, not at a
  target (#867)`); every aimed attack, melee included, is weighted 11/32
  (`confused_weight`, in `borg_attack` and the focused ranking), so the usual
  chooser picks on damage; a staff keeps its full value. `borg_attack_estimate`
  sees the weighted damage.
- **The cure waits** (owner, after the first named cases: "I threw flasks at
  the patch immediately"). The cures run in `borg_caution`, before
  `borg_attack`, so a confused Borg with a cure drank it first (a CSW in C)
  and only attacked after. On, next to an immobile monster whose blow
  confuses (`immobile_confuser_next_to`: `CONFUSE` among its blows),
  `confusion_cure_held` holds the cure at both sites (note `# Next to an
  immobile confuser: attacking before the cure (#867)`), unless the moving
  monsters near are as dangerous as the HP left. A heal for HP still runs.

**Checked.** CI's Rust and Python checks; `check-param-citations.py`; unit
tests `confused_attack_tests` (the 11/32 counted over the engine's draws, the
gate, the key sent, the weight); smoke on the default core **3/3 identical**
to `smoke-borg-rs` at the shipped 0. Named cases on this branch, 100,000
turns, `menu,exact`:

| start | settings | switch 0 | switch 1 |
|---|---|---|---|
| Gnome Necromancer 459443830 | shipped | lives (CSW first, Phase Door) | lives: throws from turn 5,181, the first confused turn, no cure first; 5 throws all astray (about 12%: (21/32)^5), HP heals at 3/12 and 6/12 clear the confusion, then Phase Door and the stairs |
| Gnome Necromancer 459443830 | + `caution_confused_clw_cure` 1 (csw's death) | **dies, turn 5,297** (as in the zoo) | **lives**: the same game as shipped with the switch on through the patch fight (the CLW is held as a confusion cure there); they part at turn 33,930, where the CLW cures a trap's confusion |
| Elf Necromancer 412975293 | shipped / + CLW cure | lives | lives (never confused next to a monster) |

(An earlier commit of this branch, without the cure hold, drank the CSW and,
with the CLW switch, all six CLW before its first throw; the oil then killed
the patch on the 3rd/4th throw.)

**Found.** Unconfused again after an HP heal, the Borg reads Phase Door and
leaves the level rather than throw oil, now at full aim, at a 2-HP patch: the
Phase rule's "fight looks lost" fires on the patch's danger (408 against 12
HP at CL2). A lead for the danger of confusing immobile monsters, not changed
here.

**Not done / left.**
- `immobile_confuser_next_to` reads the engine's blow table, so it has no
  unit test; the named case shows it.
- The unit tests are on the pure parts; the scenario tests #867 lists (a
  confused Mage throws, a Warrior melees, a wand or staff is chosen) need an
  engine-backed state, which these tests do not have. The named cases cover
  the Necromancer and oil only.
- `ranged_means` (the caution's "has a ranged attack") still says no while
  confused; activations, rings and dragon armour were never blocked by C and
  still aim at a target.
- Targets on one of the 8 lines at range (a direction reaches them too).
- **Measurement:** a borg-zoo arm, switch 1 against shipped (and with the
  CLW cure), per class: attacks chosen while confused, deaths by cause.
  Nothing here is claimed beyond the named cases (RULES 14).

**Local note.** `~/projects/borg-cemetery` was not at `death-tools-pin.txt`
(`PROVENANCE.md` differs), so smoke and `tools/tests` refused to run until
`BORG_CEMETERY_DIR` pointed at a checkout of the pin.

## Review fixes (review of #868, findings 1 and 2)

- **The cure hold needs an attack** (`confusion_cure_held`,
  `confused_attack_open`): the hold applies only while `borg_attack_estimate`
  (random state and waiting flag restored) has a winner with damage or
  kills. Afraid (no blow) and hallucinating (no item attack), as next to a
  magic mushroom patch (`SCARE`, `SPORE:HALLU`), the cure is no longer held.
  The kills count as well as the damage: the throw at a 2-HP patch has its
  damage capped at 2, and the 11/32 weight truncates that to 0
  (`confused_weight`'s integer `dmg_best`; a damage-only test held nothing in
  the named case). `immobile_confuser_next_to` skips uniques (Wiruin, the
  Maelstrom, `ENGULF:CONFUSE:3d6`), whose own blows `mobile_danger` leaves out.
- **No stale confused aim**: `confused_aim`/`confused_dir` are cleared after
  the attack is sent (`borg_attack`'s instantiation, `perform_focused_attack`),
  and `borg_launch_bolt` reads the flag only while `confused_by_direction`
  holds, so Teleport Other (`borg_fight_defend.rs`) on a later, unconfused
  think is never limited to adjacent monsters or left without its target.
- **Checked**: CI's Rust checks, `check-param-citations.py` (citations moved),
  `borg-core-vendor.py check`; smoke on the default core 3/3 identical to
  `smoke-borg-rs`; Gnome Necromancer 459443830 at 100,000 turns, switch 1
  and switch 1 + CLW cure: both live, the throw comes at turn 5,181 with no
  cure first, and the two runs part at turn 33,930, as before the fix.
- **Left**: the integer truncation of the weighted `dmg_best` (2 → 0) also
  reaches the fight-or-flee estimate and the no-mana fallback; not changed.

## Review fixes (review of #868, findings 3 to 5)

- **Fight-or-flee keeps C's blow** (finding 3): in `borg_attack`,
  `confused_weight` weights a confused blow's value (the chooser still
  compares it with the item attacks at 11/32) but not its `dmg_best` and
  `kill_best`, which `borg_attack_estimate` hands the fight-or-flee tests;
  C rated a confused blow at full damage, so a confused fight that melee
  wins is judged as in C. The item attacks, new to confused play, report
  weighted damage. The focused ranking ranks on damage, so it still weights
  every aimed attack's damage. A hypothesis either way until the zoo arm.
- **The stairs walk asks only the step hold** (finding 4):
  `confusion_cure_held` is split into `cure_held_for_attack` (#867) and
  `cure_held_for_step` (#646, #837); `desperate_for_stairs` asks only the
  second, so the #867 hold no longer lets a confused Borg start a walk
  that would replace the attack.
- **One note per cure site, with its reason** (finding 5):
  `confusion_cure_hold` returns the reason and the two cure sites note it
  once each (`# Not fixing Confusion next to an immobile confuser:
  attacking first (#867[, cures])`, or the #646 notes, unchanged). The hold
  itself notes nothing.
- **Checked**: as for findings 1 and 2, the same commands; one new unit test
  (`a_blows_damage_stays_cs_for_fight_or_flee`).
