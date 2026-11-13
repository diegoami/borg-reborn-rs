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
//!   adjacent monster that can follow gets when the step leaves its reach;
//! * the escape ladder (T4.4, #121): walk, Phase Door, Teleportation, Word
//!   of Recall and the stairs, ordered by cost and sufficiency — a blink's
//!   landing risk is the mean assessment over its remembered landing band,
//!   a stair's is the declared arrival risk, and the last known escape
//!   scroll asks a higher expected gain;
//! * hold.
//!
//! The scale is hit points from [`crate::danger`]: a response's `gain` is
//! the threat it removes, its `risk_after` the assessment where it leaves
//! the Borg, its `cost` what it spends beyond game time, and its score is
//! `gain - risk_after - cost`. One aversion parameter (`risk_aversion_pct`)
//! is passed through, as for paths and rest. Every response, including the
//! refused or losing ones, goes to the decision trace.
//!
//! What is not here yet, and stays an explicit gap until #127 supplies the
//! class's own answers: cast a spell, shoot or throw, use a device, and the
//! pack's valuation of them (#126). The response generator has a seam for
//! them; a fixture that needs one stays a gap in `docs/FIXTURES.md`, never
//! a silent skip.

use crate::danger::{assess, can_attack, one_round, one_threat, Assessment};
use crate::engine::PlayerView;
use crate::executor::Action;
use crate::knowledge::Knowledge;
use crate::params::Params;
use crate::path::enter_cost;
use crate::strategy::DIRS;
use crate::world::tv;
use crate::world::World;

