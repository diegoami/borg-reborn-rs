//! Stateful R2 policies. All external queries/effects use decision_api;
//! every BI read uses the named native-memory view. No direct engine API.
use borg_rs::{
    decisions::{Decision, ReadDecision, Value},
    memory::BorgState,
};
pub mod attack;
mod attack_preference;
pub mod caution;
pub mod defence;
pub mod equipment_identify;
pub mod equipment_wear;
pub mod escape;
mod fast_fragile;
mod flow;
pub mod home_power;
pub mod item_use;
pub mod power;
pub mod recovery;
pub mod store_buy;
pub mod store_sell;
pub mod think_store;
pub fn decide(st: &mut BorgState, request: Decision) -> Value {
    match request {
        Decision::FlowPlanDownStair { why, brave } => {
            Value::DownStair(flow::plan_down_stair(st, why, brave))
        }
        Decision::HomePower => Value::Score(home_power::borg_power_home(st)),
        Decision::StoreBuyUseful => Value::Acted(store_buy::borg_think_shop_buy_useful(st)),
        Decision::HomeBuyUseful => Value::Acted(store_buy::borg_think_home_buy_useful(st)),
        Decision::StoreGrabInteresting => {
            Value::Acted(store_buy::borg_think_shop_grab_interesting(st))
        }
        Decision::HomeGrabUseless => Value::Acted(store_buy::borg_think_home_grab_useless(st)),
        Decision::HomeBuySwapWeapon => Value::Acted(store_buy::borg_think_home_buy_swap_weapon(st)),
        Decision::HomeBuySwapArmour => Value::Acted(store_buy::borg_think_home_buy_swap_armour(st)),
        Decision::StoreBuy => Value::Acted(store_buy::borg_think_shop_buy(st)),
        Decision::StoreTakeForSale => Value::Acted(store_buy::take_selected_for_sale(st)),
        Decision::StoreConfirmPurchase {
            item,
            shop,
            keep,
            qty,
            paid,
        } => {
            store_buy::confirm_purchase(st, &item.0, shop, keep, qty, paid);
            Value::Unit
        }
        Decision::StoreMinimumQuantity { item } => {
            Value::Score(store_sell::borg_min_item_quantity(st, &item.0))
        }
        Decision::HomeSellUseful {
            mut best_home_power,
        } => {
            let acted = store_sell::borg_think_home_sell_useful(st, &mut best_home_power);
            Value::HomeSale {
                acted,
                best_home_power,
            }
        }
        Decision::StoreSellUseless => Value::Acted(store_sell::borg_think_shop_sell_useless(st)),
        Decision::StoreSell => Value::Acted(store_sell::borg_think_shop_sell(st)),
        Decision::StoreSellQuantity { quantity } => {
            Value::Acted(store_sell::sell_selected_quantity(st, quantity))
        }
        Decision::StoreCountSell => Value::Score(store_sell::borg_count_sell(st)),
        Decision::StoreChooseShop => Value::Acted(think_store::borg_choose_shop(st)),
        Decision::StoreThink => Value::Acted(think_store::borg_think_store(st)),
        Decision::WearTestStuff => Value::Acted(equipment_wear::borg_test_stuff(st)),
        Decision::WearSwapRings => Value::Acted(equipment_wear::borg_swap_rings(st)),
        Decision::WearRings => Value::Acted(equipment_wear::borg_wear_rings(st)),
        Decision::WearBackupSwap { p } => Value::Acted(equipment_wear::borg_backup_swap(st, p)),
        Decision::WearStuff => Value::Acted(equipment_wear::borg_wear_stuff(st)),
        Decision::WearBestStuff => Value::Acted(equipment_wear::borg_best_stuff(st)),
        Decision::WearClearBest => {
            equipment_wear::borg_clear_best(st);
            Value::Unit
        }
        Decision::WearRecharge => Value::Acted(equipment_wear::borg_wear_recharge(st)),
        Decision::ObjectFullyId => Value::Acted(equipment_identify::borg_object_fully_id(st)),
        Decision::Power => Value::Score(power::borg_power(st)),
        Decision::ItemQuaffCrit { no_check } => {
            Value::Acted(item_use::borg_quaff_crit(st, no_check))
        }
        Decision::ItemQuaffPotion { sval } => Value::Acted(item_use::borg_quaff_potion(st, sval)),
        Decision::ItemReadScroll { sval } => Value::Acted(item_use::borg_read_scroll(st, sval)),
        Decision::ItemEat { tval, sval } => Value::Acted(item_use::borg_eat(st, tval, sval)),
        Decision::ItemEatFoodAny => Value::Acted(item_use::borg_eat_food_any(st)),
        Decision::ItemEquipsRod { sval } => Value::Acted(item_use::borg_equips_rod(st, sval)),
        Decision::ItemZapRod { sval } => Value::Acted(item_use::borg_zap_rod(st, sval)),
        Decision::ItemUseStaff { sval } => Value::Acted(item_use::borg_use_staff(st, sval)),
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
        Decision::FightBeforeFlight => Value::Acted(caution::fight_before_flight(st)),
        Decision::AttackOnSight => Value::Acted(attack::attack_on_sight(st)),
        Decision::RecallWaitNoExplore => Value::Acted(caution::recall_wait_no_explore(st)),
        Decision::LightlessSelfCare { danger } => {
            Value::Acted(caution::lightless_self_care(st, danger))
        }
        Decision::HoldRestForMonster => Value::Acted(recovery::holding_for_monster(st)),
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
        ReadDecision::FlowSelectStep => Value::Score(flow::select_step(st)),
        ReadDecision::FlowPlanUpStair { why } => Value::UpStair(flow::plan_up_stair(st, why)),
        ReadDecision::HomePower => Value::Score(home_power::borg_power_home(st)),
        ReadDecision::StoreMinimumQuantity { item } => {
            Value::Score(store_sell::borg_min_item_quantity(st, &item.0))
        }
        ReadDecision::StoreDisabled => Value::Acted(think_store::town_strategy_disabled(st)),
        ReadDecision::PowerEquipment => Value::Score(power::borg_power_equipment(st)),
        ReadDecision::ItemEquipsItem {
            activation,
            check_charge,
        } => Value::Acted(item_use::borg_equips_item(st, activation, check_charge)),
    }
}
