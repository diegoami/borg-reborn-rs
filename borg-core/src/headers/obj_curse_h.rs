//! `/home/user/borg-rs/angband/src/obj-curse.h`
pub use super::object_h::curse;
#[allow(unused_imports)]
use super::*;
extern "C" {
    pub static mut curses: *mut curse;
}
