//! borg-reborn-rs (#840, owner 2026-11-21: "we need a mode 'not enough basic
//! kit' and they have to scale down their curiosity until their kit is
//! sufficient again"): the basic kit, a score, and what follows from it.
//!
//! The kit is what a low-level character needs before it may wander:
//! cure potions, Phase Door, something to throw or shoot, light, food.
//! Each item's share is what it holds over its target, capped at 1; the
//! kit score is the weighted mean of the shares (permille here: integers
//! only, so a game replays from its key stream). Everything below is behind
//! its own switch and every switch is 0 at the shipped values, so play is
//! unchanged until a setting turns one on.
//!
//! * `kit_leash`: the leash shrinks with the score
//!   (`kit_leash_min + (C's leash - kit_leash_min) x score^kit_leash_power`),
//!   and applies (as C's own leash does, but at any depth) while the kit is
//!   short; the hunts and item detours that the leash refuses stay refused.
//! * `kit_descent_min_percent`: no descent below a score. `borg_prepared` is
//!   asked about many depths (the power dives at +5, the recall probes at +20
//!   and 60% of the deepest level, 99), so the floor refuses every depth below
//!   the current one, not only the next: a Borg short of kit is not prepared
//!   for any of them. A depth at or above the Borg's own (the probe of 99
//!   from DL100) is never refused, so nothing here keeps a Borg from going
//!   up or to town.
//! * `kit_leave_danger_percent`: at the first real danger, take the stairs.
//! * `kit_restock` (remedy a): the kit is part of `borg_restock` at every
//!   depth, with a light-turns check.
//! * `kit_start_reserve` (remedy b): the town planner keeps a reserve of
//!   cures and Phase Door before it spends on missiles and oil.
//! * `kit_money_first` (remedy c): short of kit and too poor to fix it, the
//!   Borg stays on DL1 near the stairs and goes to town once it can pay.
//!
//! The numbers are hypotheses (RULES 14): the issue's own examples (3 cures,
//! 2 Phase Door, 20 missiles or 5 flasks, 1,000 turns of light), measured by
//! borg-zoo before they ship. Game data they rest on: Cure Light Wounds
//! heals 20 and costs 20 (`lib/gamedata/object.txt`, "Cure Light Wounds",
//! potion), Phase Door costs 15 (scroll), a Wooden Torch 2, a Flask of oil 3
//! and refuels 7500 turns (`object.txt`, "Flask of oil", pval:7500). A Mage's
//! Magic Missile and a Necromancer's Nether Bolt are level 1 spells
//! (`lib/gamedata/class.txt`), so they count as a ranged option.

use crate::borg_headers::borg_trait_h::{
    BI_ACCW, BI_ACLW, BI_ACSW, BI_AFUEL, BI_AMISSILES, BI_APHASE, BI_CDEPTH, BI_CLASS, BI_CLEVEL,
    BI_FOOD, BI_GOLD, BI_LIGHT, BI_MAXSP,
};
use crate::borg_io::borg_note;
use crate::headers::list_object_flags_h::OF_NO_FUEL;
use crate::headers::list_tvals_h::TV_LIGHT;
use crate::params::{value, Param};
use crate::state::BorgState;

const CLASS_MAGE: i32 = 1;
const CLASS_NECROMANCER: i32 = 4;

/// What a flask of oil refuels (`object.txt`, "Flask of oil", pval:7500).
const FLASK_TURNS: i32 = 7500;
/// A light that never runs out (an Everburning one, a Necromancer's unlight):
/// `borg_trait.rs` adds 1000 to `BI_AFUEL` for them.
const NO_FUEL: i32 = 1000;
/// Flasks of oil count as missiles of the core below this level and up to
/// this many (`borg_trait.rs`, "flasks of oil are ammo at low levels").
const OIL_AS_AMMO_LEVEL: i32 = 15;
const OIL_AS_AMMO_MAX: i32 = 15;

/// The Borg's kit state: what the notes and the restock rule remember.
#[derive(Default, Clone, Debug, PartialEq)]
pub struct KitState {
    /// The purse at the last think in town (what the Borg left town with);
    /// `None` before any.
    pub town_gold: Option<i32>,
    /// The (score permille, leash) last written in a note.
    pub noted: Option<(i32, i32)>,
    /// The last tick was in town: the first think in the dungeon records the
    /// purse again, so a purchase made after the last town think still counts
    /// (review of #844, finding 4).
    pub was_in_town: bool,
}

