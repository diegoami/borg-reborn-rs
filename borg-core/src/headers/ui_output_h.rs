//! `/home/user/borg-rs/angband/src/ui-output.h`
pub use super::stdint_uintn_h::uint8_t;
#[allow(unused_imports)]
use super::*;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct region {
    pub col: ::core::ffi::c_int,
    pub row: ::core::ffi::c_int,
    pub width: ::core::ffi::c_int,
    pub page_rows: ::core::ffi::c_int,
}
extern "C" {
    pub fn c_put_str(
        attr: uint8_t,
        str: *const ::core::ffi::c_char,
        row: ::core::ffi::c_int,
        col: ::core::ffi::c_int,
    );
}
extern "C" {
    pub fn screen_save();
}
extern "C" {
    pub fn screen_load();
}
extern "C" {
    pub fn window_make(
        origin_x: ::core::ffi::c_int,
        origin_y: ::core::ffi::c_int,
        end_x: ::core::ffi::c_int,
        end_y: ::core::ffi::c_int,
    );
}
extern "C" {
    pub fn prt(str: *const ::core::ffi::c_char, row: ::core::ffi::c_int, col: ::core::ffi::c_int);
}
