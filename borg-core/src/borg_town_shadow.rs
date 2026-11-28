//! Town-plan evaluation shared by live execution and optional recording.
//!
//! `borg_town_plan = 1` feeds the selected action to `borg_town_live`;
//! no record file is required. With it off, the historical chooser acts,
//! and `BORG_TOWN_PLAN_RECORD` optionally records a comparison only.
//! The evaluator restores its try-on/try-off probes. Only the live executor
//! queues shop actions. A record's `mode` distinguishes the two paths.
//!
//! What the step can price today:
//!
//! * **light** - light sources, worn and spare (the need counts sources,
//!   decided 2026-11-13); a ware of the light tval closes one;
//! * **food** - what `borg_trait` counts in `BI_FOOD`, one per copy;
//! * **bleeding** - Cure Light, Serious or Critical Wounds, one per copy;
//! * **poison** - Neutralize Poison or a poison-curing food, one per copy;
//! * **escape** - Phase Door, one per copy (`BI_APHASE`).
//!
//! **Power** comes from the try-on ([`try_on`]): each wearable is put on
//! a copy-and-restore of the state and `borg_power` measures the gain,
//! which the power need's curve prices. Not yet: **knowledge** and **trip**,
//! which read 0 until the catalogue of useful kinds (#35 item 4) exists,
//! as phase 1 decided. A ware serves one need here; a food that also cures
//! poison is priced as food, and its poison urgent need is still in `serves`.

use std::sync::atomic::{AtomicU64, Ordering};

use crate::borg_town_home::{self, Role};
use crate::borg_headers::borg_h::{BORG_TOWN_PLAN, BORG_TOWN_W_KNOWLEDGE};
use crate::borg_headers::borg_item_h::borg_item;
use crate::borg_headers::borg_trait_h::BI_CDEPTH;
use crate::borg_item::inven_wield;
use crate::borg_town_budget::{Budget, UrgentPurchase};
use crate::borg_store::BORG_HOME;
use crate::borg_town_dispose::{Disposition, KitItem, Place};
use crate::borg_town_record::{first_action, step, PlanRecord, StepInput, TryOn, WareFact, PARAMS};
use crate::borg_town_solve::Limits;
use crate::borg_town_stock::{needs_served, Stock};
use crate::borg_town_usefulness::{Need, Trip};
use crate::engine;
use crate::headers::list_tvals_h::TV_LIGHT;
use crate::params::{value as param, Param};
use crate::state::BorgState;

const GENERAL_STORE: usize = 0;

/// The 24 town settings the plan step actually runs on (borg-reborn-rs, #460;
/// owner 2026-11-16: "turn off the buying of useless unknown objects").
/// Buy-to-identify is off in every arm: a flavour-unknown ware's kind is
/// hidden (supplies() prices only aware kinds), so the knowledge weight is the
/// only value it can have, and it bought one of each flavour (Monster
/// Confusion, Resist Heat, ...) for 25 points each. The recorded `params` are
/// these, so a replay reproduces the plan; `configured_params` keeps
/// borg_town_w_knowledge as configured. The same zero under borg_trip_policy = 1
/// (#444, Weights::of) never took effect in the step: it rebuilds its settings
/// from these 24 values, and the trip policy is not one of them.
pub fn effective_params(mut params: [i64; PARAMS]) -> [i64; PARAMS] {
    params[(BORG_TOWN_W_KNOWLEDGE - BORG_TOWN_PLAN) as usize] = 0;
    params
}

/// The index of a need in [`Need::ALL`].
fn need_index(need: Need) -> i8 {
    Need::ALL.iter().position(|&n| n == need).map_or(-1, |i| i as i8)
}

/// The need a ware serves and what one copy closes of it - the reduction
/// table, as far as the needs that are priced today go (see the module
/// doc). `(-1, 0)` for a ware that serves none of them.
fn ware_role(st: &BorgState, item: &borg_item) -> (i8, i64) {
    use crate::borg_town_utility::UNIT;
    use crate::headers::list_tvals_h::{TV_FOOD, TV_POTION};
    let hp = i64::from(st.borg.tr(crate::borg_headers::borg_trait_h::BI_MAXHP)).max(1);
    if u32::from(item.tval) == TV_POTION {
        let v = &st.item_val;
        let sval = i32::from(item.sval);
        let heal = if sval == v.sv_potion_cure_light { 20 }
            else if sval == v.sv_potion_cure_serious { 40 }
            else if sval == v.sv_potion_cure_critical { 60 }
            else if sval == v.sv_potion_healing { 300 }
            else if sval == v.sv_potion_star_healing { 1200 }
            else if sval == v.sv_potion_life { hp }
            else { 0 };
        if heal > 0 { return (need_index(Need::Healing), heal.min(hp) * UNIT); }
    }

    use crate::headers::list_tvals_h::{TV_LIGHT, TV_SCROLL};
    let tval = u32::from(item.tval);
    if tval == TV_LIGHT {
        return (need_index(Need::Light), UNIT);
    }
    if tval == TV_SCROLL && i32::from(item.sval) == st.item_val.sv_scroll_phase_door {
        return (need_index(Need::Escape), UNIT);
    }
    if tval == TV_SCROLL && i32::from(item.sval) == st.item_val.sv_scroll_word_of_recall {
        return (need_index(Need::Trip), UNIT);
    }
    if tval == TV_SCROLL && i32::from(item.sval) == st.item_val.sv_scroll_satisfy_hunger {
        return (need_index(Need::Food), UNIT);
    }
    // The urgent needs' classification already mirrors borg_trait's counting
    // of food, cures and poison cures; its first need is the ware's.
    match needs_served(st, item).first() {
        Some(&n) => {
            let low_food = tval == TV_FOOD && [st.item_val.sv_food_apple,
                st.item_val.sv_food_handful, st.item_val.sv_food_slime_mold,
                st.item_val.sv_food_pint, st.item_val.sv_food_sip].contains(&i32::from(item.sval));
            (need_index(n), if n == Need::Food && low_food { UNIT / 3 } else { UNIT })
        },
        None => (-1, 0),
    }
}

