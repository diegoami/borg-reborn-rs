## 2026-11-25: Depth-adjusted mixed exploration (#1220)

`borg_param_explore_mix_depth_adjusted_level` uses
`max(1, CL - DL + 1)` for the per-class eligibility cap and the growth of
the starting radius, expansion step and maximum radius. The owner set the
shipped value to 1. The compiled default is 0. The stair leash still gates eligibility,
and the contact-damage model still uses actual dungeon depth.

The regression test exercises the mixed-mode gate with the standard stair
leash and checks the planner's radius inputs at the Priest cap of 9. It
covers DL1–3, equivalent growth at CL9/DL1 and CL10/DL2, the level-1 floor,
the leash cutoff and the disabled flag.

If [#1116][phase] adds a Phase Door radius check, it should use the radius the
mixed planner chose for the same level and stair-flow frame. The planner
already applies `explore_mix_depth_adjusted_level` to growth.
See the [Phase Door policy notes](../PHASE-DOOR.md) and the [note on #1116][phase-note].

The class caps came from DL1 tuning. A paired Zoo comparison with the flag
at 0 and 1 would measure their use as depth-relative caps on deeper floors.
When retuning a class's `explore_mix_max_clevel`, update its shipped
`prepared_dl2_min_clevel` to the cap plus one. The profile ships both values
explicitly and the registry test enforces the pair. The readiness threshold
uses actual CL. With depth adjustment enabled, the two level checks overlap
at `CL = cap + 1` on DL2.

PR #1221 keeps its original `task/1220-depth-adjusted-mix` branch name.
GitHub closes an open pull request when its head branch is renamed.

[phase]: https://github.com/diegoami/borg-reborn-rs/issues/1116
[phase-note]: https://github.com/diegoami/borg-reborn-rs/issues/1116#issuecomment-7150470774
