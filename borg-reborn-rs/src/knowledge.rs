//! L3: knowledge, the static facts of the game (ARCHITECTURE.md L3; task
//! T2.3, #14).
//!
//! The tables are read once from the engine's parsed data (`r_info`,
//! `k_info`, `e_info`, `a_info`, the class and race lists, the stores, and
//! `extract_energy`) by `engine::knowledge()`. GOALS D3 allows full monster
//! race data, not just the player's lore, as the C Borg uses. Formulas the
//! engine keeps in file-local tables are lifted here, each with its
//! `file:function:lines` at the pinned engine (`e473720`, unchanged in our
//! tree). Where a C Borg copies an engine formula, the citation names both,
//! and the better-borg-c milestone's version is preferred when they differ
//! (parity is not a goal).
//!
//! Everything is plain data, so a knowledge dump (`BORG_KNOWLEDGE_LOG`)
//! can be checked against the game's own values without the game.

use serde::{Deserialize, Serialize};

/// The static game data the Borg may use.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Knowledge {
    /// Monster races by `ridx` (index 0 is the unused entry).
    pub races: Vec<MonsterRace>,
    /// Object kinds by `kidx`.
    pub kinds: Vec<ObjectKind>,
    /// Ego items by `eidx`.
    pub egos: Vec<Ego>,
    /// Artifacts by `aidx`.
    pub artifacts: Vec<Artifact>,
    /// Player classes by `cidx`.
    pub classes: Vec<Class>,
    /// Player races by `ridx`.
    pub player_races: Vec<PlayerRace>,
    /// Stores by number (store.txt order; the home is 7).
    pub stores: Vec<StoreKind>,
    /// `extract_energy[speed]` (game-world.c:72): energy a creature of this
    /// speed gains per game turn, before `move_energy` scaling.
    pub extract_energy: Vec<u8>,
    /// `z_info->move_energy`: the energy one action costs.
    pub move_energy: i32,
    /// `adj_str_blow[]` as the engine has it (player-calcs.c:548), read to
    /// check the lifted copy ([`ADJ_STR_BLOW`]).
    pub adj_str_blow: Vec<i32>,
}

/// A monster race (`struct monster_race`): the spoiler data, not lore.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct MonsterRace {
    pub index: u32,
    pub name: String,
    pub base: String,
    pub level: i32,
    pub rarity: i32,
    /// 110 is normal speed.
    pub speed: i32,
    pub avg_hp: i32,
    pub ac: i32,
    pub sleep: i32,
    pub hearing: i32,
    pub smell: i32,
    pub exp: i32,
    /// 1 in `freq_innate` / `freq_spell` turns it uses an innate or a
    /// magical ability (0: never).
    pub freq_innate: i32,
    pub freq_spell: i32,
    pub spell_power: i32,
    /// `RF_*` bits (list-mon-race-flags.h).
    pub flags: Vec<u8>,
    /// `RSF_*` bits (list-mon-spells.h).
    pub spell_flags: Vec<u8>,
    pub blows: Vec<Blow>,
    pub glyph: u32,
    pub attr: u8,
}

/// One melee blow of a race: method, effect and damage dice.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Blow {
    pub method: String,
    pub effect: String,
    /// The effect's power (`blow_effect.power`), used for the hit chance.
    pub power: i32,
    pub dice: Dice,
}

/// `random_value`: base + dice d sides (+ m_bonus).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Dice {
    pub base: i32,
    pub dice: i32,
    pub sides: i32,
    pub m_bonus: i32,
}

impl Dice {
    /// The expected value of base + dice d sides, without `m_bonus`.
    pub fn mean(&self) -> f64 {
        f64::from(self.base) + f64::from(self.dice) * (f64::from(self.sides) + 1.0) / 2.0
    }

    /// The largest value of base + dice d sides.
    pub fn max(&self) -> i32 {
        self.base + self.dice * self.sides
    }
}

