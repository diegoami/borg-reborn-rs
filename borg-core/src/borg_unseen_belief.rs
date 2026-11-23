//! borg-reborn-rs (#879, owner 2026-11-22: "we can teach the borg to shoot
//! through a corridor, if they think a monster is there"): a position belief
//! for a monster the Borg saw recently but cannot see now.
//!
//! The C Borg aims only at monsters seen within `attack_max_target_age`
//! Borg turns (`borg_attack`, `borg_launch_bolt_aux_hack`). With
//! `unseen_belief` = 1, a monster last seen up to
//! `unseen_max_age` turns ago, at least two grids away, whose last grid
//! the Borg would not expect to see it on (dark, out of its light:
//! `borg_follow_kill_aux`), stays a target of the ranged attacks (bolts, balls,
//! missiles, throws) at that grid. Each attack's value is weighted by the chance
//! that the monster is still there. Melee never takes such a target, and it
//! never counts as the monster next to the Borg (`borg_attack`'s
//! adjacent-monster test, which turns a scary level's flight into a fight).
//!
//! What can actually be aimed at is narrower (review of #890, F3):
//! `borg_attack` keeps only a record whose grid is on the panel (`BORG_OKAY`)
//! and inside the Borg's view shape (`BORG_VIEW`, the ray-cast shape, lit or
//! not), so an unlit corridor in line works and a monster around a corner does
//! not. The two-grid floor is the light's: `borg_follow_kill_aux` also expects
//! a warm monster within `BI_INFRA` to be visible, so infravision (a race's or
//! an item's) raises the floor, and a cold-blooded monster is exempt from it.
//!
//! The engine stops a bolt at the first monster on its path
//! (src/effect-handler-attack.c:130, `PROJECT_STOP`), so a monster that closed
//! in along the line is still hit; the
//! Borg aims by location (`borg_target`: `*`, `p`, the cursor, `5`; a free-mode
//! location target, src/ui-target.c:1479-1494, src/target.c:180-191).
//!
//! The belief (a hypothesis; the zoo measures it, RULES 14): 100% for a monster
//! that never moves (`NEVER_MOVE`) or that the Borg believes asleep; otherwise
//! `100 - unseen_decay_percent` per turn the monster has had since it was
//! seen (Borg turns times its energy over the Borg's, `extract_energy`), never
//! below 0. Erratic movers (`RAND_25`/`RAND_50`, src/mon-move.c:1087-1101) are
//! not treated apart.
//!
//! The effective window (review of #890, F2) is the decay, not
//! `unseen_max_age`: at the shipped 20% and a Borg at speed 110, a monster of
//! the same speed is believed at ages 3 and 4 only (40%, 20%), a speed-100
//! one up to age 9, a speed-90 one up to 10, and one faster than the Borg
//! never. `unseen_max_age` bounds only sleepers, `NEVER_MOVE` and slow
//! monsters. The energy ratio approximates the engine's action counts
//! (src/game-world.c:136-139) and the integer division makes slow decay coarse.
//! (The figures above are for the compiled 20%. The shipped decay is 5%,
//! #1100, at which a same-speed monster keeps a belief until age 19, so
//! `unseen_max_age`, shipped 10, bounds it too.)
//!
//! #889, follow-ups of #879, each a switch shipped 0 (hypotheses, RULES 14):
//! - `unseen_death_clears`: "It dies." / "It is destroyed." after a projection
//!   with a single believed target deletes that record at once
//!   (`believed_dead`), instead of keeping it until `unseen_max_shots`.
//! - `unseen_light_reveal`: Light Room at a believed light-sensitive monster
//!   within its sphere (`light_reveal`, `reveal_target`), which hurts it and
//!   lights its grid; 2 = at any believed monster there.
//! - `unseen_spread`: the belief spreads (`belief_percent_spread`): an erratic
//!   mover's random steps (RAND_25/RAND_50) do not count as leaving, and a
//!   blast still hits a monster that moved fewer grids than the blast's
//!   reach beyond its last grid (`unseen_blast_slack`).
//!
//! Beams and arcs take believed targets as bolts and balls do: every path
//! (`borg_launch_bolt_at_location` for bolts, beams and balls,
//! `borg_launch_arc_at_location` for arcs) scores a record through
//! `borg_launch_bolt_aux_hack`, which asks `believed_weight`, and both stage
//! the targets with `ProjectedAttack::target` (#889, item 5; no recorded beam
//! or arc case yet). Beliefs from unseen contact or noise (#889 item 1, #1127) are
//! not modelled: a record needs a race, and an unseen attacker's is unknown.

