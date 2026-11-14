//! Port of `borg-io.c`: screen reading, messages and the keypress queue.
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

use crate::cctype::c_isspace;
use crate::engine;
use crate::headers::__stddef_wchar_t_h::wchar_t;
use crate::headers::game_event_h::{game_event_data, game_event_type, EVENT_BELL, EVENT_MESSAGE};
use crate::headers::list_message_h::MSG_GENERIC;
use crate::headers::ui_event_h::{keycode_t, keypress, EVT_KBRD};
use crate::state::BorgState;

/// Size of the keypress queue and history (`KEY_SIZE`).
pub const KEY_SIZE: usize = 8192;
/// `KC_ENTER`
pub const KC_ENTER: keycode_t = 0x9c;
/// `ESCAPE`
pub const ESCAPE: keycode_t = 0xe000;
/// `PW_BORG_1` (window flag for the Borg message window)
const PW_BORG_1: u32 = 0x4000;
/// `COLOUR_WHITE`
const COLOUR_WHITE: i32 = 1;

/// borg-io.c globals.
#[derive(Default)]
pub struct IoState {
    /// The keys to send (circular queue, `borg_key_queue`).
    key_queue: Vec<keycode_t>,
    key_head: i16,
    key_tail: i16,
    /// Keys already sent (`borg_key_history`).
    key_history: Vec<keypress>,
    key_history_head: i16,
    /// Direction to give if the game asks for one (`borg_queued_direction`).
    queued_direction: keycode_t,
    /// `borg_confirm_target`
    pub confirm_target: bool,
}

/// Read `n` characters of screen text at (x, y) (`borg_what_text`).
///
/// Reading stops at the first character whose attribute differs from the
/// first non-space character's; the attribute goes to `a`. The text is
/// converted with `wcstombs` into the caller's byte buffer `s`, reserving
/// one byte for NUL. `n` counts screen columns, not UTF-8 bytes. Returns 1
/// on an empty output buffer or if fewer than `n` (n > 0) characters were
/// read, else 0. A conversion error yields an empty string and follows the
/// length check (better-borg-c milestone7 patch 0068).
pub fn borg_what_text(x: i32, y: i32, n: i32, a: &mut u8, s: &mut [u8]) -> i32 {
    let mut screen_str: [wchar_t; 1024] = [0; 1024];
    let mut d_a: i32 = 0;
    let mut m = n.abs();
    let (w, _h) = engine::term_get_size();
    if x + m > w {
        m = w - x;
    }
    let mut i: i32 = 0;
    while i < m {
        let (t_a, t_c) = engine::screen_at(x + i, y);
        if t_c == ' ' as wchar_t || t_a == 0 {
            screen_str[i as usize] = ' ' as wchar_t;
        } else {
            if d_a != 0 {
                if t_a != d_a {
                    break;
                }
            } else {
                d_a = t_a;
            }
            screen_str[i as usize] = t_c;
        }
        i += 1;
    }
    screen_str[i.max(0) as usize] = 0;
    *a = d_a as u8;
    // Convert back to a char string that fits the buffer (the caller's
    // byte capacity, `size` in C); wcstombs() does not terminate a result
    // it had to truncate, and a conversion error leaves an empty string
    // (better-borg-c milestone7 patch 0068, source #287).
    // (No caller passes an empty buffer; this guard only avoids the
    // `size - 1` underflow in Rust.)
    if s.is_empty() {
        return 1;
    }
    let limit = s.len() - 1;
    let converted = engine::wcstombs(&mut s[..limit], &screen_str);
    s[if converted == usize::MAX { 0 } else { converted }] = 0;
    if n > 0 && i != n {
        return 1;
    }
    0
}

/// Log a message to a file (`borg_info`): does nothing.
fn borg_info(_what: &[u8]) {}

