## 2026-11-25: Druid and Necromancer stair-scum slack (#1233)

The Druid and Necromancer profiles set `stair_scum_slack_steps` to 2, matching
Mage and Priest. The overrides are folded into the shipped `harness/borg.txt`.

The default borg-rs core reported 2 in both classes' resolved settings during
short headless runs. All four smoke cases matched the existing reference for
keys, messages, and state. The class-file check, parameter citations, and Rust
checks passed.
