# Necrolight-296 carry: verified product scenario

Consumer [377](https://github.com/diegoami/better-borg-c/issues/377),
canonical source
[296](https://github.com/diegoami/neo-angband-tracker/issues/296)
(necrolight-296; source fix `ea838de67`, evidence fixture `742e04ed8`).
Applied as a `cherry-pick -x` of source
`ea838de6782149c83300dccdfc06d15c099fbd86`, patch
`patches/0099-necro-lit-square-penalty.patch` (applied commit
`c2c2156140`).

## Defect

The game adds 25 to a Necromancer's spell failure chance whenever the
player's square is lit (`square_islit()`: torchlight, permanent room
light, bright terrain or a lit monster nearby). The Borg's estimate
added it only when the square was lit by the Borg's own light source
(`BORG_LIGHT`), so a Necromancer without a light standing in a lit room
believed its spells were far safer than they were: with Vampire Strike
at a real 33% chance of failure it estimated 8%, passed its own 25%
limit and cast it anyway. The correction uses the same lit-square test
the game uses, `square_islit(cave, borg.c)` (`src/borg/borg-magic.c`).

## Pins and provenance

Before: merged main `1f0d50768` (snapshotted build `build/native-before`,
gate result `artifacts/native-before/result.json`, 965/965 + 5/5 at that
commit). `borg-magic.c` is otherwise pristine on the product. The
verifier proves that the source diff applied to the before module equals
the product module and that the applied commit is the only `src`/`lib`
difference from the before commit. Fixture: kit `742e04ed8` at head
`ee195556f`, asserted byte-identical to the run pin.

## Verified results (`artifacts/nl296/scenarios`, 2026-11-09)

- `lit` — fail before, pass after: before the estimate undershot the
  game's real chance (`borg_estimate=8` vs `game_chance=33`,
  `estimate_matches_game=0`) and the spell was cast anyway
  (`cast_spell=1` on a chance over the borg's own threshold); after the
  estimate matches the game (`33`) and the borg refuses
  (`cast_spell=0`).
- `dark` control: 1/1 in both phases (8% estimate, spell casts —
  nothing lit, nothing to penalise), log byte-identical
  (`ae9da541…`).
- `mage` control: 1/1 in both phases (a non-Necromancer class has no
  lit-square penalty), log byte-identical (`0db7bac3…`).
- `result.json` records `verified: true` with all commands, pins and
  hashes. The verifier passed on its first complete run; rows equal the
  kit's recorded outcomes.

## Scope and limits

Bounded spell-cast scenarios (lit room, dark room, non-Necromancer
class). They prove the estimate alignment and the refusal; no survival
claim.

## Exact next work

1. PR from `task/carry-necrolight`, agent evidence review plus required
   hosted checks, then normal merge.
2. Frozen v1 cohort at the merged tip against the runeglyph tip cohort.
3. Receipts on issue 377, source 296 and checkpoint 1.
