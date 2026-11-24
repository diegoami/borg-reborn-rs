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
use crate::params::{value as param, Param};
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

    pub(crate) fn fuel_goal(self, reserve: i64) -> i64 {
        self.turns.max(reserve)
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
    /// borg-reborn-rs (#557): `town_recall_reserve`, the Word of Recall
    /// valued at the full trip weight each, whatever the depth.
    pub(crate) recall_reserve: i64,
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
            gold_weight: i64::from(st.cfg(BORG_TOWN_W_GOLD)).max(0) * UNIT,
            recall_reserve: i64::from(crate::params::value(crate::params::Param::town_recall_reserve)).max(0) }
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
            // borg-reborn-rs (#557): the first `recall_reserve` scrolls are
            // worth the full weight each, at any depth: without one a shallow
            // Borg walks home, and the walk can fail (priest-1055640847).
            Need::Trip => crate::borg_town_capability::recall_value_scaled(
                coverage(w, amount, UNIT) * self.journey.walk / self.journey.turns.max(self.journey.walk),
            )
                .max(w * amount.min(self.recall_reserve * UNIT)),
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
    let mut out: Vec<Need> = crate::borg_town_usefulness::urgent_needs(st, trip).into_iter()
        .filter(|n| !advanced_cure || !matches!(n, Need::Healing | Need::Poison)).collect();
    if recall_urgent(st) { out.push(Need::Trip); }
    if crate::borg_town_capability::ranged_shopping() {
        if st.borg.tr(BI_APHASE) < 1 { out.push(Need::Escape); }
        if !advanced_cure && [BI_ACLW, BI_ACSW, BI_ACCW].iter().all(|&id| st.borg.tr(id) < 1)
            && !out.contains(&Need::Healing)
        {
            out.push(Need::Healing);
        }
    }
    out
}

/// borg-reborn-rs (#859, owner: "Word of recall should have a high value"):
/// with `town_recall_urgent`, the first Word of Recall is urgent when the
/// next trip reaches the dive gate (`prepared_recall_depth`) and none is
/// carried: without it the Borg cannot dive, whatever else it buys.
pub(crate) fn recall_urgent(st: &BorgState) -> bool {
    let gate = urgent_recall_gate(param(Param::town_recall_urgent_depth), param(Param::prepared_recall_depth)); // borg-reborn-rs (#984)
    param(Param::town_recall_urgent) != 0
        && recall_gate_reached(Journey::of(st).depth, gate, st.borg.tr(BI_RECALL))
}

fn recall_gate_reached(trip_depth: i64, gate: i32, carried: i32) -> bool {
    gate > 0 && trip_depth >= i64::from(gate) && carried < 1
}

/// Opportunity cost of occupying pack room: the utility of finding an
/// unknown item spread across the room remaining. There is no fixed
/// pack reserve and encumbrance is still enforced by the solver.
pub(crate) fn burden(space_weight: i64, slots: i32, weight: i64, free_slots: i32, carry: i64) -> i64 {
    space_weight.max(0) * UNIT * i64::from(slots.max(0)) / (i64::from(free_slots.max(0)) + 1)
        + space_weight.max(0) * UNIT * weight.max(0) / carry.max(1)
}

/// borg-reborn-rs (#859, owner: "every time they leave they should leave
/// with some space for a few objects of a different kind"): the room a town
/// departure keeps free, as weight under the carry allowance and as pack
/// slots. Both zero (off) by default.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct Room {
    pub weight: i64,
    pub slots: i32,
}

pub(crate) fn departure_room() -> Room {
    Room {
        weight: i64::from(param(Param::town_departure_free_weight)),
        slots: param(Param::town_departure_free_slots),
    }
}

/// The storing settings `dispositions` reads, passed in so that tests can set
/// them (the registry gives tests its compiled defaults). A plan record
/// carries them from schema 13 (#957), so a replay stores as the visit did.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct Storing {
    /// #711: `town_store_only_overweight`.
    pub(crate) only_overweight: bool,
    /// #859: the departure room.
    pub(crate) room: Room,
    /// #859: `town_store_keep_min`.
    pub(crate) keep_min: i32,
    /// #557: `town_recall_reserve`, and the gold floor that makes it count
    /// (`buy_recall_from_gold`): together they set the Recall room the
    /// storing target adds (review of #962).
    pub(crate) recall_reserve: i32,
    pub(crate) recall_floor: i32,
    pub(crate) home_stack_limit: i32,
}

impl Storing {
    /// The process's settings: what a live solve uses.
    pub(crate) fn settings() -> Self {
        Storing {
            only_overweight: param(Param::town_store_only_overweight) != 0,
            room: departure_room(),
            keep_min: param(Param::town_store_keep_min),
            recall_reserve: param(Param::town_recall_reserve),
            recall_floor: crate::borg_store_buy::recall_purchase_floor(),
            home_stack_limit: param(Param::town_home_stack_limit),
        }
    }

    /// As a plan record writes them: `[free weight, free slots, keep_min,
    /// only_overweight, recall_reserve, recall_floor, home_stack_limit]`.
    pub(crate) fn to_array(self) -> [i64; 7] {
        [self.room.weight, i64::from(self.room.slots), i64::from(self.keep_min), i64::from(self.only_overweight),
            i64::from(self.recall_reserve), i64::from(self.recall_floor), i64::from(self.home_stack_limit)]
    }

    pub(crate) fn from_array(a: [i64; 7]) -> Self {
        Storing {
            only_overweight: a[3] != 0,
            room: Room { weight: a[0], slots: a[1] as i32 },
            keep_min: a[2] as i32,
            recall_reserve: a[4] as i32,
            recall_floor: a[5] as i32,
            home_stack_limit: a[6] as i32,
        }
    }
}

/// Repeated copies of these known kinds are interchangeable at Home.
pub(crate) fn home_limited_kind(tval: u8) -> bool {
    use crate::headers::list_tvals_h::{TV_FLASK, TV_FOOD, TV_MUSHROOM, TV_POTION, TV_SCROLL};
    matches!(u32::from(tval), TV_FLASK | TV_FOOD | TV_MUSHROOM | TV_POTION | TV_SCROLL)
}

pub(crate) fn home_stack_limit() -> usize {
    param(Param::town_home_stack_limit) as usize
}

/// Work with physical copies and slots, rather than aggregate capability
/// estimates (which can include spells or substitute kinds).
fn home_stock(kit: &[KitItem]) -> (BTreeMap<u32, i32>, BTreeMap<u32, usize>) {
    let mut pack = BTreeMap::new();
    let mut home = BTreeMap::new();
    for item in kit.iter().filter(|item| item.home_limited && !item.unknown && !item.unknown_flavour) {
        match item.place {
            Place::Pack => *pack.entry(item.kind).or_default() += item.qty,
            Place::Home => *home.entry(item.kind).or_default() += 1,
        }
    }
    (pack, home)
}

fn usable_floor(u: &Utility, item: &KitItem, held: i32, minimum: i32) -> i32 {
    minimum.max(1).max(u.capability.as_ref().map_or(0, |model| model.home_stock_floor(item, held)))
}

fn home_disposal_copies(input: &StepInput, u: &Utility, item: &KitItem) -> i32 {
    if input.urgent.iter().any(|purchase| {
        let candidate = &input.candidates[purchase.candidate];
        candidate.shop == 7 && candidate.slot == item.slot
    }) {
        return 0;
    }
    // Detect Invisible has independent pack protection. Only withdraw
    // surplus for disposal if every copy can go through native overflow.
    if input.revision >= 19 && item.protect_disposal && !item.home_unwanted
        && !item.home_room.is_some_and(|[_, _, acquire]| acquire == 0)
    {
        return 0;
    }
    let full_pack = input.limits.pack_slots <= 0;
    if let Some([carry, merge, _]) = item.home_room {
        let copies = item.qty.min(if full_pack { carry.min(merge) } else { carry });
        if input.revision < 16 && copies < item.qty { return 0; }
        return copies;
    }
    let blocked = u.capability.as_ref().is_some_and(|model| model.available.iter()
        .any(|r| r[0] == 7 && r[1] == i64::from(item.slot) && r[2] == 0));
    if blocked || full_pack && !item.joins_pack { 0 } else { item.qty }
}

fn change_home_stock(u: &mut Utility, item: &KitItem, sign: i64) {
    if let Some(model) = &mut u.capability { model.change_home(item, sign); }
    if let Some(need) = usize::try_from(item.need).ok().and_then(|n| u.home.get_mut(n)) {
        *need = (*need + sign * item.reduction * i64::from(item.qty)).max(0);
    }
}

/// Clear deliberately unwanted stock ahead of small duplicate stacks.
/// After the withdrawal, the carried stack is sold or dropped.
fn prune_home(input: &StepInput, u: &mut Utility, limit: usize, minimum: i32,
    pack: &BTreeMap<u32, i32>, home: &mut BTreeMap<u32, usize>,
    dispositions: (&mut [Disposition], &mut Vec<i32>, &mut Freed),
) {
    let (out, copies, freed) = dispositions;
    let mut order: Vec<_> = input.kit.iter().enumerate()
        .filter(|(_, item)| item.place == Place::Home && (item.home_unwanted || item.home_limited)).collect();
    order.sort_by_key(|(_, item)| (!item.home_unwanted, item.qty, item.slot));
    let mut partial_stacks = BTreeMap::<u32, usize>::new();
    for (i, item) in order {
        let count = home.get(&item.kind).copied().unwrap_or(0);
        let held = pack.get(&item.kind).copied().unwrap_or(0);
        let pending = partial_stacks.get(&item.kind).copied().unwrap_or(0);
        let surplus = limit > 0 && count > limit + pending && held >= usable_floor(u, item, held, minimum);
        if !(item.home_unwanted || surplus)
            || item.unknown || item.unknown_flavour
            // Kind-level purchase history cannot make an existing surplus
            // Home stack a purchase. The carried reserve is checked above.
            || (item.just_bought && (input.revision < 17 || !surplus))
            || (item.sell_shop < 0 && !(input.revision >= 15 && item.home_unwanted))
        { continue; }
        // A recent shop purchase shields the merged pack stack from
        // ordinary sale. Leave surplus at Home until it can be disposed of.
        if input.revision >= 18 && !item.home_unwanted
            && input.kit.iter().any(|carried| carried.place == Place::Pack
                && carried.kind == item.kind && carried.just_bought)
        { continue; }
        let qty = home_disposal_copies(input, u, item);
        if qty <= 0 { continue; }
        out[i] = if item.sell_shop >= 0 { Disposition::Sell } else { Disposition::Drop };
        let proceeds = item.proceeds * i64::from(qty) / i64::from(item.qty);
        if qty == item.qty {
            if let Some(stacks) = home.get_mut(&item.kind) { *stacks -= 1; }
            freed.home_slots += 1;
        } else {
            // This slot stays occupied, but is already the next surplus
            // to finish. Keep the larger stacks reserved by the policy.
            *partial_stacks.entry(item.kind).or_default() += 1;
            copies.resize(input.kit.len(), 0);
            copies[i] = qty;
        }
        change_home_stock(u, &KitItem { qty, ..*item }, -1);
        freed.gold += proceeds;
    }
}

