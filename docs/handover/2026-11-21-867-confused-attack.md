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
