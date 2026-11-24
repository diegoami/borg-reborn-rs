//! Track P (#1194): choose a wound cure for the next combat turn.

use crate::borg_cave_util::borg_cave_floor_bold;
use crate::borg_danger::combat_reply;
use crate::borg_headers::borg_trait_h::{BI_CDEPTH, BI_CURHP, BI_ISPOISONED, BI_MAXHP};
use crate::borg_item_use::{borg_quaff_crit, borg_quaff_potion};
use crate::borg_poison::{CLW_HEAL, CSW_HEAL};
use crate::borg_projection::{borg_distance, borg_projectable};
use crate::headers::list_mon_spells_h::RSF_TELE_TO;
use crate::headers::z_type_h::loc as Loc;
use crate::params::{damage_only, value as param, Param};
use crate::state::BorgState;
use crate::{borg_io::borg_note, engine};

/// Model the reply when the Borg spends its next action in place. A pull
/// can expose it to attacks that cannot reach its current grid. Compare
/// those alternative replies without adding them to the one at this grid.
fn incoming(st: &mut BorgState) -> i32 {
    let player = st.borg.c;
    let mut worst = damage_only(|| damage_at(st, player, None));
    for i in 1..st.kills.nxt.max(0) as usize {
        let kill = st.kills.kills[i];
        if kill.r_idx == 0
            || !kill.awake
            || !kill.spell[..usize::from(kill.ranged_attack)].contains(&(RSF_TELE_TO as u8))
            || borg_distance(kill.pos.y, kill.pos.x, player.y, player.x)
                > i32::from(engine::z_info().max_range)
            || !borg_projectable(st, kill.pos.y, kill.pos.x, player.y, player.x)
        {
            continue;
        }

        // effect_handler_TELEPORT_TO (574b618c) searches outward starting
        // at the caster. Model the known free adjoining grids.
        for dy in -1..=1 {
            for dx in -1..=1 {
                if dx == 0 && dy == 0 {
                    continue;
                }
                let at = Loc {
                    x: kill.pos.x + dx,
                    y: kill.pos.y + dy,
                };
                if borg_cave_floor_bold(st, at.y, at.x) && st.cave.grid(at.x, at.y).kill == 0 {
                    worst = worst.max(damage_only(|| damage_at(st, at, Some(i))));
                }
            }
        }
    }
    worst
}

fn damage_at(st: &mut BorgState, at: Loc, puller: Option<usize>) -> i32 {
    (1..st.kills.nxt.max(0) as usize)
        .map(|i| {
            let mut kill = st.kills.kills[i];
            if kill.r_idx == 0
                || !kill.awake
                || usize::from(kill.r_idx) >= engine::r_max().saturating_sub(1)
            {
                return 0;
            }
            kill.speed = crate::borg_caution::forecast_monster_speed(&kill).clamp(0, 199) as u8;
            combat_reply(st, &kill, at.y, at.x, i as i32, puller == Some(i))
        })
        .sum()
}

pub(crate) fn enabled(st: &BorgState) -> bool {
    param(Param::caution_combat_heal) != 0 && st.borg.tr(BI_CDEPTH) != 0
}

/// If the monster reply is lethal, defer the poison-only cure. The result
/// in that case says whether a wound cure took the turn.
pub(crate) fn before_poison_cure(st: &mut BorgState) -> Option<bool> {
    if !enabled(st) {
        return None;
    }
    let damage = incoming(st);
    (damage > 0 && damage >= st.borg.tr(BI_CURHP)).then(|| quaff_for_reply(st, damage))
}

fn heal_due(hp: i32, max: i32, damage: i32, min_loss: i32) -> bool {
    damage > 0 && hp < max && (damage >= hp || (max - hp) * 100 >= max * min_loss)
}

fn covers_reply(hp: i32, max: i32, restored: i32, damage: i32, poison_hp: i32) -> bool {
    (hp + restored).min(max) > damage + poison_hp
}

pub(crate) fn quaff(st: &mut BorgState) -> bool {
    quaff_in_combat(st).unwrap_or(false)
}

/// When the next reply includes damage, the combat policy owns the choice
/// of wound potion. The legacy wound bands still apply outside this case.
pub(crate) fn quaff_in_combat(st: &mut BorgState) -> Option<bool> {
    if !enabled(st) || st.borg.tr(BI_CURHP) >= st.borg.tr(BI_MAXHP) {
        return None;
    }
    let damage = incoming(st);
    (damage > 0).then(|| quaff_for_reply(st, damage))
}

