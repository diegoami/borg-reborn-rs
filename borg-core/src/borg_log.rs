//! Port of `borg-log.c`: log borg activity either to the screen or a log
//! file.
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
//! The text written here goes through the engine's `file_putf()`, whose
//! formatter (`vstrnfmt`) prints each conversion with the C library and
//! stops at 1023 bytes; the Rust code builds the same bytes and cuts them
//! the same way (`putf`).

use crate::engine::{self, OpenFile, WriteMode};
use crate::headers::__stddef_wchar_t_h::wchar_t;
use crate::borg_headers::borg_h::{
    BORG_MONEY_SCUM_AMOUNT, BORG_PLAYS_RISKY, BORG_SAVE_DEATH, BORG_USES_SWAPS, BORG_WORSHIPS_AC,
    BORG_WORSHIPS_DAMAGE, BORG_WORSHIPS_GOLD, BORG_WORSHIPS_HP, BORG_WORSHIPS_MANA,
    BORG_WORSHIPS_SPEED,
};
use crate::borg_headers::borg_trait_h::*;
use crate::headers::list_object_flags_h::OF_AGGRAVATE;
use crate::headers::list_player_timed_h::{
    TMD_AFRAID, TMD_BLESSED, TMD_BLIND, TMD_CONFUSED, TMD_CUT, TMD_HERO, TMD_IMAGE, TMD_INVULN,
    TMD_OPP_ACID, TMD_OPP_COLD, TMD_OPP_ELEC, TMD_OPP_FIRE, TMD_OPP_POIS, TMD_POISONED,
    TMD_PROTEVIL, TMD_SHERO, TMD_SHIELD, TMD_STUN,
};
use crate::headers::obj_desc_h::{ODESC_FULL, ODESC_PREFIX};
use crate::state::BorgState;
use crate::cctype;

/// `PW_BORG_2` (player-calcs.h): the Borg status window flag.
const PW_BORG_2: u32 = 0x8000;
/// `DUNGEON_WID` (cave.h)
const DUNGEON_WID: usize = 198;
/// `DUNGEON_HGT` (cave.h)
const DUNGEON_HGT: i32 = 66;
/// `BORG_HOME` (borg-store.h): the home's store index.
const BORG_HOME: usize = 7;
/// `OF_SIZE` (obj-properties.h): bytes in an object flag set.
pub const OF_SIZE: usize = 5;

// Colours (z-color.h)
const COLOUR_WHITE: i32 = 1;
const COLOUR_SLATE: i32 = 2;
const COLOUR_GREEN: i32 = 5;
const COLOUR_BLUE: i32 = 6;

/// The buffer size of `vstrnfmt` in `file_vputf()`.
const PUTF_BUF: usize = 1024;

/// Concatenate byte strings.
fn cat(parts: &[&[u8]]) -> Vec<u8> {
    parts.concat()
}

/// `%c` as `vstrnfmt` prints it (a NUL char prints nothing).
fn chr(c: u8) -> Vec<u8> {
    if c == 0 {
        Vec::new()
    } else {
        vec![c]
    }
}

/// `%-<width>s`
fn pad_right(s: &[u8], width: usize) -> Vec<u8> {
    let mut v = s.to_vec();
    while v.len() < width {
        v.push(b' ');
    }
    v
}

/// `file_putf(f, ...)` with the formatted text: does nothing for a NULL
/// file, cuts the text to 1023 bytes as `vstrnfmt` does.
fn putf(f: &mut Option<OpenFile>, text: &[u8]) {
    if let Some(f) = f {
        f.put(&text[..text.len().min(PUTF_BUF - 1)]);
    }
}

/// `file_put(f, s)`. C dereferences a NULL file here (a crash); so does
/// this (a panic).
fn put(f: &mut Option<OpenFile>, s: &[u8]) {
    f.as_mut()
        .expect("file_put on a NULL file (crashes in C)")
        .put(s);
}

/// `borg_timer(t)`: game turns since `t`.
fn borg_timer(st: &BorgState, t: i32) -> i32 {
    st.borg.time.now.wrapping_sub(t)
}

/// The "Exp: ... Gold: ... Turn: ..." line, with C's `%lu` of a `long`.
fn exp_line(exp: i64, au: i32) -> Vec<u8> {
    format!(
        "Exp: {}  Gold: {}  Turn: {}\n",
        exp as u64,
        i64::from(au) as u64,
        i64::from(engine::game_turn()) as u64
    )
    .into_bytes()
}

/// Write a death to borg-log.txt
pub fn borg_log_death(st: &mut BorgState) {
    if st.cfg(BORG_SAVE_DEATH) == 0 {
        return;
    }

    // Build path to location of the definition file
    let path = engine::path_build_archive(1024, b"borg-log.txt");

    // Append to the file
    let Some(f) = engine::file_open(&path, WriteMode::Append) else {
        // Failure
        return;
    };
    let mut f = Some(f);

    // Get time of death
    let death_time = engine::time_now();

    // Save the date
    let date = engine::strftime_local(death_time, 80, c"%Y/%m/%d %H:%M\n");
    put(&mut f, &date);

    let p = engine::player_summary();
    putf(
        &mut f,
        &cat(&[
            &p.full_name,
            b" the ",
            p.race_name,
            b" ",
            p.class_name,
            format!(", Level {}/{}\n", p.lev, p.max_lev).as_bytes(),
        ]),
    );

    // (long)player->max_exp + (100 * player->max_depth)
    let exp = i64::from(p.max_exp) + i64::from(100 * i32::from(p.max_depth));
    putf(&mut f, &exp_line(exp, p.au));
    putf(
        &mut f,
        &cat(&[
            format!("Killed on level: {} (max. {}) by ", p.depth, p.max_depth).as_bytes(),
            &p.died_from,
            b"\n",
        ]),
    );

    putf(
        &mut f,
        format!("Borg Compile Date: {}\n", st.core.engine_date).as_bytes(),
    );

    put(&mut f, b"----------\n\n");

    if let Some(f) = f {
        f.close();
    }
}

