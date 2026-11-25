//! Town restocking when the known healing potion reserve runs low (#1214).
use crate::borg_headers::borg_h::BORG_TOWN_NO_STORES;
use crate::borg_headers::borg_item_h::borg_item;
use crate::borg_headers::borg_trait_h::{
    BI_CDEPTH, BI_FOOD, BI_GOLD, BI_ISCUT, BI_ISPOISONED, BI_LIGHT, BI_RECALL,
};
use crate::borg_store::BORG_HOME;
use crate::borg_think_dungeon::GOAL_BORE;
use crate::borg_town_record::action;
use crate::borg_town_shadow::TownAction;
use crate::headers::list_options_h::{OPT_birth_force_descend, OPT_birth_no_recall};
use crate::headers::list_terrain_h::FEAT_LESS;
use crate::headers::list_tvals_h::TV_POTION;
use crate::params::{value, Param};
use crate::{engine, state::BorgState};

#[derive(Clone, Copy, Default, PartialEq, Eq)]
enum Phase {
    #[default]
    NoReserve,
    Stocked,
    Returning {
        started_with: i32,
    },
    Shopping,
    Deferred,
}

#[derive(Default)]
pub(crate) struct Restock {
    phase: Phase,
    route_attempt: Option<(i32, i32)>,
    stair_commit: Option<u32>,
    town: TownSupply,
}

#[derive(Default)]
struct TownSupply {
    offers: Vec<(i32, i32)>,
    departure_gold: i32,
    departure_stock: i32,
}

impl TownSupply {
    fn refill_cost(&self, mut missing: i32) -> Option<i32> {
        let mut cost: i32 = 0;
        for &(price, available) in &self.offers {
            let count = missing.min(available);
            cost = cost.saturating_add(price.saturating_mul(count));
            missing -= count;
            if missing == 0 {
                return Some(cost);
            }
        }
        None
    }

    fn can_refill(&self, missing: i32, gold: i32, minimum: i32, failed_visit: bool) -> bool {
        let Some(cost) = self.refill_cost(missing) else {
            return false;
        };
        if cost == 0 {
            // A free withdrawal can justify a first trip. If that visit
            // left it behind, the same offer cannot justify a repeat.
            return !failed_visit;
        }
        gold >= cost
            && ((!failed_visit && self.departure_stock >= minimum)
                || self.departure_gold < cost
                || gold.saturating_sub(self.departure_gold) >= cost)
    }
}

impl Restock {
    fn observe_reserve(&mut self, held: i32, minimum: i32, gold: i32) {
        let old = self.phase;
        self.phase = match old {
            _ if held >= minimum => Phase::Stocked,
            // Keep a committed return across levels while still short.
            Phase::Returning { .. } => old,
            Phase::Shopping => Phase::Deferred,
            // A reserve that ran out calls for one visit even when town's
            // offers or the Borg's ability to fund them are uncertain.
            Phase::Stocked if held == 0 => Phase::Returning { started_with: held },
            _ => {
                let failed_visit = old == Phase::Deferred;
                let missing = if failed_visit && held == 0 {
                    1
                } else {
                    minimum - held
                };
                if self.town.can_refill(missing, gold, minimum, failed_visit) {
                    Phase::Returning { started_with: held }
                } else if old == Phase::NoReserve && held > 0 {
                    Phase::Stocked
                } else {
                    old
                }
            }
        };
    }
}

fn healing_potion(st: &BorgState, item: &borg_item) -> bool {
    let v = &st.item_val;
    item.iqty > 0
        && u32::from(item.tval) == TV_POTION
        && [
            v.sv_potion_cure_light,
            v.sv_potion_cure_serious,
            v.sv_potion_cure_critical,
            v.sv_potion_healing,
            v.sv_potion_star_healing,
            v.sv_potion_life,
        ]
        .contains(&i32::from(item.sval))
}

