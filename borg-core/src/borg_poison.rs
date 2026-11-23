//! borg-reborn-rs (#1140): poison triage.
//!
//! Poison costs 1 HP each world tick and stops regeneration
//! (`process_world`, game-world.c:513-519; `player_regen_hp`,
//! player-util.c:468), while `decrease_timeouts` lowers the counter by
//! `adj_con_fix[stat_ind[CON]] + 1` (game-world.c:282, 317-320). A poisoned
//! Borg with no monster near still loses `ceil(counter / (adj_con_fix + 1))`
//! HP. The core held poison as a flag (`BI_ISPOISONED`) and never compared
//! that pending damage with its HP and the healing it carries. Zoo run
//! 45121432449 (track-p `1c071812`, borg-cemetery#146): the Rogue 626789725
//! drank its Cure Light Wounds at 10/19 HP (8 HP of a 20 HP potion) and died
//! of the poison in town, no monster near; the Ranger 956247604 explored DL1
//! with poison 82 and 2 CLW; the Druid 799968561 drank 11 CLW to keep
//! meleeing a white jelly while the poison rose from 4 to 190.
//!
//! Five settings, each compiled and shipped 0 (RULES 14: hypotheses until
//! borg-zoo measures them):
//!
//! * A, `poison_pending_note`: the pending damage, from the engine counter
//!   (the owner's choice for #1136) and the CON index, is noted with the HP
//!   and the healing carried; `pending >= HP + healing` is named.
//! * B, `poison_late_drink`: poisoned and safe (danger 0, not cut), Cure Light
//!   and Cure Serious Wounds are drunk only when the whole heal fits under the
//!   maximum HP, or when the HP is at most the next turn's poison plus
//!   `poison_late_drink_margin`.
//! * C, `poison_leave_unfixable`: when the poison will take more than the HP
//!   and the healing carried and no cure is carried, leave for town now:
//!   Recall (below DL1, or on DL1 with no known up staircase), else the
//!   stairs, rising level by level.
//! * D, `poison_town_triage`: in town with `pending >= HP`, the Alchemist
//!   first: a Neutralize Poison or a Cure Critical Wounds if stocked and
//!   affordable (then drunk), else enough Cure Light Wounds to cover the gap
//!   (drunk late, as B); no sale while selling pays nothing
//!   (`birth_no_selling`); then the usual plan.
//! * E, `poison_fight_pending`: poisoned, next to an awake poisoner: the
//!   poison already pending plus the poison and damage the fight adds count
//!   against the HP and the healing carried; a lost race leaves, stepping
//!   out of reach of a poisoner that never moves.

use crate::borg_headers::borg_trait_h::*;
use crate::borg_io::borg_note;
use crate::borg_headers::borg_magic_h::{CURE_POISON, HERBAL_CURING, MINOR_HEALING};
use crate::engine;
use crate::headers::list_player_timed_h::TMD_POISONED;
use crate::headers::list_tvals_h::TV_POTION;
use crate::params::{value as param, Param};
use crate::state::BorgState;

/// `adj_con_fix[]` at 574b618c (`player-calcs.c:686-725`), by the CON index
/// (`stat_ind[STAT_CON]`, `BI_CON_INDEX`): the extra amount the poison, cut
/// and stun timers fall each world tick. Pinned to the engine by a test.
const ADJ_CON_FIX: [i32; 38] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 2, 2, 3, 3, 3, 3, 3, 4, 4, 5, 6, 6, 7, 7,
    8, 8, 8, 9, 9, 9,
];

/// The HP Cure Light and Cure Serious Wounds restore: `HEAL_HP` with
/// `dice:20` and `dice:40` (lib/gamedata/object.txt:3165-3166, 3184-3185):
/// a flat amount, the percentage part 0 (`effect_handler_HEAL_HP`,
/// effect-handler-attack.c:211-217), cut at the maximum (:229-233).
pub(crate) const CLW_HEAL: i32 = 20;
pub(crate) const CSW_HEAL: i32 = 40;

/// The amount a poison counter falls each world tick.
pub(crate) fn decay(con_index: i32) -> i32 {
    ADJ_CON_FIX[con_index.clamp(0, 37) as usize] + 1
}

