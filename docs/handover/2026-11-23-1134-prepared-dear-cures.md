## 2026-11-23: dear cures with gold to spare (#1134)

**Owner request, 2026-11-23:** "make the town planner start buying CCW and
cure poison in the same way that they buy detect invisible or other when they
have money to spare."

**Why the planner did not.** Neutralize Poison and Cure Critical Wounds
already sit on the same "prepared" rows as Detect Invisible
(`town_prepared_percent`, #863). But a prepared row prices a *copy*, not a gold
piece. At the shared 8 percent, a 15 gold Detect Invisible beats a 75 gold
Neutralize Poison and a 200 gold Cure Critical Wounds. In the 25 deaths of zoo
run 45121432449 the planner bought Neutralize Poison once and Cure Critical
Wounds never; median gold on returning to town was 7.

**Done here (all compiled and shipped 0 = today's play):**
- `town_prepared_poison_percent`: the poison remedy row's own percent (0 = the
  shared one). 40 = five times 8, the same value per gold as Detect Invisible.
- `town_prepared_ccw` and `town_prepared_ccw_percent`:
  - a new prepared row, "prepared: Cure Critical Wounds", fed one copy per CCW
    potion, worth that percent per copy for the first `town_prepared_ccw`
    copies (0 = off);
  - 112 = fourteen times 8 (200/15 = 13.3);
  - it is the model's last row, held only while it is on, so the model keeps
    its shape with it off.
- A unit test (`a_remedy_of_its_own_percent_buys_dear_cures_with_spare_gold`).
- The three lines in `harness/borg.txt`, at 0.

**Checked.**
- CI's commands:
  - `cargo fmt --check -p borg-reborn-rs`;
  - `cargo clippy --workspace --all-targets -- -D warnings`;
  - `cargo test --workspace --exclude borg-core`;
  - `cargo test --release -p borg-core` (831 passed).
- The same patch applied to 1c071812, with the settings at 0: 43 games (the 25
  deaths and an 18-game survivor sample) reproduce the zoo cells'
  `play_sha256` exactly.

**Replays with the settings on** (`town_prepared_poison_percent 40`,
`town_prepared_ccw 2`, `town_prepared_ccw_percent 112`; 1c071812 + patch;
borg-cemetery#146 has the method):
- **Deaths:** 2 of 25 death games change, both long before the death. The
  deaths are too poor to buy.
- **Survivors:** 12 of 18 survivor games change, none dies.
- **Purchases:**
  - Neutralize Poison alone at 40: Neutralize Poison bought 2 → 8 potions;
  - with the CCW row: CCW 0 → 1.
- **Town dump** (`BORG_TOWN_DUMP`, keys identical):
  - past DL2 (`town_variety_min_depth = 2`, below which no prepared row
    exists) the Borg had ≥ 200 gold on 25 town decisions;
  - CCW was on the Alchemist's shelf on all 25;
  - affordable at its marked-up price on 5, and carried already on those 5.
  - The rule works; gold is the limit.

**Left.**
- The owner decides whether to ship non-zero values; RULES 14 asks for the zoo
  first (proposed as an arm of borg-zoo#739's pilot).
- Shipping changes the play and needs `smoke-borg-rs` re-recorded in that PR.
- `town_variety_min_depth 1` (prepared rows from DL1) was being replayed when
  this note was written; see the PR.
