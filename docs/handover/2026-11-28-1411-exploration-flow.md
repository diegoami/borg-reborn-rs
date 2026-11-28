## 2026-11-28 — R5 committed exploration flow priority (#1411)

`flow.continue_exploration` tries MISC, DARK, XTRA, BORE, and VAULT in
order and returns the first goal that acts. The tape records and replays
each native continuation, including any changes made on a false return.
The dungeon driver emits the existing choice label at a distinct source
site for each goal. Its existing Recall wait runs on fallthrough.
[Native caller](../../borg-core/src/borg_think_dungeon.rs#L1343) ·
[Native continuation](../../borg-core/src/borg_think_dungeon.rs#L1739) ·
[Owned priority](../../borg-next/src/stateful/flow.rs#L161)

The constructed engine case exercises all five goals, two unmatched goal
states, and a blocked route that clears its goal with Recall pending. It
passed in off, check, and switch. The strict R5 gate now requires the
continuation entry point.
[Engine controls](../../engine/tests/test_stair_handoff.py#L186) ·
[R5 coverage reader](../../harness/check-shadow.py)

The shipped smoke at `5d72cbf34f1a3c26d24e35c836c0588cdbb18f6d` matched
native `24be98732748230062882fe2b7d9601dba5e2cc0` in off on all three
starts. Check and switch matched off for keys, messages, state, summaries,
and core traces. Each shadow mode recorded 3,177 continuation comparisons
and zero mismatches. The 4,255 core trace rows also matched the #1405
smoke after mapping the per-goal source lines to their new locations.
[Smoke starts](../../harness/suites/smoke) ·
[Parent PR #1405](https://github.com/diegoami/borg-reborn-rs/pull/1405)

All 11 stair and flow tests, all 15 pinned Zoo profile-contract tests,
Rust and Python checks, and the historical R4.5 smoke verifier passed
locally.

The owner decides whether to merge this slice after review and CI, once
#1405 has landed. The full 45-start Zoo comparison remains the R5 milestone
gate. [Track R epic #217](https://github.com/diegoami/borg-reborn-rs/issues/217)
