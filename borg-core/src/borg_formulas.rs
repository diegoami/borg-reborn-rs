//! Port of `borg-formulas.c`: the dynamic formulas (power, depth and
//! restock) read from the FORMULA SECTION of borg.txt.
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
//! The parsers work on byte buffers holding C strings: like the C code
//! they cut strings by writing NULs into their copy, and a "string" is the
//! bytes up to the first NUL (or the end of the buffer). That keeps the
//! texts in the error notes exactly as C prints them.

use std::ffi::{CStr, CString};

use crate::borg_formulas_calc::{
    calculate_dynamic_in, calculations_free, parse_calculation_line_in, CalcEnv,
};
use crate::borg_item_activation::borg_findact;
use crate::borg_item_val::borg_lookup_kind;
use crate::borg_util::borg_trim;
use crate::cctype::{c_isdigit, c_isspace};
use crate::borg_headers::borg_formulas_h::{
    value_type, VT_ACTIVATION, VT_CLASS, VT_CONFIG, VT_MAX, VT_NONE, VT_RANGE_INDEX, VT_TRAIT,
};
use crate::borg_headers::borg_h::BORG_USES_DYNAMIC_CALCS;
use crate::borg_headers::borg_trait_h::{
    BI_ADIGGER, BI_AENCH_ARM, BI_AENCH_TOD, BI_AENCH_TOH, BI_CARRY, BI_CLASS, BI_GOLD, BI_MAX,
    BI_MAXCLEVEL, BI_MAXDEPTH, BI_MAXSP, BI_NEED_ENCHANT_TO_A, BI_NEED_ENCHANT_TO_D,
    BI_NEED_ENCHANT_TO_H, BI_QUIVER_SLOTS, BI_SP_ADJ, BI_STR, BI_STR_INDEX, BI_WEIGHT,
};
use crate::headers::list_player_flags_h::PF_ZERO_FAIL;
use crate::headers::list_tvals_h::{TV_DIGGING, TV_POTION, TV_SCROLL, VT_NULL};
use crate::state::BorgState;
use crate::borg_item_analyze::borg_ego_has_random_power;
use crate::{borg_io, engine};

/// `struct value_sec`: a parsed `value(type, name)`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ValueSec {
    /// `type` (`enum value_type`)
    pub vtype: value_type,
    pub index: i32,
}

/// `struct math_section`: a number or the index of a calculation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MathSection {
    pub is_calculation: bool,
    pub calculation_index: i32,
}

/// `struct range_sec`
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RangeSec {
    pub from: i32,
    pub to: i32,
}

/// `struct borg_power_line`. `reward` is always there; `value` is there
/// whenever `range` is.
#[derive(Clone, Copy, Debug)]
pub struct PowerLine {
    pub range: Option<RangeSec>,
    pub reward: MathSection,
    pub value: Option<ValueSec>,
    pub condition: Option<MathSection>,
}

/// `struct borg_depth_line`
#[derive(Clone, Debug)]
pub struct DepthLine {
    pub dlevel: i32,
    /// The condition section as read, without its closing ')'
    /// (`condition(...`); returned by `borg_prepared_dynamic()`.
    pub reason: CString,
    pub condition: MathSection,
}

/// borg-formulas.c globals (`borg_formulas`).
#[derive(Default)]
pub struct FormulasState {
    /// power formulas
    pub power: Vec<PowerLine>,
    /// depth formulas
    pub depth: Vec<DepthLine>,
    /// depth formulas for restocking
    pub restock: Vec<DepthLine>,
}

/// `value_type_names[]`: entry `i` names the value type `i - 1`
/// (`VT_NONE` is -1), so `VT_GOLD` (40) is the last of the 42.
const VALUE_TYPE_NAMES: [&[u8]; 42] = [
    b"error",
    b"range",
    b"trait",
    b"config",
    b"activation",
    b"class",
    b"none",
    b"chest",
    b"shot",
    b"arrow",
    b"bolt",
    b"bow",
    b"digger",
    b"hafted",
    b"polearm",
    b"sword",
    b"boots",
    b"gloves",
    b"helm",
    b"crown",
    b"shield",
    b"cloak",
    b"soft armor",
    b"hard armor",
    b"dragon armor",
    b"light",
    b"amulet",
    b"ring",
    b"staff",
    b"wand",
    b"rod",
    b"scroll",
    b"potion",
    b"flask",
    b"food",
    b"mushroom",
    b"magic book",
    b"prayer book",
    b"nature book",
    b"shadow book",
    b"other book",
    b"gold",
];

/// The size of the line buffer in `borg_load_formulas()`.
const LINE_BUF: usize = 1024;

// ---------------------------------------------------------------------------
// C string helpers

/// `s[i]` on a C string: bytes past the end of the slice read as NUL.
fn at(s: &[u8], i: usize) -> u8 {
    s.get(i).copied().unwrap_or(0)
}

/// The C string starting at `buf[i]`: the bytes up to the first NUL.
fn cstr(buf: &[u8], i: usize) -> &[u8] {
    let s = buf.get(i..).unwrap_or(&[]);
    &s[..s.iter().position(|&c| c == 0).unwrap_or(s.len())]
}

/// `strchr(buf + from, c)` as an index into `buf`.
fn strchr(buf: &[u8], from: usize, c: u8) -> Option<usize> {
    cstr(buf, from)
        .iter()
        .position(|&b| b == c)
        .map(|i| from + i)
}

/// Skip `isspace()` bytes from `buf[p]`.
fn skip_space(buf: &[u8], mut p: usize) -> usize {
    while c_isspace(at(buf, p)) {
        p += 1;
    }
    p
}

/// The index of `line[strlen(line) - 1]` for the C string at `buf[p]`
/// (`p >= 1`): for an empty string it is the byte before it, as in C.
fn last_index(buf: &[u8], p: usize) -> usize {
    p + cstr(buf, p).len() - 1
}

/// `prefix_i(s, t)` (z-util.c): case-insensitive prefix test.
fn prefix_i(s: &[u8], t: &[u8]) -> bool {
    t.iter()
        .enumerate()
        .all(|(i, c)| c.eq_ignore_ascii_case(&at(s, i)))
}

/// `!my_stricmp(a, b)` (z-util.c): equal ignoring (ASCII) case.
fn stricmp_eq(a: &[u8], b: &[u8]) -> bool {
    a.eq_ignore_ascii_case(b)
}

/// `strtol(s, &end, 10)` (glibc): the value, saturated at the `long`
/// limits, and the number of bytes used (0 when there are no digits).
pub(crate) fn c_strtol(s: &[u8]) -> (i64, usize) {
    let mut p = skip_space(s, 0);
    let mut neg = false;
    if at(s, p) == b'-' || at(s, p) == b'+' {
        neg = at(s, p) == b'-';
        p += 1;
    }
    if !c_isdigit(at(s, p)) {
        return (0, 0);
    }
    let limit: u64 = if neg {
        i64::MAX as u64 + 1
    } else {
        i64::MAX as u64
    };
    let mut mag: u64 = 0;
    let mut overflow = false;
    while c_isdigit(at(s, p)) {
        let d = u64::from(at(s, p) - b'0');
        match mag.checked_mul(10).and_then(|m| m.checked_add(d)) {
            Some(m) if m <= limit => mag = m,
            _ => overflow = true,
        }
        p += 1;
    }
    let v = if overflow {
        if neg {
            i64::MIN
        } else {
            i64::MAX
        }
    } else if neg {
        (mag as i64).wrapping_neg()
    } else {
        mag as i64
    };
    (v, p)
}

