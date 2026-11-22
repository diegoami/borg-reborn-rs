## 2026-11-22: Confused attack on the grey mushroom patch, shipped (#901)

**Done.** One shipped value in `harness/borg.txt`, on the owner's decision
(2026-11-22, after #867/#868 and #869):
`borg_param_attack_confused_by_direction` 0 → 1.
`caution_ranged_vs_immobile` and `caution_ranged_only` stay 0. With the
#869 kill-on-sight entry (`harness/scary_monsters.txt`, grey mushroom patch,
`attack` / `kill-on-sight`, melee, missiles, throws, devices) and
`attack_on_sight` 1, the Borg attacks the patch on sight. Once confused next
to it, it throws oil or uses a device by direction instead of only swinging,
and holds the cure while it has an attack.

**Dropped.** The first commit also shipped
`attack_throw_point_blank_divisor` 1 (#870). The owner dropped it: the patch
does not need it, because kill-on-sight's focused planner ignores the
penalty. At 1 it changed the smoke Warrior. At CL2-3, with the starting
Dagger (1d4), the Warrior threw oil at an adjacent white worm mass, rated 84
against a full round of blows' 43 (`thrust_damage` multiplies by
`BI_BLOWS`). The engine agrees on the damage: a thrown flask is 1d4 x 3
(`THROWING`) x 3 (`EXPLODE`, `player-attack.c:1296`), about 22 a hit. What
it costs (flasks kept for light, against oil spent on weak monsters) is
left to borg-zoo#636's `confused-pb` arm.

**Checked.** `smoke-borg-rs` re-recorded with
`scripts/rerecord-smoke-borg-rs.sh` (old and new compared in the PR); CI's
Rust checks.

**Not measured.** borg-zoo#636. Shipped before the measurement on the
owner's call (RULES 14). The open points before the switch is right are in
#896 (the step-away, the hold's gate against the chooser, the 11/32
truncation in the focused ranking, the scope).
