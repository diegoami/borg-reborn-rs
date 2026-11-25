//! #1241: classify Home stock using facts already known to the player.
use crate::borg_headers::borg_item_h::borg_item;
use crate::borg_headers::borg_magic_h::{BLESS, CALL_LIGHT, HEROISM, LIGHT_ROOM};
use crate::borg_headers::borg_trait_h::{BI_CDEPTH, BI_CLEVEL};
use crate::headers::list_tvals_h::*;
use crate::params::{value as param, Param};
use crate::state::BorgState;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
#[repr(u8)]
pub enum Role {
    #[default]
    Ordinary,
    Common,
    Renewable,
    Reject,
    Gear,
    Device,
}

impl Role {
    pub(crate) fn from_code(code: i64) -> Self {
        match code {
            1 => Self::Common,
            2 => Self::Renewable,
            3 => Self::Reject,
            4 => Self::Gear,
            5 => Self::Device,
            _ => Self::Ordinary,
        }
    }

    pub(crate) fn common(self) -> bool {
        matches!(self, Self::Common | Self::Renewable)
    }

    pub(crate) fn reserve(self) -> bool {
        matches!(self, Self::Gear | Self::Device)
    }
}

pub(crate) fn enabled(st: &BorgState) -> bool {
    crate::borg_town_live::active(st) && param(Param::town_home_reserve) != 0
}

fn consumable(st: &BorgState, item: &borg_item) -> Role {
    if !item.aware || !item.flavour_aware {
        return Role::Ordinary;
    }
    let v = &st.item_val;
    let sval = i32::from(item.sval);
    let spell = match u32::from(item.tval) {
        TV_SCROLL if sval == v.sv_scroll_light => [LIGHT_ROOM, CALL_LIGHT].as_slice(),
        TV_SCROLL if sval == v.sv_scroll_blessing => [BLESS].as_slice(),
        TV_POTION if sval == v.sv_potion_heroism || sval == v.sv_potion_boldness => {
            [HEROISM].as_slice()
        }
        _ => return Role::Ordinary,
    };
    if spell
        .iter()
        .any(|&spell| crate::borg_magic::borg_spell_legal_fail(st, spell, 40))
    {
        Role::Renewable
    } else {
        Role::Common
    }
}

pub(crate) fn device(st: &BorgState, item: &borg_item) -> bool {
    if item.kind == 0
        || !item.aware
        || item.cursed
        || !matches!(u32::from(item.tval), TV_ROD | TV_WAND | TV_STAFF)
        || u32::from(item.tval) != TV_ROD && item.pval <= 0
        || !crate::borg_item_use::ranged_device_reliable(st, item)
    {
        return false;
    }
    let effects = crate::engine::kind_effect_values(item.kind as usize, st.borg.tr(BI_CLEVEL));
    !crate::borg_town_capability::effect_supply_for_consumer(
        &effects,
        100,
        1000,
        u32::from(item.tval) == TV_WAND
            && i32::from(item.sval) == st.item_val.sv_wand_confuse_monster,
    )
    .is_empty()
}

pub(crate) fn gear_slot(item: &borg_item) -> Option<usize> {
    use crate::borg_inventory::borg_wield_slot;
    if !item.aware || matches!(u32::from(item.tval), TV_LIGHT | TV_DIGGING) {
        return None;
    }
    usize::try_from(borg_wield_slot(item)).ok()
}

pub(crate) fn known_special(item: &borg_item) -> bool {
    item.art_idx != 0
        || item.ego_idx != 0
        || item.activ_idx != 0
        || item.modifiers.iter().any(|&m| m > 0)
        || item.el_info.iter().any(|element| element.res_level > 0)
        || item.brands.iter().any(|&brand| brand)
        || item.slays.iter().any(|&slay| slay > 0)
}

fn damage(item: &borg_item) -> i32 {
    i32::from(item.dd) * (i32::from(item.ds) + 1) + 2 * i32::from(item.to_d)
}

fn armor(item: &borg_item) -> i32 {
    i32::from(item.ac) + i32::from(item.to_a)
}

fn dominates(worn: &borg_item, spare: &borg_item) -> bool {
    worn.iqty > 0
        && !worn.cursed
        && spare.tval == worn.tval
        && (spare.sval == worn.sval || u32::from(spare.tval) != TV_BOW)
        && worn.weight <= spare.weight
        && armor(worn) >= armor(spare)
        && damage(worn) >= damage(spare)
        && worn.to_h >= spare.to_h
        && worn
            .flags
            .iter()
            .zip(spare.flags)
            .all(|(&have, need)| have & need == need)
}

