# D0019 overall assessment: cutpurse at turn 8056

[Investigation #406](https://github.com/diegoami/better-borg-c/issues/406) · [Feedback process](../FEEDBACK.md)

Actual life: "Measure", Half-Troll Necromancer, normal mode, seed 2993630924, source/harness
`5cd2258dd78d` (67 carried patches), upstream baseline `e473720`. Killed by a cutpurse at engine
turn 8056 at CL3, DL2, HP −4/32, after 483 Borg decisions. Sources are the original `trace.txt`,
per-frame positions/HP and diagnostic monster state from `frames.jsonl.gz`, the native shop
messages, and this dossier's snapshots. Raw trace and frames hashes match record.json. Positions
are (x, y).

## Owner feedback

Received in chat on 2026-11-10 and recorded verbatim on #406:
- "welll I got lucky, I got swarmed but I phased door to a stairs up and escaped. Still,
  inventory issues."

## Starting purchases

Starting gold was 360:
- **armour:** Soft Leather Armour [8,+5] **300**, Metal Cap [3,+0] 24;
- **survival:** **one** Phase Door 15, **no** Cure Light Wounds;
- **food:** 5 rations 15.

6 gold were left. It deposited its starting Word of Recall at Home (1161). Its only other
consumable was an unidentified Blue Potion (Neutralize Poison, per diagnostic state). Nether
Bolt had 42% fail with 3 SP maximum.

## Timeline

| Turn | Native fact |
| --- | --- |
| 7493 | Stairs to DL2 at CL2, 30/30 HP. The up staircase is at (78,53), at the bottom of a north–south corridor from (78,45). |
| 7703 | At (81,45), full HP, an apprentice is seen at (79,44): "Scary guy on level" (apprentice and cutpurse are on the CL≤8 name list), so it flees the level. Two cutpurses and a second apprentice are further west at x=68–75. |
| 7713–7753 | "Desperate for Stairs (three)": west along y=45 to the stair-corridor head (78,45), HP 30 → 21. Stair flow cost 25 → 8. |
| 7754–7763 | At (78,46), one step into the stair corridor and seven squares from the stairs. A cutpurse and an apprentice are behind it at (78,45)/(77,45); the corridor south is empty. Danger 341 vs HP 21: "Critical Phase" reads its only Phase Door. |
| 7764 | Lands at (70,48), west of the group, which is now between it and the stairs. |
| 7793–7863 | Retreat and back-away steps west, away from the pursuing group, to the junction (68,45). |
| 7873–8056 | Melee in place ("Adjacent to monster on Scaryguy Level"), danger 42–390 vs HP 24 → −4. Kills cutpurse 51 (7933), apprentices 53 and 7 (8013, 8033) and mice; reaches CL3. Cutpurse 52 (24 HP) kills it. Gold stays 6; no theft. |

## Findings

- **Inventory (#41, #154, #183).** 300 of 360 starting gold went on one body armour. It bought
  one Phase Door and no CLW, and left Recall at Home. This repeats D0013, D0015, D0016 and D0018.
- **Name-list flight (#327).** A single apprentice at full HP set whole-level leaving and
  fleeing. The same scary-list mechanism appears in D0012, D0013, D0015 and D0018.
- **The last Phase was spent in a clear stair corridor (#159).** At 7763 the attackers were
  behind it and the stairs seven squares ahead. The Phase landed on the far side of the group.
  Whether walking on with two same-speed pursuers in a corridor would have reached the stairs
  is untested.
- **An adjacent escape item was ignored.** Cutpurse 51 dropped a Phase Door at (68,46), adjacent
  to the Borg. The square was empty at 7930–7938, 8004–8020, 8024–8031 and 8034, and at 8011 no
  monster was adjacent at all. The Borg's notes list the object, but it never stepped onto it.
  Source-checked order: `borg_caution()` (`borg-think-dungeon.c:1533`), then
  `borg_attack(false)` (:1589), then `borg_flow_take(true, 5)` (:1606). An attack target always
  pre-empts the pickup; at 8013 it cast a failed Nether Bolt at a 2-HP mouse.

## Owner's manual retry from 7703 (verified, survived)

The identity is in [owner-manual-branch.json](owner-manual-branch.json), and a complete rerun is
**VERIFIED**. What happened:

1. One Nether Bolt failed. Melee took apprentice 7 from 15 HP to 0 and it fled, after
   confusing the owner.
2. While confused, the owner hit the cutpurse (24 → 18). At 7778 it stole all 6 gold and
   teleported away, removing one attacker. In the Borg's run no theft happened.
3. At 7844, on 19 HP, the Phase Door from (87,45) landed on (78,46), the stair-corridor head:
   the square the Borg had phased away from.
4. Unpursued, the owner walked to (78,53) and took the up stairs at 7923. On DL1 the owner
   killed a fruit bat and a rock lizard and saved at 9033 (CL2, HP 30/30). Lowest HP was 17.

The owner calls it luck, and the record agrees: the theft and the Phase landing both went the
owner's way. One manual line, not a population claim.

## Limits

- The stair walk from (78,46) at 7754 is untested; `--from-turn 7754` would test it.
- Whether stepping onto the dropped Phase Door at 8011 and reading it would have saved the
  character is untested.

No gameplay change, new run or survival claim is made here.
