//! borg-reborn-rs (#1062): three holes in C's stair leash, each behind a
//! switch compiled and shipped 0 (C's play).
//!
//! From the level-1 death review of track-p `206cf4e7` (borg-zoo run
//! 45070528012, attempt 2; borg-cemetery study
//! `studies/lvl1-deaths-trackp-206cf4e7`, diegoami/borg-cemetery#138). C's
//! leash keeps a weak Borg within CL x 3 + 9 flow steps of an up staircase
//! when it explores (`borg_get_leash`, `borg_flow_far_from_stairs_dist`) and
//! walks it back once it is more than 3 x CL + 14 grids away
//! (`borg_think_dungeon`'s "Return to Stair"). It has three holes:
//!
//! 1. only up staircases count: a Borg that arrives on a down staircase
//!    (after going up a connected staircase) has no leash until it sees an
//!    up one (Kobold Mage 592754909, Dwarf Mage 645909552);
//! 2. no known staircase, or none reachable, means no leash at all
//!    (`borg_flow_cost_stair` returns 0, "go off leash");
//! 3. the exploration leash measures the farthest up staircase, not the
//!    nearest (`borg_flow_dark`, C bug 31).
//!
//! And a teleport (trap or scroll) can throw a CL1 Borg far from every
//! staircase it knows (Dunadan Druid 1024822556: 84 squares, 79 flow steps
//! from the up staircase; it walked back and met Fang).
//!
//! The switches:
//!
//! - `flow_leash_any_stair`: both leashes measure the nearest known
//!   staircase of either kind (holes 1 and 3). The exploration leash takes
//!   the flow steps from the nearest of them (one flow from every known
//!   staircase), the return leash the grids to the nearest of them, and the
//!   walk back goes to the nearest staircase of either kind.
//! - `flow_leash_unknown_stair_steps`: with no known staircase the leash
//!   counts, or none reachable, the exploration leash is this many flow
//!   steps from the grid where the Borg arrived on the level (hole 2).
//! - `flow_leash_teleported_recall`: up to `flow_leash_recall_max_level`, a
//!   jump of more than one grid (a teleport of any kind) that leaves every
//!   known staircase more than `flow_leash_recall_steps` flow steps away
//!   reads Word of Recall at the first think at danger 0.
//!
//! Each is a hypothesis until borg-zoo measures it (RULES 14).

use crate::borg_danger::borg_danger;
use crate::borg_headers::borg_trait_h::*;
use crate::borg_io::borg_note;
use crate::engine;
use crate::headers::z_type_h::loc;
use crate::borg_flow::leash_holes;
use crate::state::BorgState;

/// What the leash remembers of the current level (borg-reborn-rs #1062).
#[derive(Clone, Debug, Default)]
pub struct LeashState {
    /// `time.level` of the level the grids below belong to (`None`: none yet).
    pub level_time: Option<i32>,
    /// Where the Borg stood at its first update on this level, (y, x).
    pub arrival: Option<(i32, i32)>,
    /// Where it stood at the last update, (y, x).
    pub last: Option<(i32, i32)>,
    /// A jump of more than one grid seen by the update and not yet judged
    /// by the think (its length in grids).
    pub jumped: Option<i32>,
    /// A jump left the staircases far: read Recall at danger 0.
    pub recall_due: bool,
    /// The walk back of the return leash goes to the nearest staircase of
    /// either kind (set when `flow_leash_any_stair` sets `goal.less`).
    pub return_any: bool,
}

/// `flow_leash_any_stair`, in the dungeon only: town has no up staircase,
/// so C's leashes never act there, and the town's down staircase must not
/// start one (the shops are far from it).
pub fn any_stair(st: &BorgState) -> bool {
    st.borg.tr(BI_CDEPTH) > 0 && leash_holes().any_stair
}

/// Does the update need to follow the Borg's grid (arrival, jumps)?
fn tracking() -> bool {
    let h = leash_holes();
    h.unknown_stair_steps > 0 || h.teleported_recall
}

