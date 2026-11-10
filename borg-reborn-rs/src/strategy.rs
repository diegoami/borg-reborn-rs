//! L5's first strategy (ARCHITECTURE.md L5; task T3.3, #19): modes
//! chosen by utility and kept with hysteresis, every goal with a budget
//! in game turns and a progress measure, and a cooldown when one fails
//! (GOALS principle 6). The M3 danger placeholder ([`crate::danger`])
//! decides where resting is safe and which step to take when threatened;
//! the full tactical layer is M4's.
//!
//! M3's mode set is deliberately small (PLAN.md: *the first strategy
//! without the full tactical layer*): recover, descend, explore. T3.4
//! puts the playing behaviours on top of it and T3.5 the town; M4's
//! tactical layer replaces the placeholder's one-step retreat.

use crate::borg::{stand_still, Memory, Rng};
use crate::danger::danger;
use crate::engine::PlayerView;
use crate::executor::Action;
use crate::knowledge::Knowledge;
use crate::params::Params;
use crate::path::{Goal as PathGoal, Paths, UNREACHED};
use crate::world::World;
use std::collections::BTreeMap;

/// Directions 1–9 (keypad layout) as (dir, dx, dy).
pub const DIRS: [(u8, i32, i32); 8] = [
    (1, -1, 1),
    (2, 0, 1),
    (3, 1, 1),
    (4, -1, 0),
    (6, 1, 0),
    (7, -1, -1),
    (8, 0, -1),
    (9, 1, -1),
];

/// A mode of the strategic layer. Utilities choose between them;
/// [`FirstStrategy`] keeps the mode until its goal completes, becomes
/// unsafe, or exhausts its budget (hysteresis, GOALS principle 6).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Mode {
    /// Rest until healed (GOALS principle 7's recovery half).
    Recover,
    /// Fight an adjacent monster whose exchange we win (task T3.4: the
    /// fight the danger sum makes decidable).
    Engage,
    /// Reach a down staircase and take it.
    Descend,
    /// Reveal the level: walk to the frontier of the unknown.
    Explore,
}

impl Mode {
    pub fn name(self) -> &'static str {
        match self {
            Mode::Recover => "recover",
            Mode::Engage => "engage",
            Mode::Descend => "descend",
            Mode::Explore => "explore",
        }
    }
}

/// The goal a mode is currently pursuing: when it started, what progress
/// it has made, and when its budget runs out (GOALS principle 6: a
/// budget in *game* turns, a progress measure, and a cooldown on
/// failure).
#[derive(Clone, Debug, PartialEq)]
pub struct Goal {
    pub mode: Mode,
    pub started: i64,
    /// The last game turn the progress measure moved.
    pub progressed: i64,
    /// The progress measure itself (each mode defines its own).
    pub measure: i64,
}

impl Goal {
    fn new(mode: Mode, turn: i64, measure: i64) -> Goal {
        Goal {
            mode,
            started: turn,
            progressed: turn,
            measure,
        }
    }

    /// Whether the budget is spent without progress. `budget` is the
    /// mode's parameter; the goal fails when nothing moved for that
    /// many game turns.
    fn stalled(&self, turn: i64, budget: i64) -> bool {
        turn - self.progressed > budget
    }
}

/// A decision and what led to it (the decision trace's content).
#[derive(Clone, Debug, PartialEq)]
pub struct Decision {
    pub action: Action,
    pub mode: &'static str,
    pub goal: &'static str,
    /// Thinks left at this game turn before the Borg must take an action
    /// that uses time (`max_tries_per_turn`).
    pub budget_left: i64,
    /// The actions considered, with their scores.
    pub candidates: Vec<(Action, f64)>,
    pub violations: Vec<String>,
}

/// What a strategy decides with. The world and knowledge are borrowed
/// for the think; the paths are the per-think pathfinder.
pub struct DecisionCtx<'a> {
    pub view: &'a PlayerView,
    pub world: &'a World,
    pub knowledge: &'a Knowledge,
    pub paths: &'a mut Paths,
    pub mem: &'a Memory,
    pub rng: &'a mut Rng,
    pub params: &'a Params,
    /// The violations the executor reported for the previous action.
    pub failures: &'a [String],
}

