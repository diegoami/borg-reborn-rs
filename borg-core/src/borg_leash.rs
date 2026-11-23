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
//!   staircase of either kind (holes 1 and 3), in walking steps over the
//!   known map (`steps_map`). The return leash starts on those steps, ends
//!   on them only for a return it started, and walks back to the nearest
//!   staircase of either kind.
//! - `flow_leash_unknown_stair_steps`: with no known staircase the leash
//!   counts, or none reachable, the exploration leash is this many walking
//!   steps from the grid where the Borg arrived on the level (hole 2).
//! - `flow_leash_teleported_recall`: up to `flow_leash_recall_max_level`, a
//!   jump of at least `flow_leash_recall_min_jump` grids (a teleport, not
//!   Phase Door) that leaves every known staircase more than
//!   `flow_leash_recall_steps` steps away reads Word of Recall at the first
//!   think at danger 0.
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
    /// The step maps of this think (`time.now`), by their sources: see
    /// `steps_map` (review of #1087: one spread a think, not one a grid).
    pub maps: Vec<(i32, Vec<(i32, i32)>, crate::borg_flow::BorgData)>,
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

/// The verdict of the exploration leash: `stairs` is the steps from the
/// staircases to the grid (`None`: none known, or none reachable);
/// `arrival` gives the steps from the arrival grid (`None`: unknown or
/// unreachable), asked only when needed. C: a grid with no staircase to
/// measure from is never too far. A grid the arrival grid does not reach
/// either is not judged far (the walk back could not use it).
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

/// Every known staircase of either kind, (y, x).
fn all_stairs(st: &BorgState) -> Vec<(i32, i32)> {
    let mut all = track_list(&st.stairs.less);
    all.extend(track_list(&st.stairs.more));
    all
}

/// The walking steps from the nearest of `sources` to every grid, over the
/// known map (the flow's own terrain rules, `borg_flow_spread_stairs` to 250
/// steps), **without the danger check** (review of #1087, findings 1 and 3):
/// a monster near a staircase or the arrival grid must not hide it, since
/// the leash is about the way back, not this turn's danger. The flow state
/// the Borg follows is put back as it was. Spread once a think for a set of
/// sources, then read from the cache; returns its index in
/// `LeashState::maps`.
fn steps_map(st: &mut BorgState, sources: &[(i32, i32)]) -> Option<usize> {
    if sources.is_empty() {
        return None;
    }
    let now = st.borg.time.now;
    st.leash.maps.retain(|(t, _, _)| *t == now);
    if let Some(i) = st.leash.maps.iter().position(|(_, s, _)| s == sources) {
        return Some(i);
    }
    let map = with_flow_saved(st, |st| {
        let was = std::mem::replace(&mut st.flow.flight_no_danger, true);
        // Grids already found icky by an earlier flow stay out of every
        // spread: clear them for this one (the saved copy is put back).
        crate::borg_flow::borg_flow_clear(st);
        for row in st.flow.data_icky.iter_mut() {
            row.fill(0);
        }
        for &(sy, sx) in sources {
            crate::borg_flow::borg_flow_enqueue_grid(st, sy, sx);
        }
        crate::borg_flow::borg_flow_spread_stairs(st, 250, false, false, false, -1, false);
        st.flow.flight_no_danger = was;
        st.flow.data_cost.clone()
    });
    st.leash.maps.push((now, sources.to_vec(), map));
    Some(st.leash.maps.len() - 1)
}

/// The steps from the nearest of `sources` to (y, x) on `steps_map`;
/// `None`: no source, or (y, x) unreachable.
fn steps_from(st: &mut BorgState, y: i32, x: i32, sources: &[(i32, i32)]) -> Option<i32> {
    let i = steps_map(st, sources)?;
    let cost = i32::from(st.leash.maps[i].2[y as usize][x as usize]);
    (cost != 255).then_some(cost)
}

/// Is the grid (y, x) beyond the leash `limit`, measured from the up
/// staircase `b_stair`? With both switches off this is C's
/// `borg_flow_cost_stair(y, x, b_stair) > limit`. Town is always C's. With
/// a switch on the flow codes are cleared as C's call clears them, and the
/// steps come from `steps_map` (every known staircase with
/// `flow_leash_any_stair`, else C's up staircase `b_stair`; the arrival
/// grid for `flow_leash_unknown_stair_steps`).
pub fn beyond(st: &mut BorgState, y: i32, x: i32, b_stair: i32, limit: i32) -> bool {
    let h = leash_holes();
    let (any, unknown_steps) = (h.any_stair, h.unknown_stair_steps);
    if (!any && unknown_steps == 0) || st.borg.tr(BI_CDEPTH) == 0 {
        return crate::borg_flow_stairs::borg_flow_cost_stair(st, y, x, b_stair) > limit;
    }
    crate::borg_flow::borg_flow_clear(st);
    let sources = leash_sources(any, b_stair, &track_list(&st.stairs.less), &track_list(&st.stairs.more));
    let stairs = steps_from(st, y, x, &sources);
    let arrival = st.leash.arrival;
    beyond_verdict(stairs, limit, unknown_steps, || arrival.and_then(|a| steps_from(st, y, x, &[a])))
}

/// `engine::distance` from `c` to the nearest of the staircases, (y, x);
/// -1 for none. C's scan (`nearest_up_stair` of `borg_think_dungeon`), over
/// whichever staircases it is given.
fn nearest_distance(c: (i32, i32), stairs: &[(i32, i32)]) -> i32 {
    let at = |(y, x): (i32, i32)| loc { x, y };
    stairs.iter().map(|&s| engine::distance(at(c), at(s))).min().unwrap_or(-1)
}

