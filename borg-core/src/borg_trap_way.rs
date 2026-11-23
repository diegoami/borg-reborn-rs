//! #1067: a trap next to the arrival stairs cuts a weak Borg off.
//!
//! The Half-Elf Warrior 1868115537 (borg-zoo run 45070528012, track-p
//! `206cf4e7`, the same death on `7a466d73`; diegoami/borg-cemetery#138):
//! four steps from its up staircase the corridor's last square was a poison
//! pit. Below 60 HP every known trap is a wall to the flows
//! (`spread_blocked`, C: "Avoid Traps if low level -- unless brave"), and the
//! pit filled the only corridor back, so the staircase became unreachable
//! ("Bored but unable to flow to up stairs.", 24 times) and the leash went
//! off (`borg_flow_cost_stair` returns 0 for an unreachable staircase). It
//! explored on, set off a blinding gas trap and died 5 steps from the stairs.
//!
//! Three rules, each compiled and shipped off (RULES 14: hypotheses until
//! borg-zoo measures them):
//!
//! 1. `trap_recover`: below `trap_recover_max_hp` max HP, a trap that hurts
//!    (HP below the maximum, poison, a cut, blindness, confusion or a stun
//!    after it) stops exploration. The Borg walks back to the nearest known
//!    staircase, cures or rests there, and takes it at the first threat
//!    (danger above 0 on the staircase). It explores again at full HP with
//!    the status cured. Where no staircase can be reached it rests where it
//!    stands (danger 0, not hungry).
//! 2. `trap_way_back`: when every route to every known staircase crosses a
//!    known trap that the flows refuse, the next action reopens the way:
//!    `D` (disarm) from the adjacent grid when the disarm chance is at least
//!    `trap_disarm_min_percent`, or `W` (walk onto it) when the trap's kind is
//!    known from setting it off here and its worst effect cannot kill at the
//!    current HP (a blinding or confusion gas trap; a plain pit at more HP
//!    than its 2d6). Below that chance, hurt, it rests first and disarms at
//!    full HP. It never explores past the trap, except after
//!    `trap_disarm_max_tries` attempts on one trap (then play is as before).
//! 3. `trap_leave_level_count`: below `trap_recover_max_hp`, the n-th trap set
//!    off on a level, or the n-th within `trap_leave_radius` grids of where
//!    the Borg arrived, makes it leave by the nearest staircase once it is
//!    reachable, after it has recovered there ("the form of a cursed
//!    level", owner 2026-11-23).
//!
//! A trap is "set off" when the engine prints a trap's activation message
//! (`hit_trap`, trap.c:146-246 at 574b618c: `msg`, `msg_good`, `msg_bad`,
//! `msg_xtra` of lib/gamedata/trap.txt) or "You set off the %s!"
//! (`do_cmd_disarm_aux`, cmd-cave.c:842-845). The disarm chance is the
//! engine's: the trap's skill (`SKILL_DISARM_MAGIC` for a MAGICAL trap, else
//! `SKILL_DISARM_PHYS`; the lower of the two when the kind is not known)
//! minus `depth / 5`, at least 2 (cmd-cave.c:808-825).

use crate::borg_cave::{AUTO_MAX_X, AUTO_MAX_Y};
use crate::borg_danger::borg_danger;
use crate::borg_headers::borg_trait_h::*;
use crate::borg_io::{borg_keypress, borg_note, borg_queue_direction, KC_ENTER};
use crate::engine;
use crate::headers::list_options_h::OPT_birth_force_descend;
use crate::headers::list_terrain_h::{FEAT_LESS, FEAT_MORE};
use crate::headers::ui_event_h::keycode_t;
use crate::headers::z_type_h::loc;
use crate::params::{value as param, Param};
use crate::state::BorgState;
use crate::borg::BorgTime;
use crate::{borg_flow, borg_flow_misc, borg_flow_stairs};

/// `GOAL_MISC` (borg-flow.h)
const GOAL_MISC: i32 = 3;
/// `GOAL_RECOVER` (borg-flow.h)
const GOAL_RECOVER: i32 = 9;

