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
    assert!(Session::start(
        &f.0.join("malformed"),
        Some(&f.0.join("record")),
        Fixture::meta()
    )
    .is_err());
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
    assert!(Session::start(
        &f.0.join("replay"),
        Some(&f.0.join("record")),
        Fixture::meta()
    )
    .is_err());
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

#[test]
fn review_f1_requires_all_streams_and_typed_matching_receipts() {
    for name in [
        "recording.json",
        "inputs.jsonl",
        "frames.jsonl",
        "messages.jsonl",
        "events.txt",
        "clock.txt",
        "pid.txt",
        "actions.txt",
        "file-inputs.txt",
        "geometry.json",
        "native-messages.jsonl",
        "combat.jsonl",
        "record-complete.json",
    ] {
        let f = Fixture::new();
        f.record();
        fs::remove_file(f.0.join("record").join(name)).unwrap();
        assert!(
            Session::start(
                &f.0.join("replay"),
                Some(&f.0.join("record")),
                Fixture::meta()
            )
            .is_err(),
            "{name}"
        );
        assert!(
            verify(&f.0.join("record"), &f.0.join("record")).is_err(),
            "{name}"
        );
    }
    for data in ["{bad json}", "null", "{}", "{\"inputs\":0,\"frames\":0}"] {
        let f = Fixture::new();
        f.record();
        fs::write(f.0.join("record/complete.json"), data).unwrap();
        assert!(verify(&f.0.join("record"), &f.0.join("record")).is_err());
    }
    let f = Fixture::new();
    for name in ["one", "two"] {
        fs::create_dir(f.0.join(name)).unwrap();
        fs::write(f.0.join(name).join("complete.json"), "{}").unwrap();
    }
    assert!(verify(&f.0.join("one"), &f.0.join("two")).is_err());
}
#[test]
fn review_f1_rejects_extra_blobs_and_truncated_observations() {
    let f = Fixture::new();
    f.record();
    fs::write(f.0.join("record/file-input-999999.bin"), "extra").unwrap();
    assert!(verify(&f.0.join("record"), &f.0.join("record")).is_err());
    fs::remove_file(f.0.join("record/file-input-999999.bin")).unwrap();
    fs::write(f.0.join("record/frames.jsonl"), "{}").unwrap();
    assert!(verify(&f.0.join("record"), &f.0.join("record")).is_err());
}
#[test]
fn review_f2_rejects_native_narrowing_and_invalid_queue_domains() {
    let bad: &[(&str, i64)] = &[
        ("head", 256),
        ("tail", -1),
        ("code", 4294967296),
        ("mods", 256),
        ("mods", 32),
        ("event_type", 3),
        ("event_type", 512),
        ("result", 2),
        ("wait", 2),
        ("take", -1),
        ("x", 256),
        ("button", 256),
        ("disconnect", 2),
        ("mapped", -1),
    ];
    for (field, value) in bad {
        let f = Fixture::new();
        let mut session = Session::start(&f.0.join("record"), None, Fixture::meta()).unwrap();
        let mut input = serde_json::to_value(Fixture::poll()).unwrap();
        input[*field] = json!(value);
        assert!(
            session
                .poll(serde_json::from_value(input).unwrap())
                .is_err(),
            "{field}={value}"
        );
        assert!(session.finish().is_err());
        assert!(!f.0.join("record/complete.json").exists());
    }
    for geometry in [
        [0, 24, 256, 0, 0],
        [80, 24, 65536, 0, 0],
        [80, 24, 256, 2, 0],
        [256, 24, 256, 0, 0],
    ] {
        let f = Fixture::new();
        let mut meta = Fixture::meta();
        meta.geometry = geometry;
        assert!(Session::start(&f.0.join("record"), None, meta).is_err());
    }
}
#[test]
fn review_f5_cannot_open_a_stale_or_symlinked_absent_input() {
    use std::os::unix::fs::symlink;
    for link in [false, true] {
        let f = Fixture::new();
        let mut session = Session::start(&f.0.join("record"), None, Fixture::meta()).unwrap();
        let path = f.0.join("record/file-input-000001.bin");
        if link {
            symlink(f.0.join("absent-link-target"), &path).unwrap();
        } else {
            fs::write(&path, "stale").unwrap();
        }
        assert!(session.read_path(&f.0.join("absent")).is_err());
        assert!(session.finish().is_err());
        assert!(!f.0.join("record/complete.json").exists());
    }
}
#[test]
fn review_f6_missing_blob_after_start_and_creation_errors_poison_completion() {
    let f = Fixture::new();
    f.record();
    let mut replay = Session::start(
        &f.0.join("replay"),
        Some(&f.0.join("record")),
        Fixture::meta(),
    )
    .unwrap();
    fs::remove_file(f.0.join("record/file-input-000001.bin")).unwrap();
    replay.clock(0).unwrap();
    replay.poll(Fixture::poll()).unwrap();
    assert!(replay.read_path(&f.0.join("prefs")).is_err());
    assert!(replay.finish().is_err());
    assert!(!f.0.join("replay/complete.json").exists());
    let mut capture = Session::start(&f.0.join("capture"), None, Fixture::meta()).unwrap();
    fs::create_dir(f.0.join("capture/file-input-000001.bin")).unwrap();
    assert!(capture.read_path(&f.0.join("prefs")).is_err());
    assert!(capture.finish().is_err());
}
#[test]
fn review_f6_sealed_streams_cannot_be_appended() {
    let f = Fixture::new();
    let mut session = Session::start(&f.0.join("record"), None, Fixture::meta()).unwrap();
    session.finish().unwrap();
    assert!(session.frame(&json!({"late":1})).is_err());
    assert!(session.message(&json!({"late":1})).is_err());
    assert!(session.combat(&json!({"late":1})).is_err());
    assert!(fs::read(f.0.join("record/frames.jsonl"))
        .unwrap()
        .is_empty());
    assert!(fs::read(f.0.join("record/messages.jsonl"))
        .unwrap()
        .is_empty());
    verify(&f.0.join("record"), &f.0.join("record")).unwrap();
}

