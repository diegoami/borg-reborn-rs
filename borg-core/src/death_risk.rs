//! #790: observational scores on the native core's decision trace.
use crate::borg_caution::{fight_incoming_with, forecast_monster_speed, race_turns};
use crate::borg_fight_attack::{expected_melee_rate, observed_kill_hp};
use crate::borg_headers::borg_magic_h::PHASE_DOOR;
use crate::borg_headers::borg_trait_h::{
    BI_AESCAPE, BI_ATELEPORT, BI_ATELEPORTLVL, BI_CDEPTH, BI_CLEVEL, BI_CURHP, BI_ISAFRAID,
    BI_ISBLIND, BI_ISCONFUSED, BI_ISFORGET, BI_ISPARALYZED, BI_ISPOISONED, BI_MAXHP,
};
use crate::headers::list_mon_race_flags_h::{RF_NEVER_MOVE, RF_REGENERATE};
use crate::headers::list_object_flags_h::OF_NO_TELEPORT;
use crate::headers::list_options_h::OPT_birth_force_descend;
use crate::headers::list_tvals_h::{TV_POTION, TV_SCROLL};
use crate::headers::z_type_h::loc;
use crate::{engine, state::BorgState};
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub(super) struct Observation {
    stop_eligible: bool,
    f1_danger_hp: f64,
    f2_time_to_die: f64,
    f3_race_escapes: f64,
    features: Features,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Features {
    hp_fraction: f64,
    // The fallback is in native danger units. Keep it visible so an offline
    // fit can distinguish this proxy from literal expected contact damage.
    hurt_hp_per_action: f64,
    fallback_danger: f64,
    incoming_over_hp: f64,
    turns_to_kill: Option<f64>,
    adjacent_monsters: u32,
    fastest_speed_delta: Option<i32>,
    phase_scrolls: u32,
    phase_usable: bool,
    other_escape_held: bool,
    cure_potions: u32,
    cure_usable: bool,
    stairs_distance: Option<i32>,
    stairs_usable: bool,
    clevel: i32,
    depth: i32,
    confused: bool,
    blind: bool,
    poisoned: bool,
    paralyzed: bool,
}

fn danger_hp(danger: i32, hp: i32) -> f64 {
    (100.0 * f64::from(danger) / f64::from(hp.max(1))).clamp(0.0, 100.0)
}

fn time_to_die(incoming: f64, hp: i32) -> f64 {
    if incoming <= 0.0 {
        return 0.0;
    }
    // Unfitted starting point. One e-fold per four Borg actions to death.
    100.0 * (-0.25 * f64::from(hp.max(1)) / incoming).exp()
}

fn race_with_escapes(
    hp: i32,
    hurt: f64,
    fallback: f64,
    kill_turns: Option<f64>,
    usable_escapes: u32,
) -> f64 {
    let hp = hp.max(1);
    let base = match kill_turns {
        Some(turns) if hurt > 0.0 && fallback == 0.0 => {
            let margin = turns * hurt / f64::from(hp) - 1.0;
            1.0 / (1.0 + (-4.0 * margin).exp())
        }
        _ => time_to_die(hurt + fallback, hp) / 100.0,
    };
    // Each available escape class subtracts one from the log odds. Work
    // directly in odds so risks of exactly zero or one remain well defined.
    100.0 * base / (base + (1.0 - base) * f64::from(usable_escapes).exp())
}

pub(super) fn observe(st: &mut BorgState, danger: i32, stop_eligible: bool) -> Observation {
    let t = |trait_id| st.borg.tr(trait_id);
    let (hp, max_hp, depth) = (t(BI_CURHP), t(BI_MAXHP), t(BI_CDEPTH));
    let (confused, blind, poisoned, paralyzed) = (
        t(BI_ISCONFUSED) != 0,
        t(BI_ISBLIND) != 0,
        t(BI_ISPOISONED) != 0,
        t(BI_ISPARALYZED) != 0,
    );
    let (forgetful, afraid) = (t(BI_ISFORGET) != 0, t(BI_ISAFRAID) != 0);
    let (clevel, other_escape_held) = (
        t(BI_CLEVEL),
        [BI_AESCAPE, BI_ATELEPORT, BI_ATELEPORTLVL]
            .into_iter()
            .any(|id| t(id) > 0),
    );
    let player = st.borg.c;
    let player_speed = engine::player_combat_speed();
    let player_energy =
        f64::from(engine::extract_energy(player_speed.clamp(0, 199) as usize)).max(1.0);
    let mut hurt = 0.0;
    let mut fallback = 0.0;
    let mut turns_to_kill = (!confused && !afraid && !paralyzed).then_some(0.0);
    let mut adjacent = 0;
    let mut fastest = None;
    for i in 1..st.kills.nxt.max(0) as usize {
        let kill = st.kills.kills[i];
        let race = usize::from(kill.r_idx);
        if race == 0
            || race >= engine::r_max().saturating_sub(1)
            || st.borg.time.now.wrapping_sub(kill.when) > 1
            || st.cave.grid(kill.pos.x, kill.pos.y).info & crate::borg_cave_view::BORG_VIEW == 0
        {
            continue;
        }
        let distance = engine::distance(player, kill.pos);
        adjacent += u32::from(distance <= 1);
        let speed_delta = forecast_monster_speed(&kill) - player_speed;
        fastest = Some(fastest.map_or(speed_delta, |old: i32| old.max(speed_delta)));
        if !kill.awake
            || (distance > 1
                && kill.ranged_attack == 0
                && engine::race_flag_has(race, RF_NEVER_MOVE))
        {
            continue;
        }
        let damage = fight_incoming_with(st, i, false);
        let Some(incoming) = damage.hurt_hp_per_turn else {
            fallback += f64::from(
                crate::borg_danger::borg_danger_one_kill(
                    st, player.y, player.x, 1, i as i32, true, true,
                )
                .max(0),
            );
            continue;
        };
        hurt += incoming;
        if incoming > 0.0 {
            // A melee-only #759 race. At range or with unknown remaining HP,
            // the race is unavailable and the score uses the time estimate.
            turns_to_kill = turns_to_kill.and_then(|total| {
                if distance > 1 || damage.unknown_fight || damage.unknown_damage {
                    return None;
                }
                let monster_hp = observed_kill_hp(&kill)?;
                let (_, max_hp) = kill.observed_health?;
                // mon-move.c:1893-1925 at 574b618c, as in the fight race.
                let regen = f64::from((i32::from(max_hp) / 100).max(1))
                    * if engine::race_flag_has(race, RF_REGENERATE) {
                        2.0
                    } else {
                        1.0
                    }
                    / player_energy;
                let attack = expected_melee_rate(st, i) - regen;
                race_turns(monster_hp, attack, None, 1.0, 0.0).map(|(turns, _)| total + turns)
            });
        }
    }

    let items = &st.items.items;
    let pack = &items[..crate::borg_item::inven_wield() as usize];
    let equipment =
        &items[crate::borg_item::inven_wield() as usize..crate::borg_item::quiver_start() as usize];
    let ids = &st.item_val;
    let mut phase_scrolls = 0;
    let mut healing_potions = 0;
    let mut poison_cures = 0;
    let mut confusion_cures = 0;
    let mut poison_only_potions = 0;
    for item in pack.iter().filter(|item| item.aware && item.iqty > 0) {
        let sval = i32::from(item.sval);
        let count = u32::from(item.iqty);
        match u32::from(item.tval) {
            TV_SCROLL if sval == ids.sv_scroll_phase_door => phase_scrolls += count,
            TV_POTION => {
                // object.txt at 574b618c. Light Wounds can reduce confusion.
                if [
                    ids.sv_potion_cure_light,
                    ids.sv_potion_cure_serious,
                    ids.sv_potion_cure_critical,
                    ids.sv_potion_healing,
                ]
                .contains(&sval)
                {
                    confusion_cures += count;
                }
                if [
                    ids.sv_potion_cure_critical,
                    ids.sv_potion_healing,
                    ids.sv_potion_star_healing,
                    ids.sv_potion_life,
                ]
                .contains(&sval)
                {
                    poison_cures += count;
                    healing_potions += count;
                } else if [ids.sv_potion_cure_light, ids.sv_potion_cure_serious].contains(&sval) {
                    healing_potions += count;
                } else if sval == ids.sv_potion_cure_poison {
                    poison_cures += count;
                    poison_only_potions += count;
                }
            }
            _ => {}
        }
    }
    let equipped = || equipment.iter().filter(|item| item.iqty > 0);
    // cmd-obj.c: player_can_read and effect-handler-general.c: TELEPORT
    // at 574b618c. Spell and activation use the existing Borg knowledge.
    let phase_usable = !paralyzed
        && !engine::player_arena_level()
        && !equipped().any(|item| engine::flag_has(&item.flags, OF_NO_TELEPORT as i32))
        && ((phase_scrolls > 0 && !blind && !confused && !forgetful && !engine::no_light())
            || crate::borg_magic::borg_spell_okay_fail(st, PHASE_DOOR, 25)
            || equipped().any(|item| item.activ_idx == st.act.tele_phase && item.timeout == 0));
    let cure_usable = !paralyzed
        && ((healing_potions > 0 && (hp < max_hp || blind))
            || (confused && confusion_cures > 0)
            || (poisoned && poison_cures > 0));

    // Distance is geometric for the fit. Only the current tile or one
    // unoccupied adjacent stair qualifies for a race discount.
    let up_allowed = !engine::player_opt(OPT_birth_force_descend);
    let stairs = [&st.stairs.less, &st.stairs.more];
    let mut stairs_distance = None;
    let mut stairs_usable = false;
    if depth > 0 {
        for (direction, track) in stairs.into_iter().enumerate() {
            if direction == 0 && !up_allowed {
                continue;
            }
            for i in 0..track.num as usize {
                let (y, x) = track.at(i);
                let d = engine::distance(player, loc { x, y });
                stairs_distance = Some(stairs_distance.map_or(d, |old: i32| old.min(d)));
                stairs_usable |= !paralyzed
                    && (d == 0
                        || (d == 1
                            && !confused
                            && st.cave.grid(x, y).kill == 0
                            && hurt + fallback < f64::from(hp)));
            }
        }
    }
    let escapes = [phase_usable, cure_usable, stairs_usable]
        .into_iter()
        .map(u32::from)
        .sum();
    Observation {
        stop_eligible,
        f1_danger_hp: danger_hp(danger, hp),
        f2_time_to_die: time_to_die(hurt + fallback, hp),
        f3_race_escapes: race_with_escapes(hp, hurt, fallback, turns_to_kill, escapes),
        features: Features {
            hp_fraction: (f64::from(hp) / f64::from(max_hp.max(1))).clamp(0.0, 1.0),
            hurt_hp_per_action: hurt,
            fallback_danger: fallback,
            incoming_over_hp: (hurt + fallback) / f64::from(hp.max(1)),
            turns_to_kill,
            adjacent_monsters: adjacent,
            fastest_speed_delta: fastest,
            phase_scrolls,
            phase_usable,
            other_escape_held,
            cure_potions: healing_potions + poison_only_potions,
            cure_usable,
            stairs_distance,
            stairs_usable,
            clevel,
            depth,
            confused,
            blind,
            poisoned,
            paralyzed,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn danger_hp_matches_stopscared_and_clamps() {
        assert_eq!(danger_hp(5, 20), 25.0);
        assert_eq!(danger_hp(20, 20), 100.0);
        assert_eq!(danger_hp(200, 1), 100.0);
        assert_eq!(danger_hp(-1, 0), 0.0);
    }

    #[test]
    fn time_to_die_responds_to_damage_and_health() {
        assert_eq!(time_to_die(0.0, 20), 0.0);
        assert!((time_to_die(5.0, 20) - 100.0 / std::f64::consts::E).abs() < 1e-10);
        assert!(time_to_die(10.0, 20) > time_to_die(5.0, 20));
        assert!(time_to_die(10.0, 40) < time_to_die(10.0, 20));
    }

    #[test]
    fn race_prices_equal_fights_and_discounts_escapes() {
        assert_eq!(race_with_escapes(20, 5.0, 0.0, Some(4.0), 0), 50.0);
        assert!(race_with_escapes(20, 5.0, 0.0, Some(5.0), 0) > 50.0);
        assert!(race_with_escapes(20, 5.0, 0.0, Some(4.0), 1) < 50.0);
        assert_eq!(race_with_escapes(20, 0.0, 0.0, None, 3), 0.0);
        assert_eq!(
            race_with_escapes(20, 0.0, 5.0, None, 0),
            time_to_die(5.0, 20)
        );
    }
}
