//! Phase 2 of the town inventory redesign (borg-rs#36): the budget, the
//! constraints the solver respects.
//!
//! * **the urgent needs** - bought first, before anything else (owner
//!   decision, 2026-11-14: there are no urgent needs; an urgent need is one the
//!   character has **now** with **no other way** to meet it - see
//!   [`urgent_needs`]). The urgent pass picks the cheapest set of wares
//!   that meets as many of them as the purse allows: the most met first,
//!   then the least gold, then the fewest wares, then the earliest wares.
//!   It is exact (at most four urgent needs, so at most fifteen distinct
//!   ways a ware can serve them). An urgent need no shop serves, or the
//!   purse cannot pay for, is recorded as unmet and does not block the
//!   visit. Urgent purchases are not priced, so the caution cap does not
//!   apply to them;
//! * **gold** - the purse minus what the urgent needs cost, minus the gold
//!   reserve (borg-reborn-rs #493: the larger of `town_gold_reserve` and
//!   `town_gold_reserve_percent` of the purse; the urgent needs may use it).
//!   The reserve bounds the purse at arrival; the proceeds of this visit's
//!   sales join the gold at the purchases stage (`borg_town_record`), on top
//!   of it;
//! * **pack slots** - the pack's capacity (`pack_size`, 23, less the slots
//!   the quiver takes) minus one slot **per distinct stack** carried (S3),
//!   minus the slots the urgent purchases open;
//! * **Home slots** - the Home's capacity (`store_inven_max`, 24) minus its
//!   stacks;
//! * **weight** - the carry allowance minus the carried weight, minus the
//!   urgent purchases. The allowance is half of `BI_CARRY`, the point past
//!   which `borg_trait` charges the encumbrance against the speed.
//!
//! The slot, Home and weight figures are signed: a urgent purchase can push
//! one below zero, which says the dispositions (phase 4) have to free room
//! before the urgent needs fit. The urgent pass itself is bounded by the gold
//! alone.
//!
//! Nothing in production calls this module yet; every public item carries
//! a self-removing `expect(dead_code)` marker, as phase 1's do.

use crate::borg_headers::borg_trait_h::{BI_CARRY, BI_GOLD, BI_WEIGHT};
use crate::borg_town_stock::{Candidate, Stock};
use crate::borg_town_usefulness::{Need, Trip};
use crate::state::BorgState;

/// The raw numbers the budget is computed from.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Facts {
    /// `BI_GOLD`.
    pub gold: i64,
    /// `pack_size - BI_QUIVER_SLOTS`: the slots a stack can occupy.
    pub pack_capacity: i32,
    /// `store_inven_max`: the Home's slots.
    pub home_capacity: i32,
    /// `BI_CARRY`: the weight limit; half of it is the allowance.
    pub carry: i32,
    /// `BI_WEIGHT`: the weight carried now, equipment included.
    pub weight: i32,
}

impl Facts {
    /// The facts of the current state.
    pub fn of(st: &BorgState) -> Self {
        let z = crate::engine::z_info();
        Facts {
            gold: i64::from(st.borg.tr(BI_GOLD)),
            pack_capacity: i32::from(z.pack_size) - crate::borg_town_packing::reserved(st),
            home_capacity: i32::from(z.store_inven_max),
            carry: st.borg.tr(BI_CARRY),
            weight: st.borg.tr(BI_WEIGHT),
        }
    }

    /// The weight that can still be carried before the encumbrance
    /// starts: `BI_CARRY / 2 - BI_WEIGHT`.
    pub fn allowance(&self) -> i64 {
        i64::from(self.carry / 2) - i64::from(self.weight)
    }
}

/// One urgent purchase: one copy of a candidate, and the urgent needs it
/// discharges.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UrgentPurchase {
    /// Index into [`Stock::candidates`].
    pub candidate: usize,
    /// The urgent needs this copy discharges, in enum order.
    pub needs: Vec<Need>,
}

