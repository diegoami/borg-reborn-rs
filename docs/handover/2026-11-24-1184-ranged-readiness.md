# Ranged readiness for dungeon trips (#1184)

Track P, based on `3c46575d`. The owner asked for a ranged capability
throughout the game after the Paladin ran out of oil and later equipped an
empty bow in the dungeon.

- The shared assessment counts compatible launcher ammunition, throwable
  supply after lantern fuel, sufficiently damaging and reliable attack
  spells and devices, and equipped activations. A depth-relative damage
  floor removes weak options from deeper trip estimates. Recharge and mana
  recovery are distinguished from a missing attack.
- `borg_param_town_ranged_readiness=1` prices the ranged reserve after
  urgent supplies and gives a working launcher with no class missile
  reserve a small matching-ammo target. `borg_param_trip_ranged_readiness=1`
  prioritizes reachable floor supplies, protects a short reserve from
  routine discards, and checks whether a gear swap loses the last ranged
  source. A return needs an affordable shopping opportunity from the last
  observed stock and a route by stairs or Recall. Emergency movement has
  priority. A failed resupply waits for a change in gold or launcher ammo
  type before it can trigger another return.
- Both switches ship at 0. `borg_param_ranged_readiness_trace=1` records
  changes in the assessment for all four Zoo arms in
  `ranged-readiness.jsonl` under the run directory. It can be collected on
  the control with both policies off.

## Checks

- Four shipped smoke runs matched keys, messages, and state. Formatter,
  workspace Clippy, Rust tests, and parameter citations passed.
- In the constructed Short Bow case, the shopping arm bought six arrows
  for six gold before its oil purchase. The original pinned run with the
  same carried bow, purse and seed left with zero arrows. The Ranger
  check bought arrows and kept a usable launcher.
- On named Paladin seed 302, the four arms reached the 100,000-turn cap
  alive. Control, shopping, recovery, and both spent 49.8%, 78.1%, 41.4%,
  and 42.9% of sampled dungeon time without a usable attack. The control
  reproduced the earlier key stream with diagnostics enabled. These
  figures describe a single start.
- The saved CL18 Mage at DL6 and the constructed CL45 Mage at DL45 reached
  their named continuation caps alive with both arms enabled. These
  exercise attacks that remain available as mana and gear change. Both
  fixtures are held in the [Cemetery](https://github.com/diegoami/borg-cemetery/tree/9715e9569eed252fda7cb9f93fc46e1a57492802/scenarios).

## Remaining measurement

The paired Zoo pilot needs early seeds and the saved mid- and late-game
bank. Compare turns without a ranged attack, time to restore one, and the
cost in failed or repeated town trips, survival, and depth. The damage
floor and six-use target are pilot values. Existing reserves and combat
spending behavior are the comparison baseline.
