//! Physical capabilities shared by purchases, dispositions and replay.
//!
//! Rows are engine facts, not borg_power bonuses. The eight legacy needs
//! remain intact for historical replay. New records carry these curves and
//! per-copy supply vectors, including every secondary effect.
use std::collections::{BTreeMap, BTreeSet};
use crate::borg_headers::{borg_h::*, borg_trait_h::*};
use crate::borg_town_dispose::{KitItem, Place};
use crate::borg_town_stock::Stock;
use crate::borg_town_utility::{Journey, UNIT};
use crate::borg_headers::borg_item_h::borg_item;
use crate::headers::list_tvals_h::*;
use crate::headers::list_effects_h::*;
use crate::headers::list_player_timed_h::*;
use crate::state::BorgState;
use crate::engine;
use crate::params::{value as param, Param};

pub const PACK: i64 = 8;
pub const HOME: i64 = 9;
const LIGHT: usize = 0;
const FOOD: usize = 1;
const HEAL: usize = 2;
const POISON: usize = 3;
const PHASE: usize = 4;
const RECALL: usize = 5;
const ATTACK: usize = 6;
const TELEPORT: usize = 7;
const LEVEL: usize = 8;
const OTHER: usize = 9;
const DIG: usize = 10;
const TRAPS: usize = 11;
const MONSTERS: usize = 12;
const INVISIBLE: usize = 13;
const MAP: usize = 14;
const ILLUMINATE: usize = 15;
const SPEED: usize = 16;
const FIRE: usize = 17;
const COLD: usize = 18;
const RESPOIS: usize = 19;
const ACID: usize = 20;
const ELEC: usize = 21;
const BUFF: usize = 22;
const MANA: usize = 23;
const EXPERIENCE: usize = 24;
const CURSE: usize = 25;
const ENCHANT: usize = 26;
const RECHARGE: usize = 27;
const CONFUSION: usize = 28;
const BLIND: usize = 29;
const FEAR: usize = 30;
const GLYPH: usize = 31;
const STATS: usize = 32;
const BOOKS: usize = 38;
const BANISH: usize = 47;
const DESTROY: usize = 48;
const WOUNDS: usize = 49;
const IDENTIFY: usize = 50;
const GAIN_EXPERIENCE: usize = 51;
const RECHARGE_TARGET: usize = 52;
const ACQUIRE: usize = 53;
const CUT: usize = 54;
const STUN: usize = 55;
const CONTROL: usize = 56;
const RANGER_MISSILES: usize = 57;
/// borg-reborn-rs (#491): the healing reserve. Every HP a supply heals counts
/// here as well as in HEAL; up to `town_heal_reserve_hp_percent` of max HP
/// it is worth the healing curve's first HP each (linear), past it nothing
/// more than the curve. Zero weight when the parameter is 0.
const HEAL_RESERVE: usize = 58;
/// borg-reborn-rs (#712): flasks of oil kept to throw, apart from the shared
/// ATTACK curve that launcher ammunition saturates. Each flask kept to throw
/// counts one unit here as well as its damage in ATTACK (as an arrow counts
/// in RANGER_MISSILES). Zero weight unless `town_oil_apart`.
const THROWN_OIL: usize = 59;
/// borg-reborn-rs (#716, owner): shots past the Ranger reserve's soft cap
/// (`town_ranger_arrows_max`, the requirement row's negative demand) taper:
/// each is worth less than the last, down to nothing at
/// `town_ranger_arrows_taper` shots in all. Only the shots past the cap
/// count here. Zero weight unless both are set.
const EXTRA_SHOTS: usize = 60;
const CLASS_RANGER: i32 = 7;
const COUNT: usize = NAMES.len();
/// borg-reborn-rs (food floor): the food items the core requires to be
/// prepared for any level (`borg_prepared`: "5 Food").
const READY_FOOD: i64 = 5;
/// borg-reborn-rs (#455 review): one Ration of Food's nutrition in the
/// food floor's unit (object.txt: Ration of Food, NOURISH:INC_BY 30).
pub(crate) const RATION_NUTRITION: i64 = 30;
/// borg-reborn-rs (#451): the "Hungry" grade of the FOOD timer is 15 percent
/// (player_timed.txt, `grade:y:15:Hungry`); `BI_ISHUNGRY` is food below
/// `PY_FOOD_HUNGRY`, and the core reads a Remove Hunger scroll then. A
/// `NOURISH:INC_TO` food (Remove Hunger 50, Honey-cake and Draught 60,
/// Waybread 75: object.txt) is worth what it adds from there.
const HUNGRY_PERCENT: i64 = 15;

pub const NAMES: [&str; 61] = ["light turns", "nutrition", "healing HP", "poison cure", "phase", "Recall",
    "ranged damage", "teleport", "level escape", "teleport other", "digging", "trap/door detection",
    "monster detection", "invisible detection", "mapping", "illumination", "speed", "resist fire",
    "resist cold", "resist poison", "resist acid", "resist electricity", "combat buff", "mana",
    "restore experience", "remove curse", "enchantment", "recharging", "confusion cure", "blindness cure",
    "fear cure", "glyph", "strength", "intelligence", "wisdom", "dexterity", "constitution", "charisma",
    "book 0", "book 1", "book 2", "book 3", "book 4", "book 5", "book 6", "book 7", "book 8",
    "banishment", "destruction", "wound/stun cure", "identify runes", "gain experience", "recharge target", "equipment opportunity", "cut reduction", "stun reduction", "monster control", "Ranger usable shots", "healing reserve", "thrown oil", "extra shots"];

/// Curves: [capability, weight in utility thousandths, demand, carried, Home].
/// Supplies: [scope (shop 0..7, pack 8, Home 9), slot, capability, units/copy].
/// All dynamic class/level/effect facts are frozen here for engine-free replay.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Model {
    pub available: Vec<[i64;3]>,
    pub packing: Option<crate::borg_town_packing::Packing>,
    pub curves: Vec<[i64; 5]>,
    pub supplies: Vec<[i64; 4]>,
}

/// borg-reborn-rs (#716): the extra shots past the soft cap, which the
/// Ranger reserve's requirement row holds as its negative demand (no cap, no
/// extra value). The row's weight is the reserve's, its demand the taper's
/// length L: a shot x past the cap is worth a shot under the cap times
/// (1 - x / L), so the value is w/cap * (x - x^2 / 2L) up to L, flat past it.
fn extra_gain(m: &Model, r: &[i64; 5], n: i64) -> i64 {
    let Some(reserve) = m.curves.get(RANGER_MISSILES) else { return 0 };
    if reserve[2] >= 0 || r[1] <= 0 || r[2] <= 0 { return 0; }
    let (cap, len, w) = (i128::from(-reserve[2]), i128::from(r[2]), i128::from(r[1]));
    let value = |a: i64| {
        let x = (i128::from(a) - cap).clamp(0, len);
        (w * (2 * len * x - x * x) / (2 * cap * len)) as i64
    };
    value((r[3] + n).max(0)) - value(r[3])
}

fn coverage(row: &[i64; 5], amount: i64) -> i64 {
    if amount <= 0 || row[1] <= 0 { return 0; }
    // Negative demand is an actual one-time repair/access requirement,
    // not a stock quota: a second cure cannot restore the same lost XP.
    if row[2] < 0 { return row[1] * amount.min(-row[2]) / -row[2]; }
    row[1].saturating_mul(amount) / amount.saturating_add(row[2].max(1))
}

impl Model {
    pub fn units(&self, scope: i64, slot: i64, count: i64) -> Vec<(usize, i64)> {
        self.supplies.iter().filter(|r| r[0] == scope && r[1] == slot)
            .map(|r| (r[2] as usize, r[3] * count)).collect()
    }
    pub fn gain(&self, units: &[(usize, i64)]) -> i64 {
        let mut sum = BTreeMap::<usize, i64>::new();
        for &(id, n) in units { *sum.entry(id).or_default() += n; }
        sum.into_iter().map(|(id, n)| {
            let r = &self.curves[id];
            if id == RECHARGE && self.curves[RECHARGE_TARGET][3] == 0 { return 0; }
            if id == EXTRA_SHOTS { return extra_gain(self, r, n); }
            coverage(r, (r[3] + n).max(0)) - coverage(r, r[3])
        }).sum()
    }
    pub fn ware(&self, shop: u8, slot: u8, count: i32) -> i64 {
        self.gain(&self.units(i64::from(shop), i64::from(slot), i64::from(count)))
    }
    pub fn scope(it: &KitItem) -> i64 { if it.place == Place::Pack { PACK } else { HOME } }
    pub fn keep(&self, it: &KitItem) -> i64 {
        -self.gain(&self.units(Self::scope(it), i64::from(it.slot), -i64::from(it.qty)))
    }
    pub fn take(&self, it: &KitItem) -> i64 {
        self.gain(&self.units(Self::scope(it), i64::from(it.slot), i64::from(it.qty)))
    }
    pub fn reserve(&self, it: &KitItem, sign: i64, discount: i64) -> i64 {
        self.units(Self::scope(it), i64::from(it.slot), i64::from(it.qty)).into_iter().map(|(id, n)| {
            let r = &self.curves[id];
            let from = r[4];
            let to = (from + sign * n).max(0);
            (coverage(r, to) - coverage(r, from)) * discount / UNIT
        }).sum()
    }
    pub fn change(&mut self, it: &KitItem, sign: i64) {
        for (id, n) in self.units(Self::scope(it), i64::from(it.slot), sign * i64::from(it.qty)) {
            self.curves[id][3] = (self.curves[id][3] + n).max(0);
        }
    }
    pub fn change_home(&mut self, it: &KitItem, sign: i64) {
        for (id, n) in self.units(Self::scope(it), i64::from(it.slot), sign * i64::from(it.qty)) {
            self.curves[id][4] = (self.curves[id][4] + n).max(0);
        }
    }
    pub fn mask(&self, shop: u8, slot: u8) -> u64 {
        self.units(i64::from(shop), i64::from(slot), 1).iter().fold(0, |m, &(id, _)| m | (1 << id))
    }
    pub fn valid(&self) -> bool {
        self.available.iter().all(|r| (0..=7).contains(&r[0]) && (0..=255).contains(&r[1]) && (0..=255).contains(&r[2]))
            && (53..=COUNT).contains(&self.curves.len()) && self.packing.as_ref().is_none_or(|p| p.valid()) && self.curves.iter().enumerate().all(|(i, r)| r[0] == i as i64 && r[1] >= 0 && r[3] >= 0 && r[4] >= 0)
            && self.supplies.iter().all(|r| (0..=HOME).contains(&r[0]) && r[1] >= 0 && (0..self.curves.len() as i64).contains(&r[2]) && r[3] > 0)
    }
}

/// Context for the pure effect classifier; engine facts are obtained once.
struct Context<'a> {
    st: &'a BorgState,
    journey: Journey,
    lantern: bool,
}

fn timed_cap(timed: u32) -> Option<usize> {
    match timed {
        TMD_FAST | TMD_SPRINT => Some(SPEED),
        TMD_OPP_FIRE => Some(FIRE), TMD_OPP_COLD => Some(COLD), TMD_OPP_POIS => Some(RESPOIS),
        TMD_OPP_ACID => Some(ACID), TMD_OPP_ELEC => Some(ELEC),
        // borg-reborn-rs (#486): not TMD_BOLD. A Bold timed effect is no buff
        // the core uses: the core takes Boldness only to cure fear (a Potion of
        // Boldness is CURE:AFRAID, priced through FEAR; borg_caution and
        // borg_recover quaff it when afraid), and the hunger list drinks wine
        // as food. The only kind with TIMED_INC:BOLD in object.txt is the
        // Pint of Fine Wine (no activation or class spell grants it), whose
        // buff value bought 14-21 pints a game that were never drunk for it.
        TMD_HERO | TMD_SHERO | TMD_BLESSED | TMD_SHIELD | TMD_STONESKIN | TMD_PROTEVIL
            | TMD_INVULN | TMD_ATT_ACID | TMD_ATT_ELEC | TMD_ATT_FIRE | TMD_ATT_COLD
            | TMD_ATT_POIS | TMD_ATT_EVIL | TMD_ATT_DEMON | TMD_ATT_VAMP | TMD_POWERSHOT => Some(BUFF),
        TMD_SINVIS => Some(INVISIBLE), TMD_TELEPATHY => Some(MONSTERS),
        _ => None,
    }
}

