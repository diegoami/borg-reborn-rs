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
//!   staircase of either kind (holes 1 and 3), in walking steps over known
//!   terrain (`terrain_steps`). The return leash starts on those steps, ends
//!   on them only for a return it started, and walks back to the nearest
//!   staircase of either kind.
//! - `flow_leash_unknown_stair_steps`: with no known staircase the leash
//!   counts, or none reachable, the exploration leash is this many walking
//!   steps from the grid where the Borg arrived on the level (hole 2).
//! - `flow_leash_teleported_recall`: up to `flow_leash_recall_max_level`, a
//!   teleport (`JumpCause`: a trap or the Borg's own teleport, not its Phase
//!   Door; of unknown cause, from `flow_leash_recall_min_jump` grids) that
//!   leaves every known staircase more than
//!   `flow_leash_recall_steps` steps away reads Word of Recall at the first
//!   think at danger 0.
//!
//! Each is a hypothesis until borg-zoo measures it (RULES 14).

use crate::borg_cave::AUTO_MAX_X;
use crate::borg_cave::AUTO_MAX_Y;
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
    /// The grids the arrival leash can measure from, (y, x): where the Borg
    /// stood at its first update on this level, then the landing of every
    /// jump with no known route back to these (review of #1087: a
    /// disconnected landing area is bounded too). At most `MAX_ANCHORS`;
    /// `current_anchor` picks the one the Borg's area is connected to.
    pub anchors: Vec<(i32, i32)>,
    /// Where it stood at the last update, (y, x).
    pub last: Option<(i32, i32)>,
    /// A jump of more than one grid seen by the update and not yet judged
    /// by the think: its length in grids and its cause.
    pub jumped: Option<(i32, JumpCause)>,
    /// The last thing the Borg did, or that happened to it, that moves it
    /// (`time.now` and kind), for the next jump's cause (review of #1087).
    pub cause: Option<(i32, JumpCause)>,
    /// A jump left the staircases far: read Recall at danger 0.
    pub recall_due: bool,
    /// The walk back of the return leash goes to the nearest staircase of
    /// either kind (set when `flow_leash_any_stair` sets `goal.less`).
    pub return_any: bool,
    /// The step maps of this think (`time.now`), by their sources: see
    /// `steps_map` (review of #1087: one spread a think, not one a grid).
    pub maps: Vec<(i32, Vec<(i32, i32)>, crate::borg_flow::BorgData)>,
}

/// What moved the Borg (review of #1087, finding 3 of the re-review: the
/// cause, not only the length, tells a teleport from Phase Door).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum JumpCause {
    /// Nothing the Borg knows of (a monster's teleport-to, ...): judged by
    /// the length, `flow_leash_recall_min_jump`.
    #[default]
    Unknown,
    /// Its own short jump: Phase Door (`TELEPORT:10`; scroll, spell,
    /// activation) or the aimed Dimension Door (`TELEPORT_TO`).
    OwnShort,
    /// Its own long teleport: a Teleportation scroll (`TELEPORT:M60`), staff
    /// or activation, Teleport Self (`TELEPORT:M$M`), Portal, Shadow Shift
    /// (`TELEPORT:30`) (class.txt, object.txt).
    OwnLong,
    /// "You hit a teleport trap!" (`TELEPORT:M80`, trap.txt).
    Trap,
}

/// Is a jump of `jump` grids with this cause a teleport for
/// `flow_leash_teleported_recall`?
fn is_teleport(cause: JumpCause, jump: i32, min_jump: i32) -> bool {
    match cause {
        JumpCause::OwnShort => false,
        JumpCause::OwnLong | JumpCause::Trap => true,
        JumpCause::Unknown => jump >= min_jump,
    }
}

/// Record a cause for the next jump (with the switch on only).
fn mark(st: &mut BorgState, cause: JumpCause) {
    if leash_holes().teleported_recall {
        st.leash.cause = Some((st.borg.time.now, cause));
    }
}

