# The town-visit dump

The town-visit dump is an observation-only record of every town visit the
Borg makes: the state it was in, the stock of the shops, and the decision
the current policy took.

The dump changes nothing. It never presses a key, never changes a goal,
never changes a setting and never changes what the Borg decides, so the
keypress stream of a run with the dump enabled is the stream of a run
without it. The C/Rust parity suites stay valid.

## Running it

The dump is off unless the environment variable `BORG_TOWN_DUMP` names a
file:

```sh
BORG_TOWN_DUMP=OUT/town-dump.jsonl BORG_IMPL=rust \
    harness/run.sh OUT 42 1 5 2000
```

The file is created if it does not exist and appended to if it does. One
JSON object per line.

To read it:

```sh
tools/town_dump_report.py OUT/town-dump.jsonl --limit 20
tools/town_dump_report.py OUT/town-dump.jsonl --limit 20 --markdown
```

The renderer prints one block per visit. `--full` prints every shop's stock
rather than only the shop of the decision, `--action buy` keeps only the
buys, and `--distinct` keeps only the first visit of each distinct
decision: the Borg re-decides the same thing every turn while it walks to
the shop, so the repeats are the same decision again and only the first
visit carries its reason.

In `--markdown` the state and the stock are folded into a `<details>`
block and the decision and the reason stay visible, so a batch of visits
can be scanned on one screen.

## The records

A `decision` record is one town visit:

| field | what it holds |
| --- | --- |
| `seq` | the record number over the run |
| `turn`, `depth`, `level`, `hp`, `gold` | where the Borg was |
| `race`, `class`, `name` | who the Borg was |
| `traits` | `light`, `food`, `cut`, `poisoned` |
| `pack`, `equipment` | the pack and the worn kit, with the cost of each |
| `home` | what the home held |
| `shops` | every shop, and every ware in it with its price |
| `decision` | `action`, the goal ware or item, the shop, and `reason` |

A `purchase` record is a transaction that was actually carried out: the
ware, the price, and the gold before it.

The `reason` is the Borg's own note, the same text that reaches the
message log. It is the closest thing the Borg has to a justification, so it
is kept verbatim rather than paraphrased.

## The integer projection

`BORG_TOWN_DUMP_INTEGERS=<file>` writes a second file: the same records
with **integers only**, a fixed field order and a version tag.

```sh
BORG_TOWN_DUMP=OUT/town-dump.jsonl \
BORG_TOWN_DUMP_INTEGERS=OUT/town-dump.i32 \
    harness/run.sh OUT 42 1 5 2000
```

