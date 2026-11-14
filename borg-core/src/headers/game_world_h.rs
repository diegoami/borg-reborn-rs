//! `/home/user/borg-rs/angband/src/game-world.h`
pub use super::stdint_intn_h::int32_t;
pub use super::stdint_uintn_h::uint8_t;
#[allow(unused_imports)]
use super::*;
extern "C" {
    pub static extract_energy: [uint8_t; 200];
}
extern "C" {
    pub static mut turn: int32_t;
}
