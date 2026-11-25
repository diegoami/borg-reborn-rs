# Trim surplus enchanted ammunition in town

The town planner now considers ordinary enchanted ammunition after the
existing surplus pass. It uses `town_quiver_trim_reserve` and
`junk_exploration_first`. The fallback keeps the usable launcher reserve
and excludes ego, artifact, cursed, unknown and annotated stacks. It
uses Angband's total quiver units to check whether a drop opens a pack
slot. Successful trims suppress reacquisition of that kind on the
same town visit.

The private A2 save linked in [#1269](https://github.com/diegoami/borg-reborn-rs/issues/1269)
starts with 59 pebbles, including a Slay Animal stack. The saved settings
replay first trimmed from 58 to 45 usable shots under weight pressure.
The native count stayed at 10 free pack slots. With the departure slot
target raised to 12 for a separate replay, the first trim went from 59 to
34 usable shots and from six to seven free pack slots. Both replays kept
at least 30 usable shots after each enchanted trim. The unit test raises
the reserve to 35 and checks that the slot-producing drop is declined.

Run `python3 engine/tests/check_enchanted_quiver.py /path/to/extracted-a2`
with a normal harness build. CI runs the selector unit test without the
private save.
