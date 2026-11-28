## 2026-11-28 — R5 combined stair eligibility and route selection (#1403)

`flow.plan_combined_stairs` checks eligible stair lists, food, and light,
then replays the native rising-flight veto before choosing the route. An
unweighted down-only attempt still clears and spreads the flow when the
veto leaves no target. When a route is refused as dirty, the native
caller retries up and then down under the same combined rules. The native
executor handles route search, parked stairs, and the move.
[Owned planner](../../borg-next/src/stateful/flow.rs#L120) ·
[Native caller](../../borg-core/src/borg_flow_stairs.rs#L1203) ·
[Native executor](../../borg-core/src/borg_flow_stairs.rs#L537)

The constructed engine case covers the combined and single-kind requests,
food and light gates, rising with weights zero and five, and a blocked up
corridor that forces the clean down fallback. The zero-weight case checks
that the retry invoked all three attempts. The case passed in off, check,
and switch. The strict R5 gate now requires the new stateful entry.
[Engine controls](../../engine/tests/test_stair_handoff.py#L195) ·
[R5 shadow reader](../../harness/check-shadow.py)

The shipped smoke was built at `eeb833d8ade29c273af367594deb9d667889fa78`
and compared against native `24be98732748230062882fe2b7d9601dba5e2cc0`.
Shadow-off matched all three native starts. Check and switch each matched
off for keys, messages, state, and summary, with 163 combined-planner
comparisons and zero mismatches per mode. All three arms used the same
clean binary and shipped settings. The comparisons cover the planner's
selection and its call to the recorded rising veto.
[Smoke starts](../../harness/suites/smoke)

All ten stair tests, all 15 pinned Zoo profile-contract tests, Rust and
Python tool tests, and the receiving-source and vendor checks passed.
The 45-start Zoo comparison remains the milestone gate for the completed
R5 candidate. [Track R epic #217](https://github.com/diegoami/borg-reborn-rs/issues/217)
