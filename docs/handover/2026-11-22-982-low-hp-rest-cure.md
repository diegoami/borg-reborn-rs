## 2026-11-22: Drink before resting badly hurt with a fast monster awake (#982)

Stacked on #981's PR (both change `borg_recover`). Every check below ran on
the **default core, borg-rs**; none on the reborn core.

**Shared cause with #981?** Partly. In both deaths `borg_recover` rests
because the attacker's danger is 0, and no rule drinks at danger 0. The
inputs differ, so the fixes differ. In #981 the archer was never a monster
to the Borg: a bolt did not reach the unseen-attacker mode. In #982 the
crow was a known monster, awake, 9 grids away and out of view. A one-turn
danger counts it as 0. They are two changes in two PRs.

**Reproduced.** Half-Troll Necromancer seed 2125189042 (`-race 7 -class 4`)
on track-p `4b2c2fcf` plays key for key the zoo's game (`6f2825472c67b78f`).

**Cause (checked in the code at the pin).**
- No HP floor makes `borg_recover` prefer a potion that the Borg has.
  `borg_heal` (`borg_caution.rs`) returns at once at danger 0 unless
  poisoned or cut, as in C. `borg_recover`'s costly heal (below half HP,
  `q < 25`, 45 at 2/17 HP after the hurt and critical bonuses) tries a
  Healing rod, Cure Serious and Cure Critical Wounds, never Cure Light.
  The Necromancer had only CLW, so it rested.
- A known fast monster out of view does not count. `borg_check_rest` lets
  an awake monster at 5 or more grids pass (it checks distance and danger
  only). The #615 "monster was near" hold (`recover_rest_monster_turns`) is
  shipped 0. #842's heal ban (#926) applies only in contact, so it did not
  apply here.

**Fix, behind `recover_rest_cure_hp_percent` (default and shipped 0).**
With N > 0, an HP rest that is due below N% of the maximum HP, with an
awake monster that can move known on the level and faster than the Borg by
at least `recover_rest_cure_speed_edge` (1), is replaced by Cure Light,
Cure Serious, then Cure Critical Wounds (the raw quaff, as #697's heal).
With none in the pack, it rests as before.

**Checked.**
- Replay, switch off: `6f2825472c67b78f`, the same death.
- Replay, `borg_param_recover_rest_cure_hp_percent = 50`: `aee7bb2ee27e5831`.
  At 8342 it notes "HP 2/17, a fast awake crow (speed +10 over the Borg)
  known 9 grids away, out of view" and drinks. The crow comes back; the
  Borg fights it and reads Phase Door, and drinks again at 7/17. It reaches
  the 100,000-turn cap at CL6, DL2. One game; it is not a measurement. The
  value 50 is a hypothesis.
- `smoke-borg-rs`: 3/3 identical. CI's cargo commands, the citations and
  `borg-core-vendor.py check` pass. Unit test: `a_fast_awake_monster_on_the_level`.

**Not done.**
- No borg-cemetery records, as in #981 (no write access from this session).
- No zoo measurement (a borg-zoo request, which the owner starts).
- Not done: a floor without the fast-monster condition, or one that lets
  `borg_recover`'s costly heal drink CLW. These are other candidate rules
  for the zoo to compare.

**For the owner.** Whether to ship it, and at what N, after the zoo
measures it.
