//! L1 perception's output: the `Observation`, one immutable snapshot per
//! think (ARCHITECTURE.md L1; task T2.1, #12).
//!
//! It is plain data with no references into the engine, so it serialises
//! (JSON, one observation per line in an observation log) and replays into
//! the decision layers without the game. It holds what the C Borg's two
//! input channels give it (GOALS D3, spec §1.1), and no more:
//!
//! * channel A, the screen: the message line (row 0) and the store header
//!   (row 3, columns 1–4: "Stor" or "Home" in a shop,
//!   `borg-think.c:borg_think:221-223`);
//! * channel B, direct reads: the character's status and timed effects
//!   (`borg_notice_player`), its gear through the C Borg's item analysis
//!   (`borg-item-analyze.c:borg_item_analyze:450-498`), its spells
//!   (`borg-magic.c:borg_cheat_spells:677-718`), the visible panel through
//!   `map_info()` with the cave predicates the C Borg asks about a panel grid
//!   (`borg-update.c:borg_update_map:279-353`), the real race and condition
//!   of each monster seen (`borg-flow-kill.c:borg_update_kill_old:243-330`,
//!   `borg_guess_race:874-889`), and the real kind of the top object on each
//!   grid where the player sees one (`borg-flow-take.c:borg_guess_kind:72-84`).
//!
//! The engine fills it (`engine::perceive`); nothing here touches the engine.

use serde::{Deserialize, Serialize};

/// One think's snapshot of everything the Borg may know.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Observation {
    /// The engine's game turn.
    pub turn: i64,
    pub status: Status,
    /// Every equipment slot of the body, in slot order, empty or not.
    pub equipment: Vec<Slot>,
    pub quiver: Vec<Item>,
    pub inventory: Vec<Item>,
    /// The class's spells in spell-index order (empty for a class without
    /// magic).
    pub spells: Vec<Spell>,
    pub panel: Panel,
    /// Row 0 of the screen as drawn, trailing blanks removed.
    pub message_line: String,
    /// The message line split into messages ([`parse_messages`]).
    pub messages: Vec<String>,
    /// Row 3, columns 1–4: "Stor" or "Home" when the Borg is in a shop.
    pub store_header: String,
    /// The monsters `map_info()` shows on the panel, in grid order.
    pub monsters: Vec<MonsterSeen>,
    /// The grids of the panel where the player sees an object, in grid
    /// order, with the real kind of the top object there.
    pub objects: Vec<ObjectSeen>,
    /// Every store's stock, the home included, read on every think as the
    /// C Borg does (`borg-store.c:borg_notice_store:144-189`).
    pub stores: Vec<StoreStock>,
}

/// One store's wares, in the store's own order.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct StoreStock {
    /// The store's index (`stores[]`, store.txt order; the home is 7).
    pub number: i32,
    pub wares: Vec<Ware>,
}

/// One ware: the item as the item analysis sees it (always aware in a
/// shop, as the player is), and the price the shop asks for one
/// (`price_item(store, obj, false, 1)`; 0 at home).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Ware {
    pub item: Item,
    pub price: i32,
}

/// The character's own state (`player->`, `player->state`).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Status {
    pub race: u32,
    pub class: u32,
    pub x: i32,
    pub y: i32,
    pub depth: i32,
    pub max_depth: i32,
    pub level: i32,
    pub max_level: i32,
    pub exp: i32,
    pub max_exp: i32,
    pub gold: i32,
    pub hp: i32,
    pub max_hp: i32,
    pub sp: i32,
    pub max_sp: i32,
    pub stat_cur: [i16; 5],
    pub stat_max: [i16; 5],
    /// `player->state.stat_use`: the values in use, with equipment.
    pub stat_use: [i32; 5],
    /// `player->state.stat_ind`: the engine's index of each stat into its
    /// tables (the C Borg's `BI_STR_INDEX` ...).
    pub stat_ind: [i32; 5],
    /// `player->state.speed` (110 = normal).
    pub speed: i32,
    /// `player->state.num_blows`: blows per round times 100, as the engine
    /// computed them; the check for `knowledge::blows`.
    pub blows: i32,
    /// The birth option `birth_percent_damage` (O-combat).
    pub percent_damage: bool,
    /// `player->known_state.ac + known_state.to_a`: the armour class the
    /// player is shown.
    pub known_ac: i32,
    /// `player->timed[]`, indexed by `TMD_*` (`list-player-timed.h`).
    pub timed: Vec<i16>,
    pub word_recall: i32,
    pub deep_descent: i32,
    pub food: i32,
    /// Pack slots in use, the quiver's share included: the engine's own
    /// count (`pack_slots_used`, obj-gear.c; REDESIGN-LESSONS L8).
    pub pack_used: i32,
    /// `z_info->pack_size`.
    pub pack_size: i32,
    /// Resting, running or repeating a command (spec §2.7).
    pub busy: bool,
    pub dead: bool,
    pub winner: bool,
    /// `square_isvault()` at the player's grid (danger and retreat).
    pub in_vault: bool,
}

