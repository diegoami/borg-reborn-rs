//! Physical capabilities shared by purchases, dispositions and replay.
//!
//! Rows are engine facts, not borg_power bonuses. The eight legacy needs
//! remain intact for historical replay. New records carry these curves and
//! per-copy supply vectors, including every secondary effect.
use std::collections::{BTreeMap, BTreeSet};
use crate::borg_headers::{borg_h::*, borg_trait_h::*};
use crate::borg_town_dispose::{KitItem, Place};
use crate::borg_town_stock::{Stock, BORG_BLACK_MARKET};
use crate::borg_town_utility::{Journey, UNIT};
use crate::borg_headers::borg_item_h::borg_item;
use crate::headers::list_tvals_h::*;
use crate::headers::list_effects_h::*;
use crate::headers::list_player_timed_h::*;
use crate::state::BorgState;
use crate::engine;
use crate::params::{value as param, Param};

pub const PACK: i64 = 8;
pub const HOME: i64 = 9;
const LIGHT: usize = 0;
const FOOD: usize = 1;
const HEAL: usize = 2;
const POISON: usize = 3;
const PHASE: usize = 4;
const RECALL: usize = 5;
const ATTACK: usize = 6;
const TELEPORT: usize = 7;
const LEVEL: usize = 8;
const OTHER: usize = 9;
const DIG: usize = 10;
const TRAPS: usize = 11;
const MONSTERS: usize = 12;
const INVISIBLE: usize = 13;
const MAP: usize = 14;
const ILLUMINATE: usize = 15;
const SPEED: usize = 16;
const FIRE: usize = 17;
const COLD: usize = 18;
const RESPOIS: usize = 19;
const ACID: usize = 20;
const ELEC: usize = 21;
const BUFF: usize = 22;
const MANA: usize = 23;
const EXPERIENCE: usize = 24;
const CURSE: usize = 25;
const ENCHANT: usize = 26;
const RECHARGE: usize = 27;
const CONFUSION: usize = 28;
const BLIND: usize = 29;
const FEAR: usize = 30;
const GLYPH: usize = 31;
const STATS: usize = 32;
const BOOKS: usize = 38;
const BANISH: usize = 47;
const DESTROY: usize = 48;
const WOUNDS: usize = 49;
const IDENTIFY: usize = 50;
const GAIN_EXPERIENCE: usize = 51;
const RECHARGE_TARGET: usize = 52;
const ACQUIRE: usize = 53;
const CUT: usize = 54;
const STUN: usize = 55;
const CONTROL: usize = 56;
const RANGER_MISSILES: usize = 57;
/// borg-reborn-rs (#491): the healing reserve. Every HP a supply heals counts
/// here as well as in HEAL; up to `town_heal_reserve_hp_percent` of max HP
/// it is worth the healing curve's first HP each (linear), past it nothing
/// more than the curve. Zero weight when the parameter is 0.
const HEAL_RESERVE: usize = 58;
/// borg-reborn-rs (#712): flasks of oil kept to throw, apart from the shared
/// ATTACK curve that launcher ammunition saturates. Each flask kept to throw
/// counts one unit here as well as its damage in ATTACK (as an arrow counts
/// in RANGER_MISSILES). Zero weight unless `town_oil_apart`.
const THROWN_OIL: usize = 59;
/// borg-reborn-rs (#716, owner): shots past the Ranger reserve's soft cap
/// (`town_ranger_arrows_max`, the requirement row's negative demand) on a
/// diminishing curve: C's shot curve, weight and demand scaled by
/// `town_ranger_arrows_curve_scale_percent` (as healing's). Any number is
/// worth something, each less than the last. Only the shots past the cap
/// count here. Zero weight unless both are set.
const EXTRA_SHOTS: usize = 60;
/// borg-reborn-rs (#761, owner: "we need to value phase doors much more,
/// especially for rangers"): the first `town_phase_reserve` Phase Doors as a
/// requirement, on top of the PHASE curve (whose first scroll is worth about
/// a third of the trip's escape weight, so 1-gold arrows and 20-gold potions
/// outbid it). Each is worth `town_phase_reserve_percent` of the trip's
/// escape weight; nothing past the reserve. Zero weight when the reserve is 0.
const PHASE_RESERVE: usize = 61;
const RANGED_RESERVE: usize = 62;
/// borg-reborn-rs (#859, owner): with `town_recall_curve_scale_percent` on,
/// the first `town_recall_reserve` Words of Recall as a requirement on top of
/// the RECALL curve, as PHASE_RESERVE is on top of PHASE.
const RECALL_RESERVE: usize = 63;
/// borg-reborn-rs (#863, owner: "being prepared to a variety of situations"):
/// with `town_prepared_percent` on, one small diminishing row per situation's
/// remedy, fed one unit per copy by whatever supplies the situation's
/// capability, so the first copies of each remedy are worth more than the
/// 21st of a volume kind. They sit on top of the condition-present curves.
const PREP_POISON: usize = 64;
const PREP_CURSE: usize = 65;
const PREP_INVISIBLE: usize = 66;
const PREP_CONFUSION: usize = 67;
const PREP_FEAR: usize = 68;
/// borg-reborn-rs (#863, owner: "make staple objects start losing value ...";
/// "it must be a decaying curve"; "Food and Light start decaying from 4,
/// phase door, flasks and CLW from 6, CSW from 4, CCW 2 ... class
/// dependent"): a staple's knee rows carry no supplies and no value, only a
/// knee. weight = decay percent + 128 x the units of one copy, demand = minus
/// the knee in the row's units (a positive demand: no knee). The copies past
/// the knee count for the decay percent of the copy before, so the amount the
/// curve sees is `decayed` (the 8th copy of 7 free keeps 80 percent, the 9th
/// 64, ...). Phase Door, oil, food and light: one row on their capability.
/// Each healing potion has a row of its own, fed one copy (`UNIT`) per potion
/// (Cure Light, Serious and Critical Wounds know their own knees), whose
/// units are the healing units one copy adds to the shared HEAL curve.
const STAPLE_CLW: usize = 69;
const STAPLE_CSW: usize = 70;
const STAPLE_CCW: usize = 71;
const STAPLE_PHASE: usize = 72;
const STAPLE_OIL: usize = 73;
const STAPLE_FOOD: usize = 74;
const STAPLE_LIGHT: usize = 75;
const RESTORE_LIFE_RESERVE: usize = STAPLE_LIGHT + 1;
/// borg-reborn-rs (#987): the first `town_first_cure` healing potions as a
/// requirement, each worth `town_first_cure_percent` of the trip's escape
/// weight, nothing past them. With the Phase Door reserve at 150 percent of
/// that weight a scroll, the first Cure Light Wounds (at most the healing
/// curve's first HP plus a full heal on the healing reserve) is worth less
/// than every reserved Phase Door, so a low-gold start bought no cure. Above
/// the reserve's percent, the first cures come before it. The row only exists
/// with `town_first_cure` above 0.
const FIRST_CURE: usize = RESTORE_LIFE_RESERVE + 1;
// The first-cure row must stay the last of the rows the model holds without the
// device rows: the model without it stops before it (`observe`), so a row appended
// after it would drop out of the off model. The device rows (#895) follow it and
// exist only with `town_device_percent`, padded like the first-cure row.

/// The healing potions' rows.
const HEAL_KINDS: [usize; 3] = [STAPLE_CLW, STAPLE_CSW, STAPLE_CCW];
/// borg-reborn-rs (#895, owner: "we are going to need space for other
/// devices"): with `town_device_percent` on, one requirement row per situation a
/// wand, staff or rod answers, counted in charges (`UNIT` each): a ranged attack,
/// an escape (a Staff of Teleportation), detection (Detect Invisible, Detect
/// Evil, Mapping) and monster control (Slow, Hold, Confuse Monster). The first N
/// charges together are worth `town_device_percent` of the trip weight, so a
/// learned, charged device is kept and bought as variety, not only as one more
/// source on a shared curve (an attack wand competes with arrows and oil).
const DEV_ATTACK: usize = FIRST_CURE + 1;
const DEV_ESCAPE: usize = DEV_ATTACK + 1;
const DEV_DETECT: usize = DEV_ESCAPE + 1;
const DEV_CONTROL: usize = DEV_DETECT + 1;
/// borg-reborn-rs (#1134, owner: "make the town planner start buying CCW and
/// cure poison in the same way that they buy detect invisible"): with
/// `town_prepared_ccw` above 0, a prepared row of its own fed one copy per
/// Potion of Cure Critical Wounds. Held only while it (or a row after it) is
/// on, so the model keeps its shape (and the play) with it off.
const PREP_CCW: usize = DEV_CONTROL + 1;
/// borg-reborn-rs (#1137, owner: "40 just Neutralize Poison"): with
/// `town_prepared_neutralize` above 0, a prepared row of its own fed one copy
/// per Potion of Neutralize Poison, so its percent raises that potion alone,
/// not every poison cure on the shared poison row (Resist Poison, Mushroom of
/// Purging, CCW). Held, as the CCW row, only while a dear row is on.
const PREP_NEUTRALIZE: usize = PREP_CCW + 1;
/// borg-reborn-rs (#1103; review of #1112, F1): one copy of a restoring
/// mushroom (Vigor, Purging) while nothing is drained, a requirement of one on
/// a row of its own, weighted by `town_restore_stat_percent`. It is not counted
/// on the stat rows: a mushroom that restores a drain cannot raise a healthy
/// stat, and a second copy adds nothing.
const RESTORE_STAT_RESERVE: usize = PREP_NEUTRALIZE + 1;
/// #1162: up to two full confusion cures, with extra purchase value only for CSW.
const CSW_RESERVE: usize = RESTORE_STAT_RESERVE + 1;
/// Situation capabilities a device supplies -> its device row. Invisible
/// detection is not here: a Staff of Detect Invisible already fills the prepared
/// Detect Invisible row, one copy a charge (`town_prepared_invisible`), so a
/// second row would count it twice (review of #898, F2). Teleport Other
/// (`OTHER`) is grouped with the control devices: it keeps a monster away.
const DEVICE_ROWS: [(usize, usize); 6] = [(ATTACK, DEV_ATTACK), (TELEPORT, DEV_ESCAPE),
    (MONSTERS, DEV_DETECT), (MAP, DEV_DETECT), (CONTROL, DEV_CONTROL), (OTHER, DEV_CONTROL)];
/// A capability with a knee of its own -> its row.
const STAPLES: [(usize, usize); 4] = [(PHASE, STAPLE_PHASE), (THROWN_OIL, STAPLE_OIL),
    (FOOD, STAPLE_FOOD), (LIGHT, STAPLE_LIGHT)];
const KNEE_UNIT: i64 = 128;
fn knee_row(decay: i64, per_copy: i64) -> i64 { per_copy.max(1) * KNEE_UNIT + decay.clamp(0, 100) }

/// A knee row, unpacked (#863; review F2). `per_copy` means two things, by
/// which rows read it, and the rows are never read the other way: on a
/// `STAPLES` row it is the units of one copy on the capability's own curve
/// (`Model::value` steps by it); on a `HEAL_KINDS` row, which counts potions
/// (`UNIT` each), it is the HEAL units one potion adds to the shared curve
/// (`Model::heal_loss` scales by it and steps by whole potions).
struct Knee { free: i64, per_copy: i64, decay: i64 }

/// None: the row is off (a positive demand, which is what an unset row holds).
fn knee_of(row: &[i64; 5]) -> Option<Knee> {
    if row[2] >= 0 { return None; }
    Some(Knee { free: -row[2], per_copy: row[1] / KNEE_UNIT, decay: row[1] % KNEE_UNIT })
}
/// The amount a curve sees for `amount` units: the first `free` as they are,
/// each further copy of `copy` units worth `decay` percent of the copy before.
fn decayed(amount: i64, free: i64, copy: i64, decay: i64) -> i64 {
    if copy <= 0 || amount <= free { return amount; }
    const ONE: i64 = 1_000_000;
    let (mut seen, mut at, mut keep) = (free, free, ONE);
    for _ in 0..200 {
        if at >= amount { break; }
        let next = (at + copy).min(amount);
        keep = (keep * decay / 100).max(1);
        seen += (next - at) * keep / ONE;
        at = next;
    }
    seen + (amount - at) * keep / ONE
}
/// Situation capability -> its prepared row.
const PREPARED: [(usize, usize); 5] = [(POISON, PREP_POISON), (CURSE, PREP_CURSE),
    (INVISIBLE, PREP_INVISIBLE), (CONFUSION, PREP_CONFUSION), (FEAR, PREP_FEAR)];
const CLASS_RANGER: i32 = 7;
const CLASS_MAGE: i32 = 1;
const CLASS_DRUID: i32 = 2;
const CLASS_PRIEST: i32 = 3;
const CLASS_NECROMANCER: i32 = 4;

/// #1103 (review of #1112, F2): whether an undrained stat of this value counts
/// toward the reserve of a restoring mushroom (Vigor, Purging): the kind is one the
/// Borg eats, and the stat is below 18/100 (the stat rows' own rule).
pub(crate) fn restore_reserved(reserve_kind: bool, stat: i32) -> bool { reserve_kind && stat < 18 + 100 }

/// borg-reborn-rs (#1103, owner: buy "the potions to gain stats" the Borg uses):
/// whether this class drinks the Potion of Brawn, Intellect, Contemplation,
/// Nimbleness or Toughness for `stat` (0 strength .. 4 constitution), which
/// gain that stat and lose another at random. The rule is `borg_junk`'s
/// (`borg_test_stuff`, the `sv_potion_inc_*2` kinds): a class that does not
/// want to risk the other stat never drinks it, so it is worth nothing to
/// buy. The five 8000-gold potions (Strength...) and Augmentation are drunk
/// at once by every class (`borg_use_things`) and are not asked here.
pub(crate) fn drinks_second_stat_potion(class: i32, stat: usize) -> bool {
    let caster = matches!(class, CLASS_MAGE | CLASS_DRUID | CLASS_NECROMANCER);
    match stat {
        0 => !caster,
        1 => matches!(class, CLASS_MAGE | CLASS_NECROMANCER),
        2 => matches!(class, CLASS_PRIEST | CLASS_DRUID),
        3 => !matches!(class, CLASS_MAGE | CLASS_PRIEST | CLASS_DRUID | CLASS_NECROMANCER),
        _ => true,
    }
}
/// borg-reborn-rs (#1371): the stat (0 strength .. 4 constitution) of a
/// gain-one/lose-one potion (Brawn, Intellect, Contemplation, Nimbleness,
/// Toughness: `object.txt`, `RESTORE_STAT`, `GAIN_STAT`, `LOSE_RANDOM_STAT`).
pub(crate) fn second_stat_kind(st: &BorgState, sval: i32) -> Option<usize> {
    second_stat_kind_of(&st.item_val, sval)
}

fn second_stat_kind_of(iv: &crate::borg_item_val::ItemValState, sval: i32) -> Option<usize> {
    [iv.sv_potion_inc_str2, iv.sv_potion_inc_int2, iv.sv_potion_inc_wis2, iv.sv_potion_inc_dex2, iv.sv_potion_inc_con2]
        .iter().position(|&kind| kind >= 0 && kind == sval)
}

/// borg-reborn-rs (#1371): the one rule for buying, keeping and drinking
/// the gain-one/lose-one potions is on.
pub(crate) fn stat_trade_policy() -> bool { param(Param::town_stat_trade_policy) != 0 }

/// borg-reborn-rs (#1371): a stat value on the engine's table scale, in
/// tenths of a table step. The tables (`adj_*`) are indexed by
/// `player-calcs.c:calc_bonuses`'s `ind`: `v - 3` below 18, and
/// `15 + (v - 18) / 10` from 18; a point below 18 is a step, a point of
/// 18/xx a tenth of one. The base value never passes 18/100
/// (`player.c:player_stat_inc:149`).
pub(crate) fn stat_tenths(v: i32) -> i64 {
    let v = i64::from(v.clamp(3, 18 + 100));
    if v < 18 { (v - 3) * 10 } else { 150 + (v - 18) }
}

/// borg-reborn-rs (#1371): the gain of `GAIN_STAT` from base value `v`, in
/// tenths, summed over the engine's equally likely rolls, and their number
/// (`player.c:player_stat_inc:145-169`): +1 below 18; from 18 to 18/89,
/// `randint1(g) + g / 2` with `g = ((118 - v) / 2 + 3) / 2`, at most 18/99;
/// 18/90 or more becomes 18/100; nothing at 18/100.
pub(crate) fn stat_gain_tenths(v: i32) -> (i64, i64) {
    if v >= 18 + 100 { return (0, 1); }
    if v < 18 { return (stat_tenths(v + 1) - stat_tenths(v), 1); }
    if v >= 18 + 90 { return (stat_tenths(18 + 100) - stat_tenths(v), 1); }
    let g = (((18 + 100 - v) / 2 + 3) / 2).max(1);
    let sum = (1..=g).map(|r| stat_tenths((v + r + g / 2).min(18 + 99)) - stat_tenths(v)).sum();
    (sum, i64::from(g))
}

/// borg-reborn-rs (#1371): the loss of a permanent `player_stat_dec` from
/// base value `v`, in tenths (`player.c:player_stat_dec:187-196`): 10
/// points above 18/10, down to 18 from 18/01 to 18/10, a point from 4 to
/// 18, nothing at 3. Sustains do not stop it (`effect-handler-general.c:
/// effect_handler_LOSE_RANDOM_STAT:852-869` calls it directly).
pub(crate) fn stat_loss_tenths(v: i32) -> i64 {
    let after = if v > 18 + 10 { v - 10 } else if v > 18 { 18 } else if v > 3 { v - 1 } else { v };
    stat_tenths(v) - stat_tenths(after)
}

/// borg-reborn-rs (#1371): whether drinking the gain-one/lose-one potion of
/// `stat` is worth it at base stats `stats` (strength .. constitution): the
/// expected weighted gain reaches `margin` percent of the expected weighted
/// loss. `LOSE_RANDOM_STAT` takes one of the other four stats, each with
/// chance 1/4 (`effect_handler_LOSE_RANDOM_STAT:854-858`). `weights` are
/// percent per stat. A capped stat gains nothing and is never worth it.
pub(crate) fn stat_trade_worth(stats: [i32; 5], stat: usize, weights: [i64; 5], margin: i64) -> bool {
    let (gain, rolls) = stat_gain_tenths(stats[stat]);
    if gain <= 0 { return false; }
    let loss: i64 = (0..5).filter(|&s| s != stat).map(|s| weights[s].max(0) * stat_loss_tenths(stats[s])).sum();
    // gain / rolls * w >= margin / 100 * loss / 4
    400 * gain * weights[stat].max(0) >= margin.max(0) * rolls * loss
}

/// borg-reborn-rs (#1371): whether this character buys, keeps and drinks the
/// gain-one/lose-one potion of `stat`: a class that drinks it
/// (`drinks_second_stat_potion`, `borg_junk`'s table) and a trade that is
/// worth it (`stat_trade_worth`), with the weights `town_stat_trade_weight_*`
/// and the margin `town_stat_trade_margin_percent` (per class with
/// `__<class>`). The stats are the permanent ones, `player->stat_max`
/// (review of #1372): the potion restores the stat it gains before
/// `GAIN_STAT` (`RESTORE_STAT`, `effect_handler_RESTORE_STAT`), so the gain
/// starts from the maximum, and `LOSE_RANDOM_STAT` lowers the maximum of
/// another (`player_stat_dec` with `permanent`). A temporary drain changes
/// neither. The restoration itself is not priced: the engine undoes a drain
/// at the next level-up and a Restore potion does it for less.
pub(crate) fn second_stat_potion_wanted(st: &BorgState, stat: usize) -> bool {
    let stats = engine::player_notice().stat_max.map(i32::from);
    second_stat_potion_wanted_at(st.borg.tr(BI_CLASS), stats, stat)
}

/// `second_stat_potion_wanted` at the given permanent stats (strength ..
/// constitution), without the game state.
pub(crate) fn second_stat_potion_wanted_at(class: i32, stats: [i32; 5], stat: usize) -> bool {
    if !drinks_second_stat_potion(class, stat) { return false; }
    let weights = [
        param(Param::town_stat_trade_weight_str), param(Param::town_stat_trade_weight_int),
        param(Param::town_stat_trade_weight_wis), param(Param::town_stat_trade_weight_dex),
        param(Param::town_stat_trade_weight_con),
    ].map(i64::from);
    stat_trade_worth(stats, stat, weights, i64::from(param(Param::town_stat_trade_margin_percent)))
}

/// borg-reborn-rs (#1371): the gain-one/lose-one potion `sval` is a kind
/// this character will not drink at these permanent stats.
fn stat_trade_refused_at(
    iv: &crate::borg_item_val::ItemValState, class: i32, stats: [i32; 5], tval: u32, sval: i32,
) -> bool {
    tval == TV_POTION
        && second_stat_kind_of(iv, sval).is_some_and(|stat| !second_stat_potion_wanted_at(class, stats, stat))
}

/// borg-reborn-rs (#1371): under town_stat_trade_policy, a gain-one/lose-one
/// potion this character will not drink (`second_stat_potion_wanted`).
fn stat_trade_refused(st: &BorgState, tval: u32, sval: i32) -> bool {
    stat_trade_policy() && refused_stat_trade_kind(st, tval, sval)
}

/// borg-reborn-rs (#1375): one rule for kinds this character refuses to
/// carry, switched on by `town_refused_kind_policy` (and, for the potions of
/// #1371, `town_stat_trade_policy`): the shop's offers (`supplies_of_kind`),
/// the pack and Home (`unwanted_home_item`, `borg_town_home::classify`) and
/// the floor (`refused_take_kind`) ask it. A kind is refused when it is
/// `WORTHLESS` (#443; Monster Confusion without `caution_cornered`), a
/// gain-one/lose-one potion the character will not drink (#1371), or, for an
/// Unlight class (the Necromancer, the only `PF_UNLIGHT` class in class.txt),
/// a Wooden Torch or a Scroll of Light (#1273). It does not read the stack's
/// knowledge: a caller asks it of a known kind.
pub(crate) fn refused_kind(st: &BorgState, tval: u32, sval: i32) -> bool {
    worthless(tval, sval)
        || stat_trade_refused(st, tval, sval)
        || (param(Param::town_refused_kind_policy) != 0
            && unlight_refused(st.borg.tr(BI_CLASS), &st.item_val, tval, sval))
}

/// borg-reborn-rs (#1375): an Unlight class (class.txt: `PF_UNLIGHT`, the
/// Necromancer only; `borg_light::CLASS_NECROMANCER`) does not use a Wooden
/// Torch or a Scroll of Light, the kinds `borg_town_shadow` leaves out of its
/// shop candidates and `borg_flow_take::unwanted_light_scroll` leaves on the
/// floor.
fn unlight_refused(class: i32, iv: &crate::borg_item_val::ItemValState, tval: u32, sval: i32) -> bool {
    class == crate::borg_light::CLASS_NECROMANCER
        && ((tval == TV_LIGHT && sval == iv.sv_light_torch)
            || (tval == TV_SCROLL && sval == iv.sv_scroll_light))
}

/// borg-reborn-rs (#1372 review): a kind the shipped dungeon policies still
/// use, so it is worth a walk although the town would not keep it: Treasure
/// Detection with `item_use_idle` (#1250), and the junk potions thrown with
/// `attack_throw_junk_potions` (#1253).
fn used_in_the_dungeon(st: &BorgState, tval: u32, sval: i32) -> bool {
    (param(Param::item_use_idle) != 0 && tval == TV_SCROLL && sval == st.item_val.sv_scroll_treasure_detection)
        || (param(Param::attack_throw_junk_potions) != 0 && tval == TV_POTION
            && crate::borg_junk::throwable_junk_potion_kind(sval))
}

/// borg-reborn-rs (#1371, #1375): a floor object of a known kind the Borg
/// refuses to carry is not worth a walk: the town would only sell or drop it.
/// The refused gain-one/lose-one potions count under `town_stat_trade_floor`;
/// the rest of `refused_kind` counts under `town_refused_kind_policy`, except
/// a kind a dungeon policy still uses (`used_in_the_dungeon`, review of
/// #1372). Only this walk is skipped: a stack stepped on while going elsewhere
/// is picked up by the engine's `pickup_always`; a refused potion is then
/// dropped on a quiet dungeon turn (`borg_junk::borg_discard_refused_stack`)
/// and anything else goes to the town's disposal.
pub(crate) fn refused_take_kind(st: &BorgState, kidx: usize) -> bool {
    if kidx == 0 { return false; }
    let kind = engine::kind(kidx);
    if !kind.aware { return false; }
    let (tval, sval) = (kind.tval as u32, i32::from(kind.sval));
    (stat_trade_floor() && refused_stat_trade_kind(st, tval, sval))
        || (param(Param::town_refused_kind_policy) != 0
            && refused_kind(st, tval, sval) && !used_in_the_dungeon(st, tval, sval))
}

/// borg-reborn-rs (#1371): the one place the floor rules ask: a refused
/// gain-one/lose-one potion kind, at the permanent stats.
pub(crate) fn refused_stat_trade_kind(st: &BorgState, tval: u32, sval: i32) -> bool {
    tval == TV_POTION && second_stat_kind(st, sval).is_some()
        && stat_trade_refused_at(&st.item_val, st.borg.tr(BI_CLASS),
            engine::player_notice().stat_max.map(i32::from), tval, sval)
}

/// borg-reborn-rs (#1371, review of #1372): the floor and dungeon rules for
/// the refused stat-trade potions are on (needs `town_stat_trade_policy`).
pub(crate) fn stat_trade_floor() -> bool {
    stat_trade_policy() && param(Param::town_stat_trade_floor) != 0
}

const COUNT: usize = NAMES.len();
/// borg-reborn-rs (food floor): the food items the core requires to be
/// prepared for any level (`borg_prepared`: "5 Food").
const READY_FOOD: i64 = 5;
/// borg-reborn-rs (#455 review): one Ration of Food's nutrition in the
/// food floor's unit (object.txt: Ration of Food, NOURISH:INC_BY 30).
pub(crate) const RATION_NUTRITION: i64 = 30;
/// borg-reborn-rs (#451): the "Hungry" grade of the FOOD timer is 15 percent
/// (player_timed.txt, `grade:y:15:Hungry`); `BI_ISHUNGRY` is food below
/// `PY_FOOD_HUNGRY`, and the core reads a Remove Hunger scroll then. A
/// `NOURISH:INC_TO` food (Remove Hunger 50, Honey-cake and Draught 60,
/// Waybread 75: object.txt) is worth what it adds from there.
const HUNGRY_PERCENT: i64 = 15;

const _: () = assert!(CSW_RESERVE + 1 == NAMES.len());
pub const NAMES: [&str; 86] = ["light turns", "nutrition", "healing HP", "poison cure", "phase", "Recall",
    "ranged damage", "teleport", "level escape", "teleport other", "digging", "trap/door detection",
    "monster detection", "invisible detection", "mapping", "illumination", "speed", "resist fire",
    "resist cold", "resist poison", "resist acid", "resist electricity", "combat buff", "mana",
    "restore experience", "remove curse", "enchantment", "recharging", "confusion cure", "blindness cure",
    "fear cure", "glyph", "strength", "intelligence", "wisdom", "dexterity", "constitution", "charisma",
    "book 0", "book 1", "book 2", "book 3", "book 4", "book 5", "book 6", "book 7", "book 8",
    "banishment", "destruction", "wound/stun cure", "identify runes", "gain experience", "recharge target", "equipment opportunity", "cut reduction", "stun reduction", "monster control", "Ranger usable shots", "healing reserve", "thrown oil", "extra shots", "phase reserve", "ranged reserve", "recall reserve",
    "prepared: poison cure", "prepared: remove curse", "prepared: detect invisible", "prepared: confusion cure", "prepared: fear cure", "staple: Cure Light Wounds", "staple: Cure Serious Wounds", "staple: Cure Critical Wounds", "staple: Phase Door", "staple: oil", "staple: food", "staple: light", "restore life reserve", "first cure",
    "device: attack charges", "device: escape charges", "device: detection charges", "device: control charges",
    "prepared: Cure Critical Wounds", "prepared: Neutralize Poison", "restore stat reserve", "CSW reserve"];

/// Curves: [capability, weight in utility thousandths, demand, carried, Home].
/// Supplies: [scope (shop 0..7, pack 8, Home 9), slot, capability, units/copy].
/// All dynamic class/level/effect facts are frozen here for engine-free replay.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Model {
    pub available: Vec<[i64;3]>,
    pub packing: Option<crate::borg_town_packing::Packing>,
    pub curves: Vec<[i64; 5]>,
    pub supplies: Vec<[i64; 4]>,
}

/// borg-reborn-rs (#716): the extra shots past the soft cap, which the
/// Ranger reserve's requirement row holds as its negative demand (no cap, no
/// extra value): the curve counts the shots past it.
fn extra_gain(m: &Model, r: &[i64; 5], n: i64) -> i64 {
    let Some(reserve) = m.curves.get(RANGER_MISSILES) else { return 0 };
    if reserve[2] >= 0 { return 0; }
    let cap = -reserve[2];
    let past = |a: i64| (a - cap).max(0);
    coverage(r, past((r[3] + n).max(0))) - coverage(r, past(r[3]))
}

fn coverage(row: &[i64; 5], amount: i64) -> i64 {
    if amount <= 0 || row[1] <= 0 { return 0; }
    if row[0] == CSW_RESERVE as i64 {
        let first = amount.min(UNIT);
        let second = (amount - first).min(UNIT);
        return row[1] * first / UNIT + (row[1] / 4) * second / UNIT;
    }
    if row[2] < 0 {
        let target = -row[2];
        let reserved = row[1] * amount.min(target) / target;
        // Extra light has a smaller diminishing return above the reserve.
        // Other negative-demand rows are satisfied once, such as a cure.
        if matches!(row[0] as usize, LIGHT | RESTORE_LIFE_RESERVE) && amount > target {
            let extra = amount - target;
            return reserved + (row[1] / 4).saturating_mul(extra) / (target + extra);
        }
        return reserved;
    }
    row[1].saturating_mul(amount) / amount.saturating_add(row[2].max(1))
}