/// An object kind (`struct object_kind`).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ObjectKind {
    pub index: u32,
    pub name: String,
    pub tval: i32,
    pub sval: i32,
    pub level: i32,
    pub cost: i32,
    pub weight: i32,
    pub ac: i32,
    pub dd: i32,
    pub ds: i32,
    pub to_h: Dice,
    pub to_d: Dice,
    pub to_a: Dice,
    pub pval: Dice,
    /// `OF_*` bits.
    pub flags: Vec<u8>,
    /// `KF_*` bits.
    pub kind_flags: Vec<u8>,
    /// `el_info[ELEM_*]` as (resistance level, flags).
    pub elements: Vec<(i16, u8)>,
}

/// An ego item type (`struct ego_item`).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Ego {
    pub index: u32,
    pub name: String,
    pub cost: i32,
    pub rating: i32,
    pub flags: Vec<u8>,
    pub to_h: Dice,
    pub to_d: Dice,
    pub to_a: Dice,
    pub modifiers: Vec<Dice>,
    pub elements: Vec<(i16, u8)>,
}

/// An artifact (`struct artifact`).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Artifact {
    pub index: u32,
    pub name: String,
    pub tval: i32,
    pub sval: i32,
    pub level: i32,
    pub cost: i32,
    pub weight: i32,
    pub ac: i32,
    pub dd: i32,
    pub ds: i32,
    pub to_h: i32,
    pub to_d: i32,
    pub to_a: i32,
    pub flags: Vec<u8>,
    pub modifiers: Vec<i32>,
    pub elements: Vec<(i16, u8)>,
}

/// A player class (`struct player_class`): what melee and magic need.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Class {
    pub index: u32,
    pub name: String,
    pub stat_adj: [i32; 5],
    pub skills: [i32; 10],
    pub hit_die: i32,
    pub max_attacks: i32,
    /// Tenths of a pound.
    pub min_weight: i32,
    pub att_multiply: i32,
    pub spell_first: i32,
    pub spell_weight: i32,
    pub books: Vec<Book>,
}

/// A class's spell book.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Book {
    pub tval: i32,
    pub sval: i32,
    /// A dungeon book (not sold in town).
    pub dungeon: bool,
    pub spells: Vec<ClassSpell>,
}

/// One spell of a class (`struct class_spell`).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClassSpell {
    /// The spell's index in the class (`sidx`), as `spell_flags[]` uses.
    pub index: i32,
    pub name: String,
    pub level: i32,
    pub mana: i32,
    pub fail: i32,
    pub exp: i32,
}

/// A player race (`struct player_race`).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlayerRace {
    pub index: u32,
    pub name: String,
    pub stat_adj: [i32; 5],
    pub skills: [i32; 10],
    pub hit_die: i32,
    pub exp_factor: i32,
    pub infravision: i32,
    pub elements: Vec<(i16, u8)>,
}

/// A store (`struct store`): what it always stocks and may stock.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct StoreKind {
    pub number: i32,
    /// The kinds it always has (`always_table`).
    pub always: Vec<u32>,
    /// The kinds it may stock (`normal_table`).
    pub normal: Vec<u32>,
}

/// `adj_str_blow[]` (`angband/src/player-calcs.c:548-587`): the strength
/// term of the blows calculation, by stat index.
pub const ADJ_STR_BLOW: [i32; 38] = [
    3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 20, 30, 40, 50, 60, 70, 80, 90, 100, 110,
    120, 130, 140, 150, 160, 170, 180, 190, 200, 210, 220, 230, 240,
];

/// `adj_dex_blow[]` (`angband/src/player-calcs.c:594-633`, file-local in
/// the engine; the C Borg keeps a copy, `borg_adj_dex_blow`).
pub const ADJ_DEX_BLOW: [i32; 38] = [
    0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 2, 2, 2, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7, 8, 8, 8, 9, 9,
    9, 10, 10, 11, 11, 11,
];

