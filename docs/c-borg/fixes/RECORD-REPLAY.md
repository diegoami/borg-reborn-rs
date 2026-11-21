# Local recording and verified replay (consumer134)

See the [complete toolkit guide](REPLAY-TOOLKIT.md) for the connected user workflow
and [borg-rs reproduction handoff](BORG-RS-TOOLKIT-PROMPT.md).

The owner requested full local death inspection for visible and headless games,
then selected **verified rerun plus a turn-by-turn viewer**. Future pipelines
are deferred. The separate integration lane owns the consumer bug-carry queue; this task does
not resume or override it, and introduces no tactics.

## Contract

A seed is not a complete replay recipe. Each new session freezes an exact
engine, committed runtime source/data identity, original native save or fresh
seed, initial user configuration and runtime environment/calendar context.
Logical terminal polls/events (including empty polls, peek/take and disconnect),
actual clock values, and preference/`borg.txt` bytes AT READ TIME are retained.
During replay the original game/Borg code still calculates every decision.
Only external input is supplied; game/Borg RNG is never reset at a later turn
and decisions are never substituted from the action transcript.

Every world increment and command/input boundary records actual phase/turn,
player/game/Borg RNG state and logical cells/color changes. Read-only diagnostic
gear, timed effects, Borg goals and visible/unseen monster state are separately
labeled. Native-save fingerprints at command/input boundaries use the existing
forked saver, avoiding live monster compaction. The snapshot child skips only
auxiliary CharOutput.txt generation so it cannot draw through the parent's X11
connection or consume the external-input transcript. Normal native saves still
produce their usual auxiliary output.

Serialized-world fingerprints use FNV1a64; bundle files and native save anchors
use SHA256. Verification compares ALL input/action/trace/frame streams and
starting/recent/final native save bytes, not just outcome, seed or exit status.
This is a checked observational equivalence claim, not a RAM serialization or
unqualified guarantee about undefined behavior or arbitrary future runtimes.

## Local commands

```sh
./start-borg.sh NAME --seed 29
./run-borg-headless.py NAME --turns 30000 --seed 29
./replay-borg.py RECORDING-DIRECTORY --from-turn 20340
./replay-borg.py docs/deaths/D0004-named-headless --from-turn 20340
```

Visible Borg activation stays manual. The visible wrapper preserves ordinary
save/autosave publication while isolating its frozen origin. One open descriptor
freezes an autosave inode before publishing it, avoiding native replacement
races. Different names coexist; same-name locks remain shared by both scripts.
Only heavy builds/verification/headless scenarios take the validation lock;
ordinary visible sessions do not hold it for an entire human play session.

The replay first reruns the original recorded START, then seeks in ORIGINAL
recorded frames. N in --from-turn is an absolute engine turn. From-turn does
not load a checkpoint and pretend it restores the Borg's private memory. A
session loaded from a native save can itself be reproduced from that origin;
previous sessions are linked only when a saved-state hash matches.

--verify-only emits a JSON certificate/result and no viewer; --view-only and
--dump-turn inspect original frames without a rerun claim. A failure identifies
the first mismatching stream/row/turn/field. Original bundles and named saves
are untouched. No new independent death is registered by replay.

### Counterfactual replays stop at the first divergent action

A replayed life that legitimately diverges — a counterfactual engine with a
behaviour change, exactly what improvement units compare — used to keep running
a transcript that no longer described its world: the run could spin far past
the wall limit without writing a decision boundary, and the comparison failed
late with no divergence point (issue392). The replay engine now reads the
original action transcript alongside its own generated actions and stops at
the first mismatch or shortfall: it writes a `DIVERGENCE` row to `trace.txt`
and a `replay-divergence.json` receipt with the divergence turn, decision
count and expected-versus-actual keypress, leaves the already-flushed
per-record streams in place, declines `record-complete.json`, and exits
non-zero via the ordinary quit path. `verification.json` carries the receipt
as `engine_divergence`; identical-life replays compare equal and verify
unchanged. The verifier's watchdog enforces the wall and stall limits before
each iteration's file work and reads the trace incrementally, so enforcement
no longer waits behind a whole-file re-parse of a fast-growing trace.

