//! `/home/user/borg-rs/angband/src/z-quark.h`
pub use super::__stddef_size_t_h::size_t;
#[allow(unused_imports)]
use super::*;
pub type quark_t = size_t;
extern "C" {
    pub fn quark_str(q: quark_t) -> *const ::core::ffi::c_char;
}
