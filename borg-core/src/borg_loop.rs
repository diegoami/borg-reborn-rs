//! borg-reborn-rs (#1153): a general movement-loop detector, the safety net
//! under the guards that each cover one pair of decisions (#1108, #1133,
//! #1152, #932).
//!
//! Every dungeon think records the Borg's grid when it moved, with how many
//! grids of the map it knows. A loop is `loop_moves` moves among at most
//! `loop_grids` grids while the map learned nothing, with no danger here above
//! a quarter of the HP. `loop_breaker` then breaks it, the first that
//! applies:
//! 0. the staircase underfoot, when the Borg may take it;
//! 1. a walk to a staircase it may take (up first; down only fleeing or
//!    prepared), at the normal avoidance;
//! 2. the same with the avoidance boosted to twice the HP;
//!
//! either committed (`commit_loop_walk`), continued here every think while
//! live, and the staircase taken on arrival;
//! 3. for `LOOP_BAN_THINKS` thinks, the loop's grids other than staircases
//!    closed to every spread and twitch (`step_banned`), so whatever undid the
//!    other decision cannot; boxed in by them, the twitchy think's bounded
//!    Phase Door instead.
//!
//! It runs after `borg_caution` (review of #1158).
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
    /// An exit the breaker started (review of #1158): the stair kind it walks
    /// to, taken on arrival.
    exiting: Option<BraveStairs>,
}

/// #1153: do these moves make a loop? `loop_moves` of them, among at most
/// `loop_grids` grids, the map unchanged from the first to the last.
pub(crate) fn is_loop(moves: &[Move]) -> bool {
    is_loop_with(moves, value(Param::loop_moves) as usize, value(Param::loop_grids) as usize)
}