/// Write a death to borg.dat
pub fn borg_log_death_data(st: &mut BorgState) {
    if st.cfg(BORG_SAVE_DEATH) == 0 {
        return;
    }

    let path = engine::path_build_archive(1024, b"borg.dat");

    // Append to the file
    let Some(f) = engine::file_open(&path, WriteMode::Append) else {
        // Failure
        return;
    };
    let mut f = Some(f);

    // Get time of death (unused, as in C)
    let _death_time = engine::time_now();

    // dump stuff for easy import to database
    let p = engine::player_summary();
    putf(
        &mut f,
        &cat(&[
            st.core.engine_date.as_bytes(),
            b", ",
            p.race_name,
            b", ",
            p.class_name,
            format!(", {}, {}, ", p.lev, p.depth).as_bytes(),
            &p.died_from,
            b"\n",
        ]),
    );

    if let Some(f) = f {
        f.close();
    }
}

/// Convert an inventory index into a one character label.
///
/// Note that the label does NOT distinguish inven/equip.
fn borg_index_to_label(i: i32) -> u8 {
    let pack_size = i32::from(engine::z_info().pack_size);

    // Indexes for "inven" are easy
    if i < pack_size {
        return engine::all_letters_nohjkl(i);
    }

    // Indexes for "equip" are offset
    engine::all_letters_nohjkl(i - pack_size)
}

