//! Port of `borg-prepared.c`: check how deep the borg is prepared to go.
//!
//! Copyright (c) 1997 Ben Harrison, James E. Wilson, Robert A. Koeneke
//! Copyright (c) 2007-9 Andi Sidwell, Chris Carr, Ed Graham, Erik Osheim
//!
//! This work is free software; you can redistribute it and/or modify it
//! under the terms of either:
//!
//! a) the GNU General Public License as published by the Free Software
//!    Foundation, version 2, or
//!
//! b) the "Angband License":
//!    This software may be copied and distributed for educational, research,
//!    and not for profit purposes provided that this copyright and statement
//!    are included in all such copies.  Other copyrights may also apply.
//!
//! In C the functions return a `const char *` reason (NULL when prepared).
//! Here they return `Option<Reason>`: a fixed text (a C string literal), a
//! formatted text (C: written into the one static `borg_prepared_buffer`,
//! so a later call overwrites it; `borg_prepared_c`/`borg_prepared_buf`
//! keep that, with the buffer in `PreparedState`) or the reason of a
//! depth/restock formula line (C: a pointer into the formulas).

use std::ffi::CStr;

use crate::borg_formulas::{borg_prepared_dynamic, borg_restock_dynamic, DepthLine};
use crate::borg_magic::{borg_spell_legal_fail, borg_spell_stat};
use crate::borg_headers::borg_h::{
    BORG_KILLS_UNIQUES, BORG_NO_DEEPER, BORG_PLAYS_RISKY, BORG_USES_DYNAMIC_CALCS,
};
use crate::borg_headers::borg_magic_h::TELEPORT_LEVEL;
use crate::borg_headers::borg_trait_h::{
    BI_ACCW, BI_ACLW, BI_ACSW, BI_AESCAPE, BI_AEZHEAL, BI_AFUEL, BI_AHEAL, BI_ALIFE, BI_APHASE,
    BI_ASPEED, BI_ATELEPORT, BI_ATELEPORTLVL, BI_CDEPTH, BI_CLASS, BI_CON, BI_DEX, BI_DINV,
    BI_ESP, BI_FOOD, BI_FRACT, BI_KING, BI_LIGHT, BI_MAXCLEVEL, BI_MAXDEPTH, BI_MAXHP, BI_MAXSP,
    BI_RACID, BI_RCOLD, BI_RECALL, BI_RELEC, BI_SHLIFE, BI_SINV, BI_SPEED, BI_SRACID,
    BI_SRBLIND, BI_SRCOLD, BI_SRCONF, BI_SRDIS, BI_SRELEC, BI_SRFIRE, BI_SRKAOS, BI_SRPOIS,
    BI_STR,
};
use crate::params::{value, Param, SPECS};
use crate::state::BorgState;
use crate::engine;

const CLASS_WARRIOR: i32 = 0;
const CLASS_MAGE: i32 = 1;
const CLASS_DRUID: i32 = 2;
const CLASS_PRIEST: i32 = 3;
const CLASS_NECROMANCER: i32 = 4;
const CLASS_BLACKGUARD: i32 = 8;

/// borg-prepared.c globals: track how many uniques are around at your
/// depth (set by `borg_update()`).
#[derive(Default)]
pub struct PreparedState {
    /// `borg_numb_live_unique`
    pub numb_live_unique: i32,
    /// `borg_first_living_unique` (a race index)
    pub first_living_unique: u32,
    /// `borg_depth_hunted_unique`
    pub depth_hunted_unique: i32,
    /// `borg_prepared_buffer`: the text up to its NUL (empty at start, as
    /// the zeroed C static)
    pub buffer: Vec<u8>,
}

/// `MAX_REASON`: the size of `borg_prepared_buffer`
const MAX_REASON: usize = 1024;

/// `strnfmt(borg_prepared_buffer, MAX_REASON, "%s", s)`: at most
/// `MAX_REASON - 1` bytes; a reader sees the text up to the first NUL.
fn write_buffer(st: &mut BorgState, s: &[u8]) {
    let s = &s[..s.len().min(MAX_REASON - 1)];
    let n = s.iter().position(|&c| c == 0).unwrap_or(s.len());
    st.prepared.buffer = s[..n].to_vec();
}

/// The text C's `borg_prepared()`/`borg_restock()` return, copied out at
/// once: a formatted reason goes through `borg_prepared_buffer` as in C.
fn reason_c(st: &mut BorgState, r: Option<Reason>) -> Option<Vec<u8>> {
    let r = r?;
    Some(match &r.message {
        ReasonMessage::Formatted(s) => {
            write_buffer(st, s);
            st.prepared.buffer.clone()
        }
        _ => r.text(st).to_vec(),
    })
}

/// `borg_prepared(depth)` as C returns it (`None` for NULL), writing
/// `borg_prepared_buffer` as C does.
pub fn borg_prepared_c(st: &mut BorgState, depth: i32) -> Option<Vec<u8>> {
    let r = borg_prepared(st, depth);
    reason_c(st, r)
}

/// Live callers use the same check and C-compatible return, with a trace.
#[track_caller]
pub(crate) fn prepared_c_for_decision(st: &mut BorgState, depth: i32) -> Option<Vec<u8>> {
    let result = prepared_for_decision(st, depth);
    reason_c(st, result)
}

#[track_caller]
pub(crate) fn prepared_for_decision(st: &mut BorgState, depth: i32) -> Option<Reason> {
    let result = borg_prepared(st, depth);
    crate::core_trace::record_prepared(st, depth, result.as_ref(), std::panic::Location::caller());
    result
}

/// `borg_restock(depth)` as C returns it (as `borg_prepared_c`).
pub fn borg_restock_c(st: &mut BorgState, depth: i32) -> Option<Vec<u8>> {
    let r = restock_for_return(st, depth);
    reason_c(st, r)
}

/// What C's `borg_prepared()` returned (when not NULL): a text that stays
/// as it is, or the static `borg_prepared_buffer`, whose text the next
/// formatted reason overwrites (read it with `prepared_buffer`).
pub enum PreparedC {
    /// A fixed text (literal or formula reason)
    Text(Vec<u8>),
    /// `borg_prepared_buffer`
    Buffer,
}

/// `borg_prepared(depth)`, keeping C's pointer semantics (see
/// `PreparedC`). Writes `borg_prepared_buffer` as C does.
#[track_caller]
pub fn borg_prepared_buf(st: &mut BorgState, depth: i32) -> Option<PreparedC> {
    let r = prepared_for_decision(st, depth)?;
    Some(match &r.message {
        ReasonMessage::Formatted(s) => {
            write_buffer(st, s);
            PreparedC::Buffer
        }
        _ => PreparedC::Text(r.text(st).to_vec()),
    })
}

/// The text `borg_prepared_buffer` holds now.
pub fn prepared_buffer(st: &BorgState) -> Vec<u8> {
    st.prepared.buffer.clone()
}

