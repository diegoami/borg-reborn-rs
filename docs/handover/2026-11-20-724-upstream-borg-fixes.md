## 2026-11-20: the upstream C Borg fixes carried into borg-core (#724, part 2 and 3)

Owner, 2026-11-20: "Add an issue to update engine and carry fixes from
upstream Borg in Angband in borg-rs". The engine update (part 1) is its own
PR (RULES 21).

**What was done:**
- **`007bea95`** (keep the largest slay multiplier across equipment slots):
  `borg_trait.rs` now keeps the maximum, as upstream. borg-rs used `|=`,
  which gives 7 for slays of 3 and 5. Every read of a `BI_WS_*` trait is on
  or off (`borg_fight_attack.rs:479-488`, `borg_power.rs:355-402`,
  `borg_fight_defend.rs`, `borg_fight_perm.rs`, `borg_flow_stairs.rs`,
  `borg_log.rs`), so play cannot change.
- **`95387b06`** (check the teleport landing with either kind of teleport):
  `escape_teleport_landing_either`, compiled and shipped 0.
  `lacks_teleport` is a pure function with a unit test.
- **`b5747369`** was already carried (borg-rs patch 0092).
- **The ledger:** `engine/upstream-borg.txt`, named by `borg_ledger` on the
  angband pin. `scripts/refresh-sources.py` now reports every upstream
  `src/borg` commit after its `since` that it does not list. Test:
  `test_every_upstream_borg_commit_is_in_the_ledger`.

**Checked:**
- CI's commands; `check-param-citations.py` (2202); `borg-core-vendor.py
  check`; `scripts/tests`.
- classes-18 through `harness/run.sh … 100000 exact,menu` (Human seeds 1-9,
  Half-Elf 10-18), base main `653f60ba`:
  - switch off: 18/18 identical to main;
  - switch on: 18/18 identical to off. No low-level game reaches the
    Destruction or Teleport Level defences that ask for the landing check.

**Left / for the owner:**
- Whether to ship `escape_teleport_landing_either = 1` (upstream's
  behaviour). It needs deeper games to act at all: a zoo run at 200,000
  turns or on deeper starts.
- A note in borg-rs, so its own translation gets the same two fixes.
