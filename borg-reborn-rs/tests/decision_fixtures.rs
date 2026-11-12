//! T4.1 (#118): death-derived decision fixtures.
//!
//! Every fixture is a constructed [`Observation`] and the decision it must
//! not make, with the owner-verified feasible alternative where one exists.
//! The scenario file says where each fact comes from and what was invented
//! to place it; the shared `knowledge.json` is the engine's own static data,
//! so a fixture never encodes a guessed monster stat.
//!
//! Two kinds of fixture, and neither is ever skipped:
//!
//! * **active**: the current action set can express the alternative. The
//!   decision must avoid the forbidden action and meet the feasible one.
//! * **gap**: the alternative needs an action this milestone has not built
//!   yet (quaff, read, eat, shoot, cast, wield, stair-under-contact). The
//!   fixture pins the decision the Borg makes today and names the M4 issue
//!   that owns the gap. When the behavior changes, the test fails and the
//!   fixture must be promoted to active (or re-pinned with a reason).
//!
//! The set of ids is fixed here, so a fixture cannot disappear silently,
//! and `docs/FIXTURES.md` must name every one.

use borg_reborn_rs::borg::{Memory, Rng};
use borg_reborn_rs::engine::{PlayerView, ESCAPE};
use borg_reborn_rs::executor::Action;
use borg_reborn_rs::knowledge::Knowledge;
use borg_reborn_rs::observation::{
    Grid, Item, MonsterSeen, ObjectSeen, Observation, Panel, Slot, Status,
};
use borg_reborn_rs::params::Params;
use borg_reborn_rs::path::Paths;
use borg_reborn_rs::strategy::{Decision, DecisionCtx, FirstStrategy, Strategy};
use borg_reborn_rs::world::World;
use serde::Deserialize;

/// The fixture ids this milestone ships. A deleted or renamed fixture is a
/// review finding, not a quiet edit.
const EXPECTED_IDS: &[&str] = &[
    "bbc-d0002-gallant-at-three-hp",
    "bbc-d0006-deep-descent-readiness",
    "bbc-d0008-mold-stair-loop",
    "bbc-d0009-full-pack-starvation",
    "bbc-d0010-shoot-instead-of-stairs",
    "bbc-d0011-dagger-for-maul",
    "bbc-d0012-kill-the-mold",
    "bbc-d0019-one-apprentice-not-flight",
    "bbc-d0013-kobold-not-level-flight",
    "bbc-d0013-eye-step-not-phase",
    "bbc-d0014-speed-and-clw",
    "bbc-d0017-faster-pursuer-walk",
    "bbc-d0018-dying-yeek-phase",
    "breeder-pack-one-action",
    "fear-blocks-the-attack",
    "fang-ac36-armour-ignoring-spell",
    "rs-d0005-fang-unused-cure",
    "rs-d0006-mold-at-two-hp",
    "rust-d1330-rest-with-pursuer",
    "rust-d1358-retreat-at-one-hp",
    "rust-d1413-stairs-under-fang",
    "rust-d1420-starvation-no-food",
    "rust-stuck-sealed-room",
    "rust-hungry-eats",
];

/// The open tasks a gap may name (M4 is closed by #124; these are M5's). Update this list when an issue closes:
/// the compiler cannot see GitHub, and a stale owner is exactly the silent
/// skip this check exists to prevent.
const OWNERS: &[(u32, &str)] = &[
    (126, "T5.2 equipment and pack valuation by effect"),
    (127, "T5.3 class item use, spells and ranged combat"),
];

/// Every action name a fixture may use. The first group is what the
/// executor can express; the second is the capabilities M4 adds, named here
/// so a typo cannot silently become a "gap".
const ACTION_NAMES: &[&str] = &[
    "Attack", "Walk", "Rest", "Hold", "Get", "Descend", "Ascend", "Alter", "Buy", "Sell", "Quaff",
    "Read", "Eat", "Shoot", "Cast", "Wield", "Throw", "Use",
];

const KNOWLEDGE: &str = include_str!("fixtures/knowledge.json");
const CATALOGUE: &str = include_str!("../../docs/FIXTURES.md");