impl Model {
    /// The coverage of `amount` on a curve (#863): the amount the curve sees
    /// is `decayed` when the capability has a knee row (Phase Door, oil, food,
    /// light). Without it, `coverage`.
    fn value(&self, id: usize, r: &[i64; 5], amount: i64) -> i64 {
        let knee = STAPLES.iter().find(|&&(cap, _)| cap == id).and_then(|&(_, row)| self.curves.get(row)).and_then(knee_of);
        match knee {
            Some(k) => coverage(r, decayed(amount, k.free, k.per_copy, k.decay)),
            None => coverage(r, amount),
        }
    }
    /// The healing units the decay takes off `copies` (in `UNIT`s per copy) of
    /// each healing potion: what a pack of 40 Cure Light Wounds adds to the
    /// HEAL curve is less than 40 times one. `after` adds the units of a
    /// change by row.
    fn heal_loss(&self, change: Option<&BTreeMap<usize, i64>>) -> i64 {
        HEAL_KINDS.iter().filter_map(|&row| {
            let curve = self.curves.get(row)?;
            let knee = knee_of(curve)?;
            // The row counts potions: step by one potion (`UNIT`), then price the
            // potions the decay takes off in the HEAL units one potion adds.
            let held = (curve[3] + change.and_then(|c| c.get(&row)).copied().unwrap_or(0)).max(0);
            Some(knee.per_copy * (held - decayed(held, knee.free, UNIT, knee.decay)) / UNIT)
        }).sum()
    }
    pub fn units(&self, scope: i64, slot: i64, count: i64) -> Vec<(usize, i64)> {
        self.supplies.iter().filter(|r| r[0] == scope && r[1] == slot)
            .map(|r| (r[2] as usize, r[3] * count)).collect()
    }
    pub fn gain(&self, units: &[(usize, i64)]) -> i64 {
        let mut sum = BTreeMap::<usize, i64>::new();
        for &(id, n) in units { *sum.entry(id).or_default() += n; }
        sum.iter().map(|(&id, &n)| {
            let r = &self.curves[id];
            if id == RECHARGE && self.curves[RECHARGE_TARGET][3] == 0 { return 0; }
            // A healing potion's staple row only counts copies for `heal_loss`.
            if HEAL_KINDS.contains(&id) { return 0; }
            if id == EXTRA_SHOTS { return extra_gain(self, r, n); }
            if id == HEAL {
                let (before, after) = ((r[3] - self.heal_loss(None)).max(0), ((r[3] + n).max(0) - self.heal_loss(Some(&sum))).max(0));
                return coverage(r, after) - coverage(r, before);
            }
            self.value(id, r, (r[3] + n).max(0)) - self.value(id, r, r[3])
        }).sum()
    }
    pub fn ware(&self, shop: u8, slot: u8, count: i32) -> i64 {
        self.gain(&self.units(i64::from(shop), i64::from(slot), i64::from(count)))
    }
    pub fn ranged_reserve_copies(&self, shop: u8, slot: u8, picked: i32) -> Option<i32> {
        let row = self.curves.get(RANGED_RESERVE)?;
        let short = -row[2] - row[3];
        if short <= 0 { return None; }
        let per_copy = self.units(i64::from(shop), i64::from(slot), 1)
            .into_iter().find_map(|(id, n)| (id == RANGED_RESERVE).then_some(n))?;
        Some(picked.min(((short + per_copy - 1) / per_copy) as i32))
    }
    pub fn scope(it: &KitItem) -> i64 { if it.place == Place::Pack { PACK } else { HOME } }
    /// The opt-in fuel curve uses negative demand for its reserve. Home
    /// and other excess fuel cannot justify storing the last carried supply.
    pub fn protects_fuel(&self, it: &KitItem) -> bool {
        let Some(row) = self.curves.get(LIGHT) else { return false; };
        if row[1] <= 0 || row[2] >= 0 { return false; }
        self.units(Self::scope(it), i64::from(it.slot), i64::from(it.qty)).iter()
            .any(|&(id, turns)| id == LIGHT && turns > 0 && row[3] - turns < -row[2])
    }
    /// borg-reborn-rs (#1151): the copies of this stack one weighted reserve
    /// row (negative demand) still needs once the other carried supply is
    /// counted. The copy's own units come from the model, except for light:
    /// the model spreads a flask stack's refuelling bottles over the whole
    /// stack, so the floor counts the physical fuel of a copy
    /// (`KitItem::fuel`), when it is known and the stack supplies light.
    fn row_keep(&self, id: usize, it: &KitItem) -> i32 {
        let Some(row) = self.curves.get(id) else { return 0; };
        if row[1] <= 0 || row[2] >= 0 { return 0; }
        let modeled: i64 = self.units(Self::scope(it), i64::from(it.slot), 1).iter()
            .filter(|&&(cap, _)| cap == id).map(|&(_, n)| n).sum();
        if modeled <= 0 { return 0; }
        let per_copy = if id == LIGHT && it.fuel > 0 { it.fuel } else { modeled };
        let other = row[3] - modeled * i64::from(it.qty);
        let missing = (-row[2] - other).max(0);
        ((missing + per_copy - 1) / per_copy).clamp(0, i64::from(it.qty)) as i32
    }
    /// borg-reborn-rs (#1151): the copies of a fuel stack the light reserve
    /// still needs; the copies past them can go. `protects_fuel` asks the
    /// same of the whole stack, on the modeled units. A CL17 Mage with 34
    /// torches and a 20,000-turn reserve kept all of them (four would do) at
    /// -12 speed.
    /// The dispositions read `reserve_keep`, which includes this row.
    #[cfg(test)]
    pub fn fuel_keep(&self, it: &KitItem) -> i32 {
        self.row_keep(LIGHT, it)
    }
    /// borg-reborn-rs (#1151, review of #1155): the copies of this stack
    /// every weighted reserve row still needs (the light, Phase Door, Restore
    /// Life and ranged reserves, a first cure, a spell book): what may never
    /// be treated as excess.
    pub fn reserve_keep(&self, it: &KitItem) -> i32 {
        (0..self.curves.len()).map(|id| self.row_keep(id, it)).max().unwrap_or(0)
    }
    /// borg-reborn-rs (#1151): the copies of a curse-removing ware the Borg
    /// would read now, while it has a curse it wants removed (the CURSE row
    /// has weight: `BI_FIRST_CURSED`): what the row still lacks, in copies.
    /// Zero when there is no curse or the ware does not remove one.
    pub fn decurse_copies(&self, shop: u8, slot: u8) -> i32 {
        let Some(row) = self.curves.get(CURSE) else { return 0; };
        if row[1] <= 0 || row[2] >= 0 { return 0; }
        let per_copy: i64 = self.units(i64::from(shop), i64::from(slot), 1).iter()
            .filter(|&&(id, _)| id == CURSE).map(|&(_, n)| n).sum();
        if per_copy <= 0 { return 0; }
        let missing = (-row[2] - row[3]).max(0);
        ((missing + per_copy - 1) / per_copy).min(i64::from(i32::MAX)) as i32
    }
    /// Copies this pack stack must retain after counting other carried stacks.
    pub fn restore_life_keep(&self, item: &KitItem) -> i32 {
        let Some(row) = self.curves.get(RESTORE_LIFE_RESERVE) else { return 0; };
        if row[2] >= 0 || !self.units(Self::scope(item), i64::from(item.slot), 1)
            .contains(&(RESTORE_LIFE_RESERVE, UNIT)) {
            return 0;
        }
        ((-row[2] - row[3]) / UNIT + i64::from(item.qty)).clamp(0, i64::from(item.qty)) as i32
    }

    /// Copies of one kind needed to meet the live kit's explicit reserve
    /// rows, after crediting the other carried ways to meet each reserve.
    pub(crate) fn home_stock_floor(&self, item: &KitItem, held: i32) -> i32 {
        self.units(Self::scope(item), i64::from(item.slot), 1).into_iter()
            .filter_map(|(id, per_copy)| {
                let row = self.curves.get(id)?;
                if row[1] == 0 || row[2] >= 0 { return None; }
                let other = (row[3] - i64::from(held) * per_copy).max(0);
                let missing = (-row[2] - other).max(0);
                Some(((missing + per_copy - 1) / per_copy).min(i64::from(i32::MAX)) as i32)
            }).max().unwrap_or(0)
    }

    pub fn keep(&self, it: &KitItem) -> i64 {
        -self.gain(&self.units(Self::scope(it), i64::from(it.slot), -i64::from(it.qty)))
    }
    pub fn take(&self, it: &KitItem) -> i64 {
        self.gain(&self.units(Self::scope(it), i64::from(it.slot), i64::from(it.qty)))
    }
    pub fn reserve(&self, it: &KitItem, sign: i64, discount: i64) -> i64 {
        self.units(Self::scope(it), i64::from(it.slot), i64::from(it.qty)).into_iter().filter(|(id, _)| !HEAL_KINDS.contains(id)).map(|(id, n)| {
            let r = &self.curves[id];
            let from = r[4];
            let to = (from + sign * n).max(0);
            (self.value(id, r, to) - self.value(id, r, from)) * discount / UNIT
        }).sum()
    }
    pub fn change(&mut self, it: &KitItem, sign: i64) {
        for (id, n) in self.units(Self::scope(it), i64::from(it.slot), sign * i64::from(it.qty)) {
            self.curves[id][3] = (self.curves[id][3] + n).max(0);
        }
    }
    pub fn change_home(&mut self, it: &KitItem, sign: i64) {
        for (id, n) in self.units(Self::scope(it), i64::from(it.slot), sign * i64::from(it.qty)) {
            self.curves[id][4] = (self.curves[id][4] + n).max(0);
        }
    }
    /// The capabilities a ware supplies, one bit each, for grouping wares that
    /// close the same need (`shared_groups`). 128 bits (#863 review F1): ids
    /// reach 75 and a 64-bit mask folded every id from 63 up onto one bit, which
    /// merged unrelated groups (a fear remedy and a poison remedy) and made the
    /// solver choose one option for both. Bit 63 stays the packing-room sentinel,
    /// as in #864 (the Recall reserve, id 63, shares it as it did).
    pub fn mask(&self, shop: u8, slot: u8) -> u128 {
        self.units(i64::from(shop), i64::from(slot), 1).iter().fold(0, |m, &(id, _)| m | (1u128 << id))
    }
    pub fn valid(&self) -> bool {
        self.available.iter().all(|r| (0..=7).contains(&r[0]) && (0..=255).contains(&r[1]) && (0..=255).contains(&r[2]))
            && (53..=COUNT).contains(&self.curves.len()) && self.packing.as_ref().is_none_or(|p| p.valid()) && self.curves.iter().enumerate().all(|(i, r)| r[0] == i as i64 && r[1] >= 0 && r[3] >= 0 && r[4] >= 0)
            && self.supplies.iter().all(|r| (0..=HOME).contains(&r[0]) && r[1] >= 0 && (0..self.curves.len() as i64).contains(&r[2]) && r[3] > 0)
    }
}

/// Context for the pure effect classifier; engine facts are obtained once.
struct Context<'a> {
    st: &'a BorgState,
    journey: Journey,
    lantern: bool,
    light_goal: i64,
    light_reserve: i64,
    restore_life_reserve: i64,
    restore_stat: bool,
    ranged_reserve: i64,
    readiness: Option<crate::borg_ranged::Need>,
    launcher_reserve: i64,
}

pub(crate) fn ranged_shopping() -> bool { param(Param::town_ranged_readiness) != 0 }

fn timed_cap(timed: u32) -> Option<usize> {
    match timed {
        TMD_FAST | TMD_SPRINT => Some(SPEED),
        TMD_OPP_FIRE => Some(FIRE), TMD_OPP_COLD => Some(COLD), TMD_OPP_POIS => Some(RESPOIS),
        TMD_OPP_ACID => Some(ACID), TMD_OPP_ELEC => Some(ELEC),
        // borg-reborn-rs (#486): not TMD_BOLD. A Bold timed effect is no buff
        // the core uses: the core takes Boldness only to cure fear (a Potion of
        // Boldness is CURE:AFRAID, priced through FEAR; borg_caution and
        // borg_recover quaff it when afraid), and the hunger list drinks wine
        // as food. The only kind with TIMED_INC:BOLD in object.txt is the
        // Pint of Fine Wine (no activation or class spell grants it), whose
        // buff value bought 14-21 pints a game that were never drunk for it.
        TMD_HERO | TMD_SHERO | TMD_BLESSED | TMD_SHIELD | TMD_STONESKIN | TMD_PROTEVIL
            | TMD_INVULN | TMD_ATT_ACID | TMD_ATT_ELEC | TMD_ATT_FIRE | TMD_ATT_COLD
            | TMD_ATT_POIS | TMD_ATT_EVIL | TMD_ATT_DEMON | TMD_ATT_VAMP | TMD_POWERSHOT => Some(BUFF),
        TMD_SINVIS => Some(INVISIBLE), TMD_TELEPATHY => Some(MONSTERS),
        _ => None,
    }
}

/// RANDOM/SELECT alternatives are mutually exclusive. Use the guaranteed
/// component-wise service rather than adding incompatible branches.
#[cfg(test)] // borg-reborn-rs (#400): only the effect-table tests use it.
fn effect_supply(effects: &[(u16, i32, i64)], hp: i64, service_turns: i64) -> Vec<(usize, i64)> {
    effect_supply_for_consumer(effects, hp, service_turns, false)
}

pub(crate) fn effect_supply_for_consumer(effects: &[(u16, i32, i64)], hp: i64, service_turns: i64, confusion: bool) -> Vec<(usize, i64)> {
    let mut sum = BTreeMap::<usize, i64>::new();
    let mut i = 0;
    while i < effects.len() {
        let e = effects[i];
        if matches!(u32::from(e.0), EF_RANDOM | EF_SELECT) {
            let n = (e.2.max(0) as usize).min(effects.len() - i - 1);
            let branches: Vec<BTreeMap<usize, i64>> = effects[i+1..i+1+n].iter()
                .map(|&b| effect_supply_linear(&[b], hp, service_turns, confusion).into_iter().collect()).collect();
            if let Some(first) = branches.first() {
                for &id in first.keys() {
                    let amount = branches.iter().map(|b| *b.get(&id).unwrap_or(&0)).min().unwrap_or(0);
                    *sum.entry(id).or_default() += amount;
                }
            }
            i += n + 1;
        } else {
            for (id, n) in effect_supply_linear(&[e], hp, service_turns, confusion) { *sum.entry(id).or_default() += n; }
            i += 1;
        }
    }
    sum.into_iter().filter(|&(_, n)| n > 0).collect()
}

/// borg-reborn-rs (#491, owner: "budget healing speed"): the healing units of
/// one HEAL_HP use. The HP it restores (capped at max HP), weighted by how
/// fast it brings the character back to max HP: one that heals at least
/// `speed_percent` of max HP in one swallow counts in full; a smaller one
/// counts `heal x heal / (speed_percent of max HP)`, so a Cure Light Wounds
/// (20 HP) is worth less the more HP the character has. 0 = HP alone.
fn heal_units(amount: i64, hp: i64, speed_percent: i64) -> i64 {
    let heal = amount.max(1).min(hp.max(1));
    // The full-value heal in hundredths of an HP, so no rounding moves it.
    let full = hp.max(1) * speed_percent.clamp(0, 100);
    if full > heal * 100 { heal * heal * UNIT * 100 / full } else { heal * UNIT }
}

/// borg-reborn-rs (#487): what a healing use puts in the reserve: the real
/// HP with `town_heal_reserve_real_hp`, else the speed-weighted HP.
fn reserve_heal(speed_weighted: i64, real: i64, real_hp: bool) -> i64 {
    if real_hp { real } else { speed_weighted }
}

fn reduction_cap(subtype: u32) -> Option<usize> {
    match subtype { TMD_POISONED => Some(POISON), TMD_CONFUSED => Some(CONFUSION),
        TMD_BLIND => Some(BLIND), TMD_AFRAID => Some(FEAR), TMD_CUT => Some(CUT),
        TMD_STUN => Some(STUN), _ => None }
}

/// A conservative independent-rod schedule. Initial ready rods come from
/// native ceil(timeout/cooldown); do not average away a ready rod. Each use
/// costs an action. Charge is allocated to at most that many charging rods.
fn rod_uses(quantity: i64, timeout: i64, cooldown: i64, horizon: i64) -> i64 {
    let q = quantity.clamp(1, 255);
    let cool = cooldown.max(1);
    let first_ready = rod_ready_after(q, timeout, cool);
    let mut timeout = timeout.saturating_sub(first_ready.saturating_mul(q)).max(0);
    let mut uses = 0;
    // Forecast subsequent charges with the native average cooldown.
    for _ in first_ready..horizon {
        let charging = (timeout as u64).div_ceil(cool as u64).min(q as u64) as i64;
        if charging < q { uses += 1; timeout += cool; }
        let charging = (timeout as u64).div_ceil(cool as u64).min(q as u64) as i64;
        timeout -= charging.min(timeout);
    }
    uses
}

/// Effects with contextual relevance, in physical units. Metadata/control
/// effects do not earn utility by themselves. Harmful effects never do.
fn effect_supply_linear(effects: &[(u16, i32, i64)], hp: i64, service_turns: i64, confusion: bool) -> Vec<(usize, i64)> {
    use crate::headers::list_projections_h::*;
    let mut out = BTreeMap::<usize, i64>::new();
    // borg-reborn-rs (#487): the real HP restored, for the healing reserve.
    let mut real_heal = 0;
    for &(effect, subtype, amount) in effects {
        if u32::from(effect) == EF_HEAL_HP {
            real_heal += heal_units(amount, hp, 0);
        }
        let n = amount.max(1) * UNIT;
        let service = service_turns.max(1) * UNIT;
        let role = match u32::from(effect) {
            // borg-reborn-rs (#451): INC_TO (subtype 3) fills to `amount` percent of a full stomach
            // (effect_handler_NOURISH). Read when hungry (HUNGRY_PERCENT) it gives the difference.
            EF_NOURISH if subtype == 3 && amount > HUNGRY_PERCENT => Some((FOOD, (amount - HUNGRY_PERCENT) * UNIT)),
            EF_NOURISH if subtype == 3 => None,
            EF_NOURISH if subtype != 1 && amount > 0 => Some((FOOD, n)), // A decreasing food effect is not nourishment.
            EF_HEAL_HP => Some((HEAL, heal_units(amount, hp, i64::from(param(Param::town_heal_speed_percent))))),
            EF_CURE => reduction_cap(subtype as u32).map(|cap| (cap, n)),
            EF_TIMED_DEC if amount > 0 => reduction_cap(subtype as u32).map(|cap| (cap, n)),
            // borg-reborn-rs (#452): a speed effect is one emergency use, not
            // `amount` turns of buff. The potion's +10 speed (object.txt,
            // Potion of Speed: TIMED_INC:FAST, 20+2d10 turns) is a reserve
            // the core drinks in a dangerous fight (borg_defend_aux_speed),
            // priced like the escapes (exposure), per potion. Counted in turns against
            // the whole trip it was one percent of the demand and was sold.
            EF_TIMED_INC | EF_TIMED_INC_NO_RES | EF_TIMED_SET => timed_cap(subtype as u32)
                .map(|cap| (cap, if cap == SPEED { UNIT } else { n })),
            EF_TELEPORT => Some((TELEPORT, UNIT)), EF_TELEPORT_LEVEL => Some((LEVEL, UNIT)),
            EF_TELEPORT_TO => Some((OTHER, UNIT)), EF_RECALL => Some((RECALL, UNIT)),
            EF_RESTORE_MANA => Some((MANA, n)), EF_RESTORE_EXP => Some((EXPERIENCE, UNIT)),
            EF_GAIN_EXP => Some((GAIN_EXPERIENCE, UNIT)),
            EF_IDENTIFY => Some((IDENTIFY, UNIT)),
            EF_ACQUIRE => Some((ACQUIRE, n)),
            EF_RESTORE_STAT | EF_GAIN_STAT if (0..5).contains(&subtype) => Some((STATS + subtype as usize, UNIT)),
            EF_REMOVE_CURSE => Some((CURSE, n)), EF_ENCHANT | EF_BRAND_WEAPON | EF_BRAND_AMMO | EF_BRAND_BOLTS => Some((ENCHANT, UNIT)),
            EF_RECHARGE => Some((RECHARGE, UNIT)),
            EF_MAP_AREA | EF_LIGHT_LEVEL => Some((MAP, service)), EF_LIGHT_AREA => Some((ILLUMINATE, service)),
            EF_DETECT_TRAPS | EF_DETECT_DOORS | EF_DETECT_STAIRS => Some((TRAPS, service)),
            EF_DETECT_EVIL | EF_DETECT_VISIBLE_MONSTERS | EF_DETECT_LIVING_MONSTERS | EF_DETECT_SOUL | EF_READ_MINDS => Some((MONSTERS, service)),
            EF_DETECT_INVISIBLE_MONSTERS => Some((INVISIBLE, service)),
            EF_DETECT_OBJECTS | EF_SENSE_OBJECTS | EF_DETECT_GOLD | EF_SENSE_GOLD | EF_DETECT_ORE | EF_PROBE => Some((MAP, service)),
            EF_GLYPH => Some((GLYPH, UNIT)), EF_BANISH | EF_MASS_BANISH => Some((BANISH, UNIT)),
            EF_DESTRUCTION | EF_EARTHQUAKE => Some((DESTROY, UNIT)),
            EF_BOLT | EF_BEAM | EF_BOLT_OR_BEAM | EF_BOLT_AWARE | EF_BOLT_STATUS | EF_BOLT_STATUS_DAM
                | EF_LINE | EF_BALL | EF_TOUCH | EF_TOUCH_AWARE | EF_PROJECT_LOS | EF_PROJECT_LOS_AWARE
                | EF_SPOT | EF_SPHERE | EF_BREATH | EF_ARC | EF_SHORT_BEAM | EF_SWARM | EF_STRIKE
                | EF_STAR | EF_STAR_BALL | EF_MON_TIMED_INC => match subtype as u32 {
                    PROJ_AWAY_ALL | PROJ_AWAY_EVIL | PROJ_AWAY_UNDEAD | PROJ_AWAY_SPIRIT => Some((OTHER, UNIT)),
                    PROJ_KILL_WALL => Some((DIG, UNIT)),
                    // M13 repairs the Confuse Monster wand lookup. Enable
                    // its control only for the matching wand consumer; the
                    // Confuse Monsters staff and Stun Monster wand stay out.
                    PROJ_MON_CONF if confusion => Some((CONTROL, UNIT)),
                    PROJ_MON_SLOW | PROJ_MON_HOLD | PROJ_SLEEP_ALL
                        | PROJ_SLEEP_EVIL | PROJ_TURN_ALL => Some((CONTROL, UNIT)),
                    // Native elemental damage occupies 0..=24. DARK_WEAK,
                    // terrain creation/destruction and monster-aiding types
                    // are deliberately not generic damage or buff fallbacks.
                    0..=24 | PROJ_LIGHT_WEAK | PROJ_MON_DRAIN | PROJ_MON_CRUSH
                        | PROJ_DISP_ALL | PROJ_DISP_EVIL | PROJ_DISP_UNDEAD if amount > 0 => Some((ATTACK, amount * UNIT)),
                    _ => None,
                },
            EF_WONDER => None, // no guaranteed damage for a random special handler
            _ => None,
        };
        if let Some((cap, units)) = role { *out.entry(cap).or_default() += units; }
    }
    // borg-reborn-rs (#491): the same HP also fill the healing reserve.
    // borg-reborn-rs (#487, owner): with `town_heal_reserve_real_hp` the
    // reserve counts the real HP restored; healing speed weights only the
    // healing curve (which kind is worth more), not how many potions fill
    // the reserve. Speed-weighted, a Cure Light Wounds counted about 5.6 HP
    // at 108 max HP and the reserve took dozens of them.
    if let Some(&heal) = out.get(&HEAL) {
        out.insert(HEAL_RESERVE, reserve_heal(heal, real_heal, param(Param::town_heal_reserve_real_hp) != 0));
    }
    out.into_iter().collect()
}

/// borg-reborn-rs (#649): the average damage of a thrown throwing object,
/// in `UNIT`s, as the engine computes it (`ranged_damage`,
/// `player-attack.c:554-582`): the dice's average times `2 + weight / 12`
/// (integer division, weight in tenths of a pound), and three times that
/// for an exploding object (`make_ranged_throw`, `player-attack.c:1296`).
/// A flask of oil (1d4, weight 20, `EXPLODE | THROWING`) does 22.5; the
/// planner counted 2.5, so a sling shot (5) looked twice as good.
fn thrown_damage(dd: i64, ds: i64, weight: i64, explode: bool) -> i64 {
    let dice = dd * (ds + 1) * UNIT / 2;
    let throw = dice * (2 + weight.max(0) / 12);
    (if explode { throw * 3 } else { throw }).max(UNIT)
}

fn oil_supply(quantity: i64, fuel_bottles: i64, damage: i64, fuel: i64) -> Vec<(usize, i64)> {
    let mut out = Vec::new();
    let n = quantity.max(1);
    if fuel_bottles > 0 && fuel > 0 { out.push((LIGHT, fuel * UNIT * fuel_bottles / n)); }
    if n > fuel_bottles {
        out.push((ATTACK, damage * (n - fuel_bottles) / n));
        // borg-reborn-rs (#712): each flask kept to throw, on its own curve.
        if param(Param::town_oil_apart) != 0 { out.push((THROWN_OIL, UNIT * (n - fuel_bottles) / n)); }
    }
    out
}

// Called only after native launcher compatibility and curse checks. Each
// arrow contributes both damage and one reliable shot, separate objectives.
fn missile_supply(class: i32, damage: i64) -> Vec<(usize, i64)> {
    let mut out = vec![(ATTACK, damage)];
    if missiles_percent(class, param(Param::town_missiles_percent)) > 0 {
        out.push((RANGER_MISSILES, UNIT));
        // borg-reborn-rs (#716): the same shot, counted past the soft cap.
        out.push((EXTRA_SHOTS, UNIT));
    }
    out
}

/// borg-reborn-rs (#666, owner): the reserve of shots for the class's
/// launcher, in percent of `borg_town_w_ranger_missiles`:
/// `town_missiles_percent` (per class, by shooting skill); -1 = C, the
/// Ranger's reserve only (100 for a Ranger, 0 for the others).
pub(crate) fn missiles_percent(class: i32, setting: i32) -> i64 {
    if setting < 0 { if class == CLASS_RANGER { 100 } else { 0 } } else { i64::from(setting) }
}

/// borg-reborn-rs (#666, owner): "projectile buying is more important if they
/// cannot get flasks of oil or wands". With at least `OTHER_RANGED_ENOUGH`
/// flasks of oil and attack-wand charges carried, the shots' reserve weighs
/// `town_missiles_other_ranged_percent` (100 = no change).
const OTHER_RANGED_ENOUGH: i64 = 5;
fn missiles_other_ranged(weight: i64, other_ranged: i64, percent: i32) -> i64 {
    if other_ranged >= OTHER_RANGED_ENOUGH { scale_percent(weight, percent) } else { weight }
}

/// The flasks of oil and the charges of attack wands in the pack.
fn other_ranged(st: &BorgState) -> i64 {
    let iv = &st.item_val;
    st.items.items.iter().take(crate::borg_item::inven_wield() as usize).filter(|i| i.iqty != 0).map(|i| {
        let tval = u32::from(i.tval);
        let sval = i32::from(i.sval);
        if tval == TV_FLASK && sval == iv.sv_flask_oil { i64::from(i.iqty) }
        else if tval == TV_WAND && i.aware && (sval == iv.sv_wand_magic_missile || sval == iv.sv_wand_stinking_cloud) { i64::from(i.pval).max(0) }
        else { 0 }
    }).sum()
}

/// borg-reborn-rs (#487, #520): the healing curve's scale:
/// `town_heal_curve_scale_percent`, the class's own value when it has one.
pub(crate) fn heal_curve_scale_percent(_class: i32) -> i32 {
    // borg-reborn-rs (#520): the class's own value, if any, comes through
    // `param` (town_heal_curve_scale_percent@<class> or the class file).
    param(Param::town_heal_curve_scale_percent)
}

/// borg-reborn-rs (#487, #520): the healing reserve:
/// `town_heal_reserve_hp_percent`, the class's own value when it has one
/// (0 is a reserve of none).
pub(crate) fn heal_reserve_hp_percent(_class: i32) -> i32 {
    // borg-reborn-rs (#520): the class's own value, if any, comes through
    // `param` (town_heal_reserve_hp_percent@<class> or the class file).
    param(Param::town_heal_reserve_hp_percent)
}

/// borg-reborn-rs (#443, owner 2026-11-16): consumables the Borg avoids
/// stocking, from a source audit of every kind in object.txt against the
/// core's use sites (not bookkeeping: counting, home notice, buy lists).
/// Excluded: foods (any known food is eaten, borg_eat_food_any) and potions
/// whose use-up is their benefit (stat gain, Experience, Infravision,
/// borg_junk). Harmful kinds are here too. Idle use of Treasure Detection
/// spends carried copies without assigning them stock value (#1250).
const WORTHLESS: &[(u32, &str)] = &[
    (TV_SCROLL, "Door Destruction"), (TV_SCROLL, "Dispel Undead"),
    (TV_SCROLL, "Treasure Detection"), (TV_SCROLL, "Summon Monster"), (TV_SCROLL, "Summon Undead"),
    (TV_SCROLL, "*Destruction*"), (TV_SCROLL, "Aggravate Monster"), (TV_SCROLL, "Curse Weapon"),
    (TV_SCROLL, "Curse Armour"), (TV_SCROLL, "Darkness"), (TV_SCROLL, "Trap Creation"),
    // borg-reborn-rs (#470): never read, never kept (borg_junk discards it).
    (TV_SCROLL, "Deep Descent"),
    (TV_MUSHROOM, "Second Sight"), (TV_MUSHROOM, "Emergency"), (TV_MUSHROOM, "Terror"),
    (TV_MUSHROOM, "Debility"), (TV_MUSHROOM, "Sprinting"), (TV_MUSHROOM, "Turbulence"),
    (TV_MUSHROOM, "Shadows"),
    (TV_POTION, "Poison"), (TV_POTION, "*Enlightenment*"), (TV_POTION, "Surprise"),
    (TV_POTION, "Salt Water"), (TV_POTION, "Dragon Breath"),
    (TV_WAND, "Stone to Mud"), (TV_WAND, "Stun Monster"), (TV_WAND, "Polymorph"), (TV_WAND, "Darkness"),
    (TV_WAND, "Clone Monster"), (TV_WAND, "Disable Traps"), (TV_WAND, "Dragon's Breath"),
    (TV_WAND, "Heal Monster"), (TV_WAND, "Haste Monster"),
    (TV_STAFF, "Summoning"), (TV_STAFF, "Haste Monsters"), (TV_STAFF, "Darkness"),
    (TV_STAFF, "Earthquakes"), (TV_STAFF, "Confuse Monsters"), (TV_STAFF, "Cure Light Wounds"),
    (TV_STAFF, "Starlight"),
    (TV_ROD, "Treasure Location"), (TV_ROD, "Probing"), (TV_ROD, "Restoration"),
    (TV_ROD, "Polymorph"), (TV_ROD, "Disable Traps"),
];

