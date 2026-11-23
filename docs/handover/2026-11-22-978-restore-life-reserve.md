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

## Review follow-up

The junk-quaff guard applies with the reserve off. It changes the shipped
decision when healthy full-pack junking selects a known Restore Life Levels.
A healthy Borg can still sell or discard every copy at reserve zero. The
existing level-50 dungeon recovery exception still applies.

Ordinary junking and town dispositions may remove surplus copies. A
full-pack clear must free a slot, so it protects an entire stack when
dropping it would breach the reserve. The desperate second pass can give
up that stack to free a slot.

The [Zoo measurement request](https://github.com/diegoami/borg-zoo/issues/687)
separates the guard from the one-copy reserve. After measurement, the owner
can decide whether to ship retention and calibrate the maximum-depth gate
(currently 12), reserve size, restore weight and spell failure ceiling
(currently 20%). The reserve remains off pending that decision.

PR #996 also appends capability row 76. Whichever PR integrates second
must sync track-p, give the incoming capability the next index, and
rerun the replay, smoke and review checks.
