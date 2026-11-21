# Replay and death-analysis toolkit

This is the entry point for collecting games, inspecting deaths or living runs,
playing an alternative, and turning observations into investigations. Commands
run from the repository root. Replace uppercase placeholders with actual paths
or values. This guide describes the tools at
[`a4084032485a7c7f29153d0b37ce63dede3e4d5a`](https://github.com/diegoami/better-borg-c/tree/a4084032485a7c7f29153d0b37ce63dede3e4d5a).
The implementation files and each command's `--help` define exact options.

For implementation/reproduction in another repository, use the accompanying
[borg-rs handoff prompt](BORG-RS-TOOLKIT-PROMPT.md). The lower-level contracts and
historical qualification receipts remain in [RECORD-REPLAY.md](RECORD-REPLAY.md);
queue/collector details are in [DEATH-QUEUE.md](DEATH-QUEUE.md).

## Choose a tool

| Goal | Tool | Effect |
| --- | --- | --- |
| Play a named character in X11 | `./start-borg.sh NAME` | Resume/create native save; manual Borg activation; record session |
| Let the Borg advance a named save | `./run-borg-headless.py NAME --turns N` | Automatic Borg, additional engine ticks, publish living save |
| Collect several disposable lives/deaths | `./produce_death.py --games N` | Fresh seeded characters; every death exported; reports for all attempts |
| Inspect original movie | `./replay-borg.py RECORDING_OR_DOSSIER` | Verify original-start rerun, then show original frames |
| Inspect without rerunning | `./replay-borg.py RECORDING_OR_DOSSIER --view-only` | Viewer only; no engine execution or build |
| Open next owner-unreviewed death | `./replay_next_death.py` | Resolve oldest eligible encounter family, then replay |
| Play a human alternative | `./play-from-replay.py RECORDING_OR_DOSSIER --from-turn N` | Qualify prefix; load isolated native save; Borg off |
| Reprint report/replay paths | `./produce_death.py --commands COLLECTION` | Read existing results, no new games/builds |
| Generate reports for older runs | `./produce_death.py --reports COLLECTION` | Read preserved save; optional frozen-engine character-sheet formatting |
| Curate a natural death | `tools/record-death.py` | Export immutable provenance and selected original evidence |
| Regenerate death registry | `tools/aggregate-deaths.py` | Combine curated assessments; do not count replays as independent lives |

`RECORDING` is the printed raw run directory containing `recording.json`.
`DOSSIER` is a case such as `docs/deaths/D0010-collected`, containing `record.json`
and a hash-qualified raw-bundle reference. `COLLECTION` contains `collection.json`
and game subdirectories. They are different inputs; a collection itself is not
one movie. A takeover branch contains `branch.json`; its individual recordings
are below `sessions/`.

## 1. Collect games and read the results

```sh
./produce_death.py --games 10 --turns 1000000 --timeout 250
./produce_death.py --seeds 1001 1002 1003 --turns 200000 --timeout 120
./produce_death.py --check-build
```

The default is three fresh random-race/class characters, normal Borg, absolute
turn cap200000, wall limit120seconds per character and stall limit30seconds.
Race/class combinations can repeat. The seed list and limits are frozen before
outcomes. Use `--seed N --games K` for consecutive uint32 seeds, wrapping at2^32;
`--seeds` supplies the explicit distinct list and cannot be mixed with those two
options. `--birth fixed` explicitly chooses Human Warrior. Future collection
uses normal mode; `--munchkin` exists only for an explicit new request or historical
reproduction, not the current owner default.

The first reached bound ends an attempt. A turn cap is not a minimum runtime,
and a timeout is not a death. At normal speed a standard move costs about ten
engine ticks. Rest can advance many ticks quickly in headless mode. Reports
show actual level, deepest depth, recorded kills and decisions so a large clock
value is not mistaken for good progress. A clock that advances in a bad cycle
can evade the unchanged-turn stall detector; liveness still needs inspection.

The collector continues after every successfully exported natural death and
ordinary wall/turn limit until all requested seeds are attempted. Ten attempts
do not guarantee ten deaths. Crash, stall, native/recording error or failed
export stops the batch, preserving earlier deaths and failure evidence. A real
death with an operational error is exported before stopping where possible.
There is no automatic seed/limit enlargement or unattended retry service.

At startup it prints requested source and actual matching build. Committed
source is pinned with `--ref`; dirty runtime changes are not included. It reuses
a complete compatible build when game/Borg source, carry ledger, tools, binary
and data match, even across documentation/launcher-only revisions. Otherwise it
offers a build before taking the validation lock. `--yes` accepts a needed build;
`--no-build` rejects a missing one. It does not pull remote changes. Corrupt or
incomplete cache entries are preserved and rejected.

Each attempt prints a status/progress block, report path, Watch and Play commands.
Deaths get distinct Dnnnn dossiers in `docs/deaths/`; analysis remains pending.
The final summary lists all deaths and `reports.md`. Interactive headings/status
are colored and runnable commands are cyan on separate lines. Pipes are plain;
`NO_COLOR=1` or `TERM=dumb` disables color. An already running script retains its
loaded layout.

```sh
./produce_death.py --commands artifacts/death-workflow/collections/TIMESTAMP
./produce_death.py --reports artifacts/death-workflow/collections/TIMESTAMP
```

Short reports contain current/highest level, max depth, recorded kills, XP,
gold, ticks, decisions, wall time, seed, version and actual outcome. Living
summaries omit tactical maps. Native character dumps use the run's own frozen
engine on an isolated copy of its living snapshot, without Borg activation,
movement or rest. Snapshot date and unchanged counters are checked. Unsupported
or missing counters are unavailable, not zero. Kills use native this-character
`pkills`, not all-lives lore; unseen non-unique kills may be absent. Rest counters
are **player rest turns**, not engine ticks. Do not resurrect a dead save just
to obtain a sheet. Generated reports and full saves remain outside Git.

## 2. Named visible/headless play

```sh
./start-borg.sh BorgPlay
./run-borg-headless.py BorgPlay --turns 10000 --timeout 300
```

Both use `build/native/game/lib/save/NAME`, share same-character locks and retain
five rolling backups before replacing an existing character. Different names can
coexist. Visible play asks you to create the character and start the Borg:
Ctrl+Z, confirm if asked, then z. Ctrl+X saves/exits. Headless play starts the Borg
automatically; new characters default to random race/class and random seed,
while loaded saves keep their identity and native RNG. Headless `--turns` means
**additional** ticks, unlike the collector's absolute new-game cap. It stops at
the next command boundary, so the actual advance can overshoot the target.
A failed headless run does not publish over the original named save.

Both support `--seed N` for new characters and `--from-backup` / `--from-backup=2`
for an explicitly selected slot1–5. Restoration preserves the current save in a
timestamped before-restore file. Restoring a backup is a new continuation, not a
claim that the dead character survived. Sessions print their evidence paths;
natural deaths receive dossiers. Launchers check build freshness separately from
replay: replay does not silently replace a historical build with current fixes.

## 3. Watch, verify, seek and inspect

```sh
./replay-borg.py docs/deaths/D0010-collected --from-turn 6255
./replay-borg.py RECORDING --verify-only --timeout 600
./replay-borg.py RECORDING --view-only --from-turn 6255
./replay-borg.py RECORDING --view-only --dump-turn 6255
```

Normal replay reconstructs from the original session start, verifies it, then
seeks within the **original** recorded frames. `--from-turn` and `g` are absolute
engine ticks, not offsets before death. Default inspection starts around the last
105ticks. Verification may take substantial time and waits for the shared lock;
seeking in the viewer is not another engine rerun. `--output NEW_DIRECTORY` keeps
verification evidence separately; `verification.json` reports success or first
difference. Full replay rejects incomplete recordings. A finalized timed-out
movie can be watched with `--view-only`; that flag does not claim verified execution.
A legacy seed-only death cannot acquire missing input history retrospectively.

Focus/click the terminal region before pressing keys. Escape returns from a
panel to the screen; **q exits**.

| Keys | Viewer action |
| --- | --- |
| Space | Play/pause original frames |
| Right/Left or l/h | One frame forward/back |
| Down/Up | Ten frames forward/back |
| PageDown/PageUp | Hundred frames forward/back |
| Home/End | First/last frame |
| g | Seek absolute engine tick |
| i / e | Player-known pack / equipment and quiver |
| s | Learned spells **and prayers**, costs, failure and availability |
| p, then Tab | Cycle Borg actions, all Borg notes, visible game messages, fighting log |
| d / m | Explicitly labeled diagnostic gear / monsters |
| v or Escape | Original game screen |
| j/k | Scroll panel/log history |
| [ / ] | Pan horizontally |

Player inventory uses native descriptions, slot letters, quantities, fuel,
charges and inscriptions with the character's knowledge; unidentified appearances
stay unidentified. Equipment/quiver is separate. Diagnostic actual item kinds or
unseen monsters are labeled separately and were not supplied to Borg decisions.
Spells/prayers distinguish learned/forgotten, book access, mana, blindness/darkness
and confusion. Ready is not a prediction of successful targeting/effect; low-mana
overexertion requires ordinary confirmation. Unknown spell names are not invented.

Older movies may lack inventory/spell fields. A normal interactive replay can
build a separate observer from the historical source and frozen instrumentation,
then require every old field/stream/save to match before caching **only the new
observation fields**. View-only uses an existing qualified cache or explicitly
reports missing information; it never builds or guesses it. See the detailed
[legacy recovery contract](RECORD-REPLAY.md#player-readable-inventory).

### Fighting log and timing

```sh
./replay-borg.py docs/deaths/D0010-collected --combat-log --from-turn 6255
./replay-borg.py docs/deaths/D0010-collected --view-only --combat-log
```

First command qualifies a separate historical native message/HP/speed observer;
second reuses its cache. `p`, then Tab selects Fighting log. It includes native
hits/misses/shots/spells/consumable messages before coalescing and actual player
HP-loss rows/source. A message and corresponding HP row are one event, not two
hits. Borg notes express intentions, not confirmed damage; repeated HIT_BY notes
cannot provide a hit count. Visible-top-line history also cannot recover every
unrecorded blow. Basic `p` log sources need no helper; full combat recovery requires
a complete movie. `--verify-only --combat-log` prepares it without a viewer.

Logs honor recorded frame/phase cutoffs. Legacy notes lacking phase appear only
after that tick completes; combat events appear at their next original frame.
No later same-tick event is exposed early. The speed header shows actual recorded
energy gain and move cost. Engine ticks, decisions, command inputs, frames and
player actions are distinct. Normal speed110 gains10energy/tick, a standard move
costs100 (~10ticks); speed111 gains11 (~9.1ticks). Actions can have different costs.
Existing danger/flow notes are shown where captured; this is not a complete
introspection of every Borg candidate score or a recomputation under today's AI.

## 4. Play your own alternative, including living runs

```sh
./play-from-replay.py docs/deaths/D0010-collected --from-turn 6255
./play-from-replay.py RECORDING --from-turn latest
./play-from-replay.py RECORDING --from-turn 6255 --prepare-only
./play-from-replay.py --resume BRANCH_DIRECTORY
```

Numeric selection is the first **living player command** at/after that tick.
`latest` is the first living command at the last recorded living-command tick,
not an arbitrary final displayed frame. The actual selected turn/frame is printed.
Out-of-range requests are refused. Prefix reconstruction starts from the original
start, not a guessed mid-run state. First use offers a separate frozen-source
helper; `--yes` accepts and `--no-build` requires one already qualified. Declining
cancels. Current gameplay fixes are not imported into old recordings.

The selected original prefix must match input/action/clock/file/trace/frame/RNG
and native-save fingerprint before manual launch. `--timeout` bounds this replay
(default600s), not the subsequent human game. Finalized consistent wall-timeout
movies can qualify their selected prefix; original evidence remains incomplete.
Active recordings, crashes, changed files and malformed/divergent prefixes fail.

The helper then loads the native save in isolated X11 play with Borg **off**.
Click the game, play normally, Ctrl+X to save/exit. Your save and full recording
remain under the printed branch's `sessions/`; resume that branch later without
rebuilding or changing its helper. A failed GUI launch after qualification can
be retried with `--resume`. `--prepare-only` needs no display; visible play checks
fonts/display before expensive preparation. `BORG_PLAY_FONT`, then
`ANGBAND_X11_FONT_0`, override automatic readable font selection. WSLg may need
its existing Type1 font directory registered; no font installation is performed.

Native load can compact monster IDs; the Borg's private memory/stack is not
restored. Human choices alter RNG and later events. This is a recorded diagnostic
alternative, not an independent Borg life, exact continuation of the old AI's
RAM, guaranteed rescue, or evidence of population survival improvement. Manual
session movies can be opened with replay-borg.py, but are excluded from automatic
independent Borg-death counts and the owner queue.

## 5. Queue, feedback and investigations

```sh
./replay_next_death.py --list
./replay_next_death.py --list --json
./replay_next_death.py --combat-log
./replay_next_death.py --case D0010 --view-only
./replay_next_death.py --mark-reviewed D0010
./replay_next_death.py --mark-unreviewed D0010
```

Selection uses oldest available encounter family not explicitly owner-reviewed.
Missing/corrupt/legacy recordings stay visible in the list. Only qualified family
relationships group repeated encounters; repeated replays are not new lives.
Viewer exit, saving a human branch, and agent assessment do not mark owner review
complete or close an investigation. Local reversible overrides are shared across
worktrees in primary artifacts, separate from committed owner receipts.

Follow [FEEDBACK.md](../deaths/FEEDBACK.md): retain owner observations on the
case's linked GitHub investigation; check the actual source/build and knowledge,
resources, routes and action order; distinguish observation, hypothesis and
native proof. Complete [REVIEW-TEMPLATE.md](../deaths/REVIEW-TEMPLATE.md), including
timeline, brainstorming/competing explanations, costs, priorities and next tests.
Update assessment actions and aggregate registry; acknowledgement alone is not
resolution. New original-C defects route through the source tracker and separate
consumer carry. A policy idea is not automatically a bug or authorised tactic.
Prospective implementation/survival **and progress** evaluation is separate under
[the improvement workflow](DEATH-IMPROVEMENT-WORKFLOW.md).

Curator commands (the collector already performs initial export):

```sh
python3 tools/record-death.py --run RUN --output docs/deaths/DNNNN-collected --id DNNNN --issue ISSUE_URL --investigation ISSUE_URL
python3 tools/aggregate-deaths.py --root docs/deaths
```

Exporter `--repeat`, `--reference` and `--window` retain qualified provenance;
do not use them to inflate independent counts or relabel a death with newer code.
Dossiers contain actual identity/build/seed/options, selected logs and original
pre-death frames at offsets5/12/25/40/60/82/105, assessment/report and linked actions.
A too-short movie honestly lacks an earlier window. Initial export is unassessed;
full raw movies/saves/builds stay outside Git. Dossiers need ordinary curation
and repository review, not automatic publication of arbitrary owner files.

## 6. What a complete recording contains

A seed alone is insufficient. `borg-recording@1` freezes the actual engine,
source/build/tool/data identity, starting save or seed, initial user files,
locale/timezone and relevant external inputs. A matching saved-state parent link
does not restore previous private Borg memory. Important files:

| Files | Meaning |
| --- | --- |
| `recording.json`, `engine`, `data/`, `instrumentation/`, `initial-user/`, optional `origin.save`, `timezone.tzif` | Hashed identity and frozen execution inputs |
| `events.txt` | Native terminal poll/events, wait/peek/take, geometry/queue semantics |
| `actions.txt` | Actual Borg key stream with tick/decision/phase context |
| `clock.txt`, `file-inputs.txt`, `file-input-*.bin` | Native calendar values and preference/configuration bytes when read |
| `frames.jsonl.gz` (or uncompressed JSONL) | Indexed full/delta logical cells/colors and actual phase/state observations |
| `trace.txt` | Native states, goals/notes, outcomes and RNG observations |
| `start.save`, `recent.save`, `world-fingerprint.save`, optional `final.save` | Native save anchors, where produced |
| `record-complete.json`, `result.json` | Recorder counts/finalization, real outcome/limits/errors |

Frame rows include index, phase, tick, decisions/input count, geometry,
player/resources/progress, game/Borg RNG, full/delta cells and palette; native
world fingerprints are recorded at command/input boundaries (not every tick).
Optional diagnostic/inventory/spell observations inherit across delta frames.
Cells are `[linear_offset, codepoint, attribute]`. The native source is the exact
format definition; do not treat this overview as a replacement parser spec.

Bundle files/native saves use SHA-256; serialized world fingerprints use FNV1a64.
Validation checks completeness, path containment and hashes. Replay supplies only
recorded external input; original game/Borg code computes decisions/RNG. It compares
all streams, file-read bytes, frame fields and save anchors, reporting first
difference and nonzero failure. It never forces matching by injecting decisions
or resetting RNG mid-run. Observation extension certificates identify the added
fields; all previously recorded fields remain strict. Interrupted/missing/corrupt
inputs and unknown schemas are not silently upgraded to VERIFIED.

## 7. Implementation map and storage

| Source | Responsibility |
| --- | --- |
| `start-borg.sh`, `tools/start-recorded-borg.py`, `tools/borg-saves.py` | Visible launch/build offer, isolated recording, named-save publication/backups/locks |
| `run-borg-headless.py` | Bounded named continuation and safe save/death delivery |
| `produce_death.py`, `tools/death-workflow.py` | Collection/build compatibility, manifests, commands and owner queue helpers |
| `tools/measure-native.py`, `tools/native-observer.c`, `tools/native-observer.h` | Frozen source export/build, bounded native execution/watchdog and state/trace/snapshot hooks |
| `tools/borg-recordings.py`, `tools/native-replay.c`, `tools/native-replay.h` | Frozen bundle lifecycle, external-input recording/replay and logical frames |
| `replay-borg.py`, `tools/borg-replay-viewer.py` | Strict verifier, indexed delta-frame viewer and panes/log cutoff rules |
| `tools/native-inventory.h`, `tools/native-spells.h`, `tools/replay-inventory.py` | Native player-known observations, qualified legacy extensions |
| `tools/replay-observation.py`, `tools/native-combat.h` | Separate qualified combat/wide-geometry recovery |
| `play-from-replay.py`, `tools/borg-takeover.py`, `tools/native-takeover.h` | Strict command-prefix capture, branch identity, frozen-helper manual save/load/resume |
| `tools/run-report.py` | Versioned save parsing, native character sheet, per-run/collection reports |
| `replay_next_death.py` | Explicit owner-review queue CLI |
| `tools/record-death.py`, `tools/aggregate-deaths.py` | Curated native death provenance and noninflating registry |
| `tools/measure-cohort.py` | Separate pinned measurement cohorts; not a substitute for an experimental design |

`tools/test_*.py` contain deterministic contract/negative controls; native/PTY
qualification examples and limitations are recorded in the linked detailed docs.
Preserve Angband/tool licensing and credit when reusing source.

| Location (primary checkout unless noted) | Contents |
| --- | --- |
| `artifacts/recordings/`, `artifacts/headless/runs/` | Visible/named-headless raw recordings |
| `artifacts/death-workflow/collections/` | Collector plans, individual movies and reports |
| `artifacts/death-workflow/builds/`, `artifacts/headless/builds/` | Pinned observation builds |
| `artifacts/replays/` (checkout-local) | Default ordinary replay certificates/output |
| `artifacts/death-workflow/inventory/`, `artifacts/death-workflow/combat/` | Qualified observation extension caches |
| `artifacts/death-workflow/takeover/builds/`, `takeover/branches/` | Historical helpers and isolated human alternatives |
| `artifacts/death-workflow/owner-reviews.json` | Local reversible review overrides |
| `docs/deaths/` | Curated dossiers, feedback and generated aggregate |

Always use printed paths/receipts as authoritative; older versions may use
`artifacts/replays/` or separate validation/archive directories. Preserve old
executables, frozen observer sources, native saves, manifests and required raw
evidence. Later verifiers depend on earlier builds. An archive's existence does
not establish coverage: verify file membership and SHA-256 before retiring anything.
Do not archive whole disposable worktrees as a substitute for selecting evidence.

## 8. Concurrency, portability and troubleshooting

Collection, native verification, helper builds and heavy headless work share
`/tmp/angband-validation.lock`. A collector currently holds it for its entire
batch. An original-frame `--view-only` viewer can run in another terminal at the
same time; a verifier or uncached observer build may wait. Cached combat data
can be viewed with `--view-only --combat-log`; uncached recovery cannot. Opening
the same named character or manual branch twice is separately locked. Do not kill
another task's gate or alter its evidence to make a viewer start sooner.

Linux/WSL is the tested runtime: Python3.12, standard-library curses/ctypes/fcntl,
Git, CMake, native compiler, Perl/util-linux and X11 development/runtime support.
Historical native binaries also need a compatible Linux ABI. X11/display/fonts
are needed for visible/manual play, not movie inspection or prepare-only takeover.
No web service, database or scheduler is needed.

| Symptom | Check / next action |
| --- | --- |
| Viewer ignores keys | Focus/click its terminal; Escape returns from panels, q exits |
| Long pause before viewer/manual launch | Original-start verification/helper compilation/lock wait; use view-only for inspection, not a fake certificate |
| Incomplete timeout movie rejected | Watch with view-only; manual takeover permits only validated finalized timeout prefix |
| Inventory/spells/combat unavailable | Need captured fields or separately qualified historical observer; view-only cannot build one |
| Missing legacy observer source | Restore the SHA-qualified archive/catalog; keep missing-data failure explicit |
| Missing raw dossier recording | Restore bundle; pass its direct path after relocation, do not rewrite the original identity |
| Valid old large-window recording fails | Separate compatibility reader; dimensions <=4096 each and <=1048576cells; all original data still must match |
| Small/missing X11 font | Check printed selection/explicit override; resume already prepared branch after display repair |
| Alleged long-run progress from ticks alone | Compare reports/depth/kills/resources and inspect trajectory; rest or bad loops also advance time |

Replay fidelity, causal diagnosis and measured AI improvement are separate claims.
Keep their receipts and limitations separate in every report.
