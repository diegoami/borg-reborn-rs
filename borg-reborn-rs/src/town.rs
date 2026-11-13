//! Town sessions (#191): the handover, the state records and the object log.
//!
//! Three switches, read once when the Borg starts, all off by default. With
//! every one of them unset nothing here opens a file, prints a line or
//! changes a key: the run is the run without this module.
//!
//! * `BORG_TOWN_PLAYER=1`, **the handover**: while the character is in town
//!   (depth 0) the Borg yields the input hook (`Key::None`, so the engine
//!   reads the real keyboard, `ui-input.c:inkey_ex:222-235`). The player
//!   shops; the Borg plays the dungeon and takes over again on the first key
//!   the engine asks for outside the town. The Borg makes no decision in
//!   town, so the player's keys and the Borg's never mix. This changes play;
//!   it is for interactive sessions (`harness/play.sh`), never for measured
//!   runs.
//! * `BORG_TOWN_STATE=<file>`, **the state records**: one JSON line per
//!   distinct town state, in the shape borg-rs writes (schema version 3,
//!   `borg-rs/src/borg_town_dump.rs` on `feature/town-session-tools2`,
//!   docs/TOWN-PLAN.md *The record schema*). Written in town only, with or
//!   without the handover.
//! * `BORG_OBJECT_LOG=<file>`, **the object log**: the objects the run used,
//!   bought and acquired, numbered from 1 so the end-of-session feedback can
//!   point at a line (`harness/feedback.sh`); also printed on stdout.
//!
//! The records and the log read the observation the Borg already made, and
//! write after the decision: they never press a key and never change a
//! decision.

use crate::executor::Action;
use crate::observation::{Item, Observation};
use std::collections::BTreeMap;
use std::fs::{File, OpenOptions};
use std::io::Write;

/// The state record's schema version: borg-rs's state record
/// (`STATE_SCHEMA`), the v2 shape without the decision.
pub const STATE_SCHEMA: i64 = 3;

/// The home's store index (`store.txt`: the eighth store).
pub const HOME: i32 = 7;

/// The stores' names, by store index (`terrain.txt`, the shop features in
/// `store.txt` order).
const STORE_NAMES: [&str; 8] = [
    "General Store",
    "Armoury",
    "Weapon Smiths",
    "Bookseller",
    "Alchemy Shop",
    "Magic Shop",
    "Black Market",
    "Home",
];

fn store_name(n: i32) -> &'static str {
    usize::try_from(n)
        .ok()
        .and_then(|i| STORE_NAMES.get(i))
        .copied()
        .unwrap_or("store")
}

/// The town session's switches and outputs.
#[derive(Default)]
pub struct Town {
    /// `BORG_TOWN_PLAYER=1`: the player has the controls in town.
    pub player: bool,
    states: Option<File>,
    /// The last state written, from `"turn"` on: the next one is written
    /// only if it differs.
    last_state: String,
    /// The next record number; taken only when a record is written, so the
    /// file's numbers are contiguous.
    seq: u64,
    objects: Option<ObjectLog>,
}

impl Town {
    /// The switches from the environment.
    pub fn from_env() -> Town {
        let path = |var: &str| std::env::var(var).ok().filter(|p| !p.is_empty());
        Town {
            player: std::env::var("BORG_TOWN_PLAYER").is_ok_and(|v| v == "1"),
            states: path("BORG_TOWN_STATE").and_then(|p| append(&p, "BORG_TOWN_STATE")),
            last_state: String::new(),
            seq: 0,
            objects: path("BORG_OBJECT_LOG").and_then(|p| ObjectLog::open(&p)),
        }
    }

    /// Whether the player has the controls now: the handover is on and the
    /// character is in town.
    pub fn hands_over(&self, depth: i32) -> bool {
        self.player && depth == 0
    }

    /// Whether any output wants the observation.
    pub fn records(&self) -> bool {
        self.states.is_some() || self.objects.is_some()
    }