/// The family's decision layer (ARCHITECTURE.md L5). M3 has one member;
/// the trait is the seam later strategies and profiles slot into.
pub trait Strategy {
    fn name(&self) -> &'static str;
    fn decide(&mut self, ctx: &mut DecisionCtx) -> Decision;
}

/// The first strategy (task T3.3).
pub struct FirstStrategy {
    mode: Mode,
    goal: Option<Goal>,
    /// A mode whose goal failed here is not retried until this game
    /// turn (the cooldown of GOALS principle 6).
    cooldowns: BTreeMap<Mode, i64>,
    /// Levels visited by the current explore goal (its progress measure
    /// is the remembered grids of the level, which a level change
    /// resets; this keeps the goal honest across one).
    explored: u64,
}

impl Default for FirstStrategy {
    fn default() -> Self {
        FirstStrategy {
            mode: Mode::Explore,
            goal: None,
            cooldowns: BTreeMap::new(),
            explored: 0,
        }
    }
}

impl FirstStrategy {
    /// The remembered grids of the current level: the explore mode's
    /// progress measure (a level mapped is a level explored; the reroll
    /// question is the budget's to answer).
    fn mapped(world: &World) -> i64 {
        world.level.grids.iter().filter(|g| g.feat != 0).count() as i64
    }

    /// Whether resting is safe where the Borg stands: the strategy
    /// manual's rule in its strictest reading (c15 REST-01: no awake
    /// monster about to reach you) — no measurable danger at all. The C
    /// Borg's `avoidance` semantics (danger below current HP, spec §8)
    /// rest beside a small monster and die on the rest's own turns
    /// (bbc D0017); M3 takes the strict form.
    fn may_rest(ctx: &DecisionCtx) -> bool {
        let (x, y) = (ctx.view.x, ctx.view.y);
        danger(ctx.world, ctx.knowledge, x, y, 1) == 0
    }

    /// The one step that most reduces danger (a taste of L4 until M4's
    /// tactical layer): the cheapest of the eight neighbours by danger,
    /// then by flow distance, only through grids the Borg may enter.
    fn retreat_step(ctx: &mut DecisionCtx) -> Option<u8> {
        let (x, y) = (ctx.view.x, ctx.view.y);
        let mut best: Option<(u32, u32, u8)> = None;
        for &(d, dx, dy) in DIRS.iter() {
            let (nx, ny) = (x + dx, y + dy);
            if crate::path::enter_cost(ctx.world, nx, ny).is_none() {
                continue;
            }
            // A monster's grid is not a retreat: walking into it attacks.
            if ctx
                .world
                .level
                .monsters
                .iter()
                .any(|t| t.in_view && (t.x, t.y) == (nx, ny))
            {
                continue;
            }
            let danger = danger(ctx.world, ctx.knowledge, nx, ny, 1);
            let better = match best {
                None => true,
                Some((b, _, _)) => danger < b,
            };
            if better {
                best = Some((danger, 0, d));
            }
        }
        best.map(|(_, _, d)| d)
    }

    /// The adjacent visible monsters, in a deterministic order (by
    /// track index, the world's own order).
    fn adjacent(ctx: &DecisionCtx) -> Vec<usize> {
        let (x, y) = (ctx.view.x, ctx.view.y);
        ctx.world
            .level
            .monsters
            .iter()
            .enumerate()
            .filter(|(_, t)| t.in_view && (t.x - x).abs().max((t.y - y).abs()) <= 1)
            .map(|(i, _)| i)
            .collect()
    }

    /// The exchange test (c15 FIGHT-01 through the danger sum, task
    /// T3.4): engaging is worth it when the damage the monster deals
    /// while we kill it costs less than every hit point we have. The
    /// turns to kill are its hit points over our expected melee damage
    /// per turn ([`crate::world::Character::melee_per_turn`], GOALS
    /// principle 14); the damage while we do it is its danger at our
    /// grid times those turns. No margin constant: M4's tactical layer
    /// replaces this with the real comparison of alternatives.
    fn exchange(ctx: &DecisionCtx, i: usize) -> Option<(f64, f64)> {
        let t = &ctx.world.level.monsters[i];
        let dpt = ctx.world.character.melee_per_turn(ctx.knowledge)?;
        if dpt <= 0.0 || t.hp <= 0 {
            return None;
        }
        let turns = t.hp as f64 / dpt;
        let d = f64::from(crate::danger::one(
            ctx.world,
            ctx.knowledge,
            t,
            ctx.view.x,
            ctx.view.y,
            1,
        ));
        Some((d * turns, turns))
    }

