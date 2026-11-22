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

### Codex review follow-up

- Synced #888 and regenerated parameter citations. Each applied projection
  charges every positively scored unseen record once, including arcs and
  splash damage. Quiver selection applies only its winning aim. A capped
  record contributes no score even with a longer bolt-target age.
- Score damage, kill bonuses and danger conditional on the monster being
  there, then weight the contribution by belief. Added a unit test for
  multiple contributions, weighted utility and a scoring penalty.
- Checked the parameter citations and patch whitespace. The pull request
  tracks the Rust, vendor and harness check results. Switch-on smoke only
  checks that play completes. borg-zoo#625 measured the earlier head
  `a516df64` and needs a new request at the merged track-p head.

### Codex re-review follow-up

- Merged `origin/track-p` at `9085c266` separately. Kept confused
  direction aiming and regenerated the parameter citations.
- The selected aim stages its affected records. Only a queued attack can
  advance the ledger at the next normal command input after message parsing.
  Rejected casts, cancelled prompts and flushed commands discard the pending
  ledger. A launched miss still counts. There is one cap for real launches.
- The projection collects sleepers before applying score penalties. A launch
  counts each affected believed record once and marks it awake. Stubbed I/O
  tests cover the concentration failure and a winning ball with a penalized
  sleeper.
- The PR comment records the check results. Local Linux harness startup was
  unavailable on the Windows host.

### MiniMax-M3 re-review follow-up (P8)

- Merged `origin/track-p` at `4b2c2fcf` separately, including #937, and
  regenerated parameter citations.
- The unseen-shot trace reports the target, age and belief at aim time.
  The ledger still charges the launch against the sighting current at settlement.
- Curse requires a real monster target. Scoring excludes believed records,
  which provide only a last-known location.
- Attack rejection recognizes the engine's concentration and device failure
  messages. The parser still clears queued keys on a lock or disarm failure,
  while the pending shot survives the message. An engine input flush still
  cancels it. Stubbed I/O tests cover rejection, flushing, and multi-target
  blast settlement.
- F4 acknowledged. The `borg_target` guard currently always succeeds.

## Review of #890 by MiniMax-M3 at 37f037d7 (owner's choices, 2026-11-22)

- **F1 (fixed):** a believed record no longer counts as the monster next to
  the Borg in `borg_attack`'s adjacent test (`borg_fight_attack.rs`, the
  "close and fast" clause). With the switch on, a fast monster believed two
  grids away could otherwise turn a scary level's flight into a fight.
  `believed_weight` is `None` at `unseen_belief = 0`, so the shipped test is
  C's.
- **F2 (documented, measured apart):** the belief's window is the decay, not
  `unseen_max_age`. At the shipped 20%, a monster as fast as the Borg is
  believed for two Borg turns, and a faster one never. The module doc and
  `docs/CALIBRATION.md` give the table. By the owner's choice, borg-zoo#702 gets a
  second `light+belief` arm with a lower `unseen_decay_percent`, at a pin that
  contains this PR. The model is unchanged.
- **F3 (documented):** what can be aimed at is a record on the panel and in
  the view shape (`BORG_OKAY`, `BORG_VIEW`), and infravision raises the
  two-grid floor for warm monsters.
- **F4 (fixed):** `PROJECT_STOP` is cited at src/effect-handler-attack.c:130.
- No unit test for F1: the adjacent test sits inside `borg_attack`'s live
  loop. The guard is the same `believed_weight` call the knowledge gate
  above it already uses.