/// borg-reborn-rs (#1374): the kind is a Potion of *Enlightenment* (not the
/// plain Enlightenment, `item_val.sv_potion_enlightenment`): `object.txt`
/// restores and gains Intelligence and Wisdom besides mapping the level.
pub(crate) fn star_enlightenment_sval() -> i32 {
    use std::sync::OnceLock;
    static SVAL: OnceLock<i32> = OnceLock::new();
    *SVAL.get_or_init(|| engine::lookup_sval(TV_POTION as i32, "*Enlightenment*"))
}

fn is_star_enlightenment(tval: u32, sval: i32) -> bool {
    tval == TV_POTION && sval >= 0 && sval == star_enlightenment_sval()
}

/// borg-reborn-rs (#1374): the Potion of *Enlightenment* would change
/// Intelligence or Wisdom for good: one is below 18/100 (`player_stat_inc`)
/// or drained below its maximum (`RESTORE_STAT`). Otherwise its mapping and
/// detection are all it is worth, and the town's planner does not buy or keep
/// it for that (`worthless`); a dungeon level is where it is drunk.
pub(crate) fn star_enlightenment_gains() -> bool {
    let p = engine::player_notice();
    star_enlightenment_gains_at(p.stat_max.map(i32::from), p.stat_cur.map(i32::from))
}

/// `star_enlightenment_gains` at the given maximum and current stats
/// (strength .. constitution).
pub(crate) fn star_enlightenment_gains_at(max: [i32; 5], cur: [i32; 5]) -> bool {
    [1, 2].iter().any(|&s| max[s] < 18 + 100 || cur[s] < max[s])
}

/// borg-reborn-rs (#1373): `PY_MAX_EXP` (player.h): the experience the engine
/// stops granting at.
const PY_MAX_EXP: i32 = 99_999_999;

/// borg-reborn-rs (#1373): a Potion of Experience still adds experience
/// (`effect_handler_GAIN_EXP`: nothing at `PY_MAX_EXP`, whatever the level).
pub(crate) fn experience_potion_useful() -> bool {
    engine::player_notice().exp < PY_MAX_EXP
}

fn worthless(tval: u32, sval: i32) -> bool {
    use std::sync::OnceLock;
    static KINDS: OnceLock<Vec<(u32, i32)>> = OnceLock::new();
    (KINDS.get_or_init(|| WORTHLESS.iter()
        .map(|&(t, name)| (t, engine::lookup_sval(t as i32, name)))
        .filter(|&(_, s)| s >= 0).collect())
        .contains(&(tval, sval))
        // borg-reborn-rs (#1374): *Enlightenment* is drunk on purpose
        // (`borg_use_things`) while a stat can gain, so it has the value of
        // its stat gains; when both are capped and undrained it stays
        // worthless, so the planner does not pay for its mapping in town.
        && !(param(Param::item_use_star_enlightenment) != 0 && is_star_enlightenment(tval, sval)
            && star_enlightenment_gains()))
        // borg-reborn-rs (#504): the cornered mode reads Monster Confusion;
        // without it the core never does.
        || (param(Param::caution_cornered) == 0
            && tval == TV_SCROLL
            && sval >= 0
            && sval == engine::lookup_sval(TV_SCROLL as i32, "Monster Confusion"))
}

/// #1049: apply the purchase model's deliberate exclusions to known
/// items in the pack and Home as well. A temporary lack of demand for a
/// useful effect does not make it an unwanted kind.
/// borg-reborn-rs (#1371, #1375): the kinds `refused_kind` names, so a copy
/// bought before, found or identified later (a refused gain-one/lose-one
/// potion, an Unlight class's Light scroll) is sold or dropped, and cleared
/// from Home.
pub(crate) fn unwanted_home_item(st: &BorgState, item: &borg_item) -> bool {
    param(Param::town_home_reject_unwanted) != 0
        && item.kind != 0 && item.aware && item.flavour_aware
        && refused_kind(st, u32::from(item.tval), i32::from(item.sval))
}

/// borg-reborn-rs (#761, owner): what a ware supplies, with the flavour-unseen
/// shop wares (`{unseen}`: a Potion or Scroll whose flavour the player has not
/// seen) scaled by `town_unseen_percent`. Phase Door has its own
/// coefficients (`town_phase_*`) and is not scaled. 100 = today.
fn supplies(item: &borg_item, ctx: &Context, origin: (i64, usize)) -> Vec<(usize, i64)> {
    let mut out = supplies_of_kind(item, ctx, origin);
    if let Some(readiness) = ctx.readiness {
        let end = if origin.0 == PACK { origin.1 } else { crate::borg_item::inven_wield() as usize };
        let oil = if crate::borg_ranged::Need::is_oil(ctx.st, item) {
            crate::borg_ranged::Need::preceding_oil(ctx.st, end)
        } else { 0 };
        let offer = origin.0 < PACK;
        let quivered_without_launcher = origin.0 > HOME
            && matches!(u32::from(item.tval), TV_ARROW | TV_BOLT | TV_SHOT)
            && !readiness.missile(ctx.st, item);
        let uses = if quivered_without_launcher { 0 } else { readiness.item(ctx.st, item, oil, offer) };
        if uses > 0 {
            let stack = if offer { 1 } else { i64::from(item.iqty) };
            out.push((RANGED_RESERVE, i64::from(uses) * UNIT / stack));
            if ctx.launcher_reserve > 0 && matches!(u32::from(item.tval), TV_ARROW | TV_BOLT | TV_SHOT)
                && readiness.missile(ctx.st, item)
            {
                out.push((RANGER_MISSILES, UNIT));
            }
        }
    }
    let percent = param(Param::town_unseen_percent);
    if percent == 100 || item.flavour_aware || origin.0 >= 7 { return out; }
    unseen_scaled(out, percent)
}

/// borg-reborn-rs (#761): a flavour-unseen ware's supplies at `percent`,
/// Phase Door's (its own coefficients) as they are. A row that scales to zero
/// is dropped: the model keeps no zero-unit supply row (`Model::valid` asks
/// `r[3] > 0`), and a kept one stopped the town replay of 425904011 with
/// "invalid capability facts" (review of #778).
fn unseen_scaled(out: Vec<(usize, i64)>, percent: i32) -> Vec<(usize, i64)> {
    out.into_iter().map(|(cap, n)| if matches!(cap, PHASE | PHASE_RESERVE | RANGED_RESERVE | RESTORE_LIFE_RESERVE | RESTORE_STAT_RESERVE) { (cap, n) } else { (cap, scale_percent(n, percent)) }).filter(|&(_, n)| n > 0).collect()
}

/// borg-reborn-rs (#1004): the look-ahead for a book in scope `scope`: the
/// parameter for an owned book (the pack, `PACK`, or the home, `HOME`), 0 for
/// a shop's (scopes 0..=7), so purchases are valued as before.
fn book_lookahead(scope: i64, levels: i32) -> i32 {
    if matches!(scope, PACK | HOME) { levels } else { 0 }
}

/// borg-reborn-rs (#1004): does a class spell make its book worth keeping?
/// Castable now (its level reached, its mana within the max SP), as before;
/// or, with `ahead` > 0, learnt within `ahead` levels, on the level alone:
/// the mana is not compared, so a wide window can keep a book whose spell
/// costs more than the class's SP when it is learnt. Ranger and Rogue first spells arrive at CL3 and CL5.
fn book_spell_counts(slevel: i32, smana: i32, level: i32, maxsp: i32, ahead: i32) -> bool {
    (slevel <= level && smana <= maxsp) || (ahead > 0 && slevel <= level.saturating_add(ahead))
}

fn supplies_of_kind(item: &borg_item, ctx: &Context, origin: (i64, usize)) -> Vec<(usize, i64)> {
    if item.iqty == 0 || item.kind == 0 { return Vec::new(); }
    let tval = u32::from(item.tval);
    // A shop teaches its kind on purchase; carried unknown flavours/devices
    // must not exploit diagnostic hidden kind data.
    if !item.aware { return Vec::new(); }
    let st = ctx.st;
    let qty = i64::from(item.iqty).max(1);
    let mut effects = engine::kind_effect_values(item.kind as usize, ctx.journey.level as i32);
    if tval == TV_SCROLL && i32::from(item.sval) == st.item_val.sv_scroll_phase_door {
        // TELEPORT(10) is not full Teleportation. A Phase Door also counts
        // toward the reserve (#761).
        return vec![(PHASE, UNIT), (PHASE_RESERVE, UNIT)];
    }
    if tval == TV_FLASK {
        // borg-reborn-rs (#649): the engine's thrown damage, not the dice alone.
        let damage = thrown_damage(i64::from(item.dd), i64::from(item.ds), i64::from(item.weight),
            engine::flag_has(&item.flags, crate::headers::list_object_flags_h::OF_EXPLODE as i32));
        let fuel = i64::from(item.pval).max(0);
        let light = &st.items.items[(crate::borg_item::inven_wield() + 5) as usize];
        let usable_oil = i32::from(item.sval) == st.item_val.sv_flask_oil && !item.cursed;
        let needed = if ctx.lantern && fuel > 0 && (ctx.light_reserve == 0 || usable_oil) {
            ((ctx.light_goal - i64::from(light.timeout)).max(0) as u64).div_ceil(fuel as u64) as i64
        } else { 0 };
        // Allocate whole bottles to fuel before ammunition. A bottle cannot
        // supply both uses. Shop options are prospective single copies; live
        // execution buys one then refreshes the allocation. Held stacks use
        // an average per-copy vector whose whole-stack sum is conserved.
        let end = if origin.0 == PACK { origin.1 } else { crate::borg_item::inven_wield() as usize };
        let prior: i64 = st.items.items.iter().take(end).filter(|i| i.aware && u32::from(i.tval) == TV_FLASK)
            .map(|i| i64::from(i.iqty)).sum();
        let quantity = if origin.0 < PACK { 1 } else { qty };
        // borg-reborn-rs (#682): a class's oil scale weights each throw.
        let damage = damage * i64::from(param(Param::town_oil_curve_scale_percent)) / 100;
        let fuel_bottles = (needed - prior).clamp(0, quantity);
        let mut supply = oil_supply(quantity, fuel_bottles, damage, fuel);
        if ctx.readiness.is_none() && ctx.ranged_reserve > 0 && i32::from(item.sval) == st.item_val.sv_flask_oil
            && !item.cursed && fuel_bottles < quantity
        {
            supply.push((RANGED_RESERVE, (quantity - fuel_bottles) * UNIT / quantity));
        }
        return supply;
    }
    if matches!(tval, TV_ARROW | TV_BOLT | TV_SHOT) {
        if st.borg.tr(BI_AMMO_TVAL) != i32::from(item.tval) || item.cursed { return Vec::new(); }
        let damage = (crate::borg_ranged::missile_damage_twice(item, st.borg.tr(BI_BTODAM)) * UNIT / 2).max(UNIT)
            * i64::from(st.borg.tr(BI_AMMO_POWER)).max(1);
        let mut supply = missile_supply(st.borg.tr(BI_CLASS), damage);
        if ctx.readiness.is_none() && ctx.ranged_reserve > 0 { supply.push((RANGED_RESERVE, UNIT)); }
        return supply;
    }
    if tval == TV_LIGHT {
        if ctx.light_reserve > 0 && item.cursed { return Vec::new(); }
        // Supply rows are per copy. Kit and equipment quantities are applied in observe.
        return if item.timeout > 0 { vec![(LIGHT, i64::from(item.timeout) * UNIT)] } else { Vec::new() };
    }
    if tval == TV_DIGGING { return vec![(DIG, UNIT)]; }
    if engine::tval_is_book_k(item.kind as usize) {
        for b in 0..engine::class_num_books().min(9) {
            let book = engine::class_book(b);
            if book.tval != i32::from(item.tval) || book.sval != i32::from(item.sval) { continue; }
            // borg-reborn-rs (#1004): an owned book (pack or home) also counts
            // when a spell of it is learnt within town_book_lookahead_levels.
            let ahead = book_lookahead(origin.0, param(Param::town_book_lookahead_levels));
            let usable = (0..book.num_spells).map(|s| engine::class_book_spell(b, s))
                .filter(|s| book_spell_counts(s.slevel, s.smana, ctx.journey.level as i32, st.borg.tr(BI_MAXSP), ahead)).count();
            if usable > 0 { return vec![(BOOKS + b as usize, UNIT)]; }
        }
        return Vec::new();
    }
    if !matches!(tval, TV_POTION | TV_SCROLL | TV_FOOD | TV_MUSHROOM | TV_WAND | TV_ROD | TV_STAFF) { return Vec::new(); }
    // borg-reborn-rs (#443): the core has no code to use these on purpose,
    // so they are worth nothing to the planner until it does.
    // borg-reborn-rs (#1371, #1375): and what the character refuses
    // (`refused_kind`) for every origin: the planner neither buys it nor
    // keeps it. Under town_stat_trade_policy that covers the
    // gain-one/lose-one potions, learned flavour or not.
    if refused_kind(st, tval, i32::from(item.sval)) {
        return Vec::new();
    }
    // borg-reborn-rs (#1103): a stat potion that gains one stat and loses another
    // is drunk only by the classes `borg_junk` lets (`drinks_second_stat_potion`).
    if tval == TV_POTION && !stat_trade_policy() && origin.0 <= BORG_BLACK_MARKET as i64 && param(Param::town_second_stat_potions_by_class) > 0 {
        let iv = &st.item_val;
        let sval = i32::from(item.sval);
        let class = st.borg.tr(BI_CLASS);
        if [iv.sv_potion_inc_str2, iv.sv_potion_inc_int2, iv.sv_potion_inc_wis2, iv.sv_potion_inc_dex2, iv.sv_potion_inc_con2]
            .iter().enumerate().any(|(stat, &kind)| kind == sval && !drinks_second_stat_potion(class, stat))
        {
            return Vec::new();
        }
    }
    // borg-reborn-rs (food counting): only what borg_trait counts in BI_FOOD
    // feeds the core (its readiness checks). A Remove Hunger scroll counted
    // as food let the planner sell the rations the core then missed
    // ("restock food < 3"); the core seldom read it (6 of 253 bought).
    // borg-reborn-rs (#451): now borg_trait counts the scroll beside the
    // rations under this plan (with_remove_hunger) and the core reads it
    // before any food (borg_satisfy_hunger_before_food), so the scroll
    // supplies nutrition, as much as it adds to a hungry stomach. No other
    // non-food kind does: a potion's nourishment (Cure Light Wounds and the
    // like, 1) is not a meal.
    let hunger_scroll = tval == TV_SCROLL && i32::from(item.sval) == st.item_val.sv_scroll_satisfy_hunger;
    if !matches!(tval, TV_FOOD | TV_MUSHROOM) && !hunger_scroll {
        effects.retain(|&(e, _, _)| u32::from(e) != EF_NOURISH);
    }
    // Drinks that nourish almost nothing (wine, whisky, liquor, miruvor:
    // 3-5 nutrition, object.txt) are not food to the planner. Counted, the
    // food floor bought a pint of wine for its few nutrition points, then
    // stored it at the Home as not worth its pack slot, every visit.
    effects.retain(|&(e, sub, amount)| u32::from(e) != EF_NOURISH || sub == 1 || amount >= 10);
    // Poisonous/hallucinatory nourishment must not be sold as safe food.
    if effects.iter().any(|&(e, sub, _)| matches!(u32::from(e), EF_TIMED_INC | EF_TIMED_INC_NO_RES) && matches!(sub as u32, TMD_POISONED | TMD_CONFUSED | TMD_IMAGE | TMD_PARALYZED)) {
        effects.retain(|&(e, _, _)| u32::from(e) != EF_NOURISH);
    }
    // borg-reborn-rs (#1103; review of #1112, F1, F2): a Vigor or Purging mushroom
    // the Borg eats when a stat is drained (`borg_use_things`, `borg_caution`) is
    // one copy on the restore-stat reserve row while nothing is drained, if it
    // would restore a stat below 18/100. The stat rows keep their own rule (a
    // restoration counts only for a drained stat), so the reserve is never counted
    // as a permanent gain.
    let sval = i32::from(item.sval);
    let reserve_kind = ctx.restore_stat && tval == TV_MUSHROOM
        && (sval == st.item_val.sv_mush_restoring || sval == st.item_val.sv_mush_purging);
    let restore_reserve = effects.iter().any(|&(e, sub, _)| u32::from(e) == EF_RESTORE_STAT && (0..5).contains(&sub)
        && restore_reserved(reserve_kind, st.borg.tr(BI_CSTR + sub as u32)));
    effects.retain(|&(e, sub, _)| {
        if (0..5).contains(&sub) {
            if u32::from(e) == EF_RESTORE_STAT { return st.borg.tr(BI_ISFIXSTR + sub as u32) > 0; }
            if u32::from(e) == EF_GAIN_STAT { return st.borg.tr(BI_CSTR + sub as u32) < 18 + 100; }
        }
        true
    });
    for (e, subtype, amount) in &mut effects {
        if reduction_cap(*subtype as u32).is_some() {
            let timer = i64::from(engine::player_timed(*subtype as u32)).max(0);
            if u32::from(*e) == EF_CURE { *amount = timer.max(1); }
            if u32::from(*e) == EF_TIMED_DEC { *amount = (*amount).max(0).min(timer); }
        }
        if u32::from(*e) == EF_RESTORE_MANA && *amount == 0 { *amount = i64::from(st.borg.tr(BI_MAXSP)); }
    }
    let confusion = tval == TV_WAND && i32::from(item.sval) == st.item_val.sv_wand_confuse_monster;
    let mut out = effect_supply_for_consumer(&effects, ctx.journey.hp, i64::from(st.cfg(BORG_TOWN_SUPPORT_TURNS)).max(1), confusion);
    // borg-reborn-rs (#863): a copy that answers a situation is also one
    // copy on that situation's prepared row (a device: one per charge).
    if prepared_on(st) {
        for (cap, row) in PREPARED {
            if out.iter().any(|&(id, _)| id == cap) { out.push((row, UNIT)); }
        }
        // borg-reborn-rs (#1134): a Cure Critical Wounds is one copy on its own row.
        if prepared_ccw_on(st) && tval == TV_POTION && i32::from(item.sval) == st.item_val.sv_potion_cure_critical {
            out.push((PREP_CCW, UNIT));
        }
        // borg-reborn-rs (#1137): a Neutralize Poison is one copy on its own row
        // (and still one on the shared poison row, as a CCW is).
        if prepared_neutralize_on(st) && tval == TV_POTION && i32::from(item.sval) == st.item_val.sv_potion_cure_poison {
            out.push((PREP_NEUTRALIZE, UNIT));
        }
    }
    // borg-reborn-rs (#863): each healing potion is one copy on its own
    // staple row, whose knee and decay are the kind's (the three share the
    // HEAL curve).
    if staple_on(st) && tval == TV_POTION && out.iter().any(|&(id, _)| id == HEAL) {
        let sval = i32::from(item.sval);
        let row = if sval == st.item_val.sv_potion_cure_light { Some(STAPLE_CLW) }
            else if sval == st.item_val.sv_potion_cure_serious { Some(STAPLE_CSW) }
            else if sval == st.item_val.sv_potion_cure_critical { Some(STAPLE_CCW) } else { None };
        if let Some(row) = row { out.push((row, UNIT)); }
    }
    // borg-reborn-rs (#987): a healing potion is one copy on the first-cure row.
    if first_cure_count() > 0 && tval == TV_POTION && out.iter().any(|&(id, _)| id == HEAL) {
        out.push((FIRST_CURE, UNIT));
    }
    let uses = if tval == TV_WAND || tval == TV_STAFF { i64::from(item.pval).max(0) * UNIT / qty }
        else if tval == TV_ROD {
            // MINIMISE/AVERAGE are pure; the rod's physical cooldown is not a level gate.
            let turns = engine::kind_recharge_average(item.kind as usize).max(1);
            rod_uses(qty, i64::from(item.timeout), i64::from(turns), ctx.journey.turns) * UNIT / qty
        } else { UNIT };
    for (_, units) in &mut out { *units = *units * uses / UNIT; }
    // borg-reborn-rs (#666, owner): a device is worth `town_devices_percent`
    // of what it supplies (per class, by device skill; 100 = as computed).
    if matches!(tval, TV_WAND | TV_ROD | TV_STAFF) {
        for (_, units) in &mut out { *units = scale_percent(*units, param(Param::town_devices_percent)); }
        // borg-reborn-rs (#895): a learned device's charges, once per situation
        // it answers, on that situation's device row.
        if device_on(st) {
            let charges = scale_percent(uses, param(Param::town_devices_percent));
            let attack = device_attack(&effects, ctx.journey.hp, i64::from(st.cfg(BORG_TOWN_SUPPORT_TURNS)).max(1), confusion);
            let mut rows: Vec<usize> = DEVICE_ROWS.iter().filter(|&&(cap, row)| out.iter().any(|&(id, units)| id == cap && units > 0)
                && (row != DEV_ATTACK || attack)).map(|&(_, row)| row).collect();
            rows.sort_unstable(); rows.dedup();
            for row in rows { out.push((row, charges)); }
        }
    }
    out.retain(|&(_, units)| units > 0);
    if restore_reserve { out.push((RESTORE_STAT_RESERVE, UNIT)); }
    // borg-reborn-rs (#859): a Recall supply also counts toward the reserve.
    if param(Param::town_recall_curve_scale_percent) > 0 {
        if let Some(&(_, units)) = out.iter().find(|&&(id, _)| id == RECALL) { out.push((RECALL_RESERVE, units)); }
    }
    out.extend(restore_life_supply(item, st.item_val.sv_potion_restore_life, ctx.restore_life_reserve,
        origin.0, st.borg.tr(BI_GOLD), param(Param::town_restore_life_min_gold)));
    if matches!(tval, TV_WAND | TV_STAFF) && item.pval > 0 {
        let maximum = engine::kind_value_average(engine::kind(item.kind as usize).charge).max(1);
        if i64::from(item.pval) < i64::from(maximum) * qty { out.push((RECHARGE_TARGET, UNIT)); }
    }
    out
}

/// A purse floor gates only preventive buying. Owned stock (including Home)
/// still counts toward the reserve, and the experience effect is independent.
fn restore_life_supply(item: &borg_item, sval: i32, reserve: i64, scope: i64, gold: i32, min_gold: i32) -> Option<(usize, i64)> {
    (reserve > 0 && (scope > BORG_BLACK_MARKET as i64 || gold >= min_gold) && crate::borg_item_use::is_restore_life(item, sval))
        .then_some((RESTORE_LIFE_RESERVE, UNIT))
}

fn initial_curves(
    previous_count: usize, ordinary: i64, demand: i64, drained: bool, reserve: i64, restore_weight: i64,
) -> Vec<[i64; 5]> {
    // #987: the Restore Life row, not COUNT: the first-cure row after it
    // exists only with `town_first_cure` (`observe`).
    let count = if reserve > 0 { RESTORE_LIFE_RESERVE + 1 } else { previous_count };
    let mut curves: Vec<_> = (0..count).map(|id| [id as i64, ordinary, demand, 0, 0]).collect();
    curves[EXPERIENCE] = [EXPERIENCE as i64, if drained { restore_weight } else { 0 }, -UNIT, 0, 0];
    if reserve > 0 {
        curves[RESTORE_LIFE_RESERVE] = [
            RESTORE_LIFE_RESERVE as i64, reserve * restore_weight / 4, -reserve * UNIT, 0, 0,
        ];
    }
    curves
}

fn ranged_reserves(configured: i64, readiness: i64, launcher_needs_reserve: bool) -> (i64, i64) {
    let launcher = if launcher_needs_reserve { (readiness + 1) / 2 } else { 0 };
    (configured.max(readiness), launcher)
}

/// Physical supply curves use the same forecast, gold opportunity cost and
/// capacity constraints as the core town model. No consumable calls borg_power.
pub fn observe(st: &BorgState, stock: &Stock, kit: &[KitItem], journey: Journey, gold_after_urgent: i64) -> Model {
    observe_supplies(st, stock, Some(kit), journey, gold_after_urgent)
}

/// Reuse the town reserve curves without reading shop or Home stock. Index
/// the quiver by slot as well, so its copies can compete with pack surplus.
pub(crate) fn carried_supplies(st: &BorgState) -> Model {
    resupply_model(st, &Stock::default())
}

pub(crate) fn resupply_model(st: &BorgState, stock: &Stock) -> Model {
    observe_supplies(st, stock, None, Journey::of(st), i64::from(st.borg.tr(BI_GOLD)))
}

impl Model {
    fn exploration_target(&self, cap: usize) -> Option<i64> {
        // A low purse can rule out a purchase without making carried
        // Recall or a free copy from Home expendable.
        if cap == RECALL {
            let target = i64::from(param(Param::town_recall_reserve)) * UNIT;
            return (target > 0).then_some(target);
        }
        let row = self.curves.get(cap)?;
        if row[1] <= 0 { return None; }
        match cap {
            LIGHT | FOOD => Some(row[2].abs()),
            HEAL_RESERVE | PHASE_RESERVE | RANGED_RESERVE | RANGER_MISSILES if row[2] < 0 => Some(-row[2]),
            TELEPORT | LEVEL => Some(UNIT),
            _ => None,
        }
    }

    /// A whole-stack drop or a weight trim cannot spend the trip's reserves.
    pub(crate) fn keeps_exploration_supply(&self, slot: usize, copies: u8) -> bool {
        copies > 0 && self.units(PACK, slot as i64, i64::from(copies)).into_iter().any(|(cap, units)| {
            units > 0 && self.exploration_target(cap)
                .is_some_and(|target| self.curves[cap][3] - units < target)
        })
    }

    /// This ware can cover one of the missing supplies. The caller moves
    /// one unit and observes the new kit before relaxing the limit again.
    pub(crate) fn supplies_shortfall(&self, shop: u8, slot: u8) -> bool {
        self.units(i64::from(shop), i64::from(slot), 1).into_iter().any(|(cap, units)| {
            units > 0 && self.exploration_target(cap)
                .is_some_and(|target| self.curves[cap][3] < target)
        })
    }

    /// Stock and target in the same units, for one safely removable copy.
    /// Use an independent target for staples that have one, then check the
    /// shared reserves for any other effect the copy would take with it.
    pub(crate) fn discard_surplus(&self, st: &BorgState, slot: usize) -> Option<(i64, i64)> {
        let item = &st.items.items[slot];
        if item.iqty == 0 || !item.aware || !item.flavour_aware
            || item.cursed || item.art_idx != 0 || item.ego_idx != 0 || item.weight <= 0 {
            return None;
        }
        let tval = u32::from(item.tval);
        let sval = i32::from(item.sval);
        let supplies = self.units(PACK, slot as i64, 1);
        let supply = |cap| supplies.iter().find(|&&(id, _)| id == cap).map_or(0, |&(_, n)| n);
        let demand = |cap| self.curves.get(cap).map_or(0, |row: &[i64; 5]| row[2].abs());
        let pack = crate::borg_item::inven_wield() as usize;
        let quiver = crate::borg_item::quiver_start() as usize;
        let carried = || st.items.items[..pack].iter().chain(&st.items.items[quiver..])
            .filter(|it| it.aware && !it.cursed);
        let count_kind = || carried().filter(|it| it.kind == item.kind).map(|it| i64::from(it.iqty)).sum::<i64>();
        let coverage = |cap: usize| (self.curves[cap][3], demand(cap), supply(cap));
        let oil = tval == TV_FLASK && sval == st.item_val.sv_flask_oil;
        let (held, target, per_copy) = match tval {
            TV_LIGHT if sval == st.item_val.sv_light_torch && self.curves[LIGHT][1] > 0 => coverage(LIGHT),
            TV_FLASK if oil => {
                let fuel = self.supplies.iter().filter(|r| r[0] == PACK && r[2] == LIGHT as i64
                    && st.items.items[r[1] as usize].kind == item.kind)
                    .map(|r| r[3] * i64::from(st.items.items[r[1] as usize].iqty)).sum::<i64>();
                // The model spreads the fuel bottles over the stack. Keep
                // those bottles plus the independent throwing-oil target.
                let fuel = if item.pval > 0 {
                    (fuel + i64::from(item.pval) * UNIT / 2) / (i64::from(item.pval) * UNIT)
                } else { 0 };
                let staple = i64::from(param(Param::town_staple_oil));
                let target = if staple > 0 { staple } else { demand(THROWN_OIL) / UNIT };
                (count_kind(), fuel + target.max(demand(RANGED_RESERVE) / UNIT), 1)
            }
            TV_SHOT | TV_ARROW | TV_BOLT if i32::from(item.tval) == st.borg.tr(BI_AMMO_TVAL)
                && st.borg.tr(BI_AMMO_POWER) > 0 && item.ident && item.to_h == 0 && item.to_d == 0 => {
                let reserve = param(Param::town_quiver_trim_reserve).max(param(Param::town_ranger_arrows_max));
                let ammo = carried().filter(|it| it.tval == item.tval).map(|it| i64::from(it.iqty)).sum();
                (ammo, i64::from(reserve).max(demand(RANGED_RESERVE) / UNIT), 1)
            }
            TV_POTION if supply(HEAL_RESERVE) > 0 => {
                if let Some(row) = HEAL_KINDS.into_iter().find(|&row| supply(row) > 0 && demand(row) >= UNIT) {
                    coverage(row)
                } else {
                    coverage(HEAL_RESERVE)
                }
            }
            TV_SCROLL if supply(PHASE_RESERVE) > 0 => {
                let (held, target, units) = coverage(PHASE_RESERVE);
                (held, target.max(demand(STAPLE_PHASE)), units)
            }
            TV_SCROLL if supply(RECALL_RESERVE) > 0 => {
                (count_kind(), i64::from(param(Param::town_recall_reserve)).max(1), 1)
            }
            TV_SCROLL if supply(TELEPORT) > 0 || supply(LEVEL) > 0 => {
                (count_kind(), i64::from(param(Param::caution_reserve_escape_count)).max(1), 1)
            }
            TV_FOOD | TV_MUSHROOM | TV_SCROLL if supply(FOOD) > 0 => coverage(FOOD),
            _ => return None,
        };
        if target <= 0 || per_copy <= 0 || held - per_copy < target {
            return None;
        }
        if supplies.iter().any(|&(cap, units)| {
            let row = self.curves[cap];
            // Removal spends throwing bottles first, even when the model
            // attributed a fractional share of fuel to every bottle.
            !(oil && cap == LIGHT) && row[1] > 0 && row[2] < 0 && row[3] - units < -row[2]
        }) {
            return None;
        }
        Some((held - target, target))
    }
}

