//! #1241: classify Home stock using facts already known to the player.
use crate::borg_headers::borg_item_h::borg_item;
use crate::borg_headers::borg_magic_h::{BLESS, HEROISM};
use crate::borg_headers::borg_trait_h::{BI_CARRY, BI_CDEPTH, BI_CLEVEL, BI_MAXSP, BI_WEIGHT};
use crate::engine::{self, NormalStock};
use crate::headers::list_tvals_h::*;
use crate::params::{value as param, Param};
use crate::state::BorgState;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
#[repr(u8)]
pub enum Role {
    #[default]
    Ordinary,
    // From schema 23, an intermittently stocked regular-shop spare.
    Common,
    // Regular staples and supplies the Borg can readily replace.
    Renewable,
    Reject,
    Gear,
    Device,
    Scarce,
    LightGear,
    SpareWeapon,
    LatePotion,
    Artifact,
}

// Leave low bits for utility and stay inside the integer range of JSON
// readers that represent all numbers as doubles.
const OFFER_TIER: i64 = 1 << 40;

impl Role {
    pub(crate) fn from_code(code: i64) -> Self {
        match code {
            1 => Self::Common,
            2 => Self::Renewable,
            3 => Self::Reject,
            4 => Self::Gear,
            5 => Self::Device,
            6 => Self::Scarce,
            7 => Self::LightGear,
            8 => Self::SpareWeapon,
            9 => Self::LatePotion,
            10 => Self::Artifact,
            _ => Self::Ordinary,
        }
    }

    pub(crate) fn common(self) -> bool {
        matches!(self, Self::Common | Self::Renewable)
    }

    pub(crate) fn reserve(self) -> bool {
        self.gear() || self.reserve_supply()
    }

    pub(crate) fn reserve_supply(self) -> bool {
        matches!(self, Self::Device | Self::Scarce | Self::LatePotion)
    }

    pub(crate) fn gear(self) -> bool {
        matches!(self, Self::Gear | Self::LightGear | Self::SpareWeapon | Self::Artifact)
    }

    pub(crate) fn rank(self) -> u8 {
        match self {
            Self::LatePotion => 5,
            Self::LightGear => 4,
            Self::Gear | Self::Artifact => 3,
            Self::Scarce | Self::Device => 2,
            Self::Common | Self::SpareWeapon => 1,
            _ => 0,
        }
    }

    pub(crate) fn offer_value(self, value: i64) -> i64 {
        if value <= 0 { return 0; }
        i64::from(self.rank()) * OFFER_TIER + value.min(OFFER_TIER - 1)
    }

    pub(crate) fn supply_rank(rank: u8) -> bool {
        rank == Self::Scarce.rank() || rank == Self::LatePotion.rank()
    }

    pub(crate) fn offer_rank(value: i64) -> u8 {
        u8::try_from(value / OFFER_TIER).unwrap_or(0)
    }
}

pub(crate) fn enabled(st: &BorgState) -> bool {
    param(Param::town_home_reserve) != 0 && crate::borg_town_live::active(st)
}

