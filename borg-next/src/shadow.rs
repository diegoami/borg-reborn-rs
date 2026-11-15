//! R1.2's comparison harness. Only `danger.wall_like` is ported here.
//! Danger and caution boundaries are observed but retain native decisions;
//! their stateful ports belong to R2. No native function is run twice.
use crate::View;
use borg_rs::memory::terrain::*;
use borg_rs::memory::BorgState;
use borg_rs::shadow::{CautionResult, DangerInput, Shadow};
use serde::Serialize;
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
    caution_calls: u64,
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

    fn think(&mut self, mode: Mode, borg_turn: i32) -> std::io::Result<()> {
        self.counts.thinks += 1;
        let row = serde_json::json!({
            "schema": "borg-reborn-rs/shadow-stats@1", "mode": mode,
            "borg_turn": borg_turn,
            "ported_points": ["danger.wall_like"],
            "unported_points": ["danger", "caution"],
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
    fn danger(&self, state: &BorgState, _input: DangerInput, original: i32) -> i32 {
        // Fresh native-memory view at the boundary. No full danger port yet.
        let _view = View::new(state);
        self.recorder
            .lock()
            .expect("shadow recorder")
            .counts
            .danger_calls += 1;
        original
    }

    fn before_caution(&self, state: &BorgState) {
        // This is before the native decision mutates goals or queues keys.
        let _view = View::new(state);
        self.recorder
            .lock()
            .expect("shadow recorder")
            .counts
            .caution_calls += 1;
    }

    fn caution(&self, _state: &BorgState, original: &CautionResult) {
        if original.acted {
            self.recorder
                .lock()
                .expect("shadow recorder")
                .counts
                .caution_actions += 1;
        }
        // No caution port: its state, acted result and queue remain native.
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
