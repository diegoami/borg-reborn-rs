# Local Rust recording, replay and death analysis

Run these commands from the repository root. This adapts the C toolkit at
`a4084032485a7c7f29153d0b37ce63dede3e4d5a`, with its guide/handoff at the
separate documentation commit `a9fb21bc006f71b639ae38b3032583271411581e`.
It keeps borg-rs's gameplay, issue ownership and historical evidence. See the
[implementation checkpoint](TOOLKIT-IMPLEMENTATION.md) for executed checks,
failed attempts, exact pins and remaining qualifications. Imported source and
licensing are recorded in [IMPORTED.json](../tools/IMPORTED.json).

## Play or advance a named character

```sh
./start-recording.sh MyBorg --seed 42
./run-borg-headless.py MyBorg --turns 10000 --timeout 300
./start-recording.sh MyBorg --from-backup=2
```

Visible play loads/creates the character and leaves the Borg off. Create the
character normally; Ctrl-Z, confirm if prompted, then z starts the Rust Borg.
An input key normally interrupts it at the next Borg input check; Ctrl-X saves/exits. Longer visible interruption tests are not uniformly qualified; see the retained failures in the checkpoint. Focus the game window when sending
keys. Font selection actually probes X11 loading before build/launch. Precedence:
`BORG_PLAY_FONT`, `ANGBAND_X11_FONT_0`, `ANGBAND_X11_FONT`, automatic large Courier,
10x20, 9x18, fixed. Explicit invalid fonts fail without silently substituting one.
No font installation or owner display manipulation is performed.

Both launchers use `player-cemetery/save/NAME` and the same per-character lock.
`BORG_PLAY_DATA_DIR` selects another profile, including its save/user/recordings
folders. Different names can be open concurrently. A second launcher for the
same name fails. There are five rolling **pre-launch** save backups under
`save/backups/NAME/save.1` through `save.5`. Restoration is explicit, validates the
selected living backup, and preserves the current save as `before-restore-*.save`.
A restore creates an alternative continuation, not a claim of original survival.

Headless play starts the Rust Borg automatically. Its `--turns` is an
**additional** engine-tick budget from the loaded save; it stops at a valid
observable command/key boundary and may overshoot. A fresh character defaults
to native seeded `*` race/class choices and a recorded random uint32 seed.
`--seed`, `--race` and `--class` allow explicit new-birth controls. A loaded
character keeps native identity and complex RNG, and starts a fresh private
BorgState. A supplied/requested seed is only continuation provenance. Native
saves do not serialize the Borg's map, goals, caches, local RNG or call stack.

Living successful saves are published atomically after validation. Outside
writers cause refusal. Failed and finalized timed-out attempts preserve the
named original and print retained evidence. Every session has a unique printed
recording path; paths are authoritative. No cleanup job removes old builds,
movies, saves, failures or helpers.

## Collect bounded games and read reports

```sh
./produce_death.py --check-build
./produce_death.py --yes --games 10 --seed 1001 --turns 200000 --timeout 120
./produce_death.py --no-build --seeds 11 29 47 --turns 20000 --timeout 120
./produce_death.py --commands harness/out/death-workflow/collections/COLLECTION
./produce_death.py --reports harness/out/death-workflow/collections/COLLECTION
```

| Option | Meaning/default |
| --- | --- |
| `--games N` | 1–1000 attempts; default 3; not a requested death count |
| `--seed N` | First uint32 seed; subsequent seeds increment with uint32 wrap |
| `--seeds A B ...` | Explicit distinct seeds; excludes `--games`/`--seed` |
| `--birth random/fixed` | Native random menus by default; fixed = Human Warrior |
| `--turns N` | Absolute new-game engine tick cap; default 200000 |
| `--timeout SECONDS` | Per-character wall limit, default 120; excludes builds/lock wait |
| `--stall-timeout SECONDS` | Unchanged flushed native-frame world clock (legacy key fallback), default 30 |
| `--keys N` | Additional Borg event threshold including idle events; finish current command, default 50000 |
| `--checkpoint-every N` | Native save schedule in engine ticks; default 1000 |
| `--ref COMMIT` | Committed engine/Rust/data pin, default HEAD; dirty gameplay excluded |
| `--yes` | Accept a needed frozen native build |
| `--no-build` | Require compatible existing native build; never silently compile |
| `--check-build` | Check compatibility; no games/builds or waiting lock; 0 available, 1 unavailable |
| `--build CACHE` | Explicit complete toolkit cache with prepared.json; ordinary C builds refused |
| `--munchkin` | Explicit opt-in only; new ordinary collections default off |
| `--output DIR` | New output below harness/out or BORG_DEATH_WORKFLOW_DIR |
| `--commands COLLECTION` | Reprint retained commands; no new games or builds |
| `--reports COLLECTION` | Backfill reports from existing evidence; optional frozen-engine formatting, no new lives |