/// RANDOM/SELECT alternatives are mutually exclusive. Use the guaranteed
/// component-wise service rather than adding incompatible branches.
#[cfg(test)] // borg-reborn-rs (#400): only the effect-table tests use it.
fn effect_supply(effects: &[(u16, i32, i64)], hp: i64, service_turns: i64) -> Vec<(usize, i64)> {
    effect_supply_for_consumer(effects, hp, service_turns, false)
}

fn effect_supply_for_consumer(effects: &[(u16, i32, i64)], hp: i64, service_turns: i64, confusion: bool) -> Vec<(usize, i64)> {
    let mut sum = BTreeMap::<usize, i64>::new();
    let mut i = 0;
    while i < effects.len() {
        let e = effects[i];
        if matches!(u32::from(e.0), EF_RANDOM | EF_SELECT) {
            let n = (e.2.max(0) as usize).min(effects.len() - i - 1);
            let branches: Vec<BTreeMap<usize, i64>> = effects[i+1..i+1+n].iter()
                .map(|&b| effect_supply_linear(&[b], hp, service_turns, confusion).into_iter().collect()).collect();
            if let Some(first) = branches.first() {
                for &id in first.keys() {
                    let amount = branches.iter().map(|b| *b.get(&id).unwrap_or(&0)).min().unwrap_or(0);
                    *sum.entry(id).or_default() += amount;
                }
            }
            i += n + 1;
        } else {
            for (id, n) in effect_supply_linear(&[e], hp, service_turns, confusion) { *sum.entry(id).or_default() += n; }
            i += 1;
        }
    }
    sum.into_iter().filter(|&(_, n)| n > 0).collect()
}

/// borg-reborn-rs (#491, owner: "budget healing speed"): the healing units of
/// one HEAL_HP use. The HP it restores (capped at max HP), weighted by how
/// fast it brings the character back to max HP: one that heals at least
/// `speed_percent` of max HP in one swallow counts in full; a smaller one
/// counts `heal x heal / (speed_percent of max HP)`, so a Cure Light Wounds
/// (20 HP) is worth less the more HP the character has. 0 = HP alone.
fn heal_units(amount: i64, hp: i64, speed_percent: i64) -> i64 {
    let heal = amount.max(1).min(hp.max(1));
    // The full-value heal in hundredths of an HP, so no rounding moves it.
    let full = hp.max(1) * speed_percent.clamp(0, 100);
    if full > heal * 100 { heal * heal * UNIT * 100 / full } else { heal * UNIT }
}

/// borg-reborn-rs (#487): what a healing use puts in the reserve: the real
/// HP with `town_heal_reserve_real_hp`, else the speed-weighted HP.
fn reserve_heal(speed_weighted: i64, real: i64, real_hp: bool) -> i64 {
    if real_hp { real } else { speed_weighted }
}

fn reduction_cap(subtype: u32) -> Option<usize> {
    match subtype { TMD_POISONED => Some(POISON), TMD_CONFUSED => Some(CONFUSION),
        TMD_BLIND => Some(BLIND), TMD_AFRAID => Some(FEAR), TMD_CUT => Some(CUT),
        TMD_STUN => Some(STUN), _ => None }
}

/// A conservative independent-rod schedule. Initial ready rods come from
/// native ceil(timeout/cooldown); do not average away a ready rod. Each use
/// costs an action. Charge is allocated to at most that many charging rods.
fn rod_uses(quantity: i64, timeout: i64, cooldown: i64, horizon: i64) -> i64 {
    let q = quantity.clamp(1, 255);
    let cool = cooldown.max(1);
    let mut timeout = timeout.max(0);
    let mut uses = 0;
    // Match number_charging/recharge_timeout, using the native average
    // cooldown as an explicit forecast (actual recharge rolls may differ).
    for _ in 0..horizon.max(0) {
        let charging = (timeout as u64).div_ceil(cool as u64).min(q as u64) as i64;
        if charging < q { uses += 1; timeout += cool; }
        let charging = (timeout as u64).div_ceil(cool as u64).min(q as u64) as i64;
        timeout -= charging.min(timeout);
    }
    uses
}

/// Effects with contextual relevance, in physical units. Metadata/control
/// effects do not earn utility by themselves. Harmful effects never do.
fn effect_supply_linear(effects: &[(u16, i32, i64)], hp: i64, service_turns: i64, confusion: bool) -> Vec<(usize, i64)> {
    use crate::headers::list_projections_h::*;
    let mut out = BTreeMap::<usize, i64>::new();
    // borg-reborn-rs (#487): the real HP restored, for the healing reserve.
    let mut real_heal = 0;
    for &(effect, subtype, amount) in effects {
        if u32::from(effect) == EF_HEAL_HP {
            real_heal += heal_units(amount, hp, 0);
        }
        let n = amount.max(1) * UNIT;
        let service = service_turns.max(1) * UNIT;
        let role = match u32::from(effect) {
            // borg-reborn-rs (#451): INC_TO (subtype 3) fills to `amount` percent of a full stomach
            // (effect_handler_NOURISH). Read when hungry (HUNGRY_PERCENT) it gives the difference.
            EF_NOURISH if subtype == 3 && amount > HUNGRY_PERCENT => Some((FOOD, (amount - HUNGRY_PERCENT) * UNIT)),
            EF_NOURISH if subtype == 3 => None,
            EF_NOURISH if subtype != 1 && amount > 0 => Some((FOOD, n)), // A decreasing food effect is not nourishment.
            EF_HEAL_HP => Some((HEAL, heal_units(amount, hp, i64::from(param(Param::town_heal_speed_percent))))),
            EF_CURE => reduction_cap(subtype as u32).map(|cap| (cap, n)),
            EF_TIMED_DEC if amount > 0 => reduction_cap(subtype as u32).map(|cap| (cap, n)),
            // borg-reborn-rs (#452): a speed effect is one emergency use, not
            // `amount` turns of buff. The potion's +10 speed (object.txt,
            // Potion of Speed: TIMED_INC:FAST, 20+2d10 turns) is a reserve
            // the core drinks in a dangerous fight (borg_defend_aux_speed),
            // priced like the escapes (exposure), per potion. Counted in turns against
            // the whole trip it was one percent of the demand and was sold.
            EF_TIMED_INC | EF_TIMED_INC_NO_RES | EF_TIMED_SET => timed_cap(subtype as u32)
                .map(|cap| (cap, if cap == SPEED { UNIT } else { n })),
            EF_TELEPORT => Some((TELEPORT, UNIT)), EF_TELEPORT_LEVEL => Some((LEVEL, UNIT)),
            EF_TELEPORT_TO => Some((OTHER, UNIT)), EF_RECALL => Some((RECALL, UNIT)),
            EF_RESTORE_MANA => Some((MANA, n)), EF_RESTORE_EXP => Some((EXPERIENCE, UNIT)),
            EF_GAIN_EXP => Some((GAIN_EXPERIENCE, UNIT)),
            EF_IDENTIFY => Some((IDENTIFY, UNIT)),
            EF_ACQUIRE => Some((ACQUIRE, n)),
            EF_RESTORE_STAT | EF_GAIN_STAT if (0..5).contains(&subtype) => Some((STATS + subtype as usize, UNIT)),
            EF_REMOVE_CURSE => Some((CURSE, n)), EF_ENCHANT | EF_BRAND_WEAPON | EF_BRAND_AMMO | EF_BRAND_BOLTS => Some((ENCHANT, UNIT)),
            EF_RECHARGE => Some((RECHARGE, UNIT)),
            EF_MAP_AREA | EF_LIGHT_LEVEL => Some((MAP, service)), EF_LIGHT_AREA => Some((ILLUMINATE, service)),
            EF_DETECT_TRAPS | EF_DETECT_DOORS | EF_DETECT_STAIRS => Some((TRAPS, service)),
            EF_DETECT_EVIL | EF_DETECT_VISIBLE_MONSTERS | EF_DETECT_LIVING_MONSTERS | EF_DETECT_SOUL | EF_READ_MINDS => Some((MONSTERS, service)),
            EF_DETECT_INVISIBLE_MONSTERS => Some((INVISIBLE, service)),
            EF_DETECT_OBJECTS | EF_SENSE_OBJECTS | EF_DETECT_GOLD | EF_SENSE_GOLD | EF_DETECT_ORE | EF_PROBE => Some((MAP, service)),
            EF_GLYPH => Some((GLYPH, UNIT)), EF_BANISH | EF_MASS_BANISH => Some((BANISH, UNIT)),
            EF_DESTRUCTION | EF_EARTHQUAKE => Some((DESTROY, UNIT)),
            EF_BOLT | EF_BEAM | EF_BOLT_OR_BEAM | EF_BOLT_AWARE | EF_BOLT_STATUS | EF_BOLT_STATUS_DAM
                | EF_LINE | EF_BALL | EF_TOUCH | EF_TOUCH_AWARE | EF_PROJECT_LOS | EF_PROJECT_LOS_AWARE
                | EF_SPOT | EF_SPHERE | EF_BREATH | EF_ARC | EF_SHORT_BEAM | EF_SWARM | EF_STRIKE
                | EF_STAR | EF_STAR_BALL | EF_MON_TIMED_INC => match subtype as u32 {
                    PROJ_AWAY_ALL | PROJ_AWAY_EVIL | PROJ_AWAY_UNDEAD | PROJ_AWAY_SPIRIT => Some((OTHER, UNIT)),
                    PROJ_KILL_WALL => Some((DIG, UNIT)),
                    // M13 repairs the Confuse Monster wand lookup. Enable
                    // its control only for the matching wand consumer; the
                    // Confuse Monsters staff and Stun Monster wand stay out.
                    PROJ_MON_CONF if confusion => Some((CONTROL, UNIT)),
                    PROJ_MON_SLOW | PROJ_MON_HOLD | PROJ_SLEEP_ALL
                        | PROJ_SLEEP_EVIL | PROJ_TURN_ALL => Some((CONTROL, UNIT)),
                    // Native elemental damage occupies 0..=24. DARK_WEAK,
                    // terrain creation/destruction and monster-aiding types
                    // are deliberately not generic damage or buff fallbacks.
                    0..=24 | PROJ_LIGHT_WEAK | PROJ_MON_DRAIN | PROJ_MON_CRUSH
                        | PROJ_DISP_ALL | PROJ_DISP_EVIL | PROJ_DISP_UNDEAD if amount > 0 => Some((ATTACK, amount * UNIT)),
                    _ => None,
                },
            EF_WONDER => None, // no guaranteed damage for a random special handler
            _ => None,
        };
        if let Some((cap, units)) = role { *out.entry(cap).or_default() += units; }
    }
    // borg-reborn-rs (#491): the same HP also fill the healing reserve.
    // borg-reborn-rs (#487, owner): with `town_heal_reserve_real_hp` the
    // reserve counts the real HP restored; healing speed weights only the
    // healing curve (which kind is worth more), not how many potions fill
    // the reserve. Speed-weighted, a Cure Light Wounds counted about 5.6 HP
    // at 108 max HP and the reserve took dozens of them.
    if let Some(&heal) = out.get(&HEAL) {
        out.insert(HEAL_RESERVE, reserve_heal(heal, real_heal, param(Param::town_heal_reserve_real_hp) != 0));
    }
    out.into_iter().collect()
}

