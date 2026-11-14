//! Port of `borg-commands.c`: the Borg commands reached through `^Z`.
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
//! The engine calls `do_cmd_borg` (through `engine::do_cmd_borg`) when the
//! player presses `^Z`. It asks for a command key and runs the matching
//! entry of `BORG_COMMANDS`. Every engine call here that can wait for a
//! key (prompts, menus, targeting, messages and message flushes) takes the
//! state and lends it to the engine call, as the engine may run a Borg
//! turn while it waits (`engine::enter`). C's `msg(fmt, ...)` calls are
//! formatted here and passed as one `%s` argument (same text: no argument
//! is longer than the 1023 bytes `vstrnfmt` keeps). The module has no
//! state of its own. C bugs kept are listed in `docs/c-bugs.md`
//! (section 40).

use std::ffi::CString;

use crate::borg_cave::{BorgGrid, AUTO_MAX_X, AUTO_MAX_Y};
use crate::borg_cave_util::{borg_cave_floor_bold, borg_panel_hgt, borg_panel_wid};
use crate::borg_danger::borg_danger;
use crate::borg_init::BORG_SETTINGS;
use crate::borg_io::{borg_note, borg_note_bytes};
use crate::borg_item::{inven_wield, quiver_end};
use crate::borg_power::borg_power;
use crate::borg_projection::{borg_inc_motion, borg_los, borg_projectable_dark};
use crate::borg_headers::borg_h::BORG_MAX_SETTINGS;
use crate::borg_headers::borg_trait_h::{BI_CDEPTH, BI_CSTR, BI_MAX, BI_MAXDEPTH, BI_STR};
use crate::headers::cave_h::{FEAT_MAX, SQUARE_MAX};
use crate::headers::game_event_h::EVENT_MESSAGE_FLUSH;
use crate::headers::player_h::STAT_MAX;
use crate::headers::ui_event_h::{EVT_ESCAPE, EVT_SELECT};
use crate::headers::ui_output_h::region;
use crate::state::BorgState;
use crate::engine;

const COLOUR_WHITE: u8 = 1;
const COLOUR_ORANGE: u8 = 3;
const COLOUR_RED: u8 = 4;
const COLOUR_GREEN: u8 = 5;
const COLOUR_BLUE: u8 = 6;
const COLOUR_YELLOW: u8 = 11;
const COLOUR_L_BLUE: u8 = 14;

const BORG_MARK: u16 = 0x01;
const BORG_GLOW: u16 = 0x02;
const BORG_DARK: u16 = 0x04;
const BORG_OKAY: u16 = 0x08;
const BORG_LIGHT: u16 = 0x10;
const BORG_VIEW: u16 = 0x20;
const BORG_TEMP: u16 = 0x40;
const BORG_XTRA: u16 = 0x80;
const BORG_IGNORE_MAP: u16 = 0x100;

/// `TARGET_LOOK` (target.h)
const TARGET_LOOK: i32 = 0x02;

/// One Borg command (`struct borg_commands`).
struct BorgCommand {
    selection: u8,
    description: &'static str,
    function: fn(&mut BorgState),
}

/// The Borg commands (`borg_commands[]`), in C's order (the help lists
/// them in this order).
const BORG_COMMANDS: [BorgCommand; 29] = [
    BorgCommand { selection: b'?', description: "borg command help", function: borg_cmd_help },
    BorgCommand { selection: b'$', description: "reload borg.txt file", function: borg_cmd_init_txt_file },
    BorgCommand { selection: b'z', description: "activate borg", function: borg_cmd_start },
    BorgCommand { selection: b'x', description: "step the borg", function: borg_cmd_step },
    BorgCommand { selection: b'm', description: "show map information", function: borg_cmd_map },
    BorgCommand { selection: b'G', description: "target grid features", function: borg_cmd_grid_target_feature },
    BorgCommand { selection: b'I', description: "target grid info", function: borg_cmd_grid_target_info },
    BorgCommand { selection: b'D', description: "target grid danger", function: borg_cmd_grid_target_danger },
    BorgCommand { selection: b'@', description: "line of sight map", function: borg_cmd_grid_line_of_sight },
    BorgCommand { selection: b'F', description: "map of fear levels", function: borg_cmd_fear },
    BorgCommand { selection: b'f', description: "update config flags", function: borg_cmd_flags },
    BorgCommand { selection: b'u', description: "refresh borg vars", function: borg_cmd_update },
    BorgCommand { selection: b'c', description: "toggle cheat death", function: borg_cmd_cheat },
    BorgCommand { selection: b's', description: "set search string", function: borg_cmd_search_string },
    BorgCommand { selection: b'^', description: "show the borg path", function: borg_cmd_flow },
    BorgCommand { selection: b'y', description: "step history", function: borg_cmd_steps },
    BorgCommand { selection: b't', description: "current targeting", function: borg_cmd_targeting },
    BorgCommand { selection: b'h', description: "things the borg has", function: borg_cmd_has },
    BorgCommand { selection: b'p', description: "borg power", function: borg_cmd_power },
    BorgCommand { selection: b'P', description: "preparation depth", function: borg_cmd_prepare },
    BorgCommand { selection: b'!', description: "time tracking flags", function: borg_cmd_time },
    BorgCommand { selection: b'C', description: "list count of 'nasties'", function: borg_cmd_nasties },
    BorgCommand { selection: b'0', description: "borg stats (str/int etc)", function: borg_cmd_stats },
    BorgCommand { selection: b'w', description: "swap weapons/armor", function: borg_cmd_swaps },
    BorgCommand { selection: b'S', description: "spells", function: borg_cmd_spells },
    BorgCommand { selection: b'l', description: "creates snapshot log file", function: borg_cmd_dump },
    BorgCommand { selection: b'o', description: "object details", function: borg_cmd_object_desc },
    BorgCommand { selection: b'R', description: "respawn", function: borg_cmd_respawn },
    BorgCommand { selection: b'v', description: "version", function: borg_cmd_version },
];

/// The area of the two menus (`region area = { 21, 5, 1, 10 }`).
const MENU_AREA: region = region { col: 21, row: 5, width: 1, page_rows: 10 };