The immutable `plan.json` freezes seeds, birth policy, limits and tool hashes
before outcomes. `collection.json` records every attempt and actual outcome.
Collection continues after **every successfully exported natural death** and
ordinary turn/key/wall limit. Tick/key thresholds stop at the next living top-level command; pending menu actions may finish beyond the threshold. Wall and stall bounds remain independent hard supervision limits. Operational/native/recording/export errors stop it,
retaining earlier successes and the failing attempt. A killed or inconsistent
timeout is an operational failure. No cap/seed/budget is enlarged after outcomes.
Ten attempts need not produce ten deaths. This bounded collection is not a
survival-rate experiment. Repeated race/class pairs are possible.

Each attempt prints its report and standalone Watch/Manual commands, also
stored without ANSI escapes in `commands.txt`. `reports.md` compares the
collection and lists every exported death. Short reports contain current/highest
level, deepest dungeon level, native this-life recorded kills, XP/gold, ticks,
Borg decisions, rest **player turns**, wall seconds, seed and source/tool/build
identity. Unavailable values are never invented zero. Unseen non-unique kills
may not be recorded by native lore. A stale snapshot is labeled with its tick.
Living short reports omit tactical maps and monster situations.

A living native character sheet uses the frozen engine to load an isolated
snapshot, format it, and exit without Borg activation or a game action. Native
proof checks unchanged engine ticks, energy and full game RNG. The original
snapshot remains hashed and untouched. Dead saves are never resurrected for
formatting; unsupported/corrupt saves are reported unavailable. Reports and
formatter failures live outside the immutable raw bundle.

Build reuse includes engine + Rust sources, Cargo.lock, measurement adapters,
native observer overlay, toolchain/target/features/flags, binary and runtime
files. Launcher snapshots are separate from compatible native images. Requested
source and actual compiled source are both retained across documentation/tool
changes. Incomplete/corrupt exact caches are preserved and refused, never
refreshed over a historical dependency. A needed build is offered before the
shared lock or any game. Noninteractive callers explicitly use `--yes`.

## Verify and inspect the original movie

```sh
./replay-borg.py RECORDING --verify-only --timeout 300 --output harness/out/NEW-PROOF
./replay-borg.py RECORDING --from-turn 500
./replay-borg.py RECORDING --view-only --dump-turn 500
./replay-borg.py RECORDING --view-only --combat-log
./replay-borg.py RECORDING --verify-only --engine /path/to/other/game/angband --output harness/out/COUNTERFACTUAL
```

`--engine PATH` reruns the recording against a different built executable
instead of the one frozen in the bundle, to see whether another build plays
a historical life the same way. It needs an existing binary and an explicit
`--output` directory; it refuses `--view-only`, `--combat-log` and
`--play-from-turn`. It never produces a verified certificate: any divergence
from the original recording raises, with the first differing stream/row/field
in the error and retained in `--output`, and `--output/verification.json` is
never written to the bundle's own cached certificate.

A recording directory contains `recording.json`. A death dossier contains
`record.json` and a hash-qualified reference to that original raw recording.
A collection contains `collection.json` and is not itself a movie. Default
replay executes the original frozen Rust executable from its original session
origin, checks the declared channels/saves, then opens **original frames**.
A matching trusted local certificate can satisfy a cached default verification;
`--output NEW-DIR` or explicit `--timeout` forces actual execution. Existing
output directories are refused. Failures retain a negative certificate and
first differing stream/row/tick/field where available. Certificates are local
receipts, not signatures or independent attestations.

`--view-only` runs/builds no engine, takes no native lock and makes no rerun
claim. Finalized consistent timeout movies can be inspected this way. Full
verification refuses incomplete originals. Active/crashed/partial/malformed
recordings do not become complete merely because some frames can be read.
All finalized frame data is strict UTF-8/JSON; corrupt final tails are not hidden.

| Keys | Original-movie action |
| --- | --- |
| Space | Autoplay/pause |
| Right/Left or l/h | One frame forward/back |
| Up/Down | Ten frames back/forward |
| PageUp/PageDown | Hundred frames back/forward |
| Home/End | First/last frame |
| g | Seek absolute engine tick |
| i / e | Player-known inventory / equipment and quiver |
| s | Learned spells **and prayers**, cost/failure/book/mana/status |
| p, Tab | Borg action notes / all notes / game messages / native fighting log |
| d / m | Explicit diagnostic actual-kind gear / visible and unseen monsters |
| v or Escape | Return to game screen |
| j/k, [ / ] | Panel scroll, horizontal pan |
| q | Exit; does not mark owner review complete |

