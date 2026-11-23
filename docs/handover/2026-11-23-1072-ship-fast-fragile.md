# #1072: ship the #926 fast-fragile bundle

**Done.** `harness/borg.txt` ships #672's combined arm (owner, 2026-11-23):
`fast_fragile_escape_quantile = 50`, `fast_fragile_heal_floor = 25`,
`fast_fragile_count_dot = 1`. `engine/tests/test_stair_handoff.py` pins the
incident replays to 0.

**Found.** Smoke (default core, borg-rs) plays identically, 3/3; nothing
re-recorded. No smoke game meets a fast-fragile monster at low HP.

**Left.** Not measured: borg-zoo#711 (ctl / bundle / each alone, pilot then
full at track-p 769f3fbd) decides whether it stays on. #1058's smoke game
does not cover these settings.

**For the owner.** The values are #672's proposal, not a measurement: q50
(not esc75 or q100), floor 25.
