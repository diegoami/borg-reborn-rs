# Rust launch and recording adapters

The current connected workflow is documented in [the Rust toolkit guide](../docs/REPLAY-TOOLKIT.md),
with [executed qualification and remaining gaps](../docs/TOOLKIT-IMPLEMENTATION.md).
The notes below preserve earlier implementation/review provenance; their former
first-death collector stop rule and fixed/hashed birth defaults are historical.
Current collection continues after every exported death/ordinary limit and uses
native seeded random menus. Historical engines/movies are not refreshed.

Workflow source: diegoami/better-borg-c, committed record/replay feature
08a5cd019 (already merged in donor PR147). IMPORTED.json pins full donor
commit/path/SHA. Original licence/credit is preserved. Save helpers and the
terminal-frame viewer are reused; native engine diagnostics are adapted
without reading C Borg globals. The donor repository/worktrees are read only.

The headless Rust backend uses the existing verified harness and an engine-only save
metadata reader relinked with Rust's actual engine/staticlib objects. It never
links the C Borg. Names are validated; GUI/headless locks are shared; backup
restore preserves the current file. Headless publication uses a content check
plus atomic replacement; an outside writer causes refusal rather than overwrite.
GUI play uses the ordinary native save path and retains the character lock
in both the waiting launcher and game process until they exit.
No tests open or modify owner saves.

Headless records live under BORG_PLAY_DATA_DIR/recordings (default
.angband-reborn/recordings). Each bundle freezes the executable, runtime data,
runner/supervisor/config, seed/budgets and original save. Loaded games keep
their native engine RNG; a new process starts fresh Borg memory. Seeded
headless calendar context is pinned to the epoch by the existing harness.
This is not restoration of a running Borg or live checkpoint rewind.

GUI recordings use a separate `borg-rs-gui-recording@1` schema. The ordinary
X11 launcher enables the optional Linux engine recorder; ordinary harness
builds leave it off. GUI play freezes executable/data/initial profile and
captures terminal poll results (including empty polls, peek/take, geometry,
disconnects), actual Rust hook output, calendar calls from C and Rust, the
PID used only for RNG initialization, local timezone rules, and file-read
bytes at every native read, including live preference reloads. It records
main-terminal game frames, engine RNG and fork-private native-save world
fingerprints at command/input boundaries. It supplies no future RNG or Borg
decisions during replay. The normal user/preferences directory stays live.
GUI recordings are completed only after successful child exit and the engine
footer. Replay runs the same frozen executable with a dummy display, preserving
the save name, terminal geometry/input conventions and X11 preference identity;
no X server is needed. It compares all external input/action streams, exact
frames (including world fingerprints), completion counts and final native save.
Extra unconsumed input, missing streams, changed files and divergent output fail.

The default replay executes the frozen Rust Borg from its original start and
requires byte-identical keys/messages/player state, terminal frames, final
native save, scheduled world-save set/bytes and run configuration. It also
compares normalized summaries and checks process success and all recorded
file hashes. Cached verification is invalidated by manifest or verifier-code
changes. It does not prove every private Borg field or unseen engine state
between saved boundaries. Actual inventory/monsters/timed effects are
diagnostic-only frame output, not additional AI input; C Borg private fields
are omitted. Verification is scoped to these channels and finite budgets.

--recording-only views incomplete/failed frame streams with an UNVERIFIED
badge; it never promotes them to a verified rerun. Modified files, unsupported
legacy schemas and divergent reruns are refused. Completed run evidence is
retained even if named-save publication loses a race. Death/replay branches
are not new independent natural-death samples.

Both new GUI and headless sessions are replayable. GUI support is Linux/X11
character mode with main-terminal game frames; graphics/auxiliary window pixels
and pre-character birth/menu screens are not visual recordings. External input
and final character state still cover a fresh birth. This is a frozen-origin
rerun, not arbitrary mid-session restoration of private Borg memory. Initial
bundle hashes protect retained evidence, not provide a cryptographic signature.
Frozen executables/data add disk usage per session; long GUI recordings also
include a forked engine snapshot per command/input boundary. No automatic
recording cleanup deletes historical evidence.

