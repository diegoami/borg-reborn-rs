//! Compare known loadouts and melee swap roles before disposing of spares.
use crate::borg_headers::borg_item_h::borg_item;
use crate::borg_headers::borg_trait_h::*;
use crate::borg_item::{inven_wield, quiver_start};
use crate::headers::list_player_flags_h::PF_FAST_SHOT;
use crate::headers::list_tvals_h::*;
use crate::headers::list_object_modifiers_h::OBJ_MOD_SHOTS;
use crate::headers::list_stats_h::{OBJ_MOD_WIS, STAT_WIS};
use crate::headers::list_object_flags_h::{
    OF_AFRAID, OF_AGGRAVATE, OF_DRAIN_EXP, OF_FEATHER, OF_FRAGILE, OF_IMPAIR_HP, OF_IMPAIR_MANA, OF_NO_FUEL, OF_NO_TELEPORT, OF_STICKY,
};
use crate::{engine, state::BorgState};

pub(crate) fn equipment_slot(item: &borg_item) -> Option<usize> {
    match u32::from(item.tval) {
        TV_HAFTED | TV_POLEARM | TV_SWORD | TV_BOW | TV_SOFT_ARMOR | TV_HARD_ARMOR
        | TV_DRAG_ARMOR | TV_CLOAK | TV_SHIELD | TV_HELM | TV_CROWN | TV_GLOVES
        | TV_BOOTS => usize::try_from(crate::borg_inventory::borg_wield_slot(item)).ok(),
        _ => None,
    }
}

fn other_properties_cover(worn: &borg_item, spare: &borg_item) -> bool {
    properties_cover(worn, spare, &[])
}

fn properties_cover(worn: &borg_item, spare: &borg_item, other_slots: &[&borg_item]) -> bool {
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
        && worn.el_info.iter().zip(&spare.el_info).enumerate().all(|(index, (have, need))| {
            have.res_level >= need.res_level || (have.res_level >= 0 && other_slots.iter()
                .any(|item| item.el_info[index].res_level >= need.res_level))
        })
}

pub(crate) fn known_properties_cover(worn: &borg_item, spare: &borg_item) -> bool {
    properties_and_attack_cover(worn, spare, &[])
}

