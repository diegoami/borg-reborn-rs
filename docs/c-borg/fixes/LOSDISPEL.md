# LOS-dispel carry: verified product scenario, ready for adoption

Consumer [84](https://github.com/diegoami/better-borg-c/issues/84), canonical
source [326](https://github.com/diegoami/neo-angband-tracker/issues/326)
(losdispel-326, source priority high). Fix applied as
`0489305bca448763152c6ec3814f6015fe8a8322` (import of source PR327
`53ef84c743768fdde9abaebc27bd8526b4423de9`, patch
`patches/0022-losdispel-arrival-break.patch`).

## Defect

The radius-ten beam walk in `borg_launch_bolt_at_location()` had no
arrival break: it ran past the target to maximum range and the
excessive-distance guard discarded everything it counted — instrumented
probes showed path sums of 187, 222 and 3996 thrown away in one run — so
the blast scan that values LOS effects over every monster in sight ran
only by accident (cave-edge walks), and when no candidate survived the
launcher leaked its initial −1, valuing Dispel Undead at −15 (the leak
minus the spell cost). The three-token correction (arrival break like a
ball, path damage for bolts and beams only, zero instead of the leaked
−1) lets the blast run: 534. This also retroactively explains the −1 raw
launch values recorded during dispel-259 and activ-260.

## Pins and layered provenance

Before build: twenty-one-carry `cf7a32aeafa0f5b106c5f5473c1055eeb4a7f2cf`
(retained tree `../dispel-reserve`, normal and ASan cores, gate at
`c9f24544`, source-identical to `cf7a32aea` over src/lib).
`borg-fight-attack.c` already carries 0011/0013/0014/0016/0019/0020/0021;
the verifier proves the source diff applied to the before module equals
the product module.

## Verified results (`artifacts/l326/scenarios`, 2026-11-06)

- normal-before `storm`: 0/1, `dispel=−15` (the leaked −1 minus cost).
- normal-after `storm`: 1/1, `dispel=534` — exactly the source evidence's
  value.
- `control` (orb ball): `orb=540`, 1/1 every phase, byte-identical
  before/after.
- asan-before/after mirror the normal phases; no sanitizer report.
- `result.json` records `verified: true` with all commands and hashes.

Native gate on this core: **965/965 unit + 5/5 frontend**. Tooling 29/29.

## Documented cross-carry interactions (`artifacts/l326-carry/regression.py`, 89 cases)

All 89 prior fixture cases re-ran on this core. **87 are whole-log
identical.** Two prior scenarios carry absolute expectations that embed
the now-removed leak, and their transitions are asserted (not weakened):

- **dispel-248 `undead`** (the thirteenth carry's adapted fixture):
  `LAUNCH direct=−1` becomes `direct=0` and the rating −21 becomes −20;
  the undead refusal contract (value ≤ 0, zero HP lost) is unchanged and
  the case still passes 1/1. `living` and `bolt` remain byte-identical.
- **dispel-259** (the twenty-first carry's fixture, both modes): with the
  blast scan reachable, the three vampire bats are now valued, moving
  both absolute expectations by the same +296 — `reserve` −10627 →
  −10331 and `clear` −127 → 169; only those two absolute eqs fail, and
  the doubled-penalty delta (10500 apart) remains visible in the raw
  values. The dispel-259 carry evidence remains valid at its recorded
  commits; its pinned absolute values are superseded on cores carrying
  this fix.

## Scope and limits

Bounded rating scenario: the exact production maneuver for Dispel Undead
with the reported geometry. Rating figures are the Borg's heuristic; no
combat-outcome, frequency or survival claim.

## Exact next work

1. PR from `task/carry-losdispel`, agent evidence review plus required
   hosted checks, then normal merge.
2. On merged main: frozen v1 measurement seeds 11/29/47, cap 20000, wall
   120, stall 30.
3. Mark 84 carried/open+patch-active; receipts on issue 84, source 326
   and checkpoint 1; evidence archive under `artifacts/l326-carry/`.
4. Then staff-262 (PR329/330), which stacks on this correction, then
   recheck source207 for newer demonstrations (source-ranked 261).
