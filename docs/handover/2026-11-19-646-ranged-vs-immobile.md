## 2026-11-19: ranged first against immobile attackers (#646)

Owner: "always choose ranged against immobile but annoying monsters"; "move away, drink cure, throw flask". In borg-cemetery#75 a CL1 Priest drank three cures next to a grey mushroom patch, which confused it again each time.

- **`caution_ranged_vs_immobile`** (compiled and shipped 0). At 1, next to a known immobile attacker, the Borg steps to a free grid out of its reach (`immobile_step_away`).
  - Immobile attacker: `NEVER_MOVE` with a blow and not `NEVER_BLOW`. 40 races: mushroom patches, molds (their base race), jellies, eyes, the lurker, the trapper, memory moss, Old Man Willow, the item mimics.
  - **Confused:** up to 3 steps per attacker. A step goes astray 75% of the time.
  - **Carrying a ranged attack** (`BI_AMISSILES`): 1 step per attacker.
  - The two are counted apart, so the confused tries do not use up the ranged step.
  - **The step's grid:** a floor grid less dangerous than the current one, not a bounce (`bouncing`), and not next to another immobile attacker.
- **The confusion cure is held next to the attacker at both cure sites** (`confusion_cure_held`): `borg_heal` and the cures at the end of `borg_caution`. Each site writes its own note.
- **Neither applies** when the moving monsters within two grids are as dangerous as the HP left (`mobile_danger`).
- **The step memory** (`step_allowed`; `BorgMonInfo::immobile_stepped`, `[[u8; 4]; 16]`, per level):
  - keyed by the attacker's grid, because kill indices are reused: six grey mushroom patches took index 18 on one level;
  - bounded, and it fails closed: with all 16 slots used, the step is refused.
- **Tests:** `step_away_from_an_immobile_attacker_only_when_it_helps`, `steps_away_are_bounded_per_grid_and_fail_closed`.
- **History:** the first version stepped every time, and a grey mold looped 512 steps. The first review found the second cure site ungated, the memory keyed on reused kill indices, and the memory failing open. The second review found the shared counter.
- **classes-18 at 100,000 turns, local, on main with #664, 0 against 1, before the counter split:**

| | 0 | 1 |
|---|---:|---:|
| deaths | 0 | 1 |
| sum of CL | 97 | 99 |
| sum of max DL | 37 | 38 |
| steps (confused) | – | 61 (16) |
| cures held / cured anyway in the same turn | – | 14 / 0 |

  - 13 of 18 games change.
  - The death is Blackguard 1472012981 in a yellow worm mass colony: borg-cemetery#78, #675.
- **Smoke at 0:** 3/3.
- **Not done:**
  - Walking up to an immobile monster in the first place.
  - The throw itself: the attack chooser decides, and molds below CL10 are not targets.
  - Buying something to throw: #649, #652, #666.
- **For the owner:** whether to ship 1, after a zoo pilot.
