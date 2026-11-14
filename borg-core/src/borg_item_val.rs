//! Port of `borg-item-val.c`: load the sval and kval of the items.
//!
//! Copyright (c) 1997 Ben Harrison, James E. Wilson, Robert A. Koeneke
//! Copyright (c) 2007-9 Andi Sidwell, Chris Carr, Ed Graham, Erik Osheim
//!
//! This work is free software; you can redistribute it and/or modify it
//! under the terms of either:
//!
//! a) the GNU General Public License as published by the Free Software
//!    Foundation, version 2, or
//!
//! b) the "Angband License":
//!    This software may be copied and distributed for educational, research,
//!    and not for profit purposes provided that this copyright and statement
//!    are included in all such copies.  Other copyrights may also apply.

use crate::borg_io::borg_note;
use crate::state::BorgState;
use crate::engine;

/// The svals (`sv_*`) and kind indexes (`kv_*`) of the items the Borg
/// knows, looked up by name at startup. Field names are the C globals'.
#[derive(Clone, Copy, Default, Debug)]
pub struct ItemValState {
    pub sv_food_apple: i32,
    pub sv_food_ration: i32,
    pub sv_food_slime_mold: i32,
    pub sv_food_draught: i32,
    pub sv_food_pint: i32,
    pub sv_food_sip: i32,
    pub sv_food_waybread: i32,
    pub sv_food_honey_cake: i32,
    pub sv_food_slice: i32,
    pub sv_food_handful: i32,
    pub sv_mush_second_sight: i32,
    pub sv_mush_fast_recovery: i32,
    pub sv_mush_restoring: i32,
    pub sv_mush_mana: i32,
    pub sv_mush_emergency: i32,
    pub sv_mush_terror: i32,
    pub sv_mush_stoneskin: i32,
    pub kv_mush_stoneskin: i32,
    pub sv_mush_debility: i32,
    pub sv_mush_sprinting: i32,
    pub sv_mush_cure_mind: i32,
    pub sv_mush_purging: i32,
    pub sv_light_lantern: i32,
    pub sv_light_torch: i32,
    pub sv_flask_oil: i32,
    pub kv_flask_oil: i32,
    pub sv_potion_cure_critical: i32,
    pub sv_potion_cure_serious: i32,
    pub sv_potion_cure_light: i32,
    pub sv_potion_healing: i32,
    pub kv_potion_healing: i32,
    pub sv_potion_star_healing: i32,
    pub sv_potion_life: i32,
    pub sv_potion_restore_mana: i32,
    pub kv_potion_restore_mana: i32,
    pub sv_potion_cure_poison: i32,
    pub sv_potion_resist_heat: i32,
    pub sv_potion_resist_cold: i32,
    pub sv_potion_resist_pois: i32,
    pub sv_potion_inc_str: i32,
    pub sv_potion_inc_int: i32,
    pub sv_potion_inc_wis: i32,
    pub sv_potion_inc_dex: i32,
    pub sv_potion_inc_con: i32,
    pub sv_potion_inc_str2: i32,
    pub sv_potion_inc_int2: i32,
    pub sv_potion_inc_wis2: i32,
    pub sv_potion_inc_dex2: i32,
    pub sv_potion_inc_con2: i32,
    pub sv_potion_inc_all: i32,
    pub sv_potion_restore_life: i32,
    pub sv_potion_speed: i32,
    pub sv_potion_berserk: i32,
    pub sv_potion_sleep: i32,
    pub sv_potion_slowness: i32,
    pub sv_potion_poison: i32,
    pub sv_potion_blindness: i32,
    pub sv_potion_confusion: i32,
    pub sv_potion_heroism: i32,
    pub sv_potion_boldness: i32,
    pub sv_potion_detect_invis: i32,
    pub sv_potion_enlightenment: i32,
    pub sv_potion_slime_mold: i32,
    pub sv_potion_infravision: i32,
    pub sv_potion_inc_exp: i32,
    pub sv_scroll_identify: i32,
    pub sv_scroll_phase_door: i32,
    pub sv_scroll_teleport: i32,
    pub sv_scroll_word_of_recall: i32,
    pub sv_scroll_enchant_armor: i32,
    pub sv_scroll_enchant_weapon_to_hit: i32,
    pub sv_scroll_enchant_weapon_to_dam: i32,
    pub sv_scroll_star_enchant_weapon: i32,
    pub sv_scroll_star_enchant_armor: i32,
    pub sv_scroll_protection_from_evil: i32,
    pub sv_scroll_rune_of_protection: i32,
    pub sv_scroll_teleport_level: i32,
    pub sv_scroll_deep_descent: i32,
    pub sv_scroll_recharging: i32,
    pub sv_scroll_banishment: i32,
    pub sv_scroll_mass_banishment: i32,
    pub kv_scroll_mass_banishment: i32,
    pub sv_scroll_blessing: i32,
    pub sv_scroll_holy_chant: i32,
    pub sv_scroll_holy_prayer: i32,
    pub sv_scroll_detect_invis: i32,
    pub sv_scroll_satisfy_hunger: i32,
    pub sv_scroll_light: i32,
    pub sv_scroll_mapping: i32,
    pub sv_scroll_acquirement: i32,
    pub sv_scroll_star_acquirement: i32,
    pub sv_scroll_remove_curse: i32,
    pub kv_scroll_remove_curse: i32,
    pub sv_scroll_star_remove_curse: i32,
    pub kv_scroll_star_remove_curse: i32,
    pub sv_scroll_monster_confusion: i32,
    pub sv_scroll_trap_door_destruction: i32,
    pub sv_scroll_dispel_undead: i32,
    pub sv_ring_flames: i32,
    pub sv_ring_ice: i32,
    pub sv_ring_acid: i32,
    pub sv_ring_lightning: i32,
    pub sv_ring_digging: i32,
    pub sv_ring_speed: i32,
    pub sv_ring_damage: i32,
    pub sv_ring_dog: i32,
    pub sv_amulet_teleportation: i32,
    pub sv_rod_recall: i32,
    pub kv_rod_recall: i32,
    pub sv_rod_detection: i32,
    pub sv_rod_illumination: i32,
    pub sv_rod_speed: i32,
    pub sv_rod_mapping: i32,
    pub sv_rod_healing: i32,
    pub kv_rod_healing: i32,
    pub sv_rod_light: i32,
    pub sv_rod_fire_bolt: i32,
    pub sv_rod_elec_bolt: i32,
    pub sv_rod_cold_bolt: i32,
    pub sv_rod_acid_bolt: i32,
    pub sv_rod_drain_life: i32,
    pub sv_rod_fire_ball: i32,
    pub sv_rod_elec_ball: i32,
    pub sv_rod_cold_ball: i32,
    pub sv_rod_acid_ball: i32,
    pub sv_rod_teleport_other: i32,
    pub sv_rod_slow_monster: i32,
    pub sv_rod_sleep_monster: i32,
    pub sv_rod_curing: i32,
    pub sv_staff_teleportation: i32,
    pub sv_staff_destruction: i32,
    pub sv_staff_speed: i32,
    pub sv_staff_healing: i32,
    pub sv_staff_the_magi: i32,
    pub sv_staff_power: i32,
    pub sv_staff_curing: i32,
    pub sv_staff_holiness: i32,
    pub kv_staff_holiness: i32,
    pub sv_staff_sleep_monsters: i32,
    pub sv_staff_slow_monsters: i32,
    pub sv_staff_detect_invis: i32,
    pub sv_staff_detect_evil: i32,
    pub sv_staff_dispel_evil: i32,
    pub sv_staff_banishment: i32,
    pub sv_staff_light: i32,
    pub sv_staff_mapping: i32,
    pub sv_staff_remove_curse: i32,
    pub sv_wand_light: i32,
    pub sv_wand_teleport_away: i32,
    pub sv_wand_stinking_cloud: i32,
    pub kv_wand_stinking_cloud: i32,
    pub sv_wand_magic_missile: i32,
    pub kv_wand_magic_missile: i32,
    pub sv_wand_annihilation: i32,
    pub kv_wand_annihilation: i32,
    pub sv_wand_stone_to_mud: i32,
    pub sv_wand_wonder: i32,
    pub sv_wand_slow_monster: i32,
    pub sv_wand_hold_monster: i32,
    pub sv_wand_fear_monster: i32,
    pub sv_wand_confuse_monster: i32,
    pub sv_wand_fire_bolt: i32,
    pub sv_wand_cold_bolt: i32,
    pub sv_wand_acid_bolt: i32,
    pub sv_wand_elec_bolt: i32,
    pub sv_wand_fire_ball: i32,
    pub sv_wand_cold_ball: i32,
    pub sv_wand_acid_ball: i32,
    pub sv_wand_elec_ball: i32,
    pub sv_wand_dragon_cold: i32,
    pub sv_wand_dragon_fire: i32,
    pub sv_wand_drain_life: i32,
    pub sv_dagger: i32,
    pub sv_sling: i32,
    pub sv_short_bow: i32,
    pub sv_long_bow: i32,
    pub sv_light_xbow: i32,
    pub sv_heavy_xbow: i32,
    pub sv_arrow_seeker: i32,
    pub sv_arrow_mithril: i32,
    pub sv_bolt_seeker: i32,
    pub sv_bolt_mithril: i32,
    pub sv_set_of_leather_gloves: i32,
    pub sv_cloak: i32,
    pub sv_robe: i32,
    pub sv_iron_crown: i32,
    pub sv_dragon_blue: i32,
    pub sv_dragon_black: i32,
    pub sv_dragon_white: i32,
    pub sv_dragon_red: i32,
    pub sv_dragon_green: i32,
    pub sv_dragon_multihued: i32,
    pub sv_dragon_shining: i32,
    pub sv_dragon_law: i32,
    pub sv_dragon_gold: i32,
    pub sv_dragon_chaos: i32,
    pub sv_dragon_balance: i32,
    pub sv_dragon_power: i32,
}

