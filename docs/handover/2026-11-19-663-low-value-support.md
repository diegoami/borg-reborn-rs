## 2026-11-19: Magic Mapping, Detect Invisible, Boldness and Light are worth very little (#663)

Owner: "magic mapping, detect invisible, boldness, scroll of light is something you never buy … put their value at very low"; no flag per item; "we do not even need review for that".

- **Why they were bought:** the town plan valued a read of each as `borg_town_support_turns` (1,000 turns) of service against a trip of about 1,100 turns, at the support weight. So each was worth about one Phase Door. These numbers come from borg-rs's M13 planner.
- **Change:** in `borg_town_capability.rs`, the MAP, INVISIBLE and ILLUMINATE curves, and FEAR while not afraid, weigh 1% of the ordinary weight (`low_value`). There is no parameter. A carried or found copy keeps a small value, so it is not sold. FEAR while afraid keeps the restore weight.
  - Unit test: `what_the_borg_never_buys_keeps_a_small_value`.
  - #650's PR #658 (a per-item setting) is closed; #650 is folded in here.
- **classes-18 at 100,000 turns, local, gold spent:**

| | main | #663 |
|---|---:|---:|
| Magic Mapping | 2,210 | 0 |
| Detect Invisible | 1,200 | 0 |
| Boldness | 560 | 0 |
| Light | 105 | 15 |
| Cure Light Wounds | 1,980 | 2,780 |
| Cure Serious Wounds | 1,820 | 2,730 |
| Phase Door | 1,065 | 1,455 |
| flasks of oil | 1,188 | 1,572 |
| Iron Shot | 564 | 1,418 |
| deaths | 1 | 0 |
| sum of CL | 101 | 97 |
| sum of max DL | 37 | 37 |

- **Smoke:** re-recorded (`scripts/rerecord-smoke-borg-rs.sh`).
- **Left:** rings and amulets bought for power at low level, such as Feather Falling (`power_equipment_feath_weight` 500 below DL20), Slow Digestion (750) and Searching. That is another question, and the owner asked about it.
