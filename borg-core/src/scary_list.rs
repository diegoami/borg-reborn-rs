//! borg-reborn-rs (#518): the named scary monsters as data.
//!
//! The C Borg marks a level scary while certain monsters are known on it and
//! the character is below a level (`borg_near_monster_type`, the
//! `caution_scary_named_monsters` block). This module reads that list from a
//! file instead, selected in `borg.txt` with `borg_scary_list = FILE` (a
//! relative name is resolved under the user directory). Without the key the
//! compiled C list stays in use.
//!
//! One line per monster:
//!
//! ```text
//! # monster | take on from clevel | class exceptions | flags
//! soldier | 9
//! squint-eyed rogue | 6 | mage 7, priest 7
//! giant white mouse | 16 | | breeders
//! base:ancient dragon | 45
//! ```
//!
//! - The monster is a race name exactly as in `monster.txt`, or `base:` and
//!   the start of a monster base name (as C's "ancient dragon").
//! - Below its "take on" level (the class's own level when the class has an
//!   exception) a known monster makes the level scary.
//! - `breeders`: only while at least as many breeders as the character's
//!   level are known, and not while the Borg stands against breeders (#503).
//! - An unknown monster or class, a bad number or an unknown flag stops the
//!   run before play: a list is never silently ignored.

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
pub struct Entry {
    target: Target,
    /// Scary while the character's level is below this.
    from: i32,
    /// Per-class "take on" levels, by `BI_CLASS`.
    classes: [Option<i32>; 9],
    /// Only with enough breeders known, and not while standing against them.
    breeders: bool,
}

/// The facts a lookup needs about the character and the level.
#[derive(Clone, Copy, Debug)]
pub struct Situation {
    pub clevel: i32,
    pub class: i32,
    pub breeder_count: i32,
    pub breeder_stand: bool,
}

/// Parse a list. `race` resolves a monster name to its race index.
pub fn parse(text: &str, race: &dyn Fn(&str) -> Option<usize>) -> Result<Vec<Entry>, String> {
    let mut out = Vec::new();
    for (n, raw) in text.lines().enumerate() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let at = |e: String| format!("line {}: {e}", n + 1);
        let fields: Vec<&str> = line.split('|').map(str::trim).collect();
        if fields.len() < 2 || fields.len() > 4 {
            return Err(at(format!("expected 'monster | level [| classes [| flags]]', got '{line}'")));
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
        let from = level(fields[1]).map_err(&at)?;
        let mut classes = [None; 9];
        if let Some(list) = fields.get(2).filter(|s| !s.is_empty()) {
            for part in list.split(',') {
                let mut words = part.split_whitespace();
                let (Some(name), Some(lv), None) = (words.next(), words.next(), words.next()) else {
                    return Err(at(format!("class exception '{}' is not 'class level'", part.trim())));
                };
                let c = CLASSES
                    .iter()
                    .position(|k| k.eq_ignore_ascii_case(name))
                    .ok_or_else(|| at(format!("unknown class '{name}'")))?;
                classes[c] = Some(level(lv).map_err(&at)?);
            }
        }
        let mut breeders = false;
        if let Some(flags) = fields.get(3).filter(|s| !s.is_empty()) {
            for flag in flags.split(',').map(str::trim) {
                match flag {
                    "breeders" => breeders = true,
                    _ => return Err(at(format!("unknown flag '{flag}'"))),
                }
            }
        }
        out.push(Entry { target, from, classes, breeders });
    }
    Ok(out)
}

fn level(s: &str) -> Result<i32, String> {
    s.parse::<i32>()
        .ok()
        .filter(|v| (0..=100).contains(v))
        .ok_or_else(|| format!("'{s}' is not a character level 0-100"))
}

