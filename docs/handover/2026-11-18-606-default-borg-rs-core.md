## 2026-11-18: the harness plays the borg-rs core by default (#606)

Owner: "the reviewers spent a lot of time on the wrong core, we should set the right one by default in the repository." `harness/borg.txt` names the borg-rs core (`borg_reborn_core = borg-rs`), the zoo runs it, and it alone reads `harness/borg.txt`. But `harness/run.sh` rewrote the run's `borg.txt` to the reborn core whenever `BORG_CORE` was unset (#479), so every plain smoke or full suite, and CI's smoke check, played a core that never reads the settings under review (#591, #592 and #600 lost time to it).

- `harness/run.sh`: with no `BORG_CORE` a run plays the core `harness/borg.txt` names (borg-rs). `BORG_CORE=reborn` selects the reborn core. The errands-plan check and `BORG_CORE_OVERRIDES=off` treat "unset" as borg-rs.
- `harness/reference/smoke-borg-rs`: new, the default core's smoke reference (no deaths; 3/3 identical on two runs).
- CI: the reborn-core steps (its smoke twice and reference, check-extra-load, the observation logs, the hallucinate and full test builds, smoke-exact, the rerun-death source) say `BORG_CORE=reborn` and keep their references (`smoke-rust`, `smoke-exact-rust`, `full-rust`), so no reference is re-recorded and no new death appears. A new smoke check on the default core runs twice against `smoke-borg-rs`.
- Tests: `test_build` pins the reborn core (it checks the reborn core's game options); `test_errands_plan` refuses a plan with `BORG_CORE=reborn`.
- Docs: AGENTS.md ("Which core a run plays"), `harness/README.md`.

Not done: the full suite on the borg-rs core has 3 deaths (two Druids, a Ranger); a `full-borg-rs` reference would need their Cemetery packages, so it is left for an issue if wanted.
