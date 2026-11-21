# Prompt: reproduce the recording, replay and death-analysis toolkit in borg-rs

Paste the following into a borg-rs session. This is a tooling handoff, not an
instruction to import C gameplay changes or apply better-borg-c's workflow rules.

---

Implement the equivalent local recording, replay, manual takeover, reporting and
death-analysis workflow for borg-rs. The owner wants the complete capabilities
listed below, including both deaths and living limited runs. Finish the usable
workflow and its documentation; distinguish any remaining blockers explicitly.

Before doing work, read **borg-rs's own** instructions, rules, plan/session and
latest checkpoint as applicable. Inspect its live ownership/issues/PRs and actual
engine/Rust integration. Follow that repository's process and existing owner
authorization. This handoff does not prescribe branches, labels, patch numbers,
merge policy, gate names or outside-repository writes. If its rules prevent a
step, describe that conflict rather than working around it. Do not modify
better-borg-c, the source tracker, historical recordings or another session's
work. No gameplay strategy, new C carry, release, scheduled cleanup or unattended
pipeline is requested. An inventory of the target's existing capabilities comes
first; reuse them where correct instead of duplicating them.

## Reference source and documentation

Source repository: https://github.com/diegoami/better-borg-c

Freeze the implementation reference at:
`a4084032485a7c7f29153d0b37ce63dede3e4d5a`
https://github.com/diegoami/better-borg-c/tree/a4084032485a7c7f29153d0b37ce63dede3e4d5a

The accompanying documentation PR adds the following guide without changing that
reference implementation. Read the guide and then use the pinned files for exact
formats/semantics, not a moving main branch:

- `docs/measurement/REPLAY-TOOLKIT.md`: user workflow, commands, storage and implementation map.
- `docs/measurement/RECORD-REPLAY.md`: detailed native contracts, failed prototypes, real qualification and limits.
- `docs/measurement/DEATH-QUEUE.md`: collection, queue, reports and output formatting.
- `docs/deaths/FEEDBACK.md`, `REVIEW-TEMPLATE.md`, `owner-reviews.json` and `window-profile-v2.json`.
- `docs/measurement/DEATH-ANALYSIS-PLAN.md` and `DEATH-IMPROVEMENT-WORKFLOW.md`: diagnosis versus a later prospective implementation experiment.

If these new guide files are absent at the pinned implementation commit, read
them from the merged documentation change and record its exact documentation
commit separately. Do not pretend the guides existed at the implementation pin.
Fetch/read the reference in a way permitted by borg-rs; this prompt makes no
assumption about a neighboring local checkout. Raw examples in ignored C
artifacts are not distributed with Git. Ask for a specific missing bundle only
when needed; build small target-native qualification fixtures rather than
claiming inaccessible evidence was inspected. Retain source licensing and credit.

## Required user capabilities

1. Named visible play: resume/create a named character, manual Borg activation,
   five rolling pre-launch save backups, explicit backup restoration preserving
   the current save, same-name lock and concurrent different names. Record every
   session with a printed immutable evidence path. Readable X11 font selection
   where X11 is the supported frontend; respect explicit overrides and test actual
   loading before expensive reconstruction. Use the target's available frontend
   if its repository chooses another supported equivalent.
2. Named headless continuation: load the same save used by visible play, activate
   the Rust Borg, advance **additional** engine ticks to the next valid command
   boundary, publish the living native save safely. Fresh characters use native
   random race/class and a recorded random or explicit seed. Loaded characters
   keep their identity/RNG. Failure must not overwrite the original named save.
3. Bounded batch collection: `--games`, `--seed` or distinct `--seeds`, absolute
   new-game `--turns`, per-character `--timeout`, unchanged-clock `--stall-timeout`,
   committed `--ref`, compatible build reuse/offer, `--yes`/`--no-build`/`--check-build`.
   Freeze seeds/limits/options before outcomes. Default normal Borg, random native
   race/class; no new munchkin games unless explicitly enabled. Continue after
   **every** successfully exported death and ordinary time/turn limits; stop on
   operational/native/recording/export errors, retaining all prior evidence.
   Ten requested attempts are not a promise of ten deaths. Every death has a
   separate registered dossier; every attempt has report/watch/manual commands.
4. Reports: character/current/highest level, max depth, recorded this-life kills,
   XP/gold, engine ticks, decisions, rest counters with units, wall time, source,
   engine/tool/config identity, seed and actual outcome. No tactical situation
   in a living short report. Add the native character sheet from an isolated
   frozen-engine load/format operation, validating no gameplay/time advance.
   Never resurrect a dead save for formatting. Missing counters are unavailable,
   not invented zero. Existing-collection report backfill and command reprint
   must not start new games. Print a collection comparison and final death list.
5. Original-start strict replay plus terminal viewer: verify the original frozen
   engine/Rust build, then inspect the **original** movie with forward/back/seek,
   autoplay, absolute-tick selection and dump. `--verify-only` writes a certificate;
   `--view-only` runs/builds no engine and makes no rerun claim. Allow original-frame
   inspection of finalized incomplete timeout movies without calling them complete.
