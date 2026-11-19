## 2026-11-19: smoke-borg-rs re-recorded after e0c4a527 (#635)

- **Why:** `e0c4a527` changed two shipped values in `harness/borg.txt`
  (`caution_scary_damage_percent` 60 → 50, `caution_rising_explore_ticks`
  300 → 500) without re-recording `harness/reference/smoke-borg-rs`. Main's
  CI engine job had been red since then (2/3 identical), and so was every PR
  merged with main.
- **What changed in play:** one line. `smoke-mage-2` notes `# Scary by
  damage: grey mold, 5 of max HP 9` (56%, at or above 50, below 60). Keys
  and state are the same in all three games. `caution_rising_explore_ticks`
  leaves no trace: the smoke games never rise to town.
- **How:** a clean build of main `4acf071f`, the smoke suite on the default
  core, `harness/make-reference.sh`. The review reproduced it 3/3 with
  another binary, and the reborn core's `smoke-rust` and `smoke-exact-rust`
  are unchanged.
- **Left:** #636 (`scripts/rerecord-smoke-borg-rs.sh`) makes the re-record
  one command, so a shipped value is changed with its reference.
