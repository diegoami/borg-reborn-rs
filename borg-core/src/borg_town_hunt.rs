//! #789: fight the dangerous melee-only town residents when the race is winnable.

use crate::borg_caution::{
    fight_incoming_with, fight_regeneration, forecast_monster_speed, race_turns, wins_race,
};
use crate::borg_cave_view::BORG_VIEW;
use crate::borg_danger::danger_one_kill;
use crate::borg_fight_attack::{
    focused_attack_is_ranged, ignored, observed_kill_hp, perform_focused_attack,
    town_focused_attack, town_melee_damage,
};
use crate::borg_flow_kill::{flow_town_hunt, BorgKill};
use crate::borg_headers::borg_trait_h::*;
use crate::borg_io::borg_note;
use crate::engine;
use crate::params::{value, Param};
use crate::state::BorgState;

pub(crate) const REASON: &str = "town-melee-only";

pub(crate) fn enabled(st: &BorgState) -> bool {
    st.borg.tr(BI_CDEPTH) == 0 && value(Param::town_hunt) != 0
}

pub(crate) fn avoids_maggot(st: &BorgState, race: usize) -> bool {
    enabled(st) && race != 0 && race == st.borg.id.maggot as usize
}

pub(crate) fn controls_target(st: &BorgState, race: usize) -> bool {
    if !enabled(st) || race == 0 || avoids_maggot(st, race) {
        return false;
    }
    let Some(list) = crate::scary_list::selected() else {
        return false;
    };
    let who = crate::scary_list::Situation {
        clevel: st.borg.tr(BI_CLEVEL),
        class: st.borg.tr(BI_CLASS),
        breeder_count: 0,
        breeder_stand: false,
        levels_less: 0,
    };
    crate::scary_list::attack_reason(list, race, engine::race_base_name(race), who) == Some(REASON)
        && engine::race(race).level == 0
}