/// The try-on: for every wearable ware, the power gained by wearing it -
/// the ware put on in its equipment slot, the worn item moved into a free
/// pack slot, `borg_notice` and `borg_power` run, exactly as the live
/// strategy evaluates a purchase (`borg_think_shop_buy_useful`). Returns
/// the try-ons and, per wearable candidate, its slot cost: one pack slot
/// if something is worn there now (it moves to the pack), none if the
/// slot is empty.
///
/// **The state is restored by assignment, not recomputed**: every part a
/// try-on can write - `borg` (traits, `has`, activations, the stat-gain
/// arrays, `ready_morgoth`), `traits`, `trait_swap`, the items,
/// `prepared.buffer` and `think.do_spell` - is snapshotted first and put
/// back after, so the observation stays observation. Light sources use equipment-only power
/// for their properties, with endurance supplied by the capability model.
/// Diggers are evaluated as tools rather than worn as weapons. Without a free pack slot
/// there is nowhere to put the worn item, and nothing is tried on.
fn try_on(st: &mut BorgState, stock: &Stock) -> (Vec<TryOn>, Vec<(usize, i32)>) {
    use crate::headers::list_tvals_h::TV_DIGGING;
    let hole = crate::borg_inventory::borg_first_empty_inventory_slot(st);
    let Ok(hole) = usize::try_from(hole) else {
        return (Vec::new(), Vec::new());
    };
    let wield = inven_wield() as usize;
    let buy_spare_torches = param(Param::town_light_reserve_turns) > 0
        && crate::borg_light::torch_fuel_turns(st).is_some();
    let wearables: Vec<(usize, usize)> = stock
        .candidates
        .iter()
        .enumerate()
        .filter_map(|(i, c)| {
            let w = &st.store.shops[usize::from(c.shop)].ware[usize::from(c.slot)];
            let tval = u32::from(w.tval);
            // General Store torches can refill the one already worn. Price
            // them as pack supplies, where the planner can choose a stack.
            if tval == TV_DIGGING || buy_spare_torches && usize::from(c.shop) == GENERAL_STORE
                && tval == TV_LIGHT && i32::from(w.sval) == st.item_val.sv_light_torch
            {
                return None;
            }
            let eq = crate::borg_inventory::borg_wield_slot(w);
            usize::try_from(eq).ok().map(|eq| (i, eq))
        })
        .collect();
    if wearables.is_empty() {
        return (Vec::new(), Vec::new());
    }

    // The snapshot of everything a try-on can write.
    let borg = st.borg.clone();
    let traits = st.traits.clone();
    let trait_swap = st.trait_swap.clone();
    let items = st.items.items.clone();
    let buffer = st.prepared.buffer.clone();
    let do_spell = st.think.do_spell;

    let base = i64::from(crate::borg_power::borg_power(st));
    let base_light = i64::from(crate::borg_power::borg_power_equipment(st));
    let mut tryon = Vec::new();
    let mut slots = Vec::new();
    for (i, eq) in wearables {
        let c = &stock.candidates[i];
        let mut ware = st.store.shops[usize::from(c.shop)].ware[usize::from(c.slot)];
        ware.iqty = 1;
        let worn = items[eq];
        st.items.items[hole] = worn;
        st.items.items[eq] = ware;
        crate::borg_trait::borg_notice(st, true);
        let (p, before) = if u32::from(ware.tval) == TV_LIGHT {
            (i64::from(crate::borg_power::borg_power_equipment(st)), base_light)
        } else { (i64::from(crate::borg_power::borg_power(st)), base) };
        st.items.items[eq] = items[eq];
        st.items.items[hole] = items[hole];
        let lantern_preference = crate::params::value(crate::params::Param::town_lantern_upgrade_power);
        let lantern_bonus = if lantern_preference > 0 && u32::from(ware.tval) == TV_LIGHT
            && i32::from(ware.sval) == st.item_val.sv_light_lantern && ware.timeout > 0 && !ware.cursed
            && u32::from(worn.tval) == TV_LIGHT
            && i32::from(worn.sval) == st.item_val.sv_light_torch && p >= before
            && !engine::player_has(crate::headers::list_player_flags_h::PF_UNLIGHT)
            && !engine::flag_has(&worn.flags, crate::headers::list_object_flags_h::OF_STICKY as i32)
            && !engine::flag_has(&worn.flags, crate::headers::list_object_flags_h::OF_NO_FUEL as i32)
        {
            i64::from(lantern_preference)
        } else { 0 };
        tryon.push(TryOn { shop: c.shop, slot: c.slot, eq_slot: (eq - wield) as u8, value: p - before + lantern_bonus });
        slots.push((i, i32::from(worn.iqty != 0)));
    }

    st.borg = borg;
    st.traits = traits;
    st.trait_swap = trait_swap;
    st.items.items = items;
    st.prepared.buffer = buffer;
    st.think.do_spell = do_spell;
    (tryon, slots)
}

/// Consumables receive utility from their effects, never from inventory
/// power's level-gated bonuses. Durable gear/tools retain the try-off.
fn has_inventory_power(it: &borg_item) -> bool {
    use crate::headers::list_tvals_h::{TV_FLASK, TV_FOOD, TV_MUSHROOM, TV_POTION, TV_SCROLL};
    !matches!(u32::from(it.tval), TV_FLASK | TV_FOOD | TV_MUSHROOM | TV_POTION | TV_SCROLL)
        && crate::borg_inventory::borg_wield_slot(it) >= 0
}

fn unaware_device(it: &borg_item) -> bool {
    use crate::headers::list_tvals_h::{TV_ROD, TV_STAFF, TV_WAND};
    !it.aware && matches!(u32::from(it.tval), TV_ROD | TV_STAFF | TV_WAND)
}

fn unknown_durable(it: &borg_item) -> bool {
    use crate::headers::list_tvals_h::{TV_FLASK, TV_FOOD, TV_MUSHROOM, TV_POTION, TV_SCROLL};
    (!it.ident || !it.aware || it.needs_ident)
        && !matches!(u32::from(it.tval), TV_FLASK | TV_FOOD | TV_MUSHROOM | TV_POTION | TV_SCROLL)
}

/// borg-reborn-rs (#1151): the light turns one copy holds, in `UNIT`s: a
/// torch's or lantern's turns, a flask's fuel (`object.txt`: pval 7500).
fn fuel_per_copy(it: &borg_item) -> i64 {
    use crate::headers::list_tvals_h::{TV_FLASK, TV_LIGHT};
    let unit = crate::borg_town_utility::UNIT;
    match u32::from(it.tval) {
        TV_LIGHT => i64::from(it.timeout).max(0) * unit,
        TV_FLASK => i64::from(it.pval).max(0) * unit,
        _ => 0,
    }
}

fn apparently_saleable(it: &borg_item) -> bool {
    // The engine refuses apparently worthless goods. Unknown durable
    // rune exceptions under no-selling are protected, not disposal stock.
    it.value > 0
}

/// For a capped carried kind, the Home floor protects keeper withdrawals.
/// Reserve the purchase-history shield for supplies bought from a shop.
pub(crate) fn kit_just_bought(st: &BorgState, item: &borg_item, place: Place, home_at_limit: bool) -> bool {
    let bought = &st.buy;
    let count = usize::try_from(bought.bought_item_num).unwrap_or(0).min(10);
    (0..count).any(|i| {
        bought.bought_item_tval[i] == i32::from(item.tval)
            && bought.bought_item_sval[i] == i32::from(item.sval)
            && !(place == Place::Pack && home_at_limit && bought.bought_item_store[i] == BORG_HOME as i32)
    })
}

/// The arrival pass is responsible for trimming staples. The shopping
/// planner may still identify unknown flavours and store worthwhile gear.
fn protects_arrival_supply(st: &BorgState, item: &borg_item) -> bool {
    use crate::headers::list_tvals_h::*;
    if !item.aware || !item.flavour_aware || item.cursed {
        return false;
    }
    let iv = &st.item_val;
    let sval = i32::from(item.sval);
    match u32::from(item.tval) {
        TV_FLASK => sval == iv.sv_flask_oil,
        TV_LIGHT => sval == iv.sv_light_torch && item.ego_idx == 0,
        TV_ARROW | TV_BOLT | TV_SHOT => i32::from(item.tval) == st.borg.tr(crate::borg_headers::borg_trait_h::BI_AMMO_TVAL),
        TV_FOOD | TV_MUSHROOM | TV_POTION | TV_SCROLL => !crate::borg_town_capability::unwanted_home_item(st, item),
        _ => false,
    }
}

