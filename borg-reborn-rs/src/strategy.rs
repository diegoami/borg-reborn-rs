//! L5's first strategy (ARCHITECTURE.md L5; task T3.3, #19): modes
//! chosen by utility and kept with hysteresis, every goal with a budget
//! in game turns and a progress measure, and a cooldown when one fails
//! (GOALS principle 6). The M3 danger placeholder ([`crate::danger`])
//! decides where resting is safe and which step to take when threatened;
//! the full tactical layer is M4's.
//!
//! M3's mode set is deliberately small (PLAN.md: *the first strategy
//! without the full tactical layer*): recover, engage, town, descend and
//! explore. T3.4 puts the playing behaviours on top of it and T3.5 the
//! town's survival reserves; M4's tactical layer replaces the
//! placeholder's one-step retreat.

use crate::borg::{stand_still, Memory, Rng};
use crate::danger::{assess, can_act, can_attack, danger_with, one_threat, ThreatSummary};
use crate::engine::PlayerView;
use crate::executor::Action;
use crate::knowledge::Knowledge;
use crate::observation::Item;
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
    /// Restock survival reserves and identify consumables in safety
    /// (task T3.5; GOALS principle 7).
    Town,
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
            Mode::Town => "town",
            Mode::Descend => "descend",
            Mode::Explore => "explore",
        }
    }
}

/// The survival reserves checked every think in town (GOALS principle 7).
/// These are M3's early-game kinds and minima, not M5's depth/class kit
/// policy. The equipped light does not count as a spare.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Reserve {
    Food,
    Light,
    Cure,
    Escape,
    Recall,
}

impl Reserve {
    /// Food and light first, then the D0015 cure/escape loadout, then the
    /// way home (which c15 §12.4 makes optional on DL1-4 when gold is short).
    const ALL: [Reserve; 5] = [
        Reserve::Food,
        Reserve::Light,
        Reserve::Cure,
        Reserve::Escape,
        Reserve::Recall,
    ];

    fn minimum(self, p: &Params) -> u32 {
        match self {
            Reserve::Food => p.reserve_food,
            Reserve::Light => p.reserve_light,
            Reserve::Cure => p.reserve_cure,
            Reserve::Escape => p.reserve_escape,
            Reserve::Recall => p.reserve_recall,
        }
    }

    fn matches(self, k: &Knowledge, item: &Item) -> bool {
        let name = match self {
            Reserve::Food => "Ration of Food",
            Reserve::Light => "Wooden Torch",
            Reserve::Cure => "Cure Light Wounds",
            Reserve::Escape => "Phase Door",
            Reserve::Recall => "Word of Recall",
        };
        k.kinds.get(item.kind as usize).is_some_and(|kind| {
            kind.name
                .chars()
                .filter(|c| !matches!(c, '&' | '~'))
                .collect::<String>()
                .trim()
                == name
        })
    }
}

/// One store action, recomputed from every observation so a sold-out row
/// can never leave a stale ware letter behind (S15, minimaxflash-2#24).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TownJob {
    Buy { shop: i32, ware: usize },
    Sell { item: usize },
}

