//! #1239: evaluate and execute a town swap with the displaced gear at Home.
use crate::borg_headers::borg_item_h::borg_item;
use crate::borg_headers::borg_trait_h::*;
use crate::borg_item::inven_wield;
use crate::borg_store::BORG_HOME;
use crate::borg_town_record::action;
use crate::borg_town_shadow::TownAction;
use crate::headers::list_tvals_h::*;
use crate::{engine, state::BorgState};

#[derive(Clone, Copy)]
enum Source {
    Pack(usize),
    Store(usize, usize),
}

#[derive(Clone, Copy)]
enum Phase {
    ClearHome(borg_item),
    DropEviction(borg_item),
    Acquire,
    Wear,
    Deposit,
    VerifyDeposit(u32),
}

pub(crate) struct Swap {
    slot: usize,
    old: borg_item,
    new: borg_item,
    source: Source,
    phase: Phase,
}

pub(crate) enum Next {
    Town(TownAction),
    Wear(usize),
    Done,
    Failed,
}

fn known(item: &borg_item) -> bool {
    item.iqty > 0 && item.aware && item.ident && !item.needs_ident
}

pub(crate) fn same_item(a: &borg_item, b: &borg_item) -> bool {
    a.iqty > 0
        && a.kind == b.kind
        && a.art_idx == b.art_idx
        && a.ego_idx == b.ego_idx
        && a.weight == b.weight
        && a.to_a == b.to_a
        && a.to_h == b.to_h
        && a.to_d == b.to_d
        && a.flags == b.flags
        && a.modifiers == b.modifiers
        && a.curses == b.curses
}

fn equipment_slot(item: &borg_item) -> Option<usize> {
    match u32::from(item.tval) {
        TV_HAFTED | TV_POLEARM | TV_SWORD | TV_SOFT_ARMOR | TV_HARD_ARMOR | TV_DRAG_ARMOR
        | TV_CLOAK | TV_SHIELD | TV_HELM | TV_CROWN | TV_GLOVES | TV_BOOTS => {
            usize::try_from(crate::borg_inventory::borg_wield_slot(item)).ok()
        }
        _ => None,
    }
}

fn pack_slot(st: &BorgState, item: &borg_item) -> Option<usize> {
    st.items.items[..inven_wield() as usize]
        .iter()
        .position(|i| same_item(i, item))
}

fn home_quantity(st: &BorgState, item: &borg_item) -> u32 {
    st.store.shops[BORG_HOME]
        .ware
        .iter()
        .filter(|i| same_item(i, item))
        .map(|i| u32::from(i.iqty))
        .sum()
}

fn home_space(st: &BorgState) -> bool {
    st.store.shops[BORG_HOME].ware.iter().any(|i| i.iqty == 0)
}

fn expendable_rank(
    item: &borg_item,
    st: &BorgState,
    unwanted: bool,
    redundant_gear: bool,
) -> Option<(u8, i32)> {
    if !known(item) || item.art_idx != 0 || item.ego_idx != 0 {
        return None;
    }
    let value = item.value.saturating_mul(i32::from(item.iqty));
    if crate::borg_town_home::enabled(st) {
        use crate::borg_town_home::Role;
        let role = crate::borg_town_home::classify(st, item, false, 0);
        if role.reserve() || (u32::from(item.tval) == TV_SCROLL
            && i32::from(item.sval) == st.item_val.sv_scroll_detect_invis)
        {
            return None;
        }
        // Eviction clears rejected kinds before renewable stock. Offer
        // scoring assigns both the same Role::rank().
        match role {
            Role::Reject => return Some((0, value)),
            Role::Renewable => return Some((1, value)),
            Role::Common => return Some((2, value)),
            _ => {}
        }
    }
    let sv = i32::from(item.sval);
    let v = &st.item_val;
    // #1241: clear rejected stock and readily replaced, low-utility buffs
    // before scarce consumables, charged devices or alternative equipment.
    let rank = if unwanted {
        0
    } else if u32::from(item.tval) == TV_SCROLL
        && [v.sv_scroll_light, v.sv_scroll_blessing].contains(&sv)
    {
        1
    } else if u32::from(item.tval) == TV_POTION
        && [v.sv_potion_boldness, v.sv_potion_heroism].contains(&sv)
    {
        2
    } else if redundant_gear {
        3
    } else {
        return None;
    };
    Some((rank, value))
}

