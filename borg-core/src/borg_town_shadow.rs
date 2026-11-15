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

use crate::borg_headers::borg_h::BORG_TOWN_PLAN;
use crate::borg_headers::borg_item_h::borg_item;
use crate::borg_headers::borg_trait_h::BI_CDEPTH;
use crate::borg_item::inven_wield;
use crate::borg_town_budget::Budget;
use crate::borg_store::BORG_HOME;
use crate::borg_town_dispose::{KitItem, Place};
use crate::borg_town_record::{first_action, step, PlanRecord, StepInput, TryOn, WareFact, PARAMS};
use crate::borg_town_solve::Limits;
use crate::borg_town_stock::{needs_served, Stock};
use crate::borg_town_usefulness::{Need, Trip};
use crate::engine;
use crate::state::BorgState;

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
    use crate::headers::list_tvals_h::{TV_DIGGING, TV_LIGHT};
    let hole = crate::borg_inventory::borg_first_empty_inventory_slot(st);
    let Ok(hole) = usize::try_from(hole) else {
        return (Vec::new(), Vec::new());
    };
    let wield = inven_wield() as usize;
    let wearables: Vec<(usize, usize)> = stock
        .candidates
        .iter()
        .enumerate()
        .filter_map(|(i, c)| {
            let w = &st.store.shops[usize::from(c.shop)].ware[usize::from(c.slot)];
            let tval = u32::from(w.tval);
            if tval == TV_DIGGING {
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
        tryon.push(TryOn { shop: c.shop, slot: c.slot, eq_slot: (eq - wield) as u8, value: p - before });
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

fn unknown_durable(it: &borg_item) -> bool {
    use crate::headers::list_tvals_h::{TV_FLASK, TV_FOOD, TV_MUSHROOM, TV_POTION, TV_SCROLL};
    (!it.ident || !it.aware || it.needs_ident)
        && !matches!(u32::from(it.tval), TV_FLASK | TV_FOOD | TV_MUSHROOM | TV_POTION | TV_SCROLL)
}

fn apparently_saleable(it: &borg_item) -> bool {
    // The engine refuses apparently worthless goods. Unknown durable
    // rune exceptions under no-selling are protected, not disposal stock.
    it.value > 0
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
fn kit_of(st: &mut BorgState) -> Vec<KitItem> {
    use crate::headers::list_options_h::OPT_birth_no_selling;
    let no_selling = engine::player_opt(OPT_birth_no_selling);
    let pack_end = inven_wield() as usize;
    let fact = |st: &BorgState, it: &borg_item, place: Place, slot: usize, power: i64| -> KitItem {
        let (mut need, mut reduction) = ware_role(st, it);
        let consumable = crate::borg_item_id::flavour_sale_type(it.tval);
        let unknown_flavour = !it.aware && consumable;
        if unknown_flavour { need = -1; reduction = 0; }
        let sell_shop = (0..7i8).find(|&who| apparently_saleable(it)
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
        let bought = &st.buy;
        let just_bought = (0..usize::try_from(bought.bought_item_num).unwrap_or(0).min(10))
            .any(|i| bought.bought_item_tval[i] == i32::from(it.tval) && bought.bought_item_sval[i] == i32::from(it.sval));
        KitItem {
            joins_pack: if place == Place::Home {
                engine::store_stock(&st.access, BORG_HOME).get(slot)
                    .is_some_and(|o| o.merge_capacity() >= i32::from(it.iqty))
            } else { false },
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

    let mut kit = Vec::new();
    for (slot, &it) in items.iter().enumerate().take(pack_end) {
        if it.iqty == 0 {
            continue;
        }
        if u32::from(it.tval) == crate::headers::list_tvals_h::TV_LIGHT {
            st.items.items[light_slot] = it;
            crate::borg_trait::borg_notice(st, true);
            let gain = i64::from(crate::borg_power::borg_power_equipment(st)) - base_light;
            st.items.items[light_slot] = items[light_slot];
            crate::borg_trait::borg_notice(st, true);
            kit.push(fact(st, &it, Place::Pack, slot, gain.max(0)));
            continue;
        }
        if !has_inventory_power(&it) {
            kit.push(fact(st, &it, Place::Pack, slot, 0));
            continue;
        }
        st.items.items[slot].iqty = 0;
        crate::borg_trait::borg_notice(st, true);
        let loss = base - i64::from(crate::borg_power::borg_power(st));
        st.items.items[slot] = it;
        crate::borg_trait::borg_notice(st, true);
        kit.push(fact(st, &it, Place::Pack, slot, loss));
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
        kit.push(fact(st, it, Place::Home, slot, gain));
    }

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

pub fn live_action(st: &mut BorgState) -> TownAction {
    evaluate(st, true)
}

fn unusable_torch(unlight: bool, tval: u8, sval: u8, torch: i32) -> bool {
    unlight && u32::from(tval) == crate::headers::list_tvals_h::TV_LIGHT && i32::from(sval) == torch
}

fn effective_params(mut params: [i64; PARAMS], forecast: Option<crate::borg_trip::PlannedTrip>) -> [i64; PARAMS] {
    use crate::borg_headers::borg_h::BORG_TOWN_TARGET_DEPTH;
    let index = (BORG_TOWN_TARGET_DEPTH - BORG_TOWN_PLAN) as usize;
    if params[index] == 0 {
        if let Some(forecast) = forecast { params[index] = i64::from(forecast.depth_target.clamp(1, 100)); }
    }
    params
}

fn evaluate(st: &mut BorgState, live: bool) -> TownAction {
    let file = crate::borg_town_dump::plan_record_file();
    let forecast = crate::borg_trip::forecast(st);

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
    if let Some(forecast) = forecast {
        trip.depth_target = forecast.depth_target;
        trip.levels_per_trip = forecast.levels;
    }
    let mut stock = Stock::of(st);
    // The wearables: their power gain, and their slot cost (the worn item
    // moves to the pack).
    let (tryon, wear_slots) = try_on(st, &stock);
    let kit = kit_of(st);
    for &(i, cost) in &wear_slots {
        stock.candidates[i].slots_cost = cost;
    }
    // Rejections apply only while the observed carrying/purse context is
    // unchanged. Keep zero-quantity rows to preserve recorded candidate order.
    let unlight = engine::player_has(crate::headers::list_player_flags_h::PF_UNLIGHT);
    for c in &mut stock.candidates {
        // Unlight supplies sight without a carried torch. Keep this physical
        // suitability gate ahead of fuel, gear try-on and learning bonuses.
        if unusable_torch(unlight, c.tval, c.sval, st.item_val.sv_light_torch) { c.qty = 0; }
        if live && crate::borg_town_live::buy_rejected(st, c.shop as i16, c.slot as usize) { c.qty = 0; }
    }
    let mut class = stock.candidates.iter().map(|c| c.class).max().unwrap_or(0).saturating_add(1);
    for (slot, item) in st.store.shops[BORG_HOME].ware.iter().enumerate() {
        if item.iqty == 0 { continue; }
        let available = item.aware && !unusable_torch(unlight, item.tval, item.sval, st.item_val.sv_light_torch) && !(live && crate::borg_town_live::buy_rejected(st, 7, slot));
        let native = engine::store_stock(&st.access, BORG_HOME);
        let joins = native.get(slot).is_some_and(|o| o.merge_capacity() >= 1);
        stock.candidates.push(crate::borg_town_stock::Candidate { shop:7,slot:slot as u8,
            kind:item.kind,tval:item.tval,sval:item.sval,price:0,qty:i32::from(available),weight:i32::from(item.weight),
            slots_cost:i32::from(!joins),stack_count:0,class });
        class = class.saturating_add(1);
    }
    let st: &BorgState = st;
    let budget = Budget::of(st, &stock, &trip);
    let mask = |ns: &[Need]| ns.iter().fold(0u8, |m, &n| m | (1 << need_index(n)));
    let wares: Vec<WareFact> = stock
        .candidates
        .iter()
        .map(|c| {
            let item = &st.store.shops[usize::from(c.shop)].ware[usize::from(c.slot)];
            let (need, reduction) = ware_role(st, item);
            WareFact {
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
    // Materialize the shared trip target in the effective model inputs;
    // replay reads the identical effective parameters. Keep the configured
    // parameters separately in the log, including any zero (automatic) value.
    params = effective_params(params, forecast);
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
    let mut forecast_state = BorgState::default();
    forecast_state.core.cfg = st.core.cfg.clone();
    forecast_state.borg.trait_ = st.borg.trait_.clone();
    for (i, &value) in params.iter().enumerate() { forecast_state.core.cfg[(BORG_TOWN_PLAN as usize) + i] = value as i32; }
    let capability = crate::borg_town_capability::observe(st, &stock, &kit,
        crate::borg_town_utility::Journey::for_trip(&forecast_state, forecast));
    crate::borg_town_capability::audit(st, crate::borg_town_utility::Journey::for_trip(&forecast_state, forecast), &capability);
    let limits = Limits::of(&budget);
    let out = step(&StepInput {
        revision: crate::borg_town_record::PLAN_SCHEMA,
        capability: Some(&capability),
        utility: true,
        candidates: stock.candidates.clone(),
        facts: &wares,
        tryon: &tryon,
        limits,
        urgent: budget.urgent.clone(),
        params,
        traits: &st.borg.trait_,
        power: i64::from(st.borg.power),
        carried: &carried,
        cure_spell,
        kit: &kit,
        home_slots: budget.home_slots,
    });
    let (paction, mut act, for_need, now) = if live {
        crate::borg_town_record::first_executable_action(&stock.candidates, &budget.urgent, &out, &wares, &kit, budget.home_slots, crate::borg_town_budget::Facts::of(st).pack_capacity - stock.pack_slots_used())
    } else { first_action(&stock.candidates, &budget.urgent, &out, &wares, &kit) };

    if live && paction != crate::borg_town_record::action::LEAVE && paction != crate::borg_town_record::action::DROP {
        if matches!(paction, crate::borg_town_record::action::SELL | crate::borg_town_record::action::SELL_HOME) && act[2] > 0 {
            // Charged devices need native splitting even for the one-copy
            // action; a whole-stack average can differ from the first sale.
            act[3] = if paction == crate::borg_town_record::action::SELL {
                engine::inven_object(&st.access, act[1] as usize)
                    .map_or(0, |o| i64::from(o.sale_quote(act[0] as usize, 1)))
            } else {
                engine::store_stock(&st.access, BORG_HOME).get(act[1] as usize)
                    .map_or(0, |o| i64::from(o.sale_quote(act[0] as usize, 1)))
            };
        }
        act[2] = 1;
    }
    let chosen = TownAction { code: paction, at: act };
    if file.is_none() { return chosen; }
    let rec = PlanRecord {
        capability: Some(capability),
        schema: crate::borg_town_record::PLAN_SCHEMA,
        mode: u8::from(live),
        visit,
        step: step_no,
        gturn: i64::from(engine::game_turn()),
        power: i64::from(st.borg.power),
        spells: u8::from(cure_spell),
        params,
        limits: [budget.gold, i64::from(budget.pack_slots), i64::from(budget.home_slots), budget.weight],
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
        crate::borg_town_record::PLAN_SCHEMA,
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
    fn automatic_forecast_uses_trip_target_but_explicit_target_is_preserved() {
        use crate::borg_headers::borg_h::BORG_TOWN_TARGET_DEPTH;
        let index = (BORG_TOWN_TARGET_DEPTH - BORG_TOWN_PLAN) as usize;
        let automatic = [0; PARAMS];
        let forecast = crate::borg_trip::PlannedTrip { depth_target: 18, levels: 3, recall_scrolls: 2, recall_carry: 3 };
        assert_eq!(effective_params(automatic, Some(forecast))[index], 18);
        assert_eq!(effective_params(automatic, None), automatic);
        assert_eq!(automatic[index], 0, "configured inputs must remain intact for the record");
        let mut explicit = automatic;
        explicit[index] = 7;
        assert_eq!(effective_params(explicit, Some(forecast)), explicit);
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
}
