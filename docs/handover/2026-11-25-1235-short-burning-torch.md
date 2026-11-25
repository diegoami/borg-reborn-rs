## 2026-11-25: Use short-burning replacement torches first (#1235)

Routine maintenance, recovery in the dark, and harmful-light replacement
now prefer the clean torch with the least fuel among equal-radius torches.
An ordinary equipment upgrade can use fuel to choose between two pack
torches after one has passed the existing power and danger checks. The
town equipment plan remembers the chosen radius and looks up that torch
in the pack when issuing the wield command.

While a torch is still equipped, maintenance uses the existing
aware-spare check at 250 turns to decide whether fuel is needed. The
clean-fuel requirement applies when choosing a torch to wear, including
town plans that can decurse other equipment. With an empty light slot,
maintenance prefers a clean powered lantern, then a clean burning
torch, then a clean drained lantern if refuel stock is available. If
these options are unavailable, maintenance reports an unmet light need
and caution can start leaving for fuel.

The regression reconstructs the recorded 73-, 3,000-, and 5,000-turn
spares in two pack orders. It checks the first two replacements, a brighter
torch, spent and known cursed spares, and a working torch down to its last
turn. It also covers the low-fuel departure check, the lantern fallback,
and the equipment-upgrade tie-break. The fuel comparison uses the
timeout of one wielded copy, including when the spare is a stack.

**Remaining work.** The integration decision stays with #1235. Population
effects are unmeasured. The recorded inventory was reconstructed in the
tests, and the original native recording has not been replayed.

**Owner decision.** Accept the updated PR for Track P integration. A
population comparison can be requested in borg-zoo if the owner wants one.
