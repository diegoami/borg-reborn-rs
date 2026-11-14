//! `/home/user/borg-rs/angband/src/obj-tval.h`
pub use super::object_h::object;
pub use super::object_h::object_kind;
#[allow(unused_imports)]
use super::*;
extern "C" {
    pub fn tval_is_book_k(kind: *const object_kind) -> bool;
}
extern "C" {
    pub fn tval_find_idx(name: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
}
extern "C" {
    pub fn tval_find_name(tval: ::core::ffi::c_int) -> *const ::core::ffi::c_char;
}
extern "C" {
    pub fn tval_can_have_charges(obj: *const object) -> bool;
}