/// Write a file with the current dungeon info (Borg)
/// and his equipment, inventory and home (Player)
/// and his swap armor, weapon (Borg)
/// NOTE: this uses internal game data.  This is okay since we are just
/// dumping the information rather than using it.
pub fn borg_write_map(st: &mut BorgState, ask: bool) {
    if st.cfg(BORG_SAVE_DEATH) == 0 {
        return;
    }

    let p = engine::player_summary();

    // Process the player name
    let mut buf: Vec<u8> = Vec::with_capacity(80);
    for &c in &p.full_name {
        // No control characters
        if cctype::c_iscntrl(c) {
            // Illegal characters (`%02X` of the signed char)
            engine::quit(
                format!(
                    "Illegal control char (0x{:02X}) in player name",
                    i32::from(c as i8)
                )
                .as_bytes(),
            );
        }

        // Convert all non-alphanumeric symbols; build "file_name"
        if !cctype::c_isalpha(c) && !cctype::c_isdigit(c) {
            buf.push(b'_');
        } else {
            buf.push(c);
        }
    }

    // Terminate
    buf.extend_from_slice(b".map");

    // The path, in C's 1024-byte buffer (get_string() edits it in place)
    let mut buf2 = [0u8; 1024];
    let path = engine::path_build_archive(1024, &buf);
    buf2[..path.len()].copy_from_slice(&path);

    let mut borg_map_file: Option<OpenFile> = None;

    // XXX XXX XXX Get the name and open the map file
    if ask && engine::get_string(st, c"Borg map File: ", &mut buf2, 70) {
        let n = buf2.iter().position(|&b| b == 0).unwrap_or(buf2.len());

        // Open a new file
        borg_map_file = engine::file_open(&buf2[..n], WriteMode::Write);

        // Failure
        if borg_map_file.is_none() {
            engine::msg(st, b"Cannot open that file.");
        }
    } else if !ask {
        borg_map_file = engine::file_open(&path, WriteMode::Write);
    }
    let f = &mut borg_map_file;

    putf(
        f,
        &cat(&[
            &p.full_name,
            b" the ",
            p.race_name,
            b" ",
            p.class_name,
            format!(", Level {}/{}\n", p.lev, p.max_lev).as_bytes(),
        ]),
    );

    // (long)(player->max_exp + (100 * player->max_depth)): int arithmetic
    let exp = p.max_exp.wrapping_add(100 * i32::from(p.max_depth));
    putf(f, &exp_line(i64::from(exp), p.au));
    putf(
        f,
        &cat(&[
            format!("Killed on level: {} (max. {}) by ", p.depth, p.max_depth).as_bytes(),
            &p.died_from,
            b"\n\n",
        ]),
    );
    putf(
        f,
        format!("Borg Compile Date: {}\n", st.core.engine_date).as_bytes(),
    );

    // `line` and `ch_line` are allocated once and reused for every row
    let mut line: Vec<wchar_t> = vec![0; DUNGEON_WID + 1];
    let mut ch_line: Vec<u8> = vec![0; DUNGEON_WID + 1];
    for i in 0..DUNGEON_HGT {
        for j in 0..DUNGEON_WID as i32 {
            let ag = *st.cave.grid(j, i);
            let m_idx: i16 = match engine::square_monster_midx(j, i) {
                Some(midx) => midx as i16,
                None => 0,
            };

            // reset the ch each time through
            let mut ch: wchar_t = b' ' as wchar_t;

            // Known grids
            if ag.feat != 0 {
                ch = engine::feat_d_char(usize::from(ag.feat));
            }

            // Known Items
            if ag.take != 0 {
                let kidx = st.takes.takes[usize::from(ag.take)].kidx;
                if kidx != 0 {
                    ch = engine::kind(kidx as usize).d_char;
                }
            }

            // UnKnown Monsters
            if m_idx != 0 {
                ch = b'&' as wchar_t;
            }

            // Known Monsters
            if ag.kill != 0 {
                let kill = st.kills.kills[usize::from(ag.kill)];
                ch = engine::race(usize::from(kill.r_idx)).d_char;
            }

            // The Player
            if i == st.borg.c.y && j == st.borg.c.x {
                ch = b'@' as wchar_t;
            }

            line[j as usize] = ch;
        }
        // terminate the line
        line[DUNGEON_WID] = 0;

        // wcstombs(ch_line, line, wcslen(line) + 1)
        let wlen = line.iter().position(|&c| c == 0).unwrap_or(line.len());
        engine::wcstombs(&mut ch_line[..wlen + 1], &line);

        let n = ch_line
            .iter()
            .position(|&b| b == 0)
            .unwrap_or(ch_line.len());
        putf(f, &cat(&[&ch_line[..n], b"\n"]));
    }

    // Known/Seen monsters
    let kills_nxt = i32::from(st.kills.nxt);
    for i in 1..kills_nxt {
        let kill = st.kills.kills[i as usize];

        // Skip dead monsters
        if kill.r_idx == 0 {
            continue;
        }

        // Note
        let name = crate::borg_flow_kill::borg_race_name(i32::from(kill.r_idx)).to_bytes();
        putf(
            f,
            &cat(&[
                b"monster '",
                name,
                format!(
                    "' ({}) at ({},{}) speed:{} \n",
                    kill.r_idx, kill.pos.y, kill.pos.x, kill.speed
                )
                .as_bytes(),
            ]),
        );
    }

    // *** Dump the last few messages ***
    let mut i = i32::from(engine::messages_num());
    if i > 250 {
        i = 250;
    }
    putf(f, b"\n\n  [Last Messages]\n\n");
    while i > 0 {
        i -= 1;
        let msg = engine::message_str(i as i16 as u16);

        // Eliminate some lines
        if msg.starts_with(b"# Matched")
            || msg.starts_with(b"# There is")
            || msg.starts_with(b"# Tracking")
            || msg.starts_with(b"# MISS_BY:")
            || msg.starts_with(b"# HIT_BY:")
            || msg.starts_with(b"> ")
        {
            continue;
        }

        putf(f, &cat(&[&msg, b"\n"]));
    }

    // *** Player Equipment ***
    putf(f, b"\n\n  [Character Equipment]\n\n");
    for i in 0..i32::from(engine::body_count()) {
        let o_name = engine::equip_desc(i as usize, ODESC_FULL);
        putf(
            f,
            &cat(&[&chr(borg_index_to_label(i)), b") ", &o_name, b"\n"]),
        );
    }
    putf(f, b"\n\n  [Character Quiver]\n\n");
    for i in 0..i32::from(engine::z_info().quiver_size) {
        let o_name = engine::quiver_desc(i as usize, ODESC_PREFIX | ODESC_FULL);
        putf(
            f,
            &cat(&[&chr(borg_index_to_label(i)), b") ", &o_name, b"\n"]),
        );
    }

    putf(f, b"\n\n");

    // Dump the inventory
    putf(f, b"  [Character Inventory]\n\n");
    for i in 0..i32::from(engine::z_info().pack_size) {
        let item = st.items.items[i as usize];
        let desc = c_chars(&item.desc);

        if item.iqty != 0 {
            if item.iqty > 1 {
                putf(
                    f,
                    &cat(&[
                        &chr(borg_index_to_label(i)),
                        format!(") {} ", item.iqty).as_bytes(),
                        &desc,
                        b"\n",
                    ]),
                );
            } else {
                putf(
                    f,
                    &cat(&[&chr(borg_index_to_label(i)), b") ", &desc, b"\n"]),
                );
            }
        }
    }
    putf(f, b"\n\n");

    // Dump the Home (page 1)
    putf(f, b"  [Home Inventory (page 1)]\n\n");
    let home = engine::store_stock_descs(BORG_HOME, ODESC_PREFIX | ODESC_FULL);
    let max = i32::from(engine::z_info().store_inven_max);
    for i in 0..max / 2 {
        let letter = engine::all_letters_nohjkl(i % 12);
        putf(f, &cat(&[&chr(letter), b") ", &home[i as usize], b"\n"]));
    }
    putf(f, b"\n\n");

    // Dump the Home (page 2)
    putf(f, b"  [Home Inventory (page 2)]\n\n");
    for i in max / 2..max {
        let letter = engine::all_letters_nohjkl(i % 12);
        putf(f, &cat(&[&chr(letter), b") ", &home[i as usize], b"\n"]));
    }
    putf(f, b"\n\n");

    // Write swap info
    if st.cfg(BORG_USES_SWAPS) != 0 {
        putf(f, b"  [Swap info]\n\n");
        let weapon_swap = st.trait_swap.weapon_swap;
        if weapon_swap != 0 {
            let item = st.items.items[(weapon_swap - 1) as usize];
            putf(f, &cat(&[b"Swap Weapon:  ", &c_chars(&item.desc), b"\n"]));
        } else {
            put(f, b"Swap Weapon:  NONE\n");
        }
        let armour_swap = st.trait_swap.armour_swap;
        if armour_swap != 0 {
            let item = st.items.items[(armour_swap - 1) as usize];
            putf(f, &cat(&[b"Swap Armour:  ", &c_chars(&item.desc), b"\n"]));
        } else {
            put(f, b"Swap Armour:  NONE\n");
        }
        putf(f, b"\n\n");
    }
    putf(f, b"  [Player State at Death] \n\n");

    // Dump the player state
    putf(
        f,
        format!("Current speed: {}. \n", st.borg.tr(BI_SPEED)).as_bytes(),
    );

    let timed = |t| engine::player_timed(t) != 0;
    if timed(TMD_BLIND) {
        putf(f, b"You cannot see.\n");
    }
    if timed(TMD_CONFUSED) {
        putf(f, b"You are confused.\n");
    }
    if timed(TMD_AFRAID) {
        putf(f, b"You are terrified.\n");
    }
    if timed(TMD_CUT) {
        putf(f, b"You are bleeding.\n");
    }
    if timed(TMD_STUN) {
        putf(f, b"You are stunned.\n");
    }
    if timed(TMD_POISONED) {
        putf(f, b"You are poisoned.\n");
    }
    if timed(TMD_IMAGE) {
        putf(f, b"You are hallucinating.\n");
    }
    if engine::player_of_has(OF_AGGRAVATE) {
        putf(f, b"You aggravate monsters.\n");
    }
    if timed(TMD_BLESSED) {
        putf(f, b"You feel righteous.\n");
    }
    if timed(TMD_HERO) {
        putf(f, b"You feel heroic.\n");
    }
    if timed(TMD_SHERO) {
        putf(f, b"You are in a battle rage.\n");
    }
    if timed(TMD_PROTEVIL) {
        putf(f, b"You are protected from evil.\n");
    }
    if timed(TMD_SHIELD) {
        putf(f, b"You are protected by a mystic shield.\n");
    }
    if timed(TMD_INVULN) {
        putf(f, b"You are temporarily invulnerable.\n");
    }
    // C bug 13: tests TMD_CONFUSED again
    if timed(TMD_CONFUSED) {
        putf(f, b"Your hands are glowing dull red.\n");
    }
    if p.word_recall != 0 {
        putf(
            f,
            format!("You will soon be recalled.  ({} turns)\n", p.word_recall).as_bytes(),
        );
    }
    if timed(TMD_OPP_FIRE) {
        putf(f, b"You resist fire exceptionally well.\n");
    }
    if timed(TMD_OPP_ACID) {
        putf(f, b"You resist acid exceptionally well.\n");
    }
    if timed(TMD_OPP_ELEC) {
        putf(f, b"You resist elec exceptionally well.\n");
    }
    if timed(TMD_OPP_COLD) {
        putf(f, b"You resist cold exceptionally well.\n");
    }
    if timed(TMD_OPP_POIS) {
        putf(f, b"You resist poison exceptionally well.\n");
    }
    putf(f, b"\n\n");

    // Dump the Time Variables
    putf(
        f,
        format!("Anti-bounce counter: {}\n", st.borg.antibounce_count).as_bytes(),
    );
    putf(
        f,
        format!(
            "Time on this level: {}\n",
            borg_timer(st, st.borg.time.level)
        )
        .as_bytes(),
    );
    putf(
        f,
        format!(
            "Time since left town: {}\n",
            borg_timer(st, st.borg.time.town)
        )
        .as_bytes(),
    );
    putf(
        f,
        format!("Borg_no_retreat: {}\n", st.borg.no_retreat).as_bytes(),
    );
    putf(
        f,
        format!("borg.near.breeder: {}\n", i32::from(st.borg.near.breeder)).as_bytes(),
    );
    // `%d` of the unsigned race index
    putf(
        f,
        format!("borg.mon.unique: {}\n", st.borg.mon.unique as i32).as_bytes(),
    );
    let day = 10 * i64::from(engine::z_info().day_length);
    if i64::from(engine::game_turn()) % day < day / 2 {
        putf(f, b"It is daytime in town.\n");
    } else {
        putf(f, b"It is night-time in town.\n");
    }
    putf(f, b"\n\n");

    putf(
        f,
        format!("borg_uses_swaps: {}\n", st.cfg(BORG_USES_SWAPS)).as_bytes(),
    );
    putf(
        f,
        format!("borg_worships_damage: {}\n", st.cfg(BORG_WORSHIPS_DAMAGE)).as_bytes(),
    );
    putf(
        f,
        format!("borg_worships_speed: {}\n", st.cfg(BORG_WORSHIPS_SPEED)).as_bytes(),
    );
    putf(
        f,
        format!("borg_worships_hp: {}\n", st.cfg(BORG_WORSHIPS_HP)).as_bytes(),
    );
    putf(
        f,
        format!("borg_worships_mana: {}\n", st.cfg(BORG_WORSHIPS_MANA)).as_bytes(),
    );
    putf(
        f,
        format!("borg_worships_ac: {}\n", st.cfg(BORG_WORSHIPS_AC)).as_bytes(),
    );
    putf(
        f,
        format!("borg_worships_gold: {}\n", st.cfg(BORG_WORSHIPS_GOLD)).as_bytes(),
    );
    putf(
        f,
        format!("borg_plays_risky: {}\n", st.cfg(BORG_PLAYS_RISKY)).as_bytes(),
    );
    putf(
        f,
        format!(
            "prepping for big fight: {}\n\n",
            st.borg.tr(BI_PREP_BIG_FIGHT)
        )
        .as_bytes(),
    );
    putf(f, b"\n\n");

    // Dump the spells
    if crate::borg_magic::borg_can_cast() {
        putf(f, b"\n\n   [ Spells ] \n\n");
        putf(f, b"Name                           Legal Times cast\n");
        for i in 0..engine::class_total_spells() {
            let as_ = &st.magic.magics[i as usize];

            if as_.level < 99 {
                let (spell, name, times) = (as_.spell_enum, as_.name, as_.times);
                let legal: &[u8] = if crate::borg_magic::borg_spell_legal(st, spell) {
                    b"Yes"
                } else {
                    b"No "
                };
                let failpercent = crate::borg_magic::borg_spell_fail_rate(st, spell);

                putf(
                    f,
                    &cat(&[
                        &pad_right(name.to_bytes(), 30),
                        b"   ",
                        legal,
                        format!("   {}   fail:{} \n", times, failpercent).as_bytes(),
                    ]),
                );
            }
        }
        putf(f, b"\n");
    }

    // Dump the borg.trait[] information
    for itemm in 0..BI_MAX as usize {
        let name = crate::borg_trait::trait_name(itemm);
        putf(
            f,
            &cat(&[
                format!("skill {} (", itemm).as_bytes(),
                name,
                format!(") value= {}.\n", st.borg.trait_[itemm]).as_bytes(),
            ]),
        );
    }

    // (The C file's "#if 0" artifact and unique dump is not compiled.)

    // C bug 13: file_close(NULL) crashes in C when the map file was not
    // opened (the `ask` path).
    borg_map_file
        .expect("file_close on a NULL file (crashes in C)")
        .close();
}