/// The return leash's two distances, -1 for none: the one that starts a
/// return, and the one that ends it ("Close enough"). C (`up` only): the
/// distance to the nearest up staircase for both. With
/// `flow_leash_any_stair` a return starts on `steps`, the walking steps to
/// the nearest staircase of either kind (review of #1087, finding 1: a
/// staircase across a wall is not near); only a return this leash started
/// (`return_any`) ends on them, and any other `goal.less` (the depth
/// correction's climb) ends on C's distance to an up staircase (finding 2).
fn return_distances(any: bool, return_any: bool, up: i32, steps: Option<i32>) -> (i32, i32) {
    if !any {
        return (up, up);
    }
    let steps = steps.unwrap_or(-1);
    (steps, if return_any { steps } else { up })
}

/// The return leash's distances (`return_distances`) for the Borg now; `up`
/// is C's distance to the nearest up staircase (-1 for none).
pub fn return_leash(st: &mut BorgState, up: i32) -> (i32, i32) {
    let any = any_stair(st);
    let steps = if any {
        let (y, x) = (st.borg.c.y, st.borg.c.x);
        let all = all_stairs(st);
        steps_from(st, y, x, &all)
    } else {
        None
    };
    return_distances(any, st.leash.return_any, up, steps)
}

/// Is a jump from `last` to `now` (y, x) more than a step? Returns its
/// length in grids (`engine::distance`, as the engine's teleport measures).
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
        l.jumped = Some(l.jumped.map_or(d, |j| j.max(d)));
    }
    l.last = Some(now);
}

/// The verdict of `flow_leash_teleported_recall` on a jump of `jump` grids:
/// a teleport (at least `min_jump`, review of #1087 finding 4: Phase Door
/// lands 9 to 11 grids away, `TELEPORT:10` in object.txt randomised by a
/// quarter in `effect_handler_TELEPORT`, effect-handler-general.c; the
/// teleport trap and the Teleportation scroll are `TELEPORT:M80` and `:M60`,
/// percentages of the distance to the level's edge) that leaves the nearest
/// known staircase (`stairs` steps; `None`: none known or reachable) more
/// than `steps` away.
fn jump_strands(jump: i32, min_jump: i32, stairs: Option<i32>, steps: i32) -> bool {
    jump >= min_jump && stairs.is_none_or(|cost| cost > steps)
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
        if d >= h.recall_min_jump {
            let all = all_stairs(st);
            let (y, x) = (st.borg.c.y, st.borg.c.x);
            let cost = steps_from(st, y, x, &all);
            if jump_strands(d, h.recall_min_jump, cost, steps) {
                st.leash.recall_due = true;
                let far = cost.map_or("no staircase known or reachable".to_string(), |c| format!("{c} steps from the nearest staircase"));
                borg_note(st, &format!("# Leash: a {d}-grid jump left the Borg {far} (more than {steps}): Recall at danger 0"));
            }
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
/// costs, the danger caches, the wipe flag and the flow modes), so the flow
/// the Borg follows (`borg_flow_old`) is untouched.
fn with_flow_saved<R>(st: &mut BorgState, f: impl FnOnce(&mut BorgState) -> R) -> R {
    let fl = &st.flow;
    let saved = (fl.head, fl.tail, fl.flow_x.clone(), fl.flow_y.clone(), fl.data_cost.clone(),
                 fl.data_know.clone(), fl.data_icky.clone(), fl.hunting, fl.to_stairs, fl.flight_no_danger);
    let wipe = st.danger.wipe;
    let r = f(st);
    st.danger.wipe = wipe;
    let fl = &mut st.flow;
    (fl.head, fl.tail, fl.flow_x, fl.flow_y, fl.data_cost, fl.data_know, fl.data_icky, fl.hunting, fl.to_stairs, fl.flight_no_danger) = saved;
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

    /// `engine::distance` to the nearest of the staircases given.
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

    /// Review of #1087, findings 1 and 2: the return leash starts on the
    /// walking steps; only its own return ends on them.
    #[test]
    fn the_return_leash_distances() {
        // C: the up staircase's distance for both.
        assert_eq!(return_distances(false, false, 11, Some(2)), (11, 11));
        // A staircase two grids away across a wall, 30 steps around: the
        // return starts (30 > 17 at CL1) and does not end at 2 grids.
        assert_eq!(return_distances(true, true, 2, Some(30)), (30, 30));
        // The depth correction's climb to an up staircase 20 grids away is
        // not ended by a down staircase 2 steps away.
        assert_eq!(return_distances(true, false, 20, Some(2)), (2, 20));
        // Nothing reachable: no return starts.
        assert_eq!(return_distances(true, false, -1, None), (-1, -1));
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
    /// Review of #1087, finding 4: a Phase Door (9 to 11 grids) is not a
    /// teleport; 592754909's 9-grid Phase Door 47 steps from the stairs.
    #[test]
    fn a_jump_strands_the_borg() {
        assert!(jump_strands(84, 12, Some(79), 30));
        assert!(!jump_strands(84, 12, Some(30), 30));
        assert!(jump_strands(84, 12, None, 30), "no staircase known or reachable");
        assert!(!jump_strands(9, 12, Some(47), 30), "a Phase Door");
        assert!(!jump_strands(11, 12, None, 30), "a Phase Door at its longest");
        assert!(jump_strands(12, 12, Some(31), 30));
    }

    /// The default state follows nothing.
    #[test]
    fn a_fresh_state_is_empty() {
        let l = LeashState::default();
        assert!(l.level_time.is_none() && l.arrival.is_none() && l.jumped.is_none());
        assert!(!l.recall_due && !l.return_any);
    }
}
