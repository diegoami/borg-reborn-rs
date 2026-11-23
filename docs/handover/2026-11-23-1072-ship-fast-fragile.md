# #1072: ship the #926 fast-fragile bundle

**Done.** `harness/borg.txt` ships the #926 bundle at the owner's values
(2026-11-23: "heal floor 50, 75th percentile, count_dot 1"):
`fast_fragile_escape_quantile = 75`, `fast_fragile_heal_floor = 50`,
`fast_fragile_count_dot = 1`. `engine/tests/test_stair_handoff.py` pins the
incident replays to 0.

**Found.** Smoke (default core, borg-rs) plays identically, 3/3; nothing
re-recorded. No smoke game meets a fast-fragile monster at low HP.

**Left.** Not measured: borg-zoo#711 (ctl / bundle / each alone, pilot then
full at track-p 769f3fbd) decides whether it stays on. #1058's smoke game
does not cover these settings.

**For the owner.** The values are the owner's, not a measurement; #672
proposed q50 and floor 25.
