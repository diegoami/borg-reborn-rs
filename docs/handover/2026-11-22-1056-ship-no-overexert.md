# #1056: ship attack_reserve_spell_overexert = 0

**Done.** `harness/borg.txt` ships `borg_param_attack_reserve_spell_overexert = 0`
(owner, 2026-11-22): the emergency Magic Missile (#980, #991) is cast only
within the current mana, never beyond it, so the engine's faint
(player-spell.c:540-553) no longer follows it. `engine/tests/test_stair_handoff.py`
pins the incident replays to 1, the value they were recorded with.

**Found.** Smoke (default core, borg-rs) plays identically: 3/3, nothing
re-recorded. The smoke Mage never casts the reserve spell beyond its SP.

**Left.** Not measured: borg-zoo#675 (ctl / noexert / faint100) has no results
yet. With 0, `attack_reserve_faint_percent` (25, the lone-slow bundle #1047)
no longer acts.

**For the owner.** Nothing to decide.
