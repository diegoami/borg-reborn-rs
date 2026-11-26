# Deterministic Borg harness

Every run now has a one-shot supervisor and a `process.json` outcome.
Optional fixed-cadence engine capture and explicit fresh-Borg C/C/Rust
replays are documented in [checkpoints.md](checkpoints.md). Captures are
made in forked private copies so the stock saver's monster compaction
cannot mutate the live run. Existing failure markers and strict oracles
are preserved; no automatic restart or reference promotion occurs.

## Saved-character replay (M4 D)

`harness/suites/endgame` uses `save=NAME.save` flags to load the fixtures
under `harness/saves/`. Each run copies its input and records its SHA-256;
`harness/diff --suite endgame` checks that hash against the fixture. The
frontend accepts `-load FILE` (omit `-n`), checks the living character's
race/class, and reseeds at the pre-Borg boundary. `loaded_turn` in the
summary is the initial engine turn; `-turns` is an absolute limit.

The original `harness/reference/endgame` is a historical pre-#287 oracle.
With the corrected M4 build, suite endgame against that original reference
is expected to report **4/5 identical and exit1**: end-depth98's messages
contain the corrected offscreen-monster note rather than the old stray
`# PAIN:) (+)` parse. Keys and player state still match all five cases.
Use suite `crash-fixes` with `harness/reference/crash-fixes` for the corrected
checkpoint (those five cases plus Rogue4006), or compare two corrected C
endgame passes and Rust directly. Do not rewrite the original reference.
Suite recipe bytes also stay unchanged to preserve existing receipt hashes.

`-save-on-stop FILE` can capture a living engine state, including from a
natural run. `-make-fixture DEPTH` plus `-save-on-stop FILE` creates a
wizard-equipped level-50 character and an engine-generated level without
starting the Borg. `-near-morgoth` additionally places it next to Morgoth
at depth 100. These switches exist only in the harness frontend. Ordinary
birth runs do not use this setup. `harness/make-fixtures.sh OUTDIR` makes
each fixture twice with a verified C build and refuses differing saves,
logs or summaries. See `harness/saves/README.md` for recipes and limits.
`harness/regress/save-load/run.sh` checks C/C and C/Rust replay and rejects
missing saves, wrong races and mismatched fixture hashes.

The harness runs the vendored Angband engine — upstream Angband at
`42cd156ac9f95ddf4311e88dd7ad462292d6d1d6` (the official import that
moved the pin from `e473720ed88a0f0c7ca66815cef3000eb4a38bc3`; v12R),
with the gated `#ifdef BORG_HARNESS` hooks and the documented fork
corrections on top — and a Borg behind the `inkey_hack` seam. It records every key the Borg returns, every
message added to the game's message log (Borg notes included) and, per
key, a digest of the engine state. Two Borg implementations (C and Rust)
are equivalent when these three logs are byte-identical for the same
seeds.

## Engine changes

Enabled with `cmake -DSUPPORT_BORG_HARNESS=ON`. That defines `BORG_HARNESS`
for the core library and the executable, and adds the `borgh` frontend.
The hooks are under `#ifdef BORG_HARNESS` (or `USE_BORGH`). Two changes are
unconditional, and both are equivalent to upstream in a default build:
the CMake restructure (`BORG_IMPL`, sources moved to `target_sources`; with
the default `BORG_IMPL=c` it compiles the same files) and, in `mon-make.c`,
`localtime()` moved out of the declaration of `date`.

| file | change |
|------|--------|
| `src/main-borgh.c`, `src/main-borgh.h` | new headless frontend `-mborgh` (80x24, no drawing, no sleeping) |
| `src/main.c`, `src/main.h` | register the `borgh` module (`USE_BORGH`) |
| `src/z-rand.c` | `Rand_init()` uses `borgh_seed` when it is set, instead of `time()`/`getpid()` |
| `src/mon-make.c` | `get_mon_num()` uses a fixed date when the seed is fixed (no Christmas monsters) |
| `src/ui-input.c` | `borgh_key_hook` is called with every key returned by `inkey_hack` |
| `src/message.c` | `borgh_message_hook` is called by `message_add()` for every message, before the log is changed (repeats included) |
| `CMakeLists.txt` | option `SUPPORT_BORG_HARNESS`; `BORG_IMPL` (`c` or `rust`) |

The harness hooks do not change Borg tactics. Separate owner-authorized
crash/play-blocker corrections in C and Rust are marked `FORK` and listed
in `docs/FORK-CORRECTIONS.md`; original references remain separate from
corrected-C references. `borg_rand_local` is initialised from `randint1()`
in `borg_init()`, so it follows the game seed.

## The borgh frontend

`angband -mborgh -n -uBorg -d... -- -seed N -race R -class C -turns T -keylog F [-msglog M] [-statelog S] -summary J [-menu-letters]`

1. It feeds a scripted key sequence, and only on blocking waits:
   splash ESC, race letter + Enter, class letter + Enter, Enter (point
   based), Enter (accept points), Enter (default name), `y` (history),
   Enter (final confirm). It then sets `NOSCORE_BORG` and
   `delay_factor = 0` and sends `^Z` `z` (start the Borg).
2. Once the script is used up it never produces a key, so the Borg's
   `user_abort()` never fires.
3. Key log: one line per key returned by `inkey_hack`:
   `<game turn> <event type> <keycode hex> <mods hex>`.
   `EVT_NONE` returns (the Borg idling while resting or repeating) are
   logged too.
