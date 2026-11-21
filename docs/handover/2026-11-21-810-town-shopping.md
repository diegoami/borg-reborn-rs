## 2026-11-21: Black Market spending and departure stock (#810)

The shipped planner gives the Black Market 60% of the shopping budget
above the excluded 500 gold. The shop checks that limit against the
current purse before buying. Older configurations and plan traces can
still use the legacy divisor when the percentage is absent.

The town planner limits each non-missile item kind to 35 carried copies
by default. It stashes the excess at Home, including into compatible
stacks, and uses a shop that accepts the item when Home has no room.
It also limits purchases and ordinary Home withdrawals. Dungeon pickups
remain available. The visit can still end early for danger or if the
Borg has no reachable store that accepts the excess.

Cross-class measurement of shopping and fuel remains in #810.