    /// Record the town state, if it is new. In the dungeon nothing is
    /// written: the records are about the town.
    pub fn note_state(&mut self, o: &Observation) {
        let Some(file) = self.states.as_mut() else {
            return;
        };
        if o.status.depth != 0 {
            return;
        }
        let record = state_record(o, self.seq);
        let key = state_key(&record);
        if key == self.last_state {
            return;
        }
        self.last_state = key.to_string();
        self.seq += 1;
        // A failed write loses the record, never the game.
        if writeln!(file, "{record}")
            .and_then(|_| file.flush())
            .is_err()
        {
            self.states = None;
        }
    }

    /// Log what appeared in the pack since the last look.
    pub fn note_pack(&mut self, o: &Observation) {
        if let Some(log) = self.objects.as_mut() {
            log.note_pack(o);
        }
    }

    /// Log the object the Borg's chosen action uses or buys. `ammo` is the
    /// ammunition tval the launcher takes, for a shot.
    pub fn note_action(&mut self, action: Action, o: &Observation, ammo: Option<u8>) {
        if let Some(log) = self.objects.as_mut() {
            log.note_action(action, o, ammo);
        }
    }
}

/// Open a file for appending; a file that cannot be opened is reported and
/// left off.
fn append(path: &str, var: &str) -> Option<File> {
    match OpenOptions::new().create(true).append(true).open(path) {
        Ok(f) => Some(f),
        Err(e) => {
            eprintln!("borg-reborn-rs: {var} {path}: {e}");
            None
        }
    }
}

/// What makes two state records the same state: everything from the clock
/// on (borg-rs `state_key`): the version and the record number are
/// bookkeeping.
pub fn state_key(record: &str) -> &str {
    record.find(",\"turn\":").map_or(record, |at| &record[at..])
}

/// The awareness bits (borg-rs `awareness_flags`): bit 0 the character
/// knows the kind (`object_flavor_is_aware`), bit 1 the item still has
/// unknown runes (borg-rs `needs_ident = !runes_known()`).
fn awareness(item: &Item) -> i64 {
    i64::from(item.aware) | (i64::from(!item.runes_known) << 1)
}

/// One item as integers: `[slot, tval, sval, quantity, value, flags]`.
/// borg-rs's value is the C Borg's own estimate; our Borg has none, so it
/// is -1 (absent) except for a ware, which carries the store's price.
fn push_item(out: &mut String, item: &Item, slot: usize, value: i64) {
    out.push_str(&format!(
        "[{slot},{},{},{},{value},{}]",
        item.tval,
        item.sval,
        item.number,
        awareness(item)
    ));
}

fn push_list<'a>(out: &mut String, items: impl Iterator<Item = (usize, &'a Item, i64)>) {
    out.push('[');
    for (i, (slot, item, value)) in items.enumerate() {
        if i > 0 {
            out.push(',');
        }
        push_item(out, item, slot, value);
    }
    out.push(']');
}

/// The state record (schema 3): borg-rs's field order, every value an
/// integer, -1 for an absent one. `traits` are the C Borg's trait indexes,
/// which our Borg does not have: the list is empty. The decision fields are
/// -1 (a state record carries none). Equipment slots are numbered after the
/// pack, as borg-rs's item array numbers them. Our addition is under `x`
/// (docs/TOWN-PLAN.md): the quiver, which borg-rs does not write.
pub fn state_record(o: &Observation, seq: u64) -> String {
    let s = &o.status;
    let mut out = String::with_capacity(2048);
    out.push_str(&format!(
        "{{\"v\":{STATE_SCHEMA},\"seq\":{seq},\"turn\":{},\"depth\":{},\"level\":{},\
         \"chp\":{},\"mhp\":{},\"gold\":{},\"race\":{},\"class\":{},\"traits\":[],\
         \"action\":-1,\"shop\":-1,\"ware\":-1,\"item\":-1,\"price\":-1",
        o.turn, s.depth, s.level, s.hp, s.max_hp, s.gold, s.race, s.class
    ));
    out.push_str(",\"pack\":");
    push_list(
        &mut out,
        o.inventory.iter().enumerate().map(|(i, it)| (i, it, -1)),
    );
    let pack_size = usize::try_from(s.pack_size).unwrap_or(0);
    out.push_str(",\"equip\":");
    push_list(
        &mut out,
        o.equipment
            .iter()
            .enumerate()
            .filter_map(|(i, sl)| sl.item.as_ref().map(|it| (pack_size + i, it, -1))),
    );
    let home = o.stores.iter().find(|st| st.number == HOME);
    out.push_str(",\"home\":");
    push_list(
        &mut out,
        home.into_iter()
            .flat_map(|st| st.wares.iter().enumerate())
            .map(|(i, w)| (i, &w.item, -1)),
    );
    out.push_str(",\"shops\":[");
    for (n, st) in o.stores.iter().enumerate() {
        if n > 0 {
            out.push(',');
        }
        out.push_str(&format!("[{},", st.number));
        push_list(
            &mut out,
            st.wares
                .iter()
                .enumerate()
                .map(|(i, w)| (i, &w.item, i64::from(w.price))),
        );
        out.push(']');
    }
    out.push_str("],\"x\":{\"quiver\":");
    push_list(
        &mut out,
        o.quiver.iter().enumerate().map(|(i, it)| (i, it, -1)),
    );
    out.push_str("}}");
    out
}

