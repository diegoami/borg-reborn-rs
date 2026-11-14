//! Port of `borg-reincarnate.c`: resurrect the borg with new stats and class.
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

use crate::borg_io::{borg_keypress, borg_keypresses, borg_note};
use crate::borg_headers::borg_h::{BORG_RESPAWN_CLASS, BORG_RESPAWN_RACE};
use crate::headers::ui_event_h::keycode_t;
use crate::state::BorgState;
use crate::engine;

/// `KC_ENTER` (ui-event.h)
const KC_ENTER: keycode_t = 0x9c;
/// `NOSCORE_BORG` (player.h)
const NOSCORE_BORG: u16 = 0x0020;
/// `PLAYER_NAME_LEN` (player.h): the name buffer, NUL included
const PLAYER_NAME_LEN: usize = 32;
/// `MAX_RACES` (borg-trait.h)
const MAX_RACES: u32 = 11;
/// `MAX_CLASSES` (borg-trait.h)
const MAX_CLASSES: u32 = 9;

// Race indexes (borg-trait.h, the order of p_race.txt)
const RACE_HUMAN: i32 = 0;
const RACE_HALF_ELF: i32 = 1;
const RACE_ELF: i32 = 2;
const RACE_HOBBIT: i32 = 3;
const RACE_GNOME: i32 = 4;
const RACE_DWARF: i32 = 5;
const RACE_HALF_ORC: i32 = 6;
const RACE_HALF_TROLL: i32 = 7;
const RACE_DUNADAN: i32 = 8;
const RACE_HIGH_ELF: i32 = 9;
const RACE_KOBOLD: i32 = 10;

// Name segments for random player names (same content and order as C,
// duplicates included: the RNG range is the table size)

// Dwarves
const DWARF_SYLLABLE1: &[&str] = &[
    "B", "D", "F", "G", "Gl", "H", "K", "L", "M", "N", "R", "S", "T", "Th", "V",
];

const DWARF_SYLLABLE2: &[&str] = &["a", "e", "i", "o", "oi", "u"];

const DWARF_SYLLABLE3: &[&str] = &[
    "bur", "fur", "gan", "gnus", "gnar", "li", "lin", "lir", "mli", "nar", "nus", "rin", "ran",
    "sin", "sil", "sur",
];

// Elves
const ELF_SYLLABLE1: &[&str] = &[
    "Al", "An", "Bal", "Bel", "Cal", "Cel", "El", "Elr", "Elv", "Eow", "Ear", "F", "Fal", "Fel",
    "Fin", "G", "Gal", "Gel", "Gl", "Is", "Lan", "Leg", "Lom", "N", "Nal", "Nel", "S", "Sal",
    "Sel", "T", "Tal", "Tel", "Thr", "Tin",
];

const ELF_SYLLABLE2: &[&str] = &[
    "a", "adrie", "ara", "e", "ebri", "ele", "ere", "i", "io", "ithra", "ilma", "il-Ga", "ili",
    "o", "orfi", "u", "y",
];

const ELF_SYLLABLE3: &[&str] = &[
    "l", "las", "lad", "ldor", "ldur", "linde", "lith", "mir", "n", "nd", "ndel", "ndil", "ndir",
    "nduil", "ng", "mbor", "r", "rith", "ril", "riand", "rion", "s", "thien", "viel", "wen", "wyn",
];

// Gnomes
const GNOME_SYLLABLE1: &[&str] = &[
    "Aar", "An", "Ar", "As", "C", "H", "Han", "Har", "Hel", "Iir", "J", "Jan", "Jar", "K", "L",
    "M", "Mar", "N", "Nik", "Os", "Ol", "P", "R", "S", "Sam", "San", "T", "Ter", "Tom", "Ul", "V",
    "W", "Y",
];

const GNOME_SYLLABLE2: &[&str] = &["a", "aa", "ai", "e", "ei", "i", "o", "uo", "u", "uu"];

const GNOME_SYLLABLE3: &[&str] = &[
    "ron", "re", "la", "ki", "kseli", "ksi", "ku", "ja", "ta", "na", "namari", "neli", "nika",
    "nikki", "nu", "nukka", "ka", "ko", "li", "kki", "rik", "po", "to", "pekka", "rjaana",
    "rjatta", "rjukka", "la", "lla", "lli", "mo", "nni",
];

// Hobbit
const HOBBIT_SYLLABLE1: &[&str] = &[
    "B", "Ber", "Br", "D", "Der", "Dr", "F", "Fr", "G", "H", "L", "Ler", "M", "Mer", "N", "P",
    "Pr", "Per", "R", "S", "T", "W",
];

const HOBBIT_SYLLABLE2: &[&str] = &["a", "e", "i", "ia", "o", "oi", "u"];

const HOBBIT_SYLLABLE3: &[&str] = &[
    "bo", "ck", "decan", "degar", "do", "doc", "go", "grin", "lba", "lbo", "lda", "ldo", "lla",
    "ll", "lo", "m", "mwise", "nac", "noc", "nwise", "p", "ppin", "pper", "tho", "to",
];

