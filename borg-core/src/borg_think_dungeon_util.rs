
#1211: the recorded DL19 recovery loop driven through the production
/// dungeon think — `borg_think_dungeon` on the fabricated cave, not a test
/// copy of either gate. The region is Borg261124230918's memory row: the
/// corridor y = 55 runs x = 50..85, known floor under the Borg's feet and
/// under the five sleeping cave orcs (ARROW, innate-freq 13) it had seen at
/// the recorded grids x = 75..80 — asleep (sleepiness 70, they never fired
/// in the 3,642 recorded turns), on the same straight row, so their arrow
/// danger prices every grid within max_range (20) while a destination at
/// Chebyshev distance > 8 from each of them skips the rest veto's monster
/// checks. The corridor prices danger 20-25 — 5 per orc in range, exactly
/// the recorded Loc notes (20 at x = 59 where the fifth orc falls out of
/// range, 25 from x = 60). The Priest (driver case `-class 3`) stands at
/// (66,55), HP 170/170, SP per case, Heroism (book 0, offset 5, cost 2 SP)
/// known.
///
/// Off (the parameters at C's 0): the think casts Heroism out of the
/// regenerated mana at full HP ("some danger": 17 < 25 < 85 — the recorded
/// keys m,a,f, defense type 12), then paces between recovery grids — each
/// `borg_flow_recover` picks the nearest happy grid at least 7 away (the
/// walk 66 to 59 and back; at (59,55) four orcs still price 20), the walk
/// there ends, `borg_recover` refuses the mana rest at danger 20-25 (its
/// gate 17) and the next route starts back: the pendulum. On (both shipped
/// 1): the cast is declined, the Borg rests where it stands ("Resting to
/// gain Mana"), and — with a route live, in the think's own decision order
/// (continue_recovery first, recover where the route ends) — the endpoint
/// queues the rest command instead of the next route (review of #1215, P3).
#[cfg(feature = "test-stair-handoff")]
pub(crate) fn run_recovery_mana_loop_regression(st: &mut BorgState, on: bool) -> ! {
    use crate::borg_flow::borg_flow_clear;
    use crate::borg_headers::borg_trait_h::{
        BI_CDEPTH, BI_CLASS, BI_CLEVEL, BI_CURHP, BI_CURSP, BI_FOOD, BI_LIGHT, BI_MAXHP, BI_MAXSP,
        BI_SPEED,
    };
    use crate::borg_headers::borg_magic_h::HEROISM;
    use crate::borg_magic::{BORG_MAGIC_KNOW, borg_get_spell_number};
    use crate::headers::list_mon_spells_h::RSF_ARROW;
    use crate::headers::list_terrain_h::{FEAT_FLOOR, FEAT_GRANITE};

    // The corridor row, as the recording's memory: floor under the Borg and
    // under the orcs it had seen.
    const ROW: i32 = 55;
    let floor = |y: i32, x: i32| -> bool { y == ROW && (50..=85).contains(&x) };
    let orcs = [(ROW, 75), (ROW, 76), (ROW, 77), (ROW, 79), (ROW, 80)];

    let setup = |st: &mut BorgState, sp: i32| {
        assert!(engine::option_set(c"birth_force_descend", false), "the birth option exists");
        crate::borg_cave::borg_init_cave(st);
        for y in (ROW - 4)..=(ROW + 6) {
            for x in 40..=90 {
                st.cave.grid_mut(x, y).feat = if floor(y, x) { FEAT_FLOOR as u8 } else { FEAT_GRANITE as u8 };
            }
        }
        st.borg.c = loc { x: 66, y: ROW };
        st.borg.avoidance = 170;
        for (index, value) in [
            (BI_CDEPTH, 19), (BI_CLASS, 3), (BI_CLEVEL, 26), (BI_CURHP, 170), (BI_MAXHP, 170),
            (BI_CURSP, sp), (BI_MAXSP, 47), (BI_SPEED, 110), (BI_LIGHT, 1), (BI_FOOD, 5),
        ] {
            st.borg.trait_[index as usize] = value;
        }
        // Heroism (book 0, offset 5, cost 2 SP) known and carried: the
        // recorded Priest's first-book prayer (its cast note read "(0,5)",
        // book 0 on the cast screen, letter 'a', spell letter 'f'). The
        // book index is the cast screen's slot, only checked non-negative
        // on the decision path; the keys stay queued.
        let hero = borg_get_spell_number(st, HEROISM);
        assert!(hero >= 0, "the priest's spell table carries Heroism");
        st.magic.magics[hero as usize].status = BORG_MAGIC_KNOW;
        st.borg.book_idx[st.magic.magics[hero as usize].book as usize] = 0;
        // The five remembered orcs, awake, out of sight: what the recorded
        // kill list held. The fields are what borg_update_kill would set.
        let mut r_idx = 0;
        for i in 1..i32::from(engine::z_info().r_max) {
            if engine::race_name(i as usize) == b"cave orc" {
                r_idx = i;
                break;
            }
        }
        assert!(r_idx > 0, "the game data has the cave orc");
        let race = engine::race(r_idx as usize);
        st.kills.cnt = orcs.len() as i16;
        st.kills.nxt = 1;
        for (n, (y, x)) in orcs.iter().enumerate() {
            let kill = &mut st.kills.kills[1 + n];
            kill.r_idx = r_idx as u16;
            kill.pos = loc { x: *x, y: *y };
            kill.ox = *x as u8;
            kill.oy = *y as u8;
            kill.awake = false;
            kill.speed = 110;
            kill.level = race.level as i16;
            kill.power = 55;
            kill.when = st.borg.time.now;
            kill.ranged_attack = 1;
            kill.spell[0] = RSF_ARROW as u8;
            for (slot, word) in kill.spell_flags.iter_mut().zip(race.spell_flags) {
                *slot = u32::from(word);
            }
        }
        st.kills.nxt = 1 + orcs.len() as i16;
        st.danger = Default::default();
        st.borg.mon.scary = false;
        st.borg.mon.breeders = 0;
        st.borg.near.breeder = false;
        st.borg.antibounce_count = 0;
        st.borg.no_rest_prep = 0;
        st.borg.temp.hero = false;
        st.borg.temp.bless = false;
        st.borg.temp.berserk = false;
        st.borg.goal.type_0 = 0;
        st.borg.goal.fleeing = false;
        st.borg.goal.leaving = false;
        st.borg.stair_less = false;
        st.borg.stair_more = false;
        st.borg.times_twitch = 2;
        st.borg.status.vault = false;
        // Leave food in the pack and preserve equipped gear; a sentinel
        // power keeps every other item decision out of the think (the
        // recipe). The decision's book index is set above.
        let pack = i32::from(engine::z_info().pack_size) as usize;
        for slot in &mut st.items.items[..pack] {
            if slot.iqty != 0 && crate::borg_inventory::borg_wield_slot(slot) >= 0 {
                *slot = crate::borg_item::blank_item();
            }
        }
        st.borg.power = i32::MAX / 2;
        borg_flow_clear(st);
        crate::borg_io::borg_note(st, "#1211 scenario marker");
    };

    // One queued movement key per think, consumed as the next observation
    // (the recipe, as dark-vein-loop's driver).
    let take_step = |st: &mut BorgState| {
        assert_eq!(st.io.queued_keys().len(), 1, "one movement key per think");
        let dir = (crate::borg_io::borg_inkey(st, true) - u32::from(b'0')) as usize;
        st.borg.c = loc {
            x: st.borg.c.x + i32::from(engine::ddx(dir)),
            y: st.borg.c.y + i32::from(engine::ddy(dir)),
        };
        st.borg.time.now += 1;
        assert!(floor(st.borg.c.y, st.borg.c.x), "the step stays on the carved floor");
    };
    let notes = |age: u16| String::from_utf8_lossy(&engine::message_str(age)).to_string();
    // Notes since this scenario's marker only: the message ring keeps older
    // scenarios' notes, and the negative asserts must not read them.
    let had_note = |want: &str| {
        for age in 0..60u16 {
            let note = notes(age);
            if note.contains("#1211 scenario marker") {
                return false;
            }
            if note.contains(want) {
                return true;
            }
        }
        false
    };

    // The production decision points, driven in the recorded order (the
    // note-standing precedent: the dungeon think's own borg_notice re-reads
    // the live engine character, so the fabricated traits hold for the
    // decision calls, not for the whole think). The order is the dungeon
    // think's: borg_caution (the defense cast), borg_continue_recovery
    // (borg_flow_old), borg_recover (the rest), borg_flow_recover (the next
    // walk), each consuming its keys as the next observation.
    use crate::borg_caution::borg_caution;
    use crate::borg_flow_misc::{borg_continue_recovery, borg_flow_recover};
    use crate::borg_recover::borg_recover;

    if on {
        // --- The shipped 1s: the cast is declined and the rest happens. ---
        setup(st, 2);
        let acted = borg_caution(st);
        assert!(
            !had_note("# Attempting to cast Hero"),
            "the mana-starved prep gate holds the Heroism cast"
        );
        if acted {
            assert!(
                st.io.queued_keys().is_empty(),
                "caution declines: {:?}",
                st.io.queued_keys()
            );
            while !st.io.queued_keys().is_empty() {
                crate::borg_io::borg_inkey(st, true);
            }
        }
        assert!(
            borg_recover(st),
            "the recovery decision acts on the recorded state (SP 2)"
        );
        assert!(
            had_note("# Resting to gain Mana"),
            "the full-HP mana rest is the recovery decision"
        );
        assert!(
            st.io.queued_keys().contains(&u32::from(b'R')),
            "the rest is issued: {:?}",
            st.io.queued_keys()
        );
        while !st.io.queued_keys().is_empty() {
            crate::borg_io::borg_inkey(st, true);
        }

        setup(st, 0);
        assert!(borg_recover(st), "the recovery decision acts (SP 0)");
        assert!(
            had_note("# Resting to gain Mana"),
            "the empty-mana rest is the recovery decision"
        );
        while !st.io.queued_keys().is_empty() {
            crate::borg_io::borg_inkey(st, true);
        }

        // The production handoff with a route live (review of #1215, P3):
        // the walk runs in the think's own order — borg_continue_recovery
        // first, borg_recover only where the route ends — and at the
        // endpoint the mana rest queues instead of the next route. The
        // recorded pendulum's destination is the nearest eligible grid,
        // 7 west (at (59,55) four orcs are still within max_range and the
        // corridor prices 20): the general gate is exactly what turns this
        // arrival into the rest.
        setup(st, 0);
        assert!(borg_flow_recover(st, 50), "a route commits");
        take_step(st);
        let mut walked = 0;
        while borg_continue_recovery(st) {
            assert!(
                !had_note("# Resting to gain Mana"),
                "no rest mid-route: the walk continues"
            );
            take_step(st);
            walked += 1;
        }
        assert!(walked >= 6, "the route walks to its destination: {walked}");
        assert!(
            borg_recover(st),
            "the endpoint rests instead of reversing (the general gate)"
        );
        assert!(
            had_note("# Resting to gain Mana"),
            "the mana rest is the endpoint's decision"
        );
        assert!(
            st.io.queued_keys() == vec![u32::from(b'R'), u32::from(b'*'), u32::from(KC_ENTER)],
            "the rest command is queued and no movement is: the route is not restarted, keys {:?}",
            st.io.queued_keys()
        );
        while !st.io.queued_keys().is_empty() {
            crate::borg_io::borg_inkey(st, true);
        }
    } else {
        // --- C's 0s: the recorded loop reproduces. ---
        setup(st, 2);
        assert!(
            borg_caution(st),
            "caution acts on the recorded state (SP 2)"
        );
        assert!(
            had_note("# Attempting to cast Hero"),
            "the regenerated mana is spent on Heroism (defense type 12)"
        );
        assert_eq!(
            st.io.queued_keys(),
            vec![109, 97, 102],
            "the recorded cast keys m,a,f"
        );
        while !st.io.queued_keys().is_empty() {
            crate::borg_io::borg_inkey(st, true);
        }

        // The pendulum: SP 0, no goal. Each recovery walk ends, borg_recover
        // refuses the mana rest at danger 25, and the next walk starts back
        // the other way; no rest anywhere on the corridor.
        setup(st, 0);
        let mut flows = 0;
        let mut steps = 0;
        while steps < 30 {
            if borg_continue_recovery(st) {
                take_step(st);
                steps += 1;
                continue;
            }
            assert!(
                !borg_recover(st),
                "the off arm must not rest: the recorded veto at danger 25"
            );
            assert!(
                !had_note("# Resting to gain Mana"),
                "the off arm must not rest: the recorded veto"
            );
            if borg_flow_recover(st, 50) {
                take_step(st);
                steps += 1;
                if had_note("# Flowing toward Recover Grid at cost") {
                    flows += 1;
                }
                continue;
            }
            // Nothing acted: the recorded think would twitch. Count it and
            // keep the loop bounded.
            steps += 1;
        }
        assert!(
            flows >= 2,
            "the recovery walk restarts (the recorded pacing), got {flows}"
        );
    }

    println!("recovery mana loop passed: {}", if on { "on" } else { "off" });
    // This test-only process owns a fabricated Borg observation. End before
    // its queued keys can be applied to the live engine's unrelated cave.
    std::process::exit(0);
}
