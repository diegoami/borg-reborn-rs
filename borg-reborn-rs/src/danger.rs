//! L4 threat: the one assessment every consumer reads (T4.2, #119).
//!
//! M3's placeholder was the C Borg's danger *sum*, each blow at full dice
//! (`borg-danger.c:borg_danger:2824-2865`). M4 replaces it with an estimate
//! in hit points over a declared horizon:
//!
//! * **expected** damage: blow dice rolled, the engine's hit chance
//!   (`mon-attack.c:chance_of_monster_hit_base`, `player-attack.c:hit_chance`)
//!   and armour (`mon-attack.c:adjust_dam_armor`), multiplied by the monster's
//!   attack rounds from the energy model (`borg-danger.c:2361-2395`);
//! * **worst** damage: every blow hits at its largest roll, for the same
//!   rounds;
//! * **p_death**: the chance the total reaches the character's hit points
//!   (exact convolution for small fights, a declared normal approximation
//!   for large ones);
//! * **disabling**: the chance of fear, blindness, confusion or paralysis
//!   from a blow or a spell, kept apart from hit points;
//! * **economic**: theft, destruction and drains as expected events, never
//!   added to lethal risk (ARCHITECTURE L4; GOALS principle 4).
//!
//! Ranged attacks use the C Borg's own estimates as a floor: a breath is
//! `hp / 3` (capped), a shot/arrow/bolt is `((power / 8) + 1) * 5/6/7`
//! (`borg-danger.c:619-760`; the full `projection.txt` divisors and
//! `mon-spell.c` effect dice are not in `Knowledge` yet, so uncommon breaths
//! use `hp / 6`, capped, and are marked as estimates). Summoners and
//! breeders carry growth factors of our own, *declared* (1.5, 1.8, 1.5):
//! the C Borg's code at the cited lines computes larger ones (×2.5, ×2.8,
//! ×2.3, ×2.2), and its comments say +50%, +80%, +30% (#163; see
//! [`breeder_factor`] and the summoner scale in `ranged_threat`).
//! The engine's exact spell effects, breath divisors and summon lists are
//! docs/ENGINE-READS.md's open item and the calibration report's job (#124).
//!
//! One risk number is `expected + aversion * (worst - expected)`; the
//! aversion is a parameter (`risk_aversion_pct`), and paths, rest, retreat
//! and combat all pass it. A remembered-unseen monster keeps its last
//! position and decays with `Track::confidence` (REDESIGN-LESSONS L11).
//!
//! Fear is not a damage term: the engine refuses the attack outright
//! (`mon-blows.c`, "too afraid to attack"), so `can_attack` is false and the
//! tactical layer cannot choose one. Paralysis is the same for every action.

use serde::{Deserialize, Serialize};

use crate::knowledge::{Blow, Knowledge, MonsterRace};
use crate::world::{Track, World};

/// `MON_TMD_*` indices (`list-mon-timed.h`).
mod mon_tmd {
    pub const STUN: usize = 1;
    pub const CONF: usize = 2;
}

/// `TMD_*` indices (`list-player-timed.h`: the enum counts its `TMD(...)`
/// lines from 0).
mod tmd {
    pub const BLIND: usize = 2;
    pub const PARALYZED: usize = 3;
    pub const CONFUSED: usize = 4;
    pub const AFRAID: usize = 5;
    pub const AMNESIA: usize = 25;
}

/// `RF_*` flag numbers. The enum starts at `RF_NONE = 0`, and `FLAG_START`
/// is 1: count the `RF(...)` lines of `list-mon-race-flags.h`.
mod rf {
    pub const MULTIPLY: usize = 17;
    pub const NEVER_BLOW: usize = 20;
    pub const NEVER_MOVE: usize = 21;
}

/// `RSF_*` flag numbers (`list-mon-spells.h`), the subset the estimate uses.
mod rsf {
    pub const SHRIEK: usize = 1;
    pub const WHIP: usize = 2;
    pub const SPIT: usize = 3;
    pub const SHOT: usize = 4;
    pub const ARROW: usize = 5;
    pub const BOLT: usize = 6;
    pub const BR_ACID: usize = 7;
    pub const BR_ELEC: usize = 8;
    pub const BR_FIRE: usize = 9;
    pub const BR_COLD: usize = 10;
    pub const BR_POIS: usize = 11;
    pub const BO_ACID: usize = 45;
    pub const BO_ELEC: usize = 46;
    pub const BO_FIRE: usize = 47;
    pub const BO_COLD: usize = 48;
    pub const BO_POIS: usize = 49;
    pub const MISSILE: usize = 55;
    pub const BE_ELEC: usize = 56;
    pub const SCARE: usize = 58;
    pub const BLIND: usize = 59;
    pub const CONF: usize = 60;
    pub const SLOW: usize = 61;
    pub const HOLD: usize = 62;
    pub const HEAL: usize = 64;
    pub const S_KIN: usize = 76;
    pub const S_MONSTER: usize = 78;
    pub const S_MONSTERS: usize = 79;
    pub const S_ANIMAL: usize = 80;
    pub const S_SPIDER: usize = 81;
    pub const S_HOUND: usize = 82;
    pub const S_HYDRA: usize = 83;
    pub const S_AINU: usize = 84;
    pub const S_DEMON: usize = 85;
    pub const S_UNDEAD: usize = 86;
    pub const S_DRAGON: usize = 87;
    pub const S_HI_UNDEAD: usize = 88;
    pub const S_HI_DRAGON: usize = 89;
    pub const S_WRAITH: usize = 90;
    pub const S_UNIQUE: usize = 91;
}

/// `OF_*` flag numbers (`list-object-flags.h`: a flag's number is its line
/// less sixteen).
mod of {
    pub const PROT_FEAR: usize = 6;
    pub const PROT_BLIND: usize = 7;
    pub const PROT_CONF: usize = 8;
    pub const FREE_ACT: usize = 15;
}

/// Element indices (`list-elements.h` order).
mod elem {
    pub const ACID: usize = 0;
    pub const ELEC: usize = 1;
    pub const FIRE: usize = 2;
    pub const COLD: usize = 3;
    pub const POIS: usize = 4;
}

/// The declared default aversion when a caller has no parameter table
/// (tests, the `danger()` compatibility wrapper). Strategy always passes
/// `Params::risk_aversion_pct`.
pub const DEFAULT_RISK_AVERSION_PCT: i32 = 50;

/// The highest danger a path cost can pay. HP risk itself is uncapped; this
/// only keeps a distance map's `u32` costs sane (the C Borg caps its sum at
/// 2000, `borg-danger.c:2824-2865`).
const PATH_CAP: f64 = 2000.0;

fn has(flags: &[u8], flag: usize) -> bool {
    flags
        .get(flag.wrapping_sub(1) / 8)
        .is_some_and(|b| b & (1 << ((flag - 1) % 8)) != 0)
}