/// One equipment slot: its type (`EQUIP_*`, `list-equip-slots.h`) and what
/// it holds.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Slot {
    pub slot_type: u16,
    pub item: Option<Item>,
}

/// An object as the C Borg's item analysis sees it: identity from the
/// object, numbers from its known part unless it is fully known.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Item {
    /// The real kind (`kind->kidx`), as `tval`/`sval` also give it.
    pub kind: u32,
    pub tval: u8,
    pub sval: u8,
    pub number: u8,
    /// `object_weight_one()`, in tenths of a pound.
    pub weight: i16,
    pub timeout: i16,
    /// `kind->level`.
    pub level: i32,
    /// `object_desc(..., ODESC_FULL)`, inscription included.
    pub desc: String,
    /// `object_flavor_is_aware()`.
    pub aware: bool,
    /// `object_fully_known()`.
    pub fully_known: bool,
    /// `object_runes_known()`.
    pub runes_known: bool,
    /// `object_flags_known()`.
    pub flags: [u8; 5],
    pub ac: i16,
    pub dd: u8,
    pub ds: u8,
    pub to_h: i16,
    pub to_d: i16,
    pub to_a: i16,
    /// `modifiers[OBJ_MOD_*]`.
    pub modifiers: Vec<i16>,
    /// `el_info[ELEM_*]` as (resistance level, flags).
    pub elements: Vec<(i16, u8)>,
}

/// One class spell and what the player knows of it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Spell {
    /// The book's index in the class's books.
    pub book: i32,
    pub level: i32,
    pub mana: i32,
    /// The base failure rate (`class_spell.sfail`); the Borg computes its own
    /// chance from it, as the C Borg does.
    pub base_fail: i32,
    /// `player->spell_flags[]`: `PY_SPELL_LEARNED` 1, `_WORKED` 2,
    /// `_FORGOTTEN` 4.
    pub flags: u8,
}

/// The visible part of the level, through `map_info()`: `width` x `height`
/// grids from (`x0`, `y0`), row by row. It is the terminal's map panel
/// clipped to the level.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Panel {
    pub x0: i32,
    pub y0: i32,
    pub width: i32,
    pub height: i32,
    /// The level's size (`cave->width`, `cave->height`).
    pub level_width: i32,
    pub level_height: i32,
    pub grids: Vec<Grid>,
}

impl Panel {
    /// The grid at level coordinates, if it is on the panel.
    pub fn at(&self, x: i32, y: i32) -> Option<&Grid> {
        let (dx, dy) = (x - self.x0, y - self.y0);
        if dx < 0 || dy < 0 || dx >= self.width || dy >= self.height {
            return None;
        }
        self.grids.get((dy * self.width + dx) as usize)
    }
}

/// What `map_info()` says of one grid, and the cave predicates the C Borg
/// reads for a panel grid.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Grid {
    /// The remembered feature (`FEAT_*`, `list-terrain.h`).
    pub feat: u32,
    pub in_view: bool,
    /// `grid_light_level`: 0 torch, 1 lit, 2 dark, 3 unlit.
    pub lighting: u32,
    /// A monster the player can see is drawn here.
    pub monster: bool,
    /// The kind of the first remembered object (`first_kind`), if any.
    pub object_kind: Option<u32>,
    pub multiple_objects: bool,
    pub unseen_object: bool,
    pub unseen_money: bool,
    /// A trap the player knows of.
    pub trap: bool,
    /// The shop number of a shop entrance (`square_shopnum`), else -1.
    pub shop: i32,
    pub webbed: bool,
}

