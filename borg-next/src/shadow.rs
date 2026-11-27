//! Comparisons for owned danger, caution/escape/recovery and attack policy.
//! Stateful engine effects and RNG execute once through ordered replay.
use crate::View;
use borg_rs::decisions::{Decision, ReadDecision, Value};
use borg_rs::memory::terrain::*;
use borg_rs::memory::BorgState;
use borg_rs::shadow::{DangerInput, Shadow};
use serde::Serialize;
use std::collections::BTreeMap;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

pub(crate) const DIAGNOSTIC_SCHEMA: &str = "borg-reborn-rs/shadow@10";

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Mode {
    Off,
    Check,
    Switch,
}

impl Mode {
    pub fn parse(value: &str) -> Result<Self, String> {
        match value {
            "off" => Ok(Self::Off),
            "check" => Ok(Self::Check),
            "switch" => Ok(Self::Switch),
            _ => Err(format!(
                "BORG_SHADOW: expected off, check or switch, got {value:?}"
            )),
        }
    }
}

/// Source: borg-core/src/borg_danger.rs:wall_like, vendored e6d13fc.
/// These are native feature IDs, not a new passability policy. In particular
/// permanent walls, lava and passable rubble retain the original `false`.
pub fn wall_like(feature: u32) -> bool {
    matches!(
        feature,
        FEAT_RUBBLE | FEAT_MAGMA | FEAT_QUARTZ | FEAT_MAGMA_K | FEAT_QUARTZ_K | FEAT_GRANITE
    )
}

#[derive(Default, Serialize)]
struct Counts {
    thinks: u64,
    danger_calls: u64,
    danger_comparisons: u64,
    monster_comparisons: u64,
    wall_comparisons: u64,
    caution_calls: u64,
    escape_calls: u64,
    recovery_calls: u64,
    attack_calls: u64,
    defence_calls: u64,
    item_use_calls: u64,
    power_calls: u64,
    equipment_calls: u64,
    store_calls: u64,
    flow_calls: u64,
    pure_attack_comparisons: u64,
    pure_points: BTreeMap<&'static str, u64>,
    stateful_comparisons: u64,
    readonly_comparisons: u64,
    readonly_points: BTreeMap<&'static str, u64>,
    // @2: total events = policy markers + actual dependency requests.
    tape_events: u64,
    policy_markers: u64,
    dependency_requests: u64,
    root_points: BTreeMap<&'static str, u64>,
    stateful_points: BTreeMap<&'static str, u64>,
    caution_actions: u64,
    comparisons: u64,
    mismatches: u64,
}

impl Counts {
    fn record_events(&mut self, events: u64, visits: &BTreeMap<&'static str, u64>) {
        let markers: u64 = visits.values().sum();
        assert!(events >= markers, "policy markers exceed total tape events");
        self.tape_events += events;
        self.policy_markers += markers;
        self.dependency_requests += events - markers;
        for (&name, &count) in visits {
            *self.stateful_points.entry(name).or_default() += count;
            match name.split_once('.').map(|p| p.0) {
                Some("caution") => self.caution_calls += count,
                Some("escape") => self.escape_calls += count,
                Some("recovery") => self.recovery_calls += count,
                Some("attack") => self.attack_calls += count,
                Some("defence") => self.defence_calls += count,
                Some("item_use") => self.item_use_calls += count,
                Some("power") => self.power_calls += count,
                Some("equipment") => self.equipment_calls += count,
                Some("store") => self.store_calls += count,
                Some("flow") => self.flow_calls += count,
                _ => panic!("unknown policy marker namespace: {name}"),
            }
        }
    }
}

struct Recorder<W: Write> {
    mismatches: W,
    stats: W,
    counts: Counts,
}

impl<W: Write> Recorder<W> {
    fn new(mismatches: W, stats: W) -> Self {
        Self {
            mismatches,
            stats,
            counts: Counts::default(),
        }
    }

