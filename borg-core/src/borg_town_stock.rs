//! Phase 2 of the town inventory redesign (borg-rs#36): the stock, the
//! plan's input.
//!
//! The stock turns the world into three lists:
//!
//! * the **candidates** - every ware in every purchasable shop (0..6; the
//!   Home is not a shop), each with its per-copy price, the copies the shop
//!   holds, its per-copy weight, and the slot accounting: whether a copy
//!   stacks into a stack the pack already holds (`slots_cost` 0) or opens a
//!   new one (`slots_cost` 1), and how many copies of the kind the pack
//!   already carries;
//! * the **pack** and the **Home** as groups of stacks, keyed by kind - a
//!   kind can occupy more than one stack (two stacks that do not stack with
//!   each other), and the pack's slots are counted per distinct stack, not
//!   per copy (S3);
//! * the **alternatives**, kind-only (decided 2026-11-12, #35 item 5): every
//!   ware of a kind is an alternative source for that kind, ordered cheapest
//!   first, and the price of `n` copies is the cumulative cheapest-`n`
//!   across the shops. That is what makes the Black Market's 3x price
//!   meaningful: it binds only once the 1x copies run out. Effect
//!   equivalence across kinds is not attempted.
//!
//! The stacking predicate is a parameter. In production it is the Borg's
//! own `borg_object_similar` (`borg_store_sell.rs`), which also enforces
//! the kind's stack limit; the builder [`Stock::build`] takes it as a
//! closure so a test binary can run without the engine.
//!
//! Two known approximations, both on the safe side of the slot count:
//! ammunition is accounted against the pack, though the game puts it in
//! the quiver (whose occupied slots come off the pack's capacity, which
//! the budget reads through `BI_QUIVER_SLOTS`); and the stack
//! classes ask `borg_object_similar`, whose stack limit is checked
//! against the *held* stack's count, so two large shop stacks of one kind
//! can land in separate classes and count a slot each.
//!
//! What the stock does **not** carry: the reduction table that maps a ware
//! to the (need, reduction) pairs it serves. Its catalogue of useful kinds
//! is still open (#35 item 4), so it is not invented here; the candidates
//! carry kind, tval and sval so the table can key on them when it lands.
//! The urgent needs' classification ([`needs_served`]) is the one part of that
//! table the phase-2 budget needs, and it mirrors what `borg_trait` counts.
//!
//! Nothing in production calls this module yet; every public item carries
//! a self-removing `expect(dead_code)` marker, as phase 1's do.

use std::collections::BTreeMap;

use crate::borg_headers::borg_item_h::borg_item;
use crate::borg_store::BORG_HOME;
use crate::borg_town_usefulness::Need;
use crate::state::BorgState;

/// The Black Market's shop index (`feat_store(FEAT_STORE_BLACK)`, as
/// `borg_store_buy.rs` hard-codes it in its `who == 6` test).
pub const BORG_BLACK_MARKET: usize = 6;

/// One ware a shop offers: a purchase candidate.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Candidate {
    /// 0..6; the Home is not a shop.
    pub shop: u8,
    /// The ware's slot in that shop's stock.
    pub slot: u8,
    pub kind: u32,
    pub tval: u8,
    pub sval: u8,
    /// What this shop charges per copy (3x at the Black Market).
    pub price: i32,
    /// The copies the shop holds.
    pub qty: i32,
    /// The weight of one copy, in the game's tenth-pounds.
    pub weight: i32,
    /// 0 if a copy stacks into a stack the pack already holds, else 1.
    pub slots_cost: i32,
    /// How many copies of this kind the pack already carries (the pack
    /// only: a worn copy or a quiver stack is not counted).
    pub stack_count: i32,
    /// The stack class: wares of one kind that stack with each other (or
    /// into the same pack stack) share a class, so buying from several of
    /// them opens one slot, not one per ware.
    pub class: u16,
}

impl Candidate {
    /// Is this ware sold at the Black Market?
    pub fn is_black_market(&self) -> bool {
        usize::from(self.shop) == BORG_BLACK_MARKET
    }
}

