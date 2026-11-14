//! Port of `borg-magic-play.c`: cast spells just to learn the spell.
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

use crate::borg_io::{borg_keypress, borg_note, borg_note_bytes};
use crate::borg_magic::{borg_spell, BorgMagic, BORG_MAGIC_OKAY, BORG_MAGIC_TEST};
use crate::engine;
use crate::borg_headers::borg_magic_h::{
    BLESS, COMMAND, CURSE, DECOY, FORCEFUL_BLOW, GLYPH_OF_WARDING, GRONDS_BLOW, LIGHTNING_STRIKE,
    MAGIC_MISSILE, MAIM_FOE, REMOVE_CURSE, SINGLE_COMBAT, TAP_MAGICAL_ENERGY, TAP_UNLIFE, TREMOR,
    VAMPIRE_STRIKE, WORD_OF_DESTRUCTION,
};
use crate::borg_headers::borg_trait_h::{BI_CDEPTH, BI_ISBLIND, BI_ISCONFUSED, BI_ISSTUDY, BI_LIGHT};
use crate::headers::list_effects_h::{
    EF_ALTER, EF_ARC, EF_BALL, EF_BANISH, EF_BEAM, EF_BIZARRE, EF_BOLT, EF_BOLT_AWARE,
    EF_BOLT_OR_BEAM, EF_BOLT_STATUS, EF_BOLT_STATUS_DAM, EF_BRAND_AMMO, EF_BRAND_BOLTS,
    EF_BREATH, EF_COMMAND, EF_CREATE_ARROWS, EF_CURSE, EF_ENCHANT, EF_IDENTIFY, EF_LASH, EF_LINE,
    EF_MELEE_BLOWS, EF_MOVE_ATTACK, EF_RANDOM, EF_RECHARGE, EF_SHORT_BEAM, EF_SINGLE_COMBAT,
    EF_STRIKE, EF_SWARM, EF_TELEPORT_TO, EF_WONDER,
};
use crate::headers::list_player_flags_h::PF_CHOOSE_SPELLS;
use crate::headers::ui_event_h::keycode_t;
use crate::state::BorgState;

/// `BORG_DARK` (borg-cave.h): the grid is known to be dark
const BORG_DARK: u16 = 0x04;

/// `effects[index].aim`: the `aim` column of `list-effects.h`, which
/// borg-magic-play.c copies into its own `effects[]` table (indexed by
/// effect index; `EF_NONE` and `EF_MAX` have `aim == false`).
fn effect_aim(index: u16) -> bool {
    matches!(
        u32::from(index),
        EF_BALL
            | EF_BREATH
            | EF_ARC
            | EF_SHORT_BEAM
            | EF_LASH
            | EF_SWARM
            | EF_STRIKE
            | EF_BOLT
            | EF_BEAM
            | EF_BOLT_OR_BEAM
            | EF_LINE
            | EF_ALTER
            | EF_BOLT_STATUS
            | EF_BOLT_STATUS_DAM
            | EF_BOLT_AWARE
            | EF_CURSE
            | EF_COMMAND
            | EF_MOVE_ATTACK
            | EF_SINGLE_COMBAT
            | EF_MELEE_BLOWS
            | EF_BIZARRE
            | EF_WONDER
    )
}

/// `all_letters_nohjkl[i]` as a key (a C `char`, sign-extended)
fn menu_letter(i: i32) -> keycode_t {
    engine::all_letters_nohjkl(i) as i8 as keycode_t
}

/// Check if a spell can be cast just to test it (`borg_can_play_spell`)
fn borg_can_play_spell(as_: &BorgMagic) -> bool {
    // There are some spells not worth testing
    // they can just be used when the borg is ready to use it.
    match as_.spell_enum {
        LIGHTNING_STRIKE | TAP_UNLIFE | TAP_MAGICAL_ENERGY | REMOVE_CURSE | TREMOR
        | WORD_OF_DESTRUCTION | GRONDS_BLOW | DECOY | GLYPH_OF_WARDING | SINGLE_COMBAT
        | VAMPIRE_STRIKE | COMMAND | CURSE | FORCEFUL_BLOW | MAIM_FOE => return false,
        _ => {}
    }

    let effect = u32::from(as_.effect_index);
    if effect == EF_BRAND_BOLTS {
        return false; // !FIX !TODO check for a bolt
    }
    if effect == EF_CREATE_ARROWS {
        return false; // !FIX !TODO check for a staff
    }
    if effect == EF_BRAND_AMMO {
        return false; // !FIX !TODO check for ammo
    }
    if effect == EF_ENCHANT {
        return false; // !FIX !TODO check something to enchant
    }
    if effect == EF_IDENTIFY {
        return false; // !FIX !TODO check something to identify
    }
    if effect == EF_RECHARGE {
        return false; // !FIX !TODO check for a wand or rod or staff
    }
    true
}

