//! `/home/user/borg-rs/angband/src/z-bitflag.h`
pub use super::__stddef_size_t_h::size_t;
pub use super::stdint_uintn_h::uint8_t;
#[allow(unused_imports)]
use super::*;
pub type bitflag = uint8_t;
extern "C" {
    pub fn flag_has(flags: *const bitflag, size: size_t, flag: ::core::ffi::c_int) -> bool;
}
extern "C" {
    pub fn flag_next(
        flags: *const bitflag,
        size: size_t,
        flag: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
}
extern "C" {
    pub fn flag_is_empty(flags: *const bitflag, size: size_t) -> bool;
}
extern "C" {
    pub fn flag_is_equal(flags1: *const bitflag, flags2: *const bitflag, size: size_t) -> bool;
}
