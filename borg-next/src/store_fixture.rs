//! R437 copied-memory controls. Queued commands are never consumed by the engine.
use crate::shadow::{Mode, DIAGNOSTIC_SCHEMA};
use borg_rs::decision_api::{
    borg_cave::{BorgGrid, AUTO_MAX_Y},
    borg_headers::borg_h::{
        BORG_MONEY_SCUM_AMOUNT, BORG_SELF_SCUM, BORG_TOWN_NO_STORES, BORG_TOWN_PLAN,
    },
    borg_io::{ESCAPE, KC_ENTER},
    borg_store::{BORG_HOME, SHOP_MENU_ITEMS},
    engine, equipment_fixture,
    headers::{
        list_terrain_h::{FEAT_FLOOR, FEAT_GRANITE},
        list_tvals_h::*,
    },
};
use borg_rs::decisions::{Decision, ItemSnapshot, ReadDecision, Value};
use borg_rs::memory::{traits::*, BorgState};
use borg_rs::shadow::Shadow;
use borg_rs::threat_facts::{
    item_use_fixture::{blank_item, queued_keys},
    param, store_fixture as facts, Param,
};
use std::{cell::Cell, fs::File, io::Write, sync::Once};

pub(crate) fn run(live: &BorgState, mode: Mode, provider: &impl Shadow) {
    // Wait for the real startup's engine-derived kind metadata; absence still fails CI coverage.
    if live.item_val.kv_wand_magic_missile <= 0 {
        return;
    }
    static ONCE: Once = Once::new();
    ONCE.call_once(|| {
        borg_rs::decision_api::item_use_fixture::with_stubbed_messages(|| {
            assert_ne!(mode, Mode::Off);
            let mut out = File::create(
                std::path::PathBuf::from(std::env::var_os("BORG_RUN_DIR").unwrap())
                    .join("r4-store-fixtures.jsonl"),
            )
            .unwrap();
            let mut base = live.clone();
            // Pristine parity borg.txt grants this legacy permission; construct a no-scumming copied state.
            // Do not modify the live game or its declared parity configuration.
            base.core.cfg[BORG_SELF_SCUM as usize] = 0;
            assert_eq!(base.cfg(BORG_SELF_SCUM), 0);
            assert_eq!(base.cfg(BORG_MONEY_SCUM_AMOUNT), 0);
            assert_eq!(base.cfg(BORG_TOWN_PLAN), 0);
            for item in &mut base.items.items {
                *item = blank_item();
            }
            for shop in &mut base.store.shops {
                for item in &mut shop.ware {
                    *item = blank_item();
                }
            }
            base.items.safe_items = base.items.items.clone();
            base.store.safe_shops = base.store.shops.clone();
            base.buy = Default::default();
            base.borg.goal.shop = -1;
            base.borg.goal.item = -1;
            base.borg.goal.ware = -1;
            base.borg.goal.do_best = false;
            base.borg.shop_override = None;
            base.borg.time.level = base.borg.time.now;
            base.borg.antibounce_count = 0;
            base.borg.status.cancel = false;
            base.think.shop_num = -1;
            base.borg.trait_[BI_CDEPTH as usize] = 0;
            base.borg.trait_[BI_QUIVER_SLOTS as usize] = 0;
            base.trait_swap.weapon_swap = 0;
            base.trait_swap.armour_swap = 0;
            let sim = borg_rs::decision_api::engine::z_info().store_inven_max as usize;
            assert!(sim >= 8);
            base.store_sell.test_item.resize(sim, 0);
            base.store_sell.best_item.resize(sim, 0);
            assert!(!borg_rs::decision_api::borg_buy_strategy::reserve410_unmet(
                &base
            ));
            assert!(!borg_rs::decision_api::borg_buy_strategy::buy_identify_enabled(&base));
            for (label, request, expected) in [
                (
                    "empty-shop-buy-selection",
                    Decision::StoreBuyUseful,
                    Value::Acted(false),
                ),
                (
                    "empty-home-withdrawal",
                    Decision::HomeBuyUseful,
                    Value::Acted(false),
                ),
                (
                    "empty-shop-home-stock",
                    Decision::StoreGrabInteresting,
                    Value::Acted(false),
                ),
                (
                    "empty-home-disposal",
                    Decision::HomeGrabUseless,
                    Value::Acted(false),
                ),
                (
                    "empty-home-weapon",
                    Decision::HomeBuySwapWeapon,
                    Value::Acted(false),
                ),
                (
                    "empty-home-armour",
                    Decision::HomeBuySwapArmour,
                    Value::Acted(false),
                ),
                (
                    "no-selected-purchase",
                    Decision::StoreBuy,
                    Value::Acted(false),
                ),
                (
                    "no-selected-withdrawal-for-sale",
                    Decision::StoreTakeForSale,
                    Value::Acted(false),
                ),
                (
                    "empty-pack-sell-selection",
                    Decision::StoreSellUseless,
                    Value::Acted(false),
                ),
                ("no-selected-sale", Decision::StoreSell, Value::Acted(false)),
                (
                    "no-selected-single-sale",
                    Decision::StoreSellQuantity { quantity: 1 },
                    Value::Acted(false),
                ),
                (
                    "empty-pack-sale-count",
                    Decision::StoreCountSell,
                    Value::Score(0),
                ),
                (
                    "no-stores-chooser",
                    Decision::StoreChooseShop,
                    Value::Acted(false),
                ),
                (
                    "store-user-cancel",
                    Decision::StoreThink,
                    Value::Acted(true),
                ),
            ] {
                let mut state = base.clone();
                if matches!(&request, Decision::StoreChooseShop) {
                    state.core.cfg[BORG_TOWN_NO_STORES as usize] = 1;
                }
                if matches!(&request, Decision::StoreThink) {
                    state.borg.status.cancel = true;
                }
                case(
                    &mut out,
                    &state,
                    mode,
                    label,
                    request,
                    Some(expected),
                    Some(vec![]),
                    provider,
                );
            }
            // Full valuations are parity controls; the independent delta below tests an actual weight.
            case(
                &mut out,
                &base,
                mode,
                "home-valuation-parity",
                Decision::HomePower,
                None,
                Some(vec![]),
                provider,
            );
            let mut home = base.clone();
            home.home = Default::default();
            let before = provider.read_decision(&home, ReadDecision::HomePower);
            home.home.num_free_act = 1;
            let after = provider.read_decision(&home, ReadDecision::HomePower);
            let (Value::Score(before), Value::Score(after)) = (before, after) else {
                unreachable!()
            };
            let delta = after.wrapping_sub(before);
            let weight = param(Param::home_free_act_one_weight);
            assert_eq!(delta, weight);
            record(
                &mut out,
                mode,
                "home-free-action-positive-delta",
                serde_json::json!(delta),
                Some(serde_json::json!(weight)),
            );
            let mut quantity = base.clone();
            quantity.borg.trait_[BI_GOLD as usize] = param(Param::sell_batch_min_gold);
            let mut ammo = blank_item();
            ammo.aware = true;
            ammo.tval = TV_ARROW as u8;
            ammo.value = param(Param::sell_batch_max_item_value);
            let batch = param(Param::sell_ammo_batch_count);
            assert!(batch > 1 && batch <= 5);
            ammo.iqty = (batch + 1) as u8;
            for (label, item, expected) in [
                ("ammo-value-boundary-batch", ammo, batch),
                (
                    "ammo-short-stack",
                    {
                        let mut i = ammo;
                        i.iqty = 1;
                        i
                    },
                    1,
                ),
                (
                    "unknown-item-not-batched",
                    {
                        let mut i = ammo;
                        i.aware = false;
                        i
                    },
                    1,
                ),
                (
                    "expensive-item-not-batched",
                    {
                        let mut i = ammo;
                        i.value += 1;
                        i
                    },
                    1,
                ),
            ] {
                case(
                    &mut out,
                    &quantity,
                    mode,
                    label,
                    Decision::StoreMinimumQuantity {
                        item: ItemSnapshot::new(item),
                    },
                    Some(Value::Score(expected)),
                    Some(vec![]),
                    provider,
                );
            }
            let snapshot = ItemSnapshot::new(ammo);
            ammo.iqty = 0;
            ammo.value += 1;
            case(
                &mut out,
                &quantity,
                mode,
                "owned-item-input-survives-caller-change",
                Decision::StoreMinimumQuantity { item: snapshot },
                Some(Value::Score(batch)),
                Some(vec![]),
                provider,
            );
            assert_eq!(ammo.iqty, 0);
            ammo.iqty = (batch + 1) as u8;
            ammo.value = param(Param::sell_batch_max_item_value);
            quantity.borg.trait_[BI_GOLD as usize] = param(Param::sell_batch_min_gold) - 1;
            case(
                &mut out,
                &quantity,
                mode,
                "low-gold-not-batched",
                Decision::StoreMinimumQuantity {
                    item: ItemSnapshot::new(ammo),
                },
                Some(Value::Score(1)),
                Some(vec![]),
                provider,
            );
            for (label, disabled) in [("no-stores-false", false), ("no-stores-true", true)] {
                let mut state = base.clone();
                state.core.cfg[BORG_TOWN_NO_STORES as usize] = i32::from(disabled);
                let before = state.clone();
                let value = provider.read_decision(&state, ReadDecision::StoreDisabled);
                assert_eq!(value, Value::Acted(disabled));
                assert!(state == before);
                record(
                    &mut out,
                    mode,
                    label,
                    serde_json::to_value(value).unwrap(),
                    Some(serde_json::to_value(Value::Acted(disabled)).unwrap()),
                );
            }
            let mut wand = blank_item();
            wand.kind = base.item_val.kv_wand_magic_missile as u32;
            wand.tval = TV_WAND as u8;
            wand.sval = base.item_val.sv_wand_magic_missile as u8;
            wand.iqty = 2;
            wand.pval = 7;
            wand.aware = true;
            wand.ident = true;
            wand.value = 1;
            wand.cost = 1;
            let cap = borg_rs::decision_api::engine::kind_max_stack(wand.kind as usize);
            assert!((4..=255).contains(&cap));
            for (label, a, b, wanted) in [
                (
                    "wand-pooled-charge-stack",
                    wand,
                    {
                        let mut i = wand;
                        i.pval = 3;
                        i
                    },
                    true,
                ),
                (
                    "native-stack-boundary-rejected",
                    {
                        let mut i = wand;
                        i.iqty = (cap - 1) as u8;
                        i
                    },
                    wand,
                    false,
                ),
                (
                    "incompatible-flags-rejected",
                    wand,
                    {
                        let mut i = wand;
                        i.flags[0] = 1;
                        i
                    },
                    false,
                ),
            ] {
                let native = |st: &BorgState| facts::similar(st, &a, &b);
                let ported =
                    |st: &BorgState| crate::stateful::store_sell::borg_object_similar(st, &a, &b);
                let diagnostic = context(mode, label, true);
                let (value, replayed, agreement, _, _) = match mode {
                    Mode::Check => {
                        borg_rs::decision_tape::compare_read(&base, diagnostic, native, ported)
                    }
                    Mode::Switch => {
                        borg_rs::decision_tape::compare_read(&base, diagnostic, ported, native)
                    }
                    Mode::Off => unreachable!(),
                };
                assert!(agreement.is_ok());
                assert_eq!(replayed, Some(value));
                assert_eq!(value, wanted);
                record(
                    &mut out,
                    mode,
                    label,
                    serde_json::json!(value),
                    Some(serde_json::json!(wanted)),
                );
            }
            let mut confirm = base.clone();
            confirm.core.cfg[BORG_TOWN_PLAN as usize] = 1;
            let request = Decision::StoreConfirmPurchase {
                item: ItemSnapshot::new(wand),
                shop: 5,
                keep: true,
                qty: 2,
                paid: 40,
            };
            wand.pval = 19; // The request owns the original charge count.
            let state = case(
                &mut out,
                &confirm,
                mode,
                "confirmed-keeper-history",
                request,
                Some(Value::Unit),
                Some(vec![]),
                provider,
            );
            assert_eq!(state.buy.bought_item_nxt, 1);
            assert_eq!(state.buy.bought_item_num, 1);
            assert_eq!(state.buy.bought_item_pval[0], 7);
            assert_eq!(state.buy.bought_item_store[0], 5);
            assert_eq!(state.buy.bought_item_tval[0], TV_WAND as i32);
            wand.pval = 7;
            let purchase_keys = vec![
                u32::from(SHOP_MENU_ITEMS[0]),
                u32::from(b'p'),
                KC_ENTER,
                KC_ENTER,
                ESCAPE,
                ESCAPE,
                ESCAPE,
            ];
            assert!(!borg_rs::decision_api::engine::player_opt(
                borg_rs::decision_api::headers::list_options_h::OPT_store_buy_direct
            ));
            for (label, request, shop, keeper) in [
                ("queue-selected-purchase", Decision::StoreBuy, 5, true),
                (
                    "queue-home-withdrawal-for-sale",
                    Decision::StoreTakeForSale,
                    BORG_HOME as i16,
                    false,
                ),
            ] {
                let mut state = base.clone();
                state.store.shops[shop as usize].ware[0] = wand;
                state.store.safe_shops = state.store.shops.clone();
                state.think.shop_num = shop;
                state.borg.goal.shop = shop;
                state.borg.goal.ware = 0;
                let after = case(
                    &mut out,
                    &state,
                    mode,
                    label,
                    request,
                    Some(Value::Acted(true)),
                    Some(purchase_keys.clone()),
                    provider,
                );
                assert!(after.items == state.items && after.store == state.store);
                assert_eq!(after.buy.bought_item_nxt, usize::from(keeper) as i32);
                assert_eq!(after.borg.goal.shop, -1);
                assert_eq!(after.borg.goal.ware, -1);
            }
            // Shop and home trials retain their native charge arithmetic, including their differences.
            for (label, request, shop, score, observed) in [
                (
                    "shop-device-trial-and-restore",
                    Decision::StoreBuyUseful,
                    5,
                    i32::MIN,
                    (1, 9, 0, 0, 4, 9),
                ),
                (
                    "home-device-split-and-restore",
                    Decision::HomeBuyUseful,
                    BORG_HOME,
                    0,
                    (1, 1, 4, 8, 0, 0),
                ),
            ] {
                let mut state = base.clone();
                let mut stock = wand;
                stock.iqty = 5;
                stock.pval = 9;
                state.store.shops[shop].ware[0] = stock;
                state.store.safe_shops = state.store.shops.clone();
                state.borg.power = score;
                for (index, value) in [
                    (BI_LIGHT, 2),
                    (BI_FOOD, 10),
                    (BI_GOLD, 1000),
                    (BI_MAXCLEVEL, 1),
                    (BI_GOOD_W_CHG, 0),
                ] {
                    state.borg.trait_[index as usize] = value;
                }
                borg_rs::decision_api::store_fixture::start_notices();
                let mut after = state.clone();
                let _value = provider.decision(&mut after, request);
                let snapshots = borg_rs::decision_api::store_fixture::finish_notices();
                assert!(
                    snapshots.contains(&observed),
                    "{label}: charge/quantity trial was not observed: {snapshots:?}"
                );
                assert!(after.store == state.store);
                assert!(after.items.safe_items == state.items.safe_items);
                assert_eq!(after.items.items[0].iqty, 0);
                record(
                    &mut out,
                    mode,
                    label,
                    serde_json::json!(true),
                    Some(serde_json::json!(true)),
                );
            }
            let mut full = base.clone();
            let last = borg_rs::decision_api::borg_item::inven_wield() as usize - 1;
            for item in &mut full.items.items[..=last] {
                *item = wand;
                item.iqty = 1;
            }
            full.store.shops[BORG_HOME].ware[sim - 1].iqty = 1;
            full.store_sell.test_item.fill(7);
            full.store_sell.best_item.fill(7);
            let after = case(
                &mut out,
                &full,
                mode,
                "full-pack-home-output-preserved",
                Decision::HomeSellUseful {
                    best_home_power: -321,
                },
                Some(Value::HomeSale {
                    acted: false,
                    best_home_power: -321,
                }),
                Some(vec![]),
                provider,
            );
            assert!(after.store_sell.test_item[..8].iter().all(|&v| v == 0));
            assert!(after.store_sell.test_item[8..].iter().all(|&v| v == 7));
            assert!(after.store_sell.best_item[..8].iter().all(|&v| v == 0));
            assert!(after.store_sell.best_item[8..].iter().all(|&v| v == 7));
            let rejected = case(
                &mut out,
                &full,
                mode,
                "full-pack-rejects-purchase-selection",
                Decision::StoreBuyUseful,
                Some(Value::Acted(false)),
                Some(vec![]),
                provider,
            );
            assert!(rejected.items == full.items && rejected.store == full.store);
            let mut empty = base.clone();
            let value = provider.decision(
                &mut empty,
                Decision::HomeSellUseful {
                    best_home_power: i32::MIN,
                },
            );
            let Value::HomeSale {
                acted,
                best_home_power,
            } = value
            else {
                unreachable!()
            };
            assert!(!acted);
            assert!(best_home_power >= -1);
            record(
                &mut out,
                mode,
                "home-sale-output-updated",
                serde_json::to_value(value).unwrap(),
                None,
            );
            // This invokes the real public facade through an opaque callback; replay must not invoke it twice.
            let calls = Cell::new(0);
            let mut callback_state = base.clone();
            let callback = |s: &mut BorgState| {
                borg_rs::decision_tape::effect(
                    s,
                    "store-fixture::home-sale-public",
                    format!("{:?}", ()),
                    |s| {
                        calls.set(calls.get() + 1);
                        let mut power = i32::MIN;
                        let acted =
                            borg_rs::decision_api::borg_store_sell::borg_think_home_sell_useful(
                                s, &mut power,
                            );
                        Value::HomeSale {
                            acted,
                            best_home_power: power,
                        }
                    },
                )
            };
            let (value, replayed, agreement, _, _) = borg_rs::decision_tape::compare(
                &mut callback_state,
                context(mode, "public-home-sale-callback-once", false),
                callback,
                callback,
            );
            assert!(agreement.is_ok());
            assert_eq!(replayed, Some(value));
            assert_eq!(calls.get(), 1);
            record(
                &mut out,
                mode,
                "public-home-sale-callback-once",
                serde_json::json!(calls.get()),
                Some(serde_json::json!(1)),
            );
            let mut fault = quantity.clone();
            let item = ItemSnapshot::new(wand);
            let native = |st: &BorgState| {
                ReadDecision::StoreMinimumQuantity { item: item.clone() }.native(st)
            };
            let faulty = |st: &BorgState| {
                let _ = crate::stateful::store_sell::borg_min_item_quantity(st, &item.0);
                Value::Score(-12345)
            };
            let diagnostic = context(mode, "reject-return-fault", true);
            let (value, replayed, agreement, _, _) = match mode {
                Mode::Check => {
                    borg_rs::decision_tape::compare_read(&fault, diagnostic, native, faulty)
                }
                Mode::Switch => {
                    borg_rs::decision_tape::compare_read(&fault, diagnostic, faulty, native)
                }
                Mode::Off => unreachable!(),
            };
            let wanted = if mode == Mode::Check {
                (Value::Score(1), Some(Value::Score(-12345)))
            } else {
                (Value::Score(-12345), Some(Value::Score(1)))
            };
            assert_eq!((value, replayed), wanted);
            assert_eq!(agreement, Err("read-only decision return differs".into()));
            record(
                &mut out,
                mode,
                "reject-return-fault",
                serde_json::json!(agreement.is_err()),
                Some(serde_json::json!(true)),
            );
            fault.borg.trait_[BI_GOLD as usize] = 0;
            case(
                &mut out,
                &fault,
                mode,
                "repaired-quantity",
                Decision::StoreMinimumQuantity { item },
                Some(Value::Score(1)),
                Some(vec![]),
                provider,
            );
            let mut planned = base.clone();
            planned.core.cfg[BORG_TOWN_PLAN as usize] = 1;
            planned.core.cfg[BORG_TOWN_NO_STORES as usize] = 0;
            case(
                &mut out,
                &planned,
                mode,
                "configured-shared-live-planner",
                Decision::StoreChooseShop,
                None,
                None,
                provider,
            );
            planned.core.cfg[BORG_TOWN_NO_STORES as usize] = 1;
            case(
                &mut out,
                &planned,
                mode,
                "no-stores-shared-live-gate",
                Decision::StoreChooseShop,
                None,
                None,
                provider,
            );
            out.flush().unwrap();
            eprintln!("R4 store constructed fixture complete");
            receiving_controls(&base, mode, provider);
        })
    });
}

