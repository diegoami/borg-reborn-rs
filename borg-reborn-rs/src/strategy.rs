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
use crate::danger::{assess, can_act, danger_with, ThreatSummary};
use crate::engine::PlayerView;
use crate::executor::Action;
use crate::knowledge::Knowledge;
use crate::observation::Item;
use crate::params::Params;
use crate::path::{Goal as PathGoal, Paths, UNREACHED};
use crate::world::World;
use std::collections::BTreeMap;

/// Directions 1–9 (keypad layout) as (dir, dx, dy).
/// Engage's progress measure when no monster is adjacent (#139).
const ENGAGE_NO_CONTACT: i64 = i64::MIN;

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
    /// Eat a known food item before the food timer reaches starvation.
    Eat,
    /// Head back to town when the level has nothing more to give:
    /// starving with no food left (bbc D0009/D0011, rust D1420) follows
    /// the manual's recall-and-home routine (c15 §12.12: leave when
    /// there is nothing more to gain, read recall early and in a safe
    /// spot) — take the up staircase, recall, or dig for the way up
    /// (T4.6, #123).
    Return,
    /// No mode can act here: the frontier is empty and no staircase is
    /// reachable. The state is named so the trace and the death dossier
    /// say why the Borg is standing still (T4.6, #123).
    Stuck,
}