## Qualification

Native tests use C source1b95154f65b2 (34 carries) and retain each actual
observer/binary/data/tool fingerprint. All tests use disposable saves and
isolated Xvfb displays; no test window appears on the owner's desktop.

- Fresh headless seed11 +200 and +2000: recorded and verified.
- Fresh visible seed11, ordinary birth, manual delay/Borg start/stop/save:
  recorded and verified, including complete raw trace and save bytes.
- Headless seed29, prospective+30000/wall180: natural gallant death at20445;
  complete rerun verified with all frames/actions/RNG and native save anchors.
- Visible seed29, bounded virtual-display test: natural gallant death at20445;
  complete rerun verified by the headless verifier, with the original logical
  X11 preferences, geometry, input and calendar/file transcripts.
- Loaded headless continuation retains an immutable origin and links the
  matching prior snapshot; verification is recorded in the task receipts.
- Frozen v1 non-interference controls: seeds11/29/47, cap20000/wall120/stall30,
  plain observer versus recording observer; complete traces and native
  starting/recent save bytes are compared without widening limits.
- A wrong-seed derived control is rejected as DIVERGED. Original recording
  unchanged. Changed engine/data, incomplete/legacy bundles, and state/action
  mismatches have fail-closed tooling controls.
- A PTY test exercises turn seek, original screen, gear/monster panes and exit.

D0004 and D0005 retain each mode's actual native identity, its verified repeat
and seven ORIGINAL pre-death observation windows. Their Borg notes and all23
noncheckpoint states match D0002; curator qualification groups them into that
existing encounter family rather than adding independent starting conditions.
This establishes replay, not an eligible missed heal, rescue, new C defect or
population survival benefit. Existing causal investigation34 remains open.

## Failed prototypes and limits

Early preflight hooks rejected ambiguous/mismatched source sites before running
a game. An early header missed ui_event declarations and failed compilation.
Initial visible tests revealed two genuine recording-wrapper problems: buffered
clock input across snapshot forks duplicated auxiliary timestamps, and concurrent
native autosave replacement raced publication. Both failed recordings/logs
remain retained. Unbuffered replay readers plus snapshot-only auxiliary-output
suppression fixed the clock issue; immutable descriptor reads fixed autosave
publication. Subsequent complete visible/headless reruns passed. A GUI harness
also initially selected an old process/waited behind another lane's validation;
that harness was corrected, without killing another lane.

The viewer records logical cells/colors, not font antialiasing/pixel video.
Tick observations can precede remaining processing at that engine tick; command
and input phases are explicitly distinguished. Diagnostic unknown/unseen data
is never fed into Borg inputs. Full Borg memory is reconstructed by re-executing
its recorded start; it is not loaded from a mid-run save.

Exact original binaries require their Linux ABI/runtime. Later unsupported
behavior, non-recorded external interventions or nondeterminism may diverge;
verification must report it rather than force a matching state. Legacy runs
without the recording format are not relabeled as verified movies. Future
pipelines can consume exit codes and JSON but are not installed here.

Persistent archive (primary checkout):
`/home/diego/projects/angband/better-borg-c/artifacts/recording-replay-validation/20261107T142051Z/evidence.tar.gz`

- 257,651,207 bytes; 2085 file members.
- SHA256 `ffcf4bfc60439b69ad8c7fefabe0395ec16eca1875b41f91108d1acd99a12b03`.
- Every member was read back and matched to adjacent manifest.json.
- Complete standalone original/replay bundles for D0004/D0005 are also retained
  extracted under `artifacts/recording-replay-validation/deaths/` in the primary
  checkout; dossier paths resolve there, independent of a disposable worktree.
- Canonical dossier comparison: five dossiers, TWO qualified encounter families,
  thirteen registered death process attempts (including verified repeats).

