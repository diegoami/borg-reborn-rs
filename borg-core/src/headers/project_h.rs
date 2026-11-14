//! `/home/user/borg-rs/angband/src/project.h`
pub use super::object_h::object;
pub use super::source_h::source;
pub use super::source_h::C2RustUnnamedH_25;
pub use super::source_h::C2RustUnnamedH_26;
pub use super::source_h::SRC_NONE;
pub use super::stdint_uintn_h::uint8_t;
pub use super::trap_h::trap;
pub use super::z_type_h::loc;
#[allow(unused_imports)]
use super::*;
pub type C2RustUnnamedH_27 = ::core::ffi::c_uint;
pub const PROJ_MAX: C2RustUnnamedH_27 = 56;
pub type C2RustUnnamedH_28 = ::core::ffi::c_uint;
pub const PROJECT_ROCK: C2RustUnnamedH_28 = 32768;
pub const PROJECT_SELF: C2RustUnnamedH_28 = 16384;
pub const PROJECT_SHORT: C2RustUnnamedH_28 = 8192;
pub const PROJECT_INFO: C2RustUnnamedH_28 = 4096;
pub const PROJECT_PLAY: C2RustUnnamedH_28 = 2048;
pub const PROJECT_ARC: C2RustUnnamedH_28 = 1024;
pub const PROJECT_SAFE: C2RustUnnamedH_28 = 512;
pub const PROJECT_AWARE: C2RustUnnamedH_28 = 256;
pub const PROJECT_HIDE: C2RustUnnamedH_28 = 128;
pub const PROJECT_KILL: C2RustUnnamedH_28 = 64;
pub const PROJECT_ITEM: C2RustUnnamedH_28 = 32;
pub const PROJECT_GRID: C2RustUnnamedH_28 = 16;
pub const PROJECT_STOP: C2RustUnnamedH_28 = 8;
pub const PROJECT_THRU: C2RustUnnamedH_28 = 4;
pub const PROJECT_BEAM: C2RustUnnamedH_28 = 2;
pub const PROJECT_JUMP: C2RustUnnamedH_28 = 1;
pub const PROJECT_NONE: C2RustUnnamedH_28 = 0;
extern "C" {
    pub fn project(
        origin: source,
        rad: ::core::ffi::c_int,
        finish: loc,
        dam: ::core::ffi::c_int,
        typ: ::core::ffi::c_int,
        flg: ::core::ffi::c_int,
        degrees_of_arc: ::core::ffi::c_int,
        diameter_of_source: uint8_t,
        obj: *const object,
    ) -> bool;
}