/// The grids the exploration leash measures from, (y, x): with `any`, every
/// known staircase of either kind; otherwise C's, the up staircase
/// `b_stair` (none for -1).
fn leash_sources(any: bool, b_stair: i32, less: &[(i32, i32)], more: &[(i32, i32)]) -> Vec<(i32, i32)> {
    if any {
        less.iter().chain(more).copied().collect()
    } else {
        usize::try_from(b_stair).ok().and_then(|i| less.get(i).copied()).into_iter().collect()
    }
}

/// The verdict of the exploration leash: `stairs` is the flow steps from
/// the staircases to the grid (`None`: none known, or none reachable);
/// `arrival` gives the steps from the arrival grid (`None`: unknown or
/// unreachable), asked only when needed. C: a grid with no staircase to
/// measure from is never too far.
fn beyond_verdict(stairs: Option<i32>, limit: i32, unknown_steps: i32, arrival: impl FnOnce() -> Option<i32>) -> bool {
    match stairs {
        Some(cost) => cost > limit,
        None if unknown_steps > 0 => arrival().is_some_and(|cost| cost > unknown_steps),
        None => false,
    }
}

fn track_list(t: &crate::borg_flow_stairs::StairTrack) -> Vec<(i32, i32)> {
    (0..usize::try_from(t.num).unwrap_or(0)).map(|i| t.at(i)).collect()
}

/// The flow steps from the nearest of `sources` to (y, x), as
/// `borg_flow_cost_stair` spreads it (the flow codes are cleared first, as
/// there). `None`: no source, or (y, x) unreachable.
fn cost_from(st: &mut BorgState, y: i32, x: i32, sources: &[(i32, i32)]) -> Option<i32> {
    crate::borg_flow::borg_flow_clear(st);
    if sources.is_empty() {
        return None;
    }
    for &(sy, sx) in sources {
        crate::borg_flow::borg_flow_enqueue_grid(st, sy, sx);
    }
    crate::borg_flow::borg_flow_spread_stairs(st, 250, false, false, false, -1, false);
    let cost = i32::from(st.flow.cost_at(y, x));
    (cost != 255).then_some(cost)
}

/// Is the grid (y, x) beyond the leash `limit`, measured from the up
/// staircase `b_stair`? With both switches off this is C's
/// `borg_flow_cost_stair(y, x, b_stair) > limit` (the flow codes are
/// cleared and spread as there). Town is always C's.
pub fn beyond(st: &mut BorgState, y: i32, x: i32, b_stair: i32, limit: i32) -> bool {
    let h = leash_holes();
    let (any, unknown_steps) = (h.any_stair, h.unknown_stair_steps);
    if (!any && unknown_steps == 0) || st.borg.tr(BI_CDEPTH) == 0 {
        return crate::borg_flow_stairs::borg_flow_cost_stair(st, y, x, b_stair) > limit;
    }
    let sources = leash_sources(any, b_stair, &track_list(&st.stairs.less), &track_list(&st.stairs.more));
    let stairs = cost_from(st, y, x, &sources);
    let arrival = st.leash.arrival;
    beyond_verdict(stairs, limit, unknown_steps, || arrival.and_then(|a| cost_from(st, y, x, &[a])))
}

/// `engine::distance` from `c` to the nearest of the staircases, (y, x);
/// -1 for none. C's scan (`nearest_up_stair` of `borg_think_dungeon`), over
/// whichever staircases it is given.
fn nearest_distance(c: (i32, i32), stairs: &[(i32, i32)]) -> i32 {
    let at = |(y, x): (i32, i32)| loc { x, y };
    stairs.iter().map(|&s| engine::distance(at(c), at(s))).min().unwrap_or(-1)
}

