# Recording borg-reborn-rs games

The owner's workflow (set up 2026-11-17) for recording, replaying, opening and
watching games of the consumer borg-reborn-rs with this recorder. It runs in a
separate local build: nothing here changes borg-reborn-rs or its pin, and
recordings, saves and builds stay outside Git.

The scripts are in this directory. `S` below means this directory, e.g.
`S=~/projects/angband/angband-recorder/docs/borg-reborn-rs-recording`.

| Script | What |
|---|---|
| `setup.sh [REV]` | Makes the recording build: a detached worktree of borg-reborn-rs at `$BRR_BUILD` (default `~/angband-builds/borg-reborn-rs-recording`), the return settings in its `harness/borg.txt`, and a headless game with the recorder on in `build/rec` (`-DSUPPORT_BORG_RECORDING=ON`). |
| `record.sh` | Records a game and shows it live in the terminal. |
| `rewind.sh` | Shows the last turns before a game's end and names (or opens) the save just before them. |
| `run-env.sh` | The parameters and recording settings as environment variables; the other scripts source it. |
| `replay.sh` | Replays a recording and compares it with the original. |
| `open-save.sh` | Opens a copy of a save in an X11 game window. |
| `watch-recording.py` | Plays a recording's screens in the terminal, also live (`--follow`). |

Environment: `BRR_BUILD` (the recording build, default above) and
`BORG_REBORN_RS` (the main checkout, default `~/projects/angband/borg-reborn-rs`).

## The current build

`~/angband-builds/borg-reborn-rs-recording`, borg-reborn-rs commit `f02cd294`,
executable SHA-256 `57b1885509ee88024b670845c137ca62098fa92af1c3e049eb68c862621b8cb0`.
The recorder ties a recording to its executable's hash: **don't rebuild a build
whose recordings you want to replay**. Make a new one with
`BRR_BUILD=~/angband-builds/NEW $S/setup.sh` instead.

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
cd ~/angband-builds/borg-reborn-rs-recording   # $BRR_BUILD
source $S/run-env.sh
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

## A new build

Only when you need one, e.g. after borg-reborn-rs changes. Keep the old build
for its recordings.

```sh
BRR_BUILD=~/angband-builds/NEWDIR $S/setup.sh main     # or a commit
export BRR_BUILD=~/angband-builds/NEWDIR               # for run-env.sh, replay.sh
```

## Recordings so far

These three were made before the save interval became 1,000 game turns: they
have a save every 50,000, so `rewind.sh` finds an older save for them.

| Directory | Seed | Character | Outcome |
|---|---|---|---|
| `~/angband-recordings/gnome-mage-20261117` | 20261117 | Gnome Mage | Died at level 2 on dungeon level 1 (giant white mouse), turn 14,762. Replay verified identical. |
| `~/angband-recordings/1` | 0 | Gnome Mage | Survived to turn 1,000,009, level 9, never deeper than dungeon level 3. |
| `~/angband-recordings/gnome-mage-20261117-failed-finalsave` | 20261117 | Gnome Mage | Broken first attempt (`finalsave`); not replayable, can be deleted. |
