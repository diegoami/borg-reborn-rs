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
pub(crate) mod borg_cave;
pub(crate) mod borg_caution;
mod borg_combat_heal;
mod borg_healing_restock;
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
pub(crate) mod borg_kit;
pub(crate) mod borg_ranged;
pub(crate) mod borg_junk;
pub(crate) mod borg_light;
mod borg_cursed;
mod borg_lightless;
pub(crate) mod borg_log;
pub(crate) mod borg_magic;
pub(crate) mod borg_magic_play;
pub(crate) mod borg_messages;
pub(crate) mod borg_messages_react;
pub(crate) mod borg_object_log;
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
mod borg_town_home; // borg-reborn-rs (#1241): Home reserve policy
pub(crate) mod borg_town_capability;
pub(crate) mod borg_town_packing;
pub(crate) mod borg_town_budget;
pub(crate) mod borg_town_dispose;
pub(crate) mod borg_town_dump;
pub(crate) mod borg_town_record;
pub(crate) mod borg_town_traits;
pub(crate) mod borg_town_shadow;
pub(crate) mod borg_town_solve;
pub(crate) mod borg_town_stock;
pub(crate) mod borg_town_usefulness;
pub(crate) mod borg_town_utility;
pub(crate) mod borg_town_live;
mod borg_town_loadout;
mod borg_spare_gear;
mod borg_town_recall_loadout;
pub(crate) mod borg_trip;
pub(crate) mod borg_trait;
pub(crate) mod borg_trait_swap;
pub(crate) mod borg_update;
pub(crate) mod borg_util;
#[allow(unsafe_code)]
pub(crate) mod engine;
pub(crate) mod state;
pub mod regress;
// borg-reborn-rs (#1339): the run's configuration file (the errands of
// #208, whose module held it, are retired).
pub mod config_file;
// borg-reborn-rs (#219): calibration registry.
pub(crate) mod params;
// borg-reborn-rs (#222): observational decision tracing.
pub(crate) mod core_trace;
// borg-reborn-rs (#403): pure class/family attack ranking.
pub(crate) mod attack_preference;

/// Run a key from the shared Borg command prompt.
pub fn do_cmd_borg_key(cmd: u8) {
    engine::do_cmd_borg_key(cmd)
}

pub mod scary_list;
mod borg_fast_fragile;
mod borg_unseen_belief;
mod borg_trap_way;
mod borg_lone_target;
mod borg_leash; // borg-reborn-rs (#1062)
mod borg_poison; // borg-reborn-rs (#1140)
mod borg_town_hunt;
// borg-reborn-rs (#1061): the stair_scum exploration mode.
mod borg_stair_scum;
mod borg_loop; // borg-reborn-rs (#1153)
