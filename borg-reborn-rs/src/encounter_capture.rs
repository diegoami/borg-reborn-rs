//! Optional native command-boundary evidence, independent of the decision core.
//!
//! Reads use L1's player-visible interface and a private diagnostic RNG. They
//! never advance either controller's RNG or the saved engine RNG. This is a
//! capture interface for Cemetery preparation, not a policy or a death filter.

use crate::{engine, observation::Observation};
use serde::Serialize;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

struct Capture {
    encounters: Option<BufWriter<File>>,
    observations: Option<BufWriter<File>>,
    knowledge: Option<PathBuf>,
    first: bool,
    rng: u32,
}

fn open(path: Option<std::ffi::OsString>) -> Option<BufWriter<File>> {
    path.map(|path| {
        BufWriter::new(File::create(path).expect("native encounter capture: cannot create output"))
    })
}

impl Capture {
    fn from_environment() -> Option<Self> {
        let encounters = std::env::var_os("BORG_ENCOUNTER_LOG");
        let observations = std::env::var_os("BORG_OBSERVATION_LOG");
        let knowledge = std::env::var_os("BORG_KNOWLEDGE_LOG");
        let own_knowledge = std::env::var_os("BORG_ENCOUNTER_KNOWLEDGE_LOG");
        if encounters.is_none()
            && observations.is_none()
            && knowledge.is_none()
            && own_knowledge.is_none()
        {
            return None;
        }
        // Preserve the original controller's per-think observation/world replay
        // contract. Its legacy writers remain the only owners of those files.
        let legacy =
            (observations.is_some() || knowledge.is_some()) && crate::borg::core_is_borg_rs();
        let observations = if legacy { observations } else { None };
        let knowledge = own_knowledge.or(if legacy { knowledge } else { None });
        if encounters.is_none() && observations.is_none() && knowledge.is_none() {
            return None;
        }
        Some(Self {
            encounters: open(encounters),
            observations: open(observations),
            knowledge: knowledge.map(PathBuf::from),
            first: true,
            rng: 0x43415054,
        })
    }

    fn record(&mut self) {
        // A knowledge-only request has no work after its single native dump.
        if !self.first && self.encounters.is_none() && self.observations.is_none() {
            return;
        }
        if !self.first && self.observations.is_none() {
            if let Some(frame) = engine::with_borg_rng(&mut self.rng, engine::perceive_encounter) {
                if let Some(log) = self.encounters.as_mut() {
                    writeln!(
                        log,
                        "{}",
                        serde_json::to_string(&frame).expect("native encounter serialization")
                    )
                    .and_then(|_| log.flush())
                    .expect("native encounter capture: cannot write encounter frame");
                }
            }
            return;
        }
        let observation = engine::with_borg_rng(&mut self.rng, engine::perceive);
        let Some(observation) = observation else {
            return;
        };
        if let Some(path) = self.knowledge.take() {
            let knowledge = engine::with_borg_rng(&mut self.rng, engine::knowledge)
                .expect("native encounter capture: missing knowledge");
            std::fs::write(path, knowledge.to_line() + "\n")
                .expect("native encounter capture: cannot write knowledge");
        }
        if let Some(log) = self.encounters.as_mut() {
            // The initial native frame retains exact observable gear and status.
            // Later frames keep only the incident miner's observable channels.
            let line = if self.first {
                observation.to_line()
            } else {
                compact(&observation)
            };
            writeln!(log, "{line}")
                .and_then(|_| log.flush())
                .expect("native encounter capture: cannot write encounter frame");
        }
        if let Some(log) = self.observations.as_mut() {
            writeln!(log, "{}", observation.to_line())
                .and_then(|_| log.flush())
                .expect("native encounter capture: cannot write observation frame");
        }
        self.first = false;
    }
}

#[derive(Serialize)]
pub(crate) struct Frame {
    pub turn: i64,
    pub status: Status,
    pub monsters: Vec<Monster>,
}

#[derive(Serialize)]
pub(crate) struct Status {
    pub hp: i32,
    pub max_hp: i32,
    pub sp: i32,
    pub max_sp: i32,
    pub level: i32,
    pub depth: i32,
}

#[derive(Serialize)]
pub(crate) struct Monster {
    pub index: i32,
    pub race: u32,
}

fn compact(observation: &Observation) -> String {
    let s = &observation.status;
    serde_json::to_string(&Frame {
        turn: observation.turn,
        status: Status {
            hp: s.hp,
            max_hp: s.max_hp,
            sp: s.sp,
            max_sp: s.max_sp,
            level: s.level,
            depth: s.depth,
        },
        monsters: observation
            .monsters
            .iter()
            .map(|m| Monster {
                index: m.index,
                race: m.race,
            })
            .collect(),
    })
    .expect("native encounter capture: cannot serialize frame")
}

pub(crate) fn record() {
    static CAPTURE: OnceLock<Mutex<Option<Capture>>> = OnceLock::new();
    let capture = CAPTURE.get_or_init(|| Mutex::new(Capture::from_environment()));
    if let Some(capture) = capture.lock().unwrap_or_else(|e| e.into_inner()).as_mut() {
        capture.record();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compact_frames_keep_living_zero_hp_and_visible_monsters() {
        let observation = Observation {
            turn: 123,
            status: crate::observation::Status {
                hp: 0,
                max_hp: 8,
                level: 1,
                ..Default::default()
            },
            ..Default::default()
        };
        let frame: serde_json::Value = serde_json::from_str(&compact(&observation)).unwrap();
        assert_eq!(frame["turn"], 123);
        assert_eq!(frame["status"]["hp"], 0);
        assert_eq!(frame["status"]["level"], 1);
        assert_eq!(frame["monsters"], serde_json::json!([]));
        assert!(frame.get("panel").is_none());
    }
}
