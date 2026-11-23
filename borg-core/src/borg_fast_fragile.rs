//! #842: fast-fragile monsters, by their stats and by name.
//!
//! The owner (2026-11-21): "The only sensible thing to do with fragile
//! characters, when we see them, is to attack them with all that we have
//! got, unless we want a phase door or a stair." A fast-fragile monster is
//! one that moves faster than the Borg, dies in a turn or two of the Borg's
//! best attack, and hurts enough per Borg turn to matter. The named entries
//! of the scary list (`fast-fragile`: crow, wild cat) stay as overrides: a
//! name decides its monster, characterization or not.
//!
//! Three switches, each shipped 0 (today's play):
//! `fast_fragile_policy` (attack without the "target cannot kill me" gate,
//! with Phase Door or the stairs underfoot as the only alternative),
//! `fast_fragile_by_stats` (the characterization below, besides the names)
//! and `fast_fragile_no_heal` (no healing potion in contact with one).
//!
//! #926 adds three more, each 0 by default (today's play): the heal ban
//! lifts at or below an HP floor (`fast_fragile_heal_floor`); "the next turn
//! kills" reads an upper quantile of the damage distribution instead of the
//! mean (`fast_fragile_escape_quantile`); and it counts the Borg's own cut
//! and poison ticks (`fast_fragile_count_dot`).

use crate::borg_caution::{fight_incoming_with, forecast_monster_speed};
use crate::borg_cave_view::BORG_VIEW;
use crate::borg_escape::{phase_door, PhaseCheck, PhaseSite};
use crate::borg_fight_attack::{
    attack_on_sight_reason, focused_attack, observed_kill_hp, FocusedAttack,
};
use crate::borg_headers::borg_trait_h::*;
use crate::borg_io::{borg_keypress, borg_note};
use crate::engine;
use crate::headers::list_mon_race_flags_h::{RF_MULTIPLY, RF_NEVER_BLOW, RF_NEVER_MOVE, RF_UNIQUE};
use crate::headers::list_player_flags_h::PF_ROCK;
use crate::headers::list_player_timed_h::{TMD_CUT, TMD_POISONED};
use crate::headers::list_terrain_h::{FEAT_LESS, FEAT_MORE};
use crate::headers::list_tvals_h::{TV_POTION, TV_SCROLL};
use crate::headers::ui_event_h::keycode_t;
use crate::params::{value as param, Param};
use crate::state::BorgState;

/// The reason the scary list gives the named entries.
pub(crate) const REASON: &str = "fast-fragile";

/// `fast_fragile_policy`: attack fast-fragile monsters ungated, with Phase
/// Door or the stairs underfoot as the only alternative.
pub(crate) fn policy() -> bool {
    param(Param::fast_fragile_policy) != 0
}

/// How a monster came to be marked.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Source {
    /// A `fast-fragile` entry of the scary list.
    Named,
    /// The characterization by stats.
    Stats,
}

/// The numbers of one monster against one character; everything the
/// characterization reads, so it can be tested without the engine.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Facts {
    /// The monster's speed minus the Borg's, in speed points (+10 = twice
    /// the energy at normal speed).
    pub speed_over: i32,
    /// HP the Borg still has to deal to kill it.
    pub need_hp: f64,
    /// What the Borg's best attack deals per Borg turn (hit chance, failure
    /// and the cap at the monster's HP included).
    pub per_turn: f64,
    /// The monster's expected damage per Borg turn.
    pub incoming: f64,
    pub cur_hp: i32,
    /// Monsters of the same race in view besides this one.
    pub others: i32,
}

/// Faster than the Borg by `speed_margin` speed points (monsters spread by
/// about one point around their race's speed, mon-make.c:1158-1164, so +1 is
/// no advantage), killed in `kill_turns` of its best attack, hurting
/// at least `damage_percent` of the current HP per Borg turn, and not one of
/// a swarm (more than `group_max` others of its race in view). Mobility,
/// uniqueness and breeding are the caller's (race flags).
pub(crate) fn verdict(f: &Facts, speed_margin: i32, kill_turns: i32, damage_percent: i32, group_max: i32) -> bool {
    f.speed_over >= speed_margin.max(1)
        && f.per_turn > 0.0
        && f.need_hp <= f.per_turn * f64::from(kill_turns)
        && f.cur_hp > 0
        && f.incoming * 100.0 >= f64::from(damage_percent) * f64::from(f.cur_hp)
        && f.others <= group_max
}

/// The next turn kills the Borg (the expected damage reaches `percent` of
/// its HP: 100 is the mean, less leaves room for the spread of four bites)
/// and the kill needs more than one more turn: the case for Phase Door or the
/// stairs. 0 turns the alternative off.
pub(crate) fn escape_wanted(incoming: f64, cur_hp: i32, need_hp: f64, per_turn: f64, percent: i32) -> bool {
    percent > 0 && incoming * 100.0 >= f64::from(percent) * f64::from(cur_hp) && (per_turn <= 0.0 || need_hp > per_turn)
}

