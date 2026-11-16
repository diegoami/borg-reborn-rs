//! Phase 1 of the town inventory redesign (borg-rs#36): the usefulness
//! function, the swept judgement of the plan.
//!
//! One town visit produces one plan: what to keep, store, sell and drop,
//! and what to buy, under one gold budget. Every candidate purchase is a
//! (need, reduction) pair priced by [`usefulness`]: the deficit of a
//! [`Need`] in the game's own units, weighted and discounted by the
//! flat-then-capped value curve decided on 2026-11-12 (B1):
//!
//! ```text
//! U_K(n) = w_K * min(1, n*r / g),    g = target - carried
//! ```
//!
//! The marginal value of a copy is the constant `w*r/g` until the deficit
//! closes, and zero at or above the target: a met need prices nothing.
//! (In integers the floor division lets a marginal exceed its predecessor
//! by at most one — rounding noise, not shape.) The curve is weakly
//! concave, so the group knapsack DP of phase 3 is exact (P15).
//!
//! Nothing calls this module yet: the plan itself is phase 4, and the
//! reduction table that maps a ware to its (need, reduction) pairs lands
//! with the catalogue of useful kinds (#35 item 4; planned for phase 2,
//! moved 2026-11-12 because the catalogue is still open). Every public
//! item therefore carries
//! `#[cfg_attr(not(test), expect(dead_code))]`: a marker that removes
//! itself when the first production caller lands.

use crate::borg_headers::borg_h::{
    BORG_TOWN_FOOD_TRIP, BORG_TOWN_LIGHT_DEPTH, BORG_TOWN_SHARE, BORG_TOWN_W_BLEEDING,
    BORG_TOWN_W_ESCAPE, BORG_TOWN_W_FOOD, BORG_TOWN_W_KNOWLEDGE, BORG_TOWN_W_LIGHT,
    BORG_TOWN_W_POISON, BORG_TOWN_W_POWER, BORG_TOWN_W_TRIP,
};
use crate::borg_headers::borg_trait_h::{
    BI_ACCW, BI_ACLW, BI_ACSW, BI_ACUREPOIS, BI_APHASE, BI_FOOD, BI_ISCUT, BI_ISHUNGRY,
    BI_ISPOISONED, BI_ISWEAK, BI_LIGHT, BI_MAXDEPTH, BI_RECALL,
};
use crate::headers::list_tvals_h::TV_LIGHT;
use crate::state::BorgState;

/// The eight needs a purchase can serve, as the epic log decided them on
/// 2026-11-12. There are **no floors** (owner decision, 2026-11-14): every
/// object can become redundant - another way to eat, to see or to cure
/// may exist - so none is a constraint. What replaces them is the
/// **urgent** need ([`urgent_needs`]): needed now, with no other way to
/// meet it, and then bought before anything else.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Need {
    /// Light sources carried; the descent target decides how many.
    Light,
    /// Food rations carried.
    Food,
    /// Healing for the coming trip: a stock of wound potions (Cure Light,
    /// Serious and Critical Wounds, and the cure devices), which restore
    /// hit points at once and cost no mana - a healing spell does not
    /// replace them (owner, 2026-11-14). Urgent when cut with no cure at
    /// all. Was "Bleeding" (a cure for a cut only), which priced the
    /// potions at nothing while the character was not cut. Weighted by the
    /// `borg_town_w_bleeding` setting.
    Healing,
    /// A cure for the poison the character suffers now, when it has none.
    /// Poison wears off, so a cure carried "just in case" is no need
    /// (2026-11-14).
    Poison,
    /// Phase Door scrolls, the per-level escape.
    Escape,
    /// Learning an unknown kind: a shop ware whose kind the character does
    /// not know is worth buying once - the purchase teaches the kind, so
    /// every copy found in the dungeon is recognised (owner, 2026-11-14,
    /// "buy to identify"). Each unknown kind is worth the whole weight on
    /// its own; a second copy teaches nothing. Priced per ware in the
    /// plan's step, so the deficit here reads 0.
    Knowledge,
    /// Borg power against the trip's depth target.
    Power,
    /// Getting home: Word of Recall, `RECALL_PER_TRIP` of them - one to come
    /// back, one spare (owner, 2026-11-14). Read from `BI_RECALL`, which
    /// counts the scrolls, a usable rod, the spell and recall gear.
    Trip,
}