/// `lookup_sval()` that notes a startup failure when the name is unknown
/// (`borg_lookup_sval_fail`). Returns the sval, or -1.
fn borg_lookup_sval_fail(st: &mut BorgState, tval: i32, name: &str) -> i32 {
    let sval = engine::lookup_sval(tval, name);
    if sval == -1 {
        borg_note(
            st,
            &format!("**STARTUP FAILURE** sval lookup failure - {} ", name),
        );
        st.init.init_failure = true;
    }
    sval
}

/// Look up every sval and kval (`borg_init_item_val`).
///
/// The C code's copy-and-paste slips are kept (see `docs/c-bugs.md`):
/// `sv_wand_confuse_monster` is "Scare Monster", `sv_wand_elec_ball` is
/// "Lightning Bolts", and `sv_potion_berserk` is looked up twice.
pub fn borg_init_item_val(st: &mut BorgState) {
    let mut tval = engine::tval_find_idx("food");
    st.item_val.sv_food_apple = borg_lookup_sval_fail(st, tval, "Apple");
    st.item_val.sv_food_ration = borg_lookup_sval_fail(st, tval, "Ration of Food");
    st.item_val.sv_food_slime_mold = borg_lookup_sval_fail(st, tval, "Slime Mold");
    st.item_val.sv_food_draught = borg_lookup_sval_fail(st, tval, "Draught of the Ents");
    st.item_val.sv_food_pint = borg_lookup_sval_fail(st, tval, "Pint of Fine Wine");
    st.item_val.sv_food_sip = borg_lookup_sval_fail(st, tval, "Sip of Miruvor");
    st.item_val.sv_food_waybread = borg_lookup_sval_fail(st, tval, "Piece of Elvish Waybread");
    st.item_val.sv_food_honey_cake = borg_lookup_sval_fail(st, tval, "Honey-cake");
    st.item_val.sv_food_slice = borg_lookup_sval_fail(st, tval, "Slice of Meat");
    st.item_val.sv_food_handful = borg_lookup_sval_fail(st, tval, "Handful of Dried Fruits");

    tval = engine::tval_find_idx("mushroom");
    st.item_val.sv_mush_second_sight = borg_lookup_sval_fail(st, tval, "Second Sight");
    st.item_val.sv_mush_fast_recovery = borg_lookup_sval_fail(st, tval, "Fast Recovery");
    st.item_val.sv_mush_restoring = borg_lookup_sval_fail(st, tval, "Vigor");
    st.item_val.sv_mush_mana = borg_lookup_sval_fail(st, tval, "Clear Mind");
    st.item_val.sv_mush_emergency = borg_lookup_sval_fail(st, tval, "Emergency");
    st.item_val.sv_mush_terror = borg_lookup_sval_fail(st, tval, "Terror");
    st.item_val.sv_mush_stoneskin = borg_lookup_sval_fail(st, tval, "Stoneskin");
    st.item_val.kv_mush_stoneskin = borg_lookup_kind(st, tval, st.item_val.sv_mush_stoneskin);
    st.item_val.sv_mush_debility = borg_lookup_sval_fail(st, tval, "Debility");
    st.item_val.sv_mush_sprinting = borg_lookup_sval_fail(st, tval, "Sprinting");
    st.item_val.sv_mush_cure_mind = borg_lookup_sval_fail(st, tval, "Clear Mind");
    st.item_val.sv_mush_purging = borg_lookup_sval_fail(st, tval, "Purging");

    tval = engine::tval_find_idx("light");
    st.item_val.sv_light_lantern = borg_lookup_sval_fail(st, tval, "Lantern");
    st.item_val.sv_light_torch = borg_lookup_sval_fail(st, tval, "Wooden Torch");

    tval = engine::tval_find_idx("flask");
    st.item_val.sv_flask_oil = borg_lookup_sval_fail(st, tval, "Flask of Oil");
    st.item_val.kv_flask_oil = borg_lookup_kind(st, tval, st.item_val.sv_flask_oil);

    tval = engine::tval_find_idx("potion");
    st.item_val.sv_potion_cure_critical = borg_lookup_sval_fail(st, tval, "Cure Critical Wounds");
    st.item_val.sv_potion_cure_serious = borg_lookup_sval_fail(st, tval, "Cure Serious Wounds");
    st.item_val.sv_potion_cure_light = borg_lookup_sval_fail(st, tval, "Cure Light Wounds");
    st.item_val.sv_potion_healing = borg_lookup_sval_fail(st, tval, "Healing");
    st.item_val.kv_potion_healing = borg_lookup_kind(st, tval, st.item_val.sv_potion_healing);
    st.item_val.sv_potion_star_healing = borg_lookup_sval_fail(st, tval, "*Healing*");
    st.item_val.sv_potion_life = borg_lookup_sval_fail(st, tval, "Life");
    st.item_val.sv_potion_restore_mana = borg_lookup_sval_fail(st, tval, "Restore Mana");
    st.item_val.kv_potion_restore_mana = borg_lookup_kind(st, tval, st.item_val.sv_potion_restore_mana);
    st.item_val.sv_potion_cure_poison = borg_lookup_sval_fail(st, tval, "Neutralize Poison");
    st.item_val.sv_potion_resist_heat = borg_lookup_sval_fail(st, tval, "Resist Heat");
    st.item_val.sv_potion_resist_cold = borg_lookup_sval_fail(st, tval, "Resist Cold");
    st.item_val.sv_potion_resist_pois = borg_lookup_sval_fail(st, tval, "Resist Poison");
    st.item_val.sv_potion_inc_str = borg_lookup_sval_fail(st, tval, "Strength");
    st.item_val.sv_potion_inc_int = borg_lookup_sval_fail(st, tval, "Intelligence");
    st.item_val.sv_potion_inc_wis = borg_lookup_sval_fail(st, tval, "Wisdom");
    st.item_val.sv_potion_inc_dex = borg_lookup_sval_fail(st, tval, "Dexterity");
    st.item_val.sv_potion_inc_con = borg_lookup_sval_fail(st, tval, "Constitution");
    st.item_val.sv_potion_inc_all = borg_lookup_sval_fail(st, tval, "Augmentation");
    st.item_val.sv_potion_inc_str2 = borg_lookup_sval_fail(st, tval, "Brawn");
    st.item_val.sv_potion_inc_int2 = borg_lookup_sval_fail(st, tval, "Intellect");
    st.item_val.sv_potion_inc_wis2 = borg_lookup_sval_fail(st, tval, "Contemplation");
    st.item_val.sv_potion_inc_dex2 = borg_lookup_sval_fail(st, tval, "Nimbleness");
    st.item_val.sv_potion_inc_con2 = borg_lookup_sval_fail(st, tval, "Toughness");
    st.item_val.sv_potion_restore_life = borg_lookup_sval_fail(st, tval, "Restore Life Levels");
    st.item_val.sv_potion_speed = borg_lookup_sval_fail(st, tval, "Speed");
    st.item_val.sv_potion_berserk = borg_lookup_sval_fail(st, tval, "Berserk Strength");
    st.item_val.sv_potion_sleep = borg_lookup_sval_fail(st, tval, "Sleep");
    st.item_val.sv_potion_slowness = borg_lookup_sval_fail(st, tval, "Slowness");
    st.item_val.sv_potion_poison = borg_lookup_sval_fail(st, tval, "Poison");
    st.item_val.sv_potion_blindness = borg_lookup_sval_fail(st, tval, "Blindness");
    st.item_val.sv_potion_confusion = borg_lookup_sval_fail(st, tval, "Confusion");
    st.item_val.sv_potion_heroism = borg_lookup_sval_fail(st, tval, "Heroism");
    st.item_val.sv_potion_boldness = borg_lookup_sval_fail(st, tval, "Boldness");
    st.item_val.sv_potion_detect_invis = borg_lookup_sval_fail(st, tval, "True Seeing");
    st.item_val.sv_potion_enlightenment = borg_lookup_sval_fail(st, tval, "Enlightenment");
    st.item_val.sv_potion_slime_mold = borg_lookup_sval_fail(st, tval, "Slime Mold Juice");
    st.item_val.sv_potion_berserk = borg_lookup_sval_fail(st, tval, "Berserk Strength");
    st.item_val.sv_potion_infravision = borg_lookup_sval_fail(st, tval, "Infravision");
    st.item_val.sv_potion_inc_exp = borg_lookup_sval_fail(st, tval, "Experience");

    tval = engine::tval_find_idx("scroll");
    st.item_val.sv_scroll_identify = borg_lookup_sval_fail(st, tval, "Identify Rune");
    st.item_val.sv_scroll_phase_door = borg_lookup_sval_fail(st, tval, "Phase Door");
    st.item_val.sv_scroll_teleport = borg_lookup_sval_fail(st, tval, "Teleportation");
    st.item_val.sv_scroll_word_of_recall = borg_lookup_sval_fail(st, tval, "Word of Recall");
    st.item_val.sv_scroll_enchant_armor = borg_lookup_sval_fail(st, tval, "Enchant Armour");
    st.item_val.sv_scroll_enchant_weapon_to_hit = borg_lookup_sval_fail(st, tval, "Enchant Weapon To-Hit");
    st.item_val.sv_scroll_enchant_weapon_to_dam = borg_lookup_sval_fail(st, tval, "Enchant Weapon To-Dam");
    st.item_val.sv_scroll_star_enchant_armor = borg_lookup_sval_fail(st, tval, "*Enchant Armour*");
    st.item_val.sv_scroll_star_enchant_weapon = borg_lookup_sval_fail(st, tval, "*Enchant Weapon*");
    st.item_val.sv_scroll_protection_from_evil = borg_lookup_sval_fail(st, tval, "Protection from Evil");
    st.item_val.sv_scroll_rune_of_protection = borg_lookup_sval_fail(st, tval, "Rune of Protection");
    st.item_val.sv_scroll_teleport_level = borg_lookup_sval_fail(st, tval, "Teleport Level");
    st.item_val.sv_scroll_deep_descent = borg_lookup_sval_fail(st, tval, "Deep Descent");
    st.item_val.sv_scroll_recharging = borg_lookup_sval_fail(st, tval, "Recharging");
    st.item_val.sv_scroll_banishment = borg_lookup_sval_fail(st, tval, "Banishment");
    st.item_val.sv_scroll_mass_banishment = borg_lookup_sval_fail(st, tval, "Mass Banishment");
    st.item_val.kv_scroll_mass_banishment = borg_lookup_kind(st, tval, st.item_val.sv_scroll_mass_banishment);
    st.item_val.sv_scroll_blessing = borg_lookup_sval_fail(st, tval, "Blessing");
    st.item_val.sv_scroll_holy_chant = borg_lookup_sval_fail(st, tval, "Holy Chant");
    st.item_val.sv_scroll_holy_prayer = borg_lookup_sval_fail(st, tval, "Holy Prayer");
    st.item_val.sv_scroll_detect_invis = borg_lookup_sval_fail(st, tval, "Detect Invisible");
    st.item_val.sv_scroll_satisfy_hunger = borg_lookup_sval_fail(st, tval, "Remove Hunger");
    st.item_val.sv_scroll_light = borg_lookup_sval_fail(st, tval, "Light");
    st.item_val.sv_scroll_mapping = borg_lookup_sval_fail(st, tval, "Magic Mapping");
    st.item_val.sv_scroll_acquirement = borg_lookup_sval_fail(st, tval, "Acquirement");
    st.item_val.sv_scroll_star_acquirement = borg_lookup_sval_fail(st, tval, "*Acquirement*");
    st.item_val.sv_scroll_remove_curse = borg_lookup_sval_fail(st, tval, "Remove Curse");
    st.item_val.kv_scroll_remove_curse = borg_lookup_kind(st, tval, st.item_val.sv_scroll_remove_curse);
    st.item_val.sv_scroll_star_remove_curse = borg_lookup_sval_fail(st, tval, "*Remove Curse*");
    st.item_val.kv_scroll_star_remove_curse = borg_lookup_kind(st, tval, st.item_val.sv_scroll_star_remove_curse);
    st.item_val.sv_scroll_monster_confusion = borg_lookup_sval_fail(st, tval, "Monster Confusion");
    st.item_val.sv_scroll_trap_door_destruction = borg_lookup_sval_fail(st, tval, "Door Destruction");
    st.item_val.sv_scroll_dispel_undead = borg_lookup_sval_fail(st, tval, "Dispel Undead");

    tval = engine::tval_find_idx("ring");
    st.item_val.sv_ring_flames = borg_lookup_sval_fail(st, tval, "Flames");
    st.item_val.sv_ring_ice = borg_lookup_sval_fail(st, tval, "Ice");
    st.item_val.sv_ring_acid = borg_lookup_sval_fail(st, tval, "Acid");
    st.item_val.sv_ring_lightning = borg_lookup_sval_fail(st, tval, "Lightning");
    st.item_val.sv_ring_digging = borg_lookup_sval_fail(st, tval, "Digging");
    st.item_val.sv_ring_speed = borg_lookup_sval_fail(st, tval, "Speed");
    st.item_val.sv_ring_damage = borg_lookup_sval_fail(st, tval, "Damage");
    st.item_val.sv_ring_dog = borg_lookup_sval_fail(st, tval, "the Dog");

    tval = engine::tval_find_idx("amulet");
    st.item_val.sv_amulet_teleportation = borg_lookup_sval_fail(st, tval, "Teleportation");

    tval = engine::tval_find_idx("rod");
    st.item_val.sv_rod_recall = borg_lookup_sval_fail(st, tval, "Recall");
    st.item_val.kv_rod_recall = borg_lookup_kind(st, tval, st.item_val.sv_rod_recall);
    st.item_val.sv_rod_detection = borg_lookup_sval_fail(st, tval, "Detection");
    st.item_val.sv_rod_illumination = borg_lookup_sval_fail(st, tval, "Illumination");
    st.item_val.sv_rod_speed = borg_lookup_sval_fail(st, tval, "Speed");
    st.item_val.sv_rod_mapping = borg_lookup_sval_fail(st, tval, "Magic Mapping");
    st.item_val.sv_rod_healing = borg_lookup_sval_fail(st, tval, "Healing");
    st.item_val.kv_rod_healing = borg_lookup_kind(st, tval, st.item_val.sv_rod_healing);
    st.item_val.sv_rod_light = borg_lookup_sval_fail(st, tval, "Light");
    st.item_val.sv_rod_fire_bolt = borg_lookup_sval_fail(st, tval, "Fire Bolts");
    st.item_val.sv_rod_elec_bolt = borg_lookup_sval_fail(st, tval, "Lightning Bolts");
    st.item_val.sv_rod_cold_bolt = borg_lookup_sval_fail(st, tval, "Frost Bolts");
    st.item_val.sv_rod_acid_bolt = borg_lookup_sval_fail(st, tval, "Acid Bolts");
    st.item_val.sv_rod_drain_life = borg_lookup_sval_fail(st, tval, "Drain Life");
    st.item_val.sv_rod_fire_ball = borg_lookup_sval_fail(st, tval, "Fire Balls");
    st.item_val.sv_rod_elec_ball = borg_lookup_sval_fail(st, tval, "Lightning Balls");
    st.item_val.sv_rod_cold_ball = borg_lookup_sval_fail(st, tval, "Cold Balls");
    st.item_val.sv_rod_acid_ball = borg_lookup_sval_fail(st, tval, "Acid Balls");
    st.item_val.sv_rod_teleport_other = borg_lookup_sval_fail(st, tval, "Teleport Other");
    st.item_val.sv_rod_slow_monster = borg_lookup_sval_fail(st, tval, "Slow Monster");
    st.item_val.sv_rod_sleep_monster = borg_lookup_sval_fail(st, tval, "Hold Monster");
    st.item_val.sv_rod_curing = borg_lookup_sval_fail(st, tval, "Curing");

    tval = engine::tval_find_idx("staff");
    st.item_val.sv_staff_teleportation = borg_lookup_sval_fail(st, tval, "Teleportation");
    st.item_val.sv_staff_destruction = borg_lookup_sval_fail(st, tval, "*Destruction*");
    st.item_val.sv_staff_speed = borg_lookup_sval_fail(st, tval, "Speed");
    st.item_val.sv_staff_healing = borg_lookup_sval_fail(st, tval, "Healing");
    st.item_val.sv_staff_the_magi = borg_lookup_sval_fail(st, tval, "the Magi");
    st.item_val.sv_staff_power = borg_lookup_sval_fail(st, tval, "Power");
    st.item_val.sv_staff_holiness = borg_lookup_sval_fail(st, tval, "Holiness");
    st.item_val.kv_staff_holiness = borg_lookup_kind(st, tval, st.item_val.sv_staff_holiness);
    st.item_val.sv_staff_curing = borg_lookup_sval_fail(st, tval, "Curing");
    st.item_val.sv_staff_sleep_monsters = borg_lookup_sval_fail(st, tval, "Sleep Monsters");
    st.item_val.sv_staff_slow_monsters = borg_lookup_sval_fail(st, tval, "Slow Monsters");
    st.item_val.sv_staff_detect_invis = borg_lookup_sval_fail(st, tval, "Detect Invisible");
    st.item_val.sv_staff_detect_evil = borg_lookup_sval_fail(st, tval, "Detect Evil");
    st.item_val.sv_staff_dispel_evil = borg_lookup_sval_fail(st, tval, "Dispel Evil");
    st.item_val.sv_staff_banishment = borg_lookup_sval_fail(st, tval, "Banishment");
    st.item_val.sv_staff_light = borg_lookup_sval_fail(st, tval, "Light");
    st.item_val.sv_staff_mapping = borg_lookup_sval_fail(st, tval, "Mapping");
    st.item_val.sv_staff_remove_curse = borg_lookup_sval_fail(st, tval, "Remove Curse");

    tval = engine::tval_find_idx("wand");
    st.item_val.sv_wand_light = borg_lookup_sval_fail(st, tval, "Light");
    st.item_val.sv_wand_teleport_away = borg_lookup_sval_fail(st, tval, "Teleport Other");
    st.item_val.sv_wand_stinking_cloud = borg_lookup_sval_fail(st, tval, "Stinking Cloud");
    st.item_val.kv_wand_stinking_cloud = borg_lookup_kind(st, tval, st.item_val.sv_wand_stinking_cloud);
    st.item_val.sv_wand_magic_missile = borg_lookup_sval_fail(st, tval, "Magic Missile");
    st.item_val.kv_wand_magic_missile = borg_lookup_kind(st, tval, st.item_val.sv_wand_magic_missile);
    st.item_val.sv_wand_annihilation = borg_lookup_sval_fail(st, tval, "Annihilation");
    st.item_val.kv_wand_annihilation = borg_lookup_kind(st, tval, st.item_val.sv_wand_annihilation);
    st.item_val.sv_wand_stone_to_mud = borg_lookup_sval_fail(st, tval, "Stone to Mud");
    st.item_val.sv_wand_wonder = borg_lookup_sval_fail(st, tval, "Wonder");
    st.item_val.sv_wand_hold_monster = borg_lookup_sval_fail(st, tval, "Hold Monster");
    st.item_val.sv_wand_slow_monster = borg_lookup_sval_fail(st, tval, "Slow Monster");
    st.item_val.sv_wand_fear_monster = borg_lookup_sval_fail(st, tval, "Scare Monster");
    st.item_val.sv_wand_confuse_monster = borg_lookup_sval_fail(st, tval, "Scare Monster");
    st.item_val.sv_wand_fire_bolt = borg_lookup_sval_fail(st, tval, "Fire Bolts");
    st.item_val.sv_wand_cold_bolt = borg_lookup_sval_fail(st, tval, "Frost Bolts");
    st.item_val.sv_wand_acid_bolt = borg_lookup_sval_fail(st, tval, "Acid Bolts");
    st.item_val.sv_wand_elec_bolt = borg_lookup_sval_fail(st, tval, "Lightning Bolts");
    st.item_val.sv_wand_fire_ball = borg_lookup_sval_fail(st, tval, "Fire Balls");
    st.item_val.sv_wand_cold_ball = borg_lookup_sval_fail(st, tval, "Cold Balls");
    st.item_val.sv_wand_acid_ball = borg_lookup_sval_fail(st, tval, "Acid Balls");
    st.item_val.sv_wand_elec_ball = borg_lookup_sval_fail(st, tval, "Lightning Bolts");
    st.item_val.sv_wand_dragon_cold = borg_lookup_sval_fail(st, tval, "Dragon's Frost");
    st.item_val.sv_wand_dragon_fire = borg_lookup_sval_fail(st, tval, "Dragon's Flame");
    st.item_val.sv_wand_drain_life = borg_lookup_sval_fail(st, tval, "Drain Life");

    tval = engine::tval_find_idx("sword");
    st.item_val.sv_dagger = borg_lookup_sval_fail(st, tval, "Dagger");

    tval = engine::tval_find_idx("bow");
    st.item_val.sv_sling = borg_lookup_sval_fail(st, tval, "Sling");
    st.item_val.sv_short_bow = borg_lookup_sval_fail(st, tval, "Short Bow");
    st.item_val.sv_long_bow = borg_lookup_sval_fail(st, tval, "Long Bow");
    st.item_val.sv_light_xbow = borg_lookup_sval_fail(st, tval, "Light Crossbow");
    st.item_val.sv_heavy_xbow = borg_lookup_sval_fail(st, tval, "Heavy Crossbow");

    tval = engine::tval_find_idx("arrow");
    st.item_val.sv_arrow_seeker = borg_lookup_sval_fail(st, tval, "Seeker Arrow");
    st.item_val.sv_arrow_mithril = borg_lookup_sval_fail(st, tval, "Mithril Arrow");

    tval = engine::tval_find_idx("bolt");
    st.item_val.sv_bolt_seeker = borg_lookup_sval_fail(st, tval, "Seeker Bolt");
    st.item_val.sv_bolt_mithril = borg_lookup_sval_fail(st, tval, "Mithril Bolt");

    tval = engine::tval_find_idx("gloves");
    st.item_val.sv_set_of_leather_gloves = borg_lookup_sval_fail(st, tval, "Set of Leather Gloves");

    tval = engine::tval_find_idx("cloak");
    st.item_val.sv_cloak = borg_lookup_sval_fail(st, tval, "Cloak");

    tval = engine::tval_find_idx("soft armor");
    st.item_val.sv_robe = borg_lookup_sval_fail(st, tval, "Robe");

    tval = engine::tval_find_idx("crown");
    st.item_val.sv_iron_crown = borg_lookup_sval_fail(st, tval, "Iron Crown");

    tval = engine::tval_find_idx("dragon armor");
    st.item_val.sv_dragon_blue = borg_lookup_sval_fail(st, tval, "Blue Dragon Scale Mail");
    st.item_val.sv_dragon_black = borg_lookup_sval_fail(st, tval, "Black Dragon Scale Mail");
    st.item_val.sv_dragon_white = borg_lookup_sval_fail(st, tval, "White Dragon Scale Mail");
    st.item_val.sv_dragon_red = borg_lookup_sval_fail(st, tval, "Red Dragon Scale Mail");
    st.item_val.sv_dragon_green = borg_lookup_sval_fail(st, tval, "Green Dragon Scale Mail");
    st.item_val.sv_dragon_multihued = borg_lookup_sval_fail(st, tval, "Multi-Hued Dragon Scale Mail");
    st.item_val.sv_dragon_shining = borg_lookup_sval_fail(st, tval, "Shining Dragon Scale Mail");
    st.item_val.sv_dragon_law = borg_lookup_sval_fail(st, tval, "Law Dragon Scale Mail");
    st.item_val.sv_dragon_gold = borg_lookup_sval_fail(st, tval, "Gold Dragon Scale Mail");
    st.item_val.sv_dragon_chaos = borg_lookup_sval_fail(st, tval, "Chaos Dragon Scale Mail");
    st.item_val.sv_dragon_balance = borg_lookup_sval_fail(st, tval, "Balance Dragon Scale Mail");
    st.item_val.sv_dragon_power = borg_lookup_sval_fail(st, tval, "Power Dragon Scale Mail");
}

