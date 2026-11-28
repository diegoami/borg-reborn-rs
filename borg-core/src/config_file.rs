//! borg-reborn-rs (#1339): the run's configuration file (`user/pborg.yaml`),
//! read for the parameter registry, the settings and the core selector.
//! These readers lived in the errand shell (`shell.rs`, #208) until the
//! errands were retired; they are not part of borg-rs (`OURS` in
//! `scripts/borg-core-vendor.py`).

use crate::engine;

/// The class names of `class.txt`, in its order (`player->class->cidx`).
pub(crate) const CLASSES: [&str; 9] = [
    "warrior",
    "mage",
    "druid",
    "priest",
    "necromancer",
    "paladin",
    "rogue",
    "ranger",
    "blackguard",
];

pub(crate) const PBORG_CONFIG: &[u8] = b"pborg.yaml";

pub fn warn_missing_pborg_config() {
    if let Ok(path) = borg_txt_path(engine::path_build_user(1024, PBORG_CONFIG)) {
        if matches!(path.try_exists(), Ok(false)) {
            eprintln!("borg-reborn-rs: {} is missing; using the reborn core defaults", path.display());
        }
    }
}

/// A setting of the run's configuration (the parameter registry, the core
/// selector and the scary list read it).
pub fn borg_txt_setting(name: &str) -> Option<String> {
    let text = borg_txt_contents().unwrap_or_else(|error| config_file_error(error));
    find_setting(&text, name)
}

/// Leave reporting to the strict loader when a probe runs before initialization.
pub(crate) fn borg_txt_setting_lenient(name: &str) -> Option<String> {
    find_setting(&borg_txt_contents().ok()?, name)
}

/// borg-reborn-rs (#704, review of #740): whether a `borg_world` value
/// reached the Borg unconsumed. `harness/run.sh` replaces the key with a
/// comment when it generates the world, so a value other than `none` here
/// means nothing loaded that world's data: the game would play upstream's
/// data under the world's name. (The plans module checked this until #1339.)
fn world_unloaded(value: Option<&str>) -> bool {
    value.is_some_and(|v| !matches!(v.trim(), "" | "none"))
}

/// Called before core initialization: refuse a world nothing loaded.
pub(crate) fn check_world() {
    // Lenient: an unreadable configuration is the parameter loader's to report.
    let world = borg_txt_setting_lenient("borg_world");
    if world_unloaded(world.as_deref()) {
        eprintln!(
            "borg_world: '{}' was not generated (run through harness/run.sh, or a zoo adapter that loads worlds); refusing to play the game's own data under its name",
            world.unwrap_or_default().trim()
        );
        std::process::exit(2);
    }
}

pub(crate) fn config_file_error(error: String) -> ! {
    eprintln!("borg-reborn-rs: configuration: {error}");
    std::process::exit(2)
}

fn borg_txt_path(bytes: Vec<u8>) -> Result<std::path::PathBuf, String> {
    #[cfg(unix)]
    {
        use std::os::unix::ffi::OsStringExt;
        Ok(std::ffi::OsString::from_vec(bytes).into())
    }
    #[cfg(not(unix))]
    {
        String::from_utf8(bytes).map(std::path::PathBuf::from)
            .map_err(|error| format!("invalid pborg.yaml path: {error}"))
    }
}

fn read_borg_txt(path: &std::path::Path) -> Result<String, String> {
    match std::fs::read_to_string(path) {
        Ok(text) => Ok(text),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(String::new()),
        Err(error) => Err(format!("{}: cannot read configuration: {error}", path.display())),
    }
}

/// Adapt the YAML mapping in memory to the legacy core's line-oriented
/// validators. No borg.txt is opened or produced by PBorg.
pub(crate) fn yaml_settings(text: &str) -> Result<String, String> {
    use serde_yaml_ng::Value;
    use std::fmt::Write;

    let settings: Option<serde_yaml_ng::Mapping> =
        serde_yaml_ng::from_str(text).map_err(|error| error.to_string())?;
    let mut lines = String::new();
    let mut formulas = None;
    for (key, value) in settings.into_iter().flatten() {
        let name = key.as_str().ok_or("setting names must be strings")?;
        if name == "formulas" {
            formulas = Some(value.as_str().ok_or("formulas must be a string")?.to_owned());
            continue;
        }
        if !name.starts_with("borg_")
            || !name.bytes().all(|c| c.is_ascii_alphanumeric() || b"_@".contains(&c))
        {
            return Err(format!("invalid setting name {name:?}"));
        }
        let scalar = match value {
            Value::String(value) => value,
            Value::Bool(value) => if value { "TRUE" } else { "FALSE" }.to_owned(),
            Value::Number(value) if value.is_i64() || value.is_u64() => value.to_string(),
            _ => return Err(format!("{name}: expected a string, integer or boolean")),
        };
        if scalar.contains(['\n', '\r', '\0']) || name.len() + scalar.len() + 3 >= 1023 {
            return Err(format!("{name}: expected a short, single-line value"));
        }
        writeln!(lines, "{name} = {scalar}").expect("writing to a String");
    }
    if let Some(formulas) = formulas {
        lines.push_str("[BEGIN FORMULA SECTION]\n");
        lines.push_str(&formulas);
        lines.push_str("\n[END FORMULA SECTION]\n");
    }
    Ok(lines)
}

