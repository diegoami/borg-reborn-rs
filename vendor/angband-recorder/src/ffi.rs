//! Linux engine ABI. The host calls this on its game thread. The clock wrapper
//! may be called from other threads. Forked savers bypass the inherited lock.
use crate::{Input, Metadata, Session, SCHEMA};
use serde_json::json;
use std::ffi::{c_char, c_long, CStr, CString, OsStr};
use std::os::unix::ffi::OsStrExt;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;

static SESSION: Mutex<Option<Session>> = Mutex::new(None);
static SNAPSHOT: AtomicBool = AtomicBool::new(false);
static PATH_RESULT: Mutex<Option<CString>> = Mutex::new(None);

extern "C" {
    fn __real_time(target: *mut c_long) -> c_long;
    fn ar_host_save(path: *const c_char) -> i32;
    fn ar_host_extra(full: bool) -> *const c_char;
    fn ar_host_free_extra(buffer: *const c_char);
    fn getpid() -> i32;
    fn _exit(code: i32) -> !;
    fn atexit(callback: extern "C" fn()) -> i32;
}
fn fatal(error: impl std::fmt::Display) -> ! {
    eprintln!("angband-recorder: {error}");
    unsafe { _exit(1) }
}
fn checked<T>(result: std::io::Result<T>) -> T {
    result.unwrap_or_else(|e| fatal(e))
}
fn with_session<T>(f: impl FnOnce(&mut Session) -> T) -> Option<T> {
    if SNAPSHOT.load(Ordering::Relaxed) {
        return None;
    }
    SESSION.lock().unwrap().as_mut().map(f)
}
unsafe fn text<'a>(value: *const c_char) -> &'a str {
    CStr::from_ptr(value).to_str().unwrap_or_else(|e| fatal(e))
}
fn env(primary: &str, legacy: &str) -> Option<std::ffi::OsString> {
    std::env::var_os(primary).or_else(|| std::env::var_os(legacy))
}

