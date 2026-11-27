//! Direction selection on an already committed, unweighted flow (#1298).

use borg_rs::decision_api::engine;
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
