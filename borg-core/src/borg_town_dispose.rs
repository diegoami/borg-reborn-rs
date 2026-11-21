//! Phase 4 of the town inventory redesign (borg-rs#36), slice 3: the
//! dispositions - for every stack carried and every stack in the Home,
//! keep, store, sell, drop or take.
//!
//! The rule is the owner's (2026-11-14): **value-ranked**.
//!
//! * a stack's **keep value** is the usefulness lost by not having it: its
//!   need value (the need curve with the stack against without it) plus
//!   its power value (the power `borg_power` loses without it, priced by
//!   the power need's curve). The caller supplies the raw facts - the
//!   need, the reduction, the power - since the power comes from the
//!   engine (a try-off, in production);
//! * **unknown** wearables and devices are kept until they are known (by
//!   use, a rune, or buying a copy) - stored in the Home if the weight
//!   asks for room; unknown potions and scrolls are sold, as today;
//! * a stack with **value** is kept; one **without** is sold if a shop
//!   buys it, else dropped; a stack bought this visit is never sold back
//!   (#37's Stay row);
//! * over the **weight** allowance, the kept stacks with the least value
//!   per weight go to the Home while it has slots;
//! * **Home** stacks with value are taken back while the pack has slots,
//!   the most valuable first; Home stacks without value are sold only when
//!   the sale pays gold, or when the Home needs the slot (owner,
//!   2026-11-14: under `birth_no_selling` a sale pays nothing, and the Home
//!   is a free stash).
//!
//! Two stages (owner, 2026-11-14): the dispositions are decided first,
//! and what they free - pack slots, weight, the sale proceeds - is what
//! the purchases are solved with. The joint form, where a sale is decided
//! by what its gold would buy, is phase 5. Under `birth_no_selling` (the
//! game's default) a sale pays nothing; it still frees the slot.

use crate::borg_town_usefulness::{usefulness, value, Need, Trip, Weights};
use crate::state::BorgState;

/// Where a stack is.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Place {
    Pack,
    Home,
}

/// What happens to a stack.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Disposition {
    /// Stays where it is (in the pack, or in the Home).
    Keep,
    /// From the pack into the Home.
    Store,
    /// To a shop (`KitItem::sell_shop`).
    Sell,
    /// On the floor: no shop buys it.
    Drop,
    /// From the Home into the pack.
    Take,
}

/// The facts of one stack the dispositions read.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct KitItem {
    /// Schema 9: the whole withdrawal can join existing carried stock.
    pub joins_pack: bool,
    pub place: Place,
    /// The slot in the pack, or in the Home's stock.
    pub slot: u8,
    pub kind: u32,
    pub qty: i32,
    /// The weight of one copy.
    pub weight: i32,
    /// The need it serves (an index into [`Need::ALL`]), -1 for none.
    pub need: i8,
    /// What one copy closes of that need.
    pub reduction: i64,
    /// v5 secondary capabilities, independent of primary utility units.
    pub serves: u8,
    /// The power it is worth: for a pack stack, what `borg_power` loses
    /// without it; for a Home stack, what it gains with it in the pack.
    pub power: i64,
    /// What a shop pays for the stack (0 under `birth_no_selling`).
    pub proceeds: i64,
    /// The shop that buys it, -1 for none.
    pub sell_shop: i8,
    /// Not yet known: a wearable or a device whose kind or runes are not.
    pub unknown: bool,
    /// An unknown potion or scroll: sold, as today.
    pub unknown_flavour: bool,
    /// Bought this visit: never sold back.
    pub just_bought: bool,
}

/// What the dispositions free for the purchases (the second stage).
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Freed {
    /// Pack slots: stacks stored, sold or dropped, less stacks taken.
    pub pack_slots: i32,
    /// Home slots: stacks sold from the Home, less stacks stored.
    pub home_slots: i32,
    /// Weight: stored, sold or dropped, less taken.
    pub weight: i64,
    /// Gold from the sales.
    pub gold: i64,
}

