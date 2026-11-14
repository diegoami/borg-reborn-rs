//! `/home/user/borg-rs/angband/src/cmd-core.h`
pub use super::object_h::object;
pub use super::z_type_h::loc;
#[allow(unused_imports)]
use super::*;
extern "C" {
    pub fn cmd_get_nrepeats() -> ::core::ffi::c_int;
}
pub type cmd_code = ::core::ffi::c_uint;
pub const CMD_COMMAND_MONSTER: cmd_code = 114;
pub const CMD_REPEAT: cmd_code = 113;
pub const CMD_HELP: cmd_code = 112;
pub const CMD_RETIRE: cmd_code = 111;
pub const CMD_WIZ_WIZARD_LIGHT: cmd_code = 110;
pub const CMD_WIZ_WIPE_RECALL: cmd_code = 109;
pub const CMD_WIZ_TWEAK_ITEM: cmd_code = 108;
pub const CMD_WIZ_TELEPORT_TO: cmd_code = 107;
pub const CMD_WIZ_TELEPORT_RANDOM: cmd_code = 106;
pub const CMD_WIZ_SUMMON_RANDOM: cmd_code = 105;
pub const CMD_WIZ_SUMMON_NAMED: cmd_code = 104;
pub const CMD_WIZ_STAT_ITEM: cmd_code = 103;
pub const CMD_WIZ_REROLL_ITEM: cmd_code = 102;
pub const CMD_WIZ_RERATE: cmd_code = 101;
pub const CMD_WIZ_RECALL_MONSTER: cmd_code = 100;
pub const CMD_WIZ_QUIT_NO_SAVE: cmd_code = 99;
pub const CMD_WIZ_QUERY_SQUARE_FLAG: cmd_code = 98;
pub const CMD_WIZ_QUERY_FEATURE: cmd_code = 97;
pub const CMD_WIZ_PUSH_OBJECT: cmd_code = 96;
pub const CMD_WIZ_PLAY_ITEM: cmd_code = 95;
pub const CMD_WIZ_PERFORM_EFFECT: cmd_code = 94;
pub const CMD_WIZ_PEEK_NOISE_SCENT: cmd_code = 93;
pub const CMD_WIZ_MAGIC_MAP: cmd_code = 92;
pub const CMD_WIZ_LEARN_OBJECT_KINDS: cmd_code = 91;
pub const CMD_WIZ_JUMP_LEVEL: cmd_code = 90;
pub const CMD_WIZ_INCREASE_EXP: cmd_code = 89;
pub const CMD_WIZ_HIT_ALL_LOS: cmd_code = 88;
pub const CMD_WIZ_EDIT_PLAYER_STAT: cmd_code = 87;
pub const CMD_WIZ_EDIT_PLAYER_START: cmd_code = 86;
pub const CMD_WIZ_EDIT_PLAYER_GOLD: cmd_code = 85;
pub const CMD_WIZ_EDIT_PLAYER_EXP: cmd_code = 84;
pub const CMD_WIZ_DUMP_LEVEL_MAP: cmd_code = 83;
pub const CMD_WIZ_DETECT_ALL_MONSTERS: cmd_code = 82;
pub const CMD_WIZ_DETECT_ALL_LOCAL: cmd_code = 81;
pub const CMD_WIZ_CURSE_ITEM: cmd_code = 80;
pub const CMD_WIZ_CURE_ALL: cmd_code = 79;
pub const CMD_WIZ_CREATE_TRAP: cmd_code = 78;
pub const CMD_WIZ_CREATE_OBJ: cmd_code = 77;
pub const CMD_WIZ_CREATE_ARTIFACT: cmd_code = 76;
pub const CMD_WIZ_CREATE_ALL_OBJ_FROM_TVAL: cmd_code = 75;
pub const CMD_WIZ_CREATE_ALL_OBJ: cmd_code = 74;
pub const CMD_WIZ_CREATE_ALL_ARTIFACT_FROM_TVAL: cmd_code = 73;
pub const CMD_WIZ_CREATE_ALL_ARTIFACT: cmd_code = 72;
pub const CMD_WIZ_COLLECT_PIT_STATS: cmd_code = 71;
pub const CMD_WIZ_COLLECT_OBJ_MON_STATS: cmd_code = 70;
pub const CMD_WIZ_COLLECT_DISCONNECT_STATS: cmd_code = 69;
pub const CMD_WIZ_CHANGE_ITEM_QUANTITY: cmd_code = 68;
pub const CMD_WIZ_BANISH: cmd_code = 67;
pub const CMD_WIZ_ADVANCE: cmd_code = 66;
pub const CMD_WIZ_ACQUIRE: cmd_code = 65;
pub const CMD_SPOIL_OBJ: cmd_code = 64;
pub const CMD_SPOIL_MON_BRIEF: cmd_code = 63;
pub const CMD_SPOIL_MON: cmd_code = 62;
pub const CMD_SPOIL_ARTIFACT: cmd_code = 61;
pub const CMD_RETRIEVE: cmd_code = 60;
pub const CMD_STASH: cmd_code = 59;
pub const CMD_BUY: cmd_code = 58;
pub const CMD_SELL: cmd_code = 57;
pub const CMD_SLEEP: cmd_code = 56;
pub const CMD_STEAL: cmd_code = 55;
pub const CMD_ALTER: cmd_code = 54;
pub const CMD_HOLD: cmd_code = 53;
pub const CMD_NAVIGATE_DOWN: cmd_code = 52;
pub const CMD_NAVIGATE_UP: cmd_code = 51;
pub const CMD_EXPLORE: cmd_code = 50;
pub const CMD_RUN: cmd_code = 49;
pub const CMD_CLOSE: cmd_code = 48;
pub const CMD_OPEN: cmd_code = 47;
pub const CMD_TUNNEL: cmd_code = 46;
pub const CMD_REST: cmd_code = 45;
pub const CMD_DISARM: cmd_code = 44;
pub const CMD_IGNORE: cmd_code = 43;
pub const CMD_AUTOPICKUP: cmd_code = 42;
pub const CMD_PICKUP: cmd_code = 41;
pub const CMD_THROW: cmd_code = 40;
pub const CMD_FIRE: cmd_code = 39;
pub const CMD_USE: cmd_code = 38;
pub const CMD_REFILL: cmd_code = 37;
pub const CMD_READ_SCROLL: cmd_code = 36;
pub const CMD_QUAFF: cmd_code = 35;
pub const CMD_EAT: cmd_code = 34;
pub const CMD_ACTIVATE: cmd_code = 33;
pub const CMD_USE_ROD: cmd_code = 32;
pub const CMD_USE_WAND: cmd_code = 31;
pub const CMD_USE_STAFF: cmd_code = 30;
pub const CMD_CAST: cmd_code = 29;
pub const CMD_STUDY: cmd_code = 28;
pub const CMD_BROWSE_SPELL: cmd_code = 27;
pub const CMD_DROP: cmd_code = 26;
pub const CMD_WIELD: cmd_code = 25;
pub const CMD_TAKEOFF: cmd_code = 24;
pub const CMD_AUTOINSCRIBE: cmd_code = 23;
pub const CMD_UNINSCRIBE: cmd_code = 22;
pub const CMD_INSCRIBE: cmd_code = 21;
pub const CMD_PATHFIND: cmd_code = 20;
pub const CMD_JUMP: cmd_code = 19;
pub const CMD_WALK: cmd_code = 18;
pub const CMD_GO_DOWN: cmd_code = 17;
pub const CMD_GO_UP: cmd_code = 16;
pub const CMD_ACCEPT_CHARACTER: cmd_code = 15;
pub const CMD_HISTORY_CHOICE: cmd_code = 14;
pub const CMD_NAME_CHOICE: cmd_code = 13;
pub const CMD_PREV_STATS: cmd_code = 12;
pub const CMD_ROLL_STATS: cmd_code = 11;
pub const CMD_REFRESH_STATS: cmd_code = 10;
pub const CMD_RESET_STATS: cmd_code = 9;
pub const CMD_SELL_STAT: cmd_code = 8;
pub const CMD_BUY_STAT: cmd_code = 7;
pub const CMD_CHOOSE_CLASS: cmd_code = 6;
pub const CMD_CHOOSE_RACE: cmd_code = 5;
pub const CMD_BIRTH_RESET: cmd_code = 4;
pub const CMD_BIRTH_INIT: cmd_code = 3;
pub const CMD_NEWGAME: cmd_code = 2;
pub const CMD_LOADFILE: cmd_code = 1;
pub const CMD_NULL: cmd_code = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct command {
    pub context: cmd_context,
    pub code: cmd_code,
    pub nrepeats: ::core::ffi::c_int,
    pub background_command: ::core::ffi::c_int,
    pub arg: [cmd_arg; 4],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct cmd_arg {
    pub type_0: cmd_arg_type,
    pub data: cmd_arg_data,
    pub name: [::core::ffi::c_char; 20],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union cmd_arg_data {
    pub string: *const ::core::ffi::c_char,
    pub choice: ::core::ffi::c_int,
    pub obj: *mut object,
    pub number: ::core::ffi::c_int,
    pub direction: ::core::ffi::c_int,
    pub point: loc,
}
pub type cmd_arg_type = ::core::ffi::c_uint;
pub const arg_POINT: cmd_arg_type = 7;
pub const arg_TARGET: cmd_arg_type = 6;
pub const arg_DIRECTION: cmd_arg_type = 5;
pub const arg_NUMBER: cmd_arg_type = 4;
pub const arg_ITEM: cmd_arg_type = 3;
pub const arg_CHOICE: cmd_arg_type = 2;
pub const arg_STRING: cmd_arg_type = 1;
pub const arg_NONE: cmd_arg_type = 0;
pub type cmd_context = ::core::ffi::c_uint;
pub const CTX_DEATH: cmd_context = 4;
pub const CTX_STORE: cmd_context = 3;
pub const CTX_GAME: cmd_context = 2;
pub const CTX_BIRTH: cmd_context = 1;
pub const CTX_INIT: cmd_context = 0;
