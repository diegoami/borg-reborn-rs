//! The engine boundary: the only module allowed `unsafe` code and the only
//! one the engine calls into (ARCHITECTURE.md L0).
//!
//! T1.2 (#5) gives it the one symbol the engine needs to link with
//! `BORG_IMPL=rust`: `do_cmd_borg`, called by `do_cmd_try_borg` in
//! `angband/src/cmd-misc.c` for `^Z`. It does nothing yet, so no Borg
//! starts on this arm. T1.5 (#8) brings borg-rs's `engine.rs` and the
//! control loop.

/// The engine's call into the Borg commands (`^Z`). A no-op until T1.5.
#[allow(unsafe_code)]
#[no_mangle]
extern "C" fn do_cmd_borg() {}
