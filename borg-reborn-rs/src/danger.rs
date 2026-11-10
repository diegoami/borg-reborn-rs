//! M3's danger placeholder (task T3.3, #19): the C Borg's danger sum,
//! lifted with citations from `borg-danger.c` at `e473720`, until M4's
//! threat model replaces it (ARCHITECTURE.md L4; PLAN.md's M3 row).
//!
//! What is lifted, and where from:
//!
//! - the shape of `borg_danger` — a sum over tracked monsters, capped at
//!   2000 (`borg-danger.c:borg_danger:2824-2865`);
//! - `borg_danger_one_kill`'s melee term: the energy model
//!   (`:2361-2395`), the wakefulness scaling (`:2439-2450`), the
//!   proximity model (`:2516-2593`) and the condition adjustments
//!   (`:2490-2496`);
//! - `borg_danger_physical`'s per-blow damage (`:64-545`): each blow at
//!   **full damage** (the C Borg's entry point forces `full_damage`,
//!   `:2836-2840`), with the effect adjustments of the common early
//!   blows.
//!
//! What is left for M4, declared: the spell term (`borg_danger_spell`,
//! ranged attackers score melee only), the fear-region and fear-monster
//! maps (panel artefacts of the C Borg), the friends multiplier
//! (`Knowledge` carries no friend lists), the Protection-from-Evil and
//! buff reductions (M3's Borg buffs nothing), the anti-bounce erosion of
//! danger (`:2407`) and the bravery behaviour that consumed it. The
//! spell term is the gap that matters: a casting monster at range scores
//! far too low here, which M4's calibration report will measure.

use crate::knowledge::{Knowledge, MonsterRace};
use crate::world::{Track, World};

/// `MON_TMD_*` indices (`list-mon-timed.h`): sleep 0, stun 1, conf 2.
mod mon_tmd {
    pub const STUN: usize = 1;
    pub const CONF: usize = 2;
}

/// `RF_*` flag numbers. The enum starts at `RF_NONE = 0`, and `FLAG_START`
/// is 1: count the `RF(...)` lines of `list-mon-race-flags.h`, not the
/// header's line numbers. Packed as
/// `flags[(n-1)/8] & 1 << ((n-1)%8)` (`z-bitflag.h:FLAG_OFFSET`).
mod rf {
    pub const MULTIPLY: usize = 17;
    pub const NEVER_BLOW: usize = 20;
    pub const NEVER_MOVE: usize = 21;
}

/// `OF_*` flag numbers (`list-object-flags.h`: a flag's number is its
/// line less sixteen, packed as the race flags are).
mod of {
    pub const PROT_FEAR: usize = 6;
    pub const PROT_BLIND: usize = 7;
    pub const PROT_CONF: usize = 8;
    pub const FREE_ACT: usize = 15;
}

/// Element indices (`list-elements.h` order).
mod elem {
    pub const ACID: usize = 0;
    pub const ELEC: usize = 1;
    pub const FIRE: usize = 2;
    pub const COLD: usize = 3;
    pub const POIS: usize = 4;
}

fn has(flags: &[u8], flag: usize) -> bool {
    flags
        .get(flag.wrapping_sub(1) / 8)
        .is_some_and(|b| b & (1 << ((flag - 1) % 8)) != 0)
}

/// The danger of standing at `(x, y)` for the next `c` player turns:
/// the C Borg's `borg_danger` sum, capped at 2000
/// (`borg-danger.c:borg_danger:2824-2865`). The fear maps are not ours
/// (declared above); the whole of the sum is the monsters'.
pub fn danger(w: &World, k: &Knowledge, x: i32, y: i32, c: i32) -> u32 {
    let mut p = 0u32;
    for t in &w.level.monsters {
        p = p.saturating_add(one(w, k, t, x, y, c));
    }
    p.min(2000)
}