/// When Home and the carried kit cover a kind, its nonurgent offers
/// have no claim on the shopping budget. Use the after-disposition kit.
pub(crate) fn saturated_home_kinds(input: &StepInput, u: &Utility, dispositions: &[Disposition], copies: &[i32]) -> BTreeSet<u32> {
    let Some(storing) = input.storing.filter(|storing| storing.home_stack_limit > 0) else {
        return BTreeSet::new();
    };
    let (mut pack, mut home) = home_stock(input.kit);
    for (i, (item, &action)) in input.kit.iter().zip(dispositions).enumerate() {
        if !item.home_limited || item.unknown || item.unknown_flavour { continue; }
        let qty = copies.get(i).copied().filter(|&qty| qty > 0).unwrap_or(item.qty);
        match (item.place, action) {
            (Place::Pack, Disposition::Sell | Disposition::Store | Disposition::Drop) => {
                *pack.entry(item.kind).or_default() -= qty;
            }
            (Place::Home, Disposition::Take) => {
                *pack.entry(item.kind).or_default() += qty;
                *home.entry(item.kind).or_default() -= usize::from(qty == item.qty);
            }
            (Place::Home, Disposition::Sell | Disposition::Drop) => *home.entry(item.kind).or_default() -= usize::from(qty == item.qty),
            _ => {}
        }
        if action == Disposition::Store { *home.entry(item.kind).or_default() += 1; }
    }
    input.kit.iter().filter(|item| item.home_limited && !item.unknown && !item.unknown_flavour)
        .filter_map(|item| {
            let held = pack.get(&item.kind).copied().unwrap_or(0);
            (home.get(&item.kind).copied().unwrap_or(0) >= storing.home_stack_limit as usize
                && held >= usable_floor(u, item, held, storing.keep_min)).then_some(item.kind)
        }).collect()
}

/// What storing frees before the purchases: the departure room, plus the
/// room the Word of Recall reserve still needs (#859: a Borg with 20,000 gold
/// at the allowance could not take a 0.5 lb scroll, and the dive gate waited
/// for it). The purchases then leave the departure room free, so the scroll
/// fits on top of it. Off when the departure room is.
fn storing_target(input: &StepInput, u: &Utility, room: Room) -> Room {
    if room == Room::default() { return room; }
    let (weight, slots) = u.capability.as_ref()
        .map_or((0, 0), |m| crate::borg_town_capability::recall_room(m, u.recall_reserve, &input.candidates));
    Room { weight: room.weight + weight, slots: room.slots + slots }
}

/// The order the pack is visited in: lowest keep value per weight first. With
/// `town_store_keep_min` (#859, owner: "prefer objects that they have an
/// abundance of, typically healing potions and phase doors, ideally we want
/// to have variety in the backpack"), stacks with copies past the minimum come
/// first, priced by one copy: on a diminishing curve the tenth Cure Light
/// Wounds is worth little, the only Detect Invisible a lot.
fn store_order(input: &StepInput, u: &Utility, surplus: impl Fn(&KitItem) -> bool) -> Vec<usize> {
    let mut pack: Vec<usize> = (0..input.kit.len()).filter(|&i| input.kit[i].place == Place::Pack).collect();
    let priced = |k: &KitItem| if surplus(k) { KitItem { qty: 1, ..*k } } else { *k };
    pack.sort_by(|&a, &b| {
        let (ka, kb) = (priced(&input.kit[a]), priced(&input.kit[b]));
        let wa = (i64::from(ka.weight) * i64::from(ka.qty)).max(1);
        let wb = (i64::from(kb.weight) * i64::from(kb.qty)).max(1);
        surplus(&input.kit[b]).cmp(&surplus(&input.kit[a]))
            .then((i128::from(u.keep(&ka)) * i128::from(wb)).cmp(&(i128::from(u.keep(&kb)) * i128::from(wa))))
            .then(input.kit[a].slot.cmp(&input.kit[b].slot))
    });
    pack
}

/// borg-reborn-rs (#859): a pack stack of flasks of oil.
fn is_flask(st: &BorgState, it: &KitItem) -> bool {
    use crate::headers::list_tvals_h::TV_FLASK;
    it.place == Place::Pack && st.items.items.get(usize::from(it.slot))
        .is_some_and(|i| i.iqty > 0 && u32::from(i.tval) == TV_FLASK)
}

/// borg-reborn-rs (#831): a pack stack of Scrolls of Detect Invisible whose
/// flavour is known (an unknown flavour is a scroll nobody has read yet).
fn keeps_detect_invisible(st: &BorgState, it: &KitItem) -> bool {
    use crate::headers::list_tvals_h::TV_SCROLL;
    it.place == Place::Pack && !it.unknown_flavour && st.items.items.get(usize::from(it.slot))
        .is_some_and(|i| i.iqty > 0 && u32::from(i.tval) == TV_SCROLL && i32::from(i.sval) == st.item_val.sv_scroll_detect_invis)
}

/// Compare carrying, storing and selling on marginal utility, updating
/// the shared supply after EVERY disposition. The Home is a free option
/// for later trips even when the pack is not overweight.
pub fn dispositions(input: &StepInput, u: &mut Utility, st: &BorgState) -> (Vec<Disposition>, Freed, Vec<i32>) {
    // #957: the record's settings when a replay has them; older records and
    // live solves read the process's.
    dispositions_by(input, u, st, input.storing.unwrap_or_else(Storing::settings))
}

/// `dispositions` with `town_store_only_overweight` given (#711).
#[cfg(test)]
fn dispositions_with(input: &StepInput, u: &mut Utility, st: &BorgState, only_overweight: bool) -> (Vec<Disposition>, Freed, Vec<i32>) {
    dispositions_by(input, u, st, Storing { only_overweight, ..Storing::default() })
}

