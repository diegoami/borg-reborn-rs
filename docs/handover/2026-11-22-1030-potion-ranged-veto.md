## 2026-11-22: unknown-potion test and ranged monsters out of sight (#1030)

**Found** (Codex on #1026, P2). `borg_quaff_unknown_when_safe` (#983) covered
the turns a bad potion costs with `borg_danger(.., c = item_test_unknown_danger_turns)`.
`c` scales each monster's movement energy (`borg_danger.rs:2175`), which
widens the melee reach. The ranged part does not widen: every monster
further than `max_range` (20) from the Borg is zeroed before any line-of-sight
test (`borg_danger.rs:2384-2387`), and nearer ones count only if they can
project to the Borg now or after one move (`q` capped at 20,
`borg_danger.rs:2413-2470`). So every caster beyond 20 grids, and one two
moves behind a corner, counted 0.

**Done.** `ranged_threat_in_window` (`borg_item_use.rs`): no test while a seen
(`r_idx != 0`), awake monster with a ranged attack (`kill.ranged_attack`, as
`borg_danger_spell`; breath-only races count too) is within its moves in the
window plus the engine's `max_range` (20), whether or not it can project now.
Its moves (`moves_in_window`) follow the engine's accounting: each side acts
every ceil(100 / `extract_energy`) game turns (`turn_energy`,
game-world.c:136-139), rounded up, from the monster's forecast speed (+10 if
seen hasted, `forecast_monster_speed`). A record kept after the
monster is lost counts too. Note: `# Not testing an unknown potion: <race>
could shoot or cast within N turns (#1030)`. `borg_danger` is unchanged.
Unit tests `ranged_threat_tests` (the corner case, the range edge, asleep,
melee-only, the move budget against the engine table, a caster seen hasted).
No new parameter; the potion test ships on (`harness/borg.txt`), so the veto
is live in the harness and the zoo.

**Checked.** fmt, clippy, the workspace and `borg-core` tests,
`check-param-citations.py` (2,394, 0 wrong). Smoke: 3/3 identical (nothing
re-recorded). The 14 named cases of #969/#983: the veto never fired (1-10
potion tests a game), outcomes as before (12 at the cap; Grip #974 and a crow).

**Review (Mavis, at 18afadd4): pass with findings.** P2: the budget used the
base speed (now the forecast speed) and was one move short in some speed pairs
(now the engine's accounting); P2: the budget had no test (now
`the_move_budget_follows_the_engine` and `the_forecast_speed_widens_the_veto`).
P3s: the distance gate and `:2175` above; "seen", not "known"; the PR body
says the veto is live; the body's footer is the session tooling's (see #971).

**Left.**
- borg-zoo#703: the normal-world run of the floor and the potion test (with
  this veto); how often the veto delays a test (30 grids for a normal-speed
  caster) is measured there.
- Scope limit: a *sleeping* caster beyond `borg_danger`'s reach (more than
  20 grids, or out of sight from its grid and the eight around it) counts 0
  in `borg_danger` and is not vetoed either; it can wake during a paralysis
  (mon-move.c:1726-1779).

**Owner.** Whether the veto should also count sleeping casters (much wider:
any sleeping caster within reach would stop the test).
