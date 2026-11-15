//! The town-visit dump: an observation-only record of every town visit.
//!
//! Enabled with the environment variable `BORG_TOWN_DUMP=<file>`. When it is
//! unset the module does nothing at all: no file is opened, no decision is
//! recorded, and the run is the run without it.
//!
//! The dump is observation only. It never presses a key, never changes a
//! goal, never changes a setting and never changes what the Borg decides.
//! The keypress stream of a run with the dump enabled is therefore the same
//! as the stream of a run without it, so the C/Rust parity suites stay
//! valid.
//!
//! Two kinds of record are written, one JSON object per line:
//!
//! * `decision` - the town visit itself: the state (gold, depth, class,
//!   race, level, hp, the traits the town strategy reads, the pack, the
//!   equipment and the home), the stock of every shop with the price of
//!   every ware, and the decision the current policy took (the goal ware or
//!   item, the shop, and the Borg's own note saying why).
//! * `purchase` - a transaction that was actually carried out: the ware, the
//!   price and the gold before and after.
//!
//! The dump's job is to be the fixture the usefulness function is checked
//! against (real states, not a labelling input: the function is the
//! weighted deficit model of `borg_town_usefulness`, decided 2026-11-12,
//! not a table fitted to the dump) and the diff against the current
//! policy: the states are real, and the
//! disagreements come out on their own. So the record keeps the whole shop
//! stock (the wares that were *not* bought are the contrastive signal),
//! and it keeps the non-purchases (a decision to buy nothing is a
//! decision).
//!
//! ## The integer projection
//!
//! `BORG_TOWN_DUMP_INTEGERS=<file>` writes a second file: the same records
//! with **integers only**, a fixed field order and a version tag. The
//! descriptive records carry names, and a name is not comparable between two
//! runs; an integer is.
//!
//! That is the shape the record schema (P13) asks for: a versioned,
//! machine-readable line, integers only, a stable field order, a total
//! ordering. So the projection is here to be specified against, and to make
//! the two sides of a shadow-mode comparison (today's action and the plan's)
//! diffable as numbers.
//!
//! The projection is not the record schema: P13 wants the line to carry the
//! plan's decision as well, and to be replayable from its own logged
//! inputs. This one carries today's side only.

use std::cell::RefCell;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, OnceLock};

use crate::borg_headers::borg_item_h::borg_item;
use crate::borg_headers::borg_trait_h::{
    BI_ACUREPOIS, BI_CDEPTH, BI_CLEVEL, BI_FOOD, BI_GOLD, BI_ISCUT, BI_ISPOISONED, BI_LIGHT,
};
use crate::borg_item::inven_wield;
use crate::borg_store::BORG_HOME;
use crate::engine;
use crate::state::BorgState;

/// The version of the integer record's schema.
///
/// Version 1 was the draft projection: the named traits the town strategy
/// reads, and no awareness. Version 2 is the **frozen** shape (P13): the
/// whole trait vector, sparsely, and the awareness of every item, so the
/// record carries everything the usefulness function reads and a decision
/// can be replayed from the record alone.
///
/// The version tag is there for exactly this: a file that claims version 1
/// is read as version 1, whatever wrote it.
const INTEGER_SCHEMA: i64 = 2;

/// The version of the state record's schema.
///
/// Version 3 is the **state record**: the decision record's shape with no
/// decision in it (`action` is `-1`), and the **game's** turn rather than
/// the Borg's. The game's turn is the clock the message log is on, and the
/// message log is where a human player's purchases are - so a state and the
/// choice made in it have to be on the same clock.
///
/// The version tag is what tells a reader which shape a file is.
const STATE_SCHEMA: i64 = 3;

/// The state file's record number.
static STATE_SEQ: AtomicU64 = AtomicU64::new(0);

thread_local! {
    /// The last state written, so the same state is not written twice.
    static LAST_STATE: RefCell<String> = const { RefCell::new(String::new()) };
}