/// What the Borg holds, in the units of the targets.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct Facts {
    pub cures: i32,
    pub phase: i32,
    /// Missiles for the launcher, flasks of oil not counted.
    pub missiles: i32,
    pub flasks: i32,
    /// A ranged spell stands for missiles (Mage, Necromancer).
    pub spell: bool,
    /// Turns of light in the equipped light and the spares.
    pub light_turns: i32,
    pub food: i32,
}

/// What the kit should hold: the targets for the level band.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Targets {
    pub cures: i32,
    pub phase: i32,
    pub missiles: i32,
    pub flasks: i32,
    pub light_turns: i32,
    pub food: i32,
}

/// The weights of the five shares.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Weights {
    pub cures: i32,
    pub phase: i32,
    pub ranged: i32,
    pub light: i32,
    pub food: i32,
}

/// The shares met, in permille, in the order cures, Phase Door, ranged,
/// light, food.
pub type Shares = [i32; 5];

fn share(have: i32, target: i32) -> i32 {
    if target <= 0 {
        return 1000;
    }
    (have.max(0).min(target) as i64 * 1000 / target as i64) as i32
}

/// The shares a kit meets of its targets (pure).
pub fn shares(f: &Facts, t: &Targets) -> Shares {
    let ranged = if f.spell {
        1000
    } else {
        share(f.missiles, t.missiles).max(share(f.flasks, t.flasks))
    };
    [
        share(f.cures, t.cures),
        share(f.phase, t.phase),
        ranged,
        share(f.light_turns, t.light_turns),
        share(f.food, t.food),
    ]
}

/// The weighted mean of the shares, permille (pure). All weights 0: full.
pub fn score_permille(s: &Shares, w: &Weights) -> i32 {
    let w = [w.cures, w.phase, w.ranged, w.light, w.food];
    let total: i64 = w.iter().map(|&x| i64::from(x.max(0))).sum();
    if total == 0 {
        return 1000;
    }
    let sum: i64 = s.iter().zip(w).map(|(&s, w)| i64::from(s) * i64::from(w.max(0))).sum();
    (sum / total) as i32
}

/// `score^power` in permille, integers only.
pub fn pow_permille(score: i32, power: i32) -> i32 {
    let mut r: i64 = 1000;
    for _ in 0..power.max(1) {
        r = r * i64::from(score.clamp(0, 1000)) / 1000;
    }
    r as i32
}

/// The leash that follows the score (pure): `min + (reference - min) x
/// score^power`, never above `reference`; a full kit keeps `reference`.
pub fn scaled_leash(reference: i32, min: i32, score: i32, power: i32) -> i32 {
    if score >= 1000 || reference <= min {
        return reference;
    }
    min + ((i64::from(reference - min) * i64::from(pow_permille(score, power)) + 500) / 1000) as i32
}

// ---- the parameters --------------------------------------------------------

fn flag(p: Param) -> bool {
    value(p) != 0
}

/// Targets for a character level (two bands, split at `kit_band_split_level`).
pub fn targets(clevel: i32) -> Targets {
    let high = clevel > value(Param::kit_band_split_level);
    Targets {
        cures: value(if high { Param::kit_cure_target_high } else { Param::kit_cure_target }),
        phase: value(if high { Param::kit_phase_target_high } else { Param::kit_phase_target }),
        missiles: value(Param::kit_missile_target),
        flasks: value(Param::kit_flask_target),
        light_turns: value(Param::kit_light_turns_target),
        food: value(Param::kit_food_target),
    }
}

pub fn weights() -> Weights {
    Weights {
        cures: value(Param::kit_weight_cure),
        phase: value(Param::kit_weight_phase),
        ranged: value(Param::kit_weight_ranged),
        light: value(Param::kit_weight_light),
        food: value(Param::kit_weight_food),
    }
}

/// Is any part of the kit mode switched on?
fn any_switch() -> bool {
    flag(Param::kit_leash)
        || value(Param::kit_descent_min_percent) > 0
        || value(Param::kit_leave_danger_percent) > 0
        || flag(Param::kit_restock)
        || flag(Param::kit_money_first)
}

// ---- reading the Borg ------------------------------------------------------

