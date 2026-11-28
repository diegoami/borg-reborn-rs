//! borg-reborn-rs (#1435): the Home slot budget.
//!
//! With `town_home_slot_groups` the Home's 24 slots are split into a gear
//! group and a supply group (#1241's single pool and its soft one-third
//! supply goal are the 0 setting). Each group is ranked on its own, so that
//! Home covers as many capabilities and resistances as possible:
//!
//! * **supplies** by `town_home_rank_<capability>` x coverage x fill;
//! * **gear** by a greedy cover of the properties the worn kit and the pieces
//!   already kept do not give, plus a share of the piece's power.
//!
//! The ranking decides which stacks are *inside* the budget. A stack outside
//! it gets [`Role::Overflow`] when the kit is built (`borg_town_shadow::kit_of`):
//! the planner never stores or buys it, and clears it at Home. The loop
//! guards (one decision per visit, a margin for incumbents, a cooldown for
//! evicted kinds, a move budget) live in [`Memory`], which survives the visit.
use crate::borg_headers::borg_item_h::borg_item;
use crate::borg_headers::borg_trait_h::{BI_CLEVEL, BI_MAXDEPTH, BI_MAXSP, BI_RPOIS};
use crate::borg_town_dispose::{KitItem, Place};
use crate::borg_town_home::{is_book, Role};
use crate::headers::list_elements_h::*;
use crate::headers::list_object_flags_h::*;
use crate::headers::list_tvals_h::*;
use crate::params::{value as param, Param};
use crate::state::BorgState;
use crate::{engine, borg_store::BORG_HOME};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Group {
    Gear,
    Supply,
}

/// The group a Home role belongs to; `None` for the stock that has no
/// Home budget (Renewable, Common, Reject, Ordinary: cleared as today).
pub(crate) fn group_of(role: Role) -> Option<Group> {
    match role {
        Role::Gear | Role::LightGear | Role::SpareWeapon | Role::Artifact => Some(Group::Gear),
        Role::LatePotion | Role::Scarce | Role::Device => Some(Group::Supply),
        _ => None,
    }
}

pub(crate) fn enabled() -> bool {
    param(Param::town_home_slot_groups) != 0
}

// --- the budget's numbers ---------------------------------------------------

/// The settings the ranking reads.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Budget {
    pub(crate) gear: usize,
    pub(crate) supply: usize,
    pub(crate) borrow_free: bool,
    pub(crate) margin: i64,
    pub(crate) cooldown_visits: u32,
    pub(crate) max_moves: usize,
}

impl Budget {
    /// The group capacities at a max depth: with version 2's shift the supply
    /// group takes one slot from the gear group per `town_home_shift_depth_step`
    /// levels past `town_home_shift_from_depth`, up to `town_home_supply_slots_max`.
    pub(crate) fn of(max_depth: i32) -> Self {
        let (gear, supply) = (param(Param::town_home_gear_slots).max(0) as usize,
            param(Param::town_home_supply_slots).max(0) as usize);
        let step = param(Param::town_home_shift_depth_step);
        let shift = if step > 0 {
            let past = (max_depth - param(Param::town_home_shift_from_depth)).max(0);
            let cap = (param(Param::town_home_supply_slots_max).max(0) as usize).saturating_sub(supply);
            ((past / step) as usize).min(cap).min(gear)
        } else { 0 };
        Budget {
            gear: gear - shift,
            supply: supply + shift,
            borrow_free: param(Param::town_home_borrow_free) != 0,
            margin: i64::from(param(Param::town_home_replace_margin_percent)),
            cooldown_visits: param(Param::town_home_evict_cooldown_visits).max(0) as u32,
            max_moves: param(Param::town_home_max_moves).max(0) as usize,
        }
    }

    fn capacity(&self, group: Group) -> usize {
        match group { Group::Gear => self.gear, Group::Supply => self.supply }
    }
}

// --- supply capabilities ----------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Capability {
    Teleport,
    Heal,
    Speed,
    BigHeal,
    LevelEscape,
    RestoreLife,
    Mana,
    Book,
    TeleportOther,
    RestoreStats,
    Poison,
    Destruction,
    Banishment,
    Glyph,
}

impl Capability {
    pub(crate) const ALL: [Capability; 14] = [
        Capability::Teleport, Capability::Heal, Capability::Speed, Capability::BigHeal,
        Capability::LevelEscape, Capability::RestoreLife, Capability::Mana, Capability::Book,
        Capability::TeleportOther, Capability::RestoreStats, Capability::Poison,
        Capability::Destruction, Capability::Banishment, Capability::Glyph,
    ];

    /// `town_home_rank_<capability>`.
    pub(crate) fn weight(self) -> i64 {
        i64::from(param(match self {
            Capability::Teleport => Param::town_home_rank_teleport,
            Capability::Heal => Param::town_home_rank_heal,
            Capability::Speed => Param::town_home_rank_speed,
            Capability::BigHeal => Param::town_home_rank_big_heal,
            Capability::LevelEscape => Param::town_home_rank_level_escape,
            Capability::RestoreLife => Param::town_home_rank_restore_life,
            Capability::Mana => Param::town_home_rank_mana,
            Capability::Book => Param::town_home_rank_book,
            Capability::TeleportOther => Param::town_home_rank_teleport_other,
            Capability::RestoreStats => Param::town_home_rank_restore_stats,
            Capability::Poison => Param::town_home_rank_poison,
            Capability::Destruction => Param::town_home_rank_destruction,
            Capability::Banishment => Param::town_home_rank_banishment,
            Capability::Glyph => Param::town_home_rank_glyph,
        }))
    }

