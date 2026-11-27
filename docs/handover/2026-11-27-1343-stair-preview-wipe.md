## 2026-11-27 — R5 stair-preview danger-cache wipe (#1343)

The constructed engine test calls `stair_steps_around` on copied Borg
state with a cached danger veto on the only route to a down staircase.
With no tracked stairs, the preview leaves a pending wipe available.
With the stair tracked, the wipe lets the preview cross the veto. The
preview restores the flow workspace and cached classifications and
consumes the wipe flag. The next native flow clear resets the queue
and costs, and the restored veto blocks the next preview. The fixture
also checks the caller's desperation flag and the empty command queue.
[Engine fixture](../../borg-core/src/borg_flow_stairs.rs#L1436) ·
[Harness test](../../engine/tests/test_stair_handoff.py#L163)

The fixture uses the existing `test-stair-handoff` build and exits inside
the test-only process. All four stair-handoff fixture methods passed.
In a disposable build, restoring the pending flag after the preview
failed the wipe-consumption assertion. Rust formatting, Clippy, workspace
and release core tests passed. The 91 receiving source-tool tests had
one skip, and the 158 Python tool tests passed.

The stair-policy port is tracked by the R5 epic and can use this check
when it moves stair decisions behind the shadow boundary.
[Flow port #1300](https://github.com/diegoami/borg-reborn-rs/pull/1300) ·
[Track R epic #217](https://github.com/diegoami/borg-reborn-rs/issues/217)

The owner decides whether to merge the fixture after review and CI.
