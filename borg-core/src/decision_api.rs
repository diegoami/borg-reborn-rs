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

use crate::borg_headers::borg_item_h::borg_item;
use std::ffi::CStr;
pub fn borg_danger_one_kill(
    st: &mut BorgState,
    y: i32,
    x: i32,
    c: i32,
    i: i32,
    average: bool,
    full_damage: bool,
) -> i32 {
if !crate::decision_tape::active() { return crate::borg_danger::borg_danger_one_kill(st,y,x,c,i,average,full_damage); }
crate::decision_tape::effect(st,"borg_danger::borg_danger_one_kill",format!("{:?}",(y,x,c,i,average,full_damage,)),|st| crate::borg_danger::borg_danger_one_kill(st,y,x,c,i,average,full_damage))
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

use crate::borg_headers::borg_item_h::borg_item;
use crate::engine::TermGeometry;
use std::ffi::CStr;
pub fn borg_launch_bolt(st:&mut BorgState,rad:i32,dam:i32,typ:i32,max:i32,ammo_location:i32)->i32 {
if !crate::decision_tape::active() { return crate::borg_fight_attack::borg_launch_bolt(st,rad,dam,typ,max,ammo_location); }
crate::decision_tape::effect(st,"borg_fight_attack::borg_launch_bolt",format!("{:?}",(rad,dam,typ,max,ammo_location,)),|st| crate::borg_fight_attack::borg_launch_bolt(st,rad,dam,typ,max,ammo_location))
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

use crate::borg_headers::borg_item_h::borg_item;
use std::ffi::CStr;
pub fn borg_race_name(r_idx: i32) -> &'static CStr {
if !crate::decision_tape::active() { return crate::borg_flow_kill::borg_race_name(r_idx); }
crate::decision_tape::engine("borg_flow_kill::borg_race_name",format!("{:?}",(r_idx,)),|| crate::borg_flow_kill::borg_race_name(r_idx))
}

use crate::engine::TermGeometry;
pub fn borg_delete_kill(st: &mut BorgState, i: i32) {
if !crate::decision_tape::active() { return crate::borg_flow_kill::borg_delete_kill(st,i); }
crate::decision_tape::effect(st,"borg_flow_kill::borg_delete_kill",format!("{:?}",(i,)),|st| crate::borg_flow_kill::borg_delete_kill(st,i))
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

use crate::borg_headers::borg_item_h::borg_item;
use std::ffi::CStr;
pub fn borg_extract_dir(y1: i32, x1: i32, y2: i32, x2: i32) -> i32 {
if !crate::decision_tape::active() { return crate::borg_flow_misc::borg_extract_dir(y1,x1,y2,x2); }
crate::decision_tape::engine("borg_flow_misc::borg_extract_dir",format!("{:?}",(y1,x1,y2,x2,)),|| crate::borg_flow_misc::borg_extract_dir(y1,x1,y2,x2))
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

use crate::borg_headers::borg_item_h::borg_item;
use std::ffi::CStr;
pub fn borg_wield_slot(item: &borg_item) -> i32 {
if !crate::decision_tape::active() { return crate::borg_inventory::borg_wield_slot(item); }
crate::decision_tape::engine("borg_inventory::borg_wield_slot",format!("{:?}",(item,)),|| crate::borg_inventory::borg_wield_slot(item))
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
fn call(st:&mut BorgState,what:&[u8]) {
    // #435: a constructed copied memory cannot lend the real engine root.
    // Only this declared test-driver scope stubs the shared UI notification;
    // the ordered effect and complete input/output state are still taped.
    #[cfg(feature="test-r4-item-use")]
    if super::item_use_fixture::messages_stubbed() { return; }
    crate::borg_io::borg_note_bytes(st,what)
}
if !crate::decision_tape::active() { return call(st,what); }
crate::decision_tape::effect(st,"borg_io::borg_note_bytes",format!("{:?}",(what,)),|st| call(st,what))
}

use crate::borg_headers::borg_item_h::borg_item;
use std::ffi::CStr;
pub fn borg_queue_direction(st: &mut BorgState, k: keycode_t) {
if !crate::decision_tape::active() { return crate::borg_io::borg_queue_direction(st,k); }
crate::decision_tape::effect(st,"borg_io::borg_queue_direction",format!("{:?}",(k,)),|st| crate::borg_io::borg_queue_direction(st,k))
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

use crate::borg_headers::borg_item_h::borg_item;
use std::ffi::CStr;
pub fn borg_activate_dragon(st: &mut BorgState, drag_sval: i32) -> bool {
if !crate::decision_tape::active() { return crate::borg_item_use::borg_activate_dragon(st,drag_sval); }
crate::decision_tape::effect(st,"borg_item_use::borg_activate_dragon",format!("{:?}",(drag_sval,)),|st| crate::borg_item_use::borg_activate_dragon(st,drag_sval))
}
pub fn borg_activate_ring(st: &mut BorgState, ring_sval: i32) -> bool {
if !crate::decision_tape::active() { return crate::borg_item_use::borg_activate_ring(st,ring_sval); }
crate::decision_tape::effect(st,"borg_item_use::borg_activate_ring",format!("{:?}",(ring_sval,)),|st| crate::borg_item_use::borg_activate_ring(st,ring_sval))
}
pub fn borg_aim_wand(st: &mut BorgState, sval: i32) -> bool {
if !crate::decision_tape::active() { return crate::borg_item_use::borg_aim_wand(st,sval); }
crate::decision_tape::effect(st,"borg_item_use::borg_aim_wand",format!("{:?}",(sval,)),|st| crate::borg_item_use::borg_aim_wand(st,sval))
}
pub fn borg_equips_dragon(st: &mut BorgState, drag_sval: i32) -> bool {
if !crate::decision_tape::active() { return crate::borg_item_use::borg_equips_dragon(st,drag_sval); }
crate::decision_tape::effect(st,"borg_item_use::borg_equips_dragon",format!("{:?}",(drag_sval,)),|st| crate::borg_item_use::borg_equips_dragon(st,drag_sval))
}
pub fn borg_equips_item(st: &BorgState, activation: i32, check_charge: bool) -> bool {
if !crate::decision_tape::active() { return crate::borg_item_use::borg_equips_item(st,activation,check_charge); }
crate::decision_tape::query(st,"borg_item_use::borg_equips_item",format!("{:?}",(activation,check_charge,)),|| crate::borg_item_use::borg_equips_item(st,activation,check_charge))
}
pub fn borg_equips_ring(st: &mut BorgState, ring_sval: i32) -> bool {
if !crate::decision_tape::active() { return crate::borg_item_use::borg_equips_ring(st,ring_sval); }
crate::decision_tape::effect(st,"borg_item_use::borg_equips_ring",format!("{:?}",(ring_sval,)),|st| crate::borg_item_use::borg_equips_ring(st,ring_sval))
}
pub fn borg_equips_rod(st: &mut BorgState, sval: i32) -> bool {
if !crate::decision_tape::active() { return crate::borg_item_use::borg_equips_rod(st,sval); }
crate::decision_tape::effect(st,"borg_item_use::borg_equips_rod",format!("{:?}",(sval,)),|st| crate::borg_item_use::borg_equips_rod(st,sval))
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

use crate::borg_headers::borg_item_h::borg_item;
use crate::engine::TermGeometry;
use std::ffi::CStr;
pub fn borg_light_beam(st: &mut BorgState, simulation: bool) -> bool {
if !crate::decision_tape::active() { return crate::borg_light::borg_light_beam(st,simulation); }
crate::decision_tape::effect(st,"borg_light::borg_light_beam",format!("{:?}",(simulation,)),|st| crate::borg_light::borg_light_beam(st,simulation))
}
}
pub mod borg_magic {
pub fn borg_spell_stat() -> i32 {
crate::decision_tape::engine("borg_magic::borg_spell_stat",format!("{:?}",()),crate::borg_magic::borg_spell_stat)
}
pub fn borg_get_spell_entry(st:&crate::state::BorgState,book:i32,entry:i32)->Option<usize> {
if !crate::decision_tape::active() { return crate::borg_magic::borg_get_spell_entry(st,book,entry); }
crate::decision_tape::query(st,"borg_magic::borg_get_spell_entry",format!("{:?}",(book,entry)),|| crate::borg_magic::borg_get_spell_entry(st,book,entry))
}
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

use crate::borg_headers::borg_item_h::borg_item;
use std::ffi::CStr;
pub fn borg_get_spell_power(st: &BorgState, spell: borg_spells) -> i32 {
if !crate::decision_tape::active() { return crate::borg_magic::borg_get_spell_power(st,spell); }
crate::decision_tape::query(st,"borg_magic::borg_get_spell_power",format!("{:?}",(spell,)),|| crate::borg_magic::borg_get_spell_power(st,spell))
}
pub fn borg_spell_legal_fail(st: &BorgState, spell: borg_spells, allow_fail: i32) -> bool {
if !crate::decision_tape::active() { return crate::borg_magic::borg_spell_legal_fail(st,spell,allow_fail); }
crate::decision_tape::query(st,"borg_magic::borg_spell_legal_fail",format!("{:?}",(spell,allow_fail,)),|| crate::borg_magic::borg_spell_legal_fail(st,spell,allow_fail))
}

use crate::engine::TermGeometry;
pub fn borg_heroism_level(st: &BorgState) -> i32 {
if !crate::decision_tape::active() { return crate::borg_magic::borg_heroism_level(st); }
crate::decision_tape::query(st,"borg_magic::borg_heroism_level",format!("{:?}",()),|| crate::borg_magic::borg_heroism_level(st))
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

use crate::borg_headers::borg_item_h::borg_item;
use std::ffi::CStr;
pub fn borg_offset_projectable(st: &BorgState, y1: i32, x1: i32, y2: i32, x2: i32) -> bool {
if !crate::decision_tape::active() { return crate::borg_projection::borg_offset_projectable(st,y1,x1,y2,x2); }
crate::decision_tape::query(st,"borg_projection::borg_offset_projectable",format!("{:?}",(y1,x1,y2,x2,)),|| crate::borg_projection::borg_offset_projectable(st,y1,x1,y2,x2))
}
pub fn borg_projectable_pure(st: &BorgState, y1: i32, x1: i32, y2: i32, x2: i32) -> bool {
if !crate::decision_tape::active() { return crate::borg_projection::borg_projectable_pure(st,y1,x1,y2,x2); }
crate::decision_tape::query(st,"borg_projection::borg_projectable_pure",format!("{:?}",(y1,x1,y2,x2,)),|| crate::borg_projection::borg_projectable_pure(st,y1,x1,y2,x2))
}

// #364: replay caller-owned output parameters as well as the return value.
pub fn borg_inc_motion(py:&mut i32,px:&mut i32,y1:i32,x1:i32,y2:i32,x2:i32) {
    let before=(*py,*px);
    let after=crate::decision_tape::engine("borg_projection::borg_inc_motion",
        format!("{:?}",(before,y1,x1,y2,x2)), || {
            let (mut y,mut x)=before;
            crate::borg_projection::borg_inc_motion(&mut y,&mut x,y1,x1,y2,x2);
            (y,x)
        });
    *py=after.0; *px=after.1;
}

use crate::engine::TermGeometry;
pub fn borg_projectable(st: &BorgState, y1: i32, x1: i32, y2: i32, x2: i32) -> bool {
if !crate::decision_tape::active() { return crate::borg_projection::borg_projectable(st,y1,x1,y2,x2); }
crate::decision_tape::query(st,"borg_projection::borg_projectable",format!("{:?}",(y1,x1,y2,x2,)),|| crate::borg_projection::borg_projectable(st,y1,x1,y2,x2))
}
pub fn borg_projectable_dark(st: &BorgState, y1: i32, x1: i32, y2: i32, x2: i32) -> bool {
if !crate::decision_tape::active() { return crate::borg_projection::borg_projectable_dark(st,y1,x1,y2,x2); }
crate::decision_tape::query(st,"borg_projection::borg_projectable_dark",format!("{:?}",(y1,x1,y2,x2,)),|| crate::borg_projection::borg_projectable_dark(st,y1,x1,y2,x2))
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
pub fn adj_str_hold(index:usize) -> i32 {
if !crate::decision_tape::active() { return crate::engine::adj_str_hold(index); }
crate::decision_tape::engine("engine::adj_str_hold",format!("{:?}",(index,)),|| crate::engine::adj_str_hold(index))
}
pub fn class_book_dungeon(book:i32) -> bool {
if !crate::decision_tape::active() { return crate::engine::class_book_dungeon(book); }
crate::decision_tape::engine("engine::class_book_dungeon",format!("{:?}",(book,)),|| crate::engine::class_book_dungeon(book))
}
pub fn class_spell_weight() -> i32 {
if !crate::decision_tape::active() { return crate::engine::class_spell_weight(); }
crate::decision_tape::engine("engine::class_spell_weight",format!("{:?}",()),|| crate::engine::class_spell_weight())
}
pub fn class_total_spells() -> i32 {
if !crate::decision_tape::active() { return crate::engine::class_total_spells(); }
crate::decision_tape::engine("engine::class_total_spells",format!("{:?}",()),|| crate::engine::class_total_spells())
}
pub fn player_has(flag:u32) -> bool {
if !crate::decision_tape::active() { return crate::engine::player_has(flag); }
crate::decision_tape::engine("engine::player_has",format!("{:?}",(flag,)),|| crate::engine::player_has(flag))
}
pub fn spell_chance(index:i32) -> i16 {
if !crate::decision_tape::active() { return crate::engine::spell_chance(index); }
crate::decision_tape::engine("engine::spell_chance",format!("{:?}",(index,)),|| crate::engine::spell_chance(index))
}
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

use crate::borg_headers::borg_item_h::borg_item;
use std::ffi::CStr;
pub fn adj_str_blow(i: usize) -> i32 {
if !crate::decision_tape::active() { return crate::engine::adj_str_blow(i); }
crate::decision_tape::engine("engine::adj_str_blow",format!("{:?}",(i,)),|| crate::engine::adj_str_blow(i))
}
pub fn all_letters_nohjkl(i: i32) -> u8 {
if !crate::decision_tape::active() { return crate::engine::all_letters_nohjkl(i); }
crate::decision_tape::engine("engine::all_letters_nohjkl",format!("{:?}",(i,)),|| crate::engine::all_letters_nohjkl(i))
}
pub fn brand_resist_vuln_multiplier(i: usize) -> (i32, i32, i32) {
if !crate::decision_tape::active() { return crate::engine::brand_resist_vuln_multiplier(i); }
crate::decision_tape::engine("engine::brand_resist_vuln_multiplier",format!("{:?}",(i,)),|| crate::engine::brand_resist_vuln_multiplier(i))
}
pub fn flag_has(flags: &[u8], flag: i32) -> bool {
if !crate::decision_tape::active() { return crate::engine::flag_has(flags,flag); }
crate::decision_tape::engine("engine::flag_has",format!("{:?}",(flags,flag,)),|| crate::engine::flag_has(flags,flag))
}
pub fn get_angle_to_grid(y: usize, x: usize) -> u8 {
if !crate::decision_tape::active() { return crate::engine::get_angle_to_grid(y,x); }
crate::decision_tape::engine("engine::get_angle_to_grid",format!("{:?}",(y,x,)),|| crate::engine::get_angle_to_grid(y,x))
}
pub fn obj_kind_can_browse(kind: usize) -> bool {
if !crate::decision_tape::active() { return crate::engine::obj_kind_can_browse(kind); }
crate::decision_tape::engine("engine::obj_kind_can_browse",format!("{:?}",(kind,)),|| crate::engine::obj_kind_can_browse(kind))
}
pub fn player_has_temporary_brand(idx: i32) -> bool {
if !crate::decision_tape::active() { return crate::engine::player_has_temporary_brand(idx); }
crate::decision_tape::engine("engine::player_has_temporary_brand",format!("{:?}",(idx,)),|| crate::engine::player_has_temporary_brand(idx))
}
pub fn player_has_temporary_slay(idx: i32) -> bool {
if !crate::decision_tape::active() { return crate::engine::player_has_temporary_slay(idx); }
crate::decision_tape::engine("engine::player_has_temporary_slay",format!("{:?}",(idx,)),|| crate::engine::player_has_temporary_slay(idx))
}
pub fn race_blow_effect_name(r_idx: usize, k: usize) -> Option<&'static [u8]> {
if !crate::decision_tape::active() { return crate::engine::race_blow_effect_name(r_idx,k); }
crate::decision_tape::engine("engine::race_blow_effect_name",format!("{:?}",(r_idx,k,)),|| crate::engine::race_blow_effect_name(r_idx,k))
}
pub fn race_name(r_idx: usize) -> &'static [u8] {
if !crate::decision_tape::active() { return crate::engine::race_name(r_idx); }
crate::decision_tape::engine("engine::race_name",format!("{:?}",(r_idx,)),|| crate::engine::race_name(r_idx))
}
pub fn slay_race_flag_multiplier(i: usize) -> (i32, i32) {
if !crate::decision_tape::active() { return crate::engine::slay_race_flag_multiplier(i); }
crate::decision_tape::engine("engine::slay_race_flag_multiplier",format!("{:?}",(i,)),|| crate::engine::slay_race_flag_multiplier(i))
}

// #364: pointer-free metadata, copied at the original query point.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AttackRace {
    pub ac: i32, pub d_char: i32, pub speed: i32, pub light: i32,
    pub flags: [u8; 11], pub spell_flags: [u8; 12], pub has_friends: bool,
}
pub fn race(index: usize) -> AttackRace {
    crate::decision_tape::engine("engine::race", format!("{:?}", (index,)), || {
        let r = crate::engine::race(index);
        AttackRace { ac:r.ac, d_char:r.d_char, speed:r.speed, light:r.light,
            flags:r.flags, spell_flags:r.spell_flags,
            has_friends:!r.friends.is_null() || !r.friends_base.is_null() }
    })
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AttackKind { pub tval:i32, pub sval:i32, pub dd:i32, pub ds:i32, pub aware:bool }
pub fn kind(index: usize) -> AttackKind {
    crate::decision_tape::engine("engine::kind", format!("{:?}", (index,)), || {
        let k=crate::engine::kind(index);
        AttackKind {tval:k.tval,sval:k.sval,dd:k.dd,ds:k.ds,aware:k.aware}
    })
}

use crate::engine::TermGeometry;
pub fn term_geometry() -> TermGeometry {
if !crate::decision_tape::active() { return crate::engine::term_geometry(); }
crate::decision_tape::engine("engine::term_geometry",format!("{:?}",()),|| crate::engine::term_geometry())
}

pub fn no_light() -> bool {
if !crate::decision_tape::active() { return crate::engine::no_light(); }
crate::decision_tape::engine("engine::no_light",format!("{:?}",()),|| crate::engine::no_light())
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
pub mod list_player_flags_h { pub use crate::headers::list_player_flags_h::PF_ZERO_FAIL; }
pub mod player_h { pub use crate::headers::player_h::STAT_MAX; }
pub mod list_mon_race_flags_h { pub use crate::headers::list_mon_race_flags_h::{RF_KILL_WALL,RF_PASS_WALL}; pub use crate::headers::list_mon_race_flags_h::{RF_ANIMAL,RF_DEMON,RF_DRAGON,RF_EVIL,RF_GIANT,RF_HURT_COLD,RF_HURT_FIRE,RF_HURT_LIGHT,RF_HURT_ROCK,RF_IM_ACID,RF_IM_COLD,RF_IM_ELEC,RF_IM_FIRE,RF_IM_POIS,RF_MULTIPLY,RF_NEVER_MOVE,RF_NONLIVING,RF_NO_CONF,RF_NO_FEAR,RF_NO_SLEEP,RF_ORC,RF_QUESTOR,RF_SPIRIT,RF_TROLL,RF_UNDEAD,RF_UNIQUE};
}
pub mod list_options_h { pub use crate::headers::list_options_h::{OPT_birth_force_descend}; pub use crate::headers::list_options_h::{OPT_birth_randarts};
}
pub mod list_player_timed_h { pub use crate::headers::list_player_timed_h::{TMD_OPP_ACID}; }
pub mod list_terrain_h { pub use crate::headers::list_terrain_h::{FEAT_LAVA,FEAT_LESS,FEAT_MORE,FEAT_NONE}; pub use crate::headers::list_terrain_h::{FEAT_BROKEN,FEAT_FLOOR,FEAT_MAGMA,FEAT_OPEN,FEAT_PASS_RUBBLE,FEAT_PERM};
pub use crate::headers::list_terrain_h::{FEAT_CLOSED,FEAT_GRANITE,FEAT_RUBBLE};
}
pub mod list_tvals_h {pub use crate::headers::list_tvals_h::{TV_ARROW,TV_DIGGING}; pub use crate::headers::list_tvals_h::{TV_FOOD,TV_MUSHROOM,TV_POTION,TV_ROD,TV_STAFF}; pub use crate::headers::list_tvals_h::{TV_BOOTS,TV_FLASK,TV_LIGHT,TV_RING,TV_WAND};
pub use crate::headers::list_tvals_h::{TV_SCROLL};
pub use crate::headers::list_tvals_h::{TV_DRAG_ARMOR};
}
pub mod ui_event_h { pub use crate::headers::ui_event_h::{keycode_t}; }
pub mod z_type_h { pub use crate::headers::z_type_h::{loc}; }

pub mod object_h { pub use crate::headers::object_h::{activation};
}

pub mod trap_h { pub use crate::headers::trap_h::{trap};
}

pub mod list_mon_spells_h { pub use crate::headers::list_mon_spells_h::{RSF_BA_WATE,RSF_BR_CHAO,RSF_BR_DARK,RSF_BR_DISE,RSF_BR_GRAV,RSF_BR_INER,RSF_BR_LIGHT,RSF_BR_NETH,RSF_BR_NEXU,RSF_BR_PLAS,RSF_BR_SHAR,RSF_BR_SOUN,RSF_BR_TIME,RSF_BR_WALL,RSF_S_AINU,RSF_S_ANIMAL,RSF_S_DEMON,RSF_S_DRAGON,RSF_S_HI_DEMON,RSF_S_HI_DRAGON,RSF_S_HI_UNDEAD,RSF_S_HOUND,RSF_S_HYDRA,RSF_S_KIN,RSF_S_MONSTER,RSF_S_MONSTERS,RSF_S_SPIDER,RSF_S_UNDEAD,RSF_S_UNIQUE,RSF_S_WRAITH};
}

pub mod list_stats_h { pub use crate::headers::list_stats_h::{STAT_CON,STAT_DEX,STAT_INT,STAT_STR,STAT_WIS};
}

pub mod list_object_flags_h { pub use crate::headers::list_object_flags_h::{OF_NO_FUEL};
}

pub mod ui_output_h { pub use crate::headers::ui_output_h::{region};
}

pub mod ui_term_h { pub use crate::headers::ui_term_h::{term};
}
pub mod list_effects_h { pub use crate::headers::list_effects_h::{EF_NOURISH};
}
}
pub mod borg_headers {
pub mod borg_item_h { pub use crate::borg_headers::borg_item_h::borg_item; }
pub mod borg_h {pub use crate::borg_headers::borg_h::{BORG_NO_DEEPER,BORG_USES_DYNAMIC_CALCS,BORG_WORSHIPS_AC,BORG_WORSHIPS_DAMAGE,BORG_WORSHIPS_HP,BORG_WORSHIPS_MANA,BORG_WORSHIPS_SPEED}; pub use crate::borg_headers::borg_h::{BORG_PLAYS_RISKY}; pub use crate::borg_headers::borg_h::{BORG_TOWN_PLAN};
pub use crate::borg_headers::borg_h::{BORG_MONEY_SCUM_AMOUNT};
}
pub mod borg_magic_h { pub use crate::borg_headers::borg_magic_h::{BERSERK_STRENGTH,CURE_POISON,DIMENSION_DOOR,HEALING,HERBAL_CURING,HEROISM,HOLY_WORD,MINOR_HEALING,PHASE_DOOR,PORTAL,RECHARGING,REMEMBRANCE,REMOVE_HUNGER,RESTORATION,REVITALIZE,SHADOW_SHIFT,TELEPORT_LEVEL,TELEPORT_SELF,UNHOLY_REPRIEVE,WORD_OF_RECALL}; pub use crate::borg_headers::borg_magic_h::{ACID_SPRAY,ANNIHILATE,BANISH_SPIRITS,CONFUSE_MONSTER,CRUSH,CURSE,DARK_SPEAR,DISENCHANT,DISPEL_EVIL,DISPEL_LIFE,DISPEL_UNDEAD,EARTH_RISING,ELECTRIC_ARC,EXPLOSION,FIRE_BALL,FRIGHTEN,FROST_BOLT,HOWL_OF_THE_DAMNED,ICE_STORM,LEAP_INTO_BATTLE,LIGHTNING_STRIKE,LIGHT_OF_VARDA,MAGIC_MISSILE,MAIM_FOE,MANA_BOLT,MANA_STORM,MASS_SLEEP,METEOR_SWARM,NETHER_BOLT,ORB_OF_DRAINING,RIFT,RIVER_OF_LIGHTNING,SHOCK_WAVE,SLEEP_EVIL,SLOW_MONSTER,SPEAR_OF_LIGHT,SPEAR_OF_OROME,STINKING_CLOUD,STORM_OF_DARKNESS,TAP_UNLIFE,THRUST_AWAY,TRANCE,TURN_STONE_TO_MUD,UNLEASH_CHAOS,VAMPIRE_STRIKE,VOLCANIC_ERUPTION,WHIRLWIND_ATTACK,borg_magic,borg_spells};
pub use crate::borg_headers::borg_magic_h::{BANISHMENT,BANISH_EVIL,BLESS,CLAIRVOYANCE,DETECTION,DOOR_CREATION,FUME_OF_MORDOR,GLYPH_OF_WARDING,GRIM_PURPOSE,GRONDS_BLOW,HASTE_SELF,MASS_BANISHMENT,PROTECTION_FROM_EVIL,QUAKE,RAPID_REGENERATION,RESISTANCE,RESIST_POISON,REVEAL_MONSTERS,SENSE_INVISIBLE,SMITE_EVIL,TELEPORT_OTHER,TREMOR,WORD_OF_DESTRUCTION};
}
pub mod borg_trait_h { pub use crate::borg_headers::borg_trait_h::{BI_ACCW,BI_AESCAPE,BI_AEZHEAL,BI_AFUEL,BI_AHEAL,BI_AMISSILES,BI_APHASE,BI_ATELEPORT,BI_CDEPTH,BI_CLASS,BI_CLEVEL,BI_CRSFEAR,BI_CRSNOTEL,BI_CURHP,BI_CURSP,BI_ESP,BI_FOOD,BI_GOLD,BI_IFIRE,BI_ISAFRAID,BI_ISBLIND,BI_ISCONFUSED,BI_ISCUT,BI_ISFIXALL,BI_ISFIXCON,BI_ISFIXDEX,BI_ISFIXEXP,BI_ISFIXINT,BI_ISFIXSTR,BI_ISFIXWIS,BI_ISHEAVYSTUN,BI_ISHUNGRY,BI_ISIMAGE,BI_ISPOISONED,BI_ISSTUN,BI_ISWEAK,BI_KING,BI_LIGHT,BI_MAXCLEVEL,BI_MAXDEPTH,BI_MAXHP,BI_MAXSP,BI_SPEED}; pub use crate::borg_headers::borg_trait_h::{BI_FRACT,BI_RCONF};
}

pub mod borg_fight_attack_h { pub use crate::borg_headers::borg_fight_attack_h::{BF_ACT_ACID_BALL,BF_ACT_ACID_BOLT,BF_ACT_ACID_BOLT2,BF_ACT_ACID_BOLT3,BF_ACT_ARROW,BF_ACT_COLD_BALL100,BF_ACT_COLD_BALL160,BF_ACT_COLD_BALL2,BF_ACT_COLD_BALL50,BF_ACT_COLD_BOLT,BF_ACT_COLD_BOLT2,BF_ACT_CONFUSE2,BF_ACT_DISPEL_ALL,BF_ACT_DISPEL_EVIL,BF_ACT_DISPEL_EVIL60,BF_ACT_DISPEL_UNDEAD,BF_ACT_DRAGON_BALANCE,BF_ACT_DRAGON_BLUE,BF_ACT_DRAGON_CHAOS,BF_ACT_DRAGON_GOLD,BF_ACT_DRAGON_GREEN,BF_ACT_DRAGON_LAW,BF_ACT_DRAGON_MULTIHUED,BF_ACT_DRAGON_POWER,BF_ACT_DRAGON_RED,BF_ACT_DRAGON_SHINING,BF_ACT_DRAIN_LIFE1,BF_ACT_DRAIN_LIFE2,BF_ACT_DRAIN_LIFE3,BF_ACT_DRAIN_LIFE4,BF_ACT_ELEC_BALL,BF_ACT_ELEC_BALL2,BF_ACT_ELEC_BOLT,BF_ACT_FEAR_MONSTER,BF_ACT_FIRE_BALL,BF_ACT_FIRE_BALL2,BF_ACT_FIRE_BALL200,BF_ACT_FIRE_BALL72,BF_ACT_FIRE_BOLT,BF_ACT_FIRE_BOLT2,BF_ACT_FIRE_BOLT3,BF_ACT_LIGHT_BEAM,BF_ACT_LOSCONF,BF_ACT_LOSSLEEP,BF_ACT_LOSSLOW,BF_ACT_MANA_BOLT,BF_ACT_MISSILE,BF_ACT_MON_CONFUSE,BF_ACT_MON_SLOW,BF_ACT_RING_ACID,BF_ACT_RING_FIRE,BF_ACT_RING_ICE,BF_ACT_RING_LIGHTNING,BF_ACT_SLEEPII,BF_ACT_SLEEP_ALL,BF_ACT_STAFF_HOLY,BF_ACT_STARLIGHT,BF_ACT_STARLIGHT2,BF_ACT_STAR_BALL,BF_ACT_STINKING_CLOUD,BF_ACT_WONDER,BF_DRAGON_BALANCE,BF_DRAGON_BLACK,BF_DRAGON_BLUE,BF_DRAGON_CHAOS,BF_DRAGON_GOLD,BF_DRAGON_GREEN,BF_DRAGON_LAW,BF_DRAGON_MULTIHUED,BF_DRAGON_POWER,BF_DRAGON_RED,BF_DRAGON_SHINING,BF_DRAGON_WHITE,BF_LAUNCH,BF_MAX,BF_OBJECT,BF_PRAYER_DISPEL_LIFE,BF_PRAYER_DISP_EVIL,BF_PRAYER_DISP_SPIRITS,BF_PRAYER_DISP_UNDEAD,BF_PRAYER_HOLY_ORB_BALL,BF_PRAYER_HOLY_WORD,BF_REST,BF_RING_ACID,BF_RING_FIRE,BF_RING_ICE,BF_RING_LIGHTNING,BF_ROD_ACID_BALL,BF_ROD_ACID_BOLT,BF_ROD_COLD_BALL,BF_ROD_COLD_BOLT,BF_ROD_DRAIN_LIFE,BF_ROD_ELEC_BALL,BF_ROD_ELEC_BOLT,BF_ROD_FIRE_BALL,BF_ROD_FIRE_BOLT,BF_ROD_LIGHT_BEAM,BF_ROD_SLEEP_MONSTER,BF_ROD_SLOW_MONSTER,BF_ROD_UNKNOWN,BF_SPELL_ACID_SPRAY,BF_SPELL_ANNIHILATE,BF_SPELL_BLIND_CREATURE,BF_SPELL_COLD_BOLT,BF_SPELL_COLD_STORM,BF_SPELL_CONFUSE_MONSTER,BF_SPELL_CRUSH,BF_SPELL_CURSE,BF_SPELL_DARK_SPEAR,BF_SPELL_DISENCHANT,BF_SPELL_EARTH_RISING,BF_SPELL_ELECTRIC_ARC,BF_SPELL_EXPLOSION,BF_SPELL_FIRE_BALL,BF_SPELL_FRIGHTEN,BF_SPELL_HOWL_OF_THE_DAMNED,BF_SPELL_LEAP_INTO_BATTLE,BF_SPELL_LIGHTNING_STRIKE,BF_SPELL_LIGHT_BEAM,BF_SPELL_LIGHT_OF_VARDA,BF_SPELL_MAGIC_MISSILE,BF_SPELL_MAGIC_MISSILE_RESERVE,BF_SPELL_MAIM_FOE,BF_SPELL_MANA_BOLT,BF_SPELL_MANA_STORM,BF_SPELL_METEOR_SWARM,BF_SPELL_NETHER_BOLT,BF_SPELL_RIFT,BF_SPELL_RIVER_OF_LIGHTNING,BF_SPELL_SHOCK_WAVE,BF_SPELL_SLEEP_EVIL,BF_SPELL_SLEEP_III,BF_SPELL_SLOW_MONSTER,BF_SPELL_SPEAR_OF_OROME,BF_SPELL_STINK_CLOUD,BF_SPELL_STONE_TO_MUD,BF_SPELL_STORM_OF_DARKNESS,BF_SPELL_TAP_UNLIFE,BF_SPELL_THRUST_AWAY,BF_SPELL_TRANCE,BF_SPELL_UNLEASH_CHAOS,BF_SPELL_VAMPIRE_STRIKE,BF_SPELL_VOLCANIC_ERUPTION,BF_SPELL_WHIRLWIND_ATTACK,BF_STAFF_DISPEL_EVIL,BF_STAFF_HOLINESS,BF_STAFF_POWER,BF_STAFF_SLEEP_MONSTERS,BF_STAFF_SLOW_MONSTERS,BF_THRUST,BF_WAND_ACID_BALL,BF_WAND_ACID_BOLT,BF_WAND_ANNIHILATION,BF_WAND_COLD_BALL,BF_WAND_COLD_BOLT,BF_WAND_CONFUSE_MONSTER,BF_WAND_DRAGON_COLD,BF_WAND_DRAGON_FIRE,BF_WAND_DRAIN_LIFE,BF_WAND_ELEC_BALL,BF_WAND_ELEC_BOLT,BF_WAND_FEAR_MONSTER,BF_WAND_FIRE_BALL,BF_WAND_FIRE_BOLT,BF_WAND_HOLD_MONSTER,BF_WAND_LIGHT_BEAM,BF_WAND_MAGIC_MISSILE,BF_WAND_SLOW_MONSTER,BF_WAND_STINKING_CLOUD,BF_WAND_UNKNOWN,BF_WAND_WONDER,BORG_ATTACK_ACID,BORG_ATTACK_ARROW,BORG_ATTACK_AWAY_ALL,BORG_ATTACK_AWAY_ALL_MORGOTH,BORG_ATTACK_AWAY_EVIL,BORG_ATTACK_AWAY_UNDEAD,BORG_ATTACK_CHAOS,BORG_ATTACK_COLD,BORG_ATTACK_CONFU,BORG_ATTACK_CURSE,BORG_ATTACK_DARK,BORG_ATTACK_DARK_WEAK,BORG_ATTACK_DISEN,BORG_ATTACK_DISP_ALL,BORG_ATTACK_DISP_EVIL,BORG_ATTACK_DISP_SPIRITS,BORG_ATTACK_DISP_UNDEAD,BORG_ATTACK_DRAIN_LIFE,BORG_ATTACK_ELEC,BORG_ATTACK_ELEC_STRIKE,BORG_ATTACK_FIRE,BORG_ATTACK_FORCE,BORG_ATTACK_GRAVITY,BORG_ATTACK_HOLY_ORB,BORG_ATTACK_HOLY_WORD,BORG_ATTACK_ICE,BORG_ATTACK_INERTIA,BORG_ATTACK_KILL_DOOR,BORG_ATTACK_KILL_TRAP,BORG_ATTACK_KILL_WALL,BORG_ATTACK_LIGHT,BORG_ATTACK_LIGHT_WEAK,BORG_ATTACK_MAKE_DOOR,BORG_ATTACK_MAKE_TRAP,BORG_ATTACK_MAKE_WALL,BORG_ATTACK_MANA,BORG_ATTACK_METEOR,BORG_ATTACK_MISSILE,BORG_ATTACK_NETHER,BORG_ATTACK_NEXUS,BORG_ATTACK_OLD_CLONE,BORG_ATTACK_OLD_CONF,BORG_ATTACK_OLD_DRAIN,BORG_ATTACK_OLD_HEAL,BORG_ATTACK_OLD_POLY,BORG_ATTACK_OLD_SLEEP,BORG_ATTACK_OLD_SLOW,BORG_ATTACK_OLD_SPEED,BORG_ATTACK_PLASMA,BORG_ATTACK_POIS,BORG_ATTACK_SHARD,BORG_ATTACK_SLEEP_EVIL,BORG_ATTACK_SOUND,BORG_ATTACK_TAP_UNLIFE,BORG_ATTACK_TIME,BORG_ATTACK_TURN_ALL,BORG_ATTACK_TURN_EVIL,BORG_ATTACK_TURN_UNDEAD,BORG_ATTACK_WATER};
}

pub mod borg_danger_h { pub use crate::borg_headers::borg_danger_h::{MONBLOW_ACID,MONBLOW_BLACK_BREATH,MONBLOW_BLIND,MONBLOW_COLD,MONBLOW_CONFUSE,MONBLOW_DISENCHANT,MONBLOW_DRAIN_CHARGES,MONBLOW_EAT_FOOD,MONBLOW_EAT_GOLD,MONBLOW_EAT_ITEM,MONBLOW_EAT_LIGHT,MONBLOW_ELEC,MONBLOW_EXP_10,MONBLOW_EXP_20,MONBLOW_EXP_40,MONBLOW_EXP_80,MONBLOW_FIRE,MONBLOW_HALLU,MONBLOW_HURT,MONBLOW_LOSE_ALL,MONBLOW_LOSE_CON,MONBLOW_LOSE_DEX,MONBLOW_LOSE_INT,MONBLOW_LOSE_STR,MONBLOW_LOSE_WIS,MONBLOW_NONE,MONBLOW_PARALYZE,MONBLOW_POISON,MONBLOW_SHATTER,MONBLOW_TERRIFY};
}
}
pub mod borg { pub use crate::borg::BorgTime; }
pub mod borg_cave { pub use crate::borg_cave::{AUTO_MAX_X,AUTO_MAX_Y}; pub use crate::borg_cave::{BorgGrid,DUNGEON_HGT,DUNGEON_WID};
}
pub mod borg_update { pub use crate::borg_update::Detect;
use crate::memory::BorgGrid; }

pub mod borg_item {
pub fn borg_item_weight(item:&crate::borg_headers::borg_item_h::borg_item)->i16 {
crate::decision_tape::engine("borg_item::borg_item_weight",format!("{:?}",item),|| crate::borg_item::borg_item_weight(item))
}
use crate::{state::BorgState, borg::BorgTime, headers, headers::z_type_h::loc};
use crate::borg_headers::borg_item_h::borg_item;
use crate::borg_update::Detect;
use crate::memory::BorgGrid;
use crate::headers::ui_event_h::keycode_t;
use std::ffi::CStr;
pub fn inven_wield() -> i32 {
if !crate::decision_tape::active() { return crate::borg_item::inven_wield(); }
crate::decision_tape::engine("borg_item::inven_wield",format!("{:?}",()),|| crate::borg_item::inven_wield())
}
pub fn quiver_end() -> i32 {
if !crate::decision_tape::active() { return crate::borg_item::quiver_end(); }
crate::decision_tape::engine("borg_item::quiver_end",format!("{:?}",()),|| crate::borg_item::quiver_end())
}
pub fn quiver_start() -> i32 {
if !crate::decision_tape::active() { return crate::borg_item::quiver_start(); }
crate::decision_tape::engine("borg_item::quiver_start",format!("{:?}",()),|| crate::borg_item::quiver_start())
}
}

pub mod borg_item_id {
use crate::{state::BorgState, borg::BorgTime, headers, headers::z_type_h::loc};
use crate::borg_headers::borg_item_h::borg_item;
use crate::borg_update::Detect;
use crate::memory::BorgGrid;
use crate::headers::ui_event_h::keycode_t;
use std::ffi::CStr;
pub fn borg_item_note_needs_id(st: &BorgState, item: &borg_item) -> bool {
if !crate::decision_tape::active() { return crate::borg_item_id::borg_item_note_needs_id(st,item); }
crate::decision_tape::query(st,"borg_item_id::borg_item_note_needs_id",format!("{:?}",(item,)),|| crate::borg_item_id::borg_item_note_needs_id(st,item))
}
}

pub mod borg_town_utility {
use crate::state::BorgState;
// #364: shared town/trip service; forecast can mutate owned trip memory.
pub fn supply_journey_turns(st:&mut BorgState)->i64 {
    crate::decision_tape::effect(st,"borg_town_utility::supply_journey_turns",String::new(),|st| {
        let forecast=crate::borg_trip::forecast(st);
        crate::borg_town_utility::Journey::for_trip(st,forecast).turns
    })
}
}

#[cfg(test)]
mod attack_adapter_tests {
    use super::*;
    use crate::state::BorgState;
    #[test]
    fn motion_outputs_are_replayed_and_input_changes_are_rejected() {
        let context=||serde_json::json!({"point":"attack.motion-outputs"});
        let mut state=BorgState::default();
        let (expected,replayed,result,events,_)=crate::decision_tape::compare(&mut state,context(),|_| {
            let (mut y,mut x)=(0,0);
            borg_projection::borg_inc_motion(&mut y,&mut x,0,0,6,3);
            (y,x)
        },|_| {
            let (mut y,mut x)=(0,0);
            borg_projection::borg_inc_motion(&mut y,&mut x,0,0,6,3);
            (y,x)
        });
        assert!(result.is_ok());assert_eq!(replayed,Some(expected));
        assert_ne!(expected,(0,0));assert_eq!(events,1);
        let (_,_,result,_,_)=crate::decision_tape::compare(&mut state,context(),|_| {
            let (mut y,mut x)=(0,0);
            borg_projection::borg_inc_motion(&mut y,&mut x,0,0,6,3);
        },|_| {
            let (mut y,mut x)=(0,1);
            borg_projection::borg_inc_motion(&mut y,&mut x,0,0,6,3);
        });
        assert!(result.unwrap_err().contains("expected borg_projection::borg_inc_motion"));
    }
}

pub mod borg_escape {
use crate::{state::BorgState, borg::BorgTime, headers, headers::z_type_h::loc};
use crate::borg_headers::borg_item_h::borg_item;
use crate::engine::TermGeometry;
use crate::borg_update::Detect;
use crate::memory::BorgGrid;
use crate::headers::ui_event_h::keycode_t;
use std::ffi::CStr;
pub fn borg_caution_teleport(st:&mut BorgState,emergency:i32,turns:i32)->bool {
if !crate::decision_tape::active() { return crate::borg_escape::borg_caution_teleport(st,emergency,turns); }
crate::decision_tape::effect(st,"borg_escape::borg_caution_teleport",format!("{:?}",(emergency,turns,)),|st| crate::borg_escape::borg_caution_teleport(st,emergency,turns))
}
pub fn borg_surrounded(st:&mut BorgState)->bool {
if !crate::decision_tape::active() { return crate::borg_escape::borg_surrounded(st); }
crate::decision_tape::effect(st,"borg_escape::borg_surrounded",format!("{:?}",()),|st| crate::borg_escape::borg_surrounded(st))
}
}

pub mod borg_item_analyze {
pub fn borg_ego_has_random_power(index:usize)->bool {
crate::decision_tape::engine("borg_item_analyze::borg_ego_has_random_power",format!("{:?}",(index,)),|| crate::borg_item_analyze::borg_ego_has_random_power(index))
}
use crate::state::BorgState;
use crate::borg_headers::borg_item_h::borg_item;
pub fn borg_obj_has_effect(kind: u32, index: i32, subtype: i32) -> bool {
if !crate::decision_tape::active() { return crate::borg_item_analyze::borg_obj_has_effect(kind,index,subtype); }
crate::decision_tape::engine("borg_item_analyze::borg_obj_has_effect",format!("{:?}",(kind,index,subtype,)),|| crate::borg_item_analyze::borg_obj_has_effect(kind,index,subtype))
}
}

pub mod borg_object_log {
use crate::state::BorgState;
use crate::borg_headers::borg_item_h::borg_item;
pub fn log_use(st: &BorgState, action: &str, item: &borg_item) {
if !crate::decision_tape::active() { return crate::borg_object_log::log_use(st,action,item); }
crate::decision_tape::query(st,"borg_object_log::log_use",format!("{:?}",(action,item,)),|| crate::borg_object_log::log_use(st,action,item))
}
}

// #435: test-driver capability, absent from normal builds and forbidden to policy.
#[cfg(feature="test-r4-item-use")]
pub mod item_use_fixture {
    use std::cell::Cell;
    thread_local! { static STUB_MESSAGES:Cell<bool>=const {Cell::new(false)}; }
    pub(super) fn messages_stubbed()->bool { STUB_MESSAGES.with(Cell::get) }
    pub fn with_stubbed_messages<T>(run:impl FnOnce()->T)->T {
        struct Reset(bool);
        impl Drop for Reset { fn drop(&mut self) { STUB_MESSAGES.with(|v|v.set(self.0)); } }
        let _reset=Reset(STUB_MESSAGES.with(|v|v.replace(true)));
        run()
    }
}

/// R434: engine-derived activation-name lookup; policy chain is owned by each implementation.
pub mod borg_item_activation { pub use crate::borg_item_activation::ActivationIds; }
/// R434: configured formula policy explicitly remains shared, with ordered full-state replay.
pub mod borg_formulas {
pub fn borg_power_dynamic(st:&mut crate::state::BorgState)->i32 {
if !crate::decision_tape::active() { return crate::borg_formulas::borg_power_dynamic(st); }
crate::decision_tape::effect(st,"borg_formulas::borg_power_dynamic",format!("{:?}",()),crate::borg_formulas::borg_power_dynamic)
}
}
