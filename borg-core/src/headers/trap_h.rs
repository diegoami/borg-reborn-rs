//! `/home/user/borg-rs/angband/src/trap.h`
pub use super::__stddef_wchar_t_h::wchar_t;
pub use super::object_h::effect;
pub use super::stdint_uintn_h::uint8_t;
pub use super::z_bitflag_h::bitflag;
pub use super::z_rand_h::random_value;
pub use super::z_type_h::loc;
#[allow(unused_imports)]
use super::*;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct trap {
    pub t_idx: uint8_t,
    pub kind: *mut trap_kind,
    pub next: *mut trap,
    pub grid: loc,
    pub power: uint8_t,
    pub timeout: uint8_t,
    pub flags: [bitflag; 2],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct trap_kind {
    pub name: *mut ::core::ffi::c_char,
    pub text: *mut ::core::ffi::c_char,
    pub desc: *mut ::core::ffi::c_char,
    pub msg: *mut ::core::ffi::c_char,
    pub msg_good: *mut ::core::ffi::c_char,
    pub msg_bad: *mut ::core::ffi::c_char,
    pub msg_xtra: *mut ::core::ffi::c_char,
    pub next: *mut trap_kind,
    pub tidx: ::core::ffi::c_int,
    pub d_attr: uint8_t,
    pub d_char: wchar_t,
    pub rarity: ::core::ffi::c_int,
    pub min_depth: ::core::ffi::c_int,
    pub max_num: ::core::ffi::c_int,
    pub power: random_value,
    pub flags: [bitflag; 2],
    pub save_flags: [bitflag; 5],
    pub effect: *mut effect,
    pub effect_xtra: *mut effect,
}
pub const TRF_MAX: C2RustUnnamedH_48 = 15;
pub type C2RustUnnamedH_48 = ::core::ffi::c_uint;
extern "C" {
    pub static mut trap_info: *mut trap_kind;
}
