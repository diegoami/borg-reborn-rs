//! The parameter table and profiles (ARCHITECTURE.md *Cross-cutting*; task
//! T2.4, #15).
//!
//! Every number a strategy decides with is a named parameter with a unit,
//! a range, a default and the evidence for that default. A profile is a
//! file of `name = value` lines in `borg.txt`'s syntax (`#` starts a
//! comment), read when the Borg starts (`BORG_PROFILE=FILE`); a name it
//! does not set keeps its default. The profile's SHA-256 goes into every
//! run record: the trace, `params.json` and run.sh's `config.txt`.

use serde::Serialize;
use sha2::{Digest, Sha256};

/// One entry of the table.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct Param {
    pub name: &'static str,
    pub unit: &'static str,
    pub min: i64,
    pub max: i64,
    pub default: i64,
    /// The principle or evidence behind the default.
    pub evidence: &'static str,
}

/// The minimal Borg's parameters (T1.5, a placeholder until M3).
pub const TABLE: [Param; 3] = [
    Param {
        name: "max_tries_per_turn",
        unit: "thinks at one game turn",
        min: 1,
        max: 8,
        default: 2,
        evidence: "GOALS principle 6 (no loop without game time): after a refused rest one walk \
                   may also be refused (a wall or a monster), then the Borg takes an action that \
                   always uses time (M1 review, #42)",
    },
    Param {
        name: "rest_below_hp_pct",
        unit: "percent of maximum hit points",
        min: 0,
        max: 100,
        default: 70,
        evidence: "T1.5 placeholder: rest when below 70% of maximum hit points; chosen without \
                   measurement, replaced by M3's strategy",
    },
    Param {
        name: "rest_turns",
        unit: "player turns",
        min: 1,
        max: 9999,
        default: 75,
        evidence: "the C Borg's rest command, 'R' '7' '5' Enter (spec §2.6)",
    },
];

/// The values a run uses, and where they came from.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Params {
    pub max_tries_per_turn: u32,
    pub rest_below_hp_pct: i32,
    pub rest_turns: u32,
    /// The profile file, or "default" when none was given.
    pub profile: String,
    /// SHA-256 of the profile file's bytes, or of the empty string for the
    /// defaults.
    pub profile_sha256: String,
}

fn default_of(name: &str) -> i64 {
    TABLE
        .iter()
        .find(|p| p.name == name)
        .map_or(0, |p| p.default)
}

impl Default for Params {
    fn default() -> Self {
        Params {
            max_tries_per_turn: default_of("max_tries_per_turn") as u32,
            rest_below_hp_pct: default_of("rest_below_hp_pct") as i32,
            rest_turns: default_of("rest_turns") as u32,
            profile: "default".into(),
            profile_sha256: sha256_hex(b""),
        }
    }
}

pub fn sha256_hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

impl Params {
    /// Read a profile. Unknown names, malformed lines and values outside
    /// their range are errors: a profile that does not say what it means
    /// must not run.
    pub fn from_profile(name: &str, bytes: &[u8]) -> Result<Params, String> {
        let text = std::str::from_utf8(bytes).map_err(|e| format!("{name}: not UTF-8: {e}"))?;
        let mut p = Params {
            profile: name.into(),
            profile_sha256: sha256_hex(bytes),
            ..Params::default()
        };
        for (i, raw) in text.lines().enumerate() {
            let line = raw.split('#').next().unwrap_or("").trim();
            if line.is_empty() {
                continue;
            }
            let (key, value) = line
                .split_once('=')
                .ok_or_else(|| format!("{name}:{}: expected 'name = value'", i + 1))?;
            let (key, value) = (key.trim(), value.trim());
            let param = TABLE
                .iter()
                .find(|p| p.name == key)
                .ok_or_else(|| format!("{name}:{}: unknown parameter '{key}'", i + 1))?;
            let v: i64 = value
                .parse()
                .map_err(|_| format!("{name}:{}: '{value}' is not an integer", i + 1))?;
            if v < param.min || v > param.max {
                return Err(format!(
                    "{name}:{}: {key} = {v} is outside {}..={}",
                    i + 1,
                    param.min,
                    param.max
                ));
            }
            match key {
                "max_tries_per_turn" => p.max_tries_per_turn = v as u32,
                "rest_below_hp_pct" => p.rest_below_hp_pct = v as i32,
                "rest_turns" => p.rest_turns = v as u32,
                _ => unreachable!("every table entry has a field"),
            }
        }
        Ok(p)
    }

    /// The table with the values in use: what `params.json` holds.
    pub fn table(&self) -> serde_json::Value {
        let value = |name: &str| -> i64 {
            match name {
                "max_tries_per_turn" => i64::from(self.max_tries_per_turn),
                "rest_below_hp_pct" => i64::from(self.rest_below_hp_pct),
                "rest_turns" => i64::from(self.rest_turns),
                _ => 0,
            }
        };
        serde_json::json!({
            "profile": self.profile,
            "profile_sha256": self.profile_sha256,
            "parameters": TABLE.iter().map(|p| serde_json::json!({
                "name": p.name, "unit": p.unit, "min": p.min, "max": p.max,
                "default": p.default, "value": value(p.name), "evidence": p.evidence,
            })).collect::<Vec<_>>(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_come_from_the_table() {
        let p = Params::default();
        assert_eq!(
            (p.max_tries_per_turn, p.rest_below_hp_pct, p.rest_turns),
            (2, 70, 75)
        );
        assert_eq!(
            p.profile_sha256,
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
        for t in TABLE {
            assert!(t.min <= t.default && t.default <= t.max, "{}", t.name);
            assert!(!t.evidence.is_empty() && !t.unit.is_empty(), "{}", t.name);
        }
    }

    #[test]
    fn a_profile_sets_what_it_names() {
        let text = b"# a test profile\nrest_turns = 50   # shorter rests\n\nrest_below_hp_pct=60\n";
        let p = Params::from_profile("t.txt", text).unwrap();
        assert_eq!(
            (p.max_tries_per_turn, p.rest_below_hp_pct, p.rest_turns),
            (2, 60, 50)
        );
        assert_eq!(p.profile_sha256, sha256_hex(text));
        let t = p.table();
        assert_eq!(t["parameters"][2]["value"], 50);
        assert_eq!(t["parameters"][2]["default"], 75);
    }

    #[test]
    fn a_bad_profile_is_refused() {
        for (text, why) in [
            (&b"rest_turn = 5\n"[..], "unknown parameter"),
            (b"rest_turns 5\n", "expected"),
            (b"rest_turns = five\n", "not an integer"),
            (b"rest_below_hp_pct = 101\n", "outside"),
        ] {
            let e = Params::from_profile("bad.txt", text).unwrap_err();
            assert!(e.contains(why), "{e}");
        }
    }
}
