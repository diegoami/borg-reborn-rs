//! borg-reborn-rs's errands over this core (#208, docs/ERRANDS.md). Not part of borg-rs: this file and the hook calls marked
//! `borg-reborn-rs (#208)` are the shell's whole footprint in the vendored
//! core, so borg-rs's later changes still apply to the rest as plain
//! patches.
//!
//! The errand is **trips** (owner, 2026-11-14): explore DL1 and go back to
//! town; then explore DL1, DL2, DL1 and go back to town; and so on. The
//! player writes the plan; a class or a power level calls for its own
//! (a mage repeats town-DL1-town longer, a warrior goes deeper sooner).
//!
//! - **On a level of the plan the core plays as itself.** The level is
//!   explored when the core is bored with it and free to go either way:
//!   `borg_leave_level` has weighed its own reasons (readiness for this
//!   depth, restocking, selling, fixed level or experience) and none holds.
//!   Then the trip goes to its next level, or home. Otherwise the core's
//!   choice stands, and the level watch reads where it arrives.
//! - **Between levels the errand walks the stairs**, after the core's
//!   caution, with the core's own stair flows. The next trip starts by the
//!   down staircase (never a recall to the maximum depth).
//! - **The core's own readiness stands.** A planned level the core is not
//!   prepared for (`borg_prepared`) is skipped and noted, never forced. A
//!   core that goes back to town to restock does so its own way; the level
//!   stays in the plan, and the errand takes it back.
//! - **A trip marked `*` repeats** until the core is prepared for the
//!   deepest level of the next trip (`borg_prepared`); the last one repeats
//!   until the budget. The player writes the ladder, the core's readiness
//!   climbs it: a mage stays on town-DL1-town longer, a warrior moves on
//!   sooner.
//! - The errand ends in town when the last trip is over, or at its budget,
//!   through the core's own stop (`borg_oops`); its outcome goes to
//!   `BORG_ERRAND_OUT` (one JSON line) when that is set.
//!
//! **Commands** (owner, 2026-11-14), typed at `^Z /` or given as
//! `BORG_ERRAND` / `borg_reborn_errand`; each hands control back (the
//! core's `borg_oops`) when it is done:
//!
//! - `/town`: home fast: the core reads Word of Recall if it can
//!   (`borg_recall`), else the stairs up; done in town.
//! - `/shopping`: in town, buy, sell and manage the home with the core's
//!   own choices; done when the core would leave town after shopping.
//! - `/walk-back`: home by the stairs only, finding each up staircase; done
//!   in town.
//! - `/explore-back`: the level it is on and each level above it explored,
//!   in turn, then home; done in town.
//! - `/dive N`: the stairs down to DL N, not exploring; done on arrival.
//! - `/explore-level`: the core explores the level it is on; done when it
//!   is done with it.
//! - `/dive-exploring N`: each level from here down to DL N explored, in
//!   turn; done when DL N is.
//! - `/ladder`, `/trips ...`: as above; they never end (a `*` trip repeats).
//! - `/stopscared off|flee|N`: a setting, not an errand: when the Borg
//!   stops and hands control back. `N` (1-1000): when the danger where it
//!   stands (`borg_danger`, one turn) reaches N% of its current hit
//!   points. `flee` (or `true`) is 100: one turn could kill it.
//! - `/explored N|off`: a setting: a level of the plan counts as explored
//!   once N% of its floor is seen (default 90), or, off, only when the
//!   core is done with it.
//! - `/stall N|off`: a setting: a level of the plan is given up after N
//!   game turns with nothing new seen (default 10,000).
//! - `/stop`: clear the errand.
//!
//! **Fear overrides the plan.** A core that wants town ends the trip; one
//! that flees a level leaves it its own way; a level of the plan it leaves
//! by its own move (stairs, Deep Descent, Teleport Level, a trap door)
//! counts as explored, and the plan goes on from where it arrives.
//!
//! A line can hold several, separated by `;` (`/stopscared 50; /dive 2;
//! /walk-back`): settings apply at once, the errands run in turn, each
//! starting when the one before it is done. Any other ending (scared,
//! aborted, the budget) drops the rest.
//!
//! The player's commands are the player's: `/dive` and `/dive-exploring`
//! do not skip a level the core is not prepared for (the ladders do), and
//! a command the core's own fear makes impossible (it wants town) stops
//! and hands control back.
//!
//! With no errand set and `/stopscared` off, nothing here changes a
//! decision: every hook returns at once, and play is borg-rs's byte for
//! byte.

use crate::borg_headers::borg_trait_h::{BI_CDEPTH, BI_CLEVEL, BI_CURHP, BI_MAXHP};
use crate::borg_io::{borg_keypress, borg_note};
use crate::engine;
use crate::headers::list_terrain_h::{
    FEAT_BROKEN, FEAT_CLOSED, FEAT_FLOOR, FEAT_LESS, FEAT_MORE, FEAT_NONE, FEAT_OPEN,
    FEAT_PASS_RUBBLE,
};
use crate::headers::ui_event_h::keycode_t;
use crate::state::BorgState;
use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::Mutex;

/// An errand.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Errand {
    /// Trips, in order. Each trip is the levels to explore, in order, and
    /// ends in town.
    Trips(Vec<Trip>),
    /// The class's ladder (`borg-core/ladders.txt`), resolved when it is set.
    Ladder,
    /// Home; done in town.
    Town,
    /// One shopping round in town; stop before leaving town.
    Shopping,
    /// The stairs down to DL N, not exploring; done on arrival.
    Dive(i32),
    /// The level the character is on, explored; done when the core is done.
    ExploreLevel,
    /// Each level from here down to DL N explored; done when DL N is.
    DiveExploring(i32),
    /// Home by the stairs only; done in town.
    WalkBack,
    /// Each level from here up to DL1 explored, then home; done in town.
    ExploreBack,
}

/// A line typed at `^Z /` (or given in `BORG_ERRAND`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Command {
    Errand(Errand),
    /// `None`: off; `Some(N)`: danger at N% of the current hit points.
    StopScared(Option<i32>),
    /// `/explored N`: a level of the plan is explored at N% of its floor
    /// seen (`None`: only when the core is done with it).
    Explored(Option<i32>),
    /// `/stall N`: a level of the plan is given up after N game turns with
    /// nothing new seen (`None`: never).
    Stall(Option<i32>),
    Stop,
}