/// One monster's contribution, the melee term of
/// `borg_danger_one_kill` (`borg-danger.c:2289-2588`).
pub fn one(w: &World, k: &Knowledge, t: &Track, x: i32, y: i32, c: i32) -> u32 {
    let Some(race) = k.race(t.race) else {
        // An unknown race scores the C Borg's ghost value
        // (`borg_danger_one_kill:2316`).
        return 100;
    };
    // Distance components, minimal 1, beyond 20 nothing
    // (`:2332-2345`).
    let d = (t.x - x).abs().max((t.y - y).abs()).clamp(1, 20);
    if (t.x - x).abs().max((t.y - y).abs()) > 20 {
        return 0;
    }

    // The energy model (`:2361-2395`): monster attack-turns per `c`
    // player turns, in tenths. Monsters are a little fast against a
    // fragile character in the dungeon (`:2357-2359`), a very fast Borg
    // is treated as slower (`:2347-2349`, our haste-less version keeps
    // the real speed), and a monster is never slower than the player
    // (`:2377-2395`).
    let player = w
        .character
        .status
        .as_ref()
        .map_or((110, 1, 20), |s| (s.speed, s.level, s.max_hp));
    let mut monster_speed = i32::from(t.speed);
    if player.2 < 20 && w.level.depth > 0 {
        monster_speed += 3;
    }
    let e = k.turn_energy(player.0);
    let t_turns = (100 + e - 1) / e;
    let e = k.turn_energy(monster_speed.clamp(0, 199));
    let mut q = c * (t_turns * e) / 10;
    // Full damage rounds up to whole attacks (`:2374-2375`).
    q = (q + 9) / 10 * 10;
    let q = q.max(10);

    // The melee term, per blow at full damage.
    let mut v1 = physical(w, race);
    if has(&race.flags, rf::NEVER_BLOW) {
        v1 = 0; // `:2412-2414`
    }
    if has(&race.flags, rf::NEVER_MOVE) && d > 1 {
        v1 = 0; // `:2417-2419`
    }
    if has(&race.flags, rf::MULTIPLY) && player.1 < 20 {
        v1 += v1 * 15 / 10; // `:2422-2425`
    }
    if !t.awake() {
        // Sleeping monsters: light sleepers a little worse, characters
        // of level 25+ half (`:2439-2450`).
        if player.1 >= 25 {
            v1 /= 2;
        }
        let inc = race.sleep + 5;
        v1 += v1 * inc as u32 / 100;
    }
    if t.timed[mon_tmd::CONF] > 0 {
        v1 /= 2; // `:2490-2493`
    }
    if t.timed[mon_tmd::STUN] > 0 {
        v1 = v1 * 10 / 13; // `:2494-2496`
    }

    // Proximity (`:2516-2593`): a faster monster may step and strike in
    // one round, scored as its share of the attacks; a normal-speed
    // monster must already be adjacent.
    let v1 = if q > 10 && d != 1 && !has(&race.flags, rf::NEVER_MOVE) {
        // It contributes only if one step brings it (or its reach)
        // adjacent to the threatened grid (`:2523-2582`); the C Borg
        // simulates the step over its map, we over the remembered level.
        let steps_there = crate::path::enter_cost(w, t.x, t.y).is_some()
            && (0..8).any(|i| {
                let (dx, dy) = step_deltas()[i];
                (t.x + dx - x).abs().max((t.y + dy - y).abs()) <= 1
                    && crate::path::enter_cost(w, t.x + dx, t.y + dy).is_some()
            });
        if steps_there {
            v1 * (q as u32) / (d as u32 * 10)
        } else {
            0
        }
    } else if q > 10 && d == 1 {
        v1 * (q as u32) / 10 // `:2586-2588`
    } else if q == 10 && d > 1 {
        0 // `:2590-2593`
    } else {
        v1
    };

    // The spell term is M4's (declared in the module docs). Out of view,
    // the track's existing confidence decays with the moves the monster
    // may have made (REDESIGN-LESSONS L11): a remembered threat must not
    // block a staircase forever after the monster has had time to leave.
    (f64::from(v1) * t.confidence(w.turn, k)).round() as u32
}

/// The eight neighbouring deltas (`ddy_ddd`/`ddx_ddd` order does not
/// matter here: any legal step counts).
fn step_deltas() -> [(i32, i32); 8] {
    [
        (-1, 1),
        (0, 1),
        (1, 1),
        (-1, 0),
        (1, 0),
        (-1, -1),
        (0, -1),
        (1, -1),
    ]
}

