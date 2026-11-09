//! `/home/user/borg-rs/angband/src/ui-term.h`
pub use super::__stddef_wchar_t_h::wchar_t;
pub use super::h_basic_h::errr;
pub use super::stdint_uintn_h::uint16_t;
pub use super::stdint_uintn_h::uint32_t;
pub use super::stdint_uintn_h::uint8_t;
pub use super::ui_event_h::ui_event;
#[allow(unused_imports)]
use super::*;
extern "C" {
    pub fn Term_fresh() -> errr;
}
extern "C" {
    pub fn Term_locate(x: *mut ::core::ffi::c_int, y: *mut ::core::ffi::c_int) -> errr;
}
extern "C" {
    pub fn Term_inkey(ch: *mut ui_event, wait: bool, take: bool) -> errr;
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct term {
    pub user: *mut ::core::ffi::c_void,
    pub data: *mut ::core::ffi::c_void,
    pub user_flag: bool,
    pub data_flag: bool,
    pub active_flag: bool,
    pub mapped_flag: bool,
    pub total_erase: bool,
    pub fixed_shape: bool,
    pub icky_corner: bool,
    pub soft_cursor: bool,
    pub always_pict: bool,
    pub higher_pict: bool,
    pub always_text: bool,
    pub unused_flag: bool,
    pub never_bored: bool,
    pub never_frosh: bool,
    pub sidebar_mode: ::core::ffi::c_int,
    pub complex_input: bool,
    pub key_queue: *mut ui_event,
    pub key_head: uint16_t,
    pub key_tail: uint16_t,
    pub key_xtra: uint16_t,
    pub key_size: uint16_t,
    pub wid: ::core::ffi::c_int,
    pub hgt: ::core::ffi::c_int,
    pub y1: ::core::ffi::c_int,
    pub y2: ::core::ffi::c_int,
    pub x1: *mut ::core::ffi::c_int,
    pub x2: *mut ::core::ffi::c_int,
    pub offset_x: ::core::ffi::c_int,
    pub offset_y: ::core::ffi::c_int,
    pub old: *mut term_win,
    pub scr: *mut term_win,
    pub tmp: *mut term_win,
    pub mem: *mut term_win,
    pub saved: uint8_t,
    pub init_hook: Option<unsafe extern "C" fn(*mut term) -> ()>,
    pub nuke_hook: Option<unsafe extern "C" fn(*mut term) -> ()>,
    pub xtra_hook: Option<unsafe extern "C" fn(::core::ffi::c_int, ::core::ffi::c_int) -> errr>,
    pub curs_hook: Option<unsafe extern "C" fn(::core::ffi::c_int, ::core::ffi::c_int) -> errr>,
    pub bigcurs_hook: Option<unsafe extern "C" fn(::core::ffi::c_int, ::core::ffi::c_int) -> errr>,
    pub wipe_hook: Option<
        unsafe extern "C" fn(::core::ffi::c_int, ::core::ffi::c_int, ::core::ffi::c_int) -> errr,
    >,
    pub text_hook: Option<
        unsafe extern "C" fn(
            ::core::ffi::c_int,
            ::core::ffi::c_int,
            ::core::ffi::c_int,
            ::core::ffi::c_int,
            *const wchar_t,
        ) -> errr,
    >,
    pub pict_hook: Option<
        unsafe extern "C" fn(
            ::core::ffi::c_int,
            ::core::ffi::c_int,
            ::core::ffi::c_int,
            *const ::core::ffi::c_int,
            *const wchar_t,
            *const ::core::ffi::c_int,
            *const wchar_t,
        ) -> errr,
    >,
    pub view_map_hook: Option<unsafe extern "C" fn(*mut term) -> ()>,
    pub dblh_hook: Option<unsafe extern "C" fn(::core::ffi::c_int, wchar_t) -> ::core::ffi::c_int>,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct term_win {
    pub cu: bool,
    pub cv: bool,
    pub cx: ::core::ffi::c_int,
    pub cy: ::core::ffi::c_int,
    pub cnx: ::core::ffi::c_int,
    pub cny: ::core::ffi::c_int,
    pub a: *mut *mut ::core::ffi::c_int,
    pub c: *mut *mut wchar_t,
    pub va: *mut ::core::ffi::c_int,
    pub vc: *mut wchar_t,
    pub ta: *mut *mut ::core::ffi::c_int,
    pub tc: *mut *mut wchar_t,
    pub vta: *mut ::core::ffi::c_int,
    pub vtc: *mut wchar_t,
    pub next: *mut term_win,
}
extern "C" {
    pub static mut row_top_map: [::core::ffi::c_int; 3];
}
extern "C" {
    pub static mut row_bottom_map: [::core::ffi::c_int; 3];
}
extern "C" {
    pub static mut col_map: [::core::ffi::c_int; 3];
}
extern "C" {
    pub static mut Term: *mut term;
}
extern "C" {
    pub static mut tile_width: uint8_t;
}
extern "C" {
    pub static mut tile_height: uint8_t;
}
extern "C" {
    pub static mut angband_term: [*mut term; 8];
}
extern "C" {
    pub fn Term_putstr(
        x: ::core::ffi::c_int,
        y: ::core::ffi::c_int,
        n: ::core::ffi::c_int,
        a: ::core::ffi::c_int,
        s: *const ::core::ffi::c_char,
    ) -> errr;
}
extern "C" {
    pub fn Term_erase(x: ::core::ffi::c_int, y: ::core::ffi::c_int, n: ::core::ffi::c_int) -> errr;
}
extern "C" {
    pub fn Term_clear() -> errr;
}
extern "C" {
    pub fn Term_flush() -> errr;
}
extern "C" {
    pub fn Term_save() -> errr;
}
extern "C" {
    pub fn Term_load() -> errr;
}
extern "C" {
    pub static mut window_flag: [uint32_t; 8];
}
extern "C" {
    pub fn Term_addstr(
        n: ::core::ffi::c_int,
        a: ::core::ffi::c_int,
        s: *const ::core::ffi::c_char,
    ) -> errr;
}
extern "C" {
    pub fn Term_get_size(w: *mut ::core::ffi::c_int, h: *mut ::core::ffi::c_int) -> errr;
}
extern "C" {
    pub fn Term_activate(t: *mut term) -> errr;
}
extern "C" {
    pub fn Term_putch(
        x: ::core::ffi::c_int,
        y: ::core::ffi::c_int,
        a: ::core::ffi::c_int,
        c: wchar_t,
    ) -> errr;
}
extern "C" {
    pub fn Term_xtra(n: ::core::ffi::c_int, v: ::core::ffi::c_int) -> errr;
}
