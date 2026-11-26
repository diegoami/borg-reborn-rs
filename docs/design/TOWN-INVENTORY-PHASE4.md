# Phase 4 readiness: the Stay rules as the plan implements them, and the plan record (P13)

> **Utility revision, 2026-11-14 (owner direction):** fixed stock targets and
> the five-level default ambition are superseded for current utility records by
> [the trip-relative utility model](TOWN-UTILITY.md). Supply value scales with
> journey duration, destination and character capability; retaining gold and
> storing supplies for later trips compete with purchasing/carrying. v4 replay
> preserves the historical model. Subsequent owner direction adds live
> execution with `borg_town_plan=1`; see [the combined runtime](JOINT-POLICY-TRIALS.md).

Status: **accepted** (owner, 2026-11-13): Part A as drafted, both literal
changes intended and the Home a floor source; #37's four open questions as
recommended; Part B with "displaced" = the runner-up and the stack class
appended to the ware tuple; Part C's abort kept. Drafted the same day. Epic borg-rs#36; the
phase is `docs/TOWN-INVENTORY-PLAN.md`, "Phase 4 - the plan". Where this
document and the epic's decision log disagree, the log wins and this
document is wrong.

> **Amended 2026-11-14 (owner): there are no floors.** Every object can
> become redundant - there are other ways to eat (Remove Hunger scrolls,
> spells), to see, and to cure - so none is a hard constraint. What replaces
> the floors is the **urgent need**: needed **now** with **no other way** to
> meet it (hungry or weak with nothing to eat; no light source at all; cut
> with no wound cure; poisoned with no poison cure), bought before anything
> else while it is urgent, skipped if unaffordable, never blocking the visit.
> Poison wears off, so the standing poison-cure capability is gone; Cure
> Critical Wounds and a curing spell count as cures. No early-game term:
> the high early value of torches, food and potions is left to emerge from
> the deficits. Where the text below says "floor", read "urgent need"; the
> mechanism (the exact cheapest-cover pass, phase 2) is the same.

Phase 4's entry criteria, and where each stands:

| criterion | state |
|---|---|
| #37's Stay rules | **accepted 2026-11-13** as Part A below: the table as phase 4 implements it, reconciled with the decisions taken after it (B3, P1). |
| the floors' infeasibility outcome | **decided** (B3, 2026-11-12): it falls back to #37's Stay rules, accepted with Part A. |
| the record schema (P13) | **fixed 2026-11-13** as version 4 (Part B) on top of #46's frozen version 2. |
| the sell-side suite, the shadow-plan mode | land *in* phase 4 (not entry criteria) |
| (from the phase-3 re-review) the solver's refusal and reading `optimal` | to decide *in* phase 4; Part C |

Parts A and B are accepted: **phase 4's entry criteria are met.**

## Part A - the Stay rules, as the plan implements them

#37's table rules every old town rule Stay, Go or Drop. Its Go rows become
usefulness terms and its Drop rows disappear; this part is about the
**Stay** rows only: where each lands in the plan, and where the decisions
taken after #37 change it.

### A.1 The emergency path

| #37 row (Stay) | in the plan | change from today |
|---|---|---|
| Cut or poisoned → the Alchemy shop (`borg_think_store.rs:153-157`) | the **Bleeding** and **Poison** floors, bought by the budget's floor pass (phase 2) before anything else | the floors are **kit predicates** (B3): "a cure for the current condition" and "a poison-cure capability". The capability is wanted even when **not** poisoned - broader than today, which acts only on the condition. The shop is not a rule any more: the floor pass takes the cheapest source that stocks the cure, which today is the Alchemist. No Use action (B3): the plan buys; quaffing stays the dungeon code's |
| No food, or no light at CL ≥ 2 → the General store (`:158-163`) | the **Food** and **Light** floors (≥ 1 each) | **the CL ≥ 2 condition goes**: P1 says at least one light, unconditionally. A CL1 character starts with a torch, so this should not bite; it is listed because it is a literal change |
| Instant shopping - the emergency jumps the step order (`:169-185`) | **floors first**: the plan cycle's `first(floors, chosen)`, ahead of any sale, store or purchase; and the floors **protect the kit**: a disposition (sell, store, drop) that would leave a floor unmet is not allowed, because the predicates are computed on the planned kit | the clause "light == 1 and gold ≥ 100 and CL < 10" is the lantern rule, which #37 rules **Go** (a term), so it leaves the emergency path |
| If the town has no healing left, try the house (`:186-194`) | **the Home is a floor source**: its wares join the floor pass at price 0 (the action is Take) | **phase 2's floor pass looks at the shops only** - this is a gap phase 4 must close, with a test |
| Give up after 2000 turns in town, or 1350 anti-bounces (`:141-145`) | **unchanged**: the plan sits behind the same guards in `borg_choose_shop` | the numbers are #37's open question 3 (A.4) |

