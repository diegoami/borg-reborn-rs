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

/// The first strategy's parameters (M3: modes, budgets and town reserves).
pub const TABLE: [Param; 13] = [
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
        evidence: "T1.5 placeholder, kept by the first strategy: rest when below 70% of maximum \
                   hit points; the strategy-manual's REST rules are the candidate replacement \
                   (c15 REST-01, to be measured, RULES 14)",
    },
    Param {
        name: "rest_turns",
        unit: "player turns",
        min: 1,
        max: 9999,
        default: 75,
        evidence: "the C Borg's rest command, 'R' '7' '5' Enter (spec §2.6)",
    },
    Param {
        name: "explore_budget_turns",
        unit: "game turns without progress",
        min: 100,
        max: 50000,
        default: 2000,
        evidence: "GOALS principle 6 (a budget in game turns): a level that has revealed nothing \
                   new for this long fails its explore goal. The scale is a manual-reading \
                   player's ~100 decisions on one level (borg-runner ACCUMULATED_KNOWLEDGE §1, \
                   five runs); a placeholder until the zoo measures it (RULES 14)",
    },
    Param {
        name: "goal_cooldown_turns",
        unit: "game turns",
        min: 0,
        max: 10000,
        default: 100,
        evidence: "GOALS principle 6 (a failed goal is blocked for a while): short enough that \
                   the mode is retried within a level's explore budget, long enough to break \
                   one-goal oscillation; a placeholder until the zoo measures it (RULES 14)",
    },
    Param {
        name: "max_level_changes_per_turn",
        unit: "Ascend/Descend actions at one game turn",
        min: 1,
        max: 8,
        default: 2,
        evidence: "M3 review PR #135, blocking finding 1: one up/down reroll per game turn at \
                   most, so level generation cannot loop for free. A separate per-level reroll \
                   limit prevents repeated exits from the same generation",
    },
    Param {
        name: "reserve_food",
        unit: "rations in the pack",
        min: 0,
        max: 20,
        default: 3,
        evidence: "GOALS principle 7 and the strategy manual's CL1 shopping list (c15 §12.4: \
                   3-5 rations); the lower bound is the M3 default, to be measured (RULES 14)",
    },
    Param {
        name: "reserve_light",
        unit: "spare torches in the pack",
        min: 0,
        max: 10,
        default: 2,
        evidence: "GOALS principle 7 and c15 §12.4 (2-3 spare torches); the equipped light is \
                   not a reserve. The lower bound is the M3 default, to be measured (RULES 14)",
    },
    Param {
        name: "reserve_cure",
        unit: "Cure Light Wounds items in the pack",
        min: 0,
        max: 20,
        default: 3,
        evidence: "GOALS principle 7 and the owner's verified D0015 re-shop (3 CLW); c15 §12.4 \
                   independently recommends 3-5 (RULES 14)",
    },
    Param {
        name: "reserve_escape",
        unit: "Phase Door scrolls in the pack",
        min: 0,
        max: 20,
        default: 4,
        evidence: "GOALS principle 7 and the owner's verified D0015 re-shop (4 Phase Door); \
                   survival stock outranks equipment (bbc D0013, D0015, D0018, D0019)",
    },
    Param {
        name: "reserve_recall",
        unit: "Word of Recall scrolls in the pack",
        min: 0,
        max: 5,
        default: 1,
        evidence: "GOALS principle 7 (a way home): keep the starting Recall, unlike bbc D0006, \
                   D0009, D0015, D0016, D0018 and D0019; c15 §12.4 says one if affordable",
    },
    Param {
        name: "risk_aversion_pct",
        unit: "percent of the expected-to-worst gap",
        min: 0,
        max: 100,
        default: 50,
        evidence: "ARCHITECTURE L4: one risk tolerance for every consumer (paths, rest, retreat, \
                   combat). 50 is the declared midpoint between expected and worst damage, not a \
                   measured optimum; the calibration report (#124) replaces it",
    },
    Param {
        name: "threat_horizon",
        unit: "player turns",
        min: 1,
        max: 5,
        default: 1,
        evidence: "ARCHITECTURE L4: the assessment's horizon in player turns. 1 is the shortest \
                   horizon in which a normal-speed adjacent monster acts; longer horizons are \
                   the calibration report's (#124), not a tuned value",
    },
];

/// The values a run uses, and where they came from.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Params {
    pub max_tries_per_turn: u32,
    pub rest_below_hp_pct: i32,
    pub rest_turns: u32,
    pub explore_budget_turns: u32,
    pub goal_cooldown_turns: u32,
    pub max_level_changes_per_turn: u32,
    pub reserve_food: u32,
    pub reserve_light: u32,
    pub reserve_cure: u32,
    pub reserve_escape: u32,
    pub reserve_recall: u32,
    pub risk_aversion_pct: i32,
    pub threat_horizon: i32,
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
            explore_budget_turns: default_of("explore_budget_turns") as u32,
            goal_cooldown_turns: default_of("goal_cooldown_turns") as u32,
            max_level_changes_per_turn: default_of("max_level_changes_per_turn") as u32,
            reserve_food: default_of("reserve_food") as u32,
            reserve_light: default_of("reserve_light") as u32,
            reserve_cure: default_of("reserve_cure") as u32,
            reserve_escape: default_of("reserve_escape") as u32,
            reserve_recall: default_of("reserve_recall") as u32,
            risk_aversion_pct: default_of("risk_aversion_pct") as i32,
            threat_horizon: default_of("threat_horizon") as i32,
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
                "explore_budget_turns" => p.explore_budget_turns = v as u32,
                "goal_cooldown_turns" => p.goal_cooldown_turns = v as u32,
                "max_level_changes_per_turn" => p.max_level_changes_per_turn = v as u32,
                "reserve_food" => p.reserve_food = v as u32,
                "reserve_light" => p.reserve_light = v as u32,
                "reserve_cure" => p.reserve_cure = v as u32,
                "reserve_escape" => p.reserve_escape = v as u32,
                "reserve_recall" => p.reserve_recall = v as u32,
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
                "explore_budget_turns" => i64::from(self.explore_budget_turns),
                "goal_cooldown_turns" => i64::from(self.goal_cooldown_turns),
                "max_level_changes_per_turn" => i64::from(self.max_level_changes_per_turn),
                "reserve_food" => i64::from(self.reserve_food),
                "reserve_light" => i64::from(self.reserve_light),
                "reserve_cure" => i64::from(self.reserve_cure),
                "reserve_escape" => i64::from(self.reserve_escape),
                "reserve_recall" => i64::from(self.reserve_recall),
                "risk_aversion_pct" => i64::from(self.risk_aversion_pct),
                "threat_horizon" => i64::from(self.threat_horizon),
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
        assert_eq!((p.explore_budget_turns, p.goal_cooldown_turns), (2000, 100));
        assert_eq!(p.max_level_changes_per_turn, 2);
        assert_eq!((p.risk_aversion_pct, p.threat_horizon), (50, 1));
        assert_eq!(
            (
                p.reserve_food,
                p.reserve_light,
                p.reserve_cure,
                p.reserve_escape,
                p.reserve_recall
            ),
            (3, 2, 3, 4, 1)
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
        let text = b"# a test profile\nrest_turns = 50   # shorter rests\n\nrest_below_hp_pct=60\nreserve_escape=6\n";
        let p = Params::from_profile("t.txt", text).unwrap();
        assert_eq!(
            (p.max_tries_per_turn, p.rest_below_hp_pct, p.rest_turns),
            (2, 60, 50)
        );
        assert_eq!(p.reserve_escape, 6);
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