fn properties_and_attack_cover(worn: &borg_item, spare: &borg_item, other_slots: &[&borg_item]) -> bool {
    properties_cover(worn, spare, other_slots)
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
        let bow = &st.items.items[wield + 1];
        let melee = Melee::observe(st);
        // player-attack.c:chance_of_melee_hit_base, chance_of_missile_hit_base,
        // hit_chance (89–142, 196–219). At any fixed target AC, hit probability
        // is monotone in the attack rating. Compare rating and damage
        // independently so a low-AC/high-AC tradeoff always survives.
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
                melee.damage(1), i64::from(melee.hit),
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

struct Melee {
    dice: i32,
    bonus: i32,
    blows: i32,
    hit: i32,
    armor: i32,
    speed: i32,
    saving_throw: i32,
}

impl Melee {
    fn observe(st: &BorgState) -> Self {
        let tr = |trait_| st.borg.tr(trait_);
        let weapon = &st.items.items[inven_wield() as usize];
        Self {
            dice: tr(BI_WDD) * (tr(BI_WDS) + 1),
            bonus: 2 * (tr(BI_TODAM) + tr(BI_WTODAM)),
            blows: if weapon.iqty == 0 || tr(BI_HEAVYWEPON) != 0 { 100 }
                else { crate::borg_trait::borg_calc_blows_hundredths(st, weapon) },
            hit: tr(BI_THN) + 3 * (tr(BI_TOHIT) + tr(BI_WTOHIT)),
            armor: tr(BI_ARMOR),
            speed: tr(BI_SPEED),
            saving_throw: tr(BI_SAV),
        }
    }

    // player-attack.c:melee_damage and py_attack_real (480–495, 786–828).
    // Standard combat multiplies the dice, then adds flat damage. Use twice
    // the mean per blow to keep half points, before criticals and accuracy.
    fn per_blow(&self, multiplier: i32) -> i32 {
        (self.dice * multiplier + self.bonus).max(0)
    }

    fn damage(&self, multiplier: i32) -> i64 {
        i64::from(self.per_blow(multiplier)) * i64::from(self.blows)
    }
}

pub(crate) fn melee_weapon(item: &borg_item) -> bool {
    item.iqty > 0 && matches!(u32::from(item.tval), TV_HAFTED | TV_POLEARM | TV_SWORD)
}

fn known_gear(item: &borg_item) -> bool {
    item.iqty > 0 && item.aware && item.ident && !item.cursed && !item.curses.contains(&true)
}

fn known_melee(item: &borg_item) -> bool {
    melee_weapon(item) && known_gear(item)
}

/// Compare worn gear and alternatives that will still be available after
/// disposal. For Home cleanup, also consider durable alternatives in Home.
fn redundant_melee(st: &mut BorgState, spare_slot: usize, at_home: bool) -> bool {
    use crate::headers::list_options_h::OPT_birth_percent_damage;
    let wield = inven_wield() as usize;
    let worn = st.items.items[wield];
    let spare = if at_home { st.store.shops[crate::borg_store::BORG_HOME].ware[spare_slot] }
        else { st.items.items[spare_slot] };
    if !known_melee(&spare) || spare.art_idx != 0 || spare.activ_idx != 0
        || !known_melee(&worn) || engine::player_opt(OPT_birth_percent_damage)
    {
        return false;
    }
    use crate::headers::object_h::EL_INFO_IGNORE;
    let covers = |candidate: &borg_item| {
        known_melee(candidate) && candidate.weight <= spare.weight
            && other_properties_cover(candidate, &spare)
            && candidate.el_info.iter().zip(&spare.el_info)
                .all(|(have, need)| u32::from(need.flags & !have.flags) & EL_INFO_IGNORE == 0)
            && candidate.brands.iter().zip(&spare.brands).all(|(have, need)| *have || !*need)
            && candidate.slays.iter().zip(&spare.slays).all(|(&have, &need)| need <= 1 || have > 1)
    };
    let candidates: Vec<_> = st.items.items[..wield].iter().enumerate()
        .filter(|(slot, item)| (at_home || *slot != spare_slot) && covers(item))
        .map(|(slot, item)| (Some(slot), *item, at_home || slot < spare_slot))
        .chain(at_home.then(|| &st.store.shops[crate::borg_store::BORG_HOME].ware)
            .into_iter().flatten().enumerate()
            .filter(|(slot, item)| *slot != spare_slot && covers(item))
            .map(|(slot, item)| (None, *item, slot < spare_slot)))
        .chain(covers(&worn).then_some((None, worn, true)))
        .collect();
    if candidates.is_empty() { return false; }

    let borg = st.borg.clone();
    let traits = st.traits.clone();
    let home = st.home.clone();
    let do_spell = std::mem::replace(&mut st.think.do_spell, false);
    let equip = |st: &mut BorgState, item: borg_item| {
        st.items.items[wield] = borg_item { iqty: 1, ..item };
        crate::borg_trait::borg_notice(st, false);
        Melee::observe(st)
    };
    if !at_home { st.items.items[spare_slot].iqty = 0; }
    let redundant = candidates.into_iter().any(|(slot, candidate, preferred_on_tie)| {
        let saved = slot.map(|slot| {
            let item = st.items.items[slot];
            st.items.items[slot].iqty = 0;
            item
        });
        let baseline = equip(st, worn);
        let alternative = equip(st, spare);
        let better = equip(st, candidate);
        if let Some((slot, saved)) = slot.zip(saved) { st.items.items[slot] = saved; }

        // A general-purpose role matters if the primary weapon cannot fill
        // it. Otherwise compare the spare's actual slay and brand roles.
        let general_role = baseline.hit < alternative.hit || baseline.damage(1) < alternative.damage(1);
        let damage_covers = |have, need| better.damage(have) >= alternative.damage(need)
            && (!at_home || better.per_blow(have) >= alternative.per_blow(need));
        let slays_covered = candidate.slays.iter().zip(&spare.slays)
            .all(|(&have, &need)| need <= 1 || damage_covers(have, need));
        let brands_covered = spare.brands.iter().enumerate().all(|(index, &needed)| {
            if !needed { return true; }
            // obj-slays.c:get_monster_brand_multiplier, 323–340.
            let (_, vulnerable, multiplier) = engine::brand_resist_vuln_multiplier(index);
            damage_covers(multiplier, multiplier)
                && (vulnerable == 0 || damage_covers(2 * multiplier, 2 * multiplier))
        });
        // Compare accuracy independently. A low-AC hit rate can hide an
        // accuracy advantage against the tougher members of the same race.
        better.hit >= alternative.hit && better.armor >= alternative.armor
            && better.speed >= alternative.speed && (!general_role || damage_covers(1, 1))
            && slays_covered && brands_covered
            && (candidate.weight < spare.weight || better.hit > alternative.hit
                || better.armor > alternative.armor || better.speed > alternative.speed
                || candidate.slays.iter().zip(&spare.slays).any(|(&have, &need)|
                    need > 1 && better.damage(have) > alternative.damage(need))
                || (better.damage(1), preferred_on_tie) > (alternative.damage(1), false))
    });
    st.items.items[wield] = worn;
    if !at_home { st.items.items[spare_slot] = spare; }
    st.borg = borg;
    st.traits = traits;
    st.home = home;
    st.think.do_spell = do_spell;
    redundant
}

fn armour(item: &borg_item) -> bool {
    matches!(u32::from(item.tval), TV_SOFT_ARMOR | TV_HARD_ARMOR | TV_DRAG_ARMOR
        | TV_CLOAK | TV_SHIELD | TV_HELM | TV_CROWN | TV_GLOVES | TV_BOOTS)
}

// A matched launcher subtype has the same ammunition and base firing
// mechanism. For armour, weight also covers the caster's mana burden.
fn static_gear_covers(candidate: &borg_item, spare: &borg_item, other_slots: &[&borg_item], preferred_on_tie: bool) -> bool {
    let defence = |item: &borg_item| i32::from(item.ac) + i32::from(item.to_a);
    let launcher = u32::from(spare.tval) == TV_BOW;
    known_gear(candidate) && (!launcher || candidate.sval == spare.sval && candidate.pval >= spare.pval)
        && candidate.weight <= spare.weight && defence(candidate) >= defence(spare)
        && candidate.to_h >= spare.to_h && candidate.to_d >= spare.to_d
        && properties_and_attack_cover(candidate, spare, other_slots)
        && (preferred_on_tie || candidate.weight < spare.weight || defence(candidate) > defence(spare)
            || candidate.to_h > spare.to_h || candidate.to_d > spare.to_d
            || launcher && candidate.pval > spare.pval)
}

fn redundant_static_gear(st: &BorgState, spare: &borg_item, spare_slot: usize, at_home: bool) -> bool {
    if !known_gear(spare) || spare.art_idx != 0 || spare.activ_idx != 0 {
        return false;
    }
    let Some(slot) = equipment_slot(spare) else { return false; };
    let wield = inven_wield() as usize;
    let other_slots: Vec<_> = if at_home && armour(spare) {
        st.items.items[wield..quiver_start() as usize].iter().enumerate()
            .filter(|(index, item)| wield + index != slot && item.iqty > 0 && !item.cursed)
            .map(|(_, item)| item).collect()
    } else { Vec::new() };
    let beats = |candidate: &borg_item, preferred| {
        candidate.iqty > 0 && equipment_slot(candidate) == Some(slot)
            && static_gear_covers(candidate, spare, &other_slots, preferred)
    };
    beats(&st.items.items[slot], true)
        || st.items.items[..wield].iter().enumerate().any(|(index, item)| {
            (at_home || index != spare_slot) && beats(item, at_home || index < spare_slot)
        })
        || (at_home && st.store.shops[crate::borg_store::BORG_HOME].ware.iter().enumerate().any(|(index, item)| {
            index != spare_slot && beats(item, index < spare_slot)
        }))
}

pub(crate) fn redundant_carried_launcher(st: &BorgState, slot: usize) -> bool {
    let spare = &st.items.items[slot];
    u32::from(spare.tval) == TV_BOW && !crate::borg_junk::is_swap(st, slot as i32)
        && !crate::borg_town_live::retained_gear(st, spare)
        && redundant_static_gear(st, spare, slot, false)
}

pub(crate) fn redundant_home_gear(st: &mut BorgState, slot: usize) -> bool {
    let spare = st.store.shops[crate::borg_store::BORG_HOME].ware[slot];
    crate::borg_junk::gear_alternatives() == 0 && crate::borg_junk::exploration_first()
        && !crate::borg_town_live::retained_gear(st, &spare)
        && if melee_weapon(&spare) { redundant_melee(st, slot, true) }
            else { redundant_static_gear(st, &spare, slot, true) }
}

// Stable order for equivalent gear: keep it worn if possible, then at Home
// so it uses no pack space, then in the earliest stack at either location.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum GearLocation {
    Worn(usize),
    Home(usize),
    Pack(usize),
}

