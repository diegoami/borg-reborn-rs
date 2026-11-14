//! Port of `borg-item-activation.c`: activation indexes by name.
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

/// The activation index of each activation the Borg knows (`act_*`).
#[derive(Clone, Copy, Default, Debug)]
pub struct ActivationIds {
    pub dragon_power: i32,
    pub dragon_shining: i32,
    pub dragon_balance: i32,
    pub dragon_law: i32,
    pub dragon_chaos: i32,
    pub dragon_gold: i32,
    pub dragon_multihued: i32,
    pub dragon_red: i32,
    pub dragon_green: i32,
    pub dragon_blue: i32,
    pub ring_lightning: i32,
    pub ring_ice: i32,
    pub ring_flames: i32,
    pub ring_acid: i32,
    pub shroom_purging: i32,
    pub shroom_sprinting: i32,
    pub shroom_debility: i32,
    pub shroom_stone: i32,
    pub shroom_terror: i32,
    pub shroom_emergency: i32,
    pub food_waybread: i32,
    pub drink_breath: i32,
    pub staff_holy: i32,
    pub staff_magi: i32,
    pub wand_breath: i32,
    pub wonder: i32,
    pub berserker: i32,
    pub starlight2: i32,
    pub starlight: i32,
    pub polymorph: i32,
    pub door_dest: i32,
    pub disable_traps: i32,
    pub light_line: i32,
    pub mon_scare: i32,
    pub sleep_all: i32,
    pub mon_confuse: i32,
    pub mon_slow: i32,
    pub confuse2: i32,
    pub tele_other: i32,
    pub stone_to_mud: i32,
    pub stinking_cloud: i32,
    pub arrow: i32,
    pub bizarre: i32,
    pub mana_bolt: i32,
    pub missile: i32,
    pub drain_life4: i32,
    pub drain_life3: i32,
    pub drain_life2: i32,
    pub drain_life1: i32,
    pub elec_ball2: i32,
    pub elec_ball: i32,
    pub elec_bolt: i32,
    pub acid_ball: i32,
    pub acid_bolt3: i32,
    pub acid_bolt2: i32,
    pub acid_bolt: i32,
    pub cold_ball160: i32,
    pub cold_ball100: i32,
    pub cold_ball50: i32,
    pub cold_ball2: i32,
    pub cold_bolt2: i32,
    pub cold_bolt: i32,
    pub fire_ball200: i32,
    pub fire_ball2: i32,
    pub fire_ball: i32,
    pub fire_ball72: i32,
    pub fire_bolt3: i32,
    pub fire_bolt2: i32,
    pub fire_bolt: i32,
    pub firebrand: i32,
    pub rem_fear_pois: i32,
    pub restore_life: i32,
    pub rage_bless_resist: i32,
    pub star_ball: i32,
    pub sleepii: i32,
    pub dispel_all: i32,
    pub dispel_undead: i32,
    pub dispel_evil60: i32,
    pub dispel_evil: i32,
    pub haste2: i32,
    pub haste1: i32,
    pub haste: i32,
    pub probing: i32,
    pub clairvoyance: i32,
    pub illumination: i32,
    pub loskill: i32,
    pub losconf: i32,
    pub lossleep: i32,
    pub losslow: i32,
    pub destruction2: i32,
    pub earthquakes: i32,
    pub deep_descent: i32,
    pub recall: i32,
    pub blessing3: i32,
    pub blessing2: i32,
    pub blessing: i32,
    pub satisfy: i32,
    pub protevil: i32,
    pub banishment: i32,
    pub recharge: i32,
    pub destroy_doors: i32,
    pub glyph: i32,
    pub mapping: i32,
    pub confusing: i32,
    pub tele_level: i32,
    pub tele_long: i32,
    pub tele_phase: i32,
    pub light: i32,
    pub remove_curse2: i32,
    pub remove_curse: i32,
    pub enchant_armor2: i32,
    pub enchant_armor: i32,
    pub enchant_weapon: i32,
    pub enchant_todam: i32,
    pub enchant_tohit: i32,
    pub detect_objects: i32,
    pub detect_all: i32,
    pub detect_evil: i32,
    pub detect_invis: i32,
    pub detect_treasure: i32,
    pub resist_all: i32,
    pub resist_pois: i32,
    pub resist_cold: i32,
    pub resist_fire: i32,
    pub resist_elec: i32,
    pub resist_acid: i32,
    pub shero: i32,
    pub hero: i32,
    pub enlightenment: i32,
    pub tmd_esp: i32,
    pub tmd_sinvis: i32,
    pub tmd_infra: i32,
    pub tmd_free_act: i32,
    pub restore_st_lev: i32,
    pub restore_all: i32,
    pub restore_con: i32,
    pub restore_dex: i32,
    pub restore_wis: i32,
    pub restore_int: i32,
    pub restore_str: i32,
    pub nimbleness: i32,
    pub toughness: i32,
    pub contemplation: i32,
    pub intellect: i32,
    pub brawn: i32,
    pub restore_mana: i32,
    pub restore_exp: i32,
    pub heal3: i32,
    pub heal2: i32,
    pub heal1: i32,
    pub cure_temp: i32,
    pub cure_nonorlybig: i32,
    pub cure_full2: i32,
    pub cure_full: i32,
    pub cure_critical: i32,
    pub cure_serious: i32,
    pub cure_light: i32,
    pub cure_body: i32,
    pub cure_mind: i32,
    pub cure_confusion: i32,
    pub cure_paranoia: i32,
}

