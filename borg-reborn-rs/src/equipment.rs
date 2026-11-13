//! Equipment valued by effect (T5.2, #126), not by shop price.
//!
//! A pack item replaces what the character wears only when the swap is
//! better on every effect the Borg can compute and strictly better on one
//! (a Pareto rule: no weights to declare):
//!
//! * **melee damage per turn**: [`crate::world::Character::melee_per_turn`]
//!   on a copy wearing the item, with the engine's blows, heavy-wield rule
//!   and known modifiers (`wearing` recomputes the stats in use);
//! * **armour class**: the known `ac + to_a` of the equipment
//!   (`GearBonuses::ac`), which the threat model reads through
//!   `adjust_dam_armor` and the monsters' hit chance;
//! * **mana**: for a caster, the armour weight over the class's
//!   `spell_weight` costs `(weight - allowance) / 10` maximum spell points
//!   (`player-calcs.c:calc_mana:1505-1531`; weapon, bow, rings, amulet and
//!   light do not count). A swap may not raise that penalty.
//!
//! What is deliberately not valued: a **launcher** is never a weapon here
//! (without ammunition and the shooting of #127 it adds nothing; the
//! readiness gate asks for ammunition with it, `readiness.rs`);
//! **jewellery** is worn only when fully known and only into an empty slot
//! (a second ring would prompt for which to replace), and only for the
//! effects above; resistances, slays and brands are not scored yet.
//! **Pack weight** does not change with a swap: the engine counts
//! equipment and pack together (`player->upkeep->total_weight`). **Reserves
//! outrank upgrades**: the town buys survival stock (`strategy.rs`,
//! `town_job`) and no equipment, so a purchase never displaces them.

use crate::knowledge::Knowledge;
use crate::observation::Item;
use crate::world::{tv, Character, World};

/// `EQUIP_*` slot types (`list-equip-slots.h`, numbered from `EQUIP_NONE`).
mod slot {
    pub const WEAPON: u16 = 1;
    pub const RING: u16 = 3;
    pub const AMULET: u16 = 4;
    pub const BODY_ARMOR: u16 = 6;
    pub const CLOAK: u16 = 7;
    pub const SHIELD: u16 = 8;
    pub const HAT: u16 = 9;
    pub const GLOVES: u16 = 10;
    pub const BOOTS: u16 = 11;
}

/// The slot type an item of this tval is worn in, for the kinds this
/// module values: the first slot of the item's type (`obj-gear.c:wield_slot:341`,
/// `slot_by_type`, which maps the tval through `tval_is_*`).
fn slot_type_for(tval: u8) -> Option<u16> {
    Some(match tval {
        t if tv::is_melee(t) => slot::WEAPON,
        tv::SOFT_ARMOR | tv::HARD_ARMOR | tv::DRAG_ARMOR => slot::BODY_ARMOR,
        tv::CLOAK => slot::CLOAK,
        tv::SHIELD => slot::SHIELD,
        tv::HELM | tv::CROWN => slot::HAT,
        tv::GLOVES => slot::GLOVES,
        tv::BOOTS => slot::BOOTS,
        tv::RING => slot::RING,
        tv::AMULET => slot::AMULET,
        _ => return None,
    })
}

/// The effects a swap is judged on.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Effects {
    pub melee: f64,
    pub ac: i32,
    pub mana_penalty: i32,
}

/// A pack item worth wearing, and what it changes.
#[derive(Clone, Debug, PartialEq)]
pub struct Swap {
    /// The pack index to wield.
    pub item: usize,
    pub before: Effects,
    pub after: Effects,
}

/// The effects of a character's equipment.
pub fn effects(c: &Character, k: &Knowledge) -> Effects {
    Effects {
        melee: c.melee_per_turn(k).unwrap_or(0.0),
        ac: c.gear.ac,
        mana_penalty: mana_penalty(c, k),
    }
}

/// `calc_mana`'s armour penalty in maximum spell points: 0 for a class
/// with no books.
pub fn mana_penalty(c: &Character, k: &Knowledge) -> i32 {
    let Some(class) = c.status.as_ref().and_then(|s| k.class(s.class)) else {
        return 0;
    };
    if class.books.is_empty() {
        return 0;
    }
    let weight: i32 = c
        .equipment
        .iter()
        .filter_map(|s| s.item.as_ref())
        .filter(|i| tv::is_armour(i.tval))
        .map(|i| i32::from(i.weight))
        .sum();
    ((weight - class.spell_weight) / 10).max(0)
}

/// Whether `after` is better than `before` on every effect and strictly on
/// one: more melee (by a tenth of a hit point, so rounding cannot flip a
/// swap back), more armour, and no larger mana penalty.
pub fn better(before: &Effects, after: &Effects) -> bool {
    let no_worse = after.melee + 1e-9 >= before.melee
        && after.ac >= before.ac
        && after.mana_penalty <= before.mana_penalty;
    let strictly = after.melee > before.melee + 0.1 || after.ac > before.ac;
    no_worse && strictly
}

/// Whether the Borg may wear this pack item at all: a known kind, and for
/// jewellery, fully known.
fn wearable(item: &Item) -> bool {
    if !item.aware {
        return false;
    }
    match item.tval {
        tv::RING | tv::AMULET => item.fully_known,
        _ => true,
    }
}

