## 2026-11-20: immobile creatures kept at range (#713)

Paladin 766958386 (borg-cemetery#79) meleed a yellow jelly for 120 game turns, with 24 flasks unused, and died of its poison.

- **Analysis** (on borg-cemetery#79, summary on #713):
  - The throw scored 0: the point-blank divisor (5) on the flask's dice average of 2.
  - With #687's switch the throw scored 1,003, a tie with melee: both carry the same ~1,000 danger term. Melee wins ties.
  - #646's switch takes the game elsewhere.
  - The poisoned walk to town was already lost: no cure carried. CLW and CSW do not cure poison in 4.2.
- **Owner's choice:** not a poison-cure rule, but keep immobile creatures with annoying effects at range: step away, no melee, attack only from a distance.
- **Change:** `caution_immobile_ranged_only` (shipped 0).
  - 1: the 23 of 25 immobile attackers whose blows do more than plain damage.
  - 2: all of them.
  - Effects: no melee (`borg_fight_attack.rs`), no hunt walk (`borg_flow_kill.rs`), flows kept out of their reach (`borg_flow.rs`), the step away while fleeing too, up to 3 per creature per level (`borg_caution.rs`).
  - Unit test `immobile_range_tests`.
- **Measured** (CALIBRATION, #713):
  - The case lives at 1 (CL6, 45 flasks thrown).
  - classes-18 at 1: deaths 1 and 1 (the same death), CL 111 and 111, DL 40 to 41, 2 touches by immobile creatures.
  - Mode 0 is identical to main.
- **Left:**
  - A zoo pilot; then a value to ship.
  - The tie-break and point-blank questions (#713's options a and b) were not taken.
  - A Paladin throwing a Necromantic Tome at monsters (best_throw throws other realms' books) is a separate oddity.