Local tooling:54 tests passed, including recording integrity, failed/legacy
bundles, native save protection, delta/diagnostic reconstruction, wrong engine,
state/action divergence, original-window export and immutable build policy.
The loaded headless +200 continuation actually ended after208 engine turns
(safe-boundary overshoot); its rerun verified and its exact parent save hash
linked to the prior recording. A final short bundle test also verified the
frozen instrumentation source overlays and all native save anchors.

Hosted checks and final commit/certificate receipts are on consumer134 and its PR; full raw runs/builds remain ignored
under the primary checkout's artifacts/recording-replay-validation/. Source
exports, original engine bytes, original/replay inputs and all rejected outcomes
are retained for recovery. Curated dossier windows/logs remain versioned.


## Recovery

Verify archive SHA256 and each member against manifest.json before use. Restore
complete canonical recording bundles as directories, then use replay-borg.py on
that restored directory; original engine/data/config/input files are retained.
Dossier resolution intentionally checks the manifest hash and reports missing
raw bundles rather than substituting today's engine. If restoring to another
location, pass the bundle directory directly. The recorded base source commit
and instrumented source overlays explain the original engine; Linux ABI/runtime
compatibility is still required. Retain original binaries rather than refreshing
recording caches in place. Rebuilding is a new engine variant until separately
qualified, never a silent historical replacement.

## Player-readable inventory

In the replay, `i` shows the pack and `e` shows equipment/quiver, using normal
slot letters, quantities, Angband item descriptions, weights, fuel, charges and
inscriptions. Unknown objects retain the character's unidentified appearances;
the old diagnostic true-kind list is available separately under `d`. Inventory
panels pause autoplay. Escape returns to the original screen; q exits.

New recording exports use native `object_desc(ODESC_PREFIX | ODESC_FULL)` with
player knowledge, never ODESC_SPOIL, and the actual pack/equipment/quiver order.
The observer restores the two everseen flags that object_desc normally sets;
it does not call interactive inventory commands, learn items, spend energy or
change Borg decisions. Delta frames retain the last native inventory snapshot.

Older immutable recordings lack those descriptions and slot/knowledge details.
A normal interactive replay can prepare a separate observation-enriched rebuild
of the ACTUAL historical source plus its FROZEN recording overlay. Only the
inventory observation is added. Qualification requires all original external
inputs/actions/clock/files, trace, frame fields including RNG/native world hashes,
and native save bytes to match. Only the newly added inventory JSON field is
excluded from that comparison; original inventory fields, if present, remain
strict. Failure retains evidence and stops without a claimed native inventory.
The original recording/engine/death identity is never overwritten. The first
preparation compiles locally and reruns the original life; later opens reuse a
hashed qualified cache under primary artifacts/death-workflow/inventory.

`--view-only` never builds or executes an engine. It can use an existing qualified
inventory cache; otherwise an older movie explicitly says inventory was not
captured and asks for a normal replay. It never guesses native names or reveals
raw diagnostic identities as the character's inventory. `--verify-only` and
`--dump-turn` retain the existing strict original-engine verification semantics.

Inventory recovery for the earliest C34 prototype movies uses a separate
primary `artifacts/death-workflow/legacy-observers.json` catalog when the bundle
predates embedded instrumentation files. D0004's24 modified observer/source files
were recovered from the previously hash-verified full recording archive at
`artifacts/recording-replay-validation/20261107T142051Z/evidence.tar.gz`, build
`1b95154f65b2-4b7de28fe4b7`; all four native observer/replay C/header hashes match
its original build manifest. The catalog freezes every recovered file hash and
source pin. It does not rewrite the old bundle or substitute today's Borg code.
Without those archived sources, `--view-only` still opens original frames and
inventory recovery explains the missing prerequisite honestly.

### Inventory and random-birth native evidence