```sh
python3 tools/test_borg_tools.py
python3 harness/regress/gui-replay/run.py /tmp/gui-proof --project-root /tmp/borg-rs-test-checkout
BORG_GUI_TEST_RECORDING=/path/to/borg-intervention-recording python3 tools/test_gui_recording.py
./run-borg-headless.py --help
./replay-borg.py --help
```

Tests cover five-slot rotation, restore preservation, outside-writer refusal,
shared locks, name traversal, exact real-engine replay, corrupt state/engine,
wrong replay seed and legacy refusal. Additional real integration checks
exercise named GUI restore/locks, three GUI save/reload roundtrips, death
publication/restore, and automatic non-Human/non-Warrior save continuation.

GUI regression covers six real sessions: load/save, reload, backup restore,
Rust Borg activation/interruption plus manual play/live borg.txt reload/resize,
a loaded Hobbit Rogue, and fresh birth. Negative controls exercise runtime/input/
action/save corruption, incomplete streams and extra terminal input. Tests run
on a private Xvfb display and disposable profiles/build, never owner saves.


## C-compatible parameters and death records

Replay has the same public parameter names as better-borg-c f3ced6219768:
--from-turn N, --dump-turn N, --view-only, --verify-only, --output NEW_DIR,
--timeout SECONDS. Rust's older --turn/--dump/--recording-only aliases remain.
--from-turn selects the first original frame at/before that engine turn; the
verification always reexecutes from the frozen origin. --output retains actual
rerun streams/profile and a success/failure proof; an existing output directory
is never overwritten. Failed verification also retains a temporary diagnostics
directory automatically; successful temporary outputs are cleaned up.
Verification timeouts stop the replay engine, preserve
the original and never publish a named save. Death dossier paths and record.json
resolve to their original bundle only when its manifest hash still matches.

### Playing from a recorded turn

```sh
./replay_next_death.py --case D0006 --play-from-turn 22794
./replay-borg.py docs/deaths/D0005-recorded --play-from-turn 36094 --prepare-only --output .angband-reborn/my-attempt
./play-branch.py .angband-reborn/my-attempt
```

`--from-turn` continues to select viewing. `--play-from-turn` creates a
separate human-controlled native save at the first living top-level command
boundary at or after N. Multiple decisions can share a world turn; the receipt
also records the number of preceding keys and `before_game_command` phase.
The actual turn/position/HP/SP are printed. A skipped turn rounds forward
(D5 request36095 captures36104); death/no later living boundary is refused.
`--prepare-only` does not require a display; the default opens Linux/X11.
`--timeout` bounds each preparation build/replay, never human playing time.
Viewing/dump/verification modes cannot be combined with takeover.

The tool rebuilds the recording's exact committed Rust/engine source with
declared capture and manual-start hooks, original data and version stamp.
It refuses unreconstructable/dirty historical source and currently supports
headless origins only. A complete control rerun must match all original
channels, raw frames, full final/checkpoint saves and archive bodies (with the
existing declared headless clock-header exception). The selected run must
match the original key/state/frame and message prefixes; native capture saves
use a fork so saving cannot compact the live replay's monster indices.

The copy has its own frozen engine/data, settings, native save, lock, five
rolling backups and GUI recordings. On opening/resuming, a historical native
reader and GUI handshake check the serialized player fields and full complex
game RNG before input is accepted, and require Borg off. Headless epoch-based
nonholiday monster availability is retained. Quick RNG used for flavor setup,
private Borg memory and exact future equivalence to an uninterrupted engine
are not claimed: native load can compact/rebuild engine structures. This is
a serialized game counterfactual, not a proven rescue or a Git branch.