/// `msg("%s", text)` for a text built here.
fn msg(st: &mut BorgState, text: impl AsRef<[u8]>) {
    engine::msg(st, text.as_ref());
}

/// `event_signal(EVENT_MESSAGE_FLUSH)`
fn message_flush(st: &mut BorgState) {
    engine::event_signal(st, EVENT_MESSAGE_FLUSH);
}

/// `get_com(prompt, &cmd)` with a prompt built here.
fn get_com_text(st: &mut BorgState, prompt: &[u8]) -> (bool, u8) {
    let prompt = CString::new(prompt).expect("no NUL in prompt");
    engine::get_com(st, &prompt)
}

/// The text of a C char array up to its NUL.
fn c_chars(a: &[core::ffi::c_char]) -> Vec<u8> {
    a.iter().take_while(|&&c| c != 0).map(|&c| c as u8).collect()
}

/// `"%c"` of a C `char`.
fn push_char(v: &mut Vec<u8>, c: u8) {
    v.push(c);
}

/// Get a grid square to use for a command (`borg_cmd_target`).
fn borg_cmd_target(st: &mut BorgState) -> bool {
    let mut cmd = b'n';

    // Refresh the screen in case we were in help
    engine::term_load();
    engine::term_flush();

    // Check for use of current target
    if engine::target_okay() {
        let (ok, c) = engine::get_com(st, c"Use current target? (y/n) ");
        cmd = c;
        if !ok {
            borg_note(st, "command aborted");
            return false;
        }
    }

    if cmd != b'y' && cmd != b'Y' {
        // Get a command (or Cancel). This forces an extra keypress to
        // start looking around, which lets the message be printed.
        if !engine::get_com_ex(st, c"Select grid location (t to target): ") {
            borg_note(st, "targeting aborted");
            return false;
        }

        let (cx, cy) = (st.borg.c.x, st.borg.c.y);
        if !engine::target_set_interactive(st, TARGET_LOOK, cx, cy, true) {
            borg_note(st, "targeting aborted");
            return false;
        }
    }
    true
}

/// List the commands, then read one (`borg_cmd_help`).
fn borg_cmd_help(st: &mut BorgState) {
    let command_count = BORG_COMMANDS.len();

    // Save the screen
    engine::term_save();

    // Clear the screen
    engine::term_clear();

    // List the commands in two columns
    let mut line = 2;
    let mut i = 0;
    while i < command_count {
        engine::term_putstr(2, line, -1, i32::from(COLOUR_WHITE), &help_line(&BORG_COMMANDS[i]));
        if i + 1 < command_count {
            engine::term_putstr(42, line, -1, i32::from(COLOUR_WHITE), &help_line(&BORG_COMMANDS[i + 1]));
        }
        i += 2;
        line += 1;
    }

    do_cmd_borg(st);

    // Refresh the screen
    engine::term_load();
}

/// `format("Command '%c' %s", selection, description)`
fn help_line(cmd: &BorgCommand) -> Vec<u8> {
    let mut v = b"Command '".to_vec();
    push_char(&mut v, cmd.selection);
    v.extend_from_slice(b"' ");
    v.extend_from_slice(cmd.description.as_bytes());
    v
}

/// Set up the Borg and start it (`borg_cmd_start`).
fn borg_cmd_start(st: &mut BorgState) {
    // Make sure the important game options are set correctly
    crate::borg_init::borg_reinit_options(st);

    crate::borg_item_wear::borg_clear_best(st);

    // Activate
    st.borg.status.active = true;

    // Reset cancel
    st.borg.status.cancel = false;

    // Step forever
    st.core.step = 0;

    if engine::player_lazymove_delay() != 0 {
        borg_note(st, "# Turning off lazy movement controls");
        engine::set_player_lazymove_delay(0);
    }

    // Message
    borg_note(st, "# Installing keypress hook");

    // Activate the key stealer
    crate::borg::borg_update_entrypoint(true);
}

/// Reload borg.txt (`borg_cmd_init_txt_file`).
fn borg_cmd_init_txt_file(st: &mut BorgState) {
    crate::borg_init::borg_init_txt_file(st);
}

/// The colour of a danger or regional fear value `p` on the avoidance
/// maps, or `None` below `avoidance / 10` (not shown).
fn avoidance_colour(p: i32, avoidance: i32) -> Option<u8> {
    // Skip non-avoidances
    if p < avoidance / 10 {
        return None;
    }

    // Use colors for less painful
    let mut a = COLOUR_RED;
    if p < avoidance / 2 {
        a = COLOUR_ORANGE;
    }
    if p < avoidance / 4 {
        a = COLOUR_YELLOW;
    }
    if p < avoidance / 6 {
        a = COLOUR_GREEN;
    }
    if p < avoidance / 8 {
        a = COLOUR_BLUE;
    }
    Some(a)
}

/// `"(%d,%d of %d,%d)"`: the Borg's grid and the panel.
fn panel_text(st: &BorgState) -> String {
    let t = engine::term_geometry();
    format!(
        "({},{} of {},{})",
        st.borg.c.y,
        st.borg.c.x,
        t.offset_y / borg_panel_hgt(),
        t.offset_x / borg_panel_wid()
    )
}

/// Show on the map how dangerous certain areas are
/// (`borg_cmd_avoidances`).
fn borg_cmd_avoidances(st: &mut BorgState) {
    // Scan map
    for y in 1..=AUTO_MAX_Y - 1 {
        for x in 1..=AUTO_MAX_X - 1 {
            // Obtain danger
            let p = borg_danger(st, y, x, 1, true, false);

            if let Some(a) = avoidance_colour(p, i32::from(st.borg.avoidance)) {
                // Display
                engine::print_rel(b'*', a, y, x);
            }
        }
    }

    // Get keypress
    let text = format!("{} Avoidance value {}.", panel_text(st), st.borg.avoidance);
    msg(st, text);
    message_flush(st);
}