/// #926: the distribution of one blow's damage, `pmf[d]` = P(d HP): a hit
/// (`permyriad` in 10,000: `check_hit`, mon-attack.c:527-533, against
/// AC + to_a, through `hit_chance`, player-attack.c:196-215) rolls
/// `base + dice d sides` (`randcalc`, mon-attack.c:619, z-rand.c:502; `m_bonus` enters at
/// its mean, `bonus`), times 4/3 for a vulnerable element
/// (project-player.c:72-73); a miss deals 0. Armour's cut of HURT damage
/// (`adjust_dam_armor`, mon-attack.c:540-543) is ignored, as in the mean.
pub(crate) fn blow_pmf(base: i32, dice: i32, sides: i32, bonus: i32, permyriad: i32, vulnerable: bool) -> Vec<f64> {
    let hit = f64::from(permyriad.clamp(0, 10_000)) / 10_000.0;
    let mut roll = vec![1.0];
    for _ in 0..dice.max(0) {
        if sides <= 0 { break; }
        let die = std::iter::once(0.0).chain(std::iter::repeat_n(1.0 / f64::from(sides), sides as usize)).collect::<Vec<_>>();
        roll = convolve(&roll, &die);
    }
    let shift = (base + bonus).max(0) as usize;
    let mut pmf = vec![1.0 - hit];
    for (d, &p) in roll.iter().enumerate() {
        if hit * p == 0.0 { continue; }
        let dam = d + shift;
        let dam = if vulnerable { dam * 4 / 3 } else { dam };
        if pmf.len() <= dam { pmf.resize(dam + 1, 0.0); }
        pmf[dam] += hit * p;
    }
    pmf
}

/// The distribution of the sum of two independent damages.
pub(crate) fn convolve(a: &[f64], b: &[f64]) -> Vec<f64> {
    if a.is_empty() || b.is_empty() { return a.iter().chain(b).copied().collect(); }
    let mut out = vec![0.0; a.len() + b.len() - 1];
    for (i, &x) in a.iter().enumerate() {
        if x == 0.0 { continue; }
        for (j, &y) in b.iter().enumerate() {
            out[i + j] += x * y;
        }
    }
    out
}

/// `pmf` summed `n` times.
fn repeat(pmf: &[f64], n: u32) -> Vec<f64> {
    (0..n).fold(vec![1.0], |acc, _| convolve(&acc, pmf))
}

/// #926: one monster's damage before the Borg acts again. A monster gains
/// `turn_energy(speed)` each game turn and acts at 100 (mon-move.c:1938-1945),
/// the Borg likewise (game-world.c:1092), so between two Borg actions it acts
/// `ratio` = its energy over the Borg's times on average: `floor(ratio)` or
/// `ceil(ratio)` times, the latter with probability `frac(ratio)`. Each
/// action is every blow of `action` (make_attack_normal, mon-attack.c:548-).
pub(crate) fn turn_pmf(action: &[f64], ratio: f64) -> Vec<f64> {
    let ratio = ratio.max(0.0);
    let low = ratio.floor();
    let frac = ratio - low;
    let few = repeat(action, low as u32);
    if frac <= 1e-9 { return few; }
    let many = convolve(&few, action);
    let mut out = vec![0.0; many.len()];
    for (d, p) in few.iter().enumerate() { out[d] += (1.0 - frac) * p; }
    for (d, p) in many.iter().enumerate() { out[d] += frac * p; }
    out
}

/// #926: the smallest damage whose cumulative probability reaches `percent`
/// of 100; 100 is the largest damage with any chance at all.
pub(crate) fn quantile(pmf: &[f64], percent: i32) -> f64 {
    if percent >= 100 {
        return pmf.iter().rposition(|&p| p > 1e-12).unwrap_or(0) as f64;
    }
    let want = f64::from(percent.max(0)) / 100.0;
    let mut cdf = 0.0;
    for (d, &p) in pmf.iter().enumerate() {
        cdf += p;
        if cdf + 1e-12 >= want { return d as f64; }
    }
    pmf.len().saturating_sub(1) as f64
}

/// #926: HP the Borg's own cut and poison take before its next action. Each
/// world tick (every ten game turns, game-world.c:1080-1081) poison takes 1
/// (game-world.c:513-519) and a cut 1, 2 for a Severe Cut (timer 101-200),
/// 3 for a Deep Gash or a Mortal Wound (above 200; player_timed.txt grades;
/// game-world.c:522-538), nothing for a rock-skinned player (PF_ROCK).
/// HP per world tick. Damage reduction (player-util.c:167-178) is ignored.
pub(crate) fn dot_per_tick(poisoned: bool, cut: i32, rock: bool) -> i32 {
    let cut_hp = if rock || cut <= 0 { 0 } else if cut > 200 { 3 } else if cut > 100 { 2 } else { 1 };
    i32::from(poisoned) + cut_hp
}

/// #1028 (review of #1068, F2): the world ticks between the Borg's action
/// now (game turn `turn`, energy `energy` before the action) and its next
/// one. Each game turn the player acts first when its energy reaches
/// `move_energy` (process_player, game-world.c:1016-1046), then the world
/// ticks if the turn is a multiple of ten (game-world.c:1080-1081), then the
/// player gains `gain` energy and the turn advances (game-world.c:1091-1095).
/// So the tick of the turn the Borg acts on falls inside the window, and the
/// tick of the turn it acts on next does not. The count is exact, not an
/// average: at +10 (gain 20) the Borg's windows hold one tick and none in
/// turn. An action is taken to cost `move_energy` (most do).
pub(crate) fn window_ticks(turn: i32, energy: i32, gain: i32, move_energy: i32) -> i32 {
    let mut left = energy - move_energy;
    if left >= move_energy { return 0; }
    if gain <= 0 { return 10; }
    let mut t = turn;
    let mut ticks = 0;
    for _ in 0..10_000 {
        if t.rem_euclid(10) == 0 { ticks += 1; }
        left += gain;
        t += 1;
        if left >= move_energy { break; }
    }
    ticks
}

/// #926 (review of #1002): the potion that stops the Borg's own damage over
/// time (lib/gamedata/object.txt): Cure Light Wounds takes 20 off a cut
/// (`TIMED_DEC:CUT` 20, :3168) and does not touch poison; Cure Serious
/// Wounds ends a cut (`CURE:CUT`, :3175-3191); Cure Critical Wounds ends both
/// (:3192-3211); Neutralize Poison ends poison (:3278-3288). The cheapest
/// potion that stops every part that ticks; `None` when none carried does.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum DotCure { Light, Serious, Critical, Neutralize }

