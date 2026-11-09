//! `/home/user/borg-rs/angband/src/message.h`
pub use super::stdint_uintn_h::uint16_t;
#[allow(unused_imports)]
use super::*;
extern "C" {
    pub fn msg(fmt: *const ::core::ffi::c_char, ...);
}
pub type C2RustUnnamedH_43 = ::core::ffi::c_uint;
pub const SOUND_MAX: C2RustUnnamedH_43 = 153;
extern "C" {
    pub fn message_add(str: *const ::core::ffi::c_char, type_0: uint16_t);
}
extern "C" {
    pub fn messages_num() -> uint16_t;
}
extern "C" {
    pub fn message_str(age: uint16_t) -> *const ::core::ffi::c_char;
}
