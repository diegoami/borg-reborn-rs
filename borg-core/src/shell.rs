//! borg-reborn-rs's errands over this core (#208, docs/REDESIGN.md
//! *Errands*). Not part of borg-rs: this file and the hook calls marked
//! `borg-reborn-rs (#208)` are the shell's whole footprint in the vendored
//! core, so borg-rs's later changes still apply to the rest as plain
//! patches.
//!
//! An errand is a goal with a stop predicate and a budget. The core plays
//! under its own rules; the errand changes what it aims for, and ends the
//! play (`borg_oops`, the core's own auto-stop, as `BORG_STOP_DLEVEL` does)
//! when the predicate holds or the budget is spent. The headless frontend
//! then ends the run with `stop_reason: "borg_stopped"`; the errand's
//! outcome goes to `BORG_ERRAND_OUT` (one JSON line) when that is set.
//!
//! With no errand set, nothing here changes a decision: the hooks return
//! at once, and play is borg-rs's byte for byte.

use crate::borg_headers::borg_trait_h::{BI_CDEPTH, BI_CLEVEL, BI_CURHP, BI_MAXHP};
use crate::engine;
use crate::state::BorgState;
use std::sync::Mutex;

/// An errand.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Errand {
    /// Go down to depth `N` as fast as the core's own diving allows: every
    /// depth down to `N` counts as prepared (`borg_prepared`), so the core's
    /// power-dive takes the stairs. Its caution, escapes and fleeing still
    /// run. Ends at depth `N`.
    DiveTo(i32),
}

impl Errand {
    /// `dive:N` (the `BORG_ERRAND` syntax).
    pub fn parse(s: &str) -> Option<Errand> {
        let (name, arg) = s.split_once(':').unwrap_or((s, ""));
        match name {
            "dive" => arg.parse().ok().filter(|n| *n >= 1).map(Errand::DiveTo),
            _ => None,
        }
    }
}

struct Running {
    errand: Errand,
    /// Game turns the errand may run; 0 for no budget.
    budget: i64,
    /// The game turn of the first think under the errand.
    started: Option<i64>,
}

static ERRAND: Mutex<Option<Running>> = Mutex::new(None);

/// Set (or clear) the errand, with a budget in game turns (0: none).
pub fn set_errand(errand: Option<Errand>, budget: i64) {
    *ERRAND.lock().unwrap_or_else(|e| e.into_inner()) = errand.map(|errand| Running {
        errand,
        budget,
        started: None,
    });
}

/// The errand running now.
pub fn errand() -> Option<Errand> {
    ERRAND
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .as_ref()
        .map(|r| r.errand)
}

/// `borg_prepared`'s override: a depth the dive errand asks for counts as
/// prepared.
pub(crate) fn prepared_override(depth: i32) -> bool {
    matches!(errand(), Some(Errand::DiveTo(n)) if depth <= n)
}

/// Before the dungeon think: end the errand when its predicate holds or its
/// budget is spent. `Some(true)` means the think is over (the Borg stops).
pub(crate) fn before_dungeon_think(st: &mut BorgState) -> Option<bool> {
    let turn = i64::from(engine::game_turn());
    let (errand, outcome) = {
        let mut guard = ERRAND.lock().unwrap_or_else(|e| e.into_inner());
        let running = guard.as_mut()?;
        let started = *running.started.get_or_insert(turn);
        let outcome = match running.errand {
            Errand::DiveTo(n) if st.borg.tr(BI_CDEPTH) >= n => Some("done"),
            _ if running.budget > 0 && turn - started >= running.budget => Some("budget"),
            _ => None,
        };
        (running.errand, outcome?)
    };
    report(st, errand, outcome, turn);
    set_errand(None, 0);
    crate::borg_think::borg_oops(st, &format!("errand {outcome}: {errand:?}"));
    Some(true)
}

/// The outcome line (`BORG_ERRAND_OUT`), if asked for.
fn report(st: &BorgState, errand: Errand, outcome: &str, turn: i64) {
    let Some(path) = std::env::var_os("BORG_ERRAND_OUT") else {
        return;
    };
    let line = format!(
        "{{\"errand\":\"{errand:?}\",\"outcome\":\"{outcome}\",\"turn\":{turn},\"depth\":{},\"clevel\":{},\"chp\":{},\"mhp\":{}}}\n",
        st.borg.tr(BI_CDEPTH),
        st.borg.tr(BI_CLEVEL),
        st.borg.tr(BI_CURHP),
        st.borg.tr(BI_MAXHP),
    );
    use std::io::Write;
    if let Ok(mut f) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
    {
        let _ = f.write_all(line.as_bytes());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn errands_parse() {
        assert_eq!(Errand::parse("dive:5"), Some(Errand::DiveTo(5)));
        assert_eq!(Errand::parse("dive:0"), None);
        assert_eq!(Errand::parse("dive"), None);
        assert_eq!(Errand::parse("shop"), None);
    }

    #[test]
    fn the_dive_overrides_readiness_only_down_to_its_depth() {
        set_errand(Some(Errand::DiveTo(4)), 0);
        assert!(prepared_override(4));
        assert!(!prepared_override(5));
        set_errand(None, 0);
        assert!(!prepared_override(2), "no errand, no override");
    }
}
