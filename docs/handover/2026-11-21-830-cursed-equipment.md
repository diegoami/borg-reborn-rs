## 2026-11-21: Cursed equipment comes off at once (#830)

The borg-rs core now takes off a worn item as soon as it is known to carry
a harmful curse, before any fight or flight, and leaves the slot empty. The
harmful curses are the owner's list (annoyance, anti-teleportation,
paralysis, impair hitpoint recovery, cowardice) and slowness, which I added
because it is the same loss as annoyance's speed penalty. The note reads
`# Removing cursed <item> (annoyance: -10 speed, aggravate)`. `borg_cursed.rs`
holds it; `borg_think_dungeon.rs` calls it right after `borg_notice`.

**Why the Ranger kept the boots.** The power model already priced the curses
(`borg_power.rs`) and `borg_remove_stuff` already evaluated an empty slot.
But that check runs after the fight and caution branches, and an
aggravating curse keeps monsters awake and in view, so it never ran: the
Ranger acted on fight and flight for its last 110 game turns. Nothing else in
the wear code puts a known cursed item on: every cursed item is worth 0
to the Borg (`borg_item_analyze.rs`), and `borg_wear_stuff`, `borg_wear_rings`
and `borg_best_stuff` skip worth-0 items. The emergency swap
(`borg_backup_swap`) did not, so it now refuses a known harmful curse too.
A `{??}` item cannot be avoided before its curse shows (the issue's
Priest and Warrior).

**The trip for Remove Curse.** In the dungeon, `borg_must_return_to_town`
now answers "restock remove curse" when worn equipment carries a removable
curse, the Borg has no scroll, staff, spell or activation that removes it, and
its gold covers the scroll's base price (100). That is the case where it keeps
the item. Everything harmful has been taken off already, so this is a mild
curse on an item the power model prefers worn. In town the existing shopping
round buys the scroll (`borg_store_buy.rs`) and `borg_enchanting` reads it.
The gold test is there so a Borg with no money does not walk to town and back.

**Checked.** The three cases at the zoo's commits, then the branch (main
plus the fix) on the same seeds; see the PR for the table. Ranger
1088421828 and Priest 354395849 reproduced the zoo's `messages.log` lines
exactly at 3b4924ba with the shipped settings. The Warrior did not: its zoo
cell ran at e8b3acfd with cell settings that live in the bundle's `user/`
files, which this session could not download (the artifact needs API access to
borg-zoo, and the attach request was refused). So the Warrior is **not** rerun;
the Priest's cloak is the same case (anti-teleportation on body armour).
Unit tests cover the curse classification, the slot choice and the trip's
decision. The smoke suite is identical to `smoke-borg-rs`, so the reference
is not re-recorded. `scripts/borg-core-vendor.py check` passes with
`borg_cursed.rs` declared as ours and `borg_item_decurse.rs` as hooked.

**Not checked.**
- The trip for Remove Curse has no live game that shows it. With the
  take-off disabled in a scratch build, the Ranger and the Priest died in the
  fight that the curse caused, before the restock check (it runs after the fight
  and caution branches); no case keeps a harmful curse alive long enough. Its
  decision is unit-tested; the purchase and the reading are the existing
  paths, seen in the Ranger and Priest runs on items the Borg had taken off.
- No measurement of the effect on survival: that is a zoo request
  (RULES 14, 20), not a few local seeds.

**Found, not changed.**
- After a take-off, the existing swap-decurse (`borg_decurse_armour`) reads
  Remove Curse on the item in the pack and wears it again. The Priest did
  this with the cloak (read at 71,751, worn again `{??}` at 71,761, the
  second curse at 93,200). It is now taken off again, one scroll per round,
  so the loop ends when the curses do; whether a worth-0 cloak is worth 100
  gold of scroll is a decision for the owner.
- Gameplay evidence for this task belongs in borg-cemetery (AGENTS.md); this
  session could not push there.
