# Opt-in recorder (#210)

angband-recorder v0.1.0 (c4986b26b9fd1c14852df4f1b7393d9030cf812a) is
vendored at vendor/angband-recorder and sealed by recorder-pin.json. A checker
outside the vendor is called by Cargo and recorder-enabled CMake. The optional
recording feature links through the existing libborg_reborn_rs.a; no second
Rust staticlib is linked. This is an owner-requested integration dependency,
not a dependency on borg-rs or a strategy change.

Configure engine with -DSUPPORT_BORG_RECORDING=ON. The normal harness explicitly
sets it OFF, including on a reused build directory. The hooks in
engine/recorder-hooks.patch apply only to the generated tree after our existing
patches. engine/angband remains byte-identical at e473720; no upstream upgrade
is part of this integration. The recorder's generic vanilla patch targets the
newer 42cd156; the consumer's adapter deliberately preserves its old engine.
Source identities include the adapter, vendor and pin. Raw runs stay outside
Git; training recordings belong to borg-cemetery.

Capture with ANGBAND_RECORD_DIR (BORG_RECORD_DIR alias accepted). Replay into a
separate output with ANGBAND_REPLAY_DIR and BORG_TOOLKIT_HEADLESS=1 for the
borgh frontend, which must recompute its activation and scripted birth. The
recorder checks the recomputed terminal/bot stream and never supplies engine
RNG or bot decisions. Use the standalone CLI verify on the two output trees.
The frontend finalizes before _exit; normal quit also has an atexit finalizer.

No strategy, default profile, measured baseline or existing death workflow is
changed. The one-file tree and in-game branch menu are future recorder work.
