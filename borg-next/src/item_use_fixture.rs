//! Constructed policy-memory cases. Commands are queued on copies; no test
//! command reaches the game. The shared UI note effect is explicitly stubbed;
//! real UI/root callbacks require normal-game validation. No consumption/survival claim.
use crate::shadow::Mode;
use borg_rs::decision_api::headers::list_tvals_h::{
    TV_DRAG_ARMOR, TV_FOOD, TV_POTION, TV_RING, TV_ROD, TV_SCROLL, TV_STAFF, TV_WAND,
};
use borg_rs::decisions::{Decision, Value};
use borg_rs::memory::{traits::*, BorgState};
use borg_rs::threat_facts::item_use_fixture::{blank_item, queued_keys};
use std::{fs::File, io::Write, sync::Once};

pub(crate) fn run(
    live: &BorgState,
    mode: Mode,
    mut dispatch: impl FnMut(&mut BorgState, Decision) -> Value,
) {
    static ONCE: Once = Once::new();
    ONCE.call_once(|| {
        borg_rs::decision_api::item_use_fixture::with_stubbed_messages(|| {
            assert_ne!(mode, Mode::Off, "constructed fixture requires check/switch");
            let mut base = live.clone();
            for item in &mut base.items.items {
                item.iqty = 0;
            }
            base.borg.trait_.fill(0);
            for (index, value) in [
                (BI_CURHP, 100),
                (BI_MAXHP, 100),
                (BI_DEV, 200),
                (BI_CLEVEL, 1),
                (BI_CDEPTH, 0),
            ] {
                base.borg.trait_[index as usize] = value;
            }
            base.borg.activation.fill(0);
            base.borg.book_idx.fill(-1);
            base.magic.magics.clear();
            base.item_use.when_last_quaff = 0;
            let activation = base.act.mapping;
            assert!(activation > 0 && (activation as usize) < base.borg.activation.len());
            let pack = usize::from(borg_rs::threat_facts::z_info().pack_size);
            let mut out = File::create(
                std::path::PathBuf::from(std::env::var_os("BORG_RUN_DIR").unwrap())
                    .join("r4-item-use-fixtures.jsonl"),
            )
            .unwrap();
            let absent = -1;
            let empty = [
                Decision::ItemQuaffCrit { no_check: true },
                Decision::ItemQuaffPotion { sval: absent },
                Decision::ItemReadScroll { sval: absent },
                Decision::ItemEat {
                    tval: TV_FOOD as i32,
                    sval: absent,
                },
                Decision::ItemEatFoodAny,
                Decision::ItemEquipsRod { sval: absent },
                Decision::ItemZapRod { sval: absent },
                Decision::ItemUseStaff { sval: absent },
                Decision::ItemUseStaffFail { sval: absent },
                Decision::ItemEquipsStaffFail { sval: absent },
                Decision::ItemAimWand { sval: absent },
                Decision::ItemEquipsRing { ring_sval: absent },
                Decision::ItemActivateRing { ring_sval: absent },
                Decision::ItemEquipsDragon { drag_sval: absent },
                Decision::ItemActivateDragon { drag_sval: absent },
                Decision::ItemActivateItem { activation },
                Decision::ItemEquipsItem {
                    activation,
                    check_charge: true,
                },
                Decision::ItemActivateFailure {
                    tval: TV_RING as i32,
                    sval: absent,
                },
                Decision::ItemUseThings,
                Decision::ItemRecharging,
            ];
            let mut rows = 0;
            for request in empty {
                let expected = if matches!(request, Decision::ItemActivateFailure { .. }) {
                    Value::Score(100)
                } else {
                    Value::Acted(false)
                };
                check(
                    &mut out,
                    &base,
                    mode,
                    &format!("empty-{}", request.name()),
                    request,
                    expected,
                    false,
                    &mut dispatch,
                );
                rows += 1;
            }
            let potion = base.item_val.sv_potion_cure_critical;
            let scroll = base.item_val.sv_scroll_recharging;
            let food = base.item_val.sv_food_ration;
            // Each tuple names a real source branch; positive means selected and
            // queued on the constructed copy, never consumed in the engine.
            let cases = [
                (
                    "quaff-critical",
                    Decision::ItemQuaffCrit { no_check: true },
                    TV_POTION,
                    potion,
                    0,
                    true,
                    true,
                    0,
                    true,
                ),
                (
                    "quaff-potion",
                    Decision::ItemQuaffPotion { sval: potion },
                    TV_POTION,
                    potion,
                    0,
                    true,
                    true,
                    0,
                    true,
                ),
                (
                    "read-scroll",
                    Decision::ItemReadScroll { sval: scroll },
                    TV_SCROLL,
                    scroll,
                    0,
                    true,
                    true,
                    0,
                    true,
                ),
                (
                    "eat",
                    Decision::ItemEat {
                        tval: TV_FOOD as i32,
                        sval: food,
                    },
                    TV_FOOD,
                    food,
                    0,
                    true,
                    true,
                    0,
                    true,
                ),
                (
                    "eat-normal-food",
                    Decision::ItemEatFoodAny,
                    TV_FOOD,
                    food,
                    0,
                    true,
                    true,
                    0,
                    true,
                ),
                (
                    "equip-rod",
                    Decision::ItemEquipsRod { sval: 1 },
                    TV_ROD,
                    1,
                    1,
                    true,
                    true,
                    0,
                    false,
                ),
                (
                    "zap-rod",
                    Decision::ItemZapRod { sval: 1 },
                    TV_ROD,
                    1,
                    1,
                    true,
                    true,
                    0,
                    true,
                ),
                (
                    "use-staff",
                    Decision::ItemUseStaff { sval: 1 },
                    TV_STAFF,
                    1,
                    1,
                    true,
                    true,
                    0,
                    true,
                ),
                (
                    "use-staff-fail",
                    Decision::ItemUseStaffFail { sval: 1 },
                    TV_STAFF,
                    1,
                    1,
                    true,
                    true,
                    0,
                    true,
                ),
                (
                    "equip-staff-fail",
                    Decision::ItemEquipsStaffFail { sval: 1 },
                    TV_STAFF,
                    1,
                    1,
                    true,
                    true,
                    0,
                    false,
                ),
                (
                    "aim-wand",
                    Decision::ItemAimWand { sval: 1 },
                    TV_WAND,
                    1,
                    1,
                    true,
                    true,
                    0,
                    true,
                ),
                (
                    "equip-ring",
                    Decision::ItemEquipsRing { ring_sval: 1 },
                    TV_RING,
                    1,
                    0,
                    true,
                    true,
                    pack + 2,
                    false,
                ),
                (
                    "activate-ring",
                    Decision::ItemActivateRing { ring_sval: 1 },
                    TV_RING,
                    1,
                    0,
                    true,
                    true,
                    pack + 2,
                    true,
                ),
                (
                    "equip-dragon",
                    Decision::ItemEquipsDragon { drag_sval: 1 },
                    TV_DRAG_ARMOR,
                    1,
                    0,
                    true,
                    true,
                    pack + 6,
                    false,
                ),
                (
                    "activate-dragon",
                    Decision::ItemActivateDragon { drag_sval: 1 },
                    TV_DRAG_ARMOR,
                    1,
                    0,
                    true,
                    true,
                    pack + 6,
                    true,
                ),
                (
                    "activate-item",
                    Decision::ItemActivateItem { activation },
                    TV_RING,
                    1,
                    0,
                    true,
                    true,
                    pack + 2,
                    true,
                ),
                (
                    "equip-item",
                    Decision::ItemEquipsItem {
                        activation,
                        check_charge: true,
                    },
                    TV_RING,
                    1,
                    0,
                    true,
                    true,
                    pack + 2,
                    false,
                ),
                (
                    "use-stat-gain",
                    Decision::ItemUseThings,
                    TV_POTION,
                    base.item_val.sv_potion_inc_str,
                    0,
                    true,
                    true,
                    0,
                    true,
                ),
            ];
            for (label, request, tval, sval, charges, aware, ident, slot, commands) in cases {
                let mut state = base.clone();
                let item = &mut state.items.items[slot];
                *item = blank_item();
                item.iqty = 1;
                item.tval = tval as u8;
                item.sval = sval as u8;
                item.pval = charges;
                item.aware = aware;
                item.ident = ident;
                item.level = 1;
                item.activ_idx = activation;
                state.borg.activation[activation as usize] = 1;
                check(
                    &mut out,
                    &state,
                    mode,
                    label,
                    request,
                    Value::Acted(true),
                    commands,
                    &mut dispatch,
                );
                rows += 1;
            }
            let mut failure = base.clone();
            let item = &mut failure.items.items[0];
            *item = blank_item();
            item.iqty = 1;
            item.tval = TV_RING as u8;
            item.sval = 1;
            item.pval = 1;
            item.activ_idx = activation;
            item.level = 1;
            item.aware = true;
            check(
                &mut out,
                &failure,
                mode,
                "activation-failure-formula",
                Decision::ItemActivateFailure {
                    tval: TV_RING as i32,
                    sval: 1,
                },
                Value::Score(-20),
                false,
                &mut dispatch,
            );
            rows += 1;
            let mut recharge = base.clone();
            for (slot, tval, sval) in [(0, TV_WAND, 1), (1, TV_SCROLL, scroll)] {
                let item = &mut recharge.items.items[slot];
                *item = blank_item();
                item.iqty = 1;
                item.tval = tval as u8;
                item.sval = sval as u8;
                item.aware = true;
                item.ident = true;
            }
            check(
                &mut out,
                &recharge,
                mode,
                "recharge-empty-wand",
                Decision::ItemRecharging,
                Value::Acted(true),
                true,
                &mut dispatch,
            );
            rows += 1;
            let mut dragon_boundary = base.clone();
            dragon_boundary.borg.trait_[BI_DEV as usize] = 10;
            let item = &mut dragon_boundary.items.items[pack + 6];
            *item = blank_item();
            item.iqty = 1;
            item.tval = TV_DRAG_ARMOR as u8;
            item.sval = 1;
            item.aware = true;
            item.ident = true;
            item.level = 100;
            check(
                &mut out,
                &dragon_boundary,
                mode,
                "dragon-zero-denominator-guard",
                Decision::ItemEquipsDragon { drag_sval: 1 },
                Value::Acted(false),
                false,
                &mut dispatch,
            );
            rows += 1;
            // Availability and restoration boundaries with present items.
            for (label, blind, confused, charge, timeout) in [
                ("blind-scroll", true, false, 1, 0),
                ("confused-scroll", false, true, 1, 0),
                ("empty-wand", false, false, 0, 0),
                ("charging-activation", false, false, 1, 10),
            ] {
                let mut state = base.clone();
                state.borg.trait_[BI_ISBLIND as usize] = i32::from(blind);
                state.borg.trait_[BI_ISCONFUSED as usize] = i32::from(confused);
                let (slot, tval, sval, request) = if blind || confused {
                    (
                        0,
                        TV_SCROLL,
                        scroll,
                        Decision::ItemReadScroll { sval: scroll },
                    )
                } else if timeout > 0 {
                    (
                        pack + 2,
                        TV_RING,
                        1,
                        Decision::ItemActivateItem { activation },
                    )
                } else {
                    (0, TV_WAND, 1, Decision::ItemAimWand { sval: 1 })
                };
                let item = &mut state.items.items[slot];
                *item = blank_item();
                item.iqty = 1;
                item.tval = tval as u8;
                item.sval = sval as u8;
                item.aware = true;
                item.ident = true;
                item.pval = charge;
                item.activ_idx = activation;
                item.timeout = timeout;
                state.borg.activation[activation as usize] = 1;
                check(
                    &mut out,
                    &state,
                    mode,
                    label,
                    request,
                    Value::Acted(false),
                    false,
                    &mut dispatch,
                );
                rows += 1;
            }
            let mut charge_gate = base.clone();
            charge_gate.borg.activation[activation as usize] = 1;
            let item = &mut charge_gate.items.items[pack + 2];
            *item = blank_item();
            item.iqty = 1;
            item.tval = TV_RING as u8;
            item.sval = 1;
            item.activ_idx = activation;
            item.timeout = 10;
            check(
                &mut out,
                &charge_gate,
                mode,
                "equips-item-charging",
                Decision::ItemEquipsItem {
                    activation,
                    check_charge: true,
                },
                Value::Acted(false),
                false,
                &mut dispatch,
            );
            rows += 1;
            check(
                &mut out,
                &charge_gate,
                mode,
                "equips-item-ignore-charge",
                Decision::ItemEquipsItem {
                    activation,
                    check_charge: false,
                },
                Value::Acted(true),
                false,
                &mut dispatch,
            );
            rows += 1;
            out.flush().unwrap();
            eprintln!(
            "R4 item-use constructed fixture: {rows} cases; queued actions on owned copies only"
        );
        })
    });
}

