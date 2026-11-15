//! borg-reborn-rs (#224): dependency-inverted comparison hooks.
//! The default is no registered implementation. Original decisions always
//! execute exactly once; the implementing crate owns ports and modes.
use crate::state::BorgState;
use serde::Serialize;
use std::sync::OnceLock;

#[derive(Clone, Copy, Debug, Serialize)]
pub struct DangerInput {
    pub y: i32,
    pub x: i32,
    pub turns: i32,
    pub average: bool,
    pub full: bool,
}

#[derive(Clone, Debug, Serialize)]
pub struct CautionResult {
    pub acted: bool,
    pub keys_before: Vec<u32>,
    pub keys_after: Vec<u32>,
}

/// Unported boundaries retain their native decisions. R1's only port is
/// the pure wall_like helper; R2 supplies stateful decision adapters.
pub trait Shadow: Send + Sync {
    fn danger(&self, _state: &BorgState, _input: DangerInput, original: i32) -> i32 { original }
    fn before_caution(&self, _state: &BorgState) {}
    fn caution(&self, _state: &BorgState, _original: &CautionResult) {}
    fn wall_like(&self, _turn: i32, _borg_turn: i32, _feat: u32, original: bool) -> bool { original }
    fn after_think(&self, _state: &BorgState) {}
}

static SHADOW: OnceLock<Box<dyn Shadow>> = OnceLock::new();

pub fn register_shadow(shadow: Box<dyn Shadow>) {
    let _ = SHADOW.set(shadow);
}

pub(crate) fn enabled() -> bool { SHADOW.get().is_some() }

pub(crate) fn danger(state: &BorgState, input: DangerInput, original: i32) -> i32 {
    SHADOW.get().map_or(original, |s| s.danger(state, input, original))
}

pub(crate) fn before_caution(state: &BorgState) {
    if let Some(s) = SHADOW.get() { s.before_caution(state); }
}

pub(crate) fn caution(state: &BorgState, result: &CautionResult) {
    if let Some(s) = SHADOW.get() { s.caution(state, result); }
}

pub(crate) fn wall_like(borg_turn: i32, feat: u32, original: bool) -> bool {
    SHADOW.get().map_or(original, |s| s.wall_like(crate::engine::game_turn(), borg_turn, feat, original))
}

pub(crate) fn after_think(state: &BorgState) {
    if let Some(s) = SHADOW.get() { s.after_think(state); }
}
