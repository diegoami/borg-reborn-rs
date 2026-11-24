//! borg-reborn-rs (#1061, owner proposal from the level-1 death review of
//! track-p `206cf4e7`): `stair_scum`, an exploration mode for a character
//! with few hit points.
//!
//! > "they explore the level in a way that they can go back to the stairs
//! > faster than any character can come at them, that is sometimes just two
//! > or three steps from the stair, open the doors to check what is there,
//! > or at least explore radially, without committing to a direction"
//!
//! The mode applies in the dungeon while `stair_scum` is 1, the maximum HP
//! is below the threshold (`stair_scum_max_hp`, per class, plus
//! `stair_scum_max_hp_race_delta`, per race) and the Borg does not detect
//! monsters the way its exploration does (review of #1090, P1): a Rod of
//! Detection carried or the Detection spell (zapped and cast by the panel
//! checks of `borg_light.rs` with no level gate), or Detect Monsters from
//! `light_detect_evil_spell_min_level` on, each at a fail rate of 40% or
//! less (the panel checks' `borg_spell_fail(.., 40)`). The dark-entry cast
//! (#796, one attempt per panel), Reveal Monsters, Detect Invisible and
//! Detect Life do not count.
//!
//! 1. **Safe grids** (`Map`). *s(g)* is the walk from *g* to the nearest
//!    known staircase the Borg will take (up unless descent is forced; down
//!    when prepared for the next depth and not rising: the same rule for the
//!    area, the walk back, the level change and the departure on a threat;
//!    review of #1090, P1) over known, trap-free floor, no closed door (the
//!    walk a retreat can take: below 60 HP the flows treat a known trap as a
//!    wall, `borg_flow.rs`, and a closed door costs the open and may stay
//!    shut; review of #1090, P1). A staircase a monster stands on is no
//!    escape, and a grid is also unsafe when a monster from the frontier (an
//!    unknown grid or a closed door next to the walk) nearest its staircase
//!    gets there before the Borg (`cut_off`; re-review of #1090, P1). *g* is safe when a monster first seen
//!    *D(g)* grids off, acting `stair_scum_speed_ratio` times per player
//!    action, lands fewer than HP / `stair_scum_contact_damage` attacks on
//!    the walk of *s(g)* steps and the stair command (`attacks_on_retreat`;
//!    review of #1061, P1: the attacks on the way are counted, not a
//!    formula). *D(g)* follows the engine's light (review, P1): the own
//!    light lights `cur_light - 1` grids (`cave-view.c:658`), unlight
//!    `1 + level / 6 - cur_light` (`cave-view.c:777-779`); on a lit room
//!    floor it is one less than the distance to the nearest grid a monster
//!    can come from unseen (an unknown or dark grid, or a door, next to the
//!    lit floor), when that is longer; behind a closed door it is 1, and the
//!    door costs one action more (the open). A torch in a dark corridor at
//!    12 HP against the +10 monsters of DL1-2 (`monster.txt` Grip, Fang,
//!    crow, wild cat `speed:120`, 4 HP per round of blows): 1 step.
//! 2. **The safe area** is every safe grid joined to a staircase through
//!    safe grids. The dark flows keep only candidates in it (or next to it),
//!    nearest the staircase first, and when the nearest lies farther out than
//!    the Borg, it goes back onto the staircase first (no chaining outward
//!    along one corridor; review of #1090, P2); items and monster hunts outside it are
//!    skipped; every voluntary flow step (`borg_flow_old`, a resumed flow
//!    too) that leaves it and does not come nearer the staircase is refused
//!    (review, P1); a Borg outside it walks back onto a staircase before
//!    anything but caution.
//! 3. **Doors.** A door the Borg opens is not walked through at once: the
//!    Borg steps back toward the staircase (or stays, on it) and waits a
//!    turn. A known closed door seen open that the Borg did not open sends
//!    it onto the staircase for `stair_scum_alarm_turns` game turns when it
//!    is inside the safe area's reach. "You hear a door burst open!" names
//!    no grid (`mon-move.c:1269`): when no changed door explains it by the
//!    next think, the Borg goes onto the staircase all the same (review, P2).
//! 4. **Stair scum.** Nothing left inside the area: it takes a staircase,
//!    up or down within the depth plan, preferring DL1-2.
//! 5. **Fight from the staircase.** On a staircase, a threat it does not win
//!    (`winning_fight` acts before caution) makes it take the stairs
//!    (`borg_caution::leave_stairs_on_monster`); it rests on the staircase.
//! 6. At max HP >= the threshold the mode ends; C's leash is always kept.
//!
//! **No route** (review, P1): with no staircase known, or none the Borg can
//! walk to over known trap-free grids, the dark flows keep the candidates
//! nearest a known staircase (all of them when none is known), and no item
//! or monster is chased until a route exists again (#1062, #1067 own the
//! rest). Every value is a hypothesis until borg-zoo measures it (RULES
//! 14); `stair_scum` ships 0, so play is unchanged.

use crate::borg::BorgTime;
use crate::borg_cave::{AUTO_MAX_X, AUTO_MAX_Y};
use crate::borg_cave_util::borg_cave_floor_grid;
use crate::borg_headers::borg_magic_h::{DETECTION, DETECT_MONSTERS};
use crate::borg_headers::borg_trait_h::{BI_CDEPTH, BI_CLASS, BI_CLEVEL, BI_CURHP, BI_LIGHT, BI_MAXHP};
use crate::borg_io::{borg_keypress, borg_note};
use crate::engine;
use crate::headers::list_options_h::OPT_birth_force_descend;
use crate::headers::list_terrain_h::{FEAT_BROKEN, FEAT_CLOSED, FEAT_LAVA, FEAT_LESS, FEAT_MORE, FEAT_NONE, FEAT_OPEN};
use crate::headers::list_tvals_h::TV_ROD;
use crate::headers::ui_event_h::keycode_t;
use crate::params::{value, Param};
use crate::state::BorgState;

/// `BORG_GLOW` (borg-cave.h, `borg_update.rs`): the grid is probably
/// perma-lit.
const BORG_GLOW: u16 = 0x02;

/// `GOAL_MISC` (borg-flow.h): the walk back to the staircase.
const GOAL_MISC: i32 = 3;
/// `GOAL_BORE` (borg-flow.h): the walk to the staircase it leaves by.
const GOAL_BORE: i32 = 6;
/// `GOAL_FLEE` (borg-flow.h): flights are never refused a step.
const GOAL_FLEE: i32 = 7;

/// `class.txt`'s Necromancer (`cidx` 4), who sees by unlight.
const CLASS_NECROMANCER: i32 = 4;

/// No known route.
const FAR: u16 = u16::MAX;

/// The longest walk the safe-grid test counts (the flows' 250).
const MAX_STEPS: i32 = 250;

/// The mode's memory. `level` is the level's arrival time.
#[derive(Default)]
pub struct StairScumState {
    level: BorgTime,
    /// Walking back onto the staircase (until it stands on one).
    returning: bool,
    /// A door the Borg opened and has not peeked through yet.
    peek: Option<Peek>,
    /// Game turn of the last door heard or seen opened within reach.
    alarm: Option<i32>,
    /// Game turn of a "door burst open" not yet placed on a grid.
    heard: Option<i32>,
    /// Game turn the last changed door was seen (inside reach or not).
    changed_door_seen: Option<i32>,
    /// Doors already peeked at on this level, `(y, x)`.
    peeked: Vec<(i32, i32)>,
    /// Doors a monster opened that were already judged, `(y, x)`
    /// (`track_closed` keeps them until the Borg opens a door itself).
    judged: Vec<(i32, i32)>,
    /// The note for "no route" was written on this level.
    noted_no_route: bool,
    /// `leave` chose to leave this level: no more detours for items,
    /// veins or monsters (they pulled the walk back, a vein by the stairs).
    departing: bool,
    /// The last voluntary DL1 ascent, kept across immediate re-entry.
    last_town_ascent: Option<i32>,
}

