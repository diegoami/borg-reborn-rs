//! `/home/user/borg-rs/angband/src/effect-handler.h`
pub use super::cmd_core_h::command;
pub use super::effects_h::effect_index;
pub use super::object_h::object;
pub use super::source_h::source;
pub use super::stdint_uintn_h::uint16_t;
pub use super::z_rand_h::random_value;
#[allow(unused_imports)]
use super::*;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct effect_kind {
    pub index: uint16_t,
    pub aim: bool,
    pub info: *const ::core::ffi::c_char,
    pub handler: effect_handler_f,
    pub desc: *const ::core::ffi::c_char,
    pub menu_name: *const ::core::ffi::c_char,
}
pub type effect_handler_f = Option<unsafe extern "C" fn(*mut effect_handler_context_t) -> bool>;
pub type effect_handler_context_t = effect_handler_context_s;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct effect_handler_context_s {
    pub effect: effect_index,
    pub origin: source,
    pub obj: *const object,
    pub aware: bool,
    pub dir: ::core::ffi::c_int,
    pub beam: ::core::ffi::c_int,
    pub boost: ::core::ffi::c_int,
    pub value: random_value,
    pub subtype: ::core::ffi::c_int,
    pub radius: ::core::ffi::c_int,
    pub other: ::core::ffi::c_int,
    pub y: ::core::ffi::c_int,
    pub x: ::core::ffi::c_int,
    pub msg: *const ::core::ffi::c_char,
    pub ident: bool,
    pub cmd: *mut command,
}
