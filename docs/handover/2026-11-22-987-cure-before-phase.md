## 2026-11-22: First cures before the Phase Door reserve, off by default (#987)

Every check below ran on the **default core, borg-rs** (`borg-core/`), on
the shipped `harness/borg.txt`, built from `origin/track-p` `4b2c2fcf`
plus this branch. No check ran on the reborn core: this change does not
touch it.

### Reproduced

Seed 681096527, Elf Ranger (`-race 2 -class 7`), with the zoo report's
recipe (borg-zoo#668's page, the shipped `borg.txt` plus the arm's
overrides). The play hash is `665b0b0cf9c3af43`, as in the issue. The death
is the same: a crow at game turn 9024, CL2, DL1. The plan records
(`BORG_TOWN_PLAN_RECORD`) do not change the keys.

### The valuation (from the record at game turn 541, shop 4)

The character has 14 HP and 109 gold after the torches. `borg_town_w_gold` is
0, so the solve is a knapsack on value within the purse. These are the record's
own curves:

| row | weight | demand | one copy supplies |
|---|---|---|---|
| healing HP | 240,000 | 33,600 | CLW: 14,000 (20 HP capped at max HP) |
| healing reserve (200% of 14 HP) | 200,000 | -28,000 | CLW: 14,000 |
| phase | 180,000 | 3,600 | Phase Door: 1,000 |
| phase reserve (10 for the Ranger, 150%) | 1,800,000 | -10,000 | Phase Door: 1,000 |

- First CLW: 70,588 (curve) + 100,000 (reserve) = **170,588**, for 20 gold.
- Each reserved Phase Door: 180,000 (reserve) + its curve share. The first
  is worth 219,130 and the sixth **187,849**, for 15 gold each.
- So every reserved scroll, the sixth included, outbids the first cure. It
  is also cheaper. The chosen plan was 6 Phase Door (1,190,279), 2 rations
  and 13 arrows: 1,572,667, `solve[0]`.

This holds for any HP, not only at low gold. The healing reserve gives a
potion at most `borg_town_w_bleeding` × UNIT (100,000) for a full heal. The
healing curve adds less than its weight. A reserved Phase Door is 150% of the
escape weight (180,000). Low gold only decides whether anything is left after
the 10 reserved scrolls (150 gold), so a Ranger needs about 170 gold at the
Alchemist to buy one cure. The issue's lead is confirmed, with this
mechanism: the cure is not "valued after" the reserve; it is worth less per
copy than any reserved scroll.

The engine facts behind it: CLW heals 20 HP and costs 20
(`engine/angband/lib/gamedata/object.txt:3161-3166`), Phase Door teleports 10
and costs 15 (`object.txt:2576-2581`).

### The rule (behind parameters; the default keeps today's play)

- `town_first_cure` (count, 0 = off): the live town planner values the first
  this many healing potions as a requirement on a new capability row, "first
  cure" (id 76, appended to `NAMES`). Each copy is worth
  `town_first_cure_percent` of the trip's escape weight, the Phase Door
  reserve's unit. A copy past them adds nothing.
- `town_first_cure_percent` (200): above `town_phase_reserve_percent` (150),
  the first cures come before the reserved scrolls.
- Off, the model keeps its previous shape (same row count). The plan records
  of the seed are byte-identical, and so are the keys.
- Per class with `__<class>`. `harness/borg.txt` ships `0` and `200`, with a
  comment.

The unit test `the_first_cure_comes_before_the_phase_door_reserve` uses the
record's numbers. It checks today's order (the sixth Phase Door > the first
CLW). It also checks the order with the row on: the first CLW > the first
reserved scroll, and the second CLW still < the sixth scroll.

### Checks (borg-rs core)

- `cargo fmt --check -p borg-reborn-rs`, `cargo clippy --workspace
  --all-targets -- -D warnings`, `cargo test --workspace --exclude borg-core`,
  `cargo test --release -p borg-core` (654 passed), `scripts/tests`,
  `tools/tests`, `check-param-citations.py`: pass. The citation fix also
  moved 12 rows in `docs/CALIBRATION.md` whose cited lines had shifted.
- Smoke: 3/3 runs identical to `harness/reference/smoke-borg-rs`, so it is not
  re-recorded. `harness/check-ranger-phase.py`: reserve 0 and 2 match.
- Seed 681096527 at the shipped settings: same hash `665b0b0cf9c3af43`.

### One-seed counterfactual (a lead, not a measurement)

With `borg_param_town_first_cure__ranger` set, the same seed:

| value | turn 541 purchases | end |
|---|---|---|
| 0 (shipped) | 6 Phase Door, then 2 rations and 13 arrows | killed by a crow, turn 9024, CL2 |
| 1 | 5 Phase Door, 1 CLW, 2 rations | 100,000-turn cap, CL6, DL2 (`ae44baf763d8a7f9`) |
| 2 | 2 CLW, 3 Phase Door, 18 arrows | 100,000-turn cap, CL7, DL3 (`d81348b6b29fef5b`) |

After the split every later event differs. One seed shows no cause and no
rate (RULES 14). This session could not write the counterfactual to
borg-cemetery: there is no push access. The case package is left (below).

### Review (MCode, MiniMax-M3.1-Flash, on `ced0152d`: pass with findings)

The fixes are in the second commit:

1. On is now off plus the first-cure row. Only the rows the model holds
   because of the parameter (`base_count..FIRST_CURE`) are zeroed. Before,
   `RANGED_RESERVE` and the prepared rows were zeroed even where the off
   model held them at the default weight (inert: nothing supplies them).
2. The test pins `town_first_cure = 2`: two copies at 240,000 each, and a
   third adds nothing.
3. A compile-time assertion keeps `FIRST_CURE` the last row.
4. The parameter text says the order is decided on the whole wares in the
   purse. The percent is a margin, not a switch against the reserve's.

### Left, and for the owner

- **#1006**: the zoo request for `town_first_cure` 1 and 2 against `ctl`. It
  waits for borg-zoo#669, and its pilot is the 9 seeds where track-p buys
  Phase Door instead of a cure. It must go onto borg-zoo: this session has
  no write access there and dispatches no zoo run.
- **#1007**: the borg-cemetery case for the counterfactual above. This
  session has no push access there.
- **To decide** (the review's finding 6 confirms it): the row counts every
  potion with `HEAL_HP`. In `object.txt` at `574b618c` that is CLW, CSW, CCW,
  Healing, *Healing*, Life, and also Heroism (10 HP) and Berserk Strength
  (30 HP), which are fear remedies. Should only Cure Light, Serious and
  Critical Wounds count? This must be decided before a value ships.
- To decide: whether `town_first_cure_percent` should be its own number or
  follow `town_phase_reserve_percent`.
