## 2026-11-19: an unseen attacker: walk away, rest elsewhere, heal (#697)

Owner: "it leaves if it senses an invisible thing and goes rest somewhere else, and use healing potions". The default core's version of #462 (reborn core).

- **Found** in #691's measurement: a CL3 Rogue (1502695781, `kill_flow_mode` 3) rested through 23 "Something touches you" and died to a clear icky thing with 2 Cure Light Wounds.
  - `borg_locate_kill` makes no monster for "Something", and records "Someone" as a player ghost, so the danger stays about 0.
  - Setting 0 of #695 survives only by chance: it bumps the invisible monster while picking up an item.
- **Change** (`borg_recover.rs`, shipped 0):
  - `note_unseen_attack`, called from `borg_update.rs`'s `HIT_BY`/`MISS_BY`, records the game turn and grid (fields in `borg.rs`, reset on a new level);
  - `recover_unseen_attack_turns`: no rest within `recover_unseen_attack_distance` (10) of the attack. Hurt, the Borg walks away (`borg_caution::walk_away_from`, a corridor first, as #544) and rests there. If the attacker followed, or there is nowhere to walk, it leaves the level;
  - `recover_unseen_heal_hp_percent`: drink a cure below that share of HP while the attack is recent;
  - unit tests in `unseen_tests`.
- **Measured** (CALIBRATION, #697):
  - the case: lives (walks 10 steps away, no further touch); the heal fires when set;
  - classes-18: off is identical to main 19/19 and on smoke; on: 0 deaths both, sum of max CL 108 to 111, max DL 40 to 43, no loops.
- **For the owner:** values to ship, after a zoo pilot. The heal was not exercised at 50% on classes-18.
- **Not covered:** an invisible monster that casts ("Something mumbles"); the record only reads blows (`HIT_BY`/`MISS_BY`).
- **Review of #698** (fixed): "Someone" (a male or female monster the Borg cannot see: 15 invisible gendered races, or any of the 186 gendered ones out of sight) counts, "It" (never a blow's name) does not; the walk away comes before the heal; a held rest is noted; the walk stays off unknown grids; the last Cure Critical Wounds is drunk (plain quaff); "followed" is a tested state that stays set for the window. Re-measured on main `dbc92d18`: off identical 19/19; on, sum of max CL 111 to 116, deaths 1 and 1 (the same death). The Rogue case no longer dies on current main even with the switches off.
