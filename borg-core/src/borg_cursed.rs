//! Cursed equipment (#830): never live with a harmful curse, and when the
//! Borg keeps a cursed item, go to town for Remove Curse.
//!
//! Angband 4.2 curses do not stick: any worn item can be taken off unless it
//! has `STICKY` (`engine/angband/src/obj-util.c:obj_can_takeoff`, line 796),
//! so one "take off" ends every effect of every curse on the item. The
//! power model already prices curses (`borg_power.rs`, the `crs*` weights),
//! but its take-off check, `borg_remove_stuff`, runs after the fight and
//! caution branches of `borg_think_dungeon`, and a curse that aggravates
//! keeps monsters in view, so it never ran (Ranger 1088421828: annoyance
//! boots, -10 speed, dead 110 game turns after putting them on).

use crate::borg_headers::borg_item_h::{
    borg_item, BORG_CURSE_ANNOYANCE, BORG_CURSE_ANTI_TELEPORTATION, BORG_CURSE_COWARDICE,
    BORG_CURSE_IMPAIR_HITPOINT_RECOVERY, BORG_CURSE_PARALYSIS,
};
use crate::borg_headers::borg_trait_h::{BI_CDEPTH, BI_FIRST_CURSED, BI_GOLD, BI_WHERE_CURSED, BORG_EQUIP};
use crate::borg_io::{borg_keypress, borg_note_bytes};
use crate::borg_item::{inven_wield, quiver_start};
use crate::borg_item_analyze::borg_object_value_known;
use crate::borg_item_decurse::can_decurse;
use crate::engine;
use crate::headers::list_object_flags_h::OF_STICKY;
use crate::headers::ui_event_h::keycode_t;
use crate::state::BorgState;

/// Curses the Borg does not wear at any power, with what each costs
/// (`engine/angband/lib/gamedata/curse.txt`): annoyance is `SPEED[-10]`,
/// `STEALTH[-10]` and `AGGRAVATE`; anti-teleportation is `NO_TELEPORT`
/// (it also stops Phase Door, #829); paralysis can freeze the player;
/// impair hitpoint recovery is `IMPAIR_HP`; cowardice is `AFRAID`. This is
/// the owner's list (#830, 2026-11-21) and nothing more: slowness
/// (`SPEED[-5]`) was tried and left out (owner, 2026-11-21), as a good item
/// may beat bare skin. Every other curse stays with the power model's prices
/// and `borg_remove_stuff`.
const HARMFUL: [(u32, &str); 5] = [
    (BORG_CURSE_ANNOYANCE, "annoyance: -10 speed, aggravate"),
    (BORG_CURSE_ANTI_TELEPORTATION, "anti-teleportation: no teleport, no Phase Door"),
    (BORG_CURSE_PARALYSIS, "paralysis"),
    (BORG_CURSE_IMPAIR_HITPOINT_RECOVERY, "impair hitpoint recovery"),
    (BORG_CURSE_COWARDICE, "cowardice"),
];

/// The first harmful curse an item is known to carry, as text for the note.
/// Only what the Borg has learned counts: `borg_item.curses` is read from
/// the player's knowledge of the object (a `{??}` item shows none).
pub(crate) fn harmful_curse(item: &borg_item) -> Option<&'static str> {
    if item.iqty == 0 || !item.cursed {
        return None;
    }
    HARMFUL.iter().find(|(k, _)| item.curses[*k as usize]).map(|&(_, why)| why)
}

/// What `borg_remove_cursed` has tried, to give up on an item that stays on
/// (a take-off the game refuses must not become a loop). The count is kept
/// per equipment slot, so a slot that will not come off does not hide the
/// others; per item, so a different item that arrives in the slot (the
/// emergency swap replaces one in place) starts afresh; and per level.
#[derive(Default)]
pub(crate) struct Removal {
    depth: i32,
    /// Take-offs tried, by equipment slot (`slot - inven_wield()`), with the
    /// item they were spent on (`None`: no candidate in the slot).
    tries: Vec<(u8, Option<Identity>)>,
}

/// What tells one worn item from another for the count: its kind, artifact,
/// ego and the curses it is known to carry. The description does not do it:
/// the game writes only `{cursed}` in it, never which curse (`obj-desc.c`),
/// so two cloaks with the same modifiers look alike whatever they carry.
type Identity = (u32, u8, u8, [bool; 28]);