fn covered_copies<T>(
    spare: &(GearLocation, T), candidates: &[(GearLocation, T, u8)], quantity: u8, worn_capacity: u8,
    cutoff: usize, covers: impl Fn(&T, &T) -> bool,
) -> u8 {
    let mut better = Vec::new();
    let mut keep = worn_capacity;
    for (location, candidate, copies) in candidates {
        if !covers(candidate, &spare.1) { continue; }
        if covers(&spare.1, candidate) {
            if *location < spare.0 { keep = keep.saturating_sub(*copies); }
        } else if !better.iter().any(|&other| covers(other, candidate) && covers(candidate, other)) {
            better.push(candidate);
            if better.len() == cutoff.max(usize::from(worn_capacity)) { return quantity; }
        }
    }
    // Keep enough equivalent copies to fill the slot(s) at once. Extra
    // copies never count toward releasing the last usable option.
    quantity.saturating_sub(keep)
}

fn ranked_slot(item: &borg_item) -> Option<usize> {
    match u32::from(item.tval) {
        TV_RING | TV_AMULET | TV_DIGGING => usize::try_from(crate::borg_inventory::borg_wield_slot(item)).ok(),
        TV_LIGHT if engine::flag_has(&item.flags, OF_NO_FUEL as i32) => Some(inven_wield() as usize + 5),
        _ => equipment_slot(item),
    }
}

pub(crate) fn can_rank(item: &borg_item) -> bool {
    known_gear(item) && !item.needs_ident && ranked_slot(item).is_some()
}

fn visible_artifact_weapon(item: &borg_item) -> bool {
    // A named weapon can show its dice and bonuses before every rune is
    // known. Use those facts for comparison, and leave wear eligibility
    // to the ordinary identification and curse checks.
    melee_weapon(item) && item.aware && item.art_idx != 0 && item.dd > 0 && item.ds > 0
        && item.to_h > 0 && item.to_d > 0 && !item.cursed && !item.curses.contains(&true)
}

pub(crate) struct GearPlan {
    pub upgrade: Option<(GearLocation, usize)>,
    pub stock: Vec<GearStock>,
}

pub(crate) struct GearStock {
    pub source: GearLocation,
    pub stash: u8,
    pub release: u8,
}

#[derive(Clone, Copy)]
struct GearChoice {
    source: GearLocation,
    item: borg_item,
    power: i32,
    lighter: bool,
    backup: bool,
    wearable: bool,
}

fn backup_counts(choices: &[GearChoice], worn: &[usize], capacity: usize) -> Vec<u8> {
    let mut keep = vec![0; choices.len()];
    for &index in worn {
        keep[index] += 1;
    }
    let mut ranked: Vec<_> = (0..choices.len()).filter(|&index| choices[index].backup).collect();
    ranked.sort_by_key(|&i| {
        let distinct = capacity == 1 && worn.iter().any(|&w| choices[w].item.iqty > 0
            && !known_properties_cover(&choices[w].item, &choices[i].item));
        // A stealth cloak is an alternate exploration kit. Keep the best
        // cloak for that use when the primary supplies no stealth.
        let stealth = crate::headers::list_object_modifiers_h::OBJ_MOD_STEALTH as usize;
        let stealth_backup = distinct && u32::from(choices[i].item.tval) == TV_CLOAK
            && choices[i].item.modifiers[stealth] > 0
            && worn.iter().all(|&w| choices[w].item.modifiers[stealth] <= 0);
        (std::cmp::Reverse(stealth_backup), std::cmp::Reverse(distinct), std::cmp::Reverse(choices[i].power), choices[i].source)
    });
    let mut regular = Vec::new();
    for &index in &ranked {
        if capacity == 1 && worn.iter().any(|&i| crate::borg_town_home::same_variant(&choices[i].item, &choices[index].item)
            && choices[i].item.iqty > 0) { continue; }
        let copies = usize::from(choices[index].item.iqty - keep[index]).min(capacity - regular.len());
        keep[index] += copies as u8;
        regular.extend(std::iter::repeat_n(index, copies));
        if regular.len() == capacity { break; }
    }
    if capacity == 1 {
        // Require a pound of relief. Compare with both selected pieces
        // so an already-light runner-up fills the role.
        let light_limit = worn.iter().chain(&regular).map(|&i| choices[i].item.weight)
            .min().unwrap_or(0).saturating_sub(10);
        if let Some(index) = ranked.into_iter().find(|&i| keep[i] == 0 && choices[i].lighter
            && choices[i].item.weight <= light_limit)
        {
            keep[index] = 1;
        }
    }
    keep
}

