//! Trip-relative, marginal final-kit utility (shadow model v5).
//!
//! Exposure scales demand, rather than imposing an item quota. The prior
//! flat/capped v4 model remains in borg_town_usefulness for historical replay.
//! The forecasting rates below are explicit policy estimates, not measured
//! probabilities. Destination and duration can be declared independently.

use std::collections::{BTreeMap, BTreeSet};
use crate::borg_headers::borg_h::{BORG_TOWN_TARGET_DEPTH, BORG_TOWN_TRIP_TURNS, BORG_TOWN_W_GOLD,
    BORG_TOWN_DEPTH_GAIN, BORG_TOWN_EXPLORE_TURNS, BORG_TOWN_STAIR_TURNS, BORG_TOWN_FOOD_TURNS,
    BORG_TOWN_LIGHT_TURNS, BORG_TOWN_RISK, BORG_TOWN_HOME_DISCOUNT, BORG_TOWN_W_SPACE, BORG_TOWN_POWER_SCALE};
use crate::borg_headers::borg_trait_h::*;
use crate::borg_town_dispose::{Disposition, Freed, KitItem, Place};
use crate::borg_town_record::{StepInput, WareFact};
use crate::borg_town_solve::{Choice, Group, GroupKey, Limits};
use crate::borg_town_usefulness::{Need, Weights};
use crate::state::BorgState;

/// Record utility and supply counts in thousandths. This prevents cheap
/// marginal improvements disappearing merely because weights are small.
pub const UNIT: i64 = 1000;

pub fn index(need: Need) -> usize {
    Need::ALL.iter().position(|&n| n == need).unwrap_or(0)
}

#[derive(Clone, Copy, Debug)]
pub struct Journey {
    pub depth: i64,
    pub turns: i64,
    pub walk: i64,
    pub level: i64,
    pub hp: i64,
    pub advancement: i64,
    food_turns: i64,
    light_turns: i64,
    risk: i64,
    power_scale: i64,
}
impl Journey {
    pub fn of(st: &BorgState) -> Self { Self::for_trip(st, None) }

    pub fn for_trip(st: &BorgState, forecast: Option<crate::borg_trip::PlannedTrip>) -> Self {
        let known = i64::from(st.borg.tr(BI_MAXDEPTH)).clamp(0, 100);
        let requested = i64::from(st.cfg(BORG_TOWN_TARGET_DEPTH));
        let depth = if requested > 0 { requested } else { forecast.map_or_else(
            || known + i64::from(st.cfg(BORG_TOWN_DEPTH_GAIN)).clamp(1, 100), |f| i64::from(f.depth_target)) }.clamp(1, 100);
        let advancement = (depth - known).max(1);
        let walk = 2 * i64::from(st.cfg(BORG_TOWN_STAIR_TURNS)).clamp(0, 1_000_000) * depth;
        let declared = i64::from(st.cfg(BORG_TOWN_TRIP_TURNS));
        let turns = if declared > 0 { declared } else { i64::from(st.cfg(BORG_TOWN_EXPLORE_TURNS)).clamp(1, 1_000_000) * advancement + walk }.clamp(1, 1_000_000);
        Self { depth, turns, walk, advancement,
            food_turns: i64::from(st.cfg(BORG_TOWN_FOOD_TURNS)).clamp(1, 1_000_000),
            light_turns: i64::from(st.cfg(BORG_TOWN_LIGHT_TURNS)).clamp(1, 1_000_000),
            risk: i64::from(st.cfg(BORG_TOWN_RISK)).clamp(0, 1_000_000),
            power_scale: i64::from(st.cfg(BORG_TOWN_POWER_SCALE)).clamp(1, 1_000_000_000),
            level: i64::from(st.borg.tr(BI_CLEVEL)).clamp(1, 50),
            hp: i64::from(st.borg.tr(BI_MAXHP)).max(1) }
    }

    pub(crate) fn exposure(self) -> i64 {
        (self.turns * self.depth * self.risk / (UNIT * self.level)).max(0)
    }

    fn demand(self, need: Need) -> i64 {
        match need {
            Need::Light => (self.turns * UNIT / self.light_turns).max(1),
            Need::Food => (self.turns * UNIT / self.food_turns).max(1),
            Need::Healing => self.hp * self.exposure(),
            Need::Escape => self.exposure(),
            Need::Trip | Need::Poison | Need::Knowledge => UNIT,
            Need::Power => self.power_scale * self.advancement,
        }
    }
}

/// Saturating coverage, with decreasing marginal value and no stock cap.
fn coverage(weight: i64, amount: i64, demand: i64) -> i64 {
    if weight <= 0 || amount <= 0 { return 0; }
    weight.saturating_mul(UNIT).saturating_mul(amount) / (amount + demand.max(1))
}

