//! Port of `borg-trait-swap.c`: the attributes of the swap weapon and the
//! swap armour.
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
//!
//! A swap is an item carried in the pack to be worn when needed (a digger,
//! or past dungeon level 50 a weapon or an armour with resists the worn
//! gear lacks). `borg_notice_weapon_swap` and `borg_notice_armour_swap`
//! pick the best one of each and note its flags; the C globals they write
//! are `TraitSwapState` (`st.trait_swap`).
//!
//! C's `int32_t v` sums mix `int` and `long` products and truncate the sum
//! to 32 bits after every step. Truncation commutes with addition and
//! multiplication, so the sums are `i32` wrapping arithmetic here.

use crate::borg_inventory::borg_wield_slot;
use crate::borg_item::inven_wield;
use crate::borg_magic::borg_spell_legal_fail;
use crate::engine;
use crate::borg_headers::borg_item_h::{
    borg_item, BORG_CURSE_ANTI_TELEPORTATION, BORG_CURSE_DEMON_SUMMON, BORG_CURSE_DRAGON_SUMMON,
    BORG_CURSE_HALLUCINATION, BORG_CURSE_PARALYSIS, BORG_CURSE_POISON, BORG_CURSE_SIREN,
    BORG_CURSE_STONE, BORG_CURSE_TELEPORTATION, BORG_CURSE_TREACHEROUS_WEAPON,
    BORG_CURSE_UNDEAD_SUMMON, BORG_CURSE_UNKNOWN,
};
use crate::borg_headers::borg_magic_h::{ENCHANT_ARMOUR, ENCHANT_WEAPON, TURN_STONE_TO_MUD};
use crate::borg_headers::borg_trait_h::{
    BI_AENCH_SARM, BI_AENCH_SWEP, BI_BLOWS, BI_ESP, BI_FEATH, BI_FRACT, BI_GOLD, BI_HLIFE,
    BI_IACID, BI_ICOLD, BI_IELEC, BI_IFIRE, BI_LIGHT, BI_MAXCLEVEL, BI_MAXDEPTH, BI_RACID,
    BI_RBLIND, BI_RCOLD, BI_RCONF, BI_RDARK, BI_RDIS, BI_REG, BI_RELEC, BI_RFEAR, BI_RFIRE,
    BI_RKAOS, BI_RLITE, BI_RNTHR, BI_RNXUS, BI_RPOIS, BI_RSHRD, BI_RSND, BI_SDIG, BI_SINV,
    BI_TODAM, BI_TOHIT, BI_WB_ACID, BI_WB_COLD, BI_WB_ELEC, BI_WB_FIRE, BI_WB_POIS,
    BI_WK_DRAGON, BI_WS_ANIMAL, BI_WS_DEMON, BI_WS_DRAGON, BI_WS_EVIL, BI_WS_GIANT, BI_WS_ORC,
    BI_WS_TROLL, BI_WS_UNDEAD,
};
use crate::headers::list_elements_h::{
    ELEM_ACID, ELEM_CHAOS, ELEM_COLD, ELEM_DARK, ELEM_DISEN, ELEM_ELEC, ELEM_FIRE, ELEM_LIGHT,
    ELEM_NETHER, ELEM_NEXUS, ELEM_POIS, ELEM_SHARD, ELEM_SOUND,
};
use crate::headers::list_mon_race_flags_h::{
    RF_ANIMAL, RF_DEMON, RF_DRAGON, RF_EVIL, RF_GIANT, RF_ORC, RF_TROLL, RF_UNDEAD,
};
use crate::headers::list_object_flags_h::{
    OF_AGGRAVATE, OF_DIG_1, OF_DIG_2, OF_DIG_3, OF_FEATHER, OF_FREE_ACT, OF_HOLD_LIFE, OF_IMPACT,
    OF_LIGHT_2, OF_LIGHT_3, OF_PROT_BLIND, OF_PROT_CONF, OF_REGEN, OF_SEE_INVIS,
    OF_SLOW_DIGEST, OF_TELEPATHY,
};
use crate::headers::list_object_modifiers_h::OBJ_MOD_INFRA;
use crate::headers::list_tvals_h::{
    TV_AMULET, TV_BOOTS, TV_CLOAK, TV_CROWN, TV_DIGGING, TV_DRAG_ARMOR, TV_GLOVES, TV_HAFTED,
    TV_HARD_ARMOR, TV_HELM, TV_LIGHT, TV_POLEARM, TV_RING, TV_SHIELD, TV_SOFT_ARMOR, TV_SWORD,
};
use crate::state::BorgState;

