//! borg-reborn-rs (#281): read-only inputs shared with native danger.
//! No danger policy, mutable state, RNG, keys or engine effects are exposed.
use crate::state::BorgState;
pub use crate::params::{value as param, Param};

pub fn resolved_param(param: Param) -> (i32, &'static str) { crate::params::resolved(param) }
pub use crate::engine::{adj_dex_safe, ddx_ddd, ddy_ddd, extract_energy,
    game_turn, race_flag_has, square_in_bounds, square_in_bounds_fully, z_info};
pub use crate::borg_cave_util::{borg_cave_floor_grid as floor_grid,
    borg_feature_protected as protected};
pub use crate::borg_projection::borg_distance as distance;
pub mod constants {
    pub use crate::borg_headers::borg_danger_h::*;
    pub use crate::borg_headers::borg_magic_h::{REMEMBRANCE, RESTORATION, REVITALIZE, UNHOLY_REPRIEVE};
    pub use crate::headers::list_mon_race_flags_h::*;
    pub use crate::headers::list_mon_spells_h::*;
    pub use crate::headers::list_stats_h::{STAT_INT, STAT_WIS};
    pub use crate::headers::list_terrain_h::*;
    pub use crate::borg_cave_view::BORG_VIEW;
}
#[derive(Clone, Copy)]
pub struct Race {
    pub level: i32, pub spell_power: i32, pub sleep: i32,
    pub has_friends: bool, pub freq_innate: i32, pub freq_spell: i32,
}
pub fn race(index: usize) -> Race {
    let r = crate::engine::race(index);
    Race { level: r.level, spell_power: r.spell_power, sleep: r.sleep,
        has_friends: !r.friends.is_null() || !r.friends_base.is_null(),
        freq_innate: r.freq_innate, freq_spell: r.freq_spell }
}
#[derive(Clone, Copy)]
pub struct Blow { pub dice: i32, pub sides: i32, pub effect: u32 }
pub fn blow(index: usize, slot: usize) -> Option<Blow> {
    crate::engine::race_blow(index, slot).map(|b| Blow { dice: b.dice, sides: b.sides,
        effect: crate::borg_fight_attack::borg_mon_blow_effect(b.effect_name.to_bytes()) as u32 })
}
pub fn spell_stat() -> i32 { crate::borg_magic::borg_spell_stat() }
pub fn player_is_fox() -> bool {
    crate::engine::player_is_shapechanged() && crate::engine::player_shape_name() == b"fox"
}
/// Borrowed context, valid only until the next mutation of the native state.
pub struct Facts<'a>(&'a BorgState);
impl<'a> Facts<'a> {
    pub fn new(state: &'a BorgState) -> Self { Self(state) }
    // #856: shared read-only caution policy until the breeder hunt is carried.
    pub fn breeder_engaged(&self) -> bool { crate::borg_caution::breeder_engaged(self.0) }
    pub fn is_vault(&self, x: i32, y: i32) -> bool { crate::engine::square_isvault(x,y) }
    pub fn floor(&self, y:i32, x:i32) -> bool { crate::borg_cave_util::borg_cave_floor_bold(self.0,y,x) }
    pub fn projectable(&self, y:i32,x:i32, yy:i32,xx:i32) -> bool { crate::borg_projection::borg_projectable(self.0,y,x,yy,xx) }
    pub fn projectable_pure(&self, y:i32,x:i32, yy:i32,xx:i32) -> bool { crate::borg_projection::borg_projectable_pure(self.0,y,x,yy,xx) }
    pub fn spell_legal(&self, spell: crate::borg_headers::borg_magic_h::borg_spells) -> bool { crate::borg_magic::borg_spell_legal(self.0,spell) }
    pub fn light_needs_no_fuel(&self) -> bool {
        let light = &self.0.items.items[usize::from(crate::engine::z_info().pack_size)+5];
        light.timeout == 0 || crate::engine::flag_has(&light.flags, crate::headers::list_object_flags_h::OF_NO_FUEL as i32)
    }
}

#[cfg(feature = "test-m13-danger")]
pub use crate::borg_danger::fixture_estimate;
#[cfg(feature = "test-m13-danger")]
pub use crate::engine::race_name;

/// Constructed-state controls only; absent from production builds.
#[cfg(feature = "test-r3-defence")]
pub mod defence_fixture {
    pub use crate::borg_magic::BorgMagic;
    pub use crate::borg_headers::borg_magic_h::{GRIM_PURPOSE, HASTE_SELF,
        PROTECTION_FROM_EVIL, RESISTANCE, RESIST_POISON};
    pub use crate::engine::class_total_spells;
    pub use crate::borg_fight_defend::fixture_choice;
}

// #435: key-queue witness for constructed test states only; no engine effect.
#[cfg(feature="test-r4-item-use")]
pub mod item_use_fixture {
    pub fn blank_item()->crate::borg_headers::borg_item_h::borg_item { crate::borg_item::blank_item() }
    pub fn queued_keys(state:&crate::state::BorgState)->Vec<crate::headers::ui_event_h::keycode_t> {
        state.io.queued_keys()
    }
}

#[cfg(feature="test-r4-equipment")]
pub mod equipment_fixture {
    pub use crate::borg_item_wear::fixture::{trial,critical,choice};
    pub fn bind_note(item:&mut crate::borg_headers::borg_item_h::borg_item,origin:u16,offset:usize) {
        crate::borg_item::set_item_note(item,crate::borg_item::NoteOrigin::Item(origin),offset);
    }
    pub fn needs_id(st:&crate::state::BorgState,item:&crate::borg_headers::borg_item_h::borg_item)->bool {
        crate::borg_item_id::borg_item_note_needs_id(st,item)
    }
}

#[cfg(feature="test-r4-store")]
pub mod store_fixture {
    pub fn similar(st:&crate::state::BorgState,a:&crate::borg_headers::borg_item_h::borg_item,b:&crate::borg_headers::borg_item_h::borg_item)->bool {
        crate::borg_store_sell::borg_object_similar(st,a,b)
    }
}
