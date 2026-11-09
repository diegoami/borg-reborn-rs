//! `/home/user/borg-rs/angband/src/obj-util.h`
pub use super::object_h::object;
pub use super::object_h::object_kind;
pub use super::stdint_intn_h::int16_t;
pub use super::z_bitflag_h::bitflag;
#[allow(unused_imports)]
use super::*;
extern "C" {
    pub fn obj_kind_can_browse(kind: *const object_kind) -> bool;
}
extern "C" {
    pub fn lookup_sval(
        tval: ::core::ffi::c_int,
        name: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
}
extern "C" {
    pub fn object_weight_one(obj: *const object) -> int16_t;
}
extern "C" {
    pub fn object_flags_known(obj: *const object, flags: *mut bitflag);
}
extern "C" {
    pub fn object_flags(obj: *const object, flags: *mut bitflag);
}