/// The flags of one swap: the `weapon_swap_*` or the `armour_swap_*`
/// globals (C names without the prefix), and `decurse_weapon_swap` /
/// `decurse_armour_swap` as `decurse`. C declares them all `uint8_t`
/// (the decurse flags `bool`); the ones only ever set to `true`/`false`
/// are `bool` here.
#[derive(Clone, Copy, Default, PartialEq, Eq, Debug)]
pub struct SwapFlags {
    pub slay_animal: u8,
    pub slay_evil: u8,
    pub slay_undead: u8,
    pub slay_demon: u8,
    pub slay_orc: u8,
    pub slay_troll: u8,
    pub slay_giant: u8,
    pub slay_dragon: u8,
    pub impact: bool,
    pub brand_acid: bool,
    pub brand_elec: bool,
    pub brand_fire: bool,
    pub brand_cold: bool,
    pub brand_pois: bool,
    /// `+=` of the item's infravision modifier (truncated to `uint8_t`)
    pub see_infra: u8,
    pub slow_digest: bool,
    pub aggravate: bool,
    pub bad_curse: bool,
    pub regenerate: bool,
    pub telepathy: bool,
    pub light: bool,
    pub see_invis: bool,
    pub ffall: bool,
    pub free_act: bool,
    pub hold_life: bool,
    pub immune_fire: bool,
    pub immune_acid: bool,
    pub immune_cold: bool,
    pub immune_elec: bool,
    pub resist_acid: bool,
    pub resist_elec: bool,
    pub resist_fire: bool,
    pub resist_cold: bool,
    pub resist_pois: bool,
    pub resist_conf: bool,
    pub resist_sound: bool,
    pub resist_light: bool,
    pub resist_dark: bool,
    pub resist_chaos: bool,
    pub resist_disen: bool,
    pub resist_shard: bool,
    pub resist_nexus: bool,
    pub resist_blind: bool,
    pub resist_neth: bool,
    /// never written in C, so always false (C bug 45)
    pub resist_fear: bool,
    /// `decurse_weapon_swap` / `decurse_armour_swap`: the swap is
    /// uncursable (C: "my swap is great, except its cursed")
    pub decurse: bool,
}

impl SwapFlags {
    /// C's "Clear all the swap weapon flags" block: every flag but
    /// `resist_fear`, which C never writes.
    fn clear(&mut self) {
        *self = SwapFlags { resist_fear: self.resist_fear, ..SwapFlags::default() };
    }

