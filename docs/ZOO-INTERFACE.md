# The zoo contract

*docs/SPEC.md §4, §6; task TE.7 (#77). borg-zoo measures our Borg, and
every comparison is the zoo's. This is the interface the zoo's adapter
(`zoo/borgrs.py`, kind `borg-rs`, read at our borg-zoo pin `c5f1b042`)
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
| copies `harness/borg.txt` and writes it, with its profile's settings, as the run's `user/borg.txt` | the file exists. It is borg-rs's `borg.txt` (the vendored core reads it) with no stair-scumming and no town scumming, plus three keys this repository's Borg reads (#207, #208; docs/ERRANDS.md): `borg_reborn_core` (`borg-rs`/`1`, the shipped value since #479, or `reborn`/`0`), `borg_reborn_errand` (`none`, or a command line such as `/ladder`) and `borg_reborn_stopscared` (`off`, `flee` or a percentage). A zoo profile selects an arm by setting them. Since #479 (owner, 2026-11-17) the shipped file also sets the borg-zoo#170 speed30 bundle without its plan: `borg_town_plan = 1`, `borg_trip_policy = 1` with `borg_trip_unknown_stacks = 5` and `borg_trip_pack_slack = 1`, and 18 `borg_param_*` lines (outmatched healing, speed 30, 16 class attack preferences). Since #485 it sets `attack_arrow_point_blank_divisor = 5` (the general divisor, borg-rs's), `attack_arrow_point_blank_divisor_ranger = 2` and `attack_arrow_point_blank_divisor_rogue = 3` (per-class overrides; 0 = the general divisor); since #491 three more set `town_heal_reserve_hp_percent = 400`, `town_heal_curve_scale_percent = 100` and `town_heal_speed_percent = 66`. Since #493 it also sets `borg_town_w_gold = 0` and `borg_param_town_gold_reserve = 0`, `borg_param_town_gold_reserve_percent = 0` (the explicit gold reserve, off). Since #519 the shipped file has no formula section (Power, Depth, Restock): it was unused with the shipped `borg_uses_dynamic_calcs = FALSE`. A profile that names a key rewrites its line; one that omits it now gets the bundle's value, not borg-core's compiled default. This repository's own core reads nothing else of the file: its parameters come from its profile (`BORG_PROFILE`), with the defaults when none is given |
| copies `harness/plans/NAME` into the run's `user/` directory, as `user/NAME`, when a profile sets `borg_plan = NAME`, after checking the file against the specimen's pinned hash (`"plans": {NAME: sha256}` in the zoo catalogue; borg-zoo#123, adapter since borg-zoo `26d1044`, PR #125) | the file exists at the specimen's ref. `borg_plan` is a fourth key the shell reads (#305, #352; docs/GAME-PLAN-LANGUAGE.md); a relative name is resolved under the run's user directory. It needs `borg_reborn_core = borg-rs`, no automatic errand and no nonzero `BORG_ERRAND_BUDGET`. With `borg_plan` set the Borg either loads and validates the file or exits 2 before play (missing, unreadable or invalid file, a class without a section, the wrong core): a plan is never silently ignored. Two plans ship: `errands_plan.txt` and `errands_plan_ambitious.txt`. Since 2026-11-16 their phases are `max-depth N` caps (no errand; the core plays as itself under `borg_no_deeper` = N+1); `loop-levels` was removed and fails with exit 2. The zoo records the file's SHA-256 with the cell, and keeps it with the death evidence: a plan death reruns only with the same bytes |

## Plan specimens (borg-zoo#123)

- A specimen pins every plan it ships in the catalogue, `"plans": {NAME:
  sha256}`. The zoo build refuses a `harness/plans/NAME` whose SHA-256
  differs, and gives a profile with `borg_plan = NAME` that file as the
  cell's `user/NAME`.
- An invalid, missing or misconfigured plan exits with status 2 before
  play and writes no `summary.json`. The zoo records such a cell as
  `config_error`, never as a game.
- `borg_stopped` (the Borg gave control back and the game waits for a
  real key: `engine/frontend/main-borgh.c`) is its own outcome for plan
  specimens, not a death or a turn-cap stop.
- The zoo clears the ambient `BORG_RUN_DIR`, `BORG_CORE`, `BORG_ERRAND`,
  `BORG_ERRAND_BUDGET`, `BORG_PROFILE`, `BORG_PLAN` and `BORG_PARAM_*`, so
  a cell's arm comes from its `borg.txt` and plan file alone.
- The plans' bytes are not build inputs: `harness/identity.sh source-hash
  rust` does not change with them. A plan arm is told apart by the plan's
  SHA-256 and the `borg.txt` the zoo writes.

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
`clevel`, `depth`, `borg_stopped` and `blocked` (`STOP` in its adapter;
`clevel` since borg-zoo `2416bb9`, borg-zoo#48, which keeps `clev` for
older builds). The zoo never passes `-clev`, so no cell stops that way.

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

## Core calibration profiles (track P3, #244)

The pinned adapter serializes integer and boolean settings. Set
`borg_reborn_core: 1` to select borg-rs, or `0` for the original reborn
core. Native callers may use the text aliases `borg-rs` and `reborn`.
An absent selector defaults to reborn. Every other value (including a
boolean written as `TRUE`, an empty string, or integer 2) fails with exit
status 2 and source/value diagnostics. `BORG_CORE` takes precedence over
the file; clear it for profile-only measurement. The vendored core's `borg_reborn_errand` also fails with exit status 2
for invalid configured command lines; `none`, an empty string, or absence disables errands.
Clear `BORG_ERRAND` for file-only profile measurement. The pinned adapter
only serializes booleans/integers, so arbitrary errand commands require a
native caller; an integer/boolean is not a valid errand. See ERRANDS.md.

**The scary list (#518).** The shipped `harness/borg.txt` sets
`borg_scary_list = scary_monsters.txt`: the borg-rs core reads the named
scary monsters from `user/scary_monsters.txt` (docs/CALIBRATION.md) and stops
before play with exit 2 when the file is missing or invalid. The adapter
copies the file named by the arm's profile or, without one, by the written
`borg.txt`, from the specimen's `harness/`; pins its hash in the specimen
(`"scary_lists"`); appends an arm's `borg_scary_list_lines`; and records the
written file's hash with the cell (borg-zoo#228, #230). An arm returns to the
compiled C list with `borg_scary_list = none`. A later line for the same
monster replaces the earlier one, `monster | none` removes it, and a later
`fragility:` value replaces the earlier one.

A profile's
`borg_param_<name>` integer sets any of the 2145 entries in
[CALIBRATION.md](CALIBRATION.md). For example, a profile definition in
the zoo's `catalogue/profiles.json` is:

```json
{
  "description": "Exploratory recovery arm; no improvement claim",
  "settings": {
    "borg_reborn_core": 1,
    "borg_munchkin_start": false,
    "borg_munchkin_by_class": false,
    "borg_self_lunal": false,
    "borg_lunal_mode": false,
    "borg_param_recover_rest_sp_tenths": 8
  }
}
```

A matched control uses the same explicit mode settings and omits the
parameter override (default6). The specimen's pinned harness/borg.txt
sets `borg_reborn_errand = none`; a native text errand is not an integer
zoo setting. Clear ambient `BORG_CORE`, `BORG_ERRAND`, `BORG_PARAM_*` and
other policy overrides in the runner: environment values take precedence
over the file. Freeze specimen, profile definitions and their hashes for
each experiment. Parameter ranges are input bounds, not recommendations.

The adapter writes settings into `user/borg.txt` unchanged and hashes that
file. Core initialization rejects an unknown, duplicate or out-of-range
parameter with exit2, instead of silently falling back. The existing zoo
record classifies this as `harness_failure`; it is never a valid default
cell or survival. No adapter/catalogue code or pin was changed here.

For direct adapter calls without `BORG_RUN_DIR`, our borgh frontend derives
the diagnostic directory from the parent of its existing `-summary FILE`.
An explicitly set BORG_RUN_DIR retains precedence. Without either, no
automatic directory is selected. This makes `params-core.json` and
`trace-core.jsonl` accompany the native zoo summary too, without a new
command-line option or an environment policy override. Numeric key logs,
summary fields and game behavior retain their existing contracts.

`engine/tests/test_core_profile_contract.py` archives the exact zoo pin,
imports only that source and runs the actual `run_cell` serializer/adapter
with no BORG_* environment. It checks numeric core selection, two parsed
values with borg.txt origins, file/profile/binary hashes, trace output,
invalid-value rejection, and the original-core selector0. CI requires this
contract rather than silently skipping an absent pinned checkout.

The pinned adapter's cell record still says `config_checked: written file`
and does not add a params-core hash itself. Preserve the new native output
files and hash them alongside its config/profile/binary identities when
analyzing calibration runs. A trace names a root check, not every nested
scoring term; use the parameter snapshot and first key/state divergence as
well. The sweep handoff specifies the evidence and endpoints separately.

Ready-to-copy catalogue entries (control and rest-SP7/8) are in
[core-calibration-profiles.json](examples/core-calibration-profiles.json).
They explicitly disable dynamic valuation as well as special modes. The
`power_*` parameters govern the static power implementation; choosing
`borg_uses_dynamic_calcs = TRUE` delegates valuation to the existing
borg.txt formula subsystem and does not apply those static weights.

The owner-run sweep handoff is [CORE-SWEEP.md](CORE-SWEEP.md). It pins the
validated specimen and supplies fresh-suite/candidate-selection/confirmation
rules; it does not launch a population experiment.

## Class values (#520)

Any `borg_param_<name>` can have a value for one class:

- in `borg.txt` as `borg_param_<name>@<class> = <value>` (class names of
  `class.txt`: warrior, mage, druid, priest, necromancer, paladin, rogue,
  ranger, blackguard);
- in the environment as `BORG_PARAM_<NAME>__<CLASS>`, upper case as every
  `BORG_PARAM_*` name (shells cannot export a name with `@`);
- through the pre-#520 per-class names of the folded families
  (`attack_preference_<class>_<family>_permille`,
  `attack_arrow_point_blank_divisor_<class>`,
  `town_heal_curve_scale_percent_<class>`, `town_heal_reserve_hp_percent_<class>`),
  kept as aliases. At their old "use the general value" sentinel (0, 0, -1)
  they set no class value.

Precedence: compiled default < `borg.txt` < a `borg.txt` class line <
`BORG_PARAM_<NAME>` < `BORG_PARAM_<NAME>__<CLASS>`. The character's class picks
its values (also after a reincarnation into another class).

Since #520's second half, the core's own `borg.txt` settings (`borg_*`,
`BORG_SETTINGS` in `borg-core/src/borg_init.rs`, e.g. the shopping weights
`borg_town_w_*`) have class values too, as `borg_<name>@<class> = <value>`
lines. The character's class applies at birth and at every reincarnation; a
setting with no line for that class keeps its common value. A line with an
unknown class or setting, or no value, is refused with a Borg warning. The
keys the shell reads (`borg_reborn_*`, `borg_plan`, `borg_scary_list`) and
`borg_respawn_*` have no class value, and there is no environment form for
these settings. A zoo profile cannot write such a line (`BORG_TXT_SET` names
are `[A-Za-z0-9_.-]`); it rewrites the common line, and a class line in the
shipped file still wins for its class.

The repository keeps each class's settings in `harness/borg_<class>.txt`;
`scripts/borg-class-files.py fold` writes them into `harness/borg.txt` as a
generated "CLASS FILES" block (under the legacy names where one exists, so a
profile naming that key still rewrites its line). A run, and the adapter, read
only `borg.txt`, so nothing else needs copying and `borg_txt_sha256` covers
the class values.

`params-core.json` keeps one row per parameter (now 2,116) and adds
`class_values: {"<class>": {"value", "origin"}}` where a class has its own.
The core trace's `parameter_name` for an attack candidate is the family's
parameter (`attack_preference_spell_permille`); the row's `class_name` gives the
class.

## Class/family preference profiles (#403)

Since #520 there is one preference per family, `borg_param_attack_preference_<family>_permille`,
and a class's own value is a class value of it. The 54 per-class names
`borg_param_attack_preference_<class>_<family>_permille` are still accepted, as
aliases of the class values, so existing profiles work unchanged. All default to
1000; bounds are 500..2000. See [ATTACK-PREFERENCES.md](ATTACK-PREFERENCES.md) for family semantics,
floor rounding, precedence and provenance. For example a profile can set
`borg_param_attack_preference_mage_spell_permille` to 1600. The registry default is 1000; since #479
the shipped `harness/borg.txt` sets 16 of them (the borg-zoo#170 speed30 bundle). Core trace output is now core-trace@2, retaining
the old fields and adding per-invocation candidate/winner data. Historical @1 data
and zoo profile/adapter formats remain unchanged. Freeze effective settings and
hashes and clear ambient BORG_PARAM_* overrides in profile-only comparisons.
