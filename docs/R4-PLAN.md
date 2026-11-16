# R4: items after accepted R3

Owner acceptance of R3 #427 opens R4 on feature/track-r. Accepted source
02c273db1938ab61e18b22458d229155a0204164 remains immutable. The review
ledger merge is 17992dfcfc74509b723c861b41056f45993dc37c. Reference refresh
#429/#432 does not change runtime donor M13
1443c2d1b97d110ffbf5ae420d6c6b97ac15f7ea or its accepted gameplay evidence.

## Counted scope and parameter gate

Run `python3 scripts/r4-surface.py` to verify the receiving source inventory
in R4-SOURCE-INVENTORY.json. `--write` is an explicit reviewed replacement,
never a check-time update. The lexical scanner includes tests, methods and
all cfg arms. Its namespace/field/symbol spellings are a reading aid, not
an exhaustive call graph or semantic proof. Hashes lock the actual bytes.

| Group | Files | Physical lines, including tests | Function definitions, including tests |
| --- | ---: | ---: | ---: |
| Primary power/wear/use/activation/store/home surface | 9 | 9,363 | 204 |
| Adjacent item lookup/analysis/ID/decurse/enchant | 5 | 2,068 | 48 |
| Explicit shared item/inventory/home notice/formula/buy strategy/trip/town services | 19 | 13,814 | 556 |

The original 9,000-line estimate describes the primary surface roughly;
it excludes adjacent maintenance and extensive shared M13 town services.
No claim is made that all 25,245 counted lines will be independently ported.
Each slice must state ownership before expanding its scope. Initial effort
remains provisional: R4 2–3 times the R2 migration, not a calendar promise.
The larger shared boundary and parameter gate make a precise date unjustified.
Re-estimate R5 from the completed R4 report, not from this lexical inventory.

Power uses 726 distinct registered Param entries at this receiving M13
baseline; item use uses 19. The historical CALIBRATION.md count727 for
power is superseded for this baseline by the exact source inventory (all
726 registered power_* names are used). The older accepted P1-P3 review
#259 covers these contracts. Neither lexical counts nor that acceptance
prove every remaining numeric literal is correctly classified.

Wearing, home power and inherited store-buy/sell/think contain no Param
references. Existing config keys do not cover all policy literals: wear's
ID mana50, ring timers1000/2000 and anti-bounce300; home duplication weight
50000 are concrete audit leads, not endorsed values. #433 requests the
P-before-R handoff: classify policy versus engine/mechanics/UI data, name
and validate the policy controls, then obtain owner-reviewed P promotion.
Do not implement that P work in this R session or treat a closed task as
acceptance. Item use can proceed. Detailed power inspection also finds the M13
dungeon-book reward15000 and encumbrance weight1000 still literal: these
are policy magnitudes, not book iteration bounds or engine damage formulas.
#433 includes their accepted P extraction. Power boundary design can proceed,
but complete power-policy migration and wearing/store ports stay blocked until
their exact accepted contract can be carried in its own PR. #406 remains a
separate class/family carry with its own accepted-P dependency.

## Manual boundary and restoration audit