/// One monster's contribution to one grid's assessment.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct MonsterThreat {
    pub index: i32,
    pub expected: f64,
    pub worst: f64,
    pub p_death: f64,
    /// Expected attack rounds in the horizon.
    pub rounds: f64,
    /// Chance of at least one fear/blind/confuse/paralyse from this monster.
    pub disabling: f64,
    /// Chance of at least one summon or shriek from this monster.
    pub summoning: f64,
    /// Theft, destruction or drain events, not hit points.
    pub economic: f64,
    /// Whether the ranged estimate contributed (for the trace and tests).
    pub ranged: bool,
    /// The track's confidence (1.0 in view).
    pub confidence: f64,
}

/// The whole grid's assessment for a horizon and risk tolerance.
#[derive(Clone, Debug, PartialEq)]
pub struct Assessment {
    pub horizon: i32,
    pub expected: f64,
    pub worst: f64,
    pub p_death: f64,
    pub disabling: f64,
    pub summoning: f64,
    pub economic: f64,
    pub per_monster: Vec<MonsterThreat>,
}

impl MonsterThreat {
    /// This monster's risk in hit points under the same tolerance.
    pub fn risk(&self, aversion_pct: i32) -> f64 {
        let a = f64::from(aversion_pct.clamp(0, 100)) / 100.0;
        self.expected + a * (self.worst - self.expected).max(0.0)
    }
}

/// What the decision trace records for later calibration: the grid's
/// numbers, and one compact line per monster that contributes to any of
/// them (T4.7, #124: calibration by monster, range, speed, group and
/// status). Monsters that add nothing (out of reach, forgotten) are left
/// out, so a quiet think's line stays small.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ThreatSummary {
    pub horizon: i32,
    pub expected: f64,
    pub worst: f64,
    pub p_death: f64,
    pub disabling: f64,
    pub summoning: f64,
    pub economic: f64,
    /// The contributing monsters; absent in traces written before T4.7.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub sources: Vec<SourceSummary>,
}

/// One monster's share of a think's assessment, as the trace records it.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SourceSummary {
    /// The game's monster index.
    pub index: i32,
    /// `monster.txt` race index and name.
    pub race: u32,
    pub name: String,
    /// Chebyshev distance from the assessed grid.
    pub dist: i32,
    /// The monster's own speed (110 normal), as last seen.
    pub speed: u8,
    pub awake: bool,
    pub in_view: bool,
    pub confused: bool,
    pub stunned: bool,
    /// Tracked monsters of the same race that also contribute.
    pub group: u32,
    pub confidence: f64,
    pub rounds: f64,
    pub expected: f64,
    pub worst: f64,
    /// Chance of at least one fear, blindness, confusion or paralysis.
    pub disabling: f64,
    pub ranged: bool,
}

impl Assessment {
    pub fn summary(&self) -> ThreatSummary {
        ThreatSummary {
            horizon: self.horizon,
            expected: self.expected,
            worst: self.worst,
            p_death: self.p_death,
            disabling: self.disabling,
            summoning: self.summoning,
            economic: self.economic,
            sources: Vec::new(),
        }
    }

    /// [`Self::summary`] with the contributing monsters named, for the
    /// trace. `x, y` is the grid the assessment was made for.
    pub fn summary_detailed(&self, w: &World, k: &Knowledge, x: i32, y: i32) -> ThreatSummary {
        // Any term counts: a floating eye's gaze has no damage dice and
        // only a disabling chance (`monster.txt`: `blow:GAZE:PARALYZE`).
        let contributes = |m: &MonsterThreat| {
            m.expected > 0.0
                || m.worst > 0.0
                || m.disabling > 0.0
                || m.summoning > 0.0
                || m.economic > 0.0
        };
        let track = |index: i32| w.level.monsters.iter().find(|t| t.index == index);
        let mut sources = Vec::new();
        for m in self.per_monster.iter().filter(|m| contributes(m)) {
            let Some(t) = track(m.index) else { continue };
            let group = self
                .per_monster
                .iter()
                .filter(|o| contributes(o))
                .filter_map(|o| track(o.index))
                .filter(|o| o.race == t.race)
                .count() as u32;
            sources.push(SourceSummary {
                index: m.index,
                race: t.race,
                name: k.race(t.race).map_or_else(String::new, |r| r.name.clone()),
                dist: (t.x - x).abs().max((t.y - y).abs()),
                speed: t.speed,
                awake: t.awake(),
                in_view: t.in_view,
                confused: t.timed[mon_tmd::CONF] > 0,
                stunned: t.timed[mon_tmd::STUN] > 0,
                group,
                confidence: m.confidence,
                rounds: m.rounds,
                expected: m.expected,
                worst: m.worst,
                disabling: m.disabling,
                ranged: m.ranged,
            });
        }
        ThreatSummary {
            sources,
            ..self.summary()
        }
    }

    /// The one risk number, in hit points: expected, plus `aversion_pct` of
    /// the gap to the worst case. 0 is expected only; 100 is worst only.
    pub fn risk(&self, aversion_pct: i32) -> f64 {
        let a = f64::from(aversion_pct.clamp(0, 100)) / 100.0;
        self.expected + a * (self.worst - self.expected).max(0.0)
    }
}

/// Whether the character can read a scroll this think: the engine's
/// `player_can_read` (`player-util.c:1166-1197`) refuses it when blind,
/// with no light (`no_light`: the character's grid is not seen,
/// `cave-view.c:932-935`, which the observation carries as `in_view`,
/// `cave-map.c:119-122`), confused or amnesiac. An action the engine
/// refuses is never offered (#137: a confused rogue re-chose a refused
/// Phase Door for forty turns, D1667).
pub fn can_read(w: &World) -> bool {
    let Some(s) = w.character.status.as_ref() else {
        return true;
    };
    let timed = |i: usize| s.timed.get(i).copied().unwrap_or(0);
    let lit = w.level.at(s.x, s.y).is_some_and(|g| g.seen == Some(w.turn));
    lit && timed(tmd::BLIND) <= 0 && timed(tmd::CONFUSED) <= 0 && timed(tmd::AMNESIA) <= 0
}

/// Whether the character can attack this think. Fear refuses the attack in
/// the engine (`mon-blows.c`, "too afraid to attack"); paralysis refuses
/// every action.
pub fn can_attack(w: &World) -> bool {
    let Some(s) = w.character.status.as_ref() else {
        return true;
    };
    let timed = |i: usize| s.timed.get(i).copied().unwrap_or(0);
    timed(tmd::AFRAID) <= 0 && timed(tmd::PARALYZED) <= 0
}

/// Whether the character can act at all (paralysis).
pub fn can_act(w: &World) -> bool {
    w.character
        .status
        .as_ref()
        .is_none_or(|s| s.timed.get(tmd::PARALYZED).copied().unwrap_or(0) <= 0)
}

/// The risk of standing at `(x, y)` for `c` player turns, at the default
/// aversion. This is the compatibility wrapper for callers that predate the
/// parameter table; strategy and paths pass their own tolerance.
pub fn danger(w: &World, k: &Knowledge, x: i32, y: i32, c: i32) -> u32 {
    danger_with(w, k, x, y, c, DEFAULT_RISK_AVERSION_PCT)
}

/// As [`danger`], with the caller's risk tolerance.
pub fn danger_with(w: &World, k: &Knowledge, x: i32, y: i32, c: i32, aversion_pct: i32) -> u32 {
    assess(w, k, x, y, c, aversion_pct)
        .risk(aversion_pct)
        .min(PATH_CAP)
        .round() as u32
}

