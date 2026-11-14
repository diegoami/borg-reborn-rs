//! `/home/user/borg-rs/angband/src/ui-target.h`
#[allow(unused_imports)]
use super::*;
extern "C" {
    pub fn target_set_interactive(
        mode: ::core::ffi::c_int,
        x: ::core::ffi::c_int,
        y: ::core::ffi::c_int,
        allow_pathfinding: bool,
    ) -> bool;
}