/// The town hunt can act only on a current, visible target. Use the same
/// gate when yielding the ordinary fight planner to the town hunt.
pub(crate) fn viewed_target(st: &BorgState, kill: &BorgKill) -> bool {
    let race = usize::from(kill.r_idx);
    controls_target(st, race)
        && !ignored(st, race)
        && kill.ranged_attack == 0
        && observed_kill_hp(kill).is_some()
        && st.borg.time.now.wrapping_sub(kill.when) <= value(Param::attack_max_target_age)
        && st.cave.grid(kill.pos.x, kill.pos.y).info & BORG_VIEW != 0
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Hunt {
    Continue,
    Acted,
    Retreat,
}

fn distance(st: &BorgState, kill: &BorgKill) -> i32 {
    (st.borg.c.x - kill.pos.x)
        .abs()
        .max((st.borg.c.y - kill.pos.y).abs())
}

fn low_hp(hp: i32, max_hp: i32, floor: i32) -> bool {
    i64::from(hp) * 100 < i64::from(max_hp) * i64::from(floor)
}

fn wins_before_contact(turns: f64, distance: i32, player_energy: f64, monster_energy: f64) -> bool {
    // Monsters can move diagonally. Book one step immediately for energy
    // already held, then compare remaining steps in Borg turns (mon-move.c).
    let free_turns = f64::from((distance - 2).max(0)) * player_energy / monster_energy;
    turns < free_turns
}

fn after_attack(moved: bool, at_risk: bool) -> Hunt {
    if moved {
        Hunt::Acted
    } else if at_risk {
        Hunt::Retreat
    } else {
        Hunt::Continue
    }
}

fn attack_turns(hp: i32, damage: f64, uses: Option<i32>, duration: f64, melee: f64) -> Option<f64> {
    if damage <= 0.0 || uses.is_some_and(|uses| uses <= 0) {
        return None;
    }
    if melee < damage {
        return race_turns(hp, damage, uses, duration, melee).map(|(turns, _)| turns);
    }
    // The #759 fallback can replace the primary attack outright. The town
    // planner is deciding whether to execute that attack, so charge it once.
    let remaining = (f64::from(hp) - damage * duration).ceil() as i32;
    if remaining <= 0 {
        return Some(duration);
    }
    race_turns(remaining, melee, None, 1.0, 0.0).map(|(turns, _)| duration + turns)
}

pub(crate) fn think(st: &mut BorgState) -> Hunt {
    if !enabled(st) || st.borg.goal.fleeing || st.borg.mon.scary {
        return Hunt::Continue;
    }
    let mut targets = Vec::new();
    for i in 1..st.kills.nxt.max(0) as usize {
        let kill = st.kills.kills[i];
        if !viewed_target(st, &kill) {
            continue;
        }
        // Rank by danger at contact, even when the threat is currently out
        // of reach or asleep. The native danger score includes theft.
        let awake = BorgKill {
            awake: true,
            ..kill
        };
        let danger = danger_one_kill(st, &awake, kill.pos.y, kill.pos.x, 1, i as i32, true, true);
        targets.push((i, danger));
    }
    targets.sort_by_key(|&(i, danger)| (std::cmp::Reverse(danger), i));
    for (i, danger) in targets {
        let decision = hunt_target(st, i, danger);
        if decision != Hunt::Continue {
            return decision;
        }
    }
    Hunt::Continue
}

fn hunt_target(st: &mut BorgState, i: usize, danger: i32) -> Hunt {
    let kill = st.kills.kills[i];
    let d = distance(st, &kill);
    let hp = st.borg.tr(BI_CURHP);
    if low_hp(
        hp,
        st.borg.tr(BI_MAXHP),
        value(Param::town_hunt_min_hp_percent),
    ) {
        borg_note(st, "# Town hunt: HP below retreat threshold (#789)");
        return Hunt::Retreat;
    }
    if [
        BI_ISBLIND,
        BI_ISCONFUSED,
        BI_ISIMAGE,
        BI_ISCUT,
        BI_ISPOISONED,
        BI_ISAFRAID,
        BI_ISSTUN,
        BI_ISHEAVYSTUN,
        BI_ISHUNGRY,
        BI_ISWEAK,
    ]
    .iter()
    .any(|&t| st.borg.tr(t) != 0)
    {
        return after_attack(false, d <= 1);
    }
    let Some(enemy_hp) = observed_kill_hp(&kill) else {
        return Hunt::Continue;
    };
    let incoming = fight_incoming_with(st, i, true);
    let regen = fight_regeneration(&kill);
    let (y, x) = (st.borg.c.y, st.borg.c.x);
    let mut other_danger = 0.0;
    let mut other_contact = false;
    for j in 1..st.kills.nxt.max(0) as usize {
        let other = st.kills.kills[j];
        if j == i || other.r_idx == 0 {
            continue;
        }
        let danger = danger_one_kill(st, &other, y, x, 1, j as i32, true, true);
        if danger > 0 {
            if usize::from(other.r_idx) >= engine::r_max().saturating_sub(1) {
                other_contact = true;
                continue;
            }
            let forecast = fight_incoming_with(st, j, distance(st, &other) <= 1);
            other_danger += forecast.hp_per_turn;
            other_contact |= forecast.unknown_damage || forecast.unknown_fight;
        }
    }
    let contact_is_modelled = !incoming.unknown_damage && !incoming.unknown_fight && !other_contact;
    let in_contact =
        |turns| contact_is_modelled && wins_race(turns, incoming.hp_per_turn + other_danger, hp);
    let melee = (f64::from(town_melee_damage(st, i)) / 1000.0 - regen).max(0.0);
    let can_approach = d > 1
        && race_turns(enemy_hp, melee, None, 1.0, 0.0)
            .is_some_and(|(turns, _)| in_contact(turns + 1.0));

    let energy =
        |speed: i32| f64::from(engine::extract_energy(speed.clamp(0, 199) as usize)).max(1.0);
    let player_energy = energy(engine::player_combat_speed());
    let monster_energy = energy(forecast_monster_speed(&kill));
    let attack = town_focused_attack(st, i, value(Param::town_hunt_ranged_bonus_percent), |a| {
        let fail = a.cost.map_or(0, |c| c.fail_pct);
        let uses = a.cost.and_then(|c| c.uses);
        let damage =
            f64::from(a.hp_per_turn_milli) / 1000.0 * f64::from(100 - fail) / 100.0 - regen;
        if focused_attack_is_ranged(a)
            && race_turns(enemy_hp, damage, uses, a.duration, 0.0).is_some_and(|(turns, _)| {
                !other_contact
                    && wins_race(turns, other_danger, hp)
                    && wins_before_contact(turns, d, player_energy, monster_energy)
            })
        {
            return true;
        }
        let fallback = if d <= 1 || can_approach { melee } else { 0.0 };
        attack_turns(enemy_hp, damage, uses, a.duration, fallback).is_some_and(|turns| {
            in_contact(turns + if d > 1 && fallback > 0.0 { 1.0 } else { 0.0 })
        })
    });
    let name = String::from_utf8_lossy(engine::race_name(usize::from(kill.r_idx)));
    if let Some(attack) = attack {
        if perform_focused_attack(st, i, attack) {
            borg_note(
                st,
                &format!(
                    "# Town hunt: attack {name}, danger {danger}, attack {} (#789)",
                    attack.kind
                ),
            );
            return Hunt::Acted;
        }
    }
    let moved = can_approach && flow_town_hunt(st, i, i32::from(engine::z_info().max_sight) + 1);
    // If the enemy is already bearing down on us, a known losing race is
    // enough to start flight before contact. Distant uncertified fights can
    // wait while the Borg continues shopping.
    let closing =
        kill.awake && d == 2 && danger_one_kill(st, &kill, y, x, 1, i as i32, true, true) > 0;
    let result = after_attack(
        moved,
        d <= 1 || (closing && contact_is_modelled && !can_approach),
    );
    if result == Hunt::Acted {
        borg_note(
            st,
            &format!("# Town hunt: approach {name}, danger {danger} (#789)"),
        );
    } else if result == Hunt::Retreat {
        borg_note(
            st,
            &format!("# Town hunt: no winning fight with {name} (#789)"),
        );
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_approach_needs_a_winning_forecast_and_a_path() {
        assert_eq!(after_attack(true, false), Hunt::Acted);
        assert_eq!(after_attack(false, false), Hunt::Continue);
        assert_eq!(after_attack(false, true), Hunt::Retreat);
    }

    #[test]
    fn a_ranged_plan_pays_for_the_first_command_before_switching_to_melee() {
        assert_eq!(attack_turns(10, 1.0, Some(1), 1.0, 10.0), Some(2.0));
        assert_eq!(attack_turns(10, 5.0, Some(0), 1.0, 10.0), None);
        assert_eq!(attack_turns(10, 5.0, Some(2), 1.0, 0.0), Some(2.0));
    }

    #[test]
    fn the_hp_floor_is_strict_and_uses_max_hp() {
        assert!(low_hp(9, 20, 50));
        assert!(!low_hp(10, 20, 50));
        assert!(!low_hp(1, 20, 0));
    }

    #[test]
    fn ranged_win_must_finish_before_contact_at_the_current_speeds() {
        assert!(wins_before_contact(2.0, 5, 10.0, 10.0));
        assert!(!wins_before_contact(3.0, 5, 10.0, 10.0));
        assert!(!wins_before_contact(2.0, 5, 10.0, 20.0));
        assert!(!wins_before_contact(0.5, 1, 10.0, 10.0));
    }
}
