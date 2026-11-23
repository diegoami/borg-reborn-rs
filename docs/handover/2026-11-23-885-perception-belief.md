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
- #889, item 2, `unseen_death_clears`: "It dies." / "It is destroyed." after
  a shot whose only believed target was one record deletes it.
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
- #889 item 1, beliefs from unseen contact and noise: #1127. A record needs a
  race, and an unseen attacker's is unknown.
- CBUG-0153 has no canonical issue in diegoami/neo-angband-tracker, where
  every other finding has one (`tracker` is null). This session cannot write
  there.
- None of the five switches is measured. #885 and #917 still have no
  recorded case (a forget or a mis-aim next to rubble or a monster's light).
  A zoo request would pair them with the shipped light and belief arms.

**Owner to decide.** Which switches the zoo measures, and in which milestone:
#885 and #917 are perception bugs (P8?); the #889 switches are new capability
(P9 by PLAN.md, where a PR merges into `track-p` only with its switches off,
as here).
