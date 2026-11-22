## 2026-11-22: A lone slow monster, danger without fear, the reserve spell's faint (#878)

Branch `task/T878-attack-on-sight`, from `track-p` (`7e21967b`). Track P,
milestone P7, by the owner's choice in session ("Track P (P7)", "all five,
default off"). Details in `docs/CALIBRATION.md` ("A lone slow monster, danger
without fear, the reserve spell's faint (#878)") and `docs/PHASE-DOOR.md`.

**Done.** The issue's five candidate rules, each behind a setting compiled
and shipped **off**: shipped play is unchanged, so `smoke-borg-rs` is not
re-recorded. Nine parameters (registry 2376 to 2385 on track-p `1cca4e0c`):

| rule | settings | where |
|---|---|---|
| 1. attack on sight a lone slow monster with the ranged attack | `lone_slow_attack`, `_radius` 3, `_casts` 2, `_damage_percent` 25 | new module `borg_lone_target.rs` (in `OURS`), called from `fight_before_flight`; `focused_ranged_attack` in `borg_fight_attack.rs` |
| 2. danger without fear for the Phase rule and the reserve spell | `escape_phase_lost_damage_only`, `attack_reserve_spell_damage_only` | `params::damage_only` / `danger_value`, `borg_danger_damage_only`, a fear-map hook in `borg_danger` |
| 3. no reserve-spell cast when the faint lets in too much damage | `attack_reserve_faint_percent` (0 = off) | `reserve_faint_refused` in `borg_fight_attack.rs` |
| 4. breeder bonus arms 5 and 15 | `danger_breeder_melee_extra_tenths` (existing) | no code |
| 5. unseen-bump Phase Door only for a fresh bump, with danger | `caution_unseen_bump_phase_turns`, `caution_unseen_bump_phase_min_danger` (0 = off) | `unseen_bump_response` in `borg_caution.rs` |