/// One response the Borg could take this think, with its numbers.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Response {
    pub action: Action,
    /// `gain - risk_after - cost`, in hit points.
    pub score: f64,
    /// The assessment's risk at the grid the response leaves the Borg on.
    pub risk_after: f64,
    /// The threat the response removes (a kill, a relocation, an escape).
    pub gain: f64,
    /// What the response spends beyond game time: a consumable escape pays
    /// `escape_item_hp`, the last known escape scroll pays `last_escape_hp`
    /// more (T4.4, #121).
    pub cost: f64,
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
    let hp = f64::from(view.hp.max(1));
    // A disable next to a threat can cost the rest of the character's hit
    // points; the composite is a declared expected-loss mapping, not a
    // measured coefficient. It keeps a paralysis gaze in the same scale as
    // damage.
    let current_risk = composite(&current, aversion, hp);
    let mut out = Vec::new();

    // Attacks: gain is the share of the monster's threat this turn removes.
    if can_attack(w) {
        for (i, t) in w.level.monsters.iter().enumerate() {
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
            let one = one_threat(w, k, t, x, y, horizon);
            let removed = one.risk(aversion);
            let share = (dpt / f64::from(t.hp.max(1))).min(1.0);
            let gain = removed * share;
            let status_after = current
                .per_monster
                .get(i)
                .map_or(0.0, |m| (m.disabling + 0.5 * m.summoning) * hp)
                * (1.0 - share);
            let risk_after = (current_risk - gain).max(0.0) + status_after;
            out.push(Response {
                action: Action::Attack(dir),
                score: gain - risk_after,
                risk_after,
                gain,
                cost: 0.0,
                why: "attack",
            });
        }
    }

    // A Potion of Speed (T5.3, #127; bbc D0014): `TIMED_INC:FAST` for
    // 20+2d10 turns (`object.txt`), +10 speed while it lasts
    // (`player-calcs.c:2176-2177`). Its gain is the threat the faster
    // character no longer takes: the same grid assessed at speed + 10,
    // where every monster gets half the rounds per player turn. It costs a
    // consumable (`escape_item_hp`) and is not drunk when already fast.
    let fast = w
        .character
        .status
        .as_ref()
        .is_some_and(|s| s.timed.get(tmd_fast()).copied().unwrap_or(0) > 0);
    if current_risk > 0.0 && !fast {
        if let Some((i, _)) = pack_item(w, k, "Speed").filter(|(_, it)| it.tval == tv::POTION) {
            let mut quick = w.clone();
            if let Some(s) = quick.character.status.as_mut() {
                s.speed += 10;
            }
            let after = composite(&assess(&quick, k, x, y, horizon, aversion), aversion, hp);
            if after < current_risk {
                let cost = f64::from(p.escape_item_hp);
                out.push(Response {
                    action: Action::Quaff { item: i },
                    score: (current_risk - after) - after - cost,
                    risk_after: after,
                    gain: current_risk - after,
                    cost,
                    why: "known Speed potion",
                });
            }
        }
    }

    // A shot (T5.3, #127): `h` fires at the closest visible monster in
    // line of fire (`target_set_closest`, nearest first). Its gain is the
    // share of that monster's threat one shot removes, on the attack's
    // scale; at range the threat that counts is the round it deals once it
    // reaches the character (`one_round`), since that is what killing it
    // first saves. A monster that never moves is no threat at range and is
    // not shot. The hit chance is not modelled, as for melee (declared).
    if let Some((_, shot)) = w.character.shot(k).filter(|_| shot_possible(w)) {
        let target = w
            .level
            .monsters
            .iter()
            .enumerate()
            .filter(|(_, t)| t.in_view && t.hp > 0)
            .filter(|(_, t)| crate::danger::line_clear(w, x, y, t.x, t.y))
            .min_by_key(|(_, t)| ((t.x - x).abs().max((t.y - y).abs()), t.index));
        if let Some((i, t)) = target {
            let d = (t.x - x).abs().max((t.y - y).abs());
            if shot > 0.0 && (d <= 1 || !crate::danger::never_move(k, t)) {
                let removed = if d <= 1 {
                    one_threat(w, k, t, x, y, horizon).risk(aversion)
                } else {
                    one_round(w, k, t).risk(aversion)
                };
                let share = (shot / f64::from(t.hp.max(1))).min(1.0);
                let gain = removed * share;
                let status_after = current
                    .per_monster
                    .get(i)
                    .map_or(0.0, |m| (m.disabling + 0.5 * m.summoning) * hp)
                    * (1.0 - share);
                let risk_after = (current_risk - gain).max(0.0) + status_after;
                out.push(Response {
                    action: Action::Shoot,
                    score: gain - risk_after,
                    risk_after,
                    gain,
                    cost: 0.0,
                    why: "shot at the closest monster",
                });
            }
        }
    }

    // Steps: the destination's risk plus the free hit from every adjacent
    // monster that can follow when the step leaves its reach (the engine's
    // attack of opportunity; a monster that never moves gives none).
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
        let dest = assess(w, k, nx, ny, horizon, aversion);
        let mut risk = composite(&dest, aversion, hp);
        for t in &w.level.monsters {
            if !t.in_view {
                continue;
            }
            let now = (t.x - x).abs().max((t.y - y).abs());
            let after = (t.x - nx).abs().max((t.y - ny).abs());
            if now <= 1 && after > 1 && !crate::danger::never_move(k, t) {
                let free = one_round(w, k, t);
                risk += free.risk(aversion) + (free.disabling + 0.5 * free.summoning) * hp;
            }
        }
        // On the ladder's one scale the step's gain is the threat the
        // relocation removes, exactly as an escape's is (T4.4): a clean step
        // away from a monster that never moves ends its threat for free, so
        // it must outscore spending the last Phase Door (bbc D0013).
        let gain = current_risk - risk;
        out.push(Response {
            action: Action::Walk(dir),
            score: gain - risk,
            risk_after: risk,
            gain,
            cost: 0.0,
            why: "step",
        });
    }

    // Consumables: a known cure and a better weapon. The cure's effect
    // value is the game's own where Knowledge carries it; a Cure Light
    // Wounds is 2d8 (mean 9, class.txt effect data is not in Knowledge yet,
    // so the mean is declared here).
    if view.hp < view.max_hp {
        if let Some((i, _)) = pack_item(w, k, "Cure Light Wounds") {
            let heal = 9.0_f64.min(f64::from(view.max_hp - view.hp));
            out.push(Response {
                action: Action::Quaff { item: i },
                score: heal - current_risk,
                risk_after: current_risk,
                gain: heal,
                cost: 0.0,
                why: "known cure",
            });
        }
    }
    if current_risk > 0.0 {
        if let Some((i, item)) = better_weapon(w, k) {
            let old = w.character.melee_per_turn(k).unwrap_or(0.0);
            let slot = w
                .character
                .equipment
                .iter()
                .position(|s| s.slot_type == crate::world::EQUIP_WEAPON)
                .unwrap_or(0);
            let new = w
                .character
                .wearing(k, slot, Some(item.clone()))
                .melee_per_turn(k)
                .unwrap_or(old);
            if new > old {
                // The gain is the danger the faster kill avoids: the
                // rounds saved on the most threatening adjacent monster.
                let threat = w
                    .level
                    .monsters
                    .iter()
                    .filter(|t| t.in_view && (t.x - x).abs().max((t.y - y).abs()) <= 1)
                    .map(|t| one_threat(w, k, t, x, y, horizon).risk(aversion))
                    .fold(0.0_f64, f64::max);
                let saved = w
                    .level
                    .monsters
                    .iter()
                    .filter(|t| t.in_view && (t.x - x).abs().max((t.y - y).abs()) <= 1)
                    .map(|t| (f64::from(t.hp.max(1)) / old - f64::from(t.hp.max(1)) / new).max(0.0))
                    .fold(0.0_f64, f64::max);
                let gain = (saved * threat).max(0.0);
                out.push(Response {
                    action: Action::Wield { item: i },
                    score: gain - current_risk,
                    risk_after: current_risk,
                    gain,
                    cost: 0.0,
                    why: "better weapon in the pack",
                });
            }
        }
    }

    // The escape ladder (T4.4, #121): step, Phase Door, Teleportation,
    // Word of Recall and the stairs, ordered by cost and sufficiency, not
    // by a fixed monster list. Every level-changing escape is judged by
    // where it lands: a blink's landing is the mean assessment over the
    // remembered grids of its landing band, a stair's is the declared
    // arrival risk, and walking stays available after the last consumable.
    // A consumable escape pays `escape_item_hp`; the last known escape
    // scroll pays `last_escape_hp` more, so it asks a higher expected gain.
    if current_risk > 0.0 {
        let observer = Observer {
            w,
            k,
            horizon,
            aversion,
            hp,
        };
        let last_scroll = known_escape_scrolls(w, k) <= 1;
        let scroll_cost = f64::from(p.escape_item_hp)
            + if last_scroll {
                f64::from(p.last_escape_hp)
            } else {
                0.0
            };
        if let Some(g) = w.level.at(x, y) {
            if g.feat == crate::borg::feat::LESS {
                // Upward to town is known ground; any other arrival level
                // carries the declared arrival risk.
                let landing = if view.depth <= 1 {
                    0.0
                } else {
                    f64::from(p.arrival_risk_hp)
                };
                out.push(Response {
                    action: Action::Ascend,
                    score: current_risk - landing,
                    risk_after: landing,
                    gain: current_risk,
                    cost: 0.0,
                    why: "stairs up underfoot",
                });
            }
            if g.feat == crate::borg::feat::MORE {
                let landing = f64::from(p.arrival_risk_hp);
                out.push(Response {
                    action: Action::Descend,
                    score: current_risk - landing,
                    risk_after: landing,
                    gain: current_risk,
                    cost: 0.0,
                    why: "stairs down underfoot",
                });
            }
        }
        // Scrolls only when the engine lets the character read (#137).
        let readable = crate::danger::can_read(w);
        if let Some((i, _)) = pack_item(w, k, "Phase Door").filter(|_| readable) {
            // The engine's Phase Door lands 10 grids off, ±25%
            // (effect-handler-general.c, effect_handler_TELEPORT).
            if let Some(landing) = blink_landing(&observer, x, y, 8, 12) {
                out.push(Response {
                    action: Action::Read { item: i },
                    score: current_risk - landing - scroll_cost,
                    risk_after: landing,
                    gain: current_risk,
                    cost: scroll_cost,
                    why: "known Phase Door",
                });
            }
        }
        if let Some((i, _)) = pack_item(w, k, "Teleportation").filter(|_| readable) {
            // Teleportation's range is 60% of the longest cardinal distance
            // to the level's edge, ±25% (same handler, M60 percent).
            let m = y.max(w.level.height - y).max(x.max(w.level.width - x));
            if let Some(landing) = blink_landing(&observer, x, y, m * 45 / 100, m * 75 / 100) {
                out.push(Response {
                    action: Action::Read { item: i },
                    score: current_risk - landing - scroll_cost,
                    risk_after: landing,
                    gain: current_risk,
                    cost: scroll_cost,
                    why: "known Teleportation",
                });
            }
        }
        if view.depth > 0
            && w.character
                .status
                .as_ref()
                .is_some_and(|s| s.word_recall <= 0)
        {
            if let Some((i, _)) = pack_item(w, k, "Word of Recall").filter(|_| readable) {
                // Recall lands in town, safely — but the word takes 15-34
                // turns (mean 25) to take effect, and staying that long is
                // the landing risk (player-effects.c, word_recall).
                let staying = assess(w, k, x, y, 25, aversion);
                let landing = composite(&staying, aversion, hp);
                out.push(Response {
                    action: Action::Read { item: i },
                    score: current_risk - landing - scroll_cost,
                    risk_after: landing,
                    gain: current_risk,
                    cost: scroll_cost,
                    why: "known Word of Recall (mean 25-turn delay)",
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
        cost: 0.0,
        why: "hold",
    });

    // A total order (#139): `total_cmp` never calls two different numbers
    // equal, and `ord` folds -0.0 into 0.0 so that a hold at no risk
    // (score -0.0) still ties with a zero-score response, as it did under
    // `partial_cmp`.
    out.sort_by(|a, b| {
        ord(b.score)
            .total_cmp(&ord(a.score))
            .then_with(|| ord(a.risk_after).total_cmp(&ord(b.risk_after)))
            .then_with(|| ord(b.gain).total_cmp(&ord(a.gain)))
            .then_with(|| rank(b.action).cmp(&rank(a.action)))
            .then_with(|| a.action.cmp(&b.action))
    });
    out
}

/// The first aware pack item whose knowledge name is `name`.
fn pack_item<'a>(
    w: &'a World,
    k: &Knowledge,
    name: &str,
) -> Option<(usize, &'a crate::observation::Item)> {
    w.character.inventory.iter().enumerate().find(|(_, i)| {
        i.aware
            && k.kind(i.kind)
                .is_some_and(|kind| clean_name(&kind.name) == name)
    })
}

/// An assessment on the ladder's one scale: the risk plus the declared
/// expected loss to disabling and summoning at the current hit points.
/// Strategy travel and recovery read the same scale (T4.6, #123).
pub(crate) fn composite(a: &Assessment, aversion: i32, hp: f64) -> f64 {
    a.risk(aversion) + (a.disabling + 0.5 * a.summoning) * hp
}

/// A knowledge name without the engine's plural markers (`&`, `~`).
fn clean_name(name: &str) -> String {
    name.chars()
        .filter(|c| !matches!(c, '&' | '~'))
        .collect::<String>()
        .trim()
        .to_string()
}

/// The pack's aware escape scrolls, counted: Phase Door, Teleportation and
/// Word of Recall. The count decides the last-item surcharge (T4.4, #121).
fn known_escape_scrolls(w: &World, k: &Knowledge) -> usize {
    w.character
        .inventory
        .iter()
        .filter(|i| {
            i.aware
                && k.kind(i.kind).is_some_and(|kind| {
                    matches!(
                        clean_name(&kind.name).as_str(),
                        "Phase Door" | "Teleportation" | "Word of Recall"
                    )
                })
        })
        .map(|i| usize::from(i.number))
        .sum()
}

/// The engine's distance (`MAX(dy, dx) + MIN(dy, dx) / 2`, z_averaged
/// metric — the metric teleport lands in).
fn angband_distance(dx: i32, dy: i32) -> i32 {
    let (dx, dy) = (dx.abs(), dy.abs());
    if dy > dx {
        dy + dx / 2
    } else {
        dx + dy / 2
    }
}

/// What one observer remembers of the level, for landing estimates.
struct Observer<'a> {
    w: &'a World,
    k: &'a Knowledge,
    horizon: i32,
    aversion: i32,
    hp: f64,
}

