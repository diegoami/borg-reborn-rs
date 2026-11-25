# R4.5 carry report (#856)

The accepted R4 snapshot is `40cbaf272193abb44dd80fad2cfeec24b627be22`.
The owner-selected native baseline is `24be98732748230062882fe2b7d9601dba5e2cc0`.
The [rebuild handover](handover/2026-11-25-856-r45-rebuild.md) records the
local commits, build identities and run lineage. Full-suite evidence and
owner review are pending.

## Owned source

Physical added and removed lines from accepted R4 through `4f7f1323` in the
owned runtime files. Reproduce with
`git diff --numstat 40cbaf27..4f7f1323 -- borg-next/src/danger.rs borg-next/src/stateful`.
The counts include imports, comments, formatting, dispatch and deletions.

| File in `borg-next/src` | Added | Removed |
| --- | ---: | ---: |
| `danger.rs` | 19 | 4 |
| `stateful/attack.rs` | 1,292 | 205 |
| `stateful/attack_preference.rs` | 278 | 0 |
| `stateful/caution.rs` | 3,298 | 184 |
| `stateful/defence.rs` | 63 | 19 |
| `stateful/equipment_wear.rs` | 2 | 1 |
| `stateful/escape.rs` | 838 | 123 |
| `stateful/fast_fragile.rs` | 400 | 0 |
| `stateful/item_use.rs` | 70 | 184 |
| `stateful/mod.rs` | 20 | 10 |
| `stateful/recovery.rs` | 665 | 19 |
| `stateful/store_buy.rs` | 31 | 6 |
| `stateful/store_sell.rs` | 25 | 23 |
| `stateful/think_store.rs` | 4 | 0 |
| **Total** | **7,005** | **778** |

New modules in the receiving tree have the following ownership at this pin.

| Module | Policy and dependency |
| --- | --- |
| `attack_preference` | Owned scoring, ordering and winner selection. Common ranking data types remain in the core. |
| `borg_fast_fragile` | Owned fight policy, using typed monster and item queries. |
| `borg_cursed` | Shared curse classification via `harmful_curse`. Dungeon curse removal stays in the native thinker. |
| `borg_kit` | Shared `leave_danger_percent` query. Kit tracking and leash management stay in the native thinker. |
| `borg_lightless` | The native thinker retains lightless tracking and route selection. The owned `LightlessSelfCare` decision handles immediate treatment and food. |
| `borg_town_hunt` | Shared `viewed_target`, `avoids_maggot`, and `controls_target` queries. Town hunting orchestration stays in the native thinker. |
| `scary_list` | Shared selection, matching, and reason lookup. The owned callers decide how to use those results. |

Stair and flow operations, preparation and its buffer updates, and the
existing town planner remain behind named interfaces. The receiving source
audit records 603 paired bodies, including 31 reviewed rewrites, and the
unpaired native definitions. [Source receipt](R45-PORT-SOURCE.json)

## Validation and cost

The recovered native merge and smoke handover cover a 37-minute elapsed
window between commits `a18e3486` and `4f7f1323`. The window includes builds,
tests and simultaneous work. Exclusive implementation time was not recorded.
The shipped three-start suites took about 8 seconds in `off`, 261 seconds in
`check`, and 237 seconds in `switch` in the local cloud environment.

The first receiving check failed on turn 1: native caution requested
`engine::r_max` while the old owned body requested escape. The receiving
port fixed the dependency order. The shipped smoke runs then matched
pinned main in keys, messages, state and summaries. The local coverage gate
also exercised all 73 named entries using smoke and copied-memory controls.
The later seeded teleport control checked the acceptance boundary and danger
rejection in both modes. A temporary `>` to `>=` change in the owned rule
stopped replay at the unexpected refusal note. The clean port passed again
after restoration.

## Full-suite handoff

Use the fixed receiving pin as the native control, with the rebuilt specimen
as the candidate. All arms use the shipped profile with no parameter changes.
The shipped `harness/borg.txt` SHA-256 is
`dd21cc3e95f5ac0dc504d1f821f148befe356df74d4eb69b17d3b549edc4e046`.
The cell environment needs `BORG_SHADOW=off` for native and `check` or
`switch` for the candidate, plus a unique `BORG_RUN_DIR` for each shadow cell.
Record the effective mode and retain `shadow.jsonl`, `shadow-stats.jsonl`,
keys, messages, state, summaries, execution status and build identity.

The read-only inspection of Zoo's pinned and current Borg Rust adapters
found that inherited `BORG_*` environment settings are cleared and there is no
explicit `BORG_SHADOW` arm field. Zoo needs a per-arm mode and artifact path
before a pilot can establish shadow parity. Its owner dispatches the pilot.
[Zoo adapter at the recorded pin](https://github.com/diegoami/borg-zoo/blob/7b373a702e7b9bb491ba468d97d34cf99ec3df2d/zoo/borgrs.py)

The R4.5 exit slice is the exact 45 starts in `harness/suites/full`
(SHA-256 `7c81c28c03be3e6c188cb555fa8c917402bae0819d0791a1a810e031214d2eed`).
The gate is zero check mismatches and 45/45 switch runs matching the native
keys, messages, state and meaningful summary fields. Retain every attempted
cell and reconcile natural deaths in Cemetery.

## Work after the pin and R5 sizing

The read-only comparison to main `15a064f54490cd1b93c5ab6ef52091e175fce727`
finds ten later commits. The net file diff touches `borg_caution.rs`
(+164/−20 lines) and `params.rs` (+106/−106) in `borg-core/src`. The
later work stays outside this carry. [Pinned GitHub comparison](https://github.com/diegoami/borg-reborn-rs/compare/24be98732748230062882fe2b7d9601dba5e2cc0...15a064f54490cd1b93c5ab6ef52091e175fce727)

The nine `borg_flow*.rs` and `borg_think_dungeon*.rs` source files total
13,967 physical lines at the receiving pin, up from 11,736 at `2f74e1c5`.
Use roughly 14,000 lines as the initial R5 inventory to classify before
sizing owned code. Prioritize path/cache ownership, the 1,415-line stair
module and the dungeon think dispatch. Budget source review and a fresh
off/check/switch validation cycle for each runtime slice. The local R4.5
timings size smoke validation only. R5 effort can be estimated after the
ownership inventory and an owner-accepted R4.5 review.
