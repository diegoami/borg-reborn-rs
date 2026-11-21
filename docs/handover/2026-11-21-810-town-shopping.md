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

The shipped Recall target is one. A purchase must leave 400 gold in the
purse after paying for the scroll. The planner accounts for urgent purchases,
and the executor checks the native price. `borg_param_buy_recall_from_gold`
sets the amount to leave in the purse.

Cross-class measurement of shopping and fuel remains in #810.
