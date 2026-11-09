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
//! [`MAX_TRIES_PER_TURN`] such thinks it takes an action that always uses
//! game time (see [`stand_still`]), so it cannot loop without time passing.

use crate::engine::{self, Key, PlayerView, ESCAPE, KC_ENTER};
use std::collections::VecDeque;
use std::sync::Mutex;

/// Feature indices of `list-terrain.h` (FEAT_NONE = 0, ...).
mod feat {
    pub const NONE: u32 = 0;
    pub const FLOOR: u32 = 1;
    pub const CLOSED: u32 = 2;
    pub const OPEN: u32 = 3;
    pub const BROKEN: u32 = 4;
    pub const LESS: u32 = 5;
    pub const MORE: u32 = 6;
    pub const STORE_GENERAL: u32 = 7;
    pub const HOME: u32 = 14;
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

/// A decision of the minimal Borg.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Action {
    /// Rest 75 turns ('R' '7' '5' Enter, as the C Borg does).
    Rest,
    /// Take the down staircase underfoot ('>').
    Descend,
    /// Walk one step in a direction 1–9.
    Walk(u8),
    /// Stand still one turn (','): uses game time, except on a shop
    /// entrance, where it enters the store (see [`stand_still`]).
    Hold,
    /// Alter the grid in a direction 1–9 ('+' and the direction): always
    /// uses game time (see [`stand_still`]).
    Alter(u8),
}

impl Action {
    /// The complete key sequence, opened with `ESCAPE` (spec §2.6).
    pub fn keys(self) -> Vec<u32> {
        let mut k = vec![ESCAPE];
        match self {
            Action::Rest => k.extend([b'R' as u32, b'7' as u32, b'5' as u32, KC_ENTER]),
            Action::Descend => k.push(b'>' as u32),
            Action::Walk(d) => k.push((b'0' + d) as u32),
            Action::Hold => k.push(b',' as u32),
            Action::Alter(d) => k.extend([b'+' as u32, (b'0' + d) as u32]),
        }
        k
    }
}

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
    /// it would not start).
    pub same_turn: u32,
}

impl Memory {
    /// Record a think at `turn` that chose `action`.
    pub fn record(&mut self, turn: i64, action: Action) {
        self.same_turn = match self.last {
            Some((t, _)) if t == turn => self.same_turn + 1,
            _ => 0,
        };
        self.last = Some((turn, action));
    }

    /// Thinks already made at `turn` (0 if game time has passed since).
    pub fn tries_at(&self, turn: i64) -> u32 {
        match self.last {
            Some((t, _)) if t == turn => self.same_turn + 1,
            _ => 0,
        }
    }
}

/// After this many thinks without game time passing, the Borg takes the
/// action that uses time where it stands, [`stand_still`] (GOALS principle
/// 6: no loop without time).
pub const MAX_TRIES_PER_TURN: u32 = 2;

/// Choose an action. `here` is the known feature underfoot, `around(dx, dy)`
/// the known feature of a neighbour. Pure: no engine access.
pub fn decide(
    view: &PlayerView,
    here: Option<u32>,
    around: impl Fn(i32, i32) -> Option<u32>,
    mem: &Memory,
    rng: &mut Rng,
) -> Action {
    // No loop without game time (GOALS principle 6): an earlier action at
    // this same turn took no time, so only time-taking actions remain, and
    // after a few tries the Borg stands still.
    let tries = mem.tries_at(view.turn);
    if tries >= MAX_TRIES_PER_TURN {
        return stand_still(here, around);
    }
    if here == Some(feat::MORE) {
        return Action::Descend;
    }
    if view.hp * 10 < view.max_hp * 7 && tries == 0 {
        return Action::Rest;
    }
    let options: Vec<u8> = DIRS
        .iter()
        .filter(|(_, dx, dy)| walkable(around(*dx, *dy)))
        .map(|(d, _, _)| *d)
        .collect();
    if options.is_empty() {
        // Boxed in by known walls: a walk would take no time.
        return stand_still(here, around);
    }
    Action::Walk(options[rng.below(options.len())])
}