    /// Note the flags of `item` (the slays, brands, infravision, flags,
    /// curses, immunities, resistances and uncursability block, copied
    /// four times in C).
    fn note(&mut self, item: &borg_item) {
        self.slay_animal = item.slays[RF_ANIMAL as usize] as u8;
        self.slay_evil = item.slays[RF_EVIL as usize] as u8;
        self.slay_undead = item.slays[RF_UNDEAD as usize] as u8;
        self.slay_demon = item.slays[RF_DEMON as usize] as u8;
        self.slay_orc = item.slays[RF_ORC as usize] as u8;
        self.slay_troll = item.slays[RF_TROLL as usize] as u8;
        self.slay_giant = item.slays[RF_GIANT as usize] as u8;
        self.slay_dragon = item.slays[RF_DRAGON as usize] as u8;

        if of_has(item, OF_IMPACT) {
            self.impact = true;
        }
        if item.brands[ELEM_ACID as usize] {
            self.brand_acid = true;
        }
        if item.brands[ELEM_ELEC as usize] {
            self.brand_elec = true;
        }
        if item.brands[ELEM_FIRE as usize] {
            self.brand_fire = true;
        }
        if item.brands[ELEM_COLD as usize] {
            self.brand_cold = true;
        }
        if item.brands[ELEM_POIS as usize] {
            self.brand_pois = true;
        }

        /* Affect infravision */
        self.see_infra =
            i32::from(self.see_infra).wrapping_add(i32::from(item.modifiers[OBJ_MOD_INFRA as usize]))
                as u8;

        /* Various flags */
        if of_has(item, OF_SLOW_DIGEST) {
            self.slow_digest = true;
        }
        if of_has(item, OF_AGGRAVATE) {
            self.aggravate = true;
        }
        if of_has(item, OF_REGEN) {
            self.regenerate = true;
        }
        if of_has(item, OF_TELEPATHY) {
            self.telepathy = true;
        }
        if of_has(item, OF_LIGHT_2) || of_has(item, OF_LIGHT_3) {
            self.light = true;
        }
        if of_has(item, OF_SEE_INVIS) {
            self.see_invis = true;
        }
        if of_has(item, OF_FEATHER) {
            self.ffall = true;
        }
        if of_has(item, OF_FREE_ACT) {
            self.free_act = true;
        }
        if of_has(item, OF_HOLD_LIFE) {
            self.hold_life = true;
        }
        if of_has(item, OF_PROT_CONF) {
            self.resist_conf = true;
        }
        if of_has(item, OF_PROT_BLIND) {
            self.resist_blind = true;
        }

        /* curses */
        if borg_has_bad_curse(item) {
            self.bad_curse = true;
        }

        /* Immunity flags */
        let res = |elem: u32| item.el_info[elem as usize].res_level;
        if res(ELEM_FIRE) == 3 {
            self.immune_fire = true;
        }
        if res(ELEM_ACID) == 3 {
            self.immune_acid = true;
        }
        if res(ELEM_COLD) == 3 {
            self.immune_cold = true;
        }
        if res(ELEM_ELEC) == 3 {
            self.immune_elec = true;
        }

        /* Resistance flags */
        if res(ELEM_ACID) > 0 {
            self.resist_acid = true;
        }
        if res(ELEM_ELEC) > 0 {
            self.resist_elec = true;
        }
        if res(ELEM_FIRE) > 0 {
            self.resist_fire = true;
        }
        if res(ELEM_COLD) > 0 {
            self.resist_cold = true;
        }
        if res(ELEM_POIS) > 0 {
            self.resist_pois = true;
        }
        if res(ELEM_SOUND) > 0 {
            self.resist_sound = true;
        }
        if res(ELEM_LIGHT) > 0 {
            self.resist_light = true;
        }
        if res(ELEM_DARK) > 0 {
            self.resist_dark = true;
        }
        if res(ELEM_CHAOS) > 0 {
            self.resist_chaos = true;
        }
        if res(ELEM_DISEN) > 0 {
            self.resist_disen = true;
        }
        if res(ELEM_SHARD) > 0 {
            self.resist_shard = true;
        }
        if res(ELEM_NEXUS) > 0 {
            self.resist_nexus = true;
        }
        if res(ELEM_NETHER) > 0 {
            self.resist_neth = true;
        }
        if item.uncursable {
            self.decurse = true;
        }
    }
}

/// The globals of borg-trait-swap.c.
#[derive(Clone, Default, PartialEq, Eq, Debug)]
pub struct TraitSwapState {
    /// `weapon_swap`: pack slot of the swap weapon + 1 (0 = none)
    pub weapon_swap: i32,
    /// `armour_swap`: pack slot of the swap armour + 1 (0 = none)
    pub armour_swap: i32,
    /// `weapon_swap_value`
    pub weapon_swap_value: i32,
    /// `armour_swap_value`
    pub armour_swap_value: i32,
    /// `enchant_weapon_swap_to_h`: to-hit enchantment the swap weapon wants
    pub enchant_weapon_swap_to_h: i32,
    /// `enchant_weapon_swap_to_d`: to-dam enchantment the swap weapon wants
    pub enchant_weapon_swap_to_d: i32,
    /// `enchant_armour_swap_to_a`: to-AC enchantment the swap armour wants
    pub enchant_armour_swap_to_a: i32,
    /// `weapon_swap_digger`: digging power of the last digger looked at
    pub weapon_swap_digger: i16,
    /// `weapon_swap_*` and `decurse_weapon_swap`
    pub weapon: SwapFlags,
    /// `armour_swap_*` and `decurse_armour_swap`
    pub armour: SwapFlags,
}