/// Record the state the character is in, while in town.
///
/// `BORG_TOWN_STATE=<file>` writes the states, not the decisions. A
/// human-in-the-loop session needs this: the human's choices come from the
/// game's messages, and the states they were made in come from here. The
/// decision dump cannot do it - with the null arm on (`borg_town_no_stores`)
/// the Borg makes no decision in town at all, which is exactly the point of
/// that arm.
///
/// One record per distinct state: the Borg thinks again and again while
/// the game waits for a key, and the state does not change between those
/// thinks. The state *does* change when the player buys, and that change
/// is what the next record carries.
pub fn dump_town_state(st: &mut BorgState) {
    let states = states_file();
    if states.is_none() {
        return;
    }
    /* Only in town: the baseline is about the town. */
    if st.borg.tr(BI_CDEPTH) != 0 {
        return;
    }

    /* The record number is taken only when the record is written, so the
     * file's seq stays contiguous. The Borg runs on the game's one thread,
     * so nothing takes a number between the load and the store. */
    let seq = STATE_SEQ.load(Ordering::Relaxed);
    let mut out = String::with_capacity(2048);
    push_integer_record_at(&mut out, st, seq, STATE_SCHEMA, i64::from(engine::game_turn()), false);
    /* The same state again: the Borg is thinking while the game waits for a
     * key. */
    let comparable = state_key(&out);
    let same = LAST_STATE.with(|last| {
        if last.borrow().as_str() == comparable {
            true
        } else {
            *last.borrow_mut() = comparable.to_string();
            false
        }
    });
    if same {
        return;
    }
    STATE_SEQ.store(seq + 1, Ordering::Relaxed);
    write_record(states, &out);
}

/// What makes two state records the same state: everything from the
/// clock on. The version and the seq come before it, and the seq is
/// bookkeeping - a key that kept it would never match (the first form of
/// this compared from the seq's *value* on, so it never deduplicated).
fn state_key(record: &str) -> &str {
    record.find(",\"turn\":").map_or(record, |at| &record[at..])
}

/// The action codes of the integer record.
mod action {
    /// No ware and no item: the Borg is only walking to a shop.
    pub const VISIT: i64 = 0;
    /// Buy a ware from a shop.
    pub const BUY: i64 = 1;
    /// Take a ware out of the Home.
    pub const TAKE_HOME: i64 = 2;
    /// Sell an item to a shop.
    pub const SELL: i64 = 3;
    /// Sell an item into the Home.
    pub const SELL_HOME: i64 = 4;
}

/// Open the dump file, once per process.
fn open_dump(path_var: &str) -> Option<&'static Mutex<File>> {
    static DESCRIPTIVE: OnceLock<Option<Mutex<File>>> = OnceLock::new();
    static INTEGERS: OnceLock<Option<Mutex<File>>> = OnceLock::new();
    static STATES: OnceLock<Option<Mutex<File>>> = OnceLock::new();
    static PLANS: OnceLock<Option<Mutex<File>>> = OnceLock::new();
    let cell = if path_var == "BORG_TOWN_DUMP" {
        &DESCRIPTIVE
    } else if path_var == "BORG_TOWN_DUMP_INTEGERS" {
        &INTEGERS
    } else if path_var == "BORG_TOWN_PLAN_RECORD" {
        &PLANS
    } else {
        &STATES
    };
    cell.get_or_init(|| {
        let Ok(path) = std::env::var(path_var) else {
            return None;
        };
        if path.is_empty() {
            return None;
        }
        match OpenOptions::new().create(true).append(true).open(&path) {
            Ok(file) => Some(Mutex::new(file)),
            Err(err) => {
                eprintln!("borg: {path_var} {path}: {err}");
                None
            }
        }
    })
    .as_ref()
}

/// The descriptive dump file.
fn dump_file() -> Option<&'static Mutex<File>> {
    open_dump("BORG_TOWN_DUMP")
}

/// The integer projection of the dump.
fn integers_file() -> Option<&'static Mutex<File>> {
    open_dump("BORG_TOWN_DUMP_INTEGERS")
}

/// The state file: the states the character was in while in town.
/// The plan records (version 4): the shadow plan at every town decision
/// (`borg_town_shadow`).
pub(crate) fn plan_record_file() -> Option<&'static Mutex<File>> {
    open_dump("BORG_TOWN_PLAN_RECORD")
}

