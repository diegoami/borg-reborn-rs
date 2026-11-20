# #796 Dark-entry detection

The optional exception allows Detect Monsters and Read Minds below the
usual character-level gate on a dark, undetected frontier, with a limited
attempt budget and a spell-point reserve. It ships off. Detect Evil and
Seek Battle remain level-gated. The source and engine citations are in
[CALIBRATION](../CALIBRATION.md#dark-entry-detection-exception-796).

The named floating eye is not a valid target for Read Minds, and the
CL1 Necromancer could not learn that spell. The paired replay checks
eligibility and the actual effect on this case. The accessible
room checks use Mage births with Detect Monsters.

On gameplay commit `cd0ed20714c8cef623a9cd96ae25b819b4999d27`, both
300036923 arms killed the first eye with thrown oil at turn 2,831.
The 100k runs had identical keys, no exception cast, and 4,240 resting
turns each. Neither reproduced the historical death on this base.

The final Mage pairs reached 100k. The enabled arms attempted 17 Detect
Monsters casts, spent 2 SP each, and observed 15 successes and two failed
casts. None immediately began resting. The successful casts included
low-danger approaches that revealed a large white snake, a giant white
mouse, and floating eyes. Two whole-game rest totals changed from
2,842 to 2,952 (1548306106) and 2,503 to 2,137 (801785300).

The first implementation produced immediate rests after 16 of 22 casts in
the same three named pairs; it was replaced by the recovery-floor guard.
Both iterations matched the three shipped smoke references. The last
gameplay commit passed the four CI commands and the parameter citation check
with zero wrong citations. Raw local runs are under `/tmp/p7-796-*`.

[Zoo #546](https://github.com/diegoami/borg-zoo/issues/546) requests paired
Mage and Necromancer births with the P7 tactical endpoints. The owner
chooses whether to dispatch and opens the confirmation block.
