# v0.1.0 verification

2026-11-14, Linux x86-64, local checks:

- Seven Rust stream tests pass; cargo fmt --check and clippy --all-targets
  --features ffi -- -D warnings pass.
- Fresh export of vanilla 42cd156 + the versioned hook patch, C Borg disabled:
  89 frames, 209 ordered inputs, eight player actions through turn 81.
  Input bytes, captured reads, messages, RNG/screen/world observations and
  final native save bytes match on a new-process replay. Changed RNG rejected.
  Exact hashes and limits: validation-v0.1.0.json.
- Raw evidence: /tmp/recorder-release-sealed (not committed).

The test frontend drives actual Angband birth and commands; it supplies no
Borg decisions or engine RNG. The gate does not qualify a GUI-human game,
a long game, another architecture, portable replay, a slim branch container,
a rolling-save schedule or the in-game menu. Those remain roadmap work.
Consumer-specific integration receipts belong in their own repositories.
