//! Port of `borg-flow-kill.c`: flow (move) toward a kill (monster), and
//! the monster tracking list (`borg_kills`).
//!
//! Copyright (c) 1997 Ben Harrison, James E. Wilson, Robert A. Koeneke
//! Copyright (c) 2007-9 Andi Sidwell, Chris Carr, Ed Graham, Erik Osheim
//!
//! This work is free software; you can redistribute it and/or modify it
//! under the terms of either:
//!
//! a) the GNU General Public License as published by the Free Software
//!    Foundation, version 2, or
//!
//! b) the "Angband License":
//!    This software may be copied and distributed for educational, research,
//!    and not for profit purposes provided that this copyright and statement
//!    are included in all such copies.  Other copyrights may also apply.
//!
//! Integer semantics follow C: `int` arithmetic, stores into the narrow
//! fields of `borg_kill` truncate (`as`). C bugs kept are listed in
//! `docs/c-bugs.md` (section 18).

use std::ffi::CStr;

use crate::borg::BorgTime;
use crate::borg_cave::{AUTO_MAX_X, AUTO_MAX_Y, DUNGEON_HGT, DUNGEON_WID};
use crate::borg_cave_util::{
    borg_cave_floor_bold, borg_cave_floor_grid, borg_panel_hgt, borg_panel_wid,
};
use crate::borg_cave_light::BORG_LIGHT;
use crate::borg_cave_view::BORG_VIEW;
use crate::borg_danger::{borg_danger, borg_danger_one_kill};
use crate::borg_fight_attack::{borg_attack_aux_launch, borg_attack_aux_spell_bolt};
use crate::borg_io::borg_note_bytes;
use crate::borg_magic::{borg_primarily_caster, borg_spell_okay};
use crate::borg_projection::{borg_distance, borg_los, borg_projectable};
use crate::borg_update::Detect;
use crate::borg_util::{borg_sort, TextWhat};
use crate::cctype::c_toupper;
use crate::engine::{self, MonsterView};
use crate::borg_headers::borg_fight_attack_h::{BORG_ATTACK_MISSILE, BORG_ATTACK_NETHER};
use crate::borg_headers::borg_h::{BORG_MONEY_SCUM_AMOUNT, BORG_VERBOSE};
use crate::borg_headers::borg_magic_h::{
    MAGIC_MISSILE, NETHER_BOLT, ORB_OF_DRAINING, SHATTER_STONE, TURN_STONE_TO_MUD,
};
use crate::borg_headers::borg_trait_h::{
    BI_AMISSILES, BI_APHASE, BI_CDEPTH, BI_CLASS, BI_CLEVEL, BI_CRSFEAR, BI_CURSP, BI_ESP,
    BI_FOOD, BI_INFRA, BI_ISAFRAID, BI_ISCONFUSED, BI_ISHUNGRY, BI_ISIMAGE, BI_ISWEAK, BI_LIGHT,
    BI_MAXCLEVEL, BI_RPOIS, BI_SINV, BI_SPEED, BI_SRBLIND, BI_SRCONF, BI_SRDARK, BI_SRKAOS,
    BI_SRLITE, BI_SRNTHR,
};
use crate::headers::list_mon_race_flags_h::{
    RF_ATTR_CLEAR, RF_ATTR_MULTI, RF_CHAR_CLEAR, RF_COLD_BLOOD, RF_EMPTY_MIND, RF_EVIL,
    RF_GROUP_AI, RF_INVISIBLE, RF_KILL_WALL, RF_MULTIPLY, RF_NEVER_MOVE, RF_PASS_WALL,
    RF_QUESTOR, RF_UNIQUE, RF_WEIRD_MIND,
};
use crate::headers::list_mon_spells_h::{
    RSF_BR_CHAO, RSF_MAX, RSF_NONE, RSF_S_AINU, RSF_S_ANIMAL, RSF_S_DEMON, RSF_S_DRAGON,
    RSF_S_HI_DEMON, RSF_S_HI_DRAGON, RSF_S_HI_UNDEAD, RSF_S_HOUND, RSF_S_HYDRA, RSF_S_KIN,
    RSF_S_MONSTER, RSF_S_MONSTERS, RSF_S_SPIDER, RSF_S_UNDEAD, RSF_S_UNIQUE, RSF_S_WRAITH,
};
use crate::headers::list_mon_timed_h::{MON_TMD_CONF, MON_TMD_FEAR, MON_TMD_STUN};
use crate::headers::list_terrain_h::{
    FEAT_FLOOR, FEAT_GRANITE, FEAT_MAGMA, FEAT_MORE, FEAT_NONE, FEAT_PERM, FEAT_QUARTZ_K,
};
use crate::headers::z_type_h::loc;
use crate::state::BorgState;
use crate::{borg_flow, borg_flow_misc};

/// `GOAL_KILL`, `GOAL_TAKE`, `GOAL_DIGGING` (borg-flow.h)
const GOAL_KILL: i16 = 1;
const GOAL_TAKE: i16 = 2;
const GOAL_DIGGING: i32 = 10;

/// `CLASS_*` (list-class.h order)
const CLASS_MAGE: i32 = 1;
const CLASS_PRIEST: i32 = 3;
const CLASS_NECROMANCER: i32 = 4;

/// `BORG_GLOW`, `BORG_OKAY` (borg-cave.h grid info bits)
const BORG_GLOW: u16 = 0x02;
const BORG_OKAY: u16 = 0x08;

/// `MON_TMD_SLEEP`
const MON_TMD_SLEEP: usize = 0;

/// `SHRT_MAX`
const SHRT_MAX: i16 = i16::MAX;

/// Number of tracked monster slots (`borg_kills` is allocated with 256).
pub const KILLS_MAX: usize = 256;

/// One tracked monster (`borg_kill`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BorgKill {
    /// Race index
    pub r_idx: u16,
    /// Verified race
    pub known: bool,
    /// Probably awake
    pub awake: bool,
    /// Probably confused
    pub confused: bool,
    /// Probably afraid
    pub afraid: bool,
    /// Probably quivering
    pub quiver: bool,
    pub stunned: bool,
    /// Probably poisoned
    pub poisoned: bool,
    /// Assigned motion
    pub seen: bool,
    /// Assigned message
    pub used: bool,
    /// Location
    pub pos: loc,
    /// Old location
    pub ox: u8,
    pub oy: u8,
    /// Estimated speed
    pub speed: u8,
    /// Estimates moves
    pub moves: u8,
    /// qty of ranged attacks
    pub ranged_attack: u8,
    /// spell flag for monster spells
    pub spell: [u8; RSF_MAX as usize],
    /// Estimated hit-points
    pub power: i16,
    /// Percent wounded
    pub injury: i16,
    /// Estimated something
    pub other: i16,
    /// Monsters Level
    pub level: i16,
    /// Monster race spell flags preloaded
    pub spell_flags: [u32; 85],
    /// When last seen
    pub when: BorgTime,
    /// Game's index
    pub m_idx: i16,
}

impl BorgKill {
    /// An all-zero record (`memset(kill, 0, sizeof(borg_kill))`).
    pub const BLANK: BorgKill = BorgKill {
        r_idx: 0,
        known: false,
        awake: false,
        confused: false,
        afraid: false,
        quiver: false,
        stunned: false,
        poisoned: false,
        seen: false,
        used: false,
        pos: loc { x: 0, y: 0 },
        ox: 0,
        oy: 0,
        speed: 0,
        moves: 0,
        ranged_attack: 0,
        spell: [0; RSF_MAX as usize],
        power: 0,
        injury: 0,
        other: 0,
        level: 0,
        spell_flags: [0; 85],
        when: 0,
        m_idx: 0,
    };
}

impl Default for BorgKill {
    fn default() -> Self {
        Self::BLANK
    }
}

/// borg-flow-kill.c globals.
#[derive(Clone)]
pub struct KillState {
    /// `borg_kills`: the monster list (256 entries, empty = NULL). Only
    /// `borg_init_flow_kill` / `borg_free_flow_kill` (re)allocate it.
    pub kills: Vec<BorgKill>,
    /// `borg_kills_cnt`
    pub cnt: i16,
    /// `borg_kills_nxt`
    pub nxt: i16,
    /// `borg_race_count`: racial appearances per level (`r_max` entries)
    pub race_count: Vec<i16>,
    /// `borg_race_death`: racial kills (`r_max` entries)
    pub race_death: Vec<i16>,
    /// `borg_unique_text` / `borg_unique_what` (sorted; `borg_unique_size`
    /// is the length). Each name is the C string without its NUL.
    pub unique_text: Vec<Vec<u8>>,
    pub unique_what: Vec<u32>,
    /// `borg_normal_text` / `borg_normal_what` (sorted, plurals included)
    pub normal_text: Vec<Vec<u8>>,
    pub normal_what: Vec<u32>,
    /// `borg_nasties_num`: current size of the list
    pub nasties_num: u8,
    /// `borg_nasties_count`
    pub nasties_count: [u8; 7],
    /// `borg_nasties`: order of nastiness (hounds < demons < wyrms)
    pub nasties: [u8; 7],
    /// `borg_nasties_limit`
    pub nasties_limit: [u8; 7],
}

impl Default for KillState {
    fn default() -> Self {
        KillState {
            kills: Vec::new(),
            cnt: 0,
            nxt: 0,
            race_count: Vec::new(),
            race_death: Vec::new(),
            unique_text: Vec::new(),
            unique_what: Vec::new(),
            normal_text: Vec::new(),
            normal_what: Vec::new(),
            nasties_num: 7,
            nasties_count: [0; 7],
            nasties: *b"ZAVULWD",
            nasties_limit: [20, 20, 10, 10, 10, 10, 10],
        }
    }
}

// ---------------------------------------------------------------------------
// Small helpers

/// `borg_timer(t)`: game turns since `t`.
fn borg_timer(st: &BorgState, t: BorgTime) -> i32 {
    st.borg.time.now.wrapping_sub(t)
}

/// `rf_has(r_info[r_idx].flags, flag)`
fn rf_has(r_idx: impl Into<u32>, flag: u32) -> bool {
    engine::race_flag_has(r_idx.into() as usize, flag)
}

/// `rsf_has(r_info[r_idx].spell_flags, flag)`
fn rsf_has(r_idx: impl Into<u32>, flag: u32) -> bool {
    let r = engine::race(r_idx.into() as usize);
    engine::flag_has(&r.spell_flags, flag as i32)
}

/// `borg_note(...)` from byte pieces (race names are raw bytes).
fn note(st: &mut BorgState, parts: &[&[u8]]) {
    let msg: Vec<u8> = parts.concat();
    borg_note_bytes(st, &msg);
}

/// `prefix(s, t)` (z-util.c)
fn prefix(s: &[u8], t: &[u8]) -> bool {
    s.starts_with(t)
}

/// `suffix(s, t)` (z-util.c)
fn suffix(s: &[u8], t: &[u8]) -> bool {
    s.ends_with(t)
}

/// `prefix_i(s, t)` (z-util.c): `t` is a prefix of `s`, ignoring case.
fn prefix_i(s: &[u8], t: &[u8]) -> bool {
    for (k, &tc) in t.iter().enumerate() {
        let sc = s.get(k).copied().unwrap_or(0);
        if c_toupper(tc) != c_toupper(sc) {
            return false;
        }
    }
    true
}

/// `my_strnicmp(a, b, n) == 0` (z-util.c)
fn strnieq(a: &[u8], b: &[u8], n: usize) -> bool {
    for k in 0..n {
        let z1 = c_toupper(a.get(k).copied().unwrap_or(0));
        let z2 = c_toupper(b.get(k).copied().unwrap_or(0));
        if z1 != z2 {
            return false;
        }
        if z1 == 0 {
            return true;
        }
    }
    true
}

/// `my_stristr(string, pattern) != NULL` (z-util.c), for a non-empty
/// pattern.
fn stristr(string: &[u8], pattern: &[u8]) -> bool {
    let up = |c: u8| c_toupper(c);
    (0..string.len()).any(|start| {
        string.len() - start >= pattern.len()
            && string[start..start + pattern.len()]
                .iter()
                .zip(pattern)
                .all(|(&a, &b)| up(a) == up(b))
    })
}

/// The Morgoth panel offset for one axis (`j` in the update functions).
fn morgoth_panel(pos: i32, panel: i32, dungeon: i32, screen: i32) -> i32 {
    let mut j = ((pos - panel / 2) / panel) * panel;
    if j < 0 {
        j = 0;
    }
    if j > dungeon - screen {
        j = dungeon - screen;
    }
    j
}

/// `SCREEN_HGT`, `SCREEN_WID` (ui-term.h)
fn screen_hgt_wid() -> (i32, i32) {
    let t = engine::term_geometry();
    (
        (t.hgt - t.row_top_map - t.row_bottom_map) / t.tile_height,
        (t.wid - t.col_map - 1) / t.tile_width,
    )
}