fn eviction(st: &BorgState) -> Option<borg_item> {
    let stock = engine::store_stock(&st.access, BORG_HOME);
    st.store.shops[BORG_HOME]
        .ware
        .iter()
        .zip(stock)
        .enumerate()
        .filter(|(slot, (item, native))| {
            native.merge_capacity() == 0
                && native.carry_capacity() >= i32::from(item.iqty)
                && !crate::borg_town_live::buy_rejected(st, BORG_HOME as i16, *slot)
                && pack_slot(st, item).is_none()
                && !crate::borg_town_live::retained_gear(st, item)
        })
        .filter_map(|(_, (item, _))| {
            let wield = crate::borg_inventory::borg_wield_slot(item);
            let redundant_gear = usize::try_from(wield)
                .ok()
                .and_then(|slot| st.items.items.get(slot))
                .is_some_and(|worn| same_item(worn, item) || item.cursed && !item.uncursable);
            let rank = expendable_rank(
                item,
                st,
                crate::borg_town_capability::unwanted_home_item(item),
                redundant_gear,
            )?;
            Some((rank, *item))
        })
        .min_by_key(|(rank, _)| *rank)
        .map(|(_, item)| item)
}

pub(crate) fn affordable(st: &BorgState, shop: usize, price: i32) -> bool {
    use crate::borg_town_budget::{black_market_budget, black_market_percentage, gold_reserve};
    use crate::params::{value, Param};
    let gold = i64::from(st.borg.tr(BI_GOLD));
    let reserve = gold_reserve(
        gold,
        i64::from(value(Param::town_gold_reserve)),
        i64::from(value(Param::town_gold_reserve_percent)),
    );
    i64::from(price) <= gold - reserve
        && (shop != crate::borg_town_stock::BORG_BLACK_MARKET
            || i64::from(price)
                <= black_market_budget(
                    gold,
                    i64::from(
                        st.cfg(crate::borg_headers::borg_h::BORG_SUM_EXCLUDED_FROM_BLACK_MARKET),
                    ),
                    black_market_percentage(st, gold),
                    1,
                ))
}

pub(crate) fn keeps_capabilities(before: &[i32], after: &[i32]) -> bool {
    let get = |traits: &[i32], i: u32| traits[i as usize];
    let protected = [
        BI_FRACT,
        BI_SFRACT,
        BI_HLIFE,
        BI_SHLIFE,
        BI_ESP,
        BI_SINV,
        BI_LIGHT,
        BI_FOOD,
        BI_RECALL,
        BI_APHASE,
        BI_ATELEPORT,
        BI_ATELEPORTLVL,
        BI_AESCAPE,
        BI_AHEAL,
        BI_AEZHEAL,
        BI_ALIFE,
        BI_AMISSILES,
        BI_SP_ADJ,
        BI_HP_ADJ,
    ];
    if (BI_IFIRE..=BI_RDIS)
        .chain(BI_SRFIRE..=BI_SRDIS)
        .chain(protected)
        .any(|i| get(after, i) < get(before, i))
    {
        return false;
    }
    // Keep a viable melee attack when the current kit has one. The usual
    // equipment power accounts for its damage, accuracy and slays when ranking.
    let melee = |t: &[i32]| {
        get(t, BI_WDD) > 0
            && get(t, BI_WDS) > 0
            && get(t, BI_HEAVYWEPON) == 0
            && get(t, BI_WDD) * (get(t, BI_WDS) + 1) + 2 * (get(t, BI_TODAM) + get(t, BI_WTODAM))
                > 0
    };
    (!melee(before) || melee(after))
        && (get(before, BI_HEAVYBOW) != 0 || get(after, BI_HEAVYBOW) == 0)
}

fn improves(current: (i32, i32), trial: (i32, i32)) -> bool {
    let (current_speed, current_room) = current;
    let (speed, room) = trial;
    room > current_room && speed >= current_speed
}

fn suitable_weight(candidate: i16, worn: i16, restoring: bool) -> bool {
    if restoring { candidate > worn } else { candidate < worn }
}

