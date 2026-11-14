//! Port of `borg-messages.c`: read and parse the messages that come from
//! the game.
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
//! The message queue that borg-messages.c defines (`borg_msg_*`) lives in
//! `borg_messages_react::MsgState` (`st.msgs`); this file allocates and
//! frees it. Its own globals and file statics (`borg_match`, the message
//! tables, the `borg_parse` buffer) live in `MessagesState`
//! (`st.messages`).
//!
//! Texts are byte strings without their NUL. Where C formats into a fixed
//! buffer with `strnfmt` the same truncation is applied (`strnfmt_s`).

use crate::borg::BorgTemp;
use crate::borg_io::{borg_flush, borg_keypress, borg_note, borg_note_bytes, borg_what_text, ESCAPE};
use crate::borg_messages_react::{borg_react, borg_react_prompted};
use crate::borg_util::borg_trim;
use crate::borg_headers::borg_h::BORG_VERBOSE;
use crate::borg_headers::borg_trait_h::{
    BI_CDEPTH, BI_CLEVEL, BI_ISCONFUSED, BI_ISFORGET, BI_ISIMAGE, BI_LIGHT,
};
use crate::headers::list_message_h::{MSG_FLEE, MSG_GENERIC, MSG_KILL};
use crate::headers::list_mon_message_h::{
    MON_MSG_0, MON_MSG_10, MON_MSG_20, MON_MSG_35, MON_MSG_50, MON_MSG_75, MON_MSG_95,
    MON_MSG_DISAPPEAR, MON_MSG_MAX,
};
use crate::headers::list_options_h::OPT_cheat_live;
use crate::headers::list_terrain_h::{
    FEAT_BROKEN, FEAT_GRANITE, FEAT_MAGMA, FEAT_MAGMA_K, FEAT_OPEN, FEAT_PERM, FEAT_QUARTZ,
    FEAT_QUARTZ_K, FEAT_RUBBLE,
};
use crate::headers::ui_event_h::keypress;
use crate::headers::z_type_h::loc;
use crate::state::BorgState;
use crate::{borg_flow_kill, engine};

/// `GOAL_KILL` (borg-flow.h): the goal is a monster.
const GOAL_KILL: i32 = 1;

/// `COLOUR_DARK` (z-color.h)
const COLOUR_DARK: u8 = 0;

/// Size of the `borg_parse` buffer (`static char buf[1024]`).
const PARSE_BUF_SIZE: usize = 1024;

/// One message pattern (`struct borg_read_message`): up to three parts
/// that must all appear in a message (`None` = NULL). The C arrays end in
/// an entry whose `p1` is NULL; it is kept (`p1: None`) and the scans stop
/// at the first such entry, as in C.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct ReadMessage {
    pub p1: Option<Vec<u8>>,
    pub p2: Option<Vec<u8>>,
    pub p3: Option<Vec<u8>>,
    /// `index[i]` of `struct borg_read_messages`: the spell index (or the
    /// blow's message type)
    pub index: i32,
}

/// borg-messages.c globals and file statics.
pub struct MessagesState {
    /// `borg_match`: a message containing this text cancels the Borg
    /// (`char[128]`, NUL-terminated; empty = off)
    pub match_: [u8; 128],
    /// `len` (static in `borg_parse`): bytes of `parse_buf` in use
    pub parse_len: i32,
    /// `buf` (static in `borg_parse`): the message being collected
    /// (1024 bytes)
    pub parse_buf: Vec<u8>,
    /// `suffix_pain`: monster pain and status message endings (`None` =
    /// NULL)
    pub suffix_pain: Option<Vec<Vec<u8>>>,
    /// `suffix_hit_by`: the ways monsters hit the player (`None` = NULL
    /// `messages`)
    pub hit_by: Option<Vec<ReadMessage>>,
    /// `spell_msgs`: monster spells seen being cast
    pub spell: Option<Vec<ReadMessage>>,
    /// `spell_invis_msgs`: monster spells cast unseen
    pub spell_invis: Option<Vec<ReadMessage>>,
}

impl Default for MessagesState {
    fn default() -> Self {
        let mut match_ = [0u8; 128];
        match_[..15].copy_from_slice(b"plain gold ring");
        MessagesState {
            match_,
            parse_len: 0,
            parse_buf: vec![0; PARSE_BUF_SIZE],
            suffix_pain: None,
            hit_by: None,
            spell: None,
            spell_invis: None,
        }
    }
}

/// Methods of killing a monster (`prefix_kill`, order not important).
///
/// See "mon_take_hit()" for details.
const PREFIX_KILL: [&[u8]; 3] = [b"You have killed ", b"You have slain ", b"You have destroyed "];

/// Methods of monster death (`suffix_died`, order not important).
///
/// See "project_m()", "do_cmd_fire()", "mon_take_hit()" for details.
const SUFFIX_DIED: [&[u8]; 17] = [
    b" die.",
    b" dies.",
    b" is destroyed.",
    b" are destroyed.",
    b" is destroyed!",
    b" are destroyed!",
    b" shrivel away in the light!",
    b" shrivels away in the light!",
    b" dissolve!",
    b" dissolves!",
    b" scream of agony!",
    b" screams of agony!",
    b" disintegrate!",
    b" disintegrates!",
    b" freeze and shatter!",
    b" freezes and shatters!",
    b" is drained dry!",
];

/// `suffix_blink`
const SUFFIX_BLINK: [&[u8]; 5] = [
    b" disappears!",             // from teleport other
    b" intones strange words.",  // from polymorph spell
    b" teleports away.",         // RF6_TPORT
    b" blinks.",                 // RF6_BLINK
    b" makes a soft 'pop'.",
];

/// Spontaneous level feelings (`prefix_feeling_danger`, order important).
///
/// See "do_cmd_feeling()" for details.
const PREFIX_FEELING_DANGER: [&[u8]; 10] = [
    b"You are still uncertain about this place",
    b"Omens of death haunt this place",
    b"This place seems murderous",
    b"This place seems terribly dangerous",
    b"You feel anxious about this place",
    b"You feel nervous about this place",
    b"This place does not seem too risky",
    b"This place seems reasonably safe",
    b"This seems a tame, sheltered place",
    b"This seems a quiet, peaceful place",
];

/// `suffix_feeling_stuff`
const SUFFIX_FEELING_STUFF: [&[u8]; 11] = [
    b"Looks like any other level.",
    b"you sense an item of wondrous power!",
    b"there are superb treasures here.",
    b"there are excellent treasures here.",
    b"there are very good treasures here.",
    b"there are good treasures here.",
    b"there may be something worthwhile here.",
    b"there may not be much interesting here.",
    b"there aren't many treasures here.",
    b"there are only scraps of junk here.",
    b"there is naught but cobwebs here.",
];

