## #1217 · Shallow preparedness by class

The folded `harness/borg.txt` sets DL2 one level above each class's mixed
exploration cap. The minimum rises at DL3 by one level for Warrior, two
for Ranger, Paladin, Rogue and Blackguard, and three for the other classes.
It rises by one again at DL5 during a dungeon trip. The three shipped
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

Issue #1217 tracks the population comparison against the previous
shallow defaults before adoption.