impl Command {
    /// `/town`, `/dive 2`, `/explore-level`, `/dive-exploring 2`, `/ladder`,
    /// `/trips 1*/1,2,1*@3`, `/stopscared true`, `/stop`. The slash is
    /// optional; `trips:...` and `dive:2` are accepted too.
    pub fn parse(line: &str) -> Option<Command> {
        let line = line.trim();
        let line = line.strip_prefix('/').unwrap_or(line);
        let (name, arg) = match line.find(|c: char| c == ' ' || c == ':') {
            Some(i) => (&line[..i], line[i + 1..].trim()),
            None => (line, ""),
        };
        let depth = || arg.parse::<i32>().ok().filter(|d| *d >= 1);
        Some(match (name.to_ascii_lowercase().as_str(), arg) {
            ("town", "") => Command::Errand(Errand::Town),
            ("shopping", "") => Command::Errand(Errand::Shopping),
            ("dive", _) => Command::Errand(Errand::Dive(depth()?)),
            ("explore-level", "") => Command::Errand(Errand::ExploreLevel),
            ("dive-exploring", _) => Command::Errand(Errand::DiveExploring(depth()?)),
            ("ladder", "") => Command::Errand(Errand::Ladder),
            ("walk-back", "") => Command::Errand(Errand::WalkBack),
            ("explore-back", "") => Command::Errand(Errand::ExploreBack),
            ("trips", _) => Command::Errand(Errand::parse(&format!("trips:{arg}"))?),
            ("stopscared", v) => Command::StopScared(match v.to_ascii_lowercase().as_str() {
                "true" | "on" | "yes" | "flee" => Some(100),
                "false" | "off" | "no" => None,
                n => Some(n.parse::<i32>().ok().filter(|n| (1..=1000).contains(n))?),
            }),
            ("explored", v) => Command::Explored(match v.to_ascii_lowercase().as_str() {
                "off" | "false" | "no" => None,
                n => Some(n.parse::<i32>().ok().filter(|n| (1..=100).contains(n))?),
            }),
            ("stall", v) => Command::Stall(match v.to_ascii_lowercase().as_str() {
                "off" | "false" | "no" => None,
                n => Some(n.parse::<i32>().ok().filter(|n| *n >= 1)?),
            }),
            ("stop", "") | ("none", "") => Command::Stop,
            _ => return None,
        })
    }
}

/// One trip: the levels to explore, in order, then home.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Trip {
    pub levels: Vec<i32>,
    /// Repeat until the ladder may move on to the next trip (the last
    /// trip: until the budget).
    pub repeat: bool,
    /// `@N`: the ladder reaches this trip only from character level N.
    /// Without it, only once the core is prepared for its deepest level.
    pub min_level: Option<i32>,
}

/// The class names of `class.txt`, in its order (`player->class->cidx`).
pub(crate) const CLASSES: [&str; 9] = [
    "warrior",
    "mage",
    "druid",
    "priest",
    "necromancer",
    "paladin",
    "rogue",
    "ranger",
    "blackguard",
];

/// A setting of the run's `borg.txt` (`ANGBAND_DIR_USER/borg.txt`).
/// All shell and plan callers share the strict missing/read-error policy.
pub fn borg_txt_setting(name: &str) -> Option<String> {
    let path = borg_txt_path(engine::path_build_user(1024, b"borg.txt"))
        .unwrap_or_else(|error| config_file_error(error));
    borg_txt_setting_at(&path, name)
}

fn config_file_error(error: String) -> ! {
    eprintln!("borg-reborn-rs: configuration: {error}");
    std::process::exit(2)
}

fn borg_txt_path(bytes: Vec<u8>) -> Result<std::path::PathBuf, String> {
    #[cfg(unix)]
    {
        use std::os::unix::ffi::OsStringExt;
        Ok(std::ffi::OsString::from_vec(bytes).into())
    }
    #[cfg(not(unix))]
    {
        String::from_utf8(bytes).map(std::path::PathBuf::from)
            .map_err(|error| format!("invalid borg.txt path: {error}"))
    }
}

fn read_borg_txt(path: &std::path::Path) -> Result<String, String> {
    match std::fs::read_to_string(path) {
        Ok(text) => Ok(text),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(String::new()),
        Err(error) => Err(format!("{}: cannot read borg.txt: {error}", path.display())),
    }
}

/// The parameter loader shares the same missing-file and read-error policy.
pub(crate) fn borg_txt_contents() -> Result<String, String> {
    let path = borg_txt_path(engine::path_build_user(1024, b"borg.txt"))?;
    read_borg_txt(&path)
}

fn borg_txt_setting_at(path: &std::path::Path, name: &str) -> Option<String> {
    borg_txt_value_at(path, name).unwrap_or_else(|error| config_file_error(error))
}

pub(crate) fn borg_txt_value_at(path: &std::path::Path, name: &str) -> Result<Option<String>, String> {
    let text = read_borg_txt(path)?;
    Ok(text.lines()
        .map(str::trim)
        .filter(|l| !l.starts_with('#'))
        .filter_map(|l| l.split_once('='))
        .find(|(n, _)| n.trim() == name)
        .map(|(_, v)| v.trim().to_string()))
}

/// The default ladders by class (`borg-core/ladders.txt`), or the file
/// `BORG_LADDERS` names.
fn ladders_text() -> String {
    std::env::var_os("BORG_LADDERS")
        .and_then(|p| std::fs::read_to_string(p).ok())
        .unwrap_or_else(|| include_str!("../ladders.txt").to_string())
}

/// `BORG_ERRAND=ladder`: the ladder of the character's class.
pub fn class_ladder() -> Option<Errand> {
    let class = *CLASSES.get(engine::class_cidx() as usize)?;
    ladder_of(&ladders_text(), class)
}

fn ladder_of(text: &str, class: &str) -> Option<Errand> {
    text.lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .filter_map(|l| l.split_once('='))
        .find(|(name, _)| name.trim() == class)
        .and_then(|(_, plan)| Errand::parse(plan.trim()))
}

impl Errand {
    /// `trips:1*/1,2,1*@5/1,2,3,2,1*@10` (the `BORG_ERRAND` syntax): trips
    /// separated by `/`, levels by `,`, `*` after a trip to repeat it, `@N`
    /// for the character level the ladder needs to reach it.
    pub fn parse(s: &str) -> Option<Errand> {
        let (name, arg) = s.split_once(':')?;
        if name != "trips" {
            return None;
        }
        let trips: Option<Vec<Trip>> = arg
            .split('/')
            .map(|trip| {
                let (trip, min_level) = match trip.trim().split_once('@') {
                    Some((t, l)) => (t, Some(l.trim().parse::<i32>().ok()?)),
                    None => (trip.trim(), None),
                };
                let (body, repeat) = match trip.strip_suffix('*') {
                    Some(b) => (b, true),
                    None => (trip, false),
                };
                let levels: Option<Vec<i32>> = body
                    .split(',')
                    .map(|l| l.trim().parse().ok().filter(|d: &i32| *d >= 1))
                    .collect();
                levels.map(|levels| Trip {
                    levels,
                    repeat,
                    min_level,
                })
            })
            .collect();
        trips
            .filter(|t| !t.is_empty() && t.iter().all(|trip| !trip.levels.is_empty()))
            .map(Errand::Trips)
    }
}

