//! borg-reborn-rs's errands over this core (#208, docs/REDESIGN.md
//! *Errands*). Not part of borg-rs: this file and the hook calls marked
//! `borg-reborn-rs (#208)` are the shell's whole footprint in the vendored
//! core, so borg-rs's later changes still apply to the rest as plain
//! patches.
//!
//! The errand is **trips** (owner, 2026-11-14): explore DL1 and go back to
//! town; then explore DL1, DL2, DL1 and go back to town; and so on. The
//! player writes the plan; a class or a power level calls for its own
//! (a mage repeats town-DL1-town longer, a warrior goes deeper sooner).
//!
//! - **On a level of the plan the core plays as itself.** The level is
//!   explored when the core decides to leave it (`borg_leave_level`); the
//!   trip then goes to its next level, or home.
//! - **Between levels the errand walks the stairs**, after the core's
//!   caution, with the core's own stair flows. The next trip starts by the
//!   down staircase (never a recall to the maximum depth).
//! - **The core's own readiness stands.** A planned level the core is not
//!   prepared for (`borg_prepared`) is skipped and noted, never forced. A
//!   core that must go back to town to restock ends the trip there.
//! - **A trip marked `*` repeats** until the core is prepared for the
//!   deepest level of the next trip (`borg_prepared`); the last one repeats
//!   until the budget. The player writes the ladder, the core's readiness
//!   climbs it: a mage stays on town-DL1-town longer, a warrior moves on
//!   sooner.
//! - The errand ends in town when the last trip is over, or at its budget,
//!   through the core's own stop (`borg_oops`); its outcome goes to
//!   `BORG_ERRAND_OUT` (one JSON line) when that is set.
//!
//! With no errand set, nothing here changes a decision: every hook returns
//! at once, and play is borg-rs's byte for byte.

use crate::borg_headers::borg_trait_h::{BI_CDEPTH, BI_CLEVEL, BI_CURHP, BI_MAXHP};
use crate::borg_io::{borg_keypress, borg_note};
use crate::engine;
use crate::headers::list_terrain_h::{FEAT_LESS, FEAT_MORE};
use crate::headers::ui_event_h::keycode_t;
use crate::state::BorgState;
use std::sync::Mutex;

/// An errand.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Errand {
    /// Trips, in order. Each trip is the levels to explore, in order, and
    /// ends in town.
    Trips(Vec<Trip>),
}

/// One trip: the levels to explore, in order, then home.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Trip {
    pub levels: Vec<i32>,
    /// Repeat until the core is prepared for the next trip's deepest level
    /// (the last trip: until the budget).
    pub repeat: bool,
}

impl Errand {
    /// `trips:1*/1,2,1*/1,2,3,2,1` (the `BORG_ERRAND` syntax): trips
    /// separated by `/`, levels by `,`, `*` after a trip to repeat it.
    pub fn parse(s: &str) -> Option<Errand> {
        let (name, arg) = s.split_once(':')?;
        if name != "trips" {
            return None;
        }
        let trips: Option<Vec<Trip>> = arg
            .split('/')
            .map(|trip| {
                let trip = trip.trim();
                let (body, repeat) = match trip.strip_suffix('*') {
                    Some(b) => (b, true),
                    None => (trip, false),
                };
                let levels: Option<Vec<i32>> = body
                    .split(',')
                    .map(|l| l.trim().parse().ok().filter(|d: &i32| *d >= 1))
                    .collect();
                levels.map(|levels| Trip { levels, repeat })
            })
            .collect();
        trips
            .filter(|t| !t.is_empty() && t.iter().all(|trip| !trip.levels.is_empty()))
            .map(Errand::Trips)
    }
}

/// Where a trips errand stands.
#[derive(Clone, Debug, PartialEq, Eq)]
struct Running {
    trips: Vec<Trip>,
    /// The trip under way.
    trip: usize,
    /// The level of that trip to explore next; past the end, home.
    step: usize,
    /// Game turns the errand may run; 0 for no budget.
    budget: i64,
    /// The game turn of the first think under the errand.
    started: Option<i64>,
    /// Levels skipped because the core was not prepared for them.
    skipped: u32,
}