/// borg-reborn-rs (#649): the average damage of a thrown throwing object,
/// in `UNIT`s, as the engine computes it (`ranged_damage`,
/// `player-attack.c:554-582`): the dice's average times `2 + weight / 12`
/// (integer division, weight in tenths of a pound), and three times that
/// for an exploding object (`make_ranged_throw`, `player-attack.c:1296`).
/// A flask of oil (1d4, weight 20, `EXPLODE | THROWING`) does 22.5; the
/// planner counted 2.5, so a sling shot (5) looked twice as good.
fn thrown_damage(dd: i64, ds: i64, weight: i64, explode: bool) -> i64 {
    let dice = dd * (ds + 1) * UNIT / 2;
    let throw = dice * (2 + weight.max(0) / 12);
    (if explode { throw * 3 } else { throw }).max(UNIT)
}

fn oil_supply(quantity: i64, fuel_bottles: i64, damage: i64, fuel: i64) -> Vec<(usize, i64)> {
    let mut out = Vec::new();
    let n = quantity.max(1);
    if fuel_bottles > 0 && fuel > 0 { out.push((LIGHT, fuel * UNIT * fuel_bottles / n)); }
    if n > fuel_bottles {
        out.push((ATTACK, damage * (n - fuel_bottles) / n));
        // borg-reborn-rs (#712): each flask kept to throw, on its own curve.
        if param(Param::town_oil_apart) != 0 { out.push((THROWN_OIL, UNIT * (n - fuel_bottles) / n)); }
    }
    out
}

// Called only after native launcher compatibility and curse checks. Each
// arrow contributes both damage and one reliable shot, separate objectives.
fn missile_supply(class: i32, damage: i64) -> Vec<(usize, i64)> {
    let mut out = vec![(ATTACK, damage)];
    if missiles_percent(class, param(Param::town_missiles_percent)) > 0 {
        out.push((RANGER_MISSILES, UNIT));
        // borg-reborn-rs (#716): the same shot, counted past the soft cap.
        out.push((EXTRA_SHOTS, UNIT));
    }
    out
}

/// borg-reborn-rs (#666, owner): the reserve of shots for the class's
/// launcher, in percent of `borg_town_w_ranger_missiles`:
/// `town_missiles_percent` (per class, by shooting skill); -1 = C, the
/// Ranger's reserve only (100 for a Ranger, 0 for the others).
pub(crate) fn missiles_percent(class: i32, setting: i32) -> i64 {
    if setting < 0 { if class == CLASS_RANGER { 100 } else { 0 } } else { i64::from(setting) }
}

/// borg-reborn-rs (#666, owner): "projectile buying is more important if they
/// cannot get flasks of oil or wands". With at least `OTHER_RANGED_ENOUGH`
/// flasks of oil and attack-wand charges carried, the shots' reserve weighs
/// `town_missiles_other_ranged_percent` (100 = no change).
const OTHER_RANGED_ENOUGH: i64 = 5;
fn missiles_other_ranged(weight: i64, other_ranged: i64, percent: i32) -> i64 {
    if other_ranged >= OTHER_RANGED_ENOUGH { scale_percent(weight, percent) } else { weight }
}

/// The flasks of oil and the charges of attack wands in the pack.
fn other_ranged(st: &BorgState) -> i64 {
    let iv = &st.item_val;
    st.items.items.iter().take(crate::borg_item::inven_wield() as usize).filter(|i| i.iqty != 0).map(|i| {
        let tval = u32::from(i.tval);
        let sval = i32::from(i.sval);
        if tval == TV_FLASK && sval == iv.sv_flask_oil { i64::from(i.iqty) }
        else if tval == TV_WAND && i.aware && (sval == iv.sv_wand_magic_missile || sval == iv.sv_wand_stinking_cloud) { i64::from(i.pval).max(0) }
        else { 0 }
    }).sum()
}

/// borg-reborn-rs (#487, #520): the healing curve's scale:
/// `town_heal_curve_scale_percent`, the class's own value when it has one.
pub(crate) fn heal_curve_scale_percent(_class: i32) -> i32 {
    // borg-reborn-rs (#520): the class's own value, if any, comes through
    // `param` (town_heal_curve_scale_percent@<class> or the class file).
    param(Param::town_heal_curve_scale_percent)
}

/// borg-reborn-rs (#487, #520): the healing reserve:
/// `town_heal_reserve_hp_percent`, the class's own value when it has one
/// (0 is a reserve of none).
pub(crate) fn heal_reserve_hp_percent(_class: i32) -> i32 {
    // borg-reborn-rs (#520): the class's own value, if any, comes through
    // `param` (town_heal_reserve_hp_percent@<class> or the class file).
    param(Param::town_heal_reserve_hp_percent)
}

/// borg-reborn-rs (#443, owner 2026-11-16): consumables the core never uses
/// on purpose, from a source audit of every kind in object.txt against the
/// core's use sites (not bookkeeping: counting, home notice, buy lists).
/// Excluded: foods (any known food is eaten, borg_eat_food_any) and potions
/// whose use-up is their benefit (stat gain, Experience, Infravision,
/// borg_junk). Harmful kinds are here too. Remove a kind when the core
/// learns to use it.
const WORTHLESS: &[(u32, &str)] = &[
    (TV_SCROLL, "Door Destruction"), (TV_SCROLL, "Dispel Undead"),
    (TV_SCROLL, "Treasure Detection"), (TV_SCROLL, "Summon Monster"), (TV_SCROLL, "Summon Undead"),
    (TV_SCROLL, "*Destruction*"), (TV_SCROLL, "Aggravate Monster"), (TV_SCROLL, "Curse Weapon"),
    (TV_SCROLL, "Curse Armour"), (TV_SCROLL, "Darkness"), (TV_SCROLL, "Trap Creation"),
    // borg-reborn-rs (#470): never read, never kept (borg_junk discards it).
    (TV_SCROLL, "Deep Descent"),
    (TV_MUSHROOM, "Second Sight"), (TV_MUSHROOM, "Emergency"), (TV_MUSHROOM, "Terror"),
    (TV_MUSHROOM, "Debility"), (TV_MUSHROOM, "Sprinting"), (TV_MUSHROOM, "Turbulence"),
    (TV_MUSHROOM, "Shadows"),
    (TV_POTION, "Poison"), (TV_POTION, "*Enlightenment*"), (TV_POTION, "Surprise"),
    (TV_POTION, "Salt Water"), (TV_POTION, "Dragon Breath"),
    (TV_WAND, "Stone to Mud"), (TV_WAND, "Stun Monster"), (TV_WAND, "Polymorph"), (TV_WAND, "Darkness"),
    (TV_WAND, "Clone Monster"), (TV_WAND, "Disable Traps"), (TV_WAND, "Dragon's Breath"),
    (TV_WAND, "Heal Monster"), (TV_WAND, "Haste Monster"),
    (TV_STAFF, "Summoning"), (TV_STAFF, "Haste Monsters"), (TV_STAFF, "Darkness"),
    (TV_STAFF, "Earthquakes"), (TV_STAFF, "Confuse Monsters"), (TV_STAFF, "Cure Light Wounds"),
    (TV_STAFF, "Starlight"),
    (TV_ROD, "Treasure Location"), (TV_ROD, "Probing"), (TV_ROD, "Restoration"),
    (TV_ROD, "Polymorph"), (TV_ROD, "Disable Traps"),
];

fn worthless(tval: u32, sval: i32) -> bool {
    use std::sync::OnceLock;
    static KINDS: OnceLock<Vec<(u32, i32)>> = OnceLock::new();
    KINDS.get_or_init(|| WORTHLESS.iter()
        .map(|&(t, name)| (t, engine::lookup_sval(t as i32, name)))
        .filter(|&(_, s)| s >= 0).collect())
        .contains(&(tval, sval))
        // borg-reborn-rs (#504): the cornered mode reads Monster Confusion;
        // without it the core never does.
        || (param(Param::caution_cornered) == 0
            && tval == TV_SCROLL
            && sval >= 0
            && sval == engine::lookup_sval(TV_SCROLL as i32, "Monster Confusion"))
}

