//! `/usr/include/stdio.h`
pub use super::__stddef_size_t_h::size_t;
#[allow(unused_imports)]
use super::*;
extern "C" {
    pub fn snprintf(
        __s: *mut ::core::ffi::c_char,
        __maxlen: size_t,
        __format: *const ::core::ffi::c_char,
        ...
    ) -> ::core::ffi::c_int;
}
