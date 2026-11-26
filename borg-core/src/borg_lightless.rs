//! Lightless dungeon recovery (#811). No ordinary exploration goal runs
//! while this recovery owns the turn.

use std::collections::{HashMap, HashSet, VecDeque};

use crate::borg_caution::{lightless_self_care, unseen_bump_blocked, unseen_bump_step};
use crate::borg_danger::borg_danger;
use crate::borg_escape::borg_recall;
use crate::borg_flow::borg_ddxy_ddd;
use crate::borg_flow_misc::borg_extract_dir;
use crate::borg_headers::borg_trait_h::{BI_CDEPTH, BI_CURHP, BI_ISCONFUSED, BI_LIGHT, BI_RECALL};
use crate::borg_io::{borg_keypress, borg_keypresses, borg_queue_direction, ESCAPE};
use crate::borg_think::borg_oops;
use crate::borg_think_dungeon_util::borg_think_dungeon_light;
use crate::engine;
use crate::headers::list_options_h::OPT_birth_force_descend;
use crate::headers::list_terrain_h::{
    FEAT_BROKEN, FEAT_CLOSED, FEAT_FLOOR, FEAT_LESS, FEAT_MORE, FEAT_NONE, FEAT_OPEN,
    FEAT_PASS_RUBBLE,
};
use crate::params::{value as param, Param};
use crate::state::BorgState;

const BORG_GLOW: u16 = 0x02;
const NO_EXIT: &str = "lightless recovery found no reachable safe exit";
const SEARCH_LIMIT: &str = "lightless recovery search limit reached";

