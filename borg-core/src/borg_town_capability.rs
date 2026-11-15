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
const CLASS_RANGER: i32 = 7;
const COUNT: usize = NAMES.len();

pub const NAMES: [&str; 58] = ["light turns", "nutrition", "healing HP", "poison cure", "phase", "Recall",
    "ranged damage", "teleport", "level escape", "teleport other", "digging", "trap/door detection",
    "monster detection", "invisible detection", "mapping", "illumination", "speed", "resist fire",
    "resist cold", "resist poison", "resist acid", "resist electricity", "combat buff", "mana",
    "restore experience", "remove curse", "enchantment", "recharging", "confusion cure", "blindness cure",
    "fear cure", "glyph", "strength", "intelligence", "wisdom", "dexterity", "constitution", "charisma",
    "book 0", "book 1", "book 2", "book 3", "book 4", "book 5", "book 6", "book 7", "book 8",
    "banishment", "destruction", "wound/stun cure", "identify runes", "gain experience", "recharge target", "equipment opportunity", "cut reduction", "stun reduction", "monster control", "Ranger usable shots"];

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
        TMD_HERO | TMD_SHERO | TMD_BOLD | TMD_BLESSED | TMD_SHIELD | TMD_STONESKIN | TMD_PROTEVIL
            | TMD_INVULN | TMD_ATT_ACID | TMD_ATT_ELEC | TMD_ATT_FIRE | TMD_ATT_COLD
            | TMD_ATT_POIS | TMD_ATT_EVIL | TMD_ATT_DEMON | TMD_ATT_VAMP | TMD_POWERSHOT => Some(BUFF),
        TMD_SINVIS => Some(INVISIBLE), TMD_TELEPATHY => Some(MONSTERS),
        _ => None,
    }
}