/// The HP a running poison counter still takes: 1 a world tick while it is
/// above 0, the counter falling by `decay` each tick. Damage reduction
/// (`player_apply_damage_reduction`) is ignored.
pub(crate) fn pending_hp(counter: i32, con_index: i32) -> i32 {
    let d = decay(con_index);
    (counter.max(0) + d - 1) / d
}

/// The most a potion of `heal` HP can give: drunk at 1 HP, `max - 1`.
pub(crate) fn late_heal(heal: i32, maxhp: i32) -> i32 {
    heal.min((maxhp - 1).max(0))
}

/// The healing carried, each potion drunk late (B): at most `max - 1` each.
pub(crate) fn healing_carried(clw: i32, csw: i32, maxhp: i32) -> i32 {
    clw.max(0) * late_heal(CLW_HEAL, maxhp) + csw.max(0) * late_heal(CSW_HEAL, maxhp)
}

/// The Cure Light Wounds that close a gap of `gap` HP (`pending - HP -
/// healing carried`, the poison outlasting them by `gap`), with one HP to
/// spare: `HP + healing` must exceed `pending` to live.
pub(crate) fn clw_to_cover(gap: i32, maxhp: i32) -> i32 {
    let each = late_heal(CLW_HEAL, maxhp);
    if gap < 0 || each <= 0 { return 0; }
    (gap + 1 + each - 1) / each
}

/// What the poison and the Borg's healing are, read once.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct Triage {
    /// `timed[TMD_POISONED]`.
    pub counter: i32,
    /// The HP the counter still takes (`pending_hp`).
    pub pending: i32,
    pub hp: i32,
    pub maxhp: i32,
    /// Cure Light and Cure Serious Wounds potions carried.
    pub clw: i32,
    pub csw: i32,
    /// `healing_carried(clw, csw, maxhp)`.
    pub healing: i32,
    /// A poison cure carried or castable: Neutralize Poison and any other
    /// item that cures poison (`BI_ACUREPOIS`), Cure Critical Wounds
    /// (`BI_ACCW`), Healing, *Healing* or Life, Cure Poison or Herbal Curing.
    pub cure: bool,
    /// A healing spell (Minor Healing): the healing is not finite.
    pub renewable: bool,
}

impl Triage {
    /// The poison takes more than the HP and the healing carried, and
    /// nothing carried cures it.
    pub(crate) fn unfixable(&self) -> bool {
        !self.cure && !self.renewable && self.pending >= self.hp + self.healing
    }

    /// D's condition: the poison takes at least the HP (a cure carried
    /// still counts: it is to be drunk).
    pub(crate) fn outlasts_hp(&self) -> bool {
        !self.renewable && self.pending >= self.hp
    }

    /// The CLW D buys: enough that `HP + healing` exceeds `pending`.
    pub(crate) fn clw_wanted(&self) -> i32 {
        clw_to_cover(self.pending - self.hp - self.healing, self.maxhp)
    }
}

/// The last note A wrote, and D's actions this town visit.
#[derive(Default, Debug)]
pub struct PoisonState {
    noted: Option<(i32, i32, i32, bool)>,
    /// The HP at B's last hold note.
    held: Option<i32>,
    /// (`time.level` of the town visit, triage actions on it).
    triage_actions: (i32, i32),
}

fn on(name: Param) -> bool {
    param(name) != 0
}

fn any_switch() -> bool {
    on(Param::poison_pending_note) || on(Param::poison_late_drink) || on(Param::poison_leave_unfixable)
        || on(Param::poison_town_triage) || on(Param::poison_fight_pending)
}

fn potions(st: &mut BorgState, sval: i32) -> i32 {
    let k = crate::borg_item_val::borg_lookup_kind(st, TV_POTION as i32, sval);
    if k <= 0 { return 0; }
    st.borg.has.get(k as usize).copied().map_or(0, i32::from)
}

