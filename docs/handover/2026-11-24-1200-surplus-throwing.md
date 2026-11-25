## 2026-11-24: Throw known surplus weapons (#1200)

Track P, P9. Branch `task/T1200-surplus-throwing` targets `track-p`.
The named build is `9c8024ea4e7e82d9683609a62bacef43a4cd5a67`, based on
`e34d04b788a5f6a9b600268231cfe6c4e260e820`. It runs the shipped borg-rs core.

- `attack_throw_surplus` is compiled at 0. The enabled arm
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

[Zoo #782](https://github.com/diegoami/borg-zoo/issues/782) tracks the
population comparison. The implementation PR requires green CI and owner
acceptance.

The separate #1169 investigation and proposal are in the
[Cemetery rune backlog case](https://github.com/diegoami/borg-cemetery/tree/c59ef17b7d6453cd827fd1a2a968b55af4d54439/scenarios/caster-rune-backlog-1169).

### Review follow-up

The branch incorporates Track P `20ac7d1903b9a018d0c81fe67d5441a36b483062`.
The enabled path uses native range and the same damage and hit model for
both ordinary and surplus throws. `attack_throw_surplus_max_value`
sets the separate 100-gold cap. The original model remains in use
when the new flag is off.

The synced build `31ca4dfc4d633d091141505d13436fe750c37f37` matched all
four current smoke references. All 15 named replays reproduced keys,
messages, state and decision traces. In the added launcherless Mage
control at nine tiles, the enabled arm threw oil with
`attack_throw_as_melee=0`, and the disabled arm retained its historical
six-tile aim limit. The formatter, Clippy, both Rust test suites and
parameter citations passed.
[Pinned evidence](https://github.com/diegoami/borg-cemetery/tree/cd36af16fa5b6660a073cf2aeb541da0941ddb2c/scenarios/surplus-throwing-1200/review-followup)

Zoo #782 uses eligible-start and encounter counts as the pilot endpoints.
Survival remains a guardrail. The owner dispatches the pilot.

### Shipped value

The owner chose `attack_throw_surplus=1` for all classes in the shared
profile. The cap is 100 gold. Keep `0` available as the Zoo control.