use crate::borg_flow_kill::{borg_delete_kill, borg_follow_kill_aux};
use crate::borg_headers::borg_magic_h::LIGHT_ROOM;
use crate::borg_headers::borg_trait_h::{BI_ISBLIND, BI_ISCONFUSED, BI_ISIMAGE, BI_SPEED};
use crate::engine;
use crate::headers::z_type_h::loc;
use crate::headers::list_mon_race_flags_h::{RF_HURT_LIGHT, RF_NEVER_MOVE, RF_RAND_25, RF_RAND_50};
use crate::params::{value as param, Param};
use crate::state::BorgState;

/// A believed record and its trace details at aim time.
#[derive(Clone)]
pub(crate) struct BelievedTarget {
    pub(crate) index: usize,
    pub(crate) race: u16,
    pub(crate) pos: loc,
    pub(crate) weight: i32,
    pub(crate) age: i32,
}

/// One projection, staged at aim time and armed only when its command is queued.
#[derive(Clone, Default)]
pub(crate) struct PendingAttack {
    targets: Vec<BelievedTarget>,
    queued: bool,
    /// #889: unseen deaths ("It dies.") among the command's messages.
    deaths: u8,
}

impl PendingAttack {
    pub(crate) fn stage(&mut self, targets: Vec<BelievedTarget>) {
        self.targets = targets;
        self.queued = false;
        self.deaths = 0;
    }

    pub(crate) fn command_queued(&mut self, queued: bool) {
        if queued {
            self.queued = self.has_targets();
        } else {
            self.clear();
        }
    }

    pub(crate) fn clear(&mut self) {
        self.stage(Vec::new());
    }

    pub(crate) fn has_targets(&self) -> bool {
        !self.targets.is_empty()
    }
}

/// Cancellation messages that can reach the next command without firing.
/// See player-spell.c:504–516, cmd-obj.c (check_devices and item commands),
/// and player-attack.c (do_cmd_fire/do_cmd_throw).
pub(crate) fn reject_attack(st: &mut BorgState, msg: &[u8]) {
    if !st.fight_attack.pending_unseen.has_targets() {
        return;
    }
    let rejected = [
        // engine/angband/src/player-spell.c:spell_cast:510.
        b"You failed to concentrate hard enough!".as_slice(),
        // engine/angband/src/cmd-obj.c:check_devices:66-93 formats
        // "You failed to %s properly." with these four actions.
        b"You failed to zap the rod properly.",
        b"You failed to use the wand properly.",
        b"You failed to use the staff properly.",
        b"You failed to activate it properly.",
        b"You cannot pray or produce magics",
        b"You cannot see!",
        b"You are too confused!",
        b"There are no spells you can cast",
        b"That book has no spells that you can cast",
        b"The draining fails.",
        b"The wand has no charges left",
        b"The staff has no charges left",
        b"You have no suitable ammunition to fire",
        b"You have nothing to fire with",
        b"You have nothing to throw",
        b"That item is not within your reach",
        b"That ammo cannot be fired",
        b"You have no staves to use",
        b"You have no wands to aim",
        b"You have no rods to zap",
        b"You have no items to activate",
        b"That staff has no charges",
        b"That wand has no charges",
        b"That rod is still charging",
        b"That item is still charging",
    ];
    if rejected.iter().any(|prefix| msg.starts_with(prefix)) {
        st.fight_attack.pending_unseen.clear();
    }
}

/// The engine's death messages for a monster the player cannot see: a
/// projection's kill is `MON_MSG_DIE` / `MON_MSG_DESTROYED`
/// (src/list-mon-message.h:13-14) with the subject "It" (`get_subject`,
/// src/mon-msg.c:320-330), and a missile's or a throw's note is " dies." /
/// " is destroyed." (src/player-attack.c:1122-1123) after the name "it"
/// capitalized (src/mon-util.c:1053-1062). Two at once read "2 monsters
/// die.", which names no record.
const UNSEEN_DEATHS: [&[u8]; 2] = [b"It dies.", b"It is destroyed."];

/// #889: count an unseen death among the messages of a queued projection at
/// believed targets (read whatever `unseen_death_clears` says; only
/// `finish_attack` acts on it).
pub(crate) fn death_message(st: &mut BorgState, msg: &[u8]) {
    let pending = &mut st.fight_attack.pending_unseen;
    if pending.queued && UNSEEN_DEATHS.contains(&msg) {
        pending.deaths = pending.deaths.saturating_add(1);
    }
}

