//! #1264: compare two known equipment loadouts before disposing of ordinary spares.
use crate::borg_headers::borg_item_h::borg_item;
use crate::borg_headers::borg_trait_h::*;
use crate::borg_item::{inven_wield, quiver_start};
use crate::headers::list_player_flags_h::PF_FAST_SHOT;
use crate::headers::list_tvals_h::*;
use crate::headers::list_object_modifiers_h::OBJ_MOD_SHOTS;
use crate::headers::list_object_flags_h::{
    OF_AFRAID, OF_AGGRAVATE, OF_DRAIN_EXP, OF_FRAGILE, OF_IMPAIR_HP, OF_IMPAIR_MANA, OF_NO_TELEPORT, OF_STICKY,
};
use crate::{engine, state::BorgState};

fn equipment_slot(item: &borg_item) -> Option<usize> {
    match u32::from(item.tval) {
        TV_HAFTED | TV_POLEARM | TV_SWORD | TV_BOW | TV_SOFT_ARMOR | TV_HARD_ARMOR
        | TV_DRAG_ARMOR | TV_CLOAK | TV_SHIELD | TV_HELM | TV_CROWN | TV_GLOVES
        | TV_BOOTS => usize::try_from(crate::borg_inventory::borg_wield_slot(item)).ok(),
        _ => None,
    }
}

fn known_properties_cover(worn: &borg_item, spare: &borg_item) -> bool {
    // Keep a visible situational advantage even when another slot supplies it today.
    // A different brand, slay, resistance, or modifier can matter after another swap.
    let mut benefits = spare.flags;
    for flag in [OF_IMPAIR_HP, OF_IMPAIR_MANA, OF_AFRAID, OF_NO_TELEPORT, OF_AGGRAVATE, OF_DRAIN_EXP, OF_STICKY, OF_FRAGILE] {
        if engine::flag_has(&worn.flags, flag as i32) && !engine::flag_has(&spare.flags, flag as i32) {
            return false;
        }
        let bit = (flag as i32 - engine::FLAG_START) as usize;
        benefits[bit / 8] &= !(1 << (bit % 8));
    }
    worn.flags.iter().zip(benefits).all(|(have, need)| have & need == need)
        && worn.modifiers.iter().zip(&spare.modifiers).all(|(have, need)| have >= need)
        && worn.el_info.iter().zip(&spare.el_info).all(|(have, need)| have.res_level >= need.res_level)
        && worn.brands.iter().zip(&spare.brands).all(|(have, need)| *have || !*need)
        && worn.slays.iter().zip(&spare.slays).all(|(have, need)| have >= need)
}

#[derive(Debug, PartialEq, Eq)]
struct Kit {
    ammunition: i32,
    measures: [i64; 9],
}

impl Kit {
    fn observe(st: &BorgState) -> Self {
        let tr = |trait_| st.borg.tr(trait_);
        let wield = inven_wield() as usize;
        let weapon = &st.items.items[wield];
        let bow = &st.items.items[wield + 1];
        let blows = if weapon.iqty == 0 || tr(BI_HEAVYWEPON) != 0 { 100 }
            else { crate::borg_trait::borg_calc_blows_hundredths(st, weapon) };
        // player-attack.c:chance_of_melee_hit_base, chance_of_missile_hit_base,
        // hit_chance (89–142, 196–219). At any fixed target AC, hit probability
        // is monotone in the attack rating. Compare rating and damage
        // independently so a low-AC/high-AC tradeoff always survives.
        let melee = tr(BI_WDD) * (tr(BI_WDS) + 1) + 2 * (tr(BI_TODAM) + tr(BI_WTODAM));
        let melee_hit = tr(BI_THN) + 3 * (tr(BI_TOHIT) + tr(BI_WTOHIT));
        let launcher_hit = tr(BI_THB) + 3 * (tr(BI_TOHIT) + tr(BI_BTOHIT));
        let shots = if bow.iqty == 0 { 0 } else if tr(BI_HEAVYBOW) != 0 { 10 } else {
            // player-calcs.c:2264–2287, shots per turn in tenths.
            let modifiers: i32 = st.items.items[wield..quiver_start() as usize].iter()
                .filter(|item| item.iqty > 0)
                .map(|item| i32::from(item.modifiers[OBJ_MOD_SHOTS as usize])).sum();
            let class_shots = if engine::player_has(PF_FAST_SHOT) { tr(BI_CLEVEL) / 3 } else { 0 };
            (10 + modifiers + class_shots).max(10)
        };
        let ammunition = tr(BI_AMMO_TVAL);
        let launcher_rate = i64::from(tr(BI_AMMO_POWER)) * i64::from(shots);
        let launcher_damage = i64::from((tr(BI_AMMO_SIDES) + 1 + 2 * tr(BI_BTODAM)).max(0)) * launcher_rate;
        let room = tr(BI_CARRY) / 2 - tr(BI_WEIGHT);
        let shortfall = (crate::borg_town_utility::departure_room().weight - i64::from(room)).max(0);
        let spell_armor: i32 = st.items.items[wield + 6..quiver_start() as usize].iter()
            .filter(|item| item.iqty > 0).map(|item| i32::from(item.weight)).sum();
        let mana_penalty = if engine::class_total_spells() > 0 {
            ((spell_armor - engine::class_spell_weight()) / 10).max(0)
        } else { 0 };
        Self {
            ammunition,
            measures: [
                i64::from(tr(BI_ARMOR)), i64::from(tr(BI_SPEED)), -shortfall,
                i64::from(melee.max(0)) * i64::from(blows), i64::from(melee_hit),
                launcher_damage, launcher_rate, i64::from(launcher_hit),
                -i64::from(mana_penalty),
            ],
        }
    }