/// The triage now; `None` when not poisoned (or the counter is 0).
pub(crate) fn read(st: &mut BorgState) -> Option<Triage> {
    if st.borg.tr(BI_ISPOISONED) == 0 { return None; }
    let counter = i32::from(engine::player_timed(TMD_POISONED));
    if counter <= 0 { return None; }
    let (clw_sv, csw_sv) = (st.item_val.sv_potion_cure_light, st.item_val.sv_potion_cure_serious);
    let clw = potions(st, clw_sv);
    let csw = potions(st, csw_sv);
    let maxhp = st.borg.tr(BI_MAXHP);
    let t = |i| st.borg.tr(i);
    let cure = t(BI_ACUREPOIS) + t(BI_ACCW) + t(BI_AHEAL) + t(BI_AEZHEAL) + t(BI_ALIFE) > 0
        || crate::borg_magic::borg_spell_legal(st, CURE_POISON)
        || crate::borg_magic::borg_spell_legal(st, HERBAL_CURING);
    let renewable = crate::borg_magic::borg_spell_legal(st, MINOR_HEALING);
    Some(Triage {
        counter,
        pending: pending_hp(counter, st.borg.tr(BI_CON_INDEX)),
        hp: st.borg.tr(BI_CURHP),
        maxhp,
        clw,
        csw,
        healing: healing_carried(clw, csw, maxhp),
        cure,
        renewable,
    })
}

/// A: the note, when any of the five switches is on and the state moved.
pub(crate) fn observe(st: &mut BorgState) {
    if !any_switch() { return; }
    let Some(t) = read(st) else {
        st.poison.noted = None;
        return;
    };
    let key = (t.pending - t.hp, t.healing, t.maxhp, t.cure || t.renewable);
    if st.poison.noted == Some(key) { return; }
    st.poison.noted = Some(key);
    let d = decay(st.borg.tr(BI_CON_INDEX));
    let verdict = if t.cure { "a cure carried".to_owned() } else if t.renewable {
        "a healing spell".to_owned()
    } else if t.unfixable() {
        format!("more than HP + healing by {}", t.pending - t.hp - t.healing + 1)
    } else { "covered".to_owned() };
    borg_note(st, &format!(
        "# Poison: counter {} (falls by {d} a tick), about {} HP still to come; HP {}/{}, healing carried {} ({} CLW, {} CSW): {verdict} (#1140)",
        t.counter, t.pending, t.hp, t.maxhp, t.healing, t.clw, t.csw));
}

/// The world ticks one Borg turn takes: 10 game turns a tick, the Borg
/// gaining `extract_energy[speed]` energy a game turn and spending 100 a
/// turn (at normal speed one tick a turn).
fn ticks_per_turn() -> i32 {
    let energy = i32::from(engine::extract_energy(engine::player_combat_speed().clamp(0, 199) as usize)).max(1);
    (10 + energy - 1) / energy
}

/// B's test, apart from the engine: hold a potion of `heal` HP when the
/// heal does not fit under the maximum and the HP is above the next turn's
/// poison (`per_turn`) plus `margin`.
pub(crate) fn late_drink_hold(hp: i32, maxhp: i32, heal: i32, per_turn: i32, margin: i32) -> bool {
    hp + heal > maxhp && hp > per_turn + margin
}

/// B (`poison_late_drink`; and D in town): `borg_heal` holds a Cure Light
/// (`heal` 20) or Cure Serious Wounds (40) while poisoned and safe. A cure
/// the tiers below drink (Cure Critical Wounds) is not held.
pub(crate) fn late_drink_holds(st: &mut BorgState, danger: i32, heal: i32) -> bool {
    let town = st.borg.tr(BI_CDEPTH) == 0 && on(Param::poison_town_triage);
    if !(on(Param::poison_late_drink) || town) || danger != 0 || st.borg.tr(BI_ISCUT) != 0 {
        return false;
    }
    let Some(t) = read(st) else { return false };
    if t.cure || t.renewable { return false; }
    let hold = late_drink_hold(t.hp, t.maxhp, heal, ticks_per_turn(), param(Param::poison_late_drink_margin));
    let carried = if heal == CLW_HEAL { t.clw } else { t.csw };
    if hold && carried > 0 && st.poison.held != Some(t.hp) {
        // One note a HP value, for a potion carried, not one a think.
        st.poison.held = Some(t.hp);
        borg_note(st, &format!("# Poison: holding a {heal} HP cure at HP {}/{}, drunk at HP {} or when it fits (#1140)",
            t.hp, t.maxhp, ticks_per_turn() + param(Param::poison_late_drink_margin)));
    }
    hold
}

