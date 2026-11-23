//! borg-reborn-rs (#1153): a general movement-loop detector, the safety net
//! under the guards that each cover one pair of decisions (#1108, #1133,
//! #1152, #932).
//!
//! Every dungeon think records the Borg's grid when it moved, with how many
//! grids of the map it knows. A loop is `LOOP_MOVES` moves among at most
//! `LOOP_GRIDS` grids while the map learned nothing, with no danger here above
//! a quarter of the HP. `loop_breaker` then breaks it, the first that
//! applies:
//! 1. a walk to a staircase at the normal avoidance;
//! 2. a brave walk to a staircase (the avoidance boosted);
//!
//! either committed (`commit_loop_walk`), so the next thinks continue it as
//! #1130's brave walk is continued;
//! 3. for `LOOP_BAN_THINKS` thinks, no step onto the loop's grids other than
//!    a staircase (`step_banned`, read by `borg_play_step`), so whatever
//!    undid the other decision cannot.
//!
//! The note names the two decisions that alternated (the trace's choices),
//! so each new pair can still get a fix of its own.

use std::collections::VecDeque;

use crate::borg::BorgTime;
use crate::borg_cave::{AUTO_MAX_X, AUTO_MAX_Y};
use crate::borg_danger::borg_danger;
use crate::borg_flow::BraveStairs;
use crate::borg_headers::borg_trait_h::{BI_CDEPTH, BI_CURHP};
use crate::borg_io::borg_note;
use crate::headers::list_terrain_h::{FEAT_LESS, FEAT_MORE, FEAT_NONE};
use crate::headers::z_type_h::loc;
use crate::params::{value, Param};
use crate::state::BorgState;

/// The flee goal (`borg_flow.rs`: GOAL_FLEE).
const GOAL_FLEE: i32 = 7;

/// Moves of the window a loop is judged on.
pub(crate) const LOOP_MOVES: usize = 16;
/// The most distinct grids a loop's moves visit.
pub(crate) const LOOP_GRIDS: usize = 3;
/// Thinks the loop's grids stay closed after the third remedy.
const LOOP_BAN_THINKS: BorgTime = 20;

/// One recorded move: the grid stepped onto and the grids then known.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Move {
    pub at: loc,
    pub known: u32,
}

/// #1153: the detector's state, per level.
#[derive(Clone, Debug, Default)]
pub struct LoopState {
    /// The level the moves belong to (`time.level`).
    level: BorgTime,
    /// The Borg's grid at the last think.
    last: Option<loc>,
    /// The last moves, oldest first.
    moves: VecDeque<Move>,
    /// The grids closed by the third remedy, and until which think.
    ban: Vec<loc>,
    ban_until: BorgTime,
    /// The think since which the current loop runs (for the note).
    since: BorgTime,
}

/// #1153: do these moves make a loop? `LOOP_MOVES` of them, among at most
/// `LOOP_GRIDS` grids, the map unchanged from the first to the last.
pub(crate) fn is_loop(moves: &[Move]) -> bool {
    if moves.len() < LOOP_MOVES {
        return false;
    }
    let window = &moves[moves.len() - LOOP_MOVES..];
    let mut grids: Vec<(i32, i32)> = window.iter().map(|m| (m.at.y, m.at.x)).collect();
    grids.sort_unstable();
    grids.dedup();
    grids.len() <= LOOP_GRIDS && window.first().map(|m| m.known) == window.last().map(|m| m.known)
}

/// The grids of the map the Borg knows (any feature but `FEAT_NONE`).
fn known_grids(st: &BorgState) -> u32 {
    let mut n = 0u32;
    for y in 0..AUTO_MAX_Y {
        for x in 0..AUTO_MAX_X {
            if u32::from(st.cave.grid(x, y).feat) != FEAT_NONE {
                n += 1;
            }
        }
    }
    n
}

/// #1153: record this think's grid when the Borg moved (a rest is no move),
/// per level. Called at the start of every dungeon think.
pub(crate) fn observe(st: &mut BorgState) {
    if value(Param::loop_breaker) == 0 || st.borg.tr(BI_CDEPTH) == 0 {
        return;
    }
    let level = st.borg.time.level;
    if st.loops.level != level {
        st.loops = LoopState { level, ..LoopState::default() };
    }
    let here = st.borg.c;
    if st.loops.last == Some(here) {
        return;
    }
    st.loops.last = Some(here);
    let known = known_grids(st);
    let moves = &mut st.loops.moves;
    moves.push_back(Move { at: here, known });
    while moves.len() > LOOP_MOVES {
        moves.pop_front();
    }
    if moves.len() == 1 {
        st.loops.since = st.borg.time.now;
    }
}

/// #1153: is a step onto (y, x) closed by the third remedy?
pub(crate) fn step_banned(st: &BorgState, y: i32, x: i32) -> bool {
    st.loops.ban_until >= st.borg.time.now
        && st.loops.level == st.borg.time.level
        && st.loops.ban.iter().any(|g| g.y == y && g.x == x)
}

