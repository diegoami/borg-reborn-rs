//! borg-reborn-rs (#286): value equality for owned native policy snapshots.
//! A snapshot never grants engine access: engine::check_access still requires
//! the actual root state's token. Engine effects are recorded/replayed separately.
use crate::headers::{ui_event_h::keypress, object_h::element_info};
impl PartialEq for keypress {
    fn eq(&self, other: &Self) -> bool { self.type_0==other.type_0 && self.code==other.code && self.mods==other.mods }
}
impl PartialEq for element_info {
    fn eq(&self, other: &Self) -> bool { self.res_level==other.res_level && self.flags==other.flags }
}
