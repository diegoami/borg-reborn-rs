//! `/home/user/borg-rs/angband/src/mon-blows.h`
pub use super::stdint_uintn_h::uint8_t;
#[allow(unused_imports)]
use super::*;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct blow_effect {
    pub name: *mut ::core::ffi::c_char,
    pub power: ::core::ffi::c_int,
    pub eval: ::core::ffi::c_int,
    pub desc: *mut ::core::ffi::c_char,
    pub lore_attr: uint8_t,
    pub lore_attr_resist: uint8_t,
    pub lore_attr_immune: uint8_t,
    pub effect_type: *mut ::core::ffi::c_char,
    pub resist: ::core::ffi::c_int,
    pub lash_type: ::core::ffi::c_int,
    pub next: *mut blow_effect,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct blow_method {
    pub name: *mut ::core::ffi::c_char,
    pub cut: bool,
    pub stun: bool,
    pub miss: bool,
    pub phys: bool,
    pub msgt: ::core::ffi::c_int,
    pub messages: *mut blow_message,
    pub num_messages: ::core::ffi::c_int,
    pub desc: *mut ::core::ffi::c_char,
    pub next: *mut blow_method,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct blow_message {
    pub act_msg: *mut ::core::ffi::c_char,
    pub next: *mut blow_message,
}
extern "C" {
    pub static mut blow_methods: *mut blow_method;
}