#[derive(Clone, Debug)]
pub struct Utility {
    pub capability: Option<crate::borg_town_capability::Model>,
    pub journey: Journey,
    pub weights: Weights,
    pub carried: [i64; 8],
    pub home: [i64; 8],
    sustainable: u8,
    poisoned: bool,
    cut: bool,
    cure_spell: bool,
    wound_cures: i64,
    poison_cures: i64,
    pub gold_weight: i64,
    home_discount: i64,
    pub(crate) space_weight: i64,
}
impl Utility {
    pub fn of(st: &BorgState, kit: &[KitItem], cure_spell: bool) -> Self {
        let t = |i| i64::from(st.borg.tr(i)).max(0);
        let j = Journey::of(st);
        // Sentinel traits encode a reusable capability. They are never
        // interpreted as a thousand physical consumables in this model.
        let mut sustainable = 0;
        for (need, trait_id) in [(Need::Food, BI_FOOD), (Need::Escape, BI_APHASE), (Need::Trip, BI_RECALL)] {
            if t(trait_id) >= 1000 { sustainable |= 1 << index(need); }
        }
        let potion = |trait_id, hp: i64| (t(trait_id) % 1000) * hp.min(j.hp) * UNIT;
        let mut carried = [0; 8];
        carried[index(Need::Light)] = i64::from(crate::borg_town_usefulness::carried_lights(st)) * UNIT;
        carried[index(Need::Food)] = (t(BI_FOOD) % 1000) * UNIT;
        carried[index(Need::Healing)] = potion(BI_ACLW, 20) + potion(BI_ACSW, 40) + potion(BI_ACCW, 60)
            + potion(BI_AHEAL, 300) + potion(BI_AEZHEAL, 1200) + potion(BI_ALIFE, j.hp);
        carried[index(Need::Poison)] = (t(BI_ACUREPOIS) + t(BI_ACCW) + t(BI_AHEAL) + t(BI_AEZHEAL) + t(BI_ALIFE)) * UNIT;
        carried[index(Need::Escape)] = (t(BI_APHASE) % 1000) * UNIT;
        carried[index(Need::Trip)] = (t(BI_RECALL) % 1000) * UNIT;
        if !kit.is_empty() {
            // Carried unknown flavours have no player-known HP capacity.
            // Physical potion stock comes from the known kit, not hidden
            // kind data or an inherited device sentinel in a trait.
            carried[index(Need::Healing)] = kit.iter().filter(|k| k.place == Place::Pack
                && !k.unknown_flavour && k.need == index(Need::Healing) as i8)
                .map(|k| k.reduction * i64::from(k.qty)).sum();
        }
        let mut home = [0; 8];
        // Live kit facts use the same physical units as purchase facts.
        // Replace food's inherited equal-copy counter with actual kit units.
        if kit.iter().any(|k| k.place == Place::Pack && k.need == index(Need::Food) as i8) {
            carried[index(Need::Food)] = kit.iter()
                .filter(|k| k.place == Place::Pack && k.need == index(Need::Food) as i8)
                .map(|k| k.reduction * i64::from(k.qty)).sum();
        }
        for k in kit.iter().filter(|k| k.place == Place::Home && !k.unknown_flavour) {
            if let Ok(n) = usize::try_from(k.need) {
                if let Some(x) = home.get_mut(n) { *x += k.reduction * i64::from(k.qty); }
            }
        }
        Self { capability: None, journey: j, weights: Weights::of(&st.core.cfg), carried, home, sustainable,
            wound_cures: t(BI_ACLW) + t(BI_ACSW) + t(BI_ACCW) + t(BI_AHEAL) + t(BI_AEZHEAL) + t(BI_ALIFE) + i64::from(cure_spell),
            poison_cures: t(BI_ACUREPOIS) + t(BI_ACCW) + t(BI_AHEAL) + t(BI_AEZHEAL) + t(BI_ALIFE) + i64::from(cure_spell),
            poisoned: t(BI_ISPOISONED) > 0, cut: t(BI_ISCUT) > 0, cure_spell,
            home_discount: i64::from(st.cfg(BORG_TOWN_HOME_DISCOUNT)).clamp(0, UNIT),
            space_weight: i64::from(st.cfg(BORG_TOWN_W_SPACE)).max(0),
            gold_weight: i64::from(st.cfg(BORG_TOWN_W_GOLD)).max(0) * UNIT }
    }

    pub fn total(&self, need: Need, amount: i64) -> i64 {
        if self.sustainable & (1 << index(need)) != 0 { return 0; }
        let w = self.weights.get(need);
        match need {
            Need::Poison if !self.poisoned || self.cure_spell => 0,
            Need::Poison => if amount > 0 { w * UNIT } else { 0 },
            // Recall buys avoided walking, not a mandatory pair of scrolls.
            // Its value tends to zero for a nearby destination, and rises
            // smoothly as returning on foot occupies more of the journey.
            Need::Trip => coverage(w, amount, UNIT) * self.journey.walk / self.journey.turns.max(self.journey.walk),
            Need::Healing if self.cut && !self.cure_spell => {
                (coverage(w, amount, self.journey.demand(need)) * self.journey.exposure() / UNIT)
                    .max(if amount > 0 { w * UNIT } else { 0 })
            }
            Need::Food => coverage(w, amount, self.journey.demand(need)) * self.journey.demand(need) / UNIT,
            Need::Healing | Need::Escape => coverage(w, amount, self.journey.demand(need)) * self.journey.exposure() / UNIT,
            _ => coverage(w, amount, self.journey.demand(need)),
        }
    }

    pub fn add(&mut self, need: i8, units: i64) {
        if let Ok(n) = usize::try_from(need) {
            if let Some(x) = self.carried.get_mut(n) { *x += units; }
        }
    }

    pub fn gain(&self, need: Need, units: i64) -> i64 {
        let n = self.carried[index(need)];
        self.total(need, n + units) - self.total(need, n)
    }

    fn reserve(&self, need: Need, amount: i64) -> i64 {
        // Stored supplies retain an option for a later journey. Half the
        // priority discounts delay; this is a policy coefficient, not a
        // target quantity. Additional reserves have diminishing value.
        if need == Need::Poison { return 0; }
        let scale = if need == Need::Healing { self.journey.hp * UNIT } else { UNIT };
        coverage(self.weights.get(need), amount, scale) * self.home_discount / UNIT
    }

    fn reserve_gain(&self, need: Need, units: i64) -> i64 {
        let n = self.home[index(need)];
        self.reserve(need, n + units) - self.reserve(need, n)
    }

    pub fn power(&self, raw: i64) -> i64 {
        coverage(self.weights.power, raw.max(0), self.journey.demand(Need::Power))
    }

    fn change(&mut self, it: &KitItem, sign: i64) {
        if let Some(model) = &mut self.capability { model.change(it, sign); }
        if it.serves & (1 << index(Need::Healing)) != 0 { self.wound_cures += sign * i64::from(it.qty); }
        if it.serves & (1 << index(Need::Poison)) != 0 {
            self.poison_cures += sign * i64::from(it.qty);
            if it.need != index(Need::Poison) as i8 {
                self.carried[index(Need::Poison)] = (self.carried[index(Need::Poison)] + sign * i64::from(it.qty) * UNIT).max(0);
            }
        }
        if let Ok(n) = usize::try_from(it.need) {
            if let Some(v) = self.carried.get_mut(n) {
                *v = (*v + sign * it.reduction * i64::from(it.qty)).max(0);
            }
        }
    }

