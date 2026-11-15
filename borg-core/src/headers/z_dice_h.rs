//! `/home/user/borg-rs/angband/src/z-dice.h`
#[allow(unused_imports)]
use super::*;
pub type dice_t = dice_s;
#[repr(C)]
pub struct dice_s {
    _opaque: [u8; 0],
}

extern "C" {
    pub fn dice_evaluate(dice: *const dice_t, level: i32, aspect: super::z_rand_h::aspect,
                         value: *mut super::z_rand_h::random_value) -> i32;
}
