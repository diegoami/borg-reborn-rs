## 2026-11-25: Return to restock healing potions (#1214)

### What was done

Track P proposes a minimum of two known healing potions. Below it, the Borg
can return when the stock it last saw in town and its current gold can
cover the deficit. This covers an empty pack after a never-stocked
departure when the refill becomes affordable. Free Home stock can justify
one visit. Paid retries after a short departure require new funding.
Depleting a previously stocked reserve still requests a return. The funded
check follows the flask resupply rule. [#1184](https://github.com/diegoami/borg-reborn-rs/issues/1184)

The return survives level changes, tries a reachable up stair first, and
uses Recall when the exposure check permits it. Both town shoppers can
refill the reserve. A failed trip defers ordinary cure checks through DL35.
The deeper Heal and star-healing checks remain independent.
[#1212 stair routing](https://github.com/diegoami/borg-reborn-rs/pull/1212)

### What was found

The native cases cover one potion remaining, depletion, a blocked stair,
insufficient gold, and shopping through both strategies. The review added
unit coverage of first funding, free Home offers, failed visits, and the
deep-level restock checks. The core trace records the last observed known
potion count in both arms of the comparison. In smoke seed 101, the
Warrior earned enough for its first reserve and returned at turn 13,103.
The game ended alive at CL2 and maximum DL1. The previous reference
reached CL4 and maximum DL3. The other three games matched.
[Original Ranger case](https://github.com/diegoami/borg-cemetery/tree/d7d90fbc40275da3d06af57bea22c6b3d1a16ca0/incidents/2026-11-25-watch-half-orc-ranger-poison-37450)

### What is left

The paired population comparison awaits dispatch. [Zoo #791](https://github.com/diegoami/borg-zoo/issues/791)

### Owner decisions

The draft keeps the switch enabled with a two-potion minimum. The owner
can confirm that value after the Zoo results and decide when review is
complete. The paid retry guard and preservation of the deeper healing
checks are part of the proposed behavior.
