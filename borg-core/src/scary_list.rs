//! borg-reborn-rs (#518): the named scary monsters as data.
//!
//! The C Borg marks a level scary while certain monsters are known on it and
//! the character is at or below a level (`borg_near_monster_type`, the
//! `caution_scary_named_monsters` block). This module reads that list from a
//! file instead, selected in `borg.txt` with `borg_scary_list = FILE` (a
//! relative name is resolved under the user directory). Without the key, or
//! with `borg_scary_list = none`, the compiled C list stays in use.
//!
//! ```text
//! # class fragility, 0 (toughest) to 100 (most fragile)
//! fragility: warrior 0, priest 67, mage 100
//!
//! # monster | scary up to clevel: tough - fragile | class exceptions | flags
//! Fang, Farmer Maggot's Dog | 5-8
//! soldier | 8
//! giant white mouse | 15 | | breeders
//! base:ancient dragon | 44
//! ```
//!
//! - **A range per monster, scaled by class fragility** (owner, 2026-11-17):
//!   `5-8` is the level up to which the toughest class (fragility 0) and
//!   the most fragile one (100) fear the monster; a class in between fears
//!   it up to `tough + (fragile - tough) x fragility / 100`, rounded. A
//!   single number is the same level for every class. A class not named on
//!   a `fragility:` line has fragility 0.
//! - **Monsters:** a race name exactly as in `monster.txt`, or `base:` and
//!   the start of a monster base name (as C's "ancient dragon"). A known
//!   monster makes the level scary while the character's level is at most
//!   its class's level.
//! - **Class exceptions:** `class level` pairs giving that class's level for
//!   this monster outright.
//! - **`breeders`:** only while at least as many breeders as the character's
//!   level are known, and not while the Borg stands against breeders (#503).
//! - **Overlays:** a later line for the same monster replaces the earlier
//!   one, `monster | none` removes it, and a later `fragility:` value
//!   replaces an earlier one. An experiment can append its changes to the
//!   shipped list (borg-zoo#228).
//! - An unknown monster or class, a bad number or range, or an unknown flag
//!   stops the run before play: a list is never silently ignored.

use std::sync::OnceLock;

use crate::engine;

/// The class names, in `class.txt` order (`BI_CLASS`).
const CLASSES: [&str; 9] = crate::shell::CLASSES;

