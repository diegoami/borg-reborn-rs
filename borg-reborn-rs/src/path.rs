//! L4's paths (ARCHITECTURE.md L4; task T3.2, #18).
//!
//! Distance maps over the remembered level, from goal grids outward, with
//! per-grid entry costs that include threat. The C Borg's flow is a
//! breadth-first flood with a one-byte cost and a fear gate that refuses
//! grids outright (`borg-flow.c:borg_flow_spread:255-501`, the fear
//! thresholds at `borg-flow.c:442-474`); ours keeps the flood-from-goals
//! shape but carries threat as a *cost* (ARCHITECTURE.md L4), so a
//! dangerous grid is paid for rather than forbidden, and a route through
//! it stays available when it is the only one.
//!
//! The maps are computed for one think and die with it: [`Paths`] records
//! the world snapshot (`turn`, `thinks`) it was built for and refuses any
//! other, so a cached map can never outlive the observation it was
//! computed from — the stale-flow reuse of the mold deaths (better-borg-c
//! D0008, borg-rs D0006, `source404`) is impossible by construction.
//! Within a think, each goal's map is computed once and reused.

use crate::borg::feat;
use crate::knowledge::Knowledge;
use crate::world::World;
use std::cmp::Reverse;
use std::collections::BinaryHeap;

/// Directions 1–9 (keypad layout) as (dir, dx, dy), fixed order: the first
/// lowest neighbour wins a tie, so a step never depends on container
/// order (RULES 10).
const DIRS: [(u8, i32, i32); 8] = [
    (1, -1, 1),
    (2, 0, 1),
    (3, 1, 1),
    (4, -1, 0),
    (6, 1, 0),
    (7, -1, -1),
    (8, 0, -1),
    (9, 1, -1),
];

/// A place to walk toward. M3's set (PLAN.md): stairs, the exploration
/// frontier, and explicit targets (a seen object, a shop entrance).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Goal {
    /// The known down (`MORE`) or up (`LESS`) staircases. Flowing to a
    /// stair is the C Borg's `GOAL_BORE`/`GOAL_FLEE` shape
    /// (`borg-flow-stairs.c`).
    Stairs { up: bool },
    /// The exploration frontier: known, enterable grids with an unknown
    /// neighbour. The C Borg enqueues the unknown grids themselves
    /// (`borg-flow-dark.c:borg_flow_dark_interesting:46-58`); standing on
    /// the known edge of the unknown is what reveals them, so those are
    /// our targets — and a closed door next to the unknown is one
    /// (`FEAT_CLOSED` is "interesting" there too, borg-flow-dark.c:61).
    Frontier,
    /// These grids, whichever is nearest (the C Borg's `GOAL_TAKE` /
    /// `GOAL_MISC` shape, `borg-flow-take.c`, `borg-flow-misc.c`).
    Grids(Vec<(i32, i32)>),
}

impl Goal {
    /// The goal grids, in row-by-row order (deterministic).
    fn targets(&self, w: &World) -> Vec<(i32, i32)> {
        let level = &w.level;
        match self {
            Goal::Stairs { up } => {
                let want = if *up { feat::LESS } else { feat::MORE };
                let mut out = Vec::new();
                for y in 0..level.height {
                    for x in 0..level.width {
                        if level.at(x, y).is_some_and(|g| g.feat == want) {
                            out.push((x, y));
                        }
                    }
                }
                out
            }
            Goal::Frontier => {
                let mut out = Vec::new();
                for y in 0..level.height {
                    for x in 0..level.width {
                        let known = level.at(x, y).is_some_and(|g| g.feat != feat::NONE);
                        // Only grids the Borg could stand on lead
                        // anywhere; walls and shops are not frontier. The
                        // frontier already underfoot is not a destination:
                        // holding there cannot reveal its unknown neighbour,
                        // and treating it as unfinished held seven full-suite
                        // runs in explore/cooldown loops until starvation
                        // (T3.7's G0 check).
                        let standable = enter_cost(w, x, y).is_some();
                        if !known || !standable || (x, y) == player(w) {
                            continue;
                        }
                        let unknown = DIRS.iter().any(|(_, dx, dy)| {
                            level
                                .at(x + dx, y + dy)
                                .is_some_and(|g| g.feat == feat::NONE)
                        });
                        if unknown {
                            out.push((x, y));
                        }
                    }
                }
                out
            }
            Goal::Grids(gs) => gs.clone(),
        }
    }
}