impl TownJob {
    fn shop(self) -> i32 {
        match self {
            TownJob::Buy { shop, .. } => shop,
            // The Alchemist buys every scroll and potion (store.txt:152-180).
            TownJob::Sell { .. } => 4,
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
    /// The level generation this goal belongs to. A travel goal never
    /// survives a level change, even when depth changes back at one turn.
    pub level: u64,
    /// The last game turn the progress measure moved.
    pub progressed: i64,
    /// The progress measure itself (each mode defines its own).
    pub measure: i64,
}

impl Goal {
    fn new(mode: Mode, turn: i64, measure: i64, level: u64) -> Goal {
        Goal {
            mode,
            started: turn,
            level,
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
    /// The assessment at the character's grid this think, for calibration
    /// (`expected`, `worst`, `p_death`, ...). `None` only for decisions
    /// made before a world exists.
    pub threat: Option<ThreatSummary>,
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
    /// The level generation whose one up/down reroll was already used.
    /// A fresh generation may Explore first; it may not be rerolled on
    /// arrival (M3 review, PR #135 finding 1).
    rerolled_from: Option<u64>,
}

impl Default for FirstStrategy {
    fn default() -> Self {
        FirstStrategy {
            mode: Mode::Explore,
            goal: None,
            cooldowns: BTreeMap::new(),
            rerolled_from: None,
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
        // Rest is a long action: ask the assessment over the whole rest,
        // not one turn, or a pursuer two steps away looks harmless.
        let horizon = i32::try_from(ctx.params.rest_turns).unwrap_or(i32::MAX);
        assess(
            ctx.world,
            ctx.knowledge,
            x,
            y,
            horizon,
            ctx.params.risk_aversion_pct,
        )
        .risk(ctx.params.risk_aversion_pct)
            <= 0.0
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
            let danger = danger_with(
                ctx.world,
                ctx.knowledge,
                nx,
                ny,
                ctx.params.threat_horizon,
                ctx.params.risk_aversion_pct,
            );
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
        if !can_attack(ctx.world) {
            // The engine refuses an attack while afraid or paralysed.
            return None;
        }
        let t = &ctx.world.level.monsters[i];
        let dpt = ctx.world.character.melee_per_turn(ctx.knowledge)?;
        if dpt <= 0.0 || t.hp <= 0 {
            return None;
        }
        let turns = t.hp as f64 / dpt;
        let risk = one_threat(ctx.world, ctx.knowledge, t, ctx.view.x, ctx.view.y, 1)
            .risk(ctx.params.risk_aversion_pct);
        Some((risk * turns, turns))
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
        if !can_attack(ctx.world) {
            return None;
        }
        let (x, y) = (ctx.view.x, ctx.view.y);
        Self::adjacent(ctx)
            .into_iter()
            .filter_map(|i| {
                let t = &ctx.world.level.monsters[i];
                let d = one_threat(ctx.world, ctx.knowledge, t, x, y, 1)
                    .risk(ctx.params.risk_aversion_pct)
                    .round() as u32;
                let dir = DIRS
                    .iter()
                    .find(|(_, dx, dy)| (*dx, *dy) == (t.x - x, t.y - y))
                    .map(|(d, _, _)| *d)?;
                Some((d, dir, i))
            })
            .min_by_key(|&(d, _, i)| (d, i))
            .map(|(_, dir, _)| dir)
    }

    /// How many *identified* items of one reserve are in the pack. An
    /// unknown potion is not yet a usable cure (bbc D0011/D0012), even
    /// though the C Borg's item analysis exposes its real kind.
    fn reserve_count(reserve: Reserve, ctx: &DecisionCtx) -> u32 {
        ctx.world
            .character
            .inventory
            .iter()
            .filter(|item| item.aware && reserve.matches(ctx.knowledge, item))
            .map(|item| u32::from(item.number))
            .sum()
    }

    fn reserves_met(ctx: &DecisionCtx) -> bool {
        Reserve::ALL
            .into_iter()
            .all(|reserve| Self::reserve_count(reserve, ctx) >= reserve.minimum(ctx.params))
    }

    /// The first unmet reserve with an affordable ware. One item is bought
    /// per visit; the next observation chooses anew. A full pack permits a
    /// purchase only into an existing stack, matching the engine's combine
    /// rule without guessing a free slot (GOALS principle 7).
    fn next_purchase(ctx: &DecisionCtx) -> Option<TownJob> {
        for reserve in Reserve::ALL {
            if Self::reserve_count(reserve, ctx) >= reserve.minimum(ctx.params) {
                continue;
            }
            let purchase = ctx
                .world
                .stock
                .iter()
                .flat_map(|stock| {
                    stock
                        .wares
                        .iter()
                        .enumerate()
                        .map(move |(ware, w)| (stock.number, ware, w))
                })
                .filter(|(_, _, w)| {
                    w.item.number > 0
                        && w.item.aware
                        && reserve.matches(ctx.knowledge, &w.item)
                        && w.price >= 0
                        && w.price <= ctx.world.character.status.as_ref().map_or(0, |s| s.gold)
                        && (ctx.world.character.pack_free() > 0
                            || ctx
                                .world
                                .character
                                .inventory
                                .iter()
                                .any(|item| item.kind == w.item.kind))
                })
                .min_by_key(|(shop, ware, w)| (w.price, *shop, *ware))
                .map(|(shop, ware, _)| TownJob::Buy { shop, ware });
            if purchase.is_some() {
                return purchase;
            }
        }
        None
    }

    /// An unidentified scroll or potion to give to the Alchemist. Sale is
    /// identification, not income: `do_cmd_sell` learns the flavour and
    /// runes before handing it over (store.c:1947-1956), and
    /// `birth_no_selling` pays nothing. One of a stack is enough.
    fn next_sale(ctx: &DecisionCtx) -> Option<TownJob> {
        // TV_SCROLL and TV_POTION, object_base.txt's order at the pin.
        const CONSUMABLES: [u8; 2] = [25, 26];
        ctx.world
            .character
            .inventory
            .iter()
            .enumerate()
            .find(|(_, item)| !item.aware && CONSUMABLES.contains(&item.tval))
            .map(|(item, _)| TownJob::Sell { item })
    }

    /// Reserves are invariants, so identification waits until they are
    /// satisfied. There are deliberately no discretionary purchases in M3.
    fn town_job(ctx: &DecisionCtx) -> Option<TownJob> {
        Self::next_purchase(ctx).or_else(|| {
            if Self::reserves_met(ctx) {
                Self::next_sale(ctx)
            } else {
                None
            }
        })
    }

    /// An object worth stopping for lies underfoot (task T3.4): with
    /// `birth_no_selling` an object is worth carrying only to use or
    /// wear, but gold is the one loot that is value (S15 §6b) — and a
    /// dropped escape beside a fight is a rescue (bbc D0019). M3 takes
    /// what is underfoot whenever the pack has room; triage is M5's.
    /// The assessment at the character's grid, with the strategy's horizon
    /// and tolerance.
    fn here_threat(ctx: &DecisionCtx) -> Option<ThreatSummary> {
        Some(
            assess(
                ctx.world,
                ctx.knowledge,
                ctx.view.x,
                ctx.view.y,
                ctx.params.threat_horizon,
                ctx.params.risk_aversion_pct,
            )
            .summary(),
        )
    }

    /// Stand still without the shop-entrance trap: `,` on a shop opens the
    /// store, so use the neighbouring direction when it would.
    fn still(ctx: &DecisionCtx) -> Action {
        let here = ctx.world.level.at(ctx.view.x, ctx.view.y).map(|g| g.feat);
        stand_still(here, |dx, dy| {
            ctx.world
                .level
                .at(ctx.view.x + dx, ctx.view.y + dy)
                .map(|g| g.feat)
        })
    }

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
            Mode::Town => {
                let job = Self::town_job(ctx)?;
                let shop = ctx
                    .world
                    .shops
                    .iter()
                    .find(|shop| shop.number == job.shop())?;
                if (ctx.view.x, ctx.view.y) == (shop.x, shop.y) {
                    return Some(match job {
                        TownJob::Buy { ware, .. } => Action::Buy { ware },
                        TownJob::Sell { item } => Action::Sell { item, qty: 1 },
                    });
                }
                ctx.paths
                    .step(
                        ctx.world,
                        ctx.knowledge,
                        &PathGoal::Grids(vec![(shop.x, shop.y)]),
                    )
                    .map(Action::Walk)
            }
            Mode::Descend => {
                let here = ctx.world.level.at(ctx.view.x, ctx.view.y).map(|g| g.feat);
                let threat = danger_with(
                    ctx.world,
                    ctx.knowledge,
                    ctx.view.x,
                    ctx.view.y,
                    ctx.params.threat_horizon,
                    ctx.params.risk_aversion_pct,
                );
                if threat > 0
                    && matches!(
                        here,
                        Some(crate::borg::feat::LESS) | Some(crate::borg::feat::MORE)
                    )
                {
                    // A staircase under threat is not taken, but it is not
                    // a place to die either: leave it and cool the travel
                    // goal (M3 review's blocked-reroll window).
                    self.cooldowns.insert(
                        Mode::Descend,
                        ctx.view.turn + i64::from(ctx.params.goal_cooldown_turns),
                    );
                    self.goal = None;
                    return Self::retreat_step(ctx).map(Action::Walk);
                }
                if here == Some(crate::borg::feat::MORE) && threat == 0 {
                    // A staircase under fire is not taken: whatever is
                    // pressing is dealt with first (engaged or escaped).
                    return Some(Action::Descend);
                }
                if let Some(step) =
                    ctx.paths
                        .step(ctx.world, ctx.knowledge, &PathGoal::Stairs { up: false })
                {
                    return Some(Action::Walk(step));
                }
                // Only an exhausted level may reroll (S15 §2). A fresh
                // level with a frontier belongs to Explore; selecting a
                // better-generated level is stair-scumming (GOALS 9).
                let frontier = ctx
                    .paths
                    .map(ctx.world, ctx.knowledge, &PathGoal::Frontier)
                    .remaining(ctx.world, ctx.view.x, ctx.view.y);
                let explore_cooling = self
                    .cooldowns
                    .get(&Mode::Explore)
                    .is_some_and(|&until| ctx.view.turn < until);
                if (frontier != UNREACHED && !explore_cooling)
                    || self.rerolled_from == Some(ctx.world.level.generation)
                    || ctx.mem.level_changes_at(ctx.view.turn)
                        >= ctx.params.max_level_changes_per_turn
                {
                    return None;
                }
                if here == Some(crate::borg::feat::LESS) && threat == 0 {
                    self.rerolled_from = Some(ctx.world.level.generation);
                    Some(Action::Ascend)
                } else {
                    ctx.paths
                        .step(ctx.world, ctx.knowledge, &PathGoal::Stairs { up: true })
                        .map(Action::Walk)
                }
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
            // Purchases move the reserve counts; identification sales move
            // the aware item count (down as well as up is still progress).
            Mode::Town => {
                let reserves = Reserve::ALL
                    .into_iter()
                    .map(|r| Self::reserve_count(r, ctx))
                    .sum::<u32>();
                let aware = ctx
                    .world
                    .character
                    .inventory
                    .iter()
                    .filter(|item| item.aware)
                    .map(|item| u32::from(item.number))
                    .sum::<u32>();
                i64::from(reserves + aware)
            }
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
        // Survival reserves outrank an already-live travel goal: arriving
        // in town with shopping to do must not immediately descend again
        // (GOALS principle 7). A completed or cooled town goal ends here.
        let town_cooled = self.cooldowns.get(&Mode::Town).is_none_or(|&t| turn >= t);
        if ctx.world.level.depth == 0 && town_cooled && Self::town_job(ctx).is_some() {
            return Mode::Town;
        }
        if self.goal.as_ref().is_some_and(|g| g.mode == Mode::Town) {
            self.goal = None;
        }
        // No travel goal survives a level generation. In particular,
        // depth 1 → town → depth 1 at one game turn is not goal progress:
        // the new level must be evaluated for Explore first (M3 review).
        if self
            .goal
            .as_ref()
            .is_some_and(|g| g.level != ctx.world.level.generation)
        {
            self.goal = None;
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
                Mode::Town => i64::from(ctx.params.explore_budget_turns),
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
        // Utilities, highest first. Explore is the default occupation
        // and descend takes over when explore is cooling or done — but
        // not in town: the town has no experience and one staircase the
        // character stands on at birth (c15 TOWN-01; S15 INSTRUCTIONS
        // §2), and exploring it instead of leaving is how one run
        // starved to death mapping it for 50,000 turns.
        let descend_first = ctx.world.level.depth == 0;
        let frontier = ctx
            .paths
            .map(ctx.world, ctx.knowledge, &PathGoal::Frontier)
            .remaining(ctx.world, ctx.view.x, ctx.view.y);
        let stairs = ctx
            .paths
            .map(ctx.world, ctx.knowledge, &PathGoal::Stairs { up: false })
            .remaining(ctx.world, ctx.view.x, ctx.view.y);
        let cooled = |m: Mode| self.cooldowns.get(&m).is_none_or(|&t| turn >= t);
        if descend_first && stairs != UNREACHED && cooled(Mode::Descend) {
            Mode::Descend
        } else if frontier != UNREACHED && cooled(Mode::Explore) {
            Mode::Explore
        } else if stairs != UNREACHED && cooled(Mode::Descend) {
            Mode::Descend
        } else {
            // No frontier remains (or Explore is cooling): Descend's
            // up-and-down reroll is the productive fallback. Choosing
            // Explore with an empty frontier only holds (T3.7).
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
        if !can_act(ctx.world) {
            let action = Self::still(ctx);
            return Decision {
                action,
                mode: "disabled",
                goal: "paralysed",
                budget_left: 0,
                candidates: vec![(action, 1.0)],
                threat: Self::here_threat(ctx),
                violations: Vec::new(),
            };
        }
        self.note_failures(ctx);
        let mode = self.choose_mode(ctx);
        self.mode = mode;

        // A fresh goal when the mode changed or none is live; the
        // measure is taken now, and progress is its movement.
        let now_measure = Self::measure(mode, ctx);
        if self.goal.as_ref().is_none_or(|g| g.mode != mode) {
            self.goal = Some(Goal::new(
                mode,
                ctx.view.turn,
                now_measure,
                ctx.world.level.generation,
            ));
        }
        // Progress: the measure moved, or the mode's act succeeded.
        if let Some(goal) = &mut self.goal {
            if now_measure != goal.measure {
                goal.measure = now_measure;
                goal.progressed = ctx.view.turn;
            }
        }

        let level_changes = ctx.mem.level_changes_at(ctx.view.turn);
        if level_changes >= ctx.params.max_level_changes_per_turn {
            // A separate backstop for generation loops: depth oscillation
            // is not progress, and the level cap is never reset by stairs.
            self.cooldowns.insert(
                Mode::Descend,
                ctx.view.turn + i64::from(ctx.params.goal_cooldown_turns),
            );
            if self.goal.as_ref().is_some_and(|g| g.mode == Mode::Descend) {
                self.goal = None;
            }
            let why = format!(
                "{level_changes} level changes at game turn {} took no time (reroll cap)",
                ctx.view.turn
            );
            let (x, y) = (ctx.view.x, ctx.view.y);
            let here = ctx.world.level.at(x, y).map(|g| g.feat);
            let around = |dx: i32, dy: i32| ctx.world.level.at(x + dx, y + dy).map(|g| g.feat);
            return Decision {
                action: stand_still(here, around),
                mode: mode.name(),
                goal: "stand still",
                budget_left: 0,
                candidates: vec![],
                threat: Self::here_threat(ctx),
                violations: vec![why],
            };
        }

        let tries = ctx.mem.tries_at(ctx.view.turn);
        let budget_left = (i64::from(ctx.params.max_tries_per_turn) - i64::from(tries))
            .min(i64::from(ctx.params.max_level_changes_per_turn) - i64::from(level_changes));
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
                threat: Self::here_threat(ctx),
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
            Mode::Town => "restock and identify",
            Mode::Descend => "reach the down staircase",
            Mode::Explore => "reveal the frontier",
        };
        Decision {
            action,
            mode: mode.name(),
            goal: goal_name,
            budget_left,
            candidates: vec![(action_clone(&action), 1.0)],
            threat: Self::here_threat(ctx),
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
        k.kinds = (0..=6)
            .map(|index| crate::knowledge::ObjectKind {
                index,
                ..Default::default()
            })
            .collect();
        for (index, name, tval, sval) in [
            (1, "& Ration~ of Food", 28, 5),
            (2, "& Wooden Torch~", 19, 1),
            (3, "Cure Light Wounds", 26, 8),
            (4, "Phase Door", 25, 1),
            (5, "Word of Recall", 25, 24),
            (6, "Mystery", 26, 99),
        ] {
            k.kinds[index].name = name.into();
            k.kinds[index].tval = tval;
            k.kinds[index].sval = sval;
        }
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

    /// A test consumable. Kinds 1–6 are food, light, cure, escape, Recall
    /// and an unknown potion, installed by `ctx_of`.
    fn item(kind: u32, number: u8, aware: bool) -> Item {
        let (tval, sval) = match kind {
            1 => (28, 5),
            2 => (19, 1),
            3 => (26, 8),
            4 => (25, 1),
            5 => (25, 24),
            6 => (26, 99),
            _ => unreachable!(),
        };
        Item {
            kind,
            tval,
            sval,
            number,
            weight: 1,
            timeout: 0,
            level: 1,
            desc: format!("test kind {kind}"),
            aware,
            fully_known: aware,
            runes_known: aware,
            flags: [0; 5],
            ac: 0,
            dd: 0,
            ds: 0,
            to_h: 0,
            to_d: 0,
            to_a: 0,
            modifiers: vec![0; 16],
            elements: vec![(0, 0); 25],
        }
    }

    fn stock(number: i32, kinds: &[(u32, i32)]) -> crate::observation::StoreStock {
        crate::observation::StoreStock {
            number,
            wares: kinds
                .iter()
                .map(|&(kind, price)| crate::observation::Ware {
                    item: item(kind, 40, true),
                    price,
                })
                .collect(),
        }
    }

    fn make_town(w: &mut World, gold: i32, inventory: &[(u32, u8)]) {
        w.level.depth = 0;
        // The map helper observed at depth 1, when shops are deliberately
        // not noted; expose its two shop entrances to the town tests.
        for y in 0..w.level.height {
            for x in 0..w.level.width {
                let number = match w.level.at(x, y).map(|g| g.feat) {
                    Some(crate::borg::feat::STORE_GENERAL) => 0,
                    Some(crate::borg::feat::STORE_ALCHEMY) => 4,
                    _ => continue,
                };
                w.shops.push(crate::world::Shop {
                    number,
                    x,
                    y,
                    last_visit: None,
                });
            }
        }
        w.character.inventory = inventory
            .iter()
            .map(|&(kind, number)| item(kind, number, true))
            .collect();
        if let Some(s) = w.character.status.as_mut() {
            s.depth = 0;
            s.gold = gold;
            s.pack_used = inventory.len() as i32;
            s.pack_size = 23;
        }
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
    fn the_decision_exposes_the_threat_numbers() {
        let (mut s, w, k, mut p, m, mut rng, params) =
            ctx_of(&["#######", "#@M..>#", "#######"], 20, 20);
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
        let t = d.threat.expect("an assessment on every decision");
        assert!(t.expected > 0.0 && t.worst >= t.expected, "{t:?}");
        assert_eq!(t.horizon, params.threat_horizon);
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
    fn an_exhausted_level_rerolls_from_its_up_staircase() {
        let (mut s, mut w, k, _, m, mut rng, params) =
            ctx_of(&["######", "#@<..#", "######"], 20, 20);
        w.character.status.as_mut().unwrap().x = 2;
        let mut p = Paths::new(&w);
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
        assert_eq!(d.action, Action::Ascend, "take the exit, do not hold on it");

        let (mut s, w, k, _, m, mut rng, params) = ctx_of(&["######", "#@<..#", "######"], 20, 20);
        let mut p = Paths::new(&w);
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
        assert_eq!(d.action, Action::Walk(6), "walk to the up staircase first");
    }

    #[test]
    fn an_unsafe_up_staircase_retreats_instead_of_holding() {
        let (mut s, mut w, k, _, m, mut rng, params) =
            ctx_of(&["######", "#@<M.#", "######"], 20, 20);
        w.character.status.as_mut().unwrap().x = 2;
        // A remembered threat is not visible contact, so Engage does not
        // answer it. The stair remains unsafe and must not trap Descend.
        w.level.monsters[0].in_view = false;
        let mut p = Paths::new(&w);
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
        assert_eq!(
            d.action,
            Action::Walk(4),
            "west, away from the remembered monster"
        );
        assert!(s.cooldowns.contains_key(&Mode::Descend));
    }

    #[test]
    fn a_fresh_level_explores_instead_of_rerolling() {
        let (mut s, mut w, k, _, m, mut rng, params) = ctx_of(&["#####", "#@.>#", "#####"], 20, 20);
        let mut p = Paths::new(&w);
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

        // The next generation has a frontier. Even though depth and game
        // turn could be identical after two stair actions, its generation
        // differs and the old Descend goal cannot win by hysteresis.
        w.level.generation += 1;
        let width = w.level.width as usize;
        w.level.grids[width + 3].feat = crate::borg::feat::NONE;
        let mut p = Paths::new(&w);
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
        assert_eq!(
            d.mode, "explore",
            "a fresh level is explored, never rerolled on arrival"
        );
        assert_eq!(d.action, Action::Walk(6));
    }

    #[test]
    fn an_exhausted_frontier_can_reroll_but_a_fresh_one_cannot() {
        let (mut s, mut w, k, _, m, mut rng, params) =
            ctx_of(&["######", "#@<.?#", "######"], 20, 20);
        w.character.status.as_mut().unwrap().x = 2;
        let mut p = Paths::new(&w);
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
        assert_eq!(
            d.mode, "explore",
            "a reachable fresh frontier owns the level"
        );
        s.cooldowns.insert(Mode::Explore, w.turn + 100);
        let mut p = Paths::new(&w);
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
        assert_eq!(d.action, Action::Ascend, "a failed frontier may reroll");
    }

    #[test]
    fn rerolls_are_capped_per_level_and_per_game_turn() {
        let (mut s, mut w, k, _, mem, mut rng, params) =
            ctx_of(&["######", "#@<..#", "######"], 20, 20);
        w.character.status.as_mut().unwrap().x = 2;
        let mut p = Paths::new(&w);
        let d = decide_with(
            &mut s,
            &w,
            &k,
            &mut p,
            &mem,
            &mut rng,
            &params,
            &view(&w),
            &[],
        );
        assert_eq!(d.action, Action::Ascend);

        // Still the same generation (for example, if Ascend were refused):
        // its one reroll has been used and cannot be chosen again.
        let mut p = Paths::new(&w);
        let d = decide_with(
            &mut s,
            &w,
            &k,
            &mut p,
            &mem,
            &mut rng,
            &params,
            &view(&w),
            &[],
        );
        assert_ne!(d.action, Action::Ascend, "one reroll per level generation");

        let (mut s, mut w, k, _, mut mem, mut rng, params) =
            ctx_of(&["######", "#@<..#", "######"], 20, 20);
        w.character.status.as_mut().unwrap().x = 2;
        mem.record(w.turn, Action::Descend);
        mem.record(w.turn, Action::Ascend);
        let mut p = Paths::new(&w);
        let d = decide_with(
            &mut s,
            &w,
            &k,
            &mut p,
            &mem,
            &mut rng,
            &params,
            &view(&w),
            &[],
        );
        assert_eq!(
            d.action,
            Action::Hold,
            "the third generation must take game time"
        );
        assert!(d.violations.iter().any(|v| v.contains("reroll cap")));
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
    fn reserves_are_bought_one_at_a_time_before_town_is_left() {
        let (mut s, mut w, k, _, m, mut rng, params) =
            ctx_of(&["########", "#@a..>.#", "#...s..#", "########"], 20, 20);
        make_town(&mut w, 137, &[(1, 3), (2, 2), (5, 1)]);
        w.character.status.as_mut().unwrap().x = 2;
        w.stock = vec![stock(4, &[(3, 20), (4, 15), (5, 125)])];
        let mut p = Paths::new(&w);
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
        assert_eq!(d.mode, "town");
        assert_eq!(d.action, Action::Buy { ware: 0 }, "three CLW before Phase");

        w.character.inventory.push(item(3, 3, true));
        w.character.status.as_mut().unwrap().pack_used += 1;
        let mut p = Paths::new(&w);
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
        assert_eq!(d.mode, "town");
        assert_eq!(d.action, Action::Buy { ware: 1 }, "then four Phase Door");

        w.character.inventory.push(item(4, 4, true));
        w.character.status.as_mut().unwrap().pack_used += 1;
        let mut p = Paths::new(&w);
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
        assert_eq!(
            d.mode, "descend",
            "the reserves are invariants, not a shopping loop"
        );
        assert!(
            matches!(d.action, Action::Walk(_)),
            "back toward the staircase"
        );
    }

    #[test]
    fn unknown_consumables_are_sold_for_identification_after_reserves() {
        let (mut s, mut w, k, _, m, mut rng, params) =
            ctx_of(&["########", "#@a..>.#", "########"], 20, 20);
        make_town(&mut w, 137, &[(1, 3), (2, 2), (3, 3), (4, 4), (5, 1)]);
        w.character.status.as_mut().unwrap().x = 2;
        w.character.inventory.push(item(6, 2, false));
        w.character.status.as_mut().unwrap().pack_used += 1;
        let mut p = Paths::new(&w);
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
        assert_eq!(d.mode, "town");
        assert_eq!(d.action, Action::Sell { item: 5, qty: 1 });

        // An unmet reserve outranks identification: the Phase purchase
        // happens before the unknown potion is handed over.
        w.character.inventory[3].number = 3;
        w.stock = vec![stock(4, &[(4, 15)])];
        let mut p = Paths::new(&w);
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
        assert_eq!(d.action, Action::Buy { ware: 0 });
    }

    #[test]
    fn an_unaware_cure_does_not_count_as_a_reserve() {
        let (mut s, mut w, k, _, m, mut rng, params) =
            ctx_of(&["######", "#@a..#", "######"], 20, 20);
        make_town(&mut w, 137, &[(1, 3), (2, 2), (4, 4), (5, 1)]);
        w.character.status.as_mut().unwrap().x = 2;
        w.character.inventory.push(item(3, 3, false));
        w.stock = vec![stock(4, &[(3, 20)])];
        let mut p = Paths::new(&w);
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
        assert_eq!(
            d.action,
            Action::Buy { ware: 0 },
            "unknown cures still need identifying"
        );
    }

    #[test]
    fn unavailable_stock_and_a_full_pack_do_not_trap_the_borg_in_town() {
        let (mut s, mut w, k, _, m, mut rng, params) =
            ctx_of(&["######", "#@.>.#", "######"], 20, 20);
        make_town(&mut w, 137, &[(5, 1)]);
        let mut p = Paths::new(&w);
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
        assert_eq!(d.mode, "descend", "nothing affordable or stocked: go play");

        let (mut s, mut w, k, _, m, mut rng, params) =
            ctx_of(&["######", "#@a>.#", "######"], 20, 20);
        make_town(&mut w, 137, &[(1, 3), (2, 2), (3, 3), (4, 3), (5, 1)]);
        w.character.status.as_mut().unwrap().x = 2;
        w.character.status.as_mut().unwrap().pack_used = 23;
        w.stock = vec![stock(4, &[(4, 15), (5, 125)])];
        let mut p = Paths::new(&w);
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
        assert_eq!(
            d.action,
            Action::Buy { ware: 0 },
            "the existing Phase stack has room"
        );

        w.character.inventory[3].number = 4;
        w.character.inventory.retain(|item| item.kind != 5);
        let mut p = Paths::new(&w);
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
        assert_eq!(d.mode, "descend", "a new stack cannot fit, so Recall waits");
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
