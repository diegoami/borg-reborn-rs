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

use crate::borg_caution::{fight_incoming_with, forecast_monster_speed};
use crate::borg_cave_view::BORG_VIEW;
use crate::borg_fight_attack::{
    attack_on_sight_reason, focused_attack, observed_kill_hp, FocusedAttack,
};
use crate::borg_headers::borg_trait_h::*;
use crate::borg_io::{borg_keypress, borg_note};
use crate::engine;
use crate::headers::list_mon_race_flags_h::{RF_MULTIPLY, RF_NEVER_MOVE, RF_UNIQUE};
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

/// Whether `escape_instead` could act now: a staircase underfoot, or a Phase
/// Door scroll or spell that can be used (no reading in the dark, blind or
/// confused; no teleport where it is not allowed).
fn escape_available(st: &mut BorgState) -> bool {
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

fn has_cure(st: &mut BorgState) -> bool {
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
        let incoming = incoming_total(st, target, f);
        escape_wanted(incoming, f.cur_hp, f.need_hp, f.per_turn, param(Param::fast_fragile_escape_percent))
            && !escape_available(st)
    }
}

/// Attack-on-sight steps aside for the heal when the Borg is `cornered` and
/// carries a cure (without one the attack is all it has).
pub(crate) fn yield_to_cure(st: &mut BorgState, target: usize, f: &Facts) -> bool {
    cornered(st, target, f) && has_cure(st)
}

/// Alternative to the attack (owner, #842): when the next turn kills the Borg
/// and the kill needs more than one more turn, only Phase Door or a
/// staircase underfoot. `true` when the turn was spent on one of them.
pub(crate) fn escape_instead(st: &mut BorgState, target: usize, f: &Facts) -> bool {
    let incoming = incoming_total(st, target, f);
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
    if crate::borg_escape::borg_allow_teleport(st)
        && (crate::borg_item_use::borg_read_scroll(st, st.item_val.sv_scroll_phase_door)
            || crate::borg_magic::borg_spell_fail(
                st, crate::borg_headers::borg_magic_h::PHASE_DOOR, param(Param::escape_last_ditch_phase_fail)))
    {
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