/// Whether race `r_idx` (with base name `base`) is scary for `who`.
pub fn is_scary(list: &[Entry], r_idx: usize, base: &[u8], who: Situation) -> bool {
    list.iter().any(|e| {
        let hit = match &e.target {
            Target::Race(r) => *r == r_idx,
            Target::Base(prefix) => base.len() >= prefix.len()
                && base[..prefix.len()].eq_ignore_ascii_case(prefix.as_bytes()),
        };
        if !hit {
            return false;
        }
        let from = usize::try_from(who.class)
            .ok()
            .and_then(|c| e.classes.get(c).copied().flatten())
            .unwrap_or(e.from);
        who.clevel < from && (!e.breeders || (who.breeder_count >= who.clevel && !who.breeder_stand))
    })
}

static LIST: OnceLock<Option<Vec<Entry>>> = OnceLock::new();

fn fatal(error: impl std::fmt::Display) -> ! {
    eprintln!("scary_list: {error}");
    std::process::exit(2)
}

/// The list selected by `borg_scary_list`, loaded once (after the game data,
/// which resolves the names); `None` without the key: the compiled C list.
pub fn selected() -> Option<&'static [Entry]> {
    LIST.get_or_init(|| {
        let file = crate::shell::borg_txt_setting("borg_scary_list")?;
        let file = file.trim();
        if file.is_empty() {
            fatal("empty borg_scary_list");
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
    .as_deref()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn race(name: &str) -> Option<usize> {
        ["", "soldier", "squint-eyed rogue", "giant white mouse", "Grip, Farmer Maggot's Dog"]
            .iter()
            .position(|n| !n.is_empty() && *n == name)
    }
    fn who(clevel: i32, class: i32) -> Situation {
        Situation { clevel, class, breeder_count: 0, breeder_stand: false }
    }

    #[test]
    fn a_monster_is_scary_below_its_level() {
        let l = parse("# comment\n\nsoldier | 9\n", &race).unwrap();
        assert!(is_scary(&l, 1, b"person", who(8, 0)), "C: clevel <= 8");
        assert!(!is_scary(&l, 1, b"person", who(9, 0)));
        assert!(!is_scary(&l, 2, b"person", who(1, 0)), "another monster");
    }

    #[test]
    fn a_class_exception_replaces_the_level() {
        let l = parse("squint-eyed rogue | 6 | mage 7, Priest 7\n", &race).unwrap();
        assert!(!is_scary(&l, 2, b"person", who(6, 0)), "warrior from 6");
        assert!(is_scary(&l, 2, b"person", who(6, 1)), "mage from 7");
        assert!(is_scary(&l, 2, b"person", who(6, 3)), "priest, any case");
        assert!(!is_scary(&l, 2, b"person", who(7, 1)));
    }

    #[test]
    fn breeders_need_enough_of_them_and_no_stand() {
        let l = parse("giant white mouse | 16 | | breeders\n", &race).unwrap();
        let mut w = who(5, 0);
        assert!(!is_scary(&l, 3, b"rodent", w), "fewer breeders than the level");
        w.breeder_count = 5;
        assert!(is_scary(&l, 3, b"rodent", w));
        w.breeder_stand = true;
        assert!(!is_scary(&l, 3, b"rodent", w), "standing against breeders (#503)");
    }

    #[test]
    fn a_base_matches_by_prefix() {
        let l = parse("base:Ancient dragon | 45\n", &race).unwrap();
        assert!(is_scary(&l, 9, b"ancient dragon", who(44, 0)));
        assert!(!is_scary(&l, 9, b"ancient dragon", who(45, 0)));
        assert!(!is_scary(&l, 9, b"dragon", who(1, 0)));
    }

    #[test]
    fn mistakes_stop_the_run() {
        let err = |t: &str| parse(t, &race).unwrap_err();
        assert!(err("Grip | 6\n").contains("line 1: unknown monster 'Grip'"), "full names only");
        assert!(err("soldier | 9 | wizard 3\n").contains("unknown class 'wizard'"));
        assert!(err("soldier | nine\n").contains("not a character level"));
        assert!(err("soldier | 9 | | sometimes\n").contains("unknown flag"));
        assert!(err("soldier\n").contains("expected"));
        assert!(err("soldier | 9 | mage\n").contains("is not 'class level'"));
        assert!(err("base: | 4\n").contains("empty monster base"));
        assert_eq!(parse("", &race).unwrap(), vec![], "an empty list is no named scares");
    }
}