/// The full assessment of one grid.
pub fn assess(
    w: &World,
    k: &Knowledge,
    x: i32,
    y: i32,
    horizon: i32,
    aversion_pct: i32,
) -> Assessment {
    let mut out = Assessment {
        horizon,
        expected: 0.0,
        worst: 0.0,
        p_death: 0.0,
        disabling: 0.0,
        summoning: 0.0,
        economic: 0.0,
        per_monster: Vec::new(),
    };
    for t in &w.level.monsters {
        let one = one_threat(w, k, t, x, y, horizon);
        out.expected += one.expected;
        out.worst += one.worst;
        out.disabling = 1.0 - (1.0 - out.disabling) * (1.0 - one.disabling);
        out.summoning = 1.0 - (1.0 - out.summoning) * (1.0 - one.summoning);
        out.economic += one.economic;
        // The grid's death probability is not the sum of each monster's:
        // the first killing blow already kills. Combine the survival
        // probabilities instead.
        let survive = 1.0 - out.p_death;
        out.p_death = 1.0 - survive * (1.0 - one.p_death);
        out.per_monster.push(one);
    }
    // The aversion is applied by `risk` and by every consumer, not baked
    // into the expected/worst pair.
    let _ = aversion_pct;
    out
}

/// One monster's threat at `(x, y)`.
pub fn one(w: &World, k: &Knowledge, t: &Track, x: i32, y: i32, c: i32) -> u32 {
    one_threat(w, k, t, x, y, c).expected.round() as u32
}

/// The detailed per-monster threat.
pub fn one_threat(w: &World, k: &Knowledge, t: &Track, x: i32, y: i32, c: i32) -> MonsterThreat {
    let Some(race) = k.race(t.race) else {
        // Unknown race: the C Borg's ghost value, kept as a conservative
        // default with no dice to inspect.
        return MonsterThreat {
            index: t.index,
            expected: 100.0,
            worst: 100.0,
            p_death: 0.0,
            rounds: 1.0,
            disabling: 0.0,
            summoning: 0.0,
            economic: 0.0,
            ranged: false,
            confidence: t.confidence(w.turn, k),
        };
    };
    let d = (t.x - x).abs().max((t.y - y).abs());
    let confidence = t.confidence(w.turn, k);
    if d > 20 || confidence <= 0.0 {
        return MonsterThreat {
            index: t.index,
            confidence,
            ..Default::default()
        };
    }
    let d = d.max(1);
    let player = w
        .character
        .status
        .as_ref()
        .map_or((110, 1, 20), |s| (s.speed, s.level, s.max_hp));
    let mut monster_speed = i32::from(t.speed);
    if player.2 < 20 && w.level.depth > 0 {
        monster_speed += 3; // borg-danger.c:2357-2359
    }
    let slots = attack_slots(k, player.0, monster_speed, c);
    let reach = if has(&race.flags, rf::NEVER_MOVE) && d > 1 {
        f64::INFINITY
    } else {
        f64::from(d - 1)
    };
    let rounds = (slots - reach).max(0.0);
    let awake = t.awake();
    let mut melee = melee_threat(w, k, race, t, rounds, awake);
    let growth = breeder_factor(w, race);
    melee.expected *= growth;
    melee.worst *= growth;
    melee.confidence = confidence;
    melee.expected *= confidence;
    melee.worst *= confidence;
    melee.disabling *= confidence;
    melee.summoning *= confidence;
    melee.economic *= confidence;
    // The chance of death is the chance it is still there times the
    // chance it kills if it is (#157).
    melee.p_death *= confidence;

    // The ranged half is scaled the same way (#157): a caster remembered
    // out of view at confidence 0.00015 kept its full estimate before, and
    // a rogue walked in a loop for 75,000 turns around three of them.
    let mut ranged = ranged_threat(w, race, slots, d, x, y, t);
    ranged.expected *= confidence;
    ranged.worst *= confidence;
    ranged.p_death *= confidence;
    ranged.disabling *= confidence;
    ranged.summoning *= confidence;
    ranged.economic *= confidence;
    if rounds <= 0.0 && !ranged.ranged {
        return MonsterThreat {
            index: t.index,
            confidence,
            ..Default::default()
        };
    }
    MonsterThreat {
        index: t.index,
        expected: melee.expected + ranged.expected,
        worst: melee.worst + ranged.worst,
        p_death: combine_death(melee.p_death, ranged.p_death),
        rounds,
        disabling: 1.0 - (1.0 - melee.disabling) * (1.0 - ranged.disabling),
        summoning: 1.0 - (1.0 - melee.summoning) * (1.0 - ranged.summoning),
        economic: melee.economic + ranged.economic,
        ranged: ranged.ranged,
        confidence,
    }
}

fn attack_slots(k: &Knowledge, player_speed: i32, monster_speed: i32, c: i32) -> f64 {
    let e = k.turn_energy(player_speed).max(1);
    let player_turns = (100 + e - 1) / e;
    let e = k.turn_energy(monster_speed.clamp(0, 199));
    let slots = f64::from(c) * f64::from(player_turns * e) / 100.0;
    slots.max(0.0)
}

fn combine_death(a: f64, b: f64) -> f64 {
    1.0 - (1.0 - a.clamp(0.0, 1.0)) * (1.0 - b.clamp(0.0, 1.0))
}

/// Melee damage over `rounds`, with the engine's hit and armour model.
fn melee_threat(
    w: &World,
    _k: &Knowledge,
    race: &MonsterRace,
    t: &Track,
    rounds: f64,
    awake: bool,
) -> MonsterThreat {
    let mut out = MonsterThreat::default();
    if has(&race.flags, rf::NEVER_BLOW) {
        return out;
    }
    let ac = w.character.status.as_ref().map_or(0, |s| s.known_ac).max(0);
    let stunned = t.timed[mon_tmd::STUN] > 0;
    let mut round_pmf = vec![1.0];
    let mut expected_per_round = 0.0;
    let mut worst_per_round = 0.0;
    let mut status_per_blow = 0.0;
    let mut economic_per_blow = 0.0;
    for blow in &race.blows {
        let p_hit = hit_chance(race, blow, ac, stunned);
        let (pmf, mean, max) = blow_pmf(blow, ac, stunned);
        if is_status(blow.effect.as_str()) && !protected(w, blow.effect.as_str()) {
            status_per_blow = 1.0 - (1.0 - status_per_blow) * (1.0 - p_hit);
        }
        if is_economic(blow.effect.as_str()) {
            economic_per_blow += p_hit;
        }
        // Combine this blow into the round distribution: a miss is index 0.
        round_pmf = convolve(&round_pmf, &hit_pmf(&pmf, p_hit), usize::MAX);
        expected_per_round += p_hit * mean;
        worst_per_round += max;
    }
    // The rounds it actually spends attacking. Sleeping: it must wake
    // first; the C Borg scales by sleepiness (`:2439-2450`), here half,
    // declared. Confused: half, the C Borg's (`borg-danger.c:2490-2493`).
    // Every term reads the same effective rounds (#165: before, confusion
    // scaled only expected and worst, and sleep left `disabling` whole).
    // Stun is in the hit chance and the dice above, as the engine applies it.
    let mut share = 1.0;
    if !awake {
        share *= 0.5;
    }
    if t.timed[mon_tmd::CONF] > 0 {
        share *= 0.5;
    }
    let attacking = rounds * share;
    out.expected = expected_per_round * attacking;
    out.worst = worst_per_round * attacking;
    out.p_death = death_probability(&round_pmf, attacking, hp_of(w));
    out.disabling =
        (1.0 - (1.0 - status_per_blow).powf(attacking * race.blows.len() as f64)).clamp(0.0, 1.0);
    out.economic = economic_per_blow * attacking;
    out.rounds = rounds;
    out
}