    /// The copies that fill a stack: the gate counts (`prepared_*`) where a
    /// gate names the capability, else `town_home_reserve_target`.
    pub(crate) fn target(self) -> i32 {
        let gate = |params: &[Param]| params.iter().map(|&p| param(p)).max().unwrap_or(0);
        let target = match self {
            Capability::Teleport => gate(&[Param::prepared_dl10_min_teleports, Param::prepared_dl26_min_teleports]),
            Capability::Heal => gate(&[Param::prepared_dl46_min_heal, Param::prepared_dl56_min_heal,
                Param::prepared_dl100_min_healing_potions]),
            Capability::Speed => gate(&[Param::prepared_dl100_min_speed_potions]),
            Capability::BigHeal => gate(&[Param::prepared_dl46_min_star_heal, Param::prepared_dl56_min_star_heal,
                Param::prepared_dl100_min_star_heal_or_life]),
            Capability::Mana => gate(&[Param::prepared_dl100_min_restore_mana]),
            _ => 0,
        };
        if target > 0 { target } else { param(Param::town_home_reserve_target).max(1) }
    }
}

/// The capability a kind of stock serves, `None` for stock outside the
/// supply group's list. A stat potion that restores counts as stat
/// restoration, as `borg_town_home::late_potion` classes it.
pub(crate) fn capability_of(st: &BorgState, item: &borg_item) -> Option<Capability> {
    if !item.aware { return None; }
    let v = &st.item_val;
    let sval = i32::from(item.sval);
    let tval = u32::from(item.tval);
    Some(match tval {
        TV_SCROLL if sval == v.sv_scroll_teleport => Capability::Teleport,
        TV_SCROLL if sval == v.sv_scroll_teleport_level => Capability::LevelEscape,
        TV_SCROLL if sval == v.sv_scroll_mass_banishment || sval == v.sv_scroll_banishment => Capability::Banishment,
        TV_SCROLL if sval == v.sv_scroll_rune_of_protection => Capability::Glyph,
        TV_STAFF if sval == v.sv_staff_teleportation => Capability::Teleport,
        TV_STAFF if sval == v.sv_staff_healing => Capability::Heal,
        TV_STAFF if sval == v.sv_staff_speed => Capability::Speed,
        TV_STAFF if sval == v.sv_staff_the_magi => Capability::Mana,
        TV_STAFF if sval == v.sv_staff_destruction => Capability::Destruction,
        TV_POTION if sval == v.sv_potion_healing => Capability::Heal,
        TV_POTION if sval == v.sv_potion_star_healing || sval == v.sv_potion_life => Capability::BigHeal,
        TV_POTION if sval == v.sv_potion_speed => Capability::Speed,
        TV_POTION if sval == v.sv_potion_restore_life => Capability::RestoreLife,
        TV_POTION if sval == v.sv_potion_restore_mana => Capability::Mana,
        TV_POTION if sval == v.sv_potion_resist_pois => Capability::Poison,
        TV_WAND if sval == v.sv_wand_teleport_away => Capability::TeleportOther,
        TV_ROD if sval == v.sv_rod_teleport_other => Capability::TeleportOther,
        TV_MUSHROOM if sval == v.sv_mush_restoring || sval == v.sv_mush_purging => Capability::RestoreStats,
        _ if is_book(tval) => Capability::Book,
        TV_POTION => {
            let effects = engine::kind_effect_values(item.kind as usize, st.borg.tr(BI_CLEVEL));
            if crate::borg_town_home::stat_restoring(&effects) { Capability::RestoreStats } else { return None }
        }
        _ => return None,
    })
}

/// The copies a stack counts for: the charges of a staff or wand, else the
/// stack size.
pub(crate) fn copies_of(item: &borg_item) -> i32 {
    if matches!(u32::from(item.tval), TV_STAFF | TV_WAND) {
        i32::from(item.pval).max(0)
    } else { i32::from(item.iqty) }
}

/// A capability the character already renews without stock: a spell legal at
/// `town_home_spell_fail` or better, a permanent resistance, a non-caster for
/// the caster-only capabilities.
fn renewable(st: &BorgState, capability: Capability) -> bool {
    use crate::borg_headers::borg_magic_h::*;
    let fail = param(Param::town_home_spell_fail);
    let spell = |spells: &[borg_spells]| spells.iter().any(|&s| crate::borg_magic::borg_spell_legal_fail(st, s, fail));
    let caster = st.borg.tr(BI_MAXSP) > 0;
    match capability {
        Capability::Teleport => spell(&[TELEPORT_SELF, PORTAL, SHADOW_SHIFT, DIMENSION_DOOR]),
        Capability::LevelEscape => spell(&[TELEPORT_LEVEL]),
        Capability::Heal => spell(&[HEALING]),
        Capability::Speed => spell(&[HASTE_SELF]),
        Capability::Glyph => spell(&[GLYPH_OF_WARDING]),
        Capability::Poison => st.borg.tr(BI_RPOIS) > 0,
        Capability::Mana | Capability::Book => !caster,
        _ => false,
    }
}