/// RANDOM/SELECT alternatives are mutually exclusive. Use the guaranteed
/// component-wise service rather than adding incompatible branches.
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
    for &(effect, subtype, amount) in effects {
        let n = amount.max(1) * UNIT;
        let service = service_turns.max(1) * UNIT;
        let role = match u32::from(effect) {
            EF_NOURISH if subtype != 1 && amount > 0 => Some((FOOD, n)), // A decreasing food effect is not nourishment.
            EF_HEAL_HP => Some((HEAL, amount.max(1).min(hp) * UNIT)),
            EF_CURE => reduction_cap(subtype as u32).map(|cap| (cap, n)),
            EF_TIMED_DEC if amount > 0 => reduction_cap(subtype as u32).map(|cap| (cap, n)),
            EF_TIMED_INC | EF_TIMED_INC_NO_RES | EF_TIMED_SET => timed_cap(subtype as u32).map(|cap| (cap, n)),
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
    out.into_iter().collect()
}

fn oil_supply(quantity: i64, fuel_bottles: i64, damage: i64, fuel: i64) -> Vec<(usize, i64)> {
    let mut out = Vec::new();
    let n = quantity.max(1);
    if fuel_bottles > 0 && fuel > 0 { out.push((LIGHT, fuel * UNIT * fuel_bottles / n)); }
    if n > fuel_bottles { out.push((ATTACK, damage * (n - fuel_bottles) / n)); }
    out
}

// Called only after native launcher compatibility and curse checks. Each
// arrow contributes both damage and one reliable shot, separate objectives.
fn missile_supply(class: i32, damage: i64) -> Vec<(usize, i64)> {
    let mut out = vec![(ATTACK, damage)];
    if class == CLASS_RANGER { out.push((RANGER_MISSILES, UNIT)); }
    out
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
        let damage = (i64::from(item.dd) * (i64::from(item.ds) + 1) * UNIT / 2).max(UNIT);
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
    // copy whitelist. One ration's 40 nutrition supports food_turns actions.
    curve(FOOD, cfg(BORG_TOWN_W_FOOD) * journey.turns * UNIT / cfg(BORG_TOWN_FOOD_TURNS).max(1),
          journey.turns * 40 * UNIT / cfg(BORG_TOWN_FOOD_TURNS).max(1));
    curve(HEAL, risk(BORG_TOWN_W_BLEEDING), journey.hp * exposure);
    curve(WOUNDS, 0, UNIT); // historical combined status ID is retained for replay
    for (cap, timed) in [(CUT, TMD_CUT), (STUN, TMD_STUN)] {
        let timer = i64::from(engine::player_timed(timed)).max(0);
        curve(cap, if timer > 0 { cfg(BORG_TOWN_W_BLEEDING) * UNIT } else { 0 }, -(timer * UNIT).max(UNIT));
    }
    curve(CONTROL, ordinary, exposure.max(UNIT));
    curve(POISON, if t(BI_ISPOISONED) > 0 { cfg(BORG_TOWN_W_POISON) * UNIT } else { 0 },
        -(i64::from(engine::player_timed(TMD_POISONED)).max(1) * UNIT));
    curve(PHASE, risk(BORG_TOWN_W_ESCAPE), exposure);
    curve(RECALL, cfg(BORG_TOWN_W_TRIP) * UNIT * journey.walk / journey.turns.max(journey.walk).max(1), UNIT);
    let damage_scale = journey.level * 2 + 4;
    curve(ATTACK, risk(BORG_TOWN_W_ATTACK), damage_scale * journey.turns * UNIT / cfg(BORG_TOWN_ATTACK_TURNS).max(1));
    // Reliable bow ammunition is a separate reserve for Rangers. Oil, devices
    // and spells supply damage but cannot replenish usable missile shots.
    curve(RANGER_MISSILES, if st.borg.tr(BI_CLASS) == CLASS_RANGER { risk(BORG_TOWN_W_RANGER_MISSILES) } else { 0 },
          (journey.turns * UNIT / cfg(BORG_TOWN_RANGER_MISSILE_TURNS).max(1)).max(UNIT));
    for id in [TELEPORT, LEVEL, OTHER] { curve(id, risk(BORG_TOWN_W_ESCAPE), exposure); }
    for cap in [TRAPS, MONSTERS, INVISIBLE, MAP, ILLUMINATE] { curve(cap, ordinary, journey.turns * UNIT); }
    curve(MANA, if t(BI_MAXSP) > 0 { risk(BORG_TOWN_W_MAGIC) } else { 0 }, t(BI_MAXSP).max(1) * exposure);
    curve(IDENTIFY, if t(BI_ALL_NEED_ID) > 0 { cfg(BORG_TOWN_W_KNOWLEDGE) * UNIT } else { 0 }, -UNIT);
    curve(GAIN_EXPERIENCE, cfg(BORG_TOWN_W_RESTORE) * UNIT, -UNIT);
    curve(ACQUIRE, cfg(BORG_TOWN_W_POWER) * exposure, journey.advancement * UNIT);
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
    for cap in [SPEED, BUFF] { curve(cap, ordinary, journey.turns * UNIT); }
    for (cap, condition) in [(CONFUSION, BI_ISCONFUSED), (BLIND, BI_ISBLIND), (FEAR, BI_ISAFRAID)] {
        curve(cap, if t(condition) > 0 { cfg(BORG_TOWN_W_RESTORE) * UNIT } else { ordinary }, if t(condition) > 0 {
            let timed = match cap { CONFUSION => TMD_CONFUSED, BLIND => TMD_BLIND, _ => TMD_AFRAID };
            -(i64::from(engine::player_timed(timed)).max(1) * UNIT)
        } else { exposure });
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
    // Reliable renewable spells cover a capability without pretending they
    // are 1000 physical potions. Books and mana must actually be available.
    for (si, spell) in st.magic.magics.iter().enumerate() {
        if spell.status < crate::borg_magic::BORG_MAGIC_TEST || spell.rating == 0
            || !crate::borg_magic::borg_spell_legal_fail(st, spell.spell_enum, 20) { continue; }
        let effects = engine::spell_effect_values(si as i32, journey.level as i32);
        // Only information/access services suppress duplicate purchases.
        // Mana-free attacks and healing remain valuable alongside spells.
        if effects.iter().any(|&(e, _, amount)| u32::from(e) == EF_TELEPORT && amount <= 10) { model.curves[PHASE][1] = 0; }
        for (cap, _) in effect_supply(&effects, journey.hp, cfg(BORG_TOWN_SUPPORT_TURNS).max(1)) {
            if matches!(cap, TRAPS | MONSTERS | INVISIBLE | MAP | ILLUMINATE | DIG | RECALL | FOOD) {
                model.curves[cap][1] = 0;
            }
        }
    }
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
        assert!(units.contains(&(SPEED, 30 * UNIT)) && units.contains(&(FIRE, 20 * UNIT)));
    }
    #[test]
    fn food_uses_actual_nutrition_and_crunch_is_only_a_message() {
        let biscuit = effect_supply(&[(EF_NOURISH as u16, 0, 10), (EF_CRUNCH as u16, 0, 0)], 20, 1000);
        assert_eq!(biscuit, [(FOOD, 10 * UNIT)]);
        assert_eq!(effect_supply(&[(EF_NOURISH as u16, 0, 40)], 20, 1000), [(FOOD, 40 * UNIT)]);
        assert!(effect_supply(&[(EF_NOURISH as u16, 1, 40)], 20, 1000).is_empty());
        assert!(effect_supply(&[(EF_NOURISH as u16, 2, 0)], 20, 1000).is_empty());
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

}