fn receiving_controls(base: &BorgState, mode: Mode, provider: &impl Shadow) {
    let path = std::path::PathBuf::from(std::env::var_os("BORG_RUN_DIR").unwrap())
        .join("r45-constructed.jsonl");
    let mut out = File::create(path).unwrap();
    let mut record = |case, value, queued: &[u32], rng_draws: Option<u64>| {
        let row = serde_json::json!({
            "schema": "borg-reborn-rs/r45-constructed@1",
            "case": case,
            "mode": mode,
            "synthetic": true,
            "engine_consumption": false,
            "value": value,
            "queued": queued,
            "rng_draws": rng_draws,
        });
        writeln!(out, "{row}").unwrap();
    };

    let mut state = base.clone();
    state.borg.trait_[BI_ATELEPORT as usize] = 0;
    state.borg.trait_[BI_AESCAPE as usize] = 0;
    let before = queued_keys(&state);
    let value = provider.decision(
        &mut state,
        Decision::CautionTeleport {
            emergency: 0,
            turns: 1,
        },
    );
    assert_eq!(value, Value::Acted(false));
    assert_eq!(queued_keys(&state), before);
    record("teleport-missing-escape-supply", value, &[], None);

    // The pinned 66-by-22 town gives 89 floor landings and 11 exhausted
    // searches at seed 1, origin (1, 1), and the default 100 x 100 budget.
    // Make the same sampled floors dangerous without changing the search.
    let mut landing = base.clone();
    landing.borg.c.x = 1;
    landing.borg.c.y = 1;
    landing.borg.antibounce_count = 0;
    landing.borg.trait_[BI_ATELEPORT as usize] = 1;
    landing.borg.trait_[BI_AESCAPE as usize] = 1;
    landing.borg.trait_[BI_CURHP as usize] = 50;
    landing.borg.trait_[BI_MAXHP as usize] = 50;
    landing.borg.breeder_block.fill(false);
    landing.kills.nxt = 1;
    landing.danger = Default::default();
    assert_eq!(param(Param::escape_landing_samples), 100);
    assert_eq!(param(Param::escape_landing_trials), 100);
    let town = engine::z_info();
    assert_eq!((town.town_wid, town.town_hgt), (66, 22));
    let wall = BorgGrid {
        feat: FEAT_GRANITE as u8,
        ..Default::default()
    };
    let floor = BorgGrid {
        feat: FEAT_FLOOR as u8,
        ..Default::default()
    };
    for y in 0..AUTO_MAX_Y {
        let row = landing.cave.row_mut(y);
        row.fill(wall);
        if (1..i32::from(town.town_hgt) - 1).contains(&y) {
            row[1..usize::from(town.town_wid) - 1].fill(floor);
        }
    }
    for (case, fear, expected) in [
        ("teleport-accepts-sampled-landings", 0, true),
        ("teleport-rejects-dangerous-landings", 100, false),
    ] {
        landing
            .danger
            .fear_monsters
            .iter_mut()
            .for_each(|row| row.fill(fear));
        let before = queued_keys(&landing);
        let saved_rng = engine::rand_state();
        engine::set_rand_state(true, 1);
        let initial_draws = equipment_fixture::rng_draws();
        let value = provider.decision(
            &mut landing,
            Decision::CautionTeleport {
                emergency: 11,
                turns: 1,
            },
        );
        let draws = equipment_fixture::rng_draws() - initial_draws;
        let rng = engine::rand_state();
        engine::set_rand_state(saved_rng.0, saved_rng.1);
        assert_eq!(value, Value::Acted(expected), "{case}");
        assert_eq!(draws, 16_698, "{case}");
        assert_eq!(rng, (true, 1_234_900_399), "{case}");
        assert_eq!(queued_keys(&landing), before, "{case}");
        record(case, value, &[], Some(draws));
    }

    let mut state = base.clone();
    state.borg.trait_[BI_LIGHT as usize] = 0;
    state.borg.trait_[BI_ISPOISONED as usize] = 1;
    let potion = &mut state.items.items[0];
    potion.tval = TV_POTION as u8;
    potion.sval = state.item_val.sv_potion_cure_poison as u8;
    potion.iqty = 1;
    potion.aware = true;
    potion.ident = true;

    let before = queued_keys(&state);
    let value = provider.decision(&mut state, Decision::LightlessSelfCare { danger: 0 });
    assert_eq!(value, Value::Acted(true));
    let after = queued_keys(&state);
    assert!(after.starts_with(&before));
    assert_eq!(&after[before.len()..], &[u32::from(b'q'), u32::from(b'a')]);
    assert_eq!(state.items.items[0].iqty, 1);
    record(
        "lightless-cures-poison",
        value,
        &after[before.len()..],
        None,
    );
}

