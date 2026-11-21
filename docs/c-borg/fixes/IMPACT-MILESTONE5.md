# Impact milestone 5: eight carries since impact v4 (patches 0043–0050)

Review package for [milestone 5](https://github.com/diegoami/better-borg-c/milestone/5):
the eight verified C bug carries after the milestone4 runtime snapshot
`70993393a` (42 carries) — patches 0043–0050, consumer issues 188, 196,
201, 205, 214, 216, 212 and 224. Runtime snapshot after the eighth
carry's merge: `5384f12ed` (main is at `4561515ac` after the docs-only
recording PRs; the current tip `1cd426af4` adds out-of-tree tooling only).
Every carry is merged, labeled carried/open+patch-active, and measured.

## The eight carries

| Patch | Source (issue, PR head @ pristine base `e473720ed`) | Applied | Merged | Consumer | Evidence pin | Verification |
| --- | --- | --- | --- | --- | --- | --- |
| 0043 excavate | [236](https://github.com/diegoami/neo-angband-tracker/issues/236) `8e0f8448e` | `bdd3c2ea2` | `5053915f5` | #188 | `793f723eb` | `EXCAVATE-DARK-FLAG.md` |
| 0044 calllight | [231](https://github.com/diegoami/neo-angband-tracker/issues/231) `cda4fb662` | `6b4349733` | `85b6ba8e3` | #196 | `85de924fa` | `CALL-LIGHT-OWN-GRID.md` |
| 0045 gloves | [205](https://github.com/diegoami/neo-angband-tracker/issues/205) `c52527eed` | `175ef6feb` | `813feaec7` | #201 | `7ff787acc` | `GLOVES-ARMOUR-SWAP.md` |
| 0046 detectcarry | [219](https://github.com/diegoami/neo-angband-tracker/issues/219) `3d104a8b8` | `7be651828` | `7a02072cd` | #205 | `55e000236` | `DETECTION-CARRYOVER.md` |
| 0047 maimfoe | [256](https://github.com/diegoami/neo-angband-tracker/issues/256) `91822d82a` | `c61e90c9c` | `8a30a39d9` | #214 | `ea1443bd6` | `MAIMFOE-RETURN.md` |
| 0048 recalldepth | [310](https://github.com/diegoami/neo-angband-tracker/issues/310) `9609e2c86` | `b00f2a190` | `b9e64aa85` | #216 | `9ccd191c2` | `RECALLDEPTH-ANSWER.md` |
| 0049 flowcache | [404](https://github.com/diegoami/neo-angband-tracker/issues/404) `b71b32c15` | `257e7b473` | `1c8fcdccf` | #212 | `89326f521` | the recording lane's package (`artifacts/fc404-carry/`) |
| 0050 glowtrace | [213](https://github.com/diegoami/neo-angband-tracker/issues/213) `da9934216` | `6c00db351` | `5384f12ed` | #224 | `5edc188e4` | `GLOWTRACE-FLOORS.md` |

The source heads are single C-only commits against the pristine
baseline `e473720ed`; every applied commit's diff body was proven
byte-identical to its source diff, and every file with layered carried
patches (`borg-update.c` 0039/0046; `borg-fight-attack.c`; others) was
proven by **before module + source diff == product module**. Evidence
pins are the source receipts' fixture pins; the consumer verifications
used the source fixtures **unchanged**, with each receipt chain checked
(pin identity or sha256 map, per the source kit's schema). Patches
0043–0048 and 0050 are this lane's work; 0049 (flowcache, the D0008
grey-mold flow-cache correction) was carried and measured by the
recording lane and is included here because it is part of the same
runtime delta.

## Verification contract

1. **Unchanged-fixture four-phase verification** on actual product
   cores — normal-before / normal-after / asan-before / asan-after,
   fail-before/pass-after on the correction's observable, byte-identical
   controls, sanitized phases asserting (not suppressing) the pre-
   existing init-time `borg_sort_comp_hook` overflow (source finding
   [#399](https://github.com/diegoami/neo-angband-tracker/issues/399),
   separately owned) via `-fsanitize-recover=address` +
   `halt_on_error=0` — applies to **0043–0048 and 0050**. Per-carry
   fixture shapes (failing case + byte-identical controls): 0043 1+1,
   0044 1+1, 0045 1+2, 0046 1+1, 0047 1+2, 0048 1+2; 0050 runs one
   scenario whose requirements include the perm-wall control
   (never flagged glowing). No control changed in any carry.
2. **0049 (flowcache) uses its own contract**, documented in
   `FLOW-DANGER-CACHE.md`: a 150-scenario product matrix
   (5 variants × 30 cases) instead of the four-phase sanitizer
   comparison — `before` 19/30 with the 11 expected contract failures,
   `after` 30/30, two single-writer partial-fix negative controls
   failing exactly the other writer's cases (26/30 and 23/30), the
   `alt-wipe` alternative 30/30, and all `rng_touched` counters 0.
   It has **no ASan phase and therefore makes no #399 assertion**; its
   module was pristine before the carry, so its provenance proof is
   the simpler before-equals-base / after-equals-correction check.
3. **Layered provenance** for every touched module — the whole-file
   proof "before module + source diff == product module" (for
   `borg-update.c` the before module itself carries 0018/0039 plus,
   after mid-milestone, 0046); imported diff body byte-identical to
   the source diff.
3. **Native gate at the fix commit: 965/965 unit + 5/5 frontend**, read
   from logs; identical numerics on each PR's hosted run and each
   merged-main run. Tooling grew 71 → 97 across the milestone via
   concurrent-session work outside the build inputs.
4. **Accumulated regression matrix** re-run per carry against the
   recorded prior tally: 143 → 146 → 148 → 151 → 154 cases; the only
   non-identical cases remain the documented ones (dispel-248 undead
   asserted log change; two dispel-259 asserted-red at pinned
   post-losdispel values). **No prior fixture changed** across all
   eight carries, including the behavior-active 0050.
5. **Frozen v1 cohort** (seeds 11/29/47, cap 20000, wall 120, stall 30)
   on a prepared observer build of each merged main.

## Frozen v1 cohort honesty

- 0043–0045: behavior-neutral (whole-trace identical to the prior
  lineage run each time).
- 0046, 0047, 0048: behavior-neutral (whole-trace identical to the
  immediately preceding carry's run; the 34-carry v1 baseline remains
  diverged since mapwall-212, recorded in `MAP-WALL-CAPTURE.md`).
- 0049: cohort-neutral — the recording lane's own recorded v1
  before/after traces are identical on all three seeds
  (`artifacts/fc404-carry/v1-results.json`).
- **0050 is behavior-active**: seeds 11/29/47 all stay turn-cap alive
  and well (no deaths), but whole traces diverge from the
  pre-glowtrace lineage (+3.2k/−5.1k lines per seed). Attribution is
  clean — the only interleaved carry (0049) was proven neutral — so
  the divergence is the restored Floors branch itself (glowing floors
  flagged; flow-toward-light 0 → 484 targets on the evidence fixture).
  Recorded as observed; no survival claim in either direction; the
  cohort's lineage restarts here.

## Cross-repo reproduction strength (new in this milestone)

For 0047 (maimfoe), 0048 (recalldepth) and 0050 (glowtrace), the
consumer's recorded fixture outputs match the source evidence kits
byte-for-byte (log SHA256s) or value-for-value (recorded STATE rows
including grid info bitfields) — the same unchanged fixture run on the
consumer core reproduces the source run exactly.

## Exclusions and pending decisions (checked, not counted)

- **tapunlife-249** stays blocked, not counted (masker: carried patch
  0022; diagnosis in `TAP-UNLIFE.md`).
- **tpreader-250** is duplicate-excluded (byte-identical to carried
  patch 0012/source67; notice on source250).
- **dontreact-14** is conflict-assessed, not carried: it fixes the same
  defect as carried patch 0002 (consumer #7) with the opposite
  mechanism (remove the static `borg_dont_react`, use `borg.dont_react`
  everywhere). Its upstream counterpart
  [angband/angband#6808](https://github.com/angband/angband/pull/6808)
  is **open, not merged, no formal review** — the maintainer-confirmed
  direction is comments only. Retiring 0002 is an owner decision,
  documented on source #14; the defect itself is fixed in the consumer
  by 0002 either way.
- **#13** (map-file prompt crash) has native evidence but no merged
  source fix PR; **#403** (zoo stair-cycle) and **#404's** broader
  D0008 death-analysis remain source-side; **#216-source** stays parked
  pending a geometry that exposes `hit_dist`.

## Full-disclosure index

Every carry's consumer issue records pins, reproduce commands, actual
output, numerics, measurement, and honest limits; per-carry evidence
archives (`evidence.tar.gz` + `archive-receipt.json` +
`archive-manifest.json` + `RECOVERY.txt`) live under `artifacts/` and
are hash-verified in the primary checkout. Retained red/invalid
attempts this milestone: detectcarry verifier-constant mistakes
(`scenarios-attempt5/6`), glowtrace's stale before-gate attempt
(`scenarios-attempt1-stale-before-gate`), and the recalldepth
series.json conflict-resolution incident (pushed invalid JSON before
validation, rebuilt programmatically in `6e1938f89`; full chain in
`artifacts/rd310-carry/RECOVERY.txt`). The detectcarry branch-base
correction (stale 44-carry base, reset before any fixture ran) is
disclosed on #205 with the stale artifacts retained. 0049's own
retained attempts (the first setup failure and the rejected
wrong-expected-head argument) are disclosed in
`FLOW-DANGER-CACHE.md` and on #212. No assertion was
weakened for green anywhere in this milestone; the `#399` overflow is
asserted as expected in every sanitized phase.
