//! `/home/user/borg-rs/angband/src/obj-desc.h`
pub use super::__stddef_size_t_h::size_t;
pub use super::object_h::object;
pub use super::player_h::player;
pub use super::stdint_uintn_h::uint32_t;
#[allow(unused_imports)]
use super::*;
pub const ODESC_FULL: C2RustUnnamedH_42 = 3;
pub type C2RustUnnamedH_42 = ::core::ffi::c_uint;
pub const ODESC_ALTNUM: C2RustUnnamedH_42 = 1024;
pub const ODESC_NOEGO: C2RustUnnamedH_42 = 512;
pub const ODESC_TERSE: C2RustUnnamedH_42 = 256;
pub const ODESC_CAPITAL: C2RustUnnamedH_42 = 128;
pub const ODESC_PREFIX: C2RustUnnamedH_42 = 64;
pub const ODESC_SPOIL: C2RustUnnamedH_42 = 32;
pub const ODESC_SINGULAR: C2RustUnnamedH_42 = 16;
pub const ODESC_PLURAL: C2RustUnnamedH_42 = 8;
pub const ODESC_STORE: C2RustUnnamedH_42 = 4;
pub const ODESC_EXTRA: C2RustUnnamedH_42 = 2;
pub const ODESC_COMBAT: C2RustUnnamedH_42 = 1;
pub const ODESC_BASE: C2RustUnnamedH_42 = 0;
extern "C" {
    pub fn object_desc(
        buf: *mut ::core::ffi::c_char,
        max: size_t,
        obj: *const object,
        mode: uint32_t,
        p: *const player,
    ) -> size_t;
}