/// Where a trips errand stands.
#[derive(Clone, Debug, PartialEq, Eq)]
struct Running {
    /// A repeating interval, independent of the finite/trip planner.
    trips: Vec<Trip>,
    /// The trip under way.
    trip: usize,
    /// The level of that trip to explore next; past the end, home.
    step: usize,
    /// Game turns the errand may run; 0 for no budget.
    budget: i64,
    /// The game turn of the first think under the errand.
    started: Option<i64>,
    /// Levels skipped because the core was not prepared for them.
    skipped: u32,
    /// What ends the errand.
    end: End,
    /// Skip a planned level the core is not prepared for (the ladders), or
    /// not (the player's commands).
    skip_unprepared: bool,
    /// The plan starts where the character stands, resolved at the first
    /// think: `ExploreLevel`, `DiveExploring(N)`, `ExploreBack`.
    from_here: Option<FromHere>,
    /// Read Word of Recall to go home, when the core can (`/town`).
    recall: bool,
    /// The command, for the report.
    name: String,
    /// The level the character is on, as the errand watches it.
    watch: Option<Watch>,
    /// The character the errand was given to (class, birth key) and the
    /// last game turn seen: another character drops the errand (review
    /// #227, finding 5 and re-review 4).
    owner: Option<(u32, u64, i64)>,
}

/// The level under the errand: its depth, the grids the core knows of it,
/// and the last game turn that number grew.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Watch {
    depth: i32,
    known: u32,
    progress: i64,
    /// The game turn the character arrived on this level.
    arrived: i64,
    /// The errand has given up steering on this level (`stuck`).
    stuck: bool,
}

/// A plan that starts at the character's depth.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum FromHere {
    /// This level.
    Here,
    /// This level and each one below, down to DL N.
    DownTo(i32),
    /// This level and each one above, up to DL1.
    UpToOne,
}

/// What ends an errand.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum End {
    /// Each trip goes home; the errand ends when every trip is over.
    Home,
    /// Done on arriving at the plan's level (`/dive`).
    Arrive,
    /// Done when the plan's last level is explored, where it is.
    Stop,
    /// Let the core finish its shopping, then stop before leaving town.
    Shopping,
}

impl Running {
    /// The depth the errand wants now: a level of the trip, 0 for home, or
    /// `None` when the errand is over.
    fn target(&self) -> Option<i32> {
        if self.end == End::Shopping {
            return Some(0);
        }
        let trip = self.trips.get(self.trip)?;
        match trip.levels.get(self.step) {
            Some(&d) => Some(d),
            None if self.end == End::Home => Some(0),
            None => None,
        }
    }
}

/// `/stopscared`: -1 off, N danger at N% of the current hit points.
static STOP_SCARED: AtomicI32 = AtomicI32::new(-1);

/// `/explored`: the share of a level's floor seen at which a level of the
/// plan counts as explored; 0, only when the core is done with it (owner,
/// 2026-11-14: "happy at 90% or something").
static EXPLORED_PCT: AtomicI32 = AtomicI32::new(90);

/// `/stall`: game turns with nothing new seen after which a level of the
/// plan is given up; 0, never. 10,000 is 1,000 player turns at normal
/// speed. A paladin on DL3 rested on a staircase for 60,000 turns, never
/// done with the level (docs/PILOT.md).
static STALL_TURNS: AtomicI32 = AtomicI32::new(10_000);

/// Set `/stopscared`.
pub fn set_stop_scared(level: Option<i32>) {
    STOP_SCARED.store(level.unwrap_or(-1), Ordering::Relaxed);
}

static ERRAND: Mutex<Option<Running>> = Mutex::new(None);

fn running() -> std::sync::MutexGuard<'static, Option<Running>> {
    ERRAND.lock().unwrap_or_else(|e| e.into_inner())
}

/// borg-reborn-rs (#222): read-only trace context. A nested think must not
/// block on an outer errand operation; its trace frame can inherit context.
pub(crate) fn trace_errand() -> Result<Option<(String, Option<i32>)>, ()> {
    let guard = ERRAND.try_lock().map_err(|_| ())?;
    Ok(guard.as_ref().map(|r| (r.name.clone(), r.target())))
}

/// Set (or clear) the errand, with a budget in game turns (0: none).
pub fn set_errand(errand: Option<Errand>, budget: i64) {
    let errand = match errand {
        Some(Errand::Ladder) => class_ladder(),
        e => e,
    };
    *running() = errand.map(|e| {
        let name = format!("{e:?}");
        let one = |levels| {
            vec![Trip {
                levels,
                repeat: false,
                min_level: None,
            }]
        };
        let (trips, end, skip, from_here) = match e {
            Errand::Trips(t) => (t, End::Home, true, None),
            Errand::Ladder => (Vec::new(), End::Home, true, None),
            Errand::Town | Errand::WalkBack => (one(vec![]), End::Home, false, None),
            Errand::Shopping => (Vec::new(), End::Shopping, false, None),
            Errand::Dive(n) => (one(vec![n]), End::Arrive, false, None),
            Errand::ExploreLevel => (Vec::new(), End::Stop, false, Some(FromHere::Here)),
            Errand::DiveExploring(n) => (Vec::new(), End::Stop, false, Some(FromHere::DownTo(n))),
            Errand::ExploreBack => (Vec::new(), End::Home, false, Some(FromHere::UpToOne)),
        };
        let recall = name == "Town";
        Running {
            trips,
            trip: 0,
            step: 0,
            budget,
            started: None,
            skipped: 0,
            end,
            skip_unprepared: skip,
            from_here,
            recall,
            name,
            watch: None,
            owner: None,
        }
    });
}

/// Apply a command: set the errand or the setting. `true` when an errand
/// was set (the caller starts the Borg).
pub fn apply(command: Command, budget: i64) -> bool {
    match command {
        Command::Errand(e) => {
            crate::errands_plan::cancel();
            set_errand(Some(e), budget);
            active()
        }
        Command::StopScared(level) => {
            set_stop_scared(level);
            false
        }
        Command::Explored(pct) => {
            EXPLORED_PCT.store(pct.unwrap_or(0), Ordering::Relaxed);
            false
        }
        Command::Stall(turns) => {
            STALL_TURNS.store(turns.unwrap_or(0), Ordering::Relaxed);
            false
        }
        Command::Stop => {
            crate::errands_plan::cancel();
            NEXT.lock().unwrap().clear();
            set_errand(None, 0);
            false
        }
    }
}

/// The errands queued after the running one (`/dive 2; walk-back`): each
/// starts when the one before it is done; any other ending drops them.
static NEXT: Mutex<std::collections::VecDeque<Errand>> =
    Mutex::new(std::collections::VecDeque::new());

/// The commands of a line, separated by `;`; `Err` names the first one it
/// cannot read.
pub fn parse_line(line: &str) -> Result<Vec<Command>, String> {
    line.split(';')
        .map(str::trim)
        .filter(|c| !c.is_empty())
        .map(|c| Command::parse(c).ok_or_else(|| c.to_string()))
        .collect()
}

/// Apply a command line: commands separated by `;`, settings at once, the
/// first errand now and the rest after it. `Ok(true)` when an errand runs;
/// `Err` names a command it cannot read, and then nothing is applied.
pub fn apply_line(line: &str, budget: i64) -> Result<bool, String> {
    let commands = parse_line(line)?;
    let mut errands = std::collections::VecDeque::new();
    for c in commands {
        match c {
            Command::Errand(e) => errands.push_back(e),
            other => {
                apply(other, budget);
            }
        }
    }
    let Some(first) = errands.pop_front() else {
        return Ok(active());
    };
    *NEXT.lock().unwrap() = errands;
    Ok(apply(Command::Errand(first), budget))
}