/// What the solver may spend, after the urgent needs.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Budget {
    /// The purse minus the urgent needs' cost and the gold reserve (#493),
    /// never below zero.
    pub gold: i64,
    /// Gold excluded from Black Market purchases.
    pub black_market_reserve: i64,
    pub black_market_percentage: i64,
    /// Free pack slots after the carried stacks and the urgent needs.
    pub pack_slots: i32,
    /// Free Home slots.
    pub home_slots: i32,
    /// The carry allowance left after the urgent needs.
    pub weight: i64,
    /// Bought first, never traded; one copy each.
    pub urgent: Vec<UrgentPurchase>,
    /// The urgent needs the visit cannot meet: no ware serves them, or the
    /// purse cannot pay. Recorded for #37's Stay rules.
    pub unmet: Vec<Need>,
}

/// The urgent pass: for the urgent needs `need`, the purchase set chosen
/// by (the urgent Word of Recall met, most urgent needs met, least gold,
/// fewest wares, earliest wares). Returns the chosen candidate indices and
/// the bit mask of `need` they cover.
///
/// borg-reborn-rs (#956, P7 review): `Need::Trip` is urgent only with
/// `town_recall_urgent` at the dive gate, where the Recall is "bought first,
/// before anything is weighed" (harness/borg.txt). Counting needs alone, a
/// purse of 140 with a 125-gold scroll and a 20-gold ration bought the
/// ration and left the Recall 5 gold short.
fn urgent_pass(
    cands: &[Candidate],
    need: &[Need],
    gold: i64,
    serves: &impl Fn(&Candidate) -> Vec<Need>,
) -> (Vec<usize>, u32) {
    // The cheapest ware for each distinct way of serving the urgent needs
    // (ties: the earliest candidate). Any optimal set can use these
    // representatives instead of its own wares at no extra cost.
    // `serves` is asked once per candidate: in production it reaches the
    // engine's effect tables, and the subset loop below must not.
    let full = (1u32 << need.len()) - 1;
    let trip = need.iter().position(|n| *n == Need::Trip).map_or(0u32, |b| 1 << b);
    let mut best: Vec<Option<(usize, u32)>> = vec![None; (full + 1) as usize];
    for (i, c) in cands.iter().enumerate() {
        if c.qty < 1 {
            continue;
        }
        let served = serves(c);
        let mask = need
            .iter()
            .enumerate()
            .filter(|(_, n)| served.contains(n))
            .fold(0u32, |m, (b, _)| m | (1 << b));
        if mask == 0 {
            continue;
        }
        let slot = &mut best[mask as usize];
        if slot.is_none_or(|(j, _)| c.price < cands[j].price) {
            *slot = Some((i, mask));
        }
    }
    let reps: Vec<(usize, u32)> = best.into_iter().flatten().collect();

    // Every subset of the representatives (at most 2^15).
    let mut choice: Option<(u32, i64, Vec<usize>)> = None;
    for sub in 0u32..(1u32 << reps.len()) {
        let chosen = (0..reps.len()).filter(|b| sub & (1 << b) != 0).map(|b| reps[b]);
        let covered = chosen.clone().fold(0u32, |m, (_, mask)| m | mask);
        let mut picked: Vec<usize> = chosen.map(|(i, _)| i).collect();
        picked.sort_unstable();
        let cost: i64 = picked.iter().map(|&i| i64::from(cands[i].price)).sum();
        if cost > gold {
            continue;
        }
        let better = match &choice {
            None => true,
            Some((cov, c, p)) => {
                let key = (std::cmp::Reverse(covered & trip != 0), std::cmp::Reverse(covered.count_ones()), cost, picked.len());
                let old = (std::cmp::Reverse(cov & trip != 0), std::cmp::Reverse(cov.count_ones()), *c, p.len());
                key < old || (key == old && picked < *p)
            }
        };
        if better {
            choice = Some((covered, cost, picked));
        }
    }
    let (covered, _, picked) = choice.unwrap_or((0, 0, Vec::new()));
    (picked, covered & full)
}