/// The race name to show (`borg_race_name`): adds safeguards against
/// player ghosts or dead monsters.
pub fn borg_race_name(r_idx: i32) -> &'static CStr {
    // Paranoia
    if r_idx == 0 {
        return c"dead monster";
    }

    // "player ghosts"
    if r_idx >= i32::from(engine::z_info().r_max) - 1 {
        return c"player ghost";
    }

    engine::race_name_cstr(r_idx as usize)
}

/// `borg_race_name(r_idx)` as bytes.
fn race_name(r_idx: impl Into<i32>) -> &'static [u8] {
    borg_race_name(r_idx.into()).to_bytes()
}

// ---------------------------------------------------------------------------
// Updating monster records

/// The parts shared by `borg_update_kill_new` and `borg_update_kill_old`
/// after the hit points.
fn set_timed_flags(kill: &mut BorgKill, m: &MonsterView) {
    // Is it sleeping
    kill.awake = m.m_timed[MON_TMD_SLEEP] == 0;

    // Is it afraid
    kill.afraid = m.m_timed[MON_TMD_FEAR as usize] != 0;

    // Is it confused
    kill.confused = m.m_timed[MON_TMD_CONF as usize] != 0;

    // Is it stunned
    kill.stunned = m.m_timed[MON_TMD_STUN as usize] != 0;
}

/// Cheat the health (`pct`, `power`, `injury`) from the game monster.
fn set_health(kill: &mut BorgKill, m: &MonsterView) {
    let pct: i32 = if m.maxhp != 0 {
        // Cheat the "percent" of health (a C `long` division)
        let max = if m.maxhp > 1 { i64::from(m.maxhp) } else { 1 };
        (100i64 * i64::from(m.hp) / max) as i32
    } else {
        100
    };

    // Compute estimated HP based on number of * in monster health bar
    kill.power = (i32::from(m.maxhp) * pct / 100) as i16;
    kill.injury = (100 - pct) as i16;
}

/// Preload the spells from the race into this individual monster.
fn load_spells(kill: &mut BorgKill) {
    let r = engine::race(usize::from(kill.r_idx));
    kill.spell_flags[0] = u32::from(r.spell_flags[0]);
    kill.spell_flags[1] = u32::from(r.spell_flags[1]);
    kill.spell_flags[2] = u32::from(r.spell_flags[2]);

    // Extract the spells
    let mut num = 0usize;
    for k in (RSF_NONE + 1)..RSF_MAX {
        if engine::flag_has(&r.spell_flags, k as i32) {
            kill.spell[num] = k as u8;
            num += 1;
        }
    }

    // Store the number of ranged attacks
    kill.ranged_attack = num as u8;
}

/// Remember Morgoth's panel, and force the monster's grid to floor (or to
/// granite for wall passers). The tail of both update functions.
fn update_kill_tail(st: &mut BorgState, i: usize) {
    let kill = st.kills.kills[i];

    // We want to remember Morgy's panel
    if u32::from(kill.r_idx) == st.borg.id.morgoth {
        let (screen_hgt, screen_wid) = screen_hgt_wid();
        st.borg.mon.morgoth_panel.y =
            morgoth_panel(kill.pos.y, borg_panel_hgt(), DUNGEON_HGT, screen_hgt);
        st.borg.mon.morgoth_panel.x =
            morgoth_panel(kill.pos.x, borg_panel_wid(), DUNGEON_WID, screen_wid);
    }

    // Force the monster to be sitting on a floor
    // grid unless that monster can pass through walls
    let pass_wall = rf_has(kill.r_idx, RF_PASS_WALL);
    let g = st.cave.grid_mut(kill.pos.x, kill.pos.y);
    if !pass_wall {
        g.feat = FEAT_FLOOR as u8;
    }

    // Force the ghostly monster to be in a wall
    // grid until the grid is proven to be something else
    if u32::from(g.feat) != FEAT_FLOOR && pass_wall {
        g.feat = FEAT_GRANITE as u8;
    }
}

/// The game monster on the kill's grid (`square_monster(cave, kill->pos)`).
/// C dereferences it without a check, so a missing monster is fatal.
fn monster_at(pos: loc) -> MonsterView {
    engine::square_monster(pos.x, pos.y).expect("no monster under a tracked kill (C NULL dereference)")
}

/// Update a "new" monster (`borg_update_kill_new`)
fn borg_update_kill_new(st: &mut BorgState, i: usize) {
    let mut kill = st.kills.kills[i];

    let m = engine::cave_monster(i32::from(kill.m_idx));
    let r_idx = usize::from(kill.r_idx);

    // Extract the monster speed
    kill.speed = m.mspeed;

    // Extract max hitpoints (a cheat, see the C comment: the Borg reads
    // the real hit points instead of the health bar)
    set_health(&mut kill, &m);

    // Extract the Level
    kill.level = engine::race(r_idx).level as i16;

    // Some monsters never move
    if rf_has(kill.r_idx, RF_NEVER_MOVE) {
        kill.awake = true;
    }

    // Cheat in the game's index of the monster. Used in tracking monsters
    kill.m_idx = monster_at(kill.pos).midx as i16;

    set_timed_flags(&mut kill, &m);

    load_spells(&mut kill);

    st.kills.kills[i] = kill;
    update_kill_tail(st, i);
}

/// Update a "old" monster (`borg_update_kill_old`)
///
/// We round the player speed down, and the monster speed up,
/// and we assume maximum racial speed for each monster.
fn borg_update_kill_old(st: &mut BorgState, i: usize) {
    let mut kill = st.kills.kills[i];

    let m = monster_at(kill.pos);

    // Extract actual Hitpoints, this is a cheat.
    set_health(&mut kill, &m);

    set_timed_flags(&mut kill, &m);

    // Cheat in the game's index of the monster. Used in tracking monsters
    kill.m_idx = monster_at(kill.pos).midx as i16;

    // Extract the monster speed
    kill.speed = m.mspeed;

    // Player energy per game turn
    let mut e = i32::from(engine::extract_energy(st.borg.tr(BI_SPEED) as usize));

    // Game turns per player move
    let t = (100 + (e - 1)) / e;

    // Monster energy per game turn
    e = i32::from(engine::extract_energy(usize::from(kill.speed)));

    // Monster moves (times ten)
    kill.moves = (t * e / 10) as u8;

    load_spells(&mut kill);

    st.kills.kills[i] = kill;

    // Special check on uniques, sometimes the death
    // is not caught so he thinks they are still running
    // around (C bug 24: indexed by the kill, not the race)
    let r = engine::race(usize::from(kill.r_idx));
    if rf_has(kill.r_idx, RF_UNIQUE) && r.max_num == 0 {
        st.kills.race_death[i] = 1;
    }

    update_kill_tail(st, i);
}

/// Delete an old "kill" record (`borg_delete_kill`)
pub fn borg_delete_kill(st: &mut BorgState, i: i32) {
    let i = i as usize;
    let kill = st.kills.kills[i];

    // Paranoia -- Already wiped
    if kill.r_idx == 0 {
        return;
    }

    // Note
    note(
        st,
        &[
            b"# Forgetting a monster '",
            race_name(kill.r_idx),
            format!("' at ({},{})", kill.pos.y, kill.pos.x).as_bytes(),
        ],
    );

    // Clear goals if I am flowing to this monster.
    let (fy, fx) = st.flow.first();
    if st.borg.goal.type_0 == GOAL_KILL && fy == kill.pos.y && fx == kill.pos.x {
        st.borg.goal.type_0 = 0;
    }

    // Update the grids
    st.cave.grid_mut(kill.pos.x, kill.pos.y).kill = 0;

    // save a time stamp of when the last multiplier was killed
    if rf_has(kill.r_idx, RF_MULTIPLY) {
        st.borg.time.last_kill_mult = st.borg.time.now;
    }

    // Kill the monster
    st.kills.kills[i] = BorgKill::BLANK;

    // One less monster
    st.kills.cnt = st.kills.cnt.wrapping_sub(1);

    // Recalculate danger
    st.danger.wipe = true;
}

/// Force sleep onto a "kill" record (`borg_sleep_kill`)
pub fn borg_sleep_kill(st: &mut BorgState, i: i32) {
    let i = i as usize;
    let kill = st.kills.kills[i];

    // Paranoia -- Already wiped
    if kill.r_idx == 0 {
        return;
    }

    // Note
    note(
        st,
        &[
            b"# Noting sleep on a monster '",
            race_name(kill.r_idx),
            format!("' at ({},{})", kill.pos.y, kill.pos.x).as_bytes(),
        ],
    );

    // note sleep
    st.kills.kills[i].awake = false;

    // Wipe flow goals
    st.borg.goal.type_0 = 0;

    // Recalculate danger
    st.danger.wipe = true;
}

/// Determine if a monster should be "viewable" (`borg_follow_kill_aux`)
fn borg_follow_kill_aux(st: &BorgState, i: usize, y: i32, x: i32) -> bool {
    let r_idx = st.kills.kills[i].r_idx;

    // Distance to player
    let d = engine::distance(st.borg.c, loc { x, y });

    // Too far away
    if d > i32::from(engine::z_info().max_sight) {
        return false;
    }

    // Access the grid
    let ag = st.cave.grid(x, y);

    // Not on-screen
    if ag.info & BORG_OKAY == 0 {
        return false;
    }

    // Line of sight
    if ag.info & BORG_VIEW != 0 {
        // Use "illumination"
        if ag.info & (BORG_LIGHT | BORG_GLOW) != 0 {
            // We can see invisible
            if st.borg.tr(BI_SINV) != 0 || st.borg.see_inv != 0 {
                return true;
            }

            // Monster is not invisible
            if !rf_has(r_idx, RF_INVISIBLE) {
                return true;
            }
        }

        // Use "infravision"
        if d <= st.borg.tr(BI_INFRA) {
            // Infravision works on "warm" creatures
            if !rf_has(r_idx, RF_COLD_BLOOD) {
                return true;
            }
        }
    }

    // Telepathy requires "telepathy"
    if st.borg.tr(BI_ESP) != 0 {
        // Telepathy fails on "strange" monsters
        if rf_has(r_idx, RF_EMPTY_MIND) {
            return false;
        }
        if rf_has(r_idx, RF_WEIRD_MIND) {
            return false;
        }

        // Success
        return true;
    }

    // Nope
    false
}

/// Clear the goal after a tracked monster moved: kill goals aimed at its
/// new grid (without ESP), or any munchkin take goal.
fn clear_goal_after_move(st: &mut BorgState, pos: loc) {
    let (fy, fx) = st.flow.first();
    if (st.borg.tr(BI_ESP) == 0
        && st.borg.goal.type_0 == GOAL_KILL
        && (fy == pos.y && fx == pos.x))
        || (st.borg.goal.type_0 == GOAL_TAKE && st.borg.munchkin_mode)
    {
        st.borg.goal.type_0 = 0;
    }
}

/// Attempt to "follow" a missing monster (`borg_follow_kill`)
///
/// This routine is not called when the player is blind or hallucinating.
///
/// Currently this function is a total hack, but handles the case of only
/// one possible location (taking it), two adjacent possible locations
/// (taking the diagonal one), and three adjacent locations with the
/// central one being a diagonal (taking the diagonal one).
///
/// C bug 24: every path that gets past the checks deletes the monster
/// (the "did not really move" test compares the unchanged position), so
/// the tail that moves it is dead code. Kept.
pub fn borg_follow_kill(st: &mut BorgState, i: i32) {
    let iu = i as usize;
    let kill = st.kills.kills[iu];

    // Paranoia
    if kill.r_idx == 0 {
        return;
    }

    // Old location
    let ox = kill.pos.x;
    let oy = kill.pos.y;

    // Out of sight
    if !borg_follow_kill_aux(st, iu, oy, ox) {
        return;
    }

    // Note
    note(
        st,
        &[
            b"# There was a monster '",
            race_name(kill.r_idx),
            format!("' at ({},{})", oy, ox).as_bytes(),
        ],
    );

    // Prevent silliness
    if !borg_cave_floor_bold(st, oy, ox) {
        // Delete the monster
        borg_delete_kill(st, i);
        return;
    }

    // Prevent loops (never true, but it draws a random number)
    if engine::randint0(100) + 1 < 1 {
        // Just delete the monster
        borg_delete_kill(st, i);
        return;
    }

    // Some never move, no reason to follow them
    if rf_has(kill.r_idx, RF_NEVER_MOVE)
        // Some are sleeping and don't move, no reason to follow them
        || !kill.awake
    {
        // delete them if they are under me
        if kill.pos.y == st.borg.c.y && kill.pos.x == st.borg.c.x {
            borg_delete_kill(st, i);
        }
        // Don't 'forget' certain ones
        return;
    }

    let mut b_dx = 0;
    let mut b_dy = 0;

    // Scan locations
    for j in 0..8 {
        // Access offset
        let dx = i32::from(engine::ddx_ddd(j));
        let dy = i32::from(engine::ddy_ddd(j));

        // Access location
        let x = ox + dx;
        let y = oy + dy;

        // legal
        if !engine::square_in_bounds_fully(x, y) {
            continue;
        }

        // Access the grid
        let ag = st.cave.grid(x, y);

        // Skip known walls and doors
        if !borg_cave_floor_grid(ag) {
            continue;
        }

        // Skip known monsters
        if ag.kill != 0 {
            continue;
        }

        // Skip visible grids
        if borg_follow_kill_aux(st, iu, y, x) {
            continue;
        }

        // Collect the offsets
        b_dx += dx;
        b_dy += dy;
    }

    // Don't go too far
    b_dx = b_dx.clamp(-1, 1);
    b_dy = b_dy.clamp(-1, 1);

    // Access location
    let x = ox + b_dx;
    let y = oy + b_dy;

    // Access the grid
    let ag = *st.cave.grid(x, y);

    // Avoid walls and doors
    if !borg_cave_floor_grid(&ag) {
        // Just delete the monster
        borg_delete_kill(st, i);
        return;
    }

    // Avoid monsters
    if i32::from(ag.kill) != i {
        // Just delete the monster
        borg_delete_kill(st, i);
        return;
    }

    // Delete monsters that did not really move
    if kill.pos.y == oy && kill.pos.x == ox {
        borg_delete_kill(st, i);
        return;
    }

    // Update the grids
    st.cave.grid_mut(kill.pos.x, kill.pos.y).kill = 0;

    let k = &mut st.kills.kills[iu];

    // Save the old Location
    k.ox = ox as u8;
    k.oy = oy as u8;

    // Save the Location
    k.pos.x = ox + b_dx;
    k.pos.y = oy + b_dy;
    let pos = k.pos;

    // Update the grids
    st.cave.grid_mut(pos.x, pos.y).kill = i as u8;

    // Note
    note(
        st,
        &[
            b"# Following a monster '",
            race_name(kill.r_idx),
            format!("' to ({},{}) from ({},{})", pos.y, pos.x, oy, ox).as_bytes(),
        ],
    );

    // Recalculate danger
    st.danger.wipe = true;

    // Clear goals
    clear_goal_after_move(st, pos);
}

