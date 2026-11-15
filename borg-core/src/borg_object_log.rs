//! The session's object log: the objects the Borg uses, buys and picks up.
//!
//! Enabled with the environment variable `BORG_OBJECT_LOG=<file>`. When it is
//! unset the module does nothing at all: no file is opened, no line is
//! printed, and the run is the run without it. The log never presses a key
//! and never changes a decision, so the keypress stream of a run with the log
//! is the stream of a run without it.
//!
//! Every event is printed on the console (the game's stdout) and appended to
//! the file, numbered from 1, so the end-of-session feedback can point at a
//! line:
//!
//! ```text
//! 0007 turn 142 depth 23 quaff a Potion of Healing
//! 0008 turn 150 depth 0 buy a Scroll of Light at the General Store, cost 25 gold
//! 0009 turn 150 depth 0 acquire 2 x a Potion of Healing
//! ```
//!
//! Three kinds of event:
//!
//! * `quaff`, `read`, `use`, `zap` - the Borg used an object it carried.
//! * `buy` - the Borg bought an object, with the store and the item's cost
//!   (the price the Borg judged it at, not the store's own price).
//! * `take` - the Borg took an object back from its home (no price).
//! * `acquire` - something appeared in the pack that was not there before.
//!   This is the player's side of the session: with the handover the player
//!   shops in town, and the pack is the only place that shows it. It cannot
//!   tell a purchase from a pickup, so it says `acquire` and not `buy`. The
//!   Borg's own purchases are not reported twice: they are the `buy` lines.
//!
//! Two items are the same kind when their `tval` and `sval` agree. The `pval`
//! is left out on purpose: for wands and staffs it is the number of charges,
//! so using one, recharging one, or learning its charges would otherwise
//! make it look like a new item in the pack.

use std::collections::BTreeMap;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, OnceLock};

use crate::borg_headers::borg_item_h::borg_item;
use crate::borg_headers::borg_trait_h::BI_CDEPTH;
use crate::state::BorgState;

/// The number of events logged so far.
static COUNT: AtomicU64 = AtomicU64::new(0);

