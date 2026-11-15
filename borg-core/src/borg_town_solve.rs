//! Phase 3 of the town inventory redesign (borg-rs#36): the solver - what
//! to buy.
//!
//! The form is the one the B2 write-up (`docs/TOWN-INVENTORY-B2.md`,
//! accepted 2026-11-13) derives:
//!
//! * the **groups**: one per stack class for the wares that stack (food,
//!   potions, scrolls - class = kind there), whose options are 0..n copies
//!   bought cheapest-first; and, for the wares a character wears, **one
//!   multiple-choice group per equipment slot** (Q1), whose options are
//!   "none" or exactly one of the wares that fit it;
//! * the **costs**: cumulative cheapest-n within a class, so the Black
//!   Market's 3x binds last; the caution (`cost <= purse / k`) is applied
//!   **per group** (Q2), to the Black Market copies of the whole option,
//!   and truncates the table there;
//! * the **values**: each group priced on its own, against the deficit as
//!   the carried kit stands - exact for that **separable surrogate** (Q3);
//!   the re-solve after every action prices what is left;
//! * the **DP**: the exact group knapsack over the gold, with the gold
//!   bound `min(gold, the sum of every group's dearest option)`;
//! * the **pack slots**: the Lagrangian sweep, a penalty `lambda` per slot
//!   an option opens, binary-searched to the smallest `lambda` whose
//!   solution fits (P15's recommendation). The relaxation has a gap - a
//!   tie at the critical `lambda` can drop every slot-opening option at
//!   once, down to the empty plan - so the solver keeps the best
//!   *feasible* set any `lambda` produced, and then **fills**: it adds or
//!   upgrades, one group at a time, while every limit holds. It is still
//!   not guaranteed optimal; the Lagrangian dual gives an upper bound on
//!   the optimum, and [`Solution::optimal`] is true only when the value
//!   reaches it;
//! * the **weight**: a repair after the sweep - the change whose value lost
//!   per weight freed is least, among those that keep the gold and the
//!   slots within their limits, until the carry allowance holds;
//! * the **ties**, deterministic: more usefulness, then less gold, then
//!   fewer slots, then the earlier group's choice kept. The DP and the
//!   choice among the sweep's feasible sets honour this order exactly;
//!   the fill is a greedy over single-group changes and honours it only
//!   at each step, so the final set need not be the tie order's choice
//!   among the sets of its value.
//!
//! The urgent needs are not here: the budget (phase 2) bought them first, and
//! [`groups`] removes their copies from the wares before it builds the
//! tables. Nothing in production calls this module yet; every public item
//! carries the self-removing `expect(dead_code)` marker of phases 1-2.

use std::collections::BTreeMap;

use crate::borg_town_budget::Budget;
use crate::borg_town_stock::{Candidate, Stock};

/// The largest gold bound the DP runs on as is. Above it the costs are
/// scaled down (each rounded *up*, so a solution never overspends) and
/// the solution is marked inexact.
pub const GOLD_LIMIT: i64 = 1 << 16;

/// What a group is: one stack class, or one equipment slot.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum GroupKey {
    /// The wares of one stack class; option `n` is `n` copies.
    Class(u16),
    /// Alternatives serving one shared need (utility model, v5).
    Need(u8),
    /// The wares that fit one equipment slot; at most one is bought.
    Slot(u8),
}

/// One option of a group: what it buys and what it costs.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Choice {
    /// (candidate index, copies) bought by this option.
    pub picks: Vec<(usize, i32)>,
    pub cost: i64,
    /// The usefulness this option adds (the surrogate's value).
    pub value: i64,
    /// The pack slots it opens.
    pub slots: i32,
    pub weight: i64,
}

/// A group: its key and its options, option 0 always "buy nothing".
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Group {
    pub key: GroupKey,
    pub options: Vec<Choice>,
}

/// The constraints the solver respects (the budget after the urgent needs).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Limits {
    pub gold: i64,
    pub pack_slots: i32,
    pub weight: i64,
}

impl Limits {
    /// The limits a budget leaves.
    pub fn of(budget: &Budget) -> Self {
        Limits { gold: budget.gold, pack_slots: budget.pack_slots, weight: budget.weight }
    }
}

/// The chosen set.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Solution {
    /// The option chosen in each group, by group index (0 = nothing).
    pub chosen: Vec<usize>,
    pub value: i64,
    pub cost: i64,
    pub slots: i32,
    pub weight: i64,
    /// The slot penalty the sweep settled on (0 when slots never bound).
    pub lambda: i64,
    /// The Lagrangian dual: an upper bound on the value of any set within
    /// the gold and the slots (the weight is not in it). Meaningful only
    /// when `gold_exact`.
    pub bound: i64,
    /// True when the set meets every limit (gold, slots and weight) and
    /// its value reaches `bound` with the gold unscaled: it is then
    /// provably optimal under all three (the weight-constrained optimum
    /// cannot exceed the bound). False says only that it was not proved -
    /// a sweep gap, a weight repair, a scaled purse, or a limit missed
    /// (`slots_met` / `weight_met` say which).
    pub optimal: bool,
    /// False when the gold had to be scaled (bound above [`GOLD_LIMIT`]).
    pub gold_exact: bool,
    /// The set opens no more slots than the limit. False when the limit
    /// was negative (the urgent needs overfilled the pack): nothing
    /// slot-opening is bought then, and the dispositions must free room
    /// first.
    pub slots_met: bool,
    /// The set weighs no more than the carry allowance. False only when
    /// the allowance was negative and nothing the plan can drop fixes it.
    pub weight_met: bool,
    /// True when the weight repair had to reduce or change an option.
    pub weight_repaired: bool,
}

impl Solution {
    /// The chosen options as (group, option), ranked: the most useful
    /// first, then the cheaper, then the fewer slots, then the earlier
    /// group (B2 section 6 item 6). The plan (phase 4) executes the first
    /// and re-solves.
    pub fn ranked(&self, groups: &[Group]) -> Vec<(usize, usize)> {
        let mut out: Vec<(usize, usize)> = self
            .chosen
            .iter()
            .enumerate()
            .filter(|&(_, &o)| o != 0)
            .map(|(g, &o)| (g, o))
            .collect();
        out.sort_by_key(|&(g, o)| {
            let c = &groups[g].options[o];
            (std::cmp::Reverse(c.value), c.cost, c.slots, g)
        });
        out
    }
}

