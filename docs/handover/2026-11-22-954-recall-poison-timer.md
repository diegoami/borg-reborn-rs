## 2026-11-22: Recall forecast charges a running poison timer under resistance (#954)

**Done.** `recall_poison_hp` (borg-core/src/borg_caution.rs) checked poison
resistance before the running timer, so a Borg already poisoned with
resistance on was charged nothing for the Recall delay. The engine stops
only new poison with resistance (`player_inc_check`, player-timed.c), and a
running timer keeps costing 1 HP per world tick (game-world.c:513-515). The
running timer is now charged first; resistance still blocks new poison.
The unit test that expected zero now expects the whole delay, and a new
case keeps "resistance blocks new poison". Parameter citations moved with
the lines (`scripts/check-param-citations.py --fix`).

**Found.** Codex's P7 review on #939 (finding P1). Shipped play changes
only for a Borg that is poisoned and resists poison while it weighs a
flight Recall (`caution_recall_blow_model = 1`): the forecast now counts
the poison.

**Left.** Nothing for this finding. The other review findings are #955-#958.
