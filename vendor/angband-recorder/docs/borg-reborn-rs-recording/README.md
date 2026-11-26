# Recording borg-reborn-rs games

The owner's workflow (set up 2026-11-17) for recording, replaying, opening and
watching games of the consumer borg-reborn-rs with this recorder. It runs in a
separate local build: nothing here changes borg-reborn-rs or its pin, and
recordings, saves and builds stay outside Git.

The scripts are in this directory. `S` below means this directory, e.g.
`S=~/projects/angband/angband-recorder/docs/borg-reborn-rs-recording`.

| Script | What |
|---|---|
| `setup.sh [REV]` | Makes the recording build of `REV` (default `origin/main`, fetched): a detached worktree of borg-reborn-rs at `$BRR_BUILD` (default `~/angband-builds/borg-reborn-rs-recording-<REV>`), the return settings in its `harness/borg.txt`, and a headless game with the recorder on in `build/rec` (`-DSUPPORT_BORG_RECORDING=ON`), and the `~/angband-builds/current` link. |
| `record.sh` | Records a game and shows it live in the terminal. |
| `angband-reborn-record` | Plays a game **in a window**, recorded from startup (the command you run; installed by `install-launcher.sh`). |
| `install-launcher.sh [BUILD]` | Builds the game with X11 and the recorder for a build, and installs `angband-reborn-record` from it. |
| `rewind.sh` | Shows the last turns before a game's end and names (or opens) the save just before them. |
| `run-env.sh` | The parameters and recording settings as environment variables; the other scripts source it. |
| `replay.sh` | Replays a recording and compares it with the original. |
| `open-save.sh` | Opens a copy of a save in an X11 game window. |
| `watch-recording.py` | Plays a recording's screens in the terminal, also live (`--follow`). |

Environment: `BRR_BUILD` (the recording build; when unset, the build `~/angband-builds/current` points at) and
`BORG_REBORN_RS` (the main checkout, default `~/projects/angband/borg-reborn-rs`).

## The current build

`~/angband-builds/borg-reborn-rs-recording-88d98537`, borg-reborn-rs commit
`88d98537` (made 2026-11-21), recorder v0.3.0, executable SHA-256
`4d4642c0a77eaa7ee4752b00147f9e5a87a1834c985435431aa356b54ff0fe21`. Its
recordings end with the `final` frame. `~/angband-builds/current` points at it:
the scripts use the build that link names when `BRR_BUILD` is not set, and
`setup.sh` moves the link to every build it makes. `export BRR_BUILD=DIR` still
overrides it (e.g. to replay with an older build).

Older builds, kept to replay their recordings (check `ls ~/angband-builds` for any others):

| Directory | Commit | Recorder | Executable SHA-256 |
|---|---|---|---|
| `~/angband-builds/borg-reborn-rs-recording` | `f02cd294` | v0.2.0 (no closing frame) | `57b1885509ee88024b670845c137ca62098fa92af1c3e049eb68c862621b8cb0` |
| `~/angband-builds/borg-reborn-rs-recording-v0.3.0` | `78fb330f` | v0.3.0 | `a8ad694bd5423ad3b1d6a90e6d0daabba2a012a3655469af6e51fcabc68b51e6` |

The recorder ties a recording to its executable's hash: **don't rebuild a build
whose recordings you want to replay**. Make a new one instead (see
*A new build, and how to get the latest*).

## The parameters

| Parameter | Setting |
|---|---|
| Attack preferences (blackguard melee 1600, blackguard spell 1400, druid spell 1700, mage device 1200, mage spell 2000, necromancer device 1200, necromancer spell 1800, paladin melee 1800, paladin spell 1200, priest spell 1700, ranger projectile 2000, rogue projectile 1300, rogue thrown 1600, warrior melee 1800, warrior projectile 1200, warrior thrown 1200) | `BORG_PARAM_ATTACK_PREFERENCE_<CLASS>_<FAMILY>_PERMILLE=N` in `run-env.sh` |
| Caution: heal when outmatched = 1 | `BORG_PARAM_CAUTION_HEAL_WHEN_OUTMATCHED=1` in `run-env.sh` |
| Plan: `errands_plan.txt` | `BORG_CORE=borg-rs`, `BORG_ERRANDS_PLAN=harness/plans/errands_plan.txt` in `run-env.sh` |
| New return policy: on | `borg_trip_policy = 1` in `harness/borg.txt` |
| Return with 1 pack space remaining | `borg_trip_pack_slack = 1` in `harness/borg.txt` |
| Return at 5 eligible unknown stacks | `borg_trip_unknown_stacks = 5` in `harness/borg.txt` |