fn observe_supplies(st: &BorgState, stock: &Stock, kit: Option<&[KitItem]>, journey: Journey, gold_after_urgent: i64) -> Model {
    let t = |id| i64::from(st.borg.tr(id)).max(0);
    let cfg = |id| i64::from(st.cfg(id)).clamp(0, 1_000_000);
    let exposure = journey.exposure();
    let ordinary = cfg(BORG_TOWN_W_SUPPORT) * exposure;
    let risk = |id| cfg(id) * exposure;
    let support_demand = (journey.turns * UNIT / cfg(BORG_TOWN_SUPPORT_TURNS).max(1)).max(UNIT);
    let light_slot = crate::borg_item::inven_wield() as usize + 5;
    let light = st.items.items.get(light_slot).copied().unwrap_or_else(crate::borg_item::blank_item);
    let lantern = light.iqty > 0 && u32::from(light.tval) == TV_LIGHT && i32::from(light.sval) == st.item_val.sv_light_lantern
        && !engine::flag_has(&light.flags, crate::headers::list_object_flags_h::OF_NO_FUEL as i32);
    let readiness = ranged_shopping()
        .then(|| crate::borg_ranged::Need::for_depth(st, journey.depth, journey));
    let base_ranged_reserve = i64::from(param(Param::town_ranged_reserve));
    let readiness_target = readiness.map_or(0, |need| {
        i64::from(need.backup_target(need.coverage(st, None).recoverable()))
    });
    let missile_percent = missiles_percent(st.borg.tr(BI_CLASS), param(Param::town_missiles_percent));
    // In classes without a missile reserve, finish equipping a working
    // launcher with a small stack. The rest of the ranged budget can go
    // to any useful damage source.
    let (ranged_reserve, launcher_reserve) = ranged_reserves(base_ranged_reserve, readiness_target,
        missile_percent == 0 && t(BI_AMMO_POWER) > 0);
    let light_reserve = i64::from(param(Param::town_light_reserve_turns));
    // borg-reborn-rs (#1110): more light while breadth-first exploration applies.
    let light_goal = crate::borg_light::fuel_goal(st, journey);
    let restore_life_reserve = i64::from(restore_life_reserve(st));
    let restore_stat = restore_stat_on(st);
    let ctx = Context { st, journey, lantern, light_goal, light_reserve, ranged_reserve, restore_life_reserve, restore_stat, readiness, launcher_reserve };
    // Keep the historical facts shape when the reserve is disabled.
    // #859: the Recall reserve's row only exists with the Recall curve on.
    let recall_on = param(Param::town_recall_curve_scale_percent) > 0;
    let prepared = prepared_on(st);
    let staples = staple_on(st);
    let devices = device_on(st);
    let previous_count = if prepared || staples { STAPLE_LIGHT + 1 } else if recall_on {
        RECALL_RESERVE + 1
    } else if ranged_reserve > 0 { RECALL_RESERVE } else { RANGED_RESERVE };
    let curves = initial_curves(previous_count, ordinary, support_demand,
        t(BI_ISFIXEXP) > 0 || t(BI_ISFIXLEV) > 0, restore_life_reserve, cfg(BORG_TOWN_W_RESTORE) * UNIT);
    // borg-reborn-rs (#987): with `town_first_cure` the model holds every row;
    // the rows the model would not hold without it carry no weight, so on is
    // off plus the first-cure row.
    let first_cure = first_cure_count();
    let mut curves = curves;
    // borg-reborn-rs (#895): the same with the device rows (`town_device_percent`).
    // Preserve the existing row counts when the restore-stat reserve is off.
    let full = if restore_stat { RESTORE_STAT_RESERVE + 1 }
        else if prepared_ccw_on(st) || prepared_neutralize_on(st) { PREP_NEUTRALIZE + 1 }
        else if first_cure > 0 || devices { DEV_CONTROL + 1 } else { 0 };
    while curves.len() < full { curves.push([curves.len() as i64, 0, UNIT, 0, 0]); }
    let count = curves.len();
    let mut model = Model { available: stock.candidates.iter().map(|c| [i64::from(c.shop), i64::from(c.slot), i64::from(c.qty)]).collect(),
        packing: Some(crate::borg_town_packing::Packing::of(st, stock)), curves, supplies: Vec::new() };
    let mut curve = |id: usize, weight: i64, demand: i64| { model.curves[id][1] = weight; model.curves[id][2] = demand; };
    // borg-reborn-rs (#761, owner): a class's own light weight, in percent of
    // borg_town_w_light (100 = today).
    let light_demand = if light_reserve > 0 { -light_goal } else { light_goal };
    curve(LIGHT, scale_percent(cfg(BORG_TOWN_W_LIGHT) * UNIT, param(Param::town_light_percent)), light_demand * UNIT);
    // Nutrition is the game's food-value unit (food_value=100), not an equal
    // copy whitelist. One ration's nutrition (RATION_NUTRITION, 30) supports
    // food_turns actions.
    // borg-reborn-rs (food floor): never plan for fewer rations than the
    // core needs to be prepared for any level (borg_prepared "5 Food",
    // restock "food < 3"). borg-reborn-rs (#451): the core counts each
    // Remove Hunger scroll as one of the five (with_remove_hunger) and no
    // food the stores sell supplies more than the scroll's 35 nutrition (a
    // ration 30), so the planner's floor of 150 (five rations) is never met
    // with fewer than five of them, the core's five.
    // Sized to the trip alone, a short early trip
    // asked for about one ration, so the planner sold the starting rations
    // and the core then bounced between DL1 and town for "restock food".
    // Below the floor, food is a requirement (negative demand: worth its
    // weight linearly up to five rations, nothing past them), so it is
    // bought up to the floor and never sold below it, but not hoarded.
    // borg-reborn-rs (#761, owner): a class's own food weight, in percent of
    // borg_town_w_food (100 = today).
    let w_food = scale_percent(cfg(BORG_TOWN_W_FOOD), param(Param::town_food_percent));
    // borg-reborn-rs (#1110): more food while breadth-first exploration
    // applies: the trip's rations and the five-ration floor alike.
    let food_percent = crate::borg_flow_dark::bfs_supply_percent(st, false);
    let (food_weight, food_demand) =
        food_curve(w_food, journey.turns * UNIT / cfg(BORG_TOWN_FOOD_TURNS).max(1), food_percent);
    curve(FOOD, food_weight, food_demand);
    // borg-reborn-rs (#491, owner: "we can never have enough healing").
    // `town_heal_curve_scale_percent` stretches the diminishing healing curve:
    // weight and demand scale together, so its first HP is worth the same
    // (slope weight / demand = w_bleeding / max HP) and it reaches half its
    // value at scale x today's demand (100 = today).
    // #487 (owner): a class may override the scale and the reserve.
    let class = st.borg.tr(BI_CLASS);
    let scale = i64::from(heal_curve_scale_percent(class));
    curve(HEAL, risk(BORG_TOWN_W_BLEEDING) * scale / 100, journey.hp * exposure * scale / 100);
    // Up to `town_heal_reserve_hp_percent` of max HP of healing, each HP is
    // also worth that first-HP value, independent of the trip's exposure, on
    // top of the curve. A requirement (negative demand, like food's floor):
    // linear up to the reserve, nothing more past it, so healing is bought
    // first and never sold below it (0 = off).
    // borg-reborn-rs (#840 remedy b): at least the kit's cure reserve, in HP.
    let reserve = (journey.hp * i64::from(heal_reserve_hp_percent(class)) * UNIT / 100)
        .max(i64::from(crate::borg_kit::start_cure_reserve_hp()) * UNIT);
    curve(HEAL_RESERVE, if reserve > 0 { cfg(BORG_TOWN_W_BLEEDING) * reserve / journey.hp.max(1) } else { 0 }, -reserve.max(UNIT));
    curve(WOUNDS, 0, UNIT); // historical combined status ID is retained for replay
    for (cap, timed) in [(CUT, TMD_CUT), (STUN, TMD_STUN)] {
        let timer = i64::from(engine::player_timed(timed)).max(0);
        curve(cap, if timer > 0 { cfg(BORG_TOWN_W_BLEEDING) * UNIT } else { 0 }, -(timer * UNIT).max(UNIT));
    }
    curve(CONTROL, ordinary, exposure.max(UNIT));
    curve(POISON, if t(BI_ISPOISONED) > 0 { cfg(BORG_TOWN_W_POISON) * UNIT } else { 0 },
        -(i64::from(engine::player_timed(TMD_POISONED)).max(1) * UNIT));
    // borg-reborn-rs (#682, owner): Phase Door's curve stretched per class, as
    // healing's: weight and demand scale together (100 = today).
    let phase_scale = i64::from(param(Param::town_phase_curve_scale_percent));
    curve(PHASE, risk(BORG_TOWN_W_ESCAPE) * phase_scale / 100, exposure * phase_scale / 100);
    // borg-reborn-rs (#761): the first `town_phase_reserve` Phase Doors are a
    // requirement (as Recall's reserve): each worth `town_phase_reserve_percent`
    // of the trip's escape weight, nothing past them (0 = off, today).
    // borg-reborn-rs (#840 remedy b): at least the kit's Phase Door reserve.
    let phase_reserve = i64::from(param(Param::town_phase_reserve).max(crate::borg_kit::start_phase_reserve())).max(0);
    if phase_reserve > 0 {
        curve(PHASE_RESERVE, scale_percent(risk(BORG_TOWN_W_ESCAPE), param(Param::town_phase_reserve_percent)) * phase_reserve,
            -(phase_reserve * UNIT));
    } else {
        curve(PHASE_RESERVE, 0, UNIT);
    }
    if ranged_reserve > 0 {
        // Oil kept to throw and matching missiles fill the same requirement.
        let unit_weight = risk(BORG_TOWN_W_ESCAPE).max(risk(BORG_TOWN_W_ATTACK)).max(UNIT);
        curve(RANGED_RESERVE, unit_weight * ranged_reserve, -(ranged_reserve * UNIT));
    }
    // borg-reborn-rs (#557): `town_recall_reserve` scrolls are a requirement
    // (worth the full trip weight each, nothing past them) at any depth: the
    // walk it saves is always worth less than the full weight (the journey
    // includes the exploring), so a shallow Borg otherwise never buys one and
    // walks home, and the walk can fail (priest-1055640847).
    // borg-reborn-rs (#557, owner): `town_recall_value_percent` scales what
    // the walk a scroll saves is worth (100 = as computed).
    let walk_value = recall_value_scaled(cfg(BORG_TOWN_W_TRIP) * UNIT * journey.walk / journey.turns.max(journey.walk).max(1));
    let recall_reserve = recall_reserve_for_budget(gold_after_urgent);
    // borg-reborn-rs (#859, owner: "use the trip weight"):
    // `town_recall_curve_scale_percent` on (> 0) values every scroll on a
    // diminishing curve on the trip weight, never zero, with the reserve a
    // requirement on top of it (RECALL_RESERVE, as Phase Door's). 0 = #557.
    // With the curve on, the trip weight grows with the trip's exposure, as
    // healing's and the escapes' do (#859): deeper and longer, a scroll is
    // worth more, never less than before.
    let recall_scale = param(Param::town_recall_curve_scale_percent);
    let trip_weight = if recall_scale > 0 { cfg(BORG_TOWN_W_TRIP) * exposure.max(UNIT) } else { cfg(BORG_TOWN_W_TRIP) * UNIT };
    let ((weight, demand), reserve_row) = recall_curve(trip_weight, walk_value, recall_reserve, recall_scale);
    curve(RECALL, weight, demand);
    if let Some((weight, demand)) = reserve_row { curve(RECALL_RESERVE, weight, demand); }
    // borg-reborn-rs (#863, owner: "the point is being prepared to a variety of
    // situations"): below the Recall curve's reserve row, one small
    // diminishing row per situation's remedy, on the trip weight: the first
    // copy of each is worth more than the 21st of a volume kind, past a
    // couple of copies each is worth little. The condition-present curves
    // (poisoned, cursed, ...) stay as they are on top. 0 = off.
    if count > RECALL_RESERVE && !recall_on { curve(RECALL_RESERVE, 0, UNIT); }
    if prepared {
        // borg-reborn-rs (#863, owner: "if I have 20 CLW I would have say 4
        // detect invisible and 2 remove curse"): each remedy's own copies.
        let trip_weight = cfg(BORG_TOWN_W_TRIP) * exposure.max(UNIT);
        let percent = param(Param::town_prepared_percent);
        // borg-reborn-rs (#1134): the poison remedy and the CCW row may have a
        // percent of their own (0 = the shared one): a row prices a copy, not
        // a gold piece, so at the shared percent a 15 gold scroll beats a 75
        // gold Neutralize Poison and a 200 gold Cure Critical Wounds.
        // borg-reborn-rs (#1137, owner: "when he is rich enough to get a WoR he
        // starts buying antipoison and CCW"): with `town_prepared_dear_after_recall`
        // the dear rows count only once the purse (after the urgent needs)
        // reaches the Recall floor (`buy_recall_from_gold`) and the Recall
        // reserve (`town_recall_reserve`, at least one) is carried; before,
        // they are worth nothing and an own percent falls back to the shared
        // one, so the gold goes to Recall and the staples first. Recall's own
        // gate, so a CCW at 112 percent of the trip weight never outbids a
        // missing reserve scroll (100 percent). 0 = no gate.
        let dear = dear_rows_paid(param(Param::town_prepared_dear_after_recall) > 0,
            gold_after_urgent, i64::from(crate::borg_store_buy::recall_purchase_floor()),
            t(BI_RECALL), i64::from(param(Param::town_recall_reserve)));
        let own = |p: Param| { let v = param(p); if v > 0 && dear { v } else { percent } };
        for (row, copies, row_percent) in [(PREP_POISON, Param::town_prepared_poison, own(Param::town_prepared_poison_percent)),
            (PREP_CURSE, Param::town_prepared_curse, percent),
            (PREP_INVISIBLE, Param::town_prepared_invisible, percent), (PREP_CONFUSION, Param::town_prepared_confusion, percent),
            (PREP_FEAR, Param::town_prepared_fear, percent)] {
            let (weight, demand) = prepared_row(trip_weight, row_percent, param(copies));
            curve(row, weight, demand);
        }
        if count > PREP_NEUTRALIZE {
            for (row, copies, row_percent) in [(PREP_CCW, Param::town_prepared_ccw, Param::town_prepared_ccw_percent),
                (PREP_NEUTRALIZE, Param::town_prepared_neutralize, Param::town_prepared_neutralize_percent)] {
                let copies = param(copies);
                let (weight, demand) = if copies > 0 && dear {
                    prepared_row(trip_weight, own(row_percent), copies)
                } else { (0, UNIT) };
                curve(row, weight, demand);
            }
        }
    }
    // borg-reborn-rs (#895, owner: "we are going to need space for other
    // devices"): the device rows, in charges, as requirements on the trip weight.
    let device_floor = i64::from(param(Param::town_device_escape_min_gold));
    if devices {
        let trip_weight = cfg(BORG_TOWN_W_TRIP) * exposure.max(UNIT);
        let percent = param(Param::town_device_percent);
        for (row, charges) in [(DEV_ATTACK, Param::town_device_attack), (DEV_ESCAPE, Param::town_device_escape),
            (DEV_DETECT, Param::town_device_detect), (DEV_CONTROL, Param::town_device_control)] {
            let (weight, demand) = device_row(trip_weight, percent, param(charges));
            curve(row, weight, demand);
        }
    } else if count > DEV_CONTROL {
        for row in [DEV_ATTACK, DEV_ESCAPE, DEV_DETECT, DEV_CONTROL] { curve(row, 0, UNIT); }
    }
    // borg-reborn-rs (#1103; review of #1112, F1): the restore-stat reserve, one
    // copy worth `town_restore_stat_percent` of one stat's restore weight.
    if restore_stat {
        curve(RESTORE_STAT_RESERVE, scale_percent(cfg(BORG_TOWN_W_RESTORE) * UNIT, param(Param::town_restore_stat_percent)), -UNIT);
    }
    // borg-reborn-rs (#863, owner: "make staple objects start losing value
    // from [a few] copies, compared to the objects that are not in the
    // starter's shopping list"; "it must be a decaying curve"; "Food and Light
    // start decaying from 4, phase door, flasks and CLW from 6, CSW from 4, CCW
    // 2, but again it might be class dependent"): each staple has its own
    // free copies (`town_staple_*`, 0 = no decay for it, per class with
    // @<class>) and all share `town_staple_decay_percent`. A healing copy is
    // the potion's HP (object.txt: Cure Light Wounds 20, Serious 40, Critical
    // 60) in the HEAL curve's speed-weighted units, a food copy a Ration
    // (RATION_NUTRITION), a light copy `borg_town_light_turns`. With a lantern
    // spare torches are not light, so the light row stays off.
    if staples {
        let decay = i64::from(param(Param::town_staple_decay_percent));
        let speed = i64::from(param(Param::town_heal_speed_percent));
        let free = |p: Param| i64::from(param(p));
        let row = |copies: i64, units: i64| if copies > 0 { (knee_row(decay, units), -(copies * units).max(1)) } else { (0, UNIT) };
        let kinds = [(STAPLE_CLW, free(Param::town_staple_clw), heal_units(20, journey.hp, speed)),
            (STAPLE_CSW, free(Param::town_staple_csw), heal_units(40, journey.hp, speed)),
            (STAPLE_CCW, free(Param::town_staple_ccw), heal_units(60, journey.hp, speed))];
        for (id, copies, heal) in kinds {
            // The row counts potions (UNIT each); its knee is in those units.
            let (w, _) = row(copies, heal);
            curve(id, w, if copies > 0 { -(copies * UNIT) } else { UNIT });
        }
        for (id, copies, units) in [(STAPLE_PHASE, free(Param::town_staple_phase), UNIT),
            (STAPLE_OIL, free(Param::town_staple_oil), UNIT),
            (STAPLE_FOOD, free(Param::town_staple_food), RATION_NUTRITION * UNIT),
            (STAPLE_LIGHT, if lantern { 0 } else { free(Param::town_staple_light) }, cfg(BORG_TOWN_LIGHT_TURNS).max(1) * UNIT)] {
            let (w, d) = row(copies, units);
            curve(id, w, d);
        }
    } else if count > STAPLE_LIGHT {
        // Unset rows hold the default weight and demand: a knee must be off.
        for id in HEAL_KINDS.into_iter().chain(STAPLES.iter().map(|&(_, row)| row)) { curve(id, 0, UNIT); }
    }
    // borg-reborn-rs (#987): the first `town_first_cure` healing potions, each
    // worth `town_first_cure_percent` of the trip's escape weight (the Phase
    // Door reserve's unit), nothing past them.
    if first_cure > 0 {
        curve(FIRST_CURE, scale_percent(risk(BORG_TOWN_W_ESCAPE), param(Param::town_first_cure_percent)) * first_cure,
            -(first_cure * UNIT));
    }
    let damage_scale = journey.level * 2 + 4;
    curve(ATTACK, risk(BORG_TOWN_W_ATTACK), damage_scale * journey.turns * UNIT / cfg(BORG_TOWN_ATTACK_TURNS).max(1));
    // Reliable bow ammunition is a separate reserve for Rangers. Oil, devices
    // and spells supply damage but cannot replenish usable missile shots.
    // borg-reborn-rs (#666): every class by its own percent (shooting skill),
    // less when it carries flasks or attack-wand charges.
    let missiles_weight = scale_percent(risk(BORG_TOWN_W_RANGER_MISSILES), missile_percent as i32);
    let missiles_weight = missiles_other_ranged(missiles_weight, other_ranged(st), param(Param::town_missiles_other_ranged_percent));
    // borg-reborn-rs (#716, owner: "they do not need that many arrows"): with
    // `town_ranger_arrows_max` the reserve is a requirement, as Recall's: each
    // arrow up to the cap is worth the same, past it nothing. C's curve grows
    // with the trip and never stops paying (half value at turns / missile
    // turns), so 1-gold arrows piled up to 150-200 and their weight sent
    // Phase Door to the Home. 0 = C.
    // Past the cap (owner: "a soft cap at 50 and then a curve for additional
    // arrows utility", "a curve coefficient like we defined for healing ...
    // the more he carries the less value they have"), shots count on
    // EXTRA_SHOTS: C's shot curve (the reserve's weight, the trip's demand)
    // with weight and demand scaled by `town_ranger_arrows_curve_scale_percent`.
    let arrows_max = i64::from(param(Param::town_ranger_arrows_max));
    let arrows_scale = i64::from(param(Param::town_ranger_arrows_curve_scale_percent));
    let c_demand = (journey.turns * UNIT / cfg(BORG_TOWN_RANGER_MISSILE_TURNS).max(1)).max(UNIT);
    if launcher_reserve > 0 {
        curve(RANGER_MISSILES, risk(BORG_TOWN_W_RANGER_MISSILES) * launcher_reserve,
            -(launcher_reserve * UNIT));
    } else if arrows_max > 0 {
        curve(RANGER_MISSILES, missiles_weight, -(arrows_max * UNIT));
    } else {
        curve(RANGER_MISSILES, missiles_weight, c_demand);
    }
    let extra_on = arrows_max > 0 && arrows_scale > 0;
    curve(EXTRA_SHOTS, if extra_on { missiles_weight * arrows_scale / 100 } else { 0 },
          if extra_on { (c_demand * arrows_scale / 100).max(1) } else { UNIT });
    // borg-reborn-rs (#712): flasks kept to throw, on a curve of their own
    // that shots and wands cannot fill: a stack of sling shots saturated
    // ATTACK, so the flasks' marginal damage was worth almost nothing and the
    // weight repair stored them at the Home. Weighted as ATTACK (and by the
    // class's oil scale, #682); half its value at `town_oil_flasks` flasks.
    // 0 weight (C) unless `town_oil_apart`.
    let thrown_weight = if param(Param::town_oil_apart) != 0 {
        scale_percent(risk(BORG_TOWN_W_ATTACK), param(Param::town_oil_curve_scale_percent))
    } else { 0 };
    curve(THROWN_OIL, thrown_weight, i64::from(param(Param::town_oil_flasks)).max(1) * UNIT);
    // borg-reborn-rs (#452): Speed is an escape-like reserve, per potion
    // (see effect_supply_linear), not a buff against the trip's turns.
    for id in [TELEPORT, LEVEL, OTHER, SPEED] { curve(id, risk(BORG_TOWN_W_ESCAPE), exposure); }
    for cap in [TRAPS, MONSTERS] { curve(cap, ordinary, journey.turns * UNIT); }
    // borg-reborn-rs (#663, owner): Magic Mapping, Detect Invisible and Light
    // are "something you never buy": a read counted as borg_town_support_turns
    // of service, so each was worth about a Phase Door and they took a quarter
    // of the gold. A carried or found copy keeps a small value (kept, not sold).
    for cap in [MAP, ILLUMINATE] { curve(cap, low_value(ordinary), journey.turns * UNIT); }
    // borg-reborn-rs (#831, owner: "Scrolls of Detect Invisible are not so
    // worthless after all ... especially for Blackguards and those who do not
    // have a spell to detect invisible"; "3 is already too big a reserve ...
    // no reserve, better value and a curve that depends on class"): with
    // `town_invisible_count` > 0 the reads are worth a diminishing curve, as
    // Phase Door's and the flasks', not a requirement: `invisible_percent` of the trip's escape weight is the
    // curve's first value per read (a read's realised first value is that times
    // half / (half + 1)), the curve
    // reaches half its value at `town_invisible_count` reads, and it
    // competes with the rest of the purse (nothing is bought first, nothing
    // is kept past what the next purchase is worth). 0 = #663's low value.
    let (invisible_weight, invisible_demand) = invisible_curve(
        i64::from(param(Param::town_invisible_count)),
        scale_percent(risk(BORG_TOWN_W_ESCAPE), invisible_percent(st)),
        cfg(BORG_TOWN_SUPPORT_TURNS).max(1) * UNIT,
    ).unwrap_or((low_value(ordinary), journey.turns * UNIT));
    curve(INVISIBLE, invisible_weight, invisible_demand);
    curve(MANA, if t(BI_MAXSP) > 0 { risk(BORG_TOWN_W_MAGIC) } else { 0 }, t(BI_MAXSP).max(1) * exposure);
    curve(IDENTIFY, if t(BI_ALL_NEED_ID) > 0 { cfg(BORG_TOWN_W_KNOWLEDGE) * UNIT } else { 0 }, -UNIT);
    // borg-reborn-rs (#1373): with the intentional drink, an Experience potion
    // is worth nothing once the engine grants none (`PY_MAX_EXP`).
    let exp_useful = param(Param::item_use_drink_experience) == 0 || experience_potion_useful();
    curve(GAIN_EXPERIENCE, if exp_useful { cfg(BORG_TOWN_W_RESTORE) * UNIT } else { 0 }, -UNIT);
    curve(ACQUIRE, cfg(BORG_TOWN_W_POWER) * exposure, journey.advancement * UNIT);
    // borg-reborn-rs (#487, owner): a digger only clears rubble (the engine
    // swaps it in; a weapon digs rubble too, slower). Below
    // `town_digger_min_gold` gold no digging is valued, so the first visits
    // do not spend a third to half of the purse on a Shovel (0 = off).
    if i64::from(st.borg.tr(BI_GOLD)) < i64::from(param(Param::town_digger_min_gold)) {
        curve(DIG, 0, support_demand);
    }
    curve(CURSE, if t(BI_FIRST_CURSED) > 0 { cfg(BORG_TOWN_W_RESTORE) * UNIT } else { 0 }, -UNIT);
    curve(ENCHANT, if t(BI_NEED_ENCHANT_TO_A) + t(BI_NEED_ENCHANT_TO_H) + t(BI_NEED_ENCHANT_TO_D) > 0 { cfg(BORG_TOWN_W_SUPPORT) * UNIT } else { 0 }, UNIT);
    curve(RECHARGE, cfg(BORG_TOWN_W_SUPPORT) * UNIT, -UNIT);
    curve(RECHARGE_TARGET, 0, UNIT);
    for (cap, resistance) in [(FIRE, BI_RFIRE), (COLD, BI_RCOLD), (RESPOIS, BI_RPOIS), (ACID, BI_RACID), (ELEC, BI_RELEC)] {
        // Temporary resistance still stacks with permanent resistance in combat;
        // halve its priority rather than deleting its capability.
        curve(cap, ordinary / if t(resistance) > 0 { 2 } else { 1 }, journey.turns * UNIT);
    }
    // borg-reborn-rs (#452): SPEED left this loop (the escape reserve above).
    curve(BUFF, ordinary, journey.turns * UNIT);
    // borg-reborn-rs (#516): not afraid, fear cures (a Potion of Boldness)
    // are worth only up to town_fear_cure_count in the pack (0 = the curve).
    let fear_cap = i64::from(param(Param::town_fear_cure_count));
    // borg-reborn-rs (#837, owner): while not confused, the first
    // `town_confusion_cure_reserve` confusion cures are a requirement (as
    // Recall's reserve): a full cure (CURE:CONFUSED: Cure Serious or Critical
    // Wounds, Clear Mind, a Curing staff) is one unit. Not confused, the timer
    // is 0, so a Cure Light Wounds (TIMED_DEC, clamped to the timer in
    // `item_supplies`) supplies none (review of #843, F1).
    let confusion_reserve = i64::from(param(Param::town_confusion_cure_reserve)).max(0);
    for (cap, condition) in [(CONFUSION, BI_ISCONFUSED), (BLIND, BI_ISBLIND), (FEAR, BI_ISAFRAID)] {
        if cap == CONFUSION && confusion_reserve > 0 && t(condition) == 0 {
            curve(cap, scale_percent(cfg(BORG_TOWN_W_RESTORE) * UNIT, param(Param::town_confusion_cure_reserve_percent)) * confusion_reserve,
                -(confusion_reserve * UNIT));
            continue;
        }
        // borg-reborn-rs (#663, owner): a fear cure (Boldness) is worth very
        // little while not afraid.
        let calm = if cap == FEAR { low_value(ordinary) } else { ordinary };
        curve(cap, if t(condition) > 0 { cfg(BORG_TOWN_W_RESTORE) * UNIT } else { calm }, if t(condition) > 0 {
            let timed = match cap { CONFUSION => TMD_CONFUSED, BLIND => TMD_BLIND, _ => TMD_AFRAID };
            -(i64::from(engine::player_timed(timed)).max(1) * UNIT)
        } else if cap == FEAR && fear_cap > 0 { -fear_cap * UNIT } else { exposure });
    }
    for stat in 0..5 {
        let drained = t(BI_ISFIXSTR + stat as u32) > 0;
        curve(STATS + stat, if drained || t(BI_CSTR + stat as u32) < 18 + 100 { cfg(BORG_TOWN_W_RESTORE) * UNIT } else { 0 }, -UNIT);
    }
    for cap in BOOKS..BOOKS + 9 { curve(cap, cfg(BORG_TOWN_W_MAGIC) * UNIT, -UNIT); }
    // Per-item physical supply facts are independent of the quantities held.
    // borg-reborn-rs (#895, owner: a Staff of Teleportation only with spare gold;
    // review of #898, F10): below the gold floor a Staff of Teleportation on offer
    // supplies nothing, neither its device row nor the shared TELEPORT curve, so a
    // poor Borg does not buy one at all; one already carried is never touched
    // (its supplies come from the kit below), so it is never sold for a small purse.
    let staff_gated = devices && escape_gated(device_floor, gold_after_urgent, carried_escape_charges(st));
    for c in &stock.candidates {
        let item = &st.store.shops[usize::from(c.shop)].ware[usize::from(c.slot)];
        let mut supply = supplies(item, &ctx, (i64::from(c.shop), usize::from(c.slot)));
        if staff_gated && u32::from(item.tval) == TV_STAFF && i32::from(item.sval) == st.item_val.sv_staff_teleportation {
            supply = without_escape(supply);
        }
        for (cap, n) in supply {
            model.supplies.push([i64::from(c.shop), i64::from(c.slot), cap as i64, n]);
        }
    }
    for it in kit.into_iter().flatten() {
        let item = if it.place == Place::Pack { &st.items.items[usize::from(it.slot)] }
            else { &st.store.shops[crate::borg_store::BORG_HOME].ware[usize::from(it.slot)] };
        for (cap, n) in supplies(item, &ctx, (Model::scope(it), usize::from(it.slot))) {
            model.supplies.push([Model::scope(it), i64::from(it.slot), cap as i64, n]);
            model.curves[cap][if it.place == Place::Pack { 3 } else { 4 }] += n * i64::from(it.qty);
        }
    }
    let pack_end = crate::borg_item::inven_wield() as usize;
    let quiver = crate::borg_item::quiver_start() as usize;
    let start = if kit.is_some() { pack_end } else { 0 };
    for (slot, item) in st.items.items.iter().enumerate().skip(start).filter(|(_, i)| i.iqty > 0) {
        let origin = if slot < pack_end { PACK } else { HOME + 1 };
        for (cap, n) in supplies(item, &ctx, (origin, slot)) {
            model.curves[cap][3] += n * i64::from(item.iqty);
            if kit.is_none() && (slot < pack_end || slot >= quiver) {
                model.supplies.push([PACK, slot as i64, cap as i64, n]);
            }
        }
    }
    use crate::headers::list_object_flags_h::OF_NO_FUEL;
    let fuel_free = engine::player_has(crate::headers::list_player_flags_h::PF_UNLIGHT)
        || light.iqty > 0 && engine::flag_has(&light.flags, OF_NO_FUEL as i32)
        || t(BI_LIGHT) > 0 && light.iqty == 0;
    if fuel_free {
        model.curves[LIGHT][1] = 0;
    }
    // borg-reborn-rs (#380, #400): a learned spell never zeroes a physical
    // item's value. Mana is finite and shared, a spell can fail or be
    // unusable (blind, confused, out of mana), and broad capability classes
    // conflate different effects (treasure sensing is not mapping, Detect
    // Monsters misses invisible races). Scrolls, devices and potions keep
    // their supply value next to the spells.
    let csw_weight = scale_percent(cfg(BORG_TOWN_W_RESTORE) * UNIT, param(Param::town_csw_reserve_percent));
    if csw_weight > 0 && t(BI_ISCONFUSED) == 0 {
        let food_floor = scale_percent(READY_FOOD * RATION_NUTRITION * UNIT, food_percent);
        // An urgent Recall or a Recall that can be bought within its normal
        // budget comes before this optional purchase.
        let recall_due = crate::borg_town_utility::recall_urgent(st)
            || recall_on && t(BI_RECALL) < recall_reserve && stock.candidates.iter().any(|c| {
                c.qty > 0 && c.price > 0
                    && crate::borg_town_budget::is_recall_scroll(st, c.tval, c.sval)
            });
        let required_fuel = if fuel_free { 0 } else { light_goal };
        if csw_staples_covered(&model, required_fuel, food_floor, phase_reserve) && !recall_due {
            add_csw_reserve(&mut model, stock, st.item_val.sv_potion_cure_serious, csw_weight);
        }
    }
    debug_assert!(model.valid());
    model
}

