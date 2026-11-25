//! Execute the town plan through the existing store UI, refreshing after
//! every transaction. Default-off: the historical chooser remains control.
use crate::borg_headers::borg_h::{BORG_TOWN_NO_STORES, BORG_TOWN_PLAN};
use crate::borg_store::BORG_HOME;
use crate::borg_headers::borg_trait_h::BI_CDEPTH;
use crate::borg_io::{borg_keypress, borg_keypresses, borg_note, KC_ENTER};
use crate::borg_town_record::action;
use crate::borg_town_shadow::TownAction;
use crate::state::BorgState;

#[derive(Clone, PartialEq, Eq)]
struct SaleStamp {
    shop: i16,
    slot: usize,
    desc: [core::ffi::c_char; 80],
    qty: u8,
    value: i32,
}

impl SaleStamp {
    fn of(shop: i16, slot: usize, item: &crate::borg_headers::borg_item_h::borg_item) -> Self {
        Self { shop, slot, desc: item.desc, qty: item.iqty, value: item.value }
    }
    fn matches(&self, item: &crate::borg_headers::borg_item_h::borg_item) -> bool {
        self.desc == item.desc && self.qty == item.iqty && self.value == item.value
    }
}

#[derive(Clone, PartialEq, Eq)]
struct CarryContext {
    gold: i32,
    inventory: Vec<([core::ffi::c_char; 80], u8)>,
}
impl CarryContext {
    fn of(st: &BorgState) -> Self {
        Self { gold: st.borg.tr(crate::borg_headers::borg_trait_h::BI_GOLD),
            inventory: st.items.items.iter().map(|i| (i.desc, i.iqty)).collect() }
    }
    /// borg-reborn-rs (#507): every slot of `st.items.items`, the quiver
    /// included (it runs to `quiver_end()`), so an arrow purchase that
    /// merges into the quiver still counts its copies.
    fn quantity(&self) -> u32 { self.inventory.iter().map(|i| u32::from(i.1)).sum() }
}
#[derive(Clone)]
struct BuyAttempt {
    offer: SaleStamp,
    context: CarryContext,
    item: crate::borg_headers::borg_item_h::borg_item,
    keep: bool,
    /// borg-reborn-rs (#507): copies asked for at the quantity prompt; 1 outside a purchase.
    qty: i32,
}

struct HomeMove {
    kind: u32,
    stored: bool,
    carried: u32,
}

fn carried_kind(st: &BorgState, kind: u32) -> u32 {
    count_kind(&st.items.items, kind)
}

fn count_kind(items: &[crate::borg_headers::borg_item_h::borg_item], kind: u32) -> u32 {
    items.iter().filter(|i| i.kind == kind).map(|i| u32::from(i.iqty)).sum()
}

fn copy_limit(limit: i32, tval: u8, kind: u32) -> Option<u32> {
    use crate::headers::list_tvals_h::{TV_ARROW, TV_BOLT, TV_SHOT};
    (limit > 0 && kind > 0 && !matches!(u32::from(tval), TV_ARROW | TV_BOLT | TV_SHOT))
        .then_some(limit as u32)
}

fn excess_copies(items: &[crate::borg_headers::borg_item_h::borg_item], item: &crate::borg_headers::borg_item_h::borg_item, limit: i32) -> u32 {
    copy_limit(limit, item.tval, item.kind).map_or(0, |cap|
        count_kind(items, item.kind).saturating_sub(cap).min(u32::from(item.iqty)))
}

fn configured_limit(st: &BorgState) -> i32 {
    st.cfg(crate::borg_headers::borg_h::BORG_TOWN_MAX_NON_MISSILE_COPIES)
}

pub fn acquisition_room(st: &BorgState, tval: u8, kind: u32) -> i32 {
    copy_limit(configured_limit(st), tval, kind)
        .map_or(i32::MAX, |limit| limit.saturating_sub(carried_kind(st, kind)) as i32)
}

pub(crate) fn home_at_limit(st: &BorgState, item: &crate::borg_headers::borg_item_h::borg_item) -> bool {
    let limit = crate::borg_town_utility::home_stack_limit();
    limit > 0 && item.aware && crate::borg_town_utility::home_limited_kind(item.tval)
        && st.store.shops[BORG_HOME].ware.iter()
            .filter(|ware| ware.iqty > 0 && ware.kind == item.kind).count() >= limit
}

/// Known excess can be cleared in a single native transaction.
pub(crate) fn batch_home_disposal(st: &BorgState, item: &crate::borg_headers::borg_item_h::borg_item) -> bool {
    home_at_limit(st, item) || crate::borg_town_capability::unwanted_home_item(item)
}