/// Take one or more steps (`borg_cmd_step`).
fn borg_cmd_step(st: &mut BorgState) {
    // Make sure the important game options are set correctly
    crate::borg_init::borg_reinit_options(st);

    crate::borg_item_wear::borg_clear_best(st);

    // Activate
    st.borg.status.active = true;

    // Reset cancel
    st.borg.status.cancel = false;

    // Step N times (the int is truncated to borg_step's uint16_t)
    st.core.step = engine::get_quantity(st, c"Step how many times? ", 1000) as u16;
    if st.core.step < 1 {
        st.core.step = 1;
    }

    crate::borg_trait::borg_notice(st, true);

    // Message
    borg_note(st, "# Installing keypress hook");
    let text = format!("# Stepping Borg {} times", st.core.step);
    borg_note(st, &text);

    // Activate the key stealer
    crate::borg::borg_update_entrypoint(true);
}

/// Update the Borg's internal variables (`borg_cmd_update`).
fn borg_cmd_update(st: &mut BorgState) {
    // Make sure the important game options are set correctly
    crate::borg_init::borg_reinit_options(st);

    // Activate
    st.borg.status.active = true;

    // Immediate cancel
    st.borg.status.cancel = true;

    // Step forever
    st.core.step = 0;

    crate::borg_trait::borg_notice(st, true);

    // Message
    borg_note(st, "# Installing keypress hook");

    // Activate the key stealer
    crate::borg::borg_update_entrypoint(true);
}

/// `cursor ? COLOUR_L_BLUE : COLOUR_WHITE`
fn label_attr(cursor: bool) -> u8 {
    if cursor {
        COLOUR_L_BLUE
    } else {
        COLOUR_WHITE
    }
}

/// `"TRUE"` / `"FALSE"`
fn bool_text(v: i32) -> &'static str {
    if v != 0 {
        "TRUE"
    } else {
        "FALSE"
    }
}

/// Display an entry of the config menu (`get_cfg_display`). `cfg` is a
/// copy of `borg_cfg` taken before the menu (nothing changes it while the
/// menu is shown).
fn get_cfg_display(cfg: &[i32], r: engine::MenuRow) {
    let attr = label_attr(r.cursor);
    let setting = &BORG_SETTINGS[r.oid as usize];

    // Clear the line
    engine::term_erase((r.col - 1).max(0), r.row, r.width);
    engine::c_put_str(attr, setting.name.to_bytes(), r.row, r.col);

    let value = if setting.kind == b'b' {
        bool_text(cfg[r.oid as usize]).to_string()
    } else {
        // setting type i (integer)
        cfg[r.oid as usize].to_string()
    };
    engine::c_put_str(attr, value.as_bytes(), r.row, r.col + 30);
}

/// Ask for a new value of an int setting, with its value as the default
/// (`borg_cfg_get_quantity`).
fn borg_cfg_get_quantity(st: &mut BorgState, config: usize) -> i32 {
    let amt = st.core.cfg[config];
    let mut buf = [0u8; 80];

    // Build the default
    let text = amt.to_string();
    buf[..text.len()].copy_from_slice(text.as_bytes());

    // Ask for a quantity
    let mut prompt = b"Enter new temp value for ".to_vec();
    prompt.extend_from_slice(BORG_SETTINGS[config].name.to_bytes());
    prompt.extend_from_slice(b": ");
    let prompt = CString::new(prompt).expect("no NUL in prompt");
    if !engine::get_string(st, &prompt, &mut buf, 7) {
        return 0;
    }

    // Extract a number (`atoi`)
    let mut amt = crate::borg_formulas::c_strtol(&buf).0 as i32;

    // Enforce the minimum
    if amt < -1 {
        amt = -1;
    }

    amt
}

/// Change a config flag for this run (`borg_cmd_flags`).
fn borg_cmd_flags(st: &mut BorgState) {
    msg(st, "Pick a configuration setting to change (runtime only)");

    let mut m = engine::Menu::new_scroll();
    m.setpriv(BORG_MAX_SETTINGS as i32);
    m.setpriv(BORG_MAX_SETTINGS as i32);
    m.layout(MENU_AREA);

    // Set up the screen
    engine::screen_save(st);
    engine::window_make(19, 4, 58, 15);

    let cfg = st.core.cfg.clone();
    let result = m.select(st, 0, true, &mut |r| get_cfg_display(&cfg, r));

    engine::screen_load(st);

    if result == EVT_ESCAPE {
        return;
    }

    if result == EVT_SELECT {
        let cursor = m.cursor();
        if cursor < 0 || cursor >= BORG_MAX_SETTINGS as i32 {
            borg_note(st, "oops");
            return;
        }
        let c = cursor as usize;
        let name = BORG_SETTINGS[c].name.to_bytes();
        let mut text = b"new temp config for ".to_vec();
        text.extend_from_slice(name);
        text.extend_from_slice(b" is ");
        if BORG_SETTINGS[c].kind == b'b' {
            st.core.cfg[c] = i32::from(st.core.cfg[c] == 0);
            text.extend_from_slice(bool_text(st.core.cfg[c]).as_bytes());
        } else {
            let new_value = borg_cfg_get_quantity(st, c);
            st.core.cfg[c] = new_value;
            text.extend_from_slice(st.core.cfg[c].to_string().as_bytes());
        }
        // C passes the text as the format of `msg` (it has no '%')
        msg(st, text);
    }
}

/// Toggle cheat death (`borg_cmd_cheat`).
fn borg_cmd_cheat(st: &mut BorgState) {
    st.borg.status.cheat_death = !st.borg.status.cheat_death;
    let text = format!("Borg -- borg_cheat_death is now {}.", i32::from(st.borg.status.cheat_death));
    msg(st, text);
}

/// List the number of banishable creatures (`borg_cmd_nasties`).
fn borg_cmd_nasties(st: &mut BorgState) {
    // Log Header
    borg_note(st, "Borg Nasties Count");

    // Find the numerous nasty in order of nastiness
    let mut i = 0;
    while i < usize::from(st.kills.nasties_num) {
        let mut text = b"Nasty: [".to_vec();
        push_char(&mut text, st.kills.nasties[i]);
        text.extend_from_slice(
            format!(
                "] Count: {}, limited: {}",
                st.kills.nasties_count[i], st.kills.nasties_limit[i]
            )
            .as_bytes(),
        );
        borg_note_bytes(st, &text);
        i += 1;
    }

    // Done
    msg(st, "Borg Nasty Dump Complete.  Examine Log.");
}