fn csw_staples_covered(model: &Model, light_goal: i64, food_floor: i64, phase_reserve: i64) -> bool {
    model.curves[LIGHT][3] >= light_goal * UNIT
        && model.curves[FOOD][3] >= food_floor
        && model.curves[PHASE][3] >= phase_reserve.max(1) * UNIT
}

/// Credit full cures in the pack, and give a shopping bonus to CSW alone.
fn add_csw_reserve(model: &mut Model, stock: &Stock, csw_sval: i32, weight: i64) {
    let carried: i64 = model.curves[CONFUSION][3];
    while model.curves.len() < CSW_RESERVE {
        model.curves.push([model.curves.len() as i64, 0, UNIT, 0, 0]);
    }
    model.curves.push([CSW_RESERVE as i64, weight, -2 * UNIT, carried, 0]);
    let pack_cures: Vec<_> = model.supplies.iter()
        .filter(|row| row[0] == PACK && row[2] == CONFUSION as i64)
        .map(|row| [PACK, row[1], CSW_RESERVE as i64, row[3]])
        .collect();
    model.supplies.extend(pack_cures);
    for c in &stock.candidates {
        if c.qty > 0 && c.price > 0 && u32::from(c.tval) == TV_POTION && i32::from(c.sval) == csw_sval {
            let units = model.units(i64::from(c.shop), i64::from(c.slot), 1);
            if let Some((_, supply)) = units.into_iter().find(|&(cap, _)| cap == CONFUSION) {
                model.supplies.push([i64::from(c.shop), i64::from(c.slot), CSW_RESERVE as i64, supply]);
            }
        }
    }
}

/// Optional engine-backed coverage diagnostic. These are hypothetical KNOWN
/// single items, never live inventory or receipts. It runs once per process,
/// after character birth, and mutates no player/Borg/game-RNG state.
pub fn audit(st: &BorgState, journey: Journey, model: &Model) {
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::io::Write;
    static DONE: AtomicBool = AtomicBool::new(false);
    let Ok(path) = std::env::var("BORG_TOWN_CAPABILITY_AUDIT") else { return; };
    if DONE.swap(true, Ordering::Relaxed) { return; }
    let Ok(mut file) = std::fs::File::create(path) else { return; };
    let light_slot = crate::borg_item::inven_wield() as usize + 5;
    let light = st.items.items.get(light_slot).copied().unwrap_or_else(crate::borg_item::blank_item);
    let light_reserve = i64::from(param(Param::town_light_reserve_turns));
    let ctx = Context { st, journey, restore_life_reserve: i64::from(restore_life_reserve(st)), restore_stat: restore_stat_on(st), light_goal: crate::borg_light::fuel_goal(st, journey), light_reserve, ranged_reserve: i64::from(param(Param::town_ranged_reserve)), readiness: None, launcher_reserve: 0,
        lantern: light.iqty > 0 && i32::from(light.sval) == st.item_val.sv_light_lantern
        && !engine::flag_has(&light.flags, crate::headers::list_object_flags_h::OF_NO_FUEL as i32) };
    let _ = writeln!(file, "{{\"hypothetical_known_items\":1,\"class\":{},\"level\":{},\"curves\":{:?}}}", st.borg.tr(BI_CLASS), journey.level, model.curves);
    for kind in 1..usize::from(engine::z_info().k_max) {
        let k = engine::kind(kind);
        if k.name.is_null() { continue; }
        let mut item = crate::borg_item::blank_item();
        item.kind = kind as u32;
        item.tval = k.tval as u8; item.sval = k.sval as u8;
        item.iqty = 1; item.aware = true; item.flavour_aware = true;
        item.dd = k.dd as u8; item.ds = k.ds as u8;
        item.pval = engine::kind_value_average(k.pval) as i16;
        item.flags = k.flags;
        if matches!(k.tval as u32, TV_WAND | TV_STAFF) { item.pval = engine::kind_value_average(k.charge) as i16; }
        if k.tval as u32 == TV_LIGHT {
            item.timeout = if k.sval == st.item_val.sv_light_torch { engine::z_info().fuel_torch } else { engine::z_info().fuel_lamp } as i16;
        }
        let units = supplies(&item, &ctx, (0, 0));
        let units_json = units.iter().map(|(id, n)| format!("[{id},{n}]")).collect::<Vec<_>>().join(",");
        let _ = writeln!(file, "{{\"kind\":{},\"name\":{:?},\"tval\":{},\"sval\":{},\"supplies\":[{}],\"gain\":{},\"gear\":{}}}",
            kind, String::from_utf8_lossy(engine::kind_name(kind)), k.tval, k.sval, units_json,
            model.gain(&units), i32::from(crate::borg_inventory::borg_wield_slot(&item) >= 0));
    }
}

/// Merge intersecting capabilities, so substitute devices/consumables do
/// not each claim the same shortage. Preserve pure options plus a bounded
/// deterministic mixed path; the surrogate is not a global exact optimum.
pub fn shared_groups(groups: Vec<crate::borg_town_solve::Group>, input: &crate::borg_town_record::StepInput,
                     candidates: &[crate::borg_town_stock::Candidate], model: &Model, limits: &crate::borg_town_solve::Limits,
                     economics: (i64, i64, i64)) -> Vec<crate::borg_town_solve::Group> {
    let (space, carry, knowledge) = economics;
    let market_budget = crate::borg_town_budget::black_market_budget(limits.gold, input.black_market_reserve, input.black_market_percentage, input.params[9]);
    use crate::borg_town_solve::{Choice, Group, GroupKey};
    let facts: BTreeMap<_, _> = input.facts.iter().map(|f| ((f.shop, f.slot), f)).collect();
    let mut out = Vec::new();
    let mut gear = BTreeMap::<usize,i64>::new();
    if input.revision >= 9 {
        for g in groups.iter().filter(|g| matches!(g.key, GroupKey::Slot(_))) {
            for o in g.options.iter().skip(1) {
                for &(i,n) in &o.picks {
                    let c = &candidates[i];
                    let learn = if facts.get(&(c.shop,c.slot)).is_some_and(|f| f.serves & (1 << 5) != 0) { knowledge * UNIT } else { 0 };
                    gear.insert(i, o.value - model.ware(c.shop,c.slot,n) - learn);
                }
            }
        }
    }
    let mut sets: Vec<(u128, Vec<Group>)> = Vec::new();
    for group in groups {
        if input.revision < 9 && matches!(group.key, GroupKey::Slot(_)) { out.push(group); continue; }
        let mut mask = 0;
        for option in &group.options {
            for &(i, _) in &option.picks { let c = &candidates[i]; mask |= model.mask(c.shop, c.slot);
                if model.packing.as_ref().is_some_and(|p| p.room(c.class).is_some()) { mask |= 1u128 << 63; } }
        }
        if mask == 0 { out.push(group); continue; }
        let mut joined = vec![group];
        let mut again = true;
        while again {
            again = false;
            let mut i = 0;
            while i < sets.len() {
                if sets[i].0 & mask != 0 {
                    let (m, mut gs) = sets.remove(i); mask |= m; joined.append(&mut gs); again = true;
                } else { i += 1; }
            }
        }
        sets.push((mask, joined));
    }
    let value = |picks: Vec<(usize, i32)>| {
        let mut choice = Choice { picks, ..Choice::default() };
        let mut units = Vec::new();
        let mut slots = BTreeSet::new();
        let mut learned = BTreeSet::new();
        for &(i, count) in &choice.picks {
            let c = &candidates[i];
            choice.cost += i64::from(count) * i64::from(c.price);
            choice.weight += i64::from(count) * i64::from(c.weight);
            if c.slots_cost > 0 { slots.insert(c.class); }
            units.extend(model.units(i64::from(c.shop), i64::from(c.slot), i64::from(count)));
            if facts.get(&(c.shop, c.slot)).is_some_and(|f| f.serves & (1 << crate::borg_town_utility::index(crate::borg_town_usefulness::Need::Knowledge)) != 0) { learned.insert(c.kind); }
        }
        choice.slots = slots.len() as i32;
        if let Some(p) = &model.packing { p.reprice(&mut choice, candidates); }
        choice.value = model.gain(&units) + choice.picks.iter().map(|&(i,_)| gear.get(&i).copied().unwrap_or(0)).sum::<i64>()
            + learned.len() as i64 * knowledge * UNIT
            - crate::borg_town_utility::burden(space, choice.slots, choice.weight, limits.pack_slots, carry);
        choice
    };
    for (mask, groups) in sets {
        let mut options = vec![Choice::default()];
        for g in &groups { for option in g.options.iter().skip(1) { options.push(value(option.picks.clone())); } }
        let mut positions = vec![0; groups.len()];
        loop {
            let picks = |pos: &[usize]| groups.iter().zip(pos).flat_map(|(g, &o)| g.options[o].picks.clone()).collect();
            let current = value(picks(&positions));
            let mut best: Option<(usize, usize, Choice)> = None;
            for (gi, g) in groups.iter().enumerate() {
                let alternatives: Vec<usize> = if input.revision >= 9 && matches!(g.key, GroupKey::Slot(_)) {
                    (1..g.options.len()).filter(|&oi| oi != positions[gi]).collect()
                } else { vec![positions[gi] + 1] };
                for oi in alternatives {
                if oi >= g.options.len() { continue; }
                let previous = positions[gi];
                positions[gi] = oi;
                let next = value(picks(&positions));
                positions[gi] = previous;
                let gain = next.value - current.value;
                let bm: i64 = next.picks.iter().filter(|&&(i, _)| candidates[i].is_black_market())
                    .map(|&(i, n)| i64::from(candidates[i].price) * i64::from(n)).sum();
                if gain <= 0 || next.cost > limits.gold || next.weight > limits.weight || next.slots > limits.pack_slots || bm > market_budget { continue; }
                let price = (next.cost - current.cost).max(1);
                if best.as_ref().is_none_or(|(_, _, b)| i128::from(gain) * i128::from((b.cost - current.cost).max(1)) > i128::from(b.value - current.value) * i128::from(price)) { best = Some((gi, oi, next)); }
                }
            }
            let Some((gi, oi, next)) = best else { break; };
            positions[gi] = oi;
            options.push(next);
        }
        out.push(Group { key: GroupKey::Need(mask.trailing_zeros() as u8), options });
    }
    out
}

#[cfg(test)]
mod tests {
    #[test]
    fn readiness_keeps_the_configured_reserve_floor_and_helps_empty_launchers() {
        assert_eq!(super::ranged_reserves(8, 0, true), (8, 0));
        assert_eq!(super::ranged_reserves(8, 6, true), (8, 3));
        assert_eq!(super::ranged_reserves(0, 6, false), (6, 0));
        assert_eq!(super::ranged_reserves(0, 3, true), (3, 2));
    }
    use super::*;
    fn model() -> Model {
        Model { available: Vec::new(), packing: None, curves: (0..COUNT).map(|i| [i as i64, 100 * UNIT, UNIT, 0, 0]).collect(), supplies: Vec::new() }
    }
    fn item(place: Place, slot: u8, count: i32) -> KitItem {
        KitItem { home_role: Default::default(), home_room: None, keep_last_book: false, protect_disposal: false, home_unwanted: false, home_limited: false, joins_pack: false, place, slot, kind: 1, qty: count, weight: 10, need: -1, reduction: 0,
            power: 0, proceeds: 0, sell_shop: 0, unknown: false, unknown_flavour: false, just_bought: false, cursed_spare: false, fuel: 0, serves: 0 }
    }
    #[test]
    fn csw_preference_waits_for_staples() {
        let mut m = model();
        let light_goal = 20_000;
        let food_floor = 150 * UNIT;
        let phase_reserve = 6;
        m.curves[LIGHT][3] = light_goal * UNIT;
        m.curves[FOOD][3] = food_floor;
        m.curves[PHASE][3] = phase_reserve * UNIT;
        assert!(csw_staples_covered(&m, light_goal, food_floor, phase_reserve));
        for row in [LIGHT, FOOD, PHASE] {
            m.curves[row][3] -= UNIT;
            assert!(!csw_staples_covered(&m, light_goal, food_floor, phase_reserve));
            m.curves[row][3] += UNIT;
        }
        m.curves[LIGHT][1] = 0;
        m.curves[LIGHT][3] = 0;
        assert!(!csw_staples_covered(&m, light_goal, food_floor, phase_reserve));
        assert!(csw_staples_covered(&m, 0, food_floor, phase_reserve));
        m.curves[PHASE][3] = 0;
        assert!(!csw_staples_covered(&m, 0, food_floor, 0));
    }

    #[test]
    fn csw_preference_counts_carried_full_cures_and_caps_the_bonus() {
        use crate::borg_town_stock::Candidate;

        let mut m = model();
        m.curves.truncate(PHASE_RESERVE + 1);
        m.curves[CONFUSION][1] = 0;
        m.supplies = vec![[4, 0, CONFUSION as i64, UNIT], [4, 1, CONFUSION as i64, UNIT]];
        let csw = Candidate { shop: 4, slot: 0, kind: 1, tval: TV_POTION as u8, sval: 20,
            price: 70, qty: 3, weight: 4, slots_cost: 1, stack_count: 0, class: 1 };
        let ccw = Candidate { slot: 1, kind: 2, sval: 21, price: 200, class: 2, ..csw };
        let stock = Stock { candidates: vec![csw, ccw], ..Stock::default() };
        let weight = 100 * UNIT;

        let mut empty = m.clone();
        add_csw_reserve(&mut empty, &stock, 20, weight);
        assert!(empty.valid());
        assert_eq!(empty.ware(4, 0, 1), weight);
        assert_eq!(empty.ware(4, 0, 2), weight + weight / 4);
        assert_eq!(empty.ware(4, 0, 3), empty.ware(4, 0, 2));
        assert_eq!(empty.ware(4, 1, 1), 0);

        m.curves[CONFUSION][3] = UNIT;
        m.supplies.push([PACK, 0, CONFUSION as i64, UNIT]);
        add_csw_reserve(&mut m, &stock, 20, weight);
        assert_eq!(m.ware(4, 0, 1), weight / 4);
        assert_eq!(m.ware(4, 0, 2), weight / 4);
        assert_eq!(m.gain(&m.units(PACK, 0, -1)), -weight);
    }

    #[test]
    fn alchemist_restore_life_purchase_and_retention_follow_need_and_stock() {
        use crate::borg_town_record::{action, first_action, step, StepInput, PARAMS};
        use crate::borg_town_solve::Limits;
        use crate::borg_town_stock::Candidate;

        let potion = Candidate { shop: 4, slot: 0, kind: 1, tval: TV_POTION as u8, sval: 16,
            price: 400, qty: 1, weight: 4, slots_cost: 1, stack_count: 0, class: 1 };
        let decide = |drained, reserve, stocked, gold: i32, carried: i32| {
            let kit = if carried > 0 {
                vec![KitItem { sell_shop: 4, proceeds: 100, ..item(Place::Pack, 0, carried) }]
            } else { Vec::new() };
            let candidates = if stocked { vec![potion.clone()] } else { Vec::new() };
            let mut potion_item = crate::borg_item::blank_item();
            potion_item.tval = potion.tval;
            potion_item.sval = potion.sval;
            potion_item.aware = true;
            potion_item.iqty = 1;
            let mut m = Model {
                available: candidates.iter().map(|c| [i64::from(c.shop), i64::from(c.slot), i64::from(c.qty)]).collect(),
                packing: None, curves: initial_curves(STAPLE_LIGHT + 1, 0, UNIT, drained, reserve, 100 * UNIT),
                supplies: Vec::new(),
            };
            let effects = [(EF_RESTORE_EXP as u16, 0, 0)];
            let supply = |scope| effect_supply_linear(&effects, 386, 100, false).into_iter()
                .chain(restore_life_supply(&potion_item, i32::from(potion.sval), reserve, scope, gold, 1000));
            if stocked {
                m.supplies.extend(supply(4).map(|(cap, units)| [4, 0, cap as i64, units]));
            }
            if carried > 0 {
                for (cap, units) in supply(PACK) {
                    m.curves[cap][3] = i64::from(carried) * units;
                    m.supplies.push([PACK, 0, cap as i64, units]);
                }
            }
            assert_eq!(restore_life_supply(&potion_item, i32::from(potion.sval), reserve, 7, gold, 1000).is_some(), reserve > 0);
            assert_eq!(m.curves.get(RESTORE_LIFE_RESERVE).map(|row| row[2]), (reserve > 0).then_some(-reserve * UNIT));
            let mut settings = [0; PARAMS];
            settings[(BORG_TOWN_W_SPACE - BORG_TOWN_PLAN) as usize] = 25;
            settings[(BORG_TOWN_HOME_DISCOUNT - BORG_TOWN_PLAN) as usize] = 500;
            let mut traits = [0; BI_MAX as usize];
            traits[BI_CARRY as usize] = 100;
            let input = StepInput { departure: false, shop_overloaded: false, storing: None, revision: 12, capability: Some(&m), utility: true,
                black_market_reserve: 0, black_market_percentage: -1, candidates,
                facts: &[], tryon: &[], limits: Limits { gold: i64::from(gold), pack_slots: 4, weight: 100 },
                urgent: vec![], params: settings, traits: &traits, power: 0, carried: &[],
                cure_spell: false, reads_now: false, kit: &kit, home_slots: 10 };
            let output = step(&input);
            first_action(&input.candidates, &input.urgent, &output, input.facts, input.kit)
        };

        for gold in [400, 999] {
            assert_eq!(decide(false, 1, true, gold, 0).0, action::LEAVE);
            assert_eq!(decide(false, 2, true, gold, 1).0, action::LEAVE);
            assert_eq!(decide(true, 1, true, gold, 0).0, action::BUY);
        }
        let (code, location, _, _) = decide(false, 1, true, 1000, 0);
        assert_eq!((code, location), (action::BUY, [4, 0, 1, 400]));
        assert_eq!(decide(true, 0, true, 399, 0).0, action::LEAVE);
        assert_eq!(decide(true, 0, false, 787_811, 0).0, action::LEAVE);
        assert_eq!(decide(false, 0, true, 787_811, 0).0, action::LEAVE);
        assert_eq!(decide(false, 0, false, 787_811, 1).0, action::SELL);
        assert_eq!(decide(false, 1, false, 400, 1).0, action::LEAVE);
    }

    /// #516 (review of #563): the fear-cure cap's value curve. With the cap
    /// at 2 (demand -2 units) one potion is worth half the weight, two the
    /// whole weight, and a third adds nothing, where the diminishing curve
    /// gave each potion a small, never-zero value.
    #[test]
    fn ranger_soft_cap_then_a_diminishing_curve() {
        // #716: 50 shots as a requirement, then a curve that never stops.
        let w = 600 * UNIT;
        let mut m = Model { available: Vec::new(), packing: None, curves: (0..COUNT).map(|i| [i as i64, 0, UNIT, 0, 0]).collect(), supplies: Vec::new() };
        m.curves[RANGER_MISSILES] = [RANGER_MISSILES as i64, w, -50 * UNIT, 0, 0];
        m.curves[EXTRA_SHOTS] = [EXTRA_SHOTS as i64, w * 75 / 100, 18 * UNIT, 0, 0];
        let at = |m: &mut Model, a: i64| { m.curves[RANGER_MISSILES][3] = a * UNIT; m.curves[EXTRA_SHOTS][3] = a * UNIT;
            m.gain(&[(RANGER_MISSILES, UNIT), (EXTRA_SHOTS, UNIT)]) };
        let under = at(&mut m, 0);
        assert_eq!(under, w / 50, "a shot under the cap: the requirement's share");
        let (first, later, far) = (at(&mut m, 50), at(&mut m, 80), at(&mut m, 300));
        assert!(first > later && later > far && far > 0, "past the cap: less and less, never nothing");
        m.curves[RANGER_MISSILES][2] = 30 * UNIT; // no cap (C's curve): no extra value
        m.curves[RANGER_MISSILES][3] = 0;
        assert_eq!(m.gain(&[(EXTRA_SHOTS, UNIT)]), 0);
    }

    /// #761, #1103: unseen flavour can discount the effect's utility but
    /// does not turn the wares counted in discrete reserves into partial copies.
    #[test]
    fn unseen_scaling_keeps_discrete_reserve_units() {
        let exact = [(PHASE, UNIT), (PHASE_RESERVE, UNIT), (RANGED_RESERVE, UNIT),
            (RESTORE_LIFE_RESERVE, UNIT), (RESTORE_STAT_RESERVE, UNIT)];
        let mut out = exact.to_vec();
        out.push((HEAL, 20 * UNIT));
        let mut half = exact.to_vec();
        half.push((HEAL, 10 * UNIT));
        assert_eq!(unseen_scaled(out.clone(), 100), out);
        assert_eq!(unseen_scaled(out.clone(), 50), half);
        assert_eq!(unseen_scaled(out, 0), exact);
    }

    /// #761 (review of #778): rows that scale to zero are dropped, not kept —
    /// the model rejects a zero-unit supply row (`Model::valid`), and the kept
    /// rows stopped the town replay of Ranger 425904011 with "invalid
    /// capability facts" on its first record.
    #[test]
    fn the_unseen_percent_drops_rows_that_scale_to_zero() {
        assert_eq!(unseen_scaled(vec![(HEAL, UNIT), (LIGHT, 2 * UNIT)], 0), Vec::new());
        assert_eq!(unseen_scaled(vec![(HEAL, 3 * UNIT)], 50), vec![(HEAL, 3 * UNIT / 2)]);
        assert_eq!(unseen_scaled(vec![(PHASE, UNIT)], 0), vec![(PHASE, UNIT)], "Phase Door is not scaled");
    }

    /// #761: the first `town_phase_reserve` Phase Doors are a requirement on
    /// top of the Phase Door curve: each worth the same up to the reserve,
    /// nothing more past it; off (weight 0) it adds nothing.
    #[test]
    fn the_phase_reserve_is_a_requirement_on_top_of_the_curve() {
        let w = 120 * UNIT;
        let mut m = Model { available: Vec::new(), packing: None, curves: (0..COUNT).map(|i| [i as i64, 0, UNIT, 0, 0]).collect(), supplies: Vec::new() };
        m.curves[PHASE] = [PHASE as i64, 180 * UNIT, 3600, 0, 0];
        let curve_only = m.gain(&[(PHASE, UNIT)]);
        assert_eq!(m.gain(&[(PHASE, UNIT), (PHASE_RESERVE, UNIT)]), curve_only, "reserve off: nothing added");
        m.curves[PHASE_RESERVE] = [PHASE_RESERVE as i64, w * 2, -2 * UNIT, 0, 0];
        assert_eq!(m.gain(&[(PHASE, UNIT), (PHASE_RESERVE, UNIT)]), curve_only + w, "a reserved scroll adds its share");
        assert_eq!(m.gain(&[(PHASE, 2 * UNIT), (PHASE_RESERVE, 2 * UNIT)]) - m.gain(&[(PHASE, 2 * UNIT)]), 2 * w);
        assert_eq!(m.gain(&[(PHASE_RESERVE, 3 * UNIT)]), 2 * w, "a third adds nothing to the reserve");
    }

    /// #987: the curves of seed 681096527's plan record at game turn 541
    /// (Elf Ranger, 14 HP, 109 gold in the Alchemist, 4b2c2fcf at the shipped
    /// settings). Each reserved Phase Door (180,000 on the reserve, 150 percent
    /// of the 120,000 escape weight) outbids the first Cure Light Wounds
    /// (70,588 on the healing curve and 100,000 on the healing reserve), the
    /// sixth scroll as well as the first. With the first-cure row the first
    /// cure outbids every reserved scroll; the second cure does not.
    /// Review of #1112, F1: the restoring-mushroom reserve is one copy on a row
    /// of its own: the first Vigor is worth the weight, the second nothing, a
    /// permanent stat potion keeps its whole value beside carried Vigor, and the
    /// Home counts one the same way.
    #[test]
    fn a_restoring_mushroom_is_a_reserve_of_one_and_not_a_permanent_gain() {
        let weight = 20 * UNIT;
        let mut m = Model { available: Vec::new(), packing: None, curves: (0..COUNT).map(|i| [i as i64, 0, UNIT, 0, 0]).collect(),
            supplies: vec![[PACK, 0, RESTORE_STAT_RESERVE as i64, UNIT], [HOME, 0, RESTORE_STAT_RESERVE as i64, UNIT],
                [PACK, 1, STATS as i64, UNIT]] };
        m.curves[RESTORE_STAT_RESERVE] = [RESTORE_STAT_RESERVE as i64, weight, -UNIT, 0, 0];
        m.curves[STATS] = [STATS as i64, 100 * UNIT, -UNIT, 0, 0];
        let vigor = |place, qty| KitItem { place, slot: 0, qty, ..item(place, 0, qty) };
        assert_eq!(m.gain(&[(RESTORE_STAT_RESERVE, UNIT)]), weight, "the first copy");
        m.curves[RESTORE_STAT_RESERVE][3] = UNIT;
        assert_eq!(m.gain(&[(RESTORE_STAT_RESERVE, UNIT)]), 0, "the second copy is worth nothing");
        assert_eq!(m.keep(&vigor(Place::Pack, 1)), weight, "carried, it is kept");
        // A Potion of Strength below 18/100 keeps its whole value beside it.
        assert_eq!(m.gain(&[(STATS, UNIT)]), 100 * UNIT);
        m.curves[RESTORE_STAT_RESERVE][3] = 5 * UNIT;
        assert_eq!(m.gain(&[(STATS, UNIT)]), 100 * UNIT, "five carried Vigor do not touch the stat row");
        // The Home counts the same one copy.
        m.curves[RESTORE_STAT_RESERVE][3] = 0;
        assert_eq!(m.reserve(&vigor(Place::Home, 1), 1, UNIT), weight);
        m.curves[RESTORE_STAT_RESERVE][4] = UNIT;
        assert_eq!(m.reserve(&vigor(Place::Home, 1), 1, UNIT), 0);
    }

