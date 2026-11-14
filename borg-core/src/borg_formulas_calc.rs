//! Port of `borg-formulas-calc.c`: the calculator for dynamic formulas.
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
//! The parser works on the bytes of a C string: reading past the end yields
//! the terminating NUL (0), as in C. The calls out of this module
//! (`parse_value`, `calculate_from_value`, `borg_formula_error` in
//! borg_formulas, and `borg_note`) go through the [`CalcEnv`] trait so the
//! calculator itself can be unit tested.

use crate::borg_formulas::ValueSec;
use crate::cctype::c_isdigit;
use crate::state::BorgState;

/// `OP_LEVELS`
const OP_LEVELS: usize = 4;

/// Operator tokens (`TOK_MINUS` .. `TOK_GE`).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Op {
    Minus,
    Plus,
    Mult,
    Div,
    And,
    Or,
    Equals,
    Lt,
    Gt,
    Le,
    Ge,
}

/// `enum token_type` plus the token's payload (`void *token`).
#[derive(Clone, Copy)]
pub enum TokenKind {
    /// `TOK_VALUE`: a parsed `value(type, index)`
    Value(ValueSec),
    /// `TOK_NUMBER`
    Number(i32),
    /// `TOK_MINUS` .. `TOK_GE`
    Op(Op),
}

/// `struct token`
#[derive(Clone, Copy)]
pub struct Token {
    pub kind: TokenKind,
    pub not: bool,
    pub pdepth: i32,
}

/// `struct borg_calculation`
#[derive(Clone, Default)]
pub struct Calculation {
    pub tokens: Vec<Token>,
    pub max_depth: i32,
}

/// borg-formulas-calc.c globals.
#[derive(Default)]
pub struct CalcState {
    /// `calculations`
    pub calculations: Vec<Calculation>,
}

/// What the calculator needs from the rest of the Borg.
pub trait CalcEnv {
    /// `parse_value(value, full_line)`; `None` for NULL.
    fn parse_value(&mut self, value: &[u8], full_line: &[u8]) -> Option<ValueSec>;
    /// `calculate_from_value(value, range_index)`
    fn calculate_from_value(&mut self, value: &ValueSec, range_index: i32) -> i32;
    /// `borg_formula_error(section, full_line, label, error)`
    fn formula_error(&mut self, section: &[u8], full_line: &[u8], label: &str, error: &str);
    /// `borg_note(what)`
    fn note(&mut self, what: &str);
}

/// `*(s + i)` on a C string held as a slice without its NUL.
fn at(s: &[u8], i: usize) -> u8 {
    s.get(i).copied().unwrap_or(0)
}

/// `prefix_i(s, t)` (z-util.c): case-insensitive prefix test.
fn prefix_i(s: &[u8], t: &[u8]) -> bool {
    t.iter()
        .enumerate()
        .all(|(i, c)| c.eq_ignore_ascii_case(&at(s, i)))
}

