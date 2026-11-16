# R2 danger slice validation (#281)

> Historical R dossier numbers in this report are scoped to registry commit
> `fbd0f691de655c6db61565b1e3d2b5aea92184cd`, not the restored legacy slots.
> See [R334-RECOVERY.md](R334-RECOVERY.md) and R334-DOSSIER-MAP.json for
> full qualified paths. Raw measurements/build identities remain historical.


Danger is ported; caution, escape and recovery remain #209. R2 is not complete.
The runtime core pin stays borg-rs v12 e6d13fc. Production runtime input hash
is `3ee2871785624dcb2b28f363f522d8942cd42946af97be817e87c660afdec851`.
The measured build predates the source commit and records dirty inputs;
its exact hashes match the committed runtime files. Source and binary
identities and every run's log hashes are in R2-DANGER-COMPARISON.json.

## Native/reference and switch gates

Off/check/switch smoke passes 3/3 in keys, messages and state. Full check
and full switch each pass **45/45** against the independent pinned v12
build from R1.2. Both full modes have zero mismatches across **66,900,787
whole-danger and 111,865 standalone-monster comparisons per mode**.
Check versus switch is also 45/45 identical under the standard same-build
diff. Shared v12 summary differences remain exactly the three existing
frontend telemetry fields and wall time; the standard cross-repository
summary diff remains 0/45, as in R1. No survival improvement is claimed.

All 16 full-suite natural deaths reconcile: eight check deaths are
D2377–D2384, and switch repeats those cases rather than creating independent
ones. First CI original-core controls are retained separately as
D2387/D2388, with exact artifact/run provenance. All remain unassessed.

## Deliberate disagreement and re-port

The isolated test-track-r-danger build adds one to each native danger
entry point's scalar, without changing the engine. It produces **79,314
mismatches in check and 92,252 in switch**. Check retains the changed native
results; switch retains the unchanged port results. Re-expressing the same
offset in the port drives both modes to zero mismatches. Both re-port games
match the changed native check game in keys, messages and states. Exact
first mismatches, statistics, build identities, timings and reapplicable
fault/re-port patches are R2-DANGER-CONTROLS.json and the two patch files.
These are diagnostic builds, not production policy or natural arms.

The expanded source watcher identifies three changed ported danger functions
at moving borg-rs reference 727ccff: aggregation, physical and spell
estimates. R2-DANGER-SOURCE-CHANGES.json retains that positive warning.
It does not adopt v13 or validate shared dependencies. #264 gates adoption.

## Checks and limits

Formatting, Clippy, 169 workspace tests, 138 core tests (one existing
ignored), 20 source-tool tests, 73 tool tests and vendoring check pass.
Required engine and recorder CI remains the final task merge gate.
The port is 2,532 source lines including comments/license; the whole
6,500-line R2 estimate is re-planned only when #209 is complete.

Shared projection, spell-legality, blow classification and engine facts
are outside the port. Agreeing results do not independently validate them.
No population experiment or sealed suite was opened.

Two operational failures count: the first exploratory build refused
BUILD_ID because sources changed while it compiled (no game launched),
and the first off smoke played three capped survivors but suite aggregation
failed because bc was absent from PATH. Fresh-directory smoke repeats with
the workspace tool path passed. No failure was relabelled as a death.
