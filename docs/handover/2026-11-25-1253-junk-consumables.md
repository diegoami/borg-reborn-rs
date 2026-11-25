## 2026-11-25 — Throw junk potions and discard harmful consumables in town (#1253)

Track P, branch `task/T1253-junk-consumables`, based on `1f8b96c0`.

The ordinary throw evaluator considers already-carried, known junk potions
when one-turn danger is at most a tenth of current HP. It checks native throw
range, impact dice, and hit chance against awake, visible targets. A damaging
throw gets two utility points, and an equal scoring singleton is preferred
to a larger stack. A zero-dice potion can be spent for a utility of one.
Slime Mold Juice is available when the Borg does not need its nutrition.
Healing, buffs, and other useful supplies stay in the pack.

Town cleanup follows caution and precedes shopping. It uses Angband's
individual ignore command to drop each known harmful stack. Native
auto-pickup then leaves that stack on the floor, and newly found copies
of the same kind remain eligible in the dungeon. The list follows
[the strategy manual's bad items](../strategy/07-consumables-and-devices.md#13-bad-items-do-not-use-ignore-on-sight)
and [the engine kind data](../../engine/angband/lib/gamedata/object.txt).

Both candidate settings ship at zero under the P9 gate.
Set `borg_param_attack_throw_junk_potions = 1` and
`borg_param_junk_discard_harmful_in_town = 1` to test both behaviors.
Population effects remain unmeasured.

Rust format, workspace Clippy, both Rust test suites, and parameter
citations passed. All four shipped smoke runs matched the reference keys,
messages, and state logs.

Local native fixtures confirmed that a singleton Potion of Blindness was
spent at 10 grids, a target at 11 grids received no throw until it moved
into range, and a zero-dice Slime Mold Juice was spent when there was no
better attack. The unknown-potion control kept its pack, the stronger
melee option won at close range, and the dangerous contact used
Teleportation. At the town stairs, the first five actions cleared two
potion kinds, Summon Monster scrolls, a Haste Monster wand, and a Summoning
staff before shopping. The dropped stacks stayed out of the pack during
the observed town run.
