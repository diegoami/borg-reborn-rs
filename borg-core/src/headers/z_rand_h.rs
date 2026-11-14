//! `/home/user/borg-rs/angband/src/z-rand.h`
pub use super::stdint_uintn_h::uint32_t;
#[allow(unused_imports)]
use super::*;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct random {
    pub base: ::core::ffi::c_int,
    pub dice: ::core::ffi::c_int,
    pub sides: ::core::ffi::c_int,
    pub m_bonus: ::core::ffi::c_int,
}
pub type random_value = random;
extern "C" {
    pub static mut Rand_quick: bool;
}
extern "C" {
    pub static mut Rand_value: uint32_t;
}
extern "C" {
    pub fn Rand_div(m: uint32_t) -> uint32_t;
}
pub type aspect = ::core::ffi::c_uint;
pub const RANDOMISE: aspect = 4;
pub const EXTREMIFY: aspect = 3;
pub const MAXIMISE: aspect = 2;
pub const AVERAGE: aspect = 1;
pub const MINIMISE: aspect = 0;
extern "C" {
    pub fn randcalc(
        v: random_value,
        level: ::core::ffi::c_int,
        rand_aspect: aspect,
    ) -> ::core::ffi::c_int;
}