pub(crate) fn borg_txt_contents() -> Result<String, String> {
    let path = borg_txt_path(engine::path_build_user(1024, PBORG_CONFIG))?;
    yaml_settings(&read_borg_txt(&path)?).map_err(|error| format!("{}: {error}", path.display()))
}

#[cfg(test)]
fn borg_txt_setting_at(path: &std::path::Path, name: &str) -> Option<String> {
    borg_txt_value_at(path, name).unwrap_or_else(|error| config_file_error(error))
}

#[cfg(test)]
pub(crate) fn borg_txt_value_at(path: &std::path::Path, name: &str) -> Result<Option<String>, String> {
    Ok(find_setting(&read_borg_txt(path)?, name))
}

fn find_setting(text: &str, name: &str) -> Option<String> {
    text.lines()
        .map(str::trim)
        .filter(|l| !l.starts_with('#'))
        .filter_map(|l| l.split_once('='))
        .find(|(n, _)| n.trim() == name)
        .map(|(_, v)| v.trim().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// #704 (review of #740): a world the harness did not consume stops
    /// the game before play.
    #[test]
    fn an_unloaded_world_is_refused() {
        assert!(world_unloaded(Some("breeders-x4")));
        assert!(!world_unloaded(Some("none")));
        assert!(!world_unloaded(Some(" none ")));
        assert!(!world_unloaded(Some("")));
        assert!(!world_unloaded(None));
    }

    fn config_test_dir() -> std::path::PathBuf {
        let nonce = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)
            .unwrap().as_nanos();
        let root = std::env::temp_dir().join(format!("borg-config-{}-{nonce}", std::process::id()));
        std::fs::create_dir(&root).unwrap();
        root
    }

    #[test]
    fn missing_config_is_absent_but_read_and_utf8_errors_are_not() {
        let root = config_test_dir();
        let path = root.join("borg.txt");
        assert_eq!(read_borg_txt(&path).unwrap(), "");
        assert_eq!(borg_txt_setting_at(&path, "borg_reborn_core"), None);
        std::fs::write(&path, "# comment\nborg_reborn_core = 1\n").unwrap();
        assert_eq!(borg_txt_setting_at(&path, "borg_reborn_core"), Some("1".into()));
        std::fs::write(&path, b"borg_reborn_core = 1\n# \xff").unwrap();
        let error = read_borg_txt(&path).unwrap_err();
        assert!(error.contains("borg.txt") && error.contains("UTF-8"));
        assert!(read_borg_txt(&root).unwrap_err().contains("cannot read configuration"));
        std::fs::remove_dir_all(root).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn config_paths_preserve_non_utf8_os_bytes() {
        use std::os::unix::ffi::OsStrExt;
        let root = config_test_dir();
        let path = root.join(std::ffi::OsStr::from_bytes(b"user-\xff"));
        let decoded = borg_txt_path(path.as_os_str().as_bytes().to_vec()).unwrap();
        assert_eq!(decoded, path);
        std::fs::write(&path, "borg_reborn_core=0").unwrap();
        assert_eq!(borg_txt_setting_at(&decoded, "borg_reborn_core"), Some("0".into()));
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn config_file_is_checked_again_on_reentry() {
        if let Some(path) = std::env::var_os("BORG_CONFIG_TEST_PATH") {
            let path = std::path::PathBuf::from(path);
            assert_eq!(borg_txt_setting_at(&path, "borg_reborn_core"), Some("1".into()));
            std::fs::write(&path, b"# \xff").unwrap();
            borg_txt_setting_at(&path, "borg_reborn_core");
            panic!("an invalid reread silently used defaults");
        }
        let root = config_test_dir();
        let path = root.join("borg.txt");
        std::fs::write(&path, "borg_reborn_core=1").unwrap();
        let result = std::process::Command::new(std::env::current_exe().unwrap())
            .args(["--exact", "config_file::tests::config_file_is_checked_again_on_reentry", "--nocapture"])
            .env("BORG_CONFIG_TEST_PATH", &path).output().unwrap();
        assert_eq!(result.status.code(), Some(2), "{:?}", result);
        assert!(String::from_utf8_lossy(&result.stderr).contains("cannot read configuration"));
        std::fs::remove_dir_all(root).unwrap();
    }
}