/// `atol(s)` (glibc: `strtol(s, NULL, 10)`).
fn c_atol(s: &[u8]) -> i64 {
    c_strtol(s).0
}

// ---------------------------------------------------------------------------
// Environment

/// What this module needs from the engine and the rest of the Borg. The
/// real one is [`StEnv`]; unit tests use a fake (there is no game there).
pub(crate) trait FormulaEnv {
    /// The Borg state.
    fn st(&mut self) -> &mut BorgState;
    /// `borg_note(what)`
    fn note(&mut self, what: &[u8]);
    /// `borg_findact(name)`
    fn findact(&mut self, name: &[u8]) -> i32;
    /// `borg_get_class(name)`: walk the engine's class list.
    fn find_class(&mut self, name: &[u8]) -> i32;
    /// `lookup_sval(tval, name)`
    fn lookup_sval(&mut self, tval: i32, name: &[u8]) -> i32;
    /// `env.lookup_kind(tval, sval)`
    fn lookup_kind(&mut self, tval: i32, sval: i32) -> i32;
}

/// The real environment.
struct StEnv<'a>(&'a mut BorgState);

impl FormulaEnv for StEnv<'_> {
    fn st(&mut self) -> &mut BorgState {
        self.0
    }
    fn note(&mut self, what: &[u8]) {
        borg_io::borg_note_bytes(self.0, what)
    }
    fn findact(&mut self, name: &[u8]) -> i32 {
        // borg.txt is ASCII; a non-UTF-8 name cannot match an activation
        // name either way (only the failure note could differ).
        borg_findact(self.0, &String::from_utf8_lossy(name))
    }
    fn find_class(&mut self, name: &[u8]) -> i32 {
        engine::find_class(name)
    }
    fn lookup_sval(&mut self, tval: i32, name: &[u8]) -> i32 {
        engine::lookup_sval_bytes(tval, name)
    }
    fn lookup_kind(&mut self, tval: i32, sval: i32) -> i32 {
        borg_lookup_kind(self.0, tval, sval)
    }
}

/// The calculator's environment: `value()`s and errors come back here.
struct Calc<'a, E>(&'a mut E);

impl<E: FormulaEnv> CalcEnv for Calc<'_, E> {
    fn parse_value(&mut self, value: &[u8], full_line: &[u8]) -> Option<ValueSec> {
        parse_value_in(self.0, value, full_line)
    }
    fn calculate_from_value(&mut self, value: &ValueSec, range_index: i32) -> i32 {
        calculate_from_value(self.0.st(), value, range_index)
    }
    fn formula_error(&mut self, section: &[u8], full_line: &[u8], label: &str, error: &str) {
        formula_error(self.0, section, full_line, label, Some(error.as_bytes()))
    }
    fn note(&mut self, what: &str) {
        self.0.note(what.as_bytes())
    }
}

/// `parse_calculation_line(line, full_line)` (borg-formulas-calc.c).
fn parse_calculation<E: FormulaEnv>(env: &mut E, line: &[u8], full_line: &[u8]) -> i32 {
    // Moved out while parsing: value() parsing never touches them.
    let mut calculations = std::mem::take(&mut env.st().formulas_calc.calculations);
    let r = parse_calculation_line_in(&mut Calc(&mut *env), &mut calculations, line, full_line);
    env.st().formulas_calc.calculations = calculations;
    r
}

/// `borg_calculate_dynamic(formula, range_index)` (borg-formulas-calc.c).
fn calculate<E: FormulaEnv>(env: &mut E, formula: i32, range_index: i32) -> i32 {
    let calculations = std::mem::take(&mut env.st().formulas_calc.calculations);
    let r = calculate_dynamic_in(&mut Calc(&mut *env), &calculations, formula, range_index);
    env.st().formulas_calc.calculations = calculations;
    r
}

/// `borg_formula_error(section, full_line, section_label, error)`: give an
/// error for when a formula fails.
fn formula_error<E: FormulaEnv>(
    env: &mut E,
    section: &[u8],
    full_line: &[u8],
    section_label: &str,
    error: Option<&[u8]>,
) {
    env.note(b"** borg formula failure ** ");
    env.note(&[b"** error on line '", full_line, b"'"].concat());
    env.note(
        &[
            b"** error for section ",
            section_label.as_bytes(),
            b" '",
            section,
            b"'",
        ]
        .concat(),
    );
    if let Some(error) = error {
        env.note(error);
    }
    env.note(b"** formulas disabled ** ");
}

/// `borg_formula_error(section, full_line, section_label, format(...))`.
/// C's `format()` returns its static buffer, and borg_formula_error()
/// formats its own notes with `format()` before it notes `error`, so the
/// error note repeats the "error for section" note and the message built
/// by the caller is never shown (docs/c-bugs.md).
fn formula_error_format<E: FormulaEnv>(
    env: &mut E,
    section: &[u8],
    full_line: &[u8],
    section_label: &str,
) {
    let last_format = [
        b"** error for section ",
        section_label.as_bytes(),
        b" '",
        section,
        b"'",
    ]
    .concat();
    formula_error(env, section, full_line, section_label, Some(&last_format))
}

// ---------------------------------------------------------------------------
// Parsing

/// `parse_depth()`: the line looks like `depth(nnn)`; returns nnn, or -1
/// if it fails.
fn parse_depth(line: &[u8]) -> i32 {
    let Some(open) = strchr(line, 0, b'(') else {
        return -1;
    };

    // skip the "(" and leading whitespace
    let p = skip_space(line, open + 1);

    let (l, used) = c_strtol(cstr(line, p));
    if used == 0 {
        return -1;
    }

    // skip whitespace in case the string is "depth( nn )"
    let end = skip_space(line, p + used);

    // make sure it ends ")"
    if at(line, end) != b')' {
        return -1;
    }

    // long to int
    l as i32
}

/// `is_all_number()`: does the string hold only digits (or nothing)?
fn is_all_number(line: &[u8]) -> bool {
    cstr(line, 0).iter().all(|&c| c_isdigit(c))
}

/// `parse_formula_section()`: a number, or else a formula to parse.
fn parse_formula_section<E: FormulaEnv>(
    env: &mut E,
    line: &[u8],
    full_line: &[u8],
) -> Option<MathSection> {
    // skip leading whitespace
    let line = cstr(line, skip_space(line, 0));

    if is_all_number(line) {
        return Some(MathSection {
            is_calculation: false,
            calculation_index: c_atol(line) as i32,
        });
    }

    // parse the formula so it can be calculated from; it is stored in the
    // calculations and its index is returned
    let formula = parse_calculation(env, line, full_line);
    if formula == -1 {
        return None;
    }

    Some(MathSection {
        is_calculation: true,
        calculation_index: formula,
    })
}

/// `parse_condition()` / `parse_reward()`: the line looks like
/// `name({formula})`. The closing ')' is cut off `buf` (C changes its
/// copy in place, and a depth line keeps it as its reason).
fn parse_bracketed_formula<E: FormulaEnv>(
    env: &mut E,
    buf: &mut [u8],
    full_line: &[u8],
    label: &str,
    no_open_error: Option<&[u8]>,
) -> Option<MathSection> {
    let Some(open) = strchr(buf, 0, b'(') else {
        formula_error(env, cstr(buf, 0), full_line, label, no_open_error);
        return None;
    };

    // skip the "("
    let p = open + 1;

    // remove the end )
    let last = last_index(buf, p);
    if buf[last] != b')' {
        formula_error(
            env,
            cstr(buf, 0),
            full_line,
            label,
            Some(b"** no ending ')'"),
        );
        return None;
    }
    buf[last] = 0;

    let formula = cstr(buf, p).to_vec();
    parse_formula_section(env, &formula, full_line)
}

