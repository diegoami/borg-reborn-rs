# The zoo contract

*docs/SPEC.md §4, §6; task TE.7 (#77). borg-zoo measures our Borg, and
every comparison is the zoo's. This is the interface the zoo's adapter
(`zoo/borgrs.py`, kind `borg-rs`, read at our borg-zoo pin `e56d5fa`)
calls. Our harness may change freely everywhere else; what is below may
change only by agreement with the zoo (RULES 7: an issue there).
`harness/zoo-check.py` runs that adapter against this checkout, and CI
runs `engine/tests/test_zoo_contract.py`.*

## Build

| What the zoo does | What we guarantee |
| --- | --- |
| checks out the specimen's `ref`, a full commit (tag `e` or later; never a branch head) and requires `git rev-parse HEAD` to equal it | every merged commit builds |
| runs `harness/build.sh` in the checkout, with `BORG_IMPL=<impl>` | `BORG_IMPL=rust` is the only implementation; any other value fails loudly (exit 2) |
| copies `build/<impl>/game/` | `game/angband` and its data `game/lib/`, self-contained (the game finds `./lib/` from its working directory) |
| reads `build/<impl>/BUILD_ID` (`key=value` lines) | at least `binary_sha256` (equal to the copied binary's SHA-256) and `rustc` |
| copies `harness/borg.txt` and writes it, with its profile's settings, as the run's `user/borg.txt` | the file exists. The Rust Borg doesn't read it: its own parameters come from its profile (`BORG_PROFILE`), with the defaults when none is given, as in the zoo's runs |

## Run

The zoo runs, in the copied `game/` directory, with `TZ=UTC`,
`LANG=C.UTF-8` and `LC_ALL=C.UTF-8`:

```
./angband -mborgh -n -uBorg -duser=OUT/user -dsave=OUT/save -darchive=OUT/archive -dpanic=OUT/panic -dscores=OUT/scores \
    -- -seed N -race I -class J -turns CAP -keylog OUT/keys.log -summary OUT/summary.json -menu-letters \
    [-exact-turns] [-statelog OUT/state.log]
```

Our frontend (`engine/frontend/main-borgh.c`, docs/SPEC.md §3.5) answers
every one of these flags. It refuses an unknown one, so a contract change
cannot pass unnoticed.

| Output | Fields or format the zoo reads |
| --- | --- |
| `summary.json` | `dead`, `stop_reason`, `race`, `class`, `death_cause`, `game_turns`, `depth`, `max_depth`, `clevel`, `keys` |
| `keys.log` | one line per key: `<game turn> <event type> <key code hex> <mods hex>`; the zoo hashes it as the cell's play |
| `state.log` (with `-statelog`) | one line per key, starting `<game turn> <x>,<y> d<depth>/<max depth>`; the zoo derives `first_turn_at_depth` and its milestones from it |

**`stop_reason`** is one of `turns`, `death`, `clevel`, `depth`, `keys`,
`stalled`, `borg_stopped` or `blocked`. The zoo maps `turns`, `death`,
`depth`, `borg_stopped` and `blocked` (`STOP` in its adapter); its key for
the level stop is `clev`, but ours writes `clevel`. The zoo never passes
`-clev`, so no cell can stop that way.

Further fields of `summary.json` (`game_options`, `exact_turns`, `loaded`,
the RNG states, …) are ours. The zoo may read them, but doesn't depend on
them. `exact_turns` and `loaded` are in every summary, `true` or `false`
(since the E review).

## Specimens

- `4.2.6-reborn-rust`: `impl` `rust`, `exact_turns` and `depth_stream` on.
  Its `ref` is `2dd11fd` today (M1, built on borg-rs's tree). Milestone E
  is merged (tag `e`); the `ref` moves to the commit that fixes E's review
  (borg-zoo#48), since those fixes change the full suite's play.
- `4.2.6-reborn-c-pristine` and `4.2.6-reborn-c-bbc-m7` were built by this
  repository at `2dd11fd`. Since milestone E it builds no C Borg (docs/SPEC.md
  §6), so those specimens keep their old `ref` or are retired: the zoo's
  call.
