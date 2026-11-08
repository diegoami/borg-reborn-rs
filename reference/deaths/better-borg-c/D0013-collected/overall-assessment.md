# D0013 overall assessment: small kobold at turn 3716

[Investigation #332](https://github.com/diegoami/better-borg-c/issues/332) · [Feedback process](../FEEDBACK.md)

Actual life: "Measure", Dunadan Druid, normal mode, seed 1907608238, source/harness
`867c777845fd` (65 carried patches), upstream baseline `e473720`. Killed by a small kobold at
engine turn 3716 at CL1, DL1, HP −1/13. Sources are the original `trace.txt`, per-frame
positions/HP from `frames.jsonl.gz`, and this dossier's snapshots. Positions are (x, y).

## Owner feedback

Received in chat on 2026-11-09 and recorded verbatim on #332:
- A trap at 3497 aggravates everybody.
- A paralysing floating eye is in front of it. The Phase Door seems overkill: it could have
  just gone back.
- There is no real threat, until a kobold at 3651 and later an ant corner and kill it.
- The owner will try manually from 3651.

## Timeline

| Turn | Native fact |
| --- | --- |
| 801–2331 | Town: one Phase Door, Ring of Protection [+10] (266 gold), an unidentified ring (Feather Falling, 50 gold). No Cure Light Wounds. |
| ~3497–3502 | Row 41 westward: "A horn blast sounds!" (siren trap). |
| 3511 | At (56,41): bravery boost 13 → 26, "Flowing toward dark-1", opens the door to the north. |
| 3521–3522 | Floating eye seen at (55,39). Next step enters the doorway (56,40), diagonally adjacent to it. |
| ~3540 | Paralysed; later gazes resisted. HP 12. |
| 3571 | Danger 400 → "Fleeing (excessive danger)" → reads its only Phase Door, lands at (47,37). |
| 3581–3611 | Second floating eye seen at (44,39). Heads for up stairs, cost 16. |
| 3621 | Small kobold at (49,42) → "Scary guy on level" → fleeing. |
| 3641–3651 | Backs up from danger 5. |
| 3661–3691 | Giant white ant appears. Stair flows at costs 29, then 51. Boxed in around (53,35). |
| 3695–3716 | HP 13 → 9 → 5 → 3 → −1. Both its attacks go to the ant (Whip 1d3). Dies. |

## Findings

- **Phase instead of a step (#159).** A floating eye never moves, so stepping back to (56,41)
  would have left its reach. Caution evaluates escape ("Safe to Phase") before retreat and
  back-away, so the only Phase Door was spent. The step *into* the doorway right after the eye
  appeared is the earlier error. It fits a committed one-step flow continuing without a
  re-check, but that branch is not observed.
- **Scared of a small kobold (#327 family).** The pristine scary scan names Grip, Fang and the
  small kobold for CL ≤ 5. A single small kobold (8 HP, 1d5, logged danger 5) set
  whole-level flight. The flight mixed backing away with long stair flows and ended boxed in.
  This is the same planner-versus-caution pattern as D0012 (#324).
- **Final fight.** At 9 HP, with a 1d3 whip, both attacks went to the 11-HP ant rather than
  the 8-HP kobold that was doing most of the damage. Attack target scoring is unobserved.
- **Stock (#41).** It spent 316 gold on rings and bought one escape and no cures.

## Owner's manual alternative (verified)

The owner played from turn 3651 with `play-from-replay.py`. The branch and session identities
and hashes are in [owner-manual-branch.json](owner-manual-branch.json). A complete rerun with
`replay-borg.py --verify-only` is **VERIFIED**. What happened:

1. One hit on the small kobold, which fled in terror.
2. Three exchanges with the giant white ant, HP 12 → 6, and the ant was still alive.
3. While retreating south, a grey mushroom patch confused it twice (HP 4 → 3).
4. Up the staircase at (55,45) to town at 3909, with 3 HP.
5. Rested to 13/13 and saved alive at 4709, still CL1.

The owner's conclusion is that knowing which monsters are too tough at a given level is
central, and that the Borg has this knowledge but the game does not show it. The Borg's own
danger estimate comes from full race data, not the player's monster memory, and it agrees with
the owner. The kobold alone rated 5; the ant rated 18 to 38. Its whole-level flight was keyed
to the kobold by the name list instead. One manual line, not a population claim.

## Limits

- Whether the siren trap was known before it was triggered is not shown.
- Paralysis attempts are probabilistic; the outcome of stepping back is not tested.
- The fight against kobold plus ant at CL1 is not shown to be winnable. The owner's planned
  manual attempt from 3651 will be verified and recorded as a separate counterfactual.

No gameplay change, new run or survival claim is made here.
