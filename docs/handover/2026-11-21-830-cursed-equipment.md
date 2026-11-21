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

**No scroll for an item not worth one** (owner: "don't spend a Remove Curse
scroll on an item not worth keeping"). The Priest's `Cloak [1,+0]` is worth
3 gold; the scroll costs 100. The read (`borg_decurse_any`), both purchases
(`borg_store_buy.rs`, `borg_town_capability.rs`) and the power bonus for
carrying a scroll all key on `BI_FIRST_CURSED`, and `borg_notice` sets it for
a pack item as well as for worn equipment (`borg_trait.rs`, the inventory
scan). So the gate sits there: a pack item sets it only when its worth
without the curse reaches the scroll's own price (`worth_a_scroll` in
`borg_cursed.rs`: the value of a known item, the kind's price for one only
identified by kind, nothing for an unknown kind; no new constant). The swap
decurse (`borg_decurse_armour`, `borg_decurse_weapon`) has the same test.
Worn equipment is unchanged.

**Checked.** All three cases, each with the zoo's own build as the control
(logs and patches in the borg-cemetery case `studies/cursed-equipment-830`;
single games, not measurements):
- *Warrior 1653223723*, reproduced from the zoo bundle at e8b3acfd (the key
  stream hashes to the cell's `play_sha256`, `messages.log` byte-identical).
  With the take-off applied to that commit, the game is the same until
  68,279 ("Your Robe prevents you teleporting"); the robe comes off at
  68,289 and the Warrior lives to the 100,000-turn limit (the zoo's game died
  to a yellow jelly at 68,399).
- *Ranger 1088421828*, at 3b4924ba: sandals off at 6,671 (the zoo's game
  died at 6,761); alive at 100,000.
- *Priest 354395849*, at 3b4924ba, three builds: the zoo's (reads Remove
  Curse on the pack cloak at 71,751, wears it again, dies at 93,357); the
  take-off alone (reads a scroll on the cloak at 64,884, buys another at
  64,904, "Your Cloak prevents you teleporting" at 83,138, cloak off at
  83,148); the take-off and the gate (no Remove Curse scroll bought or read
  for the cloak, the `{??}` cloak never worn again, alive at 100,000).
- The smoke suite on the branch (at `7225eb02`) is identical to `smoke-borg-rs` (3 of 3), so
  the reference is not re-recorded. fmt, clippy `-D warnings`,
  `cargo test --workspace --exclude borg-core` and
  `cargo test --release -p borg-core` (575) pass;
  `scripts/borg-core-vendor.py check` passes with `borg_cursed.rs` declared as
  ours and `borg_item_decurse.rs`, `borg_item_analyze.rs`, `borg_trait.rs`
  as hooked.

**A bug the gate had, found by the pilot starts, fixed in `7225eb02`.** At
`3be3f186` (this PR as first pushed) the Human Priest 1569794094 of the zoo's
`classes-18` pilot never finished: at turn 58,971 the game clock stopped and
the Borg wrote "# Reading Scroll of Remove Curse." without end (1.75 million
keys in 20 minutes). It wore a steelskin torch and carried a cursed pack
torch. The gate stopped counting the pack torch, and with it dropped the
inventory bit of `BI_WHERE_CURSED`; the game's "Uncurse which item?" prompt
still opens on the pack, and `borg_decurse_any` presses `/` to reach a worn
item only when that bit is set. The bit now follows the item alone and only
the demand (`BI_FIRST_CURSED`) depends on the worth (`pack_curse`, with a
test). The same start now ends at 100,000 turns; four pilot starts at the
base and at the head are in the case (three identical to the base, the Priest
different only from the second scroll on). Why smoke and the three cases did
not show it: none of them has a worn mild curse and a worthless cursed pack
item together. The prompt/bit pairing is older code; a read that fails to
select its item still retries at the same turn without end, and nothing in
this PR guards that. That is a separate issue, not done here.

**Not checked.**
- The trip for Remove Curse ("restock remove curse") is in no trace of any
  run, still. Its decision is unit-tested. The read path for a *worn* item is
  seen now (Priest 1569794094: "The steelskin curse is removed!"), but that
  Borg went to town for other reasons. A live check needs a worn item with a
  mild known curse, which `start-build` cannot construct (it rejects curses,
  harness/README.md) and which a natural start reaches by chance. The zoo
  request counts the "restock remove curse" trips.
- Survival: no measurement. The zoo request is the way (RULES 14, 20).
- The Ranger and Priest runs above are at 3b4924ba with the changes applied
  by hand (the main branch moved since); the Warrior's at e8b3acfd without
  the trip and the gate. They show what the changes do in the zoo's own
  games, not the branch's.

**Decision for the owner.** The gate's test is the item's worth without
its curse against the scroll's price, for items in the pack. It spends no
scroll on a `[1,+0]` cloak and still spends one on an item worth 100 gold or
more. A worn item with a known removable curse is unchanged: it is what the
trip check and the town purchase serve. If a different test is wanted (the
item's power contribution, say), it is one function, `worth_a_scroll`.

**Evidence** is in borg-cemetery (AGENTS.md): `studies/cursed-equipment-830`.
