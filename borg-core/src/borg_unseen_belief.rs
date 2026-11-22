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
//! that the monster is still there. Melee never takes such a target.
//!
//! The engine stops a bolt at the first monster on its path (src/project.c,
//! `PROJECT_STOP`), so a monster that closed in along the line is still hit; the
//! Borg aims by location (`borg_target`: `*`, `p`, the cursor, `5`; a free-mode
//! location target, src/ui-target.c:1479-1494, src/target.c:180-191).
//!
//! The belief (a hypothesis; the zoo measures it, RULES 14): 100% for a monster
//! that never moves (`NEVER_MOVE`) or that the Borg believes asleep; otherwise
//! `100 - unseen_decay_percent` per turn the monster has had since it was
//! seen (Borg turns times its energy over the Borg's, `extract_energy`), never
//! below 0. Erratic movers (`RAND_25`/`RAND_50`, src/mon-move.c:1087-1101) are
//! not treated apart.

use crate::borg_flow_kill::borg_follow_kill_aux;
use crate::borg_headers::borg_trait_h::BI_SPEED;
use crate::engine;
use crate::headers::list_mon_race_flags_h::RF_NEVER_MOVE;
use crate::params::{value as param, Param};
use crate::state::BorgState;

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

/// Count a shot at the believed record `k` when the chosen projection is aimed. The engine gives no name for a monster the player cannot
/// see ("It dies."), so a record shot dead stays until this cap: at most
/// `unseen_max_shots` shots per sighting.
pub(crate) fn note_shot(st: &mut BorgState, k: usize) {
    let when = st.kills.kills[k].when;
    let n = shots_since_seen(st, k) + 1;
    st.fight_attack.unseen_shots.insert(k, (when, n));
}

#[cfg(test)]
mod tests {
    use super::belief_percent;

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