/// The return leash with `flow_leash_any_stair`: the distance to the
/// nearest known staircase of either kind (-1 for none), as
/// `borg_think_dungeon` measures it.
pub fn return_distance(st: &BorgState) -> i32 {
    let mut all = track_list(&st.stairs.less);
    all.extend(track_list(&st.stairs.more));
    nearest_distance((st.borg.c.y, st.borg.c.x), &all)
}

/// Is a jump from `last` to `now` (y, x) more than a step (a teleport)?
/// Returns its length in grids.
fn jump(last: (i32, i32), now: (i32, i32)) -> Option<i32> {
    let d = nearest_distance(last, &[now]);
    (d > 1).then_some(d)
}

/// Once per update, after the map: a new level starts a new state; the
/// arrival grid and jumps are followed only with
/// `flow_leash_unknown_stair_steps` or `flow_leash_teleported_recall`.
/// Nothing here changes play by itself.
pub fn update(st: &mut BorgState) {
    let now = (st.borg.c.y, st.borg.c.x);
    let level = st.borg.time.level;
    let l = &mut st.leash;
    if l.level_time != Some(level) {
        *l = LeashState { level_time: Some(level), arrival: Some(now), last: Some(now), ..LeashState::default() };
        return;
    }
    if !tracking() {
        return;
    }
    if let Some(d) = l.last.and_then(|last| jump(last, now)) {
        l.jumped = Some(d);
    }
    l.last = Some(now);
}

/// The verdict of `flow_leash_teleported_recall` on a jump: the flow steps
/// from the nearest known staircase (`None`: none known or reachable)
/// against `steps`.
fn jump_strands(stairs: Option<i32>, steps: i32) -> bool {
    stairs.is_none_or(|cost| cost > steps)
}

/// `flow_leash_teleported_recall`, once per dungeon think (after the fights
/// that own the action): judge a jump the update saw, and read Word of
/// Recall at the first think at danger 0 after one stranded the Borg.
pub fn teleported_recall(st: &mut BorgState) -> bool {
    let h = leash_holes();
    if !h.teleported_recall {
        return false;
    }
    let depth = st.borg.tr(BI_CDEPTH);
    if depth == 0 || st.borg.tr(BI_CLEVEL) > h.recall_max_level {
        st.leash.jumped = None;
        st.leash.recall_due = false;
        return false;
    }
    if let Some(d) = st.leash.jumped.take() {
        let steps = h.recall_steps;
        let mut all = track_list(&st.stairs.less);
        all.extend(track_list(&st.stairs.more));
        let (y, x) = (st.borg.c.y, st.borg.c.x);
        // An observation: the flow the Borg follows is put back as it was.
        let cost = with_flow_saved(st, |st| cost_from(st, y, x, &all));
        if jump_strands(cost, steps) {
            st.leash.recall_due = true;
            let far = cost.map_or("no staircase known or reachable".to_string(), |c| format!("{c} steps from the nearest staircase"));
            borg_note(st, &format!("# Leash: a {d}-grid jump left the Borg {far} (more than {steps}): Recall at danger 0"));
        }
    }
    if !st.leash.recall_due {
        return false;
    }
    if st.borg.goal.recalling != 0 || st.borg.tr(BI_RECALL) <= 0 {
        st.leash.recall_due = false;
        return false;
    }
    let (y, x) = (st.borg.c.y, st.borg.c.x);
    if borg_danger(st, y, x, 1, true, false) > 0 {
        return false;
    }
    if crate::borg_escape::borg_recall(st) {
        borg_note(st, "# Leash: reading Recall after the jump");
        st.leash.recall_due = false;
        return true;
    }
    false
}

/// Run `f` and put back the flow state it may spread over (the queue, the
/// costs, the danger caches and the wipe flag), so the flow the Borg follows
/// (`borg_flow_old`) is untouched.
fn with_flow_saved<R>(st: &mut BorgState, f: impl FnOnce(&mut BorgState) -> R) -> R {
    let fl = &st.flow;
    let saved = (fl.head, fl.tail, fl.flow_x.clone(), fl.flow_y.clone(), fl.data_cost.clone(),
                 fl.data_know.clone(), fl.data_icky.clone(), fl.hunting, fl.to_stairs);
    let wipe = st.danger.wipe;
    let r = f(st);
    st.danger.wipe = wipe;
    let fl = &mut st.flow;
    (fl.head, fl.tail, fl.flow_x, fl.flow_y, fl.data_cost, fl.data_know, fl.data_icky, fl.hunting, fl.to_stairs) = saved;
    r
}

