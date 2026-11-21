## 2026-11-21: No Phase Door or teleport under a teleport curse (#829)

The Borg read Phase Door every few turns after "Teleportation forbidden!"
until it died (Priest 354395849, Ranger 1088421828, Warrior 1653223723 in
borg-zoo's reports), and never quaffed. It now learns the curse from the
engine's own words and stops.

### What changed (borg-core, all files already declared as hooked)

- `borg_escape.rs`: `TeleportBlock` and `teleport_curse`. The Borg learns the
  block from "Your X prevents you teleporting." (naming the item) and from
  "Teleportation forbidden!", and notes it
  (`# Teleport blocked (anti-teleportation on Cloak): no phase/teleport`). It
  keeps it while the watched equipment is the same (slot, kind, artifact,
  weight; the named item's slot, or every worn item when none was named) and
  lifts it when that changes or on "The anti-teleportation curse is removed!".
  `borg_allow_teleport` asks it, as does `BI_CRSNOTEL`.
- `phase_door` refuses first (`# Phase (<site>): no, teleport is blocked`), so
  all 14 sites are covered, not only the two behind `borg_allow_teleport`.
- The same check at the choke points `borg_read_scroll`, `borg_use_staff*`,
  `borg_activate_item` and `borg_spell`, for every teleport of the player
  (the engine's three handlers, effect-handler-general.c at 574b618c).
  Teleport Other is not touched: it moves a monster, not the player.
- `borg_messages.rs` feeds the messages in; `borg.rs` holds the state.
- docs/PHASE-DOOR.md says where the check sits.

### Decisions to look at

- `phase_door` asks only the curse, not `borg_allow_teleport` as the issue
  said: that also refuses on a no-teleport *grid*, which does not stop Phase
  Door (`dis` 10 is not `> 10`, `effect_handler_TELEPORT`). The long teleports
  still ask the grid.
- A bare "Teleportation forbidden!" on a no-teleport grid is the grid's, not a
  curse; it sets nothing.
- `BI_CRSNOTEL` is untouched, so equipment valuation does not change (the
  cursed-items work is #830).

### Checked

- The three cases replay exactly from seed, race and class (the zoo bundles
  were not reachable from this session, so the key hashes were not compared;
  the death turns and the `messages.log` lines match the issue's: 15401 and
  15710, 1404, 7762): Priest and Ranger on 3b4924ba, Warrior on e8b3acfd
  with the six `reborn-flight-flight` settings.
- With the change on those commits, after the first "forbidden" there is no
  Phase Door and no further "forbidden". The Priest (CL9) and the Warrior
  (CL13) now live to the 400,000-turn cap; the Ranger lives to turn 7,450
  (6,761 before), quaffing all 8 Cure Light Wounds, and dies of poison.
- The Borg took off the cursed item in the Priest and Warrior games, which
  lifted the block; the Phase Doors read after that are not refused.
- Smoke suite on the shipped settings: 3/3 identical to
  `harness/reference/smoke-borg-rs`, no re-record. Unit tests,
  clippy, fmt and `borg-core-vendor.py check` pass.
- These are three named cases (RULES 20), not a comparison; no claim of
  improvement.

### Left

- The Ranger's death after the block (8 Cure Light Wounds did not outlast a
  giant yellow centipede and poison at CL2) is not about teleport; it was not
  looked into.
- A Remove Curse errand and not wearing the item are #830.