/// Change the search string that stops the Borg
/// (`borg_cmd_search_string`).
fn borg_cmd_search_string(st: &mut BorgState) {
    // Get the new search string (or cancel the matching); edited in a
    // copy of all of borg_match, written back as a whole
    let mut buf = st.messages.match_;
    let ok = engine::get_string(st, c"Borg Match String: ", &mut buf, 70);
    if !ok {
        // Cancel it (`my_strcpy(borg_match, "", ...)`)
        buf[0] = 0;
    }
    st.messages.match_ = buf;
    if !ok {

        // Message
        msg(st, "Borg Match String de-activated.");
    }
}

/// The labels of the extra feature menu rows past `FEAT_MAX + 1`
/// (`i = oid - FEAT_MAX - 2`).
const GRID_INFO_LABELS: [&str; 9] = ["MARK", "GLOW", "DARK", "OKAY", "LIGHT", "VIEW", "TEMP", "XTRA", "IGNORE_MAP"];

/// The `borg_grid` info bits of the same rows.
const GRID_INFO_BITS: [u16; 9] = [
    BORG_MARK,
    BORG_GLOW,
    BORG_DARK,
    BORG_OKAY,
    BORG_LIGHT,
    BORG_VIEW,
    BORG_TEMP,
    BORG_XTRA,
    BORG_IGNORE_MAP,
];

/// Display an entry of the features menu (`get_feature_display`).
fn get_feature_display(r: engine::MenuRow) {
    let attr = label_attr(r.cursor);

    // Clear the line
    engine::term_erase((r.col - 1).max(0), r.row, r.width);

    let feat_max = FEAT_MAX as i32;
    if r.oid < feat_max {
        engine::c_put_str(attr, engine::feat_name(r.oid), r.row, r.col);
    } else {
        let i = r.oid - feat_max - 2;
        if i >= 0 {
            if let Some(label) = GRID_INFO_LABELS.get(i as usize) {
                engine::c_put_str(attr, label.as_bytes(), r.row, r.col);
            }
        } else {
            if r.oid == feat_max {
                engine::c_put_str(attr, b"Traps", r.row, r.col);
            }
            if r.oid == feat_max + 1 {
                engine::c_put_str(attr, b"Glyphs", r.row, r.col);
            }
        }
    }
}

/// Is the grid one of those the features map shows for menu row
/// `cursor`?
fn grid_feature_shown(cursor: i32, ag: &BorgGrid) -> bool {
    let feat_max = FEAT_MAX as i32;
    if cursor < feat_max {
        return i32::from(ag.feat) == cursor;
    }
    let i = cursor - feat_max - 2;
    if i >= 0 {
        match GRID_INFO_BITS.get(i as usize) {
            Some(&bit) => ag.info & bit != 0,
            None => true,
        }
    } else {
        // trap
        if cursor == feat_max && !ag.trap {
            return false;
        }
        // glyph
        if cursor == feat_max + 1 && !ag.glyph {
            return false;
        }
        true
    }
}

/// Show the map grids with a feature or info flag
/// (`borg_cmd_grid_feature`).
fn borg_cmd_grid_feature(st: &mut BorgState) {
    let mut m = engine::Menu::new_scroll();
    m.setpriv(FEAT_MAX as i32 + 9 + 2);
    m.layout(MENU_AREA);

    // Set up the screen
    engine::screen_save(st);
    engine::window_make(19, 4, 58, 15);

    let result = m.select(st, 0, true, &mut get_feature_display);

    engine::screen_load(st);

    if result == EVT_ESCAPE {
        return;
    }

    // Scan map
    for y in 1..=AUTO_MAX_Y - 1 {
        for x in 1..=AUTO_MAX_X - 1 {
            let mut a = COLOUR_RED;

            // Show only those grids
            if !grid_feature_shown(m.cursor(), st.cave.grid(x, y)) {
                continue;
            }

            // Color
            if borg_cave_floor_bold(st, y, x) {
                a = COLOUR_YELLOW;
            }

            // Display
            engine::print_rel(b'*', a, y, x);
        }
    }

    // Get keypress
    msg(st, "Press any key.");
    message_flush(st);

    // Redraw map
    engine::prt_map();
}

/// Display the feature of a targeted grid
/// (`borg_cmd_grid_target_feature`).
fn borg_cmd_grid_target_feature(st: &mut BorgState) {
    if !borg_cmd_target(st) {
        return;
    }

    let l = engine::target_get();
    let (y, x) = (l.y, l.x);

    let mask = u16::from(st.cave.grid(x, y).feat);

    // The game's feature under the Borg, not at the target (C bug 41)
    let feat = engine::square_feat(st.borg.c.x, st.borg.c.y);

    let text = format!("Borg's Feat for grid ({y}, {x}) is {mask}, game Feat is {feat}");
    borg_note(st, &text);
    engine::prt_map();
}

/// Display the info flags of a targeted grid (`borg_cmd_grid_target_info`).
fn borg_cmd_grid_target_info(st: &mut BorgState) {
    if !borg_cmd_target(st) {
        return;
    }

    let l = engine::target_get();
    let (y, x) = (l.y, l.x);

    // MARK GLOW DARK OKAY LITE VIEW TEMP XTRA, read before each test
    const NAMES: [&str; 8] = ["MARK", "GLOW", "DARK", "OKAY", "LITE", "VIEW", "TEMP", "XTRA"];
    for (name, bit) in NAMES.iter().zip(GRID_INFO_BITS) {
        if st.cave.grid(x, y).info & bit != 0 {
            msg(st, format!("Borg Info for grid ({y}, {x}) is {name}"));
        }
    }

    for i in 0..SQUARE_MAX as i32 {
        if engine::flag_has(&engine::square_info(l.x, l.y), i) {
            // C passes the text as the format of `msg` (it has no '%')
            msg(st, format!("Sys Info for grid ({y}, {x}) is {i}"));
        }
    }
    engine::prt_map();
}