#[derive(Clone, Debug, PartialEq, Eq)]
enum Target {
    /// A race index.
    Race(usize),
    /// The start of a monster base name, lower case.
    Base(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Entry {
    target: Target,
    /// Scary up to this level for the toughest class (fragility 0) ...
    tough: i32,
    /// ... and up to this one for the most fragile (fragility 100).
    fragile: i32,
    /// Per-class levels for this monster, replacing the range.
    classes: [Option<i32>; 9],
    /// Only with enough breeders known, and not while standing against them.
    breeders: bool,
    /// #621: an ignore entry, not a scare: the Borg doesn't target or hunt
    /// the monster up to the level (it still attacks one in its way).
    ignore: bool,
    /// #760: an attack-on-sight entry, with its required reason.
    attack: Option<String>,
}

/// A parsed list: the class fragilities and the monsters.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct List {
    fragility: [i32; 9],
    entries: Vec<Entry>,
}

/// The facts a lookup needs about the character and the level.
#[derive(Clone, Copy, Debug)]
pub struct Situation {
    pub clevel: i32,
    pub class: i32,
    pub breeder_count: i32,
    pub breeder_stand: bool,
    /// #622: character levels taken off this monster's listed level (the
    /// caller sets it for town races; 0 = the list as it is).
    pub levels_less: i32,
}

fn class_index(name: &str) -> Result<usize, String> {
    CLASSES
        .iter()
        .position(|k| k.eq_ignore_ascii_case(name))
        .ok_or_else(|| format!("unknown class '{name}'"))
}

/// `class level` pairs, comma-separated.
fn class_levels(list: &str, max: i32, what: &str) -> Result<Vec<(usize, i32)>, String> {
    let mut out = Vec::new();
    for part in list.split(',') {
        let mut words = part.split_whitespace();
        let (Some(name), Some(v), None) = (words.next(), words.next(), words.next()) else {
            return Err(format!("'{}' is not 'class number'", part.trim()));
        };
        out.push((class_index(name)?, number(v, max, what)?));
    }
    Ok(out)
}

fn number(s: &str, max: i32, what: &str) -> Result<i32, String> {
    s.parse::<i32>()
        .ok()
        .filter(|v| (0..=max).contains(v))
        .ok_or_else(|| format!("'{s}' is not a {what} 0-{max}"))
}

/// `N` or `TOUGH-FRAGILE`, character levels with TOUGH <= FRAGILE.
fn range(s: &str) -> Result<(i32, i32), String> {
    let (a, b) = s.split_once('-').map_or((s, s), |(a, b)| (a.trim(), b.trim()));
    let (tough, fragile) = (number(a, 100, "character level")?, number(b, 100, "character level")?);
    if tough > fragile {
        return Err(format!("range '{s}' runs backwards (tough - fragile, tough <= fragile)"));
    }
    Ok((tough, fragile))
}

/// Parse a list. `race` resolves a monster name to its race index.
pub fn parse(text: &str, race: &dyn Fn(&str) -> Option<usize>) -> Result<List, String> {
    let mut list = List::default();
    for (n, raw) in text.lines().enumerate() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let at = |e: String| format!("line {}: {e}", n + 1);
        if let Some(rest) = line.strip_prefix("fragility:") {
            // A later line overrides an earlier one, so an experiment can
            // append its changes to a shipped list (borg-zoo#228).
            for (c, v) in class_levels(rest, 100, "fragility").map_err(&at)? {
                list.fragility[c] = v;
            }
            continue;
        }
        let fields: Vec<&str> = line.split('|').map(str::trim).collect();
        if fields.len() < 2 || fields.len() > 5 {
            return Err(at(format!("expected 'monster | level [| classes [| flags [| reason]]]', got '{line}'")));
        }
        let target = if let Some(base) = fields[0].strip_prefix("base:") {
            let base = base.trim().to_lowercase();
            if base.is_empty() {
                return Err(at("empty monster base".into()));
            }
            Target::Base(base)
        } else {
            Target::Race(race(fields[0]).ok_or_else(|| at(format!("unknown monster '{}'", fields[0])))?)
        };
        // A later line for the same monster and kind (a scare, or #621's
        // ignore) replaces the earlier one, and `none` removes it: an
        // experiment appends its changes (#228). `monster | none | | ignore`
        // removes the ignore entry.
        let ignore = fields.get(3).is_some_and(|f| f.split(',').any(|x| x.trim() == "ignore"));
        let attack = fields.get(3).is_some_and(|f| f.split(',').any(|x| x.trim() == "attack"));
        if ignore && attack {
            return Err(at("an entry cannot be both 'ignore' and 'attack'".into()));
        }
        list.entries.retain(|e| e.target != target || e.ignore != ignore || e.attack.is_some() != attack);
        if fields[1].eq_ignore_ascii_case("none") {
            if fields.get(2).is_some_and(|c| !c.is_empty())
                || fields.get(3).is_some_and(|f| f.split(',').any(|x| !matches!(x.trim(), "ignore" | "attack")))
                || fields.get(4).is_some_and(|r| !r.is_empty())
            {
                return Err(at("'none' takes no class exceptions or reason, and only an 'ignore' or 'attack' kind".into()));
            }
            continue;
        }
        let (tough, fragile) = range(fields[1]).map_err(&at)?;
        let mut classes = [None; 9];
        if let Some(pairs) = fields.get(2).filter(|s| !s.is_empty()) {
            for (c, v) in class_levels(pairs, 100, "character level").map_err(&at)? {
                classes[c] = Some(v);
            }
        }
        let mut breeders = false;
        if let Some(flags) = fields.get(3).filter(|s| !s.is_empty()) {
            for flag in flags.split(',').map(str::trim) {
                match flag {
                    "breeders" => breeders = true,
                    "ignore" => {}
                    "attack" => {}
                    _ => return Err(at(format!("unknown flag '{flag}'"))),
                }
            }
        }
        if ignore && breeders {
            return Err(at("'ignore' takes no 'breeders' flag".into()));
        }
        let reason = fields.get(4).copied().unwrap_or("");
        if attack != !reason.is_empty() {
            return Err(at("an attack entry requires a reason; other entries take no reason".into()));
        }
        list.entries.push(Entry { target, tough, fragile, classes, breeders, ignore, attack: attack.then(|| reason.to_owned()) });
    }
    Ok(list)
}

