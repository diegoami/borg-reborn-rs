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
//! game time passed since a refused rest walks instead, so it cannot loop
//! without time passing.

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
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    /// Rest 75 turns ('R' '7' '5' Enter, as the C Borg does).
    Rest,
    /// Take the down staircase underfoot ('>').
    Descend,
    /// Walk one step in a direction 1–9.
    Walk(u8),
}

impl Action {
    /// The complete key sequence, opened with `ESCAPE` (spec §2.6).
    pub fn keys(self) -> Vec<u32> {
        let mut k = vec![ESCAPE];
        match self {
            Action::Rest => k.extend([b'R' as u32, b'7' as u32, b'5' as u32, KC_ENTER]),
            Action::Descend => k.push(b'>' as u32),
            Action::Walk(d) => k.push((b'0' + d) as u32),
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
}

/// Choose an action. `here` is the known feature underfoot, `around(dx, dy)`
/// the known feature of a neighbour. Pure: no engine access.
pub fn decide(
    view: &PlayerView,
    here: Option<u32>,
    around: impl Fn(i32, i32) -> Option<u32>,
    mem: &Memory,
    rng: &mut Rng,
) -> Action {
    // A rest that was refused (no game time passed since) is not repeated:
    // the Borg would ask again forever (GOALS principle 6).
    let rest_refused = matches!(mem.last, Some((t, Action::Rest)) if t == view.turn);
    if here == Some(feat::MORE) {
        return Action::Descend;
    }
    if view.hp * 10 < view.max_hp * 7 && !rest_refused {
        return Action::Rest;
    }
    let options: Vec<u8> = DIRS
        .iter()
        .filter(|(_, dx, dy)| walkable(around(*dx, *dy)))
        .map(|(d, _, _)| *d)
        .collect();
    if options.is_empty() {
        // Boxed in by known walls: try any direction (a door or an
        // unknown wall may still give way).
        return Action::Walk(DIRS[rng.below(DIRS.len())].0);
    }
    Action::Walk(options[rng.below(options.len())])
}

struct Borg {
    queue: VecDeque<u32>,
    rng: Rng,
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
    let here = engine::known_feature(view.x, view.y);
    let action = decide(
        &view,
        here,
        |dx, dy| engine::known_feature(view.x + dx, view.y + dy),
        &b.mem,
        &mut b.rng,
    );
    b.mem.last = Some((view.turn, action));
    b.queue.extend(action.keys());
    Key::Code(b.queue.pop_front().unwrap_or(ESCAPE))
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
        let a = decide(
            &view(10, 7),
            Some(feat::FLOOR),
            |_, _| Some(feat::FLOOR),
            &Memory::default(),
            &mut rng,
        );
        assert_eq!(a, Action::Rest);
        let refused = Memory {
            last: Some((7, Action::Rest)),
        };
        let b = decide(
            &view(10, 7),
            Some(feat::FLOOR),
            |_, _| Some(feat::FLOOR),
            &refused,
            &mut rng,
        );
        assert!(
            matches!(b, Action::Walk(_)),
            "a refused rest is not repeated: {b:?}"
        );
        let later = Memory {
            last: Some((6, Action::Rest)),
        };
        assert_eq!(
            decide(
                &view(10, 7),
                Some(feat::FLOOR),
                |_, _| Some(feat::FLOOR),
                &later,
                &mut rng
            ),
            Action::Rest
        );
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