/// Display the Borg's danger of a targeted grid
/// (`borg_cmd_grid_target_danger`).
fn borg_cmd_grid_target_danger(st: &mut BorgState) {
    if !borg_cmd_target(st) {
        return;
    }

    let l = engine::target_get();

    // Turns
    let n = engine::get_quantity(st, c"Time on square? ", 1000);

    // Danger of grid
    let p = borg_danger(st, l.y, l.x, n, true, false);
    msg(st, format!("Danger({},{},{}) is {}", l.x, l.y, n, p));
    engine::prt_map();
}

/// List the previous steps (`borg_cmd_steps`).
fn borg_cmd_steps(st: &mut BorgState) {
    let a = COLOUR_RED;

    // Check for an existing step: entries num .. 1 (C bug 41: one past
    // the newest, entry 0 never)
    let mut i = 0;
    while i < i32::from(st.flow.track_step.num) {
        let k = (i32::from(st.flow.track_step.num) - i) as usize;

        // Display
        engine::print_rel(b'*', a, st.flow.track_step.y[k], st.flow.track_step.x[k]);
        let text = format!("(-{}) Steps noted {},{}", i, st.flow.track_step.y[k], st.flow.track_step.x[k]);
        msg(st, text);
        message_flush(st);
        let k = (i32::from(st.flow.track_step.num) - i) as usize;
        engine::print_rel(b'*', COLOUR_ORANGE, st.flow.track_step.y[k], st.flow.track_step.x[k]);
        i += 1;
    }

    // Redraw map
    engine::prt_map();
}

/// Highlight where the Borg thinks there are monsters
/// (`borg_cmd_grid_monsters`).
fn borg_cmd_grid_monsters(st: &mut BorgState) {
    let mut n = 0;

    // Scan the monsters
    for i in 1..i32::from(st.kills.nxt) {
        let kill = &st.kills.kills[i as usize];

        // Still alive
        if kill.r_idx != 0 {
            // Display
            engine::print_rel(b'*', COLOUR_RED, kill.pos.y, kill.pos.x);

            // Count
            n += 1;
        }
    }

    // Get keypress
    msg(st, format!("There are {n} known monsters."));
    message_flush(st);

    // Redraw map
    engine::prt_map();
}

/// Highlight where the Borg thinks there are objects
/// (`borg_cmd_grid_objects`).
fn borg_cmd_grid_objects(st: &mut BorgState) {
    let mut n = 0;

    // Scan the objects
    for i in 1..i32::from(st.takes.nxt) {
        let take = &st.takes.takes[i as usize];

        // Still alive
        if take.kidx != 0 {
            // Display
            engine::print_rel(b'*', COLOUR_RED, i32::from(take.y), i32::from(take.x));

            // Count
            n += 1;
        }
    }

    // Get keypress
    msg(st, format!("There are {n} known objects."));
    message_flush(st);

    // Redraw map
    engine::prt_map();
}

/// Show the Borg's path to the current target (`borg_cmd_targeting`).
fn borg_cmd_targeting(st: &mut BorgState) {
    let sv_delay = engine::player_delay_factor();
    engine::set_player_delay_factor(200);

    // Determine "path"
    let mut n_x = st.borg.c.x;
    let mut n_y = st.borg.c.y;
    let l = engine::target_get();
    let (x, y) = (l.x, l.y);

    // Borg's pathway
    loop {
        // Display
        engine::print_rel(b'*', COLOUR_RED, n_y, n_x);

        if n_x == x && n_y == y {
            break;
        }

        // Calculate the new location
        borg_inc_motion(&mut n_y, &mut n_x, st.borg.c.y, st.borg.c.x, y, x);
    }

    msg(st, "Borg's Targeting Path");
    message_flush(st);

    let (x, y) = (l.x, l.y);

    // Get a "Borg command", or abort (C bug 41: the delay is not restored)
    let (ok, ch) = engine::get_com(st, c"Borg command: Show Arc (Y/y): ");
    if !ok {
        return;
    }

    msg(st, "Actual Targeting Path");
    message_flush(st);

    if ch != b'Y' && ch != b'y' {
        // Real LOS - beam
        engine::project_from_player(st, engine::Projection::Beam, x, y);
    } else {
        // Real LOS - arc
        engine::project_from_player(st, engine::Projection::Arc, x, y);
    }

    engine::set_player_delay_factor(sv_delay);

    // Redraw map
    engine::prt_map();
}

/// Show the probable flow path (`borg_cmd_flow`).
fn borg_cmd_flow(st: &mut BorgState) {
    let mut false_y = st.borg.c.y;
    let mut false_x = st.borg.c.x;

    // Continue
    for _ in 0..250 {
        let mut b_n = 0;
        let mut b_i: i32 = -1;

        // Flow cost of the current grid: the Borg's own, not the simulated
        // one (C bug 41)
        let mut b_c = i32::from(st.flow.data_flow[st.borg.c.y as usize][st.borg.c.x as usize]) * 10;

        // Prevent loops
        b_c -= 5;

        // Look around
        for i in 0..8 {
            // Grid in that direction
            let x = false_x + i32::from(engine::ddx_ddd(i));
            let y = false_y + i32::from(engine::ddy_ddd(i));

            // Flow cost at that grid (panics off the map, where C reads
            // outside the array)
            let c = i32::from(st.flow.data_flow[y as usize][x as usize]) * 10;

            // Never backtrack
            if c > b_c {
                continue;
            }

            // Notice new best value
            if c < b_c {
                b_n = 0;
            }

            // Apply the randomizer to equivalent values
            b_n += 1;
            if b_n >= 2 && engine::randint0(b_n as u32) != 0 {
                continue;
            }

            // Track it
            b_i = i as i32;
            b_c = c;
        }

        // Try it
        if b_i >= 0 {
            // Access the location
            let x = false_x + i32::from(engine::ddx_ddd(b_i as usize));
            let y = false_y + i32::from(engine::ddy_ddd(b_i as usize));

            // Display
            engine::print_rel(b'*', COLOUR_RED, y, x);

            // Simulate motion
            false_y = y;
            false_x = x;
        }
    }
    engine::print_rel(b'*', COLOUR_YELLOW, i32::from(st.flow.flow_y[0]), i32::from(st.flow.flow_x[0]));
    msg(st, "Probable Flow Path");
    message_flush(st);

    // Redraw map
    engine::prt_map();
}

