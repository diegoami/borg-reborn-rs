//! borg-reborn-rs (#882, owner 2026-11-22: "fight your way out"): against
//! slow or erratic breeders, hold, or cut the known way out; never retreat
//! into the colony.
//!
//! Dunadan Priest 399497898 (borg-cemetery study `worm-pocket-399497898`)
//! died in a checkerboard room of white worm masses whose only exit was one
//! corner. Its retreats, back-ups and stair-flight steps (214 moves in 359
//! player turns, 81 bites) walked it into fresh worms. On the exact game a
//! player who walked the known route to the exit, attacking the worm on that
//! route, escaped from every start up to turn 66262; a player who never moved
//! took 3 bites in 400 player turns.
//!
//! The rule, every number a hypothesis (RULES 14):
//! * *fires* when every awake monster within `caution_breeder_hold_scan_radius`
//!   is a breeder (`RF_MULTIPLY`) with no ranged attack or spell, none faster
//!   than the Borg and none as fast unless erratic (`RAND_25`/`RAND_50`), one
//!   of them closer than `caution_breeder_exit_distance`, and the colony is
//!   big (`caution_breeder_hold_count` known), growing (up by
//!   `caution_breeder_hold_growth` within `caution_breeder_hold_growth_turns`
//!   game turns) or has lost or scared the Borg off the level;
//! * *yields* to the other rules with a free staircase within
//!   `caution_breeder_easy_escape_steps`, or HP below
//!   `caution_breeder_hold_floor_hp_percent` with no cure potion;
//! * *cuts out* along the cheapest route over **known** grids (an unknown grid
//!   is a wall, #881) to a known staircase or a corridor or doorway at least
//!   `caution_breeder_exit_distance` from every known breeder, each breeder on
//!   it costing `caution_breeder_hold_worm_cost` extra steps, while at most
//!   `caution_breeder_hold_max_on_route` breeders are on it and the HP is at
//!   least `caution_breeder_hold_go_hp_percent` (or
//!   `caution_breeder_hold_cure_hp_percent` with a cure potion): it attacks
//!   the breeder on the route, else steps;
//! * otherwise *holds*: attacks an adjacent breeder, else stays still, for at
//!   most `caution_breeder_hold_idle_turns` idle thinks in a row.
//!
//! It sits in `borg_caution` after the heals, defence and the critical needs,
//! and before every movement rule (the flight to the stairs, the breeder and
//! crowd exits, retreat, the escapes, back-away), so while it fires none of
//! them moves the Borg. On an exit it walks on, over known grids, to the
//! nearest known staircase and takes it (`breeder_hold_stair`: the walk
//! owns the movement until then, #556; left to the ordinary flight, the
//! Borg stepped back toward the room every other think). With no staircase
//! route it holds the corridor. Engine basis
//! (574b618c): white worm mass speed 100, `RAND_25 | RAND_50`, `MULTIPLY`
//! (lib/gamedata/monster.txt:677-692); 75% of its moves stagger
//! (src/mon-move.c:1087-1102); no breeding with 4 or more neighbours
//! (src/mon-move.c:1043). Draws no random numbers.

use std::cmp::Reverse;
use std::collections::BinaryHeap;

use crate::borg_cave::{AUTO_MAX_X, AUTO_MAX_Y};
use crate::borg_cave_util::borg_cave_floor_grid;
use crate::borg_flow_kill::borg_race_name;
use crate::borg_flow_misc::borg_extract_dir;
use crate::borg_headers::borg_trait_h::{BI_ACCW, BI_ACLW, BI_ACSW, BI_CDEPTH, BI_CURHP, BI_MAXHP, BI_SPEED};
use crate::borg_io::{borg_keypress, borg_note};
use crate::engine;
use crate::headers::list_mon_race_flags_h::{RF_MULTIPLY, RF_RAND_25, RF_RAND_50};
use crate::headers::list_options_h::OPT_birth_force_descend;
use crate::headers::list_terrain_h::{FEAT_LESS, FEAT_MORE, FEAT_NONE};
use crate::headers::ui_event_h::keycode_t;
use crate::headers::z_type_h::loc;
use crate::params::{value as p, Param};
use crate::state::BorgState;

const NEIGHBOURS: [(i32, i32); 8] = [(-1, -1), (0, -1), (1, -1), (1, 0), (1, 1), (0, 1), (-1, 1), (-1, 0)];