/// Study and/or Test spells/prayers (`borg_play_magic`)
pub fn borg_play_magic(st: &mut BorgState, bored: bool) -> bool {
    let mut b_r: i32 = -1;
    let mut b_spell_num: i32 = -1;

    // Must use magic or prayers
    // !FIX !TODO add to borg struct and check that
    let total_spells = engine::class_total_spells();
    if total_spells == 0 {
        return false;
    }

    // Blind/confused
    if st.borg.tr(BI_ISBLIND) != 0 || st.borg.tr(BI_ISCONFUSED) != 0 {
        return false;
    }

    // Dark
    if st.borg.tr(BI_LIGHT) == 0 {
        return false;
    }
    // C bug 9: `==`, not `&` (kept)
    if st.cave.grid(st.borg.c.x, st.borg.c.y).info == BORG_DARK {
        return false;
    }

    // loop through spells backward
    for spell_num in (0..total_spells).rev() {
        let as_ = &st.magic.magics[spell_num as usize];

        // Look for the book in inventory
        if st.borg.book_idx[as_.book as usize] < 0 {
            continue;
        }

        // Require "learnable" status
        if as_.status != BORG_MAGIC_OKAY {
            continue;
        }

        // Obtain "rating"
        let r = i32::from(as_.rating);

        // Skip "boring" spells/prayers
        if !bored && r <= 50 {
            continue;
        }

        // Skip "icky" spells/prayers
        if r <= 0 {
            continue;
        }

        // Skip "worse" spells/prayers
        if r <= b_r {
            continue;
        }

        // Track it
        b_r = r;
        b_spell_num = spell_num;
    }

    // Study
    if st.borg.tr(BI_ISSTUDY) != 0 && b_r > 0 && b_spell_num >= 0 {
        let as_ = st.magic.magics[b_spell_num as usize];

        // Debugging Info
        let mut note = b"# Studying spell/prayer ".to_vec();
        note.extend_from_slice(as_.name.to_bytes());
        note.push(b'.');
        borg_note_bytes(st, &note);

        // Learn the spell
        borg_keypress(st, keycode_t::from(b'G'));

        // Specify the book
        let book = i32::from(st.borg.book_idx[as_.book as usize]);
        borg_keypress(st, menu_letter(book));

        // Specify the spell
        if engine::player_has(PF_CHOOSE_SPELLS) {
            // Specify the spell
            borg_keypress(st, menu_letter(as_.book_offset));
        }

        // Success
        return true;
    }

    // Only in town
    if st.borg.tr(BI_CDEPTH) != 0 && !st.borg.munchkin_mode {
        return false;
    }

    // Only when bored
    if !bored {
        return false;
    }

    // Check each spell (backwards)
    for spell_num in (0..total_spells).rev() {
        let as_ = st.magic.magics[spell_num as usize];

        // No such book
        if st.borg.book_idx[as_.book as usize] < 0 {
            continue;
        }

        // Only try "untried" spells/prayers
        if as_.status != BORG_MAGIC_TEST {
            continue;
        }

        // some spells can't be "played with" in town
        if !borg_can_play_spell(&as_) {
            continue;
        }

        // Some spells should not be tested in munchkin mode
        if st.borg.munchkin_mode {
            // Mage types
            if engine::player_has(PF_CHOOSE_SPELLS) && as_.spell_enum == MAGIC_MISSILE {
                continue;
            }

            // Priest type
            if !engine::player_has(PF_CHOOSE_SPELLS) && as_.spell_enum == BLESS {
                continue;
            }
        }

        // Note
        borg_note(st, "# Testing untried spell/prayer");

        // Use spell or prayer
        if borg_spell(st, as_.spell_enum) {
            // Allow attack spells
            // MEGAHACK -- assume "Random" is shooting.
            let effect = u32::from(as_.effect_index);
            if effect_aim(as_.effect_index) || effect == EF_RANDOM || effect == EF_TELEPORT_TO {
                // Target self
                borg_keypress(st, keycode_t::from(b'*'));
                borg_keypress(st, keycode_t::from(b't'));
            }

            // Allow spells that require selection of a monster type
            if effect == EF_BANISH {
                // Target Maggot
                borg_keypress(st, keycode_t::from(b'h'));
            }

            // Success
            return true;
        }
    }

    // Nope
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::headers::effects_h::{EF_MAX, EF_NONE};

    #[test]
    fn aim_table_matches_list_effects() {
        // 22 effects have `aim == true` in list-effects.h
        let n = (EF_NONE..=EF_MAX).filter(|&i| effect_aim(i as u16)).count();
        assert_eq!(n, 22);
        assert!(!effect_aim(EF_RANDOM as u16));
        assert!(!effect_aim(EF_TELEPORT_TO as u16));
        assert!(effect_aim(EF_BOLT as u16));
    }
}