impl Need {
    /// All needs, in enum order.
    pub const ALL: [Need; 8] = [
        Need::Light,
        Need::Food,
        Need::Healing,
        Need::Poison,
        Need::Escape,
        Need::Knowledge,
        Need::Power,
        Need::Trip,
    ];
}

/// The wound cures the character has: Cure Light, Serious and Critical
/// Wounds (potions, and the devices `borg_trait` adds 1000 for), and a
/// curing spell.
pub fn wound_cures(st: &BorgState, trip: &Trip) -> i32 {
    st.borg.tr(BI_ACLW) + st.borg.tr(BI_ACSW) + st.borg.tr(BI_ACCW) + 1000 * i32::from(trip.cure_spell)
}

/// The poison cures the character has: Neutralize Poison and the
/// poison-curing foods (`BI_ACUREPOIS`), Cure Critical Wounds, and a
/// curing spell (2026-11-14).
pub fn poison_cures(st: &BorgState, trip: &Trip) -> i32 {
    st.borg.tr(BI_ACUREPOIS) + st.borg.tr(BI_ACCW) + 1000 * i32::from(trip.cure_spell)
}

/// The urgent needs: needed **now**, with **no other way** to meet them,
/// in enum order (owner decision, 2026-11-14). They replace the floors:
/// the plan buys them before anything else while they are urgent, skips
/// one it cannot afford, and never blocks the visit for one.
///
/// * **light** - no light source at all: none carried and none worn
///   (`BI_LIGHT < 1` also catches a glowing artifact, which is a light);
/// * **food** - hungry or weak, and nothing to eat: `BI_FOOD` already
///   counts the rations, the Remove Hunger scrolls and the hunger spells;
/// * **bleeding** - cut, and no wound cure ([`wound_cures`]);
/// * **poison** - poisoned, and no poison cure ([`poison_cures`]).
pub fn urgent_needs(st: &BorgState, trip: &Trip) -> Vec<Need> {
    let mut out = Vec::new();
    if st.borg.tr(BI_LIGHT) < 1 && carried_lights(st) < 1 {
        out.push(Need::Light);
    }
    if (st.borg.tr(BI_ISHUNGRY) != 0 || st.borg.tr(BI_ISWEAK) != 0) && st.borg.tr(BI_FOOD) < 1 {
        out.push(Need::Food);
    }
    if st.borg.tr(BI_ISCUT) != 0 && wound_cures(st, trip) < 1 {
        out.push(Need::Healing);
    }
    if st.borg.tr(BI_ISPOISONED) != 0 && poison_cures(st, trip) < 1 {
        out.push(Need::Poison);
    }
    out
}

/// The trip model, stated once (decided 2026-11-12): the descent target
/// is five levels past the deepest the borg has been, each level is
/// expected to take a thousand turns (anchored on the 2000-turn give-up),
/// and the power gap is a trip's levels of preparation in `borg_power`'s
/// units (since 2026-11-14; it was two thousand per level of depth, the weakest
/// of the provisional values, to be revisited when the zoo sweeps.
#[cfg_attr(not(test), expect(dead_code))]
#[derive(Clone, Copy, Debug)]
pub struct Trip {
    /// Where the next trip aims: `BI_MAXDEPTH + LEVELS_PER_TRIP`.
    pub depth_target: i32,
    /// Levels per trip: 5.
    pub levels_per_trip: i32,
    /// Expected turns per level: 1000.
    pub turns_per_level: i32,
    /// `borg_town_light_depth`: the descent target from which two light
    /// sources are required.
    pub light_depth: i32,
    /// `borg_town_food_trip`: the food rations carried per trip.
    pub food_trip: i32,
    /// A curing spell (Herbal Curing) is usable: it cures cuts and poison.
    /// No trait counts it, so the caller sets it (the shadow producer from
    /// the engine, the replay from the record's `spells`); `of` leaves it
    /// false.
    pub cure_spell: bool,
}

