## 2026-11-19: caution_scary_unlatch_calmed shipped (#671)

Owner: make PRs for the settings the zoo found safe to adopt.

- **Shipped** (`harness/borg.txt`): `caution_scary_unlatch_calmed = 1`.
  With `caution_scary_stand = 1`, a level stops being scary once its scary
  monster is known hurt or afraid, or is killed (#593). The compiled
  default is unchanged, and CALIBRATION.md states the shipped value.
- **Evidence** (borg-zoo#400, all 1,800 classes-1800 starts on `3ba17e2c`):
  - deaths: 93 against 105 (28 rescued, 16 newly killed, p = 0.096; 0.38
    after Holm over the study's four arms), fewer in every tier;
  - mean max CL +0.09 and DL +0.05, both significant.

  Priests are the only class that does worse (13 → 15).
- **smoke-borg-rs re-recorded** (`scripts/rerecord-smoke-borg-rs.sh`, main
  `592c6984` plus this change): 2/3 games identical to the old reference.
  smoke-priest-3 diverges at key 281, turn 1882.
- **Checked:** `cargo fmt --check -p borg-reborn-rs`, clippy with
  `-D warnings`, `cargo test --workspace --exclude borg-core`,
  `cargo test --release -p borg-core` (453 passed),
  `check-param-citations.py` and `scripts/tests`. `engine/tests` was not run
  locally.

**Left:**
- A confirmation on fresh starts (classes-1800-b) is filed in borg-zoo.
- A Priest exception (`caution_scary_unlatch_calmed@priest = 0`) is a
  candidate if the confirmation keeps Priests worse.

**For the owner:** this ships on a significant progress gain and a
non-significant lean towards fewer deaths. It is not a confirmed survival
result. It interacts with #662 (the Phase Door rule), whose confirmation ran
with this flag off. Both PRs re-record smoke-borg-rs, so whichever merges
second needs a re-record after merging main.