/// The turns of light the Borg carries: the equipped light and the spares
/// (torches by their own turns, flasks of oil when a lantern is wielded).
fn light_turns(st: &BorgState) -> i32 {
    let necro = st.borg.tr(BI_CLASS) == CLASS_NECROMANCER;
    if st.borg.tr(BI_AFUEL) >= NO_FUEL {
        // Unlight, or a light that needs no fuel (still needs a light).
        return if st.borg.tr(BI_LIGHT) > 0 || necro { NO_FUEL * 10 } else { 0 };
    }
    if st.borg.tr(BI_LIGHT) == 0 {
        return 0;
    }
    let wield = crate::borg_item::inven_wield() as usize;
    let Some(light) = st.items.items.get(wield + 5) else { return 0 };
    if light.iqty == 0 || u32::from(light.tval) != TV_LIGHT {
        return 0;
    }
    if crate::engine::flag_has(&light.flags, OF_NO_FUEL as i32) {
        return NO_FUEL * 10;
    }
    let lantern = i32::from(light.sval) == st.item_val.sv_light_lantern;
    let mut turns = i32::from(light.timeout).max(0);
    for item in st.items.items.iter().take(wield) {
        // A cursed torch gives no light (the core's `light_stack_turns` skips it too).
        if item.iqty == 0 || item.cursed {
            continue;
        }
        // Each torch of a stack has its own fuel; only the wielded light burns
        // (`player-util.c`, player_update_light), so the spares keep theirs: the
        // core's own convention (`borg_light.rs`, `light_stack_turns`).
        if u32::from(item.tval) == TV_LIGHT && i32::from(item.sval) == st.item_val.sv_light_torch && !lantern {
            turns += i32::from(item.timeout).max(0) * i32::from(item.iqty);
        }
    }
    if lantern {
        turns += flasks_held(st) * FLASK_TURNS;
    }
    turns
}

fn flasks_held(st: &BorgState) -> i32 {
    st.borg.has.get(st.item_val.kv_flask_oil as usize).copied().unwrap_or(0).max(0)
}

/// What the Borg holds now.
pub fn facts(st: &BorgState) -> Facts {
    let flasks = flasks_held(st);
    let oil_in_ammo = if st.borg.tr(BI_CLEVEL) < OIL_AS_AMMO_LEVEL { flasks.min(OIL_AS_AMMO_MAX) } else { 0 };
    let class = st.borg.tr(BI_CLASS);
    Facts {
        cures: st.borg.tr(BI_ACLW) + st.borg.tr(BI_ACSW) + st.borg.tr(BI_ACCW),
        phase: st.borg.tr(BI_APHASE),
        missiles: (st.borg.tr(BI_AMISSILES) - oil_in_ammo).max(0),
        flasks,
        spell: matches!(class, CLASS_MAGE | CLASS_NECROMANCER) && st.borg.tr(BI_MAXSP) > 0,
        light_turns: light_turns(st),
        food: st.borg.tr(BI_FOOD),
    }
}

/// The score now, permille, and the shares behind it.
pub fn score(st: &BorgState) -> (i32, Shares) {
    let f = facts(st);
    let s = shares(&f, &targets(st.borg.tr(BI_CLEVEL)));
    (score_permille(&s, &weights()), s)
}

/// Does the kit mode apply to this Borg now: a switch on, in the dungeon,
/// and not above `kit_max_level`?
fn applies(st: &BorgState) -> bool {
    any_switch() && st.borg.tr(BI_CDEPTH) > 0 && st.borg.tr(BI_CLEVEL) <= value(Param::kit_max_level)
}

/// Is the kit short (score below 1) while the mode applies?
pub fn short(st: &BorgState) -> bool {
    applies(st) && score(st).0 < 1000
}

// ---- the leash -------------------------------------------------------------

/// Is the leash following the kit now (so C's leash applies at any depth and
/// level while the kit is short)?
pub fn leash_active(st: &BorgState) -> bool {
    flag(Param::kit_leash) && short(st)
}

/// Is the money-first hold on (remedy c): short of kit, a low level, and too
/// poor to fix it?
pub fn money_first_hold(st: &BorgState) -> bool {
    flag(Param::kit_money_first)
        && st.borg.tr(BI_CDEPTH) > 0
        && st.borg.tr(BI_CLEVEL) <= value(Param::kit_money_first_max_level)
        && score(st).0 < 1000
        && !affordable(st)
}

