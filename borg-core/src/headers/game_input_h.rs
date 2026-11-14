//! `/home/user/borg-rs/angband/src/game-input.h`
pub use super::__stddef_size_t_h::size_t;
#[allow(unused_imports)]
use super::*;
extern "C" {
    pub fn get_string(
        prompt: *const ::core::ffi::c_char,
        buf: *mut ::core::ffi::c_char,
        len: size_t,
    ) -> bool;
}
extern "C" {
    pub fn get_quantity(
        prompt: *const ::core::ffi::c_char,
        max: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
}
extern "C" {
    pub fn get_com(prompt: *const ::core::ffi::c_char, command: *mut ::core::ffi::c_char) -> bool;
}
extern "C" {
    pub fn panel_contains(y: ::core::ffi::c_uint, x: ::core::ffi::c_uint) -> bool;
}