/// The best swap the pack offers, if any (the largest melee gain, then the
/// largest armour gain, then the lowest pack index).
pub fn best_swap(w: &World, k: &Knowledge) -> Option<Swap> {
    let c = &w.character;
    let before = effects(c, k);
    let mut best: Option<Swap> = None;
    for (i, item) in c.inventory.iter().enumerate() {
        if !wearable(item) {
            continue;
        }
        let Some(want) = slot_type_for(item.tval) else {
            continue;
        };
        let jewellery = matches!(want, slot::RING | slot::AMULET);
        let slot = c
            .equipment
            .iter()
            .position(|s| s.slot_type == want && (!jewellery || s.item.is_none()));
        let Some(slot) = slot else { continue };
        let after = effects(&c.wearing(k, slot, Some(item.clone())), k);
        if !better(&before, &after) {
            continue;
        }
        let gain = |e: &Effects| (e.melee - before.melee, e.ac - before.ac);
        let replace = best.as_ref().is_none_or(|b| {
            let (bm, ba) = gain(&b.after);
            let (m, a) = gain(&after);
            m > bm + 1e-9 || ((m - bm).abs() <= 1e-9 && a > ba)
        });
        if replace {
            best = Some(Swap {
                item: i,
                before,
                after,
            });
        }
    }
    best
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::knowledge::{Book, Class};
    use crate::observation::Slot;

    fn energy(class: Class) -> Knowledge {
        let mut e = vec![1u8; 200];
        for (s, v) in [(110, 10), (120, 20), (130, 30)] {
            e[s] = v;
        }
        Knowledge {
            extract_energy: e,
            move_energy: 100,
            classes: vec![class],
            ..Default::default()
        }
    }

    /// The observation sample (a wielded 1d4 dagger) with empty body,
    /// ring and launcher slots.
    fn setup(class: Class) -> (World, Knowledge) {
        let o = crate::observation::tests::sample();
        let mut w = World::new();
        w.update(&o);
        for slot_type in [slot::BODY_ARMOR, slot::RING, 2] {
            w.character.equipment.push(Slot {
                slot_type,
                item: None,
            });
        }
        (w, energy(class))
    }

    fn dagger(w: &World) -> Item {
        w.character.equipment[0]
            .item
            .clone()
            .expect("the sample's dagger")
    }

    #[test]
    fn a_better_weapon_is_worn_by_its_damage() {
        let (mut w, k) = setup(crate::knowledge::tests::warrior());
        let mut maul = dagger(&w);
        maul.tval = tv::HAFTED;
        maul.dd = 4;
        maul.ds = 4;
        maul.to_d = 6;
        w.character.inventory = vec![maul];
        let s = best_swap(&w, &k).expect("the maul is better");
        assert_eq!(s.item, 0);
        assert!(s.after.melee > s.before.melee, "{s:?}");
        // The reverse is not better: no swap back.
        assert!(!better(&s.after, &s.before));
    }

    #[test]
    fn a_launcher_is_not_a_weapon() {
        let (mut w, k) = setup(crate::knowledge::tests::warrior());
        let mut bow = dagger(&w);
        bow.tval = tv::BOW;
        bow.dd = 9;
        bow.ds = 9;
        w.character.inventory = vec![bow];
        assert_eq!(best_swap(&w, &k), None);
    }

    #[test]
    fn armour_is_worn_by_its_ac_unless_it_costs_a_caster_mana() {
        let mut armour = dagger(&setup(crate::knowledge::tests::warrior()).0);
        armour.tval = tv::SOFT_ARMOR;
        armour.dd = 0;
        armour.ds = 0;
        armour.ac = 8;
        armour.weight = 80;
        let (mut w, k) = setup(crate::knowledge::tests::warrior());
        w.character.inventory = vec![armour.clone()];
        let s = best_swap(&w, &k).expect("8 AC is better");
        assert_eq!(s.after.ac, s.before.ac + 8, "{s:?}");
        // A caster whose allowance is 3 lb: 8 lb of armour costs
        // (80 - 30) / 10 = 5 spell points, so it is not worn.
        let caster = Class {
            books: vec![Book::default()],
            spell_weight: 30,
            ..crate::knowledge::tests::warrior()
        };
        let (mut w, k) = setup(caster);
        w.character.inventory = vec![armour];
        assert_eq!(best_swap(&w, &k), None);
    }

    #[test]
    fn jewellery_must_be_known_and_goes_in_an_empty_slot() {
        let (mut w, k) = setup(crate::knowledge::tests::warrior());
        let mut ring = dagger(&w);
        ring.tval = tv::RING;
        ring.dd = 0;
        ring.ds = 0;
        ring.to_a = 5;
        ring.fully_known = false;
        w.character.inventory = vec![ring.clone()];
        assert_eq!(best_swap(&w, &k), None, "runes unknown");
        ring.fully_known = true;
        w.character.inventory = vec![ring.clone()];
        assert!(best_swap(&w, &k).is_some(), "a known +5 AC ring");
        // The only ring slot taken: no swap (replacing would prompt).
        let slot = w
            .character
            .equipment
            .iter()
            .position(|s| s.slot_type == slot::RING)
            .unwrap();
        w.character.equipment[slot].item = Some(ring);
        w.character.gear = crate::world::GearBonuses::of(&w.character.equipment);
        let mut better_ring = w.character.inventory[0].clone();
        better_ring.to_a = 9;
        w.character.inventory = vec![better_ring];
        assert_eq!(best_swap(&w, &k), None);
    }
}
