//! Safe access to the C game engine: the permanent FFI boundary.
//!
//! The Borg runs inside the engine's single thread, called from
//! `inkey_hack` (or the `^Z` command). The engine globals read here are
//! stable for the duration of such a call.

use crate::headers;
use core::marker::PhantomData;

// ---------------------------------------------------------------------------
// The Borg state and the engine's entries into the Borg
//
// All Borg state is one `BorgState` (the C globals of borg/*.c). It is
// allocated on the first entry and never freed (the C globals lived for
// the whole process too), and it is reached only here: the engine calls
// into the Borg through four callbacks (the `inkey_hack` hook in borg.rs,
// the `EVENT_BELL` handler in borg_io.rs, the `EVENT_LEAVE_GAME` handler in
// borg_init.rs, and `do_cmd_borg` below), each of which goes through
// `enter`. `enter` hands the callback a `&mut BorgState`, and Borg code
// passes it down explicitly; no Borg code holds the state any other way.
//
// Nested entry (docs/SELF-REVIEW.md U1). A Borg call into the engine that
// waits for a key (a long `msg()` with `auto_more` off goes through
// `msg_flush` -> `anykey` -> `inkey_ex` -> the hook), rings the bell, or
// shows a message can run the Borg again while the outer Borg call is
// still running and holds its `&mut BorgState`. C has the same nesting (no
// guard in `inkey_ex`), so it is kept. To make it sound, every engine
// wrapper that can get there takes the caller's `&mut BorgState` and
// *lends* it for the duration of the engine call (`lend`): the nested
// entry then works on a reborrow of the caller's reference, which the
// caller cannot use until the engine call returns. A nested entry is
// reported on stderr (the harness keeps `stderr.log` per run). A nested
// entry from an engine call that does not lend the state would alias the
// outer reference; it cannot be made sound, so it panics (abort) after the
// report. The lending wrappers are: `msg`, `event_signal`,
// `event_signal_message_null`, `get_string`, `get_com`, `get_com_ex`,
// `get_quantity`, `target_set_interactive`, `screen_save`, `screen_load`,
// `Menu::select`, `project_from_player`, `do_cmd_wiz_cure_all` and
// `do_cmd_redraw`.

// Single thread (docs/SELF-REVIEW.md, review finding 3). The engine is
// single-threaded and calls the Borg only from its own thread. `enter` is
// `pub(crate)` and only the four callbacks call it; three of them are
// private `extern "C"` functions the Borg registers with the engine and
// `do_cmd_borg` is a private `#[no_mangle]` symbol, so no Rust code outside
// the crate can enter. `enter` also checks, on every entry, that the
// engine is set up (`z_info` and `player` allocated: every callback comes
// after `init_angband` and character creation) and that it runs on the
// thread of the first entry (a `ThreadId` compare). The safe wrappers
// below read engine globals; they are meant for Borg code, which only
// runs inside an entry, on the engine thread.

use crate::state::BorgState;

/// The root of the Borg state and the entry bookkeeping.
struct Root {
    /// The one `BorgState` (NULL until the first entry).
    state: *mut BorgState,
    /// How many engine callbacks into the Borg are running.
    depth: u32,
    /// The state lent to the engine call running now (NULL if none).
    lent: *mut BorgState,
    /// The engine thread (the thread of the first entry).
    thread: Option<std::thread::ThreadId>,
}

/// Engine thread only (the Borg never runs on another thread; `enter`
/// checks it).
static mut ROOT: Root =
    Root { state: core::ptr::null_mut(), depth: 0, lent: core::ptr::null_mut(), thread: None };

/// The root, as a raw pointer (no reference to the static is kept).
fn root() -> *mut Root {
    &raw mut ROOT
}

/// Run an engine callback into the Borg with the state (see above). Only
/// the engine's callbacks call it.
pub(crate) fn enter<R>(f: impl FnOnce(&mut BorgState) -> R) -> R {
    // The engine is set up before it calls the Borg (plain pointer reads).
    // SAFETY: plain reads of two pointer globals.
    let ready = unsafe { !headers::init_h::z_info.is_null() && !headers::player_h::player.is_null() };
    assert!(ready, "borg-rs: Borg entered before the engine was set up");
    let me = std::thread::current().id();
    // SAFETY: ROOT is only touched in this function, `lend` and
    // `check_access`, never through a kept reference. The thread check
    // below runs before anything else in ROOT is touched; the first entry
    // records the thread (the engine calls the Borg from one thread only,
    // so the first entry is on the engine thread).
    let (depth, p, saved_lent) = unsafe {
        let root = &mut *root();
        match root.thread {
            None => root.thread = Some(me),
            Some(t) => assert!(t == me, "borg-rs: Borg entered from a second thread"),
        }
        if root.state.is_null() {
            root.state = Box::into_raw(Box::default());
        }
        root.depth += 1;
        let p = if root.depth == 1 { root.state } else { root.lent };
        (root.depth, p, core::mem::replace(&mut root.lent, core::ptr::null_mut()))
    };
    if depth > 1 {
        eprintln!("borg-rs: nested Borg entry (depth {depth})");
        assert!(!p.is_null(), "borg-rs: nested Borg entry from an engine call that does not lend the state");
    }
    // SAFETY: at depth 1 no Borg code is running, so no other reference
    // into the state exists. At depth > 1 `p` is the reference the running
    // Borg code lent to the engine call that led here (`lend`); that code
    // cannot use it until the call returns, which is after `f` returns.
    let r = f(unsafe { &mut *p });
    // SAFETY: as above.
    unsafe {
        let root = &mut *root();
        root.depth -= 1;
        root.lent = saved_lent;
    }
    r
}

/// Run an engine call that may enter the Borg again (see above) with the
/// caller's state lent to that entry.
fn lend<R>(st: &mut BorgState, f: impl FnOnce() -> R) -> R {
    let p: *mut BorgState = st;
    // SAFETY: engine thread only; see `enter`.
    let saved = unsafe {
        // Only the root state can be lent (a `BorgState` made elsewhere
        // would hand a nested entry an unrelated state, and its `access`
        // token would not bind the caller's object handles).
        assert!(core::ptr::eq(p, (*root()).state), "borg-rs: lending a state that is not the Borg's");
        core::mem::replace(&mut (*root()).lent, p)
    };
    let r = f();
    // SAFETY: as above.
    unsafe { (*root()).lent = saved };
    r
}

/// The engine's call into the Borg commands (`do_cmd_borg`, called by
/// `do_cmd_try_borg` in cmd-misc.c for `^Z`). Private: only the engine
/// calls it (the `#[no_mangle]` symbol is exported all the same).
// borg-reborn-rs (#207): the symbol is the shell's; the shell calls this
// when `BORG_CORE=borg-rs` selects this core.
pub fn do_cmd_borg() {
    enter(crate::borg_commands::do_cmd_borg)
}

// ---------------------------------------------------------------------------
// Engine object handles (docs/SELF-REVIEW.md L4)
//
// A handle (`CarriedObject`, `StoreObject`, `GameObject`, `FloorObject`)
// is a raw pointer to an engine object plus a lifetime `'a`: the borrow of
// the state's `access` token it was made from (`inven_object(&st.access,
// i)`). Engine objects are freed or replaced only while the game runs,
// i.e. after the Borg returns a key, or inside an engine call that can run
// the game or the Borg again: exactly the calls that lend the state
// (`lend`, they take `&mut BorgState`). So:
//
// * a handle cannot be kept across a lending call: that call needs
//   `&mut st`, which conflicts with the shared borrow of `st.access` the
//   handle holds (a compile error, not a convention);
// * a handle cannot outlive the entry it was made in, nor be stored in
//   `BorgState` (which has no lifetime parameter): `st` itself is only
//   borrowed for the callback (`enter`);
// * the token must be the root state's own field: the constructors check
//   its address at run time (`check_access`), so a token of another
//   `BorgState` (or one moved out with `mem::swap`) panics instead of
//   making a handle that is not tied to the lent state.
//
// Handles stay `Copy` (copies share the lifetime). Mutating other fields
// of the state while a handle is alive is allowed (split borrows, e.g.
// `let BorgState { access, items, .. } = st;`).

/// The token that object handles borrow (`BorgState::access`).
#[derive(Default)]
pub struct Access {
    /// Not zero-sized, so its address identifies the root state's field.
    _addr: u8,
}

/// Panics unless `a` is the `access` field of the root state, inside an
/// entry.
fn check_access(a: &Access) {
    // SAFETY: engine thread only (see `enter`); plain reads of the root,
    // and the field address is computed without making a reference.
    let ok = unsafe {
        let root = &*root();
        root.depth > 0 && !root.state.is_null() && core::ptr::eq(a, core::ptr::addr_of!((*root.state).access))
    };
    assert!(ok, "borg-rs: object handle requested without the Borg state's access token");
}

/// The game's constants (`z_info`), set up by the engine before the Borg
/// can be started.
pub fn z_info() -> &'static headers::init_h::angband_constants {
    // SAFETY: z_info is allocated by init_angband() before any Borg code
    // runs (`enter` checks it) and is never freed while the game runs. It
    // is only written while the game data is parsed, so a `'static` shared
    // reference never overlaps a write.
    unsafe { &*headers::init_h::z_info }
}

use crate::headers::z_type_h::loc;

/// `square_in_bounds_fully(cave, loc(x, y))` on the current level.
pub fn square_in_bounds_fully(x: i32, y: i32) -> bool {
    // SAFETY: `cave` is the current level, valid while the game runs.
    unsafe { headers::cave_h::square_in_bounds_fully(headers::cave_h::cave, loc { x, y }) }
}

/// `distance(grid1, grid2)` (cave-view.c): the game's approximate
/// distance, `max(ay, ax) + min(ay, ax) / 2`. Computed here exactly as C
/// does; panics where C's `int` arithmetic would overflow (`grid2 - grid1`
/// or `abs` of `INT_MIN`, or the final sum), which C leaves undefined.
pub fn distance(grid1: loc, grid2: loc) -> i32 {
    let abs_diff = |a: i32, b: i32| {
        b.checked_sub(a).and_then(i32::checked_abs).expect("distance: coordinate difference overflows int")
    };
    let ay = abs_diff(grid1.y, grid2.y);
    let ax = abs_diff(grid1.x, grid2.x);
    let r = if ay > ax { ay.checked_add(ax >> 1) } else { ax.checked_add(ay >> 1) };
    r.expect("distance: result overflows int")
}

/// Panel geometry of the current terminal (`Term`), as read by
/// `borg_panel_hgt()` / `borg_panel_wid()`.
#[derive(Clone, Copy, Debug)]
pub struct TermGeometry {
    /// `Term == angband_term[0]`
    pub is_main: bool,
    pub wid: i32,
    pub hgt: i32,
    pub offset_x: i32,
    pub offset_y: i32,
    pub row_top_map: i32,
    pub row_bottom_map: i32,
    pub col_map: i32,
    pub tile_width: i32,
    pub tile_height: i32,
}

/// Read the current terminal's geometry.
pub fn term_geometry() -> TermGeometry {
    use crate::headers::ui_term_h as t;
    // SAFETY: Term and the map/tile globals are set up by the frontend
    // before the Borg runs; we only read plain values.
    unsafe {
        let term = &*t::Term;
        let mode = term.sidebar_mode as usize;
        TermGeometry {
            is_main: t::Term == t::angband_term[0],
            wid: term.wid,
            hgt: term.hgt,
            offset_x: term.offset_x,
            offset_y: term.offset_y,
            row_top_map: t::row_top_map[mode],
            row_bottom_map: t::row_bottom_map[mode],
            col_map: t::col_map[mode],
            tile_width: i32::from(t::tile_width),
            tile_height: i32::from(t::tile_height),
        }
    }
}

// ---------------------------------------------------------------------------
// Terminal

use crate::headers::game_event_h::{game_event_handler, game_event_type};
use crate::headers::__stddef_wchar_t_h::wchar_t;
use core::ffi::c_char;
use std::ffi::CString;

/// An opaque handle to an engine terminal (to restore the active one).
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct TermHandle(*mut crate::headers::ui_term_h::term);

/// The active terminal (`Term`).
pub fn term_current() -> TermHandle {
    // SAFETY: reads a pointer-sized global.
    unsafe { TermHandle(crate::headers::ui_term_h::Term) }
}

/// `angband_term[j]`, if that window exists.
pub fn angband_term(j: usize) -> Option<TermHandle> {
    // SAFETY: reads a pointer from a fixed-size global array.
    let t = unsafe { crate::headers::ui_term_h::angband_term[j] };
    (!t.is_null()).then_some(TermHandle(t))
}

/// `window_flag[j]`.
pub fn window_flag(j: usize) -> u32 {
    // SAFETY: reads a fixed-size global array.
    unsafe { crate::headers::ui_term_h::window_flag[j] }
}

/// `Term_activate(t)`.
pub fn term_activate(t: TermHandle) {
    // SAFETY: t came from term_current()/angband_term(), a live term.
    unsafe {
        crate::headers::ui_term_h::Term_activate(t.0);
    }
}

/// `Term_get_size(&w, &h)`.
pub fn term_get_size() -> (i32, i32) {
    let (mut w, mut h) = (0, 0);
    // SAFETY: out-pointers to locals.
    unsafe {
        crate::headers::ui_term_h::Term_get_size(&mut w, &mut h);
    }
    (w, h)
}

/// `Term_locate(&x, &y)`: the cursor position.
pub fn term_locate() -> (i32, i32) {
    let (mut x, mut y) = (0, 0);
    // SAFETY: out-pointers to locals.
    unsafe {
        crate::headers::ui_term_h::Term_locate(&mut x, &mut y);
    }
    (x, y)
}

/// `Term_erase(x, y, n)`.
pub fn term_erase(x: i32, y: i32, n: i32) {
    // SAFETY: plain call on the active term; the engine checks the
    // position (`Term_gotoxy`) and clips `n` to the row.
    unsafe {
        crate::headers::ui_term_h::Term_erase(x, y, n);
    }
}

/// `Term_addstr(n, a, s)` with a byte string (no interior NUL).
pub fn term_addstr(n: i32, a: i32, s: &[u8]) {
    let c = CString::new(s).expect("no NUL in term text");
    // SAFETY: c is a valid NUL-terminated string for the call.
    unsafe {
        crate::headers::ui_term_h::Term_addstr(n, a, c.as_ptr());
    }
}

/// `Term_fresh()`.
pub fn term_fresh() {
    // SAFETY: plain call on the active term.
    unsafe {
        crate::headers::ui_term_h::Term_fresh();
    }
}

/// The attribute and character on screen at (x, y) of the active term
/// (`Term->scr->a[y][x]`, `Term->scr->c[y][x]`).
/// Panics if (x, y) is off the term (C would read outside the screen).
pub fn screen_at(x: i32, y: i32) -> (i32, wchar_t) {
    // SAFETY: `Term` is the active term while the game runs; (x, y) is
    // inside it (checked below), and `scr` rows are allocated for its size.
    unsafe {
        let t = &*crate::headers::ui_term_h::Term;
        assert!((0..t.wid).contains(&x) && (0..t.hgt).contains(&y), "screen position off the term");
        let scr = &*t.scr;
        let a = *(*scr.a.offset(y as isize)).offset(x as isize);
        let c = *(*scr.c.offset(y as isize)).offset(x as isize);
        (a, c)
    }
}

// ---------------------------------------------------------------------------
// Text conversion (locale dependent, like the C code)

/// `wcstombs(dst, src, dst.len())`: convert the NUL-terminated wide
/// string `src` into `dst`. Returns the number of bytes written (not
/// counting the NUL), or `usize::MAX` on a conversion error.
pub fn wcstombs(dst: &mut [u8], src: &[wchar_t]) -> usize {
    assert!(src.contains(&0), "wide string must be NUL-terminated");
    // SAFETY: src is NUL-terminated; wcstombs writes at most dst.len().
    unsafe {
        crate::headers::stdlib_h::wcstombs(dst.as_mut_ptr() as *mut c_char, src.as_ptr(), dst.len())
    }
}

/// `text_mbstowcs(dst, src, n)`: the game's multibyte-to-wide conversion.
/// `src` must be NUL-terminated.
pub fn text_mbstowcs(dst: &mut [wchar_t], src: &[u8], n: i32) -> usize {
    assert!(src.contains(&0), "string must be NUL-terminated");
    assert!(n >= 0 && n as usize <= dst.len());
    // SAFETY: src is NUL-terminated; at most n wide chars are written.
    unsafe {
        crate::headers::z_util_h::text_mbstowcs(dst.as_mut_ptr(), src.as_ptr() as *const c_char, n)
    }
}

// ---------------------------------------------------------------------------
// Messages and events

/// `msg("%s", text)`. May wait for a key (lends the state).
pub fn msg(st: &mut BorgState, text: &[u8]) {
    let c = CString::new(text).expect("no NUL in message");
    // SAFETY: format string and argument are valid C strings.
    lend(st, || unsafe { crate::headers::message_h::msg(c"%s".as_ptr(), c.as_ptr()) })
}

/// `message_add(text, type)`.
pub fn message_add(text: &[u8], kind: u16) {
    let c = CString::new(text).expect("no NUL in message");
    // SAFETY: c is valid for the call; message_add copies it.
    unsafe { crate::headers::message_h::message_add(c.as_ptr(), kind) }
}

/// `N_GAME_EVENTS` (game-event.h): the size of the engine's handler table,
/// which every event function indexes by the event type without a check.
const N_GAME_EVENTS: game_event_type = crate::headers::game_event_h::EVENT_END + 1;

