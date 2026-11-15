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
//! * **gold** - the purse minus what the urgent needs cost. No trip reserve, no
//!   savings. The proceeds of sales join it in the joint treatment (phase
//!   5), not here;
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
    /// The purse minus the urgent needs' cost.
    pub gold: i64,
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
/// by (most urgent needs met, least gold, fewest wares, earliest wares). Returns
/// the chosen candidate indices and the bit mask of `need` they cover.
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
                let key = (std::cmp::Reverse(covered.count_ones()), cost, picked.len());
                let old = (std::cmp::Reverse(cov.count_ones()), *c, p.len());
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

        let urgent: Vec<UrgentPurchase> = picked
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
            pack_slots: facts.pack_capacity - stock.pack_slots_used() - opened.len() as i32,
            home_slots: facts.home_capacity - stock.home_slots_used(),
            weight: facts.allowance() - weight,
            urgent,
            unmet,
        }
    }

    /// The budget of the current state over a stock built from it; the
    /// trip carries what the urgent needs read beyond the traits (the
    /// curing spell).
    pub fn of(st: &BorgState, stock: &Stock, trip: &Trip) -> Self {
        Self::build(&Facts::of(st), stock, &crate::borg_town_utility::urgent_needs(st, trip), |c| {
            let ware = &st.store.shops[usize::from(c.shop)].ware[usize::from(c.slot)];
            crate::borg_town_stock::needs_served(st, ware)
        })
    }
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

    /// An urgent need no shop serves is unmet whatever the purse.
    #[test]
    fn an_urgent_need_no_shop_serves_is_unmet() {
        let t = town([&[item(TORCH, 1, 2)], &[], &[], &[], &[], &[], &[], &[]]);
        let s = build(&t, &[]);
        let b = Budget::build(&facts(10_000), &s, &[Need::Light, Need::Healing], serves);
        assert_eq!(b.urgent.len(), 1);
        assert_eq!(b.unmet, [Need::Healing]);
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

}
