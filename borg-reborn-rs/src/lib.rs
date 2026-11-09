//! The Angband Borg reborn.
//!
//! An automatic player for Angband 4.2.6, linked into the engine behind the
//! `inkey_hack` seam as borg-rs is. Nothing plays yet: milestone 0 is the
//! foundation. The layers this crate will hold are described in
//! `docs/ARCHITECTURE.md`:
//!
//! * substrate: engine boundary, perception, world model, knowledge,
//!   assessment (danger and risk), executor, decision trace, parameters;
//! * strategies: the family of decision layers behind one trait.
//!
//! The engine boundary is `engine`; it links behind the seam package
//! `borg-rs` (see `borg-rs/Cargo.toml`). The Borg itself arrives in T1.5.

#![deny(unsafe_code)]

pub mod engine;

/// The name the Borg reports in run records and notes.
pub const BORG_NAME: &str = "borg-reborn-rs";

/// The upstream Angband commit every Borg in the family plays.
pub const ENGINE_PIN: &str = "e473720ed88a0f0c7ca66815cef3000eb4a38bc3";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn engine_pin_is_a_full_commit() {
        assert_eq!(ENGINE_PIN.len(), 40);
        assert!(ENGINE_PIN.bytes().all(|b| b.is_ascii_hexdigit()));
    }
}