fn initial_phase(source: Source, can_deposit: bool, spare: Option<borg_item>) -> Option<Phase> {
    if can_deposit {
        Some(match source {
            Source::Pack(_) => Phase::Wear,
            Source::Store(_, _) => Phase::Acquire,
        })
    } else {
        spare.map(Phase::ClearHome)
    }
}

pub(crate) fn propose(st: &mut BorgState, tried: &[bool; 12], restoring: bool) -> Option<Swap> {
    let pack = inven_wield() as usize;
    let has_home_space = home_space(st);
    let spare = (!has_home_space).then(|| eviction(st)).flatten();
    let full_pack = crate::borg_inventory::borg_first_empty_inventory_slot(st) < 0;
    let mut candidates = Vec::new();
    if !restoring {
        candidates.extend(
            st.items.items[..pack]
                .iter()
                .copied()
                .enumerate()
                .filter(|(_, item)| !full_pack || item.iqty == 1)
                .map(|(slot, item)| (Source::Pack(slot), item, 0)),
        );
    }
    for shop in (if restoring { BORG_HOME } else { 0 }..=BORG_HOME).rev() {
        let stock = engine::store_stock(&st.access, shop);
        for (slot, (item, native)) in st.store.shops[shop].ware.iter().zip(stock).enumerate() {
            if !known(item)
                || equipment_slot(item).is_none()
                || native.carry_capacity() < 1
                || pack_slot(st, item).is_some()
                || crate::borg_town_live::acquisition_room(st, item.tval, item.kind) == 0
                || crate::borg_town_live::buy_rejected(st, shop as i16, slot)
                || shop == BORG_HOME && crate::borg_town_live::home_moved(st, item, true)
                || restoring && !crate::borg_town_live::retained_gear(st, item)
            {
                continue;
            }
            let price = if shop == BORG_HOME {
                0
            } else {
                native.purchase_quote(shop)
            };
            if shop == BORG_HOME || affordable(st, shop, price) {
                candidates.push((Source::Store(shop, slot), *item, price));
            }
        }
    }

    let borg = st.borg.clone();
    let traits = st.traits.clone();
    let swap = st.trait_swap.clone();
    let items = st.items.items.clone();
    let buffer = st.prepared.buffer.clone();
    let home = st.home.clone();
    let do_spell = st.think.do_spell;
    // player-calcs.c:weight_limit, calc_bonuses (1741–1750, 2222–2229).
    // The Borg's notice recomputes this from the replacement's actual STR
    // and carried weight, including the ammo and the replacement itself.
    let current = (
        borg.tr(BI_SPEED),
        borg.tr(BI_CARRY) / 2 - borg.tr(BI_WEIGHT),
    );
    let current_power = restoring.then(|| crate::borg_power::borg_power(st));
    let mut best = None;
    for (source, item, price) in candidates {
        let Some(slot) = equipment_slot(&item) else {
            continue;
        };
        let old = items[slot];
        if tried[slot - pack]
            || !known(&item)
            || item.cursed
            || old.iqty == 0
            || !suitable_weight(item.weight, old.weight, restoring)
            || crate::borg_town_live::sale_rejected(st, BORG_HOME as i16, &old)
            || engine::flag_has(
                &old.flags,
                crate::headers::list_object_flags_h::OF_STICKY as i32,
            )
        {
            continue;
        }
        let opens_home = matches!(source, Source::Store(BORG_HOME, _)) && item.iqty == 1;
        let can_deposit = has_home_space
            || opens_home
            || engine::equip_object(&st.access, slot - pack)
                .is_some_and(|i| i.store_accepts_quantity(BORG_HOME, 1));
        let Some(phase) = initial_phase(source, can_deposit, spare.filter(|_| !restoring)) else {
            continue;
        };

        st.borg.clone_from(&borg);
        st.items.items.clone_from(&items);
        st.think.do_spell = false;
        if let Source::Pack(index) = source {
            st.items.items[index].iqty -= 1;
        }
        st.items.items[slot] = borg_item { iqty: 1, ..item };
        crate::borg_trait::borg_notice(st, true);
        let speed = st.borg.tr(BI_SPEED);
        let room = st.borg.tr(BI_CARRY) / 2 - st.borg.tr(BI_WEIGHT);
        let fits = if restoring {
            room >= 0 && speed >= current.0
        } else {
            improves(current, (speed, room))
        };
        if !fits || !keeps_capabilities(&borg.trait_, &st.borg.trait_) {
            continue;
        }
        let power = crate::borg_power::borg_power(st);
        if current_power.is_some_and(|base| power <= base) {
            continue;
        }
        let rank = (room >= 0, speed, power, room, -price);
        if best.as_ref().is_none_or(|(r, _)| rank > *r) {
            best = Some((
                rank,
                Swap {
                    slot,
                    old,
                    new: item,
                    source,
                    phase,
                },
            ));
        }
    }
    st.borg = borg;
    st.traits = traits;
    st.trait_swap = swap;
    st.items.items = items;
    st.prepared.buffer = buffer;
    st.home = home;
    st.think.do_spell = do_spell;
    best.map(|(_, swap)| swap)
}