    fn compare_wall(
        &mut self,
        mode: Mode,
        turn: i32,
        borg_turn: i32,
        feature: u32,
        original: bool,
    ) -> std::io::Result<bool> {
        if mode == Mode::Off {
            return Ok(original);
        }
        let ported = wall_like(feature);
        self.counts.comparisons += 1;
        self.counts.wall_comparisons += 1;
        if original != ported {
            self.counts.mismatches += 1;
            let row = serde_json::json!({
                "schema": DIAGNOSTIC_SCHEMA, "mode": mode,
                "turn": turn, "borg_turn": borg_turn,
                "point": "danger.wall_like", "inputs": {"feature": feature},
                "original": original, "ported": ported,
            });
            // Direct writes preserve the last mismatch on engine exit.
            serde_json::to_writer(&mut self.mismatches, &row)?;
            self.mismatches.write_all(b"\n")?;
        }
        Ok(if mode == Mode::Switch {
            ported
        } else {
            original
        })
    }

    fn compare_danger(
        &mut self,
        mode: Mode,
        turn: i32,
        borg_turn: i32,
        input: DangerInput,
        slot: Option<i32>,
        results: (i32, i32),
    ) -> std::io::Result<i32> {
        let (original, ported) = results;
        if mode == Mode::Off {
            return Ok(original);
        }
        self.counts.comparisons += 1;
        if slot.is_some() {
            self.counts.monster_comparisons += 1;
        } else {
            self.counts.danger_comparisons += 1;
        }
        if original != ported {
            self.counts.mismatches += 1;
            let row = serde_json::json!({
                "schema":DIAGNOSTIC_SCHEMA, "mode":mode,
                "turn":turn, "borg_turn":borg_turn,
                "point":if slot.is_some() { "danger.one_monster" } else { "danger" },
                "inputs":input, "slot":slot, "original":original,"ported":ported,
            });
            serde_json::to_writer(&mut self.mismatches, &row)?;
            self.mismatches.write_all(b"\n")?;
        }
        Ok(if mode == Mode::Switch {
            ported
        } else {
            original
        })
    }

    #[allow(clippy::too_many_arguments)]
    fn compare_attack_scalar(
        &mut self,
        mode: Mode,
        turn: i32,
        borg_turn: Option<i32>,
        point: &'static str,
        input: impl FnOnce() -> serde_json::Value,
        original: i32,
        ported: i32,
    ) -> std::io::Result<i32> {
        if mode == Mode::Off {
            return Ok(original);
        }
        self.counts.comparisons += 1;
        self.counts.pure_attack_comparisons += 1;
        self.counts.attack_calls += 1;
        *self.counts.pure_points.entry(point).or_default() += 1;
        if original != ported {
            self.counts.mismatches += 1;
            let row = serde_json::json!({"schema":DIAGNOSTIC_SCHEMA,"mode":mode,
                "turn":turn,"borg_turn":borg_turn,"point":point,"inputs":input(),
                "original":original,"ported":ported});
            serde_json::to_writer(&mut self.mismatches, &row)?;
            self.mismatches.write_all(b"\n")?;
        }
        Ok(if mode == Mode::Switch {
            ported
        } else {
            original
        })
    }

    fn think(&mut self, mode: Mode, borg_turn: i32) -> std::io::Result<()> {
        self.counts.thinks += 1;
        let row = serde_json::json!({
            "schema": "borg-reborn-rs/shadow-stats@10", "mode": mode,
            "borg_turn": borg_turn,
            "surface": "r5",
            "ported_points": ["danger.wall_like", "danger", "danger.one_monster", "caution", "escape", "recovery", "attack", "defence", "item_use", "power", "equipment", "store", "flow"],
            "unported_points": [],
            "counts": self.counts,
        });
        serde_json::to_writer(&mut self.stats, &row)?;
        self.stats.write_all(b"\n")
    }
}

