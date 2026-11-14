//! `/usr/include/wchar.h`
pub use super::__stddef_wchar_t_h::wchar_t;
#[allow(unused_imports)]
use super::*;
extern "C" {
    pub fn wcslen(__s: *const wchar_t) -> ::core::ffi::c_ulong;
}
