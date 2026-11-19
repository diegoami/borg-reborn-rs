## 2026-11-19: dangerous town monsters killed on sight (#619)

Owner: kill the dangerous town monsters on sight with the best attack, and
test it together with the town staying scary until a level that depends on
the monster.

- **`attack_town_dangerous_on_sight`** (compiled 0, written out at 0),
  `attack_town_dangerous_min_damage` (6), `attack_town_dangerous_bonus_percent`
  (300). In town, a monster that steals or hurts for at least 6 is the
  preferred target and the hunt walks to it. The scare is left to the scary
  list.
- **On 18 classes-18 starts:** neutral with the shipped list (0 deaths, 99
  CL, 2 games of 18 change). It costs deaths once the scare is removed
  (town never scary: 3 deaths, 89 CL). With #622's levels less, N = 3: 0
  deaths, 101 CL. Thefts are not reduced.
- **First review (e389bcdb), non-blocking, now in the docs:** a Mage or
  Necromancer below CL25 never walks to them (its town hunt is breeders
  only), though it still shoots them; no wands in town (C); the preference
  is measured by kill counts, not checked per target choice (the bonus
  multiplies damage, not `melee_score`'s danger term).
- **Resynced with main after #621 (ignore list) and #595:** the ignore check
  runs before the bonus (an ignored monster stays at 0; none of the four is
  ignored); the helpers moved above `borg_thrust_damage_one`'s doc comment;
  the count is 2163. The figures above are from before both, not re-measured.
- **Left:** zoo arms with #622 (borg-zoo#328 after the merges); #620 (Druid
  Fox Form in town).
