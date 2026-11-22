//! #878: attack on sight a lone slow monster with the ranged attack.
//!
//! The Human Mage 461220874 (borg-zoo run 44936494812, csw arm; borg-cemetery
//! case 2026-11-22-reborn-confusion-csw-mage-461220874): one white worm mass,
//! speed +0 and 10 HP, adjacent to a CL3 Mage at HP 20/20 and SP 5/5. The
//! core ranked Magic Missile first, but `borg_caution` runs before
//! `borg_attack` and retreated ("32 > 1": danger 32, mostly fear), then the
//! Phase rule read Phase Door. Two Magic Missiles kill the worm (the
//! counterfactual from the save at turn 15037: HP 20/20, SP 5 -> 3).
//!
//! The rule (`lone_slow_attack`, compiled and shipped 0) is the spell
//! counterpart of `borg_adjacent_kill_in_one_blow` (borg_caution.rs, melee
//! only): it runs in `fight_before_flight`, before retreat, back-away and
//! the Phase Door escapes. A hypothesis until borg-zoo measures it.

use crate::borg_caution::{category_attack_safe_with, fight_incoming_with, forecast_monster_speed};
use crate::borg_cave_view::BORG_VIEW;
use crate::borg_fight_attack::{focused_ranged_attack, observed_kill_hp, perform_focused_attack, FocusedAttack};
use crate::borg_headers::borg_trait_h::*;
use crate::borg_io::borg_note;
use crate::engine;
use crate::params::{value as param, Param};
use crate::state::BorgState;

/// What the verdict reads, so it can be tested without the engine.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Facts {
    /// HP left to kill.
    pub need_hp: f64,
    /// The ranged attack's expected damage a Borg turn, fail rate included.
    pub per_turn: f64,
    /// The uses the mana, charges or missiles pay for (`None`: no limit).
    pub uses: Option<i32>,
    /// The target's expected blow damage a Borg turn (HP, not danger).
    pub incoming: f64,
    pub cur_hp: i32,
}

/// The uses the kill takes (rounded up), if the rule allows the attack:
/// at most `max_uses`, paid for, and the target's damage over them below
/// `damage_percent` of the current HP.
pub(crate) fn verdict(f: &Facts, max_uses: i32, damage_percent: i32) -> Option<i32> {
    if f.per_turn <= 0.0 || f.cur_hp <= 0 || f.need_hp <= 0.0 {
        return None;
    }
    let uses = (f.need_hp / f.per_turn - 1e-9).ceil().max(1.0) as i32;
    let paid = f.uses.is_none_or(|u| u >= uses);
    let hurt = f.incoming.max(0.0) * f64::from(uses) * 100.0 < f64::from(damage_percent) * f64::from(f.cur_hp);
    (uses <= max_uses && paid && hurt).then_some(uses)
}

fn fresh(st: &BorgState, i: usize) -> bool {
    let kill = &st.kills.kills[i];
    kill.r_idx != 0 && st.borg.time.now.wrapping_sub(kill.when) <= param(Param::attack_max_target_age)
}

/// The one awake monster within `lone_slow_radius`, if there is
/// exactly one and it is in view.
fn lone_target(st: &BorgState) -> Option<usize> {
    let radius = param(Param::lone_slow_radius);
    let mut found = None;
    for i in 1..st.kills.nxt.max(0) as usize {
        let kill = &st.kills.kills[i];
        if !fresh(st, i) || !kill.awake || engine::distance(st.borg.c, kill.pos) > radius {
            continue;
        }
        if found.is_some() {
            return None;
        }
        found = Some(i);
    }
    let i = found?;
    let kill = &st.kills.kills[i];
    (st.cave.grid(kill.pos.x, kill.pos.y).info & BORG_VIEW != 0).then_some(i)
}

fn per_turn(attack: FocusedAttack) -> f64 {
    f64::from(attack.hp_per_turn_milli) / 1000.0 * f64::from(100 - attack.cost.map_or(0, |c| c.fail_pct)) / 100.0
}

/// `lone_slow_attack`: whether the Borg attacked a lone slow monster.
pub(crate) fn attack(st: &mut BorgState) -> bool {
    if param(Param::lone_slow_attack) == 0 || st.borg.tr(BI_CDEPTH) == 0 {
        return false;
    }
    let Some(i) = lone_target(st) else { return false };
    let kill = st.kills.kills[i];
    let r = usize::from(kill.r_idx);
    if r >= engine::r_max().saturating_sub(1) {
        return false;
    }
    let speed = forecast_monster_speed(&kill);
    if speed > engine::player_combat_speed() {
        return false;
    }
    let Some(attack) = focused_ranged_attack(st, i) else { return false };
    let facts = Facts {
        need_hp: f64::from(observed_kill_hp(&kill).unwrap_or_else(|| engine::race(r).avg_hp.max(1))),
        per_turn: per_turn(attack),
        uses: attack.cost.and_then(|c| c.uses),
        incoming: fight_incoming_with(st, i, false).hp_per_turn,
        cur_hp: st.borg.tr(BI_CURHP),
    };
    let Some(uses) = verdict(&facts, param(Param::lone_slow_casts), param(Param::lone_slow_damage_percent)) else {
        return false;
    };
    // The other monsters' danger, as for attack on sight.
    if !category_attack_safe_with(st, i, false) {
        return false;
    }
    borg_note(st, &format!(
        "# Attack on sight: {} (lone and slow, speed {:+}), {:.0} HP, attack {} at {:.1} a turn: {uses} use{} of {}, {:.1} damage a turn against HP {} (#878)",
        String::from_utf8_lossy(engine::race_name(r)), speed - 110, facts.need_hp, attack.kind, facts.per_turn,
        if uses == 1 { "" } else { "s" },
        facts.uses.map_or("no limit".to_string(), |u| u.to_string()), facts.incoming, facts.cur_hp,
    ));
    perform_focused_attack(st, i, attack)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The white worm mass against the CL3 Mage: 10 HP, Magic Missile 3d4
    /// (7.5) at about 5% fail, 5 casts paid; 1d2 poison crawl.
    fn worm() -> Facts {
        Facts { need_hp: 10.0, per_turn: 7.1, uses: Some(5), incoming: 0.9, cur_hp: 20 }
    }

    #[test]
    fn two_magic_missiles_kill_the_worm() {
        assert_eq!(verdict(&worm(), 2, 25), Some(2));
    }

    #[test]
    fn each_part_of_the_rule_can_refuse() {
        assert_eq!(verdict(&worm(), 1, 25), None, "more uses than allowed");
        assert_eq!(verdict(&Facts { uses: Some(1), ..worm() }, 2, 25), None, "the mana pays for one cast");
        assert_eq!(verdict(&Facts { uses: Some(0), ..worm() }, 3, 25), None, "no mana: no reserve spell here");
        assert_eq!(verdict(&Facts { uses: None, ..worm() }, 2, 25), Some(2), "a staff: no limit");
        assert_eq!(verdict(&Facts { incoming: 2.5, ..worm() }, 2, 25), None, "5 damage over two turns is 25% of 20");
        assert_eq!(verdict(&Facts { incoming: 2.4, ..worm() }, 2, 25), Some(2));
        assert_eq!(verdict(&Facts { per_turn: 0.0, ..worm() }, 2, 25), None, "no ranged attack");
        assert_eq!(verdict(&Facts { need_hp: 7.1, ..worm() }, 1, 25), Some(1), "exactly one turn is one use");
        assert_eq!(verdict(&Facts { cur_hp: 0, ..worm() }, 2, 25), None);
    }
}
