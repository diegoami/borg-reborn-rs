# Changing how the Borg plays

*Part of the [contributor guide](README.md).*

## Where the play is decided

The Borg that plays is **borg-rs's decision core**, vendored in
[`borg-core/`](../../borg-core/) at the pin in `borg-core/PIN`, with our
files and hooks around it ([docs/architecture/borg-core.md](../architecture/borg-core.md)).
A change to how the Borg plays is one of three kinds:
1. a **shipped value**, in `harness/pborg.yaml`;
2. a **new named parameter**, in `borg-core/src/params.rs`, that the core
   reads;
3. a **new behaviour** in the core, behind a switch.

Read the relevant chapter of the [Strategy Manual](../strategy/README.md)
first. Then check every number against the engine (`engine/angband/src/`,
`lib/gamedata/`) and cite it there (RULES 16).

## Settings: `harness/pborg.yaml`

This file holds the shipped settings: the C Borg's own (`borg_stop_dlevel`,
`borg_no_deeper`, ...) and every `borg_param_<name>` the core reads.
- **Comments.** Every value has a comment: what it does, what changing it
  does, and its *Impact* line (how many of six short games it changes).
- **Class values.** A class's own value is a key with `__<class>` appended,
  for example `borg_param_<name>__mage`.
- **The tools.** `harness/pborg-settings.py` reads and edits the file
  without reformatting it, and `harness/pborg-config.py` converts older
  settings.

The full list of parameters, with default, unit, range, the code that reads
each one and the evidence for it, is
[docs/CALIBRATION.md](../CALIBRATION.md).

## Adding a parameter

1. Add the parameter to `borg-core/src/params.rs`, with its neutral default
   (the value that plays as before), its bounds and its citation: the file
   and line that reads it.
2. Read it at that line in the core.
3. Add its `borg_param_<name>` line to `harness/pborg.yaml`, with a comment.
   A fix for an observed defect ships switched on (RULES 28).
4. Document it in [docs/CALIBRATION.md](../CALIBRATION.md).
5. Check the citations. `--fix` updates the line numbers after an edit:

   ```bash
   python3 scripts/check-param-citations.py
   ```

6. If the shipped play changed, re-record `smoke-borg-rs`
   ([building-and-testing.md](building-and-testing.md#the-smoke-suite-and-its-references)).

## The vendored core

- **What may change.** `borg-core/` is borg-rs at its pin, plus our
  declared files (`OURS`) and our hooks, each marked
  `borg-reborn-rs (#issue)` in the code.
- **Checking it.** With `vendor/borg-rs` checked out, this shows that
  nothing else changed:

  ```bash
  python3 scripts/borg-core-vendor.py check
  ```

- **Formatting.** The vendored files keep borg-rs's formatting.
- **Other sessions.** Sessions on other tracks share `borg-core/`, so keep a
  change to the files its task names.

## The engine

The engine is never changed (RULES 9). If the Borg needs a new hook, it is a
patch in `engine/patches/`, guarded so that with the guards off the code is
upstream's ([docs/ENGINE.md](../ENGINE.md)).
