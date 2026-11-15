//! borg-reborn-rs (#286): closed effect/query boundary for stateful ports.
//! Generated from the explicit native call surface; each wrapper is audited.
#![allow(unused_imports)]
pub mod borg_cave_util {
use crate::{state::BorgState, borg::BorgTime, headers, headers::z_type_h::loc};
use crate::borg_headers::borg_magic_h::borg_spells;
use crate::borg_update::Detect;
use crate::memory::BorgGrid;
pub use crate::borg_light::BorgNeed;
use crate::headers::ui_event_h::keycode_t;
pub fn borg_cave_floor_bold(st:&BorgState,y:i32,x:i32) -> bool {
if !crate::decision_tape::active() { return crate::borg_cave_util::borg_cave_floor_bold(st,y,x); }
crate::decision_tape::query(st,"borg_cave_util::borg_cave_floor_bold",format!("{:?}",(y,x,)),|| crate::borg_cave_util::borg_cave_floor_bold(st,y,x))
}
pub fn borg_cave_floor_grid(ag:&BorgGrid) -> bool {
if !crate::decision_tape::active() { return crate::borg_cave_util::borg_cave_floor_grid(ag); }
crate::decision_tape::engine("borg_cave_util::borg_cave_floor_grid",format!("{:?}",(ag,)),|| crate::borg_cave_util::borg_cave_floor_grid(ag))
}
pub fn borg_panel_hgt() -> i32 {
if !crate::decision_tape::active() { return crate::borg_cave_util::borg_panel_hgt(); }
crate::decision_tape::engine("borg_cave_util::borg_panel_hgt",format!("{:?}",()),|| crate::borg_cave_util::borg_panel_hgt())
}
pub fn borg_panel_wid() -> i32 {
if !crate::decision_tape::active() { return crate::borg_cave_util::borg_panel_wid(); }
crate::decision_tape::engine("borg_cave_util::borg_panel_wid",format!("{:?}",()),|| crate::borg_cave_util::borg_panel_wid())
}
}
pub mod borg_danger {
use crate::{state::BorgState, borg::BorgTime, headers, headers::z_type_h::loc};
use crate::borg_headers::borg_magic_h::borg_spells;
use crate::borg_update::Detect;
use crate::memory::BorgGrid;
pub use crate::borg_light::BorgNeed;
use crate::headers::ui_event_h::keycode_t;
pub fn borg_danger(st:&mut BorgState,y:i32,x:i32,c:i32,average:bool,full_damage:bool) -> i32 {
if !crate::decision_tape::active() { return crate::borg_danger::borg_danger(st,y,x,c,average,full_damage); }
crate::decision_tape::effect(st,"borg_danger::borg_danger",format!("{:?}",(y,x,c,average,full_damage,)),|st| crate::borg_danger::borg_danger(st,y,x,c,average,full_damage))
}
}
pub mod borg_fight_attack {
use crate::{state::BorgState, borg::BorgTime, headers, headers::z_type_h::loc};
use crate::borg_headers::borg_magic_h::borg_spells;
use crate::borg_update::Detect;
use crate::memory::BorgGrid;
pub use crate::borg_light::BorgNeed;
use crate::headers::ui_event_h::keycode_t;
pub fn borg_thrust_damage_one(st:&BorgState,i:i32) -> i32 {
if !crate::decision_tape::active() { return crate::borg_fight_attack::borg_thrust_damage_one(st,i); }
crate::decision_tape::query(st,"borg_fight_attack::borg_thrust_damage_one",format!("{:?}",(i,)),|| crate::borg_fight_attack::borg_thrust_damage_one(st,i))
}
}
pub mod borg_fight_defend {
use crate::{state::BorgState, borg::BorgTime, headers, headers::z_type_h::loc};
use crate::borg_headers::borg_magic_h::borg_spells;
use crate::borg_update::Detect;
use crate::memory::BorgGrid;
pub use crate::borg_light::BorgNeed;
use crate::headers::ui_event_h::keycode_t;
pub fn borg_defend(st:&mut BorgState,p1:i32) -> bool {
if !crate::decision_tape::active() { return crate::borg_fight_defend::borg_defend(st,p1); }
crate::decision_tape::effect(st,"borg_fight_defend::borg_defend",format!("{:?}",(p1,)),|st| crate::borg_fight_defend::borg_defend(st,p1))
}
}
pub mod borg_flow_kill {
use crate::{state::BorgState, borg::BorgTime, headers, headers::z_type_h::loc};
use crate::borg_headers::borg_magic_h::borg_spells;
use crate::borg_update::Detect;
use crate::memory::BorgGrid;
pub use crate::borg_light::BorgNeed;
use crate::headers::ui_event_h::keycode_t;
pub fn borg_shoot_scoot_safe(st:&mut BorgState,emergency:i32,turns:i32,b_p:i32) -> bool {
if !crate::decision_tape::active() { return crate::borg_flow_kill::borg_shoot_scoot_safe(st,emergency,turns,b_p); }
crate::decision_tape::effect(st,"borg_flow_kill::borg_shoot_scoot_safe",format!("{:?}",(emergency,turns,b_p,)),|st| crate::borg_flow_kill::borg_shoot_scoot_safe(st,emergency,turns,b_p))
}
}
pub mod borg_flow_misc {
use crate::{state::BorgState, borg::BorgTime, headers, headers::z_type_h::loc};
use crate::borg_headers::borg_magic_h::borg_spells;
use crate::borg_update::Detect;
use crate::memory::BorgGrid;
pub use crate::borg_light::BorgNeed;
use crate::headers::ui_event_h::keycode_t;
pub fn borg_check_rest(st:&mut BorgState,y:i32,x:i32) -> bool {
if !crate::decision_tape::active() { return crate::borg_flow_misc::borg_check_rest(st,y,x); }
crate::decision_tape::effect(st,"borg_flow_misc::borg_check_rest",format!("{:?}",(y,x,)),|st| crate::borg_flow_misc::borg_check_rest(st,y,x))
}
pub fn borg_goto_dir(st:&BorgState,y1:i32,x1:i32,y2:i32,x2:i32) -> i32 {
if !crate::decision_tape::active() { return crate::borg_flow_misc::borg_goto_dir(st,y1,x1,y2,x2); }
crate::decision_tape::query(st,"borg_flow_misc::borg_goto_dir",format!("{:?}",(y1,x1,y2,x2,)),|| crate::borg_flow_misc::borg_goto_dir(st,y1,x1,y2,x2))
}
pub fn borg_happy_grid_bold(st:&BorgState,y:i32,x:i32) -> bool {
if !crate::decision_tape::active() { return crate::borg_flow_misc::borg_happy_grid_bold(st,y,x); }
crate::decision_tape::query(st,"borg_flow_misc::borg_happy_grid_bold",format!("{:?}",(y,x,)),|| crate::borg_flow_misc::borg_happy_grid_bold(st,y,x))
}
}
pub mod borg_flow_stairs {
use crate::{state::BorgState, borg::BorgTime, headers, headers::z_type_h::loc};
use crate::borg_headers::borg_magic_h::borg_spells;
use crate::borg_update::Detect;
use crate::memory::BorgGrid;
pub use crate::borg_light::BorgNeed;
use crate::headers::ui_event_h::keycode_t;
pub fn borg_flow_stair_less(st:&mut BorgState,why:i32,sneak:bool) -> bool {
if !crate::decision_tape::active() { return crate::borg_flow_stairs::borg_flow_stair_less(st,why,sneak); }
crate::decision_tape::effect(st,"borg_flow_stairs::borg_flow_stair_less",format!("{:?}",(why,sneak,)),|st| crate::borg_flow_stairs::borg_flow_stair_less(st,why,sneak))
}
pub fn borg_flow_stair_more(st:&mut BorgState,why:i32,sneak:bool,brave:bool) -> bool {
if !crate::decision_tape::active() { return crate::borg_flow_stairs::borg_flow_stair_more(st,why,sneak,brave); }
crate::decision_tape::effect(st,"borg_flow_stairs::borg_flow_stair_more",format!("{:?}",(why,sneak,brave,)),|st| crate::borg_flow_stairs::borg_flow_stair_more(st,why,sneak,brave))
}
pub fn borg_prep_leave_level_spells(st:&mut BorgState) -> bool {
if !crate::decision_tape::active() { return crate::borg_flow_stairs::borg_prep_leave_level_spells(st); }
crate::decision_tape::effect(st,"borg_flow_stairs::borg_prep_leave_level_spells",format!("{:?}",()),|st| crate::borg_flow_stairs::borg_prep_leave_level_spells(st))
}
}
pub mod borg_inventory {
use crate::{state::BorgState, borg::BorgTime, headers, headers::z_type_h::loc};
use crate::borg_headers::borg_magic_h::borg_spells;
use crate::borg_update::Detect;
use crate::memory::BorgGrid;
pub use crate::borg_light::BorgNeed;
use crate::headers::ui_event_h::keycode_t;
pub fn borg_slot(st:&mut BorgState,tval:i32,sval:i32) -> i32 {
if !crate::decision_tape::active() { return crate::borg_inventory::borg_slot(st,tval,sval); }
crate::decision_tape::effect(st,"borg_inventory::borg_slot",format!("{:?}",(tval,sval,)),|st| crate::borg_inventory::borg_slot(st,tval,sval))
}
}
pub mod borg_io {
use crate::{state::BorgState, borg::BorgTime, headers, headers::z_type_h::loc};
use crate::borg_headers::borg_magic_h::borg_spells;
use crate::borg_update::Detect;
use crate::memory::BorgGrid;
pub use crate::borg_light::BorgNeed;
use crate::headers::ui_event_h::keycode_t;
pub use crate::borg_io::ESCAPE;
pub use crate::borg_io::KC_ENTER;
pub fn borg_keypress(st:&mut BorgState,k:keycode_t) -> i32 {
if !crate::decision_tape::active() { return crate::borg_io::borg_keypress(st,k); }
crate::decision_tape::effect(st,"borg_io::borg_keypress",format!("{:?}",(k,)),|st| crate::borg_io::borg_keypress(st,k))
}
pub fn borg_note(st:&mut BorgState,what:&str)  {
if !crate::decision_tape::active() { return crate::borg_io::borg_note(st,what); }
crate::decision_tape::effect(st,"borg_io::borg_note",format!("{:?}",(what,)),|st| crate::borg_io::borg_note(st,what))
}
pub fn borg_note_bytes(st:&mut BorgState,what:&[u8])  {
if !crate::decision_tape::active() { return crate::borg_io::borg_note_bytes(st,what); }
crate::decision_tape::effect(st,"borg_io::borg_note_bytes",format!("{:?}",(what,)),|st| crate::borg_io::borg_note_bytes(st,what))
}
}
pub mod borg_item_use {
use crate::{state::BorgState, borg::BorgTime, headers, headers::z_type_h::loc};
use crate::borg_headers::borg_magic_h::borg_spells;
use crate::borg_update::Detect;
use crate::memory::BorgGrid;
pub use crate::borg_light::BorgNeed;
use crate::headers::ui_event_h::keycode_t;
pub fn borg_activate_failure(st:&mut BorgState,tval:i32,sval:i32) -> i32 {
if !crate::decision_tape::active() { return crate::borg_item_use::borg_activate_failure(st,tval,sval); }
crate::decision_tape::effect(st,"borg_item_use::borg_activate_failure",format!("{:?}",(tval,sval,)),|st| crate::borg_item_use::borg_activate_failure(st,tval,sval))
}
pub fn borg_activate_item(st:&mut BorgState,activation:i32) -> bool {
if !crate::decision_tape::active() { return crate::borg_item_use::borg_activate_item(st,activation); }
crate::decision_tape::effect(st,"borg_item_use::borg_activate_item",format!("{:?}",(activation,)),|st| crate::borg_item_use::borg_activate_item(st,activation))
}
pub fn borg_eat(st:&mut BorgState,tval:i32,sval:i32) -> bool {
if !crate::decision_tape::active() { return crate::borg_item_use::borg_eat(st,tval,sval); }
crate::decision_tape::effect(st,"borg_item_use::borg_eat",format!("{:?}",(tval,sval,)),|st| crate::borg_item_use::borg_eat(st,tval,sval))
}
pub fn borg_eat_food_any(st:&mut BorgState) -> bool {
if !crate::decision_tape::active() { return crate::borg_item_use::borg_eat_food_any(st); }
crate::decision_tape::effect(st,"borg_item_use::borg_eat_food_any",format!("{:?}",()),|st| crate::borg_item_use::borg_eat_food_any(st))
}
pub fn borg_eat_unknown(st:&mut BorgState) -> bool {
if !crate::decision_tape::active() { return crate::borg_item_use::borg_eat_unknown(st); }
crate::decision_tape::effect(st,"borg_item_use::borg_eat_unknown",format!("{:?}",()),|st| crate::borg_item_use::borg_eat_unknown(st))
}
pub fn borg_equips_staff_fail(st:&mut BorgState,sval:i32) -> bool {
if !crate::decision_tape::active() { return crate::borg_item_use::borg_equips_staff_fail(st,sval); }
crate::decision_tape::effect(st,"borg_item_use::borg_equips_staff_fail",format!("{:?}",(sval,)),|st| crate::borg_item_use::borg_equips_staff_fail(st,sval))
}
pub fn borg_quaff_crit(st:&mut BorgState,no_check:bool) -> bool {
if !crate::decision_tape::active() { return crate::borg_item_use::borg_quaff_crit(st,no_check); }
crate::decision_tape::effect(st,"borg_item_use::borg_quaff_crit",format!("{:?}",(no_check,)),|st| crate::borg_item_use::borg_quaff_crit(st,no_check))
}
pub fn borg_quaff_potion(st:&mut BorgState,sval:i32) -> bool {
if !crate::decision_tape::active() { return crate::borg_item_use::borg_quaff_potion(st,sval); }
crate::decision_tape::effect(st,"borg_item_use::borg_quaff_potion",format!("{:?}",(sval,)),|st| crate::borg_item_use::borg_quaff_potion(st,sval))
}
pub fn borg_quaff_unknown(st:&mut BorgState) -> bool {
if !crate::decision_tape::active() { return crate::borg_item_use::borg_quaff_unknown(st); }
crate::decision_tape::effect(st,"borg_item_use::borg_quaff_unknown",format!("{:?}",()),|st| crate::borg_item_use::borg_quaff_unknown(st))
}
pub fn borg_read_scroll(st:&mut BorgState,sval:i32) -> bool {
if !crate::decision_tape::active() { return crate::borg_item_use::borg_read_scroll(st,sval); }
crate::decision_tape::effect(st,"borg_item_use::borg_read_scroll",format!("{:?}",(sval,)),|st| crate::borg_item_use::borg_read_scroll(st,sval))
}
pub fn borg_read_unknown(st:&mut BorgState) -> bool {
if !crate::decision_tape::active() { return crate::borg_item_use::borg_read_unknown(st); }
crate::decision_tape::effect(st,"borg_item_use::borg_read_unknown",format!("{:?}",()),|st| crate::borg_item_use::borg_read_unknown(st))
}
pub fn borg_use_staff(st:&mut BorgState,sval:i32) -> bool {
if !crate::decision_tape::active() { return crate::borg_item_use::borg_use_staff(st,sval); }
crate::decision_tape::effect(st,"borg_item_use::borg_use_staff",format!("{:?}",(sval,)),|st| crate::borg_item_use::borg_use_staff(st,sval))
}
pub fn borg_use_staff_fail(st:&mut BorgState,sval:i32) -> bool {
if !crate::decision_tape::active() { return crate::borg_item_use::borg_use_staff_fail(st,sval); }
crate::decision_tape::effect(st,"borg_item_use::borg_use_staff_fail",format!("{:?}",(sval,)),|st| crate::borg_item_use::borg_use_staff_fail(st,sval))
}
pub fn borg_use_unknown(st:&mut BorgState) -> bool {
if !crate::decision_tape::active() { return crate::borg_item_use::borg_use_unknown(st); }
crate::decision_tape::effect(st,"borg_item_use::borg_use_unknown",format!("{:?}",()),|st| crate::borg_item_use::borg_use_unknown(st))
}
pub fn borg_zap_rod(st:&mut BorgState,sval:i32) -> bool {
if !crate::decision_tape::active() { return crate::borg_item_use::borg_zap_rod(st,sval); }
crate::decision_tape::effect(st,"borg_item_use::borg_zap_rod",format!("{:?}",(sval,)),|st| crate::borg_item_use::borg_zap_rod(st,sval))
}
}
pub mod borg_item_val {
use crate::{state::BorgState, borg::BorgTime, headers, headers::z_type_h::loc};
use crate::borg_headers::borg_magic_h::borg_spells;
use crate::borg_update::Detect;
use crate::memory::BorgGrid;
pub use crate::borg_light::BorgNeed;
use crate::headers::ui_event_h::keycode_t;
pub fn borg_lookup_kind(st:&mut BorgState,tval:i32,sval:i32) -> i32 {
if !crate::decision_tape::active() { return crate::borg_item_val::borg_lookup_kind(st,tval,sval); }
crate::decision_tape::effect(st,"borg_item_val::borg_lookup_kind",format!("{:?}",(tval,sval,)),|st| crate::borg_item_val::borg_lookup_kind(st,tval,sval))
}
}
pub mod borg_item_wear {
use crate::{state::BorgState, borg::BorgTime, headers, headers::z_type_h::loc};
use crate::borg_headers::borg_magic_h::borg_spells;
use crate::borg_update::Detect;
use crate::memory::BorgGrid;
pub use crate::borg_light::BorgNeed;
use crate::headers::ui_event_h::keycode_t;
pub fn borg_backup_swap(st:&mut BorgState,p:i32) -> bool {
if !crate::decision_tape::active() { return crate::borg_item_wear::borg_backup_swap(st,p); }
crate::decision_tape::effect(st,"borg_item_wear::borg_backup_swap",format!("{:?}",(p,)),|st| crate::borg_item_wear::borg_backup_swap(st,p))
}
}
pub mod borg_light {
use crate::{state::BorgState, borg::BorgTime, headers, headers::z_type_h::loc};
use crate::borg_headers::borg_magic_h::borg_spells;
use crate::borg_update::Detect;
use crate::memory::BorgGrid;
pub use crate::borg_light::BorgNeed;
use crate::headers::ui_event_h::keycode_t;
pub fn borg_check_light_only(st:&mut BorgState) -> bool {
if !crate::decision_tape::active() { return crate::borg_light::borg_check_light_only(st); }
crate::decision_tape::effect(st,"borg_light::borg_check_light_only",format!("{:?}",()),|st| crate::borg_light::borg_check_light_only(st))
}
pub fn borg_maintain_light(st:&mut BorgState) -> BorgNeed {
if !crate::decision_tape::active() { return crate::borg_light::borg_maintain_light(st); }
crate::decision_tape::effect(st,"borg_light::borg_maintain_light",format!("{:?}",()),|st| crate::borg_light::borg_maintain_light(st))
}
}
pub mod borg_magic {
use crate::{state::BorgState, borg::BorgTime, headers, headers::z_type_h::loc};
use crate::borg_headers::borg_magic_h::borg_spells;
use crate::borg_update::Detect;
use crate::memory::BorgGrid;
pub use crate::borg_light::BorgNeed;
use crate::headers::ui_event_h::keycode_t;
pub fn borg_spell(st:&mut BorgState,spell:borg_spells) -> bool {
if !crate::decision_tape::active() { return crate::borg_magic::borg_spell(st,spell); }
crate::decision_tape::effect(st,"borg_magic::borg_spell",format!("{:?}",(spell,)),|st| crate::borg_magic::borg_spell(st,spell))
}
pub fn borg_spell_fail(st:&mut BorgState,spell:borg_spells,allow_fail:i32) -> bool {
if !crate::decision_tape::active() { return crate::borg_magic::borg_spell_fail(st,spell,allow_fail); }
crate::decision_tape::effect(st,"borg_magic::borg_spell_fail",format!("{:?}",(spell,allow_fail,)),|st| crate::borg_magic::borg_spell_fail(st,spell,allow_fail))
}
pub fn borg_spell_legal(st:&BorgState,spell:borg_spells) -> bool {
if !crate::decision_tape::active() { return crate::borg_magic::borg_spell_legal(st,spell); }
crate::decision_tape::query(st,"borg_magic::borg_spell_legal",format!("{:?}",(spell,)),|| crate::borg_magic::borg_spell_legal(st,spell))
}
pub fn borg_spell_okay(st:&BorgState,spell:borg_spells) -> bool {
if !crate::decision_tape::active() { return crate::borg_magic::borg_spell_okay(st,spell); }
crate::decision_tape::query(st,"borg_magic::borg_spell_okay",format!("{:?}",(spell,)),|| crate::borg_magic::borg_spell_okay(st,spell))
}
pub fn borg_spell_okay_fail(st:&BorgState,spell:borg_spells,allow_fail:i32) -> bool {
if !crate::decision_tape::active() { return crate::borg_magic::borg_spell_okay_fail(st,spell,allow_fail); }
crate::decision_tape::query(st,"borg_magic::borg_spell_okay_fail",format!("{:?}",(spell,allow_fail,)),|| crate::borg_magic::borg_spell_okay_fail(st,spell,allow_fail))
}
}
pub mod borg_prepared {
use crate::{state::BorgState, borg::BorgTime, headers, headers::z_type_h::loc};
use crate::borg_headers::borg_magic_h::borg_spells;
use crate::borg_update::Detect;
use crate::memory::BorgGrid;
pub use crate::borg_light::BorgNeed;
use crate::headers::ui_event_h::keycode_t;
pub fn borg_prepared_c(st:&mut BorgState,depth:i32) -> Option<Vec<u8>> {
if !crate::decision_tape::active() { return crate::borg_prepared::borg_prepared_c(st,depth); }
crate::decision_tape::effect(st,"borg_prepared::borg_prepared_c",format!("{:?}",(depth,)),|st| crate::borg_prepared::borg_prepared_c(st,depth))
}
pub fn borg_restock_c(st:&mut BorgState,depth:i32) -> Option<Vec<u8>> {
if !crate::decision_tape::active() { return crate::borg_prepared::borg_restock_c(st,depth); }
crate::decision_tape::effect(st,"borg_prepared::borg_restock_c",format!("{:?}",(depth,)),|st| crate::borg_prepared::borg_restock_c(st,depth))
}
}
pub mod borg_projection {
use crate::{state::BorgState, borg::BorgTime, headers, headers::z_type_h::loc};
use crate::borg_headers::borg_magic_h::borg_spells;
use crate::borg_update::Detect;
use crate::memory::BorgGrid;
pub use crate::borg_light::BorgNeed;
use crate::headers::ui_event_h::keycode_t;
pub fn borg_distance(y:i32,x:i32,y2:i32,x2:i32) -> i32 {
if !crate::decision_tape::active() { return crate::borg_projection::borg_distance(y,x,y2,x2); }
crate::decision_tape::engine("borg_projection::borg_distance",format!("{:?}",(y,x,y2,x2,)),|| crate::borg_projection::borg_distance(y,x,y2,x2))
}
pub fn borg_los(st:&BorgState,y1:i32,x1:i32,y2:i32,x2:i32) -> bool {
if !crate::decision_tape::active() { return crate::borg_projection::borg_los(st,y1,x1,y2,x2); }
crate::decision_tape::query(st,"borg_projection::borg_los",format!("{:?}",(y1,x1,y2,x2,)),|| crate::borg_projection::borg_los(st,y1,x1,y2,x2))
}
pub fn borg_target(st:&mut BorgState,t:loc,require_monster:bool) -> bool {
if !crate::decision_tape::active() { return crate::borg_projection::borg_target(st,t,require_monster); }
crate::decision_tape::effect(st,"borg_projection::borg_target",format!("{:?}",(t,require_monster,)),|st| crate::borg_projection::borg_target(st,t,require_monster))
}
}
pub mod borg_store_sell {
use crate::{state::BorgState, borg::BorgTime, headers, headers::z_type_h::loc};
use crate::borg_headers::borg_magic_h::borg_spells;
use crate::borg_update::Detect;
use crate::memory::BorgGrid;
pub use crate::borg_light::BorgNeed;
use crate::headers::ui_event_h::keycode_t;
pub fn borg_count_sell(st:&mut BorgState) -> i32 {
if !crate::decision_tape::active() { return crate::borg_store_sell::borg_count_sell(st); }
crate::decision_tape::effect(st,"borg_store_sell::borg_count_sell",format!("{:?}",()),|st| crate::borg_store_sell::borg_count_sell(st))
}
}
pub mod borg_trait {
use crate::{state::BorgState, borg::BorgTime, headers, headers::z_type_h::loc};
use crate::borg_headers::borg_magic_h::borg_spells;
use crate::borg_update::Detect;
use crate::memory::BorgGrid;
pub use crate::borg_light::BorgNeed;
use crate::headers::ui_event_h::keycode_t;
pub fn borg_uses_swaps(st:&BorgState) -> bool {
if !crate::decision_tape::active() { return crate::borg_trait::borg_uses_swaps(st); }
crate::decision_tape::query(st,"borg_trait::borg_uses_swaps",format!("{:?}",()),|| crate::borg_trait::borg_uses_swaps(st))
}
}
pub mod engine {
use crate::{state::BorgState, borg::BorgTime, headers, headers::z_type_h::loc};
use crate::borg_headers::borg_magic_h::borg_spells;
use crate::borg_update::Detect;
use crate::memory::BorgGrid;
pub use crate::borg_light::BorgNeed;
use crate::headers::ui_event_h::keycode_t;
pub fn ddd(i:usize) -> i16 {
if !crate::decision_tape::active() { return crate::engine::ddd(i); }
crate::decision_tape::engine("engine::ddd",format!("{:?}",(i,)),|| crate::engine::ddd(i))
}
pub fn ddx(dir:usize) -> i16 {
if !crate::decision_tape::active() { return crate::engine::ddx(dir); }
crate::decision_tape::engine("engine::ddx",format!("{:?}",(dir,)),|| crate::engine::ddx(dir))
}
pub fn ddx_ddd(i:usize) -> i16 {
if !crate::decision_tape::active() { return crate::engine::ddx_ddd(i); }
crate::decision_tape::engine("engine::ddx_ddd",format!("{:?}",(i,)),|| crate::engine::ddx_ddd(i))
}
pub fn ddy(dir:usize) -> i16 {
if !crate::decision_tape::active() { return crate::engine::ddy(dir); }
crate::decision_tape::engine("engine::ddy",format!("{:?}",(dir,)),|| crate::engine::ddy(dir))
}
pub fn ddy_ddd(i:usize) -> i16 {
if !crate::decision_tape::active() { return crate::engine::ddy_ddd(i); }
crate::decision_tape::engine("engine::ddy_ddd",format!("{:?}",(i,)),|| crate::engine::ddy_ddd(i))
}
pub fn distance(grid1:loc,grid2:loc) -> i32 {
if !crate::decision_tape::active() { return crate::engine::distance(grid1,grid2); }
crate::decision_tape::engine("engine::distance",format!("{:?}",(grid1,grid2,)),|| crate::engine::distance(grid1,grid2))
}
pub fn feat_is_shop(feat:i32) -> bool {
if !crate::decision_tape::active() { return crate::engine::feat_is_shop(feat); }
crate::decision_tape::engine("engine::feat_is_shop",format!("{:?}",(feat,)),|| crate::engine::feat_is_shop(feat))
}
pub fn player_arena_level() -> bool {
if !crate::decision_tape::active() { return crate::engine::player_arena_level(); }
crate::decision_tape::engine("engine::player_arena_level",format!("{:?}",()),|| crate::engine::player_arena_level())
}
pub fn player_opt(opt:u32) -> bool {
if !crate::decision_tape::active() { return crate::engine::player_opt(opt); }
crate::decision_tape::engine("engine::player_opt",format!("{:?}",(opt,)),|| crate::engine::player_opt(opt))
}
pub fn player_timed(idx:u32) -> i16 {
if !crate::decision_tape::active() { return crate::engine::player_timed(idx); }
crate::decision_tape::engine("engine::player_timed",format!("{:?}",(idx,)),|| crate::engine::player_timed(idx))
}
pub fn r_max() -> usize {
if !crate::decision_tape::active() { return crate::engine::r_max(); }
crate::decision_tape::engine("engine::r_max",format!("{:?}",()),|| crate::engine::r_max())
}
pub fn race_flag_has(r_idx:usize,flag:u32) -> bool {
if !crate::decision_tape::active() { return crate::engine::race_flag_has(r_idx,flag); }
crate::decision_tape::engine("engine::race_flag_has",format!("{:?}",(r_idx,flag,)),|| crate::engine::race_flag_has(r_idx,flag))
}
pub fn randint0(m:u32) -> i32 {
if !crate::decision_tape::active() { return crate::engine::randint0(m); }
crate::decision_tape::engine("engine::randint0",format!("{:?}",(m,)),|| crate::engine::randint0(m))
}
pub fn square_in_bounds(x:i32,y:i32) -> bool {
if !crate::decision_tape::active() { return crate::engine::square_in_bounds(x,y); }
crate::decision_tape::engine("engine::square_in_bounds",format!("{:?}",(x,y,)),|| crate::engine::square_in_bounds(x,y))
}
pub fn square_in_bounds_fully(x:i32,y:i32) -> bool {
if !crate::decision_tape::active() { return crate::engine::square_in_bounds_fully(x,y); }
crate::decision_tape::engine("engine::square_in_bounds_fully",format!("{:?}",(x,y,)),|| crate::engine::square_in_bounds_fully(x,y))
}
pub fn square_isno_teleport(x:i32,y:i32) -> bool {
if !crate::decision_tape::active() { return crate::engine::square_isno_teleport(x,y); }
crate::decision_tape::engine("engine::square_isno_teleport",format!("{:?}",(x,y,)),|| crate::engine::square_isno_teleport(x,y))
}
pub fn square_isvault(x:i32,y:i32) -> bool {
if !crate::decision_tape::active() { return crate::engine::square_isvault(x,y); }
crate::decision_tape::engine("engine::square_isvault",format!("{:?}",(x,y,)),|| crate::engine::square_isvault(x,y))
}
pub fn z_info() -> &'static headers::init_h::angband_constants {
if !crate::decision_tape::active() { return crate::engine::z_info(); }
crate::decision_tape::engine("engine::z_info",format!("{:?}",()),|| crate::engine::z_info())
}
}
pub mod shell {
use crate::{state::BorgState, borg::BorgTime, headers, headers::z_type_h::loc};
use crate::borg_headers::borg_magic_h::borg_spells;
use crate::borg_update::Detect;
use crate::memory::BorgGrid;
pub use crate::borg_light::BorgNeed;
use crate::headers::ui_event_h::keycode_t;
pub fn shopping_departure(st:&mut BorgState) -> bool {
if !crate::decision_tape::active() { return crate::shell::shopping_departure(st); }
crate::decision_tape::effect(st,"shell::shopping_departure",format!("{:?}",()),|st| crate::shell::shopping_departure(st))
}
}
pub mod headers {
pub mod list_mon_race_flags_h { pub use crate::headers::list_mon_race_flags_h::{RF_KILL_WALL,RF_PASS_WALL}; }
pub mod list_options_h { pub use crate::headers::list_options_h::{OPT_birth_force_descend}; }
pub mod list_player_timed_h { pub use crate::headers::list_player_timed_h::{TMD_OPP_ACID}; }
pub mod list_terrain_h { pub use crate::headers::list_terrain_h::{FEAT_LAVA,FEAT_LESS,FEAT_MORE,FEAT_NONE}; }
pub mod list_tvals_h { pub use crate::headers::list_tvals_h::{TV_FOOD,TV_MUSHROOM,TV_POTION,TV_ROD,TV_STAFF}; }
pub mod ui_event_h { pub use crate::headers::ui_event_h::{keycode_t}; }
pub mod z_type_h { pub use crate::headers::z_type_h::{loc}; }
}
pub mod borg_headers {
pub mod borg_h { pub use crate::borg_headers::borg_h::{BORG_PLAYS_RISKY}; }
pub mod borg_magic_h { pub use crate::borg_headers::borg_magic_h::{BERSERK_STRENGTH,CURE_POISON,DIMENSION_DOOR,HEALING,HERBAL_CURING,HEROISM,HOLY_WORD,MINOR_HEALING,PHASE_DOOR,PORTAL,RECHARGING,REMEMBRANCE,REMOVE_HUNGER,RESTORATION,REVITALIZE,SHADOW_SHIFT,TELEPORT_LEVEL,TELEPORT_SELF,UNHOLY_REPRIEVE,WORD_OF_RECALL}; }
pub mod borg_trait_h { pub use crate::borg_headers::borg_trait_h::{BI_ACCW,BI_AESCAPE,BI_AEZHEAL,BI_AFUEL,BI_AHEAL,BI_AMISSILES,BI_APHASE,BI_ATELEPORT,BI_CDEPTH,BI_CLASS,BI_CLEVEL,BI_CRSFEAR,BI_CRSNOTEL,BI_CURHP,BI_CURSP,BI_ESP,BI_FOOD,BI_GOLD,BI_IFIRE,BI_ISAFRAID,BI_ISBLIND,BI_ISCONFUSED,BI_ISCUT,BI_ISFIXALL,BI_ISFIXCON,BI_ISFIXDEX,BI_ISFIXEXP,BI_ISFIXINT,BI_ISFIXSTR,BI_ISFIXWIS,BI_ISHEAVYSTUN,BI_ISHUNGRY,BI_ISIMAGE,BI_ISPOISONED,BI_ISSTUN,BI_ISWEAK,BI_KING,BI_LIGHT,BI_MAXCLEVEL,BI_MAXDEPTH,BI_MAXHP,BI_MAXSP,BI_SPEED}; }
}
pub mod borg { pub use crate::borg::BorgTime; }
pub mod borg_cave { pub use crate::borg_cave::{AUTO_MAX_X,AUTO_MAX_Y}; }
pub mod borg_update { pub use crate::borg_update::Detect;
use crate::memory::BorgGrid; }