fn states_file() -> Option<&'static Mutex<File>> {
    open_dump("BORG_TOWN_STATE")
}

/// The record number, over the whole run.
static SEQ: AtomicU64 = AtomicU64::new(0);

thread_local! {
    /// The Borg's own note about the decision it just took.
    static LAST_REASON: RefCell<Vec<u8>> = const { RefCell::new(Vec::new()) };
}

/// Remember the note the Borg made about its decision (`borg_think_store`'s
/// `note`). The dump reads it back when the decision is recorded.
pub fn record_reason(bytes: &[u8]) {
    LAST_REASON.with(|reason| *reason.borrow_mut() = bytes.to_vec());
}

/// Take the last note (and forget it).
fn take_reason() -> Vec<u8> {
    LAST_REASON.with(|reason| std::mem::take(&mut *reason.borrow_mut()))
}

/// Append one record.
pub(crate) fn write_record(file: Option<&Mutex<File>>, out: &str) {
    let Some(file) = file else { return };
    let Ok(mut file) = file.lock() else { return };
    let _ = file.write_all(out.as_bytes());
    let _ = file.write_all(b"\n");
}

/// A hex digit, lower case.
fn hex_digit(nibble: u8) -> char {
    match nibble {
        0..=9 => (b'0' + nibble) as char,
        _ => (b'a' + nibble - 10) as char,
    }
}

/// Append a JSON string. The bytes are the Borg's own bytes (a description
/// is a byte string, not necessarily UTF-8), so a byte outside ASCII is
/// written as its Latin-1 code point.
fn push_str(out: &mut String, bytes: &[u8]) {
    out.push('"');
    for &byte in bytes {
        match byte {
            b'"' => out.push_str("\\\""),
            b'\\' => out.push_str("\\\\"),
            b'\n' => out.push_str("\\n"),
            b'\r' => out.push_str("\\r"),
            b'\t' => out.push_str("\\t"),
            0x20..=0x7e => out.push(byte as char),
            _ => {
                out.push_str("\\u00");
                out.push(hex_digit(byte >> 4));
                out.push(hex_digit(byte & 0x0f));
            }
        }
    }
    out.push('"');
}

/// Append a JSON number.
fn push_num(out: &mut String, value: impl std::fmt::Display) {
    out.push_str(&value.to_string());
}

/// An item's description, up to its NUL.
fn desc_bytes(item: &borg_item) -> Vec<u8> {
    item.desc
        .iter()
        .take_while(|&&c| c != 0)
        .map(|&c| c as u8)
        .collect()
}

/// One item of the pack, the equipment, the home or a shop stock.
fn push_item(out: &mut String, item: &borg_item, slot: usize, price: bool) {
    out.push_str("{\"slot\":");
    push_num(out, slot);
    out.push_str(",\"desc\":");
    push_str(out, &desc_bytes(item));
    out.push_str(",\"qty\":");
    push_num(out, item.iqty);
    out.push_str(",\"tval\":");
    push_num(out, item.tval);
    out.push_str(",\"sval\":");
    push_num(out, item.sval);
    out.push_str(",\"value\":");
    push_num(out, item.value);
    if price {
        out.push_str(",\"price\":");
    } else {
        out.push_str(",\"cost\":");
    }
    push_num(out, item.cost);
    out.push_str(",\"aware\":");
    out.push_str(if item.aware { "true" } else { "false" });
    out.push_str(",\"needs_ident\":");
    out.push_str(if item.needs_ident { "true" } else { "false" });
    out.push('}');
}

/// One item as integers: slot, kind, subkind, quantity, and the value (the
/// pack, the equipment, the home) or the price (a ware), and the item's
/// awareness as a bitmask.
///
/// The awareness bits are what the knowledge need reads - whether the
/// character knows what the item is - so they travel with the item.
fn push_item_integers(out: &mut String, item: &borg_item, slot: usize, price: bool) {
    out.push('[');
    push_num(out, slot);
    out.push(',');
    push_num(out, item.tval);
    out.push(',');
    push_num(out, item.sval);
    out.push(',');
    push_num(out, item.iqty);
    out.push(',');
    push_num(out, if price { item.cost } else { item.value });
    out.push(',');
    push_num(out, awareness_flags(item));
    out.push(']');
}