impl Swap {
    pub(crate) fn equipment_index(&self) -> usize {
        self.slot - inven_wield() as usize
    }

    pub(crate) fn retained(&self) -> Option<borg_item> {
        (self.old.weight > self.new.weight).then_some(self.old)
    }

    pub(crate) fn next(&mut self, st: &mut BorgState) -> Next {
        let town = |code, shop, slot: usize, qty, price| {
            Next::Town(TownAction {
                code,
                at: [shop, slot as i64, qty, price],
            })
        };
        match self.phase {
            Phase::ClearHome(item) => {
                let Some(slot) = st.store.shops[BORG_HOME]
                    .ware
                    .iter()
                    .position(|i| same_item(i, &item) && i.iqty == item.iqty)
                else {
                    return Next::Failed;
                };
                if st.borg.in_shop && st.think.shop_num == BORG_HOME as i16 {
                    self.phase = Phase::DropEviction(item);
                }
                town(
                    action::DROP_HOME,
                    BORG_HOME as i64,
                    slot,
                    i64::from(item.iqty),
                    0,
                )
            }
            Phase::DropEviction(item) => {
                let Some(slot) =
                    pack_slot(st, &item).filter(|&i| st.items.items[i].iqty == item.iqty)
                else {
                    return Next::Failed;
                };
                self.phase = match self.source {
                    Source::Pack(_) => Phase::Wear,
                    Source::Store(_, _) => Phase::Acquire,
                };
                town(action::DROP, -1, slot, i64::from(item.iqty), 0)
            }
            Phase::Acquire => {
                let Source::Store(shop, original) = self.source else {
                    return Next::Failed;
                };
                let ware = &st.store.shops[shop].ware;
                let slot = if ware.get(original).is_some_and(|i| same_item(i, &self.new)) {
                    Some(original)
                } else {
                    ware.iter().position(|i| same_item(i, &self.new))
                };
                let Some(slot) = slot else {
                    return Next::Failed;
                };
                if st.borg.in_shop && st.think.shop_num == shop as i16 {
                    self.phase = Phase::Wear;
                }
                town(
                    if shop == BORG_HOME {
                        action::TAKE_HOME
                    } else {
                        action::BUY
                    },
                    shop as i64,
                    slot,
                    1,
                    if shop == BORG_HOME {
                        0
                    } else {
                        i64::from(ware[slot].cost)
                    },
                )
            }
            Phase::Wear => {
                if st.borg.in_shop {
                    return Next::Town(TownAction {
                        code: action::LEAVE,
                        at: [-1; 4],
                    });
                }
                let Some(slot) = pack_slot(st, &self.new) else {
                    return Next::Failed;
                };
                let can_deposit = engine::equip_object(&st.access, self.equipment_index())
                    .is_some_and(|i| i.store_accepts_quantity(BORG_HOME, 1));
                if !can_deposit
                    || !same_item(&st.items.items[self.slot], &self.old)
                    || st.items.items[slot].iqty > 1
                        && crate::borg_inventory::borg_first_empty_inventory_slot(st) < 0
                {
                    return Next::Failed;
                }
                self.phase = Phase::Deposit;
                Next::Wear(slot)
            }
            Phase::Deposit => {
                if !same_item(&st.items.items[self.slot], &self.new) {
                    return Next::Failed;
                }
                let Some(slot) = pack_slot(st, &self.old) else {
                    return Next::Failed;
                };
                if !engine::inven_object(&st.access, slot)
                    .is_some_and(|i| i.store_accepts_quantity(BORG_HOME, 1))
                {
                    return Next::Failed;
                }
                if st.borg.in_shop && st.think.shop_num == BORG_HOME as i16 {
                    self.phase = Phase::VerifyDeposit(home_quantity(st, &self.old));
                }
                town(action::STORE, BORG_HOME as i64, slot, 1, 0)
            }
            Phase::VerifyDeposit(before) => {
                if home_quantity(st, &self.old) > before {
                    Next::Done
                } else {
                    Next::Failed
                }
            }
        }
    }
}