/// Obtain a new "kill" index (`borg_new_kill`); -1 when the game has no
/// monster at the grid.
fn borg_new_kill(st: &mut BorgState, r_idx: u32, y: i32, x: i32) -> i32 {
    let mut n: i32 = -1;

    // Look for a "dead" monster
    let mut i = 1;
    while n < 0 && i < i32::from(st.kills.nxt) {
        // Skip real entries
        if st.kills.kills[i as usize].r_idx == 0 {
            n = i;
        }
        i += 1;
    }

    // Allocate a new monster
    if n < 0 && st.kills.nxt < 255 {
        // Acquire the entry, advance
        n = i32::from(st.kills.nxt);
        st.kills.nxt += 1;
    }

    // Steal an old monster
    if n < 0 {
        // Note
        note(st, &[b"# Too many monsters"]);

        // Pick a random monster
        n = engine::randint0((i32::from(st.kills.nxt) - 1) as u32) + 1 + 1;

        // Kill it
        borg_delete_kill(st, n);
    }

    // it might be that it can't be found
    let Some(m) = engine::square_monster(x, y) else {
        return -1;
    };

    // Count the monsters
    st.kills.cnt = st.kills.cnt.wrapping_add(1);

    let nu = n as usize;

    // The race of the slot *before* it is set (C bug 24): its flags decide
    // the grid below.
    let old_r_idx = st.kills.kills[nu].r_idx;

    {
        let kill = &mut st.kills.kills[nu];

        // Save the race
        kill.r_idx = r_idx as u16;

        // Location
        kill.pos.x = x;
        kill.ox = x as u8;
        kill.pos.y = y;
        kill.oy = y as u8;

        // Games Index of the monster
        kill.m_idx = m.midx as i16;
    }

    // Update the grids
    st.cave.grid_mut(x, y).kill = n as u8;

    // Timestamp
    st.kills.kills[nu].when = st.borg.time.now;

    // Mark the Morgoth time stamp if needed
    if u32::from(st.kills.kills[nu].r_idx) == st.borg.id.morgoth {
        st.borg.time.morgoth = st.borg.time.now;
    }

    // Update the monster
    borg_update_kill_new(st, nu);

    // Update the monster
    borg_update_kill_old(st, nu);

    let kill = st.kills.kills[nu];

    // Note
    note(
        st,
        &[
            b"# Creating a monster '",
            race_name(kill.r_idx),
            format!(
                "' at ({},{}), HP: {}, Index: {}",
                kill.pos.y, kill.pos.x, kill.power, kill.r_idx
            )
            .as_bytes(),
        ],
    );

    // Recalculate danger
    st.danger.wipe = true;

    // Remove Regional Fear which may have been induced from a non-LOS
    // monster. We assume this newly created monster is the one which
    // induced our Regional Fear.
    if borg_timer(st, st.borg.need_see_invis) > 45 {
        let y0 = st.borg.c.y / 11;
        let x0 = st.borg.c.x / 11;

        // Nearby regions
        let y1 = if y0 > 0 { y0 - 1 } else { 0 };
        let x1 = if x0 > 0 { x0 - 1 } else { 0 };
        let y2 = if y0 < 5 { y0 + 1 } else { 5 };
        let x2 = if x0 < 17 { x0 + 1 } else { 17 };

        // Remove "fear", spread around
        for (ry, rx) in [
            (y0, x0),
            (y0, x1),
            (y0, x2),
            (y1, x0),
            (y2, x0),
            (y1, x1),
            (y1, x2),
            (y2, x1),
            (y2, x2),
        ] {
            st.danger.fear_region[ry as usize][rx as usize] = 0;
        }
        note(
            st,
            &[
                format!("# Removing Regional Fear ({},{}) because of a LOS ", y, x).as_bytes(),
                race_name(kill.r_idx),
            ],
        );
    }

    // Wipe goals only if I have some light source
    if st.borg.tr(BI_LIGHT) != 0 && borg_los(st, kill.pos.y, kill.pos.x, st.borg.c.y, st.borg.c.x)
    {
        st.borg.goal.type_0 = 0;
    }

    // Force the monster to be sitting on a floor grid unless that monster
    // can pass through walls; force the ghostly monster to be in a wall
    // grid (both by the slot's old race, see above)
    let old_pass_wall = rf_has(old_r_idx, RF_PASS_WALL);
    let ag = st.cave.grid_mut(x, y);
    if !old_pass_wall {
        ag.feat = FEAT_FLOOR as u8;
    }
    if old_pass_wall {
        ag.feat = FEAT_GRANITE as u8;
    }

    // Count up out list of Nasties
    let d_char = engine::race(usize::from(kill.r_idx)).d_char;
    for k in 0..usize::from(st.kills.nasties_num) {
        // Count them up
        if d_char == i32::from(st.kills.nasties[k]) {
            st.kills.nasties_count[k] = st.kills.nasties_count[k].wrapping_add(1);
        }
    }

    // Return the monster
    n
}

/// Attempt to guess what type of monster is at the given location
/// (`borg_guess_race`). This is a cheat: it reads the game's monster.
fn borg_guess_race(y: i32, x: i32) -> u32 {
    match engine::square_monster(x, y) {
        // Actual monsters
        Some(m) => m.ridx.expect("a live monster has a race"),
        None => 0,
    }
}

/// Attempt to notice a changing "kill" (`observe_kill_diff`)
pub fn observe_kill_diff(st: &mut BorgState, y: i32, x: i32) -> bool {
    // Guess the race
    let r_idx = borg_guess_race(y, x);

    // Oops
    if r_idx == 0 {
        return false;
    }

    // no new monsters if hallucinations
    if st.borg.tr(BI_ISIMAGE) != 0 {
        return false;
    }

    // Create a new monster (never -1 here: the game has a monster there)
    let i = borg_new_kill(st, r_idx, y, x);

    // Get the object (C indexes with -1 unchecked)
    let now = st.borg.time.now;
    let morgoth = st.borg.id.morgoth;
    let kill = &mut st.kills.kills[usize::try_from(i).expect("C indexes borg_kills[-1]")];

    // Timestamp
    kill.when = now;

    // Mark the Morgoth time stamp if needed
    if u32::from(kill.r_idx) == morgoth {
        st.borg.time.morgoth = now;
    }

    // Done
    true
}

/// Attempt to "track" a "kill" at the given location
/// (`observe_kill_move`). Assume that the monster moved at most 'd'
/// grids. If "flag" is true, allow monster "conversion".
pub fn observe_kill_move(
    st: &mut BorgState,
    y: i32,
    x: i32,
    mut d: i32,
    a: u8,
    c: i32,
    flag: bool,
) -> bool {
    let mut flicker = false;

    // Look at the monsters
    for i in 1..i32::from(st.kills.nxt) {
        let iu = i as usize;
        let kill = st.kills.kills[iu];

        // Skip dead monsters
        if kill.r_idx == 0 {
            continue;
        }

        // Skip assigned monsters
        if kill.seen {
            continue;
        }

        // Old location
        let ox = kill.pos.x;
        let oy = kill.pos.y;

        // Calculate distance
        let z = borg_distance(oy, ox, y, x);

        // Access the monster race
        let r = engine::race(usize::from(kill.r_idx));

        // Double the distance for uniques so the borg does not create 2 of
        // the same unique monster on the level (C bug 24: it stays doubled
        // for the rest of the scan)
        if rf_has(kill.r_idx, RF_UNIQUE) {
            d *= 2;
        }

        // Verify distance
        if z > d {
            continue;
        }

        // Verify that we are looking at the right one
        if i32::from(kill.m_idx) != monster_at(loc { x, y }).midx {
            continue;
        }

        // Verify "reasonable" motion, if allowed
        if !flag && z > i32::from(kill.moves) / 10 + 1 {
            continue;
        }

        // Verify matching char so long as not hallucinating
        if st.borg.tr(BI_ISIMAGE) == 0 && c != r.d_char {
            continue;
        }

        // Verify matching attr so long as not hallucinating
        if (a != r.d_attr || st.borg.tr(BI_ISIMAGE) != 0)
            // Require matching attr (for normal monsters)
            && !rf_has(kill.r_idx, RF_ATTR_MULTI)
            && !rf_has(kill.r_idx, RF_ATTR_CLEAR)
        {
            // Require flag
            if !flag {
                continue;
            }

            // Never flicker known monsters
            if kill.known {
                continue;
            }

            // Find a multi-hued monster
            let r_idx = borg_guess_race(y, x);

            // Handle failure
            if r_idx == 0 {
                continue;
            }

            // Note
            note(
                st,
                &[
                    b"# Flickering monster '",
                    engine::race_name(r_idx as usize),
                    format!("' at ({},{})", y, x).as_bytes(),
                ],
            );

            // Note
            note(
                st,
                &[
                    b"# Converting a monster '",
                    race_name(kill.r_idx),
                    format!("' at ({},{})", kill.pos.y, kill.pos.x).as_bytes(),
                ],
            );

            // Change the race
            st.kills.kills[iu].r_idx = r_idx as u16;

            // Monster flickers
            flicker = true;

            // Recalculate danger
            st.danger.wipe = true;

            // Clear monster flow goals
            st.borg.goal.type_0 = 0;
        }

        // Actual movement
        if z != 0 {
            // Update the grids
            st.cave.grid_mut(kill.pos.x, kill.pos.y).kill = 0;

            let k = &mut st.kills.kills[iu];

            // Save the old Location
            k.ox = k.pos.x as u8;
            k.oy = k.pos.y as u8;

            // Save the Location
            k.pos.x = x;
            k.pos.y = y;
            let r_now = k.r_idx;

            // Update the grids
            st.cave.grid_mut(x, y).kill = i as u8;

            // Note
            note(
                st,
                &[
                    b"# Tracking a monster '",
                    race_name(r_now),
                    format!("' at ({},{}) from ({},{})", y, x, oy, ox).as_bytes(),
                ],
            );

            // Recalculate danger
            st.danger.wipe = true;

            // Clear goals
            clear_goal_after_move(st, loc { x, y });
        }

        // Note when last seen
        st.kills.kills[iu].when = st.borg.time.now;

        // Mark the Morgoth time stamp if needed
        if u32::from(st.kills.kills[iu].r_idx) == st.borg.id.morgoth {
            st.borg.time.morgoth = st.borg.time.now;
        }

        // Monster flickered
        if flicker {
            // Update the monster
            borg_update_kill_new(st, iu);
        }

        // Update the monster
        borg_update_kill_old(st, iu);

        // Mark as seen
        st.kills.kills[iu].seen = true;

        // Done
        return true;
    }

    // Oops
    false
}

