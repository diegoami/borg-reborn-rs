//! Port of `borg-magic.c`: the basic magic definitions and routines to cast
//! spells.
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

use core::ffi::CStr;

use crate::borg_io::{borg_keypress, borg_note_bytes};
use crate::borg_headers::borg_magic_h::*;
use crate::borg_headers::borg_trait_h::{
    BI_CDEPTH, BI_CLASS, BI_CLEVEL, BI_CURSP, BI_FAIL1, BI_FAIL2, BI_ISAFRAID, BI_ISBLIND,
    BI_ISCONFUSED, BI_ISFORGET, BI_ISHEAVYSTUN, BI_ISSTUN, BI_MAXSP,
};
use crate::headers::list_effects_h::{EF_NOURISH, EF_TELEPORT};
use crate::headers::list_player_flags_h::PF_ZERO_FAIL;
use crate::headers::list_tvals_h::{
    TV_MAGIC_BOOK, TV_NATURE_BOOK, TV_OTHER_BOOK, TV_PRAYER_BOOK, TV_SHADOW_BOOK,
};
use crate::headers::ui_event_h::keycode_t;
use crate::state::BorgState;
use crate::engine;

/// Spell status values (borg-magic.h)
#[allow(dead_code)] // the default 0 of a zeroed entry; never named, as in C
pub const BORG_MAGIC_ICKY: u8 = 0; // Spell is illegible
pub const BORG_MAGIC_LOST: u8 = 1; // Spell is forgotten
pub const BORG_MAGIC_HIGH: u8 = 2; // Spell is high level
pub const BORG_MAGIC_OKAY: u8 = 3; // Spell is learnable
pub const BORG_MAGIC_TEST: u8 = 4; // Spell is untried
pub const BORG_MAGIC_KNOW: u8 = 5; // Spell is known

/// `player->spell_flags` bits (player.h)
const PY_SPELL_LEARNED: i32 = 0x01; // Spell has been learned
const PY_SPELL_WORKED: i32 = 0x02; // Spell has been successfully tried
const PY_SPELL_FORGOTTEN: i32 = 0x04; // Spell has been forgotten

/// Class indices (borg-trait.h)
const CLASS_MAGE: i32 = 1;
const CLASS_DRUID: i32 = 2;
const CLASS_PRIEST: i32 = 3;
const CLASS_NECROMANCER: i32 = 4;
const CLASS_PALADIN: i32 = 5;
const CLASS_ROGUE: i32 = 6;
const CLASS_RANGER: i32 = 7;
const CLASS_BLACKGUARD: i32 = 8;

/// A spell/prayer in a book (`struct borg_magic`), with the C field names.
#[derive(Clone, Copy, Debug)]
pub struct BorgMagic {
    /// Textual name (C: `const char *`, from the rating table; an entry
    /// whose name did not match the game's is left blank, NULL in C)
    pub name: &'static CStr,
    /// Status (`BORG_MAGIC_*`)
    pub status: u8,
    /// effect index
    pub effect_index: u16,
    /// Usefulness
    pub rating: u8,
    /// Required level
    pub level: u8,
    /// Required power
    pub power: u8,
    /// Minimum chance of failure
    pub sfail: u8,
    /// offset of this spell in the book it is in
    pub book_offset: i32,
    /// book index
    pub book: i32,
    /// Times this spell was cast
    pub times: i32,
    pub spell_enum: borg_spells,
}

impl Default for BorgMagic {
    /// The zeroed entry `mem_zalloc` gave.
    fn default() -> Self {
        BorgMagic {
            name: c"",
            status: 0,
            effect_index: 0,
            rating: 0,
            level: 0,
            power: 0,
            sfail: 0,
            book_offset: 0,
            book: 0,
            times: 0,
            spell_enum: 0,
        }
    }
}

/// A spell's name, rating and enum (`struct borg_spell_rating`).
#[derive(Clone, Copy, Debug)]
pub struct SpellRating {
    pub name: &'static CStr,
    pub rating: u8,
    pub spell_enum: borg_spells,
}

const fn rating(name: &'static CStr, rating: u8, spell_enum: borg_spells) -> SpellRating {
    SpellRating { name, rating, spell_enum }
}

/// The module's globals.
#[derive(Default)]
pub struct MagicState {
    /// `borg_magics`: spell info, indexed by spell number. Empty stands for
    /// the C NULL (before `borg_prepare_book_info`, or for a class without
    /// spells, which never gets a table).
    pub magics: Vec<BorgMagic>,
    /// `borg_spell_ratings`: the rating table of the player's class.
    pub ratings: Option<&'static [SpellRating]>,
}