#[derive(Clone, Copy)]
struct Peek {
    door: (i32, i32),
    stage: PeekStage,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum PeekStage {
    /// The door was opened (or the open command sent): step back next.
    Opened,
    /// Stepped back (or stayed, on the staircase): wait a turn next.
    Back,
}

/// The mode's switch.
fn flag() -> bool {
    value(Param::stair_scum) != 0
}

/// The max HP at which the mode ends: the class's value plus the race's
/// delta (owner, 2026-11-22: "25 is a number that you could parametrize,
/// probably by class too and maybe by race. race would be a delta").
fn max_hp_threshold() -> i32 {
    value(Param::stair_scum_max_hp) + value(Param::stair_scum_max_hp_race_delta)
}

/// Below the threshold (pure).
fn below_threshold(max_hp: i32, threshold: i32) -> bool {
    max_hp < threshold
}

/// The Borg finds monsters before they find it, with the detection its
/// exploration actually uses (review of #1090, P1): `borg_light.rs`'s panel
/// checks zap a Rod of Detection and cast Detection with no level gate, and
/// cast Detect Monsters only from `light_detect_evil_spell_min_level` (the
/// Mage's 10 in the shipped `borg.txt`) or on a dark entry with
/// `light_detect_dark_entry`. Reveal Monsters is cast only against an
/// invisible attacker (`borg_fight_defend.rs`), so it does not count.
fn detects_monsters(st: &BorgState) -> bool {
    // Re-review of #1090 (P2): the panel checks cast with `borg_spell_fail(..,
    // 40)`, so a spell counts only when it is legal at a fail rate of 40% or
    // less (`borg_spell_legal_fail`). The dark-entry cast (#796) is a single
    // attempt per panel behind a mana reserve, not a standing detection: it
    // does not end the mode.
    detection_used(
        crate::borg_magic::borg_spell_legal_fail(st, DETECT_MONSTERS, 40),
        st.borg.tr(BI_CLEVEL),
        value(Param::light_detect_evil_spell_min_level),
        crate::borg_magic::borg_spell_legal_fail(st, DETECTION, 40),
        carries(st, TV_ROD as i32, st.item_val.sv_rod_detection),
    )
}

/// The detection exploration will use (pure).
fn detection_used(detect_monsters: bool, clevel: i32, spell_min_level: i32, detection: bool, rod: bool) -> bool {
    (detect_monsters && clevel >= spell_min_level) || detection || rod
}

/// An aware item of this kind in the pack (`borg_slot` >= 0, read only).
fn carries(st: &BorgState, tval: i32, sval: i32) -> bool {
    let pack = usize::from(engine::z_info().pack_size);
    st.items.items.iter().take(pack).any(|item| {
        item.iqty != 0 && item.aware && i32::from(item.tval) == tval && i32::from(item.sval) == sval
    })
}

/// Whether the mode applies now (the switch first: with it off, nothing
/// else is read).
pub(crate) fn active(st: &BorgState) -> bool {
    flag()
        && st.borg.tr(BI_CDEPTH) > 0
        && below_threshold(st.borg.tr(BI_MAXHP), max_hp_threshold())
        && !detects_monsters(st)
}

/// The grids the own light shows, by the engine's rule (pure): a light of
/// `cur_light` lights `cur_light - 1` grids (`cave-view.c:658`, `radius =
/// ABS(light) - 1`), whatever `cave_light_engine_radius` says; a
/// Necromancer's unlight at `cur_light <= 1` sees `d < 2 + level / 6 -
/// cur_light` (`cave-view.c:777-779`).
fn engine_sight(light: i32, class: i32, level: i32) -> i32 {
    if class == CLASS_NECROMANCER && light <= 1 {
        (1 + level / 6 - light).max(0)
    } else {
        (light - 1).max(0)
    }
}

/// The attacks a monster first seen `seen_at` grids off lands while the
/// Borg walks `steps` steps to a staircase and takes it (pure; review of
/// #1061, P1). The monster acts `ratio` times per player action and may act
/// `ratio - 1` times before the Borg's first (`game-world.c:1033-1067`:
/// monsters with more energy than the player act before it, the rest after
/// it). Each action closes one grid, or attacks when adjacent; each step
/// opens one grid (the monster follows); after the stair command nothing
/// acts (`cmd-cave.c:56-95`, the level is generated).
fn attacks_on_retreat(steps: i32, seen_at: i32, ratio: i32) -> i32 {
    let mut gap = seen_at.max(1);
    let mut hits = 0;
    let mut act = |gap: &mut i32| {
        if *gap > 1 {
            *gap -= 1;
        } else {
            hits += 1;
        }
    };
    for _ in 0..ratio.max(1) - 1 {
        act(&mut gap);
    }
    for _ in 0..steps.max(0) {
        gap += 1;
        for _ in 0..ratio.max(1) {
            act(&mut gap);
        }
    }
    hits
}

/// The Borg survives those attacks: fewer than HP / damage per attack
/// round (pure).
fn survives(steps: i32, seen_at: i32, ratio: i32, hp: i32, per_attack: i32) -> bool {
    i64::from(attacks_on_retreat(steps, seen_at, ratio)) * i64::from(per_attack.max(1)) < i64::from(hp)
}

/// The walk the survival test counts, `slack` steps shorter than it is
/// (pure): with slack, a grid that many steps beyond the strict radius is
/// still safe.
fn slackened(steps: i32, slack: i32) -> i32 {
    (steps - slack.max(0)).max(0)
}

/// The slack in steps (pure; owner, 2026-11-23: "a rate to make the scum
/// dependent on hp/class", then "slack grows by decile by level, say 50
/// means that druids at level 3 get to slack 2"): the fixed steps, plus
/// `per_hp_tenths` tenths of a step per point of max HP, plus
/// `per_level_percent` percent of a step per character level above 1, each
/// rounded down, so the mode loosens as the character grows. All are per
/// class (`__<class>`) and per race (`__<race>`).
fn slack_steps(steps: i32, per_hp_tenths: i32, max_hp: i32, per_level_percent: i32, clevel: i32) -> i32 {
    let by_hp = i64::from(max_hp.max(0)) * i64::from(per_hp_tenths.max(0)) / 10;
    let by_level = i64::from((clevel - 1).max(0)) * i64::from(per_level_percent.max(0)) / 100;
    (i64::from(steps.max(0)) + by_hp + by_level).min(i64::from(MAX_STEPS)) as i32
}

/// A retreat of `walk` actions is safe (pure): the slack shortens the walk
/// the damage budget counts (the owner's dial), but the cut-off is timed on
/// the real walk (re-review of #1090 at `9276b75a`, P1: with a Mage's slack 2
/// a two-step walk counted as none, and a monster one step from the
/// staircase got there first).
fn retreat_ok(walk: i32, slack: i32, sight: i32, ratio: i32, hp: i32, per_attack: i32, front: i32) -> bool {
    survives(slackened(walk, slack), sight, ratio, hp, per_attack) && !cut_off(walk, front, ratio)
}

/// The longest walk that survives (pure; for the notes and the tests).
fn radius(seen_at: i32, ratio: i32, hp: i32, per_attack: i32) -> i32 {
    let mut r = 0;
    while r < MAX_STEPS && survives(r + 1, seen_at, ratio, hp, per_attack) {
        r += 1;
    }
    r
}

/// The eight directions, `(dy, dx)`.
const DIRS: [(i32, i32); 8] = [(-1, -1), (-1, 0), (-1, 1), (0, -1), (0, 1), (1, -1), (1, 0), (1, 1)];

fn index(y: i32, x: i32) -> usize {
    (y * AUTO_MAX_X + x) as usize
}

fn inside(y: i32, x: i32) -> bool {
    (1..AUTO_MAX_Y - 1).contains(&y) && (1..AUTO_MAX_X - 1).contains(&x)
}

/// Steps from every grid to the nearest of `sources` over grids `passable`
/// says can be walked (pure; 8-connected, one step each; `FAR` where no
/// route is known). With every grid passable this is the Chebyshev
/// distance.
fn distances(sources: &[(i32, i32)], passable: impl Fn(i32, i32) -> bool) -> Vec<u16> {
    let mut dist = vec![FAR; (AUTO_MAX_X * AUTO_MAX_Y) as usize];
    let mut queue = std::collections::VecDeque::new();
    for &(y, x) in sources {
        if inside(y, x) && dist[index(y, x)] == FAR {
            dist[index(y, x)] = 0;
            queue.push_back((y, x));
        }
    }
    while let Some((y, x)) = queue.pop_front() {
        let next = dist[index(y, x)] + 1;
        for (dy, dx) in DIRS {
            let (ny, nx) = (y + dy, x + dx);
            if !inside(ny, nx) || dist[index(ny, nx)] != FAR || !passable(ny, nx) {
                continue;
            }
            dist[index(ny, nx)] = next;
            queue.push_back((ny, nx));
        }
    }
    dist
}

/// `distances`, with the index in `sources` each grid's walk leads to
/// (`FAR` where none).
fn distances_from(sources: &[(i32, i32)], passable: impl Fn(i32, i32) -> bool) -> (Vec<u16>, Vec<u16>) {
    let mut dist = vec![FAR; (AUTO_MAX_X * AUTO_MAX_Y) as usize];
    let mut from = vec![FAR; (AUTO_MAX_X * AUTO_MAX_Y) as usize];
    let mut queue = std::collections::VecDeque::new();
    for (k, &(y, x)) in sources.iter().enumerate() {
        if inside(y, x) && dist[index(y, x)] == FAR {
            dist[index(y, x)] = 0;
            from[index(y, x)] = k as u16;
            queue.push_back((y, x));
        }
    }
    while let Some((y, x)) = queue.pop_front() {
        let next = dist[index(y, x)] + 1;
        for (dy, dx) in DIRS {
            let (ny, nx) = (y + dy, x + dx);
            if !inside(ny, nx) || dist[index(ny, nx)] != FAR || !passable(ny, nx) {
                continue;
            }
            dist[index(ny, nx)] = next;
            from[index(ny, nx)] = from[index(y, x)];
            queue.push_back((ny, nx));
        }
    }
    (dist, from)
}

/// The grids where a monster can come at a lit floor unseen (pure): not lit
/// floor, not a known wall, next to lit floor.
fn entries(lit_floor: impl Fn(i32, i32) -> bool, wall: impl Fn(i32, i32) -> bool) -> Vec<(i32, i32)> {
    let mut out = Vec::new();
    for y in 1..AUTO_MAX_Y - 1 {
        for x in 1..AUTO_MAX_X - 1 {
            if lit_floor(y, x) || wall(y, x) {
                continue;
            }
            if DIRS.iter().any(|&(dy, dx)| lit_floor(y + dy, x + dx)) {
                out.push((y, x));
            }
        }
    }
    out
}

/// The connected safe area (pure): the grids reachable from the staircases
/// (`dist` 0) through walkable grids for which `safe` holds.
fn safe_area(dist: &[u16], walkable: impl Fn(i32, i32) -> bool, safe: impl Fn(i32, i32) -> bool) -> Vec<bool> {
    let mut area = vec![false; dist.len()];
    let mut queue = std::collections::VecDeque::new();
    for y in 1..AUTO_MAX_Y - 1 {
        for x in 1..AUTO_MAX_X - 1 {
            if dist[index(y, x)] == 0 {
                area[index(y, x)] = true;
                queue.push_back((y, x));
            }
        }
    }
    while let Some((y, x)) = queue.pop_front() {
        for (dy, dx) in DIRS {
            let (ny, nx) = (y + dy, x + dx);
            if !inside(ny, nx) || area[index(ny, nx)] || !walkable(ny, nx) || !safe(ny, nx) {
                continue;
            }
            area[index(ny, nx)] = true;
            queue.push_back((ny, nx));
        }
    }
    area
}

/// A grid a retreat can cross: known floor-like (not lava, not unknown), no
/// known trap. A closed door is not one until it is open (review of #1090,
/// P1): opening it costs a turn, a lock can cost more, and the Borg will not
/// open a door with a monster next to it (`borg_flow.rs`, `step_closed_door`).
/// Nor is a grid a monster stands on (re-review of #1090 at `9276b75a`, P1):
/// the retreat field is built over the grids the Borg can step on now, so a
/// monster in a short corridor leaves the longer open way as the walk.
fn walkable(st: &BorgState, y: i32, x: i32) -> bool {
    let ag = st.cave.grid(x, y);
    walk_grid(u32::from(ag.feat), ag.trap, borg_cave_floor_grid(ag), ag.kill != 0)
}

/// The grids a monster walks to the staircase over (`walkable` without the
/// monsters: they do not block each other's way here).
fn terrain_walkable(st: &BorgState, y: i32, x: i32) -> bool {
    let ag = st.cave.grid(x, y);
    walk_grid(u32::from(ag.feat), ag.trap, borg_cave_floor_grid(ag), false)
}

/// `walkable` on the grid's facts (pure).
fn walk_grid(feat: u32, trap: bool, floor_like: bool, occupied: bool) -> bool {
    feat != FEAT_NONE && feat != FEAT_LAVA && feat != FEAT_CLOSED && !trap && floor_like && !occupied
}

fn lit_floor(st: &BorgState, y: i32, x: i32) -> bool {
    inside(y, x) && {
        let ag = st.cave.grid(x, y);
        u32::from(ag.feat) != FEAT_NONE && borg_cave_floor_grid(ag) && ag.info & BORG_GLOW != 0
    }
}

/// A known grid no monster walks through (granite, rubble, ...).
fn known_wall(st: &BorgState, y: i32, x: i32) -> bool {
    let ag = st.cave.grid(x, y);
    let feat = u32::from(ag.feat);
    feat != FEAT_NONE && !borg_cave_floor_grid(ag) && feat != FEAT_CLOSED
}

/// Which staircase kinds the Borg will take (review of #1090, P1: the same
/// rule for the safe area, the walk back, the level change and the
/// departure on a threat): up unless descent is forced; down only when
/// prepared for the next depth and not rising.
fn usable_kinds(st: &mut BorgState) -> (bool, bool) {
    let up = !engine::player_opt(OPT_birth_force_descend);
    let down = !st.borg.goal.rising
        && crate::borg_prepared::prepared_c_for_decision(st, st.borg.tr(BI_CDEPTH) + 1).is_none();
    (up, down)
}

/// Every known staircase of the kinds the Borg will take, `(y, x)`, but not
/// one a monster stands on (re-review of #1090, P1: the Borg cannot step
/// onto it, so it is no escape).
fn stair_grids(st: &BorgState, up: bool, down: bool) -> Vec<(i32, i32)> {
    let mut stairs = Vec::new();
    for (track, take) in [(&st.stairs.less, up), (&st.stairs.more, down)] {
        if !take {
            continue;
        }
        for i in 0..usize::try_from(track.num).unwrap_or(0) {
            stairs.push(track.at(i));
        }
    }
    free_stairs(stairs, |y, x| st.cave.grid(x, y).kill != 0)
}

/// The staircases no monster stands on (pure).
fn free_stairs(stairs: Vec<(i32, i32)>, occupied: impl Fn(i32, i32) -> bool) -> Vec<(i32, i32)> {
    stairs.into_iter().filter(|&(y, x)| !occupied(y, x)).collect()
}

/// The grids a monster can walk in from unseen at the edge of the known map
/// (pure): unknown grids and closed doors next to a walkable grid.
fn frontier(walkable: impl Fn(i32, i32) -> bool, opening: impl Fn(i32, i32) -> bool) -> Vec<(i32, i32)> {
    let mut out = Vec::new();
    for y in 1..AUTO_MAX_Y - 1 {
        for x in 1..AUTO_MAX_X - 1 {
            if opening(y, x) && DIRS.iter().any(|&(dy, dx)| walkable(y + dy, x + dx)) {
                out.push((y, x));
            }
        }
    }
    out
}

/// A monster from the frontier, `front` steps from the staircase, gets
/// there first (pure; re-review of #1090, P1: a threat from the stair side).
/// It may act `ratio - 1` times before the Borg's first action and `ratio`
/// times after each of its steps but the last; the Borg on the staircase
/// (`steps` 0) takes it at once.
fn cut_off(steps: i32, front: i32, ratio: i32) -> bool {
    steps > 0 && (ratio.max(1) - 1) + ratio.max(1) * (steps - 1) >= front
}

/// What the mode measures on the current map.
pub(crate) struct Map {
    /// *s*: steps to the nearest known staircase over walkable grids.
    dist: Vec<u16>,
    /// For each grid, the steps from the frontier to the staircase its
    /// walk leads to (`FAR`: none can come that way).
    stair_front: Vec<u16>,
    /// Chebyshev distance to the nearest unseen entry of a lit floor.
    entry: Vec<u16>,
    /// The safe area.
    area: Vec<bool>,
    /// The Borg's grid has a known route to a staircase.
    routed: bool,
    /// Chebyshev distance to the nearest known staircase (no route).
    stair_cheb: Vec<u16>,
    light: i32,
    ratio: i32,
    hp: i32,
    per_attack: i32,
    /// `stair_scum_slack_steps`: steps the walk may take beyond the strict
    /// count (owner, 2026-11-23: "a tunable parameter to make it less
    /// strict").
    slack: i32,
    /// The staircase kinds the Borg will take (`usable_kinds`).
    up: bool,
    down: bool,
}

impl Map {
    /// The map while the mode applies; `None` otherwise (no restriction).
    pub(crate) fn of(st: &mut BorgState) -> Option<Map> {
        if !active(st) {
            return None;
        }
        let (up, down) = usable_kinds(st);
        Some(Map::measure(st, up, down))
    }