impl Trip {
    /// Levels per trip, and the depth gain the descent target assumes.
    pub const LEVELS_PER_TRIP: i32 = 5;
    /// Expected turns per level, anchored on the 2000-turn give-up.
    pub const TURNS_PER_LEVEL: i32 = 1000;
    /// The power one dungeon level of preparation is worth, in
    /// `borg_power`'s own units: it credits 40,000 per level the Borg is
    /// prepared for (`borg_power`, "a bonus for deep level prep").
    pub const POWER_PER_LEVEL: i32 = 40_000;
    /// The wound potions a trip wants (provisional, 2026-11-14).
    pub const HEALING_PER_TRIP: i32 = 5;
    /// The Word of Recall a trip wants: one to come back, one spare
    /// (2026-11-14).
    pub const RECALL_PER_TRIP: i32 = 2;

    /// The trip the current state plans for.
    pub fn of(st: &BorgState) -> Self {
        Trip {
            depth_target: st.borg.tr(BI_MAXDEPTH) + Self::LEVELS_PER_TRIP,
            levels_per_trip: Self::LEVELS_PER_TRIP,
            turns_per_level: Self::TURNS_PER_LEVEL,
            light_depth: st.cfg(BORG_TOWN_LIGHT_DEPTH),
            food_trip: st.cfg(BORG_TOWN_FOOD_TRIP),
            cure_spell: false,
        }
    }

    /// One light source, two once the descent target reaches
    /// `light_depth`.
    pub fn required_light(&self) -> i32 {
        if self.depth_target < self.light_depth {
            1
        } else {
            2
        }
    }

    /// The food rations carried per trip.
    pub fn required_food(&self) -> i32 {
        self.food_trip
    }

    /// One escape per ten levels of trip, rounded up.
    pub fn escapes_needed(&self) -> i32 {
        (self.levels_per_trip + 9) / 10
    }

    /// The wound potions a trip wants (provisional, for the zoo to sweep).
    pub fn required_healing(&self) -> i32 {
        Self::HEALING_PER_TRIP
    }

    /// The Word of Recall a trip wants: one to come back, one spare.
    pub fn required_recall(&self) -> i32 {
        Self::RECALL_PER_TRIP
    }

    /// The power gap a trip opens: a trip's levels of preparation,
    /// `POWER_PER_LEVEL * levels_per_trip` (decided 2026-11-14). It is a
    /// standing gap, not a target minus the current power: the Borg's
    /// power is 10^5-10^6 from the first levels on, so any absolute
    /// target small enough to be met early is met everywhere (the
    /// provisional `2000 x depth target` was 10,000-14,000 at DL0-2 against
    /// a power of 0.5-1.6 million, and priced nothing). A wearable worth a
    /// trip's preparation fills the need.
    pub fn power_gap(&self) -> i32 {
        Self::POWER_PER_LEVEL * self.levels_per_trip
    }
}

