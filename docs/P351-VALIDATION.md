# Current-base recorder v0.2.0 adoption (#351)

Adopt tag `v0.2.0`, commit `ee8b133ad22f66e7dad6e1b63a2d3e40b9a41ebd`.
All 41 release files are verbatim and pin-verified. The release Cargo package
still declares 0.1.0; tag/commit, schema v2 and the consumer build label
identify this release. No vendor source or dependency lock is rewritten.
The eight-file adapter preserves harness hooks and adds early initialization,
newer/directory ABI calls and live existence checks for this session's saves.
Recording remains off in the normal harness. Upstream stays `42cd156ac`.

## Source epochs and the active review gate

Track-p advanced during the first comparison. The owner-directed #373/#379
revert and #358/#359 fixes are preserved through base
`79ec0591248db68c70fdc905ebe09549aabc60fa`; relative to that base this task
changes no Borg policy source. Only handover text conflicted in the merge.
#395 reviews that exact base; this task must not move its reviewed head.

The completed #355-era first comparison remains historical proof under
P351-PRE-REVERT-VALIDATION.md/.json and its dossier map. Actual old baseline,
candidate and recorder images are frozen and hash-mapped, as are the old
CLI readers. Recordings, original metadata and source-qualified packets are
not overwritten or relabelled. Only completed compiler caches were cleaned.

This report is the refreshed comparison against the restored policy, not a
claim that #355's results describe the current base. The incident regression
now reproduces all four expected pre-#355 wild-cat deaths, as the owner-
directed revert requires; the earlier rescue result remains in its old epoch.
Native unit fixture temporary directories are not a population measurement.

## Clean measured builds and current play

Baseline revision: `79ec0591248db68c70fdc905ebe09549aabc60fa`.
Source: `87717140e88805c99b2b8f5d036036cf21eb9b8e953b8cce25d07465a2c2a9a9`.
Binary: `e9b91d1ec5238712ddfec407e2cfaab578feba5ba6534a374b42ebd9d46fd0d8`.
Candidate revision: `60a506b689ffb09a3b241ac97059adb34931898b`.
Source: `7d2af92011c8d5fa7d73cb2df1e5460048f2dd1f0987cc0143df26406a5a94f7`.
Binary: `80b500e55e7a890b6b0eab55d5b0ac447f699bb30a6d5ee6960dc8a98d4e5b75`.
Both measured BUILD_IDs report git_dirty=no and inputs_dirty=no. Subsequent
document/evidence edits do not change game inputs.

All 45 starts (nine classes, five seeds, 100,000 turns, explicit menu flags)
match byte for byte in keys/messages/states and effective borg.txt. All
deterministic summary fields match with wall time excluded. There are
242,641 consecutive classified candidate trace rows. Original-core smoke
matches all three references and its seed 1020 death control completes.
P351-VALIDATION.json records exact commands/environments, suite SHA-256,
per-side/per-case log/config/parameter/trace hashes and original summaries.

## Native recorder and required verification

The recorder-enabled current executable passes fresh-game and captured-load
record/re-execution pairs. Native final-save bytes match for both pairs;
forked checkpoint bytes match, bot actions are captured and all four output
identities use schema v2. The loaded case preserves its initial save, moves
the recorded final save out of the working path, replays sealed starting
bytes despite that path being absent, then saves using live session state.
A different replay timezone is restored to captured context. The pinned CLI
verifies both produced pairs; CLI verification does not itself rerun a game.
Actual image/completion/environment/save hashes are retained. Direct CMake
recorder output has no fabricated harness BUILD_ID; it was rebuilt from the
committed current inputs. Raw proof is /workspace/work/p351-current-recorder.

Formatting, Clippy, 156 + 4 + 5 workspace checks and 328 core tests pass
(four existing ignored). All 69 native tests pass with one local bindgen
skip; pinned zoo, upstream and incident assets are required in this run.
All 91 tool tests and 19 script tests pass. Vendor and recorder-pin audits
pass. The unchanged tagged recorder also passed 18 standalone Rust checks
(two CLI, 16 stream tests). Required CI covers local binding regeneration.
The native checks were refreshed after synchronization, not borrowed from
the earlier policy epoch.

## Retained evidence and limits

Add 26 current-source provenance packages: 12 baseline and 14 candidate.
Every original record and payload byte was checked after qualified import;
P351-DOSSIER-MAP.json records actual builds and source paths. Both registries
reconcile with none missing or unrecorded. All 2,632 archive cases verify.
Together with 28 preserved first-epoch packages, the task adds 54 packages.
Distinct-start/attempt aggregates rise from 2,562/2,565 to 2,603/2,606 (+41),
not +54: all 12 refreshed baseline results and one old baseline result were
already known. These are source-qualified registry counts, not independent
population lives. No historical packet or accepted inventory is rewritten.

V1 and v2 recordings are incompatible both ways; existing sessions need
their exact matching executable and format. Engine save files still load;
the old captured incident saves exercise the restored policy contract.
No v1 recording conversion, GUI/save-menu drive, hot activation or branch UI
is claimed. New directory/newer wrappers compile; these bounded borgh
streams do not emit those event families. The pinned library tests cover
their typed stream/replay rules.

No canonical cemetery write, Zoo repin, population experiment or Borg
threshold/default change is part of this adoption. #393 stays separate
while #395's review is active. Required task CI and that gate's disposition
precede track-p integration; another owner-selected milestone review then
gates this recorder promotion to main.
