//! `/home/user/borg-rs/angband/src/obj-slays.h`
pub use super::object_h::brand;
pub use super::object_h::slay;
pub use super::player_h::player;
#[allow(unused_imports)]
use super::*;
extern "C" {
    pub static mut slays: *mut slay;
}
extern "C" {
    pub static mut brands: *mut brand;
}
extern "C" {
    pub fn player_has_temporary_brand(p: *const player, idx: ::core::ffi::c_int) -> bool;
}
extern "C" {
    pub fn player_has_temporary_slay(p: *const player, idx: ::core::ffi::c_int) -> bool;
}
