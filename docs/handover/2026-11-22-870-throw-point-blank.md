## 2026-11-22: Separate thrown-object point-blank policy (#870)

`task/T870-throw-point-blank` starts at `origin/track-p` (`29c44cbc`),
then fast-forwards to the accepted P6 promotion on main (`f57499d0`).
The task PR targets `track-p`; main's release and staircase fixes are
preserved. P6 was accepted and promoted in #822; P7 is now open.

`attack_throw_point_blank_divisor` is compiled 0: inherit the resolved
arrow divisor, including class/profile overrides. An explicit 1 removes
that conservation penalty from native throws rated with
`attack_throw_as_melee`. Arrows and legacy throws keep their own policy.
No shipped tuning is changed. The engine has no adjacent damage divisor
(`player-attack.c:156-165` at `574b618c`).

The focused attack-on-sight / fast-fragile planner already bypasses this
penalty. This change does not promise to alter its oil-versus-melee choice
in the motivating crow encounter. The test checks the normal rating with
a flask against adjacent crow AC 14, both divisors, inheritance, unique
and distant exemptions; parser coverage checks bounds and class overrides.

Validation results and the exact Zoo request will be added before handoff.
The owner chooses a shipped value after the Zoo comparison. No population
experiment is dispatched here and no improvement is claimed.

Source refresh used `--no-fetch --no-issues` on the existing read-only
vendor caches (limited by available disk). It reports 7 borg-rs commits,
201 better-borg-c commits (new death batches D0020–D0030), 21 tracker
commits, no newer cached engine/runner commit, and cannot resolve Zoo's
origin/HEAD. No pins or donor code are changed by this task.