/// What the Borg learned of a trap by setting it off.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Kind {
    /// "pit": 2d6 (trap.txt:103-115).
    Pit,
    /// "spiked pit": 2d6, half the time 2d6 more and a 4d6 cut (trap.txt:117-134).
    SpikedPit,
    /// "poison pit": 2d6, half the time a 4d6 cut and 8d6 of poison (trap.txt:136-155).
    PoisonPit,
    /// "You fall into a spiked pit!": a spiked or a poison pit.
    SpikedOrPoisonPit,
    /// "You set off the pit!": any of the three (the engine names the kind's
    /// `name`, trap.txt's first field, init.c:1549-1557).
    AnyPit,
    /// "blinding gas trap": blind 25+1d50, no damage (trap.txt:314-324).
    GasBlind,
    /// "confusion gas trap": confused 10+1d20, no damage (trap.txt:326-336).
    GasConfuse,
    /// "You set off the gas trap!": any of the four gas traps.
    AnyGas,
    /// A physical trap of another kind (darts, other gas, the trap door).
    OtherPhysical,
    /// A MAGICAL trap (runes, spots, alarms) or one whose skill is not known.
    Other,
}

impl Kind {
    /// The kind's `MAGICAL` flag (trap.txt), `None` when not known.
    fn magical(self) -> Option<bool> {
        match self {
            Kind::Other => None,
            _ => Some(false),
        }
    }

    /// The most HP the trap can cost when walked onto, counting a timed
    /// poison or cut as one HP a turn for its whole duration
    /// (game-world.c:513-538); `None`: not to be walked onto (its effect is
    /// not bounded by HP: a trap door, a teleport or summon rune, paralysis).
    fn worst_hp(self) -> Option<i32> {
        match self {
            Kind::Pit => Some(12),
            Kind::SpikedPit => Some(12 + 12 + 24),
            Kind::PoisonPit | Kind::SpikedOrPoisonPit | Kind::AnyPit => Some(12 + 24 + 48),
            Kind::GasBlind | Kind::GasConfuse => Some(0),
            Kind::AnyGas | Kind::OtherPhysical | Kind::Other => None,
        }
    }
}

/// The trap messages of lib/gamedata/trap.txt (`msg`, `msg-good`,
/// `msg-bad`, `msg-xtra`), each with the kind it names. A message another
/// source also prints ("You are enveloped in flames!", mon-blows.c) counts
/// only when the Borg then stands on a known trap (`ambiguous`).
const MESSAGES: &[(&[u8], Kind, bool)] = &[
    (b"You fall through a trap door!", Kind::OtherPhysical, false),
    (b"You float gently down to the next level.", Kind::OtherPhysical, false),
    (b"You fall into a pit!", Kind::Pit, false),
    (b"You float gently to the bottom of the pit.", Kind::Pit, false),
    (b"You fall into a spiked pit!", Kind::SpikedOrPoisonPit, false),
    (b"You float gently to the bottom of the pit, neatly avoiding the spikes.", Kind::SpikedOrPoisonPit, false),
    (b"You are impaled on poisonous spikes!", Kind::PoisonPit, false),
    (b"You are impaled!", Kind::SpikedPit, false),
    (b"A new foe appears from nowhere!", Kind::Other, false),
    (b"You are enveloped in a cloud of smoke!", Kind::Other, false),
    (b"A sense of dread washes over you...", Kind::Other, false),
    (b"You hear a dragon's call!", Kind::Other, false),
    (b"Demons appear!", Kind::Other, false),
    (b"You hit a teleport trap!", Kind::Other, false),
    (b"You are enveloped in flames!", Kind::Other, true),
    (b"You are splashed with acid!", Kind::Other, false),
    (b"A small dart barely misses you.", Kind::OtherPhysical, false),
    (b"A small dart hits you!", Kind::OtherPhysical, false),
    (b"You are surrounded by a black gas!", Kind::GasBlind, false),
    (b"You are surrounded by a gas of scintillating colours!", Kind::GasConfuse, false),
    (b"You are surrounded by a pungent green gas!", Kind::OtherPhysical, false),
    (b"You are surrounded by a strange white mist!", Kind::OtherPhysical, false),
    (b"A high-pitched whistle blares through the dungeon!", Kind::Other, false),
    (b"A horn blast sounds!", Kind::Other, false),
    (b"The floor explodes around you!", Kind::OtherPhysical, false),
    (b"The air seems to catch fire!", Kind::OtherPhysical, false),
    (b"You feel pressure on your mind.", Kind::Other, true),
    (b"Your mind is blasted by psionic energy!", Kind::Other, true),
    (b"You shake your head and are back to normal.", Kind::Other, true),
    (b"A blazing light descends on you!", Kind::Other, true),
    (b"Blades whirl around you, slicing your skin!", Kind::Other, true),
    (b"A rock falls on your head!", Kind::Other, true),
    (b"A section of the ceiling falls in behind you!", Kind::Other, true),
    (b"The trap explodes, blasting the area clear!", Kind::Other, true),
    (b"You feel yourself turning to stone!", Kind::Other, true),
    (b"You feel your energies sap away.", Kind::Other, true),
    (b"Everything starts to fade...", Kind::Other, true),
    (b"The ground shakes.", Kind::Other, true),
];