fn overflow_action(st: &BorgState) -> Option<TownAction> {
    let limit = configured_limit(st);
    if limit <= 0 { return None; }
    let pack = st.items.items.iter().take(crate::borg_item::inven_wield() as usize);
    let excess: Vec<_> = pack.enumerate().rev().filter_map(|(slot, item)| {
        let quantity = excess_copies(&st.items.items, item, limit);
        (quantity > 0).then_some((slot, item, quantity as i32))
    }).collect();
    // Try Home even when it has no empty slot. A matching partial stack
    // may still take some or all of the excess.
    for shop in std::iter::once(BORG_HOME).chain(0..BORG_HOME) {
        for &(slot, item, excess) in &excess {
            if shop == BORG_HOME && (home_at_limit(st, item)
                || crate::borg_town_capability::unwanted_home_item(item)) { continue; }
            if sale_rejected(st, shop as i16, item) { continue; }
            if shop != BORG_HOME && !crate::borg_store_sell::borg_store_buys(item, shop as i32) { continue; }
            let excess = if shop == BORG_HOME { excess }
                else { crate::borg_store_sell::disposable_book_quantity(st, item, excess) };
            let Some(native) = crate::engine::inven_object(&st.access, slot) else { continue; };
            let Some(quantity) = (1..=excess).rev().find(|&n| native.store_accepts_quantity(shop, n)) else { continue; };
            let code = if shop == BORG_HOME { action::STORE } else { action::SELL };
            return Some(TownAction { code, at: [shop as i64, slot as i64, i64::from(quantity), 0] });
        }
    }
    None
}

impl HomeMove {
    fn pending(st: &BorgState, a: TownAction) -> Option<Self> {
        let (item, stored) = match a.code {
            action::STORE => (st.items.items.get(a.at[1] as usize)?, true),
            action::TAKE_HOME | action::DROP_HOME => (st.store.shops[BORG_HOME].ware.get(a.at[1] as usize)?, false),
            _ => return None,
        };
        Some(Self { kind: item.kind, stored, carried: carried_kind(st, item.kind) })
    }
}

#[derive(Default)]
pub struct LiveState {
    clock: Option<i32>,
    pending_sale: Option<SaleStamp>,
    rejected_sales: Vec<SaleStamp>,
    pending_buy: Option<BuyAttempt>,
    rejected_buys: Vec<BuyAttempt>,
    pending_home: Option<HomeMove>,
    /// Home transactions confirmed this visit: (item kind, true = stored,
    /// false = taken out). The planner re-solves after every action and one
    /// unit can look better on each side in turn; the reverse of a confirmed
    /// Home move is not offered again within the visit (borg-rs#90).
    home_moves: Vec<(u32, bool)>,
    /// #859: the level clock (`time.level`, when the current level was
    /// entered) at the last departure-room note. A town visit starts on
    /// arriving in town, so this notes once a visit; two departures with no
    /// level change between them share one note.
    room_noted_level: Option<i32>,
}

/// Consume one queued Home attempt after refreshing inventory. Count the kind
/// across all slots, so stack merging/reordering cannot confirm a canceled move.
fn settle_home(st: &mut BorgState) {
    if let Some(attempt) = st.think.town_live.pending_home.take() {
        let carried = carried_kind(st, attempt.kind);
        let confirmed = if attempt.stored { carried < attempt.carried } else { carried > attempt.carried };
        if confirmed {
            st.think.town_live.home_moves.push((attempt.kind, attempt.stored));
        }
    }
}

/// Whether this visit already moved `kind` into (`stored`) or out of the Home.
pub fn home_moved(st: &BorgState, kind: u32, stored: bool) -> bool {
    st.think.town_live.home_moves.iter().any(|&(k, s)| k == kind && s == stored)
}

/// The native store may reject an otherwise plausible offer (for example,
/// its stock is full). A queued command is not proof of a transaction.
fn settle_sale(st: &mut BorgState) {
    if st.think.town_live.clock != Some(st.borg.time.level) {
        st.think.town_live = LiveState { clock: Some(st.borg.time.level), ..LiveState::default() };
    }
    settle_home(st);
    if let Some(attempt) = st.think.town_live.pending_buy.take() {
        let now = CarryContext::of(st);
        if now.quantity() > attempt.context.quantity() || now.gold < attempt.context.gold {
            // The copies actually bought: what the pack gained, at most the
            // copies asked for (a stack can merge into an equipped quiver
            // slot the pack count does not cover, so never below one).
            let gained = now.quantity().saturating_sub(attempt.context.quantity());
            let bought = (gained as i32).clamp(1, attempt.qty.max(1));
            // The gold the transaction took, as the engine charged it: for
            // a stack of charged devices, price_item's total is not N times
            // the price of one (review of #509, F2).
            let paid = paid_for(attempt.context.gold, now.gold, attempt.item.cost, bought);
            crate::borg_store_buy::confirm_purchase(st, &attempt.item, i32::from(attempt.offer.shop), attempt.keep, bought, paid);
        } else {
            st.think.town_live.rejected_buys.push(attempt);
            borg_note(st, "# Town plan: purchase made no progress; exclude unchanged offer/context");
        }
    }
    if let Some(attempt) = st.think.town_live.pending_sale.take() {
        if st.items.items.get(attempt.slot).is_some_and(|it| attempt.matches(it)) {
            st.think.town_live.rejected_sales.push(attempt);
            borg_note(st, "# Town plan: sale made no inventory change; exclude that offer this visit");
        }
    }
}