impl Mode {
    pub fn name(self) -> &'static str {
        match self {
            Mode::Recover => "recover",
            Mode::Engage => "engage",
            Mode::Town => "town",
            Mode::Descend => "descend",
            Mode::Explore => "explore",
            Mode::Eat => "eat",
            Mode::Return => "return",
            Mode::Stuck => "stuck",
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
    /// Why the mode was chosen and the goal's timing (#138); `None` from a
    /// strategy that does not say.
    pub why: Option<crate::trace::Why>,
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
    /// The tactical responses of the last think, best first, for the trace
    /// (T4.3). Empty when no tactical choice was made.
    last_candidates: Vec<(Action, f64)>,
    /// The action the last think chose: the one the executor's next
    /// report is about.
    last_action: Option<Action>,
    /// Which rule of `choose_mode` chose this think's mode, for the trace
    /// (#138: the mode choice is a priority rule, not a score).
    rule: &'static str,
    /// Actions the executor reported refused, barred until this game turn
    /// (#137). Contact keeps Engage live past its cooldown, so the
    /// refusal must bar the action itself, not only the mode. The bar is
    /// the exact `Action`, pack index included: if the pack reorders, the
    /// same scroll at a new index is not barred. The causes seen so far are
    /// state refusals that `can_read`/`can_attack` now prevent; if
    /// item-specific refusals recur, bar by item kind (#165).
    refused: BTreeMap<Action, i64>,
}

impl Default for FirstStrategy {
    fn default() -> Self {
        FirstStrategy {
            mode: Mode::Explore,
            goal: None,
            cooldowns: BTreeMap::new(),
            rerolled_from: None,
            last_candidates: Vec::new(),
            last_action: None,
            rule: "",
            refused: BTreeMap::new(),
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

    /// Whether a known food item is in the pack.
    fn has_food(ctx: &DecisionCtx) -> bool {
        // Only a known kind counts, as in every other item path (#165);
        // rations carry EASY_KNOW (`object_base.txt:166-169`), so this
        // changes nothing for them, but an unknown mushroom is not food.
        ctx.world.character.inventory.iter().any(|it| {
            it.aware
                && ctx.knowledge.kind(it.kind).is_some_and(|k| {
                    k.name
                        .chars()
                        .filter(|c| !matches!(c, '&' | '~'))
                        .collect::<String>()
                        .trim()
                        == "Ration of Food"
                })
        })
    }

    /// Whether resting is safe where the Borg stands: the strategy
    /// manual's rule in its strictest reading (c15 REST-01: no awake
    /// monster about to reach you) — no measurable danger at all. The C
    /// Borg's `avoidance` semantics (danger below current HP, spec §8)
    /// rest beside a small monster and die on the rest's own turns
    /// (bbc D0017); M3 takes the strict form.
    fn may_rest(ctx: &DecisionCtx) -> bool {
        let (x, y) = (ctx.view.x, ctx.view.y);
        // One turn: a rest is interrupted when a monster arrives, and
        // what matters now is whether one can reach and strike. A distant
        // monster does not make the next turn unsafe.
        // On the composite scale, disabling included (#183): a floating eye
        // in reach deals no hit points worth a risk, but its gaze paralyses
        // again and again while the Borg rests beside it.
        let a = assess(
            ctx.world,
            ctx.knowledge,
            x,
            y,
            ctx.params.threat_horizon,
            ctx.params.risk_aversion_pct,
        );
        crate::tactics::composite(
            &a,
            ctx.params.risk_aversion_pct,
            f64::from(ctx.view.hp.max(1)),
        ) <= 0.0
    }

    /// The one step that most reduces danger: the cheapest of the eight
    /// neighbours on the tactical layer's composite scale (T4.6, #123 —
    /// `danger_with` rounds a small risk to 0 and ignores the disabling
    /// half, so a NEVER_MOVE mold read as free), only through grids the
    /// Borg may enter. Ties keep the first direction in [`DIRS`] order
    /// (keypad 1, 2, 3, 4, 6, 7, 8, 9): only a strictly lower risk replaces
    /// the best so far.
    fn retreat_step(ctx: &mut DecisionCtx) -> Option<u8> {
        let (x, y) = (ctx.view.x, ctx.view.y);
        let hp = f64::from(ctx.view.hp.max(1));
        let mut best: Option<(f64, u8)> = None;
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
            let risk = crate::tactics::composite(
                &assess(
                    ctx.world,
                    ctx.knowledge,
                    nx,
                    ny,
                    ctx.params.threat_horizon,
                    ctx.params.risk_aversion_pct,
                ),
                ctx.params.risk_aversion_pct,
                hp,
            );
            let better = match best {
                None => true,
                Some((b, _)) => risk < b,
            };
            if better {
                best = Some((risk, d));
            }
        }
        best.map(|(_, d)| d)
    }

    /// One step along the flow toward `goal`, refused when that step
    /// lands on a grid more dangerous than where the Borg stands (the
    /// composite scale; T4.6, #123): threatened travel retreats rather
    /// than walking deeper into reach. There is no `here > 0.0` gate —
    /// d0006's NEVER_MOVE mold deals no hit-point risk anywhere, and
    /// only the disabling half of the composite separates the grids.
    fn travel_step(ctx: &mut DecisionCtx, goal: &PathGoal) -> Option<Action> {
        let step = ctx.paths.step(ctx.world, ctx.knowledge, goal)?;
        let &(d, dx, dy) = DIRS.iter().find(|&&(d, _, _)| d == step)?;
        let (x, y) = (ctx.view.x, ctx.view.y);
        let hp = f64::from(ctx.view.hp.max(1));
        let at = |x: i32, y: i32| {
            crate::tactics::composite(
                &assess(
                    ctx.world,
                    ctx.knowledge,
                    x,
                    y,
                    ctx.params.threat_horizon,
                    ctx.params.risk_aversion_pct,
                ),
                ctx.params.risk_aversion_pct,
                hp,
            )
        };
        // Step back only when the next grid is riskier by at least half a
        // hit point: the path's own resolution (`danger_with` rounds the
        // risk to whole steps). Below it the path already judged the step
        // free, and retreating flips the Borg back and forth: #176's
        // priest-4 spent 20,000 turns between two town grids because a
        // squint-eyed rogue last seen 8,000 turns before put 0.0004 on the
        // next one.
        if at(x + dx, y + dy) >= at(x, y) + 0.5 {
            return Self::retreat_step(ctx).map(Action::Walk);
        }
        Some(Action::Walk(d))
    }

    /// The cure to drink when the poison and cuts on the character would
    /// take every hit point left (#175), from `object.txt`: Neutralize
    /// Poison (`CURE:POISONED`) when poisoned, else the strongest known Cure
    /// Wounds potion: Critical heals 60 and cures cuts and poison, Serious
    /// heals 40 and cures cuts, Light heals 20 and lowers a cut by 20.
    fn bleed_cure(ctx: &DecisionCtx) -> Option<usize> {
        let s = ctx.world.character.status.as_ref()?;
        let bleed = crate::danger::pending_bleed(ctx.world);
        if bleed < s.hp {
            return None;
        }
        let poisoned = s.timed.get(7).copied().unwrap_or(0) > 0;
        let potion = |name: &str| {
            crate::tactics::pack_item(ctx.world, ctx.knowledge, name)
                .filter(|(_, it)| it.tval == crate::world::tv::POTION)
                .map(|(i, _)| i)
        };
        let first = if poisoned {
            potion("Neutralize Poison")
        } else {
            None
        };
        first
            .or_else(|| potion("Cure Critical Wounds"))
            .or_else(|| potion("Cure Serious Wounds"))
            .or_else(|| potion("Cure Light Wounds"))
    }

    /// Whether the readiness policy allows `depth` (T5.1, #125; see
    /// `readiness.rs`). Depth 1 always passes.
    fn ready_for(ctx: &DecisionCtx, depth: i32) -> bool {
        crate::readiness::ready(ctx.world, ctx.knowledge, ctx.params, depth).ok()
    }

    /// A known Word of Recall scroll in the pack (the reserve's aware
    /// item; an unidentified scroll is not a way home, bbc D0011).
    fn recall_scroll(ctx: &DecisionCtx) -> Option<usize> {
        // A scroll the engine will not let the character read is not on
        // hand this think (`player-util.c:player_can_read`; #137: Return
        // read Recall with no light thirteen times, D1668).
        if !crate::danger::can_read(ctx.world) {
            return None;
        }
        ctx.world
            .character
            .inventory
            .iter()
            .enumerate()
            .find(|(_, item)| item.aware && Reserve::Recall.matches(ctx.knowledge, item))
            .map(|(i, _)| i)
    }

    /// Dig for a way out (T4.6, #123): alter an adjacent diggable wall,
    /// else walk to a walkable grid next to one, else read a known Word
    /// of Recall that is not already counting down.
    fn dig_step(ctx: &mut DecisionCtx) -> Option<Action> {
        let (x, y) = (ctx.view.x, ctx.view.y);
        for &(d, dx, dy) in DIRS.iter() {
            if crate::borg::diggable(ctx.world.level.at(x + dx, y + dy).map(|g| g.feat)) {
                return Some(Action::Alter(d));
            }
        }
        let mut targets = Vec::new();
        for gy in 0..ctx.world.level.height {
            for gx in 0..ctx.world.level.width {
                if !crate::borg::diggable(ctx.world.level.at(gx, gy).map(|g| g.feat)) {
                    continue;
                }
                for &(_, dx, dy) in DIRS.iter() {
                    let (nx, ny) = (gx + dx, gy + dy);
                    if crate::path::enter_cost(ctx.world, nx, ny).is_some() {
                        targets.push((nx, ny));
                    }
                }
            }
        }
        if !targets.is_empty() {
            if let Some(step) = ctx
                .paths
                .step(ctx.world, ctx.knowledge, &PathGoal::Grids(targets))
            {
                return Some(Action::Walk(step));
            }
        }
        // No wall to dig known: a recall scroll is the other way out of
        // a sealed room (word_recall counts down while one is active).
        let recall_idle = ctx
            .world
            .character
            .status
            .as_ref()
            .is_some_and(|s| s.word_recall <= 0);
        let scroll = if recall_idle {
            Self::recall_scroll(ctx)
        } else {
            None
        };
        if let Some(item) = scroll {
            return Some(Action::Read { item });
        }
        None
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
            .summary_detailed(ctx.world, ctx.knowledge, ctx.view.x, ctx.view.y),
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
                    // recovery may ignore the room). But when no
                    // monster's next blow can kill, rest anyway and let
                    // the engine's disturbance interrupt it (T4.6, #123;
                    // bbc #408/#409's heal-while-fleeing; c15 REST-01,
                    // `src/player-util.c`): a live Recover goal makes
                    // progress under oscillation where a retreat loop
                    // (rust D1330) does not.
                    let worst = assess(
                        ctx.world,
                        ctx.knowledge,
                        ctx.view.x,
                        ctx.view.y,
                        ctx.params.threat_horizon,
                        ctx.params.risk_aversion_pct,
                    )
                    .worst;
                    if worst < f64::from(ctx.view.hp.max(1)) {
                        return Some(Action::Rest(ctx.params.rest_turns));
                    }
                    return Self::retreat_step(ctx).map(Action::Walk);
                }
                Some(Action::Rest(ctx.params.rest_turns))
            }
            Mode::Engage => {
                // The tactical layer (T4.3): every response, scored on the
                // same risk scale. It replaces the first-applicable
                // fight-or-retreat rule.
                //
                // Contact overrides Engage's cooldown (a monster in reach
                // is answered this think), so a refused response is barred
                // by itself until the cooldown ends (#137): the next-best
                // response is taken instead of repeating the refusal.
                let responses =
                    crate::tactics::responses(ctx.world, ctx.knowledge, ctx.params, ctx.view);
                self.last_candidates = responses.iter().map(|r| (r.action, r.score)).collect();
                responses
                    .iter()
                    .find(|r| !self.refused.contains_key(&r.action))
                    .map(|r| r.action)
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
                // Down only when ready for the next depth (T5.1, #125); an
                // unready Borg on an explored level rerolls it upward below,
                // under the same exhausted-level rule as before.
                let ready_next = Self::ready_for(ctx, ctx.world.level.depth + 1);
                if ready_next && here == Some(crate::borg::feat::MORE) && threat == 0 {
                    // A staircase under fire is not taken: whatever is
                    // pressing is dealt with first (engaged or escaped).
                    return Some(Action::Descend);
                }
                if ready_next {
                    if let Some(action) = Self::travel_step(ctx, &PathGoal::Stairs { up: false }) {
                        return Some(action);
                    }
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
                    Self::travel_step(ctx, &PathGoal::Stairs { up: true })
                }
            }
            Mode::Return => {
                let here = ctx.world.level.at(ctx.view.x, ctx.view.y).map(|g| g.feat);
                if here == Some(crate::borg::feat::LESS) {
                    // The up staircase is always taken: town is where the
                    // return ends, threat or not — waiting on the stairs
                    // only feeds the monster (c15 §12.12).
                    return Some(Action::Ascend);
                }
                let threat = danger_with(
                    ctx.world,
                    ctx.knowledge,
                    ctx.view.x,
                    ctx.view.y,
                    ctx.params.threat_horizon,
                    ctx.params.risk_aversion_pct,
                );
                let recall_idle = ctx
                    .world
                    .character
                    .status
                    .as_ref()
                    .is_some_and(|s| s.word_recall <= 0);
                if threat == 0 && recall_idle {
                    // Recall early and in a safe spot (c15 §12.12): a
                    // known scroll read here beats walking up through a
                    // dungeon that already starved the Borg once.
                    if let Some(item) = Self::recall_scroll(ctx) {
                        return Some(Action::Read { item });
                    }
                }
                if let Some(action) = Self::travel_step(ctx, &PathGoal::Stairs { up: true }) {
                    return Some(action);
                }
                // No up staircase known: explore for one before any
                // down-stair fallback, then dig for the way up (rust
                // D1420's sealed room).
                if let Some(action) = Self::travel_step(ctx, &PathGoal::Frontier) {
                    return Some(action);
                }
                Self::dig_step(ctx)
            }
            Mode::Stuck => {
                // A staircase is on its way (cooldown, not absence, is
                // what keeps Descend out): wait it out. With no stair
                // and no frontier reachable anywhere, dig or recall for
                // a way out (T4.6, #123; rust-stuck-sealed-room).
                let down = ctx
                    .paths
                    .map(ctx.world, ctx.knowledge, &PathGoal::Stairs { up: false })
                    .remaining(ctx.world, ctx.view.x, ctx.view.y);
                let up = ctx
                    .paths
                    .map(ctx.world, ctx.knowledge, &PathGoal::Stairs { up: true })
                    .remaining(ctx.world, ctx.view.x, ctx.view.y);
                if down != UNREACHED || up != UNREACHED {
                    return None;
                }
                Self::dig_step(ctx)
            }
            Mode::Eat => {
                let item =
                    ctx.world
                        .character
                        .inventory
                        .iter()
                        .enumerate()
                        .find_map(|(i, it)| {
                            if !it.aware {
                                return None;
                            }
                            let name = ctx
                                .knowledge
                                .kind(it.kind)?
                                .name
                                .chars()
                                .filter(|c| !matches!(c, '&' | '~'))
                                .collect::<String>();
                            (name.trim() == "Ration of Food").then_some(i)
                        })?;
                Some(Action::Eat { item })
            }
            Mode::Explore => {
                // Standing still goes through `still`, which never opens
                // a shop by holding on its entrance (#139).
                Some(
                    Self::travel_step(ctx, &PathGoal::Frontier).unwrap_or_else(|| Self::still(ctx)),
                )
            }
        }
    }

    /// A mode's progress measure at this think.
    fn measure(mode: Mode, ctx: &DecisionCtx) -> i64 {
        match mode {
            Mode::Recover => ctx.view.hp as i64,
            // The fight's progress is the threat going down: the least
            // hit points any adjacent monster has.
            // With no adjacent monster the measure is its own value, not
            // 0, which would read as a monster at 0 HP (#139).
            Mode::Engage => Self::adjacent(ctx)
                .into_iter()
                .map(|i| i64::from(ctx.world.level.monsters[i].hp))
                .min()
                .unwrap_or(ENGAGE_NO_CONTACT),
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
            Mode::Eat => ctx
                .world
                .character
                .status
                .as_ref()
                .map_or(0, |s| i64::from(s.food)),
            // Going up is depth shrinking; on one depth, mapping more of
            // the level is the progress (the frontier walk that finds
            // the up staircase).
            Mode::Return => i64::from(ctx.world.level.depth) * 1_000_000 + Self::mapped(ctx.world),
            Mode::Stuck => 0,
        }
    }

    /// Choose the mode by utility, with hysteresis: the invariant
    /// (recovery) outranks everything; a live goal keeps its mode; a
    /// failed mode waits out its cooldown.
    fn choose_mode(&mut self, ctx: &mut DecisionCtx) -> Mode {
        let turn = ctx.view.turn;
        // Starving with no food left outranks recovery itself: resting
        // spends the food timer, and the level has nothing more to gain
        // (the manual's recall-and-home routine, c15 §12.12; bbc D0009,
        // rust D1420). Contact still answers first — Engage is chosen
        // below. Descending deeper is banned the whole time (T4.6, #123).
        let starving_no_food = ctx
            .world
            .character
            .status
            .as_ref()
            .is_some_and(|s| s.food < ctx.params.eat_below_food)
            && !Self::has_food(ctx);
        let block_descend = starving_no_food && ctx.world.level.depth > 0;
        if block_descend
            && Self::adjacent(ctx).is_empty()
            && self.cooldowns.get(&Mode::Return).is_none_or(|&t| turn >= t)
        {
            self.rule = "starving with no food: return home";
            return Mode::Return;
        }
        // A breeder colony on the level (#183; c08 §8.10: kill the first
        // breeder or leave the level, never rest while a colony grows):
        // leave by the way home, before resting. Contact still answers
        // first (Engage, below).
        let colony = ctx.params.breeder_colony > 0
            && crate::danger::breeders(ctx.world, ctx.knowledge)
                >= ctx.params.breeder_colony as usize;
        if colony
            && ctx.world.level.depth > 0
            && Self::adjacent(ctx).is_empty()
            && self.cooldowns.get(&Mode::Return).is_none_or(|&t| turn >= t)
        {
            self.rule = "a breeder colony on the level: leave it";
            return Mode::Return;
        }
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
            self.rule = "hurt and safe: recover";
            return Mode::Recover;
        }
        // Contact second (task T3.4): a monster in reach is answered
        // this think — fought when the exchange wins it, left when it
        // does not. M4's tactical layer replaces the whole comparison.
        // A monster in reach that threatens nothing (Farmer Maggot, no
        // blows) is not a fight: c15 FIGHT-06, don't fight for nothing.
        // #183: full-4-necro-3 attacked Maggot 493 times in the town, every
        // response scoring 0, and starved with 195 gold.
        let in_reach_threat = !Self::adjacent(ctx).is_empty() && {
            let a = assess(
                ctx.world,
                ctx.knowledge,
                ctx.view.x,
                ctx.view.y,
                ctx.params.threat_horizon,
                ctx.params.risk_aversion_pct,
            );
            crate::tactics::composite(
                &a,
                ctx.params.risk_aversion_pct,
                f64::from(ctx.view.hp.max(1)),
            ) > 0.0
        };
        if in_reach_threat {
            self.rule = "contact: a monster in reach";
            return Mode::Engage;
        }
        // Food before the timer reaches starvation, but after contact and
        // recovery.
        if ctx
            .world
            .character
            .status
            .as_ref()
            .is_some_and(|s| s.food < ctx.params.eat_below_food)
            && Self::has_food(ctx)
            && self.cooldowns.get(&Mode::Eat).is_none_or(|&t| turn >= t)
        {
            self.rule = "food below the threshold";
            return Mode::Eat;
        }
        // Survival reserves outrank an already-live travel goal: arriving
        // in town with shopping to do must not immediately descend again
        // (GOALS principle 7). A completed or cooled town goal ends here.
        let town_cooled = self.cooldowns.get(&Mode::Town).is_none_or(|&t| turn >= t);
        if ctx.world.level.depth == 0 && town_cooled && Self::town_job(ctx).is_some() {
            self.rule = "in town with a job to do";
            return Mode::Town;
        }
        if self.goal.as_ref().is_some_and(|g| g.mode == Mode::Town) {
            self.goal = None;
        }
        // Readiness (T5.1, #125; c15 DEPTH-04): a character no longer ready
        // for the depth it stands on goes up, as starvation does: Return
        // takes the up staircase, reads a known recall in safety, or
        // explores for a way up. It arrives here after a trapdoor, a
        // spent kit gate (cures, recall) or a level that outgrew it.
        let depth = ctx.world.level.depth;
        if depth > 1
            && Self::adjacent(ctx).is_empty()
            && self.cooldowns.get(&Mode::Return).is_none_or(|&t| turn >= t)
            && !Self::ready_for(ctx, depth)
        {
            self.rule = "not ready for this depth: go up";
            return Mode::Return;
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
                Mode::Eat => i64::from(ctx.params.explore_budget_turns),
                Mode::Return => i64::from(ctx.params.explore_budget_turns),
                Mode::Stuck => 0,
            };
            if !goal.stalled(turn, budget)
                && self.cooldowns.get(&goal.mode).is_none_or(|&t| turn >= t)
            {
                self.rule = "a live goal keeps its mode";
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
        let stairs_up = ctx
            .paths
            .map(ctx.world, ctx.knowledge, &PathGoal::Stairs { up: true })
            .remaining(ctx.world, ctx.view.x, ctx.view.y);
        let cooled = |m: Mode| self.cooldowns.get(&m).is_none_or(|&t| turn >= t);
        let on_stair =
            ctx.world.level.at(ctx.view.x, ctx.view.y).is_some_and(|g| {
                matches!(g.feat, crate::borg::feat::LESS | crate::borg::feat::MORE)
            });
        // The dive rule (c15 DEPTH-04): ready for five levels deeper, the
        // stairs come before the frontier; ready only for the next level,
        // explore first, then descend; not ready for the next, the
        // explored level rerolls (Descend's step) instead of descending.
        let depth = ctx.world.level.depth;
        let ready_next = depth == 0 || Self::ready_for(ctx, depth + 1);
        let dive = depth > 0 && Self::ready_for(ctx, depth + 5);
        let (mode, rule) =
            if descend_first && stairs != UNREACHED && cooled(Mode::Descend) && !block_descend {
                (Mode::Descend, "in town: take the stairs down")
            } else if dive && stairs != UNREACHED && cooled(Mode::Descend) && !block_descend {
                (Mode::Descend, "ready for five levels deeper: dive")
            } else if frontier != UNREACHED && cooled(Mode::Explore) {
                (Mode::Explore, "a frontier is reachable")
            } else if stairs != UNREACHED && cooled(Mode::Descend) && !block_descend && ready_next {
                (Mode::Descend, "no frontier: a down staircase is reachable")
            } else if stairs == UNREACHED && stairs_up == UNREACHED && frontier == UNREACHED {
                // Nothing is reachable anywhere — a sealed room. Rerolling a
                // fresh level from here is stair-scumming out of a trap
                // (GOALS 9): dig the way out instead (T4.6, #123). The up
                // staircase counts too: Descend's reroll travels up, so a
                // reachable up stair must not read as stuck.
                (Mode::Stuck, "nothing reachable: a sealed room")
            } else if !block_descend && (on_stair || cooled(Mode::Descend)) {
                // On a staircase, Descend's step takes it (including the up
                // stair under threat). With none and a cooling Descend, name
                // the stuck state instead of re-choosing the cooled mode every
                // think (M3 re-review, 2026-11-10).
                (
                    Mode::Descend,
                    "on a staircase, or descend cooled with nothing else",
                )
            } else {
                (Mode::Stuck, "every way on is cooling or banned")
            };
        self.rule = rule;
        mode
    }

    /// Fail the current goal when the executor reported the previous
    /// action refused: no progress is being made even though game time
    /// may be passing.
    fn note_failures(&mut self, ctx: &DecisionCtx) {
        let turn = ctx.view.turn;
        self.refused.retain(|_, until| turn < *until);
        if ctx.failures.is_empty() {
            return;
        }
        let until = turn + i64::from(ctx.params.goal_cooldown_turns);
        self.cooldowns.insert(self.mode, until);
        if let Some(action) = self.last_action {
            self.refused.insert(action, until);
        }
        self.goal = None;
    }
}

