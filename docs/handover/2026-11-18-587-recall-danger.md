## 2026-11-18: flight Recall from a danger share (#587)

Owner, after borg-zoo#267: the depth gate on flight Recall (`caution_flee_recall_gate_below_depth` 1/2/3, 918 paired starts up to big) is not shown to change survival, and it leans against, as in #255. Calibrate instead of switch: "Recall only from a certain danger".

- `caution_flee_recall_danger_percent` (0, off in `harness/borg.txt`): where the depth gate or `caution_flee_recall = 0` holds a flight's Recall back, the danger here at or above this percent of the current HP allows it, beside the breeder and chaser exceptions. Only the flight's read uses it (it has the danger at hand); the stand tests keep the old exceptions. The asymmetry is deliberate: the stand tests ask whether an escape from the level exists, the flight asks whether to spend the scroll now, and only the second depends on how pressed the Borg is. If a zoo arm shows the two disagreeing, that is why.
- The scary-flag half of the owner's request is split to #590.

**At the shipped values the setting cannot fire** (review of #591): main ships `caution_flee_recall_gate_below_depth = 1` (off; the town never asks) and `caution_flee_recall = 1`, so nothing holds a flight's Recall back and the danger exception is never asked. It acts only where an arm turns the gate on (2 or more) or sets `caution_flee_recall = 0`.

Left: a zoo pilot whose arms set the gate or `caution_flee_recall` as well:
- the shipped values (gate 1, recall 1, danger 0);
- gate 2, danger 0 (the DL1 gate alone);
- gate 2, danger 50, 100 and 200;
- recall 0 with danger 0 and with danger 100 (there the danger exception is the only way to Recall beside breeders and chasers).
An arm that sets only the danger percent measures nothing.