/// `parse_condition()`: the line looks like `condition({formula})`.
fn parse_condition<E: FormulaEnv>(
    env: &mut E,
    buf: &mut [u8],
    full_line: &[u8],
) -> Option<MathSection> {
    parse_bracketed_formula(env, buf, full_line, "condition", None)
}

/// `parse_reward()`: the line looks like `reward({formula})`.
fn parse_reward<E: FormulaEnv>(
    env: &mut E,
    buf: &mut [u8],
    full_line: &[u8],
) -> Option<MathSection> {
    parse_bracketed_formula(env, buf, full_line, "reward", Some(b"** no starting '('"))
}

/// `parse_range()`: the line looks like `range(n1, n2)` (for now n1 and n2
/// must be numbers).
fn parse_range<E: FormulaEnv>(env: &mut E, line: &[u8], full_line: &[u8]) -> Option<RangeSec> {
    let mut buf = line.to_vec();

    let Some(open) = strchr(&buf, 0, b'(') else {
        formula_error(
            env,
            cstr(&buf, 0),
            full_line,
            "range",
            Some(b"** no starting '('"),
        );
        return None;
    };

    // skip the "("
    let val1 = skip_space(&buf, open + 1);

    let Some(comma) = strchr(&buf, val1, b',') else {
        formula_error(
            env,
            cstr(&buf, 0),
            full_line,
            "range",
            Some(b"** range needs two numbers separated by a ','"),
        );
        return None;
    };
    // remove the ","
    buf[comma] = 0;
    let val2 = skip_space(&buf, comma + 1);

    // remove the end )
    let last = last_index(&buf, val2);
    if buf[last] != b')' {
        formula_error(
            env,
            cstr(&buf, 0),
            full_line,
            "range",
            Some(b"** first value is not a number"),
        );
        return None;
    }
    buf[last] = 0;

    if !is_all_number(cstr(&buf, val1)) {
        return None;
    }
    if !is_all_number(cstr(&buf, val2)) {
        formula_error(
            env,
            cstr(&buf, 0),
            full_line,
            "range",
            Some(b"** second value is not a number"),
        );
        return None;
    }

    let from = c_atol(cstr(&buf, val1)) as i32;
    let to = c_atol(cstr(&buf, val2)) as i32;
    let error = if from < 1 {
        Some(&b"** first value less than one"[..])
    } else if to < 0 {
        Some(&b"** second value less than one"[..])
    } else if to < from {
        Some(&b"** second value less first value"[..])
    } else {
        None
    };
    if let Some(error) = error {
        formula_error(env, cstr(&buf, 0), full_line, "range", Some(error));
        return None;
    }

    Some(RangeSec { from, to })
}

/// `get_value_type()`: the value type with this name, or `VT_NONE`.
fn get_value_type(value_type_name: &[u8]) -> value_type {
    VALUE_TYPE_NAMES
        .iter()
        .position(|n| stricmp_eq(value_type_name, n))
        .map_or(VT_NONE, |i| i as value_type - 1)
}

/// `get_value_index()`: turn the name in `value(type, name)` into an index
/// that `calculate_from_value()` can use; -1 on failure.
fn get_value_index<E: FormulaEnv>(
    env: &mut E,
    t: value_type,
    name: &[u8],
    full_line: &[u8],
) -> i32 {
    match t {
        VT_ACTIVATION => {
            // borg.txt is ASCII; a non-UTF-8 name cannot match an
            // activation name either way.
            let i = env.findact(name);
            if i == -1 {
                formula_error(
                    env,
                    name,
                    full_line,
                    "value",
                    Some(b"** unable to find activation"),
                );
            }
            i
        }
        VT_TRAIT => {
            for i in 0..BI_MAX as usize {
                if stricmp_eq(crate::borg_trait::trait_name(i), name) {
                    return i as i32;
                }
            }
            formula_error(
                env,
                name,
                full_line,
                "value",
                Some(b"** unable to find trait"),
            );
            -1
        }
        VT_CONFIG => {
            // (the table's NULL entry ends the C loop)
            for (i, setting) in crate::borg_init::BORG_SETTINGS.iter().enumerate() {
                // +5 to get rid of borg_
                if stricmp_eq(&setting.name.to_bytes()[5..], name) {
                    return i as i32;
                }
            }
            formula_error(
                env,
                name,
                full_line,
                "value",
                Some(b"** unable to find config"),
            );
            -1
        }
        VT_RANGE_INDEX => 1,
        VT_CLASS => {
            let i = env.find_class(name);
            if i == -1 {
                formula_error(env, name, full_line, "value", Some(b"** invalid class"));
            }
            i
        }
        VT_NONE | VT_NULL | VT_MAX => {
            formula_error(
                env,
                name,
                full_line,
                "value",
                Some(b"** invalid value type"),
            );
            -1
        }
        // lookup in the borg.has array
        _ => {
            let tval = t - VT_NULL;
            // C's error text, "** value error for object type '%s' subtype
            // '%s'" (with `value_type_names[t].name`, the name of type
            // t - 1), is lost to format()'s buffer.
            let sval = env.lookup_sval(tval, name);
            if sval == -1 {
                formula_error_format(env, name, full_line, "value");
                return -1;
            }
            let i = env.lookup_kind(tval, sval);
            if i == -1 {
                formula_error_format(env, name, full_line, "value");
            }
            i
        }
    }
}

/// `calculate_from_value()`: turn a value into a number that can be used
/// in a formula.
pub fn calculate_from_value(st: &BorgState, value: &ValueSec, range_index: i32) -> i32 {
    let index = value.index as usize;
    match value.vtype {
        VT_ACTIVATION => st.borg.activation[index],
        VT_TRAIT => st.borg.trait_[index],
        VT_CONFIG => st.core.cfg[index],
        VT_CLASS => (st.borg.tr(BI_CLASS) == value.index) as i32,
        VT_RANGE_INDEX => range_index,
        _ => st.borg.has[index],
    }
}

/// `parse_value()`: the line looks like `value(v1, v2)`. The possible
/// values for v1 are the value types; the possible values for v2 depend
/// on v1. `None` if it fails.
fn parse_value_in<E: FormulaEnv>(env: &mut E, line: &[u8], full_line: &[u8]) -> Option<ValueSec> {
    let mut buf = line.to_vec();

    let Some(open) = strchr(&buf, 0, b'(') else {
        formula_error(env, cstr(&buf, 0), full_line, "value", None);
        return None;
    };

    // skip the "(" and leading space
    let val1 = skip_space(&buf, open + 1);

    let Some(comma) = strchr(&buf, val1, b',') else {
        formula_error(
            env,
            cstr(&buf, 0),
            full_line,
            "value",
            Some(b"** value needs two values separated by a ',' "),
        );
        return None;
    };
    // remove the "," and leading space
    buf[comma] = 0;
    let val2 = skip_space(&buf, comma + 1);

    // remove the end )
    let last = last_index(&buf, val2);
    if buf[last] != b')' {
        formula_error(
            env,
            cstr(&buf, 0),
            full_line,
            "value",
            Some(b"** no ending ')' "),
        );
        return None;
    }
    buf[last] = 0;

    let t = get_value_type(cstr(&buf, val1));
    if t == VT_NONE {
        // C's "** value type (first parameter '%s') not found" is lost to
        // format()'s buffer.
        formula_error_format(env, cstr(&buf, 0), full_line, "value");
        return None;
    }

    let index = get_value_index(env, t, cstr(&buf, val2), full_line);
    if index == -1 {
        return None;
    }

    Some(ValueSec { vtype: t, index })
}

