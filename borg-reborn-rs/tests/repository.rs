//! Keeps the repository's documents and pins consistent with each other.

use std::path::{Path, PathBuf};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .to_path_buf()
}

fn read(rel: &str) -> String {
    std::fs::read_to_string(root().join(rel)).unwrap_or_else(|e| panic!("{rel}: {e}"))
}

#[test]
fn every_pinned_commit_is_a_full_sha() {
    let pins = read("pins.json");
    let mut seen = 0;
    for (i, _) in pins.match_indices("\"commit\": \"") {
        let start = i + "\"commit\": \"".len();
        let sha = &pins[start..start + 41];
        assert!(
            sha.ends_with('"'),
            "commit at byte {start} is not 40 characters"
        );
        assert!(
            sha[..40].bytes().all(|b| b.is_ascii_hexdigit()),
            "bad commit {sha}"
        );
        seen += 1;
    }
    assert!(
        seen >= 6,
        "expected a pin per related repository, found {seen}"
    );
}

#[test]
fn engine_pin_matches_pins_json() {
    let pins = read("pins.json");
    assert!(pins.contains(borg_reborn_rs::ENGINE_PIN));
}

#[test]
fn entry_documents_exist() {
    for rel in [
        "AGENTS.md",
        "RULES.md",
        "PLAN.md",
        "docs/GOALS.md",
        "docs/ARCHITECTURE.md",
        "docs/SOURCES.md",
        "docs/HANDOVER.md",
        "reference/c-borg/BORG_SPEC.md",
        "reference/c-borg/TUNABLE-BORG-PLAN.md",
    ] {
        assert!(root().join(rel).is_file(), "missing {rel}");
    }
}

#[test]
fn claude_md_only_includes_agents_md() {
    assert_eq!(read("CLAUDE.md").trim(), "@AGENTS.md");
}
