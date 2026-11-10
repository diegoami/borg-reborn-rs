//! The tactical layer (T4.3, #120): every available response, scored on one
//! scale, with the risk *after* the response.
//!
//! The M3 strategy had a first-applicable rule: fight the cheapest adjacent
//! exchange, otherwise retreat. That is how it walked at 1 HP from a 2-HP
//! monster (the free hit kills) and how it failed to take a staircase
//! underfoot. Here every response is generated and scored:
//!
//! * attack each adjacent monster (only when `can_attack`; fear and
//!   paralysis are refused by the engine);
//! * step to each of the eight neighbours, adding the *free hit* every
//!   adjacent monster gets when the step leaves its reach;
//! * take the staircase underfoot when a threat is present (its landing
//!   risk is #121's, declared unknown here);
//! * hold.
//!
//! The scale is hit points from [`crate::danger`]: a response's `gain` is
//! the threat it removes, its `risk` is the assessment after it, and its
//! score is `gain - risk`. One aversion parameter (`risk_aversion_pct`) is
//! passed through, as for paths and rest. Every response, including the
//! refused or losing ones, goes to the decision trace.
//!
//! What is not here yet, and stays an explicit gap until #122 supplies the
//! executor actions: shoot, throw, cast, quaff, read, eat, wield and use a
//! device. The response generator has a seam for them; a fixture that needs
//! one stays a gap in `docs/FIXTURES.md`, never a silent skip.

use crate::danger::{assess, can_attack, one_round, one_threat};
use crate::engine::PlayerView;
use crate::executor::Action;
use crate::knowledge::Knowledge;
use crate::params::Params;
use crate::path::enter_cost;
use crate::strategy::DIRS;
use crate::world::World;

/// One response the Borg could take this think, with its numbers.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Response {
    pub action: Action,
    /// `gain - risk_after`, in hit points.
    pub score: f64,
    /// The assessment's risk at the grid the response leaves the Borg on.
    pub risk_after: f64,
    /// The threat the response removes (a kill, an escape).
    pub gain: f64,
    /// A short trace note: what the response is for.
    pub why: &'static str,
}

/// Every response, best first. Deterministic: the input order is the fixed
/// direction order and ties break by action rank, then action value.
pub fn responses(w: &World, k: &Knowledge, p: &Params, view: &PlayerView) -> Vec<Response> {
    let (x, y) = (view.x, view.y);
    let aversion = p.risk_aversion_pct;
    let horizon = p.threat_horizon;
    let current = assess(w, k, x, y, horizon, aversion);
    let current_risk = current.risk(aversion);
    let mut out = Vec::new();

    // Attacks: gain is the share of the monster's threat this turn removes.
    if can_attack(w) {
        for t in &w.level.monsters {
            if !t.in_view || (t.x - x).abs().max((t.y - y).abs()) > 1 {
                continue;
            }
            let Some(dir) = direction(t.x - x, t.y - y) else {
                continue;
            };
            let Some(dpt) = w.character.melee_per_turn(k) else {
                continue;
            };
            if dpt <= 0.0 || t.hp <= 0 {
                continue;
            }
            let removed = one_threat(w, k, t, x, y, horizon).risk(aversion);
            let share = (dpt / f64::from(t.hp.max(1))).min(1.0);
            let gain = removed * share;
            out.push(Response {
                action: Action::Attack(dir),
                score: gain - (current_risk - gain).max(0.0),
                risk_after: (current_risk - gain).max(0.0),
                gain,
                why: "attack",
            });
        }
    }

    // Steps: the destination's risk plus the free hit from every adjacent
    // monster the step leaves behind (the engine's attack of opportunity).
    for &(dir, dx, dy) in DIRS.iter() {
        let (nx, ny) = (x + dx, y + dy);
        if enter_cost(w, nx, ny).is_none() {
            continue;
        }
        if w.level
            .monsters
            .iter()
            .any(|t| t.in_view && (t.x, t.y) == (nx, ny))
        {
            continue; // walking into a monster is the attack response
        }
        let mut risk = assess(w, k, nx, ny, horizon, aversion).risk(aversion);
        for t in &w.level.monsters {
            if !t.in_view {
                continue;
            }
            let now = (t.x - x).abs().max((t.y - y).abs());
            let after = (t.x - nx).abs().max((t.y - ny).abs());
            if now <= 1 && after > 1 {
                risk += one_round(w, k, t).risk(aversion);
            }
        }
        out.push(Response {
            action: Action::Walk(dir),
            score: -risk,
            risk_after: risk,
            gain: 0.0,
            why: "step",
        });
    }

    // The staircase underfoot is an escape from a threat. Landing risk is
    // #121's; using 0 here is declared, not measured.
    if current_risk > 0.0 {
        if let Some(g) = w.level.at(x, y) {
            // Upward only: the town or a shallower level is known ground.
            // A down staircase under a threat is not taken until #121
            // models the landing (M3's "a staircase under fire" rule).
            if g.feat == crate::borg::feat::LESS {
                out.push(Response {
                    action: Action::Ascend,
                    score: current_risk,
                    risk_after: 0.0,
                    gain: current_risk,
                    why: "stair underfoot (landing risk not modelled)",
                });
            }
        }
    }

    // Holding is always available and takes game time (unless a shop
    // entrance turns it into the store, which is itself an escape).
    out.push(Response {
        action: Action::Hold,
        score: -current_risk,
        risk_after: current_risk,
        gain: 0.0,
        why: "hold",
    });

    out.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| rank(b.action).cmp(&rank(a.action)))
            .then_with(|| a.action.cmp(&b.action))
    });
    out
}