/// The log file: `BORG_OBJECT_LOG`.
fn log_file() -> Option<&'static Mutex<File>> {
    static FILE: OnceLock<Option<Mutex<File>>> = OnceLock::new();
    FILE.get_or_init(|| {
        let path_var = "BORG_OBJECT_LOG";
        let Ok(path) = std::env::var(path_var) else {
            return None;
        };
        if path.is_empty() {
            return None;
        }
        // A file that is used again does not repeat its numbers: the count
        // starts where the file left off.
        match std::fs::read_to_string(&path) {
            Ok(text) => {
                let lines = text.lines().filter(|line| !line.trim().is_empty()).count();
                COUNT.store(lines as u64, Ordering::Relaxed);
            }
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => {}
            Err(err) => {
                eprintln!("borg: {path_var} {path}: {err}");
            }
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

/// The item's description up to its NUL (`item->desc`).
fn item_desc(item: &borg_item) -> String {
    let bytes: Vec<u8> = item
        .desc
        .iter()
        .take_while(|&&c| c != 0)
        .map(|&c| c as u8)
        .collect();
    String::from_utf8_lossy(&bytes).trim().to_string()
}

/// Write one event: on the console and in the file.
fn emit(st: &BorgState, action: &str, details: &str) {
    let Some(file) = log_file() else {
        return;
    };
    let n = COUNT.fetch_add(1, Ordering::Relaxed) + 1;
    let turn = st.borg.time.now;
    let depth = st.borg.tr(BI_CDEPTH);
    let line = format!("{n:04} turn {turn} depth {depth} {action} {details}\n");

    // The console: the owner watches the run and sees the objects go by.
    // A console that has gone away (a closed pipe) must not stop the game:
    // the error is ignored, where `print!` would panic.
    let _ = std::io::stdout().write_all(line.as_bytes());
    let _ = std::io::stdout().flush();

    if let Ok(mut file) = file.lock() {
        let _ = file.write_all(line.as_bytes());
        let _ = file.flush();
    }
}

/// The Borg used an object it carried (`quaff`, `read`, `use`, `zap`).
pub fn log_use(st: &BorgState, action: &str, item: &borg_item) {
    emit(st, action, &item_desc(item));
}

/// What makes two items the same kind of thing: `tval` and `sval`. Not the
/// `pval`, which counts a wand's or a staff's charges.
type Kind = (u8, u8);

/// The Borg bought an object: the store and the item's cost with it. In the
/// home nothing is bought: the Borg takes the object back, at no price.
pub fn log_buy(st: &BorgState, item: &borg_item, shop: &str, cost: i32, home: bool) {
    if home {
        emit(
            st,
            "take",
            &format!("{} from the {}", item_desc(item), shop),
        );
    } else {
        emit(
            st,
            "buy",
            &format!("{} at the {}, cost {cost} gold", item_desc(item), shop),
        );
    }

    // The pack will show this purchase too; it is not an acquisition.
    let mut pending = pending_buys().lock().unwrap_or_else(|err| err.into_inner());
    pending.push(PendingBuy {
        kind: (item.tval, item.sval),
        count: 1,
        turn: st.borg.time.now,
    });
}

/// Something new appeared in the pack.
fn log_acquire(st: &BorgState, desc: &str, count: u32) {
    if count == 1 {
        emit(st, "acquire", desc);
    } else {
        emit(st, "acquire", &format!("{count} x {desc}"));
    }
}

/// One pack slot, as the snapshot keeps it: the kind, the quantity and the
/// description.
type Slot = (u8, u8, u8, String);

/// The pack as it was the last time we looked.
fn previous_pack() -> &'static Mutex<Option<Vec<Slot>>> {
    static PACK: OnceLock<Mutex<Option<Vec<Slot>>>> = OnceLock::new();
    PACK.get_or_init(|| Mutex::new(None))
}

/// The pack now: the slots that hold something.
fn current_pack(st: &BorgState) -> Vec<Slot> {
    let mut out = Vec::new();
    let n = i32::from(crate::engine::z_info().pack_size);
    for i in 0..n {
        let item = &st.items.items[i as usize];
        if item.iqty == 0 {
            continue;
        }
        out.push((item.tval, item.sval, item.iqty, item_desc(item)));
    }
    out
}

/// Count every kind in a pack snapshot. The map is ordered, so the report
/// is the same every time.
fn counts(slots: &[Slot]) -> BTreeMap<Kind, u32> {
    let mut out: BTreeMap<Kind, u32> = BTreeMap::new();
    for (tval, sval, qty, _) in slots {
        *out.entry((*tval, *sval)).or_insert(0) += u32::from(*qty);
    }
    out
}

/// What grew between two packs: the kind, and how much.
fn grown(old: &BTreeMap<Kind, u32>, new: &BTreeMap<Kind, u32>) -> Vec<(Kind, u32)> {
    new.iter()
        .filter_map(|(kind, count)| {
            let before = old.get(kind).copied().unwrap_or(0);
            (*count > before).then_some((*kind, count - before))
        })
        .collect()
}

/// A purchase the Borg made that the pack has not caught up with yet.
#[derive(Clone, Copy)]
struct PendingBuy {
    kind: Kind,
    count: u32,
    turn: i32,
}

/// How many turns a purchase can wait for the pack to show it.
const BUY_WINDOW: i32 = 3;

/// The purchases the pack has not caught up with yet.
fn pending_buys() -> &'static Mutex<Vec<PendingBuy>> {
    static PENDING: OnceLock<Mutex<Vec<PendingBuy>>> = OnceLock::new();
    PENDING.get_or_init(|| Mutex::new(Vec::new()))
}

/// Take the purchases of a kind that the pack has now caught up with.
///
/// A purchase that the pack never shows (the store refused it) is dropped
/// when it gets too old to be one.
fn take_pending(kind: &Kind, turn: i32, pending: &mut Vec<PendingBuy>) -> u32 {
    let mut taken = 0;
    pending.retain(|buy| {
        if turn > buy.turn && turn - buy.turn > BUY_WINDOW {
            return false;
        }
        if buy.kind == *kind && turn >= buy.turn {
            taken += buy.count;
            return false;
        }
        true
    });
    taken
}

/// Report what appeared in the pack since the last look.
///
/// The first look only takes the baseline: the starting kit is not an
/// acquisition. After that, every kind that grew is reported with the number
/// it grew by - less what the Borg bought (those are the `buy` lines).
pub fn log_acquisitions(st: &BorgState) {
    let now = current_pack(st);
    let mut previous = previous_pack()
        .lock()
        .unwrap_or_else(|err| err.into_inner());
    let Some(old) = previous.as_ref() else {
        *previous = Some(now);
        return;
    };

    let turn = st.borg.time.now;
    let old_counts = counts(old);
    let new_counts = counts(&now);
    let mut pending = pending_buys().lock().unwrap_or_else(|err| err.into_inner());

    for (kind, count) in grown(&old_counts, &new_counts) {
        // The Borg's own purchases are already logged: they are not
        // acquisitions.
        let bought = take_pending(&kind, turn, &mut pending);
        let acquired = count.saturating_sub(bought);
        if acquired == 0 {
            continue;
        }
        // The description of the kind: the first slot that holds it.
        if let Some((_, _, _, desc)) = now.iter().find(|(t, s, _, _)| (*t, *s) == kind) {
            log_acquire(st, desc, acquired);
        }
    }

    *previous = Some(now);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn slot(tval: u8, sval: u8, qty: u8, desc: &str) -> Slot {
        (tval, sval, qty, desc.to_string())
    }

    #[test]
    fn only_what_grew_is_reported() {
        let old = counts(&[slot(1, 2, 1, "a")]);
        let new = counts(&[slot(1, 2, 2, "a"), slot(4, 5, 1, "b")]);
        assert_eq!(grown(&old, &new), vec![((1, 2), 1), ((4, 5), 1)]);
    }

    #[test]
    fn the_report_is_in_a_fixed_order() {
        let old = counts(&[]);
        let new = counts(&[slot(9, 9, 1, "c"), slot(1, 1, 1, "a")]);
        let report: Vec<Kind> = grown(&old, &new)
            .into_iter()
            .map(|(kind, _)| kind)
            .collect();
        assert_eq!(report, vec![(1, 1), (9, 9)]);
    }

    #[test]
    fn a_wand_that_spent_a_charge_is_not_new() {
        // The description changes with the charges; the kind does not.
        let old = counts(&[slot(65, 3, 1, "a Wand of Magic Missile (5 charges)")]);
        let new = counts(&[slot(65, 3, 1, "a Wand of Magic Missile (4 charges)")]);
        assert!(grown(&old, &new).is_empty());
    }

    #[test]
    fn a_purchase_is_not_reported_twice() {
        let mut pending = vec![PendingBuy {
            kind: (1, 2),
            count: 1,
            turn: 10,
        }];
        assert_eq!(take_pending(&(1, 2), 11, &mut pending), 1);
        assert!(pending.is_empty());
    }

    #[test]
    fn a_purchase_of_another_kind_is_kept() {
        let mut pending = vec![PendingBuy {
            kind: (1, 2),
            count: 1,
            turn: 10,
        }];
        assert_eq!(take_pending(&(4, 5), 11, &mut pending), 0);
        assert_eq!(pending.len(), 1);
    }

    #[test]
    fn a_purchase_that_never_shows_is_dropped() {
        let mut pending = vec![PendingBuy {
            kind: (1, 2),
            count: 1,
            turn: 10,
        }];
        assert_eq!(take_pending(&(9, 9), 20, &mut pending), 0);
        assert!(pending.is_empty());
    }
}
