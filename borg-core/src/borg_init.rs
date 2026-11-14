//! Port of `borg-init.c`: full initialization of all borg variables.
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

use std::ffi::CStr;

use crate::borg_io::{borg_note, borg_warning};
use crate::cctype::{c_isdigit, c_toupper};
use crate::borg_headers::borg_h::{
    BORG_ALLOW_STRANGE_OPTS, BORG_CHEAT_DEATH, BORG_ENCHANT_LIMIT, BORG_LUNAL_MODE,
    BORG_MAX_SETTINGS, BORG_MUNCHKIN_BY_CLASS, BORG_MUNCHKIN_CLASS_DEPTH, BORG_MUNCHKIN_DEPTH,
    BORG_MUNCHKIN_LEVEL, BORG_MUNCHKIN_START, BORG_RESPAWN_CLASS,
    BORG_RESPAWN_RACE, BORG_RESPAWN_WINNERS, BORG_RESTORE_IGNORE_SETTINGS, BORG_STOP_KING,
    BORG_USES_DYNAMIC_CALCS,
};
use crate::headers::game_event_h::{
    game_event_data, game_event_type, EVENT_LEAVE_GAME, EVENT_MESSAGE_FLUSH,
};
use crate::headers::list_options_h::{
    OPT_birth_connect_stairs, OPT_birth_force_descend, OPT_birth_no_recall,
    OPT_birth_percent_damage, OPT_cheat_live, OPT_rogue_like_commands,
};
use crate::headers::ui_keymap_h::{KEYMAP_MODE_ORIG, KEYMAP_MODE_ROGUE};
use crate::state::BorgState;
use crate::engine;

/// `MAX_RACES` (borg-trait.h).
const MAX_RACES: i32 = 11;
/// `MAX_CLASSES` (borg-trait.h).
const MAX_CLASSES: i32 = 9;
/// `PR_MAP` (player-calcs.h).
const PR_MAP: u32 = 0x10000;
/// The size of the line buffer of `borg_init_txt_file()` (`char buf[1024]`).
const TXT_BUF: usize = 1024;

/// borg-init.c's globals.
#[derive(Default)]
pub struct InitState {
    /// `borg_initialized`
    pub initialized: bool,
    /// `game_closed`: has the game been closed since the borg was
    /// initialized
    pub game_closed: bool,
    /// `borg_init_failure`
    pub init_failure: bool,
    /// `borg_munchkin_start_setting`: the munchkin start as borg.txt gave
    /// it, before the class rule
    pub munchkin_start_setting: i32,
    /// `borg_munchkin_depth_setting`
    pub munchkin_depth_setting: i32,
}

/// One borg.txt setting (`struct borg_setting`).
pub struct BorgSetting {
    /// `setting_string`
    pub name: &'static CStr,
    /// `setting_type`: `b'b'` (bool) or `b'i'` (int)
    pub kind: u8,
    /// `default_value`
    pub default_value: i32,
}

const fn b(name: &'static CStr, default_value: bool) -> BorgSetting {
    BorgSetting {
        name,
        kind: b'b',
        default_value: default_value as i32,
    }
}

const fn i(name: &'static CStr, default_value: i32) -> BorgSetting {
    BorgSetting {
        name,
        kind: b'i',
        default_value,
    }
}

/// Borg settings information (`borg_settings`, without the `{0, 0, 0}`
/// entry that ends the C table). Must match the `BORG_*` enum of borg.h.
pub const BORG_SETTINGS: [BorgSetting; BORG_MAX_SETTINGS as usize] = [
    b(c"borg_verbose", false),
    b(c"borg_munchkin_start", false),
    i(c"borg_munchkin_level", 12),
    i(c"borg_munchkin_depth", 16),
    b(c"borg_worships_damage", false),
    b(c"borg_worships_speed", false),
    b(c"borg_worships_hp", false),
    b(c"borg_worships_mana", false),
    b(c"borg_worships_ac", false),
    b(c"borg_worships_gold", false),
    b(c"borg_plays_risky", false),
    b(c"borg_kills_uniques", false),
    b(c"borg_uses_swaps", true),
    b(c"borg_uses_dynamic_calcs", false),
    i(c"borg_stop_dlevel", 128),
    i(c"borg_stop_clevel", 51),
    i(c"borg_no_deeper", 127),
    b(c"borg_stop_king", true),
    b(c"borg_cheat_death", false),
    b(c"borg_respawn_winners", false),
    i(c"borg_respawn_class", -1),
    i(c"borg_respawn_race", -1),
    i(c"borg_chest_fail_tolerance", 7),
    i(c"borg_delay_factor", 0),
    i(c"borg_money_scum_amount", 0),
    b(c"borg_self_scum", true),
    b(c"borg_lunal_mode", false),
    b(c"borg_self_lunal", false),
    i(c"borg_enchant_limit", 12),
    i(c"borg_dump_level", 1),
    i(c"borg_save_death", 1),
    b(c"borg_stop_on_bell", false),
    b(c"borg_allow_strange_opts", false),
    b(c"borg_autosave", false),
    b(c"borg_restore_ignore_settings", false),
    b(c"borg_munchkin_by_class", false),
    i(c"borg_munchkin_class_depth", 6),
    i(c"borg_buy_strategy_default", 0),
    i(c"borg_buy_strategy_munchkin", -1),
    i(c"borg_buy_strategy_current_classes", 0),
    i(c"borg_buy_strategy_reserve410_classes", 0),
    i(c"borg_buy_strategy_current_munchkin_classes", 0),
    i(c"borg_buy_strategy_reserve410_munchkin_classes", 0),
    i(c"borg_buy_identify", 0),
];

/// Class indexes of borg-trait.h used by the munchkin class rule.
const CLASS_WARRIOR: u32 = 0;
const CLASS_DRUID: u32 = 2;
const CLASS_RANGER: u32 = 7;

/// `borg_munchkin_class_settings()`: Warriors, Druids and Rangers survive
/// their first levels more often with a munchkin start. When borg.txt
/// leaves the munchkin start off and `borg_munchkin_by_class` is on, these
/// classes use it anyway, stair-scumming at `borg_munchkin_class_depth`.
/// Every other class plays as borg.txt says. This runs again after a
/// reincarnation, since the class may change. `cidx` is
/// `player->class->cidx` (`None` while there is no player class).
/// (better-borg-c milestone8, owner change PR302.)
pub fn borg_munchkin_class_settings(st: &mut BorgState, cidx: Option<u32>) {
    let Some(cidx) = cidx else {
        return;
    };
    if st.core.cfg.is_empty() {
        return;
    }

    let (start, depth) = (st.init.munchkin_start_setting, st.init.munchkin_depth_setting);
    let cfg = &mut st.core.cfg;
    cfg[BORG_MUNCHKIN_START as usize] = start;
    cfg[BORG_MUNCHKIN_DEPTH as usize] = depth;

    if start != 0 || cfg[BORG_MUNCHKIN_BY_CLASS as usize] == 0 {
        return;
    }

    if cidx == CLASS_WARRIOR || cidx == CLASS_DRUID || cidx == CLASS_RANGER {
        cfg[BORG_MUNCHKIN_START as usize] = 1;
        cfg[BORG_MUNCHKIN_DEPTH as usize] = cfg[BORG_MUNCHKIN_CLASS_DEPTH as usize];
    }
}