/// Binary search for the last entry `<= who` (C's search loop): returns
/// `m` (0 for an empty table, as in C).
fn search_last_le(text: &[Vec<u8>], who: &[u8]) -> usize {
    let mut m: i32 = 0;
    let mut n: i32 = text.len() as i32;
    while m < n - 1 {
        // Pick a "middle" entry
        let i = (m + n) / 2;

        // Search to the right (or here); strcmp is a byte comparison
        if text[i as usize].as_slice() <= who {
            m = i;
        }
        // Search to the left
        else {
            n = i;
        }
    }
    m as usize
}

/// Attempt to convert a monster name into a race index
/// (`borg_guess_race_name`).
///
/// First we check for all possible "unique" monsters, including ones we
/// have killed, and even if the monster name is "prefixed" (as in "The
/// Tarrasque" and "The Lernean Hydra"). Otherwise, if the monster is NOT
/// named "The xxx", we assume it must be a "player ghost". Then we search
/// the "normal" names (the search finds the last matching entry, and we
/// scan backwards over the matches). Finally, we assume a "player ghost".
fn borg_guess_race_name(st: &mut BorgState, who: &[u8]) -> u32 {
    let ghost = u32::from(engine::z_info().r_max) - 1;
    let cdepth = st.borg.tr(BI_CDEPTH);
    let score = |i: u32| 1000 - (engine::race(i as usize).level - cdepth).abs();
    let (r_idx, notes) = guess_race_name(&st.kills, who, ghost, &score);
    for n in notes {
        borg_note_bytes(st, &n);
    }
    r_idx
}

/// The search of `borg_guess_race_name`, without side effects: returns the
/// race and the notes to show. `ghost` is the player ghost race
/// (`r_max - 1`); `score(i)` is `1000 - ABS(r_info[i].level - depth)`.
pub fn guess_race_name(
    kills: &KillState,
    who_in: &[u8],
    ghost: u32,
    score: &dyn Fn(u32) -> i32,
) -> (u32, Vec<Vec<u8>>) {
    let mut notes = Vec::new();
    let mut who: &[u8] = who_in;
    let len = who.len();
    let partial: Vec<u8>;

    // Binary search over the uniques (C reads entry 0 of an empty table)
    let m = search_last_le(&kills.unique_text, who);

    // Check for equality
    if who == kills.unique_text[m].as_slice() {
        // Use this monster
        return (kills.unique_what[m], notes);
    }

    // Handle "offscreen"
    if suffix(who, b" (offscreen)") {
        // Remove the suffix (my_strcpy into char[160], then cut)
        let mut p = who[..len.min(159)].to_vec();
        p.truncate(len - 12);
        partial = p;
        who = &partial;

        // Message
        notes.push([b"# Handling offscreen monster (", who, b")"].concat());
    }

    // Assume player ghost
    if !prefix(who, b"The ") && !prefix(who, b"the ") {
        // Message
        notes.push([b"# Assuming player ghost (", who, b") (a)"].concat());

        // Oops
        return (ghost, notes);
    }

    // Skip the prefix
    let who = &who[4..];

    // Binary search over the normal names
    let m = search_last_le(&kills.normal_text, who);

    let mut b_i: i32 = 0;
    let mut b_s: i32 = 0;

    // Scan possibilities
    for k in (0..=m).rev() {
        // Stop when done
        if who != kills.normal_text[k].as_slice() {
            break;
        }

        // Extract the monster
        let i = kills.normal_what[k] as i32;

        // Basic score, penalize "depth miss"
        let s = score(i as u32);

        // Track best
        if b_i != 0 && s < b_s {
            continue;
        }

        // Track it
        b_i = i;
        b_s = s;
    }

    // Success
    if b_i != 0 {
        return (b_i as u32, notes);
    }

    // No match found
    notes.push([b"# Assuming player ghost (", who, b")(b)"].concat());

    // Oops
    (ghost, notes)
}

/// Attempt to locate a monster which could explain a message involving
/// the given monster name, near the given location, up to the given
/// distance from the given location (`borg_locate_kill`).
///
/// First, we attempt to convert "similar" objects into the desired
/// monster, then we attempt to convert "similar" monsters into the desired
/// monster, then we attempt to match an existing monster, and finally, we
/// give up.
pub fn borg_locate_kill(st: &mut BorgState, who: &[u8], mut c: loc, r: i32) -> i32 {
    // Handle invisible monsters
    if strnieq(who, b"Something", 9) || strnieq(who, b"It", 3) {
        // Note
        note(st, &[b"# Invisible monster nearby."]);
        // if I can, cast detect inviso--time stamp it
        if borg_timer(st, st.borg.need_see_invis) > 50 {
            st.borg.need_see_invis = st.borg.time.now;
        }

        // Ignore
        return 0;
    }

    // Handle offsreen monsters
    if suffix(who, b" (offscreen)") {
        // Note
        note(st, &[b"# Offscreen monster nearby"]);

        // Shift the panel
        st.borg.need_shift_panel = true;

        // Ignore
        return 0;
    }

    // Guess the monster race
    let r_idx = borg_guess_race_name(st, who);

    // Access the monster race
    let rr = engine::race(r_idx as usize);

    // Note
    if st.cfg(BORG_VERBOSE) != 0 {
        note(
            st,
            &[
                b"# There is a monster '",
                race_name(r_idx as i32),
                format!("' within {} grids of {},{}", r, c.y, c.x).as_bytes(),
            ],
        );
    }

    // Count racial appearances
    if st.kills.race_count[r_idx as usize] < SHRT_MAX {
        st.kills.race_count[r_idx as usize] += 1;
    }

    // Handle trappers and lurkers and mimics
    if rf_has(r_idx, RF_CHAR_CLEAR) {
        // Note
        note(st, &[b"# Bizarre monster nearby"]);
    }

    let attr_free = rf_has(r_idx, RF_ATTR_MULTI) || rf_has(r_idx, RF_ATTR_CLEAR);

    // *** Find a similar object ***

    // Nothing yet
    let mut b_i: i32 = -1;
    let mut b_d: i32 = 999;

    // Scan the objects
    for i in 1..i32::from(st.takes.nxt) {
        let take = st.takes.takes[i as usize];

        // Skip "dead" objects
        if take.kidx == 0 {
            continue;
        }

        // Access kind
        let k_ptr = engine::kind(take.kidx as usize);

        // Verify char
        if k_ptr.d_char != rr.d_char {
            continue;
        }

        // Verify attr (unless clear or multi-hued), unless flavored
        if !attr_free && k_ptr.flavor.is_null() && k_ptr.d_attr != rr.d_attr {
            continue;
        }

        // Calculate distance
        let d = engine::distance(loc { x: i32::from(take.x), y: i32::from(take.y) }, c);

        // Skip "wrong" objects
        if d > r {
            continue;
        }

        // Track closest one
        if d > b_d {
            continue;
        }

        // Track it
        b_i = i;
        b_d = d;
    }

    // Found one
    if b_i >= 0 {
        let take = st.takes.takes[b_i as usize];

        // Note
        note(
            st,
            &[
                b"# Converting an object '",
                engine::kind_name(take.kidx as usize),
                format!("' at ({},{})", take.y, take.x).as_bytes(),
            ],
        );

        // Save location
        c.x = i32::from(take.x);
        c.y = i32::from(take.y);

        // Delete the object
        crate::borg_flow_take::borg_delete_take(st, b_i);

        // Make a new monster
        let b_i = borg_new_kill(st, r_idx, c.y, c.x);
        if b_i < 0 {
            return b_i;
        }

        // Get the monster
        let now = st.borg.time.now;
        let kill = &mut st.kills.kills[b_i as usize];

        // Timestamp
        kill.when = now;

        // Mark the Morgoth time stamp if needed
        if u32::from(kill.r_idx) == st.borg.id.morgoth {
            st.borg.time.morgoth = now;
        }

        // Known identity
        if r == 0 {
            st.kills.kills[b_i as usize].known = true;
        }

        // Return the index
        return b_i;
    }

    // *** Find a similar monster ***

    // Nothing yet
    let mut b_i: i32 = -1;
    let mut b_d: i32 = 999;

    // Scan the monsters
    for i in 1..i32::from(st.kills.nxt) {
        let kill = st.kills.kills[i as usize];

        // Skip "dead" monsters
        if kill.r_idx == 0 {
            continue;
        }

        // Skip "matching" monsters
        if u32::from(kill.r_idx) == r_idx {
            continue;
        }

        // check the position is the same
        let m = engine::cave_monster(i32::from(kill.m_idx));
        if m.grid.x != kill.pos.x || m.grid.y != kill.pos.y {
            continue;
        }

        let kr = engine::race(usize::from(kill.r_idx));

        // Verify char
        if kr.d_char != rr.d_char {
            continue;
        }

        // Verify attr (unless clear or multi-hued)
        if !attr_free && kr.d_attr != rr.d_attr {
            continue;
        }

        // Distance away
        let d = engine::distance(kill.pos, c);

        // Check distance
        if d > r {
            continue;
        }

        // Track closest one
        if d > b_d {
            continue;
        }

        // Track it
        b_i = i;
        b_d = d;
    }

    // Found one
    if b_i >= 0 {
        let bu = b_i as usize;
        let kill = st.kills.kills[bu];

        // Note
        note(
            st,
            &[
                b"# Converting a monster '",
                race_name(kill.r_idx),
                format!("' at ({},{})", kill.pos.y, kill.pos.x).as_bytes(),
            ],
        );

        // Change the race
        st.kills.kills[bu].r_idx = r_idx as u16;

        // Update the monster
        borg_update_kill_new(st, bu);

        // Update the monster
        borg_update_kill_old(st, bu);

        // Known identity
        if r == 0 {
            st.kills.kills[bu].known = true;
        }

        // Recalculate danger
        st.danger.wipe = true;

        // Clear goals
        st.borg.goal.type_0 = 0;

        // Index
        return b_i;
    }

    // *** Find an existing monster ***

    // Nothing yet
    let mut b_i: i32 = -1;
    let mut b_d: i32 = 999;

    // Scan the monsters
    for i in 1..i32::from(st.kills.nxt) {
        let kill = st.kills.kills[i as usize];

        // Skip "dead" monsters
        if kill.r_idx == 0 {
            continue;
        }

        // Skip "different" monsters
        if u32::from(kill.r_idx) != r_idx {
            continue;
        }

        // Distance away
        let d = engine::distance(kill.pos, c);

        // Check distance
        if d > r + 3 {
            continue;
        }

        // Hopefully this will add fear to our grid
        if !borg_projectable(st, kill.pos.y, kill.pos.x, c.y, c.x) {
            continue;
        }

        // Track closest one
        if d > b_d {
            continue;
        }

        // Track it
        b_i = i;
        b_d = d;
    }

    // *** Find an existing monster Last Chance ***
    // There can be some problems with monsters that use melee attack.
    // The range (r) will be 1, but the known monster may be a few paces
    // further and moved closely and attacked in the same round, so we
    // extend the range of the search.
    if b_i == -1 {
        // Nothing yet
        b_i = -1;
        b_d = 999;

        // Scan the monsters
        for i in 1..i32::from(st.kills.nxt) {
            let kill = st.kills.kills[i as usize];

            // Skip "dead" monsters
            if kill.r_idx == 0 {
                continue;
            }

            // Skip "different" monsters
            if u32::from(kill.r_idx) != r_idx {
                continue;
            }

            // Distance away
            let d = engine::distance(kill.pos, c);

            // Check distance
            if d > r + 20 {
                continue;
            }

            // Track closest one
            if d > b_d {
                continue;
            }

            // Track it
            b_i = i;
            b_d = d;
        }
    }

    // Found one
    if b_i >= 0 {
        let kill = st.kills.kills[b_i as usize];

        // Note
        if st.cfg(BORG_VERBOSE) != 0 {
            note(
                st,
                &[
                    b"# Matched a monster '",
                    race_name(kill.r_idx),
                    format!("' at ({},{}) for the parsed msg.", kill.pos.y, kill.pos.x).as_bytes(),
                ],
            );
        }

        // Known identity
        if r == 0 {
            st.kills.kills[b_i as usize].known = true;
        }

        // Index
        return b_i;
    }

    // *** Oops ***

    // Note
    if st.cfg(BORG_VERBOSE) != 0 {
        note(
            st,
            &[
                b"# Unable to locate monster '",
                race_name(r_idx as i32),
                format!("' near ({},{}), which generated the msg (", c.y, c.x).as_bytes(),
                who,
                b").",
            ],
        );
    }

    // Oops: this is the case where we know the name of the monster but
    // cannot locate it on the monster list.
    -1
}

/// Notice the "death" of a monster (`borg_count_death`)
pub fn borg_count_death(st: &mut BorgState, i: i32) {
    let kill = st.kills.kills[i as usize];

    if kill.r_idx != 0 {
        let r = usize::from(kill.r_idx);

        // Count racial deaths
        if st.kills.race_death[r] < SHRT_MAX {
            st.kills.race_death[r] += 1;
        }

        // if it was a unique then remove the borg.mon.unique flag
        if rf_has(kill.r_idx, RF_UNIQUE) {
            st.borg.mon.unique = 0;
        }
    }
}