impl DotCure {
    /// The potion's name, as the rest of the trace names items (#1028 R4).
    pub(crate) fn name(self) -> &'static str {
        match self {
            DotCure::Light => "Cure Light Wounds",
            DotCure::Serious => "Cure Serious Wounds",
            DotCure::Critical => "Cure Critical Wounds",
            DotCure::Neutralize => "Neutralize Poison",
        }
    }
}

pub(crate) fn dot_cure(poisoned: bool, cut: i32, rock: bool, carried: impl Fn(DotCure) -> bool) -> Option<DotCure> {
    let cut_ticks = cut > 0 && !rock;
    let options: &[DotCure] = match (poisoned, cut_ticks) {
        (false, false) => &[],
        (true, false) => &[DotCure::Neutralize, DotCure::Critical],
        (false, true) if cut <= 20 => &[DotCure::Light, DotCure::Serious, DotCure::Critical],
        (false, true) => &[DotCure::Serious, DotCure::Critical],
        (true, true) => &[DotCure::Critical],
    };
    options.iter().copied().find(|&c| carried(c))
}

/// #926: whether `cur_hp` is at or below `floor` percent of `max_hp` (0 =
/// never): where the heal ban lifts.
pub(crate) fn below_heal_floor(cur_hp: i32, max_hp: i32, floor: i32) -> bool {
    floor > 0 && i64::from(cur_hp) * 100 <= i64::from(floor) * i64::from(max_hp.max(1))
}

fn per_turn(attack: FocusedAttack) -> f64 {
    f64::from(attack.hp_per_turn_milli) / 1000.0
        * f64::from(100 - attack.cost.map_or(0, |c| c.fail_pct)) / 100.0
}

fn viewed(st: &BorgState, i: usize) -> bool {
    let kill = &st.kills.kills[i];
    kill.r_idx != 0
        && st.borg.time.now.wrapping_sub(kill.when) <= param(Param::attack_max_target_age)
        && st.cave.grid(kill.pos.x, kill.pos.y).info & BORG_VIEW != 0
}

/// Whether the character's list names this race in an attack entry (of any
/// reason, active at its level or not): a name overrides the stats.
fn named_in_list(st: &BorgState, r: usize) -> bool {
    crate::scary_list::selected()
        .is_some_and(|list| crate::scary_list::attack_named(list, r, engine::race_base_name(r)))
        || attack_on_sight_reason(st, r).is_some()
}

/// The facts for the characterization, or `None` when a race flag already
/// excludes the monster.
fn facts(st: &mut BorgState, i: usize) -> Option<(Facts, FocusedAttack)> {
    let kill = st.kills.kills[i];
    let r = usize::from(kill.r_idx);
    if engine::race_flag_has(r, RF_UNIQUE)
        || engine::race_flag_has(r, RF_NEVER_MOVE)
        || engine::race_flag_has(r, RF_MULTIPLY)
    {
        return None;
    }
    let speed_over = forecast_monster_speed(&kill) - engine::player_combat_speed();
    if speed_over <= 0 {
        return None;
    }
    let others = (1..st.kills.nxt.max(0) as usize)
        .filter(|&j| j != i && st.kills.kills[j].r_idx == kill.r_idx && viewed(st, j))
        .count() as i32;
    let cur_hp = st.borg.tr(BI_CURHP);
    let incoming = fight_incoming_with(st, i, true).hp_per_turn;
    let attack = focused_attack(st, i)?;
    let need_hp = f64::from(observed_kill_hp(&kill).unwrap_or_else(|| engine::race(r).avg_hp.max(1)));
    Some((
        Facts { speed_over, need_hp, per_turn: per_turn(attack), incoming, cur_hp, others },
        attack,
    ))
}

/// `# Fast-fragile: crow (speed +10 vs +0, 9 HP vs 7.2 per turn, ...)`.
pub(crate) fn describe(st: &BorgState, i: usize, source: Source, f: &Facts) -> String {
    let kill = st.kills.kills[i];
    let borg_speed = engine::player_combat_speed() - 110;
    format!(
        "# Fast-fragile ({}): {} (speed {:+} vs {:+}, {:.0} HP vs {:.1} per turn, {:.1} damage/turn = {}% of HP {}, {} others in view) (#842)",
        if source == Source::Named { "named" } else { "stats" },
        String::from_utf8_lossy(engine::race_name(usize::from(kill.r_idx))),
        f.speed_over + borg_speed, borg_speed, f.need_hp, f.per_turn, f.incoming,
        if f.cur_hp > 0 { (f.incoming * 100.0 / f64::from(f.cur_hp)).round() as i32 } else { 0 },
        f.cur_hp, f.others,
    )
}

