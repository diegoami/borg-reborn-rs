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

**Shipped 1** (owner, 2026-11-20: "ship at 1"): smoke-borg-rs is still 3/3
identical (`scripts/rerecord-smoke-borg-rs.sh`: nothing to re-record). Mode 2
(the other two boosts) is not shipped. A widening (for example a larger
cornered radius, so that "closing in" happens more often) would be a
separate change.

**The four deaths of starts 1–90** (the same games in mode 0 and 2, none
involving the rule): two Druids to Fang (CL1–2, meleeing with a 1d3 whip),
a Mage to a crow during a scary-level flee (#689's case), a Druid to a wild
cat while every Phase Door was refused and Word of Recall was read.