/// `of_has(item->flags, flag)`
fn of_has(item: &borg_item, flag: u32) -> bool {
    engine::flag_has(&item.flags, flag as i32)
}

/// `borg.trait[bi]`
fn tr(st: &BorgState, bi: u32) -> i32 {
    st.borg.trait_[bi as usize]
}

/// For swap items all curses are lumped together as "bad".
fn borg_has_bad_curse(item: &borg_item) -> bool {
    [
        BORG_CURSE_TELEPORTATION,
        BORG_CURSE_POISON,
        BORG_CURSE_SIREN,
        BORG_CURSE_HALLUCINATION,
        BORG_CURSE_PARALYSIS,
        BORG_CURSE_DEMON_SUMMON,
        BORG_CURSE_DRAGON_SUMMON,
        BORG_CURSE_UNDEAD_SUMMON,
        BORG_CURSE_STONE,
        BORG_CURSE_ANTI_TELEPORTATION,
        BORG_CURSE_TREACHEROUS_WEAPON,
        BORG_CURSE_UNKNOWN,
    ]
    .iter()
    .any(|&c| item.curses[c as usize])
}

/// "Skip un-aware items", "Skip empty items", "Skip non-wearable items".
fn wearable(item: &borg_item) -> bool {
    item.iqty != 0 && item.aware && borg_wield_slot(item) != -1
}

/// The weapon tvals of the switch in `borg_notice_weapon_swap`.
fn is_weapon(tval: u8) -> bool {
    matches!(u32::from(tval), TV_HAFTED | TV_POLEARM | TV_SWORD | TV_DIGGING)
}

/// The armour tvals of the switch in `borg_notice_armour_swap` (no
/// light).
fn is_armour(tval: u8) -> bool {
    matches!(
        u32::from(tval),
        TV_RING
            | TV_AMULET
            | TV_BOOTS
            | TV_HELM
            | TV_CROWN
            | TV_SHIELD
            | TV_GLOVES
            | TV_CLOAK
            | TV_SOFT_ARMOR
            | TV_HARD_ARMOR
            | TV_DRAG_ARMOR
    )
}

