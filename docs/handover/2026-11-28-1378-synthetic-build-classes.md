## 2026-11-28: the synthetic build applies a record for any class (#1378)

## Done

`synthetic_build_apply` (`engine/frontend/synthetic-build.c`) no longer
requires a Human Warrior. It requires the record's race and class to be the
ones the harness started (`-race`, `-class`), and checks the record's maximum
mana against the native calculation, as it already checked hit points
(`player->msp != vitals[3]`). Everything else is unchanged: levels 1..15,
mundane base-kind items, the experience and hit-point progression checks, no
artifacts, egos or curses. Existing Warrior records have the same bytes, hash
and behaviour. Capture from play (`synthetic_build_capture`) stays Human
Warrior only: a record holds no spell knowledge, so a caster's capture would
be incomplete without saying so.

Two fixtures, `engine/tests/fixtures/ranger-level5.build` (Dexterity 18/50,
the stats of #1371's Nimbleness case; 57 HP, 6 mana) and
`necromancer-level5.build` (40 HP, 12 mana). The mana and HP were found by
trying values the engine accepted, not copied from play. They carry the
Warrior's kit.

`test_character_build.py::test_a_record_names_the_class_the_run_started`
applies both, and checks that a record for another class, or with another
mana value, is rejected and never falls back to birth.

## Found

- Spell knowledge is not restored: a caster applied from a record has its mana
  but no learned spells and no spellbook of its own.
- This is what lets #1371 (a Ranger drinks Nimbleness) and #1375 (a Necromancer
  refuses a Light scroll and a torch) get native cases. Those cases go in
  their own PRs.

## Not checked

- No class other than the Ranger and the Necromancer was tried.