/// The item's awareness, as a bitmask: bit 0 is "the character knows what
/// this is", bit 1 is "this item is not identified yet".
fn awareness_flags(item: &borg_item) -> i64 {
    let mut flags = 0i64;
    if item.aware {
        flags |= 1;
    }
    if item.needs_ident {
        flags |= 2;
    }
    flags
}

/// The character's traits, sparsely: every trait that is not zero, as
/// (index, value) pairs.
///
/// The whole vector, not the handful the town strategy reads today: the
/// usefulness function reads traits nobody has written down yet (the
/// poison-cure capability among them), and a record that only carries the
/// ones that happen to be read today cannot be replayed against one that
/// reads another. A trait that is absent is zero.
fn push_traits(out: &mut String, st: &BorgState) {
    out.push_str("\"traits\":[");
    let mut first = true;
    let mut traits: Vec<_> = st.borg.trait_.iter().enumerate()
        .map(|(i, value)| (crate::borg_town_traits::record_index(i), value)).collect();
    traits.sort_by_key(|&(index, _)| index);
    for (index, value) in traits {
        if *value == 0 {
            continue;
        }
        if !first {
            out.push(',');
        }
        first = false;
        out.push('[');
        push_num(out, index);
        out.push(',');
        push_num(out, value);
        out.push(']');
    }
    out.push(']');
}

/// The decision's action: the integer code and the name.
fn decision_action(shop: i16, ware: i16, item: i16) -> (i64, &'static str) {
    if ware >= 0 {
        if shop == BORG_HOME as i16 {
            (action::TAKE_HOME, "take_home")
        } else {
            (action::BUY, "buy")
        }
    } else if item >= 0 {
        if shop == BORG_HOME as i16 {
            (action::SELL_HOME, "sell_home")
        } else {
            (action::SELL, "sell")
        }
    } else {
        (action::VISIT, "visit")
    }
}

/// The state the town strategy reads.
fn push_state(out: &mut String, st: &BorgState) {
    let notice = engine::player_notice();
    let summary = engine::player_summary();

    out.push_str("\"turn\":");
    push_num(out, st.borg.time.now);
    out.push_str(",\"depth\":");
    push_num(out, st.borg.tr(BI_CDEPTH));
    out.push_str(",\"level\":");
    push_num(out, st.borg.tr(BI_CLEVEL));
    out.push_str(",\"hp\":[");
    push_num(out, notice.chp);
    out.push(',');
    push_num(out, notice.mhp);
    out.push(']');
    out.push_str(",\"gold\":");
    push_num(out, st.borg.tr(BI_GOLD));
    out.push_str(",\"race\":");
    push_str(out, summary.race_name);
    out.push_str(",\"class\":");
    push_str(out, summary.class_name);
    out.push_str(",\"name\":");
    push_str(out, &summary.full_name);
    out.push_str(",\"traits\":{\"light\":");
    push_num(out, st.borg.tr(BI_LIGHT));
    out.push_str(",\"food\":");
    push_num(out, st.borg.tr(BI_FOOD));
    out.push_str(",\"cut\":");
    push_num(out, st.borg.tr(BI_ISCUT));
    out.push_str(",\"poisoned\":");
    push_num(out, st.borg.tr(BI_ISPOISONED));
    /* The poison-cure capability: the plan's poison need reads it, and the
     * town strategy does not, so it would otherwise never be written down. */
    out.push_str(",\"acurepois\":");
    push_num(out, st.borg.tr(BI_ACUREPOIS));
    out.push('}');
}

/// The integer projection of one town visit.
///
/// Every value is an integer and the field order is fixed, so two runs'
/// projections diff as numbers rather than as text. The records are totally
/// ordered by `seq`. An absent value is `-1`, like `shop`, `ware` and
/// `item`: a class that is not set is `-1`, not a magic number.
fn push_integer_record(out: &mut String, st: &BorgState, seq: u64) {
    push_integer_record_at(out, st, seq, INTEGER_SCHEMA, i64::from(st.borg.time.now), true);
}