/// Why the Borg is not prepared (or must restock).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Reason {
    message: ReasonMessage,
    pub(crate) requirement: Requirement,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Requirement {
    pub(crate) name: &'static str,
    pub(crate) actual: i32,
    pub(crate) required: i32,
    pub(crate) relation: &'static str,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum ReasonMessage {
    /// A fixed text (a string literal in C).
    Text(&'static str),
    /// An owned reason for a configurable requirement.
    Threshold(String),
    /// A text C formats into its static `borg_prepared_buffer`
    /// (`MAX_REASON` = 1024 bytes).
    Formatted(Vec<u8>),
    /// The reason of `st.formulas.depth[i]` (`borg_prepared_dynamic()`).
    Depth(usize),
    /// The reason of `st.formulas.restock[i]` (`borg_restock_dynamic()`).
    Restock(usize),
}

impl Reason {
    fn new(message: ReasonMessage, name: &'static str, actual: i32, required: i32, relation: &'static str) -> Self {
        Self { message, requirement: Requirement { name, actual, required, relation } }
    }

    pub(crate) fn fixed(text: &'static str, name: &'static str, actual: i32, required: i32) -> Self {
        Self::new(ReasonMessage::Text(text), name, actual, required, ">=")
    }

    fn threshold(text: String, name: &'static str, actual: i32, required: i32) -> Self {
        Self::new(ReasonMessage::Threshold(text), name, actual, required, ">=")
    }

    /// The text of the reason, as C would print it.
    pub fn text<'a>(&'a self, st: &'a BorgState) -> &'a [u8] {
        match &self.message {
            ReasonMessage::Text(s) => s.as_bytes(),
            ReasonMessage::Threshold(s) => s.as_bytes(),
            ReasonMessage::Formatted(s) => s,
            ReasonMessage::Depth(i) => st.formulas.depth[*i].reason.to_bytes(),
            ReasonMessage::Restock(i) => st.formulas.restock[*i].reason.to_bytes(),
        }
    }
}

/// The index of the formula line whose reason `r` is (`r` comes from
/// `borg_prepared_dynamic()`/`borg_restock_dynamic()` on these lines).
fn line_index(lines: &[DepthLine], r: *const CStr) -> usize {
    lines
        .iter()
        .position(|d| core::ptr::eq(d.reason.as_c_str(), r))
        .expect("formula reason comes from a formula line")
}

/// The original DL1–9 gates, in check order.
fn shallow_requirements(st: &BorgState, depth: i32, value: impl Fn(Param) -> i32) -> Option<Reason> {
    let b = &st.borg;
    let risky = st.cfg(BORG_PLAYS_RISKY) != 0;
    let class = b.tr(BI_CLASS);
    let known_class = (CLASS_WARRIOR..=CLASS_BLACKGUARD).contains(&class);
    let caster = matches!(class, CLASS_MAGE | CLASS_NECROMANCER);
    let hp = b.tr(BI_MAXHP);
    let clevel = b.tr(BI_MAXCLEVEL);

    // *** Essential Items for Level 1 ***

    // Require lite (any)
    if let Some(reason) = minimum(
        Param::prepared_early_min_light_radius,
        b.tr(BI_LIGHT),
        "1 Lite", "light_radius", &value,
    ) {
        return Some(reason);
    }

    // Require food
    if let Some(reason) = minimum(Param::prepared_early_min_food, b.tr(BI_FOOD), "5 Food", "food", &value) {
        return Some(reason);
    }

    // borg-reborn-rs (#557, owner): `prepared_recall_depth` moves C's
    // "1 recall" (from depth 5) shallower: below 5 it is checked here, after
    // light and food and before the DL1 and DL2 returns, so 1 and 2 gate
    // the depths they name; from 5 on it stays where C has it.
    let recall_depth = value(Param::prepared_recall_depth);
    if depth < 5 && recall_applies(depth, recall_depth) {
        if let Some(reason) = minimum(
            Param::prepared_early_min_recall,
            b.tr(BI_RECALL),
            "1 recall", "recall", &value,
        ) {
            return Some(reason);
        }
    }

    // Usually ready for level 1
    if depth <= 1 {
        return None;
    }

    // *** Essential Items for Level 2 ***

    // Require fuel
    if b.tr(BI_LIGHT) == 0 {
        if let Some(reason) = minimum(Param::prepared_early_min_fuel, b.tr(BI_AFUEL), "5 Fuel", "fuel", &value) {
            return Some(reason);
        }
    }

    // Require recall
    // if (borg.trait[BI_RECALL] < 1) return ("1 recall");

    // borg-prepared.c:borg_prepared_aux:82-86 (574b618c)
    let min_hp = value(Param::prepared_dl2_min_hp);
    if !risky && hp < min_hp {
        return Some(Reason::threshold(format!("{min_hp} hp"), "prepared_dl2_min_hp", hp, min_hp));
    }

    // The DL2 level gate is specific to DL2. Deeper proposals use their
    // own character-level checks.
    if depth <= 2 {
        let min_clevel = value(Param::prepared_dl2_min_clevel);
        if !risky && clevel < min_clevel {
            return Some(Reason::threshold(
                format!("{min_clevel} clevel"),
                "prepared_dl2_min_clevel",
                clevel,
                min_clevel,
            ));
        }
        return None;
    }

    // *** Essential Items for Level 3 and 4 ***

    // borg-prepared.c:borg_prepared_aux:94-137 (574b618c)
    if !risky && known_class {
        let min_hp = value(Param::prepared_dl34_min_hp);
        if hp < min_hp {
            return Some(Reason::threshold(format!("{min_hp} hp"), "prepared_dl34_min_hp", hp, min_hp));
        }
        let min_clevel = value(Param::prepared_dl34_min_clevel);
        if clevel < min_clevel {
            let unit = if matches!(class, CLASS_PRIEST | CLASS_DRUID) { "level" } else { "clevel" };
            return Some(Reason::threshold(format!("{min_clevel} {unit}"), "prepared_dl34_min_clevel", clevel, min_clevel));
        }
    }

    // borg-prepared.c:borg_prepared_aux:139-142 (574b618c)
    let cure_min = value(Param::prepared_cure_min);
    let cure_until = value(Param::prepared_cure_until_clevel);
    let cures = b.tr(BI_ACLW) + b.tr(BI_ACSW) + b.tr(BI_ACCW);
    if clevel < cure_until && cures < cure_min {
        return Some(Reason::threshold(format!("{cure_min} cure"), "prepared_cure_min", cures, cure_min));
    }

    // Usually ready for level 3 and 4
    if depth <= 4 {
        return None;
    }

    // *** Essential Items for Level 5 to 9 ***

    // borg-prepared.c:borg_prepared_aux:150-195 (574b618c)
    if !risky && known_class && b.tr(BI_CDEPTH) != 0 {
        let min_hp = value(Param::prepared_dl59_min_hp);
        if hp < min_hp {
            return Some(Reason::threshold(format!("{min_hp} hp"), "prepared_dl59_min_hp", hp, min_hp));
        }
        let min_clevel = value(Param::prepared_dl59_min_clevel);
        if clevel < min_clevel {
            let unit = if caster { "level" } else { "clevel" };
            return Some(Reason::threshold(format!("{min_clevel} {unit}"), "prepared_dl59_min_clevel", clevel, min_clevel));
        }
    }

    // borg-prepared.c:borg_prepared_aux:197-200 (574b618c)
    if clevel < cure_until && cures < cure_min {
        return Some(Reason::threshold(format!("{cure_min} cures (clw + csw + ccw)"), "prepared_cure_min", cures, cure_min));
    }

    // Scrolls of Word of Recall
    if recall_applies(depth, recall_depth) {
        return minimum(Param::prepared_early_min_recall, b.tr(BI_RECALL), "1 recall", "recall", &value);
    }

    None
}

/// Determine if the Borg meets the "minimum" requirements for a level
/// (`borg_prepared_aux`).
fn borg_prepared_aux(st: &mut BorgState, depth: i32) -> Option<Reason> {
    if -1 == st.borg.ready_morgoth {
        st.borg.ready_morgoth = 0;
    }
    if st.borg.tr(BI_KING) != 0 {
        st.borg.ready_morgoth = 1;
        return None;
    }

    // Always ready for the town
    if depth == 0 {
        return None;
    }

    let shallow = shallow_requirements(st, depth, value);
    if shallow.is_some() {
        return shallow;
    }

    deeper_requirements(st, depth, value, borg_spell_stat)
}

fn deeper_requirements(
    st: &BorgState,
    depth: i32,
    value: impl Fn(Param) -> i32,
    spell_stat: impl FnOnce() -> i32,
) -> Option<Reason> {
    let b = &st.borg;
    let risky = st.cfg(BORG_PLAYS_RISKY) != 0;
    let class = b.tr(BI_CLASS);
    let known_class = (CLASS_WARRIOR..=CLASS_BLACKGUARD).contains(&class);
    let caster = matches!(class, CLASS_MAGE | CLASS_NECROMANCER);
    let clevel = b.tr(BI_MAXCLEVEL);

    if depth >= value(Param::prepared_dl10_min_depth) {
        // Require light (radius 2), necromancers like the dark
        if let Some(reason) = minimum(
            Param::prepared_dl10_min_light_radius,
            b.tr(BI_LIGHT),
            "2 light radius", "light_radius", &value,
        ) {
            return Some(reason);
        }

        // Escape or Teleport
        if let Some(reason) = minimum(
            Param::prepared_dl10_min_teleports,
            b.tr(BI_ATELEPORT) + b.tr(BI_AESCAPE),
            "2 tele + teleport staffs", "teleports", &value,
        ) {
            return Some(reason);
        }

        // borg-prepared.c:borg_prepared_aux:221-253 (574b618c).
        // Mage and Necromancer have a level ceiling; the other classes have a
        // depth ceiling. Both start at prepared_dl10_min_depth.
        if !risky && known_class {
            let applies = if caster {
                clevel <= value(Param::prepared_mid_caster_max_clevel)
            } else {
                depth <= value(Param::prepared_mid_max_depth)
            };
            let offset = value(Param::prepared_mid_clevel_offset);
            if applies && clevel < depth + offset {
                let comparison = if caster { "> =" } else { ">=" };
                return Some(depth_offset_reason(offset, comparison, "prepared_mid_clevel_offset", clevel, depth + offset));
            }
        }

        // Potions of Cure Critical Wounds
        if b.tr(BI_MAXCLEVEL) < value(Param::prepared_dl10_cure_critical_until_clevel) {
            if let Some(reason) = minimum(
                Param::prepared_dl10_min_cure_critical,
                b.tr(BI_ACCW),
                "ccw < 3", "cure_critical", &value,
            ) {
                return Some(reason);
            }
        }

        // See invisible or telepathy
        if let Some(reason) = minimum(
            Param::prepared_dl10_min_see_invisible_detection_or_esp,
            b.tr(BI_SINV).max(b.tr(BI_DINV)).max(b.tr(BI_ESP)),
            "See Invis : ESP", "see_invisible_detection_or_esp", &value,
        ) {
            return Some(reason);
        }
    }

    if depth >= value(Param::prepared_dl20_min_depth) {
        // Free action
        if let Some(reason) = minimum(
            Param::prepared_dl20_min_free_action,
            b.tr(BI_FRACT),
            "free action", "free_action", &value,
        ) {
            return Some(reason);
        }
    }

    let first_spell_stat_depth = value(Param::prepared_dl21_min_depth)
        .min(value(Param::prepared_dl40_min_depth))
        .min(value(Param::prepared_dl46_min_depth));
    let spell_stat = if depth >= first_spell_stat_depth {
        spell_stat()
    } else {
        -1
    };
    let spell_stat_trait = || b.trait_[(BI_STR as i32 + spell_stat) as usize];
    if depth >= value(Param::prepared_dl21_min_depth) {
        // must have fire + 2 other basic resists
        if let Some(reason) = minimum(
            Param::prepared_dl21_min_resist_fire,
            b.tr(BI_SRFIRE),
            "resist fire", "resist_fire", &value,
        ) {
            return Some(reason);
        }
        if let Some(reason) = minimum(
            Param::prepared_dl21_min_basic_resists,
            b.tr(BI_RACID) + b.tr(BI_RCOLD) + b.tr(BI_RELEC),
            "2 basic resists", "basic_resists", &value,
        ) {
            return Some(reason);
        }
        // have some minimal stats
        if let Some(reason) = minimum(
            Param::prepared_dl21_min_strength,
            b.tr(BI_STR),
            "low STR", "strength", &value,
        ) {
            return Some(reason);
        }

        if spell_stat != -1 {
            if let Some(reason) = minimum(
                Param::prepared_dl21_min_spell_stat,
                spell_stat_trait(),
                "low spell stat", "spell_stat", &value,
            ) {
                return Some(reason);
            }
        }
        if let Some(reason) = minimum(
            Param::prepared_dl21_min_dexterity,
            b.tr(BI_DEX),
            "low DEX", "dexterity", &value,
        ) {
            return Some(reason);
        }
        if let Some(reason) = minimum(
            Param::prepared_dl21_min_constitution,
            b.tr(BI_CON),
            "low CON", "constitution", &value,
        ) {
            return Some(reason);
        }

        // borg-prepared.c:borg_prepared_aux:304-346 (574b618c)
        if !risky && known_class {
            let max_clevel = value(Param::prepared_deep_max_clevel);
            let offset = value(Param::prepared_deep_clevel_offset);
            if clevel >= value(Param::prepared_deep_min_clevel)
                && clevel <= max_clevel
                && clevel < depth + offset
            {
                return Some(depth_offset_reason(offset, ">=", "prepared_deep_clevel_offset", clevel, depth + offset));
            }
            if caster && clevel > max_clevel && clevel <= value(Param::prepared_deep_caster_ramp_max_clevel) {
                let base_depth = value(Param::prepared_deep_caster_ramp_base_depth);
                let depth_per_level = value(Param::prepared_deep_caster_ramp_depth_per_level);
                if base_depth + (clevel - max_clevel) * depth_per_level < depth {
                    return Some(Reason::threshold(
                        format!("(clevel-{max_clevel})*{depth_per_level}+{base_depth} < dlevel"),
                        "caster_ramp_depth", base_depth + (clevel - max_clevel) * depth_per_level, depth,
                    ));
                }
            }
        }
    }

    if depth >= value(Param::prepared_dl26_min_depth) {
        // All Basic resistance
        if let Some(reason) = minimum(
            Param::prepared_dl26_min_resist_cold,
            b.tr(BI_SRCOLD),
            "resist cold", "resist_cold", &value,
        ) {
            return Some(reason);
        }
        if let Some(reason) = minimum(
            Param::prepared_dl26_min_resist_electricity,
            b.tr(BI_SRELEC),
            "resist elec", "resist_electricity", &value,
        ) {
            return Some(reason);
        }
        if let Some(reason) = minimum(
            Param::prepared_dl26_min_resist_acid,
            b.tr(BI_SRACID),
            "resist acid", "resist_acid", &value,
        ) {
            return Some(reason);
        }

        // Escape and Teleport
        if let Some(reason) = minimum(
            Param::prepared_dl26_min_teleports,
            b.tr(BI_ATELEPORT) + b.tr(BI_AESCAPE),
            "6 tell + telep staffs", "teleports", &value,
        ) {
            return Some(reason);
        }

        // Cure Critical Wounds
        if b.tr(BI_MAXCLEVEL) < value(Param::prepared_dl26_cure_serious_or_critical_until_clevel) {
            if let Some(reason) = minimum(
                Param::prepared_dl26_min_cure_serious_or_critical,
                b.tr(BI_ACCW) + b.tr(BI_ACSW),
                "10 ccw + csw", "cure_serious_or_critical", &value,
            ) {
                return Some(reason);
            }
        }
    }

    if depth >= value(Param::prepared_dl34_min_depth) {
        // borg-prepared.c:borg_prepared_aux:375-377 (574b618c)
        let min_clevel = value(Param::prepared_dl34_plus_min_clevel);
        if !risky && clevel < min_clevel {
            return Some(Reason::threshold(format!("level {min_clevel}"), "prepared_dl34_plus_min_clevel", clevel, min_clevel));
        }
    }

    if depth >= value(Param::prepared_dl40_min_depth) {
        // Resist
        if let Some(reason) = minimum(
            Param::prepared_dl40_min_resist_poison,
            b.tr(BI_SRPOIS),
            "resist pois", "resist_poison", &value,
        ) {
            return Some(reason);
        }
        if let Some(reason) = minimum(
            Param::prepared_dl40_min_resist_confusion,
            b.tr(BI_SRCONF),
            "resist conf", "resist_confusion", &value,
        ) {
            return Some(reason);
        }

        if let Some(reason) = minimum(
            Param::prepared_dl40_min_strength,
            b.tr(BI_STR),
            "STR < 16", "strength", &value,
        ) {
            return Some(reason);
        }

        if spell_stat != -1 {
            if let Some(reason) = minimum(
                Param::prepared_dl40_min_spell_stat,
                spell_stat_trait(),
                "spell stat < 16", "spell_stat", &value,
            ) {
                return Some(reason);
            }
        }
        if let Some(reason) = minimum(
            Param::prepared_dl40_min_dexterity,
            b.tr(BI_DEX),
            "dex < 16", "dexterity", &value,
        ) {
            return Some(reason);
        }
        if let Some(reason) = minimum(
            Param::prepared_dl40_min_constitution,
            b.tr(BI_CON),
            "con < 16", "constitution", &value,
        ) {
            return Some(reason);
        }
    }

    if depth >= value(Param::prepared_dl46_min_depth) {
        //  Must have +5 speed after level 46
        if let Some(reason) = minimum(
            Param::prepared_dl46_min_speed,
            b.tr(BI_SPEED),
            "+5 speed", "speed", &value,
        ) {
            return Some(reason);
        }

        // Potions of heal
        if b.tr(BI_AEZHEAL) < value(Param::prepared_dl46_min_star_heal) {
            if let Some(reason) = minimum(
                Param::prepared_dl46_min_heal,
                b.tr(BI_AHEAL),
                "1 heal", "heal", &value,
            ) {
                return Some(reason);
            }
        }

        if !risky {
            if let Some(reason) = minimum(
                Param::prepared_dl46_min_max_hp,
                b.tr(BI_MAXHP),
                "HP 500", "max_hp", &value,
            ) {
                return Some(reason);
            }
        }

        // Higher stats
        if let Some(reason) = minimum(
            Param::prepared_dl46_min_strength,
            b.tr(BI_STR),
            "str < 18(40)", "strength", &value,
        ) {
            return Some(reason);
        }

        if spell_stat != -1 {
            if let Some(reason) = minimum(
                Param::prepared_dl46_min_spell_stat,
                spell_stat_trait(),
                "spell stat needs to be max", "spell_stat", &value,
            ) {
                return Some(reason);
            }
        }
        if let Some(reason) = minimum(
            Param::prepared_dl46_min_dexterity,
            b.tr(BI_DEX),
            "dex < 18 (60)", "dexterity", &value,
        ) {
            return Some(reason);
        }
        if let Some(reason) = minimum(
            Param::prepared_dl46_min_constitution,
            b.tr(BI_CON),
            "con < 18 (60)", "constitution", &value,
        ) {
            return Some(reason);
        }

        // Hold Life
        if b.tr(BI_MAXCLEVEL) < value(Param::prepared_dl46_hold_life_until_clevel) {
            if let Some(reason) = minimum(
                Param::prepared_dl46_min_hold_life,
                b.tr(BI_SHLIFE),
                "hold life", "hold_life", &value,
            ) {
                return Some(reason);
            }
        }
    }

    if depth >= value(Param::prepared_dl56_min_depth) {
        // Potions of heal
        if b.tr(BI_AEZHEAL) < value(Param::prepared_dl56_min_star_heal) {
            if let Some(reason) = minimum(
                Param::prepared_dl56_min_heal,
                b.tr(BI_AHEAL),
                "2 heal + *heal*", "heal", &value,
            ) {
                return Some(reason);
            }
        }

        // Resists
        if let Some(reason) = minimum(
            Param::prepared_dl56_min_resist_blindness,
            b.tr(BI_SRBLIND),
            "resist blind", "resist_blindness", &value,
        ) {
            return Some(reason);
        }

        // Telepathy, better have it by now
        if let Some(reason) = minimum(Param::prepared_dl56_min_esp, b.tr(BI_ESP), "ESP", "esp", &value) {
            return Some(reason);
        }
    }

    if depth >= value(Param::prepared_dl60_min_depth) {
        // Must have +10 speed
        if let Some(reason) = minimum(
            Param::prepared_dl60_min_speed,
            b.tr(BI_SPEED),
            "+10 speed", "speed", &value,
        ) {
            return Some(reason);
        }

        // Resists
        if let Some(reason) = minimum(
            Param::prepared_dl60_min_resist_chaos,
            b.tr(BI_SRKAOS),
            "resist chaos", "resist_chaos", &value,
        ) {
            return Some(reason);
        }
        if let Some(reason) = minimum(
            Param::prepared_dl60_min_resist_disenchantment,
            b.tr(BI_SRDIS),
            "resist disenchant", "resist_disenchantment", &value,
        ) {
            return Some(reason);
        }
    }

    if depth >= value(Param::prepared_dl81_min_depth) {
        // Minimal Speed
        if let Some(reason) = minimum(
            Param::prepared_dl81_min_speed,
            b.tr(BI_SPEED),
            "+20 Speed", "speed", &value,
        ) {
            return Some(reason);
        }
    }

    if depth >= value(Param::prepared_dl100_min_depth) && b.tr(BI_KING) == 0 {
        let iv = &st.item_val;
        if b.tr(BI_MAXSP) > value(Param::prepared_dl100_restore_mana_above_sp) {
            if let Some(reason) = minimum(
                Param::prepared_dl100_min_restore_mana,
                b.has[iv.kv_potion_restore_mana as usize],
                "10 restore mana", "restore_mana", &value,
            ) {
                return Some(reason);
            }
        }
        if let Some(reason) = minimum(
            Param::prepared_dl100_min_healing_potions,
            b.has[iv.kv_potion_healing as usize],
            "5 Heal", "healing_potions", &value,
        ) {
            return Some(reason);
        }
        if let Some(reason) = minimum(
            Param::prepared_dl100_min_star_heal_or_life,
            b.tr(BI_AEZHEAL) + b.tr(BI_ALIFE),
            "15 *heal* or life", "star_heal_or_life", &value,
        ) {
            return Some(reason);
        }
        if let Some(reason) = minimum(
            Param::prepared_dl100_min_speed_potions,
            b.tr(BI_ASPEED),
            "10 speed potions", "speed_potions", &value,
        ) {
            return Some(reason);
        }
    }

    None
}

/// Preserve the legacy message at the inherited threshold and report the
/// configured threshold when it changes.
fn minimum(
    param: Param,
    actual: i32,
    legacy_message: &'static str,
    name: &'static str,
    value: impl FnOnce(Param) -> i32,
) -> Option<Reason> {
    let required = value(param);
    if actual >= required {
        None
    } else if required == SPECS[param as usize].default {
        Some(Reason::fixed(legacy_message, name, actual, required))
    } else {
        Some(Reason::threshold(format!("{name}: {actual} < {required}"), name, actual, required))
    }
}

fn depth_offset_reason(offset: i32, comparison: &str, name: &'static str, actual: i32, required: i32) -> Reason {
    let depth = match offset {
        0 => "dlevel".to_owned(),
        1.. => format!("dlevel + {offset}"),
        _ => format!("dlevel - {}", -offset),
    };
    Reason::threshold(format!("{depth} {comparison} clevel"), name, actual, required)
}

/// Determine if the Borg is "prepared" for the given level
/// (`borg_prepared`).
///
/// This routine does not help him decide how to get ready for the
/// given level, so it must work closely with "borg_power()".
///
/// Note that we ignore any "town fear", and we allow fear of one
/// level up to and including the relevant depth.
///
/// Returns the reason you are not prepared.
pub fn borg_prepared(st: &mut BorgState, depth: i32) -> Option<Reason> {
    // Town and First level
    if depth == 1 {
        return None;
    }

    // borg-reborn-rs (#840): no descent while the kit score is below its
    // floor, or while the money-first hold is on (each its own switch).
    if let Some((name, score, need)) = crate::borg_kit::descent_blocked(st, depth) {
        return Some(Reason::threshold(format!("kit score {score}% of {need}%"), name, score, need));
    }

    borg_prepared_calculated(st, depth)
}

fn borg_prepared_calculated(st: &mut BorgState, depth: i32) -> Option<Reason> {
    if st.cfg(BORG_USES_DYNAMIC_CALCS) != 0 {
        // use the base restock so special checks can be done
        if let Some(reason) = borg_restock(st, depth) {
            return Some(reason);
        }

        let r: Option<*const CStr> = borg_prepared_dynamic(st, depth).map(|r| r as *const CStr);
        if let Some(r) = r {
            return Some(Reason::new(ReasonMessage::Depth(line_index(&st.formulas.depth, r)), "dynamic_preparedness_formula", 0, 1, ">="));
        }
    } else {
        // Not prepared if I need to restock
        if let Some(reason) = borg_restock(st, depth) {
            return Some(reason);
        }

        // borg-prepared.c:borg_prepared:560-562 (574b618c)
        let clevel = st.borg.tr(BI_MAXCLEVEL);
        let margin = value(Param::prepared_clevel_depth_margin);
        if clevel < 50 && clevel < depth - margin {
            let message = match margin {
                0 => ReasonMessage::Text("Clevel < depth"),
                1.. => ReasonMessage::Threshold(format!("Clevel < depth - {margin}")),
                _ => ReasonMessage::Threshold(format!("Clevel < depth + {}", -margin)),
            };
            return Some(Reason::new(message, "prepared_clevel_depth_margin", clevel, depth - margin, ">="));
        }

        // The inherited static entry point only uses the auxiliary bands through DL99.
        if depth <= 99 {
            if let Some(reason) = borg_prepared_aux(st, depth) {
                return Some(reason);
            }
        }

        // Keep the configured limit in its original static-only position.
        if depth >= st.cfg(BORG_NO_DEEPER) {
            let limit = st.cfg(BORG_NO_DEEPER);
            return Some(Reason::new(ReasonMessage::Formatted(format!("No deeper {limit}.").into_bytes()),
                "configured_depth_limit", depth, limit, "<"));
        }
    }

    // Once Morgoth is dead
    if st.borg.tr(BI_KING) != 0 {
        return None;
    }

    // Always okay from town (C returns `reason`, which is NULL here)
    if st.borg.tr(BI_CDEPTH) == 0 {
        return None;
    }

    // Scum on depth 80-81 for some *heal* potions
    let home = &st.home;
    let home_stock_depth = value(Param::prepared_endgame_stock_min_depth);
    let home_stock = value(Param::prepared_endgame_min_home_or_pack_heals);
    if depth >= home_stock_depth
        && (i32::from(home.num_ezheal) + i32::from(home.num_life) < home_stock
            && st.borg.tr(BI_AEZHEAL) + st.borg.tr(BI_ALIFE) < home_stock)
    {
        // Must know exact number of Potions  in home
        crate::borg_home_notice::borg_notice_home(st, None, false);

        let home = &st.home;
        return Some(Reason::new(ReasonMessage::Formatted(format!(
            "Scumming *Heal* potions ({} to go).", home_stock - i32::from(home.num_ezheal)
        ).into_bytes()), "home_star_heal_or_life", i32::from(home.num_ezheal) + i32::from(home.num_life), home_stock, ">="));
    }

    // Scum on depth 80-81 for lots of *Heal* potions preparatory for Endgame
    if depth >= home_stock_depth
        && st.borg.tr(BI_MAXDEPTH) >= value(Param::prepared_endgame_total_heals_min_max_depth)
    {
        let required = value(Param::prepared_endgame_min_total_heals);
        // Must know exact number of Potions  in home
        crate::borg_home_notice::borg_notice_home(st, None, false);

        let home = &st.home;
        let heals = i32::from(home.num_ezheal_true)
            + st.borg.tr(BI_AEZHEAL)
            + i32::from(home.num_life_true)
            + st.borg.tr(BI_ALIFE);
        if heals < required {
            return Some(Reason::new(ReasonMessage::Formatted(format!(
                "Scumming *Heal* potions ({} to go).", required - heals
            ).into_bytes()), "total_star_heal_or_life", heals, required, ">="));
        }

        // Return to town to get your stock from the home
        if st.borg.tr(BI_AEZHEAL) + st.borg.tr(BI_ALIFE) < required
            && home.num_ezheal_true >= 1
            && st.borg.tr(BI_MAXDEPTH) >= value(Param::prepared_endgame_collect_heals_min_max_depth)
        {
            return Some(Reason::new(ReasonMessage::Formatted(format!(
                "Collect from house ({} potions).",
                i32::from(home.num_ezheal_true) + i32::from(home.num_life_true)
            ).into_bytes()), "pack_star_heal_or_life", st.borg.tr(BI_AEZHEAL) + st.borg.tr(BI_ALIFE), required, ">="));
        }
    }

    let pr = &st.prepared;

    // Check to make sure the borg does not go below where 3 living
    // uniques are.
    if st.borg.tr(BI_MAXDEPTH) <= 98 {
        // don't check how many uniques are alive
        if st.cfg(BORG_KILLS_UNIQUES) == 0 {
            return None;
        }

        // are too many uniques alive
        if pr.numb_live_unique < 3 {
            return None;
        }

        // Check for the dlevel of the unique
        if depth <= pr.depth_hunted_unique {
            return None;
        }

        // Reset our description for not diving
        let name = engine::race_name(pr.first_living_unique as usize);
        return Some(Reason::new(ReasonMessage::Formatted([b"Must kill ", name, b"."].concat()), "hunted_unique_depth", depth, pr.depth_hunted_unique, "<="));
    } else if st.borg.tr(BI_MAXDEPTH) >= 98 && depth >= 98 {
        // check to make sure the borg does not go to level 100
        // unless all the uniques are dead.

        // -1 is unknown.
        st.borg.ready_morgoth = -1;

        // is only Morgoth alive?
        let pr = &st.prepared;
        if pr.numb_live_unique < 1 || pr.first_living_unique == st.borg.id.morgoth {
            if depth >= 99 {
                st.borg.ready_morgoth = 1;
            }
            return None;
        }

        // Under special cases allow the borg to dive to 99 then quickly
        // get his butt to dlevel 98
        if st.borg.tr(BI_MAXDEPTH) == 99
            && depth <= 98
            // Teleport Level spell or scroll
            && (borg_spell_legal_fail(st, TELEPORT_LEVEL, 20) || st.borg.tr(BI_ATELEPORTLVL) >= 1)
        {
            return None;
        }

        // Access the living unique obtained from borg_update()
        let name = engine::race_name(st.prepared.first_living_unique as usize);
        return Some(Reason::new(ReasonMessage::Formatted([name, b" still alive!"].concat()), "living_non_morgoth_unique", 1, 0, "<="));
    }
    None
}

/// Determine if the Borg is out of "crucial" supplies (`borg_restock`).
///
/// Note that we ignore "restock" issues for the first several turns
/// on each level, to prevent repeated "level bouncing".
pub fn borg_restock(st: &mut BorgState, depth: i32) -> Option<Reason> {
    restock(st, depth, true)
}

fn restock(st: &mut BorgState, depth: i32, check_cures: bool) -> Option<Reason> {
    if let Some(reason) = borg_restock_base(st, depth, check_cures) {
        return Some(reason);
    }
    // borg-reborn-rs (#840, remedies a and c): the kit is part of the rule at
    // every depth (each switch off: nothing here).
    crate::borg_kit::restock(st).map(|(text, actual, required)| Reason::fixed(text, "kit_restock", actual, required))
}

/// `borg_restock` as C has it.
fn borg_restock_base(st: &mut BorgState, depth: i32, check_cures: bool) -> Option<Reason> {
    // We are now looking at our preparedness
    if -1 == st.borg.ready_morgoth {
        st.borg.ready_morgoth = 0;
    }

    if st.cfg(BORG_USES_DYNAMIC_CALCS) != 0 {
        let r: Option<*const CStr> = borg_restock_dynamic(st, depth).map(|r| r as *const CStr);
        return r.map(|r| Reason::new(ReasonMessage::Restock(line_index(&st.formulas.restock, r)), "dynamic_restock_formula", 0, 1, ">="));
    }

    restock_static(st, depth, check_cures, value)
}

/// The part of `borg_restock` without dynamic formulas.
fn restock_static(st: &BorgState, depth: i32, check_cures: bool, value: impl Fn(Param) -> i32) -> Option<Reason> {
    let b = &st.borg;

    // Must have some lite
    if let Some(reason) = minimum(
        Param::restock_dl1_min_light_radius,
        b.tr(BI_LIGHT),
        "restock light radius < 1", "light_radius", &value,
    ) {
        return Some(reason);
    }

    // Must have "fuel"
    if b.tr(BI_LIGHT) == 0 {
        if let Some(reason) = minimum(
            Param::restock_dl1_min_fuel,
            b.tr(BI_AFUEL),
            "restock fuel", "fuel", &value,
        ) {
            return Some(reason);
        }
    }

    if depth >= value(Param::restock_dl2_min_depth) {
        // Must have "fuel"
        if b.tr(BI_LIGHT) == 0 {
            if let Some(reason) = minimum(
                Param::restock_dl2_min_fuel,
                b.tr(BI_AFUEL),
                "restock fuel < 2", "fuel", &value,
            ) {
                return Some(reason);
            }
        }

        // Must have "food"
        if let Some(reason) = minimum(
            Param::restock_dl2_min_food,
            b.tr(BI_FOOD),
            "restock food < 3", "food", &value,
        ) {
            return Some(reason);
        }

    }

    if depth >= value(Param::restock_dl6_min_depth) {
        // Must have "phase"
        if let Some(reason) = minimum(
            Param::restock_dl6_min_phase_door,
            b.tr(BI_APHASE),
            "restock phase door", "phase_door", &value,
        ) {
            return Some(reason);
        }

        // Potions of Cure Wounds
        if check_cures && b.tr(BI_MAXCLEVEL) < value(Param::restock_dl6_cures_until_clevel) {
            if let Some(reason) = minimum(
                Param::restock_dl6_min_cures,
                b.tr(BI_ACLW) + b.tr(BI_ACSW) + b.tr(BI_ACCW),
                "restock clw+csw+ccw", "cures", &value,
            ) {
                return Some(reason);
            }
        }
    }

    if depth >= value(Param::restock_dl10_min_depth) {
        // Must have good light
        if let Some(reason) = minimum(
            Param::restock_dl10_min_light_radius,
            b.tr(BI_LIGHT),
            "2 light radius", "light_radius", &value,
        ) {
            return Some(reason);
        }

        // Must have "cure"
        if check_cures && b.tr(BI_MAXCLEVEL) < value(Param::restock_dl10_cures_until_clevel) {
            if let Some(reason) = minimum(
                Param::restock_dl10_min_cures,
                b.tr(BI_ACLW) + b.tr(BI_ACSW) + b.tr(BI_ACCW),
                "restock clw + csw + ccw ", "cures", &value,
            ) {
                return Some(reason);
            }
        }

        // Must have "teleport"
        if let Some(reason) = minimum(
            Param::restock_dl10_min_teleports,
            b.tr(BI_ATELEPORT) + b.tr(BI_AESCAPE),
            "restock tele + tele staff < 2", "teleports", &value,
        ) {
            return Some(reason);
        }
    }

    if depth >= value(Param::restock_dl20_min_depth) {
        // Must have "cure"
        if check_cures && b.tr(BI_MAXCLEVEL) < value(Param::restock_dl20_cure_serious_or_critical_until_clevel) {
            if let Some(reason) = minimum(
                Param::restock_dl20_min_cure_serious_or_critical,
                b.tr(BI_ACSW) + b.tr(BI_ACCW),
                "restock csw + ccw < 4", "cure_serious_or_critical", &value,
            ) {
                return Some(reason);
            }
        }

        // Must have "teleport" or Staff
        if let Some(reason) = minimum(
            Param::restock_dl20_min_teleports,
            b.tr(BI_ATELEPORT) + b.tr(BI_AESCAPE),
            "restock 4 > teleport + teleport staff ", "teleports", &value,
        ) {
            return Some(reason);
        }
    }

    if depth >= value(Param::restock_dl36_min_depth) {
        // Must have Scroll of Teleport (or good 2nd choice)
        if let Some(reason) = minimum(
            Param::restock_dl36_min_teleport_or_teleport_level,
            b.tr(BI_ATELEPORT) + b.tr(BI_ATELEPORTLVL),
            "restock teleport + teleport level scrolls", "teleport_or_teleport_level", &value,
        ) {
            return Some(reason);
        }
    }

    if depth >= value(Param::restock_dl65_min_depth) {
        // Must have "Heal"
        if let Some(reason) = minimum(
            Param::restock_dl65_min_healing,
            b.tr(BI_AHEAL) + b.has[st.item_val.kv_rod_healing as usize] + b.tr(BI_AEZHEAL),
            "restock heal", "healing", &value,
        ) {
            return Some(reason);
        }
    }

    if depth >= value(Param::restock_dl100_min_depth) {
        // Must have "Heal"
        // If I just got to dlevel 100 and low on heals, get out now.
        if borg_timer(st, b.time.level) < value(Param::restock_dl100_arrival_turns) {
            if let Some(reason) = minimum(
                Param::restock_dl100_min_star_heal,
                b.tr(BI_AEZHEAL),
                "restock *heal*", "star_heal", &value,
            ) {
                return Some(reason);
            }
        }
    }

    None
}

/// `borg_timer(time_in)`: turns since `time_in`.
fn borg_timer(st: &BorgState, time_in: i32) -> i32 {
    st.borg.time.now - time_in
}

/// Does the Borg need to go back to town (`borg_must_return_to_town`)?
pub fn borg_must_return_to_town(st: &mut BorgState) -> Option<Reason> {
    // don't need to go to town if in town
    if st.borg.tr(BI_CDEPTH) == 0 {
        return None;
    }

    if let Some((held, required)) = crate::borg_healing_restock::return_requirement(st) {
        return Some(Reason::fixed("restock healing potions", "healing_potions", held, required));
    }

    // Always spend time on a level unless 100
    if borg_timer(st, st.borg.time.level) < 100 && st.borg.tr(BI_CDEPTH) != 100 {
        return None;
    }

    // borg-reborn-rs (#830): a cursed item that stays worn is worth a trip
    // for Remove Curse (the harmful curses are taken off at once, so this
    // is the mild curse on an item the Borg keeps).
    if crate::borg_cursed::needs_remove_curse(st) {
        return Some(Reason::fixed(
            "restock remove curse",
            "remove_curse",
            st.borg.has[st.item_val.kv_scroll_remove_curse as usize],
            1,
        ));
    }

    // need to return to town if restock is needed
    restock_for_return(st, st.borg.tr(BI_CDEPTH))
}

fn restock_for_return(st: &mut BorgState, depth: i32) -> Option<Reason> {
    // A failed potion trip defers the ordinary cure checks through DL35.
    // Preparedness and the deeper Heal and *Healing* checks still apply.
    let check_cures = !crate::borg_healing_restock::deferred(st);
    restock(st, depth, check_cures)
}

/// borg-reborn-rs (#557): a descent to `depth` needs its Recall reserve from
/// `recall_depth` on (C: 5; 0 = never).
fn recall_applies(depth: i32, recall_depth: i32) -> bool {
    recall_depth > 0 && depth >= recall_depth
}

#[cfg(test)]
mod tests {
    use super::recall_applies;

    #[test]
    fn class_overrides_move_prep_and_restock_gates_independently() {
        use super::*;
        use crate::params::test_class_values;

        let config = "borg_param_prepared_dl20_min_depth__warrior = 19\n\
                      borg_param_restock_dl20_min_teleports__warrior = 5";
        let warrior = test_class_values(config, CLASS_WARRIOR as usize);
        let mage = test_class_values(config, CLASS_MAGE as usize);
        let mut st = BorgState::default();
        st.core.cfg = crate::borg_init::BORG_SETTINGS.iter().map(|s| s.default_value).collect();
        st.borg.trait_ = vec![0; crate::borg_headers::borg_trait_h::BI_MAX as usize];
        for (index, actual) in [
            (BI_LIGHT, 2), (BI_FOOD, 5), (BI_APHASE, 1), (BI_ATELEPORT, 4),
            (BI_MAXCLEVEL, 30), (BI_SINV, 1),
        ] {
            st.borg.trait_[index as usize] = actual;
        }

        assert!(deeper_requirements(&st, 18, &warrior, || -1).is_none());
        let reason = deeper_requirements(&st, 19, &warrior, || -1).unwrap();
        assert_eq!(reason.requirement, Requirement {
            name: "free_action", actual: 0, required: 1, relation: ">=",
        });
        assert!(restock_static(&st, 19, true, &warrior).is_none());
        let reason = restock_static(&st, 20, true, &warrior).unwrap();
        assert_eq!(reason.requirement, Requirement {
            name: "teleports", actual: 4, required: 5, relation: ">=",
        });

        st.borg.trait_[BI_CLASS as usize] = CLASS_MAGE;
        assert!(deeper_requirements(&st, 19, &mage, || -1).is_none());
        assert!(restock_static(&st, 20, true, &mage).is_none());
    }

    #[test]
    fn ranger_and_caster_keep_their_deep_level_transitions() {
        use super::*;
        use crate::params::{test_class_values, CLASSES};

        let mut st = BorgState::default();
        st.core.cfg = crate::borg_init::BORG_SETTINGS.iter().map(|s| s.default_value).collect();
        st.borg.trait_ = vec![0; crate::borg_headers::borg_trait_h::BI_MAX as usize];
        for (index, actual) in [
            (BI_LIGHT, 2), (BI_ATELEPORT, 8), (BI_ACCW, 12), (BI_SINV, 1), (BI_FRACT, 1),
            (BI_SRFIRE, 1), (BI_RACID, 1), (BI_RCOLD, 1), (BI_SRCOLD, 1),
            (BI_SRELEC, 1), (BI_SRACID, 1), (BI_STR, 9), (BI_DEX, 9), (BI_CON, 9),
        ] {
            st.borg.trait_[index as usize] = actual;
        }
        let shipped = crate::config_file::yaml_settings(include_str!("../../harness/pborg.yaml")).unwrap();
        let ranger = CLASSES.iter().position(|class| *class == "ranger").unwrap();
        let ranger_value = test_class_values(&shipped, ranger);
        st.borg.trait_[BI_CLASS as usize] = ranger as i32;
        st.borg.trait_[BI_MAXCLEVEL as usize] = 28;
        let reason = deeper_requirements(&st, 21, &ranger_value, || -1).unwrap();
        assert_eq!(reason.requirement.name, "prepared_deep_clevel_offset");
        assert_eq!(reason.requirement.required, 31);
        st.borg.trait_[BI_MAXCLEVEL as usize] = 31;
        assert!(deeper_requirements(&st, 21, &ranger_value, || -1).is_none());

        for caster in [CLASS_MAGE, CLASS_NECROMANCER] {
            let caster_value = test_class_values(&shipped, caster as usize);
            st.borg.trait_[BI_CLASS as usize] = caster;
            st.borg.trait_[BI_MAXCLEVEL as usize] = 40;
            assert!(deeper_requirements(&st, 30, &caster_value, || -1).is_none());
            let reason = deeper_requirements(&st, 31, &caster_value, || -1).unwrap();
            assert_eq!(reason.requirement.name, "prepared_deep_clevel_offset");

            st.borg.trait_[BI_MAXCLEVEL as usize] = 41;
            assert!(deeper_requirements(&st, 32, &caster_value, || -1).is_none());
            let reason = deeper_requirements(&st, 33, &caster_value, || -1).unwrap();
            assert_eq!(reason.requirement.name, "caster_ramp_depth");
            assert_eq!(reason.requirement.actual, 32);
        }
    }

    #[test]
    fn morgoth_band_preserves_all_four_kit_checks() {
        use super::*;
        use crate::params::test_class_values;

        let shipped = crate::config_file::yaml_settings(include_str!("../../harness/pborg.yaml")).unwrap();
        let value = test_class_values(&shipped, CLASS_WARRIOR as usize);
        let mut st = BorgState::default();
        st.core.cfg = crate::borg_init::BORG_SETTINGS.iter().map(|s| s.default_value).collect();
        st.borg.trait_ = vec![1; crate::borg_headers::borg_trait_h::BI_MAX as usize];
        for (index, actual) in [
            (BI_KING, 0), (BI_LIGHT, 2), (BI_MAXCLEVEL, 50), (BI_MAXHP, 600),
            (BI_MAXSP, 101), (BI_ATELEPORT, 8), (BI_SPEED, 132), (BI_STR, 118),
            (BI_DEX, 118), (BI_CON, 118), (BI_AHEAL, 3), (BI_AEZHEAL, 2),
        ] {
            st.borg.trait_[index as usize] = actual;
        }
        st.item_val.kv_potion_restore_mana = 0;
        st.item_val.kv_potion_healing = 1;
        st.borg.has = vec![0; 2];

        assert!(deeper_requirements(&st, 99, &value, || -1).is_none());
        let failure = |st: &BorgState| deeper_requirements(st, 100, &value, || -1).unwrap().requirement;
        assert_eq!(failure(&st).name, "restore_mana");
        st.borg.has[0] = 15;
        assert_eq!(failure(&st).name, "healing_potions");
        st.borg.has[1] = 5;
        assert_eq!(failure(&st).name, "star_heal_or_life");
        st.borg.trait_[BI_ALIFE as usize] = 13;
        assert_eq!(failure(&st).name, "speed_potions");
        st.borg.trait_[BI_ASPEED as usize] = 10;
        assert!(deeper_requirements(&st, 100, &value, || -1).is_none());
        st.borg.has.fill(0);
        st.borg.trait_[BI_KING as usize] = 1;
        assert!(deeper_requirements(&st, 100, &value, || -1).is_none());
    }

    #[test]
    fn recall_reserve_applies_on_both_sides_of_dl5() {
        use super::*;
        use crate::params::test_class_values;

        let warrior = test_class_values(
            "borg_param_prepared_recall_depth = 4\n\
             borg_param_prepared_early_min_recall__warrior = 2",
            CLASS_WARRIOR as usize,
        );
        let mut st = BorgState::default();
        st.core.cfg = crate::borg_init::BORG_SETTINGS.iter().map(|s| s.default_value).collect();
        st.borg.trait_ = vec![0; crate::borg_headers::borg_trait_h::BI_MAX as usize];
        for (index, actual) in [
            (BI_LIGHT, 1), (BI_FOOD, 5), (BI_MAXHP, 100), (BI_MAXCLEVEL, 30), (BI_RECALL, 1),
        ] {
            st.borg.trait_[index as usize] = actual;
        }
        assert!(shallow_requirements(&st, 3, &warrior).is_none());
        for depth in [4, 5] {
            let reason = shallow_requirements(&st, depth, &warrior).unwrap();
            assert_eq!(reason.requirement, Requirement {
                name: "recall", actual: 1, required: 2, relation: ">=",
            });
        }
    }

    #[test]
    fn deferring_cures_keeps_the_deep_healing_requirements() {
        use super::*;
        let mut st = BorgState::default();
        st.borg.trait_ = vec![0; crate::borg_headers::borg_trait_h::BI_MAX as usize];
        st.borg.has = vec![0];
        for (index, value) in [
            (BI_LIGHT, 2), (BI_FOOD, 3), (BI_APHASE, 1),
            (BI_ATELEPORT, 4), (BI_ATELEPORTLVL, 1),
        ] {
            st.borg.trait_[index as usize] = value;
        }
        assert!(restock_static(&st, 6, false, value).is_none());
        assert_eq!(restock_static(&st, 65, false, value).unwrap().requirement.name, "healing");
        st.borg.trait_[BI_AHEAL as usize] = 1;
        assert_eq!(restock_static(&st, 100, false, value).unwrap().requirement.name, "star_heal");
    }

    #[test]
    fn shallow_gates_keep_first_failure_and_report_the_compared_operands() {
        use super::*;
        let mut st = BorgState::default();
        st.borg.trait_ = vec![0; crate::borg_headers::borg_trait_h::BI_MAX as usize];
        st.core.cfg = crate::borg_init::BORG_SETTINGS.iter().map(|s| s.default_value).collect();
        st.borg.trait_[BI_CDEPTH as usize] = 4;
        let fail = |st: &BorgState| shallow_requirements(st, 5, value).unwrap().requirement;
        assert_eq!(fail(&st), Requirement { name: "light_radius", actual: 0, required: 1, relation: ">=" });
        st.borg.trait_[BI_LIGHT as usize] = 1;
        assert_eq!(fail(&st), Requirement { name: "food", actual: 0, required: 5, relation: ">=" });
        assert_eq!(restock_static(&st, 5, true, value).unwrap().requirement,
            Requirement { name: "food", actual: 0, required: 3, relation: ">=" });
        st.borg.trait_[BI_FOOD as usize] = 5;
        assert_eq!(fail(&st), Requirement { name: "prepared_dl2_min_hp", actual: 0, required: 30, relation: ">=" });
        st.borg.trait_[BI_MAXHP as usize] = 30;
        let hp = fail(&st);
        assert_eq!((hp.name, hp.actual), ("prepared_dl34_min_hp", 30));
        assert!(hp.required >= 40);
        st.borg.trait_[BI_MAXHP as usize] = 100;
        let cl = fail(&st);
        assert_eq!(cl.name, "prepared_dl34_min_clevel");
        assert_eq!(cl.actual, 0);
        assert!(cl.required >= 4);
        st.borg.trait_[BI_MAXCLEVEL as usize] = 20;
        assert_eq!(fail(&st), Requirement { name: "prepared_cure_min", actual: 0, required: 2, relation: ">=" });
        st.borg.trait_[BI_ACSW as usize] = 1;
        st.borg.trait_[BI_ACCW as usize] = 1;
        st.borg.trait_[BI_MAXHP as usize] = 60;
        let deep = fail(&st);
        if deep.name == "prepared_dl59_min_hp" {
            assert_eq!(deep.actual, 60);
            assert!(deep.required > 60);
        } else {
            assert_eq!(deep, Requirement { name: "recall", actual: 0, required: 1, relation: ">=" });
        }
        st.borg.trait_[BI_MAXHP as usize] = 100;
        assert_eq!(fail(&st), Requirement { name: "recall", actual: 0, required: 1, relation: ">=" });
        st.borg.trait_[BI_RECALL as usize] = 1;
        assert!(shallow_requirements(&st, 5, value).is_none());
    }

    #[test]
    fn shallow_ranger_level_gates_report_the_boundary_even_at_low_hp() {
        const CLASS_RANGER: i32 = 7;
        use super::*;
        let mut st = BorgState::default();
        st.borg.trait_ = vec![0; crate::borg_headers::borg_trait_h::BI_MAX as usize];
        st.core.cfg = crate::borg_init::BORG_SETTINGS.iter().map(|s| s.default_value).collect();
        st.borg.trait_[BI_CLASS as usize] = CLASS_RANGER;
        st.borg.trait_[BI_CDEPTH as usize] = 1;
        st.borg.trait_[BI_LIGHT as usize] = 1;
        st.borg.trait_[BI_FOOD as usize] = 5;
        st.borg.trait_[BI_MAXHP as usize] = 37;
        st.borg.trait_[BI_ACLW as usize] = 2;
        st.borg.trait_[BI_RECALL as usize] = 1;
        let ranger = |param| match param {
            Param::prepared_dl2_min_hp | Param::prepared_dl34_min_hp | Param::prepared_dl59_min_hp => 0,
            Param::prepared_dl2_min_clevel => 8,
            Param::prepared_dl34_min_clevel => 9,
            Param::prepared_dl59_min_clevel => 11,
            _ => value(param),
        };

        st.borg.trait_[BI_MAXCLEVEL as usize] = 4;
        assert!(shallow_requirements(&st, 1, ranger).is_none());
        assert_eq!(shallow_requirements(&st, 2, ranger).unwrap().requirement,
            Requirement { name: "prepared_dl2_min_clevel", actual: 4, required: 8, relation: ">=" });
        assert_eq!(shallow_requirements(&st, 3, ranger).unwrap().requirement,
            Requirement { name: "prepared_dl34_min_clevel", actual: 4, required: 9, relation: ">=" });

        for (depth, name, required) in [
            (2, "prepared_dl2_min_clevel", 8),
            (3, "prepared_dl34_min_clevel", 9),
            (4, "prepared_dl34_min_clevel", 9),
            (5, "prepared_dl59_min_clevel", 11),
        ] {
            st.borg.trait_[BI_CDEPTH as usize] = depth - 1;
            st.borg.trait_[BI_MAXCLEVEL as usize] = required - 1;
            assert_eq!(shallow_requirements(&st, depth, ranger).unwrap().requirement,
                Requirement { name, actual: required - 1, required, relation: ">=" });
            st.borg.trait_[BI_MAXCLEVEL as usize] = required;
            assert!(shallow_requirements(&st, depth, ranger).is_none(), "DL{depth}");
        }

        st.borg.trait_[BI_MAXCLEVEL as usize] = 4;
        st.core.cfg[BORG_PLAYS_RISKY as usize] = 1;
        assert!(shallow_requirements(&st, 5, ranger).is_none());
    }

    #[test]
    fn a_descent_needs_a_recall_from_the_set_depth() {
        assert!(!recall_applies(4, 5), "C: DL4 without one");
        assert!(recall_applies(5, 5), "C: DL5 needs one");
        assert!(recall_applies(2, 2), "moved to DL2");
        assert!(!recall_applies(1, 2));
        assert!(!recall_applies(30, 0), "0 = never");
    }
}