/// The rewards of a weapon/armour swap that are the same in both copies
/// (`v` after the damage, to-hit and to-dam rewards): the slays and
/// brands, the abilities, the immunities and the resists, in C's order.
/// `armour` selects the armour copy's differences: its slay order and no
/// fear resist.
fn value_flags(st: &BorgState, f: &SwapFlags, damage: i32, mut v: i32, armour: bool) -> i32 {
    let t = |bi| tr(st, bi);
    let blows = t(BI_BLOWS);
    let add = |v: &mut i32, x: i32| *v = v.wrapping_add(x);

    /* assume 2x base damage for x% of creatures */
    let mut dam = damage.wrapping_mul(2).wrapping_mul(blows);
    /* reward SAnimal if no electric brand */
    if t(BI_WS_ANIMAL) == 0 && t(BI_WB_ELEC) == 0 && f.slay_animal != 0 {
        add(&mut v, dam.wrapping_mul(i32::from(f.slay_animal)) / 2);
    }
    if t(BI_WS_EVIL) == 0 && f.slay_evil != 0 {
        add(&mut v, dam.wrapping_mul(i32::from(f.slay_evil)) / 2);
    }

    /* assume 3x base damage for x% of creatures */
    dam = damage.wrapping_mul(3).wrapping_mul(blows);

    if !armour {
        /* half of the reward now for SOrc and STroll*/
        if t(BI_WS_ORC) == 0 && f.slay_orc != 0 {
            add(&mut v, dam.wrapping_mul(i32::from(f.slay_orc)) / 2);
        }
        if t(BI_WS_TROLL) == 0 && f.slay_troll != 0 {
            add(&mut v, dam.wrapping_mul(i32::from(f.slay_troll)) / 2);
        }
    }

    if t(BI_WS_UNDEAD) == 0 && f.slay_undead != 0 {
        add(&mut v, dam.wrapping_mul(i32::from(f.slay_undead)) / 2);
    }
    if t(BI_WS_DEMON) == 0 && f.slay_demon != 0 {
        add(&mut v, dam.wrapping_mul(i32::from(f.slay_demon)) / 2);
    }
    if t(BI_WS_GIANT) == 0 && f.slay_giant != 0 {
        add(&mut v, dam.wrapping_mul(i32::from(f.slay_giant)) / 2);
    }
    if t(BI_WS_DRAGON) == 0 && t(BI_WK_DRAGON) == 0 && f.slay_dragon != 0 {
        add(&mut v, dam.wrapping_mul(i32::from(f.slay_dragon)) / 2);
    }
    if t(BI_WB_ACID) == 0 && f.brand_acid {
        add(&mut v, dam.wrapping_mul(4) / 2);
    }
    if t(BI_WB_ELEC) == 0 && f.brand_elec {
        add(&mut v, dam.wrapping_mul(5) / 2);
    }
    if t(BI_WB_FIRE) == 0 && f.brand_fire {
        add(&mut v, dam.wrapping_mul(3) / 2);
    }
    if t(BI_WB_COLD) == 0 && f.brand_cold {
        add(&mut v, dam.wrapping_mul(3) / 2);
    }
    if t(BI_WB_POIS) == 0 && f.brand_pois {
        add(&mut v, dam.wrapping_mul(3) / 2);
    }
    if armour {
        /* SOrc and STroll get 1/2 reward now */
        if t(BI_WS_ORC) == 0 && f.slay_orc != 0 {
            add(&mut v, dam.wrapping_mul(i32::from(f.slay_orc)) / 2);
        }
        if t(BI_WS_TROLL) == 0 && f.slay_troll != 0 {
            add(&mut v, dam.wrapping_mul(i32::from(f.slay_troll)) / 2);
        }
    }
    /* Orcs and Trolls get the second half of the reward if SEvil is */
    /* not possessed. */
    if t(BI_WS_ORC) == 0 && t(BI_WS_EVIL) == 0 && f.slay_orc != 0 {
        add(&mut v, dam.wrapping_mul(i32::from(f.slay_orc)) / 2);
    }
    if t(BI_WS_TROLL) == 0 && t(BI_WS_EVIL) == 0 && f.slay_troll != 0 {
        add(&mut v, dam.wrapping_mul(i32::from(f.slay_troll)) / 2);
    }
    v
}

/// The ability, immunity and resist rewards (both copies; the weapon's
/// digger reward comes before, the fear resist only in the weapon copy).
fn value_abilities(st: &BorgState, f: &SwapFlags, mut v: i32, fear: bool) -> i32 {
    let t = |bi| tr(st, bi);
    let mut add = |c: bool, x: i32| {
        if c {
            v = v.wrapping_add(x);
        }
    };

    /* Other Skills */
    add(t(BI_SDIG) == 0 && f.slow_digest, 10);
    add(f.aggravate, -8000);
    add(f.bad_curse, -100000);
    add(f.decurse, -5000);
    add(t(BI_REG) == 0 && f.regenerate, 2000);
    add(t(BI_ESP) == 0 && f.telepathy, 5000);
    add(t(BI_LIGHT) == 0 && f.light, 2000);
    add(t(BI_SINV) == 0 && f.see_invis, 50000);
    add(t(BI_FEATH) == 0 && f.ffall, 10);
    add(t(BI_FRACT) == 0 && f.free_act, 10000);
    add(t(BI_HLIFE) == 0 && t(BI_MAXCLEVEL) < 50 && f.hold_life, 2000);
    add(t(BI_IFIRE) == 0 && f.immune_fire, 70000);
    add(t(BI_IACID) == 0 && f.immune_acid, 30000);
    add(t(BI_ICOLD) == 0 && f.immune_cold, 50000);
    add(t(BI_IELEC) == 0 && f.immune_elec, 25000);
    add(t(BI_RFIRE) == 0 && f.resist_fire, 8000);
    add(t(BI_RACID) == 0 && f.resist_acid, 6000);
    add(t(BI_RCOLD) == 0 && f.resist_cold, 4000);
    add(t(BI_RELEC) == 0 && f.resist_elec, 3000);
    /* extra bonus for getting all basic resist */
    add(f.resist_fire && f.resist_acid && f.resist_elec && f.resist_cold, 10000);
    add(t(BI_RPOIS) == 0 && f.resist_pois, 20000);
    add(t(BI_RCONF) == 0 && f.resist_conf, 5000);
    add(t(BI_RSND) == 0 && f.resist_sound, 2000);
    add(t(BI_RLITE) == 0 && f.resist_light, 800);
    add(t(BI_RDARK) == 0 && f.resist_dark, 800);
    add(t(BI_RKAOS) == 0 && f.resist_chaos, 8000);
    add(t(BI_RDIS) == 0 && f.resist_disen, 5000);
    add(t(BI_RSHRD) == 0 && f.resist_shard, 100);
    add(t(BI_RNXUS) == 0 && f.resist_nexus, 100);
    add(t(BI_RBLIND) == 0 && f.resist_blind, 5000);
    add(t(BI_RNTHR) == 0 && f.resist_neth, 5500);
    if fear {
        add(t(BI_RFEAR) == 0 && f.resist_fear, 5500);
    }

    /* Special concern if Tarrasque is alive */
    let icold = t(BI_ICOLD) == 0 && f.immune_cold;
    let ifire = t(BI_IFIRE) == 0 && f.immune_fire;
    if t(BI_MAXDEPTH) >= 75
        && (icold || ifire)
        && st.kills.race_death[st.borg.id.tarrasque as usize] == 0
    {
        add(icold, 90000);
        add(ifire, 90000);
    }
    v
}