/// One stack's supply score. `rank` is its position among the stacks of its
/// capability (0 = the largest): the first has full coverage, the second
/// `town_home_repeat_percent`, a third none.
pub(crate) fn supply_score(capability: Capability, renews: bool, rank: usize, copies: i32) -> i64 {
    let coverage = match rank {
        _ if renews => 0,
        0 => 100,
        1 => i64::from(param(Param::town_home_repeat_percent)),
        _ => 0,
    };
    let target = i64::from(capability.target());
    let fill = (i64::from(copies.max(0)).min(target) * 100 / target)
        .max(i64::from(param(Param::town_home_fill_min_percent)));
    capability.weight() * coverage * fill / 100
}

// --- gear scores --------------------------------------------------------------

fn resists(item: &borg_item, element: u32, level: i16) -> bool {
    item.el_info[element as usize].res_level >= level
}

fn has(item: &borg_item, flag: u32) -> bool {
    engine::flag_has(&item.flags, flag as i32)
}

type Gives = fn(&borg_item) -> bool;

/// The properties of `borg_home_power.rs`, with the weights it already names
/// for a first and a second piece. Light, regeneration, slow digestion and
/// feather fall are there too; they weigh little.
const PROPERTIES: &[(Gives, Param, Param)] = &[
    (|i| resists(i, ELEM_ACID, 1), Param::home_resist_acid_one_weight, Param::home_resist_acid_two_weight),
    (|i| resists(i, ELEM_ELEC, 1), Param::home_resist_elec_one_weight, Param::home_resist_elec_two_weight),
    (|i| resists(i, ELEM_FIRE, 1), Param::home_resist_fire_one_weight, Param::home_resist_fire_two_weight),
    (|i| resists(i, ELEM_COLD, 1), Param::home_resist_cold_one_weight, Param::home_resist_cold_two_weight),
    (|i| resists(i, ELEM_ACID, 3), Param::home_immune_acid_one_weight, Param::home_immune_acid_two_weight),
    (|i| resists(i, ELEM_ELEC, 3), Param::home_immune_elec_one_weight, Param::home_immune_elec_two_weight),
    (|i| resists(i, ELEM_FIRE, 3), Param::home_immune_fire_one_weight, Param::home_immune_fire_two_weight),
    (|i| resists(i, ELEM_COLD, 3), Param::home_immune_cold_one_weight, Param::home_immune_cold_two_weight),
    (|i| resists(i, ELEM_POIS, 1), Param::home_resist_pois_one_weight, Param::home_resist_pois_two_weight),
    (|i| resists(i, ELEM_LIGHT, 1), Param::home_resist_light_one_weight, Param::home_resist_light_two_weight),
    (|i| resists(i, ELEM_DARK, 1), Param::home_resist_dark_one_weight, Param::home_resist_dark_two_weight),
    (|i| resists(i, ELEM_SOUND, 1), Param::home_resist_sound_one_weight, Param::home_resist_sound_two_weight),
    (|i| resists(i, ELEM_SHARD, 1), Param::home_resist_shard_one_weight, Param::home_resist_shard_two_weight),
    (|i| resists(i, ELEM_NEXUS, 1), Param::home_resist_nexus_one_weight, Param::home_resist_nexus_two_weight),
    (|i| resists(i, ELEM_NETHER, 1), Param::home_resist_neth_one_weight, Param::home_resist_neth_two_weight),
    (|i| resists(i, ELEM_CHAOS, 1), Param::home_resist_chaos_one_weight, Param::home_resist_chaos_two_weight),
    (|i| resists(i, ELEM_DISEN, 1), Param::home_resist_disen_one_weight, Param::home_resist_disen_two_weight),
    (|i| has(i, OF_PROT_BLIND), Param::home_resist_blind_one_weight, Param::home_resist_blind_two_weight),
    (|i| has(i, OF_PROT_CONF), Param::home_resist_conf_one_weight, Param::home_resist_conf_two_weight),
    (|i| has(i, OF_FREE_ACT), Param::home_free_act_one_weight, Param::home_free_act_two_weight),
    (|i| has(i, OF_SEE_INVIS), Param::home_see_inv_one_weight, Param::home_see_inv_two_weight),
    (|i| has(i, OF_HOLD_LIFE), Param::home_hold_life_one_weight, Param::home_hold_life_two_weight),
    (|i| has(i, OF_TELEPATHY), Param::home_telepathy_one_weight, Param::home_telepathy_two_weight),
    (|i| has(i, OF_FEATHER), Param::home_ffall_one_weight, Param::home_ffall_two_weight),
    (|i| has(i, OF_REGEN), Param::home_regenerate_one_weight, Param::home_regenerate_two_weight),
    (|i| has(i, OF_SLOW_DIGEST), Param::home_slow_digest_one_weight, Param::home_slow_digest_two_weight),
    (|i| has(i, OF_SUST_STR), Param::home_sustain_str_one_weight, Param::home_sustain_str_two_weight),
    (|i| has(i, OF_SUST_INT), Param::home_sustain_int_one_weight, Param::home_sustain_int_two_weight),
    (|i| has(i, OF_SUST_WIS), Param::home_sustain_wis_one_weight, Param::home_sustain_wis_two_weight),
    (|i| has(i, OF_SUST_DEX), Param::home_sustain_dex_one_weight, Param::home_sustain_dex_two_weight),
    (|i| has(i, OF_SUST_CON), Param::home_sustain_con_one_weight, Param::home_sustain_con_two_weight),
];