/// `get_section()`: dynamic formula lines are `name(xxx):name(xxx);`. If
/// `line` starts with `name`, store that section (up to the next ':', or
/// the end without a final ';') in `value`.
fn get_section<E: FormulaEnv>(
    env: &mut E,
    line: &[u8],
    name: &[u8],
    value: &mut Option<Vec<u8>>,
    full_line: &[u8],
) -> bool {
    if prefix_i(line, name) {
        // this value already exists
        if value.is_some() {
            formula_error(env, line, full_line, "value", Some(b"** section repeated"));
            return false;
        }

        let mut len = strchr(line, 0, b':').unwrap_or(line.len());
        if line[len - 1] == b';' {
            len -= 1;
        }
        *value = Some(line[..len].to_vec());
        return true;
    }

    false
}

/// Look for each named section at `line[*p..]` in turn, moving `*p` past
/// each one found (the scan loops of `parse_depth_line()` and
/// `parse_power_line()`). Returns false (after an error) when the text
/// there is not a section separator.
fn scan_sections<E: FormulaEnv>(
    env: &mut E,
    line: &[u8],
    p: &mut usize,
    sections: &mut [(&[u8], &mut Option<Vec<u8>>)],
    full_line: &[u8],
    label: &str,
) -> bool {
    for (name, value) in sections.iter_mut() {
        let rest = line.get(*p..).unwrap_or(&[]);
        if get_section(env, rest, name, value, full_line) {
            *p += value.as_ref().map_or(0, Vec::len);
        }
    }

    let c = at(line, *p);
    if c != 0 && !c_isspace(c) && c != b':' && c != b';' {
        let err = [
            &b"** unparsed information '"[..],
            line.get(*p..).unwrap_or(&[]),
            b"' in ",
            label.as_bytes(),
            b" line",
        ]
        .concat();
        formula_error(env, line, full_line, label, Some(&err));
        return false;
    }
    true
}

/// `parse_depth_line()`: depth lines appear as `depth(nnn):condition(xxx)`.
/// Returns true on failure.
fn parse_depth_line<E: FormulaEnv>(
    env: &mut E,
    restock: bool,
    line: &[u8],
    full_line: &[u8],
) -> bool {
    let start = line;
    let mut fail = false;
    let mut s_depth: Option<Vec<u8>> = None;
    let mut s_condition: Option<Vec<u8>> = None;

    let mut p = 0;
    while at(line, p) != 0 {
        let mut sections: [(&[u8], &mut Option<Vec<u8>>); 2] =
            [(b"depth", &mut s_depth), (b"condition", &mut s_condition)];
        if !scan_sections(env, line, &mut p, &mut sections, full_line, "depth") {
            fail = true;
            break;
        }
        p += 1;
    }

    if !fail && s_depth.is_none() {
        formula_error(
            env,
            start,
            full_line,
            "value",
            Some(b"** depth section missing"),
        );
        fail = true;
    }
    if !fail && s_condition.is_none() {
        formula_error(
            env,
            start,
            full_line,
            "value",
            Some(b"** condition section missing"),
        );
        fail = true;
    }

    let mut dlevel = 0;
    if let Some(s_depth) = s_depth {
        if !fail {
            dlevel = parse_depth(&s_depth);
            if dlevel < 0 {
                formula_error(
                    env,
                    start,
                    full_line,
                    "value",
                    Some(b"** depth section can't be parsed"),
                );
                fail = true;
            }
        }
    }

    let mut line_out = None;
    if let Some(mut s_condition) = s_condition {
        if !fail {
            let condition = parse_condition(env, &mut s_condition, full_line);
            match condition {
                Some(condition) => {
                    let reason = CString::new(cstr(&s_condition, 0)).expect("no NUL");
                    line_out = Some(DepthLine {
                        dlevel,
                        reason,
                        condition,
                    });
                }
                None => fail = true,
            }
        }
    }

    // add to the depth array
    if let (false, Some(d)) = (fail, line_out) {
        if restock {
            env.st().formulas.restock.push(d);
        } else {
            env.st().formulas.depth.push(d);
        }
    }

    fail
}

/// `parse_power_line()`: power lines appear as
/// `value(a, b):condition(xxx):range(n1, n2):reward(xxx)`; value and range
/// are optional. Returns true on failure.
fn parse_power_line<E: FormulaEnv>(env: &mut E, line: &[u8], full_line: &[u8]) -> bool {
    let start = line;
    let mut fail = false;
    let mut s_range: Option<Vec<u8>> = None;
    let mut s_reward: Option<Vec<u8>> = None;
    let mut s_value: Option<Vec<u8>> = None;
    let mut s_condition: Option<Vec<u8>> = None;

    let mut p = 0;
    while at(line, p) != 0 {
        let mut sections: [(&[u8], &mut Option<Vec<u8>>); 4] = [
            (b"range", &mut s_range),
            (b"reward", &mut s_reward),
            (b"value", &mut s_value),
            (b"condition", &mut s_condition),
        ];
        if !scan_sections(env, line, &mut p, &mut sections, full_line, "power") {
            fail = true;
            break;
        }
        p += 1;
    }

    if !fail && s_range.is_some() && s_value.is_none() {
        formula_error(
            env,
            start,
            full_line,
            "power",
            Some(b"** if range() is specified value() must also be specified"),
        );
        fail = true;
    }

    if !fail && s_reward.is_none() {
        formula_error(
            env,
            start,
            full_line,
            "power",
            Some(b"** reward() must be specified specified"),
        );
        fail = true;
    }

    let mut range = None;
    if let Some(s_range) = s_range {
        if !fail {
            range = parse_range(env, &s_range, full_line);
            if range.is_none() {
                fail = true;
            }
        }
    }

    let mut reward = None;
    if let Some(mut s_reward) = s_reward {
        if !fail {
            reward = parse_reward(env, &mut s_reward, full_line);
            if reward.is_none() {
                fail = true;
            }
        }
    }

    let mut value = None;
    if let Some(s_value) = s_value {
        if !fail {
            value = parse_value_in(env, &s_value, full_line);
            if value.is_none() {
                fail = true;
            }
        }
    }

    let mut condition = None;
    if let Some(mut s_condition) = s_condition {
        if !fail {
            condition = parse_condition(env, &mut s_condition, full_line);
            if condition.is_none() {
                fail = true;
            }
        }
    }

    if let (false, Some(reward)) = (fail, reward) {
        env.st().formulas.power.push(PowerLine {
            range,
            reward,
            value,
            condition,
        });
    }

    fail
}

// ---------------------------------------------------------------------------
// Evaluation

/// `check_condition()`
fn check_condition<E: FormulaEnv>(env: &mut E, condition: MathSection, range: i32) -> bool {
    // not sure why you would ever do this
    if !condition.is_calculation {
        return condition.calculation_index != 0;
    }

    calculate(env, condition.calculation_index, range) != 0
}

/// One reward: the number, or the formula's value at `range_index`.
fn reward_value<E: FormulaEnv>(env: &mut E, reward: MathSection, range_index: i32) -> i32 {
    if reward.is_calculation {
        calculate(env, reward.calculation_index, range_index)
    } else {
        reward.calculation_index
    }
}