/// B: the drink at the line. Poisoned, safe and carrying no cure, at HP at
/// most the next turn's poison plus `poison_late_drink_margin`, a Cure Light
/// Wounds (else a Cure Serious Wounds) is drunk here: `borg_heal`'s tiers
/// would refuse it at a large loss past `caution_clw_max_level`. True when
/// one was drunk.
pub(crate) fn late_drink(st: &mut BorgState, danger: i32) -> bool {
    let town = st.borg.tr(BI_CDEPTH) == 0 && on(Param::poison_town_triage);
    if !(on(Param::poison_late_drink) || town) || danger != 0 || st.borg.tr(BI_ISCUT) != 0 {
        return false;
    }
    let Some(t) = read(st) else { return false };
    if t.cure || t.renewable || t.hp > ticks_per_turn() + param(Param::poison_late_drink_margin) {
        return false;
    }
    let (clw, csw) = (st.item_val.sv_potion_cure_light, st.item_val.sv_potion_cure_serious);
    let drunk = crate::borg_item_use::borg_quaff_potion(st, clw) || crate::borg_item_use::borg_quaff_potion(st, csw);
    if drunk {
        borg_note(st, &format!("# Poison: late drink at HP {}/{}, about {} HP of poison to come (#1140)", t.hp, t.maxhp, t.pending));
    }
    drunk
}

/// C (`poison_leave_unfixable`): in the dungeon, the poison will take more
/// than the HP and the healing carried, and no cure is carried. The caller
/// then sets leaving and fleeing and calls `leave_recall`.
pub(crate) fn unfixable_here(st: &mut BorgState) -> Option<Triage> {
    if !on(Param::poison_leave_unfixable) || st.borg.tr(BI_CDEPTH) == 0 { return None; }
    read(st).filter(Triage::unfixable)
}

/// C: rising (each level climbs on), and Recall read below DL1, or on DL1
/// with no known up staircase. True when Recall was read.
pub(crate) fn leave_recall(st: &mut BorgState, t: &Triage) -> bool {
    st.borg.goal.rising = true;
    let recall = st.borg.goal.recalling == 0
        && (st.borg.tr(BI_CDEPTH) >= 2 || st.stairs.less.num == 0)
        && crate::borg_escape::borg_recall(st);
    if recall {
        borg_note(st, &format!("# Recalling: the poison takes about {} HP, more than HP {} + healing {} (#1140)",
            t.pending, t.hp, t.healing));
    }
    recall
}

/// E's race, apart from the engine: the HP the fight and the poison take,
/// from now to the poison's end, against the HP and the healing carried.
/// `turns` to kill every adjacent poisoner, `damage` their HP a Borg turn,
/// `timer` the poison counter they add a Borg turn, `pending` the HP the
/// running counter takes.
pub(crate) fn poison_race_lost(turns: f64, damage: f64, timer: f64, decay: i32, pending: i32, hp: i32, healing: i32) -> bool {
    let added = turns * timer / f64::from(decay.max(1));
    turns * damage + added + f64::from(pending) >= f64::from(hp + healing)
}

/// The poison counter a monster adds each Borg turn: each POISON blow that
/// hits adds `5 + randint1(rlev)` (mon-blows.c:683-684), on average
/// `5 + (rlev + 1) / 2`, times its actions per Borg turn (`ratio`).
fn poison_timer_per_turn(st: &BorgState, r: usize, ratio: f64) -> f64 {
    let race = engine::race(r);
    let rlev = i32::from(race.level).max(1);
    let mut sum = 0.0;
    for b in 0..usize::from(engine::z_info().mon_blows_max) {
        let Some(blow) = engine::race_blow(r, b) else { break };
        if blow.effect_name.to_bytes() != b"POISON" { continue; }
        let chance = crate::borg_fight_attack::hit_permyriad(3 * rlev + blow.hit_power, st.borg.tr(BI_ARMOR));
        sum += f64::from(chance) / 10_000.0 * (5.0 + f64::from(rlev + 1) / 2.0);
    }
    sum * ratio
}

