//! `/usr/include/ctype.h`
#[allow(unused_imports)]
use super::*;
pub type C2RustUnnamedH_37 = ::core::ffi::c_uint;
pub const _ISalnum: C2RustUnnamedH_37 = 8;
pub const _ISpunct: C2RustUnnamedH_37 = 4;
pub const _IScntrl: C2RustUnnamedH_37 = 2;
pub const _ISblank: C2RustUnnamedH_37 = 1;
pub const _ISgraph: C2RustUnnamedH_37 = 32768;
pub const _ISprint: C2RustUnnamedH_37 = 16384;
pub const _ISspace: C2RustUnnamedH_37 = 8192;
pub const _ISxdigit: C2RustUnnamedH_37 = 4096;
pub const _ISdigit: C2RustUnnamedH_37 = 2048;
pub const _ISalpha: C2RustUnnamedH_37 = 1024;
pub const _ISlower: C2RustUnnamedH_37 = 512;
pub const _ISupper: C2RustUnnamedH_37 = 256;
extern "C" {
    pub fn __ctype_b_loc() -> *mut *const ::core::ffi::c_ushort;
}
