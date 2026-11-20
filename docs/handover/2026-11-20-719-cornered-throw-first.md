## 2026-11-20: cornered, attack at range before the boost (#719)

Owner: "if I am cornered I should throw missile and then boost myself".

- **Code:** `borg-core/src/borg_caution.rs`.
  - `throw_first` runs at the top of `cornered_boost`. With
    `caution_cornered_throw_first` at 1 (the cornered boost) or 2 (also the
    breeder-fight and Phase-rule boosts), when no monster is adjacent and the
    attack planner's dry run (`borg_attack_estimate`) would do damage, it
    calls `borg_attack` and notes "attacking at range before the boost
    (#719)".
  - The dry run's random numbers and the waiting flag are restored, as the
    breeder estimate does. Without that, the first version changed 1–2
    classes-18 games without ever acting.
  - `throw_first_applies` is the order decision, unit-tested.
- **Shipped 0:** smoke-borg-rs 3/3 identical; classes-18 18/18 identical at 0,
  1 and 2.
- **Measured** (100k turns), modes 0 / 2:
  - starts 1–90: deaths 4 / 4, CL 517 / 518, DL 206 / 206, 89 of 90 games
    identical;
  - the rule acted 4 times in 3 games.

  It's rarely reached: a cornered Borg (a monster within 2 grids) almost
  always has the monster already adjacent.
- **Checked:** fmt, clippy `-D warnings`, both test commands, citations.

**Owner's decision:** shipping it at 1 (or 2) as the preferred behaviour; it
can't be harmful at this rate, and the zoo can't measure a rule this rare at
normal tiers. A widening (for example a larger cornered radius, so that
"closing in" happens more often) would be a separate change.