/// How many of the pieces kept so far give each property.
#[derive(Clone, Debug)]
pub(crate) struct Cover([u8; PROPERTIES.len()]);

impl Cover {
    pub(crate) fn of<'a>(items: impl IntoIterator<Item = &'a borg_item>) -> Self {
        let mut cover = Cover([0; PROPERTIES.len()]);
        for item in items { cover.add(item); }
        cover
    }

    fn add(&mut self, item: &borg_item) {
        for (count, (gives, _, _)) in self.0.iter_mut().zip(PROPERTIES) {
            if gives(item) { *count = count.saturating_add(1); }
        }
    }

    /// What `item` adds: the first piece with a property its `one` weight,
    /// the second `two` less `one`, times `town_home_gear_cover_percent`.
    fn gain(&self, item: &borg_item) -> i64 {
        let sum: i64 = self.0.iter().zip(PROPERTIES).map(|(&count, (gives, one, two))| {
            if !gives(item) { return 0; }
            match count {
                0 => i64::from(param(*one)),
                1 => i64::from(param(*two)) - i64::from(param(*one)),
                _ => 0,
            }.max(0)
        }).sum();
        sum * i64::from(param(Param::town_home_gear_cover_percent)) / 100
    }
}

// --- the ranking --------------------------------------------------------------

/// What identifies a stack across visits: its kind and variant (for gear the
/// variant, as `borg_town_home::same_variant` compares it).
pub(crate) type Key = [i64; 7];

pub(crate) fn key_of(item: &borg_item) -> Key {
    [i64::from(item.kind), i64::from(item.art_idx), i64::from(item.ego_idx), i64::from(item.to_h),
        i64::from(item.to_d), i64::from(item.to_a), i64::from(item.pval)]
}

/// One stack (or one kind, for carried supplies with no Home stack) the
/// ranking places. `members` are its kit indices.
#[derive(Clone)]
pub(crate) struct Entry {
    pub(crate) members: Vec<usize>,
    pub(crate) group: Group,
    pub(crate) key: Key,
    /// A stack at Home: an incumbent.
    pub(crate) incumbent: bool,
    /// Never evicted: a staged loadout, this visit's purchase, an unknown item.
    pub(crate) pinned: bool,
    pub(crate) weight: i32,
    pub(crate) slot: u8,
    /// A supply's score (final); for gear, its power gain, which the greedy
    /// cover adds to the marginal property score.
    pub(crate) score: i64,
    /// Gear only: what its properties are, for the cover.
    pub(crate) item: Option<borg_item>,
}

/// The loop guards' memory, across visits (#1435).
#[derive(Clone, Debug, Default)]
pub(crate) struct Memory {
    /// The town visit's number.
    pub(crate) visit: u32,
    /// Evicted kinds: (key, the last visit they stay out, the score they had).
    cooldown: Vec<(Key, u32, i64)>,
    /// The visit's decisions: (key, group, inside the budget).
    decided: Vec<(Key, bool)>,
    /// Evictions this visit.
    evicted: Vec<Key>,
}

impl Memory {
    /// A new visit: the decisions and the eviction count start again; the
    /// cooldowns stay, counted in visits.
    pub(crate) fn new_visit(mut self) -> Self {
        self.visit += 1;
        self.decided.clear();
        self.evicted.clear();
        self.cooldown.retain(|&(_, until, _)| until >= self.visit);
        self
    }

    fn cooling(&self, key: &Key) -> Option<i64> {
        self.cooldown.iter().find(|(k, until, _)| k == key && *until >= self.visit).map(|&(_, _, score)| score)
    }

    fn decision(&self, key: &Key) -> Option<bool> {
        self.decided.iter().find(|(k, _)| k == key).map(|&(_, inside)| inside)
    }

    /// Whether a stack was put outside the budget this visit.
    pub(crate) fn outside(&self, key: &Key) -> bool {
        self.decision(key) == Some(false)
    }

    pub(crate) fn evictions(&self) -> usize {
        self.evicted.len()
    }
}

/// The result of placing every entry.
#[derive(Clone, Debug)]
pub(crate) struct Placement {
    pub(crate) inside: Vec<bool>,
    /// What each entry scored when it was placed (or left out): the value a
    /// cooldown remembers.
    pub(crate) value: Vec<i64>,
}