impl Strategy for FirstStrategy {
    fn name(&self) -> &'static str {
        "first"
    }

    fn decide(&mut self, ctx: &mut DecisionCtx) -> Decision {
        self.rule = "";
        let mut d = self.decide_now(ctx);
        self.last_action = Some(d.action);
        let turn = ctx.view.turn;
        d.why = Some(crate::trace::Why {
            rule: if self.rule.is_empty() && !can_act(ctx.world) {
                "paralysed: no action".into()
            } else {
                self.rule.into()
            },
            cooling: self
                .cooldowns
                .iter()
                .filter(|(_, &until)| turn < until)
                .map(|(m, _)| m.name().to_string())
                .collect(),
            goal_started: self.goal.as_ref().map(|g| g.started),
            goal_progressed: self.goal.as_ref().map(|g| g.progressed),
        });
        d
    }
}

impl FirstStrategy {
    fn decide_now(&mut self, ctx: &mut DecisionCtx) -> Decision {
        if !can_act(ctx.world) {
            let action = Self::still(ctx);
            return Decision {
                action,
                mode: "disabled",
                goal: "paralysed",
                budget_left: 0,
                candidates: Vec::new(),
                threat: Self::here_threat(ctx),
                why: None,
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
                why: None,
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
                why: None,
                violations,
            };
        }

        self.last_candidates.clear();
        // A better item in the pack is worn first, when nothing is near
        // and the grid is quiet (T5.2, #126: valued by effect,
        // `equipment.rs`). Under threat the tactical layer decides (its
        // own weapon swap); here the swap costs a turn nothing contests.
        let quiet = mode != Mode::Engage
            && Self::adjacent(ctx).is_empty()
            && danger_with(
                ctx.world,
                ctx.knowledge,
                ctx.view.x,
                ctx.view.y,
                ctx.params.threat_horizon,
                ctx.params.risk_aversion_pct,
            ) == 0;
        // A refused wield is barred like any refused action (#137), so a
        // swap the engine will not make cannot loop.
        let swap = if quiet {
            crate::equipment::best_swap(ctx.world, ctx.knowledge)
                .filter(|s| !self.refused.contains_key(&Action::Wield { item: s.item }))
        } else {
            None
        };
        if swap.is_some() {
            self.rule = "a better item in the pack: wear it";
        }
        // No light outranks the swap and does not wait for a quiet grid:
        // unlit, the Borg cannot read its escapes or see the frontier it
        // walks to (#172). Only a monster in reach comes first (Engage).
        let light = (mode != Mode::Engage && Self::adjacent(ctx).is_empty())
            .then(|| crate::equipment::light_to_wield(ctx.world, ctx.knowledge))
            .flatten()
            .filter(|&item| !self.refused.contains_key(&Action::Wield { item }));
        if light.is_some() {
            self.rule = "no light: wield one from the pack";
        }
        // Bleeding out (#175): the poison and cuts already on the character
        // will take every hit point left, and nothing regenerates while
        // they last (`danger::pending_bleed`). Resting cannot save it; a
        // cure can. Only a monster in reach comes first.
        let cure = (light.is_none() && mode != Mode::Engage && Self::adjacent(ctx).is_empty())
            .then(|| Self::bleed_cure(ctx))
            .flatten()
            .filter(|&item| !self.refused.contains_key(&Action::Quaff { item }));
        if cure.is_some() {
            self.rule = "bleeding out: drink a cure";
        }
        let wield = light.or(swap.map(|s| s.item));
        // A spell to learn is studied when the grid is quiet, after the
        // light and before an equipment swap (T5.3, #127; `spells.rs`).
        let study = if quiet && light.is_none() {
            crate::spells::study(ctx.world, ctx.knowledge).filter(|a| !self.refused.contains_key(a))
        } else {
            None
        };
        if study.is_some() {
            self.rule = "a spell to learn: study it";
        }
        let wield = if study.is_some() { None } else { wield };
        let chosen = match cure {
            Some(item) => Some(Action::Quaff { item }),
            None => study.or(wield.map(|item| Action::Wield { item })),
        };
        let action = match chosen {
            Some(a) => a,
            None => match self.mode_step(ctx, mode) {
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
                    let around =
                        |dx: i32, dy: i32| ctx.world.level.at(x + dx, y + dy).map(|g| g.feat);
                    stand_still(here, around)
                }
            },
        };
        // Confused, a step goes in a random direction three times in four
        // (`player-util.c:player_confuse_dir:1358-1361`). Beside lava
        // (`FEAT_LAVA`, which burnt full-1-mage-4 from 11 HP to -150 in the
        // town, #183), a walk outside a fight waits the confusion out.
        let confused = ctx
            .world
            .character
            .status
            .as_ref()
            .is_some_and(|s| s.timed.get(4).copied().unwrap_or(0) > 0);
        let (x, y) = (ctx.view.x, ctx.view.y);
        let lava_near = DIRS.iter().any(|&(_, dx, dy)| {
            ctx.world
                .level
                .at(x + dx, y + dy)
                .is_some_and(|g| g.feat == crate::borg::feat::LAVA)
        });
        let action =
            if confused && lava_near && mode != Mode::Engage && matches!(action, Action::Walk(_)) {
                self.rule = "confused beside lava: wait it out";
                let here = ctx.world.level.at(x, y).map(|g| g.feat);
                let around = |dx: i32, dy: i32| ctx.world.level.at(x + dx, y + dy).map(|g| g.feat);
                stand_still(here, around)
            } else {
                action
            };
        let goal_name = match mode {
            Mode::Recover => "rest to full",
            Mode::Engage => "win the exchange",
            Mode::Town => "restock and identify",
            Mode::Descend => "reach the down staircase",
            Mode::Explore => "reveal the frontier",
            Mode::Eat => "eat before starvation",
            Mode::Return => "return home",
            Mode::Stuck => "dig or search a way out",
        };
        Decision {
            action,
            mode: mode.name(),
            goal: goal_name,
            budget_left,
            candidates: std::mem::take(&mut self.last_candidates),
            threat: Self::here_threat(ctx),
            why: None,
            violations,
        }
    }
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
                tval: 9, // TV_SWORD (a dagger)
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
            s.food = 5000; // Fed (above Hungry, 1500)
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
            s.food = 5000; // Fed (above Hungry, 1500)
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
        let t = d.threat.clone().expect("an assessment on every decision");
        assert!(t.expected > 0.0 && t.worst >= t.expected, "{t:?}");
        assert_eq!(t.horizon, params.threat_horizon);
    }

    #[test]
    fn a_refused_engage_action_is_not_chosen_again() {
        // #137: contact returns Engage before any cooldown, so a response
        // the executor refused must be barred by itself, or the next think
        // repeats it (a confused rogue's Phase Door, D1667; refused
        // attacks in the M3 baseline).
        let (mut s, w, k, mut p, m, mut rng, params) =
            ctx_of(&["#######", "#@M...#", "#.....#", "#######"], 20, 20);
        let first = decide_with(
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
        assert_eq!(first.mode, "engage");
        let refused = ["after it: neither moved the character nor took game time".to_string()];
        let second = decide_with(
            &mut s,
            &w,
            &k,
            &mut p,
            &m,
            &mut rng,
            &params,
            &view(&w),
            &refused,
        );
        assert_eq!(second.mode, "engage", "contact still answers");
        assert_ne!(
            second.action, first.action,
            "the refusal changed the decision"
        );
        // Once the cooldown is over the action may be tried again.
        let mut later = w.clone();
        later.turn += i64::from(params.goal_cooldown_turns);
        let mut p2 = Paths::new_with_risk(&later, params.risk_aversion_pct);
        let third = decide_with(
            &mut s,
            &later,
            &k,
            &mut p2,
            &m,
            &mut rng,
            &params,
            &view(&later),
            &[],
        );
        assert_eq!(third.action, first.action, "the bar ends with the cooldown");
    }

    #[test]
    fn an_unready_character_goes_up() {
        // T5.1 (#125), c15 DEPTH-04: at DL6 without the DL5 kit (Word of
        // Recall, three cures, a ranged attack) the Borg is not ready for
        // the depth it stands on, and goes up.
        let (mut s, mut w, k, mut p, m, mut rng, params) = ctx_of(
            &["#########", "#@.....<#", "#.......#", "#########"],
            20,
            20,
        );
        w.level.depth = 6;
        if let Some(st) = w.character.status.as_mut() {
            st.depth = 6;
            st.max_depth = 6;
        }
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
        assert_eq!(d.mode, "return", "{d:?}");
        assert_eq!(
            d.why.as_ref().map(|y| y.rule.as_str()),
            Some("not ready for this depth: go up")
        );
        // With the kit (and the band's monsters well inside its HP) it
        // stays to explore instead.
        let (mut s, mut w, mut k, mut p, m, mut rng, params) = ctx_of(
            &["#########", "#@.....<#", "#.......#", "#########"],
            200,
            200,
        );
        w.level.depth = 6;
        if let Some(st) = w.character.status.as_mut() {
            st.depth = 6;
            st.max_depth = 6;
        }
        // The ranged attack is five flasks of oil (tval 27).
        k.kinds.push(crate::knowledge::ObjectKind {
            index: 7,
            name: "& Flask~ of oil".into(),
            tval: 27,
            ..Default::default()
        });
        let mut oil = item(1, 5, true);
        oil.kind = 7;
        oil.tval = 27;
        w.character.inventory = vec![item(5, 1, true), item(3, 3, true), oil];
        let r = crate::readiness::ready(&w, &k, &params, 6);
        assert!(r.ok(), "{r:?}");
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
        assert_ne!(d.mode, "return", "{d:?}");
    }

    #[test]
    fn only_known_food_is_food() {
        // #165: has_food and the Eat step skip an item the character is
        // not aware of, like every other item path.
        let (_, mut w, k, mut p, m, mut rng, params) =
            ctx_of(&["#######", "#@....#", "#######"], 20, 20);
        w.character.inventory = vec![item(1, 2, false)];
        let v = view(&w);
        let ctx = DecisionCtx {
            view: &v,
            world: &w,
            knowledge: &k,
            paths: &mut p,
            mem: &m,
            rng: &mut rng,
            params: &params,
            failures: &[],
        };
        assert!(!FirstStrategy::has_food(&ctx), "unknown is not food");
        let (_, mut w, k, mut p, m, mut rng, params) =
            ctx_of(&["#######", "#@....#", "#######"], 20, 20);
        w.character.inventory = vec![item(1, 2, true)];
        let v = view(&w);
        let ctx = DecisionCtx {
            view: &v,
            world: &w,
            knowledge: &k,
            paths: &mut p,
            mem: &m,
            rng: &mut rng,
            params: &params,
            failures: &[],
        };
        assert!(FirstStrategy::has_food(&ctx), "a known ration is");
    }

    #[test]
    fn a_harmless_monster_in_reach_is_not_a_fight() {
        // #183, full-4-necro-3: Farmer Maggot (no blows) next to the Borg
        // held it in Engage until it starved. A monster with blows still
        // engages.
        let mode = |harmless: bool| {
            let (mut s, w, mut k, mut p, m, mut rng, params) =
                ctx_of(&["#######", "#@M...#", "#.....#", "#######"], 20, 20);
            if harmless {
                k.races[1].blows.clear();
            }
            let v = view(&w);
            let mut ctx = DecisionCtx {
                view: &v,
                world: &w,
                knowledge: &k,
                paths: &mut p,
                mem: &m,
                rng: &mut rng,
                params: &params,
                failures: &[],
            };
            s.decide(&mut ctx).mode
        };
        assert_eq!(mode(false), "engage");
        assert_ne!(mode(true), "engage");
    }

    #[test]
    fn a_confused_character_beside_lava_does_not_walk() {
        // #183, full-1-mage-4: confused in the town, a random step into
        // lava. The map's `~` is not a feature the test world knows, so
        // the lava is set by hand east of the character.
        let run = |confused: bool| {
            let (mut s, mut w, k, mut p, m, mut rng, params) =
                ctx_of(&["#######", "#@...?#", "#.....#", "#######"], 20, 20);
            if let Some(g) = w.level.at_mut(1, 2) {
                g.feat = crate::borg::feat::LAVA;
            }
            if let Some(st) = w.character.status.as_mut() {
                if st.timed.len() <= 4 {
                    st.timed.resize(5, 0);
                }
                st.timed[4] = if confused { 5 } else { 0 };
            }
            let v = view(&w);
            let mut ctx = DecisionCtx {
                view: &v,
                world: &w,
                knowledge: &k,
                paths: &mut p,
                mem: &m,
                rng: &mut rng,
                params: &params,
                failures: &[],
            };
            let a = s.decide(&mut ctx).action;
            (a, s.rule)
        };
        let (a, _) = run(false);
        assert!(matches!(a, Action::Walk(_)), "{a:?}");
        let (a, rule) = run(true);
        assert!(!matches!(a, Action::Walk(_)), "{a:?}");
        assert_eq!(rule, "confused beside lava: wait it out");
    }

    #[test]
    fn a_breeder_colony_sends_the_borg_home() {
        // #183: four breeders seen on DL1, none in reach, a hurt character:
        // it leaves (Return) instead of resting beside a growing colony.
        // Three, below the colony, and it rests.
        let decide = |map: &[&str]| {
            let (mut s, mut w, mut k, mut p, m, mut rng, params) = ctx_of(map, 5, 20);
            if k.races.len() < 2 {
                k.races.resize(2, Default::default());
                k.races[1].index = 1;
            }
            // MULTIPLY is RF 17: byte 2, bit 0.
            k.races[1].flags = vec![0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0];
            w.level.depth = 1;
            if let Some(st) = w.character.status.as_mut() {
                st.depth = 1;
            }
            let v = view(&w);
            let mut ctx = DecisionCtx {
                view: &v,
                world: &w,
                knowledge: &k,
                paths: &mut p,
                mem: &m,
                rng: &mut rng,
                params: &params,
                failures: &[],
            };
            s.decide(&mut ctx).mode
        };
        assert_eq!(
            decide(&[
                "############",
                "#@.....MM..#",
                "#......MM..#",
                "############"
            ]),
            "return"
        );
        assert_eq!(
            decide(&[
                "############",
                "#@.....MM..#",
                "#......M...#",
                "############"
            ]),
            "recover"
        );
    }

    #[test]
    fn a_character_bleeding_out_drinks_a_cure_before_resting() {
        // #175, smoke-mage-2 (D1848): poisoned for more turns than its hit
        // points, it rested to death. With a Cure Light Wounds (kind 3) in
        // the pack it drinks it; with its poison below its hit points it
        // does not.
        let poison = |turns: i16| {
            let (mut s, mut w, k, mut p, m, mut rng, params) =
                ctx_of(&["#######", "#@....#", "#######"], 6, 20);
            w.character.inventory = vec![item(3, 1, true)];
            if let Some(st) = w.character.status.as_mut() {
                if st.timed.len() <= 7 {
                    st.timed.resize(8, 0);
                }
                st.timed[7] = turns;
                st.stat_ind[4] = 0;
            }
            let v = view(&w);
            let mut ctx = DecisionCtx {
                view: &v,
                world: &w,
                knowledge: &k,
                paths: &mut p,
                mem: &m,
                rng: &mut rng,
                params: &params,
                failures: &[],
            };
            s.decide(&mut ctx).action
        };
        assert_eq!(poison(10), Action::Quaff { item: 0 });
        assert_ne!(poison(3), Action::Quaff { item: 0 });
    }

    #[test]
    fn engage_progress_without_contact_is_not_a_dead_monster() {
        // #139: with no adjacent monster the Engage measure is its own
        // sentinel; 0 would read as a monster at 0 HP.
        let (_, w, k, mut p, m, mut rng, params) =
            ctx_of(&["#######", "#@....#", "#######"], 20, 20);
        let v = view(&w);
        let ctx = DecisionCtx {
            view: &v,
            world: &w,
            knowledge: &k,
            paths: &mut p,
            mem: &m,
            rng: &mut rng,
            params: &params,
            failures: &[],
        };
        assert_eq!(
            FirstStrategy::measure(Mode::Engage, &ctx),
            ENGAGE_NO_CONTACT
        );
        let (_, w, k, mut p, m, mut rng, params) =
            ctx_of(&["#######", "#@M...#", "#######"], 20, 20);
        let v = view(&w);
        let ctx = DecisionCtx {
            view: &v,
            world: &w,
            knowledge: &k,
            paths: &mut p,
            mem: &m,
            rng: &mut rng,
            params: &params,
            failures: &[],
        };
        let hp = i64::from(w.level.monsters[0].hp);
        assert_eq!(FirstStrategy::measure(Mode::Engage, &ctx), hp);
    }

    #[test]
    fn the_trace_names_the_rule_and_the_goal_timing() {
        // #138: the mode is chosen by a priority rule, so the trace names
        // the rule; only scored alternatives are candidates; Goal.started
        // and Goal.progressed give the goal's age and stall.
        let (mut s, w, k, mut p, m, mut rng, params) = ctx_of(
            &["#########", "#@......#", "#.......#", "#########"],
            20,
            20,
        );
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
        let why = d.why.expect("the first strategy says why");
        assert!(!why.rule.is_empty(), "{why:?}");
        assert!(
            d.candidates.is_empty(),
            "no synthetic candidate: {:?}",
            d.candidates
        );
        assert_eq!(why.goal_started, Some(w.turn));
        assert_eq!(why.goal_progressed, Some(w.turn));

        let (mut s, w, k, mut p, m, mut rng, params) =
            ctx_of(&["#######", "#@M...#", "#.....#", "#######"], 20, 20);
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
            d.why.as_ref().map(|y| y.rule.as_str()),
            Some("contact: a monster in reach")
        );
        assert!(d.candidates.len() > 1, "contact scores its responses");

        // A refusal cools the mode, and the trace says so.
        let refused = ["refused".to_string()];
        let d = decide_with(
            &mut s,
            &w,
            &k,
            &mut p,
            &m,
            &mut rng,
            &params,
            &view(&w),
            &refused,
        );
        assert_eq!(d.why.expect("why").cooling, vec!["engage".to_string()]);
    }

    #[test]
    fn a_contact_decision_records_every_response() {
        let (mut s, w, k, mut p, m, mut rng, params) =
            ctx_of(&["#######", "#@M...#", "#.....#", "#######"], 20, 20);
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
        assert!(
            d.candidates.len() > 1,
            "the trace has the real alternatives: {:?}",
            d.candidates
        );
        assert_eq!(d.candidates[0].0, d.action);
    }

    #[test]
    fn a_sealed_room_escalates_the_stuck_state_to_digging() {
        // Nothing is reachable anywhere, so the first think already digs
        // (T4.6, #123): granite is one step away in every direction and
        // carries HURT_ROCK (engine `list-terrain.h:34`).
        let (mut s, w, k, mut p, m, mut rng, params) = ctx_of(&["#####", "#@..#", "#####"], 20, 20);
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
        assert_eq!(d.mode, "stuck", "{d:?}");
        assert_eq!(d.goal, "dig or search a way out");
        assert_eq!(d.action, Action::Alter(1), "the first think digs");
    }

    #[test]
    fn starving_with_no_food_returns_home_instead_of_descending() {
        // Food timer at starvation, no food in the pack, a down staircase
        // in the room: the Borg heads home, never deeper (T4.6, #123;
        // bbc D0009, rust D1420).
        let (mut s, w, k, mut p, m, mut rng, params) = ctx_of(&["#####", "#@.>#", "#####"], 20, 20);
        let mut w = w;
        w.character.status.as_mut().unwrap().food = 0;
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
        assert_eq!(d.mode, "return", "{d:?}");
        assert_eq!(d.goal, "return home");
        assert_ne!(d.action, Action::Descend);
    }

    #[test]
    fn travel_never_steps_onto_a_more_dangerous_grid() {
        // The frontier lies beyond a kobold in a one-wide corridor: the
        // straight step north lands in its reach, so travel refuses it
        // and retreats instead (T4.6, #123).
        let (mut s, w, k, mut p, m, mut rng, params) = ctx_of(
            &[
                "#####", "#?..#", "#...#", "#M..#", "#...#", "#@..#", "#...#", "#####",
            ],
            20,
            20,
        );
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
        assert_eq!(d.mode, "explore", "{d:?}");
        assert!(
            matches!(d.action, Action::Walk(d) if !(7..=9).contains(&d)),
            "not north toward the kobold: {:?}",
            d.action
        );
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
        assert_ne!(d.action, Action::Rest(75), "never rest beside a monster");
        assert!(
            matches!(d.action, Action::Attack(4) | Action::Walk(6)),
            "fight or step, not rest: {:?}",
            d.action
        );
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
        assert_ne!(
            d.action,
            Action::Walk(6),
            "walking into a heavy blow's free hit is not an escape"
        );
        assert_ne!(d.action, Action::Rest(75), "never rest beside a monster");
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
    fn a_staircase_under_fire_is_a_scored_escape() {
        // On the down stair with a monster adjacent, the staircase is one
        // escape on the ladder (T4.4, #121): scored by its declared arrival
        // risk against the exchange, not refused by rule. Here the stair
        // beats the exchange.
        let (mut s, mut w, k, _, m, mut rng, params) =
            ctx_of(&["######", "#>M@.#", "######"], 20, 20);
        if let Some(s) = w.character.status.as_mut() {
            s.x = 1;
            s.y = 1;
        }
        let mut p = crate::path::Paths::new(&w);
        let v = view(&w);
        let d = decide_with(&mut s, &w, &k, &mut p, &m, &mut rng, &params, &v, &[]);
        assert_eq!(d.action, Action::Descend, "the stair beats the exchange");
    }

    #[test]
    fn an_escape_descent_still_counts_against_the_reroll_cap() {
        // The ladder's escape descent is a level change like any other:
        // two of them at one game turn trip the generation-loop backstop
        // (M3 review, blocking finding 1) even though the escape is scored.
        let (mut s, mut w, k, _, mut m, mut rng, params) =
            ctx_of(&["########", "#@>F...#", "#......#", "########"], 4, 20);
        if let Some(s) = w.character.status.as_mut() {
            s.x = 2;
            s.y = 1;
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
        assert_eq!(d.action, Action::Descend, "the escape descent is taken");
        m.record(w.turn, Action::Descend);
        m.record(w.turn, Action::Ascend);
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
        assert_eq!(d.action, Action::Hold, "the reroll cap stands");
        assert!(
            d.violations.iter().any(|v| v.contains("reroll cap")),
            "{:?}",
            d.violations
        );
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
