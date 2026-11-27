## 2026-11-27: the #1372 review, and Experience, *Enlightenment* and one refused-kind rule (#1373, #1374, #1375)

Stacked on #1372 (`claude/friendly-cannon-6xxnck`, head `2f70ab1b`). One
session owns the four issues. It writes this note, and not an edit of
`2026-11-27-1371-consumables.md`, which says the stat-trade policy ships on.
**That is no longer so: see "For the owner".**

## Done

**The Codex review of #1372** (comment on the PR, findings at `2f70ab1b`):

- **P1, undrained base values.** `second_stat_potion_wanted` now reads
  `player->stat_max` (`engine::player_notice().stat_max`), not `BI_C*`
  (`stat_cur`). Nimbleness restores the stat before `GAIN_STAT`
  (`effect_handler_RESTORE_STAT`), so the gain starts from the maximum. The
  loss, `player_stat_dec` with `permanent`, lowers another stat's maximum, so
  a stat drained right now still costs its maximum. A pure variant,
  `second_stat_potion_wanted_at(class, stats, stat)`, carries the arithmetic
  and the tests: the drained-at-cap Ranger (Dexterity 18/90 from 18/100) is
  refused, and a drained Strength no longer hides a table step of loss.
  The restoration itself is not priced: the engine undoes a drain at the next
  level-up and a Restore potion does it for less. A drained stat's
  restoration is therefore not a reason to drink.
- **P2, the floor filter.** `refused_take_kind` no longer skips the whole
  `WORTHLESS` list. It skips the refused stat-trade potions
  (`town_stat_trade_policy`) and, with `town_refused_kind_policy`, the refused
  kinds that no shipped dungeon policy uses: Treasure Detection stays wanted
  with `item_use_idle`, and the junk potions (Poison, Surprise, Salt Water,
  ...) with `attack_throw_junk_potions`.
- **P8.** `harness/pborg.yaml` ships `town_stat_trade_policy: 0` again
  (PLAN.md, P8: "No new capability"; new P9 capabilities may merge into
  `track-p` only shipped off). It is one line to turn on in P9, after its own
  zoo comparison. Every other new parameter ships 0 too.

**#1373, `item_use_drink_experience`.** `borg_use_things` drinks a known
Potion of Experience next to the pure stat gains (`drink_permanent_gain`).
After the issue review: only on a quiet turn (danger here at most a tenth of
current HP, `quiet_for_a_drink`; `borg_use_things` also runs in the hungry
path before the caution checks, so the placement alone was no guarantee), and
only while the engine grants experience: `GAIN_EXP` does nothing at
`PY_MAX_EXP` whatever the level (`experience_potion_useful`). At the cap the
planner's `GAIN_EXPERIENCE` weight is 0 too, so it cannot pay for a useless
copy.

**#1374, `item_use_star_enlightenment`.** The same function drinks a known
Potion of *Enlightenment* (looked up by name:
`item_val.sv_potion_enlightenment` is the plain Enlightenment). After the
issue review it is drunk in any place only while Intelligence or Wisdom is
below 18/100 or drained (`star_enlightenment_gains`, on `stat_max` and
`stat_cur`). With both capped and undrained only its mapping and detection are
left, so it is saved for a dungeon level (drunk on the first quiet turn there)
and the kind stays on `WORTHLESS`: the planner does not buy it for the map and
a town drink would waste it. While a stat can gain, the kind leaves
`WORTHLESS` (`worthless`), so the planner values its `GAIN_STAT` rows and
`unwanted_home_item` no longer sells or drops it. One parameter for both.

**#1375, `town_refused_kind_policy`.** `refused_kind(st, tval, sval)` in
`borg_town_capability.rs` is the one predicate: `WORTHLESS` (with Monster
Confusion without `caution_cornered`), a refused gain-one/lose-one potion, and
for the Necromancer a Wooden Torch and a Scroll of Light (`unlight_refused`).
It is read by `supplies_of_kind` (every origin: shop, pack, Home, so a carried
Light scroll is no longer valued and the planner sells or drops it),
`unwanted_home_item` (so `borg_town_home::classify` says `Reject`) and
`refused_take_kind`. The shop closure in `borg_town_shadow.rs` and
`borg_flow_take::unwanted_light_scroll` still apply the Necromancer rule
unconditionally, as before; they were left alone so that 0 keeps today's play.

## Found

- The issue's example for #1375 was wrong, as its review says: a Necromancer
  already drops a carried Light scroll in the dungeon, by kind
  (`borg_discard_light_scrolls`, #1273; native test `test_light_scrolls.py`).
  The gap is town: the planner values a Light scroll in the pack or Home
  although it never buys one, and the Home classifier says `Renewable`.
- `engine::player_notice()` cannot be reached from a borg-core unit test (the
  release test binary does not link the engine's `player`), hence the `_at`
  variants. The shop/pack/Home/floor wiring is covered natively, not by unit
  tests.
- The effect-based harmful-consumable rule (#1253, `borg_junk::harmful_item`)
  stays separate: it reads a kind's effects, not its tval and sval.

## Checked

- `cargo fmt --check -p borg-reborn-rs`, `cargo clippy --workspace
  --all-targets -- -D warnings`, and `cargo test --release -p borg-core` (1017
  passed: three new unit tests on top of #1372's 1014) pass. The parameter
  citation check passes (2676 parameters).
- The native case `test_town_drinks_experience_and_star_enlightenment_on_purpose`
  (`test-junk-consumables`).
- Smoke: every new parameter, and `town_stat_trade_policy`, ships 0.
  `scripts/rerecord-smoke-borg-rs.sh` reported 4/4 games identical to
  `smoke-borg-rs` (default core), so nothing is re-recorded. The unit tests,
  scripts and tools tests, and the native cases passed as well.

## Not checked

- **P3 of the review: a stack walked over on another route.** The Borg sets
  `pickup_always`, so a refused stack stepped on is picked up (the engine's
  `auto_pickup_okay`), whatever `take_select` chooses. This change only stops
  the Borg walking to it. The stack then waits for town, where it is sold or
  dropped, and costs a pack slot until then. Closing it needs a dungeon-side
  ignore of the stack (`k`, "this item only": the engine drops it and filters
  it from pickup) on a quiet turn, like `borg_discard_town_consumable`; that
  drops the stack instead of selling it, and needs its own native case. See
  "For the owner".
- No native Necromancer case: the synthetic build is a Human Warrior only, so
  the Light-scroll and torch rule is unit-tested (`unlight_refused`) and
  covered by the same predicate path as the others.
- No zoo comparison. Nothing here is turned on.

## For the owner

- **`town_stat_trade_policy` now ships 0** (it shipped 1 in #1372). That
  follows PLAN.md P8 and the review. To ship it on during P8, record an
  exception; otherwise turn it on in P9.
- **The dungeon-side drop of a refused stack (review P3)** is not done. Say
  if it is wanted: it is a small `borg_junk` routine, but it changes the
  outcome from "sold in town" to "dropped where found".
- The Rod of Restoration remains on `WORTHLESS` (a #443 gap: no zap code).