impl Budget {
    /// The budget from its facts, the stock and the urgent needs.
    /// `serves(c)` names the urgent needs one copy of `c` discharges; in
    /// production it is `needs_served`.
    pub fn build(
        facts: &Facts,
        stock: &Stock,
        urgent_needs: &[Need],
        serves: impl Fn(&Candidate) -> Vec<Need>,
    ) -> Self {
        let cands = &stock.candidates;
        let (picked, covered) = urgent_pass(cands, urgent_needs, facts.gold.max(0), &serves);

        let mut urgent: Vec<UrgentPurchase> = picked
            .iter()
            .map(|&i| {
                let served = serves(&cands[i]);
                UrgentPurchase {
                    candidate: i,
                    needs: urgent_needs
                        .iter()
                        .copied()
                        .filter(|n| served.contains(n))
                        .collect(),
                }
            })
            .collect();
        // borg-reborn-rs (#956, review of #961): the live selector buys the
        // urgent purchases in this order, so the urgent Recall goes first
        // and a cheaper urgent ware cannot take the last slot or the weight
        // it needs (stable: the rest keep the candidate order).
        urgent.sort_by_key(|f| !f.needs.contains(&Need::Trip));
        let unmet = urgent_needs
            .iter()
            .enumerate()
            .filter(|(b, _)| covered & (1 << b) == 0)
            .map(|(_, &n)| n)
            .collect();

        // The urgent purchases' slots: one per distinct stack class that
        // no pack stack absorbs.
        let mut opened: Vec<u16> = picked
            .iter()
            .map(|&i| &cands[i])
            .filter(|c| c.slots_cost != 0)
            .map(|c| c.class)
            .collect();
        opened.sort_unstable();
        opened.dedup();

        let cost: i64 = picked.iter().map(|&i| i64::from(cands[i].price)).sum();
        let weight: i64 = picked.iter().map(|&i| i64::from(cands[i].weight)).sum();

        Budget {
            gold: facts.gold - cost,
            black_market_reserve: 0,
            black_market_percentage: -1,
            pack_slots: facts.pack_capacity - stock.pack_slots_used() - opened.len() as i32,
            home_slots: facts.home_capacity - stock.home_slots_used(),
            weight: facts.allowance() - weight,
            urgent,
            unmet,
        }
    }

    /// borg-reborn-rs (#493): [`Budget::build`] with the gold reserve. The
    /// urgent needs are bought from the whole purse; everything else from what
    /// is left above `reserve`.
    pub fn build_with_reserve(
        facts: &Facts,
        stock: &Stock,
        urgent_needs: &[Need],
        reserve: i64,
        serves: impl Fn(&Candidate) -> Vec<Need>,
    ) -> Self {
        let mut budget = Self::build(facts, stock, urgent_needs, serves);
        budget.gold = spendable(facts.gold, facts.gold - budget.gold, reserve);
        budget
    }

    /// The budget of the current state over a stock built from it, less the
    /// gold reserve (#493); the trip carries what the urgent needs read
    /// beyond the traits (the curing spell).
    pub fn of(st: &BorgState, stock: &Stock, trip: &Trip) -> Self {
        let facts = Facts::of(st);
        let reserve = gold_reserve(
            facts.gold,
            i64::from(crate::params::value(crate::params::Param::town_gold_reserve)),
            i64::from(crate::params::value(crate::params::Param::town_gold_reserve_percent)),
        );
        let black_market_reserve = i64::from(st.cfg(crate::borg_headers::borg_h::BORG_SUM_EXCLUDED_FROM_BLACK_MARKET)).max(0);
        let percentage = black_market_percentage(st, facts.gold);
        let market_gold = black_market_budget(facts.gold, black_market_reserve, percentage, 0);
        let recall_urgent = crate::borg_town_utility::recall_urgent(st);
        let mut budget = Self::build_with_reserve(&facts, stock, &crate::borg_town_utility::urgent_needs(st, trip), reserve, |c| {
            if (black_market_reserve > 0 || percentage >= 0) && c.is_black_market() && i64::from(c.price) > market_gold {
                return Vec::new();
            }
            let ware = &st.store.shops[usize::from(c.shop)].ware[usize::from(c.slot)];
            let mut served = crate::borg_town_stock::needs_served(st, ware);
            // #859: a Word of Recall serves the urgent Recall need.
            if recall_urgent && is_recall_scroll(st, ware.tval, ware.sval) { served.push(Need::Trip); }
            served
        });
        budget.black_market_reserve = black_market_reserve;
        budget.black_market_percentage = percentage;
        budget
    }
}

