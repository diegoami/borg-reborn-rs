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
use crate::params::{value, Param};
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
    let r = borg_restock(st, depth);
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
fn shallow_requirements(st: &BorgState, depth: i32) -> Option<Reason> {
    let b = &st.borg;
    let risky = st.cfg(BORG_PLAYS_RISKY) != 0;
    let class = b.tr(BI_CLASS);
    let known_class = (CLASS_WARRIOR..=CLASS_BLACKGUARD).contains(&class);
    let caster = matches!(class, CLASS_MAGE | CLASS_NECROMANCER);
    let hp = b.tr(BI_MAXHP);
    let clevel = b.tr(BI_MAXCLEVEL);

    // *** Essential Items for Level 1 ***

    // Require lite (any)
    if b.tr(BI_LIGHT) < 1 {
        return Some(Reason::fixed("1 Lite", "light_radius", b.tr(BI_LIGHT), 1));
    }

    // Require food
    if b.tr(BI_FOOD) < 5 {
        return Some(Reason::fixed("5 Food", "food", b.tr(BI_FOOD), 5));
    }

    // borg-reborn-rs (#557, owner): `prepared_recall_depth` moves C's
    // "1 recall" (from depth 5) shallower: below 5 it is checked here, after
    // light and food and before the DL1 and DL2 returns, so 1 and 2 gate
    // the depths they name; from 5 on it stays where C has it.
    let recall_depth = value(Param::prepared_recall_depth);
    if depth < 5 && needs_recall(depth, recall_depth, b.tr(BI_RECALL)) {
        return Some(Reason::fixed("1 recall", "recall", b.tr(BI_RECALL), 1));
    }

    // Usually ready for level 1
    if depth <= 1 {
        return None;
    }

    // *** Essential Items for Level 2 ***

    // Require fuel
    if b.tr(BI_AFUEL) < 5 && b.tr(BI_LIGHT) == 0 {
        return Some(Reason::fixed("5 Fuel", "fuel", b.tr(BI_AFUEL), 5));
    }

    // Require recall
    // if (borg.trait[BI_RECALL] < 1) return ("1 recall");

    // borg-prepared.c:borg_prepared_aux:82-86 (574b618c)
    let min_hp = value(Param::prepared_dl2_min_hp);
    if !risky && hp < min_hp {
        return Some(Reason::threshold(format!("{min_hp} hp"), "prepared_dl2_min_hp", hp, min_hp));
    }

    // Usually ready for level 2
    if depth <= 2 {
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
    if needs_recall(depth, recall_depth, b.tr(BI_RECALL)) {
        return Some(Reason::fixed("1 recall", "recall", b.tr(BI_RECALL), 1));
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

    let shallow = shallow_requirements(st, depth);
    if shallow.is_some() || depth <= 9 {
        return shallow;
    }

    let b = &st.borg;
    let risky = st.cfg(BORG_PLAYS_RISKY) != 0;
    let class = b.tr(BI_CLASS);
    let known_class = (CLASS_WARRIOR..=CLASS_BLACKGUARD).contains(&class);
    let caster = matches!(class, CLASS_MAGE | CLASS_NECROMANCER);
    let clevel = b.tr(BI_MAXCLEVEL);

    // *** Essential Items for Level 10 to 19 ***

    // Require light (radius 2), necromancers like the dark
    if b.tr(BI_LIGHT) < 2 && b.tr(BI_CLASS) != CLASS_NECROMANCER {
        return Some(Reason::fixed("2 light radius", "light_radius", b.tr(BI_LIGHT), 2));
    }

    // Escape or Teleport
    if b.tr(BI_ATELEPORT) + b.tr(BI_AESCAPE) < 2 {
        return Some(Reason::fixed("2 tele + teleport staffs", "teleports", b.tr(BI_ATELEPORT) + b.tr(BI_AESCAPE), 2));
    }

    // borg-prepared.c:borg_prepared_aux:221-253 (574b618c).
    // Mage and Necromancer have a level ceiling; the other classes have a
    // depth ceiling. The DL9 return above determines where this check starts.
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
    if b.tr(BI_MAXCLEVEL) < 30 && b.tr(BI_ACCW) < 3 {
        return Some(Reason::fixed("ccw < 3", "cure_critical", b.tr(BI_ACCW), 3));
    }

    // See invisible or telepathy
    if b.tr(BI_SINV) == 0 && b.tr(BI_DINV) == 0 && b.tr(BI_ESP) == 0 {
        return Some(Reason::fixed("See Invis : ESP", "see_invisible_detection_or_esp", b.tr(BI_SINV).max(b.tr(BI_DINV)).max(b.tr(BI_ESP)), 1));
    }

    // Usually ready for level 10 to 19
    if depth <= 19 {
        return None;
    }

    // *** Essential Items for Level 20 ***

    // Free action
    if b.tr(BI_FRACT) == 0 {
        return Some(Reason::fixed("free action", "free_action", b.tr(BI_FRACT), 1));
    }

    // ready for level 20
    if depth <= 20 {
        return None;
    }

    // *** Essential Items for Level 25 ***

    // must have fire + 2 other basic resists
    if b.tr(BI_SRFIRE) == 0 {
        return Some(Reason::fixed("resist fire", "resist_fire", b.tr(BI_SRFIRE), 1));
    }
    {
        let basics = b.tr(BI_RACID) + b.tr(BI_RCOLD) + b.tr(BI_RELEC);

        if basics < 2 {
            return Some(Reason::fixed("2 basic resists", "basic_resists", basics, 2));
        }
    }
    // have some minimal stats
    if b.tr(BI_STR) < 7 {
        return Some(Reason::fixed("low STR", "strength", b.tr(BI_STR), 7));
    }

    let spell_stat = borg_spell_stat();
    // `borg.trait[BI_STR + spell_stat]`
    let spell_stat_trait = || b.trait_[(BI_STR as i32 + spell_stat) as usize];
    if spell_stat != -1 && spell_stat_trait() < 7 {
        return Some(Reason::fixed("low spell stat", "spell_stat", spell_stat_trait(), 7));
    }
    if b.tr(BI_DEX) < 7 {
        return Some(Reason::fixed("low DEX", "dexterity", b.tr(BI_DEX), 7));
    }
    if b.tr(BI_CON) < 7 {
        return Some(Reason::fixed("low CON", "constitution", b.tr(BI_CON), 7));
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

    // Ready for level 25
    if depth <= 25 {
        return None;
    }

    // *** Essential Items for Level 25 to 39 ***

    // All Basic resistance
    if b.tr(BI_SRCOLD) == 0 {
        return Some(Reason::fixed("resist cold", "resist_cold", b.tr(BI_SRCOLD), 1));
    }
    if b.tr(BI_SRELEC) == 0 {
        return Some(Reason::fixed("resist elec", "resist_electricity", b.tr(BI_SRELEC), 1));
    }
    if b.tr(BI_SRACID) == 0 {
        return Some(Reason::fixed("resist acid", "resist_acid", b.tr(BI_SRACID), 1));
    }

    // Escape and Teleport
    if b.tr(BI_ATELEPORT) + b.tr(BI_AESCAPE) < 6 {
        return Some(Reason::fixed("6 tell + telep staffs", "teleports", b.tr(BI_ATELEPORT) + b.tr(BI_AESCAPE), 6));
    }

    // Cure Critical Wounds
    if b.tr(BI_MAXCLEVEL) < 30 && b.tr(BI_ACCW) + b.tr(BI_ACSW) < 10 {
        return Some(Reason::fixed("10 ccw + csw", "cure_serious_or_critical", b.tr(BI_ACCW) + b.tr(BI_ACSW), 10));
    }

    // Ready for level 33
    if depth <= 33 {
        return None;
    }

    // borg-prepared.c:borg_prepared_aux:375-377 (574b618c)
    let min_clevel = value(Param::prepared_dl34_plus_min_clevel);
    if !risky && clevel < min_clevel {
        return Some(Reason::threshold(format!("level {min_clevel}"), "prepared_dl34_plus_min_clevel", clevel, min_clevel));
    }

    // Usually ready for level 20 to 39
    if depth <= 39 {
        return None;
    }

    // *** Essential Items for Level 40 to 45 ***

    // Resist
    if b.tr(BI_SRPOIS) == 0 {
        return Some(Reason::fixed("resist pois", "resist_poison", b.tr(BI_SRPOIS), 1));
    }
    if b.tr(BI_SRCONF) == 0 {
        return Some(Reason::fixed("resist conf", "resist_confusion", b.tr(BI_SRCONF), 1));
    }

    if b.tr(BI_STR) < 16 {
        return Some(Reason::fixed("STR < 16", "strength", b.tr(BI_STR), 16));
    }

    if spell_stat != -1 && spell_stat_trait() < 16 {
        return Some(Reason::fixed("spell stat < 16", "spell_stat", spell_stat_trait(), 16));
    }
    if b.tr(BI_DEX) < 16 {
        return Some(Reason::fixed("dex < 16", "dexterity", b.tr(BI_DEX), 16));
    }
    if b.tr(BI_CON) < 16 {
        return Some(Reason::fixed("con < 16", "constitution", b.tr(BI_CON), 16));
    }

    // Ok to continue
    if depth <= 45 {
        return None;
    }

    // *** Essential Items for Level 46 to 55 ***

    //  Must have +5 speed after level 46
    if b.tr(BI_SPEED) < 115 {
        return Some(Reason::fixed("+5 speed", "speed", b.tr(BI_SPEED), 115));
    }

    // Potions of heal
    if b.tr(BI_AHEAL) < 1 && b.tr(BI_AEZHEAL) < 1 {
        return Some(Reason::fixed("1 heal", "heal", b.tr(BI_AHEAL).max(b.tr(BI_AEZHEAL)), 1));
    }

    if !risky {
        // Minimal hitpoints
        if b.tr(BI_MAXHP) < 500 {
            return Some(Reason::fixed("HP 500", "max_hp", b.tr(BI_MAXHP), 500));
        }
    }

    // High stats XXX XXX XXX
    if b.tr(BI_STR) < 18 + 40 {
        return Some(Reason::fixed("str < 18(40)", "strength", b.tr(BI_STR), 18 + 40));
    }

    if spell_stat != -1 && spell_stat_trait() < 18 + 100 {
        return Some(Reason::fixed("spell stat needs to be max", "spell_stat", spell_stat_trait(), 18 + 100));
    }
    if b.tr(BI_DEX) < 18 + 60 {
        return Some(Reason::fixed("dex < 18 (60)", "dexterity", b.tr(BI_DEX), 18 + 60));
    }
    if b.tr(BI_CON) < 18 + 60 {
        return Some(Reason::fixed("con < 18 (60)", "constitution", b.tr(BI_CON), 18 + 60));
    }

    // Hold Life
    if b.tr(BI_SHLIFE) == 0 && b.tr(BI_MAXCLEVEL) < 50 {
        return Some(Reason::fixed("hold life", "hold_life", b.tr(BI_SHLIFE), 1));
    }

    // Usually ready for level 46 to 55
    if depth <= 55 {
        return None;
    }

    // *** Essential Items for Level 55 to 59 ***

    // Potions of heal
    if b.tr(BI_AHEAL) < 2 && b.tr(BI_AEZHEAL) < 1 {
        return Some(Reason::fixed("2 heal + *heal*", "heal", b.tr(BI_AHEAL).max(2 * b.tr(BI_AEZHEAL)), 2));
    }

    // Resists
    if b.tr(BI_SRBLIND) == 0 {
        return Some(Reason::fixed("resist blind", "resist_blindness", b.tr(BI_SRBLIND), 1));
    }

    // Must have resist nether
    // if (!borg_settings[BORG_PLAYS_RISKY] && !borg.trait[BI_SRNTHR]) return
    // ("resist nether");

    // Telepathy, better have it by now
    if b.tr(BI_ESP) == 0 {
        return Some(Reason::fixed("ESP", "esp", b.tr(BI_ESP), 1));
    }

    // Usually ready for level 55 to 59
    if depth <= 59 {
        return None;
    }

    // *** Essential Items for Level 61 to 80 ***

    // Must have +10 speed
    if b.tr(BI_SPEED) < 120 {
        return Some(Reason::fixed("+10 speed", "speed", b.tr(BI_SPEED), 120));
    }

    // Resists
    if b.tr(BI_SRKAOS) == 0 {
        return Some(Reason::fixed("resist chaos", "resist_chaos", b.tr(BI_SRKAOS), 1));
    }
    if b.tr(BI_SRDIS) == 0 {
        return Some(Reason::fixed("resist disenchant", "resist_disenchantment", b.tr(BI_SRDIS), 1));
    }

    // Usually ready for level 61 to 80
    if depth <= 80 {
        return None;
    }

    // *** Essential Items for Level 81-85 ***
    // Minimal Speed
    if b.tr(BI_SPEED) < 130 {
        return Some(Reason::fixed("+20 Speed", "speed", b.tr(BI_SPEED), 130));
    }

    // Usually ready for level 81 to 85
    if depth <= 85 {
        return None;
    }

    // *** Essential Items for Level 86-99 ***

    // Usually ready for level 86 to 99
    if depth <= 99 {
        return None;
    }

    // *** Essential Items for Level 100 ***

    // must have lots of restore mana to go after MORGOTH
    if b.tr(BI_KING) == 0 {
        let iv = &st.item_val;
        if b.tr(BI_MAXSP) > 100 && b.has[iv.kv_potion_restore_mana as usize] < 15 {
            return Some(Reason::fixed("10 restore mana", "restore_mana", b.has[iv.kv_potion_restore_mana as usize], 15));
        }

        // must have lots of heal
        if b.has[iv.kv_potion_healing as usize] < 5 {
            return Some(Reason::fixed("5 Heal", "healing_potions", b.has[iv.kv_potion_healing as usize], 5));
        }

        // must have lots of ez-heal
        if b.tr(BI_AEZHEAL) + b.tr(BI_ALIFE) < 15 {
            return Some(Reason::fixed("15 *heal* or life", "star_heal_or_life", b.tr(BI_AEZHEAL) + b.tr(BI_ALIFE), 15));
        }

        // must have lots of speed
        if b.tr(BI_ASPEED) < 10 {
            return Some(Reason::fixed("10 speed potions", "speed_potions", b.tr(BI_ASPEED), 10));
        }
    }

    // Its good to be the king
    // (C: `if (depth <= 127) return NULL;`, then all bases covered)
    None
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

    // borg-reborn-rs (#447): the optional plan applies before either calculation mode.
    prepared_with_limit(depth, crate::errands_plan::plan_cap(), || {
        borg_prepared_calculated(st, depth)
    })
}

/// Keep the guard ahead of calculation dispatch, without requiring an engine
/// link in the regression that verifies calculations cannot bypass it (#447).
pub(crate) fn prepared_with_limit(depth: i32, limit: Option<i32>, calculate: impl FnOnce() -> Option<Reason>) -> Option<Reason> {
    if let Some(limit) = limit.filter(|limit| depth >= *limit) {
        Some(Reason::new(ReasonMessage::Formatted(format!("No deeper {limit}.").into_bytes()), "plan_depth_limit", depth, limit, "<"))
    } else {
        calculate()
    }
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

        // Must meet minimal requirements
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
    if depth >= 82
        && (i32::from(home.num_ezheal) + i32::from(home.num_life) < 10
            && st.borg.tr(BI_AEZHEAL) + st.borg.tr(BI_ALIFE) < 10)
    {
        // Must know exact number of Potions  in home
        crate::borg_home_notice::borg_notice_home(st, None, false);

        let home = &st.home;
        return Some(Reason::new(ReasonMessage::Formatted(format!(
            "Scumming *Heal* potions ({} to go).", 10 - i32::from(home.num_ezheal)
        ).into_bytes()), "home_star_heal_or_life", i32::from(home.num_ezheal) + i32::from(home.num_life), 10, ">="));
    }

    // Scum on depth 80-81 for lots of *Heal* potions preparatory for Endgame
    if depth >= 82 && st.borg.tr(BI_MAXDEPTH) >= 97 {
        // Must know exact number of Potions  in home
        crate::borg_home_notice::borg_notice_home(st, None, false);

        // Scum for 30
        let home = &st.home;
        let heals = i32::from(home.num_ezheal_true)
            + st.borg.tr(BI_AEZHEAL)
            + i32::from(home.num_life_true)
            + st.borg.tr(BI_ALIFE);
        if heals < 30 {
            return Some(Reason::new(ReasonMessage::Formatted(format!(
                "Scumming *Heal* potions ({} to go).", 30 - heals
            ).into_bytes()), "total_star_heal_or_life", heals, 30, ">="));
        }

        // Return to town to get your stock from the home
        if st.borg.tr(BI_AEZHEAL) + st.borg.tr(BI_ALIFE) < 30
            && heals >= 30
            && home.num_ezheal_true >= 1
            && st.borg.tr(BI_MAXDEPTH) >= 99
        {
            return Some(Reason::new(ReasonMessage::Formatted(format!(
                "Collect from house ({} potions).",
                i32::from(home.num_ezheal_true) + i32::from(home.num_life_true)
            ).into_bytes()), "pack_star_heal_or_life", st.borg.tr(BI_AEZHEAL) + st.borg.tr(BI_ALIFE), 30, ">="));
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
    // We are now looking at our preparedness
    if -1 == st.borg.ready_morgoth {
        st.borg.ready_morgoth = 0;
    }

    if st.cfg(BORG_USES_DYNAMIC_CALCS) != 0 {
        let r: Option<*const CStr> = borg_restock_dynamic(st, depth).map(|r| r as *const CStr);
        return r.map(|r| Reason::new(ReasonMessage::Restock(line_index(&st.formulas.restock, r)), "dynamic_restock_formula", 0, 1, ">="));
    }

    restock_static(st, depth)
}

/// The part of `borg_restock` without dynamic formulas.
fn restock_static(st: &BorgState, depth: i32) -> Option<Reason> {
    let b = &st.borg;

    // *** Level 1 ***

    // Must have some lite
    if b.tr(BI_LIGHT) < 1 {
        return Some(Reason::fixed("restock light radius < 1", "light_radius", b.tr(BI_LIGHT), 1));
    }

    // Must have "fuel"
    if b.tr(BI_AFUEL) < 1 && b.tr(BI_LIGHT) == 0 {
        return Some(Reason::fixed("restock fuel", "fuel", b.tr(BI_AFUEL), 1));
    }

    // Assume happy at level 1
    if depth <= 1 {
        return None;
    }

    // *** Level 2 and 3 ***

    // Must have "fuel"
    if b.tr(BI_AFUEL) < 2 && b.tr(BI_LIGHT) == 0 {
        return Some(Reason::fixed("restock fuel < 2", "fuel", b.tr(BI_AFUEL), 2));
    }

    // Must have "food"
    if b.tr(BI_FOOD) < 3 {
        return Some(Reason::fixed("restock food < 3", "food", b.tr(BI_FOOD), 3));
    }

    // Must have "recall"
    // if (borg.trait[BI_RECALL] < 2) return ("restock recall");

    // Assume happy at level 3
    if depth <= 3 {
        return None;
    }

    // *** Level 3 to 5 ***

    if depth <= 5 {
        return None;
    }

    // *** Level 6 to 9 ***

    // Must have "phase"
    if b.tr(BI_APHASE) < 1 {
        return Some(Reason::fixed("restock phase door", "phase_door", b.tr(BI_APHASE), 1));
    }

    // Potions of Cure Wounds
    if b.tr(BI_MAXCLEVEL) < 30 && b.tr(BI_ACLW) + b.tr(BI_ACSW) + b.tr(BI_ACCW) < 1 {
        return Some(Reason::fixed("restock clw+csw+ccw", "cures", b.tr(BI_ACLW) + b.tr(BI_ACSW) + b.tr(BI_ACCW), 1));
    }

    // Assume happy at level 9
    if depth <= 9 {
        return None;
    }

    // *** Level 10 - 19  ***

    // Must have good light
    if b.tr(BI_LIGHT) < 2 && b.tr(BI_CLASS) != CLASS_NECROMANCER {
        return Some(Reason::fixed("2 light radius", "light_radius", b.tr(BI_LIGHT), 2));
    }

    // Must have "cure"
    if b.tr(BI_MAXCLEVEL) < 30 && b.tr(BI_ACLW) + b.tr(BI_ACSW) + b.tr(BI_ACCW) < 2 {
        return Some(Reason::fixed("restock clw + csw + ccw ", "cures", b.tr(BI_ACLW) + b.tr(BI_ACSW) + b.tr(BI_ACCW), 2));
    }

    // Must have "teleport"
    if b.tr(BI_ATELEPORT) + b.tr(BI_AESCAPE) < 2 {
        return Some(Reason::fixed("restock tele + tele staff < 2", "teleports", b.tr(BI_ATELEPORT) + b.tr(BI_AESCAPE), 2));
    }

    // Assume happy at level 19
    if depth <= 19 {
        return None;
    }

    // *** Level 20 - 35  ***

    // Must have "cure"
    if b.tr(BI_MAXCLEVEL) < 30 && b.tr(BI_ACSW) + b.tr(BI_ACCW) < 4 {
        return Some(Reason::fixed("restock csw + ccw < 4", "cure_serious_or_critical", b.tr(BI_ACSW) + b.tr(BI_ACCW), 4));
    }

    // Must have "teleport" or Staff
    if b.tr(BI_ATELEPORT) + b.tr(BI_AESCAPE) < 4 {
        return Some(Reason::fixed("restock 4 > teleport + teleport staff ", "teleports", b.tr(BI_ATELEPORT) + b.tr(BI_AESCAPE), 4));
    }

    // Assume happy at level 44
    if depth <= 35 {
        return None;
    }

    // *** Level 36 - 45  ***

    // Must have Scroll of Teleport (or good 2nd choice)
    if b.tr(BI_ATELEPORT) + b.tr(BI_ATELEPORTLVL) < 2 {
        return Some(Reason::fixed("restock teleport + teleport level scrolls", "teleport_or_teleport_level", b.tr(BI_ATELEPORT) + b.tr(BI_ATELEPORTLVL), 2));
    }

    // Assume happy at level 44
    if depth <= 45 {
        return None;
    }

    // *** Level 46 - 64  ***

    // Assume happy at level 65
    if depth <= 64 {
        return None;
    }

    // *** Level 65 - 99  ***

    // Must have "Heal"
    if b.tr(BI_AHEAL) + b.has[st.item_val.kv_rod_healing as usize] + b.tr(BI_AEZHEAL) < 1 {
        return Some(Reason::fixed("restock heal", "healing", b.tr(BI_AHEAL) + b.has[st.item_val.kv_rod_healing as usize] + b.tr(BI_AEZHEAL), 1));
    }

    // Assume happy at level 99
    if depth <= 99 {
        return None;
    }

    // *** Level 100  ***

    // Must have "Heal"
    // If I just got to dlevel 100 and low on heals, get out now.
    if borg_timer(st, b.time.level) < 10 && b.tr(BI_AEZHEAL) < 15 {
        return Some(Reason::fixed("restock *heal*", "star_heal", b.tr(BI_AEZHEAL), 15));
    }

    // Assume happy
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

    // Always spend time on a level unless 100
    if borg_timer(st, st.borg.time.level) < 100 && st.borg.tr(BI_CDEPTH) != 100 {
        return None;
    }

    // need to return to town if restock is needed
    borg_restock(st, st.borg.tr(BI_CDEPTH))
}

/// borg-reborn-rs (#557): a descent to `depth` needs a Word of Recall from
/// `recall_depth` on (C: 5; 0 = never).
fn needs_recall(depth: i32, recall_depth: i32, carried: i32) -> bool {
    recall_depth > 0 && depth >= recall_depth && carried < 1
}

#[cfg(test)]
mod recall_depth_tests {
    use super::needs_recall;

    #[test]
    fn shallow_gates_keep_first_failure_and_report_the_compared_operands() {
        use super::*;
        let mut st = BorgState::default();
        st.borg.trait_[BI_CDEPTH as usize] = 4;
        let fail = |st: &BorgState| shallow_requirements(st, 5).unwrap().requirement;
        assert_eq!(fail(&st), Requirement { name: "light_radius", actual: 0, required: 1, relation: ">=" });
        st.borg.trait_[BI_LIGHT as usize] = 1;
        assert_eq!(fail(&st), Requirement { name: "food", actual: 0, required: 5, relation: ">=" });
        assert_eq!(restock_static(&st, 5).unwrap().requirement,
            Requirement { name: "food", actual: 0, required: 3, relation: ">=" });
        st.borg.trait_[BI_FOOD as usize] = 5;
        assert_eq!(fail(&st), Requirement { name: "prepared_dl2_min_hp", actual: 0, required: 30, relation: ">=" });
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
        assert!(shallow_requirements(&st, 5).is_none());
    }

    #[test]
    fn a_descent_needs_a_recall_from_the_set_depth() {
        assert!(!needs_recall(4, 5, 0), "C: DL4 without one");
        assert!(needs_recall(5, 5, 0), "C: DL5 needs one");
        assert!(!needs_recall(5, 5, 1), "carried");
        assert!(needs_recall(2, 2, 0), "moved to DL2");
        assert!(!needs_recall(1, 2, 0));
        assert!(!needs_recall(30, 0, 0), "0 = never");
    }
}