/// `damage`, then `v` after the damage, to-hit and to-dam rewards (the
/// two copies differ in their factors: 25/75 weapon, 35/35 armour).
fn value_base(st: &BorgState, item: &borg_item, dmg_factor: i32, todam_factor: i32) -> (i32, i32) {
    /* calculating the value of the swap weapon. */
    let damage = (i32::from(item.dd) * i32::from(item.ds)).wrapping_mul(dmg_factor);

    let mut v: i32 = -1;
    /* Reward "damage" and increased blows per round*/
    v = v.wrapping_add(damage.wrapping_mul(tr(st, BI_BLOWS).wrapping_add(1)));
    /* Reward "bonus to hit" */
    v = v.wrapping_add(tr(st, BI_TOHIT).wrapping_add(i32::from(item.to_h)).wrapping_mul(100));
    /* Reward "bonus to dam" */
    v = v.wrapping_add(
        tr(st, BI_TODAM).wrapping_add(i32::from(item.to_d)).wrapping_mul(todam_factor),
    );
    (damage, v)
}

/// The value of pack item `i` as the swap weapon (C's weapon case after
/// the flags are noted).
fn weapon_value(st: &BorgState, item: &borg_item) -> i32 {
    let f = &st.trait_swap.weapon;
    let (damage, v) = value_base(st, item, 25, 75);
    let mut v = value_flags(st, f, damage, v, false);

    /* reward the Tunnel factor when low level */
    let d = st.trait_swap.weapon_swap_digger;
    if tr(st, BI_MAXDEPTH) <= 40 && tr(st, BI_MAXDEPTH) >= 25 && tr(st, BI_GOLD) < 100000 && d != 0
    {
        v = v.wrapping_add(i32::from(d).wrapping_mul(3500).wrapping_add(1000));
    }

    let mut v = value_abilities(st, f, v, true);

    /*  Mega-Hack -- resists (level 60) */
    let depth1 = tr(st, BI_MAXDEPTH).wrapping_add(1);
    if tr(st, BI_RNTHR) == 0 && depth1 >= 55 && f.resist_neth {
        v = v.wrapping_add(100000);
    }
    if tr(st, BI_RKAOS) == 0 && depth1 >= 60 && f.resist_chaos {
        v = v.wrapping_add(100000);
    }
    if tr(st, BI_RDIS) == 0 && depth1 >= 60 && f.resist_disen {
        v = v.wrapping_add(100000);
    }
    v
}