/// A monster seen this think, with the real race and condition the C Borg
/// reads ("this is a cheat").
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MonsterSeen {
    /// The game's monster index (`midx`).
    pub index: i32,
    pub x: i32,
    pub y: i32,
    /// The real race (`race->ridx`).
    pub race: u32,
    pub hp: i32,
    pub max_hp: i32,
    /// `m_timed[MON_TMD_*]`: sleep, stun, confusion, fear, ...
    pub timed: [i16; 10],
    pub speed: u8,
    /// The display attribute.
    pub attr: u8,
    /// It is mimicking an object.
    pub mimicking: bool,
}

/// An object the player sees on the floor: the real kind of the top object
/// that is not ignored, as `borg_get_top_object` picks it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ObjectSeen {
    pub x: i32,
    pub y: i32,
    /// `None` when every object there is ignored.
    pub kind: Option<u32>,
}

/// Split the message line into messages. The game puts several messages on
/// one line, each one after the other with a space between, and ends a full
/// line with "-more-" (`borg-messages.c:borg_get_messages:183-294` reads
/// the same line). A message ends at '.', '!' or '?' followed by blanks
/// and a capital letter, which starts the next one; so a prompt such as
/// "Really quit? [y/n]" stays whole. Empty pieces are dropped.
pub fn parse_messages(line: &str) -> Vec<String> {
    let line = line.trim_end().trim_end_matches("-more-").trim_end();
    let chars: Vec<char> = line.chars().collect();
    let mut out = Vec::new();
    let mut start = 0;
    for i in 0..chars.len() {
        if !matches!(chars[i], '.' | '!' | '?') {
            continue;
        }
        let mut j = i + 1;
        while j < chars.len() && chars[j] == ' ' {
            j += 1;
        }
        if j > i + 1 && j < chars.len() && chars[j].is_ascii_uppercase() {
            out.push(chars[start..=i].iter().collect::<String>());
            start = j;
        }
    }
    out.push(chars[start..].iter().collect::<String>());
    out.iter()
        .map(|m| m.trim().to_string())
        .filter(|m| !m.is_empty())
        .collect()
}

impl Observation {
    /// One line of an observation log (JSON, no newline).
    pub fn to_line(&self) -> String {
        serde_json::to_string(self).expect("an observation is plain data")
    }