/// Whether race `r_idx` (with base name `base`) is scary for `who`.
/// #622: the listed level is lowered by `who.levels_less` (the caller sets it
/// for town races, `caution_town_scary_levels_less`).
pub fn is_scary(list: &List, r_idx: usize, base: &[u8], who: Situation) -> bool {
    list.entries.iter().any(|e| {
        !e.ignore && e.attack.is_none()
            && hits(e, r_idx, base)
            && who.clevel <= up_to(list, e, who.class) - who.levels_less
            && (!e.breeders || (who.breeder_count >= who.clevel && !who.breeder_stand))
    })
}

/// #760: the reason to attack, within this entry's class/level range.
/// A breeder entry applies while the existing policy stands against the
/// colony; it never turns a lost colony into an unconditional stand.
pub fn attack_reason<'a>(list: &'a List, r_idx: usize, base: &[u8], who: Situation) -> Option<&'a str> {
    list.entries.iter().find_map(|e| {
        e.attack.as_deref().filter(|_| hits(e, r_idx, base)
            && who.clevel <= up_to(list, e, who.class)
            && (!e.breeders || who.breeder_stand))
    })
}

/// #621: whether race `r_idx` is on the list's ignore entries for `who`: up
/// to its level the Borg doesn't target or hunt it.
/// The base name is asked for only by a base entry.
pub fn is_ignored<'a>(list: &List, r_idx: usize, base: impl Fn() -> &'a [u8], who: Situation) -> bool {
    list.entries.iter().any(|e| {
        e.ignore
            && match &e.target {
                Target::Race(r) => *r == r_idx,
                Target::Base(_) => hits(e, r_idx, base()),
            }
            && who.clevel <= up_to(list, e, who.class)
    })
}

fn hits(e: &Entry, r_idx: usize, base: &[u8]) -> bool {
    match &e.target {
        Target::Race(r) => *r == r_idx,
        Target::Base(prefix) => base.len() >= prefix.len() && base[..prefix.len()].eq_ignore_ascii_case(prefix.as_bytes()),
    }
}

/// The entry's level for the class: its exception, else the scaled range.
fn up_to(list: &List, e: &Entry, class: i32) -> i32 {
    match usize::try_from(class).ok().filter(|&c| c < CLASSES.len()) {
        Some(c) => e.classes[c].unwrap_or_else(|| scaled(e.tough, e.fragile, list.fragility[c])),
        None => e.tough,
    }
}

/// `tough + (fragile - tough) x fragility / 100`, rounded half up.
fn scaled(tough: i32, fragile: i32, fragility: i32) -> i32 {
    tough + ((fragile - tough) * fragility + 50) / 100
}

static LIST: OnceLock<Option<List>> = OnceLock::new();

fn fatal(error: impl std::fmt::Display) -> ! {
    eprintln!("scary_list: {error}");
    std::process::exit(2)
}