/// The cost of entering a grid, in steps: `None` when the Borg cannot or
/// will not pass. Known walls, shops and never-seen grids do not pass; a
/// closed door is walked open first (`move_player` hands it to
/// `do_cmd_alter_aux`, which charges a move's energy without moving, so
/// passing costs opening plus entering); a web or a known trap costs
/// extra caution. Unknown grids are never transit: the flow may lead
/// *along* the unknown, never through it (the C Borg's `avoid` shape,
/// borg-flow.c:375-386).
pub fn enter_cost(w: &World, x: i32, y: i32) -> Option<u32> {
    let g = w.level.at(x, y)?;
    match g.feat {
        feat::NONE => None,
        feat::CLOSED => Some(2 + extras(g)),
        // Floors, open and broken doors, stairs, rubble: a step each.
        f if walkable(f) => Some(1 + extras(g)),
        _ => None,
    }
}

/// A web or a known trap makes a grid more expensive.
fn extras(g: &crate::world::Remembered) -> u32 {
    u32::from(g.webbed) * 2 + u32::from(g.trap) * 2
}

/// The cost of *standing* on a goal grid. Goals may stand where transit
/// may not pass: a shop entrance is passable terrain (terrain.txt gives
/// every store `PASSABLE`) and standing on it is the point — `,` there
/// opens the store — but [`enter_cost`] refuses to route *through* shops,
/// as the C Borg's spread skips a shop that is not its target
/// (borg-flow.c:371-374). A goal under a wall (a strategy bug) anchors
/// the flood at cost 0 so its neighbours are still reached.
fn standing_cost(w: &World, k: &Knowledge, x: i32, y: i32) -> u32 {
    match enter_cost(w, x, y) {
        Some(c) => c + threat(w, k, x, y),
        None if is_shop(w, x, y) => 1 + threat(w, k, x, y),
        None => 0,
    }
}

/// Whether the remembered grid is a shop entrance (the home included).
fn is_shop(w: &World, x: i32, y: i32) -> bool {
    w.level
        .at(x, y)
        .is_some_and(|g| (feat::STORE_GENERAL..=feat::HOME).contains(&g.feat))
}

/// Whether a known feature can be walked on at all (the minimal Borg's
/// `walkable`, minus the unknown grids it tries and the shops it must
/// not transit). Mirrors `list-terrain.h`: floors, doors, stairs, rubble.
fn walkable(f: u32) -> bool {
    matches!(
        f,
        feat::FLOOR
            | feat::CLOSED
            | feat::OPEN
            | feat::BROKEN
            | feat::LESS
            | feat::MORE
            | feat::PASS_RUBBLE
    )
}

/// The threat a grid carries, as extra steps to enter it: the M3 danger
/// placeholder ([`crate::danger`]), the C Borg's danger sum at one turn
/// (`borg-danger.c`). The C Borg instead *refuses* grids whose danger
/// passes a fear threshold (borg-flow.c:442-474); paying threat as a
/// cost keeps the route available at a price.
pub fn threat(w: &World, k: &Knowledge, x: i32, y: i32) -> u32 {
    crate::danger::danger(w, k, x, y, 1)
}

/// One goal's distance map over the level. Distances are the cost of the
/// cheapest path *from the goal grids to each grid* (the flood runs
/// outward from the goals, as the C Borg's does), so walking downhill
/// heads for the goal. `UNREACHED` marks what the flood never reached.
#[derive(Clone, Debug)]
pub struct DistanceMap {
    width: i32,
    height: i32,
    dist: Vec<u32>,
}

/// What no flood reached.
pub const UNREACHED: u32 = u32::MAX;

