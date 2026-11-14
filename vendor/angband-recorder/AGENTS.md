# Recorder ownership

This repository owns the Rust recorder, the versioned format/tools and the
small C host interface. Read README.md, spec/streams-v1.md, CREDITS.md and
 docs/OWNER-HANDOVER.md before changing it. The handover is the owner's design;
its later features are a roadmap, not evidence that they are implemented.

Borg policy, consumer fixes, parity harnesses, death workflows and named
launchers stay with consumers. Do not edit a consumer or change its pin unless
the owner requests that integration. Never post to angband/angband.

Use exact immutable engine bases. Record build identities and preserve donor
licences and provenance. Keep recording inputs separate from observations;
replay supplies external inputs and checks recomputed bot keys, never RNG or
bot decisions. Taking a save may change a process: branch by a fresh process
from the save, never by pretending it restores the live state.

Run cargo fmt --check, cargo test --locked, cargo clippy --locked --all-targets
--features ffi -- -D warnings and the vanilla round trip before a release.
Builds/game suites take /tmp/angband-validation.lock. Retain raw proof outside
Git and report failures and untested profiles. Never put saves or executable
bundles in this repository. Curated recordings belong in borg-cemetery.

Consumers vendor tagged releases and verify every byte against an external
pin manifest before building. A Rust consumer links this crate through its
existing staticlib, never a second Rust runtime. Recorder-off oracle builds
stay off. Tags are immutable; a correction requires a new release.
