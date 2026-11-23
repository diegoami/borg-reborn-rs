## 2026-11-23: walk away from immobile monsters (#1147)

**Owner request, 2026-11-23:** "We need a flag which does all of this: we
need to have a list of 'walk_away immobile monster' or try to with the eye,
the patch and the jelly", and on the step: "it is very important to pick the
best square to (try to) step to - with a clean way to the stairs, if
possible, if not something not close to monsters, towards a room exit".

**Why.** borg-zoo#740 (patches-x8, track-p 1c071812, 918 starts per arm): the
18 games lost to a grey mushroom patch all end with the Borg confused beside
the patch, kill-on-sight attacking it in random directions; 4 more are
floating-eye paralysis chains. Counterfactual replays of those 22 zoo games
(borg-cemetery, to be filed with the zoo report) found partial rescues from
`attack_on_sight 0`, `attack_confused_by_direction 0` and
`caution_ranged_vs_immobile 1`, none of them the whole behaviour.

**Done (shipped 0, today's play unchanged):** `caution_walk_away_immobile`.
- **The list** (`walk_away_listed`): a race that never moves and is a
  `ranged` entry of `harness/scary_monsters.txt` for the character (floating
  eye, mushroom patches, jellies), or a paralyser without Free Action
  (#1125). Read whatever `caution_ranged_only` says (`scary_ranged_entry`).
- **Never close:** `kept_at_range` includes it (no melee, no hunt walk);
  `spread_blocked` and `borg_flow_enqueue_grid` close its grid and the eight
  around it for every flow, stair flights included (`in_walk_away_reach`);
  the Borg's own grid stays open.
- **Next to one, first action** (`walk_away_step`, `borg_think_dungeon` right
  after #1125's paralyser step):
  - on a staircase it may take (up unless forced descent; down only to a
    depth already reached and when Recall does not restrict), it takes it
    (#1139: stepping off and back killed two Borgs);
  - else a step to the square `walk_away_rank` puts first: a clean way to a
    usable staircase (known floor, out of every walk-away reach, not next to
    an awake monster that moves), fewest steps; otherwise the least danger
    from movers within two grids, then the fewest steps to a room exit
    (`chokepoint`, #544), then the most known floor around, then the least
    danger. Logged as `# Walk-away step from the NAME: WHY (#1147)`;
  - confused too (a step escapes about half the time next to a patch, a
    confused blow misses as often);
  - with no square out of reach, the think is marked cornered
    (`BorgMonInfo::walk_away_cornered`) and a blow at it is allowed in that
    think: the last resort after cures and escapes.
- Unit tests: `walk_away_tests` (the order, the path search, shipped 0); the
  `recall_restricts` call count in `borg_caution.rs` is 4.

**Checked.** CI's commands (fmt, clippy, `cargo test --workspace --exclude
borg-core`, `cargo test --release -p borg-core`: 843 passed). Replays in
patches-x8 (world data 6bdf6568...) of 60 starts, the 42 behind
borg-zoo#740's deaths and 18 survivors, 100,000 turns each, through
`harness/run.sh`:
- **Off:** all 60 games play key for key like track-p baf3871e.
- **On:**
  - 55 of 60 games change; 517 walk-away steps (grey mushroom patch 279,
    white jelly 185, floating eye 40, yellow mushroom patch 7, others 6);
  - 363 of the steps had a clean way to the stairs; no staircase take and
    no cornered think;
  - deaths 1 (Mage 300461652, a soldier, during a stair flight, one
    walk-away step in the whole game) against 0;
  - reached DL3: 6 against 10. A possible cost in progress, to be
    measured by the zoo.

**Not done / left.**
- Kill-on-sight from two grids or more is unchanged, also while confused
  (`attack_confused_by_direction` decides).
- No explicit poison or HP step in the re-evaluation: after the step the
  existing rules (cures, #1140 when it lands) act.
- Measuring it is the zoo's (RULES 14, 20): patches-x8, poison-x8
  (borg-zoo#741) and the normal world. Shipping 1 is the owner's call and
  re-records `smoke-borg-rs`.