/// `prefix_i(s, t)` (z-util.c): does `s` start with `t`, ignoring case?
/// `s` is a C string without its NUL (past its end C reads the NUL, which
/// matches no byte of `t`).
fn prefix_i(s: &[u8], t: &[u8]) -> bool {
    t.iter()
        .enumerate()
        .all(|(k, &c)| c_toupper(c) == c_toupper(s.get(k).copied().unwrap_or(0)))
}

/// `atoi(s)` (glibc: `(int) strtol(s, NULL, 10)`).
fn atoi(s: &[u8]) -> i32 {
    crate::borg_formulas::c_strtol(s).0 as i32
}

/// Read a line from the config file and parse the setting value, if there
/// is one (`borg_proc_setting`). `line` is the C string without its NUL.
/// Returns whether the line names the setting.
///
/// For an int setting, a byte after the name that is not ` `, `=`, `-` or
/// a digit ends the parse and keeps the default, and a `-` only negates a
/// number that follows it (milestone6 patches 0057/0058; before them the C
/// loop, and this one, spun forever there; c-bugs.md section 38).
fn borg_proc_setting(
    cfg: &mut [i32],
    setting: usize,
    setting_string: &[u8],
    kind: u8,
    line: &[u8],
) -> bool {
    let at = |p: usize| line.get(p).copied().unwrap_or(0);
    let mut negative = false;
    let mut p = 0;

    // skip leading space
    while at(p) == b' ' {
        p += 1;
    }
    if !prefix_i(&line[p..], setting_string) {
        return false;
    }

    p += setting_string.len();

    // accept either
    // value true or
    // value=true or
    // value=1 or
    // value y
    while at(p) != 0 {
        let ch = at(p);
        if ch == b' ' || ch == b'=' {
            p += 1;
            continue;
        }

        if kind == b'b' {
            cfg[setting] = i32::from(matches!(ch, b'T' | b't' | b'1' | b'Y' | b'y'));
            break;
        }
        if kind == b'i' {
            if ch == b'-' {
                negative = true;
                p += 1;
                continue;
            }
            if c_isdigit(ch) {
                cfg[setting] = atoi(&line[p..]);
                if negative {
                    // C `*= -1` (int overflow on INT_MIN; wraps)
                    cfg[setting] = cfg[setting].wrapping_mul(-1);
                }
                break;
            }
        }

        // anything else is not a value: keep the default
        break;
    }

    true
}

/// Release resources allocated by `borg_init_txt_file()`
/// (`borg_free_txt_file`).
fn borg_free_txt_file(st: &mut BorgState) {
    crate::borg_trait::borg_trait_free(st);
    crate::borg_formulas::borg_free_formulas(st);

    st.core.cfg = Vec::new();
}

/// What the borg.txt parser reads from and writes to: the open file, the
/// formula loader and the outputs. The game uses [`TxtFile`]; the unit
/// tests use a fake.
pub(crate) trait TxtEnv {
    fn st(&mut self) -> &mut BorgState;
    /// `file_getl(fp, buf, sizeof(buf) - 1)` with a `TXT_BUF` buffer.
    fn getl(&mut self, buf: &mut [u8; TXT_BUF]) -> bool;
    /// `borg_load_formulas(fp)`
    fn load_formulas(&mut self);
    /// `borg_warning(what)`
    fn warning(&mut self, what: &str);
    /// `Term_fresh()`
    fn term_fresh(&mut self);
    /// `player->class->cidx` (`None` while there is no player class)
    fn class_cidx(&mut self) -> Option<u32>;
}

/// The borg.txt of the game (`None` once closed).
struct TxtFile<'a> {
    st: &'a mut BorgState,
    fp: Option<engine::ReadFile>,
}

impl TxtEnv for TxtFile<'_> {
    fn st(&mut self) -> &mut BorgState {
        self.st
    }
    fn getl(&mut self, buf: &mut [u8; TXT_BUF]) -> bool {
        self.fp
            .as_mut()
            .expect("borg.txt is open")
            .getl(buf, TXT_BUF - 1)
    }
    fn load_formulas(&mut self) {
        let fp = self.fp.as_mut().expect("borg.txt is open");
        crate::borg_formulas::borg_load_formulas_from(self.st, |buf, len| fp.getl(buf, len));
    }
    fn warning(&mut self, what: &str) {
        borg_warning(self.st, what);
    }
    fn term_fresh(&mut self) {
        engine::term_fresh();
    }
    fn class_cidx(&mut self) -> Option<u32> {
        engine::class_cidx_if_set()
    }
}

/// The default borg.txt that `borg_init_txt_file()` writes when there is
/// none (each piece is one `file_putf()`).
pub(crate) fn default_txt_file() -> Vec<Vec<u8>> {
    let mut out: Vec<Vec<u8>> = vec![
        b"# BORG.txt default settings \n".to_vec(),
        b"# A more descriptive version of this file is delivered. \n".to_vec(),
        b"# Check your original install for borg.txt and \n".to_vec(),
        b"# replace this one with the delivered version. \n\n".to_vec(),
    ];
    for s in &BORG_SETTINGS {
        let name = s.name.to_str().expect("ASCII setting name");
        if s.kind == b'b' {
            let v = if s.default_value != 0 {
                "TRUE"
            } else {
                "FALSE"
            };
            out.push(format!("{name} = {v}\n").into_bytes());
        }
        if s.kind == b'i' {
            out.push(format!("{name} = {}\n", s.default_value).into_bytes());
        }
    }
    out
}

