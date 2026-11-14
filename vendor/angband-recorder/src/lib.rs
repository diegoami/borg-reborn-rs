//! External inputs are replayed; engine decisions and RNG are never supplied.
//! Adapted from better-borg-c and borg-rs; see CREDITS.md.
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::fs::{self, File, OpenOptions};
use std::io::{self, BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

#[cfg(feature = "ffi")]
mod ffi;

pub const SCHEMA: &str = "angband-recorder-streams@1";

/// One terminal poll, including an empty poll. Layout shared with recorder.h.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[repr(C)]
pub struct Input {
    pub wait: i64,
    pub take: i64,
    pub result: i64,
    pub event_type: i64,
    pub code: i64,
    pub mods: i64,
    pub x: i64,
    pub y: i64,
    pub button: i64,
    pub head: i64,
    pub tail: i64,
    pub disconnect: i64,
    pub width: i64,
    pub height: i64,
    pub mapped: i64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Event {
    Exists {
        value: bool,
    },
    Poll {
        input: Input,
    },
    Clock {
        value: i64,
    },
    File {
        present: bool,
        sha256: Option<String>,
    },
    Bot {
        turn: i64,
        count: i64,
        event_type: i64,
        code: u32,
        mods: u8,
    },
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Metadata {
    pub schema: String,
    pub build: String,
    pub pid: i64,
    pub geometry: [i64; 5],
}

fn invalid(message: impl Into<String>) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message.into())
}
fn json_line(writer: &mut File, value: &impl Serialize) -> io::Result<()> {
    serde_json::to_writer(&mut *writer, value)?;
    writer.write_all(b"\n")?;
    writer.flush()
}
fn exclusive(path: &Path) -> io::Result<File> {
    OpenOptions::new().write(true).create_new(true).open(path)
}

/// The session owns streams; separate per-process output prevents replay from
/// overwriting the source. Only complete, fully consumed sessions get a receipt.
pub struct Session {
    pub metadata: Metadata,
    output: PathBuf,
    original: Option<PathBuf>,
    replay: Option<BufReader<File>>,
    events: File,
    legacy: std::collections::BTreeMap<&'static str, File>,
    polls: u64,
    frames: File,
    messages: File,
    count: u64,
    frame_count: u64,
    file_count: u64,
    failed: bool,
    finished: bool,
    #[cfg(feature = "ffi")]
    previous_cells: Vec<(u32, i32)>,
    #[cfg(feature = "ffi")]
    previous_size: (i32, i32),
}

impl Session {
    pub fn start(
        output: &Path,
        original: Option<&Path>,
        mut metadata: Metadata,
    ) -> io::Result<Self> {
        if metadata.schema != SCHEMA {
            return Err(invalid("unsupported recording schema"));
        }
        fs::create_dir_all(output)?;
        // Hosts may already have opened a console log or archive directory.
        // Every recorder-owned file below still uses create_new, so a prior
        // capture can never be overwritten.
        if output.join("recording.json").exists() {
            return Err(invalid("recording output already contains a capture"));
        }
        let replay = if let Some(source) = original {
            let prior: Metadata =
                serde_json::from_reader(File::open(source.join("recording.json"))?)?;
            if prior.schema != SCHEMA
                || prior.build != metadata.build
                || prior.geometry != metadata.geometry
            {
                return Err(invalid(
                    "recording schema, build or terminal geometry differs",
                ));
            }
            if !source.join("complete.json").is_file() {
                return Err(invalid("source recording is incomplete"));
            }
            metadata.pid = prior.pid;
            Some(BufReader::new(File::open(source.join("inputs.jsonl"))?))
        } else {
            None
        };
        if metadata.pid < 1 {
            return Err(invalid("invalid RNG initialization PID"));
        }
        let mut manifest = exclusive(&output.join("recording.json"))?;
        json_line(&mut manifest, &metadata)?;
        let mut legacy = std::collections::BTreeMap::new();
        for name in [
            "events.txt",
            "clock.txt",
            "actions.txt",
            "file-inputs.txt",
            "native-messages.jsonl",
            "combat.jsonl",
        ] {
            legacy.insert(name, exclusive(&output.join(name))?);
        }
        writeln!(exclusive(&output.join("pid.txt"))?, "{}", metadata.pid)?;
        let [width, height, key_size, soft_cursor, complex_input] = metadata.geometry;
        json_line(
            &mut exclusive(&output.join("geometry.json"))?,
            &serde_json::json!({"width":width,"height":height,"key_size":key_size,"soft_cursor":soft_cursor,"complex_input":complex_input}),
        )?;
        Ok(Self {
            legacy,
            polls: 0,
            metadata,
            output: output.into(),
            original: original.map(Path::to_path_buf),
            replay,
            events: exclusive(&output.join("inputs.jsonl"))?,
            frames: exclusive(&output.join("frames.jsonl"))?,
            messages: exclusive(&output.join("messages.jsonl"))?,
            count: 0,
            frame_count: 0,
            file_count: 0,
            failed: false,
            finished: false,
            #[cfg(feature = "ffi")]
            previous_cells: Vec::new(),
            #[cfg(feature = "ffi")]
            previous_size: (0, 0),
        })
    }
    pub fn is_replay(&self) -> bool {
        self.replay.is_some()
    }
    fn expected(&mut self) -> io::Result<Option<Event>> {
        if self.failed || self.finished {
            return Err(invalid("recording is closed or failed"));
        }
        let result = (|| {
            if let Some(reader) = &mut self.replay {
                let mut line = String::new();
                if reader.read_line(&mut line)? == 0 {
                    return Err(invalid("replay input exhausted"));
                }
                return Ok(Some(serde_json::from_str(&line)?));
            }
            Ok(None)
        })();
        if result.is_err() {
            self.failed = true;
        }
        result
    }
    fn write_event(&mut self, event: &Event) -> io::Result<()> {
        if self.finished || self.failed {
            return Err(invalid("recording is closed or failed"));
        }
        json_line(&mut self.events, event)?;
        self.count += 1;
        Ok(())
    }
    pub fn poll(&mut self, actual: Input) -> io::Result<Input> {
        let event = self.expected()?;
        let input = match event {
            None => actual,
            Some(Event::Poll { input })
                if input.wait == actual.wait && input.take == actual.take =>
            {
                input
            }
            _ => {
                self.failed = true;
                return Err(invalid("terminal call diverged"));
            }
        };
        if input.width < 1
            || input.height < 1
            || input.width > 255
            || input.height > 255
            || input.head < 0
            || input.tail < 0
        {
            self.failed = true;
            return Err(invalid("invalid terminal geometry or queue"));
        }
        if std::env::var_os("BORG_TOOLKIT_HEADLESS").is_some() && input != actual {
            self.failed = true;
            return Err(invalid("recomputed frontend poll diverged"));
        }
        let i = input;
        let stream = self.legacy.get_mut("events.txt").unwrap();
        writeln!(
            stream,
            "{} {} {} {} {} {} {} {} {} {} {} {} {} {} {}",
            i.wait,
            i.take,
            i.result,
            i.event_type,
            i.code,
            i.mods,
            i.x,
            i.y,
            i.button,
            i.head,
            i.tail,
            i.disconnect,
            i.width,
            i.height,
            i.mapped
        )?;
        stream.flush()?;
        self.polls += 1;
        self.write_event(&Event::Poll { input })?;
        Ok(input)
    }
    pub fn exists(&mut self, actual: bool) -> io::Result<bool> {
        let value = match self.expected()? {
            None => actual,
            Some(Event::Exists { value }) => value,
            _ => {
                self.failed = true;
                return Err(invalid("file existence call diverged"));
            }
        };
        self.write_event(&Event::Exists { value })?;
        Ok(value)
    }
    pub fn clock(&mut self, actual: i64) -> io::Result<i64> {
        let value = match self.expected()? {
            None => actual,
            Some(Event::Clock { value }) => value,
            _ => {
                self.failed = true;
                return Err(invalid("clock call diverged"));
            }
        };
        writeln!(self.legacy.get_mut("clock.txt").unwrap(), "{value}")?;
        self.legacy.get_mut("clock.txt").unwrap().flush()?;
        self.write_event(&Event::Clock { value })?;
        Ok(value)
    }
    pub fn bot(
        &mut self,
        turn: i64,
        count: i64,
        event_type: i64,
        code: u32,
        mods: u8,
    ) -> io::Result<()> {
        let event = Event::Bot {
            turn,
            count,
            event_type,
            code,
            mods,
        };
        if let Some(expected) = self.expected()? {
            if expected != event {
                self.failed = true;
                return Err(invalid("bot action diverged"));
            }
        }
        writeln!(
            self.legacy.get_mut("actions.txt").unwrap(),
            "{turn} {count} {event_type} {code} {mods}"
        )?;
        self.legacy.get_mut("actions.txt").unwrap().flush()?;
        self.write_event(&event)
    }
    /// Capture at read time and return the captured bytes for the engine to open.
    pub fn read_path(&mut self, path: &Path) -> io::Result<PathBuf> {
        self.file_count += 1;
        let name = format!("file-input-{:06}.bin", self.file_count);
        let expected = self.expected()?;
        let source = self
            .original
            .as_ref()
            .map_or_else(|| path.to_path_buf(), |p| p.join(&name));
        let bytes = match (&expected, fs::read(&source)) {
            (
                Some(Event::File {
                    present: false,
                    sha256: None,
                }),
                _,
            ) => None,
            (Some(Event::File { present: true, .. }) | None, Ok(bytes)) => Some(bytes),
            (None, Err(e))
                if matches!(
                    e.kind(),
                    io::ErrorKind::NotFound | io::ErrorKind::PermissionDenied
                ) =>
            {
                None
            }
            (_, Err(e)) => return Err(e),
            _ => {
                self.failed = true;
                return Err(invalid("file read call diverged"));
            }
        };
        let digest = bytes.as_ref().map(|b| format!("{:x}", Sha256::digest(b)));
        let event = Event::File {
            present: bytes.is_some(),
            sha256: digest,
        };
        if let Some(expected) = expected {
            if event != expected {
                self.failed = true;
                return Err(invalid("captured file differs"));
            }
        }
        writeln!(
            self.legacy.get_mut("file-inputs.txt").unwrap(),
            "{}",
            i32::from(bytes.is_some())
        )?;
        self.legacy.get_mut("file-inputs.txt").unwrap().flush()?;
        self.write_event(&event)?;
        let target = self.output.join(name);
        if let Some(bytes) = bytes {
            exclusive(&target)?.write_all(&bytes)?;
        }
        Ok(target)
    }
    pub fn frame(&mut self, frame: &Value) -> io::Result<()> {
        json_line(&mut self.frames, frame)?;
        self.frame_count += 1;
        Ok(())
    }
    pub fn combat(&mut self, event: &Value) -> io::Result<()> {
        json_line(self.legacy.get_mut("combat.jsonl").unwrap(), event)
    }
    pub fn message(&mut self, message: &Value) -> io::Result<()> {
        json_line(
            self.legacy.get_mut("native-messages.jsonl").unwrap(),
            message,
        )?;
        json_line(&mut self.messages, message)
    }
    pub fn finish(&mut self) -> io::Result<()> {
        if self.finished {
            return Ok(());
        }
        if self.failed {
            return Err(invalid("recording has diverged"));
        }
        if let Some(reader) = &mut self.replay {
            let mut remaining = String::new();
            if reader.read_line(&mut remaining)? != 0 {
                self.failed = true;
                return Err(invalid("unconsumed replay input"));
            }
        }
        for stream in [&mut self.events, &mut self.frames, &mut self.messages] {
            stream.flush()?;
            stream.sync_all()?;
        }
        for stream in self.legacy.values_mut() {
            stream.flush()?;
            stream.sync_all()?;
        }
        json_line(
            &mut exclusive(&self.output.join("record-complete.json"))?,
            &serde_json::json!({"inputs":self.polls,"frames":self.frame_count}),
        )?;
        json_line(
            &mut exclusive(&self.output.join("complete.json"))?,
            &serde_json::json!({"inputs":self.count,"frames":self.frame_count}),
        )?;
        self.finished = true;
        Ok(())
    }
}

/// Byte comparison includes all observations, captured reads and the footer.
/// Re-execution must have completed before invoking this function.
pub fn verify(original: &Path, replay: &Path) -> io::Result<()> {
    if !original.join("complete.json").is_file() || !replay.join("complete.json").is_file() {
        return Err(invalid("missing completion receipt"));
    }
    for item in fs::read_dir(original)? {
        let item = item?;
        let name = item.file_name();
        let name_str = name.to_string_lossy();
        let compared = matches!(
            name_str.as_ref(),
            "recording.json"
                | "inputs.jsonl"
                | "frames.jsonl"
                | "messages.jsonl"
                | "complete.json"
                | "events.txt"
                | "clock.txt"
                | "pid.txt"
                | "actions.txt"
                | "file-inputs.txt"
                | "geometry.json"
                | "native-messages.jsonl"
                | "combat.jsonl"
                | "record-complete.json"
        ) || name_str.starts_with("file-input-");
        if compared && fs::read(item.path())? != fs::read(replay.join(&name))? {
            return Err(invalid(format!("replay differs: {name_str}")));
        }
    }
    Ok(())
}
