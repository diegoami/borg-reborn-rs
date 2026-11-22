## 2026-11-22: Attack on sight prices poison and drain blows (#1010)

Branch `task/T1010-price-status-blows`, from `track-p` `6bc4de1d`. Track P,
P8 (owner: "start working on those issues"). A follow-up of #878.

**Done.** `caution_attack_price_status_blows` (flag, compiled and shipped 0;
registry 2385 to 2386). Attack on sight's target test prices POISON
(dice plus `10 / energy` HP a Borg turn for its timer, none if resisted),
LOSE_*, EXP_*, EAT_*, DISENCHANT and DRAIN_CHARGES (their dice), instead of
refusing the target as unknown damage; PARALYZE, CONFUSE, BLIND, TERRIFY,
HALLU, SHATTER, BLACK_BREATH and unknown spells still refuse it. Every
handler cited is checked in mon-blows.c (`monster_damage_target`). New field
`IncomingDamage::unpriced_damage`; `status_blow_priced`,
`poison_hp_per_turn`, with unit tests.

**Scope, narrower than the issue's wording.** The issue named the winning
fight too. The winning fight and the town hunt also refuse any non-HURT blow
through `unknown_fight` (the #759 certificate, which ACID and FIRE already
fail), so pricing alone cannot change them; the switch is read only by
`category_attack_safe`. The Recall forecast keeps #880's own model.

**Checked.** CI's commands (see the PR); smoke at the shipped settings 3/3
identical to `smoke-borg-rs`. Named case, direction only: the Mage 461220874
save at turn 15037, this branch, shipped settings, 100,000 turns:

| arm | white worm mass attacked on sight | worms destroyed | end |
|---|---:|---:|---|
| shipped | 0 | 1 | dead at 74323 (a grey mushroom patch it attacks on sight, #969) |
| `caution_attack_price_status_blows=1` | 8 (first at 37559) | 4 | alive at the cap, CL7 |

One case, not a result (RULES 14).

**Not done / left.**
- Several poisoners each add the timer's HP (`poison_hp_per_turn` per
  monster), an over-count: the timer costs one HP a tick however many.
- The winning fight's certificate for non-HURT blows: a separate question
  (would need its own switch on `unknown_fight`).
- **Measurement:** a borg-zoo arm with this switch, and with it plus
  #1009's `caution_breeder_danger_damage_only`; deaths to worm masses and
  poison. Not filed (RULES 20).

### After the review (PR #1015, pinned `f76d6ea6`)

- F1: synchronized with `track-p` `72651d25` (citation conflicts resolved by
  keeping this PR's side, then `check-param-citations.py --fix`; registry
  2397 to 2398). The new parameter's citation is the read line again (2876;
  the lenient checker had accepted the function's first line).
- F2: the winning-fight half of #1010 is filed as #1039 (with #1011's
  observation: an all-HURT breeder is still a winning fight on a lost
  level). `Closes #1010` stays: #1010's attack-on-sight half is done here,
  the rest has its own number.
- F3: `the_target_test_reads_the_price_switch_before_its_default` pins the
  wiring in `category_attack_safe_with` (a source test: the function needs
  the engine); removing the switch's block fails it (checked).
- F4: `poison_hp_per_turn` and #880's `recall_poison_hp` now name each other:
  the same 1 HP a world tick, as a fight's rate and as the Recall window's
  bounded total.
