//! Rust port of the Angband Borg.
//!
//! One module per C file of `angband/src/borg/` (upstream copyright: Ben
//! Harrison, James E. Wilson, Robert A. Koeneke, Andi Sidwell, Chris Carr,
//! Ed Graham, Erik Osheim and other Angband/Borg contributors; GPLv2 /
//! Angband licence). `engine` is the FFI boundary to the C game engine,
//! `headers` the engine's raw C declarations, `borg_headers` the
//! declarations of the Borg's own headers (not FFI), and `state` the
//! Borg's state (its former C globals).
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
// C's dead stores are kept as they are (faithful port).
#![allow(unused_assignments)]
// No `unsafe` outside the FFI boundary: `engine` and the engine's
// declarations in `headers`.
#![deny(unsafe_code)]

extern crate libc;

#[allow(dead_code, unused_imports)]
pub(crate) mod borg_headers;
pub(crate) mod cctype;
#[allow(unsafe_code)]
#[allow(dead_code, unused_imports)]
pub(crate) mod headers;

pub(crate) mod borg;
pub(crate) mod borg_buy_strategy;
pub(crate) mod borg_attack_munchkin;
pub(crate) mod borg_cave;
pub(crate) mod borg_caution;
pub(crate) mod borg_cave_light;
pub(crate) mod borg_cave_view;
pub(crate) mod borg_cave_util;
pub(crate) mod borg_commands;
pub(crate) mod borg_danger;
pub(crate) mod borg_escape;
pub(crate) mod borg_fight_attack;
pub(crate) mod borg_fight_defend;
pub(crate) mod borg_fight_perm;
pub(crate) mod borg_flow;
pub(crate) mod borg_flow_glyph;
pub(crate) mod borg_flow_dark;
pub(crate) mod borg_flow_take;
pub(crate) mod borg_flow_kill;
pub(crate) mod borg_flow_misc;
pub(crate) mod borg_flow_stairs;
pub(crate) mod borg_formulas;
pub(crate) mod borg_formulas_calc;
pub(crate) mod borg_home_notice;
pub(crate) mod borg_home_power;
pub(crate) mod borg_init;
pub(crate) mod borg_inventory;
pub(crate) mod borg_io;
pub(crate) mod borg_item;
pub(crate) mod borg_item_activation;
pub(crate) mod borg_item_analyze;
pub(crate) mod borg_item_decurse;
pub(crate) mod borg_item_enchant;
pub(crate) mod borg_item_id;
pub(crate) mod borg_item_use;
pub(crate) mod borg_item_val;
pub(crate) mod borg_item_wear;
pub(crate) mod borg_junk;
pub(crate) mod borg_light;
pub(crate) mod borg_log;
pub(crate) mod borg_magic;
pub(crate) mod borg_magic_play;
pub(crate) mod borg_messages;
pub(crate) mod borg_messages_react;
pub(crate) mod borg_power;
pub(crate) mod borg_prepared;
pub(crate) mod borg_projection;
pub(crate) mod borg_recover;
pub(crate) mod borg_reincarnate;
pub(crate) mod borg_store;
pub(crate) mod borg_store_sell;
pub(crate) mod borg_store_buy;
pub(crate) mod borg_think;
pub(crate) mod borg_think_dungeon;
pub(crate) mod borg_think_dungeon_util;
pub(crate) mod borg_think_store;
pub(crate) mod borg_trait;
pub(crate) mod borg_trait_swap;
pub(crate) mod borg_update;
pub(crate) mod borg_util;
#[allow(unsafe_code)]
pub(crate) mod engine;
pub(crate) mod state;
pub mod regress;
// borg-reborn-rs (#208): the errands' API.
pub mod shell;

/// The `^Z` command of this core (borg-reborn-rs #207): the shell's
/// `do_cmd_borg` calls it when `BORG_CORE=borg-rs` selects this core.
pub fn do_cmd_borg() {
    engine::do_cmd_borg()
}
