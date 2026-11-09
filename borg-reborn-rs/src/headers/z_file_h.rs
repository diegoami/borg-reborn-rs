//! `/home/user/borg-rs/angband/src/z-file.h`
pub use super::__stddef_size_t_h::size_t;
#[allow(unused_imports)]
use super::*;
#[repr(C)]
pub struct ang_file {
    _opaque: [u8; 0],
}
extern "C" {
    pub fn file_getl(f: *mut ang_file, buf: *mut ::core::ffi::c_char, n: size_t) -> bool;
}
pub type file_mode = ::core::ffi::c_uint;
pub const MODE_APPEND: file_mode = 2;
pub const MODE_READ: file_mode = 1;
pub const MODE_WRITE: file_mode = 0;
pub type file_type = ::core::ffi::c_uint;
pub const FTYPE_HTML: file_type = 4;
pub const FTYPE_RAW: file_type = 3;
pub const FTYPE_SAVE: file_type = 2;
pub const FTYPE_TEXT: file_type = 1;
extern "C" {
    pub fn path_build(
        buf: *mut ::core::ffi::c_char,
        len: size_t,
        base: *const ::core::ffi::c_char,
        leaf: *const ::core::ffi::c_char,
    ) -> size_t;
}
extern "C" {
    pub fn file_open(
        buf: *const ::core::ffi::c_char,
        mode: file_mode,
        ftype: file_type,
    ) -> *mut ang_file;
}
extern "C" {
    pub fn file_close(f: *mut ang_file) -> bool;
}
extern "C" {
    pub fn file_putf(f: *mut ang_file, fmt: *const ::core::ffi::c_char, ...) -> bool;
}
extern "C" {
    pub fn file_put(f: *mut ang_file, buf: *const ::core::ffi::c_char) -> bool;
}