// !FIX !TODO for now put this in the code.  It should probably end up in borg.txt or a new borg.cfg
// I also gave low ratings to spells that are new since the borg doesn't know when to use them yet.
static RATINGS_MAGE: [SpellRating; 30] = [
    rating(c"Magic Missile", 95, MAGIC_MISSILE),
    rating(c"Light Room", 65, LIGHT_ROOM),
    rating(c"Find Traps, Doors & Stairs", 85, FIND_TRAPS_DOORS_STAIRS),
    rating(c"Phase Door", 95, PHASE_DOOR),
    rating(c"Electric Arc", 85, ELECTRIC_ARC),
    rating(c"Detect Monsters", 85, DETECT_MONSTERS),
    rating(c"Fire Ball", 75, FIRE_BALL),
    rating(c"Recharging", 65, RECHARGING),
    rating(c"Identify Rune", 95, IDENTIFY_RUNE),
    rating(c"Treasure Detection", 5, TREASURE_DETECTION), // borg never uses this
    rating(c"Frost Bolt", 75, FROST_BOLT),
    rating(c"Reveal Monsters", 85, REVEAL_MONSTERS),
    rating(c"Acid Spray", 75, ACID_SPRAY),
    rating(c"Disable Traps, Destroy Doors", 95, DISABLE_TRAPS_DESTROY_DOORS),
    rating(c"Teleport Self", 95, TELEPORT_SELF),
    rating(c"Teleport Other", 75, TELEPORT_OTHER),
    rating(c"Resistance", 90, RESISTANCE),
    rating(c"Tap Magical Energy", 5, TAP_MAGICAL_ENERGY), // need to figure out when to cast this one
    rating(c"Mana Channel", 95, MANA_CHANNEL),
    rating(c"Door Creation", 65, DOOR_CREATION),
    rating(c"Mana Bolt", 95, MANA_BOLT),
    rating(c"Teleport Level", 65, TELEPORT_LEVEL),
    rating(c"Detection", 95, DETECTION),
    rating(c"Dimension Door", 95, DIMENSION_DOOR),
    rating(c"Thrust Away", 55, THRUST_AWAY),
    rating(c"Shock Wave", 85, SHOCK_WAVE),
    rating(c"Explosion", 85, EXPLOSION),
    rating(c"Banishment", 75, BANISHMENT),
    rating(c"Mass Banishment", 65, MASS_BANISHMENT),
    rating(c"Mana Storm", 75, MANA_STORM),
];
static RATINGS_DRUID: [SpellRating; 27] = [
    rating(c"Detect Life", 95, DETECT_LIFE),
    rating(c"Fox Form", 5, FOX_FORM), // !FIX !TODO need to know when to cast any of the shapechanges
    rating(c"Remove Hunger", 85, REMOVE_HUNGER),
    rating(c"Stinking Cloud", 95, STINKING_CLOUD),
    rating(c"Confuse Monster", 55, CONFUSE_MONSTER),
    rating(c"Slow Monster", 65, SLOW_MONSTER),
    rating(c"Cure Poison", 55, CURE_POISON),
    rating(c"Resist Poison", 60, RESIST_POISON),
    rating(c"Turn Stone to Mud", 80, TURN_STONE_TO_MUD),
    rating(c"Sense Surroundings", 80, SENSE_SURROUNDINGS),
    rating(c"Lightning Strike", 85, LIGHTNING_STRIKE),
    rating(c"Earth Rising", 70, EARTH_RISING),
    rating(c"Trance", 55, TRANCE),
    rating(c"Mass Sleep", 80, MASS_SLEEP),
    rating(c"Become Pukel-man", 5, BECOME_PUKEL_MAN), // !FIX !TODO shapechange
    rating(c"Eagle's Flight", 5, EAGLES_FLIGHT), // !FIX !TODO shapechange
    rating(c"Bear Form", 5, BEAR_FORM), // !FIX !TODO shapechange
    rating(c"Tremor", 80, TREMOR),
    rating(c"Haste Self", 90, HASTE_SELF),
    rating(c"Revitalize", 95, REVITALIZE),
    rating(c"Rapid Regeneration", 55, RAPID_REGENERATION),
    rating(c"Herbal Curing", 90, HERBAL_CURING),
    rating(c"Meteor Swarm", 90, METEOR_SWARM),
    rating(c"Rift", 90, RIFT),
    rating(c"Ice Storm", 85, ICE_STORM),
    rating(c"Volcanic Eruption", 60, VOLCANIC_ERUPTION),
    rating(c"River of Lightning", 90, RIVER_OF_LIGHTNING),
];
static RATINGS_PRIEST: [SpellRating; 28] = [
    rating(c"Call Light", 65, CALL_LIGHT),
    rating(c"Detect Evil", 85, DETECT_EVIL),
    rating(c"Minor Healing", 65, MINOR_HEALING),
    rating(c"Bless", 85, BLESS),
    rating(c"Sense Invisible", 75, SENSE_INVISIBLE),
    rating(c"Heroism", 75, HEROISM),
    rating(c"Orb of Draining", 95, ORB_OF_DRAINING),
    rating(c"Spear of Light", 75, SPEAR_OF_LIGHT),
    rating(c"Dispel Undead", 65, DISPEL_UNDEAD),
    rating(c"Dispel Evil", 65, DISPEL_EVIL),
    rating(c"Protection from Evil", 85, PROTECTION_FROM_EVIL),
    rating(c"Remove Curse", 85, REMOVE_CURSE),
    rating(c"Portal", 85, PORTAL),
    rating(c"Remembrance", 75, REMEMBRANCE),
    rating(c"Word of Recall", 95, WORD_OF_RECALL),
    rating(c"Healing", 95, HEALING),
    rating(c"Restoration", 75, RESTORATION),
    rating(c"Clairvoyance", 85, CLAIRVOYANCE),
    rating(c"Enchant Weapon", 75, ENCHANT_WEAPON),
    rating(c"Enchant Armour", 75, ENCHANT_ARMOUR),
    rating(c"Smite Evil", 75, SMITE_EVIL),
    rating(c"Glyph of Warding", 95, GLYPH_OF_WARDING),
    rating(c"Demon Bane", 85, DEMON_BANE),
    rating(c"Banish Evil", 85, BANISH_EVIL),
    rating(c"Word of Destruction", 75, WORD_OF_DESTRUCTION),
    rating(c"Holy Word", 85, HOLY_WORD),
    rating(c"Spear of Orom\xC3\xab", 85, SPEAR_OF_OROME), // "Spear of Orom(e + diaresis)"
    rating(c"Light of Varda", 85, LIGHT_OF_VARDA), // "Light of Varda"
];
static RATINGS_NECROMANCER: [SpellRating; 26] = [
    rating(c"Nether Bolt", 95, NETHER_BOLT),
    rating(c"Sense Invisible", 85, SENSE_INVISIBLE),
    rating(c"Create Darkness", 5, CREATE_DARKNESS),
    rating(c"Bat Form", 5, BAT_FORM), // !FIX !TODO shapechange
    rating(c"Read Minds", 85, READ_MINDS),
    rating(c"Tap Unlife", 85, TAP_UNLIFE),
    rating(c"Crush", 95, CRUSH),
    rating(c"Sleep Evil", 85, SLEEP_EVIL),
    rating(c"Shadow Shift", 95, SHADOW_SHIFT),
    rating(c"Disenchant", 25, DISENCHANT),
    rating(c"Frighten", 85, FRIGHTEN),
    rating(c"Vampire Strike", 75, VAMPIRE_STRIKE),
    rating(c"Dispel Life", 65, DISPEL_LIFE),
    rating(c"Dark Spear", 65, DARK_SPEAR),
    rating(c"Warg Form", 5, WARG_FORM), // !FIX !TODO shapechange
    rating(c"Banish Spirits", 65, BANISH_SPIRITS),
    rating(c"Annihilate", 95, ANNIHILATE),
    rating(c"Grond's Blow", 85, GRONDS_BLOW),
    rating(c"Unleash Chaos", 85, UNLEASH_CHAOS),
    rating(c"Fume of Mordor", 75, FUME_OF_MORDOR),
    rating(c"Storm of Darkness", 65, STORM_OF_DARKNESS),
    rating(c"Power Sacrifice", 5, POWER_SACRIFICE), // not sure if this is borg happy.
    rating(c"Zone of Unmagic", 5, ZONE_OF_UNMAGIC), // !FIX !TODO defense?  not sure how to code.
    rating(c"Vampire Form", 5, VAMPIRE_FORM), // !FIX !TODO shapechange
    rating(c"Curse", 65, CURSE),
    rating(c"Command", 5, COMMAND), // !FIX !TODO defense?  not sure how to code.
];
static RATINGS_PALADIN: [SpellRating; 16] = [
    rating(c"Bless", 95, BLESS),
    rating(c"Detect Evil", 85, DETECT_EVIL),
    rating(c"Call Light", 85, CALL_LIGHT),
    rating(c"Minor Healing", 95, MINOR_HEALING),
    rating(c"Sense Invisible", 65, SENSE_INVISIBLE),
    rating(c"Heroism", 85, HEROISM),
    rating(c"Protection from Evil", 85, PROTECTION_FROM_EVIL),
    rating(c"Remove Curse", 65, REMOVE_CURSE),
    rating(c"Word of Recall", 95, WORD_OF_RECALL),
    rating(c"Healing", 95, HEALING),
    rating(c"Clairvoyance", 85, CLAIRVOYANCE),
    rating(c"Smite Evil", 55, SMITE_EVIL),
    rating(c"Demon Bane", 55, DEMON_BANE),
    rating(c"Enchant Weapon", 75, ENCHANT_WEAPON),
    rating(c"Enchant Armour", 85, ENCHANT_ARMOUR),
    rating(c"Single Combat", 95, SINGLE_COMBAT), // !FIX !TODO defense?  not sure how to code.
];
static RATINGS_ROGUE: [SpellRating; 10] = [
    rating(c"Detect Monsters", 85, DETECT_MONSTERS),
    rating(c"Phase Door", 95, PHASE_DOOR),
    rating(c"Object Detection", 55, OBJECT_DETECTION),
    rating(c"Detect Stairs", 55, DETECT_STAIRS),
    rating(c"Recharging", 85, RECHARGING),
    rating(c"Reveal Monsters", 85, REVEAL_MONSTERS),
    rating(c"Teleport Self", 95, TELEPORT_SELF),
    rating(c"Hit and Run", 15, HIT_AND_RUN), // !FIX !TODO not sure how to code this
    rating(c"Teleport Other", 85, TELEPORT_OTHER),
    rating(c"Teleport Level", 75, TELEPORT_LEVEL),
];
static RATINGS_RANGER: [SpellRating; 11] = [
    rating(c"Remove Hunger", 95, REMOVE_HUNGER),
    rating(c"Detect Life", 85, DETECT_LIFE),
    rating(c"Herbal Curing", 95, HERBAL_CURING),
    rating(c"Resist Poison", 85, RESIST_POISON),
    rating(c"Turn Stone to Mud", 85, TURN_STONE_TO_MUD),
    rating(c"Sense Surroundings", 75, SENSE_SURROUNDINGS),
    rating(c"Cover Tracks", 25, COVER_TRACKS), // !FIX !TODO prep?
    rating(c"Create Arrows", 85, CREATE_ARROWS), // !FIX !TODO
    rating(c"Haste Self", 95, HASTE_SELF),
    rating(c"Decoy", 5, DECOY), // !FIX !TODO not sure what to do with this
    rating(c"Brand Ammunition", 95, BRAND_AMMUNITION),
];
static RATINGS_BLACKGUARD: [SpellRating; 15] = [
    rating(c"Seek Battle", 55, SEEK_BATTLE),
    rating(c"Berserk Strength", 95, BERSERK_STRENGTH),
    rating(c"Whirlwind Attack", 85, WHIRLWIND_ATTACK),
    rating(c"Shatter Stone", 95, SHATTER_STONE),
    rating(c"Leap into Battle", 65, LEAP_INTO_BATTLE),
    rating(c"Grim Purpose", 65, GRIM_PURPOSE),
    rating(c"Maim Foe", 75, MAIM_FOE),
    rating(c"Howl of the Damned", 55, HOWL_OF_THE_DAMNED),
    rating(c"Relentless Taunting", 5, RELENTLESS_TAUNTING), // seems to dangerous for borg right now
    rating(c"Venom", 55, VENOM),
    rating(c"Werewolf Form", 5, WEREWOLF_FORM), // !FIX !TODO shapechange
    rating(c"Bloodlust", 5, BLOODLUST), // seems to dangerous for borg right now
    rating(c"Unholy Reprieve", 95, UNHOLY_REPRIEVE),
    rating(c"Forceful Blow", 5, FORCEFUL_BLOW), // !FIX !TODO need to code this
    rating(c"Quake", 95, QUAKE),
];