    fn measure(st: &BorgState, up: bool, down: bool) -> Map {
        let stairs = stair_grids(st, up, down);
        let (dist, source) = distances_from(&stairs, |y, x| walkable(st, y, x));
        let edge = frontier(|y, x| terrain_walkable(st, y, x), |y, x| {
            let feat = u32::from(st.cave.grid(x, y).feat);
            feat == FEAT_NONE || feat == FEAT_CLOSED
        });
        let from_edge = distances(&edge, |y, x| terrain_walkable(st, y, x));
        let front_of: Vec<u16> = stairs.iter().map(|&(y, x)| from_edge[index(y, x)]).collect();
        let stair_front = source.iter().map(|&k| if k == FAR { FAR } else { front_of[usize::from(k)] }).collect();
        let lit_sources = entries(|y, x| lit_floor(st, y, x), |y, x| known_wall(st, y, x));
        let entry = distances(&lit_sources, |_, _| true);
        let stair_cheb = distances(&stairs, |_, _| true);
        let mut map = Map {
            dist,
            stair_front,
            entry,
            area: Vec::new(),
            routed: false,
            stair_cheb,
            light: engine_sight(st.borg.tr(BI_LIGHT), st.borg.tr(BI_CLASS), st.borg.tr(BI_CLEVEL)),
            ratio: value(Param::stair_scum_speed_ratio),
            hp: st.borg.tr(BI_CURHP),
            per_attack: value(Param::stair_scum_contact_damage),
            slack: slack_steps(
                value(Param::stair_scum_slack_steps),
                value(Param::stair_scum_slack_per_hp_tenths),
                st.borg.tr(BI_MAXHP),
                value(Param::stair_scum_slack_per_level_percent),
                st.borg.tr(BI_CLEVEL),
            ),
            up,
            down,
        };
        let area = safe_area(&map.dist, |y, x| walkable(st, y, x), |y, x| map.grid_safe(st, y, x));
        map.area = area;
        map.routed = !stairs.is_empty() && map.at(&map.dist, st.borg.c.y, st.borg.c.x) != FAR;
        map
    }