/// The value of an item as the swap armour (C's armour case after the
/// flags are noted). The high resists count only if the swap weapon
/// lacks them.
fn armour_value(st: &BorgState, item: &borg_item) -> i32 {
    let f = &st.trait_swap.armour;
    let w = &st.trait_swap.weapon;
    let (damage, v) = value_base(st, item, 35, 35);
    let v = value_flags(st, f, damage, v, true);
    let mut v = value_abilities(st, f, v, false);

    let depth1 = tr(st, BI_MAXDEPTH).wrapping_add(1);
    if tr(st, BI_RNTHR) == 0 && depth1 >= 55 && !w.resist_neth && f.resist_neth {
        v = v.wrapping_add(105000);
    }
    if tr(st, BI_RKAOS) == 0 && depth1 >= 60 && !w.resist_chaos && f.resist_chaos {
        v = v.wrapping_add(104000);
    }
    if tr(st, BI_RDIS) == 0 && depth1 >= 60 && !w.resist_disen && f.resist_disen {
        v = v.wrapping_add(100000);
    }
    v
}

/// Notice the player swap weapon.
///
/// Diggers are looked at at any depth, other weapons from max depth 50 on
/// and only if the Borg uses swaps. C bugs kept (43): a non-digger weapon
/// when swaps are off returns at once, leaving the flags of the last item
/// looked at and the old enchantment needs; `weapon_swap_digger` is not
/// cleared at the end.
pub fn borg_notice_weapon_swap(st: &mut BorgState) {
    let mut b_i: i32 = -1;
    let mut b_v: i32 = -1;

    st.trait_swap.weapon_swap = 0;
    st.trait_swap.weapon_swap_value = -1;

    /*** Process the inventory ***/
    for i in 0..inven_wield() {
        let item = &st.items.items[i as usize];

        if !wearable(item) {
            continue;
        }

        /* Don't carry swaps until dlevel 50.  They are heavy.
           Unless the item is a digger, then carry it */
        let digging = u32::from(item.tval) == TV_DIGGING;
        if tr(st, BI_MAXDEPTH) < 50 && !digging {
            continue;
        }

        /* borg option to not use swaps (again, except diggers) */
        if !crate::borg_trait::borg_uses_swaps(st) && !digging {
            return;
        }
        let item = st.items.items[i as usize];

        /* Clear all the swap weapon flags as I look at each one. */
        st.trait_swap.weapon_swap_digger = 0;
        st.trait_swap.weapon.clear();

        /* Analyze the item */
        if !is_weapon(item.tval) {
            continue;
        }

        /* Digging */
        if of_has(&item, OF_DIG_1) || of_has(&item, OF_DIG_2) || of_has(&item, OF_DIG_3) {
            /* Ignore worthless ones (including cursed): C's `break` out
             * of the switch */
            if item.value <= 0 || item.cursed {
                continue;
            }
            /* Don't notice digger if we can turn stone to mud,
             * or I am using one. */
            let wield = &st.items.items[inven_wield() as usize];
            if !borg_spell_legal_fail(st, TURN_STONE_TO_MUD, 40)
                && !of_has(wield, OF_DIG_1)
                && !of_has(wield, OF_DIG_2)
                && !of_has(wield, OF_DIG_3)
            {
                st.trait_swap.weapon_swap_digger = item.pval;
            }
        }

        st.trait_swap.weapon.note(&item);

        let v = weapon_value(st, &item);

        /* skip useless ones */
        if v <= 1000 {
            continue;
        }

        /* collect the best one */
        if v < b_v {
            continue;
        }

        /* track it */
        b_i = i;
        b_v = v;
    }

    /* Clear all the swap weapon flags. */
    st.trait_swap.weapon.clear();

    /* Assume no enchantment needed */
    st.trait_swap.enchant_weapon_swap_to_h = 0;
    st.trait_swap.enchant_weapon_swap_to_d = 0;

    if b_i == -1 {
        return;
    }

    /* mark the swap item and its value */
    st.trait_swap.weapon_swap_value = b_v;
    st.trait_swap.weapon_swap = b_i + 1;

    /*** Process the best inven item ***/
    let item = st.items.items[b_i as usize];
    let not_digger = u32::from(item.tval) != TV_DIGGING;
    let (to_h, to_d) = (i32::from(item.to_h), i32::from(item.to_d));

    /* Enchant swap weapons (to hit) */
    let target = if (borg_spell_legal_fail(st, ENCHANT_WEAPON, 65) || tr(st, BI_AENCH_SWEP) >= 1)
        && not_digger
    {
        Some(10)
    } else if not_digger {
        Some(8)
    } else {
        None
    };
    if let Some(target) = target {
        let ts = &mut st.trait_swap;
        if to_h < target {
            ts.enchant_weapon_swap_to_h += target - to_h;
        }
        /* Enchant my swap (to damage) */
        if to_d < target {
            ts.enchant_weapon_swap_to_d += target - to_d;
        }
    }

    st.trait_swap.weapon.note(&item);
}

