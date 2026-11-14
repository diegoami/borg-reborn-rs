//! `/home/user/borg-rs/angband/src/target.h`
pub use super::z_type_h::loc;
#[allow(unused_imports)]
use super::*;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct target {
    pub grid: loc,
    pub midx: ::core::ffi::c_int,
}
extern "C" {
    pub fn target_okay() -> bool;
}
extern "C" {
    pub fn target_get(grid: *mut loc);
}