/// `borg_read_scroll` read one (after its keys): Phase Door or a teleport.
pub fn own_scroll(st: &mut BorgState, sval: i32) {
    let v = &st.item_val;
    if sval == v.sv_scroll_phase_door {
        mark(st, JumpCause::OwnShort);
    } else if sval == v.sv_scroll_teleport {
        mark(st, JumpCause::OwnLong);
    }
}

/// `borg_use_staff` used one: a staff of Teleportation.
pub fn own_staff(st: &mut BorgState, sval: i32) {
    if sval == st.item_val.sv_staff_teleportation {
        mark(st, JumpCause::OwnLong);
    }
}

/// `borg_activate_item` activated one: a phase or a long teleport.
pub fn own_activation(st: &mut BorgState, activation: i32) {
    if activation == st.act.tele_phase {
        mark(st, JumpCause::OwnShort);
    } else if activation == st.act.tele_long {
        mark(st, JumpCause::OwnLong);
    }
}

/// `borg_spell` cast one.
pub fn own_spell(st: &mut BorgState, spell: crate::borg_headers::borg_magic_h::borg_spells) {
    use crate::borg_headers::borg_magic_h::{DIMENSION_DOOR, PHASE_DOOR, PORTAL, SHADOW_SHIFT, TELEPORT_SELF};
    if [PHASE_DOOR, DIMENSION_DOOR].contains(&spell) {
        mark(st, JumpCause::OwnShort);
    } else if [TELEPORT_SELF, PORTAL, SHADOW_SHIFT].contains(&spell) {
        mark(st, JumpCause::OwnLong);
    }
}

/// `borg_parse` read "You hit a teleport trap!".
pub fn teleport_trap(st: &mut BorgState) {
    mark(st, JumpCause::Trap);
}

/// The cause of a jump seen at `now`: a mark from the last few thinks
/// (the action's think, or the trap's message this update), else unknown.
fn cause_at(mark: Option<(i32, JumpCause)>, now: i32) -> JumpCause {
    match mark {
        Some((t, c)) if now >= t && now - t <= 3 => c,
        _ => JumpCause::Unknown,
    }
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
/// `arrival` gives the steps from the nearest anchor (`LeashState::anchors`;
/// `None`: none, or none reaches it), asked only when needed. C: a grid with no staircase to
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

/// Can the Borg walk onto a grid of this terrain? Known terrain only, as
/// the flow's own wall rules (`spread_blocked` in borg_flow.rs: no wall or
/// rubble but passable rubble, no permanent wall, lava only when immune to
/// fire, no unknown grid), and nothing else: monsters, traps and danger are
/// what the Borg meets on the way, not the way (review of #1087).
fn walkable(feat: u32, fire_immune: bool) -> bool {
    use crate::headers::list_terrain_h::{FEAT_LAVA, FEAT_NONE, FEAT_PASS_RUBBLE, FEAT_SECRET};
    feat != FEAT_NONE && (feat < FEAT_SECRET || feat == FEAT_PASS_RUBBLE || (feat == FEAT_LAVA && fire_immune))
}

/// Breadth-first steps (8 directions, one a move) from the nearest of
/// `sources`, (y, x), over the grids `walk` allows, inside the map's
/// border; 255 = not reached, at most 250.
fn terrain_steps(walk: impl Fn(i32, i32) -> bool, sources: &[(i32, i32)]) -> crate::borg_flow::BorgData {
    let (h, w) = (AUTO_MAX_Y, AUTO_MAX_X);
    let mut map: crate::borg_flow::BorgData = vec![[255u8; AUTO_MAX_X as usize]; h as usize];
    let mut queue = std::collections::VecDeque::new();
    for &(y, x) in sources {
        if (0..h).contains(&y) && (0..w).contains(&x) && map[y as usize][x as usize] != 0 {
            map[y as usize][x as usize] = 0;
            queue.push_back((y, x));
        }
    }
    while let Some((y, x)) = queue.pop_front() {
        let n = map[y as usize][x as usize] + 1;
        if n > 250 {
            continue;
        }
        for dy in -1..=1 {
            for dx in -1..=1 {
                let (ny, nx) = (y + dy, x + dx);
                if (dy, dx) == (0, 0) || ny < 1 || nx < 1 || ny >= h - 1 || nx >= w - 1 {
                    continue;
                }
                if map[ny as usize][nx as usize] <= n || !walk(ny, nx) {
                    continue;
                }
                map[ny as usize][nx as usize] = n;
                queue.push_back((ny, nx));
            }
        }
    }
    map
}

/// The steps to (y, x) on a `terrain_steps` map: a reached grid's own; a
/// grid the walk does not enter (an unknown grid the dark flow aims at, a
/// wall) is one more than its nearest reached neighbour, the frontier
/// (review of #1087); `None` otherwise.
fn steps_at(map: &crate::borg_flow::BorgData, walk: impl Fn(i32, i32) -> bool, y: i32, x: i32) -> Option<i32> {
    let at = |y: i32, x: i32| -> Option<i32> {
        let row = map.get(usize::try_from(y).ok()?)?;
        let v = *row.get(usize::try_from(x).ok()?)?;
        (v != 255).then_some(i32::from(v))
    };
    if let Some(v) = at(y, x) {
        return Some(v);
    }
    if walk(y, x) {
        return None;
    }
    (-1..=1).flat_map(|dy| (-1..=1).map(move |dx| (dy, dx))).filter(|&d| d != (0, 0)).filter_map(|(dy, dx)| at(y + dy, x + dx)).min().map(|v| v + 1)
}

/// The Borg's `walkable` on its map now.
fn walk_here(st: &BorgState) -> impl Fn(i32, i32) -> bool + '_ {
    let fire = st.borg.tr(BI_IFIRE) != 0;
    move |y, x| (0..AUTO_MAX_Y).contains(&y) && (0..AUTO_MAX_X).contains(&x) && walkable(u32::from(st.cave.grid(x, y).feat), fire)
}

