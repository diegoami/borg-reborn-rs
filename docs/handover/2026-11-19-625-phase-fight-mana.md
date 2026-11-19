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
  - a rod: the failure, and one use (it recharges over many turns).

  `borg_attack_estimate` keeps the winner's cost and the best true damage
  of another candidate that needs no mana, never the winner itself
  (`est_cost`, `est_fallback`, `fallback_without`).
- The fight test (`fight_turns`, `Primary`) kills the adjacent foes one
  after another. While uses are left, it uses the chosen attack at true
  damage × success chance, if that beats melee or the fallback; then the
  larger of melee and the fallback. Fractions of a use carry over foes.
- The rule's note gives both numbers:
  `attack 6 at 13% fail (1 use), then 1; danger 12, HP 14: lost`.
- 0 = #612's test (full damage every turn).

**Checked:**
- CI's commands, `check-param-citations.py` (2195, after merging main `826cde4c`), and `borg-core-vendor.py check`.
- New unit test `the_fight_counts_the_mana_and_the_fail_rate`.
- Rule off: the 18 classes-18 starts play keys identical to main (`3ba17e2c`, `e381e6b9`, and again after the final merge). Recording the cost draws no random numbers, and the wand and rod failure is still computed once.
- Rule on, test 1, 18 starts, mana 0 against 1:
  - Deaths 1 and 1, the same rock lizard; sum of max CL 93 and 94.
  - Only the two casters' games change (Mage 1548306106, Druid 934061548).
  - Fight verdicts: 155 lost both ways, not lost 64 → 58; Phase Doors 81 → 89.

**Reviews:**
- **First review** (`b1528ed8`):
  - the fallback excludes the winner;
  - a rod counts one use;
  - the reserve cast counts the real mana;
  - Whirlwind Attack, Maim Foe and Vampire Strike report their true damage;
  - the expected-melee/nominal-missile mismatch is documented;
  - `docs/PHASE-DOOR.md` updated.
- **Owner:** all the mana counts for fighting ("at the beginning we have no choice, and later on we will get potions to recover it"). The follow-up for potions is #679.
- **Re-review** (`7ded5d62`):
  - Whirlwind's true damage is now the best single foe's, since the fight test applies it one foe at a time; the sum stays the ranking's. Its kills (`kill_best`, #556) count each foe it sweeps.
  - `harness/borg.txt`'s comment now describes the fallback as it works.
  - This note is updated, and `main` is merged.

**Left:**
- Staves and activations count at full damage, with no limit.
- A rod counts one use, not its exact recharge time.
- Potions of Restore Mana in the pack are not counted (#679).
- A zoo arm, if the owner wants one.