fn identity(item: &borg_item) -> Identity {
    (item.kind, item.art_idx, item.ego_idx, item.curses)
}

/// Attempts at one slot before the Borg stops trying it.
const MAX_TRIES: u8 = 3;

/// Whether an equipment item is a take-off candidate: a harmful curse that
/// can come off. `The One Ring` is left to its own handling and a `STICKY`
/// item cannot be taken off.
fn candidate(item: &borg_item) -> bool {
    !item.one_ring
        && harmful_curse(item).is_some()
        && !engine::flag_has(&item.flags, OF_STICKY as i32)
}

impl Removal {
    /// Forget what no longer applies: everything on a new level, and the
    /// count of a slot that holds no candidate any more (the take-off worked)
    /// or another item than the one the count was spent on. A curse learned
    /// later changes the item's identity and gives it a few more tries, a
    /// bounded number (one per curse). `items[first..end]` is the equipment.
    fn sync(&mut self, depth: i32, items: &[borg_item], first: usize, end: usize) {
        if self.depth != depth {
            *self = Removal { depth, tries: Vec::new() };
        }
        let end = end.min(items.len());
        self.tries.resize(end.saturating_sub(first), (0, None));
        for (n, entry) in self.tries.iter_mut().enumerate() {
            let item = &items[first + n];
            let now = candidate(item).then(|| identity(item));
            if entry.1 != now {
                *entry = (0, now);
            }
        }
    }

    fn tried(&self, slot: usize, first: usize) -> u8 {
        self.tries.get(slot - first).map_or(0, |entry| entry.0)
    }
}

/// The slot to take off: the first equipment slot (indexes into `items`,
/// `first..end`) that is a candidate and has not used up its tries.
fn pick(items: &[borg_item], first: usize, end: usize, removal: &Removal) -> Option<usize> {
    (first..end.min(items.len())).find(|&i| candidate(&items[i]) && removal.tried(i, first) < MAX_TRIES)
}

/// The bytes of a C `desc` buffer, up to its NUL.
fn desc(item: &borg_item) -> Vec<u8> {
    item.desc.iter().take_while(|&&c| c != 0).map(|&c| c as u8).collect()
}

/// Take off a worn item with a harmful curse, leaving the slot empty
/// (the engine puts it in the pack). It runs before the fight and caution
/// branches: a curse that aggravates, slows or blocks teleportation is
/// worth one turn at once. The item is then worthless to the Borg (every
/// cursed item is, `borg_item_analyze.rs`), so no wear routine puts it
/// back on.
pub(crate) fn borg_remove_cursed(st: &mut BorgState) -> bool {
    let first = inven_wield();
    let end = quiver_start();

    let depth = st.borg.tr(BI_CDEPTH);
    let removal = &mut st.think_dungeon.cursed;
    removal.sync(depth, &st.items.items, first as usize, end as usize);

    /* a slot that stays occupied after repeated tries is left alone, and the
     * others are still looked at */
    let Some(i) = pick(&st.items.items, first as usize, end as usize, removal) else {
        return false;
    };
    let slot = i as i32;
    removal.tries[i - first as usize].0 += 1;

    let item = st.items.items[i];
    let why = harmful_curse(&item).unwrap_or("");
    let mut note = b"# Removing cursed ".to_vec();
    note.extend_from_slice(&desc(&item));
    note.extend_from_slice(format!(" ({why})").as_bytes());
    borg_note_bytes(st, &note);

    /* `borg_remove_stuff`'s take-off: `t` and the equipment letter */
    borg_keypress(st, keycode_t::from(b't'));
    borg_keypress(st, engine::all_letters_nohjkl(slot - first) as i8 as keycode_t);
    true
}

