## 2026-11-24: Throw known surplus weapons (#1200)

Track P, P9. Branch `task/T1200-surplus-throwing` targets `track-p`.
The named build is `9c8024ea4e7e82d9683609a62bacef43a4cd5a67`, based on
`e34d04b788a5f6a9b600268231cfe6c4e260e820`. It runs the shipped borg-rs core.

- `attack_throw_surplus` is compiled and shipped at 0. The enabled arm
  considers known plain low-value throwing weapons from the pack.
  It excludes inscribed gear, selected swaps, unknown runes, curses,
  egos, artifacts, abilities and upgrades. A heavy equipped weapon
  cannot establish that a lighter alternative is surplus.
- The ordinary and focused attack planners use existing throw damage,
  hit chance and the normal competition with other attacks. Candidate
  range comes from Angband's strength and object-weight calculation.
  Execution reuses the existing switch from the compact throw list to
  inventory. No change to town or dungeon acquisition.
- The named Mage and Priest replays verify the actual selected projectile,
  range, damage or miss, the immediate pack, stronger competing attacks
  and protected gear. All 15 runs reproduced keys, messages, state and
  core traces. [Cemetery case](https://github.com/diegoami/borg-cemetery/tree/7204baf199b32977aa424013fcd3b58c18b304d1/scenarios/surplus-throwing-1200)
- Formatter, workspace Clippy and tests, release borg-core tests,
  parameter citation check and vendor check passed. Four shipped
  smoke runs matched their references.

[Zoo #782](https://github.com/diegoami/borg-zoo/issues/782) is ready for
owner dispatch. Keep the flag off until the population evidence and P9
gate support an activation decision. The implementation PR requires
green CI and owner acceptance.

The separate #1169 investigation and proposal are in the
[Cemetery rune backlog case](https://github.com/diegoami/borg-cemetery/tree/c59ef17b7d6453cd827fd1a2a968b55af4d54439/scenarios/caster-rune-backlog-1169).