/// `^Z /`: read a command line and apply it; an errand starts the Borg.
pub(crate) fn borg_cmd_errand(st: &mut BorgState) {
    let mut buf = [0u8; 80];
    if !engine::get_string(
        st,
        c"Errand (/town /shopping /walk-back /explore-back /dive N /dive-exploring N /explore-level /ladder /stop; ...): ",
        &mut buf,
        80,
    ) {
        return;
    }
    let line =
        String::from_utf8_lossy(&buf[..buf.iter().position(|&c| c == 0).unwrap_or(0)]).into_owned();
    match apply_line(&line, 0) {
        Ok(true) => crate::borg_commands::borg_cmd_start(st),
        Ok(false) => {}
        Err(c) => borg_note(st, &format!("# Errand: unknown command '{c}'.")),
    }
}

/// `/stopscared`: the danger where it stands reaches `level`% of its hit
/// points. The core's flee flags are not fear here: it sets `fleeing` for
/// routine moves, and `fleeing_to_town` only in town, where it stays set
/// on the level below (a first version that counted them stopped every
/// character at turn 1).
fn scared(st: &mut BorgState, level: i32) -> bool {
    let hp = st.borg.tr(BI_CURHP).max(1);
    let (x, y) = (st.borg.c.x, st.borg.c.y);
    let danger = crate::borg_danger::borg_danger(st, y, x, 1, true, false);
    let hit = i64::from(danger) * 100 >= i64::from(hp) * i64::from(level);
    if hit {
        borg_note(
            st,
            &format!("# Errand: scared, danger {danger} against {hp} hit points."),
        );
    }
    hit
}

/// End the errand with this outcome: the next queued errand starts if this
/// one is done, otherwise control goes back to the player.
fn finish(st: &mut BorgState, outcome: &str) {
    crate::errands_plan::cancel();
    let turn = i64::from(engine::game_turn());
    report(st, outcome, turn);
    let budget = running().as_ref().map_or(0, |r| r.budget);
    let next = {
        let mut q = NEXT.lock().unwrap();
        if outcome == "done" {
            q.pop_front()
        } else {
            q.clear();
            None
        }
    };
    match next {
        Some(e) => {
            borg_note(st, &format!("# Errand: done, next {e:?}."));
            set_errand(Some(e), budget);
            // The think must queue a key; a harmless one.
            borg_keypress(st, crate::borg_io::ESCAPE);
        }
        None => {
            set_errand(None, 0);
            crate::borg_think::borg_oops(st, &format!("errand {outcome}"));
        }
    }
}

/// Whether an errand is running.
pub fn active() -> bool {
    running().is_some()
}

/// The target depth now, skipping the planned levels the core is not
/// prepared for.
fn target(st: &mut BorgState) -> Option<i32> {
    loop {
        let (t, skip) = {
            let guard = running();
            let r = guard.as_ref()?;
            (r.target()?, r.skip_unprepared)
        };
        if t == 0 || !skip || crate::borg_prepared::borg_prepared(st, t).is_none() {
            return Some(t);
        }
        borg_note(st, &format!("# Errand: not prepared for DL{t}, skipped."));
        if let Some(r) = running().as_mut() {
            r.step += 1;
            r.skipped += 1;
        }
    }
}

/// Before the dungeon think: in town, a trip that came home is over, and the
/// next begins; the errand ends when the last is over, or at its budget.
/// `Some(true)` means the think is over (the Borg stops).
pub(crate) fn before_dungeon_think(st: &mut BorgState) -> Option<bool> {
    if crate::errands_plan::tick(st) {
        return Some(true);
    }
    let turn = i64::from(engine::game_turn());
    // The engine's depth, read before the core's update of the think.
    let depth = engine::player_depth();
    // Another character in this process (reincarnation, another save): the
    // errand and its queue were the old one's (review #227, finding 5). It
    // goes first, so a stale errand is not reported scared (re-review 9).
    let stale = running().as_mut().is_some_and(|r| {
        let me = (engine::class_cidx(), engine::player_birth_key());
        let stale = r
            .owner
            .is_some_and(|(c, k, last)| (c, k) != me || turn < last);
        r.owner = Some((me.0, me.1, turn));
        stale
    });
    if stale {
        borg_note(st, "# Errand: another character; the errand is dropped.");
        NEXT.lock().unwrap().clear();
        set_errand(None, 0);
    }
    // `/stopscared`: with or without an errand, a scared core hands over.
    let level = STOP_SCARED.load(Ordering::Relaxed);
    if level > 0 && depth > 0 && scared(st, level) {
        finish(st, "scared");
        return Some(true);
    }
    // Shopping is a town session, not a trip that is done on arrival in
    // town. Never let the generic trip indexing advance its empty plan.
    let shopping = {
        let mut guard = running();
        let r = guard.as_mut()?;
        if r.end == End::Shopping {
            let first = r.started.is_none();
            let started = *r.started.get_or_insert(turn);
            let elapsed = turn - started;
            let stall = i64::from(STALL_TURNS.load(Ordering::Relaxed));
            let outcome = if depth != 0 {
                Some("aborted: shopping requires town")
            } else if r.budget > 0 && elapsed >= r.budget {
                Some("budget")
            } else if stall > 0 && elapsed >= stall.max(10_000) {
                Some("aborted: shopping did not finish")
            } else {
                None
            };
            Some((first, outcome))
        } else {
            None
        }
    };
    if let Some((first, outcome)) = shopping {
        if let Some(outcome) = outcome {
            finish(st, outcome);
            return Some(true);
        }
        if first {
            // A manual player may already have spent a long time in town.
            // Give this requested round the core's normal shopping window
            // instead of inheriting its old town/anti-bounce timeout.
            st.borg.time.level = st.borg.time.now;
            st.borg.antibounce_count = 0;
            st.borg.goal.type_0 = 0;
            st.borg.goal.shop = -1;
            st.borg.goal.ware = -1;
            st.borg.goal.item = -1;
            borg_note(st, "# Errand: shopping round in town.");
        }
        return None;
    }
    // A plan that starts here, and the arrival of a dive.
    {
        let mut guard = running();
        let r = guard.as_mut()?;
        if let Some(plan) = r.from_here.take() {
            let here = depth.max(1);
            let levels = match plan {
                FromHere::Here => vec![here],
                FromHere::DownTo(n) if n > here => (here..=n).collect(),
                FromHere::DownTo(_) => vec![here],
                FromHere::UpToOne if depth == 0 => vec![],
                FromHere::UpToOne => (1..=depth).rev().collect(),
            };
            r.trips = vec![Trip {
                levels,
                repeat: false,
                min_level: None,
            }];
        }
        let arrived = r.end == End::Arrive && r.target() == Some(depth);
        // A new level. If the core left a level of the plan before the
        // errand was done with it (it fled, read Deep Descent or Teleport
        // Level, fell through a trap door), its fear overrides the plan:
        // that level counts as explored, and the plan goes on from here.
        // A core that rises by its own choice while the plan wants deeper
        // is not fit for the plan's depth (its "keep on a suitable level"
        // rule takes the stairs up below character level 10 when it is not
        // prepared): a trip goes home; a player's command stops. Without
        // this the errand walked down, the core walked up, and connected
        // stairs looped for 50,000 turns (docs/PILOT.md). A rise to town is
        // not one: the core goes home to shop or restock (at turn 1 it
        // flees the town's urchins to DL1 and comes back to finish
        // shopping), and the errand goes on from there.
        let mut left = None;
        let mut unfit = None;
        // A core going home (`rising`) is on its way to town: neither a
        // level left early nor a rise counts (re-review #227, 3).
        let homeward = st.borg.goal.rising;
        if r.watch.is_none_or(|w| w.depth != depth) {
            if let Some(w) = r.watch.filter(|_| !homeward) {
                // To town, the level stays in the plan: the core went to
                // shop or restock, and the errand takes it back.
                if w.depth > 0 && depth > 0 && r.target() == Some(w.depth) {
                    r.step += 1;
                    left = Some(w.depth);
                }
                // A rise that ends the plan (off its last level) did not
                // explore that level either: `/dive-exploring 2` was
                // reported done when the core rose off DL2.
                let next = r.target().or(left);
                if let Some(t) = next {
                    if depth > 0 && depth < w.depth && (t > depth || r.target().is_none()) {
                        unfit = Some((w.depth, t));
                        if r.end == End::Home && r.skip_unprepared {
                            r.step = usize::MAX;
                        }
                    }
                }
            }
            r.watch = Some(Watch {
                depth,
                known: 0,
                progress: turn,
                arrived: turn,
                stuck: false,
            });
        }
        let command = !r.skip_unprepared;
        drop(guard);
        if let (Some(d), None) = (left, unfit) {
            borg_note(
                st,
                &format!("# Errand: the core left DL{d} its own way; counted explored."),
            );
        }
        if let Some((from, t)) = unfit {
            if command {
                finish(
                    st,
                    &format!("aborted: the core rose from DL{from}, not fit for DL{t}"),
                );
                return Some(true);
            }
            borg_note(
                st,
                &format!(
                    "# Errand: the core rose from DL{from}, not fit for DL{t}; the trip goes home."
                ),
            );
        }
        if arrived {
            finish(st, "done");
            return Some(true);
        }
    }
    // Home after a trip: what comes next (the readiness check runs outside
    // the lock: `borg_prepared` reads the core's state only).
    let next_deepest = {
        let mut guard = running();
        let r = guard.as_mut()?;
        r.started.get_or_insert(turn);
        if depth == 0 && r.target() == Some(0) {
            r.step = 0;
            if r.trips[r.trip].repeat {
                r.trips.get(r.trip + 1).map(|next| {
                    (
                        next.levels.iter().copied().max().unwrap_or(1),
                        next.min_level,
                    )
                })
            } else {
                r.trip += 1;
                None
            }
        } else {
            None
        }
    };
    if let Some((deepest, min_level)) = next_deepest {
        // A repeating trip moves on at the next trip's character level, or,
        // with none set, once the core is ready for its deepest level.
        let ready = match min_level {
            Some(level) => st.borg.tr(BI_CLEVEL) >= level,
            None => crate::borg_prepared::borg_prepared(st, deepest).is_none(),
        };
        if ready {
            if let Some(r) = running().as_mut() {
                r.trip += 1;
            }
        }
    }
    let outcome = {
        let guard = running();
        let r = guard.as_ref()?;
        let started = r.started.unwrap_or(turn);
        if r.target().is_none() {
            "done"
        } else if r.budget > 0 && turn - started >= r.budget {
            "budget"
        } else {
            return None;
        }
    };
    // Through `finish`, so a `;` queue goes on (review #227, finding 4).
    finish(st, outcome);
    Some(true)
}