/// Get the stat used for casting spells (`borg_spell_stat`).
///
/// Assumes the first spell determines the realm thus stat for all spells
pub fn borg_spell_stat() -> i32 {
    if borg_can_cast() {
        // C: `&books[0].spells[0]` is never NULL
        return engine::class_book_spell_realm_stat(0, 0);
    }

    -1
}

/// Does this player cast spells (`borg_can_cast`)
pub fn borg_can_cast() -> bool {
    engine::class_total_spells() != 0
}

/// Does this player mostly cast spells (`borg_primarily_caster`).
/// HACK: Rather than hard code classes, assume any class with
/// more than three books is primarily casting
/// !FIX !TODO consider adding is_primary_caster to class struct
pub fn borg_primarily_caster() -> bool {
    engine::class_num_books() > 3
}

/// Get the level at which Heroism (spell) grants Heroism (effect)
/// (`borg_heroism_level`)
pub fn borg_heroism_level(st: &BorgState) -> i32 {
    if st.borg.tr(BI_CLASS) == CLASS_PRIEST {
        return 20;
    }
    if st.borg.tr(BI_CLASS) == CLASS_PALADIN {
        return 15;
    }
    99
}

/// Find the index in the books array given the books sval
/// (`borg_get_book_num`)
pub fn borg_get_book_num(sval: i32) -> i32 {
    if !borg_can_cast() {
        return -1;
    }

    let mut book_num = 0;
    while book_num < engine::class_num_books() {
        if engine::class_book(book_num).sval == sval {
            return book_num;
        }
        book_num += 1;
    }
    -1
}