#[allow(clippy::too_many_arguments)]
fn case(
    out: &mut File,
    base: &BorgState,
    mode: Mode,
    label: &str,
    request: Decision,
    expected: Option<Value>,
    commands: Option<Vec<u32>>,
    provider: &impl Shadow,
) -> BorgState {
    let mut state = base.clone();
    let before = queued_keys(&state);
    let value = provider.decision(&mut state, request);
    if let Some(wanted) = expected {
        assert_eq!(value, wanted, "{label}");
    }
    let after = queued_keys(&state);
    assert!(after.starts_with(&before));
    if let Some(commands) = commands {
        assert_eq!(
            &after[before.len()..],
            commands.as_slice(),
            "{label}: command order"
        );
    }
    record(
        out,
        mode,
        label,
        serde_json::to_value(value).unwrap(),
        expected.map(|v| serde_json::to_value(v).unwrap()),
    );
    state
}
fn record(
    out: &mut File,
    mode: Mode,
    label: &str,
    value: serde_json::Value,
    expected: Option<serde_json::Value>,
) {
    let row = serde_json::json!({"schema":"borg-reborn-rs/r4-store-constructed@1","case":label,"mode":mode,
        "synthetic":true,"engine_consumption":false,"value":value,"expected":expected,
        "control_kind":if expected.is_some(){"value-and-state-assertion"}else{"parity-and-restoration"},
        "stubbed_dependencies":["shared note engine UI callbacks only; real anti-loop state effects retained"],
        "shared_planner_independently_owned":false});
    serde_json::to_writer(&mut *out, &row).unwrap();
    out.write_all(b"\n").unwrap();
    out.flush().unwrap();
}

fn context(mode: Mode, label: &str, readonly: bool) -> serde_json::Value {
    serde_json::json!({"schema":DIAGNOSTIC_SCHEMA,"mode":mode,"point":format!("fixture.store.{label}"),"readonly":readonly,"synthetic":true})
}
