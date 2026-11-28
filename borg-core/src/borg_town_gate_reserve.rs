//! borg-reborn-rs (#1417): the Home gate reserve.
//!
//! The shops sell few of the consumables the Borg needs to pass the next
//! depth gates (`borg_prepared`: Scrolls of Teleportation, Potions of
//! Healing and Speed, ...), so a reserve builds only from what the Borg finds
//! or sees in the Black Market, a trip at a time. With `town_home_gate_reserve`
//! it banks them at Home once max depth reaches `town_home_gate_min_depth`:
//!
//! * **deposit**: a carried stack of a reserve kind keeps what the next gate
//!   and the trip need and stores the rest, up to the target, even when it is
//!   not overweight and when Home already holds the kind (`KitItem::gate_reserve`);
//! * **buy**: a Home purchase tops a kind up below its target;
//! * **withdraw**: when the descent target reaches a gate, [`withdraw`] takes
//!   from Home what the pack lacks for it. The gates keep counting the pack.
//!
//! The reserve kinds are the supply group's capabilities
//! (`borg_town_home_budget::Capability`) except the spare book; the Home
//! slots they fill are that group's 8 (#1435). Clear Mind, Emergency, Terror
//! and Cure Critical/Serious Wounds (#1433) are not in it.
use crate::borg_headers::borg_item_h::borg_item;
use crate::borg_headers::borg_trait_h::{BI_MAXDEPTH, BI_MAXSP};
use crate::borg_store::BORG_HOME;
use crate::borg_town_home_budget::{capability_of, copies_of, Capability};
use crate::borg_town_record::action;
use crate::borg_town_shadow::TownAction;
use crate::params::{value as param, Param};
use crate::state::BorgState;

/// The depth gates that ask for a count of a capability, from `borg_prepared`:
/// (the gate's first depth, the count it asks for), in depth order.
pub(crate) fn gates(capability: Capability) -> Vec<(i32, i32)> {
    let band = |depth: Param, count: Param| (param(depth), param(count));
    let mut gates = match capability {
        Capability::Teleport => vec![band(Param::prepared_dl10_min_depth, Param::prepared_dl10_min_teleports),
            band(Param::prepared_dl26_min_depth, Param::prepared_dl26_min_teleports)],
        Capability::Heal => vec![band(Param::prepared_dl46_min_depth, Param::prepared_dl46_min_heal),
            band(Param::prepared_dl56_min_depth, Param::prepared_dl56_min_heal),
            band(Param::prepared_dl100_min_depth, Param::prepared_dl100_min_healing_potions)],
        Capability::BigHeal => vec![band(Param::prepared_dl46_min_depth, Param::prepared_dl46_min_star_heal),
            band(Param::prepared_dl56_min_depth, Param::prepared_dl56_min_star_heal),
            band(Param::prepared_dl100_min_depth, Param::prepared_dl100_min_star_heal_or_life)],
        Capability::Speed => vec![band(Param::prepared_dl100_min_depth, Param::prepared_dl100_min_speed_potions)],
        Capability::Mana => vec![band(Param::prepared_dl100_min_depth, Param::prepared_dl100_min_restore_mana)],
        _ => Vec::new(),
    };
    gates.retain(|&(_, count)| count > 0);
    gates.sort_unstable();
    gates
}

/// What the gates ask the pack for at `depth`: the count of the deepest gate
/// at or above it.
pub(crate) fn requirement(capability: Capability, depth: i32) -> i32 {
    gates(capability).iter().filter(|&&(from, _)| from <= depth).map(|&(_, count)| count).max().unwrap_or(0)
}

/// The copies to hold in all (pack and Home) at `max_depth`: the count of the
/// `lookahead`-th gate past it, or the deepest gate's when fewer lie ahead,
/// at least the count of the gate it has passed. A capability no gate names
/// uses `town_home_reserve_target`.
pub(crate) fn reserve_target_at(gates: &[(i32, i32)], max_depth: i32, lookahead: i32, fallback: i32) -> i32 {
    if gates.is_empty() { return fallback; }
    let passed = gates.iter().filter(|&&(from, _)| from <= max_depth).map(|&(_, count)| count).max().unwrap_or(0);
    let ahead: Vec<i32> = gates.iter().filter(|&&(from, _)| from > max_depth).map(|&(_, count)| count).collect();
    let next = if lookahead <= 0 { None } else { ahead.get(lookahead as usize - 1).or(ahead.last()).copied() };
    next.unwrap_or(passed).max(passed)
}