impl Observer<'_> {
    fn risk_at(&self, x: i32, y: i32) -> f64 {
        composite(
            &assess(self.w, self.k, x, y, self.horizon, self.aversion),
            self.aversion,
            self.hp,
        )
    }
}

/// The mean composite risk over the remembered enterable grids at engine
/// distance `lo..=hi` from `(x, y)` — the band a blink with that range
/// lands in. Grids a visible monster stands on are skipped (the engine
/// refuses them as teleport destinations), and an empty band means the
/// landing cannot be estimated, so the escape is not offered.
fn blink_landing(o: &Observer, x: i32, y: i32, lo: i32, hi: i32) -> Option<f64> {
    let mut total = 0.0;
    let mut n = 0usize;
    for gy in 0..o.w.level.height {
        for gx in 0..o.w.level.width {
            let d = angband_distance(gx - x, gy - y);
            if d < lo || d > hi {
                continue;
            }
            if enter_cost(o.w, gx, gy).is_none() {
                continue;
            }
            if o.w
                .level
                .monsters
                .iter()
                .any(|t| t.in_view && (t.x, t.y) == (gx, gy))
            {
                continue;
            }
            total += o.risk_at(gx, gy);
            n += 1;
        }
    }
    (n > 0).then_some(total / n as f64)
}