/// The per-round melee damage of a race at full dice,
/// `borg_danger_physical` (`borg-danger.c:64-545`): every blow's worst
/// roll, with the effect adjustments of the blows that matter at the
/// depths M3 plays. Blow effects we do not model score their raw dice
/// (the honest floor); the full table of drain, theft and status blows
/// is M4's.
fn physical(w: &World, race: &MonsterRace) -> u32 {
    let g = &w.character.gear;
    let resist = |e: usize| g.resist.get(e).copied().unwrap_or(0);
    let flag = |f: usize| has(&g.flags, f);
    let mut n = 0u32;
    for blow in &race.blows {
        let raw = (blow.dice.dice * blow.dice.sides).max(0) as u32;
        let z = match blow.effect.as_str() {
            // `:108-125`: plain hurt, with the stun and knockout fudges
            // for many low-sided dice.
            "HURT" | "SHATTER" => {
                let mut z = raw;
                if blow.dice.sides < 3 {
                    z += 200;
                }
                if blow.dice.sides < 3 && blow.dice.dice > 5 {
                    z += 400;
                }
                z
            }
            // `:127-140`: poison, plus the fear unless resisted.
            "POISON" => {
                if resist(elem::POIS) >= 3 {
                    0
                } else {
                    raw + if resist(elem::POIS) >= 1 { 0 } else { 10 }
                }
            }
            // `:218-259`: the elements; immunity skips the blow, each
            // resist level takes a third (rounded), and the fear term
            // doubles the damage (acid corrodes armour instead: +200).
            "ACID" => {
                if resist(elem::ACID) >= 3 {
                    0
                } else {
                    let mut z = raw;
                    for _ in 0..resist(elem::ACID) {
                        z = z.div_ceil(3);
                    }
                    z + 200
                }
            }
            "ELEC" | "FIRE" | "COLD" => {
                let e = match blow.effect.as_str() {
                    "ELEC" => elem::ELEC,
                    "FIRE" => elem::FIRE,
                    _ => elem::COLD,
                };
                if resist(e) >= 3 {
                    0
                } else {
                    let mut z = raw;
                    for _ in 0..resist(e) {
                        z = z.div_ceil(3);
                    }
                    z * 2
                }
            }
            // `:262-310`: the status blows, skipped when protected.
            "BLIND" => {
                if flag(of::PROT_BLIND) {
                    0
                } else {
                    raw + 10
                }
            }
            "CONFUSE" => {
                if flag(of::PROT_CONF) {
                    0
                } else {
                    raw + 200
                }
            }
            "TERRIFY" => {
                if flag(of::PROT_FEAR) {
                    0
                } else {
                    raw * 2
                }
            }
            "PARALYZE" => {
                if flag(of::FREE_ACT) {
                    0
                } else {
                    raw + 200
                }
            }
            // An effect we do not model: its raw dice, the honest floor.
            _ => raw,
        };
        n = n.saturating_add(z);
    }
    n
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::knowledge::{Blow, Dice};
    use crate::world::World;

    fn race(
        name: &str,
        speed: i32,
        sleep: i32,
        blows: &[(i32, i32)],
        flags: &[usize],
    ) -> MonsterRace {
        MonsterRace {
            index: 7,
            name: name.into(),
            base: "x".into(),
            level: 1,
            rarity: 1,
            speed,
            avg_hp: 10,
            ac: 10,
            sleep,
            hearing: 0,
            smell: 0,
            exp: 5,
            freq_innate: 0,
            freq_spell: 0,
            spell_power: 0,
            flags: pack(flags),
            spell_flags: vec![0; 12],
            blows: blows
                .iter()
                .map(|&(d, s)| Blow {
                    method: "CLAW".into(),
                    effect: "HURT".into(),
                    power: 60,
                    dice: Dice {
                        base: 0,
                        dice: d,
                        sides: s,
                        m_bonus: 0,
                    },
                })
                .collect(),
            glyph: b'k' as u32,
            attr: 0,
        }
    }

    /// Pack RF flag numbers the way `z-bitflag.h` does.
    fn pack(flags: &[usize]) -> Vec<u8> {
        let mut out = vec![0u8; 11];
        for &f in flags {
            out[(f - 1) / 8] |= 1 << ((f - 1) % 8);
        }
        out
    }

    fn knowledge(r: MonsterRace) -> Knowledge {
        Knowledge {
            races: vec![
                MonsterRace {
                    index: 0,
                    name: "none".into(),
                    ..Default::default()
                },
                MonsterRace {
                    index: 0,
                    name: "none".into(),
                    ..Default::default()
                },
                MonsterRace {
                    index: 0,
                    name: "none".into(),
                    ..Default::default()
                },
                MonsterRace {
                    index: 0,
                    name: "none".into(),
                    ..Default::default()
                },
                MonsterRace {
                    index: 0,
                    name: "none".into(),
                    ..Default::default()
                },
                MonsterRace {
                    index: 0,
                    name: "none".into(),
                    ..Default::default()
                },
                MonsterRace {
                    index: 0,
                    name: "none".into(),
                    ..Default::default()
                },
                r,
            ],
            ..crate::path::tests::energy()
        }
    }

    /// A world with a three-by-nine corridor (the player at (1,1)) and
    /// one hand-built track, which is all the danger reads.
    fn world(track: Option<(i32, i32, u8, bool)>) -> World {
        let (mut w, _) = crate::path::tests::world(&["#########", "#@......#", "#########"]);
        if let Some(s) = w.character.status.as_mut() {
            s.max_hp = 20; // a CL1 character with hit points: no +3 speed
        }
        w.level.monsters.clear();
        if let Some((x, y, speed, awake)) = track {
            let mut timed = [0i16; 10];
            if !awake {
                timed[0] = 1000;
            }
            w.level.monsters.push(Track {
                index: 1,
                race: 7,
                x,
                y,
                first_seen: 100,
                last_seen: 100,
                in_view: true,
                hp: 10,
                max_hp: 10,
                timed,
                speed,
            });
        }
        w
    }

    #[test]
    fn a_normal_speed_adjacent_monster_scores_its_round() {
        // The player's grid is (1,1) in the helper's world; the monster
        // is placed adjacent at (2,1). One blow 1d6, normal speed,
        // awake, CL1: q = 10, d = 1, the danger is the raw round.
        let k = knowledge(race("kobold", 110, 0, &[(1, 6)], &[]));
        let w = world(Some((2, 1, 110, true)));
        let t = &w.level.monsters[0];
        assert_eq!(one(&w, &k, t, 1, 1, 1), 6);
        // Two blows land in the same round: 1d6 + 1d4 = 10.
        let two = race("kobold", 110, 0, &[], &[]);
        let k = knowledge(MonsterRace {
            blows: vec![
                Blow {
                    method: "CLAW".into(),
                    effect: "HURT".into(),
                    power: 60,
                    dice: Dice {
                        base: 0,
                        dice: 1,
                        sides: 6,
                        m_bonus: 0,
                    },
                },
                Blow {
                    method: "BITE".into(),
                    effect: "HURT".into(),
                    power: 60,
                    dice: Dice {
                        base: 0,
                        dice: 1,
                        sides: 4,
                        m_bonus: 0,
                    },
                },
            ],
            ..two
        });
        assert_eq!(one(&w, &k, t, 1, 1, 1), 10);
        // Many low-sided dice carry the knockout fudges: 10d2 rounds to
        // 20 + 200 (stun) + 400 (kick) = 620.
        let k = knowledge(race("kicker", 110, 0, &[(10, 2)], &[]));
        assert_eq!(one(&w, &k, t, 1, 1, 1), 620);
    }

    #[test]
    fn an_unseen_monster_fades_with_its_track_confidence() {
        let k = knowledge(race("kobold", 110, 0, &[(1, 6)], &[]));
        let mut w = world(Some((2, 1, 110, true)));
        w.turn = 110;
        let t = w.level.monsters[0].clone();
        assert_eq!(one(&w, &k, &t, 1, 1, 1), 6, "visible danger is exact");
        let mut stale = t;
        stale.in_view = false;
        assert_eq!(
            one(&w, &k, &stale, 1, 1, 1),
            3,
            "one possible move of uncertainty"
        );
        w.turn = 1000;
        assert_eq!(one(&w, &k, &stale, 1, 1, 1), 0, "not a permanent wall");
    }

    #[test]
    fn speed_multiplies_and_distance_gates() {
        // A +10 monster adjacent (q = 20): twice the round.
        let k = knowledge(race("fast", 120, 0, &[(1, 6)], &[]));
        let w = world(Some((2, 1, 120, true)));
        let t = &w.level.monsters[0];
        assert_eq!(one(&w, &k, t, 1, 1, 1), 12);
        // The same monster two grids from the threatened grid: it can
        // step adjacent, so it scores its share v1 * q / (d*10) = 12/2.
        let w = world(Some((3, 1, 120, true)));
        let t = &w.level.monsters[0];
        assert_eq!(one(&w, &k, t, 1, 1, 1), 6);
        // A normal-speed monster two grids away scores nothing.
        let k = knowledge(race("slow", 110, 0, &[(1, 6)], &[]));
        let w = world(Some((3, 1, 110, true)));
        let t = &w.level.monsters[0];
        assert_eq!(one(&w, &k, t, 1, 1, 1), 0);
        // Beyond 20 grids nothing counts.
        let k = knowledge(race("far", 110, 0, &[(1, 6)], &[]));
        let w = world(Some((2, 1, 110, true)));
        let t = &w.level.monsters[0];
        assert_eq!(one(&w, &k, t, 25, 1, 1), 0);
    }

    #[test]
    fn sleep_flags_and_conditions_adjust() {
        let k = knowledge(race("sleeper", 110, 10, &[(1, 6)], &[]));
        let w = world(Some((2, 1, 110, false)));
        let t = &w.level.monsters[0];
        // Asleep: v1 * (100 + sleep + 5) / 100 = 6 * 115/100.
        assert_eq!(one(&w, &k, t, 1, 1, 1), 6 * 115 / 100);
        // Confused halves; the helper has no confused track, so check
        // the formula directly on a copy.
        let mut c = t.clone();
        c.timed[mon_tmd::CONF] = 5;
        assert_eq!(one(&w, &k, &c, 1, 1, 1), 6 * 115 / 100 / 2);
    }

    /// The bytes are the engine's, not `pack`'s: the committed fixture
    /// slice has a grey mold's `NEVER_MOVE` in byte 2, bit 4, and a silver
    /// mouse's `MULTIPLY | RAND_50` there. This is the off-by-one that
    /// made a mould score zero (T4.1's fixtures, #142).
    #[test]
    fn race_flag_numbers_match_the_engine_enum() {
        assert_eq!((rf::MULTIPLY, rf::NEVER_BLOW, rf::NEVER_MOVE), (17, 20, 21));
        let grey_mold = [0, 0, 16];
        assert!(has(&grey_mold, rf::NEVER_MOVE), "grey mold cannot move");
        assert!(!has(&grey_mold, rf::NEVER_BLOW), "but it can spore");
        let silver_mouse = [0, 0, 65];
        assert!(has(&silver_mouse, rf::MULTIPLY), "breeders multiply");
        assert!(has(&silver_mouse, 23), "RAND_50");
        let k = knowledge(race(
            "grey mold",
            110,
            0,
            &[(1, 4), (1, 4)],
            &[rf::NEVER_MOVE],
        ));
        let w = world(Some((2, 1, 110, true)));
        let t = &w.level.monsters[0];
        assert!(
            one(&w, &k, t, 1, 1, 1) > 0,
            "an adjacent mould is a threat, not a safe rest"
        );
    }

    #[test]
    fn never_blow_and_never_move() {
        // A mold that cannot move scores only when adjacent.
        let k = knowledge(race("mold", 110, 0, &[(1, 6)], &[rf::NEVER_MOVE]));
        let w = world(Some((2, 1, 110, true)));
        let t = &w.level.monsters[0];
        assert_eq!(one(&w, &k, t, 1, 1, 1), 6);
        let w = world(Some((3, 1, 110, true)));
        let t = &w.level.monsters[0];
        assert_eq!(
            one(&w, &k, t, 1, 1, 1),
            0,
            "two grids away, it cannot close"
        );
        // A caster that never blows: the melee term is zero (its spells
        // are M4's).
        let k = knowledge(race("eye", 110, 0, &[(1, 6)], &[rf::NEVER_BLOW]));
        assert_eq!(one(&w, &k, t, 1, 1, 1), 0);
    }

    #[test]
    fn the_sum_is_capped() {
        let k = knowledge(race("dragon", 130, 0, &[(30, 50)], &[rf::MULTIPLY]));
        let w = world(Some((2, 1, 130, true)));
        let d = danger(&w, &k, 1, 1, 1);
        assert_eq!(d, 2000);
    }
}