Actual sourceb8095941ff42 observation export: seeds1001/1002/1003/1004 cap200
produce Dunadan Warrior, Half-Orc Priest, Half-Elf Druid and Kobold Mage.
Seed1001 repeat has the same character and whole trace hash. Each original
new recording rerun verifies all streams/frames/native saves. These five short
games are birth/observation controls, not survival estimates. Full results:
primary artifacts/replay-inventory-validation/validation.json.

Original D0006/C40 and D0004/C34 observation-enriched historical runs both
verify every original input/action/clock/file/trace/frame/RNG/native-save field.
Only the additional inventory JSON field is excluded; original fields cannot
be suppressed by that extension. Their qualified cache receipts and native
inventory text exports are retained beside the validation manifest. D0006
shows8 Potions of Cure Light Wounds and4 Coagulated Crimson Potions, keeping
the latter unidentified rather than exposing Sleep. Equipment is a separate
panel rather than mixed into the backpack. No new natural death was generated
or historical source/recording changed by these replays.

Early observer build attempts with missing C header includes were retained,
corrected, then rebuilt; no assertion, input or identity check was loosened.
The independent collector-unit nested-lock bug was fixed in PR169; the owner
approved terminating only the other task's already-stuck unittest child, whose
interrupted tooling result remains an interruption, not a passing gate.

The real PTY check at D0006 turn72995 verifies native pack text, unidentified
Coagulated Crimson Potions, separate equipment slots, Escape returning without
exiting and q closing the private test viewer. Transcript/receipt:
primary artifacts/replay-inventory-validation/pty-inventory.{txt,json}. An early
PTY harness read a partial terminal write too soon; it was corrected to wait for
the expected text, without changing the viewer or native comparison assertions.
Fixed-birth and loaded-Mage controls are retained separately; loaded saves use
their actual native CHARACTER rather than a hard-coded Warrior description.

The direct recorded-load control initially exposed product harness issue173:
native autosaving could overwrite the run's frozen origin copy, and immutable
validation correctly rejected it. Original supplied saves were unchanged. The
observer now redirects every recorded loaded life's save target to autosave.save.
Unchanged cap200 before/after: old loaded-Mage control fails the origin guard;
corrected loaded Mage preserves both the supplied start and copied origin,
reports Kobold Mage, and original-start replay verifies. Both fixed Human Warrior
and loaded Mage full traces are byte-identical before/after this file-destination
correction. Evidence and the failed run stay under primary
artifacts/replay-inventory-validation; origin checks were retained.

## Caster spells and prayers at the selected moment

Press `s` for learned spells and prayers: native spell letter/name, level, mana cost, current
failure percentage, book and current availability. Ready means the ordinary
casting guards and book/learned checks pass; targeting/effect success is not
predicted. Low mana is a legal overexertion attempt requiring the game's normal
confirmation, not an ordinary fully funded cast. Missing book, forgotten spells,
blindness/darkness, confusion and death are distinguished. Unknown/unlearned
spell names are not added to the replay view.

The observation uses native player spell flags, class-book definitions,
`player_can_cast(false)` and `spell_chance()`. It invokes no study/casting command
and consumes no game RNG. Carried books and known books at the player's feet
are considered, as the game permits. New recordings freeze native-spells.h with
other observer sources and carry spell snapshots through delta frames.

An older caster recording can add only the spell field through a qualified
historical observer rerun, preserving EVERY pre-existing field including native
inventory if already recorded. A cached inventory-only certificate does not
qualify spells; the certificate must explicitly include that extension. Older
noncasters are recognised from native CHARACTER and frozen class data, reusing
the existing inventory cache without another full-life run. View-only never
builds/executes an engine; absent uncached spell data stays explicitly unavailable.

Native results on source963adf0629e5, four independent birth seeds/cap200:
Dunadan Warrior (no spells), Half-Orc Priest (Bless,2MP,14% failure), Half-Elf
Druid (Detect Life,1MP,18%) and Kobold Mage (Magic Missile,1MP,17%), each with
verified original-start replay. The older Mage inventory recording also gains
spells with every original inventory/state/input/native save field identical.
Real PTY exercises s, native cost/failure/Ready, Escape-back, i and q. Evidence:
primary artifacts/replay-spells-validation/ and
artifacts/replay-inventory-validation/mage-spells-overlay.json. These short
observation checks are not survival/progress measurements. Non-ready edge states
have representation/source guards; these four native runs exercise ready
casters/noncaster, not every possible impairment or missing-book trajectory.