struct Next {
    mode: Mode,
    recorder: Mutex<Recorder<File>>,
}

impl Shadow for Next {
    fn blow_effect(&self, name: &[u8], original: i32) -> i32 {
        let ported = crate::stateful::attack::borg_mon_blow_effect(name);
        let turn = borg_rs::threat_facts::game_turn();
        let input = || serde_json::json!({"name_bytes":name});
        self.recorder
            .lock()
            .expect("shadow recorder")
            .compare_attack_scalar(
                self.mode,
                turn,
                None,
                "attack.borg_mon_blow_effect",
                input,
                original,
                ported,
            )
            .unwrap_or_else(|e| failure(e))
    }
    fn thrust_damage(&self, state: &BorgState, slot: i32, original: i32) -> i32 {
        let ported = crate::stateful::attack::borg_thrust_damage_one(state, slot);
        let turn = borg_rs::threat_facts::game_turn();
        let input = || serde_json::json!({"slot":slot});
        self.recorder
            .lock()
            .expect("shadow recorder")
            .compare_attack_scalar(
                self.mode,
                turn,
                Some(state.borg.time.now),
                "attack.borg_thrust_damage_one",
                input,
                original,
                ported,
            )
            .unwrap_or_else(|e| failure(e))
    }

    fn danger(&self, state: &BorgState, input: DangerInput, original: i32) -> i32 {
        let view = View::new(state);
        let ported = crate::danger::assess(&view, input, self.mode);
        let mut recorder = self.recorder.lock().expect("shadow recorder");
        recorder.counts.danger_calls += 1;
        recorder
            .compare_danger(
                self.mode,
                borg_rs::threat_facts::game_turn(),
                view.threat.clock.now,
                input,
                None,
                (original, ported),
            )
            .unwrap_or_else(|e| failure(e))
    }

    fn monster_danger(
        &self,
        state: &BorgState,
        input: DangerInput,
        slot: i32,
        original: i32,
    ) -> i32 {
        let view = View::new(state);
        let ported = crate::danger::assess_monster(&view, input, slot, self.mode);
        self.recorder
            .lock()
            .expect("shadow recorder")
            .compare_danger(
                self.mode,
                borg_rs::threat_facts::game_turn(),
                view.threat.clock.now,
                input,
                Some(slot),
                (original, ported),
            )
            .unwrap_or_else(|e| failure(e))
    }

