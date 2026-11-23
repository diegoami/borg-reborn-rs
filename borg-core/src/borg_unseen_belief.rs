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

use crate::borg_flow_kill::borg_follow_kill_aux;
use crate::borg_headers::borg_trait_h::BI_SPEED;
use crate::engine;
use crate::headers::z_type_h::loc;
use crate::headers::list_mon_race_flags_h::RF_NEVER_MOVE;
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
}

impl PendingAttack {
    pub(crate) fn stage(&mut self, targets: Vec<BelievedTarget>) {
        self.targets = targets;
        self.queued = false;
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

/// Called at the next normal command request, after parsing the completed
/// command's messages. A miss still spends a shot. A failure or flush
/// discards the pending projection before it gets here.
pub(crate) fn finish_attack(st: &mut BorgState, mut log: impl FnMut(&mut BorgState, BelievedTarget)) {
    let pending = std::mem::take(&mut st.fight_attack.pending_unseen);
    if !pending.queued {
        return;
    }

    let mut targets = pending.targets;
    targets.sort_unstable_by_key(|target| target.index);
    targets.dedup_by_key(|target| target.index);
    for target in targets {
        note_shot(st, target.index);
        log(st, target);
    }
}

pub(crate) fn confirm_attack(st: &mut BorgState) {
    finish_attack(st, |st, target| {
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
}

/// The chance (percent) that a monster unseen for `age` Borg turns is still
/// at its last grid.
pub(crate) fn belief_percent(age: i32, never_move: bool, awake: bool, monster_energy: i32, own_energy: i32, decay: i32) -> i32 {
    if never_move || !awake {
        return 100;
    }
    let monster_turns = age.max(0) * monster_energy.max(1) / own_energy.max(1);
    (100 - monster_turns * decay.max(0)).clamp(0, 100)
}

/// The weight (percent, 1-100) of a target the Borg believes is at the
/// record's grid without seeing it; `None` when the switch is off, the record
/// is current knowledge (seen within `attack_max_target_age`), or it is not a
/// believed target (too old, too close, its grid expected visible, belief 0).
pub(crate) fn believed_weight(st: &BorgState, k: usize) -> Option<i32> {
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
    let w = belief_percent(
        age,
        engine::race_flag_has(usize::from(kill.r_idx), RF_NEVER_MOVE),
        // A shot wakes a sleeper (or finds nothing there).
        kill.awake || shots > 0,
        energy(i32::from(kill.speed)),
        energy(st.borg.tr(BI_SPEED)),
        param(Param::unseen_decay_percent),
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

    #[test]
    fn belief_decays_with_the_monsters_turns() {
        // immobile or asleep: still there
        assert_eq!(belief_percent(9, true, true, 10, 10, 20), 100);
        assert_eq!(belief_percent(9, false, false, 10, 10, 20), 100);
        // normal speed: 20% a turn
        assert_eq!(belief_percent(3, false, true, 10, 10, 20), 40);
        // a slow monster (half the energy) has half the turns
        assert_eq!(belief_percent(4, false, true, 5, 10, 20), 60);
        // a fast one has twice as many; never below 0
        assert_eq!(belief_percent(4, false, true, 20, 10, 20), 0);
        assert_eq!(belief_percent(0, false, true, 10, 10, 20), 100);
    }
}
