# Expose preparedness and restock settings

The [descent and restock bands](../CALIBRATION.md#configurable-preparedness-and-restock-bands-1283)
in `harness/borg.txt` define the shared thresholds. The ordinary
`__<class>` suffix applies to both thresholds and starting depths. Bands
remain cumulative. The DL10 light-radius overrides retain the
Necromancer's exemption in preparedness and restock. Deep healing stock
is configurable through the home, total and collection gates.

The owner approved the stricter DL10+ settings in #1287. The shipped
DL21 offsets and upper bounds rise by two CL for all nine classes, and
Ranger now checks the offset below CL29 as well. Mage and Necromancer
enter the ramp at CL41. The shared DL34 floor is CL42. The linked
calibration table records the kit and restock increases.

The regression tests cover the live shipped Ranger and caster boundaries.
The stricter thresholds are unmeasured hypotheses. The paired DL21+
comparison requested in #1283 remains open. The review restored the
auxiliary Morgoth kit checks at their inherited thresholds. The live
static entry point still calls the auxiliary bands only through DL99.
Town Recall checks the actual destination at the shipped 100% setting
from [#1284](https://github.com/diegoami/borg-reborn-rs/pull/1284).