To change a parameter, edit `run-env.sh` here (the `BORG_PARAM_*` settings) or the build's
`harness/borg.txt` (the `borg_trip_*` settings; `setup.sh` sets them). A misspelled or
out-of-range `BORG_PARAM_*` stops the game at startup instead of being
ignored. The parameter names are in
`borg-core/src/params.rs`; the values a run actually used are in its
`params-core.json` (`origin: "environment"` for yours).

`run-env.sh` also sets `BORG_TOOLKIT_NATIVE=1` (turns recording on),
`CHECKPOINT_EVERY=1000` (a save every 1,000 game turns, about 100 player turns; ~50 KB each) and `ANGBAND_BIN`
(`$BRR_BUILD/build/rec/game/angband`).

## 1. Record a game and watch it

```sh
$S/record.sh NAME SEED RACE CLASS [TURNS]
$S/record.sh gnome-mage-2 42 4 1            # e.g. a Gnome Mage, seed 42, 1,000,000 game turns
```

The game runs headless in the background and the terminal shows its screen
live, with a status line (frame, turn, depth, HP). The game is faster than the
view, so the view lags behind and catches up at the end. **Ctrl-C closes only
the view**: the game keeps running in its own session (it logs to
`~/angband-recordings/NAME.run.log`). When the view reaches the end,
`record.sh` waits for `run.sh` and prints the outcome. `NO_VIEW=1` records
without the view; `VIEW_DELAY=S` sets the seconds per screen (default 0.02).

`record.sh` runs this, which you can also run yourself:

```sh
source $S/run-env.sh                              # sets BRR_BUILD (the current build)
cd "$BRR_BUILD"
flock /tmp/angband-validation.lock harness/run.sh ~/angband-recordings/NAME SEED RACE CLASS TURNS menu,terminal-save
```

- `NAME`: a new directory under `~/angband-recordings`; an existing one is refused.
- `SEED`: a number. The game is deterministic: the same seed and settings
  give the same game. Text is read as 0 (`SEED` literally gave seed 0).
- `RACE`, `CLASS`: indices with the `menu` flag. Races: 0 Human, 1 Half-Elf,
  2 Elf, 3 Hobbit, 4 Gnome, 5 Dwarf, 6 Half-Orc, 7 Half-Troll, 8 Dunadan,
  9 High-Elf, 10 Kobold. Classes: 0 Warrior, 1 Mage, 2 Druid, 3 Priest,
  4 Necromancer, 5 Paladin, 6 Rogue, 7 Ranger, 8 Blackguard.
- `TURNS`: engine (game) turns, not player turns; 1,000,000 is about 100,000
  player turns.
- `menu,terminal-save`: use these flags. **Not `finalsave`**: it only saves a
  living character, so a death makes the engine exit with an error and leaves
  the recording unfinished (no `native/complete.json`).

What happens: `flock` waits for any other heavy Angband job holding the
lock, then `run.sh` sets up `NAME/`, starts the game headless and the Borg
plays alone until death, a win or the turn limit. Run by hand, the
terminal stays blank until it ends.

**Expected message at the end:** `Cemetery native tools do not match
death-tools-pin.txt; refusing to execute.` `run.sh` tries to export every
run to borg-cemetery, and the export refuses because the cemetery tools
checkout doesn't match `death-tools-pin.txt`. It doesn't mean the character
died, and it doesn't affect the game or the recording.

