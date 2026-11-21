use std::fs;
use std::process::Command;

#[test]
fn turn_view_reconstructs_deltas_without_launching_the_game() {
    let root = std::env::temp_dir().join(format!("recorder-view-{}", std::process::id()));
    fs::create_dir(&root).unwrap();
    let frames=concat!(
        "{\"index\":1,\"phase\":\"command\",\"turn\":10,\"width\":2,\"height\":1,\"cells\":[[0,65],[1,66]],\"depth\":1,\"hp\":9,\"max_hp\":10}\n",
        "{\"index\":2,\"phase\":\"input\",\"turn\":20,\"width\":2,\"height\":1,\"cells\":[[1,67]],\"depth\":1,\"hp\":8,\"max_hp\":10}\n",
        "{\"index\":3,\"phase\":\"command\",\"turn\":20,\"width\":2,\"height\":1,\"cells\":[[0,68]],\"depth\":1,\"hp\":8,\"max_hp\":10}\n");
    fs::write(root.join("frames.jsonl"), frames).unwrap();
    let run = |args: &[&str]| {
        Command::new(env!("CARGO_BIN_EXE_angband-recorder"))
            .arg("view")
            .arg(&root)
            .args(args)
            .output()
            .unwrap()
    };
    let result = run(&["--turn", "15"]);
    assert!(result.status.success());
    let text = String::from_utf8(result.stdout).unwrap();
    assert!(text.contains("Frame 3, turn 20"));
    assert!(text.contains("DC"));
    assert!(!run(&["--turn", "21"]).status.success());
    assert!(!run(&["--turn", "-1"]).status.success());
    assert!(run(&["2"]).status.success());
    assert_eq!(
        fs::read_to_string(root.join("frames.jsonl")).unwrap(),
        frames
    );
    // There is no executable or launch.json: viewing only reads observations.
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn view_reaches_the_closing_frame() {
    let root = std::env::temp_dir().join(format!("recorder-view-final-{}", std::process::id()));
    fs::create_dir(&root).unwrap();
    fs::write(
        root.join("frames.jsonl"),
        concat!(
            "{\"index\":1,\"phase\":\"command\",\"turn\":10,\"width\":2,\"height\":1,\"cells\":[[0,65],[1,66]],\"depth\":1,\"hp\":9,\"max_hp\":10}\n",
            "{\"index\":2,\"phase\":\"final\",\"turn\":14,\"width\":2,\"height\":1,\"cells\":[[0,88]],\"depth\":1,\"hp\":-1,\"max_hp\":10}\n"
        ),
    )
    .unwrap();
    let run = |turn: &str| {
        Command::new(env!("CARGO_BIN_EXE_angband-recorder"))
            .arg("view")
            .arg(&root)
            .args(["--turn", turn])
            .output()
            .unwrap()
    };
    let result = run("12");
    assert!(result.status.success());
    assert!(String::from_utf8(result.stdout)
        .unwrap()
        .contains("Frame 2, turn 14"));
    assert!(!run("15").status.success());
    fs::remove_dir_all(root).unwrap();
}