/// The keep value of a stack: its need value plus its power value.
///
/// The need value is the need curve with the stack against without it:
/// for a counted need (light, food, escape) the stack's copies are added
/// back to the deficit; for a condition need (bleeding, poison) the stack
/// is worth the whole need while the condition has no other cure.
pub fn keep_value(item: &KitItem, st: &BorgState, trip: &Trip, weights: &Weights) -> i64 {
    let need = usize::try_from(item.need).ok().and_then(|i| Need::ALL.get(i)).copied();
    let need_value = match need {
        Some(n @ (Need::Light | Need::Food | Need::Escape | Need::Healing | Need::Trip)) => {
            // The curve's value with the copies against without them: a
            // carried stack is worth what losing it costs, a Home stack what
            // having it adds; copies past the target are worth nothing.
            let (carried, target) = crate::borg_town_usefulness::counted(n, st, trip).unwrap_or((0, 0));
            let u = |x: i64| value(weights.get(n), 1, target, x.max(0));
            let copies = item.reduction * i64::from(item.qty);
            let mut v = match item.place {
                Place::Pack => u(carried) - u(carried - copies),
                Place::Home => u(carried + copies) - u(carried),
            };
            // Healing is also urgent while cut with no cure: the only cure
            // carried keeps the whole need.
            if n == Need::Healing && item.place == Place::Pack {
                use crate::borg_headers::borg_trait_h::BI_ISCUT;
                let others = crate::borg_town_usefulness::wound_cures(st, trip) - i32::try_from(copies).unwrap_or(i32::MAX);
                if st.borg.tr(BI_ISCUT) != 0 && others < 1 {
                    v = v.max(weights.get(n));
                }
            }
            v
        }
        // A condition need: worth the whole need while the condition is on
        // and, without this stack, nothing else cures it. A carried cure is
        // in the cure count itself, so its own copies come off first.
        Some(n @ Need::Poison) => {
            use crate::borg_headers::borg_trait_h::BI_ISPOISONED;
            use crate::borg_town_usefulness::poison_cures;
            let (on, cures) = (st.borg.tr(BI_ISPOISONED) != 0, poison_cures(st, trip));
            let own = if item.place == Place::Pack { item.qty } else { 0 };
            if on && cures - own < 1 {
                weights.get(n)
            } else {
                0
            }
        }
        _ => 0,
    };
    let power_value = if item.power > 0 { usefulness(Need::Power, item.power, 1, st, trip, weights) } else { 0 };
    need_value + power_value
}