/// The list selected by `borg_scary_list`, loaded once (after the game data,
/// which resolves the names); `None` without the key: the compiled C list.
pub fn selected() -> Option<&'static List> {
    LIST.get_or_init(|| {
        let file = crate::shell::borg_txt_setting("borg_scary_list")?;
        let file = file.trim();
        if file.is_empty() {
            fatal("empty borg_scary_list");
        }
        // `none`: the compiled C list (an experiment's control arm).
        if file.eq_ignore_ascii_case("none") {
            return None;
        }
        let path = if std::path::Path::new(file).is_absolute() {
            file.to_string()
        } else {
            String::from_utf8(engine::path_build_user(4096, file.as_bytes())).unwrap_or_else(|e| fatal(e))
        };
        let text = std::fs::read_to_string(&path).unwrap_or_else(|e| fatal(format!("{path}: {e}")));
        // The last race slot is the player ghost, which has no name
        // (borg_race_name's "player ghost").
        let races = engine::r_max().saturating_sub(1);
        let race = |name: &str| (1..races).find(|&r| engine::race_name(r) == name.as_bytes());
        Some(parse(&text, &race).unwrap_or_else(|e| fatal(format!("{path}: {e}"))))
    })
    .as_ref()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn attack_categories_keep_reasons_levels_and_breeder_policy() {
        let resolve = |_: &str| Some(1);
        let list = parse("wild cat | 50 | mage 3 | attack | fast-fragile\nwild cat | 8\n", &resolve).unwrap();
        assert_eq!(attack_reason(&list, 1, b"feline", who(3, 1)), Some("fast-fragile"));
        assert_eq!(attack_reason(&list, 1, b"feline", who(4, 1)), None);
        assert!(is_scary(&list, 1, b"feline", who(1, 0)), "stored independently; runtime switch gives attack precedence");
        let removed = parse("wild cat | 50 | | attack | fast-fragile\nwild cat | none | | attack\n", &resolve).unwrap();
        assert_eq!(attack_reason(&removed, 1, b"feline", who(1, 0)), None);
        let breeder = parse("giant white mouse | 50 | | attack,breeders | breeder\n", &resolve).unwrap();
        assert_eq!(attack_reason(&breeder, 1, b"rodent", who(1, 0)), None);
        let mut stand = who(1, 0);
        stand.breeder_stand = true;
        assert_eq!(attack_reason(&breeder, 1, b"rodent", stand), Some("breeder"));
        assert!(!is_scary(&breeder, 1, b"rodent", stand));
        assert!(parse("wild cat | 50 | | attack\n", &resolve).is_err());
        assert!(parse("wild cat | 50 | | attack,ignore | fast-fragile\n", &resolve).is_err());
        assert!(parse("wild cat | 50 | | | fast-fragile\n", &resolve).is_err());
    }

    #[test]
    fn inactive_attack_range_does_not_remove_an_independent_scare() {
        let list = parse("wild cat | 50 | mage 3 | attack | fast-fragile\nwild cat | 8\n", &|_| Some(1)).unwrap();
        assert!(is_scary(&list, 1, b"feline", who(4, 1)));
        assert_eq!(attack_reason(&list, 1, b"feline", who(4, 1)), None);
        assert_eq!(attack_reason(&list, 1, b"feline", who(4, 7)), Some("fast-fragile"), "mage is a ceiling override, not a class restriction");
        let attack_only = parse("wild cat | 3 | | attack | fast-fragile\n", &|_| Some(1)).unwrap();
        assert_eq!(attack_reason(&attack_only, 1, b"feline", who(4, 7)), None);
        assert!(!is_scary(&attack_only, 1, b"feline", who(4, 7)), "an attack row never invents a separate scare");
    }

    fn race(name: &str) -> Option<usize> {
        ["", "soldier", "squint-eyed rogue", "giant white mouse", "Fang, Farmer Maggot's Dog"]
            .iter()
            .position(|n| !n.is_empty() && *n == name)
    }
    fn who(clevel: i32, class: i32) -> Situation {
        Situation { clevel, class, breeder_count: 0, breeder_stand: false, levels_less: 0 }
    }

    /// #621: an ignore entry is no scare, a scare is no ignore entry, and
    /// each kind replaces or removes only its own kind.
    #[test]
    fn ignore_entries_are_their_own_kind() {
        let race = |n: &str| (n == "Farmer Maggot").then_some(4);
        let who = |clevel| Situation { clevel, class: 0, breeder_count: 0, breeder_stand: false, levels_less: 0 };
        let both = parse("fragility: warrior 0\nFarmer Maggot | 8-11\nFarmer Maggot | 10-20 | | ignore\n", &race).unwrap();
        assert!(is_scary(&both, 4, b"person", who(8)) && is_ignored(&both, 4, || b"person", who(8)));
        assert!(!is_scary(&both, 4, b"person", who(9)) && is_ignored(&both, 4, || b"person", who(10)));
        assert!(!is_ignored(&both, 4, || b"person", who(11)), "warrior: ignored up to 10");
        let only = parse("Farmer Maggot | 10-20 | | ignore\n", &race).unwrap();
        assert!(!is_scary(&only, 4, b"person", who(1)), "an ignore entry scares nobody");
        let removed = parse("Farmer Maggot | 8-11\nFarmer Maggot | 10-20 | | ignore\nFarmer Maggot | none | | ignore\n", &race).unwrap();
        assert!(is_scary(&removed, 4, b"person", who(8)) && !is_ignored(&removed, 4, || b"person", who(8)));
        assert!(parse("Farmer Maggot | none | | breeders\n", &race).is_err());
        assert!(parse("Farmer Maggot | 10-20 | | ignore, breeders\n", &race).is_err(), "a flag that would do nothing");
    }

    /// #622: a monster's listed level, lowered by `levels_less`.
    #[test]
    fn levels_less_lowers_the_listed_level() {
        let list = parse("fragility: warrior 0, mage 100\nbattle-scarred veteran | 8-11\n", &|n| (n == "battle-scarred veteran").then_some(7)).unwrap();
        let at = |clevel, less| is_scary(&list, 7, b"person", Situation { clevel, class: 0, breeder_count: 0, breeder_stand: false, levels_less: less });
        assert!(at(8, 0), "warrior, CL8: scary up to 8");
        assert!(!at(9, 0));
        assert!(at(4, 4), "4 levels less: up to 4");
        assert!(!at(5, 4));
        assert!(!at(1, 20), "never scary");
    }
    const WARRIOR: i32 = 0;
    const MAGE: i32 = 1;
    const PRIEST: i32 = 3;

    #[test]
    fn a_monster_is_scary_up_to_its_level() {
        let l = parse("# comment\n\nsoldier | 8\n", &race).unwrap();
        assert!(is_scary(&l, 1, b"person", who(8, WARRIOR)), "C: clevel <= 8");
        assert!(!is_scary(&l, 1, b"person", who(9, WARRIOR)));
        assert!(!is_scary(&l, 2, b"person", who(1, WARRIOR)), "another monster");
    }

    #[test]
    fn a_range_is_scaled_by_class_fragility() {
        // The owner's example: Fang 5 - 8, a Mage 8, a Priest about 7.
        let l = parse("fragility: mage 100, Priest 67\nFang, Farmer Maggot's Dog | 5-8\nsoldier | 8\n", &race).unwrap();
        assert!(is_scary(&l, 4, b"canine", who(8, MAGE)));
        assert!(!is_scary(&l, 4, b"canine", who(9, MAGE)));
        assert!(is_scary(&l, 4, b"canine", who(7, PRIEST)), "5 + 3 x 0.67 = 7.01, rounded 7");
        assert!(!is_scary(&l, 4, b"canine", who(8, PRIEST)));
        assert!(is_scary(&l, 4, b"canine", who(5, WARRIOR)), "not named: fragility 0, the tough end");
        assert!(!is_scary(&l, 4, b"canine", who(6, WARRIOR)));
        assert!(is_scary(&l, 1, b"person", who(8, MAGE)), "a single number: the same for every class");
        assert!(!is_scary(&l, 1, b"person", who(9, MAGE)));
        assert_eq!(scaled(5, 8, 50), 7, "1.5 rounds up");
        assert_eq!(scaled(5, 8, 33), 6);
        assert_eq!(scaled(5, 8, 0), 5);
    }

    #[test]
    fn a_class_exception_replaces_the_range() {
        let l = parse("fragility: mage 100\nsquint-eyed rogue | 5-9 | mage 6, priest 6\n", &race).unwrap();
        assert!(is_scary(&l, 2, b"person", who(6, MAGE)));
        assert!(!is_scary(&l, 2, b"person", who(7, MAGE)), "6, not 9");
        assert!(is_scary(&l, 2, b"person", who(6, PRIEST)));
        assert!(!is_scary(&l, 2, b"person", who(6, WARRIOR)));
    }

    #[test]
    fn breeders_need_enough_of_them_and_no_stand() {
        let l = parse("giant white mouse | 15 | | breeders\n", &race).unwrap();
        let mut w = who(5, WARRIOR);
        assert!(!is_scary(&l, 3, b"rodent", w), "fewer breeders than the level");
        w.breeder_count = 5;
        assert!(is_scary(&l, 3, b"rodent", w));
        w.breeder_stand = true;
        assert!(!is_scary(&l, 3, b"rodent", w), "standing against breeders (#503)");
    }

    #[test]
    fn a_base_matches_by_prefix() {
        let l = parse("base:Ancient dragon | 44\n", &race).unwrap();
        assert!(is_scary(&l, 9, b"ancient dragon", who(44, WARRIOR)));
        assert!(!is_scary(&l, 9, b"ancient dragon", who(45, WARRIOR)));
        assert!(!is_scary(&l, 9, b"dragon", who(1, WARRIOR)));
    }

    /// The races of the game data, in `monster.txt` order: (name, base).
    fn game_races() -> Vec<(String, String)> {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../engine/angband/lib/gamedata/monster.txt");
        let text = std::fs::read_to_string(&path).unwrap();
        let mut out: Vec<(String, String)> = Vec::new();
        for line in text.lines() {
            if let Some(name) = line.strip_prefix("name:") {
                out.push((name.to_string(), String::new()));
            } else if let Some(base) = line.strip_prefix("base:") {
                out.last_mut().expect("base before a name").1 = base.to_string();
            }
        }
        out
    }

    #[test]
    fn town_attack_rows_have_engine_names_reasons_and_independent_scares() {
        let races = game_races();
        let resolve = |name: &str| races.iter().position(|(n, _)| n == name).map(|i| i + 1);
        let text = std::fs::read_to_string(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../harness/scary_monsters.txt")).unwrap();
        let list = parse(&text, &resolve).unwrap();
        for (name, reason) in [("mean-looking mercenary", "town-melee-danger"), ("battle-scarred veteran", "town-melee-danger"),
            ("filthy street urchin", "town-gold-theft"), ("squint-eyed rogue", "town-item-theft")] {
            let r = resolve(name).unwrap();
            for class in 0..9 {
                assert_eq!(attack_reason(&list, r, b"townsfolk", who(1, class)), Some(reason));
                assert_eq!(attack_reason(&list, r, b"townsfolk", who(50, class)), Some(reason));
                assert_eq!(attack_reason(&list, r, b"townsfolk", who(51, class)), None);
                assert!(is_scary(&list, r, b"townsfolk", who(1, class)), "town losing/unknown fights retain ordinary scare policy");
            }
        }
        let maggot = resolve("Farmer Maggot").unwrap();
        assert_eq!(attack_reason(&list, maggot, b"humanoid", who(1, 0)), None);
        assert!(is_ignored(&list, maggot, || b"humanoid", who(1, 0)));
    }

    /// Review of #527: the C file reproduces the compiled C list (and the
    /// shipped ranges do for the toughest classes) over every race of the
    /// game data, every class, levels 0-60 and the breeder cases, not only
    /// on a sample of games.
    #[test]
    fn the_c_file_matches_the_compiled_list_on_every_race() {
        let races = game_races();
        assert!(races.len() > 500, "monster.txt read");
        // Race indexes as the engine numbers them would do as well; both
        // sides only need the same numbering.
        let index = |i: usize| i + 1;
        let mut st = crate::state::BorgState::default();
        for (i, (name, _)) in races.iter().enumerate() {
            crate::borg_flow_kill::note_monster_id(&mut st, name.as_bytes(), index(i) as u32);
        }
        let resolve = |name: &str| races.iter().position(|(n, _)| n == name).map(index);
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
        let c_file = parse(&std::fs::read_to_string(root.join("harness/scary_monsters_c.txt")).unwrap(), &resolve).unwrap();
        let shipped = parse(&std::fs::read_to_string(root.join("harness/scary_monsters.txt")).unwrap(), &resolve).unwrap();
        let tough: Vec<i32> = (0..9).filter(|&c| shipped.fragility[c as usize] == 0).collect();
        assert_eq!(tough, vec![0, 5, 8], "warrior, paladin, blackguard");
        let mut matched = 0;
        for (i, (_, base)) in races.iter().enumerate() {
            let r = index(i);
            for class in 0..9 {
                for clevel in 0..=60 {
                    for (breeder_count, breeder_stand) in [(0, false), (100, false), (100, true)] {
                        let who = Situation { clevel, class, breeder_count, breeder_stand, levels_less: 0 };
                        let c = crate::borg_flow_kill::c_named_scary(&st.borg.id, r as u32, base.as_bytes(), clevel, class, breeder_count, breeder_stand);
                        assert_eq!(is_scary(&c_file, r, base.as_bytes(), who), c, "C file: race {} {who:?}", races[i].0);
                        if tough.contains(&class) {
                            assert_eq!(is_scary(&shipped, r, base.as_bytes(), who), c, "shipped, tough class: race {} {who:?}", races[i].0);
                        }
                        matched += usize::from(c);
                    }
                }
            }
        }
        assert!(matched > 0, "the compiled list fires somewhere");
        // The "ancient dragon" base: every race of that base, as C.
        let dragons = races.iter().filter(|(_, b)| b.to_lowercase().starts_with("ancient dragon")).count();
        assert!(dragons > 0);
    }

    #[test]
    fn appended_lines_override_and_none_removes() {
        let shipped = "fragility: mage 100\nsoldier | 8-11\nFang, Farmer Maggot's Dog | 5-8\n";
        let arm = format!("{shipped}# an experiment's overlay\nsoldier | 4\nFang, Farmer Maggot's Dog | none\nfragility: mage 0\n");
        let l = parse(&arm, &race).unwrap();
        assert!(is_scary(&l, 1, b"person", who(4, MAGE)), "soldier replaced by 4, fragility 0");
        assert!(!is_scary(&l, 1, b"person", who(5, MAGE)));
        assert!(!is_scary(&l, 4, b"canine", who(1, MAGE)), "Fang removed");
        assert_eq!(l.entries.len(), 1);
        assert!(parse("soldier | none\n", &race).unwrap().entries.is_empty(), "removing an absent monster is fine");
    }

    #[test]
    fn mistakes_stop_the_run() {
        let err = |t: &str| parse(t, &race).unwrap_err();
        assert!(err("Grip | 5\n").contains("line 1: unknown monster 'Grip'"), "full names only");
        assert!(err("soldier | 8 | wizard 3\n").contains("unknown class 'wizard'"));
        assert!(err("soldier | eight\n").contains("not a character level"));
        assert!(err("soldier | -1\n").contains("not a character level"));
        assert!(err("soldier | 8 | | sometimes\n").contains("unknown flag"));
        assert!(err("soldier\n").contains("expected"));
        assert!(err("soldier | 8 | mage\n").contains("is not 'class number'"));
        assert!(err("base: | 4\n").contains("empty monster base"));
        assert!(err("soldier | none | mage 3\n").contains("'none' takes no"));
        assert!(err("fragility: mage 150\n").contains("not a fragility 0-100"));
        assert!(err("soldier | 8-5\n").contains("runs backwards"));
        assert!(err("soldier | 5-\n").contains("not a character level"));
        assert_eq!(parse("", &race).unwrap(), List::default(), "an empty list is no named scares");
    }
}
