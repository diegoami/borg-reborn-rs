## 2026-11-18: flight Recall from a danger share; the scary flag clears after a kill (#587)

Owner, after borg-zoo#267 (the depth gate on flight Recall: not shown to change survival, leaning against) and borg-zoo#256 (latch off: four deaths soon after the flag cleared, three of them because the monster was only forgotten, not gone): calibrate instead of switch. Two settings, both off in `harness/borg.txt`:

- `caution_flee_recall_danger_percent` (0): where the depth gate or `caution_flee_recall = 0` holds a flight's Recall back, the danger here at or above this percent of the current HP allows it, beside the breeder and chaser exceptions. Only the flight's read uses it; the stand tests keep the old exceptions.
- `caution_scary_unlatch_kill_only` (0): with `caution_scary_latch = 0`, 1 clears the flag only after a monster of a race that made the level scary was killed. The scan records up to 8 such races per level (`mon.scary_races`), and a death message for one of them sets `mon.scary_killed`. A forgotten monster has not left the level; a hurt or afraid one is #539's (`caution_scary_hurt_percent`, `caution_scary_afraid`).

The idea of clearing after N turns unseen was dropped (owner): it guesses where a monster has gone.

Left: a zoo pilot (gate 2 with danger 50/100/200%; latch off with kill-only against latch on and plain latch off).
