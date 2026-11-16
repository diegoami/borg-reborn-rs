# Review: generated-depth synthetic starts (#371, PR #386)

Review the capability in PR #386, stacked on dedicated pin PR #377 and
cemetery PR #6. Runtime implementation is pinned to
`b336a147b204daf03563dbb317326607838a239e`; base main is
`92404c7e72de4c438ecf1028b17d38b57833d5a6`. The PR body pins the final
publication head, which also contains control dossiers and this review packet.
Those later files do not change the runtime source digest
`0c8eed7ed9dbe4a5767d905663f3c08f4deb0387f0acba5a56a23a547085efaf`.
Cemetery dependency: `0216eb3346887ddc1087b16051bb14fb22b71422`.

The owner chooses and runs the review. No reviewer has been selected or
launched, no PR merged and no milestone accepted by this session.
A concurrent independent implementation is already in PR #378; its branch
was preserved. This review packet covers #386 only. Do not merge both.

## Reviewer task

Review the diff from the stack base `e6da42dc55f9b80b951a2a4d0d7a471af3008476`
to the exact publication head given in PR #386. Check cemetery PR #6 at its
pin as part of the natural-registry boundary. Report blockers and nonblocking
findings with exact file/line references and concrete failure cases.

Focus on these contracts:

- Generation and verification happen before activating either core. The new
  guarded wrapper uses the engine's own level-transition housekeeping and
  generator; the pristine engine tree remains unchanged. Inspect synchronous
  entry from the initial command-input boundary and startup message flushing.
- `one-way-descent-v1` permits down stairs/trapdoors/downward level travel;
  forced descent blocks ascent, and no-recall blocks Recall before victory.
  The native winner exception remains. This is not single-level confinement.
- Only the normal birth kit exists. No arbitrary wizard equipment or hidden
  observations enter policy. The vendored core's allowance is limited to the
  two scenario birth options; ordinary initialization checks still apply.
- Reject invalid/duplicate depth/build options and every save/load conflict,
  including raw frontend invocation. Refuse requested/actual start mismatch
  and synthetic saved-character continuation. A successful process with no
  actual play is insufficient; the tests require initial `dN/N` and keys.
- Both summary and config identify synthetic starts, including raw options.
  The harness skips export. Shared cemetery export independently rejects
  summary/config/flag/option-pair evidence before registry access; reconciliation
  uses the same predicate. Never reclassify a synthetic death as natural.
- Ordinary keys/messages/states stay unchanged. A historical summary without
  `synthetic` compares as ordinary; a true synthetic marker is never ignored.
- Game/core RNG consumption is reproducible for a fixed seed, depth,
  race/class, birth build, profile, data and policy settings. Inspect whether
  startup lifecycle changes create any unrecorded input or observation leak.

## Reproduction

Use a fresh checkout of the publication head and the declared cemetery pin
(`.github/actions/cemetery-tools` documents checkout/API discovery). Install
Rust, CMake, a C compiler and `bc`. Take `/tmp/angband-validation.lock` for
engine builds/runs, as CI does.

```sh
cargo fmt --check -p borg-reborn-rs
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --exclude borg-core
cargo test --release -p borg-core
python3 scripts/borg-core-vendor.py check
flock /tmp/angband-validation.lock harness/build.sh
flock /tmp/angband-validation.lock python3 -m unittest discover -s engine/tests
python3 -m unittest discover -s tools/tests
python3 -m unittest discover -s scripts/tests
python3 tools/import-deaths.py --check
```

Use temporary natural registries for validation. Run both ordinary and exact
smoke suites and compare to their checked-in references without overrides.
The engine CI job also retains a `synthetic-start-smoke` artifact with real
depth-10/20 logs. Review the current publication head's CI, not an earlier run.
The first CI run correctly failed until the new source's natural control
dossiers were retained; no reconciliation assertion was weakened.

## Evidence and limits

[371-SYNTHETIC-EVIDENCE.json](371-SYNTHETIC-EVIDENCE.json) records the clean
runtime build identity, starting states, summaries, configs, stream hashes,
repeat matches, direct-export rejection and local checks. Raw runs remain
outside Git under `harness/out/synthetic371-*`; CI retains its synthetic runs.
The three new qualified dossiers are ordinary validation deaths required by
the repository's source-specific baseline, not synthetic scenario outcomes.

Depth 10 and 20 synthetic deaths are functional smoke evidence, not a survival
comparison. Synchronous engine initialization intentionally establishes its
own reproducible start and need not reproduce the spike's key stream.
Run-derived build records/hash application (#382), optional null-move threat
facts (#383), Cemetery suite generation/classification and population claims
are outside this increment. Local engine tests skipped bindings regeneration
and the external recorded incident fixture; CI supplies those dependencies.