// Human
const HUMAN_SYLLABLE1: &[&str] = &[
    "Ab", "Ac", "Ad", "Af", "Agr", "Ast", "As", "Al", "Adw", "Adr", "Ar", "B", "Br", "C", "Cr",
    "Ch", "Cad", "D", "Dr", "Dw", "Ed", "Eth", "Et", "Er", "El", "Eow", "F", "Fr", "G", "Gr", "Gw",
    "Gal", "Gl", "H", "Ha", "Ib", "Jer", "K", "Ka", "Ked", "L", "Loth", "Lar", "Leg", "M", "Mir",
    "N", "Nyd", "Ol", "Oc", "On", "P", "Pr", "R", "Rh", "S", "Sev", "T", "Tr", "Th", "V", "Y", "Z",
    "W", "Wic",
];

const HUMAN_SYLLABLE2: &[&str] = &[
    "a", "ae", "au", "ao", "are", "ale", "ali", "ay", "ardo", "e", "ei", "ea", "eri", "era", "ela",
    "eli", "enda", "erra", "i", "ia", "ie", "ire", "ira", "ila", "ili", "ira", "igo", "o", "oa",
    "oi", "oe", "ore", "u", "y",
];

const HUMAN_SYLLABLE3: &[&str] = &[
    "a", "and", "b", "bwyn", "baen", "bard", "c", "ctred", "cred", "ch", "can", "d", "dan", "don",
    "der", "dric", "dfrid", "dus", "f", "g", "gord", "gan", "l", "li", "lgrin", "lin", "lith",
    "lath", "loth", "ld", "ldric", "ldan", "m", "mas", "mos", "mar", "mond", "n", "nydd", "nidd",
    "nnon", "nwan", "nyth", "nad", "nn", "nnor", "nd", "p", "r", "ron", "rd", "s", "sh", "seth",
    "sean", "t", "th", "tha", "tlan", "trem", "tram", "v", "vudd", "w", "wan", "win", "wyn", "wyr",
    "wyr", "wyth",
];

// Orc
const ORC_SYLLABLE1: &[&str] = &[
    "B", "Er", "G", "Gr", "H", "P", "Pr", "R", "V", "Vr", "T", "Tr", "M", "Dr",
];

const ORC_SYLLABLE2: &[&str] = &["a", "i", "o", "oo", "u", "ui"];

const ORC_SYLLABLE3: &[&str] = &[
    "dash", "dish", "dush", "gar", "gor", "gdush", "lo", "gdish", "k", "lg", "nak", "rag", "rbag",
    "rg", "rk", "ng", "nk", "rt", "ol", "urk", "shnak", "mog", "mak", "rak",
];

/// `my_strcat(name, s, PLAYER_NAME_LEN)` (and `my_strcpy` on an empty
/// name): append, keeping room for the NUL.
fn strcat_trunc(name: &mut Vec<u8>, s: &str) {
    for &b in s.as_bytes() {
        if name.len() >= PLAYER_NAME_LEN - 1 {
            break;
        }
        name.push(b);
    }
}

/// `set[randint0(sizeof(set) / sizeof(char *))]`
fn pick(set: &'static [&'static str]) -> &'static str {
    set[engine::randint0(set.len() as u32) as usize]
}

/// Random Name Generator: three syllables for the race, picked in order
/// (one game-RNG call each). Other races get an empty name.
fn create_random_name(race: i32) -> Vec<u8> {
    // Select the monster type
    let sets: [&'static [&'static str]; 3] = match race {
        RACE_DWARF => [DWARF_SYLLABLE1, DWARF_SYLLABLE2, DWARF_SYLLABLE3],
        RACE_ELF | RACE_HALF_ELF | RACE_HIGH_ELF => [ELF_SYLLABLE1, ELF_SYLLABLE2, ELF_SYLLABLE3],
        RACE_GNOME => [GNOME_SYLLABLE1, GNOME_SYLLABLE2, GNOME_SYLLABLE3],
        RACE_HOBBIT => [HOBBIT_SYLLABLE1, HOBBIT_SYLLABLE2, HOBBIT_SYLLABLE3],
        RACE_HUMAN | RACE_DUNADAN => [HUMAN_SYLLABLE1, HUMAN_SYLLABLE2, HUMAN_SYLLABLE3],
        RACE_HALF_ORC | RACE_HALF_TROLL | RACE_KOBOLD => {
            [ORC_SYLLABLE1, ORC_SYLLABLE2, ORC_SYLLABLE3]
        }
        // Create an empty name
        _ => return Vec::new(),
    };
    // Create the monster name
    let mut name = Vec::with_capacity(PLAYER_NAME_LEN);
    for set in sets {
        strcat_trunc(&mut name, pick(set));
    }
    name
}

/// `all_letters[i]` as a key: C `char` is signed.
fn letter_key(i: u32) -> keycode_t {
    engine::all_letters(i as usize) as i8 as i32 as keycode_t
}

