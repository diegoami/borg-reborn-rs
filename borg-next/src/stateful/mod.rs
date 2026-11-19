//! Stateful R2 policies. All external queries/effects use decision_api;
//! every BI read uses the named native-memory view. No direct engine API.
use borg_rs::{
    decisions::{Decision, ReadDecision, Value},
    memory::BorgState,
};
pub mod attack;
pub mod caution;
pub mod defence;
pub mod escape;
pub mod item_use;
pub mod power;
pub mod recovery;
pub fn decide(st: &mut BorgState, request: Decision) -> Value {
    match request {
        Decision::Power => Value::Score(power::borg_power(st)),
        Decision::ItemQuaffCrit { no_check } => {
            Value::Acted(item_use::borg_quaff_crit(st, no_check))
        }
        Decision::ItemQuaffPotion { sval } => Value::Acted(item_use::borg_quaff_potion(st, sval)),
        Decision::ItemQuaffUnknown => Value::Acted(item_use::borg_quaff_unknown(st)),
        Decision::ItemReadScroll { sval } => Value::Acted(item_use::borg_read_scroll(st, sval)),
        Decision::ItemReadUnknown => Value::Acted(item_use::borg_read_unknown(st)),
        Decision::ItemEat { tval, sval } => Value::Acted(item_use::borg_eat(st, tval, sval)),
        Decision::ItemEatUnknown => Value::Acted(item_use::borg_eat_unknown(st)),
        Decision::ItemEatFoodAny => Value::Acted(item_use::borg_eat_food_any(st)),
        Decision::ItemEquipsRod { sval } => Value::Acted(item_use::borg_equips_rod(st, sval)),
        Decision::ItemZapRod { sval } => Value::Acted(item_use::borg_zap_rod(st, sval)),
        Decision::ItemUseStaff { sval } => Value::Acted(item_use::borg_use_staff(st, sval)),
        Decision::ItemUseUnknown => Value::Acted(item_use::borg_use_unknown(st)),
        Decision::ItemUseStaffFail { sval } => {
            Value::Acted(item_use::borg_use_staff_fail(st, sval))
        }
        Decision::ItemEquipsStaffFail { sval } => {
            Value::Acted(item_use::borg_equips_staff_fail(st, sval))
        }
        Decision::ItemAimWand { sval } => Value::Acted(item_use::borg_aim_wand(st, sval)),
        Decision::ItemEquipsRing { ring_sval } => {
            Value::Acted(item_use::borg_equips_ring(st, ring_sval))
        }
        Decision::ItemActivateRing { ring_sval } => {
            Value::Acted(item_use::borg_activate_ring(st, ring_sval))
        }
        Decision::ItemEquipsDragon { drag_sval } => {
            Value::Acted(item_use::borg_equips_dragon(st, drag_sval))
        }
        Decision::ItemActivateDragon { drag_sval } => {
            Value::Acted(item_use::borg_activate_dragon(st, drag_sval))
        }
        Decision::ItemActivateItem { activation } => {
            Value::Acted(item_use::borg_activate_item(st, activation))
        }
        Decision::ItemEquipsItem {
            activation,
            check_charge,
        } => Value::Acted(item_use::borg_equips_item(st, activation, check_charge)),
        Decision::ItemActivateFailure { tval, sval } => {
            Value::Score(item_use::borg_activate_failure(st, tval, sval))
        }
        Decision::ItemUseThings => Value::Acted(item_use::borg_use_things(st)),
        Decision::ItemRecharging => Value::Acted(item_use::borg_recharging(st)),

        Decision::Caution => Value::Acted(caution::borg_caution(st)),
        Decision::Recall => Value::Acted(escape::borg_recall(st)),
        Decision::Surrounded => Value::Acted(escape::borg_surrounded(st)),
        Decision::Freedom { y, x } => Value::Score(escape::borg_freedom(st, y, x)),
        Decision::CautionPhase { emergency, turns } => {
            Value::Acted(escape::borg_caution_phase(st, emergency, turns))
        }
        Decision::CautionTeleport { emergency, turns } => {
            Value::Acted(escape::borg_caution_teleport(st, emergency, turns))
        }
        Decision::AllowTeleport => Value::Acted(escape::borg_allow_teleport(st)),
        Decision::ShadowShift { allow_fail } => {
            Value::Acted(escape::borg_shadow_shift(st, allow_fail))
        }
        Decision::DimensionDoor { allow_fail } => {
            Value::Acted(escape::borg_dimension_door(st, allow_fail))
        }
        Decision::Escape { b_q } => Value::Acted(escape::borg_escape(st, b_q)),
        Decision::Recover => Value::Acted(recovery::borg_recover(st)),
        Decision::Defend { p1 } => Value::Acted(defence::borg_defend(st, p1)),
        Decision::AttackLaunchBolt {
            rad,
            dam,
            typ,
            max,
            ammo_location,
        } => Value::Score(attack::borg_launch_bolt(
            st,
            rad,
            dam,
            typ,
            max,
            ammo_location,
        )),
        Decision::AttackAuxLaunch => Value::Score(attack::borg_attack_aux_launch(st)),
        Decision::AttackSpellBolt {
            spell,
            rad,
            dam,
            typ,
            max_range,
            is_arc,
        } => Value::Score(attack::borg_attack_aux_spell_bolt(
            st, spell, rad, dam, typ, max_range, is_arc,
        )),
        Decision::AttackEffectiveness { attack_type } => {
            Value::Score(attack::borg_calculate_attack_effectiveness(st, attack_type))
        }
        Decision::Attack { boosted_bravery } => {
            Value::Acted(attack::borg_attack(st, boosted_bravery))
        }
    }
}

fn set_mana(st: &mut BorgState, value: i32) {
    st.borg.trait_[borg_rs::memory::traits::BI_CURSP as usize] = value;
}

fn set_confusion_resistance(st: &mut BorgState, value: i32) {
    st.borg.trait_[borg_rs::memory::traits::BI_RCONF as usize] = value;
}

fn set_free_action(st: &mut BorgState, value: i32) {
    st.borg.trait_[borg_rs::memory::traits::BI_FRACT as usize] = value;
}

/// Compiler-enforced read-only entry dispatch, with no caller state clone.
pub fn read_decide(st: &BorgState, request: ReadDecision) -> Value {
    match request {
        ReadDecision::PowerEquipment => Value::Score(power::borg_power_equipment(st)),
        ReadDecision::ItemEquipsItem {
            activation,
            check_charge,
        } => Value::Acted(item_use::borg_equips_item(st, activation, check_charge)),
    }
}
