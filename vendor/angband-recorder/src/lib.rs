//! External inputs are replayed; engine decisions and RNG are never supplied.
//! Adapted from better-borg-c and borg-rs; see CREDITS.md.
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::fs::{self, File, OpenOptions};
use std::io::{self, BufRead, BufReader, Read, Write};
use std::path::{Path, PathBuf};

#[cfg(feature = "ffi")]
use std::io::{Seek, SeekFrom};
mod environment;
#[cfg(feature = "ffi")]
mod ffi;

pub const SCHEMA: &str = "angband-recorder-streams@2";

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
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Event {
    Newer {
        value: bool,
    },
    DirectoryOpen {
        present: bool,
    },
    DirectoryRead {
        only_files: bool,
        capacity: usize,
        name: Option<Vec<u8>>,
    },
    TerminalInit {
        geometry: [i64; 5],
    },
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
#[serde(deny_unknown_fields)]
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

const REQUIRED: &[&str] = &[
    "environment.json",
    "recording.json",
    "inputs.jsonl",
    "frames.jsonl",
    "messages.jsonl",
    "complete.json",
    "events.txt",
    "clock.txt",
    "pid.txt",
    "actions.txt",
    "file-inputs.txt",
    "geometry.json",
    "native-messages.jsonl",
    "combat.jsonl",
    "record-complete.json",
];

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Receipt {
    inputs: u64,
    frames: u64,
}

fn validate_geometry(g: [i64; 5]) -> io::Result<()> {
    if !(1..=255).contains(&g[0])
        || !(1..=255).contains(&g[1])
        || !(1..=65535).contains(&g[2])
        || ![0, 1].contains(&g[3])
        || ![0, 1].contains(&g[4])
    {
        return Err(invalid("invalid initial terminal geometry/queue/flags"));
    }
    Ok(())
}
fn valid_event_type(value: i64) -> bool {
    matches!(value, 0 | 1 | 2 | 4 | 8 | 16 | 32 | 64 | 128 | 256)
}
fn validate_poll(i: Input, key_size: i64) -> io::Result<()> {
    if ![0, 1].contains(&i.wait)
        || ![0, 1].contains(&i.take)
        || ![0, 1].contains(&i.result)
        || ![0, 1].contains(&i.disconnect)
        || ![0, 1].contains(&i.mapped)
        || !valid_event_type(i.event_type)
        || !(0..=u32::MAX as i64).contains(&i.code)
        || !(0..=31).contains(&i.mods)
        || !(0..=255).contains(&i.x)
        || !(0..=255).contains(&i.y)
        || !(0..=255).contains(&i.button)
        || !(0..key_size).contains(&i.head)
        || !(0..key_size).contains(&i.tail)
        || !(1..=255).contains(&i.width)
        || !(1..=255).contains(&i.height)
        || (i.event_type == 2 && i.code != 0)
        || (i.event_type != 2 && (i.x != 0 || i.y != 0 || i.button != 0))
        || (i.result == 1 && (i.event_type != 0 || i.code != 0 || i.mods != 0))
    {
        return Err(invalid("terminal poll outside native field domains"));
    }
    Ok(())
}
fn validate_directory_entry(capacity: usize, name: &Option<Vec<u8>>) -> io::Result<()> {
    if capacity == 0
        || capacity > 65536
        || name
            .as_ref()
            .is_some_and(|n| n.len() >= capacity || n.contains(&0))
    {
        return Err(invalid("invalid directory entry/capacity"));
    }
    Ok(())
}
fn regular_file(path: &Path) -> io::Result<File> {
    if !fs::symlink_metadata(path)?.file_type().is_file() {
        return Err(invalid(format!(
            "not a regular recording file: {}",
            path.display()
        )));
    }
    File::open(path)
}
fn read_regular(path: &Path) -> io::Result<Vec<u8>> {
    let mut bytes = Vec::new();
    regular_file(path)?.read_to_end(&mut bytes)?;
    Ok(bytes)
}
fn visit_json_lines(
    path: &Path,
    mut visit: impl FnMut(Value) -> io::Result<()>,
) -> io::Result<u64> {
    let mut reader = BufReader::new(regular_file(path)?);
    let mut count = 0;
    loop {
        let mut line = Vec::new();
        let size = Read::take(&mut reader, 8 * 1024 * 1024 + 1).read_until(b'\n', &mut line)?;
        if size == 0 {
            return Ok(count);
        }
        if size > 8 * 1024 * 1024 || line.last() != Some(&b'\n') {
            return Err(invalid("truncated or oversized JSONL record"));
        }
        let value: Value = serde_json::from_slice(&line)?;
        if !value.is_object() {
            return Err(invalid("observation/event must be a JSON object"));
        }
        visit(value)?;
        count += 1;
    }
}
fn receipt_new(path: &Path, value: &Receipt) -> io::Result<()> {
    use std::sync::atomic::{AtomicU64, Ordering};
    static NEXT: AtomicU64 = AtomicU64::new(0);
    let (temporary, mut file) = loop {
        let candidate = path.with_file_name(format!(
            ".receipt-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        match exclusive(&candidate) {
            Ok(file) => break (candidate, file),
            Err(e) if e.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(e) => return Err(e),
        }
    };
    let result = (|| {
        json_line(&mut file, value)?;
        file.sync_all()?;
        fs::hard_link(&temporary, path)
    })();
    let _ = fs::remove_file(temporary);
    result
}

fn recording_files(root: &Path) -> io::Result<std::collections::BTreeSet<String>> {
    let mut result = std::collections::BTreeSet::new();
    for item in fs::read_dir(root)? {
        let item = item?;
        let name = item.file_name().to_string_lossy().into_owned();
        if REQUIRED.contains(&name.as_str())
            || (name.starts_with("file-input-") || name == "timezone.tzif")
        {
            result.insert(name);
        }
    }
    Ok(result)
}
/// Validate complete source structure and counters before replay or success.
pub fn validate_recording(root: &Path) -> io::Result<Metadata> {
    for name in REQUIRED {
        regular_file(&root.join(name))?;
    }
    let meta: Metadata = serde_json::from_slice(&read_regular(&root.join("recording.json"))?)?;
    validate_geometry(meta.geometry)?;
    if meta.schema != SCHEMA || meta.pid < 1 || meta.pid > i32::MAX as i64 || meta.build.is_empty()
    {
        return Err(invalid("invalid recording identity"));
    }
    let receipt: Receipt = serde_json::from_slice(&read_regular(&root.join("complete.json"))?)?;
    let legacy: Receipt =
        serde_json::from_slice(&read_regular(&root.join("record-complete.json"))?)?;
    let frame_count = visit_json_lines(&root.join("frames.jsonl"), |_| Ok(()))?;
    visit_json_lines(&root.join("messages.jsonl"), |_| Ok(()))?;
    visit_json_lines(&root.join("combat.jsonl"), |_| Ok(()))?;
    if receipt.frames != frame_count || legacy.frames != receipt.frames {
        return Err(invalid("completion frame counters differ"));
    }
    if read_regular(&root.join("messages.jsonl"))?
        != read_regular(&root.join("native-messages.jsonl"))?
    {
        return Err(invalid("message aliases differ"));
    }
    let mut polls = String::new();
    let mut clocks = String::new();
    let mut actions = String::new();
    let mut files = String::new();
    let environment = environment::Environment::load(root)?;
    let mut expected: std::collections::BTreeSet<String> =
        REQUIRED.iter().map(|s| s.to_string()).collect();
    if environment.has_timezone_file() {
        expected.insert("timezone.tzif".into());
    }
    let mut file_count = 0;
    let mut poll_count = 0;
    let event_count = visit_json_lines(&root.join("inputs.jsonl"), |value| {
        let event: Event = serde_json::from_value(value)?;
        match event {
            Event::Newer { .. } | Event::DirectoryOpen { .. } => {}
            Event::DirectoryRead { capacity, name, .. } => {
                validate_directory_entry(capacity, &name)?;
            }
            Event::TerminalInit { geometry } => {
                validate_geometry(geometry)?;
                if geometry != meta.geometry {
                    return Err(invalid("terminal startup metadata differs"));
                }
            }
            Event::Poll { input: i } => {
                validate_poll(i, meta.geometry[2])?;
                poll_count += 1;
                polls += &format!(
                    "{} {} {} {} {} {} {} {} {} {} {} {} {} {} {}\n",
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
                );
            }
            Event::Clock { value } => clocks += &format!("{value}\n"),
            Event::Exists { .. } => {}
            Event::Bot {
                turn,
                count,
                event_type: t,
                code,
                mods,
            } => {
                if !valid_event_type(t) || mods > 31 {
                    return Err(invalid("invalid bot key fields"));
                }
                actions += &format!("{turn} {count} {t} {code} {mods}\n");
            }
            Event::File { present, sha256 } => {
                file_count += 1;
                files += if present { "1\n" } else { "0\n" };
                let name = format!("file-input-{file_count:06}.bin");
                let path = root.join(&name);
                if present {
                    let bytes = read_regular(&path)?;
                    if sha256.as_deref() != Some(format!("{:x}", Sha256::digest(bytes)).as_str()) {
                        return Err(invalid("captured file digest differs"));
                    }
                    expected.insert(name);
                } else if sha256.is_some() || fs::symlink_metadata(path).is_ok() {
                    return Err(invalid("absent input has a blob or digest"));
                }
            }
        }
        Ok(())
    })?;
    if receipt.inputs != event_count {
        return Err(invalid("completion input counter differs"));
    }
    if legacy.inputs != poll_count {
        return Err(invalid("legacy poll count differs"));
    }
    for (name, text) in [
        ("events.txt", polls),
        ("clock.txt", clocks),
        ("actions.txt", actions),
        ("file-inputs.txt", files),
        ("pid.txt", format!("{}\n", meta.pid)),
    ] {
        if read_regular(&root.join(name))? != text.as_bytes() {
            return Err(invalid(format!("compatibility stream differs: {name}")));
        }
    }
    let [width, height, key_size, soft_cursor, complex_input] = meta.geometry;
    let geometry: Value = serde_json::from_slice(&read_regular(&root.join("geometry.json"))?)?;
    if geometry
        != serde_json::json!({"width":width,"height":height,"key_size":key_size,"soft_cursor":soft_cursor,"complex_input":complex_input})
    {
        return Err(invalid("geometry metadata differs"));
    }
    if recording_files(root)? != expected {
        return Err(invalid("recording file set differs from captured inputs"));
    }
    Ok(meta)
}

/// Host extensions may add diagnostics but cannot replace trusted core values.
pub fn merge_host_observation(core: &mut Value, extension: Value) -> io::Result<()> {
    let base = core
        .as_object_mut()
        .ok_or_else(|| invalid("core frame is not an object"))?;
    let extra = extension
        .as_object()
        .ok_or_else(|| invalid("host extension is not an object"))?;
    if let Some(key) = extra.keys().find(|key| base.contains_key(*key)) {
        return Err(invalid(format!(
            "host extension replaces a core observation: {key}"
        )));
    }
    base.extend(extra.clone());
    Ok(())
}

/// The session owns streams; separate per-process output prevents replay from
/// overwriting the source. Only complete, fully consumed sessions get a receipt.
pub struct Session {
    pub metadata: Metadata,
    output: PathBuf,
    original: Option<PathBuf>,
    replay: Option<BufReader<File>>,
    #[allow(dead_code)]
    environment: environment::Environment,
    #[allow(dead_code)]
    metadata_file: File,
    #[allow(dead_code)]
    geometry_file: File,
    geometry_pending: bool,
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
        validate_geometry(metadata.geometry)?;
        if metadata.pid < 1 || metadata.pid > i32::MAX as i64 || metadata.build.is_empty() {
            return Err(invalid("invalid initial identity"));
        }
        fs::create_dir_all(output)?;
        // Hosts may already have opened a console log or archive directory.
        // Every recorder-owned file below still uses create_new, so a prior
        // capture can never be overwritten.
        if !recording_files(output)?.is_empty() {
            return Err(invalid("recording output already contains a capture"));
        }
        let replay = if let Some(source) = original {
            let prior = validate_recording(source)?;
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
        let environment = if let Some(source) = original {
            let value = environment::Environment::load(source)?;
            value.copy_to(source, output)?;
            value
        } else {
            environment::Environment::capture(output)?
        };
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
        let mut geometry_file = exclusive(&output.join("geometry.json"))?;
        json_line(
            &mut geometry_file,
            &serde_json::json!({"width":width,"height":height,"key_size":key_size,"soft_cursor":soft_cursor,"complex_input":complex_input}),
        )?;
        Ok(Self {
            environment,
            metadata_file: manifest,
            geometry_file,
            geometry_pending: false,
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
    fn operation<T>(&mut self, work: impl FnOnce(&mut Self) -> io::Result<T>) -> io::Result<T> {
        if self.finished || self.failed {
            return Err(invalid("recording is closed or failed"));
        }
        let result = work(self);
        if result.is_err() {
            self.failed = true;
        }
        result
    }
    #[cfg(feature = "ffi")]
    pub(crate) fn start_early(
        output: &Path,
        original: Option<&Path>,
        build: String,
        pid: i64,
    ) -> io::Result<Self> {
        let geometry = if let Some(source) = original {
            validate_recording(source)?.geometry
        } else {
            [80, 24, 256, 0, 0]
        };
        let mut value = Self::start(
            output,
            original,
            Metadata {
                schema: SCHEMA.into(),
                build,
                pid,
                geometry,
            },
        )?;
        value.geometry_pending = true;
        value.environment.apply(output)?;
        if original.is_some() {
            for (name, v) in [
                "BORG_REPLAY_WIDTH",
                "BORG_REPLAY_HEIGHT",
                "BORG_REPLAY_KEY_SIZE",
                "BORG_REPLAY_SOFT_CURSOR",
                "BORG_REPLAY_COMPLEX_INPUT",
            ]
            .into_iter()
            .zip(geometry)
            {
                std::env::set_var(name, v.to_string());
            }
        }
        Ok(value)
    }
    #[cfg(feature = "ffi")]
    pub(crate) fn attach_geometry(&mut self, geometry: [i64; 5]) -> io::Result<()> {
        self.operation(|s| {
            validate_geometry(geometry)?;
            if !s.geometry_pending{return Err(invalid("terminal startup already attached"));}
            if let Some(expected)=s.expected()? {
                if expected!=(Event::TerminalInit{geometry}) || geometry!=s.metadata.geometry{return Err(invalid("terminal startup differs"));}
            }
            s.metadata.geometry=geometry;
            for file in [&mut s.metadata_file,&mut s.geometry_file]{file.set_len(0)?;file.seek(SeekFrom::Start(0))?;}
            json_line(&mut s.metadata_file,&s.metadata)?;
            let [width,height,key_size,soft_cursor,complex_input]=geometry;
            json_line(&mut s.geometry_file,&serde_json::json!({"width":width,"height":height,"key_size":key_size,"soft_cursor":soft_cursor,"complex_input":complex_input}))?;
            s.write_event(&Event::TerminalInit{geometry})?;s.geometry_pending=false;Ok(())
        })
    }
    pub fn newer(&mut self, actual: bool) -> io::Result<bool> {
        self.operation(|s| {
            let value = match s.expected()? {
                None => actual,
                Some(Event::Newer { value }) => value,
                _ => return Err(invalid("file-newer call diverged")),
            };
            s.write_event(&Event::Newer { value })?;
            Ok(value)
        })
    }
    pub fn directory_open(&mut self, actual: bool) -> io::Result<bool> {
        self.operation(|s| {
            let present = match s.expected()? {
                None => actual,
                Some(Event::DirectoryOpen { present }) => present,
                _ => return Err(invalid("directory-open call diverged")),
            };
            s.write_event(&Event::DirectoryOpen { present })?;
            Ok(present)
        })
    }
    pub fn directory_read(
        &mut self,
        only_files: bool,
        capacity: usize,
        actual: Option<Vec<u8>>,
    ) -> io::Result<Option<Vec<u8>>> {
        self.operation(|s| {
            let name = match s.expected()? {
                None => actual,
                Some(Event::DirectoryRead {
                    only_files: f,
                    capacity: c,
                    name,
                }) if f == only_files && c == capacity => name,
                _ => return Err(invalid("directory-read call diverged")),
            };
            validate_directory_entry(capacity, &name)?;
            s.write_event(&Event::DirectoryRead {
                only_files,
                capacity,
                name: name.clone(),
            })?;
            Ok(name)
        })
    }
    pub fn poll(&mut self, input: Input) -> io::Result<Input> {
        self.operation(|s| s.poll_inner(input))
    }
    pub fn exists(&mut self, value: bool) -> io::Result<bool> {
        self.operation(|s| s.exists_inner(value))
    }
    pub fn clock(&mut self, value: i64) -> io::Result<i64> {
        self.operation(|s| s.clock_inner(value))
    }
    pub fn bot(
        &mut self,
        turn: i64,
        count: i64,
        event_type: i64,
        code: u32,
        mods: u8,
    ) -> io::Result<()> {
        self.operation(|s| s.bot_inner(turn, count, event_type, code, mods))
    }
    pub fn read_path(&mut self, path: &Path) -> io::Result<PathBuf> {
        self.operation(|s| s.read_path_inner(path))
    }
    pub fn frame(&mut self, value: &Value) -> io::Result<()> {
        self.operation(|s| s.frame_inner(value))
    }
    pub fn message(&mut self, value: &Value) -> io::Result<()> {
        self.operation(|s| s.message_inner(value))
    }
    pub fn combat(&mut self, value: &Value) -> io::Result<()> {
        self.operation(|s| s.combat_inner(value))
    }
    pub fn finish(&mut self) -> io::Result<()> {
        if self.finished {
            return Ok(());
        }
        self.operation(|s| s.finish_inner())
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
    fn poll_inner(&mut self, actual: Input) -> io::Result<Input> {
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
        validate_poll(input, self.metadata.geometry[2])?;
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
    fn exists_inner(&mut self, actual: bool) -> io::Result<bool> {
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
    fn clock_inner(&mut self, actual: i64) -> io::Result<i64> {
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
    fn bot_inner(
        &mut self,
        turn: i64,
        count: i64,
        event_type: i64,
        code: u32,
        mods: u8,
    ) -> io::Result<()> {
        if !valid_event_type(event_type) || mods > 31 {
            return Err(invalid("invalid bot key fields"));
        }
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
    fn read_path_inner(&mut self, path: &Path) -> io::Result<PathBuf> {
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
        if fs::symlink_metadata(&target).is_ok() {
            return Err(invalid("captured input target already exists"));
        }
        if let Some(bytes) = bytes {
            exclusive(&target)?.write_all(&bytes)?;
        }
        Ok(target)
    }
    fn frame_inner(&mut self, frame: &Value) -> io::Result<()> {
        if !frame.is_object() {
            return Err(invalid("observation must be an object"));
        }
        json_line(&mut self.frames, frame)?;
        self.frame_count += 1;
        Ok(())
    }
    fn combat_inner(&mut self, event: &Value) -> io::Result<()> {
        if !event.is_object() {
            return Err(invalid("observation must be an object"));
        }
        json_line(self.legacy.get_mut("combat.jsonl").unwrap(), event)
    }
    fn message_inner(&mut self, message: &Value) -> io::Result<()> {
        if !message.is_object() {
            return Err(invalid("observation must be an object"));
        }
        json_line(
            self.legacy.get_mut("native-messages.jsonl").unwrap(),
            message,
        )?;
        json_line(&mut self.messages, message)
    }
    fn finish_inner(&mut self) -> io::Result<()> {
        if self.geometry_pending {
            return Err(invalid("frontend initialization incomplete"));
        }
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
        receipt_new(
            &self.output.join("record-complete.json"),
            &Receipt {
                inputs: self.polls,
                frames: self.frame_count,
            },
        )?;
        if let Err(error) = receipt_new(
            &self.output.join("complete.json"),
            &Receipt {
                inputs: self.count,
                frames: self.frame_count,
            },
        ) {
            let _ = fs::remove_file(self.output.join("record-complete.json"));
            return Err(error);
        }
        if let Err(error) = validate_recording(&self.output) {
            let _ = fs::remove_file(self.output.join("complete.json"));
            let _ = fs::remove_file(self.output.join("record-complete.json"));
            return Err(error);
        }
        self.finished = true;
        Ok(())
    }
}

/// Byte comparison includes all observations, captured reads and the footer.
/// Re-execution must have completed before invoking this function.
pub fn verify(original: &Path, replay: &Path) -> io::Result<()> {
    validate_recording(original)?;
    validate_recording(replay)?;
    let names = recording_files(original)?;
    if names != recording_files(replay)? {
        return Err(invalid("recording compared-file sets differ"));
    }
    for name in names {
        if read_regular(&original.join(&name))? != read_regular(&replay.join(&name))? {
            return Err(invalid(format!("replay differs: {name}")));
        }
    }
    Ok(())
}