    fn read_decision(&self, state: &BorgState, request: ReadDecision) -> Value {
        let mode = self.mode;
        if mode == Mode::Off {
            return request.native(state);
        }
        let context = serde_json::json!({"schema":DIAGNOSTIC_SCHEMA,"mode":mode,"point":request.name(),"inputs":request,
            "readonly":true,"turn":borg_rs::threat_facts::game_turn(),"borg_turn":state.borg.time.now,"original":null,"ported":null});
        let (chosen, replayed, agreement, events, visits) = match mode {
            Mode::Check => borg_rs::decision_tape::compare_read(
                state,
                context,
                |s| request.clone().native(s),
                |s| crate::stateful::read_decide(s, request.clone()),
            ),
            Mode::Switch => borg_rs::decision_tape::compare_read(
                state,
                context,
                |s| crate::stateful::read_decide(s, request.clone()),
                |s| request.clone().native(s),
            ),
            Mode::Off => unreachable!(),
        };
        let mut r = self.recorder.lock().expect("shadow recorder");
        r.counts.comparisons += 1;
        r.counts.readonly_comparisons += 1;
        r.counts.record_events(events as u64, &visits);
        *r.counts.readonly_points.entry(request.name()).or_default() += 1;
        if let Err(error) = agreement {
            r.counts.mismatches += 1;
            let (original, ported) = if mode == Mode::Check {
                (Some(chosen), replayed)
            } else {
                (replayed, Some(chosen))
            };
            let row = serde_json::json!({"schema":DIAGNOSTIC_SCHEMA,"mode":mode,"readonly":true,
                "turn":borg_rs::threat_facts::game_turn(),"borg_turn":state.borg.time.now,
                "point":request.name(),"inputs":request,"original":original,"ported":ported,"replay_error":error,
                "tape_events":events,"policy_markers":visits.values().sum::<u64>(),"dependency_requests":events as u64-visits.values().sum::<u64>()});
            serde_json::to_writer(&mut r.mismatches, &row).unwrap_or_else(|e| failure(e));
            r.mismatches.write_all(b"\n").unwrap_or_else(|e| failure(e));
        }
        chosen
    }
    fn decision(&self, state: &mut BorgState, request: Decision) -> Value {
        if let Decision::HomePower = request {
            return self.read_decision(state, ReadDecision::HomePower);
        }
        if let Decision::StoreMinimumQuantity { item } = &request {
            return self.read_decision(
                state,
                ReadDecision::StoreMinimumQuantity { item: item.clone() },
            );
        }
        // Retain the typed request in constructed callers, but classify the
        // immutable entry before entering the mutable comparison machinery.
        if let Decision::ItemEquipsItem {
            activation,
            check_charge,
        } = request
        {
            return self.read_decision(
                state,
                ReadDecision::ItemEquipsItem {
                    activation,
                    check_charge,
                },
            );
        }

        let mode = self.mode;
        let context = serde_json::json!({"schema":DIAGNOSTIC_SCHEMA,"mode":mode,"point":request.name(),"inputs":request,
            "turn":borg_rs::threat_facts::game_turn(),"borg_turn":state.borg.time.now,
            "original":null,"ported":null});
        let (chosen, replayed, agreement, events, visits) = match mode {
            Mode::Check => borg_rs::decision_tape::compare(
                state,
                context,
                |s| request.clone().native(s),
                |s| crate::stateful::decide(s, request.clone()),
            ),
            Mode::Switch => borg_rs::decision_tape::compare(
                state,
                context,
                |s| crate::stateful::decide(s, request.clone()),
                |s| request.clone().native(s),
            ),
            Mode::Off => return request.native(state),
        };
        let mut r = self.recorder.lock().expect("shadow recorder");
        r.counts.comparisons += 1;
        r.counts.stateful_comparisons += 1;
        r.counts.record_events(events as u64, &visits);
        *r.counts.root_points.entry(request.name()).or_default() += 1;
        if matches!(request, Decision::Caution) && chosen == Value::Acted(true) {
            r.counts.caution_actions += 1;
        }
        if let Err(error) = agreement {
            r.counts.mismatches += 1;
            let (original, ported) = if mode == Mode::Check {
                (Some(chosen), replayed)
            } else {
                (replayed, Some(chosen))
            };
            let row = serde_json::json!({"schema":DIAGNOSTIC_SCHEMA,"mode":mode,
                "turn":borg_rs::threat_facts::game_turn(),"borg_turn":state.borg.time.now,
                "point":request.name(),"inputs":request,"original":original,"ported":ported,
                "replay_error":error,"tape_events":events,"policy_markers":visits.values().sum::<u64>(),
                "dependency_requests":events as u64-visits.values().sum::<u64>()});
            serde_json::to_writer(&mut r.mismatches, &row).unwrap_or_else(|e| failure(e));
            r.mismatches.write_all(b"\n").unwrap_or_else(|e| failure(e));
        }
        chosen
    }

    fn wall_like(&self, turn: i32, borg_turn: i32, feature: u32, original: bool) -> bool {
        self.recorder
            .lock()
            .expect("shadow recorder")
            .compare_wall(self.mode, turn, borg_turn, feature, original)
            .unwrap_or_else(|e| failure(e))
    }