    fn keep(&self, it: &KitItem) -> i64 {
        if let Some(model) = &self.capability { return model.keep(it) + self.power(it.power); }
        let units = it.reduction * i64::from(it.qty);
        let v = usize::try_from(it.need).ok().and_then(|n| Need::ALL.get(n)).map_or(0, |&need| {
            let held = self.carried[index(need)];
            self.total(need, held) - self.total(need, (held - units).max(0))
        });
        v + self.power(it.power)
    }
}

/// Current model recognises the higher wound/poison potions too. Keep
/// the old helper unchanged so historical v4 records retain their semantics.
pub fn urgent_needs(st: &BorgState, trip: &crate::borg_town_usefulness::Trip) -> Vec<Need> {
    let advanced_cure = [BI_AHEAL, BI_AEZHEAL, BI_ALIFE].iter().any(|&i| st.borg.tr(i) > 0);
    crate::borg_town_usefulness::urgent_needs(st, trip).into_iter()
        .filter(|n| !advanced_cure || !matches!(n, Need::Healing | Need::Poison)).collect()
}

/// Opportunity cost of occupying pack room: the utility of finding an
/// unknown item spread across the room remaining. There is no fixed
/// pack reserve and encumbrance is still enforced by the solver.
pub(crate) fn burden(space_weight: i64, slots: i32, weight: i64, free_slots: i32, carry: i64) -> i64 {
    space_weight.max(0) * UNIT * i64::from(slots.max(0)) / (i64::from(free_slots.max(0)) + 1)
        + space_weight.max(0) * UNIT * weight.max(0) / carry.max(1)
}

