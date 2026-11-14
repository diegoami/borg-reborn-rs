//! `/home/user/borg-rs/angband/src/cave.h`
pub use super::__stddef_wchar_t_h::wchar_t;
pub use super::monster_h::monster;
pub use super::object_h::object;
pub use super::object_h::object_kind;
pub use super::player_h::player;
pub use super::stdint_intn_h::int16_t;
pub use super::stdint_intn_h::int32_t;
pub use super::stdint_uintn_h::uint16_t;
pub use super::stdint_uintn_h::uint32_t;
pub use super::stdint_uintn_h::uint8_t;
pub use super::trap_h::trap;
pub use super::z_bitflag_h::bitflag;
pub use super::z_type_h::loc;
#[allow(unused_imports)]
use super::*;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct heatmap {
    pub grids: *mut *mut uint16_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct chunk {
    pub name: *mut ::core::ffi::c_char,
    pub turn: int32_t,
    pub depth: ::core::ffi::c_int,
    pub feeling: uint8_t,
    pub obj_rating: uint32_t,
    pub mon_rating: uint32_t,
    pub good_item: bool,
    pub height: ::core::ffi::c_int,
    pub width: ::core::ffi::c_int,
    pub feeling_squares: uint16_t,
    pub feat_count: *mut ::core::ffi::c_int,
    pub squares: *mut *mut square,
    pub noise: heatmap,
    pub scent: heatmap,
    pub decoy: loc,
    pub objects: *mut *mut object,
    pub obj_max: uint16_t,
    pub monsters: *mut monster,
    pub mon_max: uint16_t,
    pub mon_cnt: uint16_t,
    pub mon_current: ::core::ffi::c_int,
    pub num_repro: ::core::ffi::c_int,
    pub monster_groups: *mut *mut monster_group,
    pub join: *mut connector,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct connector {
    pub grid: loc,
    pub feat: uint8_t,
    pub info: *mut bitflag,
    pub next: *mut connector,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct square {
    pub feat: uint8_t,
    pub info: *mut bitflag,
    pub light: ::core::ffi::c_int,
    pub mon: int16_t,
    pub obj: *mut object,
    pub trap: *mut trap,
}
#[repr(C)]
pub struct monster_group {
    _opaque: [u8; 0],
}
pub type C2RustUnnamedH_18 = ::core::ffi::c_uint;
pub const FEAT_MAX: C2RustUnnamedH_18 = 25;
extern "C" {
    pub fn distance(grid1: loc, grid2: loc) -> ::core::ffi::c_int;
}
extern "C" {
    pub static ddd: [int16_t; 9];
}
extern "C" {
    pub static ddx: [int16_t; 10];
}
extern "C" {
    pub static ddy: [int16_t; 10];
}
extern "C" {
    pub static ddx_ddd: [int16_t; 9];
}
extern "C" {
    pub static ddy_ddd: [int16_t; 9];
}
extern "C" {
    pub static mut cave: *mut chunk;
}
extern "C" {
    pub fn feat_is_shop(feat: ::core::ffi::c_int) -> bool;
}
extern "C" {
    pub fn square_isvault(c: *mut chunk, grid: loc) -> bool;
}
extern "C" {
    pub fn square_in_bounds(c: *mut chunk, grid: loc) -> bool;
}
extern "C" {
    pub fn square_in_bounds_fully(c: *mut chunk, grid: loc) -> bool;
}
pub type C2RustUnnamedH_24 = ::core::ffi::c_uint;
pub const SQUARE_MAX: C2RustUnnamedH_24 = 22;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct feature {
    pub name: *mut ::core::ffi::c_char,
    pub desc: *mut ::core::ffi::c_char,
    pub fidx: ::core::ffi::c_int,
    pub mimic: *mut feature,
    pub priority: uint8_t,
    pub shopnum: uint8_t,
    pub dig: uint8_t,
    pub flags: [bitflag; 4],
    pub d_attr: uint8_t,
    pub d_char: wchar_t,
    pub walk_msg: *mut ::core::ffi::c_char,
    pub run_msg: *mut ::core::ffi::c_char,
    pub hurt_msg: *mut ::core::ffi::c_char,
    pub die_msg: *mut ::core::ffi::c_char,
    pub confused_msg: *mut ::core::ffi::c_char,
    pub look_prefix: *mut ::core::ffi::c_char,
    pub look_in_preposition: *mut ::core::ffi::c_char,
    pub resist_flag: ::core::ffi::c_int,
}
extern "C" {
    pub static mut f_info: *mut feature;
}
extern "C" {
    pub fn square(c: *mut chunk, grid: loc) -> *const square;
}
extern "C" {
    pub fn square_isno_teleport(c: *mut chunk, grid: loc) -> bool;
}
extern "C" {
    pub fn feat_is_trap_holding(feat: ::core::ffi::c_int) -> bool;
}
extern "C" {
    pub fn square_monster(c: *mut chunk, grid: loc) -> *mut monster;
}
extern "C" {
    pub fn square_object(c: *mut chunk, grid: loc) -> *mut object;
}
extern "C" {
    pub fn no_light(p: *const player) -> bool;
}
extern "C" {
    pub fn square_isglow(c: *mut chunk, grid: loc) -> bool;
}
extern "C" {
    pub fn square_islit(c: *mut chunk, grid: loc) -> bool;
}
extern "C" {
    pub fn square_iswarded(c: *mut chunk, grid: loc) -> bool;
}
extern "C" {
    pub fn square_shopnum(c: *mut chunk, grid: loc) -> ::core::ffi::c_int;
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct grid_data {
    pub m_idx: uint32_t,
    pub f_idx: uint32_t,
    pub first_kind: *mut object_kind,
    pub trap: *mut trap,
    pub multiple_objects: bool,
    pub unseen_object: bool,
    pub unseen_money: bool,
    pub lighting: grid_light_level,
    pub in_view: bool,
    pub is_player: bool,
    pub hallucinate: bool,
}
pub type grid_light_level = ::core::ffi::c_uint;
pub const LIGHTING_MAX: grid_light_level = 4;
pub const LIGHTING_DARK: grid_light_level = 3;
pub const LIGHTING_LIT: grid_light_level = 2;
pub const LIGHTING_TORCH: grid_light_level = 1;
pub const LIGHTING_LOS: grid_light_level = 0;
extern "C" {
    pub fn map_info(grid: loc, g: *mut grid_data);
}
extern "C" {
    pub fn square_iswebbed(c: *mut chunk, grid: loc) -> bool;
}
extern "C" {
    pub fn square_isdisarmabletrap(c: *mut chunk, grid: loc) -> bool;
}
extern "C" {
    pub fn cave_monster(c: *mut chunk, idx: ::core::ffi::c_int) -> *mut monster;
}