/// The kit: the pack's and the Home's stacks, with what the dispositions
/// read (slice 3). The power of a pack stack is a **try-off** - its
/// quantity set to 0, `borg_notice` and `borg_power` run - and of a Home
/// stack a try-on into a free pack slot; both under the same snapshot and
/// restore-by-assignment as [`try_on`]. A Home stack with no free pack slot
/// to try it in gets no power. The proceeds are what a shop pays: 0 under
/// `birth_no_selling` (the game's default), else two thirds of the value
/// (the Borg's own estimate, `borg_price_item`, without the Black Market's
/// halving). The shop that buys it is the first that `borg_store_buys`
/// names.
fn kit_of(st: &mut BorgState, live: bool) -> Vec<KitItem> {
    use crate::headers::list_options_h::OPT_birth_no_selling;
    let no_selling = engine::player_opt(OPT_birth_no_selling);
    // borg-reborn-rs (#1140, `deal_with_poison`): no sale (it pays nothing)
    // while the poison triage holds.
    let triage_no_sale = crate::borg_poison::no_sale_now(st);
    let pack_end = inven_wield() as usize;
    let home_policy = borg_town_home::enabled(st);
    let can_decurse = home_policy && crate::borg_item_decurse::can_decurse(st);
    let ranked_gear = live && crate::borg_junk::gear_alternatives() > 0;
    let preferred_dagger = (home_policy && !crate::borg_town_live::gear_home_backups())
        .then(|| borg_town_home::preferred_elemental_dagger(st)).flatten();
    let redundant_at_home: Vec<bool> = if home_policy {
        (0..st.store.shops[BORG_HOME].ware.len())
            .map(|slot| crate::borg_spare_gear::redundant_home_gear(st, slot)).collect()
    } else { Vec::new() };
    // #1315: the active swaps as the pack stands, before any try-off.
    let swap_slots = [st.trait_swap.weapon_swap - 1, st.trait_swap.armour_swap - 1];
    let fact = |st: &BorgState, it: &borg_item, place: Place, slot: usize, power: i64, worn_gain: i64| -> KitItem {
        let (mut need, mut reduction) = ware_role(st, it);
        let consumable = crate::borg_item_id::flavour_sale_type(it.tval);
        // borg-reborn-rs (#470): an unaware device is never used either; one
        // is sold, like an unknown potion, scroll or mushroom, to learn it.
        let unknown_flavour = !it.aware && (consumable || unaware_device(it));
        if unknown_flavour { need = -1; reduction = 0; }
        let sell_shop = (0..7i8).find(|&who| !triage_no_sale && apparently_saleable(it)
            && !crate::borg_town_live::sale_rejected(st, i16::from(who), it)
            && crate::borg_store_sell::borg_store_buys(it, i32::from(who))).unwrap_or(-1);
        let proceeds = if no_selling || sell_shop < 0 { 0 } else {
            match place {
                Place::Pack => engine::inven_object(&st.access, slot)
                    .map_or(0, |o| i64::from(o.sale_quote(sell_shop as usize, i32::from(it.iqty)))),
                Place::Home => engine::store_stock(&st.access, BORG_HOME).get(slot)
                    .map_or(0, |o| i64::from(o.sale_quote(sell_shop as usize, i32::from(it.iqty)))),
            }
        };
        let retained_at_home = place == Place::Home && crate::borg_town_live::retained_home_gear(st, slot);
        let redundant_dagger = !retained_at_home
            && borg_town_home::redundant_elemental_dagger(st, it, preferred_dagger);
        let redundant_home_gear = place == Place::Home
            && (redundant_dagger || redundant_at_home.get(slot) == Some(&true));
        let ranked_item = ranked_gear && crate::borg_spare_gear::can_rank(it) && !redundant_dagger;
        let at_limit = place == Place::Pack && crate::borg_town_live::home_at_limit(st, it);
        let just_bought = !redundant_home_gear && kit_just_bought(st, it, place, at_limit);
        let protected_supply = live && place == Place::Pack
            && crate::borg_town_live::unload_on_arrival(st) && protects_arrival_supply(st, it);
        // borg-reborn-rs (#1239): protect retained gear from ordinary Home moves.
        let home_room = if place == Place::Home {
            engine::store_stock(&st.access, BORG_HOME).get(slot).map(|offer| {
                let blocked = live && (crate::borg_town_live::buy_rejected(st, BORG_HOME as i16, slot)
                    || (!redundant_home_gear && crate::borg_town_live::home_moved(st, it, true))
                    || retained_at_home);
                let carry = if blocked { 0 } else { offer.carry_capacity() };
                let acquire = if live { crate::borg_town_live::acquisition_room(st, it.tval, it.kind) }
                    else { i32::from(it.iqty) };
                let acquire = if live { acquire.min(crate::borg_town_live::shopping_weight_capacity(st, it.weight)) }
                    else { acquire };
                [carry, offer.merge_capacity(), acquire.min(i32::from(it.iqty))]
            })
        } else { None };
        let mut home_role = if redundant_dagger || redundant_home_gear {
            Role::Reject
        } else if home_policy {
            borg_town_home::classify(st, it, can_decurse, worn_gain)
        } else { Role::Ordinary };
        if ranked_item && matches!(home_role, Role::Reject | Role::Renewable | Role::Ordinary) {
            // Ranked cleanup makes the disposal decision. The normal planner
            // may still put the spare at Home to save carried weight.
            home_role = Role::Gear;
        }
        // #1308: an unknown-rune spare is carried until identified or given
        // to a shop, never stored. Its unknown runes still protect it from
        // the planner's own sale or drop.
        if ranked_gear && crate::borg_town_live::gear_home_backups() && place == Place::Pack
            && crate::borg_spare_gear::teaches_on_sale(it) && it.art_idx == 0 && it.activ_idx == 0 && !it.cursed
        {
            home_role = Role::Renewable;
        }
        if home_role == Role::LatePotion {
            let home = &st.store.shops[BORG_HOME].ware;
            let earlier = if place == Place::Home { &home[..slot] } else { &home[..] };
            if earlier.iter().any(|other| other.iqty > 0 && other.kind == it.kind) {
                home_role = Role::Scarce;
            }
        }
        // #1318: one spare dungeon book is the reserve. Another copy, when
        // Home already holds that book, is ordinary replaceable stock. A
        // single carried copy is the one the Borg reads, not a spare (review
        // of #1323): it keeps the earlier role.
        if home_role == Role::Scarce && borg_town_home::is_book(u32::from(it.tval))
            && place == Place::Pack && it.iqty < 2
        {
            home_role = Role::Ordinary;
        }
        if home_role == Role::Scarce && borg_town_home::is_book(u32::from(it.tval)) {
            let home = &st.store.shops[BORG_HOME].ware;
            let earlier = if place == Place::Home { &home[..slot] } else { &home[..] };
            if earlier.iter().any(|other| other.iqty > 0 && other.kind == it.kind) {
                home_role = Role::Renewable;
            }
        }
        KitItem {
            home_role,
            home_room,
            keep_last_book: protects_class_book(it),
            protect_disposal: protected_supply
                || (place == Place::Home && (retained_at_home
                    || (home_policy && it.art_idx != 0 || ranked_item) && !redundant_dagger))
                || (it.aware && u32::from(it.tval) == crate::headers::list_tvals_h::TV_SCROLL
                    && i32::from(it.sval) == st.item_val.sv_scroll_detect_invis),
            home_unwanted: crate::borg_town_capability::unwanted_home_item(st, it),
            home_limited: crate::borg_town_utility::home_limited_kind(it.tval) && it.aware,
            joins_pack: home_room.is_some_and(|[_, merge, _]| merge >= i32::from(it.iqty)),
            place,
            slot: slot as u8,
            kind: it.kind,
            qty: i32::from(it.iqty),
            weight: i32::from(it.weight),
            need,
            reduction,
            serves: if unknown_flavour { 0 } else { needs_served(st, it).iter().fold(0u8, |m, &n| m | (1 << need_index(n))) },
            power: if consumable { 0 } else { power },
            proceeds,
            sell_shop,
            unknown: unknown_durable(it) || unknown_flavour,
            unknown_flavour,
            just_bought,
            // borg-reborn-rs (#1151)
            cursed_spare: place == Place::Pack && it.cursed,
            fuel: fuel_per_copy(it),
        }
    };

    // The snapshot of everything a try-off can write (as try_on).
    let borg = st.borg.clone();
    let traits = st.traits.clone();
    let trait_swap = st.trait_swap.clone();
    let items = st.items.items.clone();
    let buffer = st.prepared.buffer.clone();
    let do_spell = st.think.do_spell;
    let hole = usize::try_from(crate::borg_inventory::borg_first_empty_inventory_slot(st)).ok();
    let base = i64::from(crate::borg_power::borg_power(st));
    let base_light = i64::from(crate::borg_power::borg_power_equipment(st));
    let light_slot = pack_end + 5;

    let worn_gain = |st: &mut BorgState, it: &borg_item| -> i64 {
        if !home_policy || it.cursed || !it.ident {
            return 0;
        }
        let Some(slot) = borg_town_home::gear_slot(it) else { return 0; };
        let worn = st.items.items[slot];
        if worn.one_ring || engine::flag_has(&worn.flags, crate::headers::list_object_flags_h::OF_STICKY as i32) {
            return 0;
        }
        st.items.items[slot] = *it;
        crate::borg_trait::borg_notice(st, true);
        let gain = i64::from(crate::borg_power::borg_power_equipment(st)) - base_light;
        st.items.items[slot] = worn;
        crate::borg_trait::borg_notice(st, true);
        gain
    };

    let mut kit = Vec::new();
    for (slot, &it) in items.iter().enumerate().take(pack_end) {
        // #1315 (review of #1324): the active weapon or armour swap stays
        // in the pack, where the swap notice looks for it. Left out of the
        // kit, no disposition can store, sell or drop it; its weight and
        // slot still count in the limits, which read the pack itself.
        if it.iqty == 0 || swap_slots.contains(&(slot as i32)) {
            continue;
        }
        if u32::from(it.tval) == crate::headers::list_tvals_h::TV_LIGHT {
            st.items.items[light_slot] = it;
            crate::borg_trait::borg_notice(st, true);
            let gain = i64::from(crate::borg_power::borg_power_equipment(st)) - base_light;
            st.items.items[light_slot] = items[light_slot];
            crate::borg_trait::borg_notice(st, true);
            kit.push(fact(st, &it, Place::Pack, slot, gain.max(0), 0));
            continue;
        }
        if !has_inventory_power(&it) {
            kit.push(fact(st, &it, Place::Pack, slot, 0, 0));
            continue;
        }
        st.items.items[slot].iqty = 0;
        crate::borg_trait::borg_notice(st, true);
        let loss = base - i64::from(crate::borg_power::borg_power(st));
        st.items.items[slot] = it;
        crate::borg_trait::borg_notice(st, true);
        let gain = worn_gain(st, &it);
        kit.push(fact(st, &it, Place::Pack, slot, loss, gain));
    }
    let home: Vec<borg_item> = st.store.shops.get(BORG_HOME).map_or(Vec::new(), |h| h.ware.clone());
    for (slot, it) in home.iter().enumerate() {
        if it.iqty == 0 {
            continue;
        }
        let gain = if u32::from(it.tval) == crate::headers::list_tvals_h::TV_LIGHT {
            st.items.items[light_slot] = *it;
            crate::borg_trait::borg_notice(st, true);
            let gain = i64::from(crate::borg_power::borg_power_equipment(st)) - base_light;
            st.items.items[light_slot] = items[light_slot];
            gain.max(0)
        } else { match hole.filter(|_| has_inventory_power(it)) {
            Some(h) => {
                st.items.items[h] = *it;
                crate::borg_trait::borg_notice(st, true);
                let p = i64::from(crate::borg_power::borg_power(st));
                st.items.items[h] = items[h];
                p - base
            }
            None => 0,
        } };
        crate::borg_trait::borg_notice(st, true);
        let equipped_gain = worn_gain(st, it);
        let gear = home_policy && borg_town_home::gear_slot(it).is_some();
        kit.push(fact(st, it, Place::Home, slot, if gear { equipped_gain } else { gain }, equipped_gain));
    }

    reserve_books(&mut kit);

    st.borg = borg;
    st.traits = traits;
    st.trait_swap = trait_swap;
    st.items.items = items;
    st.prepared.buffer = buffer;
    st.think.do_spell = do_spell;
    kit
}