### A.2 The buy decision

| #37 row (Stay) | in the plan |
|---|---|
| One empty slot, per **stack** (`borg_store_buy.rs:466-469`) | the solver's pack-slot limit (phase 3), counted per distinct stack (phase 2's S3 contract) |
| Two empty slots to wield (`:565-566`) | a wearable option's slot cost: the new item goes on, the old one into the pack, so one net pack slot unless the old item stacks. Phase 3 charges the ware's `slots_cost`; phase 4 makes it "the replaced item's slot", with a test (the reviewers' modelling note) |
| Only certain stores when in bad shape (`:490-497`) | **implemented by the order, not by a filter**: the floors are bought first, and the re-solve after them is no longer "in bad shape". Keeping the filter would block buying elsewhere once the floors are met |
| Cures for poison and cuts come from the Alchemy shop only (`:498-502`) | a fact about the stock, which the stock model sees; not a rule in the plan |
| Require sufficient cash (`:523-529`) | the solver's gold limit (phase 3) |
| The immediate-shopping special case (`:531-538`) | as the emergency path (A.1) |

### A.3 The veto, the sell side and the Home

| #37 row (Stay) | in the plan |
|---|---|
| Don't buy what you just sold to that shop (`borg_store_buy.rs:321-329`) | a **candidate filter** before `groups()`: a ware of the kind sold to that shop this visit is not a candidate (the Borg's `sold_item_*` lists already hold it) |
| Never sell worthless wares (`borg_store_sell.rs:853-859`) | a worthless item is never a **Sell** candidate; it may be a **Drop** |
| The shop has to buy that kind (`:862-865`) | a Sell candidate is offered only to shops that buy its kind |
| Never sell unidentified artifacts or random-power egos (`:928-941`) | excluded from **Sell** (they may still be kept, stored or dropped) |
| Don't sell what you just bought (`:944-953`) | excluded from **Sell** this visit (`bought_item_*`) |
| The Home is a stash, not a bank (`borg_store.rs:76-82`) | the design's premise: **Store** is free and frees a pack slot, bounded by the Home's slots (phase 2's budget); gold is never parked |

### A.4 #37's four open questions

| question | recommendation |
|---|---|
| 1. Where is the line between a floor and a need? | **Already decided** by P1 and B1 (2026-11-12): the floor is the kit predicate's minimum (one food, one light, one cure, the capability); everything above it is the target, priced by the usefulness function. Record it as decided |
| 2. The money-scum mode: dropped outright? | **Not by phase 4.** With `borg_town_plan` off nothing changes. With it on, the scum loop is not run and the plan's record says so - the mode's fate is decided when the default moves (P14), not now |
| 3. The guardrails (2000 turns, 1350 anti-bounces) | **Keep the numbers for phase 4** and make every visit that hits one an observable in the record (Part B, `guard`). Size them from the shadow-mode runs, not by argument |
| 4. Does "buy for the home" survive? | **Drop for phase 4**: the Home is the pack's overflow only. A depot for the next trip is a trip plan of its own (P11), and phase 4 is not the place to add one |

## Part B - the plan record (P13), version 4

P13 asks for a record that is "versioned, machine-readable, integers only,
stable field order, total ordering", that shows **what each action was for
and what it displaced**, and that a plan can be **replayed from its own
logged inputs**. Version 2 (frozen, #46) carries the state and today's
decision; version 3 (#47) is the handover's state record. Version 4 is the
plan's record: **version 2's fields, unchanged and in the same order, then
the plan's**. A v2 reader that stops at the v2 fields reads a v4 line.

One record per **plan step**: one solve and the action taken from it. In
shadow mode (the plan computed, not executed) the record is written next to
today's v2 decision record at the same state, and the two pair on `turn`.

### B.1 What v4 adds

| field | holds | why |
|---|---|---|
| `mode` | 0 shadow, 1 live | the two sides of a shadow run are told apart by it |
| `visit`, `step` | the town visit's number in the run, the step's number in the visit | a plan is a sequence; this is the sequence |
| `gturn` | the **game's** turn (v2's `turn` is the Borg's clock) | the clock the message log is on, as in v3 |
| `power` | `borg.power` | the power need reads it and it is not a trait; without it no record replays the power deficit |
| `params` | the twelve town settings, in their `NEW_SETTINGS` order | the arm: the weights, `k`, the trip model's two settings, `borg_town_plan` |
| `limits` | `[gold, pack_slots, home_slots, weight]` after the floors, and the raw `[pack_capacity, home_capacity]` | the solver's inputs that the budget computed |
| `floors` | `[[shop, slot, needs], ...]`, `needs` a bitmask in `Need` order; and `unmet`, a bitmask | what the floor pass bought, and what it could not |
| `tryon` | `[[shop, slot, value], ...]` for every wearable option | the only value the record cannot recompute: the try-on runs the engine's power code |
| `plan` | `[[key_kind, key, option, copies, cost, value, slots], ...]`, ranked | the chosen set, in the order the plan executes it |
| `solve` | `[value, bound, lambda, flags]`, flags a bitmask: optimal, gold_exact, slots_met, weight_met, weight_repaired | how good the set is known to be (phase 3's flags) |
| `action` | v2's codes, extended: 0 visit, 1 buy, 2 take_home, 3 sell, 4 sell_home, **5 store, 6 drop, 7 leave** | the action this step took |
| `act` | `[shop, slot_or_item, copies, price]` | its target |
| `for` | the need it served (0..7 in `Need` order), and `floor` 0/1 | **what it was for** |
| `displaced` | `[key_kind, key, option, value, cost]` of the best option the plan did **not** take, or `[-1]` | **what it displaced** - see B.2 |
| `guard` | 0 none, 1 the 2000-turn give-up, 2 the anti-bounce give-up, 3 the solver refused | A.4 question 3, and Part C |

`-1` for an absent value, as in v2. Integers only; field order as listed;
`seq` totally orders the file, as in v2.

### B.2 "What it displaced": two readings

P13 does not say which, and they record different things:

- **the runner-up** (proposed): the best option the solve left out for lack of gold, slots or weight. It is always defined, and it is computable from the record (replay the solve without the limit that bound). It answers "what did this purchase cost us".
- **today's choice**: the action the current policy would have taken at the same state. It answers "what did the plan change", but it is only known in shadow mode, where the v2 record next to it already carries it.

So v4 records the runner-up, and shadow mode gets today's side from the
paired v2 record rather than from a second copy inside v4.

### B.3 Replay

A v4 record replays when the stock, the budget, the groups and the solve
can be recomputed from it and give the same `plan` and `solve`:

- the stock and the budget come from v2's `pack`, `equip`, `home`, `shops`
  and `traits`, plus v4's `limits`;
- the usefulness comes from `traits`, `power` and `params`;
- the wearables' values come from `tryon`;
- the stacking predicate is not in the record: replay uses the record's
  stack classes, which v4 therefore adds to each ware of `shops` as a
  trailing element (a v2 reader ignores it). **This one is a change to
  v2's ware tuple**, which is why it is called out.

Phase 4's suite then gets its strongest test for free: **replay every
record a run writes and require the same plan**.

### B.4 Corrected 2026-11-13, in the implementation (slice 1, `borg_town_record.rs`)

Building the record showed three things B.1-B.3 got wrong. The decisions
stand (version 4, v2's fields first and unchanged, "displaced" = the
runner-up); the field list is corrected:

- **`tryon` is not the only value a replay cannot recompute.** v2's ware
  tuple carries the *tval* in the position the table calls "kind", and no
  weight; and which need a ware serves, by how much, which floors it
  discharges, its stack class and whether it joins a carried stack are all
  decided by the engine's item tables or the stacking predicate. So v4
  carries a **`wares`** field: `[shop, slot, kind, class, slots_cost,
  weight, need, reduction, serves]` per ware, `need` an index into the
  needs (-1 none) and `serves` a floor bitmask.
- **v2's ware tuple is therefore not changed at all.** The stack class
  travels in `wares` instead of being appended (B.3's called-out change is
  withdrawn); a v2 reader reads a v4 line's v2 fields exactly.
- **The plan's action is `paction`, not `action`.** v2 already has an
  `action` (the Borg's goal), and one object cannot carry the key twice;
  v2's `action` keeps its meaning. `tryon` also carries the equipment slot,
  `[shop, slot, eq_slot, value]`, since Q1 groups the wearables by it.

The replay recomputes the stock from v2's `shops` and v4's `wares`, the
budget from `limits` and `floors` (the floor pass is not replayed; its
result is the step's input and the pass has its own suite), and the values
from `traits`, `power`, `params` and `wares` - through the same `step()`
the producer runs, so the two cannot drift apart.

### B.5 Amended 2026-11-14, with the floors' removal

- `floors` is **`urgent`** (`[shop, slot, needs]` of each urgent purchase) and
  `unmet` holds the urgent needs no affordable ware met; the action's `floor`
  flag is **`now`** (the action was an urgent purchase).
- A new field **`spells`** follows `power`: the spell capabilities no trait
  counts, as bits - bit 0, a curing spell (Herbal Curing) is usable. The
  urgent and bleeding/poison needs read it, so the replay needs it.
- In Part A, the emergency-path rows ("cut or poisoned → the Alchemist",
  "no food or no light → the General store", "instant shopping") become the
  urgent rule; the two "literal changes" of A.1 are moot (no light floor, no
  standing poison capability); "the Home as a floor source" stays a gap,
  now for the urgent pass. The guards, the exclusions and A.4 stand.

### B.6 Added 2026-11-14, with slice 3 (the dispositions)

- **`kit`** follows `tryon`: per pack and Home stack, `[place, slot, kind,
  qty, weight, need, reduction, power, proceeds, sell_shop, flags]` -
  `place` 0 pack 1 Home; `power` the try-off (pack: what `borg_power`
  loses without it) or try-on (Home: what it gains in the pack);
  `proceeds` 0 under `birth_no_selling`; `flags` bit 0 unknown, bit 1
  unknown flavour, bit 2 just bought. The replay recomputes the
  dispositions from it.
- **`dispose`** follows `kit`: one code per stack, 0 keep, 1 store, 2 sell,
  3 drop, 4 take.
- `paction` uses the extended codes: the dispositions come before the
  purchases (pack sales and drops, stores, Home sales, takes).

## Part C - to decide in phase 4 (from the phase-3 re-review)

Not entry criteria; recorded so that they are decided in the phase, not
discovered after it:

- **the solver's refusal**: past its packed score's fields it panics,
  which the release profile turns into an abort. **Decided 2026-11-13**: keep the abort
  (the game's inputs are far inside the fields) and record `guard = 3` if
  it is ever reached in a test build;
- **reading `optimal`**: the plan treats `optimal = false` as information,
  not as an error - it executes the first ranked action either way, and
  the record's `solve.flags` say why the set was not proved.

## What accepting this means

- **Part A accepted**: #37's Stay rows are the plan's floors, guards and
  exclusions exactly as tabled, the two literal changes (the light floor
  without CL ≥ 2; the poison capability when not poisoned) are intended,
  and the Home becomes a floor source.
- **Part B accepted**: version 4 is the plan record's shape, "displaced"
  means the runner-up, and the ware tuple gains its stack class.
- Either can be amended row by row; an amendment is a dated entry here and
  in the epic's log.
