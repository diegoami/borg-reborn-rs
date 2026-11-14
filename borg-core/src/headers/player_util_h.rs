//! `/home/user/borg-rs/angband/src/player-util.h`
pub use super::player_h::player;
pub use super::stdint_intn_h::int16_t;
#[allow(unused_imports)]
use super::*;
extern "C" {
    pub fn player_of_has(p: *const player, flag: ::core::ffi::c_int) -> bool;
}
extern "C" {
    pub fn player_is_shapechanged(p: *const player) -> bool;
}
extern "C" {
    pub fn modify_stat_value(value: ::core::ffi::c_int, amount: ::core::ffi::c_int) -> int16_t;
}