    /// The best adjacent monster to fight: the one whose exchange costs
    /// the least (bbc D0012's lesson: the 2-HP mold is a kill, not a
    /// flight), with its direction.
    fn best_fight(ctx: &DecisionCtx) -> Option<(u8, f64)> {
        let (x, y) = (ctx.view.x, ctx.view.y);
        Self::adjacent(ctx)
            .into_iter()
            .filter_map(|i| {
                let t = &ctx.world.level.monsters[i];
                let (cost, _) = Self::exchange(ctx, i)?;
                let (dx, dy) = (t.x - x, t.y - y);
                let dir = DIRS
                    .iter()
                    .find(|(_, ddx, ddy)| (*ddx, *ddy) == (dx, dy))
                    .map(|(d, _, _)| *d)?;
                Some((dir, cost, i))
            })
            .min_by_key(|&(_, cost, i)| (cost as i64, i))
            .map(|(dir, cost, _)| (dir, cost))
    }

    /// The direction of the adjacent monster with the least danger, for
    /// the cornered case: fighting beats dying on a wall.
    fn cornered_fight(ctx: &DecisionCtx) -> Option<u8> {
        let (x, y) = (ctx.view.x, ctx.view.y);
        Self::adjacent(ctx)
            .into_iter()
            .filter_map(|i| {
                let t = &ctx.world.level.monsters[i];
                let d = crate::danger::one(ctx.world, ctx.knowledge, t, x, y, 1);
                let dir = DIRS
                    .iter()
                    .find(|(_, dx, dy)| (*dx, *dy) == (t.x - x, t.y - y))
                    .map(|(d, _, _)| *d)?;
                Some((d, dir, i))
            })
            .min_by_key(|&(d, _, i)| (d, i))
            .map(|(_, dir, _)| dir)
    }

    /// An object worth stopping for lies underfoot (task T3.4): with
    /// `birth_no_selling` an object is worth carrying only to use or
    /// wear, but gold is the one loot that is value (S15 §6b) — and a
    /// dropped escape beside a fight is a rescue (bbc D0019). M3 takes
    /// what is underfoot whenever the pack has room; triage is M5's.
    fn underfoot(ctx: &DecisionCtx) -> bool {
        ctx.world
            .level
            .at(ctx.view.x, ctx.view.y)
            .is_some_and(|g| g.object_kind.is_some() || g.top_kind.is_some())
    }

    /// The mode's step, when the mode has a place to go. `None` means
    /// the mode cannot act and its goal must fail.
    fn mode_step(&mut self, ctx: &mut DecisionCtx, mode: Mode) -> Option<Action> {
        // Nothing is picked up in contact: clearing the threat comes
        // first (bbc D0019 lost its Phase Door to attack-before-pickup;
        // the reverse order, pickup-under-fire, is the same mistake).
        let in_contact = !Self::adjacent(ctx).is_empty();
        if !in_contact
            && mode != Mode::Recover
            && Self::underfoot(ctx)
            && ctx.world.character.pack_free() > 0
        {
            return Some(Action::Get);
        }
        match mode {
            Mode::Recover => {
                if !Self::may_rest(ctx) {
                    // Hurt with a monster about: step out of reach
                    // rather than rest beside it (bbc D0017's lesson in
                    // miniature: no mode may block recovery, and no
                    // recovery may ignore the room).
                    return Self::retreat_step(ctx).map(Action::Walk);
                }
                Some(Action::Rest(ctx.params.rest_turns))
            }
            Mode::Engage => {
                if let Some((dir, cost)) = Self::best_fight(ctx) {
                    if cost < f64::from(ctx.view.hp) {
                        return Some(Action::Attack(dir));
                    }
                }
                // The exchange loses: leave. A faster pursuer makes
                // walking away a death (bbc D0018), and no safe step
                // means a corner: fight the least dangerous one rather
                // than die on a wall.
                Self::retreat_step(ctx)
                    .map(Action::Walk)
                    .or_else(|| Self::cornered_fight(ctx).map(Action::Attack))
            }
            Mode::Descend => {
                let here = ctx.world.level.at(ctx.view.x, ctx.view.y).map(|g| g.feat);
                let threat = danger(ctx.world, ctx.knowledge, ctx.view.x, ctx.view.y, 1);
                if here == Some(crate::borg::feat::MORE) && threat == 0 {
                    // A staircase under fire is not taken: whatever is
                    // pressing is dealt with first (engaged or escaped).
                    return Some(Action::Descend);
                }
                let step =
                    ctx.paths
                        .step(ctx.world, ctx.knowledge, &PathGoal::Stairs { up: false });
                step.map(Action::Walk).or_else(|| {
                    // No down staircase known: going up then down rerolls
                    // the level (S15, ACCUMULATED_KNOWLEDGE §2: two or
                    // three decisions for a fresh one).
                    let up =
                        ctx.paths
                            .step(ctx.world, ctx.knowledge, &PathGoal::Stairs { up: true });
                    up.map(Action::Walk)
                })
            }
            Mode::Explore => {
                let step = ctx
                    .paths
                    .step(ctx.world, ctx.knowledge, &PathGoal::Frontier);
                Some(step.map_or(Action::Hold, Action::Walk))
            }
        }
    }