/// Is the grid a wall for the "in a hall" counts (glyph or MAGMA..PERM)?
fn hall_wall(st: &BorgState, x: i32, y: i32) -> bool {
    let ag = st.cave.grid(x, y);
    let f = u32::from(ag.feat);
    ag.glyph || (FEAT_MAGMA..=FEAT_PERM).contains(&f)
}

/// Prepare to "flow" towards monsters to "kill" (`borg_flow_kill`)
/// But in a few phases, viewable, near and far.
/// Note that monsters under the player are always deleted
pub fn borg_flow_kill(st: &mut BorgState, viewable: bool, nearness: i32) -> bool {
    let mut b_j: i32 = -1;
    let mut b_stair: i32 = -1;

    let mut borg_in_hall = false;
    let mut hall_walls = 0;
    let mut skip_monster = false;

    // Efficiency -- Nothing to kill
    if st.kills.cnt == 0 {
        return false;
    }

    // Don't chase down town monsters when you are just starting out
    if st.borg.tr(BI_CDEPTH) == 0
        && st.borg.tr(BI_CLEVEL) < 20
        && st.cfg(BORG_MONEY_SCUM_AMOUNT) == 0
    {
        return false;
    }

    // YOU ARE NOT A WARRIOR!! DON'T ACT LIKE ONE!!
    if (st.borg.tr(BI_CLASS) == CLASS_MAGE || st.borg.tr(BI_CLASS) == CLASS_NECROMANCER)
        && st.borg.tr(BI_CLEVEL) < (if st.borg.tr(BI_CDEPTH) != 0 { 35 } else { 25 })
    {
        return false;
    }

    // Not if Weak from hunger or no food
    if st.borg.tr(BI_ISHUNGRY) != 0 || st.borg.tr(BI_ISWEAK) != 0 || st.borg.tr(BI_FOOD) == 0 {
        return false;
    }

    // Not if sitting in a sea of runes
    if st.borg.morgoth_position {
        return false;
    }

    // Nothing found
    st.flow.temp_n = 0;

    // check to see if in a hall, used later
    for hall_x in -1..=1 {
        for hall_y in -1..=1 {
            // Acquire location
            let x = hall_x + st.borg.c.x;
            let y = hall_y + st.borg.c.y;

            // track walls
            if hall_wall(st, x, y) {
                hall_walls += 1;
            }

            // addem up
            if hall_walls >= 5 {
                borg_in_hall = true;
            }
        }
    }

    // Check distance away from stairs, used later

    // Check for an existing "up stairs"
    for i in 0..i32::from(st.stairs.less.num) {
        let (y, x) = st.stairs.less.at(i as usize);

        // How far is the nearest up stairs
        let j = engine::distance(st.borg.c, loc { x, y });

        // skip the further ones
        if b_j < j && b_j != -1 {
            continue;
        }

        // track it
        b_j = j;
        b_stair = i;
    }

    // Scan the monster list
    for i in 1..i32::from(st.kills.nxt) {
        let kill = st.kills.kills[i as usize];
        let x9 = kill.pos.x;
        let y9 = kill.pos.y;

        // Each monster starts with a clean slate
        skip_monster = false;

        // Skip dead monsters
        if kill.r_idx == 0 {
            continue;
        }

        // Distance components
        let ax = if x9 > st.borg.c.x { x9 - st.borg.c.x } else { st.borg.c.x - x9 };
        let ay = if y9 > st.borg.c.y { y9 - st.borg.c.y } else { st.borg.c.y - y9 };

        // Distance
        let d = ax.max(ay);

        // don't bother flowing to an adjacent monster when I am afraid
        if d == 1 && (st.borg.tr(BI_ISAFRAID) != 0 || st.borg.tr(BI_CRSFEAR) != 0) {
            continue;
        }

        // Ignore multiplying monsters
        if st.borg.goal.ignoring
            && st.borg.tr(BI_ISAFRAID) == 0
            && rf_has(kill.r_idx, RF_MULTIPLY)
        {
            continue;
        }

        // Ignore molds when low level
        if st.borg.tr(BI_MAXCLEVEL) < 10 && rf_has(kill.r_idx, RF_NEVER_MOVE) {
            continue;
        }

        // Avoid flowing to a fight if a scary guy is on the level
        if st.borg.mon.scary {
            continue;
        }

        // Avoid multiplying monsters when low level
        if st.borg.tr(BI_CLEVEL) < 10 && rf_has(kill.r_idx, RF_MULTIPLY) {
            continue;
        }

        // Ignore Maggot until later.
        if rf_has(kill.r_idx, RF_UNIQUE)
            && st.borg.tr(BI_CDEPTH) == 0
            && st.borg.tr(BI_CLEVEL) < 5
        {
            continue;
        }

        // Access the location
        let x = kill.pos.x;
        let y = kill.pos.y;

        // Require line of sight if requested
        if viewable && st.cave.grid(x, y).info & BORG_VIEW == 0 {
            continue;
        }

        // Calculate danger
        let p = borg_danger(st, y, x, 1, true, false);

        // Skip "deadly" monsters unless uniques
        if st.borg.tr(BI_CLEVEL) > 25
            && !rf_has(kill.r_idx, RF_UNIQUE)
            && p > i32::from(st.borg.avoidance) / 2
        {
            continue;
        }
        if st.borg.tr(BI_CLEVEL) <= 15 && p > i32::from(st.borg.avoidance) / 3 {
            continue;
        }

        // Skip ones that make me wander too far
        if b_stair != -1 && st.borg.tr(BI_CLEVEL) < 10 {
            // Check the distance of this monster to the stair
            let (sy, sx) = st.stairs.less.at(b_stair as usize);
            let j = borg_distance(sy, sx, y, x);
            // skip far away monsters while I am close to stair
            let leash = st.borg.tr(BI_CLEVEL) * 5 + 9;
            if b_j <= leash && j >= leash {
                continue;
            }
        }

        // Avoid getting surrounded
        if borg_in_hall && rf_has(kill.r_idx, RF_GROUP_AI) {
            // check to see if monster is in a hall
            hall_walls = 0;
            for hall_x in -1..=1 {
                for hall_y in -1..=1 {
                    if !engine::square_in_bounds_fully(hall_x + x, hall_y + y) {
                        continue;
                    }

                    // track walls
                    if hall_wall(st, hall_x + x, hall_y + y) {
                        hall_walls += 1;
                    }
                }
            }

            // we want the monster to be in a hall also
            if hall_walls < 4 {
                // This monster is not in a hallway.
                // It may not be safe to fight.
                skip_monster = true;
            }
        }

        // Skip this one if it is just 2 grids from me and it can attack me
        // as soon as I move 1 grid closer to it.
        if d == 2 && kill.ranged_attack == 0 && !rf_has(kill.r_idx, RF_NEVER_MOVE) {
            skip_monster = true;
        }

        // skip certain ones
        if skip_monster {
            continue;
        }

        // don't go too far from the stairs
        if borg_flow_misc::borg_flow_far_from_stairs(st, x, y, b_stair) {
            continue;
        }

        // Careful -- Remember it
        st.flow.temp_push(y, x);
    }

    // Nothing to kill
    let n = st.flow.temp_n;
    if n == 0 {
        return false;
    }

    // Clear the flow codes
    borg_flow::borg_flow_clear(st);

    // Look for something to kill
    for i in 0..usize::from(n as u16) {
        // Enqueue the grid
        let (y, x) = st.flow.temp_at(i);
        borg_flow::borg_flow_enqueue_grid(st, y, x);
    }

    // Spread the flow: if we are not flowing toward monsters that we can
    // see, make sure they are at least easily reachable.
    borg_flow::borg_flow_spread(st, nearness, true, !viewable, false, -1, false);

    // Attempt to Commit the flow
    if !borg_flow::borg_flow_commit(st, c"kill", i32::from(GOAL_KILL)) {
        return false;
    }

    // Take one step
    if !borg_flow::borg_flow_old(st, i32::from(GOAL_KILL)) {
        return false;
    }

    // Success
    true
}

/// The borg can take a shot from a distance (`borg_has_distance_attack`)
fn borg_has_distance_attack(st: &mut BorgState) -> bool {
    let max_range = i32::from(engine::z_info().max_range);

    // line up Magic Missile shots (covers Mages)
    if borg_attack_aux_spell_bolt(
        st,
        MAGIC_MISSILE,
        0,
        10,
        BORG_ATTACK_MISSILE as i32,
        max_range,
        false,
    ) > 0
    {
        return true;
    }

    // line up Nether Bolt shots (covers Necromancers)
    if borg_attack_aux_spell_bolt(
        st,
        NETHER_BOLT,
        0,
        10,
        BORG_ATTACK_NETHER as i32,
        max_range,
        false,
    ) > 0
    {
        return true;
    }

    // or arrows (covers warrior/ranger/paladins/rogues)
    if borg_attack_aux_launch(st) > 0 {
        return true;
    }

    false
}

/// Take a couple of steps to line up a shot (`borg_flow_kill_aim`)
pub fn borg_flow_kill_aim(st: &mut BorgState, viewable: bool) -> bool {
    let s_c_y = st.borg.c.y;
    let s_c_x = st.borg.c.x;

    // Efficiency -- Nothing to kill
    if st.kills.cnt == 0 {
        return false;
    }

    // Sometimes we loop on this if we back up to a point where
    // the monster is out of site
    if st.borg.antibounce_count > 500 {
        return false;
    }

    // Not if Weak from hunger or no food
    if st.borg.tr(BI_ISHUNGRY) != 0 || st.borg.tr(BI_ISWEAK) != 0 || st.borg.tr(BI_FOOD) == 0 {
        return false;
    }

    // If you can shoot from where you are, don't bother reaiming
    if borg_has_distance_attack(st) {
        return false;
    }

    // Consider each adjacent spot
    for o_x in -2..=2 {
        for o_y in -2..=2 {
            // borg_attack would have already checked
            // for a shot from where I currently am
            if o_x == 0 && o_y == 0 {
                continue;
            }

            // XXX  Mess with where the program thinks the player is
            st.borg.c.x = s_c_x + o_x;
            st.borg.c.y = s_c_y + o_y;

            // avoid screen edges
            if st.borg.c.x > AUTO_MAX_X - 2
                || st.borg.c.x < 2
                || st.borg.c.y > AUTO_MAX_Y - 2
                || st.borg.c.y < 2
            {
                continue;
            }

            // Make sure we do not end up next to a monster (all slots
            // from 0, dead ones included, as in C)
            let nxt = i32::from(st.kills.nxt);
            let mut i = 0;
            while i < nxt {
                if engine::distance(st.borg.c, st.kills.kills[i as usize].pos) == 1 {
                    break;
                }
                i += 1;
            }
            if i != nxt {
                continue;
            }

            // Check for a distance attack from here
            if borg_has_distance_attack(st) {
                // Clear the flow codes
                borg_flow::borg_flow_clear(st);

                // Enqueue the grid
                let (cy, cx) = (st.borg.c.y, st.borg.c.x);
                borg_flow::borg_flow_enqueue_grid(st, cy, cx);

                // restore the saved player position
                st.borg.c.x = s_c_x;
                st.borg.c.y = s_c_y;

                // Spread the flow
                borg_flow::borg_flow_spread(st, 5, true, !viewable, false, -1, false);

                // Attempt to Commit the flow
                if !borg_flow::borg_flow_commit(st, c"targetable position", i32::from(GOAL_KILL)) {
                    return false;
                }

                // Take one step
                if !borg_flow::borg_flow_old(st, i32::from(GOAL_KILL)) {
                    return false;
                }

                return true;
            }
        }
    }

    // restore the saved player position
    st.borg.c.x = s_c_x;
    st.borg.c.y = s_c_y;

    false
}

/// One anti-summon corridor pattern: which of the 25 grids must be wall
/// (0) or may be anything "good" (1), and their offsets.
struct Pattern {
    array: [i32; 25],
    dy: [i32; 25],
    dx: [i32; 25],
}

const NORTH: Pattern = Pattern {
    array: [1, 0, 0, 0, 1, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 0, 0, 1, 0, 0, 1, 1, 1, 1, 1],
    dy: [-4, -4, -4, -4, -4, -3, -3, -3, -3, -3, -2, -2, -2, -2, -2, -1, -1, -1, -1, -1, 0, 0, 0, 0, 0],
    dx: [-2, -1, 0, 1, 2, -2, -1, 0, 1, 2, -2, -1, 0, 1, 2, -2, -1, 0, 1, 2, -2, -1, 0, 1, 2],
};

const SOUTH: Pattern = Pattern {
    array: [1, 1, 1, 1, 1, 0, 0, 1, 0, 0, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 1, 0, 0, 0, 1],
    dy: [0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 3, 3, 3, 3, 3, 4, 4, 4, 4, 4],
    dx: [-2, -1, 0, 1, 2, -2, -1, 0, 1, 2, -2, -1, 0, 1, 2, -2, -1, 0, 1, 2, -2, -1, 0, 1, 2],
};