/// The record number over the run.
static SEQ: AtomicU64 = AtomicU64::new(0);
/// The town visit's number, and the level clock it was counted at.
static VISIT: AtomicU64 = AtomicU64::new(0);
static VISIT_CLOCK: AtomicU64 = AtomicU64::new(u64::MAX);
/// The step's number within the visit.
static STEP: AtomicU64 = AtomicU64::new(0);

/// Shadow the town decision just taken: compute the plan's step on the
/// same state and record it. Does nothing unless `BORG_TOWN_PLAN_RECORD`
/// is set, or outside the town.
pub fn shadow_visit(st: &mut BorgState) {
    if crate::borg_town_dump::plan_record_file().is_some() && st.borg.tr(BI_CDEPTH) == 0 {
        evaluate(st, false);
    }
}

/// The same solve used by recording, now available to the live executor.
/// One purchased/taken/sold copy is executed before the next fresh solve.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TownAction {
    pub code: i64,
    pub at: [i64; 4],
}

pub(crate) struct LivePlan {
    pub action: TownAction,
    pub urgent: crate::borg_town_utility::Room,
    pub urgent_action: bool,
    pub gold: i64,
}

/// #1317: the live planner's kit, for the native town fixtures.
#[cfg(feature = "test-town-loadout")]
pub(crate) fn kit_for_test(st: &mut BorgState) -> Vec<KitItem> {
    kit_of(st, true)
}

pub(crate) fn live_plan(st: &mut BorgState) -> LivePlan {
    evaluate(st, true)
}

