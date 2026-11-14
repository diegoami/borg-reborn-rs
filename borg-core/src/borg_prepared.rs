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
use crate::state::BorgState;
use crate::engine;

const CLASS_WARRIOR: i32 = 0;
const CLASS_MAGE: i32 = 1;
const CLASS_DRUID: i32 = 2;
const CLASS_PRIEST: i32 = 3;
const CLASS_NECROMANCER: i32 = 4;
const CLASS_PALADIN: i32 = 5;
const CLASS_ROGUE: i32 = 6;
const CLASS_RANGER: i32 = 7;
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
    Some(match r? {
        Reason::Formatted(s) => {
            write_buffer(st, &s);
            st.prepared.buffer.clone()
        }
        r => r.text(st).to_vec(),
    })
}

/// `borg_prepared(depth)` as C returns it (`None` for NULL), writing
/// `borg_prepared_buffer` as C does.
pub fn borg_prepared_c(st: &mut BorgState, depth: i32) -> Option<Vec<u8>> {
    let r = borg_prepared(st, depth);
    reason_c(st, r)
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
pub fn borg_prepared_buf(st: &mut BorgState, depth: i32) -> Option<PreparedC> {
    let r = borg_prepared(st, depth)?;
    Some(match r {
        Reason::Formatted(s) => {
            write_buffer(st, &s);
            PreparedC::Buffer
        }
        r => PreparedC::Text(r.text(st).to_vec()),
    })
}

/// The text `borg_prepared_buffer` holds now.
pub fn prepared_buffer(st: &BorgState) -> Vec<u8> {
    st.prepared.buffer.clone()
}

/// Why the Borg is not prepared (or must restock).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Reason {
    /// A fixed text (a string literal in C).
    Text(&'static str),
    /// A text C formats into its static `borg_prepared_buffer`
    /// (`MAX_REASON` = 1024 bytes).
    Formatted(Vec<u8>),
    /// The reason of `st.formulas.depth[i]` (`borg_prepared_dynamic()`).
    Depth(usize),
    /// The reason of `st.formulas.restock[i]` (`borg_restock_dynamic()`).
    Restock(usize),
}

impl Reason {
    /// The text of the reason, as C would print it.
    pub fn text<'a>(&'a self, st: &'a BorgState) -> &'a [u8] {
        match self {
            Reason::Text(s) => s.as_bytes(),
            Reason::Formatted(s) => s,
            Reason::Depth(i) => st.formulas.depth[*i].reason.to_bytes(),
            Reason::Restock(i) => st.formulas.restock[*i].reason.to_bytes(),
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

/// Determine if the Borg meets the "minimum" requirements for a level
/// (`borg_prepared_aux`).
fn borg_prepared_aux(st: &mut BorgState, depth: i32) -> Option<&'static str> {
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

    let b = &st.borg;
    let risky = st.cfg(BORG_PLAYS_RISKY) != 0;

    // *** Essential Items for Level 1 ***

    // Require lite (any)
    if b.tr(BI_LIGHT) < 1 {
        return Some("1 Lite");
    }

    // Require food
    if b.tr(BI_FOOD) < 5 {
        return Some("5 Food");
    }

    // Usually ready for level 1
    if depth <= 1 {
        return None;
    }

    // *** Essential Items for Level 2 ***

    // Require fuel
    if b.tr(BI_AFUEL) < 5 && b.tr(BI_LIGHT) == 0 {
        return Some("5 Fuel");
    }

    // Require recall
    // if (borg.trait[BI_RECALL] < 1) return ("1 recall");

    if !risky {
        // Require 30 hp
        if b.tr(BI_MAXHP) < 30 {
            return Some("30 hp");
        }
    }

    // Usually ready for level 2
    if depth <= 2 {
        return None;
    }

    // *** Essential Items for Level 3 and 4 ***

    if !risky {
        // class specific requirement
        match b.tr(BI_CLASS) {
            CLASS_WARRIOR | CLASS_BLACKGUARD => {
                if b.tr(BI_MAXHP) < 50 {
                    return Some("50 hp");
                }
                if b.tr(BI_MAXCLEVEL) < 4 {
                    return Some("4 clevel");
                }
            }
            CLASS_ROGUE => {
                if b.tr(BI_MAXHP) < 50 {
                    return Some("50 hp");
                }
                if b.tr(BI_MAXCLEVEL) < 8 {
                    return Some("8 clevel");
                }
            }
            CLASS_PRIEST | CLASS_DRUID => {
                if b.tr(BI_MAXHP) < 40 {
                    return Some("40 hp");
                }
                if b.tr(BI_MAXCLEVEL) < 9 {
                    return Some("9 level");
                }
            }
            CLASS_PALADIN => {
                if b.tr(BI_MAXHP) < 50 {
                    return Some("50 hp");
                }
                if b.tr(BI_MAXCLEVEL) < 4 {
                    return Some("4 clevel");
                }
            }
            CLASS_RANGER => {
                if b.tr(BI_MAXHP) < 50 {
                    return Some("50 hp");
                }
                if b.tr(BI_MAXCLEVEL) < 4 {
                    return Some("4 clevel");
                }
            }
            CLASS_MAGE | CLASS_NECROMANCER => {
                if b.tr(BI_MAXHP) < 60 {
                    return Some("60 hp");
                }
                if b.tr(BI_MAXCLEVEL) < 11 {
                    return Some("11 clevel");
                }
            }
            _ => {}
        }
    }

    // Potions of Cure Serious Wounds
    if b.tr(BI_MAXCLEVEL) < 30 && b.tr(BI_ACLW) + b.tr(BI_ACSW) + b.tr(BI_ACCW) < 2 {
        return Some("2 cure");
    }

    // Usually ready for level 3 and 4
    if depth <= 4 {
        return None;
    }

    // *** Essential Items for Level 5 to 9 ***

    if !risky {
        // class specific requirement
        if b.tr(BI_CDEPTH) != 0 {
            match b.tr(BI_CLASS) {
                CLASS_WARRIOR | CLASS_BLACKGUARD => {
                    if b.tr(BI_MAXHP) < 60 {
                        return Some("60 hp");
                    }
                    if b.tr(BI_MAXCLEVEL) < 6 {
                        return Some("6 clevel");
                    }
                }
                CLASS_ROGUE => {
                    if b.tr(BI_MAXHP) < 60 {
                        return Some("60 hp");
                    }
                    if b.tr(BI_MAXCLEVEL) < 10 {
                        return Some("10 clevel");
                    }
                }
                CLASS_PRIEST | CLASS_DRUID => {
                    if b.tr(BI_MAXHP) < 60 {
                        return Some("60 hp");
                    }
                    if b.tr(BI_MAXCLEVEL) < 15 {
                        return Some("15 clevel");
                    }
                }
                CLASS_PALADIN => {
                    if b.tr(BI_MAXHP) < 60 {
                        return Some("60 hp");
                    }
                    if b.tr(BI_MAXCLEVEL) < 6 {
                        return Some("6 clevel");
                    }
                }
                CLASS_RANGER => {
                    if b.tr(BI_MAXHP) < 60 {
                        return Some("60 hp");
                    }
                    if b.tr(BI_MAXCLEVEL) < 6 {
                        return Some("6 clevel");
                    }
                }
                CLASS_MAGE | CLASS_NECROMANCER => {
                    if b.tr(BI_MAXHP) < 80 {
                        return Some("80 hp");
                    }
                    if b.tr(BI_MAXCLEVEL) < 15 {
                        return Some("15 level");
                    }
                }
                _ => {}
            }
        }
    }

    // Potions of Cure Serious/Critical Wounds
    if b.tr(BI_MAXCLEVEL) < 30 && b.tr(BI_ACLW) + b.tr(BI_ACSW) + b.tr(BI_ACCW) < 2 {
        return Some("2 cures (clw + csw + ccw)");
    }

    // Scrolls of Word of Recall
    if b.tr(BI_RECALL) < 1 {
        return Some("1 recall");
    }

    // Usually ready for level 5 to 9
    if depth <= 9 {
        return None;
    }

    // *** Essential Items for Level 10 to 19 ***

    // Require light (radius 2), necromancers like the dark
    if b.tr(BI_LIGHT) < 2 && b.tr(BI_CLASS) != CLASS_NECROMANCER {
        return Some("2 light radius");
    }

    // Escape or Teleport
    if b.tr(BI_ATELEPORT) + b.tr(BI_AESCAPE) < 2 {
        return Some("2 tele + teleport staffs");
    }

    if !risky {
        // class specific requirement
        let clevel = b.tr(BI_MAXCLEVEL);
        match b.tr(BI_CLASS) {
            CLASS_WARRIOR | CLASS_BLACKGUARD if clevel < depth - 4 && depth <= 19 => {
                return Some("dlevel - 4 >= clevel");
            }
            CLASS_ROGUE if clevel < depth && depth <= 19 => {
                return Some("dlevel >= clevel");
            }
            CLASS_PRIEST | CLASS_DRUID if clevel < depth && depth <= 19 => {
                return Some("dlevel >= clevel");
            }
            CLASS_PALADIN if clevel < depth && depth <= 19 => {
                return Some("dlevel >= clevel");
            }
            CLASS_RANGER if clevel < depth && depth <= 19 => {
                return Some("dlevel >= clevel");
            }
            CLASS_MAGE | CLASS_NECROMANCER if clevel < depth + 5 && clevel <= 28 => {
                return Some("dlevel + 5 > = clevel");
            }
            _ => {}
        }
    }

    // Potions of Cure Critical Wounds
    if b.tr(BI_MAXCLEVEL) < 30 && b.tr(BI_ACCW) < 3 {
        return Some("ccw < 3");
    }

    // See invisible or telepathy
    if b.tr(BI_SINV) == 0 && b.tr(BI_DINV) == 0 && b.tr(BI_ESP) == 0 {
        return Some("See Invis : ESP");
    }

    // Usually ready for level 10 to 19
    if depth <= 19 {
        return None;
    }

    // *** Essential Items for Level 20 ***

    // Free action
    if b.tr(BI_FRACT) == 0 {
        return Some("free action");
    }

    // ready for level 20
    if depth <= 20 {
        return None;
    }

    // *** Essential Items for Level 25 ***

    // must have fire + 2 other basic resists
    if b.tr(BI_SRFIRE) == 0 {
        return Some("resist fire");
    }
    {
        let basics = b.tr(BI_RACID) + b.tr(BI_RCOLD) + b.tr(BI_RELEC);

        if basics < 2 {
            return Some("2 basic resists");
        }
    }
    // have some minimal stats
    if b.tr(BI_STR) < 7 {
        return Some("low STR");
    }

    let spell_stat = borg_spell_stat();
    // `borg.trait[BI_STR + spell_stat]`
    let spell_stat_trait = || b.trait_[(BI_STR as i32 + spell_stat) as usize];
    if spell_stat != -1 && spell_stat_trait() < 7 {
        return Some("low spell stat");
    }
    if b.tr(BI_DEX) < 7 {
        return Some("low DEX");
    }
    if b.tr(BI_CON) < 7 {
        return Some("low CON");
    }

    if !risky {
        // class specific requirement
        let clevel = b.tr(BI_MAXCLEVEL);
        match b.tr(BI_CLASS) {
            CLASS_WARRIOR | CLASS_BLACKGUARD => {
                if clevel < depth + 5 && clevel <= 38 {
                    return Some("dlevel + 5 >= clevel");
                }
            }
            CLASS_ROGUE => {
                if clevel < depth + 10 && clevel <= 43 {
                    return Some("dlevel + 10 >= clevel");
                }
            }
            CLASS_PRIEST | CLASS_DRUID => {
                if clevel < depth + 13 && clevel <= 46 {
                    return Some("dlevel + 13 >= clevel");
                }
            }
            CLASS_PALADIN => {
                if clevel < depth + 7 && clevel <= 40 {
                    return Some("dlevel + 7 >= clevel");
                }
            }
            CLASS_RANGER => {
                if clevel < depth + 8 && clevel <= 41 && clevel > 28 {
                    return Some("dlevel + 8 >= clevel");
                }
            }
            CLASS_MAGE | CLASS_NECROMANCER => {
                if clevel < depth + 8 && clevel <= 38 {
                    return Some("dlevel + 8 >= clevel");
                }
                if (clevel - 38) * 2 + 30 < depth && clevel <= 44 && clevel > 38 {
                    return Some("(clevel-38)*2+30 < dlevel");
                }
            }
            _ => {}
        }
    }

    // Ready for level 25
    if depth <= 25 {
        return None;
    }

    // *** Essential Items for Level 25 to 39 ***

    // All Basic resistance
    if b.tr(BI_SRCOLD) == 0 {
        return Some("resist cold");
    }
    if b.tr(BI_SRELEC) == 0 {
        return Some("resist elec");
    }
    if b.tr(BI_SRACID) == 0 {
        return Some("resist acid");
    }

    // Escape and Teleport
    if b.tr(BI_ATELEPORT) + b.tr(BI_AESCAPE) < 6 {
        return Some("6 tell + telep staffs");
    }

    // Cure Critical Wounds
    if b.tr(BI_MAXCLEVEL) < 30 && b.tr(BI_ACCW) + b.tr(BI_ACSW) < 10 {
        return Some("10 ccw + csw");
    }

    // Ready for level 33
    if depth <= 33 {
        return None;
    }

    // Minimal level
    if b.tr(BI_MAXCLEVEL) < 40 && !risky {
        return Some("level 40");
    }

    // Usually ready for level 20 to 39
    if depth <= 39 {
        return None;
    }

    // *** Essential Items for Level 40 to 45 ***

    // Resist
    if b.tr(BI_SRPOIS) == 0 {
        return Some("resist pois");
    }
    if b.tr(BI_SRCONF) == 0 {
        return Some("resist conf");
    }

    if b.tr(BI_STR) < 16 {
        return Some("STR < 16");
    }

    if spell_stat != -1 && spell_stat_trait() < 16 {
        return Some("spell stat < 16");
    }
    if b.tr(BI_DEX) < 16 {
        return Some("dex < 16");
    }
    if b.tr(BI_CON) < 16 {
        return Some("con < 16");
    }

    // Ok to continue
    if depth <= 45 {
        return None;
    }

    // *** Essential Items for Level 46 to 55 ***

    //  Must have +5 speed after level 46
    if b.tr(BI_SPEED) < 115 {
        return Some("+5 speed");
    }

    // Potions of heal
    if b.tr(BI_AHEAL) < 1 && b.tr(BI_AEZHEAL) < 1 {
        return Some("1 heal");
    }

    if !risky {
        // Minimal hitpoints
        if b.tr(BI_MAXHP) < 500 {
            return Some("HP 500");
        }
    }

    // High stats XXX XXX XXX
    if b.tr(BI_STR) < 18 + 40 {
        return Some("str < 18(40)");
    }

    if spell_stat != -1 && spell_stat_trait() < 18 + 100 {
        return Some("spell stat needs to be max");
    }
    if b.tr(BI_DEX) < 18 + 60 {
        return Some("dex < 18 (60)");
    }
    if b.tr(BI_CON) < 18 + 60 {
        return Some("con < 18 (60)");
    }

    // Hold Life
    if b.tr(BI_SHLIFE) == 0 && b.tr(BI_MAXCLEVEL) < 50 {
        return Some("hold life");
    }

    // Usually ready for level 46 to 55
    if depth <= 55 {
        return None;
    }

    // *** Essential Items for Level 55 to 59 ***

    // Potions of heal
    if b.tr(BI_AHEAL) < 2 && b.tr(BI_AEZHEAL) < 1 {
        return Some("2 heal + *heal*");
    }

    // Resists
    if b.tr(BI_SRBLIND) == 0 {
        return Some("resist blind");
    }

    // Must have resist nether
    // if (!borg_settings[BORG_PLAYS_RISKY] && !borg.trait[BI_SRNTHR]) return
    // ("resist nether");

    // Telepathy, better have it by now
    if b.tr(BI_ESP) == 0 {
        return Some("ESP");
    }

    // Usually ready for level 55 to 59
    if depth <= 59 {
        return None;
    }

    // *** Essential Items for Level 61 to 80 ***

    // Must have +10 speed
    if b.tr(BI_SPEED) < 120 {
        return Some("+10 speed");
    }

    // Resists
    if b.tr(BI_SRKAOS) == 0 {
        return Some("resist chaos");
    }
    if b.tr(BI_SRDIS) == 0 {
        return Some("resist disenchant");
    }

    // Usually ready for level 61 to 80
    if depth <= 80 {
        return None;
    }

    // *** Essential Items for Level 81-85 ***
    // Minimal Speed
    if b.tr(BI_SPEED) < 130 {
        return Some("+20 Speed");
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
            return Some("10 restore mana");
        }

        // must have lots of heal
        if b.has[iv.kv_potion_healing as usize] < 5 {
            return Some("5 Heal");
        }

        // must have lots of ez-heal
        if b.tr(BI_AEZHEAL) + b.tr(BI_ALIFE) < 15 {
            return Some("15 *heal* or life");
        }

        // must have lots of speed
        if b.tr(BI_ASPEED) < 10 {
            return Some("10 speed potions");
        }
    }

    // Its good to be the king
    // (C: `if (depth <= 127) return NULL;`, then all bases covered)
    None
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

    if st.cfg(BORG_USES_DYNAMIC_CALCS) != 0 {
        // use the base restock so special checks can be done
        if let Some(reason) = borg_restock(st, depth) {
            return Some(reason);
        }

        let r: Option<*const CStr> = borg_prepared_dynamic(st, depth).map(|r| r as *const CStr);
        if let Some(r) = r {
            return Some(Reason::Depth(line_index(&st.formulas.depth, r)));
        }
    } else {
        // Not prepared if I need to restock
        if let Some(reason) = borg_restock(st, depth) {
            return Some(reason);
        }

        // *** Require his Clevel to be greater than or equal to Depth
        if st.borg.tr(BI_MAXCLEVEL) < depth && st.borg.tr(BI_MAXCLEVEL) < 50 {
            return Some(Reason::Text("Clevel < depth"));
        }

        // Must meet minimal requirements
        if depth <= 99 {
            if let Some(reason) = borg_prepared_aux(st, depth) {
                return Some(Reason::Text(reason));
            }
        }

        // Not if No_Deeper is set
        if depth >= st.cfg(BORG_NO_DEEPER) {
            return Some(Reason::Formatted(format!("No deeper {}.", st.cfg(BORG_NO_DEEPER)).into_bytes()));
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
        return Some(Reason::Formatted(format!(
            "Scumming *Heal* potions ({} to go).",
            10 - i32::from(home.num_ezheal)
        ).into_bytes()));
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
            return Some(Reason::Formatted(format!(
                "Scumming *Heal* potions ({} to go).",
                30 - heals
            ).into_bytes()));
        }

        // Return to town to get your stock from the home
        if st.borg.tr(BI_AEZHEAL) + st.borg.tr(BI_ALIFE) < 30
            && heals >= 30
            && home.num_ezheal_true >= 1
            && st.borg.tr(BI_MAXDEPTH) >= 99
        {
            return Some(Reason::Formatted(format!(
                "Collect from house ({} potions).",
                i32::from(home.num_ezheal_true) + i32::from(home.num_life_true)
            ).into_bytes()));
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
        return Some(Reason::Formatted([b"Must kill ", name, b"."].concat()));
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
        return Some(Reason::Formatted([name, b" still alive!"].concat()));
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
        return r.map(|r| Reason::Restock(line_index(&st.formulas.restock, r)));
    }

    restock_static(st, depth).map(Reason::Text)
}

/// The part of `borg_restock` without dynamic formulas.
fn restock_static(st: &BorgState, depth: i32) -> Option<&'static str> {
    let b = &st.borg;

    // *** Level 1 ***

    // Must have some lite
    if b.tr(BI_LIGHT) < 1 {
        return Some("restock light radius < 1");
    }

    // Must have "fuel"
    if b.tr(BI_AFUEL) < 1 && b.tr(BI_LIGHT) == 0 {
        return Some("restock fuel");
    }

    // Assume happy at level 1
    if depth <= 1 {
        return None;
    }

    // *** Level 2 and 3 ***

    // Must have "fuel"
    if b.tr(BI_AFUEL) < 2 && b.tr(BI_LIGHT) == 0 {
        return Some("restock fuel < 2");
    }

    // Must have "food"
    if b.tr(BI_FOOD) < 3 {
        return Some("restock food < 3");
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
        return Some("restock phase door");
    }

    // Potions of Cure Wounds
    if b.tr(BI_MAXCLEVEL) < 30 && b.tr(BI_ACLW) + b.tr(BI_ACSW) + b.tr(BI_ACCW) < 1 {
        return Some("restock clw+csw+ccw");
    }

    // Assume happy at level 9
    if depth <= 9 {
        return None;
    }

    // *** Level 10 - 19  ***

    // Must have good light
    if b.tr(BI_LIGHT) < 2 && b.tr(BI_CLASS) != CLASS_NECROMANCER {
        return Some("2 light radius");
    }

    // Must have "cure"
    if b.tr(BI_MAXCLEVEL) < 30 && b.tr(BI_ACLW) + b.tr(BI_ACSW) + b.tr(BI_ACCW) < 2 {
        return Some("restock clw + csw + ccw ");
    }

    // Must have "teleport"
    if b.tr(BI_ATELEPORT) + b.tr(BI_AESCAPE) < 2 {
        return Some("restock tele + tele staff < 2");
    }

    // Assume happy at level 19
    if depth <= 19 {
        return None;
    }

    // *** Level 20 - 35  ***

    // Must have "cure"
    if b.tr(BI_MAXCLEVEL) < 30 && b.tr(BI_ACSW) + b.tr(BI_ACCW) < 4 {
        return Some("restock csw + ccw < 4");
    }

    // Must have "teleport" or Staff
    if b.tr(BI_ATELEPORT) + b.tr(BI_AESCAPE) < 4 {
        return Some("restock 4 > teleport + teleport staff ");
    }

    // Assume happy at level 44
    if depth <= 35 {
        return None;
    }

    // *** Level 36 - 45  ***

    // Must have Scroll of Teleport (or good 2nd choice)
    if b.tr(BI_ATELEPORT) + b.tr(BI_ATELEPORTLVL) < 2 {
        return Some("restock teleport + teleport level scrolls");
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
        return Some("restock heal");
    }

    // Assume happy at level 99
    if depth <= 99 {
        return None;
    }

    // *** Level 100  ***

    // Must have "Heal"
    // If I just got to dlevel 100 and low on heals, get out now.
    if borg_timer(st, b.time.level) < 10 && b.tr(BI_AEZHEAL) < 15 {
        return Some("restock *heal*");
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
