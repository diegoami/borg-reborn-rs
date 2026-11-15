//! Comparison harness for complete danger assessment and the wall helper.
//! Caution remains native; no native function is run twice.
use crate::View;
use borg_rs::decisions::{Decision, Value};
use borg_rs::memory::terrain::*;
use borg_rs::memory::BorgState;
use borg_rs::shadow::{DangerInput, Shadow};
use serde::Serialize;
use std::collections::BTreeMap;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

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
    stateful_comparisons: u64,
    dependency_requests: u64,
    root_points: BTreeMap<&'static str, u64>,
    stateful_points: BTreeMap<&'static str, u64>,
    caution_actions: u64,
    comparisons: u64,
    mismatches: u64,
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
                "schema": "borg-reborn-rs/shadow@1", "mode": mode,
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
                "schema":"borg-reborn-rs/shadow@1", "mode":mode,
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

    fn think(&mut self, mode: Mode, borg_turn: i32) -> std::io::Result<()> {
        self.counts.thinks += 1;
        let row = serde_json::json!({
            "schema": "borg-reborn-rs/shadow-stats@1", "mode": mode,
            "borg_turn": borg_turn,
            "ported_points": ["danger.wall_like", "danger", "danger.one_monster", "caution", "escape", "recovery"],
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
    fn danger(&self, state: &BorgState, input: DangerInput, original: i32) -> i32 {
        let view = View::new(state);
        let ported = crate::danger::assess(&view, input);
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
        let ported = crate::danger::assess_monster(&view, input, slot);
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

    fn decision(&self, state: &mut BorgState, request: Decision) -> Value {
        let mode = self.mode;
        let context = serde_json::json!({"mode":mode,"point":request.name(),"inputs":request,
            "turn":borg_rs::threat_facts::game_turn(),"borg_turn":state.borg.time.now,
            "original":null,"ported":null});
        let (chosen, replayed, agreement, events, visits) = match mode {
            Mode::Check => borg_rs::decision_tape::compare(
                state,
                context,
                |s| request.native(s),
                |s| crate::stateful::decide(s, request),
            ),
            Mode::Switch => borg_rs::decision_tape::compare(
                state,
                context,
                |s| crate::stateful::decide(s, request),
                |s| request.native(s),
            ),
            Mode::Off => return request.native(state),
        };
        let mut r = self.recorder.lock().expect("shadow recorder");
        r.counts.comparisons += 1;
        r.counts.stateful_comparisons += 1;
        r.counts.dependency_requests += events as u64;
        *r.counts.root_points.entry(request.name()).or_default() += 1;
        for (name, count) in visits {
            *r.counts.stateful_points.entry(name).or_default() += count;
            if name.starts_with("caution.") {
                r.counts.caution_calls += count;
            } else if name.starts_with("recovery.") {
                r.counts.recovery_calls += count;
            } else {
                r.counts.escape_calls += count;
            }
        }
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
            let row = serde_json::json!({"schema":"borg-reborn-rs/shadow@1","mode":mode,
                "turn":borg_rs::threat_facts::game_turn(),"borg_turn":state.borg.time.now,
                "point":request.name(),"inputs":request,"original":original,"ported":ported,
                "replay_error":error,"dependency_requests":events});
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
        self.recorder
            .lock()
            .expect("shadow recorder")
            .think(self.mode, state.borg.time.now)
            .unwrap_or_else(|e| failure(e));
    }
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
