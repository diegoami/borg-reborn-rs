## 2026-11-22: wait out a bad test potion; 25 HP to test (#1060)

**Found** (owner: "a character drank a potion of poison and then goes happily
exploring"). Seed 1928291203 at `track-p` (with #1035): #983's test drank a
Potion of Poison (10+2d7 turns at 1 HP a turn, object.txt,
game-world.c:514-516), then walked 76 steps to the stairs (HP 26 -> 18), drank
a CLW (no cure for poison in 4.2) and, once the poison ended, tested a
Confusion and spent a Cure Serious Wounds on it. The 50% floor let a 26-HP
character test at 13 HP against a worst case of 24.

**Done.** `item_test_unknown_min_hp` (25, shipped 25): the test also needs
this many HP. `item_test_unknown_hold_turns` (30, shipped 30) and
`potion_test_hold` (`borg_item_use.rs`, called right after `borg_caution` in
`borg_think_dungeon`): after a test, while a harmful status it caused lasts
(poisoned, confused, blind, afraid, hallucinating, paralysed, amnesiac) and
the danger here is 0, stay still (`,`) where it tested; the heals and escapes
of `borg_caution` come first, and any danger, the status's end or the cap
ends it. `Borg::potion_test_hold_until` holds the cap. Unit tests in
`unknown_potion_tests`. `docs/CALIBRATION.md` documents both.

**Checked.** fmt, clippy, the workspace and `borg-core` tests (705),
`scripts/tests`, `engine/tests` (the four `test_shopping` errors are this
container's borg-cemetery checkout; CI runs them), `check-param-citations.py`.
Seed 1928291203: its Poison test (now at turn 84,948, the game diverges from
the 25-HP gate) is waited out on the spot for 8 turns, then it rests and goes
on; alive at the cap. The other 13 named cases of #969/#983: all at the cap
(Grip and the crow, the two deaths before, now live: diverged games, a
direction, not an effect); 58 tests, 68 turns of waiting. Smoke: 0/3
identical (the low-HP smoke characters no longer test below 25 HP),
re-recorded; all three at their cap.

**Where this meets other work.** #934 (another session: how poison is
weighed in general, CLW drunk against poison) is untouched; this only changes
what follows the Borg's own test. `engine/tests/test_stair_handoff.py` pins
`item_test_unknown_potions = 0`, so the incident tests do not see it.

**Left.** borg-zoo#703 (the normal-world run of the potion test) now measures
it with these two settings.

**Owner.** 25 HP means a character below 25 max HP never tests; the percent
floor (50) still applies above it.