#[cfg(test)]
mod tests_1062 {
    use super::*;

    const LESS: [(i32, i32); 2] = [(10, 10), (40, 60)];
    const MORE: [(i32, i32); 1] = [(12, 14)];

    /// C measures from the up staircase it was given; the switch from every
    /// staircase of either kind.
    #[test]
    fn the_sources_of_the_exploration_leash() {
        assert_eq!(leash_sources(false, 1, &LESS, &MORE), vec![(40, 60)]);
        assert_eq!(leash_sources(false, -1, &LESS, &MORE), Vec::<(i32, i32)>::new());
        // 592754909 / 645909552: only a down staircase known.
        assert_eq!(leash_sources(false, -1, &[], &MORE), Vec::<(i32, i32)>::new());
        assert_eq!(leash_sources(true, -1, &[], &MORE), vec![(12, 14)]);
        assert_eq!(leash_sources(true, 0, &LESS, &MORE), vec![(10, 10), (40, 60), (12, 14)]);
    }

    /// Hole 2: no staircase to measure from is never far in C; with
    /// `flow_leash_unknown_stair_steps` the arrival grid is.
    #[test]
    fn no_staircase_known_or_reachable() {
        assert!(beyond_verdict(Some(13), 12, 0, || panic!("not asked")));
        assert!(!beyond_verdict(Some(12), 12, 20, || panic!("not asked")));
        assert!(!beyond_verdict(None, 12, 0, || Some(100)), "C: off leash");
        assert!(beyond_verdict(None, 12, 20, || Some(21)));
        assert!(!beyond_verdict(None, 12, 20, || Some(20)));
        assert!(!beyond_verdict(None, 12, 20, || None), "arrival unreachable too: off leash");
    }

    /// The return leash: the nearest of both kinds, in `engine::distance`.
    #[test]
    fn the_nearest_staircase_of_either_kind() {
        assert_eq!(nearest_distance((0, 0), &[(3, 4)]), 5);
        assert_eq!(nearest_distance((0, 0), &[(8, 2)]), 9);
        let mut all = LESS.to_vec();
        all.extend(MORE);
        let c = (12, 20);
        assert_eq!(nearest_distance(c, &LESS), 11, "C: the up staircase at (10, 10)");
        assert_eq!(nearest_distance(c, &all), 6, "the down staircase at (12, 14)");
        assert_eq!(nearest_distance(c, &[]), -1);
    }

    /// A step is not a jump; a Phase Door or a teleport is.
    #[test]
    fn a_jump_is_more_than_a_step() {
        assert_eq!(jump((5, 5), (6, 6)), None);
        assert_eq!(jump((5, 5), (5, 5)), None);
        assert_eq!(jump((5, 5), (5, 7)), Some(2));
        // 1024822556: the teleport trap threw the Druid 84 squares.
        assert_eq!(jump((30, 100), (30, 16)), Some(84));
    }

    /// 1024822556: 79 flow steps from the up staircase after the trap.
    #[test]
    fn a_jump_strands_the_borg() {
        assert!(jump_strands(Some(79), 30));
        assert!(!jump_strands(Some(30), 30));
        assert!(jump_strands(None, 30), "no staircase known or reachable");
    }

    /// The default state follows nothing.
    #[test]
    fn a_fresh_state_is_empty() {
        let l = LeashState::default();
        assert!(l.level_time.is_none() && l.arrival.is_none() && l.jumped.is_none());
        assert!(!l.recall_due && !l.return_any);
    }
}