pub fn sale_rejected(st: &BorgState, shop: i16, item: &crate::borg_headers::borg_item_h::borg_item) -> bool {
    st.think.town_live.rejected_sales.iter().any(|s| s.shop == shop && s.matches(item))
}

pub fn buy_rejected(st: &BorgState, shop: i16, slot: usize) -> bool {
    let Some(item) = st.store.shops.get(shop as usize).and_then(|s| s.ware.get(slot)) else { return true; };
    let context = CarryContext::of(st);
    st.think.town_live.rejected_buys.iter().any(|a|
        a.offer.shop == shop && a.offer.slot == slot && a.offer.matches(item) && a.context == context)
}

pub fn active(st: &BorgState) -> bool { st.cfg(BORG_TOWN_PLAN) == 1 }

/// borg-reborn-rs (#507): the copies of a planned purchase to type at the quantity prompt:
/// the plan's copies, capped by the shop's stack, the pack's room
/// (`inven_carry_num`) and the gold at the price of one (`price_item`).
/// Never more than store.c's `store_purchase` offers as its maximum, so a
/// quantity of two or more always meets the prompt (it is skipped at 1).
/// The floor of 1 is never typed and never reached when nothing is
/// affordable: store_purchase refuses with "You do not have enough gold"
/// before its prompt, and the settle then records no progress. Do not turn
/// the floor into a refusal here.
fn purchase_quantity(planned: i64, stack: u8, room: i32, price: i32, gold: i32) -> i32 {
    let afford = if price > 0 { gold / price } else { i32::from(stack) };
    let planned = i32::try_from(planned.clamp(1, i64::from(u8::MAX))).unwrap_or(1);
    planned.min(i32::from(stack)).min(room).min(afford).max(1)
}

/// borg-reborn-rs (#507): the gold a purchase took, from the purse before and
/// after it; the price of one times the copies when the purse did not fall
/// (a Home take, which costs nothing, or no observed change).
fn paid_for(before: i32, after: i32, unit: i32, copies: i32) -> i32 {
    if after < before { before - after } else { unit.saturating_mul(copies) }
}

/// The copies the pending live purchase asks for; 1 when there is none.
pub fn buy_quantity(st: &BorgState) -> i32 {
    st.think.town_live.pending_buy.as_ref().map_or(1, |a| a.qty)
}

fn clear(st: &mut BorgState) {
    st.borg.goal.shop = -1;
    st.borg.goal.ware = -1;
    st.borg.goal.item = -1;
    st.think.town_action = None;
}

/// Map model actions to shop goals. Home disposal withdraws the stack
/// first so the next solve can sell or drop it from the pack.
fn goal(a: TownAction) -> Option<(i16, i16, i16)> {
    let slot = i16::try_from(a.at[1]).ok()?;
    if slot < 0 || a.at[2] <= 0 { return None; }
    match a.code {
        action::BUY if (0..7).contains(&a.at[0]) => Some((a.at[0] as i16, slot, -1)),
        action::SELL if (0..7).contains(&a.at[0]) => Some((a.at[0] as i16, -1, slot)),
        action::STORE => Some((7, -1, slot)),
        action::TAKE_HOME | action::SELL_HOME | action::DROP_HOME => Some((7, slot, -1)),
        action::DROP => Some((-1, -1, slot)),
        _ => None,
    }
}

fn visit_gate(st: &mut BorgState) -> Option<bool> {
    if st.borg.tr(BI_CDEPTH) != 0 { return Some(false); }
    if st.cfg(BORG_TOWN_NO_STORES) != 0 { clear(st); return Some(false); }
    if let Some(shop) = st.borg.shop_override {
        clear(st);
        st.borg.goal.shop = shop;
        return Some(true);
    }
    None
}

fn set_action(st: &mut BorgState, a: TownAction) -> bool {
    let Some((shop, ware, item)) = goal(a) else { clear(st); return false; };
    st.borg.goal.shop = shop;
    st.borg.goal.ware = ware;
    st.borg.goal.item = item;
    st.think.town_action = Some(a);
    true
}