/// E (`poison_fight_pending`): poisoned, next to awake visible poisoners.
/// `Some((lost, never_moves))` when the race could be computed: `lost` when
/// the fight and the poison take the HP and the healing carried;
/// `never_moves` when one of them never moves (the caller steps out of its
/// reach). `None` when the switch is off, nothing applies or a number is
/// unknown (the fight is then left to the other rules).
pub(crate) fn fight_race(st: &mut BorgState) -> Option<(bool, bool)> {
    use crate::headers::list_mon_race_flags_h::RF_NEVER_MOVE;
    if !on(Param::poison_fight_pending) || st.borg.tr(BI_CDEPTH) == 0 { return None; }
    let t = read(st)?;
    if t.cure || t.renewable { return None; }
    // Only the temporary resistance, read from the engine (`TMD_OPP_POIS`):
    // a resistance blocks every new poisoning (`player_inc_check`,
    // player-timed.c:960-967; player_timed.txt POISONED `fail:2:POIS`), and
    // the Elf Druid 799968561 had `BI_RPOIS` 1 while the white jelly raised
    // its counter by about 10 a turn (#1140's replay). A Borg that resists
    // is rarely poisoned at all, so the race errs toward leaving.
    let resists = st.borg.temp.res_pois;
    let energy = |s: i32| f64::from(engine::extract_energy(s.clamp(0, 199) as usize)).max(1.0);
    let me = energy(engine::player_combat_speed());
    let (mut turns, mut damage, mut timer, mut immobile, mut count) = (0.0, 0.0, 0.0, false, 0);
    for i in 1..st.kills.nxt.max(0) as usize {
        let kill = st.kills.kills[i];
        if kill.r_idx == 0 || !kill.awake || kill.r_idx as usize >= engine::r_max().saturating_sub(1) { continue; }
        if engine::distance(st.borg.c, kill.pos) != 1 { continue; }
        let forecast = crate::borg_caution::fight_incoming(st, i);
        if !forecast.poisons { continue; }
        let hp = crate::borg_fight_attack::observed_kill_hp(&kill)?;
        let attack = crate::borg_fight_attack::focused_attack(st, i)?;
        if attack.hp_per_turn_milli <= 0 { return None; }
        let r = usize::from(kill.r_idx);
        let ratio = energy(crate::borg_caution::forecast_monster_speed(&kill)) / me;
        turns += f64::from(hp) * 1000.0 / f64::from(attack.hp_per_turn_milli);
        damage += forecast.hp_per_turn;
        if !resists { timer += poison_timer_per_turn(st, r, ratio); }
        immobile |= engine::race_flag_has(r, RF_NEVER_MOVE);
        count += 1;
    }
    if count == 0 { return None; }
    let d = decay(st.borg.tr(BI_CON_INDEX));
    let lost = poison_race_lost(turns, damage, timer, d, t.pending, t.hp, t.healing);
    borg_note(st, &format!(
        "# Poison fight race: {count} poisoner(s), {turns:.1} turns to kill, {damage:.2} damage and {timer:.2} poison a turn, \
         about {} HP of poison pending; HP {} + healing {}: {} (#1140)",
        t.pending, t.hp, t.healing, if lost { "lost" } else { "not lost" }));
    Some((lost, immobile))
}

/// D's condition: in town, the switch on, `pending >= HP`, no healing spell.
/// A cure carried keeps it holding until it is drunk (`town_cure`), so no
/// excess trip walks the Borg into a store it must leave to drink.
pub(crate) fn town_triage_holds(st: &mut BorgState) -> bool {
    on(Param::poison_town_triage) && st.borg.tr(BI_CDEPTH) == 0
        && read(st).is_some_and(|t| t.outlasts_hp())
}

