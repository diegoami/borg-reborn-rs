## 2026-11-24: Ranged readiness for dungeon trips (#1184)

### What was done

Track P adds separate, opt-in shopping and dungeon policies. The shared
assessment counts usable matching ammunition, throwable oil after lantern
fuel, and sufficiently strong spells, devices and equipped activations at
the relevant depth. Temporary recharge or mana limits are distinguishable
from an exhausted kit.

- `borg_param_town_ranged_readiness=1` values a useful reserve in the
  existing town budget and gives an empty launcher a matching-ammo target.
  The configured `town_ranged_reserve` is the lower bound for the shared
  reserve. Existing urgent cure and Phase Door rules remain in effect.
- `borg_param_trip_ranged_readiness=1` favors available floor supplies,
  protects a short reserve during routine discards and gear changes, and
  seeks a return if known ordinary-store stock, the purse and the route
  support it. The return uses an up stair underfoot before checking
  Recall, whose sources include a rod, activation, spell and scroll.
  Emergency movement retains priority.
- The owner enabled both behaviors and the trace in Track P at a six-use
  target and one point of average damage per two levels, with a floor of one.
  The trace also records readiness transitions for the Zoo controls.

### What was found

The watched Paladin reached town at 22,741 with three oil flasks,
417 gold, 13 of 23 pack slots used, a torch and no launcher. The recorded
inventory showed 45.7 lb carried and 74.2 lb of weight headroom. It bought
four Detect Invisible scrolls for 60 gold at 23,031. Scary-monster flight
interrupted both General Store approaches, so the visit provides no oil
offer, price or valuation from that store. Oil ran out at 24,707. One
floor flask at 57,300 was spent at 58,641. The trace through 156,423
shows no town-return or rising goal after the initial depletion.

The same life eventually bought 17 flasks for 51 gold at 178,447 and
used the last at 201,492. It equipped the Short Bow at 204,994 and
found its first 27 arrows on DL4 at 336,602. The next town arrival was
354,811, after the bow had become usable. The full recording contains no
arrow purchase. [Cemetery investigation and reproduced fixtures](https://github.com/diegoami/borg-cemetery/tree/ff8ab7efbece3cc86eec632dc2b34e6a721cb825/validation/borg-reborn-rs/ranged-readiness-1184)

On the pinned baseline, a naturally born torch Paladin did not gain a
restock goal from depletion. An independent up-stair rule brought it to
town. With sufficient gold it bought 22 flasks and left with them. A
Ranger with a working launcher bought 14 arrows. The low-purse lantern
case spent three of five flasks on its fuel allocation and retained two
to throw.

After the review changes, the constructed empty-bow Paladin bought six
arrows for six gold before oil. A Ranger bought and retained matching ammo.
The constructed lantern Paladin left its first visit with 33 flasks,
including three reserved for fuel. In all four local seed-302 Paladin
arms, the Borg survived to the cap. The share of dungeon time without a
usable ranged attack was 49.8% control, 56.4% shopping, 41.4% recovery and
0.2% both. This is one start. The saved start at DL6 and constructed
start at DL45 also reached their respective Mage continuation caps.
[DL6 save](https://github.com/diegoami/borg-cemetery/tree/9715e9569eed252fda7cb9f93fc46e1a57492802/scenarios/middle-inventory-mage-9421204) ·
[DL45 save](https://github.com/diegoami/borg-cemetery/tree/9715e9569eed252fda7cb9f93fc46e1a57492802/scenarios/late-inventory-mage-10815143)

The seven seeded runs reproduced keys, messages, state and the readiness
trace. The earlier neutral build matched all four smoke references. Formatter,
workspace Clippy, Rust tests and parameter citations passed.

### What is left

[Zoo #778](https://github.com/diegoami/borg-zoo/issues/778) tracks the
paired pilot across early seeds and saved mid- and late-game starts. It
will measure coverage, time to restore it, survival, depth and the cost
of repeated or unproductive returns. The local checks do not give
population coverage for Recall-only routes or equipment swaps.

### Owner to decide

Dispatch the pinned Zoo comparison under the repository's owner workflow.
The pilot will measure how the enabled policies affect coverage, time in
town, survival, and achieved depth across starts.
