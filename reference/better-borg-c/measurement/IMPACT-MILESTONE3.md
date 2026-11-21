# Native C bug-fix v3 — accumulated review package

[Milestone3](https://github.com/diegoami/better-borg-c/milestone/3) and
[review issue128](https://github.com/diegoami/better-borg-c/issues/128) cover
the bug-fix wave after [milestone2](IMPACT-MILESTONE2.md). The prior runtime
snapshot is **2675d198e4c3fcb1c69ee63ef25a2b6ac27ab96f** (seven carries,
patches 0001-0007). This package pins the runtime snapshot after the final
carry at **702f091f6ac06cae1014df2f85ac218be0189813**. The pristine baseline
remains **e473720ed88a0f0c7ca66815cef3000eb4a38bc3**. Owner-led accumulated
review completed on 2026-11-07: milestone2 was approved, and milestone3 was
approved with evidence-durability and metadata findings
([M2 report](https://github.com/diegoami/better-borg-c/issues/38#issuecomment-6705894080),
[M3 report](https://github.com/diegoami/better-borg-c/issues/128#issuecomment-6705894431)).
Follow-up [issue130](https://github.com/diegoami/better-borg-c/issues/130)
records those findings and this correction. This is not approval of upstream
incorporation and does not reinstate a per-PR review stop.

## Scope

This milestone counts **27 independently verified C bug carries**, patches
0008-0034, consumer issues 40, 44 and 48-124. Together with the seven
milestone2 fixes, the product now has 34 carried/open+patch-active
corrections. Process, documentation, runner and tooling PRs merged in the
same interval are listed below as review context and are not counted as bug
fixes.

Exact source head, imported commit, evidence pin, credit and removal unit for
every correction are in [series.json](../../patches/series.json). Canonical
defect ownership remains in neo-angband-tracker; no source issue is marked
resolved by a consumer merge.

## The 27 carries

| Patch | Source / consumer / merge | Demonstrated correction | Report |
| --- | --- | --- | --- |
| 0008 | [279](https://github.com/diegoami/neo-angband-tracker/issues/279) / [40](https://github.com/diegoami/better-borg-c/issues/40) / [PR42](https://github.com/diegoami/better-borg-c/pull/42) `4e2af89f3` | A caster no longer obstructs its own projectile; adverse escape outcomes remain asserted | [SHOT-PATH](SHOT-PATH.md) |
| 0009 | [64](https://github.com/diegoami/neo-angband-tracker/issues/64) / [44](https://github.com/diegoami/better-borg-c/issues/44) / [PR46](https://github.com/diegoami/better-borg-c/pull/46) `cf53f9b19` | Projectile classification resets per spell instead of leaking between attacks | [PER-SPELL](PER-SPELL.md) |
| 0010 | [118](https://github.com/diegoami/neo-angband-tracker/issues/118) / [48](https://github.com/diegoami/better-borg-c/issues/48) / [PR49](https://github.com/diegoami/better-borg-c/pull/49) `b72ad8eae` | Glyph tracking stays bounded after failed rune creation; ASan WRITE proof | [GLYPH-TRACKER](GLYPH-TRACKER.md) |
| 0011 | [254](https://github.com/diegoami/neo-angband-tracker/issues/254) / [51](https://github.com/diegoami/better-borg-c/issues/51) / [PR52](https://github.com/diegoami/better-borg-c/pull/52) `ce6435679` | A targetless missile probe cannot read item index -1 | [MISSILE-NO-TARGET](MISSILE-NO-TARGET.md) |
| 0012 | [67](https://github.com/diegoami/neo-angband-tracker/issues/67) / [54](https://github.com/diegoami/better-borg-c/issues/54) / [PR55](https://github.com/diegoami/better-borg-c/pull/55) `cfaff7be7` | Teleport Other skips only listed candidates, not one entry past the list | [TP-OTHER-SKIP](TP-OTHER-SKIP.md) |
| 0013 | [248](https://github.com/diegoami/neo-angband-tracker/issues/248) / [57](https://github.com/diegoami/better-borg-c/issues/57) / [PR58](https://github.com/diegoami/better-borg-c/pull/58) `c0174d583` | Dispel Life is rated against living monsters as the game does | [DISPEL-FILTER](DISPEL-FILTER.md) |
| 0014 | [253](https://github.com/diegoami/neo-angband-tracker/issues/253) / [60](https://github.com/diegoami/better-borg-c/issues/60) / [PR61](https://github.com/diegoami/better-borg-c/pull/61) `a89e96a15` | Arc targeting tests the scanned grid's angle, not the last scanned angle | [ARC-ANGLE](ARC-ANGLE.md) |
| 0015 | [244](https://github.com/diegoami/neo-angband-tracker/issues/244) / [63](https://github.com/diegoami/better-borg-c/issues/63) / [PR64](https://github.com/diegoami/better-borg-c/pull/64) `d28dbb165` | Munchkin stair tests reacquire the Borg's own grid after the adjacent scan | [MUNCHKIN-STAIR](MUNCHKIN-STAIR.md) |
| 0016 | [257](https://github.com/diegoami/neo-angband-tracker/issues/257) / [66](https://github.com/diegoami/better-borg-c/issues/66) / [PR67](https://github.com/diegoami/better-borg-c/pull/67) `dfddcfff9` | Vampire Strike guards the chosen target's distance, not the last scanned one | [VAMPIRE-STRIKE](VAMPIRE-STRIKE.md) |
| 0017 | [239](https://github.com/diegoami/neo-angband-tracker/issues/239) / [69](https://github.com/diegoami/better-borg-c/issues/69) / [PR70](https://github.com/diegoami/better-borg-c/pull/70) `a88da7a06` | Weapon slay traits accumulate across equipment slots instead of being erased | [SLAY-TRAITS](SLAY-TRAITS.md) |
| 0018 | [290](https://github.com/diegoami/neo-angband-tracker/issues/290) / [72](https://github.com/diegoami/better-borg-c/issues/72) / [PR73](https://github.com/diegoami/better-borg-c/pull/73) `3cd2c3237` | Queued detection reactions respect panel bounds; crash/ASan READ proof | [DETECT-PANEL](DETECT-PANEL.md) |
| 0019 | [260](https://github.com/diegoami/neo-angband-tracker/issues/260) / [75](https://github.com/diegoami/better-borg-c/issues/75) / [PR76](https://github.com/diegoami/better-borg-c/pull/76) `5ea7baed6` | Activation lookup uses the maneuver's actual activation index | [ACTIV-LOOKUP](ACTIV-LOOKUP.md) |
| 0020 | [258](https://github.com/diegoami/neo-angband-tracker/issues/258) / [78](https://github.com/diegoami/better-borg-c/issues/78) / [PR79](https://github.com/diegoami/better-borg-c/pull/79) `c7f5b5758` | Reserve-bolt hitpoint veto applies to near monsters, not the first scanned | [RESERVE-BOLT](RESERVE-BOLT.md) |
| 0021 | [259](https://github.com/diegoami/neo-angband-tracker/issues/259) / [81](https://github.com/diegoami/better-borg-c/issues/81) / [PR82](https://github.com/diegoami/better-borg-c/pull/82) `0c52dee46` | Teleport-reserve mana penalty is applied once, not twice | [DISPEL-RESERVE](DISPEL-RESERVE.md) |
| 0022 | [326](https://github.com/diegoami/neo-angband-tracker/issues/326) / [84](https://github.com/diegoami/better-borg-c/issues/84) / [PR85](https://github.com/diegoami/better-borg-c/pull/85) `2e55c43f8` | LOS-dispel beam walk breaks at arrival and the launcher returns a value | [LOSDISPEL](LOSDISPEL.md) |
| 0023 | [262](https://github.com/diegoami/neo-angband-tracker/issues/262) / [87](https://github.com/diegoami/better-borg-c/issues/87) / [PR88](https://github.com/diegoami/better-borg-c/pull/88) `aa87ca74d` | Staff holiness is valued as the area-of-sight dispel it activates | [STAFF-HOLY](STAFF-HOLY.md) |
| 0024 | [261](https://github.com/diegoami/neo-angband-tracker/issues/261) / [90](https://github.com/diegoami/better-borg-c/issues/90) / [PR91](https://github.com/diegoami/better-borg-c/pull/91) `f997c8cee` | Blind Creature's Frighten scores as fear, not phantom confusion | [BLIND-FRIGHTEN](BLIND-FRIGHTEN.md) |
| 0025 | [287](https://github.com/diegoami/neo-angband-tracker/issues/287) / [93](https://github.com/diegoami/better-borg-c/issues/93) / [PR94](https://github.com/diegoami/better-borg-c/pull/94) `b68eb2223` | Multibyte screen text terminates within its character limit | [TEXT-MULTIBYTE](TEXT-MULTIBYTE.md) |
| 0026 | [325](https://github.com/diegoami/neo-angband-tracker/issues/325) / [96](https://github.com/diegoami/better-borg-c/issues/96) / [PR97](https://github.com/diegoami/better-borg-c/pull/97) `0900c5cf4` | Reload releases the calculation backing array instead of overwriting it | [CALC-RELOAD](CALC-RELOAD.md) |
| 0027 | [322](https://github.com/diegoami/neo-angband-tracker/issues/322) / [99](https://github.com/diegoami/better-borg-c/issues/99) / [PR100](https://github.com/diegoami/better-borg-c/pull/100) `29190814b` | A stale same-square target cannot queue the abort-causing center key | [STALE-TARGET](STALE-TARGET.md) |
| 0028 | [311](https://github.com/diegoami/neo-angband-tracker/issues/311) / [102](https://github.com/diegoami/better-borg-c/issues/102) / [PR103](https://github.com/diegoami/better-borg-c/pull/103) `9aee81138` | Out-of-light glow collection respects its 1536-entry arrays; ASan WRITE proof | [GLOW-OVERFLOW](GLOW-OVERFLOW.md) |
| 0029 | [238](https://github.com/diegoami/neo-angband-tracker/issues/238) / [105](https://github.com/diegoami/better-borg-c/issues/105) / [PR107](https://github.com/diegoami/better-borg-c/pull/107) `958f9744b` | An artifact's light modifier is counted once | [LIGHT-MODIFIER](LIGHT-MODIFIER.md) |
| 0030 | [312](https://github.com/diegoami/neo-angband-tracker/issues/312) / [109](https://github.com/diegoami/better-borg-c/issues/109) / [PR110](https://github.com/diegoami/better-borg-c/pull/110) `75e92924f` | Trapped walls are classified once in the surrounded chain | [SURROUNDED-TRAPS](SURROUNDED-TRAPS.md) |
| 0031 | [313](https://github.com/diegoami/neo-angband-tracker/issues/313) / [113](https://github.com/diegoami/better-borg-c/issues/113) / [PR114](https://github.com/diegoami/better-borg-c/pull/114) `8a7eba1ad` | Home blows weigh one weapon, matching the engine on stacks | [HOME-BLOWS](HOME-BLOWS.md) |
| 0032 | [314](https://github.com/diegoami/neo-angband-tracker/issues/314) / [117](https://github.com/diegoami/better-borg-c/issues/117) / [PR118](https://github.com/diegoami/better-borg-c/pull/118) `4bfb44d73` | Town stair-scum bail clears both lunal flags | [LUNAL-STATE](LUNAL-STATE.md) |
| 0033 | [324](https://github.com/diegoami/neo-angband-tracker/issues/324) / [121](https://github.com/diegoami/better-borg-c/issues/121) / [PR122](https://github.com/diegoami/better-borg-c/pull/122) `6a00d4408` | Temporary operator-block pointer arrays are freed | [OPBLOCK-LEAK](OPBLOCK-LEAK.md) |
| 0034 | [323](https://github.com/diegoami/neo-angband-tracker/issues/323) / [124](https://github.com/diegoami/better-borg-c/issues/124) / [PR125](https://github.com/diegoami/better-borg-c/pull/125) `5fb5d1f74` | Token-list backing arrays are freed with calculations | [CALFREE-TOKENS](CALFREE-TOKENS.md) |

## Verification contract used by every carry

- A distinct consumer issue and normal PR recorded the reviewed source head,
  imported commit, evidence pin, original credit and patch removal unit.
- For layered modules, provenance is mechanical: the reviewed source diff
  applied to the before-module equals the product module; otherwise the
  module was pristine before that import.
- Functional defects use actual fail-before/pass-after runs. Ownership and
  leak defects can keep byte-identical functional logs and instead require
  stack-specific AddressSanitizer/LeakSanitizer allocation proof.
- Unaffected controls remain byte-identical unless a documented cross-carry
  interaction is explicitly asserted with its cause.
- Adapted fixtures carry provenance headers, and the original red attempts
  are retained. No assertion, seed, cap, timeout or control was weakened for
  green.
- The final accumulated matrix on the 34-carry runtime snapshot records
  **115 native fixture cases**: **113 pass** — 112 with whole-log identical
  output and one dispel-248 case passing with an asserted changed log — while
  the two dispel-259 cases are **asserted-red** at pinned post-losdispel
  values (`-10627→-10331` and `-127→169`). This is deliberate interaction
  checking, not silent drift and not 115 green cases. The accumulated runner
  and ASan fixture coverage are local-only; they are not part of hosted CI.
  Final hosted checks read numerically: **41/41 tooling, 965/965 unit,
  5/5 frontend**.
- A post-review final-runtime LeakSanitizer receipt
  (`artifacts/formula-leaks-final/`) runs the unchanged calc-reload-325,
  opblock-324 and calfree-323 fixtures against the ASan build source-identical
  to `702f091f6`: all three pass with **no leaks reported**, while the retained
  pre-fix calc-reload binary still reports **320 bytes in 4 allocations** as a
  positive control.
- Every bug-fix merge reran frozen v1 seeds 11/29/47 with cap20000,
  wall120 and stall30, and all reported 3/3 turn-cap survivors. The first
  carry in this interval (0008) left seeds11/29 byte-identical and changed
  exactly eight danger-estimate lines on seed47 with every other recorded
  line/state/RNG value unchanged. Every later bug-fix merge matched its
  immediately preceding runtime snapshot byte-for-byte on all three starts.
  These are regression observations only, not a survival, speed or
  population-performance claim.

Known layered interactions that reviewers should check rather than rediscover:

- losdispel-326 changes dispel-248's undead refusal value and adds the
  previously leaked contribution to both dispel-259 modes; both transitions
  are asserted in every later matrix.
- blind-261's pinned control `-5` was pristine; after losdispel-326 the
  product measures `-4`, so the control expectation is production-derived.
- opblock-324 shifts calfree-323's before baseline from pristine 160 bytes/2
  allocations to 80/1; the final product reports no leaks for all three
  formula-memory fixtures.
- PR116 increased tooling coverage from 29 to 41 independently of these bug
  fixes; the new tests are named-headless runner/death-record controls.

Runtime changes between `2675d198e` and `702f091f6` under `src/` are confined
to twelve Borg modules: `borg-danger.c`, `borg-escape.c`,
`borg-fight-attack.c`, `borg-flow-misc.c`, `borg-formulas-calc.c`,
`borg-io.c`, `borg-messages.c`, `borg-projection.c`,
`borg-think-dungeon-util.c`, `borg-think-dungeon.c`, `borg-trait.c` and
`borg-update.c`. Each difference belongs to one of the 27 removal units in
the ledger; no game-rule file outside `src/borg` is part of this milestone.

## Process and tooling changes in the same interval

These are context for reviewers, not counted in the 27:

- [PR43](https://github.com/diegoami/better-borg-c/pull/43): durable
  owner-feedback process and per-death discussion links.
- [PR45](https://github.com/diegoami/better-borg-c/pull/45): active-session
  death-comment checks at start/stop.
- [PR47](https://github.com/diegoami/better-borg-c/pull/47): wider replay
  windows and earlier event anchors, preserving historical records.
- [PR106](https://github.com/diegoami/better-borg-c/pull/106): visible
  launcher script and README changes only.
- [PR116](https://github.com/diegoami/better-borg-c/pull/116): named headless
  runner, backup recovery, D0003 dossier and tooling/death-record tests; it
  changes observer/tooling code, not Borg gameplay source.
- [PR119](https://github.com/diegoami/better-borg-c/pull/119): groups D0003
  with the existing crowd-exposure action in the death registry.
- [PR127](https://github.com/diegoami/better-borg-c/pull/127): records the
  carry-pipeline lessons learned.
- Numerous state PRs record only PLAN/session metadata after each merge.

## Review questions

1. Are the 27 defect arguments and their measured effects correctly scoped,
   including adverse outcomes where a correction is not uniformly safer?
2. Are before/after/control conditions strong enough for each claim, and are
   the functional-versus-ownership distinctions sound?
3. Are the documented cross-carry interactions expected consequences rather
   than regressions or weakened assertions?
4. Do patch removal units and layered provenance reconstruct every product
   module from its reviewed source diffs?
5. Are retained failures (red fixtures, invalid premerge measurement,
   verifier grouping error, observer/tooling failures) sufficient to audit
   what was not proved?
6. Are the frozen-seed comparisons correctly limited to regression evidence?
7. Is the patch-retirement path clear once upstream incorporates canonical
   fixes, including source ownership and official-master reconciliation?

## Evidence access and limits

Every consumer issue linked above contains its verification comment with
commands, actual results, archive size/member count and SHA256. Persistent
evidence archives for all 34 carries are outside Git under the primary
checkout's ignored `artifacts/` tree. On 2026-11-08, follow-up
[issue130](https://github.com/diegoami/better-borg-c/issues/130) copied the
25 milestone3 carry evidence directories (patches 0010-0034) from retained
task worktrees into that primary tree — 1,122,191,729 archive bytes and 9,390
members — re-checked every archive against its sidecar and receipt, and wrote
a per-directory `RECOVERY.txt` plus the aggregate
`artifacts/milestone3-evidence-recovery/manifest.json`. No originating
worktree was modified or removed.

Git carries the per-carry verifier tools, patch ledger, reports and exact
pins needed to rebuild and re-run individual controlled scenarios. The
accumulated 115-case matrix runner itself remains local-only and is preserved
as ignored `artifacts/c323-carry/regression.py`; hosted CI runs the tooling,
unit and frontend suites but no Borg fixture or ASan scenario. No workstation
`/tmp` path is part of recovery. Official upstream master was still
`e473720e` at the latest review check; all 34 consumer issues remain
carried/open+patch-active pending upstream incorporation. No release, tag,
official post, Rust integration, private-data operation or new gameplay
strategy is part of this milestone.