/// Town (`borg_leave_level` at depth 0): the core shops until it is bored,
/// then the trip starts by the stairs (never a recall to the maximum
/// depth). `None` lets the core decide as usual.
pub(crate) fn on_leave_level(st: &mut BorgState, bored: bool) -> Option<bool> {
    if !active() || st.borg.tr(BI_CDEPTH) != 0 {
        return None;
    }
    if running().as_ref().is_some_and(|r| r.end == End::Shopping) {
        if bored {
            // The core has explored town, checked its shops and home, and
            // has no further transaction to do. Hand over instead of diving.
            finish(st, "done");
        }
        return Some(bored);
    }
    if !bored || target(st).is_none_or(|t| t == 0) {
        return Some(false);
    }
    if stuck(st, 0) {
        // The core leaves town its own way (`leave_town`).
        return None;
    }
    if !prepared_below(st, 0) {
        return Some(refuse_descent(st, 1));
    }
    Some(stairs_down(st))
}

/// Caution may take the stairs directly, before the ordinary town-leave
/// hook. A town-only shopping job hands over rather than escaping below.
pub(crate) fn shopping_departure(st: &mut BorgState) -> bool {
    let shopping = running().as_ref().is_some_and(|r| r.end == End::Shopping);
    if !shopping || st.borg.tr(BI_CDEPTH) != 0 {
        return false;
    }
    finish(st, "aborted: the core needs to leave town");
    true
}

/// In the dungeon, `borg_leave_level` once the core has weighed its own
/// reasons (readiness for this depth, restocking, selling, fixed level or
/// experience, the time on the level). `free`: the core is bored with the
/// level (done with it), and none of those reasons holds (not rising, not
/// restocking, prepared for this depth). Only then is the level explored
/// and the way on the errand's. Otherwise `None`: the core's own choice
/// stands (it keeps exploring, climbs, or goes home), and the level watch
/// reads where it arrives. Review #227, findings 1-3: the hook used to run
/// first, so routine `borg_leave_level(st, false)` calls ended levels, and
/// the core's readiness and restock reasons were skipped.
pub(crate) fn on_leave_choice(st: &mut BorgState, free: bool) -> Option<bool> {
    if !active() || !free {
        return None;
    }
    Some(move_on(st))
}

/// The level under the errand is explored (the core is done with it, or
/// `/explored`, or `/stall`): the plan moves on, and the errand walks
/// toward its next level. Both ways a level ends come here (re-review
/// #227, 1: `/explored` and `/stall` used to call the town-only hook).
fn move_on(st: &mut BorgState) -> bool {
    let depth = st.borg.tr(BI_CDEPTH);
    match scared_home(st) {
        Home::Fear if running().as_ref().is_some_and(|r| r.end != End::Home) => {
            // The player's command cannot go on: hand control back.
            finish(st, "aborted: the core wants town");
            return true;
        }
        Home::Fear | Home::Errand => return false,
        Home::No => {}
    }
    let mut advanced = false;
    if let Some(r) = running().as_mut() {
        if r.target() == Some(depth) {
            r.step += 1;
            advanced = true;
        }
    }
    let Some(t) = target(st) else {
        // The plan's last level is explored (`/explore-level`,
        // `/dive-exploring`): done, where it is.
        finish(st, "done");
        return true;
    };
    if advanced {
        borg_note(st, &format!("# Errand: DL{depth} explored, next DL{t}."));
    }
    if stuck(st, depth) {
        return false;
    }
    if walk(st, depth, t) {
        return true;
    }
    // With no staircase known toward the target, another one: a fresh
    // level is the way on, as the core itself does when it is bored and
    // cannot dive (DL1 up is the town, and the trip goes on from there).
    // Without this, warriors with DL2 as the target sat on a used-up DL1
    // with no down staircase known. Down only when the core is prepared
    // for the level below (re-review #227, 2).
    stairs_up(st) || (prepared_below(st, depth) && stairs_down(st))
}