    fn dominates(&self, other: &Self) -> bool {
        self.ammunition == other.ammunition
            && self.measures != other.measures
            && self.measures.iter().zip(other.measures).all(|(have, need)| *have >= need)
    }
}

/// Assess the kit after leaving the entire spare stack or the displaced
/// piece behind. borg_notice reads only the Borg's known item copies.
pub(crate) fn inferior(st: &mut BorgState, pack_slot: usize) -> bool {
    let spare = st.items.items[pack_slot];
    if spare.iqty == 0 || !spare.aware || spare.curses.contains(&true)
        || crate::borg_junk::is_swap(st, pack_slot as i32)
        || crate::borg_town_home::known_special(&spare)
        || crate::borg_town_live::retained_gear(st, &spare)
        || inspection_pending(st, &spare)
    {
        return false;
    }
    let Some(slot) = equipment_slot(&spare) else { return false; };
    let worn = st.items.items[slot];
    // A digger in the weapon slot may only be equipped for tunnelling.
    if worn.iqty == 0 || worn.cursed || u32::from(worn.tval) == TV_DIGGING
        || !known_properties_cover(&worn, &spare)
    {
        return false;
    }
    let borg = st.borg.clone();
    let traits = st.traits.clone();
    let home = st.home.clone();
    let items = st.items.items.clone();
    let do_spell = std::mem::replace(&mut st.think.do_spell, false);
    st.items.items[pack_slot].iqty = 0;
    crate::borg_trait::borg_notice(st, false);
    let current = Kit::observe(st);
    st.items.items[slot] = borg_item { iqty: 1, ..spare };
    crate::borg_trait::borg_notice(st, false);
    let alternative = Kit::observe(st);
    st.items.items = items;
    st.borg = borg;
    st.traits = traits;
    st.home = home;
    st.think.do_spell = do_spell;
    current.dominates(&alternative)
}

pub(crate) fn inspection_pending(st: &BorgState, item: &borg_item) -> bool {
    item.needs_ident && st.borg.tr(BI_AID) > 0 && equipment_slot(item).is_some()
}

