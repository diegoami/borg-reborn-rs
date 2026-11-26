//! R436 controls use copied memory, real shared facts, and queued commands only.
//! Shared note UI callbacks are stubbed; the note's actual anti-loop state logic is retained.
use crate::shadow::Mode;
use borg_rs::decisions::{Decision, Value};
use borg_rs::memory::{traits::*, BorgState};
use borg_rs::shadow::Shadow;
use borg_rs::threat_facts::{
    equipment_fixture as facts,
    item_use_fixture::{blank_item, queued_keys},
};
use std::{cell::Cell, fs::File, io::Write, sync::Once};

pub(crate) fn run(live: &BorgState, mode: Mode, provider: &impl Shadow) {
    static ONCE: Once = Once::new();
    ONCE.call_once(|| {
        borg_rs::decision_api::item_use_fixture::with_stubbed_messages(|| {
            assert_ne!(mode, Mode::Off);
            let mut base = live.clone();
            for item in &mut base.items.items {
                *item = blank_item();
            }
            base.items.safe_items = base.items.items.clone();
            base.borg.trait_[BI_ISWEAK as usize] = 1;
            base.borg.trait_[BI_ISHUNGRY as usize] = 1;
            base.borg.trait_[BI_CDEPTH as usize] = 0;
            base.borg.trait_[BI_QUIVER_SLOTS as usize] = 0;
            base.borg.goal.best_item.clear();
            base.borg.goal.do_best = false;
            base.trait_swap.weapon_swap = 0;
            base.trait_swap.armour_swap = 0;
            base.think.shop_num = -1;
            base.borg.time.level = base.borg.time.now;
            let wield = borg_rs::decision_api::borg_item::inven_wield() as usize;
            let mut out = File::create(
                std::path::PathBuf::from(std::env::var_os("BORG_RUN_DIR").unwrap())
                    .join("r4-equipment-fixtures.jsonl"),
            )
            .unwrap();
            for (label, request, expected) in [
                (
                    "empty-identify",
                    Decision::WearTestStuff,
                    Value::Acted(false),
                ),
                (
                    "empty-ring-reorder",
                    Decision::WearSwapRings,
                    Value::Acted(false),
                ),
                ("hungry-ring-wear", Decision::WearRings, Value::Acted(false)),
                (
                    "hungry-backup-swap",
                    Decision::WearBackupSwap { p: 10 },
                    Value::Acted(false),
                ),
                ("hungry-wear", Decision::WearStuff, Value::Acted(false)),
                (
                    "best-outside-home",
                    Decision::WearBestStuff,
                    Value::Acted(false),
                ),
                ("clear-empty-plan", Decision::WearClearBest, Value::Unit),
                ("weak-recharge", Decision::WearRecharge, Value::Acted(false)),
                // The native inscription scan reports completion even when no command is queued.
                (
                    "empty-inscription-scan",
                    Decision::ObjectFullyId,
                    Value::Acted(true),
                ),
            ] {
                run_case(
                    &mut out,
                    &base,
                    mode,
                    label,
                    request,
                    expected,
                    Some(vec![]),
                    provider,
                );
            }
            let mut ring = base.clone();
            ring.items.items[wield + 3] = blank_item();
            ring.items.items[wield + 3].tval =
                borg_rs::decision_api::headers::list_tvals_h::TV_RING as u8;
            ring.items.items[wield + 3].iqty = 1;
            let count = ring.borg.antibounce_count;
            let state = run_case(
                &mut out,
                &ring,
                mode,
                "remove-loose-ring",
                Decision::WearSwapRings,
                Value::Acted(true),
                Some(vec![u32::from(b't'), u32::from(b'd')]),
                provider,
            );
            assert_eq!(
                state.borg.antibounce_count,
                count.wrapping_add(10),
                "stub must preserve actual shared note state effects"
            );
            // Synthetic candidate: exercise the real positive wear chooser and its trials.
            // Trials leave residual slot bytes, so assert quantities rather than full rollback.
            let mut wear = base.clone();
            wear.borg.trait_[BI_ISWEAK as usize] = 0;
            wear.borg.trait_[BI_ISHUNGRY as usize] = 0;
            wear.borg.power = 100;
            wear.items.items[0].iqty = 2;
            wear.items.items[0].aware = true;
            wear.items.items[0].ident = true;
            wear.items.items[0].value = 1;
            wear.items.items[0].one_ring = true;
            wear.items.items[0].tval = borg_rs::decision_api::headers::list_tvals_h::TV_RING as u8;
            let before = queued_keys(&wear);
            let count = wear.borg.antibounce_count;
            let value = provider.decision(&mut wear, Decision::WearRings);
            assert_eq!(value, Value::Acted(true));
            let keys = queued_keys(&wear);
            assert!(keys.starts_with(&before));
            assert_eq!(&keys[before.len()..], &[u32::from(b'w'), u32::from(b'a')]);
            assert_eq!(wear.items.items[0].iqty, 2);
            assert_eq!(wear.items.items[wield + 2].iqty, 0);
            assert_eq!(wear.items.items[wield + 3].iqty, 0);
            assert_eq!(wear.borg.antibounce_count, count.wrapping_add(1));
            record(
                &mut out,
                mode,
                "wear-synthetic-one-ring",
                serde_json::to_value(value).unwrap(),
                Some(serde_json::to_value(Value::Acted(true)).unwrap()),
            );
            let mut identify = base.clone();
            identify.borg.trait_[BI_CDEPTH as usize] = 1;
            identify.items.items[0].iqty = 1;
            identify.items.items[0].ident = true;
            identify.items.items[0].tval =
                borg_rs::decision_api::headers::list_tvals_h::TV_BOW as u8;
            identify.items.items[0].art_idx = 1;
            identify.items.items[0].modifiers
                [borg_rs::decision_api::headers::list_object_modifiers_h::OBJ_MOD_SPEED as usize] =
                1;
            run_case(
                &mut out,
                &identify,
                mode,
                "inscribe-known-speed-artifact",
                Decision::ObjectFullyId,
                Value::Acted(true),
                Some(vec![
                    u32::from(b'{'),
                    u32::from(b'a'),
                    u32::from(b'S'),
                    u32::from(b'p'),
                    u32::from(b'd'),
                    borg_rs::decision_api::borg_io::KC_ENTER,
                ]),
                provider,
            );
            let mut recharge = base.clone();
            recharge.borg.trait_[BI_ISWEAK as usize] = 0;
            recharge.borg.trait_[BI_ISHUNGRY as usize] = 0;
            recharge.items.items[0].iqty = 1;
            recharge.items.items[0].tval =
                borg_rs::decision_api::headers::list_tvals_h::TV_ROD as u8;
            recharge.items.items[0].sval = recharge.item_val.sv_rod_recall as u8;
            recharge.items.items[0].timeout = 3;
            run_case(
                &mut out,
                &recharge,
                mode,
                "recall-rod-recharge",
                Decision::WearRecharge,
                Value::Acted(true),
                Some(vec![
                    u32::from(b'R'),
                    u32::from(b'7'),
                    u32::from(b'5'),
                    borg_rs::decision_api::borg_io::KC_ENTER,
                ]),
                provider,
            );
            let mut plan = base.clone();
            plan.borg.goal.best_item = vec![borg_rs::decision_api::borg::BorgBest::default()];
            let cleared = run_case(
                &mut out,
                &plan,
                mode,
                "clear-populated-plan",
                Decision::WearClearBest,
                Value::Unit,
                Some(vec![]),
                provider,
            );
            assert!(cleared.borg.goal.best_item.is_empty() && cleared.borg.goal.do_best);
            let mut critical = base.clone();
            for (index, value) in [(BI_RCONF, 2), (BI_RBLIND, 3), (BI_FRACT, 7), (BI_SAV, 99)] {
                critical.borg.trait_[index as usize] = value;
            }
            let actual = crate::stateful::equipment_wear::lost_critical(&critical, 2, 3, 7);
            assert_eq!(actual, facts::critical(&critical, 2, 3, 7));
            assert_eq!(actual, [false, false, false]);
            record(
                &mut out,
                mode,
                "nonbinary-critical-preserved",
                serde_json::json!(actual),
                Some(serde_json::json!([false, false, false])),
            );
            for index in [BI_RCONF, BI_RBLIND, BI_FRACT, BI_RLITE, BI_RDARK] {
                critical.borg.trait_[index as usize] = 0;
            }
            let actual = crate::stateful::equipment_wear::lost_critical(&critical, 2, 3, 7);
            assert_eq!(actual, facts::critical(&critical, 2, 3, 7));
            assert_eq!(actual, [true, true, true]);
            record(
                &mut out,
                mode,
                "lost-critical-rejected",
                serde_json::json!(actual),
                Some(serde_json::json!([true, true, true])),
            );
            for (label, args, expected) in [
                ("armour-lower-danger", (3, 5, 20, 30), (20, 2)),
                ("weapon-wins-tie", (3, 5, 40, 40), (40, 4)),
            ] {
                let (a, w, x, y) = args;
                let actual = crate::stateful::equipment_wear::backup_swap_choice(a, w, x, y);
                assert_eq!(actual, facts::choice(a, w, x, y));
                assert_eq!(actual, expected);
                record(
                    &mut out,
                    mode,
                    label,
                    serde_json::json!(actual),
                    Some(serde_json::json!(expected)),
                );
            }
            let mut trial = live.clone();
            let slot = wield;
            let candidate = 0;
            let hole = wield - 1;
            let mut item = trial.items.items[slot];
            if item.iqty == 0 {
                item = blank_item();
                item.tval = borg_rs::decision_api::headers::list_tvals_h::TV_SWORD as u8;
                item.dd = 1;
                item.ds = 4;
            }
            item.iqty = 2;
            item.to_d = item.to_d.wrapping_add(3);
            item.ident = true;
            trial.items.items[candidate] = item;
            trial.items.items[hole] = blank_item();
            trial.items.safe_items = trial.items.items.clone();
            let before = trial.clone();
            let native =
                |s: &mut BorgState| facts::trial(s, hole as i32, slot as i32, candidate as i32);
            let port = |s: &mut BorgState| {
                crate::stateful::equipment_wear::wear_trial(
                    s,
                    hole as i32,
                    slot as i32,
                    candidate as i32,
                )
            };
            let (_, _, agreement, events, _) = if mode == Mode::Check {
                borg_rs::decision_tape::compare(
                    &mut trial,
                    serde_json::json!({"synthetic":true}),
                    native,
                    port,
                )
            } else {
                borg_rs::decision_tape::compare(
                    &mut trial,
                    serde_json::json!({"synthetic":true}),
                    port,
                    native,
                )
            };
            assert!(agreement.is_ok(), "trial: {agreement:?}");
            assert!(events >= 3);
            assert!(trial.items.items[candidate] == before.items.items[candidate]);
            assert!(trial.items.items[slot] == before.items.items[slot]);
            assert_eq!(trial.items.items[hole].iqty, 0);
            assert!(trial.items.safe_items == before.items.safe_items);
            // Native trial intentionally leaves recomputed traits and an empty hole's residual bytes.
            record(
                &mut out,
                mode,
                "trial-restores-slots-and-quantity",
                serde_json::json!(true),
                None,
            );
            let calls = Cell::new(0);
            let mut nested = base.clone();
            let service = |s: &mut BorgState| {
                calls.set(calls.get() + 1);
                borg_rs::decision_api::borg_item_wear::borg_backup_swap(s, 10)
            };
            let (_, _, agreement, _, _) = borg_rs::decision_tape::compare(
                &mut nested,
                serde_json::json!({"synthetic":true}),
                |s| {
                    borg_rs::decision_tape::effect(
                        s,
                        "r436::real-public-wear",
                        String::new(),
                        service,
                    )
                },
                |s| {
                    borg_rs::decision_tape::effect(
                        s,
                        "r436::real-public-wear",
                        String::new(),
                        service,
                    )
                },
            );
            assert!(agreement.is_ok());
            assert_eq!(calls.get(), 1);
            assert!(!borg_rs::decision_tape::in_transaction());
            record(
                &mut out,
                mode,
                "nested-public-facade-effect-once",
                serde_json::json!(calls.get()),
                Some(serde_json::json!(1)),
            );
            let mut fault = base.clone();
            let (_, _, agreement, _, _) = borg_rs::decision_tape::compare(
                &mut fault,
                serde_json::json!({"synthetic":true}),
                |s| Decision::WearBackupSwap { p: 10 }.native(s),
                |s| {
                    let _ = crate::stateful::decide(s, Decision::WearBackupSwap { p: 10 });
                    Value::Acted(true)
                },
            );
            assert_eq!(agreement, Err("decision return differs".into()));
            record(
                &mut out,
                mode,
                "reject-return-fault",
                serde_json::json!(agreement.is_err()),
                Some(serde_json::json!(true)),
            );
            run_case(
                &mut out,
                &base,
                mode,
                "repaired-backup-swap",
                Decision::WearBackupSwap { p: 10 },
                Value::Acted(false),
                Some(vec![]),
                provider,
            );
            let mut notes = base.clone();
            facts::bind_note(&mut notes.items.items[0], 1, 0);
            for (label, text, wanted) in [
                ("note-live-unknown", "{??}", true),
                ("note-live-known", "{known}", false),
                ("note-live-replaced", "{??} changed", true),
            ] {
                notes.items.items[1].desc.fill(0);
                for (dst, byte) in notes.items.items[1].desc.iter_mut().zip(text.bytes()) {
                    *dst = byte as std::ffi::c_char;
                }
                let item = notes.items.items[0];
                let native = |s: &BorgState| facts::needs_id(s, &item);
                let port = |s: &BorgState| {
                    crate::stateful::equipment_identify::borg_item_note_needs_id(s, &item)
                };
                let (value, _, agreement, _, _) = if mode == Mode::Check {
                    borg_rs::decision_tape::compare_read(
                        &notes,
                        serde_json::json!({"synthetic":true}),
                        native,
                        port,
                    )
                } else {
                    borg_rs::decision_tape::compare_read(
                        &notes,
                        serde_json::json!({"synthetic":true}),
                        port,
                        native,
                    )
                };
                assert!(agreement.is_ok());
                assert_eq!(value, wanted);
                record(
                    &mut out,
                    mode,
                    label,
                    serde_json::json!(value),
                    Some(serde_json::json!(wanted)),
                );
            }
            // Rand_div uses engine RNG machinery with the Borg quick RNG installed here.
            // generate_keypress restores the game RNG after borg_think/after_think returns.
            assert!(borg_rs::decision_api::equipment_fixture::rng_is_quick());
            let before_rng = borg_rs::decision_api::equipment_fixture::rng_draws();
            let mut rng_state = base.clone();
            let draw = |_: &mut BorgState| borg_rs::decision_api::engine::randint0(64);
            let (value, replayed, agreement, _, _) = borg_rs::decision_tape::compare(
                &mut rng_state,
                serde_json::json!({"synthetic":true}),
                draw,
                draw,
            );
            assert!(agreement.is_ok());
            assert_eq!(replayed, Some(value));
            assert!((0..64).contains(&value));
            let draws = borg_rs::decision_api::equipment_fixture::rng_draws() - before_rng;
            assert_eq!(draws, 1);
            record(
                &mut out,
                mode,
                "borg-rng-draw-once",
                serde_json::json!(draws),
                Some(serde_json::json!(1)),
            );
            out.flush().unwrap();
            eprintln!("R4 equipment constructed fixture complete");
        })
    });
}
#[allow(clippy::too_many_arguments)]
fn run_case(
    out: &mut File,
    base: &BorgState,
    mode: Mode,
    label: &str,
    request: Decision,
    expected: Value,
    commands: Option<Vec<u32>>,
    provider: &impl Shadow,
) -> BorgState {
    let mut state = base.clone();
    let before = queued_keys(&state);
    let value = provider.decision(&mut state, request);
    assert_eq!(value, expected, "{label}");
    let keys = queued_keys(&state);
    assert!(keys.starts_with(&before));
    if let Some(commands) = commands {
        assert_eq!(
            &keys[before.len()..],
            commands.as_slice(),
            "{label}: command ordering"
        );
    }
    assert!(
        state.items == base.items,
        "{label}: command selection consumed or changed items"
    );
    record(
        out,
        mode,
        label,
        serde_json::to_value(value).unwrap(),
        Some(serde_json::to_value(expected).unwrap()),
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
    let row = serde_json::json!({"schema":"borg-reborn-rs/r4-equipment-constructed@1","case":label,"synthetic":true,"mode":mode,
 "value":value,"expected":expected,"control_kind":if expected.is_some(){"value-and-state-assertion"}else{"parity-and-restoration"},
 "engine_consumption":false,"stubbed_dependencies":["shared note engine UI callbacks only; actual anti-loop state effects retained"]});
    serde_json::to_writer(&mut *out, &row).unwrap();
    out.write_all(b"\n").unwrap();
    out.flush().unwrap();
}