impl DistanceMap {
    /// Flood from the goal grids. Dijkstra over the eight neighbours,
    /// paying each entered grid's `enter_cost` plus its `threat`.
    fn build(w: &World, k: &Knowledge, targets: &[(i32, i32)], threat_on: bool) -> DistanceMap {
        let (width, height) = (w.level.width.max(0), w.level.height.max(0));
        let mut map = DistanceMap {
            width,
            height,
            dist: vec![UNREACHED; (width * height) as usize],
        };
        let mut queue: BinaryHeap<Reverse<(u32, i32, i32)>> = BinaryHeap::new();
        for &(x, y) in targets {
            if x < 0 || y < 0 || x >= width || y >= height {
                continue;
            }
            let base = standing_cost(w, k, x, y);
            let i = (y * width + x) as usize;
            if base < map.dist[i] {
                map.dist[i] = base;
                queue.push(Reverse((base, x, y)));
            }
        }
        while let Some(Reverse((d, x, y))) = queue.pop() {
            let i = (y * width + x) as usize;
            if map.dist[i] < d {
                continue; // a cheaper path was found meanwhile
            }
            for (_, dx, dy) in DIRS.iter() {
                let (nx, ny) = (x + dx, y + dy);
                let Some(cost) = enter_cost(w, nx, ny) else {
                    continue;
                };
                let step = cost + if threat_on { threat(w, k, nx, ny) } else { 0 };
                let next = d.saturating_add(step);
                let ni = (ny * width + nx) as usize;
                if next < map.dist[ni] {
                    map.dist[ni] = next;
                    queue.push(Reverse((next, nx, ny)));
                }
            }
        }
        // A shop is a goal but not a road. When the Borg stands on its
        // entrance after a purchase, the flood cannot enter its grid, yet
        // every destination is not thereby unreachable: the way out is the
        // cheapest reached neighbour plus the step out (T3.5).
        let (px, py) = player(w);
        if is_shop(w, px, py) && map.at(px, py) == UNREACHED {
            let best = DIRS
                .iter()
                .map(|(_, dx, dy)| map.at(px + dx, py + dy))
                .filter(|&d| d != UNREACHED)
                .min();
            if let Some(best) = best {
                map.dist[(py * width + px) as usize] = best.saturating_add(1);
            }
        }
        map
    }

    /// The distance of a grid, `UNREACHED` when the flood never reached
    /// it (or it lies outside the level).
    pub fn at(&self, x: i32, y: i32) -> u32 {
        if x < 0 || y < 0 || x >= self.width || y >= self.height {
            return UNREACHED;
        }
        self.dist[(y * self.width + x) as usize]
    }

    /// The cost still to pay, walking from `(x, y)` to the goal: the
    /// map's value there, minus the entry cost of the grid the walker
    /// already stands on. `UNREACHED` passes through.
    pub fn remaining(&self, w: &World, x: i32, y: i32) -> u32 {
        // A shop costs one to stand on just as a floor does, even though
        // transit may not enter it. Subtract that standing cost both when
        // it is the goal and when a purchase left the Borg there.
        let own = enter_cost(w, x, y)
            .or_else(|| is_shop(w, x, y).then_some(1))
            .unwrap_or(0);
        if self.at(x, y) == UNREACHED {
            return UNREACHED;
        }
        self.at(x, y).saturating_sub(own)
    }

    /// The direction whose grid is strictly cheaper than the one the Borg
    /// stands on, first in [`DIRS`] order on a tie — the greedy downhill
    /// step of the C Borg's `borg_flow_old` (borg-flow.c:1122-1203),
    /// without its random town choice, which would break determinism.
    /// `None` when the Borg is at the goal or nothing is downhill.
    pub fn step(&self, x: i32, y: i32) -> Option<u8> {
        let here = self.at(x, y);
        if here == UNREACHED {
            return None;
        }
        let mut best: Option<(u32, u8)> = None;
        for &(d, dx, dy) in DIRS.iter() {
            let there = self.at(x + dx, y + dy);
            let better = match best {
                None => true,
                Some((b, _)) => there < b,
            };
            if there < here && better {
                best = Some((there, d));
            }
        }
        best.map(|(_, d)| d)
    }
}

/// The per-think pathfinder: goal maps, computed on demand and cached for
/// the rest of the think. Built from one world snapshot; any other world
/// is refused. Nothing of it survives the think, so no map can be stale
/// (D0008, D0006).
#[derive(Debug)]
pub struct Paths {
    turn: i64,
    thinks: u64,
    maps: Vec<(Goal, DistanceMap)>,
}

impl Paths {
    /// For this world snapshot. Panics if used with any other: that would
    /// be a stale map (a defect, reported loudly, RULES 13).
    pub fn new(w: &World) -> Paths {
        Paths {
            turn: w.turn,
            thinks: w.thinks,
            maps: Vec::new(),
        }
    }

