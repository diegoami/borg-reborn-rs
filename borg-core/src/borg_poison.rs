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
//! One switch, `deal_with_poison` (owner, 2026-11-23: "just one switch"),
//! compiled and shipped 0, turns on all six parts (RULES 14: a hypothesis
//! until borg-zoo measures it); `poison_late_drink_margin` and
//! `poison_cure_pending_percent` tune B and F:
//!
//! * A, the note: the pending damage, from the engine counter
//!   (the owner's choice for #1136) and the CON index, is noted with the HP
//!   and the healing carried; `pending >= HP + healing` is named.
//! * B, the late drink: poisoned and safe (danger 0, not cut), Cure Light
//!   and Cure Serious Wounds are drunk only when the whole heal fits under the
//!   maximum HP, or when the HP is at most the next turn's poison plus
//!   `poison_late_drink_margin`.
//! * C, leave while it can be fixed: when the poison will take more than the HP
//!   and the healing carried and no cure is carried, leave for town now:
//!   Recall (below DL1, or on DL1 with no known up staircase), else the
//!   stairs, rising level by level.
//! * D, the town triage: in town with `pending >= HP`, the Alchemist
//!   first: a Neutralize Poison or a Cure Critical Wounds if stocked and
//!   affordable (then drunk), else enough Cure Light Wounds to cover the gap
//!   (drunk late, as B); no sale while selling pays nothing
//!   (`birth_no_selling`); then the usual plan.
//! * F, the cure carried: a poison cure the Borg has is used once the
//!   poison still to come is `poison_cure_pending_percent` of the HP, at any
//!   danger and with no roll (the spells, a rod, an activation, food,
//!   Neutralize Poison, a staff, Cure Critical Wounds; Healing, *Healing* or
//!   Life only when the CLW/CSW carried cannot outlast it). Today's cures
//!   wait out the danger or a 25 % roll: the Paladin 1200156945 (#1134)
//!   died holding 2 Neutralize Poison. Never next to an awake poisoner.
//! * E, the poison fight race: poisoned, next to an awake poisoner: the
//!   poison already pending plus the poison and damage the fight adds count
//!   against the HP and the healing carried; a lost race leaves, stepping
//!   out of reach of a poisoner that never moves.

use crate::borg_headers::borg_trait_h::*;
use crate::borg_io::borg_note;
use crate::borg_headers::borg_magic_h::{CURE_POISON, HERBAL_CURING};
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

/// What a potion of `heal` HP gives drunk late: B drinks it at HP at most
/// `line` (the next turn's poison plus `poison_late_drink_margin`), where it
/// can restore at most `max - line`. Review of #1146 (finding 1): crediting
/// `max - 1` counted 18 HP for a CLW at max HP 19, where the drink at 3 HP
/// gives 16, and called a lost poison survivable.
pub(crate) fn late_heal(heal: i32, maxhp: i32, line: i32) -> i32 {
    heal.min((maxhp - line.max(1)).max(0))
}

/// The healing carried, each potion drunk late (B) at the line.
pub(crate) fn healing_carried(clw: i32, csw: i32, maxhp: i32, line: i32) -> i32 {
    clw.max(0) * late_heal(CLW_HEAL, maxhp, line) + csw.max(0) * late_heal(CSW_HEAL, maxhp, line)
}

/// The Cure Light Wounds that close a gap of `gap` HP (`pending - HP -
/// healing carried`, the poison outlasting them by `gap`), with one HP to
/// spare: `HP + healing` must exceed `pending` to live.
pub(crate) fn clw_to_cover(gap: i32, maxhp: i32, line: i32) -> i32 {
    let each = late_heal(CLW_HEAL, maxhp, line);
    if gap < 0 || each <= 0 { return 0; }
    (gap + 1 + each - 1) / each
}

/// Word of Recall's delay: `15 + randint0(20)` (effect-handler-general.c:1145),
/// counted down once a world tick (game-world.c:709-711), while the poison
/// takes 1 HP a tick.
const RECALL_MIN_TICKS: i32 = 15;
const RECALL_MAX_TICKS: i32 = 34;

/// C's Recall test, apart from the engine (review of #1146, finding 4): read
/// Recall only when the HP and the healing carried outlast its longest delay,
/// or, with no up staircase known, its shortest (some chance beats none).
/// Otherwise the stairs.
pub(crate) fn recall_survivable(budget: i32, stairs_known: bool) -> bool {
    budget > RECALL_MAX_TICKS || (!stairs_known && budget > RECALL_MIN_TICKS)
}

