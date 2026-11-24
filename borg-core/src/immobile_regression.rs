//! Fabricated Borg observations with a live engine and the shipped profile.

use crate::borg_cave_view::BORG_VIEW;
use crate::borg_caution::{borg_caution, in_walk_away_reach, walk_away_listed, walk_away_step};
use crate::borg_escape::PhaseVerdict;
use crate::borg_flow_kill::BorgKill;
use crate::borg_flow_stairs::borg_flow_stair_more;
use crate::borg_headers::borg_trait_h::{
    BI_AMMO_POWER, BI_AMMO_TVAL, BI_APHASE, BI_CDEPTH, BI_CLEVEL, BI_CURHP, BI_FOOD, BI_FRACT,
    BI_LIGHT, BI_MAXDEPTH, BI_MAXHP, BI_SHOTS,
};
use crate::borg_io::{borg_inkey, ESCAPE};
use crate::engine;
use crate::headers::list_terrain_h::{FEAT_FLOOR, FEAT_GRANITE, FEAT_MORE};
use crate::headers::list_tvals_h::{TV_ARROW, TV_SCROLL};
use crate::headers::z_type_h::loc;
use crate::params::{value, Param};
use crate::state::BorgState;

const GOAL_FLEE: i32 = 7;
const START: loc = loc { x: 40, y: 20 };
const WEST: loc = loc { x: 39, y: 20 };
const EAST: loc = loc { x: 41, y: 20 };
const EYE: loc = loc { x: 41, y: 19 };

fn put(st: &mut BorgState, grid: loc, feat: u32) {
    st.cave.grid_mut(grid.x, grid.y).feat = feat as u8;
}

fn cave(st: &mut BorgState, feat: u32) {
    crate::borg_cave::borg_init_cave(st);
    for y in 1..40 {
        for x in 1..60 {
            put(st, loc { x, y }, feat);
        }
    }
    put(st, START, FEAT_FLOOR);
    st.borg.c = START;
    st.borg.time.now = 100;
    st.borg.time.level = 20;
    st.borg.goal.recalling = 0;
    st.borg.avoidance = 72;
    let pack_size = usize::from(engine::z_info().pack_size);
    st.items.items[..pack_size].fill(crate::borg_item::blank_item());
    st.danger = Default::default();
    st.kills.cnt = 0;
    st.kills.nxt = 1;
    st.stairs.less.num = 0;
    st.stairs.more.num = 0;
    for (index, val) in [
        (BI_CURHP, 72),
        (BI_MAXHP, 72),
        (BI_CLEVEL, 5),
        (BI_LIGHT, 2),
        (BI_FOOD, 3),
        (BI_FRACT, 0),
        (BI_APHASE, 0),
    ] {
        st.borg.trait_[index as usize] = val;
    }
    st.borg.trait_[BI_MAXDEPTH as usize] = st.borg.tr(BI_CDEPTH) + 1;
}

fn monster(st: &mut BorgState, name: &[u8], at: loc) -> usize {
    let r_idx = (1..usize::from(engine::r_max()))
        .find(|&r| engine::race_name(r) == name)
        .expect("fixture race exists in the engine");
    let i = st.kills.nxt as usize;
    st.kills.kills[i] = BorgKill {
        r_idx: r_idx as u16,
        known: true,
        awake: true,
        pos: at,
        speed: 110,
        moves: 10,
        power: 11,
        when: st.borg.time.now,
        ..BorgKill::BLANK
    };
    st.cave.grid_mut(at.x, at.y).kill = i as u8;
    put(st, at, FEAT_FLOOR);
    st.kills.nxt += 1;
    st.kills.cnt += 1;
    i
}

fn check_keys(st: &mut BorgState, expected: &[u32]) {
    assert_eq!(st.io.queued_keys(), expected);
    for _ in expected {
        borg_inkey(st, true);
    }
}

