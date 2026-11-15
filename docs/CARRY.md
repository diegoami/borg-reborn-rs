# Carrying borg-rs changes across Track R

There are two pins. `pins.json` tracks moving reference material;
`borg-core/PIN` identifies the runtime core. A reference refresh never
implicitly changes the latter. Every production core refresh is its own
vendor/policy PR, targeting `feature/track-r` while R is unfinished.

## Procedure

1. Record the feature-branch head, native core pin, donor revision and
   build identities. Work on an isolated task branch or worktree. Keep the
   current reference and the source of the comparison exact.
2. Run `python3 scripts/ported-changes.py --head FULL_DONOR_SHA --json`.
   Its base defaults to `borg-core/PIN`. It lists changed native Rust
   functions and the subset named in `docs/ported-functions.json`.
   `scripts/refresh-sources.py` includes the same report. The manifest must
   expand as ports are added. Invalid revisions or missing declared native
   functions fail; they cannot silently produce an empty watch list.
3. Run `python3 scripts/borg-core-vendor.py refresh FULL_DONOR_SHA` in the
   isolated checkout. Our own files survive; declared hook files merge
   three ways. Resolve each conflict by retaining the parameter mechanism,
   native fixes and shadow hooks. Do not discard Track P's work. Review
   all changed files, imports, data and trait indices: a function-name
   watcher is not a dependency or semantic analysis.
4. Update the typed view for new native fields and update parameter
   defaults/provenance where upstream behavior changed. A source change
   may affect an unported function or may change comments only; neither
   implies a port mismatch. Conversely, a changed constant or helper can
   affect a port whose own function text did not change.
5. Build under the shared lock. Run `BORG_SHADOW=check`, then
   `harness/check-shadow.py RUN_OR_SUITE`. Read actual mismatch inputs and
   both results; re-express the changed native behavior in the port and
   update its tests. Repeat until comparisons are positive and mismatches
   are zero. Never treat an empty log without coverage as a pass.
6. Run the required native/reference and switch parity gates against the
   newly chosen exact baseline. Preserve every natural death and every
   failure. Source refreshes and diagnostic test builds are not evidence
   of improved play. Record manual work, wall time and limitations before
   the task PR; merge only into the feature branch after CI passes.

## R1.3 measured trial

The real source was borg-rs `dc790dc46e13` (v13R), after v12 `e6d13fc`.
The source report lists 25 changed Rust functions but **zero changed ported
functions**: R1's only port is `danger.wall_like`. That is a valid negative
control, not a claim that v13's danger/caution behavior is unchanged.

A full isolated re-vendor produced conflicts in three files. Manual work:

- Caution: retain named healing parameters and change six defaults to
  express upstream's fixed 20/40/60 HP cure estimates; retain parameterized
  thresholds while updating the explanatory comments.
- Danger: retain the named gravity/ice fear weights and use upstream's
  `BI_PSTUN` protection checks.
- Defence: retain the named failure adjustment while removing the branch
  upstream removed.

The resolved full core builds, the vendoring check passes, and a marked
10,000-turn test run made 291,865 helper comparisons with zero mismatches.
The original merge command took 2.50 seconds. The full trial's recorded
wall interval to evidence capture is about 403 seconds; it includes manual
inspection, compilation, execution, patch verification and intervening
bookkeeping. It is not exclusive engineering time or a steady carry-rate
estimate. A focused healing-unit build/run took 38.71 seconds on the
provenance repeat (the first exploratory pass took 58.60 seconds).

The full refresh also adds five traits absent from the current v12 view:
PSTUN and four elemental vulnerabilities. Production adoption still needs
that view extension, parameter-provenance/retired-parameter review and a
new full policy baseline. These are recorded in #264. **The production
core pin has not moved.** Full-v13 policy parity was not measured; only the
pure helper is ported in this frame.

## Positive control and re-port

Because the real commit did not touch the only ported helper, a separate
constructed change classified ordinary floor as a wall in the native
helper. This is a deliberate fault in a marked test build, never an engine
change or production policy. It generated 97,506 mismatch lines in check
and 98,360 in switch. The source watcher identified `danger.wall_like`.
After re-porting the same constructed change and updating its test
expectations, both modes had zero mismatches and byte-identical keys,
messages and states to the changed native check run.

The provenance repeats took 61.78 seconds for mismatch controls and 43.42
seconds for the re-port/build/tests/runs. Initial runs and repeat timing
limits are retained in [R1.3-CARRY-RESULTS.json](R1.3-CARRY-RESULTS.json).
Four repeat invocations were initially refused because their output
folders already existed; no games launched. They were recorded as
configuration failures and rerun into fresh directories. Old output was
not used as evidence for those failed repeats.

All trials use `test-track-r-carry` in BUILD_ID. They are diagnostic test
builds, not natural measured arms or population evidence. Exact source and
binary identities and outcomes are retained. Raw logs/builds stay ignored.
The patches below reproduce the input trees; the trial commits are local
scratch identities, not commits to adopt into the feature branch.

- [Full carry adaptation](R1.3-FULL-CARRY-ADAPTATION.patch): apply after
  `borg-core-vendor.py refresh dc790dc46e13` leaves its three conflicted
  files. It resolves them and adds the test-build label. The patch was
  checked to reproduce all five adapted files exactly.
- [Focused healing unit](R1.3-HEALING-CARRY.patch): alternative isolated
  six-default trial from the v12 frame, not applied with the full refresh.
- [Constructed native change](R1.3-POSITIVE-CONTROL.patch): alternative
  isolated positive control from the v12 frame.
- [Re-port control](R1.3-RE-PORT-CONTROL.patch): apply after the constructed
  native change, then build/run both enabled modes again.
- [Named source changes](R1.3-SOURCE-CHANGES.json): exact real donor comparison.

The runtime inputs of the delivered R1.3 task are unchanged from R1.2
(`aabe60c9cdbc`). The carry scripts, manifest, patches and report do not
adopt any of the diagnostic policy changes.