/// Panics unless `kind` indexes the engine's handler table.
fn check_event(kind: game_event_type) {
    assert!(kind < N_GAME_EVENTS, "game event type {kind} out of range");
}

/// A game event handler (`game_event_handler`), as a safe function: the
/// engine calls it with an event type, the event data (or NULL) and the
/// NULL user pointer registered here.
pub type EventHandler =
    extern "C" fn(game_event_type, *mut crate::headers::game_event_h::game_event_data, *mut core::ffi::c_void);

/// `event_signal_message(type, t, NULL)` (runs event handlers: lends the
/// state).
pub fn event_signal_message_null(st: &mut BorgState, kind: game_event_type, t: i32) {
    check_event(kind);
    // SAFETY: NULL message text is allowed by the engine; the type indexes
    // the handler table (checked).
    lend(st, || unsafe { crate::headers::game_event_h::event_signal_message(kind, t, core::ptr::null()) })
}

/// `event_add_handler(type, fn, NULL)`.
pub fn event_add_handler(kind: game_event_type, f: EventHandler) {
    check_event(kind);
    let f: game_event_handler = f;
    // SAFETY: registers a safe function pointer (no precondition for the
    // engine to meet) with a NULL user pointer; the type indexes the
    // handler table (checked).
    unsafe { crate::headers::game_event_h::event_add_handler(kind, Some(f), core::ptr::null_mut()) }
}

/// `event_remove_handler(type, fn, NULL)`.
pub fn event_remove_handler(kind: game_event_type, f: EventHandler) {
    check_event(kind);
    let f: game_event_handler = f;
    // SAFETY: unregisters a function pointer (a no-op if it is not
    // registered); the type indexes the handler table (checked).
    unsafe {
        crate::headers::game_event_h::event_remove_handler(kind, Some(f), core::ptr::null_mut())
    }
}

// ---------------------------------------------------------------------------
// Player, input, RNG

use crate::headers::ui_event_h::{keypress, ui_event_type};

/// `*player`. Private: the engine writes the player all the time, so the
/// reference is only used for the field reads at hand inside one wrapper,
/// never across an engine call (the compiler does not check this; it is
/// confined to this file).
fn player_ref() -> &'static crate::headers::player_h::player {
    // SAFETY: `player` is allocated at startup and lives for the game
    // (`enter` checks it is set up before any Borg code runs).
    unsafe { &*crate::headers::player_h::player }
}

/// `player->is_dead`
pub fn player_is_dead() -> bool {
    player_ref().is_dead
}

/// `player->died_from` (bytes up to the NUL).
pub fn player_died_from() -> Vec<u8> {
    let d = &player_ref().died_from;
    d.iter().take_while(|&&c| c != 0).map(|&c| c as u8).collect()
}

/// `player->full_name` (bytes up to the NUL).
pub fn player_full_name() -> Vec<u8> {
    c_array_bytes(&player_ref().full_name)
}

/// `player_is_shapechanged(player)`.
pub fn player_is_shapechanged() -> bool {
    let p: *const crate::headers::player_h::player = player_ref();
    // SAFETY: p is the live player; the call only reads `p->shape`.
    unsafe { crate::headers::player_util_h::player_is_shapechanged(p) }
}

/// A copy of the game's `savefile` buffer (ui-game.c, `char savefile[1024]`).
pub fn savefile() -> [u8; 1024] {
    // SAFETY: plain copy of a char array global; engine thread.
    let s = unsafe { *core::ptr::addr_of!(crate::headers::ui_game_h::savefile) };
    s.map(|c| c as u8)
}

/// `memcpy(savefile, v, sizeof(savefile))`.
pub fn set_savefile(v: &[u8; 1024]) {
    // SAFETY: plain write of a char array global; engine thread.
    unsafe { *core::ptr::addr_of_mut!(crate::headers::ui_game_h::savefile) = v.map(|c| c as c_char) };
}

/// `path_build(savefile, sizeof(savefile), ANGBAND_DIR_ARCHIVE, leaf)`:
/// builds the path in place (bytes after its NUL are left as they were).
pub fn savefile_path_build_archive(leaf: &[u8]) {
    let leaf = CString::new(leaf).expect("no NUL in file name");
    // SAFETY: savefile has 1024 bytes and path_build writes at most that
    // many; ANGBAND_DIR_ARCHIVE is a C string set up at startup (or NULL,
    // which path_build accepts); leaf is a C string.
    unsafe {
        crate::headers::z_file_h::path_build(
            core::ptr::addr_of_mut!(crate::headers::ui_game_h::savefile) as *mut c_char,
            1024,
            crate::headers::init_h::ANGBAND_DIR_ARCHIVE,
            leaf.as_ptr(),
        );
    }
}

/// `player->upkeep->resting`
pub fn player_resting() -> i32 {
    // SAFETY: upkeep is allocated with the player.
    unsafe { (*player_ref().upkeep).resting }
}

/// `player->upkeep->arena_level`
pub fn player_arena_level() -> bool {
    // SAFETY: upkeep is allocated with the player.
    unsafe { (*player_ref().upkeep).arena_level }
}

/// `cmd_get_nrepeats()`
pub fn cmd_get_nrepeats() -> i32 {
    // SAFETY: plain call.
    unsafe { crate::headers::cmd_core_h::cmd_get_nrepeats() }
}

/// `Term_inkey(&ev, wait, take)`: returns the event type and its key.
pub fn term_inkey(wait: bool, take: bool) -> (ui_event_type, keypress) {
    // SAFETY: ui_event is a C union of plain-data structs that all start
    // with the type; Term_inkey zeroes it first, so reading the key
    // variant is always defined.
    unsafe {
        let mut ev: crate::headers::ui_event_h::ui_event = core::mem::zeroed();
        crate::headers::ui_term_h::Term_inkey(&mut ev, wait, take);
        (ev.type_0, ev.key)
    }
}

/// `flush(0, 0, 0)` (ui-input.c): discard pending input.
pub fn flush_input() {
    // SAFETY: the engine's flush ignores its arguments.
    unsafe { crate::headers::ui_input_h::flush(0, core::ptr::null_mut(), core::ptr::null_mut()) }
}

/// The game RNG's simple-mode state (`Rand_quick`, `Rand_value`).
pub fn rand_state() -> (bool, u32) {
    // SAFETY: plain globals.
    unsafe { (crate::headers::z_rand_h::Rand_quick, crate::headers::z_rand_h::Rand_value) }
}

/// Set `Rand_quick` and `Rand_value`.
pub fn set_rand_state(quick: bool, value: u32) {
    // SAFETY: plain globals, engine thread.
    unsafe {
        crate::headers::z_rand_h::Rand_quick = quick;
        crate::headers::z_rand_h::Rand_value = value;
    }
}

/// The `inkey_hack` callback type (a safe function: the engine has no
/// precondition to meet when it calls it).
pub type InkeyHack = extern "C" fn(core::ffi::c_int) -> keypress;

/// Set or clear `inkey_hack` (ui-input.c).
pub fn set_inkey_hack(f: Option<InkeyHack>) {
    let f: Option<unsafe extern "C" fn(core::ffi::c_int) -> keypress> = f.map(|f| f as _);
    // SAFETY: plain global, engine thread.
    unsafe { crate::headers::ui_input_h::inkey_hack = f }
}

// ---------------------------------------------------------------------------
// Game data lookups

/// Walk the activation list from `activations[1]` and return the index of
/// the first entry whose name equals `name` ignoring case (`my_stricmp`).
pub fn find_activation(name: &str) -> Option<i32> {
    let cname = CString::new(name).expect("no NUL in name");
    // SAFETY: `activations` is loaded at startup; the list is linked
    // through `next` and terminated by NULL; names are C strings.
    unsafe {
        let mut act = crate::headers::object_h::activations.add(1);
        while !act.is_null() {
            if crate::headers::z_util_h::my_stricmp((*act).name, cname.as_ptr()) == 0 {
                return Some((*act).index);
            }
            act = (*act).next;
        }
    }
    None
}

/// An object kind (`struct object_kind`).
pub type ObjectKind = crate::headers::object_h::object_kind;

// Engine tables (docs/SELF-REVIEW.md L5). `k_info`, `r_info`, `a_info`
// and `e_info` are written while the game runs (`aware`, `tried`,
// `everseen` and `ignore` of a kind, `everseen` of an ego -- `object_desc`
// itself sets them --, `cur_num` and `max_num` of a race, randarts at
// birth), so no reference into them is handed out: the accessors return
// a copy of one entry, read at the moment of the call (C reads through a
// pointer at the moment of each field access; the Borg makes no engine
// call that writes an entry between taking a copy and reading it, the
// same argument as for the old reference). `&'static` stays for data
// that nothing writes after the game data is loaded: `z_info`, `kb_info`
// (object bases), the race/class lists and the name strings.

/// `z_info->k_max`: the number of object kinds.
pub fn k_max() -> usize {
    usize::from(z_info().k_max)
}

/// `&k_info[kind]` (panics for `kind >= k_max`). No reference is made.
fn kind_ptr(kind: usize) -> *const ObjectKind {
    assert!(kind < k_max(), "object kind {kind} out of range");
    // SAFETY: plain read of the k_info pointer global (set when the game
    // data is loaded). k_info has k_max entries (index checked), so the
    // offset stays inside the allocation.
    unsafe { crate::headers::object_h::k_info.add(kind) }
}

/// A copy of `k_info[kind]` (panics for `kind >= k_max`).
#[inline]
pub fn kind(kind: usize) -> ObjectKind {
    let p = kind_ptr(kind);
    // SAFETY: p points at a live entry of k_info (see kind_ptr); copied at
    // once, no reference is kept.
    unsafe { *p }
}

/// `k_info[kind].base->max_stack`: how many objects of this kind fit in
/// one stack (panics past `k_max` or for a kind without a base, where C
/// would read out of bounds or through NULL).
pub fn kind_max_stack(kind: usize) -> i32 {
    let base = self::kind(kind).base;
    assert!(!base.is_null(), "k_info[{}] has no base", kind);
    // SAFETY: a kind's base points into kb_info, which is allocated with
    // the game data before any Borg code runs and lives for the game
    // (checked non-NULL above).
    unsafe { (*base).max_stack }
}

/// An object base (`struct object_base`).
pub type ObjectBase = crate::headers::object_h::object_base;

/// `kb_info[0 .. TV_MAX]`, the object bases (indexed by tval).
pub fn kb_info() -> &'static [ObjectBase] {
    let n = crate::headers::obj_properties_h::TV_MAX as usize;
    // SAFETY: kb_info is allocated with TV_MAX entries (obj-init.c) when
    // the game data is loaded, before any Borg code runs, and lives for the
    // game. Nothing writes it after loading (no engine code assigns to an
    // object base outside the parser), so a shared `'static` slice never
    // overlaps a write.
    unsafe { std::slice::from_raw_parts(crate::headers::object_h::kb_info, n) }
}

/// `tval_find_idx(name)`: the tval with this name, or -1.
pub fn tval_find_idx(name: &str) -> i32 {
    let c = CString::new(name).expect("no NUL in name");
    // SAFETY: c is a valid C string for the call.
    unsafe { crate::headers::obj_tval_h::tval_find_idx(c.as_ptr()) }
}

/// `tval_find_name(tval)`: the tval's name ("unknown" if there is none).
pub fn tval_find_name(tval: i32) -> &'static [u8] {
    // SAFETY: the result points into the static tval name table (or is a
    // string literal), never NULL.
    unsafe {
        std::ffi::CStr::from_ptr(crate::headers::obj_tval_h::tval_find_name(tval)).to_bytes()
    }
}

/// `lookup_sval(tval, name)`: the sval of the kind with this name, or -1.
pub fn lookup_sval(tval: i32, name: &str) -> i32 {
    lookup_sval_bytes(tval, name.as_bytes())
}

/// `lookup_sval(tval, name)` for a name that may not be UTF-8 (borg.txt
/// bytes).
pub fn lookup_sval_bytes(tval: i32, name: &[u8]) -> i32 {
    let c = CString::new(name).expect("no NUL in name");
    // SAFETY: c is a valid C string; lookup_sval only reads it and k_info.
    unsafe { crate::headers::obj_util_h::lookup_sval(tval, c.as_ptr()) }
}

/// A monster race (`struct monster_race`).
pub type MonsterRace = crate::headers::monster_h::monster_race;

/// `z_info->r_max`: the number of monster races.
pub fn r_max() -> usize {
    usize::from(z_info().r_max)
}

/// A copy of `r_info[r_idx]` (panics for `r_idx >= r_max`; see "Engine
/// tables": `cur_num`/`max_num` change while the game runs).
#[inline]
pub fn race(r_idx: usize) -> MonsterRace {
    assert!(r_idx < r_max(), "monster race {r_idx} out of range");
    // SAFETY: r_info is allocated with z_info->r_max entries when the game
    // data is loaded, before any Borg code runs, and lives for the game
    // (index checked); copied at once, no reference is kept.
    unsafe { *crate::headers::monster_h::r_info.add(r_idx) }
}

/// `r_info[r_idx].name` (bytes up to the NUL).
pub fn race_name(r_idx: usize) -> &'static [u8] {
    race_name_cstr(r_idx).to_bytes()
}

/// `r_info[r_idx].name` as a C string.
pub fn race_name_cstr(r_idx: usize) -> &'static CStr {
    let name = race(r_idx).name;
    assert!(!name.is_null(), "race {r_idx} has no name");
    // SAFETY: a race's name is a C string allocated when the game data is
    // loaded and kept for the game; checked non-null.
    unsafe { CStr::from_ptr(name) }
}

/// `r_info[r_idx].plural` (bytes up to the NUL), `None` for NULL.
pub fn race_plural(r_idx: usize) -> Option<&'static [u8]> {
    let p = race(r_idx).plural;
    // SAFETY: a non-NULL plural is a C string allocated when the game data
    // is loaded and kept for the game.
    (!p.is_null()).then(|| unsafe { CStr::from_ptr(p).to_bytes() })
}

/// `r_info[r_idx].base->name` (bytes up to the NUL).
pub fn race_base_name(r_idx: usize) -> &'static [u8] {
    let base = race(r_idx).base;
    assert!(!base.is_null(), "race {r_idx} has no base");
    // SAFETY: a race's base points into rb_info, loaded with the game data
    // and kept for the game (checked non-null); its name is checked too.
    unsafe {
        let name = (*base).name;
        assert!(!name.is_null(), "race {r_idx} base has no name");
        CStr::from_ptr(name).to_bytes()
    }
}

/// `k_info[kidx].name` (bytes up to the NUL).
pub fn kind_name(kidx: usize) -> &'static [u8] {
    let name = self::kind(kidx).name;
    assert!(!name.is_null(), "kind {kidx} has no name");
    // SAFETY: a kind's name is a C string allocated when the game data is
    // loaded and kept for the game; checked non-null.
    unsafe { CStr::from_ptr(name).to_bytes() }
}

/// The fields of a game monster (`struct monster`) the Borg cheats.
#[derive(Clone, Copy, Debug)]
pub struct MonsterView {
    pub midx: i32,
    pub grid: loc,
    pub hp: i16,
    pub maxhp: i16,
    pub m_timed: [i16; crate::headers::list_mon_timed_h::MON_TMD_MAX as usize],
    pub mspeed: u8,
    /// `race->ridx`, `None` for a NULL race (an unused slot).
    pub ridx: Option<u32>,
    /// `attr`: the colour the monster is drawn in
    pub attr: u8,
    /// `mimicked_obj != NULL`: the monster mimics an object
    pub mimics: bool,
}

/// Copy the fields of a monster.
///
/// SAFETY (caller): `m` points at a `struct monster` of the current level.
unsafe fn monster_view(m: *const crate::headers::monster_h::monster) -> MonsterView {
    let m = &*m;
    MonsterView {
        midx: m.midx,
        grid: m.grid,
        hp: m.hp,
        maxhp: m.maxhp,
        m_timed: m.m_timed,
        mspeed: m.mspeed,
        ridx: (!m.race.is_null()).then(|| (*m.race).ridx),
        attr: m.attr,
        mimics: !m.mimicked_obj.is_null(),
    }
}

/// `square_monster(cave, loc(x, y))`, `None` for no monster (or a grid
/// out of bounds: `square_monster` checks it itself, with or without
/// NDEBUG).
pub fn square_monster(x: i32, y: i32) -> Option<MonsterView> {
    // SAFETY: `cave` is the current level; square_monster returns NULL for
    // a grid out of bounds (its own check), else NULL or a live monster of
    // the level, whose race (if any) points into r_info.
    unsafe {
        let m = crate::headers::cave_h::square_monster(crate::headers::cave_h::cave, loc { x, y });
        (!m.is_null()).then(|| monster_view(m))
    }
}

/// `cave->monsters[m_idx]` (any slot, live or not).
pub fn cave_monster(m_idx: i32) -> MonsterView {
    let max = i32::from(z_info().level_monster_max);
    assert!((0..max).contains(&m_idx), "monster index {m_idx} out of range");
    // SAFETY: `cave->monsters` has z_info->level_monster_max entries
    // (cave_new zeroes them); the index is checked; a slot's race is NULL
    // or a pointer into r_info.
    unsafe {
        let c = &*crate::headers::cave_h::cave;
        monster_view(c.monsters.add(m_idx as usize))
    }
}

/// `rf_has(r_info[r_idx].flags, flag)`.
pub fn race_flag_has(r_idx: usize, flag: u32) -> bool {
    // The flag is checked against the RF_SIZE-byte array by `flag_has`
    // (as the `int` C gets: a value past `i32::MAX` is negative and
    // panics).
    flag_has(&race(r_idx).flags, flag as i32)
}

// ---------------------------------------------------------------------------
// Player class magic (books and spells)

