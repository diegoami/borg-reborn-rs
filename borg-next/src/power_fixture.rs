//! R434 synthetic valuation controls over copied memory and real engine facts.
//! These cases never issue a game command or count as independent games.
use crate::shadow::Mode;
use borg_rs::decisions::{Decision, ReadDecision, Value};
use borg_rs::memory::{traits::*, BorgState};
use borg_rs::shadow::Shadow;
use borg_rs::threat_facts::{param, Param};
use std::{cell::Cell, fs::File, io::Write, sync::Once};

pub(crate) fn run(live: &BorgState, mode: Mode, provider: &impl Shadow) {
    static ONCE: Once = Once::new();
    ONCE.call_once(|| {
        assert_ne!(mode, Mode::Off);
        let mut base = live.clone();
        base.borg.trait_.fill(0);
        base.borg.has.fill(0);
        base.borg.activation.fill(0);
        base.borg.book_idx.fill(-1);
        base.borg.amt_book.fill(0);
        base.magic.magics.clear();
        for item in &mut base.items.items {
            *item = borg_rs::threat_facts::item_use_fixture::blank_item();
        }
        base.trait_swap.weapon_swap_value = 0;
        base.trait_swap.armour_swap_value = 0;
        base.core.cfg
            [borg_rs::decision_api::borg_headers::borg_h::BORG_USES_DYNAMIC_CALCS as usize] = 0;
        base.borg.trait_[BI_CARRY as usize] = 1000;
        base.prepared.buffer = b"R434 sentinel".to_vec();
        let mut out = File::create(
            std::path::PathBuf::from(std::env::var_os("BORG_RUN_DIR").unwrap())
                .join("r4-power-fixtures.jsonl"),
        )
        .unwrap();
        let equipment = score(provider.read_decision(&base, ReadDecision::PowerEquipment));
        emit(
            &mut out,
            mode,
            "empty-equipment",
            equipment,
            None,
            &base,
            &base,
        );
        let mut collision = base.clone();
        let index = collision.act.mapping;
        assert!(index > 0);
        collision.act.dragon_power = index;
        collision.borg.activation[index as usize] = 1;
        let value = score(provider.read_decision(&collision, ReadDecision::PowerEquipment));
        emit(
            &mut out,
            mode,
            "activation-first-match",
            value.wrapping_sub(equipment),
            Some(550),
            &collision,
            &collision,
        );
        let mut total_state = base.clone();
        let total = score(provider.decision(&mut total_state, Decision::Power));
        emit(
            &mut out,
            mode,
            "static-empty-preparation-stop",
            total,
            None,
            &base,
            &total_state,
        );
        for (label, weapon, armour) in [
            ("swap-positive", 12345, 6789),
            ("swap-overflow", i32::MAX, 1),
            ("swap-negative", -9000, -13),
        ] {
            let mut state = base.clone();
            state.trait_swap.weapon_swap_value = weapon;
            state.trait_swap.armour_swap_value = armour;
            let before = state.clone();
            let value = score(provider.decision(&mut state, Decision::Power));
            emit(
                &mut out,
                mode,
                label,
                value,
                Some(total.wrapping_add(weapon).wrapping_add(armour)),
                &before,
                &state,
            );
        }
        let mut fuel = base.clone();
        fuel.borg.trait_[BI_AFUEL as usize] = 1;
        let before = fuel.clone();
        let value = score(provider.decision(&mut fuel, Decision::Power));
        emit(
            &mut out,
            mode,
            "inventory-first-fuel",
            value.wrapping_sub(total),
            Some(param(Param::power_inventory_afuel_weight)),
            &before,
            &fuel,
        );
        // The real preparation service formats the depth-limit reason. Its buffer must persist.
        let mut prepared = base.clone();
        for (index, value) in [
            (BI_MAXCLEVEL, 50),
            (BI_MAXDEPTH, 2),
            (BI_LIGHT, 2),
            (BI_FOOD, 10),
            (BI_RECALL, 10),
            (BI_MAXHP, 500),
        ] {
            prepared.borg.trait_[index as usize] = value;
        }
        prepared.core.cfg[borg_rs::decision_api::borg_headers::borg_h::BORG_NO_DEEPER as usize] = 2;
        let before = prepared.clone();
        let value = score(provider.decision(&mut prepared, Decision::Power));
        assert_eq!(prepared.prepared.buffer, b"No deeper 2.");
        emit(
            &mut out,
            mode,
            "preparation-buffer-persists",
            value,
            None,
            &before,
            &prepared,
        );
        let mut dynamic = base.clone();
        dynamic.formulas.power.clear();
        dynamic.formulas.depth.clear();
        dynamic.formulas.restock.clear();
        dynamic.core.cfg
            [borg_rs::decision_api::borg_headers::borg_h::BORG_USES_DYNAMIC_CALCS as usize] = 1;
        let before = dynamic.clone();
        let value = score(provider.decision(&mut dynamic, Decision::Power));
        emit(
            &mut out,
            mode,
            "explicit-shared-dynamic-formula",
            value,
            None,
            &before,
            &dynamic,
        );
        // The power provider is entered under an opaque outer effect.
        // This checks nested provider replay, not the native public facade.
        // The outer replay must consume its state/result without invoking it twice.
        let calls = Cell::new(0);
        let mut nested = base.clone();
        let service = |st: &mut BorgState| {
            calls.set(calls.get() + 1);
            provider.decision(st, Decision::Power)
        };
        let (_, replayed, agreement, events, _) = borg_rs::decision_tape::compare(
            &mut nested,
            serde_json::json!({"synthetic":true}),
            |s| borg_rs::decision_tape::effect(s, "r434::outer-power", String::new(), service),
            |s| borg_rs::decision_tape::effect(s, "r434::outer-power", String::new(), service),
        );
        assert!(agreement.is_ok());
        assert_eq!(replayed, Some(Value::Score(total)));
        assert_eq!(events, 1);
        assert_eq!(calls.get(), 1);
        assert!(!borg_rs::decision_tape::in_transaction());
        emit(
            &mut out,
            mode,
            "nested-effect-once",
            calls.get(),
            Some(1),
            &base,
            &nested,
        );
        // Reject a deliberately incorrect return, then verify the repaired policy.
        // Literal mutation rejection is checked separately by the source audit; this is a return-fault control.
        let mut faulty = base.clone();
        let (_, _, agreement, _, _) = borg_rs::decision_tape::compare(
            &mut faulty,
            serde_json::json!({"synthetic":true}),
            |s| Decision::Power.native(s),
            |s| match crate::stateful::decide(s, Decision::Power) {
                Value::Score(v) => Value::Score(v.wrapping_add(1)),
                other => other,
            },
        );
        assert_eq!(agreement, Err("decision return differs".into()));
        emit(
            &mut out,
            mode,
            "reject-return-fault",
            i32::from(agreement.is_err()),
            Some(1),
            &base,
            &faulty,
        );
        let mut repaired = base.clone();
        let value = score(provider.decision(&mut repaired, Decision::Power));
        emit(
            &mut out,
            mode,
            "repaired-power",
            value,
            Some(total),
            &base,
            &repaired,
        );
        out.flush().unwrap();
        eprintln!("R4 power constructed fixture complete");
    });
}
fn score(value: Value) -> i32 {
    match value {
        Value::Score(v) => v,
        _ => panic!("power did not return a score"),
    }
}
fn emit(
    out: &mut File,
    mode: Mode,
    label: &str,
    value: i32,
    expected: Option<i32>,
    before: &BorgState,
    after: &BorgState,
) {
    let row = serde_json::json!({"schema":"borg-reborn-rs/r4-power-constructed@2","synthetic":true,"mode":mode,
        "case":label,"value":value,"expected":expected,
        "control_kind":if expected.is_some() { "value-assertion" } else { "parity-and-side-effects" },"items_unchanged":before.items==after.items,
        "prepared_buffer_changed":before.prepared.buffer!=after.prepared.buffer,"engine_consumption":false,
        "limits":"shared dynamic formulas are replayed, not independently owned; empty/branch controls are not game trials"});
    serde_json::to_writer(&mut *out, &row).unwrap();
    out.write_all(b"\n").unwrap();
    out.flush().unwrap();
    if let Some(expected) = expected {
        assert_eq!(value, expected, "{label}");
    }
    assert!(
        before.items == after.items,
        "{label}: valuation changed items"
    );
}
