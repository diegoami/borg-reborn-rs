## 2026-11-18: the harness plays the borg-rs core by default (#606)

Owner: "the reviewers spent a lot of time on the wrong core, we should set the right one by default in the repository." `harness/borg.txt` names the borg-rs core (`borg_reborn_core = borg-rs`), the zoo runs it, and it alone reads `harness/borg.txt`. But `harness/run.sh` rewrote the run's `borg.txt` to the reborn core whenever `BORG_CORE` was unset (#479), so every plain smoke or full suite, and CI's smoke check, played a core that never reads the settings under review (#591, #592 and #600 lost time to it).

- `harness/run.sh`: with no `BORG_CORE` a run plays the core `harness/borg.txt` names (borg-rs). `BORG_CORE=reborn` selects the reborn core. The errands-plan check and `BORG_CORE_OVERRIDES=off` treat "unset" as borg-rs.
- `harness/reference/smoke-borg-rs`: new, the default core's smoke reference (no deaths; 3/3 identical on two runs).
- CI: the reborn-core steps (its smoke twice and reference, check-extra-load, the observation logs, the hallucinate and full test builds, smoke-exact, the rerun-death source) say `BORG_CORE=reborn` and keep their references (`smoke-rust`, `smoke-exact-rust`, `full-rust`), so no reference is re-recorded and no new death appears. A new smoke check on the default core runs twice against `smoke-borg-rs`.
- Tests: `test_build` pins the reborn core (it checks the reborn core's game options); `test_errands_plan` refuses a plan with `BORG_CORE=reborn`.
- Docs: AGENTS.md ("Which core a run plays"), `harness/README.md`.

After the review of #607:
- a `BORG_CORE=reborn` run's `borg.txt` says `borg_reborn_core = reborn` again, so the file and `borg_txt_sha256` describe the core that played;
- `config.txt` always records the core that played (`borg_core=`; with no `BORG_CORE` it is read from the run's `borg.txt`), so a death record and its rerun know the core;
- `test_character_build` pins the reborn core (its capture expects the reborn core's character), and `test_errands_plan` checks that a plan plays on the default core;
- CI's dossier check accepts either core's pair (`trace-tail.jsonl` and `params.json`, or `trace-core-tail.jsonl` and `params-core.json`). A borg-rs death needs a Cemetery package like any other: `reconcile-deaths.py`'s registry is the Cemetery's;
- live docs say the new default: `harness/borg.txt`'s comment, `docs/ERRANDS.md`, `docs/CALIBRATION.md`, `docs/ENGINE.md` (References), and the CI scope comment. **`docs/HANDOVER.md` (the archive, 2026-11-17, #577) says no CI step plays the shipped `harness/borg.txt` on the borg-rs core, and that `harness/run.sh` rewrites the core to reborn: both are no longer true.** The default core's smoke against `smoke-borg-rs` now runs on every PR, and covers the borg-rs arm when the full-suite scope rule skips the reborn full suite.

After the re-review of #607:
- `BORG_CORE_OVERRIDES=off` with no `BORG_CORE` now sets `BORG_CORE=borg-rs`: `borg-core/borg.txt` names no core, so the game fell back to the reborn core and the record said `borg_core=` (empty, unreplayable). The record is never empty (no core named means the compiled fallback, reborn);
- `harness/reference/smoke-borg-rs` is re-recorded at the merged head: the fix commit had edited `harness/borg.txt`'s comment, so the reference's recorded `run_config` hashes no longer matched (the play did not change). The reborn references' stale `run_config` hashes predate this PR (as on main) and are left as they are.
- `smoke-borg-rs` re-recorded again after merging main's #602 (the Phase Door decision), which changes the borg-rs core's smoke play: CI compares the PR merged with main, and it failed against the older reference. AGENTS.md now says a PR that changes the borg-rs core's play re-records `smoke-borg-rs`.

Left:
- replaying a record with no `borg_core` made before #606: diegoami/borg-cemetery#71;
- a `full-borg-rs` reference: the full suite on the borg-rs core has 3 deaths (two Druids, a Ranger), #609;
- the reborn references' `REFERENCE` `run_config` hashes predate later `harness/borg.txt` edits (as on main); `harness/diff` compares content, not that provenance.