    /// Read one line of an observation log.
    pub fn from_line(line: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(line)
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    pub(crate) fn sample() -> Observation {
        let item = Item {
            kind: 300,
            tval: 9, // TV_SWORD (a dagger)
            sval: 4,
            number: 1,
            weight: 120,
            timeout: 0,
            level: 5,
            desc: "a Dagger (1d4) (+0,+0) {@w1}".into(),
            aware: true,
            fully_known: false,
            runes_known: false,
            flags: [0, 2, 0, 0, 0],
            ac: 0,
            dd: 1,
            ds: 4,
            to_h: 0,
            to_d: 0,
            to_a: 0,
            modifiers: vec![0; 16],
            elements: vec![(0, 0); 25],
        };
        let grid = |feat| Grid {
            feat,
            in_view: true,
            lighting: 1,
            monster: false,
            object_kind: None,
            multiple_objects: false,
            unseen_object: false,
            unseen_money: false,
            trap: false,
            shop: -1,
            webbed: false,
        };
        let equipment_item = item.clone();
        let mut grids = vec![grid(1); 6];
        grids[4].monster = true;
        grids[5].object_kind = Some(300);
        Observation {
            turn: 12345,
            status: Status {
                race: 0,
                class: 0,
                x: 11,
                y: 21,
                depth: 1,
                max_depth: 2,
                level: 3,
                max_level: 3,
                exp: 40,
                max_exp: 40,
                gold: 120,
                hp: 25,
                max_hp: 30,
                sp: 0,
                max_sp: 0,
                stat_cur: [17, 10, 10, 14, 15],
                stat_max: [17, 10, 10, 14, 15],
                stat_use: [17, 10, 10, 14, 15],
                stat_ind: [14, 7, 7, 11, 12],
                speed: 110,
                blows: 105,
                percent_damage: false,
                known_ac: 12,
                timed: vec![0; 53],
                word_recall: 0,
                deep_descent: 0,
                food: 8000,
                pack_used: 1,
                pack_size: 23,
                busy: false,
                dead: false,
                winner: false,
                in_vault: false,
            },
            equipment: vec![
                Slot {
                    slot_type: 1,
                    item: Some(item.clone()),
                },
                Slot {
                    slot_type: 2,
                    item: None,
                },
            ],
            quiver: vec![],
            inventory: vec![item],
            spells: vec![Spell {
                book: 0,
                level: 1,
                mana: 1,
                base_fail: 22,
                flags: 1,
            }],
            panel: Panel {
                x0: 10,
                y0: 20,
                width: 3,
                height: 2,
                level_width: 198,
                level_height: 66,
                grids,
            },
            message_line: "You hit the jackal. The jackal dies.".into(),
            messages: vec!["You hit the jackal.".into(), "The jackal dies.".into()],
            store_header: String::new(),
            monsters: vec![MonsterSeen {
                index: 7,
                x: 11,
                y: 21,
                race: 52,
                hp: 4,
                max_hp: 8,
                timed: [0; 10],
                speed: 120,
                attr: 3,
                mimicking: false,
            }],
            objects: vec![ObjectSeen {
                x: 12,
                y: 21,
                kind: Some(300),
            }],
            stores: vec![StoreStock {
                number: 0,
                wares: vec![Ware {
                    item: equipment_item,
                    price: 25,
                }],
            }],
        }
    }

    #[test]
    fn an_observation_round_trips_through_its_log_line() {
        let o = sample();
        let line = o.to_line();
        assert!(!line.contains('\n'));
        let back = Observation::from_line(&line).unwrap();
        assert_eq!(back, o);
        assert_eq!(back.to_line(), line, "the log line is canonical");
    }

    /// A recorded observation log round-trips line by line (T2.1 exit).
    /// CI records one with `BORG_OBSERVATION_LOG` and points
    /// `BORG_OBSERVATION_LOG_CHECK` at it; without the variable this test
    /// has nothing to read.
    #[test]
    fn a_recorded_log_round_trips() {
        let Some(path) = std::env::var_os("BORG_OBSERVATION_LOG_CHECK") else {
            return;
        };
        let text = std::fs::read_to_string(&path).expect("the observation log");
        let mut n = 0;
        for (i, line) in text.lines().enumerate() {
            let o = Observation::from_line(line).unwrap_or_else(|e| panic!("line {}: {e}", i + 1));
            assert_eq!(o.to_line(), line, "line {} is not canonical", i + 1);
            assert_eq!(
                o.panel.grids.len(),
                (o.panel.width * o.panel.height) as usize
            );
            assert_eq!(o.messages, parse_messages(&o.message_line));
            n += 1;
        }
        assert!(n > 0, "the observation log is empty");
        eprintln!("{n} recorded observations round-trip");
    }

    #[test]
    fn panel_lookup_uses_level_coordinates() {
        let p = sample().panel;
        assert!(p.at(11, 21).unwrap().monster);
        assert_eq!(p.at(12, 21).unwrap().object_kind, Some(300));
        assert!(p.at(9, 20).is_none());
        assert!(p.at(13, 20).is_none(), "past the panel's width");
        assert!(p.at(10, 22).is_none(), "past the panel's height");
    }

    #[test]
    fn the_message_line_splits_into_messages() {
        assert_eq!(
            parse_messages("You hit the jackal.  The jackal dies. -more-"),
            vec!["You hit the jackal.", "The jackal dies."]
        );
        assert_eq!(
            parse_messages("You have no more Flasks of Oil (b)."),
            vec!["You have no more Flasks of Oil (b)."]
        );
        assert_eq!(parse_messages("   "), Vec::<String>::new());
        assert_eq!(
            parse_messages("Really quit? [y/n]"),
            vec!["Really quit? [y/n]"],
            "a prompt is one message"
        );
        assert_eq!(
            parse_messages("It hits you! It misses you."),
            vec!["It hits you!", "It misses you."]
        );
    }
}