/// Is this a dungeon book (not a basic book) (`borg_is_dungeon_book`)
pub fn borg_is_dungeon_book(tval: i32, sval: i32) -> bool {
    match tval as u32 {
        TV_PRAYER_BOOK | TV_MAGIC_BOOK | TV_NATURE_BOOK | TV_SHADOW_BOOK | TV_OTHER_BOOK => {}
        _ => return false,
    }

    // keep track of if this is a book from the dungeon
    let mut i = 0;
    while i < engine::class_num_books() {
        let book = engine::class_book(i);
        if tval == book.tval && sval == book.sval && book.dungeon {
            return true;
        }
        i += 1;
    }

    false
}

/// Find the magic structure given a book/entry (`borg_get_spell_entry`):
/// its index in `st.magic.magics`.
pub fn borg_get_spell_entry(st: &BorgState, book: i32, entry: i32) -> Option<usize> {
    let mut entry_in_book = 0;

    let mut spell_num = 0;
    while spell_num < engine::class_total_spells() {
        if st.magic.magics[spell_num as usize].book == book {
            if entry_in_book == entry {
                return Some(spell_num as usize);
            }
            entry_in_book += 1;
        }
        spell_num += 1;
    }
    None
}

/// Find the spell index for a given spell (`borg_get_spell_number`)
fn borg_get_spell_number(st: &BorgState, spell: borg_spells) -> i32 {
    // The borg must be able to "cast" spells
    if st.magic.magics.is_empty() {
        return -1;
    }

    let total_spells = engine::class_total_spells();
    let mut spell_num = 0;
    while spell_num < total_spells {
        if st.magic.magics[spell_num as usize].spell_enum == spell {
            return spell_num;
        }
        spell_num += 1;
    }

    -1
}