const EAST: Pattern = Pattern {
    array: [1, 0, 0, 1, 1, 1, 0, 1, 0, 0, 1, 1, 0, 1, 0, 1, 0, 1, 0, 0, 1, 0, 0, 1, 1],
    dy: [-2, -2, -2, -2, -2, -1, -1, -1, -1, -1, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 2, 2, 2, 2, 2],
    dx: [0, 1, 2, 3, 4, 0, 1, 2, 3, 4, 0, 1, 2, 3, 4, 0, 1, 2, 3, 4, 0, 1, 2, 3, 4],
};

const WEST: Pattern = Pattern {
    array: [1, 1, 0, 0, 1, 0, 0, 1, 0, 1, 0, 1, 0, 1, 1, 0, 0, 1, 0, 1, 1, 1, 0, 0, 1],
    dy: [-2, -2, -2, -2, -2, -1, -1, -1, -1, -1, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 2, 2, 2, 2, 2],
    dx: [-4, -3, -2, -1, 0, -4, -3, -2, -1, 0, -4, -3, -2, -1, 0, -4, -3, -2, -1, 0, -4, -3, -2, -1, 0],
};

/// Count the grids of a pattern placed at the Borg plus (o_x, o_y) that
/// fit it (25 = a good spot). Out-of-bounds grids do not count.
fn pattern_score(st: &BorgState, p: &Pattern, o_x: i32, o_y: i32) -> i32 {
    let mut walls = 0;
    for i in 0..25 {
        // Check grids near borg
        let m_y = st.borg.c.y + o_y + p.dy[i];
        let m_x = st.borg.c.x + o_x + p.dx[i];

        // avoid screen edges
        if !engine::square_in_bounds_fully(m_x, m_y) {
            continue;
        }

        let f = u32::from(st.cave.grid(m_x, m_y).feat);
        let wall_like = (FEAT_MAGMA..=FEAT_QUARTZ_K).contains(&f) || f == FEAT_GRANITE;

        // Certain grids must not be floor types
        if p.array[i] == 0 && (f == FEAT_NONE || wall_like) {
            // This is a good grid
            walls += 1;
        }
        if p.array[i] == 1 && (f <= FEAT_MORE || wall_like) {
            // A good wall would score 25.
            walls += 1;
        }
    }
    walls
}

/// Enqueue a hiding grid, dig to it and take one step.
fn dig_to(st: &mut BorgState, y: i32, x: i32, depth: i32, who: &CStr, twice: bool) -> bool {
    // Clear the flow codes
    borg_flow::borg_flow_clear(st);

    // Enqueue the grid where I will hide
    st.borg.status.digging = true;
    borg_flow::borg_flow_enqueue_grid(st, y, x);

    // Spread the flow
    if twice {
        st.borg.status.digging = true;
    }
    borg_flow::borg_flow_spread(st, depth, true, false, true, -1, false);

    // Attempt to Commit the flow
    if !borg_flow::borg_flow_commit(st, who, GOAL_DIGGING) {
        return false;
    }

    // Take one step
    if !borg_flow::borg_flow_old(st, GOAL_DIGGING) {
        return false;
    }

    true
}

/// Dig an anti-summon corridor. Type I (`borg_flow_kill_corridor`)
///
/// We want the borg to dig a tunnel which limits the LOS of summoned
/// monsters; it works better in hallways. The Borg looks at the grids
/// near him for these patterns (N, S, W, E):
///
/// ```text
/// #####  ..@..  ####.  .####
/// ##.##  ##.##  ##.#.  .#.##
/// #.#.#  #.#.#  #.#.@  @.#.#
/// ##.##  ##.##  ##.#.  .#.##
/// ..@..  #####  ####.  .####
/// ```
pub fn borg_flow_kill_corridor(st: &mut BorgState) -> bool {
    let mut b_y = 0;
    let mut b_x = 0;
    let mut b_distance = 99;

    let mut b_n = false;
    let mut b_s = false;
    let mut b_e = false;
    let mut b_w = false;

    st.borg.status.digging = false;

    // Efficiency -- Nothing to kill
    if st.kills.cnt == 0 {
        return false;
    }

    // Only do this to summoners when they are close
    if st.borg.near.summoner_idx == -1 {
        return false;
    }

    // Hungry,starving
    if st.borg.tr(BI_ISHUNGRY) != 0 || st.borg.tr(BI_ISWEAK) != 0 {
        return false;
    }

    // Sometimes we loop on this
    if st.borg.antibounce_count > 500 {
        return false;
    }

    // Do not dig when confused
    if st.borg.tr(BI_ISCONFUSED) != 0 {
        return false;
    }

    // Not when darkened
    if st.borg.tr(BI_LIGHT) == 0 {
        return false;
    }

    // Not if sitting in a sea of runes
    if st.borg.morgoth_position {
        return false;
    }
    if st.borg.status.anti_summon {
        return false;
    }

    // get the summoning monster
    let kill = st.kills.kills[st.borg.near.summoner_idx as usize];

    // Summoner must be mobile
    if rf_has(kill.r_idx, RF_NEVER_MOVE) {
        return false;
    }
    // Summoner must be able to pass through walls
    if rf_has(kill.r_idx, RF_PASS_WALL) {
        return false;
    }
    if rf_has(kill.r_idx, RF_KILL_WALL) {
        return false;
    }

    // Summoner has to be awake (so he will chase me
    if !kill.awake {
        return false;
    }

    // Must have Stone to Mud spell
    if !borg_spell_okay(st, TURN_STONE_TO_MUD)
        && !borg_spell_okay(st, SHATTER_STONE)
        && !crate::borg_item_use::borg_equips_ring(st, st.item_val.sv_ring_digging)
        && !crate::borg_item_use::borg_equips_item(st, st.act.stone_to_mud, true)
    {
        return false;
    }

    // Summoner needs to be able to follow me.
    // So I either need to be able to
    // 1) have LOS on him or
    // 2) this panel needs to have had Magic Map or Wizard light cast on it.
    // If Mapped, then the flow codes needs to be used.
    if !borg_los(st, kill.pos.y, kill.pos.x, st.borg.c.y, st.borg.c.x) {
        // Extract panel
        let q_x = st.core.w_x / borg_panel_wid();
        let q_y = st.core.w_y / borg_panel_hgt();

        let mapped = st.update.detect(Detect::Wall, q_y, q_x)
            && st.update.detect(Detect::Wall, q_y, q_x + 1)
            && st.update.detect(Detect::Wall, q_y + 1, q_x)
            && st.update.detect(Detect::Wall, q_y + 1, q_x + 1);
        borg_flow::borg_flow_clear(st);
        st.borg.status.digging = true;
        borg_flow::borg_flow_enqueue_grid(st, kill.pos.y, kill.pos.x);
        borg_flow::borg_flow_spread(st, 10, true, !mapped, false, -1, false);
        if !borg_flow::borg_flow_commit(st, c"Monster Path", i32::from(GOAL_KILL)) {
            return false;
        }
    }

    // NORTH -- Consider each area near the borg, looking for a good spot
    // to hide
    for o_y in -2..1 {
        // No E-W offset when looking North-South
        let o_x = 0;

        // If I found 25 grids, then that spot will work well
        if pattern_score(st, &NORTH, o_x, o_y) == 25 {
            let spot = loc { x: st.borg.c.x + o_x + NORTH.dx[7], y: st.borg.c.y + o_y + NORTH.dy[7] };
            if engine::distance(st.borg.c, spot) < b_distance {
                b_y = o_y;
                b_x = o_x;
                b_n = true;
                b_distance = engine::distance(st.borg.c, spot);
            }
        }
    }

    // SOUTH -- Consider each area near the borg, looking for a good spot
    // to hide
    for o_y in -1..2 {
        // No lateral offset on South check
        let o_x = 0;

        // If I found 25 grids, then that spot will work well
        if pattern_score(st, &SOUTH, o_x, o_y) == 25 {
            let (cy, cx) = (st.borg.c.y, st.borg.c.x);
            if borg_distance(cy, cx, cy + o_y + SOUTH.dy[17], cx + o_x + SOUTH.dx[17])
                < b_distance
            {
                b_y = o_y;
                b_x = o_x;
                b_s = true;
                b_n = false;
                b_distance =
                    borg_distance(cy, cx, cy + b_y + SOUTH.dy[17], cx + b_x + SOUTH.dx[17]);
            }
        }
    }

    // EAST -- Consider each area near the borg, looking for a good spot to
    // hide
    for o_x in -1..2 {
        // No N-S offset check when looking E-W
        let o_y = 0;

        // If I found 25 grids, then that spot will work well
        if pattern_score(st, &EAST, o_x, o_y) == 25 {
            let (cy, cx) = (st.borg.c.y, st.borg.c.x);
            if borg_distance(cy, cx, cy + o_y + EAST.dy[13], cx + o_x + EAST.dx[13]) < b_distance
            {
                b_y = o_y;
                b_x = o_x;
                b_e = true;
                b_s = false;
                b_n = false;
                b_distance =
                    borg_distance(cy, cx, cy + b_y + EAST.dy[13], cx + b_x + EAST.dx[13]);
            }
        }
    }

    // WEST -- Consider each area near the borg, looking for a good spot to
    // hide
    for o_x in -2..1 {
        // No N-S offset check when looking E-W
        let o_y = 0;

        // If I found 25 grids, then that spot will work well
        if pattern_score(st, &WEST, o_x, o_y) == 25 {
            let (cy, cx) = (st.borg.c.y, st.borg.c.x);
            if borg_distance(cy, cx, cy + o_y + WEST.dy[11], cx + o_x + WEST.dx[11]) < b_distance
            {
                b_y = o_y;
                b_x = o_x;
                b_w = true;
                b_e = false;
                b_s = false;
                b_n = false;
                b_distance =
                    borg_distance(cy, cx, cy + o_y + WEST.dy[11], cx + o_x + WEST.dx[11]);
            }
        }
    }

    // Attempt to enqueue the grids that should be floor grids and have the
    // borg move onto those grids
    let (cy, cx) = (st.borg.c.y, st.borg.c.x);
    if b_n {
        return dig_to(
            st,
            cy + b_y + NORTH.dy[7],
            cx + b_x + NORTH.dx[7],
            5,
            c"anti-summon corridor north type 1",
            false,
        );
    }
    if b_s {
        return dig_to(
            st,
            cy + b_y + SOUTH.dy[17],
            cx + b_x + SOUTH.dx[17],
            6,
            c"anti-summon corridor south type 1",
            false,
        );
    }
    if b_e {
        return dig_to(
            st,
            cy + b_y + EAST.dy[13],
            cx + b_x + EAST.dx[13],
            5,
            c"anti-summon corridor east type 1",
            true,
        );
    }
    if b_w {
        return dig_to(
            st,
            cy + b_y + WEST.dy[11],
            cx + b_x + WEST.dx[11],
            5,
            c"anti-summon corridor west type 1",
            false,
        );
    }

    false
}

/// Dig a straight Tunnel to a close monster (`borg_flow_kill_direct`)
pub fn borg_flow_kill_direct(st: &mut BorgState, twitchy: bool) -> bool {
    let mut b_i: i32 = -1;
    let mut b_d = i32::from(engine::z_info().max_sight);

    // Assume we need to dig granite
    if !borg_flow::borg_can_dig(st, false, FEAT_GRANITE as u8) {
        return false;
    }

    // Not if Weak from hunger or no food
    if !twitchy
        && (st.borg.tr(BI_ISHUNGRY) != 0 || st.borg.tr(BI_ISWEAK) != 0 || st.borg.tr(BI_FOOD) == 0)
    {
        return false;
    }

    // Only when sitting for too long or twitchy
    if !twitchy && borg_timer(st, st.borg.time.level) < 3000 && st.borg.times_twitch < 5 {
        return false;
    }

    // Do not dig when confused
    if st.borg.tr(BI_ISCONFUSED) != 0 {
        return false;
    }

    // Not when darkened
    if st.borg.tr(BI_LIGHT) == 0 {
        return false;
    }

    // Efficiency -- Nothing to kill
    if st.kills.cnt != 0 {
        // Scan the monsters
        for i in 1..i32::from(st.kills.nxt) {
            let kill = st.kills.kills[i as usize];

            // Skip "dead" monsters
            if kill.r_idx == 0 {
                continue;
            }

            // Distance away
            let d = engine::distance(kill.pos, st.borg.c);

            // Track closest one
            if d > b_d {
                continue;
            }

            // Track it
            b_i = i;
            b_d = d;
        }
    }

    // If no Kill, then pick the center of the map
    if b_i == -1 {
        return dig_to_direct(st, AUTO_MAX_Y / 2, AUTO_MAX_X / 2, 150, c"center direct");
    }

    // don't want it near permawall
    if b_i != 0 {
        // get the closest monster
        let kill = st.kills.kills[b_i as usize];
        return dig_to_direct(st, kill.pos.y, kill.pos.x, 15, c"kill direct");
    }

    false
}