/// The parse loop of `borg_init_txt_file()`: start `borg_cfg` with the
/// defaults and read the settings. Returns `(formulas_found,
/// old_formulas_found)`.
pub(crate) fn read_txt_settings<E: TxtEnv>(env: &mut E) -> (bool, bool) {
    let mut formulas_found = false;
    let mut old_formulas_found = false;
    let n = BORG_SETTINGS.len();

    // allocate the config data and start it with the default values
    env.st().core.cfg = BORG_SETTINGS.iter().map(|s| s.default_value).collect();

    let mut buf = [0u8; TXT_BUF];

    // Parse the file
    while env.getl(&mut buf) {
        // Skip comments and blank lines
        if buf[0] == 0 || buf[0] == b'#' {
            continue;
        }

        // Chop the buffer
        buf[TXT_BUF - 1] = 0;
        let len = buf.iter().position(|&c| c == 0).expect("NUL-terminated");
        let line = &buf[..len];

        let mut i = 0;
        while i < n {
            let s = &BORG_SETTINGS[i];
            if borg_proc_setting(&mut env.st().core.cfg, i, s.name.to_bytes(), s.kind, line) {
                break;
            }
            i += 1;
        }
        if i != n {
            continue;
        }

        // other settings
        if prefix_i(line, b"REQ") {
            old_formulas_found = true;
            continue;
        }
        if prefix_i(line, b"FORMULA") {
            old_formulas_found = true;
            continue;
        }
        if prefix_i(line, b"CND") {
            old_formulas_found = true;
            continue;
        }
        if prefix_i(line, b"POWER") {
            old_formulas_found = true;
            continue;
        }

        if prefix_i(line, b"[BEGIN FORMULA SECTION]") {
            formulas_found = true;
            env.load_formulas();
        }
    }

    (formulas_found, old_formulas_found)
}

/// The end of `borg_init_txt_file()`, after the file is closed: the
/// dynamic calculation warnings and the range checks. Returns whether a
/// warning was given.
pub(crate) fn finish_txt_settings<E: TxtEnv>(
    env: &mut E,
    formulas_found: bool,
    old_formulas_found: bool,
) -> bool {
    let mut warning = false;
    let dyn_calcs = BORG_USES_DYNAMIC_CALCS as usize;

    if env.st().core.cfg[dyn_calcs] != 0 {
        warning = true;
        if old_formulas_found {
            env.warning(
                "** Borg's dynamic calculations enabled but old formulas found in borg.txt.",
            );
            env.warning("** formulas disabled ** ");
            env.st().core.cfg[dyn_calcs] = 0;
        } else if !formulas_found {
            env.warning(
                "** Borg's dynamic calculations enabled but no formulas found in borg.txt.",
            );
            env.warning("** formulas disabled ** ");
            env.st().core.cfg[dyn_calcs] = 0;
        } else {
            env.warning(
                "Borg's dynamic calculations enabled.  You may see some performance loss (~20 percent).",
            );
        }

        // Flush it
        env.term_fresh();
    }

    let st = env.st();
    let cfg = &mut st.core.cfg;

    // lunal mode is a default rather than a setting
    st.borg.lunal_mode = cfg[BORG_LUNAL_MODE as usize] != 0;

    // a few sanity range checks (C: `<= lo` then `>= hi` tests)
    let level = BORG_MUNCHKIN_LEVEL as usize;
    cfg[level] = cfg[level].clamp(1, 50);

    let depth = BORG_MUNCHKIN_DEPTH as usize;
    if cfg[depth] <= 1 {
        cfg[depth] = 8;
    }
    if cfg[depth] >= 100 {
        cfg[depth] = 100;
    }

    let class_depth = BORG_MUNCHKIN_CLASS_DEPTH as usize;
    if cfg[class_depth] <= 1 {
        cfg[class_depth] = 8;
    }
    if cfg[class_depth] >= 100 {
        cfg[class_depth] = 100;
    }

    st.init.munchkin_start_setting = cfg[BORG_MUNCHKIN_START as usize];
    st.init.munchkin_depth_setting = cfg[depth];
    let cidx = env.class_cidx();
    let st = env.st();
    borg_munchkin_class_settings(st, cidx);
    let cfg = &mut st.core.cfg;

    let enchant = BORG_ENCHANT_LIMIT as usize;
    cfg[enchant] = cfg[enchant].clamp(8, 15);

    let race = BORG_RESPAWN_RACE as usize;
    if cfg[race] >= MAX_RACES || cfg[race] < -1 {
        cfg[race] = 0;
    }

    let class = BORG_RESPAWN_CLASS as usize;
    if cfg[class] >= MAX_CLASSES || cfg[class] < -1 {
        cfg[class] = 0;
    }

    // make sure it continues to run if reset
    if cfg[BORG_RESPAWN_WINNERS as usize] != 0 {
        cfg[BORG_STOP_KING as usize] = 0;
    }

    // Post-M10 family settings are Rust-only behaviour. The C parser carries
    // the same fields so the default `current` control keeps identical input.
    if let Some(error) = crate::borg_buy_strategy::config_error(cfg) {
        env.warning(error);
        env.st().init.init_failure = true;
    }

    // Success
    warning
}

/// Initialize borg.txt (`borg_init_txt_file`). Returns whether a warning
/// was given.
///
/// The config starts with the default values before borg.txt is opened, so
/// a borg.txt that is missing and cannot be written, or cannot be read back,
/// leaves the defaults in place (milestone6 patch 0059; before it, C read
/// through a NULL `borg_cfg` and the first Rust `cfg` read panicked;
/// c-bugs.md section 38).
pub fn borg_init_txt_file(st: &mut BorgState) -> bool {
    borg_free_txt_file(st);
    crate::borg_trait::borg_trait_init(st);

    // a couple of spot checks on settings definitions
    if BORG_SETTINGS[BORG_MUNCHKIN_LEVEL as usize].name != c"borg_munchkin_level"
        || BORG_SETTINGS[BORG_RESPAWN_RACE as usize].name != c"borg_respawn_race"
    {
        engine::msg(st, b"borg settings structures not correct.  aborting. ");
        st.init.init_failure = true;
    }

    // allocate the config data and start it with the default values, so
    // every early return below leaves the defaults in place
    st.core.cfg = BORG_SETTINGS.iter().map(|s| s.default_value).collect();

    let path = engine::path_build_user(TXT_BUF, b"borg.txt");

    // Open the file
    let mut fp = engine::file_open_read(&path);

    // No file, use defaults
    if fp.is_none() {
        // Complain
        engine::msg(st, b"*****WARNING***** You do not have a proper BORG.TXT file!");
        engine::msg(st, b"writing borg.txt to the \\user\\ subdirectory with default values");
        engine::msg(st, &[b"Which is: ".as_slice(), &path].concat());
        engine::event_signal(st, EVENT_MESSAGE_FLUSH);

        let Some(mut out) = engine::file_open(&path, engine::WriteMode::Write) else {
            borg_warning(
                st,
                "*****WARNING***** unable to write default BORG.TXT file!",
            );
            return true;
        };
        for piece in default_txt_file() {
            // file_putf(fp, ...): file_put of the formatted text
            out.put(&piece);
        }
        out.close();
        fp = engine::file_open_read(&path);
        if fp.is_none() {
            borg_warning(st, "*****WARNING***** unable to read default BORG.TXT file!");
            return true;
        }
    }

    let fp = fp.expect("borg.txt opened above");
    let mut file = TxtFile { st, fp: Some(fp) };
    let (formulas_found, old_formulas_found) = read_txt_settings(&mut file);

    // Close it
    if let Some(fp) = file.fp.take() {
        fp.close();
    }

    finish_txt_settings(&mut file, formulas_found, old_formulas_found)
}