/// How the monster `i` is marked, with the numbers and its best attack:
/// `None` when it is not fast-fragile (or the switches that would mark it
/// are off). `by_stats` is `fast_fragile_by_stats`.
pub(crate) fn classify(st: &mut BorgState, i: usize) -> Option<(Source, Facts, FocusedAttack)> {
    let r = usize::from(st.kills.kills[i].r_idx);
    if r == 0 || !viewed(st, i) {
        return None;
    }
    if attack_on_sight_reason(st, r) == Some(REASON) {
        let (f, attack) = facts(st, i).or_else(|| {
            // A named monster stays marked whatever the numbers say (it can
            // be slower than a hasted Borg, say): only its attack is needed.
            let attack = focused_attack(st, i)?;
            let kill = st.kills.kills[i];
            let need_hp = f64::from(observed_kill_hp(&kill).unwrap_or(1));
            Some((Facts {
                speed_over: forecast_monster_speed(&kill) - engine::player_combat_speed(),
                need_hp, per_turn: per_turn(attack),
                incoming: fight_incoming_with(st, i, true).hp_per_turn,
                cur_hp: st.borg.tr(BI_CURHP), others: 0,
            }, attack))
        })?;
        return Some((Source::Named, f, attack));
    }
    if param(Param::fast_fragile_by_stats) == 0 || named_in_list(st, r) {
        return None;
    }
    let (f, attack) = facts(st, i)?;
    verdict(
        &f,
        param(Param::fast_fragile_speed_margin),
        param(Param::fast_fragile_kill_turns),
        param(Param::fast_fragile_damage_percent),
        param(Param::fast_fragile_group_max),
    )
    .then_some((Source::Stats, f, attack))
}

/// Every awake fast-fragile monster within reach of the next turn, not only
/// the target (two cats bite twice as much).
fn incoming_total(st: &mut BorgState, target: usize, f: &Facts) -> f64 {
    let mut incoming = f.incoming;
    for j in 1..st.kills.nxt.max(0) as usize {
        if j == target || st.kills.kills[j].r_idx == 0 || !st.kills.kills[j].awake || !viewed(st, j) {
            continue;
        }
        if engine::distance(st.borg.c, st.kills.kills[j].pos) <= 2 && classify(st, j).is_some() {
            incoming += fight_incoming_with(st, j, true).hp_per_turn;
        }
    }
    incoming
}

/// #926: the monsters `incoming_total` counts, the target first.
fn in_reach(st: &mut BorgState, target: usize) -> Vec<usize> {
    let mut out = vec![target];
    for j in 1..st.kills.nxt.max(0) as usize {
        if j == target || st.kills.kills[j].r_idx == 0 || !st.kills.kills[j].awake || !viewed(st, j) {
            continue;
        }
        if engine::distance(st.borg.c, st.kills.kills[j].pos) <= 2 && classify(st, j).is_some() {
            out.push(j);
        }
    }
    out
}

/// #926: the distribution of the monster `i`'s damage before the Borg acts
/// again, with the blows, hit chances and speeds `fight_incoming_with` takes
/// for its mean.
fn monster_pmf(st: &BorgState, i: usize) -> Vec<f64> {
    let kill = st.kills.kills[i];
    let r = usize::from(kill.r_idx);
    if r == 0 || engine::race_flag_has(r, RF_NEVER_BLOW) { return vec![1.0]; }
    let race = engine::race(r);
    let level = i32::from(race.level);
    let energy = |s: i32| f64::from(engine::extract_energy(s.clamp(0, 199) as usize)).max(1.0);
    let ratio = energy(forecast_monster_speed(&kill)) / energy(engine::player_combat_speed());
    let mut action = vec![1.0];
    for b in 0..usize::from(engine::z_info().mon_blows_max) {
        let Some(blow) = engine::race_blow(r, b) else { break };
        let vulnerable = match blow.effect_name.to_bytes() {
            b"ACID" => st.borg.tr(BI_VACID) != 0,
            b"ELEC" => st.borg.tr(BI_VELEC) != 0,
            b"FIRE" => st.borg.tr(BI_VFIRE) != 0,
            b"COLD" => st.borg.tr(BI_VCOLD) != 0,
            _ => false,
        };
        let chance = crate::borg_fight_attack::hit_permyriad(3 * level.max(1) + blow.hit_power, st.borg.tr(BI_ARMOR));
        // m_bonus at its mean, as blow_average_tenths (MAX_RAND_DEPTH 128).
        let bonus = (f64::from(blow.m_bonus) * f64::from(level) / 128.0).round() as i32;
        action = convolve(&action, &blow_pmf(blow.base, blow.dice, blow.sides, bonus, chance, vulnerable));
    }
    turn_pmf(&action, ratio)
}

/// #926: the Borg's own cut and poison expected before its next action, when
/// `fast_fragile_count_dot` is on; 0 otherwise.
fn own_dot() -> f64 {
    let (per_tick, ticks) = own_dot_parts();
    f64::from(per_tick * ticks)
}

/// The HP a world tick of the Borg's own cut and poison takes, and the world
/// ticks before its next action (`window_ticks`); (0, 0) with
/// `fast_fragile_count_dot` off.
fn own_dot_parts() -> (i32, i32) {
    if param(Param::fast_fragile_count_dot) == 0 { return (0, 0); }
    // The timers as borg_trait reads them (BI_ISPOISONED, BI_ISCUT come from
    // the same player->timed, borg_trait.rs:2852-2858).
    let move_energy = i32::from(engine::z_info().move_energy).max(1);
    let gain = i32::from(engine::extract_energy(engine::player_combat_speed().clamp(0, 199) as usize))
        * move_energy / 100; // turn_energy, game-world.c:136-139
    (dot_per_tick(engine::player_timed(TMD_POISONED) > 0, i32::from(engine::player_timed(TMD_CUT)),
        engine::player_has(PF_ROCK)), window_ticks(engine::game_turn(), engine::player_energy(), gain, move_energy))
}