type Grid = (i32, i32);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum Tile {
    Blocked,
    Floor,
    Door,
    Up,
    RecallLight,
    Unknown,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct Step {
    to: Grid,
    tile: Tile,
}

#[derive(Default, Clone, PartialEq)]
pub(crate) struct Recovery {
    depth: i32,
    actions_left: usize,
    search_left: usize,
    climbed: HashSet<Grid>,
    stood_on: HashSet<Grid>,
    failed: HashMap<(Grid, Step), usize>,
    door_attempts: usize,
    pending: Option<(Grid, Step)>,
}

pub(crate) fn update(st: &mut BorgState) -> bool {
    let depth = st.borg.tr(BI_CDEPTH);
    // borg_trait.rs sets the effective radius for necromancers to one.
    let needs_light = depth > 0 && st.borg.tr(BI_LIGHT) <= 0;
    let recovery = &mut st.think_dungeon.lightless;
    if !needs_light {
        *recovery = Recovery::default();
    } else if recovery.depth != depth {
        *recovery = Recovery {
            depth,
            actions_left: param(Param::lightless_recovery_actions) as usize,
            search_left: param(Param::lightless_search_actions) as usize,
            door_attempts: param(Param::lightless_door_attempts) as usize,
            ..Recovery::default()
        };
    }
    needs_light
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Route {
    Climb,
    Return(Step),
    Search(Step),
}

impl Recovery {
    fn observe(&mut self, at: Grid) {
        self.stood_on.insert(at);
        if let Some((from, step)) = self.pending.take() {
            if at == from {
                // An opened door will now be a Floor, so it can be entered.
                *self.failed.entry((from, step)).or_default() += 1;
            }
        }
    }

    fn route(&self, start: Grid, mut tile_at: impl FnMut(Grid) -> Tile) -> Option<Route> {
        if tile_at(start) == Tile::Up && !self.climbed.contains(&start) {
            return Some(Route::Climb);
        }

        let mut queue = VecDeque::from([(start, None, 0)]);
        let mut seen = HashSet::from([start]);
        let mut recall_light = None;
        let mut frontier = None;

        while let Some((from, first, distance)) = queue.pop_front() {
            if distance >= self.actions_left {
                continue;
            }
            for direction in 0..8 {
                let (dx, dy) = borg_ddxy_ddd(direction);
                let to = (from.0 + dx, from.1 + dy);
                if seen.contains(&to) {
                    continue;
                }

                let tile = match tile_at(to) {
                    Tile::Unknown if self.stood_on.contains(&to) => Tile::Floor,
                    tile => tile,
                };
                let step = Step { to, tile };
                let limit = if tile == Tile::Door {
                    self.door_attempts
                } else {
                    1
                };
                if self
                    .failed
                    .get(&(from, step))
                    .is_some_and(|&count| count >= limit)
                {
                    continue;
                }
                seen.insert(to);
                let next = first.unwrap_or(step);

                match tile {
                    Tile::Blocked => continue,
                    Tile::Up if !self.climbed.contains(&to) => return Some(Route::Return(next)),
                    Tile::RecallLight => {
                        recall_light.get_or_insert(Route::Return(next));
                    }
                    Tile::Unknown => {
                        // Probe only the edge of the known map. Never plan
                        // a route through multiple unobserved grids.
                        if distance < self.search_left {
                            frontier.get_or_insert(Route::Search(next));
                        }
                        continue;
                    }
                    _ => {}
                }
                queue.push_back((to, Some(next), distance + 1));
            }
        }
        recall_light.or(frontier)
    }
}

fn terrain(st: &mut BorgState, at: Grid, up_allowed: bool, seek_recall_light: bool) -> Tile {
    let (x, y) = at;
    if !engine::square_in_bounds_fully(x, y) {
        return Tile::Blocked;
    }
    let grid = *st.cave.grid(x, y);
    if grid.kill != 0 || grid.trap || unseen_bump_blocked(st, x, y) {
        return Tile::Blocked;
    }

    let tile = match u32::from(grid.feat) {
        FEAT_LESS if up_allowed => Tile::Up,
        FEAT_CLOSED => Tile::Door,
        FEAT_BROKEN | FEAT_FLOOR | FEAT_LESS | FEAT_MORE | FEAT_OPEN | FEAT_PASS_RUBBLE => {
            if seek_recall_light && grid.info & BORG_GLOW != 0 {
                Tile::RecallLight
            } else {
                Tile::Floor
            }
        }
        FEAT_NONE => Tile::Unknown,
        _ => return Tile::Blocked,
    };
    // Use actual HP so temporary flight bravery cannot relax this gate.
    let danger_limit = st.borg.tr(BI_CURHP) / param(Param::recover_danger_divisor);
    if at != (st.borg.c.x, st.borg.c.y) && borg_danger(st, y, x, 1, true, false) > danger_limit {
        return Tile::Blocked;
    }
    tile
}

fn handoff(st: &mut BorgState, reason: &str) {
    st.think_dungeon.lightless = Recovery::default();
    borg_oops(st, reason);
    // Return a key to the caller so its empty-queue abort does not replace
    // the reason. The inactive Borg removes its hook on the next read.
    borg_keypress(st, ESCAPE);
}

#[derive(Debug, PartialEq, Eq)]
enum Decision {
    Recall,
    Climb,
    Move {
        step: Step,
        searching: bool,
        direction: u32,
    },
    Stop(&'static str),
}

fn decide(
    route: Option<Route>,
    at: Grid,
    search_left: usize,
    recall: impl FnOnce() -> bool,
) -> Decision {
    if matches!(route, None | Some(Route::Search(_))) && recall() {
        return Decision::Recall;
    }
    match route {
        Some(Route::Climb) => Decision::Climb,
        Some(Route::Return(step) | Route::Search(step)) => Decision::Move {
            step,
            searching: matches!(route, Some(Route::Search(_))),
            direction: u32::from(b'0') + borg_extract_dir(at.1, at.0, step.to.1, step.to.0) as u32,
        },
        None if search_left == 0 => Decision::Stop(SEARCH_LIMIT),
        None => Decision::Stop(NO_EXIT),
    }
}

fn prepare_for_return(st: &mut BorgState) -> bool {
    if st.borg.goal.recalling == 0 {
        if crate::borg_light::borg_restore_light(st) {
            return true;
        }
        let danger = borg_danger(st, st.borg.c.y, st.borg.c.x, 1, true, false);
        if lightless_self_care(st, danger) {
            return true;
        }
    }
    borg_think_dungeon_light(st, true)
}

pub(crate) fn recover(st: &mut BorgState) {
    st.borg.goal.type_0 = 0;
    let at = (st.borg.c.x, st.borg.c.y);
    st.think_dungeon.lightless.observe(at);
    if st.think_dungeon.lightless.actions_left == 0 {
        handoff(st, "lightless recovery action limit reached");
        return;
    }
    if prepare_for_return(st) {
        st.think_dungeon.lightless.actions_left -= 1;
        return;
    }
    if st.borg.tr(BI_ISCONFUSED) != 0 {
        if borg_recall(st) {
            st.think_dungeon.lightless.actions_left -= 1;
        } else {
            handoff(st, "lightless recovery cannot navigate while confused");
        }
        return;
    }

    let up_allowed = !engine::player_opt(OPT_birth_force_descend);
    let seek_recall_light = st.borg.tr(BI_RECALL) != 0 && engine::no_light();
    let recovery = std::mem::take(&mut st.think_dungeon.lightless);
    let route = recovery.route(at, |grid| terrain(st, grid, up_allowed, seek_recall_light));
    st.think_dungeon.lightless = recovery;
    st.think_dungeon.lightless.actions_left -= 1;

    let search_left = st.think_dungeon.lightless.search_left;
    match decide(route, at, search_left, || borg_recall(st)) {
        Decision::Recall => {}
        Decision::Climb => {
            st.think_dungeon.lightless.climbed.insert(at);
            borg_keypress(st, u32::from(b'<'));
        }
        Decision::Move {
            step,
            searching,
            direction,
        } => {
            let recovery = &mut st.think_dungeon.lightless;
            if searching {
                recovery.search_left -= 1;
            }
            recovery.pending = Some((at, step));

            let (x, y) = step.to;
            if step.tile == Tile::Door {
                // ui-input.c:1817 accepts a count prefixed by '0'.
                // A count of one avoids CMD_OPEN's default 99 repeats
                // (cmd-core.c:82,342,391) so we reassess after each try.
                borg_keypresses(st, b"01o");
                borg_queue_direction(st, direction);
            } else {
                unseen_bump_step(st, x, y);
                borg_keypress(st, direction);
            }
        }
        Decision::Stop(reason) => handoff(st, reason),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn recovery() -> Recovery {
        Recovery {
            actions_left: 250,
            search_left: 32,
            door_attempts: 3,
            ..Recovery::default()
        }
    }

    #[test]
    fn takes_known_upstairs_before_a_nearer_frontier() {
        let start = (85, 12);
        let door = Step {
            to: (86, 12),
            tile: Tile::Door,
        };
        let stairs = (87, 12);
        let map = |grid| match grid {
            (85, 11) => Tile::Unknown,
            (86, 12) => Tile::Door,
            (87, 12) => Tile::Up,
            _ => Tile::Blocked,
        };
        let mut recovery = recovery();
        assert_eq!(recovery.route(start, map), Some(Route::Return(door)));
        recovery.pending = Some((start, door));
        recovery.observe(start);
        // Opening a door changes the action even when our position is unchanged.
        assert_eq!(
            recovery.route(start, |g| if g == door.to { Tile::Floor } else { map(g) }),
            Some(Route::Return(Step {
                tile: Tile::Floor,
                ..door
            })),
        );
        assert_eq!(recovery.route(stairs, map), Some(Route::Climb));
    }

    #[test]
    fn incident_811_does_not_reverse_toward_the_vein() {
        // FBorg's east and west grids from #811. The vein would send
        // it east from the west grid. Ordinary exploration would send
        // it west again from the east grid.
        let west = (85, 12);
        let east = (86, 12);
        let probe = Step {
            to: (84, 12),
            tile: Tile::Unknown,
        };
        let map = |g| match g {
            (85, 12) | (86, 12) => Tile::Floor,
            (84, 12) => Tile::Unknown,
            _ => Tile::Blocked,
        };
        let mut recovery = recovery();
        assert_eq!(
            recovery.route(east, map),
            Some(Route::Search(Step {
                to: west,
                tile: Tile::Floor
            })),
        );
        assert_eq!(recovery.route(west, map), Some(Route::Search(probe)));
        // The probe hits a wall with no visible terrain update. Walking
        // east again would only put us back in the old loop.
        recovery.pending = Some((west, probe));
        recovery.observe(west);
        assert_eq!(recovery.route(west, map), None);
        assert_eq!(recovery.route(east, map), None);
    }

    #[test]
    fn retries_an_unopened_door_only_within_the_budget() {
        let at = (85, 12);
        let door = Step {
            to: (86, 12),
            tile: Tile::Door,
        };
        let map = |grid| match grid {
            (85, 12) => Tile::Floor,
            (86, 12) => Tile::Door,
            (87, 12) => Tile::Up,
            _ => Tile::Blocked,
        };
        let mut recovery = recovery();
        for _ in 0..recovery.door_attempts {
            let route = recovery.route(at, map);
            assert_eq!(
                decide(route, at, recovery.search_left, || panic!(
                    "known exit takes priority"
                )),
                Decision::Move {
                    step: door,
                    searching: false,
                    direction: u32::from(b'6')
                },
            );
            recovery.pending = Some((at, door));
            recovery.observe(at);
        }
        assert_eq!(recovery.route(at, map), None);
    }

    #[test]
    fn recalls_before_probing_or_handing_back_control() {
        let at = (85, 12);
        let probe = Route::Search(Step {
            to: (86, 12),
            tile: Tile::Unknown,
        });
        assert_eq!(decide(Some(probe), at, 1, || true), Decision::Recall);
        assert_eq!(decide(None, at, 1, || true), Decision::Recall);
        assert_eq!(decide(None, at, 1, || false), Decision::Stop(NO_EXIT));
        assert_eq!(decide(None, at, 0, || false), Decision::Stop(SEARCH_LIMIT));
        assert_eq!(
            decide(Some(Route::Climb), at, 1, || panic!(
                "already on known upstairs"
            )),
            Decision::Climb,
        );
    }

    #[test]
    fn search_is_bounded_and_does_not_enter_blocked_ground() {
        let mut recovery = recovery();
        let start = (85, 12);
        let map = |grid| match grid {
            (85, 12) => Tile::Floor,
            (86, 12) => Tile::Unknown,
            (88, 12) => Tile::Up,
            _ => Tile::Blocked,
        };
        assert!(matches!(recovery.route(start, map), Some(Route::Search(_))));
        recovery.search_left = 0;
        assert_eq!(recovery.route(start, map), None);
    }
}