/// Two items are the same kind when their tval and sval agree (borg-rs
/// `Kind`): not the charges, which a wand's description carries.
type Kind = (u8, u8);

/// The object log (`BORG_OBJECT_LOG`): borg-rs `borg_object_log.rs`.
struct ObjectLog {
    file: File,
    /// Lines written so far, the file's earlier lines included.
    count: u64,
    /// The pack at the last look, by kind; `None` before the first look,
    /// which only takes the baseline (the starting kit is not acquired).
    previous: Option<BTreeMap<Kind, u32>>,
    /// The Borg's purchases since the last look: the pack will show them,
    /// and they are `buy` lines, not acquisitions.
    bought: Vec<Kind>,
}

impl ObjectLog {
    fn open(path: &str) -> Option<ObjectLog> {
        // A file used again does not repeat its numbers.
        let count = std::fs::read_to_string(path)
            .map(|t| t.lines().filter(|l| !l.trim().is_empty()).count() as u64)
            .unwrap_or(0);
        Some(ObjectLog {
            file: append(path, "BORG_OBJECT_LOG")?,
            count,
            previous: None,
            bought: Vec::new(),
        })
    }

    fn emit(&mut self, o: &Observation, what: &str, details: &str) {
        self.count += 1;
        let line = log_line(self.count, o.turn, o.status.depth, what, details);
        let mut out = std::io::stdout();
        let _ = out.write_all(line.as_bytes()).and_then(|_| out.flush());
        let _ = self
            .file
            .write_all(line.as_bytes())
            .and_then(|_| self.file.flush());
    }

    fn note_pack(&mut self, o: &Observation) {
        let now = counts(&o.inventory);
        let bought = std::mem::take(&mut self.bought);
        let Some(old) = self.previous.replace(now.clone()) else {
            return;
        };
        for (kind, mut grew) in grown(&old, &now) {
            // The Borg's own purchases are the `buy` lines already.
            let mine = bought.iter().filter(|k| **k == kind).count() as u32;
            grew = grew.saturating_sub(mine);
            if grew == 0 {
                continue;
            }
            let Some(item) = o.inventory.iter().find(|i| (i.tval, i.sval) == kind) else {
                continue;
            };
            let details = if grew == 1 {
                item.desc.clone()
            } else {
                format!("{grew} x {}", item.desc)
            };
            self.emit(o, "acquire", &details);
        }
    }