/// Rank one group and keep the best `capacity`: pinned entries first, then
/// by score (an incumbent's raised by the margin), the lighter first, the
/// earlier slot first. A non-incumbent in its cooldown stays out unless its
/// score has risen past the evicted one's by the margin. Decisions made
/// earlier in the visit stand; only new entries are placed, in the room left.
fn place_group(entries: &[Entry], group: Group, capacity: usize, margin: i64, memory: &Memory,
    start: &Cover, inside: &mut [bool], values: &mut [i64]) {
    let mut ids: Vec<usize> = (0..entries.len()).filter(|&i| entries[i].group == group).collect();
    let mut cover = start.clone();
    // Entries the visit already decided keep their decision, and so do those a
    // first pass placed (the borrowing pass runs over its result).
    for &i in &ids {
        let e = &entries[i];
        if e.pinned || memory.decision(&e.key) == Some(true) { inside[i] = true; }
    }
    let mut used = 0usize;
    for &i in &ids {
        if inside[i] {
            used += 1;
            if let Some(item) = &entries[i].item { cover.add(item); }
        }
    }
    ids.retain(|&i| !inside[i] && memory.decision(&entries[i].key).is_none());
    let value = |i: usize, cover: &Cover| -> i64 {
        let e = &entries[i];
        let base = match (&e.item, group) {
            (Some(item), Group::Gear) => cover.gain(item)
                + e.score * i64::from(param(Param::town_home_gear_power_percent)) / 100,
            _ => e.score,
        };
        if e.incumbent { base * (100 + margin) / 100 } else { base }
    };
    // Greedy: the best marginal score first, recomputed as pieces are kept.
    while used < capacity && !ids.is_empty() {
        let blocked = |i: usize, cover: &Cover| {
            let e = &entries[i];
            !e.incumbent && memory.cooling(&e.key).is_some_and(|old| value(i, cover) * 100 <= old * (100 + margin))
        };
        let best = ids.iter().copied().filter(|&i| !blocked(i, &cover) && (group == Group::Gear || entries[i].score > 0))
            .max_by_key(|&i| (value(i, &cover), std::cmp::Reverse(entries[i].weight),
                entries[i].incumbent, std::cmp::Reverse(entries[i].slot), std::cmp::Reverse(i)));
        let Some(i) = best else { break; };
        inside[i] = true;
        values[i] = value(i, &cover);
        used += 1;
        if let Some(item) = &entries[i].item { cover.add(item); }
        ids.retain(|&j| j != i);
    }
    for i in ids { values[i] = value(i, &cover); }
}

/// Place all entries. `occupied` counts, per group, the stacks outside the
/// entries (none now); with `borrow_free` a group may also use the other
/// group's empty slots.
pub(crate) fn place(entries: &[Entry], budget: &Budget, memory: &Memory, worn: &Cover, exhausted: bool) -> Placement {
    let mut inside = vec![false; entries.len()];
    let mut values: Vec<i64> = entries.iter().map(|e| e.score).collect();
    let used = |group: Group, inside: &[bool]| (0..entries.len()).filter(|&i| entries[i].group == group && inside[i]).count();
    for group in [Group::Gear, Group::Supply] {
        // With the move budget spent, only the stacks already at Home stay.
        let mut shown: Vec<Entry> = entries.to_vec();
        if exhausted {
            for e in &mut shown { if !e.incumbent && !e.pinned { e.score = i64::MIN / 4; } }
        }
        let start = if group == Group::Gear { worn.clone() } else { Cover::of([]) };
        place_group(&shown, group, budget.capacity(group), budget.margin, memory, &start, &mut inside, &mut values);
        if exhausted {
            for (i, e) in entries.iter().enumerate() { if e.group == group && !e.incumbent && !e.pinned { inside[i] = false; } }
        }
    }
    if budget.borrow_free {
        // A group may use the other's empty slots; the stacks left out
        // above are placed again in the room the other group leaves.
        for group in [Group::Gear, Group::Supply] {
            let other = if group == Group::Gear { Group::Supply } else { Group::Gear };
            let free = budget.capacity(other).saturating_sub(used(other, &inside));
            if free == 0 || exhausted { continue; }
            let extra = budget.capacity(group) + free;
            let start = if group == Group::Gear { worn.clone() } else { Cover::of([]) };
            place_group(entries, group, extra, budget.margin, memory, &start, &mut inside, &mut values);
        }
    }
    Placement { inside, value: values }
}

// --- the adapter: the kit and the state ----------------------------------------

/// Whether the visit's move budget is spent: Home stores, withdrawals and
/// evictions so far.
fn exhausted(st: &BorgState, budget: &Budget, memory: &Memory) -> bool {
    budget.max_moves > 0 && memory.evictions() + crate::borg_town_live::home_moves_count(st) >= budget.max_moves
}

