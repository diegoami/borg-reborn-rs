# Expose preparedness and restock settings

The [descent and restock bands](../CALIBRATION.md#configurable-preparedness-and-restock-bands-1283)
in `harness/borg.txt` define the shared thresholds. The ordinary
`__<class>` suffix applies to both thresholds and starting depths. Bands
remain cumulative. The DL10 light-radius overrides retain the
Necromancer's exemption in preparedness and restock. Deep healing stock
is configurable through the home, total and collection gates.

The inherited character-level values are the control for
[#1283](https://github.com/diegoami/borg-reborn-rs/issues/1283). The
regressions cover a class-specific depth and restock threshold, a Recall
reserve above and below DL5, the Ranger CL29 boundary, and the Mage and
Necromancer ramp. The existing Ranger offset starts at CL29. The caster
ramp starts at CL39 with a depth cap of 32.

Higher character-level defaults still need the paired all-class Zoo
comparison in #1283. The actual-destination town Recall gate is tracked
in [#1281](https://github.com/diegoami/borg-reborn-rs/issues/1281).