pub(crate) fn planned_slot(item: &borg_item) -> Option<usize> {
    // A tunnelling tool is not a weapon backup. Fueled lights have their
    // own reserve based on remaining turns.
    ((can_rank(item) || visible_artifact_weapon(item)) && !item.one_ring
        && u32::from(item.tval) != TV_DIGGING).then(|| ranked_slot(item)).flatten()
}

fn protected_gear(st: &BorgState, item: &borg_item) -> bool {
    item.art_idx != 0 || item.activ_idx != 0 || crate::borg_town_live::retained_gear(st, item)
}

fn uninspected_gear(item: &borg_item) -> bool {
    if !item.aware || item.iqty == 0 || !item.needs_ident || item.art_idx != 0 || item.ego_idx != 0
        || item.cursed || item.curses.contains(&true) || !(armour(item) || melee_weapon(item)) { return false; }
    item.desc.iter().map(|&c| c as u8).skip_while(|&c| c != b'{')
        .take_while(|&c| c != 0).eq(*b"{??}")
}

pub(crate) fn cleanup_slot(st: &BorgState, item: &borg_item) -> Option<usize> {
    planned_slot(item).or_else(|| {
        uninspected_cleanup_slot(item, st.borg.tr(BI_AID) > 0)
    })
}

fn uninspected_cleanup_slot(item: &borg_item, can_identify: bool) -> Option<usize> {
    (uninspected_gear(item) && (!can_identify || armour(item) && item.iqty > 1))
        .then(|| ranked_slot(item)).flatten()
}

pub(crate) fn gear_plan(st: &mut BorgState, slot: usize, include_home: bool) -> GearPlan {
    let pack = inven_wield() as usize;
    let ring = slot == pack + 3;
    let slots = if ring { slot - 1..=slot } else { slot..=slot };
    let capacity = if ring { 2 } else { 1 };
    let choice = |(source, item)| {
        let available = match source {
            GearLocation::Home(index) => !crate::borg_town_live::retained_gear(st, &item)
                && !crate::borg_town_live::buy_rejected(st, crate::borg_store::BORG_HOME as i16, index)
                && crate::borg_town_live::shopping_weight_capacity(st, item.weight) > 0,
            _ => true,
        };
        GearChoice {
            source, item, power: i32::MIN, lighter: false, backup: planned_slot(&item) == Some(slot),
            wearable: available && (can_rank(&item) || matches!(source, GearLocation::Pack(_))
                && visible_artifact_weapon(&item)
                && !engine::player_opt(crate::headers::list_options_h::OPT_birth_randarts)),
        }
    };
    let mut choices: Vec<_> = slots.clone().map(|i| choice((GearLocation::Worn(i), st.items.items[i])))
        .chain(st.store.shops[crate::borg_store::BORG_HOME].ware.iter().copied().enumerate()
            .filter(|(_, item)| include_home && planned_slot(item) == Some(slot))
            .map(|(i, item)| choice((GearLocation::Home(i), item))))
        .chain(st.items.items[..pack].iter().copied().enumerate()
            .filter(|(_, item)| planned_slot(item) == Some(slot))
            .map(|(i, item)| choice((GearLocation::Pack(i), item))))
        .collect();
    let can_identify = st.borg.tr(BI_AID) > 0;
    let uninspected: Vec<_> = st.items.items[..pack].iter().copied().enumerate()
        .filter(|(_, item)| uninspected_cleanup_slot(item, can_identify) == Some(slot)
            && !protected_gear(st, item))
        .collect();
    if choices.len() == capacity && uninspected.is_empty() {
        return GearPlan { upgrade: None, stock: Vec::new() };
    }

    let items = st.items.items.clone();
    let borg = st.borg.clone();
    let traits = st.traits.clone();
    let home = st.home.clone();
    let do_spell = std::mem::replace(&mut st.think.do_spell, false);
    // Judge the fit of gear to wear, independent of how many of the
    // prospective spares are temporarily in the pack.
    for entry in &choices[capacity..] {
        if let GearLocation::Pack(i) = entry.source { st.items.items[i].iqty = 0; }
    }
    let destinations: Vec<_> = slots.collect();
    let current: Vec<_> = (0..capacity).collect();
    let pack_has_room = engine::pack_slots_free() > 0;
    let evaluate = |st: &mut BorgState, kit: &[usize], choices: &[GearChoice]| {
        for (&index, &target) in kit.iter().zip(&destinations) {
            let item = choices[index].item;
            st.items.items[target] = borg_item { iqty: item.iqty.min(1), ..item };
        }
        crate::borg_trait::borg_notice(st, false);
        (crate::borg_power::borg_power_equipment(st),
            super::borg_town_loadout::keeps_capabilities(&borg.trait_, &st.borg.trait_))
    };
    let wear_allowed = |choices: &[GearChoice], i: usize, target: usize| {
        if i < capacity { return i == target; }
        choices[i].wearable && match choices[i].source {
            GearLocation::Pack(source) => {
                // Use the real inventory for capacity and the ring prompt.
                let old = &items[destinations[target]];
                crate::borg_item_wear::allows_ranked_wear(&choices[i].item, old)
                    && (old.iqty == 0 || items[source].iqty == 1 || pack_has_room)
            }
            _ => crate::borg_item_wear::allows_ranked_wear(&choices[i].item, &items[destinations[target]]),
        }
    };
    let (baseline, _) = evaluate(st, &current, &choices);
    let mut best = current.clone();
    let mut best_power = baseline;
    // Evaluate both fingers together. A stack with two copies is allowed
    // to supply both, and we keep the current arrangement on a tie.
    for first in 0..choices.len() {
        if !wear_allowed(&choices, first, 0) { continue; }
        if ring {
            for second in 0..choices.len() {
                if !wear_allowed(&choices, second, 1) || first == second && choices[first].item.iqty < 2 { continue; }
                let trial = [first, second];
                let (power, _) = evaluate(st, &trial, &choices);
                if power > best_power { best.copy_from_slice(&trial); best_power = power; }
            }
        } else {
            let (power, _) = evaluate(st, &[first], &choices);
            if power > best_power { best[0] = first; best_power = power; }
        }
    }
    if ring {
        for i in 0..choices.len() {
            for target in 0..capacity {
                if i < capacity && i != target || i == best[1 - target] && choices[i].item.iqty < 2 { continue; }
                let mut trial = best.clone();
                trial[target] = i;
                let (power, _) = evaluate(st, &trial, &choices);
                choices[i].power = choices[i].power.max(power);
            }
        }
    } else {
        for i in 0..choices.len() {
            let (power, preserves) = evaluate(st, &[i], &choices);
            choices[i].power = power;
            choices[i].lighter = preserves;
        }
    }
    let keep = backup_counts(&choices, &best, capacity);
    let mut observed_releases = Vec::new();
    if slot == pack && !uninspected.is_empty()
        && !engine::player_opt(crate::headers::list_options_h::OPT_birth_percent_damage)
    {
        let swap = st.trait_swap.weapon_swap;
        for &(index, _) in &uninspected {
            st.items.items[index].iqty = 0;
        }
        let mut observe = |item: borg_item| {
            st.items.items[slot] = borg_item { iqty: 1, ..item };
            crate::borg_trait::borg_notice(st, false);
            Melee::observe(st)
        };
        // Count retained alternatives only. Protected artifacts remain
        // available even when they were not selected as the regular backup.
        let alternatives: Vec<_> = choices.iter().zip(&keep).enumerate().filter(|(index, (entry, count))| {
            wear_allowed(&choices, *index, 0) && (can_rank(&entry.item) || visible_artifact_weapon(&entry.item))
                && (**count > 0 || entry.item.art_idx != 0)
        }).map(|(_, (entry, _))| (entry.item, observe(entry.item))).collect();
        let other_worn: Vec<_> = items[pack + 1..quiver_start() as usize].iter()
            .filter(|item| item.iqty > 0 && !item.cursed).collect();
        let wisdom_casting = crate::borg_magic::borg_spell_stat() == STAT_WIS as i32;
        for &(index, item) in &uninspected {
            if swap == index as i32 + 1 { continue; }
            let spare = observe(item);
            let mut covering = alternatives.iter().filter(|(candidate, trial)| {
                observed_melee_properties_cover(candidate, trial, &item, &spare, &other_worn, wisdom_casting)
                    && melee_damage_roles_cover(candidate, trial, &item, &spare)
            });
            if covering.next().is_some_and(|(first, _)| covering.any(|(candidate, _)| {
                !crate::borg_town_home::same_variant(first, candidate)
            })) {
                observed_releases.push((index, item.iqty));
            }
        }
    }

    st.items.items = items;
    st.borg = borg;
    st.traits = traits;
    st.home = home;
    st.think.do_spell = do_spell;

    let upgrade = best.iter().enumerate().filter(|&(target, &i)| target != i)
        // The engine always uses the open finger before asking which ring
        // to replace. Fill it first if the pair changes both slots.
        .min_by_key(|&(target, _)| choices[target].item.iqty != 0)
        .map(|(target, &i)| (choices[i].source, destinations[target]));
    let mut stock: Vec<_> = choices.iter().zip(&keep).enumerate().skip(capacity).map(|(index, (entry, &keep))| {
        let item = entry.item;
        let protected = protected_gear(st, &item)
            || item.desc.contains(&(b'{' as core::ffi::c_char));
        GearStock {
            source: entry.source,
            stash: keep - best.iter().filter(|&&i| i == index).count() as u8,
            release: if protected { 0 } else { item.iqty - keep },
        }
    }).collect();
    // obj-pile.c:object_similar requires matching actual armour properties
    // for a stack. Keep one copy available to teach any unresolved rune.
    for (index, item) in uninspected.into_iter().filter(|(_, item)| armour(item)) {
        let covered = !can_identify && !crate::borg_junk::is_swap(st, index as i32) && choices.iter().zip(&keep).filter(|(entry, qty)| {
            **qty > 0 && can_rank(&entry.item) && static_gear_covers(&entry.item, &item, &[], false)
        }).take(2).count() == 2;
        let release = if covered { item.iqty } else { item.iqty - 1 };
        if release > 0 {
            observed_releases.push((index, release));
        }
    }
    stock.extend(observed_releases.into_iter().map(|(index, release)| GearStock {
        source: GearLocation::Pack(index), stash: 0, release,
    }));
    GearPlan { upgrade, stock }
}