    /// The world must still be the snapshot this `Paths` was built for.
    fn check(&self, w: &World) {
        assert!(
            w.turn == self.turn && w.thinks == self.thinks,
            "paths: the world moved (turn {}, thinks {}) since this pathfinder was built \
             (turn {}, thinks {}): a cached map would be stale",
            w.turn,
            w.thinks,
            self.turn,
            self.thinks
        );
    }

    /// The distance map of a goal, computing it once per think.
    pub fn map(&mut self, w: &World, k: &Knowledge, goal: &Goal) -> &DistanceMap {
        self.check(w);
        let index = self.maps.iter().position(|(g, _)| g == goal);
        let i = match index {
            Some(i) => i,
            None => {
                let map = DistanceMap::build(w, k, &goal.targets(w), true);
                self.maps.push((goal.clone(), map));
                self.maps.len() - 1
            }
        };
        &self.maps[i].1
    }

    /// The cost from the Borg's grid to the nearest goal grid, paying
    /// every grid the walk enters: [`UNREACHED`] when no route exists.
    pub fn distance(&mut self, w: &World, k: &Knowledge, goal: &Goal) -> u32 {
        let (x, y) = player(w);
        self.map(w, k, goal).remaining(w, x, y)
    }

    /// The downhill step toward a goal, if one is strictly cheaper
    /// ([`DistanceMap::step`]).
    pub fn step(&mut self, w: &World, k: &Knowledge, goal: &Goal) -> Option<u8> {
        let (x, y) = player(w);
        self.map(w, k, goal).step(x, y)
    }
}

