## 2026-11-22: wait out a bad test potion; 25 HP to test (#1060)

**Found** (owner: "a character drank a potion of poison and then goes happily
exploring"). Seed 1928291203 at `track-p` (with #1035): #983's test drank a
Potion of Poison (10+2d7 turns at 1 HP a turn, object.txt,
game-world.c:514-516), then walked 76 steps to the stairs (HP 26 -> 18), drank
a CLW (no cure for poison in 4.2) and, once the poison ended, tested a
Confusion and spent a Cure Serious Wounds on it. The 50% floor let a 26-HP
character test at 13 HP against a worst case of 24.

**Done.** `item_test_unknown_min_hp` (25, shipped 25): the test also needs
this many HP. `item_test_unknown_hold_turns` (180, shipped 180; 30 before the re-review) and
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

**Review (Codex, at 529511a6).** Four P2s, fixed: (1) a Slowness test (no
Borg trait) neither held nor stopped the next test: its timer is read
(`slowed`) in both; (2) the hold returned before `borg_recover`, so a known
Neutralize Poison waited for caution's HP threshold: the hold now runs
`borg_recover` first and stands still only when it does nothing; (3) the cap
was in Borg turns, about two per world tick at +10 speed: it is now a game-turn
deadline (10 per world tick, game-world.c:1080); (4) an escape by
`borg_caution` returned before the hold saw the danger, and a Phase Door
landing could be waited on: `potion_test_hold_validate` runs before
`borg_caution`, and the hold is bound to the test grid and level.

**Re-review (Codex, at eda49ea1).** P2: the 30-tick cap covered Poison but
not Blindness (75+4d25, up to 175 ticks) or a long Slowness (up to 40):
Codex reproduced a blind Borg walking off at 69,960 with the blind cures
switched off. The cap is now 180 ticks (compiled and shipped), the longest
supported potion's; the hold still ends earlier on the status's end, any
danger or a move. P3 (a complete rest `R&` started by `borg_recover` inside
the hold outlasting a short cap): kept; the rest is in place and the engine
ends it on any disturbance, and with the cap above every status it cannot
outlast the hold's purpose.

**Second re-review (Codex, at 8fb76110): no blocker, two P3s.** (1) A
short configured cap still cannot interrupt a recovery rest the hold starts
(Codex's fixture: a two-tick cap, `R&` at turn 21, the next decision at 211).
Codex also ran the fixture with the hold off: ordinary recovery chose the same
actions, with identical key, message and state logs. Kept and documented in
CALIBRATION.md: the cap bounds when a hold decision starts, not a rest it
starts. (2) `fight_before_flight` could act and return before
`potion_test_hold_validate`, so a fight won from the test grid would let the
hold resume once the danger was back to 0. Validation now runs before the
early fight.

**Third re-review (Codex GPT-6, at c963bfee).** P1: the branch conflicted
with `track-p`, where #1097 (#1063) had shipped the test's HP floor at 60
with a known Neutralize Poison or Cure Critical Wounds waiving it, on this
PR's own `item_test_unknown_min_hp`. The merge keeps #1097's gate, floor,
exemption and settings, and adds only the hold (`item_test_unknown_hold_turns`
180). P3: a Potion of Surprise's stat scramble (`TIMED_INC:SCRAMBLE`
20+1d20, object.txt) has no trait, so the hold let a scrambled Borg explore:
it is now read from its timer with Slowness (`untraited_bad_timer`), in the
hold and in the test gate.

**Where this meets other work.** #934 (another session: how poison is
weighed in general, CLW drunk against poison) is untouched; this only changes
what follows the Borg's own test. `engine/tests/test_stair_handoff.py` pins
`item_test_unknown_potions = 0`, so the incident tests do not see it.

**Left.** borg-zoo#703 (the normal-world run of the potion test) now measures
it with these two settings.

**Owner.** 25 HP means a character below 25 max HP never tests; the percent
floor (50) still applies above it.