/// `borg_msg_repository` (from list-mon-message.h): `(msg, type)` for
/// every `MON_MSG_*` up to and including `MON_MSG_MAX`.
const MON_MSG_REPOSITORY: [(&[u8], u32); 63] = [
    (b"[is|are] hurt.", MSG_GENERIC),
    (b"die[s].", MSG_KILL),
    (b"[is|are] destroyed.", MSG_KILL),
    (b"resist[s] a lot.", MSG_GENERIC),
    (b"[is|are] hit hard.", MSG_GENERIC),
    (b"resist[s].", MSG_GENERIC),
    (b"[is|are] immune.", MSG_GENERIC),
    (b"resist[s] somewhat.", MSG_GENERIC),
    (b"[is|are] unaffected!", MSG_GENERIC),
    (b"spawn[s]!", MSG_GENERIC),
    (b"look[s] healthier.", MSG_GENERIC),
    (b"fall[s] asleep!", MSG_GENERIC),
    (b"wake[s] up.", MSG_GENERIC),
    (b"cringe[s] from the light!", MSG_GENERIC),
    (b"shrivel[s] away in the light!", MSG_KILL),
    (b"lose[s] some skin!", MSG_GENERIC),
    (b"dissolve[s]!", MSG_KILL),
    (b"catch[es] fire!", MSG_GENERIC),
    (b"[is|are] badly frozen.", MSG_GENERIC),
    (b"shudder[s].", MSG_GENERIC),
    (b"change[s]!", MSG_GENERIC),
    (b"disappear[s]!", MSG_GENERIC),
    (b"[is|are] even more stunned.", MSG_GENERIC),
    (b"[is|are] stunned.", MSG_GENERIC),
    (b"[is|are] no longer stunned.", MSG_GENERIC),
    (b"look[s] more confused.", MSG_GENERIC),
    (b"look[s] confused.", MSG_GENERIC),
    (b"[is|are] no longer confused.", MSG_GENERIC),
    (b"look[s] more slowed.", MSG_GENERIC),
    (b"look[s] slowed.", MSG_GENERIC),
    (b"speed[s] up.", MSG_GENERIC),
    (b"look[s] even faster!", MSG_GENERIC),
    (b"start[s] moving faster.", MSG_GENERIC),
    (b"slow[s] down.", MSG_GENERIC),
    (b"look[s] more terrified!", MSG_GENERIC),
    (b"flee[s] in terror!", MSG_FLEE),
    (b"[is|are] no longer afraid.", MSG_GENERIC),
    (b"[is|are] frozen to the spot!", MSG_GENERIC),
    (b"can move again.", MSG_GENERIC),
    (b"seem[s] less magical!", MSG_GENERIC),
    (b"seem[s] magical again.", MSG_GENERIC),
    (b"falls under your spell!", MSG_GENERIC),
    (b"is no longer under your control.", MSG_GENERIC),
    (b"shimmers for a moment.", MSG_GENERIC),
    (b"You hear [a|several] scream[|s] of agony!", MSG_KILL),
    (b"disintegrate[s]!", MSG_KILL),
    (b"freeze[s] and shatter[s]!", MSG_KILL),
    (b"lose[s] some mana!", MSG_GENERIC),
    (b"look[s] briefly puzzled.", MSG_GENERIC),
    (b"maintain[s] the same shape.", MSG_GENERIC),
    (b"[is|are] unharmed.", MSG_GENERIC),
    (b"appear[s]!", MSG_GENERIC),
    (b"There is a puff of smoke!", MSG_GENERIC),
    (b"[is|are] embedded in rock!", MSG_KILL),
    (b"wail[s] out in pain!", MSG_GENERIC),
    (b"", MSG_GENERIC),
    (b"", MSG_GENERIC),
    (b"", MSG_GENERIC),
    (b"", MSG_GENERIC),
    (b"", MSG_GENERIC),
    (b"", MSG_GENERIC),
    (b"", MSG_GENERIC),
    (b"", MSG_GENERIC),
];

// ---------------------------------------------------------------------------
// C string helpers

/// The C string in `buf` (up to the first NUL). Panics if there is none
/// (C would read past the buffer).
fn cstr(buf: &[u8]) -> &[u8] {
    let n = buf.iter().position(|&c| c == 0).expect("unterminated C string");
    &buf[..n]
}

/// `prefix(s, t)` (z-util.c)
fn prefix(s: &[u8], t: &[u8]) -> bool {
    s.starts_with(t)
}

/// `suffix(s, t)` (z-util.c)
fn suffix(s: &[u8], t: &[u8]) -> bool {
    s.ends_with(t)
}

/// `streq(s, t)`
fn streq(s: &[u8], t: &[u8]) -> bool {
    s == t
}

/// `strstr(hay, needle)` as an offset (an empty needle matches at 0).
fn strstr(hay: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.is_empty() {
        return Some(0);
    }
    hay.windows(needle.len()).position(|w| w == needle)
}

/// A `%s` argument of `vstrnfmt` (z-form.c): copied into a 1024-byte
/// buffer first, so at most 1023 bytes.
fn fmt_arg(s: &[u8]) -> &[u8] {
    &s[..s.len().min(1023)]
}

/// `strnfmt(dst, max, "%s", src)` (z-form.c): copies the text until
/// `max - 1` bytes are written, then a NUL; returns the bytes written.
/// With `max == 0` nothing stops the copy (as in C, where `max - 1` wraps);
/// a text longer than `dst` then panics where C overflows the buffer.
fn strnfmt_s(dst: &mut [u8], max: usize, src: &[u8]) -> usize {
    let mut n = 0;
    for &c in fmt_arg(src) {
        if n == max.wrapping_sub(1) {
            break;
        }
        dst[n] = c;
        n += 1;
    }
    dst[n] = 0;
    n
}

/// `strnfmt(who, max, "%s", src)` into the 256-byte `who` of
/// `borg_parse_aux`; returns the text.
fn who_text(max: usize, src: &[u8]) -> Vec<u8> {
    let mut who = [0u8; 256];
    let n = strnfmt_s(&mut who, max, src);
    who[..n].to_vec()
}

/// `strnfmt(buf, 256, "<head>%s", who)`: the text cut at 255 bytes.
fn tagged(head: &[u8], who: &[u8]) -> Vec<u8> {
    let mut v = head.to_vec();
    v.extend_from_slice(fmt_arg(who));
    v.truncate(255);
    v
}

/// `size_t` of a C `int` expression (negative values wrap, as in C).
fn size_of_int(v: i32) -> usize {
    v as isize as usize
}

/// `borg_message_contains`: every part of the pattern appears in `value`.
fn borg_message_contains(value: &[u8], message: &ReadMessage) -> bool {
    let has = |p: &Option<Vec<u8>>| strstr(value, p.as_deref().expect("message part")).is_some();
    has(&message.p1)
        && (message.p2.is_none() || has(&message.p2))
        && (message.p3.is_none() || has(&message.p3))
}

/// The entries of a pattern table up to its NULL end (C loops `for (i =
/// 0; msgs.messages[i].message_p1; i++)`). Panics if the table is not
/// allocated (C dereferences NULL).
fn patterns(t: &Option<Vec<ReadMessage>>) -> impl Iterator<Item = &ReadMessage> {
    t.as_ref().expect("message table not initialised").iter().take_while(|m| m.p1.is_some())
}

// ---------------------------------------------------------------------------
// Reading the screen

/// `(Term->wid - 1) / tile_width`: the width of the message line.
fn message_line_width() -> i32 {
    let g = engine::term_geometry();
    (g.wid - 1) / g.tile_width
}

/// Read the messages from the top of the screen (`borg_get_messages`).
///
/// Returns true when `key` holds the key to send.
pub fn borg_get_messages(st: &mut BorgState, key: &mut keypress, cursor: loc) -> bool {
    let mut borg_prompt = false;

    let mut buffer = [0u8; 1024];
    let mut t_a: u8 = 0;

    // get everything on the message line
    borg_what_text(0, 0, message_line_width(), &mut t_a, &mut buffer);

    // Trim whitespace
    let off = borg_trim(&mut buffer);

    // Mega-Hack -- check for possible prompts/messages
    // If the first four characters on the message line all
    // have the same attribute (or are all spaces), and they
    // are not all spaces (ascii value 0x20)...
    if t_a != COLOUR_DARK
        && (buffer[off] != b' '
            || buffer[off + 1] != b' '
            || buffer[off + 2] != b' '
            || buffer[off + 3] != b' ')
    {
        // Assume a prompt/message is available
        borg_prompt = true;
    }

    // check for strings starting "Type" because those are also prompts
    if borg_prompt && prefix(cstr(&buffer[off..]), b"Type") {
        borg_prompt = false;
    }

    // handle the messages the borg has to react to immediately
    if borg_prompt && !engine::inkey_flag() && !cstr(&buffer[off..]).is_empty() {
        let text = cstr(&buffer[off..]).to_vec();
        if let Some(code) = borg_react_prompted(st, &text, cursor) {
            key.code = code;
            return true;
        }
    }

    // Mega-Hack -- Catch "-more-" messages
    // If there is text on the first line...
    // And the game does not want a command...
    // And the cursor is on the top line...
    // And there is text before the cursor...
    // And that text is "-more-"
    if borg_prompt
        && !engine::inkey_flag()
        && cursor.y == 0
        && cursor.x >= 7
        && 0 == borg_what_text(cursor.x - 7, cursor.y, 7, &mut t_a, &mut buffer)
        && suffix(cstr(&buffer), b" -more-")
    {
        if st.cfg(BORG_VERBOSE) != 0 {
            borg_note(st, "# message with -more-");
        }

        // Get the message
        if 0 == borg_what_text(0, 0, cursor.x - 7, &mut t_a, &mut buffer) {
            // Parse it
            let text = cstr(&buffer).to_vec();
            borg_parse(st, Some(&text));
        }
        // Clear the message
        if st.cfg(BORG_VERBOSE) != 0 {
            borg_note(st, "clearing -more-");
        }
        key.code = u32::from(b' ');
        return true;
    }

    // Catch normal messages
    // If there is text on the first line...
    // And the game wants a command
    if borg_prompt && engine::inkey_flag() {
        if st.cfg(BORG_VERBOSE) != 0 {
            borg_note(st, "# parse normal message");
        }
        // Get the message(s)
        if 0 == borg_what_text(0, 0, message_line_width(), &mut t_a, &mut buffer) {
            let mut k = cstr(&buffer).len();

            // Strip trailing spaces
            while k > 0 && buffer[k - 1] == b' ' {
                k -= 1;
            }

            // Terminate
            buffer[k] = 0;

            // Parse it
            let text = buffer[..k].to_vec();
            borg_parse(st, Some(&text));
        }

        // Clear the message
        key.code = u32::from(b' ');
        return true;
    }

    false
}