/// #926 (review of #1002): the carried potion that stops the Borg's own cut
/// and poison, if any.
fn own_dot_cure(st: &mut BorgState) -> Option<(DotCure, i32)> {
    let sval = |st: &BorgState, c: DotCure| match c {
        DotCure::Light => st.item_val.sv_potion_cure_light,
        DotCure::Serious => st.item_val.sv_potion_cure_serious,
        DotCure::Critical => st.item_val.sv_potion_cure_critical,
        DotCure::Neutralize => st.item_val.sv_potion_cure_poison,
    };
    let carried: Vec<DotCure> = [DotCure::Light, DotCure::Serious, DotCure::Critical, DotCure::Neutralize]
        .into_iter()
        .filter(|&c| crate::borg_inventory::borg_slot(st, TV_POTION as i32, sval(st, c)) >= 0)
        .collect();
    dot_cure(engine::player_timed(TMD_POISONED) > 0, i32::from(engine::player_timed(TMD_CUT)),
        engine::player_has(PF_ROCK), |c| carried.contains(&c))
        .map(|c| (c, sval(st, c)))
}

/// #926: the damage "the next turn kills" weighs: the mean of the
/// fast-fragile monsters within reach (`incoming_total`, today), or with
/// `fast_fragile_escape_quantile` above 0 that quantile of their summed
/// distribution; plus the Borg's own cut and poison with
/// `fast_fragile_count_dot`.
fn next_turn_damage(st: &mut BorgState, target: usize, f: &Facts) -> f64 {
    let q = param(Param::fast_fragile_escape_quantile);
    let blows = if q > 0 {
        let monsters = in_reach(st, target);
        let pmf = monsters.iter().fold(vec![1.0], |acc, &j| convolve(&acc, &monster_pmf(st, j)));
        quantile(&pmf, q)
    } else {
        incoming_total(st, target, f)
    };
    blows + own_dot()
}

/// #926: the Borg's own cut and poison kill it before its next action
/// wherever it stands (death is below 0 HP, player-util.c:228): Phase Door
/// and the stairs cannot save it, a cure can. Only with
/// `fast_fragile_count_dot`.
fn dot_kills(cur_hp: i32) -> bool {
    // #1028: the ticks of this window, counted from the turn and the energy
    // (review of #1068, F2), not the expectation, which under-counts a
    // hasted Borg's risk on the windows that hold a tick.
    let (per_tick, ticks) = own_dot_parts();
    per_tick > 0 && per_tick * ticks > cur_hp
}

/// #926: the heal ban lifts at or below `fast_fragile_heal_floor` percent of
/// the maximum HP, or when the Borg's own cut and poison kill it first.
fn ban_lifted(st: &BorgState, cur_hp: i32) -> bool {
    below_heal_floor(cur_hp, st.borg.tr(BI_MAXHP), param(Param::fast_fragile_heal_floor)) || dot_kills(cur_hp)
}

/// Whether `escape_instead` could act now: a staircase underfoot, or a Phase
/// Door scroll or spell that can be used (no reading in the dark, blind or
/// confused; no teleport where it is not allowed). Also #969.
pub(crate) fn escape_available(st: &mut BorgState) -> bool {
    let feat = u32::from(st.cave.grid(st.borg.c.x, st.borg.c.y).feat);
    if (feat == FEAT_LESS && !engine::player_opt(crate::headers::list_options_h::OPT_birth_force_descend)) || feat == FEAT_MORE {
        return true;
    }
    if !crate::borg_escape::borg_allow_teleport(st) {
        return false;
    }
    let can_read = !engine::no_light()
        && st.borg.tr(BI_ISBLIND) == 0 && st.borg.tr(BI_ISCONFUSED) == 0 && st.borg.tr(BI_ISFORGET) == 0;
    (can_read && crate::borg_inventory::borg_slot(st, TV_SCROLL as i32, st.item_val.sv_scroll_phase_door) >= 0)
        || crate::borg_magic::borg_spell_okay_fail(
            st, crate::borg_headers::borg_magic_h::PHASE_DOOR, param(Param::escape_last_ditch_phase_fail))
}

/// A Potion of Cure Light, Serious or Critical Wounds in the pack (also #969).
pub(crate) fn has_cure(st: &mut BorgState) -> bool {
    let svals = [st.item_val.sv_potion_cure_light, st.item_val.sv_potion_cure_serious, st.item_val.sv_potion_cure_critical];
    svals.into_iter().any(|sval| crate::borg_inventory::borg_slot(st, TV_POTION as i32, sval) >= 0)
}

/// The one exception to the heal ban (`fast_fragile_no_heal` = 2; the issue's
/// "the attack cannot kill this turn and no escape is available"): the next
/// turn kills, the kill needs more than one more turn, and neither Phase Door
/// nor a staircase is at hand. A cure is the only move left that does not
/// lose the race at once.
pub(crate) fn cornered(st: &mut BorgState, target: usize, f: &Facts) -> bool {
    param(Param::fast_fragile_no_heal) == 2 && {
        let incoming = next_turn_damage(st, target, f);
        escape_wanted(incoming, f.cur_hp, f.need_hp, f.per_turn, param(Param::fast_fragile_escape_percent))
            && !escape_available(st)
    }
}

/// Attack-on-sight steps aside for the heal when the Borg carries a cure and
/// is `cornered` (#842) or at the heal floor (#926); without one the attack
/// is all it has. `Some(note)` names the reason for the decision trace.
pub(crate) fn yield_to_cure(st: &mut BorgState, target: usize, f: &Facts) -> Option<&'static str> {
    if below_heal_floor(f.cur_hp, st.borg.tr(BI_MAXHP), param(Param::fast_fragile_heal_floor)) {
        let cure = has_cure(st);
        borg_note(st, &format!("# Fast-fragile: HP {}/{}, at the heal floor; a cure is {} (#926)",
            f.cur_hp, st.borg.tr(BI_MAXHP), if cure { "carried" } else { "not carried, the attack goes on" }));
        if cure { return Some("# At the fast-fragile heal floor: the cure decides (#926)"); }
    }
    (cornered(st, target, f) && has_cure(st))
        .then_some("# Cornered by a fast-fragile attacker: no escape, the kill is more than one turn off; the cure decides (#842)")
}

