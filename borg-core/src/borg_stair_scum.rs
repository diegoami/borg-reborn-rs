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
//! `stair_scum_max_hp_race_delta`, per race) and the Borg cannot detect
//! monsters (a Detect Monsters, Reveal Monsters or Detection spell it can
//! cast, or a Rod of Detection carried: the spells and the rod with
//! `DETECT_VISIBLE_MONSTERS`, `lib/gamedata/class.txt` and `object.txt`;
//! Detect Invisible and Detect Life do not count).
//!
//! 1. **The safe radius.** A target grid *g* (an unknown grid, a door, an
//!    item, a monster) is taken only if *s(g)* <= *R(g)*. *s(g)* is the walk,
//!    in steps over known grids, from *g* to the nearest known staircase of
//!    either kind (`Map`; an unknown grid counts one step past its nearest
//!    known neighbour). *R(g)* = *D(g)* / (*r* - 1) + *HP* /
//!    `stair_scum_contact_damage`: *D(g)* is how far off a monster coming at
//!    *g* is first seen (the Borg's own light radius, or, on a lit room
//!    floor, the shortest of the eight straight runs of lit floor from *g*,
//!    if longer); *r* is `stair_scum_speed_ratio` (2: the +10 monsters of
//!    DL1-2, `monster.txt` Grip, Fang, crow, wild cat `speed:120`). A torch
//!    in a corridor at 12 HP: 2 / 1 + 12 / 8 = 3 steps.
//! 2. **Radial exploration.** The dark flows keep only their candidates
//!    nearest the staircase; a Borg outside its radius (its HP fell, a
//!    fight moved it) walks back onto the staircase before anything but
//!    caution.
//! 3. **Doors.** A door the Borg opens is not walked through at once: the
//!    Borg steps back toward the staircase (or stays, on it) and waits a turn.
//!    "You hear a door burst open!", or a known closed door seen open that
//!    the Borg did not open within its radius, sends it onto the staircase
//!    for `stair_scum_alarm_turns` game turns.
//! 4. **Stair scum.** Nothing left inside the radius: it takes a staircase,
//!    up or down within the depth plan, preferring DL1-2.
//! 5. **Fight from the staircase.** On a staircase, a threat it does not win
//!    (`winning_fight` acts before caution) makes it take the stairs
//!    (`borg_caution::leave_stairs_on_monster`); it rests on the staircase.
//! 6. At max HP >= the threshold the mode ends; C's leash is always kept.
//!
//! With no staircase known, or none reachable from the Borg's grid over
//! known grids, the mode does not restrict the targets (#1062's holes
//! stay C's). Every value is a hypothesis until borg-zoo measures it
//! (RULES 14); `stair_scum` ships 0, so play is unchanged.

use crate::borg::BorgTime;
use crate::borg_cave::{AUTO_MAX_X, AUTO_MAX_Y};
use crate::borg_cave_util::borg_cave_floor_grid;
use crate::borg_headers::borg_magic_h::{DETECTION, DETECT_MONSTERS, REVEAL_MONSTERS};
use crate::borg_headers::borg_trait_h::{BI_CDEPTH, BI_CURHP, BI_MAXHP};
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

/// No known route.
const FAR: u16 = u16::MAX;