use core::ffi::CStr;

fn class_magic_ref() -> &'static crate::headers::player_h::class_magic {
    // SAFETY: `player->class` points into the class list loaded at startup
    // (it is set at birth and lives for the game).
    unsafe { &(*player_ref().class).magic }
}

/// `player->class->cidx`, or `None` while `player` or `player->class` is
/// not set
pub fn class_cidx_if_set() -> Option<u32> {
    // SAFETY: `player` and `player->class` are only read when non-null; the
    // class list lives for the game.
    unsafe {
        let p = crate::headers::player_h::player;
        if p.is_null() || (*p).class.is_null() {
            return None;
        }
        Some((*(*p).class).cidx)
    }
}

/// `player->class->cidx`
pub fn class_cidx() -> u32 {
    // SAFETY: as in class_magic_ref().
    unsafe { (*player_ref().class).cidx }
}

/// `player->class->magic.num_books`
pub fn class_num_books() -> i32 {
    class_magic_ref().num_books
}

/// `player->class->magic.total_spells`
pub fn class_total_spells() -> i32 {
    class_magic_ref().total_spells
}

/// The plain data of a `struct class_book`.
#[derive(Clone, Copy, Debug)]
pub struct ClassBook {
    pub tval: i32,
    pub sval: i32,
    pub dungeon: bool,
    pub num_spells: i32,
}

fn class_book_ref(book: i32) -> &'static crate::headers::player_h::class_book {
    let m = class_magic_ref();
    assert!(book >= 0 && book < m.num_books, "class book index out of range");
    // SAFETY: `books` has `num_books` entries (checked above).
    unsafe { &*m.books.add(book as usize) }
}

/// `player->class->magic.books[book]` (0 <= book < num_books).
pub fn class_book(book: i32) -> ClassBook {
    let b = class_book_ref(book);
    ClassBook { tval: b.tval, sval: b.sval, dungeon: b.dungeon, num_spells: b.num_spells }
}

/// The plain data of a `struct class_spell`.
#[derive(Clone, Copy, Debug)]
pub struct ClassSpell {
    /// `name` (bytes up to the NUL)
    pub name: &'static [u8],
    pub sidx: i32,
    pub bidx: i32,
    pub slevel: i32,
    pub smana: i32,
    pub sfail: i32,
}

fn class_spell_data(s: &'static crate::headers::player_h::class_spell) -> ClassSpell {
    // SAFETY: spell names are C strings loaded from class.txt that live
    // for the game.
    let name = unsafe { CStr::from_ptr(s.name).to_bytes() };
    ClassSpell {
        name,
        sidx: s.sidx,
        bidx: s.bidx,
        slevel: s.slevel,
        smana: s.smana,
        sfail: s.sfail,
    }
}

fn class_book_spell_ref(book: i32, n: i32) -> &'static crate::headers::player_h::class_spell {
    let b = class_book_ref(book);
    assert!(n >= 0 && n < b.num_spells, "book spell index out of range");
    // SAFETY: `spells` has `num_spells` entries (checked above).
    unsafe { &*b.spells.add(n as usize) }
}

/// `player->class->magic.books[book].spells[n]`.
pub fn class_book_spell(book: i32, n: i32) -> ClassSpell {
    class_spell_data(class_book_spell_ref(book, n))
}

/// `player->class->magic.books[book].spells[n].realm->stat`.
pub fn class_book_spell_realm_stat(book: i32, n: i32) -> i32 {
    let s = class_book_spell_ref(book, n);
    assert!(!s.realm.is_null(), "class spell without a realm");
    // SAFETY: realm points into the static realm list (checked non-NULL).
    unsafe { (*s.realm).stat }
}

fn spell_by_index_ref(index: i32) -> &'static crate::headers::player_h::class_spell {
    // SAFETY: spell_by_index only reads the player's class data and
    // returns NULL or a pointer into it.
    let p = unsafe {
        crate::headers::player_spell_h::spell_by_index(crate::headers::player_h::player, index)
    };
    assert!(!p.is_null(), "spell_by_index: no such spell");
    // SAFETY: non-NULL pointer into the class data, which lives for the game.
    unsafe { &*p }
}

/// `spell_by_index(player, index)` (panics where C would get NULL).
pub fn spell_by_index(index: i32) -> ClassSpell {
    class_spell_data(spell_by_index_ref(index))
}

/// The effect indices of `spell_by_index(player, index)->effect`, in list
/// order (following `next` until NULL).
pub fn spell_effect_indices(index: i32) -> Vec<u16> {
    let s = spell_by_index_ref(index);
    let mut out = Vec::new();
    let mut eff = s.effect;
    while !eff.is_null() {
        // SAFETY: effects form a NULL-terminated list loaded at startup.
        unsafe {
            out.push((*eff).index);
            eff = (*eff).next;
        }
    }
    out
}

/// `spell_okay_to_cast(player, index)` (0 <= index < total_spells; C reads
/// `p->spell_flags[index]` without a check).
pub fn spell_okay_to_cast(index: i32) -> bool {
    assert!(index >= 0 && index < class_total_spells(), "spell index out of range");
    // SAFETY: reads `player->spell_flags[index]`; spell_flags is allocated
    // with total_spells entries at birth (index checked above).
    unsafe {
        crate::headers::player_spell_h::spell_okay_to_cast(crate::headers::player_h::player, index)
    }
}

/// `player->spell_flags[sidx]`.
pub fn player_spell_flags(sidx: i32) -> u8 {
    assert!(sidx >= 0 && sidx < class_total_spells(), "spell index out of range");
    // SAFETY: spell_flags is allocated with total_spells entries at birth.
    unsafe { *player_ref().spell_flags.add(sidx as usize) }
}

/// `player_has(player, flag)`: a player flag in `player->state.pflags`.
pub fn player_has(flag: u32) -> bool {
    // A copy of the PF_SIZE-byte array; `flag_has` checks the flag.
    let f = player_ref().state.pflags;
    flag_has(&f, flag as i32)
}

/// `no_light(player)`: the player's grid is not lit.
pub fn no_light() -> bool {
    // SAFETY: reads the player and the current level.
    unsafe { crate::headers::cave_h::no_light(crate::headers::player_h::player) }
}

/// `all_letters_nohjkl[i]` (ui-menu.c): the menu letters without h/j/k/l
/// (49 bytes with the NUL; panics past it, where C would read out of bounds).
pub fn all_letters_nohjkl(i: i32) -> u8 {
    // SAFETY: a constant array defined in ui-menu.c, declared with its C
    // length (`[c_char; 49]`); nothing writes it.
    unsafe { crate::headers::ui_menu_h::all_letters_nohjkl[i as usize] as u8 }
}

/// `all_letters[i]` (ui-menu.c): the menu letters a-z, A-Z (53 bytes with
/// the NUL; panics past it, where C would read out of bounds).
pub fn all_letters(i: usize) -> u8 {
    // SAFETY: a constant array defined in ui-menu.c, declared with its C
    // length (`[c_char; 53]`); nothing writes it.
    unsafe { crate::headers::ui_menu_h::all_letters[i] as u8 }
}

/// `randint0(m)` (z-rand.h): `(int32_t) Rand_div(m)` from the game RNG
/// (the Borg's own RNG while it thinks, see `borg::generate_keypress`).
/// Panics for `m > 0x10000000`, where `Rand_div` divides by zero (its
/// `assert` is compiled out).
pub fn randint0(m: u32) -> i32 {
    assert!(m <= 0x1000_0000, "randint0({m}): Rand_div would divide by zero");
    // SAFETY: plain call (m in Rand_div's domain, checked); the RNG state
    // is engine-thread global.
    unsafe { crate::headers::z_rand_h::Rand_div(m) as i32 }
}

/// `player->race->ridx`
pub fn race_ridx() -> u32 {
    // SAFETY: `player->race` points into the race list loaded at startup
    // (set at birth, lives for the game).
    unsafe { (*player_ref().race).ridx }
}

/// `player->noscore`
pub fn player_noscore() -> u16 {
    player_ref().noscore
}

/// `player->noscore = v`
pub fn set_player_noscore(v: u16) {
    // SAFETY: `player` is allocated at startup; plain field store on the
    // engine thread, no reference to it is held across the call.
    unsafe { (*crate::headers::player_h::player).noscore = v }
}

/// `do_cmd_wiz_cure_all(NULL)` (cmd-wizard.c): heal the player fully
/// (shows messages: lends the state).
pub fn do_cmd_wiz_cure_all(st: &mut BorgState) {
    // SAFETY: the command argument is unused by this wizard command (the
    // C Borg passes NULL); it only changes the player on the engine thread.
    lend(st, || unsafe { crate::headers::cmds_h::do_cmd_wiz_cure_all(core::ptr::null_mut()) })
}

/// `option_set(name, val)` (option.c): returns whether the option exists.
pub fn option_set(name: &CStr, val: bool) -> bool {
    // SAFETY: option_set only reads the NUL-terminated name.
    unsafe { crate::headers::option_h::option_set(name.as_ptr(), i32::from(val)) }
}

// ---------------------------------------------------------------------------
// Player class

fn player_class_ref() -> &'static crate::headers::player_h::player_class {
    // SAFETY: the player's class is set at birth, before the Borg runs,
    // and points into the static class list.
    unsafe { &*player_ref().class }
}

/// The `cidx` of the first class in the `classes` list whose name equals
/// `name` ignoring case (`my_stricmp`), or -1.
pub fn find_class(name: &[u8]) -> i32 {
    let cname = CString::new(name).expect("no NUL in name");
    // SAFETY: `classes` is loaded at startup, linked through `next` and
    // terminated by NULL; names are C strings.
    unsafe {
        let mut c = crate::headers::player_h::classes;
        while !c.is_null() {
            if crate::headers::z_util_h::my_stricmp((*c).name, cname.as_ptr()) == 0 {
                return (*c).cidx as i32;
            }
            c = (*c).next;
        }
    }
    -1
}

/// `player->class->magic.spell_weight`
pub fn class_spell_weight() -> i32 {
    player_class_ref().magic.spell_weight
}

/// `player->class->magic.books[book].dungeon`
pub fn class_book_dungeon(book: i32) -> bool {
    // Panics on a book the class does not have (C does not check; callers
    // only pass books the Borg holds, `borg.amt_book[book] != 0`).
    class_book_ref(book).dungeon
}

/// `spell_chance(spell_index)`: the player's fail rate for a spell.
pub fn spell_chance(spell_index: i32) -> i16 {
    // SAFETY: plain call; the engine checks the index itself.
    unsafe { crate::headers::player_spell_h::spell_chance(spell_index) }
}

// ---------------------------------------------------------------------------
// Files

/// An engine file (`ang_file *`) opened by someone else. It has no public
/// constructor: the only one is made by `ReadFile::getl` for one call,
/// from an open `ReadFile`.
#[derive(Clone, Copy)]
pub struct AngFile(*mut crate::headers::z_file_h::ang_file);

impl AngFile {
}

/// `file_getl(f, buf, len)`: read a line into `buf` (at most `len - 1`
/// bytes and a NUL; `\r`, `\n` and `\r\n` end a line, tabs become
/// spaces). Returns false at the end of the file.
pub fn file_getl(f: AngFile, buf: &mut [u8], len: usize) -> bool {
    assert!(len >= 1 && len <= buf.len());
    // SAFETY: f is open (an AngFile only comes from an open ReadFile, see
    // the type); file_getl writes at most `len` bytes into buf (len checked).
    unsafe { crate::headers::z_file_h::file_getl(f.0, buf.as_mut_ptr() as *mut c_char, len) }
}

// ---------------------------------------------------------------------------
// Flags, player flags and race resistances

/// `FLAG_END` (z-bitflag.h): no flag.
pub const FLAG_END: i32 = 0;
/// `FLAG_START` (z-bitflag.h): the first flag.
pub const FLAG_START: i32 = 1;

/// The precondition of C's `flag_has(flags, size, flag)` with
/// `size = size_bytes`: `flag` is `FLAG_END` (C returns false before it
/// reads anything) or a flag whose byte `FLAG_OFFSET(flag)` is below the
/// size. C only `assert`s the upper bound, and the engine is built with
/// `-DNDEBUG`; a negative flag gives a huge `size_t` offset. Panics
/// otherwise (C would read outside the array).
fn check_flag_has(size_bytes: usize, flag: i32) {
    assert!(
        flag == FLAG_END || (flag >= FLAG_START && ((flag - FLAG_START) as usize) / 8 < size_bytes),
        "flag {flag} out of range for a {size_bytes}-byte flag set"
    );
}

/// `flag_has(flags, size, flag)` (z-bitflag.c) on a whole flag array; the
/// function behind `of_has`, `kf_has`, `rf_has` and friends. `FLAG_END`
/// (0) gives false, as in C; a flag outside the array panics (see
/// `check_flag_has`). A transcription of the C function (no FFI call, so
/// it is testable without the engine): for a checked flag it reads the
/// same byte and bit (`FLAG_OFFSET`, `FLAG_BINARY`).
pub fn flag_has(flags: &[u8], flag: i32) -> bool {
    check_flag_has(flags.len(), flag);
    if flag == FLAG_END {
        return false;
    }
    let f = (flag - FLAG_START) as usize;
    flags[f / 8] & (1 << (f % 8)) != 0
}

/// `OF_SIZE`: bytes in an object flag set.
pub const OF_SIZE: usize = 5;

/// `player_flags(player, f)`: the player's race and class object flags.
pub fn player_flags() -> [u8; OF_SIZE] {
    let mut f = [0u8; OF_SIZE];
    // SAFETY: `player` lives for the game; player_flags writes exactly
    // OF_SIZE bytes into f (a copy of the race's flags, then unions).
    unsafe {
        crate::headers::player_h::player_flags(crate::headers::player_h::player, f.as_mut_ptr())
    };
    f
}

/// `player->race->el_info[elem].res_level`.
pub fn player_race_res_level(elem: u32) -> i16 {
    // SAFETY: the race is set at birth and points into the race table,
    // which lives for the game.
    let race = unsafe { &*player_ref().race };
    race.el_info[elem as usize].res_level
}

/// `obj_kind_can_browse(&k_info[kind])`: the player's class can browse
/// books of this kind.
pub fn obj_kind_can_browse(kind: usize) -> bool {
    let k = kind_ptr(kind);
    // SAFETY: k points into k_info (index checked by kind_ptr); the callee only
    // reads the kind and the player's class.
    unsafe { crate::headers::obj_util_h::obj_kind_can_browse(k) }
}

// ---------------------------------------------------------------------------
// Writable files, paths and the clock (borg-log.c)

/// How `file_open()` opens a file for writing.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum WriteMode {
    /// `MODE_WRITE`: create or truncate.
    Write,
    /// `MODE_APPEND`: create or append.
    Append,
}

/// An engine file opened for writing by `file_open()` (`ang_file *`). Not
/// `Copy`: `close()` consumes it. Dropping it without closing leaks it.
pub struct OpenFile(*mut crate::headers::z_file_h::ang_file);

/// `file_open(path, mode, FTYPE_TEXT)`; `None` where C gets NULL.
pub fn file_open(path: &[u8], mode: WriteMode) -> Option<OpenFile> {
    use crate::headers::z_file_h as z;
    let c = CString::new(path).expect("no NUL in path");
    let m = match mode {
        WriteMode::Write => z::MODE_WRITE,
        WriteMode::Append => z::MODE_APPEND,
    };
    // SAFETY: c is a valid C string for the call; the result is NULL or a
    // file the engine allocated.
    let f = unsafe { z::file_open(c.as_ptr(), m, z::FTYPE_TEXT) };
    (!f.is_null()).then_some(OpenFile(f))
}

impl OpenFile {
    /// `file_put(f, s)` (no NUL in `s`).
    pub fn put(&mut self, s: &[u8]) -> bool {
        let c = CString::new(s).expect("no NUL in file text");
        // SAFETY: self.0 is open (only close() ends it, by value); c is a
        // valid C string.
        unsafe { crate::headers::z_file_h::file_put(self.0, c.as_ptr()) }
    }

    /// `file_close(f)`.
    pub fn close(self) -> bool {
        // SAFETY: self.0 is open and is not used again (self is consumed).
        unsafe { crate::headers::z_file_h::file_close(self.0) }
    }
}

/// `path_build(buf, len, ANGBAND_DIR_ARCHIVE, leaf)`: the path (bytes up
/// to the NUL) as built in a buffer of `len` bytes.
pub fn path_build_archive(len: usize, leaf: &[u8]) -> Vec<u8> {
    // SAFETY: plain read of a pointer global set up at startup.
    path_build_in(unsafe { crate::headers::init_h::ANGBAND_DIR_ARCHIVE }, len, leaf)
}

/// `path_build(buf, len, ANGBAND_DIR_USER, leaf)`: the path (bytes up to
/// the NUL) as built in a buffer of `len` bytes.
pub fn path_build_user(len: usize, leaf: &[u8]) -> Vec<u8> {
    // SAFETY: plain read of a pointer global set up at startup.
    path_build_in(unsafe { crate::headers::init_h::ANGBAND_DIR_USER }, len, leaf)
}

/// `path_build(buf, len, dir, leaf)` for one of the `ANGBAND_DIR_*` globals.
fn path_build_in(dir: *mut c_char, len: usize, leaf: &[u8]) -> Vec<u8> {
    assert!(len >= 1);
    let leaf = CString::new(leaf).expect("no NUL in file name");
    let mut buf = vec![0u8; len];
    // SAFETY: buf has len bytes; dir is an `ANGBAND_DIR_*` C string set up
    // at startup (or NULL, which path_build accepts); leaf is a C string.
    unsafe {
        crate::headers::z_file_h::path_build(buf.as_mut_ptr() as *mut c_char, len, dir, leaf.as_ptr());
    }
    let n = buf.iter().position(|&b| b == 0).unwrap_or(len);
    buf.truncate(n);
    buf
}

