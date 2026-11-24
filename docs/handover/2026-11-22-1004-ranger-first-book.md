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
- `town_book_lookahead_levels`, compiled fallback 0, shipped at 1 level:
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
  ran on `5ad50d96` with 20 Ranger and Rogue starts, at 0 and 4. The
  re-pin to `af296c62` with a Paladin arm was requested but not run. Both
  arms had zero deaths. Book sales across all books fell from 49 to 30 at 4.
  The report leaves first-book sales before spell eligibility, buyback cost
  and the Paladin arm open in that issue.
- No measurement exists at the shipped value, 1 (second review, finding 1):
  to file as an issue if the owner wants one.
- The smoke suite (a Warrior, a Mage, a Priest and a Kobold Mage) has no
  Ranger, Rogue or Paladin, so its identical play says nothing about this
  change (second review, finding 5).
- Fixed in this PR after the second review: nine parameter citations in
  `borg_town_capability.rs` that the PR's own insertion moved off their
  reads (finding 2), and the shipped value in `docs/CALIBRATION.md`
  (finding 3).

### Ranger follow-up (2026-11-24)

The shared setting remains 1. Rangers use 2 so that their CL3 first
spell counts from CL1. A named replay of Gnome Ranger seed 1895660873
on the default borg-rs core reproduced the Lesser Charms sale at
turn 221 with 1. With 2, there was no sale of Lesser Charms through
100,000 turns. All four smoke runs matched the existing reference.
The population pilot at 4 remains documented in borg-zoo#693.