The descriptive records carry names, and a name is not comparable between
two runs; an integer is. The projection is what the record schema (P13)
asks for - a versioned, machine-readable line, integers only, a stable
field order, a total ordering - so it is here to be specified against, and
to make the two sides of a shadow-mode comparison (today's action and the
plan's) diffable as numbers.

It is not the record schema: P13 wants the line to carry the plan's
decision as well, and to be replayable from its own logged inputs. This
one carries today's side only.

| field | what it holds |
| --- | --- |
| `v` | the schema version; **2** is the frozen shape (P13) |
| `seq` | the record number over the run; the records are totally ordered by it |
| `turn`, `depth`, `level`, `chp`, `mhp`, `gold` | where the Borg was |
| `race`, `class` | the game's indexes, not the names; `-1` if the class is not set |
| `traits` | the **whole** trait vector, sparsely: `[index, value]` each, and a trait that is absent is zero |
| `action` | 0 visit, 1 buy, 2 take_home, 3 sell, 4 sell_home |
| `shop`, `ware`, `item`, `price` | the goal, and the price of the goal ware |
| `pack`, `equip`, `home` | `[slot, kind, subkind, quantity, value, flags]` each |
| `shops` | `[shop, [[slot, kind, subkind, quantity, price, flags], ...]]` each |

The file holds two record shapes. A **decision** record is the table above; a
**purchase** record is the transaction that was carried out: `v`, `seq`,
`turn`, `shop`, `price`, `gold_before`, and the `ware` as
`[slot, kind, subkind, quantity, price, flags]`. A decision record carries an
`action`; a purchase record carries the gold before it. The renderer tells
the two apart on that.

An absent value is `-1`, everywhere: no goal ware, no goal item, no price,
no class.

**Why the whole trait vector.** Version 1 carried the four traits the town
strategy reads today - light, food, cut, poisoned. Version 2 carries every
trait that is not zero, by index, because the usefulness function reads
traits nobody has written down yet (the poison-cure capability among them),
and a record that carries only what happens to be read today cannot be
replayed against one that reads something else. The descriptive record keeps
its named handful - it is the human view, not the machine one.

**The awareness flags** are a bitmask on every item: bit 0 is "the character
knows what this is", bit 1 is "this item is not identified yet". They travel
with the item because the knowledge need reads them.

**Version 2 is frozen.** P13 wants the record schema fixed before phase 4,
and this is it: the shape the baseline is built on, and the shape the
shadow-plan mode will carry both sides in. A change to the field set is a
new version, not an edit - the tag is what tells a reader which shape a file
is.

To read it:

```sh
tools/town_dump_report.py --integers OUT/town-dump.i32
```

## What the dump is for

The owner's judgement in this plan enters at exactly one place: the
**usefulness table of phase 1** (#41) - the table of states and their
expected usefulness that phase 1's suite asserts. That table is the door
the owner reads, and it is the one to read first. The dump is not a
second door.

It is what that table is built from:

- **the states are real.** The dump carries the town visits of headless
  runs, each with the state and the stock: the inputs a usefulness
  function needs. Phase 1's table can be seeded from the discriminating
  visits rather than from invented states.
- **the diff is automatic.** Once the usefulness function exists, the
  dump says what the current policy bought at every visit and the new
  function says what it would buy. The disagreements come out on their
  own; the owner reads those, not every visit.
- **it is the debugging view for phases 1 to 4**, which is where a
  usefulness function needs one.

The labelling route - the owner naming the need for every visit - was
dropped on 2026-11-12. The need list (#35), the decay (P4), the
alternatives (P5) and the floors were all decided in the epic's log that
day, and the weights are the free parameters the zoo sweeps in phase 4;
so the labels re-derived what was already settled, at eighteen times the
effort.

Two limits are worth keeping in mind:

- The dump is a **control description, not a judgement**. It shows what
  the current policy decides; it cannot say whether those decisions are
  right, because the escape deficit and the trip model do not exist yet.
- The visits are the **current policy's visits**: the states where the
  current algorithm is not obviously wrong. Harvesting from several
  policies, including the `borg_town_no_stores` baseline, widens the
  sample.

## The baseline

The dump is a **live log**: it records what the current policy does, in the
run that is happening. The **baseline** is the frozen half of it - the
distinct decisions the current policy makes, at real states, in a form a
decision function can be run against.

- `fixtures/town-baseline-*.jsonl` - the distinct decisions, one file per
  class, in the integer projection's frozen shape (schema version 2). Each
  record is a state and the control's choice in it.
- `docs/TOWN-BASELINE.md` - the readable half: the choices with their names,
  the gold, the depth and the Borg's own reason, and the provenance.
- `tools/town_baseline.py` - what made them, from the runs.

The baseline is what the new algorithm is checked against. It does not claim
the current choices are right; it says what they are.

## The plan record (version 4)

`BORG_TOWN_PLAN_RECORD=<file>` writes the plan's side next to today's: at
every town decision (the same call that writes the v2 decision record) the
plan's step runs on the same state, in **shadow mode** - computed, never
executed - and one line is written: v2's fields for that state, today's
decision among them, with `"v":4`, then the plan's fields
(`docs/TOWN-INVENTORY-PHASE4.md`, Part B and its correction B.4). Every line
replays to its own plan from the line alone:

```sh
BORG_TOWN_PLAN_RECORD=OUT/plan.jsonl harness/run.sh OUT 101 0 0 20000
PLAN_RECORD_FILE=OUT/plan.jsonl cargo test --release -- --ignored a_run_s_records_all_replay
```

The record is observation only: the key log is the same with it and without.

## Where the code is

- `borg-rs/src/borg_town_dump.rs` - the dump and its integer projection.
- `borg-rs/src/borg_think_store.rs` - `borg_choose_shop` records the
  decision, `note` records the reason, and `town_gate_closed` is the null
  arm's gate.
- `borg-rs/src/borg_store_buy.rs` - `borg_think_shop_buy` records the
  purchase.
- `borg-rs/src/borg_io.rs` - `queued_keys`, the test's window on the key
  queue.
- `tools/town_dump_report.py` - the renderer.
- `tools/town_baseline.py` - the curation: the runs to the baseline fixture
  and its summary.

## Plan record v5: trip-relative utility (2026-11-14)

Schema 5 supersedes the fixed-target shadow model for newly produced records.
The v4 field order remains, with these explicit changes:

- `params` has 24 entries: the original 12, followed by `target_depth`,
  `trip_turns`, `w_gold`, `depth_gain`, `explore_turns`, `stair_turns`,
  `food_turns`, `light_turns`, `risk`, `home_discount`, `w_space`, `power_scale`
  (each prefixed `borg_town_`).
- Ware and kit `reduction` use thousandths of a supply unit; Healing uses
  thousandths of HP, with per-copy capacity capped at maximum HP.
- Utility values use thousandths. `solve[0]` is net kit improvement after the
  cost of spending the purse; individual plan-option values are gross.
- `kit` rows append a twelfth field, `serves`, the secondary-capability bitmask.
  CCW can therefore be protected as a poison cure independently of HP utility.
- `plan` group key kind 2 identifies one shared need (key = Need enum index).
  Key kinds 0 (stack class) and 1 (equipment slot) retain their meanings.
- A solve certificate applies to the generated options; the shared-need mixed
  option path is not an exhaustive enumeration of every possible mixture.

The forecast derives from recorded traits and parameters; it reads no additional
engine data at replay. The reader also accepts historical v4 records with their
12 parameters, 11-field kit rows and old fixed-target code path. Unit and utility
semantics are chosen by the schema, never inferred from the numeric magnitudes.
See [TOWN-UTILITY](TOWN-UTILITY.md) for equations and policy assumptions.

## Live record revision 6

The [experimental live executor](TOWN-LIVE.md) emits revision 6 with the same
row layouts and 24 parameters. Mode 1 records actual quote-based live planning;
mode 0 remains observational. Purchases are applied one copy at a time. The
revision also distinguishes the empty-value Home-cycle correction from schema
5's historical dispositions; schemas 4/5 still replay with their original rules.

### Capability records (v7)

The combined capability repair adds integer tables `capability_curves`
(`[id, evaluated_weight, demand, carried_units, Home_units]`) and
`capability_supplies` (`[scope, slot, id, units_per_copy]`). Scopes 0–6 are
shops, 8 is the pack and 9 is the Home. IDs are append-only in
`borg_town_capability::NAMES`; the recharge-target row is a zero-weight
prerequisite marker. Negative demand denotes a one-time access/repair
requirement, positive demand a diminishing stock-coverage scale. All native
class, item-effect and spell facts necessary for pricing are materialized;
replay does not consult live game data. Raw new calibration settings remain
visible in native `effective_settings` and the run's config; derived weights
and demands are sufficient to reproduce each individual solve.

Versions 4, 5 and 6 keep their historical replay paths. Version 7 uses capability
facts consistently for pack/Home dispositions and purchases. The version-6
worthless-storage and just-bought Home-sale guards are retained.

## Schema 9: reviewed capacity and transaction facts

The PR83 correction candidate writes schema 9; schemas 4–8 keep their original
replay behavior. `packing=[quiver_slot_size,current_weighted_quiver_units]`
and `quiver_classes=[[class,compatible_existing_stack_room],...]` record native
quiver capacity. Ammo groups share the aggregate pack-reservation bands rather
than each claiming the same free quiver room. `available=[[shop,slot,qty],...]`
records effective offer quantities, including zero for offers rejected under
an unchanged live carrying/wallet context and one for an urgent Home source.
These quantities override raw stock quantity during replay, without editing
v2's observed shop rows. Kit flag bit 3 marks a whole Home stack that fits
existing reserved/pack stack room. Known Home sources can satisfy urgent needs
before a paid purchase. Full-pack urgent actions first free feasible room.

Capability IDs 54–56 append cut reduction, stun reduction and monster control;
prior IDs are preserved, including the historical combined wound/stun row.
Active condition demands and reductions use actual native timer units.
See [the review disposition map](TOWN-PR83-FIXES.md) for classifier, rod,
random-branch, shared-slot and reserve changes. Native receipts and observed
wallet/inventory changes establish transactions; a queued command does not.

## Ranger missile utility (schema 10)

Schema 10 preserves IDs 0–56 and all schema 9 transaction/packing behavior.
ID 57 is Ranger usable shots: each known, non-cursed missile compatible with
the current launcher supplies one shot in addition to its damage. Oil, devices
and spells supply no shots. Pack, quiver and Home supply use the same curves.
The Ranger-only weight is `borg_town_w_ranger_missiles` (calibrated candidate 250;
0 disables); demand is planned trip turns / `borg_town_ranger_missile_turns`
(initial candidate 50 actions/shot), at least one shot. Decreasing marginal
utility lets price, gold, healing, escape and capacity still compete. These
are policy estimates for calibration, not measured combat rates or fixed quotas.

Native `PF_UNLIGHT` suppresses fuel utility even with a worn light. Torches
are unavailable purchase/urgent-withdrawal candidates for unlight characters,
before equipment or learning bonuses; the recorded `available` rows preserve
that eligibility decision. Other equipment lights may still offer gear benefits.
Schemas 4–9 retain their original recorded curves and replay behavior.

The subsequent PR85 review follow-up narrows the live control-effect producer
without changing row layouts: unsupported confusion/stun devices have no
control supply, while slow/hold/sleep and the enabled Scare Monster consumer
retain it. Schema 10 records freeze their own curves/supplies, so earlier
recorded plans retain their original values during replay. Feasible urgent
selection scans beyond a blocked first candidate in live mode.


## M13 integration (plan schema11)

Town trait IDs now have a stable serialized namespace, independent of the
native engine enum. IDs0..264 retain M12 meanings; appended IDs265..269 are
M13 stun protection, fire/electric/cold/acid vulnerability. Native M13 inserted
these fields internally, so replay translates indices explicitly. All traits
remain represented, and the v2/v3 row layout is unchanged. New plan records
use v11; archived v4..10 records retain their values and replay semantics.
The M13 Confuse Monster wand consumer is enabled without enabling the unused
staff. The M12 experiment remains on its frozen earlier source and schema10.