Default paths are `.angband-reborn/human-branches/TIMESTAMP-tN/`, or under
`BORG_PLAY_DATA_DIR`. The printed `play-branch.py DIRECTORY` command resumes
the same human character; running `--play-from-turn` again creates another
attempt. Existing output directories are never replaced. Failed preparation
is retained but cannot be launched as a ready branch. Ctrl-X saves; Escape
closes the native exit screen (Return opens scores). Counterfactual GUI
recordings can themselves be verified/replayed, carry original provenance,
and are excluded from automatic death-dossier registration and owner-review
marks. They do not add independent lives to the original death cohort.

Headless --timeout aliases --wall-seconds, and --death-issue links an exported
case. Rust keeps its own defaults (seed42, optional100k-turn budget) and extra
race/class/new/key/checkpoint controls. Rust additionally supports --stall-timeout
(default off for the named launcher,30seconds in the collector), alongside
independent world-turn/key/wall limits. These launchers are not fully identical
argument sets. Donor death-issue defaults are not copied across repos.

Both launchers export each confirmed natural terminal death after the session
completes. The registry defaults to docs/deaths; BORG_DEATH_RECORDS_DIR overrides
it for private/testing profiles. Registry allocation is locked and dossier
publication uses a staged directory; original identity and evidence are not
replaced by later builds. Caps, failures and retirement do not become deaths.
Borg/game cheat-enabled cases are rejected from natural-death export. GUI mixed
control and loaded/backup branches retain explicit context. Every assessment
starts unassessed; logging does not infer a defect, open an issue or alter tactics.

Dossiers contain native Borg maps, borg.dat/borg-log.txt, bounded original
keys/messages/state/actions, actual terminal state, source/binary/config/input
hashes, checkpoint hashes and explicit load-validation status. Full frozen
bundles and saves stay outside Git. GUI archive output is isolated per session (concurrent characters cannot mix
Borg.map/death logs); it preserves newly written native archives in its bundle (stale archive files are excluded) and replay checks them
byte for byte. Headless replay also compares archives: maps/data/all log-body
bytes must match. Its historical Rust death-log header uses the real wall clock;
only recognized date/time header-line differences are reported separately in
verification.json. Original timestamp bytes remain hashed and immutable. The
existing gameplay streams, frames and native saves still require exact matches.

The donor's default pre-death offsets5/12/25/40/60/82/105 are engine turns.
Exact original frames are exported when present. Missing exact frames remain
unavailable; separately named nearby anchors carry actual turn, phase and offset,
with no interpolation or false claim of exact timing. Several requested offsets
can share one actual boundary; they are not distinct decisions or lives.

Use tools/record-death.py --run BUNDLE --output DOSSIER --id D0005
[--investigation URL] [--repeat BUNDLE] for manual export; repeat evidence must
match actual source/start and complete observable channels. Existing assessments
and attempts are preserved; issue metadata and qualified repeats may be appended.
Then run tools/aggregate-deaths.py. The index groups attempts by original input,
keeps source versions separate, and groups actions by canonical issue/key. This
is a selected-case registry, not a mortality estimate. The four earlier cohort
deaths keep their actual historical builds, twelve C/C/Rust attempts, original
assessments and explicit absence of movies.


## M5 review follow-up qualifications

For a headless continuation, the requested --seed remains recorded provenance
and the frozen replay still uses it to reproduce its summary. Gameplay resumes
the saved engine RNG. Effective death identity/family therefore has seed null;
different requested seeds on the same saved origin do not create another case
or starting condition. Each actual run remains a separate process attempt with
its requested seed recorded. Aggregation also derives this effective family
for older continuation dossiers without rewriting their stored identity or
artifacts. Fresh births still use seed as part of starting-condition identity.

Headless --from-backup now builds and reads a private snapshot of the selected
backup, validates living state and requested race/class, and only then restores
those exact validated bytes and rotates backups. Rejected build/metadata/race/
class/dead/corrupt requests leave the named save and backup set unchanged.

The headless native death-log clock exception is restricted to **line 1** of
borg-log.txt; later lines compare exactly even when both look like timestamps.
The original first date is preserved/hashed and reported alongside the rerun
value; that date is not authenticated by replay. GUI archives remain byte-exact.