/// The integer projection of one record: the state, and the decision in it.
///
/// `version` is the record's schema version, `turn` the clock the record is
/// on, and `decision` says whether the record carries a decision. A record
/// without a decision is a **state record**: the state on its own, which is
/// what a human-in-the-loop session needs (the human's choices come from
/// the messages, the states they were made in come from here).
pub(crate) fn push_integer_record_at(
    out: &mut String,
    st: &BorgState,
    seq: u64,
    version: i64,
    turn: i64,
    decision: bool,
) {
    let notice = engine::player_notice();
    let shop = st.borg.goal.shop;
    let ware = st.borg.goal.ware;
    let item = st.borg.goal.item;

    out.push_str("{\"v\":");
    push_num(out, version);
    out.push_str(",\"seq\":");
    push_num(out, seq);
    out.push_str(",\"turn\":");
    push_num(out, turn);
    out.push_str(",\"depth\":");
    push_num(out, st.borg.tr(BI_CDEPTH));
    out.push_str(",\"level\":");
    push_num(out, st.borg.tr(BI_CLEVEL));
    out.push_str(",\"chp\":");
    push_num(out, notice.chp);
    out.push_str(",\"mhp\":");
    push_num(out, notice.mhp);
    out.push_str(",\"gold\":");
    push_num(out, st.borg.tr(BI_GOLD));
    out.push_str(",\"race\":");
    push_num(out, engine::race_ridx());
    out.push_str(",\"class\":");
    /* The absent value is -1, like every other absent value in the record. */
    push_num(
        out,
        engine::class_cidx_if_set().map(i64::from).unwrap_or(-1),
    );
    /* The whole trait vector, sparsely: the record carries what the
     * usefulness function reads, not what the town strategy reads today. */
    out.push(',');
    push_traits(out, st);
    out.push_str(",\"action\":");
    if decision {
        let (code, _) = decision_action(shop, ware, item);
        push_num(out, code);
    } else {
        /* No decision in this record: the state on its own. */
        push_num(out, -1);
    }
    out.push_str(",\"shop\":");
    if decision {
        push_num(out, shop);
    } else {
        push_num(out, -1);
    }
    out.push_str(",\"ware\":");
    if decision {
        push_num(out, ware);
    } else {
        push_num(out, -1);
    }
    out.push_str(",\"item\":");
    if decision {
        push_num(out, item);
    } else {
        push_num(out, -1);
    }
    out.push_str(",\"price\":");
    if decision && ware >= 0 {
        match st.store.shops.get(shop as usize).and_then(|s| s.ware.get(ware as usize)) {
            Some(w) => push_num(out, w.cost),
            None => push_num(out, -1),
        }
    } else {
        push_num(out, -1);
    }

    // The pack (slots 0..pack_size) and the equipment (the next twelve).
    let pack_end = inven_wield() as usize;
    let mut first = true;
    out.push_str(",\"pack\":[");
    for slot in 0..pack_end {
        let Some(item) = st.items.items.get(slot) else {
            break;
        };
        if item.iqty == 0 {
            continue;
        }
        if !first {
            out.push(',');
        }
        first = false;
        push_item_integers(out, item, slot, false);
    }
    out.push_str("],\"equip\":[");
    first = true;
    for slot in pack_end..pack_end + 12 {
        let Some(item) = st.items.items.get(slot) else {
            break;
        };
        if item.iqty == 0 {
            continue;
        }
        if !first {
            out.push(',');
        }
        first = false;
        push_item_integers(out, item, slot, false);
    }
    out.push(']');

    // The home.
    out.push_str(",\"home\":[");
    first = true;
    if let Some(home) = st.store.shops.get(BORG_HOME) {
        for (slot, item) in home.ware.iter().enumerate() {
            if item.iqty == 0 {
                continue;
            }
            if !first {
                out.push(',');
            }
            first = false;
            push_item_integers(out, item, slot, false);
        }
    }
    out.push(']');

    // The stock of every shop, with the price of every ware.
    out.push_str(",\"shops\":[");
    first = true;
    for (shop, stock) in st.store.shops.iter().enumerate() {
        if !first {
            out.push(',');
        }
        first = false;
        out.push('[');
        push_num(out, shop);
        out.push_str(",[");
        let mut first_ware = true;
        for (slot, item) in stock.ware.iter().enumerate() {
            if item.iqty == 0 {
                continue;
            }
            if !first_ware {
                out.push(',');
            }
            first_ware = false;
            push_item_integers(out, item, slot, true);
        }
        out.push_str("]]");
    }
    out.push(']');
    out.push('}');
}