/// `calc_power()`: get a single value from a power line.
fn calc_power<E: FormulaEnv>(env: &mut E, p: &PowerLine) -> i32 {
    let mut total: i32 = 0;
    if let Some(condition) = p.condition {
        if !check_condition(env, condition, 0) {
            return 0;
        }
    }

    if let Some(range) = p.range {
        // loop from the "from" to the count or the "to", whichever is lower
        let value = p.value.expect("range() comes with value()");
        let end = calculate_from_value(env.st(), &value, 0).min(range.to);
        let mut i = range.from;
        while i <= end {
            total = total.wrapping_add(reward_value(env, p.reward, i - 1));
            i += 1;
        }
        return total;
    }

    total.wrapping_add(reward_value(env, p.reward, 0))
}

/// Is this (the last item in the pack) one that will be used right away,
/// so it should not count towards encumbrance?
fn used_immediately(st: &BorgState, item: &crate::borg_headers::borg_item_h::borg_item) -> bool {
    let iv = &st.item_val;
    let sval = i32::from(item.sval);
    let tval = u32::from(item.tval);
    item.iqty != 0
        && item.aware
        && ((tval == TV_SCROLL
            && ((sval == iv.sv_scroll_enchant_armor
                && st.borg.tr(BI_AENCH_ARM) < 1000
                && st.borg.tr(BI_NEED_ENCHANT_TO_A) != 0)
                || (sval == iv.sv_scroll_enchant_weapon_to_hit
                    && st.borg.tr(BI_AENCH_TOH) < 1000
                    && st.borg.tr(BI_NEED_ENCHANT_TO_H) != 0)
                || (sval == iv.sv_scroll_enchant_weapon_to_dam
                    && st.borg.tr(BI_AENCH_TOD) < 1000
                    && st.borg.tr(BI_NEED_ENCHANT_TO_D) != 0)
                || sval == iv.sv_scroll_star_enchant_weapon
                || sval == iv.sv_scroll_star_enchant_armor))
            || (tval == TV_POTION
                && (sval == iv.sv_potion_inc_str
                    || sval == iv.sv_potion_inc_int
                    || sval == iv.sv_potion_inc_wis
                    || sval == iv.sv_potion_inc_dex
                    || sval == iv.sv_potion_inc_con
                    || sval == iv.sv_potion_inc_all)))
}

/// `total - (n * k)` computed in `long` and stored back in an `int32_t`.
/// `borg_get_spell_entry(book, entry)` (borg-magic.c): the spell's
/// `(level, power)`, or `None` for NULL.
fn spell_entry(st: &mut BorgState, book: i32, entry: i32) -> Option<(u8, u8)> {
    crate::borg_magic::borg_get_spell_entry(st, book, entry).map(|i| {
        let m = &st.magic.magics[i];
        (m.level, m.power)
    })
}

fn sub_long(total: i32, n: i32, k: i64) -> i32 {
    (i64::from(total) - i64::from(n) * k) as i32
}

/// `borg_power_dynamic()`: calculate the basic "power".
pub fn borg_power_dynamic(st: &mut BorgState) -> i32 {
    let mut total: i32 = 0;

    for i in 0..st.formulas.power.len() {
        let p = st.formulas.power[i];
        total = total.wrapping_add(calc_power(&mut StEnv(st), &p));
    }

    // HUGE HACK need some extra stuff that is hard to make dynamic
    if crate::borg_magic::borg_spell_stat() > 0 {
        let chance = i32::from(engine::spell_chance(0));
        total = total.wrapping_add((100 - chance).wrapping_mul(100));
    }

    // should try to get min fail to 0
    if engine::player_has(PF_ZERO_FAIL) {
        // other fail rates
        if i32::from(engine::spell_chance(0)) < 1 {
            total = total.wrapping_add(30000);
        }
    }

    let pack_size = i32::from(engine::z_info().pack_size);
    let inven_wield = pack_size;
    let inven_body = pack_size + 6;
    let inven_outer = pack_size + 7;
    let inven_arm = pack_size + 8;
    let inven_head = pack_size + 9;
    let inven_hands = pack_size + 10;
    let inven_feet = pack_size + 11;

    // Hack-- Reward the borg for carrying a NON-ID items that have random
    // powers
    let outer = st.items.items[inven_outer as usize];
    if outer.iqty != 0 && borg_ego_has_random_power(usize::from(outer.ego_idx)) && !outer.ident {
        total = total.wrapping_add(999999);
    }

    // Penalize armor weight
    if st.borg.tr(BI_STR_INDEX) < 15 {
        let body = i32::from(crate::borg_item::borg_item_weight(&st.items.items[inven_body as usize]));
        if body > 200 {
            total = total.wrapping_sub((body - 200) * 15);
        }
        if i32::from(crate::borg_item::borg_item_weight(&st.items.items[inven_head as usize])) > 30 {
            total = total.wrapping_sub(250);
        }
        if i32::from(crate::borg_item::borg_item_weight(&st.items.items[inven_arm as usize])) > 10 {
            total = total.wrapping_sub(250);
        }
        if i32::from(crate::borg_item::borg_item_weight(&st.items.items[inven_feet as usize])) > 50 {
            total = total.wrapping_sub(250);
        }
    }

    // Compute the total armor weight
    let mut cur_wgt: i32 = 0;
    for slot in [
        inven_body,
        inven_head,
        inven_arm,
        inven_outer,
        inven_hands,
        inven_feet,
    ] {
        cur_wgt += i32::from(crate::borg_item::borg_item_weight(&st.items.items[slot as usize]));
    }

    // Determine the weight allowance
    let max_wgt = engine::class_spell_weight();

    // HACK: Heavy armor hurts magic
    if crate::borg_magic::borg_can_cast() && (cur_wgt - max_wgt) / 10 > 0 {
        let lost = (cur_wgt - max_wgt) / 10;
        // max sp must be calculated in case it changed with the armor
        let mut max_sp = st.borg.tr(BI_SP_ADJ) / 100 + 1;
        max_sp -= lost;
        // Mega-Hack -- Penalize heavy armor which hurts mana
        if (300..=350).contains(&max_sp) {
            total = sub_long(total, lost, 400);
        }
        if (200..=299).contains(&max_sp) {
            total = sub_long(total, lost, 800);
        }
        if (100..=199).contains(&max_sp) {
            total = sub_long(total, lost, 1600);
        }
        if max_sp <= 99 {
            total = sub_long(total, lost, 3200);
        }
    }

    // SECOND MAJOR HACK for inventory

    // Reward carrying a shovel if low level
    if st.borg.tr(BI_MAXDEPTH) <= 40
        && st.borg.tr(BI_MAXDEPTH) >= 25
        && st.borg.tr(BI_GOLD) < 100000
        && u32::from(st.items.items[inven_wield as usize].tval) != TV_DIGGING
        && st.borg.tr(BI_ADIGGER) == 1
    {
        total = total.wrapping_add(5000);
    }

    // HACK: Books: reward books
    for book in 0..9i32 {
        let amt = i32::from(st.borg.amt_book[book as usize]);
        // No copies
        if amt == 0 {
            continue;
        }

        if engine::class_book_dungeon(book) {
            // The "hard" books: scan the spells
            for what in 0..9 {
                let Some((level, power)) = spell_entry(st, book, what) else {
                    break;
                };

                // Track minimum level
                if i32::from(level) > st.borg.tr(BI_MAXCLEVEL) {
                    continue;
                }

                // Track Mana req.
                if i32::from(power) > st.borg.tr(BI_MAXSP) {
                    continue;
                }

                // Reward the book based on the spells I can cast
                total = total.wrapping_add(15000);
            }
        } else {
            // The "easy" books: scan the spells
            let mut when: i32 = 99;
            for what in 0..9 {
                let Some((level, _power)) = spell_entry(st, book, what) else {
                    break;
                };

                // Track minimum level
                if i32::from(level) < when {
                    when = i32::from(level);
                }
            }

            // HACK: Ignore "difficult" normal books
            if when > 5 && when >= st.borg.tr(BI_MAXCLEVEL) + 2 {
                continue;
            }

            // Reward the book
            let mut k = 0;
            while k < 1 && k < amt {
                total = total.wrapping_add(500000);
                k += 1;
            }
            if st.borg.tr(BI_STR) > 5 {
                while k < 2 && k < amt {
                    total = total.wrapping_add(10000);
                    k += 1;
                }
            }
        }
    }

    // HACK: Apply "encumbrance" from weight
    if st.borg.tr(BI_WEIGHT) > st.borg.tr(BI_CARRY) / 2 {
        // *HACK* when testing items, the borg puts them in the last empty
        // slot so this is POSSIBLY just a test item
        let mut item = None;
        let mut i = pack_size - st.borg.tr(BI_QUIVER_SLOTS);
        while i >= 0 {
            let it = st.items.items[i as usize];
            if it.iqty != 0 {
                item = Some(it);
                break;
            }
            i -= 1;
        }

        // Some items will be used immediately and should not contribute to
        // encumbrance
        if !item.is_some_and(|it| used_immediately(st, &it)) {
            let carry = st.borg.tr(BI_CARRY);
            let over = (st.borg.tr(BI_WEIGHT) - carry / 2) / (carry / 10);
            total = sub_long(total, over, 1000);
        }
    }
    // END SECOND MAJOR HACK

    total
}

