## 2026-11-19: projectiles and devices weighed by class (#666)

Owner: "weighing buying projectile and magic by class"; by shooting and by device skill; "projectile buying is more important if they cannot get flasks of oil or wands"; "I would make them buy it mostly when they cannot find flasks of oil at all"; ship only the fix and the device weights.

- **Shipped:**
  - **`town_devices_percent`** (compiled 100): a wand, rod or staff is worth that percent of what it supplies. The class files set it by `class.txt`'s device skill, with 24 = 100: Mage and Necromancer 150, Rogue 135, Priest 125, Ranger 115, Druid, Paladin and Blackguard 100, Warrior 75.
- **Compiled at today's play:**
  - **`town_missiles_percent`** (-1 = the Ranger's reserve only). Any class can have a reserve of shots for its launcher, in percent of `borg_town_w_ranger_missiles`.
  - **`town_missiles_other_ranged_percent`** (100). With 5 or more flasks of oil and attack-wand charges in the pack, the shots weigh that percent.
- **Measured and not shipped:**
  - **Shots by shooting skill** (Rogue 90, Warrior 75, Paladin 70, Druid 55, Priest and Blackguard 50, Mage and Necromancer 30; Ranger 72 = 100). Over classes-18, ammunition gold went 2,200 → 2,916, healing bought 54 → 51, Phase Door 45 → 40, sum of CL 97 → 95, max DL 37 → 34. Shots at 10% with flasks did not hold it back: the flasks are often used up by the time the Borg is in town.
  - **A first-launcher boost:** it changed nothing, and it was removed.
- **Why no launcher is bought: not known.** A first-launcher boost of 2,000 or 8,000 power changed no game.
  - My first explanation was wrong. `borg_town_share` is not applied in play: `caution_allows` is dead code outside tests (`borg_town_usefulness.rs`).
  - Flasks of oil are always stocked in the General Store (`store.txt`, `always:flask:Flask of Oil`).
  - A wand is bought when the purse allows: smoke's priest-3 buys a 330-gold Wand of Magic Missile.
  - Follow-up issue to investigate.
- **Not a fix (review of #668):** the town plan already values ammunition only with a launcher worn. `BI_AMMO_TVAL` is set only from the bow slot and reset at every `borg_notice`. A `launcher_worn` guard added at first could never fire, so it was removed with its claim.
- **Results:**
  - classes-18: 18 of 18 games are key-for-key main's.
  - Smoke: priest-3 buys a Wand of Magic Missile at turn 15,156 (device weight 125). `smoke-borg-rs` re-recorded.
- **Left:**
  - Why no launcher is bought: #669.
  - Shots per class, and `borg_power`'s missile reward that fights them: #676.