/// One stack the pack or the Home holds.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Stack {
    /// The slot in the pack or in the Home's stock.
    pub slot: u8,
    pub kind: u32,
    pub tval: u8,
    pub sval: u8,
    pub qty: i32,
    /// The weight of one copy.
    pub weight: i32,
}

/// The stacks of one kind, in slot order.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Group {
    pub kind: u32,
    pub stacks: Vec<Stack>,
}

impl Group {
    /// The copies of the kind across its stacks.
    pub fn count(&self) -> i32 {
        self.stacks.iter().map(|s| s.qty).sum()
    }

    /// The slots the kind occupies: one per distinct stack.
    pub fn slots(&self) -> i32 {
        self.stacks.len() as i32
    }
}

/// The wares of one kind across the shops, cheapest first: the kind-only
/// alternatives.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct KindOffer {
    pub kind: u32,
    /// Indices into [`Stock::candidates`], ordered by (price, shop, slot).
    pub wares: Vec<usize>,
}

impl KindOffer {
    /// The copies of the kind the shops hold between them.
    #[cfg_attr(not(test), expect(dead_code))]
    pub fn available(&self, cands: &[Candidate]) -> i32 {
        self.wares.iter().map(|&i| cands[i].qty).sum()
    }

    /// Where `n` copies come from: the cheapest-first split, as
    /// (candidate index, copies) pairs. `None` if the shops do not hold
    /// `n` copies.
    #[cfg_attr(not(test), expect(dead_code))]
    pub fn split(&self, cands: &[Candidate], n: i32) -> Option<Vec<(usize, i32)>> {
        let mut left = n.max(0);
        let mut out = Vec::new();
        for &i in &self.wares {
            if left == 0 {
                break;
            }
            let take = left.min(cands[i].qty);
            if take > 0 {
                out.push((i, take));
                left -= take;
            }
        }
        (left == 0).then_some(out)
    }

    /// `cost_K(n)`: the cumulative cheapest-`n` price. `None` past the
    /// copies available.
    #[cfg_attr(not(test), expect(dead_code))]
    pub fn cost(&self, cands: &[Candidate], n: i32) -> Option<i64> {
        self.split(cands, n).map(|s| {
            s.iter()
                .map(|&(i, c)| i64::from(cands[i].price) * i64::from(c))
                .sum()
        })
    }

    /// The weight of `n` copies bought cheapest-first.
    #[cfg_attr(not(test), expect(dead_code))]
    pub fn weight(&self, cands: &[Candidate], n: i32) -> Option<i64> {
        self.split(cands, n).map(|s| {
            s.iter()
                .map(|&(i, c)| i64::from(cands[i].weight) * i64::from(c))
                .sum()
        })
    }

    /// The pack slots `n` copies bought cheapest-first open: one per
    /// distinct stack class in the split that no pack stack absorbs.
    #[cfg_attr(not(test), expect(dead_code))]
    pub fn slots_cost(&self, cands: &[Candidate], n: i32) -> Option<i32> {
        self.split(cands, n).map(|s| {
            let mut opened: Vec<u16> = s
                .iter()
                .map(|&(i, _)| &cands[i])
                .filter(|c| c.slots_cost != 0)
                .map(|c| c.class)
                .collect();
            opened.sort_unstable();
            opened.dedup();
            opened.len() as i32
        })
    }

    /// The table `cost_K(0..=n)`, stopping where the copies run out.
    #[cfg_attr(not(test), expect(dead_code))]
    pub fn cost_table(&self, cands: &[Candidate], n: i32) -> Vec<i64> {
        (0..=n).map_while(|k| self.cost(cands, k)).collect()
    }
}

/// The plan's input: the candidates, the pack and the Home as groups of
/// stacks, and the kind-only alternatives.
#[cfg_attr(not(test), expect(dead_code))]
#[derive(Clone, Debug, Default)]
pub struct Stock {
    /// Every ware of every purchasable shop, in (shop, slot) order.
    pub candidates: Vec<Candidate>,
    /// The pack's stacks by kind, in kind order.
    pub pack: Vec<Group>,
    /// The Home's stacks by kind, in kind order.
    pub home: Vec<Group>,
    /// The alternatives by kind, in kind order.
    pub offers: Vec<KindOffer>,
}