fn supplies(item: &borg_item, ctx: &Context, origin: (i64, usize)) -> Vec<(usize, i64)> {
    if item.iqty == 0 || item.kind == 0 { return Vec::new(); }
    let tval = u32::from(item.tval);
    // A shop teaches its kind on purchase; carried unknown flavours/devices
    // must not exploit diagnostic hidden kind data.
    if !item.aware { return Vec::new(); }
    let st = ctx.st;
    let qty = i64::from(item.iqty).max(1);
    let mut effects = engine::kind_effect_values(item.kind as usize, ctx.journey.level as i32);
    if tval == TV_SCROLL && i32::from(item.sval) == st.item_val.sv_scroll_phase_door {
        return vec![(PHASE, UNIT)]; // TELEPORT(10) is not full Teleportation.
    }
    if tval == TV_FLASK {
        // borg-reborn-rs (#649): the engine's thrown damage, not the dice alone.
        let damage = thrown_damage(i64::from(item.dd), i64::from(item.ds), i64::from(item.weight),
            engine::flag_has(&item.flags, crate::headers::list_object_flags_h::OF_EXPLODE as i32));
        let fuel = i64::from(item.pval).max(0);
        let light = &st.items.items[(crate::borg_item::inven_wield() + 5) as usize];
        let needed = if ctx.lantern && fuel > 0 {
            ((ctx.journey.turns - i64::from(light.timeout)).max(0) as u64).div_ceil(fuel as u64) as i64
        } else { 0 };
        // Allocate whole bottles to fuel before ammunition. A bottle cannot
        // supply both uses. Shop options are prospective single copies; live
        // execution buys one then refreshes the allocation. Held stacks use
        // an average per-copy vector whose whole-stack sum is conserved.
        let end = if origin.0 == PACK { origin.1 } else { crate::borg_item::inven_wield() as usize };
        let prior: i64 = st.items.items.iter().take(end).filter(|i| i.aware && u32::from(i.tval) == TV_FLASK)
            .map(|i| i64::from(i.iqty)).sum();
        let quantity = if origin.0 < PACK { 1 } else { qty };
        // borg-reborn-rs (#682): a class's oil scale weights each throw.
        let damage = damage * i64::from(param(Param::town_oil_curve_scale_percent)) / 100;
        return oil_supply(quantity, (needed - prior).clamp(0, quantity), damage, fuel);
    }
    if matches!(tval, TV_ARROW | TV_BOLT | TV_SHOT) {
        if st.borg.tr(BI_AMMO_TVAL) != i32::from(item.tval) || item.cursed { return Vec::new(); }
        let damage = (i64::from(item.dd) * (i64::from(item.ds) + 1) * UNIT / 2
            + (i64::from(item.to_d) + i64::from(st.borg.tr(BI_BTODAM))) * UNIT).max(UNIT)
            * i64::from(st.borg.tr(BI_AMMO_POWER)).max(1);
        return missile_supply(st.borg.tr(BI_CLASS), damage);
    }
    if tval == TV_LIGHT {
        return if item.timeout > 0 { vec![(LIGHT, i64::from(item.timeout) * UNIT)] } else { Vec::new() };
    }
    if tval == TV_DIGGING { return vec![(DIG, UNIT)]; }
    if engine::tval_is_book_k(item.kind as usize) {
        for b in 0..engine::class_num_books().min(9) {
            let book = engine::class_book(b);
            if book.tval != i32::from(item.tval) || book.sval != i32::from(item.sval) { continue; }
            let usable = (0..book.num_spells).map(|s| engine::class_book_spell(b, s))
                .filter(|s| s.slevel <= ctx.journey.level as i32 && s.smana <= st.borg.tr(BI_MAXSP)).count();
            if usable > 0 { return vec![(BOOKS + b as usize, UNIT)]; }
        }
        return Vec::new();
    }
    if !matches!(tval, TV_POTION | TV_SCROLL | TV_FOOD | TV_MUSHROOM | TV_WAND | TV_ROD | TV_STAFF) { return Vec::new(); }
    // borg-reborn-rs (#443): the core has no code to use these on purpose,
    // so they are worth nothing to the planner until it does.
    if worthless(tval, i32::from(item.sval)) {
        return Vec::new();
    }
    // borg-reborn-rs (food counting): only what borg_trait counts in BI_FOOD
    // feeds the core (its readiness checks). A Remove Hunger scroll counted
    // as food let the planner sell the rations the core then missed
    // ("restock food < 3"); the core seldom read it (6 of 253 bought).
    // borg-reborn-rs (#451): now borg_trait counts the scroll beside the
    // rations under this plan (with_remove_hunger) and the core reads it
    // before any food (borg_satisfy_hunger_before_food), so the scroll
    // supplies nutrition, as much as it adds to a hungry stomach. No other
    // non-food kind does: a potion's nourishment (Cure Light Wounds and the
    // like, 1) is not a meal.
    let hunger_scroll = tval == TV_SCROLL && i32::from(item.sval) == st.item_val.sv_scroll_satisfy_hunger;
    if !matches!(tval, TV_FOOD | TV_MUSHROOM) && !hunger_scroll {
        effects.retain(|&(e, _, _)| u32::from(e) != EF_NOURISH);
    }
    // Drinks that nourish almost nothing (wine, whisky, liquor, miruvor:
    // 3-5 nutrition, object.txt) are not food to the planner. Counted, the
    // food floor bought a pint of wine for its few nutrition points, then
    // stored it at the Home as not worth its pack slot, every visit.
    effects.retain(|&(e, sub, amount)| u32::from(e) != EF_NOURISH || sub == 1 || amount >= 10);
    // Poisonous/hallucinatory nourishment must not be sold as safe food.
    if effects.iter().any(|&(e, sub, _)| matches!(u32::from(e), EF_TIMED_INC | EF_TIMED_INC_NO_RES) && matches!(sub as u32, TMD_POISONED | TMD_CONFUSED | TMD_IMAGE | TMD_PARALYZED)) {
        effects.retain(|&(e, _, _)| u32::from(e) != EF_NOURISH);
    }
    effects.retain(|&(e, sub, _)| {
        if (0..5).contains(&sub) {
            if u32::from(e) == EF_RESTORE_STAT { return st.borg.tr(BI_ISFIXSTR + sub as u32) > 0; }
            if u32::from(e) == EF_GAIN_STAT { return st.borg.tr(BI_CSTR + sub as u32) < 18 + 100; }
        }
        true
    });
    for (e, subtype, amount) in &mut effects {
        if reduction_cap(*subtype as u32).is_some() {
            let timer = i64::from(engine::player_timed(*subtype as u32)).max(0);
            if u32::from(*e) == EF_CURE { *amount = timer.max(1); }
            if u32::from(*e) == EF_TIMED_DEC { *amount = (*amount).max(0).min(timer); }
        }
        if u32::from(*e) == EF_RESTORE_MANA && *amount == 0 { *amount = i64::from(st.borg.tr(BI_MAXSP)); }
    }
    let confusion = tval == TV_WAND && i32::from(item.sval) == st.item_val.sv_wand_confuse_monster;
    let mut out = effect_supply_for_consumer(&effects, ctx.journey.hp, i64::from(st.cfg(BORG_TOWN_SUPPORT_TURNS)).max(1), confusion);
    let uses = if tval == TV_WAND || tval == TV_STAFF { i64::from(item.pval).max(0) * UNIT / qty }
        else if tval == TV_ROD {
            // MINIMISE/AVERAGE are pure; the rod's physical cooldown is not a level gate.
            let turns = engine::kind_recharge_average(item.kind as usize).max(1);
            rod_uses(qty, i64::from(item.timeout), i64::from(turns), ctx.journey.turns) * UNIT / qty
        } else { UNIT };
    for (_, units) in &mut out { *units = *units * uses / UNIT; }
    // borg-reborn-rs (#666, owner): a device is worth `town_devices_percent`
    // of what it supplies (per class, by device skill; 100 = as computed).
    if matches!(tval, TV_WAND | TV_ROD | TV_STAFF) {
        for (_, units) in &mut out { *units = scale_percent(*units, param(Param::town_devices_percent)); }
    }
    out.retain(|&(_, units)| units > 0);
    if matches!(tval, TV_WAND | TV_STAFF) && item.pval > 0 {
        let maximum = engine::kind_value_average(engine::kind(item.kind as usize).charge).max(1);
        if i64::from(item.pval) < i64::from(maximum) * qty { out.push((RECHARGE_TARGET, UNIT)); }
    }
    out
}

