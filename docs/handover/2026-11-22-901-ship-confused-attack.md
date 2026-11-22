## 2026-11-22: Confused attack on the grey mushroom patch, shipped (#901)

**Done.** Two shipped values in `harness/borg.txt`, on the owner's decision
(2026-11-22, after #867/#868 and #869):

- `borg_param_attack_confused_by_direction` 0 → 1;
- `borg_param_attack_throw_point_blank_divisor` (new line; compiled 0 =
  inherit the arrow divisor, 5) → 1.

`caution_ranged_vs_immobile` and `caution_ranged_only` stay 0. With the
#869 kill-on-sight entry (`harness/scary_monsters.txt`, grey mushroom patch,
`attack` / `kill-on-sight`, melee, missiles, throws, devices) and
`attack_on_sight` 1, the Borg attacks the patch on sight; once confused next
to it, it throws oil or uses a device by direction instead of only swinging,
and holds the cure while it has an attack. `docs/CALIBRATION.md`: "Confused
attack on the grey mushroom patch, shipped (#901)".

**Checked.** `smoke-borg-rs` re-recorded with `scripts/rerecord-smoke-borg-rs.sh`
(the old and new references compared in the PR); CI's Rust checks.

**Not measured.** borg-zoo#636 (pilot, arm `confused-pb` is this pair).
Shipped before the measurement on the owner's call (RULES 14). The open
points before the switch is right are in #896 (the step-away, the hold's gate
against the chooser, the 11/32 truncation in the focused ranking, the scope).
