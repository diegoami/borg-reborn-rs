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

### The flag

`escape_teleport_block` (compiled 0, shipped 1 in `harness/borg.txt`, row in
docs/CALIBRATION.md). At 0 the message hook, `phase_door` and the choke points
do nothing new: the Ranger replayed with it at 0 is byte-identical to the
original (`messages.log` and `keys.log`). It exists so that borg-zoo can run a
control (`borg_param_escape_teleport_block = 0`) against the shipped build.
The param citations in params.rs and CALIBRATION.md moved with the new lines
(`scripts/check-param-citations.py --fix`).

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

- The three cases replay exactly from seed, race and class, and their
  `keys.log` SHA-256 equals the one borg-cemetery records for each dossier
  (D-c78e3d5a, D-0c44ce2b, D-7a8b9fc3): Priest and Ranger on 3b4924ba,
  Warrior on e8b3acfd with the six `reborn-flight-flight` settings. (The
  zoo bundles themselves were not reachable from this session; the
  Cemetery's records were. My `borg.txt` is the commit's shipped file with
  the arm's settings, not the zoo's rebuilt one, and plays the same keys.)
- With the change on those commits, after the first "forbidden" there is no
  Phase Door and no further "forbidden". The Priest (CL9) and the Warrior
  (CL13) now live to the 100,000-turn cap of the zoo cells; the Ranger lives to turn 7,450
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