/// The mode's memory, for the current level only (`level` is the level's
/// arrival time; a new level starts afresh).
#[derive(Default)]
pub struct StairScumState {
    level: BorgTime,
    /// Walking back onto the staircase (until it stands on one).
    returning: bool,
    /// A door the Borg opened and has not peeked through yet.
    peek: Option<Peek>,
    /// Game turn of the last door heard or seen opened within the radius.
    alarm: Option<i32>,
    /// Doors already peeked at on this level, `(y, x)`.
    peeked: Vec<(i32, i32)>,
    /// Doors a monster opened that already raised the alarm, `(y, x)`
    /// (`track_closed` keeps them until the Borg opens a door itself).
    alarmed: Vec<(i32, i32)>,
    /// The widest *D* of a target admitted since the Borg last stood on a
    /// staircase: the walk to a lit-room target may cross a dark corridor
    /// whose own radius is smaller, and that walk is not a reason to turn
    /// back (it would turn back and come again for ever).
    sight_allowed: i32,
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

/// The mode's switch and thresholds, in the order the issue gives them.
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

/// The Borg can find monsters before they find it: a monster detection
/// spell it can cast, or a Rod of Detection carried (charging or not).
fn detects_monsters(st: &BorgState) -> bool {
    [DETECT_MONSTERS, REVEAL_MONSTERS, DETECTION]
        .into_iter()
        .any(|spell| crate::borg_magic::borg_spell_legal(st, spell))
        || carries(st, TV_ROD as i32, st.item_val.sv_rod_detection)
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

/// R = D / (r - 1) + HP / contact damage (pure; `r` >= 2, contact >= 1 by
/// their ranges).
fn radius(sight: i32, ratio: i32, hp: i32, contact: i32) -> i32 {
    sight.max(0) / (ratio - 1).max(1) + hp.max(0) / contact.max(1)
}

/// The eight directions, `(dy, dx)`.
const DIRS: [(i32, i32); 8] = [(-1, -1), (-1, 0), (-1, 1), (0, -1), (0, 1), (1, -1), (1, 0), (1, 1)];

/// The shortest of the eight straight runs of lit floor from `(y, x)`
/// (pure: `lit_floor` says whether a grid is known, lit floor).
fn lit_extent(y: i32, x: i32, lit_floor: impl Fn(i32, i32) -> bool) -> i32 {
    DIRS.iter()
        .map(|&(dy, dx)| {
            let mut t = 0;
            while t < AUTO_MAX_X && lit_floor(y + dy * (t + 1), x + dx * (t + 1)) {
                t += 1;
            }
            t
        })
        .min()
        .unwrap_or(0)
}

/// Steps from every grid to the nearest of `sources` over grids
/// `passable` says can be walked (pure; 8-connected, one step each;
/// `FAR` where no route is known).
fn stair_distances(sources: &[(i32, i32)], passable: impl Fn(i32, i32) -> bool) -> Vec<u16> {
    let index = |y: i32, x: i32| (y * AUTO_MAX_X + x) as usize;
    let mut dist = vec![FAR; (AUTO_MAX_X * AUTO_MAX_Y) as usize];
    let mut queue = std::collections::VecDeque::new();
    for &(y, x) in sources {
        if (0..AUTO_MAX_Y).contains(&y) && (0..AUTO_MAX_X).contains(&x) && dist[index(y, x)] == FAR {
            dist[index(y, x)] = 0;
            queue.push_back((y, x));
        }
    }
    while let Some((y, x)) = queue.pop_front() {
        let next = dist[index(y, x)] + 1;
        for (dy, dx) in DIRS {
            let (ny, nx) = (y + dy, x + dx);
            if !(1..AUTO_MAX_Y - 1).contains(&ny) || !(1..AUTO_MAX_X - 1).contains(&nx) {
                continue;
            }
            if dist[index(ny, nx)] != FAR || !passable(ny, nx) {
                continue;
            }
            dist[index(ny, nx)] = next;
            queue.push_back((ny, nx));
        }
    }
    dist
}

/// What the mode measures on the current map: the stair distances and the
/// numbers of the radius.
pub(crate) struct Map {
    dist: Vec<u16>,
    light: i32,
    ratio: i32,
    hp: i32,
    contact: i32,
}

/// Every known staircase of either kind, `(y, x)`.
fn stair_grids(st: &BorgState) -> Vec<(i32, i32)> {
    let mut stairs = Vec::new();
    for track in [&st.stairs.less, &st.stairs.more] {
        for i in 0..usize::try_from(track.num).unwrap_or(0) {
            stairs.push(track.at(i));
        }
    }
    stairs
}

/// A grid the walk may cross: known, floor-like (not lava) or a closed door.
fn walkable(st: &BorgState, y: i32, x: i32) -> bool {
    let ag = st.cave.grid(x, y);
    let feat = u32::from(ag.feat);
    feat != FEAT_NONE && feat != FEAT_LAVA && (borg_cave_floor_grid(ag) || feat == FEAT_CLOSED)
}

impl Map {
    /// The map while the mode applies and a known staircase can be reached
    /// from the Borg's grid; `None` otherwise (no restriction).
    pub(crate) fn of(st: &BorgState) -> Option<Map> {
        if !active(st) {
            return None;
        }
        Map::measure(st)
    }

    fn measure(st: &BorgState) -> Option<Map> {
        let stairs = stair_grids(st);
        if stairs.is_empty() {
            return None;
        }
        let dist = stair_distances(&stairs, |y, x| walkable(st, y, x));
        let map = Map {
            dist,
            light: crate::borg_cave_light::own_light_radius(st),
            ratio: value(Param::stair_scum_speed_ratio),
            hp: st.borg.tr(BI_CURHP),
            contact: value(Param::stair_scum_contact_damage),
        };
        (map.steps(st, st.borg.c.y, st.borg.c.x) != i32::from(FAR)).then_some(map)
    }

    fn at(&self, y: i32, x: i32) -> u16 {
        if !(0..AUTO_MAX_Y).contains(&y) || !(0..AUTO_MAX_X).contains(&x) {
            return FAR;
        }
        self.dist[(y * AUTO_MAX_X + x) as usize]
    }

    /// *s(g)*: the steps from `(y, x)` to the nearest known staircase; a grid
    /// off the walk (unknown, rubble, a wall) one past its best neighbour.
    pub(crate) fn steps(&self, st: &BorgState, y: i32, x: i32) -> i32 {
        let here = self.at(y, x);
        if here != FAR && walkable(st, y, x) {
            return i32::from(here);
        }
        DIRS.iter()
            .map(|&(dy, dx)| self.at(y + dy, x + dx))
            .filter(|&d| d != FAR)
            .min()
            .map_or(i32::from(FAR), |d| i32::from(d) + 1)
    }

    /// *D(g)*: how far from `(y, x)` a monster coming at it is first seen.
    fn sight(&self, st: &BorgState, y: i32, x: i32) -> i32 {
        let lit = |y: i32, x: i32| {
            (0..AUTO_MAX_Y).contains(&y) && (0..AUTO_MAX_X).contains(&x) && {
                let ag = st.cave.grid(x, y);
                u32::from(ag.feat) != FEAT_NONE && borg_cave_floor_grid(ag) && ag.info & BORG_GLOW != 0
            }
        };
        if lit(y, x) { self.light.max(lit_extent(y, x, lit)) } else { self.light }
    }

    /// *R(g)*.
    pub(crate) fn radius(&self, st: &BorgState, y: i32, x: i32) -> i32 {
        radius(self.sight(st, y, x), self.ratio, self.hp, self.contact)
    }

    /// *s(g)* <= *R(g)*.
    pub(crate) fn safe(&self, st: &BorgState, y: i32, x: i32) -> bool {
        self.steps(st, y, x) <= self.radius(st, y, x)
    }
}

/// A target grid of a flow (an item, a monster): inside the radius. An
/// admitted target widens the walk's allowance (`sight_allowed`).
pub(crate) fn admit(st: &mut BorgState, map: &Map, y: i32, x: i32) -> bool {
    if !map.safe(st, y, x) {
        return false;
    }
    sync_level(st);
    let sight = map.sight(st, y, x);
    st.stair_scum.sight_allowed = st.stair_scum.sight_allowed.max(sight);
    true
}

/// A new level forgets the last one's doors and walks.
fn sync_level(st: &mut BorgState) {
    if st.stair_scum.level != st.borg.time.level {
        st.stair_scum = StairScumState { level: st.borg.time.level, ..StairScumState::default() };
    }
}

/// The dark flows (`borg_flow_dark`): of the `borg_temp_*` candidates, keep
/// those inside the radius and, of them, those nearest the staircase.
pub(crate) fn filter_dark(st: &mut BorgState, map: &Map) {
    let n = usize::try_from(st.flow.temp_n).unwrap_or(0);
    let mut kept: Vec<(i32, (i32, i32))> = Vec::new();
    for k in 0..n {
        let (y, x) = st.flow.temp_at(k);
        if admit(st, map, y, x) {
            kept.push((map.steps(st, y, x), (y, x)));
        }
    }
    let nearest = kept.iter().map(|k| k.0).min();
    st.flow.temp_n = 0;
    for (s, (y, x)) in kept {
        if Some(s) == nearest {
            st.flow.temp_push(y, x);
        }
    }
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
/// door; the message gives no grid).
pub(crate) fn heard_door(st: &mut BorgState) {
    if !active(st) {
        return;
    }
    sync_level(st);
    borg_note(st, "# Stair scum: a door burst open; back onto the stairs.");
    st.stair_scum.alarm = Some(engine::game_turn());
}

/// A known closed door is open and the Borg did not open it
/// (`borg_update.rs`, the `track_closed` scan): an alarm when it is inside
/// the radius.
pub(crate) fn door_opened_by_monster(st: &mut BorgState, y: i32, x: i32) {
    sync_level(st);
    if st.stair_scum.alarmed.contains(&(y, x)) {
        return;
    }
    let Some(map) = Map::of(st) else { return };
    if map.steps(st, y, x) > map.radius(st, y, x) {
        return;
    }
    st.stair_scum.alarmed.push((y, x));
    borg_note(st, &format!("# Stair scum: a monster opened the door at {y},{x}; back onto the stairs."));
    st.stair_scum.alarm = Some(engine::game_turn());
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
pub(crate) fn holds_on_stairs(st: &BorgState) -> bool {
    active(st) && on_stairs(st)
}

/// Stay still a turn (`,`, as the Recall wait does).
fn hold(st: &mut BorgState, why: &str) -> bool {
    borg_note(st, &format!("# Stair scum: {why}."));
    borg_keypress(st, keycode_t::from(b','));
    st.borg.in_shop = false;
    true
}

/// One step of the walk back onto the nearest known staircase.
fn step_to_stairs(st: &mut BorgState, why: &str) -> bool {
    crate::borg_flow::borg_flow_clear(st);
    for (y, x) in stair_grids(st) {
        crate::borg_flow::borg_flow_enqueue_grid(st, y, x);
    }
    crate::borg_flow::borg_flow_spread(st, 250, true, true, false, -1, false);
    let who = format!("stairs ({why})");
    let who = std::ffi::CString::new(who).unwrap_or_default();
    if !crate::borg_flow::borg_flow_commit(st, &who, GOAL_MISC) {
        return false;
    }
    crate::borg_flow::borg_flow_old(st, GOAL_MISC)
}

/// After caution (`borg_think_dungeon`): the alarm, the door peek and the
/// walk back inside the radius. Returns true when it acted.
pub(crate) fn think(st: &mut BorgState) -> bool {
    if !flag() {
        return false;
    }
    sync_level(st);
    let Some(map) = Map::of(st) else {
        st.stair_scum.returning = false;
        st.stair_scum.peek = None;
        st.stair_scum.alarm = None;
        return false;
    };
    let (y, x) = (st.borg.c.y, st.borg.c.x);
    let here = map.steps(st, y, x);

    // 3. A door heard or seen opening: onto the staircase, and wait there.
    if let Some(since) = st.stair_scum.alarm {
        if alarm_holds(since, engine::game_turn(), value(Param::stair_scum_alarm_turns)) {
            if here == 0 {
                return hold(st, "holding on the stairs after a door opened");
            }
            if step_to_stairs(st, "a door opened") {
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
                if here > 0 && step_to_stairs(st, "stepping back from a door") {
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

    // 2. Outside the radius: back onto the staircase (all the way).
    if here == 0 {
        st.stair_scum.returning = false;
        st.stair_scum.sight_allowed = 0;
        return false;
    }
    let sight = map.sight(st, y, x).max(st.stair_scum.sight_allowed);
    let r = radius(sight, map.ratio, map.hp, map.contact);
    if st.stair_scum.returning || here > r {
        if !st.stair_scum.returning {
            borg_note(st, &format!("# Stair scum: {here} steps from the stairs, radius {r}: returning."));
        }
        st.stair_scum.returning = step_to_stairs(st, "stair scum return");
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

/// 4. Nothing left inside the radius (after the exploration flows): take a
/// staircase to a new level.
pub(crate) fn leave(st: &mut BorgState) -> bool {
    if !flag() {
        return false;
    }
    if Map::of(st).is_none() {
        return false;
    }
    let depth = st.borg.tr(BI_CDEPTH);
    let up_ok = !engine::player_opt(OPT_birth_force_descend);
    let down_ok = !st.borg.goal.rising
        && crate::borg_prepared::prepared_c_for_decision(st, depth + 1).is_none();
    let feat = u32::from(st.cave.grid(st.borg.c.x, st.borg.c.y).feat);
    for up in leave_order(depth, up_ok, down_ok) {
        let (here, k) = if up { (FEAT_LESS, b'<') } else { (FEAT_MORE, b'>') };
        if feat == here {
            borg_note(st, &format!("# Stair scum: nothing left inside the radius; taking the stairs {}.", if up { "up" } else { "down" }));
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
            borg_note(st, &format!("# Stair scum: nothing left inside the radius; to the stairs {}.", if up { "up" } else { "down" }));
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
    match u32::from(st.cave.grid(st.borg.c.x, st.borg.c.y).feat) {
        FEAT_LESS if !engine::player_opt(OPT_birth_force_descend) => Some(true),
        FEAT_MORE
            if crate::borg_prepared::prepared_c_for_decision(st, st.borg.tr(BI_CDEPTH) + 1).is_none() =>
        {
            Some(false)
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The issue's two examples: a torch in a corridor at 12 HP, and a lit
    /// room of radius 8.
    #[test]
    fn the_radius_follows_sight_speed_and_hp() {
        assert_eq!(radius(2, 2, 12, 8), 3, "torch, corridor, 12 HP");
        assert_eq!(radius(8, 2, 12, 8), 9, "lit room of radius 8");
        assert_eq!(radius(2, 2, 7, 8), 2, "7 HP: no turn of contact");
        assert_eq!(radius(2, 3, 16, 8), 3, "monsters at three times the speed");
        assert_eq!(radius(0, 2, 0, 8), 0, "no light, no HP");
    }

    #[test]
    fn the_threshold_is_the_class_value_plus_the_race_delta() {
        assert!(below_threshold(24, 25));
        assert!(!below_threshold(25, 25));
        assert!(below_threshold(27, 25 + 3), "a hobbit's +3");
        assert!(!below_threshold(21, 25 - 5), "a half-troll's -5");
    }

    /// A 12 x 1 corridor from a staircase at (5, 5): the steps grow along it,
    /// and a wall breaks the walk.
    #[test]
    fn stair_distances_walk_known_grids() {
        let floor = |y: i32, x: i32| y == 5 && (5..=16).contains(&x) && x != 12;
        let d = stair_distances(&[(5, 5)], floor);
        let at = |y: i32, x: i32| d[(y * AUTO_MAX_X + x) as usize];
        assert_eq!(at(5, 5), 0);
        assert_eq!(at(5, 8), 3);
        assert_eq!(at(5, 11), 6);
        assert_eq!(at(5, 13), FAR, "beyond the wall");
        // Two staircases: the nearer counts.
        let d = stair_distances(&[(5, 5), (5, 11)], floor);
        assert_eq!(d[(5 * AUTO_MAX_X + 9) as usize], 2);
    }

    #[test]
    fn lit_extent_is_the_shortest_run() {
        // A lit room rows 10..=26, columns 20..=36: centre (18, 28) is 8 from
        // every wall, a grid near the north wall 1.
        let room = |y: i32, x: i32| (10..=26).contains(&y) && (20..=36).contains(&x);
        assert_eq!(lit_extent(18, 28, room), 8);
        assert_eq!(lit_extent(11, 28, room), 1);
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