/// An engine file opened for reading by `file_open(path, MODE_READ, -1)`
/// (`ang_file *`). Not `Copy`: `close()` consumes it. Dropping it without
/// closing leaks it.
pub struct ReadFile(*mut crate::headers::z_file_h::ang_file);

/// `file_open(path, MODE_READ, -1)`; `None` where C gets NULL.
pub fn file_open_read(path: &[u8]) -> Option<ReadFile> {
    use crate::headers::z_file_h as z;
    let c = CString::new(path).expect("no NUL in path");
    // SAFETY: c is a valid C string for the call; the file type is not
    // used for reading (C passes -1); the result is NULL or a file the
    // engine allocated.
    let f = unsafe { z::file_open(c.as_ptr(), z::MODE_READ, u32::MAX) };
    (!f.is_null()).then_some(ReadFile(f))
}

impl ReadFile {
    /// `file_getl(f, buf, len)`, as [`file_getl`].
    pub fn getl(&mut self, buf: &mut [u8], len: usize) -> bool {
        // AngFile contract: self.0 is open (only close() ends it, by value)
        // and the handle does not outlive this call.
        file_getl(AngFile(self.0), buf, len)
    }

    /// `file_close(f)`.
    pub fn close(self) -> bool {
        // SAFETY: self.0 is open and is not used again (self is consumed).
        unsafe { crate::headers::z_file_h::file_close(self.0) }
    }
}

/// `time(NULL)`.
pub fn time_now() -> i64 {
    // SAFETY: time() accepts a NULL out-pointer.
    unsafe { crate::headers::time_h::time(core::ptr::null_mut()) }
}

/// `strftime(buf, max, fmt, localtime(&t))`: the bytes written (empty
/// where strftime returns 0).
pub fn strftime_local(t: i64, max: usize, fmt: &CStr) -> Vec<u8> {
    let mut buf = vec![0u8; max];
    // SAFETY: localtime reads t and returns its static struct (or NULL,
    // checked); strftime writes at most max bytes into buf.
    let n = unsafe {
        let tm = crate::headers::time_h::localtime(&t);
        assert!(!tm.is_null(), "localtime failed");
        crate::headers::time_h::strftime(buf.as_mut_ptr() as *mut c_char, max, fmt.as_ptr(), tm)
    };
    buf.truncate(n);
    buf
}

/// `quit_fmt("%s", text)`: end the game with a message. Never returns.
pub fn quit(text: &[u8]) -> ! {
    let c = CString::new(text).expect("no NUL in message");
    // SAFETY: format string and argument are valid C strings.
    unsafe { crate::headers::z_form_h::quit_fmt(c"%s".as_ptr(), c.as_ptr()) }
    unreachable!("quit() returned")
}

/// `get_string(prompt, buf, len)`: let the player edit the NUL-terminated
/// text in `buf` (at most `len` bytes, NUL included). False on escape.
/// Waits for keys (lends the state).
pub fn get_string(st: &mut BorgState, prompt: &CStr, buf: &mut [u8], len: usize) -> bool {
    assert!(len >= 1 && len <= buf.len() && buf.contains(&0));
    // SAFETY: buf is NUL-terminated and has at least len bytes; get_string
    // writes at most len bytes.
    lend(st, || unsafe {
        crate::headers::game_input_h::get_string(
            prompt.as_ptr(),
            buf.as_mut_ptr() as *mut c_char,
            len,
        )
    })
}

// ---------------------------------------------------------------------------
// Player summary, timed effects, objects (borg-log.c)

/// The bytes of a C string from the game data ("" for NULL, as `strnfmt`
/// prints it).
fn c_bytes(p: *const c_char) -> &'static [u8] {
    if p.is_null() {
        return b"";
    }
    // SAFETY: non-NULL C strings passed here are game data (names) that
    // live for the game.
    unsafe { CStr::from_ptr(p).to_bytes() }
}

/// The bytes of a C char array up to its NUL.
fn c_array_bytes(a: &[c_char]) -> Vec<u8> {
    a.iter().take_while(|&&c| c != 0).map(|&c| c as u8).collect()
}

/// The player fields that the death log and map dump print.
#[derive(Clone, Debug)]
pub struct PlayerSummary {
    /// `player->full_name`
    pub full_name: Vec<u8>,
    /// `player->race->name` (NULL reads as "", as in `strnfmt`)
    pub race_name: &'static [u8],
    /// `player->class->name`
    pub class_name: &'static [u8],
    pub lev: i16,
    pub max_lev: i16,
    pub max_exp: i32,
    pub max_depth: i16,
    pub au: i32,
    pub depth: i16,
    /// `player->died_from`
    pub died_from: Vec<u8>,
    pub word_recall: i16,
}

/// Read the player's summary fields.
pub fn player_summary() -> PlayerSummary {
    let p = player_ref();
    // SAFETY: race and class point into the static race/class lists (set
    // at birth, live for the game).
    let (race, class) = unsafe { ((*p.race).name, (*p.class).name) };
    PlayerSummary {
        full_name: c_array_bytes(&p.full_name),
        race_name: c_bytes(race),
        class_name: c_bytes(class),
        lev: p.lev,
        max_lev: p.max_lev,
        max_exp: p.max_exp,
        max_depth: p.max_depth,
        au: p.au,
        depth: p.depth,
        died_from: c_array_bytes(&p.died_from),
        word_recall: p.word_recall,
    }
}

/// `player->timed[idx]` (idx < TMD_MAX).
pub fn player_timed(idx: u32) -> i16 {
    assert!(idx < crate::headers::player_timed_h::TMD_MAX, "timed index out of range");
    // SAFETY: timed is allocated with TMD_MAX entries with the player.
    unsafe { *player_ref().timed.add(idx as usize) }
}

/// `player_of_has(player, flag)`: an object flag on the player.
pub fn player_of_has(flag: u32) -> bool {
    // C: `of_has(p->state.flags, flag)`, i.e. `flag_has` on OF_SIZE bytes.
    check_flag_has(player_ref().state.flags.len(), flag as i32);
    // SAFETY: reads the player's state only; the flag is FLAG_END or inside
    // the OF_SIZE-byte array (checked above).
    unsafe {
        crate::headers::player_util_h::player_of_has(crate::headers::player_h::player, flag as i32)
    }
}

/// The game turn (`turn`).
pub fn game_turn() -> i32 {
    // SAFETY: plain global, engine thread.
    unsafe { crate::headers::game_world_h::turn }
}

/// `object_desc(buf, 80, obj, mode, player)` (bytes up to the NUL).
fn object_desc80(obj: *const crate::headers::object_h::object, mode: u32) -> Vec<u8> {
    let mut buf = [0u8; 80];
    // SAFETY: obj is NULL or a live object of the player or a store
    // (callers); object_desc writes at most 80 bytes, NUL included.
    unsafe {
        crate::headers::obj_desc_h::object_desc(
            buf.as_mut_ptr() as *mut c_char,
            buf.len(),
            obj,
            mode,
            crate::headers::player_h::player,
        );
    }
    let n = buf.iter().position(|&b| b == 0).unwrap_or(buf.len());
    buf[..n].to_vec()
}

/// `player->body.count`: the number of equipment slots.
pub fn body_count() -> u16 {
    player_ref().body.count
}

/// `object_desc()` of `player->body.slots[i].obj` (NULL: "(nothing)").
pub fn equip_desc(i: usize, mode: u32) -> Vec<u8> {
    let body = &player_ref().body;
    assert!(i < usize::from(body.count), "equipment slot out of range");
    // SAFETY: slots has body.count entries (checked).
    let obj = unsafe { (*body.slots.add(i)).obj };
    object_desc80(obj, mode)
}

/// `object_desc()` of `player->upkeep->quiver[i]` (i < quiver_size).
pub fn quiver_desc(i: usize, mode: u32) -> Vec<u8> {
    assert!(i < usize::from(z_info().quiver_size), "quiver slot out of range");
    // SAFETY: the quiver is allocated with z_info->quiver_size entries
    // with the player's upkeep.
    let obj = unsafe { *(*player_ref().upkeep).quiver.add(i) };
    object_desc80(obj, mode)
}

/// `store_stock_list(&stores[store], list, z_info->store_inven_max)`, then
/// `object_desc()` of each list entry, in order.
pub fn store_stock_descs(store: usize, mode: u32) -> Vec<Vec<u8>> {
    let n = usize::from(z_info().store_inven_max);
    assert!(store < usize::from(z_info().store_max), "store index out of range");
    let mut list: Vec<*mut crate::headers::object_h::object> = vec![core::ptr::null_mut(); n];
    // SAFETY: stores has store_max entries (checked); list has room for
    // the n entries store_stock_list fills.
    unsafe {
        crate::headers::store_h::store_stock_list(
            crate::headers::store_h::stores.add(store),
            list.as_mut_ptr(),
            n as i32,
        );
    }
    list.iter().map(|&o| object_desc80(o, mode)).collect()
}

// ---------------------------------------------------------------------------
// Store stock (borg-store.c)

/// An object in a store's stock (an entry of `store_stock_list`). Bound
/// to the borrow of the state's `access` token (see "Engine object
/// handles"): the stock only changes while the game runs.
#[derive(Clone, Copy)]
pub struct StoreObject<'a>(*const crate::headers::object_h::object, PhantomData<&'a Access>);

/// `store_stock_list(&stores[store], list, z_info->store_inven_max)`: the
/// entries before the first NULL, in order.
pub fn store_stock(access: &Access, store: usize) -> Vec<StoreObject<'_>> {
    check_access(access);
    let n = usize::from(z_info().store_inven_max);
    assert!(store < usize::from(z_info().store_max), "store index out of range");
    let mut list: Vec<*mut crate::headers::object_h::object> = vec![core::ptr::null_mut(); n];
    // SAFETY: stores has store_max entries (checked); list has room for
    // the n entries store_stock_list fills.
    unsafe {
        crate::headers::store_h::store_stock_list(
            crate::headers::store_h::stores.add(store),
            list.as_mut_ptr(),
            n as i32,
        );
    }
    list.iter().take_while(|o| !o.is_null()).map(|&o| StoreObject(o, PhantomData)).collect()
}

impl StoreObject<'_> {

    /// `object_desc(buf, 120, obj, mode, player)` (bytes up to the NUL).
    pub fn desc120(self, mode: u32) -> Vec<u8> {
        let mut buf = [0u8; 120];
        // SAFETY: self is a live store object (see the type); object_desc
        // writes at most 120 bytes, NUL included.
        unsafe {
            crate::headers::obj_desc_h::object_desc(
                buf.as_mut_ptr() as *mut c_char,
                buf.len(),
                self.0,
                mode,
                crate::headers::player_h::player,
            );
        }
        let n = buf.iter().position(|&b| b == 0).unwrap_or(buf.len());
        buf[..n].to_vec()
    }

    /// `tval_can_have_charges(obj)`
    pub fn can_have_charges(self) -> bool {
        // SAFETY: self is a live store object; the call only reads it.
        unsafe { crate::headers::obj_tval_h::tval_can_have_charges(self.0) }
    }

    /// C computes `value * qty`, `pval * qty` and divides by
    /// `obj->number` in plain `int` (obj-power.c): panics outside the domain
    /// where that is defined for any item value. The Borg only prices one
    /// item (`qty` 1) of a ware, which always has `number >= 1`.
    fn check_value_qty(self, qty: i32) {
        assert!((0..=1).contains(&qty), "object value of {qty} items: C's value * qty may overflow");
        // SAFETY: self is a live store object (lifetime-bound handle); a
        // plain field read.
        let number = unsafe { (*self.0).number };
        assert!(number > 0, "object value of an empty stack: C divides by obj->number");
    }

    /// `object_value_real(obj, qty)`; `qty` 0 or 1 (see `check_value_qty`).
    pub fn value_real(self, qty: i32) -> i32 {
        self.check_value_qty(qty);
        // SAFETY: self is a live store object; the call only reads it; qty
        // and number are in the domain checked above.
        unsafe { crate::headers::obj_power_h::object_value_real(self.0, qty) }
    }

    /// `object_value(obj, qty)`; `qty` 0 or 1 (see `check_value_qty`).
    pub fn value(self, qty: i32) -> i32 {
        self.check_value_qty(qty);
        // SAFETY: self is a live store object; the call only reads it; qty
        // and number are in the domain checked above.
        unsafe { crate::headers::obj_power_h::object_value(self.0, qty) }
    }
}

/// `stores[store].owner->max_cost`
pub fn store_owner_max_cost(store: usize) -> i32 {
    assert!(store < usize::from(z_info().store_max), "store index out of range");
    // SAFETY: stores has store_max entries (checked), each with an owner
    // set when the stores are created.
    unsafe {
        let owner = (*crate::headers::store_h::stores.add(store)).owner;
        assert!(!owner.is_null(), "store without owner");
        (*owner).max_cost
    }
}

/// `f_info[feat].shopnum`
pub fn feat_shopnum(feat: usize) -> u8 {
    assert!(feat < crate::headers::cave_h::FEAT_MAX as usize, "feature out of range");
    // SAFETY: f_info is allocated with FEAT_MAX entries when the game data
    // is loaded (index checked).
    unsafe { (*crate::headers::cave_h::f_info.add(feat)).shopnum }
}

/// `f_info[stores[store].feat].name` (bytes up to the NUL).
pub fn store_feat_name(store: usize) -> Vec<u8> {
    assert!(store < usize::from(z_info().store_max), "store index out of range");
    // SAFETY: stores has store_max entries (checked); each store's feature
    // is a valid f_info index whose name is a NUL-terminated string set
    // when the game data is loaded.
    unsafe {
        let feat = (*crate::headers::store_h::stores.add(store)).feat;
        assert!(
            (0..crate::headers::cave_h::FEAT_MAX as i32).contains(&feat),
            "feature out of range"
        );
        let name = (*crate::headers::cave_h::f_info.add(feat as usize)).name;
        assert!(!name.is_null(), "feature without a name");
        std::ffi::CStr::from_ptr(name).to_bytes().to_vec()
    }
}

/// `square_shopnum(cave, player->grid)`: the store at the player's grid,
/// or -1.
pub fn player_square_shopnum() -> i32 {
    // SAFETY: `cave` is the current level and the player's grid is inside
    // it; the call only reads the square.
    unsafe { crate::headers::cave_h::square_shopnum(headers::cave_h::cave, player_ref().grid) }
}

/// `player->opts.opt[opt]` (the `OPT(player, ...)` macro).
pub fn player_opt(opt: u32) -> bool {
    player_ref().opts.opt[opt as usize]
}

// ---------------------------------------------------------------------------
// Map and messages (borg-log.c)

/// `square_monster(cave, loc(x, y))->midx`, or `None` for no monster.
pub fn square_monster_midx(x: i32, y: i32) -> Option<i32> {
    // SAFETY: `cave` is the current level; square_monster checks the
    // bounds itself (NULL outside); the result is NULL or a live monster.
    unsafe {
        let m = crate::headers::cave_h::square_monster(crate::headers::cave_h::cave, loc { x, y });
        (!m.is_null()).then(|| (*m).midx)
    }
}

/// `f_info[feat].d_char`.
pub fn feat_d_char(feat: usize) -> wchar_t {
    assert!(feat < crate::headers::cave_h::FEAT_MAX as usize, "feature index out of range");
    // SAFETY: f_info is allocated with FEAT_MAX entries when the game data
    // is loaded and lives for the game (index checked).
    unsafe { (*crate::headers::cave_h::f_info.add(feat)).d_char }
}

/// `messages_num()`
pub fn messages_num() -> u16 {
    // SAFETY: plain call.
    unsafe { crate::headers::message_h::messages_num() }
}

/// `message_str(age)` (bytes up to the NUL; "" for no such message).
pub fn message_str(age: u16) -> Vec<u8> {
    // SAFETY: message_str returns a string owned by the message list (or
    // a literal); it is copied out at once.
    unsafe { c_bytes(crate::headers::message_h::message_str(age)).to_vec() }
}

// ---------------------------------------------------------------------------
// More terminal output (borg-log.c)

/// `Term_putstr(x, y, n, a, s)` (no NUL in `s`).
pub fn term_putstr(x: i32, y: i32, n: i32, a: i32, s: &[u8]) {
    let c = CString::new(s).expect("no NUL in term text");
    // SAFETY: c is a valid C string for the call.
    unsafe {
        crate::headers::ui_term_h::Term_putstr(x, y, n, a, c.as_ptr());
    }
}

/// `Term_putch(x, y, a, c)`.
pub fn term_putch(x: i32, y: i32, a: i32, c: wchar_t) {
    // SAFETY: plain call on the active term; the engine checks the
    // position (`Term_gotoxy`).
    unsafe {
        crate::headers::ui_term_h::Term_putch(x, y, a, c);
    }
}

/// `Term_clear()`.
pub fn term_clear() {
    // SAFETY: plain call on the active term.
    unsafe {
        crate::headers::ui_term_h::Term_clear();
    }
}

/// `prt(s, row, col)` (no NUL in `s`).
pub fn prt(s: &[u8], row: i32, col: i32) {
    let c = CString::new(s).expect("no NUL in term text");
    // SAFETY: c is a valid C string for the call.
    unsafe { crate::headers::ui_output_h::prt(c.as_ptr(), row, col) }
}

// ---------------------------------------------------------------------------
// Carried objects (borg-inventory.c)

