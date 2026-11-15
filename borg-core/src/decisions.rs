//! borg-reborn-rs (#286): typed requests for all stateful R2 entry points.
use crate::state::BorgState;
use serde::Serialize;
#[derive(Clone,Copy,Debug,PartialEq,Serialize)]
#[serde(tag="kind")]
pub enum Decision {
Caution,
Recall,
Surrounded,
Freedom { y:i32,x:i32 },
CautionPhase { emergency:i32,turns:i32 },
CautionTeleport { emergency:i32,turns:i32 },
AllowTeleport,
ShadowShift { allow_fail:i32 },
DimensionDoor { allow_fail:i32 },
Escape { b_q:i32 },
Recover,
}
#[derive(Clone,Copy,Debug,PartialEq,Serialize)]
pub enum Value { Acted(bool), Score(i32) }
impl Decision {
pub fn name(self)->&'static str { match self {
Self::Caution=>"caution.borg_caution",
Self::Recall=>"escape.borg_recall",
Self::Surrounded=>"escape.borg_surrounded",
Self::Freedom {..}=>"escape.borg_freedom",
Self::CautionPhase {..}=>"escape.borg_caution_phase",
Self::CautionTeleport {..}=>"escape.borg_caution_teleport",
Self::AllowTeleport=>"escape.borg_allow_teleport",
Self::ShadowShift {..}=>"escape.borg_shadow_shift",
Self::DimensionDoor {..}=>"escape.borg_dimension_door",
Self::Escape {..}=>"escape.borg_escape",
Self::Recover=>"recovery.borg_recover",
} }
pub fn native(self,st:&mut BorgState)->Value { match self {
Self::Caution=>Value::Acted(crate::borg_caution::borg_caution_native(st)),
Self::Recall=>Value::Acted(crate::borg_escape::borg_recall_native(st)),
Self::Surrounded=>Value::Acted(crate::borg_escape::borg_surrounded_native(st)),
Self::Freedom { y,x }=>Value::Score(crate::borg_escape::borg_freedom_native(st,y,x)),
Self::CautionPhase { emergency,turns }=>Value::Acted(crate::borg_escape::borg_caution_phase_native(st,emergency,turns)),
Self::CautionTeleport { emergency,turns }=>Value::Acted(crate::borg_escape::borg_caution_teleport_native(st,emergency,turns)),
Self::AllowTeleport=>Value::Acted(crate::borg_escape::borg_allow_teleport_native(st)),
Self::ShadowShift { allow_fail }=>Value::Acted(crate::borg_escape::borg_shadow_shift_native(st,allow_fail)),
Self::DimensionDoor { allow_fail }=>Value::Acted(crate::borg_escape::borg_dimension_door_native(st,allow_fail)),
Self::Escape { b_q }=>Value::Acted(crate::borg_escape::borg_escape_native(st,b_q)),
Self::Recover=>Value::Acted(crate::borg_recover::borg_recover_native(st)),
} }
}
