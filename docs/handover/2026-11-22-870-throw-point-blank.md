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

Validation at runtime `e037588b8933a42e577808b5299d5da74e37cef7`:
formatting and Clippy pass; workspace 156 + 4 + 6 tests pass; core 615
pass / 4 ignored; citations 2353/2353; vendor contract passes. Python
checks: scripts 53 (1 skipped), tools 96, engine 96 (4 skipped). The sparse
checkout initially lacked historical fixture/link targets; after restoring
the tracked evidence files all workspace checks passed.

Native smoke is 3/3 byte-identical to current reference keys, messages and
states. Named Half-Troll Necromancer 1705507033 (race7/class4,
`menu,exact`, 100,000 turns) lives at CL6/max DL2 with both inheritance and
divisor 1. Keys and game states match exactly; messages and full traces do
not: eleven throw scores differ across eleven thinks, four throw eligibility
gates change from nonpositive to positive, and two interim incumbent flags
change. Final winners and every key remain unchanged. The wild-dog scores
12→18 and 10→16 are two examples, not the full count. At the original crow both choose melee at 4442, then
Phase Door at 4452. No re-recording, no natural deaths, and no independent
population samples are claimed.

Canonical packet: [Cemetery immutable cfce44c5](https://github.com/diegoami/borg-cemetery/tree/cfce44c534d97f8ee1d77d8a1a5524ecdeaac6fe/validation/borg-reborn-rs/throw-point-blank-870),
113 artifact hashes verified; binary, full settings/logs/traces/saves and
receipts retained. Cemetery identity check verifies 1802 retained dossiers;
no identity changes. The evidence branch was pushed, but automatic approval
review rejected opening its PR, citing insufficient explicit authorization
to export logs/saves/binaries to the separate repository. Its PR is pending
owner approval; no merge was attempted.

[Zoo #618](https://github.com/diegoami/borg-zoo/issues/618) requests paired
inheritance versus divisor 1 on the same exact source, deaths by 100k,
CL/depth guardrails, per-class outcomes, oil use and adjacent-fast-monster
deaths; owner dispatch and a new unopened confirmation block remain pending.
The owner chooses a shipped value after the Zoo comparison. No population
experiment is dispatched here and no improvement is claimed.

Source refresh used `--no-fetch --no-issues` on the existing read-only
vendor caches (limited by available disk). It reports 7 borg-rs commits,
201 better-borg-c commits (new death batches D0020–D0030), 21 tracker
commits, no newer cached engine/runner commit, and cannot resolve Zoo's
origin/HEAD. No pins or donor code are changed by this task.

Review #875 at `239e9a7f` approved the code with F1/F2 requested before
integration. F1 is independently verified from the retained full traces
(11 changed BF_OBJECT candidates at turns 11915, 25815, 28035, 28045,
31257, 33210, 42350, 48880, 48890, 68911, 84915; eligibility flips at
48880, 48890, 68911, 84915). The earlier two-example wording understated
the scoring surface; no final choice or key changed.

F2: merge current `track-p` `7e21967b` (now including #864 and #874)
into this published task branch without rewriting history. Keep both
calibration sections and repair citations; preserve track-p's shipped
settings/scary list and current reference files. Main's existing staircase
and release changes remain in this branch. Fresh merged-runtime checks
are recorded below when complete. Historical evidence above stays pinned
to `e037588b`; it does not measure this newly synchronized runtime.