#[derive(Deserialize)]
struct Fixture {
    id: String,
    title: String,
    dossier: String,
    citations: Vec<String>,
    construction: String,
    status: String,
    #[serde(default)]
    blocked_by: Option<u32>,
    #[serde(default)]
    turn: i64,
    #[serde(default = "surface")]
    depth: i32,
    #[serde(default)]
    max_depth: Option<i32>,
    player: Player,
    #[serde(default)]
    equipment: Vec<Gear>,
    #[serde(default)]
    inventory: Vec<Pack>,
    map: Vec<String>,
    #[serde(default)]
    monsters: Vec<Monster>,
    #[serde(default)]
    objects: Vec<Floor>,
    forbidden: Vec<Expect>,
    #[serde(default)]
    feasible: Vec<Expect>,
    #[serde(default)]
    current: Option<Expect>,
    /// True when the pinned current action is itself the recorded fatal
    /// decision (the gap has not even changed the action, only the reason).
    #[serde(default)]
    pins_forbidden: bool,
    /// The owner-manual branch, quoted in words: what it actually verified
    /// (turn range, end state), or that no verified branch exists.
    #[serde(default)]
    verified: Option<String>,
    /// What the fixture cannot claim, even when it looks like it does.
    #[serde(default)]
    limits: Option<String>,
}

#[derive(Deserialize)]
struct Player {
    race: u32,
    class: u32,
    level: i32,
    hp: i32,
    max_hp: i32,
    #[serde(default)]
    sp: i32,
    #[serde(default)]
    max_sp: i32,
    #[serde(default = "normal_speed")]
    speed: i32,
    #[serde(default)]
    x: i32,
    #[serde(default)]
    y: i32,
    #[serde(default = "pack_size")]
    pack_size: i32,
    #[serde(default)]
    pack_used: i32,
    #[serde(default = "fed")]
    food: i32,
    /// Sparse `(TMD_* index, value)` pairs; fear is 5, poison 7.
    #[serde(default)]
    timed: Vec<(usize, i16)>,
}

fn fed() -> i32 {
    500
}

fn normal_speed() -> i32 {
    110
}
fn surface() -> i32 {
    1
}
fn pack_size() -> i32 {
    23
}

#[derive(Deserialize)]
struct Gear {
    slot: u16,
    kind: u32,
    #[serde(default)]
    tval: u8,
    #[serde(default)]
    sval: u8,
    #[serde(default = "one")]
    number: u8,
    #[serde(default)]
    weight: i16,
    desc: String,
    #[serde(default)]
    dd: u8,
    #[serde(default)]
    ds: u8,
    #[serde(default)]
    to_h: i16,
    #[serde(default)]
    to_d: i16,
    #[serde(default)]
    to_a: i16,
}

fn one() -> u8 {
    1
}

#[derive(Deserialize)]
struct Pack {
    kind: u32,
    #[serde(default)]
    tval: u8,
    #[serde(default)]
    sval: u8,
    #[serde(default = "one")]
    number: u8,
    desc: String,
    #[serde(default)]
    aware: bool,
}

#[derive(Deserialize)]
struct Monster {
    x: i32,
    y: i32,
    race: u32,
    hp: i32,
    #[serde(default)]
    max_hp: Option<i32>,
    #[serde(default)]
    speed: Option<u8>,
    /// `MON_TMD_SLEEP`; zero (the default) means awake.
    #[serde(default)]
    sleep: i16,
}

#[derive(Deserialize)]
struct Floor {
    x: i32,
    y: i32,
    kind: u32,
}

#[derive(Deserialize)]
struct Expect {
    /// `Attack`, `Walk`, `Rest`, `Hold`, `Get`, `Descend`, `Ascend`,
    /// `Alter`, `Buy`, `Sell`; or a capability the executor does not have
    /// yet: `Quaff`, `Read`, `Eat`, `Shoot`, `Cast`, `Wield`, `Throw`.
    action: String,
    #[serde(default)]
    dir: Option<u8>,
    #[serde(default)]
    mode: Option<String>,
    /// Documentation for an item action (the cure, the scroll, the spell).
    #[serde(default)]
    item: Option<String>,
}

