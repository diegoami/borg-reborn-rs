## 2026-11-19: ranged first against immobile attackers (#646)

Owner: "always choose ranged against immobile but annoying monsters"; "move away, drink cure, throw flask". In borg-cemetery#75 a CL1 Priest drank three cures next to a grey mushroom patch, which confused it again each time.

- **`caution_ranged_vs_immobile`** (compiled and shipped 0). At 1, next to a known immobile attacker (`NEVER_MOVE`, with a blow, not `NEVER_BLOW`), the Borg steps to a free grid out of its reach, in two cases:
  - **confused:** every time. The step goes astray 75% of the time, so it is tried again.
  - **carrying a ranged attack** (`BI_AMISSILES`): once per monster per level.
- **The cure is held:** "# Not fixing Confusion next to an immobile attacker".
- **Neither applies** when the monsters that move are as dangerous as the HP left (`mobile_danger`).
- **Code:** `immobile_step_away`, `step_away_wanted`, `first_step_from` (`borg_caution.rs`). The step runs before the heal block in `borg_caution`. `BorgMonInfo` gains `immobile_stepped` and `immobile_stepped_level`.
- **A loop, found and fixed:** the first version stepped away every time. A Paladin's exploring flow walked back past a grey mold 512 times (1,298 steps in 18 games). The core ignores molds below CL10, so no throw followed. The once-per-level memory ends it: 49 steps, at most 10 in a game.
- **classes-18 at 100,000 turns, local, 0 against 1:**

| | 0 | 1 |
|---|---:|---:|
| deaths | 1 | 0 |
| sum of CL | 101 | 105 |
| sum of max DL | 37 | 36 |
| steps (confused) | – | 49 (18) |
| cures held | – | 27 |

  - 9 of 18 games change.
  - Priest 1569794094 (#75) lives, at CL6.
- **Smoke at 0:** 3/3, so the reference is unchanged.
- **Not done:**
  - Walking up to an immobile monster in the first place (the hunt and flows).
  - The throw itself: it follows only if the attack chooser picks it, and molds below CL10 are not targets.
  - Buying something to throw (#649, #652).
- **For the owner:** whether to ship 1, now or after a zoo pilot.
