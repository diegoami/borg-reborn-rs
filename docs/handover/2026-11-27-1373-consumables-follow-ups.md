## 2026-11-28: Experience, *Enlightenment* and one refused-kind rule (#1373, #1374, #1375)

Stacked on #1372 (`claude/friendly-cannon-6xxnck`, head `a0f8e196`, which
carries the review fixes: permanent stats, the narrowed floor filter, the
dungeon drop and RULES 28). One session owns the four issues. The three new
settings ship **on** (RULES 28, owner, 2026-11-28: "we do not have time to
wait for zoo ... we have no customers or rollouts"). Nothing is measured and
no improvement is claimed (RULES 14).

## Done

**The review of #1372** is fixed in #1372's own branch (its note,
`2026-11-27-1371-consumables.md`): permanent stats, a floor filter of the five
stat-trade potions only, the dungeon drop of a refused stack, and
`town_stat_trade_policy` and `town_stat_trade_floor` shipped on. This work
adds to it:

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
  --all-targets -- -D warnings`, `cargo test --release -p borg-core` (1017) and
  the parameter citation check pass.
- Native, at the shipped values: `test_junk_consumables.py` (the case for
  Experience and *Enlightenment*, and the quiet-turn case), and the stair,
  loadout and exploration-pack cases CI runs.
- The shipped smoke result is in the PR.

## Not checked

- The Ranger's Nimbleness and the Necromancer's torches have native cases
  (`test_a_ranger_drinks_nimbleness_and_sells_contemplation`,
  `test_a_necromancer_sells_the_torches_it_cannot_use`) on the class fixtures
  of #1378. A Necromancer's Light scroll is dropped as a kind either way
  (#1273), so the case shows the rule on its torches. No seeded Home case: no
  fixture seeds Home with a Light scroll, so what #1375 changes for Home
  classification is unit-reasoned, not native.
- No native Experience case at the XP cap: the build derives experience from
  the level, so the cap cannot be set.
- No native walk over a refused stack on another route; the drop of it is
  covered (#1372).
- No zoo comparison (RULES 28).

## For the owner

- The Rod of Restoration remains on `WORTHLESS` (a #443 gap: no zap code).
