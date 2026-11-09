//! `/home/user/borg-rs/angband/src/game-event.h`
pub use super::object_h::object;
pub use super::z_type_h::loc;
#[allow(unused_imports)]
use super::*;
pub type game_event_type = ::core::ffi::c_uint;
pub const EVENT_END: game_event_type = 64;
pub const EVENT_GEN_TUNNEL_FINISHED: game_event_type = 63;
pub const EVENT_GEN_ROOM_END: game_event_type = 62;
pub const EVENT_GEN_ROOM_CHOOSE_SUBTYPE: game_event_type = 61;
pub const EVENT_GEN_ROOM_CHOOSE_SIZE: game_event_type = 60;
pub const EVENT_GEN_ROOM_START: game_event_type = 59;
pub const EVENT_GEN_LEVEL_END: game_event_type = 58;
pub const EVENT_GEN_LEVEL_START: game_event_type = 57;
pub const EVENT_LEAVE_DEATH: game_event_type = 56;
pub const EVENT_ENTER_DEATH: game_event_type = 55;
pub const EVENT_LEAVE_STORE: game_event_type = 54;
pub const EVENT_ENTER_STORE: game_event_type = 53;
pub const EVENT_LEAVE_WORLD: game_event_type = 52;
pub const EVENT_ENTER_WORLD: game_event_type = 51;
pub const EVENT_LEAVE_GAME: game_event_type = 50;
pub const EVENT_ENTER_GAME: game_event_type = 49;
pub const EVENT_LEAVE_BIRTH: game_event_type = 48;
pub const EVENT_ENTER_BIRTH: game_event_type = 47;
pub const EVENT_LEAVE_INIT: game_event_type = 46;
pub const EVENT_ENTER_INIT: game_event_type = 45;
pub const EVENT_BIRTHPOINTS: game_event_type = 44;
pub const EVENT_INITSTATUS: game_event_type = 43;
pub const EVENT_CHEAT_DEATH: game_event_type = 42;
pub const EVENT_ANIMATE: game_event_type = 41;
pub const EVENT_COMMAND_REPEAT: game_event_type = 40;
pub const EVENT_NEW_LEVEL_DISPLAY: game_event_type = 39;
pub const EVENT_REFRESH: game_event_type = 38;
pub const EVENT_CHECK_INTERRUPT: game_event_type = 37;
pub const EVENT_MESSAGE_FLUSH: game_event_type = 36;
pub const EVENT_INPUT_FLUSH: game_event_type = 35;
pub const EVENT_STORECHANGED: game_event_type = 34;
pub const EVENT_USE_STORE: game_event_type = 33;
pub const EVENT_BELL: game_event_type = 32;
pub const EVENT_SOUND: game_event_type = 31;
pub const EVENT_MESSAGE: game_event_type = 30;
pub const EVENT_OBJECTTARGET: game_event_type = 29;
pub const EVENT_MONSTERTARGET: game_event_type = 28;
pub const EVENT_MONSTERLIST: game_event_type = 27;
pub const EVENT_ITEMLIST: game_event_type = 26;
pub const EVENT_EQUIPMENT: game_event_type = 25;
pub const EVENT_INVENTORY: game_event_type = 24;
pub const EVENT_MISSILE: game_event_type = 23;
pub const EVENT_BOLT: game_event_type = 22;
pub const EVENT_EXPLOSION: game_event_type = 21;
pub const EVENT_SEEFLOOR: game_event_type = 20;
pub const EVENT_PLAYERMOVED: game_event_type = 19;
pub const EVENT_STATE: game_event_type = 18;
pub const EVENT_LIGHT: game_event_type = 17;
pub const EVENT_FEELING: game_event_type = 16;
pub const EVENT_DETECTIONSTATUS: game_event_type = 15;
pub const EVENT_STATUS: game_event_type = 14;
pub const EVENT_STUDYSTATUS: game_event_type = 13;
pub const EVENT_RACE_CLASS: game_event_type = 12;
pub const EVENT_PLAYERSPEED: game_event_type = 11;
pub const EVENT_DUNGEONLEVEL: game_event_type = 10;
pub const EVENT_MONSTERHEALTH: game_event_type = 9;
pub const EVENT_GOLD: game_event_type = 8;
pub const EVENT_PLAYERTITLE: game_event_type = 7;
pub const EVENT_PLAYERLEVEL: game_event_type = 6;
pub const EVENT_EXPERIENCE: game_event_type = 5;
pub const EVENT_AC: game_event_type = 4;
pub const EVENT_MANA: game_event_type = 3;
pub const EVENT_HP: game_event_type = 2;
pub const EVENT_STATS: game_event_type = 1;
pub const EVENT_MAP: game_event_type = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub union game_event_data {
    pub point: loc,
    pub string: *const ::core::ffi::c_char,
    pub flag: bool,
    pub message: C2RustUnnamedH_7,
    pub birthstage: C2RustUnnamedH_6,
    pub birthpoints: C2RustUnnamedH_5,
    pub explosion: C2RustUnnamedH_4,
    pub bolt: C2RustUnnamedH_3,
    pub missile: C2RustUnnamedH_2,
    pub size: C2RustUnnamedH_1,
    pub tunnel: C2RustUnnamedH_0,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamedH_0 {
    pub nstep: ::core::ffi::c_int,
    pub npierce: ::core::ffi::c_int,
    pub ndug: ::core::ffi::c_int,
    pub dstart: ::core::ffi::c_int,
    pub dend: ::core::ffi::c_int,
    pub early: bool,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamedH_1 {
    pub h: ::core::ffi::c_int,
    pub w: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamedH_2 {
    pub obj: *mut object,
    pub seen: bool,
    pub y: ::core::ffi::c_int,
    pub x: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamedH_3 {
    pub proj_type: ::core::ffi::c_int,
    pub drawing: bool,
    pub seen: bool,
    pub beam: bool,
    pub oy: ::core::ffi::c_int,
    pub ox: ::core::ffi::c_int,
    pub y: ::core::ffi::c_int,
    pub x: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamedH_4 {
    pub proj_type: ::core::ffi::c_int,
    pub num_grids: ::core::ffi::c_int,
    pub distance_to_grid: *mut ::core::ffi::c_int,
    pub drawing: bool,
    pub player_sees_grid: *mut bool,
    pub blast_grid: *mut loc,
    pub centre: loc,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamedH_5 {
    pub points: *const ::core::ffi::c_int,
    pub inc_points: *const ::core::ffi::c_int,
    pub remaining: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamedH_6 {
    pub reset: bool,
    pub hint: *const ::core::ffi::c_char,
    pub n_choices: ::core::ffi::c_int,
    pub initial_choice: ::core::ffi::c_int,
    pub choices: *mut *const ::core::ffi::c_char,
    pub helptexts: *mut *const ::core::ffi::c_char,
    pub xtra: *mut ::core::ffi::c_void,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamedH_7 {
    pub msg: *const ::core::ffi::c_char,
    pub type_0: ::core::ffi::c_int,
}
extern "C" {
    pub fn event_signal(_: game_event_type);
}
pub type game_event_handler =
    unsafe extern "C" fn(game_event_type, *mut game_event_data, *mut ::core::ffi::c_void) -> ();
extern "C" {
    pub fn event_add_handler(
        type_0: game_event_type,
        fn_0: Option<game_event_handler>,
        user: *mut ::core::ffi::c_void,
    );
}
extern "C" {
    pub fn event_remove_handler(
        type_0: game_event_type,
        fn_0: Option<game_event_handler>,
        user: *mut ::core::ffi::c_void,
    );
}
extern "C" {
    pub fn event_signal_message(
        type_0: game_event_type,
        t: ::core::ffi::c_int,
        s: *const ::core::ffi::c_char,
    );
}
