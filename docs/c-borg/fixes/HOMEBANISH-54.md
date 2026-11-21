# Homebanish-54 carry: verified product scenario

Consumer [298](https://github.com/diegoami/better-borg-c/issues/298),
canonical source
[54](https://github.com/diegoami/neo-angband-tracker/issues/54)
(CBUG-0038, homebanish-54; source fix `e49ab948`, evidence fixture
`8484955b`). Fix applied as `6373f9b8a` (`cherry-pick -x` of source
`e49ab94809d6f18d0b1d155ff2b1bc6ba8d3db78`, patch
`patches/0077-home-banishment-count.patch`).

## Defect

The home score rewards Scrolls of Banishment through `num_genocide`,
but nothing ever counted them: the scroll scan in
`borg_notice_home_aux()` only counted Mass Banishment
(`sv_scroll_mass_banishment`). The reward was always zero, so the Borg
never kept a Banishment scroll in the home. The correction counts
`sv_scroll_banishment` into `num_genocide`. Home stocking decision
only; no survival claim.

## Pins and provenance

Before: merged main `63979b54b` (snapshotted build `build/native-before`,
gate result `artifacts/native-before/result.json`, 965/965 + 5/5 at that
commit). `borg-home-notice.c` is otherwise pristine on the product. The
verifier proves that the source diff applied to the before module equals
the product module and that the applied commit is the only `src`/`lib`
difference from the before commit. Fixture: kit `8484955b` at head
`e24fa8cc`, asserted byte-identical to the run pin.

## Verified results (`artifacts/hb54/scenarios`, 2026-11-08)

- `banish` — fail before, pass after: before 0/1
  (`stores_banishment_scroll=0`; the home score ignored the stored
  scroll — reward always zero, scroll sold), after 1/1
  (`stores_banishment_scroll=1`; the scroll is counted and kept).
- `mass` control: 1/1 in both phases, log byte-identical before/after
  (`b802c599…`).
- `result.json` records `verified: true` with all commands, pins and
  hashes. The verifier passed on the first complete run; no harness
  corrections were needed.

Native gate on the product: **965/965 unit + 5/5 frontend** (before
snapshot gate and product tip gate); tooling **129/129**.
Prior-fixture matrix not rerun (uncommitted runner; known gap recorded
on checkpoint 1 and the milestone 6 review).

## Scope and limits

Bounded home-stocking scenario driving the production home score. It
proves a stored Banishment scroll is now counted and kept; no claim
about when the Borg buys or keeps one in ordinary play.

## Exact next work

1. PR from `task/carry-homebanish`, agent evidence review plus required
   hosted checks, then normal merge.
2. Frozen v1 cohort seeds 11/29/47 at the merged tip against the
   0076-after cohort at `63979b54b`.
3. Receipts on issue 298, source 54 and checkpoint 1.