/// Refill the torch reserve after immediate needs and before other shopping.
/// Spend the ordinary shopping budget and use the same one-copy action as
/// the light emergency, then replan with the torch actually carried.
/// borg-reborn-rs (#1239): apply the current policy's carry limit.
fn replenish_torch_reserve(st: &BorgState, stock: &Stock, budget: &mut Budget, overloaded_shopping: bool) {
    let reserve = param(Param::town_light_reserve_turns);
    // borg-reborn-rs (#1110): to the planner's goal, scaled while
    // breadth-first exploration applies (review of #1113).
    let goal = crate::borg_light::fuel_goal(st, crate::borg_town_utility::Journey::of(st));
    if reserve == 0 || !budget.urgent.is_empty()
        || crate::borg_light::torch_fuel_turns(st).is_none_or(|turns| i64::from(turns) >= goal)
    {
        return;
    }
    let found = stock.candidates.iter().enumerate().filter(|(_, c)| {
        let shop = usize::from(c.shop);
        if !matches!(shop, BORG_HOME | GENERAL_STORE) || u32::from(c.tval) != TV_LIGHT
            || i32::from(c.sval) != st.item_val.sv_light_torch || c.qty <= 0
            || i64::from(c.price) > budget.gold || c.slots_cost > budget.pack_slots
            || (!overloaded_shopping && i64::from(c.weight) > budget.weight)
        {
            return false;
        }
        let torch = &st.store.shops[shop].ware[usize::from(c.slot)];
        torch.timeout > 0 && !torch.cursed
    }).min_by_key(|(_, c)| (c.price, c.shop, c.slot));
    if let Some((candidate, c)) = found {
        budget.gold -= i64::from(c.price);
        budget.pack_slots -= c.slots_cost;
        budget.weight -= i64::from(c.weight);
        budget.urgent.push(UrgentPurchase { candidate, needs: vec![Need::Light] });
    }
}

fn unusable_torch(unlight: bool, tval: u8, sval: u8, torch: i32) -> bool {
    unlight && u32::from(tval) == crate::headers::list_tvals_h::TV_LIGHT && i32::from(sval) == torch
}

/// Reserve fuel is a turn count. Allow the last torch to round up the
/// shortfall. A zero quantity leaves the candidate index stable for replay.
fn torch_offer_quantity(shortfall: i64, torch: &borg_item, qty: i32) -> i32 {
    if torch.cursed || torch.timeout <= 0 {
        return 0;
    }
    qty.min((shortfall.max(0) as u64).div_ceil(torch.timeout as u64) as i32)
}

/// The copies the live executor moves for a planned action (#507): a
/// purchase keeps the copies the plan picked, every other action moves one
/// copy and the plan is solved again. A planned quantity larger than one
/// (a whole stack, or the surplus past `town_store_keep_min`, #859) is
/// therefore reached one copy at a time.
fn executed_copies(code: i64, planned: i64) -> i64 {
    if matches!(code, crate::borg_town_record::action::BUY | crate::borg_town_record::action::BUY_FOR_HOME) { planned.max(1) } else { 1 }
}