impl Expect {
    fn matches(&self, action: &Action, mode: &str) -> bool {
        if self.mode.as_deref().is_some_and(|m| m != mode) {
            return false;
        }
        match (self.action.as_str(), action) {
            ("Attack", Action::Attack(d)) | ("Walk", Action::Walk(d)) => {
                self.dir.is_none_or(|want| want == *d)
            }
            ("Rest", Action::Rest(_))
            | ("Quaff", Action::Quaff { .. })
            | ("Read", Action::Read { .. })
            | ("Eat", Action::Eat { .. })
            | ("Wield", Action::Wield { .. })
            | ("Hold", Action::Hold)
            | ("Get", Action::Get)
            | ("Descend", Action::Descend)
            | ("Ascend", Action::Ascend)
            | ("Alter", Action::Alter(_))
            | ("Buy", Action::Buy { .. })
            | ("Sell", Action::Sell { .. }) => true,
            _ => false,
        }
    }
}

struct Loaded {
    fixture: Fixture,
    observation: Observation,
}

fn load() -> Vec<Loaded> {
    let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/decisions");
    let mut files: Vec<_> = std::fs::read_dir(dir)
        .expect("the fixture directory")
        .map(|e| e.expect("a fixture entry").path())
        .filter(|p| p.extension().is_some_and(|e| e == "json"))
        .collect();
    files.sort();
    files
        .into_iter()
        .map(|path| {
            let text = std::fs::read_to_string(&path).expect("a fixture file");
            let fixture: Fixture =
                serde_json::from_str(&text).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
            let observation = build(&fixture);
            Loaded {
                fixture,
                observation,
            }
        })
        .collect()
}

fn knowledge() -> Knowledge {
    serde_json::from_str(KNOWLEDGE).expect("the fixture knowledge slice")
}

