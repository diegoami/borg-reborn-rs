## 2026-11-21: Town healing tiers (#845)

The town planner can test `borg_param_town_heal_quaff_bonus_percent`.
It gives larger heals more curve credit as max HP grows. Real HP and
status cure effects retain their existing valuation. The switch defaults
to zero pending evidence from the Zoo.

The #810 recording's source (`d4471de7`) declares healing speed 66, a
real-HP reserve, Ranger reserve 200% of max HP, and Ranger curve scale
100%. The recording's resolved overrides and store offers are still
unverified. In the constructed 108-HP Ranger comparison, all three potions
are below the full-value threshold at speed 66 and 100. Increasing the
speed setting cuts each potion's HP credit by about one third and leaves
the 1:4:9 ratio roughly constant. The real-HP reserve already has 580 of
the required 216 HP and adds no marginal purchase value in that scenario.
CCW brings poison and stun cures as well as healing. Those status curves
have zero weight for this healthy shopper.

At 7,899 gold, with 29 CLW carried and stock of 6 CLW, 4 CSW and 3 CCW
priced at 20, 70 and 200 gold, the planner with no bonus chooses six CLW
at both speed settings. With bonus 200 it chooses three CCW. Both plans
also buy five Phase Door scrolls. The planned spend is 195 versus 675
gold, and raw carried healing rises from 700 to 760 HP. At CL1 with 18 max
HP and 55 gold, both buy two CLW and one Phase Door. With ten CLW in the
pack and CCW absent, the candidate still buys CSW. These are constructed
solver checks with six free slots, no healing at Home and no active
status effects.

Every offered stack is affordable in the late case. CLW joins an existing
stack while CCW opens a slot. At speed 66 the three CCW add 2,896 utility
before space and weight costs, and at speed 100 they add 3,784. Their
combined slot and weight cost is 3,821. The candidate brings the same
three CCW to 4,460 utility before those costs.

[Zoo #597](https://github.com/diegoami/borg-zoo/issues/597) requests
the native pilot at gameplay commit `bba5e4be`. It needs receipts for availability, prices,
pickups and purchases, departure healing and escapes, and quaffs during
danger. The historical X11 run also needs a verified replay or its
original plan records before attributing its stock imbalance to pricing.