// Use only the observed loadout. A small Wisdom bonus can be redundant for
// a non-Wisdom caster when it does not improve the actual saving throw.
fn observed_melee_properties_cover(
    candidate: &borg_item, better: &Melee, item: &borg_item, spare: &Melee,
    other_worn: &[&borg_item], wisdom_casting: bool,
) -> bool {
    let mut required = *item;
    if !wisdom_casting && better.saving_throw >= spare.saving_throw {
        required.modifiers[OBJ_MOD_WIS as usize] = 0;
    }
    // Feather Fall is already available with either weapon in this kit.
    if other_worn.iter().any(|gear| engine::flag_has(&gear.flags, OF_FEATHER as i32)) {
        let bit = (OF_FEATHER as i32 - engine::FLAG_START) as usize;
        required.flags[bit / 8] &= !(1 << (bit % 8));
    }
    candidate.weight <= item.weight
        && properties_and_attack_cover(candidate, &required, other_worn)
        && better.hit >= spare.hit && better.armor >= spare.armor && better.speed >= spare.speed
        && better.saving_throw >= spare.saving_throw && better.damage(1) >= spare.damage(1)
}

fn melee_damage_roles_cover(have: &borg_item, better: &Melee, need: &borg_item, spare: &Melee) -> bool {
    need.slays.iter().zip(&have.slays)
        .all(|(&need, &have)| need <= 1 || better.damage(have) >= spare.damage(need))
        && need.brands.iter().enumerate().all(|(index, &needed)| {
            if !needed { return true; }
            let (_, vulnerable, multiplier) = engine::brand_resist_vuln_multiplier(index);
            better.damage(multiplier) >= spare.damage(multiplier)
                && (vulnerable == 0 || better.damage(2 * multiplier) >= spare.damage(2 * multiplier))
        })
}