/// The free parameters the zoo sweeps in phase 4: the eight need weights
/// (`w_light = 100` is the numeraire, so seven are free) and the caution
/// `k`, the purse-share cap of a single purchase.
#[derive(Clone, Copy, Debug)]
pub struct Weights {
    /// `borg_town_w_light` (numeraire, 100).
    pub light: i64,
    /// `borg_town_w_food` (100).
    pub food: i64,
    /// `borg_town_w_bleeding` (100).
    pub bleeding: i64,
    /// `borg_town_w_poison` (100).
    pub poison: i64,
    /// `borg_town_w_escape` (50).
    pub escape: i64,
    /// `borg_town_w_knowledge` (25).
    pub knowledge: i64,
    /// `borg_town_w_power` (50).
    pub power: i64,
    /// `borg_town_w_trip` (50).
    pub trip: i64,
    /// `borg_town_share`: a single purchase may cost at most purse / k.
    pub caution: i64,
}

impl Weights {
    /// The weights as borg.txt gives them.
    pub fn of(cfg: &[i32]) -> Self {
        Weights {
            light: i64::from(cfg[BORG_TOWN_W_LIGHT as usize]),
            food: i64::from(cfg[BORG_TOWN_W_FOOD as usize]),
            bleeding: i64::from(cfg[BORG_TOWN_W_BLEEDING as usize]),
            poison: i64::from(cfg[BORG_TOWN_W_POISON as usize]),
            escape: i64::from(cfg[BORG_TOWN_W_ESCAPE as usize]),
            knowledge: i64::from(cfg[BORG_TOWN_W_KNOWLEDGE as usize]),
            power: i64::from(cfg[BORG_TOWN_W_POWER as usize]),
            trip: i64::from(cfg[BORG_TOWN_W_TRIP as usize]),
            caution: i64::from(cfg[BORG_TOWN_SHARE as usize]),
        }
    }

    /// The weight of a need.
    pub fn get(&self, need: Need) -> i64 {
        match need {
            Need::Light => self.light,
            Need::Food => self.food,
            Need::Healing => self.bleeding,
            Need::Poison => self.poison,
            Need::Escape => self.escape,
            Need::Knowledge => self.knowledge,
            Need::Power => self.power,
            Need::Trip => self.trip,
        }
    }

    /// The caution cap: `cost <= purse / k`, so `cost * k <= purse`.
    /// `k = 0` disables the cap.
    #[cfg_attr(not(test), expect(dead_code))]
    pub fn caution_allows(&self, cost: i64, purse: i64) -> bool {
        cost.saturating_mul(self.caution) <= purse
    }
}

/// The value curve (B1): `w * min(1, n*r / g)` in integers,
/// `w * min(n*r, g) / g`. Zero weight, reduction, gap or count prices
/// nothing; at or above the target the gap is closed and nothing is
/// priced.
pub fn value(weight: i64, reduction: i64, gap: i64, copies: i64) -> i64 {
    if weight <= 0 || reduction <= 0 || gap <= 0 || copies <= 0 {
        return 0;
    }
    let closed = reduction.saturating_mul(copies).min(gap);
    weight.saturating_mul(closed) / gap
}

/// The marginal value of copy `copy` (1-based): constant `w*r/g` while
/// the deficit is open, zero once it closes.
#[cfg_attr(not(test), expect(dead_code))]
pub fn marginal(weight: i64, reduction: i64, gap: i64, copy: i64) -> i64 {
    value(weight, reduction, gap, copy) - value(weight, reduction, gap, copy - 1)
}

/// The light sources the character carries, worn and spare: every item
/// of the light tval, by count. The light **need** counts sources (decided
/// 2026-11-13: a second light is wanted past the descent target, and a
/// spare matters because torches burn out); the light is **urgent** only
/// when there is none at all (2026-11-14, [`urgent_needs`]).
pub fn carried_lights(st: &BorgState) -> i32 {
    st.items
        .items
        .iter()
        .filter(|it| it.iqty != 0 && u32::from(it.tval) == TV_LIGHT)
        .map(|it| i32::from(it.iqty))
        .sum()
}

