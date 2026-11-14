//! `/usr/include/stdlib.h`
pub use super::__stddef_size_t_h::size_t;
pub use super::__stddef_wchar_t_h::wchar_t;
#[allow(unused_imports)]
use super::*;
extern "C" {
    pub fn atoi(__nptr: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
}
extern "C" {
    pub fn atol(__nptr: *const ::core::ffi::c_char) -> ::core::ffi::c_long;
}
extern "C" {
    pub fn strtol(
        __nptr: *const ::core::ffi::c_char,
        __endptr: *mut *mut ::core::ffi::c_char,
        __base: ::core::ffi::c_int,
    ) -> ::core::ffi::c_long;
}
extern "C" {
    pub fn wcstombs(__s: *mut ::core::ffi::c_char, __pwcs: *const wchar_t, __n: size_t) -> size_t;
}
pub type __compar_fn_t = Option<
    unsafe extern "C" fn(
        *const ::core::ffi::c_void,
        *const ::core::ffi::c_void,
    ) -> ::core::ffi::c_int,
>;
extern "C" {
    pub fn qsort(
        __base: *mut ::core::ffi::c_void,
        __nmemb: size_t,
        __size: size_t,
        __compar: __compar_fn_t,
    );
}