fn evaluate(st: &mut BorgState, live: bool) -> LivePlan {
    let file = crate::borg_town_dump::plan_record_file();
    // borg-reborn-rs (#1239): select the schema and record the departure pass.
    let overloaded_shopping = crate::borg_town_live::shop_overloaded();
    let reserve_on = borg_town_home::enabled(st);
    let revision = match (overloaded_shopping, reserve_on) {
        (true, true) => crate::borg_town_record::PLAN_SCHEMA,
        (true, false) => 21,
        (false, true) => crate::borg_town_record::HOME_BALANCE_SCHEMA,
        (false, false) => 20,
    };
    let departure = live && overloaded_shopping && crate::borg_town_live::departure_preparing(st);

    // A new visit when the level clock moved (new_level restarts it on
    // every arrival, the town included).
    let clock = st.borg.time.level as u64;
    if VISIT_CLOCK.swap(clock, Ordering::Relaxed) != clock {
        VISIT.fetch_add(1, Ordering::Relaxed);
        STEP.store(0, Ordering::Relaxed);
    }
    let visit = VISIT.load(Ordering::Relaxed);
    let step_no = STEP.fetch_add(1, Ordering::Relaxed);

    // A curing spell cures cuts and poison, and no trait counts it.
    let cure_spell = crate::borg_magic::borg_spell_legal_fail(
        st,
        crate::borg_headers::borg_magic_h::HERBAL_CURING,
        80,
    );
    let mut trip = Trip::of(st);
    trip.cure_spell = cure_spell;
    let reads_now = crate::borg_item_enchant::reads_scroll_in_town(st);
    let mut stock = Stock::of(st);
    // The wearables: their power gain, and their slot cost (the worn item
    // moves to the pack).
    let (tryon, wear_slots) = try_on(st, &stock);
    let kit = kit_of(st, live);
    for &(i, cost) in &wear_slots {
        stock.candidates[i].slots_cost = cost;
    }
    // Rejections apply only while the observed carrying/purse context is
    // unchanged. Keep zero-quantity rows to preserve recorded candidate order.
    let unlight = engine::player_has(crate::headers::list_player_flags_h::PF_UNLIGHT);
    let torch = st.item_val.sv_light_torch;
    // borg-reborn-rs (#1273): Necromancer is the only PF_UNLIGHT class in class.txt.
    let light_scroll = st.item_val.sv_scroll_light;
    let unsuitable = |tval: u8, sval: u8| {
        unusable_torch(unlight, tval, sval, torch)
            || (unlight && u32::from(tval) == crate::headers::list_tvals_h::TV_SCROLL
                && i32::from(sval) == light_scroll)
    };
    // The zero mode is the historical shopping policy used by exact replays.
    let torch_shortfall = if live && param(Param::town_shop_overloaded) > 0
        && param(Param::town_torch_purchase_limit) != 0 && param(Param::town_light_reserve_turns) > 0 {
        crate::borg_light::torch_fuel_turns(st).map(|turns| {
            crate::borg_light::fuel_goal(st, crate::borg_town_utility::Journey::of(st)) - i64::from(turns)
        })
    } else {
        None
    };
    let cap_torch = |item: &borg_item, qty| {
        if u32::from(item.tval) == TV_LIGHT && i32::from(item.sval) == torch
            && item.ego_idx == 0 && item.art_idx == 0 {
            torch_shortfall.map_or(qty, |shortfall| torch_offer_quantity(shortfall, item, qty))
        } else {
            qty
        }
    };
    for c in &mut stock.candidates {
        // Unlight supplies sight without a carried torch. Keep this physical
        // suitability gate ahead of fuel, gear try-on and learning bonuses.
        if unsuitable(c.tval, c.sval) { c.qty = 0; }
        c.qty = cap_torch(&st.store.shops[usize::from(c.shop)].ware[usize::from(c.slot)], c.qty);
        if live {
            c.qty = c.qty.min(crate::borg_town_live::acquisition_room(st, c.tval, c.kind));
            let weight = st.store.shops[usize::from(c.shop)].ware[usize::from(c.slot)].weight;
            c.qty = c.qty.min(crate::borg_town_live::shopping_weight_capacity(st, weight));
            if crate::borg_town_live::buy_rejected(st, c.shop as i16, c.slot as usize) { c.qty = 0; }
        }
    }
    let mut class = stock.candidates.iter().map(|c| c.class).max().unwrap_or(0).saturating_add(1);
    for (slot, item) in st.store.shops[BORG_HOME].ware.iter().enumerate() {
        if item.iqty == 0 { continue; }
        // borg-reborn-rs (#1239): reserve saved gear for the loadout pass.
        let available = (!live || crate::borg_town_live::acquisition_room(st, item.tval, item.kind) > 0
                && crate::borg_town_live::shopping_weight_capacity(st, item.weight) > 0)
            && item.aware && !unsuitable(item.tval, item.sval) && !(live && crate::borg_town_live::buy_rejected(st, 7, slot))
            && !(live && (crate::borg_town_live::home_moved(st, item, true)
                || crate::borg_town_live::retained_home_gear(st, slot)));
        let native = engine::store_stock(&st.access, BORG_HOME);
        let joins = native.get(slot).is_some_and(|o| o.merge_capacity() >= 1);
        stock.candidates.push(crate::borg_town_stock::Candidate { shop:7,slot:slot as u8,
            kind:item.kind,tval:item.tval,sval:item.sval,price:0,qty:cap_torch(item, i32::from(available)),weight:i32::from(item.weight),
            slots_cost:i32::from(!joins),stack_count:0,class });
        class = class.saturating_add(1);
    }
    let st: &BorgState = st;
    let mut budget = Budget::of(st, &stock, &trip);
    replenish_torch_reserve(st, &stock, &mut budget, overloaded_shopping);
    let urgent_cost: i64 = budget.urgent.iter().map(|purchase|
        i64::from(stock.candidates[purchase.candidate].price)).sum();
    let gold_after_urgent = i64::from(st.borg.tr(crate::borg_headers::borg_trait_h::BI_GOLD))
        .saturating_sub(urgent_cost);
    let recall_floor = crate::borg_store_buy::recall_purchase_floor();
    if recall_floor > 0 {
        let recall_budget = budget.gold.min(crate::borg_store_buy::recall_purchase_budget(gold_after_urgent, recall_floor));
        for c in &mut stock.candidates {
            if usize::from(c.shop) != BORG_HOME && c.price > 0
                && crate::borg_town_budget::is_recall_scroll(st, c.tval, c.sval)
            {
                c.qty = i64::from(c.qty).min(recall_budget / i64::from(c.price)) as i32;
            }
        }
    }
    let capability = crate::borg_town_capability::observe(st, &stock, &kit,
        crate::borg_town_utility::Journey::of(st), gold_after_urgent);
    let may_buy_for_home = reserve_on && budget.unmet.is_empty()
        && crate::borg_town_live::home_buy_available(st);
    let home_value = |c: &crate::borg_town_stock::Candidate, item: &borg_item| {
        if reserve_on && c.shop < BORG_HOME as u8
            && crate::borg_town_live::home_kind_cleared(st, c.kind)
        {
            return -1;
        }
        if !may_buy_for_home || c.shop >= BORG_HOME as u8 || c.qty <= 0 || !item.flavour_aware
            || item.cursed || c.stack_count > 0 || kit.iter().any(|k| k.kind == c.kind)
            || !engine::store_stock(&st.access, usize::from(c.shop)).get(usize::from(c.slot))
                .is_some_and(|offer| offer.carry_capacity() >= 1)
        {
            return 0;
        }
        let gain = tryon.iter().find(|t| t.shop == c.shop && t.slot == c.slot)
            .map_or(0, |t| t.value);
        let role = borg_town_home::classify(st, item, false, gain);
        if role == Role::LatePotion {
            // Healing is budgeted against the trip separately. Even a potion
            // with no current coverage gain can start a future Home reserve.
            return role.offer_value(capability.home_offer(c.shop, c.slot).max(1));
        }
        if matches!(role, Role::Device | Role::Scarce) {
            return role.offer_value(capability.home_offer(c.shop, c.slot));
        }
        if !c.is_black_market() || !item.ident || !borg_town_home::known_special(item) {
            return 0;
        }
        if !role.gear() {
            return 0;
        }
        let worn = borg_town_home::gear_slot(item).map(|slot| &st.items.items[slot]);
        let weight_saved = worn.map_or(0, |w| (i64::from(w.weight) - i64::from(item.weight)).max(0));
        role.offer_value(gain.max(0) + weight_saved)
    };
    let mask = |ns: &[Need]| ns.iter().fold(0u8, |m, &n| m | (1 << need_index(n)));
    let wares: Vec<WareFact> = stock
        .candidates
        .iter()
        .map(|c| {
            let item = &st.store.shops[usize::from(c.shop)].ware[usize::from(c.slot)];
            let (need, reduction) = ware_role(st, item);
            WareFact {
                home_value: home_value(c, item),
                shop: c.shop,
                slot: c.slot,
                kind: c.kind,
                class: c.class,
                slots_cost: c.slots_cost,
                weight: c.weight,
                need,
                reduction,
                // The knowledge bit marks a learnable ware: an unaware kind
                // ({unseen} in the shop: `flavour_aware`, since `aware` is
                // forced true in a store and an unaware potion or scroll has
                // no unknown runes) or unknown runes ({??}). Buying one
                // makes the flavour aware and learns every rune
                // (store.c, "Learn flavor, any effect and all the runes").
                serves: mask(&needs_served(st, item))
                    | if c.shop != 7 && (item.needs_ident || !item.flavour_aware) { 1 << need_index(Need::Knowledge) } else { 0 },
            }
        })
        .collect();
    let mut params = [0i64; PARAMS];
    for (i, p) in params.iter_mut().enumerate() {
        *p = i64::from(st.cfg(BORG_TOWN_PLAN + i as u32));
    }
    let configured_params = params;
    let params = effective_params(params);
    // The pack and the equipment, as v2 writes them: the light need
    // counts the light sources among them.
    let carried: Vec<(u8, i32)> = st
        .items
        .items
        .iter()
        .take(inven_wield() as usize + 12)
        .filter(|it| it.iqty != 0)
        .map(|it| (it.tval, i32::from(it.iqty)))
        .collect();
    crate::borg_town_capability::audit(st, crate::borg_town_utility::Journey::of(st), &capability);
    let limits = Limits::of(&budget);
    // #957: the storing settings this solve uses, recorded with it (v13).
    let storing = crate::borg_town_utility::Storing::settings();
    let out = step(&StepInput { departure, storing: Some(storing),
        shop_overloaded: overloaded_shopping,
        revision,
        capability: Some(&capability),
        utility: true,
        candidates: stock.candidates.clone(),
        facts: &wares,
        tryon: &tryon,
        limits,
        black_market_reserve: budget.black_market_reserve,
        black_market_percentage: budget.black_market_percentage,
        urgent: budget.urgent.clone(),
        params,
        traits: &st.borg.trait_,
        power: i64::from(st.borg.power),
        carried: &carried,
        cure_spell,
        reads_now,
        kit: &kit,
        home_slots: budget.home_slots,
    });
    let (paction, mut act, for_need, now) = if live {
        // Do not undo this visit's Home moves (borg-rs#90). The recorded
        // step output stays as solved; only the executed choice is barred.
        let mut chosen_from = out.clone();
        for (k, d) in kit.iter().zip(chosen_from.dispositions.iter_mut()) {
            let reverses = match (k.place, *d) {
                (Place::Pack, Disposition::Store) => crate::borg_town_live::home_moved(st, &st.items.items[usize::from(k.slot)], false),
                (Place::Home, Disposition::Take | Disposition::Sell | Disposition::Drop) =>
                    crate::borg_town_recall_loadout::retained_at_home(st, &st.store.shops[BORG_HOME].ware[usize::from(k.slot)])
                    || *d == Disposition::Take && crate::borg_town_live::home_moved(st, &st.store.shops[BORG_HOME].ware[usize::from(k.slot)], true),
                _ => false,
            };
            if reverses { *d = Disposition::Keep; }
        }
        {
            let facts = crate::borg_town_budget::Facts::of(st);
            // #939 re-review: an urgent ware that cannot be carried must not
            // hide an affordable urgent ware that can.
            // borg-reborn-rs (#1239): shop before disposing for weight.
            let pack_slots = facts.pack_capacity - stock.pack_slots_used();
            let acquire = if overloaded_shopping && facts.allowance() < 0 {
                crate::borg_town_record::acquisition_before_disposal(
                    &stock.candidates, &budget.urgent, &chosen_from, &wares, &kit, pack_slots)
            } else { None };
            let weight_room = if overloaded_shopping { i64::MAX } else { facts.allowance() };
            acquire.unwrap_or_else(|| crate::borg_town_record::live_selection(
                &stock.candidates, &budget.urgent, &chosen_from, &wares, &kit,
                budget.home_slots, pack_slots, weight_room, |blocked| {
                    let mut others = stock.clone();
                    for &i in blocked { others.candidates[i].qty = 0; }
                    let mut b = Budget::of(st, &others, &trip);
                    replenish_torch_reserve(st, &others, &mut b, overloaded_shopping);
                    b.urgent
                }))
        }
    } else { first_action(&stock.candidates, &budget.urgent, &out, &wares, &kit) };

    if live && paction != crate::borg_town_record::action::LEAVE && paction != crate::borg_town_record::action::DROP {
        use crate::borg_town_record::action;
        // Use batches only for intentional Home cleanup. The existing
        // one-copy actions still settle and re-solve as before.
        let slot = act[1] as usize;
        let selling = match paction {
            action::SELL => st.items.items.get(slot),
            action::SELL_HOME => st.store.shops[BORG_HOME].ware.get(slot),
            _ => None,
        };
        let replenishing = paction == action::TAKE_HOME && !now
            && kit.iter().enumerate().any(|(i, item)| item.place == Place::Home && usize::from(item.slot) == slot
                && out.copies.get(i).is_some_and(|&qty| qty > 0));
        // borg-reborn-rs (#1239): unload the solver's full surplus quantity.
        let unloading = departure && matches!(paction, action::STORE | action::SELL)
            && kit.iter().enumerate().any(|(i, item)| item.place == Place::Pack && usize::from(item.slot) == slot
                && out.copies.get(i).is_some_and(|&qty| qty > 0));
        let policy_cleanup = paction == action::SELL_HOME && reserve_on;
        if paction != action::DROP_HOME && !replenishing && !policy_cleanup && !unloading
            && !selling.is_some_and(|item| crate::borg_town_live::batch_home_disposal(st, item))
        {
            act[2] = executed_copies(paction, act[2]);
        }
        if selling.is_some() && act[2] > 0 {
            let qty = act[2] as i32;
            act[3] = if paction == action::SELL {
                engine::inven_object(&st.access, slot)
                    .map_or(0, |o| i64::from(o.sale_quote(act[0] as usize, qty)))
            } else {
                engine::store_stock(&st.access, BORG_HOME).get(slot)
                    .map_or(0, |o| i64::from(o.sale_quote(act[0] as usize, qty)))
            };
        }
    }
    let chosen = LivePlan {
        action: TownAction { code: paction, at: act },
        urgent_action: now,
        urgent: crate::borg_town_utility::Room {
            weight: budget.urgent.iter().map(|p| i64::from(stock.candidates[p.candidate].weight)).sum(),
            slots: crate::borg_town_budget::Facts::of(st).pack_capacity - stock.pack_slots_used() - budget.pack_slots,
        },
        gold: budget.gold,
    };
    if file.is_none() { return chosen; }
    let rec = PlanRecord {
        storing: Some(storing.to_array()),
        capability: Some(capability),
        schema: revision,
        legacy_home_21: false,
        mode: if departure { 2 } else { u8::from(live) },
        visit,
        step: step_no,
        gturn: i64::from(engine::game_turn()),
        power: i64::from(st.borg.power),
        spells: u8::from(cure_spell) | u8::from(reads_now) << 1,
        params,
        limits: [budget.gold, i64::from(budget.pack_slots), i64::from(budget.home_slots), budget.weight],
        black_market_reserve: budget.black_market_reserve,
        black_market_percentage: budget.black_market_percentage,
        caps: [
            i64::from(engine::z_info().pack_size),
            i64::from(engine::z_info().store_inven_max),
        ],
        urgent: budget
            .urgent
            .iter()
            .map(|f| {
                let c = &stock.candidates[f.candidate];
                (c.shop, c.slot, mask(&f.needs))
            })
            .collect(),
        unmet: mask(&budget.unmet),
        wares,
        tryon,
        dispose: out.dispositions.iter().map(|&d| crate::borg_town_record::dispose_code(d)).collect(),
        kit,
        plan: out.plan.clone(),
        solve: out.solve,
        action: paction,
        act,
        for_need,
        now,
        displaced: out.displaced,
        guard: 0,
    };

    // v2's fields for this state (today's decision among them), with the
    // version tag 4, then the plan's.
    let mut line = String::with_capacity(4096);
    crate::borg_town_dump::push_integer_record_at(
        &mut line,
        st,
        SEQ.fetch_add(1, Ordering::Relaxed),
        revision,
        i64::from(st.borg.time.now),
        true,
    );
    if line.ends_with('}') {
        line.pop();
    }
    rec.push_fields(&mut line);
    line.push_str(",\"configured_params\":[");
    line.push_str(&configured_params.iter().map(i64::to_string).collect::<Vec<_>>().join(","));
    line.push_str("]}");
    crate::borg_town_dump::write_record(file, &line);
    chosen
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::headers::list_tvals_h::*;

    /// #859 (review F1): what reaches the executor. The live path moves one
    /// copy of a store and solves again; a purchase keeps its planned copies.
    #[test]
    fn the_live_executor_moves_one_copy_of_a_store_and_the_planned_copies_of_a_purchase() {
        use crate::borg_town_record::action;
        for code in [action::STORE, action::SELL, action::SELL_HOME, action::TAKE_HOME] {
            assert_eq!(executed_copies(code, 9), 1, "action {code}: one copy, then solve again");
        }
        assert_eq!(executed_copies(action::BUY, 3), 3);
        assert_eq!(executed_copies(action::BUY, 0), 1);
    }

    #[test]
    fn unaware_devices_are_protected_even_when_no_runes_are_missing() {
        let mut it = crate::borg_item::blank_item();
        it.tval = TV_STAFF as u8;
        it.value = 70;
        assert!(unknown_durable(&it));
        it.aware = true;
        it.ident = true;
        assert!(!unknown_durable(&it));
        it.needs_ident = true;
        assert!(unknown_durable(&it));
        it.tval = TV_POTION as u8;
        assert!(!unknown_durable(&it));
    }

    #[test]
    fn known_worthless_potions_are_not_offered_to_a_merchant() {
        let mut it = crate::borg_item::blank_item();
        it.tval = TV_POTION as u8;
        it.aware = true;
        assert!(!apparently_saleable(&it));
        it.aware = false;
        it.value = 20;
        assert!(apparently_saleable(&it), "an unknown flavour may still be sold to learn it");
    }

    #[test]
    fn consumables_do_not_inherit_inventory_powers_level_gates() {
        for tval in [TV_POTION, TV_SCROLL, TV_FOOD, TV_MUSHROOM, TV_FLASK, TV_WAND, TV_STAFF, TV_ROD, TV_ARROW, TV_MAGIC_BOOK] {
            let mut it = crate::borg_item::blank_item();
            it.tval = tval as u8;
            assert!(!has_inventory_power(&it));
        }
        for tval in [TV_SWORD, TV_HARD_ARMOR, TV_RING] {
            let mut it = crate::borg_item::blank_item();
            it.tval = tval as u8;
            assert!(has_inventory_power(&it));
        }
    }
}

