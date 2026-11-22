## 2026-11-22: Keep a first spellbook a few levels before its first spell (#1004)

**Core:** every check ran on the default **borg-rs** core. None ran on the reborn core.

### Found
- The town planner (`borg_town_capability.rs`) supplies a class book only while one of its spells is
  castable now: `slevel` at most the level, `smana` at most the max SP.
- The Ranger's first spell is at CL3 and the Rogue's at CL5 (`class.txt`). Their first books are worth
  nothing at CL1, and the first visit sells them for no gold.
- Short starts (seeds 11, 22, 33) at today's setting: Rogues and Rangers sold their first book 6 of 6
  times, as did the #985 Ranger 1895660873. The other seven classes sold it 0 of 21 times.

### Done
- `town_book_lookahead_levels`, compiled and shipped 0 (today's play):
  - a book the Borg carries or keeps at home also counts when a spell of it is learnt within this many
    levels;
  - its mana is not asked, since the SP grow with the level;
  - shop books are valued as before.
- A unit test.
- Results:
  - smoke-borg-rs is 3/3 identical (not re-recorded);
  - 1895660873 plays `31c994254ecdb188` at 0;
  - with 4, no first book is sold early in 7 starts. The sales left are duplicates and other classes'
    books.
- Study: borg-cemetery#132. Details: `docs/CALIBRATION.md` ("#1004").

### Left
- The zoo request (a pilot on Rogues and Rangers).

### For the owner
Should the switch ship as 4 after the zoo reads it? 4 is the largest gap between a class's first level
and its first spell (Rogue: CL5 − CL1).
