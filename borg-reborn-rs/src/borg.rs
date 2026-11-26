//! The control loop and the minimal Borg (task T1.5, issue #8).
//!
//! The loop keeps the C Borg's observable mechanism (spec §2.2, §2.7;
//! ARCHITECTURE.md *The control loop*): queued keys are returned first; the
//! Borg thinks only when its queue is empty; when the game is resting or
//! repeating a command it returns `EVT_NONE` and lets the game run; every
//! action is a complete key sequence that starts with `ESCAPE`. Decisions use
//! the Borg's own RNG and never the game's.
//!
//! The minimal Borg is a placeholder for M2–M3: it rests when hurt, takes a
//! down staircase when standing on one, and otherwise walks to a random
//! neighbour that is not known to be a wall or a shop. A think that finds no
//! game time passed since a refused rest walks instead, and after
//! `max_tries_per_turn` such thinks (a parameter, [`crate::params`]) it takes an action that always uses
//! game time (see [`stand_still`]), so it cannot loop without time passing.

use crate::engine::{self, Key, PlayerView, ESCAPE};
use crate::executor::{self, Action, Executor};
use crate::params::Params;
use crate::strategy::{Decision, DecisionCtx, FirstStrategy, Strategy};
use crate::trace::{Budget, Candidate, TraceLine};
use std::sync::Mutex;

/// Feature indices of `list-terrain.h` (FEAT_NONE = 0, ...).
pub mod feat {
    pub const NONE: u32 = 0;
    pub const FLOOR: u32 = 1;
    pub const CLOSED: u32 = 2;
    pub const OPEN: u32 = 3;
    pub const BROKEN: u32 = 4;
    pub const LESS: u32 = 5;
    pub const MORE: u32 = 6;
    pub const STORE_GENERAL: u32 = 7;
    pub const STORE_ALCHEMY: u32 = 11;
    pub const HOME: u32 = 14;
    pub const RUBBLE: u32 = 16;
    pub const MAGMA: u32 = 17;
    pub const QUARTZ: u32 = 18;
    pub const MAGMA_K: u32 = 19;
    pub const QUARTZ_K: u32 = 20;
    pub const GRANITE: u32 = 21;
    pub const PERM: u32 = 22;
    pub const LAVA: u32 = 23;
    pub const PASS_RUBBLE: u32 = 24;
}

/// Directions 1–9 (keypad layout, 5 = stay) as (dx, dy).
const DIRS: [(u8, i32, i32); 8] = [
    (1, -1, 1),
    (2, 0, 1),
    (3, 1, 1),
    (4, -1, 0),
    (6, 1, 0),
    (7, -1, -1),
    (8, 0, -1),
    (9, 1, -1),
];

/// Whether the minimal Borg may step onto a grid with this known feature.
/// Unknown grids are tried; known walls, rubble, lava and shops are not.
pub fn walkable(f: Option<u32>) -> bool {
    matches!(
        f,
        Some(
            feat::NONE
                | feat::FLOOR
                | feat::CLOSED
                | feat::OPEN
                | feat::BROKEN
                | feat::LESS
                | feat::MORE
                | feat::PASS_RUBBLE
        )
    )
}

/// Whether a known feature can be tunnelled: rubble, magma, quartz and
/// granite carry HURT_ROCK (engine `list-terrain.h:29-34`); permanent
/// walls (`FEAT_PERM`, :35) never do, and a secret door is opened, not
/// dug (T4.6, #123).
pub fn diggable(f: Option<u32>) -> bool {
    matches!(f, Some(feat) if (feat::RUBBLE..=feat::GRANITE).contains(&feat))
}

/// Whether a known feature is a shop entrance (the home included).
pub fn is_shop(f: Option<u32>) -> bool {
    matches!(f, Some(feat::STORE_GENERAL..=feat::HOME))
}

/// The action that takes game time where the Borg stands. `Hold` does,
/// except on a shop entrance: there `do_cmd_hold` enters the store and sets
/// `energy_use = 0` ("turn will be taken exiting the shop",
/// `angband/src/cmd-cave.c`), so repeated holds would stay at one turn
/// (M1 re-review N2). A step off is not certain to take time either: an
/// afraid player walking into a visible monster is refused before any
/// energy is spent (`do_cmd_walk_test`; re-review R2). So on a shop the
/// Borg alters a neighbouring grid instead: `do_cmd_alter_aux` spends a
/// move's energy before it looks at the grid, and its attack path charges
/// every blow even when afraid (`py_attack`). It picks a known passable
/// neighbour first, where altering just spins it around, then an unknown
/// one, then the rest.
pub fn stand_still(here: Option<u32>, around: impl Fn(i32, i32) -> Option<u32>) -> Action {
    if !is_shop(here) {
        return Action::Hold;
    }
    let rank = |f: Option<u32>| match f {
        Some(feat::NONE) => 1,
        f if walkable(f) => 0,
        _ => 2,
    };
    let (d, _, _) = DIRS
        .iter()
        .min_by_key(|(_, dx, dy)| rank(around(*dx, *dy)))
        .expect("eight directions");
    Action::Alter(*d)
}

