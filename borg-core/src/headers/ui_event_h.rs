//! `/home/user/borg-rs/angband/src/ui-event.h`
pub use super::stdint_uintn_h::uint32_t;
pub use super::stdint_uintn_h::uint8_t;
#[allow(unused_imports)]
use super::*;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct keypress {
    pub type_0: ui_event_type,
    pub code: keycode_t,
    pub mods: uint8_t,
}
pub type keycode_t = uint32_t;
pub type ui_event_type = ::core::ffi::c_uint;
pub const EVT_DISCONNECT: ui_event_type = 256;
pub const EVT_SWITCH: ui_event_type = 128;
pub const EVT_SELECT: ui_event_type = 64;
pub const EVT_MOVE: ui_event_type = 32;
pub const EVT_ESCAPE: ui_event_type = 16;
pub const EVT_BUTTON: ui_event_type = 8;
pub const EVT_RESIZE: ui_event_type = 4;
pub const EVT_MOUSE: ui_event_type = 2;
pub const EVT_KBRD: ui_event_type = 1;
pub const EVT_NONE: ui_event_type = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub union ui_event {
    pub type_0: ui_event_type,
    pub mouse: mouseclick,
    pub key: keypress,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct mouseclick {
    pub type_0: ui_event_type,
    pub x: uint8_t,
    pub y: uint8_t,
    pub button: uint8_t,
    pub mods: uint8_t,
}
pub static mut EVENT_EMPTY: ui_event = ui_event { type_0: EVT_NONE };
