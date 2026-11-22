## 2026-11-22: #1001 (unknown-potion test) into track-p (#1025)

**Done.** Merged `promote/P7` at `54cf7182` into `track-p` (after #1021,
which brought `promote/P7` at `39908ee4`: #971, #976). This adds #1001 (#983):
`item_test_unknown_potions` (shipped 1), `item_test_unknown_min_hp_percent`
(50), `item_test_unknown_danger_turns` (10), `item_test_unknown_max_depth`
(30), `borg_quaff_unknown_when_safe`, its `CALIBRATION.md` section, the
stair-handoff test's pin and #1001's smoke reference.

**Conflicts.** `borg-core/src/params.rs` only: the four `item_test_unknown_*`
rows are kept with `track-p`'s rows for `attack_on_sight` and
`attack_kill_on_sight_min_hp_percent`; the `item_use_*` rows differed only in
citations (re-checked with `check-param-citations.py`: 2,393 parameters, 0
wrong). `harness/borg.txt` and `docs/CALIBRATION.md` merged without conflict.

**Checked.** fmt, clippy, `cargo test --workspace --exclude borg-core`,
`cargo test --release -p borg-core` (674), `scripts/tests`, `engine/tests`
(the four `test_shopping` errors are this container's borg-cemetery checkout,
not at `death-tools-pin.txt`; CI runs them). Smoke on the merged tree: 3/3
identical to the reference (#1001's), so P8's merges do not change the smoke
games and nothing is re-recorded. Seed 17626119: test on, alive at the cap;
off, dead at 8,105 (as on `promote/P7`).

**Left.** `promote/P7` is unchanged (#939 unaffected). A `track-p` zoo
baseline now plays the floor and the potion test together; the normal-world
measurement of both is still to be defined (owner).
