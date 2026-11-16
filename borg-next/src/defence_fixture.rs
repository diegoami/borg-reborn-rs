//! Declared constructed memories, never survival or population evidence.
//! Only compiled by test-r3-defence; the live Borg state is never written.
use crate::shadow::Mode;
use borg_rs::memory::{traits::*, BorgState};
use borg_rs::threat_facts::defence_fixture as native;
use std::{fs::File, io::Write, sync::Once};

pub(crate) fn run(live: &BorgState, mode: Mode) {
    static ONCE: Once = Once::new();
    ONCE.call_once(|| {
        let dir = std::env::var_os("BORG_RUN_DIR").expect("fixture output");
        let mut output =
            File::create(std::path::PathBuf::from(dir).join("r3-defence-fixtures.jsonl")).unwrap();
        let total_spells = native::class_total_spells() as usize;
        assert!(
            total_spells > 0,
            "run this declared fixture with a caster class"
        );
        let mut base = live.clone();
        base.borg.trait_.fill(0);
        for (index, value) in [
            (BI_CURHP, 1000),
            (BI_MAXHP, 1000),
            (BI_CURSP, 1000),
            (BI_MAXSP, 1000),
            (BI_CLEVEL, 50),
            (BI_CDEPTH, 20),
            (BI_SPEED, 110),
            (BI_CLASS, 1),
        ] {
            base.borg.trait_[index as usize] = value;
        }
        base.borg.avoidance = 500;
        base.borg.temp = Default::default();
        base.borg.near = Default::default();
        base.borg.mon.unique = 0;
        base.borg.mon.scary = false;
        base.borg.status.anti_summon = false;
        base.borg.morgoth_position = false;
        base.borg.has.fill(0);
        base.borg.activation.fill(0);
        base.borg.book_idx.fill(-1);
        base.magic.magics.clear();
        for item in &mut base.items.items {
            item.iqty = 0;
        }
        base.kills.nxt = 1;
        base.kills.kills.fill(Default::default());
        for row in &mut base.danger.fear_region {
            row.fill(0);
        }
        for row in &mut base.danger.fear_monsters {
            row.fill(0);
        }
        base.traits.simulate = true;
        let mut rows = 0;
        for choice in 0..31 {
            check(
                &mut output,
                &base,
                mode,
                &format!("selector-{choice}"),
                choice,
                None,
            );
            rows += 1;
        }
        let cases = [
            (1, "speed-restoration", native::HASTE_SELF, 0, 0),
            (2, "grim-confusion-nonbinary", native::GRIM_PURPOSE, 7, 0),
            (2, "grim-free-action-nonbinary", native::GRIM_PURPOSE, 0, 7),
            (4, "fire-restoration", native::RESISTANCE, 0, 0),
            (5, "cold-restoration", native::RESISTANCE, 0, 0),
            (6, "acid-restoration", native::RESISTANCE, 0, 0),
            (7, "electricity-restoration", native::RESISTANCE, 0, 0),
            (8, "poison-restoration", native::RESIST_POISON, 0, 0),
            (
                9,
                "protection-evil-restoration",
                native::PROTECTION_FROM_EVIL,
                0,
                0,
            ),
        ];
        for (choice, label, spell, confusion, free_action) in cases {
            let mut state = base.clone();
            state.magic.magics = vec![native::BorgMagic::default(); total_spells];
            state.magic.magics[0] = native::BorgMagic {
                spell_enum: spell,
                status: 5,
                level: 1,
                power: 1,
                sfail: 0,
                book: 0,
                ..Default::default()
            };
            state.borg.book_idx[0] = 0;
            state.borg.trait_[BI_RCONF as usize] = confusion;
            state.borg.trait_[BI_FRACT as usize] = free_action;
            assert!(
                borg_rs::decision_api::borg_magic::borg_spell_okay_fail(&state, spell, 5),
                "constructed spell unavailable: {label}"
            );
            let normalized =
                (choice == 2).then_some((i32::from(confusion != 0), i32::from(free_action != 0)));
            check(&mut output, &state, mode, label, choice, normalized);
            rows += 1;
        }
        let mut shield = base.clone();
        shield.borg.has[shield.item_val.kv_mush_stoneskin as usize] = 1;
        check(&mut output, &shield, mode, "shield-restoration", 10, None);
        rows += 1;
        let mut all_resists = base.clone();
        let activation = all_resists.act.resist_all;
        all_resists.borg.activation[activation as usize] = 1;
        let slot = usize::from(borg_rs::threat_facts::z_info().pack_size);
        all_resists.items.items[slot].iqty = 1;
        all_resists.items.items[slot].activ_idx = activation;
        all_resists.items.items[slot].timeout = 0;
        check(
            &mut output,
            &all_resists,
            mode,
            "all-resists-restoration",
            3,
            None,
        );
        rows += 1;
        let mut morgoth = base.clone();
        morgoth.borg.trait_[BI_CDEPTH as usize] = 100;
        morgoth.borg.morgoth_position = true;
        morgoth.borg.near.morgoth = true;
        check(
            &mut output,
            &morgoth,
            mode,
            "morgoth-rest-simulation",
            27,
            None,
        );
        rows += 1;
        output.flush().unwrap();
        eprintln!(
            "R3 defence constructed fixture: {rows} native/port matches; live memory untouched"
        );
    });
}