pub(crate) fn carried_potions(st: &BorgState) -> i32 {
    // The quaff command uses known pack items. The healing traits can also
    // count spells and device charges, which cannot replace a potion here.
    st.items
        .items
        .iter()
        .take(crate::borg_item::inven_wield() as usize)
        .filter(|item| item.aware && healing_potion(st, item))
        .map(|item| i32::from(item.iqty))
        .sum()
}

fn enabled(st: &BorgState) -> bool {
    value(Param::restock_healing) != 0 && st.cfg(BORG_TOWN_NO_STORES) == 0
}

fn minimum() -> i32 {
    value(Param::restock_healing_min_potions)
}

pub(crate) fn carries_unknown_potion(st: &BorgState) -> bool {
    st.items
        .items
        .iter()
        .take(crate::borg_item::inven_wield() as usize)
        .any(|item| item.iqty > 0 && u32::from(item.tval) == TV_POTION && !item.aware)
}

pub(crate) fn observe_dungeon(st: &mut BorgState) {
    if st.borg.tr(BI_CDEPTH) == 0 || !enabled(st) {
        return;
    }
    let held = carried_potions(st);
    let gold = st.borg.tr(BI_GOLD);
    let restock = &mut st.think_dungeon.healing_restock;
    let old = restock.phase;
    restock.observe_reserve(held, minimum(), gold);
    if matches!(old, Phase::Returning { .. }) && restock.phase == Phase::Stocked {
        clear_stair_route(st);
    } else if !matches!(old, Phase::Returning { .. }) && returning(st) {
        crate::borg_io::borg_note(
            st,
            &format!(
                "# Returning to town: low on healing potions ({held}/{} carried)",
                minimum()
            ),
        );
        if st.borg.goal.type_0 == GOAL_BORE as i16 {
            st.borg.goal.type_0 = 0;
        }
    }
}

fn clear_stair_route(st: &mut BorgState) {
    if st.think_dungeon.healing_restock.stair_commit.take() == Some(st.flow.commit_seq)
        && st.borg.goal.type_0 == GOAL_BORE as i16
    {
        st.borg.goal.type_0 = 0;
        st.borg.flight_to_stairs = false;
    }
}

pub(crate) fn returning(st: &BorgState) -> bool {
    return_requirement(st).is_some()
}

pub(crate) fn return_requirement(st: &BorgState) -> Option<(i32, i32)> {
    let Phase::Returning { started_with } = st.think_dungeon.healing_restock.phase else {
        return None;
    };
    (st.borg.tr(BI_CDEPTH) != 0
        && enabled(st)
        && (!engine::player_opt(OPT_birth_force_descend)
            || st.borg.goal.recalling != 0
            || can_try_recall(st)))
    .then(|| (started_with, minimum()))
}

pub(crate) fn deferred(st: &BorgState) -> bool {
    st.think_dungeon.healing_restock.phase == Phase::Deferred
}

pub(crate) fn follow_return(st: &mut BorgState) -> bool {
    let attempt = (st.borg.time.level, st.borg.time.now);
    if st
        .think_dungeon
        .healing_restock
        .route_attempt
        .replace(attempt)
        == Some(attempt)
        || st.borg.goal.recalling != 0
    {
        return false;
    }

    if !engine::player_opt(OPT_birth_force_descend) {
        if u32::from(st.cave.grid(st.borg.c.x, st.borg.c.y).feat) == FEAT_LESS {
            crate::borg_io::borg_keypress(st, u32::from(b'<'));
            return true;
        }
        let previous_commit = st.flow.commit_seq;
        let stepped = crate::borg_flow_stairs::borg_flow_stair_less(st, GOAL_BORE, false);
        if st.flow.commit_seq != previous_commit {
            st.think_dungeon.healing_restock.stair_commit = Some(st.flow.commit_seq);
        }
        if stepped {
            return true;
        }
        // Committing a flow can succeed before a usable first step is found.
        clear_stair_route(st);
    }

    // With no usable up-stair route, Recall may be worthwhile even on a
    // shallow level. Keep the usual exposure check and spell/item checks.
    can_try_recall(st) && crate::borg_escape::borg_recall_for_flight(st)
}

