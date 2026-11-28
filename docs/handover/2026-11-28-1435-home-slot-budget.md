## 2026-11-28: The Home slot budget, 16 gear and 8 supplies (#1435)

### What was done

`town_home_slot_groups` (compiled default 0, shipped 0: it fixes no observed
defect, so the owner decides when it is on, RULES 28, PLAN P9) splits the
Home's 24 slots into `town_home_gear_slots` 16 and `town_home_supply_slots` 8.
The ranking and the guards are in the new `borg_town_home_budget.rs`; the
parameters and their reading are in `docs/CALIBRATION.md` ("The Home slot
budget").

- **Groups.** `group_of(Role)`: Gear, LightGear, SpareWeapon, Artifact are
  gear; LatePotion, Scarce, Device are supplies; Renewable, Common, Reject,
  Ordinary have no budget and are cleared as before. A sum above the Home's 24
  slots is refused at start (`params::check_relations`).
- **Supplies** score `town_home_rank_<capability>` (the owner's 14 weights)
  x coverage (100 percent, then `town_home_repeat_percent` 25, then 0; 0 when
  a spell legal at `town_home_spell_fail` or a permanent rPois renews it) x
  fill (copies against the target, at least 10 percent). Targets come from
  the `prepared_*` gate counts (Teleportation, Healing, *Healing*/Life, Speed,
  Restore Mana) and `town_home_reserve_target` for the rest.
- **Gear** is a greedy set cover over `borg_home_power.rs`'s property weights
  (`home_<property>_one_weight`, then two less one for a second piece, 31
  properties) plus `town_home_gear_power_percent` of the piece's power, ties
  by weight then slot. Unknown wearables, a staged loadout and this visit's
  moves are pinned inside.
- **Enforcement.** The ranking runs once per kit (`borg_town_shadow::kit_of`,
  through `borg_town_home_budget::apply`) and sets the role `Overflow` (new,
  rank 0) on every stack outside its budget. The planner (`borg_town_utility`)
  never stores an Overflow stack, clears one at Home (`prune_low_priority_home`:
  sold, or dropped when no shop buys it), and, with the groups on, replaces no
  stack inside a budget by one of the other group; the one-third goal is off.
  Plan records are unchanged in shape: the roles are already recorded, and
  `slot_groups` is bit 1 of the `only_overweight` slot of the `storing` row.
  The ranked gear step (`organize_gear`) releases Home gear the budget put
  outside, and does not stash a pack piece it put outside. A Home purchase of
  a supply is offered only if its capability would fit
  (`supply_fits`: a free supply slot, or a score above the weakest stack by
  the margin).
- **Loop guards** (`Memory`, in `LiveState`, kept across visits): (1) one
  decision per key per visit, standing while the state moves; (2) an incumbent
  scores `town_home_replace_margin_percent` 25 higher; (3) an evicted kind
  (gear: the variant) is out for `town_home_evict_cooldown_visits` 3 unless
  its score rises past the margin; (4) a stack this visit bought, stored or
  withdrew is pinned (`just_bought`, `home_moved`); (5) after
  `town_home_max_moves` 8 stores, withdrawals and evictions nothing new is
  placed; (6) `home_moves` already refuses the reverse of a move in the same
  visit.
- **Version 2** (off): `town_home_shift_depth_step` (0) shifts slots from gear
  to supplies by depth, up to `town_home_supply_slots_max`.
- Tests: 14 unit tests in `borg_town_home_budget` (groups, shipped budget,
  fill and coverage, margin, cooldown, one decision per visit, move budget,
  the fixed point and five alternating visits, borrowing, rPois cover, ties);
  a planner test in `borg_town_utility` (Overflow cleared at Home and not
  stored, no cross-group trade, the old trade with the groups off); native
  `reserve-budget` (`engine/tests/test_town_loadout.py`): over the Priest
  fixture's Home with no slot in either group, every gear and supply stack is
  Overflow, the roles are the same when asked again, evictions are recorded
  once.

### What was found

- The one-third goal (`home_supply_balance`) and `home_replacement` ranked by
  role (gear 3, scarce 2): with the groups on neither may trade across them,
  or a potion would replace zero-power gear inside the 16.
- A Home stack with no capability in the owner's list (Deep Descent, a
  stat-neutral Scarce item) is Overflow: eight slots, fourteen capabilities.
  That clears it at Home; a carried one stays carried.

### How it was checked, and not

- `cargo fmt --check -p borg-reborn-rs`, `cargo clippy --workspace
  --all-targets -- -D warnings`, `cargo test --release -p borg-core`,
  `cargo test --workspace --exclude borg-core`,
  `scripts/check-param-citations.py`, and the native `test_town_loadout`
  and `test_exploration_pack` on a `test-town-loadout` build.
- Off (the shipped value) the new code reads no parameter but the switch, so
  play is unchanged; smoke is compared at the end of the session (see the
  note of #1417).
- Not checked: the issue's native case of a Home with 20 gear pieces and 4
  supplies and an arriving Teleportation stack, and the native two-visit
  fixed point (the same Home and pack, zero Home moves the second time). The
  unit tests show the ranking is a fixed point and settles two near-equal
  candidates over five visits; the full flow through the shop was not run.
  Nothing here is measured (RULES 14): the zoo request (Home composition per
  visit from `BORG_TOWN_DUMP`, full-Home visits) has not been made.

### What is left, and owner decisions

- Not built: the section 5 cross-group exchange (a stack of
  `town_home_cross_min_copies` 10 taking a gear piece's slot); the borrowing
  rules (a borrowed slot is given back first, nothing bought into one); an
  allowance for unknown wearables (the owner's answer was none, so they are
  pinned inside and count against the 16).
- `town_home_reserve_target` is 5 copies for a capability no gate names: the
  issue says "one stack", which could be a copy or a full stack of 40. Say
  which.
- A renewable source is a spell legal at `town_home_spell_fail` (or a
  permanent rPois); a known activation does not zero a capability yet.
- The gear score uses the Home power `KitItem::power` (the gain as worn, for
  Home stock) where the issue names `GearChoice::power`; `town_home_gear_cover_percent`
  stays 100 until #1319 is decided.
- Two identical gear pieces share one decision (the key is the variant).

### Review of the design (Codex, GPT-6, issue comment 7245376597)

Fixed:
- Unknown wearables at Home count against the 16 whatever their role (pinned
  gear entries), and a supply top-up of a kind Home holds merges without a new
  slot (`supply_fits`). A gear purchase for Home needs a free gear slot
  (`gear_fits`).
- Over-cap stock converges: 20 gear pieces at a cap of 16 keep the best 16 and
  a pinned piece (a staged loadout, `retained_home_gear`) is never the one to
  go; a second pass moves nothing (`an_over_full_gear_group_converges_to_its_cap`).
  Evictions are decided in one pass, so the 20/4 case needs four evictions, not
  one per move budget. The ranked organizer's stash is gated by the decision;
  the staged loadout's deposit is not hooked, it is pinned and the others yield
  to it.

Not fixed (left, each worth an issue):
- Scoring a candidate as the first source after removing the only current one,
  and comparing Home before and after the proposed move.
- A failed command's retry guard beyond `buy_rejected`, a native refusal case,
  and a loop test beyond the three-visit cooldown.
- Planning the exact victim and the Home moves before buying into a full group.
  A purchase into a full group without a victim is not made today
  (`home_replacement` finds none), which is safe but never replaces.
- The DL100 targets are voluntary hoarding targets (the static entry point
  skips them); `Heal` still counts a Staff of Healing with the potions, and
  #1433's surplus CCW/CSW clause is not reconciled.