pub(crate) fn is_recall_scroll(st: &BorgState, tval: u8, sval: u8) -> bool {
    u32::from(tval) == crate::headers::list_tvals_h::TV_SCROLL
        && i32::from(sval) == st.item_val.sv_scroll_word_of_recall
}

/// borg-reborn-rs (#1103, owner: "keep and buy whenever it finds on the black
/// market up from a certain budget"; "flat 6000 starting with a level negative
/// delta of 100"): the purse the Black Market opens at for a character level,
/// `max(0, town_black_market_min_gold - town_black_market_level_delta x level)`.
/// 0 when `town_black_market_min_gold` is 0 (no threshold).
pub(crate) fn black_market_threshold(min_gold: i64, delta: i64, level: i64) -> i64 {
    if min_gold <= 0 { return 0; }
    (min_gold - delta.max(0) * level.max(0)).max(0)
}

/// The Black Market's spending share (`borg_black_market_max_percentage`) at
/// this purse: 0 below the threshold (nothing is bought there, the urgent
/// needs included, as for any purse the cap leaves nothing of), the configured
/// share above it. The planner, its record and the live purchase all read it
/// here, so a replay sees what the game saw.
pub(crate) fn black_market_percentage(st: &BorgState, gold: i64) -> i64 {
    use crate::params::{value, Param};
    black_market_share(
        i64::from(st.cfg(crate::borg_headers::borg_h::BORG_BLACK_MARKET_MAX_PERCENTAGE)), gold,
        i64::from(st.borg.tr(crate::borg_headers::borg_trait_h::BI_CLEVEL)),
        i64::from(value(Param::town_black_market_min_gold)), i64::from(value(Param::town_black_market_level_delta)))
}

pub(crate) fn black_market_share(percentage: i64, gold: i64, level: i64, min_gold: i64, delta: i64) -> i64 {
    if gold < black_market_threshold(min_gold, delta, level) { 0 } else { percentage }
}

pub(crate) fn black_market_budget(gold: i64, reserve: i64, percentage: i64, legacy_share: i64) -> i64 {
    let eligible = gold.saturating_sub(reserve.max(0)).max(0);
    if percentage < 0 {
        eligible / legacy_share.max(1)
    } else {
        (i128::from(eligible) * i128::from(percentage.min(100)) / 100) as i64
    }
}

/// borg-reborn-rs (#493): the gold never spent outside the urgent needs: the
/// larger of `town_gold_reserve` and `town_gold_reserve_percent` of the purse,
/// at most the purse.
pub fn gold_reserve(purse: i64, fixed: i64, percent: i64) -> i64 {
    fixed.max(purse.max(0) * percent / 100).clamp(0, purse.max(0))
}

