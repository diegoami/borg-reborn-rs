use angband_recorder::{verify, Input, Metadata, Session, SCHEMA};
use serde_json::json;
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
static NEXT: AtomicUsize = AtomicUsize::new(0);
struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "recorder-test-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
    fn meta() -> Metadata {
        Metadata {
            schema: SCHEMA.into(),
            build: "vanilla-test".into(),
            pid: 42,
            geometry: [80, 24, 256, 0, 0],
        }
    }
    fn poll() -> Input {
        Input {
            wait: 0,
            take: 1,
            result: 1,
            width: 80,
            height: 24,
            ..Input::default()
        }
    }
    fn record(&self) {
        let mut s = Session::start(&self.0.join("record"), None, Self::meta()).unwrap();
        assert_eq!(s.clock(123).unwrap(), 123);
        assert_eq!(s.poll(Self::poll()).unwrap().result, 1); // empty poll matters
        fs::write(self.0.join("prefs"), b"original keymap").unwrap();
        s.read_path(&self.0.join("prefs")).unwrap();
        s.read_path(&self.0.join("absent")).unwrap();
        s.bot(1, 1, 1, 65, 0).unwrap();
        s.frame(&json!({"rng":[1,2],"hp":100})).unwrap();
        s.message(&json!({"text":"hello"})).unwrap();
        s.finish().unwrap();
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}
#[test]
fn round_trip_replays_external_values_and_captured_read_bytes() {
    let f = Fixture::new();
    f.record();
    fs::write(f.0.join("prefs"), b"changed after capture").unwrap();
    let mut s = Session::start(
        &f.0.join("replay"),
        Some(&f.0.join("record")),
        Fixture::meta(),
    )
    .unwrap();
    assert_eq!(s.clock(999).unwrap(), 123);
    assert_eq!(s.poll(Fixture::poll()).unwrap().result, 1);
    let snapshot = s.read_path(&f.0.join("prefs")).unwrap();
    assert_eq!(fs::read(snapshot).unwrap(), b"original keymap");
    assert!(!s.read_path(&f.0.join("absent")).unwrap().exists());
    s.bot(1, 1, 1, 65, 0).unwrap();
    s.frame(&json!({"rng":[1,2],"hp":100})).unwrap();
    s.message(&json!({"text":"hello"})).unwrap();
    s.finish().unwrap();
    verify(&f.0.join("record"), &f.0.join("replay")).unwrap();
    fs::write(f.0.join("replay/frames.jsonl"), "changed RNG").unwrap();
    assert!(verify(&f.0.join("record"), &f.0.join("replay")).is_err());
}
#[test]
fn cannot_complete_an_unconsumed_or_malformed_replay() {
    let f = Fixture::new();
    f.record();
    let mut s = Session::start(
        &f.0.join("replay"),
        Some(&f.0.join("record")),
        Fixture::meta(),
    )
    .unwrap();
    assert!(s.finish().is_err());
    assert!(!f.0.join("replay/complete.json").exists());
    fs::write(f.0.join("record/inputs.jsonl"), "{bad json}\n").unwrap();
    let mut s = Session::start(
        &f.0.join("malformed"),
        Some(&f.0.join("record")),
        Fixture::meta(),
    )
    .unwrap();
    assert!(s.clock(1).is_err());
    assert!(s.finish().is_err());
}
#[test]
fn bot_actions_are_checked_not_supplied() {
    let f = Fixture::new();
    let mut s = Session::start(&f.0.join("record"), None, Fixture::meta()).unwrap();
    s.bot(1, 1, 1, 65, 0).unwrap();
    s.finish().unwrap();
    let mut s = Session::start(
        &f.0.join("replay"),
        Some(&f.0.join("record")),
        Fixture::meta(),
    )
    .unwrap();
    assert!(s.bot(1, 1, 1, 66, 0).is_err());
    assert!(s.finish().is_err());
}
#[test]
fn corrupt_file_cannot_be_replayed() {
    let f = Fixture::new();
    f.record();
    fs::write(f.0.join("record/file-input-000001.bin"), "tamper").unwrap();
    let mut s = Session::start(
        &f.0.join("replay"),
        Some(&f.0.join("record")),
        Fixture::meta(),
    )
    .unwrap();
    s.clock(0).unwrap();
    s.poll(Fixture::poll()).unwrap();
    assert!(s.read_path(&f.0.join("prefs")).is_err());
    assert!(s.finish().is_err());
}
#[test]
fn identity_and_output_are_guarded() {
    let f = Fixture::new();
    f.record();
    assert!(Session::start(&f.0.join("record"), None, Fixture::meta()).is_err());
    let mut meta = Fixture::meta();
    meta.build = "different executable".into();
    assert!(Session::start(&f.0.join("wrong"), Some(&f.0.join("record")), meta).is_err());
    fs::remove_file(f.0.join("record/complete.json")).unwrap();
    assert!(Session::start(
        &f.0.join("incomplete"),
        Some(&f.0.join("record")),
        Fixture::meta()
    )
    .is_err());
}
#[test]
fn call_order_and_peek_take_semantics_are_checked() {
    let f = Fixture::new();
    f.record();
    let mut s = Session::start(
        &f.0.join("order"),
        Some(&f.0.join("record")),
        Fixture::meta(),
    )
    .unwrap();
    assert!(s.poll(Fixture::poll()).is_err());
    assert!(s.finish().is_err());
    let mut s = Session::start(
        &f.0.join("mode"),
        Some(&f.0.join("record")),
        Fixture::meta(),
    )
    .unwrap();
    s.clock(0).unwrap();
    let mut poll = Fixture::poll();
    poll.take = 0;
    assert!(s.poll(poll).is_err());
    assert!(s.finish().is_err());
}

#[test]
fn host_logs_can_precede_capture_without_overwriting_a_recording() {
    let f = Fixture::new();
    let output = f.0.join("with-host-log");
    fs::create_dir(&output).unwrap();
    fs::write(output.join("engine.log"), b"host startup").unwrap();
    fs::create_dir(output.join("archive")).unwrap();
    let mut s = Session::start(&output, None, Fixture::meta()).unwrap();
    s.finish().unwrap();
    assert_eq!(
        fs::read(output.join("engine.log")).unwrap(),
        b"host startup"
    );
    assert!(Session::start(&output, None, Fixture::meta()).is_err());
}