    fn at(&self, v: &[u16], y: i32, x: i32) -> u16 {
        if !(0..AUTO_MAX_Y).contains(&y) || !(0..AUTO_MAX_X).contains(&x) {
            return FAR;
        }
        v[index(y, x)]
    }

    /// *s(g)*: the steps from `(y, x)` to the nearest known staircase; a grid
    /// off the walk (unknown, rubble, a trap) one past its best neighbour.
    pub(crate) fn steps(&self, st: &BorgState, y: i32, x: i32) -> i32 {
        let here = self.at(&self.dist, y, x);
        if here != FAR && walkable(st, y, x) {
            return i32::from(here);
        }
        DIRS.iter()
            .map(|&(dy, dx)| self.at(&self.dist, y + dy, x + dx))
            .filter(|&d| d != FAR)
            .min()
            .map_or(i32::from(FAR), |d| i32::from(d) + 1)
    }

    /// *D(g)*: how far from `(y, x)` a monster coming at it is first seen.
    fn sight(&self, st: &BorgState, y: i32, x: i32) -> i32 {
        if u32::from(st.cave.grid(x, y).feat) == FEAT_CLOSED {
            return 1;
        }
        if !lit_floor(st, y, x) {
            return self.light;
        }
        let e = self.at(&self.entry, y, x);
        if e == FAR { self.light.max(MAX_STEPS) } else { self.light.max(i32::from(e) - 1) }
    }

    /// The retreat from `(y, x)` survives a monster first seen there.
    fn grid_safe(&self, st: &BorgState, y: i32, x: i32) -> bool {
        let s = self.steps(st, y, x);
        if s >= MAX_STEPS {
            return false;
        }
        // A closed door: the open is one action more before the retreat.
        let extra = i32::from(u32::from(st.cave.grid(x, y).feat) == FEAT_CLOSED);
        let walk = s + extra;
        // The staircase its walk leads to (an off-walk grid: its best neighbour's).
        let front = if walkable(st, y, x) {
            self.at(&self.stair_front, y, x)
        } else {
            DIRS.iter()
                .filter(|&&(dy, dx)| self.at(&self.dist, y + dy, x + dx) != FAR)
                .min_by_key(|&&(dy, dx)| self.at(&self.dist, y + dy, x + dx))
                .map_or(FAR, |&(dy, dx)| self.at(&self.stair_front, y + dy, x + dx))
        };
        retreat_ok(walk, self.slack, self.sight(st, y, x), self.ratio, self.hp, self.per_attack, i32::from(front))
    }

    fn in_area(&self, y: i32, x: i32) -> bool {
        inside(y, x) && self.area[index(y, x)]
    }

    /// A target grid: in the safe area; or, off the walk (unknown, rubble)
    /// or a closed door, next to the area and safe itself.
    pub(crate) fn target_safe(&self, st: &BorgState, y: i32, x: i32) -> bool {
        if !inside(y, x) {
            return false;
        }
        let door = u32::from(st.cave.grid(x, y).feat) == FEAT_CLOSED;
        if walkable(st, y, x) && !door {
            return self.in_area(y, x);
        }
        DIRS.iter().any(|&(dy, dx)| self.in_area(y + dy, x + dx)) && self.grid_safe(st, y, x)
    }

    /// `(y, x)` is a step nearer the staircase on the retreat walk itself
    /// (re-review of #1090, P1: a closed door is not, whatever `steps` says).
    fn nearer(&self, st: &BorgState, y: i32, x: i32) -> bool {
        walkable(st, y, x)
            && self.at(&self.dist, y, x) != FAR
            && self.at(&self.dist, y, x) < self.at(&self.dist, st.borg.c.y, st.borg.c.x)
    }

    /// The Borg's next step home on the retreat walk (re-review of #1090,
    /// P1: the return follows the safety map's terrain, not a flow that may
    /// open a closed door): the walkable neighbour nearest the staircase
    /// with no monster on it.
    fn step_home(&self, st: &BorgState) -> Option<(i32, i32)> {
        let (y, x) = (st.borg.c.y, st.borg.c.x);
        next_home(self.at(&self.dist, y, x), DIRS.iter().map(|&(dy, dx)| {
            let (ny, nx) = (y + dy, x + dx);
            let open = inside(ny, nx) && walkable(st, ny, nx);
            ((ny, nx), if open { self.at(&self.dist, ny, nx) } else { FAR })
        }))
    }

    /// A breadth-first exploration target (`borg_flow_dark_breadth_first`):
    /// in the safe area, or anything while there is no route.
    pub(crate) fn admits_target(&self, st: &BorgState, y: i32, x: i32) -> bool {
        !self.routed || self.target_safe(st, y, x)
    }