### What a run leaves

| File | What |
|---|---|
| `summary.json` | The outcome: `stop_reason` (`death`, `turns`), `dead`, `death_cause`, `game_turns`, `clevel`, `max_depth` |
| `native/` | The recording; `native/complete.json` means it finished properly |
| `final.save` | The game at the stop, alive or dead |
| `checkpoints/` | A save every 1,000 game turns (the file name has the turn) |
| `messages.log`, `keys.log`, `state.log` | Game messages (Borg notes start with `#`), keys, one state line per key |
| `trace-core.jsonl`, `params-core.json` | The Borg's decisions and the parameter values it used |
| `config.txt`, `process.json`, `stderr.log` | Run settings and hashes, process outcome, the last screen |

`native/frames.jsonl` ends with the end state when the build's recorder is v0.3.0
or later: a `final` frame for the headless runs here (a death, with `dead: 1`,
or the turn limit), or an `exit` frame for a run that leaves through the game's
own quit (e.g. a game window). Builds with an older recorder end a headless run
at the last command's frame, so the final turns, including a death, are missing
from their recordings.

A 1,000,000-turn run takes a few minutes and about **1.1 GB**, almost all of
it `native/frames.jsonl`.

## 2. Check that a recording replays

```sh
$S/replay.sh ~/angband-recordings/NAME ~/angband-recordings/NAME-replay
```

It reruns the recording's own command (from `NAME/process.json`) with the
recorded inputs fed back, and prints `identical` or `DIFFERS` for
`keys.log`, `state.log` and `final.save`. If the recording's executable has
moved, it uses `$BRR_BUILD`'s; the recorder refuses an executable with a
different hash. It unsets `BORG_ERRANDS_PLAN`: `run.sh` doesn't pass it to
the game, which reads the plan from `borg.txt`, and leaving it set breaks
the replay.

## 3. Go back to just before the end

```sh
$S/rewind.sh ~/angband-recordings/NAME              # the last 50 player turns
$S/rewind.sh ~/angband-recordings/NAME 200          # the last 200
$S/rewind.sh ~/angband-recordings/NAME 50 --open    # then open the save before them in a window
```

It plays the recorded screens from that point to the death (or stop) slowly
(`VIEW_DELAY`, default 0.3 s per screen), then names the nearest checkpoint
save at or before that point; with `--open` it opens a copy of it with
`open-save.sh`. With a save every ~100 player turns, the save is at most about
100 player turns before the view's start.

The screens are the recording itself, so they show exactly what happened. A
save opened in a window is a new game from that position: the Borg restarts
with fresh memory, so playing on is not guaranteed to end the same way.

## 4. Open a save in a game window

```sh
$S/open-save.sh ~/angband-recordings/NAME/final.save
$S/open-save.sh ~/angband-recordings/NAME/checkpoints/checkpoint-000010-t0000500003.save
$S/open-save.sh SAVEFILE COPYNAME     # a second, separate copy
```

It copies the save to `~/angband-play/COPYNAME/` and opens it with the X11
build in the main checkout (`~/projects/angband/borg-reborn-rs/build/watch`,
same engine). The original is never changed. Running it again with the same
name continues that copy.

In the game, look with `C` (character), `i` (inventory), `e` (equipment),
`M` (map), or play yourself. Ctrl-Z, then `z` starts the Borg; any key stops
it. For the Borg to use your parameters, `source $S/run-env.sh` first. The Borg
starts with no memory of the run, and play in this window isn't recorded.

## 5. Watch a recording

One screen, with the recorder's own tool:

```sh
cd ~/projects/angband/angband-recorder
cargo build --release --locked                                                    # once
target/release/angband-recorder view ~/angband-recordings/NAME/native --turn 500000
target/release/angband-recorder view ~/angband-recordings/NAME/native 43496       # by frame
```

Each call reads the whole frame file (about 2 s for 1 GB).

As an animation in the terminal (at least 80×25):