/// Put every stack outside the Home budget at [`Role::Overflow`]. Called as
/// the kit is built (`borg_town_shadow::kit_of`), after the roles are set.
/// With `live`, the visit's decisions and evictions are written down; a read
/// only call (a replay's kit) leaves the memory alone.
pub(crate) fn apply(st: &mut BorgState, kit: &mut [KitItem], live: bool) {
    if !enabled() || !crate::borg_town_home::enabled(st) { return; }
    let budget = Budget::of(st.borg.tr(BI_MAXDEPTH));
    let mut memory = std::mem::take(&mut st.think.town_live.home_budget);
    let home: Vec<borg_item> = st.store.shops[BORG_HOME].ware.clone();
    let item_of = |st: &BorgState, k: &KitItem| -> borg_item {
        match k.place { Place::Home => home[usize::from(k.slot)], Place::Pack => st.items.items[usize::from(k.slot)] }
    };
    let mut entries: Vec<Entry> = Vec::new();
    let mut no_capability: Vec<usize> = Vec::new();

    // Supplies: one entry per Home stack, and per kind carried with no Home stack.
    let mut supply: Vec<(usize, Capability)> = Vec::new();
    for (i, k) in kit.iter().enumerate() {
        if group_of(k.home_role) != Some(Group::Supply) || k.unknown_flavour { continue; }
        match capability_of(st, &item_of(st, k)) {
            Some(capability) => supply.push((i, capability)),
            None => no_capability.push(i),
        }
    }
    let mut ranked: Vec<(usize, Capability, i32)> = Vec::new();
    for &(i, capability) in &supply {
        let k = &kit[i];
        if k.place == Place::Pack && kit.iter().any(|h| h.place == Place::Home && h.kind == k.kind && group_of(h.home_role) == Some(Group::Supply)) {
            continue;
        }
        let members: Vec<usize> = if k.place == Place::Pack {
            supply.iter().filter(|&&(j, _)| kit[j].place == Place::Pack && kit[j].kind == k.kind).map(|&(j, _)| j).collect()
        } else { vec![i] };
        if k.place == Place::Pack && members[0] != i { continue; }
        let copies: i32 = members.iter().map(|&j| copies_of(&item_of(st, &kit[j]))).sum();
        let item = item_of(st, k);
        entries.push(Entry {
            members, group: Group::Supply, key: key_of(&item), incumbent: k.place == Place::Home,
            pinned: k.just_bought || k.keep_last_book || k.unknown
                || k.place == Place::Home && crate::borg_town_live::home_moved(st, &item, true),
            weight: k.weight, slot: k.slot, score: 0, item: None,
        });
        ranked.push((entries.len() - 1, capability, copies));
    }
    // Merge the carried stacks of a kind that Home holds into its entry.
    for &(i, _) in &supply {
        let k = &kit[i];
        if k.place != Place::Pack { continue; }
        if let Some(entry) = entries.iter_mut().find(|e| e.incumbent && kit[e.members[0]].kind == k.kind) {
            entry.members.push(i);
        }
    }
    for capability in Capability::ALL {
        let mut of: Vec<usize> = (0..ranked.len()).filter(|&r| ranked[r].1 == capability).collect();
        of.sort_by_key(|&r| (std::cmp::Reverse(ranked[r].2), !entries[ranked[r].0].incumbent, entries[ranked[r].0].slot));
        let renews = renewable(st, capability);
        for (rank, &r) in of.iter().enumerate() {
            entries[ranked[r].0].score = supply_score(capability, renews, rank, ranked[r].2);
        }
    }

    // Gear: every stack with a gear role.
    let wield = crate::borg_item::inven_wield() as usize;
    let worn = Cover::of(st.items.items[wield..crate::borg_item::quiver_start() as usize].iter()
        .filter(|item| item.iqty > 0 && !item.cursed));
    for (i, k) in kit.iter().enumerate() {
        if group_of(k.home_role) != Some(Group::Gear) { continue; }
        let item = item_of(st, k);
        entries.push(Entry {
            members: vec![i], group: Group::Gear, key: key_of(&item), incumbent: k.place == Place::Home,
            pinned: k.just_bought || k.unknown || k.place == Place::Home
                && (crate::borg_town_live::retained_home_gear(st, usize::from(k.slot))
                    || crate::borg_town_live::home_moved(st, &item, true)),
            weight: k.weight, slot: k.slot, score: k.power.max(0), item: Some(item),
        });
    }

    let spent = exhausted(st, &budget, &memory);
    let placed = place(&entries, &budget, &memory, &worn, spent);
    for (e, (&inside, &value)) in entries.iter().zip(placed.inside.iter().zip(&placed.value)) {
        if live && memory.decision(&e.key).is_none() {
            memory.decided.push((e.key, inside));
            if !inside && e.incumbent && !e.pinned {
                memory.evicted.push(e.key);
                if budget.cooldown_visits > 0 {
                    let until = memory.visit + budget.cooldown_visits;
                    memory.cooldown.retain(|(k, _, _)| *k != e.key);
                    memory.cooldown.push((e.key, until, value));
                }
            }
        }
        let outside = !inside && !e.pinned && memory.decision(&e.key) != Some(true);
        if outside {
            for &m in &e.members { kit[m].home_role = Role::Overflow; }
        }
    }
    for i in no_capability { kit[i].home_role = Role::Overflow; }
    st.think.town_live.home_budget = memory;
}

/// Whether the budget has put this stack outside the Home slots this visit.
pub(crate) fn outside(st: &BorgState, item: &borg_item) -> bool {
    enabled() && st.think.town_live.home_budget.outside(&key_of(item))
}

