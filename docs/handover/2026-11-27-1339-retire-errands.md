## 2026-11-27: Retire the errands, /level-up and the plans (#1339)

Owner decision of 2026-11-27: the errands are out of scope, no current zoo
arm uses them, and they complicate the source and risk breaking too much.
(Earlier experiments did use them: the option-C pilot's ladder arm,
`reborn-pilot-c-classes-1800`, and the plan arms of the borg-zoo#151
pilots, `reference/zoo/catalogue/profiles.json`. Those ran on their own
pinned specimens. The recent definitions set `borg_reborn_errand: none`
and no `borg_plan`.) `max-depth` plans and
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
  `BORG_ERRAND`, `BORG_STOPSCARED`, `BORG_ERRANDS_PLAN`. `borg_plan: none`
  is a new inert value (the old loader would have tried to open a file
  named `none`). The check runs for the borg-rs core only, as the old
  readers did; a reborn-core run ignores a `borg_plan` line, where the old
  `core_is_borg_rs()` exited 2 for it (plans only ever ran on borg-rs).
  `harness/run.sh` refuses `BORG_ERRANDS_PLAN` for every core. `harness/run.sh`
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
suite against `smoke-borg-rs` (4/4 identical); a manual run showing the
world guard still refuses an unloaded `borg_world`. Review on the PR
(DeepSeek V4.1 Flash, OpenCode): code and default play confirmed; its
documentation findings are fixed in the same PR, except the archived pages,
which stay unchanged as `docs/archive/README.md` requires.

### What is left

Nothing in this repository. The audit's owner decisions stay open: #1331
(the kit descent gate and Recall from town) and #1332 (the caps in
`borg_power`'s depth bonus).

### Owner decisions

The cemetery-owned `rerun-death.py` (borg-cemetery
`scripts/native/rerun-death.py`) still restores `borg_errand`,
`borg_stopscared`, `borg_errand_budget`, `borg_ladders` and
`borg_errands_plan` from a dossier. A rerun of an errand or plan death on a
build after #1339 therefore stops at start (exit 2) rather than replaying
(loudly, not silently). The review of #1340 proposes that the tool refuse
such a dossier with a message naming #1339 and point to replaying at the
dossier's own build. That is a borg-cemetery change: this session was not
permitted to open the issue there, so it is described here for the owner to
route (RULES 7). `tools/tests/test_rerun_death.py` says what it pins until
then.

borg-zoo still carries plan arms and `borg_reborn_errand: none` in its
definitions and settings pages. Pinned specimens keep working, and a plan
arm on a specimen after this change is a `config_error`. Should the zoo's
pages be cleaned up as well (a borg-zoo change, not made here)?