/// The bytes of a C char array up to its NUL.
fn c_chars(a: &[core::ffi::c_char]) -> Vec<u8> {
    a.iter()
        .take_while(|&&c| c != 0)
        .map(|&c| c as u8)
        .collect()
}

/// Output a long int in binary format.
fn borg_prt_binary(flags: u32, row: i32, mut col: i32) {
    let mut bitmask: u32 = 1;

    // Scan the flags
    for _ in 1..=32 {
        if flags & bitmask != 0 {
            // Dump set bits
            engine::term_putch(col, row, COLOUR_BLUE, b'*' as wchar_t);
        } else {
            // Dump unset bits
            engine::term_putch(col, row, COLOUR_WHITE, b'-' as wchar_t);
        }
        col += 1;
        bitmask = bitmask.wrapping_mul(2);
    }
}

/// This will display the values which the borg believes an item has.
/// Select the item by inven # prior to hitting the ^zo.
///
/// `f` holds `object_flags()` of the game object (`item2` in C), read by
/// the caller.
pub fn borg_display_item(st: &mut BorgState, f: &[u8; OF_SIZE], n: i32) {
    let j = 0;

    let item = st.items.items[n as usize];

    // Clear screen
    engine::term_clear();

    // Describe fully
    engine::prt(&c_chars(&item.desc), 2, j);

    engine::prt(
        format!(
            "kind = {:<5}  level = {:<4}  tval = {:<5}  sval = {:<5}",
            item.kind, item.level, item.tval, item.sval
        )
        .as_bytes(),
        4,
        j,
    );

    engine::prt(
        format!(
            "number = {:<3}  wgt = {:<6}  ac = {:<5}    damage = {}d{}",
            item.iqty, item.weight, item.ac, item.dd, item.ds
        )
        .as_bytes(),
        5,
        j,
    );

    engine::prt(
        format!(
            "pval = {:<5}  toac = {:<5}  tohit = {:<4}  todam = {:<4}",
            item.pval, item.to_a, item.to_h, item.to_d
        )
        .as_bytes(),
        6,
        j,
    );

    engine::prt(
        format!(
            "name1 = {:<4}  name2 = {:<4}  value = {}   cursed = {}   can uncurse = {}",
            item.art_idx,
            item.ego_idx,
            item.value,
            i32::from(item.cursed),
            i32::from(item.uncursable)
        )
        .as_bytes(),
        7,
        j,
    );

    engine::prt(
        format!(
            "ident = {}      timeout = {}",
            i32::from(item.ident),
            item.timeout
        )
        .as_bytes(),
        8,
        j,
    );

    // maybe print the inscription
    let note = crate::borg_item::borg_get_note(st, n as usize);
    engine::prt(
        &cat(&[
            b"Inscription: ",
            &note,
            format!(", chance: {}", st.borg.tr(BI_DEV) - i32::from(item.level)).as_bytes(),
        ]),
        9,
        j,
    );

    engine::prt(b"+------------FLAGS1------------+", 10, j);
    engine::prt(b"AFFECT..........SLAY.......BRAND", 11, j);
    engine::prt(b"                ae      xxxpaefc", 12, j);
    engine::prt(b"siwdcc  ssidsasmnvudotgddduoclio", 13, j);
    engine::prt(b"tnieoh  trnipthgiinmrrnrrmniierl", 14, j);
    engine::prt(b"rtsxna..lcfgdkttmldncltggndsdced", 15, j);
    if item.ident {
        borg_prt_binary(u32::from(f[0]), 16, j);
    }

    engine::prt(b"+------------FLAGS2------------+", 17, j);
    engine::prt(b"SUST........IMM.RESIST.........", 18, j);
    engine::prt(b"            afecaefcpfldbc s n  ", 19, j);
    engine::prt(b"siwdcc      cilocliooeialoshnecd", 20, j);
    engine::prt(b"tnieoh      irelierliatrnnnrethi", 21, j);
    engine::prt(b"rtsxna......decddcedsrekdfddxhss", 22, j);
    if item.ident {
        borg_prt_binary(u32::from(f[1]), 23, j);
    }

    engine::prt(b"+------------FLAGS3------------+", 10, j + 32);
    engine::prt(b"s   ts h     tadiiii   aiehs  hp", 11, j + 32);
    engine::prt(b"lf  eefo     egrgggg  bcnaih  vr", 12, j + 32);
    engine::prt(b"we  lerln   ilgannnn  ltssdo  ym", 13, j + 32);
    engine::prt(b"da reiedo   merirrrr  eityew ccc", 14, j + 32);
    engine::prt(b"itlepnelf   ppanaefc  svaktm uuu", 15, j + 32);
    engine::prt(b"ghigavaiu   aoveclio  saanyo rrr", 16, j + 32);
    engine::prt(b"seteticfe   craxierl  etropd sss", 17, j + 32);
    engine::prt(b"trenhstel   tttpdced  detwes eee", 18, j + 32);
    if item.ident {
        borg_prt_binary(u32::from(f[2]), 19, j + 32);
    }
}