/// Compare carrying, storing and selling on marginal utility, updating
/// the shared supply after EVERY disposition. The Home is a free option
/// for later trips even when the pack is not overweight.
pub fn dispositions(input: &StepInput, u: &mut Utility, st: &BorgState) -> (Vec<Disposition>, Freed) {
    let mut out = vec![Disposition::Keep; input.kit.len()];
    let mut freed = Freed::default();
    let carry = i64::from(st.borg.tr(BI_CARRY)).max(2) / 2;
    let mut home_free = input.home_slots;
    let mut pack: Vec<usize> = (0..input.kit.len()).filter(|&i| input.kit[i].place == Place::Pack).collect();
    pack.sort_by(|&a, &b| {
        let wa = (i64::from(input.kit[a].weight) * i64::from(input.kit[a].qty)).max(1);
        let wb = (i64::from(input.kit[b].weight) * i64::from(input.kit[b].qty)).max(1);
        (i128::from(u.keep(&input.kit[a])) * i128::from(wb))
            .cmp(&(i128::from(u.keep(&input.kit[b])) * i128::from(wa)))
            .then(input.kit[a].slot.cmp(&input.kit[b].slot))
    });
    for i in pack {
        let it = &input.kit[i];
        if it.unknown_flavour && !it.just_bought && (input.revision < 8 || it.sell_shop >= 0) {
            out[i] = if it.sell_shop >= 0 { Disposition::Sell } else { Disposition::Drop };
            u.change(it, -1);
            freed.pack_slots += 1;
            freed.weight += i64::from(it.weight) * i64::from(it.qty);
            if out[i] == Disposition::Sell { freed.gold += it.proceeds; }
            continue;
        }
        let loss = u.keep(it);
        let weight = i64::from(it.weight) * i64::from(it.qty);
        let cost = burden(u.space_weight, 1, weight, input.limits.pack_slots + freed.pack_slots, carry);
        let need = usize::try_from(it.need).ok().and_then(|n| Need::ALL.get(n)).copied();
        let future = u.capability.as_ref().map_or_else(
            || need.map_or(0, |n| u.reserve_gain(n, it.reduction * i64::from(it.qty))),
            |m| m.reserve(it, 1, u.home_discount)) + u.power(it.power) * u.home_discount / UNIT;
        // A worthless Home stack can pay for room even under no-selling.
        // Unknown durable items and this visit's purchases stay protected.
        let spare = input.kit.iter().enumerate().find(|(j, k)| k.place == Place::Home
            && out[*j] == Disposition::Keep
            && !u.capability.as_ref().is_some_and(|m| m.available.iter().any(|r| r[0] == 7 && r[1] == i64::from(k.slot) && r[2] == 0))
            && !(input.revision >= 9 && input.urgent.iter().any(|f| {
                let c = &input.candidates[f.candidate]; c.shop == 7 && c.slot == k.slot
            })) && !k.unknown && !k.just_bought && k.sell_shop >= 0
            && u.power(k.power) == 0 && (if let Some(m) = &u.capability {
                m.take(k) == 0 && m.reserve(k, -1, u.home_discount) == 0
            } else { k.unknown_flavour ||
                usize::try_from(k.need).ok().and_then(|n| Need::ALL.get(n)).is_none_or(|&n| {
                    let units = k.reduction * i64::from(k.qty);
                    u.gain(n, units) == 0 && u.reserve(n, u.home[index(n)]) == u.reserve(n, (u.home[index(n)] - units).max(0))
                }) }) ).map(|(j, _)| j);
        let home_cost = u.space_weight.max(0) * UNIT / (i64::from(home_free.max(0)) + 1);
        let loses_cure = (u.cut && it.serves & (1 << index(Need::Healing)) != 0 && u.wound_cures <= i64::from(it.qty))
            || (u.poisoned && it.serves & (1 << index(Need::Poison)) != 0 && u.poison_cures <= i64::from(it.qty));
        let urgent_loss = loses_cure || need.is_some_and(|n| {
            let left = u.carried[index(n)] - it.reduction * i64::from(it.qty);
            left <= 0 && match n {
                Need::Light => st.borg.tr(BI_LIGHT) < 1,
                Need::Food => st.borg.tr(BI_ISHUNGRY) != 0 || st.borg.tr(BI_ISWEAK) != 0,
                Need::Healing => u.cut && !u.cure_spell,
                Need::Poison => u.poisoned && !u.cure_spell,
                _ => false,
            }
        });
        let protected = it.unknown || it.just_bought || urgent_loss;
        let store = !urgent_loss && (input.revision < 6 || future > 0 || protected || loss > 0) && (home_free > 0 || spare.is_some()) && (future - home_cost > loss - cost ||
            input.limits.weight + freed.weight < 0 && (loss > 0 || future > 0 || protected));
        let action = if store { Disposition::Store }
            else if !protected && loss <= cost && future <= home_cost {
                if it.sell_shop >= 0 { Disposition::Sell } else { Disposition::Drop }
            } else { Disposition::Keep };
        out[i] = action;
        if action != Disposition::Keep {
            u.change(it, -1);
            freed.pack_slots += 1;
            freed.weight += weight;
            if action == Disposition::Store {
                if home_free <= 0 {
                    if let Some(j) = spare {
                        out[j] = Disposition::Sell;
                        if input.revision >= 9 {
                            if let Some(m) = &mut u.capability { m.change_home(&input.kit[j], -1); }
                            if let Ok(n) = usize::try_from(input.kit[j].need) {
                                if let Some(v) = u.home.get_mut(n) { *v = (*v - input.kit[j].reduction * i64::from(input.kit[j].qty)).max(0); }
                            }
                        }
                        freed.home_slots += 1;
                        freed.gold += input.kit[j].proceeds;
                        home_free += 1;
                    }
                }
                home_free -= 1;
                freed.home_slots -= 1;
                if let Some(m) = &mut u.capability { m.change_home(it, 1); }
                if let Some(n) = need { u.home[index(n)] += it.reduction * i64::from(it.qty); }
            } else if action == Disposition::Sell { freed.gold += it.proceeds; }
        }
    }
    // Reprice each withdrawal against the kit after previous withdrawals;
    // several Home stacks must not all satisfy the same missing need.
    for (i, it) in input.kit.iter().enumerate().filter(|(_, it)| it.place == Place::Home) {
        if out[i] != Disposition::Keep || it.unknown_flavour { continue; }
        if input.revision >= 9 && u.capability.as_ref().is_some_and(|m|
            m.available.iter().any(|r| r[0] == 7 && r[1] == i64::from(it.slot) && r[2] == 0)) { continue; }
        // One urgent copy is already accounted for. Re-evaluate the remaining
        // stack after that native withdrawal, rather than taking it twice.
        if input.revision >= 9 && input.urgent.iter().any(|f| {
            let c = &input.candidates[f.candidate]; c.shop == 7 && c.slot == it.slot
        }) { continue; }
        let weight = i64::from(it.weight) * i64::from(it.qty);
        let slots = input.limits.pack_slots + freed.pack_slots;
        let need = usize::try_from(it.need).ok().and_then(|n| Need::ALL.get(n)).copied();
        let units = it.reduction * i64::from(it.qty);
        let value = u.capability.as_ref().map_or_else(|| need.map_or(0, |n| u.gain(n, units)), |m| m.take(it)) + u.power(it.power);
        let future_loss = u.capability.as_ref().map_or_else(|| need.map_or(0, |n| {
            let h = u.home[index(n)];
            u.reserve(n, h) - u.reserve(n, (h - units).max(0))
        }), |m| -m.reserve(it, -1, u.home_discount)) + u.power(it.power) * u.home_discount / UNIT;
        let opening = i32::from(!it.joins_pack || input.revision < 9);
        let cost = burden(u.space_weight, opening, weight, slots, carry);
        if slots >= opening && weight <= input.limits.weight + freed.weight && value - cost > future_loss {
            out[i] = Disposition::Take;
            u.change(it, 1);
            if let Some(m) = &mut u.capability { m.change_home(it, -1); }
            if let Some(n) = need { u.home[index(n)] = (u.home[index(n)] - units).max(0); }
            freed.pack_slots -= opening;
            freed.home_slots += 1;
            freed.weight -= weight;
        } else if value == 0 && future_loss == 0 && !it.unknown && (input.revision < 6 || !it.just_bought) && it.proceeds > 0 && it.sell_shop >= 0 {
            out[i] = Disposition::Sell;
            if input.revision >= 9 {
                if let Some(m) = &mut u.capability { m.change_home(it, -1); }
                if let Some(n) = need { u.home[index(n)] = (u.home[index(n)] - units).max(0); }
            }
            freed.home_slots += 1;
            freed.gold += it.proceeds;
        }
    }
    (out, freed)
}

