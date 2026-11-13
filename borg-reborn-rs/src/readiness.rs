//! Depth readiness (T5.1, #125): one policy for every way down.
//!
//! The strategy manual's two gates, checked against the engine's data:
//!
//! * **DEPTH-01** (c15 §15.8): depth `d` is allowed when the worst single
//!   turn of every race native to `d - band_below ..= d + band_above`
//!   (`monster.txt` `depth:`; the band covers out-of-depth generation) stays
//!   below `depth_hp_frac_pct` of maximum hit points. The worst turn is
//!   [`crate::danger::worst_turn`], c15 DANGER-01's `T_max`. The races are
//!   the spoiler list (`S`): the whole monster file, not only the ones met.
//! * **DEPTH-02** (c15 §15.8, anchored to `monster.txt` there): the kit
//!   gates. DL5 wants Word of Recall, three cure potions and a ranged
//!   attack; DL15 wants 100 maximum hit points and ten Phase Doors; DL20
//!   wants Free Action. Deeper rows (See Invisible or ESP, resistances,
//!   speed) are not checked yet and fail, which keeps the policy
//!   conservative where nothing reaches today.
//!
//! The borg-runner lesson behind the first gate (S15 edition 7,
//! `mmf-mage.3`): a level is judged by the worst round it can serve, not
//! by the experience it pays. Depth 1 is always allowed: the town has no
//! experience, and nothing is learned by staying in it.
//!
//! One policy for every way down (#125):
//! * **stairs**: the strategy takes a down staircase only when ready for
//!   the next depth, and goes up when no longer ready for the current one
//!   (c15 DEPTH-04, which also gives the dive rule: ready for `d + 5`
//!   prefers the stairs to exploring);
//! * **Deep Descent** lands on `max_depth + 5`
//!   (`effect-handler-general.c:effect_handler_DEEP_DESCENT:1166-1168`), so
//!   its destination is [`deep_descent_target`]; the Borg never reads one
//!   today (no response offers it), and any later offer must pass
//!   [`ready`] for that target;
//! * **trapdoors**: the observation marks a known trap without its kind, so
//!   a trapdoor cannot be avoided by kind; a fall to an unready depth is
//!   answered by the go-up rule on arrival;
//! * **recall depth**: the Borg reads Word of Recall only to go home; a
//!   recall down would land on `max_depth` and must pass [`ready`] for it.

use crate::danger::{has_free_action, worst_turn};
use crate::knowledge::Knowledge;
use crate::params::Params;
use crate::world::{World, EQUIP_BOW};

/// `TV_*` values the gates count: `list-tvals.h` numbers its `TV(...)`
/// lines from `TV_NULL` = 0 (SCROLL is line 31, POTION 32, FLASK 33).
mod tv {
    pub const FLASK: u8 = 27;
    pub const SCROLL: u8 = 25;
    pub const POTION: u8 = 26;
}

/// What the policy found for one depth.
#[derive(Clone, Debug, PartialEq)]
pub struct Readiness {
    pub depth: i32,
    /// The worst single turn of the band, and the race that deals it.
    pub worst_turn: f64,
    pub worst_race: String,
    /// What the worst turn must stay below: `depth_hp_frac_pct` of max HP.
    pub limit: f64,
    /// The kit gates not met, by name.
    pub missing: Vec<&'static str>,
}

impl Readiness {
    pub fn ok(&self) -> bool {
        self.depth <= 1 || (self.worst_turn < self.limit && self.missing.is_empty())
    }
}

/// The readiness for depth `depth`.
pub fn ready(w: &World, k: &Knowledge, p: &Params, depth: i32) -> Readiness {
    let max_hp = w.character.status.as_ref().map_or(1, |s| s.max_hp.max(1));
    let limit = f64::from(max_hp) * f64::from(p.depth_hp_frac_pct) / 100.0;
    let mut out = Readiness {
        depth,
        worst_turn: 0.0,
        worst_race: String::new(),
        limit,
        missing: Vec::new(),
    };
    if depth <= 1 {
        return out;
    }
    let low = depth - p.readiness_band_below;
    let high = depth + p.readiness_band_above;
    for race in &k.races {
        // Level 0 is the town's people, never generated in the dungeon.
        if race.name.is_empty() || race.level < low.max(1) || race.level > high {
            continue;
        }
        let t = worst_turn(w, k, race);
        if t > out.worst_turn {
            out.worst_turn = t;
            out.worst_race = race.name.clone();
        }
    }
    out.missing = missing_kit(w, k, depth, max_hp);
    out
}