/// The depth (or restock) lines.
fn depth_lines(st: &BorgState, restock: bool) -> &[DepthLine] {
    if restock {
        &st.formulas.restock
    } else {
        &st.formulas.depth
    }
}

/// The index of the first depth (or restock) line at or above `depth`
/// whose condition holds.
fn dynamic_reason<E: FormulaEnv>(env: &mut E, restock: bool, depth: i32) -> Option<usize> {
    let mut i = 0;
    while let Some(d) = depth_lines(env.st(), restock).get(i) {
        let (dlevel, condition) = (d.dlevel, d.condition);
        if dlevel > depth {
            break;
        }
        if check_condition(env, condition, depth) {
            return Some(i);
        }
        i += 1;
    }
    None
}

/// `borg_prepared_dynamic()`: determine what level the borg is prepared to
/// dive to. Returns the reason it is not prepared for `depth`.
pub fn borg_prepared_dynamic(st: &mut BorgState, depth: i32) -> Option<&CStr> {
    let i = dynamic_reason(&mut StEnv(st), false, depth)?;
    Some(st.formulas.depth[i].reason.as_c_str())
}

/// `borg_restock_dynamic()`: determine if the borg needs to restock.
pub fn borg_restock_dynamic(st: &mut BorgState, depth: i32) -> Option<&CStr> {
    let i = dynamic_reason(&mut StEnv(st), true, depth)?;
    Some(st.formulas.restock[i].reason.as_c_str())
}

// ---------------------------------------------------------------------------
// Loading

/// Sections of the formula part of borg.txt (`enum e_section`).
#[derive(Clone, Copy, PartialEq, Eq)]
enum Section {
    None,
    Power,
    Depth,
    Restock,
}

/// `borg_load_formulas(fp)` reading the lines with `getl(buf, len)`
/// (`file_getl(fp, buf, len)`): borg-init.c passes its open borg.txt.
pub fn borg_load_formulas_from(
    st: &mut BorgState,
    mut getl: impl FnMut(&mut [u8], usize) -> bool,
) -> bool {
    load_formulas(&mut StEnv(st), |buf| getl(buf, LINE_BUF - 1))
}

/// `borg_load_formulas()` reading lines with `getl` (`file_getl()` into a
/// `LINE_BUF` buffer).
fn load_formulas<E: FormulaEnv>(env: &mut E, mut getl: impl FnMut(&mut [u8]) -> bool) -> bool {
    let mut buf = [0u8; LINE_BUF];
    let mut formulas_off = false;
    let mut section = Section::None;
    let mut skip_open = true;

    env.st().formulas.restock.clear();
    env.st().formulas.depth.clear();
    env.st().formulas.power.clear();

    // read but don't process formulas if we aren't using them
    if env.st().cfg(BORG_USES_DYNAMIC_CALCS) == 0 {
        formulas_off = true;
    }

    // Parse the formula section of the file
    while getl(&mut buf) {
        let off = borg_trim(&mut buf);
        let line = cstr(&buf, off).to_vec();
        let full = cstr(&buf, 0).to_vec();

        // Skip comments and blank lines
        if line.is_empty() || line[0] == b'#' {
            continue;
        }

        if prefix_i(&line, b"[END FORMULA SECTION]") {
            break;
        }

        // if we failed anywhere
        if formulas_off {
            continue;
        }

        // sections are bounded in []
        if line[0] == b'[' {
            if !skip_open {
                formulas_off = true;
            }

            skip_open = false;
            continue;
        }
        if line[0] == b']' {
            if section == Section::None {
                return formulas_off;
            }
            section = Section::None;
            continue;
        }

        if prefix_i(&line, b"Power") {
            if section != Section::None {
                formulas_off = true;
            }

            section = Section::Power;
            skip_open = true;
            continue;
        }
        if prefix_i(&line, b"Depth Requirement") {
            if section != Section::None {
                formulas_off = true;
            }

            section = Section::Depth;
            skip_open = true;
            continue;
        }
        if prefix_i(&line, b"Restock Requirement") {
            if section != Section::None {
                formulas_off = true;
            }

            section = Section::Restock;
            skip_open = true;
            continue;
        }

        // lines must end in ;
        if line[line.len() - 1] != b';' {
            let label = match section {
                Section::Depth => "depth",
                Section::Power => "power",
                Section::Restock => "restock",
                Section::None => "base",
            };
            formula_error(env, &full, &full, label, Some(b"** line must end in a ;"));
            formulas_off = true;
        } else {
            match section {
                Section::Depth => formulas_off = parse_depth_line(env, false, &line, &full),
                Section::Restock => formulas_off = parse_depth_line(env, true, &line, &full),
                Section::Power => formulas_off = parse_power_line(env, &line, &full),
                Section::None => {
                    formula_error(
                        env,
                        &full,
                        &full,
                        "base",
                        Some(b"** line must be part of a Power or Depth section"),
                    );
                    formulas_off = true;
                }
            }
        }
    }
    if formulas_off && env.st().cfg(BORG_USES_DYNAMIC_CALCS) != 0 {
        env.st().core.cfg[BORG_USES_DYNAMIC_CALCS as usize] = 0;
    }

    formulas_off
}

