## 2026-11-22: Confused attack on the grey mushroom patch, shipped (#901)

**Done.** Shipped values in `harness/borg.txt`, on the owner's decisions
(2026-11-22, after #867/#868 and #869):

- `borg_param_attack_confused_by_direction` 0 → 1.
- `borg_param_attack_throw_point_blank_divisor` 1 in the common file, and
  0 (the arrow divisor: 5, Rogue 3) for Warrior, Blackguard, Paladin and
  Rogue, in `harness/borg_<class>.txt`, folded with
  `scripts/borg-class-files.py fold`. Owner: "melee is infinite, flasks are
  not"; a workaround until #912 saves flasks for when they are needed.

`caution_ranged_vs_immobile` and `caution_ranged_only` stay 0. With the
#869 kill-on-sight entry and `attack_on_sight` 1, the Borg attacks the patch
on sight. Once confused next to it, it throws oil or uses a device by
direction, and holds the cure while it has an attack. A caster now also
throws oil at point blank in ordinary fights.

**History of the PR.** The first commit shipped the divisor 1 for everyone.
The smoke Warrior then threw oil at an adjacent white worm mass at turn
5,294. At CL2-3 with the starting Dagger, a flask rated 84 against a round
of blows' 43 (`thrust_damage` multiplies by `BI_BLOWS`). The engine agrees:
a thrown flask is 1d4 x 3 (`THROWING`) x 3 (`EXPLODE`,
`player-attack.c:1296`), about 22 a hit. The owner dropped the line, then
asked for the class split.

**Checked.** Smoke at the final shipped settings, default core: 3/3
identical to `track-p`'s `smoke-borg-rs` (the reference did not need
re-recording). The switch and the casters' divisor do not fire in smoke.
The Warrior's 0 is resolved (`params-core.json`, `class_values`) and is
what keeps its game: at the common 1 it diverged. `scripts/tests` (class
files) pass.

**Not measured.** borg-zoo#636 measures the switch and the divisor at 1 for
everyone, not the class split. Shipped before the measurement on the owner's
call (RULES 14). Open: #896 (the confused attack), #912 (saving flasks).