/// Show a heat map of the Borg's fear levels (`borg_cmd_fear`).
fn borg_cmd_fear(st: &mut BorgState) {
    // Scan map
    for y in 1..=AUTO_MAX_Y - 1 {
        for x in 1..=AUTO_MAX_X - 1 {
            // Obtain danger
            let p = i32::from(st.danger.fear_region[(y / 11) as usize][(x / 11) as usize]);

            if let Some(a) = avoidance_colour(p, i32::from(st.borg.avoidance)) {
                // Display
                engine::print_rel(b'*', a, y, x);
            }
        }
    }

    // Get keypress
    msg(st, format!("{} Regional Fear.", panel_text(st)));
    message_flush(st);

    // Redraw map
    engine::prt_map();

    // Scan map
    for y in 1..=AUTO_MAX_Y {
        for x in 1..=AUTO_MAX_X {
            let mut a = COLOUR_BLUE;
            let avoidance = i32::from(st.borg.avoidance);

            // Obtain danger
            let p = i32::from(st.danger.fear_monsters[y as usize][x as usize]);

            // Skip non-fears
            if p <= 0 {
                continue;
            }

            // Color Defines
            if p == 1 {
                a = COLOUR_L_BLUE;
            }
            if p < avoidance / 20 && p > 1 {
                a = COLOUR_BLUE;
            }
            if p < avoidance / 10 && p > avoidance / 20 {
                a = COLOUR_GREEN;
            }
            if p < avoidance / 4 && p > avoidance / 10 {
                a = COLOUR_YELLOW;
            }
            if p < avoidance / 2 && p > avoidance / 4 {
                a = COLOUR_ORANGE;
            }
            if p > avoidance / 2 {
                a = COLOUR_RED;
            }

            // Display
            engine::print_rel(b'*', a, y, x);
        }
    }

    // Get keypress
    msg(st, format!("{} Monster Fear.", panel_text(st)));
    message_flush(st);

    // Redraw map
    engine::prt_map();
}

/// Display the time flags the Borg keeps (`borg_cmd_time`).
fn borg_cmd_time(st: &mut BorgState) {
    msg(st, format!("time: ({}) ", st.borg.time.now));
    // borg_timer(t) = now - t
    msg(st, format!("; from town ({})", st.borg.time.now.wrapping_sub(st.borg.time.town)));
    msg(st, format!("; bounciness ({})", st.borg.antibounce_count));
    msg(st, format!("; when inviso ({})", st.borg.need_see_invis));
}

/// `SCREEN_HGT`, read from the term at each use.
fn screen_hgt() -> i32 {
    let t = engine::term_geometry();
    (t.hgt - t.row_top_map - t.row_bottom_map) / t.tile_height
}

/// `SCREEN_WID`, read from the term at each use.
fn screen_wid() -> i32 {
    let t = engine::term_geometry();
    (t.wid - t.col_map - 1) / t.tile_width
}

/// Display the Borg's line of sight (`borg_cmd_grid_line_of_sight`).
fn borg_cmd_grid_line_of_sight(st: &mut BorgState) {
    // Scan map
    let mut y = st.core.w_y;
    while y < st.core.w_y + screen_hgt() {
        let mut x = st.core.w_x;
        while x < st.core.w_x + screen_wid() {
            if borg_los(st, st.borg.c.y, st.borg.c.x, y, x) {
                engine::print_rel(b'*', COLOUR_RED, y, x);
            }
            x += 1;
        }
        y += 1;
    }

    // Get keypress
    msg(st, "Borg has Projectable to these places.");
    message_flush(st);

    // Scan map
    let mut y = st.core.w_y;
    while y < st.core.w_y + screen_hgt() {
        let mut x = st.core.w_x;
        while x < st.core.w_x + screen_wid() {
            if engine::square_in_bounds(x, y) && borg_projectable_dark(st, st.borg.c.y, st.borg.c.x, y, x) {
                engine::print_rel(b'*', COLOUR_YELLOW, y, x);
            }
            x += 1;
        }
        y += 1;
    }
    msg(st, "Borg has Projectable Dark to these places.");
    message_flush(st);

    // Scan map
    let mut y = st.core.w_y;
    while y < st.core.w_y + screen_hgt() {
        let mut x = st.core.w_x;
        while x < st.core.w_x + screen_wid() {
            if borg_los(st, st.borg.c.y, st.borg.c.x, y, x) {
                engine::print_rel(b'*', COLOUR_GREEN, y, x);
            }
            x += 1;
        }
        y += 1;
    }
    msg(st, "Borg has LOS to these places.");
    message_flush(st);

    // Redraw map
    engine::prt_map();
}

/// `%ld` of an `int32_t` argument: gcc passes it zero-extended and
/// `vstrnfmt` reads a `long`, so a negative value prints as its unsigned
/// 32-bit value (C bug 41).
fn ld_of_int32(v: i32) -> u32 {
    v as u32
}

/// Refresh the Borg's view of the items, the screen and the home, as the
/// info commands do before they report.
fn find_notice_update(st: &mut BorgState) {
    // find all items currently in use
    crate::borg_item::borg_find_all_items(st);

    // Examine the screen
    crate::borg_trait::borg_notice(st, true);

    // Examine the screen
    crate::borg_update::borg_update(st);
}

/// Show the Borg's power (`borg_cmd_power`).
fn borg_cmd_power(st: &mut BorgState) {
    find_notice_update(st);

    // Examine the inventory
    crate::borg_item_id::borg_object_fully_id(st);

    // Evaluate
    let p = borg_power(st);

    crate::borg_home_notice::borg_notice_home(st, None, false);

    // Report it
    msg(st, format!("Current Borg Power {}", ld_of_int32(p)));
    let home = crate::borg_home_power::borg_power_home(st);
    msg(st, format!("Current Home Power {}", ld_of_int32(home)));
}

