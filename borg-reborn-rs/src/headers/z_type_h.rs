//! `/home/user/borg-rs/angband/src/z-type.h`
#[allow(unused_imports)]
use super::*;
#[derive(Copy, Clone, Default, Debug, PartialEq, Eq)]
#[repr(C)]
pub struct loc {
    pub x: ::core::ffi::c_int,
    pub y: ::core::ffi::c_int,
}
extern "C" {
    pub fn loc(x: ::core::ffi::c_int, y: ::core::ffi::c_int) -> loc;
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct grouper {
    pub tval: ::core::ffi::c_int,
    pub name: *const ::core::ffi::c_char,
}