pub fn choose_overflow(st: &mut BorgState) -> bool {
    if st.borg.tr(BI_CDEPTH) != 0 || !active(st) || configured_limit(st) <= 0
        || st.cfg(BORG_TOWN_NO_STORES) != 0 || st.borg.shop_override.is_some()
        // borg-reborn-rs (#1140): the poison triage goes first.
        || crate::borg_poison::town_triage_holds(st) {
        return false;
    }
    let Some(action) = overflow_action(st) else { return false; };
    settle_sale(st);
    if sale_rejected(st, action.at[0] as i16, &st.items.items[action.at[1] as usize]) {
        return overflow_action(st).is_some_and(|a| set_action(st, a));
    }
    set_action(st, action)
}

pub fn choose(st: &mut BorgState) -> bool {
    settle_sale(st);
    if let Some(result) = visit_gate(st) { return result; }
    // borg-reborn-rs (#1140, `deal_with_poison`): the poison first, and no
    // sale or excess trip while selling pays nothing during it.
    if let Some(a) = crate::borg_poison::town_triage_action(st) { return set_action(st, a); }
    let no_sale = crate::borg_poison::no_sale_now(st);
    if let Some(a) = overflow_action(st).filter(|_| !no_sale) { return set_action(st, a); }
    if st.borg.time.now.wrapping_sub(st.borg.time.level) > 2000
        || st.borg.antibounce_count > 1350 { clear(st); return false; }
    // While walking, retain only the destination. On entering a store the
    // plan is recomputed against freshly read inventory and stock indices.
    if !st.borg.in_shop && st.borg.goal.shop >= 0 && st.think.town_action.is_some() {
        return true;
    }
    let mut a = crate::borg_town_shadow::live_action(st);
    // The opt-in trial tests discarding surplus plain shots before the
    // planner disposes of other kit. Poison triage and purchases go first.
    if !st.borg.in_shop && !no_sale && matches!(a.code, action::LEAVE | action::DROP | action::SELL | action::STORE) {
        let reserve = crate::params::value(crate::params::Param::town_quiver_trim_reserve);
        if reserve > 0 {
            let allowance = crate::borg_town_budget::Facts::of(st).allowance();
            if allowance < 0 {
                let shortfall = crate::borg_town_utility::departure_room().weight - allowance;
                if let Some(slot) = crate::borg_junk::town_quiver_surplus(st, reserve, shortfall) {
                    a = TownAction { code: action::DROP, at: [-1, slot as i64, i64::from(st.items.items[slot].iqty), 0] };
                }
            }
        }
    }
    if a.code == action::LEAVE {
        if let Some(refill) = crate::borg_healing_restock::purchase(st) {
            return set_action(st, refill);
        }
        note_departure_room(st);
    }
    set_action(st, a)
}

/// borg-reborn-rs (#859): what a departure leaves free, against the room
/// `town_departure_free_weight` and `town_departure_free_slots` ask for, in
/// the messages a zoo run counts. Nothing when both are off.
fn note_departure_room(st: &mut BorgState) {
    let room = crate::borg_town_utility::departure_room();
    if room == crate::borg_town_utility::Room::default() { return; }
    let level = st.borg.time.level;
    if st.think.town_live.room_noted_level.replace(level) == Some(level) { return; }
    let facts = crate::borg_town_budget::Facts::of(st);
    let pack = &st.items.items[..crate::borg_item::inven_wield() as usize];
    let slots = facts.pack_capacity - crate::borg_town_stock::pack_slots_used_of(pack);
    let note = format!("# Town departure room: weight {} free (keep {}), slots {} free (keep {})",
        facts.allowance(), room.weight, slots, room.slots);
    borg_note(st, &note);
}