fn hp_of(w: &World) -> i32 {
    // Current hit points, not the maximum: a character at 3/49 is in
    // danger from a hit that would be trivial at full health.
    w.character.status.as_ref().map_or(1, |s| s.hp.max(1))
}

fn protected(w: &World, effect: &str) -> bool {
    let flag = |f: usize| has(&w.character.gear.flags, f);
    match effect {
        "BLIND" => flag(of::PROT_BLIND),
        "CONFUSE" => flag(of::PROT_CONF),
        "TERRIFY" => flag(of::PROT_FEAR),
        "PARALYZE" => flag(of::FREE_ACT),
        _ => false,
    }
}

fn is_status(effect: &str) -> bool {
    matches!(effect, "BLIND" | "CONFUSE" | "TERRIFY" | "PARALYZE")
}

fn is_economic(effect: &str) -> bool {
    matches!(
        effect,
        "EAT_GOLD"
            | "EAT_ITEM"
            | "EAT_FOOD"
            | "EAT_LIGHT"
            | "DESTROY_ITEM"
            | "DESTROY_ARMOR"
            | "DESTROY_WEAPON"
            | "DRAIN_MANA"
            | "DRAIN_CHARGE"
            | "UN_POWER"
            | "UN_STAT"
    )
}

/// The engine's chance for one blow to hit the player
/// (`mon-attack.c:chance_of_monster_hit_base`, `player-attack.c:hit_chance`).
fn hit_chance(race: &MonsterRace, blow: &Blow, ac: i32, stunned: bool) -> f64 {
    let mut to_hit = race.level.max(1) * 3 + blow.power;
    // A stunned monster's accuracy falls by STUN_HIT_REDUCTION, 25%
    // (`mon-attack.c:chance_of_monster_hit:353-356`, `mon-timed.h:26`; #165).
    if stunned {
        to_hit = to_hit * (100 - 25) / 100;
    }
    to_hit = to_hit.max(9);
    let numerator = (to_hit - ac * 2 / 3).max(0);
    let base = f64::from(numerator) / f64::from(to_hit);
    0.12 + 0.83 * base
}

/// A blow's damage before the hit roll: the dice, minus armour the way
/// `adjust_dam_armor` does (`mon-attack.c:537-541`). Returns (pmf, mean,
/// max).
fn blow_pmf(blow: &Blow, ac: i32, stunned: bool) -> (Vec<f64>, f64, f64) {
    // A stunned monster's rolled damage falls by STUN_DAM_REDUCTION, 25%,
    // before armour (`mon-attack.c:621-624`, integer arithmetic; #165).
    let roll = |d: usize| if stunned { d * (100 - 25) / 100 } else { d };
    if !matches!(
        blow.effect.as_str(),
        "HURT" | "SHATTER" | "POISON" | "ACID" | "ELEC" | "FIRE" | "COLD"
    ) {
        // Status and economic blows do no hit-point damage; unknown effects
        // keep their dice as the honest floor.
        if is_status(blow.effect.as_str()) || is_economic(blow.effect.as_str()) {
            return (vec![1.0], 0.0, 0.0);
        }
    }
    let dice = blow.dice;
    let dice_n = dice.dice.max(0) as usize;
    let sides = dice.sides.max(0) as usize;
    let base = dice.base.max(0) as usize;
    if dice_n == 0 || sides == 0 {
        let d = reduce(roll(base) as f64, ac);
        let mut out = vec![0.0; d as usize + 1];
        out[d as usize] = 1.0;
        return (out, d, d);
    }
    if dice_n * sides > 64 {
        // Too wide to enumerate per assessment: a two-point model, declared.
        // All the mass sits on 0 and on the largest roll, weighted so the
        // mean is kept: P(max) = mean / max. It is a distribution (it sums
        // to 1), keeps `expected` exact and overstates the spread, which is
        // conservative for `p_death` (#164: it used to return `[0, mean]`,
        // mass `mean` at damage 1, which is not a distribution). The widest
        // blow in `monster.txt` is 20d10, so the vector stays short.
        let mut mean = (dice_n as f64) * ((sides + 1) as f64) / 2.0 + base as f64;
        if stunned {
            mean *= 0.75;
        }
        let max = roll(dice_n * sides + base) as f64;
        let (mean, max) = (reduce(mean, ac), reduce(max, ac));
        let top = max.round().max(1.0) as usize;
        let p_top = if max > 0.0 {
            (mean / max).clamp(0.0, 1.0)
        } else {
            0.0
        };
        let mut out = vec![0.0; top + 1];
        out[0] = 1.0 - p_top;
        out[top] += p_top;
        return (out, mean, max);
    }
    // Enumerate the dice by dynamic programming, then fold in the base.
    let mut pmf = vec![0.0; dice_n * sides + 1];
    pmf[0] = 1.0;
    for _ in 0..dice_n {
        let mut next = vec![0.0; pmf.len()];
        for (i, p) in pmf.iter().enumerate() {
            if *p == 0.0 {
                continue;
            }
            for s in 1..=sides {
                next[i + s] += p / sides as f64;
            }
        }
        pmf = next;
    }
    let max = pmf.len() - 1 + base;
    let mut out = vec![0.0; max + 1];
    let mut mean = 0.0;
    let mut max_dam = 0.0f64;
    for (i, p) in pmf.iter().enumerate() {
        if *p == 0.0 {
            continue;
        }
        let d = reduce(roll(i + base) as f64, ac);
        out[d as usize] += *p;
        mean += *p * d;
        max_dam = max_dam.max(d);
    }
    (out, mean, max_dam)
}

fn reduce(damage: f64, ac: i32) -> f64 {
    let ac = ac.clamp(0, 240) as f64;
    (damage - damage * ac / 400.0).max(0.0).round()
}

fn hit_pmf(pmf: &[f64], p_hit: f64) -> Vec<f64> {
    let mut out = vec![0.0; pmf.len().max(1)];
    out[0] += 1.0 - p_hit;
    for (i, p) in pmf.iter().enumerate() {
        if i < out.len() {
            out[i] += p * p_hit;
        }
    }
    out
}