/// `GOAL_FLEE` (borg-flow.h), as the core's think modules declare it.
const GOAL_FLEE: i32 = 7;

/// After the core's caution: away from the target depth, walk the stairs
/// toward it. `true` when a step was queued.
pub(crate) fn steer(st: &mut BorgState) -> bool {
    if !active() {
        return false;
    }
    let depth = st.borg.tr(BI_CDEPTH);
    if running().as_ref().is_some_and(|r| r.end == End::Shopping) {
        // Remain after caution but before the general exploration/leave
        // decisions. In particular, "nothing to sell" must not bypass
        // purchases and home management by power-diving out of town.
        if st.borg.goal.fleeing || st.borg.mon.scary {
            return shopping_departure(st);
        }
        if crate::borg_think_store::borg_choose_shop(st) {
            let shop = i32::from(st.borg.goal.shop);
            if !crate::borg_flow_misc::borg_flow_shop_entry(st, shop) {
                finish(st, "aborted: no way to the selected shop");
            }
        } else {
            finish(st, "done");
        }
        return true;
    }
    // `/town`: the core's own recall (scroll, rod, spell, activation) when
    // it has one, whatever else the core wants (review #227, finding 6,
    // and re-review 5); until it takes effect the stairs up are walked.
    let recall = running()
        .as_ref()
        .is_some_and(|r| r.recall && r.target() == Some(0));
    if recall && depth > 0 && st.borg.goal.recalling == 0 && crate::borg_escape::borg_recall(st) {
        return true;
    }
    // A scared core is not held to the plan (owner, 2026-11-14): it wants
    // town, so the trip ends and goes home, or a command stops. A core
    // going home on an errand of its own (restock, sell, fix its level)
    // goes its own way, and the plan waits (re-review #227, 3). It wants
    // off this level: it leaves its own way (the think's "flee the level"
    // steps), and the trip goes on from wherever it arrives.
    match scared_home(st) {
        Home::Fear if running().as_ref().is_some_and(|r| r.end != End::Home) => {
            finish(st, "aborted: the core wants town");
            return true;
        }
        Home::Fear | Home::Errand => return false,
        Home::No => {}
    }
    let Some(t) = target(st) else {
        return false;
    };
    // `/explored` and `/stall` are measured on every think of a planned
    // level, the core's own leaving or fleeing included: it spends much of
    // a level "leaving" ("Looking for any stair"), and they never fired
    // when only checked after that defer (re-review #227, 1).
    if depth > 0 && depth == t {
        if let Some(why) = level_done(st, depth) {
            borg_note(st, &format!("# Errand: DL{depth} {why}."));
            if !(st.borg.goal.leaving || st.borg.goal.fleeing) {
                return move_on(st);
            }
            // The core is leaving already: the level is explored, and it
            // leaves its own way; the plan goes on from where it arrives.
            if let Some(r) = running().as_mut() {
                r.step += 1;
            }
            if target(st).is_none() {
                finish(st, "done");
                return true;
            }
            return false;
        }
    }
    if st.borg.goal.leaving || st.borg.goal.fleeing {
        return false;
    }
    // In town the core shops and leaves by itself (`on_leave_level`); on a
    // level of the plan it explores, until it is done with the level, or
    // the level is explored enough, or nothing new is seen for too long.
    if depth == 0 {
        return false;
    }
    if depth == t {
        // Measured above: not done yet.
        return false;
    }
    if stuck(st, depth) {
        return false;
    }
    walk(st, depth, t)
}

/// Whether the level under the errand is done with before the core says so:
/// `/explored` (the share of its floor seen) or `/stall` (nothing new seen
/// for that many game turns). The share counts the level's real floor, a
/// read of the engine (the core reads it too, its "cheats"); it only ends
/// a level, it never steers a move.
fn level_done(st: &BorgState, depth: i32) -> Option<String> {
    let turn = i64::from(engine::game_turn());
    let (mut floor, mut seen, mut known) = (0u32, 0u32, 0u32);
    for y in 0..crate::borg_cave::AUTO_MAX_Y {
        for x in 0..crate::borg_cave::AUTO_MAX_X {
            if !engine::square_in_bounds(x, y) {
                continue;
            }
            let mine = st.cave.grid(x, y).feat != FEAT_NONE as u8;
            known += u32::from(mine);
            if matches!(
                u32::from(engine::square_feat(x, y)),
                FEAT_FLOOR
                    | FEAT_CLOSED
                    | FEAT_OPEN
                    | FEAT_BROKEN
                    | FEAT_LESS
                    | FEAT_MORE
                    | FEAT_PASS_RUBBLE
            ) {
                floor += 1;
                seen += u32::from(mine);
            }
        }
    }
    let stalled = {
        let mut guard = running();
        let r = guard.as_mut()?;
        let w = r.watch.get_or_insert(Watch {
            depth,
            known,
            progress: turn,
            arrived: turn,
            stuck: false,
        });
        if w.known == 0 || known < w.known {
            // First count, or the core rebuilt its map without a level
            // change (its clock-wrap precaution): the clock starts again
            // (review #227, finding 7).
            w.known = known;
            w.progress = turn;
        } else if known > w.known {
            w.known = known;
            w.progress = turn;
        }
        turn - w.progress
    };
    let pct = EXPLORED_PCT.load(Ordering::Relaxed);
    if pct > 0 && floor > 0 && seen * 100 >= floor * pct as u32 {
        return Some(format!("explored: {seen} of {floor} floor grids seen"));
    }
    let stall = i64::from(STALL_TURNS.load(Ordering::Relaxed));
    if stall > 0 && stalled >= stall {
        return Some(format!(
            "given up: nothing new seen for {stalled} game turns"
        ));
    }
    None
}

/// Whether the core wants town, and why.
enum Home {
    /// It does not.
    No,
    /// Fear: it flees to town (`fleeing_to_town`), or it rises out of a
    /// depth it is not prepared for. The trip ends: home is its next level.
    Fear,
    /// An errand of its own (`rising` while prepared for this depth):
    /// restocking, selling, fixing its level or experience, missing home.
    /// It goes its own way; the plan waits and goes on after town
    /// (re-review #227, 3; docs/ERRANDS.md: "a trip to town is not fear").
    Errand,
}

fn scared_home(st: &mut BorgState) -> Home {
    let depth = st.borg.tr(BI_CDEPTH);
    if depth == 0 || !(st.borg.goal.fleeing_to_town || st.borg.goal.rising) {
        return Home::No;
    }
    if !st.borg.goal.fleeing_to_town && crate::borg_prepared::borg_prepared(st, depth).is_none() {
        return Home::Errand;
    }
    let ended = match running().as_mut() {
        Some(r) if r.step < usize::MAX && r.target().is_some_and(|t| t > 0) => {
            r.step = usize::MAX;
            true
        }
        _ => false,
    };
    if ended {
        borg_note(st, "# Errand: the core wants town, the trip goes home.");
    }
    Home::Fear
}