pub(crate) fn reserve_target(capability: Capability, max_depth: i32) -> i32 {
    reserve_target_at(&gates(capability), max_depth, param(Param::town_home_gate_lookahead),
        param(Param::town_home_reserve_target).max(1))
}

/// The copies a carried stack keeps: what the pack needs, and every copy
/// beyond what Home still wants to reach the target.
pub(crate) fn pack_keep(pack: i32, home: i32, need: i32, target: i32) -> i32 {
    need.max(pack - (target - home).max(0)).clamp(0, pack.max(0))
}

/// The routine is on: the switch, the Home reserve policy, and a max depth at
/// `town_home_gate_min_depth` (hoarding starts well before the gates ask).
pub(crate) fn active(st: &BorgState) -> bool {
    param(Param::town_home_gate_reserve) != 0 && crate::borg_town_home::enabled(st)
        && st.borg.tr(BI_MAXDEPTH) >= param(Param::town_home_gate_min_depth)
}

/// The capability a reserve kind serves; the caster-only ones only for a caster.
pub(crate) fn kind_of(st: &BorgState, item: &borg_item) -> Option<Capability> {
    let capability = capability_of(st, item)?;
    match capability {
        Capability::Book => None,
        Capability::Mana if st.borg.tr(BI_MAXSP) <= 0 => None,
        other => Some(other),
    }
}

/// (copies carried, copies at Home) of a capability.
pub(crate) fn held(st: &BorgState, capability: Capability) -> (i32, i32) {
    let count = |items: &[borg_item]| items.iter().filter(|it| it.iqty > 0 && capability_of(st, it) == Some(capability))
        .map(copies_of).sum::<i32>();
    let pack = crate::borg_item::inven_wield() as usize;
    (count(&st.items.items[..pack]), count(&st.store.shops[BORG_HOME].ware))
}

fn device(item: &borg_item) -> bool {
    use crate::headers::list_tvals_h::{TV_ROD, TV_STAFF, TV_WAND};
    matches!(u32::from(item.tval), TV_STAFF | TV_WAND | TV_ROD)
}

/// The copies (in stack units) a carried stack keeps, for `KitItem::gate_reserve`;
/// `None` when it is not a reserve kind. A device is stored whole or not at all.
pub(crate) fn keep(st: &BorgState, item: &borg_item) -> Option<i32> {
    if !active(st) { return None; }
    let capability = kind_of(st, item)?;
    let depth = st.borg.tr(BI_MAXDEPTH);
    let (pack, home) = held(st, capability);
    let need = requirement(capability, depth + 1);
    let keep = pack_keep(pack, home, need, reserve_target(capability, depth));
    // This visit's withdrawal stays carried.
    if crate::borg_town_live::home_moved(st, item, false) { return Some(i32::from(item.iqty)); }
    Some(if device(item) {
        if keep >= pack { i32::from(item.iqty) } else { 0 }
    } else {
        // `keep` is the stack-wide count; the planner's floor is per stack, and
        // a carried capability may sit in more than one stack: keep it all in
        // the first stack and none of the others' copies beyond what is wanted.
        keep.min(i32::from(item.iqty))
    })
}

/// Whether a Home purchase of this ware (one copy) tops a reserve kind up.
pub(crate) fn wants_more(st: &BorgState, item: &borg_item) -> bool {
    if !active(st) { return false; }
    let Some(capability) = kind_of(st, item) else { return false; };
    let (pack, home) = held(st, capability);
    pack + home < reserve_target(capability, st.borg.tr(BI_MAXDEPTH))
}

/// The copies to take out of one Home stack for a carried count that falls
/// short of a need: no more than the stack holds or Home can hand over.
pub(crate) fn withdraw_copies(need: i32, pack: i32, stack_copies: i32, stack_items: i32, capacity: i32, is_device: bool) -> i32 {
    if need <= pack || stack_copies <= 0 { return 0; }
    if is_device { return stack_items.min(capacity).max(0); }
    (need - pack).min(stack_copies).min(capacity).max(0)
}