fn gear(item: &borg_item, worn: &borg_item, can_decurse: bool, gain: i64) -> Role {
    let special = known_special(item);
    if item.cursed {
        let promising = special
            || gain > 0
            || worn.iqty > 0
                && (armor(item) > armor(worn)
                    || damage(item) > damage(worn)
                    || item.to_h > worn.to_h);
        if item.uncursable && promising && can_decurse {
            return Role::Gear;
        }
        // A known inferior curse needs no rune identification to reject it.
        // Keep unidentified gear where known bonuses still suggest an upgrade.
        if !promising && (item.ident || worn.iqty > 0 || item.value <= 0) {
            return Role::Reject;
        }
        return Role::Ordinary;
    }

    if !special && item.ident && gain <= 0 && dominates(worn, item) {
        return Role::Reject;
    }
    let lighter = worn.iqty > 0
        && item.weight < worn.weight
        && match u32::from(item.tval) {
            TV_SWORD | TV_HAFTED | TV_POLEARM | TV_BOW => damage(item) > 0 || item.to_h > 0,
            _ => armor(item) > 0,
        };
    if gain > 0 || item.ident && (special || lighter) {
        return Role::Gear;
    }
    Role::Ordinary
}

pub(crate) fn classify(st: &BorgState, item: &borg_item, can_decurse: bool, gain: i64) -> Role {
    if device(st, item) {
        return Role::Device;
    }
    if let Some(worn) = gear_slot(item).and_then(|slot| st.items.items.get(slot)) {
        return gear(item, worn, can_decurse, gain);
    }
    consumable(st, item)
}

/// Withdrawal can reorder the pack or merge copies. Use the known
/// equipment variant rather than the former Home or pack index.
pub(crate) fn same_variant(a: &borg_item, b: &borg_item) -> bool {
    a.kind == b.kind
        && a.art_idx == b.art_idx
        && a.ego_idx == b.ego_idx
        && a.to_h == b.to_h
        && a.to_d == b.to_d
        && a.to_a == b.to_a
        && a.pval == b.pval
        && a.flags == b.flags
        && a.modifiers == b.modifiers
        && a.cursed == b.cursed
        && a.curses == b.curses
}

/// Ordinary dungeon pack cleanup can wait until town for one useful spare
/// of a kind. Emergency slot/weight relief can still use its normal fallback.
pub(crate) fn keep_in_dungeon(st: &BorgState, slot: usize) -> bool {
    if !enabled(st) || st.borg.tr(BI_CDEPTH) == 0 {
        return false;
    }
    let item = &st.items.items[slot];
    if item.iqty == 0 || item.cursed || !item.aware {
        return false;
    }
    let first_of_kind = st.items.items[..slot]
        .iter()
        .all(|other| other.iqty == 0 || other.kind != item.kind);
    let not_stashed = st
        .store
        .shops
        .get(crate::borg_store::BORG_HOME)
        .is_none_or(|home| {
            home.ware
                .iter()
                .all(|other| other.iqty == 0 || other.kind != item.kind)
        });
    first_of_kind && not_stashed && classify(st, item, false, 0).reserve()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::borg_item::blank_item;

    #[test]
    fn known_curse_with_unknown_runes_does_not_make_a_plain_spare_worth_stashing() {
        let mut sling = blank_item();
        sling.iqty = 1;
        sling.aware = true;
        sling.cursed = true;
        sling.uncursable = true;
        sling.tval = TV_BOW as u8;
        let mut worn = sling;
        worn.cursed = false;
        worn.to_d = 5;
        assert_eq!(gear(&sling, &worn, true, -100), Role::Reject);
        sling.to_d = 8;
        assert_eq!(gear(&sling, &worn, true, -100), Role::Gear);
        sling.to_d = 0;
        sling.ego_idx = 1;
        assert_eq!(gear(&sling, &worn, true, -100), Role::Gear);
        assert_eq!(gear(&sling, &worn, false, -100), Role::Ordinary);
    }

    #[test]
    fn keep_a_lighter_usable_alternative_and_discard_a_dominated_plain_spare() {
        let mut spare = blank_item();
        spare.iqty = 1;
        spare.ident = true;
        spare.tval = TV_HAFTED as u8;
        spare.weight = 30;
        spare.dd = 1;
        spare.ds = 4;
        let mut worn = spare;
        worn.weight = 80;
        worn.dd = 2;
        assert_eq!(gear(&spare, &worn, false, -100), Role::Gear);
        worn.weight = spare.weight;
        assert_eq!(gear(&spare, &worn, false, -100), Role::Reject);
    }
}