/// Convolve, optionally only up to `limit` (index `limit` means "at least").
fn convolve(a: &[f64], b: &[f64], limit: usize) -> Vec<f64> {
    let n = if limit == usize::MAX {
        a.len() + b.len() - 1
    } else {
        limit + 1
    };
    let mut out = vec![0.0; n.max(1)];
    for (i, pa) in a.iter().enumerate() {
        if *pa == 0.0 {
            continue;
        }
        for (j, pb) in b.iter().enumerate() {
            if *pb == 0.0 {
                continue;
            }
            let k = (i + j).min(n - 1);
            out[k] += pa * pb;
        }
    }
    out
}

/// Probability that `rounds` rounds of `pmf` deal at least `hp`.
fn death_probability(pmf: &[f64], rounds: f64, hp: i32) -> f64 {
    let hp = hp.max(1) as usize;
    let n = rounds.floor() as usize;
    if n == 0 {
        return 0.0;
    }
    if n <= 12 && hp <= 80 {
        let mut dist = vec![0.0; hp + 1];
        dist[0] = 1.0;
        for _ in 0..n {
            dist = convolve(&dist, pmf, hp);
        }
        return dist[hp].clamp(0.0, 1.0);
    }
    // Large fight: normal approximation with a continuity correction.
    let mean = pmf
        .iter()
        .enumerate()
        .map(|(d, p)| d as f64 * p)
        .sum::<f64>()
        * n as f64;
    let var = pmf
        .iter()
        .enumerate()
        .map(|(d, p)| (d as f64 - mean / n as f64).powi(2) * p)
        .sum::<f64>()
        * n as f64;
    if var <= 0.0 {
        return if mean >= hp as f64 { 1.0 } else { 0.0 };
    }
    let z = (hp as f64 - 0.5 - mean) / var.sqrt();
    0.5 * erfc(z / std::f64::consts::SQRT_2)
}

/// Abramowitz-Stegun 7.1.26, enough for a declaration, not a statistic.
fn erfc(x: f64) -> f64 {
    let z = x.abs();
    let t = 1.0 / (1.0 + 0.5 * z);
    let y = t
        * (-z * z - 1.265_512_23
            + t * (1.000_023_68
                + t * (0.374_091_96
                    + t * (0.096_784_18
                        + t * (-0.186_288_06
                            + t * (0.278_868_07
                                + t * (-1.135_203_98
                                    + t * (1.488_515_87
                                        + t * (-0.822_152_23 + t * 0.170_872_77)))))))))
            .exp();
    if x >= 0.0 {
        y
    } else {
        2.0 - y
    }
}

fn line_clear(w: &World, x0: i32, y0: i32, x1: i32, y1: i32) -> bool {
    // Bresenham over the remembered level; walls block. The engine's
    // projectability is more subtle (permissive corners); this is the
    // declared conservative approximation until docs/ENGINE-READS lists it.
    let (dx, dy) = ((x1 - x0).abs(), (y1 - y0).abs());
    let (sx, sy) = (if x0 < x1 { 1 } else { -1 }, if y0 < y1 { 1 } else { -1 });
    let (mut x, mut y, mut err) = (x0, y0, dx - dy);
    loop {
        if (x, y) != (x0, y0) && (x, y) != (x1, y1) {
            if let Some(g) = w.level.at(x, y) {
                if matches!(
                    g.feat,
                    21 | crate::borg::feat::CLOSED | crate::borg::feat::NONE
                ) {
                    return false;
                }
            } else {
                return false;
            }
        }
        if x == x1 && y == y1 {
            return true;
        }
        let e2 = 2 * err;
        if e2 > -dy {
            err -= dy;
            x += sx;
        }
        if e2 < dx {
            err += dx;
            y += sy;
        }
    }
}

/// Ranged spells, the C Borg's estimates (`borg-danger.c:619-760`).
fn ranged_threat(
    w: &World,
    race: &MonsterRace,
    rounds: f64,
    d: i32,
    x: i32,
    y: i32,
    t: &Track,
) -> MonsterThreat {
    let mut out = MonsterThreat::default();
    let freq = if race.freq_spell > 0 {
        race.freq_spell
    } else {
        race.freq_innate
    };
    if freq <= 0 || !line_clear(w, t.x, t.y, x, y) {
        return out;
    }
    let p_cast = (1.0 / f64::from(freq)).min(1.0);
    let mut damage: f64 = 0.0;
    let mut status = false;
    let mut summon = false;
    let economic = false;
    for flag in spell_flags(race) {
        match flag {
            rsf::BR_ACID | rsf::BR_ELEC | rsf::BR_FIRE | rsf::BR_COLD | rsf::BR_POIS => {
                let (e, cap) = match flag {
                    rsf::BR_ACID => (elem::ACID, 1600.0),
                    rsf::BR_ELEC => (elem::ELEC, 1600.0),
                    rsf::BR_FIRE => (elem::FIRE, 1600.0),
                    rsf::BR_COLD => (elem::COLD, 1600.0),
                    _ => (elem::POIS, 800.0),
                };
                let z = (f64::from(race.avg_hp) / 3.0).min(cap);
                damage = damage.max(z * resist(w, e));
            }
            rsf::SHOT | rsf::ARROW | rsf::BOLT => {
                let factor = match flag {
                    rsf::SHOT => 5.0,
                    rsf::ARROW => 6.0,
                    _ => 7.0,
                };
                damage = damage.max((f64::from(race.spell_power) / 8.0 + 1.0) * factor);
            }
            rsf::SPIT | rsf::WHIP => {
                if d < 4 {
                    damage = damage.max(100.0);
                }
            }
            rsf::BO_ACID
            | rsf::BO_ELEC
            | rsf::BO_FIRE
            | rsf::BO_COLD
            | rsf::BO_POIS
            | rsf::MISSILE
            | rsf::BE_ELEC => {
                // A conservative generic ball/bolt: the effect dice are not
                // in Knowledge. Calibration checks it (#124).
                damage = damage.max(f64::from(race.spell_power).max(1.0) * 8.0);
            }
            rsf::SCARE | rsf::BLIND | rsf::CONF | rsf::SLOW | rsf::HOLD => status = true,
            rsf::HEAL => {}
            rsf::S_KIN
            | rsf::S_MONSTER
            | rsf::S_MONSTERS
            | rsf::S_ANIMAL
            | rsf::S_SPIDER
            | rsf::S_HOUND
            | rsf::S_HYDRA
            | rsf::S_AINU
            | rsf::S_DEMON
            | rsf::S_UNDEAD
            | rsf::S_DRAGON
            | rsf::S_HI_UNDEAD
            | rsf::S_HI_DRAGON
            | rsf::S_WRAITH
            | rsf::S_UNIQUE
            | rsf::SHRIEK => summon = true,
            _ => {}
        }
    }
    if damage <= 0.0 && !status && !summon && !economic {
        return out;
    }
    let casts = rounds * p_cast;
    out.expected = casts * damage;
    out.worst = rounds * damage;
    out.disabling = if status {
        (1.0 - (1.0 - p_cast).powf(rounds)).clamp(0.0, 1.0)
    } else {
        0.0
    };
    out.summoning = if summon {
        (1.0 - (1.0 - p_cast).powf(rounds)).clamp(0.0, 1.0)
    } else {
        0.0
    };
    out.economic = if economic { casts } else { 0.0 };
    // Summoners: a growth factor of our own, declared (#163). A summoner's
    // future friends are not in the world yet, so the present estimate is
    // scaled: 1.8 below character level 15, 1.5 from there. The C Borg has
    // no summoner factor; its nearest is the one for monsters with friends
    // (`borg-danger.c:borg_danger_one_kill:2426-2436`), whose code computes
    // `v1 + v1*18/10` (×2.8) below CL15 and `v1 + v1*13/10` (×2.3) from
    // CL15, below CL20 only, while its comments say "extra 80%" and "extra
    // 30%". Our 1.8 follows the first comment; our 1.5 matches neither
    // (comment ×1.3, code ×2.3) and is ours. The calibration measures the
    // composite (docs/M4-CALIBRATION.md); none of these is tuned.
    if summon {
        let player_level = w.character.status.as_ref().map_or(1, |s| s.level);
        let factor = if player_level < 15 { 1.8 } else { 1.5 };
        out.expected *= factor;
        out.worst *= factor;
    }
    out.ranged = true;
    out
}