/// For a counted need (light, food, healing, escape, trip): what the
/// character carries of it, in the need's units, and what the trip wants.
/// `None` for the others. The keep value of a stack is the curve's value at
/// `carried` against at `carried` less its copies - so surplus copies,
/// past the target, are worth nothing.
pub fn counted(need: Need, st: &BorgState, trip: &Trip) -> Option<(i64, i64)> {
    let t = |i: u32| i64::from(st.borg.tr(i));
    match need {
        Need::Light => Some((i64::from(carried_lights(st)), i64::from(trip.required_light()))),
        Need::Food => Some((t(BI_FOOD), i64::from(trip.required_food()))),
        Need::Healing => Some((t(BI_ACLW) + t(BI_ACSW) + t(BI_ACCW), i64::from(trip.required_healing()))),
        Need::Escape => Some((t(BI_APHASE), i64::from(trip.escapes_needed()))),
        Need::Trip => Some((t(BI_RECALL), i64::from(trip.required_recall()))),
        _ => None,
    }
}

/// The deficit of a need in the game's own units: how far the carried
/// kit is below the target. Never negative; a met need has no deficit and
/// prices nothing. Healing is the trip's stock of wound potions short, at
/// least 1 while cut with no cure (2026-11-14); Trip is the Recall short of
/// `RECALL_PER_TRIP`. Poison is a boolean: 1 while poisoned (as Bleeding was while cut, or
/// poisoned) with no cure of it (2026-11-14; until then poison also
/// counted a standing capability, carried whether poisoned or not -
/// poison wears off, so that is gone). Knowledge reads 0 until the
/// catalogue (#35 item 4) prices it, Trip reads 0 until the stock's
/// reduction table prices the recall.
pub fn deficit(need: Need, st: &BorgState, trip: &Trip) -> i64 {
    let d = match need {
        Need::Light => trip.required_light() - carried_lights(st),
        Need::Food => trip.required_food() - st.borg.tr(BI_FOOD),
        // The stock for the trip, counted in potions and devices (a spell
        // is no stock: potions cost no mana); at least 1 while cut with no
        // cure at all, the spell included.
        Need::Healing => {
            let stock = trip.required_healing()
                - (st.borg.tr(BI_ACLW) + st.borg.tr(BI_ACSW) + st.borg.tr(BI_ACCW));
            stock.max(i32::from(st.borg.tr(BI_ISCUT) != 0 && wound_cures(st, trip) < 1))
        }
        Need::Poison => (st.borg.tr(BI_ISPOISONED) != 0 && poison_cures(st, trip) < 1) as i32,
        Need::Escape => trip.escapes_needed() - st.borg.tr(BI_APHASE),
        Need::Knowledge => 0,
        Need::Power => trip.power_gap(),
        Need::Trip => trip.required_recall() - st.borg.tr(BI_RECALL),
    };
    i64::from(d).max(0)
}