/// Alternative to the attack (owner, #842): when the next turn kills the Borg
/// and the kill needs more than one more turn, only Phase Door or a
/// staircase underfoot. `true` when the turn was spent on one of them.
pub(crate) fn escape_instead(st: &mut BorgState, target: usize, f: &Facts) -> bool {
    // #926: the Borg's own cut and poison kill it before its next action
    // wherever it stands; the potion that stops them comes before the
    // attack, Phase Door and the stairs (review of #1002: the one that stops
    // this damage, not any cure).
    if dot_kills(f.cur_hp) {
        if let Some((cure, sval)) = own_dot_cure(st) {
            if crate::borg_item_use::borg_quaff_potion(st, sval) {
                let (per_tick, ticks) = own_dot_parts();
                borg_note(st, &format!("# Fast-fragile: its own cut and poison ({ticks} world ticks of {per_tick} HP before its next action) take it below 0 from HP {}: {} instead of the attack or Phase Door (#1028)", f.cur_hp, cure.name()));
                return true;
            }
        }
    }
    let incoming = next_turn_damage(st, target, f);
    if !escape_wanted(incoming, f.cur_hp, f.need_hp, f.per_turn, param(Param::fast_fragile_escape_percent)) {
        return false;
    }
    let feat = u32::from(st.cave.grid(st.borg.c.x, st.borg.c.y).feat);
    if feat == FEAT_LESS && !engine::player_opt(crate::headers::list_options_h::OPT_birth_force_descend) {
        borg_note(st, &format!("# Fast-fragile: the next turn kills ({incoming:.1} damage/turn, HP {}), the kill needs {:.1} turns: stairs underfoot (#842)", f.cur_hp, f.need_hp / f.per_turn.max(0.001)));
        borg_keypress(st, keycode_t::from(b'<'));
        return true;
    }
    if feat == FEAT_MORE {
        borg_note(st, &format!("# Fast-fragile: the next turn kills ({incoming:.1} damage/turn, HP {}), the kill needs {:.1} turns: stairs underfoot (#842)", f.cur_hp, f.need_hp / f.per_turn.max(0.001)));
        borg_keypress(st, keycode_t::from(b'>'));
        return true;
    }
    // Through the one Phase Door decision (docs/PHASE-DOOR.md): the owner's
    // adjacent-enemy rule (#584) and the anti-summon gate apply as at every
    // other site. The landing test (the emergency tolerance) is behind
    // `fast_fragile_phase_landing`: at the HP this fires at (3 to 6 against a
    // crow's danger of 18) every landing grid is "scary", 100 of 100, so the
    // test refuses the Phase Door in the fights it is for (8 crow cases: 4
    // dead of 8 with it, 1 without).
    let check = if param(Param::fast_fragile_phase_landing) != 0 {
        PhaseCheck::Landing {
            tolerance: param(Param::escape_emergency_phase_tolerance),
            horizon: param(Param::escape_emergency_phase_horizon),
        }
    } else {
        PhaseCheck::None
    };
    if phase_door(st, PhaseSite::FastFragile, check, None, |st| {
        st.borg.time.now.wrapping_sub(st.borg.time.antisummon) > param(Param::escape_phase_antisummon_delay)
            && crate::borg_escape::borg_allow_teleport(st)
            && (crate::borg_item_use::borg_read_scroll(st, st.item_val.sv_scroll_phase_door)
                || crate::borg_magic::borg_spell_fail(
                    st, crate::borg_headers::borg_magic_h::PHASE_DOOR, param(Param::escape_last_ditch_phase_fail)))
    }) {
        crate::borg_escape::reset_antisummon(st);
        borg_note(st, &format!("# Fast-fragile: the next turn kills ({incoming:.1} damage/turn, HP {}), the kill needs {:.1} turns: Phase Door (#842)", f.cur_hp, f.need_hp / f.per_turn.max(0.001)));
        return true;
    }
    false
}