/// The `terrain_steps` map of `sources` this think (`time.now`), spread
/// once and cached in `LeashState::maps`; its index there.
fn steps_map(st: &mut BorgState, sources: &[(i32, i32)]) -> Option<usize> {
    if sources.is_empty() {
        return None;
    }
    let now = st.borg.time.now;
    st.leash.maps.retain(|(t, _, _)| *t == now);
    if let Some(i) = st.leash.maps.iter().position(|(_, s, _)| s == sources) {
        return Some(i);
    }
    let map = terrain_steps(walk_here(st), sources);
    st.leash.maps.push((now, sources.to_vec(), map));
    Some(st.leash.maps.len() - 1)
}

/// The walking steps from the nearest of `sources` to (y, x) over known
/// terrain (`steps_map`, `steps_at`); `None`: no source, or not reached.
fn steps_from(st: &mut BorgState, y: i32, x: i32, sources: &[(i32, i32)]) -> Option<i32> {
    let i = steps_map(st, sources)?;
    steps_at(&st.leash.maps[i].2, walk_here(st), y, x)
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
    beyond_verdict(stairs, limit, unknown_steps, || {
        let anchor = current_anchor(st)?;
        steps_from(st, y, x, &[anchor])
    })
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
        *l = LeashState { level_time: Some(level), anchors: vec![now], last: Some(now), ..LeashState::default() };
        return;
    }
    if !tracking() {
        return;
    }
    // Review of #1087: a mark belongs to the first update after the action
    // (or the trap's message in this one), jump or not.
    let mark = l.cause.take();
    if let Some(d) = l.last.and_then(|last| jump(last, now)) {
        let cause = cause_at(mark, st.borg.time.now);
        let l = &mut st.leash;
        l.jumped = Some(match l.jumped {
            Some((j, c)) if j >= d => (j, c),
            _ => (d, cause),
        });
        if leash_holes().unknown_stair_steps > 0 {
            let anchors = st.leash.anchors.clone();
            let back = steps_at(&terrain_steps(walk_here(st), &anchors), walk_here(st), now.0, now.1);
            add_anchor(&mut st.leash.anchors, now, back.is_some());
        }
    }
    st.leash.last = Some(now);
}