/// The best pack weapon by expected melee damage, when it beats the wielded
/// one. `wearing` models the swap without touching the world.
fn better_weapon<'a>(w: &'a World, k: &Knowledge) -> Option<(usize, &'a crate::observation::Item)> {
    let current = w.character.melee_per_turn(k)?;
    w.character
        .inventory
        .iter()
        .enumerate()
        // The engine's melee tvals (#126: `tval == 23` was the pre-4.2
        // sword and is hard armour in 4.2, so no real weapon was ever
        // offered; a Maul is hafted, bbc D0011).
        .filter(|(_, i)| i.aware && crate::world::tv::is_melee(i.tval))
        .filter_map(|(i, item)| {
            let slot = w
                .character
                .equipment
                .iter()
                .position(|s| s.slot_type == crate::world::EQUIP_WEAPON)
                .unwrap_or(0);
            let dpt = w
                .character
                .wearing(k, slot, Some(item.clone()))
                .melee_per_turn(k)?;
            (dpt > current + 0.1).then_some((i, item, dpt))
        })
        .max_by(|a, b| ord(a.2).total_cmp(&ord(b.2)))
        .map(|(i, item, _)| (i, item))
}

/// `TMD_FAST` (list-player-timed.h: the first entry, numbered from 0).
fn tmd_fast() -> usize {
    0
}