/// Physical supply curves use the same forecast, gold opportunity cost and
/// capacity constraints as the core town model. No consumable calls borg_power.
pub fn observe(st: &BorgState, stock: &Stock, kit: &[KitItem], journey: Journey) -> Model {
    let t = |id| i64::from(st.borg.tr(id)).max(0);
    let cfg = |id| i64::from(st.cfg(id)).clamp(0, 1_000_000);
    let exposure = journey.exposure();
    let ordinary = cfg(BORG_TOWN_W_SUPPORT) * exposure;
    let risk = |id| cfg(id) * exposure;
    let support_demand = (journey.turns * UNIT / cfg(BORG_TOWN_SUPPORT_TURNS).max(1)).max(UNIT);
    let light_slot = crate::borg_item::inven_wield() as usize + 5;
    let light = st.items.items.get(light_slot).copied().unwrap_or_else(crate::borg_item::blank_item);
    let lantern = light.iqty > 0 && u32::from(light.tval) == TV_LIGHT && i32::from(light.sval) == st.item_val.sv_light_lantern
        && !engine::flag_has(&light.flags, crate::headers::list_object_flags_h::OF_NO_FUEL as i32);
    let ctx = Context { st, journey, lantern };
    let mut model = Model { available: stock.candidates.iter().map(|c| [i64::from(c.shop),i64::from(c.slot),i64::from(c.qty)]).collect(), packing: Some(crate::borg_town_packing::Packing::of(st, stock)), curves: (0..COUNT).map(|i| [i as i64, ordinary, support_demand, 0, 0]).collect(), supplies: Vec::new() };
    let mut curve = |id: usize, weight: i64, demand: i64| { model.curves[id][1] = weight; model.curves[id][2] = demand; };
    curve(LIGHT, cfg(BORG_TOWN_W_LIGHT) * UNIT, journey.turns * UNIT);
    // Nutrition is the game's food-value unit (food_value=100), not an equal
    // copy whitelist. One ration's nutrition (RATION_NUTRITION, 30) supports
    // food_turns actions.
    // borg-reborn-rs (food floor): never plan for fewer rations than the
    // core needs to be prepared for any level (borg_prepared "5 Food",
    // restock "food < 3"). borg-reborn-rs (#451): the core counts each
    // Remove Hunger scroll as one of the five (with_remove_hunger) and no
    // food the stores sell supplies more than the scroll's 35 nutrition (a
    // ration 30), so the planner's floor of 150 (five rations) is never met
    // with fewer than five of them, the core's five.
    // Sized to the trip alone, a short early trip
    // asked for about one ration, so the planner sold the starting rations
    // and the core then bounced between DL1 and town for "restock food".
    // Below the floor, food is a requirement (negative demand: worth its
    // weight linearly up to five rations, nothing past them), so it is
    // bought up to the floor and never sold below it, but not hoarded.
    let rations = journey.turns * UNIT / cfg(BORG_TOWN_FOOD_TURNS).max(1);
    if rations < READY_FOOD * UNIT {
        curve(FOOD, cfg(BORG_TOWN_W_FOOD) * READY_FOOD * UNIT, -(RATION_NUTRITION * READY_FOOD * UNIT));
    } else {
        curve(FOOD, cfg(BORG_TOWN_W_FOOD) * rations, RATION_NUTRITION * rations);
    }
    // borg-reborn-rs (#491, owner: "we can never have enough healing").
    // `town_heal_curve_scale_percent` stretches the diminishing healing curve:
    // weight and demand scale together, so its first HP is worth the same
    // (slope weight / demand = w_bleeding / max HP) and it reaches half its
    // value at scale x today's demand (100 = today).
    // #487 (owner): a class may override the scale and the reserve.
    let class = st.borg.tr(BI_CLASS);
    let scale = i64::from(heal_curve_scale_percent(class));
    curve(HEAL, risk(BORG_TOWN_W_BLEEDING) * scale / 100, journey.hp * exposure * scale / 100);
    // Up to `town_heal_reserve_hp_percent` of max HP of healing, each HP is
    // also worth that first-HP value, independent of the trip's exposure, on
    // top of the curve. A requirement (negative demand, like food's floor):
    // linear up to the reserve, nothing more past it, so healing is bought
    // first and never sold below it (0 = off).
    let reserve = journey.hp * i64::from(heal_reserve_hp_percent(class)) * UNIT / 100;
    curve(HEAL_RESERVE, if reserve > 0 { cfg(BORG_TOWN_W_BLEEDING) * reserve / journey.hp.max(1) } else { 0 }, -reserve.max(UNIT));
    curve(WOUNDS, 0, UNIT); // historical combined status ID is retained for replay
    for (cap, timed) in [(CUT, TMD_CUT), (STUN, TMD_STUN)] {
        let timer = i64::from(engine::player_timed(timed)).max(0);
        curve(cap, if timer > 0 { cfg(BORG_TOWN_W_BLEEDING) * UNIT } else { 0 }, -(timer * UNIT).max(UNIT));
    }
    curve(CONTROL, ordinary, exposure.max(UNIT));
    curve(POISON, if t(BI_ISPOISONED) > 0 { cfg(BORG_TOWN_W_POISON) * UNIT } else { 0 },
        -(i64::from(engine::player_timed(TMD_POISONED)).max(1) * UNIT));
    // borg-reborn-rs (#682, owner): Phase Door's curve stretched per class, as
    // healing's: weight and demand scale together (100 = today).
    let phase_scale = i64::from(param(Param::town_phase_curve_scale_percent));
    curve(PHASE, risk(BORG_TOWN_W_ESCAPE) * phase_scale / 100, exposure * phase_scale / 100);
    // borg-reborn-rs (#557): `town_recall_reserve` scrolls are a requirement
    // (worth the full trip weight each, nothing past them) at any depth: the
    // walk it saves is always worth less than the full weight (the journey
    // includes the exploring), so a shallow Borg otherwise never buys one and
    // walks home, and the walk can fail (priest-1055640847).
    // borg-reborn-rs (#557, owner): `town_recall_value_percent` scales what
    // the walk a scroll saves is worth (100 = as computed).
    let walk_value = recall_value_scaled(cfg(BORG_TOWN_W_TRIP) * UNIT * journey.walk / journey.turns.max(journey.walk).max(1));
    let recall_reserve = i64::from(param(Param::town_recall_reserve)).max(0);
    if recall_reserve > 0 {
        curve(RECALL, cfg(BORG_TOWN_W_TRIP) * UNIT * recall_reserve, -(recall_reserve * UNIT));
    } else {
        curve(RECALL, walk_value, UNIT);
    }
    let damage_scale = journey.level * 2 + 4;
    curve(ATTACK, risk(BORG_TOWN_W_ATTACK), damage_scale * journey.turns * UNIT / cfg(BORG_TOWN_ATTACK_TURNS).max(1));
    // Reliable bow ammunition is a separate reserve for Rangers. Oil, devices
    // and spells supply damage but cannot replenish usable missile shots.
    // borg-reborn-rs (#666): every class by its own percent (shooting skill),
    // less when it carries flasks or attack-wand charges.
    let missiles_weight = scale_percent(risk(BORG_TOWN_W_RANGER_MISSILES), missiles_percent(st.borg.tr(BI_CLASS), param(Param::town_missiles_percent)) as i32);
    let missiles_weight = missiles_other_ranged(missiles_weight, other_ranged(st), param(Param::town_missiles_other_ranged_percent));
    // borg-reborn-rs (#716, owner: "they do not need that many arrows"): with
    // `town_ranger_arrows_max` the reserve is a requirement, as Recall's: each
    // arrow up to the cap is worth the same, past it nothing. C's curve grows
    // with the trip and never stops paying (half value at turns / missile
    // turns), so 1-gold arrows piled up to 150-200 and their weight sent
    // Phase Door to the Home. 0 = C.
    // Past the cap (owner: "a soft cap at 50 and then a curve for additional
    // arrows utility, configurable, say 75"), shots count on EXTRA_SHOTS:
    // each worth less than the last, nothing past `town_ranger_arrows_taper`
    // shots in all (see extra_gain).
    let arrows_max = i64::from(param(Param::town_ranger_arrows_max));
    let arrows_taper = i64::from(param(Param::town_ranger_arrows_taper));
    if arrows_max > 0 {
        curve(RANGER_MISSILES, missiles_weight, -(arrows_max * UNIT));
    } else {
        curve(RANGER_MISSILES, missiles_weight,
              (journey.turns * UNIT / cfg(BORG_TOWN_RANGER_MISSILE_TURNS).max(1)).max(UNIT));
    }
    let taper = (arrows_taper - arrows_max).max(0);
    curve(EXTRA_SHOTS, if arrows_max > 0 && taper > 0 { missiles_weight } else { 0 }, taper.max(1) * UNIT);
    // borg-reborn-rs (#712): flasks kept to throw, on a curve of their own
    // that shots and wands cannot fill: a stack of sling shots saturated
    // ATTACK, so the flasks' marginal damage was worth almost nothing and the
    // weight repair stored them at the Home. Weighted as ATTACK (and by the
    // class's oil scale, #682); half its value at `town_oil_flasks` flasks.
    // 0 weight (C) unless `town_oil_apart`.
    let thrown_weight = if param(Param::town_oil_apart) != 0 {
        scale_percent(risk(BORG_TOWN_W_ATTACK), param(Param::town_oil_curve_scale_percent))
    } else { 0 };
    curve(THROWN_OIL, thrown_weight, i64::from(param(Param::town_oil_flasks)).max(1) * UNIT);
    // borg-reborn-rs (#452): Speed is an escape-like reserve, per potion
    // (see effect_supply_linear), not a buff against the trip's turns.
    for id in [TELEPORT, LEVEL, OTHER, SPEED] { curve(id, risk(BORG_TOWN_W_ESCAPE), exposure); }
    for cap in [TRAPS, MONSTERS] { curve(cap, ordinary, journey.turns * UNIT); }
    // borg-reborn-rs (#663, owner): Magic Mapping, Detect Invisible and Light
    // are "something you never buy": a read counted as borg_town_support_turns
    // of service, so each was worth about a Phase Door and they took a quarter
    // of the gold. A carried or found copy keeps a small value (kept, not sold).
    for cap in [INVISIBLE, MAP, ILLUMINATE] { curve(cap, low_value(ordinary), journey.turns * UNIT); }
    curve(MANA, if t(BI_MAXSP) > 0 { risk(BORG_TOWN_W_MAGIC) } else { 0 }, t(BI_MAXSP).max(1) * exposure);
    curve(IDENTIFY, if t(BI_ALL_NEED_ID) > 0 { cfg(BORG_TOWN_W_KNOWLEDGE) * UNIT } else { 0 }, -UNIT);
    curve(GAIN_EXPERIENCE, cfg(BORG_TOWN_W_RESTORE) * UNIT, -UNIT);
    curve(ACQUIRE, cfg(BORG_TOWN_W_POWER) * exposure, journey.advancement * UNIT);
    // borg-reborn-rs (#487, owner): a digger only clears rubble (the engine
    // swaps it in; a weapon digs rubble too, slower). Below
    // `town_digger_min_gold` gold no digging is valued, so the first visits
    // do not spend a third to half of the purse on a Shovel (0 = off).
    if i64::from(st.borg.tr(BI_GOLD)) < i64::from(param(Param::town_digger_min_gold)) {
        curve(DIG, 0, support_demand);
    }
    curve(EXPERIENCE, if t(BI_ISFIXEXP) > 0 || t(BI_ISFIXLEV) > 0 { cfg(BORG_TOWN_W_RESTORE) * UNIT } else { 0 }, -UNIT);
    curve(CURSE, if t(BI_FIRST_CURSED) > 0 { cfg(BORG_TOWN_W_RESTORE) * UNIT } else { 0 }, -UNIT);
    curve(ENCHANT, if t(BI_NEED_ENCHANT_TO_A) + t(BI_NEED_ENCHANT_TO_H) + t(BI_NEED_ENCHANT_TO_D) > 0 { cfg(BORG_TOWN_W_SUPPORT) * UNIT } else { 0 }, UNIT);
    curve(RECHARGE, cfg(BORG_TOWN_W_SUPPORT) * UNIT, -UNIT);
    curve(RECHARGE_TARGET, 0, UNIT);
    for (cap, resistance) in [(FIRE, BI_RFIRE), (COLD, BI_RCOLD), (RESPOIS, BI_RPOIS), (ACID, BI_RACID), (ELEC, BI_RELEC)] {
        // Temporary resistance still stacks with permanent resistance in combat;
        // halve its priority rather than deleting its capability.
        curve(cap, ordinary / if t(resistance) > 0 { 2 } else { 1 }, journey.turns * UNIT);
    }
    // borg-reborn-rs (#452): SPEED left this loop (the escape reserve above).
    curve(BUFF, ordinary, journey.turns * UNIT);
    // borg-reborn-rs (#516): not afraid, fear cures (a Potion of Boldness)
    // are worth only up to town_fear_cure_count in the pack (0 = the curve).
    let fear_cap = i64::from(param(Param::town_fear_cure_count));
    for (cap, condition) in [(CONFUSION, BI_ISCONFUSED), (BLIND, BI_ISBLIND), (FEAR, BI_ISAFRAID)] {
        // borg-reborn-rs (#663, owner): a fear cure (Boldness) is worth very
        // little while not afraid.
        let calm = if cap == FEAR { low_value(ordinary) } else { ordinary };
        curve(cap, if t(condition) > 0 { cfg(BORG_TOWN_W_RESTORE) * UNIT } else { calm }, if t(condition) > 0 {
            let timed = match cap { CONFUSION => TMD_CONFUSED, BLIND => TMD_BLIND, _ => TMD_AFRAID };
            -(i64::from(engine::player_timed(timed)).max(1) * UNIT)
        } else if cap == FEAR && fear_cap > 0 { -fear_cap * UNIT } else { exposure });
    }
    for stat in 0..5 {
        let drained = t(BI_ISFIXSTR + stat as u32) > 0;
        curve(STATS + stat, if drained || t(BI_CSTR + stat as u32) < 18 + 100 { cfg(BORG_TOWN_W_RESTORE) * UNIT } else { 0 }, -UNIT);
    }
    for cap in BOOKS..BOOKS + 9 { curve(cap, cfg(BORG_TOWN_W_MAGIC) * UNIT, -UNIT); }
    // Per-item physical supply facts are independent of the quantities held.
    for c in &stock.candidates {
        let item = &st.store.shops[usize::from(c.shop)].ware[usize::from(c.slot)];
        for (cap, n) in supplies(item, &ctx, (i64::from(c.shop), usize::from(c.slot))) {
            model.supplies.push([i64::from(c.shop), i64::from(c.slot), cap as i64, n]);
        }
    }
    for it in kit {
        let item = if it.place == Place::Pack { &st.items.items[usize::from(it.slot)] }
            else { &st.store.shops[crate::borg_store::BORG_HOME].ware[usize::from(it.slot)] };
        for (cap, n) in supplies(item, &ctx, (Model::scope(it), usize::from(it.slot))) {
            model.supplies.push([Model::scope(it), i64::from(it.slot), cap as i64, n]);
            model.curves[cap][if it.place == Place::Pack { 3 } else { 4 }] += n * i64::from(it.qty);
        }
    }
    // Equipment/quiver supplies are not movable pack stacks, but do close
    // the same needs (especially current fuel and already-loaded arrows).
    for item in st.items.items.iter().skip(crate::borg_item::inven_wield() as usize).filter(|i| i.iqty > 0) {
        for (cap, n) in supplies(item, &ctx, (10, 0)) { model.curves[cap][3] += n * i64::from(item.iqty); }
    }
    use crate::headers::list_object_flags_h::OF_NO_FUEL;
    if engine::player_has(crate::headers::list_player_flags_h::PF_UNLIGHT) || light.iqty > 0 && engine::flag_has(&light.flags, OF_NO_FUEL as i32) || t(BI_LIGHT) > 0 && light.iqty == 0 {
        model.curves[LIGHT][1] = 0;
    }
    // borg-reborn-rs (#380, #400): a learned spell never zeroes a physical
    // item's value. Mana is finite and shared, a spell can fail or be
    // unusable (blind, confused, out of mana), and broad capability classes
    // conflate different effects (treasure sensing is not mapping, Detect
    // Monsters misses invisible races). Scrolls, devices and potions keep
    // their supply value next to the spells.
    debug_assert!(model.valid());
    model
}

/// Optional engine-backed coverage diagnostic. These are hypothetical KNOWN
/// single items, never live inventory or receipts. It runs once per process,
/// after character birth, and mutates no player/Borg/game-RNG state.
pub fn audit(st: &BorgState, journey: Journey, model: &Model) {
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::io::Write;
    static DONE: AtomicBool = AtomicBool::new(false);
    let Ok(path) = std::env::var("BORG_TOWN_CAPABILITY_AUDIT") else { return; };
    if DONE.swap(true, Ordering::Relaxed) { return; }
    let Ok(mut file) = std::fs::File::create(path) else { return; };
    let light_slot = crate::borg_item::inven_wield() as usize + 5;
    let light = st.items.items.get(light_slot).copied().unwrap_or_else(crate::borg_item::blank_item);
    let ctx = Context { st, journey, lantern: light.iqty > 0 && i32::from(light.sval) == st.item_val.sv_light_lantern
        && !engine::flag_has(&light.flags, crate::headers::list_object_flags_h::OF_NO_FUEL as i32) };
    let _ = writeln!(file, "{{\"hypothetical_known_items\":1,\"class\":{},\"level\":{},\"curves\":{:?}}}", st.borg.tr(BI_CLASS), journey.level, model.curves);
    for kind in 1..usize::from(engine::z_info().k_max) {
        let k = engine::kind(kind);
        if k.name.is_null() { continue; }
        let mut item = crate::borg_item::blank_item();
        item.kind = kind as u32;
        item.tval = k.tval as u8; item.sval = k.sval as u8;
        item.iqty = 1; item.aware = true; item.flavour_aware = true;
        item.dd = k.dd as u8; item.ds = k.ds as u8;
        item.pval = engine::kind_value_average(k.pval) as i16;
        item.flags = k.flags;
        if matches!(k.tval as u32, TV_WAND | TV_STAFF) { item.pval = engine::kind_value_average(k.charge) as i16; }
        if k.tval as u32 == TV_LIGHT {
            item.timeout = if k.sval == st.item_val.sv_light_torch { engine::z_info().fuel_torch } else { engine::z_info().fuel_lamp } as i16;
        }
        let units = supplies(&item, &ctx, (0, 0));
        let units_json = units.iter().map(|(id, n)| format!("[{id},{n}]")).collect::<Vec<_>>().join(",");
        let _ = writeln!(file, "{{\"kind\":{},\"name\":{:?},\"tval\":{},\"sval\":{},\"supplies\":[{}],\"gain\":{},\"gear\":{}}}",
            kind, String::from_utf8_lossy(engine::kind_name(kind)), k.tval, k.sval, units_json,
            model.gain(&units), i32::from(crate::borg_inventory::borg_wield_slot(&item) >= 0));
    }
}

