## 2026-11-20: Ranger arrows, a soft cap then a diminishing curve (#716)

Owner: "they do not need that many arrows"; "a soft cap at 50 and then a
curve for additional arrows utility"; 75 is "a curve coefficient like we
defined for healing ... can take infinite arrows, but the more he carries the
less value they have"; then "let us do scale 50 then, and let us do
experiment to calibrate it".

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
  - `town_ranger_arrows_curve_scale_percent`: a capability "extra shots"
    (60) counts only the shots past the cap. It is C's shot curve, weight
    and demand scaled by the percent (`extra_gain`, with a unit test).
  - No plan-schema change.
- **Shipped:** the Ranger, 50 and 50.
- **Stacked on #712** (PR #715), because both add a capability (#712 is 59,
  #716 is 60). PR #725 carries both.
- **Local check** (same two Rangers, main with #711 merged):

  | | C | Scale 25 | Scale 75 |
  |---|---|---|---|
  | Arrows bought | 290 / 316 | 205 / 242 | 270 / 244 |
  | Median carried | 166 / 124 | 68 / 68 | 104 / 145 |
  | Most carried | 200 / 160 | 120 / 120 | 157 / 160 |
  | Phase Door sent home | 5 / 0 | 0 / 0 | 0 / 0 |

  - Off, both games play main's keys.
  - Two earlier tries were dropped:
    - a curve past the cap weighted as the whole reserve carried more
      arrows than C;
    - a taper to nothing at 75 was not what the owner meant.
  - Scale 50 itself was not run locally. The zoo experiment measures it.
- **smoke-borg-rs re-recorded** (`scripts/rerecord-smoke-borg-rs.sh`).
- **Checked:** fmt, clippy `-D warnings`, both test commands, the class
  files check, `borg-core-vendor.py check`, citations (2205).

**For the owner:** whether to lower pack power's 180 too, so that picked-up
arrows stop piling up.

**Left:** the zoo calibration of the scale (borg-zoo, Ranger starts).