/// Build the observation the fixture describes. The map supplies terrain;
/// monsters and floor objects are placed by coordinate. Facts the dossier
/// does not give (stat lines, exact room shape) are the fixture's declared
/// construction, never a claim about the original game.
fn build(f: &Fixture) -> Observation {
    let height = f.map.len() as i32;
    let width = f.map.iter().map(|r| r.chars().count()).max().unwrap_or(0) as i32;
    let (mut px, mut py) = (f.player.x, f.player.y);
    let mut grids = Vec::with_capacity((width * height) as usize);
    let mut monsters = Vec::new();
    let mut seen = Vec::new();

    for (y, row) in f.map.iter().enumerate() {
        for (x, ch) in row
            .chars()
            .chain(core::iter::repeat('#'))
            .take(width as usize)
            .enumerate()
        {
            let (x, y) = (x as i32, y as i32);
            if ch == '@' {
                px = x;
                py = y;
            }
            let feat = match ch {
                '#' => 21,
                '<' => borg_reborn_rs::borg::feat::LESS,
                '>' => borg_reborn_rs::borg::feat::MORE,
                '+' => borg_reborn_rs::borg::feat::CLOSED,
                ',' => borg_reborn_rs::borg::feat::PASS_RUBBLE,
                '?' => borg_reborn_rs::borg::feat::NONE,
                _ => borg_reborn_rs::borg::feat::FLOOR,
            };
            let object = f
                .objects
                .iter()
                .find(|o| (o.x, o.y) == (x, y))
                .map(|o| o.kind);
            if let Some(kind) = object {
                seen.push(ObjectSeen {
                    x,
                    y,
                    kind: Some(kind),
                });
            }
            grids.push(Grid {
                feat,
                in_view: ch != '?',
                lighting: 1,
                monster: f.monsters.iter().any(|m| (m.x, m.y) == (x, y)),
                object_kind: object,
                multiple_objects: false,
                unseen_object: false,
                unseen_money: false,
                trap: false,
                shop: -1,
                webbed: false,
            });
        }
    }

    for (index, m) in f.monsters.iter().enumerate() {
        let mut timed = [0i16; 10];
        timed[0] = m.sleep;
        monsters.push(MonsterSeen {
            index: index as i32 + 1,
            x: m.x,
            y: m.y,
            race: m.race,
            hp: m.hp,
            max_hp: m.max_hp.unwrap_or(m.hp),
            timed,
            speed: m.speed.unwrap_or(110),
            attr: 0,
            mimicking: false,
        });
    }

    let mut timed = vec![0i16; 11];
    for &(index, value) in &f.player.timed {
        if timed.len() <= index {
            timed.resize(index + 1, 0);
        }
        timed[index] = value;
    }

    let equipment = f
        .equipment
        .iter()
        .map(|g| Slot {
            slot_type: g.slot,
            item: Some(Item {
                kind: g.kind,
                tval: g.tval,
                sval: g.sval,
                number: g.number,
                weight: g.weight,
                timeout: 0,
                level: 1,
                desc: g.desc.clone(),
                aware: true,
                fully_known: true,
                runes_known: true,
                flags: [0; 5],
                ac: 0,
                dd: g.dd,
                ds: g.ds,
                to_h: g.to_h,
                to_d: g.to_d,
                to_a: g.to_a,
                modifiers: vec![0; 16],
                elements: vec![(0, 0); 25],
            }),
        })
        .collect();

    let inventory = f
        .inventory
        .iter()
        .map(|p| Item {
            kind: p.kind,
            tval: p.tval,
            sval: p.sval,
            number: p.number,
            weight: 10,
            timeout: 0,
            level: 1,
            desc: p.desc.clone(),
            aware: p.aware,
            fully_known: p.aware,
            runes_known: p.aware,
            flags: [0; 5],
            ac: 0,
            dd: 0,
            ds: 0,
            to_h: 0,
            to_d: 0,
            to_a: 0,
            modifiers: vec![0; 16],
            elements: vec![(0, 0); 25],
        })
        .collect();

    // Stats are part of the construction: the dossiers do not record them.
    // 17 in every stat is an ordinary starting line and keeps the blows and
    // the exchange arithmetic inspectable.
    let stat = [17i16; 5];
    let stat_ind = [borg_reborn_rs::knowledge::stat_index(17) as i32; 5];

    Observation {
        turn: f.turn,
        status: Status {
            race: f.player.race,
            class: f.player.class,
            x: px,
            y: py,
            depth: f.depth,
            max_depth: f.max_depth.unwrap_or(f.depth),
            level: f.player.level,
            max_level: f.player.level,
            exp: 0,
            max_exp: 1,
            gold: 0,
            hp: f.player.hp,
            max_hp: f.player.max_hp,
            sp: f.player.sp,
            max_sp: f.player.max_sp,
            stat_cur: stat,
            stat_max: [18; 5],
            stat_use: [17; 5],
            stat_ind,
            speed: f.player.speed,
            blows: 100,
            percent_damage: false,
            known_ac: 0,
            timed,
            word_recall: 0,
            deep_descent: 0,
            food: f.player.food,
            pack_used: if f.player.pack_used > 0 {
                f.player.pack_used
            } else {
                f.inventory.len() as i32
            },
            pack_size: f.player.pack_size,
            busy: false,
            dead: false,
            winner: false,
            in_vault: false,
        },
        equipment,
        quiver: Vec::new(),
        inventory,
        spells: Vec::new(),
        panel: Panel {
            x0: 0,
            y0: 0,
            width,
            height,
            level_width: width,
            level_height: height,
            grids,
        },
        message_line: String::new(),
        messages: Vec::new(),
        store_header: String::new(),
        monsters,
        objects: seen,
        stores: Vec::new(),
    }
}

fn run(_f: &Fixture, o: &Observation) -> (Decision, String, Knowledge) {
    let k = knowledge();
    let mut world = World::new();
    world.update(o);
    let mut paths = Paths::new(&world);
    let mut strategy = FirstStrategy::default();
    let memory = Memory::default();
    let mut rng = Rng::new();
    let params = Params::default();
    let view = PlayerView {
        hp: o.status.hp,
        max_hp: o.status.max_hp,
        x: o.status.x,
        y: o.status.y,
        depth: o.status.depth,
        dead: false,
        busy: false,
        turn: o.turn,
    };
    let mut ctx = DecisionCtx {
        view: &view,
        world: &world,
        knowledge: &k,
        paths: &mut paths,
        mem: &memory,
        rng: &mut rng,
        params: &params,
        failures: &[],
    };
    let d = strategy.decide(&mut ctx);
    let digest = world.digest();
    (d, digest, k)
}