/// The Borg's own random numbers (xorshift64*). Seeded with a constant, so
/// a run is reproducible; it never touches the game's RNG.
#[derive(Clone, Debug)]
pub struct Rng(u64);

impl Rng {
    pub const SEED: u64 = 0x9E37_79B9_7F4A_7C15;

    pub fn new() -> Self {
        Rng(Self::SEED)
    }

    pub fn next_u64(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.0 = x;
        x.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    pub fn below(&mut self, n: usize) -> usize {
        (self.next_u64() % n as u64) as usize
    }

    pub fn state(&self) -> u32 {
        self.0 as u32
    }
}

impl Default for Rng {
    fn default() -> Self {
        Self::new()
    }
}

/// What the Borg remembers between thinks.
#[derive(Debug, Default)]
pub struct Memory {
    /// The game turn and action of the last think.
    pub last: Option<(i64, Action)>,
    /// How many earlier thinks saw the current game turn: actions that the
    /// game refused, or that took no time (a walk into a known wall, a rest
    /// it would not start). Store actions reset this count because their
    /// effects are checked and the town's finite jobs are progress (T3.5).
    pub same_turn: u32,
    /// Ascend/Descend actions already chosen at this game turn. These have
    /// their own cap: level generation can make depth oscillate forever
    /// without advancing time, so stairs must not reset `same_turn` (M3
    /// review, PR #135 finding 1).
    pub level_changes: u32,
}

impl Memory {
    /// Record a think at `turn` that chose `action`.
    pub fn record(&mut self, turn: i64, action: Action) {
        if self.last.is_none_or(|(t, _)| t != turn) {
            self.same_turn = 0;
            self.level_changes = 0;
        } else if matches!(action, Action::Buy { .. } | Action::Sell { .. }) {
            self.same_turn = 0;
        } else {
            self.same_turn += 1;
        }
        if matches!(action, Action::Descend | Action::Ascend) {
            self.level_changes += 1;
        }
        self.last = Some((turn, action));
    }

    /// Thinks already made at `turn` (0 if game time has passed since).
    pub fn tries_at(&self, turn: i64) -> u32 {
        match self.last {
            Some((t, _)) if t == turn => self.same_turn + 1,
            _ => 0,
        }
    }

