# Glyph tracker carry: verified product scenario, ready for adoption

Consumer [48](https://github.com/diegoami/better-borg-c/issues/48), canonical
source [118](https://github.com/diegoami/neo-angband-tracker/issues/118)
(CBUG-0102). Fix applied as `40b002da088ac48e7c1322aca2159e22c593fd73`
(import of source PR283 `ec172531ec38fdd3ac2783d21e3faf6dc1e2b3f3`, patch
`patches/0010-glyph-tracker-guard.patch`). This branch verifies the fix on
the actual product core; merge follows the normal review/checks policy.

## Pins

Before build: nine-carry `19870e97d89afa4b302bdd9d27337ed617c97554`
(retained product tree `../per-spell`, gate 965/965 unit + 5/5 frontend,
binary SHA256 `e2ce175441e34bf071e4daf848543d6568a4b384d2dc30f1cb95492eaabbd689`).
Product core: binary SHA256
`4b66e88cd5f6a2b69637121dd0f214610474bff3e3553a52b014586e65f01474` —
identical in the first failed attempt, this verification and the ASan build,
proving no source drift since `40b002da`. Official Angband master was checked
this cycle and remains immutable baseline `e473720ed88a0f0c7ca66815cef3000eb4a38bc3`.
The fixed message module SHA256 is
`3189a4b646b8f7db4df82c88302acba015a4333332c995b8f367f7062cbafd69`; pristine
`e473720` + `patches/0010` reproduces it exactly (round trip re-verified
2026-11-06).

## Why the original product attempt failed (measured, not assumed)

The unchanged source fixture (`f1f53106526412cfe44fa01c73d00e7dbf4dbeed`,
source PR284) stalled at phaseB: `DANGER phaseB p1=266 p2=266`, both builds,
`DECISION phaseB acted=0 keys=0`, all three modes. A diagnostic observer
(`artifacts/glyph/diag-phaseb`, product objects, same state) decomposed the
danger with `borg_danger_one_kill` per-spell windows:

- Physical `v1 = 56`; spell `v2 = 266 = 121 × 2.2` (level-4 friends
  multiplier on `friends_base`), where **121 is a single BO_FIRE** (40 fear +
  81 damage) and the glyph-sensitive `S_MONSTER` contributes only 70
  (spell_power 28 × 5 / spot_safe 2; the vacated doorway still counts as a
  protected grid). `full_damage` is forced in `borg_danger`, so no frequency
  reduction applies; speeds 110/110 give q=10 and no speed boost.
- `borg_defend_aux_glyph` requires `p1 > p2 && p2 <= avoidance/2 (90) &&
  p1 > avoidance/7`. A rune does not stop ranged bolts, so with correct
  projection the glyph cannot reduce this danger: the refusal is the Borg's
  honest heuristic, not a defect.
- **The evidence fixture only ever passed phaseB because pristine
  `borg_projectable_pure()` broke at the caster's own tracked grid**,
  zeroing all bolt damage. The carried caster-shot-origin fix
  (`patches/0008`, consumer 40) corrected exactly that, so the unchanged
  fixture is inapplicable to the nine-carry product. Confirmed by
  measurement: the S_MONSTER-only window reproduces the source evidence's
  numbers exactly (no-glyph 154 = 70×2.2 with bolts zeroed; glyphed
  max-remaining 40 = BO_FIRE fear → 88), matching the recorded
  `p1=154 p2=88` pass. The per-spell carry (0009) does not change this
  scenario; the before and product builds behave identically here.

The original failed attempt, its 12 processes, object/binary hashes and the
ignored archive (`artifacts/glyph-stop/glyph-unverified-evidence.tar.gz`,
SHA256 `f8e110e0659be22ce90d595cefaba1b92a25e9861d4e560d5356c2edee6fd196`,
28 members) are retained unchanged under `artifacts/glyph/scenarios`.

## Adapted scenario and why it is legitimate

`tools/glyph-tracker-scenario.c` is the evidence fixture with one functional
change: the adjacent `mage` threat becomes **Uldor the Accursed at distance
2** — a visible, non-friends, speed-110 unique whose only ranged spell is
`S_KIN` (no damaging spells, no SHAPECHANGE, invisible-base families like
quylthulgs excluded). At distance 2 with q=10 his physical danger is zeroed
(`q == 10 && d > 1`), so the glyph-sensitive summon dominates legitimately:
measured `p1=147 p2=63`, inside the unique-adjusted gate
(`63 <= 120`, `147 > 25`). Every phase, control, mode, intervention,
assertion and threshold is byte-for-byte the evidence fixture's; no game
rule changed. The emergency phaseA read, the real
"There is no clear floor on which to cast the spell." message, the real
terminal scrape and the defended creation path are unchanged.

## Verified results (`artifacts/glyph/adapted`, 2026-11-06)

Command (under `/tmp/angband-validation.lock`):

```sh
python3 tools/verify-glyph-tracker.py \
  --before-build ../per-spell/build/native \
  --output artifacts/glyph/adapted
```

- normal-before `stairs`: exit 0, 0/1, `phaseA num=-1`, `phaseB num=0`,
  `num_final=0`, glyph created on the native grid.
- normal-after `stairs`: exit 0, 1/1, `0 / 1 / 1`; DECISION lines identical
  before/after.
- `clear-floor` and `pairing` controls: 1/1 on every phase, byte-identical
  logs before/after (identical SHA256 pairs in `result.json`).
- asan-before `stairs`: exit 1, **heap-buffer-overflow WRITE of size 4** in
  `borg_defend_aux_glyph` (`borg-fight-defend.c:1607`), allocation stack
  `borg_init_track`/`borg_init_flow_glyph`; controls clean.
- asan-after: all three modes 1/1, no sanitizer report.
- `result.json` records `verified: true` with all commands, binary/object
  hashes and the adapted/original fixture SHA256s.

Regression matrix (`artifacts/glyph-carry/regression.py`): all **60 prior
native fixture cases re-run against this core are whole-log identical** to
the nine-carry baselines — 49 older cases (swap 9, knockout 4, preparation
9, cold-protection 6, rings 7, potion-timing 9, itangast 5), 6 source279
shot-path cases, 5 source64 per-spell cases. 60 cumulative fixture cases,
not 60 independent encounters. Tooling 29/29. The normal gate at this core
(965/965 + 5/5) is the recorded `40b002da` run in `artifacts/native`; the
exact-head hosted check on the PR covers the branch tip (docs/tools only
beyond the verified core).

## Scope and limits

Bounded native producer/parser scenario, not autonomous reachability or a
natural death/frequency claim. ASan runs use
`detect_leaks=0:intercept_strcmp=0` (unrelated pristine name-sort strcmp
overread, disclosed in the source evidence); the tracked write is
compiler-instrumented. The diagnostic observer temporarily windows the
in-memory `borg_kills` spell list (always restored) and toggles
`borg_on_glyph`; no product source or persistent state is modified. Danger
figures are the Borg's heuristic, not HP predictions. The failed original
attempt is retained; nothing was weakened to obtain green.

## Exact next work

1. PR from `task/carry-glyph-tracker`, agent evidence review plus required
   hosted checks, then normal merge (no per-PR external review stop).
2. On merged main: measure actual frozen v1 seeds 11/29/47, cap 20000,
   wall 120, stall 30; keep all outcomes.
3. Mark 48 carried/open+patch-active; post the Verification section with
   actual numbers; reply on source 118; checkpoint receipts on issue 1.
   Evidence archive with manifest under `artifacts/glyph-carry/`.
4. Then recheck live source67 readiness (medium-priority skip-list
   correction) before low-priority 81 or discretionary death work.