/// A trap's kind from its name in "You set off the %s!": the kind's
/// `name`, the first field of trap.txt's `name:` line ("pit" for all three
/// pits, "gas trap" for the four gas traps; init.c:1549-1557).
fn kind_of_name(name: &[u8]) -> Kind {
    match name {
        b"pit" => Kind::AnyPit,
        b"gas trap" => Kind::AnyGas,
        b"trap door" | b"dart trap" | b"explosion" => Kind::OtherPhysical,
        _ => Kind::Other,
    }
}

/// The kind a message reports and whether it is ambiguous, or `None` when
/// it is no trap message.
pub(crate) fn classify(msg: &[u8]) -> Option<(Kind, bool)> {
    if let Some(rest) = msg.strip_prefix(b"You set off the ") {
        let name = rest.strip_suffix(b"!").unwrap_or(rest);
        return Some((kind_of_name(name), false));
    }
    MESSAGES.iter().find(|(text, _, _)| msg == *text).map(|&(_, kind, ambiguous)| (kind, ambiguous))
}

/// A trap message's kind refines an earlier one of the same trap: the
/// "You are impaled ..." line tells a spiked pit from a poison pit.
/// A general name ("You set off the pit!") never replaces what the Borg
/// already knows of that trap.
fn refine(old: Kind, new: Kind) -> Kind {
    use Kind::*;
    match (old, new) {
        (SpikedPit | PoisonPit, SpikedOrPoisonPit | AnyPit) | (SpikedOrPoisonPit | Pit, AnyPit) => old,
        (GasBlind | GasConfuse | OtherPhysical, AnyGas) => old,
        _ => new,
    }
}

/// The engine's disarm chance (cmd-cave.c:808-825): the skill less
/// `depth / 5`, at least 2. The penalties for blindness, darkness,
/// confusion and hallucination are not applied: the rule waits them out.
pub(crate) fn disarm_chance(skill: i32, depth: i32) -> i32 {
    (skill - depth / 5).max(2)
}

/// Whether a trap of `kind` may be walked onto at `cur_hp`: its worst cost
/// leaves the Borg alive.
pub(crate) fn walk_safe(kind: Option<Kind>, cur_hp: i32) -> bool {
    kind.and_then(Kind::worst_hp).is_some_and(|worst| worst < cur_hp)
}

/// What the Borg does at a trap that cuts it off from every staircase.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Reopen {
    Disarm,
    Walk,
    /// Rest first (hurt, the chance low, not safe to walk).
    Rest,
}

/// The way-back verdict: disarm when the chance is high enough or the Borg
/// is whole (the owner: "I would have disarmed it at least"); walk onto it
/// when that cannot kill and the chance is low; otherwise rest first.
pub(crate) fn reopen(chance: i32, min_percent: i32, walk: bool, whole: bool) -> Reopen {
    if chance >= min_percent {
        Reopen::Disarm
    } else if walk {
        Reopen::Walk
    } else if whole {
        Reopen::Disarm
    } else {
        Reopen::Rest
    }
}

/// Rule 3's count: the `count`-th trap of the level, or of those near the
/// arrival grid.
pub(crate) fn trap_dense(count: i32, set_off: i32, near: i32) -> bool {
    count > 0 && (set_off >= count || near >= count)
}

