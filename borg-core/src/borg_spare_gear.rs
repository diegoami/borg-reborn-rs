//! #1264: compare two known equipment loadouts before making pack room.
use crate::borg_headers::borg_item_h::borg_item;
use crate::borg_headers::borg_trait_h::*;
use crate::borg_item::{inven_wield, quiver_start};
use crate::headers::list_player_flags_h::PF_FAST_SHOT;
use crate::headers::list_tvals_h::*;
use crate::headers::list_object_modifiers_h::OBJ_MOD_SHOTS;
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
    worn.flags.iter().zip(&spare.flags).all(|(have, need)| have & need == *need)
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
    if spare.iqty == 0 || !spare.aware || spare.cursed
        || crate::borg_town_home::known_special(&spare)
        || crate::borg_town_live::retained_gear(st, &spare)
        || inspection_pending(st, &spare)
    {
        return false;
    }
    let Some(slot) = equipment_slot(&spare) else { return false; };
    let worn = st.items.items[slot];
    if worn.iqty == 0 || worn.cursed || !known_properties_cover(&worn, &spare) {
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
    equipment_slot(item).is_some() && (!item.ident || item.needs_ident) && st.borg.tr(BI_AID) > 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_spare_survives_a_speed_or_accuracy_tradeoff() {
        let worn = Kit { ammunition: 0, measures: [20, 109, 0, 500, 30, 0, 0, 0, 0] };
        let mut spare = Kit { ammunition: 0, measures: [15, 109, 0, 450, 30, 0, 0, 0, 0] };
        assert!(worn.dominates(&spare));
        spare.measures[1] = 110;
        assert!(!worn.dominates(&spare));
        spare.measures[1] = 109;
        spare.measures[4] = 35;
        assert!(!worn.dominates(&spare));
        assert!(!worn.dominates(&worn));
    }
}