/// Merge substitute supplies into one group per NEED. Every option is
/// evaluated on aggregate coverage; buying CLW and CSW cannot both earn
/// the same full Healing weight. Keep all pure-kind options plus a
/// deterministic mixed path (best next marginal gain per gold). This is
/// an explicit feasible option set, not an exhaustive joint optimum.
pub fn shared_groups(groups: Vec<Group>, input: &StepInput, candidates: &[crate::borg_town_stock::Candidate], u: &Utility, limits: &Limits, st: &BorgState) -> Vec<Group> {
    let facts: BTreeMap<_, _> = input.facts.iter().map(|f| ((f.shop, f.slot), f)).collect();
    let mut by_need: BTreeMap<u8, Vec<Group>> = BTreeMap::new();
    let mut out = Vec::new();
    let input = StepInput {
        revision: input.revision, capability: input.capability, utility: input.utility, candidates: candidates.to_vec(), facts: input.facts, tryon: input.tryon, limits: input.limits,
        urgent: input.urgent.clone(), params: input.params, traits: input.traits, power: input.power, carried: input.carried,
        cure_spell: input.cure_spell, kit: input.kit, home_slots: input.home_slots };
    let input = &input;
    for group in groups {
        let need = group.options.iter().skip(1).flat_map(|o| &o.picks).next()
            .and_then(|&(i, _)| facts.get(&(input.candidates[i].shop, input.candidates[i].slot)))
            .filter(|f| f.need >= 0).map(|f| f.need as u8);
        if !matches!(group.key, GroupKey::Slot(_)) {
            if let Some(n) = need { by_need.entry(n).or_default().push(group); continue; }
        }
        out.push(group);
    }
    for (n, groups) in by_need {
        let mut options = vec![Choice::default()];
        for g in &groups {
            for oi in 1..g.options.len() {
                options.push(combine(std::slice::from_ref(g), &[oi], input, &facts, u, limits, st));
            }
        }
        let mut positions = vec![0; groups.len()];
        loop {
            let current = combine(&groups, &positions, input, &facts, u, limits, st);
            let mut best: Option<(usize, usize, Choice)> = None;
            for (gi, group) in groups.iter().enumerate() {
                let oi = positions[gi] + 1;
                if oi >= group.options.len() { continue; }
                positions[gi] = oi;
                let next = combine(&groups, &positions, input, &facts, u, limits, st);
                positions[gi] -= 1;
                let gain = next.value - current.value;
                let price = (next.cost - current.cost).max(1);
                let bm: i64 = next.picks.iter().filter(|&&(i, _)| input.candidates[i].is_black_market())
                    .map(|&(i, k)| i64::from(input.candidates[i].price) * i64::from(k)).sum();
                if bm.saturating_mul(u.weights.caution) > limits.gold || gain <= 0 || next.cost > limits.gold || next.weight > limits.weight || next.slots > limits.pack_slots { continue; }
                if best.as_ref().is_none_or(|(_, _, b)| i128::from(gain) * i128::from((b.cost - current.cost).max(1))
                    > i128::from(b.value - current.value) * i128::from(price)) {
                    best = Some((gi, oi, next));
                }
            }
            let Some((gi, oi, choice)) = best else { break; };
            positions[gi] = oi;
            options.push(choice);
        }
        out.push(Group { key: GroupKey::Need(n), options });
    }
    out
}

fn combine(groups: &[Group], pos: &[usize], input: &StepInput, facts: &BTreeMap<(u8, u8), &WareFact>, u: &Utility, limits: &Limits, st: &BorgState) -> Choice {
    let picks: Vec<_> = groups.iter().zip(pos).flat_map(|(g, &o)| g.options[o].picks.clone()).collect();
    let mut units = [0; 8];
    let mut learned = BTreeSet::new();
    let mut classes = BTreeSet::new();
    let mut choice = Choice { picks, ..Choice::default() };
    for &(i, count) in &choice.picks {
        let c = &input.candidates[i];
        choice.cost += i64::from(count) * i64::from(c.price);
        choice.weight += i64::from(count) * i64::from(c.weight);
        if c.slots_cost > 0 { classes.insert(c.class); }
        if let Some(f) = facts.get(&(c.shop, c.slot)) {
            if let Ok(n) = usize::try_from(f.need) { units[n] += f.reduction * i64::from(count); }
            if f.serves & (1 << index(Need::Knowledge)) != 0 { learned.insert(f.kind); }
        }
    }
    choice.slots = classes.len() as i32;
    choice.value = Need::ALL.iter().enumerate().map(|(i, &n)| u.gain(n, units[i])).sum::<i64>()
        + learned.len() as i64 * u.weights.knowledge.max(0) * UNIT
        - burden(u.space_weight, choice.slots, choice.weight, limits.pack_slots, i64::from(st.borg.tr(BI_CARRY)).max(2) / 2);
    choice
}