/// Restore the player's ignore settings saved by `borg_reinit_options()`
/// (if `borg_restore_ignore_settings` is on) and drop the saved copy
/// (`borg_reset_ignore`).
pub fn borg_reset_ignore(st: &mut BorgState) {
    let e_max = usize::from(engine::z_info().e_max);

    if st.cfg(BORG_RESTORE_IGNORE_SETTINGS) != 0 {
        let save = &st.borg.init_save;
        if !save.kinfo_ignore.is_empty() {
            // Reset ignore bits
            for (k, &v) in save.kinfo_ignore.iter().enumerate() {
                engine::set_kind_ignore(k, v);
            }
        }

        // Clear the ignore bytes
        engine::set_ignore_level(&save.ignore_level);

        // Clear ego ignore
        if !save.ego_ignore_types.is_empty() {
            for (e, types) in save.ego_ignore_types.iter().enumerate() {
                engine::set_ego_ignore_types(e, types);
            }
        }
    }

    // C frees `ego_ignore_types[i]` for every ego, reading through the
    // array pointer even when it is NULL
    assert!(
        !st.borg.init_save.ego_ignore_types.is_empty() || e_max == 0,
        "borg_reset_ignore() without saved ignore settings (C reads through NULL)"
    );
    st.borg.init_save.ego_ignore_types = Vec::new();
    st.borg.init_save.kinfo_ignore = Vec::new();
}

/// Save the player's ignore settings and reset them (`borg_init_ignore`).
fn borg_init_ignore(st: &mut BorgState) {
    let save = &mut st.borg.init_save;

    // (C allocates new arrays here; a previous copy is leaked)
    // Reset ignore bits
    save.kinfo_ignore = (0..engine::k_max()).map(|k| engine::kind(k).ignore).collect();

    // Clear the ignore bytes
    save.ignore_level = engine::ignore_level();

    // Clear ego ignore
    let e_max = usize::from(engine::z_info().e_max);
    save.ego_ignore_types = (0..e_max).map(engine::ego_ignore_types).collect();

    // clear the saved flags
    engine::ignore_birth_init();
}

/// Reset the required options when returning from user control
/// (`borg_reinit_options`).
pub fn borg_reinit_options(st: &mut BorgState) {
    // Save current key mode
    st.borg.init_save.key_mode = if engine::player_opt(OPT_rogue_like_commands) {
        KEYMAP_MODE_ROGUE as i32
    } else {
        KEYMAP_MODE_ORIG as i32
    };

    borg_init_ignore(st);

    // The Borg uses the original keypress codes
    engine::option_set(c"rogue_like_commands", false);

    // No auto_more
    engine::option_set(c"auto_more", false);

    // We pick up items when we step on them
    engine::option_set(c"pickup_always", true);

    // We do not want verbose messages
    engine::option_set(c"pickup_detail", false);

    // We specify targets by hand
    engine::option_set(c"use_old_target", false);

    // We must pick items up without verification
    engine::option_set(c"pickup_inven", true);

    // Pile symbol '&' confuse the borg
    engine::option_set(c"show_piles", false);

    // We need space
    engine::option_set(c"show_labels", false);

    // flavors are confusing
    engine::option_set(c"show_flavors", false);

    // The "easy" options confuse the Borg
    engine::option_set(c"easy_open", false);
    engine::option_set(c"easy_alter", false);

    // Efficiency
    engine::set_player_hitpoint_warn(0);
}

/// Restore the user's settings (`borg_reset_settings`).
pub fn borg_reset_settings(st: &mut BorgState) {
    // Restore user key mode
    if st.borg.init_save.key_mode == KEYMAP_MODE_ROGUE as i32 {
        engine::option_set(c"rogue_like_commands", true);
    } else if st.borg.init_save.key_mode == KEYMAP_MODE_ORIG as i32 {
        engine::option_set(c"rogue_like_commands", false);
    }

    borg_reset_ignore(st);
    crate::borg_update::borg_free_detection(st);
}

/// Tell the borg that the game was closed (`borg_leave_game`, the
/// `EVENT_LEAVE_GAME` handler).
extern "C" fn borg_leave_game(
    ev_type: game_event_type,
    ev_data: *mut game_event_data,
    user: *mut core::ffi::c_void,
) {
    // (a C assert; the C build has NDEBUG)
    debug_assert!(ev_type == EVENT_LEAVE_GAME && ev_data.is_null() && user.is_null());
    crate::engine::enter(|st| st.init.game_closed = true);
}

/// Prepare some stuff based on the player race and class
/// (`borg_prepare_race_class_info`).
pub fn borg_prepare_race_class_info(st: &mut BorgState) {
    // The munchkin start may depend on the class
    borg_munchkin_class_settings(st, engine::class_cidx_if_set());

    // Initialize the various spell arrays by book
    crate::borg_magic::borg_prepare_book_info(st);

    // Notice if the game is closed so a reinitialization can be done if the
    // game is restarted without exiting.
    // this needs to be done here because the game clears all events when
    // the dungeon is recreated
    st.init.game_closed = false;
    engine::event_add_handler(EVENT_LEAVE_GAME, borg_leave_game);
}