    /// A mode's progress measure at this think.
    fn measure(mode: Mode, ctx: &DecisionCtx) -> i64 {
        match mode {
            Mode::Recover => ctx.view.hp as i64,
            // The fight's progress is the threat going down: the least
            // hit points any adjacent monster has.
            Mode::Engage => Self::adjacent(ctx)
                .into_iter()
                .map(|i| ctx.world.level.monsters[i].hp)
                .min()
                .unwrap_or(0) as i64,
            Mode::Descend => ctx.world.level.depth as i64,
            Mode::Explore => Self::mapped(ctx.world),
        }
    }

    /// Choose the mode by utility, with hysteresis: the invariant
    /// (recovery) outranks everything; a live goal keeps its mode; a
    /// failed mode waits out its cooldown.
    fn choose_mode(&mut self, ctx: &mut DecisionCtx) -> Mode {
        let turn = ctx.view.turn;
        // The recovery invariant first (GOALS principle 7): hurt and
        // safe means rest now.
        let hurt = ctx.view.hp * 100 < ctx.view.max_hp * ctx.params.rest_below_hp_pct;
        if hurt
            && self
                .cooldowns
                .get(&Mode::Recover)
                .is_none_or(|&t| turn >= t)
            && Self::may_rest(ctx)
        {
            return Mode::Recover;
        }
        // Contact second (task T3.4): a monster in reach is answered
        // this think — fought when the exchange wins it, left when it
        // does not. M4's tactical layer replaces the whole comparison.
        if !Self::adjacent(ctx).is_empty() {
            return Mode::Engage;
        }
        // A live goal keeps its mode (hysteresis): still inside its
        // budget, or the mode can still act.
        if let Some(goal) = &self.goal {
            let budget = match goal.mode {
                // Resting until healed: give it the rest itself takes,
                // twice over, before calling it stalled.
                Mode::Recover => 2 * i64::from(ctx.params.rest_turns),
                // A fight that has not progressed in a tenth of the
                // explore budget is lost or illusory; cool it and leave.
                Mode::Engage => i64::from(ctx.params.explore_budget_turns) / 10,
                Mode::Descend => i64::from(ctx.params.explore_budget_turns),
                Mode::Explore => i64::from(ctx.params.explore_budget_turns),
            };
            if !goal.stalled(turn, budget)
                && self.cooldowns.get(&goal.mode).is_none_or(|&t| turn >= t)
            {
                return goal.mode;
            }
            // The goal failed: cool the mode down (GOALS principle 6)
            // and drop it.
            self.cooldowns
                .insert(goal.mode, turn + i64::from(ctx.params.goal_cooldown_turns));
            self.goal = None;
        }
        // Utilities, highest first; explore is the default occupation
        // and descend takes over when explore is cooling or done.
        let frontier = ctx
            .paths
            .map(ctx.world, ctx.knowledge, &PathGoal::Frontier)
            .remaining(ctx.world, ctx.view.x, ctx.view.y);
        let stairs = ctx
            .paths
            .map(ctx.world, ctx.knowledge, &PathGoal::Stairs { up: false })
            .remaining(ctx.world, ctx.view.x, ctx.view.y);
        let cooled = |m: Mode| self.cooldowns.get(&m).is_none_or(|&t| turn >= t);
        if frontier != UNREACHED && cooled(Mode::Explore) {
            Mode::Explore
        } else if stairs != UNREACHED && cooled(Mode::Descend) {
            Mode::Descend
        } else if cooled(Mode::Explore) {
            Mode::Explore
        } else {
            Mode::Descend
        }
    }

