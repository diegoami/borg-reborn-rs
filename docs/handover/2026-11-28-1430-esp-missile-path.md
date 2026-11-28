## 2026-11-28: With ESP, a silent missile miss needs a known open path (#1430)

All checks ran on the default core, **borg-rs** (`borg-core/`, `harness/pborg.yaml`).
This change is stacked on #1431 (#1425).

### What was done

#1425 left one way into the same loop: a fired or thrown missile still
counted as "a miss, not a wall" at any monster in `BORG_VIEW` seen at the
shot. A missile also stops silently before a wall (`ranged_helper`,
player-attack.c:1101-1108). ESP shows a monster every think even behind walls
the Borg has never seen, and `BORG_VIEW` passes through unknown grids. So a
throw stopped by an unseen wall could be taken for a miss after every throw.

The owner chose (2026-11-28) to test the path only while the Borg has ESP.
With `BI_ESP`, `missed_visible_target` counts a silent missile miss as a miss
only when `known_open_path` holds: every grid of the `borg_inc_motion` walk
before the goal is known and not a wall or door. The goal's own grid is not
asked. Otherwise the miss takes the #929-guarded wall guess, as a
no-damage cast does since #1425. Without ESP, nothing changes. Test:
`with_esp_a_silent_miss_needs_a_known_open_path`. The `params.rs`
description, `docs/CALIBRATION.md` and `docs/c-borg/borg-rs-c-bugs.md` §41
now say so.

### What was found

The test is not applied without ESP because, in #1425's trials, requiring a
known path for every missile changed smoke mage 102 at turn 2501. There, an
ordinary flask miss at a rock lizard, across dark floor the Borg had not
seen, got a granite guess. Without ESP a monster on screen is seen by
sight, infravision or detection, so such a miss is an ordinary one.

### What was not checked

- No recorded case of the missile loop exists. The change comes from reading
  the source, like the issue.
- Smoke characters have no ESP, so smoke cannot show the new path. Its result
  is in the PR.
- No zoo measurement. The owner noted that measuring this needs a
  purpose-built world: characters with ESP facing monsters behind unseen
  walls. That is not set up yet.

### What is left, and owner decisions

- With ESP, a real miss across unseen dark floor now gets granite on that
  floor, as in C, until the Borg sees the grid. That is the accepted cost of
  this option.
