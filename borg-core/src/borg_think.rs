//! Port of `borg-think.c`: the entry point for the Borg thinking about
//! what to do.
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
//! `borg_think` runs once per Borg decision: it refreshes what the Borg
//! knows (items, spells, equipment, the screen, its power) and hands over
//! to the store or the dungeon thinker. The globals `shop_num` and
//! `borg_do_spell` and the statics of `borg_check_save` live in
//! `ThinkState`. C bugs kept are listed in `docs/c-bugs.md` (section 51).
use crate::borg::BorgTime;
use crate::borg_io::{borg_flush, borg_keypress, borg_note, borg_warning, borg_what_text, ESCAPE};
use crate::cctype::{c_isalpha, c_iscntrl, c_isdigit};
use crate::engine;
use crate::borg_headers::borg_h::BORG_DUMP_LEVEL;
use crate::borg_headers::borg_trait_h::{BI_CDEPTH, BI_CLEVEL, BI_MAXCLEVEL, BI_MAXDEPTH};
use crate::state::BorgState;

/// `BORG_TIME_MAX` (borg.h: `INT32_MAX`)
const BORG_TIME_MAX: BorgTime = i32::MAX;

/// Size of the game's `savefile` buffer and of `svSavefile`.
pub const SAVEFILE_LEN: usize = 1024;

/// The globals of borg-think.c.
pub struct ThinkState {
    /// `shop_num`: the current shop index, or -1.
    pub shop_num: i16,
    /// `borg_do_spell`: acquire the spell info at the next `borg_think`.
    pub do_spell: bool,
    /// `svSavefile` (static in `borg_check_save`): the game's `savefile`
    /// kept while the scum file is written (`SAVEFILE_LEN` bytes).
    pub sv_savefile: Vec<u8>,
    /// `justSaved` (static in `borg_check_save`)
    pub just_saved: bool,
}

impl Default for ThinkState {
    fn default() -> Self {
        ThinkState {
            shop_num: -1,
            do_spell: true,
            sv_savefile: vec![0; SAVEFILE_LEN],
            just_saved: false,
        }
    }
}

/// Abort the Borg, noting the reason (`borg_oops`).
pub fn borg_oops(st: &mut BorgState, what: &str) {
    /* Stop processing */
    st.borg.status.active = false;

    /* Give a warning */
    borg_note(st, &format!("# Aborting ({what})."));

    /* Forget borg keys */
    borg_flush(st);
}

/// Attempt to save the game (`borg_save_game`). Always true.
fn borg_save_game(st: &mut BorgState) -> bool {
    /* Cancel everything */
    borg_keypress(st, ESCAPE);
    borg_keypress(st, ESCAPE);

    /* Save the game */
    borg_keypress(st, '^' as u32);
    borg_keypress(st, 'S' as u32);

    /* Cancel everything */
    borg_keypress(st, ESCAPE);
    borg_keypress(st, ESCAPE);

    /* Success */
    true
}

/// Check if the Borg should save the game and do so if necessary
/// (`borg_check_save`).
fn borg_check_save(st: &mut BorgState) -> bool {
    /* save now */
    if st.borg.status.save && borg_save_game(st) {
        /* Log */
        borg_note(st, "# Auto Save!");

        st.borg.status.save = false;

        /* Create a scum file */
        if st.borg.tr(BI_CLEVEL) >= st.cfg(BORG_DUMP_LEVEL)
            || engine::player_died_from().windows(10).any(|w| w == b"starvation")
        {
            st.think.sv_savefile.copy_from_slice(&engine::savefile());

            /* Process the player name (`svSavefile2`) */
            let mut name: Vec<u8> = Vec::new();
            for mut c in engine::player_full_name() {
                /* No control characters */
                if c_iscntrl(c) {
                    /* Illegal characters (`%02X` of the signed char) */
                    engine::quit(
                        format!(
                            "Illegal control char (0x{:02X}) in player name",
                            i32::from(c as i8)
                        )
                        .as_bytes(),
                    );
                }

                /* Convert all non-alphanumeric symbols */
                if !c_isalpha(c) && !c_isdigit(c) {
                    c = b'_';
                }

                /* Build "file_name" */
                name.push(c);
            }

            engine::savefile_path_build_archive(&name);

            st.think.just_saved = true;
        }
        return true;
    }
    if st.think.just_saved {
        let sv: &[u8; SAVEFILE_LEN] = st.think.sv_savefile[..].try_into().expect("svSavefile size");
        engine::set_savefile(sv);
        borg_save_game(st);
        st.think.just_saved = false;
        return true;
    }

    false
}

/// The bytes of a C string buffer up to its NUL.
fn cstr(buf: &[u8]) -> &[u8] {
    let n = buf.iter().position(|&b| b == 0).unwrap_or(buf.len());
    &buf[..n]
}

/// Think about the world and perform an action (`borg_think`).
///
/// Check inventory/equipment/spells/panel once per "turn"; process the
/// store and the dungeon when necessary.
pub fn borg_think(st: &mut BorgState) -> bool {
    let mut t_a: u8 = 0;
    let mut buf = [0u8; 128];

    /* find all items currently in use */
    crate::borg_item::borg_find_all_items(st);

    /* check to see if we should save */
    if borg_check_save(st) {
        return true;
    }

    /*** Process books/spells ***/
    if st.think.do_spell {
        crate::borg_magic::borg_cheat_spells(st);
        st.think.do_spell = false;
    }

    /* Always revert shapechanged players to normal form. */
    if engine::player_is_shapechanged() {
        /* it looks like throw is a good command that checks */
        /* your form without a prerequisite check */
        borg_keypress(st, 'v' as u32);
        borg_keypress(st, 'r' as u32);
        return true;
    }

    /* Examine the equipment/inventory */
    crate::borg_trait::borg_notice(st, true);

    /* Examine the screen */
    crate::borg_update::borg_update(st);

    /* Evaluate the current world */
    st.borg.power = crate::borg_power::borg_power(st);

    /*** Handle stores ***/

    /* Check for being in a store */
    if 0 == borg_what_text(1, 3, 4, &mut t_a, &mut buf) && matches!(cstr(&buf), b"Stor" | b"Home") {
        return crate::borg_think_store::borg_think_store(st);
    }

    /* Check spells again later */
    st.think.do_spell = true;

    /*** Analyze status ***/

    /* Track best level */
    if st.borg.tr(BI_CLEVEL) > st.borg.tr(BI_MAXCLEVEL) {
        st.borg.trait_[BI_MAXCLEVEL as usize] = st.borg.tr(BI_CLEVEL);
    }
    if st.borg.tr(BI_CDEPTH) > st.borg.tr(BI_MAXDEPTH) {
        st.borg.trait_[BI_MAXDEPTH as usize] = st.borg.tr(BI_CDEPTH);
    }

    /* Increment the clock */
    st.borg.time.now += 1;

    /* If the clock overflowed, fix that (C bug 50: never reached in play) */
    if st.borg.time.now > BORG_TIME_MAX - 100 {
        st.borg.time.now = 1;
        /* `msg` may wait for a key: it lends the state (engine::msg) */
        borg_warning(st, "*****WARNING***** Borg Clock Overflow");
    }

    /* Increment our anti-bounce count to avoid loops */
    st.borg.antibounce_count = st.borg.antibounce_count.wrapping_add(1);

    /* Allow user abort */
    if st.borg.status.cancel {
        return true;
    }

    /* Do something */
    crate::borg_think_dungeon::borg_think_dungeon(st)
}