// Only the switch changes across runs. The real landing sampler and the
// scroll selector both have to accept before the read keys are emitted.
fn cornered_phase(st: &mut BorgState, with_phase_override: bool) {
    cave(st, FEAT_FLOOR);
    for y in START.y - 1..=START.y + 1 {
        for x in START.x - 1..=START.x + 1 {
            if (x, y) != (START.x, START.y) {
                put(st, loc { x, y }, FEAT_GRANITE);
            }
        }
    }
    let eye = monster(st, b"floating eye", EYE);
    put(st, EAST, FEAT_MORE);
    assert!(in_walk_away_reach(st, EAST.x, EAST.y));
    assert!(!engine::no_light(), "the fixture must be able to read");
    let sval = st.item_val.sv_scroll_phase_door;
    assert!(sval > 0);
    let item = &mut st.items.items[0];
    item.tval = TV_SCROLL as u8;
    item.sval = sval as u8;
    item.iqty = 1;
    item.aware = true;
    st.borg.trait_[BI_APHASE as usize] = 1;

    assert_eq!(value(Param::escape_paralyser_fight_lost), i32::from(with_phase_override));
    assert_eq!(crate::borg_escape::paralyser_defers_early_fight(st), with_phase_override);
    assert_eq!(walk_away_step(st), with_phase_override);
    let verdict = if with_phase_override { PhaseVerdict::Phase } else { PhaseVerdict::Fight };
    assert_eq!(st.borg.phase_rule.map(|(_, verdict, _)| verdict), Some(verdict));
    if with_phase_override {
        check_keys(st, &[ESCAPE, ESCAPE, u32::from(b'r'), u32::from(b'a')]);
    } else {
        check_keys(st, &[]);
        assert_eq!(st.borg.mon.walk_away_cornered, Some(st.borg.time.now));
    }

    // A newly seen eye may still be recorded as asleep. The override needs
    // a known awake adjacent enemy even though walk-away also covers sleepers.
    st.borg.time.now += 1;
    st.kills.kills[eye].awake = false;
    assert!(!crate::borg_escape::paralyser_defers_early_fight(st));
    assert!(!walk_away_step(st));
    assert_eq!(st.borg.phase_rule.map(|(_, verdict, _)| verdict), Some(PhaseVerdict::Shoot));
    check_keys(st, &[]);
}

fn retention(st: &mut BorgState, remember_all: bool) {
    assert_eq!(value(Param::caution_remember_never_move), i32::from(remember_all));
    for reset in [true, false] {
        cave(st, FEAT_GRANITE);
        let eye = monster(st, b"floating eye", EYE);
        let mold_at = loc { x: 43, y: 20 };
        let mold = monster(st, b"grey mold", mold_at);
        let rat_at = loc { x: 44, y: 20 };
        let rat = monster(st, b"giant white mouse", rat_at);
        assert!(walk_away_listed(st, usize::from(st.kills.kills[eye].r_idx)));
        assert!(!walk_away_listed(st, usize::from(st.kills.kills[mold].r_idx)));
        if reset {
            st.borg.antibounce_count = 500;
            crate::borg_think_dungeon::borg_check_bounciness(st);
        } else {
            st.borg.time.now += 1999;
            crate::borg_update::expire_kills(st);
            assert_ne!(st.kills.kills[mold].r_idx, 0, "the fresh record survives");
            st.borg.time.now += 1;
            crate::borg_update::expire_kills(st);
        }
        assert_ne!(st.kills.kills[eye].r_idx, 0, "the listed eye stays known");
        assert!(in_walk_away_reach(st, EAST.x, EAST.y));
        assert_eq!(st.kills.kills[mold].r_idx != 0, remember_all);
        assert_eq!(st.cave.grid(mold_at.x, mold_at.y).kill != 0, remember_all);
        assert_eq!(st.kills.kills[rat].r_idx, 0, "a mobile race still clears");
    }
}