/// A DP cell's score: the penalised value, then less gold, then fewer
/// slots, packed into one integer so that a cell update is one add and
/// one compare. The packing is exact: the cost and the slots of any
/// selection fit below their fields (a cost below 2^47, at most 2^16
/// slots), so the lexicographic order and its addition carry over to the
/// integer. Lexicographic order is translation-invariant, so the group DP
/// stays exact for it.
type Score = i128;

/// The score of one option.
fn score(value: i64, cost: i64, slots: i32) -> Score {
    (i128::from(value) << 64) - (i128::from(cost) << 16) - i128::from(slots)
}

/// The exact group DP at slot penalty `lambda`, on costs divided by
/// `scale` (rounded up) under the scaled bound. Returns the option chosen
/// per group.
fn dp(groups: &[Group], bound: i64, scale: i64, lambda: i64, cash: Cash) -> Vec<usize> {
    let cap = usize::try_from(bound / scale).unwrap_or(0);
    let scaled = |c: i64| (c + scale - 1) / scale;
    let mut best: Vec<Score> = vec![0; cap + 1];
    let mut arg: Vec<Vec<u16>> = Vec::with_capacity(groups.len());
    for g in groups {
        // The options this group can afford at all, scaled and penalised.
        let opts: Vec<(u16, usize, Score)> = g
            .options
            .iter()
            .enumerate()
            .skip(1)
            .filter_map(|(oi, o)| {
                let c = usize::try_from(scaled(o.cost)).ok().filter(|&c| c <= cap)?;
                let add = score(o.value - lambda * i64::from(o.slots), o.cost, o.slots);
                Some((oi as u16, c, add))
            })
            .collect();
        let mut pick = vec![0u16; cap + 1];
        // In place, the budget downward: every option of cost c >= 1 reads
        // best[b - c], which this group has not written yet, so the row
        // never needs copying. A cost-0 option reads the old best[b],
        // kept in `old`. The first option to reach a score keeps it.
        for b in (0..=cap).rev() {
            let old = best[b];
            let mut cur = old;
            for &(oi, c, add) in &opts {
                if c > b {
                    continue;
                }
                let cand = if c == 0 { old } else { best[b - c] } + add;
                if cand > cur {
                    cur = cand;
                    pick[b] = oi;
                }
            }
            best[b] = cur;
        }
        arg.push(pick);
    }
    // Walk back from the full bound.
    let mut chosen = vec![0usize; groups.len()];
    // best[b] is the best kit within b gold. Comparing all b prices
    // retained cash against the WHOLE kit, not independently per ware.
    let mut b = (0..=cap).max_by_key(|&b| {
        let v = (best[b] + ((1i128 << 64) - 1)) >> 64;
        let spent = (((v << 64) - best[b]) >> 16) as i64;
        best[b] - (i128::from(cash.loss(spent)) << 64)
    }).unwrap_or(0);
    for (gi, g) in groups.iter().enumerate().rev() {
        let oi = usize::from(arg[gi][b]);
        chosen[gi] = oi;
        if oi != 0 {
            b -= usize::try_from(scaled(g.options[oi].cost)).unwrap_or(0);
        }
    }
    chosen
}

/// The totals of a choice.
fn totals(groups: &[Group], chosen: &[usize]) -> (i64, i64, i32, i64) {
    chosen.iter().enumerate().fold((0, 0, 0, 0), |(v, c, s, w), (g, &o)| {
        let x = &groups[g].options[o];
        (v + x.value, c + x.cost, s + x.slots, w + x.weight)
    })
}

/// The packed score's fields (see [`Score`]): a selection's total cost
/// must stay below 2^47 and its slots below 2^16.
const COST_FIELD: i64 = 1 << 47;
const SLOT_FIELD: i64 = 1 << 16;

/// Solve: the chosen option per group under the limits.
///
/// Panics (an abort, under the release profile's `panic = "abort"`) when
/// the inputs are past the packed score's fields - a gold bound of 2^47
/// or more, 2^16 slots or more, or a negative cost - rather than return a
/// silently wrong set. The game's inputs are far inside them; whether the
/// plan degrades instead (scaling further, or clamping the bound) is a
/// phase-4 decision.
pub fn solve(groups: &[Group], limits: &Limits) -> Solution {
    solve_cash(groups, limits, 0)
}

/// Convex cost of consuming the purse's future purchasing options.
/// No fixed reserve: spending c loses w*c/(remaining+1), in utility units.
#[derive(Clone, Copy)]
struct Cash { gold: i64, weight: i64 }
impl Cash {
    fn loss(self, spent: i64) -> i64 {
        self.weight.saturating_mul(spent.max(0)) / (self.gold - spent).max(0).saturating_add(1)
    }
}

