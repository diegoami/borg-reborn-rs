//! The decision trace (ARCHITECTURE.md *Cross-cutting*; task T2.4, #15).
//!
//! One structured line per think: the game turn, the mode, the goal and
//! its budget, the candidate actions with their scores, the choice, the
//! invariant violations seen, and the profile's hash. It is written after
//! the decision, from data the decision already made, so it never changes
//! play. Run outputs keep it (`trace.jsonl`) and death dossiers keep its
//! end (`trace-tail.jsonl`).

use serde::{Deserialize, Serialize};

/// One think.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TraceLine {
    pub turn: i64,
    /// Thinks since the Borg started.
    pub think: u64,
    /// The strategy's mode. "placeholder" for the minimal Borg until M3.
    pub mode: String,
    pub goal: String,
    /// What is left of the goal's budget, in its own unit.
    pub budget: Option<Budget>,
    /// The actions considered, best first, with their scores.
    pub candidates: Vec<Candidate>,
    pub choice: String,
    /// The threat assessment at the character's grid (T4.2); absent in
    /// traces written before it existed.
    #[serde(default)]
    pub threat: Option<crate::danger::ThreatSummary>,
    /// Invariants that did not hold at this think (GOALS principles).
    pub violations: Vec<String>,
    /// Prompts the catcher answered between the last think and this one
    /// (T3.1); empty until then, and in older traces.
    #[serde(default)]
    pub prompts: Vec<String>,
    pub profile_sha256: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Budget {
    pub unit: String,
    pub left: i64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Candidate {
    pub action: String,
    pub score: f64,
}

impl TraceLine {
    pub fn to_line(&self) -> String {
        serde_json::to_string(self).expect("a trace line is plain data")
    }

    pub fn from_line(line: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(line)
    }
}