/// A known monster, as the rule reads it.
#[derive(Clone, Copy, Debug)]
struct Near {
    awake: bool,
    breeder: bool,
    /// Ranged attacks or spells (`ranged_attack`).
    ranged: bool,
    /// `RAND_25` or `RAND_50`.
    erratic: bool,
    /// `extract_energy` of its speed.
    energy: i32,
}

/// Why the rule does not fire, or that it does.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Colony {
    /// Nothing for the rule: no breeder near, or the Borg already clear.
    None,
    /// Something near the rule does not cover (a non-breeder, a ranged or
    /// fast breeder): the other rules decide.
    Other,
    /// A slow colony, but neither big, growing nor lost.
    Small,
    /// The rule applies.
    Fires,
}

/// The firing test (`Colony`), with its inputs given. `known` is every known
/// monster within the scan radius (distance, the monster); `own` the Borg's
/// energy; `clear` the exit distance; `big` the colony test.
fn colony(known: &[(i32, Near)], own: i32, clear: i32, big: bool) -> Colony {
    let awake: Vec<&(i32, Near)> = known.iter().filter(|(_, m)| m.awake).collect();
    if awake.iter().any(|(_, m)| !m.breeder || m.ranged || m.energy > own || (m.energy == own && !m.erratic)) {
        return Colony::Other;
    }
    if !known.iter().any(|(d, m)| m.breeder && *d < clear) {
        return Colony::None;
    }
    if big {
        Colony::Fires
    } else {
        Colony::Small
    }
}

/// The colony is big, growing or has the level: `count` known breeders,
/// `growth` more than the fewest known within the window.
fn colony_big(count: i32, growth: i32, lost_or_scary: bool) -> bool {
    count >= p(Param::caution_breeder_hold_count) || growth >= p(Param::caution_breeder_hold_growth) || lost_or_scary
}

/// Keep the known breeder count of the last `window` game turns (one sample
/// per think while any is known) and return the rise over the fewest of the
/// earlier samples. First sight is not growth: with no earlier sample in the
/// window (none known before) the rise is 0.
fn breeder_growth(samples: &mut Vec<(i32, i32)>, now: i32, count: i32, window: i32) -> i32 {
    samples.retain(|&(t, _)| now >= t && now - t <= window);
    let low = samples.iter().map(|&(_, c)| c).min();
    if count > 0 {
        samples.push((now, count));
    }
    low.map_or(0, |low| (count - low).max(0))
}

/// The HP margin of the cut-out: `hp` of `max` at least `go` percent, or at
/// least `cure` percent with a cure potion.
fn may_cut_out(hp: i32, max: i32, cures: i32, go: i32, cure: i32) -> bool {
    let pct = hp * 100 / max.max(1);
    pct >= go || (cures > 0 && pct >= cure)
}

/// The cheapest route from `start` to a `target` grid over `passable` grids
/// (8 directions), each step costing 1 plus `worm` when a breeder stands on
/// the grid it enters. Returns the route (the first step first, the target
/// last) and the breeders on it; `None` when no target is reachable or the
/// Borg stands on one. Ties are broken by grid index, so the route is the
/// same on every run.
fn cut_route(
    w: i32,
    h: i32,
    start: loc,
    passable: &dyn Fn(i32, i32) -> bool,
    breeder: &dyn Fn(i32, i32) -> bool,
    target: &dyn Fn(i32, i32) -> bool,
    worm: i32,
) -> Option<(Vec<loc>, i32)> {
    if target(start.x, start.y) {
        return None;
    }
    let idx = |x: i32, y: i32| (y * w + x) as usize;
    let mut cost = vec![i32::MAX; (w * h) as usize];
    let mut from = vec![usize::MAX; (w * h) as usize];
    let mut heap = BinaryHeap::new();
    cost[idx(start.x, start.y)] = 0;
    heap.push(Reverse((0, idx(start.x, start.y))));
    while let Some(Reverse((c, i))) = heap.pop() {
        if c > cost[i] {
            continue;
        }
        let (x, y) = ((i as i32) % w, (i as i32) / w);
        if i != idx(start.x, start.y) && target(x, y) {
            let mut route = vec![];
            let mut at = i;
            while at != idx(start.x, start.y) {
                route.push(loc { x: (at as i32) % w, y: (at as i32) / w });
                at = from[at];
            }
            route.reverse();
            let on = route.iter().filter(|g| breeder(g.x, g.y)).count() as i32;
            return Some((route, on));
        }
        for (dx, dy) in NEIGHBOURS {
            let (nx, ny) = (x + dx, y + dy);
            if nx < 0 || ny < 0 || nx >= w || ny >= h || !passable(nx, ny) {
                continue;
            }
            let step = 1 + if breeder(nx, ny) { worm } else { 0 };
            let j = idx(nx, ny);
            if c + step < cost[j] {
                cost[j] = c + step;
                from[j] = i;
                heap.push(Reverse((c + step, j)));
            }
        }
    }
    None
}

