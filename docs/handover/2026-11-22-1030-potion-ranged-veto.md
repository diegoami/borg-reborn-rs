## 2026-11-22: unknown-potion test and ranged monsters out of sight (#1030)

**Found** (Codex on #1026, P2). `borg_quaff_unknown_when_safe` (#983) covered
the turns a bad potion costs with `borg_danger(.., c = item_test_unknown_danger_turns)`.
`c` scales each monster's movement energy (`borg_danger.rs:2173`), which
widens the melee reach; the ranged part counts a monster only if it can
project to the Borg now or after one move (`q` capped at 20,
`borg_danger.rs:2413-2470`). A caster two moves behind a corner counted 0.

**Done.** `ranged_threat_in_window` (`borg_item_use.rs`): no test while a known,
awake monster with a ranged attack (`kill.ranged_attack`, as
`borg_danger_spell`) is within its moves in the window plus the engine's
`max_range` (20), whether or not it can project now. Its moves come from its
speed and the player's (`extract_energy`), rounded up. A record kept after the
monster is lost counts too. Note: `# Not testing an unknown potion: <race>
could shoot or cast within N turns (#1030)`. `borg_danger` is unchanged.
Unit test `ranged_threat_tests` (the corner case, the range edge, asleep,
melee-only, a fast caster). No new parameter.

**Checked.** fmt, clippy, the workspace and `borg-core` tests,
`check-param-citations.py` (2,394, 0 wrong). Smoke: 3/3 identical (nothing
re-recorded). The 14 named cases of #969/#983: the veto never fired (1-10
potion tests a game), outcomes as before (12 at the cap; Grip #974 and a crow).

**Left.** The veto is conservative (30 grids for a normal-speed caster) and
may delay tests on caster-heavy levels; the normal-world zoo run of the potion
test measures it.
