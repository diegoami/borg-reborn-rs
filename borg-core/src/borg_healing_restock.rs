//! Town restocking after the last known healing potion is used (#1214).
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
    Returning,
    Shopping {
        retry_price: Option<i32>,
    },
    Deferred {
        retry_price: Option<i32>,
    },
}

#[derive(Default)]
pub(crate) struct Restock {
    phase: Phase,
    route_attempt: Option<(i32, i32)>,
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

fn has_reserve(st: &BorgState) -> bool {
    // The quaff command uses known pack items. The healing traits can also
    // count spells and device charges, which cannot replace a potion here.
    st.items
        .items
        .iter()
        .take(crate::borg_item::inven_wield() as usize)
        .any(|item| item.aware && healing_potion(st, item))
}

fn enabled(st: &BorgState) -> bool {
    value(Param::restock_empty_healing) != 0 && st.cfg(BORG_TOWN_NO_STORES) == 0
}

pub(crate) fn observe_dungeon(st: &mut BorgState) {
    if st.borg.tr(BI_CDEPTH) == 0 || !enabled(st) {
        return;
    }
    let gold = st.borg.tr(BI_GOLD);
    let old = st.think_dungeon.healing_restock.phase;
    let phase = match (old, has_reserve(st)) {
        // Once committed, continue the trip through dungeon level changes.
        (Phase::Returning, _) => Phase::Returning,
        (_, true) => Phase::Stocked,
        (Phase::Stocked, false) => Phase::Returning,
        (Phase::Shopping { retry_price }, false) => Phase::Deferred { retry_price },
        (
            Phase::Deferred {
                retry_price: Some(price),
            },
            false,
        ) if gold >= price => Phase::Returning,
        _ => old,
    };
    st.think_dungeon.healing_restock.phase = phase;
    if returning(st) {
        if old != Phase::Returning {
            crate::borg_io::borg_note(st, "# Returning to town: out of healing potions");
            if st.borg.goal.type_0 == GOAL_BORE as i16 {
                st.borg.goal.type_0 = 0;
            }
        }
        st.borg.goal.rising = true;
    }
}

pub(crate) fn returning(st: &BorgState) -> bool {
    st.borg.tr(BI_CDEPTH) != 0
        && st.think_dungeon.healing_restock.phase == Phase::Returning
        && enabled(st)
        && (!engine::player_opt(OPT_birth_force_descend)
            || st.borg.goal.recalling != 0
            || can_try_recall(st))
}

pub(crate) fn deferred(st: &BorgState) -> bool {
    matches!(
        st.think_dungeon.healing_restock.phase,
        Phase::Deferred { .. }
    )
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
        if crate::borg_flow_stairs::borg_flow_stair_less(st, GOAL_BORE, false) {
            return true;
        }
        // Commit can succeed before the first step is found to be unusable.
        if st.borg.goal.type_0 == GOAL_BORE as i16 {
            st.borg.goal.type_0 = 0;
        }
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
    match st.think_dungeon.healing_restock.phase {
        Phase::Returning | Phase::Shopping { .. } | Phase::Deferred { .. } => {
            // Cache only prices observed in town. If the visit fails, a
            // later increase that pays for a known offer can justify a retry.
            let gold = st.borg.tr(BI_GOLD);
            let retry_price = cheapest_offer(st, false)
                .map(|(_, price)| price)
                .filter(|&price| price > gold);
            st.think_dungeon.healing_restock.phase = Phase::Shopping { retry_price };
        }
        _ => {
            st.think_dungeon.healing_restock.phase = if has_reserve(st) {
                Phase::Stocked
            } else {
                Phase::NoReserve
            };
        }
    }
}

pub(crate) fn purchase(st: &BorgState) -> Option<TownAction> {
    if st.borg.tr(BI_CDEPTH) != 0
        || !enabled(st)
        || !matches!(
            st.think_dungeon.healing_restock.phase,
            Phase::Shopping { .. }
        )
        || st.borg.tr(BI_LIGHT) == 0
        || st.borg.tr(BI_FOOD) == 0
        || st.borg.tr(BI_ISCUT) != 0
        || st.borg.tr(BI_ISPOISONED) != 0
        || has_reserve(st)
    {
        return None;
    }
    cheapest_offer(st, true).map(|(action, _)| action)
}

fn cheapest_offer(st: &BorgState, buying: bool) -> Option<(TownAction, i32)> {
    let gold = st.borg.tr(BI_GOLD);
    let mut best = None;
    for (shop, stock) in st.store.shops.iter().enumerate() {
        // Leave Black Market spending to the normal planner and its budget.
        if shop == crate::borg_town_stock::BORG_BLACK_MARKET {
            continue;
        }
        let native = engine::store_stock(&st.access, shop);
        for (slot, item) in stock.ware.iter().enumerate() {
            if !healing_potion(st, item) || (shop == BORG_HOME && !item.aware) {
                continue;
            }
            let Some(offer) = native.get(slot) else {
                continue;
            };
            let price = if shop == BORG_HOME {
                0
            } else {
                offer.purchase_quote(shop)
            };
            if buying
                && (price > gold
                    || offer.carry_capacity() == 0
                    || crate::borg_town_live::acquisition_room(st, item.tval, item.kind) == 0
                    || crate::borg_town_live::buy_rejected(st, shop as i16, slot))
            {
                continue;
            }

            let code = if shop == BORG_HOME {
                action::TAKE_HOME
            } else {
                action::BUY
            };
            if best
                .as_ref()
                .is_none_or(|&(_, best_price)| price < best_price)
            {
                best = Some((
                    TownAction {
                        code,
                        at: [shop as i64, slot as i64, 1, 0],
                    },
                    price,
                ));
            }
        }
    }
    best
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

    pub(crate) fn dungeon(st: &mut BorgState) -> ! {
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
        potion.iqty = 1;
        observe_dungeon(st);
        assert!(!returning(st));
        assert!(crate::borg_item_use::borg_quaff_potion(
            st,
            st.item_val.sv_potion_cure_light
        ));
        assert_eq!(borg_inkey(st, true), u32::from(b'q'));
        borg_inkey(st, true);
        st.items.items[0].iqty = 0;
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
        observe_dungeon(st);
        assert!(st.borg.goal.rising);
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
        let Phase::Shopping {
            retry_price: Some(price),
        } = st.think_dungeon.healing_restock.phase
        else {
            panic!("fixture needs a regular shop with healing for sale");
        };
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
                !st.borg.goal.rising,
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

        println!("#1214 healing restock passed: dungeon");
        std::process::exit(0);
    }

    // Use the native town and the production shop chooser and purchase UI.
    // The hook returns until a subsequent inventory read observes the potion.
    pub(crate) fn town(st: &mut BorgState) {
        assert_eq!(
            st.borg.tr(BI_CDEPTH),
            0,
            "the Borg should shop before leaving"
        );
        assert!(enabled(st));
        if st.think_dungeon.healing_restock.phase == Phase::NoReserve {
            assert!(!has_reserve(st));
            st.think_dungeon.healing_restock.phase = Phase::Returning;
        } else if has_reserve(st) {
            println!("#1214 healing restock passed: town");
            std::process::exit(0);
        }
    }
}
