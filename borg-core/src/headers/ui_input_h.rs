//! `/home/user/borg-rs/angband/src/ui-input.h`
pub use super::cmd_core_h::cmd_code;
pub use super::game_event_h::game_event_data;
pub use super::game_event_h::game_event_type;
pub use super::game_event_h::EVENT_MAP;
pub use super::ui_event_h::keycode_t;
pub use super::ui_event_h::ui_event;
#[allow(unused_imports)]
use super::*;
extern "C" {
    /// The Borg's key hook (ui-input.c; declared in borg.h).
    pub static mut inkey_hack: Option<unsafe extern "C" fn(::core::ffi::c_int) -> super::ui_event_h::keypress>;
    pub fn flush(
        unused: game_event_type,
        data: *mut game_event_data,
        user: *mut ::core::ffi::c_void,
    );
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct cmd_info {
    pub desc: *const ::core::ffi::c_char,
    pub key: [keycode_t; 2],
    pub cmd: cmd_code,
    pub hook: Option<unsafe extern "C" fn() -> ()>,
    pub prereq: Option<unsafe extern "C" fn() -> bool>,
    pub nested_keymap: ::core::ffi::c_int,
    pub nested_prompt: *const ::core::ffi::c_char,
    pub nested_error: *const ::core::ffi::c_char,
    pub nested_name: *const ::core::ffi::c_char,
    pub nested_cached_idx: ::core::ffi::c_int,
}
extern "C" {
    pub fn get_com_ex(prompt: *const ::core::ffi::c_char, command: *mut ui_event) -> bool;
}
extern "C" {
    pub static mut inkey_flag: bool;
}
