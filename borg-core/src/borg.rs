//! Port of `borg.c`: the Borg's entry point and its core state.
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

use crate::borg_io::{borg_flush, borg_inkey, borg_note, ESCAPE};
use crate::borg_headers::borg_h::{
    BORG_DUMP_LEVEL, BORG_MONEY_SCUM_AMOUNT, BORG_RESPAWN_WINNERS, BORG_SELF_SCUM,
    BORG_STOP_CLEVEL, BORG_STOP_DLEVEL, BORG_STOP_KING,
};
use crate::borg_headers::borg_trait_h::{BI_CDEPTH, BI_CLEVEL, BI_GOLD, BI_KING};
use crate::headers::ui_event_h::{keypress, EVT_DISCONNECT, EVT_KBRD, EVT_NONE};
use crate::headers::z_type_h::loc;
use crate::state::BorgState;
use crate::engine;

/// A Borg game-turn counter (`borg_time`).
pub type BorgTime = i32;

/// `KTRL(X)`
pub const fn ktrl(c: u8) -> u32 {
    (c & 0x1f) as u32
}

/// All the information the Borg knows about itself (`struct borg_struct`).
#[derive(Clone, Default)]
pub struct Borg {
    /// Indexed by `BI_*` (`borg.trait`, allocated by borg-trait.c).
    pub trait_: Vec<i32>,
    /// Indexed by object kind (`borg.has`).
    pub has: Vec<i32>,
    /// Indexed by activation (`borg.activation`).
    pub activation: Vec<i32>,
    pub power: i32,
    pub c: loc,
    pub panels: loc,
    pub avoidance: i16,
    pub oldchp: i16,
    pub lunal_mode: bool,
    pub munchkin_mode: bool,
    pub stair_less: bool,
    pub stair_more: bool,
    pub in_shop: bool,
    pub ready_morgoth: i32,
    pub temp: BorgTemp,
    pub status: BorgStatus,
    pub time: BorgTimes,
    pub need_see_invis: BorgTime,
    pub see_inv: i16,
    pub need_shift_panel: bool,
    pub when_shift_panel: BorgTime,
    pub antibounce_count: i16,
    pub no_retreat: i16,
    pub resistance: i16,
    /// `no_rest_prep`: C `int`; time left before resting after a prep spell.
    pub no_rest_prep: i32,
    pub times_twitch: i16,
    pub escapes: i16,
    pub trying_unknown: bool,
    /// `borg.dont_react`: the reaction gate (set while knocked out or
    /// paralyzed, cleared on every message pump).
    pub dont_react: bool,
    pub targeting: bool,
    pub goal: BorgGoal,
    pub init_save: BorgInitSave,
    pub mon: BorgMonInfo,
    pub id: BorgIdInfo,
    pub near: BorgNear,
    pub morgoth_position: bool,
    pub amt_book: [i16; 9],
    pub book_idx: [i16; 9],
    pub need_statgain: [bool; 5],
    pub amt_statgain: [i16; 5],
}

#[derive(Copy, Clone, Default)]
#[repr(C)]
pub struct BorgBest {
    pub home: bool,
    pub tval: u8,
    pub sval: u8,
    pub pval: i16,
}

#[derive(Copy, Clone, Default)]
pub struct BorgTemp {
    // C fields that no C code reads either (kept as in `borg_temp`).
    #[allow(dead_code)]
    pub need_see_invis: i16,
    #[allow(dead_code)]
    pub see_inv: i16,
    pub res_fire: bool,
    pub res_cold: bool,
    pub res_acid: bool,
    pub res_elec: bool,
    pub res_pois: bool,
    pub prot_from_evil: bool,
    pub fast: bool,
    pub bless: bool,
    pub hero: bool,
    pub berserk: bool,
    pub fastcast: bool,
    pub regen: bool,
    pub smite_evil: bool,
    pub venom: bool,
    pub shield: bool,
}

#[derive(Copy, Clone, Default)]
pub struct BorgStatus {
    pub active: bool,
    pub cancel: bool,
    pub save: bool,
    pub old_depth: i16,
    pub respawning: i16,
    pub cheat_death: bool,
    pub anti_summon: bool,
    pub digging: bool,
    pub need_alter: bool,
    pub no_alter: bool,
    pub redraw: bool,
    pub vault: bool,
    pub desperate: bool,
}

