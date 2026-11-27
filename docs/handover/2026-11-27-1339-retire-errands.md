## 2026-11-27: Retire the errands, /level-up and the plans (#1339)

Owner decision of 2026-11-27: the errands are out of scope, no measured run
uses them (every borg-zoo definition sets `borg_reborn_errand: none`), and
they complicate the source and risk breaking too much; `max-depth` plans and
`/level-up` go with them. This came out of an audit of how readiness
(`borg_prepared`) is routed between the C Borg, borg-core and the errand
shell: its errand-shell findings (#1326–#1329, #1333, PRs #1334–#1338) were
closed as superseded.

### What was done

- **Removed:** `borg-core/src/shell.rs` (every errand and `/stopscared`,
  `/explored`, `/stall`, the `^Z /` command), `borg-core/ladders.txt`,
  `borg-core/src/errands_plan.rs` (the `max-depth` plans, the plan cap in
  `borg_prepared` and its overlay on `borg_no_deeper`), `harness/plans/`,
  `tools/try-errands-plan.py`, and their hooks in `borg_think.rs`,
  `borg_think_dungeon.rs`, `borg_think_dungeon_util.rs`, `borg_caution.rs`,
  `borg_reincarnate.rs`, `borg_init.rs`, `borg_commands.rs`,
  `core_trace.rs` and `engine.rs` (`player_birth_key`, used only by the
  errand's owner check).
- **Kept:** the configuration readers moved to `borg-core/src/config_file.rs`
  (`OURS`), with the unloaded-world guard (#704) that the plans module held;
  `core-trace@2` keeps its `errand` field, always null.
- **Old configuration:** `check_retired_errand_settings`
  (`borg-reborn-rs/src/borg.rs`, borg-rs core only) ignores `none`/`off`
  values (a pborg.yaml line with one warning) and stops the run (exit 2) at
  any value that would select an errand, a fear stop or a plan, from
  `borg_reborn_errand`, `borg_reborn_stopscared`, `borg_plan` or
  `BORG_ERRAND`, `BORG_STOPSCARED`, `BORG_ERRANDS_PLAN`. `harness/run.sh`
  refuses `BORG_ERRANDS_PLAN` and no longer stages plans or records errand
  variables.
- **Tests:** `test_shopping.py`, `test_level_up.py` and
  `test_errands_plan.py` are gone. `test_phase_stock.py` lets the Borg shop
  as it always does (the run ends at its cap). `test_survival_incident.py`
  replays the wild-cat checkpoints without their `max-depth` plan; all four
  saves keep their outcomes. `test_core_profile_contract.py` and
  `test_borg_file_config.py` check the retired settings.
- **Docs:** `ERRANDS.md` and `GAME-PLAN-LANGUAGE.md` moved to
  `docs/archive/`; README, AGENTS, SPEC, ZOO-INTERFACE, borg-core.md and the
  other current pages say what went. `pborg.yaml` drops the errand lines and
  corrects the `borg_no_deeper` comment (N means DL N-1 at most, as in C;
  from #1338).

### Checks

CI's four cargo commands; `scripts/tests`, `tools/tests`,
`check-param-citations.py`; a native build; `engine/tests` (with the
wild-cat and phase-stock fixtures and the pinned zoo adapter); the smoke
suite against `smoke-borg-rs` (see the PR for the result); a manual run
showing the world guard still refuses an unloaded `borg_world`.

### What is left

Nothing in this repository. The audit's owner decisions stay open: #1331
(the kit descent gate and Recall from town) and #1332 (the caps in
`borg_power`'s depth bonus).

### Owner decisions

borg-zoo still carries plan arms and `borg_reborn_errand: none` in its
definitions and settings pages. Pinned specimens keep working, and a plan
arm on a specimen after this change is a `config_error`. Should the zoo's
pages be cleaned up as well (a borg-zoo change, not made here)?