/// As solve, with utility for retaining gold. Values include the cash loss;
/// costs, repairs, fills and the dual all use the same objective.
pub fn solve_cash(groups: &[Group], limits: &Limits, gold_weight: i64) -> Solution {
    let cash = Cash { gold: limits.gold.max(0), weight: gold_weight.max(0) };
    // The gold bound: the purse, or less when every group's dearest
    // option together costs less.
    let dearest: i64 = groups
        .iter()
        .map(|g| g.options.iter().map(|o| o.cost).max().unwrap_or(0))
        .sum();
    let gold = limits.gold.max(0);
    let bound = gold.min(dearest);
    // The packed score is exact only inside its fields: a wrong answer
    // past them would be silent, so refuse instead.
    let most_slots: i64 = groups
        .iter()
        .map(|g| g.options.iter().map(|o| i64::from(o.slots)).max().unwrap_or(0))
        .sum();
    assert!(bound < COST_FIELD, "solve: gold bound {bound} past the score's cost field");
    assert!(most_slots < SLOT_FIELD, "solve: {most_slots} slots past the score's slot field");
    assert!(groups.iter().all(|g| g.options.iter().all(|o| o.cost >= 0)), "solve: a negative cost");
    let scale = if bound > GOLD_LIMIT { (bound + GOLD_LIMIT - 1) / GOLD_LIMIT } else { 1 };

    let slot_limit = limits.pack_slots.max(0);
    let within = |c: i64, s: i32| c <= gold && s <= slot_limit;

    // The sweep. Every lambda tried is a dual bound, and every feasible
    // set it produces is a candidate: the best of them is kept, not only
    // the one at the smallest fitting lambda.
    let mut dual = i64::MAX;
    let mut best: Option<(Vec<usize>, (i64, i64, i32))> = None;
    let mut consider = |lambda: i64, ch: Vec<usize>| -> bool {
        let (gross, c, s, _) = totals(groups, &ch);
        let v = gross - cash.loss(c);
        dual = dual.min(v - lambda * i64::from(s) + lambda * i64::from(slot_limit));
        let fits = within(c, s);
        if fits {
            // More value, then less gold, then fewer slots.
            let key = (v, -c, -s);
            if best.as_ref().is_none_or(|(_, k)| key > *k) {
                best = Some((ch, key));
            }
        }
        fits
    };
    let mut lambda = 0;
    if !consider(0, dp(groups, bound, scale, 0, cash)) {
        // Above the largest option value, nothing slot-opening pays.
        let mut hi = groups
            .iter()
            .flat_map(|g| g.options.iter().map(|o| o.value))
            .max()
            .unwrap_or(0)
            + 1;
        let mut lo = 0;
        let mut found = false;
        while hi - lo > 1 {
            let mid = lo + (hi - lo) / 2;
            if consider(mid, dp(groups, bound, scale, mid, cash)) {
                hi = mid;
                found = true;
            } else {
                lo = mid;
            }
        }
        if !found {
            consider(hi, dp(groups, bound, scale, hi, cash));
        }
        lambda = hi;
    }
    let mut chosen = best.map_or_else(|| vec![0; groups.len()], |(ch, _)| ch);

    // The weight: the change losing the least value per weight freed,
    // among those that keep the gold and the slots within their limits.
    // Option 0 always qualifies (it frees all, costs nothing, opens
    // nothing), so the loop ends.
    let mut weight_repaired = false;
    loop {
        let (_, c, s, w) = totals(groups, &chosen);
        if w <= limits.weight || w <= 0 {
            break;
        }
        // (group, option, value lost, weight freed); ties to the earlier
        // group, then the lower option.
        let mut pick: Option<(usize, usize, i64, i64)> = None;
        for (g, &o) in chosen.iter().enumerate() {
            let cur = &groups[g].options[o];
            for (j, alt) in groups[g].options.iter().enumerate() {
                let freed = cur.weight - alt.weight;
                if j == o || freed <= 0 || !within(c - cur.cost + alt.cost, s - cur.slots + alt.slots) {
                    continue;
                }
                let loss = cur.value - alt.value - cash.loss(c) + cash.loss(c - cur.cost + alt.cost);
                let better = match pick {
                    None => true,
                    Some((_, _, l, f)) => i128::from(loss) * i128::from(f) < i128::from(l) * i128::from(freed),
                };
                if better {
                    pick = Some((g, j, loss, freed));
                }
            }
        }
        let Some((g, j, _, _)) = pick else { break };
        chosen[g] = j;
        weight_repaired = true;
    }

    // The fill: the single-group change that adds the most value while
    // the gold, the slots and the weight hold (the weight may not grow
    // past the allowance, nor at all while it is over); ties to less gold,
    // then fewer slots, then the earlier group, then the lower option.
    loop {
        let (_, c, s, w) = totals(groups, &chosen);
        let mut pick: Option<(usize, usize, (i64, i64, i32))> = None;
        for (g, &o) in chosen.iter().enumerate() {
            let cur = &groups[g].options[o];
            for (j, alt) in groups[g].options.iter().enumerate() {
                let gain = alt.value - cur.value + cash.loss(c) - cash.loss(c - cur.cost + alt.cost);
                let (nc, ns, nw) = (c - cur.cost + alt.cost, s - cur.slots + alt.slots, w - cur.weight + alt.weight);
                if gain <= 0 || !within(nc, ns) || (nw > limits.weight && nw > w) {
                    continue;
                }
                let key = (gain, -(alt.cost - cur.cost), -(alt.slots - cur.slots));
                if pick.is_none_or(|(_, _, k)| key > k) {
                    pick = Some((g, j, key));
                }
            }
        }
        let Some((g, j, _)) = pick else { break };
        chosen[g] = j;
    }

    let (gross, cost, slots, weight) = totals(groups, &chosen);
    let value = gross - cash.loss(cost);
    let gold_exact = scale == 1;
    Solution {
        chosen,
        value,
        cost,
        slots,
        weight,
        lambda,
        bound: dual,
        // The dual is built on the slot limit clamped at 0 and ignores the
        // weight, so it proves nothing about a set that misses either.
        optimal: gold_exact && slots <= limits.pack_slots && weight <= limits.weight && value >= dual,
        gold_exact,
        slots_met: slots <= limits.pack_slots,
        weight_met: weight <= limits.weight,
        weight_repaired,
    }
}

