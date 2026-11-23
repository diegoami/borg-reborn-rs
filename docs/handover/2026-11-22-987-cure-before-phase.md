## 2026-11-22: First cures before the Phase Door reserve; the Ranger ships 2 (#987)

Every check below ran on the **default core, borg-rs** (`borg-core/`), on
the shipped `harness/borg.txt`, built from `origin/track-p` (first `4b2c2fcf`,
now `cd1e81e0`, see the merge section) plus this branch. No check ran on the reborn core: this change does not
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
  cure" (id 77, appended to `NAMES` after #978's Restore Life reserve, 76). Each copy is worth
  `town_first_cure_percent` of the trip's escape weight, the Phase Door
  reserve's unit. A copy past them adds nothing.
- `town_first_cure_percent` (200): a margin on the cure's whole value, not a
  threshold against `town_phase_reserve_percent` (150). The purse decides on
  whole wares; at the record, a first cure at 200 is worth 410,588 against a
  reserved Phase Door's 219,130.
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
  `cargo test --release -p borg-core` (654 passed at `4b2c2fcf`, 665 after
  the merges), `scripts/tests`, `tools/tests` and `check-param-citations.py`
  pass. `--fix` re-pointed the citations whose cited lines had shifted, in
  `params.rs` and `docs/CALIBRATION.md`.
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

1. On is now off plus the first-cure row. At the time, the fix zeroed only
   the rows the model held because of the parameter; since the merge of
   `2415bedf`, the model is padded to every row with zero-weight rows
   instead. Before the fix,
   `RANGED_RESERVE` and the prepared rows were zeroed even where the off
   model held them at the default weight (inert: nothing supplies them).
2. The test pins `town_first_cure = 2`: two copies at 240,000 each, and a
   third adds nothing.
3. A compile-time assertion keeps `FIRST_CURE` the last row.
4. The parameter text says the order is decided on the whole wares in the
   purse. The percent is a margin, not a switch against the reserve's.

### Merges of track-p `2415bedf` (#978, #979) and `c15a4e73` (#975)

#978 appended its own last row (Restore Life reserve, 76). The first-cure
row is now 77. `initial_curves` sizes the model to the Restore Life row, not
to `COUNT`, so neither row appears unless its own parameter is on. With
`town_first_cure` the model is padded to every row with zero-weight rows.
Re-checked on the default core (borg-rs) against a build of `2415bedf`:
- Seed 681096527 plays the same game on both, `665b0b0cf9c3af43`, with
  byte-identical plan records.
- The counterfactuals keep `ae44baf763d8a7f9` and `d81348b6b29fef5b`.
- Smoke is 3/3 identical to `smoke-borg-rs`.

`c15a4e73` (#975, `kill_live_zero_hp`, off by default) conflicted only in
`docs/CALIBRATION.md`, where both sides appended a section. Re-checked against
a build of `c15a4e73`, with the same results: the same hash and plan records,
the same counterfactual hashes, and smoke 3/3.

`cd1e81e0` (#1021, the P7 review fixes) conflicted only in `params.rs`:
both sides added parameters. Re-checked against a build of `cd1e81e0`, with
the same results as before: the same hash and plan records, the same
counterfactual hashes, smoke 3/3, and `cargo test --release -p borg-core` at
674 passed.

`206cf4e7` (#980, the emergency attack beyond the current mana; #1026, the
P8/P9 split) conflicted only in `docs/CALIBRATION.md`, where both sides
added a section. **#980 changes this seed's game on `track-p` itself.** The
keys split at byte 8437, after the town visit. On a build of `206cf4e7`:

| arm | turn 541 | end | play hash |
|---|---|---|---|
| control (base and this branch, off) | 6 Phase Door | 100,000-turn cap, CL6, DL2 | `cfe52e69e9114774` |
| `town_first_cure__ranger = 1` | 5 Phase Door + 1 CLW | killed by a metallic red centipede, turn 7583, CL2 | `a318e3097fe7d2e5` |
| `town_first_cure__ranger = 2` | 2 CLW + 3 Phase Door | 100,000-turn cap, CL6, DL2 | `5b8d3f58c1a265ad` |

The off state is still the base's: the same hash and byte-identical plan
records. Smoke is 3/3, and `cargo test --release -p borg-core` passes with
677 tests. The one-seed counterfactual has now gone both ways on two builds
a few commits apart. It says nothing about the rule's effect; only the zoo
request (#1006) can.

### Re-review (MCode, on `c679cbcd`)

The verdict: the code is right and all six findings are closed. The four
documentation points it raised are fixed:
- `town_first_cure_percent` now cites its read line
  (`borg_town_capability.rs:1062`). `--fix` had left it pointing inside
  `observe`.
- The "above the reserve's percent" framing is gone from `harness/borg.txt`
  and from this note.
- The review section above says how the code works now.
- The baselines and counts are the current ones.

### Shipped for the Ranger (owner, 2026-11-22)

The owner asked: "let us merge like that having the value 2 for clw for
rangers". `harness/borg_ranger.txt` now sets `town_first_cure = 2`, folded
into `harness/borg.txt` as `town_first_cure__ranger = 2` with
`scripts/borg-class-files.py fold`. Every other class stays at 0. The row
still counts every potion with `HEAL_HP`; the owner kept that scope ("let us
merge like that").

- Smoke has no Ranger, so `scripts/rerecord-smoke-borg-rs.sh` found its 3
  games identical and re-recorded nothing.
- `harness/check-ranger-phase.py` (#797's Phase Door purchase trace) already
  pins its own settings. It now also pins `town_first_cure__ranger=0`. With
  the floor on, the trace's 3 CLW come before the rations, and nothing else
  changes.
- Seeds at the new shipped settings (default core, a build of this branch on
  `track-p` `4383976b`), leads only:
  - 681096527 buys 2 CLW at turn 541 and reaches the cap, CL6
    (`5b8d3f58c1a265ad`).
  - 282602274 plays the zoo's recorded game (`13e34b27193b2e8c`). It already
    bought 2 CLW; its grey mold death is #975's case.
  - 1895660873 reaches the cap, CL7. Other `track-p` changes are in that
    game too, so it is not evidence for the rule.

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