    fn after_think(&self, state: &BorgState) {
        #[cfg(feature = "test-r4-store")]
        crate::store_fixture::run(state, self.mode, self);
        #[cfg(feature = "test-r4-equipment")]
        crate::equipment_fixture::run(state, self.mode, self);
        #[cfg(feature = "test-r4-power")]
        crate::power_fixture::run(state, self.mode, self);
        #[cfg(feature = "test-r4-item-use")]
        crate::item_use_fixture::run(state, self.mode, |s, request| self.decision(s, request));
        #[cfg(feature = "test-r3-defence")]
        crate::defence_fixture::run(state, self.mode);
        #[cfg(feature = "test-m13-danger")]
        crate::m13_fixture::run(state, self.mode);
        self.recorder
            .lock()
            .expect("shadow recorder")
            .think(self.mode, state.borg.time.now)
            .unwrap_or_else(|e| failure(e));
    }
}

/// Invalid remembered memory is a validation failure, never a made-up scalar.
/// Called before taking the recorder mutex, including from nested transactions.
/// It uses no engine query: the game turn is deliberately unknown here.
pub(crate) fn invalid_danger_grid(mode: Mode, view: &View<'_>, x: i32, y: i32) -> ! {
    let row = serde_json::json!({
        "schema": DIAGNOSTIC_SCHEMA,
        "mode": mode,
        "point": "danger.grid", "inputs": {"x":x,"y":y},
        "turn": null, "borg_turn": view.threat.clock.now,
        "original": null, "ported": null, "validation_stopped": true,
        "danger_grid_error": "remembered grid is unallocated or out of bounds",
    });
    let saved = std::env::var_os("BORG_RUN_DIR")
        .ok_or_else(|| std::io::Error::other("missing shadow run directory"))
        .and_then(|dir| {
            let mut file = std::fs::OpenOptions::new()
                .append(true)
                .open(PathBuf::from(dir).join("shadow.jsonl"))?;
            serde_json::to_writer(&mut file, &row)?;
            file.write_all(b"\n")?;
            file.flush()
        });
    if let Err(error) = saved {
        eprintln!("cannot persist invalid danger grid: {error}");
    }
    failure(format!("invalid remembered danger grid ({x}, {y})"))
}

fn failure(error: impl std::fmt::Display) -> ! {
    eprintln!("borg-next: shadow: {error}");
    std::process::exit(2)
}