#[cfg(test)]
mod missile_and_light_tests {
    #[test]
    fn unlight_rejects_torches_before_knowledge_or_equipment_scoring() {
        use crate::headers::list_tvals_h::{TV_LIGHT, TV_FLASK};
        assert!(super::unusable_torch(true, TV_LIGHT as u8, 1, 1));
        assert!(!super::unusable_torch(false, TV_LIGHT as u8, 1, 1));
        assert!(!super::unusable_torch(true, TV_LIGHT as u8, 2, 1));
        assert!(!super::unusable_torch(true, TV_FLASK as u8, 1, 1));
    }

    #[test]
    fn torch_offer_is_bounded_by_the_current_fuel_shortfall() {
        let mut torch = crate::borg_item::blank_item();
        torch.timeout = 5000;
        let available = 8;
        // Four full spares cover the Priest's 20,000-turn reserve. The
        // following two visits with 11 and 18 also need no torches.
        for copies in [4, 11, 18] {
            assert_eq!(super::torch_offer_quantity(20_000 - copies * 5000, &torch, available), 0);
        }
        // Four partly used spares at 3,000 turns plus 2,003 worn.
        let carried = 4 * 3000 + 2003;
        let bought = super::torch_offer_quantity(20_000 - carried, &torch, available);
        assert_eq!(bought, 2);
        assert_eq!(super::torch_offer_quantity(20_000 - carried - i64::from(bought) * 5000, &torch, available), 0);
        assert_eq!(super::torch_offer_quantity(40_000 - 20_000, &torch, available), 4);
        torch.cursed = true;
        assert_eq!(super::torch_offer_quantity(20_000, &torch, available), 0);
    }
}

