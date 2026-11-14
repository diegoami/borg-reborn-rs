//! `/home/user/borg-rs/angband/src/ui-map.h`
pub use super::__stddef_wchar_t_h::wchar_t;
pub use super::stdint_uintn_h::uint8_t;
#[allow(unused_imports)]
use super::*;
extern "C" {
    pub fn print_rel(c: wchar_t, a: uint8_t, y: ::core::ffi::c_int, x: ::core::ffi::c_int);
}
extern "C" {
    pub fn prt_map();
}