/// `Term_putstr(x, y, -1, attr, s)`
fn put_at(x: i32, y: i32, attr: i32, s: &[u8]) {
    engine::term_putstr(x, y, -1, attr, s);
}

/// `attr = cond ? on : COLOUR_SLATE`
fn lit(cond: bool, on: i32) -> i32 {
    if cond {
        on
    } else {
        COLOUR_SLATE
    }
}

/// Display the status of various info: what the borg is thinking.
pub fn borg_display_status(st: &mut BorgState) {
    // Scan windows
    for j in 0..8 {
        let old = engine::term_current();

        // Unused
        let Some(term) = engine::angband_term(j) else {
            continue;
        };

        // Check for borg status term
        if engine::window_flag(j) & PW_BORG_2 != 0 {
            // Activate
            engine::term_activate(term);
            display_status_window(st);

            // Fresh
            engine::term_fresh();

            // Restore
            engine::term_activate(old);
        }
    }
}

/// The body of `borg_display_status()` for one status window.
fn display_status_window(st: &mut BorgState) {
    let tr = |st: &BorgState, i: u32| st.borg.tr(i) != 0;

    // Display what resists the borg (thinks he) has
    put_at(5, 0, COLOUR_WHITE, b"RESISTS");

    // Basic four
    let basic = [
        (BI_RACID, st.borg.temp.res_acid, BI_IACID, 1, &b"Acid"[..]),
        (BI_RELEC, st.borg.temp.res_elec, BI_IELEC, 2, b"Elec"),
        (BI_RFIRE, st.borg.temp.res_fire, BI_IFIRE, 3, b"Fire"),
        (BI_RCOLD, st.borg.temp.res_cold, BI_ICOLD, 4, b"Cold"),
    ];
    for (res, temp, imm, y, name) in basic {
        let mut attr = COLOUR_SLATE;
        if tr(st, res) {
            attr = COLOUR_BLUE;
        }
        if temp {
            attr = COLOUR_GREEN;
        }
        if tr(st, imm) {
            attr = COLOUR_WHITE;
        }
        put_at(1, y, attr, name);
    }

    // High resists
    let mut attr = COLOUR_SLATE;
    if tr(st, BI_RPOIS) {
        attr = COLOUR_BLUE;
    }
    if st.borg.temp.res_pois {
        attr = COLOUR_GREEN;
    }
    put_at(1, 5, attr, b"Pois");

    // (trait, x, y, text): blue when the Borg has it
    let blue: [(u32, i32, i32, &[u8]); 19] = [
        (BI_RFEAR, 1, 6, b"Fear"),
        (BI_RLITE, 1, 7, b"Lite"),
        (BI_RDARK, 1, 8, b"Dark"),
        (BI_RBLIND, 6, 1, b"Blind"),
        (BI_RCONF, 6, 2, b"Confu"),
        (BI_RSND, 6, 3, b"Sound"),
        (BI_RSHRD, 6, 4, b"Shard"),
        (BI_RNXUS, 6, 5, b"Nexus"),
        (BI_RNTHR, 6, 6, b"Nethr"),
        (BI_RKAOS, 6, 7, b"Chaos"),
        (BI_RDIS, 6, 8, b"Disen"),
        // Other abilities
        (BI_SDIG, 12, 1, b"S.Dig"),
        (BI_FEATH, 12, 2, b"Feath"),
        (BI_LIGHT, 12, 3, b"PLite"),
        (BI_REG, 12, 4, b"Regen"),
        (BI_ESP, 12, 5, b"Telep"),
        (BI_SINV, 12, 6, b"Invis"),
        (BI_FRACT, 12, 7, b"FrAct"),
        (BI_HLIFE, 12, 8, b"HLife"),
    ];
    for (t, x, y, name) in blue {
        put_at(x, y, lit(tr(st, t), COLOUR_BLUE), name);
    }

    // Display the slays
    put_at(5, 10, COLOUR_WHITE, b"Weapon Slays:");

    put_at(1, 11, lit(tr(st, BI_WS_ANIMAL), COLOUR_BLUE), b"Animal");
    put_at(8, 11, lit(tr(st, BI_WS_EVIL), COLOUR_BLUE), b"Evil");
    put_at(15, 11, lit(tr(st, BI_WS_UNDEAD), COLOUR_BLUE), b"Undead");

    // C bug 13: the slay's blue is always overwritten by the kill test
    let mut attr;
    if tr(st, BI_WS_DEMON) {
        attr = COLOUR_BLUE;
    }
    attr = lit(tr(st, BI_WK_DEMON), COLOUR_GREEN);
    put_at(22, 11, attr, b"Demon");

    put_at(1, 12, lit(tr(st, BI_WS_ORC), COLOUR_BLUE), b"Orc");
    put_at(8, 12, lit(tr(st, BI_WS_TROLL), COLOUR_BLUE), b"Troll");
    put_at(15, 12, lit(tr(st, BI_WS_GIANT), COLOUR_BLUE), b"Giant");

    // C bug 13, as for demons
    if tr(st, BI_WS_DRAGON) {
        attr = COLOUR_BLUE;
    }
    attr = lit(tr(st, BI_WK_DRAGON), COLOUR_GREEN);
    put_at(22, 12, attr, b"Dragon");

    put_at(1, 13, lit(tr(st, BI_WB_ACID), COLOUR_BLUE), b"Acid");
    put_at(8, 13, lit(tr(st, BI_WB_COLD), COLOUR_BLUE), b"Cold");
    put_at(15, 13, lit(tr(st, BI_WB_ELEC), COLOUR_BLUE), b"Elec");
    put_at(22, 13, lit(tr(st, BI_WB_FIRE), COLOUR_BLUE), b"Fire");

    // Display the Concerns
    put_at(36, 10, COLOUR_WHITE, b"Concerns:");

    put_at(29, 11, lit(tr(st, BI_FIRST_CURSED), COLOUR_BLUE), b"Cursed");
    put_at(36, 11, lit(tr(st, BI_ISWEAK), COLOUR_BLUE), b"Weak");
    put_at(43, 11, lit(tr(st, BI_ISPOISONED), COLOUR_BLUE), b"Poison");
    put_at(29, 12, lit(tr(st, BI_ISCUT), COLOUR_BLUE), b"Cut");
    put_at(36, 12, lit(tr(st, BI_ISSTUN), COLOUR_BLUE), b"Stun");
    put_at(43, 12, lit(tr(st, BI_ISCONFUSED), COLOUR_BLUE), b"Confused");
    put_at(
        29,
        13,
        lit(st.borg.goal.fleeing, COLOUR_BLUE),
        b"Goal Fleeing",
    );
    put_at(
        43,
        13,
        lit(st.borg.no_rest_prep > 0, COLOUR_BLUE),
        b"No Resting",
    );

    // Display the Time
    put_at(60, 10, COLOUR_WHITE, b"Time:");

    put_at(54, 11, COLOUR_SLATE, b"This Level         ");
    put_at(
        65,
        11,
        COLOUR_WHITE,
        borg_timer(st, st.borg.time.level).to_string().as_bytes(),
    );

    put_at(54, 12, COLOUR_SLATE, b"Since Town         ");
    put_at(
        65,
        12,
        COLOUR_WHITE,
        borg_timer(st, st.borg.time.town).to_string().as_bytes(),
    );

    put_at(54, 13, COLOUR_SLATE, b"Anti-bounce count  ");
    put_at(
        65,
        13,
        COLOUR_WHITE,
        st.borg.antibounce_count.to_string().as_bytes(),
    );

    // Sustains
    put_at(19, 0, COLOUR_WHITE, b"Sustains");

    put_at(21, 1, lit(tr(st, BI_SSTR), COLOUR_WHITE), b"STR");
    put_at(21, 2, lit(tr(st, BI_SINT), COLOUR_WHITE), b"INT");
    put_at(21, 3, lit(tr(st, BI_SWIS), COLOUR_WHITE), b"WIS");
    put_at(21, 4, lit(tr(st, BI_SDEX), COLOUR_WHITE), b"DEX");
    put_at(21, 5, lit(tr(st, BI_SCON), COLOUR_WHITE), b"CON");

    // Temporary effects
    put_at(28, 0, COLOUR_WHITE, b"Temp Effects");

    let temp = &st.borg.temp;
    put_at(28, 1, lit(temp.prot_from_evil, COLOUR_WHITE), b"Prot. Evil");
    put_at(28, 2, lit(temp.fastcast, COLOUR_WHITE), b"Fastcast");
    put_at(28, 3, lit(temp.hero, COLOUR_WHITE), b"Heroism");
    put_at(28, 4, lit(temp.berserk, COLOUR_WHITE), b"Berserk");
    put_at(28, 5, lit(temp.shield, COLOUR_WHITE), b"Shielded");
    put_at(28, 6, lit(temp.bless, COLOUR_WHITE), b"Blessed");
    put_at(28, 7, lit(temp.fast, COLOUR_WHITE), b"Fast");
    put_at(28, 8, lit(st.borg.see_inv >= 1, COLOUR_WHITE), b"See Inv");

    // Temporary effects
    put_at(42, 0, COLOUR_WHITE, b"Level Information");

    put_at(
        42,
        1,
        lit(st.borg.status.vault, COLOUR_WHITE),
        b"Vault on level",
    );

    let unique = st.borg.mon.unique;
    let attr = lit(unique != 0, COLOUR_WHITE);
    put_at(42, 2, attr, b"Unique on level");
    if unique != 0 {
        put_at(
            58,
            2,
            attr,
            &cat(&[b"(", engine::race_name(unique as usize), b")"]),
        );
    } else {
        put_at(
            58,
            2,
            attr,
            b"                                                ",
        );
    }

    put_at(
        42,
        3,
        lit(st.borg.mon.scary, COLOUR_WHITE),
        b"Scary Guy on level",
    );
    put_at(
        42,
        4,
        lit(st.borg.near.breeder, COLOUR_WHITE),
        b"Breeder level (closing doors)",
    );
    put_at(
        42,
        5,
        lit(st.borg.near.summoner_idx != -1, COLOUR_WHITE),
        b"Summoner very close (AS-Corridor)",
    );

    // level preparedness
    put_at(42, 6, COLOUR_SLATE, b"Reason for not diving:");
    let depth = st.borg.tr(BI_MAXDEPTH) + 1;
    let reason = crate::borg_prepared::borg_prepared_c(st, depth).unwrap_or_default();
    put_at(
        64,
        6,
        COLOUR_WHITE,
        &cat(&[&reason, b"                              "]),
    );

    put_at(
        42,
        7,
        COLOUR_SLATE,
        b"Scumming: not active                          ",
    );
    if st.cfg(BORG_MONEY_SCUM_AMOUNT) != 0 {
        put_at(
            42,
            7,
            COLOUR_WHITE,
            format!(
                "Scumming: ${}                  ",
                st.cfg(BORG_MONEY_SCUM_AMOUNT)
            )
            .as_bytes(),
        );
    }
    put_at(42, 8, COLOUR_SLATE, b"Maximal Depth:");
    put_at(
        56,
        8,
        COLOUR_WHITE,
        format!("{}    ", st.borg.tr(BI_MAXDEPTH)).as_bytes(),
    );

    let h = &st.home;
    let (ezheal, heal, life, mana) = (
        i32::from(h.num_ezheal),
        i32::from(h.num_heal),
        i32::from(h.num_life),
        i32::from(h.num_mana),
    );

    // Important endgame information
    if st.borg.tr(BI_MAXDEPTH) >= 50 {
        // 85
        put_at(5, 15, COLOUR_WHITE, b"Important Deep Events:");

        put_at(1, 16, COLOUR_SLATE, b"Home *Heal*:        ");
        put_at(13, 16, COLOUR_WHITE, format!("{ezheal}   ").as_bytes());

        put_at(1, 17, COLOUR_SLATE, b"Home Heal:        ");
        put_at(11, 17, COLOUR_WHITE, format!("{heal}   ").as_bytes());

        put_at(1, 18, COLOUR_SLATE, b"Home Life:        ");
        put_at(11, 18, COLOUR_WHITE, format!("{life}   ").as_bytes());

        put_at(1, 19, COLOUR_SLATE, b"Res_Mana:        ");
        put_at(11, 19, COLOUR_WHITE, format!("{mana}   ").as_bytes());

        put_at(
            1,
            20,
            lit(st.borg.near.morgoth, COLOUR_BLUE),
            format!(
                "Morgoth on Level.  Last seen:{}       ",
                borg_timer(st, st.borg.time.morgoth)
            )
            .as_bytes(),
        );

        let mut attr = lit(st.borg.morgoth_position, COLOUR_BLUE);
        if st.glyph.needs_new_sea {
            attr = COLOUR_WHITE;
        }
        put_at(1, 21, attr, b"Sea of Runes.");

        put_at(
            1,
            22,
            lit(st.borg.ready_morgoth != 0, COLOUR_BLUE),
            b"Ready for Morgoth.",
        );
    } else {
        put_at(5, 15, COLOUR_WHITE, b"                        ");

        put_at(1, 16, COLOUR_SLATE, b"                    ");
        put_at(10, 16, COLOUR_WHITE, format!("{ezheal}       ").as_bytes());

        put_at(1, 17, COLOUR_SLATE, b"                    ");
        put_at(10, 17, COLOUR_WHITE, format!("{life}       ").as_bytes());

        put_at(1, 18, COLOUR_SLATE, b"                    ");
        put_at(11, 18, COLOUR_WHITE, format!("{heal}       ").as_bytes());

        put_at(1, 19, COLOUR_SLATE, b"                   ");
        put_at(11, 19, COLOUR_WHITE, format!("{mana}       ").as_bytes());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pad_right_matches_printf() {
        assert_eq!(
            pad_right(b"Magic Missile", 30),
            b"Magic Missile                 ".to_vec()
        );
        assert_eq!(pad_right(&[b'x'; 35], 30).len(), 35);
    }

    #[test]
    fn chr_skips_nul() {
        assert!(chr(0).is_empty());
        assert_eq!(chr(b'a'), b"a".to_vec());
    }
}