#[derive(Copy, Clone, Default)]
pub struct BorgTimes {
    pub now: BorgTime,
    pub level: BorgTime,
    pub town: BorgTime,
    pub morgoth: BorgTime,
    pub antisummon: BorgTime,
    pub call_light: BorgTime,
    pub wizard_light: BorgTime,
    pub detect_traps: BorgTime,
    pub detect_doors: BorgTime,
    pub detect_walls: BorgTime,
    pub detect_evil: BorgTime,
    pub detect_obj: BorgTime,
    pub last_kill_mult: BorgTime,
}

#[derive(Clone, Default)]
pub struct BorgGoal {
    pub type_0: i16,
    pub g: loc,
    pub rising: bool,
    pub leaving: bool,
    pub fleeing: bool,
    pub fleeing_lunal: bool,
    pub fleeing_munchkin: bool,
    pub fleeing_to_town: bool,
    pub ignoring: bool,
    pub less: bool,
    pub waiting: bool,
    pub recalling: i32,
    pub descending: i32,
    pub shop: i16,
    pub ware: i16,
    pub item: i16,
    pub do_best: bool,
    /// `z_info->equip_slots_max` entries while a "best" list is pending
    /// (borg-item-wear.c); empty = NULL.
    pub best_item: Vec<BorgBest>,
}

/// The player's settings saved while the Borg runs (borg-init.c).
#[derive(Clone, Default)]
pub struct BorgInitSave {
    /// The user's keymap mode (`KEYMAP_MODE_*`).
    pub key_mode: i32,
    /// `k_info[i].ignore` for every object kind; empty = NULL.
    pub kinfo_ignore: Vec<u8>,
    pub ignore_level: [u8; engine::ITYPE_MAX],
    /// `ego_ignore_types[i]` for every ego item; empty = NULL.
    pub ego_ignore_types: Vec<[bool; engine::ITYPE_MAX]>,
}

#[derive(Copy, Clone, Default)]
pub struct BorgMonInfo {
    pub breeders: u16,
    pub scary: bool,
    pub morgoth_panel: loc,
    pub unique: u32,
}

#[derive(Copy, Clone, Default)]
pub struct BorgIdInfo {
    pub morgoth: u32,
    pub sauron: u32,
    pub tarrasque: u32,
    pub maggot: u32,
    pub bullroarer: u32,
    pub beorn: u32,
    pub grip: u32,
    pub fang: u32,
    pub osse: u32,
    pub squint_eyed_rogue: u32,
    pub small_kobold: u32,
    pub soldier: u32,
    pub cutpurse: u32,
    pub acolyte: u32,
    pub apprentice: u32,
    pub kobold: u32,
    pub shrieker_mushroom_patch: u32,
    pub filthy_street_urchin: u32,
    pub battle_scarred_veteran: u32,
    pub mean_looking_mercenary: u32,
    pub giant_white_mouse: u32,
    pub white_worm_mass: u32,
    pub green_worm_mass: u32,
    pub cave_spider: u32,
    pub red_naga: u32,
    pub giant_red_frog: u32,
    pub radiation_eye: u32,
    pub yellow_worm_mass: u32,
    pub gravity_hound: u32,
    pub inertia_hound: u32,
    pub dread: u32,
    pub dracolich: u32,
    pub dracolisk: u32,
    pub light_hound: u32,
    pub dark_hound: u32,
    pub pukelman: u32,
    pub night_mare: u32,
    pub drolem: u32,
}

#[derive(Copy, Clone, Default)]
pub struct BorgNear {
    pub morgoth: bool,
    pub breeder: bool,
    pub unique: i16,
    pub evil_unique: bool,
    pub summoner: bool,
    pub summoner_idx: i16,
}

impl Borg {
    /// `borg.trait[i]`
    pub fn tr(&self, i: u32) -> i32 {
        self.trait_[i as usize]
    }
}

/// borg.c's other globals.
pub struct CoreState {
    /// Configuration from borg.txt (`borg_cfg`), indexed by `BORG_*`.
    pub cfg: Vec<i32>,
    /// The Borg's own RNG state, swapped in around `borg_think()`
    /// (`borg_rand_local`).
    pub rand_local: u32,
    /// `borg_engine_date`: `__DATE__` of the C build this was transpiled
    /// from (shown by the version command only).
    pub engine_date: &'static str,
    /// Steps left before a clean cancel (`borg_step`).
    pub step: u16,
    /// Current panel offset (`w_x`, `w_y`).
    pub w_x: i32,
    pub w_y: i32,
}

