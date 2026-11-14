//! Port of `borg-messages-react.c`: react to messages that have been
//! received and parsed.
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

use crate::borg_io::{
    borg_dump_recent_keys, borg_flush, borg_get_queued_direction, borg_inkey, borg_note,
    borg_note_bytes, ESCAPE,
};
use crate::borg_headers::borg_h::{BORG_CHEAT_DEATH, BORG_DUMP_LEVEL, BORG_VERBOSE};
use crate::borg_headers::borg_trait_h::{BI_CLEVEL, BI_ISPARALYZED};
use crate::headers::ui_event_h::keycode_t;
use crate::headers::z_type_h::loc;
use crate::state::BorgState;
use crate::engine;

/// The queue of parsed messages waiting for `borg_update()`.
///
/// The queue globals are defined in borg-messages.c, which allocates them
/// (`borg_init_messages`) and frees them (`borg_free_messages`); this
/// module fills them. The texts are packed into `buf`, each followed by a
/// NUL; message `i` starts at `buf[pos[i]]`. The sizes keep the C types
/// and the C limits (`siz`, `max`) are checked against the counters, not
/// against the `Vec` lengths (an empty `Vec` is the C NULL).
#[derive(Default)]
pub struct MsgState {
    /// `borg_msg_len`: bytes of `buf` in use
    pub len: i16,
    /// `borg_msg_siz`: size of `buf`
    pub siz: i16,
    /// `borg_msg_buf`
    pub buf: Vec<u8>,
    /// `borg_msg_num`: messages queued
    pub num: i16,
    /// `borg_msg_max`: room in `pos`/`use_`
    pub max: i16,
    /// `borg_msg_pos`: start of each message in `buf`
    pub pos: Vec<i16>,
    /// `borg_msg_use`: which pass of `borg_update()` used each message
    pub use_: Vec<i16>,
}

/// `my_strcpy(dst, src, bufsize)` (z-util.c): copy at most `bufsize - 1`
/// bytes and a NUL; nothing when `bufsize` is 0.
fn my_strcpy(dst: &mut [u8], src: &[u8], bufsize: usize) {
    if bufsize > 0 {
        let n = src.len().min(bufsize - 1);
        dst[..n].copy_from_slice(&src[..n]);
        dst[n] = 0;
    }
}

/// Handle various "important" messages (`borg_react`).
///
/// Actually, we simply "queue" them for later analysis. `msg` is the game
/// message (for the verbose note), `buf` the parsed text to queue (without
/// its NUL).
///
/// Returns true when `buf` ended in ',' and the C code cut it in the
/// caller's buffer (it writes a NUL over the ',' through a cast-away
/// `const`); the queued copy never has the ','.
pub fn borg_react(st: &mut BorgState, msg: &[u8], buf: &[u8]) -> bool {
    if st.borg.dont_react || st.borg.tr(BI_ISPARALYZED) != 0 {
        borg_note(st, "# Ignoring messages.");
        return false;
    }

    // Note actual message
    if st.cfg(BORG_VERBOSE) != 0 {
        let mut note = b"> Reacting Msg (".to_vec();
        note.extend_from_slice(msg);
        note.push(b')');
        borg_note_bytes(st, &note);
    }

    // trim off trailing , if there is one, seems to have been introduced to
    // some uniques messages
    let (text, cut) = cut_comma(buf);

    if let Err(note) = st.msgs.push(text) {
        borg_note(st, note);
    }
    cut
}

/// `buf` without a trailing ',', and whether there was one.
fn cut_comma(buf: &[u8]) -> (&[u8], bool) {
    match buf.split_last() {
        Some((b',', text)) => (text, true),
        _ => (buf, false),
    }
}

impl MsgState {
    /// Queue `text` (the tail of `borg_react`). When there is no room,
    /// queues nothing and returns the note to show.
    fn push(&mut self, text: &[u8]) -> Result<(), &'static str> {
        // Extract length of parsed message
        let len = text.len() as i32;

        // Verify space
        if i32::from(self.num) + 1 > i32::from(self.max) {
            return Err("too many messages");
        }

        // Verify space
        if i32::from(self.len) + len + 1 > i32::from(self.siz) {
            return Err("too much messages");
        }

        let num = self.num as usize;
        let off = self.len as usize;

        // Assume not used yet
        self.use_[num] = 0;

        // Save the message position
        self.pos[num] = self.len;

        // Save the message text
        let bufsize = (i32::from(self.siz) - i32::from(self.len)) as usize;
        my_strcpy(&mut self.buf[off..], text, bufsize);

        // Advance the buf
        self.len = (i32::from(self.len) + (len + 1)) as i16;

        // Advance the pos
        self.num += 1;

        Ok(())
    }
}

