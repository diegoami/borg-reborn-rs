fn main() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap();
    println!(
        "cargo:rerun-if-changed={}",
        root.join("recorder-pin.json").display()
    );
    println!(
        "cargo:rerun-if-changed={}",
        root.join("vendor/angband-recorder").display()
    );
    let status = std::process::Command::new("python3")
        .arg(root.join("borg-reborn-rs/check-recorder-pin.py"))
        .arg(root.join("recorder-pin.json"))
        .arg(root.join("vendor/angband-recorder"))
        .status()
        .expect("run recorder integrity check");
    assert!(
        status.success(),
        "vendored recorder differs from its release pin"
    );
}