/// The leash with the kit applied, given the leash the C rules give
/// (`base`, with C's boredom growth `growth` included) and C's formula for
/// the level (`c_formula`, the leash where C has none but the kit is
/// short). The growth of a bored Borg is added after the scaling, as C adds
/// it to its own leash.
pub fn leash(st: &BorgState, base: i32, c_formula: i32, growth: i32) -> i32 {
    if !leash_active(st) {
        return base;
    }
    let (score, _) = score(st);
    let reference = (base - growth).min(c_formula);
    let mut leash = scaled_leash(reference, value(Param::kit_leash_min), score, value(Param::kit_leash_power));
    if money_first_hold(st) {
        // It has to be able to reach the gold near the stairs.
        leash = leash.max(value(Param::kit_money_leash).min(reference));
    }
    leash + growth
}

// ---- the descent ------------------------------------------------------------

/// A descent to `depth` is refused: the score (percent) and what it must
/// reach. Only a descent from the dungeon and below DL1 (DL1 is the way out
/// of a poor Borg's town), by the score switch, or by the money-first hold.
pub fn descent_blocked(st: &BorgState, depth: i32) -> Option<(&'static str, i32, i32)> {
    if !applies(st) {
        return None;
    }
    let (score, _) = score(st);
    descent_decision(st.borg.tr(BI_CDEPTH), depth, score, value(Param::kit_descent_min_percent), money_first_hold(st))
}

/// `descent_blocked` with the facts given (pure).
pub fn descent_decision(here: i32, depth: i32, score: i32, min_percent: i32, money_hold: bool) -> Option<(&'static str, i32, i32)> {
    if depth <= here || depth < 2 || here == 0 {
        return None;
    }
    if min_percent > 0 && score < min_percent * 10 {
        return Some(("kit_descent_min_percent", score / 10, min_percent));
    }
    if money_hold {
        return Some(("kit_money_first", score / 10, 100));
    }
    None
}

// ---- the danger rule ---------------------------------------------------------

/// The share of the current HP at which a danger makes a short-kit Borg
/// leave by the stairs (None: the rule is off or the kit is complete).
pub fn leave_danger_percent(st: &BorgState) -> Option<i32> {
    let pct = value(Param::kit_leave_danger_percent);
    (pct > 0 && short(st)).then_some(pct)
}

// ---- restock ------------------------------------------------------------------

/// Gold gained since leaving town reaches `kit_restock_min_gold`: a trip
/// could buy something. Leaving town with the kit still short (a store out
/// of stock) must not bring the Borg back at once.
pub fn affordable(st: &BorgState) -> bool {
    let left = st.kit.town_gold.unwrap_or(0);
    st.borg.tr(BI_GOLD) >= left + value(Param::kit_restock_min_gold)
}

/// `borg_restock`'s kit rule (remedy a, and c's trip once it can pay):
/// a reason to return to town, in the dungeon only.
pub fn restock(st: &BorgState) -> Option<(&'static str, i32, i32)> {
    let a = flag(Param::kit_restock);
    let c = flag(Param::kit_money_first) && st.borg.tr(BI_CLEVEL) <= value(Param::kit_money_first_max_level);
    if !(a || c) || !applies(st) {
        return None;
    }
    let (score, _) = score(st);
    restock_decision(
        a,
        score,
        facts(st).light_turns,
        st.borg.tr(BI_LIGHT) > 0,
        affordable(st),
        value(Param::kit_restock_below_percent),
        value(Param::kit_restock_light_turns),
    )
}

/// `restock` with the facts given (pure). `a` is the restock switch (the
/// light-turns check belongs to it); `c` alone (money first) trips on the
/// score only.
pub fn restock_decision(
    a: bool,
    score: i32,
    light_turns: i32,
    lit: bool,
    can_pay: bool,
    below_percent: i32,
    light_required: i32,
) -> Option<(&'static str, i32, i32)> {
    if !can_pay {
        return None;
    }
    // The light-turns check: a light about to go out with no spare, whatever
    // the rest of the kit holds.
    if a && light_required > 0 && lit && light_turns < light_required {
        return Some(("restock light turns", light_turns, light_required));
    }
    if score < below_percent * 10 {
        return Some(("restock kit", score / 10, below_percent));
    }
    None
}

// ---- the start reserve --------------------------------------------------------

/// Remedy b: Phase Door scrolls the town planner keeps as a requirement.
pub fn start_phase_reserve() -> i32 {
    if flag(Param::kit_start_reserve) { value(Param::kit_start_phase_reserve) } else { 0 }
}

/// Remedy b: hit points of cures the town planner keeps as a requirement.
pub fn start_cure_reserve_hp() -> i32 {
    if flag(Param::kit_start_reserve) {
        value(Param::kit_start_cure_reserve) * value(Param::kit_start_cure_hp)
    } else {
        0
    }
}