/// Kill the borg and force a reincarnation.
pub fn borg_force_reincarnate(st: &mut BorgState) {
    // Kill the borg
    borg_keypress(st, b'Q' as keycode_t);
    borg_keypress(st, b'y' as keycode_t);
    borg_keypress(st, b'@' as keycode_t);

    // Respawn the borg
    borg_reincarnate_start(st);
}

/// Wrap up the reincarnation process
pub fn borg_reincarnate_end(st: &mut BorgState) {
    // C starts with `borg.player = player;` (an MSVC work-around). The
    // Rust `Borg` has no such field: the player is read through `engine`.

    // Notice the new race and class
    crate::borg_init::borg_prepare_race_class_info(st);

    crate::borg_trait::borg_notice_player(st);

    // clear out the old reactions again in case more messages came after
    // the borg died
    crate::borg_messages_react::borg_clear_reactions(st);

    // set the old depth so we know we are on a new level
    st.borg.status.old_depth = 128;

    // Mark savefile as borg cheater
    let noscore = engine::player_noscore();
    if noscore & NOSCORE_BORG == 0 {
        engine::set_player_noscore(noscore | NOSCORE_BORG);
    }

    crate::borg_init::borg_reinit_options(st);

    // We need the borg to keep playing after he dies
    engine::option_set(c"cheat_live", true);

    // Message
    borg_note(st, "# Done respawning");
}

/// Allow the borg to play continuously. Reset all values.
pub fn borg_reincarnate_start(st: &mut BorgState) {
    st.borg.status.respawning = 1;

    // clear out the old reactions
    crate::borg_messages_react::borg_clear_reactions(st);

    borg_note(st, "# Respawning");

    // New character
    borg_keypress(st, b'n' as keycode_t);
    borg_keypress(st, b'y' as keycode_t);

    // Not based on old character
    borg_keypress(st, b'n' as keycode_t);

    // C keeps race and class in `unsigned int`s
    let race: u32 = if st.cfg(BORG_RESPAWN_RACE) != -1 {
        st.cfg(BORG_RESPAWN_RACE) as u32
    } else {
        engine::randint0(MAX_RACES) as u32
    };
    let class: u32 = if st.cfg(BORG_RESPAWN_CLASS) != -1 {
        st.cfg(BORG_RESPAWN_CLASS) as u32
    } else {
        engine::randint0(MAX_CLASSES) as u32
    };

    // pick race
    if race != engine::race_ridx() {
        borg_keypress(st, letter_key(race));
    }
    borg_keypress(st, KC_ENTER);

    // pick class
    if class != engine::class_cidx() {
        borg_keypress(st, letter_key(class));
    }
    borg_keypress(st, KC_ENTER);

    // point based
    borg_keypress(st, KC_ENTER);

    // accept points
    borg_keypress(st, KC_ENTER);

    // Get a random name
    let full_name = create_random_name(race as i32);

    borg_keypresses(st, &full_name);
    borg_keypress(st, KC_ENTER);

    // accept history
    borg_keypress(st, KC_ENTER);

    // continue
    borg_keypress(st, KC_ENTER);
}

#[cfg(test)]
mod tests {
    use super::*;

    const TABLES: [&[&str]; 18] = [
        DWARF_SYLLABLE1,
        DWARF_SYLLABLE2,
        DWARF_SYLLABLE3,
        ELF_SYLLABLE1,
        ELF_SYLLABLE2,
        ELF_SYLLABLE3,
        GNOME_SYLLABLE1,
        GNOME_SYLLABLE2,
        GNOME_SYLLABLE3,
        HOBBIT_SYLLABLE1,
        HOBBIT_SYLLABLE2,
        HOBBIT_SYLLABLE3,
        HUMAN_SYLLABLE1,
        HUMAN_SYLLABLE2,
        HUMAN_SYLLABLE3,
        ORC_SYLLABLE1,
        ORC_SYLLABLE2,
        ORC_SYLLABLE3,
    ];

    /// The C array sizes (the RNG ranges).
    #[test]
    fn table_sizes_match_c() {
        let sizes: Vec<usize> = TABLES.iter().map(|t| t.len()).collect();
        assert_eq!(sizes, [15, 6, 16, 34, 17, 26, 33, 10, 32, 22, 7, 25, 65, 34, 70, 14, 6, 24]);
    }

    #[test]
    fn strcat_truncates_like_my_strcat() {
        let mut n = Vec::new();
        strcat_trunc(&mut n, "abcdefghijklmnopqrstuvwxyz");
        strcat_trunc(&mut n, "ABCDEFGHIJ");
        assert_eq!(n.len(), PLAYER_NAME_LEN - 1);
        assert_eq!(&n[26..], b"ABCDE");
    }

    /// No three-syllable name is long enough to be cut.
    #[test]
    fn longest_names_fit() {
        let longest = |t: &[&str]| t.iter().map(|s| s.len()).max().unwrap_or(0);
        for race in TABLES.chunks(3) {
            assert!(race.iter().map(|t| longest(t)).sum::<usize>() < PLAYER_NAME_LEN);
        }
    }
}