/// The delay C's exposure test charges: the longest when the budget outlasts
/// it, else the shortest (the no-stairs chance `recall_survivable` allows).
pub(crate) fn recall_delay_charged(budget: i32) -> i32 {
    if budget > RECALL_MAX_TICKS { RECALL_MAX_TICKS } else { RECALL_MIN_TICKS }
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
    /// B's drink line: the next turn's poison plus the margin.
    pub line: i32,
    /// `healing_carried(clw, csw, maxhp, line)`.
    pub healing: i32,
    /// A poison cure usable now (`usable_cure`): Cure Poison or Herbal Curing
    /// castable (the mana and the fail rate, not only known), a charged rod
    /// or staff of Curing, a charged activation that cures poison or is Cure
    /// Critical Wounds, a food or mushroom with `CURE:POISONED`, Neutralize
    /// Poison or Cure Critical Wounds. Review of #1146 (findings 2 and 5):
    /// a known spell without mana counted, a rod or staff of Curing did not.
    pub cure_carried: bool,
    /// Healing, *Healing* or Life carried: F drinks one only when the poison
    /// outlasts the HP and the CLW/CSW; they do not make the others stand
    /// aside.
    pub precious: bool,
    /// `cure_carried`, trusted to fix the poison: only with `deal_with_poison` on,
    /// whose part F drinks it. Off, nothing drinks it
    /// reliably (`borg_heal`'s cures wait while the danger is above a fifth
    /// of the avoidance; `borg_recover`'s costly cure needs a 25 % roll and
    /// keeps the last two CCW), so the parts act as if none were carried
    /// (they are off then too; kept for the tests and the note).
    pub cure: bool,
}

impl Triage {
    /// The poison takes more than the HP and the healing carried, and
    /// nothing carried cures it.
    pub(crate) fn unfixable(&self) -> bool {
        !self.cure && self.pending >= self.hp + self.healing
    }

    /// D's condition: the poison takes at least the HP (a cure carried
    /// still counts: it is to be drunk).
    pub(crate) fn outlasts_hp(&self) -> bool {
        self.pending >= self.hp
    }

