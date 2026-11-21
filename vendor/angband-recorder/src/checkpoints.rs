//! Optional native save retention. A save starts a fresh process; it is not a
//! serialized recorder cursor or a snapshot of the bot's process memory.
use crate::{exclusive, invalid, read_regular, visit_json_lines};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub recent: u64,
    pub every_turns: i64,
}
impl Config {
    pub fn parse(recent: Option<&str>, every: Option<&str>) -> io::Result<Option<Self>> {
        if recent.is_none() && every.is_none() {
            return Ok(None);
        }
        let recent = recent
            .unwrap_or("128")
            .parse::<u64>()
            .map_err(|_| invalid("ANGBAND_SAVE_WINDOW must be an integer 0..10000"))?;
        let every_turns = every
            .unwrap_or("1000")
            .parse::<i64>()
            .map_err(|_| invalid("ANGBAND_SAVE_EVERY must be an integer 0..1000000000"))?;
        if recent > 10000 || !(0..=1_000_000_000).contains(&every_turns) {
            return Err(invalid("native save retention setting out of range"));
        }
        Ok((recent != 0 || every_turns != 0).then_some(Self {
            recent,
            every_turns,
        }))
    }
    #[cfg(feature = "ffi")]
    pub(crate) fn environment() -> io::Result<Option<Self>> {
        let value = |key| match std::env::var(key) {
            Ok(value) => Ok(Some(value)),
            Err(std::env::VarError::NotPresent) => Ok(None),
            Err(_) => Err(invalid(format!("{key}: non-UTF8 save retention setting"))),
        };
        Self::parse(
            value("ANGBAND_SAVE_WINDOW")?.as_deref(),
            value("ANGBAND_SAVE_EVERY")?.as_deref(),
        )
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SavePoint {
    pub sequence: u64,
    pub frame: u64,
    pub turn: i64,
    pub input: u64,
    pub file: String,
    pub sha256: String,
    pub world: String,
    pub periodic: bool,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Index {
    schema: String,
    config: Config,
    native_save_only: bool,
    saves: Vec<SavePoint>,
}

pub struct RollingSaves {
    root: PathBuf,
    index: Index,
    sequence: u64,
    last_periodic: Option<i64>,
}
impl RollingSaves {
    pub fn new(recording: &Path, config: Config) -> io::Result<Self> {
        let root = recording.join("checkpoints");
        fs::create_dir(&root)?;
        let value = Self {
            root,
            index: Index {
                schema: "angband-native-save-points@1".into(),
                config,
                native_save_only: true,
                saves: Vec::new(),
            },
            sequence: 0,
            last_periodic: None,
        };
        serde_json::to_writer_pretty(exclusive(&value.root.join("index.json"))?, &value.index)?;
        Ok(value)
    }
    /// The host already forked this save to fingerprint the command frame.
    /// Retain those exact bytes without another save or any live-game changes.
    pub fn capture(&mut self, frame: &Value, source: &Path) -> io::Result<()> {
        if frame["phase"] != "command" || frame["dead"].as_i64() != Some(0) {
            return Ok(());
        }
        let number = frame["index"]
            .as_u64()
            .ok_or_else(|| invalid("save point lacks frame"))?;
        let turn = frame["turn"]
            .as_i64()
            .ok_or_else(|| invalid("save point lacks turn"))?;
        let input = frame["input"]
            .as_u64()
            .ok_or_else(|| invalid("save point lacks input"))?;
        let periodic = self.index.config.every_turns != 0
            && self.last_periodic.is_none_or(|last| {
                turn < last
                    || turn / self.index.config.every_turns > last / self.index.config.every_turns
            });
        if !periodic && self.index.config.recent == 0 {
            return Ok(());
        }
        if self.index.saves.last().is_some_and(|s| s.frame >= number) {
            return Err(invalid("save point frame order regressed"));
        }
        let bytes = read_regular(source)?;
        let world = fingerprint(&bytes);
        if frame["world"].as_str() != Some(&world) {
            return Err(invalid(
                "native save differs from its observed command frame",
            ));
        }
        let file = format!("frame-{number:012}-turn-{turn:012}.save");
        let mut output = exclusive(&self.root.join(&file))?;
        output.write_all(&bytes)?;
        output.sync_all()?;
        self.sequence += 1;
        self.index.saves.push(SavePoint {
            sequence: self.sequence,
            frame: number,
            turn,
            input,
            file,
            sha256: format!("{:x}", Sha256::digest(&bytes)),
            world,
            periodic,
        });
        if periodic {
            self.last_periodic = Some(turn);
        }
        let mut expired = Vec::new();
        self.index.saves.retain(|s| {
            let keep = s.periodic || self.sequence - s.sequence < self.index.config.recent;
            if !keep {
                expired.push(s.file.clone());
            }
            keep
        });
        // Publish the new index before pruning old saves: a crash can leave an
        // extra file, but never an index referring to an already deleted save.
        let temporary = self.root.join(format!(".index-{number}.tmp"));
        let mut index = exclusive(&temporary)?;
        serde_json::to_writer_pretty(&mut index, &self.index)?;
        index.write_all(b"\n")?;
        index.sync_all()?;
        fs::rename(&temporary, self.root.join("index.json"))?;
        for name in expired {
            fs::remove_file(self.root.join(name))?;
        }
        Ok(())
    }
}
fn fingerprint(bytes: &[u8]) -> String {
    format!(
        "{:016x}",
        bytes
            .iter()
            .fold(14695981039346656037u64, |h, b| (h ^ u64::from(*b))
                .wrapping_mul(1099511628211))
    )
}

/// Verify retained bytes and their links to the recorded command frames.
pub fn validate(recording: &Path) -> io::Result<()> {
    let root = recording.join("checkpoints");
    match fs::symlink_metadata(&root) {
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(()),
        Err(e) => return Err(e),
        Ok(m) if !m.file_type().is_dir() => {
            return Err(invalid("checkpoint root is not a real directory"))
        }
        _ => {}
    }
    let index: Index = serde_json::from_slice(&read_regular(&root.join("index.json"))?)?;
    if index.schema != "angband-native-save-points@1" || !index.native_save_only {
        return Err(invalid("unsupported native save index"));
    }
    let mut wanted = std::collections::BTreeMap::new();
    let mut expected = std::collections::BTreeSet::from(["index.json".to_string()]);
    for s in &index.saves {
        if s.file != format!("frame-{:012}-turn-{:012}.save", s.frame, s.turn)
            || wanted.insert(s.frame, s).is_some()
            || !expected.insert(s.file.clone())
        {
            return Err(invalid("invalid/duplicate native save entry"));
        }
        let bytes = read_regular(&root.join(&s.file))?;
        if s.sha256 != format!("{:x}", Sha256::digest(&bytes)) || s.world != fingerprint(&bytes) {
            return Err(invalid("native save checksum mismatch"));
        }
    }
    let actual = fs::read_dir(&root)?
        .map(|p| p.map(|p| p.file_name().to_string_lossy().into_owned()))
        .collect::<io::Result<std::collections::BTreeSet<_>>>()?;
    if actual != expected {
        return Err(invalid("checkpoint index/file inventory differs"));
    }
    visit_json_lines(&recording.join("frames.jsonl"), |frame| {
        if let Some(s) = frame["index"].as_u64().and_then(|n| wanted.remove(&n)) {
            if frame["phase"] != "command"
                || frame["dead"] != 0
                || frame["turn"] != s.turn
                || frame["input"] != s.input
                || frame["world"] != s.world
            {
                return Err(invalid("checkpoint does not match its command frame"));
            }
        }
        Ok(())
    })?;
    if !wanted.is_empty() {
        return Err(invalid("native save references a missing frame"));
    }
    Ok(())
}