4. Message log (`-msglog`, `messages.log`): one line per `message_add()`
   call, `<keys logged so far> <game turn> <message type> <text>`, text
   bytes below 0x20, 0x7f and `\` written as `\xNN`. Repeats are logged
   each time (the message log only counts them). Game messages (`msg`,
   `msgt`) and the Borg's notes go through `message_add()` in both builds:
   C `borg_note()` calls `message_add(what, MSG_GENERIC)` and
   `borg_warning()` calls `msg("%s", what)` (`borg-io.c`); the Rust
   `borg_io.rs` calls the same engine functions through `engine.rs`. So
   the hook is engine side, the same for both builds, and the C Borg is
   untouched. Messages before the Borg starts (birth) have key count 0.
5. State log (`-statelog`, `state.log`): one line per logged key (line i
   belongs to line i of `keys.log`), written right after the key line,
   before the stop checks:
   `<turn> <x>,<y> d<depth>/<max depth> hp<chp>/<mhp> sp<csp>/<msp>
   f<food> au<gold> xp<exp> lv<level> t<i>:<v>,... o<g>:<timeout>,...
   p<hash> e<hash> r<hash>`. `t` lists the set timed effects by index
   (`list-player-timed.h` order, food excluded), `o` the gear objects
   (gear-list position) with a non-zero timeout (light fuel, recharge),
   `-` for none. `p` and `e` are FNV-1a 64 hashes over the objects in
   the pack and quiver, and the equipped ones with their slot, in gear
   order: kind index, tval, sval, number, pval, ac, to_h, to_d, to_a,
   notice, flags, modifiers, element info, curses, and the same fields of
   the object's known version (what the player has learnt of it: its
   runes); `r` hashes which runes the player knows at all
   (`player_knows_rune`). The digest only reads: no RNG call, no engine
   function that changes state.
   A run ends at the key that meets a stop condition, so neither log has
   anything after the last key, except on a stop while the game waits for
   a real key (`death`, `blocked`, `borg_stopped`), where the messages up
   to that wait are logged.
6. Stops when, after a key, `turn >= -turns`, the player is dead,
   `lev >= -clev` or `max_depth >= -depth`. It also stops when the game
   blocks for a real key (`death`, `borg_stopped` or `blocked`). It then
   writes the summary JSON and calls `_exit(0)` (no save). If writing a
   log or the summary fails it exits with 3.
7. `-screen` dumps the 80x24 screen to stderr on any stop that is not a
   turn-limit stop. `BORGH_DEBUG=1` dumps the screen before each scripted
   key.
6. Start-up `-more-`: if, after `^Z` `z`, the game blocks on a `-more-`
   prompt before the Borg has returned any key, the frontend answers it
   with ESC (at most 16 times) and the summary gets
   `"startup_acks": N`. This is the borg.txt warning that `borg_init()`
   prints with `msg()` for `borg_uses_dynamic_calcs` ("Borg's dynamic
   calculations enabled...") before the keypress hook is installed;
   without the answer both Borgs stopped at turn 1 with zero keys. The ESC
   is not a Borg key, so it is not in the key log; both builds get it at
   the same point. A run without such a prompt never blocks there (it
   would have stopped as `borg_stopped` with zero keys), so no other key
   log changes.
7. `-cheat-live` (suite flag `cheatlive`): the frontend turns on the
   game option `cheat_live` just before `^Z`. With `borg_cheat_death`
   off, `borg_init()` then sets the Borg's own cheat-death status from the
   game option, and at death the Borg answers "Die?" with `y` and
   reincarnates (`borg_reincarnate_start`: death menu, new game, birth
   with `borg_respawn_race/class`). The frontend does not stop on death
   with this option; it counts deaths (`"cheat_live": true`,
   `"deaths": N` in the summary) and the run goes on with the new
   character until the turn limit (the game turn restarts at 1 with each
   new character, so `game_turns` is the last character's).

### Race and class letters (known quirk)

By default the script picks race and class with `"abc...z"[index]`. The
birth menus use `all_letters_nohjkl` (`abcdefgimn...`, no h, j, k, l). So
for an index >= 7 the default picks a different entry, for races and
classes alike:

| index | letter sent | race picked | class picked |
|---|---|---|---|
| 0-6 | a-g | as requested | as requested |
| 7 | h (not a menu tag) | stays Human | stays Warrior |
| 8 | i (menu item 7) | Half-Troll | Ranger |
| 9 | j (not a menu tag) | stays Human | - |
| 10 | k (not a menu tag) | stays Human | - |

So `full` and `deep` never play a Blackguard, a Dunadan, a High-Elf or a
Kobold: their `*-ranger-*` runs are Warriors and their `*-blackguard-*`
runs are Rangers. (The header comments of `harness/suites/full` and
`deep`, "all 9 classes", "all 11 races", "one run per class", describe the
requested indices; the suite files are left unchanged.) The default is kept so the existing reference logs stay
valid. `-menu-letters` (suite flag `menu`) sends `all_letters_nohjkl[index]`
instead, so the indices mean `p_race.txt` order (0 Human ... 7 Half-Troll,
8 Dunadan, 9 High-Elf, 10 Kobold) and `class.txt` order (0 Warrior ...
7 Ranger, 8 Blackguard), and adds `"menu_letters": true` to the summary.
Every `summary.json` records the race and class actually played. The
Borg's own respawn code has the same C bug (`docs/c-bugs.md` #11).

## Scripts

* `harness/build.sh`: builds into `build/$BORG_IMPL` (`c` by default,
  `rust` for the Rust Borg). It always passes `-DBORG_IMPL`. After a
  successful build it writes `build/<impl>/BUILD_ID` (see "Build identity"
  below); it deletes the old one first, so a failed build leaves none. It
  fails if the build inputs changed during the build or if the linked
  binary does not read as the requested implementation.
* `harness/identity.sh`: the helpers behind the build identity:
  `source-hash IMPL`, `inputs IMPL`, `inputs-dirty IMPL`, `binary-impl BIN`.
* `harness/run.sh OUTDIR SEED RACE CLASS TURNS [FLAGS]`: one isolated run.
  It gets its own user, save, archive, panic and scores dirs, and a copy of
  `harness/borg.txt` with `borg_respawn_race/class` set to the run's race
  and class. It writes `keys.log`, `messages.log`, `state.log`,
  `summary.json`, `stderr.log` and `config.txt`. `TZ=UTC`, `LC_ALL=C.UTF-8`. FLAGS (comma-separated, `-` for
  none): `cheat` sets `borg_cheat_death = TRUE`; `menu` passes
  `-menu-letters`; `cheatlive` passes `-cheat-live` (see "The borgh
  frontend"). Config flags each set one more option of the run's
  `borg.txt`:
  * `munchkin` (`borg_munchkin_start = TRUE`), `lunal`
    (`borg_lunal_mode = TRUE`), `selflunal` (`borg_self_lunal = TRUE`) -
    **refused since 2026-11-14** (borg-rs#60): the modes were removed, and
    `run.sh` exits 2 naming the flag; a historical mode run needs a checkout
    from before the removal;
  * `NAME=VALUE`, any option: the line `NAME = ...` becomes
    `NAME = VALUE`, e.g. `borg_money_scum_amount=5000`,
    `borg_uses_swaps=FALSE`, `borg_no_deeper=3`. `NAME` is the option's
    full name as in `borg.txt`, `VALUE` is `TRUE`, `FALSE` or an integer
    (the C parser reads a boolean from its first letter, an integer with
    `atoi`). The run fails (exit 2) if `NAME` has not exactly one
    `NAME = ` line in `harness/borg.txt` (an unknown or misspelt option),
    if `VALUE` is not of that form, if two flags give one option
    different values, or for `borg_cheat_death` (use `cheat`). These
    lines are rewritten after the ones `run.sh` sets itself, so
    `borg_respawn_race=-1` overrides the run's race.

  The run fails if a config flag's line is not in `harness/borg.txt`.
  Without config flags the run's `borg.txt` is the same as before they
  existed, byte for byte, so the stock suites are unchanged. Unknown flags
  are an error. Each run writes `config.txt` (its flags and the sha256 of
  its `borg.txt`); the suite manifest copies it as
  `run_config.<run>=<flags> <sha256>`. `harness/diff --suite` checks that
  `config.txt` records exactly the suite line's flags (so a reference made
  with other options fails); the key oracle itself does not read the
  hash. A non-zero exit code or a
  missing summary fails the run. It does not check the build identity:
  a single run is a debugging tool, suite outputs are the evidence.
* `harness/suite.sh SUITE [OUTDIR]`: runs `harness/suites/SUITE` in
  parallel (`JOBS`, default nproc). It runs into `OUTDIR.partial.<pid>` and
  replaces `OUTDIR` at the end, so an output dir never mixes two passes.
  Before any run it checks the binary's implementation and build identity
  and refuses (exit 2, no output dir) on a mismatch; see "Build identity".
  It writes `manifest` (the binary's implementation as each source sees
  it, its sha256 and `BUILD_ID` fields, the checkout's source hash, the
  result of both checks, `overrides=`, the checkout's git revision and
  dirty flag, suite file and `borg.txt` sha256), `timing.txt` (always;
  failed runs are marked), and last `DONE` if every run succeeded or
  `FAILED` otherwise. A failed run does not stop the others; the suite
  then exits 1 and reports the missing runs as `MISSING`. The binary is
  hashed again at the end; if it changed during the suite, the output is
  marked `FAILED`.
* `harness/diff [--suite NAME] [--expect-impl c|rust] A B`: compares two logs or two suite
  directories (`.xz` is accepted). In directory mode it compares the union
  of the run names (a run on one side only fails) and prints
  `N/M runs identical (expected M)`; `--suite NAME` also requires exactly
  the suite's runs on both sides, and that they are the suite's
  scenarios: a suite output's manifest must record this suite file
  (`suite_sha256`), and every run's `summary.json` (both sides) must show
  the suite's seed, race and class indices, menu-letter flag and a final
  turn consistent with its turn limit, and a config run's `config.txt`
  the suite's flags. (References do not record cheat death or an exact
  upper turn bound; see `harness/diff`'s help.) Per run it checks that the key logs are
  byte-identical, that on each side the log has `summary.keys` lines and
  ends at `summary.game_turns`, that the two `summary.json` match apart
  from `wall_seconds`, and that the message logs and the state logs
  (`messages.log[.xz]`, `state.log[.xz]`) are byte-identical; a state log
  must also have `summary.keys` lines, line i at the game turn of key i.
  Missing message/state logs fail directory comparisons, including compressed
  outputs and both operands missing a log. `--allow-legacy-logs` explicitly
  permits a historical reference without modern build identity to omit them;
  it never excuses a suite output's missing logs. Skips are printed as NOTE
  and are not full three-log validation. A message or state divergence is
  printed like a key divergence: run, seed, line, game turn, key index
  and 20 lines of context. It refuses a suite output dir without `DONE`
  (`--allow-incomplete` turns that into a warning; reference dirs hold only
  `.xz` logs and need no marker). For a divergence it prints run, seed,
  game turn, key index, and 20 lines of context. For each suite output
  dir it prints the build identity from the manifest and fails (exit 1)
  if the manifest shows `BORG_IMPL` different from the binary's
  implementation, a recorded override, a failed check, a binary changed
  during the suite. The role of a dir comes from its manifest, not from
  the log format: a dir whose manifest has the build identity is a suite
  output even if its logs are compressed (so compressing an output does not
  drop its checks); a reference is a dir without such a manifest and
  without plain logs, and if it keeps the manifest of its C run (as
  `reference/classes` does), that manifest must record a C build.
  `--expect-impl X` requires that a verified suite output of X takes part
  (either operand); otherwise the diff is refused (exit 2), e.g. two
  references, or a C output compressed to look like one. A
  suite output dir without a manifest, or with a manifest from before the
  build identity (no `overrides=` line), is refused (exit 2).
  `--allow-unverified` turns all of these into warnings. Reference dirs
  (no manifest, `.xz` logs only) are accepted as before; they are the C
  oracle by definition.
* `harness/check-no-c-borg.sh`: checks that the Rust binary has no C Borg
  code: no `src/borg/*.c` compiled; every staticlib member is a rustc
  object or one of 35 known compiler-builtins C objects; no C-compiled
  symbol with a C Borg name in the binary; every engine object
  byte-identical between `build/c` and `build/rust` (only `buildid.c.o`
  may differ); and (step 5) the actual link inputs: it re-runs the build's
  own link command (`CMakeFiles/OurExecutable.dir/link.txt`) with ld's
  `--trace` given twice into a temporary file, requires the result to be
  byte-identical to the binary, and then requires every input file and
  archive member to be one of the engine objects of step 4, a staticlib
  member accepted by step 2, or a system file under `/usr/lib`, `/lib` or
  `/lib64` (crt objects, libc, libm, libgcc, X11). It fails on empty or
  implausibly small inputs (no C Borg objects, stripped binary). It
  matches by name and member, so C code renamed on purpose would not be
  caught, and system libraries are trusted as a class.

`harness/borg.txt` is the stock `src/borg/borg.txt` (cheat death off,
`borg_delay_factor = 0`).

## Build identity

The key oracle only proves something about the binary that produced the
logs. Since review finding 4 of the external review (`REVIEW-4c06cec08`),
the harness ties a suite output to a binary and the binary to the sources:

* **`BUILD_ID`.** `harness/build.sh` writes `build/<impl>/BUILD_ID` next to
  the binary: `impl`, `git_rev`, `git_dirty` (whole checkout),
  `inputs_dirty`, `source_sha256`, `binary`, `binary_sha256` (after
  linking), the C compiler and `rustc` versions and the build time.
* **Source hash.** `harness/identity.sh source-hash IMPL` is sha256 over
  `<git blob id> <path>` of every build input as it is in the working
  tree: the files git tracks plus untracked files not ignored, under
  `angband/` for `c`, and under `angband/`, `borg-rs/`, `Cargo.toml` and
  `Cargo.lock` for `rust`. It depends only on contents and names, so
  committing a change or editing `docs/` or `harness/` does not change it,
  and a Rust change does not make the C binary stale. (The binary itself
  embeds `git describe` of the checkout through Angband's `version.sh`,
  so the binary's sha256 does change with every commit after a rebuild.)
* **Implementation.** `harness/identity.sh binary-impl BIN` reads the
  executable: `rust` if its `.comment` section names rustc (the Rust
  staticlib is linked in), `c` if not and it defines `do_cmd_borg` (the C
  Borg is linked in), `unknown` otherwise.

`harness/suite.sh` checks, before any run, and refuses with exit 2:

| check | refused when | override (recorded in the manifest) |
|---|---|---|
| implementation | the executable, `CMakeCache.txt` and `BUILD_ID` of the binary's build dir disagree, or the implementation they give is not `BORG_IMPL` (default `c`) | none |
| implementation | none of the three knows the implementation (e.g. a stripped external binary) | `ALLOW_UNKNOWN_IMPL=1` |
| build identity | no `BUILD_ID`; the binary's sha256 differs from `BUILD_ID`'s (binary changed after the build); `BUILD_ID`'s `source_sha256` differs from the checkout's (stale binary: rebuild) | `ALLOW_STALE_BUILD=1` |

An override that was needed is recorded as `overrides=...` and in
`impl_check=` / `build_check=` of the manifest, and `harness/diff` then
fails on that output unless given `--allow-unverified`. So the wrong-binary
command of the review,
`BORG_IMPL=rust ANGBAND_BIN=build/c/game/angband harness/suite.sh smoke`,
now stops before running, and an output dir made with an override cannot
pass the diff by default. To check a Rust build:

    BORG_IMPL=rust harness/suite.sh full
    harness/diff --suite full --expect-impl rust harness/out/rust/full harness/reference/full

Limits: make and cargo decide what to rebuild from timestamps and
fingerprints, so `BUILD_ID` trusts the build system that the binary was
rebuilt from the inputs it hashes; the toolchain is recorded but not
part of the source hash; the checks protect against mistakes, not against
someone forging a manifest or a `BUILD_ID`. The references are made from
the C build; `smoke`, `full` and `deep` carry no manifest (they predate
it); `classes` keeps the manifest of the C run that made it (no build
identity), and `config` the full manifest of its C run (checked like a C
suite output).

## Suites and reference logs

| suite | runs | what it covers |
|---|---|---|
| `smoke` | 3 x 20k game turns | Human Warrior, Elf Mage, Dwarf Priest |
| `full` | 45 x 100k | early game: clevel 1-10 (median 4), max depth 1-5 (median 2), 15/45 die (all by clevel 5); 8 classes, 8 races (see the quirk) |
| `deep` | 9 x 1M, cheat death | clevel 8-30 (median 18), max depth 2-26 (median 7); 8 classes, 5 races |
| `classes` | 15 x 100k + 1 x 1M cheat death, flag `menu` | Blackguard with 7 races, Dunadan, High-Elf and Kobold with 7 other classes, a real Half-Elf Ranger; the 1M Dunadan Blackguard reaches clevel 26, max depth 22 |
| `config` | 6 runs of 100k/20k + 1 x 1M, flags `menu,cheat` plus `munchkin` or `lunal` | Borg config options: munchkin start and lunal mode (see below) |
| `options` | 25 runs (22 x 100k, 200k, 400k, 1M), flag `menu`, `cheat` or `cheatlive`, one `NAME=VALUE` option each | the other borg.txt options: money scum, self scum, dynamic calcs, worships, risky, uniques, swaps, depth/level stops, autosave, verbose, chests, munchkin limits, respawn (see below) |
| `family-buying` | 8 x 20k, Rust-only | current vs reserve410, direct and class/mode-selected pairs; validator `harness/regress/family-buying-test.py` |
| `family-identify` | 4 x 20k, Rust-only | `borg_buy_identify = 1` across both strategies; C ignores the switch and never learns, so a C run would fail the validator |

`harness/reference/<suite>/<run>/{keys.log.xz,messages.log.xz,state.log.xz,summary.json,config.txt}`
are the C Borg's logs for each suite, made from the C build. They are the
oracle a build is checked against:

    BORG_IMPL=rust harness/suite.sh full
    harness/diff --suite full --expect-impl rust harness/out/rust/full harness/reference/full

The suite oracles were refreshed for the v12R carry, on the imported
`42cd156ac` engine with the consumer's milestone 12 Borg: the C build ran
each suite **twice**, the two passes were identical on `keys.log`,
`messages.log`, `state.log` and `summary.json` (apart from `wall_seconds`),
and one pass was stored as the `.xz` logs above, with the suite's
`manifest`, `timing.txt` and `DONE` marker. That covers `smoke`, `full`,
`classes`, `config` and `config-six`; the `options` oracle is still the
M1-era recording (on the imported engine that suite costs about four hours
per C pass here, and it was not affordable in the refresh session). The
digests of every stored file are in
`docs/validation/2026-11-13-c-v12r/oracle-digests.json`; the map of
which corpus is the current oracle and which is frozen investigation
evidence is in [harness/reference/README.md](../harness/reference/README.md).

The oracle (keys, messages, state digest) does not see screen output,
the Borg's internal state (only through keys and notes), engine state
outside the digest such as the cave, monsters, floor objects and stores
(only through later keys, messages or player state), anything after the
turn limit, or a Borg config option in a way no suite run plays (the
`config` and `options` suites change one option per run; all other
suites run with the stock `borg.txt`; see "The options suite" for what
each option reaches).

### The config suite

**2026-11-14 (borg-rs#60):** the `config` and `lunal-investigation` suites
are removed, and the mode cases of `options` (`opt-munchkin-limits-war`) and
`synthetic-tactics` (`synth-lunal-*`) with them; `family-buying`'s warrior
pair uses the class masks. Their references under `harness/reference/` stay
as the record. The description below is that record.

`harness/suites/config` replays the runs the second external review
(`REVIEW-821cad310`) made by hand, plus one long run. Every run uses
`menu,cheat` and one config flag:

| runs | config | characters | turns |
|---|---|---|---|
| `cfg-munchkin-*` | `borg_munchkin_start = TRUE` | Human Warrior seed 42, Half-Elf Mage 12345, Dunadan Blackguard 1080 | 100k, 100k, 20k |
| `cfg-lunal-*` | `borg_lunal_mode = TRUE` | the same three | 100k, 100k, 20k |
| `cfg-lunal-deep-warrior` | `borg_lunal_mode = TRUE` | Human Warrior seed 2007 | 1M |

The Blackguard munchkin run stops at 20k game turns because it is slow:
137k keys by turn 20,011. The suite takes about 4 minutes with `JOBS=2`.

What it reaches (checked once with a temporarily instrumented C build, not
committed):

* Munchkin: `borg_think_dungeon_munchkin` runs in all three munchkin runs
  from dungeon level 1 on, and `munchkin_mode` changes the stair choice.
  The munchkin runs reach clevel 8-12, max depth 2-8.
* Lunal: `borg.lunal_mode` is set from the config at start and stays on in
  all these runs. It changes the flow code: down stairs are taken without
  the prepared check, and resting is skipped. But the lunal thinker
  `borg_think_dungeon_lunal` is never entered: its only caller is the
  self-lunal check in `borg_think_dungeon` (C `borg-think-dungeon.c:1439`),
  which needs `borg_self_lunal` and a borg standing at least 15 levels
  above its max depth, prepared for 15 levels deeper, and deeper than
  clevel / 3.
* Self-lunal: not covered. With `selflunal` added, eight 1M cheat-death
  runs (seeds 2000, 2001, 2005, 2007 with and without `lunal`, and the
  `classes` Dunadan Blackguard 3100) never passed that check; the five
  without `lunal` have key logs identical to their `deep`/`classes`
  references, since `borg_self_lunal` is read nowhere else. So the
  `selflunal` flag exists, but no suite run uses it.

`harness/reference/config` was made like the other references: the C build
ran the suite twice, the two outputs were identical, and one of them was
stored as `.xz` logs, `summary.json`, `config.txt`, `timing.txt`, its
manifest (which records each run's flags and `borg.txt` hash as
`run_config.<run>`) and its `DONE` marker. Because the manifest has the
build identity, `harness/diff` treats this reference as a compressed C
suite output: it prints the C build's identity and checks it (C, no
override, `DONE`), unlike the older references, which have no such
manifest.

### The options suite

`harness/suites/options` sets, per run, one `borg.txt` option (two for
the money-scum and munchkin-limit runs) with a `NAME=VALUE` flag; all
runs use `menu`, all but the respawn runs `cheat`. Characters are those
of `config`/`smoke` (Human Warrior 42, Half-Elf Mage 12345, Dwarf Priest
103) and of `deep` (Human Warrior 2000, Hobbit Rogue 2006; their indices
are below 7, so `menu` picks the same characters as in `deep`), so each
run can be set beside the same character on the stock `borg.txt`.

What the runs reach was checked once with a temporarily instrumented C
build (not committed): every read of a `borg_cfg[...]` option and a few
sites (`borg_money_scum`, "Money Scum complete", the dynamic branch of
`borg_power`, `borg_reincarnate_start/end`) printed to stderr. "Keys
change" compares the key log with the same character and the stock
`borg.txt` (same flags otherwise): the first differing key's game turn,
or "same" if the whole log is identical.

| run | option(s) | reached (instrumented C) | keys change |
|---|---|---|---|
| `opt-scum-war`, `opt-scum-mage` | `borg_money_scum_amount=5000` | the "stay in town" branches (`borg-think-dungeon.c` 1984, 2032) and `borg_money_scum()` (waits for townspeople) from turn 542 / 201 on; both stay in town at clevel 1 for 100k turns | turn 1 / 181 |
| `opt-scum-noself-mage` | `borg_money_scum_amount=1000`, `borg_self_scum=FALSE` | as above, and the `!borg_self_scum` store branch (`borg-think-store.c` 167) | turn 111 |
| `opt-scum-done-mage` | `borg_money_scum_amount=300`, `borg_self_scum=FALSE` | gold is already above 300: "Money Scum complete" (`borg.c` 192), the Borg stops (`borg_stopped`, 2 keys) | turn 1 |
| `opt-dyn-war`, `-mage`, `-priest` | `borg_uses_dynamic_calcs=TRUE` (stock formulas) | the start-up warning (answered by the frontend, `startup_acks: 1`), formula loading, the dynamic branch of `borg_power` and of `borg_prepared` (`borg-prepared.c` 546, 701) throughout | warrior: turn 1, then a different game (clevel 14 instead of 8); mage, priest: one extra first key (the Borg clears the warning message), then the stock key stream |
| `opt-wdam-war` | `borg_worships_damage=TRUE` | the six damage bonuses in `borg_power` | turn 33439 |
| `opt-wspeed-war` | `borg_worships_speed=TRUE` | the speed check in `borg_power`; its bonus needs speed +25 or more, never reached | same |
| `opt-whp-war` | `borg_worships_hp=TRUE` | the CON/HP bonus in `borg_power` | same |
| `opt-wmana-rogue` (1M) | `borg_worships_mana=TRUE` | the SP bonus in `borg_power` | turn 964861 |
| `opt-wac-war` | `borg_worships_ac=TRUE` | the AC bonus in `borg_power` | turn 6498 |
| `opt-wgold-war` (200k) | `borg_worships_gold=TRUE` | `borg-junk.c` 387, `borg-store-sell.c` 847, `borg-think-dungeon-util.c` 660, 674, 802 | turn 168184 |
| `opt-risky-war` | `borg_plays_risky=TRUE` | caution, escape, preparedness and stair checks | turn 6334 |
| `opt-uniques-war` | `borg_kills_uniques=TRUE` | the living-uniques depth check (`borg-prepared.c` 631) | turn 40274 |
| `opt-noswap-war` | `borg_uses_swaps=FALSE` | `borg_uses_swaps()` returns false (swap scans skipped) | same |
| `opt-nodeeper-war` | `borg_no_deeper=3` | "No deeper" in `borg_prepared` and the climb in `borg-think-dungeon-util.c` 874 | turn 31194 (max depth 2 instead of 8) |
| `opt-stopdlevel-war` | `borg_stop_dlevel=2` | "Auto-stop for user DLevel" (`borg.c` 174): stops at turn 20833 | at the stop |
| `opt-stopclevel-war` | `borg_stop_clevel=5` | "Auto-stop for user CLevel" (`borg.c` 183): stops at turn 27889 | at the stop |
| `opt-autosave-war` | `borg_autosave=TRUE` | a save on each new level (`borg-update.c` 2347) | turn 11 |
| `opt-verbose-war` | `borg_verbose=TRUE` | the verbose notes (`borg-io.c`, `borg-messages.c`, `borg-flow-*`, `borg-store-buy.c`, ...) | same (notes only) |
| `opt-chest-war` (400k) | `borg_chest_fail_tolerance=0` | the chest check in `borg-flow.c` 789, once, at turn 307489 | same |
| `opt-munchkin-limits-war` | `munchkin`, `borg_munchkin_level=5`, `borg_munchkin_depth=4` | the munchkin level in `borg_power` and the munchkin depth in `borg_think_dungeon` | turn 7608 against the stock run, turn 10018 against `config`'s `cfg-munchkin-human-war` (munchkin with the stock limits 12 and 16) |
| `opt-respawn-druid` | `cheatlive` (game `cheat_live` on, `borg_cheat_death` off), stock `borg_respawn_race/class` (the run's race and class, set by `run.sh`) | five deaths, each `borg_reincarnate_start` (race 3, class 2 from `borg_respawn_race/class`) and `borg_reincarnate_end`; death menu, new game and birth through the Borg's keys | the first 880 keys are those of `full-2-druid-1` (no cheat death), which stops at its death at turn 5865; here the Borg answers "Die?" with `y` and goes on |
| `opt-respawn-random-druid` | `cheatlive`, `borg_respawn_race=-1`, `borg_respawn_class=-1` | one death; random race and class (`randint0`) give a Hobbit Rogue; both indices are below 7, so the wrong letters of C bug #11 do not show | as above up to the death |

So 20 of the 25 runs play a different key stream from the stock
`borg.txt` (for `opt-dyn-mage` and `opt-dyn-priest` only by one extra
first key); `opt-wspeed-war`, `opt-whp-war`, `opt-noswap-war`,
`opt-verbose-war` and `opt-chest-war` read their option where it matters
but keep the stock key stream (the option's branch is evaluated with the
new value, but no choice changes within these runs). Longer tries did no
better: `borg_worships_speed` and `borg_worships_hp` on 1M `deep` runs
(Human Warriors 2000, 2007; Dunadan Paladin 2005; Half-Troll Blackguard
2008), `borg_worships_hp` on the `classes` Paladin and Blackguard,
`borg_uses_swaps=FALSE` on 1M runs 2000, 2005, 2007 and
`borg_self_scum=FALSE` alone on 1M runs 2000 and 2007 all kept the
reference key logs. `borg_enchant_limit=9` on 1M run 2000 was never read
(no enchanting) and is not in the suite.

Not covered: `borg_respawn_winners` and `borg_stop_king` (need a win),
`borg_allow_strange_opts` (only matters with birth options the harness
does not set), `borg_delay_factor` (only sleeps), `borg_stop_on_bell`
(no bell in these games), `borg_restore_ignore_settings` (read when the
Borg is stopped by the user), `borg_dump_level` and `borg_save_death`
other than their stock values (they decide which files are written at a
death; the respawn runs read `borg_dump_level` with its stock value), `borg_self_lunal`
(see the config suite), and the dynamic-calcs formulas other than the
stock ones.

`harness/reference/options` was made like `reference/config`: the C build
ran the suite twice with `JOBS=2`, the outputs were identical, and one was
stored with `.xz` logs, `summary.json`, `config.txt`, `timing.txt`, the
manifest and `DONE`.

## Natural extended replay inputs

`harness/saves/captured-extended` retains30 fork-isolated engine captures
from the corrected28b8170 extended C/C/Rust matrix, with identical metadata
and direct bytes in all three passes. SOURCES.json and SHA256SUMS preserve
provenance. The named `natural` suite loads Warrior78/clevel50 and Dunadan
Blackguard40/clevel35 for100k additional turns; strict C/C and Rust/C2/2,
reaching84/41. Set `SAVE_ROOT=harness/saves/captured-extended` when running
and pass the same `--save-root` to `harness/diff`. Both C passes establish
the separately identified natural reference. These are fresh Borgs on
Borg-played cheat-death engine states, not restored private Borg memory or
stitched continuations. See the latest melee validation receipt.

### Synthetic engine tactic cases

The new six-case synthetic-tactics suite reaches the real self-lunal and
glyph callers with unchanged production code. C/C and Rust/C6/6 compare all
three logs plus complete final engine-save bytes (finalsave flag). The lunal
fixture has depth20/max99/clevel50 and stops at its first descent to21;
option-off and max34 controls have zero thinker entries. The glyph preparation
case consumes a real Rune scroll and adds an engine glyph; supply9 controls
have zero builder entries. A prelaid24-glyph case exposes existing#295's
recognition predicate. These are wizard states and bounded actions, not an
unassisted win or completed full rune tactic. See harness/scenarios/README.md.

## Maintained GUI save/reload validation

The archived M4 save-roundtrip.py preserves the original test, including its
window-exit timing race. New runs use harness/regress/launcher-save/run.py
with a required isolated --project-root and a private Xvfb display. The new
driver confirms game exit0 before tolerating BadWindow, then checks save
replacement, level50 and both exact archived save hashes. Four deterministic
race controls and five complete real roundtrips pass; see
[the driver README](../harness/regress/launcher-save/README.md) and
[review follow-up](validation/2026-11-07-m4-review-followup/README.md).

## Borg zoo measurements (2026-11-08)

Both `harness/build.sh` backends expose these additional borgh arguments.
Pass them after `--` in `harness/run.sh`; all are off by default.

| Option | Recording / stop |
| --- | --- |
| `-states FILE` | Native-observer-style `STATE` lines: `start`, `borg_start`, `progress` every 100 Borg think decisions, and the stop reason. Flushed per record. |
| `-stall-decisions N` | Stop as `stalled` after a decision-count delta of N at one unchanged game turn. Energy increases do not reset the clock. N must be positive. |
| `-exact-turns` | Stop from the world-turn increment at the positive `-turns` limit, before requesting another key. |
| `-random-birth` | Send the birth menus' `*` choices for race and class; the game seed chooses the character. Cannot combine with `-load`. |

Example (substitute an absolute output path):

```sh
BORG_IMPL=rust harness/run.sh /tmp/zoo-run 1045 0 0 100000 menu,munchkin -- \
  -states /tmp/zoo-run/states.log -stall-decisions 10000 -exact-turns
```

`STATE KIND` is followed by space-separated `name=value` tokens:
`turn energy decisions depth max_depth level max_level hp max_hp dead winner
borg_rng quick quick_rng rng_index rng`. These match the concepts and names
of better-borg-c's `tools/native-observer.c`; `max_hp` is an additional token.
`energy` is `player->total_energy`, the cumulative energy spent (including
resting), not the current action-energy budget. `rng` concatenates the 32
32-bit complex-RNG words as eight lowercase hexadecimal digits each.
The quick RNG is temporarily the Borg's RNG during a think call, exactly as
in the native observer; `quick` records that fact. Recording never draws RNG.

Every summary adds these fields, including when all options are off:

| Field | Meaning |
| --- | --- |
| `calendar` | `"fixed"`: the existing seeded non-holiday date remains unchanged. |
| `max_clevel` | Engine `player->max_lev`, including any earlier maximum before level loss. |
| `hp`, `max_hp` | Engine HP and maximum HP at the stop. |
| `borg_decisions` | Observed Borg think boundaries, including iterations that return false without yielding a key. |
| `player_energy` | Same cumulative energy as the stream's `energy`. |
| `borg_rng` | Borg's stored local RNG state (native observer's `borg_rng`). |
| `game_rng` | Object with `quick` (boolean), `quick_rng`, `rng_index`, and the same hexadecimal `rng` string as STATE. Captured before any requested final save. |
| `effective_settings` | Map from Borg setting name to parsed integer value; booleans are 0/1, as in native `BORG_OPTIONS`. Read from the live configuration, including parser adjustments, rather than reconstructed from `borg.txt`. Empty if the Borg never reaches a decision. |

Opt-in runs also add `exact_turns: true` or `random_birth: true` as
applicable. Random births add `played_race_idx` and `played_class_idx` from
the engine. Existing `race_idx`/`class_idx` continue to mean the requested
indices, preserving legacy menu behavior and references; `race`/`class`
continue to name the character actually played. Negative requested indices
remain rejected; the old header's misleading “-1 = random” text is removed.

A stall adds `stall_turn`, `stall_decisions` and `stall_energy_delta`.
The latter two are deltas from the first decision observed at the frozen
turn; the first observation establishes the baseline. Any change in turn
(including a backwards change on reincarnation) establishes a new baseline.
Stalling can stop inside the think loop without another key. As with an
exact world-turn stop, the last key can precede `summary.game_turns`.
`harness/diff` permits that explicitly declared boundary and validates stall
deltas. Ordinary legacy runs retain the last-key/summary equality check.

Keys remain the oracle. Pure observation and a nontriggering stall limit
leave all three existing log streams byte-identical. A triggered limit
retains their key prefix. `-exact-turns` deliberately omits the old final
key generated at/after the cap; it can therefore shorten the key log.
`-random-birth` deliberately changes the character and key stream versus a
fixed birth. It uses the same seeded choice as the observer; it cannot also
preserve a fixed character's keys. No Borg choice is changed by recording.

The checked-in C and Rust Borg files are untouched. Only harness builds
instrument their think-call boundary: C uses a forced compiler include on
`borg.c`; Rust builds a generated source overlay under
`build/<impl>/measurement-rust`, with a read-only FFI bridge in its engine
module. The two exact Rust insertion sites are checked and fail closed if
they change. Ordinary Rust builds use the original workspace. The adapters
copy live values synchronously, retain no Borg pointers, and call no game
callbacks. The harness adapter files participate in `BUILD_ID` source hashes.
Do not edit generated sources to implement gameplay changes. Harness unit-test
executables use separate no-op observation sinks; those sinks are never linked
into the game executable.

New fields are compared between new summaries. For historical references
that predate `calendar: "fixed"`, only the new measurement fields are skipped;
all existing fields and streams remain required. New-format summaries must
contain the complete added field set. No stored reference is rewritten.

Focused controls: `python3 harness/regress/measurement-test.py`. The
smoke/config option sweep and additional-suite baseline controls are in
`harness/regress/measurement-suites.py` and `measurement-remaining.py`.
Native-observer birth expectations record their donor source/result hashes
in `harness/regress/measurement-birth-reference.json`.
