## 2026-11-28 — R5 down-stair eligibility and route selection (#1376)

`flow.plan_down_stair` checks for a known down stair, then calls the native
rising-flight veto and the native shared descent refusal in that order.
These dependencies can update danger, preparedness, and sale state. As
specified in #1376, the tape records their inputs and resulting state and
replays both effects in the comparing arm. The returned plan carries the
resolved weight. The native executor supplies the caller's depth, sneak,
and ranged reach.
[Owned planner](../../borg-next/src/stateful/flow.rs#L96) ·
[Native executor](../../borg-core/src/borg_flow_stairs.rs#L903) ·
[Native rising veto](../../borg-core/src/borg_caution.rs#L4558) ·
[Shared refusal](../../borg-core/src/borg_flow_stairs.rs#L853)

The constructed controls passed in `off`, `check`, and `switch` at
weights zero and five. They cover absent stairs, the rising hold, an
ordinary route while rising, Recall without an up stair, the light refusal,
refusal with no food and `brave = false`, the matching brave route, and the
one- versus two-move budget for weighted flight. The test process
exits before a queued movement can reach the game. All nine stair tests
and all 15 pinned Zoo profile-contract tests passed locally.
[Engine controls](../../engine/tests/test_stair_handoff.py#L180)

The shipped smoke ran at `bc8094ad539976f755ba881d21e38c33356bb50a`
against native `24be98732748230062882fe2b7d9601dba5e2cc0`. Shadow-off
matched native 3/3. Check and switch each matched off 3/3 for keys,
messages, state and summary. Each shadow mode recorded 272 comparisons of
stair presence, call order and route selection, with zero mismatches. Those
comparisons replay the results and state changes of both native checks. All
three arms used the same verified clean binary and shipped settings.
[Smoke starts](../../harness/suites/smoke) ·
[Shipped settings](../../harness/borg.txt)

The strict R5 gate now requires the down-stair planner. Rust formatting,
Clippy, workspace and release core tests, receiving source tests, Python
tool tests, and vendor checks passed. The retained R4.5 smoke passed its
historical verifier.
[R5 shadow reader](../../harness/check-shadow.py)

The owner decides whether to merge this slice after review and CI. The
full 45-start Zoo comparison is the milestone gate, with the completed R5
candidate and fixed native reference pinned at dispatch. The owner starts
that comparison. Remaining stair policy and dungeon modes are tracked in
the R5 epic.
[Track R epic #217](https://github.com/diegoami/borg-reborn-rs/issues/217)