// ---------------------------------------------------------------------------
// Parsing

/// Forget the known glyphs and re-add the one under the Borg, if any
/// (cheat): the common tail of "you were hit" and "the rune is broken".
fn rebuild_glyphs(st: &mut BorgState) {
    let t = &mut st.glyph.track;
    // Remove the entire array
    for i in 0..t.num.max(0) as usize {
        t.x[i] = 0;
        t.y[i] = 0;
    }
    // no known glyphs
    t.num = 0;

    // Check for glyphs under player -- Cheat.  The position may still be on
    // the previous level when this is parsed just after a level change.
    let c = st.borg.c;
    if engine::square_in_bounds(c.x, c.y) && engine::square_iswarded(c.x, c.y) {
        let t = &mut st.glyph.track;
        t.x[t.num as usize] = c.x;
        t.y[t.num as usize] = c.y;
        t.num += 1;
    }
}

/// Delete every monster closer to the Borg than `near` (`d < near`), or
/// within `near` grids (`d <= near`), as the two loops of
/// `borg_parse_aux` do.
fn delete_kills_near(st: &mut BorgState, within: bool, near: i32) {
    // Examine all the monsters
    let mut i = 1;
    while i < i32::from(st.kills.nxt) {
        let kill = st.kills.kills[i as usize];

        // Skip dead monsters
        if kill.r_idx != 0 {
            // Now do distance considerations
            let x9 = kill.pos.x;
            let y9 = kill.pos.y;
            let c = st.borg.c;

            // Distance components
            let ax = if x9 > c.x { x9 - c.x } else { c.x - x9 };
            let ay = if y9 > c.y { y9 - c.y } else { c.y - y9 };

            // Distance
            let d = ax.max(ay);

            if (within && d <= near) || (!within && d < near) {
                // Kill em
                borg_flow_kill::borg_delete_kill(st, i);
            }
        }
        i += 1;
    }
}

