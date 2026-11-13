//! Spells (T5.3, #127): what to study and what an attack spell is worth.
//!
//! Everything here is read from the engine through the observation, the way
//! the spell menu shows it to a player:
//!
//! * the failure chance is the engine's own (`player-spell.c:spell_chance`,
//!   the menu's "Fail" column), with this think's level, stats, mana, light,
//!   fear, stun and amnesia;
//! * the damage dice are the spell effect's, evaluated for the character's
//!   level without a roll (`z-dice.c:dice_random_value`);
//! * the book is found in the pack by the class's book kind
//!   (`class.txt` `book:` lines), and the spell's menu letter is its position
//!   in that book (`ui-spell.c:spell_menu_new:251`).
//!
//! **Studying** (`cmd-obj.c:do_cmd_study:1245`): a `CHOOSE_SPELLS` class
//! learns an attack spell first, then the lowest level; the others (priest,
//! paladin) learn a random spell of the chosen book (`do_cmd_study_book`:
//! the engine draws it, so the prayer the Borg gets is the game's, not
//! ours), from the first book that still has one.
//!
//! **Attack spells**: only the target-centred effects, which deal their full
//! damage to the target (`BOLT`, `BOLT_OR_BEAM`, `BALL`, `STRIKE`). Arcs,
//! short beams and lines change with distance and are not used yet
//! (declared). The expected damage of one cast is the dice's mean, times the
//! chance it works (`1 - fail`), times the target's resistance to the
//! element (`project-mon.c`: nether 0 on undead, about a third on
//! `IM_NETHER`, half on evil; poison a ninth on `IM_POIS`).

use crate::executor::{Action, SPELL_LETTERS};
use crate::knowledge::Knowledge;
use crate::observation::{Spell, SpellDamage};
use crate::world::{Track, World};

/// `PY_SPELL_*` (`player.h`).
const LEARNED: u8 = 1;
const FORGOTTEN: u8 = 4;

/// `PF_CHOOSE_SPELLS` (`list-player-flags.h:19`, the sixth after `PF_NONE`).
const PF_CHOOSE_SPELLS: usize = 6;

/// `RF_*` flag numbers (`list-mon-race-flags.h`, counted from `RF_NONE`).
mod rf {
    pub const EVIL: usize = 62;
    pub const UNDEAD: usize = 63;
    pub const IM_POIS: usize = 74;
    pub const IM_NETHER: usize = 75;
}

/// `ELEM_*` (`list-elements.h`).
mod elem {
    pub const POIS: i32 = 4;
    pub const NETHER: i32 = 10;
}

/// Whether the character can cast at all (`player-util.c:player_can_cast`):
/// its grid seen (`no_light`), not blind, not confused.
pub fn can_cast(w: &World) -> bool {
    let Some(s) = w.character.status.as_ref() else {
        return false;
    };
    let timed = |i: usize| s.timed.get(i).copied().unwrap_or(0);
    let lit = w.level.at(s.x, s.y).is_some_and(|g| g.seen == Some(w.turn));
    // TMD_BLIND 2, TMD_CONFUSED 4 (list-player-timed.h).
    lit && timed(2) <= 0 && timed(4) <= 0
}

/// The pack index of the class's book `book` (its index in the class's
/// books), when the pack holds a known copy.
fn book_item(w: &World, k: &Knowledge, book: i32) -> Option<usize> {
    let s = w.character.status.as_ref()?;
    let class = k.class(s.class)?;
    let b = class.books.get(usize::try_from(book).ok()?)?;
    w.character
        .inventory
        .iter()
        .position(|it| it.aware && i32::from(it.tval) == b.tval && i32::from(it.sval) == b.sval)
}

/// The spell's menu letter: its position in its book.
fn letter(k: &Knowledge, class: u32, book: i32, index: usize) -> Option<u8> {
    let b = k.class(class)?.books.get(usize::try_from(book).ok()?)?;
    let pos = b
        .spells
        .iter()
        .position(|cs| usize::try_from(cs.index).ok() == Some(index))?;
    SPELL_LETTERS.get(pos).copied()
}