verification.json is a trusted local cache, not a signature or independent
attestation. A matching cache may satisfy the default command without another
game execution. Use --output NEW_DIRECTORY or --timeout SECONDS to force an
actual frozen-origin rerun. The manifest/runtime hash checks still run before
cache acceptance. Death/CLI controls derive expected cause/turn/frame from the
supplied original bundle and can be rerun on a separately generated death.

## Collecting a death and reviewing the next encounter

`produce_death.py` and `replay_next_death.py` adapt the committed C workflow
at22f817c96843 (source hashes/licences in IMPORTED.json). These are post-M5
operational tools; the accepted m5 tag stays fixed, and no newer C Borg gameplay
fixes or C milestone are imported by these commands.

```
./produce_death.py
./produce_death.py --birth fixed --seeds 8400 8401 --turns 100000
./replay_next_death.py --list
./replay_next_death.py --verify-only
./replay_next_death.py --from-turn 37000
./replay_next_death.py --mark-reviewed D0005
./replay_next_death.py --mark-unreviewed D0005
```

The collector freezes the committed Rust `--ref` (default HEAD), prospective
seeds/race/class choices and all budgets before building/running. Defaults:
3games, random first uint32 seed/consecutive subsequent seeds, varied race/class,
200000world turns,120wall seconds and30seconds without observed world-turn
advance per game. Rust adds independent --keys (default50000) and checkpoint
interval controls. --seeds excludes --games/--seed; limits/seeds are never
expanded after results. --birth fixed selects Human Warrior. With random birth,
Rust freezes SHA-256-derived race/class choices before play and uses ordinary
explicit native menus/point buy. The C donor uses native random menu keys;
identical seeds do not assert identical cross-project choices/RNG trajectories.
Neither selector changes or supplies decisions to a playing Borg.

The pinned source snapshot/data and its own Rust recording build are prepared
in ignored primary build/death-workflow caches. --build can reuse an ordinary
Rust harness directory only when source hash/binary/implementation/features
match the committed pin. Game inputs come from the pinned snapshot, not edited
working-tree data. Current observation runners and their exact source files are frozen separately,
fingerprinted and checked across games. An immutable plan.json and its hash bind
the prospective contract; each game recording retains that contract and a tool-
source snapshot, even when the tools were being developed outside the selected ref. A changed input/binary/tool fails collection. Build,
configuration, source/archive hashes and logs stay available; incomplete builds
are retained for inspection rather than silently retried or treated as valid.

Each game uses a new private profile, with cheat death/live and winner respawn
off and death evidence on. Collection stops at the first confirmed natural
death, operational failure or exhaustion of the prospective games. A world-turn
cap, key cap, stall, timeout, crash or unexpected Borg stop retains its own label;
none is relabeled a death/survivor. A death gets an ordinary unassessed dossier;
failed capture/export retains the death/failure evidence and does not start
another game. The collector is a selected-death search, not a mortality cohort.
A cap can overshoot at the next observable native key boundary.

--stall-timeout watches complete flushed key-log world turns, independently of
wall time and output activity; zero-clock cycling cannot reset it. Trace buffering
means it observes key boundaries, not every internal tick. The supervisor sends
signals only to its own child and retains process/checkpoint failure receipts.
The optional stall limit defaults off for older ordinary headless launches;
frozen legacy recordings keep their original supervisor and semantics.

Collections default to primary-checkout harness/out/death-workflow/collections;
--output must be a new ignored path under primary harness/out or the explicit
BORG_DEATH_WORKFLOW_DIR override. All games and recordings remain outside Git.
BORG_DEATH_RECORDS_DIR chooses the dossier registry; --death-issue optionally
links discussion (no borrowed C issue default and no automatic issue creation).
Build wait is separate from each native game's wall budget.

The replay queue groups canonical encounter families, including pre-fix saved-
origin seed aliases and explicitly verified reference-family qualifications.
It lists/skips legacy, missing or corrupt full recordings and chooses the first
unreviewed playable family. --case overrides that selection, including reviewed
cases. Default replay forces a fresh verified run and retains evidence under
harness/out/death-workflow/replays; --view-only opens UNVERIFIED original frames
without creating verification output. --from-turn/--timeout are passed through.

