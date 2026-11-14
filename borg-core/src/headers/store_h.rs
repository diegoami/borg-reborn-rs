//! `/home/user/borg-rs/angband/src/store.h`
pub use super::__stddef_size_t_h::size_t;
pub use super::object_h::object;
pub use super::object_h::object_kind;
pub use super::stdint_intn_h::int16_t;
pub use super::stdint_intn_h::int32_t;
pub use super::stdint_uintn_h::uint8_t;
#[allow(unused_imports)]
use super::*;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct store {
    pub owners: *mut owner,
    pub owner: *mut owner,
    pub feat: ::core::ffi::c_int,
    pub stock_num: uint8_t,
    pub stock_size: int16_t,
    pub stock: *mut object,
    pub stock_k: *mut object,
    pub always_size: size_t,
    pub always_num: size_t,
    pub always_table: *mut *mut object_kind,
    pub normal_size: size_t,
    pub normal_num: size_t,
    pub normal_table: *mut *mut object_kind,
    pub buy: *mut object_buy,
    pub turnover: ::core::ffi::c_int,
    pub normal_stock_min: ::core::ffi::c_int,
    pub normal_stock_max: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct object_buy {
    pub next: *mut object_buy,
    pub tval: size_t,
    pub flag: size_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct owner {
    pub oidx: ::core::ffi::c_uint,
    pub next: *mut owner,
    pub name: *mut ::core::ffi::c_char,
    pub max_cost: int32_t,
}
extern "C" {
    pub static mut stores: *mut store;
}
extern "C" {
    pub fn store_stock_list(store: *mut store, list: *mut *mut object, n: ::core::ffi::c_int);
}