fn can_try_recall(st: &BorgState) -> bool {
    st.borg.tr(BI_RECALL) > 0 && !engine::player_opt(OPT_birth_no_recall)
}

pub(crate) fn observe_town(st: &mut BorgState) {
    if st.borg.tr(BI_CDEPTH) != 0 || !enabled(st) {
        return;
    }
    let held = carried_potions(st);
    let gold = st.borg.tr(BI_GOLD);
    // Refresh only from town observations. Dungeon decisions never see
    // offscreen restocks, and an unaffordable or fruitless visit is not
    // repeated with the same resources.
    let mut offers: Vec<_> = town_offers(st, false)
        .map(|(_, _, price, count)| (price, count))
        .collect();
    offers.sort_unstable_by_key(|&(price, _)| price);
    let restock = &mut st.think_dungeon.healing_restock;
    restock.stair_commit = None;
    restock.town = TownSupply {
        offers,
        departure_gold: gold,
        departure_stock: held,
    };
    restock.phase = match restock.phase {
        Phase::Returning { .. } | Phase::Shopping | Phase::Deferred => Phase::Shopping,
        _ if held > 0 => Phase::Stocked,
        _ => Phase::NoReserve,
    };
}

pub(crate) fn purchase(st: &BorgState) -> Option<TownAction> {
    if st.borg.tr(BI_CDEPTH) != 0
        || !enabled(st)
        || st.think_dungeon.healing_restock.phase != Phase::Shopping
        || st.borg.tr(BI_LIGHT) == 0
        || st.borg.tr(BI_FOOD) == 0
        || st.borg.tr(BI_ISCUT) != 0
        || st.borg.tr(BI_ISPOISONED) != 0
        || carried_potions(st) >= minimum()
    {
        return None;
    }
    let gold = st.borg.tr(BI_GOLD);
    town_offers(st, true)
        .filter(|&(_, _, price, _)| price <= gold)
        .min_by_key(|&(_, _, price, _)| price)
        .map(|(shop, slot, _, _)| TownAction {
            code: if shop == BORG_HOME {
                action::TAKE_HOME
            } else {
                action::BUY
            },
            at: [shop as i64, slot as i64, 1, 0],
        })
}