/// A few values against `lib/gamedata/monster.txt` at `e473720`, so a
/// corrupted or hand-edited slice cannot pass unnoticed.
#[test]
fn the_knowledge_slice_matches_the_game_data() {
    let k = knowledge();
    for (index, name, speed, hp, ac) in [
        (17, "grey mold", 110, 2, 1),
        (48, "Fang, Farmer Maggot's Dog", 120, 25, 36),
        (80, "silver mouse", 110, 2, 4),
        (72, "gallant", 110, 27, 24),
    ] {
        let r = k.race(index).expect("a named race");
        assert_eq!(
            (r.name.as_str(), r.speed, r.avg_hp, r.ac),
            (name, speed, hp, ac),
            "race {index}"
        );
    }
    assert_eq!(k.classes.len(), 9);
    assert_eq!(k.player_races.len(), 11);
    assert_eq!(k.extract_energy[110], 10);
    assert_eq!(k.move_energy, 100);
}

#[test]
fn every_fixture_exists_and_is_owned() {
    let fixtures = load();
    let ids: Vec<&str> = fixtures.iter().map(|f| f.fixture.id.as_str()).collect();
    assert_eq!(
        ids.len(),
        EXPECTED_IDS.len(),
        "fixture count changed: {ids:?}"
    );
    for id in EXPECTED_IDS {
        assert!(ids.contains(id), "missing fixture {id}");
    }
    let k = knowledge();
    for f in &fixtures {
        let x = &f.fixture;
        assert!(!x.title.trim().is_empty(), "{}: no title", x.id);
        let width = x.map.first().map_or(0, |r| r.chars().count());
        assert!(width > 0, "{}: empty map", x.id);
        for row in &x.map {
            assert_eq!(row.chars().count(), width, "{}: ragged map row {row}", x.id);
        }
        assert!(
            k.player_race(x.player.race).is_some(),
            "{}: player race {} is not in the knowledge slice",
            x.id,
            x.player.race
        );
        assert!(
            k.class(x.player.class).is_some(),
            "{}: class {} is not in the knowledge slice",
            x.id,
            x.player.class
        );
        for m in &x.monsters {
            let race = k.race(m.race).unwrap_or_else(|| {
                panic!(
                    "{}: monster race {} is not in the knowledge slice",
                    x.id, m.race
                )
            });
            assert!(
                !race.name.is_empty(),
                "{}: race {} has no name",
                x.id,
                m.race
            );
            assert!(
                m.hp <= m.max_hp.unwrap_or(m.hp),
                "{}: monster hp > max",
                x.id
            );
        }
        for e in x
            .forbidden
            .iter()
            .chain(&x.feasible)
            .chain(x.current.iter())
        {
            assert!(
                ACTION_NAMES.contains(&e.action.as_str()),
                "{}: unknown action name {}",
                x.id,
                e.action
            );
        }
        assert!(!x.citations.is_empty(), "{}: no citation", x.id);
        assert!(!x.forbidden.is_empty(), "{}: no forbidden decision", x.id);
        assert!(
            !x.construction.trim().is_empty(),
            "{}: no construction note",
            x.id
        );
        assert!(
            !x.limits.as_deref().unwrap_or("").trim().is_empty(),
            "{}: no limits note; a constructed fixture must say what it cannot claim",
            x.id
        );
        if x.citations
            .iter()
            .any(|c| c.contains("owner-manual-branch"))
        {
            assert!(
                !x.verified.as_deref().unwrap_or("").trim().is_empty(),
                "{}: cites an owner branch but does not quote what it verified",
                x.id
            );
        }
        assert!(
            std::path::Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/.."))
                .join(&x.dossier)
                .exists(),
            "{}: dossier {} does not exist",
            x.id,
            x.dossier
        );
        assert!(
            CATALOGUE.contains(&x.id),
            "{}: not named in docs/FIXTURES.md",
            x.id
        );
        match x.status.as_str() {
            "active" => assert!(
                x.blocked_by.is_none() && x.current.is_none(),
                "{}: an active fixture does not pin a gap",
                x.id
            ),
            "gap" => {
                let owner = x
                    .blocked_by
                    .unwrap_or_else(|| panic!("{}: a gap needs an owner", x.id));
                assert!(
                    OWNERS.iter().any(|(n, _)| *n == owner),
                    "{}: owner #{owner} is not an open task in OWNERS",
                    x.id
                );
                assert!(x.current.is_some(), "{}: a gap pins what happens now", x.id);
                assert!(
                    !x.feasible.is_empty(),
                    "{}: a gap names the missing alternative",
                    x.id
                );
            }
            other => panic!("{}: unknown status {other}", x.id),
        }
    }
}