pub(crate) fn redundant_copies(st: &mut BorgState, location: GearLocation, cutoff: usize) -> u8 {
    let wield = inven_wield() as usize;
    let home = &st.store.shops[crate::borg_store::BORG_HOME].ware;
    let spare = match location {
        GearLocation::Pack(index) => st.items.items[index],
        GearLocation::Home(index) => home[index],
        GearLocation::Worn(_) => return 0,
    };
    if !can_rank(&spare) || spare.art_idx != 0 || spare.activ_idx != 0
        || spare.desc.contains(&(b'{' as core::ffi::c_char))
        || crate::borg_town_live::retained_gear(st, &spare)
    {
        return 0;
    }
    let slot = ranked_slot(&spare).expect("ranked gear has a slot");
    let ring = u32::from(spare.tval) == TV_RING;
    let digger = u32::from(spare.tval) == TV_DIGGING;
    let worn_capacity = if ring { 2 } else { 1 };
    let covers_activation = |have: &borg_item, need: &borg_item| {
        need.activ_idx == 0 || have.activ_idx == need.activ_idx
    };
    let peers: Vec<_> = (slot + 1 - usize::from(worn_capacity)..=slot)
        .map(|i| (GearLocation::Worn(i), &st.items.items[i]))
        .chain(home.iter().enumerate().map(|(i, item)| (GearLocation::Home(i), item)))
        .chain(st.items.items[..wield].iter().enumerate().map(|(i, item)| (GearLocation::Pack(i), item)))
        .filter(|(source, item)| *source != location && (can_rank(item)
            || visible_artifact_weapon(item) && item.kind == spare.kind && item.weight <= spare.weight)
            && ranked_slot(item) == Some(slot)
            && (u32::from(item.tval) == TV_DIGGING) == digger)
        .map(|(source, item)| (source, *item, item.iqty)).collect();
    if !melee_weapon(&spare) {
        return covered_copies(&(location, spare), &peers, spare.iqty, worn_capacity, cutoff,
            |have, need| covers_activation(have, need) && (!digger || have.pval >= need.pval)
                && static_gear_covers(have, need, &[], true));
    }
    if peers.is_empty() || engine::player_opt(crate::headers::list_options_h::OPT_birth_percent_damage) {
        return spare.iqty - 1;
    }

    // Test one weapon at a time with the same character and remaining kit.
    // The candidate and all other weapons removed here are the pack options
    // being compared, not weight that a particular trial has to carry.
    let items = st.items.items.clone();
    let borg = st.borg.clone();
    let traits = st.traits.clone();
    let home = st.home.clone();
    let do_spell = std::mem::replace(&mut st.think.do_spell, false);
    for item in &mut st.items.items[..wield] {
        if melee_weapon(item) { item.iqty = 0; }
    }
    let mut evaluate = |item| {
        st.items.items[slot] = borg_item { iqty: 1, ..item };
        crate::borg_trait::borg_notice(st, false);
        let melee = Melee::observe(st);
        let free_weight = (st.borg.tr(BI_CARRY) / 2 - st.borg.tr(BI_WEIGHT)).min(0);
        (item, melee, free_weight)
    };
    let sample = evaluate(spare);
    let peers: Vec<_> = peers.into_iter().map(|(source, item, copies)| (source, evaluate(item), copies)).collect();
    st.items.items = items;
    st.borg = borg;
    st.traits = traits;
    st.home = home;
    st.think.do_spell = do_spell;
    covered_copies(&(location, sample), &peers, spare.iqty, worn_capacity, cutoff,
        |(have, better, room), (need, worse, required)| {
            (need.ident || need.art_idx == 0 || have.art_idx == need.art_idx) && covers_activation(have, need)
                && properties_and_attack_cover(have, need, &[])
                && room >= required && better.speed >= worse.speed && better.armor >= worse.armor
                && better.hit >= worse.hit && better.damage(1) >= worse.damage(1)
                && melee_damage_roles_cover(have, better, need, worse)
        })
}