/// Quotes of known healing potions from ordinary stores and Home. The
/// Black Market remains subject to the normal planner's separate budget.
fn town_offers(
    st: &BorgState,
    buying: bool,
) -> impl Iterator<Item = (usize, usize, i32, i32)> + '_ {
    st.store
        .shops
        .iter()
        .enumerate()
        .filter(|&(shop, _)| shop != crate::borg_town_stock::BORG_BLACK_MARKET)
        .flat_map(move |(shop, stock)| {
            let native = engine::store_stock(&st.access, shop);
            stock
                .ware
                .iter()
                .enumerate()
                .filter_map(move |(slot, item)| {
                    if !healing_potion(st, item) || (shop == BORG_HOME && !item.aware) {
                        return None;
                    }
                    let offer = native.get(slot)?;
                    if buying
                        && (offer.carry_capacity() == 0
                            || crate::borg_town_live::acquisition_room(st, item.tval, item.kind)
                                == 0
                            || crate::borg_town_live::buy_rejected(st, shop as i16, slot))
                    {
                        return None;
                    }
                    let price = if shop == BORG_HOME {
                        0
                    } else {
                        offer.purchase_quote(shop)
                    };
                    Some((shop, slot, price, i32::from(item.iqty)))
                })
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn refill_uses_cheapest_stacks_and_requires_enough_observed_stock() {
        let supply = TownSupply {
            offers: vec![(0, 1), (20, 2), (70, 1)],
            departure_stock: 2,
            ..Default::default()
        };
        assert_eq!(supply.refill_cost(3), Some(40));
        assert_eq!(supply.refill_cost(5), None);
        assert!(!supply.can_refill(3, 39, 2, false));
        assert!(supply.can_refill(3, 40, 2, false));
    }

    #[test]
    fn a_never_stocked_borg_can_return_when_it_can_fund_the_reserve() {
        let mut restock = Restock {
            town: TownSupply {
                offers: vec![(20, 2)],
                departure_gold: 20,
                departure_stock: 0,
            },
            ..Default::default()
        };
        restock.observe_reserve(0, 2, 39);
        assert!(restock.phase == Phase::NoReserve);
        restock.observe_reserve(0, 2, 40);
        assert!(restock.phase == Phase::Returning { started_with: 0 });
    }

    #[test]
    fn free_home_stock_can_start_a_return_but_cannot_repeat_a_failed_visit() {
        let mut restock = Restock {
            town: TownSupply {
                offers: vec![(0, 2)],
                ..Default::default()
            },
            ..Default::default()
        };
        restock.observe_reserve(0, 2, 0);
        assert!(restock.phase == Phase::Returning { started_with: 0 });
        restock.phase = Phase::Shopping;
        restock.observe_reserve(1, 2, 0);
        assert!(restock.phase == Phase::Deferred);
        restock.observe_reserve(1, 2, 100);
        assert!(restock.phase == Phase::Deferred);
        restock.observe_reserve(2, 2, 100);
        assert!(restock.phase == Phase::Stocked);
    }

    #[test]
    fn paid_short_departure_waits_for_new_funding() {
        let supply = TownSupply {
            offers: vec![(20, 2)],
            departure_gold: 25,
            departure_stock: 1,
        };
        assert!(!supply.can_refill(1, 25, 2, false));
        assert!(!supply.can_refill(1, 44, 2, true));
        assert!(supply.can_refill(1, 45, 2, true));
    }

    #[test]
    fn depletion_requests_a_return_without_quotes_and_keeps_it_across_observations() {
        let mut restock = Restock::default();
        restock.observe_reserve(2, 2, 0);
        assert!(restock.phase == Phase::Stocked);
        restock.observe_reserve(0, 2, 0);
        assert!(restock.phase == Phase::Returning { started_with: 0 });
        restock.observe_reserve(1, 2, 0);
        assert!(restock.phase == Phase::Returning { started_with: 0 });
    }

    #[test]
    fn replenishing_the_reserve_ends_a_committed_return() {
        let mut restock = Restock {
            phase: Phase::Returning { started_with: 0 },
            ..Default::default()
        };
        restock.observe_reserve(1, 2, 0);
        assert!(restock.phase == Phase::Returning { started_with: 0 });
        restock.observe_reserve(2, 2, 0);
        assert!(restock.phase == Phase::Stocked);
    }
}

#[cfg(feature = "test-stair-handoff")]
pub(crate) mod fixture {
    use super::*;
    use crate::borg_headers::borg_trait_h::{BI_AHEAL, BI_APHASE, BI_CURHP, BI_MAXHP};
    use crate::borg_io::borg_inkey;
    use crate::borg_think_dungeon::borg_think_dungeon;
    use crate::borg_think_dungeon_util::borg_leave_level;
    use crate::headers::list_terrain_h::{FEAT_FLOOR, FEAT_GRANITE};
    use crate::headers::z_type_h::loc;

    pub(crate) fn dungeon(st: &mut BorgState, low: bool) -> ! {
        assert_eq!(st.borg.tr(BI_CDEPTH), 2);
        assert!(engine::option_set(c"birth_force_descend", false));
        let start = loc { x: 40, y: 20 };
        let up = loc { x: 41, y: 20 };
        let pack_size = crate::borg_item::inven_wield() as usize;
        st.items.items[..pack_size].fill(crate::borg_item::blank_item());
        let potion = &mut st.items.items[0];
        potion.aware = true;
        potion.tval = TV_POTION as u8;
        potion.sval = st.item_val.sv_potion_cure_light as u8;
        potion.iqty = if low { 2 } else { 1 };
        if low {
            st.borg.trait_[BI_CDEPTH as usize] = 0;
            st.borg.trait_[BI_GOLD as usize] = 0;
            observe_town(st);
            st.borg.trait_[BI_CDEPTH as usize] = 2;
        }
        observe_dungeon(st);
        assert!(!returning(st));
        assert!(crate::borg_item_use::borg_quaff_potion(
            st,
            st.item_val.sv_potion_cure_light
        ));
        assert_eq!(borg_inkey(st, true), u32::from(b'q'));
        borg_inkey(st, true);
        st.items.items[0].iqty -= 1;
        st.borg.trait_[BI_AHEAL as usize] = 1000; // A healing spell is no potion.
        st.borg.trait_[BI_RECALL as usize] = 0;
        st.borg.goal.type_0 = 0;

        let arrive = |st: &mut BorgState, at: loc, stair: loc| {
            crate::borg_cave::borg_init_cave(st);
            for y in 19..=21 {
                for x in 39..=42 {
                    st.cave.grid_mut(x, y).feat = FEAT_GRANITE as u8;
                }
            }
            st.cave.grid_mut(at.x, at.y).feat = FEAT_FLOOR as u8;
            st.cave.grid_mut(stair.x, stair.y).feat = FEAT_LESS as u8;
            st.borg.c = at;
            st.borg.time.now += 1;
            st.borg.time.level = st.borg.time.now;
            st.stairs.less.num = 1;
            st.stairs.less.x[0] = stair.x;
            st.stairs.less.y[0] = stair.y;
            st.stairs.more.num = 0;
            st.kills.cnt = 0;
            st.kills.nxt = 1;
            st.danger = Default::default();
            crate::borg_flow::borg_flow_clear(st);
        };
        arrive(st, start, loc { x: 42, y: 20 });
        if low {
            observe_dungeon(st);
            assert_eq!(carried_potions(st), 1);
            assert!(
                !returning(st),
                "one potion remains and the refill is unaffordable"
            );
            let price = st
                .think_dungeon
                .healing_restock
                .town
                .refill_cost(1)
                .unwrap();
            st.borg.trait_[BI_GOLD as usize] = price;
        }
        observe_dungeon(st);
        assert_eq!(return_requirement(st), Some((i32::from(low), 2)));
        assert!(st.rising_to_town());
        assert!(crate::borg_prepared::borg_must_return_to_town(st).is_some());
        assert!(
            !borg_leave_level(st, false),
            "a wall blocks the only staircase"
        );
        assert!(
            !borg_leave_level(st, true),
            "a later call in the same think does not retry"
        );
        assert!(st.io.queued_keys().is_empty());

        arrive(st, start, up);
        for (index, value) in [(BI_FOOD, 5), (BI_LIGHT, 2), (BI_CURHP, 50), (BI_MAXHP, 50)] {
            st.borg.trait_[index as usize] = value;
        }
        st.borg.avoidance = 50;
        assert!(
            borg_think_dungeon(st),
            "normal dungeon decision follows the return"
        );
        assert_eq!(st.io.queued_keys(), [u32::from(b'6')]);
        borg_inkey(st, true);
        if low {
            let restock = &st.think_dungeon.healing_restock;
            let returning_phase = restock.phase;
            let stairs = restock
                .stair_commit
                .expect("the healing walk owns its stairs");
            st.items.items[0].iqty = 2;
            observe_dungeon(st);
            assert!(!st.rising_to_town());
            assert_eq!(st.borg.goal.type_0, 0, "the healing stair walk was cleared");

            // A separate town objective may already have replaced the flow.
            st.think_dungeon.healing_restock.phase = returning_phase;
            st.think_dungeon.healing_restock.stair_commit = Some(stairs);
            st.flow.commit_seq = stairs.wrapping_add(1);
            st.borg.goal.rising = true;
            st.borg.goal.type_0 = GOAL_BORE as i16;
            observe_dungeon(st);
            assert!(!returning(st));
            assert!(st.rising_to_town(), "the other return survives");
            assert_eq!(st.borg.goal.type_0, GOAL_BORE as i16, "so does its walk");

            st.borg.goal.rising = false;
            st.items.items[0].iqty = 1;
            observe_dungeon(st);
            assert!(returning(st));
        }
        st.borg.c = up;
        st.borg.time.now += 1;
        assert!(borg_leave_level(st, false));
        assert_eq!(borg_inkey(st, true), u32::from(b'<'));

        st.borg.trait_[BI_CDEPTH as usize] = 1;
        arrive(st, up, up);
        observe_dungeon(st);
        assert!(
            borg_leave_level(st, false),
            "the next level's grace period does not interrupt the return"
        );
        assert_eq!(borg_inkey(st, true), u32::from(b'<'));

        st.borg.trait_[BI_CDEPTH as usize] = 0;
        st.borg.trait_[BI_GOLD as usize] = 0;
        st.borg.goal.rising = false;
        observe_town(st);
        let price = st
            .think_dungeon
            .healing_restock
            .town
            .refill_cost(1)
            .expect("a regular shop has healing for sale");
        assert!(
            purchase(st).is_none(),
            "a potion the Borg cannot afford is never selected"
        );
        st.borg.trait_[BI_CDEPTH as usize] = 1;
        for gold in [0, price - 1] {
            st.borg.trait_[BI_GOLD as usize] = gold;
            observe_dungeon(st);
            assert!(!returning(st));
            assert!(
                !st.rising_to_town(),
                "an unchanged funding gap does not bounce back"
            );
        }
        st.borg.trait_[BI_CDEPTH as usize] = 6;
        st.borg.time.now += 101;
        st.borg.trait_[BI_FOOD as usize] = 5;
        st.borg.trait_[BI_LIGHT as usize] = 2;
        st.borg.trait_[BI_APHASE as usize] = 1;
        assert!(
            crate::borg_prepared::borg_must_return_to_town(st).is_none(),
            "the old cure check does not restart the deferred trip at DL6"
        );
        st.borg.trait_[BI_APHASE as usize] = 0;
        assert!(
            crate::borg_prepared::borg_must_return_to_town(st).is_some(),
            "a different supply shortage can still trigger restocking"
        );

        st.borg.trait_[BI_GOLD as usize] = price;
        observe_dungeon(st);
        assert!(
            returning(st),
            "dungeon gold now covers the last quoted price"
        );

        let case = if low { "short" } else { "dungeon" };
        println!("#1214 healing restock passed: {case}");
        std::process::exit(0);
    }

    // Use the native town and the production shop chooser and purchase UI.
    // The hook returns until a subsequent inventory read observes the potion.
    pub(crate) fn town(st: &mut BorgState, case: &str) {
        assert_eq!(
            st.borg.tr(BI_CDEPTH),
            0,
            "the Borg should shop before leaving"
        );
        assert!(enabled(st));
        if st.think_dungeon.healing_restock.phase == Phase::NoReserve {
            assert_eq!(carried_potions(st), 0);
            if case == "healing-restock-town-legacy-unknown" {
                assert!(!crate::borg_town_live::active(st));
                engine::carry_test_potion(st, st.item_val.sv_potion_cure_light);
                crate::borg_item::borg_find_all_items(st);
                crate::borg_trait::borg_notice(st, true);
                assert!(carries_unknown_potion(st));
                assert_eq!(carried_potions(st), 0);
            }
            st.think_dungeon.healing_restock.phase = Phase::Returning { started_with: 0 };
        } else if carried_potions(st) >= minimum() {
            let case = case.strip_prefix("healing-restock-").unwrap();
            println!("#1214 healing restock passed: {case}");
            std::process::exit(0);
        }
    }
}
