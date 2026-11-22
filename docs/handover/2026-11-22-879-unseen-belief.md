## #879: shoot at a monster believed in the dark (switch, shipped 0)

Branch `task/T879-unseen-belief`, track P (P7, owner 2026-11-22), stacked on
#888 (#877, `cave_light_engine_radius`).

### What was done

- New module `borg-core/src/borg_unseen_belief.rs` (ours, registered in
  `scripts/borg-core-vendor.py`). `believed_weight()` returns the belief
  (percent) for a record that is not current knowledge (older than
  `attack_max_target_age`) but is believed at its last grid. That means seen
  within `unseen_max_age` turns, at least two grids away, the grid not
  expected visible (`borg_follow_kill_aux`, now `pub(crate)`), and fewer than
  `unseen_max_shots` unseen shots since it was seen. The belief is 100% for
  NEVER_MOVE or a sleeper (until it has been shot at). Otherwise it is
  100 − `unseen_decay_percent` per monster turn.
- Hooks in `borg_fight_attack.rs`:
  - `borg_attack`'s target list admits believed records.
  - `borg_launch_bolt_aux_hack` admits them and multiplies the damage by the
    belief. This covers bolts, balls, missiles and throws.
  - `thrust_damage_with` gives 0, so melee never takes such a target.
  - `borg_launch_bolt` notes and counts each real shot
    (`# Unseen target (#879): …`).
  - A new field `FightAttackState::unseen_shots`.
- Parameters `unseen_belief` (0), `unseen_max_age` (10), `unseen_decay_percent`
  (20) and `unseen_max_shots` (3), all shipped at these values in
  `harness/borg.txt`. They have a CALIBRATION.md section, and the registry
  test maps the `unseen` prefix.

### Checked

- CI's four commands; `check-param-citations.py`.
- Smoke at the shipped settings: 3/3 identical to `smoke-borg-rs`.
- Human Mage 461220874 (zoo run 44936494812, csw cell config), local, 100,000
  turns:

  | `cave_light_engine_radius` / `unseen_belief` | keys | outcome |
  |---|---|---|
  | 0 / 0 | `d8a39edf` (= zoo) | dead, poison, turn 18,520, CL3 |
  | 1 / 0 | `0edf7944` | dead, grey mushroom patch (confused), turn 12,025 |
  | 0 / 1 | `d8a39edf` | unchanged: with C's radius the record is deleted |
  | 1 / 1 | `c0dab6ff` | **alive at the cap**, CL7, DL3, 12 unseen shots |

  This is one chosen case, so it gives a direction only (RULES 14).
- A first version had no shot cap. It made 41 unseen shots: 8 flasks at a
  "sleeping" white worm mass that was gone, and shots at a floating eye after
  "It dies.". That led to `unseen_max_shots` and the rule that a sleeper
  counts as awake once shot at.
- Dunadan Priest 399497898: I couldn't reproduce the Cemetery's keys locally
  (`c83e46cc` vs `dde95ceb`; the scary list may differ), so it is not reported.

### Not done / left

- #889: beliefs from unseen bumps and noises, attributing "It dies." to an
  unseen target, Light Room vs HURT_LIGHT, spreading the belief, beams/arcs on
  a case.
- `scripts/borg-core-vendor.py check` was not run: `vendor/borg-rs` is not
  checked out in this session.
- The zoo request is on the PR. The owner decides the shipped values after the
  result. The useful arms are both switches together and #877 alone.