/// Whether the effect is one this module values (see the module docs).
fn targeted(d: &SpellDamage) -> bool {
    matches!(
        d.effect.as_str(),
        "BOLT" | "BOLT_OR_BEAM" | "BALL" | "STRIKE"
    )
}

/// What to study now, if anything (`new_spells > 0`, a learnable spell in a
/// book the pack holds).
pub fn study(w: &World, k: &Knowledge) -> Option<Action> {
    let s = w.character.status.as_ref()?;
    if s.new_spells <= 0 || !can_cast(w) {
        return None;
    }
    let class = k.class(s.class)?;
    let learnable = |sp: &Spell| sp.level <= s.level && sp.flags & LEARNED == 0;
    let candidates = w
        .character
        .spells
        .iter()
        .enumerate()
        .filter(|(_, sp)| learnable(sp))
        .filter_map(|(i, sp)| book_item(w, k, sp.book).map(|item| (i, sp, item)));
    if class.has_pflag(PF_CHOOSE_SPELLS) {
        let (i, sp, item) = candidates.min_by_key(|(i, sp, _)| {
            let attack = sp.damage.as_ref().is_some_and(targeted);
            (!attack, sp.level, *i)
        })?;
        Some(Action::Study {
            book: item,
            letter: Some(letter(k, s.class, sp.book, i)?),
        })
    } else {
        let (_, _, item) = candidates.min_by_key(|(i, sp, _)| (sp.book, *i))?;
        Some(Action::Study {
            book: item,
            letter: None,
        })
    }
}

/// A castable attack spell and its expected damage before resistance.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Attack {
    pub action: Action,
    /// The spell's index in the class (`sidx`).
    pub index: usize,
    /// The dice's mean times the chance the cast works.
    pub damage: f64,
    pub mana: i32,
}

/// Every attack spell the character can cast now: learned, not forgotten,
/// its book in the pack, its mana in hand (the Borg never casts on the
/// "Attempt it anyway?" of `do_cmd_cast:1141-1152`).
pub fn attacks(w: &World, k: &Knowledge) -> Vec<Attack> {
    let Some(s) = w.character.status.as_ref() else {
        return Vec::new();
    };
    if !can_cast(w) {
        return Vec::new();
    }
    let mut out = Vec::new();
    for (i, sp) in w.character.spells.iter().enumerate() {
        if sp.flags & LEARNED == 0 || sp.flags & FORGOTTEN != 0 || sp.mana > s.sp {
            continue;
        }
        let Some(d) = sp.damage.as_ref().filter(|d| targeted(d)) else {
            continue;
        };
        let Some(item) = book_item(w, k, sp.book) else {
            continue;
        };
        let Some(l) = letter(k, s.class, sp.book, i) else {
            continue;
        };
        let mean = f64::from(d.base) + f64::from(d.dice) * f64::from(d.sides + 1) / 2.0;
        let works = 1.0 - f64::from(sp.fail.clamp(0, 100)) / 100.0;
        out.push(Attack {
            action: Action::Cast {
                book: item,
                letter: l,
                aim: sp.aim,
            },
            index: i,
            damage: mean * works,
            mana: sp.mana,
        });
    }
    out
}

/// Whether the character knows an attack spell and carries its book,
/// mana and light aside: the readiness gate's ranged attack (#125).
pub fn knows_attack(w: &World, k: &Knowledge) -> bool {
    w.character.spells.iter().any(|sp| {
        sp.flags & LEARNED != 0
            && sp.flags & FORGOTTEN == 0
            && sp.damage.as_ref().is_some_and(targeted)
            && book_item(w, k, sp.book).is_some()
    })
}

