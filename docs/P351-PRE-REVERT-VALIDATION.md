# Historical pre-revert comparison

This receipt preserves the completed first comparison from before the
concurrent owner-directed #373/#379 revert. It is not the final current-base
validation. Actual binaries are frozen and mapped in
P351-PRE-REVERT-FROZEN-BUILDS.json; original metadata and recordings remain.

# Recorder v0.2.0 consumer adoption (#351)

Adopt exact release tag `v0.2.0`, commit
`ee8b133ad22f66e7dad6e1b63a2d3e40b9a41ebd`. All 41 vendored files remain
verbatim and the external pin checker passes. The release's Cargo package
version remains 0.1.0; its tag, commit, schema v2 and consumer build label
identify this adoption. No dependency lock or vendor source is rewritten.

Regenerate the eight-file consumer hook adapter from our actual generated
engine tree, retaining existing harness hooks. Add early initialization at
main entry, file-newer and directory open/read ABI calls, and live save-path
stat handling. The latter avoids treating the replay session's own save as
an external captured existence result. The normal harness explicitly keeps
recording off. Pristine upstream stays `42cd156ac`; no Borg policy changes.

## Measured builds and recorder-off play

Baseline: clean `cf687bb24de8e0715854ca1b7a0f827b7e71be82`, source
`0f2e003f988fba56929f259f6098d356ec12c0ec024419750937a13a7dce5d58`,
binary `50f10942b807d7d6f5762e475db086f96cf5bce44ba16f279bf7a78dc323f0a4`.
Candidate: clean `c310d01859904895adaafda39f81535fe8c26ee8`, source
`b2532940e9de215ed159991db3219d30d9736716bd661209fc8fde1865d303b3`,
binary `95f4d61e223a91215585efbbfded9b98cb91052c59aa2fdce92c40da849db82b`.
Both BUILD_ID receipts report git_dirty=no and inputs_dirty=no. The baseline
includes main's accepted #355 safety behavior; this is a before/after
consumer-adoption comparison, not a claim of fresh M13 donor parity.

All 45 cases (nine classes, five starts, 100,000 turns, explicit menu flags)
match byte for byte in keys, messages and states. Deterministic summaries
match with wall time excluded. Effective borg.txt files match, and the
candidate has 241,403 contiguous classified trace rows. Original smoke
matches all three references; the seed 1020 death control completes.
P351-PRE-REVERT-VALIDATION.json records exact suite hashes, command/environment receipts,
per-side/per-case log/config/parameter/trace hashes and original summaries.

## Native recorder and required checks

The v2 recorder-enabled executable passes fresh-game capture/re-execution
and loaded-capture re-execution. Final native saves are byte-identical for
both pairs. Forked checkpoint saves match; captured bot actions and v2
completion receipts exist. The loaded test preserves the starting save,
moves the recorded final save out of the working path, then successfully
replays from sealed starting bytes and saves despite the absent live path.
Replay starts with a different ambient timezone; the recorder restores the
captured environment. The release CLI verifies both produced stream pairs;
CLI verification itself does not re-execute the engine.

The CMake recorder proof was built in the precommit working tree with the
same pinned runtime inputs as the committed adoption. Its actual executable
hash and v2 recording/completion/environment receipts are retained in the
JSON; no clean Git BUILD_ID is invented for that CMake artifact. Raw native
proof is at /workspace/work/p351-recorder-loaded. The separate recorder-off
candidate above has its actual clean build receipt.

Formatting, Clippy and required Rust tests pass: 156 + 4 + 5 workspace tests
and 328 core tests (four existing ignored). The standalone pinned recorder
passes 18 Rust checks (two CLI and 16 stream tests). All 88 tool and 19
script tests pass; vendor and recorder-pin audits pass. The local engine
suite ran 69 tests with two skips: missing bindgen and the optional incident
archive. Fetching the immutable incident archive by its recorded hash and
rerunning that test passes all four wild-cat save positions; only local
binding regeneration remains untested. Required CI installs the tool and
exercises that path separately.

No validation failure or weaker game assertion was needed for this port.
Completed Cargo caches were cleaned to make room; source, executables,
raw runs, old sessions and evidence were preserved. Initial fresh-game-only
native proof remains separately retained at p351-recorder-initial; the
extended loaded-save check passes in a fresh output directory.

## Evidence and limits

Retain 28 new provenance packages: 13 baseline and 15 candidate natural
deaths. Original record bytes and every payload byte were compared after
qualification. P351-PRE-REVERT-DOSSIER-MAP.json records both lineages and actual builds.
Both registries reconcile with none missing or unrecorded. All 2,606 archive
packages verify. Distinct-start/attempt aggregates change from 2,562/2,565
to 2,589/2,592 (+27): one baseline result was already retained. Package
counts are not independent lives; no historical package is removed.

Schema v2 and v1 recordings are incompatible in both directions. Existing
v1 sessions are untouched and require their matching older executable and
format. Game saves remain engine saves; the existing incident-save control
passes. No v1 recording migration, full GUI/save-menu drive, hot activation
or in-game branch UI is claimed. The new directory/newer wrappers compile;
these bounded borgh streams do not contain those event families. The pinned
library tests cover their typed streams and replay rules.

No canonical cemetery write, Zoo repin, population experiment, runtime
threshold/default change or survival improvement is claimed. Task CI gates
integration into track-p; a separate owner-selected milestone review gates
promotion to main. Later report/dossier edits do not change game inputs.
