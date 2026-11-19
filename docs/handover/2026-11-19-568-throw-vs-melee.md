## 2026-11-19: throws rated on melee's footing (#568)

From #539 (`priest-809881907`): cornered by Grip with 19 Flasks of oil, the Borg meleed (score 44) instead of throwing oil (15); the owner, from the 1,721 save, threw one flask and Grip fled.

- **Why the throw lost:** in `borg_fight_attack.rs` a throw counted its dice average (a flask's 1d4: 2; the engine does 22), used the bow skill, the launcher's to-hit and multiplier, and got the bolt's danger term (1 turn, partial damage) where melee gets 2 turns at full damage. The last is melee's bigger danger bonus.
- **Change:** `attack_throw_as_melee` (flag, compiled and shipped 0 = C). With 1: the engine's throw damage (`thrown_damage`) ranks the objects and is the attack's damage; the engine's hit chance with the throwing skill (`throw_hit_percent`) scales it; the danger term is melee's (`FightAttackState::throwing` marks the rating). `best_throw`'s gates keep the dice average. Unit tests: `throw_tests`.
- **Measured** (CALIBRATION, #568): switch off identical to main on 19 of 19 games (classes-18 plus the Priest seed) and on smoke. Switch on: 17 of 19 differ, flasks thrown 456 to 630, deaths 1 to 2 (an unseen floating eye at CL2; no attack choice involved), sum of max CL 110 to 104. In the owner's scene, at game turn 1,881 with Grip adjacent, main melees (44), the switch throws (80); both survive.
- **For the owner:** whether to ship 1 (a zoo pilot first, as for #646). The point-blank divisor (an adjacent non-unique: damage / 5 for most classes) still keeps flasks for distance; left as it is.
- **Related:** #649 (PR #686) values flasks in the town plan at the same engine damage; together the Borg buys flasks and then uses them.