#[cfg(feature = "test-town-loadout")]
pub(crate) fn fixture(st: &mut BorgState, case: &str) {
    use std::sync::atomic::{AtomicU8, Ordering::Relaxed};
    static SETUP: AtomicU8 = AtomicU8::new(0);
    assert!(matches!(case, "trim" | "arrival" | "arrival-shop" | "restore" | "no-storage" | "reserve-overlap" | "reserve-swap" | "reserve-book" | "swap-spare" | "digger-swap" | "reserve-overweight" | "reserve-overweight-shop"));
    let shop_reserve = case == "reserve-overweight-shop";
    let reserve = shop_reserve || case == "reserve-overweight";
    let overlap = matches!(case, "reserve-overlap" | "reserve-swap" | "reserve-book");
    if SETUP.load(Relaxed) == 1 { return; }
    let wield = inven_wield() as usize;
    if SETUP.load(Relaxed) == 0 {
        SETUP.store(1, Relaxed);
        let shop = case == "arrival-shop";
        let excess = match case {
            "trim" | "arrival" => 188,
            "arrival-shop" => 80,
            _ if reserve => 170,
            _ => 104,
        };
        engine::setup_test_town_loadout(
            st, excess, !shop && !reserve && case != "no-storage", false,
            if shop || shop_reserve { 500 } else { 0 }, shop || overlap, !shop_reserve, case == "reserve-swap", case == "reserve-book", case == "swap-spare", case == "digger-swap",
        );
        crate::borg_item::borg_find_all_items(st);
        crate::borg_trait::borg_notice(st, true);
        assert_eq!(st.borg.tr(BI_CLEVEL), 20);
        assert_eq!(crate::borg_town_budget::Facts::of(st).allowance(), -i64::from(excess));
        if reserve {
            assert_eq!(st.borg.tr(BI_RECALL), 0);
            SETUP.store(2, Relaxed);
            return;
        }
        if overlap {
            use crate::borg_town_home::{self, Role};
            assert!(borg_town_home::enabled(st));
            let home_item = |tval, sval| {
                st.store.shops[BORG_HOME].ware.iter().find(|it| {
                    it.iqty > 0 && u32::from(it.tval) == tval && i32::from(it.sval) == sval
                }).expect("fixture stock should be known at Home")
            };
            let second_sight = home_item(TV_MUSHROOM, st.item_val.sv_mush_second_sight);
            let speed = home_item(TV_POTION, st.item_val.sv_potion_speed);
            assert!(crate::borg_town_capability::unwanted_home_item(second_sight));
            assert_eq!(borg_town_home::classify(st, second_sight, false, 0), Role::Reject);
            assert_eq!(borg_town_home::classify(st, speed, false, 0), Role::LatePotion);
            assert!(same_item(&eviction(st).expect("the excluded stack makes room"), second_sight));
            if case == "reserve-swap" {
                // #1317: under the shipped ranked gear policy known Home gear
                // is protected from Home replacement and from the loadout's
                // eviction, whatever its power (a unit test in
                // borg_town_utility covers a swap with no power).
                let ring = *st.store.shops[BORG_HOME].ware.iter()
                    .find(|it| it.iqty > 0 && u32::from(it.tval) == TV_RING)
                    .expect("the fixture ring is known at Home");
                assert!(crate::borg_junk::gear_alternatives() > 0, "shipped ranked gear policy");
                assert!(borg_town_home::classify(st, &ring, false, 0).gear());
                let kit = crate::borg_town_shadow::kit_for_test(st);
                let fact = kit.iter().find(|k| k.place == crate::borg_town_dispose::Place::Home
                    && k.kind == u32::from(ring.kind)).expect("the ring is in the Home kit");
                assert!(fact.home_role.gear() && fact.protect_disposal,
                    "role {:?}, protected {}, power {}", fact.home_role, fact.protect_disposal, fact.power);
            }
            if case == "reserve-book" {
                use crate::borg_town_dispose::Place;
                let book = |tval: u32| st.store.shops[BORG_HOME].ware.iter()
                    .find(|it| it.iqty > 0 && u32::from(it.tval) == tval).copied();
                let priest = book(TV_PRAYER_BOOK).expect("the Priest dungeon book at Home");
                let mage = book(TV_MAGIC_BOOK).expect("the Mage dungeon book at Home");
                assert_eq!(borg_town_home::classify(st, &priest, false, 0), Role::Scarce);
                assert_ne!(borg_town_home::classify(st, &mage, false, 0), Role::Scarce);
                let handbook = st.items.items[..wield].iter().find(|it| it.iqty > 0
                    && u32::from(it.tval) == TV_PRAYER_BOOK && it.kind != priest.kind)
                    .expect("the carried town book");
                assert_eq!(borg_town_home::classify(st, handbook, false, 0), Role::Renewable);
                let kit = crate::borg_town_shadow::kit_for_test(st);
                let role = |place: Place, kind: u32| kit.iter()
                    .find(|k| k.place == place && k.kind == kind).map(|k| k.home_role);
                assert_eq!(role(Place::Home, u32::from(priest.kind)), Some(Role::Scarce));
                assert_eq!(role(Place::Pack, u32::from(priest.kind)), Some(Role::Ordinary),
                    "a single carried copy is the one the Borg reads, not a spare");
            }
            println!("#1239 town loadout passed: {case}");
            std::process::exit(0);
        }
        if case == "digger-swap" {
            // #1320: a carried plain Shovel at max depth 30, in the 25-40
            // band where weapon_value rewards a digger.
            let slot = st.items.items[..wield].iter()
                .position(|it| it.iqty > 0 && u32::from(it.tval) == TV_DIGGING)
                .expect("the carried Shovel");
            let shovel = st.items.items[slot];
            assert_eq!((st.borg.tr(BI_MAXDEPTH), shovel.pval), (30, 0));
            assert_eq!(crate::borg_trait_swap::digger_power(&shovel), 1);
            let on = crate::params::value(crate::params::Param::swap_digger_tunnel) != 0;
            let (swap, value, digger) = (st.trait_swap.weapon_swap, st.trait_swap.weapon_swap_value, st.trait_swap.weapon_swap_digger);
            if on {
                assert_eq!((swap, digger), (slot as i32 + 1, 1), "the Shovel is the tunnelling swap (value {value})");
                // The swap value is the Shovel's value with its power, and
                // exactly the reward 1 * 3500 + 1000 above its melee value.
                assert_eq!(crate::borg_trait_swap::weapon_value_for_test(st, &shovel, 1), value);
                let melee = crate::borg_trait_swap::weapon_value_for_test(st, &shovel, 0);
                assert_eq!(value - melee, 4500, "melee {melee}, swap {value}");
                assert!(melee <= 1000, "without the reward the Shovel is no swap: {melee}");
            } else {
                assert_eq!((swap, value, digger), (0, -1, 0), "C's pval reading: no reward, no swap");
            }
            println!("#1239 town loadout passed: {case} {}", if on { "on" } else { "off" });
            std::process::exit(0);
        }
        if case == "swap-spare" {
            use crate::borg_town_dispose::Place;
            let slot = st.items.items[..wield].iter()
                .position(|it| it.iqty > 0 && u32::from(it.tval) == TV_SHIELD)
                .expect("the carried Leather Shield");
            // Below swap_carry_min_depth no swap is noticed. Make the shield
            // the active armour swap directly, as a deeper Borg would carry it.
            st.trait_swap.armour_swap = slot as i32 + 1;
            assert!(crate::borg_junk::is_swap(st, slot as i32));
            let kind = u32::from(st.items.items[slot].kind);
            let in_kit = |st: &mut BorgState| crate::borg_town_shadow::kit_for_test(st).iter()
                .any(|k| k.place == Place::Pack && k.kind == kind);
            // Every planner disposition acts on a kit stack, so a swap left
            // out of the kit cannot be stored, sold or dropped there.
            assert!(!in_kit(st), "the active swap is left out of the planner's kit");
            // The overweight Priest is under pack pressure: the full live plan
            // does not move the swap either.
            let targets = |a: TownAction| matches!(a.code, action::STORE | action::SELL | action::DROP)
                && a.at[1] == slot as i64;
            let plan = crate::borg_town_shadow::live_plan(st).action;
            assert!(!targets(plan), "the live plan moves the active swap: {:?}", (plan.code, plan.at));
            // Second review of #1324: copy-limit cleanup runs before the
            // planner. At one copy per kind, the worn and carried shields
            // make one excess copy: a plain spare is stored, the swap is not.
            let limit = crate::borg_headers::borg_h::BORG_TOWN_MAX_NON_MISSILE_COPIES as usize;
            let configured = st.core.cfg[limit];
            st.core.cfg[limit] = 1;
            let overflow = crate::borg_town_live::overflow_action_for_test(st);
            assert!(!overflow.is_some_and(targets), "copy cleanup moves the active swap: {:?}", overflow.map(|a| (a.code, a.at)));
            st.trait_swap.armour_swap = 0;
            let overflow = crate::borg_town_live::overflow_action_for_test(st);
            assert!(overflow.is_some_and(targets), "the fixture exercises copy cleanup: {:?}", overflow.map(|a| (a.code, a.at)));
            st.trait_swap.armour_swap = slot as i32 + 1;
            st.core.cfg[limit] = configured;
            st.trait_swap.armour_swap = 0;
            assert!(in_kit(st), "an ordinary spare is in the kit");
            st.trait_swap.armour_swap = slot as i32 + 1;
            // The same shield, not the swap, is the ranked plan's stash.
            st.trait_swap.armour_swap = 0;
            let stored = |a: Option<TownAction>| a.is_some_and(|a| a.code == action::STORE && a.at[1] == slot as i64);
            assert!(stored(crate::borg_town_live::organize_gear_for_test(st)), "the fixture exercises the stash");
            st.trait_swap.armour_swap = slot as i32 + 1;
            let organized = crate::borg_town_live::organize_gear_for_test(st);
            assert!(!stored(organized), "the active swap is not stored at Home: {:?}", organized.map(|a| (a.code, a.at)));
            println!("#1239 town loadout passed: {case}");
            std::process::exit(0);
        }
        let plan = propose(st, &[false; 12], false);
        if case == "no-storage" {
            assert!(plan.is_none(), "no safe deposit can be proposed with full Home stock");
            println!("#1239 town loadout passed: {case}");
            std::process::exit(0);
        }
        let swap = plan.expect("the overweight Priest should be offered lighter equipment");
        if shop {
            assert!(matches!(swap.source, Source::Store(store, _) if store != BORG_HOME));
            SETUP.store(2, Relaxed);
            return;
        }
        assert!(matches!(swap.source, Source::Store(BORG_HOME, _)));
        assert_eq!((swap.old.weight, swap.new.weight), (180, 30));
        SETUP.store(2, Relaxed);
    }
    if reserve {
        if st.borg.tr(BI_CDEPTH) > 0 {
            assert!(st.borg.tr(BI_RECALL) > 0, "departed without the replenished Recall");
            assert_eq!(st.items.items[wield].weight, 180);
            assert!(crate::borg_town_budget::Facts::of(st).allowance() < 0);
            println!("#1264 exploration reserve passed: {case}");
            std::process::exit(0);
        }
        return;
    }
    assert_eq!(st.borg.tr(BI_CDEPTH), 0, "left town before the fixture observed the exchange");
    if case == "arrival-shop" {
        let stored = st.store.shops[BORG_HOME].ware.iter().any(|item| {
            item.iqty > 0 && equipment_slot(item).is_some()
                && crate::borg_town_live::retained_gear(st, item)
        });
        if stored && st.borg.tr(BI_RECALL) > 0
            && crate::borg_town_budget::Facts::of(st).allowance() >= 0
        {
            println!("#1239 town loadout passed: {case}");
            std::process::exit(0);
        }
        return;
    }
    if SETUP.load(Relaxed) == 2 && st.items.items[wield].weight == 30 {
        let heavy = st.store.shops[BORG_HOME].ware.iter()
            .find(|item| item.weight == 180 && crate::borg_town_live::retained_gear(st, item));
        if heavy.is_some() && st.borg.tr(BI_RECALL) > 0 {
            assert!(crate::borg_town_budget::Facts::of(st).allowance() >= 0);
            if matches!(case, "trim" | "arrival") {
                if case == "trim" {
                    assert_eq!(st.borg.tr(BI_AMISSILES), 20);
                } else {
                    assert!((20..57).contains(&st.borg.tr(BI_AMISSILES)));
                }
                println!("#1239 town loadout passed: {case}");
                std::process::exit(0);
            }
            SETUP.store(1, Relaxed);
            engine::strengthen_test_town_loadout(st);
            st.borg.time.level -= 1;
            crate::borg_trait::borg_notice(st, true);
            crate::borg_town_live::choose(st);
            let swap = propose(st, &[false; 12], true)
                .expect("a stronger Priest should be offered the heavier stored mace");
            assert_eq!((swap.old.weight, swap.new.weight), (30, 180));
            let mace = swap.new;
            let after = st.borg.tr(BI_CARRY) / 2 - st.borg.tr(BI_WEIGHT) - i32::from(mace.weight - swap.old.weight);
            assert!(after >= 0);
            println!("#1239 town loadout passed: {case}");
            std::process::exit(0);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::borg_item::blank_item;

    #[test]
    fn priest_can_finish_lightening_after_a_partial_quiver_trim() {
        // 10.4 lb over, plus a 0.5 lb Recall. A constructed quiver split
        // can yield a 4 lb whole stack and still leave the Borg overweight.
        let before = (110, -104 - 5 + 40);
        assert!(before.1 < 0);

        // Home holds a 3 lb whip. Wielding alone still carries the 18 lb mace;
        // depositing the mace brings the true weight inside the allowance.
        assert!(!improves(before, (110, before.1 - 30)));
        assert!(improves(before, (110, before.1 - 30 + 180)));
        assert!(
            !improves(before, (109, before.1 - 30 + 180)),
            "the lighter piece cannot make the actual speed worse"
        );

        let mut traits = vec![0; BI_MAX as usize];
        traits[BI_RECALL as usize] = 1;
        traits[BI_SP_ADJ as usize] = 12;
        traits[BI_WDD as usize] = 2;
        traits[BI_WDS as usize] = 4;
        traits[BI_RCONF as usize] = 1;
        let mut whip = traits.clone();
        whip[BI_WDD as usize] = 1;
        assert!(keeps_capabilities(&traits, &whip));
        for lost in [BI_RECALL, BI_SP_ADJ, BI_WDD, BI_RCONF] {
            let mut unusable = whip.clone();
            unusable[lost as usize] = 0;
            assert!(!keeps_capabilities(&traits, &unusable), "lost trait {lost}");
        }
    }

    #[test]
    fn full_home_prefers_the_withdrawal_or_clears_expendable_stock() {
        let mut st = BorgState::default();
        st.item_val.sv_scroll_blessing = 8;
        let common = borg_item {
            tval: TV_SCROLL as u8,
            sval: 8,
            kind: 2,
            aware: true,
            ident: true,
            iqty: 27,
            ..blank_item()
        };
        let scarce = borg_item {
            sval: 10,
            kind: 3,
            ..common
        };
        assert!(expendable_rank(&common, &st, false, false).is_some());
        assert!(expendable_rank(&scarce, &st, false, false).is_none());

        let home_whip = Source::Store(BORG_HOME, 4);
        assert!(matches!(
            initial_phase(home_whip, true, Some(common)),
            Some(Phase::Acquire)
        ));
        let shop_robes = Source::Store(1, 3);
        assert!(
            matches!(initial_phase(shop_robes, false, Some(common)), Some(Phase::ClearHome(i)) if i.kind == common.kind)
        );
        assert!(
            initial_phase(shop_robes, false, None).is_none(),
            "keep the worn kit when Home has no safe vacancy"
        );
        assert!(
            matches!(
                initial_phase(Source::Pack(0), true, None),
                Some(Phase::Wear)
            ),
            "a one-item pack slot can be reused by the displaced gear"
        );
    }
}
