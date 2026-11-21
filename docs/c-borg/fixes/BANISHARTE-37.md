# Banisharte-37 carry: verified product scenario

Consumer [305](https://github.com/diegoami/better-borg-c/issues/305),
canonical source
[37](https://github.com/diegoami/neo-angband-tracker/issues/37)
(banisharte-37; source fix `caa6c6ac`, evidence fixture `97bcf2a98`).
Fix applied as `8f2669a1f1` (`cherry-pick -x` of source
`caa6c6acc70c35ced4bfaaf2781036d99b5d32e5`, patch
`patches/0078-banish-artifact-hit-points.patch`), first step of the
defend-rest-shop PR (stacked with restmult-149 and shopline-125).

## Defect

The Banishment-artifact defence tested the item count instead of the
borg's current hit points, so it could banish while too badly hurt to
afford it or skip when healthy. The correction tests the hit points.
Defence decision only; no combat effect measured.

## Pins and provenance

Before: merged main `b46a577a4` (snapshotted build `build/native-before`,
gate result `artifacts/native-before/result.json`, 965/965 + 5/5 at that
commit). `borg-fight-defend.c` is otherwise pristine on the product. The
verifier proves that the source diff applied to the before module equals
the product module and that the applied commit is the only `src`/`lib`
difference from the before commit. Fixture: kit `97bcf2a98` at head
`bbc5de859`, asserted byte-identical to the run pin.

## Verified results (`artifacts/ba37/scenarios`, 2026-11-08)

- `main` — fail before, pass after: before 0/1 (`banishes_evil=0`; the
  item-count test skipped the banish), after 1/1 (`banishes_evil=1`).
- `hurt` control: 1/1 in both phases, log byte-identical before/after
  (`b7437b1f…`).
- `result.json` records `verified: true` with all commands, pins and
  hashes. The verifier passed on its first complete run.

## Scope and limits

Bounded defence scenario driving the production banish decision. It
proves the hit-point test; no survival claim.

## Exact next work

1. Stacked steps 2-3 (restmult-149, shopline-125) in the same PR with
   per-step gates; agent evidence review plus required hosted checks,
   then normal merge.
2. Frozen v1 cohort at the merged tip against the 0077-after cohort.
3. Receipts on issue 305, source 37 and checkpoint 1.
