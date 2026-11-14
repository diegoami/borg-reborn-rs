//! C `<ctype.h>` predicates as the Borg sees them.
//!
//! The harness runs the game with `LC_ALL=C.UTF-8`. Measured with glibc,
//! the classes for single bytes there are exactly the ASCII ones below, and
//! no byte >= 0x80 is in any class. They differ from Rust's
//! `u8::is_ascii_whitespace`, which leaves out `\v` (0x0B).

/// `isspace()`: space, `\t`, `\n`, `\v`, `\f`, `\r`.
pub fn c_isspace(c: u8) -> bool {
    matches!(c, b' ' | b'\t' | b'\n' | 0x0b | 0x0c | b'\r')
}

/// `isdigit()`.
pub fn c_isdigit(c: u8) -> bool {
    c.is_ascii_digit()
}

/// `isalpha()`.
pub fn c_isalpha(c: u8) -> bool {
    c.is_ascii_alphabetic()
}

/// `iscntrl()`: 0x00-0x1F and 0x7F.
pub fn c_iscntrl(c: u8) -> bool {
    c.is_ascii_control()
}

/// `toupper()`.
pub fn c_toupper(c: u8) -> u8 {
    c.to_ascii_uppercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn space_set_matches_glibc_c_utf8() {
        let set: Vec<u8> = (0u8..=255).filter(|&c| c_isspace(c)).collect();
        assert_eq!(set, vec![9, 10, 11, 12, 13, 32]);
    }
}
