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

/// Unported boundaries retain their native decisions. R1's only port is
/// the pure wall_like helper; R2 supplies stateful decision adapters.
pub trait Shadow: Send + Sync {
    fn blow_effect(&self,name:&[u8],original:i32)->i32;
    fn thrust_damage(&self,state:&BorgState,slot:i32,original:i32)->i32;

    fn read_decision(&self,state:&BorgState,request:crate::decisions::ReadDecision)->crate::decisions::Value { request.native(state) }
    fn decision(&self, state: &mut BorgState, request: crate::decisions::Decision) -> crate::decisions::Value { request.native(state) }
    fn danger(&self, _state: &BorgState, _input: DangerInput, original: i32) -> i32 { original }
    fn monster_danger(&self, _state: &BorgState, _input: DangerInput, _slot: i32, original: i32) -> i32 { original }
    fn wall_like(&self, _turn: i32, _borg_turn: i32, _feat: u32, original: bool) -> bool { original }
    fn after_think(&self, _state: &BorgState) {}
}

static SHADOW: OnceLock<Box<dyn Shadow>> = OnceLock::new();

pub fn register_shadow(shadow: Box<dyn Shadow>) {
    let _ = SHADOW.set(shadow);
}


pub(crate) fn danger(state: &BorgState, input: DangerInput, original: i32) -> i32 {
    SHADOW.get().map_or(original, |s| s.danger(state, input, original))
}



pub(crate) fn wall_like(borg_turn: i32, feat: u32, original: bool) -> bool {
    SHADOW.get().map_or(original, |s| s.wall_like(crate::engine::game_turn(), borg_turn, feat, original))
}

pub(crate) fn after_think(state: &BorgState) {
    if let Some(s) = SHADOW.get() { s.after_think(state); }
}

// borg-reborn-rs (#281): standalone monster estimates share the complete port.
pub(crate) fn monster_danger(state: &BorgState, input: DangerInput, slot: i32, original: i32) -> i32 {
    SHADOW.get().map_or(original, |s| s.monster_danger(state,input,slot,original))
}

// A dependency or nested entry already belongs to its driving transaction.
pub(crate) fn decision(state:&mut BorgState,request:crate::decisions::Decision)->crate::decisions::Value {
    if crate::decision_tape::active() { return request.native(state); }
    match SHADOW.get() { Some(s)=>s.decision(state,request),None=>request.native(state) }
}

// borg-reborn-rs (#364): pure/read-only attack boundaries.
pub(crate) fn blow_effect(name:&[u8],original:i32)->i32 {
    SHADOW.get().map_or(original,|s|s.blow_effect(name,original))
}
pub(crate) fn thrust_damage(state:&BorgState,slot:i32,original:i32)->i32 {
    SHADOW.get().map_or(original,|s|s.thrust_damage(state,slot,original))
}

// #435: immutable predicate boundary. Stateful comparison remains unchanged.
pub(crate) fn read_decision(state:&BorgState,request:crate::decisions::ReadDecision)->crate::decisions::Value {
    if crate::decision_tape::active() { return request.native(state); }
    match SHADOW.get() {Some(s)=>s.read_decision(state,request),None=>request.native(state)}
}
