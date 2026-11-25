## 2026-11-25: Pin the unknown-consumables fixture's Warrior preparation (#1225)

### What was done

The seed-101 native fixture uses the Warrior preparedness values from
before the shallow class calibration. The overrides live in the fixture
process alongside its historical stair-routing override.

### What was found

With the newer class preparedness values the replay sells the potions
and reads Blessing, then misses the known Cure Serious Wounds quaff.
Pinning just the DL2 character-level gate produces a quaff at a different
encounter without the confusion clear. The six historical HP and level
gates reproduce all ten events, ending with the quaff and confusion
clear at turn 27,992. [Track P CI failure](https://github.com/diegoami/borg-reborn-rs/actions/runs/45523507202)

### What is left

Track P integration for this fixture repair awaits CI and owner review.
[#1225](https://github.com/diegoami/borg-reborn-rs/issues/1225)

### Owner decisions

The fixture retains the historical encounter while Track P uses the
shipped class calibration. [Calibration PR #1219](https://github.com/diegoami/borg-reborn-rs/pull/1219)