/// #1190: `is_loop` with the window given, so the zoo arms can be reasoned
/// about (and tested) without touching the registry.
///
/// The supported region is `loop_grids < loop_moves` (shipped 16/3, the
/// #1190 arm 16/4). At `loop_grids >= loop_moves` the grid test is vacuous —
/// distinct grids can never exceed the window's moves — and the detector
/// degenerates to "`loop_moves` quiet moves with the map unchanged is a
/// loop": a 1-move profile flags the first quiet move, a 16/16 one flags a
/// walk over 16 different known squares. Nothing rejects such a profile at
/// load (the registry checks each parameter's own range); the test below
/// pins the degeneracy so it stays a documented choice, not a surprise.
fn is_loop_with(moves: &[Move], loop_moves: usize, loop_grids: usize) -> bool {
    if moves.len() < loop_moves {
        return false;
    }
    let window = &moves[moves.len() - loop_moves..];
    let mut grids: Vec<(i32, i32)> = window.iter().map(|m| (m.at.y, m.at.x)).collect();
    grids.sort_unstable();
    grids.dedup();
    grids.len() <= loop_grids && window.first().map(|m| m.known) == window.last().map(|m| m.known)
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
/// per level. Called at the start of every dungeon think. The arrival on a
/// level only seeds the position (no move); any think on which the map
/// learned something, or the danger here passes a quarter of the HP, clears
/// the window, moved or not (review of #1158).
pub(crate) fn observe(st: &mut BorgState) {
    if value(Param::loop_breaker) == 0 || st.borg.tr(BI_CDEPTH) == 0 {
        return;
    }
    let level = st.borg.time.level;
    let here = st.borg.c;
    if st.loops.level != level {
        st.loops = LoopState { level, last: Some(here), ..LoopState::default() };
        return;
    }
    let known = known_grids(st);
    let danger = borg_danger(st, here.y, here.x, 1, true, false);
    let stale_now = stale(st.loops.moves.back().map(|m| m.known), known, danger, st.borg.tr(BI_CURHP));
    if stale_now {
        st.loops.moves.clear();
    }
    if st.loops.last == Some(here) {
        return;
    }
    st.loops.last = Some(here);
    // A move under danger is not counted either (review of #1158): the
    // window holds quiet moves only.
    if danger > st.borg.tr(BI_CURHP) / 4 {
        return;
    }
    let moves = &mut st.loops.moves;
    moves.push_back(Move { at: here, known });
    let loop_moves = value(Param::loop_moves) as usize;
    while moves.len() > loop_moves {
        moves.pop_front();
    }
    if moves.len() == 1 {
        st.loops.since = st.borg.time.now;
    }
}

/// #1153 (review of #1158): does this think make the window stale? The map
/// learned something since its last move, or the danger here passes a
/// quarter of the HP.
fn stale(last_known: Option<u32>, known: u32, danger: i32, curhp: i32) -> bool {
    last_known.is_some_and(|k| k != known) || danger > curhp / 4
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

/// #1153 (reviews of #1158): may the Borg take up stairs, and down stairs,
/// by the gates `borg_caution` applies before it takes them: up not under
/// forced descent nor while a Recall hold keeps it (`recall_holds_up_stair`);
/// down only fleeing or prepared for the next depth, and not while Recall
/// restricts it, nor under the rising holds (`rising_holds_descent`,
/// `rising_holds_flee_descent`).
fn stairs_allowed(st: &mut BorgState) -> (bool, bool) {
    use crate::borg_caution::{recall_holds_up_stair, recall_restricts, rising_holds_descent, rising_holds_flee_descent};
    let here = borg_danger(st, st.borg.c.y, st.borg.c.x, 1, true, false);
    let up = !crate::engine::player_opt(crate::headers::list_options_h::OPT_birth_force_descend)
        && !recall_holds_up_stair(st, here);
    let next = st.borg.tr(BI_CDEPTH) + 1;
    let down = (st.borg.goal.fleeing || crate::borg_prepared::prepared_c_for_decision(st, next).is_none())
        && !recall_restricts(st)
        && !rising_holds_descent(st, here)
        && !rising_holds_flee_descent(st);
    (up, down)
}

/// #1153 (review of #1158): take the staircase underfoot when it is one the
/// Borg may take (`stairs_allowed`).
fn take_stairs_here(st: &mut BorgState) -> bool {
    let (up, down) = stairs_allowed(st);
    let feat = u32::from(st.cave.grid(st.borg.c.x, st.borg.c.y).feat);
    let key = if feat == FEAT_LESS && up {
        b'<'
    } else if feat == FEAT_MORE && down {
        b'>'
    } else {
        return false;
    };
    crate::borg_io::borg_keypress(st, crate::headers::ui_event_h::keycode_t::from(key));
    true
}

/// #1153: is any of the eight grids around the Borg one a flow could enter,
/// other than the closed ones? Used to judge it boxed in by the ban. The test
/// is the flows' own (`flow_open`, review of #1158): a closed door is an exit,
/// lava only with fire immunity.
fn open_neighbour(st: &BorgState, closed: &[loc]) -> bool {
    (0..8).any(|i| {
        let x = st.borg.c.x + i32::from(crate::engine::ddx_ddd(i));
        let y = st.borg.c.y + i32::from(crate::engine::ddy_ddd(i));
        (1..AUTO_MAX_X - 1).contains(&x)
            && (1..AUTO_MAX_Y - 1).contains(&y)
            && !closed.iter().any(|g| g.x == x && g.y == y)
            && crate::borg_flow::flow_open(st, x, y)
    })
}

/// #1153: break a loop the Borg is in, or continue the exit it started.
/// Called after `borg_caution` (review of #1158). `true`: it acted this
/// think.
pub(crate) fn break_loop(st: &mut BorgState) -> bool {
    if value(Param::loop_breaker) == 0 || st.borg.tr(BI_CDEPTH) == 0 {
        return false;
    }

    /* An exit underway: take the stairs on arrival, else continue the walk
     * while it is live and owns its flow; otherwise it ends here (no
     * ordinary step falls through, review of #1158) */
    if st.loops.exiting.is_some() {
        if take_stairs_here(st) {
            borg_note(st, "# Loop broken: taking the stairs (#1153)");
            st.loops.exiting = None;
            return true;
        }
        let here = borg_danger(st, st.borg.c.y, st.borg.c.x, 1, true, false);
        let live = st.borg.brave_walk.is_some_and(|w| w.from_loop)
            && i32::from(st.borg.goal.type_0) == GOAL_FLEE
            && crate::borg_flow::brave_walk_avoidance(st, here).is_some();
        if live && crate::borg_flow::borg_flow_old(st, GOAL_FLEE) {
            return true;
        }
        st.loops.exiting = None;
        if st.borg.brave_walk.is_some_and(|w| w.from_loop) {
            st.borg.brave_walk = None;
        }
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
    st.loops.moves.clear();

    /* 0: standing on a staircase the Borg may take */
    if take_stairs_here(st) {
        borg_note(st, &format!("{head}; breaking it by the stairs underfoot (#1153)"));
        return true;
    }

    /* 1, 2: a walk to a staircase the Borg may take (an up one first: a new
     * level is always safe), at the normal avoidance, then boosted;
     * committed, continued above, taken on arrival */
    let (up, down) = stairs_allowed(st);
    let saved = st.borg.avoidance;
    let boosted = st.borg.tr(BI_CURHP).saturating_mul(2).min(i32::from(i16::MAX)) as i16;
    let mut tries = vec![(saved, "a walk to the stairs")];
    if boosted > saved {
        tries.push((boosted, "a brave walk to the stairs"));
    }
    for (avoid, how) in tries {
        st.borg.avoidance = avoid;
        st.danger.wipe = true;
        let brave = avoid > saved;
        let kind = if up && crate::borg_flow_stairs::borg_flow_stair_less(st, GOAL_FLEE, false) {
            Some(BraveStairs::Up)
        } else if down && crate::borg_flow_stairs::borg_flow_stair_more(st, GOAL_FLEE, false, brave) {
            Some(BraveStairs::Down)
        } else {
            None
        };
        if let Some(kind) = kind {
            crate::borg_flow::commit_loop_walk(st, kind);
        }
        st.borg.avoidance = saved;
        st.danger.wipe = true;
        if let Some(kind) = kind {
            st.loops.exiting = Some(kind);
            borg_note(st, &format!("{head}; breaking it by {how} (#1153)"));
            return true;
        }
    }

    /* 3: close the loop's grids (staircases and the Borg's own excepted) for
     * a while; boxed in by them, the twitchy think's bounded Phase Door */
    let closed: Vec<loc> = grids
        .into_iter()
        .filter(|g| !is_staircase(u32::from(st.cave.grid(g.x, g.y).feat)) && (g.y, g.x) != (st.borg.c.y, st.borg.c.x))
        .collect();
    if open_neighbour(st, &closed) {
        let n = closed.len();
        st.loops.ban = closed;
        st.loops.ban_until = st.borg.time.now.saturating_add(LOOP_BAN_THINKS);
        // A committed route was spread before the ban: drop it, so the next
        // flow is spread around the closed grids (review of #1158).
        st.borg.goal.type_0 = 0;
        borg_note(st, &format!("{head}; no stairs to take: closing its {n} other grids for {LOOP_BAN_THINKS} thinks (#1153)"));
        return false;
    }
    if st.borg.times_twitch < 3 {
        st.borg.times_twitch = st.borg.times_twitch.wrapping_add(1);
        borg_note(st, &format!("{head}; boxed in: the twitchy Phase Door (#1153)"));
        if crate::borg_think_dungeon::twitch_phase(st) {
            return true;
        }
    }
    borg_note(st, &format!("{head}; boxed in, no Phase Door left: left to the stuck think (#1153)"));
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
    fn the_window_is_named_and_the_four_grid_cycle_is_seen_at_four() {
        // #1190: the window is `loop_moves` x `loop_grids`, named so the zoo
        // can measure the arm at 4 grids. The recorded cycle of #1189 walked
        // 444666 among four grids: invisible at the shipped 3, a loop at 4.
        let four: Vec<Move> = (0..16).map(|i| mv(10, 5 + (i % 4) as i32, 500)).collect();
        assert!(!is_loop_with(&four, 16, 3), "the shipped window does not see it");
        assert!(is_loop_with(&four, 16, 4), "#1190's arm does");
        assert!(!is_loop_with(&four, 17, 4), "too few moves is no loop");
        let learning: Vec<Move> = (0..16).map(|i| mv(10, 5 + (i % 4) as i32, 500 + i as u32)).collect();
        assert!(!is_loop_with(&learning, 16, 4), "a walk that learns the map is no loop at 4 either");
    }

    #[test]
    fn degenerate_windows_are_documented_not_rejected() {
        // #1190 (review of #1205, finding P3): the registry checks each
        // parameter's own range, so `loop_grids >= loop_moves` is loadable.
        // There the grid test is vacuous and the detector degenerates to
        // "loop_moves quiet moves with the map unchanged is a loop" — these
        // assertions pin that as a documented choice; the supported region
        // (loop_grids < loop_moves) is in is_loop_with's doc and CALIBRATION.
        let one = [mv(10, 5, 500)];
        assert!(is_loop_with(&one, 1, 3), "1 move x 3 grids: the first quiet move is a loop");
        let spread: Vec<Move> = (0..16).map(|i| mv(10, i, 500)).collect();
        assert!(is_loop_with(&spread, 16, 16), "16 moves x 16 grids: 16 known squares are a loop");
    }

    #[test]
    fn only_the_last_window_counts() {
        // An exploring walk, then a two-grid loop.
        let mut moves: Vec<Move> = (0..10).map(|i| mv(10, i, 400 + i as u32)).collect();
        moves.extend((0..16).map(|i| mv(20 + i % 2, 3, 600)));
        assert!(is_loop(&moves));
    }

    #[test]
    fn a_map_change_or_danger_makes_the_window_stale() {
        // Review of #1158, finding 4.
        assert!(!stale(Some(500), 500, 0, 40), "nothing learned, no danger");
        assert!(stale(Some(500), 501, 0, 40), "the map learned a grid in place");
        assert!(stale(Some(500), 500, 11, 40), "danger above a quarter of the HP");
        assert!(!stale(None, 500, 0, 40), "an empty window");
    }

    #[test]
    fn the_switch_ships_off_in_the_code() {
        assert_eq!(value(Param::loop_breaker), 0, "compiled 0; harness/pborg.yaml ships 1");
        assert_eq!(value(Param::loop_moves), 16, "#1190: the window's moves, #1153's guess");
        assert_eq!(value(Param::loop_grids), 3, "#1190: the window's grids, #1153's guess");
    }
}
