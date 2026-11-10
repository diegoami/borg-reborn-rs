# Decision fixtures (T4.1, #118; T4.2, #119; T4.3, #120)

*Death-derived `Observation` fixtures. Each one records a decision the Borg must not make, and the verified feasible alternative where a branch exists. The scenario files, with their citations and construction notes, are in [`borg-reborn-rs/tests/fixtures/decisions/`](../borg-reborn-rs/tests/fixtures/decisions/); the engine data they read is [`knowledge.json`](../borg-reborn-rs/tests/fixtures/knowledge.json); the format is in [`tests/fixtures/README.md`](../borg-reborn-rs/tests/fixtures/README.md). A **gap** is not a skip: `tests/decision_fixtures.rs` pins what the Borg does today and fails when the behaviour changes, and the fixture names the open issue that owns the missing capability.*

| Fixture | Dossier | Forbidden | Feasible / verified alternative | Status | Now (pinned) |
| --- | --- | --- | --- | --- | --- |
| `bbc-d0002-gallant-at-three-hp` — D0002: 3/49 HP, a gallant adjacent, twelve known cures | `reference/deaths/better-borg-c/D0002-gallant` | Attack — the gallant, the fatal melee at 3 HP | Quaff — Cure Light Wounds (12 known) | gap (#122) | Attack **(still the fatal decision)** |
| `bbc-d0006-deep-descent-readiness` — D0006: north into the dark with pursuers and no way home | `reference/deaths/better-borg-c/D0006-collected` | Walk(8) — north into the unknown; Walk(9) — north-east into the unknown; Walk(7) — north-west into the unknown; Descend | Walk(2) — south through the open door | gap (#123) | Walk(7) **(still the fatal decision)** |
| `bbc-d0008-mold-stair-loop` — D0008: the grey mold beside the up staircase | `reference/deaths/better-borg-c/D0008-collected` | Ascend | Attack(6) — the 2-HP grey mold east of the player | active | — |
| `bbc-d0009-full-pack-starvation` — D0009: starving with a full pack and an unknown wand underfoot | `reference/deaths/better-borg-c/D0009-collected` | Get — the Bronze Wand underfoot at 23/23 slots | Eat — food, or leave for town; Ascend — to town | gap (#122) | Rest |
| `bbc-d0010-shoot-instead-of-stairs` — D0010: 1/15 HP, a kobold adjacent, a bow and eleven arrows | `reference/deaths/better-borg-c/D0010-collected` | Ascend; Read — the unknown scroll at 1 HP | Shoot — the kobold with the short bow (11 arrows); Attack — the kobold **Verified:** owner-manual-branch.json: 6255 -> 7303, CL2 HP12 D1; six arrow hits and one miss; the kobold was shot dead at 7293; both enemies killed. | gap (#127) | Walk |
| `bbc-d0011-dagger-for-maul` — D0011: the Dagger instead of the carried Maul, with known cures | `reference/deaths/better-borg-c/D0011-collected` | Attack — the griffon with the Dagger instead of the Maul | Wield — the carried Maul; Quaff — Cure Light Wounds | gap (#122) | Attack **(still the fatal decision)** |
| `bbc-d0012-kill-the-mold` — D0012: the owner branch kills the 2-HP mold in one blow | `reference/deaths/better-borg-c/D0012-collected` | Walk; Descend | Attack(4) — the 2-HP grey mold at (3,3) **Verified:** owner-manual-branch.json: 27526 one blow destroys the grey mold at 17 HP; rest to 21/21 by 27751; two white worm masses killed; saved 28253 at CL2 HP17. | active | — |
| `bbc-d0013-kobold-not-level-flight` — D0013: a small kobold is not a reason to leave the level | `reference/deaths/better-borg-c/D0013-collected` | Ascend | Attack — the small kobold, as the owner branch did; Walk — or step back **Verified:** owner-manual-branch.json: 3651 -> 4709, CL1 HP13 D0. The kobold fled after one hit; the ant was tested 12->6; town reached at 3 HP and rested to 13/13. | active | — |
| `bbc-d0014-speed-and-clw` — D0014: the last Phase at full HP, Speed and CLW unused | `reference/deaths/better-borg-c/D0014-collected` | Descend; Read — the last Phase Door at full HP | Quaff — Speed, then Cure Light Wounds **Verified:** owner-manual-branch.json: 306703 no Phase read; Speed on sight; both falcons one-hit; CLW cured blindness/confusion/bleeding; Detect Monsters; saved 306980 HP88/91. | gap (#122) | Attack |
| `bbc-d0019-one-apprentice-not-flight` — D0019: one apprentice on DL2 is not a reason to leave the level | `reference/deaths/better-borg-c/D0019-collected` | Ascend — whole-level flight over one apprentice | Walk — explore or fight, not the up staircase | active | — |
| `breeder-pack-one-action` — mmx-war-1: eleven visible silver mice, one action per think | `reference/borg-runner/records/mmx-war-1/REPORT.md` | Rest; Hold | Attack — one mouse, one action; Walk — one step away, one action | active | — |
| `fang-ac36-armour-ignoring-spell` — minimaxflash-2.2: Fang AC 36, oil is a 12% lottery | `reference/borg-runner/records/minimaxflash-2.2/REPORT.md` | Attack — Fang in melee or with oil at 4 HP | Cast — Magic Missile (1 SP, ignores armour); Read — the Phase Door scroll at 4/16 HP | gap (#127) | Attack **(still the fatal decision)** |
| `fear-blocks-the-attack` — minimaxflash-1.5: afraid, a 2-HP crow, a shrieker two tiles away | `reference/borg-runner/records/minimaxflash-1.5/REPORT.md` | Attack — the crow while afraid: the engine refuses | Walk — away from the shrieker, or hold while the fear runs out | active | — |
| `rs-d0005-fang-unused-cure` — rs D0005: 2/15 HP, Fang adjacent, one known CLW unused | `reference/deaths/borg-rs/D0005-recorded` | Attack — Fang at 2 HP | Quaff — the one known Cure Light Wounds; Walk(7) — the legal north-west step | gap (#122) | Attack **(still the fatal decision)** |
| `rs-d0006-mold-at-two-hp` — rs D0006: a stationary 2-HP mold, 2/15 HP, three retreats | `reference/deaths/borg-rs/D0006-recorded` | Walk — the retreat/re-entry oscillation; Descend | Attack — the 2-HP grey mold | active | — |
| `rust-d1330-rest-with-pursuer` — D1330: rest at 1/17 HP with a scrawny cat two steps away | `docs/deaths/D1330-rust-seed1053` | Rest — while a pursuer can close within the rest | Ascend — the up staircase underfoot | active | — |
| `rust-d1358-retreat-at-one-hp` — D1358: 1/51 HP, a 2-HP mouse adjacent, retreat chosen | `docs/deaths/D1358-rust-seed1001` | Walk — retreating at 1 HP provokes the lethal free hit | Attack — kill the 2-HP mouse before it acts | active | — |
| `rust-d1413-stairs-under-fang` — D1413: 1/17 HP, Fang adjacent, standing on the up staircase | `docs/deaths/D1413-rust-seed1082` | Attack; Walk | Ascend — the staircase underfoot, before Fang acts | active | — |
| `rust-d1420-starvation-no-food` — D1420: starving at 0 food on a down staircase with a full pack | `docs/deaths/D1420-rust-seed1032` | Descend — going deeper while starving | Eat — food, or return to town; Ascend | gap (#122) | Descend **(still the fatal decision)** |

## M3 triage (D1326-D1420)

Ninety-five M3 dossiers were read (`docs/deaths/D1326` to `D1420`). Five reproducible families:

| Family | Dossiers | Fixture | State |
| --- | ---: | --- | --- |
| `engage/Walk`: retreat from an adjacent monster at low HP | 59 | `rust-d1358-retreat-at-one-hp` | active after #120 |
| `recover/Rest`: rest while a pursuer can close | 21 | `rust-d1330-rest-with-pursuer` | active after #119 |
| `engage/Attack`: losing melee or stairs under contact | 12 | `rust-d1413-stairs-under-fang` | active after #120 |
| Starvation: no eat action, food 0, full pack | 4 | `rust-d1420-starvation-no-food`; bbc D0009 | gap #122 |
| Grey mold: the danger sum scored the mold 0 (fixed #142) | 5 | `rs-d0006-mold-at-two-hp` | active after #120 |

The cause view adds poison (4) and 28 monster kinds; grey mould (5) and Fang (2) get their own fixtures, and the rest repeat the families. Poison and the low-HP attack deaths need a cure or an escape the action set does not have, so they pin the #122 gap. The floating-eye deaths are covered by bbc D0013.


## What the fixtures found

1. **Race flag numbers were one too high (#142, fixed).** A grey mould was read as `NEVER_BLOW`, scored zero and was a safe rest. The fix and its raw-byte test landed in that task.

2. **Fear refused the attack but the strategy chose it (#119, fixed).** `TMD_AFRAID` makes `can_attack` false; the decision is a walk. `fear-blocks-the-attack` is active.

3. **A one-turn danger read let the Borg rest by a pursuer (#119, fixed).** The assessment now sees the cat and the Borg leaves, taking the up staircase. `rust-d1330-rest-with-pursuer` is active.

4. **Walking away from an adjacent monster is not an escape (#120).** The engine gives every adjacent monster a free hit. The tactical layer scores that hit, so at 1 HP it attacks a 2-HP mouse (`rust-d1358`), kills a 2-HP mould (`rs-d0006`) and ascends when the staircase is underfoot (`rust-d1413`). All three are active. The fixtures also explain bbc D0018, whose eight-step retreat from a faster wild cat was the death.

5. **Every response is scored and recorded (#120).** The trace carries all candidate actions with scores, not a synthetic `1.0`; a refused attack (fear) is absent by construction, and the losing alternatives stay visible. #138 can narrow the remaining trace contract.

6. **The remaining gaps are capabilities, not decisions.** Quaff/read/eat/wield and the strategic return home are #122/#123; shooting and casting are #127 (M5.3). The fixtures stay as explicit, owned gaps rather than passing by accident.


## What is not claimed

* The committed dossiers do not contain the original snapshots or traces, so every scenario here is **constructed** from the dossier's tables and anchors. The room shape, the stat line and (for our own deaths) the monster positions are declared in each file's `construction` note. No fixture is a recording.

* The owner-manual branches are save-load diagnostics, not independent lives or population evidence. Their `verified` text states the turn range and end state they actually verified.

* A gap names the action or assessment the milestone has not built. It does **not** claim the recorded death would have been avoided; it claims the current Borg cannot yet make the recorded feasible response.

* No fixture asserts parity with the C Borg or a survival benefit. The comparison against the C Borg is descriptive (PLAN M4).