/// A message that sets a temporary effect flag: (message prefix, the
/// flag, its new value).
type TempEffect = (&'static [u8], fn(&mut BorgTemp) -> &mut bool, bool);

/// Temporary effects, in C order.
const TEMP_EFFECTS: &[TempEffect] = &[
    // protect from evil
    (b"You feel safe from evil!", |t| &mut t.prot_from_evil, true),
    (b"You no longer feel safe from evil.", |t| &mut t.prot_from_evil, false),
    // haste self
    (b"You feel yourself moving faster!", |t| &mut t.fast, true),
    (b"You feel yourself slow down.", |t| &mut t.fast, false),
    // Bless
    (b"You feel righteous", |t| &mut t.bless, true),
    (b"The prayer has expired.", |t| &mut t.bless, false),
    // fastcast
    (b"You feel your mind accelerate.", |t| &mut t.fastcast, true),
    (b"You feel your mind slow again.", |t| &mut t.fastcast, false),
    // hero
    (b"You feel like a hero!", |t| &mut t.hero, true),
    (b"You no longer feel heroic.", |t| &mut t.hero, false),
    // berserk
    (b"You feel like a killing machine!", |t| &mut t.berserk, true),
    (b"You no longer feel berserk.", |t| &mut t.berserk, false),
];

/// Temporary resistances and the shield, in C order.
const TEMP_RESISTS: &[TempEffect] = &[
    // resist acid
    (b"You feel resistant to acid!", |t| &mut t.res_acid, true),
    (b"You are no longer resistant to acid.", |t| &mut t.res_acid, false),
    // resist electricity
    (b"You feel resistant to electricity!", |t| &mut t.res_elec, true),
    (b"You are no longer resistant to electricity.", |t| &mut t.res_elec, false),
    // resist fire
    (b"You feel resistant to fire!", |t| &mut t.res_fire, true),
    (b"You are no longer resistant to fire.", |t| &mut t.res_fire, false),
    // resist cold
    (b"You feel resistant to cold!", |t| &mut t.res_cold, true),
    (b"You are no longer resistant to cold.", |t| &mut t.res_cold, false),
    // resist poison
    (b"You feel resistant to poison!", |t| &mut t.res_pois, true),
    (b"You are no longer resistant to poison.", |t| &mut t.res_pois, false),
    // Shield
    (b"A mystic shield forms around your body!", |t| &mut t.shield, true),
    (b"Your skin turns to stone.", |t| &mut t.shield, true),
    (b"Your mystic shield crumbles away.", |t| &mut t.shield, false),
    (b"A fleshy shade returns to your skin.", |t| &mut t.shield, false),
];

/// Set the flag of the first entry whose text starts `msg`; returns
/// whether one matched.
fn set_temp(st: &mut BorgState, msg: &[u8], effects: &[TempEffect]) -> bool {
    for (text, flag, v) in effects {
        if prefix(msg, text) {
            *flag(&mut st.borg.temp) = *v;
            return true;
        }
    }
    false
}

/// `INVEN_WIELD` (`z_info->pack_size`)
fn inven_wield() -> usize {
    usize::from(engine::z_info().pack_size)
}

/// Parse one message (`borg_parse_aux`).
///
/// `msg` is the text, `len` its length as C passes it.
///
/// Note that detecting "death" is EXTREMELY important, to prevent
/// all sorts of errors arising from attempting to parse the "tomb"
/// screen, and to allow the user to "observe" the "cause" of death.
///
/// Note that detecting "failure" is EXTREMELY important, to prevent
/// bizarre situations after failing to use a staff of perceptions,
/// which would otherwise go ahead and send the "item index" which
/// might be a legal command (such as "a" for "aim").  This method
/// is necessary because the Borg cannot parse "prompts", and must
/// assume the success of the prompt-inducing command, unless told
/// otherwise by a failure message.  Also, we need to detect failure
/// because some commands, such as detection spells, need to induce
/// further processing if they succeed, but messages are only given
/// if the command fails.
///
/// Note that certain other messages may contain useful information,
/// and so they are "analyzed" and sent to "borg_react()", which just
/// queues the messages for later analysis in the proper context.
///
/// Along with the actual message, we send a special formatted buffer,
/// containing a leading "opcode", which may contain extra information,
/// such as the index of a spell, and an "argument" (for example, the
/// capitalized name of a monster), with a "colon" to separate them.
///
/// We notice a few "terrain feature" messages here so we can acquire
/// knowledge about wall types and door types.
fn borg_parse_aux(st: &mut BorgState, msg: &[u8], len: i32) {
    // `ag` (`&borg_grids[borg.goal.g.y][borg.goal.g.x]`), read at each use
    let g = st.borg.goal.g;
    let ag_feat = |st: &BorgState| i32::from(st.cave.grid(g.x, g.y).feat);
    let set_ag_feat = |st: &mut BorgState, f: u32| st.cave.grid_mut(g.x, g.y).feat = f as u8;

    // "who" after a prefix of `tmp` bytes:
    // `strnfmt(who, 1 + len - (tmp + 1), "%s", msg + tmp)`
    let who_after = |tmp: usize| who_text(size_of_int(1 + len - (tmp as i32 + 1)), &msg[tmp..]);

    // Log (if needed)
    if st.cfg(BORG_VERBOSE) != 0 {
        let mut note = b"# Parse Msg bite <".to_vec();
        note.extend_from_slice(fmt_arg(msg));
        note.push(b'>');
        borg_note_bytes(st, &note);
    }

    // Notice death
    if prefix(msg, b"You die.") {
        // Abort (unless cheating)
        if !(engine::player_wizard()
            || engine::player_opt(OPT_cheat_live)
            || st.borg.status.cheat_death)
        {
            // Abort
            crate::borg_think::borg_oops(st, "death");

            // Abort right now if not reincarnating
            if st.borg.status.respawning == 0 {
                st.borg.status.active = false;
            }
            // Noise XXX XXX XXX
            engine::term_xtra(engine::TERM_XTRA_NOISE, 1);
        }

        // Done
        return;
    }

    // Notice "failure"
    if prefix(msg, b"You failed ") {
        // Store the keypress
        borg_note(st, "# Normal failure.");

        // Set the failure flag
        st.update.failure = true;

        // Flush our key-buffer
        borg_flush(st);

        // If we were casting a targetted spell and failed
        // it does not mean we can't target that location
        st.fight_attack.successful_target = 0;

        // In case we failed our emergency use of MM
        st.io.confirm_target = false;

        // In case it was a Resistance refresh
        if st.fight_defend.attempting_refresh_resist {
            if st.borg.resistance > 1 {
                st.borg.resistance = (i32::from(st.borg.resistance) - 25000) as i16;
            }
            st.fight_defend.attempting_refresh_resist = false;
        }

        return;
    }

    // Mega-Hack -- Check against the search string
    if st.messages.match_[0] != 0 && strstr(msg, cstr(&st.messages.match_)).is_some() {
        // Clean cancel
        st.borg.status.cancel = true;
    }

    // Ignore teleport trap
    if prefix(msg, b"You hit a teleport") {
        return;
    }

    // Ignore arrow traps
    if prefix(msg, b"An arrow ") {
        return;
    }

    // Ignore dart traps
    if prefix(msg, b"A small dart ") {
        return;
    }

    if prefix(msg, b"The cave ") {
        borg_react(st, msg, b"QUAKE:Somebody");
        st.glyph.needs_new_sea = true;
        return;
    }

    if prefix(msg, b"You are too afraid to attack ") {
        let who = who_after(b"You are too afraid to attack ".len());
        borg_react(st, msg, &tagged(b"AFRAID:", &who));
        return;
    }

    // amnesia attacks, re-id wands, staves, equipment.
    if prefix(msg, b"You feel your memories fade.") {
        // Set the borg flag
        st.borg.trait_[BI_ISFORGET as usize] = 1;
    }
    if streq(msg, b"Your memories come flooding back.") {
        st.borg.trait_[BI_ISFORGET as usize] = 0;
    }

    if streq(msg, b"You have been knocked out.") {
        borg_note(st, "Ignoring Messages While KO'd");
        st.msgs.dont_react = true;
    }
    if streq(msg, b"You are paralyzed") {
        borg_note(st, "Ignoring Messages While Paralyzed");
        st.msgs.dont_react = true;
    }

    // Hallucination -- Open
    if streq(msg, b"You feel drugged!") {
        borg_note(st, "# Hallucinating.  Special control of wanks.");
        st.borg.trait_[BI_ISIMAGE as usize] = 1;
    }

    if streq(msg, b"The draining fails.") {
        borg_react(st, msg, b"MISS_BY:something");
        return;
    }

    // Hallucination -- Close
    if streq(msg, b"You can see clearly again.") {
        borg_note(st, "# Hallucination ended.  Normal control of wanks.");
        st.borg.trait_[BI_ISIMAGE as usize] = 0;
    }

    // Hit somebody
    if prefix(msg, b"You hit ") {
        let who = who_after(b"You hit ".len());
        borg_react(st, msg, &tagged(b"HIT:", &who));
        return;
    }
    if prefix(msg, b"You bite ") {
        let who = who_after(b"You bite ".len());
        borg_react(st, msg, &tagged(b"HIT:", &who));
        st.fight_attack.target_closest = 1;
        return;
    }
    if prefix(msg, b"You draw power from") {
        st.fight_attack.target_closest = 1;
        return;
    }
    if prefix(msg, b"No Available Target.") {
        st.fight_attack.target_closest = -12;
        return;
    }
    if prefix(msg, b"This spell must target a monster.") {
        st.fight_attack.target_closest = -12;
        return;
    }
    if prefix(msg, b"Not enough room next to ") {
        st.fight_attack.target_closest = -12;
        return;
    }

    // Miss somebody
    if prefix(msg, b"You miss ") {
        let who = who_after(b"You miss ".len());
        borg_react(st, msg, &tagged(b"MISS:", &who));
        return;
    }

    // Miss somebody (because of fear): never reached, the same prefix
    // returned above
    if prefix(msg, b"You are too afraid to attack ") {
        let who = who_after(b"You are too afraid to attack ".len());
        borg_react(st, msg, &tagged(b"MISS:", &who));
        return;
    }

    // "Your <equipment> is unaffected!"
    // Note that this check must be before the suffix_pain
    // because suffix_pain will look for 'is unaffected!' and
    // assume it is talking about a monster which in turn will
    // yield to the Player Ghost being created.
    if prefix(msg, b"Your ") {
        if suffix(msg, b" is unaffected!") {
            // Your equipment ignored the attack.
            // Ignore the message
            return;
        }
    } else if parse_monster_message(st, msg, len) {
        return;
    }

    // Feature XXX XXX XXX
    if streq(msg, b"The door appears to be broken.") {
        // Only process open doors
        if ag_feat(st) == FEAT_OPEN as i32 {
            // Mark as broken
            set_ag_feat(st, FEAT_BROKEN);

            // Clear goals
            st.borg.goal.type_0 = 0;
        }
        return;
    }

    // Feature XXX XXX XXX
    if streq(msg, b"This seems to be permanent rock.") {
        // Only process walls
        if ag_feat(st) >= FEAT_GRANITE as i32 && ag_feat(st) <= FEAT_PERM as i32 {
            // Mark the wall as permanent
            set_ag_feat(st, FEAT_PERM);

            // Clear goals
            st.borg.goal.type_0 = 0;
        }

        return;
    }

    // Feature XXX XXX XXX
    if streq(msg, b"You tunnel into the granite wall.") {
        // reseting my anti-bounce count
        st.borg.antibounce_count = 1;

        // Only process walls
        if ag_feat(st) >= FEAT_GRANITE as i32 && ag_feat(st) <= FEAT_PERM as i32 {
            // Mark the wall as granite
            set_ag_feat(st, FEAT_GRANITE);

            // Clear goals
            st.borg.goal.type_0 = 0;
        }

        return;
    }

    // Feature XXX XXX XXX
    if streq(msg, b"You tunnel into the quartz vein.") {
        if ag_feat(st) == FEAT_MAGMA_K as i32 {
            // Process magma veins with treasure
            set_ag_feat(st, FEAT_QUARTZ_K);
            st.borg.goal.type_0 = 0;
        } else if ag_feat(st) == FEAT_MAGMA as i32 {
            // Process magma veins
            set_ag_feat(st, FEAT_QUARTZ);
            st.borg.goal.type_0 = 0;
        }

        return;
    }

    // Feature XXX XXX XXX
    if streq(msg, b"You tunnel into the magma vein.") {
        if ag_feat(st) == FEAT_QUARTZ_K as i32 {
            // Process quartz veins with treasure
            set_ag_feat(st, FEAT_MAGMA_K);
            st.borg.goal.type_0 = 0;
        } else if ag_feat(st) == FEAT_QUARTZ as i32 {
            // Process quartz veins
            set_ag_feat(st, FEAT_MAGMA);
            st.borg.goal.type_0 = 0;
        }

        return;
    }

    // check for trying to dig when you can't
    if prefix(msg, b"You chip away futilely ") {
        // get rid of the goal monster we were chasing
        if i32::from(st.borg.goal.type_0) == GOAL_KILL {
            let k = st.cave.grid(g.x, g.y).kill;
            if k != 0 {
                borg_flow_kill::borg_delete_kill(st, i32::from(k));
            }
        }
        return;
    }

    // Word of Recall -- Ignition
    if prefix(msg, b"The air about you becomes ") {
        // Initiate recall
        // Guess how long it will take to lift off
        // Guess. game turns x 1000 ( 15+rand(20))
        st.borg.goal.recalling = 15000 + 5000;
        return;
    }

    // Deep Descent -- Ignition
    if prefix(msg, b"The air around you starts ") {
        // Initiate descent
        // Guess. game turns x 1000 ( 3+rand(4))
        st.borg.goal.descending = 3000 + 2000;
        return;
    }

    // Word of Recall -- Lift off
    if prefix(msg, b"You feel yourself yanked ") {
        // Flush our key-buffer
        // this is done in case the borg had been aiming a
        // shot before recall hit
        borg_flush(st);

        // Recall complete
        st.borg.goal.recalling = 0;
        return;
    }

    // Deep Descent  -- Lift off
    if prefix(msg, b"The floor opens beneath you!") {
        // Flush our key-buffer
        borg_flush(st);

        // Recall complete
        st.borg.goal.descending = 0;
        return;
    }

    // Word of Recall -- Cancelled
    if prefix(msg, b"A tension leaves ") {
        st.borg.goal.recalling = 0;
        return;
    }

    // Deep Descent -- Cancelled (only happens on death)
    if prefix(msg, b"The air around you stops ") {
        st.borg.goal.descending = 0;
        return;
    }

    // Wearing Cursed Item
    if prefix(msg, b"Oops!  It feels deathly cold!") {
        // this should only happen with STICKY items, The Crown of Morgoth
        // or The One Ring
        return;
    }

    // Temporary effects (each test returns when it matches)
    if set_temp(st, msg, TEMP_EFFECTS) {
        return;
    }

    // Sense Invisible
    if prefix(msg, b"Your eyes feel very sensitive!") {
        st.borg.see_inv = 30000;
        return;
    }
    if prefix(msg, b"Your eyes no longer feel so sensitive.") {
        st.borg.see_inv = 0;
        return;
    }

    // check for wall blocking but not when confused
    if prefix(msg, b"There is a wall ") && st.borg.tr(BI_ISCONFUSED) == 0 {
        st.borg.status.redraw = true;
        st.borg.status.need_alter = true;
        st.borg.goal.type_0 = 0;
        return;
    }

    // check for closed door but not when confused
    if prefix(msg, b"There is a closed door blocking your way.")
        && st.borg.tr(BI_ISCONFUSED) == 0
        && st.borg.tr(BI_ISIMAGE) == 0
    {
        st.borg.status.redraw = true;
        st.borg.status.need_alter = true;
        st.borg.goal.type_0 = 0;
        return;
    }

    // check for mis-alter command.  Sometime induced by never_move guys
    if prefix(msg, b"You spin around.") && st.borg.tr(BI_ISCONFUSED) == 0 {
        // if the guy is too close then delete him.
        delete_kills_near(st, false, 4);

        st.borg.status.no_alter = true;
        st.borg.goal.type_0 = 0;
        return;
    }

    // Check for the missing staircase
    if prefix(msg, b"No known path to ") || prefix(msg, b"Something is here.") {
        // make sure the borg does not think he's on one
        // Remove all stairs from the array.
        st.stairs.less.num = 0;
        st.stairs.more.num = 0;
        let c = st.borg.c;
        st.cave.grid_mut(c.x, c.y).feat = FEAT_BROKEN as u8;

        return;
    }

    // Feature XXX XXX XXX
    if prefix(msg, b"You see nothing there ") {
        set_ag_feat(st, FEAT_BROKEN);

        st.borg.status.no_alter = true;
        // Clear goals
        st.borg.goal.type_0 = 0;
        return;
    }

    // Hack to protect against clock overflows and errors
    if prefix(msg, b"Illegal ") {
        borg_keypress(st, ESCAPE);
        borg_keypress(st, ESCAPE);
        st.borg.antibounce_count = (i32::from(st.borg.antibounce_count) + 100) as i16;
        return;
    }

    // Hack to protect against clock overflows and errors
    if prefix(msg, b"You have nothing to identify") {
        borg_keypress(st, ESCAPE);
        borg_keypress(st, ESCAPE);
        st.borg.antibounce_count = (i32::from(st.borg.antibounce_count) + 100) as i16;

        // ID all items (equipment, INVEN_WIELD ..= INVEN_FEET)
        for i in inven_wield()..=inven_wield() + 11 {
            let item = &mut st.items.items[i];
            if item.iqty != 0 {
                item.ident = true;
            }
        }

        // ID all items (inventory; up to and including INVEN_WIELD)
        for i in 0..=inven_wield() {
            let item = &mut st.items.items[i];
            if item.iqty != 0 {
                item.ident = true;
            }
        }
        return;
    }

    // Hack to protect against clock overflows and errors (no return: the
    // tests below run too)
    if prefix(msg, b"Identifying The Phial") {
        // ID item (equipment, INVEN_LIGHT)
        st.items.items[inven_wield() + 5].ident = true;

        borg_keypress(st, ESCAPE);
        borg_keypress(st, ESCAPE);
        st.borg.antibounce_count = (i32::from(st.borg.antibounce_count) + 100) as i16;
    }

    // Temporary resistances and the shield (each test returns when it
    // matches)
    if set_temp(st, msg, TEMP_RESISTS) {
        return;
    }

    // Glyph of Warding (the spell no longer gives a report)
    // Sadly  Rune of Protection has no message
    if prefix(msg, b"You inscribe a mystic symbol on the ground!") {
        let c = st.borg.c;
        let t = &st.glyph.track;
        // Check for an existing glyph
        let mut i: i32 = 0;
        while i < i32::from(t.num) {
            // Stop if we already new about this glyph
            if t.x[i as usize] == c.x && t.y[i as usize] == c.y {
                break;
            }
            i += 1;
        }

        // Track the newly discovered glyph
        if i == i32::from(t.num) && i < i32::from(t.size) {
            borg_note(st, "# Noting the creation of a glyph.");
            let t = &mut st.glyph.track;
            t.x[i as usize] = c.x;
            t.y[i as usize] = c.y;
            t.num += 1;
        }

        return;
    }
    if prefix(msg, b"The rune of protection is broken!") {
        // we won't know which is broken so erase them all and
        // allow the borg to scan the screen and rebuild the array.
        // He won't see the one under him though.  So a special check
        // must be made.
        rebuild_glyphs(st);
        return;
    }
    // failed glyph spell message
    if prefix(msg, b"The object resists the spell") || prefix(msg, b"There is no clear floor") {
        // Forget the newly created-though-failed glyph.  Nothing was
        // tracked if the list is already empty, and clearing the
        // actually-tracked entry keeps the write inside the array.
        let t = &mut st.glyph.track;
        if t.num > 0 {
            t.num -= 1;
            t.x[t.num as usize] = 0;
            t.y[t.num as usize] = 0;
        }

        // note it
        borg_note(st, "# Removing the Glyph under me, placing with broken door.");

        // mark that we are not on a clear spot.  The borg ignores
        // broken doors and this will keep him from casting it again.
        set_ag_feat(st, FEAT_BROKEN);
        return;
    }

    // Removed rubble.  Important when out of lite
    if prefix(msg, b"You have removed the ") {
        // remove rubbles from array: the loop runs over 2x2 grids but
        // only ever looks at the goal grid
        let c = st.borg.c;
        for _y in c.y - 1..c.y + 1 {
            for _x in c.x - 1..c.x + 1 {
                // replace all rubble with broken doors, the borg ignores
                // broken doors.  This routine is only needed if the borg
                // is out of lite and searching in the dark.
                if st.borg.tr(BI_LIGHT) != 0 {
                    continue;
                }

                if ag_feat(st) == FEAT_RUBBLE as i32 {
                    set_ag_feat(st, FEAT_BROKEN);
                }
            }
        }
        return;
    }

    if prefix(msg, b"The enchantment failed") {
        // reset our anti-bounce count for this
        st.borg.antibounce_count = 1;
        return;
    }

    // need to kill monsters when WoD is used
    if prefix(msg, b"There is a searing blast of light!") {
        // Examine all the monsters (within 12 grids)
        delete_kills_near(st, true, 12);

        // Remove the region fear as well
        let c = st.borg.c;
        st.danger.fear_region[(c.y / 11) as usize][(c.x / 11) as usize] = 0;

        return;
    }

    // Be aware and concerned of busted doors
    if prefix(msg, b"You hear a door burst open!") {
        // on level 1 and 2 be concerned.  Could be Grip or Fang
        if st.borg.tr(BI_CDEPTH) <= 3 && st.borg.tr(BI_CLEVEL) <= 5 {
            st.borg.mon.scary = true;
        }
    }

    // Some spells move the borg from his grid
    if prefix(msg, b"commands you to return.")
        || prefix(msg, b"teleports you away.")
        || prefix(msg, b"gestures at your feet.")
    {
        // If in Lunal mode better shut that off, he is not on the stairs
        // anymore
        st.borg.lunal_mode = false;
        borg_note(st, "# Disconnecting Lunal Mode due to monster spell.");
    }

    // Feelings about the level
    for (i, text) in PREFIX_FEELING_DANGER.iter().enumerate() {
        // "You feel..." (etc)
        if prefix(msg, text) {
            borg_react(st, msg, format!("FEELING_DANGER:{}", i).as_bytes());
            return;
        }
    }

    for (i, text) in SUFFIX_FEELING_STUFF.iter().enumerate() {
        // "You feel..." (etc)
        if suffix(msg, text) {
            borg_react(st, msg, format!("FEELING_STUFF:{}", i).as_bytes());
            return;
        }
    }
}

/// The monster messages of `borg_parse_aux` (pain, kills, deaths, blinks,
/// misses, hits, spells, states), for messages that do not start with
/// "Your ". Returns true where C returns.
fn parse_monster_message(st: &mut BorgState, msg: &[u8], len: i32) -> bool {
    // "who" before a suffix of `tmp` bytes: `strnfmt(who, 1 + len - tmp, "%s", msg)`
    let who_before = |tmp: usize| who_text(size_of_int(1 + len - tmp as i32), msg);
    // "who" after a prefix of `tmp` bytes:
    // `strnfmt(who, 1 + len - (tmp + 1), "%s", msg + tmp)`
    let who_after = |tmp: usize| who_text(size_of_int(1 + len - (tmp as i32 + 1)), &msg[tmp..]);

    // "It screams in pain." (etc)
    let pain = st.messages.suffix_pain.as_ref().expect("pain messages not initialised");
    if let Some(p) = pain.iter().find(|p| suffix(msg, p)) {
        let who = who_before(p.len());
        borg_react(st, msg, &tagged(b"PAIN:", &who));
        return true;
    }

    // "You have killed it." (etc)
    for p in PREFIX_KILL {
        if prefix(msg, p) {
            let who = who_after(p.len());
            borg_react(st, msg, &tagged(b"KILL:", &who));
            return true;
        }
    }

    // "It dies." (etc)
    for s in SUFFIX_DIED {
        if suffix(msg, s) {
            let who = who_before(s.len());
            borg_react(st, msg, &tagged(b"DIED:", &who));
            return true;
        }
    }

    // "It blinks or telports." (etc)
    for s in SUFFIX_BLINK {
        if suffix(msg, s) {
            let who = who_before(s.len());
            borg_react(st, msg, &tagged(b"BLINK:", &who));
            return true;
        }
    }

    // "It misses you."
    if suffix(msg, b" misses you.") {
        let who = who_before(b" misses you.".len());
        borg_react(st, msg, &tagged(b"MISS_BY:", &who));
        return true;
    }

    // "It is repelled.."
    // treat as a miss
    if suffix(msg, b" is repelled.") {
        let who = who_before(b" is repelled.".len());
        borg_react(st, msg, &tagged(b"MISS_BY:", &who));
        return true;
    }

    // "It hits you." (etc)
    let hit = patterns(&st.messages.hit_by)
        .find(|m| borg_message_contains(msg, m))
        .and_then(|m| strstr(msg, m.p1.as_deref().expect("p1")));
    if let Some(start) = hit {
        // `strnfmt(who, start - msg, "%s", msg)`
        let who = who_text(start, msg);
        borg_react(st, msg, &tagged(b"HIT_BY:", &who));

        // If I was hit, then I am not on a glyph
        if st.glyph.track.num != 0 {
            // erase them all and allow the borg to scan the screen and
            // rebuild the array. He won't see the one under him though.
            // So a special check must be made.
            rebuild_glyphs(st);
        }
        return true;
    }

    // get rid of the messages that aren't for invisible spells (C tests
    // this inside the loop, before the first entry)
    if patterns(&st.messages.spell_invis).next().is_some()
        && (prefix(msg, b"Something ") || prefix(msg, b"You "))
    {
        let found = patterns(&st.messages.spell_invis)
            .find(|m| borg_message_contains(msg, m))
            .map(|m| m.index);
        if let Some(index) = found {
            borg_react(st, msg, &tagged(format!("SPELL_{:03}:", index).as_bytes(), b"Something"));
            return true;
        }
    }
    let spell = patterns(&st.messages.spell)
        .find(|m| borg_message_contains(msg, m))
        .map(|m| (m.index, strstr(msg, m.p1.as_deref().expect("p1"))));
    if let Some((index, Some(start))) = spell {
        // `strnfmt(who, start - msg, "%s", msg)`
        let who = who_text(start, msg);
        borg_react(st, msg, &tagged(format!("SPELL_{:03}:", index).as_bytes(), &who));
        return true;
    }

    // States: (message ending, tag)
    const STATES: [(&[u8], &[u8]); 8] = [
        // State -- Asleep
        (b" falls asleep!", b"STATE_SLEEP:"),
        // State -- confused
        (b" looks confused.", b"STATE_CONFUSED:"),
        // State -- confused
        (b" looks more confused.", b"STATE_CONFUSED:"),
        // State -- Not Asleep
        (b" wakes up.", b"STATE_AWAKE:"),
        // State -- Afraid
        (b" flees in terror!", b"STATE__FEAR:"),
        // State -- Not Afraid
        (b" recovers his courage.", b"STATE__BOLD:"),
        (b" recovers her courage.", b"STATE__BOLD:"),
        (b" recovers its courage.", b"STATE__BOLD:"),
    ];
    for (s, tag) in STATES {
        if suffix(msg, s) {
            let who = who_before(s.len());
            borg_react(st, msg, &tagged(tag, &who));
            return true;
        }
    }

    false
}

/// Parse a message, piece of a message, or set of messages
/// (`borg_parse`). `None` (C: NULL) flushes the collected message.
///
/// We must handle long messages which are "split" into multiple
/// pieces, and also multiple messages which may be "combined"
/// into a single set of messages.
pub fn borg_parse(st: &mut BorgState, msg: Option<&[u8]>) {
    // Note the long message
    if st.cfg(BORG_VERBOSE) != 0 {
        if let Some(m) = msg {
            let mut note = b"# Parsing msg <".to_vec();
            note.extend_from_slice(fmt_arg(m));
            note.push(b'>');
            borg_note_bytes(st, &note);
        }
    }

    // `msg[0]` (NUL for an empty text)
    let first = msg.map(|m| m.first().copied().unwrap_or(0));

    // Flush messages
    if st.messages.parse_len != 0 && first != Some(b' ') {
        let len = st.messages.parse_len;
        let mut j: i32 = 0;

        // Split out punctuation
        let mut i: i32 = 0;
        while i < len - 1 {
            let c = st.messages.parse_buf[i as usize];
            // Check for punctuation; require space
            if (c == b'.' || c == b'!' || c == b'?' || c == b'"')
                && st.messages.parse_buf[(i + 1) as usize] == b' '
            {
                // Parse fragment (C terminates it at i + 1 and restores
                // the space afterwards)
                let frag = st.messages.parse_buf[j as usize..(i + 1) as usize].to_vec();
                borg_parse_aux(st, &frag, (i + 1) - j);

                // Advance past spaces
                j = i + 2;
                while st.messages.parse_buf[j as usize] == b' ' {
                    j += 1;
                }
            }
            i += 1;
        }

        // Parse tail
        let tail = cstr(&st.messages.parse_buf[j as usize..]).to_vec();
        borg_parse_aux(st, &tail, len - j);

        // Forget
        st.messages.parse_len = 0;
    }

    let m = &mut st.messages;
    match msg {
        // No message: start over
        None => m.parse_len = 0,
        // Continued message: collect, verify, and grow
        Some(text) if first == Some(b' ') => {
            let len = m.parse_len as usize;
            let n = strnfmt_s(&mut m.parse_buf[len..], PARSE_BUF_SIZE - len, text);
            m.parse_len = (len + n) as i32;
        }
        // New message: collect, verify, and grow
        Some(text) => m.parse_len = strnfmt_s(&mut m.parse_buf, PARSE_BUF_SIZE, text) as i32,
    }
}

// ---------------------------------------------------------------------------
// The message tables

/// All parts equal or the same nullness (`borg_read_message_equal`).
fn borg_read_message_equal(a: &ReadMessage, b: &ReadMessage) -> bool {
    a.p1 == b.p1 && a.p2 == b.p2 && a.p3 == b.p3
}

/// Add a pattern to a table unless an equal one is there (`insert_msg`);
/// `None` adds the NULL end.
fn insert_msg(msgs: &mut Option<Vec<ReadMessage>>, msg: Option<ReadMessage>) {
    // this way we don't have to pre-create the array
    let v = msgs.get_or_insert_with(Vec::new);

    let Some(msg) = msg else {
        // NULL end (the C array is shrunk to fit here)
        v.push(ReadMessage::default());
        return;
    };

    if !v.iter().any(|m| borg_read_message_equal(m, &msg)) {
        v.push(msg);
    }
}

/// `borg_trim_lead_space`: drop one leading space.
fn borg_trim_lead_space(s: &[u8]) -> &[u8] {
    s.strip_prefix(b" ").unwrap_or(s)
}

/// `format("%.*s", n, s)`: at most `n` bytes (and at most 1023).
fn first_bytes(s: &[u8], n: usize) -> &[u8] {
    let s = fmt_arg(s);
    &s[..n.min(s.len())]
}

/// Skip leading spaces (`while (suffix[0] == ' ') suffix++;`).
fn skip_spaces(s: &[u8]) -> &[u8] {
    let n = s.iter().take_while(|&&c| c == b' ').count();
    &s[n..]
}

/// The offset of the first `c` in `s` (`strchr`).
fn strchr(s: &[u8], c: u8) -> Option<usize> {
    s.iter().position(|&x| x == c)
}

/// Break a string into a pattern (`borg_load_read_message`).
///
/// a message can have up to three parts broken up by variables
/// ex: "{name} hits {pronoun} followers with {type} ax."
/// " hits ", " followers with ", " ax."
/// if the message has more parts than that, they are ignored so
/// ex: "{name} hits {pronoun} followers with {type} ax and {type} breath."
/// would end up as
/// " hits ", " followers with ", " ax and "
/// hopefully this is enough to keep the messages as unique as possible
fn borg_load_read_message(message: &[u8], index: i32) -> ReadMessage {
    let mut r = ReadMessage { index, ..ReadMessage::default() };

    let Some(close) = strchr(message, b'}') else {
        // no variables, use message as is
        r.p1 = Some(borg_trim_lead_space(message).to_vec());
        return r;
    };
    // skip leading variable, if there is one
    let mut suffix: &[u8] = if message[0] == b'{' { &message[close + 1..] } else { message };
    let Some(var) = strchr(suffix, b'{') else {
        // one variable, use message post variable
        r.p1 = Some(borg_trim_lead_space(suffix).to_vec());
        return r;
    };
    // `var` as a slice (`strchr` result)
    let mut var: &[u8] = &suffix[var..];
    suffix = skip_spaces(suffix);
    let part_len = suffix.len() - var.len();
    r.p1 = Some(first_bytes(suffix, part_len).to_vec());
    let Some(close) = strchr(var, b'}') else {
        return r; // this should never happen but ... if a string is { with no }
    };
    suffix = &var[close + 1..];
    let Some(v) = strchr(suffix, b'{') else {
        suffix = skip_spaces(suffix);

        // two variables, ignore if last part is just .
        if !suffix.is_empty() && !streq(suffix, b".") {
            r.p2 = Some(suffix.to_vec());
        }
        return r;
    };
    var = &suffix[v..];
    suffix = skip_spaces(suffix);
    let part_len = suffix.len() - var.len();
    if part_len != 0 {
        r.p2 = Some(borg_trim_lead_space(first_bytes(suffix, part_len)).to_vec());
    }
    let Some(close) = strchr(var, b'}') else {
        return r; // this should never happen but ... if a string is { with no }
    };
    suffix = &var[close + 1..];
    let Some(v) = strchr(suffix, b'{') else {
        suffix = skip_spaces(suffix);
        // three variables, ignore if last part is just .
        if !suffix.is_empty() && !streq(suffix, b".") {
            let part = Some(borg_trim_lead_space(suffix).to_vec());
            if r.p2.is_some() {
                r.p3 = part;
            } else {
                r.p2 = part;
            }
        }
        return r;
    };
    var = &suffix[v..];
    suffix = skip_spaces(suffix);
    let part_len = suffix.len() - var.len();
    let part = Some(borg_trim_lead_space(first_bytes(suffix, part_len)).to_vec());
    if r.p2.is_some() {
        r.p3 = part;
    } else {
        r.p2 = part;
    }

    r
}

/// Load the monster spell messages (`borg_init_spell_messages`).
fn borg_init_spell_messages(st: &mut BorgState) {
    for (index, levels) in engine::monster_spell_texts() {
        let index = i32::from(index);
        for level in levels {
            if let Some(m) = &level.blind_message {
                insert_msg(&mut st.messages.spell_invis, Some(borg_load_read_message(m, index)));
            }
            if let Some(m) = &level.message {
                insert_msg(&mut st.messages.spell, Some(borg_load_read_message(m, index)));
            }
            if let Some(m) = &level.miss_message {
                insert_msg(&mut st.messages.spell, Some(borg_load_read_message(m, index)));
            }
        }
    }
    // null terminate
    insert_msg(&mut st.messages.spell_invis, None);
    insert_msg(&mut st.messages.spell, None);
}

/// A pain message with its `[singular|plural]` parts resolved, after a
/// leading space (`borg_get_parsed_pain`; HACK pluralize code stolen from
/// mon-msg.c).
///
/// As in C, the buffer has room for the leading space, the text and the
/// terminator (`strlen(pain) + 2`), so the last character is kept
/// (better-borg-c milestone7 patch 0071).
fn borg_get_parsed_pain(pain: &[u8], do_plural: bool) -> Vec<u8> {
    #[derive(PartialEq)]
    enum Parse {
        Normal,
        Single,
        Plural,
    }
    // Room for the leading space, the text and the terminator
    let buflen = pain.len() + 2;
    let mut buf = vec![b' '];
    let mut state = Parse::Normal;

    // for the borg, always start with a space
    for &cur in pain {
        if buf.len() >= buflen - 1 {
            break;
        }
        // The characters '[|]' switch parsing mode and are never output.
        // The syntax is [singular|plural]
        if state == Parse::Normal && cur == b'[' {
            state = Parse::Single;
        } else if state == Parse::Single && cur == b'|' {
            state = Parse::Plural;
        } else if state != Parse::Normal && cur == b']' {
            state = Parse::Normal;
        } else if state == Parse::Normal
            || (state == Parse::Single && !do_plural)
            || (state == Parse::Plural && do_plural)
        {
            // Copy the characters according to the mode
            buf.push(cur);
        }
    }
    buf
}

/// Add the singular and plural forms of a pain message
/// (`borg_insert_pain`).
fn borg_insert_pain(list: &mut Vec<Vec<u8>>, pain: &[u8]) {
    list.push(borg_get_parsed_pain(pain, false));
    list.push(borg_get_parsed_pain(pain, true));
}

/// Load the monster pain messages (`borg_init_pain_messages`).
fn borg_init_pain_messages(st: &mut BorgState) {
    let mut list = Vec::new();

    for idx in 0..usize::from(engine::z_info().mp_max) {
        for m in engine::pain_texts(idx) {
            let Some(m) = m else { break };
            borg_insert_pain(&mut list, &m);
        }
    }

    // some more standard messages
    for (idx, &(std_pain, kind)) in MON_MSG_REPOSITORY.iter().enumerate().take(MON_MSG_MAX as usize) {
        if kind == MSG_KILL {
            continue;
        }

        let idx = idx as u32;
        if [
            MON_MSG_DISAPPEAR,
            MON_MSG_95,
            MON_MSG_75,
            MON_MSG_50,
            MON_MSG_35,
            MON_MSG_20,
            MON_MSG_10,
            MON_MSG_0,
        ]
        .contains(&idx)
        {
            continue;
        }
        borg_insert_pain(&mut list, std_pain);
    }

    st.messages.suffix_pain = Some(list);
}

/// Load the "player hit by" messages (`borg_init_hit_by_messages`).
fn borg_init_hit_by_messages(st: &mut BorgState) {
    for (msgt, messages) in engine::blow_method_texts() {
        for m in messages.iter().flatten() {
            insert_msg(&mut st.messages.hit_by, Some(borg_load_read_message(m, msgt)));
        }
    }
    // null terminate
    insert_msg(&mut st.messages.hit_by, None);
}

/// Init all messages used by the borg (`borg_init_messages`).
pub fn borg_init_messages(st: &mut BorgState) {
    borg_free_messages(st);

    borg_init_spell_messages(st);
    borg_init_pain_messages(st);
    borg_init_hit_by_messages(st);

    // Message tracking
    let m = &mut st.msgs;

    // No chars saved yet
    m.len = 0;

    // Maximum buffer size
    m.siz = 4096;

    // Allocate a buffer
    m.buf = vec![0; m.siz as usize];

    // No msg's saved yet
    m.num = 0;

    // Maximum number of messages
    m.max = 256;

    // Allocate array of positions
    m.pos = vec![0; m.max as usize];

    // Allocate array of use-types
    m.use_ = vec![0; m.max as usize];
}

/// Free all messages used by the borg (`borg_free_messages`).
/// `borg_msg_len` and `borg_msg_max` are left as they are, as in C.
pub fn borg_free_messages(st: &mut BorgState) {
    let m = &mut st.msgs;
    m.buf = Vec::new();
    m.use_ = Vec::new();
    m.pos = Vec::new();
    m.num = 0;
    m.siz = 0;

    let s = &mut st.messages;
    s.suffix_pain = None;
    s.hit_by = None;
    s.spell_invis = None;
    s.spell = None;
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `MON_MSG_REPOSITORY` is list-mon-message.h.
    #[test]
    fn repository_matches_header() {
        let h = include_str!("../../engine/angband/src/list-mon-message.h");
        let rows: Vec<&str> = h.lines().filter(|l| l.starts_with("MON_MSG(")).collect();
        assert_eq!(rows.len(), MON_MSG_REPOSITORY.len());
        for (row, (msg, kind)) in rows.iter().zip(MON_MSG_REPOSITORY.iter()) {
            let q1 = row.find('"').unwrap();
            let q2 = row.rfind('"').unwrap();
            assert_eq!(&&row.as_bytes()[q1 + 1..q2], msg, "{row}");
            let k = match *kind {
                MSG_GENERIC => "MSG_GENERIC",
                MSG_KILL => "MSG_KILL",
                MSG_FLEE => "MSG_FLEE",
                _ => unreachable!(),
            };
            assert!(row.contains(k), "{row}");
        }
        assert_eq!(MON_MSG_MAX as usize, MON_MSG_REPOSITORY.len() - 1);
    }

    #[test]
    fn parsed_pain() {
        assert_eq!(borg_get_parsed_pain(b"[is|are] hurt.", false), b" is hurt.");
        assert_eq!(borg_get_parsed_pain(b"[is|are] hurt.", true), b" are hurt.");
        // "[s]" is the singular part ("It dies.", "They die.")
        assert_eq!(borg_get_parsed_pain(b"die[s].", false), b" dies.");
        assert_eq!(borg_get_parsed_pain(b"die[s].", true), b" die.");
        // no brackets: the whole text is kept (better-borg-c milestone7
        // patch 0071; before it the last character was cut)
        assert_eq!(borg_get_parsed_pain(b"can move again.", false), b" can move again.");
        assert_eq!(
            borg_get_parsed_pain(b"You hear [a|several] scream[|s] of agony!", true),
            b" You hear several screams of agony!"
        );
    }

    fn rm(p1: &[u8], p2: Option<&[u8]>, p3: Option<&[u8]>) -> ReadMessage {
        ReadMessage {
            p1: Some(p1.to_vec()),
            p2: p2.map(|s| s.to_vec()),
            p3: p3.map(|s| s.to_vec()),
            index: 0,
        }
    }

    #[test]
    fn read_message_parts() {
        let f = |s: &[u8]| borg_load_read_message(s, 0);
        assert_eq!(f(b"You hear a scream."), rm(b"You hear a scream.", None, None));
        assert_eq!(f(b" lead space"), rm(b"lead space", None, None));
        assert_eq!(f(b"{name} hits you."), rm(b"hits you.", None, None));
        assert_eq!(f(b"{name} casts a spell at {target}."), rm(b"casts a spell at ", None, None));
        assert_eq!(
            f(b"{name} hits {pronoun} followers with {type} ax."),
            rm(b"hits ", Some(b"followers with "), Some(b"ax."))
        );
        assert_eq!(
            f(b"{name} hits {pronoun} followers with {type} ax and {type} breath."),
            rm(b"hits ", Some(b"followers with "), Some(b"ax and "))
        );
        // variable not at the start
        assert_eq!(f(b"Something {kind} mumbles."), rm(b"Something ", Some(b"mumbles."), None));
        // two variables in a row: empty first part
        assert_eq!(f(b"{name} {verb} at you."), rm(b"", Some(b"at you."), None));
        assert_eq!(f(b"{name} {verb} {x}."), rm(b"", None, None));
        assert_eq!(f(b"{name} {verb} {x} {y}."), rm(b"", Some(b""), None));
        // unclosed variable
        assert_eq!(f(b"{name} hits {oops"), rm(b"hits ", None, None));
    }

    #[test]
    fn strnfmt_like_c() {
        let mut b = [0xffu8; 8];
        assert_eq!(strnfmt_s(&mut b, 4, b"abcdef"), 3);
        assert_eq!(&b[..4], b"abc\0");
        assert_eq!(strnfmt_s(&mut b, 1, b"abc"), 0);
        assert_eq!(b[0], 0);
        // max 0: no limit
        assert_eq!(strnfmt_s(&mut b, 0, b"abc"), 3);
        assert_eq!(&b[..4], b"abc\0");
        assert_eq!(tagged(b"HIT:", &[b'x'; 300]).len(), 255);
    }
}