/// Not a check: `BORG_FIXTURE_PRINT=1 cargo test --test decision_fixtures
/// print_current -- --nocapture` lists what every scenario decides today;
/// adding `BORG_FIXTURE_CANDIDATES=1` lists every scored candidate under
/// it (`score`, then the action).
#[test]
fn print_current() {
    if std::env::var_os("BORG_FIXTURE_PRINT").is_none() {
        return;
    }
    for loaded in load() {
        let (d, _, _) = run(&loaded.fixture, &loaded.observation);
        eprintln!(
            "{:40} {:6} {:8} {:?}",
            loaded.fixture.id, loaded.fixture.status, d.mode, d.action
        );
        if std::env::var_os("BORG_FIXTURE_CANDIDATES").is_some() {
            for (a, s) in &d.candidates {
                eprintln!("{s:8.2} {a:?}");
            }
        }
    }
}

#[test]
fn fixtures_replay_and_meet_their_expectation() {
    for loaded in load() {
        let f = &loaded.fixture;
        let (d1, digest1, _) = run(f, &loaded.observation);
        let (d2, digest2, _) = run(f, &loaded.observation);
        assert_eq!(
            d1.action, d2.action,
            "{}: action is not deterministic",
            f.id
        );
        assert_eq!(d1.mode, d2.mode, "{}: mode is not deterministic", f.id);
        assert_eq!(digest1, digest2, "{}: world is not deterministic", f.id);

        match f.status.as_str() {
            "active" => {
                for bad in &f.forbidden {
                    assert!(
                        !bad.matches(&d1.action, d1.mode),
                        "{}: made the forbidden decision {:?} in {} (want {:?})",
                        f.id,
                        d1.action,
                        d1.mode,
                        f.feasible
                            .iter()
                            .map(|e| (&e.action, e.dir, &e.item))
                            .collect::<Vec<_>>()
                    );
                }
                assert!(
                    f.feasible.iter().any(|e| e.matches(&d1.action, d1.mode)),
                    "{}: chose {:?} in {}; the feasible alternative is {:?}",
                    f.id,
                    d1.action,
                    d1.mode,
                    f.feasible
                        .iter()
                        .map(|e| (&e.action, e.dir, &e.item))
                        .collect::<Vec<_>>()
                );
            }
            "gap" => {
                let current = f.current.as_ref().expect("checked above");
                assert!(
                    current.matches(&d1.action, d1.mode),
                    "{}: pinned as {:?}/{} now {:?}/{}; promote or re-pin this fixture (owner #{:?})",
                    f.id,
                    current.action,
                    current.mode.as_deref().unwrap_or("any"),
                    d1.action,
                    d1.mode,
                    f.blocked_by
                );
                assert!(
                    !f.feasible.iter().any(|e| e.matches(&d1.action, d1.mode)),
                    "{}: the missing alternative now happens; promote this fixture from gap to active",
                    f.id
                );
                let makes_forbidden = f.forbidden.iter().any(|e| e.matches(&d1.action, d1.mode));
                if f.pins_forbidden {
                    assert!(
                        makes_forbidden,
                        "{}: pinned as still making the recorded fatal decision, but now {:?}/{}",
                        f.id, d1.action, d1.mode
                    );
                } else {
                    assert!(
                        !makes_forbidden,
                        "{}: the forbidden decision is back ({:?}/{}); re-pin or fix it",
                        f.id, d1.action, d1.mode
                    );
                }
            }
            _ => unreachable!(),
        }

        // The breeder lesson: one think is one action. A chain cannot be
        // batched by the executor, so prove it here for every fixture.
        let keys = d1.action.keys(&loaded.observation);
        let commands = keys.iter().filter(|&&k| k != ESCAPE).count();
        assert!(
            commands <= 6,
            "{}: {} command keys in one think: {keys:?}",
            f.id,
            commands
        );
    }
}