/// Initialize the Borg (`borg_init`).
pub fn borg_init(st: &mut BorgState) {
    // Verify system
    // Redraw everything
    engine::do_cmd_redraw(st);

    // Message
    borg_note(st, "Initializing the Borg... (memory)");
    st.init.init_failure = false;

    // Flush it
    engine::term_fresh();

    // Mega-Hack -- verify memory (C allocates and frees 400k zero bytes)

    // Prapare a local random number seed (randint1)
    if st.core.rand_local == 0 {
        st.core.rand_local = (engine::randint0(0x1000_0000) + 1) as u32;
    }

    // C: `borg.player = player;` (an MSVC work-around). The Rust `Borg`
    // has no such field: the player is read through `engine`.

    // old depth starts at deepest level
    st.borg.status.old_depth = 128;

    // Initialize borg.ini options

    // Message
    borg_note(st, "Initializing the Borg... (borg.txt)");
    let warning_given = borg_init_txt_file(st);

    // Initialize game options

    // Message
    borg_note(st, "Initializing the Borg... (options)");

    // Flush it
    engine::term_fresh();

    // Make sure it rolls up a new guy at death
    if engine::screensaver() || st.cfg(BORG_CHEAT_DEATH) != 0 {
        // We need the borg to keep playing after he dies
        engine::option_set(c"cheat_live", true);
    }

    // Redraw map
    engine::player_redraw_add(PR_MAP);

    // Various

    // Message
    borg_note(st, "Initializing the Borg... (various)");

    // Flush it
    engine::term_fresh();

    // Cheat / Panic

    // more cheating
    st.borg.status.cheat_death = false;

    // set the continuous play mode if the game cheat death is on
    if engine::player_opt(OPT_cheat_live) {
        st.borg.status.cheat_death = true;
    }

    // Initialize

    crate::borg_messages::borg_init_messages(st);

    // Initialize
    crate::borg_io::borg_init_io(st);
    crate::borg_item_val::borg_init_item_val(st);
    crate::borg_item_activation::borg_init_item_activation(st);

    // Grids
    crate::borg_cave::borg_init_cave(st);
    crate::borg_flow::borg_init_flow(st);

    crate::borg_item_wear::borg_init_item_wear(st);
    crate::borg_flow_take::borg_init_flow_take(st);
    crate::borg_flow_kill::borg_init_flow_kill(st);

    crate::borg_item::borg_init_item(st);
    crate::borg_store::borg_init_store(st);
    // Object/Monster tracking
    crate::borg_update::borg_init_update(st);

    // React to race and class

    // Notice the new race and class
    borg_prepare_race_class_info(st);

    // All done

    // Done initialization
    borg_note(st, "Initializing the Borg... done.");

    // Reset the clock
    st.borg.time.now = 10;

    // note: I would check if player_id2class returns null but it
    // never does, even on a bad class
    const CLASSES: [&[u8]; 9] = [
        b"Warrior",
        b"Mage",
        b"Druid",
        b"Priest",
        b"Necromancer",
        b"Paladin",
        b"Rogue",
        b"Ranger",
        b"Blackguard",
    ];
    // (CLASS_WARRIOR .. CLASS_BLACKGUARD are 0 .. 8; `||` stops at the
    // first mismatch)
    if CLASSES
        .iter()
        .enumerate()
        .any(|(c, &name)| engine::class_name_by_id(c as u32) != name)
    {
        borg_note(st, "**STARTUP FAILURE** classes do not match");
        st.init.init_failure = true;
    }

    // Don't allow the user to do stupid things unless they ask to
    if st.cfg(BORG_ALLOW_STRANGE_OPTS) == 0 {
        let (tile_width, tile_height) = engine::tile_size();
        if tile_width > 1 || tile_height > 1 {
            borg_note(st, "**STARTUP FAILURE** Scaled tiles not allowed");
            borg_note(st, "** can be bypassed with borg_allow_strange_opts **");
            st.init.init_failure = true;
        }

        if engine::player_opt(OPT_birth_force_descend) {
            birth_option_failure(st, "**STARTUP FAILURE** must allow up stairs");
        }

        if !engine::player_opt(OPT_birth_connect_stairs) {
            birth_option_failure(st, "**STARTUP FAILURE** must connect stairs");
        }

        if engine::player_opt(OPT_birth_no_recall) {
            birth_option_failure(st, "**STARTUP FAILURE** must allow recall");
        }

        if engine::player_opt(OPT_birth_percent_damage) {
            birth_option_failure(st, "**STARTUP FAILURE** strange damage calculation");
        }
    }

    // Official message
    if !st.init.init_failure && !warning_given {
        borg_note(st, "# Ready...");
    }

    // Now it is ready
    st.init.initialized = true;
}

/// The three notes of a birth option start-up failure, and the flag.
fn birth_option_failure(st: &mut BorgState, what: &str) {
    borg_note(st, what);
    borg_note(st, "** birth option failure **");
    borg_note(st, "** can be bypassed with borg_allow_strange_opts **");
    st.init.init_failure = true;
}