/// The flow of `borg_flow_kill_direct` (no `digging` flag).
fn dig_to_direct(st: &mut BorgState, y: i32, x: i32, depth: i32, who: &CStr) -> bool {
    // Clear the flow codes
    borg_flow::borg_flow_clear(st);

    // Enqueue the grid
    borg_flow::borg_flow_enqueue_grid(st, y, x);

    // Spread the flow
    borg_flow::borg_flow_spread(st, depth, true, false, true, -1, false);

    // Attempt to Commit the flow
    if !borg_flow::borg_flow_commit(st, who, GOAL_DIGGING) {
        return false;
    }

    // Take one step
    if !borg_flow::borg_flow_old(st, GOAL_DIGGING) {
        return false;
    }

    true
}

/// Scan the monster lists for certain types of monster that we should be
/// concerned over (`borg_near_monster_type`). This only works for monsters
/// we know about. Special consideration is given to Morgoth.
pub fn borg_near_monster_type(st: &mut BorgState, dist: i32) {
    let breeder_count = i32::from(st.borg.mon.breeders);

    // reset the borg flags
    st.borg.near.summoner = false;
    st.borg.near.unique = 0;
    st.borg.near.evil_unique = false;
    st.borg.near.summoner_idx = -1;

    // Scan the monsters
    for i in 1..i32::from(st.kills.nxt) {
        let kill = st.kills.kills[i as usize];

        // Skip dead monsters
        if kill.r_idx == 0 {
            continue;
        }

        // "player ghosts"
        if i32::from(kill.r_idx) >= i32::from(engine::z_info().r_max) - 1 {
            continue;
        }

        let r_idx = kill.r_idx;
        let ridx = engine::race(usize::from(r_idx)).ridx;
        let id = st.borg.id;
        let b = &st.borg;
        let clevel = b.tr(BI_CLEVEL);
        let mut scary = false;

        // *** Scan for Scary Guys ***
        // Done before the distance checks: we are looking for scary guys on
        // the level, not scary guys near me.

        // run from certain scaries
        if clevel <= 5 && ridx == id.squint_eyed_rogue {
            scary = true;
        }

        // Mage and priest are extra fearful
        if clevel <= 6
            && (b.tr(BI_CLASS) == CLASS_MAGE || b.tr(BI_CLASS) == CLASS_PRIEST)
            && ridx == id.squint_eyed_rogue
        {
            scary = true;
        }

        // run from certain dungeon scaries
        if clevel <= 5 && (ridx == id.grip || ridx == id.fang || ridx == id.small_kobold) {
            scary = true;
        }

        // run from certain scaries
        if clevel <= 8
            && (ridx == id.soldier
                || ridx == id.cutpurse
                || ridx == id.acolyte
                || ridx == id.apprentice
                || ridx == id.kobold
                || ridx == id.shrieker_mushroom_patch
                || ridx == id.maggot
                || ridx == id.filthy_street_urchin
                || ridx == id.battle_scarred_veteran
                || ridx == id.mean_looking_mercenary)
        {
            scary = true;
        }

        if clevel <= 15
            && (ridx == id.bullroarer
                || ((ridx == id.giant_white_mouse
                    || ridx == id.white_worm_mass
                    || ridx == id.green_worm_mass)
                    && breeder_count >= clevel))
        {
            scary = true;
        }

        if clevel <= 20
            && (ridx == id.cave_spider
                || ridx == id.red_naga
                || ridx == id.giant_red_frog
                || ridx == id.radiation_eye
                || (ridx == id.yellow_worm_mass && breeder_count >= clevel))
        {
            scary = true;
        }

        if clevel < 45
            && (ridx == id.gravity_hound
                || ridx == id.inertia_hound
                || prefix_i(engine::race_base_name(usize::from(r_idx)), b"ancient dragon")
                || ridx == id.beorn
                || ridx == id.dread)
        {
            scary = true;
        }

        // Nether breath is bad
        if b.tr(BI_SRNTHR) == 0
            && (ridx == id.osse // Ossë, Herald of Ulmo
                || ridx == id.dracolich
                || ridx == id.dracolisk)
        {
            scary = true;
        }

        // Blindness is really bad
        if b.tr(BI_SRBLIND) == 0
            && ((ridx == id.light_hound && b.tr(BI_SRLITE) == 0)
                || (ridx == id.dark_hound && b.tr(BI_SRDARK) == 0))
        {
            scary = true;
        }

        // Chaos and Confusion are really bad
        if (b.tr(BI_SRKAOS) == 0 && b.tr(BI_SRCONF) == 0) && rsf_has(r_idx, RSF_BR_CHAO) {
            scary = true;
        }

        if b.tr(BI_SRCONF) == 0 && (ridx == id.pukelman || ridx == id.night_mare) {
            scary = true;
        }

        // Poison is really Bad (note the RPois not SRPois)
        if b.tr(BI_RPOIS) == 0 && ridx == id.drolem {
            scary = true;
        }

        if scary {
            st.borg.mon.scary = true;
        }

        // Now do distance considerations
        let x9 = kill.pos.x;
        let y9 = kill.pos.y;

        // Distance components
        let ax = if x9 > st.borg.c.x { x9 - st.borg.c.x } else { st.borg.c.x - x9 };
        let ay = if y9 > st.borg.c.y { y9 - st.borg.c.y } else { st.borg.c.y - y9 };

        // Distance
        let d = ax.max(ay);

        // if the guy is too far then skip it unless in town.
        if d > dist && st.borg.tr(BI_CDEPTH) != 0 {
            continue;
        }

        // *** Scan for Uniques ***

        // this is a unique.
        if rf_has(r_idx, RF_UNIQUE) {
            // Set a flag for use with certain types of spells
            st.borg.mon.unique = u32::from(r_idx);

            // return 1 if not Morgy, +10 if it is Morgy or Sauron
            if rf_has(r_idx, RF_QUESTOR) {
                st.borg.near.unique = st.borg.near.unique.wrapping_add(10);
            }

            // regular unique
            st.borg.near.unique = st.borg.near.unique.wrapping_add(1);

            // Note that fighting a Questor would result in a 11 value
            if rf_has(r_idx, RF_EVIL) {
                st.borg.near.evil_unique = true;
            }
        }

        // *** Scan for Summoners ***
        let summons = [
            RSF_S_KIN,
            RSF_S_HI_DEMON,
            RSF_S_MONSTER,
            RSF_S_MONSTERS,
            RSF_S_ANIMAL,
            RSF_S_SPIDER,
            RSF_S_HOUND,
            RSF_S_HYDRA,
            RSF_S_AINU,
            RSF_S_DEMON,
            RSF_S_UNDEAD,
            RSF_S_DRAGON,
            RSF_S_HI_DRAGON,
            RSF_S_HI_UNDEAD,
            RSF_S_WRAITH,
            RSF_S_UNIQUE,
        ];
        if summons.iter().any(|&s| rsf_has(r_idx, s)) {
            // mark the flag
            st.borg.near.summoner = true;

            // recheck the distance to see if close
            // and mark the index for as-corridor
            if d < 8 {
                st.borg.near.summoner_idx = i as i16;
            }
        }
    }
}

/// Help determine if PHASE_DOOR with Shoot N Scoot seems like a good idea
/// (`borg_shoot_scoot_safe`).
///
/// The emergency is the number of dangerous grids out of 100 that we
/// tolerate. The pattern of ShootN'Scoot: shoot a far monster as it walks
/// closer; when it gets within 1 grid of the borg, phase away; repeat.
///
pub fn borg_shoot_scoot_safe(st: &mut BorgState, emergency: i32, turns: i32, b_p: i32) -> bool {
    let dis = 10;
    let min = dis / 2;

    let mut adjacent_monster = false;

    // no need if high level in town
    if st.borg.tr(BI_CLEVEL) >= 8 && st.borg.tr(BI_CDEPTH) == 0 {
        return false;
    }

    // must have the ability
    if st.borg.tr(BI_APHASE) == 0 {
        return false;
    }

    // Not if No Light
    if st.borg.tr(BI_LIGHT) == 0 {
        return false;
    }

    // Cheat the floor grid. Not if in a vault since it throws us out of
    // the vault
    if engine::square_isvault(st.borg.c.x, st.borg.c.y) {
        return false;
    }

    // *** Need Missiles or cheap spells ***

    // classes that are mainly spellcaster
    if borg_primarily_caster() {
        // Low mana
        if st.borg.tr(BI_CLEVEL) >= 45 && st.borg.tr(BI_CURSP) < 15 {
            return false;
        }

        // Low mana, low level, generally OK
        if st.borg.tr(BI_CLEVEL) < 45 && st.borg.tr(BI_CURSP) < 5 {
            return false;
        }
    } else {
        // Other classes need some missiles
        if st.borg.tr(BI_AMISSILES) < 5 || st.borg.tr(BI_CLEVEL) >= 45 {
            return false;
        }
    }

    // Not if I am in a safe spot for killing special monsters
    if st.borg.morgoth_position || st.borg.status.anti_summon {
        return false;
    }

    // scan the adjacent grids for an awake monster
    for i in 0..8 {
        // Grid in that direction
        let x = st.borg.c.x + i32::from(engine::ddx_ddd(i));
        let y = st.borg.c.y + i32::from(engine::ddy_ddd(i));

        // Access the grid, obtain the monster
        let ag_kill = st.cave.grid(x, y).kill;
        let kill = st.kills.kills[usize::from(ag_kill)];
        let r = usize::from(kill.r_idx);

        // If a qualifying monster is adjacent to me.
        if (ag_kill != 0 && kill.awake)
            && !rf_has(kill.r_idx, RF_NEVER_MOVE)
            && !rf_has(kill.r_idx, RF_PASS_WALL)
            && !rf_has(kill.r_idx, RF_KILL_WALL)
            && i32::from(kill.power) >= st.borg.tr(BI_CLEVEL)
        {
            // Spell casters shoot at everything.
            // All other borgs need to make sure he would shoot. The
            // criteria are exactly the same as the list in
            // borg_launch_damage_one().
            if borg_spell_okay(st, MAGIC_MISSILE)
                || borg_spell_okay(st, ORB_OF_DRAINING)
                || borg_spell_okay(st, NETHER_BOLT)
                || borg_danger_one_kill(
                    st,
                    kill.pos.y,
                    kill.pos.x,
                    1,
                    i32::from(ag_kill),
                    true,
                    false,
                ) > i32::from(st.borg.avoidance) * 3 / 10
                || ((!engine::race(r).friends.is_null()
                    || !engine::race(r).friends_base.is_null()) // monster has friends
                    && i32::from(kill.level) >= st.borg.tr(BI_CLEVEL) - 5) // close levels
                || kill.ranged_attack != 0 // monster has a ranged attack
                || rf_has(kill.r_idx, RF_UNIQUE)
                || rf_has(kill.r_idx, RF_MULTIPLY)
                || st.borg.tr(BI_CLEVEL) <= 5
            // still very weak
            {
                adjacent_monster = true;
            }
        }
    }

    // if No Adjacent_monster no need for it
    if !adjacent_monster {
        return false;
    }

    // Simulate 100 attempts
    let mut n = 0;
    for _k in 0..100 {
        let mut x = 0;
        let mut y = 0;

        // Pick a location
        let mut i = 0;
        while i < 100 {
            // Pick a (possibly illegal) location
            loop {
                y = st.borg.c.y + engine::randint0((1 + dis + dis) as u32) - dis;
                x = st.borg.c.x + engine::randint0((1 + dis + dis) as u32) - dis;
                let d = engine::distance(st.borg.c, loc { x, y });
                if d >= min && d <= dis {
                    break;
                }
            }

            // Ignore illegal locations
            if y <= 0 || y >= AUTO_MAX_Y - 2 {
                i += 1;
                continue;
            }
            if x <= 0 || x >= AUTO_MAX_X - 2 {
                i += 1;
                continue;
            }

            // Access
            let ag = st.cave.grid(x, y);

            // Skip unknown grids
            if u32::from(ag.feat) == FEAT_NONE {
                i += 1;
                continue;
            }

            // Skip walls
            if !borg_cave_floor_bold(st, y, x) {
                i += 1;
                continue;
            }

            // Skip monsters
            if st.cave.grid(x, y).kill != 0 {
                i += 1;
                continue;
            }

            // Stop looking.
            break;
        }

        // No location: assume that there are unknown spots that you would
        // be able to go but we define it as dangerous.
        if i >= 100 {
            n += 1;
            continue;
        }

        // Examine danger of that grid
        let p = borg_danger(st, y, x, turns, true, false);

        // if more scary than my current one, do not allow jumps at all
        if p > b_p {
            n += 1;
            continue;
        }

        // Should not land next to a monster either.
        for u in 0..8 {
            // Access the grid, obtain the monster
            let ag_kill = st
                .cave
                .grid(x + i32::from(engine::ddx_ddd(u)), y + i32::from(engine::ddy_ddd(u)))
                .kill;
            let kill = &st.kills.kills[usize::from(ag_kill)];

            // If monster adjacent to that grid...
            if ag_kill != 0 && kill.awake {
                n += 1;
            }
        }
    }

    // Too much danger: in an emergency try with extra danger allowed
    if n > emergency {
        note(st, &[format!("# No Shoot'N'Scoot. scary squares: {}/100", n).as_bytes()]);
        return false;
    } else {
        note(st, &[format!("# Safe to Shoot'N'Scoot. scary squares: {}/100", n).as_bytes()]);
    }

    // Okay
    true
}

