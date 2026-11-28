## 2026-11-28: The Home gate reserve (#1417)

Stacked on #1435 (the supply group's 8 slots) and #1416 (the weapon cap that
frees slots), in that order, as the owner asked.

### What was done

`town_home_gate_reserve` (compiled default 0, shipped 0: it fixes no observed
defect, so the owner decides when it is on, RULES 28, PLAN P9),
`town_home_gate_min_depth` 5 (owner: hoarding starts at max depth 5) and
`town_home_gate_lookahead` 1. The routine is in the new
`borg_town_gate_reserve.rs`; the reading is in `docs/CALIBRATION.md` ("The
Home gate reserve").

- **Kinds**: the owner's list, as the supply capabilities of #1435 without
  the spare book: Teleportation (Staff of Teleportation charges counted),
  Teleport Level, Mass Banishment, Rune of Protection; Healing, *Healing*,
  Life, Speed, Restore Life Levels, Restore Mana (casters), Resist Poison; the
  stat-restoring mushrooms and potions; the six device kinds, one Home stack
  each. Not Clear Mind/Emergency/Terror, not CCW/CSW (#1433).
- **Targets** come from the gate parameters: `reserve_target_at` takes the count
  of the `town_home_gate_lookahead`-th gate past max depth (the deepest
  gate's when fewer lie ahead), at least the gate passed; a kind no gate
  names uses `town_home_reserve_target`.
- **Deposit.** `KitItem::gate_reserve` (new; `Some(keep)` on a carried
  reserve stack, bits 16-23 of the record's kit flags, so no schema change)
  is the copies the pack keeps: what the coming gate asks (descent target =
  max depth + 1) and every copy Home does not still want. The planner takes it
  as one more floor, lifts the one-Home-stack-per-kind rule (`spare_is_unique`)
  and the overweight-only/`loss <= cost` conditions for the copies past it,
  and stores them into the stack Home holds without a free slot.
- **Buy.** `home_value` offers a Home purchase of a kind below its target
  although the kit holds the kind (the old rule returned 0 once the kit had
  it). It takes the existing reserve purchase path: after the trip kit, within
  the gold reserve and `borg_black_market_max_percentage`, one copy a visit
  (`reserve_bought`).
- **Withdraw.** `withdraw` (called from `choose_plan` after the ranked gear
  step, outside urgent shopping) takes from Home what the pack lacks for the
  gate the descent target reaches, one copy a step, before Recall. The gates
  keep counting the pack only (owner). Loop guards: the reverse of a confirmed
  move is barred for the visit (`home_moves`), and a stack taken out is kept
  carried (`keep` returns the whole stack).
- Tests: `the_target_follows_the_next_gate`, `what_the_gates_ask_at_a_depth`,
  `the_pack_keeps_the_need_and_the_copies_home_no_longer_wants` and
  `the_withdrawal_brings_the_pack_up_to_the_gate` (the issue's case: Home 5,
  pack 3, the gate asks 8, the pack leaves with 8) in `borg_town_gate_reserve`;
  `a_reserve_kind_below_its_target_is_stored_into_the_stack_home_holds` in
  `borg_town_utility`: a full Home holding the kind takes two of three carried
  copies, none at the pack's keep or without the reserve.

### What was found

- `home_value` and `spare_is_unique` were the two places that kept a Home
  reserve at one stack per kind, as the issue said; the planner's floors
  (`reserve_keep`) do not know the gates, hence the per-stack `gate_reserve`
  keep rather than the model's.
- A reserve spread over several carried stacks of one capability (scrolls and
  a staff) is approximated: each stack keeps the capability's keep, a device
  is stored whole or not at all.

### How it was checked, and not

- `cargo fmt --check -p borg-reborn-rs`, `cargo clippy --workspace
  --all-targets -- -D warnings`, `cargo test --release -p borg-core`,
  `cargo test --workspace --exclude borg-core`, `scripts/check-param-citations.py`,
  and the native `test_town_loadout` and `test_exploration_pack` on a
  `test-town-loadout` build.
- Off, the shipped value, the new code is read only through
  `town_home_gate_reserve` (`keep`, `wants_more` and `withdraw` return at
  once). Smoke at the shipped settings of all three tasks (#1416 on at 2,
  #1435 and this one off) equals `smoke-borg-rs`: 4/4 runs identical, at
  `af0c0162c29f`, so no re-record. Those games stay on DL1-2 and hold no
  spare weapons or Home stock, so this checks only that nothing else moved.
- Not checked: the issue's native town cases (a Home with 5 Teleportation and
  3 carried, the Recall depth at DL26's gate, the Borg leaving with 8; and a
  case where the gate is not reached and Home stock is untouched). The unit
  tests cover the arithmetic and the planner; the walk to the Home, the
  `TAKE_HOME` action in a shop and the Black Market top-up were not run in a
  game, and nothing is measured (RULES 14). The zoo request (alive at the cap
  and the first failed gate per trip, with the Home's gate stock from
  `BORG_TOWN_DUMP`) has not been made.

### What is left, and owner decisions

- One top-up copy a visit (the existing `reserve_bought` limit) fills the
  target slowly; a quantity for the purchase needs a change to the reserve
  purchase.
- `town_home_reserve_target` (5 copies) is a stand-in for "one stack" for
  the kinds no gate names (see #1435).
- Withdraw takes one copy per step and re-solves; a stack of eight is eight
  town steps.
- The Alchemist's Restore Life Levels top-up works through the same offer as
  every other ware, only if `restore_life_keep`'s model does not already cover
  it.
- The deposit leaves the Teleportation scrolls of a Borg below DL10 in the
  pack (the first gate asks nothing before DL10); say if the reserve should
  hold the DL10 count from DL5 (`town_home_gate_lookahead` 1 already sets the
  target to 3, but the pack's keep is 0 until the descent reaches the gate).