6. Player-known inventory, equipment/quiver, learned spells **and prayers**, native
   description/slot/weight/fuel/charge/inscription and current casting availability.
   Unknown appearances stay unknown. Label actual-kind/unseen-monster diagnostics
   separately. Observers must not change knowledge, everseen flags, energy or RNG.
   No full hidden-state diagnostic may become an input to the Rust Borg.
7. Log panes: Borg actions/notes, visible message history and native fighting log.
   Native hits/misses/shots/spells/consumables before message coalescing; actual
   player HP loss/source; actual speed/energy gain/move budget. Count a hit message
   and its damage row as one event. Do not infer exact hits from doubled Borg
   notes or coalesced top lines. Keep frame/phase cutoffs so later same-tick events
   cannot appear early. Engine ticks, player turns, decisions, frames and keys
   must be explained separately.
8. Human takeover: choose first living command at/after a numeric tick; `latest`
   chooses the first command at the last living-command tick. Reconstruct and
   strictly verify the selected prefix from its original start. Create a native
   save, then load isolated manual play with Rust Borg **off**, original gameplay
   source pinned. Record parent/actual boundary/build/save identity and every
   manual session. Support prepare-only, pinned-branch resume, first-use separate
   historical-helper offer and display/font failure retry without rerunning the
   prefix. Finalized consistent timeout movies may qualify a prefix; crashes,
   active recordings and divergent prefixes are refused. Do not claim private
   Rust Borg memory/stack is restored by a native save. Manual alternatives are
   excluded from independent Borg survival/death counts.
9. Next-owner-unreviewed death queue: list missing/legacy/corrupt cases honestly,
   explicit selection and reversible reviewed/unreviewed marking, oldest eligible
   qualified encounter family, JSON listing. Quitting a viewer, saving a manual
   branch or agent assessment does not mark the owner's review complete. Technical
   issue closure is separate. Share local receipts across task worktrees if used.
10. Death curation/analysis: actual source/build/config/binary/tool/seed identity,
    native logs/snapshots, original pre-death windows5/12/25/40/60/82/105ticks where
    available, assessment/feedback/actions, and aggregate without counting reruns
    as new lives. Brainstorm alternatives and competing explanations, including
    attempted walking escape after consumables expire, actual knowledge/resources,
    costs, confidence and next evidence. Link focused investigations using the
    target's own issue-routing rules. Keep policy suggestions and causal bugs
    distinct; diagnosis or one human rescue does not establish AI improvement.
11. Readable CLI: separated build/plan/game/batch sections, friendly outcomes,
    elapsed/progress and labeled report paths, copyable standalone command blocks,
    TTY-only status colors/cyan commands, plain redirected output, NO_COLOR and
    TERM=dumb. Preserve machine-readable manifests and plain stored commands.
12. Concurrency/preservation: movie-only viewing can coexist with collection;
    target-appropriate shared locks serialize heavy native work, with clear wait
    messages. Never kill another lane's gate. Protect same saves/branches, preserve
    unresolved failures and historical helper/build dependencies. No cleanup jobs.

## Implementation reference map

Read/copy/adapt these **as a connected implementation**, not only top-level CLIs:

- `start-borg.sh`, `tools/start-recorded-borg.py`, `tools/borg-saves.py`.
- `run-borg-headless.py`, `produce_death.py`, `tools/death-workflow.py`.
- `tools/measure-native.py`, `tools/native-observer.c`, `tools/native-observer.h`.
- `tools/borg-recordings.py`, `tools/native-replay.c`, `tools/native-replay.h`.
- `replay-borg.py`, `tools/borg-replay-viewer.py`.
- `tools/native-inventory.h`, `tools/native-spells.h`, `tools/replay-inventory.py`.
- `tools/replay-observation.py`, `tools/native-combat.h`.
- `play-from-replay.py`, `tools/borg-takeover.py`, `tools/native-takeover.h`.
- `tools/run-report.py`, `replay_next_death.py`.
- `tools/record-death.py`, `tools/aggregate-deaths.py`.
- Corresponding `tools/test_*.py`, particularly recording, native-runner, named
  headless, producer-build-offer, death-workflow, death-record, run-report,
  replay-log, combat-log and takeover controls.

These files make disposable instrumented exports; do not patch the target's
ordinary gameplay engine in place just to force a recording. Inspect integration
sites and native ABI before deciding how to reuse the observers. Do not use
maintenance/refresh commands on frozen historical caches.

## Rust-specific adaptation requirements

The C toolkit's `borg-recording@1` format and native-save hashes are references,
not proof that C private state covers Rust. Inventory the real Rust input/action
seam, RNG ownership, clock/files/config reads, FFI lifecycle, logging, caches and
any threading/nondeterminism. Record any Rust-owned deterministic state/RNG needed
for observation and diagnosis without advancing it. Reconstruct private memory
by executing the original session from birth/origin; a later native save alone
is not an equivalent Rust AI continuation. If deterministic reproduction cannot
be achieved for a path, fail honestly and retain the first divergence.

