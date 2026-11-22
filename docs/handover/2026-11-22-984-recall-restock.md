## 2026-11-22: A pending Recall on DL1 and a Recall valued too low on shallow trips (#984)

Branch `task/984-recall-restock`, from `track-p` at `4b2c2fcf`. Every
check below ran on the **default core, borg-rs** (`borg-core/`), with
`harness/borg.txt` and the zoo report's overrides.

**Reproduced.** Seed 312081895, Half-Troll Blackguard (`-race 7 -class 8`).
With the zoo report's command, `keys.log` hashes to `4aa99f20ea1ca7e6`: the
same game. It dies of poison on DL2 at 39700.

**The issue's leads, checked.** In `messages.log` the third column is not
the depth; `state.log` is the source here.
- *"Reads its last Recall in town."* Refuted. At 10848 it was on DL1
  (`d1/2`). It read the scroll as a flight from a scary monster ("Fleeing
  the level (recall)"). It then walked to the up staircase and took it to
  town at 10948 with the Recall pending. That path is caution's "Take
  stairs up", which the C Borg also leaves ungated (`borg-caution.c:1171`;
  only down stairs check `goal.recalling`, `:1189`).
- *"A trapdoor drops it to DL2 at 10990."* Refuted. "You feel yourself
  yanked downwards!" is the pending Recall firing in town
  (`game-world.c:707-731`). The Recall takes it to its maximum depth, DL2,
  15 to 34 world ticks after the reading (`effect-handler-general.c:1145`).
  The visit lasted 42 game turns: it bought a torch, and it was on its way
  to the Alchemist to sell an unknown scroll.
- *Why no scroll.* Read from the plan records (`BORG_TOWN_PLAN_RECORD`,
  seq 14–16, visit 9). The Alchemist sold Word of Recall at **125 gold**,
  40 in stock. The urgent pass held one torch (6 gold). The record's
  `available` row for the Recall is 0. The stock cap
  (`borg_town_shadow.rs:496-505`) leaves `(241 - 6 - 200) / 125 = 0`
  copies, so the **`buy_recall_from_gold` 200 floor** removed it. The
  **urgent rule** (#859) would have bypassed the floor, but only for a
  trip to `prepared_recall_depth` 5 or deeper; this trip went to DL2–3.
  **Departure room** (#864): not involved; the plan used all 235 gold
  before any room limit applied. With the floor at 0 (counterfactual
  replay), play did not change at all: Recall was available (35 copies),
  but the solve still chose Phase Door ×3, oil ×20, Cure Serious and
  4 potions (235 gold). Below the dive gate, one Recall is worth less to
  the planner than those supplies.

**The cause.** There are two:
1. On DL1 the Borg takes the up staircase with a Recall pending, so the
   Recall cuts the town visit short and returns it to its maximum depth.
2. Below `prepared_recall_depth`, a Borg with no scroll does not buy one:
   the floor removes it under 325 gold, and above that the solve ranks it
   below other supplies.

**The fix: two switches, both off by default** (play unchanged):
- `town_recall_urgent_depth` (`borg_town_utility.rs:255`) sets the trip
  depth from which `town_recall_urgent` makes the first Recall urgent. 0
  keeps the dive gate, as #859 does today; 1 means any dungeon trip.
- `caution_recall_dl1_hold_up` (`borg_caution.rs:7467`): while a Recall
  counts down on DL1, caution does not take the up staircase unless the
  danger is above the avoidance, the boundary of the stairs rule next to
  it. The danger escapes are not held. It is the up half of the town/DL1
  crossing; `recall_restricts` (#603, #627) already holds the down half.
  An arena level has no staircase (`arena_gen`, `gen-cave.c:4010`), so the
  hold never applies there.

Unit tests: `urgent_recall_depth_tests`, `recall_dl1_hold_up_tests`.
Both are new modules at the ends of their files, so that no other
parameter citation moved. `scripts/check-param-citations.py` reports 0
wrong citations; CALIBRATION.md has a new section, "Word of Recall on
shallow trips (#984)".

**Replays of 312081895** (build of this branch, 100,000-turn cap):

| arm | play hash | end |
|---|---|---|
| shipped | `4aa99f20ea1ca7e6` | poison, DL2, CL4, turn 39700 (unchanged) |
| `town_recall_urgent_depth = 1` | `858a7f7de5dcaff7` | alive at the cap, max DL4, CL7. That visit heads to the Alchemist first but is still cut short; it is stranded on DL2 until 45792, then buys Recall first on every later visit |
| `caution_recall_dl1_hold_up = 1` | `f0903d718796c565` | alive at the cap, max DL3, CL8. The Recall fires on DL1 ("yanked upwards") |
| both | `f0463d22b1d8928a` | alive at the cap, max DL2, CL5. The Recall fires on DL1, and it buys a Word of Recall in town at 11291 |

One seed: no claim either way (RULES 14).

**Checks.** `cargo fmt --check -p borg-reborn-rs`, `cargo clippy
--workspace --all-targets -- -D warnings`, `cargo test --workspace
--exclude borg-core` and `cargo test --release -p borg-core` (655 passed)
pass. `scripts/borg-core-vendor.py check` passes. Smoke on the default
core is 3/3 identical to `smoke-borg-rs`, so it was not re-recorded.

**Left, and for the owner.**
- The zoo measures the two switches, each alone and both, against the
  shipped settings: the request is [borg-zoo#688](https://github.com/diegoami/borg-zoo/issues/688), on `434d5371` (RULES 20). The owner starts
  it.
- The counterfactual replays are the study
  [borg-cemetery#133](https://github.com/diegoami/borg-cemetery/pull/133)
  (`studies/recall-shallow-blackguard-312081895`): six arms with their keys,
  plus the town plan records of the visit. The death's dossier is
  `D-6fb7bf106e60…` on `runs/2026-11-22-trackp-9085c266-full-deaths`
  (borg-cemetery#129).
- The 29,000 turns on DL2 with "no up stairs found" are not covered here.
  They are the stair-finding pattern of #935 (this seed was added there).
  The jelly loop at the end is #932.
- Whether to ship either switch is the owner's decision.

**Review (MiniMax-M3.1-Flash, PR #997 comment, on `1eecc73d`).** N1: the
zoo request is borg-zoo#688. N2: the hold now names `recall_restricts` and
`recall_pending_escape`. N3: the hold releases above the avoidance (`>`),
as the stairs rule beside it does; the four replays keep their hashes.
N4: not done; the proxy refuses the branch rename (owner to rename it or to say so). N5: #935 is named
above. N6: an arena has no staircase, so this is a note. N7: the zoo
request asks for deaths on DL1 with a Recall counting down.
