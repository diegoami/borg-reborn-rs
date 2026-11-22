# #1056: a smoke game for the emergency attack spell beyond the mana

**Done.** `harness/suites/smoke` gains `smoke-mage-overexert-4` (owner,
2026-11-22: "change the smoke tests", then "Add a mage smoke game"): Kobold
Mage, seed 592754909, 6,000 game turns, `start-depth=2` with
`start-monsters=kobold:2:4`. With `attack_reserve_spell_overexert = 1` the
Borg casts Magic Missile with no mana at game turn 41, faints and dies to a
kobold at 56; at 0 (shipped, #1057) it kills a kobold and plays all 6,000
turns. Both references are re-recorded with the fourth game: `smoke-borg-rs`
and the reborn core's `smoke-rust` (the reborn Borg dies at 36). The other
three games play the same on both cores.

**Found.**
- No short ordinary game reaches the case. 80 Mage games (seeds 200-279,
  20,000 turns, every race) cast no emergency spell at all with
  overexert 1. Seed 592754909 itself, from the start, is refused by the
  #878 faint check (`attack_reserve_faint_percent`) before overexertion matters.
- Of 27 built scenarios (9 monster sets x 3 seeds), 4 changed with the
  setting; this one is the shortest where 1 dies and 0 lives.
- A `start-depth` game is a synthetic start: its death needs no Cemetery
  record (`export-death.py` `not_natural`), so CI's reconcile passes.
- Local runs export natural deaths into the primary checkout's
  `vendor/borg-cemetery`. I removed the ones my search runs left.

**Left.** The scenario covers one path (Magic Missile from attack on sight);
the zoo still has to measure the setting over many starts (borg-zoo#675).
