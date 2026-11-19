## 2026-11-19: seeing invisible monsters is low priority while shallow (#650)

Owner: a Scroll of Detect Invisible "should have low priority". In borg-cemetery#75 the CL1 Priest's town plan gave one Detect Invisible scroll the same gain as one Phase Door (26,190). The INVISIBLE capability had the same weight as trap and map detection at any depth.

- **New parameters:**
  - `town_invisible_min_depth` (compiled and shipped 0, off);
  - `town_invisible_shallow_percent` (100).
- **What they do:** while the max depth is above the depth, INVISIBLE weighs that percent of its ordinary weight (`invisible_weight`, `borg_town_capability.rs`). That covers Detect Invisible and See Invisible.
- **Unit test:** `seeing_invisible_is_low_priority_while_shallow`.
- **Engine:** the invisible monsters that matter start around DL10–15 (clear mushroom patch 10, air spirit 12, clear hound 15). Before that there are only the clear icky thing, the clear worm mass and Sméagol.
- **classes-18 at 100,000 turns, local, off against 10 / 10%:**

| | off | 10 / 10% |
|---|---:|---:|
| Detect Invisible bought | 13 | 0 |
| flasks bought | 41 | 48 |
| Cure Light Wounds bought | 8 | 14 |
| Phase Door bought | 16 | 12 |
| deaths | 1 | 1 |
| sum of CL | 101 | 100 |
| sum of max DL | 37 | 38 |

  - 16 of 18 games change.
  - The deaths: Priest 1569794094 (the #75 case) lives; Paladin 766958386 dies to an acolyte.
- **Smoke at the shipped values:** 3/3 against `smoke-borg-rs`, so the reference is unchanged.
- **For the owner:** whether to ship 10 / 10%, now or after a zoo pilot. Fewer Phase Doors bought is the one change to watch.
- **Left:** the Paladin's death was not traced.
