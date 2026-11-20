## 2026-11-20: immobile creatures kept at range (#713)

Paladin 766958386 (borg-cemetery#79) meleed a yellow jelly for 120 game turns, with 24 flasks unused, and died of its poison.

- **Analysis** (on borg-cemetery#79, summary on #713):
  - The throw scored 0: the point-blank divisor (5) on the flask's dice average of 2.
  - With #687's switch the throw tied melee at 1,003, the same ~1,000 danger term in both. Melee wins ties.
  - The poisoned walk to town was lost already: no cure carried. CLW and CSW do not cure poison in 4.2.
- **Owner:**
  - "step away from immobile creature with annoying effects and only attack them from a distance, if you have to";
  - "consolidate them" (one switch with #646's);
  - "they can be in the way, so kill them with ranged attack";
  - "do not block the path through their reach if they need to go to their stairs";
  - "set the switch to on".
- **Change:** `caution_ranged_vs_immobile` becomes a mode, compiled 0 and **shipped 2**.
  - 1 = #646.
  - 2 = also keep immobile creatures whose blows do more than plain damage at range (23 of 25).
  - 3 = all immobile attackers.
  - Kept at range means:
    - no melee (`borg_fight_attack.rs`);
    - no hunt walk (`borg_flow_kill.rs`);
    - flows keep out of the reach, its grid and the 8 next to it, never the Borg's own grid (`borg_flow.rs`). Stair flows (`borg_flow_spread_stairs`, `borg_flow_stairs.rs`) may pass through the reach, never onto the grid;
    - the step away runs while fleeing too, up to 3 per creature per level (`borg_caution.rs`).
  - Unit test `immobile_range_tests`.
- **Fixed along the way:**
  - #646's step did not run while fleeing;
  - an item under a jelly drew the Borg back beside it;
  - a reach block on the Borg's own grid would have stranded it.
- **Measured** (CALIBRATION, #713):
  - classes-18: no extra death in any mode (the same worm mass death in all four); CL 111 / 116 / 109 / 107 for modes 0–3.
  - Mode 0 is identical to main.
  - The case lives at 1–3 (another path); at 2 in the earlier two-switch version it stepped away, threw 45 flasks and lived.
  - smoke-borg-rs: identical at the shipped 2 (its three games meet no immobile attacker), so not re-recorded.
- **Left:**
  - The zoo experiment (borg-zoo, filed with #717) decides the shipped value.
  - #713's options (a) point-blank and (b) tie-break were not taken.
  - A Paladin throws other realms' books (`best_throw`), which can drop them under a monster.
  - Owner's idea, not yet an issue: when cornered, throw first while the monster approaches, then buff (`cornered_boost` buffs first).