impl Default for CoreState {
    fn default() -> Self {
        CoreState {
            cfg: Vec::new(),
            rand_local: 0,
            engine_date: "Nov  4 2026",
            step: 0,
            w_x: 0,
            w_y: 0,
        }
    }
}

impl BorgState {
    /// `borg_cfg[i]`
    pub fn cfg(&self, i: u32) -> i32 {
        self.core.cfg[i as usize]
    }
}

/// The `inkey_hack` callback: capture control from the game and feed it
/// keypresses (`borg_entry_point`).
extern "C" fn borg_entry_point(flush_first: core::ffi::c_int) -> keypress {
    crate::engine::enter(|st| {
        let k = generate_keypress(st, flush_first != 0);
        crate::borg_io::borg_save_keypress(st, k)
    })
}

/// Install or remove the keypress hook (`borg_update_entrypoint`).
pub fn borg_update_entrypoint(start: bool) {
    engine::set_inkey_hack(if start { Some(borg_entry_point) } else { None });
}

/// Check for Borg deactivation or player death
/// (`check_for_deactivate_or_death`). May set the key to return.
fn check_for_deactivate_or_death(st: &mut BorgState, key: &mut keypress) -> bool {
    if !st.borg.status.active {
        borg_note(st, "# Removing keypress hook");
        engine::set_inkey_hack(None);
        borg_flush(st);
        engine::flush_input();
        crate::borg_init::borg_reset_settings(st);
        key.code = ESCAPE;
        return true;
    }

    if engine::player_is_dead() && st.borg.status.respawning == 0 {
        let died_from = engine::player_died_from();
        if st.borg.tr(BI_CLEVEL) >= st.cfg(BORG_DUMP_LEVEL)
            || died_from.windows(10).any(|w| w == b"starvation")
        {
            crate::borg_log::borg_write_map(st, false);
        }
        crate::borg_log::borg_log_death(st);
        crate::borg_log::borg_log_death_data(st);
        borg_flush(st);
        crate::borg_messages::borg_parse(st, None);
        crate::borg_messages_react::borg_clear_reactions(st);
        crate::borg_think::borg_oops(st, "player died");
        key.code = ktrl(b'C');
        return true;
    }

    if st.borg.tr(BI_KING) != 0 {
        if st.cfg(BORG_STOP_KING) != 0 {
            crate::borg_log::borg_write_map(st, false);
            crate::borg_think::borg_oops(st, "retire");
        }
        if st.cfg(BORG_RESPAWN_WINNERS) != 0 {
            crate::borg_log::borg_write_map(st, false);
            crate::borg_log::borg_log_death(st);
            crate::borg_log::borg_log_death_data(st);
            borg_flush(st);
            crate::borg_reincarnate::borg_force_reincarnate(st);
            return false;
        }
    }

    if st.borg.tr(BI_CDEPTH) == st.cfg(BORG_STOP_DLEVEL) {
        crate::borg_think::borg_oops(st, "Auto-stop for user DLevel.");
        key.code = ktrl(b'C');
        return true;
    }

    if st.borg.tr(BI_CLEVEL) == st.cfg(BORG_STOP_CLEVEL) {
        crate::borg_think::borg_oops(st, "Auto-stop for user CLevel.");
        key.code = ktrl(b'C');
        return true;
    }

    // HACK to end all hacks, allow the borg to stop if money scumming
    if st.borg.tr(BI_GOLD) > st.cfg(BORG_MONEY_SCUM_AMOUNT)
        && st.cfg(BORG_MONEY_SCUM_AMOUNT) != 0
        && st.borg.tr(BI_CDEPTH) == 0
        && st.cfg(BORG_SELF_SCUM) == 0
    {
        crate::borg_think::borg_oops(st, "Money Scum complete.");
        key.code = ktrl(b'C');
        return true;
    }

    false
}