/// `atol(s)` (glibc: `strtol(s, NULL, 10)`, saturating at the `long`
/// limits on overflow).
fn c_atol(s: &[u8]) -> i64 {
    let mut p = 0;
    while crate::cctype::c_isspace(at(s, p)) {
        p += 1;
    }
    let mut neg = false;
    if at(s, p) == b'-' || at(s, p) == b'+' {
        neg = at(s, p) == b'-';
        p += 1;
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
    if overflow {
        return if neg { i64::MIN } else { i64::MAX };
    }
    if neg {
        (mag as i64).wrapping_neg()
    } else {
        mag as i64
    }
}

/// `get_operator()`: grab an operator (+, -, and, or etc) from the line.
/// Advances `*p` past it.
fn get_operator(line: &[u8], p: &mut usize) -> Option<Op> {
    let s = &line[*p..];
    let c = at(s, 0);
    if c == b'+' {
        *p += 1;
        return Some(Op::Plus);
    }
    if c == b'-' {
        if c_isdigit(at(s, 1)) {
            return None;
        }
        *p += 1;
        return Some(Op::Minus);
    }
    if c == b'*' {
        *p += 1;
        return Some(Op::Mult);
    }
    if c == b'/' {
        *p += 1;
        return Some(Op::Div);
    }
    if prefix_i(s, b"and") {
        *p += 3;
        return Some(Op::And);
    }
    if prefix_i(s, b"&&") {
        *p += 2;
        return Some(Op::And);
    }
    if prefix_i(s, b"or") {
        *p += 2;
        return Some(Op::Or);
    }
    if prefix_i(s, b"||") {
        *p += 2;
        return Some(Op::Or);
    }
    if c == b'=' {
        *p += 1;
        return Some(Op::Equals);
    }
    if c == b'>' {
        *p += 1;
        if at(s, 1) == b'=' {
            *p += 1;
            return Some(Op::Ge);
        }
        return Some(Op::Gt);
    }
    if c == b'<' {
        *p += 1;
        if at(s, 1) == b'=' {
            *p += 1;
            return Some(Op::Le);
        }
        return Some(Op::Lt);
    }
    None
}

/// `get_value_string()`: if `s` starts with "value", the text up to and
/// including the first ')' (or all of it).
fn get_value_string(s: &[u8]) -> Option<&[u8]> {
    if prefix_i(s, b"value") {
        // C copies this into a 1024-byte stack buffer; lines are read into
        // 1024-byte buffers too, so it cannot overflow.
        let len = match s.iter().position(|&c| c == b')') {
            Some(i) => i + 1,
            None => s.len(),
        };
        return Some(&s[..len]);
    }
    None
}

/// `tokenize_math()`: turn a formula into an array of tokens. Returns
/// true on failure.
fn tokenize_math(
    env: &mut impl CalcEnv,
    f: &mut Calculation,
    line: &[u8],
    full_line: &[u8],
) -> bool {
    let mut fail = false;
    let mut negate = false;
    let mut pdepth: i32 = 0;
    let start = line;
    let mut p: usize = 0;

    f.tokens = Vec::new();

    while at(line, p) != 0 {
        if let Some(value_token) = get_value_string(&line[p..]) {
            p += value_token.len();
            let v = env.parse_value(value_token, full_line);
            if v.is_none() {
                fail = true;
            }
            // `fail` may already be set by an earlier "not (!)" error.
            let Some(v) = v.filter(|_| !fail) else {
                break;
            };
            if pdepth > f.max_depth {
                f.max_depth = pdepth;
            }
            f.tokens.push(Token {
                kind: TokenKind::Value(v),
                not: negate,
                pdepth,
            });
            negate = false;
            continue;
        }

        let c = at(line, p);
        if c == b'!' {
            negate = true;
            p += 1;
            continue;
        }

        if c == b'(' {
            pdepth += 1;
            p += 1;
            continue;
        }

        if c == b')' {
            pdepth -= 1;
            if pdepth < 0 {
                env.formula_error(start, full_line, "calculation", "** unmatched parenthesis");
                fail = true;
                break;
            }
            p += 1;
            continue;
        }

        if c_isdigit(c) || (c == b'-' && c_isdigit(at(line, p + 1))) {
            let val_start = p;
            let val = c_atol(&line[p..]) as i32;

            if pdepth > f.max_depth {
                f.max_depth = pdepth;
            }
            f.tokens.push(Token {
                kind: TokenKind::Number(val),
                not: negate,
                pdepth,
            });
            negate = false;
            while c_isdigit(at(line, p)) || (at(line, p) == b'-' && p == val_start) {
                p += 1;
            }
            if at(line, p) == b'-' && p != val_start {
                env.formula_error(start, full_line, "calculation", "** number parsing error");
            }
            continue;
        }

        // get operator moves the line pointer since operators are
        // varying length
        if let Some(op) = get_operator(line, &mut p) {
            if pdepth > f.max_depth {
                f.max_depth = pdepth;
            }
            f.tokens.push(Token {
                kind: TokenKind::Op(op),
                not: false,
                pdepth,
            });
            if negate {
                env.formula_error(
                    start,
                    full_line,
                    "calculation",
                    "** not (!) can only be applied to values",
                );
                fail = true;
            }
            continue;
        }
        p += 1;
    }
    if pdepth != 0 && !fail {
        env.formula_error(start, full_line, "calculation", "** unmatched parenthesis");
        fail = true;
    }

    fail
}

/// `is_operator()`
fn is_operator(t: &Token) -> bool {
    matches!(t.kind, TokenKind::Op(_))
}

/// `operation_level()`: precedence order for operators
/// 0) mult/div 1) add/sub 2) compare (eq, gt, lt, le, ge) 3) logic (and or)
fn operation_level(op: Op) -> usize {
    match op {
        Op::Div | Op::Mult => 0,
        Op::Plus | Op::Minus => 1,
        Op::And | Op::Or => 3,
        _ => 2,
    }
}

/// `add_depth()`: add "depth" (virtual parenthesis) to the operation at
/// `pos` and its operands.
fn add_depth(f: &mut Calculation, pos: usize) {
    let pdepth = f.tokens[pos].pdepth;

    f.tokens[pos].pdepth += 1;

    // left
    for i in (0..pos).rev() {
        let tok = &mut f.tokens[i];
        // check for after this operand
        if tok.pdepth < pdepth || (is_operator(tok) && tok.pdepth == pdepth) {
            break;
        }
        tok.pdepth += 1;
        if tok.pdepth > f.max_depth {
            f.max_depth = tok.pdepth;
        }
    }
    // right
    let end = f.tokens.len();
    for i in pos + 1..end {
        let tok = &mut f.tokens[i];
        if tok.pdepth < pdepth || (is_operator(tok) && tok.pdepth == pdepth) {
            break;
        }
        tok.pdepth += 1;
        if tok.pdepth > f.max_depth {
            f.max_depth = tok.pdepth;
        }
    }
}

/// `adjust_order_ops_depth_block()`: make the depths of the tokens in
/// `[start, end)` at `pdepth` reflect the order of operations, as if
/// `a + b * c = d` were `(a + (b * c)) = d`. Always returns false.
fn adjust_order_ops_depth_block(f: &mut Calculation, pdepth: i32, start: i32, end: i32) -> bool {
    // (level, position) of each operator at this depth
    let mut operators: Vec<(usize, usize)> = Vec::new();
    let mut l = [0i32; OP_LEVELS];

    let mut tok = start;
    while tok < end {
        let token = &f.tokens[tok as usize];
        if token.pdepth == pdepth {
            if let TokenKind::Op(op) = token.kind {
                let level = operation_level(op);
                l[level] += 1;
                operators.push((level, tok as usize));
            }
        }
        tok += 1;
    }

    // Group the operators by level and push the higher-precedence ones
    // (and all but the last at a level, so a*b*c is (a*b)*c) one deeper.
    // `count_at_level` counts across levels (as in C), so at the last
    // level all but the last operator are deepened.
    let count = operators.len() as i32;
    if count > 1 {
        let mut count_at_level: i32 = 0;
        for lvl in 0..OP_LEVELS {
            if l[lvl] != 0 {
                for &(level, position) in &operators {
                    if level == lvl {
                        let higher = l[lvl + 1..].iter().any(|&n| n != 0);
                        if higher || count_at_level < count - 1 {
                            add_depth(f, position);
                            count_at_level += 1;
                        }
                    }
                }
            }
        }
    }

    false
}

/// `adjust_order_operations()`: apply order of operations by pretending
/// there are parens around operations of higher precedence.
fn adjust_order_operations(f: &mut Calculation) -> bool {
    let mut fail = false;
    let mut pdepth: i32 = 0;
    // max_depth grows while this runs; C re-reads it each time round.
    while pdepth <= f.max_depth {
        let mut start: i32 = -1;
        let mut itoken: i32 = 0;
        while (itoken as usize) < f.tokens.len() {
            let tok_depth = f.tokens[itoken as usize].pdepth;
            if start == -1 && tok_depth == pdepth {
                start = itoken;
                itoken += 1;
                continue;
            }
            // do a block of tokens
            if start != -1 && tok_depth < pdepth {
                fail = adjust_order_ops_depth_block(f, pdepth, start, itoken - 1);
                start = -1;
            }
            itoken += 1;
        }
        if start != -1 {
            fail = adjust_order_ops_depth_block(f, pdepth, start, itoken - 1);
        }
        pdepth += 1;
    }
    fail
}

/// `validate_calculation()`: check the format of a formula. Returns true
/// on failure.
fn validate_calculation(
    env: &mut impl CalcEnv,
    f: &Calculation,
    line: &[u8],
    full_line: &[u8],
) -> bool {
    if f.tokens.is_empty() {
        env.formula_error(
            line,
            full_line,
            "calculation",
            "** formula must contain values",
        );
        return true;
    }

    for (ntoken, tok) in f.tokens.iter().enumerate() {
        if is_operator(tok) != (ntoken % 2 == 1) {
            env.formula_error(
                line,
                full_line,
                "calculation",
                "** formula must be values separated by operators",
            );
            return true;
        }
    }
    if is_operator(&f.tokens[f.tokens.len() - 1]) {
        env.formula_error(
            line,
            full_line,
            "calculation",
            "** formula can't end in an operation",
        );
        return true;
    }
    false
}

/// `parse_calculation()`: parse the formula into a structure that can be
/// calculated from. Returns true on failure.
fn parse_calculation(
    env: &mut impl CalcEnv,
    f: &mut Calculation,
    line: &[u8],
    full_line: &[u8],
) -> bool {
    if tokenize_math(env, f, line, full_line) {
        return true;
    }

    // modify the depth's to conform with order of operations
    if adjust_order_operations(f) {
        return true;
    }

    validate_calculation(env, f, line, full_line)
}

/// `parse_calculation_line()` on an explicit list of calculations.
pub(crate) fn parse_calculation_line_in(
    env: &mut impl CalcEnv,
    calculations: &mut Vec<Calculation>,
    line: &[u8],
    full_line: &[u8],
) -> i32 {
    let mut f = Calculation::default();
    if parse_calculation(env, &mut f, line, full_line) {
        return -1;
    }

    // borg_array_add() never fails, so C's "** can't add to formula
    // array" error is unreachable.
    calculations.push(f);
    (calculations.len() - 1) as i32
}

/// The value of a single (non-operator) operand token.
fn operand_value(env: &mut impl CalcEnv, t: &Token, range_index: i32) -> i32 {
    let mut v = 0;
    match t.kind {
        TokenKind::Number(n) => {
            v = n;
            if t.not {
                v = (v == 0) as i32;
            }
        }
        TokenKind::Value(ref value) => {
            v = env.calculate_from_value(value, range_index);
            if t.not {
                v = (v == 0) as i32;
            }
        }
        TokenKind::Op(_) => {}
    }
    v
}

/// `calculate_value_from_formula_depth()`: the value of the formula at
/// one parenthesis level, recursing for deeper levels.
fn calculate_value_from_formula_depth(
    env: &mut impl CalcEnv,
    f: &[Token],
    i: &mut usize,
    pdepth: i32,
    range_index: i32,
) -> i32 {
    let left_token = f[*i];

    // get the value on the left
    let left_value = if left_token.pdepth > pdepth {
        calculate_value_from_formula_depth(env, f, i, pdepth + 1, range_index)
    } else {
        operand_value(env, &left_token, range_index)
    };

    if *i + 1 >= f.len() {
        return left_value;
    }

    let operation = f[*i + 1];
    if operation.pdepth < pdepth {
        return left_value;
    }
    *i += 1;

    // get the value on the right
    *i += 1;
    let right_token = f[*i];
    let right_value = if right_token.pdepth > pdepth {
        calculate_value_from_formula_depth(env, f, i, pdepth + 1, range_index)
    } else {
        operand_value(env, &right_token, range_index)
    };

    let mut total: i32 = 0;
    match operation.kind {
        TokenKind::Op(Op::Minus) => total = left_value.wrapping_sub(right_value),
        TokenKind::Op(Op::Plus) => total = left_value.wrapping_add(right_value),
        TokenKind::Op(Op::Mult) => total = left_value.wrapping_mul(right_value),
        TokenKind::Op(Op::Div) => {
            if right_value == 0 {
                env.note("** borg formula failure ** ");
                env.note("** divide by zero error calculating a formula value");
                env.note("** setting denominator to one");
                total = left_value;
            } else {
                // i32::MIN / -1 traps in C (SIGFPE); here it panics.
                total = left_value / right_value;
            }
        }
        TokenKind::Op(Op::And) => total = (left_value != 0 && right_value != 0) as i32,
        TokenKind::Op(Op::Or) => total = (left_value != 0 || right_value != 0) as i32,
        TokenKind::Op(Op::Equals) => total = (left_value == right_value) as i32,
        TokenKind::Op(Op::Lt) => total = (left_value < right_value) as i32,
        TokenKind::Op(Op::Gt) => total = (left_value > right_value) as i32,
        TokenKind::Op(Op::Le) => total = (left_value <= right_value) as i32,
        TokenKind::Op(Op::Ge) => total = (left_value >= right_value) as i32,
        // should never get here.
        _ => {
            env.note("** borg formula failure ** ");
            env.note("** error calculating a formula value");
        }
    }
    if *i + 1 >= f.len() {
        return total;
    }
    let next_token = f[*i + 1];
    if next_token.pdepth != pdepth {
        return total;
    }

    // we should never get here.  The parsing should handle all validation
    env.note("** borg formula failure ** ");
    env.note("** error calculating a formula value");
    0
}

/// `borg_calculate_dynamic()` on an explicit list of calculations.
pub(crate) fn calculate_dynamic_in(
    env: &mut impl CalcEnv,
    calculations: &[Calculation],
    formula: i32,
    range_index: i32,
) -> i32 {
    let mut i = 0;
    let f = &calculations[formula as usize];
    calculate_value_from_formula_depth(env, &f.tokens, &mut i, 0, range_index)
}

/// `calculations_free()`: free up all memory used in calculations.
pub fn calculations_free(st: &mut BorgState) {
    st.formulas_calc.calculations.clear();
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test environment: `value(N)` parses to an index; the value of
    /// index n is `vals[n]`, and index 99 means "the range index".
    #[derive(Default)]
    struct Env {
        vals: Vec<i32>,
        errors: Vec<String>,
        notes: Vec<String>,
        values_parsed: Vec<String>,
    }

    impl CalcEnv for Env {
        fn parse_value(&mut self, value: &[u8], _full: &[u8]) -> Option<ValueSec> {
            let s = String::from_utf8(value.to_vec()).unwrap();
            self.values_parsed.push(s.clone());
            let inner = s.to_ascii_lowercase();
            let inner = inner.strip_prefix("value(")?.strip_suffix(')')?;
            let index = inner.trim().parse().ok()?;
            Some(ValueSec { vtype: 0, index })
        }
        fn calculate_from_value(&mut self, v: &ValueSec, range_index: i32) -> i32 {
            if v.index == 99 {
                range_index
            } else {
                self.vals[v.index as usize]
            }
        }
        fn formula_error(&mut self, _s: &[u8], _f: &[u8], label: &str, error: &str) {
            self.errors.push(format!("{label}: {error}"));
        }
        fn note(&mut self, what: &str) {
            self.notes.push(what.to_string());
        }
    }

    fn parse(env: &mut Env, s: &str) -> Option<Calculation> {
        let mut c = Vec::new();
        let r = parse_calculation_line_in(env, &mut c, s.as_bytes(), s.as_bytes());
        if r == -1 {
            assert!(c.is_empty());
            None
        } else {
            assert_eq!(r, 0);
            c.pop()
        }
    }

    fn eval_env(env: &mut Env, s: &str, range: i32) -> i32 {
        let c = parse(env, s).unwrap_or_else(|| panic!("parse failed: {s} {:?}", env.errors));
        calculate_dynamic_in(env, &[c], 0, range)
    }

    fn eval(s: &str) -> i32 {
        let mut env = Env::default();
        let v = eval_env(&mut env, s, 0);
        assert!(env.errors.is_empty(), "{:?}", env.errors);
        v
    }

    fn depths(s: &str) -> Vec<i32> {
        parse(&mut Env::default(), s)
            .unwrap()
            .tokens
            .iter()
            .map(|t| t.pdepth)
            .collect()
    }

    /// Results of the C code (borg-formulas-calc.c compiled with stubs
    /// where value(n) is n * 7 - 10 and value(99) the range index 0).
    #[test]
    fn pinned_c_results() {
        let cases = [
            ("2 && value(99) and value(1) - !0 || !0 - !value(0) / 2 <= !0 > 100 = value(99)", 1),
            ("(3 + value(99) - 100 >= 1 && 10 and 0 > !0) && -4", 0),
            ("2 - (value(99) <= !value(0) + 100 = 10 && 10 + 1 > value(5) > !0 < 10)", 2),
            ("(!0 and 1 >= value(1) * 7 or 2) && value(1) <= value(5)", 1),
            ("!0 < (1 <= 0 < 1) || value(1) or 100 >= 1 or 100 = !0", 1),
            ("((!value(0) >= value(1) >= 10 < 10) or -4 and (2 or 0 / 1 > !0 || 2 = 2) / value(5))", 0),
            ("1 <= !0 && (2 / -4 = !0 * 100 + 10 or 7 > 1 <= value(99) || 0)", 0),
        ];
        for (s, want) in cases {
            let mut env = Env {
                vals: (0..10).map(|i| i * 7 - 10).collect(),
                ..Env::default()
            };
            assert_eq!(eval_env(&mut env, s, 0), want, "{s}");
        }
    }

    #[test]
    fn atol_matches_glibc() {
        assert_eq!(c_atol(b"123abc"), 123);
        assert_eq!(c_atol(b"-42-1"), -42);
        assert_eq!(c_atol(b" \x0b+7"), 7);
        assert_eq!(c_atol(b"9223372036854775807"), i64::MAX);
        assert_eq!(c_atol(b"9223372036854775808"), i64::MAX);
        assert_eq!(c_atol(b"-9223372036854775808"), i64::MIN);
        assert_eq!(c_atol(b"-99999999999999999999"), i64::MIN);
        assert_eq!(c_atol(b"x1"), 0);
        // atol() then (int32_t): truncation
        assert_eq!(eval("4294967297"), 1);
        assert_eq!(eval("2147483648"), i32::MIN);
    }

    #[test]
    fn single_values() {
        assert_eq!(eval("42"), 42);
        assert_eq!(eval("-42"), -42);
        assert_eq!(eval("!0"), 1);
        assert_eq!(eval("!5"), 0);
        assert_eq!(eval("((7))"), 7);
    }

    #[test]
    fn arithmetic_and_precedence() {
        assert_eq!(eval("1 + 2 * 3"), 7);
        assert_eq!(eval("2 * 3 + 1"), 7);
        assert_eq!(eval("(1 + 2) * 3"), 9);
        assert_eq!(eval("10 - 3 - 2"), 5);
        assert_eq!(eval("100 / 10 / 5"), 2);
        assert_eq!(eval("2 * 3 * 4"), 24);
        assert_eq!(eval("1 + 2 * (3 + 4 * 5) + 6 + 7"), 60);
        assert_eq!(eval("7 / 2"), 3);
        assert_eq!(eval("-7 / 2"), -3);
        assert_eq!(eval("1 + 2 > 2"), 1);
        assert_eq!(eval("1 + 1 = 2 and 3 < 2"), 0);
        assert_eq!(eval("1 + 1 = 2 or 3 < 2"), 1);
        assert_eq!(eval("2 >= 2 && 1 <= 0"), 0);
        assert_eq!(eval("2 >= 2 || 1 <= 0"), 1);
        assert_eq!(eval("3 AND 4"), 1);
        assert_eq!(eval("0 Or 0"), 0);
        assert_eq!(eval("1 + 1 = 2"), 1);
        assert_eq!(eval("5 - 4"), 1);
        assert_eq!(eval("2147483647 + 1"), i32::MIN);
        assert_eq!(eval("65536 * 65536"), 0);
    }

    #[test]
    fn depths_follow_order_of_operations() {
        // 1 + 2 * 3 -> 1 + (2 * 3)
        assert_eq!(depths("1 + 2 * 3"), vec![0, 0, 1, 1, 1]);
        // a*b*c -> (a*b)*c
        assert_eq!(depths("2 * 3 * 4"), vec![1, 1, 1, 0, 0]);
        // a-b-c -> (a-b)-c
        assert_eq!(depths("10 - 3 - 2"), vec![1, 1, 1, 0, 0]);
    }

    #[test]
    fn mixed_levels_quirks() {
        // a + b * c = d becomes (a + (b * c)) = d.
        assert_eq!(depths("1 + 2 * 3 = 7"), vec![1, 1, 2, 2, 2, 0, 0]);
        assert_eq!(eval("1 + 2 * 3 = 7"), 1);
        assert_eq!(eval("2 * 3 + 4 * 5"), 26);
        // Four operators at one level: the last stays at depth 0.
        assert_eq!(eval("1 - 1 - 1 - 1 - 1"), -3);
    }

    #[test]
    fn values_and_negation() {
        let mut env = Env {
            vals: vec![10, 0, 3],
            ..Env::default()
        };
        assert_eq!(eval_env(&mut env, "value(0) * value(2) + !value(1)", 0), 31);
        assert_eq!(env.values_parsed, ["value(0)", "value(2)", "value(1)"]);
        assert_eq!(eval_env(&mut env, "VALUE(99) + 1", 5), 6);
        assert_eq!(eval_env(&mut env, "!value(0)", 0), 0);
        assert!(env.errors.is_empty());
        // without ')' the value string runs to the end
        let mut env = Env::default();
        assert!(parse(&mut env, "1 + value(3").is_none());
        assert_eq!(env.values_parsed, ["value(3"]);
        // a failed parse_value() fails silently here (parse_value reports it)
        assert!(env.errors.is_empty());
    }

    #[test]
    fn block_end_is_exclusive() {
        // The blocks passed to adjust_order_ops_depth_block() leave out
        // their last token, so the second '+' is not seen and the
        // evaluator hits its "should never get here" path.
        let mut env = Env::default();
        assert_eq!(depths("(1 + 2 +) 3"), vec![1, 1, 1, 1, 0]);
        assert_eq!(eval_env(&mut env, "(1 + 2 +) 3", 0), 3);
        assert!(env.errors.is_empty());
        assert_eq!(
            env.notes,
            [
                "** borg formula failure ** ",
                "** error calculating a formula value"
            ]
        );
    }

    #[test]
    fn divide_by_zero_notes_and_keeps_left() {
        let mut env = Env::default();
        assert_eq!(eval_env(&mut env, "7 / 0", 0), 7);
        assert_eq!(
            env.notes,
            [
                "** borg formula failure ** ",
                "** divide by zero error calculating a formula value",
                "** setting denominator to one"
            ]
        );
    }

    #[test]
    fn parse_errors() {
        let check = |s: &str, err: &str| {
            let mut env = Env::default();
            assert!(parse(&mut env, s).is_none(), "{s}");
            assert_eq!(env.errors, [format!("calculation: {err}")], "{s}");
        };
        check("", "** formula must contain values");
        check("   ", "** formula must contain values");
        check("1 +", "** formula can't end in an operation");
        check("1 2", "** formula must be values separated by operators");
        check("+ 1", "** formula must be values separated by operators");
        check(
            "1 + + 2",
            "** formula must be values separated by operators",
        );
        check("(1 + 2", "** unmatched parenthesis");
        check("1 + 2)", "** unmatched parenthesis");
        check("1 + !* 2", "** not (!) can only be applied to values");
    }

    #[test]
    fn not_error_stops_at_next_value() {
        // After the "not (!)" error the loop goes on, but the next value()
        // (parsed first) ends it, so "1-2" is never reached.
        let mut env = Env {
            vals: vec![1],
            ..Env::default()
        };
        assert!(parse(&mut env, "!+ value(0) + 1-2").is_none());
        assert_eq!(
            env.errors,
            ["calculation: ** not (!) can only be applied to values"]
        );
        assert_eq!(env.values_parsed, ["value(0)"]);
        // without a value() it goes on to the end
        let mut env = Env::default();
        assert!(parse(&mut env, "!+ 1-2").is_none());
        assert_eq!(env.errors.len(), 2);
    }

    #[test]
    fn number_followed_by_minus() {
        // "1-2" lexes as 1 and -2: a (non-fatal) number parsing error, then
        // the fatal operator check.
        let mut env = Env::default();
        assert!(parse(&mut env, "1-2").is_none());
        assert_eq!(
            env.errors,
            [
                "calculation: ** number parsing error",
                "calculation: ** formula must be values separated by operators"
            ]
        );
        // "1 - 2" is fine; "1 -2" is two numbers.
        assert_eq!(eval("1 - 2"), -1);
        let mut env = Env::default();
        assert!(parse(&mut env, "1 -2").is_none());
        // "5--3": 5 (and the error), the operator '-', then the number -3
        let mut env = Env::default();
        assert_eq!(eval_env(&mut env, "5--3", 0), 8);
        assert_eq!(env.errors, ["calculation: ** number parsing error"]);
    }

    #[test]
    fn unknown_bytes_are_skipped() {
        assert_eq!(eval("1 + x2"), 3);
        assert_eq!(eval("\x0b3\t*\n4"), 12);
        assert_eq!(eval("2 \u{e9} * 4"), 8);
    }

    #[test]
    fn indexes_and_free() {
        let mut env = Env::default();
        let mut c = Vec::new();
        assert_eq!(parse_calculation_line_in(&mut env, &mut c, b"1", b"1"), 0);
        assert_eq!(
            parse_calculation_line_in(&mut env, &mut c, b"1 +", b"1 +"),
            -1
        );
        assert_eq!(
            parse_calculation_line_in(&mut env, &mut c, b"2 * 3", b"2 * 3"),
            1
        );
        assert_eq!(calculate_dynamic_in(&mut env, &c, 1, 0), 6);
    }
}