/// Where a Deep Descent read now would land: `max_depth + 5`
/// (`effect-handler-general.c:effect_handler_DEEP_DESCENT:1166-1168`, the
/// increment `4 / stair_skip + 1` with the default `stair_skip` 1).
pub fn deep_descent_target(w: &World) -> i32 {
    let max_depth = w.character.status.as_ref().map_or(0, |s| s.max_depth);
    max_depth + 5
}

fn missing_kit(w: &World, k: &Knowledge, depth: i32, max_hp: i32) -> Vec<&'static str> {
    let mut out = Vec::new();
    if depth >= 5 {
        if count(w, k, tv::SCROLL, &["Word of Recall"]) < 1 {
            out.push("Word of Recall");
        }
        let cures = count(
            w,
            k,
            tv::POTION,
            &[
                "Cure Light Wounds",
                "Cure Serious Wounds",
                "Cure Critical Wounds",
            ],
        );
        if cures < 3 {
            out.push("three cure potions");
        }
        if !has_ranged(w, k) {
            out.push("a ranged attack");
        }
    }
    if depth >= 15 {
        if max_hp < 100 {
            out.push("100 maximum hit points");
        }
        if count(w, k, tv::SCROLL, &["Phase Door"]) < 10 {
            out.push("ten Phase Doors");
        }
    }
    if depth >= 20 && !has_free_action(w) {
        out.push("Free Action");
    }
    if depth >= 25 {
        // c15 DEPTH-02's rows from DL25 (See Invisible or ESP, Teleport x5,
        // resistances, speed) are not checked yet: fail, conservatively.
        out.push("the DL25+ gates (not checked yet)");
    }
    out
}

/// Aware pack items of `tval` whose kind is one of `names`, counted.
fn count(w: &World, k: &Knowledge, tval: u8, names: &[&str]) -> u32 {
    w.character
        .inventory
        .iter()
        .filter(|it| it.aware && it.tval == tval)
        .filter(|it| {
            k.kind(it.kind)
                .is_some_and(|kind| names.contains(&plain(&kind.name).as_str()))
        })
        .map(|it| u32::from(it.number))
        .sum()
}

/// A ranged attack carried: a launcher with ammunition in the quiver, or
/// flasks of oil to throw (five, a declared handful). Using them is #127's;
/// the gate asks for the kit.
fn has_ranged(w: &World, k: &Knowledge) -> bool {
    let launcher = w
        .character
        .equipment
        .iter()
        .any(|s| s.slot_type == EQUIP_BOW && s.item.is_some());
    let ammo: u32 = w
        .character
        .quiver
        .iter()
        .map(|it| u32::from(it.number))
        .sum();
    (launcher && ammo > 0) || count(w, k, tv::FLASK, &["Flask of oil"]) >= 5
}

