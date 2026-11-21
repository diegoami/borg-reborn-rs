# Dispel Life filter carry: verified product scenario, ready for adoption

Consumer [57](https://github.com/diegoami/better-borg-c/issues/57), canonical
source [248](https://github.com/diegoami/neo-angband-tracker/issues/248)
(dispel-248). Fix applied as `23d8ed579317db9e9d2037d21d529af3c29fd433`
(import of source PR291 `6912b85cb7b902616ad786dfc20afe3b4cf28b10`, patch
`patches/0013-dispel-life-filter.patch`).

## Defect

The Dispel Life filter in `borg_launch_damage_one()` kept damage only for
monsters flagged **both** nonliving and undead — the exact inverse of the
game's drain, which hurts every living monster (nonliving is undead or
nonliving). Since no monster in the game data carries NONLIVING, the spell
was rated below doing nothing against every enemy (value −21 = launch
failure minus mana). The corrected OR-filter matches the game.

## Pins and layered provenance

Before build: twelve-carry `b287fa712da8f3355b432c25a0365b8fb2d8391e`
(retained tree `../tp-other-skip`, normal and ASan cores, gate at
`706fb49b`, source-identical to `b287fa712` over src/lib).
`borg-fight-attack.c` already carries the missile guard (0011), so the
verifier proves the before module + the source filter block equals the
product module (the inverted filter occurs exactly once); the patch round
trip from the twelve-carry module reproduces the product module SHA256
`9c9231d946abc58a0e978d7d2312eeb8eea599a39461971824046dd581f54726`.

## Why the unchanged fixture needed an adapted expectation (measured)

The first product run with the **unchanged** evidence fixture
(`artifacts/dispel/scenarios`, retained red: verified:false) failed its
living-mode contract even after the fix: `value=39` vs the fixture's
`expected=19`. Measured cause: `borg_launch_bolt_aux_hack()` adds the
target's danger at the player's grid to the launch reward
(`d = d + p1`), so the rating is `dam + p1 − mana`. The pinned fixture's
`expected = dam − mana` implicitly assumed `p1 == 0` — true only on
pristine C, where the hill giant's projectile danger was zeroed by the
caster-self-block defect that the carried 0008 correction (consumer 40)
removed; that carry's own receipt measured this exact giant's danger
change. On this product `p1 = 20` (printed as `target_danger`), so launch
= 59 = 39+20 and the rating = 39 = 59−20 — the chooser's arithmetic is
correct; the fixture's absolute expectation was not.

The adapted fixture (`tools/dispel-filter-scenario.c`, three edits from
the pinned one: provenance header, a `p1` computed with the same
production `borg_danger_one_kill` call, and `expected = dam + p1 − mana`)
keeps the contract relational. No game rule, threshold or assertion
strength changed; the undead and bolt controls are byte-for-byte the
pinned ones (the bolt control's log is even identical to the original
attempt's). The original red attempt is preserved under
`artifacts/dispel/scenarios`.

## Verified results (`artifacts/dispel/adapted`, 2026-11-06)

- normal-before `living`: 0/1, `value=−21`, `target_danger=20`.
- normal-after `living`: 1/1, `value=39 == expected (39+20−20)`, the
  native cast drains 66 HP from the living giant (225→159).
- `undead` (vampire bat): `value=−21`, zero HP lost, `undead=1`, 1/1 on
  every phase, byte-identical before/after — never worth casting in
  either version.
- `bolt` (primary-bolt control): rating 31, 1/1 everywhere,
  byte-identical before/after.
- asan-before/after mirror the normal phases; no sanitizer report.
- `result.json` records `verified: true` with all commands and hashes.

Native gate on this core: **965/965 unit + 5/5 frontend**. Regression
matrix (`artifacts/dispel-carry/regression.py`): all **69 prior native
fixture cases whole-log identical** to their baselines (60 nine-carry +
3 glyph + 3 missile + 3 tp-other). Tooling 29/29.

## Scope and limits

Bounded native rating scenario: the exact production calls
(`borg_launch_bolt`, `borg_attack_aux_spell_bolt`) for Dispel Life plus a
real native cast of the learned spell; the target list is filled the way
the attack chooser fills it. Danger figures are the Borg's heuristic, not
HP predictions; the 66 HP drain is the game's, not the Borg's estimate. No
combat-outcome, frequency or survival claim. The adapted expectation's
`p1` is computed by the production danger function, not hardcoded.

## Exact next work

1. PR from `task/carry-dispel-filter`, agent evidence review plus required
   hosted checks, then normal merge.
2. On merged main: frozen v1 measurement seeds 11/29/47, cap 20000, wall
   120, stall 30.
3. Mark 57 carried/open+patch-active; receipts on issue 57, source 248 and
   checkpoint 1; evidence archive under `artifacts/dispel-carry/`.
4. Then recheck source253 (arc angle, PR297/298) readiness plus any newer
   demonstrations.