#[derive(Default, Debug)]
pub struct TrapWayState {
    /// `st.borg.time.level` of the level this state is about.
    level: Option<BorgTime>,
    /// Where the Borg stood at its first think on the level.
    arrival: Option<loc>,
    /// The traps set off on this level (each grid once), how many, and how
    /// many within `trap_leave_radius` of `arrival`.
    counted: Vec<loc>,
    set_off: i32,
    set_off_near: i32,
    /// A trap message not yet placed: its kind, whether it is ambiguous,
    /// whether it came from a disarm, and `time.now` when it arrived.
    pending: Option<(Kind, bool, bool, BorgTime)>,
    /// The kinds learned on this level, by grid.
    kinds: Vec<(loc, Kind)>,
    /// Disarm and walk attempts on this level, by grid.
    tries: Vec<(loc, i32)>,
    /// The trap the last `D` or `W` acted on.
    target: Option<loc>,
    /// Rule 1 is on: back to the stairs until whole.
    recovering: bool,
    /// Rule 3 is on: leave the level.
    leaving: bool,
    /// Rule 2 committed a flow through the traps (`GOAL_MISC`).
    committed: bool,
}

fn any_on() -> bool {
    param(Param::trap_recover) != 0 || param(Param::trap_way_back) != 0 || param(Param::trap_leave_level_count) != 0
}

/// Read one game message (`borg_parse_aux`): a trap's activation.
pub(crate) fn note_message(st: &mut BorgState, msg: &[u8]) {
    if !any_on() {
        return;
    }
    let Some((kind, ambiguous)) = classify(msg) else { return };
    let now = st.borg.time.now;
    let disarm = msg.starts_with(b"You set off the ");
    st.trap_way.pending = Some(match st.trap_way.pending {
        // The lines of one trap arrive together.
        Some((old, old_ambiguous, old_disarm, when)) if when == now => {
            (refine(old, kind), old_ambiguous && ambiguous, old_disarm || disarm, when)
        }
        _ => (kind, ambiguous, disarm, now),
    });
}

fn whole(st: &BorgState) -> bool {
    let b = &st.borg;
    b.tr(BI_CURHP) >= b.tr(BI_MAXHP)
        && b.tr(BI_ISPOISONED) == 0
        && b.tr(BI_ISCUT) == 0
        && b.tr(BI_ISBLIND) == 0
        && b.tr(BI_ISCONFUSED) == 0
        && b.tr(BI_ISSTUN) == 0
}

fn weak(st: &BorgState) -> bool {
    st.borg.tr(BI_MAXHP) < param(Param::trap_recover_max_hp)
}

fn on_stairs(st: &BorgState) -> Option<u32> {
    let feat = u32::from(st.cave.grid(st.borg.c.x, st.borg.c.y).feat);
    (feat == FEAT_LESS || feat == FEAT_MORE).then_some(feat)
}

fn danger_here(st: &mut BorgState) -> i32 {
    let c = st.borg.c;
    borg_danger(st, c.y, c.x, 1, true, false)
}

fn sync_level(st: &mut BorgState) {
    if st.trap_way.level == Some(st.borg.time.level) {
        return;
    }
    // A goal this rule committed on the old level is the old level's.
    st.trap_way = TrapWayState {
        level: Some(st.borg.time.level),
        arrival: Some(st.borg.c),
        ..TrapWayState::default()
    };
}

/// Place the pending message: count it, remember the trap's kind, and
/// switch rules 1 and 3 on.
fn place_pending(st: &mut BorgState) {
    let Some((kind, ambiguous, disarm, _)) = st.trap_way.pending.take() else { return };
    let here = st.borg.c;
    let grid = if disarm { st.trap_way.target.unwrap_or(here) } else { here };
    if ambiguous && !st.cave.grid(grid.x, grid.y).trap {
        return;
    }
    let w = &mut st.trap_way;
    if !w.counted.contains(&grid) {
        w.counted.push(grid);
        w.set_off += 1;
        if w.arrival.is_some_and(|a| engine::distance(a, grid) <= param(Param::trap_leave_radius)) {
            w.set_off_near += 1;
        }
    }
    match w.kinds.iter_mut().find(|(g, _)| *g == grid) {
        Some((_, k)) => *k = refine(*k, kind),
        None => w.kinds.push((grid, kind)),
    }
    let (set_off, near) = (w.set_off, w.set_off_near);
    borg_note(st, &format!("# Trap: set off {kind:?} at ({},{}), {set_off} on this level, {near} near the arrival", grid.y, grid.x));
    if !weak(st) {
        return;
    }
    if param(Param::trap_recover) != 0 && !whole(st) && !st.trap_way.recovering {
        st.trap_way.recovering = true;
        borg_note(st, "# Trap: hurt by a trap, back to the stairs to recover");
    }
    if trap_dense(param(Param::trap_leave_level_count), set_off, near) && !st.trap_way.leaving {
        st.trap_way.leaving = true;
        borg_note(st, "# Trap: a trap-dense level, leaving");
    }
}