/// Memorize a message, log it, check it, and show it in the Borg windows
/// (`borg_note_internal`).
fn borg_note_internal(st: &mut BorgState, warning: bool, what: &[u8]) {
    let old = engine::term_current();
    if warning {
        engine::msg(st, what);
    } else {
        engine::message_add(what, MSG_GENERIC as u16);
        engine::event_signal_message_null(st, EVENT_MESSAGE, MSG_GENERIC as i32);
    }
    borg_info(what);

    // Mega-Hack -- Check against the swap loops
    if contains(what, b"Best Combo") || contains(what, b"Taking off ") {
        st.borg.antibounce_count = st.borg.antibounce_count.wrapping_add(10);
        let msg = format!("# Anti-loop variable tick ({}).", st.borg.antibounce_count);
        borg_note(st, &msg);
    }

    for j in 0..8 {
        let Some(t) = engine::angband_term(j) else {
            continue;
        };
        if engine::window_flag(j) & PW_BORG_1 == 0 {
            continue;
        }
        engine::term_activate(t);
        let (w, h) = engine::term_get_size();
        let (_x, mut y) = engine::term_locate();
        engine::term_erase(0, y, w);
        let mut what = what;
        let mut n = what.len() as i32;
        if n > w - 2 {
            while n > w - 2 {
                let mut k = w - 2;
                for i in (w / 2)..(w - 2) {
                    if c_isspace(what[i as usize]) {
                        k = i;
                    }
                }
                let mut buf = what[..k as usize].to_vec();
                buf.push(b'\\');
                engine::term_addstr(-1, COLOUR_WHITE, &buf);
                y += 1;
                if y >= h {
                    y = 0;
                }
                engine::term_erase(0, y, w);
                what = &what[k as usize..];
                n -= k;
            }
        }
        engine::term_addstr(-1, COLOUR_WHITE, what);
        y += 1;
        if y >= h {
            y = 0;
        }
        engine::term_erase(0, y, w);
        engine::term_fresh();
        engine::term_activate(old);
    }
}

fn contains(hay: &[u8], needle: &[u8]) -> bool {
    hay.windows(needle.len()).any(|w| w == needle)
}

/// Show a warning (`borg_warning`).
pub fn borg_warning(st: &mut BorgState, what: &str) {
    borg_note_internal(st, true, what.as_bytes());
}

/// Memorize and show a note (`borg_note`).
pub fn borg_note(st: &mut BorgState, what: &str) {
    borg_note_internal(st, false, what.as_bytes());
}

/// `borg_note` for text that may not be UTF-8 (bytes from the screen).
pub fn borg_note_bytes(st: &mut BorgState, what: &[u8]) {
    borg_note_internal(st, false, what);
}

/// Add a keypress to the queue (`borg_keypress`).
pub fn borg_keypress(st: &mut BorgState, k: keycode_t) -> i32 {
    if k == 0 {
        borg_note(st, " & Key * *BAD KEY * *");
        return -1;
    }
    let io = &mut st.io;
    io.key_queue[io.key_head as usize] = k;
    io.key_head += 1;
    if io.key_head as usize == KEY_SIZE {
        io.key_head = 0;
    }
    if io.key_head == io.key_tail {
        crate::borg_think::borg_oops(st, "overflow");
    }
    let io = &mut st.io;
    if io.key_tail as usize == KEY_SIZE {
        io.key_tail = 0;
    }
    0
}

/// Record a keypress returned to the game (`borg_save_keypress`).
pub fn borg_save_keypress(st: &mut BorgState, kp: keypress) -> keypress {
    if st.cfg(crate::borg_headers::borg_h::BORG_VERBOSE) != 0 {
        if kp.type_0 == EVT_KBRD {
            let k = kp.code;
            let msg = if (32..=126).contains(&k) {
                format!("& Key <{}> (0x{:02X})", k as u8 as char, k)
            } else if k == KC_ENTER {
                format!("& Key <Enter> (0x{:02X})", k)
            } else if k == ESCAPE {
                format!("& Key <Esc> (0x{:02X})", k)
            } else {
                format!("& Key <0x{:02X}>", k)
            };
            borg_note(st, &msg);
        } else {
            borg_note(st, &format!("& non-Keyboard <0x{:02X}>", kp.type_0));
        }
    }
    let io = &mut st.io;
    let h = io.key_history_head as usize;
    io.key_history[h].code = kp.code;
    io.key_history[h].type_0 = kp.type_0;
    io.key_history_head += 1;
    if io.key_history_head as usize == KEY_SIZE {
        io.key_history.copy_within(KEY_SIZE - 101..KEY_SIZE - 1, 0);
        io.key_history_head = 100;
    }
    kp
}

/// Queue a string of keys (`borg_keypresses`). Each byte is a C `char`
/// (signed), converted to a keycode the way C does it.
pub fn borg_keypresses(st: &mut BorgState, s: &[u8]) -> i32 {
    for &c in s {
        borg_keypress(st, c as i8 as i32 as keycode_t);
    }
    0
}