Inventory uses native known descriptions, inscriptions, fuel/charges, slot
labels and weights. Unknown appearances stay unknown. Native spell readiness
is separate from the Borg's private tactical admission and does not guarantee
success. Diagnostic actual kinds/unseen monsters are output only and never
supplied as additional AI inputs. Legacy missing observations remain unavailable
unless a separate original-source extension passes every pre-existing stream,
field and native-save comparison. View-only never compiles to fill gaps.

The native fighting log records messages before coalescing, actual HP loss/source,
speed, energy gain/tick and movement cost. Nearby same-tick HP-loss rows are grouped with their message for readability;
they are not additional hits or proof of a causal association from timing alone. Borg HIT_BY/action notes are intentions/observations, not
an exact native hit count. Frame/phase cutoffs prevent later same-tick events
appearing early. Missing legacy combat remains explicitly unavailable.

Engine ticks, cumulative player energy, player rest turns, Borg think decisions,
returned key events and terminal frames are different units. At speed110, gain10
energy/tick and a standard move costs100energy (about10ticks); other actions,
rest, haste and slows differ. GUI's historical `decisions` field counts returned
hook events; the toolkit's `borg_decisions` and `borg_rng` are separately sampled
Rust observer values. Tick count alone is not progress.

## Play a human alternative

```sh
./play-from-replay.py RECORDING --from-turn 500 --prepare-only --yes --output harness/out/MY-BRANCH
./play-from-replay.py RECORDING --from-turn latest --yes
./play-from-replay.py --resume harness/out/MY-BRANCH
# Existing compatible interfaces:
./replay-borg.py RECORDING --play-from-turn 500 --prepare-only --yes
./play-branch.py harness/out/MY-BRANCH
```

Numeric selection chooses the first living native command at/after that absolute
tick. `latest` chooses the first command at the last living command tick. The
actual tick, key/frame boundary, source/helper/save identity are retained. An
out-of-range request, active/crash recording or divergent prefix is refused.

First use offers a **separate historical-source helper**; `--yes` accepts and
`--no-build` requires one already available. Its observer/capture changes are
explicit and qualified against the original. Reconstruct from the original
birth/load/input history, never by substituting recorded AI actions or reseeding
at a later tick. Every original prefix stream and recorded world-save fingerprint
must match. A finalized consistent timeout can qualify a prefix but never gets
a full-original replay certificate. Legacy headless prefixes also require an original save anchor at the selected
command; missing anchors cannot be retroactively inferred. Missing historical observer sources are a
blocker; an archive filename alone is not proof of its contents.

Visible play probes display/font before expensive reconstruction. `--prepare-only`
needs no display. If display/font loading later fails, resume the already prepared
branch; do not repeat prefix reconstruction. Resume hash-checks and reuses that
exact helper even after current source/tool changes. Ctrl-X saves the branch.
Each manual session is independently recorded beneath the branch profile and can
itself be replayed. The native load validates player fields and complex game RNG
with Borg off before allowing commands. Native loading can compact/rebuild engine
structures. It does not restore private Borg memory or guarantee uninterrupted
future equality. Human choices alter later RNG/events. Manual alternatives are
excluded from independent Borg lives, death collection and owner queue counts.

## Death queue and diagnosis

```sh
./replay_next_death.py --list --json
./replay_next_death.py --case CASE_ID --view-only
./replay_next_death.py --mark-reviewed CASE_ID
./replay_next_death.py --mark-unreviewed CASE_ID
python3 tools/aggregate-deaths.py
```

Default selection is the oldest available owner-unreviewed qualified encounter
family. Missing/legacy/corrupt cases remain listed honestly. Explicit review
receipts are reversible and shared through the primary workflow directory
across worktrees. Viewing, preparing/saving a human branch, agent assessment and
technical issue closure do not complete personal owner review.

New deaths get separately allocated dossiers with actual original source/build/
config/tool/seed identity, native logs, snapshots and original pre-death windows
5/12/25/40/60/82/105 **engine ticks** where available. Missing exact windows remain
missing; nearby anchors carry their actual tick/phase, with no interpolation.
Raw cause text stays intact. Initial export is unassessed. Replays and manual
alternatives never add independent starts to the registry.

Use [FEEDBACK](deaths/FEEDBACK.md) and [REVIEW-TEMPLATE](deaths/REVIEW-TEMPLATE.md):
whole trajectory, actual player-known stock/spells/routes, early choices, fatal
sequence, competing explanations and prioritized next evidence. Inspect attempted
walking retreat after consumables expire; no scrolls does not mean no legal
escape attempt. Distinguish observation, source hypothesis, causal native proof
and proposed policy, with costs/confidence. Follow borg-rs's
[C-consumer adoption contract](C-CONSUMER-ADOPTION.md) for issue ownership; no
automatic source issue, tactic, C carry, rescue claim or policy experiment follows
from collection or diagnosis. Survival **and progress** need a separately frozen
prospective comparison, not one rescue or this smoke qualification.