fn direction(dx: i32, dy: i32) -> Option<u8> {
    DIRS.iter()
        .find(|(_, ddx, ddy)| (*ddx, *ddy) == (dx, dy))
        .map(|(d, _, _)| *d)
}

/// Ties on score prefer an action that ends the threat, then the stairs,
/// then a step, then holding. The order is fixed so replay is stable.
fn rank(a: Action) -> u8 {
    match a {
        // A stair is an escape; a step leaves a losing fight; a tie on
        // score should not turn a non-killing attack into a preference.
        Action::Ascend | Action::Descend => 4,
        Action::Walk(_) => 3,
        Action::Attack(_) => 2,
        _ => 1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::knowledge::{Blow, Dice, MonsterRace};

    fn race(name: &str, speed: i32, hp: i32, blows: &[(i32, i32)]) -> MonsterRace {
        MonsterRace {
            index: 7,
            name: name.into(),
            base: "x".into(),
            level: 1,
            rarity: 1,
            speed,
            avg_hp: hp,
            ac: 10,
            sleep: 0,
            hearing: 0,
            smell: 0,
            exp: 5,
            freq_innate: 0,
            freq_spell: 0,
            spell_power: 1,
            flags: vec![0; 11],
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
            glyph: 0,
            attr: 0,
        }
    }

    fn setup(
        map: &[&str],
        hp: i32,
        max_hp: i32,
        monster_hp: i32,
        speed: u8,
        player: Option<(i32, i32)>,
    ) -> (World, Knowledge, PlayerView, Params) {
        let (mut w, mut k) = crate::path::tests::world(map);
        let mut r = race("test", i32::from(speed), monster_hp, &[(1, 2)]);
        r.index = 1;
        k.races = vec![MonsterRace::default(), r];
        k.classes = vec![crate::knowledge::tests::warrior()];
        if let Some(s) = w.character.status.as_mut() {
            s.hp = hp;
            s.max_hp = max_hp;
            s.level = 1;
            s.known_ac = 10;
            s.speed = 110;
            if let Some((x, y)) = player {
                s.x = x;
                s.y = y;
            }
        }
        w.character.equipment = vec![crate::observation::Slot {
            slot_type: 1,
            item: Some(crate::observation::Item {
                kind: 6,
                tval: 23,
                sval: 4,
                number: 1,
                weight: 12,
                timeout: 0,
                level: 1,
                desc: "a Dagger (1d4)".into(),
                aware: true,
                fully_known: true,
                runes_known: true,
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
        let view = PlayerView {
            hp,
            max_hp,
            x: w.character.status.as_ref().unwrap().x,
            y: w.character.status.as_ref().unwrap().y,
            depth: 1,
            dead: false,
            busy: false,
            turn: 100,
        };
        (w, k, view, Params::default())
    }

    #[test]
    fn a_kill_beats_a_retreat_that_takes_a_free_hit() {
        let (w, k, v, p) = setup(
            &["#######", "#@M...#", "#.....#", "#######"],
            1,
            20,
            2,
            110,
            None,
        );
        let r = responses(&w, &k, &p, &v);
        assert_eq!(r[0].action, Action::Attack(6), "{r:?}");
    }

    #[test]
    fn stairs_under_a_pursuer_are_taken() {
        let (w, k, v, p) = setup(
            &["#######", "#<F...#", "#.....#", "#######"],
            1,
            20,
            25,
            120,
            Some((1, 1)),
        );
        let r = responses(&w, &k, &p, &v);
        assert_eq!(r[0].action, Action::Ascend, "{r:?}");
    }

    #[test]
    fn every_available_response_is_scored() {
        let (w, k, v, p) = setup(
            &["#######", "#.@M..#", "#.....#", "#######"],
            20,
            20,
            2,
            110,
            None,
        );
        let r = responses(&w, &k, &p, &v);
        assert!(r.iter().any(|x| matches!(x.action, Action::Attack(6))));
        assert!(r.iter().any(|x| matches!(x.action, Action::Walk(4))));
        assert!(r.iter().any(|x| x.action == Action::Hold));
        assert!(r.iter().all(|x| x.score.is_finite()));
        // The best is first; the rest are still there for the trace.
        assert!(r.windows(2).all(|w| w[0].score >= w[1].score));
    }

    #[test]
    fn fear_removes_every_attack_candidate() {
        let (mut w, k, v, p) = setup(
            &["#######", "#.@M..#", "#.....#", "#######"],
            20,
            20,
            2,
            110,
            None,
        );
        if let Some(s) = w.character.status.as_mut() {
            s.timed = vec![0; 11];
            s.timed[5] = 5;
        }
        let r = responses(&w, &k, &p, &v);
        assert!(
            !r.iter().any(|x| matches!(x.action, Action::Attack(_))),
            "{r:?}"
        );
        assert!(r.iter().any(|x| matches!(x.action, Action::Walk(_))));
    }
}