/// The k_idx of the object kind with the given `tval` and `sval`, or 0
/// (with a message) when there is none (`borg_lookup_kind`).
pub fn borg_lookup_kind(st: &mut BorgState, tval: i32, sval: i32) -> i32 {
    let tval_sval = |k: usize| {
        let kind = engine::kind(k);
        (kind.tval, kind.sval)
    };
    if let Some(k) = find_kind(engine::k_max(), tval_sval, tval, sval) {
        return k;
    }

    /* Failure */
    engine::msg(st, &no_object_message(engine::tval_find_name(tval), tval, sval));
    0
}

/// The first index from 1 (index 0 is skipped, as in C) whose kind has
/// this `tval` and `sval`, among `k_max` kinds (`tval_sval(k)` gives a
/// kind's pair).
fn find_kind(k_max: usize, tval_sval: impl Fn(usize) -> (i32, i32), tval: i32, sval: i32) -> Option<i32> {
    (1..k_max).find(|&k| tval_sval(k) == (tval, sval)).map(|k| k as i32)
}

/// `format("No object (%s,%d,%d)", name, tval, sval)`.
fn no_object_message(name: &[u8], tval: i32, sval: i32) -> Vec<u8> {
    let mut m = b"No object (".to_vec();
    m.extend_from_slice(name);
    m.extend_from_slice(format!(",{},{})", tval, sval).as_bytes());
    m
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_object_message_matches_c_format() {
        assert_eq!(no_object_message(b"potion", 26, -1), b"No object (potion,26,-1)".to_vec());
    }
}
