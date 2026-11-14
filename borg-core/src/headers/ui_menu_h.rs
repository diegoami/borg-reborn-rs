//! `/home/user/borg-rs/angband/src/ui-menu.h`
pub use super::ui_event_h::ui_event;
pub use super::ui_output_h::region;
#[allow(unused_imports)]
use super::*;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct menu {
    pub header: *const ::core::ffi::c_char,
    pub title: *const ::core::ffi::c_char,
    pub prompt: *const ::core::ffi::c_char,
    pub selections: *const ::core::ffi::c_char,
    pub inscriptions: *mut ::core::ffi::c_char,
    pub cmd_keys: *const ::core::ffi::c_char,
    pub switch_keys: *const ::core::ffi::c_char,
    pub browse_hook: Option<
        unsafe extern "C" fn(::core::ffi::c_int, *mut ::core::ffi::c_void, *const region) -> (),
    >,
    pub keys_hook:
        Option<unsafe extern "C" fn(*mut menu, *const ui_event, ::core::ffi::c_int) -> bool>,
    pub context_hook:
        Option<unsafe extern "C" fn(*mut menu, *const ui_event, *mut ui_event) -> bool>,
    pub flags: ::core::ffi::c_int,
    pub boundary: region,
    pub filter_count: ::core::ffi::c_int,
    pub filter_list: *const ::core::ffi::c_int,
    pub count: ::core::ffi::c_int,
    pub menu_data: *mut ::core::ffi::c_void,
    pub skin: *const menu_skin,
    pub row_funcs: *const menu_iter,
    pub cursor: ::core::ffi::c_int,
    pub top: ::core::ffi::c_int,
    pub active: region,
    pub cursor_x_offset: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct menu_iter {
    pub get_tag: Option<unsafe extern "C" fn(*mut menu, ::core::ffi::c_int) -> ::core::ffi::c_char>,
    pub valid_row:
        Option<unsafe extern "C" fn(*mut menu, ::core::ffi::c_int) -> ::core::ffi::c_int>,
    pub display_row: Option<
        unsafe extern "C" fn(
            *mut menu,
            ::core::ffi::c_int,
            bool,
            ::core::ffi::c_int,
            ::core::ffi::c_int,
            ::core::ffi::c_int,
        ) -> (),
    >,
    pub row_handler:
        Option<unsafe extern "C" fn(*mut menu, *const ui_event, ::core::ffi::c_int) -> bool>,
    pub resize: Option<unsafe extern "C" fn(*mut menu) -> ()>,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct menu_skin {
    pub get_cursor: Option<
        unsafe extern "C" fn(
            ::core::ffi::c_int,
            ::core::ffi::c_int,
            ::core::ffi::c_int,
            ::core::ffi::c_int,
            *mut region,
        ) -> ::core::ffi::c_int,
    >,
    pub display_list: Option<
        unsafe extern "C" fn(
            *mut menu,
            ::core::ffi::c_int,
            *mut ::core::ffi::c_int,
            *mut region,
        ) -> (),
    >,
    pub get_tag: Option<unsafe extern "C" fn(*mut menu, ::core::ffi::c_int) -> ::core::ffi::c_char>,
    pub process_dir: Option<unsafe extern "C" fn(*mut menu, ::core::ffi::c_int) -> ui_event>,
}
pub type skin_id = ::core::ffi::c_uint;
pub const MN_SKIN_COLUMNS: skin_id = 3;
pub const MN_SKIN_OBJECT: skin_id = 2;
pub const MN_SKIN_SCROLL: skin_id = 1;
extern "C" {
    pub fn menu_new(id: skin_id, iter: *const menu_iter) -> *mut menu;
}
extern "C" {
    pub fn menu_setpriv(menu: *mut menu, count: ::core::ffi::c_int, data: *mut ::core::ffi::c_void);
}
extern "C" {
    pub fn menu_layout(menu: *mut menu, loc_0: *const region) -> bool;
}
extern "C" {
    pub fn menu_select(menu: *mut menu, notify: ::core::ffi::c_int, popup: bool) -> ui_event;
}
extern "C" {
    pub static all_letters_nohjkl: [::core::ffi::c_char; 49];
}
extern "C" {
    pub static all_letters: [::core::ffi::c_char; 53];
}