/// #889: the record an unseen death message names, when it can name one:
/// the projection had a single believed target. With several, or with none
/// ("It dies." for a monster the Borg had no record of), nothing is deleted.
pub(crate) fn believed_dead(deaths: u8, targets: &[BelievedTarget]) -> Option<BelievedTarget> {
    match targets {
        [only] if deaths > 0 => Some(only.clone()),
        _ => None,
    }
}

/// Called at the next normal command request, after parsing the completed
/// command's messages. A miss still spends a shot. A failure or flush
/// discards the pending projection before it gets here. With
/// `unseen_death_clears` = 1 it returns the record an unseen death message
/// names (`believed_dead`), for the caller to delete.
pub(crate) fn finish_attack(
    st: &mut BorgState,
    mut log: impl FnMut(&mut BorgState, BelievedTarget),
) -> Option<BelievedTarget> {
    let pending = std::mem::take(&mut st.fight_attack.pending_unseen);
    if !pending.queued {
        return None;
    }

    let mut targets = pending.targets;
    targets.sort_unstable_by_key(|target| target.index);
    targets.dedup_by_key(|target| target.index);
    let dead = if param(Param::unseen_death_clears) != 0 { believed_dead(pending.deaths, &targets) } else { None };
    for target in targets {
        note_shot(st, target.index);
        log(st, target);
    }
    dead
}

pub(crate) fn confirm_attack(st: &mut BorgState) {
    let dead = finish_attack(st, |st, target| {
        crate::borg_io::borg_note(
            st,
            &format!(
                "# Unseen target (#879): {} believed at ({},{}), seen {} turns ago, belief {}%",
                engine::race_name(usize::from(target.race)).iter().map(|&c| c as char).collect::<String>(),
                target.pos.y,
                target.pos.x,
                target.age,
                target.weight
            ),
        );
    });
    // #889: the record is gone, unless its slot was reused meanwhile.
    if let Some(target) = dead.filter(|target| st.kills.kills[target.index].r_idx == target.race) {
        crate::borg_io::borg_note(
            st,
            &format!(
                "# Unseen target (#889): an unseen death after the only believed target, {} at ({},{})",
                engine::race_name(usize::from(target.race)).iter().map(|&c| c as char).collect::<String>(),
                target.pos.y,
                target.pos.x
            ),
        );
        borg_delete_kill(st, target.index as i32);
    }
}

/// #889: the share (percent) of an erratic mover's turns that are random
/// steps: RAND_25 and RAND_50 add up (src/mon-move.c:1087-1101).
pub(crate) fn erratic_percent(rand_25: bool, rand_50: bool) -> i32 {
    25 * i32::from(rand_25) + 50 * i32::from(rand_50)
}

/// The chance (percent) that a monster unseen for `age` Borg turns is still
/// at its last grid: 100 for one that never moves or is asleep, else
/// `100 - decay` per monster turn, never below 0.
///
/// #889 (`unseen_spread` = 1): the belief spread over
/// the grids around the last one. Only the monster's directed turns count
/// as leaving (`erratic` percent of its turns are random steps), and the
/// first `slack` of them do not: a blast still reaches a monster that many
/// grids from its last grid. With `unseen_spread` 0 both are 0.
#[allow(clippy::too_many_arguments)]
pub(crate) fn belief_percent_spread(
    age: i32, never_move: bool, awake: bool, monster_energy: i32, own_energy: i32, decay: i32,
    erratic: i32, slack: i32,
) -> i32 {
    if never_move || !awake {
        return 100;
    }
    let monster_turns = age.max(0) * monster_energy.max(1) / own_energy.max(1);
    let directed = monster_turns * (100 - erratic.clamp(0, 100)) / 100;
    (100 - (directed - slack.max(0)).max(0) * decay.max(0)).clamp(0, 100)
}

/// Light Room's sphere radius: `SPHERE:LIGHT_WEAK:2` (lib/gamedata/class.txt:221-224).
/// Call Light's is 1 (:727-730), never as far as a believed target (at
/// least two grids away), so only Light Room is cast.
const LIGHT_ROOM_RADIUS: i32 = 2;