struct Borg {
    queue: VecDeque<u32>,
    rng: Rng,
    /// The quick-generator value engine reads draw from (`with_borg_rng`).
    engine_rng: u32,
    mem: Memory,
}

static BORG: Mutex<Option<Borg>> = Mutex::new(None);

/// `^Z`: read the command key; `z` starts the Borg (as the C Borg's command
/// menu does, so borgh's start script and the GUI work unchanged).
pub fn command() {
    if engine::read_command() != Some(b'z') {
        return;
    }
    // Settings the Borg relies on (spec §1.2: the C Borg sets options too):
    // no -more- prompts, and walking into a door opens it.
    engine::set_option(c"auto_more", true);
    engine::set_option(c"easy_alter", true);
    let mut b = BORG.lock().unwrap_or_else(|e| e.into_inner());
    b.get_or_insert_with(|| Borg {
        queue: VecDeque::new(),
        rng: Rng::new(),
        engine_rng: Rng::SEED as u32,
        mem: Memory::default(),
    })
    .queue
    .clear();
    drop(b);
    engine::set_hook(true);
}

/// The engine wants a key (`inkey_hack`).
pub fn next_key(flush: bool) -> Key {
    let mut guard = BORG.lock().unwrap_or_else(|e| e.into_inner());
    let Some(b) = guard.as_mut() else {
        return Key::Code(ESCAPE);
    };
    if flush {
        b.queue.clear();
    }
    if let Some(k) = b.queue.pop_front() {
        return Key::Code(k);
    }
    // Any real key stops the Borg (spec §2.2 step 8; the GUI's "any key
    // stops it").
    if engine::take_user_key().is_some() {
        engine::set_hook(false);
        return Key::Code(ESCAPE);
    }
    let Some(view) = engine::player_view() else {
        return Key::Code(ESCAPE);
    };
    if view.dead {
        engine::set_hook(false);
        return Key::Code(ESCAPE);
    }
    if view.busy {
        return Key::None;
    }
    engine::report_decision(b.rng.state());
    let action = think(b, &view);
    b.mem.record(view.turn, action);
    b.queue.extend(action.keys());
    Key::Code(b.queue.pop_front().unwrap_or(ESCAPE))
}