/// No healing potion while a fast-fragile attacker is in contact (owner,
/// #842: "drink healing potion that against such creatures is a death
/// sentence"): a cure buys a turn or two and loses the race. The exception
/// the issue raises (the attack cannot kill this turn and no escape is
/// available) is not made: the Borg attacks, as before the potion would
/// have been quaffed.
pub(crate) fn heal_banned(st: &mut BorgState) -> bool {
    if param(Param::fast_fragile_no_heal) == 0 || param(Param::attack_on_sight) == 0 {
        return false;
    }
    for i in 1..st.kills.nxt.max(0) as usize {
        let kill = st.kills.kills[i];
        if kill.r_idx == 0 || !kill.awake || engine::distance(st.borg.c, kill.pos) > 1 {
            continue;
        }
        if let Some((source, f, _)) = classify(st, i) {
            if ban_lifted(st, f.cur_hp) {
                borg_note(st, &format!("# Fast-fragile in contact, HP {}/{}: at the heal floor or bleeding out, a cure is allowed (#926)", f.cur_hp, st.borg.tr(BI_MAXHP)));
                continue;
            }
            if cornered(st, i, &f) {
                borg_note(st, "# Cornered by a fast-fragile attacker: no escape, the kill is more than one turn off; a cure is allowed (#842)");
                continue;
            }
            let note = describe(st, i, source, &f);
            borg_note(st, &format!("# No healing potion in contact with a fast-fragile attacker (#842); {}", note.trim_start_matches("# ")));
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The crow against a CL1 Necromancer: speed +10 vs +0, 9 HP, the best
    /// attack deals 4.1 a turn (melee); 6.4 damage a turn against 15 HP.
    fn crow() -> Facts {
        Facts { speed_over: 10, need_hp: 9.0, per_turn: 4.5, incoming: 6.4, cur_hp: 15, others: 0 }
    }

    #[test]
    fn the_crow_is_fast_fragile_at_level_one() {
        assert!(verdict(&crow(), 5, 2, 25, 2));
    }

    #[test]
    fn each_part_of_the_characterization_can_drop_the_category() {
        let v = |f: Facts| verdict(&f, 5, 2, 25, 2);
        assert!(!v(Facts { speed_over: 0, ..crow() }), "not faster than the Borg (Speed drops it)");
        assert!(!v(Facts { speed_over: -10, ..crow() }));
        assert!(!v(Facts { speed_over: 1, ..crow() }), "+1 is the spread of a normal-speed race (the small kobold)");
        assert!(v(Facts { speed_over: 5, ..crow() }));
        assert!(!v(Facts { need_hp: 9.1, ..crow() }), "not killed in two of the best attacks");
        assert!(v(Facts { need_hp: 9.0, per_turn: 4.5, ..crow() }), "exactly two turns is killed in two");
        assert!(!v(Facts { incoming: 3.7, ..crow() }), "under 25% of 15 HP is just a target");
        assert!(v(Facts { incoming: 3.75, ..crow() }), "25% is enough");
        assert!(!v(Facts { cur_hp: 60, ..crow() }), "a CL20 Borg is not threatened by a crow");
        assert!(!v(Facts { per_turn: 0.0, ..crow() }), "no attack at all");
        assert!(!v(Facts { others: 3, ..crow() }), "a swarm is a corridor question");
        assert!(v(Facts { others: 2, ..crow() }), "alone or with at most two others");
    }

    #[test]
    fn the_cave_spider_swarm_is_not_fast_fragile_in_the_open() {
        // 2d8 cave spiders: 7 HP, BITE 1d4, speed +10, eight in view.
        let spider = Facts { speed_over: 10, need_hp: 7.0, per_turn: 4.5, incoming: 4.0, cur_hp: 12, others: 7 };
        assert!(!verdict(&spider, 5, 2, 25, 2));
        assert!(verdict(&Facts { others: 0, ..spider }, 5, 2, 25, 2), "one alone is");
    }

    #[test]
    fn phase_or_stairs_only_when_the_next_turn_kills_and_the_kill_is_not_one_turn() {
        assert!(escape_wanted(6.4, 5, 9.0, 4.5, 100), "HP 5 under 6.4 a turn, two turns to kill");
        assert!(!escape_wanted(6.4, 15, 9.0, 4.5, 100), "the next turn does not kill");
        assert!(!escape_wanted(6.4, 5, 4.0, 4.5, 100), "one attack kills: attack");
        assert!(escape_wanted(6.4, 6, 9.0, 0.0, 100), "nothing to attack with");
        // The Dunadan Priest 1309976551: 7.2 damage a turn against 8 HP.
        assert!(!escape_wanted(7.2, 8, 10.0, 8.1, 100), "the mean does not reach 8 HP");
        assert!(escape_wanted(7.2, 8, 10.0, 8.1, 75), "with room for the spread it does");
        assert!(!escape_wanted(100.0, 1, 10.0, 1.0, 0), "0 = never");
    }
}

#[cfg(test)]
mod tests_926 {
    use super::*;

    /// The wild cat (monster.txt:1133-1147): two CLAW:HURT:1d3 blows, speed
    /// 120 against a normal-speed Borg (energy 20 over 10: two actions), at
    /// 95% a blow against AC 0 (`hit_permyriad`: 12% + 83% x 1).
    fn cat() -> Vec<f64> {
        let claw = blow_pmf(0, 1, 3, 0, 9500, false);
        turn_pmf(&convolve(&claw, &claw), 2.0)
    }

    fn mean(pmf: &[f64]) -> f64 {
        pmf.iter().enumerate().map(|(d, p)| d as f64 * p).sum()
    }

    #[test]
    fn the_wild_cat_distribution_matches_the_engine() {
        let pmf = cat();
        assert!((pmf.iter().sum::<f64>() - 1.0).abs() < 1e-9);
        assert!((mean(&pmf) - 7.6).abs() < 1e-9, "the mean the Borg logged at 1822");
        assert_eq!(pmf.len(), 13, "four claws of at most 3");
        // Dwarf Necromancer 1398933929 at HP 8: dead at 9 or more.
        let dead: f64 = pmf[9..].iter().sum();
        assert!((dead - 0.3181).abs() < 1e-3, "{dead}");
        assert_eq!(quantile(&pmf, 47), 7.0);
        assert_eq!(quantile(&pmf, 50), 8.0, "the median reaches HP 8");
        assert_eq!(quantile(&pmf, 68), 8.0);
        assert_eq!(quantile(&pmf, 69), 9.0, "the first quantile that kills");
        assert_eq!(quantile(&pmf, 100), 12.0, "the maximum");
    }

    #[test]
    fn the_quantile_flips_seed_1398933929_and_the_mean_does_not() {
        let pmf = cat();
        // 10 HP to kill at 5.8 a turn: more than one more turn.
        assert!(!escape_wanted(mean(&pmf), 8, 10.0, 5.8, 100), "today: 7.6 < 8, it attacks");
        assert!(escape_wanted(quantile(&pmf, 50), 8, 10.0, 5.8, 100));
        assert!(escape_wanted(quantile(&pmf, 100), 8, 10.0, 5.8, 100));
        assert!(!escape_wanted(quantile(&pmf, 100), 13, 10.0, 5.8, 100), "at full HP 13 even the maximum does not reach");
    }

    #[test]
    fn a_fractional_energy_ratio_mixes_the_number_of_actions() {
        // Speed +9 (energy 19 over 10, the Priest 1294425391's cat): one
        // action 10% of the time, two 90%.
        let claw = blow_pmf(0, 1, 3, 0, 10_000, false);
        let action = convolve(&claw, &claw);
        let pmf = turn_pmf(&action, 1.9);
        assert!((mean(&pmf) - 1.9 * 4.0).abs() < 1e-9);
        assert!((pmf[2] - 0.1 / 9.0).abs() < 1e-9, "one action, both claws 1");
        assert_eq!(quantile(&pmf, 100), 12.0);
        assert_eq!(turn_pmf(&action, 0.0), vec![1.0], "no action, no damage");
    }

    #[test]
    fn blow_pmf_shapes() {
        assert_eq!(blow_pmf(0, 1, 3, 0, 0, false), vec![1.0], "never hits");
        let p = blow_pmf(2, 1, 2, 0, 10_000, false);
        assert_eq!(p, vec![0.0, 0.0, 0.0, 0.5, 0.5], "2 + 1d2");
        let v = blow_pmf(0, 1, 3, 0, 10_000, true);
        assert_eq!(quantile(&v, 100), 4.0, "3 x 4/3 against a vulnerability");
    }

    #[test]
    fn cut_and_poison_ticks_follow_process_world() {
        assert_eq!(dot_per_tick(false, 0, false), 0);
        assert_eq!(dot_per_tick(true, 0, false), 1, "poison: 1 a tick");
        assert_eq!(dot_per_tick(false, 7, false), 1, "the 798380952 graze");
        assert_eq!(dot_per_tick(false, 100, false), 1, "a Nasty Cut");
        assert_eq!(dot_per_tick(false, 101, false), 2, "a Severe Cut");
        assert_eq!(dot_per_tick(false, 201, false), 3, "a Deep Gash");
        assert_eq!(dot_per_tick(true, 5000, false), 4, "a Mortal Wound and poison");
        assert_eq!(dot_per_tick(true, 50, true), 1, "rock skin keeps the cut, not the poison");
    }




    #[test]
    fn the_cure_is_the_one_that_stops_the_damage() {
        use DotCure::*;
        let all = |_: DotCure| true;
        let only = |set: &'static [DotCure]| move |c: DotCure| set.contains(&c);
        assert_eq!(dot_cure(false, 0, false, all), None, "nothing ticks");
        assert_eq!(dot_cure(false, 7, false, all), Some(Light), "a graze: CLW takes 20 off");
        assert_eq!(dot_cure(false, 20, false, all), Some(Light));
        assert_eq!(dot_cure(false, 21, false, all), Some(Serious), "past a Light Cut CLW does not end it");
        assert_eq!(dot_cure(false, 100, false, only(&[Light])), None, "a Nasty Cut: CLW is not a cure");
        assert_eq!(dot_cure(false, 100, false, only(&[Light, Critical])), Some(Critical));
        assert_eq!(dot_cure(true, 0, false, all), Some(Neutralize), "poison");
        assert_eq!(dot_cure(true, 0, false, only(&[Light, Serious])), None, "neither CLW nor CSW cures poison");
        assert_eq!(dot_cure(true, 0, false, only(&[Critical])), Some(Critical));
        assert_eq!(dot_cure(true, 7, false, only(&[Light, Serious, Neutralize])), None, "both: only CCW stops both");
        assert_eq!(dot_cure(true, 7, false, all), Some(Critical));
        assert_eq!(dot_cure(true, 7, true, all), Some(Neutralize), "rock skin: only the poison ticks");
    }

    #[test]
    fn the_heal_floor_is_a_share_of_the_maximum() {
        assert!(!below_heal_floor(2, 11, 0), "0 = never (today)");
        assert!(below_heal_floor(2, 11, 20), "the Priest 1294425391: 2/11 = 18%");
        assert!(!below_heal_floor(3, 11, 20), "27%");
        assert!(below_heal_floor(0, 41, 1), "HP 0");
        assert!(below_heal_floor(5, 10, 50), "at the floor counts");
    }
}

#[cfg(test)]
mod tests_1028 {
    use super::*;

    #[test]
    fn the_window_counts_the_world_ticks_of_the_engine_loop() {
        // Normal speed (gain 10): one tick in every window, wherever it falls.
        assert_eq!(window_ticks(100, 100, 10, 100), 1, "acting on a tick turn: that tick is after the action");
        assert_eq!(window_ticks(105, 100, 10, 100), 1);
        // +10 (gain 20): windows of five turns, one tick and none in turn.
        assert_eq!(window_ticks(100, 100, 20, 100), 1, "the turn's own tick falls in the window");
        assert_eq!(window_ticks(105, 100, 20, 100), 0, "next action at 110, before its tick");
        // Energy left after the action: the next action comes sooner.
        assert_eq!(window_ticks(100, 190, 20, 100), 1, "90 left: acts again at 101");
        assert_eq!(window_ticks(101, 190, 20, 100), 0);
        assert_eq!(window_ticks(100, 200, 20, 100), 0, "acts again this turn, before the tick");
        // Slowed (gain 5): twenty turns, two ticks.
        assert_eq!(window_ticks(100, 100, 5, 100), 2);
        assert_eq!(window_ticks(103, 100, 5, 100), 2);
        assert_eq!(window_ticks(100, 100, 0, 100), 10, "no energy at all: bounded");
    }

    #[test]
    fn the_cures_are_named_as_items() {
        assert_eq!(DotCure::Serious.name(), "Cure Serious Wounds");
        assert_eq!(DotCure::Neutralize.name(), "Neutralize Poison");
    }
}