```sh
$S/watch-recording.py ~/angband-recordings/NAME/native
$S/watch-recording.py ~/angband-recordings/NAME/native --from-turn 500000 --to-turn 600000
$S/watch-recording.py ~/angband-recordings/NAME/native --delay 0.2                  # slower
$S/watch-recording.py ~/angband-recordings/NAME/native --every 10 --delay 0.02      # fast-forward
```

Add `--follow` to watch a recording that is still being written. Ctrl-C
stops. These only show recorded screens; `replay.sh` is the check
that the game reproduces.

## 6. Play in a window, recorded: `angband-reborn-record`

The headless scripts above have no window. To watch or play the Borg in an X11
window with the recorder on from process startup, use the launcher command:

```sh
angband-reborn-record CHARACTER            # a game window (default character: RustBorg)
angband-reborn-record --watch ~/angband-recordings/SESSION [--from-turn TURN]   # play the screens in the terminal
angband-reborn-record --frame ~/angband-recordings/SESSION [FRAME]              # one screen (default: the last)
```

It needs a desktop session (`DISPLAY`). Each run writes
`~/angband-recordings/<date-time>-<CHARACTER>.XXXXXX/native` and copies the
`borg.txt`, `errands_plan.txt` and `scary_monsters.txt` it used into
`borg-settings/` beside it. The character's saves and settings live in
`$BORG_REBORN_RS/.angband-reborn` (override with `BORG_PLAY_DATA_DIR`;
recordings with `BORG_RECORDINGS_DIR`), not in the recording. In the game,
Ctrl-Z then `z` starts the Borg and any key stops it; **Ctrl-X saves and
quits**.

A window build is not the headless build: `setup.sh` builds with X11 off, and
a recorder variable does nothing in a binary built without the recorder. So the
launcher has its own install, made from a build:

```sh
$S/install-launcher.sh                  # for ~/angband-builds/current (or give a build directory)
```

It builds that revision with `-DSUPPORT_X11=ON -DSUPPORT_BORG_RECORDING=ON`
(into `build/rec-x11`) and the recorder's command-line tool from the build's
vendored copy, then copies the game, its data, the tool, the launcher and
`watch-recording.py` into `BUILD/launcher/` (frozen: a later rebuild cannot
replace the executable that recordings need), writes `BUILD/launcher/INSTALL-RECEIPT`
(revision, recorder pin, hashes) and points `~/.local/bin/angband-reborn-record`
at it. It prints the link's previous target; `NO_LINK=1` leaves the link alone.
It needs the libX11 and OpenSSL headers, takes the validation lock and a few
minutes, and refuses an existing `BUILD/launcher`. Install again after making a
new build (section *A new build*); the older installs keep working for their
recordings.

Current install: `~/angband-builds/borg-reborn-rs-recording-88d98537/launcher`
(borg-reborn-rs `88d98537`, recorder v0.3.0, game SHA-256
`29bc3092a67d64bb5934af0bcac78f58cfe4360ea159600fc68ef28e36e6a582`). The
previous one, made by hand, was
`~/projects/angband/borg-reborn-rs/build/installed/d4471de7…/recording`
(`d4471de7`, recorder v0.2.0); it is not in any repository and no script made it.

**How a windowed recording ends** (checked with a test character under a virtual
display, 2026-11-21):
- **Quit through the game (Ctrl-X):** the recording is sealed and its last frame is
  `exit`, holding the end state (turn, `dead`). A death is recorded the same way
  once you quit.
- **The window is closed or killed abruptly** (the X connection breaks): the
  game exits and the recording is sealed (`complete.json`). With recorder
  v0.3.0 its last frame was the last `command`/`wait` frame, so what happened
  after it was lost. From the release after v0.3.0 (planned v0.3.1) the exit
  handler writes a `final` frame of the end state, as the headless frontend
  does. Checked under a virtual display on a scratch build of that change: a
  killed window ends with `final`, a Ctrl-X quit with `exit` (no `final` after
  it). Quit with Ctrl-X when you can; the `exit` frame is the game's own.

