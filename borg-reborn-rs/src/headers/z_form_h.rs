//! `/home/user/borg-rs/angband/src/z-form.h`
pub use super::__stddef_size_t_h::size_t;
#[allow(unused_imports)]
use super::*;
extern "C" {
    pub fn format(fmt: *const ::core::ffi::c_char, ...) -> *mut ::core::ffi::c_char;
}
extern "C" {
    pub fn strnfmt(
        buf: *mut ::core::ffi::c_char,
        max: size_t,
        fmt: *const ::core::ffi::c_char,
        ...
    ) -> size_t;
}
extern "C" {
    pub fn quit_fmt(fmt: *const ::core::ffi::c_char, ...);
}
