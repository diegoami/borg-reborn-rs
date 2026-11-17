# M13 inventory calibration contract (#433 / P4)

The owner selected this mechanical extraction on 2026-11-17. The source is
accepted main `68e39b7e848017347ed80a9f474d9552ac98de6c`, donor
`8c5d000a58e9ddca1f1954ab9552e505856b313f`. New parameters append to the
registry: **362 new, 2,143 total**; existing enum positions and defaults stay
unchanged. This is an experiment surface, not a tuned policy or improvement
claim. [The complete inventory](M13-INVENTORY-PARAMETERS.json) lists each
name, default, units, inclusive bounds and pre-extraction source line, with
371 literal-use sites. Source references point to the accepted snapshot,
not shifting lines in the extracted files.

| Receiving file | New parameters |
|---|---:|
| `borg_home_power.rs` | 237 |
| `borg_item_wear.rs` | 23 |
| `borg_power.rs` | 48 |
| `borg_store_buy.rs` | 14 |
| `borg_store_sell.rs` | 34 |
| `borg_think_store.rs` | 6 |

## Policy extracted

- Wear: identification priorities and mana/danger/depth gates, ring-selection
  time budgets, backup danger fractions, bounce/cooldown limits, minimum
  power gain and the full-ring trial's RNG bound. The fixed ticket 10 remains:
  a bound N gives the same inherited one-of-N event. The lower bound 11
  keeps that ticket reachable; default 100 preserves the draw and call order.
- Buy/sell: enchanted-ammo and book level gates, black-market budgets,
  wand supply gates, home-stock purchase reserves, batch limits, sale value
  cap, protected supplies, greedy-sale tiers and power tolerance. The initial
  impossible sale price is derived as `cap.wrapping_add(1)`, default 30001,
  rather than exposing a sentinel independently.
- Store choice: town decision time/bounce budgets and light purchase gates.
- Home: per-copy flag weights, spare/excess equipment counts and penalties,
  stat valuation slopes/intercepts, consumable stock limits/weights/decay,
  level/depth gates, duplicate penalty and books/artifact/ego valuations.
  Intercepts are independent parameters: the inherited caster-tier jump and
  resistance-loop counter carry-over are deliberately preserved.
- Power: 48 remaining inventory weights, including dungeon-book reward
  15000 and encumbrance penalty 1000. The prior 726 power entries all remain
  referenced in the current power implementation; none is repurposed or
  renumbered. Earlier P2 counts/source lines describe its historical v12
  extraction, not a claim that M13 had complete coverage.

`params-core.json` records values and their default/file/environment origins.
Existing `borg_param_<name>` keys, `BORG_PARAM_<UPPERCASE_NAME>` overrides and
Zoo profile handling work unchanged. New names follow the receiving module:
`wear_`, `buy_`, `sell_`, `shopping_`, `home_`, and `power_inventory_`.
No new RNG, perception reads or decision passes are introduced. Widened
C-long terms remain i64 until their original truncation; i32 wrapping,
comparisons, tie order and quantity/charge save-and-restore remain in place.

## Retained literals and interfaces

This is a classification audit, not a replacement of every number.

| Remaining category | Why fixed |
|---|---|
| Empty/boolean checks, loop increments, one-item transfers | Presence and inventory mutation semantics, not preferences |
| `-1`, `255`, `9999`, `-99999`, initial best scores | Missing/invalid candidate sentinels; never experimental utility bonuses |
| Equipment offsets, shop/class IDs, 100-offset home encoding | Index/command representation; body slots are engine data (`lib/gamedata/body.txt`) |
| Array capacities, history ring 9/10, MEMSET_SIZE, book/spell bounds | Memory representation and existing protocol limits; not tunable counts |
| Batch key digits 2..5 | UI amount-entry implementation; new batch parameters are restricted to 1..5 |
| 18+100, stat index 37, class level 49/50 | Stat encoding/table end and maximum level; `src/player.h:74,81` at upstream 42cd156ac |
| Saving throw 100, slay damage 2x/3x/5x, dice/unit conversions | Game effect scales (`lib/gamedata/slay.txt`, `src/player-calcs.c`); existing policy weights remain separate |
| Carry weight /2 and /10, mana scale /100 | Engine burden/unit formula, `src/player-calcs.c:2220-2228`; only its policy penalty weight is extracted |
| Engine max_stack and two-copy spare basis | Data capacity and the existing valuation representation; stock multipliers and soft limits are policy |
| Store clock-wrap interval 20000..20010 | Existing defensive lifecycle guard, not a town preference |
| BORG_* options, supply helper contracts | Separate existing settings/contracts; not proof of named-threshold coverage |
| Self/money-scumming guards, stock/budget/mugging thresholds | Disabled exploit paths remain byte-for-byte and receive no new experiment knobs |

The source references above were checked against the pinned engine, after
reading manual chapters 6 and 11. Those manuals cite an earlier engine pin;
the checked-in engine at 42cd156ac is authoritative. No mechanics correction
or donor-policy change is bundled with this extraction.

## Bounds and safety

Divisors are positive. Narrow bounce counters stay within i16, RNG bounds
fit u32, and item counts fit their original u8 comparisons. Batch counts are
1..5 because the unchanged UI emits those quantities. The conservative home
stack guard can be tightened but cannot exceed its original 90. Generic
counts/weights are bounded independently; these are allowed experiments,
not recommendations. A reversed greed floor/ceiling is rejected during
configuration loading rather than panicking in `clamp`; stat tier and
resistance depth bounds are also checked for ordering. Different bases and
slopes may remain discontinuous, as in the original policy.

## Shopping replacement and R4 handoff

#293 remains open. The historical chooser is still selected by
`borg_town_plan = 0` (the harness default); the explicit M13 planner is chosen
by `borg_town_plan = 1` in `borg_choose_shop`. This work names the existing
reachable policy; it does not endorse it as the replacement specification,
restore removed opportunism or create a second shopping implementation.
The live planner's design, reserve policies and eventual chooser removal
need their own #293 contract/review. Tests cover both existing profiles.

#434 consumes this contract only after the P4 milestone is reviewed and
promoted to main. R must carry the exact accepted source separately and
preserve the existing R3 streams/history. Neither a closed task issue nor
an integrated track-p task constitutes promotion acceptance.

## Validation boundary

Every literal substitution was checked by reversing it to the exact source
expression before import/registry additions; the sale sentinel's default
also folds to the original 30001. Release tests retain home stat-tier and
integer-wrap semantics. Configuration tests reject unsafe bounds/orderings.
A bounded native test contrasts the default town budget with one turn and
requires changed keys plus recorded parameter provenance. Full native
45-case accepted-main/candidate comparisons, build receipts and canonical
natural-death custody belong in Cemetery; the source handover/PR link their
immutable evidence. These are compatibility/scenario checks, not a population
or survival experiment. Interactive GUI behavior is not claimed.