Owner review is separate from assessment/classification/actions. Closing a
viewer, passing verification, technical analysis or a later fix never marks a
family reviewed. Only explicit --mark-reviewed/--mark-unreviewed changes the
local atomic locked ledger, and it applies to the whole qualified family.
The committed docs/deaths/owner-reviews.json is a baseline; local overrides in
harness/out/death-workflow/owner-reviews.json take precedence. No C owner-review
marks are copied. BORG_DEATH_WORKFLOW_DIR isolates the ledger and replay output
for tests/private workflows. Existing dossier and assessment bytes stay intact.


## Updated player-known replay views and historical recovery

The C tooling comparison is pinned at85b6ba8e3748e061db1ff1206163c27272d2de31.
Only observation/viewer/analysis scaffolding is adapted; that C commit's game
fixes are not imported or selected as a gameplay milestone. M5 stays0203227.

**i** shows native player-known inventory; **e** equipment/quiver and burden;
**s** learned/forgotten spells/prayers, native cost/failure/book/current status;
**p** the Borg-action/all-note/game-message log (Tab source, k older/j newer).
These panels pause playback on entry. Escape returns to the screen; q exits.
**d** remains clearly labeled actual/spoiler gear diagnostics and **m** actual
monster diagnostics. Inventory is never inferred from those hidden kinds.
The footer flags a stale HP/SP sidebar separately from recorded actual state;
it does not edit the original screen or call it a gameplay discrepancy.

Spells use the historical engine's player flags, spell_chance, book knowledge,
visibility/confusion and current mana. Ready means native prerequisites for an
attempt, not Borg admission/selection or guaranteed success. Low mana can still
allow an overexertion attempt, as in the game. No book, forgotten, blindness/
confusion, death and noncaster states are distinct. Descriptions preserve
knowledge: no ODESC_SPOIL, and temporary kind/ego seen flags are restored.
All these getters are observation output, never additional input to Borg.

New headless and GUI captures store these fields as full/delta observations.
GUI also records exact native message events with frame boundaries; capture
metadata declares that stream and deletion fails validation. Headless logs use
the existing key-count boundaries. Notes/game events emitted after a frame are
not exposed at that frame, even if their world turn is the same. The older C
trace format uses conservative completed-turn visibility; old GUI movies fall
back to recorded visible message-line changes when no native stream exists.
Logs can include intent and simulation notes; verify actual movement/action
results from keys/native state, not a note substring alone.

Old original recordings and dossier identities are immutable. For a normal
interactive replay of a reconstructible old headless recording, the adapter
rebuilds its **historical Rust/engine source**, adds only the declared known-
view observer headers/calls, and verifies every original frame value/channel,
final save and scheduled save. Only previously absent inventory/spells/vitals/
caster fields may be added; old values, extra unknown keys and row-count changes
fail. An observer-disabled rerun must also match original channels exactly.
The qualified extension lives separately under harness/out/death-workflow/
observations, with original manifest/image/new image/helper hashes and proof.
The display explicitly labels recovered observations. These are supplemental
matched historical observations, not an original capture or another life.

--view-only never builds an extension; it can use a previously qualified cache
with its UNVERIFIED badge. --verify-only/dumps retain the original channel
semantics. If the historical source cannot be qualified, the verified original
movie still opens with missing-view labels; no spell or item state is guessed.
Old GUI originals remain available, while new GUI recordings have native views.
No arbitrary mid-run private Borg memory restoration is introduced.

[Death-review hub #3](https://github.com/diegoami/borg-rs/issues/3) is the default
for this registry; a private registry without discussion.json stays unassigned.
The owner can comment with case/actual turn/phase. The structured review template
requires overall picture, timeline, brainstorm/candidate fit and next evidence;
observational review can be complete while exact private gates/rescue remain
provisional. Detailed D0005 assessment and follow-up are in its dossier and
[focused investigation #4](https://github.com/diegoami/borg-rs/issues/4).
