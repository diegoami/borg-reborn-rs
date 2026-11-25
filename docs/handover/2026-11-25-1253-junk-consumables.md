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

The owner expanded #1253 in chat to all known junk potions and asked
for immediate town cleanup of harmful items. The owner then approved
shipping both settings at `1` on Track P. The curated throw set covers
the named harmful potions and expendable Slime Mold Juice. It stays
independent of any broader effect classification. Population effects
remain unmeasured.

[#1256](https://github.com/diegoami/borg-reborn-rs/issues/1256) also needs
town cleanup as part of its carry-reserve work. Its
[draft PR](https://github.com/diegoami/borg-reborn-rs/pull/1259) should
incorporate this town path and use the current Track P smoke reference
before integration.

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

CI now runs these native cases through the shipped Borg and Angband's
inventory commands. The test build positions an awake monster at the
requested range or puts the stocked character on town stairs.
`BORG_TEST_FEATURES=test-junk-consumables harness/build.sh` builds the
fixture and `python3 engine/tests/test_junk_consumables.py` runs it.
The three exact-turn stair replays pin both settings off to preserve
their recorded inventory actions and incident turns.
