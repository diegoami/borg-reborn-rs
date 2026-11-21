use angband_recorder::checkpoints::{validate, Config, RollingSaves};
use serde_json::{json, Value};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

struct Scratch(PathBuf);
impl Scratch {
    fn new() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let root = std::env::temp_dir().join(format!(
            "recorder-saves-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&root).unwrap();
        fs::write(root.join("frames.jsonl"), b"").unwrap();
        Self(root)
    }
    fn frame(&self, number: u64, turn: i64) -> Value {
        let bytes = format!("native save at {number}").into_bytes();
        fs::write(self.0.join("world-fingerprint.save"), &bytes).unwrap();
        let hash = bytes.iter().fold(14695981039346656037u64, |h, b| {
            (h ^ u64::from(*b)).wrapping_mul(1099511628211)
        });
        let frame = json!({"index": number, "phase":"command", "dead":0, "turn":turn, "input":number*2, "world":format!("{hash:016x}")});
        writeln!(
            OpenOptions::new()
                .append(true)
                .open(self.0.join("frames.jsonl"))
                .unwrap(),
            "{frame}"
        )
        .unwrap();
        frame
    }
    fn index(&self) -> Value {
        serde_json::from_slice(&fs::read(self.0.join("checkpoints/index.json")).unwrap()).unwrap()
    }
}
impl Drop for Scratch {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}

#[test]
fn opt_in_defaults_ranges_and_disabled_capture() {
    assert!(Config::parse(None, None).unwrap().is_none());
    let config = Config::parse(Some("3"), None).unwrap().unwrap();
    assert_eq!((config.recent, config.every_turns), (3, 1000));
    assert!(Config::parse(Some("0"), Some("0")).unwrap().is_none());
    for (window, every) in [
        ("-1", "10"),
        ("10001", "10"),
        ("2", "-1"),
        ("2", "1000000001"),
        (" ", "1"),
        ("2", "x"),
    ] {
        assert!(Config::parse(Some(window), Some(every)).is_err());
    }
}
#[test]
fn recent_window_and_periodic_points_preserve_exact_bytes() {
    let s = Scratch::new();
    let mut rolling = RollingSaves::new(
        &s.0,
        Config {
            recent: 2,
            every_turns: 10,
        },
    )
    .unwrap();
    for (number, turn) in [(1, 10), (2, 11), (3, 12), (4, 20), (5, 21)] {
        rolling
            .capture(&s.frame(number, turn), &s.0.join("world-fingerprint.save"))
            .unwrap();
    }
    let index = s.index();
    let entries = index["saves"].as_array().unwrap();
    assert_eq!(
        entries
            .iter()
            .map(|e| e["frame"].as_u64().unwrap())
            .collect::<Vec<_>>(),
        vec![1, 4, 5]
    );
    for entry in entries {
        assert_eq!(
            fs::read(
                s.0.join("checkpoints")
                    .join(entry["file"].as_str().unwrap())
            )
            .unwrap(),
            format!("native save at {}", entry["frame"]).as_bytes()
        );
    }
    validate(&s.0).unwrap();
}
#[test]
fn dead_and_non_command_frames_never_replace_a_living_point() {
    let s = Scratch::new();
    let mut rolling = RollingSaves::new(
        &s.0,
        Config {
            recent: 1,
            every_turns: 0,
        },
    )
    .unwrap();
    rolling
        .capture(&s.frame(1, 10), &s.0.join("world-fingerprint.save"))
        .unwrap();
    let before = fs::read(s.0.join("checkpoints/index.json")).unwrap();
    let mut dead = s.frame(2, 20);
    dead["dead"] = json!(1);
    rolling
        .capture(&dead, &s.0.join("world-fingerprint.save"))
        .unwrap();
    let mut input = s.frame(3, 30);
    input["phase"] = json!("input");
    rolling
        .capture(&input, &s.0.join("world-fingerprint.save"))
        .unwrap();
    assert_eq!(
        before,
        fs::read(s.0.join("checkpoints/index.json")).unwrap()
    );
    validate(&s.0).unwrap();
}
#[test]
fn changed_or_missing_saves_are_rejected() {
    let s = Scratch::new();
    let mut rolling = RollingSaves::new(
        &s.0,
        Config {
            recent: 1,
            every_turns: 0,
        },
    )
    .unwrap();
    rolling
        .capture(&s.frame(1, 10), &s.0.join("world-fingerprint.save"))
        .unwrap();
    let file =
        s.0.join("checkpoints")
            .join(s.index()["saves"][0]["file"].as_str().unwrap());
    fs::write(&file, b"corrupt").unwrap();
    assert!(validate(&s.0).is_err());
    fs::remove_file(file).unwrap();
    assert!(validate(&s.0).is_err());
}
#[test]
fn mismatched_snapshot_and_existing_directory_are_not_overwritten() {
    let s = Scratch::new();
    let mut rolling = RollingSaves::new(
        &s.0,
        Config {
            recent: 1,
            every_turns: 0,
        },
    )
    .unwrap();
    assert!(RollingSaves::new(
        &s.0,
        Config {
            recent: 1,
            every_turns: 0
        }
    )
    .is_err());
    let frame = s.frame(1, 10);
    fs::write(s.0.join("world-fingerprint.save"), b"wrong state").unwrap();
    assert!(rolling
        .capture(&frame, &s.0.join("world-fingerprint.save"))
        .is_err());
    assert!(s.index()["saves"].as_array().unwrap().is_empty());
}
#[test]
fn turn_rewind_retains_a_new_periodic_point_and_input_frames_are_bound() {
    let s = Scratch::new();
    let mut rolling = RollingSaves::new(
        &s.0,
        Config {
            recent: 0,
            every_turns: 10,
        },
    )
    .unwrap();
    for (number, turn) in [(1, 100), (2, 101), (3, 1)] {
        rolling
            .capture(&s.frame(number, turn), &s.0.join("world-fingerprint.save"))
            .unwrap();
    }
    assert_eq!(s.index()["saves"].as_array().unwrap().len(), 2);
    validate(&s.0).unwrap();
    let file = s.0.join("checkpoints/index.json");
    let mut index = s.index();
    index["saves"][0]["input"] = json!(999);
    fs::write(file, serde_json::to_vec(&index).unwrap()).unwrap();
    assert!(validate(&s.0).is_err());
}
