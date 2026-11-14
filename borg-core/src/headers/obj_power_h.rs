//! `/home/user/borg-rs/angband/src/obj-power.h`
pub use super::object_h::object;
#[allow(unused_imports)]
use super::*;
extern "C" {
    pub fn object_value_real(obj: *const object, qty: ::core::ffi::c_int) -> ::core::ffi::c_int;
}
extern "C" {
    pub fn object_value(obj: *const object, qty: ::core::ffi::c_int) -> ::core::ffi::c_int;
}