/// The stacks of a list of items, grouped by kind. Empty items are
/// skipped; the groups come out in kind order, the stacks in slot order.
fn groups(items: &[borg_item]) -> Vec<Group> {
    let mut by_kind: BTreeMap<u32, Vec<Stack>> = BTreeMap::new();
    for (slot, it) in items.iter().enumerate() {
        if it.iqty == 0 {
            continue;
        }
        by_kind.entry(it.kind).or_default().push(Stack {
            slot: slot as u8,
            kind: it.kind,
            tval: it.tval,
            sval: it.sval,
            qty: i32::from(it.iqty),
            weight: i32::from(it.weight),
        });
    }
    by_kind
        .into_iter()
        .map(|(kind, stacks)| Group { kind, stacks })
        .collect()
}

impl Stock {
    /// Build the stock from the shops' wares (index = shop number, Home
    /// included or not: the Home's index is skipped), the pack and the
    /// Home. `stacks_with(held, ware)` says whether one copy of `ware`
    /// joins the stack `held`; in production it is `borg_object_similar`.
    pub fn build(
        shops: &[&[borg_item]],
        pack: &[borg_item],
        home: &[borg_item],
        stacks_with: impl Fn(&borg_item, &borg_item) -> bool,
    ) -> Self {
        let pack_groups = groups(pack);
        let carried = |kind: u32| -> i32 {
            pack_groups
                .iter()
                .find(|g| g.kind == kind)
                .map_or(0, Group::count)
        };

        // The wares, with the per-ware slot cost against the pack.
        let mut raw: Vec<&borg_item> = Vec::new();
        let mut candidates = Vec::new();
        for (shop, wares) in shops.iter().enumerate() {
            if shop == BORG_HOME {
                continue;
            }
            for (slot, w) in wares.iter().enumerate() {
                if w.iqty == 0 {
                    continue;
                }
                let joins = pack
                    .iter()
                    .any(|p| p.iqty != 0 && p.kind == w.kind && stacks_with(p, w));
                raw.push(w);
                candidates.push(Candidate {
                    shop: shop as u8,
                    slot: slot as u8,
                    kind: w.kind,
                    tval: w.tval,
                    sval: w.sval,
                    price: w.cost,
                    qty: i32::from(w.iqty),
                    weight: i32::from(w.weight),
                    slots_cost: i32::from(!joins),
                    stack_count: carried(w.kind),
                    class: 0,
                });
            }
        }

        // The stack classes: a ware joins the class of the first earlier
        // ware of its kind it stacks with (either way round), else opens
        // its own. The classes are numbered in candidate order.
        let mut next: u16 = 0;
        for i in 0..candidates.len() {
            let earlier = (0..i).find(|&j| {
                candidates[j].kind == candidates[i].kind
                    && (stacks_with(raw[j], raw[i]) || stacks_with(raw[i], raw[j]))
            });
            candidates[i].class = match earlier {
                Some(j) => candidates[j].class,
                None => {
                    next += 1;
                    next - 1
                }
            };
        }

        // The alternatives: the wares of each kind, cheapest first.
        let mut by_kind: BTreeMap<u32, Vec<usize>> = BTreeMap::new();
        for (i, c) in candidates.iter().enumerate() {
            by_kind.entry(c.kind).or_default().push(i);
        }
        let offers = by_kind
            .into_iter()
            .map(|(kind, mut wares)| {
                wares.sort_by_key(|&i| {
                    let c = &candidates[i];
                    (c.price, c.shop, c.slot)
                });
                KindOffer { kind, wares }
            })
            .collect();

        Stock {
            candidates,
            pack: pack_groups,
            home: groups(home),
            offers,
        }
    }