/// Stop if the user pressed a key (`user_abort`).
fn user_abort(st: &mut BorgState) -> bool {
    let (ev_type, ev_key) = engine::term_inkey(false, true);
    if !st.borg.in_shop
        && ((ev_type & EVT_KBRD != 0 && ev_key.code > 0 && ev_key.code != 10)
            || ev_type == EVT_DISCONNECT)
    {
        if ev_type == EVT_DISCONNECT {
            crate::borg_think::borg_oops(st, "terminal disconnect abort");
        } else {
            let k = ev_key.code;
            if (32..=126).contains(&k) {
                borg_note(st, &format!("# User key press <{}><{}>", k, k as u8 as char));
            } else {
                borg_note(st, &format!("# User key press <{}>", k));
            }
            // "%c" of the event type: the low byte as a char
            let mut msg = format!("# Key type was <{}><", ev_type).into_bytes();
            msg.push(ev_type as u8);
            msg.push(b'>');
            crate::borg_io::borg_note_bytes(st, &msg);
            crate::borg_think::borg_oops(st, "user abort");
        }
        return true;
    }
    false
}

/// Return one keypress to the game (`generate_keypress`).
fn generate_keypress(st: &mut BorgState, flush_first: bool) -> keypress {
    let mut key = keypress { type_0: EVT_KBRD, code: 0, mods: 0 };

    let (cx, cy) = engine::term_locate();
    let cursor = loc { x: cx, y: cy };
    engine::term_fresh();

    // Analyze the player's current stats (before any check that uses them)
    crate::borg_trait::borg_notice_player(st);

    if check_for_deactivate_or_death(st, &mut key) {
        return key;
    }

    if flush_first && borg_inkey(st, false) != 0 {
        borg_note(st, "# Flushing keypress buffer");
        borg_flush(st);
    }

    // Messages at the top of the screen; some need an answer right away
    if crate::borg_messages::borg_get_messages(st, &mut key, cursor) {
        return key;
    }

    crate::borg_messages::borg_parse(st, None);
    st.borg.dont_react = false;

    let borg_ch = borg_inkey(st, true);
    if borg_ch != 0 {
        key.code = borg_ch;
        return key;
    }

    if user_abort(st) {
        key.code = ESCAPE;
        return key;
    }

    // Keep him active in town
    if st.borg.tr(BI_CDEPTH) >= 1 {
        st.borg.in_shop = false;
    }

    // Don't interrupt our own resting or a repeating command
    if engine::player_resting() != 0 || engine::cmd_get_nrepeats() > 0 {
        key.type_0 = EVT_NONE;
        return key;
    }

    if st.borg.status.respawning != 0 {
        if !engine::player_is_dead() {
            crate::borg_reincarnate::borg_reincarnate_end(st);
            st.borg.status.respawning = 0;
        } else {
            key.code = ESCAPE;
            return key;
        }
    }

    st.borg.targeting = false;

    // Swap in the Borg's own RNG around thinking
    let (rand_quick, rand_value) = engine::rand_state();
    engine::set_rand_state(true, st.core.rand_local);

    #[cfg(feature = "test-force-nested")]
    force_nested_entry(st);

    while !crate::borg_think::borg_think(st) {}

    crate::borg_log::borg_display_status(st);

    st.core.rand_local = engine::rand_state().1;
    engine::set_rand_state(rand_quick, rand_value);

    // Allow stepping to induce a clean cancel
    if st.core.step != 0 {
        st.core.step -= 1;
        if st.core.step == 0 {
            st.borg.status.cancel = true;
        }
    }

    let borg_ch = borg_inkey(st, true);
    if borg_ch != 0 {
        key.code = borg_ch;
        return key;
    }

    crate::borg_think::borg_oops(st, "normal abort");
    key.code = ESCAPE;
    key
}

/// Test only (`test-force-nested`, harness/regress/nested): every 500th
/// think step shows two 70-byte messages. With `auto_more` off the second
/// one makes the engine flush and wait for a key, which calls the Borg's
/// input hook again while this turn runs: a nested entry (see
/// `engine::enter` and docs/SELF-REVIEW.md U1).
#[cfg(feature = "test-force-nested")]
fn force_nested_entry(st: &mut BorgState) {
    use std::sync::atomic::{AtomicU32, Ordering};
    static STEPS: AtomicU32 = AtomicU32::new(0);
    if STEPS.fetch_add(1, Ordering::Relaxed) % 500 == 499 {
        engine::msg(st, &[b'x'; 70]);
        engine::msg(st, &[b'y'; 70]);
    }
}