/// Show the Borg's preparation level for depth (`borg_cmd_prepare`).
fn borg_cmd_prepare(st: &mut BorgState) {
    find_notice_update(st);

    // Examine the inventory
    crate::borg_item_id::borg_object_fully_id(st);
    crate::borg_home_notice::borg_notice_home(st, None, false);

    // Dump prep codes
    let mut i = 1;
    while i <= 101 {
        // Dump fear code
        if crate::borg_prepared::borg_prepared_c(st, i).is_some() {
            break;
        }
        i += 1;
    }
    let max_depth = st.borg.tr(BI_MAXDEPTH);
    // NULL prints as "" (vstrnfmt)
    let reason = crate::borg_prepared::borg_prepared_c(st, i).unwrap_or_default();
    let mut text = format!("Max Level: {}  Prep'd For: {}  Reason: ", max_depth, i - 1).into_bytes();
    text.extend_from_slice(&reason);
    msg(st, text);
    if st.borg.tr(BI_CDEPTH) == 0 {
        msg(st, "Unable to check for big fight from town.");
    } else if st.borg.ready_morgoth == 1 {
        msg(st, "You are ready for the big fight!!");
    } else if st.borg.ready_morgoth == 0 {
        msg(st, "You are NOT ready for the big fight!!");
    } else if st.borg.ready_morgoth == -1 {
        msg(st, "No readiness check done.");
    }
}

/// Show the Borg's stats as known by him (`borg_cmd_stats`).
fn borg_cmd_stats(st: &mut BorgState) {
    for i in 0..STAT_MAX {
        let mut text = b"stat # ".to_vec();
        text.extend_from_slice(crate::borg_trait::trait_name((BI_STR + i) as usize));
        text.extend_from_slice(
            format!(", is: cur {}, used {}", st.borg.tr(BI_CSTR + i), st.borg.tr(BI_STR + i)).as_bytes(),
        );
        borg_note_bytes(st, &text);
    }
}

/// `"Swap <what>:  %s, value= %d"` or `"Swap <what>:  NONE"` for the
/// swap in slot `swap - 1` (0 = none).
fn swap_text(st: &BorgState, what: &str, swap: i32, value: i32) -> Vec<u8> {
    let mut text = format!("Swap {what}:  ").into_bytes();
    if swap != 0 {
        text.extend_from_slice(&c_chars(&st.items.items[(swap - 1) as usize].desc));
        text.extend_from_slice(format!(", value= {value}").as_bytes());
    } else {
        text.extend_from_slice(b"NONE");
    }
    text
}

/// Show the Borg's current swap items (`borg_cmd_swaps`).
fn borg_cmd_swaps(st: &mut BorgState) {
    // find all items currently in use
    crate::borg_item::borg_find_all_items(st);

    // Examine the inventory
    crate::borg_trait::borg_notice(st, true);
    crate::borg_home_notice::borg_notice_home(st, None, false);

    // Check the power
    borg_power(st);

    // Examine the screen
    crate::borg_update::borg_update(st);

    // note the swap items
    let (swap, value) = (st.trait_swap.weapon_swap, st.trait_swap.weapon_swap_value);
    let text = swap_text(st, "Weapon", swap, value);
    msg(st, text);

    let (swap, value) = (st.trait_swap.armour_swap, st.trait_swap.armour_swap_value);
    let text = swap_text(st, "Armour", swap, value);
    msg(st, text);
}

/// `format("Exam spell book %d.  Press any key for next book.", book)`
fn book_prompt(book: i32) -> Vec<u8> {
    format!("Exam spell book {book}.  Press any key for next book.").into_bytes()
}

/// Show the Borg's spells (`borg_cmd_spells`).
fn borg_cmd_spells(st: &mut BorgState) {
    let mut ii = 2;

    // Save the screen
    engine::term_save();

    // Dump the spells
    if crate::borg_magic::borg_can_cast() {
        let mut book = -1;

        for i in 0..engine::class_total_spells() {
            let i = i as usize;
            if st.magic.magics[i].level < 99 {
                if book != st.magic.magics[i].book {
                    if book != -1 {
                        get_com_text(st, &book_prompt(st.magic.magics[i].book));
                    }

                    ii = 2;
                    // Clear the screen
                    engine::term_clear();
                    engine::term_putstr(1, ii, -1, i32::from(COLOUR_WHITE), b"[ Spells ].");
                    ii += 1;
                    book = st.magic.magics[i].book;
                }

                let spell = st.magic.magics[i].spell_enum;
                let legal = if crate::borg_magic::borg_spell_legal(st, spell) {
                    "legal"
                } else {
                    "Not Legal "
                };
                let failpercent = crate::borg_magic::borg_spell_fail_rate(st, spell);

                let as_ = &st.magic.magics[i];
                let mut text = as_.name.to_bytes().to_vec();
                text.extend_from_slice(
                    format!(", {legal}, attempted {} times, fail rate:{failpercent}", as_.times).as_bytes(),
                );
                engine::term_putstr(1, ii, -1, i32::from(COLOUR_WHITE), &text);
                ii += 1;
            }
        }
        get_com_text(st, &book_prompt(book));
    }

    // Restore the screen
    engine::term_load();
}

/// What `borg_cmd_has` lists.
#[derive(Clone, Copy)]
enum HasWhat {
    /// `a`: kinds the Borg has (`borg.has[]`)
    Any,
    /// `i`: pack items
    Inven,
    /// `w`: worn items and the quiver
    Worn,
    /// `r`: artifacts in all slots
    Artifact,
    /// anything else: traits
    Trait,
}