/// #889: whether Light Room is cast at a record: believed at its last grid,
/// inside the sphere and in line of sight for a projection from the Borg
/// (the sphere is a `project()` from the Borg's grid,
/// src/effect-handler-attack.c:571-586), and light-sensitive (`HURT_LIGHT`,
/// the only monsters `LIGHT_WEAK` hurts) unless `mode` is 2. The sphere
/// lights the grids it reaches, so the monster is revealed either way.
pub(crate) fn reveal_target(mode: i32, believed: bool, hurt_light: bool, distance: i32, projectable: bool) -> bool {
    mode != 0 && believed && projectable && distance <= LIGHT_ROOM_RADIUS && (mode == 2 || hurt_light)
}

/// #889 (`unseen_light_reveal`): cast Light Room at a believed monster in its
/// sphere (`reveal_target`). Called before the attack choice
/// (`borg_think_dungeon`). Once lit, the grid shows the monster or the
/// record is forgotten (`borg_follow_kill`), so the cast does not repeat.
pub(crate) fn light_reveal(st: &mut BorgState) -> bool {
    let mode = param(Param::unseen_light_reveal);
    if mode == 0 {
        return false;
    }
    if st.borg.tr(BI_ISBLIND) != 0 || st.borg.tr(BI_ISCONFUSED) != 0 || st.borg.tr(BI_ISIMAGE) != 0 {
        return false;
    }
    let c = st.borg.c;
    let Some(k) = (1..usize::try_from(st.kills.nxt).unwrap_or(0)).find(|&k| {
        let kill = &st.kills.kills[k];
        kill.r_idx != 0
            && reveal_target(
                mode,
                believed_weight(st, k).is_some(),
                engine::race_flag_has(usize::from(kill.r_idx), RF_HURT_LIGHT),
                engine::distance(c, kill.pos),
                crate::borg_projection::borg_los_projectable(st, c.y, c.x, kill.pos.y, kill.pos.x),
            )
    }) else {
        return false;
    };
    let kill = st.kills.kills[k];
    if !crate::borg_magic::borg_spell_fail(st, LIGHT_ROOM, 40) {
        return false;
    }
    crate::borg_io::borg_note(
        st,
        &format!(
            "# Unseen target (#889): Light Room at {} believed at ({},{})",
            engine::race_name(usize::from(kill.r_idx)).iter().map(|&c| c as char).collect::<String>(),
            kill.pos.y,
            kill.pos.x
        ),
    );
    true
}


/// The weight (percent, 1-100) of a target the Borg believes is at the
/// record's grid without seeing it; `None` when the switch is off, the record
/// is current knowledge (seen within `attack_max_target_age`), or it is not a
/// believed target (too old, too close, its grid expected visible, belief 0).
pub(crate) fn believed_weight(st: &BorgState, k: usize) -> Option<i32> {
    believed_weight_with(st, k, 0)
}

/// `believed_weight` for a record a blast reaches with `slack` grids to spare
/// beyond its last grid (`unseen_spread`; 0 outside a blast).
pub(crate) fn believed_weight_with(st: &BorgState, k: usize, slack: i32) -> Option<i32> {
    if param(Param::unseen_belief) == 0 {
        return None;
    }
    let kill = &st.kills.kills[k];
    if kill.r_idx == 0 || kill.seen {
        return None;
    }
    let age = st.borg.time.now.wrapping_sub(kill.when);
    if age <= param(Param::attack_max_target_age) || age > param(Param::unseen_max_age) {
        return None;
    }
    if engine::distance(st.borg.c, kill.pos) < 2 {
        return None;
    }
    // A monster the Borg should see there and does not is gone (or unseen
    // for another reason it cannot model): no belief.
    if borg_follow_kill_aux(st, k, kill.pos.y, kill.pos.x) {
        return None;
    }
    let shots = shots_since_seen(st, k);
    if shots >= param(Param::unseen_max_shots) {
        return None;
    }
    let energy = |speed: i32| i32::from(engine::extract_energy(speed.clamp(0, 199) as usize));
    let race = usize::from(kill.r_idx);
    // #889: with unseen_spread 0 the erratic share and the slack are 0, and
    // belief_percent_spread is belief_percent.
    let (erratic, slack) = if param(Param::unseen_spread) != 0 {
        (erratic_percent(engine::race_flag_has(race, RF_RAND_25), engine::race_flag_has(race, RF_RAND_50)), slack)
    } else {
        (0, 0)
    };
    let w = belief_percent_spread(
        age,
        engine::race_flag_has(race, RF_NEVER_MOVE),
        // A shot wakes a sleeper (or finds nothing there).
        kill.awake || shots > 0,
        energy(i32::from(kill.speed)),
        energy(st.borg.tr(BI_SPEED)),
        param(Param::unseen_decay_percent),
        erratic,
        slack,
    );
    (w > 0).then_some(w)
}