/// Notice the player swap armour.
///
/// Lights are wearable but never valued (not in C's switch; gloves are,
/// since better-borg-c milestone5 patch 0045);
/// a cursed item counts only if it can be uncursed, and the One Ring
/// never.
pub fn borg_notice_armour_swap(st: &mut BorgState) {
    let mut b_i: i32 = -1;
    let mut b_v: i32 = 0;

    st.trait_swap.armour_swap = 0;
    st.trait_swap.armour_swap_value = -1;

    /* borg option to not use them */
    if !crate::borg_trait::borg_uses_swaps(st) {
        return;
    }

    /*** Process the inventory ***/
    for i in 0..inven_wield() {
        let item = st.items.items[i as usize];

        if !wearable(&item) {
            continue;
        }

        /* Dont carry swaps until dlevel 50.  They are heavy */
        if tr(st, BI_MAXDEPTH) < 50 {
            continue;
        }

        /* Skip it if it is not decursable */
        if item.cursed && !item.uncursable {
            continue;
        }

        /* One Ring is not a swap */
        if item.one_ring {
            continue;
        }

        /* Clear all the swap weapon flags as I look at each one. */
        st.trait_swap.armour.clear();

        /* Analyze the item */
        if !is_armour(item.tval) {
            continue;
        }

        /* as of 280, armours don't have slays but random artifacts might. */
        st.trait_swap.armour.note(&item);

        let v = armour_value(st, &item);

        /* skip useless ones */
        if v <= 1000 {
            continue;
        }

        /* collect the best one */
        if b_i >= 0 && v < b_v {
            continue;
        }

        /* track it */
        b_i = i;
        b_v = v;
        st.trait_swap.armour_swap_value = v;
        st.trait_swap.armour_swap = i + 1;
    }

    /* Clear all the swap weapon flags as I look at each one. */
    st.trait_swap.armour.clear();

    if b_i == -1 {
        return;
    }

    /*** Process the best inven item ***/
    let item = st.items.items[b_i as usize];
    st.trait_swap.armour.note(&item);

    st.trait_swap.enchant_armour_swap_to_a = 0;

    /* don't look for enchantment on non armours */
    if u32::from(item.tval) >= TV_LIGHT {
        return;
    }

    /* Enchant the swap equipment (armor) */
    let to_a = i32::from(item.to_a);
    let target = if borg_spell_legal_fail(st, ENCHANT_ARMOUR, 65) || tr(st, BI_AENCH_SARM) >= 1 {
        10
    } else {
        8
    };
    if to_a < target {
        st.trait_swap.enchant_armour_swap_to_a += target - to_a;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clear_keeps_resist_fear_only() {
        let mut f = SwapFlags {
            slay_evil: 3,
            see_infra: 7,
            resist_neth: true,
            decurse: true,
            resist_fear: true,
            ..SwapFlags::default()
        };
        f.clear();
        assert_eq!(f, SwapFlags { resist_fear: true, ..SwapFlags::default() });
    }

    #[test]
    fn tvals() {
        assert!(is_weapon(TV_DIGGING as u8) && is_weapon(TV_SWORD as u8));
        assert!(!is_weapon(TV_BOOTS as u8));
        assert!(is_armour(TV_RING as u8) && is_armour(TV_DRAG_ARMOR as u8));
        assert!(is_armour(TV_GLOVES as u8));
        assert!(!is_armour(TV_LIGHT as u8));
    }
}