/// Whether a Home purchase of `item` (`copies` of it) would fit the supply
/// budget: its capability's score reaches a free slot, or beats the weakest
/// stack of the group by the margin.
pub(crate) fn supply_fits(st: &BorgState, item: &borg_item, copies: i32) -> bool {
    if !enabled() { return true; }
    let Some(capability) = capability_of(st, item) else { return false; };
    let budget = Budget::of(st.borg.tr(BI_MAXDEPTH));
    let memory = &st.think.town_live.home_budget;
    if memory.cooling(&key_of(item)).is_some() { return false; }
    // The Home's own stacks of the group, scored as they stand.
    let home = &st.store.shops[BORG_HOME].ware;
    let mut stacks = 0usize;
    let mut same = 0usize;
    let mut weakest = i64::MAX;
    for ware in home.iter().filter(|w| w.iqty > 0) {
        let Some(c) = capability_of(st, ware) else { continue; };
        if !matches!(u32::from(ware.tval), TV_POTION | TV_SCROLL | TV_MUSHROOM | TV_STAFF | TV_WAND | TV_ROD) && !is_book(u32::from(ware.tval)) { continue; }
        stacks += 1;
        same += usize::from(c == capability);
        weakest = weakest.min(supply_score(c, renewable(st, c), 0, copies_of(ware)));
    }
    let score = supply_score(capability, renewable(st, capability), same, copies);
    if score <= 0 { return false; }
    stacks < budget.supply || score * 100 > weakest.saturating_mul(100 + budget.margin)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn supply(key: i64, slot: u8, score: i64, incumbent: bool) -> Entry {
        Entry { members: vec![slot as usize], group: Group::Supply, key: [key, 0, 0, 0, 0, 0, 0], incumbent,
            pinned: false, weight: 1, slot, score, item: None }
    }

    fn budget(gear: usize, supply: usize) -> Budget {
        Budget { gear, supply, borrow_free: false, margin: 25, cooldown_visits: 3, max_moves: 8 }
    }

    #[test]
    fn roles_belong_to_one_group() {
        for role in [Role::Gear, Role::LightGear, Role::SpareWeapon, Role::Artifact] {
            assert_eq!(group_of(role), Some(Group::Gear));
        }
        for role in [Role::LatePotion, Role::Scarce, Role::Device] {
            assert_eq!(group_of(role), Some(Group::Supply));
        }
        for role in [Role::Renewable, Role::Common, Role::Reject, Role::Ordinary, Role::Overflow] {
            assert_eq!(group_of(role), None);
        }
    }

    #[test]
    fn the_shipped_budget_fits_the_home() {
        let b = Budget::of(0);
        assert_eq!((b.gear, b.supply), (16, 8));
        assert!(b.gear + b.supply <= 24);
    }

    /// The group keeps its best stacks; a single copy never beats a full stack.
    #[test]
    fn the_best_stacks_fill_the_group() {
        let entries = [supply(1, 0, 100, false), supply(2, 1, 90, false), supply(3, 2, 10, false)];
        let placed = place(&entries, &budget(16, 2), &Memory::default(), &Cover::of([]), false);
        assert_eq!(placed.inside, [true, true, false]);
        let placed = place(&entries, &budget(16, 8), &Memory::default(), &Cover::of([]), false);
        assert_eq!(placed.inside, [true, true, true]);
    }

    /// Guard 2: a challenger needs a higher score by the margin.
    #[test]
    fn a_near_equal_challenger_does_not_take_the_slot() {
        let incumbent = supply(1, 0, 100, true);
        let challenger = supply(2, 1, 120, false);
        let placed = place(&[incumbent.clone(), challenger], &budget(16, 1), &Memory::default(), &Cover::of([]), false);
        assert_eq!(placed.inside, [true, false], "120 does not beat 100 + 25 percent");
        let challenger = supply(2, 1, 130, false);
        let placed = place(&[incumbent, challenger], &budget(16, 1), &Memory::default(), &Cover::of([]), false);
        assert_eq!(placed.inside, [false, true], "130 does");
    }

    /// Guard 3: an evicted kind stays out for the cooldown unless its score
    /// rises past the margin; the cooldown ends with the visits.
    #[test]
    fn an_evicted_kind_waits_out_its_cooldown() {
        let mut memory = Memory::default();
        memory.cooldown.push(([2, 0, 0, 0, 0, 0, 0], 3, 50));
        let entries = [supply(2, 0, 50, false)];
        assert_eq!(place(&entries, &budget(16, 8), &memory, &Cover::of([]), false).inside, [false], "cooling");
        let rise = [supply(2, 0, 70, false)];
        assert_eq!(place(&rise, &budget(16, 8), &memory, &Cover::of([]), false).inside, [true], "70 beats 50 + 25 percent");
        let mut memory = memory;
        memory.visit = 4;
        assert_eq!(place(&entries, &budget(16, 8), &memory, &Cover::of([]), false).inside, [true], "the cooldown is over");
    }

    /// Guard 1: the visit's decisions stand while the state moves under them.
    #[test]
    fn a_decision_stands_for_the_visit() {
        let mut memory = Memory::default();
        memory.decided.push(([1, 0, 0, 0, 0, 0, 0], true));
        memory.decided.push(([2, 0, 0, 0, 0, 0, 0], false));
        // The stack decided in is kept though a much better one comes in; the
        // one decided out is not placed again.
        let entries = [supply(1, 0, 1, true), supply(2, 1, 100, false), supply(3, 2, 100, false)];
        let placed = place(&entries, &budget(16, 2), &memory, &Cover::of([]), false);
        assert_eq!(placed.inside, [true, false, true]);
    }

    /// Guard 5: with the move budget spent, only the stacks at Home stay.
    #[test]
    fn a_spent_move_budget_admits_nothing_new() {
        let entries = [supply(1, 0, 10, true), supply(2, 1, 100, false)];
        let placed = place(&entries, &budget(16, 8), &Memory::default(), &Cover::of([]), true);
        assert_eq!(placed.inside, [true, false]);
    }

    /// The fixed point: the same state ranks the same way twice, and the
    /// result already placed changes nothing.
    #[test]
    fn the_same_state_places_the_same_way() {
        let entries = [supply(1, 0, 90, true), supply(2, 1, 95, true), supply(3, 2, 80, false), supply(4, 3, 99, false)];
        let first = place(&entries, &budget(16, 2), &Memory::default(), &Cover::of([]), false);
        let again = place(&entries, &budget(16, 2), &Memory::default(), &Cover::of([]), false);
        assert_eq!(first.inside, again.inside);
        // Next visit the incumbents are the ones placed inside.
        let next: Vec<_> = entries.iter().zip(&first.inside).map(|(e, &inside)| Entry { incumbent: inside, ..e.clone() }).collect();
        let settled = place(&next, &budget(16, 2), &Memory::default(), &Cover::of([]), false);
        assert_eq!(settled.inside, first.inside, "the second visit moves nothing");
    }

    /// The incumbent margin settles two near-equal candidates: alternating
    /// the order they arrive in over five visits changes nothing after the first.
    #[test]
    fn two_near_equal_candidates_settle_after_the_first_visit() {
        let mut kept: Option<usize> = None;
        for visit in 0..5 {
            let (a, b) = if visit % 2 == 0 { (100, 104) } else { (104, 100) };
            let mut entries = [supply(1, 0, a, false), supply(2, 1, b, false)];
            if let Some(winner) = kept { entries[winner].incumbent = true; }
            let placed = place(&entries, &budget(16, 1), &Memory::default(), &Cover::of([]), false);
            let winner = placed.inside.iter().position(|&x| x).unwrap();
            if let Some(first) = kept { assert_eq!(winner, first, "visit {visit} swapped"); }
            kept = Some(winner);
        }
    }

    #[test]
    fn coverage_scores_by_position_and_renewal() {
        let teleport = Capability::Teleport;
        let full = supply_score(teleport, false, 0, 100);
        assert_eq!(full, 100 * 100);
        assert_eq!(supply_score(teleport, false, 1, 100), full / 4, "a second stack at town_home_repeat_percent");
        assert_eq!(supply_score(teleport, false, 2, 100), 0, "a third stack");
        assert_eq!(supply_score(teleport, true, 0, 100), 0, "a spell renews it");
        assert_eq!(supply_score(teleport, false, 0, 0), 1000, "the fill floor is 10 percent of a full stack");
        assert!(supply_score(teleport, false, 0, 1) < supply_score(Capability::Glyph, false, 0, 100) * 4,
            "a single copy never beats a full stack of a capability a third of its weight");
    }

    #[test]
    fn a_resistance_outranks_a_plain_piece_at_equal_power() {
        let mut rpois = crate::borg_item::blank_item();
        rpois.el_info[ELEM_POIS as usize].res_level = 1;
        let plain = crate::borg_item::blank_item();
        let gear = |key: i64, item: borg_item, slot: u8| Entry { members: vec![slot as usize], group: Group::Gear,
            key: [key, 0, 0, 0, 0, 0, 0], incumbent: false, pinned: false, weight: 20, slot, score: 0, item: Some(item) };
        let entries = [gear(1, plain, 0), gear(2, rpois, 1)];
        let placed = place(&entries, &budget(1, 8), &Memory::default(), &Cover::of([]), false);
        assert_eq!(placed.inside, [false, true], "rPois first");
        // A second piece with the property still adds (two less one), the third nothing:
        // with rPois on two worn pieces the plain piece, the earlier slot, wins the tie.
        let placed = place(&entries, &budget(1, 8), &Memory::default(), &Cover::of([&rpois]), false);
        assert_eq!(placed.inside, [false, true], "a second source of rPois still adds");
        let placed = place(&entries, &budget(1, 8), &Memory::default(), &Cover::of([&rpois, &rpois]), false);
        assert_eq!(placed.inside, [true, false], "two worn pieces already give rPois");
    }

    /// `town_home_borrow_free`: a group places stacks in the other's empty slots.
    #[test]
    fn a_group_borrows_the_other_groups_empty_slots() {
        let entries = [supply(1, 0, 90, false), supply(2, 1, 80, false), supply(3, 2, 70, false)];
        let strict = budget(16, 2);
        assert_eq!(place(&entries, &strict, &Memory::default(), &Cover::of([]), false).inside, [true, true, false]);
        let borrowing = Budget { borrow_free: true, ..strict };
        let placed = place(&entries, &borrowing, &Memory::default(), &Cover::of([]), false);
        assert_eq!(placed.inside, [true, true, true], "the empty gear slots take the third stack");
        let full: Vec<_> = (0..16).map(|slot| Entry { group: Group::Gear, item: Some(crate::borg_item::blank_item()),
            ..supply(100 + i64::from(slot), slot, 1, true) }).collect();
        let all: Vec<_> = full.iter().cloned().chain(entries.iter().cloned()).collect();
        let placed = place(&all, &borrowing, &Memory::default(), &Cover::of([]), false);
        assert_eq!(placed.inside.iter().filter(|&&x| x).count(), 16 + 2, "no free gear slot to borrow");
    }

    #[test]
    fn ties_break_by_weight_then_slot() {
        let gear = |key: i64, weight: i32, slot: u8| Entry { members: vec![slot as usize], group: Group::Gear,
            key: [key, 0, 0, 0, 0, 0, 0], incumbent: false, pinned: false, weight, slot, score: 10,
            item: Some(crate::borg_item::blank_item()) };
        let entries = [gear(1, 30, 0), gear(2, 20, 1), gear(3, 20, 2)];
        let placed = place(&entries, &budget(1, 8), &Memory::default(), &Cover::of([]), false);
        assert_eq!(placed.inside, [false, true, false], "the lighter, then the earlier slot");
    }
}