    /// The radius at the Borg's grid, for the notes.
    fn radius_here(&self, st: &BorgState) -> i32 {
        radius(self.sight(st, st.borg.c.y, st.borg.c.x), self.ratio, self.hp, self.per_attack) + self.slack
    }
}

/// A new level forgets the last one's doors and walks.
fn sync_level(st: &mut BorgState) {
    if st.stair_scum.level != st.borg.time.level {
        st.stair_scum = StairScumState {
            level: st.borg.time.level,
            last_town_ascent: st.stair_scum.last_town_ascent,
            ..StairScumState::default()
        };
    }
}

fn note_no_route(st: &mut BorgState) {
    sync_level(st);
    if !st.stair_scum.noted_no_route {
        st.stair_scum.noted_no_route = true;
        borg_note(st, "# Stair scum: no known route to a staircase; exploring toward one, no detours.");
    }
}

/// Keep the candidates of `borg_temp_*` whose `key` is the smallest
/// (`None`: dropped).
fn keep_lowest(st: &mut BorgState, key: impl Fn(&BorgState, i32, i32) -> Option<i32>) {
    let n = usize::try_from(st.flow.temp_n).unwrap_or(0);
    let mut kept: Vec<(i32, (i32, i32))> = Vec::new();
    for k in 0..n {
        let (y, x) = st.flow.temp_at(k);
        if let Some(v) = key(st, y, x) {
            kept.push((v, (y, x)));
        }
    }
    let lowest = kept.iter().map(|k| k.0).min();
    st.flow.temp_n = 0;
    for (v, (y, x)) in kept {
        if Some(v) == lowest {
            st.flow.temp_push(y, x);
        }
    }
}

/// The dark flows (`borg_flow_dark`): of the `borg_temp_*` candidates, keep
/// the safe ones nearest the staircase; with no route, those nearest a
/// known staircase as the crow flies (all, when none is known).
pub(crate) fn filter_dark(st: &mut BorgState, map: &Map) {
    if map.routed {
        keep_lowest(st, |st, y, x| map.target_safe(st, y, x).then(|| map.steps(st, y, x)));
        // Review of #1090, P2: between targets the Borg goes back toward the
        // staircase instead of chaining outward along one corridor: when the
        // nearest new target lies farther from the staircase than the Borg,
        // it walks onto the staircase first and takes the next ring from
        // there (`think`, `leave`).
        let goals: Vec<(i32, i32)> = (0..usize::try_from(st.flow.temp_n).unwrap_or(0)).map(|k| st.flow.temp_at(k)).collect();
        if return_before(st, map, &goals) {
            st.flow.temp_n = 0;
        }
    } else {
        note_no_route(st);
        keep_lowest(st, |_, y, x| Some(i32::from(map.at(&map.stair_cheb, y, x))));
    }
}

/// The exploration targets `goals` (in the safe area): when the nearest lies
/// farther from the staircase than the Borg, the Borg goes back onto the
/// staircase first (`returning`; `think` and `leave` walk it). True then.
pub(crate) fn return_before(st: &mut BorgState, map: &Map, goals: &[(i32, i32)]) -> bool {
    if !map.routed {
        return false;
    }
    let here = map.steps(st, st.borg.c.y, st.borg.c.x);
    let nearest = goals.iter().map(|&(y, x)| map.steps(st, y, x)).min();
    if !nearest.is_some_and(|n| return_first(here, n)) {
        return false;
    }
    sync_level(st);
    if !st.stair_scum.returning {
        borg_note(st, &format!("# Stair scum: the next target is {} steps out, the Borg {here}: back to the stairs first.", nearest.unwrap_or(0)));
    }
    st.stair_scum.returning = true;
    true
}

/// Back to the staircase before the next target (pure): the Borg is off
/// the staircase and the nearest target is farther out than it is.
fn return_first(here: i32, nearest_target: i32) -> bool {
    here > 0 && nearest_target > here
}

/// An item, vein or monster target: inside the safe area (none while there
/// is no route, nor once the mode is leaving the level).
pub(crate) fn admit(st: &mut BorgState, map: &Map, y: i32, x: i32) -> bool {
    sync_level(st);
    if st.stair_scum.departing {
        return false;
    }
    if !map.routed {
        note_no_route(st);
        return false;
    }
    map.target_safe(st, y, x)
}

/// A voluntary step of a flow (`borg_flow_old`, `why` its goal) onto
/// `(y, x)`: refused when it leaves the safe area and does not come nearer
/// the staircase. Flights and the walk to the staircase it leaves by
/// (`leave`, `GOAL_BORE`: the kind the depth plan allows may not be the
/// nearest) are never refused; with no route nothing is.
pub(crate) fn step_allowed(st: &mut BorgState, why: i32, y: i32, x: i32) -> bool {
    if !flag() || why == GOAL_FLEE || why == GOAL_BORE {
        return true;
    }
    let Some(map) = Map::of(st) else { return true };
    if !map.routed {
        return true;
    }
    map.nearer(st, y, x) || map.target_safe(st, y, x)
}

/// `borg_flow_step`'s door: the Borg sends the open command toward `dir`.
pub(crate) fn opened_door(st: &mut BorgState, dir: i32) {
    if !active(st) {
        return;
    }
    sync_level(st);
    let y = st.borg.c.y + i32::from(engine::ddy(dir as usize));
    let x = st.borg.c.x + i32::from(engine::ddx(dir as usize));
    if !st.stair_scum.peeked.contains(&(y, x)) {
        st.stair_scum.peek = Some(Peek { door: (y, x), stage: PeekStage::Opened });
    }
}

/// "You hear a door burst open!" (mon-move.c:1269, a monster bashed a
/// door; the message gives no grid): placed on a changed door, or not, at
/// the next think.
pub(crate) fn heard_door(st: &mut BorgState) {
    if !active(st) {
        return;
    }
    sync_level(st);
    st.stair_scum.heard = Some(engine::game_turn());
}

/// A known closed door is open and the Borg did not open it
/// (`borg_update.rs`, the `track_closed` scan): an alarm when a monster
/// there could reach the Borg's retreat (the door next to the safe area).
pub(crate) fn door_opened_by_monster(st: &mut BorgState, y: i32, x: i32) {
    sync_level(st);
    if st.stair_scum.judged.contains(&(y, x)) {
        return;
    }
    st.stair_scum.judged.push((y, x));
    st.stair_scum.changed_door_seen = Some(engine::game_turn());
    let Some(map) = Map::of(st) else { return };
    let near = map.in_area(y, x) || DIRS.iter().any(|&(dy, dx)| map.in_area(y + dy, x + dx));
    if near || !map.routed {
        borg_note(st, &format!("# Stair scum: a monster opened the door at {y},{x}; back onto the stairs."));
        st.stair_scum.alarm = Some(engine::game_turn());
    }
}

/// What a door sound leads to (pure; review of #1061, P2): a changed door
/// seen since the sound was judged by its grid; none seen, the Borg goes
/// onto the staircase.
fn sound_alarms(heard: i32, changed_door_seen: Option<i32>) -> bool {
    !changed_door_seen.is_some_and(|seen| seen.wrapping_sub(heard) >= 0)
}

/// The alarm holds for `window` game turns from `since` (pure; 0 = never).
fn alarm_holds(since: i32, now: i32, window: i32) -> bool {
    window > 0 && now.wrapping_sub(since) < window
}

fn on_stairs(st: &BorgState) -> bool {
    matches!(u32::from(st.cave.grid(st.borg.c.x, st.borg.c.y).feat), FEAT_LESS | FEAT_MORE)
}

/// While the mode applies and the Borg stands on a staircase: it rests
/// there, it does not walk to another grid to recover.
pub(crate) fn holds_on_stairs(st: &mut BorgState) -> bool {
    if !active(st) || !on_stairs(st) {
        return false;
    }
    let (up, down) = usable_kinds(st);
    match u32::from(st.cave.grid(st.borg.c.x, st.borg.c.y).feat) {
        FEAT_LESS => up,
        _ => down,
    }
}

/// Stay still a turn (`,`, as the Recall wait does).
fn hold(st: &mut BorgState, why: &str) -> bool {
    borg_note(st, &format!("# Stair scum: {why}."));
    borg_keypress(st, keycode_t::from(b','));
    st.borg.in_shop = false;
    true
}

/// The neighbour to step to (pure): the one with the smallest walk, if
/// smaller than `here`'s; the first in `DIRS` order on a tie.
fn next_home(here: u16, neighbours: impl Iterator<Item = ((i32, i32), u16)>) -> Option<(i32, i32)> {
    let mut best: Option<((i32, i32), u16)> = None;
    for (grid, d) in neighbours {
        if d != FAR && d < here && best.is_none_or(|(_, b)| d < b) {
            best = Some((grid, d));
        }
    }
    best.map(|(grid, _)| grid)
}

/// One step of the walk back onto the nearest usable staircase, over the
/// retreat walk's own grids (`Map::step_home`): a one-grid flow to it.
fn step_to_stairs(st: &mut BorgState, map: &Map, why: &str) -> bool {
    let Some((y, x)) = map.step_home(st) else { return false };
    crate::borg_flow::borg_flow_clear(st);
    crate::borg_flow::borg_flow_enqueue_grid(st, y, x);
    crate::borg_flow::borg_flow_spread(st, 1, false, true, false, -1, false);
    let who = std::ffi::CString::new(format!("stairs ({why})")).unwrap_or_default();
    if !crate::borg_flow::borg_flow_commit(st, &who, GOAL_MISC) {
        return false;
    }
    crate::borg_flow::borg_flow_old(st, GOAL_MISC)
}

/// After caution (`borg_think_dungeon`): the door sound, the alarm, the
/// door peek and the walk back into the safe area. True when it acted.
pub(crate) fn think(st: &mut BorgState) -> bool {
    if !flag() {
        return false;
    }
    sync_level(st);
    let Some(map) = Map::of(st) else {
        st.stair_scum.returning = false;
        st.stair_scum.peek = None;
        st.stair_scum.alarm = None;
        st.stair_scum.heard = None;
        return false;
    };
    let (y, x) = (st.borg.c.y, st.borg.c.x);
    let here = map.steps(st, y, x);

    // 3. A door sound no changed door explains: onto the staircase.
    if let Some(heard) = st.stair_scum.heard.take() {
        if sound_alarms(heard, st.stair_scum.changed_door_seen) {
            borg_note(st, "# Stair scum: a door burst open, no door to place it; back onto the stairs.");
            st.stair_scum.alarm = Some(heard);
        }
    }

    // 3. A door opened: onto the staircase, and wait there.
    if let Some(since) = st.stair_scum.alarm {
        if map.routed && alarm_holds(since, engine::game_turn(), value(Param::stair_scum_alarm_turns)) {
            if here == 0 {
                return hold(st, "holding on the stairs after a door opened");
            }
            if step_to_stairs(st, &map, "a door opened") {
                return true;
            }
        }
        st.stair_scum.alarm = None;
    }

    // 3. A door it opened: step back toward the staircase, then wait a turn.
    if let Some(peek) = st.stair_scum.peek {
        let (dy, dx) = peek.door;
        let feat = u32::from(st.cave.grid(dx, dy).feat);
        let adjacent = (dy - y).abs() <= 1 && (dx - x).abs() <= 1;
        match (feat, peek.stage) {
            // Not open yet (locked, or the command failed): the flows retry.
            (FEAT_CLOSED, PeekStage::Opened) if adjacent => {}
            (FEAT_OPEN | FEAT_BROKEN, PeekStage::Opened) => {
                st.stair_scum.peek = Some(Peek { stage: PeekStage::Back, ..peek });
                if map.routed && here > 0 && step_to_stairs(st, &map, "stepping back from a door") {
                    return true;
                }
                return hold(st, "waiting a turn by the door it opened");
            }
            (FEAT_OPEN | FEAT_BROKEN, PeekStage::Back) => {
                st.stair_scum.peek = None;
                st.stair_scum.peeked.push(peek.door);
                st.borg.goal.type_0 = 0;
                return hold(st, "waiting a turn before the open door");
            }
            _ => {
                st.stair_scum.peek = None;
                st.stair_scum.peeked.push(peek.door);
            }
        }
    }

    // 4. A committed departure (`leave`'s walk to the staircase it leaves by)
    // goes on before the return rule (re-review of #1090, P2): on DL1 the
    // preferred down staircase may lie past an unsafe grid, and the return
    // would walk the Borg back to the nearer up staircase every turn.
    if st.stair_scum.departing && i32::from(st.borg.goal.type_0) == GOAL_BORE {
        if crate::borg_flow::borg_flow_old(st, GOAL_BORE) {
            return true;
        }
    }

    // 2. Outside the safe area: back onto the staircase (all the way).
    if !map.routed || here == 0 {
        st.stair_scum.returning = false;
        return false;
    }
    if st.stair_scum.returning || !map.in_area(y, x) {
        if !st.stair_scum.returning {
            let r = map.radius_here(st);
            borg_note(st, &format!("# Stair scum: {here} steps from the stairs, outside the safe area (radius here {r}): returning."));
        }
        st.stair_scum.returning = step_to_stairs(st, &map, "stair scum return");
        return st.stair_scum.returning;
    }
    false
}

/// Which staircase kinds to leave by, in order (pure). `true` = up.
/// "up or down within the depth plan, preferring DL1-2": on DL1 down first
/// (to DL2) when prepared for it, else up; deeper, up first, then down
/// when prepared for the next depth.
fn leave_order(depth: i32, up_ok: bool, down_ok: bool) -> Vec<bool> {
    let mut order = Vec::new();
    if depth <= 1 {
        if down_ok {
            order.push(false);
        }
        if up_ok {
            order.push(true);
        }
    } else {
        if up_ok {
            order.push(true);
        }
        if down_ok {
            order.push(false);
        }
    }
    order
}

/// 4. Nothing left inside the safe area (after the exploration flows): take
/// a staircase to a new level.
pub(crate) fn leave(st: &mut BorgState) -> bool {
    if !flag() {
        return false;
    }
    let Some(map) = Map::of(st).filter(|m| m.routed) else {
        return false;
    };
    sync_level(st);
    // The exploration flows found nothing because the Borg goes back to the
    // staircase between targets (`filter_dark`): that walk, not a new level.
    if st.stair_scum.returning {
        st.stair_scum.returning = step_to_stairs(st, &map, "back to the stairs between targets");
        if st.stair_scum.returning {
            return true;
        }
    }
    let depth = st.borg.tr(BI_CDEPTH);
    let (up_ok, down_ok) = (map.up, map.down);
    let feat = u32::from(st.cave.grid(st.borg.c.x, st.borg.c.y).feat);
    for up in leave_order(depth, up_ok, down_ok) {
        let (here, k) = if up { (FEAT_LESS, b'<') } else { (FEAT_MORE, b'>') };
        if feat == here {
            if up && depth == 1 {
                // Angband cmd-cave.c:56–95 generates the new level without
                // an ordinary move. Break same-turn town rerolls with a
                // hold (cmd-cave.c:1580), when there is no threat here.
                let turn = engine::game_turn();
                let pos = st.borg.c;
                if st.stair_scum.last_town_ascent == Some(turn)
                    && !st.borg.goal.fleeing
                    && crate::borg_danger::borg_danger(st, pos.y, pos.x, 1, true, false) <= 0
                {
                    return hold(st, "waiting before another same-turn return to town");
                }
                st.stair_scum.last_town_ascent = Some(turn);
            }
            borg_note(st, &format!("# Stair scum: nothing left inside the safe area; taking the stairs {}.", if up { "up" } else { "down" }));
            st.stair_scum.departing = true;
            borg_keypress(st, keycode_t::from(k));
            return true;
        }
    }
    for up in leave_order(depth, up_ok, down_ok) {
        let walked = if up {
            crate::borg_flow_stairs::borg_flow_stair_less(st, GOAL_BORE, false)
        } else {
            crate::borg_flow_stairs::borg_flow_stair_more(st, GOAL_BORE, false, false)
        };
        if walked {
            st.stair_scum.departing = true;
            borg_note(st, &format!("# Stair scum: nothing left inside the safe area; to the stairs {}.", if up { "up" } else { "down" }));
            return true;
        }
    }
    false
}

/// 5. On a staircase, a threat the fight race does not win (it acts before
/// caution) is left by the staircase: up, or down when prepared for the
/// next depth. `None` when the mode does not decide.
pub(crate) fn leave_on_threat(st: &mut BorgState, pos_danger: i32, threshold: i32) -> Option<bool> {
    if !flag() || !active(st) || pos_danger < threshold {
        return None;
    }
    let (up, down) = usable_kinds(st);
    match u32::from(st.cave.grid(st.borg.c.x, st.borg.c.y).feat) {
        FEAT_LESS if up => Some(true),
        FEAT_MORE if down => Some(false),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Review of #1061, P1: Codex's case, a +10 monster first seen two grids
    /// behind, 12 HP, 8 per attack. With the monster's spare action before
    /// the Borg's first, the walk of 3 steps takes three attacks; even with
    /// no spare action, two (16 HP, lethal before the stair command).
    #[test]
    fn the_attacks_on_the_way_are_counted() {
        // First sight at 2: one spare action closes to 1.
        assert_eq!(attacks_on_retreat(0, 2, 2), 0, "on the stairs: the stair command first");
        assert_eq!(attacks_on_retreat(1, 2, 2), 1);
        assert_eq!(attacks_on_retreat(3, 2, 2), 3);
        assert!(!survives(3, 2, 2, 12, 8), "Codex: three steps are lethal");
        assert!(survives(1, 2, 2, 12, 8));
        // First sight at 1 (adjacent): it bites before the first step.
        assert_eq!(attacks_on_retreat(0, 1, 2), 1);
        assert_eq!(attacks_on_retreat(2, 1, 2), 3);
        // A monster as fast as the Borg never catches up from 2.
        assert_eq!(attacks_on_retreat(10, 2, 1), 0);
    }

    /// The radius: a torch in a dark corridor (the engine lights 1 grid) at
    /// 12 HP and 4 per attack; a lit room's centre 8 from its entries.
    #[test]
    fn the_radius_follows_sight_speed_and_hp() {
        let torch = engine_sight(2, 0, 1);
        assert_eq!(torch, 1, "LIGHT_2 lights distance 1 (cave-view.c:658)");
        assert_eq!(radius(torch, 2, 12, 4), 1);
        assert_eq!(radius(torch, 2, 7, 4), 0, "7 HP: not one step");
        assert_eq!(radius(7, 2, 12, 4), 7, "seen 7 off in a lit room");
        assert_eq!(radius(7, 3, 12, 4), 3, "monsters three times as fast");
        // A Necromancer's unlight at CL1 with no light: 1 + 0 - 0.
        assert_eq!(engine_sight(0, CLASS_NECROMANCER, 1), 1);
        assert_eq!(engine_sight(0, CLASS_NECROMANCER, 12), 3);
    }

    /// Owner, 2026-11-23: the slack loosens the radius by its steps; 0 is
    /// the strict count. A torch, 12 HP, 4 per attack: 1, 2 and 3 steps.
    #[test]
    fn the_slack_widens_the_radius_by_its_steps() {
        let walk = |slack: i32| (0..MAX_STEPS).take_while(|&s| survives(slackened(s, slack), 1, 2, 12, 4)).last().unwrap();
        assert_eq!(walk(0), 1);
        assert_eq!(walk(1), 2);
        assert_eq!(walk(2), 3, "the owner's \"two or three steps\"");
        assert_eq!(slackened(1, 5), 0, "never below the stairs themselves");
        assert_eq!(slackened(4, -1), 4, "a negative slack is none");
        // The rate: a tenth of a step per max HP point, on top of the steps.
        assert_eq!(slack_steps(0, 1, 12, 0, 1), 1, "12 HP: one step");
        assert_eq!(slack_steps(0, 1, 24, 0, 1), 2, "24 HP: two");
        assert_eq!(slack_steps(1, 1, 9, 0, 1), 1, "9 HP: the fixed step only");
        assert_eq!(slack_steps(0, 0, 24, 0, 1), 0, "no rate: strict");
        // By level (owner: "50 means that druids at level 3 get to slack 2").
        assert_eq!(slack_steps(1, 0, 0, 50, 1), 1, "a Druid at CL1: its base");
        assert_eq!(slack_steps(1, 0, 0, 50, 2), 1, "CL2: half a step, rounded down");
        assert_eq!(slack_steps(1, 0, 0, 50, 3), 2, "CL3: slack 2");
        assert_eq!(slack_steps(1, 0, 0, 50, 5), 3, "CL5: slack 3");
        assert_eq!(slack_steps(2, 1, 12, 50, 3), 4, "steps, HP and level add up");
    }

    #[test]
    fn the_threshold_is_the_class_value_plus_the_race_delta() {
        assert!(below_threshold(24, 25));
        assert!(!below_threshold(25, 25));
        assert!(below_threshold(27, 25 + 3), "a hobbit's +3");
        assert!(!below_threshold(21, 25 - 5), "a half-troll's -5");
    }

    /// Review, P1: a lit room with a dark side entrance. Its warning
    /// distance is the entrance's, not the room's extent.
    #[test]
    fn a_dark_side_entrance_shortens_the_lit_room() {
        // Lit room rows 10..=26, cols 20..=36, walls around, a dark doorway
        // in the west wall at (18, 19).
        let lit = |y: i32, x: i32| (10..=26).contains(&y) && (20..=36).contains(&x);
        let wall = |y: i32, x: i32| {
            (9..=27).contains(&y) && (19..=37).contains(&x) && !lit(y, x) && (y, x) != (18, 19)
        };
        let e = entries(lit, wall);
        assert_eq!(e, vec![(18, 19)], "only the doorway");
        let d = distances(&e, |_, _| true);
        assert_eq!(d[index(18, 28)], 9, "the centre is 9 from the doorway");
        assert_eq!(d[index(18, 21)], 2, "a grid by the doorway");
        // No entry at all: the room's walls are not entries.
        assert!(entries(lit, |y, x| (9..=27).contains(&y) && (19..=37).contains(&x) && !lit(y, x)).is_empty());
    }

    /// Review, P1: a frontier whose only route to the stairs crosses an
    /// unknown grid is not reachable; loot beyond the area is not admitted.
    #[test]
    fn the_safe_area_needs_a_known_route_of_safe_grids() {
        // A corridor along row 5 from a staircase at x = 5; x = 9 unknown.
        let walk = |y: i32, x: i32| y == 5 && (5..=16).contains(&x) && x != 9;
        let dist = distances(&[(5, 5)], walk);
        assert_eq!(dist[index(5, 8)], 3);
        assert_eq!(dist[index(5, 10)], FAR, "past the unknown grid: no route");
        // Safe while within 2 steps (say): the area stops there.
        let area = safe_area(&dist, walk, |y, x| dist[index(y, x)] <= 2);
        assert!(area[index(5, 7)]);
        assert!(!area[index(5, 8)], "loot at 3 steps is outside");
        assert!(!area[index(5, 12)]);
        // A safe grid cut off by an unsafe one is not in the area.
        let area = safe_area(&dist, walk, |y, x| dist[index(y, x)] != 2);
        assert!(!area[index(5, 8)]);
    }

    /// Review of #1090, P1: a closed door is not on the retreat walk until
    /// it is open; a corridor behind one is cut off from the staircase.
    #[test]
    fn a_closed_door_is_not_on_the_retreat() {
        assert!(!walk_grid(FEAT_CLOSED, false, false, false));
        assert!(walk_grid(FEAT_OPEN, false, true, false), "an open door is floor");
        assert!(walk_grid(FEAT_BROKEN, false, true, false));
        assert!(!walk_grid(FEAT_OPEN, true, true, false), "nor a known trap");
        assert!(!walk_grid(FEAT_OPEN, false, true, true), "nor a grid a monster stands on");
        // A staircase at x = 5, a closed door at x = 7: the floor beyond has
        // no route, the door itself is one past its neighbour.
        let walk = |y: i32, x: i32| y == 5 && (5..=12).contains(&x) && x != 7;
        let dist = distances(&[(5, 5)], walk);
        assert_eq!(dist[index(5, 6)], 1);
        assert_eq!(dist[index(5, 8)], FAR, "beyond the closed door");
    }

    /// Review of #1090, P1: only detection the exploration casts lifts the
    /// mode. A CL3 Mage that knows Detect Monsters, with the shipped spell
    /// gate at CL10, stays in it.
    #[test]
    fn only_used_detection_lifts_the_mode() {
        assert!(!detection_used(true, 3, 10, false, false), "Mage CL3, gate 10");
        assert!(detection_used(true, 10, 10, false, false), "at the gate");
        assert!(detection_used(false, 1, 10, true, false), "the Detection spell, no gate");
        assert!(detection_used(false, 1, 10, false, true), "a Rod of Detection");
        assert!(!detection_used(false, 30, 0, false, false));
    }

    /// Review of #1090, P1: the staircases the safe area is measured from
    /// are those the Borg takes. With only a down staircase known and the
    /// Borg not prepared for the next depth, nothing anchors the area.
    #[test]
    fn the_area_counts_only_stairs_the_borg_takes() {
        let walk = |y: i32, x: i32| y == 5 && (5..=12).contains(&x);
        let none: Vec<(i32, i32)> = Vec::new();
        assert!(distances(&none, walk).iter().all(|&d| d == FAR), "no usable staircase: no route");
        let dist = distances(&[(5, 12)], walk);
        assert_eq!(dist[index(5, 10)], 2, "measured from the usable one only");
    }

    /// Review of #1090, P2: two branches off one staircase. At the end of
    /// branch A (2 steps out), with only targets 3 steps out (A's next grid
    /// or B's), the Borg goes back first; a target nearer the staircase
    /// than it (B at 1) is taken directly; on the staircase it never
    /// returns.
    #[test]
    fn between_targets_it_returns_toward_the_stairs() {
        // Row 5: branch B west of the staircase at x = 10, branch A east.
        let walk = |y: i32, x: i32| y == 5 && (6..=14).contains(&x);
        let dist = distances(&[(5, 10)], walk);
        let here = i32::from(dist[index(5, 12)]);
        assert_eq!(here, 2, "the Borg at A's second grid");
        let a_next = i32::from(dist[index(5, 13)]);
        let b_far = i32::from(dist[index(5, 7)]);
        let b_near = i32::from(dist[index(5, 9)]);
        assert!(return_first(here, a_next.min(b_far)), "both rings 3 out: back first");
        assert!(!return_first(here, b_near), "B's grid 1 out: straight there (past the stairs)");
        assert!(!return_first(here, here), "the same ring: no return");
        assert!(!return_first(0, a_next), "on the staircase: go");
    }

    /// Re-review of #1090, P1: a monster on the only staircase leaves no
    /// escape, and one from the frontier near the staircase can get there
    /// first.
    #[test]
    fn the_retreat_can_be_cut_off() {
        let stairs = vec![(5, 5), (9, 9)];
        assert_eq!(free_stairs(stairs.clone(), |y, _| y == 5), vec![(9, 9)], "the occupied one is dropped");
        assert!(free_stairs(vec![(5, 5)], |_, _| true).is_empty(), "the only one occupied: no route");
        // A +10 monster from a frontier grid 3 steps from the staircase.
        assert!(!cut_off(0, 3, 2), "on the staircase: it takes it");
        assert!(!cut_off(1, 3, 2), "one step: the monster needs 3 moves, has 1");
        assert!(cut_off(2, 3, 2), "two steps: 1 + 2 moves reach the staircase");
        assert!(!cut_off(2, 4, 2));
        assert!(cut_off(1, 1, 2), "the frontier next to the staircase");
        // The frontier: unknown grids and closed doors next to the walk.
        let walk = |y: i32, x: i32| y == 5 && (5..=8).contains(&x);
        let edge = frontier(walk, |y, x| y == 5 && x == 9 || y == 4 && x == 5);
        assert_eq!(edge, vec![(4, 5), (5, 9)]);
        let d = distances(&edge, walk);
        assert_eq!(d[index(5, 5)], 1, "the staircase at (5,5) is 1 from the unknown grid above it");
        let (dist, from) = distances_from(&[(5, 5), (5, 8)], walk);
        assert_eq!((dist[index(5, 6)], from[index(5, 6)]), (1, 0));
        assert_eq!((dist[index(5, 7)], from[index(5, 7)]), (1, 1));
    }

    /// Re-review of #1090, P1: the return steps over the walk's own grids:
    /// a closed door next to the staircase is not a shortcut.
    #[test]
    fn the_return_walks_the_retreat_grids() {
        // The Borg 3 from the staircase; neighbour A 2 (walkable), B FAR (a
        // closed door that `steps` would score 2), C 4.
        let n = [((5, 6), 2u16), ((4, 7), FAR), ((5, 8), 4)];
        assert_eq!(next_home(3, n.into_iter()), Some((5, 6)));
        assert_eq!(next_home(3, [((4, 7), FAR)].into_iter()), None, "only the door: no step");
        assert_eq!(next_home(1, [((5, 5), 0), ((4, 5), 0)].into_iter()), Some((5, 5)), "a tie: the first");
        assert_eq!(next_home(2, [((5, 6), 2)].into_iter()), None, "not nearer");
    }

    /// Re-review of #1090 at `9276b75a`, P1: a monster in the three-step
    /// shortcut leaves the open five-step detour as the walk home, and the
    /// first step of the detour is a step home.
    #[test]
    fn a_monster_in_the_shortcut_leaves_the_detour() {
        // The staircase at (5, 5). Shortcut along row 5 to (5, 8); detour
        // round row 7: (6,6)..(7,7)..(6,9)? A plain loop: row 5 and row 7,
        // joined at x = 5 and x = 9 through rows 6.
        let floor = |y: i32, x: i32| (y == 5 && (5..=9).contains(&x)) || (y == 7 && (5..=9).contains(&x))
            || ((y == 6) && (x == 4 || x == 10)) || (y == 5 && x == 4) || (y == 7 && x == 4)
            || (y == 5 && x == 10) || (y == 7 && x == 10);
        let monster = |y: i32, x: i32| (y, x) == (5, 7);
        let walk = |y: i32, x: i32| floor(y, x) && !monster(y, x);
        let through = distances(&[(5, 5)], floor);
        let open = distances(&[(5, 5)], walk);
        assert_eq!(through[index(5, 8)], 3, "the shortcut");
        assert!(open[index(5, 8)] > 3, "the open way is the detour");
        // From (5, 8) the step home is onto the detour, not the monster.
        let here = open[index(5, 8)];
        let n = DIRS.iter().map(|&(dy, dx)| {
            let (y, x) = (5 + dy, 8 + dx);
            ((y, x), if walk(y, x) { open[index(y, x)] } else { FAR })
        });
        let step = next_home(here, n).expect("a step home exists");
        assert_ne!(step, (5, 7));
    }

    /// Re-review of #1090 at `9276b75a`, P1: the slack loosens the damage
    /// budget, not the race to the staircase.
    #[test]
    fn the_cutoff_is_timed_on_the_real_walk() {
        // A Mage's slack 2, a two-step walk, a +10 monster one step from the
        // staircase, full HP 20: the damage budget passes, the race does not.
        assert!(survives(slackened(2, 2), 1, 2, 20, 4));
        assert!(!retreat_ok(2, 2, 1, 2, 20, 4, 1), "the monster reaches the staircase first");
        assert!(retreat_ok(2, 2, 1, 2, 20, 4, 9), "a far frontier: safe");
        assert!(retreat_ok(0, 2, 1, 2, 20, 4, 1), "on the staircase");
    }

    /// Review, P2: a door sound is placed by a changed door seen at or after
    /// it; none seen, the Borg goes onto the staircase.
    #[test]
    fn an_unplaced_door_sound_sends_it_to_the_stairs() {
        assert!(sound_alarms(1000, None), "no changed door: retreat");
        assert!(sound_alarms(1000, Some(900)), "a door seen before the sound does not place it");
        assert!(!sound_alarms(1000, Some(1000)), "a changed door this turn: judged by its grid");
    }

    #[test]
    fn the_alarm_window() {
        assert!(alarm_holds(1000, 1050, 100));
        assert!(!alarm_holds(1000, 1100, 100));
        assert!(!alarm_holds(1000, 1000, 0), "0 = no alarm");
    }

    #[test]
    fn leaving_prefers_dl1_and_dl2() {
        assert_eq!(leave_order(1, true, true), vec![false, true], "DL1: down to DL2 first");
        assert_eq!(leave_order(1, true, false), vec![true], "DL1, not prepared for DL2: up");
        assert_eq!(leave_order(2, true, true), vec![true, false], "DL2: up first");
        assert_eq!(leave_order(3, false, true), vec![false], "force_descend: down only");
        assert!(leave_order(4, false, false).is_empty());
    }
}