    fn note_action(&mut self, action: Action, o: &Observation, ammo: Option<u8>) {
        let pack = |i: usize| o.inventory.get(i);
        let (what, item) = match action {
            Action::Quaff { item } => ("quaff", pack(item)),
            Action::Read { item } => ("read", pack(item)),
            Action::Eat { item } => ("eat", pack(item)),
            Action::Throw { item } => ("throw", pack(item)),
            Action::Wield { item } => ("wield", pack(item)),
            Action::Shoot => ("fire", o.quiver.iter().find(|i| Some(i.tval) == ammo)),
            Action::Sell { item, .. } => {
                if let Some(it) = pack(item) {
                    let shop = shop_here(o);
                    let d = format!("{} to the {}", it.desc, store_name(shop));
                    self.emit(o, if shop == HOME { "store" } else { "sell" }, &d);
                }
                return;
            }
            Action::Buy { ware } => {
                let shop = shop_here(o);
                let Some(w) = o
                    .stores
                    .iter()
                    .find(|s| s.number == shop)
                    .and_then(|s| s.wares.get(ware))
                else {
                    return;
                };
                // The purchase takes the default quantity, one
                // (executor: `Enter` at the quantity prompt).
                let one = one_of(&w.item);
                if shop == HOME {
                    self.emit(o, "take", &format!("{one} from the Home"));
                } else {
                    let d = format!("{one} at the {}, cost {} gold", store_name(shop), w.price);
                    self.emit(o, "buy", &d);
                }
                self.bought.push((w.item.tval, w.item.sval));
                return;
            }
            _ => return,
        };
        if let Some(it) = item {
            self.emit(o, what, &one_of(it));
        }
    }
}

/// The store whose entrance the character stands on, or -1.
fn shop_here(o: &Observation) -> i32 {
    o.panel.at(o.status.x, o.status.y).map_or(-1, |g| g.shop)
}

/// The item's description as one of it: a stack's description names its
/// count, and an action uses or buys one.
fn one_of(item: &Item) -> String {
    if item.number > 1 {
        format!("one of {}", item.desc)
    } else {
        item.desc.clone()
    }
}

/// One object log line (borg-rs's format).
fn log_line(n: u64, turn: i64, depth: i32, what: &str, details: &str) -> String {
    format!("{n:04} turn {turn} depth {depth} {what} {details}\n")
}

/// Every kind's count in a pack.
fn counts(items: &[Item]) -> BTreeMap<Kind, u32> {
    let mut out = BTreeMap::new();
    for i in items {
        *out.entry((i.tval, i.sval)).or_insert(0) += u32::from(i.number);
    }
    out
}