/// Dump the Borg's items/inventory/worn/artifacts/skills
/// (`borg_cmd_has`).
fn borg_cmd_has(st: &mut BorgState) {
    // Get a "Borg command", or abort
    let (ok, cmd) = engine::get_com(st, c"Dynamic Borg Has What: ((a)ny/(i)nv/(w)orn/a(r)tifact/(s)kill) ");
    if !ok {
        return;
    }

    let z = engine::z_info();
    let (what, mut item, to) = match cmd {
        b'a' | b'A' => (HasWhat::Any, 0, i32::from(z.k_max)),
        b'i' | b'I' => (HasWhat::Inven, 0, i32::from(z.pack_size)),
        b'w' | b'W' => (HasWhat::Worn, inven_wield(), quiver_end()),
        b'r' | b'R' => (HasWhat::Artifact, 0, quiver_end()),
        _ => (HasWhat::Trait, 0, BI_MAX as i32),
    };

    find_notice_update(st);

    // Examine the inventory
    crate::borg_item_id::borg_object_fully_id(st);
    crate::borg_home_notice::borg_notice_home(st, None, false);
    while item < to {
        let i = item as usize;
        match what {
            HasWhat::Any => {
                if st.borg.has[i] != 0 {
                    let mut text = format!("Item-Kind:{item:03} name=").into_bytes();
                    text.extend_from_slice(engine::kind_name_or_empty(i));
                    text.extend_from_slice(format!(" value= {}.", st.borg.has[i]).as_bytes());
                    borg_note_bytes(st, &text);
                }
            }
            HasWhat::Inven | HasWhat::Worn => {
                if st.items.items[i].iqty != 0 {
                    let kind = if matches!(what, HasWhat::Inven) { "Invn" } else { "Worn" };
                    let mut text = format!("Item-{kind}:{item:03} desc= ").into_bytes();
                    text.extend_from_slice(&c_chars(&st.items.items[i].desc));
                    text.extend_from_slice(format!(" qty {}.", st.items.items[i].iqty).as_bytes());
                    borg_note_bytes(st, &text);
                }
            }
            HasWhat::Artifact => {
                if st.items.items[i].iqty != 0 && st.items.items[i].art_idx != 0 {
                    let mut text = format!("Item-Arti:{item:03} name= ").into_bytes();
                    text.extend_from_slice(engine::artifact_name_or_empty(usize::from(st.items.items[i].art_idx)));
                    text.push(b'.');
                    borg_note_bytes(st, &text);
                }
            }
            HasWhat::Trait => {
                let mut text = format!("trait {item} (").into_bytes();
                text.extend_from_slice(crate::borg_trait::trait_name(i));
                text.extend_from_slice(format!(") value= {}.", st.borg.trait_[i]).as_bytes());
                borg_note_bytes(st, &text);
            }
        }
        item += 1;
    }

    // note the completion.
    msg(st, "Borg_has[] dump complete.  Examine Log. ");
}

/// Create a map dump (`borg_cmd_dump`).
fn borg_cmd_dump(st: &mut BorgState) {
    // Cheat the "inven" screen
    crate::borg_inventory::borg_inventory(st);

    crate::borg_log::borg_write_map(st, true);
}

/// Show the version (`borg_cmd_version`).
fn borg_cmd_version(st: &mut BorgState) {
    let text = format!("Borg Version: {}", st.core.engine_date);
    msg(st, text);
}

/// Show what the Borg believes about a pack item
/// (`borg_cmd_object_desc`).
fn borg_cmd_object_desc(st: &mut BorgState) {
    // use this item
    let n = engine::get_quantity(st, c"Which item?", i32::from(engine::z_info().pack_size));

    // find all items currently in use
    crate::borg_item::borg_find_all_items(st);

    // Examine the inventory
    crate::borg_trait::borg_notice(st, true);
    crate::borg_home_notice::borg_notice_home(st, None, false);

    // Check the power
    borg_power(st);

    // Examine the screen
    crate::borg_update::borg_update(st);

    // Save the screen
    engine::term_save();

    // get the item (its flags)
    let f = engine::inven_object_flags(n);

    // Display the special screen
    crate::borg_log::borg_display_item(st, &f, n);

    // pause for study
    msg(st, "Borg believes: ");
    message_flush(st);

    // Restore the screen
    engine::term_load();
}

/// Respawn (reincarnate) the Borg (`borg_cmd_respawn`).
fn borg_cmd_respawn(st: &mut BorgState) {
    // Confirm it
    let (_, cmd) = engine::get_com(st, c"Are you sure you want to Respawn this borg? (y or n)? ");

    if cmd == b'y' || cmd == b'Y' {
        crate::borg_reincarnate::borg_force_reincarnate(st);
        borg_cmd_start(st);
    }
}

/// Show where the Borg thinks the glyphs are (`borg_cmd_grid_glyphs`).
fn borg_cmd_grid_glyphs(st: &mut BorgState) {
    let a = COLOUR_RED;

    let mut glyph_check = 0;
    while glyph_check < i32::from(st.glyph.track.num) {
        let g = glyph_check as usize;
        // Display
        engine::print_rel(b'*', a, st.glyph.track.y[g], st.glyph.track.x[g]);
        msg(st, format!("Borg has Glyph ({glyph_check})noted."));
        message_flush(st);
        glyph_check += 1;
    }
}

/// Map information (`borg_cmd_map`).
fn borg_cmd_map(st: &mut BorgState) {
    let (ok, option) =
        engine::get_com(st, c"What to show ((a)voidances, (f)eatures, (g)lyphs, (m)onsters, (o)bjects): ");
    if !ok {
        return;
    }

    match option {
        b'a' => borg_cmd_avoidances(st),
        b'f' => borg_cmd_grid_feature(st),
        b'g' => borg_cmd_grid_glyphs(st),
        b'm' => borg_cmd_grid_monsters(st),
        b'o' => borg_cmd_grid_objects(st),
        _ => msg(st, "That is not a legal option."),
    }
}

/// Interact with the Borg (`do_cmd_borg`).
pub fn do_cmd_borg(st: &mut BorgState) {
    // Set the Borg's player location from the game: the screen has not
    // been scanned yet, and some commands need the position.
    st.borg.c = engine::player_grid();

    // Get a "Borg command", or abort
    let (ok, cmd) = engine::get_com(st, c"Borg command (? for help): ");
    if !ok {
        // Restore the screen
        engine::term_load();
        return;
    }

    for command in &BORG_COMMANDS {
        if command.selection == cmd {
            // Force initialization or reinitialize if the game was closed
            // and restarted without exiting since the last initialization
            if !st.init.initialized || st.init.game_closed {
                if st.init.initialized {
                    crate::borg_init::borg_free(st);
                }
                crate::borg_init::borg_init(st);

                if st.init.init_failure {
                    st.init.initialized = false;
                    crate::borg_init::borg_free(st);
                    borg_note(st, "** startup failure borg cannot run ** ");
                    engine::term_fresh();
                    return;
                }
            }

            (command.function)(st);
            return;
        }
    }

    // Message
    msg(st, "That is not a legal Borg command.");
}
