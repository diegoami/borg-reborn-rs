//! `angband/src/borg/borg-item-use.h`
pub const BORG_UNMET_NEED: borg_need = 2;
pub type borg_need = ::core::ffi::c_uint;
pub const BORG_MET_NEED: borg_need = 1;
pub const BORG_NO_NEED: borg_need = 0;