**Checked.** CI's commands: `cargo fmt --check -p borg-reborn-rs`, `cargo
clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace
--exclude borg-core`, `cargo test --release -p borg-core` (634 passed, 6 new
tests: the lone verdict, the fear-term set and scope, the faint mean and the
walkers, the bump gate), `scripts/check-param-citations.py`, `scripts/tests`.
Engine build and smoke at the shipped settings: **3/3 identical** to
`smoke-borg-rs`.

Named case, direction only (one case, RULES 14/20): the Cemetery save at turn
15037 (`counterfactual/start-key1461-turn15037.save`), this branch, track-p's
shipped `harness/borg.txt` (not the csw arm's), fresh Borg, 100,000 turns,
`exact`; every arm alive at the cap:

| arm | first contact (15047) | Phase Doors read | worms destroyed | CL |
|---|---|---:|---:|---:|
| shipped | Phase rule "danger 32": Phase Door | 5 (3 unseen bump) | 3 | 6 |
| `lone_slow_attack` | attack on sight, a flask of oil: destroyed | 0 | 9 | 7 |
| `escape_phase_lost_damage_only` | "danger 2 without fear (with fear 32)": no Phase Door, Magic Missile | 2 (2 unseen bump) | 2 | 6 |
| all five on (faint 25%, bump 20 turns, min danger 5) | as `lone_slow_attack` | 0 (5 unseen-bump Phase Doors refused) | 5 | 7 |

The lone rule chose the flask of oil, not Magic Missile: the focused
planner scored the throw higher (10.4 a turn), and one throw killed the worm.
The trial's runs are local (scratch), not published.

**Found.**
- `params::value` cannot grow: any extra code in it (even an out-of-line
  call) made the release unit test `an_easy_escape_is_near_stairs_or_an_escape_item`
  fail to link (`z_info`, `r_info` from `recall_escape_allowed`). It depends
  on what LLVM inlines. The damage-only switch is therefore in
  `borg_danger.rs`'s own accessor (`danger_value`), the only file that reads
  the fear terms. The fragile test is pre-existing; not changed.
- `danger_breeder_melee_extra_tenths` 15 is +150%; the comment in
  `danger_one_kill` (and C's) says 50%. Left as vendored.

**Not done / left.**
- Rule 3 never fired: none of the fresh-Borg arms reached a reserve cast
  (the original death's path needed its memory). It is covered by unit tests
  only.
- The danger without fear is still the danger's dice x sides and monster
  moves, not expected damage; poison over time is counted by none of the
  rules.
- `scripts/borg-core-vendor.py check` not run: `vendor/borg-rs` is not
  checked out in this session.
- The `tools/tests` suite fails locally (5 errors) on `track-p` too: the
  sibling borg-cemetery checkout is not at `death-tools-pin.txt`. The smoke
  ran with `BORG_CEMETERY_DIR` set to a checkout at the pin.
- **Measurement:** a borg-zoo request with the arms over the shipped
  `harness/borg.txt` (RULES 20): each rule alone, `danger_breeder_melee_extra_tenths`
  5 and 15, and the five together; deaths by 100,000 turns, white worm mass
  and poison deaths, Phase Doors read, CL and depth. Not filed: the owner
  starts zoo work.
- The named-case trial belongs in borg-cemetery (case-specific alternative
  plays, AGENTS.md); not published there.

**For the owner.** Whether to file the zoo request and publish the trial in
the Cemetery case; which values the arms use (the defaults are within the
issue's ranges: radius 3, two uses, 25%; the faint and bump settings have none).

### After the review (PR #891, pinned `1f4c040d`, follow-up at `296adfbf`)

- **F1 fixed:** `is_fear_term` skips `flag`, `mode` and `choice` units, so
  `danger_hunt_breeder_no_acid_fear` is no longer zeroed inside
  `damage_only`: 119 terms (114 prices, 5 bonuses). No danger value changed
  (its acid fear is zeroed anyway).
- **F2 fixed, with a correction:** the paralysis counter falls once every 10
  game turns (`decrease_timeouts` from `process_world`, game-world.c:651,
  :1080), not every game turn as the review said; the conclusion holds.
  `faint_borg_turns` converts the ticks to Borg turns (`energy / 10`), so a
  Borg at +10 speed counts twice the blows. Only speeds above normal change.
- **F3 fixed:** the reserve spell's loop passes the near monster it found
  (its Chebyshev distance) as the target; the faint's distances use the same
  metric.
- **F4:** `track-p` merged again (`4c686250`); citation-only conflicts.
- **F5 fixed:** the counts in CALIBRATION.md.
- **F6 not changed (note):** the Phase rule's damage-only danger uses `c = 1`
  rather than the site's horizon; they agree at the shipped
  `caution_danger_horizon` 1.
- Checked on the new head: CI's Rust commands (643 release unit tests),
  citations, smoke 3/3 identical. borg-zoo#626 was pinned to `296adfbf`; its
  pin moves to the new head: every arm, `ctl` included, now plays track-p
  `4c686250` (#863 and the rest), and `faint25` and `all` also change for
  characters faster than normal.
- Second review round (checklist at `b1218cfe`): merged `track-p`
  `e8895f5e` (#868, citation-only conflicts; #868's
  `attack_confused_by_direction` arrives with it); "turns" became "world
  ticks" in the `attack_reserve_faint_percent` description, CALIBRATION.md
  and a test comment; the PR body brought up to date. Checked: 649 release
  unit tests, `tools/tests` 96 with the Cemetery at `death-tools-pin.txt`,
  smoke 3/3 identical. borg-zoo#626's pilot ran on `296adfbf`; its
  `faint25` arm never fired there, so the fixes do not change its numbers.
- Third review round (MiniMax-M3 at `be94bb1f`, no new defect): merged
  `track-p` `4b2c2fcf` (#940, with main through #937; #901/#904: `attack_confused_by_direction`
  now shipped 1, `attack_throw_point_blank_divisor` 1 with four class zeros);
  smoke re-run on the merged tree: 3/3 identical to `smoke-borg-rs`, so the
  shipped settings play as recorded. F3: every new parameter cites its read
  line exactly. F4: the comments at `fight_before_flight` and its call name
  the lone rule. F5: `faint_reaches` (a monster that never moves strikes only
  when adjacent) is the tested skip; the 0-moves arm is labelled defensive.
  F6: CALIBRATION point 3 names the CON risk of the same over-exertion
  (player-util.c:824-831). Registry 2376 to 2385; 660 release unit tests.
  The named-case trial and the zoo arms ran with confused attacks off: with
  them on, a confused Borg's aimed item attack is rated at the engine's aim
  chance (#867), so the lone rule fires less often while confused.
- Fourth review round (at `91b5c42c`, no code defect): `faint_reaches`'s
  doc says what it decides (not excluded outright; arrival is the caller's
  reach); the merge reference is #940; the PR body says the zoo pin's rule
  code differs from the head only by the `faint_reaches` extraction, which
  skips the same monsters.
