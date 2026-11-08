# D0012 overall assessment: poison at turn 27770

[Investigation #325](https://github.com/diegoami/better-borg-c/issues/325) · [Feedback process](../FEEDBACK.md)

Actual life: "Measure", Gnome Paladin, normal mode, seed 1907608237, source/harness
`867c777845fd` (65 carried patches, **including FLOWCACHE-404**), upstream baseline `e473720`.
Died of poison at engine turn 27770 at CL2, DL1 (max 2), HP −1/21. Sources are the original
`trace.txt`, the per-frame positions/HP in the original `frames.jsonl.gz`, and this dossier's
snapshots. Coordinates are (x, y).

## Owner feedback

Received in chat on 2026-11-08 and recorded verbatim on #325:
- 27237–27238: stairs down, then up.
- The eye and the soldier ant are killed.
- From 27525 it fights a mold, going back and forth because it is scared and wants to leave.
- One staircase is in the mold's way and it can't reach the one on the left. It never
  fights back and keeps getting hit.
- At 27728 it reaches the stairs area, but worms swarm it and it has no potions to drink.

## Geometry

The grey mold sits at (105,11). In this version it has **2 HP, AC 1, two SPORE 1d4 blows,
and never moves**. The east down staircase is at (107,11). Its only access square, (106,11),
is next to the mold, because (106,10) and (106,12) are walls. The west down staircase at
(91,11) is reached through the open door at (99,12) and the room beyond, where the white worm
masses were.

## Timeline

| Turn | Position / HP | Native fact |
| --- | --- | --- |
| 27237 | — | "Needing to get back on correct depth": stair down/up to DL1. |
| 27353, 27416 | — | Floating eye and soldier ant killed in melee. |
| 27476 | (99,9), 20 HP | Second white worm mass seen: "Scary guy on level", leaving and fleeing. |
| 27507→27518 | (103,10)→(104,10), 21→17 | Stair flow (cost 4) steps next to the mold; it hits. |
| 27527→27545 | back to d2, then d1, 17→14 | Caution "Retreating … (16 > 0)", then the stair flow again. |
| 27547→27564 | d2→d1, 14→9 | Same. |
| 27567→27582 | d2→d1, 9→5 | Same. |
| 27597–27627 | west to (99,13) | "Flowing toward stairs at cost 12" (west stair). |
| 27637–27682 | back east to (104,12), 6→3 | New worm mass seen at (96,12), flow cost 8, and the mold hits again. |
| 27687–27737 | west to (98,13) | Retreat west. |
| 27746–27770 | (98,13)→(99,11), 2→−1 | Worm masses crawl on it and poison it; it dies opening a door. |

## Findings against the owner's points

**It never fights the mold (#324).** In the frames, every decision made while adjacent ends in
caution's retreat (`pos_danger > avoidance/3`). That runs before attack selection. Every
decision at distance 2 reaches `borg_attack`, which notes "Adjacent to monster on Scaryguy
Level", but melee needs distance 1, so the stair flow steps back into reach. A single
Main Gauche hit would very probably kill a 2-HP, AC-1 mold. This is not a guaranteed rescue,
but no comparison of that option ever happens.

The shape matches D0008 (#212), but here FLOWCACHE-404 is carried and the re-approach comes
from flee-to-stairs flow, not exploration through the cache.

**Stairs.** The east stair can only be reached next to the mold. The west stair was reachable,
and it did go west twice: once it turned back when a new worm mass appeared, and once the
worms caught it.

**Scared (#327).** The pristine scary rule treats white or green worm masses, or giant
white mice, as scary when the level-wide count of tracked breeders is at least the character
level. At CL2, two worms set whole-level *flight*. The Borg's own breeder caution
(`borg_check_breeders`) would only start *leaving* at `MIN(CL+2, 5)` = 4. White worm masses
here are slow (speed 100), with 10 HP and a 1d2 poison crawl. The owner's verdict was
"scared without reason".

**No potions (#34, #180).** It knew no cure. But it carried 8 Teal Potions that were really
Cure Light Wounds. The Borg's own floor-object notes printed the true kind name; that name is
log-only and not used for decisions. Two emergency fallbacks could have tried them, and
neither ran:
- The poison-emergency "quaff unknown potion" fallback is inside `borg_heal`, and caution
  calls `borg_heal` only when not fleeing.
- The final unknown-use fallback requires `MAXSP == 0`, and the Paladin had 3.

Also unidentified: an Orange and a Metallic Red potion, and two scrolls. It had no Phase Door
(last read at 24506) and 219 gold.

## Why the flight went back and forth

The owner's follow-up question (2026-11-09) was whether this was a pathfinding problem or
fear of walking next to a monster. It is both: two layers use opposite danger limits.

- **The route planner is bolder while fleeing.** While the scary flag is set at CL ≤ 5, the
  flee-to-stairs flow (`borg_flow_spread`) accepts any square whose danger is up to
  **3 × current HP**. Normally the limit is 3/10 × HP.
- **The step-by-step caution is timid.** `borg_caution` retreats from any square whose danger
  is above **HP / 3**, and it runs before the next planned step or any attack.

Both use the same danger value, 16 for the squares next to the mold. So the planner steps the
character in, and caution steps it back out.

Because the planner's limit scales with HP, the target staircase flips:

| Decision | HP | Mold squares | Target |
| --- | --- | --- | --- |
| 27536, 27556, 27576 | 17, 14, 9 | allowed | east, cost 4 |
| 27596 | 5 | forbidden | west, cost 12 |
| 27636 | 6 (regenerated 1 HP) | allowed again | east, cost 8 |
| 27696 | 3 | forbidden | west, cost 12, into the worms |

Each retreat also drops the current flow, so the next decision plans again from scratch. This
explanation is source-derived and matches every logged choice; the runtime values have not
been observed.

## Owner's manual alternative (verified)

The owner played the same seed from the requested turn 27525 (actual 27526) with
`play-from-replay.py`. The branch and session identities and hashes are in
[owner-manual-branch.json](owner-manual-branch.json). A complete rerun of the saved session
with `replay-borg.py --verify-only` is **VERIFIED**: every input, action, clock/file channel,
RNG/player state, native save and frame matches. What happened:

1. At 27526, HP 17, standing next to the mold: one blow, "You have destroyed the grey mold."
2. Rested to 21/21 by 27751.
3. Walked west and killed two white worm masses, with a brief poisoning.
4. Saved alive at 28253, CL2, HP 17.

This demonstrates that the cheap alternative #324 assumed existed and worked once. It is a
save/load manual line with different RNG use and no Borg memory restore. It is not an
independent Borg life or a population or policy benefit.

## Competing explanations and limits

- Walking past the mold to the east stair costs about three mold turns (2 × 1d4 each). At 20 HP
  that was plausible at first and no longer by about 9 HP. Attacking it is the cheaper
  alternative.
- The scary flag may be reasonable caution against breeders. The problem is what flight
  disables: attacking a weak blocker, and the healing and unknown-item fallbacks.
- Why the distance-2 decision notes "Adjacent" (the speed comparison in the filter) is not
  observed.

## Next discriminating evidence

A diagnostic-only observer on the original start/build:
- at 27517/27537, log `pos_danger`, avoidance, the caution branch that returns, and the attack
  value that would have been computed;
- at 27746–27761, record whether `borg_heal` is skipped by the fleeing guard.

Any candidate change needs D0008 and D0012 fixtures and a survival and progress evaluation. A
reusable original-C defect, if proven, goes to neo-angband-tracker/source207 first. No
gameplay change, new run or survival claim is made here.
