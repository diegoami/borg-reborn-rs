//! Stateful R2 policies. All external queries/effects use decision_api;
//! every BI read uses the named native-memory view. No direct engine API.
use borg_rs::{
    decisions::{Decision, Value},
    memory::BorgState,
};
pub mod attack;
pub mod caution;
pub mod escape;
pub mod recovery;
pub fn decide(st: &mut BorgState, request: Decision) -> Value {
    match request {
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