#[no_mangle]
pub unsafe extern "C" fn ar_init(build: *const c_char, geometry: *const i64) {
    let Some(output) = env("ANGBAND_RECORD_DIR", "BORG_RECORD_DIR") else {
        if env("ANGBAND_REPLAY_DIR", "BORG_REPLAY_DIR").is_some() {
            fatal("replay requires a separate recording output directory");
        }
        return;
    };
    let original = env("ANGBAND_REPLAY_DIR", "BORG_REPLAY_DIR");
    let geometry: [i64; 5] = std::slice::from_raw_parts(geometry, 5).try_into().unwrap();
    let metadata = Metadata {
        schema: SCHEMA.into(),
        build: text(build).into(),
        pid: getpid().into(),
        geometry,
    };
    let mut state = SESSION.lock().unwrap();
    if state.is_some() {
        fatal("recorder already initialized");
    }
    *state = Some(checked(Session::start(
        Path::new(&output),
        original.as_deref().map(Path::new),
        metadata,
    )));
    if atexit(ar_finish) != 0 {
        fatal("cannot register recording finalization");
    }
}
#[no_mangle]
pub extern "C" fn ar_active() -> bool {
    with_session(|_| true).unwrap_or(false)
}
#[no_mangle]
pub extern "C" fn ar_replaying() -> bool {
    with_session(|s| s.is_replay()).unwrap_or(false)
}
#[no_mangle]
pub extern "C" fn ar_in_snapshot() -> bool {
    SNAPSHOT.load(Ordering::Relaxed)
}
#[no_mangle]
pub extern "C" fn ar_snapshot_enter() {
    SNAPSHOT.store(true, Ordering::Relaxed);
}
#[no_mangle]
pub extern "C" fn ar_pid() -> i64 {
    with_session(|s| s.metadata.pid).unwrap_or_else(|| unsafe { getpid().into() })
}
#[no_mangle]
pub unsafe extern "C" fn __wrap_time(target: *mut c_long) -> c_long {
    let actual = __real_time(std::ptr::null_mut());
    let value = with_session(|s| checked(s.clock(actual))).unwrap_or(actual);
    if !target.is_null() {
        *target = value;
    }
    value
}
#[no_mangle]
pub unsafe extern "C" fn ar_poll(input: *mut Input) {
    if let Some(value) = with_session(|s| checked(s.poll(*input))) {
        *input = value;
    }
}
#[no_mangle]
pub extern "C" fn ar_exists(actual: bool) -> bool {
    with_session(|s| checked(s.exists(actual))).unwrap_or(actual)
}
#[no_mangle]
pub unsafe extern "C" fn ar_read_path(path: *const c_char) -> *const c_char {
    let path_bytes = CStr::from_ptr(path).to_bytes();
    let Some(output) =
        with_session(|s| checked(s.read_path(Path::new(OsStr::from_bytes(path_bytes)))))
    else {
        return path;
    };
    let mut result = PATH_RESULT.lock().unwrap();
    *result = Some(CString::new(output.as_os_str().as_bytes()).unwrap_or_else(|e| fatal(e)));
    result.as_ref().unwrap().as_ptr()
}
#[no_mangle]
pub extern "C" fn ar_bot(turn: i64, count: i64, event_type: i64, code: u32, mods: u8) {
    with_session(|s| checked(s.bot(turn, count, event_type, code, mods)));
}
#[repr(C)]
pub struct Cell {
    code: u32,
    attr: i32,
}
#[repr(C)]
pub struct Observation {
    turn: i64,
    decisions: i64,
    hp: i64,
    sp: i64,
    level: i64,
    depth: i64,
    gold: i64,
    dead: i64,
    x: i64,
    y: i64,
    max_hp: i64,
    max_sp: i64,
    quick: i64,
    quick_rng: i64,
    rng_index: i64,
    rng: [u32; 32],
    width: i32,
    height: i32,
    cells: *const Cell,
}
#[no_mangle]
pub unsafe extern "C" fn ar_frame(phase: *const c_char, observation: *const Observation) {
    with_session(|s| {
        let o = &*observation;
        if !(1..=255).contains(&o.width) || !(1..=255).contains(&o.height) {
            fatal("invalid frame geometry");
        }
        let size = (o.width, o.height);
        let full = s.previous_size != size || s.frame_count % 100 == 0;
        let current: Vec<_> = std::slice::from_raw_parts(o.cells, (o.width * o.height) as usize)
            .iter()
            .map(|c| (c.code, c.attr))
            .collect();
        let cells: Vec<_> = current
            .iter()
            .enumerate()
            .filter(|(i, c)| full || s.previous_cells.get(*i) != Some(c))
            .map(|(i, (code, attr))| (i, *code, *attr))
            .collect();
        s.previous_size = size;
        s.previous_cells = current;
        let phase = text(phase);
        let world = if phase == "tick" {
            None
        } else {
            let path = s.output.join("world-fingerprint.save");
            let c_path = CString::new(path.as_os_str().as_bytes()).unwrap_or_else(|e| fatal(e));
            if ar_host_save(c_path.as_ptr()) != 0 {
                fatal("forked world save failed");
            }
            Some(format!(
                "{:016x}",
                checked(std::fs::read(path))
                    .iter()
                    .fold(14695981039346656037u64, |h, b| (h ^ u64::from(*b))
                        .wrapping_mul(1099511628211))
            ))
        };
        let mut frame = json!({"index":s.frame_count+1,"phase":phase,"turn":o.turn,
            "decisions":o.decisions,"input":s.polls,"hp":o.hp,"sp":o.sp,"level":o.level,
            "depth":o.depth,"gold":o.gold,"dead":o.dead,"x":o.x,"y":o.y,
            "max_hp":o.max_hp,"max_sp":o.max_sp,"rng":o.rng,"quick":o.quick,
            "quick_rng":o.quick_rng,"rng_index":o.rng_index,"width":o.width,
            "height":o.height,"full":full,"cells":cells,"world":world});
        frame["rng"] = json!(o.rng.iter().map(|v| format!("{v:08x}")).collect::<String>());
        let extra = ar_host_extra(full);
        let observation: serde_json::Value =
            serde_json::from_str(text(extra)).unwrap_or_else(|e| fatal(e));
        ar_host_free_extra(extra);
        frame
            .as_object_mut()
            .unwrap()
            .extend(observation.as_object().unwrap().clone());
        checked(s.frame(&frame));
    });
}
#[no_mangle]
pub extern "C" fn ar_frame_count() -> u64 {
    with_session(|s| s.frame_count).unwrap_or(0)
}
#[no_mangle]
pub unsafe extern "C" fn ar_combat_message(turn: i64, name: *const c_char, message: *const c_char) {
    with_session(|s| {
        checked(s.combat(&json!({"kind":"message","frame":s.frame_count+1,"turn":turn,"type":text(name),"text":text(message)})))
    });
}
#[no_mangle]
pub unsafe extern "C" fn ar_damage(
    turn: i64,
    requested: i32,
    before: i32,
    after: i32,
    cause: *const c_char,
) {
    if before == after {
        return;
    }
    with_session(|s| {
        checked(s.combat(&json!({"kind":"damage","frame":s.frame_count+1,"turn":turn,"before":before,"after":after,"requested":requested,"cause":text(cause)})))
    });
}
#[no_mangle]
pub unsafe extern "C" fn ar_message(turn: i64, message_type: u16, message: *const c_char) {
    with_session(|s| {
        checked(s.message(
            &json!({"frame":s.frame_count,"turn":turn,"type":message_type,"text":text(message)}),
        ))
    });
}
#[no_mangle]
pub extern "C" fn ar_finish() {
    // Never emit a receipt from the forked saver, even if the host uses exit().
    with_session(|s| checked(s.finish()));
}