fn consumable(st: &BorgState, item: &borg_item) -> Role {
    if !item.aware || !item.flavour_aware {
        return Role::Ordinary;
    }
    let v = &st.item_val;
    let sval = i32::from(item.sval);
    let tval = u32::from(item.tval);
    if tval == TV_POTION && item.kind != 0 && !item.cursed && late_potion(st, item) {
        return Role::LatePotion;
    }
    let supply = match tval {
        // The Magic Shop can roll this staff, but it is an escape reserve.
        TV_STAFF if sval == v.sv_staff_teleportation => NormalStock::Unlisted,
        // The Alchemist normally restocks this. A useful spare is expendable.
        TV_SCROLL if sval == v.sv_scroll_remove_curse => NormalStock::Intermittent,
        _ => engine::normal_store_stock(&st.access, item.kind),
    };
    let repeatable = match tval {
        TV_SCROLL if sval == v.sv_scroll_light => return Role::Renewable,
        TV_SCROLL if sval == v.sv_scroll_blessing => Some(BLESS),
        TV_POTION if sval == v.sv_potion_boldness => Some(HEROISM),
        _ => None,
    };
    if repeatable.is_some_and(|spell| crate::borg_magic::borg_spell_legal_fail(st, spell, 40)) {
        return Role::Renewable;
    }
    if supply == NormalStock::Staple {
        return Role::Renewable;
    }
    // #1318: town books are Bookseller staples (store.c:208-230 adds every
    // book whose class_book has no `dungeon` flag), so the check above makes
    // them renewable. A dungeon book the class can browse is never sold in
    // a regular shop and fire destroys books, so one spare is a scarce reserve.
    if is_book(tval) {
        return if supply == NormalStock::Unlisted && item.kind != 0 && !item.cursed
            && engine::obj_kind_can_browse(item.kind as usize)
        { Role::Scarce } else { Role::Ordinary };
    }
    if !matches!(tval, TV_ROD | TV_WAND | TV_STAFF | TV_POTION | TV_SCROLL | TV_MUSHROOM) {
        return Role::Ordinary;
    }
    let is_device = matches!(tval, TV_ROD | TV_WAND | TV_STAFF);
    let useful = if is_device {
        if !device(st, item) {
            return Role::Ordinary;
        }
        true
    } else if matches!(tval, TV_POTION | TV_SCROLL | TV_MUSHROOM) && !item.cursed {
        let effects = engine::kind_effect_values(item.kind as usize, st.borg.tr(BI_CLEVEL));
        crate::borg_town_capability::home_consumable_effect(&effects, st)
    } else {
        false
    };
    supply_role(supply, useful, is_device)
}

pub(crate) fn is_book(tval: u32) -> bool {
    matches!(tval, TV_MAGIC_BOOK | TV_PRAYER_BOOK | TV_NATURE_BOOK | TV_SHADOW_BOOK | TV_OTHER_BOOK)
}

fn late_potion(st: &BorgState, item: &borg_item) -> bool {
    let v = &st.item_val;
    let sval = i32::from(item.sval);
    if [v.sv_potion_healing, v.sv_potion_star_healing, v.sv_potion_life,
        v.sv_potion_speed, v.sv_potion_restore_life].contains(&sval)
        || sval == v.sv_potion_restore_mana && st.borg.tr(BI_MAXSP) > 0
    {
        return true;
    }
    // object.txt's stat-gain potions also restore the corresponding stat.
    // Gain-one/lose-one potions are unsuitable as a recovery reserve.
    let effects = engine::kind_effect_values(item.kind as usize, st.borg.tr(BI_CLEVEL));
    safe_stat_restoration(&effects)
}

fn safe_stat_restoration(effects: &[(u16, i32, i64)]) -> bool {
    use crate::headers::list_effects_h::{EF_LOSE_RANDOM_STAT, EF_RESTORE_STAT};
    let has = |effect| effects.iter().any(|&(id, _, _)| u32::from(id) == effect);
    has(EF_RESTORE_STAT) && !has(EF_LOSE_RANDOM_STAT)
}

fn supply_role(supply: NormalStock, useful: bool, is_device: bool) -> Role {
    match (supply, useful, is_device) {
        (NormalStock::Staple, _, _) | (_, false, _) => Role::Renewable,
        (NormalStock::Intermittent, true, _) => Role::Common,
        (NormalStock::Unlisted, true, true) => Role::Device,
        (NormalStock::Unlisted, true, false) => Role::Scarce,
    }
}

