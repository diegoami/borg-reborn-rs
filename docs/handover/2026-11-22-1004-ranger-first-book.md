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
  - the look-ahead uses the spell level alone, so it can keep books with spells above current max SP;
  - shop books are valued as before.
- A unit test.
- Results:
  - smoke-borg-rs is 3/3 identical (not re-recorded);
  - 1895660873 plays `31c994254ecdb188` at 0;
  - with 4, no first book is sold early in 7 starts. The sales left are duplicates and other classes'
    books.
- Study: borg-cemetery#132. Details: `docs/CALIBRATION.md` ("#1004").

### Review
MiniMax MCode reviewed `0de39a27`: pass with findings.
- **Fixed:**
  - F2: the citation points to the parameter read;
  - F3: `book_lookahead` is unit-tested, shop scopes 0..=7 get no look-ahead;
  - F4: Paladins of three races have 1 SP at CL1, below Bless's 2, and are
    affected too;
  - F5: the bound is stated;
  - F6: the numbers are marked as local direction, and the SP table is in
    borg-cemetery#132;
  - F8: the zoo request is linked here.
- **Handled:** F1, by re-pinning the zoo request to the new head with
  Paladins added.
- **The owner's call:** F7 (model trailers on the first two commits;
  removing them means rewriting the branch) and F9 (the branch name).

### Left
- The zoo pilot in [borg-zoo#693](https://github.com/diegoami/borg-zoo/issues/693)
  ran on `5ad50d96` with 20 Ranger and Rogue starts. Both arms had zero
  deaths. Book sales across all books fell from 49 to 30 at 4. The report
  leaves first-book sales before spell eligibility, buyback cost and the
  Paladin arm open in that issue.

### For the owner
Should the switch ship as 4 after the zoo reads it? 4 is the largest gap between a class's first level
and its first spell (Rogue: CL5 − CL1).