pub fn execute_store(st: &mut BorgState) -> bool {
    let Some(a) = st.think.town_action else { return false; };
    if st.borg.goal.shop != st.think.shop_num || st.think.shop_num < 0 { return false; }
    let home_disposal = matches!(a.code, action::SELL_HOME | action::DROP_HOME);
    if home_disposal || matches!(a.code, action::BUY | action::TAKE_HOME) {
        let shop = st.think.shop_num as usize;
        let slot = a.at[1] as usize;
        let Some(item) = st.store.shops[shop].ware.get(slot).copied().filter(|i| i.iqty > 0) else { clear(st); return false; };
        let offer = crate::engine::store_stock(&st.access, shop).get(slot)
            .copied();
        let gold = st.borg.tr(crate::borg_headers::borg_trait_h::BI_GOLD);
        let reserve = st.cfg(crate::borg_headers::borg_h::BORG_SUM_EXCLUDED_FROM_BLACK_MARKET);
        // The configured share, not the purse threshold (#1103): the plan this
        // action comes from was made at this purse (it is rebuilt after every
        // action) and has already closed the Black Market to ordinary purchases
        // below the threshold; an urgent one keeps the cap (review of #1112, F1).
        let percentage = st.cfg(crate::borg_headers::borg_h::BORG_BLACK_MARKET_MAX_PERCENTAGE);
        let protects_gold = shop == crate::borg_town_stock::BORG_BLACK_MARKET && (reserve > 0 || percentage >= 0);
        let mut purchase_gold = if protects_gold {
            crate::borg_town_budget::black_market_budget(i64::from(gold), i64::from(reserve), i64::from(percentage), 0) as i32
        } else { gold };
        let recall_floor = crate::borg_store_buy::recall_purchase_floor();
        // #859: an urgent Recall (the dive gate's) is not held back by the floor.
        let protects_recall = a.code == action::BUY && recall_floor > 0
            && crate::borg_town_budget::is_recall_scroll(st, item.tval, item.sval)
            && !crate::borg_town_utility::recall_urgent(st);
        if protects_recall {
            purchase_gold = purchase_gold.min(crate::borg_store_buy::recall_purchase_budget(i64::from(gold), recall_floor) as i32);
        }
        let room_under_cap = if home_disposal { i32::MAX }
            else { acquisition_room(st, item.tval, item.kind) };
        let mut fits = room_under_cap > 0 && offer.is_some_and(|o| o.carry_capacity() > 0);
        let mut qty = 1;
        if let Some(offer) = offer.filter(|_| fits && a.code == action::TAKE_HOME) {
            qty = purchase_quantity(a.at[2], item.iqty, offer.carry_capacity().min(room_under_cap), 0, 0);
        }
        if let Some(offer) = offer.filter(|_| fits && a.code == action::BUY) {
            qty = purchase_quantity(a.at[2], item.iqty, offer.carry_capacity().min(room_under_cap),
                offer.purchase_quote(shop), purchase_gold);
            if protects_gold || protects_recall {
                let priced = (1..=qty).rev().find(|&n| offer.purchase_quote_quantity(shop, n) <= purchase_gold);
                fits = priced.is_some();
                qty = priced.unwrap_or(1);
            }
        }
        if let Some(offer) = offer.filter(|_| home_disposal) {
            if a.code == action::DROP_HOME || batch_home_disposal(st, &item) {
                qty = purchase_quantity(a.at[2], item.iqty, offer.carry_capacity(), 0, 0);
            }
        }
        let attempt = BuyAttempt { offer: SaleStamp::of(shop as i16, slot, &item),
            context: CarryContext::of(st), item, keep: !home_disposal, qty };
        if !fits {
            st.think.town_live.rejected_buys.push(attempt);
            borg_note(st, "# Town plan: native carrying preflight rejected purchase");
            clear(st);
            return false;
        }
        st.think.town_live.pending_buy = Some(attempt);
    }
    borg_note(st, &format!("# Town plan executing action {} at shop {} slot {}", a.code, st.think.shop_num, a.at[1]));
    let moved = HomeMove::pending(st, a);
    let executed = match a.code {
        action::BUY | action::TAKE_HOME => crate::borg_store_buy::borg_think_shop_buy(st),
        action::SELL_HOME | action::DROP_HOME => crate::borg_store_buy::take_selected_for_disposal(st),
        action::SELL | action::STORE => {
            let slot = a.at[1] as usize;
            let stamp = SaleStamp::of(st.think.shop_num, slot, &st.items.items[slot]);
            let sold = crate::borg_store_sell::sell_selected_quantity(st, a.at[2] as i32);
            if sold { st.think.town_live.pending_sale = Some(stamp); }
            sold
        }
        _ => false,
    };
    if executed {
        st.think.town_live.pending_home = moved;
        st.think.town_action = None;
    }
    else { st.think.town_live.pending_buy = None; }
    executed
}

/// The copies a planned drop types: the plan's split when the stack may be
/// split (it is one of `splits`' kinds) and the split leaves some carried,
/// else the whole stack.
fn drop_quantity(planned: i64, qty: i64, splits: bool) -> i64 {
    if splits && (1..qty).contains(&planned) { planned } else { qty }
}

