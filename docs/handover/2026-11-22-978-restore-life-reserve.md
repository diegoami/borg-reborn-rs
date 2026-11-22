## 2026-11-22 — Restore Life Levels retention (#978)

Full-pack junking quaffs a known Restore Life Levels potion only when the
Borg detects experience drain. The existing recovery priority still applies.
The town and dungeon reserve is off by default. With the setting enabled, it
starts at maximum depth 12 and yields to repeatable restoration or a
desperate dungeon pack clear. The town planner prices a current drain
separately and values extra copies at diminishing rates.

The town regression exercises a stocked, affordable Alchemist offer, an
unaffordable offer, an empty offer list and sale versus retention with the
reserve disabled and enabled. The [Cemetery validation](https://github.com/diegoami/borg-cemetery/pull/125)
records the controlled quaff and the XP restored. The original Alchemist
stock is unverified.

Owner acceptance remains the integration gate after green CI on track-p.
The reserve size and weights are hypotheses for measurement.