fn known_trap(st: &BorgState) -> bool {
    (1..AUTO_MAX_Y - 1).any(|y| {
        (1..AUTO_MAX_X - 1).any(|x| {
            let g = st.cave.grid(x, y);
            g.trap && !g.glyph
        })
    })
}

/// Spread from every known staircase; the Borg's flow cost.
fn stair_cost(st: &mut BorgState, through_traps: bool) -> u8 {
    borg_flow::borg_flow_clear(st);
    for i in 0..st.stairs.less.num.max(0) as usize {
        let (y, x) = (st.stairs.less.y[i], st.stairs.less.x[i]);
        borg_flow::borg_flow_enqueue_grid(st, y, x);
    }
    for i in 0..st.stairs.more.num.max(0) as usize {
        let (y, x) = (st.stairs.more.y[i], st.stairs.more.x[i]);
        borg_flow::borg_flow_enqueue_grid(st, y, x);
    }
    let was = std::mem::replace(&mut st.flow.through_traps, through_traps);
    borg_flow::borg_flow_spread_stairs(st, 250, false, true, false, -1, false);
    st.flow.through_traps = was;
    st.flow.cost_at(st.borg.c.y, st.borg.c.x)
}

/// The first known trap on the cheapest route of the spread flow from the
/// Borg's grid to a staircase.
fn first_trap(st: &BorgState) -> Option<loc> {
    let mut p = st.borg.c;
    for _ in 0..250 {
        let cost = st.flow.cost_at(p.y, p.x);
        if cost == 0 {
            return None;
        }
        let mut best: Option<(u8, loc)> = None;
        for d in 0..8 {
            let q = loc { x: p.x + i32::from(engine::ddx_ddd(d)), y: p.y + i32::from(engine::ddy_ddd(d)) };
            if !(1..AUTO_MAX_X - 1).contains(&q.x) || !(1..AUTO_MAX_Y - 1).contains(&q.y) {
                continue;
            }
            let c = st.flow.cost_at(q.y, q.x);
            if c < cost && best.is_none_or(|(b, _)| c < b) {
                best = Some((c, q));
            }
        }
        let (_, q) = best?;
        let g = st.cave.grid(q.x, q.y);
        if g.trap && !g.glyph {
            return Some(q);
        }
        p = q;
    }
    None
}

fn rest_here(st: &mut BorgState, why: &str) -> bool {
    if whole(st) || st.borg.tr(BI_ISHUNGRY) != 0 || st.borg.tr(BI_ISWEAK) != 0 || danger_here(st) > 0 {
        return false;
    }
    borg_note(st, &format!("# Trap: resting {why}"));
    borg_keypress(st, keycode_t::from(b'R'));
    borg_keypress(st, keycode_t::from(b'&'));
    borg_keypress(st, KC_ENTER);
    true
}

fn tries_at(st: &BorgState, t: loc) -> i32 {
    st.trap_way.tries.iter().find(|(g, _)| *g == t).map_or(0, |(_, n)| *n)
}

fn count_try(st: &mut BorgState, t: loc) {
    match st.trap_way.tries.iter_mut().find(|(g, _)| *g == t) {
        Some((_, n)) => *n += 1,
        None => st.trap_way.tries.push((t, 1)),
    }
}