/// A drop is an outside action, not a request to route to shop -1.
pub fn execute_outside(st: &mut BorgState) -> bool {
    if !active(st) || st.borg.in_shop { return false; }
    let Some(a) = st.think.town_action else { return false; };
    if a.code != action::DROP { return false; }
    let slot = a.at[1] as usize;
    let Some(item) = st.items.items.get(slot) else { clear(st); return false; };
    let qty = i64::from(item.iqty);
    if qty == 0 { clear(st); return false; }
    if slot >= crate::borg_item::quiver_start() as usize {
        let reserve = crate::params::value(crate::params::Param::town_quiver_trim_reserve);
        if reserve == 0 || a.at[2] != qty { clear(st); return false; }
        crate::borg_junk::town_drop_quiver_stack(st, slot, qty);
        clear(st);
        return true;
    }
    if slot >= crate::borg_item::inven_wield() as usize { clear(st); return false; }
    // A split Restore Life Levels stack leaves the protected copies carried.
    // borg-reborn-rs (#1151, review of #1155): so does a split fuel stack
    // (torches, flasks), whose floor is the light reserve. Other drops stay
    // whole: a split applied to every drop moved a smoke game (review of #989).
    let fuel = matches!(u32::from(item.tval),
        crate::headers::list_tvals_h::TV_LIGHT | crate::headers::list_tvals_h::TV_FLASK);
    let splits = (crate::borg_item_use::is_restore_life(item, st.item_val.sv_potion_restore_life)
        && crate::borg_town_capability::restore_life_reserve(st) > 0) || home_at_limit(st, item) || fuel;
    let drop_qty = drop_quantity(a.at[2], qty, splits);
    if i64::from(crate::borg_store_sell::disposable_book_quantity(st, item, drop_qty as i32)) < drop_qty {
        clear(st);
        return false;
    }
    let letter = crate::engine::all_letters_nohjkl(slot as i32) as i8 as u32;
    borg_note(st, "# Town plan dropping worthless stack");
    borg_keypress(st, u32::from(b'{'));
    borg_keypress(st, letter);
    borg_keypresses(st, b"borg ignore");
    borg_keypress(st, KC_ENTER);
    borg_keypress(st, u32::from(b'd'));
    borg_keypress(st, letter);
    if qty > 1 {
        borg_keypresses(st, drop_qty.to_string().as_bytes());
        borg_keypress(st, KC_ENTER);
    }
    clear(st);
    true
}