#[cfg(feature = "test-town-loadout")]
pub(crate) fn fixture(st: &mut BorgState, case: &str) -> bool {
    let wield = inven_wield() as usize;
    if case == "gear" {
        let slot = st.items.items[..wield].iter()
            .position(|item| item.iqty > 0 && u32::from(item.tval) == TV_POLEARM)
            .expect("the fixture carries a Trident");
        assert!(inferior(st, slot), "the worn Dagger beats the Trident");
        let trident = st.items.items[slot];
        st.items.items[slot].ident = false;
        st.borg.trait_[BI_AID as usize] = 1;
        assert!(inferior(st, slot), "an unknown effect alone cannot be learned from a rune scroll");
        st.items.items[slot].needs_ident = true;
        assert!(!inferior(st, slot), "give the Borg its remaining rune-identification chance");
        st.borg.trait_[BI_AID as usize] = 0;
        assert!(inferior(st, slot), "without a way to identify runes, compare known properties");
        st.items.items[slot] = trident;
        st.items.items[slot].slays[1] = 3;
        assert!(!inferior(st, slot), "the Trident has a slay the Dagger lacks");
        st.items.items[slot] = trident;
        let fragile = (OF_FRAGILE as i32 - engine::FLAG_START) as usize;
        st.items.items[slot].flags[fragile / 8] |= 1 << (fragile % 8);
        st.items.items[slot].cursed = true;
        assert!(inferior(st, slot), "a known drawback does not make inferior gear useful");
        st.items.items[slot] = trident;
        let dagger = st.items.items[wield];
        st.items.items[wield].tval = TV_DIGGING as u8;
        assert!(!inferior(st, slot), "a digger does not replace the combat weapon");
        st.items.items[wield] = dagger;

        let body = wield + 6;
        let slot = st.items.items[..wield].iter()
            .position(|item| item.iqty > 0 && u32::from(item.tval) == TV_HARD_ARMOR)
            .expect("the fixture carries Metal Scale Mail");
        let plate = st.items.items[slot];
        assert!(inferior(st, slot), "the worn soft armour beats the spare hard armour");
        st.items.items[slot].to_a += 20;
        assert!(!inferior(st, slot), "keep a known armour upgrade");
        st.items.items[slot] = plate;
        st.items.items[body].weight += 1200;
        assert!(!inferior(st, slot), "keep the spare that prevents a slowdown");
        println!("#1264 exploration pack passed: {case}");
        std::process::exit(0);
    }

    assert_eq!(case, "recall-control");
    let recall = |item: &borg_item| item.iqty > 0 && u32::from(item.tval) == TV_SCROLL
        && i32::from(item.sval) == st.item_val.sv_scroll_word_of_recall;
    if !st.items.items[..wield].iter().any(recall) {
        assert!(!crate::borg_junk::exploration_first(), "the protected Recall was lost");
        println!("#1264 exploration pack passed: {case}");
        std::process::exit(0);
    }
    assert!(st.items.items[..wield].iter().all(|item| item.iqty > 0), "fixture must fill the pack");
    // The control isolates the reserve rule by making Recall the sole
    // candidate. The independent full-pack tests use the unmodified kit.
    let items = st.items.items.clone();
    for item in &mut st.items.items[..wield] {
        if !recall(item) {
            item.art_idx = 1;
        }
    }
    let dropped = crate::borg_junk::borg_drop_hole(st, false);
    st.items.items = items;
    assert_ne!(dropped, crate::borg_junk::exploration_first());
    if !dropped {
        println!("#1264 exploration pack passed: {case}");
        std::process::exit(0);
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn drawbacks_do_not_count_as_spare_advantages() {
        let mut spare = crate::borg_item::blank_item();
        let worn = spare;
        let bit = (OF_FRAGILE as i32 - engine::FLAG_START) as usize;
        spare.flags[bit / 8] |= 1 << (bit % 8);
        assert!(known_properties_cover(&worn, &spare));
        assert!(!known_properties_cover(&spare, &worn));
        spare.el_info[0].res_level = 1;
        assert!(!known_properties_cover(&worn, &spare));
        spare.el_info[0].res_level = 0;
        spare.slays[1] = 3;
        assert!(!known_properties_cover(&worn, &spare));
    }

    #[test]
    fn a_spare_survives_a_speed_or_accuracy_tradeoff() {
        let worn = Kit { ammunition: 0, measures: [20, 109, 0, 500, 30, 0, 0, 0, 0] };
        let mut spare = Kit { ammunition: 0, measures: [15, 109, 0, 450, 30, 0, 0, 0, 0] };
        assert!(worn.dominates(&spare));
        spare.measures[1] = 110;
        assert!(!worn.dominates(&spare));
        spare.measures[1] = 109;
        spare.measures[2] = 0;
        let mut burdened = Kit { ammunition: worn.ammunition, measures: worn.measures };
        burdened.measures[2] = -20;
        assert!(!burdened.dominates(&spare));
        spare.measures[4] = 35;
        assert!(!worn.dominates(&spare));
        assert!(!worn.dominates(&worn));
    }
}