/// The usefulness of `copies` of a purchase that reduces `need`'s
/// deficit by `reduction` each: `value(w, reduction, deficit, copies)`.
pub fn usefulness(
    need: Need,
    reduction: i64,
    copies: i64,
    st: &BorgState,
    trip: &Trip,
    weights: &Weights,
) -> i64 {
    value(weights.get(need), reduction, deficit(need, st, trip), copies)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::borg_headers::borg_trait_h::BI_MAX;
    use crate::borg_init::BORG_SETTINGS;

    /// The borg.txt defaults as the parser reads them.
    fn cfg() -> Vec<i32> {
        BORG_SETTINGS.iter().map(|s| s.default_value).collect()
    }

    /// An empty borg with the default settings and a zeroed trait array.
    fn state() -> BorgState {
        let mut st = BorgState::default();
        st.core.cfg = cfg();
        st.borg.trait_ = vec![0; BI_MAX as usize];
        st
    }

    /// w=100, r=2, g=10: the value rises by the constant w*r/g = 20 per
    /// copy until the deficit closes at five copies, then stays at w.
    #[test]
    fn curve_is_flat_then_capped() {
        let got: Vec<i64> = (0..=6).map(|n| value(100, 2, 10, n)).collect();
        assert_eq!(got, [0, 20, 40, 60, 80, 100, 100]);
    }

    /// At or above the target the gap is closed and nothing is priced; a
    /// zero weight, reduction or count prices nothing either.
    #[test]
    fn curve_prices_nothing_at_or_above_target() {
        assert_eq!(value(100, 2, 0, 3), 0);
        assert_eq!(value(100, 2, -3, 3), 0);
        assert_eq!(value(100, 2, 10, 0), 0);
        assert_eq!(value(0, 2, 10, 3), 0);
        assert_eq!(value(100, 0, 10, 3), 0);
    }

    /// The curve is nondecreasing, capped by the weight, and weakly
    /// concave up to the integer rounding of the floor division: below
    /// the cap a marginal is `floor(w*r/g)` or one more, so a marginal
    /// can exceed its predecessor by at most one; once the deficit is
    /// closed the marginal is exactly zero and the total exactly w.
    #[test]
    fn curve_is_weakly_concave_up_to_rounding_and_capped() {
        let cases: [(i64, i64, i64); 4] = [(100, 2, 10), (50, 1, 3), (25, 7, 4), (100, 1, 1)];
        for (w, r, g) in cases {
            let mut prev = i64::MAX;
            let cap = g.div_euclid(r) + i64::from(g % r != 0); // the first n with n*r >= g
            for n in 1..=12 {
                let v = value(w, r, g, n);
                assert!(v <= w, "value above weight at n={n}");
                assert!(v >= value(w, r, g, n - 1), "value fell at n={n}");
                let m = marginal(w, r, g, n);
                if n > 1 {
                    assert!(m <= prev + 1, "marginal rose by more than rounding at n={n} for w={w} r={r} g={g}");
                }
                if n > cap {
                    assert_eq!(m, 0, "marginal past the cap at n={n}");
                }
                if n >= cap {
                    assert_eq!(v, w, "below the weight at the cap, n={n}");
                }
                prev = m;
            }
        }
    }

    /// The trip model: depth target = maxdepth + 5, a thousand turns per
    /// level, light keyed to the descent target, food the per-trip
    /// ration, one escape per ten levels, power gap = 40000 x 5 levels.
    #[test]
    fn trip_model_reads_state_and_settings() {
        let mut st = state();
        st.borg.trait_[BI_MAXDEPTH as usize] = 10;
        let trip = Trip::of(&st);
        assert_eq!(trip.depth_target, 15);
        assert_eq!(trip.levels_per_trip, 5);
        assert_eq!(trip.turns_per_level, 1000);
        assert_eq!(trip.light_depth, 15);
        assert_eq!(trip.food_trip, 10);
        // at the boundary the second light is already required
        assert_eq!(trip.required_light(), 2);
        assert_eq!(trip.required_food(), 10);
        assert_eq!(trip.escapes_needed(), 1);
        assert_eq!(trip.power_gap(), 200_000);
        // a shallower target needs one light
        st.borg.trait_[BI_MAXDEPTH as usize] = 4;
        assert_eq!(Trip::of(&st).required_light(), 1);
    }

    #[test]
    fn weights_read_the_settings() {
        let w = Weights::of(&cfg());
        assert_eq!(Need::ALL.len(), 8);
        assert_eq!(w.get(Need::Light), 100);
        assert_eq!(w.get(Need::Food), 100);
        assert_eq!(w.get(Need::Healing), 100);
        assert_eq!(w.get(Need::Poison), 100);
        assert_eq!(w.get(Need::Escape), 50);
        assert_eq!(w.get(Need::Knowledge), 25);
        assert_eq!(w.get(Need::Power), 50);
        assert_eq!(w.get(Need::Trip), 50);
        assert_eq!(w.caution, 10);
    }

    /// k=10: a purchase may cost at most a tenth of the purse; k=0
    /// disables the cap.
    #[test]
    fn caution_is_a_share_cap() {
        let w = Weights::of(&cfg());
        assert!(w.caution_allows(600, 6000));
        assert!(!w.caution_allows(601, 6000));
        let mut uncapped = w;
        uncapped.caution = 0;
        assert!(uncapped.caution_allows(999_999, 1));
    }

    /// The urgent needs (2026-11-14): needed now, and no other way.
    #[test]
    fn urgent_means_needed_now_with_no_other_way() {
        let urgent = |st: &BorgState, spell: bool| {
            let mut trip = Trip::of(st);
            trip.cure_spell = spell;
            urgent_needs(st, &trip)
        };
        // the empty kit, fed and well: only the missing light is urgent -
        // no floors, so no food or poison cure is wanted "just in case"
        let mut st = state();
        assert_eq!(urgent(&st, false), [Need::Light]);
        // a glowing artifact is a light; so is a torch carried unworn
        st.borg.trait_[BI_LIGHT as usize] = 1;
        assert_eq!(urgent(&st, false), []);
        st.borg.trait_[BI_LIGHT as usize] = 0;
        let mut torch = crate::borg_item::blank_item();
        torch.tval = TV_LIGHT as u8;
        torch.iqty = 1;
        st.items.items = vec![torch];
        assert_eq!(urgent(&st, false), []);
        // hungry with nothing to eat is urgent; a Remove Hunger scroll or
        // a hunger spell is food (BI_FOOD counts them), so it is not
        st.borg.trait_[BI_ISHUNGRY as usize] = 1;
        assert_eq!(urgent(&st, false), [Need::Food]);
        st.borg.trait_[BI_FOOD as usize] = 1000;
        assert_eq!(urgent(&st, false), []);
        st.borg.trait_[BI_ISHUNGRY as usize] = 0;
        st.borg.trait_[BI_ISWEAK as usize] = 1;
        st.borg.trait_[BI_FOOD as usize] = 0;
        assert_eq!(urgent(&st, false), [Need::Food]);
        st.borg.trait_[BI_ISWEAK as usize] = 0;
        // cut with no cure: urgent; any wound cure, or the spell, is a way
        st.borg.trait_[BI_ISCUT as usize] = 1;
        assert_eq!(urgent(&st, false), [Need::Healing]);
        assert_eq!(urgent(&st, true), []);
        st.borg.trait_[BI_ACSW as usize] = 1;
        assert_eq!(urgent(&st, false), []);
        st.borg.trait_[BI_ISCUT as usize] = 0;
        st.borg.trait_[BI_ACSW as usize] = 0;
        // poisoned with no cure: urgent; Neutralize Poison, CCW or the
        // spell is a way
        st.borg.trait_[BI_ISPOISONED as usize] = 1;
        assert_eq!(urgent(&st, false), [Need::Poison]);
        assert_eq!(urgent(&st, true), []);
        st.borg.trait_[BI_ACCW as usize] = 1;
        assert_eq!(urgent(&st, false), []);
        st.borg.trait_[BI_ACCW as usize] = 0;
        st.borg.trait_[BI_ACUREPOIS as usize] = 1;
        assert_eq!(urgent(&st, false), []);
    }

    /// maxdepth 10, so the depth target is 15: one light source carried
    /// of the two wanted (gap 1, worth 100 - the need counts sources since
    /// 2026-11-13, not the radius), six more rations (gap 6: 16 for the first, 100 at six
    /// and no more after), escapes already met (gap 0), and the power gap
    /// of a trip, 200000: a +40000 item (one level of preparation) is worth
    /// 50*40000/200000 = 10, and one worth the whole trip the full 50.
    #[test]
    fn usefulness_values_for_fixed_states() {
        let mut st = state();
        st.borg.trait_[BI_MAXDEPTH as usize] = 10;
        st.borg.trait_[BI_LIGHT as usize] = 2; // a torch's radius: not the count
        let mut torch = crate::borg_item::blank_item();
        torch.tval = TV_LIGHT as u8;
        torch.iqty = 1;
        st.items.items = vec![torch];
        assert_eq!(carried_lights(&st), 1);
        st.borg.trait_[BI_FOOD as usize] = 4;
        st.borg.trait_[BI_APHASE as usize] = 1;
        st.borg.power = 20000;
        let trip = Trip::of(&st);
        let w = Weights::of(&cfg());
        assert_eq!(usefulness(Need::Light, 1, 1, &st, &trip, &w), 100);
        assert_eq!(usefulness(Need::Food, 1, 1, &st, &trip, &w), 16);
        assert_eq!(usefulness(Need::Food, 1, 6, &st, &trip, &w), 100);
        assert_eq!(usefulness(Need::Food, 1, 7, &st, &trip, &w), 100);
        assert_eq!(usefulness(Need::Escape, 1, 1, &st, &trip, &w), 0);
        assert_eq!(usefulness(Need::Power, 40_000, 1, &st, &trip, &w), 10);
        assert_eq!(usefulness(Need::Power, 250_000, 1, &st, &trip, &w), 50);
        // Knowledge is priced per ware in the step, so its deficit reads 0;
        // the trip wants two Recall and carries none: one is worth 50*1/2
        assert_eq!(usefulness(Need::Knowledge, 1, 1, &st, &trip, &w), 0);
        assert_eq!(usefulness(Need::Trip, 1, 1, &st, &trip, &w), 25);
        assert_eq!(usefulness(Need::Trip, 1, 3, &st, &trip, &w), 50);
    }

    /// Healing is a trip's stock of wound potions (5), urgent while cut
    /// with no cure; poison is a condition only, with no standing need
    /// (2026-11-14).
    #[test]
    fn usefulness_values_for_condition_states() {
        let mut st = state();
        let mut trip = Trip::of(&st);
        let w = Weights::of(&cfg());
        // no potions, not cut: the stock is 5 short, one potion is 100/5
        assert_eq!(usefulness(Need::Healing, 1, 1, &st, &trip, &w), 20);
        assert_eq!(usefulness(Need::Healing, 1, 5, &st, &trip, &w), 100);
        // four carried: one more fills the stock, a sixth adds nothing
        st.borg.trait_[BI_ACLW as usize] = 4;
        assert_eq!(usefulness(Need::Healing, 1, 1, &st, &trip, &w), 100);
        st.borg.trait_[BI_ACLW as usize] = 5;
        assert_eq!(usefulness(Need::Healing, 1, 1, &st, &trip, &w), 0);
        // a healing spell is no stock: potions cost no mana
        st.borg.trait_[BI_ACLW as usize] = 0;
        trip.cure_spell = true;
        assert_eq!(usefulness(Need::Healing, 1, 1, &st, &trip, &w), 20);
        // cut with no cure at all: still at least the urgent 1 - with the
        // spell, the cut is curable, and the stock alone counts
        trip.cure_spell = false;
        st.borg.trait_[BI_ACLW as usize] = 5;
        st.borg.trait_[BI_ISCUT as usize] = 0;
        assert_eq!(deficit(Need::Healing, &st, &trip), 0);
        // not poisoned: no poison need at all, cure or no cure (poison
        // wears off; the standing capability is gone, 2026-11-14)
        assert_eq!(usefulness(Need::Poison, 1, 1, &st, &trip, &w), 0);
        // poisoned with no cure: gap 1
        st.borg.trait_[BI_ISPOISONED as usize] = 1;
        assert_eq!(usefulness(Need::Poison, 1, 1, &st, &trip, &w), 100);
        // a CCW carried is a cure
        st.borg.trait_[BI_ACCW as usize] = 1;
        assert_eq!(usefulness(Need::Poison, 1, 1, &st, &trip, &w), 0);
    }
}