/// Create a kill at the given location (`borg_create_kill`)
pub fn borg_create_kill(st: &mut BorgState, who: &[u8], c: loc) -> i32 {
    let r_idx = borg_guess_race_name(st, who);
    borg_new_kill(st, r_idx, c.y, c.x)
}

/// Sort a name table with the Borg's quicksort (strcmp, then index).
fn sort_names(text: Vec<Vec<u8>>, what: Vec<i16>) -> (Vec<Vec<u8>>, Vec<u32>) {
    let names: Vec<std::ffi::CString> = text
        .into_iter()
        .map(|t| std::ffi::CString::new(t).expect("no NUL inside a name"))
        .collect();
    let mut refs: Vec<&CStr> = names.iter().map(|c| c.as_c_str()).collect();
    let mut what = what;
    let size = refs.len() as i32;
    borg_sort(&mut TextWhat { text: &mut refs, what: &mut what }, size);
    let text = refs.iter().map(|c| c.to_bytes().to_vec()).collect();
    // (unsigned int)what[i]: an int16_t widened
    let what = what.iter().map(|&w| i32::from(w) as u32).collect();
    (text, what)
}

/// `borg_massage_special_chars(name)` as the C string (up to its NUL).
fn massage(name: &[u8]) -> Vec<u8> {
    let mut v = crate::borg_io::borg_massage_special_chars(name);
    let n = v.iter().position(|&b| b == 0).unwrap_or(v.len());
    v.truncate(n);
    v
}

/// The C name buffers are `char *text[1024]`.
const NAMES_MAX: usize = 1024;

/// Record the race indexes the Borg code refers to by name (each test
/// sets its own field, so the C order does not matter).
fn note_monster_id(st: &mut BorgState, name: &[u8], ridx: u32) {
    let id = &mut st.borg.id;
    if name == b"Morgoth, Lord of Darkness" {
        id.morgoth = ridx;
    }
    if name == b"Sauron, the Sorcerer" {
        id.sauron = ridx;
    }
    if name == b"The Tarrasque" {
        id.tarrasque = ridx;
    }
    if name == b"squint-eyed rogue" {
        id.squint_eyed_rogue = ridx;
    }
    if prefix(name, b"Grip") {
        id.grip = ridx;
    }
    if prefix(name, b"Fang") {
        id.fang = ridx;
    }
    let exact: [(&[u8], &mut u32); 30] = [
        (b"small kobold", &mut id.small_kobold),
        (b"soldier", &mut id.soldier),
        (b"cutpurse", &mut id.cutpurse),
        (b"acolyte", &mut id.acolyte),
        (b"apprentice", &mut id.apprentice),
        (b"kobold", &mut id.kobold),
        (b"shrieker mushroom patch", &mut id.shrieker_mushroom_patch),
        (b"Farmer Maggot", &mut id.maggot),
        (b"filthy street urchin", &mut id.filthy_street_urchin),
        (b"battle-scarred veteran", &mut id.battle_scarred_veteran),
        (b"mean-looking mercenary", &mut id.mean_looking_mercenary),
        (b"Bullroarer the Hobbit", &mut id.bullroarer),
        (b"giant white mouse", &mut id.giant_white_mouse),
        (b"white worm mass", &mut id.white_worm_mass),
        (b"green worm mass", &mut id.green_worm_mass),
        (b"cave spider", &mut id.cave_spider),
        (b"red naga", &mut id.red_naga),
        (b"giant red frog", &mut id.giant_red_frog),
        (b"radiation eye", &mut id.radiation_eye),
        (b"yellow worm mass", &mut id.yellow_worm_mass),
        (b"gravity hound", &mut id.gravity_hound),
        (b"inertia hound", &mut id.inertia_hound),
        (b"dread", &mut id.dread),
        (b"Beorn, the Shape-Changer", &mut id.beorn),
        (b"dracolich", &mut id.dracolich),
        (b"dracolisk", &mut id.dracolisk),
        (b"light hound", &mut id.light_hound),
        (b"dark hound", &mut id.dark_hound),
        (b"night mare", &mut id.night_mare),
        (b"pukelman", &mut id.pukelman),
    ];
    for (want, slot) in exact {
        if name == want {
            *slot = ridx;
        }
    }
    if stristr(name, b"Herald of Ulmo") {
        id.osse = ridx;
    }
    if name == b"drolem" {
        id.drolem = ridx;
    }
}

/// Build the sorted unique and normal name tables
/// (`borg_init_monster_names`), and note the race ids used in the code.
fn borg_init_monster_names(st: &mut BorgState) {
    let r_max = u32::from(engine::z_info().r_max.wrapping_sub(1));

    // *** Parse "unique" monster names ***
    let mut text: Vec<Vec<u8>> = Vec::new();
    let mut what: Vec<i16> = Vec::new();

    // Collect "unique" monsters
    for i in 1..r_max {
        let r = engine::race(i as usize);

        // Skip non-monsters
        if r.name.is_null() {
            continue;
        }

        // mark off any monster ids that are used in the code
        note_monster_id(st, engine::race_name(i as usize), r.ridx);

        // Skip non-unique monsters
        if !rf_has(i, RF_UNIQUE) {
            continue;
        }

        assert!(text.len() < NAMES_MAX, "C overflows text[1024]");
        text.push(massage(engine::race_name(i as usize)));
        what.push(i as i16);
    }

    // Sort, save
    let (t, w) = sort_names(text, what);
    st.kills.unique_text = t;
    st.kills.unique_what = w;

    // *** Parse "normal" monster names ***
    let mut text: Vec<Vec<u8>> = Vec::new();
    let mut what: Vec<i16> = Vec::new();

    // Collect "normal" monsters
    for i in 1..r_max {
        let r = engine::race(i as usize);

        // Skip non-monsters
        if r.name.is_null() {
            continue;
        }

        // Skip unique monsters
        if rf_has(i, RF_UNIQUE) {
            continue;
        }

        assert!(text.len() < NAMES_MAX, "C overflows text[1024]");
        text.push(massage(engine::race_name(i as usize)));
        what.push(i as i16);
        if let Some(plural) = engine::race_plural(i as usize) {
            assert!(text.len() < NAMES_MAX, "C overflows text[1024]");
            text.push(massage(plural));
            what.push(i as i16);
        }
    }

    // Sort, save
    let (t, w) = sort_names(text, what);
    st.kills.normal_text = t;
    st.kills.normal_what = w;
}

/// Free the name tables (`borg_free_monster_names`).
fn borg_free_monster_names(st: &mut BorgState) {
    st.kills.normal_what = Vec::new();
    st.kills.normal_text = Vec::new();
    st.kills.unique_what = Vec::new();
    st.kills.unique_text = Vec::new();
}

/// Set up monster tracking (`borg_init_flow_kill`).
pub fn borg_init_flow_kill(st: &mut BorgState) {
    borg_free_flow_kill(st);

    // *** Monster tracking ***

    // No monsters yet
    st.kills.cnt = 0;
    st.kills.nxt = 1;

    // Array of monsters
    st.kills.kills = vec![BorgKill::BLANK; KILLS_MAX];

    let r_max = usize::from(engine::z_info().r_max);

    // Count racial appearances
    st.kills.race_count = vec![0; r_max];

    // Count racial deaths
    st.kills.race_death = vec![0; r_max];

    // *** XXX XXX XXX Hack -- Cheat ***

    // Extract dead uniques (C bug 24: the test is inverted, so this marks
    // non-uniques with max_num 0)
    for i in 1..(r_max as i32 - 1) {
        let r = engine::race(i as usize);

        // Skip non-monsters
        if r.name.is_null() {
            continue;
        }

        // Skip non-uniques
        if rf_has(i as u32, RF_UNIQUE) {
            continue;
        }

        // Mega-Hack -- Access "dead unique" list
        if r.max_num == 0 {
            st.kills.race_death[i as usize] = 1;
        }
    }

    borg_init_monster_names(st);
}

/// Free monster tracking (`borg_free_flow_kill`).
pub fn borg_free_flow_kill(st: &mut BorgState) {
    borg_free_monster_names(st);
    st.kills.race_death = Vec::new();
    st.kills.race_count = Vec::new();
    st.kills.kills = Vec::new();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn string_helpers_match_z_util() {
        assert!(strnieq(b"something", b"Something", 9));
        assert!(strnieq(b"It", b"It", 2));
        assert!(strnieq(b"itangast", b"It", 2));
        assert!(!strnieq(b"I", b"It", 2));
        assert!(!strnieq(b"Some", b"Something", 9));
        assert!(prefix_i(b"Ancient dragons", b"ancient dragon"));
        assert!(!prefix_i(b"ancient drag", b"ancient dragon"));
        assert!(stristr(b"Osse, Herald of Ulmo", b"herald of ulmo"));
        assert!(stristr(b"hheralD OF ULMO", b"Herald of Ulmo"));
        assert!(!stristr(b"Herald of Ulm", b"Herald of Ulmo"));
        assert!(suffix(b"x (offscreen)", b" (offscreen)"));
        assert!(!suffix(b"(offscreen)", b" (offscreen)"));
    }

    #[test]
    fn search_finds_last_le() {
        let t: Vec<Vec<u8>> = ["a", "b", "b", "c"].iter().map(|s| s.as_bytes().to_vec()).collect();
        assert_eq!(search_last_le(&t, b"b"), 2);
        assert_eq!(search_last_le(&t, b"bb"), 2);
        assert_eq!(search_last_le(&t, b"0"), 0);
        assert_eq!(search_last_le(&t, b"z"), 3);
        assert_eq!(search_last_le(&[], b"z"), 0);
    }

    fn table(names: &[(&str, u32)]) -> (Vec<Vec<u8>>, Vec<u32>) {
        let text = names.iter().map(|(n, _)| n.as_bytes().to_vec()).collect();
        let what = names.iter().map(|&(_, w)| w).collect();
        (text, what)
    }

    #[test]
    fn guess_race_name_matches_c_search() {
        let mut k = KillState::default();
        (k.unique_text, k.unique_what) = table(&[("Fang, Farmer Maggot's Dog", 5), ("Grip", 4)]);
        // sorted as the Borg sorts them: by name, then race index
        (k.normal_text, k.normal_what) =
            table(&[("cave spider", 9), ("cave spiders", 9), ("kobold", 3), ("kobold", 7), ("kobolds", 3)]);
        let ghost = 99;
        // race 3 is level 2, race 7 level 10; depth 8
        let score = |i: u32| 1000 - (if i == 7 { 10 } else { 2 } - 8i32).abs();

        assert_eq!(guess_race_name(&k, b"Grip", ghost, &score), (4, vec![]));
        assert_eq!(guess_race_name(&k, b"The kobold", ghost, &score).0, 7);
        assert_eq!(guess_race_name(&k, b"the kobolds", ghost, &score).0, 3);
        assert_eq!(guess_race_name(&k, b"The cave spiders", ghost, &score).0, 9);
        let (r, notes) = guess_race_name(&k, b"kobold", ghost, &score);
        assert_eq!((r, notes), (99, vec![b"# Assuming player ghost (kobold) (a)".to_vec()]));
        let (r, notes) = guess_race_name(&k, b"The kobold (offscreen)", ghost, &score);
        assert_eq!(r, 7);
        assert_eq!(notes, vec![b"# Handling offscreen monster (The kobold)".to_vec()]);
        let (r, notes) = guess_race_name(&k, b"The orc", ghost, &score);
        assert_eq!((r, notes), (99, vec![b"# Assuming player ghost (orc)(b)".to_vec()]));
        // equal scores: the scan runs backwards, the earlier entry wins ties
        let flat = |_: u32| 1000;
        assert_eq!(guess_race_name(&k, b"The kobold", ghost, &flat).0, 3);
    }

    #[test]
    fn corridor_patterns_match_c() {
        let ones = |p: &Pattern| p.array.iter().sum::<i32>();
        assert_eq!((ones(&NORTH), ones(&SOUTH), ones(&EAST), ones(&WEST)), (13, 13, 13, 13));
        assert_eq!((NORTH.dy[7], NORTH.dx[7]), (-3, 0));
        assert_eq!((SOUTH.dy[17], SOUTH.dx[17]), (3, 0));
        assert_eq!((EAST.dy[13], EAST.dx[13]), (0, 3));
        assert_eq!((WEST.dy[11], WEST.dx[11]), (0, -3));
    }

}