Not checked: a window on a real display, a death in a window, killing the game
with a signal, and replaying a windowed recording (`replay --verify` needs a
`launch.json`, which the launcher's recordings do not have).

## A new build, and how to get the latest

Make one when borg-reborn-rs has changed: a new Borg, or a recorder pin update.
A build is a snapshot of borg-reborn-rs at one revision with the Borg compiled
into the game, so a changed Borg needs a new build. **Keep the old builds**: a
recording replays only with the executable that made it.

1. **Build the latest.** With no arguments `setup.sh` fetches borg-reborn-rs and
   builds `origin/main` into `~/angband-builds/borg-reborn-rs-recording-<revision>`
   (the revision as 8 hex digits, so the name says which Borg it has). It takes
   the validation lock and a few minutes, refuses an existing directory, and
   points `~/angband-builds/current` at the new build, which is what the other
   scripts then use.
   ```sh
   $S/setup.sh
   ```
   For another revision give it (a commit hash or branch of borg-reborn-rs):
   `$S/setup.sh 88d98537`. `NO_CURRENT=1 $S/setup.sh REV` builds without moving
   the link. To choose the directory: `BRR_BUILD=~/angband-builds/NAME $S/setup.sh`.
   The parameters in `run-env.sh` and the `borg_trip_*` settings carry over:
   `setup.sh` applies them to every new build.
   For the window launcher, install it for the new build too:
   `$S/install-launcher.sh` (section 6); `angband-reborn-record` keeps running the
   previous install until you do.

   Don't give it plain `main` and don't fetch by hand: `main` is the checkout's
   local branch, which a fetch doesn't move, so it can be hundreds of commits
   behind and carry an old recorder pin. `origin/main` (the default) is right.
2. **Check the recorder** the build carries. It comes from the `recorder-pin.json`
   of that revision, not from this repository, and a Borg update alone never
   changes it. From v0.3.0 on, a headless recording ends with a `final` frame
   holding the end state (the death turn); older recorders lose the turns after
   the last command.
   ```sh
   source $S/run-env.sh
   grep '"tag"' "$BRR_BUILD/recorder-pin.json"      # "tag": "v0.3.0" or later
   ```
   To see it work, record a game that dies and look at the last frame:
   ```sh
   tail -n1 ~/angband-recordings/NAME/native/frames.jsonl | python3 -c "import sys,json;f=json.loads(sys.stdin.read());print(f['phase'],f['turn'],f['dead'])"   # final <death turn> 1 (alive at the turn limit: final <turn> 0)
   ```
3. **Remove a build you no longer need** with git, not `rm -rf`: the build is a
   git worktree of borg-reborn-rs, and deleting the directory leaves a record
   that makes `setup.sh` fail ("missing but already registered worktree").
   ```sh
   git -C ~/projects/angband/borg-reborn-rs worktree remove --force ~/angband-builds/DIR
   # if the directory is already gone:  git -C ~/projects/angband/borg-reborn-rs worktree prune
   ```
   Recordings made with a removed build can no longer be replayed. If `current`
   pointed at it, run `setup.sh` again or `ln -sfnT ~/angband-builds/DIR ~/angband-builds/current`.

A recorder release reaches the builds only through a pin-update change in
borg-reborn-rs (v0.3.0: its PR #809). After one is merged, a build made from a
revision at or after the merge has it.

## Recordings so far

These three were made before the save interval became 1,000 game turns: they
have a save every 50,000, so `rewind.sh` finds an older save for them.

| Directory | Seed | Character | Outcome |
|---|---|---|---|
| `~/angband-recordings/gnome-mage-20261117` | 20261117 | Gnome Mage | Died at level 2 on dungeon level 1 (giant white mouse), turn 14,762. Replay verified identical. |
| `~/angband-recordings/1` | 0 | Gnome Mage | Survived to turn 1,000,009, level 9, never deeper than dungeon level 3. |
| `~/angband-recordings/gnome-mage-20261117-failed-finalsave` | 20261117 | Gnome Mage | Broken first attempt (`finalsave`); not replayable, can be deleted. |