fn resist(w: &World, e: usize) -> f64 {
    match w.character.gear.resist.get(e).copied().unwrap_or(0) {
        0 => 1.0,
        1 => 2.0 / 3.0,
        2 => 4.0 / 9.0,
        _ => 0.0,
    }
}

fn spell_flags(race: &MonsterRace) -> Vec<usize> {
    let mut out = Vec::new();
    for (i, byte) in race.spell_flags.iter().enumerate() {
        for bit in 0..8 {
            if byte & (1 << bit) != 0 {
                out.push(i * 8 + bit + 1);
            }
        }
    }
    out
}

/// Breeders below character level 20: 1.5×, declared (#163). The C Borg's
/// multiplier at `borg-danger.c:borg_danger_one_kill:2420-2424` says "extra
/// 50%" in its comment but computes `v1 + v1*15/10`, i.e. ×2.5; its second
/// pass (`:2700-2703`) computes `v2 + v2*12/10`, ×2.2. We follow the
/// comment's 1.5, as a declaration, not a lifted number.
pub fn breeder_factor(w: &World, race: &MonsterRace) -> f64 {
    let level = w.character.status.as_ref().map_or(1, |s| s.level);
    if has(&race.flags, rf::MULTIPLY) && level < 20 {
        1.5
    } else {
        1.0
    }
}

/// Whether the race never moves (`RF_NEVER_MOVE`). A step out of such a
/// monster's reach ends its threat for good: it cannot follow, so the
/// relocation removes the whole assessment, and no parting blow is due —
/// a monster that never moves is not giving chase (bbc D0013's floating
/// eye: the free step beats spending the only Phase Door).
pub fn never_move(k: &Knowledge, t: &Track) -> bool {
    k.race(t.race)
        .is_some_and(|r| has(&r.flags, rf::NEVER_MOVE))
}

/// One melee round from a monster, regardless of distance: the free hit a
/// character takes for leaving its reach (the engine's attack of
/// opportunity). Confidence and breeder growth apply; ranged spells do not.
pub fn one_round(w: &World, k: &Knowledge, t: &Track) -> MonsterThreat {
    if !t.awake() {
        // A sleeping monster does not get the attack of opportunity;
        // leaving may wake it for the next round, not this one.
        return MonsterThreat {
            index: t.index,
            confidence: t.confidence(w.turn, k),
            ..Default::default()
        };
    }
    let Some(race) = k.race(t.race) else {
        return MonsterThreat {
            index: t.index,
            expected: 100.0,
            worst: 100.0,
            ..Default::default()
        };
    };
    let conf = t.confidence(w.turn, k);
    let mut m = melee_threat(w, k, race, t, 1.0, t.awake());
    let growth = breeder_factor(w, race);
    m.expected *= growth * conf;
    m.worst *= growth * conf;
    m.disabling *= conf;
    m.summoning *= conf;
    m.economic *= conf;
    m.confidence = conf;
    m.rounds = 1.0;
    m
}