// ---- bookkeeping and notes -----------------------------------------------------

/// Once per think: remember the purse in town, and note a change of the kit
/// score or leash (one line when either moves).
pub fn tick(st: &mut BorgState) {
    if !any_switch() {
        return;
    }
    if st.borg.tr(BI_CDEPTH) == 0 {
        st.kit.town_gold = Some(st.borg.tr(BI_GOLD));
        st.kit.noted = None;
        st.kit.was_in_town = true;
        return;
    }
    if st.kit.was_in_town {
        // The first think below: the purse the Borg left town with.
        st.kit.was_in_town = false;
        st.kit.town_gold = Some(st.borg.tr(BI_GOLD));
    }
    if !applies(st) {
        return;
    }
    let (score, s) = score(st);
    let f = facts(st);
    let t = targets(st.borg.tr(BI_CLEVEL));
    let leash_now = if flag(Param::kit_leash) {
        leash(st, 250, st.borg.tr(BI_CLEVEL) * value(Param::flow_leash_per_level) + value(Param::flow_leash_base), 0)
    } else {
        -1
    };
    let key = (score, leash_now);
    if st.kit.noted == Some(key) {
        return;
    }
    st.kit.noted = Some(key);
    if score >= 1000 {
        borg_note(st, "# Kit complete: leash as C's");
        return;
    }
    let mut short_of = Vec::new();
    for (name, sh) in ["cures", "Phase", "ranged", "light", "food"].iter().zip(s) {
        if sh < 1000 {
            short_of.push(*name);
        }
    }
    borg_note(
        st,
        &format!(
            "# Kit {}.{:02} ({}/{} cures, {}/{} Phase, {} missiles+{} oil, {} light turns, {} food; short: {}): leash {}",
            score / 1000,
            score % 1000 / 10,
            f.cures,
            t.cures,
            f.phase,
            t.phase,
            f.missiles,
            f.flasks,
            f.light_turns,
            f.food,
            short_of.join(", "),
            if leash_now >= 0 { leash_now.to_string() } else { "unchanged".to_string() },
        ),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    const T: Targets = Targets { cures: 3, phase: 2, missiles: 20, flasks: 5, light_turns: 1000, food: 3 };
    const W: Weights = Weights { cures: 1, phase: 1, ranged: 1, light: 1, food: 1 };

    fn full() -> Facts {
        Facts { cures: 3, phase: 2, missiles: 20, flasks: 0, spell: false, light_turns: 5000, food: 5 }
    }

    #[test]
    fn a_full_kit_scores_one_and_keeps_the_leash() {
        let s = shares(&full(), &T);
        assert_eq!(score_permille(&s, &W), 1000);
        assert_eq!(scaled_leash(12, 1, 1000, 2), 12);
    }

    #[test]
    fn an_empty_kit_gets_the_minimum_leash() {
        let f = Facts { food: 0, ..Facts::default() };
        let s = shares(&f, &T);
        assert_eq!(score_permille(&s, &W), 0);
        assert_eq!(scaled_leash(12, 1, 0, 1), 1);
        assert_eq!(scaled_leash(12, 0, 0, 1), 0);
    }

    #[test]
    fn the_leash_follows_the_score_and_the_power() {
        // C's leash at CL1 is 12; a half kit, linear: 1 + 11 x 0.5 = 6.5 -> 7.
        assert_eq!(scaled_leash(12, 1, 500, 1), 7);
        // Squared: 1 + 11 x 0.25 = 3.75 -> 4.
        assert_eq!(scaled_leash(12, 1, 500, 2), 4);
        // The issue's Ranger, no cures, no Phase Door, missiles and light:
        // 3 of 5 shares... with all five weighted equally 0.6 -> 0.36.
        assert_eq!(scaled_leash(12, 1, 600, 2), 5);
        // Never above the reference, never raised from below the minimum.
        assert_eq!(scaled_leash(3, 5, 100, 1), 3);
    }

    #[test]
    fn the_issue_ranger_with_no_cures_and_no_phase_scores_the_illustration() {
        // Fang's Ranger: 56 arrows, a torch, no cures, no Phase Door. The
        // issue's illustration (four items, food not counted) says 0.50.
        let f = Facts { cures: 0, phase: 0, missiles: 56, flasks: 0, spell: false, light_turns: 5000, food: 5 };
        let s = shares(&f, &T);
        assert_eq!(s, [0, 0, 1000, 1000, 1000]);
        let four = Weights { food: 0, ..W };
        assert_eq!(score_permille(&s, &four), 500);
        assert_eq!(score_permille(&s, &W), 600);
    }

    #[test]
    fn flasks_or_missiles_make_the_ranged_share_and_a_spell_counts() {
        let none = Facts { missiles: 0, flasks: 0, ..full() };
        assert_eq!(shares(&none, &T)[2], 0);
        assert_eq!(shares(&Facts { flasks: 5, ..none }, &T)[2], 1000);
        assert_eq!(shares(&Facts { flasks: 2, ..none }, &T)[2], 400);
        assert_eq!(shares(&Facts { missiles: 10, ..none }, &T)[2], 500);
        assert_eq!(shares(&Facts { spell: true, ..none }, &T)[2], 1000);
    }

    #[test]
    fn the_light_share_is_turns_over_the_target() {
        let dim = Facts { light_turns: 250, ..full() };
        assert_eq!(shares(&dim, &T)[3], 250);
        assert_eq!(shares(&Facts { light_turns: 0, ..full() }, &T)[3], 0);
    }

    #[test]
    fn weights_move_the_score_and_zero_weights_leave_the_rest() {
        let f = Facts { cures: 0, phase: 2, missiles: 20, flasks: 0, spell: false, light_turns: 5000, food: 5 };
        let s = shares(&f, &T);
        assert_eq!(score_permille(&s, &Weights { cures: 3, phase: 1, ranged: 1, light: 1, food: 1 }), 571);
        assert_eq!(score_permille(&s, &Weights { cures: 0, ..W }), 1000);
        assert_eq!(score_permille(&s, &Weights { cures: 0, phase: 0, ranged: 0, light: 0, food: 0 }), 1000);
    }

    #[test]
    fn power_is_integer_and_exact() {
        assert_eq!(pow_permille(500, 1), 500);
        assert_eq!(pow_permille(500, 2), 250);
        assert_eq!(pow_permille(500, 3), 125);
        assert_eq!(pow_permille(1000, 4), 1000);
        assert_eq!(pow_permille(0, 2), 0);
    }

    #[test]
    fn the_descent_floor_spares_the_way_out_of_town_and_dl1() {
        // From town (1 is the way out), from DL1 to DL2 on a score below the floor, not upward or level.
        assert_eq!(descent_decision(0, 1, 0, 50, false), None);
        assert_eq!(descent_decision(1, 2, 400, 50, false), Some(("kit_descent_min_percent", 40, 50)));
        assert_eq!(descent_decision(1, 2, 500, 50, false), None, "at the floor");
        assert_eq!(descent_decision(2, 2, 0, 50, false), None, "staying is not a descent");
        assert_eq!(descent_decision(3, 2, 0, 50, false), None, "going up is not a descent");
        assert_eq!(descent_decision(1, 2, 0, 0, false), None, "switch off");
        // The probes deeper than the next level are refused too; one at or above the Borg's own is not.
        assert_eq!(descent_decision(10, 15, 0, 50, false), Some(("kit_descent_min_percent", 0, 50)));
        assert_eq!(descent_decision(100, 99, 0, 50, false), None, "the probe of 99 from DL100");
        assert_eq!(descent_decision(1, 2, 900, 0, true), Some(("kit_money_first", 90, 100)), "money first holds it on DL1");
    }

    #[test]
    fn restock_needs_the_purse_and_a_short_kit() {
        // Short kit and a purse: go. No purse: stay (nothing could be bought).
        assert_eq!(restock_decision(true, 500, 5000, true, true, 75, 0), Some(("restock kit", 50, 75)));
        assert_eq!(restock_decision(true, 500, 5000, true, false, 75, 0), None);
        // A kit at or above the line stays.
        assert_eq!(restock_decision(true, 750, 5000, true, true, 75, 0), None);
        // The light-turns check, with a switch of its own value; 0 = off.
        assert_eq!(restock_decision(true, 900, 300, true, true, 75, 500), Some(("restock light turns", 300, 500)));
        assert_eq!(restock_decision(true, 900, 300, true, true, 75, 0), None);
        assert_eq!(restock_decision(true, 900, 300, false, true, 75, 500), None, "no light at all is C's rule's");
        // Money first alone trips on the score, not on the light.
        assert_eq!(restock_decision(false, 900, 300, true, true, 75, 500), None);
        assert_eq!(restock_decision(false, 300, 5000, true, true, 75, 500), Some(("restock kit", 30, 75)));
    }
}