fn check(
    output: &mut File,
    initial: &BorgState,
    mode: Mode,
    label: &str,
    choice: i32,
    normalized: Option<(i32, i32)>,
) {
    let mut state = initial.clone();
    let context = serde_json::json!({"schema":"borg-reborn-rs/shadow@3", "mode":mode,
        "point":"defence.constructed", "case":label, "choice":choice,
        "synthetic":true,"turn":null});
    let (value, replayed, agreement, events, _) = match mode {
        Mode::Check => borg_rs::decision_tape::compare(
            &mut state,
            context,
            |s| native::fixture_choice(s, choice, 1000),
            |s| crate::stateful::defence::fixture_choice(s, choice, 1000),
        ),
        Mode::Switch => borg_rs::decision_tape::compare(
            &mut state,
            context,
            |s| crate::stateful::defence::fixture_choice(s, choice, 1000),
            |s| native::fixture_choice(s, choice, 1000),
        ),
        Mode::Off => unreachable!("fixture requires shadow comparison"),
    };
    let restored = state.borg.temp == initial.borg.temp;
    let traits = (
        state.borg.trait_[BI_RCONF as usize],
        state.borg.trait_[BI_FRACT as usize],
    );
    let row = serde_json::json!({"schema":"borg-reborn-rs/r3-defence-constructed@1",
        "synthetic":true,"mode":mode,"case":label,"choice":choice,
        "driving_value":value,"replayed_value":replayed,"agreement_error":agreement.as_ref().err(),
        "tape_events":events,"temporary_buffs_restored":restored,"final_confusion_free_action":traits,
        "expected_native_boolean_restoration":normalized});
    serde_json::to_writer(&mut *output, &row).unwrap();
    output.write_all(b"\n").unwrap();
    output.flush().unwrap();
    assert!(agreement.is_ok(), "{label}: {agreement:?}");
    assert_eq!(replayed, Some(value), "{label}");
    assert!(restored, "{label}: temporary buffs not restored");
    if let Some(expected) = normalized {
        assert_eq!(
            traits, expected,
            "{label}: native boolean restoration never reached"
        );
    }
    if label.contains("restoration") || label.starts_with("grim-") {
        let minimum = match choice {
            10 => 1,
            9 => 2,
            _ => 3,
        };
        assert!(
            events >= minimum,
            "{label}: guarded simulation was not reached"
        );
    }
    if label == "morgoth-rest-simulation" {
        assert!(value > 0, "Morgoth rest path not reached");
    }
}
