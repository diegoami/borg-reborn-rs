## 2026-11-18: the scary flag clears after the scary monster is killed (#590)

Owner: with the shipped latch (`caution_scary_latch = 1`) killing the monster that made a level scary does not clear the flag; with the latch off (#562) it clears whenever none is known, and borg-zoo#256 found four deaths soon after, three with the monster only forgotten (a monster never leaves its level).

- `caution_scary_unlatch_kill` (0, off in `harness/borg.txt`; needs `caution_scary_stand = 1`): 1 clears the flag once no scary monster is known and the last one known was killed, with the latch on or off. Forgetting is not enough. The scan records the races that made the level scary (`mon.scary_races`, a bit per race, no cap); a death message for one (`kill_found`, and the late pass in `handle_late_messages`) sets `mon.scary_killed`; a scan that still finds a scary monster known spends it (`note_scan`, review of #592: otherwise an old kill cleared a later scare whose monster was only forgotten). A new level forgets both.
- Door scares stay latched (`stand_inputs`).
- A hurt or afraid scary monster is the next issue (owner).

Left: a zoo pilot (latch on; latch on with the kill rule; latch off; latch off with the kill rule).