/// Record one town visit: the state, the shop stock, and the decision the
/// current policy took with the Borg's own reason for it.
pub fn dump_town_visit(st: &mut BorgState) {
    let descriptive = dump_file();
    let integers = integers_file();
    if descriptive.is_none() && integers.is_none() {
        return;
    }
    let seq = SEQ.fetch_add(1, Ordering::Relaxed);
    let reason = take_reason();

    if descriptive.is_some() {
        let mut out = String::with_capacity(4096);
        out.push_str("{\"type\":\"decision\",\"seq\":");
        push_num(&mut out, seq);
        out.push(',');
        push_state(&mut out, st);

        // The pack (slots 0..pack_size) and the equipment (the next twelve).
        let pack_end = inven_wield() as usize;
        let mut first = true;
        out.push_str(",\"pack\":[");
        for slot in 0..pack_end {
            let Some(item) = st.items.items.get(slot) else {
                break;
            };
            if item.iqty == 0 {
                continue;
            }
            if !first {
                out.push(',');
            }
            first = false;
            push_item(&mut out, item, slot, false);
        }
        out.push_str("],\"equipment\":[");
        first = true;
        for slot in pack_end..pack_end + 12 {
            let Some(item) = st.items.items.get(slot) else {
                break;
            };
            if item.iqty == 0 {
                continue;
            }
            if !first {
                out.push(',');
            }
            first = false;
            push_item(&mut out, item, slot, false);
        }
        out.push(']');

        // The home.
        out.push_str(",\"home\":[");
        if let Some(home) = st.store.shops.get(BORG_HOME) {
            first = true;
            for (slot, item) in home.ware.iter().enumerate() {
                if item.iqty == 0 {
                    continue;
                }
                if !first {
                    out.push(',');
                }
                first = false;
                push_item(&mut out, item, slot, false);
            }
        }
        out.push(']');

        // The stock of every shop, with the price of every ware. The wares
        // that were not bought are the contrastive signal, so the whole
        // stock is kept.
        out.push_str(",\"shops\":[");
        first = true;
        for (shop, stock) in st.store.shops.iter().enumerate() {
            if !first {
                out.push(',');
            }
            first = false;
            out.push_str("{\"shop\":");
            push_num(&mut out, shop);
            out.push_str(",\"name\":");
            push_str(&mut out, &engine::store_feat_name(shop));
            out.push_str(",\"wares\":[");
            let mut first_ware = true;
            for (slot, item) in stock.ware.iter().enumerate() {
                if item.iqty == 0 {
                    continue;
                }
                if !first_ware {
                    out.push(',');
                }
                first_ware = false;
                push_item(&mut out, item, slot, true);
            }
            out.push_str("]}");
        }
        out.push(']');

        // The decision.
        let shop = st.borg.goal.shop;
        let ware = st.borg.goal.ware;
        let item = st.borg.goal.item;
        let (_, action_name) = decision_action(shop, ware, item);

        out.push_str(",\"decision\":{\"action\":");
        push_str(&mut out, action_name.as_bytes());
        out.push_str(",\"shop\":");
        push_num(&mut out, shop);
        out.push_str(",\"shop_name\":");
        if shop >= 0 && usize::from(shop as u16) < st.store.shops.len() {
            push_str(&mut out, &engine::store_feat_name(shop as usize));
        } else {
            push_str(&mut out, b"");
        }
        out.push_str(",\"ware\":");
        push_num(&mut out, ware);
        out.push_str(",\"ware_desc\":");
        if ware >= 0 {
            if let Some(shops) = st.store.shops.get(shop as usize) {
                if let Some(w) = shops.ware.get(ware as usize) {
                    push_str(&mut out, &desc_bytes(w));
                } else {
                    push_str(&mut out, b"");
                }
            } else {
                push_str(&mut out, b"");
            }
        } else {
            push_str(&mut out, b"");
        }
        out.push_str(",\"item\":");
        push_num(&mut out, item);
        out.push_str(",\"item_desc\":");
        if item >= 0 {
            if let Some(it) = st.items.items.get(item as usize) {
                push_str(&mut out, &desc_bytes(it));
            } else {
                push_str(&mut out, b"");
            }
        } else {
            push_str(&mut out, b"");
        }
        out.push_str(",\"reason\":");
        push_str(&mut out, &reason);
        out.push_str("}}");

        write_record(descriptive, &out);
    }

    if integers.is_some() {
        let mut out = String::with_capacity(2048);
        push_integer_record(&mut out, st, seq);
        write_record(integers, &out);
    }
}

