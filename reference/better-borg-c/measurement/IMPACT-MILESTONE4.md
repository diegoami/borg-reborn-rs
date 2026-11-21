# Native C bug-fix v4 — accumulated review package

[Milestone4](https://github.com/diegoami/better-borg-c/milestone/4) and the
review issue collect the bug-fix carries after the reviewed
[impact v3](IMPACT-MILESTONE3.md) package. The prior runtime snapshot is
**702f091f6ac06cae1014df2f85ac218be0189813** (34 carries, patches
0001-0034, v3 owner-approved). This package pins the runtime snapshot
after the final carry at **70993393a1c946ce8cd38ee558bf1ce6464350e7**
(42 carries, patches 0001-0042). The pristine baseline remains
**e473720ed88a0f0c7ca66815cef3000eb4a38bc3**. No external verdict is
claimed; the owner launches the review.

## Scope

This milestone counts **eight independently verified C bug carries**,
patches 0035-0042, consumer issues 133, 137, 141, 144, 148, 157, 167 and
176. Together with the 34 previously carried fixes, the product now has 42
carried/open+patch-active corrections. Process, documentation and tooling
merges in the same interval are listed as context and are not counted.

Exact source head, imported commit, evidence pin, credit and removal unit
for every correction are in [series.json](../../patches/series.json).
Canonical defect ownership remains in neo-angband-tracker; no source issue
is marked resolved by a consumer merge.

## The eight carries

| Patch | Source / consumer / merge | Demonstrated correction | Report |
| --- | --- | --- | --- |
| 0035 | [240](https://github.com/diegoami/neo-angband-tracker/issues/240) / [133](https://github.com/diegoami/better-borg-c/issues/133) / [PR135](https://github.com/diegoami/better-borg-c/pull/135) `638513e91` | Brand-ammo need counted once per stack (20 → 10) | [BRAND-NEED](BRAND-NEED.md) |
| 0036 | [232](https://github.com/diegoami/neo-angband-tracker/issues/232) / [137](https://github.com/diegoami/better-borg-c/issues/137) / [PR138](https://github.com/diegoami/better-borg-c/pull/138) `4fc3c481f` | Known up-staircase no longer re-tracked on every visit | [STAIRTRACK](STAIRTRACK.md) |
| 0037 | [252](https://github.com/diegoami/neo-angband-tracker/issues/252) / [141](https://github.com/diegoami/better-borg-c/issues/141) / [PR142](https://github.com/diegoami/better-borg-c/pull/142) `d619eec5f` | Ball/arc blast scans include the south/east radius edges | [ACID-ARC-BOUNDS](ACID-ARC-BOUNDS.md) |
| 0038 | [229](https://github.com/diegoami/neo-angband-tracker/issues/229) / [144](https://github.com/diegoami/better-borg-c/issues/144) / [PR145](https://github.com/diegoami/better-borg-c/pull/145) `04ab89381` | Dying light starts the out-of-light maneuvers | [OUT-OF-LIGHT-GATE](OUT-OF-LIGHT-GATE.md) |
| 0039 | [212](https://github.com/diegoami/neo-angband-tracker/issues/212) / [148](https://github.com/diegoami/better-borg-c/issues/148) / [PR149](https://github.com/diegoami/better-borg-c/pull/149) `f0288cca5` | Map-pass wall-change detection actually fires | [MAP-WALL-CAPTURE](MAP-WALL-CAPTURE.md) |
| 0040 | [81](https://github.com/diegoami/neo-angband-tracker/issues/81) / [157](https://github.com/diegoami/better-borg-c/issues/157) / [PR160](https://github.com/diegoami/better-borg-c/pull/160) `503e7a07b` | Recognized Enlightenment potions are drunk in the dungeon | [ENLIGHTENMENT-USE](ENLIGHTENMENT-USE.md) |
| 0041 | [230](https://github.com/diegoami/neo-angband-tracker/issues/230) / [167](https://github.com/diegoami/better-borg-c/issues/167) / [PR171](https://github.com/diegoami/better-borg-c/pull/171) `bc8cf805f` | Out-of-light stair command reports acting | [STAIR-RETURN-REPORT](STAIR-RETURN-REPORT.md) |
| 0042 | [247](https://github.com/diegoami/neo-angband-tracker/issues/247) / [176](https://github.com/diegoami/better-borg-c/issues/176) / [PR179](https://github.com/diegoami/better-borg-c/pull/179) `70993393a` | Blow-name lookup stops before its NULL sentinel (crash class) | [BLOW-NAME-SENTINEL](BLOW-NAME-SENTINEL.md) |

## Verification contract used by every carry

- A distinct consumer issue and normal PR recorded the reviewed source
  head, imported commit, evidence pin, original credit and patch removal
  unit.
- For layered modules, provenance is mechanical: the reviewed source diff
  applied to the before module equals the product module (applied for
  borg-trait.c over 0017/0029/0031, borg-think-dungeon-util.c over
  0032/0036/0038/0041, borg-fight-attack.c over twelve patches,
  borg-update.c over 0018, borg-item-use.c over 0005/0006); otherwise the
  module was pristine. Each applied diff body is content-identical to its
  source commit's diff; where the consumer module had pre-existing drift
  from other carried fixes the hunk-header line numbers are adjusted and
  the +/- lines are byte-identical (byte-identical throughout for 0038).
- Functional defects use actual fail-before/pass-after runs; the one crash
  defect (0042) uses the SIGSEGV itself as fail-before with the sanitizer
  naming the NULL-sentinel dereference directly.
- Unaffected controls remain byte-identical unless a documented
  cross-carry interaction is explicitly asserted with its cause.
- The unchanged source evidence fixture ran unchanged in every carry.
  Each carry's retained evidence (fixture runs, matrix runs, measurement
  builds, frozen v1 runs) is archived with per-member SHA256 manifests and
  copied into the primary checkout's `artifacts/` (b240, s232, a252, o229,
  m212, e065, s230, b247), following the v3 evidence-durability practice.
- Retained red/invalid attempts stay in the archives: blowname's first
  fixture attempt (plain ASan options halted at the pre-existing init
  overflow), tapunlife's blocked attempts, and per-carry tooling/build
  misconfigurations, all diagnosed without weakening any product assertion.

## Reviewed results

- The accumulated regression matrix was re-run at every carry against the
  then-current product core, growing **115 → 129 → 135 → 138** prior cases
  as fixtures accumulated (every prior case plus each new carry's cases in
  the next carry's run). All tallies match their recorded values: whole-log
  identical output, the dispel-248 undead case passing with its asserted
  changed log, and the two dispel-259 cases **asserted-red** at pinned
  post-losdispel values. The accumulated runner and ASan fixture coverage
  are local-only; they are not part of hosted CI.
- Hosted checks were read numerically for every carry PR and merged main:
  **965/965 unit and 5/5 frontend** throughout. Tooling grew with
  concurrent-session work unrelated to these fixes — 41 at v3, 54 with the
  recording/replay feature, 56 with death-phase feedback docs, 66 with the
  death-documentation tests, 71 on current main — each count read from the
  carry receipts at their recorded main.
- Every carry reran frozen v1 seeds 11/29/47 with cap 20000, wall 120,
  stall 30 on a prepared observer build of its merged main. Outcomes were
  **3/3 turn-cap survivors in every run** — no deaths, no survival
  regression anywhere in the interval.
- **mapwall-212 (0039) is behavior-active**: it is the first carry whose
  fix changes decisions on ordinary maps, and the v1 traces legitimately
  diverge from the 34-carry recording (seed11 and seed47 substantially,
  seed29 barely; examined divergence points are additional or re-ordered
  flow notes after ordinary feature changes). Carries 0040-0042 are
  byte-identical to the mapwall-lineage run. This is the measured impact
  of the fix, recorded rather than asserted away; it is a decision-stream
  change, not a survival, speed or population-performance claim.

Known interactions and findings that reviewers should check rather than
rediscover:

- The two dispel-259 asserted-red transitions and the dispel-248 asserted
  changed log continue from v3 unchanged through every run in this
  interval.
- **tapunlife-249 was attempted and is blocked, not counted**: the
  unchanged fixture produces byte-identical before/after logs on this
  product. A completed bisection — closed after this package's external
  review with a step reverting the masker alone — identifies carried
  patch 0022 (losdispel-326) as the masker: the rad-10 walk the tap-ray
  model uses collected per-step path damage on pristine and the misrating
  (dam = target power) dominated; 0022's arrival break converges the
  valuations so the injured bat's kill bonus wins with the misrating
  still present. Full bisection table (steps A-J, the last added to close
  the reviewer's M4-06 observation):

  | Step | Reverted patches | undead targeting |
  | --- | --- | --- |
  | A | none (36-carry as-is) | `* p 3 6 6 6 5 m c a` (injured; reproduces the red run) |
  | B | all ten fight-attack patches | `* p 2 2 2 2 2 2 5 m c a` (healthy; pristine behavior restored) |
  | C | 0024+0023+0022+0021+0020 | healthy |
  | D | 0019+0016+0014+0013+0011 | injured |
  | E | 0024+0023 | injured |
  | F | 0022+0021+0020 | healthy |
  | G | 0020 | injured |
  | H | 0022+0021 | healthy |
  | I | 0021 | injured |
  | J | 0022 alone | healthy — 0022 alone is the masker |

  The consumer issue [140](https://github.com/diegoami/better-borg-c/issues/140)
  retains the red attempt, the diagnosis (now landed as
  `docs/measurement/TAP-UNLIFE.md`) and the unmerged branch; the source
  demonstration stays valid at its recorded commits.
- **source #399 (new finding, not carried)**: blowname's sanitized runs
  surface a pre-existing init-time heap-buffer-overflow in
  `borg_sort_comp_hook` (borg-util.c:106) reachable in every sanitized
  `borg_init` of this pin. It is separately owned by the source lane
  (needs call-site identification, native reproduction and a source fix
  before any carry) and its reports are expected in this milestone's
  sanitized evidence.
- The queued-but-disowned stair key (0041) and the dying-light entry gate
  (0038) both sit in `borg_think_dungeon_util.c`'s out-of-light family;
  each carries its own removal unit and its own fixture.

Runtime changes between `702f091f6` and `70993393a` under `src/` are
confined to five Borg modules: `borg-fight-attack.c` (0037, 0042),
`borg-item-use.c` (0040), `borg-think-dungeon-util.c` (0036, 0038, 0041),
`borg-trait.c` (0035) and `borg-update.c` (0039). Each difference belongs
to one of the eight removal units in the ledger; no game-rule file outside
`src/borg` is part of this milestone.

## Process and tooling changes in the same interval

These are context for reviewers, not counted in the eight; they were
delivered by the concurrent recording/death sessions on the owner's
separate requests and changed nothing under the build inputs:

- [PR147](https://github.com/diegoami/better-borg-c/pull/147): the
  recording/replay feature (`replay-borg.py`, recording streams, D0004/
  D0005 dossiers); verified behavior-neutral on the frozen cohort
  (plain-vs-recorded controls identical) and outside `src/`.
- [PR156](https://github.com/diegoami/better-borg-c/pull/156),
  [PR161](https://github.com/diegoami/better-borg-c/pull/161),
  [PR163](https://github.com/diegoami/better-borg-c/pull/163),
  [PR164](https://github.com/diegoami/better-borg-c/pull/164),
  [PR165](https://github.com/diegoami/better-borg-c/pull/165) and
  [PR175](https://github.com/diegoami/better-borg-c/pull/175):
  death-phase feedback separation, brainstorming record, death
  documentation and replay-spell UI; documentation/observer work outside
  the build inputs.
- Tooling counts moved 41 → 71 across these merges as above; the tooling
  suite now takes the shared validation lock itself, so carry receipts run
  it outside a lock wrapper.

## Limits

Everything here is a bug-fix measurement, not a performance result: the v1
divergence after 0039 is a recorded decision-stream change with unchanged
survivor outcomes; no death, speed or population-performance claim is made
anywhere in this package. The eight issues remain carried/open+patch-active
pending accepted upstream corrections.
