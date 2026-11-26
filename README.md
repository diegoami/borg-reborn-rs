<!-- Cemetery ownership: normal death dossiers now live in diegoami/borg-cemetery. -->

# borg-reborn-rs

An automatic player (a "Borg") for [Angband](https://github.com/angband/angband)
4.2.6, in Rust. Press **Ctrl-Z** to open the Borg prompt. PBorg is selected
at startup. Press **B** at the prompt to switch Borgs and **Ctrl-Z**, **z**
to start the selected Borg. The game engine is upstream Angband at
`574b618c3bfec595f3503a58f8e488b1c4ae3a46`. PBorg plays with the
**borg-rs core**: a line-by-line Rust translation of the original C Borg,
vendored in `borg-core/`, with named settings, errands and a decision trace
added here. [borg-zoo](https://diegoami.github.io/borg-zoo/) measures it.

**Status (2026-11-21):** the borg-rs core plays every class from birth and
is the default core (#479, #606). Its thresholds are being turned into named
settings (track P, P1–P5 on `main`, P6 under review) and its decisions are
being redesigned on a separate branch (track R, R4 in progress). Our own
first-principles core (`BORG_CORE=reborn`) is frozen. Details:
[docs/architecture/reborn-plan.md](docs/architecture/reborn-plan.md).

This page is for **playing**: setting up, building, watching the Borg and
changing what it does. For how it works, see
[docs/architecture/](docs/architecture/README.md); for the game knowledge it
and its developers use, [docs/GAME-KNOWLEDGE.md](docs/GAME-KNOWLEDGE.md);
for working on the repository, [AGENTS.md](AGENTS.md). This repository is
the central place for our Borg documentation, including our
[Angband Strategy Manual](docs/strategy/README.md); what is ours and what is
other authors' is in [docs/DOCUMENTATION-MAP.md](docs/DOCUMENTATION-MAP.md).

**Without building:** a release's Linux or Windows package plays as it is:
unpack it and start the game from its folder (`./angband`, or `angband.exe`),
then Ctrl-Z, z; see its `PLAYING.txt`.
[docs/RELEASING.md](docs/RELEASING.md) says how to install one and how
releases are made.

## 1. Requirements

A Linux machine (the commands below were checked on Ubuntu) with:

| What | Why | Debian/Ubuntu |
| --- | --- | --- |
| git, Python 3, CMake, a C compiler | the build and the harness | `git python3 cmake build-essential` |
| Rust (stable, with cargo) | the Borg | [rustup](https://rustup.rs/) |
| OpenSSL's crypto library | the engine links libcrypto | `libssl-dev` |
| X11 headers and bitmap fonts, a display | only to **watch** the Borg in a window | `libx11-dev xfonts-base` |
| a UTF-8 locale | the game refuses to start without one ("Angband requires UTF-8 support") | e.g. `export LANG=C.UTF-8` |

```bash
sudo apt-get install -y git python3 cmake build-essential libssl-dev libx11-dev xfonts-base
```

## 2. Get the code

```bash
git clone https://github.com/diegoami/borg-reborn-rs.git
```

```bash
cd borg-reborn-rs && bash scripts/setup.sh
```

`scripts/setup.sh` checks the tools above and clones the related
repositories into `vendor/` at their pins (the original Angband, the C Borg,
borg-rs, borg-zoo, ...). Playing does not need them; developing does. It is
safe to rerun.

**For headless runs**, run `scripts/setup.sh` once: it keeps
[borg-cemetery](https://github.com/diegoami/borg-cemetery) in `vendor/` at the
commit named in `death-tools-pin.txt` (about 1.2 GB). Every run ends by handing
a death (if any) to its tools, and `harness/run.sh` fails without them.

```bash
bash scripts/setup.sh
```

A borg-cemetery checkout next to this repository is used instead when its
tools match the pin, and `BORG_CEMETERY_DIR` names a checkout explicitly; see
[tools/README.md](tools/README.md#cemetery-owned-death-tools).

## 3. Watch the Borg play

```bash
./watch-borg.sh --build-only
```

builds the game with the X11 window into `build/watch/` (about a minute).
The game's user directory for watched games is `.angband-reborn/user/`. On
its first start `watch-borg.sh` writes the shipped settings there
(`harness/pborg.yaml` and its scary list),
so the borg-rs core plays them. Later starts keep that `pborg.yaml`, yours to
edit, and say when it differs from the shipped one. After updating the
repository, take the new shipped settings with:

```bash
./watch-borg.sh --reset-settings --name MyBorg
```

```bash
LANG=C.UTF-8 ./watch-borg.sh --name MyBorg
```

1. The game opens. Create a character (Enter through the birth screens), or
   load `MyBorg` if it exists: the same `--name` reloads the same save.
2. In the game, press **Ctrl-Z**. Dismiss the two warnings (Enter) and
   answer **y** to "Are you sure you want to use the borg commands?".
3. At `Borg command (? for help):` press **z**. The Borg plays.
4. Any key stops it. Ctrl-Z, z starts it again.

Saves and settings stay in `.angband-reborn/` (ignored by git).
`BORG_PLAY_DATA_DIR` moves them elsewhere. A watched game is not a measured
run: its death is not recorded (to record one, reproduce it with
`harness/run.sh`).

## 4. A headless run

```bash
BORG_IMPL=rust harness/build.sh
```

builds the headless game into `build/rust/` (the zoo's entry point,
[docs/ZOO-INTERFACE.md](docs/ZOO-INTERFACE.md)). Then:

```bash
harness/run.sh /tmp/run1 1 0 0 5000 menu
```

plays seed 1, race 0 (Human), class 0 (Warrior; indices in the order of the
game's `p_race.txt` and `class.txt`) for 5,000 game turns, deterministically.
The output directory holds `keys.log`, `messages.log`, `state.log`,
`summary.json` (the outcome), `trace-core.jsonl` (one line per decision),
`params-core.json` and `settings-resolved.json` (the settings it played
with). Flags and environment variables are listed at the top of
[harness/run.sh](harness/run.sh) and in [harness/README.md](harness/README.md).
A suite of runs: `harness/suite.sh smoke OUTDIR`.

## 5. Which core plays

| Core | What | How to select |
| --- | --- | --- |
| **borg-rs** (default) | borg-rs's core in `borg-core/`, with our settings, errands and trace | `borg_reborn_core: "borg-rs"` in `pborg.yaml` (shipped) |
| reborn (frozen) | this repository's own first-principles core | `BORG_CORE=reborn` in the environment, or `borg_reborn_core: reborn` |

`BORG_CORE` in the environment wins over the file. The reborn core reads
`harness/profiles/reborn-default.txt`, not `pborg.yaml`, and none of the
settings below apply to it.

## 6. Changing what the Borg does

PBorg reads **`user/pborg.yaml`**. The package, harness and Zoo use
`harness/pborg.yaml` as their source for the shipped settings. For a watched game, edit
`.angband-reborn/user/pborg.yaml`. CBorg reads its own `user/borg.txt`. Each
setting is commented: what it does, what raising or lowering it does, and
how many short test games it changed. The values shipped are the ones the
zoo measures; a change you make is your own experiment. To import settings
from an older PBorg install, run `python3 harness/pborg-config.py OLD_PBORG_SETTINGS > pborg.yaml`
and place the resulting file in the game's user directory.

| To change | Where |
| --- | --- |
| the core's built-in settings (`borg_stop_dlevel`, `borg_no_deeper`, `borg_plays_risky`, ...) | their keys in `pborg.yaml` |
| a named threshold of the core | its `borg_param_<name>` key in `pborg.yaml`; the full list with units, ranges and the code each one reaches is [docs/CALIBRATION.md](docs/CALIBRATION.md) |
| one class only | a YAML entry such as `borg_param_<name>__mage: value` |
| a setting for one run | `BORG_PARAM_<NAME>=value` (or `BORG_PARAM_<NAME>__<CLASS>`) in the environment; `PBORG_SET="name=value;..."` for `harness/run.sh` |
| the monsters that make a level "scary" | `harness/scary_monsters.txt` (format in its header), selected by `borg_scary_list` |
| a long-term goal: go to town, dive to a depth, a ladder, a trip plan | an **errand**: `borg_reborn_errand` in `pborg.yaml`, `BORG_ERRAND`, or in the game (below); [docs/ERRANDS.md](docs/ERRANDS.md) |
| a whole game plan by class and level | a plan file ([docs/GAME-PLAN-LANGUAGE.md](docs/GAME-PLAN-LANGUAGE.md); examples in `harness/plans/`), `borg_plan: FILE` or `BORG_ERRANDS_PLAN` |
| when the Borg hands control back to you | `borg_reborn_stopscared` (off, a percent of HP, or `flee`) |

**In the game**, Ctrl-Z then **/** opens the Borg's command line
([docs/ERRANDS.md](docs/ERRANDS.md)): `/town`, `/dive 5`, `/ladder`,
`/explore-level`, `/stopscared flee`, `/stopscared off`, `/stop`, several
joined with `;`. Ctrl-Z then **?** lists PBorg's other commands
(**$** reloads `pborg.yaml`'s built-in Borg settings; the `borg_param_*` values are read
once per game process).

## 7. Deaths and measurement

- **Deaths.** A headless run that dies is exported as a dossier (the trace's
  tail, the settings, the keys) to
  [borg-cemetery](https://github.com/diegoami/borg-cemetery), the registry of
  every recorded death, its investigations and replayable scenarios.
  [tools/README.md](tools/README.md) explains the tools and how to rerun a
  death.
- **Measurement.** Whether a change makes the Borg better is decided by
  [borg-zoo](https://diegoami.github.io/borg-zoo/), which plays frozen seed
  suites with a given build and settings and compares Borgs, not by a few
  local games ([docs/ZOO-INTERFACE.md](docs/ZOO-INTERFACE.md)).

## Related repositories

| Repository | Role here |
| --- | --- |
| [angband/angband](https://github.com/angband/angband) | The game and the original C Borg (`src/borg/`), at the pin above. Read only. |
| [better-borg-c](https://github.com/diegoami/better-borg-c) | The C Borg with verified bug fixes: the C ancestor the architecture documents cite. |
| [borg-rs](https://github.com/diegoami/borg-rs) | The faithful Rust port; its decision core is vendored in `borg-core/` at a pin ([docs/architecture/borg-core.md](docs/architecture/borg-core.md)). |
| [borg-zoo](https://github.com/diegoami/borg-zoo) | Measures Borg versions on frozen seed suites. Our yardstick. |
| [borg-cemetery](https://github.com/diegoami/borg-cemetery) | The death registry: recorded runs, investigations, scenarios. |
| [neo-angband-tracker](https://github.com/diegoami/neo-angband-tracker) | Where the C Borg specification, the Angband Strategy Manual and the C Borg defect catalogue were written; they are kept in `docs/` now. |
| [borg-runner](https://github.com/diegoami/borg-runner) | Language-model agents playing from the screen; what they learn is a source of game knowledge. |
| [borg-agent](https://github.com/diegoami/borg-agent) | Retired 2026-11-11. |

More on each: [docs/SOURCES.md](docs/SOURCES.md).

Licence: GPLv2 (see `LICENSE`), as borg-rs. Angband itself is dual-licensed
GPLv2 / Angband licence; files copied from other projects keep their
headers and are listed with their provenance ([docs/PROVENANCE.md](docs/PROVENANCE.md)).
