//! Captured frontend/calendar environment. Display connection and recorder
//! control paths are caller-selected; behavior settings and timezone are sealed.
use crate::{exclusive, invalid, read_regular};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Environment {
    variables: BTreeMap<String, String>,
    timezone: Timezone,
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "kind", deny_unknown_fields)]
enum Timezone {
    File { sha256: String },
    Posix { spec: String },
}
fn relevant(name: &str) -> bool {
    matches!(
        name,
        "TZ" | "TZDIR"
            | "PARSE_ERROR_LIMIT"
            | "LANG"
            | "LC_ALL"
            | "LC_CTYPE"
            | "BORG_TOOLKIT_HEADLESS"
            | "BORG_VISIBLE_SEED"
            | "BORG_NO_EXTRA_OBSERVATIONS"
            | "BORG_REPLAY_SYS"
    ) || name.starts_with("ANGBAND_X11_")
        || matches!(
            name,
            "BORG_REPLAY_WIDTH"
                | "BORG_REPLAY_HEIGHT"
                | "BORG_REPLAY_KEY_SIZE"
                | "BORG_REPLAY_SOFT_CURSOR"
                | "BORG_REPLAY_COMPLEX_INPUT"
        )
}
impl Environment {
    pub(crate) fn capture(output: &Path) -> io::Result<Self> {
        let mut variables = BTreeMap::new();
        for (key, value) in std::env::vars_os() {
            if let Some(key) = key.to_str().filter(|k| relevant(k)) {
                variables.insert(
                    key.to_string(),
                    value
                        .into_string()
                        .map_err(|_| invalid("non-UTF8 frontend environment"))?,
                );
            }
        }
        let tz = variables.get("TZ");
        // libc resolves named zones through TZDIR; the captured value, not the
        // ambient default, decides which file the recording depends on.
        let zone_dir = variables
            .get("TZDIR")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("/usr/share/zoneinfo"));
        let path = match tz {
            None => Some(PathBuf::from("/etc/localtime")),
            Some(value) if value.is_empty() => None,
            Some(value) => {
                let value = value.strip_prefix(':').unwrap_or(value);
                let path = if Path::new(value).is_absolute() {
                    PathBuf::from(value)
                } else {
                    zone_dir.join(value)
                };
                if path.is_file() {
                    Some(path)
                } else if value.contains('/') || value.starts_with('.') {
                    return Err(invalid("timezone file is unavailable"));
                } else {
                    None
                }
            }
        };
        let timezone = if let Some(path) = path {
            // System timezone links are inputs, not output paths: capture their bytes.
            let data = fs::read(path)?;
            exclusive(&output.join("timezone.tzif"))?.write_all(&data)?;
            Timezone::File {
                sha256: format!("{:x}", Sha256::digest(data)),
            }
        } else {
            Timezone::Posix {
                spec: tz.cloned().unwrap_or_default(),
            }
        };
        let value = Self {
            variables,
            timezone,
        };
        serde_json::to_writer(exclusive(&output.join("environment.json"))?, &value)?;
        Ok(value)
    }
    pub(crate) fn load(root: &Path) -> io::Result<Self> {
        let bytes = read_regular(&root.join("environment.json"))?;
        if bytes.len() > 1024 * 1024 {
            return Err(invalid("oversized environment metadata"));
        }
        let value: Self = serde_json::from_slice(&bytes)?;
        if value.variables.keys().any(|k| !relevant(k)) {
            return Err(invalid("unexpected captured environment key"));
        }
        match &value.timezone {
            Timezone::File { sha256 } => {
                let data = read_regular(&root.join("timezone.tzif"))?;
                if *sha256 != format!("{:x}", Sha256::digest(data)) {
                    return Err(invalid("timezone data differs"));
                }
            }
            Timezone::Posix { spec } => {
                if spec.contains('/')
                    || spec.as_bytes().contains(&0)
                    || fs::symlink_metadata(root.join("timezone.tzif")).is_ok()
                {
                    return Err(invalid("invalid POSIX timezone metadata"));
                }
            }
        }
        if value.variables.values().any(|s| s.as_bytes().contains(&0)) {
            return Err(invalid("invalid environment value"));
        }
        Ok(value)
    }
    pub(crate) fn copy_to(&self, source: &Path, output: &Path) -> io::Result<()> {
        exclusive(&output.join("environment.json"))?
            .write_all(&read_regular(&source.join("environment.json"))?)?;
        if matches!(self.timezone, Timezone::File { .. }) {
            exclusive(&output.join("timezone.tzif"))?
                .write_all(&read_regular(&source.join("timezone.tzif"))?)?;
        }
        Ok(())
    }
    pub(crate) fn has_timezone_file(&self) -> bool {
        matches!(self.timezone, Timezone::File { .. })
    }
    #[cfg(feature = "ffi")]
    pub(crate) fn apply(&self, root: &Path) -> io::Result<()> {
        // Called at the first engine main entry, before frontend threads/locale.
        for (name, _) in std::env::vars_os() {
            if name.to_str().is_some_and(relevant) {
                std::env::remove_var(name);
            }
        }
        for (name, value) in &self.variables {
            std::env::set_var(name, value);
        }
        match &self.timezone {
            Timezone::File { .. } => {
                let path = fs::canonicalize(root.join("timezone.tzif"))?;
                let mut spec = std::ffi::OsString::from(":");
                spec.push(path);
                std::env::set_var("TZ", spec);
            }
            Timezone::Posix { spec } => std::env::set_var("TZ", spec),
        }
        unsafe {
            extern "C" {
                fn tzset();
            }
            tzset();
        }
        Ok(())
    }
}