#[cfg(test)]
mod knowledge_only_purchase_tests {
    use super::*;
    use crate::borg_headers::borg_trait_h::BI_MAX;
    use crate::borg_town_solve::Limits;
    use crate::borg_town_stock::Candidate;

    fn configured() -> [i64; PARAMS] {
        let mut p = [0; PARAMS];
        for (i, v) in p.iter_mut().enumerate() {
            *v = i64::from(crate::borg_init::BORG_SETTINGS[BORG_TOWN_PLAN as usize + i].default_value);
        }
        p
    }

    /// The copies planned of one flavour-unknown ware that serves nothing but
    /// knowledge (a scroll of Monster Confusion before it is known, #460).
    fn copies_bought(params: [i64; PARAMS], utility: bool) -> i32 {
        let candidates = vec![Candidate { shop: 4, slot: 10, kind: 216, tval: 0, sval: 0, price: 30, qty: 1,
            weight: 5, slots_cost: 1, stack_count: 0, class: 48 }];
        let wares = [WareFact { home_value: 0, shop: 4, slot: 10, kind: 216, class: 48, slots_cost: 1, weight: 5, need: -1, reduction: 0,
            serves: 1 << need_index(Need::Knowledge) }];
        let mut traits = vec![0; BI_MAX as usize];
        traits[crate::borg_headers::borg_trait_h::BI_CARRY as usize] = 2000;
        let out = step(&StepInput { departure: false, shop_overloaded: false, storing: None,
            revision: crate::borg_town_record::PLAN_SCHEMA, capability: None, utility, black_market_reserve: 0, black_market_percentage: -1, candidates, facts: &wares,
            tryon: &[], limits: Limits { gold: 500, pack_slots: 10, weight: 1000 }, urgent: Vec::new(), params,
            traits: &traits, power: 1000, carried: &[], cure_spell: false, reads_now: false, kit: &[], home_slots: 10 });
        out.plan.iter().map(|e| e.copies).sum()
    }

    #[test]
    fn effective_params_zero_only_the_knowledge_weight() {
        let configured = configured();
        let effective = effective_params(configured);
        let at = (BORG_TOWN_W_KNOWLEDGE - BORG_TOWN_PLAN) as usize;
        assert_eq!(configured[at], 25);
        assert_eq!(effective[at], 0);
        for i in (0..PARAMS).filter(|&i| i != at) { assert_eq!(effective[i], configured[i]); }
    }

    #[test]
    fn a_flavour_unknown_ware_serving_only_knowledge_is_not_bought() {
        for utility in [false, true] {
            assert_eq!(copies_bought(configured(), utility), 1, "the configured weight would buy it (utility {utility})");
            assert_eq!(copies_bought(effective_params(configured()), utility), 0, "utility {utility}");
        }
    }
}

/// Reserve one stack per book kind, preferring a copy already safe at Home.
/// #1318 (review of #1323): a scarce dungeon book's reserve is the stack its
/// role made `Scarce`, so the role and the last-book protection are the same
/// stack and the other Home stacks of that book can be pruned.
pub(crate) fn reserve_books(kit: &mut [KitItem]) {
    let mut reserved = std::collections::BTreeSet::new();
    for stack in kit.iter().filter(|stack| stack.keep_last_book && stack.home_role == Role::Scarce) {
        reserved.insert(stack.kind);
    }
    let mut chosen = std::collections::BTreeSet::new();
    for stack in kit.iter_mut().rev().filter(|stack| stack.keep_last_book) {
        stack.keep_last_book = if reserved.contains(&stack.kind) {
            stack.home_role == Role::Scarce && chosen.insert(stack.kind)
        } else {
            chosen.insert(stack.kind)
        };
    }
}

/// Match the engine's class book list independently of spell level and mana.
pub(crate) fn protects_class_book(item: &borg_item) -> bool {
    param(Param::town_keep_last_class_book) != 0 && item.kind != 0
        && engine::obj_kind_can_browse(item.kind as usize)
}

#[cfg(test)]
mod unaware_device_tests {
    use super::*;
    use crate::headers::list_tvals_h::{TV_RING, TV_ROD, TV_SCROLL, TV_STAFF, TV_WAND};

    fn item(tval: u32, aware: bool) -> borg_item {
        let mut it = crate::borg_item::blank_item();
        it.tval = tval as u8;
        it.iqty = 1;
        it.aware = aware;
        it
    }

    /// #470: an unaware wand, staff or rod is a kind to learn by selling
    /// one; aware devices, rings and scrolls are not devices to learn.
    #[test]
    fn only_unaware_devices_are_sold_to_learn() {
        for tval in [TV_WAND, TV_STAFF, TV_ROD] {
            assert!(unaware_device(&item(tval, false)), "unaware tval {tval}");
            assert!(!unaware_device(&item(tval, true)), "aware tval {tval}");
        }
        assert!(!unaware_device(&item(TV_RING, false)));
        assert!(!unaware_device(&item(TV_SCROLL, false)));
    }
}
