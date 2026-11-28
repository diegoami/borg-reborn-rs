## 2026-11-28: Step off a shop entrance before a town drop (#1397)

### What was done

- `borg_junk::step_to_drop_floor` is the clear-floor rule `drop_surplus`
  already had (#1256), moved into a helper. Off plain floor (a shop
  entrance or stairs) or on a visible pile, it queues a step to an adjacent
  clear, visible, safe floor grid, or refuses the drop when there is none.
  `drop_surplus` calls it unchanged, apart from a new
  `# Stepping to clear floor before dropping` note.
- The town planner's plain DROP (`borg_town_live::execute_outside`, pack
  and quiver drops) now calls it before any side effect. After a step the
  action is cleared and the next think plans the drop again from the floor
  grid; a Home disposal keeps its `disposal` record across the step.
- The plain-drop step is behind `town_drop_clear_floor` (shipped 1 in
  `harness/pborg.yaml`, documented in `docs/CALIBRATION.md`). The exact-turn
  stair replays (`test_stair_handoff.py`, seeds 204 and 224) pin it to 0:
  their games make such drops long before the recorded stair turns (seed 204
  steps off a shop door at turn 31060 to drop 2 Potions of Slowness), so the
  routes moved and the tests stopped reaching their assertions (review on
  #1398).
- The drop is still `d` with the `borg ignore` inscription; no `k` (owner,
  on #1397: `k` would open the door to discards nobody notices).
- `test_inferior_spares.py` (`TownSpareSaleTest`, the ranked-home case)
  asserts the step comes before the drop, the dropped sandals are ignored
  where they lie (`# Destroying floor item`) and are never picked up again.

### What was found

- The existing ranked-home fixture already showed the defect: the Borg took
  3 sandals out of Home and dropped them from the Home entrance at game
  turn 4325 with no move in between; the engine put them on (29,17) and the
  Borg walked away without ignoring them. With the change it steps
  (turn 4335), drops under itself and the next think ignores the stack.
- Only open floor holds objects (`terrain.txt`, flag `OBJECT`), so every
  drop from a shop entrance scatters (`obj-pile.c:drop_find_grid`), and
  `pickup_always` overrides pickup inscriptions (`cmd-pickup.c:169`).
- Smoke at the shipped settings is unchanged against `smoke-borg-rs`
  (4/4 identical), so no re-record.

### What is left, and owner decisions

- #1397's last fix bullet, recording a drop that did not take so the same
  drop is not planned again at the same game turn, is not done: the step
  removes the case that was seen. The owner decides whether it needs an
  issue of its own.
- Not checked: other town drops outside the planner (cursed-spare and
  dungeon junk paths that can run in town).
- Whether the failed drop contributed to the #1395 loop is still open.