/// A game object the player carries (pack, equipment or quiver) that has
/// a kind. Opaque: the pointer stays on the FFI side (`engine`). Bound to
/// the borrow of the state's `access` token (see "Engine object handles").
#[derive(Clone, Copy)]
pub struct CarriedObject<'a>(*const crate::headers::object_h::object, PhantomData<&'a Access>);

impl CarriedObject<'_> {

    /// `obj && obj->kind`
    /// (`access` is checked by the caller, before it reads the slot.)
    fn with_kind<'a>(_access: &'a Access, obj: *const crate::headers::object_h::object) -> Option<CarriedObject<'a>> {
        if obj.is_null() {
            return None;
        }
        // SAFETY: a non-NULL slot of the player's pack, body or quiver
        // points at a live object.
        let kind = unsafe { (*obj).kind };
        (!kind.is_null()).then_some(CarriedObject(obj, PhantomData))
    }
}

/// `player->upkeep->inven[i]` (i < pack_size), if it is an object with a
/// kind.
pub fn inven_object(access: &Access, i: usize) -> Option<CarriedObject<'_>> {
    check_access(access);
    assert!(i < usize::from(z_info().pack_size), "pack slot out of range");
    // SAFETY: inven is allocated with pack_size + 1 entries with the
    // player's upkeep (index checked).
    CarriedObject::with_kind(access, unsafe { *(*player_ref().upkeep).inven.add(i) })
}

/// `player->body.slots[i].obj` (i < body.count), if it is an object with a
/// kind.
pub fn equip_object(access: &Access, i: usize) -> Option<CarriedObject<'_>> {
    check_access(access);
    let body = &player_ref().body;
    assert!(i < usize::from(body.count), "equipment slot out of range");
    // SAFETY: slots has body.count entries (checked).
    CarriedObject::with_kind(access, unsafe { (*body.slots.add(i)).obj })
}

/// `player->upkeep->quiver[i]` (i < quiver_size), if it is an object with
/// a kind.
pub fn quiver_object(access: &Access, i: usize) -> Option<CarriedObject<'_>> {
    check_access(access);
    assert!(i < usize::from(z_info().quiver_size), "quiver slot out of range");
    // SAFETY: the quiver is allocated with z_info->quiver_size entries
    // with the player's upkeep (index checked).
    CarriedObject::with_kind(access, unsafe { *(*player_ref().upkeep).quiver.add(i) })
}

/// `object_desc(buf, 256, obj, ODESC_FULL, player)` (bytes up to the NUL).
pub fn object_desc_full(obj: CarriedObject<'_>) -> Vec<u8> {
    let mut buf = [0u8; 256];
    // SAFETY: obj is a live carried object (its lifetime keeps it from
    // crossing a call that runs the game); object_desc writes at most
    // 256 bytes, NUL included.
    unsafe {
        crate::headers::obj_desc_h::object_desc(
            buf.as_mut_ptr() as *mut c_char,
            buf.len(),
            obj.0,
            crate::headers::obj_desc_h::ODESC_FULL,
            crate::headers::player_h::player,
        );
    }
    let n = buf.iter().position(|&b| b == 0).unwrap_or(buf.len());
    buf[..n].to_vec()
}
// Danger (borg-danger.c): cave squares, game tables and monster blows

/// `square_in_bounds(cave, loc(x, y))` on the current level.
pub fn square_in_bounds(x: i32, y: i32) -> bool {
    // SAFETY: `cave` is the current level, valid while the game runs; the
    // function only compares the grid with the level's size.
    unsafe { headers::cave_h::square_in_bounds(headers::cave_h::cave, loc { x, y }) }
}

/// `square_isvault(cave, loc(x, y))` on the current level. Panics if the
/// grid is out of bounds (the engine's own `assert` is compiled out in the
/// release build, so C would read outside the level).
pub fn square_isvault(x: i32, y: i32) -> bool {
    assert!(square_in_bounds(x, y), "grid out of bounds");
    // SAFETY: `cave` is the current level, valid while the game runs; the
    // grid is in bounds (checked above).
    unsafe { headers::cave_h::square_isvault(headers::cave_h::cave, loc { x, y }) }
}

/// `square_isno_teleport(cave, loc(x, y))` on the current level: the
/// grid forbids teleporting (borg-escape.c). Panics if the grid is out of
/// bounds (the engine would read outside the level).
pub fn square_isno_teleport(x: i32, y: i32) -> bool {
    assert!(square_in_bounds(x, y), "grid out of bounds");
    // SAFETY: `cave` is the current level, valid while the game runs; the
    // grid is in bounds (checked above).
    unsafe { headers::cave_h::square_isno_teleport(headers::cave_h::cave, loc { x, y }) }
}

/// `adj_dex_safe[i]` (player-calcs.c); panics out of range.
pub fn adj_dex_safe(i: usize) -> i32 {
    // SAFETY: a constant table of the engine; nothing writes it.
    unsafe { headers::player_calcs_h::adj_dex_safe[i] }
}

/// `adj_str_hold[i]` (player-calcs.c); panics out of range.
pub fn adj_str_hold(i: usize) -> i32 {
    // SAFETY: a constant table of the engine; nothing writes it.
    unsafe { headers::player_calcs_h::adj_str_hold[i] }
}

/// `extract_energy[speed]` (game-world.c); panics out of range.
pub fn extract_energy(speed: usize) -> u8 {
    // SAFETY: a constant table of the engine; nothing writes it.
    unsafe { headers::game_world_h::extract_energy[speed] }
}

/// `square_isglow(cave, loc(x, y))` on the current level. Panics if the
/// grid is out of bounds (as `square_isvault`).
pub fn square_isglow(x: i32, y: i32) -> bool {
    assert!(square_in_bounds(x, y), "grid out of bounds");
    // SAFETY: `cave` is the current level, valid while the game runs; the
    // grid is in bounds (checked above).
    unsafe { headers::cave_h::square_isglow(headers::cave_h::cave, loc { x, y }) }
}

/// `square_islit(cave, loc(x, y))` on the current level. Panics if the
/// grid is out of bounds (as `square_isglow`).
pub fn square_islit(x: i32, y: i32) -> bool {
    assert!(square_in_bounds(x, y), "grid out of bounds");
    // SAFETY: `cave` is the current level, valid while the game runs; the
    // grid is in bounds (checked above).
    unsafe { headers::cave_h::square_islit(headers::cave_h::cave, loc { x, y }) }
}

/// `panel_contains(y, x)` (game-input.c, `textui_panel_contains`): is the
/// grid on the current map panel? The arguments are C `unsigned int`s (a
/// negative value wraps, as in C).
pub fn panel_contains(y: i32, x: i32) -> bool {
    // SAFETY: plain call; the hook reads the main term's offsets and size.
    unsafe { headers::game_input_h::panel_contains(y as u32, x as u32) }
}

/// `feat_is_trap_holding(feat)` (cave.c): can the terrain feature hold a
/// trap?
pub fn feat_is_trap_holding(feat: i32) -> bool {
    assert!((0..crate::headers::cave_h::FEAT_MAX as i32).contains(&feat), "feature out of range");
    // SAFETY: plain call; reads `f_info[feat]` (index checked above).
    unsafe { headers::cave_h::feat_is_trap_holding(feat) }
}

/// `ddx_ddd[i]` (cave.c): the x offsets of the 8 directions (and 0).
pub fn ddx_ddd(i: usize) -> i16 {
    // SAFETY: a constant table of the engine; nothing writes it.
    unsafe { headers::cave_h::ddx_ddd[i] }
}

/// `ddy_ddd[i]` (cave.c): the y offsets of the 8 directions (and 0).
pub fn ddy_ddd(i: usize) -> i16 {
    // SAFETY: a constant table of the engine; nothing writes it.
    unsafe { headers::cave_h::ddy_ddd[i] }
}

/// `ddd[i]` (cave.c): the keypad directions in `ddx_ddd` order (and 5).
pub fn ddd(i: usize) -> i16 {
    // SAFETY: a constant table of the engine; nothing writes it.
    unsafe { headers::cave_h::ddd[i] }
}

/// `ddx[dir]` (cave.c): the x offset of keypad direction `dir` (0..9).
pub fn ddx(dir: usize) -> i16 {
    // SAFETY: a constant table of the engine; nothing writes it.
    unsafe { headers::cave_h::ddx[dir] }
}

/// `ddy[dir]` (cave.c): the y offset of keypad direction `dir` (0..9).
pub fn ddy(dir: usize) -> i16 {
    // SAFETY: a constant table of the engine; nothing writes it.
    unsafe { headers::cave_h::ddy[dir] }
}

/// `feat_is_shop(feat)` (cave-square.c): the terrain is a shop entrance.
pub fn feat_is_shop(feat: i32) -> bool {
    assert!((0..crate::headers::cave_h::FEAT_MAX as i32).contains(&feat), "feature out of range");
    // SAFETY: plain call; reads `f_info[feat]` (index checked above).
    unsafe { headers::cave_h::feat_is_shop(feat) }
}

/// One blow of a monster race (`r_info[r_idx].blow[k]`).
#[derive(Clone, Copy)]
pub struct RaceBlow {
    /// `effect->name`
    pub effect_name: &'static CStr,
    /// `dice.dice`
    pub dice: i32,
    /// `dice.sides`
    pub sides: i32,
}

/// `r_info[r_idx].blow[k]` for `k < z_info->mon_blows_max`, or `None` when
/// the blow has no method (the end of the race's blows).
pub fn race_blow(r_idx: usize, k: usize) -> Option<RaceBlow> {
    assert!(k < usize::from(z_info().mon_blows_max));
    let blows = race(r_idx).blow;
    assert!(!blows.is_null(), "race {r_idx} has no blow array");
    // SAFETY: a race's blow array has z_info->mon_blows_max entries
    // (checked above), allocated when the game data is loaded and kept
    // for the game. A blow with a method has an effect with a name (the
    // C code dereferences both without a check; asserted here).
    unsafe {
        let b = &*blows.add(k);
        if b.method.is_null() {
            return None;
        }
        assert!(!b.effect.is_null(), "race {r_idx} blow {k} has no effect");
        let name = (*b.effect).name;
        assert!(!name.is_null(), "race {r_idx} blow {k} effect has no name");
        Some(RaceBlow { effect_name: CStr::from_ptr(name), dice: b.dice.dice, sides: b.dice.sides })
    }
}

// ---------------------------------------------------------------------------
// Game objects and object tables (borg-item-analyze.c)

use crate::headers::object_h::{element_info, object, object_kind};

/// A live game object (`const struct object *`): a carried object, a
/// store ware, or the player's knowledge of one (`obj->known`). Opaque:
/// the pointer stays on the FFI side. Made only from a `CarriedObject` or
/// a `StoreObject` (or as their `known`), with the same lifetime (see
/// "Engine object handles").
#[derive(Clone, Copy)]
pub struct GameObject<'a>(*const object, PhantomData<&'a Access>);

impl<'a> From<StoreObject<'a>> for GameObject<'a> {
    fn from(o: StoreObject<'a>) -> Self {
        GameObject(o.0, PhantomData)
    }
}

impl<'a> From<CarriedObject<'a>> for GameObject<'a> {
    fn from(o: CarriedObject<'a>) -> Self {
        GameObject(o.0, PhantomData)
    }
}

impl<'a> GameObject<'a> {

    fn obj(&self) -> &object {
        // SAFETY: the handle was made from a live object (a CarriedObject
        // or StoreObject, or its `known`), and its lifetime keeps it from
        // crossing an engine call that runs the game (the only place
        // objects are freed). The reference is only used for the field
        // read at hand, never across an engine call.
        unsafe { &*self.0 }
    }

    /// The object's kind (panics where C would dereference NULL). Private:
    /// the reference into k_info (which the engine writes, L5) is only used
    /// for one field read, never across an engine call.
    fn kind(&self) -> &object_kind {
        let k = self.obj().kind;
        assert!(!k.is_null(), "object without a kind");
        // SAFETY: non-NULL kinds point into k_info, which lives for the game.
        unsafe { &*k }
    }

    /// `obj->kind != NULL`
    pub fn has_kind(self) -> bool {
        !self.obj().kind.is_null()
    }
    /// `obj->number`
    pub fn number(self) -> u8 {
        self.obj().number
    }
    /// `obj->tval`
    pub fn tval(self) -> u8 {
        self.obj().tval
    }
    /// `obj->sval`
    pub fn sval(self) -> u8 {
        self.obj().sval
    }
    /// `obj->pval`
    pub fn pval(self) -> i16 {
        self.obj().pval
    }
    /// `obj->timeout`
    pub fn timeout(self) -> i16 {
        self.obj().timeout
    }
    /// `obj->ac`
    pub fn ac(self) -> i16 {
        self.obj().ac
    }
    /// `obj->dd`
    pub fn dd(self) -> u8 {
        self.obj().dd
    }
    /// `obj->ds`
    pub fn ds(self) -> u8 {
        self.obj().ds
    }
    /// `obj->to_h`
    pub fn to_h(self) -> i16 {
        self.obj().to_h
    }
    /// `obj->to_d`
    pub fn to_d(self) -> i16 {
        self.obj().to_d
    }
    /// `obj->to_a`
    pub fn to_a(self) -> i16 {
        self.obj().to_a
    }
    /// `obj->modifiers`
    pub fn modifiers(self) -> [i16; 16] {
        self.obj().modifiers
    }
    /// `obj->el_info`
    pub fn el_info(self) -> [element_info; 25] {
        self.obj().el_info
    }

    /// `obj->known`, if not NULL.
    pub fn known(self) -> Option<GameObject<'a>> {
        let k = self.obj().known;
        (!k.is_null()).then_some(GameObject(k, PhantomData))
    }

    /// `object_fully_known(obj)`
    pub fn fully_known(self) -> bool {
        // SAFETY: reads the live object (and its known version) only.
        unsafe { crate::headers::obj_knowledge_h::object_fully_known(self.0) }
    }
    /// `object_runes_known(obj)`
    pub fn runes_known(self) -> bool {
        // SAFETY: reads the live object (and its known version) only.
        unsafe { crate::headers::obj_knowledge_h::object_runes_known(self.0) }
    }
    /// `object_flavor_is_aware(obj)` (the object must have a kind).
    pub fn flavor_is_aware(self) -> bool {
        // SAFETY: reads the object's kind only.
        unsafe { crate::headers::obj_knowledge_h::object_flavor_is_aware(self.0) }
    }
    /// `object_flags_known(obj, flags)`
    pub fn flags_known(self) -> [u8; OF_SIZE] {
        let mut f = [0u8; OF_SIZE];
        // SAFETY: reads the live object (C dereferences `obj->known`, as
        // here); writes exactly OF_SIZE bytes into f.
        unsafe { crate::headers::obj_util_h::object_flags_known(self.0, f.as_mut_ptr()) };
        f
    }
    /// `object_weight_one(obj)`
    pub fn weight_one(self) -> i16 {
        // SAFETY: reads the live object and the curse table only.
        unsafe { crate::headers::obj_util_h::object_weight_one(self.0) }
    }

    /// `obj->kind->kidx`
    pub fn kind_kidx(self) -> u32 {
        self.kind().kidx
    }
    /// `obj->kind->level`
    pub fn kind_level(self) -> i32 {
        self.kind().level
    }
    /// `obj->kind->cost`
    pub fn kind_cost(self) -> i32 {
        self.kind().cost
    }
    /// `randcalc(obj->kind->time, 0, MINIMISE)` (draws no random numbers).
    pub fn kind_time_min(self) -> i32 {
        let t = self.kind().time;
        // SAFETY: pure arithmetic on a copied value; MINIMISE uses no RNG.
        unsafe { crate::headers::z_rand_h::randcalc(t, 0, crate::headers::z_rand_h::MINIMISE) }
    }

    /// `obj->curses[0 .. curse_max].power`, or `None` for NULL `curses`.
    pub fn curse_powers(self) -> Option<Vec<i32>> {
        let p = self.obj().curses;
        if p.is_null() {
            return None;
        }
        let n = usize::from(z_info().curse_max);
        // SAFETY: a non-NULL curses array has z_info->curse_max entries.
        let c = unsafe { std::slice::from_raw_parts(p, n) };
        Some(c.iter().map(|c| c.power).collect())
    }
    /// `obj->slays[0 .. slay_max]`, or `None` for NULL `slays`.
    pub fn slays(self) -> Option<Vec<bool>> {
        let p = self.obj().slays;
        if p.is_null() {
            return None;
        }
        let n = usize::from(z_info().slay_max);
        // SAFETY: a non-NULL slays array has z_info->slay_max entries.
        Some(unsafe { std::slice::from_raw_parts(p, n) }.to_vec())
    }
    /// `obj->brands[0 .. brand_max]`, or `None` for NULL `brands`.
    pub fn brands(self) -> Option<Vec<bool>> {
        let p = self.obj().brands;
        if p.is_null() {
            return None;
        }
        let n = usize::from(z_info().brand_max);
        // SAFETY: a non-NULL brands array has z_info->brand_max entries.
        Some(unsafe { std::slice::from_raw_parts(p, n) }.to_vec())
    }
    /// `obj->activation->index`, or `None` for no activation.
    pub fn activation_index(self) -> Option<i32> {
        let a = self.obj().activation;
        // SAFETY: a non-NULL activation points into the activation list,
        // which lives for the game.
        (!a.is_null()).then(|| unsafe { (*a).index })
    }
    /// `obj->artifact->aidx`, or `None` for no artifact.
    pub fn artifact_aidx(self) -> Option<u32> {
        let a = self.obj().artifact;
        // SAFETY: a non-NULL artifact points into a_info (lives for the game).
        (!a.is_null()).then(|| unsafe { (*a).aidx })
    }
    /// `obj->ego->eidx`, or `None` for no ego.
    pub fn ego_eidx(self) -> Option<u32> {
        let e = self.obj().ego;
        // SAFETY: a non-NULL ego points into e_info (lives for the game).
        (!e.is_null()).then(|| unsafe { (*e).eidx })
    }
}