/// `blows_table[P][D]` (`angband/src/player-calcs.c:889-928`, file-local;
/// the C Borg's copy is `borg_blows_table`): the energy one blow costs.
pub const BLOWS_TABLE: [[i32; 12]; 12] = [
    [100, 100, 95, 85, 75, 60, 50, 42, 35, 30, 25, 23],
    [100, 95, 85, 75, 60, 50, 42, 35, 30, 25, 23, 21],
    [95, 85, 75, 60, 50, 42, 35, 30, 26, 23, 21, 20],
    [85, 75, 60, 50, 42, 36, 32, 28, 25, 22, 20, 19],
    [75, 60, 50, 42, 36, 33, 28, 25, 23, 21, 19, 18],
    [60, 50, 42, 36, 33, 30, 27, 24, 22, 21, 19, 17],
    [50, 42, 36, 33, 30, 27, 25, 23, 21, 20, 18, 17],
    [42, 36, 33, 30, 28, 26, 24, 22, 20, 19, 18, 17],
    [36, 33, 30, 28, 26, 24, 22, 21, 20, 19, 17, 16],
    [35, 32, 29, 26, 24, 22, 21, 20, 19, 18, 17, 16],
    [34, 30, 27, 25, 23, 22, 21, 20, 19, 18, 17, 16],
    [33, 29, 26, 24, 22, 21, 20, 19, 18, 17, 16, 15],
];

/// The index of a stat value into the stat tables
/// (`angband/src/player-calcs.c:calc_bonuses:2065-2073`): 3–18 give 0–15,
/// 18/00–18/219 one step per 10 points, 18/220 and more give 37.
pub fn stat_index(value: i32) -> usize {
    let ind = if value <= 3 {
        0
    } else if value <= 18 {
        value - 3
    } else if value <= 18 + 219 {
        15 + (value - 18) / 10
    } else {
        37
    };
    ind as usize
}

/// Blows per round times 100, as the engine computes them
/// (`angband/src/player-calcs.c:calc_blows:1703-1735`). The C Borg's copy
/// (`borg-trait.c:borg_calc_blows:1035-1072`) multiplied the weight by the
/// stack's size; better-borg-c fixed that to one weapon (`668ac32`, in
/// milestone 8), as the engine does, and this follows them.
///
/// `weight` is one weapon's weight in tenths of a pound (0 bare-handed),
/// `extra_blows` the gear's `OBJ_MOD_BLOWS` (whole blows), and
/// `percent_damage` the birth option `birth_percent_damage` (O-combat).
pub fn blows(
    class: &Class,
    str_index: usize,
    dex_index: usize,
    weight: i32,
    extra_blows: i32,
    percent_damage: bool,
) -> i32 {
    let div = weight.max(class.min_weight).max(1);
    let str_i = (ADJ_STR_BLOW[str_index.min(37)] * class.att_multiply / div).clamp(0, 11);
    let dex_i = ADJ_DEX_BLOW[dex_index.min(37)].min(11);
    let energy = BLOWS_TABLE[str_i as usize][dex_i as usize];
    let blows = (10000 / energy).min(100 * class.max_attacks);
    (blows + 100 * extra_blows).max(if percent_damage { 200 } else { 100 })
}

impl Knowledge {
    /// Energy gained per game turn at `speed` (`game-world.c:turn_energy:
    /// 140-143`): `extract_energy[speed] * move_energy / 100`.
    pub fn turn_energy(&self, speed: i32) -> i32 {
        let i = speed.clamp(0, self.extract_energy.len() as i32 - 1).max(0) as usize;
        i32::from(self.extract_energy.get(i).copied().unwrap_or(10)) * self.move_energy / 100
    }

    /// Game turns between two actions at `speed`: `move_energy` over the
    /// energy gained per game turn.
    pub fn turns_per_action(&self, speed: i32) -> f64 {
        f64::from(self.move_energy.max(1)) / f64::from(self.turn_energy(speed).max(1))
    }

    pub fn race(&self, index: u32) -> Option<&MonsterRace> {
        self.races.get(index as usize).filter(|r| r.index == index)
    }