fn quaff_for_reply(st: &mut BorgState, damage: i32) -> bool {
    let hp = st.borg.tr(BI_CURHP);
    let max = st.borg.tr(BI_MAXHP);
    if !heal_due(hp, max, damage, param(Param::caution_combat_heal_min_loss))
        || crate::borg_fast_fragile::heal_banned(st)
    {
        return false;
    }

    // object.txt:3156-3209 (574b618c): fixed 20/40/60 HP, and of
    // these three potions only Cure Critical Wounds clears poison.
    // game-world.c:process_world:514-518 (574b618c): one HP per world tick.
    let poison_hp = if st.borg.tr(BI_ISPOISONED) != 0 {
        crate::borg_poison::ticks_per_turn()
    } else {
        0
    };
    let v = &st.item_val;
    let cures = [
        (v.sv_potion_cure_light, CLW_HEAL),
        (v.sv_potion_cure_serious, CSW_HEAL),
        (v.sv_potion_cure_critical, 60),
    ];
    for (sval, restored) in cures {
        let critical = restored == 60;
        if covers_reply(
            hp,
            max,
            restored,
            damage,
            if critical { 0 } else { poison_hp },
        ) && if critical {
            borg_quaff_crit(st, true)
        } else {
            borg_quaff_potion(st, sval)
        } {
            borg_note(st, &format!("# Combat healing: +{restored} HP at {hp}/{max}, modelled reply {damage} (#1194)"));
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::{covers_reply, heal_due};

    #[test]
    fn tengu_light_cure_can_cover_the_reply_except_at_nine_hp() {
        assert!(heal_due(175, 225, 16, 20));
        assert!(!heal_due(185, 225, 16, 20));
        assert!(!heal_due(175, 225, 0, 20));
        assert!(covers_reply(89, 237, 20, 32, 0));
        assert!(covers_reply(38, 237, 20, 32, 0));
        assert!(!covers_reply(9, 237, 20, 32, 0));
        assert!(covers_reply(9, 237, 40, 32, 0));
    }

    #[test]
    fn potion_is_due_for_a_lethal_reply_even_below_the_wound_threshold() {
        assert!(heal_due(4, 91, 7, 100));
        assert!(covers_reply(4, 91, 20, 7, 0));
        assert!(!covers_reply(3, 131, 0, 7, 0));
        assert!(covers_reply(3, 131, 40, 7, 1));
        assert!(!covers_reply(3, 131, 20, 22, 1));
        assert!(!covers_reply(10, 12, 60, 12, 0));
    }
}

#[cfg(feature = "test-combat-heal")]
pub(crate) mod fixture {
    use super::*;
    use crate::borg_caution::{borg_caution, fight_before_flight, fight_race};
    use crate::borg_escape::{borg_escape, phase_rule_act, PhaseVerdict};
    use crate::borg_flow_kill::BorgKill;
    use crate::borg_headers::borg_trait_h::{
        BI_APHASE, BI_ARMOR, BI_CLEVEL, BI_CON_INDEX, BI_DEV, BI_MAXCLEVEL, BI_SPEED, BI_THN,
    };
    use crate::headers::list_mon_spells_h::RSF_MAX;
    use crate::headers::list_terrain_h::FEAT_FLOOR;
    use crate::headers::list_tvals_h::{TV_POTION, TV_SCROLL, TV_WAND};

    fn add_monster(st: &mut BorgState, name: &[u8], x: i32, hp: i16, max: i16) {
        let index = (1..engine::r_max())
            .find(|&i| engine::race_name(i) == name)
            .expect("fixture race is in the engine");
        let race = engine::race(index);
        let mut kill = BorgKill {
            r_idx: index as u16,
            known: true,
            awake: true,
            seen: true,
            pos: Loc { x, y: st.borg.c.y },
            speed: race.speed as u8,
            power: hp,
            observed_health: Some((hp, max)),
            when: st.borg.time.now,
            ..BorgKill::BLANK
        };
        for flag in 1..RSF_MAX {
            if engine::flag_has(&race.spell_flags, flag as i32) {
                kill.spell[usize::from(kill.ranged_attack)] = flag as u8;
                kill.ranged_attack += 1;
            }
        }
        let i = st.kills.nxt;
        st.kills.kills[i as usize] = kill;
        st.kills.nxt += 1;
        st.kills.cnt += 1;
        st.cave.grid_mut(x, kill.pos.y).kill = i as u8;
    }

    fn add_item(st: &mut BorgState, slot: usize, tval: u32, sval: i32) {
        let kind = (1..engine::k_max())
            .find(|&i| {
                let kind = engine::kind(i);
                kind.tval as u32 == tval && i32::from(kind.sval) == sval
            })
            .expect("fixture kind is in the engine");
        let item = &mut st.items.items[slot];
        item.aware = true;
        item.ident = true;
        item.kind = kind as u32;
        item.tval = tval as u8;
        item.sval = sval as u8;
        item.iqty = 1;
        item.value = engine::kind_cost_of(kind);
        st.borg.has[kind] = 1;
    }

    fn check_potion(st: &BorgState, expected: u8, case: &str) {
        assert_eq!(
            st.io.queued_keys(),
            [u32::from(b'q'), u32::from(expected)],
            "{case}: the expected potion takes the turn"
        );
    }

    /// Fabricated decision states based on the three recorded deaths.
    /// Run the production decisions with real game kinds, blows and spells.
    pub(crate) fn run(st: &mut BorgState, case: &str) -> ! {
        let (depth, level, hp, max) = match case {
            "tengu-early" => (5, 15, 175, 225),
            "tengu-light" => (5, 15, 89, 237),
            "tengu-phase-fight" => (5, 15, 38, 237),
            "tengu-unsafe-phase" => (5, 15, 9, 237),
            "tengu-recall" => (5, 15, 9, 237),
            "paladin-wand" => (8, 14, 59, 131),
            "paladin-poison" => (8, 14, 3, 131),
            "rogue-fight-race" => (5, 16, 4, 91),
            _ => panic!("unknown #1194 fixture: {case}"),
        };
        let enabled = param(Param::caution_combat_heal) != 0;
        crate::borg_cave::borg_init_cave(st);
        st.borg.c = Loc { x: 40, y: 20 };
        for y in 7..=33 {
            for x in 27..=53 {
                let grid = st.cave.grid_mut(x, y);
                grid.feat = FEAT_FLOOR as u8;
                grid.info = crate::borg_cave_view::BORG_VIEW | 0x08 /* BORG_OKAY */;
            }
        }
        st.kills.nxt = 1;
        st.kills.cnt = 0;
        st.danger = Default::default();
        st.stairs.less.num = 0;
        st.stairs.more.num = 0;
        st.borg.goal = Default::default();
        st.borg.time.now += 10;
        st.borg.time.level = st.borg.time.now;
        st.borg.avoidance = hp as i16;
        for (key, value) in [
            (BI_CDEPTH, depth),
            (BI_CLEVEL, level),
            (BI_MAXCLEVEL, level),
            (BI_CURHP, hp),
            (BI_MAXHP, max),
            (BI_SPEED, 110),
            (BI_APHASE, 1),
        ] {
            st.borg.trait_[key as usize] = value;
        }
        let n = usize::from(engine::z_info().pack_size);
        st.items.items[..n].fill(crate::borg_item::blank_item());
        st.borg.has.fill(0);
        let v = &st.item_val;
        let (light, serious, critical, poison, phase) = (
            v.sv_potion_cure_light,
            v.sv_potion_cure_serious,
            v.sv_potion_cure_critical,
            v.sv_potion_cure_poison,
            v.sv_scroll_phase_door,
        );
        for (slot, sval) in [(0, light), (1, serious), (3, poison)] {
            add_item(st, slot, TV_POTION, sval);
        }
        if !case.starts_with("paladin-") {
            add_item(st, 2, TV_POTION, critical);
        }
        add_item(st, 4, TV_SCROLL, phase);

        let expected = match case {
            "tengu-early" | "tengu-light" => {
                // Both enemies are outside immediate melee reach. One can
                // pull the Borg into range of both, so its potion has a job.
                add_monster(st, b"tengu", 47, 80, 80);
                add_monster(st, b"tengu", 48, 80, 80);
                let position = st.borg.c;
                assert_eq!(damage_only(|| damage_at(st, position, None)), 0);
                assert!(incoming(st) > 0, "a pull can expose the Borg to damage");
                st.borg.goal.fleeing = true;
                assert!(borg_caution(st));
                b'a'
            }
            "tengu-phase-fight" => {
                add_monster(st, b"tengu", 41, 20, 80);
                add_monster(st, b"tengu", 42, 60, 80);
                st.borg.phase_rule = Some((st.borg.time.now, PhaseVerdict::Fight, false));
                assert!(phase_rule_act(st));
                b'a'
            }
            "tengu-unsafe-phase" | "tengu-recall" => {
                add_monster(st, b"tengu", 41, 20, 80);
                add_monster(st, b"tengu", 39, 60, 80);
                assert!(incoming(st) >= hp + 20, "light cannot cover the reply");
                if case == "tengu-recall" {
                    st.borg.goal.recalling = 10;
                    st.borg.goal.fleeing = true;
                    for row in &mut st.danger.fear_region {
                        row.fill(500);
                    }
                    assert!(borg_caution(st));
                    assert_eq!(st.borg.goal.recalling, 10);
                } else {
                    for row in &mut st.danger.fear_region {
                        row.fill(500);
                    }
                    st.borg.phase_rule = Some((st.borg.time.now, PhaseVerdict::Phase, false));
                    st.borg.goal.type_0 = 5;
                    let escapes = st.borg.escapes;
                    assert!(borg_escape(st, 2000));
                    assert_eq!(st.borg.escapes, escapes);
                    if !enabled {
                        assert!(
                            st.io
                                .queued_keys()
                                .ends_with(&[u32::from(b'r'), u32::from(b'e')]),
                            "the old branch reads Phase Door"
                        );
                        println!("#1194 combat healing passed: {case}");
                        std::process::exit(0);
                    }
                    assert_eq!(st.borg.goal.type_0, 5);
                }
                b'b'
            }
            "paladin-wand" => {
                add_item(st, 5, TV_WAND, st.item_val.sv_wand_wonder);
                st.items.items[5].pval = 5;
                st.borg.trait_[BI_DEV as usize] = 60;
                add_monster(st, b"Wormtongue, Agent of Saruman", 44, 62, 250);
                st.kills.kills[1].observed_fast = true;
                st.borg.trait_[BI_ISPOISONED as usize] = 1;
                st.borg.trait_[BI_CON_INDEX as usize] = 11;
                engine::set_test_poison(22);
                st.borg.phase_rule = Some((st.borg.time.now, PhaseVerdict::Shoot, false));
                assert!(incoming(st) > 0);
                assert!(phase_rule_act(st));
                if !enabled {
                    assert!(
                        st.io
                            .queued_keys()
                            .windows(2)
                            .any(|keys| keys == [u32::from(b'a'), u32::from(b'f')]),
                        "the old branch aims the Wand of Wonder: {:?}",
                        st.io.queued_keys()
                    );
                    println!("#1194 combat healing passed: {case}");
                    std::process::exit(0);
                }
                b'a'
            }
            "paladin-poison" => {
                add_monster(st, b"Wormtongue, Agent of Saruman", 41, 15, 250);
                st.borg.trait_[BI_ISPOISONED as usize] = 1;
                st.borg.trait_[BI_CON_INDEX as usize] = 11;
                st.kills.kills[1].observed_fast = true;
                engine::set_test_poison(36);
                assert!(crate::borg_poison::cure_carried(st));
                if enabled {
                    b'b'
                } else {
                    b'd'
                }
            }
            "rogue-fight-race" => {
                add_monster(st, b"zombified orc", 41, 0, 50);
                st.borg.trait_[BI_ARMOR as usize] = 64;
                st.borg.trait_[BI_THN as usize] = 160;
                let race = fight_race(st, false).expect("the old fight verdict is available");
                assert!(
                    race.won,
                    "the old fight verdict wins on expectation: {} turns, {} damage",
                    race.turns, race.incoming
                );
                assert!(fight_before_flight(st));
                if !enabled {
                    assert!(
                        st.io
                            .queued_keys()
                            .ends_with(&[u32::from(b'+'), u32::from(b'6')]),
                        "the old branch attacks the orc: {:?}",
                        st.io.queued_keys()
                    );
                    println!("#1194 combat healing passed: {case}");
                    std::process::exit(0);
                }
                b'a'
            }
            _ => unreachable!(),
        };
        check_potion(st, expected, case);
        println!("#1194 combat healing passed: {case}");
        std::process::exit(0);
    }
}