/// An artifact (`struct artifact`).
pub type Artifact = crate::headers::object_h::artifact;

/// A copy of `a_info[aidx]` (panics for `aidx >= a_max`; see "Engine
/// tables": randart generation rewrites the table).
pub fn artifact(aidx: usize) -> Artifact {
    assert!(aidx < usize::from(z_info().a_max), "artifact {aidx} out of range");
    // SAFETY: a_info is allocated with z_info->a_max entries when the game
    // data is loaded, before any Borg code runs, and lives for the game
    // (index checked); copied at once, no reference is kept.
    unsafe { *crate::headers::object_h::a_info.add(aidx) }
}

/// An ego item type (`struct ego_item`).
pub type EgoItem = crate::headers::object_h::ego_item;

/// A copy of `e_info[eidx]` (panics for `eidx >= e_max`; see "Engine
/// tables": `everseen` changes while the game runs).
pub fn ego(eidx: usize) -> EgoItem {
    assert!(eidx < usize::from(z_info().e_max), "ego item {eidx} out of range");
    // SAFETY: e_info is allocated with z_info->e_max entries when the game
    // data is loaded, before any Borg code runs, and lives for the game
    // (index checked); copied at once, no reference is kept.
    unsafe { *crate::headers::object_h::e_info.add(eidx) }
}

/// `curses[i].name` (i < curse_max; panics where C would read NULL).
pub fn curse_name(i: usize) -> &'static [u8] {
    assert!(i < usize::from(z_info().curse_max), "curse index out of range");
    // SAFETY: curses has z_info->curse_max entries (index checked), loaded
    // at startup and kept for the game.
    let name = unsafe { (*crate::headers::obj_curse_h::curses.add(i)).name };
    assert!(!name.is_null(), "curse {i} has no name");
    // SAFETY: a non-NULL curse name is a C string that lives for the game.
    unsafe { CStr::from_ptr(name).to_bytes() }
}

/// `(slays[i].race_flag, slays[i].multiplier)` (i < slay_max).
pub fn slay_race_flag_multiplier(i: usize) -> (i32, i32) {
    assert!(i < usize::from(z_info().slay_max), "slay index out of range");
    // SAFETY: slays has z_info->slay_max entries (index checked), loaded at
    // startup and kept for the game.
    let s = unsafe { &*crate::headers::obj_slays_h::slays.add(i) };
    (s.race_flag, s.multiplier)
}

/// `r_info[r_idx].blow[k].effect->name` (bytes), or `None` when the blow
/// has no effect (`effect == NULL`), for `k < z_info->mon_blows_max`. Unlike
/// `race_blow` this does not look at the method (borg-fight-attack.c tests
/// only the effect).
pub fn race_blow_effect_name(r_idx: usize, k: usize) -> Option<&'static [u8]> {
    assert!(k < usize::from(z_info().mon_blows_max));
    let blows = race(r_idx).blow;
    assert!(!blows.is_null(), "race {r_idx} has no blow array");
    // SAFETY: a race's blow array has z_info->mon_blows_max entries
    // (checked above), allocated when the game data is loaded and kept for
    // the game; a non-NULL effect points into the blow effect list, whose
    // names are C strings kept for the game (checked non-NULL: C would
    // pass NULL to strcmp).
    unsafe {
        let b = &*blows.add(k);
        if b.effect.is_null() {
            return None;
        }
        let name = (*b.effect).name;
        assert!(!name.is_null(), "race {r_idx} blow {k} effect has no name");
        Some(CStr::from_ptr(name).to_bytes())
    }
}

/// `(brands[i].resist_flag, brands[i].vuln_flag, brands[i].multiplier)`
/// (i < brand_max).
pub fn brand_resist_vuln_multiplier(i: usize) -> (i32, i32, i32) {
    assert!(i < usize::from(z_info().brand_max), "brand index out of range");
    // SAFETY: brands has z_info->brand_max entries (index checked), loaded
    // at startup and kept for the game.
    let b = unsafe { &*crate::headers::obj_slays_h::brands.add(i) };
    (b.resist_flag, b.vuln_flag, b.multiplier)
}

/// `player_has_temporary_brand(player, idx)` (obj-slays.c).
pub fn player_has_temporary_brand(idx: i32) -> bool {
    // SAFETY: `player` lives for the game; the callee only reads the
    // player's timed effects and the brand list.
    unsafe { crate::headers::obj_slays_h::player_has_temporary_brand(crate::headers::player_h::player, idx) }
}

/// `player_has_temporary_slay(player, idx)` (obj-slays.c).
pub fn player_has_temporary_slay(idx: i32) -> bool {
    // SAFETY: as player_has_temporary_brand.
    unsafe { crate::headers::obj_slays_h::player_has_temporary_slay(crate::headers::player_h::player, idx) }
}

/// `get_angle_to_grid[y][x]` (generate.c, 41x41); panics out of range.
pub fn get_angle_to_grid(y: usize, x: usize) -> u8 {
    // SAFETY: a constant table of the engine (declared with its C size);
    // nothing writes it.
    unsafe { headers::generate_h::get_angle_to_grid[y][x] }
}

/// `lookup_obj_property(OBJ_PROPERTY_FLAG, flag)->power`, or `None` where
/// the lookup gives NULL.
pub fn obj_flag_power(flag: i32) -> Option<i32> {
    // SAFETY: the property list is loaded at startup; the result is NULL
    // or an entry of it.
    unsafe {
        let p = crate::headers::obj_properties_h::lookup_obj_property(
            crate::headers::obj_properties_h::OBJ_PROPERTY_FLAG as i32,
            flag,
        );
        (!p.is_null()).then(|| (*p).power)
    }
}

/// `flag_next(flags, size, flag)` (z-bitflag.c) on a whole flag array: the
/// first set flag from `flag` on, or `FLAG_END` (0). Panics for
/// `flag < FLAG_START`: C reads before the array there (for `FLAG_END`,
/// which its comment says starts at the beginning, `FLAG_OFFSET(0)` is
/// `SIZE_MAX / 8` stored in an `int`, i.e. -1). A start past the last flag
/// reads nothing and gives `FLAG_END`, as in C.
///
/// A transcription of the C loop (no FFI call): `for (f = flag; f <
/// FLAG_MAX(size); f++) if (flags[FLAG_OFFSET(f)] & FLAG_BINARY(f)) return
/// f;`. From `flag >= FLAG_START` every offset is inside the array.
pub fn flag_next(flags: &[u8], flag: i32) -> i32 {
    assert!(flag >= FLAG_START, "flag_next from flag {flag} (before FLAG_START)");
    // C's `FLAG_MAX(size)` is an `int`.
    assert!(flags.len() <= (i32::MAX as usize - 1) / 8, "flag set too large");
    let max_flags = (flags.len() * 8) as i32 + FLAG_START;
    (flag..max_flags)
        .find(|&f| {
            let f0 = (f - FLAG_START) as usize;
            flags[f0 / 8] & (1 << (f0 % 8)) != 0
        })
        .unwrap_or(FLAG_END)
}

/// The effects of `k_info[kind]` as `(index, subtype)`, in list order
/// (following `next` until NULL).
pub fn kind_effects(kind: usize) -> Vec<(u16, i32)> {
    let mut out = Vec::new();
    let mut e = self::kind(kind).effect;
    while !e.is_null() {
        // SAFETY: effects form a NULL-terminated list loaded at startup.
        unsafe {
            out.push(((*e).index, (*e).subtype));
            e = (*e).next;
        }
    }
    out
}

// ---------------------------------------------------------------------------
// Floor objects and object kinds (borg-flow-take.c)

/// `k_info[kidx].base->name` (bytes up to the NUL), or `None` when the
/// kind has no base (`base` is NULL). Panics for `kidx >= z_info->k_max`.
pub fn kind_base_name(kidx: usize) -> Option<&'static [u8]> {
    let base = self::kind(kidx).base;
    // SAFETY: a non-NULL kind base points into kb_info, which (with its
    // names) lives for the game.
    (!base.is_null()).then(|| c_bytes(unsafe { (*base).name }))
}

/// `tval_is_book_k(&k_info[kidx])`.
pub fn tval_is_book_k(kidx: usize) -> bool {
    let k = kind_ptr(kidx);
    // SAFETY: k points into k_info (index checked by kind_ptr); the callee only
    // reads the kind's tval.
    unsafe { crate::headers::obj_tval_h::tval_is_book_k(k) }
}

/// An object lying on the floor of the current level (an entry of a
/// grid's object pile). Opaque: the pointer stays on the FFI side
/// (`engine`). The pile only changes while the game runs; the handle is
/// bound to the borrow of the state's `access` token (see "Engine object
/// handles").
#[derive(Clone, Copy)]
pub struct FloorObject<'a>(*mut crate::headers::object_h::object, PhantomData<&'a Access>);

impl FloorObject<'_> {
    /// A copy of `*obj->kind` (a floor object always has a kind; it points
    /// into `k_info`, which the engine writes at run time: L5).
    pub fn kind(self) -> ObjectKind {
        // SAFETY: a live floor object; its kind points into k_info, which
        // lives for the game (checked non-NULL); copied at once.
        unsafe {
            let k = (*self.0).kind;
            assert!(!k.is_null(), "floor object without a kind");
            *k
        }
    }

    /// `object_fully_known(obj)`.
    pub fn fully_known(self) -> bool {
        // SAFETY: a live floor object; the callee only reads it.
        unsafe { crate::headers::obj_knowledge_h::object_fully_known(self.0) }
    }

    /// `(obj->to_a, obj->to_h, obj->to_d)`.
    pub fn to_ahd(self) -> (i16, i16, i16) {
        // SAFETY: a live floor object (plain field reads).
        unsafe { ((*self.0).to_a, (*self.0).to_h, (*self.0).to_d) }
    }

    /// `quark_str(obj->note)` when the object has a note (`obj->note`
    /// non-zero), as bytes.
    pub fn note(self) -> Option<&'static [u8]> {
        // SAFETY: a live floor object; a non-zero quark is a valid index
        // into the quark table. Quark strings are never written or freed
        // while the game runs (the table only grows; a string keeps its
        // address), so a `'static` slice is sound.
        unsafe {
            let q = (*self.0).note;
            (q != 0).then(|| c_bytes(crate::headers::z_quark_h::quark_str(q)))
        }
    }

    /// `obj->pval`.
    pub fn pval(self) -> i16 {
        // SAFETY: a live floor object (plain field read).
        unsafe { (*self.0).pval }
    }

    /// `obj->known != NULL`.
    pub fn has_known(self) -> bool {
        // SAFETY: a live floor object (plain field read).
        unsafe { !(*self.0).known.is_null() }
    }
}

/// `borg_get_top_object(cave, loc(x, y))` (borg-flow-take.c): the first
/// object of the grid's pile that is neither ignored by the player
/// (`known->notice & OBJ_NOTICE_IGNORE`) nor of an ignored kind, or `None`.
/// An out-of-bounds grid has no pile (`square_object` checks it).
pub fn top_floor_object(access: &Access, x: i32, y: i32) -> Option<FloorObject<'_>> {
    use crate::headers::object_h::OBJ_NOTICE_IGNORE;
    check_access(access);
    // SAFETY: `cave` is the current level, valid while the game runs;
    // square_object returns NULL for a grid out of bounds (its own check,
    // kept with NDEBUG), else the head of the grid's pile (or NULL). Each
    // object of a pile is live and has a kind, and its `next` and `known`
    // pointers are NULL or live objects.
    unsafe {
        let mut o = headers::cave_h::square_object(headers::cave_h::cave, loc { x, y });
        while !o.is_null() {
            let ignored_known = !(*o).known.is_null()
                && i32::from((*(*o).known).notice) & OBJ_NOTICE_IGNORE as i32 != 0;
            if !(ignored_known || (*(*o).kind).ignore != 0) {
                break;
            }
            o = (*o).next;
        }
        (!o.is_null()).then_some(FloorObject(o, PhantomData))
    }
}


// ---------------------------------------------------------------------------
// Borg start-up (borg-init.c)

/// `do_cmd_redraw()`: redraw the whole screen (signals events: lends the
/// state).
pub fn do_cmd_redraw(st: &mut BorgState) {
    // SAFETY: plain call on the engine thread.
    lend(st, || unsafe { crate::headers::ui_command_h::do_cmd_redraw() })
}

/// `event_signal(type)` (a message flush can wait for a key: lends the
/// state).
pub fn event_signal(st: &mut BorgState, kind: game_event_type) {
    check_event(kind);
    // SAFETY: plain call; handlers get no data, as in C; the type indexes
    // the handler table (checked).
    lend(st, || unsafe { crate::headers::game_event_h::event_signal(kind) })
}

/// `screensaver` (option.c).
pub fn screensaver() -> bool {
    // SAFETY: plain read of a bool global on the engine thread.
    unsafe { crate::headers::option_h::screensaver }
}

/// `(tile_width, tile_height)` (ui-term.c).
pub fn tile_size() -> (u8, u8) {
    // SAFETY: plain reads of two byte globals on the engine thread.
    unsafe { (crate::headers::ui_term_h::tile_width, crate::headers::ui_term_h::tile_height) }
}

/// `player->upkeep->redraw |= flags`.
pub fn player_redraw_add(flags: u32) {
    // SAFETY: `player` and its upkeep are allocated at startup and live for
    // the game; single-threaded write.
    unsafe { (*(*crate::headers::player_h::player).upkeep).redraw |= flags }
}

/// `player->opts.hitpoint_warn = v`.
pub fn set_player_hitpoint_warn(v: u8) {
    // SAFETY: `player` is allocated at startup; single-threaded write.
    unsafe { (*crate::headers::player_h::player).opts.hitpoint_warn = v }
}

/// `player_id2class(id)->name` (panics where C would read through NULL).
pub fn class_name_by_id(id: u32) -> &'static [u8] {
    // SAFETY: player_id2class returns NULL or an entry of the static class
    // list, whose name is a C string that lives for the game.
    unsafe {
        let c = crate::headers::player_h::player_id2class(id);
        assert!(!c.is_null(), "player_id2class returned NULL");
        CStr::from_ptr((*c).name).to_bytes()
    }
}

/// `ITYPE_MAX` (obj-ignore.h): the number of ignore types.
pub const ITYPE_MAX: usize = crate::headers::obj_ignore_h::ITYPE_MAX as usize;

/// `k_info[kind].ignore = v`.
pub fn set_kind_ignore(kind: usize, v: u8) {
    assert!(kind < usize::from(z_info().k_max));
    // SAFETY: k_info has z_info->k_max entries (checked); single-threaded
    // write of a byte field.
    unsafe { (*crate::headers::object_h::k_info.add(kind)).ignore = v }
}

/// `ignore_level[0 .. ITYPE_MAX]` (obj-ignore.c).
pub fn ignore_level() -> [u8; ITYPE_MAX] {
    // SAFETY: ignore_level is a static array of ITYPE_MAX bytes, declared
    // with its C length in the header (so the read stays inside the
    // static); plain copy on the engine thread.
    unsafe { *core::ptr::addr_of!(crate::headers::obj_ignore_h::ignore_level) }
}

/// `ignore_level[i] = levels[i]` for all ignore types.
pub fn set_ignore_level(levels: &[u8; ITYPE_MAX]) {
    // SAFETY: as ignore_level(); single-threaded write.
    unsafe { *core::ptr::addr_of_mut!(crate::headers::obj_ignore_h::ignore_level) = *levels }
}

/// `ego_ignore_types[ego][0 .. ITYPE_MAX]` (obj-ignore.c).
pub fn ego_ignore_types(ego: usize) -> [bool; ITYPE_MAX] {
    assert!(ego < usize::from(z_info().e_max));
    // SAFETY: ego_ignore_types has z_info->e_max rows (checked) of
    // ITYPE_MAX bools each, allocated by ignore_init() at startup.
    unsafe {
        *(*crate::headers::obj_ignore_h::ego_ignore_types.add(ego) as *const [bool; ITYPE_MAX])
    }
}

/// `ego_ignore_types[ego][i] = types[i]` for all ignore types.
pub fn set_ego_ignore_types(ego: usize, types: &[bool; ITYPE_MAX]) {
    assert!(ego < usize::from(z_info().e_max));
    // SAFETY: as ego_ignore_types(); single-threaded write.
    unsafe {
        *(*crate::headers::obj_ignore_h::ego_ignore_types.add(ego) as *mut [bool; ITYPE_MAX]) =
            *types;
    }
}

/// `ignore_birth_init()` (obj-ignore.c): reset the ignore settings.
pub fn ignore_birth_init() {
    // SAFETY: plain call on the engine thread.
    unsafe { crate::headers::obj_ignore_h::ignore_birth_init() }
}

// ---------------------------------------------------------------------------
// The Borg's ^Z command interface (borg-commands.c). The calls that can
// wait for a key (`get_com`, `get_com_ex`, `get_quantity`, `menu_select`,
// `target_set_interactive`, `screen_save`/`screen_load` (message flush),
// `project` (display events)) lend the caller's state (see `enter`).

/// `player->grid`
pub fn player_grid() -> loc {
    player_ref().grid
}

/// `player->opts.lazymove_delay`
pub fn player_lazymove_delay() -> u8 {
    player_ref().opts.lazymove_delay
}

