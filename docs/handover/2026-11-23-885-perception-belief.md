## 2026-11-23: stale light, sight through rubble, the C light-radius record, belief follow-ups (#885, #917, #886, #889)

**Owner, 2026-11-23:** "fix them in a single PR", after a check that all
four issues still applied on `track-p` `40c1c963`. So this PR closes four
issues (RULES 5 asks for one per PR; the owner asked for one PR).

**Done.** Five switches, compiled 0 and shipped 0 in `harness/borg.txt`, so
play at the shipped settings is unchanged. Each is documented in
`docs/CALIBRATION.md`, *Perception and belief follow-ups*.

- #885, `update_dark_clears_light`: a floor the engine shows dark loses a
  stale `BORG_LIGHT` (`dark_floor_info`, `borg_update.rs`). A cleared mark
  asks for `borg_update_light` in the same think, so the Borg's own light set
  is marked again.
- #917, `view_rubble_blocks_sight`: passable rubble stops the Borg's view
  (`sight_grid`, `borg_cave_view.rs`) and the view's line-of-sight fallback
  (`borg_los_sight`, `borg_projection.rs`), as the engine's `update_view_one`
  and `los()` do. `borg_cave_view.rs` is now a declared hooked file
  (`scripts/borg-core-vendor.py`).
- #889, item 2, `unseen_death_stops_shots`: "It dies." / "It is destroyed."
  after a shot whose only believed target was one record ends the shots at
  it for this sighting; the record is kept.
- #889, item 3, `unseen_light_reveal` (mode): Light Room at a believed
  `HURT_LIGHT` monster (2 = any believed monster) within its radius-2 sphere,
  as its own step before `borg_attack` in `borg_think_dungeon`. It is not a
  new `BF_*` attack kind, which would renumber the kinds before
  `BF_ACT_STARLIGHT`.
- #889, item 4, `unseen_spread`: erratic movers (RAND_25/RAND_50) and a
  ball's spare reach (`unseen_blast_slack`) spread the belief.
- #889, item 5: beams and arcs already reach the belief through
  `borg_launch_bolt_aux_hack`. Checked in the code and documented; no code
  change.
- #886: `docs/c-borg/c-bugs/findings/CBUG-0153.json`, `record_kind`
  `local-finding`, added to `catalogue.json`'s `finding_files`. Neither
  borg-rs's list nor better-borg-c's fixes track the defect. It is linked to
  CBUG-0003 and CBUG-0024, the other `borg_update_light` defects.

**Found.**
- The belief refuses a record wherever `borg_follow_kill_aux` expects it to be
  visible, so #885 and #917 also take beliefs away, not only records. That
  matters more now that `unseen_belief = 1` ships.
- The module notes in `borg_unseen_belief.rs` quoted the windows of the
  compiled 20% decay as "shipped". Corrected: 5% ships, and at 5%
  `unseen_max_age` (10) bounds a same-speed monster too.

**Left.**
- #889 item 1, beliefs from unseen contact and noise: #1127 (P9). A record
  needs a race, and an unseen attacker's is unknown.
- CBUG-0153's canonical issue is diegoami/neo-angband-tracker#644 (source
  check; no native C reproduction yet).
- The zoo measures #885 and #917 on borg-zoo#736: ctl / dark / rubble / both
  at `c831a693` (this PR merged with `track-p` `d3d1629f`), pilot then full.
  On the smoke suite locally, `rubble` changed 2 of 4 games (no outcome
  changed) and every other switch none. Whether each ships in P8 waits for
  that result.

**Review of #1128** (relayed by the owner, posted on the PR, at `f817c6cc`):
four P2 findings with the switches on, all fixed, all still shipped 0.
- F1: an anonymous "It dies." cannot name the victim. The switch is renamed
  `unseen_death_stops_shots` (was `unseen_death_clears`): it ends the shots
  at the record for this sighting instead of deleting it.
- F2: an unknown grid found to be passable rubble did not redo the view
  (the walk test sees no change). `view_needs_update` now compares the sight
  test before and after the map read.
- F3: Light Room could repeat at an invisible monster on a lit grid. It is
  now cast only at an unlit grid, once per sighting (`unseen_reveals`).
- F4: a record only a blast could reach never reached the aim list.
  `believed_for_aim` (slack up to 10, a dispel's radius) admits it there and
  in the next-to-me test; with `unseen_spread` 0 it is `believed_weight`.

**Decided (owner, 2026-11-23: "your call").** #885 and #917 are P8: they are
perception bugs the shipped light and belief exposed, and they are measured
now. The three #889 switches are P9: new capability, kept off until P8 is
promoted, and measured then.
