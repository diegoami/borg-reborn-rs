## 2026-11-20: Ranger arrows, a soft cap then a taper (#716)

Owner: "they do not need that many arrows"; "a soft cap at 50 and then a
curve for additional arrows utility, configurable, say 75".

- **Found** (classes-18, the two Rangers, 100,000 turns):
  - Rangers bought about 300 arrows a game and carried 124–166 (median, up
    to 200), and fired fewer than 100.
  - The town planner's shot reserve ("Ranger usable shots") is a curve whose
    demand grows with the trip: 24 arrows at birth, half value there. It
    never stops paying, and an arrow costs 1 gold.
  - The weight repair then sent Phase Door to the Home (5 in one game).
- **Code** (`borg_town_capability.rs`, `params.rs`):
  - `town_ranger_arrows_max` (0 = C): the reserve becomes a requirement,
    each shot up to the cap worth the same.
  - `town_ranger_arrows_taper`: a capability "extra shots" (60) counts only
    the shots past the cap. Each is worth linearly less than the last,
    nothing past the taper (`extra_gain`, with a unit test).
  - No plan-schema change.
- **Shipped:** the Ranger, 50 and 75.
- **Stacked on #712** (PR #715): this branch merges it, because both add a
  capability (#712 is 59, #716 is 60). If #715 merges first, this PR
  carries only #716.
- **Local check** (same two Rangers):

  | | C | Cap 60 (first try) | Soft cap 50, taper 75 |
  |---|---|---|---|
  | Arrows bought | 290 / 316 | 219 / 165 | 217 / 120 |
  | Median carried | 166 / 124 | 70 / 57 | 72 / 62 |
  | Most carried | 200 / 160 | 120 / 80 | 120 / 80 |
  | Phase Door sent home | 5 / 0 | 0 / 0 | 0 / 0 |
  | Max CL | 6 / 6 | 5 / 7 | 6 / 5 |

  - Off, both games play main's keys.
  - A first try, a diminishing curve past the cap weighted as the reserve,
    carried more arrows than C (median 102 and 195) and was replaced by the
    taper.
  - The most carried (80–120) comes from arrows picked up in the dungeon:
    pack power keeps up to 180 (`power_inventory_amissiles_limit_3`,
    unchanged).
- **smoke-borg-rs:** no Ranger in it; the games match the reference merged
  from #712 (`scripts/rerecord-smoke-borg-rs.sh`: nothing to re-record).
- **Checked:** fmt, clippy `-D warnings`, both test commands (core 473),
  the class files check, `borg-core-vendor.py check`, citations (2204).

**For the owner:** whether to lower pack power's 180 too, so that picked-up
arrows stop piling up.

**Left:** a zoo measurement for the Ranger.