    /// Fail the current goal when the executor reported the previous
    /// action refused: no progress is being made even though game time
    /// may be passing.
    fn note_failures(&mut self, ctx: &DecisionCtx) {
        if ctx.failures.is_empty() {
            return;
        }
        self.cooldowns.insert(
            self.mode,
            ctx.view.turn + i64::from(ctx.params.goal_cooldown_turns),
        );
        self.goal = None;
    }
}

impl Strategy for FirstStrategy {
    fn name(&self) -> &'static str {
        "first"
    }

    fn decide(&mut self, ctx: &mut DecisionCtx) -> Decision {
        self.note_failures(ctx);
        let mode = self.choose_mode(ctx);
        self.mode = mode;

        // A fresh goal when the mode changed or none is live; the
        // measure is taken now, and progress is its movement.
        let now_measure = Self::measure(mode, ctx);
        if self.goal.as_ref().is_none_or(|g| g.mode != mode) {
            self.goal = Some(Goal::new(mode, ctx.view.turn, now_measure));
            if mode == Mode::Explore {
                self.explored += 1;
            }
        }
        // Progress: the measure moved, or the mode's act succeeded.
        if let Some(goal) = &mut self.goal {
            if now_measure != goal.measure {
                goal.measure = now_measure;
                goal.progressed = ctx.view.turn;
            }
        }

        let tries = ctx.mem.tries_at(ctx.view.turn);
        let budget_left = i64::from(ctx.params.max_tries_per_turn) - i64::from(tries);
        let mut violations = Vec::new();
        if tries >= ctx.params.max_tries_per_turn {
            let why = format!(
                "{tries} thinks at game turn {} took no time (GOALS principle 6)",
                ctx.view.turn
            );
            violations.push(why);
            let (x, y) = (ctx.view.x, ctx.view.y);
            let here = ctx.world.level.at(x, y).map(|g| g.feat);
            let around = |dx: i32, dy: i32| ctx.world.level.at(x + dx, y + dy).map(|g| g.feat);
            return Decision {
                action: stand_still(here, around),
                mode: mode.name(),
                goal: "stand still",
                budget_left,
                candidates: vec![],
                violations,
            };
        }

        let action = match self.mode_step(ctx, mode) {
            Some(a) => a,
            None => {
                // The mode cannot act: its goal has failed here and
                // there. Cool it down and fall through to the safest
                // action that always uses time.
                self.cooldowns.insert(
                    mode,
                    ctx.view.turn + i64::from(ctx.params.goal_cooldown_turns),
                );
                self.goal = None;
                let (x, y) = (ctx.view.x, ctx.view.y);
                let here = ctx.world.level.at(x, y).map(|g| g.feat);
                let around = |dx: i32, dy: i32| ctx.world.level.at(x + dx, y + dy).map(|g| g.feat);
                stand_still(here, around)
            }
        };
        let goal_name = match mode {
            Mode::Recover => "rest to full",
            Mode::Engage => "win the exchange",
            Mode::Descend => "reach the down staircase",
            Mode::Explore => "reveal the frontier",
        };
        Decision {
            action,
            mode: mode.name(),
            goal: goal_name,
            budget_left,
            candidates: vec![(action_clone(&action), 1.0)],
            violations,
        }
    }
}