/// D, the Alchemist (`store.txt`: store 5, index 4).
const ALCHEMIST: usize = 4;

/// The most triage actions one town visit takes (a refused purchase is
/// asked again otherwise).
const TRIAGE_ACTIONS: i32 = 4;

/// D: the purchase the triage asks for, from the Borg's record of the
/// Alchemist's stock: a Neutralize Poison, else a Cure Critical Wounds,
/// affordable; else the Cure Light Wounds that cover the gap (as many as
/// the gold buys; the executor caps the copies by the stack, the room and
/// the gold). `None` when nothing is wanted or nothing listed is affordable.
pub(crate) fn town_triage_action(st: &mut BorgState) -> Option<crate::borg_town_shadow::TownAction> {
    use crate::borg_town_record::action;
    if !town_triage_holds(st) { return None; }
    if read(st).is_some_and(|t| t.cure) { return None; }
    let level = st.borg.time.level;
    if st.poison.triage_actions.0 != level { st.poison.triage_actions = (level, 0); }
    if st.poison.triage_actions.1 >= TRIAGE_ACTIONS { return None; }
    let t = read(st)?;
    let gold = st.borg.tr(BI_GOLD);
    let v = &st.item_val;
    let (np, ccw, clw) = (v.sv_potion_cure_poison, v.sv_potion_cure_critical, v.sv_potion_cure_light);
    let ware = st.store.shops.get(ALCHEMIST)?.ware.clone();
    let find = |sval: i32| ware.iter().enumerate().find(|(slot, w)| {
        w.iqty > 0 && u32::from(w.tval) == TV_POTION && i32::from(w.sval) == sval && w.cost > 0 && w.cost <= gold
            && !crate::borg_town_live::buy_rejected(st, ALCHEMIST as i16, *slot)
    }).map(|(slot, w)| (slot, w.cost));
    let (pick, copies, what) = if let Some(p) = find(np) {
        (p, 1, "Neutralize Poison")
    } else if let Some(p) = find(ccw) {
        (p, 1, "Cure Critical Wounds")
    } else {
        let n = t.clw_wanted();
        if n <= 0 { return None; }
        (find(clw)?, n, "Cure Light Wounds")
    };
    let (slot, price) = pick;
    // Counted (and noted) in the store, where the purchase is tried; the
    // walk there asks the same action each think.
    if st.borg.in_shop {
        st.poison.triage_actions.1 += 1;
        borg_note(st, &format!(
            "# Poison triage: about {} HP of poison against HP {} + healing {}: buying {copies} {what} at {price} gold ({gold} gold) (#1140)",
            t.pending, t.hp, t.healing));
    }
    Some(crate::borg_town_shadow::TownAction {
        code: action::BUY,
        at: [ALCHEMIST as i64, slot as i64, i64::from(copies), i64::from(price)],
    })
}

/// D: no sale while the triage holds and selling pays nothing
/// (`birth_no_selling`, list-options.h:90): a sale costs a turn and pays 0.
pub(crate) fn no_sale_now(st: &mut BorgState) -> bool {
    use crate::headers::list_options_h::OPT_birth_no_selling;
    engine::player_opt(OPT_birth_no_selling) && town_triage_holds(st)
}

/// D: what to drink in town while the triage holds: a Neutralize Poison,
/// else a Cure Critical Wounds (it cures poison; the raw quaff, as
/// `unseen_heal`: `borg_quaff_crit` keeps the last two). CLW are left to
/// `borg_heal` (late, B). True when a potion was drunk.
pub(crate) fn town_cure(st: &mut BorgState) -> bool {
    if !on(Param::poison_town_triage) || st.borg.tr(BI_CDEPTH) != 0 { return false; }
    let Some(t) = read(st) else { return false };
    if t.pending < t.hp || st.borg.in_shop { return false; }
    let (np, ccw) = (st.item_val.sv_potion_cure_poison, st.item_val.sv_potion_cure_critical);
    let drunk = crate::borg_item_use::borg_quaff_potion(st, np) || crate::borg_item_use::borg_quaff_potion(st, ccw);
    if drunk {
        borg_note(st, &format!("# Poison triage: cured in town, about {} HP of poison against HP {} (#1140)", t.pending, t.hp));
    }
    drunk
}