fn is_staircase(feat: u32) -> bool {
    feat == FEAT_LESS || feat == FEAT_MORE
}

/// #1153: break a loop the Borg is in. `true`: it acted this think (a step
/// toward a staircase); `false`: no loop, or the third remedy closed its
/// grids and the think goes on.
pub(crate) fn break_loop(st: &mut BorgState) -> bool {
    if value(Param::loop_breaker) == 0 || st.borg.tr(BI_CDEPTH) == 0 {
        return false;
    }
    let moves: Vec<Move> = st.loops.moves.iter().copied().collect();
    if !is_loop(&moves) {
        return false;
    }
    let here = borg_danger(st, st.borg.c.y, st.borg.c.x, 1, true, false);
    if here > st.borg.tr(BI_CURHP) / 4 {
        return false;
    }
    let mut grids: Vec<loc> = Vec::new();
    for m in &moves {
        if !grids.iter().any(|g| g.y == m.at.y && g.x == m.at.x) {
            grids.push(m.at);
        }
    }
    let choices = crate::core_trace::recent_choices().join(" / ");
    let since = st.loops.since;
    let head = format!(
        "# Loop detected: {} moves among {} grids since think {since}, the map unchanged; choices {choices}",
        moves.len(),
        grids.len()
    );

    /* 1: a staircase at the normal avoidance */
    if crate::borg_flow_stairs::borg_flow_stair_both(st, GOAL_FLEE, false) {
        crate::borg_flow::commit_loop_walk(st, BraveStairs::Any);
        borg_note(st, &format!("{head}; breaking it by a walk to the stairs (#1153)"));
        st.loops.moves.clear();
        return true;
    }

    /* 2: a committed brave walk to a staircase (#1130) */
    let saved = st.borg.avoidance;
    let to = st.borg.tr(BI_CURHP).saturating_mul(2);
    if i32::from(saved) < to {
        st.borg.avoidance = to as i16;
        st.danger.wipe = true;
        let stepped = crate::borg_flow_stairs::borg_flow_stair_both(st, GOAL_FLEE, false);
        if stepped {
            crate::borg_flow::commit_loop_walk(st, BraveStairs::Any);
        }
        st.borg.avoidance = saved;
        st.danger.wipe = true;
        if stepped {
            borg_note(st, &format!("{head}; breaking it by a brave walk to the stairs (#1153)"));
            st.loops.moves.clear();
            return true;
        }
    }

    /* 3: close the loop's grids (staircases excepted) for a while */
    st.loops.ban = grids
        .into_iter()
        .filter(|g| !is_staircase(u32::from(st.cave.grid(g.x, g.y).feat)) && (g.y, g.x) != (st.borg.c.y, st.borg.c.x))
        .collect();
    st.loops.ban_until = st.borg.time.now.saturating_add(LOOP_BAN_THINKS);
    let n = st.loops.ban.len();
    borg_note(st, &format!("{head}; no stairs reachable: closing its {n} other grids for {LOOP_BAN_THINKS} thinks (#1153)"));
    st.loops.moves.clear();
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mv(y: i32, x: i32, known: u32) -> Move {
        Move { at: loc { x, y }, known }
    }

    #[test]
    fn two_and_three_grid_loops_are_loops() {
        // #1130's Blackguard: (85,36) and (85,37).
        let two: Vec<Move> = (0..16).map(|i| mv(36 + i % 2, 85, 500)).collect();
        assert!(is_loop(&two));
        // #1131's Paladin: three floor grids, west and east.
        let three: Vec<Move> = (0..16).map(|i| mv(26, [171, 172, 173, 172][i % 4], 900)).collect();
        assert!(is_loop(&three));
    }

    #[test]
    fn a_walk_that_learns_the_map_is_no_loop() {
        // Back and forth along a corridor while new grids become known.
        let learning: Vec<Move> = (0..16).map(|i| mv(10, 5 + (i % 2), 500 + i as u32)).collect();
        assert!(!is_loop(&learning));
    }

    #[test]
    fn four_grids_or_too_few_moves_are_no_loop() {
        let four: Vec<Move> = (0..16).map(|i| mv(10, 5 + (i % 4) as i32, 500)).collect();
        assert!(!is_loop(&four), "a four-grid walk");
        let short: Vec<Move> = (0..15).map(|i| mv(10, 5 + (i % 2), 500)).collect();
        assert!(!is_loop(&short), "fewer than LOOP_MOVES moves");
    }

    #[test]
    fn only_the_last_window_counts() {
        // An exploring walk, then a two-grid loop.
        let mut moves: Vec<Move> = (0..10).map(|i| mv(10, i, 400 + i as u32)).collect();
        moves.extend((0..16).map(|i| mv(20 + i % 2, 3, 600)));
        assert!(is_loop(&moves));
    }

    #[test]
    fn the_switch_ships_off_in_the_code() {
        assert_eq!(value(Param::loop_breaker), 0, "compiled 0; harness/borg.txt ships 1");
    }
}