/// Get (and maybe take) the next queued key, 0 if none (`borg_inkey`).
pub fn borg_inkey(st: &mut BorgState, take: bool) -> keycode_t {
    let io = &mut st.io;
    if io.key_head == io.key_tail {
        return 0;
    }
    let i = io.key_queue[io.key_tail as usize];
    if !take {
        return i;
    }
    io.key_tail += 1;
    if io.key_tail as usize == KEY_SIZE {
        io.key_tail = 0;
    }
    i
}

/// Forget all queued keys (`borg_flush`).
pub fn borg_flush(st: &mut BorgState) {
    st.io.key_tail = st.io.key_head;
    st.io.queued_direction = 0;
}

/// Remember a direction in case the game asks (`borg_queue_direction`).
pub fn borg_queue_direction(st: &mut BorgState, k: keycode_t) {
    st.io.queued_direction = k;
    st.io.confirm_target = true;
}

/// Take the remembered direction (`borg_get_queued_direction`).
pub fn borg_get_queued_direction(st: &mut BorgState) -> keycode_t {
    let k = st.io.queued_direction;
    st.io.queued_direction = 0;
    k
}

/// Round-trip a name through wide characters, as the screen does, so it
/// compares equal to screen text (`borg_massage_special_chars`). The result
/// has `name.len() + 1` bytes (NUL-padded, like the zeroed C allocation).
pub fn borg_massage_special_chars(name: &[u8]) -> Vec<u8> {
    let len = name.len();
    let mut memory = vec![0u8; len + 1];
    let mut wide_name: [wchar_t; 1024] = [0; 1024];
    let mut src = name.to_vec();
    src.push(0);
    engine::text_mbstowcs(&mut wide_name, &src, (len + 1) as i32);
    engine::wcstombs(&mut memory, &wide_name);
    memory
}

/// Show the last `num` keys sent (`borg_dump_recent_keys`).
pub fn borg_dump_recent_keys(st: &mut BorgState, num: i32) {
    let end = i32::from(st.io.key_history_head);
    let mut start = if end < num { 0 } else { end - num };
    while start < end {
        let kp = st.io.key_history[start as usize];
        let msg = if kp.type_0 == EVT_KBRD {
            let k = kp.code;
            if (32..=126).contains(&k) {
                format!("& Key history <{}> (0x{:02X})", k as u8 as char, k)
            } else if k == KC_ENTER {
                format!("& Key history <Enter> (0x{:02X})", k)
            } else if k == ESCAPE {
                format!("& Key history <Esc> (0x{:02X})", k)
            } else {
                format!("& Key history <0x{:02X}>", k)
            }
        } else {
            format!("& non-Keyboard <0x{:02X}>", kp.type_0)
        };
        borg_note(st, &msg);
        start += 1;
    }
}

/// The bell should never sound while the Borg runs (`borg_bell`).
fn borg_bell_impl(st: &mut BorgState) {
    borg_note(st, "** BELL SOUNDED Dumping keypress history ***");
    borg_dump_recent_keys(st, 20);
    if st.cfg(crate::borg_headers::borg_h::BORG_STOP_ON_BELL) != 0 {
        crate::borg_think::borg_oops(st, "** BELL SOUNDED***");
    }
}

/// Engine event handler for `EVENT_BELL`.
extern "C" fn borg_bell(
    _unused: game_event_type,
    _data: *mut game_event_data,
    _user: *mut core::ffi::c_void,
) {
    crate::engine::enter(borg_bell_impl);
}

/// Set up the key queue and history (`borg_init_io`).
pub fn borg_init_io(st: &mut BorgState) {
    borg_free_io(st);
    let zero = keypress { type_0: 0, code: 0, mods: 0 };
    st.io.key_queue = vec![0; KEY_SIZE];
    st.io.key_history = vec![zero; KEY_SIZE];
    engine::event_add_handler(EVENT_BELL, borg_bell);
}

/// Release the key queue and history (`borg_free_io`).
pub fn borg_free_io(st: &mut BorgState) {
    engine::event_remove_handler(EVENT_BELL, borg_bell);
    st.io.key_history = Vec::new();
    st.io.key_queue = Vec::new();
}
