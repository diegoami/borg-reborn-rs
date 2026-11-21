//! Lightless dungeon recovery (#811). No ordinary exploration goal runs
//! while this recovery owns the turn.

use std::collections::{HashSet, VecDeque};

use crate::borg_caution::{unseen_bump_blocked, unseen_bump_step};
use crate::borg_danger::borg_danger;
use crate::borg_escape::borg_recall;
use crate::borg_flow::borg_ddxy_ddd;
use crate::borg_headers::borg_trait_h::{
    BI_CDEPTH, BI_CLASS, BI_CURHP, BI_ISCONFUSED, BI_LIGHT, BI_RECALL,
};
use crate::borg_io::{borg_keypress, borg_queue_direction, ESCAPE};
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

const CLASS_NECROMANCER: i32 = 4;
const BORG_GLOW: u16 = 0x02;
// Use the existing stair flow's maximum distance as the total action limit.
// Allow a shorter search when the map contains no known route out.
const MAX_ACTIONS: usize = 250;
const MAX_SEARCH_ACTIONS: usize = 32;

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

#[derive(Default)]
pub(crate) struct Recovery {
    depth: i32,
    actions_left: usize,
    search_left: usize,
    climbed: HashSet<Grid>,
    stood_on: HashSet<Grid>,
    failed: HashSet<(Grid, Step)>,
    pending: Option<(Grid, Step)>,
}

pub(crate) fn update(st: &mut BorgState) -> bool {
    let depth = st.borg.tr(BI_CDEPTH);
    let needs_light =
        depth > 0 && st.borg.tr(BI_LIGHT) <= 0 && st.borg.tr(BI_CLASS) != CLASS_NECROMANCER;
    let recovery = &mut st.think_dungeon.lightless;
    if !needs_light {
        *recovery = Recovery::default();
    } else if recovery.depth != depth {
        *recovery = Recovery {
            depth,
            actions_left: MAX_ACTIONS,
            search_left: MAX_SEARCH_ACTIONS,
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
                self.failed.insert((from, step));
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
                if self.failed.contains(&(from, step)) {
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

pub(crate) fn recover(st: &mut BorgState) {
    st.borg.goal.type_0 = 0;
    let at = (st.borg.c.x, st.borg.c.y);
    st.think_dungeon.lightless.observe(at);
    if st.think_dungeon.lightless.actions_left == 0 {
        handoff(st, "lightless recovery action limit reached");
        return;
    }
    if borg_think_dungeon_light(st, true) {
        st.think_dungeon.lightless.actions_left -= 1;
        return;
    }
    if st.borg.tr(BI_ISCONFUSED) != 0 {
        handoff(st, "lightless recovery cannot navigate while confused");
        return;
    }

    let up_allowed = !engine::player_opt(OPT_birth_force_descend);
    let seek_recall_light = st.borg.tr(BI_RECALL) != 0 && engine::no_light();
    let recovery = std::mem::take(&mut st.think_dungeon.lightless);
    let route = recovery.route(at, |grid| terrain(st, grid, up_allowed, seek_recall_light));
    st.think_dungeon.lightless = recovery;
    st.think_dungeon.lightless.actions_left -= 1;

    if matches!(route, None | Some(Route::Search(_))) && borg_recall(st) {
        return;
    }

    match route {
        Some(Route::Climb) => {
            st.think_dungeon.lightless.climbed.insert(at);
            borg_keypress(st, u32::from(b'<'));
        }
        Some(Route::Return(step) | Route::Search(step)) => {
            let recovery = &mut st.think_dungeon.lightless;
            if matches!(route, Some(Route::Search(_))) {
                recovery.search_left -= 1;
            }
            recovery.pending = Some((at, step));

            let (x, y) = step.to;
            // The movement keys use the Angband keypad layout (cave.c).
            let dir = u32::from(b'0') + (5 + (x - at.0) - 3 * (y - at.1)) as u32;
            if step.tile == Tile::Door {
                // cmd-cave.c: do_cmd_open can open an adjacent known door.
                borg_keypress(st, u32::from(b'o'));
                borg_queue_direction(st, dir);
            } else {
                unseen_bump_step(st, x, y);
                borg_keypress(st, dir);
            }
        }
        None if st.think_dungeon.lightless.search_left == 0 => {
            handoff(st, "lightless recovery search limit reached");
        }
        None => handoff(st, "lightless recovery found no reachable safe exit"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn recovery() -> Recovery {
        Recovery {
            actions_left: MAX_ACTIONS,
            search_left: MAX_SEARCH_ACTIONS,
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
