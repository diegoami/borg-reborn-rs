# Extraction comparison, 2026-11-14

The source was inspected without switching or resetting any active checkout.

| Source | Findings | Decision |
| --- | --- | --- |
| borg-rs `2411e52f411d6984037a85cdb237bd54dd5b81cb` | Recorder is `angband/src/native-replay.c`; CMake option off by default; hooks in nine engine files including the test frontend. Captures PID, messages, mapped state, 15 fields per poll; wraps time across the Rust staticlib; no C Borg globals. Toolkit and parity harness are separate. | Base for the engine interface and observations. Rewrite stream ownership/capture/replay in Rust. |
| better-borg-c `cfcb6f13f` | `tools/measure-native.py:recording_profile` instruments disposable source exports; the primary game tree has no recorder hooks. `native-observer.c` invokes recorder frames. Recorder depends on C Borg globals and decision counts; 14 fields per poll. Checks action divergence immediately and checks leftover actions. | Retain fail-fast action verification in the Rust core; preserve engine-only observations. Remove the disposable recorder path from the C consumer, preserving historical bundles. |
| borg-reborn-rs `e9a6a521f3ac2fc631aed2c2e3d6c595dde9ab71` | C engine at `e473720` is pristine. Own patch series/build/frontend; `libborg_reborn_rs.a` supplies `do_cmd_borg`; input is through `inkey_hack`. Copied recorder tools were removed in TE.6. | Depend on the crate through the existing Rust staticlib, add opt-in hooks to the generated engine tree; no second Rust runtime. |

The patch target is vanilla Angband 4.2.6 at exact commit
`42cd156ac9f95ddf4311e88dd7ad462292d6d1d6`. It includes the official import
used by borg-rs v12R, without either consumer's Borg or engine fixes. The older
reborn engine needs a separately checked hook adapter against `e473720`, not
an implicit upstream upgrade.

The Rust prototype adds ordered call-kind checks and captures file-existence
queries. A vanilla startup replay exposed this missing external dependency:
the first run creates a save that changes the next process's file-existence
answers. Raw byte capture alone did not replay that startup.

New recordings have a versioned ordered trace, plus compatibility streams for
existing consumers. Old bundles keep their frozen executables and adapters;
they are not rewritten or relabelled as the new format.