/// Find the power (cost in sp) value for a given spell
/// (`borg_get_spell_power`)
pub fn borg_get_spell_power(st: &BorgState, spell: borg_spells) -> i32 {
    let spell_num = borg_get_spell_number(st, spell);
    if spell_num < 0 {
        return -1;
    }

    let as_ = &st.magic.magics[spell_num as usize];

    i32::from(as_.power)
}

/// Determine if borg can cast a given spell (when fully rested)
/// (`borg_spell_legal`)
pub fn borg_spell_legal(st: &BorgState, spell: borg_spells) -> bool {
    let spell_num = borg_get_spell_number(st, spell);
    if spell_num < 0 {
        return false;
    }

    let as_ = &st.magic.magics[spell_num as usize];

    // The book must be possessed
    if st.borg.book_idx[as_.book as usize] < 0 {
        return false;
    }

    // The spell must be "known"
    if as_.status < BORG_MAGIC_TEST {
        return false;
    }

    // The spell must be affordable (when rested)
    if i32::from(as_.power) > st.borg.tr(BI_MAXSP) {
        return false;
    }

    // Success
    true
}

/// Check a spell for a given effect (`borg_spell_has_effect`)
fn borg_spell_has_effect(spell_num: i32, effect: u16) -> bool {
    engine::spell_effect_indices(spell_num).contains(&effect)
}