/// The most anchors the arrival leash keeps (the arrival grid and the
/// landings with no known route back).
const MAX_ANCHORS: usize = 16;

/// A jump landed on `landing`: it becomes an anchor unless the known
/// terrain leads back to one (`reached`). A full list replaces its newest
/// landing, so the arrival grid and the current landing always stay (review
/// of #1087).
fn add_anchor(anchors: &mut Vec<(i32, i32)>, landing: (i32, i32), reached: bool) {
    if reached || anchors.contains(&landing) {
        return;
    }
    if anchors.len() >= MAX_ANCHORS {
        anchors.pop();
    }
    anchors.push(landing);
}

/// The first anchor, oldest first, that `reaches` the Borg's grid: the
/// arrival grid while the Borg's area is connected to it, a landing only
/// while it is not (review of #1087: a reconnected area returns to the
/// arrival grid). `None`: no anchor reaches it.
fn first_reaching(anchors: &[(i32, i32)], mut reaches: impl FnMut((i32, i32)) -> bool) -> Option<(i32, i32)> {
    anchors.iter().copied().find(|&a| reaches(a))
}

/// The anchor the arrival leash measures from now (`first_reaching`, over
/// known terrain).
fn current_anchor(st: &mut BorgState) -> Option<(i32, i32)> {
    let anchors = st.leash.anchors.clone();
    let (y, x) = (st.borg.c.y, st.borg.c.x);
    first_reaching(&anchors, |a| steps_from(st, y, x, &[a]).is_some())
}

/// A "You failed ..." message: the action that marked a cause did not
/// happen (review of #1087).
pub fn action_failed(st: &mut BorgState) {
    st.leash.cause = None;
}

/// The early "far while near" filters of `take_select` and the kill flow
/// with `flow_leash_any_stair` (review of #1087): skip the target (y, x)
/// when the Borg is within `leash` of the nearest known staircase of either
/// kind and the target is `leash` or more from it, both in walking steps
/// over known terrain (`steps_from`; a staircase across a wall is not
/// near). `None`: the switch is off, or no staircase is known (C's
/// up-staircase filter decides).
pub fn far_while_near(st: &mut BorgState, y: i32, x: i32, leash: i32) -> Option<bool> {
    if !any_stair(st) {
        return None;
    }
    let all = all_stairs(st);
    if all.is_empty() {
        return None;
    }
    let (by, bx) = (st.borg.c.y, st.borg.c.x);
    let near = steps_from(st, by, bx, &all);
    let far = steps_from(st, y, x, &all);
    Some(far_while_near_verdict(near, far, leash))
}

/// `far_while_near`'s verdict: C's `b_j <= leash && j >= leash`, with a
/// distance the walk does not reach never near and never far.
fn far_while_near_verdict(near: Option<i32>, far: Option<i32>, leash: i32) -> bool {
    matches!((near, far), (Some(n), Some(f)) if n <= leash && f >= leash)
}