    /// Level changes already chosen at `turn` (0 if game time has passed).
    pub fn level_changes_at(&self, turn: i64) -> u32 {
        match self.last {
            Some((t, _)) if t == turn => self.level_changes,
            _ => 0,
        }
    }
}

struct Borg {
    /// The executor (T3.1): the key queue, the pending effect and the
    /// prompts answered since the last think.
    executor: Executor,
    rng: Rng,
    /// The quick-generator value engine reads draw from (`with_borg_rng`).
    engine_rng: u32,
    mem: Memory,
    /// The observation log (`BORG_OBSERVATION_LOG=FILE`): one JSON line per
    /// think (T2.1). Off by default.
    observations: Option<std::io::BufWriter<std::fs::File>>,
    /// L2: what the Borg remembers, updated once per think (T2.2).
    world: crate::world::World,
    /// The world log (`BORG_WORLD_LOG=FILE`): the world's digest after each
    /// think, to check that replaying the observation log rebuilds it.
    worlds: Option<std::io::BufWriter<std::fs::File>>,
    /// The engine-truth log (`BORG_RUN_DIR/truth.jsonl`, test-perception
    /// builds only, T2.5): each think's comparison with the engine, one JSON
    /// line.
    #[cfg(feature = "test-perception")]
    truth: Option<std::io::BufWriter<std::fs::File>>,
    /// L3: the game's static data, read at the first think (T2.3). Written
    /// once to `BORG_KNOWLEDGE_LOG=FILE` when that is set.
    knowledge: Option<crate::knowledge::Knowledge>,
    /// L5: the first strategy (T3.3), which replaces the minimal Borg's
    /// placeholder policy.
    strategy: FirstStrategy,
    /// The parameters, from the profile read at start (T2.4).
    params: Params,
    /// The decision trace (`BORG_RUN_DIR/trace.jsonl`), one line per think.
    trace: Option<std::io::BufWriter<std::fs::File>>,
    thinks: u64,
    /// Town sessions (#191): the handover, the state records and the
    /// object log, all off unless their variables are set.
    town: crate::town::Town,
}

/// The parameters of this run: the profile named by `BORG_PROFILE`, or the
/// defaults. A profile that cannot be read or does not parse stops the
/// game: a run must never play with parameters it did not ask for.
fn load_params() -> Params {
    let Some(path) = std::env::var_os("BORG_PROFILE") else {
        return Params::default();
    };
    // The name records keep (run.sh: relative to the checkout).
    let name =
        std::env::var("BORG_PROFILE_NAME").unwrap_or_else(|_| path.to_string_lossy().into_owned());
    let bytes =
        std::fs::read(&path).unwrap_or_else(|e| panic!("borg-reborn-rs: profile {name}: {e}"));
    Params::from_profile(&name, &bytes).unwrap_or_else(|e| panic!("borg-reborn-rs: {e}"))
}

/// The run's outputs (`BORG_RUN_DIR`, set by harness/run.sh): the parameter
/// table goes to `params.json` now, and the trace is opened.
fn open_run_outputs(params: &Params) -> Option<std::io::BufWriter<std::fs::File>> {
    let dir = std::path::PathBuf::from(std::env::var_os("BORG_RUN_DIR")?);
    let table = serde_json::to_string_pretty(&params.table()).expect("plain data");
    std::fs::write(dir.join("params.json"), table + "\n").ok()?;
    std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(dir.join("trace.jsonl"))
        .ok()
        .map(std::io::BufWriter::new)
}

/// An append-only log named by an environment variable, if it is set.
fn open_log(var: &str) -> Option<std::io::BufWriter<std::fs::File>> {
    std::env::var_os(var).and_then(|path| {
        std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .ok()
            .map(std::io::BufWriter::new)
    })
}

static BORG: Mutex<Option<Borg>> = Mutex::new(None);

/// The game options the Borg sets when it starts (docs/ENGINE.md).
pub const GAME_OPTIONS: &[(&core::ffi::CStr, bool)] = &[(c"auto_more", true)];

/// Whether the decision core is borg-rs's (#207), read once at the first
/// `^Z`: `BORG_CORE` in the environment, else `borg_reborn_core` in the
/// run's pborg.yaml. The zoo's integer profiles use 1 for borg-rs, 0 for reborn.
pub fn core_is_borg_rs() -> bool {
    static CORE: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *CORE.get_or_init(|| {
        let environment = std::env::var_os("BORG_CORE");
        let file = if environment.is_none() {
            borg_rs::shell::warn_missing_pborg_config();
            borg_rs::shell::borg_txt_setting("borg_reborn_core")
        } else {
            None
        };
        let selected = if let Some(value) = environment.as_ref() {
            value
                .to_str()
                .ok_or_else(|| format!("BORG_CORE: invalid non-UTF-8 core selector {value:?}"))
                .and_then(|value| parse_core_selector(Some(value), "BORG_CORE"))
        } else {
            parse_core_selector(file.as_deref(), "pborg.yaml borg_reborn_core")
        };
        let selected = selected.unwrap_or_else(|error| {
            eprintln!("borg-reborn-rs: {error}");
            std::process::exit(2);
        });
        // An explicit core environment override keeps file access lazy. A
        // file-selected original core still rejects a requested file plan.
        let plan_requested = std::env::var_os("BORG_ERRANDS_PLAN").is_some()
            || (!selected
                && environment.is_none()
                && borg_rs::shell::borg_txt_setting("borg_plan").is_some());
        if !selected && plan_requested {
            eprintln!("borg-reborn-rs: errands plans require BORG_CORE=borg-rs");
            std::process::exit(2);
        }
        selected
    })
}

/// The absence of a selector preserves the original-core default. Invalid
/// selectors must never silently change the arm of a measured experiment.
fn parse_core_selector(value: Option<&str>, source: &str) -> Result<bool, String> {
    match value {
        None | Some("reborn" | "0") => Ok(false),
        Some("borg-rs" | "1") => Ok(true),
        Some(value) => Err(format!(
            "{source}: invalid core selector {value:?}; expected reborn/0 or borg-rs/1"
        )),
    }
}

/// Validate the configured shell command before a measured run starts.
/// Original-core runs do not consult the vendored core's errand setting.
pub fn configured_errand() -> Option<String> {
    let environment = std::env::var_os("BORG_ERRAND");
    let file = if environment.is_none() {
        borg_rs::shell::borg_txt_setting("borg_reborn_errand")
    } else {
        None
    };
    let selected = if let Some(value) = environment.as_ref() {
        value
            .to_str()
            .ok_or_else(|| format!("BORG_ERRAND: invalid non-UTF-8 errand selector {value:?}"))
            .and_then(|value| parse_errand_setting(Some(value), "BORG_ERRAND"))
    } else {
        parse_errand_setting(file.as_deref(), "pborg.yaml borg_reborn_errand")
    };
    selected.unwrap_or_else(|error| {
        eprintln!("borg-reborn-rs: {error}");
        std::process::exit(2);
    })
}

fn parse_errand_setting(value: Option<&str>, source: &str) -> Result<Option<String>, String> {
    match value {
        None | Some("" | "none") => Ok(None),
        Some(value) => {
            let invalid =
                |reason: &str| format!("{source}: invalid errand selector {value:?}: {reason}");
            let commands = borg_rs::shell::parse_line(value).map_err(|error| invalid(&error))?;
            if commands.is_empty() {
                return Err(invalid("empty command line; use none to disable errands"));
            }
            Ok(Some(value.to_owned()))
        }
    }
}

/// A configured fear threshold is either valid or a configuration error;
/// an invalid value must not silently become the core's default threshold.
pub fn configured_stopscared() -> Option<borg_rs::shell::Command> {
    let environment = std::env::var_os("BORG_STOPSCARED");
    let file = if environment.is_none() {
        borg_rs::shell::borg_txt_setting("borg_reborn_stopscared")
    } else {
        None
    };
    let selected = if let Some(value) = environment.as_ref() {
        value
            .to_str()
            .ok_or_else(|| format!("BORG_STOPSCARED: invalid non-UTF-8 stopscared value {value:?}"))
            .and_then(|value| parse_stopscared_setting(Some(value), "BORG_STOPSCARED"))
    } else {
        parse_stopscared_setting(file.as_deref(), "pborg.yaml borg_reborn_stopscared")
    };
    selected.unwrap_or_else(|error| {
        eprintln!("borg-reborn-rs: {error}");
        std::process::exit(2);
    })
}

fn parse_stopscared_setting(
    value: Option<&str>,
    source: &str,
) -> Result<Option<borg_rs::shell::Command>, String> {
    match value {
        None | Some("" | "none") => Ok(None),
        Some(value) => borg_rs::shell::Command::parse(&format!("stopscared {value}"))
            .map(Some)
            .ok_or_else(|| format!(
                "{source}: invalid stopscared value {value:?}; expected off, flee, or 1..=1000 (boolean aliases also accepted)"
            )),
    }
}

/// A configured errand budget is a nonnegative number of game turns.
/// This is consulted only for a configured vendored-core command line.
pub fn configured_errand_budget() -> i64 {
    let value = std::env::var_os("BORG_ERRAND_BUDGET");
    let selected = match value.as_ref() {
        None => Ok(0),
        Some(value) => value
            .to_str()
            .ok_or_else(|| format!("BORG_ERRAND_BUDGET: invalid non-UTF-8 budget {value:?}"))
            .and_then(|value| parse_errand_budget(Some(value))),
    };
    selected.unwrap_or_else(|error| {
        eprintln!("borg-reborn-rs: {error}");
        std::process::exit(2);
    })
}

fn parse_errand_budget(value: Option<&str>) -> Result<i64, String> {
    match value {
        None | Some("none") => Ok(0),
        Some(value) => value.parse::<i64>().ok().filter(|n| *n >= 0).ok_or_else(|| {
            format!("BORG_ERRAND_BUDGET: invalid budget {value:?}; expected 0..={} game turns or none (0 means unbounded)", i64::MAX)
        }),
    }
}

/// Run a key from the shared Borg command prompt. The reborn core supports start.
pub fn command(cmd: u8) {
    if cmd != b'z' {
        return;
    }
    // The game options the Borg relies on (spec §1.2; docs/ENGINE.md): no
    // -more- prompts. Walking into a closed door opens it in 4.2.6 without
    // an option (`move_player` calls `do_cmd_alter_aux`); the `easy_alter`
    // the Borg used to set does not exist in 4.2.6, and setting it did
    // nothing. An option the engine does not know stops the Borg.
    for (name, on) in GAME_OPTIONS {
        assert!(
            engine::set_option(name, *on),
            "borg-reborn-rs: the engine has no option {name:?}"
        );
    }
    let mut b = BORG.lock().unwrap_or_else(|e| e.into_inner());
    b.get_or_insert_with(|| {
        let params = load_params();
        let trace = open_run_outputs(&params);
        Borg {
            executor: Executor::default(),
            rng: Rng::new(),
            engine_rng: Rng::SEED as u32,
            mem: Memory::default(),
            observations: open_log("BORG_OBSERVATION_LOG"),
            world: crate::world::World::new(),
            worlds: open_log("BORG_WORLD_LOG"),
            #[cfg(feature = "test-perception")]
            truth: std::env::var_os("BORG_RUN_DIR").map(|d| {
                std::fs::File::create(std::path::Path::new(&d).join("truth.jsonl"))
                    .map(std::io::BufWriter::new)
                    .expect("truth.jsonl cannot be created")
            }),
            knowledge: None,
            strategy: FirstStrategy::default(),
            params,
            trace,
            thinks: 0,
            town: crate::town::Town::from_env(),
        }
    })
    .executor
    .clear();
    drop(b);
    engine::set_hook(true);
}

/// The engine wants a key (`inkey_hack`). The steps follow the C Borg's
/// `generate_keypress` (borg.c:268-321): deactivation and death first,
/// then the flush, then the prompt catcher, then the key queue, the user
/// key, the rest/repeat pass-through, and only then a think.
pub fn next_key(flush: bool) -> Key {
    let mut guard = BORG.lock().unwrap_or_else(|e| e.into_inner());
    let Some(b) = guard.as_mut() else {
        return Key::Code(ESCAPE);
    };
    if flush {
        b.executor.clear();
    }
    // A dead player stops the Borg (check_for_deactivate_or_death,
    // borg.c:289) before any key is chosen.
    let Some(view) = engine::player_view() else {
        return Key::Code(ESCAPE);
    };
    if view.dead {
        engine::set_hook(false);
        return Key::Code(ESCAPE);
    }
    // The town handover (`BORG_TOWN_PLAYER=1`, #191; borg-rs
    // `borg.rs:407-462`): in town the engine reads the real keyboard and the
    // Borg decides nothing, so no prompt is answered and no key of the
    // player's stops it. The state and the pack are recorded as the player
    // changes them.
    if b.town.hands_over(view.depth) {
        b.executor.clear();
        if b.town.records() {
            if let Some(o) = engine::with_borg_rng(&mut b.engine_rng, engine::perceive) {
                b.town.note_state(&o);
                b.town.note_pack(&o);
            }
        }
        return Key::None;
    }
    // The prompt catcher (T3.1): before the queue, as in the C Borg
    // (borg.c:304-308), so a prompt is answered even when keys remain
    // queued. No rule may match a prompt an action's own keys pre-answer;
    // `None` falls through to the queue.
    let line = engine::message_line();
    if let Some(p) = executor::answer(
        &line.text,
        line.dark,
        line.cursor,
        engine::wants_command(),
        b.executor.is_empty(),
    ) {
        b.executor.answered(p.why);
        return Key::Code(p.key);
    }
    if let Some(k) = b.executor.pop() {
        return Key::Code(k);
    }
    // Any real key stops the Borg (spec §2.2 step 8; the GUI's "any key
    // stops it").
    if engine::take_user_key().is_some() {
        engine::set_hook(false);
        return Key::Code(ESCAPE);
    }
    if view.busy {
        return Key::None;
    }
    engine::report_decision(b.rng.state());
    let d = think(b, &view);
    b.thinks += 1;
    write_trace(b, &view, &d);
    b.mem.record(view.turn, d.action);
    Key::Code(b.executor.pop().unwrap_or(ESCAPE))
}

/// One line of the decision trace, after the decision: it reads only what
/// the decision made, so it cannot change play. A failed write loses the
/// trace, never the game.
fn write_trace(b: &mut Borg, view: &PlayerView, d: &Decision) {
    use std::io::Write;
    let Some(log) = b.trace.as_mut() else {
        return;
    };
    let line = TraceLine {
        turn: view.turn,
        think: b.thinks,
        mode: d.mode.into(),
        goal: d.goal.into(),
        budget: Some(Budget {
            unit: "thinks at this game turn".into(),
            left: d.budget_left,
        }),
        candidates: d
            .candidates
            .iter()
            .map(|(a, s)| Candidate {
                action: format!("{a:?}"),
                score: *s,
            })
            .collect(),
        choice: format!("{:?}", d.action),
        threat: d.threat.clone(),
        why: d.why.clone(),
        violations: d.violations.clone(),
        prompts: b.executor.take_answered(),
        profile_sha256: b.params.profile_sha256.clone(),
    };
    if writeln!(log, "{}", line.to_line())
        .and_then(|_| log.flush())
        .is_err()
    {
        b.trace = None;
    }
}

/// Perceive and decide. Every engine read runs with the Borg's own RNG.
fn think(b: &mut Borg, view: &PlayerView) -> Decision {
    #[cfg(feature = "test-hallucinate")]
    let before = engine::game_rng();
    let perceive = |b: &mut Borg| {
        let (x, y) = (view.x, view.y);
        engine::with_borg_rng(&mut b.engine_rng, || {
            let here = engine::known_feature(x, y);
            let around: Vec<Option<u32>> = DIRS
                .iter()
                .map(|(_, dx, dy)| engine::known_feature(x + dx, y + dy))
                .collect();
            // L1 (T2.1): the whole observation, after the minimal Borg's own
            // reads so their draws from the Borg's RNG are unchanged. The
            // placeholder strategy does not use it yet; it is recorded when
            // the observation log is on.
            let observation = engine::perceive();
            (here, around, observation)
        })
    };
    #[cfg(feature = "test-hallucinate")]
    let (here, around, observation) = engine::while_hallucinating(|| perceive(b));
    // ... and hallucination changes only what is drawn, never what is
    // observed: the same think without it sees the same monsters, objects
    // and grid entities (E review, finding 2).
    #[cfg(feature = "test-hallucinate")]
    {
        let plain = engine::with_borg_rng(&mut b.engine_rng, engine::perceive);
        let entities = |o: &Option<crate::observation::Observation>| {
            o.as_ref().map(|o| {
                let grids: Vec<_> = o
                    .panel
                    .grids
                    .iter()
                    .map(|g| (g.monster, g.object_kind))
                    .collect();
                (o.monsters.clone(), o.objects.clone(), grids)
            })
        };
        let (with, without) = (entities(&observation), entities(&plain));
        if with != without {
            let detail = match (with, without) {
                (Some((m1, o1, g1)), Some((m2, o2, g2))) => format!(
                    "monsters {:?} vs {:?}; objects {:?} vs {:?}; grids {:?}",
                    m1.iter().filter(|m| !m2.contains(m)).collect::<Vec<_>>(),
                    m2.iter().filter(|m| !m1.contains(m)).collect::<Vec<_>>(),
                    o1.iter().filter(|o| !o2.contains(o)).collect::<Vec<_>>(),
                    o2.iter().filter(|o| !o1.contains(o)).collect::<Vec<_>>(),
                    g1.iter()
                        .zip(&g2)
                        .enumerate()
                        .filter(|(_, (a, b))| a != b)
                        .take(5)
                        .collect::<Vec<_>>()
                ),
                (a, b) => format!("{} vs {}", a.is_some(), b.is_some()),
            };
            panic!("hallucination changed what is observed: {detail}");
        }
    }
    #[cfg(not(feature = "test-hallucinate"))]
    let (_here, _around, observation) = perceive(b);
    if let Some(o) = observation.as_ref() {
        use std::io::Write;
        if b.knowledge.is_none() {
            b.knowledge = engine::knowledge();
            if let (Some(k), Some(mut log)) = (b.knowledge.as_ref(), open_log("BORG_KNOWLEDGE_LOG"))
            {
                let _ = writeln!(log, "{}", k.to_line()).and_then(|_| log.flush());
            }
        }
        b.world.update(o);
        // A failed write loses the log, never the game.
        if let Some(log) = b.observations.as_mut() {
            if writeln!(log, "{}", o.to_line())
                .and_then(|_| log.flush())
                .is_err()
            {
                b.observations = None;
            }
        }
        if let Some(log) = b.worlds.as_mut() {
            if writeln!(log, "{}", b.world.digest())
                .and_then(|_| log.flush())
                .is_err()
            {
                b.worlds = None;
            }
        }
        // T2.5: the observation and the world against the engine.
        #[cfg(feature = "test-perception")]
        if let Some(log) = b.truth.as_mut() {
            // A test build: a lost report fails loudly, never silently
            // (M2 review, finding 4).
            let line = serde_json::to_string(&engine::truth::check(o, &b.world))
                .expect("truth report serialises");
            writeln!(log, "{line}")
                .and_then(|_| log.flush())
                .expect("truth.jsonl write failed");
        }
    }
    #[cfg(feature = "test-hallucinate")]
    assert_eq!(
        engine::game_rng(),
        before,
        "perception moved the game's RNG"
    );
    // The pending action's effect, checked against this think's
    // observation (ARCHITECTURE.md L6): each action is checked exactly
    // once, by the think after it. A mismatch is a violation for the
    // trace, so a key sequence the game silently ignored is visible —
    // and the strategy's signal that its goal is not progressing.
    // The town records and the object log (#191): read only.
    if let Some(o) = observation.as_ref() {
        b.town.note_state(o);
        b.town.note_pack(o);
    }
    let mut mismatches = match observation.as_ref() {
        Some(o) => b.executor.check_pending(o),
        None => b.executor.drop_pending().into_iter().collect(),
    };
    // L5 decides (T3.3), on the world the observation built, the
    // knowledge read at the first think, and a pathfinder that lives
    // exactly this think (ARCHITECTURE.md L4: no map outlives its
    // observation).
    let empty_knowledge = crate::knowledge::Knowledge::default();
    let knowledge = b.knowledge.as_ref().unwrap_or(&empty_knowledge);
    let mut paths = crate::path::Paths::new_with_risk(&b.world, b.params.risk_aversion_pct);
    let mut ctx = DecisionCtx {
        view,
        world: &b.world,
        knowledge,
        paths: &mut paths,
        mem: &b.mem,
        rng: &mut b.rng,
        params: &b.params,
        failures: &mismatches,
    };
    let mut d = b.strategy.decide(&mut ctx);
    d.violations.append(&mut mismatches);
    // Queue the chosen action's keys. Without an observation the keys
    // still go out (only a sale's quantity reads the pack), and the
    // violation above already says why.
    let empty = crate::observation::Observation::default();
    let o = observation.as_ref().unwrap_or(&empty);
    if b.town.records() {
        let ammo = b
            .knowledge
            .as_ref()
            .and_then(|k| b.world.character.ammo_tval(k));
        b.town.note_action(d.action, o, ammo);
    }
    b.executor.queue(d.action, o);
    if observation.is_none() {
        d.violations
            .push("no observation: perception found no character or level".into());
    }
    d
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::KC_ENTER;
    use crate::observation::Observation;

    #[test]
    fn core_selector_rejects_silent_arm_changes() {
        for (value, expected) in [
            (None, false),
            (Some("reborn"), false),
            (Some("0"), false),
            (Some("borg-rs"), true),
            (Some("1"), true),
        ] {
            assert_eq!(parse_core_selector(value, "BORG_CORE"), Ok(expected));
        }
        for source in ["BORG_CORE", "pborg.yaml borg_reborn_core"] {
            for value in ["", "true", "2", "7", "borg_rs", "BORG-RS", " 1"] {
                let error = parse_core_selector(Some(value), source).unwrap_err();
                assert!(error.contains(source));
                assert!(error.contains(&format!("{value:?}")));
            }
        }
    }

    #[test]
    fn configured_errands_preserve_valid_commands_and_reject_silent_fallback() {
        for value in [None, Some(""), Some("none")] {
            assert_eq!(parse_errand_setting(value, "BORG_ERRAND"), Ok(None));
        }
        for value in [
            "/town",
            "/level-up 10",
            "/shopping",
            "dive:2",
            "trips:1*/1,2,1*@3",
            "/town; /shopping",
            "/stopscared off; /dive 2",
            "/none",
        ] {
            assert_eq!(
                parse_errand_setting(Some(value), "BORG_ERRAND"),
                Ok(Some(value.into()))
            );
        }
        for source in ["BORG_ERRAND", "pborg.yaml borg_reborn_errand"] {
            for value in [" ", "; ;", "true", "2", "dvie 2", "/dive 0", "/town; typo"] {
                let error = parse_errand_setting(Some(value), source).unwrap_err();
                assert!(error.contains(source));
                assert!(error.contains(&format!("{value:?}")));
            }
            let error = parse_errand_setting(Some("/level-up 51"), source).unwrap_err();
            assert!(error.contains(source));
            assert!(error.contains("requires a character level from 1 to 50"));
        }
    }

    #[test]
    fn configured_fear_preserves_aliases_and_rejects_invalid_thresholds() {
        for value in [None, Some(""), Some("none")] {
            assert!(parse_stopscared_setting(value, "BORG_STOPSCARED")
                .unwrap()
                .is_none());
        }
        for (value, expected) in [
            ("off", None),
            ("FALSE", None),
            ("no", None),
            ("flee", Some(100)),
            ("true", Some(100)),
            ("on", Some(100)),
            ("YES", Some(100)),
            ("1", Some(1)),
            ("1000", Some(1000)),
        ] {
            let command = parse_stopscared_setting(Some(value), "BORG_STOPSCARED").unwrap();
            assert!(
                matches!(command, Some(borg_rs::shell::Command::StopScared(n)) if n == expected)
            );
        }
        for source in ["BORG_STOPSCARED", "pborg.yaml borg_reborn_stopscared"] {
            for value in [
                "bogus",
                "0",
                "-1",
                "1001",
                "2147483648",
                " ",
                "off; /dive 2",
            ] {
                let error = parse_stopscared_setting(Some(value), source).unwrap_err();
                assert!(error.contains(source) && error.contains(&format!("{value:?}")));
            }
        }
    }

    #[test]
    fn configured_budget_cannot_silently_remove_its_bound() {
        for (value, expected) in [
            (None, 0),
            (Some("none"), 0),
            (Some("0"), 0),
            (Some("1"), 1),
            (Some("+12"), 12),
            (Some("9223372036854775807"), i64::MAX),
        ] {
            assert_eq!(parse_errand_budget(value), Ok(expected));
        }
        for value in [
            "bogus",
            "",
            " ",
            "-1",
            "-9223372036854775808",
            "9223372036854775808",
            "1.5",
            " 12 ",
        ] {
            let error = parse_errand_budget(Some(value)).unwrap_err();
            assert!(error.contains("BORG_ERRAND_BUDGET") && error.contains(&format!("{value:?}")));
        }
    }

    /// An empty observation, for the key sequences of the actions the
    /// strategy chooses (none of them reads the pack).
    fn obs() -> Observation {
        Observation::default()
    }

    /// M1 re-review N2 and R2: on a shop entrance ',' enters the store and
    /// takes no time, and a step can be refused without time (fear), so the
    /// Borg alters a neighbour instead, known floor first.
    #[test]
    fn on_a_shop_it_alters_instead_of_holding() {
        // Shop walls (permanent, 22) all round, floor to the south (direction 2).
        let around = |dx: i32, dy: i32| Some(if (dx, dy) == (0, 1) { feat::FLOOR } else { 22 });
        let a = stand_still(Some(feat::STORE_GENERAL), around);
        assert_ne!(a, Action::Hold, "held on a shop");
        assert_eq!(a, Action::Alter(2));
        assert_eq!(
            Action::Alter(2).keys(&obs()),
            vec![ESCAPE, b'+' as u32, b'2' as u32]
        );
        // Boxed in on the home by unknown grids and walls: still an Alter.
        let a = stand_still(Some(feat::HOME), |dx, _| {
            Some(if dx == 1 { feat::NONE } else { 21 })
        });
        assert_eq!(a, Action::Alter(3));
        assert_eq!(stand_still(Some(feat::FLOOR), around), Action::Hold);
    }

    #[test]
    fn time_passing_resets_the_count() {
        let mut mem = Memory::default();
        mem.record(5, Action::Walk(2));
        mem.record(5, Action::Walk(2));
        assert_eq!(mem.tries_at(5), 2);
        assert_eq!(mem.tries_at(6), 0);
        mem.record(6, Action::Walk(2));
        assert_eq!(mem.same_turn, 0);
    }

    #[test]
    fn free_and_level_actions_have_separate_budgets() {
        let mut mem = Memory::default();
        mem.record(5, Action::Walk(2));
        assert_eq!(mem.tries_at(5), 1);
        mem.record(5, Action::Buy { ware: 0 });
        assert_eq!(mem.tries_at(5), 1);
        mem.record(5, Action::Sell { item: 0, qty: 1 });
        assert_eq!(mem.tries_at(5), 1);
        mem.record(5, Action::Descend);
        assert_eq!(mem.tries_at(5), 2, "stairs count as no-time actions");
        assert_eq!(mem.level_changes_at(5), 1);
        mem.record(5, Action::Ascend);
        assert_eq!(mem.tries_at(5), 3);
        assert_eq!(mem.level_changes_at(5), 2);
        assert_eq!(mem.level_changes_at(6), 0, "a new turn resets the cap");
    }

    #[test]
    fn shops_and_walls_are_not_walkable_but_unknown_grids_are() {
        assert!(walkable(Some(feat::NONE)));
        assert!(walkable(Some(feat::CLOSED)));
        assert!(!walkable(Some(7)), "a shop entrance");
        assert!(!walkable(Some(21)), "granite");
        assert!(!walkable(None), "outside the level");
    }

    #[test]
    fn keys_open_with_escape() {
        assert_eq!(Action::Descend.keys(&obs()), vec![ESCAPE, b'>' as u32]);
        assert_eq!(Action::Walk(3).keys(&obs()), vec![ESCAPE, b'3' as u32]);
        assert_eq!(
            Action::Rest(Params::default().rest_turns).keys(&obs()),
            vec![ESCAPE, b'R' as u32, b'7' as u32, b'5' as u32, KC_ENTER],
            "the default rest is the C Borg's R75"
        );
    }

    #[test]
    fn the_rng_is_reproducible() {
        let (mut a, mut b) = (Rng::new(), Rng::new());
        let xs: Vec<u64> = (0..5).map(|_| a.next_u64()).collect();
        let ys: Vec<u64> = (0..5).map(|_| b.next_u64()).collect();
        assert_eq!(xs, ys);
        assert_ne!(xs[0], xs[1]);
    }
}