    /// The stock the Borg sees now: the shops as `borg_notice_store` last
    /// read them, the pack, and the Home, with `borg_object_similar` as
    /// the stacking predicate.
    pub fn of(st: &BorgState) -> Self {
        let pack_slots = usize::from(crate::engine::z_info().pack_size);
        let shops: Vec<&[borg_item]> = st.store.shops.iter().map(|s| s.ware.as_slice()).collect();
        let home: &[borg_item] = st
            .store
            .shops
            .get(BORG_HOME)
            .map_or(&[], |s| s.ware.as_slice());
        let pack = &st.items.items[..pack_slots.min(st.items.items.len())];
        Self::build(&shops, pack, home, |held, ware| {
            crate::borg_store_sell::borg_object_similar(st, held, ware)
        })
    }

    /// The offer of a kind, if any shop sells it.
    #[cfg_attr(not(test), expect(dead_code))]
    pub fn offer(&self, kind: u32) -> Option<&KindOffer> {
        self.offers.iter().find(|o| o.kind == kind)
    }

    /// The pack slots the carried stacks occupy: one per distinct stack.
    pub fn pack_slots_used(&self) -> i32 {
        pack_slots_used_of_groups(&self.pack)
    }

    /// The Home slots its stacks occupy.
    pub fn home_slots_used(&self) -> i32 {
        self.home.iter().map(Group::slots).sum()
    }
}

/// The slots a list of groups occupies: one per distinct stack.
fn pack_slots_used_of_groups(groups: &[Group]) -> i32 {
    groups.iter().map(Group::slots).sum()
}

/// The pack slots a pack occupies, one per distinct stack, without
/// building the stock: the count the trip policy's pack pressure reads
/// (borg-rs#60), so the dungeon and the town agree when a pack is full.
pub fn pack_slots_used_of(pack: &[borg_item]) -> i32 {
    pack_slots_used_of_groups(&groups(pack))
}