    /// The CLW D buys: enough that `HP + healing` exceeds `pending`.
    pub(crate) fn clw_wanted(&self) -> i32 {
        clw_to_cover(self.pending - self.hp - self.healing, self.maxhp, self.line)
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

/// The one switch (owner, 2026-11-23: "just one switch"): every part,
/// A to F, acts only with `deal_with_poison` on.
fn enabled() -> bool {
    param(Param::deal_with_poison) != 0
}

fn potions(st: &mut BorgState, sval: i32) -> i32 {
    let k = crate::borg_item_val::borg_lookup_kind(st, TV_POTION as i32, sval);
    if k <= 0 { return 0; }
    st.borg.has.get(k as usize).copied().map_or(0, i32::from)
}

/// A carried food or mushroom that cures poison (`CURE:POISONED`, the items
/// `BI_ACUREPOIS` counts), as (tval, sval).
fn food_cure(st: &BorgState) -> Option<(i32, i32)> {
    use crate::headers::list_effects_h::EF_CURE;
    use crate::headers::list_tvals_h::{TV_FOOD, TV_MUSHROOM};
    st.items.items.iter().take(crate::borg_item::inven_wield() as usize)
        .find(|it| it.iqty > 0 && it.aware && matches!(u32::from(it.tval), TV_FOOD | TV_MUSHROOM)
            && crate::borg_item_analyze::borg_obj_has_effect(it.kind, EF_CURE as i32, TMD_POISONED as i32))
        .map(|it| (i32::from(it.tval), i32::from(it.sval)))
}

/// A poison cure usable now (`Triage::cure_carried`): what `cure_carried`
/// tries first, each checked as its use would be (spell mana and fail rate,
/// device charge and fail rate, activation charge).
fn usable_cure(st: &mut BorgState) -> bool {
    use crate::borg_item_use::{borg_equips_item, device_would_fire};
    use crate::headers::list_tvals_h::{TV_ROD, TV_STAFF};
    let fail = param(Param::caution_heal_spell_fail);
    let v = &st.item_val;
    let (rod, staff, np, ccw) = (v.sv_rod_curing, v.sv_staff_curing, v.sv_potion_cure_poison, v.sv_potion_cure_critical);
    let (act_pois, act_ccw) = (st.act.rem_fear_pois, st.act.cure_critical);
    crate::borg_magic::borg_spell_okay_fail(st, CURE_POISON, fail)
        || crate::borg_magic::borg_spell_okay_fail(st, HERBAL_CURING, fail)
        // The use limits, as `borg_zap_rod` and `borg_use_staff_fail` read
        // them (re-review of #1146).
        || device_would_fire(st, TV_ROD, rod)
        || device_would_fire(st, TV_STAFF, staff)
        || borg_equips_item(st, act_pois, true)
        || borg_equips_item(st, act_ccw, true)
        // `borg_eat` refuses while shapechanged (a Druid in fox form).
        || (!engine::player_is_shapechanged() && food_cure(st).is_some())
        || potions(st, np) > 0
        || potions(st, ccw) > 0
}

/// B's drink line: the next turn's poison plus `poison_late_drink_margin`.
fn drink_line() -> i32 {
    ticks_per_turn() + param(Param::poison_late_drink_margin)
}

/// The triage now; `None` when not poisoned (or the counter is 0).
pub(crate) fn read(st: &mut BorgState) -> Option<Triage> {
    if st.borg.tr(BI_ISPOISONED) == 0 { return None; }
    let counter = i32::from(engine::player_timed(TMD_POISONED));
    if counter <= 0 { return None; }
    let v = &st.item_val;
    let (clw_sv, csw_sv) = (v.sv_potion_cure_light, v.sv_potion_cure_serious);
    let (heal_sv, star_sv, life_sv) = (v.sv_potion_healing, v.sv_potion_star_healing, v.sv_potion_life);
    let clw = potions(st, clw_sv);
    let csw = potions(st, csw_sv);
    let precious = potions(st, heal_sv) + potions(st, star_sv) + potions(st, life_sv) > 0;
    let maxhp = st.borg.tr(BI_MAXHP);
    let line = drink_line();
    let cure_carried = usable_cure(st);
    Some(Triage {
        counter,
        pending: pending_hp(counter, st.borg.tr(BI_CON_INDEX)),
        hp: st.borg.tr(BI_CURHP),
        maxhp,
        clw,
        csw,
        line,
        healing: healing_carried(clw, csw, maxhp, line),
        cure_carried,
        precious,
        // Re-review of #1146: F declines next to an awake poisoner, so a cure
        // it cannot use there must not stand E and C aside (a Borg with
        // Neutralize Poison beside a white jelly still races and leaves).
        cure: cure_carried && enabled() && !poisoner_adjacent(st),
    })
}

/// A: the note, when any of the five switches is on and the state moved.
pub(crate) fn observe(st: &mut BorgState) {
    if !enabled() { return; }
    let Some(t) = read(st) else {
        st.poison.noted = None;
        return;
    };
    let key = (t.pending - t.hp, t.healing, t.maxhp, t.cure_carried);
    if st.poison.noted == Some(key) { return; }
    st.poison.noted = Some(key);
    let d = decay(st.borg.tr(BI_CON_INDEX));
    let verdict = if t.cure_carried { "a cure carried".to_owned() } else if t.unfixable() {
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

/// B (and D in town): `borg_heal` holds a Cure Light
/// (`heal` 20) or Cure Serious Wounds (40) while poisoned and safe. A cure
/// the tiers below drink (Cure Critical Wounds) is not held.
pub(crate) fn late_drink_holds(st: &mut BorgState, danger: i32, heal: i32) -> bool {
    if !enabled() || danger != 0 || st.borg.tr(BI_ISCUT) != 0 {
        return false;
    }
    let Some(t) = read(st) else { return false };
    if t.cure { return false; }
    let hold = late_drink_hold(t.hp, t.maxhp, heal, ticks_per_turn(), param(Param::poison_late_drink_margin));
    let carried = if heal == CLW_HEAL { t.clw } else { t.csw };
    if hold && carried > 0 && st.poison.held != Some(t.hp) {
        // One note a HP value, for a potion carried, not one a think.
        st.poison.held = Some(t.hp);
        borg_note(st, &format!("# Poison: holding a {heal} HP cure at HP {}/{}, drunk at HP {} or when it fits (#1140)",
            t.hp, t.maxhp, t.line));
    }
    hold
}

/// B: the drink at the line. Poisoned, safe and carrying no cure, at HP at
/// most the next turn's poison plus `poison_late_drink_margin`, a Cure Light
/// Wounds (else a Cure Serious Wounds) is drunk here: `borg_heal`'s tiers
/// would refuse it at a large loss past `caution_clw_max_level`. True when
/// one was drunk.
pub(crate) fn late_drink(st: &mut BorgState, danger: i32) -> bool {
    if !enabled() || danger != 0 || st.borg.tr(BI_ISCUT) != 0 {
        return false;
    }
    let Some(t) = read(st) else { return false };
    if t.cure || t.hp > t.line {
        return false;
    }
    let (clw, csw) = (st.item_val.sv_potion_cure_light, st.item_val.sv_potion_cure_serious);
    let drunk = crate::borg_item_use::borg_quaff_potion(st, clw) || crate::borg_item_use::borg_quaff_potion(st, csw);
    if drunk {
        borg_note(st, &format!("# Poison: late drink at HP {}/{}, about {} HP of poison to come (#1140)", t.hp, t.maxhp, t.pending));
    }
    drunk
}

/// F's trigger, apart from the engine: the poison still to come is at least
/// `percent` of the current HP.
pub(crate) fn cure_due(pending: i32, hp: i32, percent: i32) -> bool {
    pending > 0 && i64::from(pending) * 100 >= i64::from(hp.max(0)) * i64::from(percent)
}

/// F: an awake monster next to the Borg with a `POISON` blow (it would
/// poison again at once).
fn poisoner_adjacent(st: &BorgState) -> bool {
    (1..st.kills.nxt.max(0) as usize).any(|i| {
        let kill = &st.kills.kills[i];
        kill.r_idx != 0 && kill.awake && engine::distance(st.borg.c, kill.pos) == 1
            && crate::borg_caution::fight_incoming(st, i).poisons
    })
}

/// F: poisoned with the poison still to come at
/// least `poison_cure_pending_percent` of the HP, use a poison cure the
/// Borg has, whatever the danger and with no roll; never next to an awake
/// poisoner. Cheapest first:
/// Cure Poison and Herbal Curing (the spells), a rod of Curing, an
/// activation that cures poison, a food or mushroom that does
/// (`CURE:POISONED`, the items `BI_ACUREPOIS` counts), Neutralize Poison, a
/// staff of Curing, Cure Critical Wounds (the raw quaff: `borg_quaff_crit`
/// keeps the last two) or its activation. Healing, *Healing* and Life only
/// when the poison outlasts the HP and the CLW/CSW carried. True when one
/// was used.
pub(crate) fn cure_carried(st: &mut BorgState) -> bool {
    use crate::borg_item_use::{borg_activate_item, borg_eat, borg_quaff_potion, borg_use_staff_fail, borg_zap_rod};
    if !enabled() { return false; }
    let Some(t) = read(st) else { return false };
    let cheap = t.cure_carried && cure_due(t.pending, t.hp, param(Param::poison_cure_pending_percent));
    let dear = t.precious && t.pending >= t.hp + t.healing;
    if !cheap && !dear { return false; }
    // Never next to an awake poisoner: it poisons again at once (the Paladin
    // 1200156945, #1140's replay: Neutralize Poison at HP 10/49 beside a
    // blubbering icky thing, poisoned again the next blow, dead 20 turns
    // sooner). E and the escapes act there; the cure follows.
    if poisoner_adjacent(st) { return false; }
    let fail = param(Param::caution_heal_spell_fail);
    let v = &st.item_val;
    let (rod, staff, np, ccw) = (v.sv_rod_curing, v.sv_staff_curing, v.sv_potion_cure_poison, v.sv_potion_cure_critical);
    let (heal, star, life) = (v.sv_potion_healing, v.sv_potion_star_healing, v.sv_potion_life);
    let (act_pois, act_ccw) = (st.act.rem_fear_pois, st.act.cure_critical);
    let food = food_cure(st);
    let mut used = cheap && (crate::borg_magic::borg_spell_fail(st, CURE_POISON, fail)
        || crate::borg_magic::borg_spell_fail(st, HERBAL_CURING, fail)
        || borg_zap_rod(st, rod)
        || borg_activate_item(st, act_pois)
        || food.is_some_and(|(tval, sval)| borg_eat(st, tval, sval))
        || borg_quaff_potion(st, np)
        || borg_use_staff_fail(st, staff)
        || borg_quaff_potion(st, ccw)
        || borg_activate_item(st, act_ccw));
    let precious = !used && dear;
    if precious {
        used = borg_quaff_potion(st, heal) || borg_quaff_potion(st, star) || borg_quaff_potion(st, life);
    }
    if used {
        borg_note(st, &format!("# Poison: cured with what it carries, about {} HP of poison against HP {}{} (#1140)",
            t.pending, t.hp, if precious { ", a dear potion: the CLW carried do not outlast it" } else { "" }));
    }
    used
}

/// Review of #1146 (finding 3): `fight_before_flight` (attack on sight, the
/// lone slow monster, the winning fight) runs before `borg_caution`, so an
/// early attack could bypass E's race and F's cure. While poisoned, the early
/// fight waits when an awake poisoner is next to the Borg (E decides), when
/// F would use a cure now, or when the poison is beyond the HP and the
/// healing (C leaves): `borg_caution` decides first, and `borg_attack` still
/// follows it when nothing there acts.
pub(crate) fn defers_early_fight(st: &mut BorgState) -> bool {
    if !enabled() || st.borg.tr(BI_CDEPTH) == 0 { return false; }
    let Some(t) = read(st) else { return false };
    let cheap = t.cure_carried && cure_due(t.pending, t.hp, param(Param::poison_cure_pending_percent));
    let dear = t.precious && t.pending >= t.hp + t.healing;
    poisoner_adjacent(st) || cheap || dear || t.unfixable()
}

/// C: in the dungeon, the poison will take more
/// than the HP and the healing carried, and no cure is carried. The caller
/// then sets leaving and fleeing and calls `leave_recall`.
pub(crate) fn unfixable_here(st: &mut BorgState) -> Option<Triage> {
    if !enabled() || st.borg.tr(BI_CDEPTH) == 0 { return None; }
    read(st).filter(Triage::unfixable)
}

/// C: rising (each level climbs on), and Recall read below DL1, or on DL1
/// with no known up staircase, when its delay can be outlived
/// (`recall_survivable`) and the flight Recall's exposure test allows it;
/// otherwise the stairs. True when Recall was read.
pub(crate) fn leave_recall(st: &mut BorgState, t: &Triage) -> bool {
    st.borg.goal.rising = true;
    let stairs_known = st.stairs.less.num != 0;
    // Reviews of #1146: the flight Recall's monster exposure over the same
    // delay and against the same budget (`poison_recall_survivable`), so a
    // nearby monster that makes waiting lethal still refuses it, and the
    // running poison is not charged the whole delay against the HP alone.
    let budget = t.hp + t.healing;
    let recall = st.borg.goal.recalling == 0
        && (st.borg.tr(BI_CDEPTH) >= 2 || !stairs_known)
        && recall_survivable(budget, stairs_known)
        && crate::borg_caution::poison_recall_survivable(st, budget, recall_delay_charged(budget), t.pending)
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

/// E: poisoned, next to awake visible poisoners.
/// `Some((lost, never_moves))` when the race could be computed: `lost` when
/// the fight and the poison take the HP and the healing carried;
/// `never_moves` when one of them never moves (the caller steps out of its
/// reach). `None` when the switch is off, nothing applies or a number is
/// unknown (the fight is then left to the other rules).
pub(crate) fn fight_race(st: &mut BorgState) -> Option<(bool, bool)> {
    use crate::headers::list_mon_race_flags_h::RF_NEVER_MOVE;
    if !enabled() || st.borg.tr(BI_CDEPTH) == 0 { return None; }
    let t = read(st)?;
    if t.cure { return None; }
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
    enabled() && st.borg.tr(BI_CDEPTH) == 0
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
    if read(st).is_some_and(|t| t.cure_carried) { return None; }
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
    if !enabled() || st.borg.tr(BI_CDEPTH) != 0 { return false; }
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
    if !enabled() || st.borg.tr(BI_CDEPTH) != 0 { return false; }
    let Some(t) = read(st) else { return false };
    let np = potions(st, st.item_val.sv_potion_cure_poison);
    let ccw = potions(st, st.item_val.sv_potion_cure_critical);
    let cure_due = t.pending >= t.hp && np + ccw > 0
        || (enabled() && t.cure_carried
            && cure_due(t.pending, t.hp, param(Param::poison_cure_pending_percent)));
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
    fn healing_carried_is_what_a_drink_at_the_line_restores() {
        // Review of #1146 (finding 1): the Rogue (max HP 19) drinks at the
        // line, 3 HP (one tick plus a margin of 2): 16 of 20, not 18.
        assert_eq!(late_heal(CLW_HEAL, 19, 3), 16);
        assert_eq!(late_heal(CLW_HEAL, 19, 1), 18, "a line of 1: max - 1");
        assert_eq!(late_heal(CLW_HEAL, 47, 3), 20);
        assert_eq!(late_heal(CLW_HEAL, 19, 0), 18, "a line below 1 counts as 1");
        assert_eq!(healing_carried(2, 1, 19, 3), 48, "a CSW at max HP 19 is worth 16 too");
        assert_eq!(healing_carried(2, 0, 47, 3), 40);
        assert_eq!(healing_carried(0, 0, 1, 3), 0);
        assert_eq!(healing_carried(3, 0, 2, 3), 0, "max HP below the line: nothing");
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
        Triage { counter: pending, pending, hp, maxhp, clw, csw: 0, line: 3, healing: healing_carried(clw, 0, maxhp, 3), cure_carried: false, precious: false, cure: false }
    }

    #[test]
    fn unfixable_and_the_clw_that_cover_it() {
        // 956247604: poison 82 (falling by 1), HP 37/47, 2 CLW: 82 >= 37 + 40.
        let ranger = triage(82, 37, 47, 2);
        assert!(ranger.unfixable());
        assert_eq!(ranger.clw_wanted(), 1, "82 - 77 = 5: one more CLW");
        // 626789725 at the last drink: poison 28, HP 10/19, 1 CLW: 28 >= 10 + 16.
        let rogue = triage(28, 10, 19, 1);
        assert!(rogue.unfixable());
        assert_eq!(rogue.clw_wanted(), 1);
        // Review of #1146 (finding 1): 27 to come at 10/19 with one CLW is
        // lost (the drink at 3 HP gives 16: 10 + 16 = 26 < 27).
        let reviewed = triage(27, 10, 19, 1);
        assert!(reviewed.unfixable());
        assert_eq!(reviewed.clw_wanted(), 1);
        let covered = triage(25, 10, 19, 1);
        assert!(!covered.unfixable(), "25 < 26");
        assert_eq!(covered.clw_wanted(), 0);
        assert!(!Triage { cure: true, ..ranger }.unfixable());
        assert!(triage(20, 20, 30, 0).outlasts_hp());
        assert!(!triage(19, 20, 30, 0).outlasts_hp());
    }

    #[test]
    fn town_triage_with_9_53_and_171_gold() {
        // 626789725: HP 14/19, poison 21: 21 >= 14, no CLW: 1 CLW covers it
        // (14 + 16 > 21). With 9 gold the Alchemist's CLW (cost 20,
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
        assert_eq!(clw_to_cover(0, 47, 3), 1, "pending = HP + healing: one HP short");
        assert_eq!(clw_to_cover(-1, 47, 3), 0);
        assert_eq!(clw_to_cover(5, 1, 3), 0, "max HP 1: a potion gives nothing");
        assert_eq!(clw_to_cover(16, 19, 3), 2, "16 a potion at max HP 19: 17 needs two");
    }

    #[test]
    fn recall_only_when_its_delay_can_be_outlived() {
        // Review of #1146 (finding 4): 8 HP, 12 poison to come, no healing,
        // an up staircase known: Recall's 15 to 34 ticks cannot be outlived.
        assert!(!recall_survivable(8, true));
        assert!(!recall_survivable(8, false), "not even the shortest delay");
        assert!(recall_survivable(20, false), "no stairs: the shortest delay is a chance");
        assert!(!recall_survivable(20, true), "stairs known: walk");
        assert!(recall_survivable(35, true), "the longest delay is outlived");
        assert!(!recall_survivable(34, true));
        assert_eq!(recall_delay_charged(45), 34);
        assert_eq!(recall_delay_charged(20), 15, "the no-stairs chance: the shortest delay");
    }

    #[test]
    fn a_carried_cure_is_used_once_the_poison_matters() {
        // The Paladin 1200156945 (#1134) died holding 2 Neutralize Poison.
        assert!(cure_due(20, 20, 50), "the poison takes all the HP");
        assert!(cure_due(10, 20, 50), "half");
        assert!(!cure_due(9, 20, 50));
        assert!(cure_due(1, 20, 0), "0: any poison");
        assert!(cure_due(5, 0, 100), "at 0 HP any poison is due");
        assert!(!cure_due(0, 20, 0), "no poison, nothing to cure");
        assert!(!cure_due(19, 20, 100));
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