/// Record a purchase that was actually carried out.
pub fn dump_purchase(st: &mut BorgState, shop: i32, item: &borg_item) {
    let descriptive = dump_file();
    let integers = integers_file();
    if descriptive.is_none() && integers.is_none() {
        return;
    }

    let seq = SEQ.fetch_add(1, Ordering::Relaxed);
    if descriptive.is_some() {
        let mut out = String::with_capacity(512);
        out.push_str("{\"type\":\"purchase\",\"seq\":");
        push_num(&mut out, seq);
        out.push_str(",\"turn\":");
        push_num(&mut out, st.borg.time.now);
        out.push_str(",\"shop\":");
        push_num(&mut out, shop);
        out.push_str(",\"shop_name\":");
        if shop >= 0 && usize::from(shop as u16) < st.store.shops.len() {
            push_str(&mut out, &engine::store_feat_name(shop as usize));
        } else {
            push_str(&mut out, b"");
        }
        out.push_str(",\"gold_before\":");
        push_num(&mut out, st.borg.tr(BI_GOLD));
        out.push_str(",\"ware\":");
        push_item(&mut out, item, 0, true);
        out.push('}');

        write_record(descriptive, &out);
    }

    if integers.is_some() {
        let mut out = String::with_capacity(256);
        out.push_str("{\"v\":");
        push_num(&mut out, INTEGER_SCHEMA);
        out.push_str(",\"seq\":");
        push_num(&mut out, seq);
        out.push_str(",\"turn\":");
        push_num(&mut out, st.borg.time.now);
        out.push_str(",\"shop\":");
        push_num(&mut out, shop);
        out.push_str(",\"price\":");
        push_num(&mut out, item.cost);
        out.push_str(",\"gold_before\":");
        push_num(&mut out, st.borg.tr(BI_GOLD));
        out.push_str(",\"ware\":");
        push_item_integers(&mut out, item, 0, true);
        out.push('}');

        write_record(integers, &out);
    }
}

#[cfg(test)]
mod tests {
    use super::state_key;

    /// Two records of the same state that differ only in their seq have
    /// the same key: the seq is bookkeeping, not state.
    #[test]
    fn the_seq_is_not_part_of_the_state() {
        let a = r#"{"v":3,"seq":7,"turn":120,"depth":0,"gold":50}"#;
        let b = r#"{"v":3,"seq":8,"turn":120,"depth":0,"gold":50}"#;
        assert_eq!(state_key(a), state_key(b));
        assert_eq!(state_key(a), r#","turn":120,"depth":0,"gold":50}"#);
    }

    /// A change anywhere from the clock on is a different state.
    #[test]
    fn a_change_after_the_seq_is_a_new_state() {
        let a = r#"{"v":3,"seq":7,"turn":120,"depth":0,"gold":50}"#;
        assert_ne!(state_key(a), state_key(r#"{"v":3,"seq":8,"turn":120,"depth":0,"gold":49}"#));
        assert_ne!(state_key(a), state_key(r#"{"v":3,"seq":8,"turn":121,"depth":0,"gold":50}"#));
    }
}
