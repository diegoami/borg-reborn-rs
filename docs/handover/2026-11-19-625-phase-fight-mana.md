## 2026-11-19: the Phase Door rule's fight counts mana and fail rates (#625)

Owner, 2026-11-19: the "can I win" test (#584, PR #612) should check that a
caster still has the mana, and that its spells are likely enough to work.

**What was done:** `escape_phase_lost_mana` (flag, compiled 1). It acts
only inside the rule (`escape_phase_adjacent_rule`, shipped 0) with
`escape_phase_lost_test` 1 or 2.
- The ranking notes each candidate's cost (`FightAttackState.cost`,
  `AttackCost`):
  - a spell when it passes `borg_spell_okay_fail` (`spell_okay_fail_noted`):
    fail rate, and casts = current SP / its mana;
  - a wand: `borg_activate_failure` / 10, and its charges;
  - a rod: the failure only.

  `borg_attack_estimate` keeps the winner's cost and the best true damage
  of a candidate that is not a spell (`est_cost`, `est_fallback`).
- The fight test (`fight_turns`, `Primary`) kills the adjacent foes one
  after another. While uses are left, it uses the chosen attack at true
  damage × success chance, if that beats melee or the fallback; then the
  larger of melee and the fallback. Fractions of a use carry over foes.
- The rule's note gives both numbers:
  `attack 6 at 13% fail (1 use), then 1; danger 12, HP 14: lost`.
- 0 = #612's test (full damage every turn).

**Checked:**
- CI's commands, `check-param-citations.py` (2170), and `borg-core-vendor.py check`.
- New unit test `the_fight_counts_the_mana_and_the_fail_rate`.
- Rule off: the 18 classes-18 starts play keys identical to main `3ba17e2c`. Recording the cost draws no random numbers, and the wand and rod failure is still computed once.
- Rule on, test 1, 18 starts, mana 0 against 1:
  - Deaths 1 and 1, the same rock lizard; sum of max CL 93 and 94.
  - Only the two casters' games change (Mage 1548306106, Druid 934061548).
  - Fight verdicts: 155 lost both ways, not lost 64 → 58; Phase Doors 81 → 89.

**Left:**
- Staves and activations count at full damage, with no limit.
- A rod's recharge is not modelled.
- A zoo arm on top of the borg-zoo#347 pilot, if the owner wants one.