/// A kind's name without the article and plural marks (`& Flask~ of oil`).
fn plain(name: &str) -> String {
    name.chars()
        .filter(|c| !matches!(c, '&' | '~'))
        .collect::<String>()
        .trim()
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::knowledge::{Blow, Dice, MonsterRace, ObjectKind};
    use crate::observation::Item;

    fn race(name: &str, level: i32, dice: i32, sides: i32) -> MonsterRace {
        MonsterRace {
            index: 1,
            name: name.into(),
            level,
            speed: 110,
            blows: vec![Blow {
                method: "BITE".into(),
                effect: "HURT".into(),
                power: 60,
                dice: Dice {
                    base: 0,
                    dice,
                    sides,
                    m_bonus: 0,
                },
            }],
            ..Default::default()
        }
    }

    fn item(kind: u32, tval: u8, number: u8) -> Item {
        Item {
            kind,
            tval,
            sval: 0,
            number,
            weight: 1,
            timeout: 0,
            level: 1,
            desc: String::new(),
            aware: true,
            fully_known: true,
            runes_known: true,
            flags: [0; 5],
            ac: 0,
            dd: 0,
            ds: 0,
            to_h: 0,
            to_d: 0,
            to_a: 0,
            modifiers: vec![0; 16],
            elements: vec![(0, 0); 25],
        }
    }

    /// A character with `max_hp` and an empty pack, and knowledge holding
    /// `races` and the kit's kinds (1 Recall, 2 CLW, 3 Phase, 4 oil).
    fn setup(max_hp: i32, races: Vec<MonsterRace>) -> (World, Knowledge, Params) {
        let (mut w, _) = crate::path::tests::world(&["#####", "#@..#", "#####"]);
        if let Some(s) = w.character.status.as_mut() {
            s.max_hp = max_hp;
            s.hp = max_hp;
            s.known_ac = 0;
            s.max_depth = 4;
        }
        w.character.inventory.clear();
        let mut k = crate::path::tests::energy();
        k.races = races;
        k.kinds = (0..5)
            .map(|index| ObjectKind {
                index,
                ..Default::default()
            })
            .collect();
        for (i, name) in [
            (1, "Word of Recall"),
            (2, "Cure Light Wounds"),
            (3, "Phase Door"),
            (4, "& Flask~ of oil"),
        ] {
            k.kinds[i].name = name.into();
        }
        (w, k, Params::default())
    }

    #[test]
    fn depth_one_is_always_allowed() {
        let (w, k, p) = setup(5, vec![race("dragon", 1, 20, 10)]);
        assert!(ready(&w, &k, &p, 1).ok());
        assert!(ready(&w, &k, &p, 0).ok());
    }

    #[test]
    fn the_worst_turn_of_the_band_gates_the_depth() {
        // A 2d8 biter native to DL4: 16 in one turn. It is in DL2's band
        // (0..=7) and DL3's; at 20 max HP the limit is 10, at 40 it is 20.
        let (w, k, p) = setup(20, vec![race("hound", 4, 2, 8)]);
        let r = ready(&w, &k, &p, 3);
        assert_eq!((r.worst_turn, r.worst_race.as_str()), (16.0, "hound"));
        assert!(!r.ok() && r.limit == 10.0, "{r:?}");
        let (w, k, p) = setup(40, vec![race("hound", 4, 2, 8)]);
        assert!(ready(&w, &k, &p, 3).ok());
        // Out of the band: a race native to DL20 does not gate DL3.
        let (w, k, p) = setup(20, vec![race("wyrm", 20, 20, 10)]);
        assert!(ready(&w, &k, &p, 3).ok());
        assert!(!ready(&w, &k, &p, 15).ok());
    }

    #[test]
    fn the_kit_gates_follow_the_manual() {
        let (mut w, k, p) = setup(200, vec![]);
        let r = ready(&w, &k, &p, 5);
        assert_eq!(
            r.missing,
            vec!["Word of Recall", "three cure potions", "a ranged attack"]
        );
        w.character.inventory = vec![
            item(1, tv::SCROLL, 1),
            item(2, tv::POTION, 3),
            item(4, tv::FLASK, 5),
        ];
        assert!(ready(&w, &k, &p, 5).ok(), "{:?}", ready(&w, &k, &p, 5));
        // A staff of the same name is not a potion.
        w.character.inventory[1].tval = 21;
        assert_eq!(ready(&w, &k, &p, 5).missing, vec!["three cure potions"]);
        w.character.inventory[1].tval = tv::POTION;
        // DL15 adds 100 max HP (met here) and ten Phase Doors.
        assert_eq!(ready(&w, &k, &p, 15).missing, vec!["ten Phase Doors"]);
        w.character.inventory.push(item(3, tv::SCROLL, 10));
        assert!(ready(&w, &k, &p, 15).ok());
        // DL20 wants Free Action, which this gear lacks.
        assert_eq!(ready(&w, &k, &p, 20).missing, vec!["Free Action"]);
    }

    #[test]
    fn deep_descent_lands_five_below_the_deepest() {
        let (w, _, _) = setup(20, vec![]);
        assert_eq!(deep_descent_target(&w), 9);
    }
}
