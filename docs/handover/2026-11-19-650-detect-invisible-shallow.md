## 2026-11-19: seeing invisible monsters is worth less (#650)

Owner: a Scroll of Detect Invisible "should have low priority"; then "just make those objects less valuable" (the first version gated by depth).

- **Where the value came from:** a capability is worth `weight × supply / (supply + demand)` in the town plan.
  - INVISIBLE's weight is `borg_town_w_support` × the trip's exposure (55,000 for the CL1 Priest of borg-cemetery#75), and its demand is the trip's turns (about 1,100).
  - One read of Detect Invisible supplies `borg_town_support_turns` = 1,000 turns of service. The engine's scroll detects nearby invisible monsters once.
  - So one scroll was worth 26,190, exactly a Phase Door.
- **`town_invisible_value_percent`** (compiled 100, **shipped 10**): a value multiplier on INVISIBLE, like `town_recall_value_percent`.
  - Unit test: `seeing_invisible_is_worth_a_share_of_the_ordinary_weight`.
  - The first version's `town_invisible_min_depth` and `town_invisible_shallow_percent` are gone.
- **classes-18 at 100,000 turns, local**, measured with the first version at depth 10 and 10%. Every game stays above DL10, so that is the same as 10 here.

| | 100 | 10 |
|---|---:|---:|
| Detect Invisible bought | 13 | 0 |
| flasks bought | 41 | 48 |
| Cure Light Wounds bought | 8 | 14 |
| Phase Door bought | 16 | 12 |
| deaths | 1 | 1 |
| sum of CL | 101 | 100 |
| sum of max DL | 37 | 38 |

  - 16 of 18 games change.
  - The deaths: Priest 1569794094 lives; Paladin 766958386 dies to an acolyte. The Paladin was not traced.
- **Left:**
  - The same 1,000-turn service applies to every one-shot detection and mapping effect: traps, doors, stairs, monsters, map. Those may be overvalued too; a separate issue if the owner wants it.
  - Fewer Phase Doors bought is the change to watch in a zoo pilot.