/// D: in a store, leave it when a drink is due (a cure carried and `pending
/// >= HP`, or the HP at the late-drink line with a CLW or CSW carried): the
/// drink happens outside.
pub(crate) fn store_leave_to_drink(st: &mut BorgState) -> bool {
    if !on(Param::poison_town_triage) || st.borg.tr(BI_CDEPTH) != 0 { return false; }
    let Some(t) = read(st) else { return false };
    let np = potions(st, st.item_val.sv_potion_cure_poison);
    let ccw = potions(st, st.item_val.sv_potion_cure_critical);
    let cure_due = t.pending >= t.hp && np + ccw > 0;
    let heal_due = t.clw + t.csw > 0 && t.hp <= ticks_per_turn() + param(Param::poison_late_drink_margin);
    if cure_due || heal_due {
        borg_note(st, &format!("# Poison triage: leaving the store to drink at HP {}/{} (#1140)", t.hp, t.maxhp));
    }
    cure_due || heal_due
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adj_con_fix_is_the_engines_table() {
        let src = include_str!("../../engine/angband/src/player-calcs.c");
        let start = src.find("const int adj_con_fix[STAT_RANGE] =").expect("adj_con_fix in player-calcs.c");
        let body = &src[start..];
        let body = &body[body.find('{').unwrap() + 1..body.find("};").unwrap()];
        let values: Vec<i32> = body.lines()
            .filter_map(|l| l.split_whitespace().next())
            .filter_map(|v| v.trim_end_matches(',').parse().ok())
            .collect();
        assert_eq!(values, ADJ_CON_FIX.to_vec());
    }

    #[test]
    fn potion_heals_are_the_engines() {
        let src = include_str!("../../engine/angband/lib/gamedata/object.txt");
        for (name, heal) in [("Cure Light Wounds", CLW_HEAL), ("Cure Serious Wounds", CSW_HEAL)] {
            let at = src.find(&format!("name:{name}\ntype:potion\n")).expect(name);
            let block = &src[at..at + src[at..].find("\n\n").unwrap()];
            let after = &block[block.find("effect:HEAL_HP\n").expect("HEAL_HP") + 15..];
            assert!(after.starts_with(&format!("dice:{heal}\n")), "{name}: {after}");
        }
    }

    #[test]
    fn pending_damage_from_counter_and_con() {
        // 626789725: poison 28, falling by 1 a tick: 28 HP to come.
        assert_eq!(pending_hp(28, 0), 28);
        // 799968561: poison 162, falling by 2 a tick: 81 HP.
        assert_eq!(pending_hp(162, 15), 54, "CON index 15 falls by 3");
        assert_eq!(pending_hp(162, 11), 81, "CON index 11 falls by 2");
        assert_eq!(pending_hp(1, 37), 1, "a counter above 0 still takes the tick");
        assert_eq!(pending_hp(0, 10), 0);
        assert_eq!(pending_hp(-3, 10), 0);
        assert_eq!(pending_hp(14, 99), 2, "an index past the table is the last");
    }

    #[test]
    fn healing_carried_is_capped_at_max_hp() {
        assert_eq!(late_heal(CLW_HEAL, 19), 18, "the Rogue: drunk at 1 HP, 18 of 20");
        assert_eq!(late_heal(CLW_HEAL, 47), 20);
        assert_eq!(healing_carried(2, 1, 19), 54, "a CSW at max HP 19 is worth 18 too");
        assert_eq!(healing_carried(2, 0, 47), 40);
        assert_eq!(healing_carried(0, 0, 1), 0);
    }

    #[test]
    fn late_drink_at_the_max_hp_cap() {
        // 626789725 (max HP 19): at 10/19 a CLW gives 9 of 20: hold.
        assert!(late_drink_hold(10, 19, CLW_HEAL, 1, 2));
        // At 3 (one tick plus a margin of 2) it drinks.
        assert!(!late_drink_hold(3, 19, CLW_HEAL, 1, 2));
        assert!(late_drink_hold(4, 19, CLW_HEAL, 1, 2));
        // A slow Borg loses two a turn: drinks at 4.
        assert!(!late_drink_hold(4, 19, CLW_HEAL, 2, 2));
        // The whole heal fits: drink as before.
        assert!(!late_drink_hold(27, 47, CLW_HEAL, 1, 2));
        assert!(late_drink_hold(28, 47, CLW_HEAL, 1, 2));
        assert!(!late_drink_hold(7, 47, CSW_HEAL, 1, 2), "CSW fits at 7/47");
    }

    fn triage(pending: i32, hp: i32, maxhp: i32, clw: i32) -> Triage {
        Triage { counter: pending, pending, hp, maxhp, clw, csw: 0, healing: healing_carried(clw, 0, maxhp), cure: false, renewable: false }
    }

    #[test]
    fn unfixable_and_the_clw_that_cover_it() {
        // 956247604: poison 82 (falling by 1), HP 37/47, 2 CLW: 82 >= 37 + 40.
        let ranger = triage(82, 37, 47, 2);
        assert!(ranger.unfixable());
        assert_eq!(ranger.clw_wanted(), 1, "82 - 77 = 5: one more CLW");
        // 626789725 at the last drink: poison 28, HP 10/19, 1 CLW: 28 >= 10 + 18.
        let rogue = triage(28, 10, 19, 1);
        assert!(rogue.unfixable());
        assert_eq!(rogue.clw_wanted(), 1);
        let covered = triage(27, 10, 19, 1);
        assert!(!covered.unfixable(), "27 < 28");
        assert_eq!(covered.clw_wanted(), 0);
        assert!(!Triage { cure: true, ..ranger }.unfixable());
        assert!(!Triage { renewable: true, ..ranger }.unfixable());
        assert!(triage(20, 20, 30, 0).outlasts_hp());
        assert!(!triage(19, 20, 30, 0).outlasts_hp());
    }

    #[test]
    fn town_triage_with_9_53_and_171_gold() {
        // 626789725: HP 14/19, poison 21: 21 >= 14, no CLW: 1 CLW covers it
        // (14 + 18 > 21). With 9 gold the Alchemist's CLW (cost 20,
        // object.txt) is not affordable: `town_triage_action` lists only
        // wares at most the gold, so it asks nothing.
        let rogue = triage(21, 14, 19, 0);
        assert!(rogue.outlasts_hp());
        assert_eq!(rogue.clw_wanted(), 1);
        // 799968561: HP 29/32, poison 162 falling by 2 (81 HP), 53 gold:
        // 81 - 29 = 52 HP to cover, 20 a CLW (max HP 32): 3; 53 gold buys 2.
        // The executor (`purchase_quantity`) caps the 3 at what the gold buys.
        let druid = triage(81, 29, 32, 0);
        assert_eq!(druid.clw_wanted(), 3);
        // 956247604: 171 gold would buy 8; it needs 1 more.
        assert_eq!(triage(82, 37, 47, 2).clw_wanted(), 1);
        assert_eq!(clw_to_cover(0, 47), 1, "pending = HP + healing: one HP short");
        assert_eq!(clw_to_cover(-1, 47), 0);
        assert_eq!(clw_to_cover(5, 1), 0, "max HP 1: a potion gives nothing");
    }

    #[test]
    fn poison_race_counts_the_pending_poison() {
        // 799968561's white jelly, illustrative numbers: 36 HP to kill at
        // 1.5 HP a turn (24 turns), 1 HP of damage and 4.5 poison a turn,
        // falling by 2; pending 2, HP 29/32, 11 CLW (220): 24 + 54 + 2 = 80.
        assert!(!poison_race_lost(24.0, 1.0, 4.5, 2, 2, 29, 220));
        // With no CLW left it is lost.
        assert!(poison_race_lost(24.0, 1.0, 4.5, 2, 2, 29, 0));
        // The pending poison alone can lose it.
        assert!(poison_race_lost(1.0, 0.0, 0.0, 1, 40, 29, 10));
        assert!(!poison_race_lost(1.0, 0.0, 0.0, 1, 38, 29, 10));
    }
}
