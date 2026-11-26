# 2026-11-23 · Pending poison in tactical decisions (#1083)

PR #1107 uses the engine's running poison timer and effective CON index,
following the Phase experiment on `review/track-p-poison-followups-r3`.
This accounts for poison from any source, recovery ticks already applied,
and CON changes from effects such as sickliness or shapechanging.

`escape_phase_lost_poison_pending` compiles and ships at 0. It supplies a
shared poison budget to the Phase fight verdict, newcomer wait, unseen
landing emergency threshold, outmatched cure checks, and danger-based
fleeing. Phase still requires an adjacent awake enemy for its fight verdict.
Known landing hazards and the immediate reply a combat cure must cover
are evaluated separately.

The [Cemetery Ranger study](https://github.com/diegoami/borg-cemetery/tree/main/studies/fastfrag-white-jelly-829474344)
retains the original 829474344 incident. The original PR's survival was
measured on a historical build with the escape-first override set to 75.
That result is specific to those inputs. The switch remains a hypothesis
for a paired Zoo comparison in the jelly worlds of
[borg-zoo#712](https://github.com/diegoami/borg-zoo/issues/712).

The native fixture in `engine/tests/test_combat_heal.py` checks the live
engine timer with the switch off and on. At 23 HP, danger 8, and 15 HP of
pending poison, the off arm stays to fight and the on arm reads Phase Door.
The shipped smoke matches the reference in all four games. Current Track P
replays of the Ranger and Elf Druid 799968561 (in poison-x8) reach the cap
with identical keys in both arms and without being poisoned.