/// `borg_free_formulas()`: free all memory used in dynamic calculations.
pub fn borg_free_formulas(st: &mut BorgState) {
    st.formulas.restock.clear();
    st.formulas.depth.clear();
    st.formulas.power.clear();

    calculations_free(st);
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    /// A fake environment: notes are collected, and the engine lookups
    /// know a few names (the same stubs as the C differential test).
    pub(crate) struct TestEnv {
        pub st: BorgState,
        pub notes: Vec<String>,
    }

    impl FormulaEnv for TestEnv {
        fn st(&mut self) -> &mut BorgState {
            &mut self.st
        }
        fn note(&mut self, what: &[u8]) {
            self.notes.push(String::from_utf8_lossy(what).into_owned());
        }
        fn findact(&mut self, name: &[u8]) -> i32 {
            if stricmp_eq(name, b"FIRE_BOLT") {
                7
            } else {
                0
            }
        }
        fn find_class(&mut self, name: &[u8]) -> i32 {
            if stricmp_eq(name, b"warrior") {
                0
            } else if stricmp_eq(name, b"mage") {
                1
            } else {
                -1
            }
        }
        fn lookup_sval(&mut self, tval: i32, name: &[u8]) -> i32 {
            if tval == TV_POTION as i32 && stricmp_eq(name, b"Cure Light Wounds") {
                2
            } else {
                -1
            }
        }
        fn lookup_kind(&mut self, tval: i32, sval: i32) -> i32 {
            tval * 100 + sval
        }
    }

    impl TestEnv {
        pub(crate) fn new(dynamic: bool) -> TestEnv {
            let mut st = BorgState::default();
            st.core.cfg = vec![0; crate::borg_headers::borg_h::BORG_MAX_SETTINGS as usize];
            st.core.cfg[BORG_USES_DYNAMIC_CALCS as usize] = dynamic as i32;
            st.borg.trait_ = vec![0; BI_MAX as usize];
            TestEnv {
                st,
                notes: Vec::new(),
            }
        }

        fn take_notes(&mut self) -> Vec<String> {
            std::mem::take(&mut self.notes)
        }

        /// Run `load_formulas` on the given text (lines read as
        /// file_getl() would, for text without tabs or `\r`).
        pub(crate) fn load(&mut self, text: &str) -> bool {
            let mut lines = text.lines();
            load_formulas(self, |buf| match lines.next() {
                Some(l) => {
                    let n = l.len().min(LINE_BUF - 2);
                    buf[..n].copy_from_slice(&l.as_bytes()[..n]);
                    buf[n] = 0;
                    true
                }
                None => {
                    buf[0] = 0;
                    false
                }
            })
        }

        fn reason(&mut self, restock: bool, depth: i32) -> Option<String> {
            let i = dynamic_reason(self, restock, depth)?;
            let r = &depth_lines(&self.st, restock)[i].reason;
            Some(r.to_str().unwrap().to_string())
        }
    }

    #[test]
    fn value_type_table_matches_header() {
        use crate::headers::list_tvals_h::{VT_GOLD, VT_POTION};
        assert_eq!(get_value_type(b"error"), VT_NONE);
        assert_eq!(get_value_type(b"RANGE"), VT_RANGE_INDEX);
        assert_eq!(get_value_type(b"Trait"), VT_TRAIT);
        assert_eq!(get_value_type(b"config"), VT_CONFIG);
        assert_eq!(get_value_type(b"activation"), VT_ACTIVATION);
        assert_eq!(get_value_type(b"class"), VT_CLASS);
        assert_eq!(get_value_type(b"none"), VT_NULL);
        assert_eq!(get_value_type(b"potion"), VT_POTION);
        assert_eq!(get_value_type(b"gold"), VT_GOLD);
        assert_eq!(get_value_type(b"gold "), VT_NONE);
        assert_eq!(VT_MAX as usize, VALUE_TYPE_NAMES.len() - 1);
    }

    #[test]
    fn strtol_and_depth() {
        assert_eq!(c_strtol(b"  -12x"), (-12, 5));
        assert_eq!(c_strtol(b"+"), (0, 0));
        assert_eq!(c_strtol(b"x"), (0, 0));
        assert_eq!(c_strtol(b"99999999999999999999"), (i64::MAX, 20));
        assert_eq!(parse_depth(b"depth(5)"), 5);
        assert_eq!(parse_depth(b"depth(  7  )"), 7);
        assert_eq!(parse_depth(b"depth(-3)"), -3);
        assert_eq!(parse_depth(b"depth(5"), -1);
        assert_eq!(parse_depth(b"depth()"), -1);
        assert_eq!(parse_depth(b"depth 5"), -1);
        assert_eq!(parse_depth(b"depth(5x)"), -1);
        // long -> int truncation
        assert_eq!(parse_depth(b"depth(4294967301)"), 5);
        assert_eq!(parse_depth(b"depth(99999999999999999999)"), -1);
    }

    #[test]
    fn helpers() {
        assert!(is_all_number(b""));
        assert!(is_all_number(b"0123"));
        assert!(!is_all_number(b"12 "));
        assert!(!is_all_number(b"-1"));
        assert!(prefix_i(b"POWER x", b"power"));
        assert!(!prefix_i(b"pow", b"power"));
        assert_eq!(cstr(b"ab\0cd", 0), b"ab");
        assert_eq!(cstr(b"ab\0cd", 3), b"cd");
        assert_eq!(cstr(b"ab", 5), b"");
        assert_eq!(last_index(b"(x)", 1), 2);
        assert_eq!(last_index(b"(", 1), 0);
    }

    #[test]
    fn get_section_cuts_at_colon_and_semicolon() {
        let mut env = TestEnv::new(true);
        let mut v = None;
        assert!(get_section(
            &mut env,
            b"depth(5):condition(1);",
            b"depth",
            &mut v,
            b""
        ));
        assert_eq!(v.as_deref(), Some(&b"depth(5)"[..]));
        let mut v = None;
        assert!(get_section(
            &mut env,
            b"Condition(1);",
            b"condition",
            &mut v,
            b""
        ));
        assert_eq!(v.as_deref(), Some(&b"Condition(1)"[..]));
        let mut v = None;
        assert!(get_section(
            &mut env,
            b"reward(1);;",
            b"reward",
            &mut v,
            b""
        ));
        assert_eq!(v.as_deref(), Some(&b"reward(1);"[..]));
        assert!(!get_section(
            &mut env,
            b"reward(2);",
            b"reward",
            &mut v,
            b"full"
        ));
        assert_eq!(
            env.take_notes(),
            [
                "** borg formula failure ** ",
                "** error on line 'full'",
                "** error for section value 'reward(2);'",
                "** section repeated",
                "** formulas disabled ** ",
            ]
        );
    }

    #[test]
    fn loads_and_evaluates() {
        let mut env = TestEnv::new(true);
        let off = env.load(
            "# comment\n\
             \n\
             Power\n\
             [\n\
             \tcondition(value(trait, str) > 3): reward(value(trait, str) * 10);\n\
             reward(7);\n\
             value(trait, int):range(2, 4):reward(value(range, x) + 100);\n\
             ]\n\
             Depth Requirement\n\
             [\n\
             depth(5): condition(value(trait, str) < 10);\n\
             depth(20): condition(1);\n\
             ]\n\
             Restock Requirement\n\
             [\n\
             depth(1):condition(0);\n\
             ]\n\
             [END FORMULA SECTION]\n\
             not read\n",
        );
        assert!(!off, "{:?}", env.take_notes());
        assert!(env.take_notes().is_empty());
        assert_eq!(env.st.formulas.power.len(), 3);
        assert_eq!(env.st.formulas.depth.len(), 2);
        assert_eq!(env.st.formulas.restock.len(), 1);
        assert_eq!(
            env.st.formulas.power[1].reward,
            MathSection {
                is_calculation: false,
                calculation_index: 7
            }
        );
        assert_eq!(
            env.st.formulas.power[2].range,
            Some(RangeSec { from: 2, to: 4 })
        );
        assert_eq!(
            env.st.formulas.power[2].value,
            Some(ValueSec {
                vtype: VT_TRAIT,
                index: crate::borg_headers::borg_trait_h::BI_INT as i32
            })
        );
        assert_eq!(
            env.st.formulas.depth[0].reason.as_bytes(),
            b"condition(value(trait, str) < 10"
        );
        assert_eq!(env.st.formulas.depth[1].reason.as_bytes(), b"condition(1");

        // str 5, int 3: 50 + 7 + (range 2..=3: (1 + 100) + (2 + 100))
        env.st.borg.trait_[BI_STR as usize] = 5;
        env.st.borg.trait_[crate::borg_headers::borg_trait_h::BI_INT as usize] = 3;
        let mut total = 0;
        for i in 0..env.st.formulas.power.len() {
            let p = env.st.formulas.power[i];
            total += calc_power(&mut env, &p);
        }
        assert_eq!(total, 50 + 7 + 101 + 102);
        env.st.borg.trait_[BI_STR as usize] = 2;
        let p = env.st.formulas.power[0];
        assert_eq!(calc_power(&mut env, &p), 0);

        assert_eq!(env.reason(false, 4), None);
        assert_eq!(
            env.reason(false, 5).as_deref(),
            Some("condition(value(trait, str) < 10")
        );
        env.st.borg.trait_[BI_STR as usize] = 10;
        assert_eq!(env.reason(false, 19), None);
        assert_eq!(env.reason(false, 20).as_deref(), Some("condition(1"));
        assert_eq!(env.reason(true, 50), None);

        borg_free_formulas(&mut env.st);
        assert!(env.st.formulas.power.is_empty());
        assert!(env.st.formulas.depth.is_empty());
        assert!(env.st.formulas.restock.is_empty());
        assert!(env.st.formulas_calc.calculations.is_empty());
    }

    #[test]
    fn off_when_not_dynamic() {
        let mut env = TestEnv::new(false);
        assert!(env.load("Power\n[\nbad line\n]\n[END FORMULA SECTION]\n"));
        assert!(env.st.formulas.power.is_empty());
        assert!(env.take_notes().is_empty());
    }

    /// Errors as the C code reports them (checked against borg-formulas.c
    /// compiled with stubs).
    #[test]
    fn errors_match_c() {
        let check = |text: &str, want_off: bool, want: &[String]| {
            let mut env = TestEnv::new(true);
            let off = env.load(text);
            assert_eq!(off, want_off, "{text}");
            assert_eq!(env.take_notes(), want, "{text}");
            if want_off {
                assert_eq!(env.st.cfg(BORG_USES_DYNAMIC_CALCS), 0);
            }
        };
        let fail = |line: &str, label: &str, section: &str, error: Option<&str>| {
            let mut v = vec![
                "** borg formula failure ** ".to_string(),
                format!("** error on line '{line}'"),
                format!("** error for section {label} '{section}'"),
            ];
            v.extend(error.map(str::to_string));
            v.push("** formulas disabled ** ".to_string());
            v
        };

        let w = fail(
            "  reward(1)",
            "power",
            "  reward(1)",
            Some("** line must end in a ;"),
        );
        check("Power\n[\n  reward(1)\n", true, &w);

        let w = fail(
            "x;",
            "base",
            "x;",
            Some("** line must be part of a Power or Depth section"),
        );
        check("x;\n", true, &w);

        let w = fail(
            "reward(1):junk;",
            "power",
            "reward(1):junk;",
            Some("** unparsed information 'junk;' in power line"),
        );
        check("Power\n[\nreward(1):junk;\n", true, &w);

        let w = fail(
            "range(1,2):reward(1);",
            "power",
            "range(1,2):reward(1);",
            Some("** if range() is specified value() must also be specified"),
        );
        check("Power\n[\nrange(1,2):reward(1);\n", true, &w);

        let w = fail(
            "condition(1);",
            "power",
            "condition(1);",
            Some("** reward() must be specified specified"),
        );
        check("Power\n[\ncondition(1);\n", true, &w);

        // the section printed is cut at the ',' (C writes a NUL there)
        let w = fail(
            "value(trait,str):range(1, 2:reward(1);",
            "range",
            "range(1",
            Some("** first value is not a number"),
        );
        check(
            "Power\n[\nvalue(trait,str):range(1, 2:reward(1);\n",
            true,
            &w,
        );

        // a first value that is not a number fails silently
        check(
            "Power\n[\nvalue(trait,str):range(x, 2):reward(1);\n",
            true,
            &[],
        );

        let w = fail(
            "value(trait,str):range(3, 2):reward(1);",
            "range",
            "range(3",
            Some("** second value less first value"),
        );
        check(
            "Power\n[\nvalue(trait,str):range(3, 2):reward(1);\n",
            true,
            &w,
        );

        let w = fail(
            "value(trait,str):range(0, 2):reward(1);",
            "range",
            "range(0",
            Some("** first value less than one"),
        );
        check(
            "Power\n[\nvalue(trait,str):range(0, 2):reward(1);\n",
            true,
            &w,
        );

        let w = fail(
            "reward(value(bogus, x));",
            "value",
            "value(bogus",
            Some("** error for section value 'value(bogus'"),
        );
        check("Power\n[\nreward(value(bogus, x));\n", true, &w);

        let w = fail(
            "reward(value(trait, nope));",
            "value",
            "nope",
            Some("** unable to find trait"),
        );
        check("Power\n[\nreward(value(trait, nope));\n", true, &w);

        let w = fail(
            "reward(value(none, x));",
            "value",
            "x",
            Some("** invalid value type"),
        );
        check("Power\n[\nreward(value(none, x));\n", true, &w);

        let w = fail("reward(1;", "reward", "reward(1", Some("** no ending ')'"));
        check("Power\n[\nreward(1;\n", true, &w);

        let w = fail(
            "depth(x):condition(1);",
            "value",
            "depth(x):condition(1);",
            Some("** depth section can't be parsed"),
        );
        check("Depth Requirement\n[\ndepth(x):condition(1);\n", true, &w);

        let w = fail(
            "depth(1);",
            "value",
            "depth(1);",
            Some("** condition section missing"),
        );
        check("Depth Requirement\n[\ndepth(1);\n", true, &w);

        let w = fail("condition;", "condition", "condition", None);
        check("Restock Requirement\n[\ndepth(1):condition;\n", true, &{
            let mut w = w;
            w[1] = "** error on line 'depth(1):condition;'".to_string();
            w
        });

        // an unknown object name: the error note repeats the one before
        let w = fail(
            "reward(value(potion, Healing));",
            "value",
            "Healing",
            Some("** error for section value 'Healing'"),
        );
        check("Power\n[\nreward(value(potion, Healing));\n", true, &w);
        let mut env = TestEnv::new(true);
        assert!(!env.load("Power\n[\nreward(value(potion, cure light wounds));\n"));
        let want = ValueSec {
            vtype: crate::headers::list_tvals_h::VT_POTION,
            index: TV_POTION as i32 * 100 + 2,
        };
        let c = &env.st.formulas_calc.calculations[0];
        assert!(
            matches!(c.tokens[0].kind, crate::borg_formulas_calc::TokenKind::Value(v) if v == want)
        );

        // a repeated section: two errors
        let mut w = fail(
            "reward(2):reward(3);",
            "value",
            "reward(3);",
            Some("** section repeated"),
        );
        w.extend(fail(
            "reward(2):reward(3);",
            "power",
            "reward(2):reward(3);",
            Some("** unparsed information 'reward(3);' in power line"),
        ));
        check("Power\n[\nreward(2):reward(3);\n", true, &w);

        // a second '[' without a section header in between turns formulas
        // off without a note; ']' outside a section returns at once
        check("Power\n[\n[\n", true, &[]);
        let mut env = TestEnv::new(true);
        assert!(!env.load("]\nx;\n"));
        assert!(env.take_notes().is_empty());
        assert_eq!(env.st.cfg(BORG_USES_DYNAMIC_CALCS), 1);

        // config values drop the "borg_" prefix
        check(
            "Power\n[\ncondition(value(config, worships_damage)): reward(1);\n]\n",
            false,
            &[],
        );
    }
}