fn stair_inside_reach(st: &mut BorgState) {
    cave(st, FEAT_GRANITE);
    let eye = monster(st, b"floating eye", EYE);
    put(st, WEST, FEAT_FLOOR);
    put(st, EAST, FEAT_MORE);
    st.stairs.more.num = 1;
    st.stairs.more.x[0] = EAST.x;
    st.stairs.more.y[0] = EAST.y;
    assert!(in_walk_away_reach(st, EAST.x, EAST.y));
    assert!(!in_walk_away_reach(st, WEST.x, WEST.y));

    assert!(crate::borg_think_dungeon::borg_think_dungeon(st));
    check_keys(st, &[u32::from(b'4')]);
    st.borg.c = WEST;
    st.borg.time.now += 1;
    assert!(!borg_flow_stair_more(st, GOAL_FLEE, false, true), "the only stair stays blocked");
    check_keys(st, &[]);

    // A visible shot from outside the eye's reach stays available even
    // though the flow will not enter its reach on foot.
    put(st, loc { x: 40, y: 19 }, FEAT_FLOOR);
    st.cave.grid_mut(EYE.x, EYE.y).info |= BORG_VIEW;
    st.flow.temp_n = 0;
    st.flow.temp_push(EYE.y, EYE.x);
    st.traits.simulate = true;
    let bow = (crate::borg_item::inven_wield() + 1) as usize;
    let quiver = crate::borg_item::quiver_start() as usize;
    st.items.items[bow] = crate::borg_item::blank_item();
    st.items.items[quiver] = crate::borg_item::blank_item();
    assert_eq!(crate::borg_fight_attack::borg_attack_aux_launch(st), 0);
    st.items.items[bow].iqty = 1;
    let arrow = &mut st.items.items[quiver];
    arrow.tval = TV_ARROW as u8;
    arrow.iqty = 3;
    arrow.value = 1;
    arrow.dd = 1;
    arrow.ds = 4;
    st.borg.trait_[BI_AMMO_TVAL as usize] = TV_ARROW as i32;
    st.borg.trait_[BI_AMMO_POWER as usize] = 2;
    st.borg.trait_[BI_SHOTS as usize] = 10;
    assert!(crate::borg_fight_attack::borg_attack_aux_launch(st) > 0);
    st.traits.simulate = false;
    check_keys(st, &[]);

    // Removing the eye makes the same route usable. This also checks that
    // the travel policy was eligible to start in the first place.
    crate::borg_flow_kill::borg_delete_kill(st, eye as i32);
    assert!(borg_flow_stair_more(st, GOAL_FLEE, false, true));
    check_keys(st, &[u32::from(b'6')]);

    // The same stair is usable when the Borg starts on it beside the eye.
    monster(st, b"floating eye", EYE);
    st.borg.trait_[BI_MAXDEPTH as usize] = st.borg.tr(BI_CDEPTH) + 1;
    st.borg.c = EAST;
    st.borg.time.now += 1;
    assert!(walk_away_step(st));
    check_keys(st, &[u32::from(b'>')]);
}

fn strategic_retreat(st: &mut BorgState) {
    cave(st, FEAT_GRANITE);
    put(st, WEST, FEAT_FLOOR);
    put(st, loc { x: 38, y: 20 }, FEAT_FLOOR);
    st.view.view_n = 2;
    st.view.view_x[1] = WEST.x as u8;
    st.view.view_y[1] = WEST.y as u8;
    st.borg.time.antisummon = -1000;
    // Between the shipped strategic-retreat and ordinary back-away thresholds.
    st.danger.fear_monsters[START.y as usize][START.x as usize] = 27;

    assert_eq!(value(Param::caution_retreat_oscillation_limit), 3);
    assert_eq!(value(Param::caution_retreat_oscillation_thinks), 3);
    assert_eq!(value(Param::caution_retreat_oscillation_backaway), 0);
    for attempt in 1..=3 {
        assert!(borg_caution(st), "strategic retreat {attempt} should act");
        check_keys(st, &[u32::from(b'4')]);
        assert_eq!(st.borg.mon.retreat_run, attempt);
        st.borg.c = WEST;
        st.borg.time.now += 1;
        // The observed landing is part of the shipped decision chain.
        let _ = borg_caution(st);
        check_keys(st, &[]);
        assert!(st.borg.mon.retreat_landed);
        st.borg.c = START;
        st.borg.time.now += 1;
    }
    let _ = borg_caution(st);
    assert_eq!(st.borg.mon.retreat_noted, st.borg.time.now, "the fourth retreat is vetoed");
    assert_eq!(st.borg.mon.retreat_run, 3);
    check_keys(st, &[]);

    st.borg.time.now += 1;
    let _ = borg_caution(st);
    assert_eq!(st.borg.mon.retreat_noted, st.borg.time.now, "the third think is still inside the window");
    check_keys(st, &[]);

    st.borg.time.now += 1;
    assert!(borg_caution(st), "the expired window permits a new retreat");
    check_keys(st, &[u32::from(b'4')]);
    assert_eq!(st.borg.mon.retreat_run, 1);
}

pub(crate) fn run(st: &mut BorgState, case: &str) -> ! {
    assert!(st.borg.tr(BI_CDEPTH) > 0);
    assert_eq!(value(Param::caution_walk_away_immobile), 1);
    match case {
        "phase-shipped" => cornered_phase(st, false),
        "phase-override" => cornered_phase(st, true),
        "memory-shipped" => retention(st, false),
        "memory-override" => retention(st, true),
        "stair-in-reach" => stair_inside_reach(st),
        "strategic-retreat" => strategic_retreat(st),
        _ => panic!("unknown immobile decision case: {case}"),
    }
    println!("immobile decision passed: {case}");
    std::process::exit(0);
}
