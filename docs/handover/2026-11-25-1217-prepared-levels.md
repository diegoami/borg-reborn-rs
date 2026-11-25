## 2026-11-25: Shallow preparedness by class (#1217)

The folded `harness/borg.txt` sets DL2 one level above each class's mixed
exploration cap. The owner set the DL3 minimum to 11 for Mage, 10 for
Druid, 12 for Priest, 10 for Paladin, 8 for Rogue and 9 for Ranger. DL5
uses its own class minimum during a dungeon trip. The three shipped
shallow max-HP minimums are zero. The new DL2 minimum defaults to zero in
older profiles, and their compiled HP and CL defaults are preserved.

The [calibration table](../CALIBRATION.md#c-borg-dive-gates-768) lists the
exact thresholds. All prepared settings resolve from the shipped file in
all nine classes. The depth margin, cures, Recall and mid- and deep-depth
gates retain their prior values for this first measurement.

The Ranger's CL4 / 37 HP state now fails the shallow DL2 check with
`prepared_dl2_min_clevel` (4 < 8). The fixture supplies light, food, cures
and Recall so it can also check the level immediately below and exactly
at each CL boundary through DL5. The
[watched incident](https://github.com/diegoami/borg-cemetery/tree/d7d90fbc40275da3d06af57bea22c6b3d1a16ca0/incidents/2026-11-25-watch-half-orc-ranger-poison-37450)
provides the character state for the fixture.

The [review](https://github.com/diegoami/borg-reborn-rs/pull/1219#issuecomment-7148517918)
found four feature-gated stair fixtures that need the DL2 level minimum
disabled explicitly. The common and Warrior settings are both pinned in
those fixtures. DL2 proposals report their own CL failure, and deeper
proposals follow the inherited check order.

The [re-review](https://github.com/diegoami/borg-reborn-rs/pull/1219#issuecomment-7148970465)
identified a duplicate vein-loop test class. The retained fixture pins the
incident's breadth-first exploration and stair-scumming settings. The
seven older deep-gate citations now point at the parameter reads.

The Warrior smoke run reaches maximum DL1 by the cap with the proposed
settings and DL3 with the control. The owner approved integration into
Track P. Issue #1217 tracks the full policy decision and the
[Zoo comparison](https://github.com/diegoami/borg-zoo/issues/788).