    #[test]
    fn the_first_cure_comes_before_the_phase_door_reserve() {
        let mut m = Model { available: Vec::new(), packing: None, curves: (0..COUNT).map(|i| [i as i64, 0, UNIT, 0, 0]).collect(), supplies: Vec::new() };
        m.curves[HEAL] = [HEAL as i64, 240_000, 33_600, 0, 0];
        m.curves[HEAL_RESERVE] = [HEAL_RESERVE as i64, 200_000, -28_000, 0, 0];
        m.curves[PHASE] = [PHASE as i64, 180_000, 3_600, 0, 0];
        m.curves[PHASE_RESERVE] = [PHASE_RESERVE as i64, 1_800_000, -10 * UNIT, 0, 0];
        let clw = |n: i64| vec![(HEAL, 14_000 * n), (HEAL_RESERVE, 14_000 * n), (FIRST_CURE, n * UNIT)];
        let phase = |n: i64| vec![(PHASE, n * UNIT), (PHASE_RESERVE, n * UNIT)];
        let first_clw = m.gain(&clw(1));
        assert_eq!(first_clw, 70_588 + 100_000);
        let sixth_phase = m.gain(&phase(6)) - m.gain(&phase(5));
        assert!(sixth_phase > first_clw, "today: the sixth Phase Door ({sixth_phase}) outbids the first cure ({first_clw})");
        assert!(m.gain(&phase(1)) > first_clw);
        // town_first_cure = 1 at 200 percent of the escape weight.
        m.curves[FIRST_CURE] = [FIRST_CURE as i64, 240_000, -UNIT, 0, 0];
        let first_clw = m.gain(&clw(1));
        assert_eq!(first_clw, 70_588 + 100_000 + 240_000);
        assert!(first_clw > m.gain(&phase(1)), "the first cure outbids the first reserved Phase Door");
        let second_clw = m.gain(&clw(2)) - first_clw;
        assert!(second_clw < sixth_phase, "the second cure ({second_clw}) still waits for the reserve");
        // town_first_cure = 2: each of the first two copies adds 240,000 on the
        // row, a third nothing.
        m.curves[FIRST_CURE] = [FIRST_CURE as i64, 480_000, -2 * UNIT, 0, 0];
        let row = |m: &Model, n: i64| m.gain(&[(FIRST_CURE, n * UNIT)]);
        assert_eq!(row(&m, 1), 240_000);
        assert_eq!(row(&m, 2) - row(&m, 1), 240_000);
        assert_eq!(row(&m, 3), row(&m, 2), "a third cure adds nothing on the row");
    }

    /// #859 (owner: "make it be 110 by default, so that they buy a few"):
    /// the scale stretches the Recall curve; 100 is the curve before it.
    #[test]
    fn the_recall_curve_values_every_scroll_less_than_the_last() {
        let (trip, walk) = (500 * UNIT, 80 * UNIT);
        assert_eq!(recall_curve(trip, walk, 1, 0), ((trip, -UNIT), None), "0: the #557 requirement");
        assert_eq!(recall_curve(trip, walk, 0, 0), ((walk, UNIT), None), "0: the walk curve");
        let mut m = Model { available: Vec::new(), packing: None, curves: (0..COUNT).map(|i| [i as i64, 0, UNIT, 0, 0]).collect(), supplies: Vec::new() };
        let scrolls = |m: &Model, n: i64| m.gain(&[(RECALL, n * UNIT), (RECALL_RESERVE, n * UNIT)]);
        for scale in [100, 110, 200] {
            let ((w, d), reserve) = recall_curve(trip, walk, 1, scale);
            m.curves[RECALL] = [RECALL as i64, w, d, 0, 0];
            let (rw, rd) = reserve.unwrap();
            m.curves[RECALL_RESERVE] = [RECALL_RESERVE as i64, rw, rd, 0, 0];
            let gains: Vec<i64> = (1..=5).map(|n| scrolls(&m, n) - scrolls(&m, n - 1)).collect();
            assert!(gains[0] > trip, "{scale}: the first carries the reserve and the curve: {gains:?}");
            assert!(gains.windows(2).all(|g| g[1] < g[0] && g[1] > 0), "{scale}: each less, never nothing: {gains:?}");
        }
        let ((w110, _), _) = recall_curve(trip, walk, 1, 110);
        let ((w200, _), _) = recall_curve(trip, walk, 1, 200);
        assert!(w200 > w110, "a higher scale values the spares more");
        let (_, reserve) = recall_curve(trip, walk, 0, 110);
        assert_eq!(reserve, Some((0, UNIT)), "no reserve: an empty row");
    }

    /// #863 (owner: "better 4 cure poison, 4 detect invisible and 3 remove
    /// curse in the pack and 20 light wounds than 40 light wound potions"):
    /// with 20 Cure Light Wounds carried, the first copies of each remedy are
    /// worth more than the 21st Cure Light Wounds; past a couple they are
    /// worth nothing.
    #[test]
    fn the_first_remedies_are_worth_more_than_the_21st_cure_light_wounds() {
        // The numbers `observe` makes at CL5, DL3, 100 HP: exposure 780, the
        // trip weight 50 x 1000, healing at the shipped 125 percent scale,
        // and a Cure Light Wounds worth 6.06 HP of speed-weighted units.
        let exposure = 780;
        let trip = 50 * exposure.max(UNIT);
        let mut m = Model { available: Vec::new(), packing: None, curves: (0..COUNT).map(|i| [i as i64, 0, UNIT, 0, 0]).collect(), supplies: Vec::new() };
        let heal = 100 * exposure * 125 / 100;
        m.curves[HEAL] = [HEAL as i64, heal, heal, 20 * 6060, 0];
        let clw = 6060;
        let twenty_first = m.gain(&[(HEAL, clw)]);
        assert!(twenty_first > 0, "a copy is worth something: {twenty_first}");
        let (w, d) = prepared_row(trip, 8, 2);
        for (_, row) in PREPARED { m.curves[row] = [row as i64, w, d, 0, 0]; }
        for (cap, row) in PREPARED {
            let gains: Vec<i64> = (1..=4).map(|n| m.gain(&[(row, n * UNIT)]) - m.gain(&[(row, (n - 1) * UNIT)])).collect();
            assert!(gains[0] > 2 * twenty_first && gains[1] > 2 * twenty_first, "capability {cap}: the target copies {gains:?} beat the 21st CLW {twenty_first}");
            assert_eq!(gains[0], gains[1], "capability {cap}: each target copy is worth the same");
            assert_eq!(gains[2..], [0, 0], "capability {cap}: nothing past the target");
        }
        assert_eq!(prepared_row(trip, 0, 2).0, 0, "0 percent: no weight");
        assert_eq!(prepared_row(trip, 8, 0), (scale_percent(trip, 8), -UNIT), "at least one copy");
        assert_eq!(prepared_row(trip, 8, 4), (4 * scale_percent(trip, 8), -4 * UNIT), "the same value a copy, whatever the target");
    }

    fn staple_model() -> Model {
        let mut m = Model { available: Vec::new(), packing: None, curves: (0..COUNT).map(|i| [i as i64, 0, UNIT, 0, 0]).collect(), supplies: Vec::new() };
        m.curves[HEAL] = [HEAL as i64, 97_500, 97_500, 0, 0];
        m.curves[PHASE] = [PHASE as i64, 180 * UNIT, 3600, 0, 0];
        m.curves[THROWN_OIL] = [THROWN_OIL as i64, 40 * UNIT, 10 * UNIT, 0, 0];
        m.curves[FOOD] = [FOOD as i64, 100_000, 150 * UNIT, 0, 0];
        m.curves[LIGHT] = [LIGHT as i64, 100_000, 40_000 * UNIT, 0, 0];
        m
    }

    /// #863 (owner: "make staple objects start losing value"; "it must be a
    /// decaying curve"): the free copies of each staple are valued as before,
    /// the next keeps the decay percent of the marginal value, the one after
    /// that of that, and so on: never nothing, each less than the one before.
    /// An unset row (a positive demand) is the plain curve.
    #[test]
    fn staples_decay_after_their_free_copies() {
        let (clw, csw, ccw) = (6060, 12_000, 18_000);
        let staples = [(PHASE, STAPLE_PHASE, UNIT, 6), (THROWN_OIL, STAPLE_OIL, UNIT, 6),
            (FOOD, STAPLE_FOOD, 30 * UNIT, 4), (LIGHT, STAPLE_LIGHT, 5000 * UNIT, 4)];
        let mut m = staple_model();
        let marginal = |m: &Model, cap: usize, unit: i64, n: i64| m.gain(&[(cap, n * unit)]) - m.gain(&[(cap, (n - 1) * unit)]);
        for (cap, row, unit, free) in staples {
            let plain: Vec<i64> = (1..=14).map(|n| marginal(&m, cap, unit, n)).collect();
            m.curves[row] = [row as i64, knee_row(80, unit), -free * unit, 0, 0];
            let kneed: Vec<i64> = (1..=14).map(|n| marginal(&m, cap, unit, n)).collect();
            let free = free as usize;
            assert_eq!(&kneed[..free], &plain[..free], "capability {cap}: the free copies are as before");
            assert!((kneed[free] as f64 - plain[free] as f64 * 0.8).abs() <= plain[free] as f64 * 0.05 + 2.0,
                "capability {cap}: the first copy past them keeps 80 percent: {} vs {}", kneed[free], plain[free]);
            for n in free..14 {
                assert!(kneed[n] > 0, "capability {cap}: copy {} is still worth something", n + 1);
                assert!(kneed[n] < plain[n], "capability {cap}: copy {} is worth less than plain", n + 1);
                if n > free { assert!(kneed[n] < kneed[n - 1], "capability {cap}: each less than the one before: {kneed:?}"); }
            }
            m.curves[row] = [row as i64, 0, UNIT, 0, 0];
        }
        assert_eq!(decayed(5_000, 10_000, 1000, 80), 5_000, "under the knee");
        assert_eq!(decayed(11_000, 10_000, 1000, 80), 10_800);
        assert_eq!(decayed(12_000, 10_000, 1000, 80), 11_440);
        assert_eq!(decayed(12_500, 10_000, 1000, 100), 12_500, "100 percent: no decay");
        // Each healing potion has its own knee on the shared curve.
        let mut m = staple_model();
        let kinds = [(STAPLE_CLW, clw, 6), (STAPLE_CSW, csw, 4), (STAPLE_CCW, ccw, 2)];
        let one = |m: &Model, row: usize, heal: i64, carried: i64| {
            let mut m = m.clone();
            m.curves[HEAL][3] = carried * heal;
            m.curves[row][3] = carried * UNIT;
            m.gain(&[(HEAL, heal), (row, UNIT)])
        };
        let plain: Vec<i64> = kinds.iter().map(|&(_, heal, _)| m.gain(&[(HEAL, heal)])).collect();
        for &(row, heal, free) in &kinds { m.curves[row] = [row as i64, knee_row(80, heal), -free * UNIT, 0, 0]; }
        for (&(row, heal, free), plain) in kinds.iter().zip(&plain) {
            assert_eq!(one(&m, row, heal, 0), *plain, "the first potion is as before");
            let mut last_free = staple_model(); last_free.curves[HEAL][3] = (free - 1) * heal;
            assert_eq!(one(&m, row, heal, free - 1), last_free.gain(&[(HEAL, heal)]), "the last free potion is as before");
            let next = one(&m, row, heal, free);
            let mut p = staple_model(); p.curves[HEAL][3] = free * heal;
            let plain_next = p.gain(&[(HEAL, heal)]);
            assert!(next > 0 && (next as f64) < plain_next as f64 * 0.9, "kind {row}: the copy past the knee is worth less: {next} vs {plain_next}");
        }
        // Copies carried count: 20 carried Cure Light Wounds, the 21st is worth
        // a tenth of its plain value, and the 3rd Remove Curse is worth more.
        let plain_21st = { let mut p = staple_model(); p.curves[HEAL][3] = 20 * clw; p.gain(&[(HEAL, clw)]) };
        let twenty_first = one(&m, STAPLE_CLW, clw, 20);
        assert!(twenty_first * 10 < plain_21st * 2, "the 21st: {twenty_first} vs {plain_21st}");
        let (w, d) = prepared_row(50 * 1000, 8, 2);
        m.curves[PREP_CURSE] = [PREP_CURSE as i64, w, d, 0, 0];
        let second_remove_curse = m.gain(&[(PREP_CURSE, 2 * UNIT)]) - m.gain(&[(PREP_CURSE, UNIT)]);
        assert!(second_remove_curse > twenty_first, "the 2nd Remove Curse {second_remove_curse} beats the 21st Cure Light Wounds {twenty_first}");
        // The owner's pack: 20 Cure Light Wounds, 4 Detect Invisible, 2 Remove Curse.
        let (w, d) = prepared_row(50 * 1000, 8, 4);
        m.curves[PREP_INVISIBLE] = [PREP_INVISIBLE as i64, w, d, 0, 0];
        let fourth_detect = m.gain(&[(PREP_INVISIBLE, 4 * UNIT)]) - m.gain(&[(PREP_INVISIBLE, 3 * UNIT)]);
        assert!(fourth_detect > twenty_first, "the 4th Detect Invisible {fourth_detect} beats the 21st Cure Light Wounds {twenty_first}");
        assert_eq!(m.gain(&[(PREP_INVISIBLE, 5 * UNIT)]) - m.gain(&[(PREP_INVISIBLE, 4 * UNIT)]), 0, "the 5th is worth nothing");
    }

    /// #1134 (owner: "make the town planner start buying CCW and cure poison in
    /// the same way that they buy detect invisible"): a prepared row prices a
    /// copy, not a gold piece. At the shared percent the first Neutralize
    /// Poison (75 gold) is worth less per gold than the first Detect Invisible
    /// (15 gold); at five times the percent the same; the CCW row (200 gold)
    /// needs fourteen times (200/15 = 13.3). Past its copies a row is worth nothing.
    #[test]
    fn a_remedy_of_its_own_percent_buys_dear_cures_with_spare_gold() {
        let mut m = Model { available: Vec::new(), packing: None, curves: (0..COUNT).map(|i| [i as i64, 0, UNIT, 0, 0]).collect(), supplies: Vec::new() };
        let first = |m: &mut Model, row: usize, percent: i32, copies: i32| {
            let (w, d) = prepared_row(50 * 1000, percent, copies);
            m.curves[row] = [row as i64, w, d, 0, 0];
            m.gain(&[(row, UNIT)])
        };
        let detect = first(&mut m, PREP_INVISIBLE, 8, 4);
        let shared = first(&mut m, PREP_POISON, 8, 3);
        assert!(shared * 15 < detect * 75, "shared percent: Neutralize Poison {shared}/75 below Detect Invisible {detect}/15");
        let own = first(&mut m, PREP_POISON, 40, 3);
        assert!(own * 15 >= detect * 75, "five times: Neutralize Poison {own}/75 at least Detect Invisible {detect}/15");
        let ccw = first(&mut m, PREP_CCW, 8 * 14, 2);
        assert!(ccw * 15 >= detect * 200, "fourteen times: CCW {ccw}/200 at least Detect Invisible {detect}/15");
        assert_eq!(m.gain(&[(PREP_CCW, 3 * UNIT)]) - m.gain(&[(PREP_CCW, 2 * UNIT)]), 0, "the 3rd CCW past 2 copies is worth nothing");
        assert_eq!(PREP_CCW, DEV_CONTROL + 1);
        assert_eq!(PREP_NEUTRALIZE, PREP_CCW + 1);
        assert_eq!(RESTORE_STAT_RESERVE, PREP_NEUTRALIZE + 1);
        assert_eq!(NAMES[PREP_CCW], "prepared: Cure Critical Wounds");
        assert_eq!(NAMES[PREP_NEUTRALIZE], "prepared: Neutralize Poison");
        let neutralize = first(&mut m, PREP_NEUTRALIZE, 40, 2);
        assert!(neutralize * 15 >= detect * 75, "its own row at five times: Neutralize Poison {neutralize}/75 at least Detect Invisible {detect}/15");
        assert_eq!(m.gain(&[(PREP_NEUTRALIZE, 3 * UNIT)]) - m.gain(&[(PREP_NEUTRALIZE, 2 * UNIT)]), 0, "the 3rd past 2 copies is worth nothing");
    }

    /// #1137 (owner: "when he is rich enough to get a WoR he starts buying
    /// antipoison and CCW"): the dear rows wait for Recall's purse floor and
    /// the Recall reserve in the pack; off, they always count.
    #[test]
    fn the_dear_rows_wait_for_recall() {
        assert!(dear_rows_paid(false, 0, 200, 0, 2), "no gate: always");
        assert!(!dear_rows_paid(true, 10_000, 200, 1, 2), "rich, but a reserve scroll is missing: Recall first");
        assert!(!dear_rows_paid(true, 199, 200, 2, 2), "the reserve carried, but below Recall's purse floor");
        assert!(dear_rows_paid(true, 200, 200, 2, 2), "the reserve carried and the purse at the floor");
        assert!(!dear_rows_paid(true, 500, 200, 0, 0), "no reserve set: still one scroll first");
        assert!(dear_rows_paid(true, 500, 200, 1, 0));
    }

    /// #863 (review F2): a healing row's knee counts potions and prices them in
    /// the HEAL units one potion adds: 40 Cure Light Wounds with 7 free are worth
    /// about 11 potions; and `value`, which reads only `STAPLES` rows, never
    /// touches a healing row.
    #[test]
    fn a_healing_knee_counts_potions() {
        let clw = 6060;
        let mut m = staple_model();
        m.curves[STAPLE_CLW] = [STAPLE_CLW as i64, knee_row(80, clw), -7 * UNIT, 40 * UNIT, 0];
        let lost = m.heal_loss(None);
        let effective = (40 * clw - lost) as f64 / clw as f64;
        assert!((effective - 11.0).abs() < 0.2, "40 potions are worth about 11, not {effective}");
        m.curves[STAPLE_CLW][3] = 7 * UNIT;
        assert_eq!(m.heal_loss(None), 0, "the free copies lose nothing");
        let plain = coverage(&m.curves[HEAL], 40 * clw);
        assert_eq!(m.value(HEAL, &m.curves[HEAL], 40 * clw), plain, "`value` leaves the healing curve to `gain`");
    }

    /// #895 (review of #898, F2 and F6): a Staff of Detect Invisible feeds the
    /// prepared Detect Invisible row and no device row; a Wand of Light's damage
    /// is light only and does not fill the attack row, where a Wand of Magic
    /// Missile's does.
    #[test]
    fn device_rows_do_not_double_count_or_count_light() {
        use crate::headers::list_effects_h::{EF_BOLT_OR_BEAM, EF_LINE};
        use crate::headers::list_projections_h::PROJ_LIGHT_WEAK;
        assert!(DEVICE_ROWS.iter().all(|&(cap, _)| cap != INVISIBLE), "invisible detection is the prepared row's");
        assert!(DEVICE_ROWS.iter().any(|&(cap, row)| cap == MONSTERS && row == DEV_DETECT));
        let light = [(EF_LINE as u16, PROJ_LIGHT_WEAK as i32, 27)];
        let missile = [(EF_BOLT_OR_BEAM as u16, 0, 18)]; // subtype 0 is PROJ_ARROW, native damage 0..=24
        assert!(!device_attack(&light, 100, 1000, false), "a Wand of Light is not an attack device");
        assert!(device_attack(&missile, 100, 1000, false), "a Wand of Magic Missile is");
        assert!(device_attack(&[light[0], missile[0]], 100, 1000, false), "one real attack effect is enough");
    }

    /// #895 (owner: "we are going to need space for other devices"): a learned,
    /// charged device is worth keeping on its situation's row (each charge an Nth
    /// of the target, nothing past it), where on the shared ATTACK curve, saturated
    /// by arrows and oil, it is worth next to nothing; an empty one has no charges
    /// and no value; the Staff of Teleportation is gated by the gold floor unless
    /// one is carried.
    #[test]
    fn a_charged_device_is_kept_as_variety() {
        let trip = 50 * 1000;
        let (weight, demand) = device_row(trip, 24, 10);
        assert_eq!((weight, demand), (12_000, -10 * UNIT), "the target together is the percent of the trip weight");
        let mut m = staple_model();
        // ATTACK saturated by 100 shots and flasks already carried.
        m.curves[ATTACK] = [ATTACK as i64, 100 * UNIT, 100 * UNIT, 500 * UNIT, 0];
        m.supplies = vec![[PACK, 0, ATTACK as i64, 10 * UNIT], [PACK, 0, DEV_ATTACK as i64, 10 * UNIT]];
        let wand = item(Place::Pack, 0, 1);
        let plain = m.keep(&wand);
        m.curves[DEV_ATTACK] = [DEV_ATTACK as i64, weight, demand, 10 * UNIT, 0];
        let kept = m.keep(&wand);
        assert!(kept >= weight, "the wand's ten charges are worth the target: {kept} against {plain} on the shared curve");
        assert!(kept > 5 * plain.max(1), "far more than on the saturated curve");
        // Nothing past the target: an eleventh charge adds nothing.
        assert_eq!(m.gain(&[(DEV_ATTACK, UNIT)]), 0);
        // Each charge up to it is worth the same.
        m.curves[DEV_ATTACK][3] = 0;
        assert_eq!(m.gain(&[(DEV_ATTACK, UNIT)]), weight / 10);
        assert_eq!(m.gain(&[(DEV_ATTACK, 4 * UNIT)]), 4 * (weight / 10));
        // An empty device supplies no row at all (units > 0 only).
        assert_eq!(m.gain(&[]), 0);
        // The Staff of Teleportation: gated below the floor unless carried.
        assert!(escape_gated(3000, 2999, 0));
        assert!(!escape_gated(3000, 3000, 0), "at the floor");
        assert!(!escape_gated(3000, 100, 5 * UNIT), "one carried is never sold for a small purse");
        assert!(!escape_gated(0, 0, 0), "no floor");
        // Gated, a staff on offer supplies neither the device row nor the shared escape curve (F10),
        // and keeps what else it supplies.
        let staff = vec![(TELEPORT, 5 * UNIT), (DEV_ESCAPE, 5 * UNIT), (RECHARGE_TARGET, UNIT)];
        assert_eq!(without_escape(staff), vec![(RECHARGE_TARGET, UNIT)]);
    }

    /// #863 (review F1): every capability id has a mask bit of its own, ids
    /// past 63 included, except the Recall reserve (63), which shares the packing
    /// room's bit as it did in #864.
    #[test]
    fn every_capability_has_its_own_mask_bit() {
        let supplies = (0..COUNT).map(|i| [5, i as i64, i as i64, UNIT]).collect();
        let m = Model { available: Vec::new(), packing: None, curves: (0..COUNT).map(|i| [i as i64, 0, UNIT, 0, 0]).collect(), supplies };
        let masks: Vec<u128> = (0..COUNT).map(|i| m.mask(5, i as u8)).collect();
        for (i, &mask) in masks.iter().enumerate() {
            assert_eq!(mask, 1u128 << i, "capability {i}");
        }
        assert!(masks.iter().enumerate().all(|(i, a)| masks.iter().skip(i + 1).all(|b| a & b == 0)), "no two capabilities share a bit");
        assert!(m.valid());
    }

    fn two_remedy_groups(a: (usize, usize), b: (usize, usize)) -> usize {
        use crate::borg_town_stock::Candidate;
        use crate::borg_town_solve::{Choice, Group, GroupKey, Limits};
        let cs: Vec<_> = (0..2).map(|i| Candidate { shop: 0, slot: i, kind: u32::from(i) + 1, tval: 0, sval: 0,
            price: 15, qty: 1, weight: 0, slots_cost: 1, stack_count: 0, class: u16::from(i) }).collect();
        let mut m = model();
        for row in [PREP_FEAR, PREP_POISON, POISON, FEAR] { m.curves[row] = [row as i64, 8000, -2 * UNIT, 0, 0]; }
        m.supplies = vec![[0, 0, a.0 as i64, UNIT], [0, 0, a.1 as i64, UNIT], [0, 1, b.0 as i64, UNIT], [0, 1, b.1 as i64, UNIT]];
        let choice = |i: usize| Choice { picks: vec![(i, 1)], cost: 15, value: m.ware(0, i as u8, 1), slots: 1, weight: 0 };
        let groups = vec![Group { key: GroupKey::Class(0), options: vec![Choice::default(), choice(0)] },
            Group { key: GroupKey::Class(1), options: vec![Choice::default(), choice(1)] }];
        let limits = Limits { gold: 100, pack_slots: 5, weight: 100 };
        let input = crate::borg_town_record::StepInput { departure: false, shop_overloaded: false, storing: None, revision: 9, capability: Some(&m), utility: true, black_market_reserve: 0,
            black_market_percentage: -1, candidates: cs.clone(), facts: &[], tryon: &[], limits, urgent: vec![],
            params: [0; crate::borg_town_record::PARAMS], traits: &[], power: 0, carried: &[], cure_spell: false, reads_now: false, kit: &[], home_slots: 10 };
        shared_groups(groups, &input, &cs, &m, &limits, (0, 100, 0)).len()
    }

    /// #863 (review F1): a fear remedy and a poison remedy are different
    /// needs: their groups stay two, so the solver chooses for each; two wares
    /// of the same remedy still share one group (the shortage is the same).
    #[test]
    fn different_remedies_are_different_groups() {
        assert_eq!(two_remedy_groups((FEAR, PREP_FEAR), (POISON, PREP_POISON)), 2, "a fear and a poison remedy stay apart");
        assert_eq!(two_remedy_groups((PREP_FEAR, PREP_FEAR), (PREP_POISON, PREP_POISON)), 2, "so do their prepared rows");
        assert_eq!(two_remedy_groups((FEAR, PREP_FEAR), (FEAR, PREP_FEAR)), 1, "two fear remedies share a group");
    }

    /// #859: the room the Recall reserve still needs: the copies short,
    /// the lightest offered copy's weight, and a slot when none is carried.
    #[test]
    fn the_recall_room_counts_the_copies_short_of_the_reserve() {
        let mut m = Model { available: Vec::new(), packing: None, curves: (0..COUNT).map(|i| [i as i64, 0, UNIT, 0, 0]).collect(), supplies: vec![[5, 3, RECALL as i64, UNIT]] };
        let scroll = crate::borg_town_stock::Candidate { shop: 5, slot: 3, kind: 1, tval: 0, sval: 0, price: 125,
            qty: 4, weight: 5, slots_cost: 1, stack_count: 0, class: 0 };
        let other = crate::borg_town_stock::Candidate { slot: 4, weight: 1, ..scroll };
        let offers = [scroll, other];
        assert_eq!(recall_room(&m, 1, &offers), (5, 1), "one scroll and its slot");
        assert_eq!(recall_room(&m, 0, &offers), (0, 0), "no reserve");
        m.curves[RECALL][3] = UNIT;
        assert_eq!(recall_room(&m, 1, &offers), (0, 0), "carried");
        assert_eq!(recall_room(&m, 2, &offers), (5, 0), "one more joins the carried stack");
        assert_eq!(recall_room(&m, 2, &[other]), (0, 0), "none on offer");
    }

    #[test]
    fn the_ranged_reserve_counts_both_supplies_and_caps_the_first_purchase() {
        let mut model = Model {
            available: Vec::new(), packing: None,
            curves: (0..COUNT).map(|i| [i as i64, 0, UNIT, 0, 0]).collect(),
            supplies: vec![[0, 0, RANGED_RESERVE as i64, UNIT], [1, 0, RANGED_RESERVE as i64, UNIT]],
        };
        model.curves[RANGED_RESERVE] = [RANGED_RESERVE as i64, 300, -3 * UNIT, UNIT, 0];
        assert_eq!(model.ware(0, 0, 1), 100);
        assert_eq!(model.ware(1, 0, 3), 200);
        assert_eq!(model.ranged_reserve_copies(1, 0, 8), Some(2));
        assert_eq!(model.ranged_reserve_copies(1, 0, 1), Some(1));
        assert_eq!(model.ranged_reserve_copies(1, 1, 8), None);
        model.curves[RANGED_RESERVE][3] = 3 * UNIT;
        assert_eq!(model.ranged_reserve_copies(0, 0, 5), None);
        model.curves.pop();
        assert_eq!(model.ranged_reserve_copies(0, 0, 5), None);
    }

    #[test]
    fn the_fear_cure_cap_values_potions_linearly_up_to_the_cap() {
        let w = 50_000;
        let capped = [FEAR as i64, w, -2 * UNIT, 0, 0];
        assert_eq!(coverage(&capped, UNIT), w / 2, "one potion");
        assert_eq!(coverage(&capped, 2 * UNIT), w, "two");
        assert_eq!(coverage(&capped, 3 * UNIT), w, "a third is worth nothing");
        assert_eq!(coverage(&capped, 5 * UNIT), w);
        let curve = [FEAR as i64, w, 100 * UNIT, 0, 0];
        let (one, three) = (coverage(&curve, UNIT), coverage(&curve, 3 * UNIT));
        assert!(one < w / 2 && three > coverage(&curve, 2 * UNIT), "the curve: small, still rising");
    }

    #[test]
    fn ranger_empty_quiver_remains_valuable_when_devices_cover_damage() {
        let mut m = model();
        m.curves[ATTACK] = [ATTACK as i64, 100 * UNIT, 60 * UNIT, 100000 * UNIT, 0];
        m.curves[RANGER_MISSILES] = [RANGER_MISSILES as i64, 250 * UNIT, 22 * UNIT, 0, 0];
        let arrow = missile_supply(CLASS_RANGER, 6 * UNIT);
        let empty = m.gain(&arrow);
        // A 1-gold arrow beats a 15-gold first-copy learning bonus of 25.
        assert!(empty * 15 > 25 * UNIT);
        assert!(empty > 100 * m.gain(&[(ATTACK, 6 * UNIT)]), "devices cannot close the shot reserve");
        m.curves[RANGER_MISSILES][3] = 40 * UNIT;
        assert!(m.gain(&arrow) < empty / 5, "already carried arrows reduce marginal buying priority");
        m.curves[RANGER_MISSILES][2] *= 2;
        assert!(m.gain(&arrow) > 0, "longer trips retain missile value");
        m.curves[RANGER_MISSILES][1] = 0;
        assert_eq!(m.gain(&arrow), m.gain(&[(ATTACK, 6 * UNIT)]), "zero weight restores generic attack valuation");
        assert_eq!(missile_supply(0, 6 * UNIT), [(ATTACK, 6 * UNIT)], "other classes keep their existing supply");
    }