- **Power (#434):** own equipment/inventory/activation valuation and exact
  signed wrapping/truncation/addition order. borg_power is mutable despite
  being a valuation: borg_prepared_c writes the prepared-message buffer.
  Dynamic formulas are a configured shared policy dependency until explicitly
  ported; a parity pass through that branch is not independent formula coverage.
  Engine tables/spell facts, item weight, random-power facts and spell stat
  remain typed queries. ActivationIds is engine-derived lookup knowledge.
- **Use and activation execution (#435):** all24 public item-use functions,
  not only the already taped callers. Slots/availability, failure thresholds,
  charge/timeout checks, command choice, random selection, messages and object
  logs must preserve exact native ordering. Engine no_light, random draws and
  key mappings stay named tape queries/effects. Returned item facts must be
  owned or safely borrowed; no borrow survives an effect. The365-line
  activation file initializes names/IDs and startup-failure state; leaving it
  shared is explicit, not a second independently owned activation chooser.
- **Wearing/maintenance (#436, gated):** wear_trial changes equipped and pack
  safe-item slots, reduces quantity, calls borg_notice, then power and danger,
  restores slots and clears the hole. Other paths have their own restoration
  order. Preserve native recomputation and any persistent native side effects;
  comparing only the selected slot misses corrupted traits/safe copies.
  backup_try intentionally leaves restoration to its caller. Rings, native
  nonbinary critical resist values, anti-bounce clocks, ID, charge wearing and
  adjacent maintenance need separately named witnesses. Inventory classification,
  notice/recomputation, spells and flow/rest checks remain declared dependencies.
- **Store/home (#437, gated):** buy/sell trials mutate home and inventory
  quantities, device pval, safe_shops/safe_items, goals and notice caches.
  Charge division/remainder order, empty-slot constraints and early returns
  are part of parity. Actual stock/price/options/keypresses stay behind the
  receiving engine boundary. Default no-selling means giving an item away;
  preserve actual native outcomes and rejected transaction state. Disabled
  scumming branches do not authorize enabling them.

The M13 opt-in planner is a separate, substantial algorithm: town_live
executes choices from town_shadow and shared utility/usefulness/packing/
solver/record/trip services. With BORG_TOWN_PLAN=0 the inherited chooser
remains the control. A port of store UI execution does not establish that
this planner's choices are independently owned. Identify that shared scope,
config admission and possible future ownership in the store task. No newer
upstream home/return fix is imported by the source inventory.

### Nested comparisons

Current decision_api item helpers record an opaque stateful effect. The
driving effect runs under decision_tape::Pause, which lets a public item
facade open its own comparison transaction without consuming the suspended
outer tape. That inner scope must restore the outer tape and pause depth;
outer replay consumes the recorded result/post-state and never repeats the
real command, RNG draw, logging or transaction. Existing R3 public services
use this structure. Add distinct typed Decision/Value cases, private native
implementations and complete-entry registration; never let a wrapper call
itself, obtain its own native policy result as a dependency, or compare the
same active entry recursively. Internal port calls own their policy bodies.
Test real nested callbacks, both driving modes and complete state equality;
this audit does not claim those future R4 wrappers are implemented.

## Ordered tasks and shopping direction

1. #429 reference refresh; #431 this inventory/ownership audit.
2. #435 complete item use/activation execution under accepted parameters.
   #434 power boundary design can proceed, but complete inventory policy
   depends on the M13 magnitude gaps in #433. Select the lowest fully
   dependency-ready unclaimed task; do not confuse existing names with
   exhaustive policy classification.
3. #433 accepted P threshold contract and a separate exact source carry;
   then #436 wearing/maintenance and #437 store/home ownership.
4. #438 combined evidence, metrics, R5 forecast and immutable owner review.

#293 requests replacing inherited shopping with an explicit plan: reserves,
shortages and useful purchases precede disposal, with explained completion
and interruption. A mechanical parity migration does not fulfill or close
that request. The store task must reconcile the new plan contract with the
existing M13 planner and distinguish migration from changed strategy. A new
policy needs its own scoped behavior contract and positive purchase/retention
cases. Neither an opaque native chooser result nor an unchanged no-purchase
run is proof of a new shopping implementation. Population comparisons remain
the owner's zoo workflow. Proposed direct-buy engine behavior is excluded.

## Validation and custody

Each runtime slice pins exact receiver source/build/settings/native donor
and runs off/check/switch smoke plus the full45 suite, with zero check
mismatches and exact switch gameplay streams. Keep the accepted R3 M13
control as history; a later accepted runtime carry must establish and name
its own baseline rather than rewrite old claims. Per-entry strict coverage
must identify the expected check/switch mode. Invocation is not branch or
positive-action coverage. Include selected positive valuation/consumption/
activation/wear/store cases, unavailable/immune/empty/charging/invalid inputs,
failed transaction/early-return restoration, RNG/effect-once nested calls,
and literal fault controls followed by the exact repaired re-port. Owned
constructed states are synthetic; they cannot inflate natural-game counts.

No assertion, cap, timeout, seed list or engine boundary is weakened. Keep
every failed attempt and reconcile every measured natural death. Original
reports/dossiers/scenarios go to borg-cemetery, with qualified source/build/
registry identities and immutable source receipts/manifests. Code/tests,
source metrics and this concise work plan belong here. The R364/R365 owner
publication approval applies to those prepared bytes, not automatically to
new unrelated recordings. #344 remains the external central numeric-ID gate.

All green task merges target feature/track-r. At #438 freeze a new immutable
review/R4-SHA snapshot into review/track-r-baseline and stop. The owner chooses
the reviewer and accepts the milestone; name implementation family Codex/GPT-6.
No R4 task authorizes main/global-reviewed promotion, R5 implementation,
sealed seed use, performance/survival claims or population runs.
