//! `/home/user/borg-rs/angband/src/option.h`
pub use super::stdint_uintn_h::uint8_t;
#[allow(unused_imports)]
use super::*;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct player_options {
    pub opt: [bool; 46],
    pub hitpoint_warn: uint8_t,
    pub lazymove_delay: uint8_t,
    pub delay_factor: uint8_t,
    pub name_suffix: uint8_t,
}
pub type C2RustUnnamedH_21 = ::core::ffi::c_uint;
pub const OPT_MAX: C2RustUnnamedH_21 = 46;
extern "C" {
    pub static mut screensaver: bool;
}
extern "C" {
    pub fn option_set(opt: *const ::core::ffi::c_char, val: ::core::ffi::c_int) -> bool;
}