/// Before Recall: when the descent target (the next depth below the max)
/// reaches a gate, take from Home what the pack lacks for it. One copy a
/// step is what the executor moves; the planner solves again.
pub(crate) fn withdraw(st: &BorgState) -> Option<TownAction> {
    if !active(st) { return None; }
    let depth = st.borg.tr(BI_MAXDEPTH) + 1;
    for capability in Capability::ALL {
        let need = requirement(capability, depth);
        if need == 0 { continue; }
        let (pack, _) = held(st, capability);
        if pack >= need { continue; }
        let ware = &st.store.shops[BORG_HOME].ware;
        let stock = crate::engine::store_stock(&st.access, BORG_HOME);
        for (slot, item) in ware.iter().enumerate() {
            if item.iqty == 0 || kind_of(st, item) != Some(capability)
                || crate::borg_town_live::home_moved(st, item, true)
                || crate::borg_town_live::buy_rejected(st, BORG_HOME as i16, slot)
                || crate::borg_town_live::shopping_weight_capacity(st, item.weight) <= 0
            { continue; }
            let Some(offer) = stock.get(slot) else { continue; };
            let copies = withdraw_copies(need, pack, copies_of(item), i32::from(item.iqty), offer.carry_capacity(), device(item));
            if copies > 0 {
                return Some(TownAction { code: action::TAKE_HOME, at: [BORG_HOME as i64, slot as i64, 1, 0] });
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_target_follows_the_next_gate() {
        let gates = [(10, 3), (26, 8)];
        assert_eq!(reserve_target_at(&gates, 5, 1, 5), 3, "DL5: stock for the DL10 gate");
        assert_eq!(reserve_target_at(&gates, 5, 2, 5), 8, "two gates ahead");
        assert_eq!(reserve_target_at(&gates, 5, 9, 5), 8, "fewer gates than the lookahead: the deepest");
        assert_eq!(reserve_target_at(&gates, 12, 1, 5), 8, "DL10 passed: the DL26 gate");
        assert_eq!(reserve_target_at(&gates, 30, 1, 5), 8, "every gate passed: the last count");
        assert_eq!(reserve_target_at(&gates, 5, 0, 5), 0, "no lookahead, no gate passed");
        assert_eq!(reserve_target_at(&gates, 12, 0, 5), 3, "the passed gate stays");
        assert_eq!(reserve_target_at(&[], 12, 1, 5), 5, "no gate names it: the parameter");
    }

    #[test]
    fn what_the_gates_ask_at_a_depth() {
        let teleports = requirement(Capability::Teleport, 9);
        assert_eq!(teleports, 0, "below the first gate");
        assert_eq!(requirement(Capability::Teleport, 10), param(Param::prepared_dl10_min_teleports));
        assert_eq!(requirement(Capability::Teleport, 40), param(Param::prepared_dl26_min_teleports));
        assert_eq!(requirement(Capability::Glyph, 100), 0, "no gate names a glyph");
    }

    #[test]
    fn the_pack_keeps_the_need_and_the_copies_home_no_longer_wants() {
        // 5 carried, 0 at Home, a target of 8: Home wants 8, so all 5 may go
        // down to the need of the coming gate.
        assert_eq!(pack_keep(5, 0, 3, 8), 3);
        assert_eq!(pack_keep(5, 0, 0, 8), 0, "no gate reached, no copy kept for it");
        // 5 carried, 6 at Home, target 8: Home wants 2 more, 3 stay carried.
        assert_eq!(pack_keep(5, 6, 0, 8), 3);
        // Home at the target: nothing more goes.
        assert_eq!(pack_keep(5, 8, 0, 8), 5);
        assert_eq!(pack_keep(5, 9, 0, 8), 5);
        assert_eq!(pack_keep(2, 0, 3, 8), 2, "never more than the pack holds");
    }

    #[test]
    fn the_withdrawal_brings_the_pack_up_to_the_gate() {
        // The issue's case: Home 5 Teleportation, pack 3, the DL26 gate asks 8.
        assert_eq!(withdraw_copies(8, 3, 5, 5, 40, false), 5);
        assert_eq!(withdraw_copies(8, 3, 9, 9, 40, false), 5, "only what the pack lacks");
        assert_eq!(withdraw_copies(8, 3, 5, 5, 2, false), 2, "or what the pack can carry");
        assert_eq!(withdraw_copies(8, 8, 5, 5, 40, false), 0, "the gate is met: Home stock is untouched");
        assert_eq!(withdraw_copies(8, 3, 0, 0, 40, false), 0);
        // A Staff of Teleportation is taken whole, its charges counted.
        assert_eq!(withdraw_copies(8, 3, 6, 1, 1, true), 1);
    }
}