/// `player->opts.lazymove_delay = v`
pub fn set_player_lazymove_delay(v: u8) {
    // SAFETY: `player` is allocated at startup; single-threaded write.
    unsafe { (*crate::headers::player_h::player).opts.lazymove_delay = v }
}

/// `player->opts.delay_factor`
pub fn player_delay_factor() -> u8 {
    player_ref().opts.delay_factor
}

/// `player->opts.delay_factor = v`
pub fn set_player_delay_factor(v: u8) {
    // SAFETY: `player` is allocated at startup; single-threaded write.
    unsafe { (*crate::headers::player_h::player).opts.delay_factor = v }
}

/// `Term_save()`
pub fn term_save() {
    // SAFETY: plain call on the active term.
    unsafe {
        crate::headers::ui_term_h::Term_save();
    }
}

/// `Term_load()`
pub fn term_load() {
    // SAFETY: plain call on the active term (with no saved screen it
    // loads a blank one).
    unsafe {
        crate::headers::ui_term_h::Term_load();
    }
}

/// `Term_flush()`: discard pending keypresses.
pub fn term_flush() {
    // SAFETY: plain call on the active term.
    unsafe {
        crate::headers::ui_term_h::Term_flush();
    }
}

/// `target_okay()`
pub fn target_okay() -> bool {
    // SAFETY: plain call; reads the target and the current level.
    unsafe { crate::headers::target_h::target_okay() }
}

/// `target_get(&grid)`
pub fn target_get() -> loc {
    let mut l = loc { x: 0, y: 0 };
    // SAFETY: out-pointer to a local.
    unsafe { crate::headers::target_h::target_get(&mut l) };
    l
}

/// `get_com(prompt, &cmd)`: the result and the key as the C `char`
/// (byte). Waits for a key (lends the state).
pub fn get_com(st: &mut BorgState, prompt: &CStr) -> (bool, u8) {
    let mut cmd: c_char = 0;
    // SAFETY: valid prompt and out-pointer to a local.
    let r = lend(st, || unsafe { crate::headers::game_input_h::get_com(prompt.as_ptr(), &mut cmd) });
    (r, cmd as u8)
}

/// `get_com_ex(prompt, &ke)`; the event itself is dropped. Waits for a
/// key (lends the state).
pub fn get_com_ex(st: &mut BorgState, prompt: &CStr) -> bool {
    // SAFETY: ui_event is plain data (zeroed = EVT_NONE); valid prompt and
    // out-pointer to a local.
    lend(st, || unsafe {
        let mut ke: crate::headers::ui_event_h::ui_event = core::mem::zeroed();
        crate::headers::ui_input_h::get_com_ex(prompt.as_ptr(), &mut ke)
    })
}

/// `get_quantity(prompt, max)`. Waits for keys (lends the state).
pub fn get_quantity(st: &mut BorgState, prompt: &CStr, max: i32) -> i32 {
    // SAFETY: valid prompt.
    lend(st, || unsafe { crate::headers::game_input_h::get_quantity(prompt.as_ptr(), max) })
}

/// `target_set_interactive(mode, x, y, allow_pathfinding)`. Waits for
/// keys (lends the state).
pub fn target_set_interactive(st: &mut BorgState, mode: i32, x: i32, y: i32, allow_pathfinding: bool) -> bool {
    // SAFETY: plain call on the engine thread.
    lend(st, || unsafe { crate::headers::ui_target_h::target_set_interactive(mode, x, y, allow_pathfinding) })
}

/// `print_rel(c, a, y, x)`: draw a character at a map grid (if it is on
/// the panel).
pub fn print_rel(c: u8, a: u8, y: i32, x: i32) {
    // SAFETY: plain call; the function checks the grid against the panel
    // and each map window before it draws (any x, y).
    unsafe { crate::headers::ui_map_h::print_rel(wchar_t::from(c), a, y, x) }
}

/// `prt_map()`: redraw the map.
pub fn prt_map() {
    // SAFETY: plain call on the engine thread.
    unsafe { crate::headers::ui_map_h::prt_map() }
}

/// `c_put_str(attr, s, row, col)` (no NUL in `s`).
pub fn c_put_str(attr: u8, s: &[u8], row: i32, col: i32) {
    let c = CString::new(s).expect("no NUL in term text");
    // SAFETY: c is a valid C string for the call.
    unsafe { crate::headers::ui_output_h::c_put_str(attr, c.as_ptr(), row, col) }
}

/// `screen_save()` (flushes messages, which can wait for a key: lends
/// the state).
pub fn screen_save(st: &mut BorgState) {
    // SAFETY: plain call on the engine thread.
    lend(st, || unsafe { crate::headers::ui_output_h::screen_save() })
}

/// `screen_load()` (flushes messages, which can wait for a key: lends
/// the state).
pub fn screen_load(st: &mut BorgState) {
    // SAFETY: plain call on the engine thread.
    lend(st, || unsafe { crate::headers::ui_output_h::screen_load() })
}

/// `window_make(origin_x, origin_y, end_x, end_y)`
pub fn window_make(origin_x: i32, origin_y: i32, end_x: i32, end_y: i32) {
    // SAFETY: plain call; draws on the active term.
    unsafe { crate::headers::ui_output_h::window_make(origin_x, origin_y, end_x, end_y) }
}

/// One `display_row` call of a menu.
#[derive(Clone, Copy, Debug)]
pub struct MenuRow {
    pub oid: i32,
    pub cursor: bool,
    pub row: i32,
    pub col: i32,
    pub width: i32,
}

/// The row display of the menu shown by `Menu::select` (lifetime erased;
/// only set while `select` runs).
static mut MENU_DISPLAY: Option<*mut dyn FnMut(MenuRow)> = None;

unsafe extern "C" fn menu_display_row(
    _menu: *mut crate::headers::ui_menu_h::menu,
    oid: core::ffi::c_int,
    cursor: bool,
    row: core::ffi::c_int,
    col: core::ffi::c_int,
    width: core::ffi::c_int,
) {
    let f = MENU_DISPLAY.expect("menu row drawn outside Menu::select");
    (*f)(MenuRow { oid, cursor, row, col, width });
}

/// The row functions of a Borg menu: only `display_row`, as the
/// `menu_iter`s of borg-commands.c.
static MENU_ITER: crate::headers::ui_menu_h::menu_iter = crate::headers::ui_menu_h::menu_iter {
    get_tag: None,
    valid_row: None,
    display_row: Some(menu_display_row),
    row_handler: None,
    resize: None,
};

/// A scrolling menu (`menu_new(MN_SKIN_SCROLL, iter)`). As in C it is
/// never freed.
pub struct Menu(*mut crate::headers::ui_menu_h::menu);

impl Menu {
    /// `menu_new(MN_SKIN_SCROLL, &iter)`
    pub fn new_scroll() -> Menu {
        // SAFETY: MENU_ITER is a static that outlives the menu.
        let m = unsafe {
            crate::headers::ui_menu_h::menu_new(crate::headers::ui_menu_h::MN_SKIN_SCROLL, &MENU_ITER)
        };
        assert!(!m.is_null());
        Menu(m)
    }

    /// `menu_setpriv(m, count, data)`. The private data is never read
    /// (the row display does not use it); a pointer to the row functions
    /// is stored.
    pub fn setpriv(&mut self, count: i32) {
        // SAFETY: live menu; the data pointer is only stored.
        unsafe {
            crate::headers::ui_menu_h::menu_setpriv(
                self.0,
                count,
                &MENU_ITER as *const _ as *mut core::ffi::c_void,
            )
        }
    }

    /// `menu_layout(m, &area)`
    pub fn layout(&mut self, area: crate::headers::ui_output_h::region) -> bool {
        // SAFETY: live menu; the region is read during the call.
        unsafe { crate::headers::ui_menu_h::menu_layout(self.0, &area) }
    }

    /// `menu_select(m, notify, popup)`: the type of the returned event.
    /// `display` draws a row (the `display_row` of the C iterator).
    /// Waits for keys (lends the state; `display` cannot use it).
    pub fn select(
        &mut self,
        st: &mut BorgState,
        notify: i32,
        popup: bool,
        display: &mut dyn FnMut(MenuRow),
    ) -> ui_event_type {
        let p: *mut (dyn FnMut(MenuRow) + '_) = display;
        // SAFETY: the pointer is only used by menu_display_row while this
        // call runs (restored before returning), so the erased lifetime is
        // never outlived; engine thread only.
        lend(st, || unsafe {
            let saved = MENU_DISPLAY;
            MENU_DISPLAY = Some(core::mem::transmute::<
                *mut (dyn FnMut(MenuRow) + '_),
                *mut (dyn FnMut(MenuRow) + 'static),
            >(p));
            let ev = crate::headers::ui_menu_h::menu_select(self.0, notify, popup);
            MENU_DISPLAY = saved;
            ev.type_0
        })
    }

    /// `m->cursor`
    pub fn cursor(&self) -> i32 {
        // SAFETY: live menu.
        unsafe { (*self.0).cursor }
    }
}

/// The projections the Borg's `^Z` targeting display draws (borg-commands.c).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Projection {
    /// `project(source_player(), 0, grid, 1, PROJ_MISSILE, PROJECT_BEAM, 0, 0, NULL)`
    Beam,
    /// `project(source_player(), 10, grid, 50, PROJ_MISSILE, PROJECT_ARC, 60, 4, NULL)`
    Arc,
}

/// `project(source_player(), rad, loc(x, y), dam, typ, flg, arc,
/// diameter, NULL)` with the arguments of one of the Borg's projections.
/// `project` has preconditions it does not check (`typ` indexes the
/// projection table, the blast fits its 256-grid arrays, the path stays
/// on the level), so only these two fixed shapes are offered, at a grid in
/// bounds (panics otherwise). Signals display events (lends the state).
pub fn project_from_player(st: &mut BorgState, shape: Projection, x: i32, y: i32) -> bool {
    use crate::headers::list_elements_h::PROJ_MISSILE;
    use crate::headers::project_h::{PROJECT_ARC, PROJECT_BEAM};
    let (rad, dam, flg, degrees_of_arc, diameter_of_source) = match shape {
        Projection::Beam => (0, 1, PROJECT_BEAM as i32, 0, 0),
        Projection::Arc => (10, 50, PROJECT_ARC as i32, 60, 4),
    };
    let typ = PROJ_MISSILE as i32;
    assert!(square_in_bounds(x, y), "projection target out of bounds");
    // SAFETY: a NULL object is allowed; the type is a projection, the
    // radius and arc are the Borg's (an arc of radius 10 and 60 degrees,
    // or a beam: within the 256 grids), and the path runs between two
    // grids of the level (the player's and the checked target).
    lend(st, || unsafe {
        crate::headers::project_h::project(
            crate::headers::source_h::source_player(),
            rad,
            loc { x, y },
            dam,
            typ,
            flg,
            degrees_of_arc,
            diameter_of_source,
            core::ptr::null(),
        )
    })
}

/// `SQUARE_SIZE`: bytes in a square's info flags.
pub const SQUARE_SIZE: usize = (crate::headers::cave_h::SQUARE_MAX as usize).div_ceil(8);

/// `square(cave, loc(x, y))->feat`; panics out of bounds (the engine's
/// assert is compiled out).
pub fn square_feat(x: i32, y: i32) -> u8 {
    assert!(square_in_bounds(x, y), "grid out of bounds");
    // SAFETY: `cave` is the current level; the grid is in bounds.
    unsafe { (*headers::cave_h::square(headers::cave_h::cave, loc { x, y })).feat }
}

/// `square(cave, loc(x, y))->info` (`SQUARE_SIZE` bytes); panics out of
/// bounds.
pub fn square_info(x: i32, y: i32) -> [u8; SQUARE_SIZE] {
    assert!(square_in_bounds(x, y), "grid out of bounds");
    // SAFETY: `cave` is the current level; the grid is in bounds and its
    // info points at SQUARE_SIZE bytes.
    unsafe {
        let sq = headers::cave_h::square(headers::cave_h::cave, loc { x, y });
        *((*sq).info as *const [u8; SQUARE_SIZE])
    }
}

/// `f_info[feat].name` (feat < FEAT_MAX; panics on a NULL name, where C
/// would crash in `c_put_str`).
pub fn feat_name(feat: i32) -> &'static [u8] {
    assert!((0..crate::headers::cave_h::FEAT_MAX as i32).contains(&feat), "feature out of range");
    // SAFETY: f_info has FEAT_MAX entries (index checked), loaded at
    // startup and kept for the game.
    let name = unsafe { (*crate::headers::cave_h::f_info.add(feat as usize)).name };
    assert!(!name.is_null(), "feature without a name");
    // SAFETY: a non-NULL game data string.
    unsafe { CStr::from_ptr(name).to_bytes() }
}

/// `k_info[kidx].name` as `%s` prints it (NULL prints as "").
pub fn kind_name_or_empty(kidx: usize) -> &'static [u8] {
    c_bytes(self::kind(kidx).name)
}

/// `a_info[aidx].name` as `%s` prints it (NULL prints as "").
pub fn artifact_name_or_empty(aidx: usize) -> &'static [u8] {
    c_bytes(artifact(aidx).name)
}

/// `object_flags(player->upkeep->inven[n], f)` (all zero for an empty
/// slot); `0 <= n <= pack_size` (the array has a NULL entry at
/// `pack_size`).
pub fn inven_object_flags(n: i32) -> [u8; OF_SIZE] {
    assert!((0..=i32::from(z_info().pack_size)).contains(&n), "pack slot out of range");
    let mut f = [0u8; OF_SIZE];
    // SAFETY: inven has pack_size + 1 entries (index checked); object_flags
    // accepts NULL and writes OF_SIZE bytes.
    unsafe {
        let obj = *(*player_ref().upkeep).inven.add(n as usize);
        crate::headers::obj_util_h::object_flags(obj, f.as_mut_ptr());
    }
    f
}

// ---------------------------------------------------------------------------
// Map updates (borg-update.c)

/// `cave->width`, `cave->height` of the current level.
pub fn cave_size() -> (i32, i32) {
    // SAFETY: `cave` is the current level, valid while the game runs.
    unsafe {
        let c = &*headers::cave_h::cave;
        (c.width, c.height)
    }
}

/// What `map_info()` reports about a grid (the fields the Borg reads).
#[derive(Clone, Copy, Debug)]
pub struct MapGrid {
    /// `m_idx`: the visible monster (0 = none)
    pub m_idx: u32,
    /// `f_idx`: the remembered feature (after mimics)
    pub f_idx: u32,
    /// `first_kind`: `(kidx, d_attr, d_char)` of the first visible object
    /// kind, `None` for NULL
    pub first_kind: Option<(u32, u8, wchar_t)>,
    /// `lighting` (`LIGHTING_*`). Never read: the corrected "Floors" branch
    /// of `borg_update_map` tests `square_islit()` instead.
    #[allow(dead_code)]
    pub lighting: u32,
    pub in_view: bool,
    pub is_player: bool,
    /// `g.trap && trf_has(g.trap->flags, TRF_GLYPH)`: the trap the player
    /// remembers on the grid is a glyph (better-borg-c patch 0098).
    pub glyph: bool,
}

/// `map_info(loc(x, y), &g)` (cave-map.c). It also memorizes the grid
/// when the player sees it, as in C. Panics out of bounds (the engine's
/// asserts are compiled out).
pub fn map_info(x: i32, y: i32) -> MapGrid {
    assert!(square_in_bounds(x, y), "grid out of bounds");
    let mut g = headers::cave_h::grid_data {
        m_idx: 0,
        f_idx: 0,
        first_kind: core::ptr::null_mut(),
        trap: core::ptr::null_mut(),
        multiple_objects: false,
        unseen_object: false,
        unseen_money: false,
        lighting: 0,
        in_view: false,
        is_player: false,
        hallucinate: false,
    };
    // SAFETY: the grid is in bounds (checked); map_info fills every field
    // the Borg reads; first_kind is NULL or an object kind of the game
    // data, which lives for the game. g.trap is NULL or an engine-owned
    // remembered trap, valid until the next engine mutation; copy its flag
    // before returning, without retaining any reference.
    unsafe {
        headers::cave_h::map_info(loc { x, y }, &mut g);
        MapGrid {
            m_idx: g.m_idx,
            f_idx: g.f_idx,
            first_kind: (!g.first_kind.is_null()).then(|| {
                let k = &*g.first_kind;
                (k.kidx, k.d_attr, k.d_char)
            }),
            lighting: g.lighting,
            in_view: g.in_view,
            is_player: g.is_player,
            glyph: !g.trap.is_null()
                && flag_has(&(*g.trap).flags, headers::list_trap_flags_h::TRF_GLYPH as i32),
        }
    }
}

/// `square(cave, loc(x, y))->trap` is set and the (first) trap on the grid
/// is a glyph: `trf_has(trap_info[...->t_idx].flags, TRF_GLYPH)`. False
/// without a trap, where better-borg-c patch 0098 reads through NULL (the
/// local 0098 guard). Panics out of bounds or for a bad trap index.
pub fn square_trap_is_glyph(x: i32, y: i32) -> bool {
    assert!(square_in_bounds(x, y), "grid out of bounds");
    // SAFETY: `cave` is the current level and the grid is in bounds; the
    // square's trap pointer is NULL or an engine-owned trap; trap_info has
    // z_info->trap_max entries (index checked); the kind's flag array is
    // copied.
    unsafe {
        let sq = headers::cave_h::square(headers::cave_h::cave, loc { x, y });
        trap_head_is_glyph((*sq).trap, |t_idx| {
            assert!(t_idx < usize::from(z_info().trap_max), "trap index out of range");
            let flags = (*headers::trap_h::trap_info.add(t_idx)).flags;
            flag_has(&flags, headers::list_trap_flags_h::TRF_GLYPH as i32)
        })
    }
}

