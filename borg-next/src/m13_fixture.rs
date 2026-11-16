//! Declared constructed states over loaded engine tables. No live-state writes,
//! engine effects or RNG; these are branch checks, never population evidence.
use borg_rs::memory::{traits::*, BorgKill, BorgState};
use borg_rs::threat_facts::{self as facts, constants::*};
use std::{fs::File, io::Write, sync::Once};

pub(crate) fn run(live: &BorgState, mode: crate::shadow::Mode) {
    static ONCE: Once = Once::new();
    ONCE.call_once(|| {
        let dir = std::env::var_os("BORG_RUN_DIR").expect("fixture output");
        let mut output = File::create(std::path::PathBuf::from(dir).join("m13-fixtures.jsonl")).unwrap();
        let mut state = live.clone();
        state.borg.trait_.fill(0);
        state.borg.trait_[BI_CURHP as usize] = 5000;
        state.borg.trait_[BI_MAXHP as usize] = 5000;
        state.borg.trait_[BI_CLEVEL as usize] = 30;
        state.borg.trait_[BI_CDEPTH as usize] = 20;
        state.borg.temp = Default::default();
        state.glyph.track.num = 0;
        let race = (1..usize::from(facts::z_info().r_max) - 1)
            .find(|&r| facts::race(r).spell_power >= 40).expect("spell-power table");
        let mut rows = 0;
        let mut faults = 0;
        let elements = [
            ("acid", BI_VACID, MONBLOW_ACID, [RSF_BR_ACID, RSF_BA_ACID, RSF_BO_ACID]),
            ("electricity", BI_VELEC, MONBLOW_ELEC, [RSF_BR_ELEC, RSF_BA_ELEC, RSF_BO_ELEC]),
            ("fire", BI_VFIRE, MONBLOW_FIRE, [RSF_BR_FIRE, RSF_BA_FIRE, RSF_BO_FIRE]),
            ("cold", BI_VCOLD, MONBLOW_COLD, [RSF_BR_COLD, RSF_BA_COLD, RSF_BO_COLD]),
        ];
        for (element, index, blow, spells) in elements {
            let melee_race = (1..usize::from(facts::z_info().r_max) - 1).find(|&r|
                (0..usize::from(facts::z_info().mon_blows_max)).any(|b| facts::blow(r, b).is_some_and(|b| b.effect == blow)))
                .expect("elemental melee table");
            let initial_faults = faults;
            for temporary in [false, true] {
                state.borg.temp.res_acid = temporary;
                state.borg.temp.res_elec = temporary;
                state.borg.temp.res_fire = temporary;
                state.borg.temp.res_cold = temporary;
                for vulnerable in [0, 1] {
                    state.borg.trait_[index as usize] = vulnerable;
                    for spell in spells.into_iter().chain((element == "electricity").then_some(RSF_BE_ELEC)) {
                        let mut kill = BorgKill { r_idx: race as u16, power: 900, awake: true, ranged_attack: 1, ..Default::default() };
                        kill.spell[0] = spell as u8;
                        check(&mut output, &state, &kill, false, element, temporary, mode, &mut rows, &mut faults);
                    }
                    let kill = BorgKill { r_idx: melee_race as u16, power: 900, awake: true, ..Default::default() };
                    check(&mut output, &state, &kill, true, element, temporary, mode, &mut rows, &mut faults);
                }
            }
            assert!(faults > initial_faults, "vulnerability projection fault must be detected: {element}");
            state.borg.trait_[index as usize] = 0;
        }
        let initial_faults = faults;
        for spell in [RSF_BR_SOUN, RSF_BR_GRAV, RSF_BR_PLAS, RSF_BR_WALL,
            RSF_BA_WATE, RSF_STORM, RSF_BO_WATE, RSF_BO_PLAS, RSF_BO_ICE] {
            for sound in [0, 1] {
                for protection in [0, 1] {
                    for stun in [0, 1] {
                        state.borg.trait_[BI_RSND as usize] = sound;
                        state.borg.trait_[BI_PSTUN as usize] = protection;
                        state.borg.trait_[BI_ISSTUN as usize] = stun;
                        let mut kill = BorgKill { r_idx: race as u16, power: 900, awake: true, ranged_attack: 1, ..Default::default() };
                        kill.spell[0] = spell as u8;
                        check(&mut output, &state, &kill, false, "stun", false, mode, &mut rows, &mut faults);
                    }
                }
            }
        }
        assert!(faults > initial_faults, "stun projection fault must be detected");
        output.flush().unwrap();
        assert!(*live != state, "fixture must be distinct from the live state");
        eprintln!("M13 constructed-state fixture: {rows} native/port matches; {faults} detected projection faults");
    });
}

#[allow(clippy::too_many_arguments)]
fn check(
    output: &mut File,
    state: &BorgState,
    kill: &BorgKill,
    physical: bool,
    branch: &str,
    temporary: bool,
    mode: crate::shadow::Mode,
    rows: &mut usize,
    faults: &mut usize,
) {
    let mut native_state = state.clone();
    let native = facts::fixture_estimate(&mut native_state, kill, physical);
    assert!(native_state == *state, "native estimator mutated fixture");
    let port = crate::danger::fixture_estimate(state, kill, physical, mode);
    assert_eq!(
        native, port,
        "M13 {branch} physical={physical} spell={}",
        kill.spell[0]
    );
    // A deliberate test-only input projection fault. Production ports still
    // execute their independent M13 bodies; no stale-policy fallback exists.
    let mut broken = state.clone();
    for index in [BI_VACID, BI_VELEC, BI_VFIRE, BI_VCOLD, BI_PSTUN] {
        broken.borg.trait_[index as usize] = 0;
    }
    let fault = crate::danger::fixture_estimate(&broken, kill, physical, mode);
    *faults += usize::from(fault != native);
    let row = serde_json::json!({"schema":"borg-reborn-rs/m13-constructed@1", "synthetic":true,
        "branch":branch,"physical":physical,"race":kill.r_idx,
        "race_name":String::from_utf8_lossy(facts::race_name(usize::from(kill.r_idx))),
        "spell":kill.spell[0],"temporary":temporary,"traits":state.borg.trait_,
        "native":native,"ported":port,"faulted_projection":fault,"fault_detected":fault != native});
    serde_json::to_writer(&mut *output, &row).unwrap();
    output.write_all(b"\n").unwrap();
    *rows += 1;
}