/// An exit of the cut-out: a known staircase, or a corridor or doorway
/// (`room_exit_grid`: a checkerboard room's grids are not exits) at least
/// `clear` from every known breeder.
fn exit_grid(passable: &dyn Fn(i32, i32) -> bool, stair: bool, at: loc, breeders: &[loc], clear: i32) -> bool {
    passable(at.x, at.y) && (stair || crate::borg_caution::room_exit_grid(passable, at, breeders, clear))
}

/// `+` and a direction: attack the monster there.
fn attack(st: &mut BorgState, at: loc) {
    let dir = borg_extract_dir(st.borg.c.y, st.borg.c.x, at.y, at.x);
    borg_keypress(st, keycode_t::from(b'+'));
    borg_keypress(st, (dir + i32::from(b'0')) as keycode_t);
}

/// borg-reborn-rs (#882): the slow-breeder rule. True when it acted (an
/// attack, a step or a hold); false when the other rules decide.
pub(crate) fn breeder_hold(st: &mut BorgState) -> bool {
    if p(Param::caution_breeder_hold) == 0 || st.borg.tr(BI_CDEPTH) == 0 {
        return false;
    }
    let energy = |speed: i32| i32::from(engine::extract_energy(speed.clamp(0, 199) as usize));
    let own = energy(st.borg.tr(BI_SPEED));
    let scan = p(Param::caution_breeder_hold_scan_radius);
    let clear = p(Param::caution_breeder_exit_distance);
    let mut known = Vec::new();
    let mut breeders = Vec::new();
    for i in 1..st.kills.nxt.max(0) as usize {
        let kill = &st.kills.kills[i];
        if kill.r_idx == 0 || engine::r_max() == 0 || kill.r_idx as i32 >= engine::r_max() as i32 - 1 {
            continue;
        }
        let r = usize::from(kill.r_idx);
        let breeder = engine::race_flag_has(r, RF_MULTIPLY);
        if breeder {
            breeders.push(kill.pos);
        }
        let d = engine::distance(st.borg.c, kill.pos);
        if d > scan {
            continue;
        }
        known.push((
            d,
            Near {
                awake: kill.awake,
                breeder,
                ranged: kill.ranged_attack > 0,
                erratic: engine::race_flag_has(r, RF_RAND_25) || engine::race_flag_has(r, RF_RAND_50),
                energy: energy(i32::from(kill.speed)),
            },
        ));
    }
    let count = breeders.len() as i32;
    let growth = breeder_growth(
        &mut st.borg.breeder_hold_samples,
        engine::game_turn(),
        count,
        p(Param::caution_breeder_hold_growth_turns),
    );
    let big = colony_big(count, growth, st.borg.breeders_lost || st.borg.mon.scary);
    let state = colony(&known, own, clear, big);
    // Out of the colony and on the way to a staircase (`breeder_hold_stair`),
    // the walk goes on while only slow breeders are about, near or not.
    let walking = st.borg.breeder_hold_stair && state != Colony::Other;
    if state != Colony::Fires && !walking {
        st.borg.breeder_hold_idle = 0;
        st.borg.breeder_hold_stair = false;
        return false;
    }
    let hp = st.borg.tr(BI_CURHP);
    let max = st.borg.tr(BI_MAXHP);
    let cures = st.borg.tr(BI_ACLW) + st.borg.tr(BI_ACSW) + st.borg.tr(BI_ACCW);
    let steps = p(Param::caution_breeder_easy_escape_steps);
    let stair_near = [true, false].iter().any(|&up| {
        let d = crate::borg_caution::nearest_free_stair(st, up);
        d != -1 && d <= steps
    });
    let floor = hp * 100 < max * p(Param::caution_breeder_hold_floor_hp_percent) && cures == 0;
    if (stair_near && !walking) || floor {
        st.borg.breeder_hold_idle = 0;
        st.borg.breeder_hold_stair = false;
        return false;
    }

    // On a staircase at the end of the walk: take it.
    let here = u32::from(st.cave.grid(st.borg.c.x, st.borg.c.y).feat);
    if walking || state == Colony::Fires {
        let down = here == FEAT_MORE && !crate::borg_caution::recall_restricts(st);
        let up = here == FEAT_LESS && !engine::player_opt(OPT_birth_force_descend);
        if down || up {
            st.borg.breeder_hold_stair = false;
            st.borg.breeder_hold_idle = 0;
            borg_note(st, &format!("# Leaving the breeders by the stairs ({count} known)"));
            borg_keypress(st, keycode_t::from(if down { b'>' } else { b'<' }));
            return true;
        }
    }

    // The known map, read once.
    let (w, h) = (AUTO_MAX_X, AUTO_MAX_Y);
    let n = (w * h) as usize;
    let mut floor_at = vec![false; n];
    let mut stair_at = vec![false; n];
    let mut breeder_at = vec![false; n];
    let mut other_at = vec![false; n];
    for y in 0..h {
        for x in 0..w {
            let i = (y * w + x) as usize;
            let ag = st.cave.grid(x, y);
            let feat = u32::from(ag.feat);
            floor_at[i] = engine::square_in_bounds_fully(x, y) && borg_cave_floor_grid(ag) && feat != FEAT_NONE;
            stair_at[i] = feat == FEAT_LESS || feat == FEAT_MORE;
            if ag.kill != 0 {
                let r = usize::from(st.kills.kills[usize::from(ag.kill)].r_idx);
                if r != 0 && engine::race_flag_has(r, RF_MULTIPLY) {
                    breeder_at[i] = true;
                } else {
                    other_at[i] = true;
                }
            }
        }
    }
    let inside = |x: i32, y: i32| x >= 0 && y >= 0 && x < w && y < h;
    let at = |v: &[bool], x: i32, y: i32| inside(x, y) && v[(y * w + x) as usize];
    let passable = |x: i32, y: i32| at(&floor_at, x, y) && !at(&other_at, x, y);
    let breeder = |x: i32, y: i32| at(&breeder_at, x, y);
    let exit = |x: i32, y: i32| {
        !at(&breeder_at, x, y) && exit_grid(&passable, at(&stair_at, x, y), loc { x, y }, &breeders, clear)
    };
    let stair = |x: i32, y: i32| !at(&breeder_at, x, y) && at(&stair_at, x, y) && passable(x, y);
    let worm = p(Param::caution_breeder_hold_worm_cost);
    // In the colony, the route is to the nearest exit. On an exit, or out
    // and walking, it is on to the nearest known staircase, the exit owning
    // the movement until the stairs are taken (#556): left to the ordinary
    // flight, the Borg stepped back toward the room each other think.
    let released = st.borg.breeder_hold_released;
    let to_stair = walking || released || exit(st.borg.c.x, st.borg.c.y);
    let route = if to_stair {
        cut_route(w, h, st.borg.c, &passable, &breeder, &stair, worm)
    } else {
        cut_route(w, h, st.borg.c, &passable, &breeder, &exit, worm)
    };
    if to_stair && route.is_none() {
        st.borg.breeder_hold_stair = false;
        // Out on the exit with no known staircase: let the colony go. Left
        // to pull the Borg back to the exit, the rule undid every step of
        // the ordinary search for stairs (Priest 399497898 at track-p
        // 21c0d9c1: about 200 shuttles over 20,000 game turns).
        if !released && exit(st.borg.c.x, st.borg.c.y) {
            st.borg.breeder_hold_released = true;
            borg_note(st, &format!("# Letting the breeders go: no known staircase from the exit ({count} known)"));
        }
        if state != Colony::Fires {
            return false;
        }
    }
    let released = st.borg.breeder_hold_released;

    if let Some((route, on)) = route {
        let go = may_cut_out(
            hp,
            max,
            cures,
            p(Param::caution_breeder_hold_go_hp_percent),
            p(Param::caution_breeder_hold_cure_hp_percent),
        );
        if on <= p(Param::caution_breeder_hold_max_on_route) && go {
            st.borg.breeder_hold_idle = 0;
            // The step onto the exit starts the walk on: there the Borg is
            // clear of the breeders, and the rule would no longer fire.
            st.borg.breeder_hold_stair = to_stair || route.len() == 1;
            st.borg.goal.type_0 = 0;
            let next = route[0];
            let end = route[route.len() - 1];
            let what = if to_stair { "the stairs at" } else { "the exit at" };
            if breeder(next.x, next.y) {
                let kill = st.kills.kills[usize::from(st.cave.grid(next.x, next.y).kill)];
                let name = borg_race_name(i32::from(kill.r_idx)).to_string_lossy().into_owned();
                borg_note(
                    st,
                    &format!(
                        "# Cutting out through the breeders: attacking {name} on the route to {what} ({},{}) ({} steps, {on} breeders on it, {count} known)",
                        end.y,
                        end.x,
                        route.len()
                    ),
                );
                attack(st, next);
            } else {
                borg_note(
                    st,
                    &format!(
                        "# Cutting out through the breeders to {what} ({},{}) ({} steps, {on} breeders on it, {count} known)",
                        end.y,
                        end.x,
                        route.len()
                    ),
                );
                let dir = borg_extract_dir(st.borg.c.y, st.borg.c.x, next.y, next.x);
                borg_keypress(st, (dir + i32::from(b'0')) as keycode_t);
            }
            return true;
        }
        if to_stair && state != Colony::Fires {
            // Out of the colony, but the stairs' route is too costly now.
            st.borg.breeder_hold_stair = false;
            return false;
        }
    }

    // Hold: the weakest adjacent breeder (fewest known HP, then the first
    // in the monster list), else stay still.
    let mut best: Option<(i16, usize)> = None;
    for i in 1..st.kills.nxt.max(0) as usize {
        let kill = &st.kills.kills[i];
        if kill.r_idx == 0 || !engine::race_flag_has(usize::from(kill.r_idx), RF_MULTIPLY) {
            continue;
        }
        if (kill.pos.x - st.borg.c.x).abs() > 1 || (kill.pos.y - st.borg.c.y).abs() > 1 {
            continue;
        }
        if best.is_none_or(|(power, _)| kill.power < power) {
            best = Some((kill.power, i));
        }
    }
    if let Some((_, i)) = best {
        st.borg.breeder_hold_idle = 0;
        let kill = st.kills.kills[i];
        let name = borg_race_name(i32::from(kill.r_idx)).to_string_lossy().into_owned();
        borg_note(st, &format!("# Holding against the breeders: attacking {name} ({count} known, HP {hp}/{max})"));
        attack(st, kill.pos);
        return true;
    }
    if released || st.borg.breeder_hold_idle >= p(Param::caution_breeder_hold_idle_turns) {
        st.borg.breeder_hold_idle = 0;
        return false;
    }
    st.borg.breeder_hold_idle += 1;
    borg_note(st, &format!("# Holding against the breeders: staying still ({count} known, HP {hp}/{max})"));
    borg_keypress(st, keycode_t::from(b','));
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    fn worm(d: i32) -> (i32, Near) {
        (d, Near { awake: true, breeder: true, ranged: false, erratic: true, energy: 5 })
    }

    #[test]
    fn fires_only_on_a_big_slow_colony_near() {
        assert_eq!(colony(&[worm(1), worm(3)], 10, 6, true), Colony::Fires);
        assert_eq!(colony(&[worm(1), worm(3)], 10, 6, false), Colony::Small);
        // Clear of every breeder: the flows take over.
        assert_eq!(colony(&[worm(7)], 10, 6, true), Colony::None);
        // A non-breeder awake within the scan: the other rules.
        let mut orc = worm(5);
        orc.1.breeder = false;
        assert_eq!(colony(&[worm(1), orc], 10, 6, true), Colony::Other);
        // A sleeping non-breeder does not count.
        orc.1.awake = false;
        assert_eq!(colony(&[worm(1), orc], 10, 6, true), Colony::Fires);
        // A ranged breeder, a faster one, an equally fast steady one.
        let mut b = worm(2);
        b.1.ranged = true;
        assert_eq!(colony(&[b], 10, 6, true), Colony::Other);
        let mut b = worm(2);
        b.1.energy = 20;
        assert_eq!(colony(&[b], 10, 6, true), Colony::Other);
        let mut b = worm(2);
        b.1.energy = 10;
        assert_eq!(colony(&[b], 10, 6, true), Colony::Fires);
        b.1.erratic = false;
        assert_eq!(colony(&[b], 10, 6, true), Colony::Other);
    }

    #[test]
    fn growth_is_over_the_window() {
        let mut s = vec![];
        // None known, then two seen at once: first sight, not growth.
        assert_eq!(breeder_growth(&mut s, 990, 0, 100), 0);
        assert_eq!(breeder_growth(&mut s, 1000, 2, 100), 0);
        assert_eq!(breeder_growth(&mut s, 1010, 3, 100), 1);
        assert_eq!(breeder_growth(&mut s, 1050, 5, 100), 3);
        // The samples of turns 1000 and 1010 leave the window.
        assert_eq!(breeder_growth(&mut s, 1120, 6, 100), 1);
        // Fewer known (out of view) is no growth.
        assert_eq!(breeder_growth(&mut s, 1130, 2, 100), 0);
    }

    #[test]
    fn hp_margin() {
        assert!(may_cut_out(24, 59, 0, 40, 25));
        assert!(!may_cut_out(23, 59, 0, 40, 25));
        assert!(may_cut_out(15, 59, 1, 40, 25));
        assert!(!may_cut_out(14, 59, 1, 40, 25));
    }

    /// A checkerboard room (floor where x + y is even, inside x 3..=11,
    /// y 1..=9) whose only way out is a corridor west of its south-west
    /// corner (x 0..=2, y 9); worms at the given grids.
    fn lattice(worms: &[(i32, i32)]) -> (Vec<bool>, Vec<bool>) {
        let (w, h) = (14, 12);
        let mut floor = vec![false; (w * h) as usize];
        let mut b = vec![false; (w * h) as usize];
        for y in 1..=9 {
            for x in 3..=11 {
                if (x + y) % 2 == 0 {
                    floor[(y * w + x) as usize] = true;
                }
            }
        }
        for x in 0..=2 {
            floor[(9 * w + x) as usize] = true;
        }
        for &(x, y) in worms {
            b[(y * w + x) as usize] = true;
        }
        (floor, b)
    }

    #[test]
    fn the_route_is_the_known_way_out_through_the_worm_on_it() {
        let (w, h) = (14, 12);
        let corridor = |g: &loc| g.x <= 2 && g.y == 9;
        // A worm far from the way out: the route does not cross it.
        let (floor, b) = lattice(&[(10, 2)]);
        let pass = |x: i32, y: i32| x >= 0 && y >= 0 && x < w && y < h && floor[(y * w + x) as usize];
        let br = |x: i32, y: i32| x >= 0 && y >= 0 && x < w && y < h && b[(y * w + x) as usize];
        let breeders = [loc { x: 10, y: 2 }];
        let target = |x: i32, y: i32| !br(x, y) && exit_grid(&pass, false, loc { x, y }, &breeders, 6);
        // The room's grids touch only diagonally: none of them is an exit,
        // even the dead-end tips; the doorway (3,9) and the corridor are.
        assert!(!target(5, 5) && !target(11, 1) && !target(3, 1) && target(3, 9) && target(1, 9));
        let (route, on) = cut_route(w, h, loc { x: 7, y: 5 }, &pass, &br, &target, 3).unwrap();
        assert_eq!(on, 0);
        assert_eq!(route.last().map(|g| (g.x, g.y)), Some((3, 9)));
        assert!(route.iter().all(|g| pass(g.x, g.y)));
        // The doorway's only room neighbour holds a worm: the route goes
        // through it (to attack it), out to the corridor.
        let (floor2, b2) = lattice(&[(4, 8)]);
        let pass2 = |x: i32, y: i32| x >= 0 && y >= 0 && x < w && y < h && floor2[(y * w + x) as usize];
        let br2 = |x: i32, y: i32| x >= 0 && y >= 0 && x < w && y < h && b2[(y * w + x) as usize];
        let worms = [loc { x: 4, y: 8 }];
        let target2 = |x: i32, y: i32| !br2(x, y) && exit_grid(&pass2, false, loc { x, y }, &worms, 3);
        let (route, on) = cut_route(w, h, loc { x: 7, y: 5 }, &pass2, &br2, &target2, 3).unwrap();
        assert_eq!(on, 1);
        assert!(route.iter().any(|g| (g.x, g.y) == (4, 8)));
        assert!(route.last().is_some_and(corridor));
        // On the target already: no route.
        assert!(cut_route(w, h, loc { x: 1, y: 9 }, &pass2, &br2, &target2, 3).is_none());
    }

    #[test]
    fn unknown_grids_are_walls() {
        let (w, h) = (14, 12);
        let (floor, b) = lattice(&[]);
        // No corridor known: no exit, no route through the unknown.
        let pass = |x: i32, y: i32| x >= 3 && y >= 0 && x < w && y < h && floor[(y * w + x) as usize];
        let br = |x: i32, y: i32| x >= 0 && y >= 0 && x < w && y < h && b[(y * w + x) as usize];
        let target = |x: i32, y: i32| exit_grid(&pass, false, loc { x, y }, &[], 2);
        assert!(cut_route(w, h, loc { x: 7, y: 5 }, &pass, &br, &target, 3).is_none());
    }
}