#[test]
fn review_f16_host_extensions_cannot_replace_core_or_panic_on_shape() {
    use angband_recorder::merge_host_observation;
    let original = json!({"turn":1,"rng":"abc","cells":[],"index":1,"world":"123"});
    for extension in [
        json!(null),
        json!([]),
        json!(42),
        json!({"turn":2}),
        json!({"rng":"bad"}),
        json!({"world":"bad"}),
        json!({"cells":[]}),
        json!({"index":2}),
    ] {
        let mut frame = original.clone();
        assert!(merge_host_observation(&mut frame, extension).is_err());
        assert_eq!(frame, original);
    }
    let mut frame = original.clone();
    merge_host_observation(&mut frame, json!({"inventory":[]})).unwrap();
    assert_eq!(frame["turn"], 1);
}

#[test]
fn review_f3_f9_filesystem_decisions_are_ordered_inputs_not_live_state() {
    let f = Fixture::new();
    let mut record = Session::start(&f.0.join("record"), None, Fixture::meta()).unwrap();
    assert!(record.newer(true).unwrap());
    assert!(record.directory_open(true).unwrap());
    assert_eq!(
        record
            .directory_read(true, 32, Some(b"second.save".to_vec()))
            .unwrap(),
        Some(b"second.save".to_vec())
    );
    assert_eq!(
        record
            .directory_read(true, 32, Some(b"first.save".to_vec()))
            .unwrap(),
        Some(b"first.save".to_vec())
    );
    assert_eq!(record.directory_read(true, 32, None).unwrap(), None);
    record.finish().unwrap();
    let mut replay = Session::start(
        &f.0.join("replay"),
        Some(&f.0.join("record")),
        Fixture::meta(),
    )
    .unwrap();
    assert!(replay.newer(false).unwrap());
    assert!(replay.directory_open(false).unwrap());
    assert_eq!(
        replay
            .directory_read(true, 32, Some(b"unrelated.new".to_vec()))
            .unwrap(),
        Some(b"second.save".to_vec())
    );
    assert_eq!(
        replay.directory_read(true, 32, None).unwrap(),
        Some(b"first.save".to_vec())
    );
    assert_eq!(
        replay
            .directory_read(true, 32, Some(b"extra".to_vec()))
            .unwrap(),
        None
    );
    replay.finish().unwrap();
    verify(&f.0.join("record"), &f.0.join("replay")).unwrap();
}
#[test]
fn directory_filter_buffer_and_entry_domains_fail_closed() {
    let f = Fixture::new();
    let mut record = Session::start(&f.0.join("record"), None, Fixture::meta()).unwrap();
    assert!(record
        .directory_read(true, 1, Some(b"long".to_vec()))
        .is_err());
    assert!(record.finish().is_err());
}