/// Determine if borg can cast a given spell (right now) (`borg_spell_okay`)
pub fn borg_spell_okay(st: &BorgState, spell: borg_spells) -> bool {
    let mut reserve_mana = 0;

    let spell_num = borg_get_spell_number(st, spell);
    if spell_num < 0 {
        return false;
    }

    let as_ = &st.magic.magics[spell_num as usize];

    // Dark
    if engine::no_light() {
        return false;
    }

    // Define reserve_mana for each class
    match st.borg.tr(BI_CLASS) {
        CLASS_MAGE => reserve_mana = 6,
        CLASS_RANGER => reserve_mana = 22,
        CLASS_ROGUE => reserve_mana = 20,
        CLASS_NECROMANCER => reserve_mana = 10,
        CLASS_PRIEST => reserve_mana = 8,
        CLASS_PALADIN => reserve_mana = 20,
        CLASS_BLACKGUARD => reserve_mana = 0,
        _ => {}
    }

    // Low level spell casters should not worry about this
    if st.borg.tr(BI_CLEVEL) < 35 {
        reserve_mana = 0;
    }

    // Require ability (when rested)
    if !borg_spell_legal(st, spell) {
        return false;
    }

    // Blind/confused/amnesia
    if st.borg.tr(BI_ISBLIND) != 0 || st.borg.tr(BI_ISCONFUSED) != 0 {
        return false;
    }

    // The spell must be affordable (now)
    if i32::from(as_.power) > st.borg.tr(BI_CURSP) {
        return false;
    }

    // Do not cut into reserve mana (for final teleport)
    if st.borg.tr(BI_CURSP) - i32::from(as_.power) < reserve_mana {
        // nourishing spells okay
        if borg_spell_has_effect(spell_num, EF_NOURISH as u16) {
            return true;
        }

        // okay to run away
        if borg_spell_has_effect(spell_num, EF_TELEPORT as u16) {
            return true;
        }

        // Magic Missile OK
        if MAGIC_MISSILE == spell && st.borg.tr(BI_CDEPTH) <= 35 {
            return true;
        }

        // others are rejected
        return false;
    }

    // Success
    true
}

/// Fail rate on a spell (`borg_spell_fail_rate`)
pub fn borg_spell_fail_rate(st: &BorgState, spell: borg_spells) -> i32 {
    let spell_num = borg_get_spell_number(st, spell);
    if spell_num < 0 {
        return 100;
    }

    let as_ = &st.magic.magics[spell_num as usize];

    // Access the spell
    let mut chance = i32::from(as_.sfail);

    // Reduce failure rate by "effective" level adjustment
    chance -= 3 * (st.borg.tr(BI_CLEVEL) - i32::from(as_.level));

    // Reduce failure rate by stat adjustment
    chance -= st.borg.tr(BI_FAIL1);

    // Fear makes the failrate higher
    if st.borg.tr(BI_ISAFRAID) != 0 {
        chance += 20;
    }

    // Extract the minimum failure rate
    let mut minfail = st.borg.tr(BI_FAIL2);

    // Non mage characters never get too good
    if !engine::player_has(PF_ZERO_FAIL) && minfail < 5 {
        minfail = 5;
    }

    // Necromancers are punished by being on lit squares, whatever lights
    // them (their own light, permanent light, a monster's light)
    // FORK (0099 guard): just after a level change the Borg's position can
    // still be on the previous, larger level
    if st.borg.tr(BI_CLASS) == CLASS_NECROMANCER
        && engine::square_in_bounds(st.borg.c.x, st.borg.c.y)
        && engine::square_islit(st.borg.c.x, st.borg.c.y)
    {
        chance += 25;
    }

    // Minimum failure rate and max
    if chance < minfail {
        chance = minfail;
    }
    if chance > 50 {
        chance = 50;
    }

    // Stunning makes spells harder
    if st.borg.tr(BI_ISHEAVYSTUN) != 0 {
        chance += 25;
    }
    if st.borg.tr(BI_ISSTUN) != 0 {
        chance += 15;
    }

    // Amnesia makes it harder
    if st.borg.tr(BI_ISFORGET) != 0 {
        chance *= 2;
    }

    // Always a 5 percent chance of working
    if chance > 95 {
        chance = 95;
    }

    // Return the chance
    chance
}

/// Same as `borg_spell_okay` with a fail % check (`borg_spell_okay_fail`)
pub fn borg_spell_okay_fail(st: &BorgState, spell: borg_spells, allow_fail: i32) -> bool {
    if borg_spell_fail_rate(st, spell) > allow_fail {
        return false;
    }
    borg_spell_okay(st, spell)
}