/// Rule 2: the Borg is cut off from every staircase by the trap `t`.
fn reopen_way(st: &mut BorgState, t: loc) -> bool {
    let b = &st.borg;
    if b.tr(BI_ISBLIND) != 0 || b.tr(BI_ISCONFUSED) != 0 || b.tr(BI_ISIMAGE) != 0 || b.tr(BI_LIGHT) == 0 {
        return rest_here(st, "before reopening the way back");
    }
    if tries_at(st, t) >= param(Param::trap_disarm_max_tries) {
        return false;
    }
    let kind = st.trap_way.kinds.iter().find(|(g, _)| *g == t).map(|(_, k)| *k);
    let skill = match kind.and_then(Kind::magical) {
        Some(true) => b.tr(BI_DISM),
        Some(false) => b.tr(BI_DISP),
        None => b.tr(BI_DISP).min(b.tr(BI_DISM)),
    };
    let chance = disarm_chance(skill, b.tr(BI_CDEPTH));
    let walk = walk_safe(kind, b.tr(BI_CURHP));
    let verdict = reopen(chance, param(Param::trap_disarm_min_percent), walk, whole(st));
    if verdict == Reopen::Rest {
        return rest_here(st, &format!("before disarming the trap at ({},{}) (chance {chance}%)", t.y, t.x));
    }
    let c = st.borg.c;
    if engine::distance(c, t) != 1 {
        // Walk to it along the flow through the traps.
        if borg_flow::flow_commit(st, Some(b"the trap on the way back"), GOAL_MISC) {
            st.trap_way.committed = true;
            if borg_flow::borg_flow_old(st, GOAL_MISC) {
                return true;
            }
        }
        return false;
    }
    let dir = borg_flow_misc::borg_goto_dir(st, c.y, c.x, t.y, t.x);
    if dir == 5 {
        return false;
    }
    count_try(st, t);
    st.trap_way.target = Some(t);
    let key = keycode_t::from(b'0' + dir as u8);
    if verdict == Reopen::Walk {
        borg_note(st, &format!("# Trap: walking onto the {kind:?} that cuts the way back (disarm {chance}%)"));
        borg_keypress(st, keycode_t::from(b'W'));
    } else {
        borg_note(st, &format!("# Trap: disarming the trap that cuts the way back ({chance}%)"));
        borg_keypress(st, keycode_t::from(b'D'));
        // As borg_play_step: pretend it goes; the map says otherwise next think.
        st.cave.grid_mut(t.x, t.y).trap = false;
    }
    borg_queue_direction(st, key);
    true
}

/// Take the staircase the Borg stands on, if it may.
fn take_stairs(st: &mut BorgState, feat: u32, why: &str) -> bool {
    let key = if feat == FEAT_LESS {
        if engine::player_opt(OPT_birth_force_descend) {
            return false;
        }
        b'<'
    } else {
        if st.borg.goal.rising || borg_flow_stairs::more_refused(st, false) {
            return false;
        }
        b'>'
    };
    borg_note(st, &format!("# Trap: taking the stairs ({why})"));
    borg_keypress(st, keycode_t::from(key));
    true
}

