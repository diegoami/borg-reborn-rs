//! `/home/user/borg-rs/angband/src/obj-properties.h`
#[allow(unused_imports)]
use super::*;
pub type C2RustUnnamedH_20 = ::core::ffi::c_uint;
pub const TV_MAX: C2RustUnnamedH_20 = 36;
pub type C2RustUnnamedH_31 = ::core::ffi::c_uint;
pub const OF_NONE: C2RustUnnamedH_31 = 0;
pub type C2RustUnnamedH_36 = ::core::ffi::c_uint;
pub type C2RustUnnamedH_39 = ::core::ffi::c_uint;
pub const OBJ_MOD_MAX: C2RustUnnamedH_39 = 16;
pub type obj_property_type = ::core::ffi::c_uint;
pub const OBJ_PROPERTY_MAX: obj_property_type = 8;
pub const OBJ_PROPERTY_IMM: obj_property_type = 7;
pub const OBJ_PROPERTY_VULN: obj_property_type = 6;
pub const OBJ_PROPERTY_RESIST: obj_property_type = 5;
pub const OBJ_PROPERTY_IGNORE: obj_property_type = 4;
pub const OBJ_PROPERTY_FLAG: obj_property_type = 3;
pub const OBJ_PROPERTY_MOD: obj_property_type = 2;
pub const OBJ_PROPERTY_STAT: obj_property_type = 1;
pub const OBJ_PROPERTY_NONE: obj_property_type = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct obj_property {
    pub next: *mut obj_property,
    pub type_0: ::core::ffi::c_int,
    pub subtype: ::core::ffi::c_int,
    pub id_type: ::core::ffi::c_int,
    pub index: ::core::ffi::c_int,
    pub power: ::core::ffi::c_int,
    pub mult: ::core::ffi::c_int,
    pub type_mult: [::core::ffi::c_int; 36],
    pub name: *mut ::core::ffi::c_char,
    pub adjective: *mut ::core::ffi::c_char,
    pub neg_adj: *mut ::core::ffi::c_char,
    pub msg: *mut ::core::ffi::c_char,
    pub desc: *mut ::core::ffi::c_char,
}
extern "C" {
    pub fn lookup_obj_property(
        type_0: ::core::ffi::c_int,
        index: ::core::ffi::c_int,
    ) -> *mut obj_property;
}
