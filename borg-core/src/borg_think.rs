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
// borg-reborn-rs (#222): root trace frame and non-dungeon decisions.
use crate::core_trace::choice;
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
    pub town_action: Option<crate::borg_town_shadow::TownAction>,
    pub town_live: crate::borg_town_live::LiveState,
    pub phase_observation: crate::borg_town_dump::PhaseObservation,
}

impl Default for ThinkState {
    fn default() -> Self {
        ThinkState {
            shop_num: -1,
            do_spell: true,
            sv_savefile: vec![0; SAVEFILE_LEN],
            just_saved: false,
            town_action: None,
            town_live: crate::borg_town_live::LiveState::default(),
            phase_observation: crate::borg_town_dump::PhaseObservation::default(),
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
    // borg-reborn-rs (#222): includes early saves, stores and errand handoffs.
    let tracing = crate::core_trace::begin();
    let done = borg_think_inner(st);
    crate::core_trace::finish(st, tracing, done);
    done
}

fn borg_think_inner(st: &mut BorgState) -> bool {
    let mut t_a: u8 = 0;
    let mut buf = [0u8; 128];

    /* find all items currently in use */
    crate::borg_item::borg_find_all_items(st);

    /* The session object log: what appeared in the pack since the last
     * look. */
    crate::borg_object_log::log_acquisitions(st);

    /* Record the state, if the state file is on and the character is in
     * town. A human-in-the-loop session needs this: the human's choices
     * come from the game's messages, and the states they were made in come
     * from here. */
    crate::borg_town_dump::dump_town_state(st);

    /* check to see if we should save */
    if borg_check_save(st) {
        return choice!("borg_check_save");
    }

    /*** Process books/spells ***/
    if st.think.do_spell {
        crate::borg_magic::borg_cheat_spells(st);
        st.think.do_spell = false;
    }

    /* Always revert shapechanged players to normal form. */
    // borg-reborn-rs (#551): with escape_fox_form, a fleeing fox keeps its
    // shape; the revert's "(r)eturn" is then answered by the prompt
    // handler (borg_messages), so only the command is queued here.
    if !engine::player_is_shapechanged() {
        st.borg.fox_revert = false;
    }
    if engine::player_is_shapechanged() && !crate::borg_escape::keep_fox_form(st) {
        /* it looks like throw is a good command that checks */
        /* your form without a prerequisite check */
        if crate::borg_escape::any_fox_form_on() {
            crate::borg_escape::note_fox_revert(st);
            crate::borg_escape::queue_fox_revert(st);
        } else {
            borg_keypress(st, 'v' as u32);
            borg_keypress(st, 'r' as u32);
        }
        return choice!("revert_shapechange");
    }

    /* Examine the equipment/inventory */
    crate::borg_trait::borg_notice(st, true);

    /* Examine the screen */
    crate::borg_update::borg_update(st);
    crate::core_trace::observe_healing_potions(st);

    #[cfg(feature = "test-stair-handoff")]
    if let Ok(case) = std::env::var("BORG_TEST_STAIR_HANDOFF") {
        match case.as_str() {
            "healing-restock-dungeon" => crate::borg_healing_restock::fixture::dungeon(st, false),
            "healing-restock-short" => crate::borg_healing_restock::fixture::dungeon(st, true),
            "healing-restock-town" | "healing-restock-town-legacy-unknown" => {
                crate::borg_healing_restock::fixture::town(st, &case);
            }
            _ => crate::borg_flow_stairs::run_handoff_regression(st, &case),
        }
    }

    #[cfg(feature = "test-combat-heal")]
    if let Ok(case) = std::env::var("BORG_TEST_COMBAT_HEAL") {
        crate::borg_combat_heal::fixture::run(st, &case);
    }

    #[cfg(feature = "test-cursed-spares")]
    if let Ok(case) = std::env::var("BORG_TEST_CURSED_SPARES") {
        if case == "live-lantern" {
            crate::borg_junk::check_cursed_lantern_in_game(st);
            return choice!("test_cursed_lantern");
        }
        crate::borg_junk::check_cursed_spares(st);
    }

    // Remember learning opportunities only while actually in town. The
    // dungeon decision must not inspect freshly restocked hidden shop data.
    crate::borg_trip::observe_town(st);
    crate::borg_healing_restock::observe_town(st);
    crate::borg_ranged::observe(st);

    /* Evaluate the current world */
    st.borg.power = crate::borg_power::borg_power(st);

    /*** Handle stores ***/

    /* Check for being in a store */
    if 0 == borg_what_text(1, 3, 4, &mut t_a, &mut buf) && matches!(cstr(&buf), b"Stor" | b"Home") {
        let done = crate::borg_think_store::borg_think_store(st);
        crate::core_trace::selected("borg_think_store", concat!(file!(), ":", line!()));
        return done;
    }

    crate::borg_town_dump::record_phase_observation(st, None);

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
        return choice!("cancel");
    }

    // borg-reborn-rs (#208): an errand that is done stops the Borg here.
    if let Some(done) = crate::shell::before_dungeon_think(st) {
        crate::core_trace::selected("errand_handoff", concat!(file!(), ":", line!()));
        return done;
    }

    /* Do something */
    crate::borg_think_dungeon::borg_think_dungeon(st)
}
