//! Committed direction selection and stair route policy.

use borg_rs::decision_api::borg_flow::GOAL_FLEE;
use borg_rs::decision_api::borg_flow_stairs::{
    more_refused, rising_holds_flee_descent, CombinedStairPlan, StairDirection, StairPlan,
};
use borg_rs::decision_api::engine;
use borg_rs::decision_api::headers::list_options_h::OPT_birth_force_descend;
use borg_rs::decision_api::headers::z_type_h::loc;

use borg_rs::memory::{BorgState, AUTO_MAX_X, AUTO_MAX_Y, GOAL_DIGGING};

struct CommittedFlow<'a> {
    costs: &'a [[u8; AUTO_MAX_X as usize]],
    position: loc,
    depth: i32,
    digging: bool,
    first: loc,
}

impl<'a> CommittedFlow<'a> {
    fn read(state: &'a BorgState) -> Self {
        let (y, x) = state.flow.first();
        Self {
            costs: &state.flow.data_flow,
            position: state.borg.c,
            depth: crate::Read::new(state).character().cdepth(),
            digging: i32::from(state.borg.goal.type_0) == GOAL_DIGGING,
            first: loc { x, y },
        }
    }

    fn scaled_cost(&self, x: i32, y: i32) -> i32 {
        i32::from(self.costs[y as usize][x as usize]) * 10
    }
}

pub(super) fn select_step(state: &BorgState) -> i32 {
    borg_rs::decision_tape::mark("policy::flow.select_step", String::new());
    let flow = CommittedFlow::read(state);
    let mut best_cost = flow.scaled_cost(flow.position.x, flow.position.y) - 5;
    let mut direction = -1;
    let mut tied = 0;

    for index in 0..8 {
        let x = flow.position.x + i32::from(engine::ddx_ddd(index));
        let y = flow.position.y + i32::from(engine::ddy_ddd(index));
        let cost = flow.scaled_cost(x, y);
        if cost > best_cost || !(1..AUTO_MAX_X).contains(&x) || !(1..AUTO_MAX_Y).contains(&y) {
            continue;
        }
        if cost < best_cost {
            tied = 0;
        }

        if flow.depth == 0 {
            tied += 1;
            if tied >= 2 && engine::randint0(tied as u32) != 0 {
                continue;
            }
        } else if flow.depth >= 1 {
            tied += 1;
            if tied >= 2 {
                continue;
            }
        }

        // The native tie check happens before the anti-summoning filter.
        if flow.digging
            && (engine::ddx_ddd(index) == 0 || engine::ddy_ddd(index) == 0)
            && engine::distance(flow.position, flow.first) <= 2
        {
            continue;
        }
        direction = index as i32;
        best_cost = cost;
    }

    direction
}

pub(super) fn plan_up_stair(state: &BorgState, why: i32) -> StairPlan {
    borg_rs::decision_tape::mark("policy::flow.plan_up_stair", String::new());
    if engine::player_opt(OPT_birth_force_descend) || state.stairs.less.num == 0 {
        return StairPlan::Refused;
    }

    let leaving = state.borg.goal.fleeing || state.borg.goal.leaving;
    let weight = crate::stateful::caution::stair_flight_weight(why, leaving);
    if weight <= 0 {
        return StairPlan::Unweighted;
    }
    let character = crate::Read::new(state).character();
    let avoid = character.clevel() <= 35 && character.light() != 0 && !state.borg.status.desperate;
    StairPlan::Weighted { weight, avoid }
}

pub(super) fn plan_down_stair(state: &mut BorgState, why: i32, brave: bool) -> StairPlan {
    borg_rs::decision_tape::mark("policy::flow.plan_down_stair", String::new());
    if state.stairs.more.num == 0 {
        return StairPlan::Refused;
    }
    if why == GOAL_FLEE && rising_holds_flee_descent(state) {
        return StairPlan::Refused;
    }
    if more_refused(state, brave) {
        return StairPlan::Refused;
    }
    let weight = crate::stateful::caution::stair_flight_weight(why, false);
    if weight > 0 {
        StairPlan::Weighted {
            weight,
            avoid: false,
        }
    } else {
        StairPlan::Unweighted
    }
}

pub(super) fn plan_combined_stairs(
    state: &mut BorgState,
    why: i32,
    only: Option<StairDirection>,
) -> CombinedStairPlan {
    borg_rs::decision_tape::mark("policy::flow.plan_combined_stairs", String::new());
    let up = !matches!(only, Some(StairDirection::More));
    let down = !matches!(only, Some(StairDirection::Less));
    if (!up || state.stairs.less.num == 0) && (!down || state.stairs.more.num == 0) {
        return CombinedStairPlan::Refused;
    }

    let read = crate::Read::new(state);
    let character = read.character();
    let status = read.status();
    if !state.borg.goal.fleeing
        && !state.borg.mon.scary
        && state.stairs.less.num == 0
        && i32::from(state.borg.avoidance) <= character.curhp() * 15 / 10
        && (status.isweak() != 0 || status.ishungry() != 0 || read.food().food() < 2)
    {
        return CombinedStairPlan::Refused;
    }
    if character.light() == 0 && character.cdepth() != 0 {
        return CombinedStairPlan::Refused;
    }

    let down = down && !(why == GOAL_FLEE && rising_holds_flee_descent(state));
    let weight = crate::stateful::caution::stair_flight_weight(why, false);
    if weight <= 0 {
        return CombinedStairPlan::Unweighted { up, down };
    }
    let only = match (up, down) {
        (true, true) => None,
        (true, false) => Some(StairDirection::Less),
        (false, true) => Some(StairDirection::More),
        (false, false) => return CombinedStairPlan::Refused,
    };
    CombinedStairPlan::Weighted { only, weight }
}
