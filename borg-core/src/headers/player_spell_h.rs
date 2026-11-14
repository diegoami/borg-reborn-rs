//! `/home/user/borg-rs/angband/src/player-spell.h`
pub use super::player_h::class_spell;
pub use super::player_h::player;
pub use super::stdint_intn_h::int16_t;
#[allow(unused_imports)]
use super::*;
extern "C" {
    pub fn spell_chance(spell_index: ::core::ffi::c_int) -> int16_t;
}
extern "C" {
    pub fn spell_by_index(p: *const player, index: ::core::ffi::c_int) -> *const class_spell;
}
extern "C" {
    pub fn spell_okay_to_cast(p: *const player, spell_index: ::core::ffi::c_int) -> bool;
}