/// Configure once at core startup. Off creates no files and registers no
/// hooks. Enabled runs need BORG_RUN_DIR (the harness/frontend sets it).
pub fn register() {
    static REGISTERED: OnceLock<()> = OnceLock::new();
    REGISTERED.get_or_init(|| {
        let value = std::env::var("BORG_SHADOW").unwrap_or_else(|_| "off".into());
        let mode = Mode::parse(&value).unwrap_or_else(|e| failure(e));
        if mode == Mode::Off {
            return;
        }
        let dir = PathBuf::from(
            std::env::var_os("BORG_RUN_DIR")
                .unwrap_or_else(|| failure("enabled mode requires BORG_RUN_DIR")),
        );
        let mismatches = File::create(dir.join("shadow.jsonl")).unwrap_or_else(|e| failure(e));
        let stats = File::create(dir.join("shadow-stats.jsonl")).unwrap_or_else(|e| failure(e));
        borg_rs::shadow::register_shadow(Box::new(Next {
            mode,
            recorder: Mutex::new(Recorder::new(mismatches, stats)),
        }));
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn replay_stop_preserves_the_versioned_context_and_flushes_before_exit() {
        const CHILD: &str = "BORG_TEST_REPLAY_SCHEMA_MODE";
        if let Ok(value) = std::env::var(CHILD) {
            let mode = Mode::parse(&value).unwrap();
            let mut state = BorgState::default();
            let _ = borg_rs::decision_tape::compare(
                &mut state,
                serde_json::json!({"schema":DIAGNOSTIC_SCHEMA, "mode":mode,
                    "point":"defence.replay-schema-control", "turn":null}),
                |_| borg_rs::decision_tape::engine("expected", String::new(), || 1_i32),
                |_| borg_rs::decision_tape::engine("changed", String::new(), || 2_i32),
            );
            panic!("divergent replay continued");
        }
        let root = std::env::temp_dir().join(format!("borg-replay-schema-{}", std::process::id()));
        std::fs::create_dir_all(&root).unwrap();
        for mode in ["check", "switch"] {
            let dir = root.join(mode);
            std::fs::create_dir_all(&dir).unwrap();
            std::fs::write(dir.join("shadow.jsonl"), "").unwrap();
            let output = std::process::Command::new(std::env::current_exe().unwrap())
                .args(["--nocapture", "--exact", "shadow::tests::replay_stop_preserves_the_versioned_context_and_flushes_before_exit"])
                .env(CHILD, mode)
                .env("BORG_RUN_DIR", &dir)
                .env("BORG_SHADOW", "changed-after-provider-parse")
                .output().unwrap();
            assert_eq!(output.status.code(), Some(2));
            assert!(String::from_utf8_lossy(&output.stderr).contains("replay stopped"));
            let text = std::fs::read_to_string(dir.join("shadow.jsonl")).unwrap();
            assert_eq!(text.lines().count(), 1);
            let row: serde_json::Value = serde_json::from_str(&text).unwrap();
            assert_eq!(row["schema"], "borg-reborn-rs/shadow@10");
            assert_eq!(row["mode"], mode);
            assert_eq!(row["point"], "defence.replay-schema-control");
            assert_eq!(row["replay_stopped"], true);
            assert!(row["replay_error"]
                .as_str()
                .unwrap()
                .contains("requested changed"));
        }
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn mixed_markers_requests_and_total_events_have_explicit_meanings() {
        let mut counts = Counts::default();
        counts.record_events(
            5,
            &BTreeMap::from([("caution.borg_caution", 1), ("escape.borg_recall", 2)]),
        );
        counts.record_events(4, &BTreeMap::from([("attack.borg_attack", 1)]));
        assert_eq!(
            (
                counts.tape_events,
                counts.policy_markers,
                counts.dependency_requests
            ),
            (9, 4, 5)
        );
        assert_eq!(
            (
                counts.caution_calls,
                counts.escape_calls,
                counts.attack_calls
            ),
            (1, 2, 1)
        );
        assert_eq!(
            counts.policy_markers,
            counts.stateful_points.values().sum::<u64>()
        );
        counts.record_events(3, &BTreeMap::new());
        counts.record_events(2, &BTreeMap::from([("recovery.borg_recover", 2)]));
        counts.record_events(0, &BTreeMap::new());
        assert_eq!(
            (
                counts.tape_events,
                counts.policy_markers,
                counts.dependency_requests
            ),
            (14, 6, 8)
        );
    }

    #[test]
    #[should_panic(expected = "policy markers exceed total tape events")]
    fn impossible_marker_counts_are_rejected() {
        Counts::default().record_events(0, &BTreeMap::from([("attack.borg_attack", 1)]));
    }

    #[test]
    fn pure_attack_comparisons_choose_the_driving_mode_and_log_values() {
        for (mode, chosen) in [(Mode::Check, 3), (Mode::Switch, 4), (Mode::Off, 3)] {
            let mut r = Recorder::new(Vec::new(), Vec::new());
            assert_eq!(
                r.compare_attack_scalar(
                    mode,
                    11,
                    Some(7),
                    "attack.borg_thrust_damage_one",
                    || serde_json::json!({"slot":2}),
                    3,
                    4
                )
                .unwrap(),
                chosen
            );
            if mode == Mode::Off {
                assert!(r.mismatches.is_empty());
                continue;
            }
            let row: serde_json::Value = serde_json::from_slice(&r.mismatches).unwrap();
            assert_eq!(row["schema"], "borg-reborn-rs/shadow@10");
            assert_eq!(row["inputs"]["slot"], 2);
            assert_eq!(row["original"], 3);
            assert_eq!(row["ported"], 4);
            assert_eq!(r.counts.pure_attack_comparisons, 1);
            assert_eq!(r.counts.attack_calls, 1);
            assert_eq!(
                (
                    r.counts.tape_events,
                    r.counts.policy_markers,
                    r.counts.dependency_requests
                ),
                (0, 0, 0)
            );
        }
    }

    #[test]
    fn scalar_disagreement_records_inputs_and_respects_mode() {
        let input = DangerInput {
            y: 9,
            x: 14,
            turns: 2,
            average: true,
            full: false,
        };
        for slot in [None, Some(3)] {
            for (mode, expected) in [(Mode::Check, 91), (Mode::Switch, 92), (Mode::Off, 91)] {
                let mut r = Recorder::new(Vec::new(), Vec::new());
                assert_eq!(
                    r.compare_danger(mode, 1000, 21, input, slot, (91, 92))
                        .unwrap(),
                    expected
                );
                if mode == Mode::Off {
                    assert!(r.mismatches.is_empty());
                    assert_eq!(r.counts.comparisons, 0);
                } else {
                    let row: serde_json::Value = serde_json::from_slice(&r.mismatches).unwrap();
                    assert_eq!(row["inputs"]["turns"], 2);
                    assert_eq!(row["original"], 91);
                    assert_eq!(row["ported"], 92);
                    assert_eq!(row["slot"], serde_json::json!(slot));
                    assert_eq!(r.counts.mismatches, 1);
                }
            }
        }
    }

    #[test]
    fn check_keeps_native_switch_uses_port_and_both_log_actual_mismatch() {
        for (mode, chosen) in [(Mode::Check, false), (Mode::Switch, true)] {
            let mut recorder = Recorder::new(Vec::new(), Vec::new());
            assert_eq!(
                recorder
                    .compare_wall(mode, 850, 91, FEAT_GRANITE, false)
                    .unwrap(),
                chosen
            );
            let row: serde_json::Value = serde_json::from_slice(&recorder.mismatches).unwrap();
            assert_eq!(row["turn"], 850);
            assert_eq!(row["borg_turn"], 91);
            assert_eq!(row["inputs"]["feature"], FEAT_GRANITE);
            assert_eq!(row["original"], false);
            assert_eq!(row["ported"], true);
            recorder.think(mode, 91).unwrap();
            let stats: serde_json::Value = serde_json::from_slice(&recorder.stats).unwrap();
            assert_eq!(stats["counts"]["comparisons"], 1);
            assert_eq!(stats["counts"]["mismatches"], 1);
        }
    }

    #[test]
    fn off_never_compares_or_writes_and_agreement_is_not_a_mismatch() {
        let mut recorder = Recorder::new(Vec::new(), Vec::new());
        assert!(!recorder
            .compare_wall(Mode::Off, 1, 1, FEAT_GRANITE, false)
            .unwrap());
        assert_eq!(recorder.counts.comparisons, 0);
        for feature in [0, 1, 2, 22, 23, 24, u32::MAX] {
            assert!(!recorder
                .compare_wall(Mode::Check, 1, 1, feature, false)
                .unwrap());
        }
        for feature in [
            FEAT_RUBBLE,
            FEAT_MAGMA,
            FEAT_QUARTZ,
            FEAT_MAGMA_K,
            FEAT_QUARTZ_K,
            FEAT_GRANITE,
        ] {
            assert!(recorder
                .compare_wall(Mode::Switch, 1, 1, feature, true)
                .unwrap());
        }
        assert!(recorder.mismatches.is_empty());
        assert_eq!(recorder.counts.mismatches, 0);
    }

    #[test]
    fn invalid_mode_is_rejected() {
        assert_eq!(Mode::parse("check").unwrap(), Mode::Check);
        assert!(Mode::parse("chek").unwrap_err().contains("BORG_SHADOW"));
    }
}
