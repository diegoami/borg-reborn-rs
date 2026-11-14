//! `/home/user/borg-rs/angband/src/source.h`
pub use super::object_h::chest_trap;
pub use super::object_h::object;
pub use super::trap_h::trap;
#[allow(unused_imports)]
use super::*;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct source {
    pub what: C2RustUnnamedH_26,
    pub which: C2RustUnnamedH_25,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2RustUnnamedH_25 {
    pub trap: *mut trap,
    pub monster: ::core::ffi::c_int,
    pub object: *mut object,
    pub chest_trap: *mut chest_trap,
}
pub type C2RustUnnamedH_26 = ::core::ffi::c_uint;
pub const SRC_CHEST_TRAP: C2RustUnnamedH_26 = 5;
pub const SRC_OBJECT: C2RustUnnamedH_26 = 4;
pub const SRC_MONSTER: C2RustUnnamedH_26 = 3;
pub const SRC_PLAYER: C2RustUnnamedH_26 = 2;
pub const SRC_TRAP: C2RustUnnamedH_26 = 1;
pub const SRC_NONE: C2RustUnnamedH_26 = 0;
extern "C" {
    pub fn source_player() -> source;
}