impl Running {
    /// The depth the errand wants now: a level of the trip, 0 for home, or
    /// `None` when every trip is over.
    fn target(&self) -> Option<i32> {
        let trip = self.trips.get(self.trip)?;
        Some(trip.levels.get(self.step).copied().unwrap_or(0))
    }
}

static ERRAND: Mutex<Option<Running>> = Mutex::new(None);

fn running() -> std::sync::MutexGuard<'static, Option<Running>> {
    ERRAND.lock().unwrap_or_else(|e| e.into_inner())
}

/// Set (or clear) the errand, with a budget in game turns (0: none).
pub fn set_errand(errand: Option<Errand>, budget: i64) {
    *running() = errand.map(|Errand::Trips(trips)| Running {
        trips,
        trip: 0,
        step: 0,
        budget,
        started: None,
        skipped: 0,
    });
}

/// Whether an errand is running.
pub fn active() -> bool {
    running().is_some()
}

/// The target depth now, skipping the planned levels the core is not
/// prepared for.
fn target(st: &mut BorgState) -> Option<i32> {
    loop {
        let t = running().as_ref()?.target()?;
        if t == 0 || crate::borg_prepared::borg_prepared(st, t).is_none() {
            return Some(t);
        }
        borg_note(st, &format!("# Errand: not prepared for DL{t}, skipped."));
        if let Some(r) = running().as_mut() {
            r.step += 1;
            r.skipped += 1;
        }
    }
}

/// Before the dungeon think: in town, a trip that came home is over, and the
/// next begins; the errand ends when the last is over, or at its budget.
/// `Some(true)` means the think is over (the Borg stops).
pub(crate) fn before_dungeon_think(st: &mut BorgState) -> Option<bool> {
    let turn = i64::from(engine::game_turn());
    let depth = st.borg.tr(BI_CDEPTH);
    // Home after a trip: what comes next (the readiness check runs outside
    // the lock: `borg_prepared` reads the core's state only).
    let next_deepest = {
        let mut guard = running();
        let r = guard.as_mut()?;
        r.started.get_or_insert(turn);
        if depth == 0 && r.target() == Some(0) {
            r.step = 0;
            if r.trips[r.trip].repeat {
                r.trips
                    .get(r.trip + 1)
                    .map(|next| next.levels.iter().copied().max().unwrap_or(1))
            } else {
                r.trip += 1;
                None
            }
        } else {
            None
        }
    };
    if let Some(deepest) = next_deepest {
        // A repeating trip moves on once the core is ready for the next.
        if crate::borg_prepared::borg_prepared(st, deepest).is_none() {
            if let Some(r) = running().as_mut() {
                r.trip += 1;
            }
        }
    }
    let outcome = {
        let guard = running();
        let r = guard.as_ref()?;
        let started = r.started.unwrap_or(turn);
        if r.target().is_none() {
            "done"
        } else if r.budget > 0 && turn - started >= r.budget {
            "budget"
        } else {
            return None;
        }
    };
    report(st, outcome, turn);
    set_errand(None, 0);
    crate::borg_think::borg_oops(st, &format!("errand {outcome}"));
    Some(true)
}

/// The core decided to leave the level (`borg_leave_level`): the level is
/// explored. The trip moves on, and the errand walks toward its next level.
/// `None` lets the core decide as usual.
pub(crate) fn on_leave_level(st: &mut BorgState, bored: bool) -> Option<bool> {
    if !active() {
        return None;
    }
    let depth = st.borg.tr(BI_CDEPTH);
    if depth == 0 {
        // Town: the core shops until it is bored, then the trip starts by
        // the stairs.
        return Some(bored && stairs_down(st));
    }
    let restock = crate::borg_prepared::borg_must_return_to_town(st).is_some();
    if let Some(r) = running().as_mut() {
        if restock {
            // The core must restock: the trip ends here, at home.
            r.step = usize::MAX;
        } else if r.target() == Some(depth) {
            r.step += 1;
        }
    }
    if restock {
        borg_note(st, "# Errand: must restock, the trip goes home.");
    }
    let t = target(st)?;
    borg_note(st, &format!("# Errand: DL{depth} explored, next DL{t}."));
    Some(walk(st, depth, t))
}