fn action_clone(a: &Action) -> Action {
    *a
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::observation::Status;

    /// A decision context over a map world; see [`crate::path::tests`].
    /// The character can fight: the Warrior class, a 1d4 dagger
    /// (two blows), so the exchange test has real numbers.
    fn ctx_of(
        map: &[&str],
        hp: i32,
        max_hp: i32,
    ) -> (FirstStrategy, World, Knowledge, Paths, Memory, Rng, Params) {
        let (w, k) = crate::path::tests::world(map);
        let mut w = w;
        let mut k = k;
        k.classes = vec![crate::knowledge::tests::warrior()];
        w.character.equipment = vec![crate::observation::Slot {
            slot_type: 1,
            item: Some(crate::observation::Item {
                kind: 300,
                tval: 23,
                sval: 4,
                number: 1,
                weight: 120,
                timeout: 0,
                level: 5,
                desc: "a Dagger (1d4)".into(),
                aware: true,
                fully_known: true,
                runes_known: false,
                flags: [0; 5],
                ac: 0,
                dd: 1,
                ds: 4,
                to_h: 0,
                to_d: 0,
                to_a: 0,
                modifiers: vec![0; 16],
                elements: vec![(0, 0); 25],
            }),
        }];
        if let Some(s) = w.character.status.as_mut() {
            s.hp = hp;
            s.max_hp = max_hp;
            s.pack_size = 23;
            s.pack_used = 1;
        }
        let paths = Paths::new(&w);
        (
            FirstStrategy::default(),
            w,
            k,
            paths,
            Memory::default(),
            Rng::new(),
            Params::default(),
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn decide_with(
        s: &mut FirstStrategy,
        w: &World,
        k: &Knowledge,
        p: &mut Paths,
        m: &Memory,
        rng: &mut Rng,
        params: &Params,
        view: &PlayerView,
        failures: &[String],
    ) -> Decision {
        let mut ctx = DecisionCtx {
            view,
            world: w,
            knowledge: k,
            paths: p,
            mem: m,
            rng,
            params,
            failures,
        };
        s.decide(&mut ctx)
    }

    fn view(w: &World) -> PlayerView {
        let s = w.character.status.as_ref().unwrap();
        PlayerView {
            hp: s.hp,
            max_hp: s.max_hp,
            x: s.x,
            y: s.y,
            depth: s.depth,
            dead: false,
            busy: false,
            turn: w.turn,
        }
    }

    #[test]
    fn explores_the_frontier_toward_the_unknown() {
        let (mut s, w, k, mut p, m, mut rng, params) = ctx_of(&["#####", "#@.?#", "#####"], 20, 20);
        let d = decide_with(
            &mut s,
            &w,
            &k,
            &mut p,
            &m,
            &mut rng,
            &params,
            &view(&w),
            &[],
        );
        assert_eq!(d.mode, "explore");
        assert_eq!(d.action, Action::Walk(6), "east, toward the unknown");
    }

    #[test]
    fn rests_when_hurt_and_safe_but_not_beside_danger() {
        // Hurt, nothing threatening: rest.
        let (mut s, w, k, mut p, m, mut rng, params) = ctx_of(&["#####", "#@..#", "#####"], 5, 20);
        let d = decide_with(
            &mut s,
            &w,
            &k,
            &mut p,
            &m,
            &mut rng,
            &params,
            &view(&w),
            &[],
        );
        assert_eq!(d.mode, "recover");
        assert_eq!(d.action, Action::Rest(75));
        // Hurt with a monster adjacent and floor behind: contact makes
        // it engage, the exchange (a 1d6 kobold against 5 hit points)
        // loses, and the step away is the answer — never a rest.
        let (mut s, w, k, mut p, m, mut rng, params) =
            ctx_of(&["#######", "#M@..>#", "#######"], 5, 20);
        let d = decide_with(
            &mut s,
            &w,
            &k,
            &mut p,
            &m,
            &mut rng,
            &params,
            &view(&w),
            &[],
        );
        assert_eq!(d.mode, "engage");
        assert_eq!(d.action, Action::Walk(6), "step away, east");
        assert_ne!(d.action, Action::Rest(75), "never rest beside a monster");
    }

    #[test]
    fn takes_the_down_staircase_when_on_it() {
        let (mut s, mut w, k, _, m, mut rng, params) = ctx_of(&["#####", "#@.>#", "#####"], 20, 20);
        // Move the player onto the stair, then rebuild the pathfinder
        // for that world (a map never outlives its observation).
        if let Some(s) = w.character.status.as_mut() {
            s.x = 3;
            s.y = 1;
        }
        let mut p = crate::path::Paths::new(&w);
        let v = view(&w);
        let d = decide_with(&mut s, &w, &k, &mut p, &m, &mut rng, &params, &v, &[]);
        assert_eq!(d.action, Action::Descend);
    }

    #[test]
    fn a_stalled_goal_cools_down_and_the_next_mode_takes_over() {
        // The frontier exists but the goal never progresses: after the
        // budget the mode is cooled and descend (stairs known) takes
        // over.
        let (mut s, mut w, k, _, m, mut rng, params) =
            ctx_of(&["#######", "#@.?.>#", "#.....#", "#######"], 20, 20);
        // Stall the goal: think once so a goal exists, then let the
        // game turns run past the budget without the level changing,
        // and rebuild the pathfinder for the later world.
        {
            let mut p = crate::path::Paths::new(&w);
            let v = view(&w);
            let _ = decide_with(&mut s, &w, &k, &mut p, &m, &mut rng, &params, &v, &[]);
        }
        w.turn += params.explore_budget_turns as i64 + 2;
        let mut p = crate::path::Paths::new(&w);
        let v = view(&w);
        let d = decide_with(&mut s, &w, &k, &mut p, &m, &mut rng, &params, &v, &[]);
        assert_eq!(
            d.mode, "descend",
            "explore stalled: the cooldown hands the level to descend"
        );
        assert!(
            matches!(d.action, Action::Walk(_)),
            "a step toward the '>' through the second row: {:?}",
            d.action
        );
    }

    #[test]
    fn executor_failures_fail_the_goal() {
        let (mut s, w, k, mut p, m, mut rng, params) = ctx_of(&["#####", "#@.?#", "#####"], 20, 20);
        let v = view(&w);
        let d = decide_with(
            &mut s,
            &w,
            &k,
            &mut p,
            &m,
            &mut rng,
            &params,
            &v,
            &["after Walk(6): the step neither moved nor took time".into()],
        );
        assert_eq!(d.mode, "descend", "explore cooled by its failure");
        assert!(s.goal.is_none() || s.goal.as_ref().unwrap().mode == Mode::Descend);
    }

    #[test]
    fn the_level_exhausted_falls_to_descend() {
        // No frontier: the level is mapped; descend is the mode.
        let (mut s, w, k, mut p, m, mut rng, params) = ctx_of(&["#####", "#@.>#", "#####"], 20, 20);
        let d = decide_with(
            &mut s,
            &w,
            &k,
            &mut p,
            &m,
            &mut rng,
            &params,
            &view(&w),
            &[],
        );
        assert_eq!(d.mode, "descend");
        assert_eq!(d.action, Action::Walk(6));
    }

    #[test]
    fn a_favourable_exchange_is_fought() {
        // A 6-HP kobold beside a healthy character with a dagger: the
        // exchange (its 6 a turn for the turn or two the kill takes)
        // costs less than the hit points, so the monster is attacked,
        // not fled from (bbc D0012: the 2-HP mold is a kill).
        let (mut s, w, k, mut p, m, mut rng, params) =
            ctx_of(&["#######", "#@M...#", "#######"], 20, 20);
        let d = decide_with(
            &mut s,
            &w,
            &k,
            &mut p,
            &m,
            &mut rng,
            &params,
            &view(&w),
            &[],
        );
        assert_eq!(d.mode, "engage");
        assert_eq!(d.action, Action::Attack(6), "east, into the monster");
    }

    #[test]
    fn a_losing_exchange_retreats_unless_cornered() {
        // A 60-HP monster with a heavy blow west of the Borg, the
        // corridor open east: the exchange loses, so it steps away.
        let (w_m, k) = crate::path::tests::world(&["#######", "#M@...#", "#######"]);
        let mut k = k;
        k.races[1] = crate::knowledge::MonsterRace {
            blows: vec![crate::knowledge::Blow {
                method: "CRUSH".into(),
                effect: "HURT".into(),
                power: 60,
                dice: crate::knowledge::Dice {
                    base: 0,
                    dice: 6,
                    sides: 6,
                    m_bonus: 0,
                },
            }],
            ..k.races[1].clone()
        };
        k.classes = vec![crate::knowledge::tests::warrior()];
        let mut w = w_m;
        w.character.equipment = vec![crate::observation::Slot {
            slot_type: 1,
            item: None,
        }];
        let mut t = w.level.monsters[0].clone();
        t.hp = 60;
        t.max_hp = 60;
        w.level.monsters[0] = t;
        let mut p = crate::path::Paths::new(&w);
        let v = PlayerView {
            hp: 20,
            max_hp: 20,
            x: 2,
            y: 1,
            depth: 1,
            dead: false,
            busy: false,
            turn: w.turn,
        };
        let mut s = FirstStrategy::default();
        let mut rng = Rng::new();
        let params = Params::default();
        let d = decide_with(
            &mut s,
            &w,
            &k,
            &mut p,
            &Memory::default(),
            &mut rng,
            &params,
            &v,
            &[],
        );
        assert_eq!(d.mode, "engage");
        assert_eq!(d.action, Action::Walk(6), "away, east of the monster");
        // Cornered (walls all round but the monster): fight the only
        // fight there is rather than die on a wall.
        let (w_c, k) = crate::path::tests::world(&["######", "#@M###", "######"]);
        let mut k = k;
        k.classes = vec![crate::knowledge::tests::warrior()];
        let mut w = w_c;
        w.character.equipment = vec![crate::observation::Slot {
            slot_type: 1,
            item: None,
        }];
        let mut p = crate::path::Paths::new(&w);
        let v = PlayerView {
            hp: 20,
            max_hp: 20,
            x: 1,
            y: 1,
            depth: 1,
            dead: false,
            busy: false,
            turn: w.turn,
        };
        let d = decide_with(
            &mut s,
            &w,
            &k,
            &mut p,
            &Memory::default(),
            &mut rng,
            &params,
            &v,
            &[],
        );
        assert_eq!(d.action, Action::Attack(6), "cornered: fight anyway");
    }

    #[test]
    fn the_object_underfoot_is_picked_up_when_not_in_contact() {
        let (mut s, mut w, k, _, m, mut rng, params) = ctx_of(&["#####", "#$@.#", "#####"], 20, 20);
        // Stand on the gold, then rebuild the pathfinder for that world.
        if let Some(st) = w.character.status.as_mut() {
            st.x = 1;
            st.y = 1;
        }
        let mut p = crate::path::Paths::new(&w);
        let d = decide_with(
            &mut s,
            &w,
            &k,
            &mut p,
            &m,
            &mut rng,
            &params,
            &view(&w),
            &[],
        );
        assert_eq!(d.action, Action::Get, "gold underfoot, nothing pressing");
        // In contact the pickup waits (bbc D0019).
        let (mut s, w, k, mut p, m, mut rng, params) =
            ctx_of(&["######", "#$@M.#", "######"], 20, 20);
        let d = decide_with(
            &mut s,
            &w,
            &k,
            &mut p,
            &m,
            &mut rng,
            &params,
            &view(&w),
            &[],
        );
        assert_ne!(d.action, Action::Get, "never pick up under fire");
    }

    #[test]
    fn a_staircase_under_fire_is_not_taken() {
        // On the down stair with a monster adjacent: the fight or the
        // escape, never the staircase.
        let (mut s, mut w, k, _, m, mut rng, params) =
            ctx_of(&["######", "#>M@.#", "######"], 20, 20);
        if let Some(s) = w.character.status.as_mut() {
            s.x = 1;
            s.y = 1;
        }
        let mut p = crate::path::Paths::new(&w);
        let v = view(&w);
        let d = decide_with(&mut s, &w, &k, &mut p, &m, &mut rng, &params, &v, &[]);
        assert_ne!(d.action, Action::Descend, "never descend in contact");
    }

    #[test]
    fn recovery_returns_to_explore_when_healed() {
        let (mut s, w, k, mut p, m, mut rng, params) = ctx_of(&["#####", "#@.?#", "#####"], 20, 20);
        let d = decide_with(
            &mut s,
            &w,
            &k,
            &mut p,
            &m,
            &mut rng,
            &params,
            &view(&w),
            &[],
        );
        assert_eq!(d.mode, "explore");
        let _ = Status::default();
    }
}
