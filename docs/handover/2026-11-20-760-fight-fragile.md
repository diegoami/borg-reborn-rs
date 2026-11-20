## 2026-11-20: attack categories and winning fights (#760, #759)

Implementation: [PR #763](https://github.com/diegoami/borg-reborn-rs/pull/763),
`task/T760-fight-fragile`, worktree `.worktrees/T760-fight`. #758 is the
separate stacked PR #764. #757/#761 belong to another session and were not
modified. Accepted main #755 and #762/#754 were synchronized, preserving
their settings and code; both new #754 switches remain shipped 0.

**Behavior:** `attack_on_sight` and `caution_winning_fight` compile/ship 0.
Attack entries extend the existing scary/ignore database with required
reasons, class/level limits and independent removal/replacement. Initial
reasons are fast/fragile and breeder; breeder entries retain the existing
stoppable/lost-colony policy. The all-class CL50 range is a hypothesis.
The category proposal, engine citations and historical per-class death
counts were presented before building.

The focused planner separates damage from utility. It retains exact HP
from the existing permitted observation, counts damage required to go below
zero, caps overkill before hit chance, uses fractional own blows/shots and
speed, finite supplies/shared mana, regeneration and observed monster Haste.
Slow is not assumed to persist throughout a forecast. A fox uses the native
zero-energy revert before a weapon blow, then the next think recomputes.
Unknown/unmodelled conditions fall through to ordinary caution. The action
rung precedes light/countdown/depth-return flight rungs; no_retreat still
advances. `caution_winning_fight=2` is a non-acting diagnostic, not a zoo arm.

**Checked:** clean baseline `08f94e1f` reproduced all six Ranger deaths with
exact canonical keys/states/messages before implementation. The seven named
starts (the six Rangers plus Rogue 390305857) were checked locally, at the
100,000-engine-turn cap, with separate switches and combinations. After the damage-first selector correction, all three together reach the
cap in six of seven named starts. Seed 1976035382 dies to a large white
snake at engine turn 17873; earlier versions reached the cap on that life and remain
retained. The Rogue control was already alive. This is selected-case evidence, not population improvement. Some
lives diverge before the old killer appears. The unchanged original
425904011 finishing state is independently evaluated by diagnostic mode:
1.044 turns to kill against about 1.064 turns to die, so active mode attacks.
All seven diagnostic controls preserve exact keys and states. Won-fight
examples and losing/multiple-attacker verdicts were inspected.

Canonical report, exact per-attempt builds/hashes, before/after table,
first differing keys and all retained deaths:
[borg-cemetery PR #88](https://github.com/diegoami/borg-cemetery/pull/88),
`studies/reborn-flight-fights-20261120/local-cases.md`. The initial active
fight checks used code `734b7c1a`, diagnostic checks `4b1e4c1f`; the final
wake-aware Recall/all checks used `dab3ea69`. Category/all checks after the
selector correction used `5acb8104`. Final main synchronization
retains #754's default-off additions. Do not duplicate the gameplay report
in this source repository or count repeated starts as independent samples.

CI's commands, citation checks (0 wrong), vendor check and shipped-off
smoke/reference/replay checks passed on the implementation. The final PR
head and its latest CI/reviewer checklist are recorded on the PR. No smoke
reference was rerecorded because the new switches ship off.

**Owner decision, 2026-11-20:** measuring belongs to borg-zoo (RULES 20).
This machine's lock is for builds/smoke/single named-case replays, not
population arms. No 50-per-class or 600-start candidate comparison was
started here. Measurement is [borg-zoo #511](https://github.com/diegoami/borg-zoo/issues/511):
exact specimen, shipped control, one arm per switch, all together, all nine
classes, and the exact retained 600-start manifest. A separate owner-authorized zoo session ran the initial pilot on
`78b8f297`; no arm led on survival. Its recorded Rogue crow death exposed
a selector bug: weak missiles outranked stronger throws. Fix `64f20bb4`
sorts expected damage first and uses family preference only for ties;
a regression test covers oil versus pebbles. Canonical investigation is
in Cemetery #88. Future specimen `5acb8104` includes that correction and
possible-wake Recall; it is explicitly repinned in Zoo #511 and remains
unmeasured. This session dispatched no run; the owner chooses scheduling. The manifest/census/control audit
are evidence inputs, not a locally played candidate population.

**Left:** owner-selected review and owner-scheduled measurement #511.
Higher-level/class-range and conservative-estimate bounds remain unmeasured;
no shipped adoption or general improvement is claimed. No reviewer was
chosen by this session. No PR is merged without the owner's word.

Raw local artifacts remain `/tmp/T760-fight/`; canonical evidence work is
`/tmp/T760-cemetery` on `task/T760-flight-evidence`, PR #88. Original input
scratchpad remains read-only. A machine transfer was considered, then the
owner chose waiting here; no files were moved. The private replay helper
rejects population inputs and locks each named replay separately.
