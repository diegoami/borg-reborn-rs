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

/// Links between the copied files of a death registry must resolve (review
/// finding on PR #1: copied assessments linked to absent
/// `overall-assessment.md` files). Checked: Markdown link targets, and JSON
/// string values starting with `./` or `../`, that end in `.md` or `.json`
/// and land at the registry root or a case root, which is exactly what the
/// copy includes. Links leaving the registry, or into the case
/// subdirectories that are not copied (snapshots, logs), are listed in
/// `reference/README.md` and not checked here.
#[test]
fn copied_death_registries_have_no_dangling_internal_links() {
    let mut checked = 0;
    let mut dangling = Vec::new();
    for registry in ["reference/deaths/better-borg-c", "reference/deaths/borg-rs"] {
        let base = root().join(registry).canonicalize().unwrap();
        let mut stack = vec![base.clone()];
        while let Some(dir) = stack.pop() {
            for entry in std::fs::read_dir(&dir).unwrap() {
                let path = entry.unwrap().path();
                if path.is_dir() {
                    stack.push(path);
                    continue;
                }
                let text = std::fs::read_to_string(&path).unwrap_or_default();
                let links = if path.extension().is_some_and(|e| e == "md") {
                    markdown_targets(&text)
                } else {
                    json_relative_strings(&text)
                };
                for link in links {
                    let link = link.split('#').next().unwrap_or_default().to_string();
                    if link.contains("://") || !(link.ends_with(".md") || link.ends_with(".json")) {
                        continue;
                    }
                    let target = normalize(&dir.join(&link));
                    let Ok(rel) = target.strip_prefix(&base) else {
                        continue;
                    };
                    if rel.components().count() > 2 {
                        continue;
                    }
                    checked += 1;
                    if !target.exists() {
                        dangling.push(format!("{} -> {link}", path.display()));
                    }
                }
            }
        }
    }
    assert!(
        checked > 50,
        "only {checked} links checked: the scan is broken"
    );
    assert!(
        dangling.is_empty(),
        "dangling links:\n{}",
        dangling.join("\n")
    );
}

/// Targets of Markdown links: `](target)`.
fn markdown_targets(text: &str) -> Vec<String> {
    text.match_indices("](")
        .filter_map(|(i, _)| {
            let rest = &text[i + 2..];
            let end = rest.find(')')?;
            let target = &rest[..end];
            (!target.contains(char::is_whitespace)).then(|| target.to_string())
        })
        .collect()
}

/// JSON string values that start with `./` or `../`.
fn json_relative_strings(text: &str) -> Vec<String> {
    text.split('"')
        .skip(1)
        .step_by(2)
        .filter(|s| {
            (s.starts_with("./") || s.starts_with("../")) && !s.contains(char::is_whitespace)
        })
        .map(str::to_string)
        .collect()
}

fn normalize(path: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for part in path.components() {
        match part {
            std::path::Component::ParentDir => {
                out.pop();
            }
            std::path::Component::CurDir => {}
            other => out.push(other),
        }
    }
    out
}

/// `angband/` is never edited here (RULES 9): its committed tree must be the
/// one recorded in `pins.json` (`engine_tree.tree`, docs/ENGINE-TREE.md). A
/// re-sync updates both in the same PR.
#[test]
fn engine_tree_matches_its_record() {
    let pins = read("pins.json");
    let Some(at) = pins.find("\"engine_tree\"") else {
        panic!("pins.json has no engine_tree record");
    };
    let rest = &pins[at..];
    let key = "\"tree\": \"";
    let start = rest.find(key).expect("engine_tree.tree") + key.len();
    let recorded = &rest[start..start + 40];
    let out = std::process::Command::new("git")
        .args(["rev-parse", "HEAD:angband"])
        .current_dir(root())
        .output();
    let Ok(out) = out else {
        eprintln!("git not available: engine tree not checked");
        return;
    };
    if !out.status.success() {
        eprintln!("not a git checkout with angband/: engine tree not checked");
        return;
    }
    assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), recorded);
}