/// Handle various messages that need response (`borg_react_prompted`).
///
/// `buf` is the prompt text at the top of the screen. Returns the key to
/// answer with (C: stores it in `key->code` and returns true), or `None`
/// (C: returns false and leaves the key alone).
pub fn borg_react_prompted(st: &mut BorgState, buf: &[u8], cursor: loc) -> Option<keycode_t> {
    let verbose = |st: &BorgState| st.cfg(BORG_VERBOSE) != 0;

    if cursor.y == 0 {
        // Mega-Hack -- Catch "Die? [y/n]" messages
        // If there is text on the first line...
        // And the game does not want a command...
        // And the cursor is on the top line...
        // And the text acquired above is "Die?"
        if buf.starts_with(b"Die?") && st.borg.status.cheat_death {
            // Flush messages
            crate::borg_messages::borg_parse(st, None);

            // flush the buffer
            borg_flush(st);

            // Take note
            borg_note(st, "# Cheating death...");

            // Dump the Character Map
            if st.borg.tr(BI_CLEVEL) >= st.cfg(BORG_DUMP_LEVEL)
                || engine::player_died_from().windows(10).any(|w| w == b"starvation")
            {
                crate::borg_log::borg_write_map(st, false);
            }

            // Log the death
            crate::borg_log::borg_log_death(st);
            crate::borg_log::borg_log_death_data(st);

            // Flush keys
            borg_flush(st);

            if st.cfg(BORG_CHEAT_DEATH) == 0 {
                crate::borg_reincarnate::borg_reincarnate_start(st);
                return Some(b'y'.into());
            } else {
                engine::do_cmd_wiz_cure_all(st);
                st.borg.time.now = 100;
                return Some(b'n'.into());
            }
        }

        // with 292, there is a flush(0, 0, 0) introduced as it asks for
        // confirmation. This flush is messing up the borg.  This will allow
        // the borg to work around the flush Attempt to catch "Attempt it
        // anyway? [y/n]"
        if buf.starts_with(b"Atte") {
            // Return the confirmation
            if verbose(st) {
                borg_note(st, "# Confirming use of Spell/Prayer when low on mana.");
            }
            return Some(b'y'.into());
        }

        // Wearing two rings.  Place this on the left hand
        if buf.starts_with(b"(Equip: c-d,") {
            // Left hand
            if verbose(st) {
                borg_note(st, "# Putting ring on the left hand.");
            }
            return Some(b'c'.into());
        }

        // This frequently catches when the key queue has become messed up
        // The borg will press a key and get a "Direction?" type prompt and
        // not be expecting it. The borg should halt in that case so the bad
        // code can be found and cleaned up.
        // (C: `strncmp(buf, "Direction", 9) == 0`.)
        if borg_inkey(st, false) == 0 && buf.starts_with(b"Direction") {
            if st.io.confirm_target {
                if verbose(st) {
                    borg_note(st, "# Expected request for Direction.");
                }
                // reset the flag
                st.io.confirm_target = false;
                // Return queued target (C: `return key;`, a non-NULL
                // pointer, so true)
                return Some(borg_get_queued_direction(st));
            } else {
                borg_note(st, "** UNEXPECTED REQUEST FOR DIRECTION Dumping keypress history ***");
                let mut note = b"** line starting <".to_vec();
                note.extend_from_slice(buf);
                note.extend_from_slice(b"> ***");
                borg_note_bytes(st, &note);
                borg_dump_recent_keys(st, 20);
                crate::borg_think::borg_oops(st, "unexpected request for direction");
                return Some(ESCAPE);
            }
        }

        // Stepping on a stack when the inventory is full gives a message
        // and the keypress when given this message is requeued so the borg
        // thinks it is a user keypress when it isn't
        if buf.starts_with(b"You have no room") {
            if verbose(st) {
                borg_note(st, "# 'You have no room' is a no-op");
            }
            // key code 0 seems to be a no-op and ignored as a user keypress
            return Some(0);
        }

        // This will be hit if the borg uses am item with the WONDER effect
        // and it picks banishment.  Always just escape out. Banishment is
        // too dangerous to randomly pick
        if borg_inkey(st, false) == 0
            && buf.starts_with(b"Choose a monster race (by symbol) to banish")
        {
            if verbose(st) {
                borg_note(st, "# Use of Wonder selected BANISHMENT");
            }
            // (C: `return key;`, true)
            return Some(ESCAPE);
        }

        // prompt for stepping in lava.  This should be avoided but
        // if the borg is stuck, give him a pass
        if buf.starts_with(b"The lava will") || buf.starts_with(b"Lava blocks y") {
            if verbose(st) {
                borg_note(st, "# ignoring Lava warning");
            }
            // yes step in
            return Some(b'y'.into());
        }

        // prompt for recall depth
        if borg_inkey(st, false) == 0 && buf.starts_with(b"Set recall depth") {
            if verbose(st) {
                borg_note(st, "# Use of unknown object with recall");
            }
            // the first selection (a) is random
            return Some(b'n'.into());
        }

        // Sometimes the borg will overshoot the range limit of his shooter
        if buf.starts_with(b"Target out of range") {
            // Fire Anyway? [Y/N]
            // yes step in
            return Some(b'y'.into());
        }

        // Word of Recall -- cancel
        if buf.starts_with(b"Word of Recall is already") {
            // this shouldn't happen but can if the borg is "playing with"
            // and unknown rod
            if !st.borg.trying_unknown {
                crate::borg_think::borg_oops(st, "unexpected double Word of Recall");
            }

            // no canceling it
            return Some(b'n'.into());
        }

        // exit sub screens

        // Parse equipment mode
        // this shouldn't happen because we now pull equipment information
        // directly from the game
        if buf == b"(Equipment) " {
            // Leave this mode
            return Some(ESCAPE);
        }

        // Parse Inventory mode
        // this shouldn't happen because we now pull inventory information
        // directly from the game
        if buf == b"(Inventory) " {
            // Leave this mode
            return Some(ESCAPE);
        }

        // Parse worn equipment mode
        // this shouldn't happen because we now pull worn equipment
        // information directly from the game
        if buf == b"Wear o" {
            // Leave this mode
            return Some(ESCAPE);
        }

        // Check for "browse spells" mode
        // this shouldn't happen because we now pull spell information
        // directly from the game
        if buf == b"Lv Mana Fail" {
            // Leave this mode
            return Some(ESCAPE);
        }
    }

    // make sure inventory is used for throws
    if buf.starts_with(b"Throw which item? (Throw") {
        if verbose(st) {
            borg_note(st, "# switching to inventory for throws");
        }
        // yes step in
        return Some(b'/'.into());
    }

    // ***MEGA-HACK***
    // This will be hit if the borg uses an unidentified effect that has
    // EF_SELECT/multiple effects. Always pick "one of the following at
    // random" when the item is used post ID, an effect will be selected
    if cursor.y == 1 && borg_inkey(st, false) == 0 && buf.starts_with(b"Which effect?") {
        if verbose(st) {
            borg_note(st, "# Use of unknown object with multiple effects");
        }
        // the first selection (a) is random (C: `return key;`, true)
        return Some(b'a'.into());
    }

    None
}