/// The dispositions of the pack and the Home, and what they free.
/// `pack_free` and `home_free` are the free slots now; `weight_room` is
/// the carry allowance less the carried weight (negative: over it).
pub fn dispose(
    kit: &[KitItem],
    st: &BorgState,
    trip: &Trip,
    weights: &Weights,
    pack_free: i32,
    home_free: i32,
    weight_room: i64,
) -> (Vec<Disposition>, Freed) {
    let values: Vec<i64> = kit.iter().map(|it| keep_value(it, st, trip, weights)).collect();
    let mut out = vec![Disposition::Keep; kit.len()];
    let mut freed = Freed::default();

    // The pack: keep, sell or drop.
    for (i, it) in kit.iter().enumerate().filter(|(_, it)| it.place == Place::Pack) {
        let keep = it.unknown || it.just_bought || (values[i] > 0 && !it.unknown_flavour);
        if keep {
            continue;
        }
        out[i] = if it.sell_shop >= 0 { Disposition::Sell } else { Disposition::Drop };
        freed.pack_slots += 1;
        freed.weight += i64::from(it.weight) * i64::from(it.qty);
        if out[i] == Disposition::Sell {
            freed.gold += it.proceeds;
        }
    }

    // The Home: a worthless stack is sold only when the sale pays gold
    // (owner, 2026-11-14) - under birth_no_selling it pays nothing, and the
    // Home is a free stash - or below, when the Home needs the slot.
    let worthless = |i: usize, it: &KitItem| values[i] <= 0 && !it.unknown && it.sell_shop >= 0;
    for (i, it) in kit.iter().enumerate().filter(|(_, it)| it.place == Place::Home) {
        if worthless(i, it) && it.proceeds > 0 {
            out[i] = Disposition::Sell;
            freed.home_slots += 1;
            freed.gold += it.proceeds;
        }
    }
    let mut home_free = home_free + freed.home_slots;
    // The worthless Home stacks still kept: sold to make room for a store.
    let mut spare: Vec<usize> = (0..kit.len())
        .filter(|&i| kit[i].place == Place::Home && out[i] == Disposition::Keep && worthless(i, &kit[i]))
        .collect();
    spare.reverse();

    // Over the allowance: store the kept stacks with the least value per
    // weight, while the Home has slots. Ties to the earlier slot.
    let mut over = -weight_room - freed.weight;
    if over > 0 {
        let mut kept: Vec<usize> = (0..kit.len())
            .filter(|&i| kit[i].place == Place::Pack && out[i] == Disposition::Keep && kit[i].weight > 0)
            .collect();
        kept.sort_by(|&a, &b| {
            let (wa, wb) = (i64::from(kit[a].weight) * i64::from(kit[a].qty), i64::from(kit[b].weight) * i64::from(kit[b].qty));
            (i128::from(values[a]) * i128::from(wb))
                .cmp(&(i128::from(values[b]) * i128::from(wa)))
                .then(kit[a].slot.cmp(&kit[b].slot))
        });
        for i in kept {
            if over <= 0 {
                break;
            }
            if home_free <= 0 {
                // Make room: sell a worthless Home stack, if there is one.
                let Some(j) = spare.pop() else { break };
                out[j] = Disposition::Sell;
                freed.home_slots += 1;
                freed.gold += kit[j].proceeds;
                home_free += 1;
            }
            out[i] = Disposition::Store;
            let w = i64::from(kit[i].weight) * i64::from(kit[i].qty);
            over -= w;
            freed.weight += w;
            freed.pack_slots += 1;
            freed.home_slots -= 1;
            home_free -= 1;
        }
    }

    // Take back the Home's valuable stacks, the most valuable first,
    // while the pack has slots (and the weight allows). Ties to the
    // earlier slot.
    let mut pack_free = pack_free + freed.pack_slots;
    let mut room = -over;
    let mut wanted: Vec<usize> = (0..kit.len())
        .filter(|&i| kit[i].place == Place::Home && out[i] == Disposition::Keep && values[i] > 0)
        .collect();
    wanted.sort_by(|&a, &b| values[b].cmp(&values[a]).then(kit[a].slot.cmp(&kit[b].slot)));
    for i in wanted {
        let w = i64::from(kit[i].weight) * i64::from(kit[i].qty);
        if pack_free <= 0 || w > room {
            continue;
        }
        out[i] = Disposition::Take;
        pack_free -= 1;
        room -= w;
        freed.pack_slots -= 1;
        freed.home_slots += 1;
        freed.weight -= w;
    }
    (out, freed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::borg_headers::borg_trait_h::{BI_ACLW, BI_FOOD, BI_ISCUT, BI_MAX, BI_MAXDEPTH, BI_RECALL};
    use crate::borg_init::BORG_SETTINGS;

    fn state() -> BorgState {
        let mut st = BorgState::default();
        st.core.cfg = BORG_SETTINGS.iter().map(|s| s.default_value).collect();
        st.borg.trait_ = vec![0; BI_MAX as usize];
        st.borg.trait_[BI_MAXDEPTH as usize] = 1;
        st
    }

    fn item(place: Place, slot: u8, need: i8, qty: i32) -> KitItem {
        KitItem {
            joins_pack: false,
            place,
            slot,
            kind: u32::from(slot) + 100,
            qty,
            weight: 10,
            need,
            reduction: 1,
            serves: 0,
            power: 0,
            proceeds: 5,
            sell_shop: 0,
            unknown: false,
            unknown_flavour: false,
            just_bought: false,
        }
    }

    fn run(kit: &[KitItem], st: &BorgState, pack_free: i32, home_free: i32, room: i64) -> (Vec<Disposition>, Freed) {
        let trip = Trip::of(st);
        let w = Weights::of(&st.core.cfg);
        dispose(kit, st, &trip, &w, pack_free, home_free, room)
    }

    /// The rations the food need wants are kept; a stack serving nothing
    /// is sold, or dropped when no shop buys it.
    #[test]
    fn useful_is_kept_useless_is_sold_or_dropped() {
        let mut st = state();
        st.borg.trait_[BI_FOOD as usize] = 3; // 3 of 10 rations carried
        let mut junk = item(Place::Pack, 1, -1, 1);
        let mut unsellable = item(Place::Pack, 2, -1, 1);
        unsellable.sell_shop = -1;
        let kit = [item(Place::Pack, 0, 1, 3), junk, unsellable];
        let (d, f) = run(&kit, &st, 5, 5, 1000);
        assert_eq!(d, [Disposition::Keep, Disposition::Sell, Disposition::Drop]);
        assert_eq!(f, Freed { pack_slots: 2, home_slots: 0, weight: 20, gold: 5 });
        // power alone keeps a stack (a spellbook, a wand)
        junk.power = 50_000;
        let (d, _) = run(&[junk], &st, 5, 5, 1000);
        assert_eq!(d, [Disposition::Keep]);
    }

    /// Unknown wearables and devices stay until known; unknown potions
    /// and scrolls are sold; what was just bought is never sold back.
    #[test]
    fn unknowns_and_just_bought() {
        let st = state();
        let mut unknown = item(Place::Pack, 0, -1, 1);
        unknown.unknown = true;
        let mut flavour = item(Place::Pack, 1, -1, 1);
        flavour.unknown_flavour = true;
        flavour.power = 10_000; // valued, but unknown: sold to learn it
        let mut bought = item(Place::Pack, 2, -1, 1);
        bought.just_bought = true;
        let (d, _) = run(&[unknown, flavour, bought], &st, 5, 5, 1000);
        assert_eq!(d, [Disposition::Keep, Disposition::Sell, Disposition::Keep]);
    }

    /// Wound potions are a trip's healing stock (5): kept below it, the
    /// surplus beyond it sold; the only cure is kept while cut.
    #[test]
    fn healing_potions_are_a_stock_and_the_only_cure_is_kept_while_cut() {
        let mut st = state();
        let cure = item(Place::Pack, 0, 2, 1);
        st.borg.trait_[BI_ACLW as usize] = 1; // the carried one counts itself
        let (d, _) = run(&[cure], &st, 5, 5, 1000);
        assert_eq!(d, [Disposition::Keep], "below the stock: kept");
        // six carried, this stack one of them: past the stock, no use
        st.borg.trait_[BI_ACLW as usize] = 6;
        let (d, _) = run(&[cure], &st, 5, 5, 1000);
        assert_eq!(d, [Disposition::Sell], "the surplus is sold");
        // a stack of 3 where 7 are carried: 2 of them are surplus, but the
        // stack still fills the stock, so it is kept whole
        let mut three = cure;
        three.qty = 3;
        st.borg.trait_[BI_ACLW as usize] = 7;
        let (d, _) = run(&[three], &st, 5, 5, 1000);
        assert_eq!(d, [Disposition::Keep]);
        // cut, and this is the only cure: kept whatever the stock says
        let mut st = state();
        st.borg.trait_[BI_ISCUT as usize] = 1;
        st.borg.trait_[BI_ACLW as usize] = 1;
        let (d, _) = run(&[cure], &st, 5, 5, 1000);
        assert_eq!(d, [Disposition::Keep], "the only cure is never sold");
    }

    /// A worthless Home stack is sold only for gold or room: under
    /// no_selling it stays, unless a store needs its slot.
    #[test]
    fn a_home_stack_is_sold_only_for_gold_or_room() {
        let st = state();
        let mut junk = item(Place::Home, 0, -1, 1);
        junk.proceeds = 0;
        let (d, _) = run(&[junk], &st, 5, 0, 1000);
        assert_eq!(d, [Disposition::Keep], "no gold, no room wanted: stays");
        // the pack is overweight with a low-value kept stack, the Home full
        let mut st = state();
        st.borg.trait_[BI_FOOD as usize] = 1; // the carried ration itself
        let mut heavy = item(Place::Pack, 1, 1, 1);
        heavy.weight = 2000;
        let (d, f) = run(&[junk, heavy], &st, 5, 0, -1900);
        assert_eq!(d, [Disposition::Sell, Disposition::Store], "{f:?}");
    }

    /// A Recall scroll is the trip need: the pack's is kept, a spare in
    /// the Home taken back when none is carried.
    #[test]
    fn recall_is_kept_and_taken_back() {
        let mut st = state();
        let trip_need = Need::ALL.iter().position(|&n| n == Need::Trip).unwrap() as i8;
        st.borg.trait_[BI_RECALL as usize] = 1;
        let carried = item(Place::Pack, 0, trip_need, 1);
        let (d, _) = run(&[carried], &st, 5, 5, 1000);
        assert_eq!(d, [Disposition::Keep]);
        st.borg.trait_[BI_RECALL as usize] = 0;
        let spare = item(Place::Home, 1, trip_need, 1);
        let (d, _) = run(&[spare], &st, 5, 5, 1000);
        assert_eq!(d, [Disposition::Take]);
    }

    /// Over the weight allowance, the kept stacks with the least value per
    /// weight go to the Home while it has slots.
    #[test]
    fn overweight_stores_the_least_value_per_weight() {
        let st = state();
        let mut a = item(Place::Pack, 0, -1, 1);
        a.power = 200_000; // worth 50
        a.weight = 100;
        let mut b = item(Place::Pack, 1, -1, 1);
        b.power = 40_000; // worth 10
        b.weight = 100;
        let mut c = item(Place::Pack, 2, -1, 1);
        c.power = 40_000; // worth 10, light
        c.weight = 10;
        let (d, f) = run(&[a, b, c], &st, 0, 5, -50);
        assert_eq!(d, [Disposition::Keep, Disposition::Store, Disposition::Keep]);
        assert_eq!((f.pack_slots, f.home_slots, f.weight), (1, -1, 100));
        // no Home slot: nothing can be stored
        let (d, _) = run(&[a, b, c], &st, 0, 0, -50);
        assert_eq!(d, [Disposition::Keep; 3]);
    }

    /// The Home: valuable stacks come back while the pack has room, the
    /// most valuable first; worthless ones are sold.
    #[test]
    fn the_home_gives_back_what_is_wanted() {
        let mut st = state();
        st.borg.trait_[BI_FOOD as usize] = 0;
        let rations = item(Place::Home, 0, 1, 5); // food: wanted
        let mut armour = item(Place::Home, 1, -1, 1);
        armour.power = 400_000; // worth 50
        let junk = item(Place::Home, 2, -1, 1);
        let (d, f) = run(&[rations, armour, junk], &st, 1, 0, 1000);
        // one pack slot: the rations (5 of a 10-ration gap, worth 50) and
        // the armour (a trip's preparation, worth 50) tie; the earlier
        // slot wins. The worthless stack is sold.
        assert_eq!(d, [Disposition::Take, Disposition::Keep, Disposition::Sell]);
        assert_eq!((f.pack_slots, f.home_slots, f.gold), (-1, 2, 5));
    }
}