    #[test]
    fn wand_teleport_other_and_stone_to_mud_are_services_not_damage() {
        use crate::headers::list_projections_h::*;
        assert_eq!(effect_supply(&[(EF_BOLT as u16, PROJ_AWAY_ALL as i32, 100)], 20, 1000), [(OTHER, UNIT)]);
        assert_eq!(effect_supply(&[(EF_BOLT as u16, PROJ_KILL_WALL as i32, 30)], 20, 1000), [(DIG, UNIT)]);
        let light = effect_supply(&[(EF_LINE as u16, PROJ_LIGHT_WEAK as i32, 6)], 20, 1000);
        assert_eq!(light, [(ATTACK, 6 * UNIT)], "the shop's Wand of Light has a LINE effect");
        for projection in [PROJ_MON_HEAL, PROJ_MON_SPEED, PROJ_MON_CLONE, PROJ_MON_POLY] {
            for effect in [EF_BOLT, EF_BOLT_STATUS, EF_BOLT_AWARE, EF_PROJECT_LOS] {
                assert!(effect_supply(&[(effect as u16, projection as i32, 100)], 20, 1000).is_empty());
            }
        }
        assert_eq!(effect_supply(&[(EF_BOLT_STATUS_DAM as u16, PROJ_MON_DRAIN as i32, 150)], 20, 1000), [(ATTACK, 150 * UNIT)]);
    }
    #[test]
    fn secondary_cures_and_buffs_are_not_discarded_by_a_primary_healing_role() {
        let units = effect_supply(&[(EF_HEAL_HP as u16, 0, 60), (EF_CURE as u16, TMD_CONFUSED as i32, 0),
            (EF_CURE as u16, TMD_POISONED as i32, 0), (EF_CURE as u16, TMD_BLIND as i32, 0)], 25, 1000);
        assert!(units.contains(&(HEAL, 25 * UNIT)));
        for cap in [CONFUSION, POISON, BLIND] { assert!(units.contains(&(cap, UNIT))); }
        let units = effect_supply(&[(EF_TIMED_INC as u16, TMD_FAST as i32, 30),
            (EF_TIMED_INC as u16, TMD_OPP_FIRE as i32, 20)], 25, 1000);
        // borg-reborn-rs (#452): speed is one use per effect, not its turns.
        assert!(units.contains(&(SPEED, UNIT)) && units.contains(&(FIRE, 20 * UNIT)));
    }
    #[test]
    fn breadth_first_food_scales_the_trip_and_the_floor() {
        // #1110: at 100 the curve is today's; at 150 a short trip's floor is
        // 7.5 rations instead of 5, and a long trip's rations grow alike.
        assert_eq!(food_curve(3, 2 * UNIT, 100), (3 * READY_FOOD * UNIT, -(RATION_NUTRITION * READY_FOOD * UNIT)));
        assert_eq!(food_curve(3, 8 * UNIT, 100), (3 * 8 * UNIT, RATION_NUTRITION * 8 * UNIT));
        let ready = READY_FOOD * UNIT * 3 / 2;
        assert_eq!(food_curve(3, 2 * UNIT, 150), (3 * ready, -(RATION_NUTRITION * ready)));
        assert_eq!(food_curve(3, 8 * UNIT, 150), (3 * 12 * UNIT, RATION_NUTRITION * 12 * UNIT));
        assert_eq!(scale_percent(20_000, 150), 30_000, "the light goal: 20,000 reserve turns become 30,000");
    }
    #[test]
    fn food_uses_actual_nutrition_and_crunch_is_only_a_message() {
        let biscuit = effect_supply(&[(EF_NOURISH as u16, 0, 10), (EF_CRUNCH as u16, 0, 0)], 20, 1000);
        assert_eq!(biscuit, [(FOOD, 10 * UNIT)]);
        assert_eq!(effect_supply(&[(EF_NOURISH as u16, 0, 40)], 20, 1000), [(FOOD, 40 * UNIT)]);
        assert!(effect_supply(&[(EF_NOURISH as u16, 1, 40)], 20, 1000).is_empty());
        assert!(effect_supply(&[(EF_NOURISH as u16, 2, 0)], 20, 1000).is_empty());
    }
    /// borg-reborn-rs (#451): NOURISH:INC_TO fills to a percentage of a full
    /// stomach (effect_handler_NOURISH, subtype 3). Read hungry (15 percent,
    /// player_timed.txt) the Remove Hunger scroll (50) adds 35, more than a
    /// ration's INC_BY 30, and worth neither nothing nor a flat ration.
    #[test]
    fn remove_hunger_is_worth_what_it_adds_to_a_hungry_stomach() {
        let scroll = effect_supply(&[(EF_NOURISH as u16, 3, 50)], 20, 1000);
        assert_eq!(scroll, [(FOOD, 35 * UNIT)]);
        let ration = effect_supply(&[(EF_NOURISH as u16, 0, 30)], 20, 1000);
        assert_eq!(ration, [(FOOD, 30 * UNIT)]);
        assert!(scroll[0].1 > ration[0].1 && scroll[0].1 < 50 * UNIT);
        // a fill-to level at or under the hungry grade adds nothing
        assert!(effect_supply(&[(EF_NOURISH as u16, 3, 15)], 20, 1000).is_empty());
        assert!(effect_supply(&[(EF_NOURISH as u16, 3, 10)], 20, 1000).is_empty());
    }
    #[test]
    fn replenishment_competes_with_existing_and_substitute_stock() {
        let mut m = model();
        m.supplies = vec![[0, 0, ATTACK as i64, UNIT], [1, 0, ATTACK as i64, UNIT]];
        let first = m.ware(0, 0, 1);
        let two_kinds = m.gain(&[(ATTACK, UNIT), (ATTACK, UNIT)]);
        assert!(two_kinds < first * 2, "two kinds do not each claim the full shortage");
        m.curves[ATTACK][3] = 10 * UNIT;
        assert!(m.ware(0, 0, 1) < first);
        m.curves[ATTACK][2] = -UNIT; // one-time access/repair, such as a class book
        assert_eq!(m.ware(0, 0, 1), 0);
    }
    #[test]
    fn light_after_the_reserve_has_positive_diminishing_purchase_value() {
        let mut m = model();
        let torch = 5000 * UNIT;
        let target = 4 * torch;
        m.supplies.push([0, 0, LIGHT as i64, torch]);
        m.curves[LIGHT] = [LIGHT as i64, 100 * UNIT, -target, target - torch, 0];
        let to_reserve = m.ware(0, 0, 1);
        m.curves[LIGHT][3] = target;
        let extra = m.ware(0, 0, 1);
        m.curves[LIGHT][3] += torch;
        let another = m.ware(0, 0, 1);
        assert!(to_reserve > extra && extra > another && another > 0);
    }

    #[test]
    fn fuel_reserve_protects_carried_light_even_when_home_has_enough() {
        let mut m = model();
        let spare = item(Place::Pack, 0, 1);
        let torch_turns = 5000 * UNIT;
        m.supplies.push([PACK, 0, LIGHT as i64, torch_turns]);
        m.curves[LIGHT] = [LIGHT as i64, 100 * UNIT, -2 * torch_turns, 2 * torch_turns, 4 * torch_turns];
        assert!(m.protects_fuel(&spare));
        m.curves[LIGHT][3] += torch_turns;
        assert!(!m.protects_fuel(&spare));
        m.curves[LIGHT][2] = torch_turns;
        m.curves[LIGHT][3] = 2 * torch_turns;
        assert!(!m.protects_fuel(&spare));
    }

    #[test]
    fn exploration_resupply_and_disposal_read_the_same_reserve() {
        let mut m = model();
        m.supplies = vec![[PACK, 1, PHASE_RESERVE as i64, UNIT], [4, 0, PHASE_RESERVE as i64, UNIT]];
        m.curves[PHASE_RESERVE] = [PHASE_RESERVE as i64, UNIT, -2 * UNIT, UNIT, 0];
        assert!(m.keeps_exploration_supply(1, 1));
        assert!(m.supplies_shortfall(4, 0));
        m.curves[PHASE_RESERVE][3] = 3 * UNIT;
        assert!(!m.keeps_exploration_supply(1, 1));
        assert!(m.keeps_exploration_supply(1, 2));
        assert!(!m.supplies_shortfall(4, 0));
    }

    /// #1151: the copies of a fuel stack the light reserve still needs. 34
    /// torches of 5,000 turns against 20,000: four, the rest can go. Other
    /// carried fuel lowers it; a reserve met elsewhere needs none; the whole
    /// stack short of the reserve keeps all of it (what `protects_fuel` says).
    #[test]
    fn the_fuel_reserve_keeps_only_the_copies_it_needs() {
        let mut m = model();
        let torch_turns = 5000 * UNIT;
        m.supplies.push([PACK, 0, LIGHT as i64, torch_turns]);
        let stack = item(Place::Pack, 0, 34);
        m.curves[LIGHT] = [LIGHT as i64, 100 * UNIT, -4 * torch_turns, 34 * torch_turns, 0];
        assert_eq!(m.fuel_keep(&stack), 4);
        assert!(m.protects_fuel(&stack), "the whole stack would undercut the reserve");
        m.curves[LIGHT][3] = 34 * torch_turns + 3 * torch_turns / 2; // a flask's worth carried elsewhere
        assert_eq!(m.fuel_keep(&stack), 3);
        m.curves[LIGHT][3] = 38 * torch_turns;
        assert_eq!(m.fuel_keep(&stack), 0);
        assert!(!m.protects_fuel(&stack));
        let few = item(Place::Pack, 0, 3);
        m.curves[LIGHT][3] = 3 * torch_turns;
        assert_eq!(m.fuel_keep(&few), 3, "short of the reserve: all of it stays");
        m.curves[LIGHT][2] = torch_turns; // no reserve row
        assert_eq!(m.fuel_keep(&stack), 0);
    }

    /// #1151: the copies of a Remove Curse ware read now: what the CURSE row
    /// lacks while it has weight (`BI_FIRST_CURSED`); none without a curse,
    /// none once a carried scroll covers it.
    #[test]
    fn a_remove_curse_ware_is_read_now_only_for_a_curse() {
        let mut m = model();
        m.supplies.push([5, 2, CURSE as i64, UNIT]);
        m.curves[CURSE] = [CURSE as i64, 100 * UNIT, -UNIT, 0, 0];
        assert_eq!(m.decurse_copies(5, 2), 1);
        assert_eq!(m.decurse_copies(5, 3), 0, "another ware");
        m.curves[CURSE][3] = UNIT;
        assert_eq!(m.decurse_copies(5, 2), 0, "a carried scroll covers it");
        m.curves[CURSE] = [CURSE as i64, 0, -UNIT, 0, 0];
        assert_eq!(m.decurse_copies(5, 2), 0, "no curse to remove");
    }

    /// #1151 (review of #1155): a lantern at 5,000 turns and six 7,500-turn
    /// flasks against a 20,000-turn reserve. The model gives two bottles to
    /// refuelling and spreads their fuel over the six (2,500 a copy); the
    /// floor counts the physical fuel, so two flasks stay, not six.
    #[test]
    fn the_light_floor_counts_physical_flasks() {
        let mut m = model();
        let flask = 7500 * UNIT;
        m.supplies.push([PACK, 0, LIGHT as i64, 2 * flask / 6]);
        m.curves[LIGHT] = [LIGHT as i64, 100 * UNIT, -20_000 * UNIT, 5000 * UNIT + 2 * flask, 0];
        let stack = KitItem { fuel: flask, ..item(Place::Pack, 0, 6) };
        assert_eq!(m.fuel_keep(&stack), 2);
        assert_eq!(m.fuel_keep(&KitItem { fuel: 0, ..stack }), 6, "the modeled units alone");
    }

    /// #1151 (review of #1155): every weighted reserve row counts. Six Phase
    /// Door against a reserve of six keep all six; a seventh carried
    /// elsewhere frees one.
    #[test]
    fn the_reserve_floor_keeps_the_phase_door_reserve() {
        let mut m = model();
        m.supplies.push([PACK, 0, PHASE as i64, UNIT]);
        m.supplies.push([PACK, 0, PHASE_RESERVE as i64, UNIT]);
        m.curves[PHASE_RESERVE] = [PHASE_RESERVE as i64, 100 * UNIT, -6 * UNIT, 6 * UNIT, 0];
        let stack = item(Place::Pack, 0, 6);
        assert_eq!(m.reserve_keep(&stack), 6);
        assert_eq!(m.fuel_keep(&stack), 0, "no light row");
        m.curves[PHASE_RESERVE][3] = 7 * UNIT;
        assert_eq!(m.reserve_keep(&stack), 5);
        m.curves[PHASE_RESERVE][1] = 0;
        assert_eq!(m.reserve_keep(&stack), 0, "an unweighted row reserves nothing");
    }

    /// #1151: an overweight Borg with a curse to remove still buys a Remove
    /// Curse scroll (schema 20): it is read in town, so the weight repair
    /// does not take it away. Schema 19 left without it.
    #[test]
    fn an_overweight_borg_still_buys_the_remove_curse_it_reads_in_town() {
        use crate::borg_town_record::{action, first_action, step, StepInput, PARAMS};
        use crate::borg_town_solve::Limits;
        use crate::borg_town_stock::Candidate;
        let scroll = Candidate { shop: 4, slot: 0, kind: 1, tval: crate::headers::list_tvals_h::TV_SCROLL as u8, sval: 1,
            price: 100, qty: 5, weight: 5, slots_cost: 1, stack_count: 0, class: 1 };
        let decide_with = |revision: i64, weight: i64, curse: bool, unreadable: bool| {
            let mut m = Model { available: vec![[4, 0, 5]], packing: None,
                curves: (0..COUNT).map(|i| [i as i64, 0, UNIT, 0, 0]).collect(), supplies: vec![[4, 0, CURSE as i64, UNIT]] };
            m.curves[CURSE] = [CURSE as i64, if curse { 100 * UNIT } else { 0 }, -UNIT, 0, 0];
            let mut traits = [0; BI_MAX as usize];
            traits[BI_CARRY as usize] = 100;
            let input = StepInput { departure: false, shop_overloaded: false, storing: None, revision, capability: Some(&m), utility: true,
                black_market_reserve: 0, black_market_percentage: -1, candidates: vec![scroll.clone()],
                facts: &[], tryon: &[], limits: Limits { gold: 89_159, pack_slots: 4, weight },
                urgent: vec![], params: [0; PARAMS], traits: &traits, power: 0, carried: &[],
                cure_spell: false, reads_now: !unreadable, kit: &[], home_slots: 0 };
            let output = step(&input);
            let (code, at, _, _) = first_action(&input.candidates, &input.urgent, &output, input.facts, input.kit);
            (code, at[2])
        };
        let decide = |revision: i64, weight: i64, curse: bool| decide_with(revision, weight, curse, false).0;
        assert_eq!(decide_with(20, -993, true, false), (action::BUY, 1), "overweight, read in town: the copy read now");
        assert_eq!(decide(20, -993, true), action::BUY);
        assert_eq!(decide_with(20, -993, true, true).0, action::LEAVE,
            "town enchanting would not read it (blind, confused, amnesiac, dark, or its town timer run out): weighed");
        assert_eq!(decide(19, -993, true), action::LEAVE, "the recorded visit's weight repair");
        assert_eq!(decide(20, 100, true), action::BUY);
        assert_eq!(decide(20, -993, false), action::LEAVE, "no curse to remove");
    }

    #[test]
    fn keeping_and_buying_use_identical_units_and_home_withdrawals_reprice() {
        let mut m = model();
        m.supplies = vec![[PACK, 0, ATTACK as i64, UNIT], [HOME, 0, ATTACK as i64, UNIT]];
        let pack = item(Place::Pack, 0, 1);
        let home = item(Place::Home, 0, 1);
        m.curves[ATTACK][3] = UNIT;
        let loss = m.keep(&pack);
        m.change(&pack, -1);
        assert_eq!(loss, m.take(&home));
        let first = m.take(&home);
        m.change(&home, 1);
        assert!(m.take(&home) < first);
        m.curves[ATTACK][4] = UNIT;
        assert!(m.reserve(&home, -1, 500) < 0);
    }
    #[test]
    fn selling_or_storing_the_recharge_target_removes_the_purchase_demand() {
        let mut m = model();
        m.curves[RECHARGE_TARGET][1] = 0;
        m.curves[RECHARGE_TARGET][3] = UNIT;
        m.supplies.push([PACK, 0, RECHARGE_TARGET as i64, UNIT]);
        assert!(m.gain(&[(RECHARGE, UNIT)]) > 0);
        m.change(&item(Place::Pack, 0, 1), -1);
        assert_eq!(m.gain(&[(RECHARGE, UNIT)]), 0);
    }
    /// #1004: the look-ahead keeps a first book whose spell comes in a few
    /// levels; 0 is today's test (castable now).
    #[test]
    fn book_look_ahead_counts_spells_a_few_levels_away() {
        let spec = crate::params::SPECS.iter().find(|s| s.name == "town_book_lookahead_levels").unwrap();
        assert_eq!((spec.default, spec.min, spec.max), (0, 0, 50));
        // today's test: castable now only
        assert!(book_spell_counts(1, 1, 1, 1, 0));
        assert!(!book_spell_counts(1, 2, 1, 1, 0), "mana above the max SP");
        // Ranger, Remove Hunger L3, at CL1 with 0 SP
        assert!(!book_spell_counts(3, 1, 1, 0, 0));
        assert!(book_spell_counts(3, 1, 1, 0, 2));
        assert!(!book_spell_counts(3, 1, 1, 0, 1));
        // Rogue, Detect Monsters L5, at CL1
        assert!(book_spell_counts(5, 1, 1, 0, 4));
        assert!(!book_spell_counts(5, 1, 1, 0, 3));
        // Paladin, Bless L1 2 SP, at CL1 with 1 SP (Half-Elf, Elf, Half-Troll)
        assert!(!book_spell_counts(1, 2, 1, 1, 0));
        assert!(book_spell_counts(1, 2, 1, 1, 4));
    }

    /// #1004: the look-ahead applies to owned books only, never to a shop's,
    /// so purchases are unchanged (review of #1019, finding 3).
    #[test]
    fn book_look_ahead_is_for_owned_books_only() {
        assert_eq!(book_lookahead(PACK, 4), 4);
        assert_eq!(book_lookahead(HOME, 4), 4);
        for shop in 0..=7 {
            assert_eq!(book_lookahead(shop, 4), 0, "shop scope {shop}");
        }
        assert_eq!(book_lookahead(PACK, 0), 0);
    }

    #[test]
    fn owning_a_usable_class_book_closes_access_without_a_duplicate_stock_target() {
        let mut m = model();
        m.curves[BOOKS][2] = -UNIT;
        m.supplies.push([4, 0, BOOKS as i64, UNIT]);
        assert!(m.ware(4, 0, 1) > 0);
        assert_eq!(m.ware(4, 0, 2), m.ware(4, 0, 1));
        m.curves[BOOKS][3] = UNIT;
        assert_eq!(m.ware(4, 0, 1), 0);
    }
    #[test]
    fn temporary_sight_and_snapshot_detection_share_time_units() {
        assert_eq!(effect_supply(&[(EF_DETECT_INVISIBLE_MONSTERS as u16, 0, 0)], 20, 1000), [(INVISIBLE, 1000 * UNIT)]);
        assert_eq!(effect_supply(&[(EF_TIMED_INC as u16, TMD_SINVIS as i32, 30)], 20, 1000), [(INVISIBLE, 30 * UNIT)]);
    }
    #[test]
    fn schema_seven_accepts_its_earlier_append_only_table() {
        let mut m = model();
        m.curves.truncate(53);
        assert!(m.valid());
        assert!(m.gain(&[(ATTACK, UNIT)]) > 0);
    }
    #[test]
    fn oil_is_allocated_to_fuel_or_ammunition_without_counting_a_bottle_twice() {
        assert_eq!(oil_supply(1, 1, 2500, 7500), [(LIGHT, 7500 * UNIT)]);
        // #649: a flask of oil (1d4, weight 20, exploding) does 22.5, as the engine.
        assert_eq!(thrown_damage(1, 4, 20, true), 22_500);
        assert_eq!(thrown_damage(1, 4, 5, false), 5_000, "an Iron Shot thrown by hand");
        assert_eq!(thrown_damage(1, 2, 4, false), 3_000, "a Rounded Pebble thrown by hand");
        assert_eq!(oil_supply(1, 0, 2500, 7500), [(ATTACK, 2500)]);
        let held = oil_supply(5, 2, 2500, 7500);
        assert_eq!(held, [(LIGHT, 3000 * UNIT), (ATTACK, 1500)]);
        assert_eq!(held[0].1 * 5 / (7500 * UNIT) + held[1].1 * 5 / 2500, 5);
    }
    #[test]
    fn malformed_replay_facts_cannot_index_outside_the_capability_table() {
        let mut m = model();
        assert!(m.valid());
        m.supplies.push([0, 0, COUNT as i64, UNIT]);
        assert!(!m.valid());
        m.supplies[0][2] = 0;
        m.supplies[0][3] = -UNIT;
        assert!(!m.valid());
    }
    #[test]
    fn unsupported_preparation_and_harmful_projections_have_no_combat_value() {
        use crate::headers::list_projections_h::*;
        assert!(effect_supply(&[(EF_TIMED_INC as u16,TMD_ATT_CONF as i32,10000)],20,1000).is_empty());
        for (e,sub) in [(EF_SPHERE,PROJ_DARK_WEAK),(EF_TOUCH,PROJ_MAKE_TRAP)] {
            assert!(effect_supply(&[(e as u16,sub as i32,20)],20,1000).is_empty());
        }
        assert_eq!(effect_supply(&[(EF_BOLT_STATUS as u16,PROJ_AWAY_ALL as i32,100)],20,1000),[(OTHER,UNIT)]);
    }
    #[test]
    fn m13_confusion_wand_has_a_consumer_but_the_staff_does_not() {
        use crate::headers::list_projections_h::PROJ_MON_CONF;
        let effect=[(EF_BOLT_AWARE as u16,PROJ_MON_CONF as i32,20)];
        assert_eq!(effect_supply_for_consumer(&effect,20,1000,true),[(CONTROL,UNIT)]);
        assert!(effect_supply_for_consumer(&effect,20,1000,false).is_empty());
        let alternative=[(EF_RANDOM as u16,0,2),effect[0],(EF_BOLT_AWARE as u16,crate::headers::list_projections_h::PROJ_MON_STUN as i32,20)];
        assert!(effect_supply_for_consumer(&alternative,20,1000,true).is_empty(),"unsupported random alternatives do not acquire guaranteed control");
    }

    #[test]
    fn shop_control_requires_a_working_consumer() {
        use crate::headers::list_projections_h::*;
        for effect in [EF_BOLT_AWARE, EF_PROJECT_LOS_AWARE] {
            for projection in [PROJ_MON_CONF, PROJ_MON_STUN, PROJ_SLEEP_UNDEAD,
                PROJ_TURN_EVIL, PROJ_TURN_LIVING, PROJ_TURN_UNDEAD] {
                assert!(effect_supply(&[(effect as u16,projection as i32,20)],20,1000).is_empty());
            }
            for projection in [PROJ_MON_SLOW, PROJ_MON_HOLD, PROJ_SLEEP_ALL, PROJ_SLEEP_EVIL, PROJ_TURN_ALL] {
                assert_eq!(effect_supply(&[(effect as u16,projection as i32,20)],20,1000),[(CONTROL,UNIT)]);
            }
        }
    }

    #[test]
    fn mutually_exclusive_damage_branches_are_not_added() {
        use crate::headers::list_elements_h::{PROJ_FIRE,PROJ_COLD};
        for e in [EF_RANDOM,EF_SELECT] {
            assert_eq!(effect_supply(&[(e as u16,0,2),(EF_BALL as u16,PROJ_FIRE as i32,80),
                (EF_BALL as u16,PROJ_COLD as i32,80)],20,1000),[(ATTACK,80*UNIT)]);
            assert!(effect_supply(&[(e as u16,0,2),(EF_HEAL_HP as u16,0,20),
                (EF_BALL as u16,PROJ_FIRE as i32,80)],20,1000).is_empty(),"no guaranteed shared service");
        }
    }
    #[test]
    fn partial_timed_reductions_leave_value_for_another_dose() {
        let mut m=model();m.curves[CONFUSION]=[CONFUSION as i64,100*UNIT,-60*UNIT,0,0];
        let partial=effect_supply(&[(EF_TIMED_DEC as u16,TMD_CONFUSED as i32,20)],20,1000);
        assert_eq!(partial,[(CONFUSION,20*UNIT)]);
        let first=m.gain(&partial);m.curves[CONFUSION][3]+=20*UNIT;
        assert!(first>0 && m.gain(&partial)>0);
        m.curves[CONFUSION][3]=60*UNIT;assert_eq!(m.gain(&partial),0);
    }
    #[test]
    fn stacked_rods_keep_the_ready_rod_and_respect_charging_count() {
        assert_eq!(rod_uses(2,40,40,10),1);
        assert_eq!(rod_uses(2,40,40,20),1);
        assert_eq!(rod_uses(2,80,40,10),0);
        assert_eq!(rod_uses(2,0,40,2),2);
        assert_eq!(rod_uses(1,0,40,41),2);
    }
    #[test]
    fn wearable_and_fuel_share_one_curve_and_keep_slot_exclusivity() {
        use crate::borg_town_stock::Candidate;
        use crate::borg_town_solve::{Choice,Group,GroupKey,Limits};
        let cs:Vec<_>=(0..3).map(|i|Candidate {shop:0,slot:i,kind:u32::from(i)+1,tval:0,sval:0,
            price:1,qty:1,weight:0,slots_cost:1,stack_count:0,class:u16::from(i)}).collect();
        let mut m=model();m.supplies=(0..3).map(|i|[0,i,LIGHT as i64,UNIT]).collect();
        let choice=|i|Choice {picks:vec![(i,1)],cost:1,value:m.ware(0,i as u8,1),slots:1,weight:0};
        let groups=vec![Group {key:GroupKey::Slot(5),options:vec![Choice::default(),choice(0),choice(1)]},
            Group {key:GroupKey::Class(2),options:vec![Choice::default(),choice(2)]}];
        let limits=Limits {gold:100,pack_slots:5,weight:100};
        let input=crate::borg_town_record::StepInput { departure: false, shop_overloaded: false,storing:None,revision:9,capability:Some(&m),utility:true,black_market_reserve:0,black_market_percentage:-1,candidates:cs.clone(),facts:&[],tryon:&[],limits,
            urgent:vec![],params:[0;crate::borg_town_record::PARAMS],traits:&[],power:0,carried:&[],cure_spell:false,reads_now:false,kit:&[],home_slots:10};
        let merged=shared_groups(groups,&input,&cs,&m,&limits,(0,100,0));assert_eq!(merged.len(),1);
        for option in &merged[0].options {
            assert!(option.picks.iter().filter(|&&(i,_)|i<2).count()<=1);
            assert_eq!(option.value,m.gain(&[(LIGHT,option.picks.len() as i64*UNIT)]));
        }
        assert!(merged[0].options.iter().any(|o|o.picks.len()==2));
    }
    #[test]
    fn home_sale_changes_the_next_reserve_marginal() {
        let mut m=model();let home=item(Place::Home,0,1);m.supplies=vec![[HOME,0,ATTACK as i64,UNIT]];
        m.curves[ATTACK][4]=2*UNIT;let first=m.reserve(&home,-1,500);
        m.change_home(&home,-1);let second=m.reserve(&home,-1,500);
        assert!(second<first,"the final remaining reserve is more valuable");
    }

    /// #491: a healing supply fills the reserve too; below the reserve each
    /// potion adds the same value on top of the diminishing curve, past it
    /// only the curve's.
    #[test]
    fn healing_reserve_is_linear_up_to_its_size_then_the_curve() {
        let hp = 40;
        let units = effect_supply_linear(&[(EF_HEAL_HP as u16, 0, 20)], hp, 1, false);
        assert!(units.contains(&(HEAL, 20 * UNIT)) && units.contains(&(HEAL_RESERVE, 20 * UNIT)));
        let mut m = model();
        // Healing curve: weight 100 per HP of demand 1000 HP; reserve of 3
        // potions (60 HP) at the curve's first-HP slope (weight / demand).
        m.curves[HEAL] = [HEAL as i64, 100 * UNIT, 1000 * UNIT, 0, 0];
        let reserve = 3 * 20 * UNIT;
        m.curves[HEAL_RESERVE] = [HEAL_RESERVE as i64, 100 * UNIT * reserve / (1000 * UNIT), -reserve, 0, 0];
        m.supplies.push([0, 0, HEAL as i64, 20 * UNIT]);
        m.supplies.push([0, 0, HEAL_RESERVE as i64, 20 * UNIT]);
        let curve_only = |m: &Model| coverage(&m.curves[HEAL], m.curves[HEAL][3] + 20 * UNIT) - coverage(&m.curves[HEAL], m.curves[HEAL][3]);
        let mut extra = Vec::new();
        for _ in 0..4 {
            extra.push(m.ware(0, 0, 1) - curve_only(&m));
            m.curves[HEAL][3] += 20 * UNIT;
            m.curves[HEAL_RESERVE][3] += 20 * UNIT;
        }
        assert!(extra[0] > 0 && extra[0] == extra[1] && extra[1] == extra[2], "{extra:?}");
        assert_eq!(extra[3], 0, "nothing past the reserve");
        // Off: zero weight adds nothing.
        m.curves[HEAL_RESERVE][1] = 0;
        assert_eq!(m.ware(0, 0, 1), curve_only(&m));
    }

