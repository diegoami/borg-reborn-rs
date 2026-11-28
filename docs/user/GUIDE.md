# Play guide

*Layer 1 of the documentation (#817): for people who want to watch the
Borg or play with it, without reading code. It describes `track-p`, the
branch where the work happens, which reaches `main` at each reviewed
milestone. The online copy is
<https://github.com/diegoami/borg-reborn-rs/blob/main/docs/user/GUIDE.md>.
To help with the project, read the
[contributor guide](../contributing/README.md) instead.*

- [1. Install](#1-install)
- [2. Watch the Borg play](#2-watch-the-borg-play)
- [3. The Borg's commands](#3-the-borgs-commands)
- [4. Change how it plays](#4-change-how-it-plays)
- [5. A game without a window](#5-a-game-without-a-window)
- [6. Record and replay a game](#6-record-and-replay-a-game)
- [7. When it dies, and how it is measured](#7-when-it-dies-and-how-it-is-measured)
- [8. The family of projects](#8-the-family-of-projects)

## 1. Install

The [README](../../README.md) gives the two ways: a **release package**, or
a **build from source**. This section adds the details.

### From a package

Unpack it anywhere you can write. Start the game from its own folder,
because the game looks for `lib/` there:

| System | Start | Needs |
| --- | --- | --- |
| Linux x86_64 | `./angband` opens a window when there is a display. `./angband -mgcu` plays in the terminal. | glibc 2.39 or newer (Ubuntu 24.04, Debian 13), `libssl3t64`, `libncursesw6`, `libx11-6`, and `xfonts-base` for the window |
| Windows 10/11 | double-click `angband.exe`. It plays in a console window of at least 80x24. | nothing else. Windows may warn that the program is not signed: *More info*, then *Run anyway*. |

In the package folder:

| Path | What |
| --- | --- |
| `lib/save/` | your saves |
| `lib/user/pborg.yaml` | PBorg's settings. Edit them, see [section 4](#4-change-how-it-plays). |
| `lib/user/borg.txt` | CBorg's settings |
| `borg/` | a copy of the shipped settings, to compare or start over |
| `docs/` | Angband's manual and this guide |
| `RELEASE.txt`, `BUILD_ID.*` | exactly what the package was built from |

### From source

The commands below were checked on Ubuntu.

| You need | Why | Debian/Ubuntu package |
| --- | --- | --- |
| git, Python 3, CMake, a C compiler | the build | `git python3 cmake build-essential` |
| Rust (stable, with cargo) | the Borg | from [rustup](https://rustup.rs/) |
| OpenSSL's crypto library | the game links it | `libssl-dev` |
| X11 headers and fonts, a display | the game's window | `libx11-dev xfonts-base` |
| a UTF-8 locale | the game refuses to start without one ("Angband requires UTF-8 support") | `export LANG=C.UTF-8` |

For headless runs ([section 5](#5-a-game-without-a-window)), use **GCC on
x86-64**. With clang, or on an arm64 machine, the same seed plays a
different game from the zoo's, so `harness/build.sh` refuses to build there
unless `BORG_ALLOW_OTHER_GAMES=1` is set (#855). Watching is not affected.

## 2. Watch the Borg play

```bash
./watch-borg.sh --build-only
```

This builds the game, with its window, into `build/watch/`. Then start it:

```bash
LANG=C.UTF-8 ./watch-borg.sh --name MyBorg
```

- **The character.** `--name MyBorg` is the character's name, and the same
  name loads the same save. Without `--name`, the character is called
  `RustBorg`. Create a new character by pressing Enter through the birth
  screens.
- **Starting the Borg.** Press **Ctrl-Z**, dismiss the warnings with Enter,
  answer **y**, then press **z** at the `PBorg command (? for help):`
  prompt. The first time, PBorg introduces itself on a screen of its own:
  press a key and it starts playing.
- **Taking over.** Any key stops the Borg and gives you the controls. Play
  on yourself, or press Ctrl-Z, z to hand the game back.
- **Where things are kept.** Saves and settings are in `.angband-reborn/`,
  which git ignores: saves in `save/`, settings in `user/`. Set
  `BORG_PLAY_DATA_DIR` to keep them somewhere else.
- **The settings.** On its first start, `watch-borg.sh` copies the shipped
  settings (`harness/pborg.yaml` and the scary-monster list) into
  `.angband-reborn/user/`. After that it keeps your copy and says when it
  differs from the shipped one. To take the shipped settings again, for
  example after updating the repository:

  ```bash
  ./watch-borg.sh --reset-settings --name MyBorg
  ```

  `watch-borg.sh` checks `pborg.yaml` before the game opens and stops with
  a message if a setting is wrong.

A watched game is for watching: its death is not recorded. To record a
death, play the game again by seed without a window
([section 5](#5-a-game-without-a-window)).

## 3. The Borg's commands

Ctrl-Z opens the prompt of the selected Borg. **PBorg** is selected when
the game starts, and **B** switches to **CBorg** and back. Then:

- **z** starts the Borg. Any key stops it.
- **?** lists the commands, which are mostly for looking at what the Borg
  knows.

PBorg's commands (`borg-core/src/borg_commands.rs`):

| Key | What it does |
| --- | --- |
| `z` | start the Borg; any key stops it |
| `x` | one step: the Borg makes one decision and stops |
| `?` | this list |
| `B` | switch to CBorg (in CBorg, `B` switches back) |
| `$` | reload `pborg.yaml`'s Borg settings. The `borg_param_*` values are read once per game, so restart the game for those. |
| `c` | toggle cheat death for this game (`borg_cheat_death`: the Borg dies and keeps going) |
| `R` | respawn: a new character, as `borg_respawn_*` says |
| `m` | map information |
| `G`, `I`, `D` | a grid you target: its features, what the Borg knows of it, its danger |
| `@` | the line-of-sight map |
| `F` | the map of fear levels |
| `^` | the path the Borg follows |
| `t` | current targeting |
| `y` | the step history |
| `h` | the things the Borg has |
| `p` | the Borg's power, its own score of its equipment |
| `P` | its preparation depth: how deep it believes it is ready to go |
| `0` | its stats (Str, Int, ...) |
| `w` | the weapon and armour swaps it keeps |
| `S` | its spells |
| `o` | an object's details |
| `C` | a count of the "nasties" it has seen |
| `!` | its time-tracking flags |
| `f` | update its configuration flags |
| `u` | refresh its variables |
| `s` | set a search string |
| `l` | write a snapshot log file |
| `v` | the version |

The keys `1`–`9`, `+`, `-` and `>` choose the shop the Borg visits.

CBorg is Angband's own Borg, and its commands are upstream's: **?** at the
`CBorg command` prompt lists them.

The errands that older builds had (`/town`, `/dive N`, `/ladder`,
`/stopscared` and the others typed after Ctrl-Z, /) and the depth plans
were retired on 2026-11-27 (#1339). The archived page
[ERRANDS.md](../archive/ERRANDS.md) describes them.

## 4. Change how it plays

### Where the settings are

| Borg | File | Watched game | Package |
| --- | --- | --- | --- |
| PBorg | `pborg.yaml` (YAML) | `.angband-reborn/user/pborg.yaml` | `lib/user/pborg.yaml` |
| CBorg | `borg.txt` (upstream's format) | `.angband-reborn/user/borg.txt` | `lib/user/borg.txt` |

CBorg writes a default `borg.txt` when there is none.

The shipped PBorg settings are
[`harness/pborg.yaml`](../../harness/pborg.yaml). Every setting in it is
commented:
- what it does, and what raising or lowering it does;
- which settings change together;
- how many of six short test games it changed (its "Impact" line).

The shipped values are the ones the zoo measures, so a value you change is
your own experiment.

### What you can change

| To change | Edit in `pborg.yaml` |
| --- | --- |
| when the Borg stops: a dungeon level, a character level, after Morgoth | `borg_stop_dlevel`, `borg_stop_clevel`, `borg_stop_king` |
| how deep it goes | `borg_no_deeper: N`: it does not choose to go down to level N or deeper, and heads up when it is there. Lower it for a more careful Borg. |
| whether it takes risks | `borg_plays_risky` (shipped `false`). With `true` it dives regardless of its level and stays in fights instead of teleporting: faster, and more deaths. |
| how fast it plays on screen | `borg_delay_factor` (0 is the fastest) |
| a new character when it dies | `borg_respawn_race`, `borg_respawn_class`, `borg_respawn_winners` (-1 is random) |
| a named threshold: when to heal, flee, buy, rest, ... | its `borg_param_<name>` key. [docs/CALIBRATION.md](../CALIBRATION.md) lists every one, with its unit, range and what it reaches. |
| one class only | a key with the class after a double underscore: `borg_param_<name>__mage: value` |
| the monsters that make a level "scary" | [`harness/scary_monsters.txt`](../../harness/scary_monsters.txt) (format in its header), chosen by `borg_scary_list` |

The file is sectioned (Town, Word of Recall, Descending, Combat, Healing,
Fleeing, Cornered, Breeders, Scary monsters, and the class overrides), so
use the section headings to find a setting. Some common changes:

- **More careful:** a lower `borg_no_deeper`, for example `borg_no_deeper: 5`.
- **Braver and faster:** `borg_plays_risky: true`.
- **Stop at a depth to take over yourself:** `borg_stop_dlevel: 10`.
- **Slower, to follow it:** `borg_delay_factor: 5`.

After editing, restart the game, or press Ctrl-Z, `$` for the Borg's own
settings. To bring settings over from an older PBorg install, use the
import script; it writes a `pborg.yaml` for the game's user directory:

```bash
python3 harness/pborg-config.py OLD_PBORG_SETTINGS > pborg.yaml
```

### Which core PBorg plays

PBorg plays the **borg-rs core** (`borg_reborn_core: "borg-rs"`): a Rust
translation of the original C Borg, with this project's settings. The
project's own first-principles core, **reborn**, is frozen. It plays only
when `BORG_CORE=reborn` is set in the environment, and it reads none of the
settings above.

## 5. A game without a window

A headless game plays a seed **deterministically**: the same build, seed
and settings give the same game, key for key. This is how deaths are
recorded and how the zoo measures the Borg. From a source checkout, first
set up the death tools once:

```bash
bash scripts/setup.sh
```

This checks your tools and clones the related repositories into `vendor/`.
It includes [borg-cemetery](https://github.com/diegoami/borg-cemetery), about
1.2 GB, whose tools `harness/run.sh` needs to file a death. Then build the
headless game into `build/rust/`:

```bash
BORG_IMPL=rust harness/build.sh
```

Play one game:

```bash
harness/run.sh /tmp/run1 1 0 0 5000 menu
```

This plays seed 1 with race 0 (Human) and class 0 (Warrior) for 5,000 game
turns. The numbers follow the game's race and class lists. `/tmp/run1` then
holds:

| File | What |
| --- | --- |
| `summary.json` | the outcome: alive or dead, turns, depth, level |
| `messages.log`, `keys.log`, `state.log` | the game's messages, every key, the character's state |
| `trace-core.jsonl` | one line per Borg decision ([docs/CORE-TRACE.md](../CORE-TRACE.md)) |
| `params-core.json`, `settings-resolved.json` | the settings it played with |

To change a setting for one run, put it in the environment:
- one setting: `BORG_PARAM_<NAME>=value`;
- one class's value: `BORG_PARAM_<NAME>__<CLASS>=value`;
- several settings for `harness/run.sh`: `PBORG_SET="name=value;..."`.

All the flags are listed at the top of [harness/run.sh](../../harness/run.sh).

## 6. Record and replay a game

A headless game replays from its seed, so it needs no recording. A game
you watch or play by hand can be recorded with the optional
[angband-recorder](https://github.com/diegoami/angband-recorder). It is
pinned in `recorder-pin.json` and kept in `vendor/angband-recorder`. It
records what the game is given (keys, the terminal, the time) and replays
it later into a separate directory, checking that the Borg chooses the same
keys again.

- **Build** with the recorder: configure CMake with
  `-DSUPPORT_BORG_RECORDING=ON`. Normal builds and measured runs leave it
  off.
- **Record:** set `ANGBAND_RECORD_DIR=DIR` when you start the game.
- **Replay:** set `ANGBAND_REPLAY_DIR=DIR` (and `BORG_TOOLKIT_HEADLESS=1`
  for the headless frontend).

The recipe, its limits and its checks are in
[docs/ANGBAND-RECORDER.md](../ANGBAND-RECORDER.md).
[docs/GUIDED-DEBUGGING.md](../GUIDED-DEBUGGING.md) gives the steps for a
recorded game that went wrong.

## 7. When it dies, and how it is measured

- **Deaths.** When a headless game dies, its death is filed as a dossier
  in [borg-cemetery](https://github.com/diegoami/borg-cemetery). A dossier
  holds the settings, the keys and the last decisions, so the death can be
  replayed exactly. The Cemetery also keeps the investigations of deaths,
  and saved scenarios to try a Borg on. Watched games are not filed.
- **Measurement.** A few games say little, because the same Borg lives in
  one game and dies in the next. Whether the Borg plays better is decided by
  [borg-zoo](https://diegoami.github.io/borg-zoo/). It plays the same frozen
  set of seeds with each build and its settings, then compares the results:
  - how deep each Borg gets, and how often it dies;
  - the deaths a change rescued or caused;
  - the games that did not change at all.

  The zoo's dashboard shows the latest runs. The values shipped in
  `pborg.yaml` are the ones it measured.

## 8. The family of projects

| Project | What it is |
| --- | --- |
| [Angband](https://github.com/angband/angband) | the game, and its original C Borg (CBorg) |
| [borg-rs](https://github.com/diegoami/borg-rs) | the line-by-line Rust port of the C Borg; its core is PBorg's |
| [better-borg-c](https://github.com/diegoami/better-borg-c) | the C Borg with verified bug fixes |
| [borg-zoo](https://diegoami.github.io/borg-zoo/) | measures Borgs on frozen seed suites |
| [borg-cemetery](https://github.com/diegoami/borg-cemetery) | every recorded death, its investigation, scenarios |
| [angband-recorder](https://github.com/diegoami/angband-recorder) | the optional game recorder |

The full list, with what each project is used for here and its pinned
commit, is in [the contributor guide](../contributing/repositories.md).
