//! `/usr/include/time.h`
pub use super::__stddef_size_t_h::size_t;
pub use super::struct_tm_h::tm;
pub use super::time_t_h::time_t;
#[allow(unused_imports)]
use super::*;
extern "C" {
    pub fn time(__timer: *mut time_t) -> time_t;
}
extern "C" {
    pub fn strftime(
        __s: *mut ::core::ffi::c_char,
        __maxsize: size_t,
        __format: *const ::core::ffi::c_char,
        __tp: *const tm,
    ) -> size_t;
}
extern "C" {
    pub fn localtime(__timer: *const time_t) -> *mut tm;
}
