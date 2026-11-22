## 2026-11-22: Fang at CL1: the oil's hit chance is the engine's (#986)

**Core:** every check and replay ran on the default **borg-rs** core (`borg-core/`), built
from track-p `4b2c2fcf`. The settings are the shipped `harness/borg.txt` plus the zoo arm
`reborn-head-shipped`.

### Reproduced

Both deaths play key for key:

| Seed | Character | `sha256 keys.log` | End |
|---|---|---|---|
| 1194051723 | Elf Paladin | `7fb11d46a041a04a` | dead at game turn 1700 |
| 1024822556 | Dunadan Druid | `3687ca84c5cdeb34` | dead at game turn 3086 |

The run mirrors the zoo's `borgrs.run_cell` (config overrides, argv, `-exact-turns`).

### Found

- **Hit chance: no gap.** The fight race rates a throw with `focused_launch_damage`. That
  multiplies the damage, capped at the HP still needed, by `hit_permyriad`
  (`borg-core/src/borg_fight_attack.rs:5463` at the merge with track-p `206cf4e7`; `:5318` at `4b2c2fcf`), which is `hit_chance` in
  `engine/angband/src/player-attack.c:196-219`. A gdb script breaks on
  `random_chance_check` (`z-rand.c:543-547`) inside `make_ranged_throw`. It reads
  `chance.numerator`, which is the hit chance `hit_chance` computed, not the random draw.
  - Paladin against Fang: 6770 for the engine; the Borg's 14.894 is 22 × 0.6770.
  - Druid against Fang: 5668; 12.469 is 22 × 0.5668.
  - White icky thing: 8701; 4.350 is 5 × 0.87, with the damage capped at the 5 HP still
    needed.
  - The gap is 0 on every throw that has a fight-race rating just before it, 8 in all:
    - the Paladin's 6 against Fang (game turns 1551, 1571, 1591, 1611, 1621, 1662, all at
      6770);
    - the Druid's throw at Fang (2342, rated "not winning"; the throw came from another
      planner);
    - the Druid's throw at the icky thing (2392).
    The engine's chance for every throw of both games is in the cemetery study's
    `engine-throws.txt`.
  - **So there is no fix behind a parameter:** nothing differs.
- **"2.000 turns to kill" is the expected damage, not an assumption of hits.** It is the HP
  still needed over (capped damage × hit chance), rounded up to whole commands
  (`borg_caution.rs:2163-2181`). `wins_race` compares means: 2 × 4.75 = 9.5 against HP 10–11.
  - A simulation fought to the end gives about 81–90% to win, so "winning" was fair.
  - The Paladin instead drank a CLW at every "not winning" flip (HP 6–9). Then five misses at
    67.70% (0.35%) took it to HP 2.
  - That policy question is **#992**.
- **Druid:** every one of its 16 fight-race lines against Fang says "not winning", and it threw
  at Fang only once. The issue's "3 hits in 6" includes throws at a white icky thing and a grey
  mushroom patch. Its death is the flight and the Recall refusal (#758).
- **#929's wall write does not change these fights.** I ran an uncommitted, environment-gated
  patch that never writes the guessed wall on the Borg's own grid. The guard fired 5 times
  (Paladin) and 2 times (Druid), and both key streams stayed byte-identical.

### Changed

- One unit test, `fang_oil_hit_chance_is_the_engines_chance` in `borg_fight_attack.rs`. It
  pins:
  - the engine's hit chance at the three measured (to-hit, AC) pairs: (73, 36), (52, 36) and
    (52, 8);
  - the arithmetic of the race's path through its own helpers: `thrown_damage` (22 HP), the cap
    at the HP still needed (both the uncapped and the capped case), `race_turns` and
    `wins_race`.
  - It does not run `focused_launch_damage` itself, which needs a full game state.
- After review: `scripts/check-param-citations.py --fix` moved the citation of
  `attack_speed_fight_danger_tenths` (`params.rs`, `docs/CALIBRATION.md`) to the line the new
  test pushed it to.
- **No play change**, so `smoke-borg-rs` is not re-recorded.

### Evidence

borg-cemetery branch `study/986-fang-cl1-throw-hit`, `studies/fang-cl1-throw-hit-986/`:

- the replay script, the gdb script, the counterfactual patch and the simulation;
- for both seeds: the keys, the messages, `borg.txt` and the engine throw log.

### Left

- **#992:** the race decides on means with a thin margin, and the Borg switches between
  throwing and CLW. A lead for the owner, to go behind a parameter and be measured by borg-zoo.
- **#929** stays open on its own evidence. These two games show no effect from it.
- **#758:** the Druid's Recall refusals.

### For the owner

- Whether to take up #992.
- Whether the cemetery branch gets a PR. It is pushed; no PR is opened.