/// Perceive and decide. Every engine read runs with the Borg's own RNG.
fn think(b: &mut Borg, view: &PlayerView) -> Action {
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
            (here, around)
        })
    };
    #[cfg(feature = "test-hallucinate")]
    let (here, around) = engine::while_hallucinating(|| perceive(b));
    #[cfg(not(feature = "test-hallucinate"))]
    let (here, around) = perceive(b);
    #[cfg(feature = "test-hallucinate")]
    assert_eq!(
        engine::game_rng(),
        before,
        "perception moved the game's RNG"
    );
    let lookup = |dx: i32, dy: i32| {
        DIRS.iter()
            .position(|(_, x, y)| (*x, *y) == (dx, dy))
            .and_then(|i| around[i])
    };
    decide(view, here, lookup, &b.mem, &mut b.rng)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn view(hp: i32, turn: i64) -> PlayerView {
        PlayerView {
            hp,
            max_hp: 20,
            x: 5,
            y: 5,
            depth: 1,
            dead: false,
            busy: false,
            turn,
        }
    }

    #[test]
    fn descends_when_on_a_down_staircase() {
        let a = decide(
            &view(20, 1),
            Some(feat::MORE),
            |_, _| Some(feat::FLOOR),
            &Memory::default(),
            &mut Rng::new(),
        );
        assert_eq!(a, Action::Descend);
    }

    #[test]
    fn rests_when_hurt_but_not_again_without_time_passing() {
        let mut rng = Rng::new();
        let floor = |_, _| Some(feat::FLOOR);
        let mut mem = Memory::default();
        let a = decide(&view(10, 7), Some(feat::FLOOR), floor, &mem, &mut rng);
        assert_eq!(a, Action::Rest);
        mem.record(7, a);
        let b = decide(&view(10, 7), Some(feat::FLOOR), floor, &mem, &mut rng);
        assert!(
            matches!(b, Action::Walk(_)),
            "a refused rest is not repeated: {b:?}"
        );
        mem.record(7, b);
        let mut later = Memory::default();
        later.record(6, Action::Rest);
        assert_eq!(
            decide(&view(10, 7), Some(feat::FLOOR), floor, &later, &mut rng),
            Action::Rest
        );
    }

    /// Review finding on #39: Rest -> Walk -> Rest at one unchanged turn, and
    /// walks into known walls, were possible. Simulate a game that refuses
    /// every action: the Borg must reach Hold (which always takes time)
    /// within MAX_TRIES_PER_TURN thinks and never rest twice at one turn.
    #[test]
    fn no_loop_without_game_time() {
        let mut rng = Rng::new();
        let mut mem = Memory::default();
        let mut rests = 0;
        let mut held = false;
        for _ in 0..10 {
            let a = decide(
                &view(10, 42),
                Some(feat::FLOOR),
                |_, _| Some(feat::FLOOR),
                &mem,
                &mut rng,
            );
            rests += (a == Action::Rest) as u32;
            mem.record(42, a);
            if a == Action::Hold {
                held = true;
                break;
            }
        }
        assert!(held, "no Hold after repeated refusals");
        assert!(mem.same_turn < MAX_TRIES_PER_TURN + 1);
        assert!(rests <= 1, "rested {rests} times at one turn");
    }

    #[test]
    fn boxed_in_by_known_walls_holds() {
        let a = decide(
            &view(20, 1),
            Some(feat::FLOOR),
            |_, _| Some(21),
            &Memory::default(),
            &mut Rng::new(),
        );
        assert_eq!(a, Action::Hold);
        assert_eq!(a.keys(), vec![ESCAPE, b',' as u32]);
    }

    /// M1 re-review N2 and R2: on a shop entrance ',' enters the store and
    /// takes no time, and a step can be refused without time (fear), so the
    /// Borg alters a neighbour instead, known floor first.
    #[test]
    fn on_a_shop_it_alters_instead_of_holding() {
        // Shop walls (permanent, 22) all round, floor to the south (direction 2).
        let around = |dx: i32, dy: i32| Some(if (dx, dy) == (0, 1) { feat::FLOOR } else { 22 });
        let mut mem = Memory::default();
        let mut forced = Vec::new();
        for _ in 0..6 {
            let a = decide(
                &view(20, 9),
                Some(feat::STORE_GENERAL),
                around,
                &mem,
                &mut Rng::new(),
            );
            assert_ne!(a, Action::Hold, "held on a shop");
            if mem.tries_at(9) >= MAX_TRIES_PER_TURN {
                forced.push(a);
            }
            mem.record(9, a);
        }
        assert!(!forced.is_empty());
        assert!(forced.iter().all(|a| *a == Action::Alter(2)), "{forced:?}");
        assert_eq!(
            Action::Alter(2).keys(),
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
    fn walks_only_where_it_may() {
        // Only the grid to the east (direction 6) is floor; the rest are granite (21).
        let around = |dx: i32, dy: i32| Some(if (dx, dy) == (1, 0) { feat::FLOOR } else { 21 });
        for _ in 0..20 {
            assert_eq!(
                decide(
                    &view(20, 1),
                    Some(feat::FLOOR),
                    around,
                    &Memory::default(),
                    &mut Rng::new()
                ),
                Action::Walk(6)
            );
        }
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
        assert_eq!(Action::Descend.keys(), vec![ESCAPE, b'>' as u32]);
        assert_eq!(Action::Walk(3).keys(), vec![ESCAPE, b'3' as u32]);
        assert_eq!(Action::Rest.keys()[0], ESCAPE);
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