/// Clean up resources allocated for the borg (`borg_free`).
pub fn borg_free(st: &mut BorgState) {
    // Undo the allocations in reverse order from what borg_init() does.
    engine::event_remove_handler(EVENT_LEAVE_GAME, borg_leave_game);
    crate::borg_update::borg_free_update(st);
    crate::borg_item::borg_free_item(st);
    crate::borg_store::borg_free_store(st);

    crate::borg_flow_kill::borg_free_flow_kill(st);
    crate::borg_flow_take::borg_free_flow_take(st);
    crate::borg_item_wear::borg_free_item_wear(st);

    crate::borg_flow::borg_free_flow(st);
    crate::borg_cave::borg_free_cave(st);
    crate::borg_io::borg_free_io(st);

    crate::borg_messages::borg_free_messages(st);
    borg_free_txt_file(st);
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A borg.txt in memory: `file_getl()` semantics (z-file.c), a stub
    /// formula loader that skips to the `[END` line, recorded outputs.
    struct FakeTxt {
        st: BorgState,
        data: Vec<u8>,
        pos: usize,
        out: Vec<String>,
        class: Option<u32>,
    }

    impl FakeTxt {
        fn new(data: &[u8]) -> Self {
            FakeTxt { st: BorgState::default(), data: data.to_vec(), pos: 0, out: Vec::new(), class: None }
        }

        /// `file_getl(f, buf, len)` (z-file.c), on `data`.
        fn file_getl(&mut self, buf: &mut [u8], len: usize) -> bool {
            let mut seen_cr = false;
            let mut i = 0;
            let max_len = len - 1;
            while i < max_len {
                let Some(&c) = self.data.get(self.pos) else {
                    buf[i] = 0;
                    return i != 0;
                };
                self.pos += 1;
                if c == b'\r' {
                    seen_cr = true;
                    continue;
                }
                if seen_cr && c != b'\n' {
                    self.pos -= 1;
                    buf[i] = 0;
                    return true;
                }
                if c == b'\n' {
                    buf[i] = 0;
                    return true;
                }
                if c == b'\t' {
                    let tabstop = (i + 4) / 4 * 4;
                    if tabstop >= len {
                        break;
                    }
                    while i < tabstop {
                        buf[i] = b' ';
                        i += 1;
                    }
                    continue;
                }
                buf[i] = c;
                i += 1;
            }
            buf[i] = 0;
            true
        }

        /// What the C driver prints for the same file.
        fn run(data: &[u8]) -> Vec<String> {
            let mut env = FakeTxt::new(data);
            let (found, old) = read_txt_settings(&mut env);
            let w = finish_txt_settings(&mut env, found, old);
            let mut out = env.out;
            out.push(format!("RET {}", i32::from(w)));
            out.push(format!("LUNAL {}", i32::from(env.st.borg.lunal_mode)));
            for (i, v) in env.st.core.cfg.iter().enumerate() {
                out.push(format!("CFG {i} {v}"));
            }
            out
        }
    }

    impl TxtEnv for FakeTxt {
        fn st(&mut self) -> &mut BorgState {
            &mut self.st
        }
        fn getl(&mut self, buf: &mut [u8; TXT_BUF]) -> bool {
            self.file_getl(buf, TXT_BUF - 1)
        }
        fn load_formulas(&mut self) {
            self.out.push("FORMULAS".into());
            let mut b = [0u8; 1024];
            while self.file_getl(&mut b, 1023) {
                if b.starts_with(b"[END") {
                    break;
                }
            }
        }
        fn warning(&mut self, what: &str) {
            self.out.push(format!("WARN {what}"));
        }
        fn term_fresh(&mut self) {
            self.out.push("FRESH".into());
        }
        fn class_cidx(&mut self) -> Option<u32> {
            self.class
        }
    }

    fn cfg_of(data: &[u8]) -> (Vec<i32>, Vec<String>) {
        let out = FakeTxt::run(data);
        let cfg = out
            .iter()
            .filter_map(|l| l.strip_prefix("CFG "))
            .map(|l| l.split(' ').nth(1).unwrap().parse().unwrap())
            .collect();
        (cfg, out.into_iter().filter(|l| !l.starts_with("CFG ")).collect())
    }

    #[test]
    fn table_matches_enum() {
        use crate::borg_headers::borg_h::*;
        assert_eq!(BORG_SETTINGS[BORG_VERBOSE as usize].name, c"borg_verbose");
        assert_eq!(BORG_SETTINGS[BORG_MUNCHKIN_LEVEL as usize].name, c"borg_munchkin_level");
        assert_eq!(BORG_SETTINGS[BORG_RESPAWN_RACE as usize].name, c"borg_respawn_race");
        assert_eq!(
            BORG_SETTINGS[BORG_RESTORE_IGNORE_SETTINGS as usize].name,
            c"borg_restore_ignore_settings"
        );
    }

    /// Expected values from the C parser (borg-init.c with stubs).
    #[test]
    fn parses_like_c() {
        use crate::borg_headers::borg_h::*;
        let (cfg, out) = cfg_of(b"");
        let defaults: Vec<i32> = BORG_SETTINGS.iter().map(|s| s.default_value).collect();
        assert_eq!(cfg, defaults);
        assert_eq!(out, ["RET 0", "LUNAL 0"]);

        let (cfg, _) = cfg_of(
            b"  BORG_VERBOSE=yes\r\nborg_munchkin_level  = -  3\nborg_stop_dlevel==-12x\n\
              borg_uses_swaps Nope\nborg_respawn_race 11\nborg_respawn_class -2\n\
              borg_enchant_limit 99999999999\nborg_no_deeper -\nborg_respawn_winners\tTRUE\n\
              # borg_kills_uniques = TRUE\n borg_lunal_mode 1\nborg_munchkin_depth 0\n",
        );
        assert_eq!(cfg[BORG_VERBOSE as usize], 1);
        assert_eq!(cfg[BORG_MUNCHKIN_LEVEL as usize], 1);
        assert_eq!(cfg[BORG_STOP_DLEVEL as usize], -12);
        assert_eq!(cfg[BORG_USES_SWAPS as usize], 0);
        assert_eq!(cfg[BORG_RESPAWN_RACE as usize], 0);
        assert_eq!(cfg[BORG_RESPAWN_CLASS as usize], 0);
        // (int) 99999999999L = 1215752191
        assert_eq!(cfg[BORG_ENCHANT_LIMIT as usize], 15);
        // a "-" with no number keeps the default (better-borg-c milestone6
        // patch 0058; the original negated the default to -127)
        assert_eq!(cfg[BORG_NO_DEEPER as usize], 127);
        assert_eq!(cfg[BORG_STOP_KING as usize], 0);
        assert_eq!(cfg[BORG_KILLS_UNIQUES as usize], 0);
        assert_eq!(cfg[BORG_LUNAL_MODE as usize], 1);
        assert_eq!(cfg[BORG_MUNCHKIN_DEPTH as usize], 8);

        // an integer setting followed by a non-number keeps its default and
        // the parser moves on (milestone6 patch 0057; the original looped
        // forever on such a line)
        let (cfg, _) = cfg_of(b"borg_respawn_race = x\nborg_respawn_class = 3\n");
        assert_eq!(cfg[BORG_RESPAWN_RACE as usize], defaults[BORG_RESPAWN_RACE as usize]);
        assert_eq!(cfg[BORG_RESPAWN_CLASS as usize], 3);

        let (cfg, out) = cfg_of(b"borg_uses_dynamic_calcs = TRUE\nCND foo\n");
        assert_eq!(cfg[BORG_USES_DYNAMIC_CALCS as usize], 0);
        assert_eq!(
            out,
            [
                "WARN ** Borg's dynamic calculations enabled but old formulas found in borg.txt.",
                "WARN ** formulas disabled ** ",
                "FRESH",
                "RET 1",
                "LUNAL 0"
            ]
        );
    }

    /// The munchkin start by class (better-borg-c milestone8, PR302):
    /// Warriors, Druids and Rangers take it at `borg_munchkin_class_depth`
    /// unless borg.txt turns the munchkin start on or the rule off.
    #[test]
    fn munchkin_class_rule() {
        use crate::borg_headers::borg_h::BORG_MUNCHKIN_START;
        let run = |data: &[u8], class: Option<u32>| {
            let mut env = FakeTxt::new(data);
            env.class = class;
            let (found, old) = read_txt_settings(&mut env);
            finish_txt_settings(&mut env, found, old);
            let c = &env.st.core.cfg;
            (c[BORG_MUNCHKIN_START as usize], c[BORG_MUNCHKIN_DEPTH as usize])
        };
        // with no borg.txt the rule is off: the original C Borg's behaviour
        for class in [0, 1, 2, 3, 4, 5, 6, 7, 8] {
            assert_eq!(run(b"", Some(class)), (0, 16), "class {class}");
        }
        assert_eq!(run(b"", None), (0, 16));
        // the rule on: the three classes take the munchkin start at depth 6
        for class in [0, 2, 7] {
            assert_eq!(
                run(b"borg_munchkin_by_class = TRUE\n", Some(class)),
                (1, 6),
                "class {class}"
            );
        }
        // every other class, and no player class yet
        for class in [1, 3, 4, 5, 6, 8] {
            assert_eq!(
                run(b"borg_munchkin_by_class = TRUE\n", Some(class)),
                (0, 16),
                "class {class}"
            );
        }
        assert_eq!(run(b"borg_munchkin_by_class = TRUE\n", None), (0, 16));
        // the rule off, a munchkin start from borg.txt, the depth setting
        assert_eq!(run(b"borg_munchkin_by_class = FALSE\n", Some(0)), (0, 16));
        assert_eq!(run(b"borg_munchkin_start = TRUE\n", Some(0)), (1, 16));
        assert_eq!(
            run(b"borg_munchkin_by_class = TRUE\nborg_munchkin_class_depth = 9\n", Some(2)),
            (1, 9)
        );
        assert_eq!(
            run(b"borg_munchkin_by_class = TRUE\nborg_munchkin_class_depth = 0\n", Some(7)),
            (1, 8)
        );

        // a reincarnation into another class restores the borg.txt values
        let mut env = FakeTxt::new(b"borg_munchkin_by_class = TRUE\n");
        env.class = Some(0);
        let (found, old) = read_txt_settings(&mut env);
        finish_txt_settings(&mut env, found, old);
        borg_munchkin_class_settings(&mut env.st, Some(1));
        let c = &env.st.core.cfg;
        assert_eq!((c[BORG_MUNCHKIN_START as usize], c[BORG_MUNCHKIN_DEPTH as usize]), (0, 16));
    }

    #[test]
    fn buying_strategy_settings_parse_and_validate() {
        use crate::borg_headers::borg_h::*;
        let (cfg, _) = cfg_of(
            b"borg_buy_strategy_default = 1\nborg_buy_strategy_munchkin = 0\n\
              borg_buy_strategy_current_classes = 1\nborg_buy_strategy_reserve410_classes = 2\n\
              borg_buy_strategy_current_munchkin_classes = 4\n\
              borg_buy_strategy_reserve410_munchkin_classes = 8\n\
              borg_buy_identify = 1\n",
        );
        assert_eq!(cfg[BORG_BUY_STRATEGY_DEFAULT as usize], 1);
        assert_eq!(cfg[BORG_BUY_STRATEGY_MUNCHKIN as usize], 0);
        assert_eq!(cfg[BORG_BUY_STRATEGY_CURRENT_CLASSES as usize], 1);
        assert_eq!(cfg[BORG_BUY_STRATEGY_RESERVE410_CLASSES as usize], 2);
        assert_eq!(cfg[BORG_BUY_STRATEGY_CURRENT_MUNCHKIN_CLASSES as usize], 4);
        assert_eq!(cfg[BORG_BUY_STRATEGY_RESERVE410_MUNCHKIN_CLASSES as usize], 8);
        assert_eq!(cfg[BORG_BUY_IDENTIFY as usize], 1);

        let mut env = FakeTxt::new(
            b"borg_buy_strategy_current_classes = 1\nborg_buy_strategy_reserve410_classes = 1\n",
        );
        let (found, old) = read_txt_settings(&mut env);
        finish_txt_settings(&mut env, found, old);
        assert!(env.st.init.init_failure);
        assert!(env.out.iter().any(|line| line.contains("masks conflict")));
    }

    #[test]
    fn default_file_matches_c() {
        let text: Vec<u8> = default_txt_file().concat();
        let text = String::from_utf8(text).unwrap();
        assert!(text.starts_with("# BORG.txt default settings \n"));
        assert!(text.contains("\n\nborg_verbose = FALSE\nborg_munchkin_start = FALSE\n"));
        assert!(text.contains("\nborg_respawn_class = -1\n"));
        assert!(text.ends_with(
            "borg_buy_strategy_default = 0\nborg_buy_strategy_munchkin = -1\n\
             borg_buy_strategy_current_classes = 0\nborg_buy_strategy_reserve410_classes = 0\n\
             borg_buy_strategy_current_munchkin_classes = 0\n\
             borg_buy_strategy_reserve410_munchkin_classes = 0\n\
             borg_buy_identify = 0\n"
        ));
        assert!(text.contains("\nborg_munchkin_by_class = FALSE\n"));
    }

    /// Differential check against the C parser: for every file in
    /// `$BORG_TXT_DIFF_DIR` (not `*.out`), write what the C driver prints
    /// to `<file>.rust.out`. Run with `--ignored`.
    #[test]
    #[ignore]
    fn dump_txt_dir() {
        let Ok(dir) = std::env::var("BORG_TXT_DIFF_DIR") else { return };
        for e in std::fs::read_dir(&dir).unwrap() {
            let p = e.unwrap().path();
            if p.extension().is_some_and(|x| x == "out") {
                continue;
            }
            let data = std::fs::read(&p).unwrap();
            let mut text = FakeTxt::run(&data).join("\n");
            text.push('\n');
            std::fs::write(format!("{}.rust.out", p.display()), text).unwrap();
        }
        let text: Vec<u8> = default_txt_file().concat();
        std::fs::write(format!("{dir}/default.rust.out"), text).unwrap();
    }

    /// The original C Borg's settings, with the defaults Angband's own C Borg
    /// had at the upstream pin `e473720ed88a0f0c7ca66815cef3000eb4a38bc3`
    /// (the tree vendored under `angband/`). Every one of them keeps that
    /// default in borg-rs; a setting the upstream did not have is declared in
    /// `NEW_SETTINGS` instead.
    const UPSTREAM_SETTINGS: &[(&str, u8, i32)] = &[
        ("borg_verbose", b'b', 0),
        ("borg_munchkin_start", b'b', 0),
        ("borg_munchkin_level", b'i', 12),
        ("borg_munchkin_depth", b'i', 16),
        ("borg_worships_damage", b'b', 0),
        ("borg_worships_speed", b'b', 0),
        ("borg_worships_hp", b'b', 0),
        ("borg_worships_mana", b'b', 0),
        ("borg_worships_ac", b'b', 0),
        ("borg_worships_gold", b'b', 0),
        ("borg_plays_risky", b'b', 0),
        ("borg_kills_uniques", b'b', 0),
        ("borg_uses_swaps", b'b', 1),
        ("borg_uses_dynamic_calcs", b'b', 0),
        ("borg_stop_dlevel", b'i', 128),
        ("borg_stop_clevel", b'i', 51),
        ("borg_no_deeper", b'i', 127),
        ("borg_stop_king", b'b', 1),
        ("borg_cheat_death", b'b', 0),
        ("borg_respawn_winners", b'b', 0),
        ("borg_respawn_class", b'i', -1),
        ("borg_respawn_race", b'i', -1),
        ("borg_chest_fail_tolerance", b'i', 7),
        ("borg_delay_factor", b'i', 0),
        ("borg_money_scum_amount", b'i', 0),
        ("borg_self_scum", b'b', 1),
        ("borg_lunal_mode", b'b', 0),
        ("borg_self_lunal", b'b', 0),
        ("borg_enchant_limit", b'i', 12),
        ("borg_dump_level", b'i', 1),
        ("borg_save_death", b'i', 1),
        ("borg_stop_on_bell", b'b', 0),
        ("borg_allow_strange_opts", b'b', 0),
        ("borg_autosave", b'b', 0),
        ("borg_restore_ignore_settings", b'b', 0),
    ];

    /// The settings the upstream C Borg did not have: name, kind, the built-in
    /// default, and the value the two borg.txt files give them. A setting
    /// whose two values differ is a product choice, named here; every other
    /// new setting reproduces the behaviour the original C Borg had.
    ///
    /// `borg_munchkin_by_class` is the one that differs: the built-in default
    /// is FALSE, the original C Borg's behaviour, and the product's milestone
    /// 8 decision (better-borg-c PR302, `fd683ced`) is carried by the two
    /// borg.txt files instead. `docs/ZOO-INTERFACE.md` is the prose version
    /// of this table, and the audit behind it.
    const NEW_SETTINGS: &[(&str, u8, i32, &str)] = &[
        ("borg_munchkin_by_class", b'b', 0, "TRUE"),
        ("borg_munchkin_class_depth", b'i', 6, "6"),
        ("borg_buy_strategy_default", b'i', 0, "0"),
        ("borg_buy_strategy_munchkin", b'i', -1, "-1"),
        ("borg_buy_strategy_current_classes", b'i', 0, "0"),
        ("borg_buy_strategy_reserve410_classes", b'i', 0, "0"),
        ("borg_buy_strategy_current_munchkin_classes", b'i', 0, "0"),
        ("borg_buy_strategy_reserve410_munchkin_classes", b'i', 0, "0"),
        ("borg_buy_identify", b'i', 0, "0"),
    ];

    /// The setting lines of a borg.txt: the name and the value, in order.
    /// Comment lines and the formula section carry no setting lines.
    fn borg_txt_settings(text: &str) -> Vec<(String, String)> {
        text.lines()
            .filter(|l| !l.trim_start().starts_with('#'))
            .filter_map(|l| {
                let (name, value) = l.split_once('=')?;
                let name = name.trim();
                if !name.starts_with("borg_") {
                    return None;
                }
                Some((name.to_string(), value.trim().to_string()))
            })
            .collect()
    }

    /// The table entry for a setting name.
    fn setting(name: &str) -> Option<&'static BorgSetting> {
        BORG_SETTINGS.iter().find(|s| s.name.to_str().ok() == Some(name))
    }

    /// The upstream C Borg's defaults are the ones borg-rs keeps (RULES 1 of
    /// the zoo: a parameter enters it with a default that reproduces the
    /// original behaviour).
    #[test]
    fn upstream_defaults_are_unchanged() {
        for (name, kind, default) in UPSTREAM_SETTINGS {
            let s = setting(name).unwrap_or_else(|| panic!("{name} is missing from the table"));
            assert_eq!(s.kind, *kind, "{name}");
            assert_eq!(s.default_value, *default, "{name}");
        }
    }

    /// Every setting is either an upstream setting or a declared new one, and
    /// a declared one says what it is: adding a setting is a deliberate act.
    #[test]
    fn every_setting_is_declared() {
        for s in BORG_SETTINGS.iter() {
            let name = s.name.to_str().unwrap();
            let upstream = UPSTREAM_SETTINGS.iter().any(|(n, ..)| *n == name);
            let new = NEW_SETTINGS.iter().find(|(n, ..)| *n == name);
            assert!(
                upstream || new.is_some(),
                "{name} is neither an upstream setting nor a declared new one"
            );
            if let Some((_, kind, default, _)) = new {
                assert_eq!(s.kind, *kind, "{name}");
                assert_eq!(s.default_value, *default, "{name}");
            }
        }
        for (name, ..) in NEW_SETTINGS {
            assert!(setting(name).is_some(), "{name} is declared but not in the table");
        }
    }

    /// The C parser table and the Rust table carry the same settings, in the
    /// same order, with the same kinds and defaults: the C Borg reads them
    /// (and ignores the Rust-only ones), so a drift between the two would
    /// make the two backends read different configurations.
    #[test]
    fn the_c_table_matches_the_rust_table() {
        let c = include_str!("../c-reference/borg-init.c");
        let mut entries: Vec<(&str, u8, i32)> = Vec::new();
        for line in c.lines() {
            let line = line.trim();
            let Some(rest) = line.strip_prefix("{ \"") else {
                continue;
            };
            let Some((name, rest)) = rest.split_once("\", '") else {
                continue;
            };
            let Some((kind, rest)) = rest.split_once('\'') else {
                continue;
            };
            let default = rest
                .trim()
                .trim_start_matches(',')
                .trim()
                .trim_end_matches(',')
                .trim()
                .trim_end_matches('}')
                .trim();
            let value = match default {
                "true" => 1,
                "false" => 0,
                v => v.parse().unwrap(),
            };
            entries.push((name, kind.as_bytes()[0], value));
        }
        assert_eq!(entries.len(), BORG_SETTINGS.len());
        for (i, (name, kind, value)) in entries.iter().enumerate() {
            let s = &BORG_SETTINGS[i];
            assert_eq!(s.name.to_str().unwrap(), *name);
            assert_eq!(s.kind, *kind);
            assert_eq!(s.default_value, *value);
        }
    }

    /// The two borg.txt copies carry the same settings, and between them they
    /// name every setting in the table.
    #[test]
    fn the_borg_txt_copies_agree_and_name_every_setting() {
        let harness = borg_txt_settings(include_str!("../borg.txt"));
        let shipped = borg_txt_settings(include_str!("../c-reference/borg.txt"));
        assert_eq!(harness, shipped, "the two borg.txt copies disagree");
        for s in BORG_SETTINGS.iter() {
            let name = s.name.to_str().unwrap();
            assert!(
                harness.iter().any(|(n, _)| n == name),
                "{name} is missing from the borg.txt files"
            );
        }
    }

    /// The configured values follow the declaration: an upstream setting keeps
    /// its upstream default in the config, and a new setting carries the value
    /// `NEW_SETTINGS` names. A configured value that differs from the built-in
    /// default is a product choice, and it is listed.
    #[test]
    fn the_borg_txt_values_follow_the_declaration() {
        let harness = borg_txt_settings(include_str!("../borg.txt"));
        // the configured values are compared case-insensitively: one of the
        // borg.txt lines says "false" in lower case
        let value_of = |name: &str| -> String {
            harness
                .iter()
                .find(|(n, _)| n == name)
                .unwrap_or_else(|| panic!("{name} is missing from borg.txt"))
                .1
                .to_ascii_uppercase()
        };
        for (name, _, default) in UPSTREAM_SETTINGS {
            let want = if setting(name).unwrap().kind == b'b' {
                if *default != 0 { "TRUE" } else { "FALSE" }.to_string()
            } else {
                default.to_string()
            };
            assert_eq!(value_of(name), want, "{name}");
        }
        for (name, _, _, configured) in NEW_SETTINGS {
            assert_eq!(value_of(name), configured.to_ascii_uppercase(), "{name}");
        }
    }

}