/// Where the Borg stands.
fn player(w: &World) -> (i32, i32) {
    w.character.status.as_ref().map_or((0, 0), |s| (s.x, s.y))
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::observation::{Grid, MonsterSeen, Observation, Panel, Status};

    /// Knowledge with the game's energy table around normal speed.
    pub(crate) fn energy() -> Knowledge {
        let mut e = vec![1u8; 200];
        for (s, v) in [(110, 10), (120, 20), (130, 30)] {
            e[s] = v;
        }
        Knowledge {
            extract_energy: e,
            move_energy: 100,
            ..Default::default()
        }
    }

    /// The energy table plus the test monster race (one 1d6 hurt blow),
    /// so the danger-based threat has a race to read.
    pub(crate) fn energy_with_race() -> Knowledge {
        let mut k = energy();
        let kobold = crate::knowledge::MonsterRace {
            index: 1,
            name: "test kobold".into(),
            sleep: 20,
            blows: vec![crate::knowledge::Blow {
                method: "CLAW".into(),
                effect: "HURT".into(),
                power: 60,
                dice: crate::knowledge::Dice {
                    base: 0,
                    dice: 1,
                    sides: 6,
                    m_bonus: 0,
                },
            }],
            ..Default::default()
        };
        k.races = vec![crate::knowledge::MonsterRace::default(), kobold];
        k
    }

    /// A world from an ASCII map: `#` wall, `.` floor, `<`/`>` stairs,
    /// `+` closed door, `,` rubble, `w` webbed floor, `^` known trap,
    /// `?` unknown (never seen), `s` the General Store, `a` the Alchemist,
    /// `@` the player, `M` an awake normal-speed monster, `F` a fast one.
    /// Everything but `?` is in view at one turn.
    pub(crate) fn world(map: &[&str]) -> (World, Knowledge) {
        let height = map.len() as i32;
        let width = map
            .iter()
            .map(|r| r.chars().count() as i32)
            .max()
            .unwrap_or(0);
        let mut grids = Vec::new();
        let (mut px, mut py) = (0, 0);
        let mut monsters = Vec::new();
        for (y, row) in map.iter().enumerate() {
            for (x, ch) in row
                .chars()
                .chain(core::iter::repeat(' '))
                .take(width as usize)
                .enumerate()
            {
                let (f, webbed, trap, shop) = match ch {
                    '#' => (21, false, false, -1),
                    '?' => (feat::NONE, false, false, -1),
                    '<' => (feat::LESS, false, false, -1),
                    '>' => (feat::MORE, false, false, -1),
                    '+' => (feat::CLOSED, false, false, -1),
                    ',' => (feat::PASS_RUBBLE, false, false, -1),
                    'w' => (feat::FLOOR, true, false, -1),
                    '^' => (feat::FLOOR, false, true, -1),
                    's' => (feat::STORE_GENERAL, false, false, 0),
                    'a' => (feat::STORE_ALCHEMY, false, false, 4),
                    _ => (feat::FLOOR, false, false, -1),
                };
                // '$' is a gold pile underfoot; 'x' a floor with nothing.
                let object = if ch == '$' { Some(500) } else { None };
                match ch {
                    '@' => {
                        px = x as i32;
                        py = y as i32;
                    }
                    'M' | 'F' => monsters.push(MonsterSeen {
                        index: monsters.len() as i32 + 1,
                        x: x as i32,
                        y: y as i32,
                        race: 1,
                        hp: 10,
                        max_hp: 10,
                        timed: [0; 10],
                        speed: if ch == 'F' { 120 } else { 110 },
                        attr: 0,
                        mimicking: false,
                    }),
                    _ => {}
                }
                grids.push(Grid {
                    feat: f,
                    in_view: ch != '?',
                    lighting: 1,
                    monster: ch == 'M' || ch == 'F',
                    object_kind: object,
                    multiple_objects: false,
                    unseen_object: false,
                    unseen_money: false,
                    trap,
                    shop,
                    webbed,
                });
            }
        }
        let o = Observation {
            turn: 100,
            status: Status {
                x: px,
                y: py,
                depth: 1,
                speed: 110,
                ..Status::default()
            },
            panel: Panel {
                x0: 0,
                y0: 0,
                width,
                height,
                level_width: width,
                level_height: height,
                grids,
            },
            monsters,
            ..Observation::default()
        };
        let mut w = World::new();
        w.update(&o);
        // A character with hit points: the danger's low-HP speed boost
        // (borg-danger.c:2357) must not move test speeds onto empty
        // table entries.
        if let Some(s) = w.character.status.as_mut() {
            s.max_hp = 20;
            s.hp = 20;
        }
        let has_monsters = o.monsters.iter().any(|m| m.race == 1);
        (
            w,
            if has_monsters {
                energy_with_race()
            } else {
                energy()
            },
        )
    }

    fn distance(map: &[&str], goal: &Goal) -> (World, Knowledge, u32) {
        let (w, k) = world(map);
        let d = Paths::new(&w).distance(&w, &k, goal);
        (w, k, d)
    }

    #[test]
    fn a_corridor_path_follows_the_floor() {
        let goal = Goal::Stairs { up: false };
        let (w, k, d) = distance(&["##########", "#@...>...#", "##########"], &goal);
        assert_eq!(d, 4, "four steps to the stair");
        // Walking the gradient: each step lands on a strictly cheaper
        // grid, and the walk reaches the stair.
        let mut p = Paths::new(&w);
        let (mut x, mut y) = (1_i32, 1_i32);
        let mut walked = 0;
        let map = p.map(&w, &k, &goal).clone();
        while let Some(dir) = map.step(x, y) {
            let (_, dx, dy) = DIRS.iter().find(|(d, _, _)| *d == dir).unwrap();
            assert!(map.at(x + dx, y + dy) < map.at(x, y), "strictly downhill");
            (x, y) = (x + dx, y + dy);
            walked += 1;
            assert!(walked < 20, "the walk terminates");
        }
        assert_eq!((x, y), (5, 1), "the walk reaches the stair");
    }

    #[test]
    fn doors_and_webs_cost_more_than_floor() {
        // Through the door: entering it costs 2 (walked open, then
        // entered), then the stair 1.
        // No side route: passing the door is open (2) then enter (1).
        let (w, k, door) = distance(&["######", "#@+>.#", "######"], &Goal::Stairs { up: false });
        assert_eq!(door, 3, "open (2) plus the stair (1)");
        // The same shape with a webbed grid: entering costs 3.
        let (_, _, webbed) = distance(&["######", "#@w>.#", "######"], &Goal::Stairs { up: false });
        assert_eq!(webbed, 4, "the web (3) plus the stair (1)");
        // Upstairs too.
        let (_, _, up) = distance(&["######", "#@.<.#", "######"], &Goal::Stairs { up: true });
        assert_eq!(up, 2);
        // The first step aims east, at the door.
        assert_eq!(
            Paths::new(&w).step(&w, &k, &Goal::Stairs { up: false }),
            Some(6)
        );
    }

    #[test]
    fn walls_and_unknown_do_not_pass() {
        // The stair walled off: unreached.
        let (w, _, d) = distance(&["#####", "#@#>#", "#####"], &Goal::Stairs { up: false });
        assert_eq!(d, UNREACHED);
        assert_eq!(
            Paths::new(&w).step(&w, &energy(), &Goal::Stairs { up: false }),
            None
        );
        // The only way to the stair is through never-seen terrain: the
        // flow never crosses the unknown.
        let (_, _, unknown) =
            distance(&["######", "#@? >#", "######"], &Goal::Stairs { up: false });
        assert_eq!(unknown, UNREACHED, "the flow never crosses the unknown");
    }

    #[test]
    fn the_frontier_is_the_known_edge_of_the_unknown() {
        let (w, k) = world(&["#########", "#@..??..#", "#########"]);
        let mut p = Paths::new(&w);
        let map = p.map(&w, &k, &Goal::Frontier).clone();
        assert_eq!(map.at(3, 1), 1, "standing on the frontier costs its step");
        assert_eq!(map.at(6, 1), 1, "on the frontier too");
        assert_eq!(
            map.remaining(&w, 1, 1),
            2,
            "two steps to the nearer frontier"
        );
        assert_eq!(map.at(4, 1), UNREACHED, "the unknown is never transit");
        // A closed door next to the unknown is frontier as well
        // (borg-flow-dark.c:61), and standing on it costs opening it.
        let (w, k) = world(&["########", "#@.?#+?#", "########"]);
        let mut p = Paths::new(&w);
        let map = p.map(&w, &k, &Goal::Frontier);
        assert_eq!(map.at(2, 1), 1, "the floor by the unknown");
        assert_eq!(map.at(5, 1), 2, "the door by the unknown");
    }

    #[test]
    fn the_frontier_underfoot_is_finished_not_a_place_to_hold() {
        let (w, k) = world(&["#####", "#@?.#", "#...#", "#####"]);
        let mut p = Paths::new(&w);
        assert_eq!(
            p.distance(&w, &k, &Goal::Frontier),
            1,
            "the next frontier, south"
        );
        assert_eq!(p.step(&w, &k, &Goal::Frontier), Some(2));
        let (w, k) = world(&["#####", "#@?##", "#####"]);
        let mut p = Paths::new(&w);
        assert_eq!(p.distance(&w, &k, &Goal::Frontier), UNREACHED);
        assert_eq!(p.step(&w, &k, &Goal::Frontier), None);
    }

    #[test]
    fn a_shop_is_a_goal_but_not_a_road() {
        let (w, k) = world(&["########", "#@...s.#", "########"]);
        let goal = Goal::Grids(vec![(5, 1)]);
        let mut p = Paths::new(&w);
        assert_eq!(p.distance(&w, &k, &goal), 4, "standing on the entrance");
        assert_eq!(p.step(&w, &k, &goal), Some(6));
        // The shop grid is not transited: the stair beyond it, in a
        // pocket closed by the shop, is unreached.
        let (_, _, behind) = distance(
            &["#######", "#@.s.>#", "#######"],
            &Goal::Stairs { up: false },
        );
        assert_eq!(behind, UNREACHED, "no route through the shop");
    }

    #[test]
    fn a_shop_entrance_can_be_left_after_shopping() {
        let (mut w, k) = world(&["#######", "#@.s.>#", "#######"]);
        w.character.status.as_mut().unwrap().x = 3;
        let goal = Goal::Stairs { up: false };
        let mut p = Paths::new(&w);
        assert_eq!(p.distance(&w, &k, &goal), 2, "two steps out and east");
        assert_eq!(p.step(&w, &k, &goal), Some(6));
    }

    #[test]
    fn threat_is_paid_as_a_cost_not_a_wall() {
        let goal = Goal::Stairs { up: false };
        let (_, _, plain) = distance(&["#######", "#@...>#", "#######"], &goal);
        let (_, _, with_m) = distance(&["#######", "#@.M.>#", "#######"], &goal);
        assert_eq!(plain, 4);
        assert!(with_m > plain, "the monster's grid is paid for: {with_m}");
        // With a detour two rows below the monster, going around is
        // cheaper: a normal-speed monster must be adjacent to strike.
        let (_, _, around) = distance(
            &["#######", "#@.M.>#", "#.....#", "#.....#", "#######"],
            &goal,
        );
        assert!(around < with_m, "around beats through: {around} < {with_m}");
        // A faster monster costs more than a normal one.
        let (_, _, with_f) = distance(&["#######", "#@.F.>#", "#######"], &goal);
        assert!(with_f > with_m, "speed weighs: {with_f} > {with_m}");
        // A sleeping light sleeper scores a little *more*, not less: the
        // C Borg adds the wake-up chance to the full melee term rather
        // than reducing it (borg-danger.c:2439-2450); only a character
        // of level 25 or more halves a sleeper first.
        let (w, k) = world(&["#######", "#@.M.>#", "#######"]);
        let mut asleep = w.clone();
        for t in &mut asleep.level.monsters {
            t.timed[0] = 100; // MON_TMD_SLEEP
        }
        let d = Paths::new(&asleep).distance(&asleep, &k, &goal);
        assert_eq!(
            d,
            with_m + 3,
            "each threatened grid pays sleep 20 + 5%: {d}"
        );
    }

    #[test]
    fn explicit_targets_flow_like_any_goal() {
        let (w, k) = world(&["########", "#@.....#", "########"]);
        let goal = Goal::Grids(vec![(6, 1)]);
        let mut p = Paths::new(&w);
        assert_eq!(p.distance(&w, &k, &goal), 5);
        assert_eq!(p.step(&w, &k, &goal), Some(6), "east");
        // An empty target set reaches nothing.
        assert_eq!(p.distance(&w, &k, &Goal::Grids(vec![])), UNREACHED);
    }

    #[test]
    fn maps_are_cached_within_a_think_and_refuse_a_moved_world() {
        let (mut w, k) = world(&["#####", "#@.>#", "#####"]);
        let mut p = Paths::new(&w);
        let goal = Goal::Stairs { up: false };
        let first = std::ptr::from_ref(p.map(&w, &k, &goal));
        assert_eq!(p.maps.len(), 1, "one map, not two");
        assert_eq!(std::ptr::from_ref(p.map(&w, &k, &goal)), first);
        // The world moves on: the same pathfinder must refuse it — the
        // stale-flow deaths (D0008, D0006) are impossible by construction.
        w.thinks += 1;
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            p.map(&w, &k, &goal);
        }));
        assert!(result.is_err(), "a moved world is refused");
    }

    /// Every think of a recorded run: each goal's map, where reachable,
    /// steps strictly downhill onto an enterable grid. CI records the
    /// observation and knowledge logs of one run and points the `_CHECK`
    /// variables at them (as the world replay test does); without them
    /// this test has nothing to read.
    #[test]
    fn recorded_walks_are_strictly_downhill() {
        let (Some(obs), Some(know)) = (
            std::env::var_os("BORG_OBSERVATION_LOG_CHECK"),
            std::env::var_os("BORG_KNOWLEDGE_LOG_CHECK"),
        ) else {
            return;
        };
        let know = std::fs::read_to_string(know).expect("the knowledge log");
        let k = Knowledge::from_line(know.lines().next().unwrap()).unwrap();
        let obs = std::fs::read_to_string(obs).expect("the observation log");
        let mut w = World::new();
        let mut checked = 0_u64;
        for (i, line) in obs.lines().enumerate() {
            let o = Observation::from_line(line).unwrap();
            w.update(&o);
            let mut p = Paths::new(&w);
            for goal in [
                Goal::Stairs { up: false },
                Goal::Stairs { up: true },
                Goal::Frontier,
            ] {
                if let Some(dir) = p.step(&w, &k, &goal) {
                    let s = w.character.status.as_ref().unwrap();
                    let (_, dx, dy) = DIRS.iter().find(|(d, _, _)| *d == dir).unwrap();
                    let (nx, ny) = (s.x + dx, s.y + dy);
                    let map = p.map(&w, &k, &goal);
                    assert!(
                        map.at(nx, ny) < map.at(s.x, s.y),
                        "think {}: the step is downhill",
                        i + 1
                    );
                    assert!(
                        enter_cost(&w, nx, ny).is_some(),
                        "think {}: the step enters a passable grid",
                        i + 1
                    );
                }
            }
            checked += 1;
        }
        assert!(checked > 0);
        eprintln!("{checked} thinks: every reachable goal steps downhill");
    }
}