## Recorded action and message log

Press `p` to inspect the log at the selected moment. `Tab` cycles **Borg action
notes**, **all Borg notes**, and **game messages**. `k` scrolls older entries and
`j` returns toward newer entries. Arrows/g still change the replay position;
the log follows that position. Opening the panel pauses autoplay, Escape returns
to the screen, and q exits. Inventory/equipment/spell views retain i/e/s.

Borg action notes include attacks/targets, spells/prayers, firing, consumables,
activation, retreat, rest and other native action descriptions. All-notes also
shows existing danger/flow/rejection notes where originally logged; values are
never recomputed under today's Borg. These are recorded notes/intent, not proof
that an attempted attack or spell succeeded.

Legacy BORG trace rows have engine-turn timestamps but no frame/decision phase.
Several actions and level changes can share one turn (D0007 turn178286 is a real
example). The panel therefore includes those notes only AFTER that turn has
completed, with an explicit through-turn caption. At the final frame of a
complete movie, final-turn notes are included. It does not present a later
same-turn note as if it had already occurred at an earlier frame.

Game-message view indexes changes of the recorded visible top message line,
using exact frame/turn/phase. Transient prompts/combat messages remain inspectable
after the live sidebar clears them. Identical unchanged text is not a separate
visible change; a clear followed by repeated text is retained. Undated MESSAGE
rows from the terminal's end-of-run buffer are not assigned invented historical
times. This is the available recorded history, not a promise of an unrecorded
complete per-blow event stream. No game rerun/instrumentation or movie modification
is needed to open the log, including with --view-only.

Controls verify same-turn future-note/message exclusion, completed-final-turn
handling, action/all-note filtering, delta/repeated-message reconstruction and
bounded scrollback to the oldest entry. Real D0007 PTY178380 shows the original
Casting Phase Door178379, cycles sources, scrolls back and exercises Escape/i/q.
Evidence is primary artifacts/replay-log-validation. The initial PTY harness
expected a contiguous repainted title; curses correctly emitted incremental
updates. The final harness observes actual window rows while sending real PTY
keys, with no change to the viewer's output or game data.

## Play a manual alternative from a recorded turn

```sh
./play-from-replay.py docs/deaths/D0007-collected --from-turn 178325
```

This separate script replays the historical game up to the **first living player
command boundary at or after** the requested absolute engine turn, then opens
X11 with the Borg off. It prints the actual turn and frame: for D0007,178325
selects178326. A turn may contain several commands or other phases; this selects
the first qualifying command, not an arbitrary displayed frame. Requests before
the recording's first command or beyond its last living command are rejected.
Use normal Angband controls (arrow keys with the standard keymap), click/focus
the game window, and use Ctrl+X to save and exit.

Historical replay engines/build caches and original recording bundles are never
updated in place. On first use the script **offers a separate historical takeover
helper build**, using the recorded source commit, frozen observer overlay and
runtime data. Declining cancels; `--yes` accepts that build, and `--no-build`
requires a matching existing helper. No current-main gameplay fixes are imported.
A missing/changed/incomplete helper is preserved and rejected, not overwritten.
Legacy movies need their archived historical observer sources restored first.
The ordinary `replay-borg.py` behavior and its original engine are unchanged.

Before opening manual play, the helper reruns the original inputs to the selected
command, freezes its native world save and stops. Every output row of terminal
input, Borg actions, clock/file inputs and native trace must match the original
prefix. All logical terminal frames, RNG/world fingerprints and recorded
observation fields must match through the exact command frame; footer counts,
consumed file contents and the actual saved-world fingerprint are checked too.
A mismatch, timeout, missing source or missing save prevents manual launch and
retains diagnostic output. `--timeout` bounds the prefix replay (default600s);
it does not limit your manual session. This is prefix qualification, not a claim
to have rerun or verified the remainder of the original death.