/// The budget's gold: the purse less the urgent purchases' cost and the
/// reserve, never below zero (the urgent purchases may dip into the reserve).
pub fn spendable(purse: i64, urgent_cost: i64, reserve: i64) -> i64 {
    (purse - urgent_cost - reserve).max(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::borg_headers::borg_item_h::borg_item;
    use crate::borg_town_stock::tests::{build, item, town};

    // Test kinds: the tval doubles as the kind.
    const TORCH: u32 = 1;
    const RATION: u32 = 2;
    const CLW: u32 = 3;
    const CURE_POISON: u32 = 4;
    const PHASE: u32 = 5;
    /// A food that also cures poison.
    const FOOD_POISON: u32 = 6;

    fn serves(c: &Candidate) -> Vec<Need> {
        match c.kind {
            TORCH => vec![Need::Light],
            RATION => vec![Need::Food],
            CLW => vec![Need::Healing],
            CURE_POISON => vec![Need::Poison],
            FOOD_POISON => vec![Need::Food, Need::Poison],
            _ => vec![],
        }
    }

    fn facts(gold: i64) -> Facts {
        Facts {
            gold,
            pack_capacity: 23,
            home_capacity: 24,
            carry: 2000,
            weight: 300,
        }
    }

    /// The general store sells torches and rations, the alchemist cures.
    fn general_and_alchemist() -> Vec<Vec<borg_item>> {
        town([
            &[item(TORCH, 5, 2), item(RATION, 20, 4)],
            &[],
            &[],
            &[],
            &[item(CLW, 5, 20), item(CURE_POISON, 3, 30)],
            &[],
            &[],
            &[],
        ])
    }

    /// The contract (S3): 23 pack slots, counted per distinct stack and
    /// not per copy, less the quiver's slots; a ware that joins a carried
    /// stack costs no slot, one that does not costs one.
    #[test]
    fn pack_slots_are_counted_per_distinct_stack() {
        let t = general_and_alchemist();
        let mut variant = item(PHASE, 1, 0);
        variant.to_h = 1;
        // 40 copies in five stacks of three kinds: five slots, not 40
        let pack = [item(TORCH, 3, 0), item(RATION, 30, 0), item(PHASE, 5, 0), variant, item(CLW, 1, 0)];
        let s = build(&t, &pack);
        assert_eq!(s.pack_slots_used(), 5);
        let b = Budget::build(&facts(1000), &s, &[], serves);
        assert_eq!(b.pack_slots, 23 - 5);
        // the quiver's slots come off the capacity
        let mut f = facts(1000);
        f.pack_capacity = 23 - 2;
        assert_eq!(Budget::build(&f, &s, &[], serves).pack_slots, 16);
        // an urgent need bought into a carried stack opens no slot ...
        let b = Budget::build(&facts(1000), &s, &[Need::Poison], serves);
        assert_eq!(b.pack_slots, 18 - 1, "the cure opens a new stack");
        let s = build(&t, &pack[..4]);
        let b = Budget::build(&facts(1000), &s, &[Need::Light], serves);
        assert_eq!(b.pack_slots, 23 - 4, "the torch joins the carried torches");
        // ... and an empty pack has all 23
        let s = build(&t, &[]);
        assert_eq!(Budget::build(&facts(0), &s, &[], serves).pack_slots, 23);
    }

    /// With nothing unmet the budget is the purse, the free slots and the
    /// allowance: half the carry limit less the carried weight.
    #[test]
    fn the_budget_without_urgent_needs() {
        let mut t = general_and_alchemist();
        t[7][0] = item(PHASE, 3, 0);
        t[7][1] = item(TORCH, 1, 0);
        let s = build(&t, &[item(RATION, 4, 0)]);
        let b = Budget::build(&facts(500), &s, &[], serves);
        assert_eq!(b.gold, 500);
        assert_eq!(b.pack_slots, 22);
        assert_eq!(b.home_slots, 22);
        assert_eq!(b.weight, 1000 - 300);
        assert!(b.urgent.is_empty());
        assert!(b.unmet.is_empty());
    }

    /// The urgent needs come off the budget first: their gold, their slots and
    /// their weight.
    #[test]
    fn the_urgent_needs_come_first() {
        let s = build(&general_and_alchemist(), &[]);
        let unmet = [Need::Light, Need::Food, Need::Healing, Need::Poison];
        let b = Budget::build(&facts(100), &s, &unmet, serves);
        let bought: Vec<u32> = b.urgent.iter().map(|f| s.candidates[f.candidate].kind).collect();
        assert_eq!(bought, [TORCH, RATION, CLW, CURE_POISON]);
        assert_eq!(b.gold, 100 - (2 + 4 + 20 + 30));
        assert_eq!(b.pack_slots, 23 - 4);
        assert_eq!(b.weight, 700 - 40);
        assert!(b.unmet.is_empty());
    }

    /// A ware that discharges two urgent needs at once beats two that discharge
    /// one each when it is cheaper.
    #[test]
    fn the_urgent_pass_is_exact() {
        let t = town([
            &[item(RATION, 5, 4), item(FOOD_POISON, 1, 10)],
            &[],
            &[],
            &[],
            &[item(CURE_POISON, 1, 30)],
            &[],
            &[],
            &[],
        ]);
        let s = build(&t, &[]);
        let b = Budget::build(&facts(100), &s, &[Need::Food, Need::Poison], serves);
        assert_eq!(b.urgent.len(), 1);
        assert_eq!(s.candidates[b.urgent[0].candidate].kind, FOOD_POISON);
        assert_eq!(b.urgent[0].needs, [Need::Food, Need::Poison]);
        assert_eq!(b.gold, 90);
    }

    /// The purse bounds the urgent: the most urgent needs it can pay for are
    /// met, the rest recorded unmet, and the visit is not blocked.
    #[test]
    fn an_unaffordable_urgent_need_is_recorded_not_blocking() {
        let s = build(&general_and_alchemist(), &[]);
        let unmet = [Need::Light, Need::Food, Need::Poison];
        // 2 + 4 = 6 is affordable; the 30-gold cure is not
        let b = Budget::build(&facts(10), &s, &unmet, serves);
        let bought: Vec<u32> = b.urgent.iter().map(|f| s.candidates[f.candidate].kind).collect();
        assert_eq!(bought, [TORCH, RATION]);
        assert_eq!(b.unmet, [Need::Poison]);
        assert_eq!(b.gold, 4);
        // with 32 gold two urgent needs are still the most it can meet (all
        // three cost 36); of the pairs it can pay for, the cheapest wins
        let b = Budget::build(&facts(32), &s, &unmet, serves);
        assert_eq!(b.unmet, [Need::Poison]);
        assert_eq!(b.gold, 26);
        // with 36 all three are met
        let b = Budget::build(&facts(36), &s, &unmet, serves);
        assert!(b.unmet.is_empty());
        assert_eq!(b.gold, 0);
    }

    /// #859: an urgent Recall (the dive gate's) is bought first, ahead of
    /// everything the solver would weigh, with a heavy purse and no other need.
    #[test]
    fn an_urgent_recall_is_bought_first() {
        const RECALL: u32 = 7;
        let t = town([&[], &[], &[], &[], &[item(CLW, 5, 20), item(RECALL, 3, 125)], &[], &[], &[]]);
        let s = build(&t, &[]);
        let serves = |c: &Candidate| if c.kind == RECALL { vec![Need::Trip] } else { serves(c) };
        let b = Budget::build(&facts(20_000), &s, &[Need::Trip], serves);
        let bought: Vec<u32> = b.urgent.iter().map(|f| s.candidates[f.candidate].kind).collect();
        assert_eq!(bought, [RECALL]);
        assert_eq!(b.urgent[0].needs, [Need::Trip]);
        assert!(b.unmet.is_empty());
    }

    /// #956 (P7 review): when the purse cannot pay for every urgent need,
    /// the urgent Recall comes before a cheaper need. 140 gold, a 125-gold
    /// Recall and a 20-gold ration: the Recall, not the ration.
    #[test]
    fn an_urgent_recall_is_not_starved_by_a_cheaper_urgent_need() {
        const RECALL: u32 = 7;
        let t = town([&[item(RATION, 20, 20)], &[], &[], &[], &[item(RECALL, 3, 125)], &[], &[], &[]]);
        let s = build(&t, &[]);
        let price = |k: u32| i64::from(s.candidates.iter().find(|c| c.kind == k).unwrap().price);
        let purse = price(RECALL) + price(RATION) - 5;
        assert!(purse >= price(RECALL));
        let serves = |c: &Candidate| if c.kind == RECALL { vec![Need::Trip] } else { serves(c) };
        let b = Budget::build(&facts(purse), &s, &[Need::Food, Need::Trip], serves);
        let bought: Vec<u32> = b.urgent.iter().map(|f| s.candidates[f.candidate].kind).collect();
        assert_eq!(bought, [RECALL]);
        assert_eq!(b.unmet, [Need::Food]);
        // Without the Recall among the needs, the count still decides.
        let b = Budget::build(&facts(purse), &s, &[Need::Food], serves);
        let bought: Vec<u32> = b.urgent.iter().map(|f| s.candidates[f.candidate].kind).collect();
        assert_eq!(bought, [RATION]);
        // When both are paid for, the Recall is first in the purchase order
        // even though the ration is the earlier candidate (review of #961).
        let b = Budget::build(&facts(purse + 5), &s, &[Need::Food, Need::Trip], serves);
        let bought: Vec<u32> = b.urgent.iter().map(|f| s.candidates[f.candidate].kind).collect();
        assert_eq!(bought, [RECALL, RATION]);
    }

    /// An urgent need no shop serves is unmet whatever the purse.
    #[test]
    fn an_urgent_need_no_shop_serves_is_unmet() {
        let t = town([&[item(TORCH, 1, 2)], &[], &[], &[], &[], &[], &[], &[]]);
        let s = build(&t, &[]);
        let b = Budget::build(&facts(10_000), &s, &[Need::Light, Need::Healing], serves);
        assert_eq!(b.urgent.len(), 1);
        assert_eq!(b.unmet, [Need::Healing]);
    }

    /// #1103 (owner: flat 6000, a level negative delta of 100): the purse the
    /// Black Market opens at.
    #[test]
    fn the_black_market_opens_at_a_threshold_that_falls_with_level() {
        assert_eq!(black_market_threshold(6000, 100, 1), 5900);
        assert_eq!(black_market_threshold(6000, 100, 30), 3000);
        assert_eq!(black_market_threshold(6000, 100, 50), 1000);
        assert_eq!(black_market_threshold(6000, 100, 80), 0, "never below zero");
        assert_eq!(black_market_threshold(6000, 0, 50), 6000, "delta 0: flat");
        assert_eq!(black_market_threshold(0, 100, 10), 0, "min_gold 0: no threshold");
        assert_eq!(black_market_threshold(6000, -5, 10), 6000, "a negative delta is none");
        // The share: nothing below the purse, the configured one from it.
        assert_eq!(black_market_share(60, 5999, 1, 6000, 0), 0);
        assert_eq!(black_market_share(60, 6000, 1, 6000, 0), 60);
        assert_eq!(black_market_share(60, 2999, 30, 6000, 100), 0, "level 30 opens at 3000");
        assert_eq!(black_market_share(60, 3000, 30, 6000, 100), 60);
        assert_eq!(black_market_share(-1, 100, 30, 0, 100), -1, "no threshold: the legacy share, unconfigured");
        assert_eq!(black_market_share(-1, 100, 30, 6000, 100), 0, "below it even an unconfigured share is closed");
        assert_eq!(black_market_share(-1, 9000, 30, 6000, 100), -1);
    }

    /// The urgent needs take the cheapest source, Black Market included - they
    /// are not priced, so the caution cap does not bind them - and equal
    /// prices go to the earliest ware.
    #[test]
    fn the_urgent_needs_take_the_cheapest_source() {
        let t = town([&[], &[], &[], &[], &[item(CLW, 1, 20)], &[item(CLW, 1, 20)], &[item(CLW, 1, 60)], &[]]);
        let s = build(&t, &[]);
        let b = Budget::build(&facts(100), &s, &[Need::Healing], serves);
        assert_eq!(s.candidates[b.urgent[0].candidate].shop, 4);
        // only the Black Market has it: it is bought, cap or no cap
        let t = town([&[], &[], &[], &[], &[], &[], &[item(CLW, 1, 60)], &[]]);
        let s = build(&t, &[]);
        let b = Budget::build(&facts(100), &s, &[Need::Healing], serves);
        assert!(s.candidates[b.urgent[0].candidate].is_black_market());
        assert_eq!(b.gold, 40);
    }

    /// `serves` is asked once per candidate in the urgent pass and once per
    /// urgent purchase after it - never per subset, because in production
    /// it reaches the engine's effect tables.
    #[test]
    fn serves_is_asked_once_per_candidate() {
        use std::cell::Cell;
        // fifteen candidates across the four urgent needs' masks
        let t = town([
            &[item(TORCH, 1, 2), item(RATION, 1, 4), item(FOOD_POISON, 1, 9)],
            &[item(TORCH, 1, 3), item(RATION, 1, 5)],
            &[],
            &[],
            &[item(CLW, 1, 20), item(CURE_POISON, 1, 30), item(CLW, 1, 21)],
            &[item(CURE_POISON, 1, 31)],
            &[item(TORCH, 1, 6), item(CLW, 1, 60)],
            &[],
        ]);
        let s = build(&t, &[]);
        let calls = Cell::new(0);
        let counting = |c: &Candidate| {
            calls.set(calls.get() + 1);
            serves(c)
        };
        let unmet = [Need::Light, Need::Food, Need::Healing, Need::Poison];
        let b = Budget::build(&facts(1000), &s, &unmet, counting);
        assert!(b.unmet.is_empty());
        assert_eq!(calls.get(), s.candidates.len() + b.urgent.len());
    }

    /// The same inputs give the same budget.
    #[test]
    fn the_budget_is_deterministic() {
        let s = build(&general_and_alchemist(), &[item(TORCH, 1, 0)]);
        let unmet = [Need::Food, Need::Healing, Need::Poison];
        let a = Budget::build(&facts(40), &s, &unmet, serves);
        let b = Budget::build(&facts(40), &s, &unmet, serves);
        assert_eq!(a, b);
    }    #[test]
    fn an_urgent_home_cure_precedes_a_paid_shop_cure() {
        let mut s=build(&general_and_alchemist(),&[]);
        let mut home=*s.candidates.iter().find(|c|c.kind==CURE_POISON).unwrap();
        home.shop=7;home.slot=0;home.class=200;home.qty=1;home.price=0;home.slots_cost=0;
        s.candidates.push(home);
        let mut f=facts(0);f.pack_capacity=0;
        let b=Budget::build(&f,&s,&[Need::Poison],serves);
        assert!(b.unmet.is_empty());assert_eq!(b.gold,0);assert_eq!(b.pack_slots,0);
        assert_eq!(s.candidates[b.urgent[0].candidate].shop,7);
        let b=Budget::build(&facts(100),&s,&[Need::Poison],serves);
        assert_eq!(s.candidates[b.urgent[0].candidate].shop,7);
    }


    /// #493: the reserve is the larger of the fixed amount and the percent of
    /// the purse, never more than the purse; urgent purchases may use it.
    #[test]
    fn gold_reserve_is_the_larger_form_and_urgent_purchases_may_use_it() {
        assert_eq!(gold_reserve(1000, 0, 0), 0);
        assert_eq!(gold_reserve(1000, 100, 0), 100);
        assert_eq!(gold_reserve(1000, 0, 25), 250);
        assert_eq!(gold_reserve(1000, 100, 25), 250);
        assert_eq!(gold_reserve(1000, 300, 25), 300);
        assert_eq!(gold_reserve(50, 300, 0), 50, "never more than the purse");
        assert_eq!(spendable(1000, 0, 0), 1000);
        assert_eq!(spendable(1000, 100, 250), 650);
        assert_eq!(spendable(100, 80, 50), 0, "urgent purchases dipped into the reserve");
    }

    /// #493: the budget's gold leaves the reserve unspent; the urgent needs
    /// are bought from the whole purse and may use it.
    #[test]
    fn the_budget_keeps_the_reserve_and_the_urgent_needs_may_use_it() {
        let t = general_and_alchemist();
        let s = build(&t, &[]);
        assert_eq!(Budget::build_with_reserve(&facts(1000), &s, &[], 0, serves).gold, 1000);
        assert_eq!(Budget::build_with_reserve(&facts(1000), &s, &[], 250, serves).gold, 750);
        let urgent = Budget::build(&facts(100), &s, &[Need::Light], serves);
        let cost = 100 - urgent.gold;
        assert!(cost > 0, "the torch is bought");
        let b = Budget::build_with_reserve(&facts(100), &s, &[Need::Light], 100, serves);
        assert_eq!(b.urgent, urgent.urgent, "the urgent purchase is unchanged by the reserve");
        assert_eq!(b.gold, 0, "nothing else is bought from the reserve");
    }
}
