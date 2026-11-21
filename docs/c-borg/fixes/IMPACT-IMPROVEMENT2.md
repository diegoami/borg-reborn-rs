# Improvement I-002 impact record

Unit: let `borg_caution()` heal while fleeing at survivable danger
([#408](https://github.com/diegoami/better-borg-c/issues/408), contract and
evidence there; ledger `patches/improvements.json`; merged `861ecb2da`,
PR #414). Local-only improvement, never proposed upstream (charter,
PR #389). Owner direction 2026-11-10: "Fix 408".

Post-merge confirmation (merged head `861ecb2da`, the tooling-only PR #413
also on it): frozen cohort-v1 rerun - 6/6 runs alive at cap, every per-seed
final and whole-trace sha256 identical to the tested branch build (seeds
11/29/47), so the merge carried exactly the tested behavior.

## Change

One 7-line `else if` in `borg_caution()`: while `goal.fleeing` is set, if
the current grid is survivable this round (`pos_danger < borg.trait[BI_CURHP]`,
the file's own scale - excessive danger is `pos_danger > 2 * HP`), still offer
`borg_heal(pos_danger)`. `borg_defend()` stays gated off while fleeing
(issue question 4: separate unit). `borg_heal()`'s own internal gates are
untouched (no healing at danger 0 unless poisoned/cut; cures bail at
danger > 2 * avoidance; tier windows unchanged, #319 untouched). The flee
goal itself is untouched.

Threshold note: `borg.avoidance` resets to `CURHP` every `borg_update()`,
so an avoidance-scaled low-danger test would collapse to 0 exactly at
low-HP states like the recorded D0017 one (HP 7, danger 1) - measured out
before choosing.

## Upstream history (issue question 1)

The `if (!goal_fleeing)` wrapper is unchanged since APW's original borg
import (a50705002, 2011 file set, `src/borg/borg6.c`), through Borg 4x
(80018afaf) and the 2023 reorg (1665c15db). No commit or comment states a
rationale beyond placement: "try healing before running away". The
level-long duration comes from `goal.fleeing` being cleared only on a depth
change (`borg-update.c:2191`). No upstream evidence says a safe, alone,
poisoned borg must not cure.

## Prevalence on recorded lives (issue question 2)

The death-anchored event windows of the 25 collected dossiers in the working
record (D0006-D0030) each contain at least one `# Fleeing` note when matched
case-insensitively. Only 14 of those dossiers (D0006-D0019) are committed in
this repository; D0020-D0030 are local death records, so this is a
working-record consistency check, not a measurement reproducible from the
repository. A turn-span recount on 2026-11-10 finds the last fleeing note in
the final 5% of the window in 18/25; the original survey recorded 20/25 and
the difference sits at the 5% boundary. The windows are death-anchored by
design, so neither figure is a frequency estimate and no decision here
depends on the fraction. Case-level: D0017
(poison death while safe and alone, 9 CCW + Herbal Curing in pack), D0016
(crow kill at 5/12 HP with 2 known CLW), D0012 (healing call skipped in the
grey-mold loop). A per-turn frequency estimate would need new observer
instrumentation and is not part of this unit.

## Contract and acceptance

Contract posted before implementation (2026-11-10, comment
6756334861). Status per item:

1. **Native gate**: green locally on the branch build (965/965 + 5/5,
   `flock /tmp/angband-validation.lock python3 tools/verify-native.py`);
   hosted CI on the PR.
2. **Targeted evidence - scenario fixture** (`tools/fleeing-heal-scenario.c`,
   matrix group `fleeheal-408`, baseline before-build logs in
   `artifacts/fleeheal-408/`):
   - `flee-low` drives the production chooser at the recorded D0017 fatal
     decision state (Dwarf Ranger DL21, HP 7/230, SP 19/39, poison 485
     turns, `goal.fleeing` set, recall read, empty safe grid, 9 CCW +
     Herbal Curing learned). Before build (`ac0cc65c5`): the recorded death
     decision - rest-for-recall keys `R500<enter>`, `heal=0 rest=1`, fixture
     fails its after-contract (0/1). After build: the Herbal Curing cure is
     queued (`sequence=mac`, `heal=1 rest=0`), 1/1. Fail-before / pass-after
     at the recorded state values.
   - Controls byte-identical across the builds: `flee-highdanger` (awake
     adjacent hill giant: the new branch stays closed; the queued action is
     the tail's existing recall buy-time CCW quaff `qc`, shared code),
     `noflee-wounded` (the existing not-fleeing path heals `mac` on both),
     `flee-full` (healthy fleeing borg rests `R500` on both).
3. **Regression matrix**: 234/234 cases pass, zero failed groups, including
   the new `fleeheal-408` group pinned by `rows_must` with `flee-low` marked
   `rows_free` as the intentional improvement transition; no unexpected
   changes anywhere.
4. **Bounded paired cohort** (cohort-v1, Human Warrior, seeds 11/29/47,
   20k turns, fixed birth; old `ac0cc65c5` vs new `dfecabb59` binaries,
   product default config): 6/6 runs alive at cap on both builds - no
   survival regression. Seeds 29/47 identical finals (6/6 hp 83 CL 8 depth
   6; 5/max 6 hp 109 CL 8). Seed 11: first divergence at turn 4481
   (up-stairs flow) with **zero heal/cure notes in either life** - the
   shift is the branch's extra borg-RNG draw (borg_heal's `randint0(100)`
   chance roll), not a spent consumable; final depth 4/CL 6/HP 54 versus
   the old life's depth 5/CL 7/HP 72, same max depth 6. Direction
   unfavorable on that one seed; at n=3 this is a directional, inconclusive
   progress comparison, reported as such per the workflow.
5. **Paired targeted branch runs** (seed 2993630922 `--random-birth`, the
   D0017 seed, cap 1.2M): start.save sha256 equals the dossier's recorded
   start (`59ae121c...`) on both binaries. Under the product default config
   (which since `fd683cedb`, 2026-11-08, gives Warriors/Druids/Rangers the
   by-class munchkin start) neither life is the recorded normal-mode life:
   both are munchkin-styled, the branch fired (borg-RNG stream shift, first
   note difference at turn 1122: scary-squares sampling 15 vs 17) but
   produced **zero** heal/cure actions in either life, and both finished
   `turn_cap` at 1,200,000 alive with identical decision counts and game
   RNG hash. **Contract deviation, reported honestly:** the recorded life
   is not replayable on current main at all - 181 behavior-active commits
   separate `ac0cc65c5` from the recorded build `5cd2258dd`, and the
   by-class munchkin default changes the mode; the measured driver
   hardcodes its borg.txt (no config override), so a normal-mode rerun of
   the recorded trajectory was not possible. The fatal state itself is
   covered by the scenario fixture instead.

## Mechanism claim and limits

With this change, the recorded D0017 terminal decision state is survivable:
caution cures the poison (Herbal Curing, 6 SP of 19 held) instead of
resting over it at 1 HP per turn - proven at the recorded state values by
the fixture. Live survival/progress benefit in ordinary play is directional
only: the paired cohort shows no survival regression and one seed with
slightly shallower progress at cap (RNG-shifted trajectory, no consumable
spent); n=3 cannot establish or exclude a population effect. Known honest
cost: while fleeing at survivable danger the borg may spend a consumable
and a turn it did not spend before, and every branch firing shifts the
borg-RNG stream (traces diverge; no decision identity is claimed past the
first firing).

Not in this unit (each would be its own contract): clearing `goal.fleeing`
after a safe escape; recall-rest declining while poisoned/cut; any
`borg_defend` change (#340).