/// Build the groups from the stock, after the budget's urgent needs.
///
/// * `caution` is `k` of the share cap (`borg_town_share`; 0 disables
///   it): an option's Black Market copies may cost at most `purse / k`
///   together (per group, Q2), `purse` the budget's gold;
/// * `wearable(c)` names the equipment slot a ware is worn in and the
///   value of wearing it (the try-on), or `None` for a ware that stacks;
/// * `value(c, n)` is the usefulness of `n` copies of `c`'s class - the
///   flat-then-capped curve of phase 1 in production. Every copy count
///   up to the copies available is priced, and an option is kept only
///   when it raises the value above every smaller one: so the table ends
///   at the first count that reaches the maximum (B2 section 6 item 3),
///   and an early flat step - the curve's integer rounding can make the
///   first copy of a large gap worth 0 - does not end it. A count the
///   caution forbids ends it, since the Black Market copies only grow
///   with the count.
pub fn groups(
    stock: &Stock,
    budget: &Budget,
    caution: i64,
    wearable: impl Fn(&Candidate) -> Option<(u8, i64)>,
    value: impl Fn(&Candidate, i32) -> i64,
) -> Vec<Group> {
    // The copies left after the urgent needs.
    let mut qty: Vec<i32> = stock.candidates.iter().map(|c| c.qty).collect();
    for f in &budget.urgent {
        qty[f.candidate] -= 1;
    }
    let purse = budget.gold.max(0);
    let allowed = |bm_cost: i64| caution <= 0 || bm_cost.saturating_mul(caution) <= purse;

    let mut by_class: BTreeMap<u16, Vec<usize>> = BTreeMap::new();
    let mut by_slot: BTreeMap<u8, Vec<(usize, i64)>> = BTreeMap::new();
    for (i, c) in stock.candidates.iter().enumerate() {
        if qty[i] < 1 {
            continue;
        }
        match wearable(c) {
            Some((slot, v)) => by_slot.entry(slot).or_default().push((i, v)),
            None => by_class.entry(c.class).or_default().push(i),
        }
    }

    let mut out = Vec::new();
    for (class, mut wares) in by_class {
        let cands = &stock.candidates;
        wares.sort_by_key(|&i| (cands[i].price, cands[i].shop, cands[i].slot));
        // A class opens a slot if any of its wares would. The production
        // stacking predicate makes a class's wares agree (stacking is an
        // equivalence within a kind); should they ever not, `any` errs to
        // the safe side - a slot counted that may not be needed.
        let opens = i32::from(wares.iter().any(|&i| cands[i].slots_cost != 0));
        let rep = &cands[wares[0]];
        let mut options = vec![Choice::default()];
        let mut prev = 0;
        let mut n = 0;
        loop {
            n += 1;
            // The n cheapest copies.
            let mut left = n;
            let mut picks = Vec::new();
            for &i in &wares {
                if left == 0 {
                    break;
                }
                let take = left.min(qty[i]);
                if take > 0 {
                    picks.push((i, take));
                    left -= take;
                }
            }
            if left > 0 {
                break; // the copies ran out
            }
            let cost = picks.iter().map(|&(i, k)| i64::from(cands[i].price) * i64::from(k)).sum();
            let bm: i64 = picks
                .iter()
                .filter(|&&(i, _)| cands[i].is_black_market())
                .map(|&(i, k)| i64::from(cands[i].price) * i64::from(k))
                .sum();
            if !allowed(bm) {
                break; // the caution truncates the table
            }
            let v = value(rep, n);
            if v <= prev {
                continue; // no more than a smaller count: dominated
            }
            let weight = picks.iter().map(|&(i, k)| i64::from(cands[i].weight) * i64::from(k)).sum();
            options.push(Choice { picks, cost, value: v, slots: opens, weight });
            prev = v;
        }
        if options.len() > 1 {
            out.push(Group { key: GroupKey::Class(class), options });
        }
    }
    for (slot, wares) in by_slot {
        let cands = &stock.candidates;
        let mut options = vec![Choice::default()];
        for (i, v) in wares {
            let c = &cands[i];
            let cost = i64::from(c.price);
            if v <= 0 || (c.is_black_market() && !allowed(cost)) {
                continue;
            }
            options.push(Choice {
                picks: vec![(i, 1)],
                cost,
                value: v,
                slots: c.slots_cost,
                weight: i64::from(c.weight),
            });
        }
        if options.len() > 1 {
            out.push(Group { key: GroupKey::Slot(slot), options });
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::borg_town_budget::{Facts, UrgentPurchase};
    use crate::borg_town_stock::tests::{build, item, town};
    use crate::borg_town_usefulness::value as curve;

    fn opt(cost: i64, value: i64, slots: i32, weight: i64) -> Choice {
        Choice { picks: Vec::new(), cost, value, slots, weight }
    }

    /// A class group from (cost, value) per copy count, one slot.
    fn class(key: u16, table: &[(i64, i64)]) -> Group {
        let mut options = vec![Choice::default()];
        options.extend(table.iter().map(|&(c, v)| opt(c, v, 1, 0)));
        Group { key: GroupKey::Class(key), options }
    }

    fn limits(gold: i64) -> Limits {
        Limits { gold, pack_slots: 23, weight: 1_000_000 }
    }

    /// Brute force over every combination: the best value under gold
    /// and slots (and the tie order).
    fn brute(groups: &[Group], l: &Limits) -> i64 {
        fn go(g: &[Group], i: usize, left: (i64, i32, i64), acc: i64, best: &mut i64) {
            if i == g.len() {
                *best = (*best).max(acc);
                return;
            }
            let (gold, slots, weight) = left;
            for o in &g[i].options {
                if o.cost <= gold && o.slots <= slots && o.weight <= weight {
                    go(g, i + 1, (gold - o.cost, slots - o.slots, weight - o.weight), acc + o.value, best);
                }
            }
        }
        let mut best = 0;
        go(groups, 0, (l.gold, l.pack_slots, l.weight), 0, &mut best);
        best
    }

    /// The greedy the old strategy approximates: the best value per gold
    /// first, one option per group, while the purse lasts.
    fn greedy(groups: &[Group], gold: i64) -> i64 {
        let mut opts: Vec<&Choice> = groups.iter().flat_map(|g| g.options.iter().skip(1).take(1)).collect();
        opts.sort_by(|a, b| (b.value * a.cost).cmp(&(a.value * b.cost)));
        let (mut left, mut v) = (gold, 0);
        for o in opts {
            if o.cost <= left {
                left -= o.cost;
                v += o.value;
            }
        }
        v
    }

    /// A slot group (one equipment slot) from (cost, value, slots, weight).
    fn slot_group(key: u8, opts: &[(i64, i64, i32, i64)]) -> Group {
        let mut options = vec![Choice::default()];
        options.extend(opts.iter().enumerate().map(|(i, &(c, v, s, w))| Choice {
            picks: vec![(i, 1)],
            ..opt(c, v, s, w)
        }));
        Group { key: GroupKey::Slot(key), options }
    }

    /// P15, the worked instance: A worth 12 for 6 (cap 1), B worth 9 a
    /// copy for 5 (cap 2), purse 10. The greedy by ratio takes A and
    /// scores 12; the optimum is two B for 18.
    #[test]
    fn p15_the_greedy_scores_12_the_optimum_is_18() {
        let g = [class(0, &[(6, 12)]), class(1, &[(5, 9), (10, 18)])];
        assert_eq!(greedy(&g, 10), 12, "the greedy takes A first and is stuck");
        let s = solve(&g, &limits(10));
        assert_eq!(s.value, 18);
        assert!(s.optimal);
        assert_eq!(s.chosen, [0, 2]);
        assert_eq!(s.cost, 10);
        assert!(s.gold_exact);
    }

    /// Exact against brute force on small seeded instances, gold-only.
    #[test]
    fn the_gold_dp_is_exact() {
        let mut seed: u64 = 0x5eed;
        let mut rnd = |m: i64| {
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            ((seed >> 33) as i64) % m
        };
        for _ in 0..200 {
            let groups: Vec<Group> = (0..4)
                .map(|k| {
                    let (mut c, mut v) = (0, 0);
                    let table: Vec<(i64, i64)> = (0..1 + rnd(3))
                        .map(|_| {
                            c += 1 + rnd(9);
                            v += 1 + rnd(9);
                            (c, v)
                        })
                        .collect();
                    class(k, &table)
                })
                .collect();
            let l = limits(rnd(30));
            let s = solve(&groups, &l);
            assert_eq!(s.value, brute(&groups, &l), "{groups:?} {l:?}");
            assert!(s.cost <= l.gold);
        }
    }

    /// Compare the cash-aware objective with exhaustive kit enumeration,
    /// including free options and different purse sizes. This catches
    /// fills or slot sweeps silently reverting to gross item value.
    #[test]
    fn cash_utility_matches_exhaustive_kits() {
        for gold in 0..30 {
            for weight in [0, 3, 25, 100] {
                let groups = [class(0, &[(0, 2), (3, 14), (8, 20)]),
                    class(1, &[(2, 11), (7, 17)]), class(2, &[(5, 18)])];
                for slots in [1, 2, 3] {
                    let l = Limits { gold, pack_slots: slots, weight: 1000 };
                    let got = solve_cash(&groups, &l, weight);
                    let mut best = 0;
                    for a in &groups[0].options {
                        for b in &groups[1].options {
                            for c in &groups[2].options {
                                let spent = a.cost + b.cost + c.cost;
                                if spent <= gold && a.slots + b.slots + c.slots <= slots {
                                    best = best.max(a.value + b.value + c.value - weight * spent / (gold - spent + 1));
                                }
                            }
                        }
                    }
                    assert!(got.value <= best);
                    assert!(got.cost <= gold && got.slots <= slots);
                    if slots == 3 { assert_eq!(got.value, best, "gold={gold}, weight={weight}"); }
                    if got.optimal { assert_eq!(got.value, best); }
                    assert!(got.bound >= best, "dual bound gold={gold}, slots={slots}, weight={weight}");
                }
            }
        }
    }

    /// The pack slots bind: the gold-only optimum takes three kinds, two
    /// slots are free, and the sweep finds the best two.
    #[test]
    fn the_sweep_fits_the_slots() {
        let g = [class(0, &[(1, 10)]), class(1, &[(1, 9)]), class(2, &[(1, 8)])];
        let mut l = limits(100);
        assert_eq!(solve(&g, &l).slots, 3);
        l.pack_slots = 2;
        let s = solve(&g, &l);
        assert_eq!(s.slots, 2);
        assert_eq!(s.value, 19);
        assert_eq!(s.value, brute(&g, &l));
        assert!(s.lambda > 0);
        assert!(s.slots_met);
    }

    /// A copy that joins a carried stack opens no slot, so it is free
    /// under the slot limit.
    #[test]
    fn a_stacking_group_costs_no_slot() {
        let mut free = class(1, &[(1, 5)]);
        free.options[1].slots = 0;
        let g = [class(0, &[(1, 10)]), free];
        let l = Limits { gold: 100, pack_slots: 1, weight: 1_000_000 };
        let s = solve(&g, &l);
        assert_eq!(s.chosen, [1, 1]);
        assert_eq!(s.slots, 1);
    }

    /// A negative slot limit (the urgent needs overfilled the pack): nothing
    /// slot-opening is bought, and the solution says the limit was not
    /// met rather than pretending.
    #[test]
    fn a_negative_slot_limit_buys_only_what_stacks() {
        let mut free = class(1, &[(1, 5)]);
        free.options[1].slots = 0;
        let g = [class(0, &[(1, 10)]), free];
        let l = Limits { gold: 100, pack_slots: -1, weight: 1_000_000 };
        let s = solve(&g, &l);
        assert_eq!(s.chosen, [0, 1]);
        assert_eq!(s.slots, 0);
        assert!(!s.slots_met, "the limit was negative: the dispositions must free room");
        assert!(!s.optimal, "a set that misses a limit is never called optimal");
        let s = solve(&g, &Limits { pack_slots: 1, ..l });
        assert!(s.slots_met);
    }

    /// The weight repair drops the least value per weight freed.
    #[test]
    fn the_weight_repair_drops_the_cheapest_loss() {
        let g = [
            Group { key: GroupKey::Class(0), options: vec![Choice::default(), opt(1, 10, 0, 50)] },
            Group { key: GroupKey::Class(1), options: vec![Choice::default(), opt(1, 9, 0, 10)] },
        ];
        let l = Limits { gold: 100, pack_slots: 23, weight: 55 };
        let s = solve(&g, &l);
        // dropping the heavy one loses 10 for 50 freed; the light one
        // would free too little
        assert_eq!(s.chosen, [0, 1]);
        assert!(s.weight_repaired);
        assert!(s.weight <= 55);
    }

    /// A purse that covers everything buys every useful option.
    #[test]
    fn a_purse_that_covers_everything() {
        let g = [class(0, &[(5, 3), (10, 4)]), class(1, &[(7, 9)])];
        let s = solve(&g, &limits(1_000));
        assert_eq!(s.chosen, [2, 1]);
        assert_eq!(s.cost, 17);
    }

    /// Ties go to less gold, then fewer slots, then the earlier group.
    #[test]
    fn the_ties_are_deterministic() {
        let g = [class(0, &[(5, 10)]), class(1, &[(4, 10)]), class(2, &[(4, 10)])];
        let s = solve(&g, &limits(4));
        assert_eq!(s.chosen, [0, 1, 0], "the cheaper, then the earlier");
        assert_eq!(solve(&g, &limits(4)), s);
    }

    /// Above the gold limit the costs are scaled up-rounded: the
    /// solution never overspends, and says it is not exact.
    #[test]
    fn a_large_purse_is_scaled_and_marked() {
        let g = [class(0, &[(70_001, 10)]), class(1, &[(70_001, 9)])];
        let s = solve(&g, &limits(140_002));
        assert!(!s.gold_exact);
        assert!(s.cost <= 140_002);
        let s = solve(&g, &limits(100_000));
        assert_eq!(s.chosen, [1, 0]);
    }

    /// The ranking: the most useful first.
    #[test]
    fn the_solution_is_ranked() {
        let g = [class(0, &[(1, 3)]), class(1, &[(1, 9)]), class(2, &[(2, 9)])];
        let s = solve(&g, &limits(10));
        assert_eq!(s.ranked(&g), [(1, 1), (2, 1), (0, 1)]);
    }

    /// The cost of a solve at the size the plan budgets for (risk list:
    /// "to be measured, not assumed"): 168 groups - every ware of seven
    /// shops - of five options, the gold at the limit, and the slots
    /// binding so the sweep runs its full binary search. Run by hand:
    /// `cargo test --release -- --ignored --nocapture solve_cost`.
    #[test]
    #[ignore]
    fn solve_cost_at_full_size() {
        let groups: Vec<Group> = (0..168u16)
            .map(|k| {
                let t: Vec<(i64, i64)> =
                    (1..=5).map(|n| (n * (50 + i64::from(k) * 7), n * (1 + i64::from(k % 13)))).collect();
                class(k, &t)
            })
            .collect();
        let l = Limits { gold: GOLD_LIMIT, pack_slots: 10, weight: 1_000_000 };
        let start = std::time::Instant::now();
        let s = solve(&groups, &l);
        let took = start.elapsed();
        assert!(s.slots <= 10 && s.cost <= GOLD_LIMIT);
        println!("solve: 168 groups x 5 options, gold {GOLD_LIMIT}, lambda {}: {took:?}", s.lambda);

        // The plan's own estimate (P15): 30,000 gold x 30 kinds x 20
        // copies, the slots binding as well.
        let groups: Vec<Group> = (0..30u16)
            .map(|k| {
                let t: Vec<(i64, i64)> =
                    (1..=20).map(|n| (n * (40 + i64::from(k) * 11), n * (1 + i64::from(k % 7)))).collect();
                class(k, &t)
            })
            .collect();
        let l = Limits { gold: 30_000, pack_slots: 10, weight: 1_000_000 };
        let start = std::time::Instant::now();
        let s = solve(&groups, &l);
        let took = start.elapsed();
        assert!(s.slots <= 10 && s.cost <= 30_000);
        println!("solve: 30 groups x 20 options, gold 30000, lambda {}: {took:?}", s.lambda);
    }

    // --- the review of 5c9eb30 (PR #42): its failing instances, kept ---

    /// The sweep's tie at the critical lambda dropped every option, and
    /// the solver returned nothing while one 10 fitted. The best feasible
    /// set and the fill find it.
    #[test]
    fn the_sweep_never_returns_nothing_when_something_fits() {
        let g = [class(0, &[(5, 10)]), class(1, &[(4, 10)]), class(2, &[(4, 10)])];
        let l = Limits { gold: 9, pack_slots: 1, weight: 1_000_000 };
        let s = solve(&g, &l);
        assert_eq!(s.value, 10);
        assert_eq!(s.value, brute(&g, &l));
        let g = [class(0, &[(10, 10)]), class(1, &[(10, 10)]), Group {
            key: GroupKey::Class(2),
            options: vec![Choice::default(), opt(20, 19, 2, 0)],
        }];
        let l = Limits { gold: 20, pack_slots: 1, weight: 1_000_000 };
        assert_eq!(solve(&g, &l).value, brute(&g, &l));
    }

    /// The gap was not "one item": one 100 and nine 60s with five slots
    /// returned 100 where 340 fits.
    #[test]
    fn the_sweep_gap_is_filled() {
        let mut g = vec![class(0, &[(1, 100)])];
        for k in 1..=9u16 {
            g.push(class(k, &[(1, 60)]));
        }
        let l = Limits { gold: 1000, pack_slots: 5, weight: 1_000_000 };
        let s = solve(&g, &l);
        assert_eq!(s.value, 340);
        assert!(s.slots_met);
    }

    /// The weight repair "downgraded" a slot group to a dearer ware and
    /// overspent the purse (500 against 100); now it keeps the gold.
    #[test]
    fn the_weight_repair_keeps_the_gold() {
        let g = [slot_group(3, &[(500, 40, 1, 5), (10, 50, 1, 95), (15, 10, 1, 80)])];
        let l = Limits { gold: 100, pack_slots: 23, weight: 60 };
        let s = solve(&g, &l);
        assert!(s.cost <= 100, "cost {}", s.cost);
        assert!(s.weight_met && s.weight <= 60);
        assert_eq!(s.value, brute(&g, &l));
        let g = [slot_group(3, &[(100, 40, 1, 20), (10, 50, 1, 100)])];
        let l = Limits { gold: 50, pack_slots: 5, weight: 30 };
        let s = solve(&g, &l);
        assert!(s.cost <= 50 && s.weight <= 30);
    }

    /// ... and the slots: the repair opened a slot the limit forbade.
    #[test]
    fn the_weight_repair_keeps_the_slots() {
        let g = [slot_group(3, &[(10, 29, 1, 10), (20, 30, 0, 100)])];
        let l = Limits { gold: 100, pack_slots: 0, weight: 60 };
        let s = solve(&g, &l);
        assert!(s.slots <= 0, "slots {}", s.slots);
        assert!(s.slots_met && s.weight <= 60);
    }

    /// A negative carry allowance: nothing the plan buys can fix it, and
    /// the solution says so instead of pretending.
    #[test]
    fn a_negative_weight_limit_is_reported() {
        let g = [Group { key: GroupKey::Class(0), options: vec![Choice::default(), opt(1, 10, 0, 5)] }];
        let s = solve(&g, &Limits { gold: 10, pack_slots: 5, weight: -1 });
        assert_eq!(s.weight, 0);
        assert!(!s.weight_met);
        assert!(!s.optimal, "a set that misses a limit is never called optimal");
    }

    /// The fuzz behind the two tests below: `n` seeded instances, slot
    /// groups whose option costs are not monotone among them; every limit
    /// must hold, the dual must bound the optimum, and "optimal" must
    /// never be claimed falsely. Returns (gaps, proved).
    fn fuzz(seed: u64, n: usize, weight_binds: bool) -> (usize, usize) {
        let mut seed = seed;
        let mut rnd = |m: i64| {
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            ((seed >> 33) as i64) % m
        };
        let (mut gaps, mut proved) = (0, 0);
        for _ in 0..n {
            let groups: Vec<Group> = (0..1 + rnd(5))
                .map(|k| {
                    if rnd(3) == 0 {
                        let opts: Vec<(i64, i64, i32, i64)> = (0..1 + rnd(3))
                            .map(|_| (rnd(15), 1 + rnd(20), rnd(2) as i32, rnd(30)))
                            .collect();
                        slot_group(k as u8, &opts)
                    } else {
                        let (mut c, mut v, mut w) = (0, 0, 0);
                        let slots = rnd(2) as i32;
                        let mut options = vec![Choice::default()];
                        for _ in 0..1 + rnd(3) {
                            c += 1 + rnd(8);
                            v += 1 + rnd(9);
                            w += rnd(12);
                            options.push(opt(c, v, slots, w));
                        }
                        Group { key: GroupKey::Class(k as u16), options }
                    }
                })
                .collect();
            let weight = if weight_binds { rnd(60) } else { 1 << 40 };
            let l = Limits { gold: rnd(40), pack_slots: rnd(4) as i32, weight };
            let s = solve(&groups, &l);
            let b = brute(&groups, &l);
            assert!(s.cost <= l.gold && s.slots <= l.pack_slots && s.weight <= l.weight, "{groups:?} {l:?} {s:?}");
            assert!(s.slots_met && s.weight_met);
            assert!(s.value <= b);
            assert!(s.bound >= brute(&groups, &Limits { weight: i64::MAX, ..l }), "the dual is a bound");
            if s.optimal {
                assert_eq!(s.value, b, "optimal claimed falsely: {groups:?} {l:?}");
                proved += 1;
            }
            if s.value < b {
                gaps += 1;
            }
        }
        (gaps, proved)
    }

    /// The gold and the slots binding, the weight slack: the sweep plus
    /// the fill. Measured at this commit: 12 gaps and 2,981 proved
    /// optimal in 3,000 (the sweep alone gapped on 1.9-5.8% in the
    /// review). The bounds below are regression lines, not guarantees.
    #[test]
    fn every_limit_holds_and_the_sweep_gap_is_rare() {
        let (gaps, proved) = fuzz(0xb2b2, 3000, false);
        assert!(gaps <= 30, "{gaps} gaps in 3000");
        assert!(proved >= 2900, "only {proved} proved");
    }

    /// The weight binding as well: the repair is a myopic repair, not an
    /// optimiser (P15's option 1), so the gaps are more frequent - 153 in
    /// 3,000 at this commit. Every limit still holds and "optimal" is
    /// still never claimed falsely.
    #[test]
    fn every_limit_holds_when_the_weight_binds() {
        let (gaps, _) = fuzz(0xb2b2, 3000, true);
        assert!(gaps <= 300, "{gaps} gaps in 3000");
    }

    /// The packed score's fields are enforced: past them the answer would
    /// be silently wrong, so the solver refuses.
    #[test]
    #[should_panic(expected = "past the score's cost field")]
    fn a_purse_past_the_score_field_is_refused() {
        let big = 1i64 << 48;
        let g = [class(0, &[(big, 1)]), class(1, &[(big, 1)])];
        solve(&g, &Limits { gold: big, pack_slots: 23, weight: 1_000_000 });
    }

    /// The ranking's ties: equal value and cost go to the fewer slots.
    #[test]
    fn the_ranking_prefers_fewer_slots() {
        let mut g = vec![class(0, &[(2, 9)]), class(1, &[(2, 9)])];
        g[1].options[1].slots = 0;
        let s = solve(&g, &limits(10));
        assert_eq!(s.ranked(&g), [(1, 1), (0, 1)]);
    }

    // --- the groups from the stock (B2 section 7) ---

    fn budget(gold: i64) -> Budget {
        let f = Facts { gold, pack_capacity: 23, home_capacity: 24, carry: 2000, weight: 0 };
        let s = build(&town([&[], &[], &[], &[], &[], &[], &[], &[]]), &[]);
        Budget::build(&f, &s, &[], |_| vec![])
    }

    /// A value table: `cap` copies, `per` each (flat then capped).
    fn capped(cap: i32, per: i64) -> impl Fn(&Candidate, i32) -> i64 {
        move |_, n| per * i64::from(n.min(cap))
    }

    /// The cap closes before the 1x copies run out: the 3x price is
    /// never in a table.
    #[test]
    fn the_black_market_is_not_paid_when_the_cap_closes_first() {
        let t = town([&[], &[], &[], &[], &[item(1, 2, 20)], &[], &[item(1, 3, 60)], &[]]);
        let s = build(&t, &[]);
        let g = groups(&s, &budget(1_000), 10, |_| None, capped(2, 50));
        assert_eq!(g.len(), 1);
        assert_eq!(g[0].options.len(), 3);
        assert!(g[0].options.iter().flat_map(|o| &o.picks).all(|&(i, _)| !s.candidates[i].is_black_market()));
    }

    /// A deficit of four: the third and fourth copies come from the
    /// Black Market, within the caution (per group: 120 * 10 <= 1200);
    /// a smaller purse truncates the table there.
    #[test]
    fn the_black_market_is_paid_within_the_caution_per_group() {
        let t = town([&[], &[], &[], &[], &[item(1, 2, 20)], &[], &[item(1, 3, 60)], &[]]);
        let s = build(&t, &[]);
        let g = groups(&s, &budget(1_200), 10, |_| None, capped(4, 50));
        assert_eq!(g[0].options.len(), 5);
        assert_eq!(g[0].options[4].cost, 20 + 20 + 60 + 60);
        // purse 1199: two BM copies (120) break the cap, one (60) does not
        let g = groups(&s, &budget(1_199), 10, |_| None, capped(4, 50));
        assert_eq!(g[0].options.len(), 4);
        // caution 0 disables the cap
        let g = groups(&s, &budget(1), 0, |_| None, capped(4, 50));
        assert_eq!(g[0].options.len(), 5);
    }

    /// Two rings of one kind with different bonuses are two classes,
    /// hence two groups - never one cheapest-n curve.
    #[test]
    fn a_split_class_is_two_groups() {
        let mut better = item(1, 1, 500);
        better.to_h = 5;
        let t = town([&[item(1, 1, 100), better], &[], &[], &[], &[], &[], &[], &[]]);
        let s = build(&t, &[]);
        let g = groups(&s, &budget(1_000), 10, |_| None, capped(1, 30));
        assert_eq!(g.len(), 2);
        assert_ne!(g[0].key, g[1].key);
    }

    /// Q1: two wearables for one equipment slot are one group, and at
    /// most one is ever chosen, whatever the purse.
    #[test]
    fn two_wearables_for_one_slot_are_never_both_chosen() {
        let t = town([&[], &[item(7, 1, 100), item(8, 1, 120)], &[], &[], &[], &[], &[], &[]]);
        let s = build(&t, &[]);
        let wear = |c: &Candidate| match c.kind {
            7 => Some((3u8, 40)),
            8 => Some((3u8, 45)),
            _ => None,
        };
        let g = groups(&s, &budget(10_000), 10, wear, |_, _| 0);
        assert_eq!(g.len(), 1);
        assert_eq!(g[0].key, GroupKey::Slot(3));
        let sol = solve(&g, &limits(10_000));
        assert_eq!(sol.chosen, [2], "the better armour, not both");
        assert_eq!(sol.value, 45);
    }

    /// The urgent needs' copies are gone before the tables are built.
    #[test]
    fn the_urgent_copies_are_removed() {
        let t = town([&[item(1, 2, 5)], &[], &[], &[], &[], &[], &[], &[]]);
        let s = build(&t, &[]);
        let mut b = budget(100);
        b.urgent = vec![UrgentPurchase { candidate: 0, needs: vec![] }];
        let g = groups(&s, &b, 10, |_| None, capped(5, 10));
        assert_eq!(g[0].options.len(), 2, "one copy left after the urgent need");
    }

    /// The cap is where the value reaches its maximum, not where it first
    /// fails to rise: with w = 25 and a gap of 50 the curve's rounding
    /// makes the first copy worth 0, and the class must not vanish.
    #[test]
    fn an_early_flat_step_does_not_end_the_table() {
        let t = town([&[item(1, 60, 1)], &[], &[], &[], &[], &[], &[], &[]]);
        let s = build(&t, &[]);
        let need = |_: &Candidate, n: i32| curve(25, 1, 50, i64::from(n));
        assert_eq!((need(&s.candidates[0], 1), need(&s.candidates[0], 2)), (0, 1));
        let g = groups(&s, &budget(1_000), 10, |_| None, need);
        assert_eq!(g.len(), 1, "the class is kept");
        let last = g[0].options.last().unwrap();
        assert_eq!(last.value, 25);
        assert_eq!(last.picks, [(0, 50)], "the table ends where the value reaches its maximum");
        assert!(g[0].options.windows(2).all(|w| w[1].value > w[0].value), "dominated counts dropped");
    }

    /// A class whose wares disagree on joining a carried stack counts the
    /// slot (the safe side).
    #[test]
    fn a_class_that_disagrees_counts_the_slot() {
        let t = town([&[item(1, 1, 10)], &[], &[], &[], &[item(1, 1, 12)], &[], &[], &[]]);
        let mut s = build(&t, &[]);
        assert_eq!(s.candidates[0].class, s.candidates[1].class);
        s.candidates[0].slots_cost = 0;
        let g = groups(&s, &budget(1_000), 10, |_| None, capped(2, 10));
        assert!(g[0].options.iter().skip(1).all(|o| o.slots == 1));
    }

    /// Q3, the surrogate pinned as intended: two kinds of one need, each
    /// priced against the whole gap (2), are both chosen in one solve;
    /// after the first purchase closes half the gap, the re-solve prices
    /// the second against what is left (1) and stops at the cap.
    #[test]
    fn the_surrogate_and_the_re_solve() {
        let t = town([&[], &[], &[], &[], &[item(1, 5, 20), item(2, 5, 30)], &[], &[], &[]]);
        let s = build(&t, &[]);
        let gap = std::cell::Cell::new(2i64);
        let need = |_: &Candidate, n: i32| curve(100, 1, gap.get(), i64::from(n));
        let g = groups(&s, &budget(1_000), 10, |_| None, need);
        let first = solve(&g, &Limits::of(&budget(1_000)));
        assert_eq!(first.chosen, [2, 2], "both kinds, each against the whole gap");
        // one copy bought: the gap is 1 now
        gap.set(1);
        let g = groups(&s, &budget(1_000), 10, |_| None, need);
        let again = solve(&g, &Limits::of(&budget(1_000)));
        let copies: i32 = again
            .chosen
            .iter()
            .enumerate()
            .flat_map(|(gi, &o)| g[gi].options[o].picks.iter().map(|&(_, k)| k))
            .sum();
        assert_eq!(copies, 2, "one copy per kind against the gap that is left");
    }
}
