//! `/home/user/borg-rs/angband/src/z-util.h`
pub use super::__stddef_size_t_h::size_t;
pub use super::__stddef_wchar_t_h::wchar_t;
#[allow(unused_imports)]
use super::*;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct my_rational {
    pub n: ::core::ffi::c_uint,
    pub d: ::core::ffi::c_uint,
}
extern "C" {
    pub fn my_strcpy(
        buf: *mut ::core::ffi::c_char,
        src: *const ::core::ffi::c_char,
        bufsize: size_t,
    ) -> size_t;
}
extern "C" {
    pub fn my_strnicmp(
        a: *const ::core::ffi::c_char,
        b: *const ::core::ffi::c_char,
        n: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
}
extern "C" {
    pub fn my_stristr(
        string: *const ::core::ffi::c_char,
        pattern: *const ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_char;
}
extern "C" {
    pub fn prefix(s: *const ::core::ffi::c_char, t: *const ::core::ffi::c_char) -> bool;
}
extern "C" {
    pub fn prefix_i(s: *const ::core::ffi::c_char, t: *const ::core::ffi::c_char) -> bool;
}
extern "C" {
    pub fn suffix(s: *const ::core::ffi::c_char, t: *const ::core::ffi::c_char) -> bool;
}
extern "C" {
    pub fn my_stricmp(
        s1: *const ::core::ffi::c_char,
        s2: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
}
extern "C" {
    pub fn text_mbstowcs(
        dest: *mut wchar_t,
        src: *const ::core::ffi::c_char,
        n: ::core::ffi::c_int,
    ) -> size_t;
}
extern "C" {
    pub fn my_strcat(
        buf: *mut ::core::ffi::c_char,
        src: *const ::core::ffi::c_char,
        bufsize: size_t,
    ) -> size_t;
}
