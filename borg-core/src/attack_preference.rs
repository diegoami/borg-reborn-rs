//! Class/family preferences (#403, design #396).
//!
//! Invocation determines family, including hybrid weapon spells. This module
//! performs no simulation, engine query or RNG draw. Native helper utility and
//! eligibility are inputs; preference never promotes nonpositive results.
use crate::borg_headers::borg_fight_attack_h::*;
use crate::params::{self, Param};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Family { Melee, Projectile, Thrown, Spell, Device, Activation, Wait }

impl Family {
    pub(crate) fn name(self) -> &'static str {
        match self {
            Self::Melee => "melee",
            Self::Projectile => "projectile",
            Self::Thrown => "thrown",
            Self::Spell => "spell",
            Self::Device => "device",
            Self::Activation => "activation",
            Self::Wait => "wait",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct AttackKind {
    pub name: &'static str,
    pub family: Family,
}

// Symbolic cases, deliberately not BF numeric ranges or runtime name prefixes.
// Unknown integer IDs are rejected explicitly. The inventory test compares this
// entire list to the header declarations, including spelling and numeric IDs.
macro_rules! attacks {
    ($($id:ident => $family:ident),* $(,)?) => {
        pub(crate) fn classify(id: i32) -> Option<AttackKind> {
            match u32::try_from(id).ok()? {
                $($id => Some(AttackKind { name: stringify!($id), family: Family::$family }),)*
                _ => None,
            }
        }
        #[cfg(test)]
        const INVENTORY: &[(u32, &str)] = &[$(($id, stringify!($id))),*];
    };
}

attacks! {
    BF_REST => Wait,
    BF_THRUST => Melee,
    BF_OBJECT => Thrown,
    BF_LAUNCH => Projectile,
    BF_SPELL_MAGIC_MISSILE => Spell,
    BF_SPELL_MAGIC_MISSILE_RESERVE => Spell,
    BF_SPELL_STINK_CLOUD => Spell,
    BF_SPELL_LIGHT_BEAM => Spell,
    BF_SPELL_COLD_BOLT => Spell,
    BF_SPELL_STONE_TO_MUD => Spell,
    BF_SPELL_SLOW_MONSTER => Spell,
    BF_SPELL_SLEEP_III => Spell,
    BF_SPELL_FIRE_BALL => Spell,
    BF_SPELL_SHOCK_WAVE => Spell,
    BF_SPELL_EXPLOSION => Spell,
    BF_SPELL_CONFUSE_MONSTER => Spell,
    BF_SPELL_COLD_STORM => Spell,
    BF_SPELL_METEOR_SWARM => Spell,
    BF_SPELL_RIFT => Spell,
    BF_SPELL_MANA_STORM => Spell,
    BF_SPELL_BLIND_CREATURE => Spell,
    BF_SPELL_TRANCE => Spell,
    BF_PRAYER_HOLY_ORB_BALL => Spell,
    BF_PRAYER_DISP_UNDEAD => Spell,
    BF_PRAYER_DISP_EVIL => Spell,
    BF_PRAYER_DISP_SPIRITS => Spell,
    BF_PRAYER_HOLY_WORD => Spell,
    BF_SPELL_ANNIHILATE => Spell,
    BF_SPELL_ELECTRIC_ARC => Spell,
    BF_SPELL_ACID_SPRAY => Spell,
    BF_SPELL_MANA_BOLT => Spell,
    BF_SPELL_THRUST_AWAY => Spell,
    BF_SPELL_LIGHTNING_STRIKE => Spell,
    BF_SPELL_EARTH_RISING => Spell,
    BF_SPELL_VOLCANIC_ERUPTION => Spell,
    BF_SPELL_RIVER_OF_LIGHTNING => Spell,
    BF_SPELL_SPEAR_OF_OROME => Spell,
    BF_SPELL_LIGHT_OF_VARDA => Spell,
    BF_SPELL_NETHER_BOLT => Spell,
    BF_SPELL_TAP_UNLIFE => Spell,
    BF_SPELL_CRUSH => Spell,
    BF_SPELL_SLEEP_EVIL => Spell,
    BF_SPELL_DISENCHANT => Spell,
    BF_SPELL_FRIGHTEN => Spell,
    BF_SPELL_VAMPIRE_STRIKE => Spell,
    BF_PRAYER_DISPEL_LIFE => Spell,
    BF_SPELL_DARK_SPEAR => Spell,
    BF_SPELL_UNLEASH_CHAOS => Spell,
    BF_SPELL_STORM_OF_DARKNESS => Spell,
    BF_SPELL_CURSE => Spell,
    BF_SPELL_WHIRLWIND_ATTACK => Spell,
    BF_SPELL_LEAP_INTO_BATTLE => Spell,
    BF_SPELL_MAIM_FOE => Spell,
    BF_SPELL_HOWL_OF_THE_DAMNED => Spell,
    BF_ROD_ELEC_BOLT => Device,
    BF_ROD_COLD_BOLT => Device,
    BF_ROD_ACID_BOLT => Device,
    BF_ROD_FIRE_BOLT => Device,
    BF_ROD_LIGHT_BEAM => Device,
    BF_ROD_DRAIN_LIFE => Device,
    BF_ROD_ELEC_BALL => Device,
    BF_ROD_COLD_BALL => Device,
    BF_ROD_ACID_BALL => Device,
    BF_ROD_FIRE_BALL => Device,
    BF_ROD_SLOW_MONSTER => Device,
    BF_ROD_SLEEP_MONSTER => Device,
    BF_ROD_UNKNOWN => Device,
    BF_STAFF_SLEEP_MONSTERS => Device,
    BF_STAFF_SLOW_MONSTERS => Device,
    BF_STAFF_DISPEL_EVIL => Device,
    BF_STAFF_POWER => Device,
    BF_STAFF_HOLINESS => Device,
    BF_WAND_UNKNOWN => Device,
    BF_WAND_MAGIC_MISSILE => Device,
    BF_WAND_ELEC_BOLT => Device,
    BF_WAND_COLD_BOLT => Device,
    BF_WAND_ACID_BOLT => Device,
    BF_WAND_FIRE_BOLT => Device,
    BF_WAND_SLOW_MONSTER => Device,
    BF_WAND_HOLD_MONSTER => Device,
    BF_WAND_CONFUSE_MONSTER => Device,
    BF_WAND_FEAR_MONSTER => Device,
    BF_WAND_ANNIHILATION => Device,
    BF_WAND_DRAIN_LIFE => Device,
    BF_WAND_LIGHT_BEAM => Device,
    BF_WAND_STINKING_CLOUD => Device,
    BF_WAND_ELEC_BALL => Device,
    BF_WAND_COLD_BALL => Device,
    BF_WAND_ACID_BALL => Device,
    BF_WAND_FIRE_BALL => Device,
    BF_WAND_WONDER => Device,
    BF_WAND_DRAGON_COLD => Device,
    BF_WAND_DRAGON_FIRE => Device,
    BF_RING_ACID => Activation,
    BF_RING_FIRE => Activation,
    BF_RING_ICE => Activation,
    BF_RING_LIGHTNING => Activation,
    BF_DRAGON_BLUE => Activation,
    BF_DRAGON_WHITE => Activation,
    BF_DRAGON_BLACK => Activation,
    BF_DRAGON_GREEN => Activation,
    BF_DRAGON_RED => Activation,
    BF_DRAGON_MULTIHUED => Activation,
    BF_DRAGON_GOLD => Activation,
    BF_DRAGON_CHAOS => Activation,
    BF_DRAGON_LAW => Activation,
    BF_DRAGON_BALANCE => Activation,
    BF_DRAGON_SHINING => Activation,
    BF_DRAGON_POWER => Activation,
    BF_ACT_FIRE_BOLT => Activation,
    BF_ACT_FIRE_BALL72 => Activation,
    BF_ACT_FIRE_BALL => Activation,
    BF_ACT_COLD_BOLT => Activation,
    BF_ACT_COLD_BALL50 => Activation,
    BF_ACT_COLD_BALL100 => Activation,
    BF_ACT_COLD_BOLT2 => Activation,
    BF_ACT_DRAIN_LIFE1 => Activation,
    BF_ACT_DRAIN_LIFE2 => Activation,
    BF_ACT_STINKING_CLOUD => Activation,
    BF_ACT_CONFUSE2 => Activation,
    BF_ACT_ARROW => Activation,
    BF_ACT_MISSILE => Activation,
    BF_ACT_SLEEPII => Activation,
    BF_ACT_ELEC_BOLT => Activation,
    BF_ACT_ACID_BOLT => Activation,
    BF_ACT_DISPEL_EVIL => Activation,
    BF_ACT_MANA_BOLT => Activation,
    BF_ACT_STAR_BALL => Activation,
    BF_ACT_STARLIGHT2 => Activation,
    BF_ACT_STARLIGHT => Activation,
    BF_ACT_MON_SLOW => Activation,
    BF_ACT_MON_CONFUSE => Activation,
    BF_ACT_SLEEP_ALL => Activation,
    BF_ACT_FEAR_MONSTER => Activation,
    BF_ACT_LIGHT_BEAM => Activation,
    BF_ACT_DRAIN_LIFE3 => Activation,
    BF_ACT_DRAIN_LIFE4 => Activation,
    BF_ACT_ELEC_BALL => Activation,
    BF_ACT_ELEC_BALL2 => Activation,
    BF_ACT_ACID_BOLT2 => Activation,
    BF_ACT_ACID_BOLT3 => Activation,
    BF_ACT_ACID_BALL => Activation,
    BF_ACT_COLD_BALL160 => Activation,
    BF_ACT_COLD_BALL2 => Activation,
    BF_ACT_FIRE_BALL2 => Activation,
    BF_ACT_FIRE_BALL200 => Activation,
    BF_ACT_FIRE_BOLT2 => Activation,
    BF_ACT_FIRE_BOLT3 => Activation,
    BF_ACT_DISPEL_EVIL60 => Activation,
    BF_ACT_DISPEL_UNDEAD => Activation,
    BF_ACT_DISPEL_ALL => Activation,
    BF_ACT_LOSSLOW => Activation,
    BF_ACT_LOSSLEEP => Activation,
    BF_ACT_LOSCONF => Activation,
    BF_ACT_WONDER => Activation,
    BF_ACT_STAFF_HOLY => Activation,
    BF_ACT_RING_ACID => Activation,
    BF_ACT_RING_FIRE => Activation,
    BF_ACT_RING_ICE => Activation,
    BF_ACT_RING_LIGHTNING => Activation,
    BF_ACT_DRAGON_BLUE => Activation,
    BF_ACT_DRAGON_GREEN => Activation,
    BF_ACT_DRAGON_RED => Activation,
    BF_ACT_DRAGON_MULTIHUED => Activation,
    BF_ACT_DRAGON_GOLD => Activation,
    BF_ACT_DRAGON_CHAOS => Activation,
    BF_ACT_DRAGON_LAW => Activation,
    BF_ACT_DRAGON_BALANCE => Activation,
    BF_ACT_DRAGON_SHINING => Activation,
    BF_ACT_DRAGON_POWER => Activation,
}

// Class IDs are game data: engine/angband/lib/gamedata/class.txt at 42cd156ac,
// declaration order Warrior, Mage, Druid, Priest, Necromancer, Paladin, Rogue,
// Ranger, Blackguard. Unknown classes use a traced neutral fallback.
pub(crate) fn class_name(class: i32) -> Option<&'static str> {
    match class {
        0 => Some("warrior"),
        1 => Some("mage"),
        2 => Some("druid"),
        3 => Some("priest"),
        4 => Some("necromancer"),
        5 => Some("paladin"),
        6 => Some("rogue"),
        7 => Some("ranger"),
        8 => Some("blackguard"),
        _ => None,
    }
}

/// The family's preference parameter. borg-reborn-rs (#520): one per family;
/// its value is the class's own (`attack_preference_<family>_permille@<class>`
/// or the class file) through `params::value`, so the class only gates which
/// classes have preferences at all.
pub(crate) fn parameter(class: i32, family: Family) -> Option<Param> {
    class_name(class)?;
    match family {
        Family::Melee => Some(Param::attack_preference_melee_permille),
        Family::Projectile => Some(Param::attack_preference_projectile_permille),
        Family::Thrown => Some(Param::attack_preference_thrown_permille),
        Family::Spell => Some(Param::attack_preference_spell_permille),
        Family::Device => Some(Param::attack_preference_device_permille),
        Family::Activation => Some(Param::attack_preference_activation_permille),
        // Waiting has no preference.
        Family::Wait => None,
    }
}

/// Floor rounding in utility units, widened before multiplication. Weights are
/// validated at initialization. Positive clamp preserves selectable tiny scores;
/// negative sentinels and zero return unchanged. No eligibility checks rerun.
pub(crate) fn adjust(raw: i32, weight: i32) -> i32 {
    if raw <= 0 || weight == 1000 { return raw; }
    ((i64::from(raw) * i64::from(weight)) / 1000).clamp(1, i64::from(i32::MAX)) as i32
}

pub(crate) struct Candidate {
    pub attack: Option<AttackKind>,
    pub parameter: Option<Param>,
    pub weight: i32,
    pub origin: &'static str,
    pub adjusted: i32,
}

pub(crate) fn evaluate(class: i32, attack_id: i32, raw: i32) -> Candidate {
    let attack = classify(attack_id);
    let parameter = attack.and_then(|a| parameter(class, a.family));
    let (weight, origin) = parameter.map_or((1000, "fixed_neutral"), params::resolved);
    Candidate {
        attack, parameter, weight, origin,
        adjusted: if attack.is_some() { adjust(raw, weight) } else { 0 },
    }
}

/// Shared by the production loop and selection tests. Strict comparison retains
/// the first equal candidate. Execution never calls this ranking adjustment.
#[derive(Default)]
pub(crate) struct Ranking {
    pub winner: Option<i32>,
    pub utility: i32,
}

impl Ranking {
    pub(crate) fn consider(&mut self, id: i32, candidate: &Candidate) -> bool {
        if candidate.adjusted <= self.utility { return false; }
        self.winner = Some(id);
        self.utility = candidate.adjusted;
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_declared_attack_is_mapped_exactly_once() {
        let header = include_str!("borg_headers/borg_fight_attack_h.rs");
        let declared: std::collections::BTreeMap<_, _> = header.lines()
            .filter_map(|line| {
                let line = line.strip_prefix("pub const BF_")?;
                let (name, rest) = line.split_once(':')?;
                if name == "MAX" { return None; }
                let id = rest.split_once(" = ")?.1.trim_end_matches(';').parse::<u32>().ok()?;
                Some((id, format!("BF_{name}")))
            }).collect();
        assert_eq!(INVENTORY.len(), BF_MAX as usize);
        assert_eq!(declared.len(), INVENTORY.len());
        let mapped: std::collections::BTreeMap<_, _> = INVENTORY.iter()
            .map(|(id, name)| (*id, name.to_string())).collect();
        assert_eq!(mapped.len(), INVENTORY.len());
        assert_eq!(declared, mapped);
        for (id, name) in INVENTORY {
            assert_eq!(classify(*id as i32).unwrap().name, *name);
        }
        for id in [-1, BF_MAX as i32, i32::MAX] { assert!(classify(id).is_none()); }
        let counts: Vec<_> = [Family::Melee, Family::Projectile, Family::Thrown,
            Family::Spell, Family::Device, Family::Activation, Family::Wait].iter()
            .map(|family| INVENTORY.iter().filter(|(id, _)| classify(*id as i32).unwrap().family == *family).count()).collect();
        assert_eq!(counts, vec![1, 1, 1, 50, 39, 77, 1]);
    }

    #[test]
    fn invocation_family_includes_hybrids_controls_and_emergencies() {
        for id in [BF_SPELL_WHIRLWIND_ATTACK, BF_SPELL_LEAP_INTO_BATTLE,
            BF_SPELL_MAIM_FOE, BF_SPELL_MAGIC_MISSILE_RESERVE, BF_SPELL_SLEEP_III] {
            assert_eq!(classify(id as i32).unwrap().family, Family::Spell);
        }
        assert_eq!(classify(BF_WAND_HOLD_MONSTER as i32).unwrap().family, Family::Device);
        assert_eq!(classify(BF_ACT_ARROW as i32).unwrap().family, Family::Activation);
        assert_eq!(classify(BF_RING_FIRE as i32).unwrap().family, Family::Activation);
        assert_eq!(classify(BF_DRAGON_BLUE as i32).unwrap().family, Family::Activation);
    }

    #[test]
    fn arithmetic_vectors_preserve_sentinels_tiny_values_and_neutral_identity() {
        for (raw, weight, expected) in [
            (17, 1500, 25), (17, 1600, 27), (25, 700, 17), (1, 500, 1),
            (3, 500, 1), (0, 2000, 0), (-999, 2000, -999),
            (i32::MIN, 2000, i32::MIN), (i32::MAX, 2000, i32::MAX),
            (i32::MAX, 500, i32::MAX / 2),
        ] { assert_eq!(adjust(raw, weight), expected); }
        for raw in [i32::MIN, -999, -1, 0, 1, 17, i32::MAX] {
            assert_eq!(adjust(raw, 1000), raw);
        }
    }

    #[test]
    fn all_classes_share_six_distinct_neutral_parameters() {
        // #520: one parameter per family; a class's own value comes from
        // `<name>@<class>` or its class file.
        let mut names = std::collections::BTreeSet::new();
        for class in 0..9 {
            assert!(class_name(class).is_some());
            for family in [Family::Melee, Family::Projectile, Family::Thrown,
                Family::Spell, Family::Device, Family::Activation] {
                let p = parameter(class, family).unwrap();
                let spec = &params::SPECS[p as usize];
                names.insert(spec.name);
                assert_eq!(spec.name, format!("attack_preference_{}_permille", family.name()));
                assert_eq!((spec.default, spec.min, spec.max, spec.unit), (1000, 500, 2000, "utility_permille"));
            }
            assert!(parameter(class, Family::Wait).is_none());
        }
        assert_eq!(names.len(), 6);
        assert!(class_name(9).is_none());
        let candidate = evaluate(9, BF_THRUST as i32, 25);
        assert_eq!(candidate.adjusted, 25);
        assert!(candidate.parameter.is_none());
    }

    #[test]
    fn class_and_family_preferences_change_only_the_final_ranking() {
        // Utility fixtures, not game replays. Exercise the same classification,
        // parameter identity, arithmetic and comparison as the production loop.
        let cases = [
            (1, BF_THRUST, 25, BF_SPELL_MAGIC_MISSILE, 17, Family::Spell, 1600, BF_SPELL_MAGIC_MISSILE),
            (1, BF_SPELL_MAGIC_MISSILE, 17, BF_WAND_MAGIC_MISSILE, 14, Family::Device, 1500, BF_WAND_MAGIC_MISSILE),
            (0, BF_THRUST, 20, BF_WAND_MAGIC_MISSILE, 25, Family::Melee, 1500, BF_THRUST),
            (7, BF_THRUST, 20, BF_LAUNCH, 16, Family::Projectile, 1500, BF_LAUNCH),
            (8, BF_THRUST, 25, BF_SPELL_LEAP_INTO_BATTLE, 17, Family::Spell, 1600, BF_SPELL_LEAP_INTO_BATTLE),
            (1, BF_THRUST, 12, BF_WAND_HOLD_MONSTER, 10, Family::Device, 1500, BF_WAND_HOLD_MONSTER),
            (0, BF_REST, 1, BF_THRUST, 1, Family::Melee, 500, BF_REST),
            (1, BF_REST, 1, BF_WAND_MAGIC_MISSILE, 0, Family::Device, 2000, BF_REST),
        ];
        for (class, a, raw_a, b, raw_b, family, weight, expected) in cases {
            let selected_param = parameter(class, family).unwrap();
            let mut ranking = Ranking::default();
            for (id, raw) in [(a, raw_a), (b, raw_b)] {
                let mut candidate = evaluate(class, id as i32, raw);
                if candidate.parameter.map(|p| p as usize) == Some(selected_param as usize) {
                    candidate.adjusted = adjust(raw, weight);
                }
                ranking.consider(id as i32, &candidate);
            }
            assert_eq!(ranking.winner, Some(expected as i32));
        }
    }

    #[test]
    fn ranking_preserves_first_ties_and_never_promotes_invalid_results() {
        let mut ranking = Ranking::default();
        let melee = evaluate(1, BF_THRUST as i32, 25);
        assert!(ranking.consider(BF_THRUST as i32, &melee));
        let mut spell = evaluate(1, BF_SPELL_MAGIC_MISSILE as i32, 17);
        spell.adjusted = adjust(17, 1500);
        assert!(!ranking.consider(BF_SPELL_MAGIC_MISSILE as i32, &spell));
        spell.adjusted = adjust(17, 1600);
        assert!(ranking.consider(BF_SPELL_MAGIC_MISSILE as i32, &spell));
        assert_eq!(ranking.winner, Some(BF_SPELL_MAGIC_MISSILE as i32));
        for raw in [0, -1, -999] {
            let mut invalid = evaluate(1, BF_WAND_MAGIC_MISSILE as i32, raw);
            invalid.adjusted = adjust(raw, 2000);
            assert!(!Ranking::default().consider(BF_WAND_MAGIC_MISSILE as i32, &invalid));
        }
        assert!(!Ranking::default().consider(999, &evaluate(1, 999, 100)));
    }
}