/// Assess the kit after leaving the entire spare stack or the displaced
/// piece behind. borg_notice reads only the Borg's known item copies.
pub(crate) fn inferior(st: &mut BorgState, pack_slot: usize) -> bool {
    let spare = st.items.items[pack_slot];
    if spare.iqty == 0 || !spare.aware || spare.curses.contains(&true)
        || crate::borg_junk::is_swap(st, pack_slot as i32)
        || crate::borg_town_live::retained_gear(st, &spare)
        || inspection_pending(st, &spare)
    {
        return false;
    }
    if redundant_carried_launcher(st, pack_slot) || redundant_melee(st, pack_slot, false) { return true; }
    if crate::borg_town_home::known_special(&spare) { return false; }
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
    if matches!(case, "ranked-gear" | "weight-blocked-upgrade") {
        use GearLocation::{Home, Pack};
        let worn_slot = wield + 6;
        let home = crate::borg_store::BORG_HOME;
        let robe = borg_item { iqty: 5, aware: true, ident: true, value: 800,
            tval: TV_SOFT_ARMOR as u8, ac: 2, weight: 20, ..crate::borg_item::blank_item() };
        let better = borg_item { iqty: 1, to_a: 4, ..robe };
        let best = borg_item { to_a: 6, ..better };
        let empty = crate::borg_item::blank_item();
        st.items.items[..wield].fill(empty);
        st.store.shops[home].ware.fill(empty);
        if case == "weight-blocked-upgrade" {
            st.items.items[worn_slot] = borg_item { iqty: 1, ..robe };
            st.items.items[0] = better;
            st.store.shops[home].ware[0] = best;
            st.borg.trait_[BI_WEIGHT as usize] = st.borg.tr(BI_CARRY) / 2 - 5;
            assert_eq!(crate::borg_town_live::shopping_weight_capacity(st, best.weight), 0);
            let plan = gear_plan(st, worn_slot, true);
            assert_eq!(plan.upgrade, Some((Pack(0), worn_slot)));
            assert!(plan.stock.iter().any(|entry| entry.source == Home(0) && entry.release == 0));
            st.items.items[0].desc[0] = b'!' as core::ffi::c_char;
            assert_eq!(gear_plan(st, worn_slot, true).upgrade, None);
            println!("#1264 exploration pack passed: {case}");
            std::process::exit(0);
        }
        st.items.items[0] = robe;
        st.items.items[worn_slot] = best;
        st.store.shops[home].ware[1] = better;
        assert_eq!(redundant_copies(st, Pack(0), 2), 5, "worn and Home clear the valuable stack");
        assert_eq!(redundant_copies(st, Pack(0), 3), 4, "keep one copy without a third better option");
        st.store.shops[home].ware[0] = robe;
        st.store.shops[home].ware[1] = empty;
        st.items.items[0] = better;
        assert_eq!(redundant_copies(st, Home(0), 2), 5, "worn and pack clear the Home stack");
        st.items.items[worn_slot] = empty;
        st.store.shops[home].ware[1] = best;
        assert_eq!(redundant_copies(st, Home(0), 2), 5, "pack and Home clear the Home stack");
        st.store.shops[home].ware[0] = empty;
        assert_eq!(redundant_copies(st, Pack(0), 2), 0, "the last two choices stay");
        st.items.items[0] = borg_item { iqty: 1, ..robe };
        st.items.items[0].el_info[0].res_level = 1;
        st.items.items[worn_slot] = better;
        assert_eq!(redundant_copies(st, Pack(0), 2), 0, "keep the only resistance swap");

        st.items.items[0] = borg_item { ident: false, needs_ident: true, ..robe };
        for (index, byte) in b"{??}".iter().enumerate() {
            st.items.items[0].desc[index] = *byte as core::ffi::c_char;
        }
        st.items.items[worn_slot] = best;
        st.store.shops[home].ware[1] = better;
        st.borg.trait_[BI_AID as usize] = 1;
        let plan = gear_plan(st, worn_slot, true);
        assert!(plan.stock.iter().any(|entry| entry.source == Pack(0) && entry.release == 4),
            "trim the valuable uninspected stack while keeping one copy for identification");
        st.borg.trait_[BI_AID as usize] = 0;
        let plan = gear_plan(st, worn_slot, true);
        assert!(plan.stock.iter().any(|entry| entry.source == Pack(0) && entry.release == 5),
            "two retained alternatives cover the uninspected armour");
        st.store.shops[home].ware[1] = empty;
        let plan = gear_plan(st, worn_slot, true);
        assert!(plan.stock.iter().any(|entry| entry.source == Pack(0) && entry.release == 4),
            "keep one unresolved armour copy when fewer than two alternatives cover it");

        let weapon = borg_item { iqty: 1, aware: true, ident: true, weight: 40,
            tval: TV_SWORD as u8, dd: 2, ds: 6, to_h: 10, to_d: 20, ..empty };
        st.items.items[wield] = weapon;
        st.store.shops[home].ware[1] = borg_item { to_d: 19, ..weapon };
        let mut spare = borg_item { iqty: 2, ident: false, needs_ident: true, to_h: 0, to_d: 0, ..weapon };
        for (index, byte) in b"{??}".iter().enumerate() {
            spare.desc[index] = *byte as core::ffi::c_char;
        }
        st.items.items[0] = spare;
        let release = |st: &mut BorgState, include_home| {
            gear_plan(st, wield, include_home).stock.iter()
                .find(|entry| entry.source == Pack(0)).map_or(0, |entry| entry.release)
        };
        assert_eq!(release(st, true), 2, "worn weapon and Home backup cover the stack");
        assert_eq!(release(st, false), 0, "one alternative is insufficient");
        st.items.items[0].activ_idx = 1;
        assert_eq!(release(st, true), 0, "protect a known activation despite unresolved runes");
        st.items.items[0].activ_idx = 0;
        st.borg.trait_[BI_AID as usize] = 1;
        assert_eq!(release(st, true), 0, "give identification priority");
        st.borg.trait_[BI_AID as usize] = 0;

        let undead = crate::headers::list_mon_race_flags_h::RF_UNDEAD as usize;
        st.items.items[0].slays[undead] = 3;
        st.items.items[wield].slays[undead] = 3;
        st.store.shops[home].ware[1].slays[undead] = 3;
        assert_eq!(release(st, true), 2, "both alternatives cover the slay damage");
        st.store.shops[home].ware[1].dd = 1;
        st.store.shops[home].ware[1].ds = 2;
        st.store.shops[home].ware[1].to_d = 10;
        assert_eq!(release(st, true), 0,
            "keep the spare when the Home backup wins on base damage but loses against undead");
        println!("#1264 exploration pack passed: {case}");
        std::process::exit(0);
    }
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

        let peer_slot = st.items.items[..wield].iter().position(|item| u32::from(item.tval) == TV_FOOD)
            .expect("replace the fixture food while comparing two melee alternatives");
        let peer_before = st.items.items[peer_slot];
        let mut slayer = borg_item { ident: true, needs_ident: false, ..trident };
        let dragon = crate::headers::list_mon_race_flags_h::RF_DRAGON as usize;
        slayer.slays[dragon] = 3;
        slayer.ego_idx = 1;
        st.items.items[slot] = slayer;
        assert!(!inferior(st, slot), "keep the only dragon swap");
        st.items.items[peer_slot] = borg_item { to_h: slayer.to_h + 1, to_d: slayer.to_d + 1, ..slayer };
        assert!(inferior(st, slot), "the better carried slayer fills the same role");
        assert!(!inferior(st, peer_slot), "keep the better slayer");
        st.items.items[slot].activ_idx = 1;
        assert!(!inferior(st, slot), "the slayer has its own activation");
        st.items.items[slot] = slayer;
        st.items.items[slot].slays[dragon] = 5;
        assert!(!inferior(st, slot), "the weaker slay cannot cover higher dragon damage");
        st.items.items[slot] = trident;
        st.items.items[peer_slot] = peer_before;

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
    fn cloak_backup_keeps_stealth_and_requires_a_pound_for_an_extra_copy() {
        let cloak = |source, power, weight, stealth| {
            let mut item = borg_item { iqty: 1, tval: TV_CLOAK as u8, weight,
                to_a: power as i16, ..crate::borg_item::blank_item() };
            item.modifiers[crate::headers::list_object_modifiers_h::OBJ_MOD_STEALTH as usize] = stealth;
            GearChoice { source, item, power, lighter: true, backup: true, wearable: true }
        };
        let choices = [
            cloak(GearLocation::Worn(0), 20, 25, 0),
            cloak(GearLocation::Home(0), 18, 25, 0),
            cloak(GearLocation::Pack(0), 10, 25, 2),
            cloak(GearLocation::Pack(1), 8, 16, 0),
            cloak(GearLocation::Pack(2), 7, 15, 0),
        ];
        assert_eq!(backup_counts(&choices, &[0], 1), [1, 0, 1, 0, 1]);
    }

    #[test]
    fn better_options_count_across_all_three_locations() {
        use GearLocation::{Home, Pack, Worn};
        let covers = |have: &i32, need: &i32| have >= need;
        for peers in [[(Worn(0), 6, 1), (Home(0), 5, 1)], [(Worn(0), 6, 1), (Pack(0), 5, 1)],
            [(Home(0), 6, 1), (Pack(0), 5, 1)]]
        {
            for spare in [Pack(1), Home(1)] {
                assert_eq!(covered_copies(&(spare, 1), &peers, 5, 1, 2, covers), 5);
                assert_eq!(covered_copies(&(spare, 1), &peers, 5, 1, 3, covers), 4);
            }
        }
    }

    #[test]
    fn cleanup_keeps_one_equivalent_option_and_preserves_distinct_roles() {
        use GearLocation::{Home, Pack, Worn};
        let covers = |have: &(i32, bool), need: &(i32, bool)| have.0 >= need.0 && (have.1 || !need.1);
        let mut gear = vec![(Worn(0), (8, false), 1), (Home(0), (8, false), 1), (Home(1), (7, false), 1),
            (Pack(0), (6, false), 1), (Pack(1), (5, true), 1)];
        // The same value on two stacks counts as one better option.
        assert_eq!(covered_copies(&(Pack(2), (7, false)), &gear[..2], 1, 1, 2, covers), 0);
        assert_eq!(covered_copies(&(Home(0), (8, false)), &gear[..1], 3, 2, 2, covers), 2);
        for i in (1..gear.len()).rev() {
            let (source, quality, copies) = gear.remove(i);
            if covered_copies(&(source, quality), &gear, copies, 1, 2, covers) == 0 {
                gear.insert(i, (source, quality, copies));
            }
        }
        assert_eq!(gear, [(Worn(0), (8, false), 1), (Home(1), (7, false), 1), (Pack(1), (5, true), 1)]);
    }

    #[test]
    fn an_uninspected_weapon_keeps_its_distinct_observed_roles() {
        let mut spare = crate::borg_item::blank_item();
        let candidate = spare;
        let mut trial = Melee { dice: 8, bonus: 0, blows: 100, hit: 10, armor: 10, speed: 110, saving_throw: 40 };
        let primary = Melee { dice: 12, ..trial };
        let bit = (OF_FEATHER as i32 - engine::FLAG_START) as usize;
        spare.flags[bit / 8] |= 1 << (bit % 8);
        spare.el_info[0].res_level = 1;
        spare.modifiers[OBJ_MOD_WIS as usize] = 3;
        let other_worn = spare;
        assert!(!observed_melee_properties_cover(&candidate, &primary, &spare, &trial, &[], false));
        assert!(observed_melee_properties_cover(&candidate, &primary, &spare, &trial, &[&other_worn], false));
        assert!(!observed_melee_properties_cover(&candidate, &primary, &spare, &trial, &[&other_worn], true));
        trial.saving_throw += 1;
        assert!(!observed_melee_properties_cover(&candidate, &primary, &spare, &trial, &[&other_worn], false));
        trial.saving_throw -= 1;
        spare.slays[1] = 3;
        assert!(!observed_melee_properties_cover(&candidate, &primary, &spare, &trial, &[&other_worn], false));
    }

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
    fn home_armour_keeps_uncovered_resistance_and_lighter_kits() {
        let mut spare = borg_item { iqty: 1, ident: true, aware: true, weight: 60,
            tval: TV_SOFT_ARMOR as u8, ac: 12, to_a: 1, ..crate::borg_item::blank_item() };
        let better = borg_item { to_a: 7, ..spare };
        spare.el_info[0].res_level = 1;
        let shield = spare;
        assert!(!static_gear_covers(&better, &spare, &[], true));
        assert!(static_gear_covers(&better, &spare, &[&shield], true));
        spare.weight -= 1;
        assert!(!static_gear_covers(&better, &spare, &[&shield], true));
        assert!(!static_gear_covers(&spare, &spare, &[], false));
        assert!(static_gear_covers(&spare, &spare, &[], true));
    }

    #[test]
    fn launcher_comparison_keeps_ammunition_and_accuracy_tradeoffs() {
        let sling = borg_item { iqty: 1, ident: true, aware: true, tval: TV_BOW as u8,
            sval: 1, pval: 2, weight: 5, to_h: 17, to_d: 6, ..crate::borg_item::blank_item() };
        let plain = borg_item { to_h: 5, to_d: 4, ..sling };
        let damage_sling = borg_item { to_h: 15, to_d: 8, ..sling };
        let bow = borg_item { sval: 2, ..plain };
        assert!(static_gear_covers(&sling, &plain, &[], false));
        assert!(!static_gear_covers(&sling, &damage_sling, &[], true));
        assert!(!static_gear_covers(&damage_sling, &sling, &[], true));
        assert!(!static_gear_covers(&sling, &bow, &[], true));
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