    pub fn kind(&self, index: u32) -> Option<&ObjectKind> {
        self.kinds.get(index as usize).filter(|k| k.index == index)
    }

    pub fn class(&self, index: u32) -> Option<&Class> {
        self.classes.iter().find(|c| c.index == index)
    }

    /// One line of a knowledge log (JSON, no newline).
    pub fn to_line(&self) -> String {
        serde_json::to_string(self).expect("knowledge is plain data")
    }

    pub fn from_line(line: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(line)
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    /// The Warrior of class.txt (4.2.6): max-attacks 6, min-weight 30,
    /// strength-multiplier 5.
    pub(crate) fn warrior() -> Class {
        Class {
            index: 0,
            name: "Warrior".into(),
            max_attacks: 6,
            min_weight: 30,
            att_multiply: 5,
            ..Default::default()
        }
    }

    /// The Mage of class.txt (4.2.6): max-attacks 4, min-weight 40,
    /// strength-multiplier 2.
    fn mage() -> Class {
        Class {
            index: 1,
            name: "Mage".into(),
            max_attacks: 4,
            min_weight: 40,
            att_multiply: 2,
            ..Default::default()
        }
    }

    #[test]
    fn stat_index_follows_calc_bonuses() {
        assert_eq!(stat_index(3), 0);
        assert_eq!(stat_index(10), 7);
        assert_eq!(stat_index(18), 15);
        assert_eq!(stat_index(18 + 9), 15, "18/09");
        assert_eq!(stat_index(18 + 10), 16, "18/10");
        assert_eq!(stat_index(18 + 100), 25, "18/100");
        assert_eq!(stat_index(18 + 219), 36);
        assert_eq!(stat_index(18 + 220), 37);
        assert_eq!(stat_index(400), 37);
    }

    /// The C values, worked by hand from calc_blows and its tables.
    #[test]
    fn blows_follow_calc_blows() {
        // STR 17 (index 14: adj 17), DEX 17 (index 14: adj 2), a 12 lb
        // dagger: 17 * 5 / 120 = 0 -> table[0][2] = 95 -> 10000/95 = 105.
        assert_eq!(blows(&warrior(), 14, 14, 120, 0, false), 105);
        // STR 18/100 (index 25: adj 120), DEX 18/50 (index 20: adj 4), the
        // same dagger: 120 * 5 / 120 = 5 -> table[5][4] = 33 -> 303.
        assert_eq!(blows(&warrior(), 25, 20, 120, 0, false), 303);
        // A light weapon is weighed at min-weight (3 lb): 120 * 5 / 30 = 20,
        // capped at 11; DEX 18/220 (adj 11) -> 15 -> 666, capped at 6 blows.
        assert_eq!(blows(&warrior(), 25, 37, 10, 0, false), 600);
        // A Mage with STR 10 (adj 10) and DEX 14 (adj 1), a dagger:
        // 10 * 2 / 120 = 0 -> table[0][1] = 100 -> 100.
        assert_eq!(blows(&mage(), 7, 11, 120, 0, false), 100);
        // Extra blows add whole blows; O-combat needs at least two.
        assert_eq!(blows(&mage(), 7, 11, 120, 1, false), 200);
        assert_eq!(blows(&mage(), 7, 11, 120, 0, true), 200);
    }

    #[test]
    fn energy_scales_with_speed() {
        // extract_energy (game-world.c:72-134) at a few speeds.
        let mut e = vec![1u8; 200];
        e[110] = 10;
        e[120] = 20;
        e[130] = 30;
        let k = Knowledge {
            extract_energy: e,
            move_energy: 100,
            ..Default::default()
        };
        assert_eq!(k.turn_energy(110), 10);
        assert_eq!(k.turns_per_action(110), 10.0);
        assert_eq!(k.turns_per_action(120), 5.0);
        assert_eq!(k.turn_energy(500), 1, "clamped to the table");
    }

    #[test]
    fn dice_means() {
        let d = Dice {
            base: 2,
            dice: 1,
            sides: 4,
            m_bonus: 0,
        };
        assert_eq!(d.mean(), 4.5);
        assert_eq!(d.max(), 6);
    }

    /// The T2.3 check against the game's own values. CI dumps the knowledge
    /// of a real game (`BORG_KNOWLEDGE_LOG`) and points
    /// `BORG_KNOWLEDGE_LOG_CHECK` at it; without it this test has nothing
    /// to read. Values from the pinned `lib/gamedata` and engine tables.
    #[test]
    fn recorded_knowledge_matches_the_game_data() {
        let Some(path) = std::env::var_os("BORG_KNOWLEDGE_LOG_CHECK") else {
            return;
        };
        let text = std::fs::read_to_string(path).expect("the knowledge log");
        let k = Knowledge::from_line(text.lines().next().expect("one line")).unwrap();
        assert_eq!(k.to_line(), text.lines().next().unwrap(), "canonical");
        assert_eq!(
            k.adj_str_blow,
            ADJ_STR_BLOW.to_vec(),
            "the lifted adj_str_blow"
        );
        assert_eq!(k.extract_energy.len(), 200);
        assert_eq!((k.extract_energy[110], k.extract_energy[120]), (10, 20));
        assert_eq!(k.move_energy, 100);
        // monster.txt: Grip, Farmer Maggot's Dog.
        let grip = k
            .races
            .iter()
            .find(|r| r.name == "Grip, Farmer Maggot's Dog")
            .expect("Grip");
        assert_eq!(
            (grip.speed, grip.avg_hp, grip.ac, grip.level, grip.sleep),
            (120, 25, 36, 2, 0)
        );
        assert_eq!(grip.blows.len(), 1);
        assert_eq!(
            (grip.blows[0].method.as_str(), grip.blows[0].effect.as_str()),
            ("BITE", "HURT")
        );
        assert_eq!((grip.blows[0].dice.dice, grip.blows[0].dice.sides), (1, 4));
        // Unused entries (no name, index 0) pad the arrays; every named one
        // sits at its own index, so `race()` and `kind()` can index.
        for (i, r) in k
            .races
            .iter()
            .enumerate()
            .filter(|(_, r)| !r.name.is_empty())
        {
            assert_eq!(r.index as usize, i, "races are indexed by ridx");
        }
        for (i, o) in k
            .kinds
            .iter()
            .enumerate()
            .filter(|(_, o)| !o.name.is_empty())
        {
            assert_eq!(o.index as usize, i, "kinds are indexed by kidx");
        }
        assert!(k.races.iter().filter(|r| !r.name.is_empty()).count() > 500);
        assert_eq!(
            k.race(grip.index).map(|r| r.name.as_str()),
            Some(grip.name.as_str())
        );
        // class.txt: the Warrior's blows parameters.
        let w = k
            .classes
            .iter()
            .find(|c| c.name == "Warrior")
            .expect("Warrior");
        let lifted = warrior();
        assert_eq!(
            (w.max_attacks, w.min_weight, w.att_multiply),
            (lifted.max_attacks, lifted.min_weight, lifted.att_multiply)
        );
        let m = k.classes.iter().find(|c| c.name == "Mage").expect("Mage");
        assert_eq!((m.max_attacks, m.min_weight, m.att_multiply), (4, 40, 2));
        assert!(
            m.books.iter().any(|b| !b.spells.is_empty()),
            "the Mage has spells"
        );
        assert_eq!(
            k.stores.len(),
            8,
            "store.txt has 8 stores, the home included"
        );
        assert!(
            !k.stores[0].always.is_empty(),
            "the General Store always stocks food"
        );
        eprintln!(
            "knowledge: {} races, {} kinds, {} egos, {} artifacts, {} classes, {} player races",
            k.races.len(),
            k.kinds.len(),
            k.egos.len(),
            k.artifacts.len(),
            k.classes.len(),
            k.player_races.len()
        );
    }
}