/// Find an activation's index by name (`borg_findact`). On failure this is
/// a startup failure, and 0 is returned.
pub fn borg_findact(st: &mut BorgState, act_name: &str) -> i32 {
    if let Some(index) = engine::find_activation(act_name) {
        return index;
    }
    borg_note(
        st,
        &format!("**STARTUP FAILURE** activation lookup failure - {} ", act_name),
    );
    st.init.init_failure = true;
    0
}

/// Look up every activation (`borg_init_item_activation`).
pub fn borg_init_item_activation(st: &mut BorgState) {
    st.act.dragon_power = borg_findact(st, "DRAGON_POWER");
    st.act.dragon_shining = borg_findact(st, "DRAGON_SHINING");
    st.act.dragon_balance = borg_findact(st, "DRAGON_BALANCE");
    st.act.dragon_law = borg_findact(st, "DRAGON_LAW");
    st.act.dragon_chaos = borg_findact(st, "DRAGON_CHAOS");
    st.act.dragon_gold = borg_findact(st, "DRAGON_GOLD");
    st.act.dragon_multihued = borg_findact(st, "DRAGON_MULTIHUED");
    st.act.dragon_red = borg_findact(st, "DRAGON_RED");
    st.act.dragon_green = borg_findact(st, "DRAGON_GREEN");
    st.act.dragon_blue = borg_findact(st, "DRAGON_BLUE");
    st.act.ring_lightning = borg_findact(st, "RING_LIGHTNING");
    st.act.ring_ice = borg_findact(st, "RING_ICE");
    st.act.ring_flames = borg_findact(st, "RING_FLAMES");
    st.act.ring_acid = borg_findact(st, "RING_ACID");
    st.act.shroom_purging = borg_findact(st, "SHROOM_PURGING");
    st.act.shroom_sprinting = borg_findact(st, "SHROOM_SPRINTING");
    st.act.shroom_debility = borg_findact(st, "SHROOM_DEBILITY");
    st.act.shroom_stone = borg_findact(st, "SHROOM_STONE");
    st.act.shroom_terror = borg_findact(st, "SHROOM_TERROR");
    st.act.shroom_emergency = borg_findact(st, "SHROOM_EMERGENCY");
    st.act.food_waybread = borg_findact(st, "FOOD_WAYBREAD");
    st.act.drink_breath = borg_findact(st, "DRINK_BREATH");
    st.act.staff_holy = borg_findact(st, "STAFF_HOLY");
    st.act.staff_magi = borg_findact(st, "STAFF_MAGI");
    st.act.wand_breath = borg_findact(st, "WAND_BREATH");
    st.act.wonder = borg_findact(st, "WONDER");
    st.act.berserker = borg_findact(st, "BERSERKER");
    st.act.starlight2 = borg_findact(st, "STARLIGHT2");
    st.act.starlight = borg_findact(st, "STARLIGHT");
    st.act.polymorph = borg_findact(st, "POLYMORPH");
    st.act.door_dest = borg_findact(st, "DOOR_DEST");
    st.act.disable_traps = borg_findact(st, "DISABLE_TRAPS");
    st.act.light_line = borg_findact(st, "LIGHT_LINE");
    st.act.mon_scare = borg_findact(st, "MON_SCARE");
    st.act.sleep_all = borg_findact(st, "SLEEP_ALL");
    st.act.mon_confuse = borg_findact(st, "MON_CONFUSE");
    st.act.mon_slow = borg_findact(st, "MON_SLOW");
    st.act.confuse2 = borg_findact(st, "CONFUSE2");
    st.act.tele_other = borg_findact(st, "TELE_OTHER");
    st.act.stone_to_mud = borg_findact(st, "STONE_TO_MUD");
    st.act.stinking_cloud = borg_findact(st, "STINKING_CLOUD");
    st.act.arrow = borg_findact(st, "ARROW");
    st.act.bizarre = borg_findact(st, "BIZARRE");
    st.act.mana_bolt = borg_findact(st, "MANA_BOLT");
    st.act.missile = borg_findact(st, "MISSILE");
    st.act.drain_life4 = borg_findact(st, "DRAIN_LIFE4");
    st.act.drain_life3 = borg_findact(st, "DRAIN_LIFE3");
    st.act.drain_life2 = borg_findact(st, "DRAIN_LIFE2");
    st.act.drain_life1 = borg_findact(st, "DRAIN_LIFE1");
    st.act.elec_ball2 = borg_findact(st, "ELEC_BALL2");
    st.act.elec_ball = borg_findact(st, "ELEC_BALL");
    st.act.elec_bolt = borg_findact(st, "ELEC_BOLT");
    st.act.acid_ball = borg_findact(st, "ACID_BALL");
    st.act.acid_bolt3 = borg_findact(st, "ACID_BOLT3");
    st.act.acid_bolt2 = borg_findact(st, "ACID_BOLT2");
    st.act.acid_bolt = borg_findact(st, "ACID_BOLT");
    st.act.cold_ball160 = borg_findact(st, "COLD_BALL160");
    st.act.cold_ball100 = borg_findact(st, "COLD_BALL100");
    st.act.cold_ball50 = borg_findact(st, "COLD_BALL50");
    st.act.cold_ball2 = borg_findact(st, "COLD_BALL2");
    st.act.cold_bolt2 = borg_findact(st, "COLD_BOLT2");
    st.act.cold_bolt = borg_findact(st, "COLD_BOLT");
    st.act.fire_ball200 = borg_findact(st, "FIRE_BALL200");
    st.act.fire_ball2 = borg_findact(st, "FIRE_BALL2");
    st.act.fire_ball = borg_findact(st, "FIRE_BALL");
    st.act.fire_ball72 = borg_findact(st, "FIRE_BALL72");
    st.act.fire_bolt3 = borg_findact(st, "FIRE_BOLT3");
    st.act.fire_bolt2 = borg_findact(st, "FIRE_BOLT2");
    st.act.fire_bolt = borg_findact(st, "FIRE_BOLT");
    st.act.firebrand = borg_findact(st, "FIREBRAND");
    st.act.rem_fear_pois = borg_findact(st, "REM_FEAR_POIS");
    st.act.restore_life = borg_findact(st, "RESTORE_LIFE");
    st.act.rage_bless_resist = borg_findact(st, "RAGE_BLESS_RESIST");
    st.act.star_ball = borg_findact(st, "STAR_BALL");
    st.act.sleepii = borg_findact(st, "SLEEPII");
    st.act.dispel_all = borg_findact(st, "DISPEL_ALL");
    st.act.dispel_undead = borg_findact(st, "DISPEL_UNDEAD");
    st.act.dispel_evil60 = borg_findact(st, "DISPEL_EVIL60");
    st.act.dispel_evil = borg_findact(st, "DISPEL_EVIL");
    st.act.haste2 = borg_findact(st, "HASTE2");
    st.act.haste1 = borg_findact(st, "HASTE1");
    st.act.haste = borg_findact(st, "HASTE");
    st.act.probing = borg_findact(st, "PROBING");
    st.act.clairvoyance = borg_findact(st, "CLAIRVOYANCE");
    st.act.illumination = borg_findact(st, "ILLUMINATION");
    st.act.loskill = borg_findact(st, "LOSKILL");
    st.act.losconf = borg_findact(st, "LOSCONF");
    st.act.lossleep = borg_findact(st, "LOSSLEEP");
    st.act.losslow = borg_findact(st, "LOSSLOW");
    st.act.destruction2 = borg_findact(st, "DESTRUCTION2");
    st.act.earthquakes = borg_findact(st, "EARTHQUAKES");
    st.act.deep_descent = borg_findact(st, "DEEP_DESCENT");
    st.act.recall = borg_findact(st, "RECALL");
    st.act.blessing3 = borg_findact(st, "BLESSING3");
    st.act.blessing2 = borg_findact(st, "BLESSING2");
    st.act.blessing = borg_findact(st, "BLESSING");
    st.act.satisfy = borg_findact(st, "SATISFY");
    st.act.protevil = borg_findact(st, "PROTEVIL");
    st.act.banishment = borg_findact(st, "BANISHMENT");
    st.act.recharge = borg_findact(st, "RECHARGE");
    st.act.destroy_doors = borg_findact(st, "DESTROY_DOORS");
    st.act.glyph = borg_findact(st, "GLYPH");
    st.act.mapping = borg_findact(st, "MAPPING");
    st.act.confusing = borg_findact(st, "CONFUSING");
    st.act.tele_level = borg_findact(st, "TELE_LEVEL");
    st.act.tele_long = borg_findact(st, "TELE_LONG");
    st.act.tele_phase = borg_findact(st, "TELE_PHASE");
    st.act.light = borg_findact(st, "LIGHT");
    st.act.remove_curse2 = borg_findact(st, "REMOVE_CURSE2");
    st.act.remove_curse = borg_findact(st, "REMOVE_CURSE");
    st.act.enchant_armor2 = borg_findact(st, "ENCHANT_ARMOR2");
    st.act.enchant_armor = borg_findact(st, "ENCHANT_ARMOR");
    st.act.enchant_weapon = borg_findact(st, "ENCHANT_WEAPON");
    st.act.enchant_todam = borg_findact(st, "ENCHANT_TODAM");
    st.act.enchant_tohit = borg_findact(st, "ENCHANT_TOHIT");
    st.act.detect_objects = borg_findact(st, "DETECT_OBJECTS");
    st.act.detect_all = borg_findact(st, "DETECT_ALL");
    st.act.detect_evil = borg_findact(st, "DETECT_EVIL");
    st.act.detect_invis = borg_findact(st, "DETECT_INVIS");
    st.act.detect_treasure = borg_findact(st, "DETECT_TREASURE");
    st.act.resist_all = borg_findact(st, "RESIST_ALL");
    st.act.resist_pois = borg_findact(st, "RESIST_POIS");
    st.act.resist_cold = borg_findact(st, "RESIST_COLD");
    st.act.resist_fire = borg_findact(st, "RESIST_FIRE");
    st.act.resist_elec = borg_findact(st, "RESIST_ELEC");
    st.act.resist_acid = borg_findact(st, "RESIST_ACID");
    st.act.shero = borg_findact(st, "SHERO");
    st.act.hero = borg_findact(st, "HERO");
    st.act.enlightenment = borg_findact(st, "ENLIGHTENMENT");
    st.act.tmd_esp = borg_findact(st, "TMD_ESP");
    st.act.tmd_sinvis = borg_findact(st, "TMD_SINVIS");
    st.act.tmd_infra = borg_findact(st, "TMD_INFRA");
    st.act.tmd_free_act = borg_findact(st, "TMD_FREE_ACT");
    st.act.restore_st_lev = borg_findact(st, "RESTORE_ST_LEV");
    st.act.restore_all = borg_findact(st, "RESTORE_ALL");
    st.act.restore_con = borg_findact(st, "RESTORE_CON");
    st.act.restore_dex = borg_findact(st, "RESTORE_DEX");
    st.act.restore_wis = borg_findact(st, "RESTORE_WIS");
    st.act.restore_int = borg_findact(st, "RESTORE_INT");
    st.act.restore_str = borg_findact(st, "RESTORE_STR");
    st.act.nimbleness = borg_findact(st, "NIMBLENESS");
    st.act.toughness = borg_findact(st, "TOUGHNESS");
    st.act.contemplation = borg_findact(st, "CONTEMPLATION");
    st.act.intellect = borg_findact(st, "INTELLECT");
    st.act.brawn = borg_findact(st, "BRAWN");
    st.act.restore_mana = borg_findact(st, "RESTORE_MANA");
    st.act.restore_exp = borg_findact(st, "RESTORE_EXP");
    st.act.heal3 = borg_findact(st, "HEAL3");
    st.act.heal2 = borg_findact(st, "HEAL2");
    st.act.heal1 = borg_findact(st, "HEAL1");
    st.act.cure_temp = borg_findact(st, "CURE_TEMP");
    st.act.cure_nonorlybig = borg_findact(st, "CURE_NONORLYBIG");
    st.act.cure_full2 = borg_findact(st, "CURE_FULL2");
    st.act.cure_full = borg_findact(st, "CURE_FULL");
    st.act.cure_critical = borg_findact(st, "CURE_CRITICAL");
    st.act.cure_serious = borg_findact(st, "CURE_SERIOUS");
    st.act.cure_light = borg_findact(st, "CURE_LIGHT");
    st.act.cure_body = borg_findact(st, "CURE_BODY");
    st.act.cure_mind = borg_findact(st, "CURE_MIND");
    st.act.cure_confusion = borg_findact(st, "CURE_CONFUSION");
    st.act.cure_paranoia = borg_findact(st, "CURE_PARANOIA");
}