#[allow(clippy::too_many_arguments)] // Explicit fixture witness fields stay visible at each case.
fn check(
    out: &mut File,
    initial: &BorgState,
    mode: Mode,
    label: &str,
    request: Decision,
    expected: Value,
    commands: bool,
    dispatch: &mut impl FnMut(&mut BorgState, Decision) -> Value,
) {
    let mut state = initial.clone();
    let before = queued_keys(&state);
    let value = dispatch(&mut state, request.clone());
    let after = queued_keys(&state);
    let unchanged_items = state.items == initial.items;
    let row = serde_json::json!({"schema":"borg-reborn-rs/r4-item-use-constructed@1","synthetic":true,
        "mode":mode,"case":label,"request":request,"value":value,"expected":expected,
        "keys_before":before,"keys_after":after,"items_unchanged":unchanged_items,
        "engine_consumption":false,"stubbed_dependencies":["borg_io::borg_note_bytes"]});
    serde_json::to_writer(&mut *out, &row).unwrap();
    out.write_all(b"\n").unwrap();
    out.flush().unwrap();
    assert_eq!(value, expected, "{label}");
    assert!(
        unchanged_items,
        "{label}: selection must not consume inventory itself"
    );
    if commands {
        assert!(
            after.starts_with(&before) && after.len() > before.len(),
            "{label}: no command queued"
        );
    } else {
        assert_eq!(after, before, "{label}: unexpected command");
    }
}