/// The dungeon think's hook, after the flight, the caution and the light,
/// before the recovery flows: true when it acted.
pub(crate) fn think(st: &mut BorgState) -> bool {
    if !any_on() || st.borg.tr(BI_CDEPTH) == 0 {
        return false;
    }
    sync_level(st);
    place_pending(st);
    if st.trap_way.recovering && whole(st) {
        st.trap_way.recovering = false;
        borg_note(st, "# Trap: recovered, exploring again");
    }

    let stairs = st.stairs.less.num > 0 || st.stairs.more.num > 0;
    if param(Param::trap_way_back) != 0 && stairs && known_trap(st) {
        if stair_cost(st, false) >= 250 && stair_cost(st, true) < 250 {
            if let Some(t) = first_trap(st) {
                return reopen_way(st, t);
            }
        } else if st.trap_way.committed {
            st.trap_way.committed = false;
            if i32::from(st.borg.goal.type_0) == GOAL_MISC {
                st.borg.goal.type_0 = 0;
            }
        }
    }

    let (recovering, leaving) = (st.trap_way.recovering, st.trap_way.leaving);
    if !recovering && !leaving {
        return false;
    }
    if let Some(feat) = on_stairs(st) {
        if danger_here(st) > 0 && take_stairs(st, feat, "a threat while recovering from a trap") {
            return true;
        }
        if recovering {
            return crate::borg_recover::borg_recover(st) || rest_here(st, "on the stairs after a trap");
        }
        return take_stairs(st, feat, "a trap-dense level")
            || borg_flow_stairs::borg_flow_stair_less(st, GOAL_RECOVER, false);
    }
    if borg_flow_stairs::borg_flow_stair_both(st, GOAL_RECOVER, false) {
        borg_note(st, "# Trap: back to the stairs");
        return true;
    }
    recovering && rest_here(st, "where it stands (no staircase in reach)")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn messages_name_their_traps() {
        assert_eq!(classify(b"You fall into a spiked pit!"), Some((Kind::SpikedOrPoisonPit, false)));
        assert_eq!(classify(b"You are impaled on poisonous spikes!"), Some((Kind::PoisonPit, false)));
        assert_eq!(classify(b"You are surrounded by a black gas!"), Some((Kind::GasBlind, false)));
        assert_eq!(classify(b"You set off the pit!"), Some((Kind::AnyPit, false)));
        assert_eq!(classify(b"You set off the gas trap!"), Some((Kind::AnyGas, false)));
        assert_eq!(classify(b"You set off the dart trap!"), Some((Kind::OtherPhysical, false)));
        assert_eq!(classify(b"You set off the strange rune!"), Some((Kind::Other, false)));
        assert_eq!(classify(b"You are enveloped in flames!"), Some((Kind::Other, true)));
        assert_eq!(classify(b"You failed to disarm the pit."), None);
        assert_eq!(classify(b"You have disarmed the pit."), None);
        assert_eq!(refine(Kind::SpikedOrPoisonPit, Kind::PoisonPit), Kind::PoisonPit);
        assert_eq!(refine(Kind::PoisonPit, Kind::SpikedOrPoisonPit), Kind::PoisonPit);
        assert_eq!(refine(Kind::Pit, Kind::AnyPit), Kind::Pit);
        assert_eq!(refine(Kind::AnyPit, Kind::PoisonPit), Kind::PoisonPit);
        assert_eq!(refine(Kind::GasConfuse, Kind::AnyGas), Kind::GasConfuse);
    }

    /// Every activation message of the engine's trap.txt is read.
    #[test]
    fn every_trap_txt_message_is_listed() {
        let text = include_str!("../../engine/angband/lib/gamedata/trap.txt");
        for line in text.lines() {
            let Some((tag, msg)) = line.split_once(':') else { continue };
            if matches!(tag, "msg" | "msg-good" | "msg-bad" | "msg-xtra") {
                assert!(classify(msg.as_bytes()).is_some(), "trap.txt message not read: {msg}");
            }
        }
    }

    #[test]
    fn disarm_chance_is_the_engines() {
        assert_eq!(disarm_chance(40, 2), 40);
        assert_eq!(disarm_chance(40, 10), 38);
        assert_eq!(disarm_chance(1, 30), 2);
    }

    #[test]
    fn walk_onto_only_what_cannot_kill() {
        assert!(walk_safe(Some(Kind::GasBlind), 1));
        assert!(walk_safe(Some(Kind::Pit), 13));
        assert!(!walk_safe(Some(Kind::Pit), 12));
        assert!(!walk_safe(Some(Kind::SpikedOrPoisonPit), 31));
        assert!(!walk_safe(Some(Kind::AnyPit), 31));
        assert!(!walk_safe(Some(Kind::AnyGas), 500));
        assert!(!walk_safe(Some(Kind::OtherPhysical), 500));
        assert!(!walk_safe(None, 500));
    }

    #[test]
    fn reopen_disarms_walks_or_rests() {
        assert_eq!(reopen(50, 50, false, false), Reopen::Disarm);
        assert_eq!(reopen(30, 50, true, false), Reopen::Walk);
        assert_eq!(reopen(30, 50, false, false), Reopen::Rest);
        assert_eq!(reopen(30, 50, false, true), Reopen::Disarm);
    }

    #[test]
    fn trap_dense_counts_the_level_or_the_arrival() {
        assert!(!trap_dense(0, 9, 9));
        assert!(!trap_dense(2, 1, 1));
        assert!(trap_dense(2, 2, 0));
        assert!(trap_dense(2, 1, 2));
    }
}