/// Whether a shot can go where it is aimed this think: not blind (no
/// monster is visible to target), not paralysed (no action), not confused
/// (the shot goes astray, `player_confuse_dir` in `ranged_helper`). Light
/// is not needed to shoot.
fn shot_possible(w: &World) -> bool {
    let Some(s) = w.character.status.as_ref() else {
        return true;
    };
    let timed = |i: usize| s.timed.get(i).copied().unwrap_or(0);
    // TMD_BLIND 2, TMD_PARALYZED 3, TMD_CONFUSED 4 (list-player-timed.h).
    timed(2) <= 0 && timed(3) <= 0 && timed(4) <= 0
}

/// A score for `total_cmp`: -0.0 is 0.0, as `partial_cmp` treats it.
fn ord(x: f64) -> f64 {
    if x == 0.0 {
        0.0
    } else {
        x
    }
}

fn direction(dx: i32, dy: i32) -> Option<u8> {
    DIRS.iter()
        .find(|(_, ddx, ddy)| (*ddx, *ddy) == (dx, dy))
        .map(|(d, _, _)| *d)
}

/// Ties on score prefer the smaller risk left behind, then the larger
/// gain, then an action that ends the threat, then the stairs, then a
/// step, then holding. The order is fixed so replay is stable.
fn rank(a: Action) -> u8 {
    match a {
        Action::Ascend | Action::Descend => 4,
        // An exact tie ends the threat instead of postponing it (GOALS
        // principle 6): killing the mold beats stepping off it (bbc D0012),
        // while a non-killing attack still loses to a cleaner step on the
        // risk_after tie-break.
        Action::Attack(_) => 3,
        Action::Walk(_) => 2,
        _ => 1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::knowledge::{Blow, Dice, MonsterRace, ObjectKind};

    /// A known pack item: the kind in Knowledge, the item in the pack.
    fn give(w: &mut World, k: &mut Knowledge, kind: u32, name: &str) {
        let idx = kind as usize;
        if k.kinds.len() <= idx {
            k.kinds.resize(idx + 1, ObjectKind::default());
        }
        k.kinds[idx] = ObjectKind {
            index: kind,
            name: name.into(),
            ..Default::default()
        };
        w.character.inventory.push(crate::observation::Item {
            kind,
            tval: 70,
            sval: 0,
            number: 1,
            weight: 5,
            timeout: 0,
            level: 1,
            desc: name.into(),
            aware: true,
            fully_known: true,
            runes_known: true,
            flags: [0; 5],
            ac: 0,
            dd: 0,
            ds: 0,
            to_h: 0,
            to_d: 0,
            to_a: 0,
            modifiers: vec![0; 16],
            elements: vec![(0, 0); 25],
        });
    }

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
                tval: 9, // TV_SWORD (a dagger)
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

    #[test]
    fn the_last_phase_door_is_not_spent_on_a_dying_monster() {
        // The kill ends the threat for free; the last Phase Door pays
        // escape_item_hp + last_escape_hp and is kept (T4.4, #121; bbc
        // D0018's dying yeek).
        let (mut w, mut k, v, p) = setup(
            &[
                "###############",
                "#@M...........#",
                "#.............#",
                "#.............#",
                "###############",
            ],
            12,
            13,
            1,
            110,
            None,
        );
        give(&mut w, &mut k, 185, "Phase Door");
        let r = responses(&w, &k, &p, &v);
        assert_eq!(r[0].action, Action::Attack(6), "{r:?}");
        let phase = r
            .iter()
            .find(|x| matches!(x.action, Action::Read { .. }))
            .expect("the Phase Door is offered");
        assert_eq!(
            phase.cost,
            f64::from(p.escape_item_hp) + f64::from(p.last_escape_hp),
            "the last scroll pays both costs"
        );
        assert!(
            r.iter().any(|x| matches!(x.action, Action::Walk(_))),
            "walking stays available after the last consumable"
        );
    }

    #[test]
    fn a_phase_door_is_spent_when_staying_is_worse() {
        // Two Phase Doors: no last-item surcharge, and at 4/20 HP beside a
        // 60-HP monster the blink's landing beats staying (T4.4, #121).
        let (mut w, mut k, v, p) = setup(
            &[
                "###############",
                "#@M...........#",
                "#.............#",
                "#.............#",
                "###############",
            ],
            4,
            20,
            60,
            110,
            None,
        );
        give(&mut w, &mut k, 185, "Phase Door");
        give(&mut w, &mut k, 185, "Phase Door");
        let r = responses(&w, &k, &p, &v);
        let phase = r
            .iter()
            .find(|x| matches!(x.action, Action::Read { .. }))
            .expect("the Phase Door is offered");
        assert_eq!(
            phase.cost,
            f64::from(p.escape_item_hp),
            "a second scroll lifts the surcharge"
        );
        assert!(matches!(r[0].action, Action::Read { .. }), "{r:?}");
    }

    /// A short bow (x2, the engine's kind 77: SHOOTS_ARROWS) in the bow
    /// slot and `arrows` 1d4 arrows in the quiver.
    fn arm_with_bow(w: &mut World, k: &mut Knowledge, arrows: u8) {
        let mut bow_kind = crate::knowledge::ObjectKind {
            index: 77,
            tval: 5,
            sval: 2,
            kind_flags: vec![0, 5],
            ..Default::default()
        };
        bow_kind.pval.base = 2;
        if k.kinds.len() <= 77 {
            k.kinds.resize(78, crate::knowledge::ObjectKind::default());
        }
        k.kinds[77] = bow_kind;
        let mut bow = w.character.equipment[0].item.clone().unwrap();
        bow.kind = 77;
        bow.tval = 5;
        bow.dd = 0;
        bow.ds = 0;
        w.character.equipment.push(crate::observation::Slot {
            slot_type: crate::world::EQUIP_BOW,
            item: Some(bow),
        });
        let mut arrow = w.character.equipment[0].item.clone().unwrap();
        arrow.kind = 81;
        arrow.tval = 3;
        arrow.dd = 1;
        arrow.ds = 4;
        arrow.number = arrows;
        w.character.quiver = vec![arrow];
    }

    #[test]
    fn a_shot_follows_the_engines_launcher_and_ammunition() {
        // ranged_damage: (1d4 mean 2.5 + 0 + 0) x 2 = 5 (T5.3, #127).
        let (mut w, mut k, _, _) = setup(&["#######", "#@...M#", "#######"], 20, 20, 10, 110, None);
        assert_eq!(w.character.shot(&k), None, "no launcher");
        arm_with_bow(&mut w, &mut k, 11);
        assert_eq!(w.character.shot(&k), Some((0, 5.0)));
        w.character.quiver[0].tval = 2; // shots: not what a bow fires
        assert_eq!(w.character.shot(&k), None);
    }

    #[test]
    fn speed_halves_the_threat_it_is_drunk_against() {
        // A Potion of Speed: +10 speed, half the monster rounds per player
        // turn (T5.3, #127; bbc D0014). Not drunk when already fast.
        let (mut w, mut k, v, p) = setup(&["#######", "#@M...#", "#######"], 20, 20, 60, 110, None);
        give(&mut w, &mut k, 247, "Speed");
        w.character.inventory.last_mut().unwrap().tval = tv::POTION;
        let speed = |w: &World| {
            responses(w, &k, &p, &v)
                .into_iter()
                .find(|x| x.why == "known Speed potion")
        };
        let r = speed(&w).expect("offered against an adjacent monster");
        assert!(
            r.gain > 0.0 && r.cost == f64::from(p.escape_item_hp),
            "{r:?}"
        );
        if let Some(s) = w.character.status.as_mut() {
            if s.timed.is_empty() {
                s.timed.resize(1, 0);
            }
            s.timed[0] = 10;
        }
        assert_eq!(speed(&w), None, "already fast");
    }

    #[test]
    fn the_closest_monster_coming_is_shot() {
        let (mut w, mut k, v, p) = setup(
            &["#########", "#@.....M#", "#########"],
            20,
            20,
            10,
            110,
            None,
        );
        arm_with_bow(&mut w, &mut k, 11);
        let r = responses(&w, &k, &p, &v);
        let shot = r
            .iter()
            .find(|x| x.action == Action::Shoot)
            .expect("a coming monster in line of fire is shot");
        assert!(shot.gain > 0.0, "{shot:?}");
        // Confused, the shot goes astray: not offered.
        if let Some(s) = w.character.status.as_mut() {
            if s.timed.len() <= 4 {
                s.timed.resize(5, 0);
            }
            s.timed[4] = 5;
        }
        assert!(!responses(&w, &k, &p, &v)
            .iter()
            .any(|x| x.action == Action::Shoot));
    }

    #[test]
    fn scores_are_totally_ordered_and_zero_has_one_sign() {
        // #139: `total_cmp` orders every score, NaN included, and `ord`
        // keeps -0.0 tied with 0.0 as `partial_cmp` did, so the later
        // keys (risk after, gain, action rank) still break the tie.
        use std::cmp::Ordering;
        assert_eq!(ord(-0.0).total_cmp(&ord(0.0)), Ordering::Equal);
        assert_eq!((-0.0f64).total_cmp(&0.0), Ordering::Less, "why ord exists");
        assert_eq!(ord(f64::NAN).total_cmp(&ord(1.0)), Ordering::Greater);
        assert_eq!(ord(-2.5), -2.5);
    }

    #[test]
    fn no_scroll_is_offered_when_the_engine_refuses_reading() {
        // `player-util.c:player_can_read`: blind, no light, confused or
        // amnesiac refuses a Read; offering one anyway re-chose a refused
        // Phase Door for forty turns (#137, D1667).
        let map = &[
            "###############",
            "#@M...........#",
            "#.............#",
            "#.............#",
            "###############",
        ];
        let reads = |w: &World, k: &Knowledge, v: &PlayerView, p: &Params| {
            responses(w, k, p, v)
                .iter()
                .filter(|x| matches!(x.action, Action::Read { .. }))
                .count()
        };
        let (mut w, mut k, v, p) = setup(map, 4, 20, 60, 110, None);
        give(&mut w, &mut k, 185, "Phase Door");
        give(&mut w, &mut k, 185, "Phase Door");
        assert_eq!(reads(&w, &k, &v, &p), 1, "readable: offered");
        for (index, what) in [(2, "blind"), (4, "confused"), (25, "amnesiac")] {
            let mut w = w.clone();
            if let Some(s) = w.character.status.as_mut() {
                if s.timed.len() <= index {
                    s.timed.resize(index + 1, 0);
                }
                s.timed[index] = 10;
            }
            assert!(!crate::danger::can_read(&w), "{what}");
            assert_eq!(reads(&w, &k, &v, &p), 0, "{what}: not offered");
        }
        // No light: the character's grid is not seen this think.
        let mut dark = w.clone();
        dark.turn += 1;
        assert!(!crate::danger::can_read(&dark));
        assert_eq!(reads(&dark, &k, &v, &p), 0, "no light: not offered");
    }

    #[test]
    fn a_phase_door_with_no_remembered_landing_is_not_offered() {
        // A room too small for the blink's 8-12 band: the landing cannot be
        // estimated, and an unestimated landing is not an offered escape.
        let (mut w, mut k, v, p) = setup(
            &["#######", "#@M...#", "#.....#", "#######"],
            4,
            20,
            60,
            110,
            None,
        );
        give(&mut w, &mut k, 185, "Phase Door");
        let r = responses(&w, &k, &p, &v);
        assert!(
            !r.iter().any(|x| matches!(x.action, Action::Read { .. })),
            "{r:?}"
        );
    }

    #[test]
    fn word_of_recall_is_not_read_while_active() {
        // Reading Recall while the word is running cancels it
        // (player-effects.c): not offered until the timer is 0.
        let (mut w, mut k, v, p) = setup(
            &[
                "###############",
                "#@M...........#",
                "#.............#",
                "#.............#",
                "###############",
            ],
            4,
            20,
            60,
            110,
            None,
        );
        give(&mut w, &mut k, 208, "Word of Recall");
        if let Some(s) = w.character.status.as_mut() {
            s.word_recall = 12;
        }
        let r = responses(&w, &k, &p, &v);
        assert!(
            !r.iter().any(|x| matches!(x.action, Action::Read { .. })),
            "an active word is not re-read: {r:?}"
        );
        w.character.status.as_mut().unwrap().word_recall = 0;
        let r = responses(&w, &k, &p, &v);
        let recall = r
            .iter()
            .find(|x| matches!(x.action, Action::Read { .. }))
            .expect("the idle word is offered");
        assert!(recall.why.contains("Word of Recall"), "{recall:?}");
    }

    #[test]
    fn down_stairs_under_threat_carry_the_declared_arrival_risk() {
        // Every level-changing escape is judged by where it lands: the down
        // stair's risk_after is the declared arrival risk, at no item cost.
        let (w, k, v, p) = setup(
            &["#######", "#>M...#", "#.....#", "#######"],
            4,
            20,
            60,
            110,
            Some((1, 1)),
        );
        let r = responses(&w, &k, &p, &v);
        let down = r
            .iter()
            .find(|x| x.action == Action::Descend)
            .expect("the down stair is offered");
        assert_eq!(down.risk_after, f64::from(p.arrival_risk_hp), "{down:?}");
        assert_eq!(down.cost, 0.0, "the stairs spend no scroll");
    }
}