fn device(st: &BorgState, item: &borg_item) -> bool {
    let tval = u32::from(item.tval);
    if item.kind == 0 || item.iqty == 0 || !item.aware || item.cursed {
        return false;
    }

    let usable = match tval {
        // Rods track recharge in timeout (obj-util.c:number_charging).
        // A cooling rod can serve as a reserve if it will recharge on the trip.
        TV_ROD => {
            item.timeout <= 0
                || crate::borg_town_capability::rod_ready_after(
                    i64::from(item.iqty),
                    i64::from(item.timeout),
                    i64::from(crate::engine::kind_recharge_average(item.kind as usize)),
                ) < crate::borg_town_utility::Journey::of(st).turns
        }
        TV_WAND | TV_STAFF => item.pval > 0,
        _ => false,
    };
    if !usable || !crate::borg_item_use::ranged_device_reliable(st, item) {
        return false;
    }
    let effects = crate::engine::kind_effect_values(item.kind as usize, st.borg.tr(BI_CLEVEL));
    !crate::borg_town_capability::effect_supply_for_consumer(
        &effects,
        100,
        1000,
        tval == TV_WAND && i32::from(item.sval) == st.item_val.sv_wand_confuse_monster,
    )
    .is_empty()
}

/// The original elemental daggers share melee stats and each supplies a
/// single basic resistance, its brand and a bolt activation (artifact.txt).
fn elemental_dagger(item: &borg_item) -> Option<u32> {
    use crate::borg_headers::borg_trait_h::{BI_RCOLD, BI_RELEC, BI_RFIRE};
    if item.iqty == 0 || !item.aware || !item.ident || item.cursed || item.art_idx == 0
        || u32::from(item.tval) != TV_SWORD
    {
        return None;
    }
    match engine::artifact_name_or_empty(usize::from(item.art_idx)) {
        b"'Narthanc'" => Some(BI_RFIRE),
        b"'Nimthanc'" => Some(BI_RCOLD),
        b"'Dethanc'" => Some(BI_RELEC),
        _ => None,
    }
}

pub(crate) fn preferred_elemental_dagger(st: &BorgState) -> Option<u8> {
    let wield = crate::borg_item::inven_wield() as usize;
    let worn = &st.items.items[wield];
    if elemental_dagger(worn).is_some() { return Some(worn.art_idx); }
    let home = &st.store.shops[crate::borg_store::BORG_HOME].ware;
    home.iter().map(|item| (true, item)).chain(st.items.items[..wield].iter().map(|item| (false, item)))
        .filter_map(|(at_home, item)| {
            let resist = elemental_dagger(item)?;
            let fills_gap = st.borg.tr(resist) == 0;
            Some(((fills_gap, at_home, item.to_h, item.to_d, item.value, item.art_idx), item.art_idx))
        })
        .max_by_key(|(score, _)| *score).map(|(_, artifact)| artifact)
}

pub(crate) fn redundant_elemental_dagger(st: &BorgState, item: &borg_item, preferred: Option<u8>) -> bool {
    preferred.is_some_and(|artifact| artifact != item.art_idx)
        && elemental_dagger(item).is_some_and(|resist| st.borg.tr(resist) > 0)
        && !crate::borg_town_live::retained_gear(st, item)
}