/// Same as `borg_spell` with a fail % check (`borg_spell_fail`)
pub fn borg_spell_fail(st: &mut BorgState, spell: borg_spells, allow_fail: i32) -> bool {
    if borg_spell_fail_rate(st, spell) > allow_fail {
        return false;
    }
    borg_spell(st, spell)
}

/// Same as `borg_spell_legal` with a fail % check (`borg_spell_legal_fail`)
pub fn borg_spell_legal_fail(st: &BorgState, spell: borg_spells, allow_fail: i32) -> bool {
    if borg_spell_fail_rate(st, spell) > allow_fail {
        return false;
    }
    borg_spell_legal(st, spell)
}

/// `all_letters_nohjkl[i]` as a key (a C `char` widened to `keycode_t`).
fn menu_letter(i: i32) -> keycode_t {
    engine::all_letters_nohjkl(i) as i8 as keycode_t
}

/// Attempt to cast a spell (`borg_spell`)
pub fn borg_spell(st: &mut BorgState, spell: borg_spells) -> bool {
    let spell_num = borg_get_spell_number(st, spell);
    if spell_num < 0 {
        return false;
    }
    let spell_num = spell_num as usize;

    // Require ability (right now)
    if !borg_spell_okay(st, spell) {
        return false;
    }

    let as_ = st.magic.magics[spell_num];

    // Look for the book
    let i = i32::from(st.borg.book_idx[as_.book as usize]);

    // Paranoia
    if i < 0 {
        return false;
    }

    // Debugging Info
    let mut note = b"# Casting ".to_vec();
    note.extend_from_slice(as_.name.to_bytes());
    note.extend_from_slice(format!(" ({},{}).", i, as_.book_offset).as_bytes());
    borg_note_bytes(st, &note);

    // Cast a spell
    borg_keypress(st, keycode_t::from(b'm'));
    borg_keypress(st, menu_letter(i));
    borg_keypress(st, menu_letter(as_.book_offset));

    // increment the spell counter
    st.magic.magics[spell_num].times += 1;

    // Success
    true
}

/// Cheat the "spell" info for a single book (`borg_cheat_spell`)
fn borg_cheat_spell(st: &mut BorgState, book_num: i32) {
    let book = engine::class_book(book_num);
    for spell_num in 0..book.num_spells {
        let cspell = engine::class_book_spell(book_num, spell_num);
        let flags = i32::from(engine::player_spell_flags(cspell.sidx));
        let clevel = st.borg.tr(BI_CLEVEL);
        let as_ = &mut st.magic.magics[cspell.sidx as usize];

        // Note "forgotten" spells
        if flags & PY_SPELL_FORGOTTEN != 0 {
            // Forgotten
            as_.status = BORG_MAGIC_LOST;
        }
        // Note "difficult" spells
        else if clevel < i32::from(as_.level) {
            // Unknown
            as_.status = BORG_MAGIC_HIGH;
        }
        // Note "Unknown" spells
        else if flags & PY_SPELL_LEARNED == 0 {
            // UnKnown
            as_.status = BORG_MAGIC_OKAY;
        }
        // Note "untried" spells
        else if flags & PY_SPELL_WORKED == 0 {
            // Untried
            as_.status = BORG_MAGIC_TEST;
        }
        // Note "known" spells
        else {
            // Known
            as_.status = BORG_MAGIC_KNOW;
        }
    }
}

/// `(borg_items[i].tval, borg_items[i].sval)` for a pack slot.
fn item_tval_sval(st: &BorgState, i: i32) -> (u8, u8) {
    assert!(i >= 0 && i < i32::from(engine::z_info().pack_size));
    let it = &st.items.items[i as usize];
    (it.tval, it.sval)
}

/// Cheat the "spell" info (`borg_cheat_spells`)
pub fn borg_cheat_spells(st: &mut BorgState) {
    // Assume no books
    st.borg.book_idx.fill(-1);

    // Scan the pack
    for i in 0..i32::from(engine::z_info().pack_size) {
        let (tval, sval) = item_tval_sval(st, i);

        let mut book_num = 0;
        while book_num < engine::class_num_books() {
            let book = engine::class_book(book_num);
            if i32::from(tval) == book.tval && i32::from(sval) == book.sval {
                // Note book locations
                st.borg.book_idx[book_num as usize] = i as i16;
                break;
            }
            book_num += 1;
        }
    }

    // only browse spells if casting is possible
    if !borg_can_cast() {
        return;
    }

    // XXX XXX XXX Dark

    for book_idx in 0..8 {
        // Look for the book
        let i = st.borg.book_idx[book_idx as usize];

        // Cheat the "spell" screens (all of them)
        if i >= 0 {
            // Cheat that page
            borg_cheat_spell(st, book_idx);
        }
    }
}

