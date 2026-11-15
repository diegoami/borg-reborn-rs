//! borg-reborn-rs (#222): the vendored core's decision trace.
//!
//! Nothing here is read by policy. Snapshots use the core's own state after
//! a selected check and before its queued keys are consumed. Danger is a
//! read-only evaluation (no RNG); full-suite parity checks guard that seam.
use crate::borg_headers::borg_trait_h::{BI_CDEPTH, BI_CURHP, BI_CURSP, BI_MAXHP, BI_MAXSP};
use crate::state::BorgState;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

#[derive(Debug, Default)]
struct Selection {
    name: Option<&'static str>,
    source: Option<&'static str>,
    errand: Option<(String, Option<i32>)>,
}

thread_local! {
    // Nested engine message handling may invoke another think. A stack keeps
    // its selection separate from the outer think, without holding a lock.
    static FRAMES: RefCell<Vec<Selection>> = const { RefCell::new(Vec::new()) };
}
static PATH: OnceLock<Option<PathBuf>> = OnceLock::new();
static WRITER: OnceLock<Mutex<Recorder>> = OnceLock::new();

struct Recorder {
    file: File,
    think: u64,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
struct Goals {
    leaving: bool,
    fleeing: bool,
    fleeing_to_town: bool,
    rising: bool,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
struct Errand {
    name: String,
    target: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
struct Line {
    schema: String,
    think: u64,
    turn: i32,
    borg_turn: i32,
    context: String,
    depth: i32,
    hp: i32,
    max_hp: i32,
    sp: i32,
    max_sp: i32,
    danger: i32,
    avoidance: i32,
    check: String,
    check_source: Option<String>,
    goals: Goals,
    errand: Option<Errand>,
    keys: Vec<u32>,
    acted: bool,
}

fn failure(error: impl std::fmt::Display) -> ! {
    eprintln!("borg-reborn-rs: core trace: {error}");
    std::process::exit(2)
}

pub(crate) fn begin() -> bool {
    let path = PATH.get_or_init(|| {
        if std::env::var("BORG_CORE_TRACE")
            .is_ok_and(|v| matches!(v.as_str(), "off" | "0" | "false"))
        {
            return None;
        }
        std::env::var_os("BORG_RUN_DIR").map(|dir| PathBuf::from(dir).join("trace-core.jsonl"))
    });
    if path.is_none() {
        return false;
    }
    let errand = crate::shell::trace_errand();
    FRAMES.with(|frames| {
        let mut frames = frames.borrow_mut();
        let errand = match errand {
            Ok(value) => value,
            Err(()) => frames.last().and_then(|f| f.errand.clone()),
        };
        frames.push(Selection {
            errand,
            ..Selection::default()
        });
    });
    true
}

pub(crate) fn selected(name: &'static str, source: &'static str) {
    FRAMES.with(|frames| {
        if let Some(frame) = frames.borrow_mut().last_mut() {
            frame.name = Some(name);
            frame.source = Some(source);
        }
    });
}

// One marked hook at each successful check's return. It evaluates no policy
// expression again and returns the original `true` unchanged.
macro_rules! choice {
    ($name:literal) => {{
        crate::core_trace::selected($name, concat!(file!(), ":", line!()));
        true
    }};
}
pub(crate) use choice;

pub(crate) fn finish(st: &mut BorgState, active: bool, acted: bool) {
    if !active {
        return;
    }
    let frame = FRAMES.with(|frames| frames.borrow_mut().pop().expect("core trace frame"));
    let depth = st.borg.tr(BI_CDEPTH);
    let check = frame.name.unwrap_or(if acted {
        "unclassified"
    } else {
        "borg_think_no_action"
    });
    let context = if check == "borg_think_store" {
        "store"
    } else if depth == 0 {
        "town"
    } else {
        "dungeon"
    };
    let danger = crate::borg_danger::borg_danger(st, st.borg.c.y, st.borg.c.x, 1, true, false);
    let errand = match crate::shell::trace_errand() {
        Ok(Some(value)) => Some(value),
        Ok(None) if matches!(check, "errand_handoff" | "errand_steer") => frame.errand,
        Ok(None) => None,
        Err(()) => frame.errand,
    }
    .map(|(name, target)| Errand { name, target });
    let mut line = Line {
        schema: "borg-reborn-rs/core-trace@1".into(),
        think: 0,
        turn: crate::engine::game_turn(),
        borg_turn: st.borg.time.now,
        context: context.into(),
        depth,
        hp: st.borg.tr(BI_CURHP),
        max_hp: st.borg.tr(BI_MAXHP),
        sp: st.borg.tr(BI_CURSP),
        max_sp: st.borg.tr(BI_MAXSP),
        danger,
        avoidance: i32::from(st.borg.avoidance),
        check: check.into(),
        check_source: frame.source.map(str::to_string),
        goals: Goals {
            leaving: st.borg.goal.leaving,
            fleeing: st.borg.goal.fleeing,
            fleeing_to_town: st.borg.goal.fleeing_to_town,
            rising: st.borg.goal.rising,
        },
        errand,
        keys: st.io.queued_keys(),
        acted,
    };
    let writer = WRITER.get_or_init(|| {
        let path = PATH
            .get()
            .and_then(Option::as_ref)
            .expect("enabled core trace path");
        let file =
            File::create(path).unwrap_or_else(|e| failure(format!("{}: {e}", path.display())));
        Mutex::new(Recorder { file, think: 0 })
    });
    let mut writer = writer.lock().unwrap_or_else(|e| e.into_inner());
    writer.think = writer.think.wrapping_add(1);
    line.think = writer.think;
    let mut bytes = serde_json::to_vec(&line).unwrap_or_else(|e| failure(e));
    bytes.push(b'\n');
    // A direct write per line leaves no userspace buffer stranded when the C
    // engine exits on a death. Never hold this lock across a policy call.
    writer.file.write_all(&bytes).unwrap_or_else(|e| failure(e));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nested_frames_keep_their_own_selected_check() {
        FRAMES.with(|frames| frames.borrow_mut().push(Selection::default()));
        selected("outer", "outer.rs:1");
        FRAMES.with(|frames| frames.borrow_mut().push(Selection::default()));
        selected("inner", "inner.rs:2");
        FRAMES.with(|frames| {
            let mut frames = frames.borrow_mut();
            assert_eq!(frames.pop().unwrap().name, Some("inner"));
            assert_eq!(frames.pop().unwrap().name, Some("outer"));
        });
    }

    #[test]
    fn line_round_trips_with_keys_and_errand() {
        let line = Line {
            schema: "borg-reborn-rs/core-trace@1".into(),
            think: 9,
            turn: 1234,
            borg_turn: 120,
            context: "dungeon".into(),
            depth: 3,
            hp: 12,
            max_hp: 40,
            sp: 2,
            max_sp: 9,
            danger: 15,
            avoidance: 12,
            check: "borg_caution".into(),
            check_source: Some("borg_think_dungeon.rs:1536".into()),
            goals: Goals {
                leaving: true,
                fleeing: true,
                fleeing_to_town: false,
                rising: false,
            },
            errand: Some(Errand {
                name: "trips:1*/1,2\nquoted \"plan\"".into(),
                target: Some(2),
            }),
            keys: vec![27, 82, 49, 48, 48, 13, 0x80],
            acted: true,
        };
        let text = serde_json::to_string(&line).unwrap();
        assert!(!text.contains('\n'));
        assert_eq!(serde_json::from_str::<Line>(&text).unwrap(), line);
    }
}
