//! `/home/user/borg-rs/angband/src/obj-knowledge.h`
pub use super::object_h::object;
#[allow(unused_imports)]
use super::*;
extern "C" {
    pub fn object_fully_known(obj: *const object) -> bool;
}
extern "C" {
    pub fn object_runes_known(obj: *const object) -> bool;
}
extern "C" {
    pub fn object_flavor_is_aware(obj: *const object) -> bool;
}