/// The core is prepared for the level below `depth` (`borg_prepared`).
fn prepared_below(st: &mut BorgState, depth: i32) -> bool {
    crate::borg_prepared::borg_prepared(st, depth + 1).is_none()
}

/// The errand would take the core below where it is prepared for
/// (re-review #227, 2: "bored and unable to dive" let the errand walk
/// down). A command stops; a ladder trip goes home. `true` when the
/// errand stopped.
fn refuse_descent(st: &mut BorgState, below: i32) -> bool {
    if running().as_ref().is_some_and(|r| !r.skip_unprepared) {
        finish(
            st,
            &format!("aborted: the core is not prepared for DL{below}"),
        );
        return true;
    }
    if let Some(r) = running().as_mut() {
        r.step = usize::MAX;
    }
    borg_note(
        st,
        &format!("# Errand: not prepared for DL{below}; the trip goes home."),
    );
    false
}

/// The errand has been on this level for longer than `/stall` allows (at
/// least 10,000 game turns) and has not moved on: its stairs are not
/// reachable, or the core keeps doing something else (the owner saw a
/// ladder run to a corner of a level and stay; a necromancer sold and
/// re-bought in town for 13,890 turns). The errand stops steering here: a
/// command stops; a ladder or trips lets the core play as itself until the
/// level changes. `true` when the errand stops steering.
fn stuck(st: &mut BorgState, depth: i32) -> bool {
    let turn = i64::from(engine::game_turn());
    // `/stall`'s value, but never under 10,000 game turns: walking a
    // level to its stairs takes longer than noticing nothing new (a
    // `/stall 60` trial aborted the `/walk-back` after it). `/stall off`
    // turns this off too.
    let stall = i64::from(STALL_TURNS.load(Ordering::Relaxed));
    let limit = if stall == 0 { 0 } else { stall.max(10_000) };
    let (first, command) = {
        let mut guard = running();
        let Some(r) = guard.as_mut() else {
            return false;
        };
        let command = !r.skip_unprepared;
        let Some(w) = r.watch.as_mut().filter(|w| w.depth == depth) else {
            return false;
        };
        if w.stuck {
            return true;
        }
        if limit == 0 || turn - w.arrived < limit {
            return false;
        }
        w.stuck = true;
        (turn - w.arrived, command)
    };
    if command {
        finish(
            st,
            &format!("aborted: no way on from DL{depth} in {first} game turns"),
        );
    } else {
        borg_note(
            st,
            &format!(
                "# Errand: no way on from DL{depth} in {first} game turns; the core plays on."
            ),
        );
    }
    true
}

/// One step of the stairs from `depth` toward `t`, with the core's own
/// stair flows (the Lunal power-dive's steps, `borg_think_dungeon.rs`).
fn walk(st: &mut BorgState, depth: i32, t: i32) -> bool {
    if t > depth {
        // No level is forced (docs/ERRANDS.md): never down to a level the
        // core is not prepared for.
        if !prepared_below(st, depth) {
            return refuse_descent(st, depth + 1) || stairs_up(st);
        }
        stairs_down(st)
    } else if t < depth {
        stairs_up(st)
    } else {
        false
    }
}

fn underfoot(st: &BorgState) -> u32 {
    let (x, y) = (st.borg.c.x, st.borg.c.y);
    u32::from(st.cave.grid(x, y).feat)
}

fn stairs_down(st: &mut BorgState) -> bool {
    if underfoot(st) == FEAT_MORE {
        borg_keypress(st, keycode_t::from(b'>'));
        return true;
    }
    crate::borg_flow::borg_flow_old(st, GOAL_FLEE)
        || crate::borg_flow_stairs::borg_flow_stair_more(st, GOAL_FLEE, false, false)
}

fn stairs_up(st: &mut BorgState) -> bool {
    if underfoot(st) == FEAT_LESS {
        borg_keypress(st, keycode_t::from(b'<'));
        return true;
    }
    crate::borg_flow::borg_flow_old(st, GOAL_FLEE)
        || crate::borg_flow_stairs::borg_flow_stair_less(st, GOAL_FLEE, false)
}