/// Unseen shots at record `k` since the monster was last seen.
fn shots_since_seen(st: &BorgState, k: usize) -> i32 {
    match st.fight_attack.unseen_shots.get(&k) {
        Some(&(when, n)) if when == st.kills.kills[k].when => n,
        _ => 0,
    }
}

/// Count a launched shot and treat the last-known sleeper as awake.
fn note_shot(st: &mut BorgState, k: usize) {
    let when = st.kills.kills[k].when;
    let n = shots_since_seen(st, k) + 1;
    st.fight_attack.unseen_shots.insert(k, (when, n));
    st.kills.kills[k].awake = true;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn armed_attack() -> BorgState {
        let mut st = BorgState::default();
        st.kills.kills = vec![crate::borg_flow_kill::BorgKill::BLANK; 2];
        st.kills.kills[1].r_idx = 1;
        st.fight_attack.pending_unseen.stage(vec![BelievedTarget { index: 1, race: 1, pos: loc { x: 4, y: 5 }, weight: 100, age: 3 }]);
        st.fight_attack.pending_unseen.command_queued(true);
        st
    }

    fn assert_discarded(st: &mut BorgState) {
        finish_attack(st, |_, _| panic!("cancelled attack wrote an unseen-shot trace"));
        assert!(st.fight_attack.unseen_shots.is_empty());
        assert!(!st.kills.kills[1].awake);
    }

    #[test]
    fn every_engine_rejection_message_discards_the_pending_ledger() {
        let messages: &[&[u8]] = &[
            b"You failed to concentrate hard enough!",
            b"You failed to zap the rod properly.",
            b"You failed to use the wand properly.",
            b"You failed to use the staff properly.",
            b"You failed to activate it properly.",
            b"You cannot pray or produce magics.",
            b"You cannot see!",
            b"You are too confused!",
            b"There are no spells you can cast.",
            b"That book has no spells that you can cast.",
            b"The draining fails.",
            b"The wand has no charges left.",
            b"The staff has no charges left.",
            b"You have no suitable ammunition to fire.",
            b"You have nothing to fire with.",
            b"You have nothing to throw.",
            b"That item is not within your reach.",
            b"That ammo cannot be fired by your current weapon.",
            b"You have no staves to use.",
            b"You have no wands to aim.",
            b"You have no rods to zap.",
            b"You have no items to activate.",
            b"That staff has no charges.",
            b"That wand has no charges.",
            b"That rod is still charging.",
            b"That item is still charging.",
        ];
        for &message in messages {
            let mut st = armed_attack();
            // Stub the engine response and normal command input.
            reject_attack(&mut st, message);
            assert!(!st.fight_attack.pending_unseen.has_targets(), "{}", String::from_utf8_lossy(message));
            assert_discarded(&mut st);
        }
    }

    #[test]
    fn a_flush_discards_the_pending_ledger() {
        let mut st = armed_attack();
        crate::borg_io::borg_flush(&mut st);
        assert!(!st.fight_attack.pending_unseen.has_targets());
        assert_discarded(&mut st);
    }

    #[test]
    fn lock_and_disarm_failures_leave_the_pending_ledger_armed() {
        // engine/angband/src/cmd-cave.c:209,767,774,839 and obj-chest.c:615,716.
        let messages: &[&[u8]] = &[
            b"You failed to pick the lock.",
            b"You failed to lock the door.",
            b"You failed to disarm the pit.",
            b"You failed to disarm the chest.",
        ];
        for &message in messages {
            let mut st = armed_attack();
            // Stub a failure message and the parser's key-buffer cleanup.
            reject_attack(&mut st, message);
            crate::borg_io::borg_flush_keys(&mut st);
            finish_attack(&mut st, |_, _| ());
            assert_eq!(shots_since_seen(&st, 1), 1, "{}", String::from_utf8_lossy(message));
        }
    }

    fn belief_percent_test(age: i32, never_move: bool, awake: bool, monster_energy: i32, own_energy: i32, decay: i32) -> i32 {
        belief_percent_spread(age, never_move, awake, monster_energy, own_energy, decay, 0, 0)
    }

    #[test]
    fn belief_decays_with_the_monsters_turns() {
        // immobile or asleep: still there
        assert_eq!(belief_percent_test(9, true, true, 10, 10, 20), 100);
        assert_eq!(belief_percent_test(9, false, false, 10, 10, 20), 100);
        // normal speed: 20% a turn
        assert_eq!(belief_percent_test(3, false, true, 10, 10, 20), 40);
        // a slow monster (half the energy) has half the turns
        assert_eq!(belief_percent_test(4, false, true, 5, 10, 20), 60);
        // a fast one has twice as many; never below 0
        assert_eq!(belief_percent_test(4, false, true, 20, 10, 20), 0);
        assert_eq!(belief_percent_test(0, false, true, 10, 10, 20), 100);
    }

    fn target(index: usize) -> BelievedTarget {
        BelievedTarget { index, race: index as u16, pos: loc { x: 4, y: 5 }, weight: 100, age: 3 }
    }

    /// #889, item 2: an unseen death names the record only when the
    /// projection had a single believed target.
    #[test]
    fn an_unseen_death_names_only_a_single_believed_target() {
        assert_eq!(believed_dead(1, &[target(1)]).map(|t| t.index), Some(1));
        assert_eq!(believed_dead(2, &[target(1)]).map(|t| t.index), Some(1));
        assert!(believed_dead(0, &[target(1)]).is_none(), "no death message");
        assert!(believed_dead(1, &[target(1), target(2)]).is_none(), "which one died?");
        assert!(believed_dead(1, &[]).is_none());
    }

    /// #889, item 2: only the engine's unseen death messages count, and only
    /// for a queued projection; the switch (shipped 0) keeps the record.
    #[test]
    fn unseen_death_messages_count_for_a_queued_projection_only() {
        let mut st = armed_attack();
        for msg in [b"It dies.".as_slice(), b"It is destroyed.", b"The cave spider dies.", b"2 monsters die.", b"It is hit."] {
            death_message(&mut st, msg);
        }
        assert_eq!(st.fight_attack.pending_unseen.deaths, 2);
        assert!(finish_attack(&mut st, |_, _| ()).is_none(), "unseen_death_clears ships 0");
        assert_eq!(shots_since_seen(&st, 1), 1, "the shot still counts");

        let mut st = BorgState::default();
        st.fight_attack.pending_unseen.stage(vec![target(1)]);
        death_message(&mut st, b"It dies.");
        assert_eq!(st.fight_attack.pending_unseen.deaths, 0, "not queued yet");
    }

    /// #889, item 4: erratic movers and blast slack spread the belief;
    /// with neither, the decay is the shipped one.
    #[test]
    fn the_belief_spreads_for_erratic_movers_and_blasts() {
        assert_eq!(erratic_percent(false, false), 0);
        assert_eq!(erratic_percent(true, false), 25);
        assert_eq!(erratic_percent(false, true), 50);
        assert_eq!(erratic_percent(true, true), 75);
        // 4 turns at 20%: 20 left; a RAND_50 mover took 2 directed steps.
        assert_eq!(belief_percent_spread(4, false, true, 10, 10, 20, 0, 0), 20);
        assert_eq!(belief_percent_spread(4, false, true, 10, 10, 20, 50, 0), 60);
        // A radius-2 ball at its last grid still hits after 2 steps.
        assert_eq!(belief_percent_spread(4, false, true, 10, 10, 20, 0, 2), 60);
        assert_eq!(belief_percent_spread(4, false, true, 10, 10, 20, 50, 2), 100);
        // Never above 100, never below 0; sleepers and NEVER_MOVE stay 100.
        assert_eq!(belief_percent_spread(1, false, true, 10, 10, 20, 0, 5), 100);
        assert_eq!(belief_percent_spread(9, false, true, 10, 10, 20, 0, 1), 0);
        assert_eq!(belief_percent_spread(9, false, false, 10, 10, 20, 0, 0), 100);
    }

    /// #889, item 3: Light Room is cast only at a believed record inside its
    /// sphere and in line of sight; mode 1 needs HURT_LIGHT, 2 does not.
    #[test]
    fn light_room_reveals_a_believed_monster_inside_its_sphere() {
        assert!(reveal_target(1, true, true, 2, true));
        assert!(!reveal_target(1, true, false, 2, true), "not light-sensitive");
        assert!(reveal_target(2, true, false, 2, true));
        assert!(!reveal_target(0, true, true, 2, true), "switch off");
        assert!(!reveal_target(1, false, true, 2, true), "not believed");
        assert!(!reveal_target(1, true, true, 3, true), "outside the sphere");
        assert!(!reveal_target(1, true, true, 2, false), "behind a wall");
    }
}