/// The verdict of `flow_leash_teleported_recall` on a jump of `jump` grids
/// with its cause: a teleport (`is_teleport`: the Borg's own Phase Door
/// never, its own teleport or a teleport trap always, an unknown cause from
/// `min_jump` grids: Phase Door asks for 9 to 11, `TELEPORT:10` randomised
/// by a quarter in `effect_handler_TELEPORT`, effect-handler-general.c, and
/// lands on the eligible grid nearest that) that leaves the nearest known
/// staircase (`stairs` steps; `None`: none known or reachable) more than
/// `steps` away.
fn jump_strands(jump: i32, cause: JumpCause, min_jump: i32, stairs: Option<i32>, steps: i32) -> bool {
    is_teleport(cause, jump, min_jump) && stairs.is_none_or(|cost| cost > steps)
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
    if let Some((d, cause)) = st.leash.jumped.take() {
        let steps = h.recall_steps;
        if is_teleport(cause, d, h.recall_min_jump) {
            let all = all_stairs(st);
            let (y, x) = (st.borg.c.y, st.borg.c.x);
            let cost = steps_from(st, y, x, &all);
            if jump_strands(d, cause, h.recall_min_jump, cost, steps) {
                st.leash.recall_due = true;
                let far = cost.map_or("no staircase known or reachable".to_string(), |c| format!("{c} steps from the nearest staircase"));
                borg_note(st, &format!("# Leash: a {d}-grid jump ({cause:?}) left the Borg {far} (more than {steps}): Recall at danger 0"));
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

    /// 1024822556: 79 steps from the up staircase after the trap.
    #[test]
    fn a_jump_strands_the_borg() {
        use JumpCause::*;
        assert!(jump_strands(84, Trap, 12, Some(79), 30));
        assert!(!jump_strands(84, Trap, 12, Some(30), 30));
        assert!(jump_strands(84, Unknown, 12, None, 30), "no staircase known or reachable");
        // Review of #1087: 592754909's own 9-grid Phase Door, 47 steps out.
        assert!(!jump_strands(9, OwnShort, 12, Some(47), 30));
        // The re-review: a Phase Door whose nearest eligible landing is far.
        assert!(!jump_strands(14, OwnShort, 12, Some(47), 30));
        // The Borg's own teleport counts at any length.
        assert!(jump_strands(8, OwnLong, 12, None, 30));
        // An unknown cause falls back on the length.
        assert!(!jump_strands(11, Unknown, 12, None, 30));
        assert!(jump_strands(12, Unknown, 12, Some(31), 30));
    }

    /// The cause is the last mark of the last few thinks.
    #[test]
    fn the_cause_of_a_jump() {
        use JumpCause::*;
        assert_eq!(cause_at(Some((100, OwnShort)), 101), OwnShort);
        assert_eq!(cause_at(Some((100, Trap)), 100), Trap);
        assert_eq!(cause_at(Some((100, OwnLong)), 104), Unknown, "too old");
        assert_eq!(cause_at(None, 100), Unknown);
    }

    /// A map of known terrain: `#` wall, `.` floor, ` ` unknown, `<` `>` stairs.
    fn plan<'a>(rows: &'a [&'a str]) -> impl Fn(i32, i32) -> bool + 'a {
        move |y, x| {
            let c = usize::try_from(y).ok().and_then(|y| rows.get(y)).and_then(|r| r.as_bytes().get(usize::try_from(x).ok()?).copied());
            matches!(c, Some(b'.' | b'<' | b'>'))
        }
    }

    /// Review of #1087 (re-review 1): the route over known terrain. A
    /// staircase across an unexplored gap is not near: the walk goes round.
    #[test]
    fn steps_over_known_terrain() {
        let rows = [
            "##########",
            "#.>#  #..#",
            "#..#  #..#",
            "#.#######.",
            "#........#",
            "##########",
        ];
        let walk = plan(&rows);
        let map = terrain_steps(&walk, &[(1, 2)]);
        // The Borg at (1, 7), five grids from the staircase across the gap:
        // over known floor it is 12 steps round through row 4.
        assert_eq!(steps_at(&map, &walk, 1, 7), Some(12));
        // An unknown grid in the gap is not reached; the frontier rule
        // gives it no neighbour either (walls around), so no distance.
        assert_eq!(steps_at(&map, &walk, 1, 4), None);
        // A wall beside the staircase is one more than it.
        assert_eq!(steps_at(&map, &walk, 2, 3), Some(1));
    }

    /// The dark flow aims at unknown grids beside known floor: they get one
    /// more step than the floor beside them (the frontier).
    #[test]
    fn an_unknown_target_beside_known_floor() {
        let rows = [
            "#######",
            "#>....  ",
            "#######",
        ];
        let walk = plan(&rows);
        let map = terrain_steps(&walk, &[(1, 1)]);
        assert_eq!(steps_at(&map, &walk, 1, 5), Some(4));
        assert_eq!(steps_at(&map, &walk, 1, 6), Some(5), "the frontier");
        assert_eq!(steps_at(&map, &walk, 1, 7), None, "beyond the frontier");
    }

    /// Review of #1087 (re-review 2): the walk ignores monsters, traps and
    /// danger, so a monster in the only corridor back does not lift the
    /// arrival leash: the map is terrain only (`walkable`).
    #[test]
    fn terrain_only() {
        use crate::headers::list_terrain_h::*;
        assert!(walkable(FEAT_FLOOR, false));
        assert!(walkable(FEAT_CLOSED, false));
        assert!(walkable(FEAT_LESS, false) && walkable(FEAT_MORE, false));
        assert!(walkable(FEAT_PASS_RUBBLE, false));
        assert!(!walkable(FEAT_NONE, false), "unknown");
        assert!(!walkable(FEAT_RUBBLE, false) && !walkable(FEAT_GRANITE, false) && !walkable(FEAT_PERM, false));
        assert!(!walkable(FEAT_LAVA, false) && walkable(FEAT_LAVA, true));
    }

    /// Review of #1087: a landing with no known route back is an anchor;
    /// one the terrain leads back from is not; at most `MAX_ANCHORS`.
    #[test]
    fn a_disconnected_landing_is_an_anchor() {
        let mut a = vec![(10, 10)];
        add_anchor(&mut a, (40, 90), false);
        assert_eq!(a, vec![(10, 10), (40, 90)]);
        add_anchor(&mut a, (12, 20), true);
        assert_eq!(a.len(), 2, "reached: no anchor");
        add_anchor(&mut a, (40, 90), false);
        assert_eq!(a.len(), 2, "no duplicate");
        for i in 0..40 {
            add_anchor(&mut a, (1, i), false);
        }
        assert_eq!(a.len(), MAX_ANCHORS);
        assert_eq!(a[0], (10, 10), "the arrival grid stays");
        assert_eq!(a[MAX_ANCHORS - 1], (1, 39), "the newest landing is kept");
        // Reconnected: the arrival grid reaches the Borg, so it is the one
        // measured from, not the old landing 15 steps away.
        assert_eq!(first_reaching(&a, |_| true), Some((10, 10)));
        // Cut off: the first landing that reaches it.
        assert_eq!(first_reaching(&a, |g| g == (1, 39)), Some((1, 39)));
        assert_eq!(first_reaching(&a, |_| false), None);
        // The verdict measures from the nearest anchor: 25 steps from the
        // landing is beyond 20 even with the arrival unreachable.
        assert!(beyond_verdict(None, 12, 20, || Some(25)));
    }

    /// Review of #1087: the early filters in walking steps. A staircase 10
    /// grids away across a wall is 34 steps away: the Borg is not near it,
    /// so the item 3 moves away (36 steps from it) is not skipped.
    #[test]
    fn far_while_near_in_steps() {
        assert!(!far_while_near_verdict(Some(34), Some(36), 12));
        assert!(far_while_near_verdict(Some(10), Some(13), 12));
        assert!(!far_while_near_verdict(None, Some(13), 12));
        assert!(!far_while_near_verdict(Some(10), None, 12));
    }

    /// The default state follows nothing.
    #[test]
    fn a_fresh_state_is_empty() {
        let l = LeashState::default();
        assert!(l.level_time.is_none() && l.anchors.is_empty() && l.jumped.is_none() && l.cause.is_none());
        assert!(!l.recall_due && !l.return_any);
    }
}