## Storage, locks and dependencies

| Location | Contents |
| --- | --- |
| `player-cemetery/` or BORG_PLAY_DATA_DIR | Named save/user/backups and raw session recordings |
| `harness/out/death-workflow/` or BORG_DEATH_WORKFLOW_DIR | Collections, local review receipts, observer caches and historical helper builds |
| `build/death-workflow/` | Immutable committed-source/native-build/tool snapshots; preserve dependencies |
| Branch `runtime/`, `capture/`, `profile/recordings/` | Exact helper, qualified native snapshot and separately recorded manual sessions |
| `docs/deaths/` or BORG_DEATH_RECORDS_DIR | Curated dossiers; use a private override for tests |
| New replay `--output` or retained temporary path | Positive/negative certificates and actual rerun outputs |

| Lock | Scope |
| --- | --- |
| `/tmp/angband-validation.lock` | Heavy native build/headless/replay/prefix/formatter work; prints waits |
| Profile `save/.locks/NAME.lock` | Entire named session, including child/finalization |
| Prepared helper/cache lock | Allocation/qualification without overwriting failed artifacts |
| Registry lock / local owner-review lock | Sequential dossier IDs and reversible review receipts |

Movie-only viewing can coexist with collection. Different named visible games
can coexist. Human sessions do not hold the heavy lock for the entire game.
Never kill another lane's process or gate; only owned child processes are signaled.
There is no cleanup service. Long native movies and retained raw/compressed frames
cost disk space; preserve historical builds and verified source bundles.

Linux requirements: Python3.12+ (curses, fcntl, ctypes), Git, CMake, a C compiler,
Cargo/Rust and X11 development support for builds; X11/fonts for visible/manual
play; xdotool/Xvfb only for GUI qualification. Frozen replay/view-only need no
Rust rebuild. Terminal resource bounds are positive dimensions, at most4096each
and1048576cells; replay input queues are1–65536 with call/queue validation.
The headless frontend remains its native80x24; GUI logical movies support real
wide windows. These tests do not establish cross-platform bit equality.

For this session Rust/Cargo were absent and installed only under `/tmp`:

```sh
export PATH=/tmp/borg-cargo/bin:$PATH
export RUSTUP_HOME=/tmp/borg-rustup
export CARGO_HOME=/tmp/borg-cargo
```

Use your normal compatible toolchain in a persistent installation. Replays keep
their frozen executable/data; no maintenance command replaces an old engine.

| Problem | Action |
| --- | --- |
| Missing display/font | Fix DISPLAY/explicit font, then resume the prepared branch |
| Shared-lock wait | Inspect the printed lock scope; view-only can run without it |
| Incomplete timeout | Use view-only or strict prefix qualification, never full-replay claims |
| Missing raw movie | Restore the hash-qualified bundle; dossier summaries cannot recreate input history |
| Legacy observations absent | Restore verified original sources and qualify a separate helper; do not guess |
| Stale/corrupt build cache | Preserve and inspect it; do not refresh historical dependencies |
| Replay divergence | Read the retained certificate/first difference; original data stays untouched |
| Keys ignored | Focus the terminal/game, Escape backs out of panes, q exits the movie |

Rust self-replay means the same frozen Rust execution reproduces its recorded
channels. C/Rust parity is a different pinned comparison. Neither establishes
whole-private-state equality, an unassisted win or population survival benefit.

## Format evolution and verification scope

The target retains `borg-rs-headless-recording@1` and
`borg-rs-gui-recording@1`; these are distinct from the C donor's
`borg-recording@1`, which is rejected as a Rust recording. The compatible
`toolkit: borg-rs-toolkit-overlay@1` headless extension declares the native movie
channel, terminal/clock/file/action/combat streams, native footer and frozen
observer sources. Unknown toolkit versions are rejected. All new extension files
are required, hash checked, and compared during full replay; compressed and raw
movies must agree losslessly. Frozen build identity, runtime input binding and
original save identity are checked separately. GUI channel availability is
recorded and existing original channels remain strict.

Earlier Rust verifiers do not qualify the new optional channels. Their receipts
remain historical; current cache acceptance requires the current verifier hash
and unchanged recording-manifest hash, forcing fresh qualification for this
workflow. No original movie is rewritten or automatically upgraded. A legacy
movie can still be watched with its declared missing fields; a separate helper
may add only named observations after every old field/channel/save matches.
Legacy arbitrary takeover without an original selected-save anchor is refused.
Missing frozen legacy source or raw evidence remains a reproducibility blocker.