/// What grew between two packs, in kind order.
fn grown(old: &BTreeMap<Kind, u32>, new: &BTreeMap<Kind, u32>) -> Vec<(Kind, u32)> {
    new.iter()
        .filter_map(|(k, &n)| {
            let before = old.get(k).copied().unwrap_or(0);
            (n > before).then_some((*k, n - before))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::observation::{Slot, StoreStock, Ware};

    fn item(tval: u8, sval: u8, number: u8, desc: &str) -> Item {
        Item {
            kind: 0,
            tval,
            sval,
            number,
            weight: 0,
            timeout: 0,
            level: 0,
            desc: desc.into(),
            aware: true,
            fully_known: true,
            runes_known: true,
            flags: [0; 5],
            ac: 0,
            dd: 0,
            ds: 0,
            to_h: 0,
            to_d: 0,
            to_a: 0,
            modifiers: vec![],
            elements: vec![],
        }
    }

    fn town() -> Observation {
        let mut o = Observation {
            turn: 1000,
            ..Observation::default()
        };
        o.status.gold = 200;
        o.status.hp = 10;
        o.status.max_hp = 12;
        o.status.level = 1;
        o.status.race = 2;
        o.status.class = 3;
        o.status.pack_size = 23;
        o.inventory = vec![item(80, 1, 5, "5 Rations of Food")];
        o.equipment = vec![
            Slot {
                slot_type: 1,
                item: Some(item(23, 4, 1, "a Dagger")),
            },
            Slot {
                slot_type: 2,
                item: None,
            },
        ];
        let mut unknown = item(75, 9, 1, "a Purple Potion");
        unknown.aware = false;
        unknown.runes_known = false;
        o.stores = vec![
            StoreStock {
                number: 0,
                wares: vec![Ware {
                    item: item(39, 1, 3, "3 Wooden Torches"),
                    price: 2,
                }],
            },
            StoreStock {
                number: HOME,
                wares: vec![Ware {
                    item: unknown,
                    price: 0,
                }],
            },
        ];
        o
    }

    #[test]
    fn the_state_record_has_borg_rs_shape() {
        let r = state_record(&town(), 4);
        assert_eq!(
            r,
            "{\"v\":3,\"seq\":4,\"turn\":1000,\"depth\":0,\"level\":1,\"chp\":10,\"mhp\":12,\
             \"gold\":200,\"race\":2,\"class\":3,\"traits\":[],\"action\":-1,\"shop\":-1,\
             \"ware\":-1,\"item\":-1,\"price\":-1,\"pack\":[[0,80,1,5,-1,1]],\
             \"equip\":[[23,23,4,1,-1,1]],\"home\":[[0,75,9,1,-1,2]],\
             \"shops\":[[0,[[0,39,1,3,2,1]]],[7,[[0,75,9,1,0,2]]]],\"x\":{\"quiver\":[]}}"
        );
        let v: serde_json::Value = serde_json::from_str(&r).expect("valid JSON");
        assert_eq!(v["v"], 3);
    }

    #[test]
    fn the_same_state_is_written_once() {
        let a = state_record(&town(), 0);
        let b = state_record(&town(), 1);
        assert_ne!(a, b);
        assert_eq!(state_key(&a), state_key(&b), "seq is not the state");
        let mut later = town();
        later.status.gold = 150;
        assert_ne!(state_key(&a), state_key(&state_record(&later, 1)));
    }

    #[test]
    fn everything_is_off_by_default() {
        let t = Town::default();
        assert!(!t.hands_over(0));
        assert!(!t.records());
        let on = Town {
            player: true,
            ..Town::default()
        };
        assert!(on.hands_over(0));
        assert!(!on.hands_over(1), "the dungeon is the Borg's");
    }

    #[test]
    fn only_what_grew_is_acquired() {
        let old = counts(&[item(1, 2, 1, "a")]);
        let new = counts(&[item(1, 2, 2, "a"), item(4, 5, 1, "b")]);
        assert_eq!(grown(&old, &new), vec![((1, 2), 1), ((4, 5), 1)]);
        let wand = |d| counts(&[item(65, 3, 1, d)]);
        assert!(grown(&wand("(5 charges)"), &wand("(4 charges)")).is_empty());
    }

    #[test]
    fn the_line_format_is_borg_rs() {
        assert_eq!(
            log_line(7, 142, 23, "quaff", "a Potion of Cure Light Wounds"),
            "0007 turn 142 depth 23 quaff a Potion of Cure Light Wounds\n"
        );
    }

    #[test]
    fn a_purchase_is_a_buy_not_an_acquisition() {
        let dir = std::env::temp_dir().join(format!("town-log-{}", std::process::id()));
        let _ = std::fs::remove_file(&dir);
        let mut log = ObjectLog::open(dir.to_str().unwrap()).unwrap();
        let mut o = town();
        log.note_pack(&o); // the baseline: nothing logged
        log.note_action(Action::Buy { ware: 0 }, &o, None); // not on a shop: nothing
        assert_eq!(log.count, 0);
        // On the General Store's entrance.
        o.panel.width = 1;
        o.panel.height = 1;
        o.panel.grids = vec![crate::observation::Grid {
            feat: 7,
            in_view: true,
            lighting: 1,
            monster: false,
            object_kind: None,
            multiple_objects: false,
            unseen_object: false,
            unseen_money: false,
            trap: false,
            shop: 0,
            webbed: false,
        }];
        log.note_action(Action::Buy { ware: 0 }, &o, None);
        o.inventory.push(item(39, 1, 1, "a Wooden Torch"));
        o.inventory.push(item(70, 4, 1, "a Scroll of Phase Door"));
        log.note_pack(&o);
        let text = std::fs::read_to_string(&dir).unwrap();
        let _ = std::fs::remove_file(&dir);
        let lines: Vec<&str> = text.lines().collect();
        assert_eq!(lines.len(), 2, "{text}");
        assert!(lines[0].ends_with("buy one of 3 Wooden Torches at the General Store, cost 2 gold"));
        assert!(lines[1].ends_with("acquire a Scroll of Phase Door"));
    }
}