/// Get the offset in the book this spell is so you can cast it (book)
/// (offset) (`borg_get_book_offset`)
fn borg_get_book_offset(index: i32) -> i32 {
    let mut book = 0;
    let mut count = 0;

    // Check index validity
    if index < 0 || index >= engine::class_total_spells() {
        return -1;
    }

    // Find the book, count the spells in previous books
    while count + engine::class_book(book).num_spells - 1 < index {
        count += engine::class_book(book).num_spells;
        book += 1;
    }

    // Find the spell
    index - count
}

/// Initialize the spell data (`borg_init_spell`)
fn borg_init_spell(st: &mut BorgState, spell_num: i32) {
    let cspell = engine::spell_by_index(spell_num);
    let ratings = st.magic.ratings.expect("borg_spell_ratings set");
    // C reads past the table if the class has more spells than it has rows
    let r = ratings[spell_num as usize];
    if cspell.name != r.name.to_bytes() {
        let mut note = b"**STARTUP FAILURE** spell definition mismatch. <".to_vec();
        note.extend_from_slice(cspell.name);
        note.extend_from_slice(b"> not the same as <");
        note.extend_from_slice(r.name.to_bytes());
        note.push(b'>');
        borg_note_bytes(st, &note);
        st.init.init_failure = true;
        return;
    }
    // C dereferences `cspell->effect` (never NULL for a class spell)
    let effect_index = engine::spell_effect_indices(spell_num)[0];
    let book_offset = borg_get_book_offset(cspell.sidx);
    let status = engine::spell_okay_to_cast(spell_num);
    let spell = &mut st.magic.magics[spell_num as usize];
    spell.rating = r.rating;
    spell.name = r.name;
    spell.spell_enum = r.spell_enum;
    spell.level = cspell.slevel as u8;
    spell.book_offset = book_offset;
    spell.effect_index = effect_index;
    spell.power = cspell.smana as u8;
    spell.sfail = cspell.sfail as u8;
    // a bool: 0 or 1 (BORG_MAGIC_LOST) until borg_cheat_spells runs
    spell.status = u8::from(status);
    spell.times = 0;
    spell.book = cspell.bidx;
}

/// Prepare a book (`borg_prepare_book_info`)
pub fn borg_prepare_book_info(st: &mut BorgState) {
    let ratings: &'static [SpellRating] = match engine::class_cidx() as i32 {
        CLASS_MAGE => &RATINGS_MAGE,
        CLASS_DRUID => &RATINGS_DRUID,
        CLASS_PRIEST => &RATINGS_PRIEST,
        CLASS_NECROMANCER => &RATINGS_NECROMANCER,
        CLASS_PALADIN => &RATINGS_PALADIN,
        CLASS_ROGUE => &RATINGS_ROGUE,
        CLASS_RANGER => &RATINGS_RANGER,
        CLASS_BLACKGUARD => &RATINGS_BLACKGUARD,
        _ => {
            st.magic.ratings = None;
            return;
        }
    };
    st.magic.ratings = Some(ratings);

    let total_spells = engine::class_total_spells();
    st.magic.magics = vec![BorgMagic::default(); total_spells as usize];

    for spell in 0..total_spells {
        borg_init_spell(st, spell);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rating_tables_match_the_c_source() {
        let all: [&[SpellRating]; 8] = [
            &RATINGS_MAGE,
            &RATINGS_DRUID,
            &RATINGS_PRIEST,
            &RATINGS_NECROMANCER,
            &RATINGS_PALADIN,
            &RATINGS_ROGUE,
            &RATINGS_RANGER,
            &RATINGS_BLACKGUARD,
        ];
        let lens: Vec<usize> = all.iter().map(|t| t.len()).collect();
        assert_eq!(lens, [30, 27, 28, 26, 16, 10, 11, 15]);
        // the C literal "Spear of Orom\xC3\xab"
        assert_eq!(RATINGS_PRIEST[26].name.to_bytes(), b"Spear of Orom\xC3\xab");
        assert_eq!(RATINGS_PRIEST[26].spell_enum, SPEAR_OF_OROME);
        assert_eq!(RATINGS_MAGE[0].spell_enum, MAGIC_MISSILE);
        assert_eq!(RATINGS_BLACKGUARD[14].spell_enum, QUAKE);
    }
}