/// The outcome line (`BORG_ERRAND_OUT`), if asked for.
fn report(st: &BorgState, outcome: &str, turn: i64) {
    let Some(path) = std::env::var_os("BORG_ERRAND_OUT") else {
        return;
    };
    let (name, trips, skipped) = running().as_ref().map_or((String::new(), 0, 0), |r| {
        (r.name.clone(), r.trip.min(r.trips.len()), r.skipped)
    });
    let line = format!(
        "{{\"errand\":\"{}\",\"outcome\":\"{outcome}\",\"turn\":{turn},\"trips_done\":{trips},\"skipped\":{skipped},\"depth\":{},\"clevel\":{},\"chp\":{},\"mhp\":{}}}\n",
        name.replace('"', "'"),
        st.borg.tr(BI_CDEPTH),
        st.borg.tr(BI_CLEVEL),
        st.borg.tr(BI_CURHP),
        st.borg.tr(BI_MAXHP),
    );
    use std::io::Write;
    if let Ok(mut f) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
    {
        let _ = f.write_all(line.as_bytes());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config_test_dir() -> std::path::PathBuf {
        let nonce = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)
            .unwrap().as_nanos();
        let root = std::env::temp_dir().join(format!("borg-config-{}-{nonce}", std::process::id()));
        std::fs::create_dir(&root).unwrap();
        root
    }

    #[test]
    fn missing_config_is_absent_but_read_and_utf8_errors_are_not() {
        let root = config_test_dir();
        let path = root.join("borg.txt");
        assert_eq!(read_borg_txt(&path).unwrap(), "");
        assert_eq!(borg_txt_setting_at(&path, "borg_reborn_core"), None);
        std::fs::write(&path, "# comment\nborg_reborn_core = 1\n").unwrap();
        assert_eq!(borg_txt_setting_at(&path, "borg_reborn_core"), Some("1".into()));
        std::fs::write(&path, b"borg_reborn_core = 1\n# \xff").unwrap();
        let error = read_borg_txt(&path).unwrap_err();
        assert!(error.contains("borg.txt") && error.contains("UTF-8"));
        assert!(read_borg_txt(&root).unwrap_err().contains("cannot read borg.txt"));
        std::fs::remove_dir_all(root).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn config_paths_preserve_non_utf8_os_bytes() {
        use std::os::unix::ffi::OsStrExt;
        let root = config_test_dir();
        let path = root.join(std::ffi::OsStr::from_bytes(b"user-\xff"));
        let decoded = borg_txt_path(path.as_os_str().as_bytes().to_vec()).unwrap();
        assert_eq!(decoded, path);
        std::fs::write(&path, "borg_reborn_core=0").unwrap();
        assert_eq!(borg_txt_setting_at(&decoded, "borg_reborn_core"), Some("0".into()));
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn config_file_is_checked_again_on_reentry() {
        if let Some(path) = std::env::var_os("BORG_CONFIG_TEST_PATH") {
            let path = std::path::PathBuf::from(path);
            assert_eq!(borg_txt_setting_at(&path, "borg_reborn_core"), Some("1".into()));
            std::fs::write(&path, b"# \xff").unwrap();
            borg_txt_setting_at(&path, "borg_reborn_core");
            panic!("an invalid reread silently used defaults");
        }
        let root = config_test_dir();
        let path = root.join("borg.txt");
        std::fs::write(&path, "borg_reborn_core=1").unwrap();
        let result = std::process::Command::new(std::env::current_exe().unwrap())
            .args(["--exact", "shell::tests::config_file_is_checked_again_on_reentry", "--nocapture"])
            .env("BORG_CONFIG_TEST_PATH", &path).output().unwrap();
        assert_eq!(result.status.code(), Some(2), "{:?}", result);
        assert!(String::from_utf8_lossy(&result.stderr).contains("cannot read borg.txt"));
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn trips_parse() {
        assert_eq!(
            Errand::parse("trips:1*/1,2,1"),
            Some(Errand::Trips(vec![
                Trip {
                    levels: vec![1],
                    repeat: true,
                    min_level: None
                },
                Trip {
                    levels: vec![1, 2, 1],
                    repeat: false,
                    min_level: None
                },
            ]))
        );
        assert_eq!(
            Errand::parse("trips:1*/1,2,1*@5"),
            Some(Errand::Trips(vec![
                Trip {
                    levels: vec![1],
                    repeat: true,
                    min_level: None
                },
                Trip {
                    levels: vec![1, 2, 1],
                    repeat: true,
                    min_level: Some(5)
                },
            ]))
        );
        assert_eq!(Errand::parse("trips:1*/1,2@x"), None);
        assert_eq!(Errand::parse("trips:1,0"), None, "depth 0 is not a level");
        assert_eq!(Errand::parse("trips:"), None);
        assert_eq!(Errand::parse("dive:5"), None);
    }

    #[test]
    fn a_line_holds_commands_in_turn() {
        use Command as C;
        assert_eq!(
            parse_line("/stopscared 40; /dive 2;/walk-back"),
            Ok(vec![
                C::StopScared(Some(40)),
                C::Errand(Errand::Dive(2)),
                C::Errand(Errand::WalkBack)
            ])
        );
        assert_eq!(parse_line("/dive 2; /sing"), Err("/sing".to_string()));
    }

    #[test]
    fn every_class_has_a_ladder() {
        let text = include_str!("../ladders.txt");
        for class in CLASSES {
            assert!(ladder_of(text, class).is_some(), "{class}");
        }
    }

    #[test]
    fn commands_parse() {
        use Command as C;
        assert_eq!(C::parse("/town"), Some(C::Errand(Errand::Town)));
        assert_eq!(C::parse("/shopping"), Some(C::Errand(Errand::Shopping)));
        assert_eq!(C::parse("/shopping 2"), None);
        assert_eq!(C::parse("/dive 2"), Some(C::Errand(Errand::Dive(2))));
        assert_eq!(C::parse("dive:3"), Some(C::Errand(Errand::Dive(3))));
        assert_eq!(C::parse("/dive"), None);
        assert_eq!(C::parse("/dive 0"), None);
        assert_eq!(
            C::parse("/explore-level"),
            Some(C::Errand(Errand::ExploreLevel))
        );
        assert_eq!(
            C::parse("/dive-exploring 2"),
            Some(C::Errand(Errand::DiveExploring(2)))
        );
        assert_eq!(C::parse("/ladder"), Some(C::Errand(Errand::Ladder)));
        assert_eq!(C::parse("/walk-back"), Some(C::Errand(Errand::WalkBack)));
        assert_eq!(
            C::parse("/explore-back"),
            Some(C::Errand(Errand::ExploreBack))
        );
        assert!(matches!(
            C::parse("/trips 1*/1,2,1*@3"),
            Some(C::Errand(Errand::Trips(_)))
        ));
        assert!(matches!(
            C::parse("trips:1*/1,2,1"),
            Some(C::Errand(Errand::Trips(_)))
        ));
        assert_eq!(C::parse("/stopscared true"), Some(C::StopScared(Some(100))));
        assert_eq!(C::parse("/stopscared flee"), Some(C::StopScared(Some(100))));
        assert_eq!(C::parse("/stopscared 50"), Some(C::StopScared(Some(50))));
        assert_eq!(C::parse("/stopscared off"), Some(C::StopScared(None)));
        assert_eq!(C::parse("/stopscared 0"), None);
        assert_eq!(C::parse("/stopscared maybe"), None);
        assert_eq!(C::parse("/stop"), Some(C::Stop));
        assert_eq!(C::parse("none"), Some(C::Stop));
        assert_eq!(C::parse("/explored 80"), Some(C::Explored(Some(80))));
        assert_eq!(C::parse("/explored off"), Some(C::Explored(None)));
        assert_eq!(C::parse("/explored 120"), None);
        assert_eq!(C::parse("/stall 5000"), Some(C::Stall(Some(5000))));
        assert_eq!(C::parse("/fly"), None);
    }

    #[test]
    fn a_dive_ends_on_arrival_and_town_ends_home() {
        let dive = Running {
            trips: vec![Trip {
                levels: vec![3],
                repeat: false,
                min_level: None,
            }],
            trip: 0,
            step: 0,
            budget: 0,
            started: None,
            skipped: 0,
            end: End::Arrive,
            skip_unprepared: false,
            from_here: None,
            recall: false,
            name: String::new(),
            watch: None,
            owner: None,
        };
        assert_eq!(dive.target(), Some(3));
        let mut stop = Running {
            end: End::Stop,
            ..dive.clone()
        };
        stop.step = 1;
        assert_eq!(stop.target(), None, "explored: done where it is");
        let town = Running {
            trips: vec![Trip {
                levels: vec![],
                repeat: false,
                min_level: None,
            }],
            end: End::Home,
            ..dive
        };
        assert_eq!(town.target(), Some(0));
    }

    #[test]
    fn a_trip_walks_its_levels_then_home_then_the_next_trip() {
        let trip = |levels: Vec<i32>| Trip {
            levels,
            repeat: false,
            min_level: None,
        };
        let mut r = Running {
            trips: vec![trip(vec![1]), trip(vec![1, 2, 1])],
            trip: 0,
            step: 0,
            budget: 0,
            started: None,
            skipped: 0,
            end: End::Home,
            skip_unprepared: true,
            from_here: None,
            recall: false,
            name: String::new(),
            watch: None,
            owner: None,
        };
        assert_eq!(r.target(), Some(1));
        r.step = 1;
        assert_eq!(r.target(), Some(0), "the trip goes home");
        r.trip = 1;
        r.step = 0;
        assert_eq!(r.target(), Some(1));
        r.step = 1;
        assert_eq!(r.target(), Some(2));
        r.step = 3;
        assert_eq!(r.target(), Some(0));
        r.trip = 2;
        assert_eq!(r.target(), None, "every trip is over");
    }
}