/// The old full-damage value, kept for one release of comparisons:
/// the worst case, rounded. New code uses [`assess`].
pub fn worst(w: &World, k: &Knowledge, t: &Track, x: i32, y: i32, c: i32) -> u32 {
    one_threat(w, k, t, x, y, c).worst.round() as u32
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::knowledge::{Blow, Dice};
    use crate::world::World;

    fn race(
        name: &str,
        speed: i32,
        sleep: i32,
        blows: &[(i32, i32)],
        flags: &[usize],
    ) -> MonsterRace {
        MonsterRace {
            index: 7,
            name: name.into(),
            base: "x".into(),
            level: 1,
            rarity: 1,
            speed,
            avg_hp: 10,
            ac: 10,
            sleep,
            hearing: 0,
            smell: 0,
            exp: 5,
            freq_innate: 0,
            freq_spell: 0,
            spell_power: 1,
            flags: pack(flags),
            spell_flags: vec![0; 12],
            blows: blows
                .iter()
                .map(|&(d, s)| Blow {
                    method: "CLAW".into(),
                    effect: "HURT".into(),
                    power: 60,
                    dice: Dice {
                        base: 0,
                        dice: d,
                        sides: s,
                        m_bonus: 0,
                    },
                })
                .collect(),
            glyph: 0,
            attr: 0,
        }
    }

    fn pack(flags: &[usize]) -> Vec<u8> {
        let mut out = vec![0u8; 11];
        for &f in flags {
            out[(f - 1) / 8] |= 1 << ((f - 1) % 8);
        }
        out
    }

    fn knowledge(r: MonsterRace) -> Knowledge {
        let mut races = vec![MonsterRace::default(); 7];
        races.push(r);
        Knowledge {
            races,
            ..crate::path::tests::energy_with_race()
        }
    }

    fn world(track: Option<(i32, i32, u8, bool)>) -> World {
        let (mut w, _) = crate::path::tests::world(&["#########", "#@......#", "#########"]);
        if let Some(s) = w.character.status.as_mut() {
            s.max_hp = 20;
            s.hp = 20;
            s.known_ac = 20;
            s.level = 1;
        }
        w.level.monsters.clear();
        if let Some((x, y, speed, awake)) = track {
            let mut timed = [0i16; 10];
            if !awake {
                timed[0] = 1000;
            }
            w.level.monsters.push(Track {
                index: 1,
                race: 7,
                x,
                y,
                first_seen: 100,
                last_seen: 100,
                in_view: true,
                hp: 10,
                max_hp: 10,
                timed,
                speed,
            });
        }
        w
    }

    #[test]
    fn expected_and_worst_bracket_the_round() {
        let k = knowledge(race("kobold", 110, 0, &[(1, 6)], &[]));
        let w = world(Some((2, 1, 110, true)));
        let mut w = w;
        if let Some(s) = w.character.status.as_mut() {
            s.max_hp = 20;
            s.hp = 3;
        }
        assert_eq!(hp_of(&w), 3, "{:?}", w.character.status);
        let a = assess(&w, &k, 1, 1, 1, 50);
        // One 1d6 blow at normal speed, with the engine's hit chance
        // against AC 20 and armour reduction: expected is below the
        // full-damage 6, and worst is 6.
        assert!(a.expected > 0.0 && a.expected < 6.0, "{a:?}");
        assert!((a.worst - 6.0).abs() < 1e-9, "{a:?}");
        assert!(a.p_death > 0.0 && a.p_death < 1.0, "{a:?}");
    }

    #[test]
    fn speed_and_distance_change_the_rounds() {
        let k = knowledge(race("fast", 120, 0, &[(1, 6)], &[]));
        let adjacent = world(Some((2, 1, 120, true)));
        assert_eq!(
            one_threat(&adjacent, &k, &adjacent.level.monsters[0], 1, 1, 1).rounds,
            2.0
        );
        let two = world(Some((3, 1, 120, true)));
        assert_eq!(
            one_threat(&two, &k, &two.level.monsters[0], 1, 1, 1).rounds,
            1.0
        );
        let normal = world(Some((3, 1, 110, true)));
        let kk = knowledge(race("normal", 110, 0, &[(1, 6)], &[]));
        assert_eq!(
            one_threat(&normal, &kk, &normal.level.monsters[0], 1, 1, 1).rounds,
            0.0
        );
    }

    #[test]
    fn never_move_monsters_only_score_adjacent() {
        let k = knowledge(race("mold", 110, 0, &[(1, 4), (1, 4)], &[rf::NEVER_MOVE]));
        let w = world(Some((3, 1, 110, true)));
        assert_eq!(
            one_threat(&w, &k, &w.level.monsters[0], 1, 1, 1).expected,
            0.0
        );
        let w = world(Some((2, 1, 110, true)));
        let t = &w.level.monsters[0];
        let a = one_threat(&w, &k, t, 1, 1, 1);
        assert!(a.expected > 0.0 && a.worst >= a.expected, "{a:?}");
    }

    #[test]
    fn fear_refuses_the_attack() {
        let (mut w, _) = crate::path::tests::world(&["#########", "#@......#", "#########"]);
        assert!(can_attack(&w));
        if let Some(s) = w.character.status.as_mut() {
            s.timed = vec![0; 11];
            s.timed[tmd::AFRAID] = 5;
        }
        assert!(!can_attack(&w));
    }

    #[test]
    fn a_ranged_caster_is_seen_through_clear_ground() {
        let mut r = race("acolyte", 110, 0, &[(1, 1)], &[]);
        r.freq_spell = 8;
        r.spell_power = 2;
        r.spell_flags = pack(&[rsf::SCARE, rsf::HEAL]);
        let k = knowledge(r);
        let w = world(Some((4, 1, 110, true)));
        let a = one_threat(&w, &k, &w.level.monsters[0], 1, 1, 1);
        assert!(a.ranged, "{a:?}");
        assert!(a.disabling > 0.0, "fear is a status risk: {a:?}");
    }

    #[test]
    fn a_shrieker_is_a_summoning_risk() {
        let mut r = race("shrieker", 110, 0, &[], &[]);
        r.freq_innate = 25;
        r.spell_flags = pack(&[rsf::SHRIEK]);
        let k = knowledge(r);
        let w = world(Some((3, 1, 110, true)));
        let a = one_threat(&w, &k, &w.level.monsters[0], 1, 1, 1);
        assert!(a.summoning > 0.0, "{a:?}");
        assert_eq!(a.worst, 0.0, "a shriek does no hit-point damage");
    }

    #[test]
    fn a_wall_stops_the_ranged_estimate() {
        let mut r = race("archer", 110, 0, &[(1, 1)], &[]);
        r.freq_innate = 2;
        r.spell_flags = pack(&[rsf::ARROW]);
        let k = knowledge(r);
        let (mut w, _) = crate::path::tests::world(&["#####", "#@#.#", "#####"]);
        if let Some(s) = w.character.status.as_mut() {
            s.max_hp = 20;
            s.known_ac = 20;
        }
        w.level.monsters.clear();
        w.level.monsters.push(Track {
            index: 1,
            race: 7,
            x: 3,
            y: 1,
            first_seen: 100,
            last_seen: 100,
            in_view: true,
            hp: 5,
            max_hp: 5,
            timed: [0; 10],
            speed: 110,
        });
        assert!(!one_threat(&w, &k, &w.level.monsters[0], 1, 1, 1).ranged);
    }

    #[test]
    fn sleep_and_stun_reduce_the_estimate() {
        let k = knowledge(race("sleeper", 110, 10, &[(1, 6)], &[]));
        let awake = world(Some((2, 1, 110, true)));
        let asleep = world(Some((2, 1, 110, false)));
        let a = one_threat(&awake, &k, &awake.level.monsters[0], 1, 1, 1).expected;
        let b = one_threat(&asleep, &k, &asleep.level.monsters[0], 1, 1, 1).expected;
        assert!(b < a, "{b} !< {a}");
    }

    #[test]
    fn the_trace_names_each_contributing_monster() {
        let k = knowledge(race("kobold", 110, 0, &[(1, 6)], &[]));
        let mut w = world(Some((2, 1, 120, true)));
        let mut far = w.level.monsters[0].clone();
        far.index = 2;
        far.x = 7;
        far.timed[0] = 1000; // asleep
        w.level.monsters.push(far);
        let mut gone = w.level.monsters[0].clone();
        gone.index = 3;
        gone.in_view = false;
        gone.last_seen = -1_000_000; // forgotten: confidence near zero
        w.level.monsters.push(gone);
        let a = assess(&w, &k, 1, 1, 1, 50);
        let s = a.summary_detailed(&w, &k, 1, 1);
        // The plain summary's numbers are unchanged; only `sources` is new.
        assert_eq!(
            ThreatSummary {
                sources: Vec::new(),
                ..s.clone()
            },
            a.summary()
        );
        let adjacent = s.sources.iter().find(|m| m.index == 1).expect("adjacent");
        assert_eq!(adjacent.name, "kobold");
        assert_eq!((adjacent.dist, adjacent.speed), (1, 120));
        assert!(adjacent.awake && adjacent.in_view && !adjacent.ranged);
        assert!(adjacent.expected > 0.0 && adjacent.worst >= adjacent.expected);
        for m in &s.sources {
            assert!(
                m.expected > 0.0 || m.worst > 0.0 || m.disabling > 0.0,
                "{m:?}"
            );
            assert_eq!(
                m.group,
                s.sources.len() as u32,
                "every source is a kobold: {m:?}"
            );
        }
        assert!(
            s.sources.iter().all(|m| m.dist <= 20),
            "a monster that adds nothing is left out: {:?}",
            s.sources
        );
    }

    #[test]
    fn a_wide_blow_is_a_distribution() {
        // #164: a blow too wide to enumerate (10d10, dice × sides > 64)
        // must still be a probability distribution, with its mean kept, so
        // that `p_death` is a probability and not a clamped artefact.
        let blow = Blow {
            method: "HIT".into(),
            effect: "HURT".into(),
            power: 60,
            dice: Dice {
                base: 0,
                dice: 10,
                sides: 10,
                m_bonus: 0,
            },
        };
        let (pmf, mean, max) = blow_pmf(&blow, 0, false);
        let total: f64 = pmf.iter().sum();
        assert!((total - 1.0).abs() < 1e-9, "sums to {total}");
        let kept: f64 = pmf.iter().enumerate().map(|(d, p)| d as f64 * p).sum();
        assert!((kept - mean).abs() < 0.6, "mean {mean} kept as {kept}");
        assert!((max - 100.0).abs() < 1e-9 && (mean - 55.0).abs() < 1e-9);

        let k = knowledge(race("colossus", 110, 0, &[(10, 10)], &[]));
        let mut w = world(Some((2, 1, 110, true)));
        if let Some(s) = w.character.status.as_mut() {
            s.max_hp = 50;
            s.hp = 50;
        }
        let a = assess(&w, &k, 1, 1, 1, 50);
        assert!(a.p_death > 0.0 && a.p_death < 1.0, "{a:?}");
    }

    #[test]
    fn confusion_and_stun_scale_every_term() {
        // #165: confusion halves the rounds a monster spends attacking, so
        // p_death and disabling fall with expected and worst; stun lowers
        // the hit chance and the dice as the engine does
        // (`mon-attack.c:353-356`, `:621-624`).
        let mut r = race("biter", 110, 0, &[(2, 6)], &[]);
        r.blows.push(Blow {
            method: "GAZE".into(),
            effect: "PARALYZE".into(),
            power: 0,
            dice: Dice::default(),
        });
        let k = knowledge(r);
        let mut w = world(Some((2, 1, 120, true)));
        if let Some(s) = w.character.status.as_mut() {
            s.hp = 3;
        }
        let plain = one_threat(&w, &k, &w.level.monsters[0], 1, 1, 2);
        let mut confused = w.level.monsters[0].clone();
        confused.timed[mon_tmd::CONF] = 5;
        let c = one_threat(&w, &k, &confused, 1, 1, 2);
        assert!(c.expected < plain.expected, "{c:?} vs {plain:?}");
        assert!(c.p_death < plain.p_death, "{c:?} vs {plain:?}");
        assert!(c.disabling < plain.disabling, "{c:?} vs {plain:?}");
        let mut stunned = w.level.monsters[0].clone();
        stunned.timed[mon_tmd::STUN] = 5;
        let s = one_threat(&w, &k, &stunned, 1, 1, 2);
        assert!(
            s.expected < plain.expected && s.worst < plain.worst,
            "{s:?}"
        );
        assert!(s.p_death < plain.p_death, "{s:?} vs {plain:?}");
        let race = k.race(7).expect("the test race");
        assert!(
            hit_chance(race, &race.blows[0], 20, true)
                < hit_chance(race, &race.blows[0], 20, false)
        );
    }

    #[test]
    fn the_risk_tolerance_is_the_only_dial() {
        let k = knowledge(race("kobold", 110, 0, &[(1, 6)], &[]));
        let w = world(Some((2, 1, 110, true)));
        let a = assess(&w, &k, 1, 1, 1, 0);
        assert!((a.risk(0) - a.expected).abs() < 1e-9);
        assert!((a.risk(100) - a.worst).abs() < 1e-9);
        assert!(a.risk(50) > a.expected && a.risk(50) < a.worst);
    }

    #[test]
    fn unseen_monsters_fade_with_confidence() {
        let k = knowledge(race("kobold", 110, 0, &[(1, 6)], &[]));
        let mut w = world(Some((2, 1, 110, true)));
        let mut t = w.level.monsters[0].clone();
        let seen = one_threat(&w, &k, &t, 1, 1, 1).expected;
        t.in_view = false;
        w.turn = 103;
        let fading = one_threat(&w, &k, &t, 1, 1, 1).expected;
        assert!(fading < seen, "{fading} !< {seen}");
        w.turn = 1_000;
        assert!(
            one_threat(&w, &k, &t, 1, 1, 1).expected < 0.1,
            "confidence keeps falling: {:?}",
            one_threat(&w, &k, &t, 1, 1, 1)
        );
    }

    #[test]
    fn a_forgotten_caster_adds_almost_nothing() {
        // #157: the ranged half fades with the track's confidence, as the
        // melee half does; so does the chance of death.
        let mut r = race("caster", 110, 0, &[(1, 8)], &[]);
        r.freq_spell = 2;
        r.spell_power = 5;
        r.spell_flags = pack(&[rsf::BOLT]);
        let k = knowledge(r);
        let mut w = world(Some((4, 1, 110, true)));
        let seen = one_threat(&w, &k, &w.level.monsters[0], 1, 1, 1);
        assert!(seen.ranged && seen.expected > 0.0, "{seen:?}");
        let mut t = w.level.monsters[0].clone();
        t.in_view = false;
        w.turn = 1_000_000;
        let gone = one_threat(&w, &k, &t, 1, 1, 1);
        assert!(gone.confidence < 0.001, "{gone:?}");
        assert!(gone.expected < 0.01 && gone.worst < 0.1, "{gone:?}");
        assert!(gone.p_death < 0.001, "{gone:?}");
    }

    #[test]
    fn status_blows_are_disabling_not_hit_points() {
        let mut r = race("paralyser", 110, 0, &[], &[]);
        r.blows.push(Blow {
            method: "GAZE".into(),
            effect: "PARALYZE".into(),
            power: 0,
            dice: Dice::default(),
        });
        let k = knowledge(r);
        let w = world(Some((2, 1, 110, true)));
        let a = one_threat(&w, &k, &w.level.monsters[0], 1, 1, 1);
        assert!(a.disabling > 0.0, "{a:?}");
        assert_eq!(a.worst, 0.0, "{a:?}");
        // The trace still names it: a gaze with no dice is not "nothing".
        let s = assess(&w, &k, 1, 1, 1, 50).summary_detailed(&w, &k, 1, 1);
        assert_eq!(s.sources.len(), 1, "{s:?}");
        assert!(s.sources[0].disabling > 0.0 && s.sources[0].worst == 0.0);
    }

    #[test]
    fn summoners_and_breeders_grow_the_estimate() {
        let base = race("mouse", 110, 0, &[(1, 2)], &[]);
        let breeder = MonsterRace {
            flags: pack(&[rf::MULTIPLY]),
            ..base.clone()
        };
        let k = knowledge(breeder);
        let w = world(Some((2, 1, 110, true)));
        let grown = one_threat(&w, &k, &w.level.monsters[0], 1, 1, 1).expected;
        let k0 = knowledge(base);
        let plain = one_threat(&w, &k0, &w.level.monsters[0], 1, 1, 1).expected;
        assert!(grown > plain, "{grown} !> {plain}");
    }

    #[test]
    fn economic_events_are_not_added_to_hp_damage() {
        let mut r = race("cutpurse", 110, 0, &[(1, 1)], &[]);
        r.blows.push(Blow {
            method: "TOUCH".into(),
            effect: "EAT_GOLD".into(),
            power: 0,
            dice: Dice::default(),
        });
        let k = knowledge(r);
        let w = world(Some((2, 1, 110, true)));
        let a = one_threat(&w, &k, &w.level.monsters[0], 1, 1, 1);
        assert!(a.economic > 0.0, "{a:?}");
        assert!(a.worst <= 1.0, "theft adds no hit-point damage: {a:?}");
    }
}