/// The local 0098 guard: whether the trap list head `trap` (the cave's
/// `square(cave, loc)->trap`) is a glyph, where `kind_is_glyph(t_idx)` reads
/// the trap kind. False for a NULL head, without reading any kind.
///
/// # Safety
/// `trap` must be NULL or point to a valid trap.
unsafe fn trap_head_is_glyph(
    trap: *const headers::trap_h::trap,
    kind_is_glyph: impl FnOnce(usize) -> bool,
) -> bool {
    if trap.is_null() {
        return false;
    }
    // SAFETY: non-null, valid by the caller's contract.
    kind_is_glyph(usize::from(unsafe { (*trap).t_idx }))
}

/// `square_shopnum(cave, loc(x, y))`: the store of a shop entrance (-1
/// otherwise). Panics out of bounds.
pub fn square_shopnum(x: i32, y: i32) -> i32 {
    assert!(square_in_bounds(x, y), "grid out of bounds");
    // SAFETY: `cave` is the current level; the grid is in bounds.
    unsafe { headers::cave_h::square_shopnum(headers::cave_h::cave, loc { x, y }) }
}

/// `square_isdisarmabletrap(cave, loc(x, y))`. Panics out of bounds.
pub fn square_isdisarmabletrap(x: i32, y: i32) -> bool {
    assert!(square_in_bounds(x, y), "grid out of bounds");
    // SAFETY: `cave` is the current level; the grid is in bounds.
    unsafe { headers::cave_h::square_isdisarmabletrap(headers::cave_h::cave, loc { x, y }) }
}

/// `square_iswebbed(cave, loc(x, y))`. Panics out of bounds.
pub fn square_iswebbed(x: i32, y: i32) -> bool {
    assert!(square_in_bounds(x, y), "grid out of bounds");
    // SAFETY: `cave` is the current level; the grid is in bounds.
    unsafe { headers::cave_h::square_iswebbed(headers::cave_h::cave, loc { x, y }) }
}

// ---------------------------------------------------------------------------
// Messages (borg-messages.c)

/// A C string of game data as owned bytes, `None` for NULL.
fn opt_text(p: *const c_char) -> Option<Vec<u8>> {
    // SAFETY: non-NULL pointers passed here are game data strings that live
    // for the game.
    (!p.is_null()).then(|| unsafe { CStr::from_ptr(p).to_bytes().to_vec() })
}

/// `inkey_flag` (ui-input.c): the game is waiting for a command.
pub fn inkey_flag() -> bool {
    // SAFETY: plain read of a bool global; engine thread.
    unsafe { crate::headers::ui_input_h::inkey_flag }
}

/// `player->wizard`
pub fn player_wizard() -> bool {
    player_ref().wizard
}

/// `TERM_XTRA_NOISE` (ui-term.h): make a noise.
pub const TERM_XTRA_NOISE: i32 = 7;
/// `TERM_XTRA_DELAY` (ui-term.h): wait some milliseconds.
pub const TERM_XTRA_DELAY: i32 = 13;

/// `Term_xtra(n, v)` on the current terminal, for `TERM_XTRA_NOISE` and
/// `TERM_XTRA_DELAY` only (the ones the Borg uses; other actions, such as
/// `TERM_XTRA_EVENT`, process input; panics). The headless frontend
/// ignores the noise and does not sleep.
pub fn term_xtra(n: i32, v: i32) -> i32 {
    assert!(n == TERM_XTRA_NOISE || n == TERM_XTRA_DELAY, "Term_xtra action {n} not supported");
    // SAFETY: the active terminal is set up by the frontend; a noise or
    // delay request (checked) never waits for a key.
    unsafe { crate::headers::ui_term_h::Term_xtra(n, v) }
}

/// `square_iswarded(cave, loc(x, y))` on the current level: a glyph of
/// warding (or another protective rune) on the grid. Panics if the grid
/// is out of bounds (as `square_isvault`).
pub fn square_iswarded(x: i32, y: i32) -> bool {
    assert!(square_in_bounds(x, y), "grid out of bounds");
    // SAFETY: `cave` is the current level, valid while the game runs; the
    // grid is in bounds (checked above).
    unsafe { headers::cave_h::square_iswarded(headers::cave_h::cave, loc { x, y }) }
}

/// The texts of one level of a monster spell (`struct
/// monster_spell_level`), `None` for NULL.
pub struct SpellLevelTexts {
    pub blind_message: Option<Vec<u8>>,
    pub message: Option<Vec<u8>>,
    pub miss_message: Option<Vec<u8>>,
}

/// The monster spells (`monster_spells`, in list order): each spell's
/// `index` and the texts of its levels (in list order).
pub fn monster_spell_texts() -> Vec<(u16, Vec<SpellLevelTexts>)> {
    use crate::headers::monster_h::{monster_spell, monster_spell_level};
    let mut out = Vec::new();
    // SAFETY: `monster_spells` is parsed game data (NULL-ended lists,
    // never freed while the game runs); the nodes are only read.
    unsafe {
        let mut spell = crate::headers::monster_h::monster_spells as *const monster_spell;
        while !spell.is_null() {
            let mut levels = Vec::new();
            let mut lv = (*spell).level as *const monster_spell_level;
            while !lv.is_null() {
                levels.push(SpellLevelTexts {
                    blind_message: opt_text((*lv).blind_message),
                    message: opt_text((*lv).message),
                    miss_message: opt_text((*lv).miss_message),
                });
                lv = (*lv).next;
            }
            out.push(((*spell).index, levels));
            spell = (*spell).next;
        }
    }
    out
}

/// `pain_messages[idx].messages` (`None` for NULL); panics unless
/// `idx < z_info->mp_max`.
pub fn pain_texts(idx: usize) -> [Option<Vec<u8>>; 7] {
    assert!(idx < usize::from(z_info().mp_max), "pain index out of range");
    // SAFETY: plain read of a pointer global set up by the game's init.
    let p = unsafe { crate::headers::monster_h::pain_messages };
    assert!(!p.is_null(), "no pain messages");
    // SAFETY: `pain_messages` has `mp_max` entries (index checked) of game
    // data that live for the game.
    let pain = unsafe { &*p.add(idx) };
    core::array::from_fn(|i| opt_text(pain.messages[i]))
}

/// The blow methods (`blow_methods[0 .. z_info->blow_methods_max]`): each
/// method's `msgt` and the `act_msg` of its messages (in list order,
/// `None` for NULL).
pub fn blow_method_texts() -> Vec<(i32, Vec<Option<Vec<u8>>>)> {
    use crate::headers::mon_blows_h::blow_message;
    let n = usize::from(z_info().blow_methods_max);
    // SAFETY: plain read of a pointer global set up by the game's init.
    let base = unsafe { crate::headers::mon_blows_h::blow_methods };
    assert!(n == 0 || !base.is_null(), "no blow methods");
    let mut out = Vec::with_capacity(n);
    for i in 0..n {
        // SAFETY: `blow_methods` has `blow_methods_max` entries of parsed
        // game data (NULL-ended message lists) that live for the game.
        unsafe {
            let m = &*base.add(i);
            let mut texts = Vec::new();
            let mut msg = m.messages as *const blow_message;
            while !msg.is_null() {
                texts.push(opt_text((*msg).act_msg));
                msg = (*msg).next;
            }
            out.push((m.msgt, texts));
        }
    }
    out
}

// ---------------------------------------------------------------------------
// The player as borg-trait.c reads it

/// `player->race`: the player's race (race list entry, set at birth).
pub fn player_race_info() -> &'static crate::headers::player_h::player_race {
    let r = player_ref().race;
    assert!(!r.is_null(), "player has no race");
    // SAFETY: `player->race` points into the race list loaded at startup
    // (set at birth, lives for the game); non-NULL (checked above). The
    // race list is only written while it is parsed (no runtime field),
    // so a `'static` shared reference never overlaps a write.
    unsafe { &*r }
}

/// `player->class`: the player's class (class list entry, set at birth).
pub fn player_class_info() -> &'static crate::headers::player_h::player_class {
    let c = player_ref().class;
    assert!(!c.is_null(), "player has no class");
    // SAFETY: `player->class` points into the class list loaded at
    // startup (set at birth, lives for the game); non-NULL (checked above).
    // The class list (with its magic) is only written while it is parsed,
    // so a `'static` shared reference never overlaps a write.
    unsafe { &*c }
}

/// The player fields that borg-trait.c reads (`borg_notice_player`,
/// `borg_notice_equipment`), copied.
#[derive(Clone, Copy, Debug)]
pub struct PlayerNotice {
    pub lev: i16,
    pub max_lev: i16,
    pub total_winner: u16,
    pub exp: i32,
    pub max_exp: i32,
    pub depth: i16,
    pub max_depth: i16,
    pub au: i32,
    pub word_recall: i16,
    pub chp: i16,
    pub mhp: i16,
    pub csp: i16,
    pub msp: i16,
    pub stat_cur: [i16; 5],
    pub stat_max: [i16; 5],
    /// `player->player_hp` (hit points per level)
    pub player_hp: [i16; 50],
    /// `player->upkeep->new_spells`
    pub new_spells: i32,
    /// `player->obj_k->modifiers` (the modifier runes the player knows)
    pub obj_k_modifiers: [i16; 16],
}

/// Read the player fields of `PlayerNotice`.
pub fn player_notice() -> PlayerNotice {
    let p = player_ref();
    assert!(!p.upkeep.is_null() && !p.obj_k.is_null(), "player not set up");
    // SAFETY: `upkeep` and `obj_k` are allocated with the player (checked
    // non-NULL above) and live for the game.
    let (new_spells, obj_k_modifiers) = unsafe { ((*p.upkeep).new_spells, (*p.obj_k).modifiers) };
    PlayerNotice {
        lev: p.lev,
        max_lev: p.max_lev,
        total_winner: p.total_winner,
        exp: p.exp,
        max_exp: p.max_exp,
        depth: p.depth,
        max_depth: p.max_depth,
        au: p.au,
        word_recall: p.word_recall,
        chp: p.chp,
        mhp: p.mhp,
        csp: p.csp,
        msp: p.msp,
        stat_cur: p.stat_cur,
        stat_max: p.stat_max,
        player_hp: p.player_hp,
        new_spells,
        obj_k_modifiers,
    }
}

/// `modify_stat_value(value, amount)` (player-util.c): a stat value
/// raised or lowered by `amount` points.
pub fn modify_stat_value(value: i32, amount: i32) -> i16 {
    // player-util.c, step by step as C does it; panics where C's `int`
    // arithmetic would overflow (`value += 10` past INT_MAX, `0 - amount`
    // for INT_MIN), which C leaves undefined.
    let mut value = value;
    if amount > 0 {
        for _ in 0..amount {
            if value < 18 {
                value += 1;
            } else {
                value = value.checked_add(10).expect("modify_stat_value: value overflows int");
            }
        }
    } else if amount < 0 {
        let n = 0i32.checked_sub(amount).expect("modify_stat_value: -amount overflows int");
        for _ in 0..n {
            if value >= 18 + 10 {
                value -= 10;
            } else if value > 18 {
                value = 18;
            } else if value > 3 {
                value -= 1;
            }
        }
    }
    // `return (value);` from an int to `int16_t`: gcc keeps the low 16 bits
    value as i16
}

/// `adj_str_blow[i]` (player-calcs.c); panics out of range.
pub fn adj_str_blow(i: usize) -> i32 {
    // SAFETY: a constant table of the engine; nothing writes it.
    unsafe { headers::player_calcs_h::adj_str_blow[i] }
}

/// The food levels of player-timed.c: (`PY_FOOD_WEAK`, `PY_FOOD_HUNGRY`,
/// `PY_FOOD_FULL`, `PY_FOOD_MAX`).
pub fn py_food_levels() -> (i32, i32, i32, i32) {
    use crate::headers::player_timed_h as t;
    // SAFETY: plain reads of int globals the game sets at init and never
    // changes afterwards.
    unsafe { (t::PY_FOOD_WEAK, t::PY_FOOD_HUNGRY, t::PY_FOOD_FULL, t::PY_FOOD_MAX) }
}

#[cfg(test)]
mod tests {
    use super::*;

    // `flag_has`/`flag_next` need no engine: these are the boundaries of
    // review finding 2 (docs/SELF-REVIEW.md F2).

    fn trap_with_index(t_idx: u8) -> headers::trap_h::trap {
        headers::trap_h::trap {
            t_idx,
            kind: core::ptr::null_mut(),
            next: core::ptr::null_mut(),
            grid: loc { x: 0, y: 0 },
            power: 0,
            timeout: 0,
            flags: [0; 2],
        }
    }

    /// The local 0098 guard: a grid whose rune is gone from the cave (NULL
    /// trap head, as after a rune is broken out of view) is not a glyph, and
    /// no trap kind is read (better-borg-c patch 0098 read through NULL).
    #[test]
    fn trap_head_is_glyph_without_a_trap_is_false() {
        // SAFETY: NULL is allowed.
        let glyph = unsafe {
            trap_head_is_glyph(core::ptr::null(), |_| panic!("kind read without a trap"))
        };
        assert!(!glyph);
    }

    #[test]
    fn trap_head_is_glyph_reads_the_head_kind() {
        let rune = trap_with_index(7);
        // SAFETY: a valid trap.
        unsafe {
            assert!(trap_head_is_glyph(&rune, |t_idx| t_idx == 7));
            assert!(!trap_head_is_glyph(&rune, |t_idx| t_idx == 3));
        }
    }

    #[test]
    fn flag_has_end_is_false_even_on_an_empty_set() {
        assert!(!flag_has(&[], FLAG_END));
        assert!(!flag_has(&[0xff], FLAG_END));
    }

    #[test]
    fn flag_has_matches_the_c_macros() {
        // FLAG_OFFSET(2) = 0, FLAG_BINARY(2) = 1 << 1
        assert!(flag_has(&[0x02], 2));
        assert!(!flag_has(&[0x02], 1));
        assert!(flag_has(&[0x01], 1));
        assert!(flag_has(&[0x80], 8));
        assert!(flag_has(&[0x00, 0x01], 9));
        assert!(flag_has(&[0x00, 0x80], 16));
        assert!(!flag_has(&[0xff, 0x7f], 16));
    }

    #[test]
    #[should_panic(expected = "out of range")]
    fn flag_has_past_an_empty_set_panics() {
        // the reviewer's reproducer: C read address 0x2
        flag_has(&[], 9);
    }

    #[test]
    #[should_panic(expected = "out of range")]
    fn flag_has_past_the_last_byte_panics() {
        // C read the byte after a one-byte allocation
        flag_has(&[0], 9);
    }

    #[test]
    #[should_panic(expected = "out of range")]
    fn flag_has_negative_panics() {
        flag_has(&[0xff; 8], -1);
    }

    #[test]
    fn flag_next_matches_the_c_loop() {
        let f = [0b0000_0101u8, 0, 0b1000_0000];
        assert_eq!(flag_next(&f, FLAG_START), 1);
        assert_eq!(flag_next(&f, 2), 3);
        assert_eq!(flag_next(&f, 4), 24);
        assert_eq!(flag_next(&f, 25), FLAG_END);
        // a start past FLAG_MAX reads nothing
        assert_eq!(flag_next(&f, 1000), FLAG_END);
        assert_eq!(flag_next(&[], FLAG_START), FLAG_END);
    }

    #[test]
    #[should_panic(expected = "before FLAG_START")]
    fn flag_next_from_flag_end_panics() {
        // C reads flags[-1]
        flag_next(&[0xff], FLAG_END);
    }

    #[test]
    #[should_panic(expected = "before FLAG_START")]
    fn flag_next_negative_panics() {
        flag_next(&[0xff], -5);
    }

    #[test]
    #[should_panic(expected = "without the Borg state's access token")]
    fn handle_outside_an_entry_panics() {
        // no entry is running in a unit test: a token of a fresh state is
        // refused (every handle constructor checks it before it reads
        // anything of the engine)
        let st = BorgState::default();
        check_access(&st.access);
    }
}

#[cfg(test)]
mod arith_tests {
    //! `distance` and `modify_stat_value` are computed in Rust, as C does
    //! (checked against the C functions on 547,157 inputs, see
    //! docs/REVIEW-RESPONSE-821cad310.md); the inputs where C's `int`
    //! arithmetic overflows panic (second review of 821cad310, finding 2).
    use super::{distance, loc, modify_stat_value};

    #[test]
    fn values_as_c() {
        assert_eq!(distance(loc { x: 0, y: 0 }, loc { x: 3, y: 4 }), 5);
        assert_eq!(distance(loc { x: 10, y: -2 }, loc { x: -10, y: -2 }), 20);
        assert_eq!(modify_stat_value(17, 2), 28);
        assert_eq!(modify_stat_value(28, -1), 18);
        assert_eq!(modify_stat_value(20, -1), 18);
        assert_eq!(modify_stat_value(5, -3), 3);
        assert_eq!(modify_stat_value(32760, 1), 32770i32 as i16); // C's int16_t
    }

    #[test]
    #[should_panic(expected = "overflows int")]
    fn distance_min_int() {
        distance(loc { x: i32::MIN, y: 0 }, loc { x: 1, y: 0 });
    }

    #[test]
    #[should_panic(expected = "overflows int")]
    fn stat_max_int() {
        modify_stat_value(i32::MAX, 1);
    }

    #[test]
    #[should_panic(expected = "overflows int")]
    fn stat_min_amount() {
        modify_stat_value(18, i32::MIN);
    }
}