/// Whether the Borg must go to town for Remove Curse: it wears an item with
/// a removable curse, has no way to remove it (no scroll, staff, spell or
/// activation), and can pay for a scroll. In the dungeon only. This is the
/// case where it keeps the item: `borg_remove_cursed` has already taken off
/// every harmful one it can, so what is left is a mild curse on an item the
/// power model prefers to bare skin, or a curse the take-off cannot end.
/// In town the shopping round buys the scroll and `borg_enchanting` reads it
/// (`borg_store_buy.rs`: Remove Curse is acceptable while `BI_FIRST_CURSED`
/// is set).
pub(crate) fn needs_remove_curse(st: &mut BorgState) -> bool {
    let kind = st.item_val.kv_scroll_remove_curse;
    let price = usize::try_from(kind).ok().map_or(0, engine::kind_cost_of);
    let (depth, first, wh, gold) = (
        st.borg.tr(BI_CDEPTH),
        st.borg.tr(BI_FIRST_CURSED),
        st.borg.tr(BI_WHERE_CURSED),
        st.borg.tr(BI_GOLD),
    );
    /* the cheap tests first: `can_decurse` looks through the pack */
    if !worn_cursed(depth, first, wh) {
        return false;
    }
    needs_remove_curse_with(can_decurse(st), gold, price)
}

/// A removable curse on worn equipment, in the dungeon (a cursed arrow in
/// the quiver is not worth a trip).
fn worn_cursed(depth: i32, first_cursed: i32, where_cursed: i32) -> bool {
    depth != 0 && first_cursed != 0 && where_cursed & BORG_EQUIP as i32 != 0
}

/// The rest of the decision, once its inputs are known: no means to remove
/// a curse in hand, and the purse covers the scroll's base price. The
/// Alchemist always stocks it at exactly that price (`store.txt`: `always`
/// Remove Curse, cost 100), so that is the price that binds; the Black
/// Market would charge more (`store.c`, 150%) and is not counted on. No trip
/// is made for a scroll the Borg cannot pay for.
fn needs_remove_curse_with(can_decurse: bool, gold: i32, price: i32) -> bool {
    !can_decurse && price > 0 && gold >= price
}

/// Whether a pack item is worth a Remove Curse scroll, for the swap decurse
/// (`borg_decurse_armour`, `borg_decurse_weapon`): what it would be worth
/// without its curse must reach the scroll's own price. A cursed item is
/// worth 0 to the Borg (`borg_item_analyze.rs`), so its worth is taken
/// again here: the value of a known item, the kind's price for one only
/// identified by kind, nothing for an unknown kind. A `[1,+0]` cloak is worth
/// 3 gold, not 100 (Priest 354395849 read a scroll on one and wore it again,
/// with a second curse still on it).
pub(crate) fn worth_a_scroll(st: &BorgState, i: i32) -> bool {
    let kind = st.item_val.kv_scroll_remove_curse;
    let price = usize::try_from(kind).ok().map_or(0, engine::kind_cost_of);
    let item = &st.items.items[i as usize];
    let worth = if item.ident {
        borg_object_value_known(&st.item_val, item)
    } else if item.aware {
        engine::kind_cost_of(item.kind as usize)
    } else {
        0
    };
    worth_a_scroll_with(worth, price)
}

/// The test of `worth_a_scroll`, once its inputs are known. With no known
/// price for the scroll the old behaviour stays.
fn worth_a_scroll_with(worth: i32, price: i32) -> bool {
    price <= 0 || worth >= price
}