    /// #491: healing speed. At 66%, Cure Light Wounds (20 HP) counts in full
    /// up to 30 max HP and less above; one that heals the whole of max HP
    /// always counts in full; 0 is HP alone.
    #[test]
    fn healing_units_weigh_the_share_of_max_hp_one_swallow_heals() {
        assert_eq!(heal_units(20, 30, 66), 20 * UNIT);
        assert!(heal_units(20, 31, 66) < 20 * UNIT);
        assert_eq!(heal_units(20, 300, 66), 20 * 20 * UNIT * 100 / (300 * 66));
        assert_eq!(heal_units(60, 300, 66), 60 * 60 * UNIT * 100 / (300 * 66));
        assert_eq!(heal_units(300, 300, 66), 300 * UNIT);
        assert_eq!(heal_units(1200, 300, 66), 300 * UNIT);
        assert_eq!(heal_units(20, 300, 0), 20 * UNIT);
        assert_eq!(heal_units(20, 10, 66), 10 * UNIT);
        // #487: at 108 max HP a Cure Light Wounds is ~5.6 HP on the curve,
        // but fills the reserve with its 20 real HP.
        let cure_light = heal_units(20, 108, 66);
        assert!(cure_light < 6 * UNIT);
        assert_eq!(reserve_heal(cure_light, heal_units(20, 108, 0), true), 20 * UNIT);
        assert_eq!(reserve_heal(cure_light, heal_units(20, 108, 0), false), cure_light);
    }
}

#[cfg(test)]
mod bold_is_not_a_buff_tests {
    use super::*;

    /// #486: a Bold timed effect (wine's) supplies nothing, while a fear
    /// cure (a Potion of Boldness, CURE:AFRAID) still supplies FEAR.
    #[test]
    fn a_bold_effect_supplies_nothing_and_a_fear_cure_still_supplies_fear() {
        let wine = effect_supply(&[(EF_TIMED_INC as u16, TMD_BOLD as i32, 99)], 20, 1000);
        assert!(wine.iter().all(|&(cap, _)| cap != BUFF), "{wine:?}");
        let boldness = effect_supply(&[(EF_CURE as u16, TMD_AFRAID as i32, 0)], 20, 1000);
        assert!(boldness.iter().any(|&(cap, units)| cap == FEAR && units > 0), "{boldness:?}");
    }

    /// #486: Boldness is no buff to the mapping; the buffs the core does
    /// take stay.
    #[test]
    fn bold_is_not_a_buff() {
        assert_eq!(timed_cap(TMD_BOLD), None);
        assert_eq!(timed_cap(TMD_HERO), Some(BUFF));
        assert_eq!(timed_cap(TMD_SHERO), Some(BUFF));
        assert_eq!(timed_cap(TMD_BLESSED), Some(BUFF));
    }
}

#[cfg(test)]
mod class_heal_tests {
    use super::*;

    /// #487: with no class override (the compiled defaults) every class
    /// reads the general scale and reserve.
    #[test]
    fn class_overrides_fall_back_to_the_general_values() {
        for class in 0..9 {
            assert_eq!(heal_curve_scale_percent(class), param(Param::town_heal_curve_scale_percent));
            assert_eq!(heal_reserve_hp_percent(class), param(Param::town_heal_reserve_hp_percent));
        }
        assert_eq!(heal_curve_scale_percent(99), param(Param::town_heal_curve_scale_percent));
        assert_eq!(heal_reserve_hp_percent(99), param(Param::town_heal_reserve_hp_percent));
    }

    #[test]
    fn shots_for_every_class_by_its_percent() {
        // #666: -1 keeps C (the Ranger only); a percent applies to any class.
        assert_eq!(missiles_percent(CLASS_RANGER, -1), 100);
        assert_eq!(missiles_percent(0, -1), 0);
        assert_eq!(missiles_percent(0, 75), 75);
        assert_eq!(missiles_percent(CLASS_RANGER, 100), 100);
        // Fewer shots wanted with flasks or attack-wand charges in the pack.
        assert_eq!(missiles_other_ranged(1000, 4, 50), 1000);
        assert_eq!(missiles_other_ranged(1000, 5, 50), 500);
        assert_eq!(missiles_other_ranged(1000, 20, 100), 1000);
    }

    #[test]
    fn detect_invisible_is_a_diminishing_curve_not_a_reserve() {
        // #831: off, #663's low value stands; on, the first read is worth
        // `first` * half / (half + 1), each next one less, none a requirement.
        assert_eq!(invisible_curve(0, 500, 1000), None);
        let (weight, demand) = invisible_curve(2, 500, 1000).unwrap();
        assert_eq!((weight, demand), (1000, 2000));
        let row = [INVISIBLE as i64, weight, demand, 0, 0];
        let (one, two, three) = (coverage(&row, 1000), coverage(&row, 2000), coverage(&row, 3000));
        assert_eq!(two, 500, "half of the most it pays, at `half` reads");
        assert!(one > two - one && two - one > three - two, "diminishing: {one} {two} {three}");
        assert!(coverage(&row, 9000) < weight, "never the whole: it is no requirement");
    }

    #[test]
    fn what_the_borg_never_buys_keeps_a_small_value() {
        // #663: one percent, above zero so a carried copy is kept.
        assert_eq!(low_value(55_000), 550);
        assert!(low_value(55_000) > 0);
        let scroll = coverage(&[MAP as i64, low_value(55_000), 1_100_000, 0, 0], 1_000_000);
        let phase = coverage(&[PHASE as i64, 55_000, 1_100, 0, 0], 1_000);
        assert!(scroll * 50 < phase, "{scroll} against a Phase Door's {phase}");
    }
}

/// borg-reborn-rs (#831): the curve's first value per read, in percent of the
/// trip's escape weight, by what the Borg can already do about an invisible
/// monster: nothing (Warriors, Blackguards, and others until they learn a
/// spell), cast Detect Evil (reveals the evil ones, ghosts and poltergeists,
/// not a clear icky thing), or cast a spell that reveals invisible monsters.
fn invisible_percent(st: &BorgState) -> i32 {
    param(if crate::borg_magic::detects_invisible(st) { Param::town_invisible_spell_percent }
        else if crate::borg_magic::detects_evil(st) { Param::town_invisible_evil_percent }
        else { Param::town_invisible_percent })
}

/// borg-reborn-rs (#831): the Detect Invisible curve: `first` is the value of
/// the first read, `half` reads (each `service` units of INVISIBLE supply)
/// give half of the most the curve can pay (`first * half`): the diminishing
/// shape of Phase Door and the flasks, so a second and a third read are worth
/// less than the first and no number of them is a requirement. `None` with
/// no `half`: the curve stays #663's low value.
fn invisible_curve(half: i64, first: i64, service: i64) -> Option<(i64, i64)> {
    (half > 0).then(|| (first * half, half * service))
}

/// borg-reborn-rs (#663, owner): the weight of what the Borg never buys
/// (Magic Mapping, Detect Invisible, Light, a fear cure while not afraid): one
/// percent of the ordinary weight, so a copy it carries still counts.
fn low_value(ordinary: i64) -> i64 {
    ordinary / 100
}

/// borg-reborn-rs (#557, owner): a Word of Recall's value (the walk it
/// saves) times `town_recall_value_percent` / 100.
pub(crate) fn recall_value_scaled(value: i64) -> i64 {
    scale_percent(value, param(Param::town_recall_value_percent))
}

/// borg-reborn-rs (#557, #859): the RECALL curve's weight and demand, and the
/// RECALL_RESERVE row. `scale` 0 is #557: with a reserve, a requirement
/// (negative demand) worth the full trip weight a scroll and nothing past
/// it; without, the walk a scroll saves on a one-scroll diminishing curve.
/// `scale` > 0 (#859, owner: "use the trip weight"): every scroll on a
/// diminishing curve on the trip weight, weight and demand scaled together
/// as the healing and Phase Door curves are (each scroll worth less than the
/// last, never nothing), and the reserve a requirement on top of it.
fn recall_curve(trip_weight: i64, walk_value: i64, reserve: i64, scale: i32) -> ((i64, i64), Option<(i64, i64)>) {
    if scale > 0 {
        let reserve_row = (reserve > 0).then(|| (trip_weight * reserve, -(reserve * UNIT)));
        return ((scale_percent(trip_weight, scale), scale_percent(UNIT, scale)), reserve_row.or(Some((0, UNIT))));
    }
    if reserve > 0 { ((trip_weight * reserve, -(reserve * UNIT)), None) } else { ((walk_value, UNIT), None) }
}

/// borg-reborn-rs (#859): the room the Word of Recall reserve still needs in
/// the pack: the copies short of `reserve` (whole scrolls, the carried ones
/// counted on the RECALL curve), times the lightest copy on offer, and a slot
/// when none is carried. Nothing when no offered ware supplies Recall.
pub(crate) fn recall_room(m: &Model, reserve: i64, candidates: &[crate::borg_town_stock::Candidate]) -> (i64, i32) {
    let Some(row) = m.curves.get(RECALL) else { return (0, 0) };
    let short = reserve - row[3] / UNIT;
    if reserve <= 0 || short <= 0 { return (0, 0); }
    let lightest = candidates.iter()
        .filter(|c| c.qty > 0 && m.units(i64::from(c.shop), i64::from(c.slot), 1).iter().any(|&(id, _)| id == RECALL))
        .map(|c| i64::from(c.weight)).min();
    lightest.map_or((0, 0), |w| (short * w, i32::from(row[3] == 0)))
}

/// borg-reborn-rs (#863): whether the prepared rows exist (`town_prepared_percent`
/// above 0); off, the model has the shape it had before.
/// borg-reborn-rs (#863, owner: "it is important that in the first level we
/// have the staples"): the variety logic (the staples' decay and the prepared
/// remedies) starts once the Borg has been `town_variety_min_depth` deep. Before
/// it the first purchases are the staples' alone: healing, escapes, oil, food,
/// light, missiles and Recall, as before #863. Constant within a town visit.
fn variety_started(st: &BorgState) -> bool {
    st.borg.tr(BI_MAXDEPTH) >= param(Param::town_variety_min_depth)
}

/// The restoring-mushroom reserve applies during town planning from its depth
/// threshold, while there is no current stat drain. Out-of-depth stat drains
/// and immediate restoration are handled by the ordinary stat rows.
fn restore_stat_on(st: &BorgState) -> bool {
    variety_started(st) && param(Param::town_restore_stat_percent) > 0
        && st.borg.tr(BI_MAXDEPTH) >= param(Param::town_restore_stat_min_depth)
        && (0..5).all(|stat| st.borg.tr(BI_ISFIXSTR + stat) == 0)
}

/// borg-reborn-rs (#895): device rows apply once variety starts.
fn device_on(st: &BorgState) -> bool { variety_started(st) && param(Param::town_device_percent) > 0 }

/// borg-reborn-rs (#895): a device row's weight and demand: a requirement of
/// `charges` charges worth `percent` of the trip weight together, each an Nth.
fn device_row(trip_weight: i64, percent: i32, charges: i32) -> (i64, i64) {
    (scale_percent(trip_weight, percent), -i64::from(charges.max(1)) * UNIT)
}

/// borg-reborn-rs (#895, review of #898 F6): whether a device does damage
/// beyond light. A Wand of Light's `LINE:LIGHT_WEAK` hurts light-sensitive
/// monsters only, so its charges do not fill the ranged attack row; the shared
/// ATTACK curve still counts them as before.
fn device_attack(effects: &[(u16, i32, i64)], hp: i64, service_turns: i64, confusion: bool) -> bool {
    use crate::headers::list_projections_h::PROJ_LIGHT_WEAK;
    let kept: Vec<(u16, i32, i64)> = effects.iter().filter(|&&(_, subtype, _)| subtype as u32 != PROJ_LIGHT_WEAK).copied().collect();
    effect_supply_for_consumer(&kept, hp, service_turns, confusion).iter().any(|&(id, units)| id == ATTACK && units > 0)
}

/// Damage that can cover general threats, leaving niche light or dispels
/// to the existing attack scorer's monster-specific evaluation.
pub(crate) fn ranged_effect_damage(effects: &[(u16, i32, i64)], hp: i64) -> i64 {
    use crate::headers::list_projections_h::{PROJ_DISP_EVIL, PROJ_DISP_UNDEAD, PROJ_LIGHT_WEAK};
    let general: Vec<_> = effects.iter().copied().filter(|&(_, projection, _)| {
        !matches!(projection as u32, PROJ_LIGHT_WEAK | PROJ_DISP_EVIL | PROJ_DISP_UNDEAD)
    }).collect();
    effect_supply_for_consumer(&general, hp, 1, false).into_iter()
        .find_map(|(id, damage)| (id == ATTACK).then_some(damage / UNIT)).unwrap_or(0)
}

/// borg-reborn-rs (#895): a ware's supplies without what an escape staff gives:
/// the shared TELEPORT capability and the device row.
fn without_escape(supply: Vec<(usize, i64)>) -> Vec<(usize, i64)> {
    supply.into_iter().filter(|&(id, _)| id != TELEPORT && id != DEV_ESCAPE).collect()
}

/// borg-reborn-rs (#895): the charges of Staffs of Teleportation in the pack, in `UNIT`s.
fn carried_escape_charges(st: &BorgState) -> i64 {
    st.items.items.iter()
        .filter(|i| i.iqty > 0 && i.aware && u32::from(i.tval) == TV_STAFF && i32::from(i.sval) == st.item_val.sv_staff_teleportation)
        .map(|i| i64::from(i.pval).max(0) * UNIT).sum()
}

/// borg-reborn-rs (#895): whether a Staff of Teleportation on offer is switched off for a
/// small purse: below the `floor` (on the purse, not the price) and with none carried (`carried` charges).
fn escape_gated(floor: i64, gold: i64, carried: i64) -> bool { floor > 0 && gold < floor && carried == 0 }

/// borg-reborn-rs (#863): the staple rows exist when any staple has a knee.
fn staple_on(st: &BorgState) -> bool {
    variety_started(st) && [Param::town_staple_clw, Param::town_staple_csw, Param::town_staple_ccw, Param::town_staple_phase,
     Param::town_staple_oil, Param::town_staple_food, Param::town_staple_light].into_iter().any(|p| param(p) > 0)
}

/// borg-reborn-rs (#863): whether the prepared rows exist (`town_prepared_percent`
/// above 0, the variety started); off, the model has the shape it had before.
fn prepared_on(st: &BorgState) -> bool { variety_started(st) && param(Param::town_prepared_percent) > 0 }

/// borg-reborn-rs (#1134): whether the Cure Critical Wounds row exists.
fn prepared_ccw_on(st: &BorgState) -> bool { prepared_on(st) && param(Param::town_prepared_ccw) > 0 }

/// borg-reborn-rs (#1137): whether the Neutralize Poison row exists.
fn prepared_neutralize_on(st: &BorgState) -> bool { prepared_on(st) && param(Param::town_prepared_neutralize) > 0 }

/// borg-reborn-rs (#1137): whether the dear rows count at this decision: no
/// gate, or `gold` (after the urgent needs) at the Recall floor and the
/// Recall reserve (at least one scroll) already `carried`.
fn dear_rows_paid(after_recall: bool, gold: i64, recall_floor: i64, carried: i64, reserve: i64) -> bool {
    !after_recall || (gold >= recall_floor && carried >= reserve.max(1))
}

/// borg-reborn-rs (#863): a prepared row's weight and demand. A requirement
/// (negative demand, as Phase Door's reserve): each of the first `copies`
/// copies is worth `percent` of the trip weight, a copy past them nothing, so
/// a cheap remedy (an 8 gold Potion of Boldness) is bought to its target and
/// no further (the diminishing curve it replaced let it pile up to 8).
fn prepared_row(trip_weight: i64, percent: i32, copies: i32) -> (i64, i64) {
    let copies = i64::from(copies.max(1));
    (scale_percent(trip_weight, percent) * copies, -copies * UNIT)
}

/// borg-reborn-rs (#987): the healing potions the first-cure row reserves
/// (`town_first_cure`, 0 = off).
fn first_cure_count() -> i64 { i64::from(param(Param::town_first_cure)).max(0) }

/// The food row's (weight, demand): the trip's rations (in UNITs), never
/// below the five-ration floor, both scaled by `percent` (#1110: more while
/// breadth-first exploration applies; 100 = today). Below the floor, food is
/// a requirement (negative demand).
fn food_curve(w_food: i64, rations: i64, percent: i32) -> (i64, i64) {
    let rations = scale_percent(rations, percent);
    let ready = scale_percent(READY_FOOD * UNIT, percent);
    if rations < ready {
        (w_food * ready, -(RATION_NUTRITION * ready))
    } else {
        (w_food * rations, RATION_NUTRITION * rations)
    }
}

fn scale_percent(value: i64, percent: i32) -> i64 {
    value.saturating_mul(i64::from(percent.max(0))) / 100
}

/// Only value the reserve when the purse after urgent purchases reaches the
/// floor. A small purse still gets the ordinary Recall curve.
pub(crate) fn recall_reserve_for_budget(gold: i64) -> i64 {
    eligible_recall_reserve(gold, crate::borg_store_buy::recall_purchase_floor(), param(Param::town_recall_reserve))
}

pub(crate) fn eligible_recall_reserve(gold: i64, minimum: i32, reserve: i32) -> i64 {
    if gold < i64::from(minimum) { 0 } else { i64::from(reserve) }
}

#[cfg(test)]
mod recall_value_tests {
    /// #1103: a Warrior buys Brawn and Toughness for the Borg to drink, not
    /// Intellect or Contemplation; a Mage the reverse (`borg_junk`).
    #[test]
    fn an_undrained_stat_below_the_cap_counts_toward_a_restoring_mushroom_reserve() {
        use super::restore_reserved as r;
        assert!(r(true, 17) && r(true, 117), "below 18/100");
        assert!(!r(true, 118), "at 18/100 nothing is left to restore");
        assert!(!r(false, 10), "a kind the Borg does not eat as a restoration");
    }

    /// #1371: the engine's stat arithmetic (player.c:player_stat_inc and
    /// player_stat_dec) on the table scale.
    #[test]
    fn stat_gain_and_loss_follow_the_engine_in_table_tenths() {
        use super::{stat_gain_tenths as gain, stat_loss_tenths as loss, stat_tenths as t};
        assert_eq!((t(3), t(17), t(18), t(18 + 50), t(18 + 100)), (0, 140, 150, 200, 250));
        assert_eq!(gain(16), (10, 1), "a point below 18 is a table step");
        // 18/50: g = 14, the rolls 1..14 each add r + 7 points.
        assert_eq!(gain(18 + 50), ((1..=14).map(|r| r + 7).sum(), 14));
        assert_eq!(gain(18 + 95), (5, 1), "18/90 and above become 18/100");
        assert_eq!(gain(18 + 100), (0, 1), "nothing past 18/100");
        assert_eq!((loss(18 + 50), loss(18 + 5), loss(15), loss(3)), (10, 5, 10, 0));
    }

    fn kinds() -> crate::borg_item_val::ItemValState {
        let mut iv = crate::borg_item_val::ItemValState::default();
        (iv.sv_potion_inc_str2, iv.sv_potion_inc_int2, iv.sv_potion_inc_wis2, iv.sv_potion_inc_dex2, iv.sv_potion_inc_con2)
            = (40, 41, 42, 43, 44);
        (iv.sv_potion_cure_light, iv.sv_light_torch, iv.sv_scroll_light) = (8, 2, 17);
        iv
    }

    /// #1371 (owner: "a Ranger would choose to drink Nimbleness"): the
    /// Ranger's trade at its own permanent stats, with the shipped neutral
    /// weights.
    #[test]
    fn a_ranger_drinks_nimbleness_only_while_the_dexterity_gain_outweighs_the_loss() {
        use super::{second_stat_kind_of, second_stat_potion_wanted_at as wanted, stat_trade_worth, CLASS_RANGER};
        // Strength 18/40, Intelligence 14, Wisdom 18/20, Dexterity 18/50,
        // Constitution 18/30: 14.5 tenths expected against 10.
        let stats = [18 + 40, 14, 18 + 20, 18 + 50, 18 + 30];
        assert_eq!(second_stat_kind_of(&kinds(), 43), Some(3));
        assert!(wanted(CLASS_RANGER, stats, 3), "worthwhile: Nimbleness");
        assert!(!wanted(CLASS_RANGER, stats, 2), "Contemplation: the class table refuses it");
        assert!(!wanted(CLASS_RANGER, [18 + 40, 14, 18 + 20, 18 + 100, 18 + 30], 3), "Dexterity capped");
        assert!(!wanted(CLASS_RANGER, [18 + 40, 14, 18 + 20, 18 + 95, 18 + 30], 3),
            "5 tenths to 18/100 against a table step lost");
        // The class's own weights: Strength, Wisdom (its casting stat) and
        // Constitution at twice a Dexterity step outweigh the gain at 18/50.
        assert!(stat_trade_worth(stats, 3, [100; 5], 100));
        assert!(!stat_trade_worth(stats, 3, [200, 100, 200, 100, 200], 100));
        assert!(!stat_trade_worth(stats, 3, [100; 5], 150), "a margin asks for more gain");
    }

    /// Review of #1372: the rule reads the permanent stats. Nimbleness
    /// restores Dexterity before `GAIN_STAT` (`RESTORE_STAT`), so a Ranger
    /// whose Dexterity is drained to 18/90 from 18/100 gains nothing, and
    /// `LOSE_RANDOM_STAT` lowers the maximum of another stat, so a stat drained
    /// now still costs its maximum.
    #[test]
    fn the_trade_is_priced_at_the_permanent_stats_not_the_drained_ones() {
        use super::{second_stat_potion_wanted_at as wanted, CLASS_RANGER};
        // Four other stats at 17 (a table step lost each); Dexterity 18/90
        // drained from 18/100.
        let at_cap = [17, 17, 17, 18 + 100, 17];
        let drained = [17, 17, 17, 18 + 90, 17];
        assert!(wanted(CLASS_RANGER, drained, 3), "the drained value alone looks like an even trade");
        assert!(!wanted(CLASS_RANGER, at_cap, 3), "the maximum is capped: no gain");
        // Dexterity 18/70 gains 9 tenths on average. Four other stats at
        // 18/50 lose 10 each; with Strength drained to 3 the current values
        // would lose only 7.5 and accept, but the permanent loss is the
        // maximum's.
        let maxima = [18 + 50, 18 + 50, 18 + 50, 18 + 70, 18 + 50];
        let current = [3, 18 + 50, 18 + 50, 18 + 70, 18 + 50];
        assert!(!wanted(CLASS_RANGER, maxima, 3), "9 gained against 10 lost");
        assert!(wanted(CLASS_RANGER, current, 3), "the drained Strength hides a table step of loss");
    }

    /// #1371: the one rule refuses a gain-one/lose-one potion whatever the
    /// shelf, and nothing else.
    #[test]
    fn a_refused_stat_trade_is_a_potion_the_class_will_not_drink() {
        use super::{stat_trade_refused_at as refused, CLASS_RANGER};
        use crate::headers::list_tvals_h::{TV_POTION, TV_SCROLL};
        let (iv, stats) = (kinds(), [18 + 40, 14, 18 + 20, 18 + 50, 18 + 30]);
        assert!(refused(&iv, CLASS_RANGER, stats, TV_POTION, 42), "Contemplation, by the class table");
        assert!(!refused(&iv, CLASS_RANGER, stats, TV_POTION, 43), "Nimbleness is drunk");
        assert!(!refused(&iv, CLASS_RANGER, stats, TV_POTION, 8), "Cure Light Wounds is not a trade");
        assert!(!refused(&iv, CLASS_RANGER, stats, TV_SCROLL, 42), "a scroll of the same sval is not a potion");
    }

    /// #1375: an Unlight class refuses a Wooden Torch and a Scroll of Light.
    #[test]
    fn an_unlight_class_refuses_the_torch_and_the_light_scroll() {
        use super::unlight_refused as refused;
        use crate::headers::list_tvals_h::{TV_LIGHT, TV_POTION, TV_SCROLL};
        let iv = kinds();
        let (necromancer, mage) = (crate::borg_light::CLASS_NECROMANCER, 1);
        assert!(refused(necromancer, &iv, TV_SCROLL, 17), "Light scroll");
        assert!(refused(necromancer, &iv, TV_LIGHT, 2), "Wooden Torch");
        assert!(!refused(necromancer, &iv, TV_LIGHT, 3), "any other light");
        assert!(!refused(necromancer, &iv, TV_SCROLL, 5), "any other scroll");
        assert!(!refused(necromancer, &iv, TV_POTION, 17), "a potion of the same sval");
        assert!(!refused(mage, &iv, TV_SCROLL, 17) && !refused(mage, &iv, TV_LIGHT, 2), "a class that sees by light");
    }

    /// #1374 (review): *Enlightenment* is worth a drink in town only while
    /// Intelligence or Wisdom can change for good.
    #[test]
    fn star_enlightenment_is_for_town_only_while_a_stat_can_gain_or_is_drained() {
        use super::star_enlightenment_gains_at as gains;
        let capped = [10, 18 + 100, 18 + 100, 10, 10];
        assert!(!gains(capped, capped), "both capped and undrained: only the mapping is left");
        assert!(gains([10, 17, 18 + 100, 10, 10], [10, 17, 18 + 100, 10, 10]), "Intelligence below 18/100");
        assert!(gains(capped, [10, 18 + 100, 18 + 90, 10, 10]), "Wisdom drained below a capped maximum");
        assert!(!gains([10, 18 + 100, 18 + 100, 3, 3], [3, 18 + 100, 18 + 100, 3, 3]),
            "other stats do not matter");
    }

    #[test]
    fn a_class_buys_only_the_second_stat_potions_it_drinks() {
        use super::drinks_second_stat_potion as d;
        let (warrior, mage, druid, priest, necromancer, rogue) = (0, 1, 2, 3, 4, 6);
        assert!(d(warrior, 0) && !d(warrior, 1) && !d(warrior, 2) && d(warrior, 3) && d(warrior, 4));
        assert!(!d(mage, 0) && d(mage, 1) && !d(mage, 2) && !d(mage, 3) && d(mage, 4));
        assert!(!d(druid, 0) && !d(druid, 1) && d(druid, 2) && !d(druid, 3));
        assert!(d(priest, 0) && !d(priest, 1) && d(priest, 2) && !d(priest, 3));
        assert!(!d(necromancer, 0) && d(necromancer, 1) && !d(necromancer, 2) && !d(necromancer, 3));
        assert!(d(rogue, 0) && d(rogue, 3), "the other classes drink strength and dexterity");
    }

    #[test]
    fn recall_reserve_and_purchase_respect_the_gold_floor() {
        use super::eligible_recall_reserve;
        assert_eq!(eligible_recall_reserve(399, 400, 1), 0);
        assert_eq!(eligible_recall_reserve(400, 400, 1), 1);
        assert_eq!(eligible_recall_reserve(800, 400, 0), 0);
        assert_eq!(eligible_recall_reserve(0, 0, 1), 1);
        use crate::borg_store_buy::recall_purchase_budget;
        assert_eq!(recall_purchase_budget(524, 400) / 125, 0);
        assert_eq!(recall_purchase_budget(525, 400) / 125, 1);
        assert_eq!(recall_purchase_budget(649, 400) / 125, 1);
        assert_eq!(recall_purchase_budget(650, 400) / 125, 2);
    }

    use super::scale_percent;

    #[test]
    fn the_recall_value_scales_by_its_percent() {
        assert_eq!(scale_percent(1000, 100), 1000, "100 = as computed");
        assert_eq!(scale_percent(1000, 500), 5000, "five times");
        assert_eq!(scale_percent(1000, 0), 0);
        assert_eq!(scale_percent(1000, -5), 0, "never negative");
    }
}

/// #978: the opt-in reserve starts when experience-draining monsters enter
/// the normal depth range (grape jelly, monster.txt:3306-3319). A repeatable
/// equipped activation or a learned low-failure restoration spell replaces it.
pub(crate) fn restore_life_reserve(st: &BorgState) -> i32 {
    use crate::borg_headers::borg_magic_h::{REMEMBRANCE, REVITALIZE};

    let copies = param(Param::town_restore_life_reserve);
    if copies == 0 || st.borg.tr(BI_MAXDEPTH) < param(Param::town_restore_life_min_depth) {
        return 0;
    }
    let repeatable = [st.act.cure_nonorlybig, st.act.restore_exp, st.act.restore_st_lev, st.act.restore_life]
        .into_iter().any(|activation| activation > 0 && st.borg.activation[activation as usize] != 0)
        || [REVITALIZE, REMEMBRANCE].into_iter()
            .any(|spell| crate::borg_magic::borg_spell_legal_fail(st, spell, param(Param::town_restore_life_spell_fail)));
    if repeatable { 0 } else { copies }
}

impl Model {
    /// borg-reborn-rs (#1241): marginal future use of a shop copy against Home stock.
    pub(crate) fn home_offer(&self, shop: u8, slot: u8) -> i64 {
        self.units(i64::from(shop), i64::from(slot), 1).into_iter()
            .filter(|(id, _)| !HEAL_KINDS.contains(id))
            .map(|(id, n)| {
                let row = &self.curves[id];
                self.value(id, row, row[4] + n) - self.value(id, row, row[4])
            }).sum()
    }
}

pub(crate) fn home_consumable_effect(effects: &[(u16, i32, i64)], st: &BorgState) -> bool {
    effect_supply_for_consumer(effects, i64::from(st.borg.tr(BI_MAXHP)), Journey::of(st).turns, false)
        .iter()
        .any(|&(id, _)| id != MANA || st.borg.tr(BI_MAXSP) > 0)
}

/// World ticks before at least one rod is ready. While all rods are cooling,
/// the engine subtracts the full stack size from its aggregate timeout.
pub(crate) fn rod_ready_after(quantity: i64, timeout: i64, cooldown: i64) -> i64 {
    let quantity = quantity.clamp(1, 255);
    let threshold = (quantity - 1).saturating_mul(cooldown.max(1));
    let remaining = timeout.saturating_sub(threshold).max(0) as u64;
    remaining.div_ceil(quantity as u64) as i64
}

#[test]
fn a_fully_cooling_stack_supplies_a_charge_only_after_its_first_ready_tick() {
    assert_eq!(rod_uses(1, 40, 40, 40), 0);
    assert_eq!(rod_uses(1, 40, 40, 41), 1);
    assert_eq!(rod_ready_after(2, 40, 40), 0);
    assert_eq!(rod_ready_after(2, 80, 40), 20);
    assert_eq!(rod_uses(2, 80, 40, 20), 0);
    assert_eq!(rod_uses(2, 80, 40, 21), 1);
}
