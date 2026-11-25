## 2026-11-25: Depth-adjusted mixed exploration (#1220)

The optional `borg_param_explore_mix_depth_adjusted_level` uses
`max(1, CL - DL + 1)` for the per-class eligibility cap and the growth of
the starting radius, expansion step and maximum radius. The compiled and
shipped values are both 0. The existing stair leash still gates eligibility,
and the contact-damage model still uses actual dungeon depth.

The regression test covers the Priest cap of 9 on DL1–3, equivalent
growth at CL9/DL1 and CL10/DL2, the level-1 floor, and the disabled flag.
The Phase Door landing policy on Track P has no mixed-radius input.
The landing-policy work remains in [#1116][phase].

[phase]: https://github.com/diegoami/borg-reborn-rs/issues/1116
