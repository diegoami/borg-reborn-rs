# Provenance

*docs/SPEC.md §2.2: what was copied from another project, once. A copy is
ours from then on: it is changed or deleted as we need, and never kept in
sync. Licence of every source below: GPL-2.0.*

| Ours | Source | Commit | Path there | What we changed |
| --- | --- | --- | --- | --- |
| `engine/angband/` | angband/angband | `42cd156ac` (from `e473720`, #215) | the whole tree | nothing (byte for byte; `engine/UPSTREAM`) |
| `engine/patches/02`–`07` | diegoami/borg-rs | `286215c` | `angband/src/{game-world,z-rand,mon-make,message,ui-input,ui-game}.c` | only their `BORG_HARNESS` hunks; the `BORG_RECORDING` hunks are dropped |
| `engine/frontend/main-borgh.c`, `main-borgh.h` | diegoami/borg-rs | `286215c` | `angband/src/main-borgh.c`, `main-borgh.h` | trimmed to the contract of docs/SPEC.md §3.5: dropped `-states`, `-frames` and its diagnostics, `-make-fixture`, `-near-morgoth`, `-stop-rune-pattern`, `-cheat-live` (1,247 → 931 lines) |
| `engine/patches/01` | diegoami/borg-rs | `286215c` | `angband/src/main.c`, `main.h` | only the `USE_BORGH` hunks; the `main.h` declarations are guarded here |
| `engine/sources.cmake` | angband/angband | `42cd156ac` | `CMakeLists.txt`, `add_library(OurCoreLib OBJECT …)` | the list only, without the 59 `src/borg/*.c`; `engine/tests/test_build.py` checks it is upstream's minus those |
| `harness/run.sh`, `suite.sh`, `diff`, `identity.sh`, `build.sh`, `suites/smoke`, `suites/full` | diegoami/borg-rs | `e5cb263` | `harness/` | copied in T1.2 (#5) with three adaptations (our crate names in `identity.sh`, the death export in `run.sh`, the Rust overlay); re-founded on our own build in milestone E (TE.4, TE.5): `build.sh` builds `engine/` only, `identity.sh` hashes our inputs, `suite.sh` and `diff` accept only `rust`. The borg-rs logic that remains (the suite runner, the binary checks, the diff of keys, messages and states) is ours to change |
| `borg-reborn-rs/src/engine.rs` | diegoami/borg-rs | `e5cb263` | `borg-rs/src/engine.rs` | copied in T1.5 (#8), then cut to the reads of docs/ENGINE-READS.md and re-based on our generated bindings in TE.3; perception (`perceive`, `analyze`) is ours |
| `borg-core/` | diegoami/borg-rs | `e6d13fc` (tag `4.2.6-impact-v12`) | `borg-rs/` (the crate), `harness/borg.txt` (as `borg-core/borg.txt`), `angband/src/borg/borg-init.c` and `borg.txt` (as `borg-core/c-reference/`, test data) | vendored for option C (#207, #208; docs/ERRANDS.md): package renamed `borg-core` (the library keeps the name `borg_rs`), `rlib` only; its `#[no_mangle] do_cmd_borg` became a plain function the shell calls when `BORG_CORE=borg-rs`; three test `include_str!` paths repointed; clippy's lints allowed in its `Cargo.toml`. Unlike the copies above, it is **refreshed** from borg-rs at a new pin: `scripts/borg-core-vendor.py refresh REV` merges borg-rs's changes since `borg-core/PIN` into each file three ways, keeps our own files, writes `PIN` and lists conflicts; `delta` prints our changes as a patch and `check` shows nothing else differs (review #227, finding 9). `scripts/refresh-sources.py` reports borg-rs commits since `PIN` that touch the vendored files, and its `--update` never moves `PIN`; ours there: `src/shell.rs`, `src/params.rs` (#219, our calibration registry), `src/core_trace.rs` (#222, our named decision recorder), `src/memory.rs` (#223, Track R source types), `src/shadow.rs` (#224, Track R boundary hooks), `ladders.txt`, `PIN`, and the hooks marked `borg-reborn-rs (#207)`/`(#208)`/`(#219)`/`(#220)`/`(#221)`/`(#222)`/`(#240)`/`(#241)`/`(#242)`/`(#243)`/`(#223)` (checked cave access for Track R; shadow wrappers in danger/caution/think (#224); attack/defence/item-use/power parameters, trace hooks in think/dungeon/io, parameter initialization in `borg_init.rs` and named thresholds in caution, danger, escape and recovery; docs/CALIBRATION.md). the engine is the upstream commit borg-rs v12 plays (#215); its fork correction in `cmd-wizard.c` (debug mode) is not carried |
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

Optional recorder crate/host adapter v0.1.0 at c4986b26b9fd1c14852df4f1b7393d9030cf812a. See recorder-pin.json and docs/ANGBAND-RECORDER.md. Donor credits are retained in the vendored CREDITS.md; engine/Borg policy is unchanged.

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
[Integration decisions](M13-INTEGRATION.md) enumerate conflicts, the native
read-only query patch, receiving option ABI adapter and parameter migration.
Engine/harness, recorder and cemetery integration remain receiving-owned.

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
