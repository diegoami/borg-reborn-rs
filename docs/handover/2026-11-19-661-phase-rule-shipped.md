## 2026-11-19: the owner's Phase Door rule shipped (#661)

Owner: make a PR with the shipped-config changes the zoo evidence supports.

- **Shipped** (`harness/borg.txt`): `escape_phase_adjacent_rule = 1` with
  `escape_phase_lost_test = 0`, which uses the danger threshold at
  `escape_phase_lost_danger_percent = 100`. This is borg-zoo#347's thr-100
  arm. Compiled defaults are unchanged. CALIBRATION.md and PHASE-DOOR.md
  state the shipped values.
- **Evidence** (borg-zoo#347, `4acf071f`, 1,728 paired starts): 77 deaths
  against 93 with the rule off (55 rescued, 39 newly killed, p = 0.12).
  Mean max CL and DL are unchanged, and Druids drop from 19 to 12. The big
  tier alone gave p = 0.011 but the full tier was flat (47 against 46).
- **Confirmed** (borg-zoo#347, `reborn-phase-rule-main-confirm`, fresh
  starts on classes-1800-b, 3,600 games on main `3ba17e2c`): deaths 128 to
  89 (79 rescued, 40 newly killed, exact McNemar p = 0.00045); mean max CL
  +0.04 and DL -0.01, both non-inferior at the prespecified margins. Only
  Blackguards do worse (7 to 11); Rogues 8 to 0, Paladins 11 to 5, Priests
  24 to 15.
- **smoke-borg-rs re-recorded** with `scripts/rerecord-smoke-borg-rs.sh`:
  2/3 games identical to the old reference. smoke-mage-2 diverges at key 744:
  the rule refuses a Phase Door with no adjacent enemy (a shrieker and an icky
  thing in view) and the Borg attacks at range instead.
- **Checked:** `cargo fmt --check -p borg-reborn-rs`, clippy with
  `-D warnings`, `cargo test --workspace --exclude borg-core`,
  `cargo test --release -p borg-core` (447 passed), `scripts/tests`,
  `tools/tests` and `check-param-citations.py`. `engine/tests` ran
  locally: 92 tests, 18 skipped for vendor fixtures that CI provides.
- **Local note:** the smoke suite needs the cemetery's death tools. Without
  `BORG_CEMETERY_DIR` or a checkout beside the repository, every smoke game is
  reported FAILED even though it played.

**Left:**
- Blackguards: 7 to 11 deaths with the rule, the one class worse; worth a
  look at their Phase Door sites.
- **Errands plan by default** (borg-zoo#383: 105 against 91 deaths, p =
  0.004, on main `3ba17e2c`) is not shipped here. `harness/run.sh` refuses
  a `borg.txt` that names a plan. The zoo adapter ships a plan only when a
  profile names it, so every other zoo cell would end as `config_error`. The
  adapter change is asked in borg-zoo#398. Shipping it then needs a run.sh
  change and a way to turn the plan off (`borg_plan = none` in the core),
  after the #383 confirmation on fresh starts.

**Taken over** (owner, "take over 662"): main merged after #648 and #653,
the docs updated to the confirmed result, smoke-borg-rs re-recorded on the
merge.
