//! `/home/user/borg-rs/angband/src/generate.h`
pub use super::stdint_uintn_h::uint8_t;
#[allow(unused_imports)]
use super::*;
extern "C" {
    pub static mut get_angle_to_grid: [[uint8_t; 41]; 41];
}
