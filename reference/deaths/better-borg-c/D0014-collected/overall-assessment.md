# D0014 overall assessment: blood falcon at turn 306771

[Investigation #341](https://github.com/diegoami/better-borg-c/issues/341) · [Feedback process](../FEEDBACK.md)

Actual life: "Measure", High-Elf Rogue, normal mode, seed 1907608242, source/harness
`867c777845fd`, upstream baseline `e473720`. Killed by a blood falcon at engine turn 306771 at
CL11 (reached at 306736), DL20 (max 20), HP −4/91. Sources are the original `trace.txt`,
per-frame positions/HP and the diagnostic monster list from `frames.jsonl.gz` (diagnostic only,
never a Borg input), and this dossier's snapshots. Positions are (x, y).

## Owner feedback

Received in chat on 2026-11-09 and recorded verbatim on #341:
- At 306703 the digging rogue meets a falcon and phases.
- It lands near an orc archer and an elemental, and smashes the archer with a Lance of Slay Orc.
- It is surrounded with no Phase Door, though there is a gap at 306726.
- It kills a falcon; an illusionist comes; it backs up and gets stuck (306747).
- A second falcon kills it.
- Speed and Cure Light Wounds could have helped. The owner wants to try both before and after
  the Phase.

## How it reached DL20

Max depth stayed 5 until 282983. Three Deep Descent reads followed (#183), as in D0011:

| Turn | Situation | Result |
| --- | --- | --- |
| 282983 | DL2, HP 85/85, danger 97 | DL10 |
| 284602 | DL9, HP 85/85, a baby green dragon, danger 87 | DL15 |
| 284961 | DL15, HP 38/85, danger 203 | DL20, at CL9 |

It then spent about 21k turns on DL20 with "restock" climb goals and repeated "no up stairs
found, going down" notes (#321).

## Fatal encounter

| Turn | Native fact |
| --- | --- |
| 306690–306703 | Digging a magma vein at (72,37), HP 87/87. An illusionist blinks nearby and a blood falcon is out of view at (79,38). Danger 141 → reads its last Phase Door scroll. |
| 306713 | Lands at (82,36) next to an orc archer (83,36) and a fire spirit (84,36). Earth hounds are out of view. Danger 399. "You smite the orc archer", which flees; it learns the Lance is of Slay Orc. |
| 306721–306728 | Danger 1046 → 1325. The blood falcon claws and bites: 87 → 60. |
| 306728–306736 | One hit kills the falcon (6 HP). CL11. |
| 306736–306746 | Illusionist at (76,36). "Backing up" twice to (81,35); it stays there until death. |
| 306746 | Second blood falcon seen at (76,36). |
| 306756–306763 | Attacks the illusionist, which blinks. Logged danger 22 while the second falcon is 3–4 squares away and out of view. |
| 306769–306771 | Two falcon rounds before the rogue acts again: 59 → 29 → −4. |

**Pack at death:**
- **Healing:** 21 known Cure Light Wounds, 5 known Potions of Speed, 5 Neutralize Poison.
- **Unidentified potions:** Vespertine, Gloopy Green ×4, Teal ×5, Viscous Pink.
- **Ranged:** two Slings without ammunition, and an empty Wand of Stinking Cloud.
- **Spells:** Phase Door at 42% fail (9 SP), Detect Monsters, Object Detection.

## Findings

- **The blood falcon.** Speed +30, blows 2d10/2d10/3d8 (about 35 per round), but only 6 HP. It
  is lethal if it gets turns, and dies to one hit.
- **The Phase at 306703.** It spent the last scroll at full HP and landed into a group. Whether
  staying and fighting or digging away was safer is untested; this is the owner's "before"
  test.
- **No escape at danger 1046–1325.** No escape branch fired, and the Phase spell was never
  attempted. The branch reasons are not observed. Walking through the gap is part of #159.
- **Speed and CLW unused (#340, #34).** `borg_defend_aux_speed` drinks a Speed potion only when
  the hasted danger falls to about HP/3, so never in a real emergency. The CLW rungs need danger
  < HP + heal. The last-resort unknown use excludes characters with spell points.
- **The second falcon.** The logged danger was 22 with a known speed +30 falcon just out of
  view, and it then did 63 damage before the rogue's next action. Whether out-of-view fast
  monsters are underweighted remains open on #341.

## Owner's manual alternative before the Phase (verified)

The owner took over at the Phase decision (requested 306700, actual 306703) with
`play-from-replay.py`. The identity and hashes are in
[owner-manual-branch.json](owner-manual-branch.json). A complete rerun with
`replay-borg.py --verify-only` is **VERIFIED**. What happened:

1. No Phase Door was read. A Potion of Speed was drunk on first sight of the falcon.
2. The first falcon clawed for 87 → 60 and was killed in one hit.
3. The illusionist's blindness and confusion were each cured with Cure Light Wounds.
4. The second falcon was killed in one hit. The illusionist fled, then teleported away.
5. Detect Monsters was cast, then an earth hound fight (shards and bleeding, cured with CLW),
   after which the owner retreated from the pack.
6. Saved alive at 306980 with 88/91 HP on DL20. Used 1 Speed and 5 CLW.

By contrast, the original Borg cast Detect Monsters 234 times as routine detection upkeep. Its
last cast was at 300483, and it did not use detection tactically in this encounter. This is one
manual line, not a population claim. The owner's after-Phase try (306713) is still open.

## Limits and next evidence

- Whether Speed plus CLW would have survived is untested. The owner's takeovers at 306700
  (before the Phase) and 306713 (after it) will be verified and recorded as manual
  counterfactuals.
- A diagnostic observer at 306713–306763 should log the escape, speed and heal branch values,
  and the falcon's danger contribution.

No gameplay change, new run or survival claim is made here.