/// Clear saved messages (`borg_clear_reactions`).
pub fn borg_clear_reactions(st: &mut BorgState) {
    st.msgs.len = 0;
    st.msgs.num = 0;
}

// The tests cover the queue only: notes need the game's terminal.
#[cfg(test)]
mod tests {
    use super::*;

    fn text(m: &MsgState, i: usize) -> &[u8] {
        let p = m.pos[i] as usize;
        let end = p + m.buf[p..].iter().position(|&c| c == 0).unwrap();
        &m.buf[p..end]
    }

    /// As `borg_init_messages` leaves it, with other sizes.
    fn queue(siz: i16, max: i16) -> MsgState {
        MsgState {
            siz,
            buf: vec![0; siz as usize],
            max,
            pos: vec![0; max as usize],
            use_: vec![7; max as usize],
            ..MsgState::default()
        }
    }

    #[test]
    fn cut() {
        assert_eq!(cut_comma(b"KILL:Grip,"), (&b"KILL:Grip"[..], true));
        assert_eq!(cut_comma(b"a,,"), (&b"a,"[..], true));
        assert_eq!(cut_comma(b","), (&b""[..], true));
        assert_eq!(cut_comma(b""), (&b""[..], false));
        assert_eq!(cut_comma(b"SELF:lite"), (&b"SELF:lite"[..], false));
    }

    #[test]
    fn push_and_clear() {
        let mut m = queue(4096, 256);
        assert!(m.push(b"SELF:lite").is_ok());
        assert!(m.push(b"KILL:Grip").is_ok());
        assert!(m.push(b"").is_ok());
        assert_eq!((m.num, m.len), (3, 10 + 10 + 1));
        assert_eq!(&m.pos[..3], &[0, 10, 20]);
        assert_eq!(&m.use_[..4], &[0, 0, 0, 7]);
        assert_eq!(text(&m, 0), b"SELF:lite");
        assert_eq!(text(&m, 1), b"KILL:Grip");
        assert_eq!(text(&m, 2), b"");
        let mut st = BorgState { msgs: m, ..BorgState::default() };
        borg_clear_reactions(&mut st);
        assert_eq!((st.msgs.num, st.msgs.len), (0, 0));
    }

    #[test]
    fn limits() {
        // len + text + NUL == siz still fits; my_strcpy writes the NUL
        let mut m = queue(8, 4);
        m.buf = vec![0xff; 8];
        assert!(m.push(b"abcdefg").is_ok());
        assert_eq!((m.num, m.len), (1, 8));
        assert_eq!(m.buf, b"abcdefg\0");
        assert_eq!(m.push(b""), Err("too much messages"));
        assert_eq!((m.num, m.len), (1, 8));
        // one byte too long
        let mut m = queue(8, 4);
        assert_eq!(m.push(b"abcdefgh"), Err("too much messages"));
        assert_eq!((m.num, m.len), (0, 0));
        // message count is checked first
        let mut m = queue(8, 1);
        assert!(m.push(b"a").is_ok());
        assert_eq!(m.push(b"abcdefghijk"), Err("too many messages"));
        // freed queue (borg_free_messages: siz 0, NULL buffers, len kept)
        let mut m = MsgState { len: 5, max: 256, ..MsgState::default() };
        assert_eq!(m.push(b"x"), Err("too much messages"));
        assert_eq!(m.num, 0);
    }
}