pub fn purchase_value(u: &Utility, f: Option<&&WareFact>, copies: i32) -> i64 {
    f.and_then(|f| usize::try_from(f.need).ok().and_then(|i| Need::ALL.get(i)).map(|&n| u.gain(n, f.reduction * i64::from(copies)))).unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::borg_headers::borg_h::BORG_TOWN_W_GOLD;
    use crate::borg_town_record::{PARAMS, StepInput};
    use crate::borg_town_solve::{solve_cash, Choice};

    fn state(depth: i32, level: i32, hp: i32) -> BorgState {
        let mut st = BorgState::default();
        st.core.cfg = crate::borg_init::BORG_SETTINGS.iter().map(|s| s.default_value).collect();
        st.borg.trait_ = vec![0; BI_MAX as usize];
        st.borg.trait_[BI_MAXDEPTH as usize] = depth;
        st.borg.trait_[BI_CLEVEL as usize] = level;
        st.borg.trait_[BI_MAXHP as usize] = hp;
        st.borg.trait_[BI_CARRY as usize] = 2000;
        st
    }

    #[test]
    fn destination_and_duration_express_ambition_independently() {
        let mut st = state(0, 1, 20);
        let initial = Journey::of(&st);
        assert_eq!(initial.depth, 1, "no invented five-level first dive");
        st.core.cfg[BORG_TOWN_TARGET_DEPTH as usize] = 2;
        st.core.cfg[BORG_TOWN_TRIP_TURNS as usize] = 4000;
        let explicit = Journey::of(&st);
        assert_eq!((explicit.depth, explicit.turns), (2, 4000));
        st.core.cfg[BORG_TOWN_TARGET_DEPTH as usize] = 30;
        assert_eq!(Journey::of(&st).turns, 4000);
    }

    #[test]
    fn longer_and_more_dangerous_trips_raise_supply_utility_without_quotas() {
        let mut st = state(0, 1, 20);
        let short = Utility::of(&st, &[], false);
        st.core.cfg[BORG_TOWN_TRIP_TURNS as usize] = 6000;
        let long = Utility::of(&st, &[], false);
        for n in [Need::Food, Need::Healing, Need::Escape] {
            let units = if n == Need::Healing { 20 * UNIT } else { UNIT };
            assert!(long.gain(n, units) > short.gain(n, units), "{n:?}");
        }
        st.core.cfg[BORG_TOWN_TARGET_DEPTH as usize] = 10;
        let danger = Utility::of(&st, &[], false);
        assert!(danger.gain(Need::Healing, 20 * UNIT) > long.gain(Need::Healing, 20 * UNIT));
        assert!(danger.gain(Need::Escape, UNIT) > long.gain(Need::Escape, UNIT));
        assert!(long.total(Need::Food, 20 * UNIT) > long.total(Need::Food, 10 * UNIT), "no hard ten-food cap");
    }

    #[test]
    fn marginal_value_decreases_and_healing_strength_tracks_hp() {
        let st = state(20, 20, 200);
        let u = Utility::of(&st, &[], false);
        for n in [Need::Food, Need::Healing, Need::Escape, Need::Trip] {
            let unit = if n == Need::Healing { 20 * UNIT } else { UNIT };
            let gains: Vec<_> = (0..10).map(|k| u.total(n, (k + 1) * unit) - u.total(n, k * unit)).collect();
            assert!(gains.windows(2).all(|x| x[1] <= x[0] + 1), "{n:?}: {gains:?}");
        }
        assert!(u.gain(Need::Healing, 60 * UNIT) > u.gain(Need::Healing, 20 * UNIT));
        let novice = Utility::of(&state(0, 1, 20), &[], false);
        assert!(novice.gain(Need::Healing, 20 * UNIT) > u.gain(Need::Healing, 20 * UNIT));
    }

    #[test]
    fn capability_sentinels_are_not_a_thousand_potions() {
        let mut st = state(10, 10, 100);
        st.borg.trait_[BI_ACLW as usize] = 1001;
        st.borg.trait_[BI_RECALL as usize] = 1000;
        let u = Utility::of(&st, &[], false);
        assert_eq!(u.carried[index(Need::Healing)], 20 * UNIT);
        assert_eq!(u.gain(Need::Trip, UNIT), 0, "usable recall spell substitutes for scrolls");
        assert!(u.gain(Need::Healing, 20 * UNIT) > 0, "a device/spell is not mana-free potion stock");
    }

    fn input<'a>(kit: &'a [KitItem]) -> StepInput<'a> {
        StepInput { revision: 7, capability: None, utility: true, candidates: Vec::new(), facts: &[], tryon: &[],
            limits: Limits { gold: 300, pack_slots: 15, weight: 1000 }, urgent: Vec::new(),
            params: [0; PARAMS], traits: &[], power: 0, carried: &[], cure_spell: false, kit, home_slots: 20 }
    }

    fn recall(place: Place) -> KitItem {
        KitItem { joins_pack: false, place, slot: 0, kind: 42, qty: 1, weight: 5, need: index(Need::Trip) as i8,
            reduction: UNIT, serves: 0, power: 0, proceeds: 0, sell_shop: 4, unknown: false, unknown_flavour: false, just_bought: false }
    }

    #[test]
    fn unknown_flavour_requires_a_buyer_and_is_never_dropped_for_learning() {
        let st = state(0, 1, 20);
        let unknown = KitItem { joins_pack: false, need: -1, reduction: 0, unknown: true,
            unknown_flavour: true, sell_shop: -1, ..recall(Place::Pack) };
        let kit = [unknown];
        let mut inp = input(&kit);
        inp.revision = 8;
        inp.home_slots = 0;
        let mut u = Utility::of(&st, inp.kit, false);
        assert_eq!(dispositions(&inp, &mut u, &st).0, [Disposition::Keep]);
        inp.home_slots = 20;
        let mut u = Utility::of(&st, inp.kit, false);
        assert_eq!(dispositions(&inp, &mut u, &st).0, [Disposition::Store]);
        // Historical records retain their original decision.
        inp.revision = 7;
        let mut u = Utility::of(&st, inp.kit, false);
        assert_eq!(dispositions(&inp, &mut u, &st).0, [Disposition::Drop]);
        let sale = [KitItem { joins_pack: false, sell_shop: 4, ..unknown }];
        let mut inp = input(&sale);
        inp.revision = 8;
        let mut u = Utility::of(&st, inp.kit, false);
        assert_eq!(dispositions(&inp, &mut u, &st).0, [Disposition::Sell]);
        // Stashing the item cannot be recorded as a learning sale.
        let home = [KitItem { joins_pack: false, place: Place::Home, ..unknown }];
        let mut inp = input(&home);
        inp.revision = 8;
        let mut u = Utility::of(&st, inp.kit, false);
        assert_eq!(dispositions(&inp, &mut u, &st).0, [Disposition::Keep]);
    }

    #[test]
    fn shallow_recall_is_stored_even_with_room_and_deep_recall_is_taken() {
        let mut st = state(0, 1, 20);
        st.borg.trait_[BI_RECALL as usize] = 1;
        let kit = [recall(Place::Pack)];
        let mut u = Utility::of(&st, &kit, false);
        let (actions, freed) = dispositions(&input(&kit), &mut u, &st);
        assert_eq!(actions, [Disposition::Store]);
        assert_eq!((freed.gold, freed.home_slots), (0, -1), "no sale or purchase needed");
        assert_eq!(u.carried[index(Need::Trip)], 0);
        let mut st = state(29, 20, 200);
        st.core.cfg[BORG_TOWN_TARGET_DEPTH as usize] = 30;
        let kit = [recall(Place::Home)];
        let mut u = Utility::of(&st, &kit, false);
        let (actions, _) = dispositions(&input(&kit), &mut u, &st);
        assert_eq!(actions, [Disposition::Take]);
    }

    #[test]
    fn shallow_recall_loses_to_saving_gold_and_a_useful_wound_potion() {
        let st = state(0, 1, 20);
        let u = Utility::of(&st, &[], false);
        let groups = [
            Group { key: GroupKey::Need(index(Need::Trip) as u8), options: vec![Choice::default(), Choice {
                cost: 150, value: u.gain(Need::Trip, UNIT), ..Choice::default() }] },
            Group { key: GroupKey::Need(index(Need::Healing) as u8), options: vec![Choice::default(), Choice {
                cost: 20, value: u.gain(Need::Healing, 20 * UNIT), ..Choice::default() }] },
        ];
        let s = solve_cash(&groups, &Limits { gold: 300, pack_slots: 15, weight: 1000 }, u.gold_weight);
        assert_eq!(s.chosen, [0, 1]);
        assert_eq!(s.cost, 20);
        assert!(s.optimal);
    }

    #[test]
    fn urgent_only_cure_cannot_be_stored_or_sold() {
        let mut st = state(0, 1, 20);
        st.borg.trait_[BI_ISCUT as usize] = 1;
        st.borg.trait_[BI_ACLW as usize] = 1;
        let kit = [KitItem { joins_pack: false, need: index(Need::Healing) as i8, reduction: 20 * UNIT, ..recall(Place::Pack) }];
        let mut u = Utility::of(&st, &kit, false);
        let mut inp = input(&kit);
        inp.limits.weight = -100;
        assert_eq!(dispositions(&inp, &mut u, &st).0, [Disposition::Keep]);
    }

    #[test]
    fn zoo_rates_change_forecasts_and_choices() {
        let mut st = state(0, 1, 20);
        let base = Utility::of(&st, &[], false);
        st.core.cfg[BORG_TOWN_DEPTH_GAIN as usize] = 4;
        assert_eq!(Journey::of(&st).depth, 4);
        st.core.cfg[BORG_TOWN_TARGET_DEPTH as usize] = 1;
        st.core.cfg[BORG_TOWN_EXPLORE_TURNS as usize] = 500;
        assert_eq!(Journey::of(&st).turns, 600);
        st.core.cfg[BORG_TOWN_STAIR_TURNS as usize] = 100;
        assert_eq!(Journey::of(&st).walk, 200);
        st.core.cfg[BORG_TOWN_TRIP_TURNS as usize] = 1100;
        st.core.cfg[BORG_TOWN_FOOD_TURNS as usize] = 6000;
        assert!(Utility::of(&st, &[], false).gain(Need::Food, UNIT) < base.gain(Need::Food, UNIT));
        st.core.cfg[BORG_TOWN_LIGHT_TURNS as usize] = 10000;
        assert!(Utility::of(&st, &[], false).gain(Need::Light, UNIT) > base.gain(Need::Light, UNIT));
        st.core.cfg[BORG_TOWN_RISK as usize] = 2000;
        assert!(Utility::of(&st, &[], false).gain(Need::Healing, 20 * UNIT) > base.gain(Need::Healing, 20 * UNIT));
        st.core.cfg[BORG_TOWN_RISK as usize] = 0;
        assert_eq!(Utility::of(&st, &[], false).gain(Need::Healing, 20 * UNIT), 0);
        st.core.cfg[BORG_TOWN_POWER_SCALE as usize] = 80000;
        assert!(Utility::of(&st, &[], false).power(40000) < base.power(40000));
        st.borg.trait_[BI_RECALL as usize] = 1;
        st.core.cfg[BORG_TOWN_HOME_DISCOUNT as usize] = 0;
        st.core.cfg[BORG_TOWN_W_SPACE as usize] = 0;
        let kit = [recall(Place::Pack)];
        let mut u = Utility::of(&st, &kit, false);
        assert_eq!(dispositions(&input(&kit), &mut u, &st).0, [Disposition::Keep]);
        st.core.cfg[BORG_TOWN_HOME_DISCOUNT as usize] = 1000;
        let mut u = Utility::of(&st, &kit, false);
        assert_eq!(dispositions(&input(&kit), &mut u, &st).0, [Disposition::Store]);
    }

    #[test]
    fn substitute_potions_share_one_aggregate_utility_curve() {
        use crate::borg_town_stock::Candidate;
        let st = state(9, 10, 100);
        let u = Utility::of(&st, &[], false);
        let candidate = |kind, slot, price| Candidate { shop: 4, slot, kind, tval: 1, sval: 0,
            price, qty: 2, weight: 4, slots_cost: 1, stack_count: 0, class: slot as u16 };
        let candidates = vec![candidate(100, 0, 20), candidate(101, 1, 35)];
        let facts = [WareFact { shop: 4, slot: 0, kind: 100, class: 0, slots_cost: 1, weight: 4,
            need: index(Need::Healing) as i8, reduction: 20 * UNIT, serves: 0 },
            WareFact { shop: 4, slot: 1, kind: 101, class: 1, slots_cost: 1, weight: 4,
            need: index(Need::Healing) as i8, reduction: 40 * UNIT, serves: 0 }];
        let mut inp = input(&[]);
        inp.candidates = candidates.clone();
        inp.facts = &facts;
        let groups: Vec<_> = candidates.iter().enumerate().map(|(i, c)| Group {
            key: GroupKey::Class(c.class), options: vec![Choice::default(), Choice {
                picks: vec![(i, 1)], cost: i64::from(c.price), slots: 1, weight: 4,
                value: u.gain(Need::Healing, facts[i].reduction) }] }).collect();
        let shared = shared_groups(groups, &inp, &candidates, &u, &inp.limits, &st);
        assert_eq!(shared.len(), 1, "two kinds are alternatives for one need");
        let mixed = shared[0].options.iter().find(|o| o.picks.len() == 2).expect("mixed option");
        let independent = u.gain(Need::Healing, 20 * UNIT) + u.gain(Need::Healing, 40 * UNIT);
        assert!(mixed.value < independent, "cannot count the same coverage twice");
        assert!(mixed.value <= u.gain(Need::Healing, 60 * UNIT));
    }

    #[test]
    fn a_secondary_poison_cure_is_protected_even_with_surplus_healing() {
        let mut st = state(0, 1, 20);
        st.borg.trait_[BI_ISPOISONED as usize] = 1;
        st.borg.trait_[BI_ACCW as usize] = 1;
        st.borg.trait_[BI_ACLW as usize] = 50;
        let kit = [KitItem { joins_pack: false, need: index(Need::Healing) as i8, reduction: 20 * UNIT,
            serves: (1 << index(Need::Healing)) | (1 << index(Need::Poison)), ..recall(Place::Pack) }];
        let mut u = Utility::of(&st, &kit, false);
        let mut inp = input(&kit);
        inp.limits.weight = -100;
        assert_eq!(dispositions(&inp, &mut u, &st).0, [Disposition::Keep]);
    }

    #[test]
    fn a_current_cut_does_not_reduce_the_coming_trips_healing_utility() {
        let mut st = state(29, 20, 200);
        st.core.cfg[BORG_TOWN_TRIP_TURNS as usize] = 10000;
        let healthy = Utility::of(&st, &[], false);
        st.borg.trait_[BI_ISCUT as usize] = 1;
        let cut = Utility::of(&st, &[], false);
        for hp in [20, 100, 500, 2000] {
            assert!(cut.total(Need::Healing, hp * UNIT) >= healthy.total(Need::Healing, hp * UNIT));
        }
    }

    #[test]
    fn advanced_healing_already_cures_current_cut_and_poison() {
        let mut st = state(20, 20, 200);
        st.borg.trait_[BI_ISCUT as usize] = 1;
        st.borg.trait_[BI_ISPOISONED as usize] = 1;
        st.borg.trait_[BI_AHEAL as usize] = 1;
        let trip = crate::borg_town_usefulness::Trip::of(&st);
        let needs = urgent_needs(&st, &trip);
        assert!(!needs.contains(&Need::Healing) && !needs.contains(&Need::Poison));
        assert!(crate::borg_town_usefulness::urgent_needs(&st, &trip).contains(&Need::Healing), "legacy helper unchanged");
    }

    #[test]
    fn unknown_flavours_do_not_count_as_known_healing_or_get_stored_by_hidden_kind() {
        let mut st = state(0, 1, 20);
        st.borg.trait_[BI_ACLW as usize] = 5;
        let unknown = KitItem { joins_pack: false, need: -1, reduction: 0, unknown_flavour: true, ..recall(Place::Pack) };
        let kit = [unknown];
        let mut u = Utility::of(&st, &kit, false);
        assert_eq!(u.carried[index(Need::Healing)], 0);
        assert_eq!(dispositions(&input(&kit), &mut u, &st).0, [Disposition::Sell]);
        let home = [KitItem { joins_pack: false, place: Place::Home, ..unknown }];
        let mut u = Utility::of(&st, &home, false);
        assert_eq!(dispositions(&input(&home), &mut u, &st).0, [Disposition::Keep], "unaware Home stock is no known supply");
    }

    #[test]
    fn full_home_frees_worthless_stock_for_needed_storage_without_gold() {
        let mut st = state(0, 1, 20);
        st.borg.trait_[BI_FOOD as usize] = 1;
        let junk = KitItem { joins_pack: false, place: Place::Home, need: -1, reduction: 0, ..recall(Place::Pack) };
        let heavy = KitItem { joins_pack: false, slot: 1, need: index(Need::Food) as i8, weight: 2000, ..recall(Place::Pack) };
        let kit = [junk, heavy];
        let mut u = Utility::of(&st, &kit, false);
        let mut inp = input(&kit);
        inp.home_slots = 0;
        inp.limits.weight = -1900;
        let (actions, freed) = dispositions(&inp, &mut u, &st);
        assert_eq!(actions, [Disposition::Sell, Disposition::Store]);
        assert_eq!((freed.gold, freed.home_slots, freed.weight), (0, 0, 2000));
    }

    #[test]
    fn overweight_gear_stores_the_lowest_utility_per_weight_first() {
        let st = state(0, 1, 20);
        let a = KitItem { joins_pack: false, need: -1, reduction: 0, weight: 100, power: 200000, ..recall(Place::Pack) };
        let b = KitItem { joins_pack: false, slot: 1, power: 40000, ..a };
        let c = KitItem { joins_pack: false, slot: 2, weight: 10, ..b };
        let kit = [a, b, c];
        let mut u = Utility::of(&st, &kit, false);
        let mut inp = input(&kit);
        inp.limits.weight = -50;
        assert_eq!(dispositions(&inp, &mut u, &st).0, [Disposition::Keep, Disposition::Store, Disposition::Keep]);
    }

    #[test]
    fn gold_weight_is_an_explicit_priority_not_a_reserved_fraction() {
        let mut st = state(0, 1, 20);
        st.core.cfg[BORG_TOWN_W_GOLD as usize] = 0;
        assert_eq!(Utility::of(&st, &[], false).gold_weight, 0);
    }
    #[test]
    fn a_home_join_can_be_withdrawn_into_a_full_pack() {
        let mut st=state(10,10,100);st.core.cfg[BORG_TOWN_W_SPACE as usize]=0;
        let kit=[KitItem {joins_pack:true,just_bought:true,..recall(Place::Home)}];
        let mut inp=input(&kit);inp.revision=9;inp.limits.pack_slots=0;
        let mut u=Utility::of(&st,&kit,false);u.home_discount=0;
        let (d,freed)=dispositions(&inp,&mut u,&st);assert_eq!(d,[Disposition::Take]);assert_eq!(freed.pack_slots,0);
        let kit=[KitItem {joins_pack:false,..kit[0]}];let mut inp=input(&kit);inp.revision=9;inp.limits.pack_slots=0;
        let mut u=Utility::of(&st,&kit,false);u.home_discount=0;
        assert_eq!(dispositions(&inp,&mut u,&st).0,[Disposition::Keep]);
    }
    #[test]
    fn a_shared_forecast_preserves_declared_overrides_for_town_and_throwing() {
        let mut st=state(0,1,20);
        let forecast=crate::borg_trip::PlannedTrip {depth_target:12,levels:1,recall_scrolls:0,recall_carry:0};
        let town=Journey::for_trip(&st,Some(forecast));assert_eq!(town.depth,12);
        st.core.cfg[BORG_TOWN_TARGET_DEPTH as usize]=2;st.core.cfg[BORG_TOWN_TRIP_TURNS as usize]=777;
        let explicit=Journey::for_trip(&st,Some(forecast));assert_eq!((explicit.depth,explicit.turns),(2,777));
    }

}
