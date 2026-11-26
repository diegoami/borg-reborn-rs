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
| `engine/sources.cmake` | angband/angband | `574b618c` | `CMakeLists.txt`, `add_library(OurCoreLib OBJECT …)` | the list only, without the 59 `src/borg/*.c`; `engine/tests/test_build.py` checks it is upstream's minus those |
| `harness/run.sh`, `suite.sh`, `diff`, `identity.sh`, `build.sh`, `suites/smoke`, `suites/full` | diegoami/borg-rs | `e5cb263` | `harness/` | copied in T1.2 (#5) with three adaptations (our crate names in `identity.sh`, the death export in `run.sh`, the Rust overlay); re-founded on our own build in milestone E (TE.4, TE.5): `build.sh` builds `engine/` only, `identity.sh` hashes our inputs, `suite.sh` and `diff` accept only `rust`. The borg-rs logic that remains (the suite runner, the binary checks, the diff of keys, messages and states) is ours to change |
| `borg-reborn-rs/src/engine.rs` | diegoami/borg-rs | `e5cb263` | `borg-rs/src/engine.rs` | copied in T1.5 (#8), then cut to the reads of docs/ENGINE-READS.md and re-based on our generated bindings in TE.3; perception (`perceive`, `analyze`) is ours |
| `borg-core/` | diegoami/borg-rs | `e6d13fc` (tag `4.2.6-impact-v12`); refreshed since, now `8c5d000a` (#368, `borg-core/PIN`) | `borg-rs/` (the crate), `harness/borg.txt` (as `borg-core/borg.txt`), `angband/src/borg/borg-init.c` and `borg.txt` (as `borg-core/c-reference/`, test data) | vendored for option C (#207, #208; docs/ERRANDS.md): package renamed `borg-core` (the library keeps the name `borg_rs`), `rlib` only; its `#[no_mangle] do_cmd_borg` became a plain function the shell calls when the borg-rs core is selected (`BORG_CORE=borg-rs`, or by default since #606); three test `include_str!` paths repointed; clippy's lints allowed in its `Cargo.toml`. Unlike the copies above, it is **refreshed** from borg-rs at a new pin: `scripts/borg-core-vendor.py refresh REV` merges borg-rs's changes since `borg-core/PIN` into each file three ways, keeps our own files, writes `PIN` and lists conflicts; `delta` prints our changes as a patch and `check` shows nothing else differs (review #227, finding 9). `scripts/refresh-sources.py` reports borg-rs commits since `PIN` that touch the vendored files, and its `--update` never moves `PIN`; ours there: `src/shell.rs`, `src/params.rs` (#219, our calibration registry), `src/core_trace.rs` (#222, our named decision recorder), `ladders.txt`, `PIN`, and the hooks marked `borg-reborn-rs (#207)`/`(#208)`/`(#219)`/`(#220)`/`(#221)`/`(#222)`/`(#240)`/`(#241)`/`(#242)`/`(#243)` (attack/defence/item-use/power parameters, trace hooks in think/dungeon/io, parameter initialization in `borg_init.rs` and named thresholds in caution, danger, escape and recovery; docs/CALIBRATION.md); the full list, grown since, is `OURS` and `HOOKED` in `scripts/borg-core-vendor.py`. The engine was the upstream commit borg-rs v12 plays (#215), and is upstream `574b618c` since #724; its fork correction in `cmd-wizard.c` (debug mode) is not carried |
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

### Track R danger port (#281, R2)

`borg-next/src/danger.rs` adapts `borg-rs/src/borg_danger.rs` at runtime
pin e6d13fc9baff, with the already adopted Track P named parameters. The
copyright and GPL/Angband license are retained in the file. This is our
owned port, not a second vendored core. Trait reads use named R1 view
fields; temporary buffs, simulation flags, fear maps, glyphs, monster slots
and teleport exclusions come from a borrowed typed danger context.

`borg-core/src/threat_facts.rs` supplies immutable engine/table facts and
existing unported projection, grid, spell-legality and blow-classification
queries. Both arms intentionally share these dependencies. The port owns
physical/spell estimates, modifier ordering, per-monster combination and
whole-square aggregation. It adds no engine reads to the fair-information
surface beyond the reads the original danger code already makes. None of
these services exposes RNG, key emission, mutation or original danger
results. The projection query's unnecessarily mutable signature is narrowed
to a shared borrow without changing its logic.

### Track R stateful ports (#286, R2)

`borg-next/src/stateful/{caution,escape,recovery}.rs` adapts borg-rs v12
`e6d13fc9baff`'s borg_caution.rs, borg_escape.rs and borg_recover.rs, retaining
copyright/license notices and the already adopted Track P named parameters.
Native trait reads are re-expressed as named View fields; the port owns the
policy branches, priorities, goal mutations and action selection. Existing
unported operations are behind the shared closed dependency API, not copied
as a second engine. STATEFUL-SHADOW.md describes effect-once record/replay,
full state equality, nested entry, panic=abort handling and the source gate.
The 68 exact dependency signatures are R2-STATEFUL-DEPENDENCIES.json.

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

## Track R M13 carry (#343, R3 preflight)

The independently owned danger and stateful policy bodies are now carried
from accepted main `2f74e1c5d57ce2b41ded1b5b694174b91b51f265`, whose native
core is donor `1443c2d1b97d110ffbf5ae420d6c6b97ac15f7ea`. The v12 citations
above describe the initial R2 adaptation and its immutable measurements.
Five view fields preserve exact native BI_PSTUN/BI_VACID/BI_VELEC/BI_VFIRE/
BI_VCOLD values. Physical cold's temporary-resistance asymmetry is retained
exactly as in the donor. No independent gameplay improvement is claimed.

The optional `test-m13-danger` build calls the actual native and owned
estimators on declared constructed memories using loaded engine race/blow
tables. Test-only native accessors are unavailable in production builds.
The fixture performs no live-state writes, RNG draws or game effects; its
input-projection fault control deliberately clears the five carried fields
on a private port snapshot. It is a branch validation, never population
or survival evidence. Normal policies never call the native accessor.

## Track R attack port (#364, R3)

`borg-next/src/stateful/attack.rs` adapts `src/borg_fight_attack.rs` from
borg-rs M13 `1443c2d1b97d110ffbf5ae420d6c6b97ac15f7ea`, retaining its
copyright and license. R364-ATTACK-SURFACE.json records the original
function inventory and source digest; ported-functions.json registers the
owned policy functions for subsequent source-change watching. Seven public
entries comprise five stateful transactions and two scalar comparisons.
The native Default implementation remains in owned memory; the indexed
trait helper is replaced by borrowed named Read accessors.

The port owns attack scoring, ordering, mana-reserve arithmetic and action
selection. Shared dependency adapters supply engine metadata, existing
unported services and recorded effects. AttackRace and AttackKind contain
values only; race friendship is represented by a boolean. Static names and
tables retain immutable process-lifetime borrows. Motion captures both
output parameters and checks their initial values on replay. The existing
town journey forecast is an effect because it mutates memory; its adapter
returns only the turn count used by the owned reserve calculation.

R364-ADDED-DEPENDENCIES.json inventories the added signatures. The source
gate rejects an attack port delegating its own policy to the native attack
API. Runtime parity and entry coverage remain acceptance gates; registering
the source does not establish them or imply a gameplay improvement.

## Track R defence port (#365, R3)

`borg-next/src/stateful/defence.rs` adapts M13 borg-rs
`1443c2d1b97d110ffbf5ae420d6c6b97ac15f7ea`'s `borg_fight_defend.rs`, retaining
its copyright and license. R365-DEFENCE-SURFACE.json records all 57 native
functions and the sole public entry, borg_defend. The owned port has all 56
policy functions; its indexed trait helper becomes named borrowed reads.
R365-SOURCE-AUDIT.json and scripts/audit-defence-port.py compare function
bodies, quoted strings and every module constant after declared seam and
formatting normalization. The carry watcher registers all owned functions.

Defence owns choice ordering, scores, simulation writes and their native
restoration sequence. Two exact named adapters write confusion resistance
and Free Action during Grim Purpose simulation. The original boolean save
and 0/1 restoration remain unchanged, including for a nonbinary input.
The source gate admits only the exact adapters and forbids native defence
delegation. Other temporary buffs, lists and goals stay owned snapshot
fields checked by complete state equality.

R365-ADDED-DEPENDENCIES.json lists nine additional typed query/effect
adapters. Terminal geometry is an immutable plain-value record; race facts
use the existing pointer-free metadata. Attack and escape cross-calls remain
recorded services with their own independent shadow comparisons. Permanent
buff/preparation/flow policy, including borg_fight_perm, remains explicitly
shared. No new fair-information engine reads or ownership claim for those
services is introduced. Native-equivalent games, rare-path and restoration
fixtures, literal fault/re-port controls and death reconciliation remain
runtime acceptance gates.

## Track R item-use port (#435, R4)

borg-next/src/stateful/item_use.rs owns the item-use policy bodies adapted
from borg-rs runtime donor1443c2d1, with receiving Track P named parameters
at accepted source02c273db. Original GPL/Angband notices are retained.
The24 public entries have independent typed dispatch; all BI reads use Read.
ItemUseState and engine-derived activation/type lookup knowledge stay shared.
New dependencies are effect/query-once adapters for object logging, kind-effect
facts and no-light facts; existing inventory/spell/keypress APIs remain shared.
R435-NEW-DEPENDENCIES.json records the exact delta. The test-only constructed
fixture queues commands on copied memories; it is not actual engine consumption
or a natural-play claim. Runtime acceptance remains pending until the task's
exact native/check/switch validation and canonical evidence are complete.

The R4 availability predicate borg_equips_item now has an immutable
&BorgState signature on native and owned bodies, a typed ReadDecision and an
ordered query-once comparison without a state fork. Its previous mutable
public spelling was stronger than the body needed; all callers coerce their
borrow without changing branch semantics. The API dependency retains complete
input-state checking as a query. All genuinely stateful entries retain full
fork/return/post-state comparison. @5 telemetry partitions these roots;
previous @4 evidence is historical and explicitly readable, never upgraded.

## Track R power port (#434, R4)

`borg-next/src/stateful/power.rs` adapts the 11 production bodies from the
receiving `borg-core/src/borg_power.rs` at `99fa7cc5`, M13 donor `1443c2d1`
plus the accepted P4 carry #529. Original copyright/license notices remain.
Named memory projections, ordered API facts and entry markers are the seams;
valuation arithmetic and policy literals are unchanged. Dynamic formula policy
remains explicitly shared. `scripts/audit-power-port.py` checks correspondence;
it is not runtime validation or a new claim about game mechanics.

## Track R wear and identification (#436, R4)

The equipment_wear/equipment_identify modules adapt the receiving native wear
and ID bodies plus inventory's worth-ID helper after accepted power #434.
Original copyright/license headers remain. R436-SOURCE.json names every owned
body and the native receiving hashes. The exact seam patch folds back native
wear, ID, inventory and the shared note implementation. The latter retains
its original state logic; only constructed test builds can suppress engine UI.
Adjacent enchant/decurse policy remains shared pending accepted P contract #680.

### R437 store/home ownership (in progress)

`borg-next/src/stateful/{home_power,store_buy,store_sell,think_store}.rs`
re-express 87 production bodies from the corresponding vendored `borg-core`
modules at accepted feature commit `64acb1f1e5c269eff258d8696494b120051e9b87`.
Original license/copyright notices are retained. R437-SOURCE.json names all
bodies and receiving hashes; audit-store-port.py checks correspondence to
both the accepted native source and the owned code. Shared data services,
M13 planner scope and validation limits are in R437-HANDOVER.md. No donor,
engine, parameter-default or policy change is implied by this ownership pass.