/// Controlled refreshed-inventory observations for the native regression fixture.
/// UI commands are queued by production code, then discarded by the fixture.
#[cfg(feature = "test-town-review")]
pub(crate) fn test_home_attempt(st: &mut BorgState, stored: bool, success: bool, kind: u32) -> u32 {
    *st = BorgState::default();
    st.core.cfg = crate::borg_init::BORG_SETTINGS.iter().map(|s| s.default_value).collect();
    st.core.cfg[BORG_TOWN_PLAN as usize] = 1;
    st.borg.trait_ = vec![0; crate::borg_headers::borg_trait_h::BI_MAX as usize];
    st.borg.time.level = 100;
    st.think.town_live.clock = Some(100);
    let mut item = crate::borg_item::blank_item();
    item.kind = kind;
    item.tval = crate::headers::list_tvals_h::TV_FLASK as u8;
    item.iqty = 2;
    st.items.items = vec![if stored { item } else { crate::borg_item::blank_item() }];
    st.store.shops = (0..8).map(|_| crate::borg_store::BorgShop::default()).collect();
    st.store.shops[BORG_HOME].ware = vec![item];
    st.borg.goal.shop = 7;
    st.borg.goal.ware = if stored { -1 } else { 0 };
    st.borg.goal.item = if stored { 0 } else { -1 };
    st.think.shop_num = 7;
    st.borg.in_shop = true;
    st.think.town_action = Some(TownAction {
        code: if stored { action::STORE } else { action::TAKE_HOME }, at: [7, 0, 1, 0],
    });
    crate::borg_io::borg_init_io(st);
    let queued = execute_store(st);
    let before = home_moved(st, kind, stored);
    if success {
        item.iqty = 1;
        st.items.items[0] = item;
    }
    settle_sale(st);
    let after = home_moved(st, kind, stored);
    let rejected = !stored && buy_rejected(st, 7, 0);
    // The new visit must clear both confirmed and pending history.
    st.borg.time.level += 1;
    settle_sale(st);
    let reset = !home_moved(st, kind, stored) && st.think.town_live.pending_home.is_none();
    crate::borg_io::borg_free_io(st);
    u32::from(queued) | (u32::from(before) << 1) | (u32::from(after) << 2)
        | (u32::from(rejected) << 3) | (u32::from(reset) << 4)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn a_queued_sale_requires_observed_inventory_progress() {
        let mut it = crate::borg_item::blank_item();
        it.iqty = 3;
        it.value = 20;
        let stamp = SaleStamp::of(4, 0, &it);
        assert!(stamp.matches(&it), "unchanged item is not a completed sale");
        it.iqty = 2;
        assert!(!stamp.matches(&it), "one copy really left");
        it.iqty = 3;
        it.desc[0] = b'K' as core::ffi::c_char;
        assert!(!stamp.matches(&it), "a different item in the slot must not be rejected");
    }
    #[test]
    fn a_planned_purchase_types_its_copies_capped_by_the_native_offer() {
        // 120 arrows planned, 99 in the stack, room for 80, 1 gold each, 200 gold.
        assert_eq!(purchase_quantity(120, 99, 80, 1, 200), 80, "the pack's room caps it");
        assert_eq!(purchase_quantity(120, 40, 80, 1, 200), 40, "the shop's stack caps it");
        assert_eq!(purchase_quantity(120, 99, 80, 3, 200), 66, "the gold at the price of one caps it");
        assert_eq!(purchase_quantity(5, 99, 80, 1, 200), 5, "the plan's copies when nothing binds");
        // Never 0: the prompt is skipped at 1 and the native code refuses
        // what cannot be paid for or carried.
        assert_eq!(purchase_quantity(0, 99, 80, 1, 200), 1);
        assert_eq!(purchase_quantity(5, 99, 80, 50, 20), 1);
        assert_eq!(purchase_quantity(5, 99, 0, 1, 200), 1);
        assert_eq!(purchase_quantity(5, 99, 80, 0, 0), 5, "a free ware is capped by the stack only");
    }
    /// #1151 (review of #1155): a split drop of a fuel stack no shop buys
    /// leaves the light reserve carried: 30 of 34 torches, not all 34.
    /// Kinds that may not split, and a plan for the whole stack or more, drop
    /// the whole stack.
    #[test]
    fn a_split_drop_keeps_the_reserved_copies() {
        assert_eq!(drop_quantity(30, 34, true), 30);
        assert_eq!(drop_quantity(30, 34, false), 34, "a kind that may not split");
        assert_eq!(drop_quantity(34, 34, true), 34);
        assert_eq!(drop_quantity(0, 34, true), 34);
        assert_eq!(drop_quantity(1, 1, true), 1);
    }
    #[test]
    fn a_purchase_logs_the_gold_it_took() {
        // 8 wands with uneven charges: one is 52 gold, not 70 (review F2).
        assert_eq!(paid_for(500, 448, 70, 1), 52);
        assert_eq!(paid_for(200, 178, 1, 22), 22, "22 arrows at 1 gold");
        assert_eq!(paid_for(200, 200, 0, 3), 0, "a Home take costs nothing");
    }
    #[test]
    fn the_pending_purchase_reports_its_copies() {
        let mut st = state();
        assert_eq!(buy_quantity(&st), 1, "no pending purchase asks for one");
        let item = crate::borg_item::blank_item();
        st.think.town_live.pending_buy = Some(BuyAttempt { offer: SaleStamp::of(1, 0, &item),
            context: CarryContext::of(&st), item, keep: true, qty: 40 });
        assert_eq!(buy_quantity(&st), 40);
    }
    fn state() -> BorgState {
        let mut st = BorgState::default();
        st.core.cfg = crate::borg_init::BORG_SETTINGS.iter().map(|s| s.default_value).collect();
        st.borg.trait_ = vec![0; crate::borg_headers::borg_trait_h::BI_MAX as usize];
        st.core.cfg[BORG_TOWN_PLAN as usize] = 1;
        st
    }
    #[test]
    fn disabled_stores_cancel_a_live_destination() {
        let mut st = state();
        st.core.cfg[BORG_TOWN_NO_STORES as usize] = 1;
        st.borg.goal.shop = 4;
        st.think.town_action = Some(TownAction { code: action::BUY, at: [4, 2, 1, 10] });
        assert_eq!(visit_gate(&mut st), Some(false));
        assert_eq!(st.borg.goal.shop, -1);
        assert!(st.think.town_action.is_none());
    }
    #[test]
    fn manual_shop_override_cancels_cached_actions() {
        let mut st = state();
        st.borg.shop_override = Some(7);
        st.think.town_action = Some(TownAction { code: action::BUY, at: [4, 2, 1, 10] });
        assert_eq!(visit_gate(&mut st), Some(true));
        assert_eq!((st.borg.goal.shop, st.borg.goal.ware, st.borg.goal.item), (7, -1, -1));
        assert!(st.think.town_action.is_none());
    }
    #[test]
    fn actions_route_pack_and_home_indices_to_the_correct_ui() {
        let a = |code, shop, slot| TownAction { code, at: [shop, slot, 1, 0] };
        assert_eq!(goal(a(action::BUY, 4, 2)), Some((4, 2, -1)));
        assert_eq!(goal(a(action::SELL, 1, 3)), Some((1, -1, 3)));
        assert_eq!(goal(a(action::STORE, 7, 3)), Some((7, -1, 3)));
        assert_eq!(goal(a(action::TAKE_HOME, 7, 5)), Some((7, 5, -1)));
        assert_eq!(goal(a(action::SELL_HOME, 4, 5)), Some((7, 5, -1)));
        assert_eq!(goal(a(action::DROP_HOME, 7, 5)), Some((7, 5, -1)));
        assert_eq!(goal(a(action::DROP, -1, 3)), Some((-1, -1, 3)));
        assert_eq!(goal(a(action::LEAVE, -1, -1)), None);
        assert_eq!(goal(a(action::BUY, 99, 2)), None);
        assert_eq!(goal(a(action::SELL, 1, -1)), None);
    }
    #[test]
    fn home_moves_are_remembered_per_direction() {
        let mut st = state();
        st.think.town_live.home_moves.push((42, true));
        assert!(home_moved(&st, 42, true));
        assert!(!home_moved(&st, 42, false), "only the executed direction is recorded");
        assert!(!home_moved(&st, 43, true));
        // A new visit rebuilds LiveState (settle_sale), dropping the moves.
        st.think.town_live = LiveState::default();
        assert!(!home_moved(&st, 42, true));
    }
    fn home_state() -> BorgState {
        let mut st = state();
        let mut item = crate::borg_item::blank_item();
        item.kind = 42;
        item.iqty = 3;
        st.items.items = vec![item, crate::borg_item::blank_item()];
        st.store.shops = (0..8).map(|_| crate::borg_store::BorgShop::default()).collect();
        st.store.shops[BORG_HOME].ware = vec![item];
        st
    }
    #[test]
    fn home_history_waits_for_successful_store_and_take() {
        for (code, stored, after) in [(action::STORE, true, 2), (action::TAKE_HOME, false, 4),
            (action::DROP_HOME, false, 6)] {
            let mut st = home_state();
            st.think.town_live.pending_home = HomeMove::pending(&st, TownAction { code, at: [7, 0, 1, 0] });
            assert!(!home_moved(&st, 42, stored), "queuing is not confirmation");
            st.items.items[0].iqty = after;
            // A refreshed inventory may put the stack in another slot.
            st.items.items.swap(0, 1);
            settle_home(&mut st);
            assert!(home_moved(&st, 42, stored));
            assert!(!home_moved(&st, 42, !stored));
            assert!(st.think.town_live.pending_home.is_none());
            settle_home(&mut st);
            assert_eq!(st.think.town_live.home_moves.len(), 1);
        }
    }
    #[test]
    fn canceled_home_moves_do_not_constrain_the_visit() {
        for code in [action::STORE, action::TAKE_HOME, action::DROP_HOME] {
            let mut st = home_state();
            st.think.town_live.pending_home = HomeMove::pending(&st, TownAction { code, at: [7, 0, 1, 0] });
            st.items.items.swap(0, 1);
            settle_home(&mut st);
            assert!(st.think.town_live.home_moves.is_empty());
            assert!(st.think.town_live.pending_home.is_none());
            // Later unrelated progress cannot resurrect a rejected attempt.
            st.items.items[1].iqty = 2;
            settle_home(&mut st);
            assert!(st.think.town_live.home_moves.is_empty());
        }
    }

    #[test]
    fn rejected_buys_are_scoped_to_the_offer_and_carrying_context() {
        let mut st=state();st.items.items=vec![crate::borg_item::blank_item()];
        st.store.shops=(0..8).map(|_|crate::borg_store::BorgShop::default()).collect();
        st.store.shops[4].ware=vec![crate::borg_item::blank_item()];st.store.shops[4].ware[0].iqty=3;
        let item=st.store.shops[4].ware[0];
        st.think.town_live.rejected_buys.push(BuyAttempt {offer:SaleStamp::of(4,0,&item),context:CarryContext::of(&st),item,keep:true,qty:1});
        assert!(buy_rejected(&st,4,0));st.items.items[0].iqty=1;assert!(!buy_rejected(&st,4,0));
        st.items.items[0].iqty=0;st.store.shops[4].ware[0].iqty=2;assert!(!buy_rejected(&st,4,0));
    }

}


#[cfg(test)]
mod departure_limit_tests {
    use super::*;
    use crate::headers::list_tvals_h::{TV_ARROW, TV_BOLT, TV_POTION, TV_SHOT};

    #[test]
    fn limits_the_total_across_stacks_and_exempts_all_missiles() {
        let mut full = crate::borg_item::blank_item();
        full.kind = 1;
        full.tval = TV_POTION as u8;
        full.iqty = 40;
        let partial = crate::borg_headers::borg_item_h::borg_item { iqty: 6, ..full };
        let items = [full, partial];
        assert_eq!(excess_copies(&items, &full, 35), 11);
        assert_eq!(excess_copies(&items, &partial, 35), 6);
        assert_eq!(excess_copies(&items, &full, 30), 16);
        assert_eq!(excess_copies(&items, &full, 0), 0);
        for missile in [TV_ARROW, TV_BOLT, TV_SHOT] {
            let item = crate::borg_headers::borg_item_h::borg_item { tval: missile as u8, ..full };
            assert_eq!(excess_copies(&[item], &item, 35), 0);
        }
    }
}
