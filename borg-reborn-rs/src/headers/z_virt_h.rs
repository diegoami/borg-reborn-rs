//! `/home/user/borg-rs/angband/src/z-virt.h`
pub use super::__stddef_size_t_h::size_t;
#[allow(unused_imports)]
use super::*;
extern "C" {
    pub fn mem_zalloc(len: size_t) -> *mut ::core::ffi::c_void;
}
extern "C" {
    pub fn mem_free(p: *mut ::core::ffi::c_void);
}
extern "C" {
    pub fn string_free(str: *mut ::core::ffi::c_char);
}
extern "C" {
    pub fn mem_alloc(len: size_t) -> *mut ::core::ffi::c_void;
}
extern "C" {
    pub fn mem_realloc(p: *mut ::core::ffi::c_void, len: size_t) -> *mut ::core::ffi::c_void;
}
extern "C" {
    pub fn string_make(str: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
}