Manual play starts by loading that native save, using the frozen historical
helper in an isolated session. Normal save/load semantics apply: the Borg's
in-memory goals are not serialized or restored. The script does not restore
mid-animation/mid-command execution, feed diagnostic knowledge to the player,
or promise the same future random events after your choices differ. Initial
player-known state is retained by the native save; the new branch is a human
counterfactual, not an additional independent Borg life or survival improvement.

The script prints the branch directory under
`artifacts/death-workflow/takeover/branches/`. Saves and new full recordings stay
there, including parent recording/build/boundary identity. Named playable saves,
the original death dossier and owner-review state are unaffected. To prepare a
verified save without opening a window, append `--prepare-only`. To continue your
own saved branch later:

```sh
./play-from-replay.py --resume /absolute/path/to/takeover/branches/D0017-t3983-TIMESTAMP
```

Resume retains the branch's exact helper build even after newer game fixes or
script updates. A dead branch cannot be resumed as alive; start a fresh branch
from the historical recording to try another alternative. Concurrent launches
of the same branch serialize on its manual lock. A display is required for play;
`--prepare-only` works headlessly. Manual branch recordings can be inspected with
`./replay-borg.py SESSION_DIRECTORY`; they are explicitly labeled counterfactuals
and are not automatically added to the next-unreviewed Borg-death queue.

### Timed-out survivors and the latest recorded turn

```sh
./play-from-replay.py RECORDING_DIRECTORY --from-turn latest
```

`latest` selects the **first living command boundary at the last recorded
living-command turn**. Multiple command phases can share one turn; selecting
that first boundary preserves the historical hook's exact semantics. Explicit
numeric turns still select the first command at/after that turn. The script
prints the actual turn and frame before qualification.

Manual takeover now accepts a **finalized wall-timeout recording** as well as a
complete normal recording. It checks all frozen file hashes, the finalized input
manifest and matching timeout outcome, then requires exactly the same
frame/input/action/clock/file/trace prefix and native save fingerprint as for a
complete movie. Only that prefix is qualified; the original remains marked
incomplete and the receipt records its original status/outcome. No full-run
verification, normal completion or independent survivor is inferred from it.
Active/unfinalized recordings, changed files, recording errors and crashes are
refused. A missing/malformed tail or divergent rerun can still prevent takeover;
no terminal input/RNG is fabricated to get past it. The original replay command
remains strict by default; use `--view-only` to watch a timed-out movie.

First use can offer the separate helper build for the **recorded historical
source**, even if current main has newer fixes. Existing replay engines and
recordings remain untouched. `--prepare-only` qualifies without opening X11;
`--resume BRANCH_DIRECTORY` later plays the isolated branch with its pinned
helper. `--timeout` on this script bounds prefix reconstruction (default600s),
not your manual session; it is independent of the collector's per-character
`--timeout`.

### Display/font preflight and retrying a failed launch

Before building a helper or reconstructing a prefix for visible play, takeover
now opens the requested X display and probes fonts with the same Xlib font-load
operation the native game uses. It honors `BORG_PLAY_FONT`, then
`ANGBAND_X11_FONT_0`. Explicit unavailable choices fail immediately; default
selection tries readable Courier/misc-fixed sizes and installed bitmap aliases,
then the server's `fixed` fallback. Some minimal WSLg servers advertise scalable
names but only actually load `fixed`; font listing alone is not sufficient.
The chosen font is printed. `--prepare-only` remains independent of X11.

If prefix verification succeeded but the visible launch failed, the branch is
still valid. Use the printed `--resume BRANCH_DIRECTORY` command to retry,
without another helper build or replay from birth. Failed launch evidence stays
in its session directory. For an older launcher whose default font is missing:

```sh
ANGBAND_X11_FONT_0=fixed ./play-from-replay.py --resume BRANCH_DIRECTORY
```

With the updated launcher a loadable default is selected automatically. Use
`BORG_PLAY_FONT=INSTALLED_FONT` to choose a different available size. No fonts
are installed, no X server settings change and no original recording/build is
modified by this selection.

If only tiny built-in fonts load, visible takeover now tries the installed
Type1 font directories before accepting the small fallback. WSLg can access the
client distro's fonts via `/mnt/wslg/distro/usr/share/fonts/X11/Type1`, even when
its own `/usr/share/fonts/X11/Type1` is absent. The launcher preserves all existing
X font paths, appends a directory only if it supplies the requested/readable font,
and restores the old paths after unsuccessful attempts. No font package or
engine rebuild is required. Courier/Nimbus28 is preferred to6x13; actual tested
Courier metrics are17pixels wide and30high. Explicit font overrides still win.
This setup is repeated on demand after an X server restart; no background job.

## Native fighting log and speed

```sh
./replay-borg.py docs/deaths/D0010-collected --combat-log --from-turn 6255
```

Press **p**, then **Tab** to select **Fighting log**. Native messages include
hits, misses, shots, fear, consumables and deaths; `HP` rows separately record
actual before/after player HP and the native damage source. A hit message and
its HP row describe the same event, not two hits. Borg diagnostic `#` messages
are excluded from this source; the existing Borg-note sources remain available.

Old recordings did not capture every message before native message coalescing.
`--combat-log` prepares a **separate frozen-source observation build**, captures
native message calls and damage outcomes, and requires every original stream,
frame/RNG/player-state field and native save to match. No extra danger evaluation,
RNG draw, gameplay input, heuristic or original-engine/cache change is made.
It qualifies a complete original movie, not a timed-out partial one. Once
qualified, the added channel is cached; `--view-only --combat-log` reuses it
without building/running. Without cached data, the panel explicitly asks for
recovery rather than inventing hit counts from repeated Borg notes or HP samples.
`--verify-only --combat-log` prepares/verifies without opening the viewer. The
owner-review queue also forwards `--combat-log`.

Events carry their actual native tick and the next original frame boundary at
which they are available. This avoids revealing a later same-tick hit when
looking at an earlier frame. Speed changes are linked to their actual recorded
frames. Full original frame equality is checked; original cells aren't repainted
with newly inferred combat observations. Recovered log/qualification evidence
stays under `artifacts/death-workflow/combat/`.

An **engine tick is a shared world-clock increment**, not a player action or a
frame. Player and monsters gain energy according to their own speed. With the
normal move budget100, speed110 gains10energy/tick (~10ticks per move); speed120
gains20 (~5ticks), speed100 gains5 (~20ticks). Other action costs, accumulated
energy, firing/blow rates and turn order can change the exact interval. Inventory
and viewer inspection don't spend a normal movement turn. Multiple command/input
frames can share a tick. The fighting header displays the actual recorded player
speed/gain/move budget, not an assumed fixed "turn" conversion.

D0010 qualified example: ranger speed110/gain10; kobold speed111/gain11. Native
incoming hits occur6291/6300/6309, dealing8/6/2HP,15→7→1→-1. This is3hits,
not the duplicated `HIT_BY(+)`/`HIT_BY(3)` diagnostic lines. The owner's separate
manual branch survives and retains its own input/RNG trajectory and certification.

## Maximized terminal compatibility

Native term dimensions are ints. The old recorded input reader incorrectly
rejected widths/heights above255, including a real424×104owner session.
New exports use bounded int geometry (maximum4096each, at most1048576cells),
retaining positive-size/call/queue checks. For an older immutable engine the CLI
automatically selects the separate compatibility observer when its frozen reader
has the old limit. The old executable is preserved. Corrected historical replay
must still match every original event/frame/native save. Native negative tests
refuse zero-size,4097-wide and excessive-cell inputs; the actual424×104manual
recording verifies completely. Normal-size recordings retain their usual path.