/// `GOAL_FLEE` (borg-flow.h), as the core's think modules declare it.
const GOAL_FLEE: i32 = 7;

/// After the core's caution: away from the target depth, walk the stairs
/// toward it. `true` when a step was queued.
pub(crate) fn steer(st: &mut BorgState) -> bool {
    if !active() {
        return false;
    }
    let depth = st.borg.tr(BI_CDEPTH);
    let Some(t) = target(st) else {
        return false;
    };
    // In town the core shops and leaves by itself (`on_leave_level`); on a
    // level of the plan it explores.
    if depth == 0 || depth == t {
        return false;
    }
    walk(st, depth, t)
}

/// One step of the stairs from `depth` toward `t`, with the core's own
/// stair flows (the Lunal power-dive's steps, `borg_think_dungeon.rs`).
fn walk(st: &mut BorgState, depth: i32, t: i32) -> bool {
    if t > depth {
        stairs_down(st)
    } else if t < depth {
        stairs_up(st)
    } else {
        false
    }
}

fn underfoot(st: &BorgState) -> u32 {
    let (x, y) = (st.borg.c.x, st.borg.c.y);
    u32::from(st.cave.grid(x, y).feat)
}

fn stairs_down(st: &mut BorgState) -> bool {
    if underfoot(st) == FEAT_MORE {
        borg_keypress(st, keycode_t::from(b'>'));
        return true;
    }
    crate::borg_flow::borg_flow_old(st, GOAL_FLEE)
        || crate::borg_flow_stairs::borg_flow_stair_more(st, GOAL_FLEE, false, false)
}

fn stairs_up(st: &mut BorgState) -> bool {
    if underfoot(st) == FEAT_LESS {
        borg_keypress(st, keycode_t::from(b'<'));
        return true;
    }
    crate::borg_flow::borg_flow_old(st, GOAL_FLEE)
        || crate::borg_flow_stairs::borg_flow_stair_less(st, GOAL_FLEE, false)
}

/// The outcome line (`BORG_ERRAND_OUT`), if asked for.
fn report(st: &BorgState, outcome: &str, turn: i64) {
    let Some(path) = std::env::var_os("BORG_ERRAND_OUT") else {
        return;
    };
    let (trips, skipped) = running()
        .as_ref()
        .map_or((0, 0), |r| (r.trip.min(r.trips.len()), r.skipped));
    let line = format!(
        "{{\"errand\":\"trips\",\"outcome\":\"{outcome}\",\"turn\":{turn},\"trips_done\":{trips},\"skipped\":{skipped},\"depth\":{},\"clevel\":{},\"chp\":{},\"mhp\":{}}}\n",
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
    fn trips_parse() {
        assert_eq!(
            Errand::parse("trips:1*/1,2,1"),
            Some(Errand::Trips(vec![
                Trip {
                    levels: vec![1],
                    repeat: true
                },
                Trip {
                    levels: vec![1, 2, 1],
                    repeat: false
                },
            ]))
        );
        assert_eq!(Errand::parse("trips:1,0"), None, "depth 0 is not a level");
        assert_eq!(Errand::parse("trips:"), None);
        assert_eq!(Errand::parse("dive:5"), None);
    }

    #[test]
    fn a_trip_walks_its_levels_then_home_then_the_next_trip() {
        let trip = |levels: Vec<i32>| Trip {
            levels,
            repeat: false,
        };
        let mut r = Running {
            trips: vec![trip(vec![1]), trip(vec![1, 2, 1])],
            trip: 0,
            step: 0,
            budget: 0,
            started: None,
            skipped: 0,
        };
        assert_eq!(r.target(), Some(1));
        r.step = 1;
        assert_eq!(r.target(), Some(0), "the trip goes home");
        r.trip = 1;
        r.step = 0;
        assert_eq!(r.target(), Some(1));
        r.step = 1;
        assert_eq!(r.target(), Some(2));
        r.step = 3;
        assert_eq!(r.target(), Some(0));
        r.trip = 2;
        assert_eq!(r.target(), None, "every trip is over");
    }
}
