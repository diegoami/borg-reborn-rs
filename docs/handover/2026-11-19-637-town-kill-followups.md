## 2026-11-19: kill on sight in town, the three follow-ups (#637)

From the review of #624 (owner: "file an issue for 1-2-3", then "do 637").

- **Three switches, each 0 (= #619):** `attack_town_dangerous_caster_walk`
  (a low-level Mage or Necromancer walks to the four too),
  `attack_town_dangerous_wand` (a wand in town, only at the four),
  `attack_town_dangerous_bonus_whole` (the bonus on the danger term too).
- **Found:** C's wand paranoia roll draws the Borg's own RNG, carried across
  thinks; rolled with no wand it changed 17 of 18 games. In town the wand is
  now checked first.
- **On 18 classes-18 starts with kill on sight:** each switch alone changes
  nothing. With the town never scary too, all three change 2 Necromancer
  games, same levels, deaths 4 and 4.
- **With #622's N raised (owner):** N = 6, the three change nothing; N = 11
  (no list scare left in town), they change 5 games: sum of CL 94 to 98,
  kills of the four 22 to 27, thefts 15 to 13, deaths 1 and 1.
- **Review of bacaf463:** the caster walk reached only CL1-19 (the town
  gate); now CL1-24, with a test. The wand counts only a dangerous monster
  seen within `attack_max_target_age`. The RNG wording corrected. Left as
  is: `town_wand` is a field set around one call (the reviewer checked it
  cannot leak).
- **For the owner:** nothing to ship; keep the switches at 0 for a later
  arm, or close #637 as measured.
