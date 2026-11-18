## 2026-11-18: flight Recall from a danger share (#587)

Owner, after borg-zoo#267: the depth gate on flight Recall (`caution_flee_recall_gate_below_depth` 1/2/3, 918 paired starts up to big) is not shown to change survival, and it leans against, as in #255. Calibrate instead of switch: "Recall only from a certain danger".

- `caution_flee_recall_danger_percent` (0, off in `harness/borg.txt`): where the depth gate or `caution_flee_recall = 0` holds a flight's Recall back, the danger here at or above this percent of the current HP allows it, beside the breeder and chaser exceptions. Only the flight's read uses it (it has the danger at hand); the stand tests keep the old exceptions. The asymmetry is deliberate: the stand tests ask whether an escape from the level exists, the flight asks whether to spend the scroll now, and only the second depends on how pressed the Borg is. If a zoo arm shows the two disagreeing, that is why.
- The scary-flag half of the owner's request is split to #590.

Left: a zoo pilot, gate 2 with danger 50, 100 and 200 percent of HP.