/// The share of a spell's damage that reaches this monster
/// (`project-mon.c`, the element handlers' mean effect).
pub fn resisted(w: &World, k: &Knowledge, index: usize, t: &Track) -> f64 {
    let Some(d) = w
        .character
        .spells
        .get(index)
        .and_then(|sp| sp.damage.as_ref())
    else {
        return 0.0;
    };
    let Some(race) = k.race(t.race) else {
        return 1.0;
    };
    let has = |flag: usize| {
        race.flags
            .get((flag - 1) / 8)
            .is_some_and(|b| b & (1 << ((flag - 1) % 8)) != 0)
    };
    match d.element {
        // project_monster_handler_NETHER:590-623: undead immune; IM_NETHER
        // 3 / (7..=12), whose mean is 0.33; evil halves.
        elem::NETHER if has(rf::UNDEAD) => 0.0,
        elem::NETHER if has(rf::IM_NETHER) => 0.33,
        elem::NETHER if has(rf::EVIL) => 0.5,
        // project_monster_handler_POIS:528-531: IM_POIS divides by 9.
        elem::POIS if has(rf::IM_POIS) => 1.0 / 9.0,
        _ => 1.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::knowledge::{Book, Class, ClassSpell};

    /// A mage-like class: one book (tval 90, sval 1) holding Magic Missile
    /// (index 0) and Detect Monsters (index 1).
    fn setup(choose: bool) -> (World, Knowledge) {
        let o = crate::observation::tests::sample();
        let mut w = World::new();
        w.update(&o);
        let mut k = crate::path::tests::energy();
        let s = w.character.status.as_mut().unwrap();
        s.sp = 5;
        s.new_spells = 1;
        let class_index = s.class;
        k.classes = vec![Class {
            index: class_index,
            books: vec![Book {
                tval: 90,
                sval: 1,
                dungeon: false,
                spells: vec![
                    ClassSpell {
                        index: 0,
                        ..Default::default()
                    },
                    ClassSpell {
                        index: 1,
                        ..Default::default()
                    },
                ],
            }],
            pflags: if choose { vec![1 << 5] } else { vec![] },
            ..Default::default()
        }];
        let mut book = w.character.inventory[0].clone();
        book.tval = 90;
        book.sval = 1;
        book.aware = true;
        w.character.inventory = vec![w.character.inventory[0].clone(), book];
        let missile = w.character.spells[0].clone();
        let mut detect = missile.clone();
        detect.damage = None;
        detect.aim = false;
        w.character.spells = vec![missile, detect];
        assert!(can_cast(&w), "the sample's grid is seen this think");
        (w, k)
    }

    #[test]
    fn a_learned_missile_is_cast_at_its_letter_and_aims() {
        let (w, k) = setup(true);
        let a = attacks(&w, &k);
        assert_eq!(a.len(), 1, "{a:?}");
        assert_eq!(
            a[0].action,
            Action::Cast {
                book: 1,
                letter: b'a',
                aim: true
            }
        );
        // 3d4 is 7.5 on average; 22% fail leaves 5.85.
        assert!((a[0].damage - 5.85).abs() < 1e-9, "{a:?}");
    }

    #[test]
    fn no_cast_without_mana_light_or_the_book() {
        let (mut w, k) = setup(true);
        w.character.status.as_mut().unwrap().sp = 0;
        assert!(attacks(&w, &k).is_empty(), "mana");
        let (mut w, k) = setup(true);
        w.character.inventory.truncate(1);
        assert!(attacks(&w, &k).is_empty(), "book");
        let (mut w, k) = setup(true);
        w.turn += 1; // the grid was seen last think, not this one
        assert!(attacks(&w, &k).is_empty(), "light");
        let (mut w, k) = setup(true);
        w.character.spells[0].flags = 0;
        assert!(attacks(&w, &k).is_empty(), "not learned");
    }

    #[test]
    fn a_chooser_studies_the_attack_first_and_a_priest_the_book() {
        let (mut w, k) = setup(true);
        for sp in &mut w.character.spells {
            sp.flags = 0;
        }
        // Detect is spell 0 (book letter a), the missile spell 1 (letter
        // b): the attack is studied first.
        w.character.spells.swap(0, 1);
        assert_eq!(
            study(&w, &k),
            Some(Action::Study {
                book: 1,
                letter: Some(b'b')
            })
        );
        let (mut w, k) = setup(false);
        for sp in &mut w.character.spells {
            sp.flags = 0;
        }
        assert_eq!(
            study(&w, &k),
            Some(Action::Study {
                book: 1,
                letter: None
            })
        );
        w.character.status.as_mut().unwrap().new_spells = 0;
        assert_eq!(study(&w, &k), None);
    }
}
