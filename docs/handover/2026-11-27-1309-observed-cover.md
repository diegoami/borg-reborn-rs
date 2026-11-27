# Let retained, partly known gear cover spare gear

Issue #1309, stacked on #1308 (PR #1312). Third of four fixes from the
equipment routine audit of `8d85f44f`.

## Found

- Spare-gear cover counted only fully known items. `known_gear` requires
  `ident`, which stays false until every rune and the activation effect
  are known (`borg_item_analyze.rs:392`, `obj-knowledge.c:754-763`).
- So gear the Borg certainly keeps covered nothing: partly known worn
  gear, artifacts whose activation was never tried, non-melee artifacts
  with unknown runes, and items with activations.
- In #1304, Thorongil (worn) and Colannon (carried) left the plain
  Cloak [1,+10] as the regular backup.

## Done

- The retained, partly known items are those that are worn, artifacts,
  carry a known activation, or are staged loadouts. With no known curse
  they enter the ranked plan as observed choices (`retained_cover`,
  `GearChoice::observed`). An observed choice:
  - can take the regular backup's place;
  - covers unresolved armour and melee spares;
  - counts as a better peer in `redundant_copies`.
- The plan never wears, stores or releases an observed choice. A carried
  artifact with an activation therefore stays in the pack for its
  activation.
- `static_gear_covers` takes the observed flag. The `junk_gear_alternatives
  = 0` comparison keeps requiring fully known candidates.

## Review fix (owner, PR #1313)

With `birth_randarts`, a partly known artifact no longer covers through the
retained-cover path, and a partly known artifact weapon no longer takes the
regular backup's place. A random artifact can carry any curse, so the
accepted-risk argument holds only for the standard artifacts. #1311 already
excluded such weapons as peers in `redundant_copies`. The release unit test
`retained_cover_excludes_known_curses_and_randarts` covers the rule. The
fixture cannot change a birth option, so no native test covers it.

## Checked

- Native `ranked-cleanup` cases:
  - a worn partly known artifact cloak and a carried artifact cloak with
    an untried activation release a plain known cloak and a stack of
    unresolved cloaks;
  - the carried artifact is neither stored nor released;
  - a known curse removes the cover.
- The gear and town `engine/tests` modules pass. The shipped smoke suite is
  identical to `smoke-borg-rs`.

## Owner decision recorded

A hidden curse on retained gear is an accepted risk (owner, 2026-11-27).
Calris is the only standard artifact with a curse.

## Left

#1310.
