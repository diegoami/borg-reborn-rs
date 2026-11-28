# Provenance

*docs/SPEC.md §2.2: what was copied from another project, once. A copy is
ours from then on: it is changed or deleted as we need, and never kept in
sync. Licence of every source below: GPL-2.0.*

| Ours | Source | Commit | Path there | What we changed |
| --- | --- | --- | --- | --- |
| `engine/angband/` | angband/angband | `574b618c` (from `42cd156ac`, #724; from `e473720`, #215) | the whole tree | nothing (byte for byte; `engine/UPSTREAM`) |
| `engine/patches/02`–`07` | diegoami/borg-rs | `286215c` | `angband/src/{game-world,z-rand,mon-make,message,ui-input,ui-game}.c` | only their `BORG_HARNESS` hunks; the `BORG_RECORDING` hunks are dropped |
| `engine/frontend/main-borgh.c`, `main-borgh.h` | diegoami/borg-rs | `286215c` | `angband/src/main-borgh.c`, `main-borgh.h` | trimmed to the contract of docs/SPEC.md §3.5: dropped `-states`, `-frames` and its diagnostics, `-make-fixture`, `-near-morgoth`, `-stop-rune-pattern`, `-cheat-live` (1,247 → 931 lines) |
| `engine/patches/01` | diegoami/borg-rs | `286215c` | `angband/src/main.c`, `main.h` | only the `USE_BORGH` hunks; the `main.h` declarations are guarded here |
| `engine/sources.cmake` | angband/angband | `574b618c` | `CMakeLists.txt`, `add_library(OurCoreLib OBJECT …)` | all 209 upstream core sources, including the 59 `src/borg/*.c` sources. `engine/tests/test_build.py` checks the lists match |
| `harness/run.sh`, `suite.sh`, `diff`, `identity.sh`, `build.sh`, `suites/smoke`, `suites/full` | diegoami/borg-rs | `e5cb263` | `harness/` | copied in T1.2 (#5) with three adaptations (our crate names in `identity.sh`, the death export in `run.sh`, the Rust overlay); re-founded on our own build in milestone E (TE.4, TE.5): `build.sh` builds `engine/` only, `identity.sh` hashes our inputs, `suite.sh` and `diff` accept only `rust`. The borg-rs logic that remains (the suite runner, the binary checks, the diff of keys, messages and states) is ours to change |
| `borg-reborn-rs/src/engine.rs` | diegoami/borg-rs | `e5cb263` | `borg-rs/src/engine.rs` | copied in T1.5 (#8), then cut to the reads of docs/ENGINE-READS.md and re-based on our generated bindings in TE.3; perception (`perceive`, `analyze`) is ours |
| `borg-core/` | diegoami/borg-rs | `e6d13fc` (tag `4.2.6-impact-v12`); refreshed since, now `8c5d000a` (#368, `borg-core/PIN`) | `borg-rs/` (the crate), `harness/borg.txt` (converted at vendoring to `borg-core/pborg.yaml`), `angband/src/borg/borg-init.c` and `borg.txt` (as `borg-core/c-reference/`, test data) | vendored for option C (#207, #208; docs/archive/ERRANDS.md; the errand shell, `src/shell.rs` and `ladders.txt`, was retired on 2026-11-27, #1339, and its configuration readers moved to `src/config_file.rs`): package renamed `borg-core` (the library keeps the name `borg_rs`), `rlib` only; its `#[no_mangle] do_cmd_borg` became a plain function the shell calls when the borg-rs core is selected (`BORG_CORE=borg-rs`, or by default since #606); three test `include_str!` paths repointed; clippy's lints allowed in its `Cargo.toml`. Unlike the copies above, it is **refreshed** from borg-rs at a new pin: `scripts/borg-core-vendor.py refresh REV` merges borg-rs's changes since `borg-core/PIN` into each file three ways, keeps our own files, writes `PIN` and lists conflicts; `delta` prints our changes as a patch and `check` shows nothing else differs (review #227, finding 9). `scripts/refresh-sources.py` reports borg-rs commits since `PIN` that touch the vendored files, and its `--update` never moves `PIN`; ours there: `src/shell.rs`, `src/params.rs` (#219, our calibration registry), `src/core_trace.rs` (#222, our named decision recorder), `ladders.txt`, `PIN`, and the hooks marked `borg-reborn-rs (#207)`/`(#208)`/`(#219)`/`(#220)`/`(#221)`/`(#222)`/`(#240)`/`(#241)`/`(#242)`/`(#243)` (attack/defence/item-use/power parameters, trace hooks in think/dungeon/io, parameter initialization in `borg_init.rs` and named thresholds in caution, danger, escape and recovery; docs/CALIBRATION.md); the full list, grown since, is `OURS` and `HOOKED` in `scripts/borg-core-vendor.py`. The engine was the upstream commit borg-rs v12 plays (#215), and is upstream `574b618c` since #724; its fork correction in `cmd-wizard.c` (debug mode) is not carried |
| `borg-reborn-rs/src/town.rs` | diegoami/borg-rs | `a7711a7` (`feature/town-session-tools2`) | `borg-rs/src/borg.rs:407-462` (the handover), `borg_town_dump.rs` (`dump_town_state`, `state_key`, the schema-3 record), `borg_object_log.rs` | rewritten on our observation (#191): the record's fields and order are borg-rs's; `traits` is `[]` (we have no C-Borg traits), a pack item's value is -1 (we have no value estimate), the quiver is added under `x`; the object log's purchases are paired with the next think's pack instead of a turn window, and it adds `eat`, `throw`, `fire`, `wield`, `sell`, `store` |
| `harness/play.sh` | diegoami/borg-rs | `a7711a7` | `harness/play.sh` | rewritten on our X11 build (`watch-borg.sh`): no `borg.txt` flags, no native recording, no decision dumps; the trace via `BORG_RUN_DIR` |
| `harness/feedback.sh` | diegoami/borg-rs | `a7711a7` | `harness/feedback.sh` | nothing |

`borg-reborn-rs/src/headers/` (c2rust bindings copied from borg-rs `e5cb263` in M1) was replaced in TE.3 by bindings we generate (`engine/bindings/generate.sh`); nothing of it remains.

`tools/death_registry.py` keeps the registry index of `tools/borg_deaths.py`
as T1.6 copied it from borg-rs `e5cb263` (`registry_lock`,
`canonical_identity`, `aggregate`; borg-rs took them from better-borg-c
`f3ced62`, `tools/aggregate-deaths.py`). `index.json` and the README's case
rows regenerate byte for byte; the README's closing instructions name our
tools instead of the removed ones. The rest of
the tools copied in T1.6 (recorded in `tools/IMPORTED.json` until then) were
removed in TE.6, and so were borg-rs's `angband/` tree, the `borg-rs/` shim
crate and `harness/measurement/`.

## Angband Recorder (owner integration, 2026-11-14)

Optional recorder crate/host adapter v0.3.1 at
`88c9324b9958b7b08fad161841fb1a7fc0c2a3f4`, adopted in #834 (v0.3.0 at
`5146ba6ff5187ec2320b9fd310dea466820db160` in #356; v0.2.0 at
`ee8b133ad22f66e7dad6e1b63a2d3e40b9a41ebd` in #351). All 58
release files remain verbatim and sealed in recorder-pin.json. The receiving
hook patch derives from the release adapter, preserving our harness hooks
and applying only to the generated upstream-574b618c copy. Donor licences
and credits remain in the vendor. The v0.1.0 pin was
`c4986b26b9fd1c14852df4f1b7393d9030cf812a`; its sessions retain their
original identity and are not migrated. See ANGBAND-RECORDER.md for the
schema boundary and consumer validation.

## M13 refresh (#328)

borg-core now vendors diegoami/borg-rs tag `4.2.6-rs-town-trip-m13-v1`,
peeled commit `1443c2d1b97d110ffbf5ae420d6c6b97ac15f7ea`, GPL-2.0-only.
The three-way refresh preserves declared OURS/HOOKED changes.
[Integration decisions](validation/M13-INTEGRATION.md) enumerate conflicts, the native
read-only query patch, receiving option ABI adapter and parameter migration.
Engine/harness, recorder and cemetery integration remain receiving-owned.

## Home-oscillation and additive-trip refresh (#368)

borg-core now vendors diegoami/borg-rs `8c5d000a58e9ddca1f1954ab9552e505856b313f`
(main, the merge of borg-rs#92), refreshed three-way from `1443c2d` with no
conflicts. It adds borg-rs#88 (the trip policy only adds return requests) and
borg-rs#92 (the live town planner does not undo a confirmed Home move within
one town visit, borg-rs#90). Both act only with `borg_town_plan = 1` or
`borg_trip_policy = 1`.

## Recorded wild-cat incident fix (#350)

Receiving-owned hooks remove the retired opportunistic stair-loot path from
borg_think_dungeon_util/borg_flow_take, keep known monster squares out of
GOAL_FLEE movement, and correct ordinary healing and short-escape decisions.
The additional donor deviations in borg_flow.rs and borg_flow_take.rs are
explicitly declared in the vendor checker. These intentionally change vendored
core play; the recorded M13 parity evidence retains its original source.
No engine or recorder implementation is copied or patched for this fix.

## Class/family preferences (#403)

Receiving-owned attack_preference.rs implements the #396 design: explicit BF
classification, named class/family parameter selection, pure widened floor
arithmetic and strict ranking. The only native policy hook is in borg_attack;
params.rs retains resolved origins and core_trace.rs records candidate/winner
data as core-trace@2. No donor policy or engine code is copied for this feature.
The new module is declared OURS by scripts/borg-core-vendor.py. Native helpers,
including their availability checks, RNG order and internal target selection,
remain unchanged. R must independently carry the accepted contract after P review.

## Synthetic P1 startup review fixes (#371 / #378)

The owner-supplied #378 review identified the vendored core's two native birth
option rejections. The narrow `borgh`-feature authorization follows the approach
also present in competing PR #386 at `72dbcbaa5c43499dc094d6ab36d3bc832743bc04`:
only the validated headless synthetic request permits force-descend/no-recall;
other birth checks and non-harness behavior remain in force. The guarded native level-transition wrapper also follows that PR; it calls
the ordinary leave/generate/arrival lifecycle before Borg activation. No Borg
strategy/tactical policy code is copied from that PR. Its competing primitive
must not be merged on top of this implementation without explicit reconciliation.

## Plan cap lifecycle review (#447)

*(Historical: the plans were retired on 2026-11-27, #1339.)*

Receiving-owned `errands_plan` state supplies an independent depth threshold.
The declared hooks in `borg_prepared.rs` and `borg_think_dungeon_util.rs`
apply the phase limit independently and combine it with configuration for
depth correction, without changing cfg. The configured preparedness check
retains its original static-only position and refusal priority. The declared
`borg_reincarnate.rs` hook refreshes the limit at completed reincarnation.
Phase selection no longer clears errands or the core goal. No engine changes.

## M13 inventory calibration (#433)

Receiving-owned parameters append 362 entries at accepted source 68e39b7e
(donor 8c5d000a). Declared hooks in borg_power, borg_item_wear, borg_store_buy,
borg_store_sell, borg_think_store and borg_home_power substitute policy
weights/thresholds without changing neutral arithmetic, quantities or RNG
order. The source inventory and retained mechanics classification are in
M13-INVENTORY-PARAMETERS.md/.json. No engine, recorder or donor code is
updated. #293's explicit shopping replacement remains separate.


## Unseen bump observation (#781)

Receiving-owned hooks in borg-core's caution, flow, messages, update, attack
and escape modules remember a failed step with unseen melee feedback and act
on that bounded observation. The normal oil chooser is shared to preserve its
fuel reserve. No donor refresh or hidden engine monster read is added. Message
semantics were checked in upstream 574b618c `player-attack.c:738-743,767,840-860`,
`mon-desc.h:45`, and `mon-desc.c:143-165`; the engine is unchanged.

## Documentation moved here (#816, owner 2026-11-21)

This repository is the central place for our Borg documentation. These
documents of ours were written in our other repositories and moved here;
they are edited here from now on. They were taken whole at the commit
below; on the move only their links changed, to their new places (or, for
files that stayed behind, to the source at that commit). The official
user manual the strategy manual carried as a copy was not moved: it is the
Angband developers', untouched in `engine/angband/docs/`.

| Here | From | Commit |
| --- | --- | --- |
| `docs/strategy/` | diegoami/neo-angband-tracker `docs/c-manual/` | `dbde80762dd45857c7840f1af9df1bd39930d4ab` |
| `docs/c-borg/` (the whole `docs/c-borg/`, `c-bugs/`, `C-BUGS.md`, `borg-rs-c-bugs.md`) | neo-angband-tracker `docs/c-borg/`, `docs/c-bugs/`, `docs/C-BUGS.md`, `docs/audit/c-reports/borg-rs-c-bugs.md` | same |
| `docs/c-borg/fixes/` | diegoami/better-borg-c `docs/measurement/*.md`, `docs/VERIFICATION.md` | `cc4c3e43aa0a863ad2fc4adab8f930277d181900` |
| `docs/design/` | diegoami/borg-rs `docs/` (town, trip, family, shop, joint-policy, c-bugs) | `8c5d000a58e9ddca1f1954ab9552e505856b313f` |
| `docs/research/borg-agent/` | diegoami/borg-agent `knowledge/`, `docs/POST-MORTEM.md`, `docs/KNOWLEDGE.md` | `57a480ceffdd9d3ceaff80c24d40fd2a5b5d2b86` |

## P6 enchant/decurse extraction (#680)

The existing borg-rs 8c5d000a copies of `borg_item_enchant.rs` and
`borg_item_decurse.rs` gain named neutral policy controls; the vendor audit
marks these two files hooked. Existing declarations and notices remain. No
new donor source is copied or engine behavior adopted. The receiving parameter
registry adds an automated test of bounds, precedence and provenance only.
A recorded native harness run with an environment override exercises the
production decision; it is not an automated decision-path regression test. See [the contract](CALIBRATION.md#enchantdecurse-policy-680--p6).