/// Merge intersecting capabilities, so substitute devices/consumables do
/// not each claim the same shortage. Preserve pure options plus a bounded
/// deterministic mixed path; the surrogate is not a global exact optimum.
pub fn shared_groups(groups: Vec<crate::borg_town_solve::Group>, input: &crate::borg_town_record::StepInput,
                     candidates: &[crate::borg_town_stock::Candidate], model: &Model, limits: &crate::borg_town_solve::Limits,
                     economics: (i64, i64, i64)) -> Vec<crate::borg_town_solve::Group> {
    let (space, carry, knowledge) = economics;
    use crate::borg_town_solve::{Choice, Group, GroupKey};
    let facts: BTreeMap<_, _> = input.facts.iter().map(|f| ((f.shop, f.slot), f)).collect();
    let mut out = Vec::new();
    let mut gear = BTreeMap::<usize,i64>::new();
    if input.revision >= 9 {
        for g in groups.iter().filter(|g| matches!(g.key, GroupKey::Slot(_))) {
            for o in g.options.iter().skip(1) {
                for &(i,n) in &o.picks {
                    let c = &candidates[i];
                    let learn = if facts.get(&(c.shop,c.slot)).is_some_and(|f| f.serves & (1 << 5) != 0) { knowledge * UNIT } else { 0 };
                    gear.insert(i, o.value - model.ware(c.shop,c.slot,n) - learn);
                }
            }
        }
    }
    let mut sets: Vec<(u64, Vec<Group>)> = Vec::new();
    for group in groups {
        if input.revision < 9 && matches!(group.key, GroupKey::Slot(_)) { out.push(group); continue; }
        let mut mask = 0;
        for option in &group.options {
            for &(i, _) in &option.picks { let c = &candidates[i]; mask |= model.mask(c.shop, c.slot);
                if model.packing.as_ref().is_some_and(|p| p.room(c.class).is_some()) { mask |= 1u64 << 63; } }
        }
        if mask == 0 { out.push(group); continue; }
        let mut joined = vec![group];
        let mut again = true;
        while again {
            again = false;
            let mut i = 0;
            while i < sets.len() {
                if sets[i].0 & mask != 0 {
                    let (m, mut gs) = sets.remove(i); mask |= m; joined.append(&mut gs); again = true;
                } else { i += 1; }
            }
        }
        sets.push((mask, joined));
    }
    let value = |picks: Vec<(usize, i32)>| {
        let mut choice = Choice { picks, ..Choice::default() };
        let mut units = Vec::new();
        let mut slots = BTreeSet::new();
        let mut learned = BTreeSet::new();
        for &(i, count) in &choice.picks {
            let c = &candidates[i];
            choice.cost += i64::from(count) * i64::from(c.price);
            choice.weight += i64::from(count) * i64::from(c.weight);
            if c.slots_cost > 0 { slots.insert(c.class); }
            units.extend(model.units(i64::from(c.shop), i64::from(c.slot), i64::from(count)));
            if facts.get(&(c.shop, c.slot)).is_some_and(|f| f.serves & (1 << crate::borg_town_utility::index(crate::borg_town_usefulness::Need::Knowledge)) != 0) { learned.insert(c.kind); }
        }
        choice.slots = slots.len() as i32;
        if let Some(p) = &model.packing { p.reprice(&mut choice, candidates); }
        choice.value = model.gain(&units) + choice.picks.iter().map(|&(i,_)| gear.get(&i).copied().unwrap_or(0)).sum::<i64>()
            + learned.len() as i64 * knowledge * UNIT
            - crate::borg_town_utility::burden(space, choice.slots, choice.weight, limits.pack_slots, carry);
        choice
    };
    for (mask, groups) in sets {
        let mut options = vec![Choice::default()];
        for g in &groups { for option in g.options.iter().skip(1) { options.push(value(option.picks.clone())); } }
        let mut positions = vec![0; groups.len()];
        loop {
            let picks = |pos: &[usize]| groups.iter().zip(pos).flat_map(|(g, &o)| g.options[o].picks.clone()).collect();
            let current = value(picks(&positions));
            let mut best: Option<(usize, usize, Choice)> = None;
            for (gi, g) in groups.iter().enumerate() {
                let alternatives: Vec<usize> = if input.revision >= 9 && matches!(g.key, GroupKey::Slot(_)) {
                    (1..g.options.len()).filter(|&oi| oi != positions[gi]).collect()
                } else { vec![positions[gi] + 1] };
                for oi in alternatives {
                if oi >= g.options.len() { continue; }
                let previous = positions[gi];
                positions[gi] = oi;
                let next = value(picks(&positions));
                positions[gi] = previous;
                let gain = next.value - current.value;
                let bm: i64 = next.picks.iter().filter(|&&(i, _)| candidates[i].is_black_market())
                    .map(|&(i, n)| i64::from(candidates[i].price) * i64::from(n)).sum();
                let caution = input.params[9].max(1);
                if gain <= 0 || next.cost > limits.gold || next.weight > limits.weight || next.slots > limits.pack_slots || bm.saturating_mul(caution) > limits.gold { continue; }
                let price = (next.cost - current.cost).max(1);
                if best.as_ref().is_none_or(|(_, _, b)| i128::from(gain) * i128::from((b.cost - current.cost).max(1)) > i128::from(b.value - current.value) * i128::from(price)) { best = Some((gi, oi, next)); }
                }
            }
            let Some((gi, oi, next)) = best else { break; };
            positions[gi] = oi;
            options.push(next);
        }
        out.push(Group { key: GroupKey::Need(mask.trailing_zeros() as u8), options });
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    fn model() -> Model {
        Model { available: Vec::new(), packing: None, curves: (0..COUNT).map(|i| [i as i64, 100 * UNIT, UNIT, 0, 0]).collect(), supplies: Vec::new() }
    }
    fn item(place: Place, slot: u8, count: i32) -> KitItem {
        KitItem { joins_pack: false, place, slot, kind: 1, qty: count, weight: 10, need: -1, reduction: 0,
            power: 0, proceeds: 0, sell_shop: 0, unknown: false, unknown_flavour: false, just_bought: false, serves: 0 }
    }
    /// #516 (review of #563): the fear-cure cap's value curve. With the cap
    /// at 2 (demand -2 units) one potion is worth half the weight, two the
    /// whole weight, and a third adds nothing, where the diminishing curve
    /// gave each potion a small, never-zero value.
    #[test]
    fn ranger_soft_cap_tapers_to_nothing() {
        // #716: 50 shots as a requirement, then a taper to nothing at 75.
        let w = 600 * UNIT;
        let mut m = Model { available: Vec::new(), packing: None, curves: (0..COUNT).map(|i| [i as i64, 0, UNIT, 0, 0]).collect(), supplies: Vec::new() };
        m.curves[RANGER_MISSILES] = [RANGER_MISSILES as i64, w, -50 * UNIT, 0, 0];
        m.curves[EXTRA_SHOTS] = [EXTRA_SHOTS as i64, w, 25 * UNIT, 0, 0];
        let shot = |m: &Model| m.gain(&[(RANGER_MISSILES, UNIT), (EXTRA_SHOTS, UNIT)]);
        let under = shot(&m);
        assert_eq!(under, w / 50, "a shot under the cap: the requirement's share");
        m.curves[RANGER_MISSILES][3] = 50 * UNIT;
        m.curves[EXTRA_SHOTS][3] = 50 * UNIT;
        let first = shot(&m);
        assert!(first > 0 && first < under, "the first shot past the cap is worth a little less");
        m.curves[RANGER_MISSILES][3] = 70 * UNIT;
        m.curves[EXTRA_SHOTS][3] = 70 * UNIT;
        let late = shot(&m);
        assert!(late > 0 && late < first / 3, "near the taper's end: little");
        m.curves[RANGER_MISSILES][3] = 75 * UNIT;
        m.curves[EXTRA_SHOTS][3] = 75 * UNIT;
        assert_eq!(shot(&m), 0, "past the taper: nothing");
        m.curves[RANGER_MISSILES][2] = 30 * UNIT; // no cap (C's curve): no extra value
        m.curves[RANGER_MISSILES][3] = 0;
        assert_eq!(m.gain(&[(EXTRA_SHOTS, UNIT)]), 0);
    }

    #[test]
    fn the_fear_cure_cap_values_potions_linearly_up_to_the_cap() {
        let w = 50_000;
        let capped = [FEAR as i64, w, -2 * UNIT, 0, 0];
        assert_eq!(coverage(&capped, UNIT), w / 2, "one potion");
        assert_eq!(coverage(&capped, 2 * UNIT), w, "two");
        assert_eq!(coverage(&capped, 3 * UNIT), w, "a third is worth nothing");
        assert_eq!(coverage(&capped, 5 * UNIT), w);
        let curve = [FEAR as i64, w, 100 * UNIT, 0, 0];
        let (one, three) = (coverage(&curve, UNIT), coverage(&curve, 3 * UNIT));
        assert!(one < w / 2 && three > coverage(&curve, 2 * UNIT), "the curve: small, still rising");
    }

    #[test]
    fn ranger_empty_quiver_remains_valuable_when_devices_cover_damage() {
        let mut m = model();
        m.curves[ATTACK] = [ATTACK as i64, 100 * UNIT, 60 * UNIT, 100000 * UNIT, 0];
        m.curves[RANGER_MISSILES] = [RANGER_MISSILES as i64, 250 * UNIT, 22 * UNIT, 0, 0];
        let arrow = missile_supply(CLASS_RANGER, 6 * UNIT);
        let empty = m.gain(&arrow);
        // A 1-gold arrow beats a 15-gold first-copy learning bonus of 25.
        assert!(empty * 15 > 25 * UNIT);
        assert!(empty > 100 * m.gain(&[(ATTACK, 6 * UNIT)]), "devices cannot close the shot reserve");
        m.curves[RANGER_MISSILES][3] = 40 * UNIT;
        assert!(m.gain(&arrow) < empty / 5, "already carried arrows reduce marginal buying priority");
        m.curves[RANGER_MISSILES][2] *= 2;
        assert!(m.gain(&arrow) > 0, "longer trips retain missile value");
        m.curves[RANGER_MISSILES][1] = 0;
        assert_eq!(m.gain(&arrow), m.gain(&[(ATTACK, 6 * UNIT)]), "zero weight restores generic attack valuation");
        assert_eq!(missile_supply(0, 6 * UNIT), [(ATTACK, 6 * UNIT)], "other classes keep their existing supply");
    }

    #[test]
    fn wand_teleport_other_and_stone_to_mud_are_services_not_damage() {
        use crate::headers::list_projections_h::*;
        assert_eq!(effect_supply(&[(EF_BOLT as u16, PROJ_AWAY_ALL as i32, 100)], 20, 1000), [(OTHER, UNIT)]);
        assert_eq!(effect_supply(&[(EF_BOLT as u16, PROJ_KILL_WALL as i32, 30)], 20, 1000), [(DIG, UNIT)]);
        let light = effect_supply(&[(EF_LINE as u16, PROJ_LIGHT_WEAK as i32, 6)], 20, 1000);
        assert_eq!(light, [(ATTACK, 6 * UNIT)], "the shop's Wand of Light has a LINE effect");
        for projection in [PROJ_MON_HEAL, PROJ_MON_SPEED, PROJ_MON_CLONE, PROJ_MON_POLY] {
            for effect in [EF_BOLT, EF_BOLT_STATUS, EF_BOLT_AWARE, EF_PROJECT_LOS] {
                assert!(effect_supply(&[(effect as u16, projection as i32, 100)], 20, 1000).is_empty());
            }
        }
        assert_eq!(effect_supply(&[(EF_BOLT_STATUS_DAM as u16, PROJ_MON_DRAIN as i32, 150)], 20, 1000), [(ATTACK, 150 * UNIT)]);
    }
    #[test]
    fn secondary_cures_and_buffs_are_not_discarded_by_a_primary_healing_role() {
        let units = effect_supply(&[(EF_HEAL_HP as u16, 0, 60), (EF_CURE as u16, TMD_CONFUSED as i32, 0),
            (EF_CURE as u16, TMD_POISONED as i32, 0), (EF_CURE as u16, TMD_BLIND as i32, 0)], 25, 1000);
        assert!(units.contains(&(HEAL, 25 * UNIT)));
        for cap in [CONFUSION, POISON, BLIND] { assert!(units.contains(&(cap, UNIT))); }
        let units = effect_supply(&[(EF_TIMED_INC as u16, TMD_FAST as i32, 30),
            (EF_TIMED_INC as u16, TMD_OPP_FIRE as i32, 20)], 25, 1000);
        // borg-reborn-rs (#452): speed is one use per effect, not its turns.
        assert!(units.contains(&(SPEED, UNIT)) && units.contains(&(FIRE, 20 * UNIT)));
    }
    #[test]
    fn food_uses_actual_nutrition_and_crunch_is_only_a_message() {
        let biscuit = effect_supply(&[(EF_NOURISH as u16, 0, 10), (EF_CRUNCH as u16, 0, 0)], 20, 1000);
        assert_eq!(biscuit, [(FOOD, 10 * UNIT)]);
        assert_eq!(effect_supply(&[(EF_NOURISH as u16, 0, 40)], 20, 1000), [(FOOD, 40 * UNIT)]);
        assert!(effect_supply(&[(EF_NOURISH as u16, 1, 40)], 20, 1000).is_empty());
        assert!(effect_supply(&[(EF_NOURISH as u16, 2, 0)], 20, 1000).is_empty());
    }
    /// borg-reborn-rs (#451): NOURISH:INC_TO fills to a percentage of a full
    /// stomach (effect_handler_NOURISH, subtype 3). Read hungry (15 percent,
    /// player_timed.txt) the Remove Hunger scroll (50) adds 35, more than a
    /// ration's INC_BY 30, and worth neither nothing nor a flat ration.
    #[test]
    fn remove_hunger_is_worth_what_it_adds_to_a_hungry_stomach() {
        let scroll = effect_supply(&[(EF_NOURISH as u16, 3, 50)], 20, 1000);
        assert_eq!(scroll, [(FOOD, 35 * UNIT)]);
        let ration = effect_supply(&[(EF_NOURISH as u16, 0, 30)], 20, 1000);
        assert_eq!(ration, [(FOOD, 30 * UNIT)]);
        assert!(scroll[0].1 > ration[0].1 && scroll[0].1 < 50 * UNIT);
        // a fill-to level at or under the hungry grade adds nothing
        assert!(effect_supply(&[(EF_NOURISH as u16, 3, 15)], 20, 1000).is_empty());
        assert!(effect_supply(&[(EF_NOURISH as u16, 3, 10)], 20, 1000).is_empty());
    }
    #[test]
    fn replenishment_competes_with_existing_and_substitute_stock() {
        let mut m = model();
        m.supplies = vec![[0, 0, ATTACK as i64, UNIT], [1, 0, ATTACK as i64, UNIT]];
        let first = m.ware(0, 0, 1);
        let two_kinds = m.gain(&[(ATTACK, UNIT), (ATTACK, UNIT)]);
        assert!(two_kinds < first * 2, "two kinds do not each claim the full shortage");
        m.curves[ATTACK][3] = 10 * UNIT;
        assert!(m.ware(0, 0, 1) < first);
        m.curves[ATTACK][2] = -UNIT; // one-time access/repair, such as a class book
        assert_eq!(m.ware(0, 0, 1), 0);
    }
    #[test]
    fn keeping_and_buying_use_identical_units_and_home_withdrawals_reprice() {
        let mut m = model();
        m.supplies = vec![[PACK, 0, ATTACK as i64, UNIT], [HOME, 0, ATTACK as i64, UNIT]];
        let pack = item(Place::Pack, 0, 1);
        let home = item(Place::Home, 0, 1);
        m.curves[ATTACK][3] = UNIT;
        let loss = m.keep(&pack);
        m.change(&pack, -1);
        assert_eq!(loss, m.take(&home));
        let first = m.take(&home);
        m.change(&home, 1);
        assert!(m.take(&home) < first);
        m.curves[ATTACK][4] = UNIT;
        assert!(m.reserve(&home, -1, 500) < 0);
    }
    #[test]
    fn selling_or_storing_the_recharge_target_removes_the_purchase_demand() {
        let mut m = model();
        m.curves[RECHARGE_TARGET][1] = 0;
        m.curves[RECHARGE_TARGET][3] = UNIT;
        m.supplies.push([PACK, 0, RECHARGE_TARGET as i64, UNIT]);
        assert!(m.gain(&[(RECHARGE, UNIT)]) > 0);
        m.change(&item(Place::Pack, 0, 1), -1);
        assert_eq!(m.gain(&[(RECHARGE, UNIT)]), 0);
    }
    #[test]
    fn owning_a_usable_class_book_closes_access_without_a_duplicate_stock_target() {
        let mut m = model();
        m.curves[BOOKS][2] = -UNIT;
        m.supplies.push([4, 0, BOOKS as i64, UNIT]);
        assert!(m.ware(4, 0, 1) > 0);
        assert_eq!(m.ware(4, 0, 2), m.ware(4, 0, 1));
        m.curves[BOOKS][3] = UNIT;
        assert_eq!(m.ware(4, 0, 1), 0);
    }
    #[test]
    fn temporary_sight_and_snapshot_detection_share_time_units() {
        assert_eq!(effect_supply(&[(EF_DETECT_INVISIBLE_MONSTERS as u16, 0, 0)], 20, 1000), [(INVISIBLE, 1000 * UNIT)]);
        assert_eq!(effect_supply(&[(EF_TIMED_INC as u16, TMD_SINVIS as i32, 30)], 20, 1000), [(INVISIBLE, 30 * UNIT)]);
    }
    #[test]
    fn schema_seven_accepts_its_earlier_append_only_table() {
        let mut m = model();
        m.curves.truncate(53);
        assert!(m.valid());
        assert!(m.gain(&[(ATTACK, UNIT)]) > 0);
    }
    #[test]
    fn oil_is_allocated_to_fuel_or_ammunition_without_counting_a_bottle_twice() {
        assert_eq!(oil_supply(1, 1, 2500, 7500), [(LIGHT, 7500 * UNIT)]);
        // #649: a flask of oil (1d4, weight 20, exploding) does 22.5, as the engine.
        assert_eq!(thrown_damage(1, 4, 20, true), 22_500);
        assert_eq!(thrown_damage(1, 4, 5, false), 5_000, "an Iron Shot thrown by hand");
        assert_eq!(thrown_damage(1, 2, 4, false), 3_000, "a Rounded Pebble thrown by hand");
        assert_eq!(oil_supply(1, 0, 2500, 7500), [(ATTACK, 2500)]);
        let held = oil_supply(5, 2, 2500, 7500);
        assert_eq!(held, [(LIGHT, 3000 * UNIT), (ATTACK, 1500)]);
        assert_eq!(held[0].1 * 5 / (7500 * UNIT) + held[1].1 * 5 / 2500, 5);
    }
    #[test]
    fn malformed_replay_facts_cannot_index_outside_the_capability_table() {
        let mut m = model();
        assert!(m.valid());
        m.supplies.push([0, 0, COUNT as i64, UNIT]);
        assert!(!m.valid());
        m.supplies[0][2] = 0;
        m.supplies[0][3] = -UNIT;
        assert!(!m.valid());
    }
    #[test]
    fn unsupported_preparation_and_harmful_projections_have_no_combat_value() {
        use crate::headers::list_projections_h::*;
        assert!(effect_supply(&[(EF_TIMED_INC as u16,TMD_ATT_CONF as i32,10000)],20,1000).is_empty());
        for (e,sub) in [(EF_SPHERE,PROJ_DARK_WEAK),(EF_TOUCH,PROJ_MAKE_TRAP)] {
            assert!(effect_supply(&[(e as u16,sub as i32,20)],20,1000).is_empty());
        }
        assert_eq!(effect_supply(&[(EF_BOLT_STATUS as u16,PROJ_AWAY_ALL as i32,100)],20,1000),[(OTHER,UNIT)]);
    }
    #[test]
    fn m13_confusion_wand_has_a_consumer_but_the_staff_does_not() {
        use crate::headers::list_projections_h::PROJ_MON_CONF;
        let effect=[(EF_BOLT_AWARE as u16,PROJ_MON_CONF as i32,20)];
        assert_eq!(effect_supply_for_consumer(&effect,20,1000,true),[(CONTROL,UNIT)]);
        assert!(effect_supply_for_consumer(&effect,20,1000,false).is_empty());
        let alternative=[(EF_RANDOM as u16,0,2),effect[0],(EF_BOLT_AWARE as u16,crate::headers::list_projections_h::PROJ_MON_STUN as i32,20)];
        assert!(effect_supply_for_consumer(&alternative,20,1000,true).is_empty(),"unsupported random alternatives do not acquire guaranteed control");
    }

    #[test]
    fn shop_control_requires_a_working_consumer() {
        use crate::headers::list_projections_h::*;
        for effect in [EF_BOLT_AWARE, EF_PROJECT_LOS_AWARE] {
            for projection in [PROJ_MON_CONF, PROJ_MON_STUN, PROJ_SLEEP_UNDEAD,
                PROJ_TURN_EVIL, PROJ_TURN_LIVING, PROJ_TURN_UNDEAD] {
                assert!(effect_supply(&[(effect as u16,projection as i32,20)],20,1000).is_empty());
            }
            for projection in [PROJ_MON_SLOW, PROJ_MON_HOLD, PROJ_SLEEP_ALL, PROJ_SLEEP_EVIL, PROJ_TURN_ALL] {
                assert_eq!(effect_supply(&[(effect as u16,projection as i32,20)],20,1000),[(CONTROL,UNIT)]);
            }
        }
    }

    #[test]
    fn mutually_exclusive_damage_branches_are_not_added() {
        use crate::headers::list_elements_h::{PROJ_FIRE,PROJ_COLD};
        for e in [EF_RANDOM,EF_SELECT] {
            assert_eq!(effect_supply(&[(e as u16,0,2),(EF_BALL as u16,PROJ_FIRE as i32,80),
                (EF_BALL as u16,PROJ_COLD as i32,80)],20,1000),[(ATTACK,80*UNIT)]);
            assert!(effect_supply(&[(e as u16,0,2),(EF_HEAL_HP as u16,0,20),
                (EF_BALL as u16,PROJ_FIRE as i32,80)],20,1000).is_empty(),"no guaranteed shared service");
        }
    }
    #[test]
    fn partial_timed_reductions_leave_value_for_another_dose() {
        let mut m=model();m.curves[CONFUSION]=[CONFUSION as i64,100*UNIT,-60*UNIT,0,0];
        let partial=effect_supply(&[(EF_TIMED_DEC as u16,TMD_CONFUSED as i32,20)],20,1000);
        assert_eq!(partial,[(CONFUSION,20*UNIT)]);
        let first=m.gain(&partial);m.curves[CONFUSION][3]+=20*UNIT;
        assert!(first>0 && m.gain(&partial)>0);
        m.curves[CONFUSION][3]=60*UNIT;assert_eq!(m.gain(&partial),0);
    }
    #[test]
    fn stacked_rods_keep_the_ready_rod_and_respect_charging_count() {
        assert_eq!(rod_uses(2,40,40,10),1);
        assert_eq!(rod_uses(2,40,40,20),1);
        assert_eq!(rod_uses(2,80,40,10),0);
        assert_eq!(rod_uses(2,0,40,2),2);
        assert_eq!(rod_uses(1,0,40,41),2);
    }
    #[test]
    fn wearable_and_fuel_share_one_curve_and_keep_slot_exclusivity() {
        use crate::borg_town_stock::Candidate;
        use crate::borg_town_solve::{Choice,Group,GroupKey,Limits};
        let cs:Vec<_>=(0..3).map(|i|Candidate {shop:0,slot:i,kind:u32::from(i)+1,tval:0,sval:0,
            price:1,qty:1,weight:0,slots_cost:1,stack_count:0,class:u16::from(i)}).collect();
        let mut m=model();m.supplies=(0..3).map(|i|[0,i,LIGHT as i64,UNIT]).collect();
        let choice=|i|Choice {picks:vec![(i,1)],cost:1,value:m.ware(0,i as u8,1),slots:1,weight:0};
        let groups=vec![Group {key:GroupKey::Slot(5),options:vec![Choice::default(),choice(0),choice(1)]},
            Group {key:GroupKey::Class(2),options:vec![Choice::default(),choice(2)]}];
        let limits=Limits {gold:100,pack_slots:5,weight:100};
        let input=crate::borg_town_record::StepInput {revision:9,capability:Some(&m),utility:true,candidates:cs.clone(),facts:&[],tryon:&[],limits,
            urgent:vec![],params:[0;crate::borg_town_record::PARAMS],traits:&[],power:0,carried:&[],cure_spell:false,kit:&[],home_slots:10};
        let merged=shared_groups(groups,&input,&cs,&m,&limits,(0,100,0));assert_eq!(merged.len(),1);
        for option in &merged[0].options {
            assert!(option.picks.iter().filter(|&&(i,_)|i<2).count()<=1);
            assert_eq!(option.value,m.gain(&[(LIGHT,option.picks.len() as i64*UNIT)]));
        }
        assert!(merged[0].options.iter().any(|o|o.picks.len()==2));
    }
    #[test]
    fn home_sale_changes_the_next_reserve_marginal() {
        let mut m=model();let home=item(Place::Home,0,1);m.supplies=vec![[HOME,0,ATTACK as i64,UNIT]];
        m.curves[ATTACK][4]=2*UNIT;let first=m.reserve(&home,-1,500);
        m.change_home(&home,-1);let second=m.reserve(&home,-1,500);
        assert!(second<first,"the final remaining reserve is more valuable");
    }

    /// #491: a healing supply fills the reserve too; below the reserve each
    /// potion adds the same value on top of the diminishing curve, past it
    /// only the curve's.
    #[test]
    fn healing_reserve_is_linear_up_to_its_size_then_the_curve() {
        let hp = 40;
        let units = effect_supply_linear(&[(EF_HEAL_HP as u16, 0, 20)], hp, 1, false);
        assert!(units.contains(&(HEAL, 20 * UNIT)) && units.contains(&(HEAL_RESERVE, 20 * UNIT)));
        let mut m = model();
        // Healing curve: weight 100 per HP of demand 1000 HP; reserve of 3
        // potions (60 HP) at the curve's first-HP slope (weight / demand).
        m.curves[HEAL] = [HEAL as i64, 100 * UNIT, 1000 * UNIT, 0, 0];
        let reserve = 3 * 20 * UNIT;
        m.curves[HEAL_RESERVE] = [HEAL_RESERVE as i64, 100 * UNIT * reserve / (1000 * UNIT), -reserve, 0, 0];
        m.supplies.push([0, 0, HEAL as i64, 20 * UNIT]);
        m.supplies.push([0, 0, HEAL_RESERVE as i64, 20 * UNIT]);
        let curve_only = |m: &Model| coverage(&m.curves[HEAL], m.curves[HEAL][3] + 20 * UNIT) - coverage(&m.curves[HEAL], m.curves[HEAL][3]);
        let mut extra = Vec::new();
        for _ in 0..4 {
            extra.push(m.ware(0, 0, 1) - curve_only(&m));
            m.curves[HEAL][3] += 20 * UNIT;
            m.curves[HEAL_RESERVE][3] += 20 * UNIT;
        }
        assert!(extra[0] > 0 && extra[0] == extra[1] && extra[1] == extra[2], "{extra:?}");
        assert_eq!(extra[3], 0, "nothing past the reserve");
        // Off: zero weight adds nothing.
        m.curves[HEAL_RESERVE][1] = 0;
        assert_eq!(m.ware(0, 0, 1), curve_only(&m));
    }

    /// #491: healing speed. At 66%, Cure Light Wounds (20 HP) counts in full
    /// up to 30 max HP and less above; one that heals the whole of max HP
    /// always counts in full; 0 is HP alone.
    #[test]
    fn healing_units_weigh_the_share_of_max_hp_one_swallow_heals() {
        assert_eq!(heal_units(20, 30, 66), 20 * UNIT);
        assert!(heal_units(20, 31, 66) < 20 * UNIT);
        assert_eq!(heal_units(20, 300, 66), 20 * 20 * UNIT * 100 / (300 * 66));
        assert_eq!(heal_units(60, 300, 66), 60 * 60 * UNIT * 100 / (300 * 66));
        assert_eq!(heal_units(300, 300, 66), 300 * UNIT);
        assert_eq!(heal_units(1200, 300, 66), 300 * UNIT);
        assert_eq!(heal_units(20, 300, 0), 20 * UNIT);
        assert_eq!(heal_units(20, 10, 66), 10 * UNIT);
        // #487: at 108 max HP a Cure Light Wounds is ~5.6 HP on the curve,
        // but fills the reserve with its 20 real HP.
        let cure_light = heal_units(20, 108, 66);
        assert!(cure_light < 6 * UNIT);
        assert_eq!(reserve_heal(cure_light, heal_units(20, 108, 0), true), 20 * UNIT);
        assert_eq!(reserve_heal(cure_light, heal_units(20, 108, 0), false), cure_light);
    }
}

#[cfg(test)]
mod bold_is_not_a_buff_tests {
    use super::*;

    /// #486: a Bold timed effect (wine's) supplies nothing, while a fear
    /// cure (a Potion of Boldness, CURE:AFRAID) still supplies FEAR.
    #[test]
    fn a_bold_effect_supplies_nothing_and_a_fear_cure_still_supplies_fear() {
        let wine = effect_supply(&[(EF_TIMED_INC as u16, TMD_BOLD as i32, 99)], 20, 1000);
        assert!(wine.iter().all(|&(cap, _)| cap != BUFF), "{wine:?}");
        let boldness = effect_supply(&[(EF_CURE as u16, TMD_AFRAID as i32, 0)], 20, 1000);
        assert!(boldness.iter().any(|&(cap, units)| cap == FEAR && units > 0), "{boldness:?}");
    }

    /// #486: Boldness is no buff to the mapping; the buffs the core does
    /// take stay.
    #[test]
    fn bold_is_not_a_buff() {
        assert_eq!(timed_cap(TMD_BOLD), None);
        assert_eq!(timed_cap(TMD_HERO), Some(BUFF));
        assert_eq!(timed_cap(TMD_SHERO), Some(BUFF));
        assert_eq!(timed_cap(TMD_BLESSED), Some(BUFF));
    }
}

#[cfg(test)]
mod class_heal_tests {
    use super::*;

    /// #487: with no class override (the compiled defaults) every class
    /// reads the general scale and reserve.
    #[test]
    fn class_overrides_fall_back_to_the_general_values() {
        for class in 0..9 {
            assert_eq!(heal_curve_scale_percent(class), param(Param::town_heal_curve_scale_percent));
            assert_eq!(heal_reserve_hp_percent(class), param(Param::town_heal_reserve_hp_percent));
        }
        assert_eq!(heal_curve_scale_percent(99), param(Param::town_heal_curve_scale_percent));
        assert_eq!(heal_reserve_hp_percent(99), param(Param::town_heal_reserve_hp_percent));
    }

    #[test]
    fn shots_for_every_class_by_its_percent() {
        // #666: -1 keeps C (the Ranger only); a percent applies to any class.
        assert_eq!(missiles_percent(CLASS_RANGER, -1), 100);
        assert_eq!(missiles_percent(0, -1), 0);
        assert_eq!(missiles_percent(0, 75), 75);
        assert_eq!(missiles_percent(CLASS_RANGER, 100), 100);
        // Fewer shots wanted with flasks or attack-wand charges in the pack.
        assert_eq!(missiles_other_ranged(1000, 4, 50), 1000);
        assert_eq!(missiles_other_ranged(1000, 5, 50), 500);
        assert_eq!(missiles_other_ranged(1000, 20, 100), 1000);
    }

    #[test]
    fn what_the_borg_never_buys_keeps_a_small_value() {
        // #663: one percent, above zero so a carried copy is kept.
        assert_eq!(low_value(55_000), 550);
        assert!(low_value(55_000) > 0);
        let scroll = coverage(&[MAP as i64, low_value(55_000), 1_100_000, 0, 0], 1_000_000);
        let phase = coverage(&[PHASE as i64, 55_000, 1_100, 0, 0], 1_000);
        assert!(scroll * 50 < phase, "{scroll} against a Phase Door's {phase}");
    }
}

/// borg-reborn-rs (#663, owner): the weight of what the Borg never buys
/// (Magic Mapping, Detect Invisible, Light, a fear cure while not afraid): one
/// percent of the ordinary weight, so a copy it carries still counts.
fn low_value(ordinary: i64) -> i64 {
    ordinary / 100
}

/// borg-reborn-rs (#557, owner): a Word of Recall's value (the walk it
/// saves) times `town_recall_value_percent` / 100.
pub(crate) fn recall_value_scaled(value: i64) -> i64 {
    scale_percent(value, param(Param::town_recall_value_percent))
}

fn scale_percent(value: i64, percent: i32) -> i64 {
    value.saturating_mul(i64::from(percent.max(0))) / 100
}

#[cfg(test)]
mod recall_value_tests {
    use super::scale_percent;

    #[test]
    fn the_recall_value_scales_by_its_percent() {
        assert_eq!(scale_percent(1000, 100), 1000, "100 = as computed");
        assert_eq!(scale_percent(1000, 500), 5000, "five times");
        assert_eq!(scale_percent(1000, 0), 0);
        assert_eq!(scale_percent(1000, -5), 0, "never negative");
    }
}