pub(crate) fn gear_slot(item: &borg_item) -> Option<usize> {
    use crate::borg_inventory::borg_wield_slot;
    if !item.aware || matches!(u32::from(item.tval), TV_LIGHT | TV_DIGGING) && !known_special(item) {
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
    spare.tval == worn.tval && outclasses_spare(worn, spare)
}

/// Compare observed properties when both items use the same equipment slot.
pub(crate) fn outclasses_spare(worn: &borg_item, spare: &borg_item) -> bool {
    worn.iqty > 0
        && !worn.cursed
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

fn gear(item: &borg_item, worn: &borg_item, can_decurse: bool, gain: i64, tight_carry: bool) -> Role {
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
    if item.art_idx != 0 && item.ident {
        return Role::Artifact;
    }
    if lighter && tight_carry && (item.ident || gain > 0) {
        return Role::LightGear;
    }
    if gain > 0 || item.ident && (special || lighter) {
        return if gain <= 0 && item.art_idx == 0 && item.activ_idx == 0
            && crate::borg_spare_gear::melee_weapon(item)
        { Role::SpareWeapon } else { Role::Gear };
    }
    Role::Ordinary
}

pub(crate) fn classify(st: &BorgState, item: &borg_item, can_decurse: bool, gain: i64) -> Role {
    // Respect the explicit consumer exclusions before inferring reserve value
    // from effects. Both town cleanup and loadout eviction use this policy.
    if crate::borg_town_capability::unwanted_home_item(item) {
        return Role::Reject;
    }
    if let Some(worn) = gear_slot(item).and_then(|slot| st.items.items.get(slot)) {
        let allowance = st.borg.tr(BI_CARRY) / 2;
        let tight_carry = allowance > 0 && st.borg.tr(BI_WEIGHT) >= allowance;
        let role = gear(item, worn, can_decurse, gain, tight_carry);
        return if role == Role::Ordinary && item.ident && !item.cursed {
            Role::Renewable
        } else {
            role
        };
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
    if st.borg.tr(BI_CDEPTH) == 0 {
        return false;
    }
    let item = &st.items.items[slot];
    if crate::borg_junk::gear_alternatives() > 0 && crate::borg_spare_gear::can_rank(item) {
        return true;
    }
    if !enabled(st) || item.iqty == 0 || item.cursed || !item.aware {
        return false;
    }
    let role = classify(st, item, false, 0);
    if !role.reserve() {
        return false;
    }
    let different = |other: &borg_item| {
        other.iqty == 0 || other.kind != item.kind || role.gear() && !same_variant(item, other)
    };
    let first_of_kind = st.items.items[..slot]
        .iter()
        .all(different);
    let not_stashed = st
        .store
        .shops
        .get(crate::borg_store::BORG_HOME)
        .is_none_or(|home| {
            home.ware.iter().all(different)
        });
    first_of_kind && not_stashed
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::borg_item::blank_item;

    #[test]
    fn restocking_prefers_safe_stat_recovery() {
        use crate::headers::list_effects_h::{EF_LOSE_RANDOM_STAT, EF_RESTORE_STAT};
        let restore = (EF_RESTORE_STAT as u16, 0, 0);
        assert!(safe_stat_restoration(&[restore]));
        assert!(!safe_stat_restoration(&[restore, (EF_LOSE_RANDOM_STAT as u16, 0, 0)]));
        assert!(!safe_stat_restoration(&[]));
        assert!(Role::LatePotion.rank() > Role::LightGear.rank());
        assert!(Role::Device.rank() > Role::SpareWeapon.rank());
    }

    #[test]
    fn shops_and_usefulness_set_the_supply_tier() {
        assert_eq!(supply_role(NormalStock::Staple, true, false), Role::Renewable);
        assert_eq!(supply_role(NormalStock::Intermittent, true, false), Role::Common);
        assert_eq!(supply_role(NormalStock::Intermittent, true, true), Role::Common);
        assert_eq!(supply_role(NormalStock::Unlisted, true, false), Role::Scarce);
        assert_eq!(supply_role(NormalStock::Unlisted, true, true), Role::Device);
        assert_eq!(supply_role(NormalStock::Unlisted, false, false), Role::Renewable);
        assert!(Role::Gear.offer_value(1) > Role::Device.offer_value(100_000));
        assert!(Role::Device.offer_value(20) > Role::Scarce.offer_value(10));
    }

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
        assert_eq!(gear(&sling, &worn, true, -100, false), Role::Reject);
        sling.to_d = 8;
        assert_eq!(gear(&sling, &worn, true, -100, false), Role::Gear);
        sling.to_d = 0;
        sling.ego_idx = 1;
        assert_eq!(gear(&sling, &worn, true, -100, false), Role::Gear);
        assert_eq!(gear(&sling, &worn, false, -100, false), Role::Ordinary);
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
        assert_eq!(gear(&spare, &worn, false, -100, false), Role::SpareWeapon);
        assert_eq!(gear(&spare, &worn, false, -100, true), Role::LightGear);
        worn.weight = spare.weight;
        assert_eq!(gear(&spare, &worn, false, -100, true), Role::Reject);
    }
}