/// `dispositions` with the storing settings given.
/// The third value records explicit quantities for split and short-kit
/// moves. A zero or absent quantity denotes the whole stack.
fn dispositions_by(input: &StepInput, u: &mut Utility, st: &BorgState, storing: Storing) -> (Vec<Disposition>, Freed, Vec<i32>) {
    let Storing { only_overweight, room, keep_min, home_stack_limit, .. } = storing;
    let mut out = vec![Disposition::Keep; input.kit.len()];
    let mut freed = Freed::default();
    let mut copies: Vec<i32> = Vec::new();
    let carry = i64::from(st.borg.tr(BI_CARRY)).max(2) / 2;
    let limit = home_stack_limit.max(0) as usize;
    let (mut pack_counts, mut home_counts) = if limit > 0 { home_stock(input.kit) } else { Default::default() };
    if limit > 0 || input.kit.iter().any(|item| item.home_unwanted) {
        prune_home(input, u, limit, keep_min, &pack_counts, &mut home_counts, (&mut out, &mut copies, &mut freed));
    }
    let mut home_free = input.home_slots + freed.home_slots;
    let target = storing_target(input, u, room);
    // A stack only frees a slot when all of it goes, so the surplus comes
    // first only when the pack is short of weight, not of slots.
    let surplus_first = keep_min > 0 && input.limits.pack_slots >= target.slots;
    let surplus = |k: &KitItem| surplus_first && k.qty > keep_min;
    // borg-reborn-rs (#859, re-review of #864): the minimum is a floor, not
    // only an order. While storing for weight, a stack that serves a need
    // (potions, scrolls, torches, food: `need` or `serves`) or a stack of
    // flasks of oil (owner: "check that flasks are taken"; oil serves no
    // `need`, its value is the capability model's) is kept whole once it is
    // down to `keep_min` copies: not stored, sold or dropped, so the live
    // path (one copy per solve, #507) stops there. An item that serves no
    // need (a lone armour piece) has no floor and can still go; short of
    // slots, whole stacks go.
    let floored = |k: &KitItem| surplus_first && k.qty <= keep_min && !k.unknown_flavour
        && (k.need >= 0 || k.serves != 0 || is_flask(st, k));
    for i in store_order(input, u, surplus) {
        let whole = &input.kit[i];
        if whole.home_unwanted && !whole.unknown && !whole.unknown_flavour {
            out[i] = if whole.sell_shop >= 0 { Disposition::Sell } else { Disposition::Drop };
            u.change(whole, -1);
            if let Some(held) = pack_counts.get_mut(&whole.kind) { *held -= whole.qty; }
            freed.pack_slots += 1;
            freed.weight += i64::from(whole.weight) * i64::from(whole.qty);
            if out[i] == Disposition::Sell { freed.gold += whole.proceeds; }
            continue;
        }
        // The copies past the minimum, when they go first. They travel with
        // the disposition (the third value returned), so the plan's action moves the
        // surplus and not the whole stack (#859, review F1). The live
        // executor still moves one copy and solves again (#507).
        let restore_keep = u.capability.as_ref().map_or(0, |model| model.restore_life_keep(whole));
        if restore_keep > 0 && restore_keep >= whole.qty { continue; }
        let held = pack_counts.get(&whole.kind).copied().unwrap_or(0);
        let capped = limit > 0 && whole.home_limited && !whole.unknown && !whole.unknown_flavour
            && home_counts.get(&whole.kind).copied().unwrap_or(0) >= limit;
        let capped_floor = if capped { (usable_floor(u, whole, held, keep_min) - (held - whole.qty)).max(0) } else { 0 };
        if capped && capped_floor >= whole.qty { continue; }
        // borg-reborn-rs (#1151, plan schema 20): a stack keeps the copies
        // every weighted reserve it serves still needs (the light reserve,
        // physical fuel counted; Phase Door, ranged, Restore Life, ...), and
        // the rest can go, stored, sold or dropped; before, a fuel stack was
        // skipped whole when all of it would undercut the light reserve.
        // Review of #1155: the floor is set before the move is priced, so a
        // six-scroll Phase Door reserve is neither stored nor sold down to
        // `town_store_keep_min`.
        let reserve = if input.revision >= 20 {
            u.capability.as_ref().map_or(0, |model| model.reserve_keep(whole))
        } else { 0 };
        if reserve >= whole.qty { continue; }
        let book_floor = i32::from(whole.keep_last_book && whole.qty > 1);
        let floor = capped_floor.max(restore_keep).max(reserve).max(book_floor)
            .max(if surplus(whole) { keep_min } else { 0 });
        let qty = whole.qty - floor;
        let proceeds = if restore_keep > 0 || capped || reserve > 0 || book_floor > 0 {
            whole.proceeds * i64::from(qty) / i64::from(whole.qty)
        } else { whole.proceeds };
        let part = KitItem { qty, proceeds, ..*whole };
        let partial = floor > 0 && !whole.unknown_flavour;
        let it = if partial { &part } else { whole };
        if input.revision < 20 && u.capability.as_ref().is_some_and(|model| model.protects_fuel(whole)) {
            continue;
        }
        if floored(whole) { continue; }
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
            && !k.keep_last_book && (input.revision < 19 || !k.protect_disposal)
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
        // borg-reborn-rs (#831, owner: "selling detect invisible scrolls is
        // wrong in the first place, once identified"): a known Scroll of
        // Detect Invisible is carried, or stored while overweight, never
        // sold or dropped; the copy cap (`borg_town_max_non_missile_copies`)
        // still sends the copies past it away.
        let potion_or_scroll_protected = if capped { it.protect_disposal } else { keeps_detect_invisible(st, it) };
        // borg-reborn-rs (#1151, plan schema 20): an unknown pack item is
        // kept to learn its runes, but one known to be cursed that adds no
        // power is not worth carrying over the weight allowance (a CL17 Mage
        // at -12 speed carried a 25 lb cursed Lochaber Axe). It is stored if
        // it is worth that, else sold or dropped. An unknown item with no
        // known curse stays protected.
        let cursed_spare = input.revision >= 20 && it.cursed_spare && it.place == Place::Pack
            && u.power(it.power) == 0 && input.limits.weight + freed.weight < 0;
        let protected = (it.unknown && !cursed_spare) || it.just_bought || urgent_loss || potion_or_scroll_protected
            || (whole.keep_last_book && !partial);
        // borg-reborn-rs (#651, plan schema 12): what this visit bought is
        // carried, not stored: a Cure Light Wounds bought to carry went to
        // the Home and was bought again (borg-cemetery#75).
        // borg-reborn-rs (#711, owner 2026-11-20: "only when it weighs too
        // much they should stash the object with less value at home, until
        // they are not overcarrying again"): with
        // `town_store_only_overweight`, a stack goes home only while the pack
        // is over its weight allowance. The pack is visited lowest keep value
        // per weight first, and `freed.weight` ends the storing once it is
        // under; a stack worth more kept for later than carried now stays.
        // borg-reborn-rs (#859): with `town_departure_free_weight` or
        // `town_departure_free_slots`, storing goes on until the departure
        // room (and the Recall reserve's) is free, not only to the allowance.
        let overweight = input.limits.weight + freed.weight < target.weight
            || (target.slots > 0 && input.limits.pack_slots + freed.pack_slots < target.slots);
        let worth_more_at_home = !only_overweight && future - home_cost > loss - cost;
        let store = !capped && !(input.revision >= 12 && it.just_bought) && !urgent_loss && (input.revision < 6 || future > 0 || protected || loss > 0) && (home_free > 0 || spare.is_some()) && (worth_more_at_home ||
            overweight && (loss > 0 || future > 0 || protected));
        // borg-reborn-rs (#1151, plan schema 20): over the weight allowance
        // itself, with no room at Home, the copies past a stack's floor (the
        // light reserve, `town_store_keep_min`, the Restore Life reserve) are
        // excess: sold, or dropped when no shop buys them, the least
        // valuable per weight first until the pack is under its allowance.
        // The floor keeps every weighted reserve. A drop applies only to a
        // fuel stack, whose split the live executor carries out
        // (`execute_outside`); it drops other stacks whole.
        let excess = input.revision >= 20 && partial && (it.sell_shop >= 0 || it.fuel > 0)
            && input.limits.weight + freed.weight < 0;
        let worthless = capped || loss <= cost && future <= home_cost;
        let action = if store { Disposition::Store }
            else if !protected && (worthless || excess) {
                if it.sell_shop >= 0 { Disposition::Sell } else { Disposition::Drop }
            } else { Disposition::Keep };
        // Second re-review of #1155: a drop whose split keeps a reserve is
        // planned only for a stack the live executor splits (a fuel stack,
        // Restore Life Levels, a kind at its Home limit: `execute_outside`).
        // It drops any other stack whole, so seven Phase Door with a reserve
        // of six and no buyer would all go: kept instead.
        let unsplittable = action == Disposition::Drop && partial && (reserve > 0 || book_floor > 0)
            && it.fuel <= 0 && restore_keep <= 0 && !capped;
        let action = if unsplittable { Disposition::Keep } else { action };
        out[i] = action;
        // Review of #1155: what goes only because the pack is over its
        // allowance is the copies that cover the deficit, not all past the
        // floor (one 2.2 lb torch for a 1 lb deficit, not thirty).
        let trimmed;
        let (it, weight) = if excess && !worthless && action != Disposition::Store && action != Disposition::Keep {
            let deficit = -(input.limits.weight + freed.weight);
            let one = i64::from(it.weight.max(1));
            let need = i32::try_from(((deficit + one - 1) / one).max(1)).unwrap_or(i32::MAX);
            if need < it.qty {
                trimmed = KitItem { qty: need, proceeds: whole.proceeds * i64::from(need) / i64::from(whole.qty), ..*it };
                (&trimmed, one * i64::from(need))
            } else { (it, weight) }
        } else { (it, weight) };
        if action != Disposition::Keep {
            u.change(it, -1);
            if let Some(held) = pack_counts.get_mut(&it.kind) { *held -= it.qty; }
            freed.pack_slots += i32::from(!partial);
            freed.weight += weight;
            if partial {
                copies.resize(input.kit.len(), 0);
                copies[i] = it.qty;
            }
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
                        if limit > 0 && input.kit[j].home_limited {
                            if let Some(count) = home_counts.get_mut(&input.kit[j].kind) { *count -= 1; }
                        }
                        freed.home_slots += 1;
                        freed.gold += input.kit[j].proceeds;
                        home_free += 1;
                    }
                }
                home_free -= 1;
                freed.home_slots -= 1;
                if limit > 0 && it.home_limited { *home_counts.entry(it.kind).or_default() += 1; }
                change_home_stock(u, it, 1);
            } else if action == Disposition::Sell { freed.gold += it.proceeds; }
        }
    }
    // Reprice each withdrawal against the kit after previous withdrawals;
    // several Home stacks must not all satisfy the same missing need.
    for (i, it) in input.kit.iter().enumerate().filter(|(_, it)| it.place == Place::Home) {
        if out[i] != Disposition::Keep || it.unknown_flavour || it.home_unwanted { continue; }
        let held = pack_counts.get(&it.kind).copied().unwrap_or(0);
        if limit > 0 && it.home_limited && home_counts.get(&it.kind).copied().unwrap_or(0) >= limit
            && held >= usable_floor(u, it, held, keep_min)
        { continue; }
        if input.revision >= 9 && u.capability.as_ref().is_some_and(|m|
            m.available.iter().any(|r| r[0] == 7 && r[1] == i64::from(it.slot) && r[2] == 0)) { continue; }
        // One urgent copy is already accounted for. Re-evaluate the remaining
        // stack after that native withdrawal, rather than taking it twice.
        if input.revision >= 9 && input.urgent.iter().any(|f| {
            let c = &input.candidates[f.candidate]; c.shop == 7 && c.slot == it.slot
        }) { continue; }
        let slots = input.limits.pack_slots + freed.pack_slots;
        let weight_room = input.limits.weight + freed.weight - room.weight;
        let short = if limit > 0 && it.home_limited { usable_floor(u, it, held, keep_min) - held } else { 0 };
        let withdrawal = it.home_room.filter(|_| short > 0)
            .map(|[carry, merge, acquire]| {
                let slot_room = if slots > room.slots { carry } else { merge.min(carry) };
                let by_weight = if it.weight > 0 { weight_room / i64::from(it.weight) } else { i64::from(it.qty) };
                let qty = i64::from(it.qty.min(short).min(slot_room).min(acquire)).min(by_weight);
                (qty as i32, merge)
            });
        let mut taken = *it;
        if let Some((qty, _)) = withdrawal {
            if qty <= 0 { continue; }
            taken.qty = qty;
        }
        let it = &taken;
        let weight = i64::from(it.weight) * i64::from(it.qty);
        let need = usize::try_from(it.need).ok().and_then(|n| Need::ALL.get(n)).copied();
        let units = it.reduction * i64::from(it.qty);
        let value = u.capability.as_ref().map_or_else(|| need.map_or(0, |n| u.gain(n, units)), |m| m.take(it)) + u.power(it.power);
        let future_loss = u.capability.as_ref().map_or_else(|| need.map_or(0, |n| {
            let h = u.home[index(n)];
            u.reserve(n, h) - u.reserve(n, (h - units).max(0))
        }), |m| -m.reserve(it, -1, u.home_discount)) + u.power(it.power) * u.home_discount / UNIT;
        let opening = i32::from(withdrawal.map_or(!it.joins_pack || input.revision < 9, |(qty, merge)| qty > merge));
        let cost = burden(u.space_weight, opening, weight, slots, carry);
        // borg-reborn-rs (#859): a withdrawal leaves the departure room free.
        let whole = it.qty == input.kit[i].qty;
        if slots - room.slots >= opening && weight <= weight_room && value - cost > future_loss {
            out[i] = Disposition::Take;
            if withdrawal.is_some() {
                copies.resize(input.kit.len(), 0);
                copies[i] = it.qty;
            }
            if limit > 0 && it.home_limited { *pack_counts.entry(it.kind).or_default() += it.qty; }
            u.change(it, 1);
            change_home_stock(u, it, -1);
            if whole && limit > 0 && it.home_limited {
                if let Some(count) = home_counts.get_mut(&it.kind) { *count -= 1; }
            }
            freed.pack_slots -= opening;
            freed.home_slots += i32::from(whole);
            freed.weight -= weight;
        } else if whole && value == 0 && future_loss == 0 && !it.unknown && (input.revision < 6 || !it.just_bought)
            && !it.keep_last_book && (input.revision < 19 || !it.protect_disposal) && it.proceeds > 0 && it.sell_shop >= 0 {
            out[i] = Disposition::Sell;
            if input.revision >= 9 {
                if let Some(m) = &mut u.capability { m.change_home(it, -1); }
                if let Some(n) = need { u.home[index(n)] = (u.home[index(n)] - units).max(0); }
            }
            freed.home_slots += 1;
            freed.gold += it.proceeds;
        }
    }
    (out, freed, copies)
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
        black_market_reserve: input.black_market_reserve, black_market_percentage: input.black_market_percentage, urgent: input.urgent.clone(), params: input.params, traits: input.traits, power: input.power, carried: input.carried, storing: input.storing,
        cure_spell: input.cure_spell, reads_now: input.reads_now, kit: input.kit, home_slots: input.home_slots };
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
                let market_budget = crate::borg_town_budget::black_market_budget(limits.gold, input.black_market_reserve, input.black_market_percentage, u.weights.caution);
                if ((u.weights.caution > 0 || input.black_market_reserve > 0 || input.black_market_percentage >= 0) && bm > market_budget) || gain <= 0 || next.cost > limits.gold || next.weight > limits.weight || next.slots > limits.pack_slots { continue; }
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
        StepInput { storing: None, revision: 7, capability: None, utility: true, black_market_reserve: 0, black_market_percentage: -1, candidates: Vec::new(), facts: &[], tryon: &[],
            limits: Limits { gold: 300, pack_slots: 15, weight: 1000 }, urgent: Vec::new(),
            params: [0; PARAMS], traits: &[], power: 0, carried: &[], cure_spell: false, reads_now: false, kit, home_slots: 20 }
    }

    fn recall(place: Place) -> KitItem {
        KitItem { home_room: None, keep_last_book: false, protect_disposal: false, home_unwanted: false, home_limited: false, joins_pack: false, place, slot: 0, kind: 42, qty: 1, weight: 5, need: index(Need::Trip) as i8,
            reduction: UNIT, serves: 0, power: 0, proceeds: 0, sell_shop: 4, unknown: false, unknown_flavour: false, just_bought: false, cursed_spare: false, fuel: 0 }
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
    fn what_this_visit_bought_is_not_stored() {
        // #651: the shallow Recall below is stored; bought this visit, it
        // stays in the pack from plan schema 12 on.
        let mut st = state(0, 1, 20);
        st.borg.trait_[BI_RECALL as usize] = 1;
        let kit = [KitItem { just_bought: true, ..recall(Place::Pack) }];
        let mut inp = input(&kit);
        inp.revision = 12;
        let mut u = Utility::of(&st, &kit, false);
        assert_eq!(dispositions(&inp, &mut u, &st).0, [Disposition::Keep]);
        // Historical records keep their decision.
        inp.revision = 11;
        let mut u = Utility::of(&st, &kit, false);
        assert_eq!(dispositions(&inp, &mut u, &st).0, [Disposition::Store]);
        // Review of #654: the reported shape, a stack bought unaware (the
        // flavour learned on purchase) and still flagged unknown.
        let unaware = [KitItem { just_bought: true, need: -1, reduction: 0, unknown: true,
            unknown_flavour: true, sell_shop: -1, ..recall(Place::Pack) }];
        let mut inp = input(&unaware);
        inp.revision = 11;
        let mut u = Utility::of(&st, inp.kit, false);
        assert_eq!(dispositions(&inp, &mut u, &st).0, [Disposition::Store]);
        inp.revision = 12;
        let mut u = Utility::of(&st, inp.kit, false);
        assert_eq!(dispositions(&inp, &mut u, &st).0, [Disposition::Keep]);
    }

    /// #711 (owner): with `town_store_only_overweight`, nothing goes home
    /// while the pack is within its weight allowance; over it, the stacks
    /// with the least keep value per weight go home until it is under.
    #[test]
    fn only_an_overweight_pack_stores_the_least_valuable_first() {
        let mut st = state(0, 1, 20);
        st.borg.trait_[BI_RECALL as usize] = 1;
        // The shallow Recall goes home today even with room (the test below).
        let kit = [recall(Place::Pack)];
        let mut u = Utility::of(&st, &kit, false);
        assert_eq!(dispositions_with(&input(&kit), &mut u, &st, false).0, [Disposition::Store], "today");
        let mut u = Utility::of(&st, &kit, false);
        assert_eq!(dispositions_with(&input(&kit), &mut u, &st, true).0, [Disposition::Keep], "within the allowance");
        // Over the allowance by 3: one 5-weight stack is enough, the one
        // worth least per weight (the shallow Recall, worth nothing now).
        let food = KitItem { slot: 1, kind: 43, weight: 5, need: index(Need::Food) as i8, ..recall(Place::Pack) };
        let kit = [recall(Place::Pack), food];
        let mut inp = input(&kit);
        inp.limits.weight = -3;
        let mut u = Utility::of(&st, &kit, false);
        let (d, freed, _) = dispositions_with(&inp, &mut u, &st, true);
        assert_eq!(d.iter().filter(|&&x| x == Disposition::Store).count(), 1, "{d:?}");
        assert_eq!(d[0], Disposition::Store, "the least valuable goes: {d:?}");
        assert!(inp.limits.weight + freed.weight >= 0, "under the allowance after it");
        // Review of #714: the public entry point reads the switch the right
        // way round (at the value the registry gives this test process).
        let kit = [recall(Place::Pack)];
        let on = crate::params::value(crate::params::Param::town_store_only_overweight) != 0;
        let mut u = Utility::of(&st, &kit, false);
        let public = dispositions(&input(&kit), &mut u, &st).0;
        let mut u = Utility::of(&st, &kit, false);
        assert_eq!(public, dispositions_with(&input(&kit), &mut u, &st, on).0);
        let mut u = Utility::of(&st, &kit, false);
        assert_ne!(public, dispositions_with(&input(&kit), &mut u, &st, !on).0, "the switch changes this case");
    }

    #[test]
    fn shallow_recall_is_stored_even_with_room_and_deep_recall_is_taken() {
        let mut st = state(0, 1, 20);
        st.borg.trait_[BI_RECALL as usize] = 1;
        let kit = [recall(Place::Pack)];
        let mut u = Utility::of(&st, &kit, false);
        let (actions, freed, _) = dispositions(&input(&kit), &mut u, &st);
        assert_eq!(actions, [Disposition::Store]);
        assert_eq!((freed.gold, freed.home_slots), (0, -1), "no sale or purchase needed");
        assert_eq!(u.carried[index(Need::Trip)], 0);
        let mut st = state(29, 20, 200);
        st.core.cfg[BORG_TOWN_TARGET_DEPTH as usize] = 30;
        let kit = [recall(Place::Home)];
        let mut u = Utility::of(&st, &kit, false);
        let (actions, _, _) = dispositions(&input(&kit), &mut u, &st);
        assert_eq!(actions, [Disposition::Take]);
    }

    #[test]
    fn shallow_recall_loses_to_saving_gold_and_a_useful_wound_potion() {
        // The soft gold price at borg-rs's 25 (the compiled default is 0
        // since #493): saving gold still beats a shallow recall when priced.
        let mut st = state(0, 1, 20);
        st.core.cfg[BORG_TOWN_W_GOLD as usize] = 25;
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

    /// #557: with `town_recall_reserve` 1, the shallow Recall that loses to
    /// saving gold above is bought, and a carried one is kept in the pack.
    #[test]
    fn a_recall_reserve_buys_and_keeps_a_shallow_recall() {
        let mut st = state(0, 1, 20);
        st.core.cfg[BORG_TOWN_W_GOLD as usize] = 25;
        let mut u = Utility::of(&st, &[], false);
        u.recall_reserve = 1;
        assert_eq!(u.gain(Need::Trip, UNIT), 50 * UNIT, "the first scroll is worth the full weight");
        let groups = [
            Group { key: GroupKey::Need(index(Need::Trip) as u8), options: vec![Choice::default(), Choice {
                cost: 150, value: u.gain(Need::Trip, UNIT), ..Choice::default() }] },
            Group { key: GroupKey::Need(index(Need::Healing) as u8), options: vec![Choice::default(), Choice {
                cost: 20, value: u.gain(Need::Healing, 20 * UNIT), ..Choice::default() }] },
        ];
        let s = solve_cash(&groups, &Limits { gold: 300, pack_slots: 15, weight: 1000 }, u.gold_weight);
        assert_eq!(s.chosen, [1, 1], "the Recall is bought with the potion");

        st.borg.trait_[BI_RECALL as usize] = 1;
        let kit = [recall(Place::Pack)];
        let mut u = Utility::of(&st, &kit, false);
        u.recall_reserve = 1;
        let (actions, _, _) = dispositions(&input(&kit), &mut u, &st);
        assert_eq!(actions, [Disposition::Keep], "the reserve scroll stays in the pack");
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
    fn a_known_detect_invisible_scroll_is_never_sold() {
        // #831 (owner): worth nothing to the plan (no need, no power) and
        // sellable, a known Scroll of Detect Invisible is still kept (or stored); the
        // same stack in another kind is sold.
        let mut st = state(0, 1, 20);
        st.item_val.sv_scroll_detect_invis = 31;
        let scroll = KitItem { joins_pack: false, need: -1, reduction: 0, sell_shop: 4, ..recall(Place::Pack) };
        let kit = [scroll];
        let mut u = Utility::of(&st, &kit, false);
        assert_eq!(dispositions(&input(&kit), &mut u, &st).0, [Disposition::Sell], "any other worthless stack");
        st.items.items.push(crate::borg_item::blank_item());
        st.items.items[0].tval = crate::headers::list_tvals_h::TV_SCROLL as u8;
        st.items.items[0].sval = 31;
        st.items.items[0].iqty = 1;
        let mut u = Utility::of(&st, &kit, false);
        let kept = dispositions(&input(&kit), &mut u, &st).0[0];
        assert!(matches!(kept, Disposition::Keep | Disposition::Store), "Detect Invisible is never sold or dropped: {kept:?}");
        let unread = [KitItem { unknown_flavour: true, ..scroll }];
        let mut u = Utility::of(&st, &unread, false);
        assert_eq!(dispositions(&input(&unread), &mut u, &st).0, [Disposition::Sell], "not yet identified: sold as before");
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
        let (actions, freed, _) = dispositions(&inp, &mut u, &st);
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
        let (d,freed,_)=dispositions(&inp,&mut u,&st);assert_eq!(d,[Disposition::Take]);assert_eq!(freed.pack_slots,0);
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


    /// borg-reborn-rs (#451): a carried Remove Hunger scroll supplies
    /// nutrition (35: 50 to a hungry 15 percent), so the planner keeps it
    /// below the food floor; valued at nothing it was stored or sold in the
    /// visit that bought it.
    #[test]
    fn a_carried_remove_hunger_scroll_is_kept_while_food_is_short() {
        use crate::borg_town_capability::{Model, PACK};
        let st = state(0, 1, 20);
        let scroll = KitItem { joins_pack: false, need: -1, reduction: 0, weight: 5, sell_shop: 5, proceeds: 5, ..recall(Place::Pack) };
        let kit = [scroll];
        let floor = 5 * UNIT;
        let model = |units: i64| {
            let mut curves: Vec<[i64; 5]> = (0..58).map(|i| [i as i64, 0, UNIT, 0, 0]).collect();
            // the food floor: a requirement linear up to five rations' nutrition
            curves[1] = [1, 100 * floor, -(crate::borg_town_capability::RATION_NUTRITION * floor), units, 0];
            Model { available: Vec::new(), packing: None, curves,
                supplies: if units > 0 { vec![[PACK, 0, 1, units]] } else { Vec::new() } }
        };
        let mut u = Utility::of(&st, &kit, false);
        u.capability = Some(model(35 * UNIT));
        assert_eq!(dispositions(&input(&kit), &mut u, &st).0, [Disposition::Keep]);
        // worth nothing (before #451): sold at once
        let mut u = Utility::of(&st, &kit, false);
        u.capability = Some(model(0));
        assert_eq!(dispositions(&input(&kit), &mut u, &st).0, [Disposition::Sell]);
    }

    /// borg-reborn-rs (#452): a known Potion of Speed is an emergency
    /// reserve, valued per potion like the escapes (exposure), so the
    /// planner keeps it while the pack has room. Valued as 20+2d10 turns
    /// of buff against the whole trip (the old curve) it was about one
    /// percent of the demand, under its slot's burden, and was sold.
    #[test]
    fn a_known_speed_potion_is_kept_while_the_pack_has_room() {
        use crate::borg_town_capability::{Model, PACK};
        use crate::borg_headers::borg_h::{BORG_TOWN_W_ESCAPE, BORG_TOWN_W_SUPPORT};
        const SPEED: usize = 16;
        // the measured arm's trip: 100000 turns
        let mut st = state(5, 25, 100);
        st.core.cfg[BORG_TOWN_TRIP_TURNS as usize] = 100_000;
        let journey = Journey::of(&st);
        let exposure = journey.exposure();
        let weight = i64::from(st.cfg(BORG_TOWN_W_ESCAPE)) * exposure;
        let potion = KitItem { joins_pack: false, need: -1, reduction: 0, weight: 4, sell_shop: 5, proceeds: 40, ..recall(Place::Pack) };
        let kit = [potion];
        let model = |curve: [i64; 5], units: i64| {
            let mut curves: Vec<[i64; 5]> = (0..58).map(|i| [i as i64, 0, UNIT, 0, 0]).collect();
            curves[SPEED] = curve;
            Model { available: Vec::new(), packing: None, curves, supplies: vec![[PACK, 0, SPEED as i64, units]] }
        };
        // the pack has room (limits.pack_slots is 15): kept, not sold or stored
        let mut u = Utility::of(&st, &kit, false);
        u.capability = Some(model([SPEED as i64, weight, exposure, UNIT, 0], UNIT));
        let mut inp = input(&kit);
        inp.revision = 9;
        assert_eq!(dispositions(&inp, &mut u, &st).0, [Disposition::Keep]);
        // several potions: the stack is kept whole
        let stack = [KitItem { qty: 4, ..kit[0] }];
        let mut u = Utility::of(&st, &stack, false);
        u.capability = Some(model([SPEED as i64, weight, exposure, 4 * UNIT, 0], UNIT));
        let mut inp = input(&stack);
        inp.revision = 9;
        assert_eq!(dispositions(&inp, &mut u, &st).0, [Disposition::Keep]);
        // the old valuation, 31 turns of buff against the trip: sold
        let ordinary = i64::from(st.cfg(BORG_TOWN_W_SUPPORT)) * exposure;
        let mut u = Utility::of(&st, &kit, false);
        u.capability = Some(model([SPEED as i64, ordinary, journey.turns * UNIT, 31 * UNIT, 0], 31 * UNIT));
        let mut inp = input(&kit);
        inp.revision = 9;
        assert_ne!(dispositions(&inp, &mut u, &st).0, [Disposition::Keep]);
    }

    /// #859 (owner): a pack at its allowance makes the departure room by
    /// storing, the surplus of its abundant stack first: the variety stays.
    #[test]
    fn the_departure_room_is_made_from_the_surplus_first() {
        let mut st = state(0, 1, 20);
        st.borg.trait_[BI_ACLW as usize] = 12;
        st.borg.trait_[BI_APHASE as usize] = 1;
        let potions = KitItem { slot: 0, kind: 50, qty: 12, weight: 4, need: index(Need::Healing) as i8,
            reduction: 20 * UNIT, ..recall(Place::Pack) };
        let phase = KitItem { slot: 1, kind: 51, qty: 1, weight: 5, need: index(Need::Escape) as i8, ..recall(Place::Pack) };
        let kit = [potions, phase];
        let mut inp = input(&kit);
        inp.limits.weight = 0;
        let storing = |keep_min| Storing { only_overweight: true, room: Room { weight: 30, slots: 0 }, keep_min, ..Storing::default() };
        // Off: nothing goes, the pack is within its allowance.
        let mut u = Utility::of(&st, &kit, false);
        assert_eq!(dispositions_by(&inp, &mut u, &st, Storing { only_overweight: true, ..Storing::default() }).0,
            [Disposition::Keep, Disposition::Keep]);
        // With the room and three copies kept: nine potions free 36, the
        // Phase Door stays, and no slot is freed (the stack stays).
        let mut u = Utility::of(&st, &kit, false);
        let (d, freed, copies) = dispositions_by(&inp, &mut u, &st, storing(3));
        assert_eq!(d, [Disposition::Store, Disposition::Keep], "the surplus goes first");
        assert_eq!((freed.weight, freed.pack_slots), (36, 0));
        assert_eq!(copies, [9, 0], "the nine travel with the disposition");
        // A minimum at or past the stack: the stack is floored and stays.
        let mut u = Utility::of(&st, &kit, false);
        let (d, freed, _) = dispositions_by(&inp, &mut u, &st, storing(12));
        assert_eq!((d[0], freed.pack_slots), (Disposition::Keep, 0), "the floor keeps it whole");
    }

    /// #957 (review of #962): the step's entry point stores as the record's
    /// settings say, not the process's: the same kit with the departure
    /// room recorded stores the surplus, and with none recorded keeps it.
    #[test]
    fn the_recorded_storing_settings_decide_the_dispositions() {
        let mut st = state(0, 1, 20);
        st.borg.trait_[BI_ACLW as usize] = 12;
        st.borg.trait_[BI_APHASE as usize] = 1;
        let potions = KitItem { slot: 0, kind: 50, qty: 12, weight: 4, need: index(Need::Healing) as i8,
            reduction: 20 * UNIT, ..recall(Place::Pack) };
        let phase = KitItem { slot: 1, kind: 51, qty: 1, weight: 5, need: index(Need::Escape) as i8, ..recall(Place::Pack) };
        let kit = [potions, phase];
        let mut inp = input(&kit);
        inp.limits.weight = 0;
        inp.storing = Some(Storing { only_overweight: true, room: Room { weight: 30, slots: 0 }, keep_min: 3, ..Storing::default() });
        let mut u = Utility::of(&st, &kit, false);
        assert_eq!(dispositions(&inp, &mut u, &st).0, [Disposition::Store, Disposition::Keep]);
        inp.storing = Some(Storing { only_overweight: true, ..Storing::default() });
        let mut u = Utility::of(&st, &kit, false);
        assert_eq!(dispositions(&inp, &mut u, &st).0, [Disposition::Keep, Disposition::Keep]);
    }

    /// #859 (re-review of #864): the live path stores one copy per solve
    /// (#507). Solved again at every count, a stack of 12 with three kept
    /// goes down to three and stops there, freeing no slot; the minimum is a
    /// floor, not only an order.
    #[test]
    fn the_live_walk_stops_at_the_minimum() {
        let mut st = state(0, 1, 20);
        let storing = Storing { only_overweight: true, room: Room { weight: 30, slots: 0 }, keep_min: 3, ..Storing::default() };
        let mut stored_at = Vec::new();
        for qty in (1..=12).rev() {
            st.borg.trait_[BI_ACLW as usize] = qty;
            let potions = KitItem { slot: 0, kind: 50, qty, weight: 4, need: index(Need::Healing) as i8,
                reduction: 20 * UNIT, ..recall(Place::Pack) };
            let kit = [potions];
            let mut inp = input(&kit);
            inp.limits.weight = 0;
            let mut u = Utility::of(&st, &kit, false);
            let (d, freed, _) = dispositions_by(&inp, &mut u, &st, storing);
            if d[0] == Disposition::Store {
                stored_at.push(qty);
                assert_eq!(freed.pack_slots, 0, "a stack that stays frees no slot ({qty} copies)");
            }
        }
        assert_eq!(stored_at, (4..=12).rev().collect::<Vec<_>>(), "stored down to the minimum, never below it");
    }

    /// #859: the floor is for stacks that serve a need. A lone item that
    /// serves none (an armour piece) still goes home while the pack is short
    /// of weight.
    #[test]
    fn an_item_that_serves_no_need_has_no_floor() {
        let st = state(0, 1, 20);
        let armour = KitItem { slot: 0, kind: 70, qty: 1, weight: 200, need: -1, serves: 0, reduction: 0,
            power: 0, ..recall(Place::Pack) };
        let kit = [armour];
        let mut inp = input(&kit);
        inp.limits.weight = -100;
        let storing = |keep_min| Storing { only_overweight: true, room: Room { weight: 30, slots: 0 }, keep_min, ..Storing::default() };
        let mut u = Utility::of(&st, &kit, false);
        let off = dispositions_by(&inp, &mut u, &st, storing(0)).0[0];
        let mut u = Utility::of(&st, &kit, false);
        let on = dispositions_by(&inp, &mut u, &st, storing(3)).0[0];
        assert_eq!(on, off, "the minimum changes nothing for an item that serves no need");
    }

    /// #859 (owner: "check that flasks are taken"): oil serves no `need`,
    /// but a stack of flasks is floored too. Walked one solve per copy, 12
    /// flasks go home down to three; at three or fewer they are kept, not
    /// stored, sold or dropped (the re-review of `c2dc7793` found three
    /// flasks sold).
    #[test]
    fn flasks_are_floored_too() {
        let mut st = state(0, 1, 20);
        st.items.items.push(crate::borg_item::blank_item());
        st.items.items[0].tval = crate::headers::list_tvals_h::TV_FLASK as u8;
        let storing = Storing { only_overweight: true, room: Room { weight: 30, slots: 0 }, keep_min: 3, ..Storing::default() };
        let mut gone = Vec::new();
        for qty in (1..=12).rev() {
            st.items.items[0].iqty = qty as u8;
            let oil = KitItem { slot: 0, kind: 80, qty, weight: 10, need: -1, serves: 0, reduction: 0,
                sell_shop: 1, ..recall(Place::Pack) };
            let kit = [oil];
            let mut inp = input(&kit);
            inp.limits.weight = 0;
            let mut u = Utility::of(&st, &kit, false);
            let d = dispositions_by(&inp, &mut u, &st, storing).0[0];
            if d != Disposition::Keep { gone.push(qty); }
        }
        assert_eq!(gone, (4..=12).rev().collect::<Vec<_>>(), "flasks leave down to the minimum, never below it");
    }

    #[test]
    fn unwanted_treasure_scrolls_make_room_for_a_useful_staple() {
        let st = state(9, 26, 198);
        let treasure = KitItem { protect_disposal: false, home_unwanted: true, home_limited: true, kind: 78, qty: 20,
            need: -1, reduction: 0, proceeds: 0, sell_shop: 4, ..recall(Place::Pack) };
        let useful = KitItem { protect_disposal: false, home_limited: true, kind: 51, slot: 1, qty: 4,
            need: index(Need::Escape) as i8, reduction: UNIT, ..recall(Place::Pack) };
        let at_home = KitItem { place: Place::Home, qty: 40, joins_pack: true, ..treasure };
        let kit = [treasure, useful, at_home];
        let mut inp = input(&kit);
        inp.home_slots = 0;
        inp.limits.pack_slots = 2;
        inp.limits.weight = -300;
        let storing = Storing { only_overweight: true, ..Storing::default() };
        let mut u = Utility::of(&st, &kit, false);
        let (out, _, _) = dispositions_by(&inp, &mut u, &st, storing);
        assert_eq!(out, [Disposition::Sell, Disposition::Store, Disposition::Sell]);
        let unmapped = KitItem { unknown: true, unknown_flavour: true, sell_shop: -1, ..at_home };
        let kit = [unmapped];
        let inp = input(&kit);
        let mut u = Utility::of(&st, &kit, false);
        assert_eq!(dispositions_by(&inp, &mut u, &st, storing).0, [Disposition::Keep]);
    }

    #[test]
    fn unsellable_home_stock_is_withdrawn_and_then_dropped() {
        use crate::borg_town_record::{action, first_executable_action, step};
        let st = state(9, 26, 198);
        let poison = KitItem { slot: 3, kind: 79, qty: 4, home_unwanted: true,
            home_limited: true, home_room: Some([4, 0, 4]), sell_shop: -1, need: -1,
            reduction: 0, ..recall(Place::Home) };
        let select = |kit: &[KitItem], revision, slots| {
            let mut inp = input(kit);
            inp.revision = revision;
            inp.limits.pack_slots = slots;
            let storing = Storing::default();
            let mut u = Utility::of(&st, kit, false);
            let mut out = step(&inp);
            (out.dispositions, out.freed, out.copies) = dispositions_by(&inp, &mut u, &st, storing);
            let (code, at, _, _) = first_executable_action(&[], &[], &out, &[], kit, 0, slots, 20);
            (code, at, out.freed)
        };
        let (code, at, freed) = select(&[poison], 15, 1);
        assert_eq!((code, at), (action::DROP_HOME, [7, 3, 4, 0]));
        assert_eq!(freed.home_slots, 1);
        assert_eq!(freed.gold, 0);
        assert_eq!(select(&[poison], 15, 0).0, action::LEAVE, "requires native pack room");
        assert_eq!(select(&[poison], 14, 1).0, action::LEAVE, "preserve the old replay decision");
        let carried = KitItem { slot: 1, place: Place::Pack, home_room: None, ..poison };
        let (code, at, freed) = select(&[carried], 15, 0);
        assert_eq!((code, at), (action::DROP, [-1, 1, 4, 0]));
        assert_eq!(freed.pack_slots, 1);
    }

    #[test]
    fn two_home_stacks_protect_the_carried_floor_and_release_the_rest() {
        let mut st = state(9, 26, 198);
        st.borg.trait_[BI_APHASE as usize] = 20;
        let pack = KitItem { home_room: None, protect_disposal: false, home_unwanted: false, home_limited: true, kind: 51, qty: 20, need: index(Need::Escape) as i8,
            reduction: UNIT, sell_shop: 4, proceeds: 0, ..recall(Place::Pack) };
        let home = KitItem { place: Place::Home, qty: 40, joins_pack: true, ..pack };
        let kit = [pack, home, KitItem { slot: 1, ..home }, KitItem { slot: 2, qty: 19, ..home }];
        let mut inp = input(&kit);
        let storing = Storing { keep_min: 6, home_stack_limit: 2, ..Storing::default() };
        inp.storing = Some(storing);
        inp.limits.pack_slots = 2;
        let mut u = Utility::of(&st, &kit, false);
        let (actions, freed, copies) = dispositions_by(&inp, &mut u, &st, storing);
        assert_eq!(actions, [Disposition::Sell, Disposition::Keep, Disposition::Keep, Disposition::Sell]);
        assert_eq!(copies[0], 14);
        assert_eq!(freed.pack_slots, 0, "the carried reserve keeps its slot");
        assert_eq!(freed.home_slots, 1);
        assert_eq!(freed.gold, 0, "the no-selling birth option still accepts excess");
        assert_eq!(saturated_home_kinds(&inp, &u, &actions, &copies), BTreeSet::from([51]));
    }

    #[test]
    fn a_capped_home_can_replenish_a_short_kit_and_leave_other_kinds_available() {
        let mut st = state(9, 26, 198);
        st.borg.trait_[BI_APHASE as usize] = 2;
        let phase = KitItem { home_room: None, protect_disposal: false, home_unwanted: false, home_limited: true, kind: 51, qty: 2,
            need: index(Need::Escape) as i8, reduction: UNIT, ..recall(Place::Pack) };
        let home = KitItem { place: Place::Home, qty: 6, home_room: Some([6, 4, 35]), ..phase };
        let food = KitItem { kind: 52, qty: 1, ..phase };
        let kit = [phase, home, KitItem { slot: 1, ..home }, food];
        let mut inp = input(&kit);
        let storing = Storing { keep_min: 6, home_stack_limit: 2, ..Storing::default() };
        inp.storing = Some(storing);
        inp.revision = 15;
        inp.limits.pack_slots = 0;
        inp.limits.weight = 20;
        let mut u = Utility::of(&st, &kit, false);
        u.home_discount = 0;
        u.space_weight = 0;
        let (actions, freed, copies) = dispositions_by(&inp, &mut u, &st, storing);
        assert_eq!(&actions[..3], &[Disposition::Keep, Disposition::Take, Disposition::Keep]);
        assert_eq!(copies[1], 4);
        assert_eq!(freed.weight, -20);
        assert_eq!(freed.home_slots, 0, "the two remaining copies still occupy the Home slot");
        assert_eq!(freed.pack_slots, 0, "all four copies join the carried stack");
        assert_eq!(saturated_home_kinds(&inp, &u, &actions, &copies), BTreeSet::from([51]));
        use crate::borg_town_record::{action, first_executable_action, step};
        let mut out = step(&inp);
        out.dispositions = actions;
        out.copies = copies;
        let (code, at, _, _) = first_executable_action(&[], &[], &out, &[], &kit, 0, 0, 20);
        assert_eq!((code, at[1], at[2]), (action::TAKE_HOME, i64::from(home.slot), 4));
    }

    #[test]
    fn topping_up_from_home_still_allows_the_surplus_to_be_sold() {
        use crate::borg_town_shadow::kit_just_bought;
        use crate::borg_store::BORG_HOME;
        use crate::headers::list_tvals_h::TV_SCROLL;

        let mut st = state(9, 26, 198);
        let mut item = crate::borg_item::blank_item();
        item.tval = TV_SCROLL as u8;
        item.sval = 12;
        let phase = KitItem { kind: 51, qty: 2, home_limited: true,
            need: index(Need::Escape) as i8, ..recall(Place::Pack) };
        let home = KitItem { place: Place::Home, home_room: Some([19, 19, 19]), qty: 19, ..phase };
        let storing = Storing { keep_min: 6, home_stack_limit: 2, ..Storing::default() };
        let solve = |st: &BorgState, kit: &[KitItem], revision| {
            let mut inp = input(kit);
            inp.revision = revision;
            inp.storing = Some(storing);
            let mut u = Utility::of(st, kit, false);
            u.home_discount = 0;
            u.space_weight = 0;
            dispositions_by(&inp, &mut u, st, storing)
        };
        let observe = |st: &BorgState, held, surplus| [
            KitItem { qty: held, just_bought: kit_just_bought(st, &item, Place::Pack, true), ..phase },
            KitItem { qty: surplus, just_bought: kit_just_bought(st, &item, Place::Home, false), ..home },
            KitItem { slot: 1, qty: 40, just_bought: kit_just_bought(st, &item, Place::Home, false), ..home },
            KitItem { slot: 2, qty: 40, just_bought: kit_just_bought(st, &item, Place::Home, false), ..home },
        ];

        st.borg.trait_[BI_APHASE as usize] = 2;
        let (actions, _, copies) = solve(&st, &observe(&st, 2, 19), 18);
        assert_eq!(actions, [Disposition::Keep, Disposition::Take, Disposition::Keep, Disposition::Keep]);
        assert_eq!(copies[1], 4);

        st.buy.bought_item_num = 1;
        st.buy.bought_item_store[0] = BORG_HOME as i32;
        st.buy.bought_item_tval[0] = i32::from(item.tval);
        st.buy.bought_item_sval[0] = i32::from(item.sval);
        assert!(kit_just_bought(&st, &item, Place::Pack, false), "uncapped keepers keep the usual protection");
        st.borg.trait_[BI_APHASE as usize] = 6;
        let kit = observe(&st, 6, 15);
        let (actions, freed, _) = solve(&st, &kit, 18);
        assert_eq!(actions, [Disposition::Keep, Disposition::Sell, Disposition::Keep, Disposition::Keep]);
        assert_eq!(freed.home_slots, 1);
        assert_eq!(solve(&st, &kit, 16).0, [Disposition::Keep; 4]);

        // The sale withdrawal itself does not replace the keeper history.
        st.borg.trait_[BI_APHASE as usize] = 21;
        let kit = [observe(&st, 21, 15)[0],
            KitItem { slot: 1, qty: 40, ..home }, KitItem { slot: 2, qty: 40, ..home }];
        let (actions, freed, copies) = solve(&st, &kit, 18);
        assert_eq!(actions, [Disposition::Sell, Disposition::Keep, Disposition::Keep]);
        assert_eq!(copies[0], 15, "the six-scroll reserve stays carried");
        assert_eq!(freed.pack_slots, 0);

        st.buy.bought_item_store[0] = 4;
        st.borg.trait_[BI_APHASE as usize] = 6;
        let kit = observe(&st, 6, 15);
        assert!(kit[0].just_bought, "actual shop purchases stay protected");
        let (actions, freed, _) = solve(&st, &kit, 18);
        assert_eq!(actions, [Disposition::Keep; 4], "do not withdraw stock the pack cannot sell");
        assert_eq!(freed.home_slots, 0);
        assert_eq!(solve(&st, &kit, 17).0[1], Disposition::Sell, "schema 17 keeps its recorded decision");

        st.buy.bought_item_num = 0;
        assert_eq!(solve(&st, &observe(&st, 6, 15), 18).0[1], Disposition::Sell,
            "cleanup resumes when the purchase history clears");
    }

    #[test]
    fn home_disposal_uses_native_room_even_at_the_acquisition_cap() {
        use crate::borg_town_capability::Model;
        let st = state(9, 26, 198);
        let pack = KitItem { home_unwanted: false, home_limited: true, protect_disposal: true, kind: 51, qty: 35,
            sell_shop: 4, proceeds: 0, ..recall(Place::Pack) };
        let home = KitItem { place: Place::Home, qty: 7, home_room: Some([7, 0, 0]), ..pack };
        let storing = Storing { home_stack_limit: 2, ..Storing::default() };
        for (room, expected) in [(Some([7, 0, 0]), Disposition::Sell),
            (Some([0, 0, 0]), Disposition::Keep), (None, Disposition::Keep)] {
            let kit = [pack, home, KitItem { slot: 1, ..home }, KitItem { slot: 2, qty: 5, home_room: room, ..home }];
            let mut inp = input(&kit);
            inp.limits.pack_slots = 1;
            let mut u = Utility::of(&st, &kit, false);
            u.capability = Some(Model { available: vec![[7, 2, 0]], packing: None,
                curves: Vec::new(), supplies: Vec::new() });
            assert_eq!(dispositions_by(&inp, &mut u, &st, storing).0[3], expected);
        }
    }

    #[test]
    fn protected_detection_surplus_waits_until_all_withdrawn_copies_can_overflow() {
        let st = state(9, 26, 198);
        let pack = KitItem { home_limited: true, protect_disposal: true, kind: 51,
            qty: 10, need: index(Need::Escape) as i8, ..recall(Place::Pack) };
        let home = KitItem { place: Place::Home, qty: 7, joins_pack: true, ..pack };
        let storing = Storing { home_stack_limit: 2, ..Storing::default() };
        for (held, acquire, expected) in [(10, 7, Disposition::Keep), (34, 1, Disposition::Keep),
            (35, 0, Disposition::Sell)] {
            let kit = [KitItem { qty: held, ..pack }, KitItem { home_room: Some([7, 7, acquire]), ..home },
                KitItem { slot: 1, qty: 40, ..home }, KitItem { slot: 2, qty: 40, ..home }];
            let mut inp = input(&kit);
            inp.revision = 19;
            inp.storing = Some(storing);
            let mut u = Utility::of(&st, &kit, false);
            let (actions, freed, _) = dispositions_by(&inp, &mut u, &st, storing);
            assert_eq!(actions, [Disposition::Keep, expected, Disposition::Keep, Disposition::Keep]);
            assert_eq!(freed.home_slots, i32::from(expected == Disposition::Sell));
            inp.revision = 18;
            let mut u = Utility::of(&st, &kit, false);
            assert_eq!(dispositions_by(&inp, &mut u, &st, storing).0[1], Disposition::Sell);
        }
    }

    #[test]
    fn surplus_disposal_with_a_full_pack_preserves_the_home_slot_until_empty() {
        use crate::borg_town_capability::Model;
        use crate::borg_town_record::{action, first_executable_action, step};
        let st = state(9, 26, 198);
        let pack = KitItem { home_limited: true, home_unwanted: false, protect_disposal: true, kind: 51, qty: 35,
            sell_shop: 4, need: index(Need::Escape) as i8, proceeds: 0, ..recall(Place::Pack) };
        let home = KitItem { place: Place::Home, qty: 40, home_room: Some([5, 5, 0]), ..pack };
        let storing = Storing { home_stack_limit: 2, ..Storing::default() };
        for (qty, expected, freed_slots) in [(7, 5, 0), (2, 2, 1)] {
            let kit = [pack, home, KitItem { slot: 1, ..home },
                KitItem { slot: 2, qty, joins_pack: qty <= 5, ..home }];
            let mut inp = input(&kit);
            inp.revision = 19;
            inp.storing = Some(storing);
            inp.limits.pack_slots = 0;
            inp.home_slots = 0;
            let mut u = Utility::of(&st, &kit, false);
            u.capability = Some(Model { available: vec![[7, 2, 0]], packing: None,
                curves: Vec::new(), supplies: Vec::new() });
            let mut out = step(&inp);
            (out.dispositions, out.freed, out.copies) = dispositions_by(&inp, &mut u, &st, storing);
            assert_eq!(out.dispositions, [Disposition::Keep, Disposition::Keep, Disposition::Keep, Disposition::Sell]);
            assert_eq!(out.freed.home_slots, freed_slots);
            assert_eq!(out.freed.pack_slots, 0);
            assert_eq!(out.freed.gold, 0);
            let (code, at, _, _) = first_executable_action(&[], &[], &out, &[], &kit, 0, 0, 20);
            assert_eq!((code, at), (action::SELL_HOME, [4, 2, expected, 0]));
            assert_eq!(saturated_home_kinds(&inp, &u, &out.dispositions, &out.copies), BTreeSet::from([51]));
            if qty == 7 {
                inp.revision = 15;
                assert_eq!(dispositions_by(&inp, &mut u, &st, storing).0[3], Disposition::Keep);
            }
        }
    }

    /// #859 (review F1): the plan's first action stores the surplus the
    /// disposition priced, and the whole stack when nothing is split.
    #[test]
    fn the_first_store_action_moves_the_surplus_not_the_stack() {
        use crate::borg_town_record::{action, first_action, step};
        let mut st = state(0, 1, 20);
        st.borg.trait_[BI_ACLW as usize] = 12;
        let potions = KitItem { slot: 3, kind: 50, qty: 12, weight: 4, need: index(Need::Healing) as i8,
            reduction: 20 * UNIT, ..recall(Place::Pack) };
        let first_store = |kit: &[KitItem], keep_min: i32| {
            let mut inp = input(kit);
            inp.limits.weight = 0;
            let mut out = step(&inp);
            let mut u = Utility::of(&st, kit, false);
            let (d, _, copies) = dispositions_by(&inp, &mut u, &st,
                Storing { only_overweight: true, room: Room { weight: 30, slots: 0 }, keep_min, ..Storing::default() });
            out.dispositions = d;
            out.copies = copies;
            first_action(&inp.candidates, &[], &out, &[], kit)
        };
        let kit = [potions];
        let (code, at, _, _) = first_store(&kit, 3);
        assert_eq!((code, at[1], at[2]), (action::STORE, 3, 9), "twelve copies, three kept: nine go");
        let (code, at, _, _) = first_store(&kit, 0);
        assert_eq!((code, at[2]), (action::STORE, 12), "keep_min 0: the whole stack");
        // An unknown flavour is never split: it is sold whole, as before.
        let unknown = [KitItem { need: -1, reduction: 0, unknown: true, unknown_flavour: true, ..potions }];
        let (code, at, _, _) = first_store(&unknown, 3);
        assert_eq!((code, at[2]), (action::SELL, 12), "an unknown flavour is not split");
    }

    /// #859: short of slots, whole stacks go until the room is free.
    #[test]
    fn the_departure_room_frees_slots_with_whole_stacks() {
        let st = state(0, 1, 20);
        let kit: Vec<KitItem> = (0..3).map(|i| KitItem { slot: i, kind: 60 + u32::from(i), qty: 5, weight: 5,
            need: index(Need::Food) as i8, ..recall(Place::Pack) }).collect();
        let mut inp = input(&kit);
        inp.limits.pack_slots = 0;
        let mut u = Utility::of(&st, &kit, false);
        let (_, freed, _) = dispositions_by(&inp, &mut u, &st,
            Storing { only_overweight: true, room: Room { weight: 0, slots: 2 }, keep_min: 3, ..Storing::default() });
        assert_eq!(freed.pack_slots, 2, "two stacks, not their surplus");
    }

    /// #859: a withdrawal from the Home leaves the departure room free.
    #[test]
    fn a_withdrawal_leaves_the_departure_room_free() {
        let mut st = state(29, 20, 200);
        st.core.cfg[BORG_TOWN_TARGET_DEPTH as usize] = 30;
        let kit = [recall(Place::Home)];
        let mut inp = input(&kit);
        inp.limits.weight = 40;
        let mut u = Utility::of(&st, &kit, false);
        assert_eq!(dispositions_by(&inp, &mut u, &st, Storing::default()).0, [Disposition::Take]);
        let mut u = Utility::of(&st, &kit, false);
        assert_eq!(dispositions_by(&inp, &mut u, &st,
            Storing { room: Room { weight: 40, slots: 0 }, ..Storing::default() }).0, [Disposition::Keep]);
    }

    /// #859: storing makes room for the Recall the reserve still needs on
    /// top of the departure room (a Borg with 20,000 gold at its allowance
    /// could not take a 0.5 lb scroll). Off with the room.
    #[test]
    fn storing_makes_room_for_the_recall_reserve_too() {
        let st = state(0, 1, 20);
        let recall = crate::borg_town_capability::NAMES.iter().position(|&n| n == "Recall").unwrap();
        let mut u = Utility::of(&st, &[], false);
        u.capability = Some(crate::borg_town_capability::Model { available: Vec::new(), packing: None,
            curves: (0..crate::borg_town_capability::NAMES.len()).map(|i| [i as i64, 0, UNIT, 0, 0]).collect(),
            supplies: vec![[5, 3, recall as i64, UNIT]] });
        u.recall_reserve = 1;
        let mut inp = input(&[]);
        inp.candidates = vec![crate::borg_town_stock::Candidate { shop: 5, slot: 3, kind: 1, tval: 0, sval: 0, price: 125,
            qty: 4, weight: 5, slots_cost: 1, stack_count: 0, class: 0 }];
        assert_eq!(storing_target(&inp, &u, Room { weight: 30, slots: 2 }), Room { weight: 35, slots: 3 });
        assert_eq!(storing_target(&inp, &u, Room::default()), Room::default());
    }

    /// #1151: the observed kit of a CL17 Gnome Mage at -12 speed (game 14,
    /// turn 9,221,029; borg-cemetery scenario
    /// mage-overweight-cursed-kit-9221204): 34 spare Wooden Torches of 5,000
    /// turns against a 20,000-turn reserve, a cursed Lochaber Axe {cursed,
    /// ??}, a Dagger {??} with no known curse, three Phase Door, a full Home,
    /// and 99.3 lb over the departure room with the shipped storing
    /// settings. Schema 20 keeps four torches and lets thirty go, and lets
    /// the axe go; the dagger, unknown and not known cursed, stays. Schema 19
    /// (the recorded visit) kept everything.
    #[test]
    fn an_overweight_mage_unloads_surplus_torches_and_a_cursed_spare() {
        use crate::borg_town_capability::{Model, NAMES, PACK};
        let light = NAMES.iter().position(|&n| n == "light turns").unwrap();
        let st = state(5, 17, 105);
        let torch_turns = 5000 * UNIT;
        let torches = KitItem { slot: 6, kind: 300, qty: 34, weight: 22, need: -1, reduction: 0,
            sell_shop: 0, fuel: torch_turns, ..recall(Place::Pack) };
        let axe = KitItem { slot: 20, kind: 301, qty: 1, weight: 250, need: -1, reduction: 0,
            sell_shop: -1, unknown: true, cursed_spare: true, ..recall(Place::Pack) };
        let dagger = KitItem { slot: 15, kind: 302, qty: 1, weight: 12, need: -1, reduction: 0,
            sell_shop: -1, unknown: true, ..recall(Place::Pack) };
        let phase = KitItem { slot: 5, kind: 303, qty: 3, weight: 5, need: index(Need::Escape) as i8,
            ..recall(Place::Pack) };
        let kit = [torches, axe, dagger, phase];
        let model = || {
            let mut curves: Vec<[i64; 5]> = (0..NAMES.len()).map(|i| [i as i64, 0, UNIT, 0, 0]).collect();
            curves[light] = [light as i64, 100 * UNIT, -4 * torch_turns, 34 * torch_turns, 0];
            Model { available: Vec::new(), packing: None, curves, supplies: vec![[PACK, 6, light as i64, torch_turns]] }
        };
        let shipped = Storing { only_overweight: true, room: Room { weight: 50, slots: 2 }, keep_min: 3,
            home_stack_limit: 2, ..Storing::default() };
        let solve = |revision: i64| {
            let mut inp = input(&kit);
            inp.revision = revision;
            inp.home_slots = 0;
            inp.limits.weight = -943;
            let mut u = Utility::of(&st, &kit, false);
            u.capability = Some(model());
            dispositions_by(&inp, &mut u, &st, shipped)
        };
        let (d, freed, copies) = solve(20);
        assert_eq!(d[0], Disposition::Sell, "the surplus torches go");
        assert_eq!(copies[0], 30, "four torches cover the 20,000-turn reserve");
        assert_eq!(d[1], Disposition::Drop, "the cursed spare goes");
        assert_eq!(d[2], Disposition::Keep, "an unknown item with no known curse stays");
        assert_eq!(freed.weight, 30 * 22 + 250);
        assert_eq!(freed.pack_slots, 1, "the torch stack keeps its slot");
        let (d, freed, _) = solve(19);
        assert_eq!(&d[..3], &[Disposition::Keep; 3], "the recorded visit kept the kit");
        assert_eq!(freed.weight, 0);
        // Within the allowance (only the departure room short) with a full
        // Home: no sale and no drop, the cursed spare is not junk yet.
        let mut inp = input(&kit);
        inp.revision = 20;
        inp.home_slots = 0;
        inp.limits.weight = 20;
        let mut u = Utility::of(&st, &kit, false);
        u.capability = Some(model());
        let (d, freed, _) = dispositions_by(&inp, &mut u, &st, shipped);
        assert_eq!(&d[..3], &[Disposition::Keep; 3], "nothing is sold or dropped within the allowance");
        assert_eq!(freed.weight, 0);
        // With room at Home, the surplus torches are stored (the cursed
        // spare is worth nothing there and goes).
        inp.home_slots = 5;
        inp.limits.weight = -943;
        let mut u = Utility::of(&st, &kit, false);
        u.capability = Some(model());
        let (d, _, copies) = dispositions_by(&inp, &mut u, &st, shipped);
        assert_eq!((d[0], copies[0]), (Disposition::Store, 30));
        assert_eq!(d[1], Disposition::Drop);
    }

    #[test]
    fn class_book_reserve_keeps_one_copy_and_allows_spares_to_be_sold() {
        let st = state(5, 17, 105);
        let book = KitItem { keep_last_book: true, need: -1, reduction: 0,
            proceeds: 100, ..recall(Place::Pack) };
        let solve = |kit: &[KitItem]| {
            let mut inp = input(kit);
            inp.home_slots = 0;
            let mut u = Utility::of(&st, kit, false);
            dispositions_with(&inp, &mut u, &st, true)
        };
        assert_eq!(solve(&[book]).0, [Disposition::Keep]);
        let (dispositions, freed, copies) = solve(&[KitItem { qty: 2, ..book }]);
        assert_eq!(dispositions, [Disposition::Sell]);
        assert_eq!(copies, [1]);
        assert_eq!(freed.gold, 50);
        let home = KitItem { place: Place::Home, ..book };
        let spare = KitItem { keep_last_book: false, ..book };
        assert_eq!(solve(&[spare, home]).0, [Disposition::Sell, Disposition::Keep]);
        assert_eq!(solve(&[KitItem { qty: 2, sell_shop: -1, ..book }]).0, [Disposition::Keep]);
    }

    /// #1151 (review of #1155): selling for weight keeps every reserve. An
    /// overweight Borg with six Phase Door against a reserve of six, a full
    /// Home and `town_store_keep_min` 3 keeps all six (the excess rule sold
    /// three). Torches no shop buys drop only the copies past the light
    /// reserve, and an unsplittable stack with no buyer is not dropped
    /// for weight.
    #[test]
    fn selling_for_weight_keeps_every_reserve() {
        use crate::borg_town_capability::{Model, NAMES, PACK};
        let col = |name: &str| NAMES.iter().position(|&n| n == name).unwrap();
        let (phase, reserve, light) = (col("phase"), 61, col("light turns"));
        let st = state(5, 17, 105);
        let scrolls = KitItem { slot: 5, kind: 303, qty: 6, weight: 5, need: -1, reduction: 0,
            sell_shop: 5, ..recall(Place::Pack) };
        let model = |reserve_units: i64| {
            let mut curves: Vec<[i64; 5]> = (0..NAMES.len()).map(|i| [i as i64, 0, UNIT, 0, 0]).collect();
            curves[phase] = [phase as i64, 100 * UNIT, 10 * UNIT, 6 * UNIT, 0];
            curves[reserve] = [reserve as i64, 100 * UNIT, -reserve_units, 6 * UNIT, 0];
            Model { available: Vec::new(), packing: None, curves,
                supplies: vec![[PACK, 5, phase as i64, UNIT], [PACK, 5, reserve as i64, UNIT]] }
        };
        let shipped = Storing { only_overweight: true, room: Room { weight: 50, slots: 2 }, keep_min: 3,
            home_stack_limit: 2, ..Storing::default() };
        let solve = |kit: &[KitItem], m: Model| {
            let mut inp = input(kit);
            inp.revision = 20;
            inp.home_slots = 0;
            inp.limits.weight = -943;
            let mut u = Utility::of(&st, kit, false);
            u.capability = Some(m);
            dispositions_by(&inp, &mut u, &st, shipped)
        };
        let kit = [scrolls];
        let (d, freed, _) = solve(&kit, model(6 * UNIT));
        assert_eq!((d[0], freed.weight), (Disposition::Keep, 0), "the reserve of six stays");
        // A reserve of three: the floor covers it, so the three past it are excess.
        let (d, _, copies) = solve(&kit, model(3 * UNIT));
        assert_eq!((d[0], copies[0]), (Disposition::Sell, 3));
        // No buyer for a stack that the executor drops whole: not dropped for weight.
        let unsold = [KitItem { sell_shop: -1, ..scrolls }];
        let (d, _, _) = solve(&unsold, model(3 * UNIT));
        assert_ne!(d[0], Disposition::Drop, "a split drop would lose the floor");
        // Torches no shop buys: the copies past the light reserve are dropped.
        let torch_turns = 5000 * UNIT;
        let torches = [KitItem { slot: 6, kind: 300, qty: 34, weight: 22, need: -1, reduction: 0,
            sell_shop: -1, fuel: torch_turns, ..recall(Place::Pack) }];
        let mut curves: Vec<[i64; 5]> = (0..NAMES.len()).map(|i| [i as i64, 0, UNIT, 0, 0]).collect();
        curves[light] = [light as i64, 100 * UNIT, -4 * torch_turns, 34 * torch_turns, 0];
        let m = Model { available: Vec::new(), packing: None, curves, supplies: vec![[PACK, 6, light as i64, torch_turns]] };
        let (d, _, copies) = solve(&torches, m.clone());
        assert_eq!((d[0], copies[0]), (Disposition::Drop, 30));
        // Re-review of #1155: 1 lb over the allowance, one 2.2 lb torch covers
        // the deficit; the other 29 past the reserve stay.
        let mut inp = input(&torches);
        inp.revision = 20;
        inp.home_slots = 0;
        inp.limits.weight = -10;
        let mut u = Utility::of(&st, &torches, false);
        u.capability = Some(m);
        let (d, freed, copies) = dispositions_by(&inp, &mut u, &st, shipped);
        assert_eq!((d[0], copies[0], freed.weight), (Disposition::Drop, 1, 22));
        // Re-review of #1155: the reserve is the floor before the move is
        // priced. With room at Home, six Phase Door against a reserve of six
        // are not stored down to `town_store_keep_min`.
        let mut inp = input(&kit);
        inp.revision = 20;
        inp.home_slots = 5;
        inp.limits.weight = -943;
        let mut u = Utility::of(&st, &kit, false);
        u.capability = Some(model(6 * UNIT));
        let (d, freed, _) = dispositions_by(&inp, &mut u, &st, shipped);
        assert_eq!((d[0], freed.weight), (Disposition::Keep, 0), "the reserve is not stored");
        // Nine against a reserve of six, full Home: the three past it are
        // true surplus, sold for weight.
        let nine = [KitItem { qty: 9, ..scrolls }];
        let mut nine_model = model(6 * UNIT);
        nine_model.curves[reserve][3] = 9 * UNIT;
        nine_model.curves[phase][3] = 9 * UNIT;
        let (d, _, copies) = solve(&nine, nine_model);
        assert_eq!((d[0], copies[0]), (Disposition::Sell, 3));
        // Second re-review of #1155: seven Phase Door against a reserve of
        // six, no buyer, a full Home, within the allowance: the executor
        // would drop all seven, so the surplus copy is not planned as a drop.
        let seven = [KitItem { qty: 7, sell_shop: -1, ..scrolls }];
        let mut seven_model = model(6 * UNIT);
        seven_model.curves[reserve][3] = 7 * UNIT;
        seven_model.curves[phase][3] = 7 * UNIT;
        seven_model.curves[phase][1] = 0; // a copy past the reserve worth less than its burden
        for limit in [-943, 100] {
            let mut inp = input(&seven);
            inp.revision = 20;
            inp.home_slots = 0;
            inp.limits.weight = limit;
            let mut u = Utility::of(&st, &seven, false);
            u.capability = Some(seven_model.clone());
            let (d, _, _) = dispositions_by(&inp, &mut u, &st, shipped);
            assert_eq!(d[0], Disposition::Keep, "weight room {limit}");
        }
    }

    /// #859: the first Recall is urgent once the next trip reaches the dive
    /// gate and none is carried; never with the gate off.
    #[test]
    fn the_first_recall_is_urgent_at_the_dive_gate() {
        assert!(recall_gate_reached(5, 5, 0));
        assert!(recall_gate_reached(9, 5, 0));
        assert!(!recall_gate_reached(4, 5, 0), "the trip stays above the gate");
        assert!(!recall_gate_reached(5, 5, 1), "one is carried");
        assert!(!recall_gate_reached(5, 0, 0), "no gate");
    }

}

/// borg-reborn-rs (#984): the trip depth from which `recall_urgent` makes the
/// first Word of Recall urgent: `town_recall_urgent_depth`, or the dive gate
/// (`prepared_recall_depth`, #859) when it is 0. The dive gate itself is not
/// moved. 312081895 left town at 235 gold for a DL2 trip with none: above
/// the dive gate, the floor (`buy_recall_from_gold`) left no copy to buy,
/// and with the floor off the solve still spent the gold on other supplies.
fn urgent_recall_gate(urgent_depth: i32, dive_gate: i32) -> i32 {
    if urgent_depth > 0 { urgent_depth } else { dive_gate }
}

#[cfg(test)]
mod urgent_recall_depth_tests {
    use super::{recall_gate_reached, urgent_recall_gate};

    #[test]
    fn the_urgent_recall_depth_moves_only_the_urgent_gate() {
        assert_eq!(urgent_recall_gate(0, 5), 5, "0: the dive gate, as #859");
        assert_eq!(urgent_recall_gate(1, 5), 1);
        assert!(!recall_gate_reached(2, urgent_recall_gate(0, 5), 0), "today: a DL2 trip stays above the gate");
        assert!(recall_gate_reached(2, urgent_recall_gate(1, 5), 0), "from DL1: a DL2 trip with none carried (312081895)");
        assert!(!recall_gate_reached(2, urgent_recall_gate(1, 5), 1), "one is carried");
        assert!(!recall_gate_reached(0, urgent_recall_gate(1, 5), 0), "no trip into the dungeon");
    }
}
