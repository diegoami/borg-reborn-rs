# Teleport Other skip-list carry: verified product scenario, ready for adoption

Consumer [54](https://github.com/diegoami/better-borg-c/issues/54), canonical
source [67](https://github.com/diegoami/neo-angband-tracker/issues/67)
(CBUG-0051). Fix applied as `706fb49b0731ebaf54408c0699b4cd1da1a15929`
(import of source PR285 `25f8a43828204dd1226d5bf5c3b36547b4a07a40`, patch
`patches/0012-tp-other-skip.patch`).

## Defect

The Teleport Other skip-list loop in `borg_danger_one_kill()` read
`ii <= borg_tp_other_n`, one entry past the count. The Morgoth-depth defense
stores each listed monster through two branches, writing slot 1; the
per-candidate code resets only the count, not the slots. The later regular
evaluation then read the leftover slot and suppressed the monster it named,
inflating the cast's value by the weak monster's danger. No memory error:
the stale read stays inside the allocated array (canonical 250's unbounded
writer is unreachable in native C; the reader is this defect).

## Pins and layered provenance

Before build: eleven-carry `c3bc27e58bb1a720f509d9728a85d9e1814abc32`
(retained tree `../missile-no-target`, normal and ASan cores, gate at
`09dd4435`, source-identical to `c3bc27e58` over src/lib). Evidence
fixture: unchanged source pin `83cedb3fe`
(`docs/audit/c-reports/CBUG-0051/`, PR286). `borg-danger.c` already
carries cold-protection (0004) and per-spell (0009), so provenance is
proven structurally in the verifier: the before module + the source
one-comparison diff equals the product module (the stale line occurs
exactly once), and the patch round trip from the eleven-carry module
reproduces the product module SHA256
`dad9a3ae6f4954298ffa8a342df069f280ee1f29c05dcb7ab022c7109f70e2e8`.

## Verified results (`artifacts/tp/scenarios`, 2026-11-06)

Command (under `/tmp/angband-validation.lock`):

```sh
python3 tools/verify-tp-other-skip.py \
  --before-build ../missile-no-target/build/native \
  --before-asan-build ../missile-no-target/build/asan \
  --output artifacts/tp/scenarios
```

Product-accurate numbers (carried danger corrections shift the absolute
values; the relational contract is the fixture's own): the side hill giant
values **192** on this product (source evidence recorded 128 on pristine),
kobold 16.

- normal-before `two-phase`: exit 0, 0/1, `p2=208 = 192+16 ≠ expected 192`
  — the stale slot suppressed the giant during the kobold candidate's
  evaluation, same defect shape as the source's `144 = 128+16`.
- normal-after `two-phase`: 1/1, `p2=192 == expected`, `tp_other_n=0`.
- `no-morgoth-phase` (`192/192`) and `single` (`16/16`) controls: 1/1 on
  every phase, byte-identical logs before/after.
- asan-before/asan-after: identical behavior to the normal phases; no
  sanitizer report anywhere (the stale read is inside the allocated
  array), two-phase still fail-before/pass-after under ASan.
- `result.json` records `verified: true` with all commands and hashes.

Native gate on this core: **965/965 unit + 5/5 frontend**
(`artifacts/native/result.json`). Regression matrix
(`artifacts/tp-carry/regression.py`): all **66 prior native fixture cases
whole-log identical** to their baselines (49 older + 6 source279 +
5 source64 + 3 glyph + 3 missile). 66 cumulative fixture cases, not 66
independent encounters. Tooling 29/29.

## Scope and limits

Bounded native scenario on the real Teleport Other defense evaluation with
production `borg_launch_bolt` calls from both defenses; the temp fill is
replicated from the defense and `near.morgoth` is set directly to match the
borg's own depth-100 assumption (documented fixture prep in the source
evidence). Danger figures are the Borg's heuristic; no death, survival or
frequency claim. ASan options
`detect_leaks=0:intercept_strcmp=0` (disclosed pristine init overread).

## Exact next work

1. PR from `task/carry-tp-other-skip`, agent evidence review plus required
   hosted checks, then normal merge.
2. On merged main: frozen v1 measurement seeds 11/29/47, cap 20000, wall
   120, stall 30.
3. Mark 54 carried/open+patch-active; receipts on issue 54, source 67 and
   checkpoint 1; evidence archive under `artifacts/tp-carry/`.
4. Then recheck source248 (dispel rating) and source253 (arc angle)
   readiness — the remaining demonstrated valuation pairs — plus any newer
   demonstrations before claiming.