/// The needs a ware meets now - what the urgent pass reads (the urgent needs
/// until 2026-11-14) - by the counting `borg_trait` does for the carried
/// kit: a light source (`TV_LIGHT`) for Light; a food `borg_trait` counts
/// in `BI_FOOD` for Food; Cure Light, Serious or Critical Wounds for
/// Bleeding; and Neutralize Poison, a poison-curing food or Cure Critical
/// Wounds for Poison (CCW added 2026-11-14).
pub fn needs_served(st: &BorgState, item: &borg_item) -> Vec<Need> {
    use crate::borg_item_analyze::borg_obj_has_effect;
    use crate::headers::list_effects_h::{EF_CRUNCH, EF_CURE, EF_NOURISH, EF_TIMED_INC};
    use crate::headers::list_player_timed_h::{TMD_CONFUSED, TMD_POISONED};
    use crate::headers::list_tvals_h::{TV_FOOD, TV_LIGHT, TV_MUSHROOM, TV_POTION, TV_SCROLL};

    let v = &st.item_val;
    let tval = u32::from(item.tval);
    let sval = i32::from(item.sval);
    let mut out = Vec::new();
    if tval == TV_SCROLL && sval == v.sv_scroll_satisfy_hunger {
        out.push(Need::Food);
    }
    if tval == TV_LIGHT && (item.timeout > 0 || crate::engine::flag_has(&item.flags,
        crate::headers::list_object_flags_h::OF_NO_FUEL as i32)) {
        out.push(Need::Light);
    }
    if tval == TV_FOOD || tval == TV_MUSHROOM {
        let capability_mode = st.cfg(crate::borg_headers::borg_h::BORG_TOWN_PLAN) == 1;
        let harmful = (!capability_mode && borg_obj_has_effect(item.kind, EF_CRUNCH as i32, -1))
            || borg_obj_has_effect(item.kind, EF_TIMED_INC as i32, TMD_CONFUSED as i32);
        let nourishes = if capability_mode {
            borg_obj_has_effect(item.kind, EF_NOURISH as i32, 0)
                || borg_obj_has_effect(item.kind, EF_NOURISH as i32, 3)
        } else if tval == TV_MUSHROOM {
            borg_obj_has_effect(item.kind, EF_NOURISH as i32, 0)
        } else {
            [
                v.sv_food_apple,
                v.sv_food_handful,
                v.sv_food_slime_mold,
                v.sv_food_pint,
                v.sv_food_sip,
                v.sv_food_ration,
                v.sv_food_slice,
                v.sv_food_honey_cake,
                v.sv_food_waybread,
                v.sv_food_draught,
            ]
            .contains(&sval)
        };
        if !harmful && nourishes {
            out.push(Need::Food);
        }
        if !harmful && borg_obj_has_effect(item.kind, EF_CURE as i32, TMD_POISONED as i32) {
            out.push(Need::Poison);
        }
    }
    if tval == TV_POTION {
        if sval == v.sv_potion_cure_light
            || sval == v.sv_potion_cure_serious
            || sval == v.sv_potion_cure_critical
            || sval == v.sv_potion_healing
            || sval == v.sv_potion_star_healing
            || sval == v.sv_potion_life
        {
            out.push(Need::Healing);
        }
        if [v.sv_potion_cure_poison, v.sv_potion_cure_critical, v.sv_potion_healing, v.sv_potion_star_healing, v.sv_potion_life].contains(&sval) {
            out.push(Need::Poison);
        }
    }
    out
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::borg_item::blank_item;

    /// An item of `kind` with `qty` copies at `cost` per copy, weight 10.
    pub(crate) fn item(kind: u32, qty: u8, cost: i32) -> borg_item {
        let mut it = blank_item();
        it.kind = kind;
        it.tval = kind as u8;
        it.iqty = qty;
        it.cost = cost;
        it.weight = 10;
        it
    }

    /// The test stacking rule: same kind, same `to_h` (the variant), and
    /// under a stack limit of 40.
    pub(crate) fn similar(held: &borg_item, ware: &borg_item) -> bool {
        held.kind == ware.kind && held.to_h == ware.to_h && i32::from(held.iqty) + 1 < 40
    }

    /// A shop row of `n` slots holding `wares` from slot 0.
    pub(crate) fn shop(wares: &[borg_item]) -> Vec<borg_item> {
        let mut row = vec![blank_item(); 24];
        row[..wares.len()].copy_from_slice(wares);
        row
    }

    /// Eight shops; shop 6 is the Black Market, shop 7 the Home.
    pub(crate) fn town(per_shop: [&[borg_item]; 8]) -> Vec<Vec<borg_item>> {
        per_shop.iter().map(|w| shop(w)).collect()
    }

    pub(crate) fn build(town: &[Vec<borg_item>], pack: &[borg_item]) -> Stock {
        let shops: Vec<&[borg_item]> = town.iter().map(Vec::as_slice).collect();
        Stock::build(&shops, pack, &town[BORG_HOME], similar)
    }

    /// Every ware of every shop is a candidate, the Home's are not, and
    /// empty slots are skipped.
    #[test]
    fn every_ware_of_every_shop_is_a_candidate() {
        let t = town([
            &[item(1, 5, 10), item(2, 1, 50)],
            &[],
            &[item(3, 1, 100)],
            &[],
            &[],
            &[],
            &[item(1, 2, 30)],
            &[item(4, 1, 0)],
        ]);
        let s = build(&t, &[]);
        let got: Vec<(u8, u8, u32)> = s.candidates.iter().map(|c| (c.shop, c.slot, c.kind)).collect();
        assert_eq!(got, [(0, 0, 1), (0, 1, 2), (2, 0, 3), (6, 0, 1)]);
        assert!(s.candidates[3].is_black_market());
        assert!(!s.candidates[0].is_black_market());
        assert_eq!(s.home.len(), 1);
        assert_eq!(s.home[0].kind, 4);
        assert_eq!(s.home_slots_used(), 1);
    }

    /// The alternatives are kind-only and cumulative cheapest-n: the 1x
    /// copies first, the Black Market's 3x only once they run out.
    #[test]
    fn alternatives_are_cumulative_cheapest_n() {
        let t = town([&[item(1, 2, 10)], &[], &[], &[], &[item(1, 1, 12)], &[], &[item(1, 5, 30)], &[]]);
        let s = build(&t, &[]);
        let o = s.offer(1).expect("kind 1 is offered");
        let c = &s.candidates;
        assert_eq!(o.available(c), 8);
        assert_eq!(o.cost_table(c, 10), [0, 10, 20, 32, 62, 92, 122, 152, 182]);
        assert_eq!(o.cost(c, 9), None);
        assert_eq!(o.split(c, 4), Some(vec![(0, 2), (1, 1), (2, 1)]));
        assert_eq!(o.weight(c, 4), Some(40));
        // the ware order is (price, shop, slot), whatever the shop order
        let prices: Vec<i32> = o.wares.iter().map(|&i| c[i].price).collect();
        assert_eq!(prices, [10, 12, 30]);
    }

    /// Equal prices fall back to the shop and the slot, so the split is
    /// deterministic.
    #[test]
    fn alternatives_break_ties_by_shop_then_slot() {
        let t = town([&[], &[], &[item(9, 1, 5), item(9, 1, 5)], &[], &[], &[item(9, 1, 5)], &[], &[]]);
        let s = build(&t, &[]);
        let o = s.offer(9).unwrap();
        let order: Vec<(u8, u8)> = o.wares.iter().map(|&i| (s.candidates[i].shop, s.candidates[i].slot)).collect();
        assert_eq!(order, [(2, 0), (2, 1), (5, 0)]);
    }

    /// The pack is a list of groups of stacks: one kind can hold two
    /// stacks, and the count is across them.
    #[test]
    fn the_pack_is_groups_of_stacks() {
        let mut variant = item(1, 2, 0);
        variant.to_h = 3;
        let pack = [item(1, 5, 0), item(2, 1, 0), variant];
        let s = build(&town([&[], &[], &[], &[], &[], &[], &[], &[]]), &pack);
        assert_eq!(s.pack.len(), 2);
        assert_eq!(s.pack[0].kind, 1);
        assert_eq!(s.pack[0].count(), 7);
        assert_eq!(s.pack[0].slots(), 2);
        assert_eq!(s.pack_slots_used(), 3);
    }

    /// The per-ware slot cost: 0 when a copy joins a pack stack, 1 when
    /// it opens one; the carried count rides along.
    #[test]
    fn a_ware_that_stacks_into_the_pack_costs_no_slot() {
        let mut variant = item(1, 1, 10);
        variant.to_h = 3;
        let t = town([&[item(1, 5, 10), variant, item(2, 1, 10)], &[], &[], &[], &[], &[], &[], &[]]);
        let s = build(&t, &[item(1, 4, 0)]);
        let cost: Vec<(i32, i32)> = s.candidates.iter().map(|c| (c.slots_cost, c.stack_count)).collect();
        assert_eq!(cost, [(0, 4), (1, 4), (1, 0)]);
    }

    /// A full pack stack accepts nothing more (the predicate carries the
    /// stack limit), so its ware opens a slot.
    #[test]
    fn a_full_pack_stack_does_not_absorb_a_ware() {
        let t = town([&[item(1, 5, 10)], &[], &[], &[], &[], &[], &[], &[]]);
        let s = build(&t, &[item(1, 39, 0)]);
        assert_eq!(s.candidates[0].slots_cost, 1);
    }

    /// Buying one kind from several shops opens one slot per distinct
    /// stack class, not one per shop; a class that joins the pack opens
    /// none.
    #[test]
    fn a_kind_from_several_shops_opens_one_slot_per_stack_class() {
        let mut variant = item(1, 1, 40);
        variant.to_h = 3;
        let t = town([&[item(1, 1, 10)], &[], &[], &[], &[item(1, 1, 20)], &[], &[variant], &[]]);
        let s = build(&t, &[]);
        let o = s.offer(1).unwrap();
        let c = &s.candidates;
        assert_eq!(c[0].class, c[1].class);
        assert_ne!(c[0].class, c[2].class);
        assert_eq!(o.slots_cost(c, 0), Some(0));
        assert_eq!(o.slots_cost(c, 2), Some(1));
        assert_eq!(o.slots_cost(c, 3), Some(2));
        // with a pack stack of the plain kind, only the variant opens one
        let s = build(&t, &[item(1, 1, 0)]);
        let o = s.offer(1).unwrap();
        assert_eq!(o.slots_cost(&s.candidates, 2), Some(0));
        assert_eq!(o.slots_cost(&s.candidates, 3), Some(1));
    }
}
