# Stale same-square target carry: verified product scenario, ready for adoption

Consumer [99](https://github.com/diegoami/better-borg-c/issues/99), canonical
source [322](https://github.com/diegoami/neo-angband-tracker/issues/322)
(staletarget-322, priority high). Fix applied as
`0d843a414c8deaa8ce7aa3178dea39418dc22d62` (import of source PR343
`1ab30ef0bc9a12caa5b1cf48e60c204ef564e9de`, patch
`patches/0027-stale-same-square-target.patch`).

## Defect

A tracked monster entry left on the player's own grid after teleportation and
blindness satisfied the melee adjacency check at distance zero. In acting
mode the production attack chooser queued `+5`, the center-square command
whose empty direction prompt trips the Borg's unexpected-direction abort.
The correction requires distance exactly one, so the stale target is ignored.
A genuinely adjacent monster remains a valid target.

## Pins and layered provenance

Before build: twenty-six-carry `54896eb4298c7f964d45fed42695299483f7b338`
(retained tree `../calc-reload`, normal and ASan cores, gate at `c528eb81`,
source-identical to `54896eb42` over src/lib). `borg-fight-attack.c` carries
the earlier attack patches through 0024; the verifier proves that the source
diff applied to the before module equals the product module. The source
evidence fixture at `929764f4` is used unchanged.

## Verified results (`artifacts/s322/scenarios`, 2026-11-07)

- normal-before `stale`: 0/1, `acted=1`, `keys="+5"`, `center_attack=1` —
  the production chooser attacks the player's own square.
- normal-after `stale`: 1/1, `acted=0`, `keys=""`, `center_attack=0` — no
  action or key is queued for the stale target.
- `control` (real adjacent kobold): `acted=1`, `keys="+2"`,
  `center_attack=0`, 1/1 on every phase, byte-identical before/after.
- asan-before/after mirror the normal phases; no sanitizer report.
- `result.json` records `verified: true` with all commands and hashes.

Native gate on this core: **965/965 unit + 5/5 frontend**. Regression matrix
(`artifacts/s322-carry/regression.py`): all **99 prior native fixture cases**
pass (97 identical plus the two documented losdispel interactions asserted),
and the two new target cases pass, **101 total**. Tooling 29/29.

## Scope and limits

Bounded native production-chooser scenario in acting mode using the recorded
post-teleport/blindness stale-target state. It proves rejection of the
own-square attack and preservation of a genuine adjacent attack. It does not
claim a frequency estimate or a measured survival improvement.

## Exact next work

1. PR from `task/carry-stale-target`, agent evidence review plus required
   hosted checks, then normal merge.
2. On merged main: frozen v1 measurement seeds 11/29/47, cap 20000, wall
   120, stall 30.
3. Mark 99 carried/open+patch-active; receipts on issue 99, source 322 and
   checkpoint 1; evidence archive under `artifacts/s322-carry/`.
4. Resume the low-priority queue with opblock-324 and calfree-323, then
   brand-240; recheck source207 for newer high-priority demonstrations.