/// What an uncursable pack item adds to the curse traits, as `(sets the
/// inventory bit of BI_WHERE_CURSED, may become BI_FIRST_CURSED)`. The bit
/// follows the item alone: it mirrors the game's "Uncurse which item?"
/// prompt, which opens on the pack while any pack item qualifies, and
/// `borg_decurse_any` presses `/` to reach a worn one only when the bit is
/// set. Dropping it for an item not worth a scroll made the Borg type the
/// equipment letter into the pack list and retry it at the same game turn
/// for good (Human Priest 1569794094, seen live). Only the demand depends on
/// the worth: such an item must not be the one a scroll is bought and read for.
pub(crate) fn pack_curse(uncursable: bool, worth_a_scroll: bool) -> (bool, bool) {
    (uncursable, uncursable && worth_a_scroll)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::borg_headers::borg_item_h::BORG_CURSE_AIR_SWING;
    use crate::borg_headers::borg_trait_h::{BORG_INVEN, BORG_QUILL};
    use crate::borg_item::blank_item;

    fn cursed(curse: u32) -> borg_item {
        let mut item = blank_item();
        item.iqty = 1;
        item.cursed = true;
        item.curses[curse as usize] = true;
        item
    }

    #[test]
    fn the_owners_curses_are_harmful_at_any_power() {
        for curse in [
            BORG_CURSE_ANNOYANCE,
            BORG_CURSE_ANTI_TELEPORTATION,
            BORG_CURSE_PARALYSIS,
            BORG_CURSE_IMPAIR_HITPOINT_RECOVERY,
            BORG_CURSE_COWARDICE,
        ] {
            assert!(harmful_curse(&cursed(curse)).is_some(), "curse {curse}");
        }
        assert!(harmful_curse(&cursed(BORG_CURSE_AIR_SWING)).is_none());
    }

    #[test]
    fn an_empty_or_uncursed_slot_is_not_harmful() {
        let mut item = cursed(BORG_CURSE_ANNOYANCE);
        item.iqty = 0;
        assert!(harmful_curse(&item).is_none());
        let mut item = cursed(BORG_CURSE_ANNOYANCE);
        item.cursed = false;
        assert!(harmful_curse(&item).is_none());
    }

    #[test]
    fn the_first_harmful_equipment_slot_is_taken_off() {
        let mut items = vec![blank_item(); 8];
        items[1] = cursed(BORG_CURSE_AIR_SWING);
        items[3] = cursed(BORG_CURSE_ANTI_TELEPORTATION);
        items[5] = cursed(BORG_CURSE_ANNOYANCE);
        let none = Removal::default();
        assert_eq!(pick(&items, 2, 8, &none), Some(3));
        assert_eq!(pick(&items, 4, 8, &none), Some(5));
        /* the pack (slots before `first`) is never taken off */
        assert_eq!(pick(&items, 6, 8, &none), None);
    }

    /// One turn of `borg_remove_cursed` on `items[first..end]`, take-off
    /// refused: the slot it would try, if any.
    fn turn(removal: &mut Removal, depth: i32, items: &[borg_item], first: usize, end: usize) -> Option<usize> {
        removal.sync(depth, items, first, end);
        let i = pick(items, first, end, removal)?;
        removal.tries[i - first].0 += 1;
        Some(i)
    }

    #[test]
    fn a_slot_that_will_not_come_off_does_not_hide_the_others() {
        /* a cloak and boots, both harmful; the take-off of the cloak fails */
        let mut items = vec![blank_item(); 8];
        items[3] = cursed(BORG_CURSE_ANTI_TELEPORTATION);
        items[5] = cursed(BORG_CURSE_ANNOYANCE);
        let mut removal = Removal::default();
        let tried: Vec<_> = (0..6).map(|_| turn(&mut removal, 5, &items, 2, 8)).collect();
        /* three tries on the cloak, then the boots get theirs, then nothing */
        assert_eq!(tried, [Some(3), Some(3), Some(3), Some(5), Some(5), Some(5)]);
        assert_eq!(turn(&mut removal, 5, &items, 2, 8), None);
    }

    #[test]
    fn a_new_level_starts_the_counts_afresh() {
        let mut items = vec![blank_item(); 8];
        items[5] = cursed(BORG_CURSE_ANNOYANCE);
        let mut removal = Removal::default();
        for _ in 0..MAX_TRIES {
            assert_eq!(turn(&mut removal, 5, &items, 2, 8), Some(5));
        }
        assert_eq!(turn(&mut removal, 5, &items, 2, 8), None);
        /* DL9: the same slot, tried again */
        assert_eq!(turn(&mut removal, 9, &items, 2, 8), Some(5));
    }

    #[test]
    fn a_slot_that_no_longer_holds_a_candidate_is_forgotten() {
        let mut items = vec![blank_item(); 8];
        items[5] = cursed(BORG_CURSE_ANNOYANCE);
        let mut removal = Removal::default();
        for _ in 0..MAX_TRIES {
            turn(&mut removal, 5, &items, 2, 8);
        }
        assert_eq!(turn(&mut removal, 5, &items, 2, 8), None);
        /* the take-off worked (slot empty), then another cursed item is worn */
        items[5] = blank_item();
        assert_eq!(turn(&mut removal, 5, &items, 2, 8), None);
        items[5] = cursed(BORG_CURSE_PARALYSIS);
        assert_eq!(turn(&mut removal, 5, &items, 2, 8), Some(5));
    }

    #[test]
    fn a_different_item_in_an_exhausted_slot_is_tried() {
        /* two cloaks of one kind with the same modifiers, so with one
         * description, `Cloak [1,+0] {cursed}`; only the curse differs */
        let mut items = vec![blank_item(); 8];
        items[5] = cursed(BORG_CURSE_ANTI_TELEPORTATION);
        items[5].kind = 40;
        let mut removal = Removal::default();
        for _ in 0..MAX_TRIES {
            assert_eq!(turn(&mut removal, 5, &items, 2, 8), Some(5));
        }
        assert_eq!(turn(&mut removal, 5, &items, 2, 8), None);
        /* the emergency swap puts the other cloak into the same slot */
        items[5] = cursed(BORG_CURSE_PARALYSIS);
        items[5].kind = 40;
        assert_eq!(turn(&mut removal, 5, &items, 2, 8), Some(5));
        /* the same curse on another kind of item is another item too */
        for _ in 1..MAX_TRIES {
            turn(&mut removal, 5, &items, 2, 8);
        }
        assert_eq!(turn(&mut removal, 5, &items, 2, 8), None);
        items[5].kind = 41;
        assert_eq!(turn(&mut removal, 5, &items, 2, 8), Some(5));
        /* the same item stays counted */
        for _ in 1..MAX_TRIES {
            turn(&mut removal, 5, &items, 2, 8);
        }
        assert_eq!(turn(&mut removal, 5, &items, 2, 8), None);
    }

    #[test]
    fn the_one_ring_is_left_alone() {
        let mut items = vec![blank_item(); 4];
        items[2] = cursed(BORG_CURSE_ANNOYANCE);
        items[2].one_ring = true;
        assert_eq!(pick(&items, 0, 4, &Removal::default()), None);
    }

    #[test]
    fn only_worn_equipment_in_the_dungeon_is_worth_a_trip() {
        let eq = BORG_EQUIP as i32;
        assert!(worn_cursed(1, 7, eq));
        /* the bit set may name the quiver as well */
        assert!(worn_cursed(1, 7, eq | BORG_QUILL as i32));
        /* in town the shopping round already buys it */
        assert!(!worn_cursed(0, 7, eq));
        /* nothing cursed */
        assert!(!worn_cursed(1, 0, 0));
        /* a cursed pack item is not worn (`BORG_INVEN` and `BORG_QUILL` only) */
        assert!(!worn_cursed(1, 7, BORG_INVEN as i32 | BORG_QUILL as i32));
    }

    #[test]
    fn an_item_not_worth_a_scroll_still_marks_the_pack_for_the_prompt() {
        /* worth a scroll: marks the pack and may be the first cursed item */
        assert_eq!(pack_curse(true, true), (true, true));
        /* not worth one: the pack bit stays (the game's prompt opens on the
         * pack), the demand goes */
        assert_eq!(pack_curse(true, false), (true, false));
        /* nothing to uncurse */
        assert_eq!(pack_curse(false, true), (false, false));
        assert_eq!(pack_curse(false, false), (false, false));
    }

    #[test]
    fn a_scroll_is_for_an_item_worth_its_price() {
        /* a plain cloak (3 gold) against a 100-gold scroll */
        assert!(!worth_a_scroll_with(3, 100));
        assert!(!worth_a_scroll_with(99, 100));
        assert!(worth_a_scroll_with(100, 100));
        assert!(worth_a_scroll_with(2500, 100));
        /* the scroll's price unknown: unchanged */
        assert!(worth_a_scroll_with(3, 0));
    }

    #[test]
    fn a_trip_for_remove_curse_needs_no_other_way_and_the_gold() {
        assert!(needs_remove_curse_with(false, 100, 100));
        assert!(needs_remove_curse_with(false, 500, 100));
        /* cannot pay */
        assert!(!needs_remove_curse_with(false, 99, 100));
        /* a scroll, staff, spell or activation is in hand */
        assert!(!needs_remove_curse_with(true, 500, 100));
        /* the scroll kind is unknown to the Borg */
        assert!(!needs_remove_curse_with(false, 500, 0));
    }
}