Freeze the actual engine + Rust library/executable identity, game and Rust source
commits/dirty policy, Cargo.lock, Rust/toolchain/target/features, C compiler/link
settings, data/config, observer sources and built hashes. Cache compatibility must
include Rust inputs; a changed crate cannot reuse a merely matching C build.
Preserve requested versus actual compatible-build provenance, also across pure
documentation/launcher changes. Separate rebuilt historical helper variants from
original engines, and qualify before use. Resume must retain the exact helper.

Self-replay means the **same frozen Rust implementation** reproduces its recorded
execution. It does not require Rust actions/RNG/private state to match a C run.
C/Rust parity and population survival/progress comparisons are separate tests,
with their own pins and claims. Do not relax self-replay to accommodate a parity
mismatch or use the C Borg in place of Rust unnoticed.

Use a target-specific schema/implementation discriminator or justified compatible
extension where needed. Never present a C-only bundle as a full Rust recording.
Record schema evolution and migration/rejection behavior. Preserve existing
player data and old C/Rust baselines under the target's own rules.

## Replay integrity that must survive the port

Freeze binary/data/origin/configuration/locale/timezone and recorded terminal
poll semantics (empty polls, wait/peek/take, resize/queue), clock values and
file bytes at read time. Replay only external input. Never inject recorded AI
decisions, skip divergent actions, patch results, or reset RNG at a later tick.

Compare input/action/clock/file/trace streams, actual frame/phase/player/RNG/world
fields and native save anchors. The C format uses SHA-256 for files/saves and
FNV1a64 for serialized-world fingerprints. Full/delta frame reconstruction must
preserve geometry/palette and observation inheritance. Validate hashes/path
containment/schema/footer counts; verify compression losslessly before removing
generated raw frames. Fail with first stream/row/tick/field difference.

Legacy inventory/spell/combat recovery adds only named observation channels in a
**separate** historical build, with all pre-existing fields still strict. Cache
only after verified equivalence and hash-check that cache on reuse. View-only
never compiles or runs to fill missing data. Missing legacy frozen observer
sources remain a blocker unless restored from a verified archive. An existing
archive is not proof of coverage; verify members and hashes.

Support real wide native terminal dimensions with explicit resource bounds
(reference4096each,1048576cells), while retaining positive-size, terminal-call and
input-queue checks. Keep old failures; do not replace original engines/movies.
Exclude snapshot child output from parent external-input streams; forked native
saving must not mutate/compact the live parent world. Preserve immutable origin
saves against ordinary autosave publication. Validate UTF-8 live trace reads
without hiding malformed finalized evidence.

## Qualification and delivery

Choose target-native bounded fixtures and gates under borg-rs's rules. At minimum
report actual numeric results for:

- Fresh headless random/fixed birth repeatability, visible manual-start/save,
  loaded named continuation and save publication/backup error paths.
- Recorder non-interference: same frozen Rust build, observer-only versus full
  recorder, unchanged inputs/seeds/caps, paired trace/actions and save anchors.
- Full original-start replay of completed living run, natural death, visible and
  loaded session; interrupted/finalized timeout view-only and strict prefix case.
- Negative controls: wrong seed/build/data/config/origin, altered or missing
  input/frame/RNG/save, footer/count/schema/path faults, mid-stream corruption,
  incomplete/crash refusal. Preserve failed receipts; do not weaken assertions.
- Legacy observations: every original field remains strict, player knowledge
  preserved, unknown item/spell data not invented, no RNG/knowledge side effects.
- Log cutoff and hit-count semantics; actual incoming damage and speed; valid
  wide-window recording plus zero/oversized geometry negatives.
- Real terminal interaction: seek/step/log modes/i/e/s, Escape-back/q/focus;
  native display/font probe and retry/resume; no owner game/window interruption.
- Manual prefix selection/numeric/latest, out-of-range rejection, actual save
  fingerprint, frozen resume identity, original unchanged, both deaths and
  living finalized timeout prefixes; manual sessions separately recorded.
- Ten simulated batch deaths all attempted/exported separately, mixed caps/timeouts,
  crash/export/native failure after prior success, sequential unique dossier IDs,
  per-run reports/commands and no implicit budget enlargement.
- Reports from known native saves: real counters, unsupported versions/checksums,
  stale snapshot marking, living native formatting without advancing the game.
- Queue grouping, missing movies, manifest changes, explicit review receipts,
  reversible overrides and replay/manual alternatives excluded from independent
  starts. Existing repository gates and any changed-interface controls.

Do not start an unbounded cohort to test CLI plumbing. Scope native fixtures and
resource bounds prospectively and do not interrupt others' runs. A short smoke
is not a survival estimate; label simulated controls versus native games, full
replay versus prefix verification, and diagnosed cause versus proposed policy.

Deliver a local user guide with copyable commands, keys, option/time-unit/storage/
lock tables, build provenance, incomplete/legacy behavior and troubleshooting.
Provide exact target/reference/source/build/toolchain pins, actual per-test totals,
first-divergence/negative evidence, feature coverage table and next action under
the target's own checkpoint/PR process. State what cannot yet be reproduced and
why. Keep full raw evidence/builds outside Git and curated case evidence reviewable.
Do not claim cross-platform or C/Rust bit equality from a Linux self-replay test.
