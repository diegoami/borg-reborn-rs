# D0016 overall assessment: crow at turn 4199

[Investigation #353](https://github.com/diegoami/better-borg-c/issues/353) · [Feedback process](../FEEDBACK.md)

Actual life: "Measure", Human Necromancer, normal mode, seed 2993630921, source/harness
`5cd2258dd78d` (67 carried patches), upstream baseline `e473720`. Killed by a crow at engine turn
4199 at CL1, DL1, HP −1/12. Sources are the original `trace.txt`, per-frame positions/HP from
`frames.jsonl.gz`, the native shop messages, and this dossier's snapshots. Positions are (x, y).

## Owner feedback

Received in chat on 2026-11-09 and recorded verbatim on #353:
- A CL1 Necromancer with questionable inventory and equipment hits a teleport trap at 3965.
- It ends up in a room with no clean exit, and takes on wild dogs, a centipede and a crow.
- The crow kills it.
- Inventory is an area a reborn Borg should work on.
- The owner may retry from 3974.

## Starting purchases

Starting gold was 360:
- **armour:** Soft Leather Armour [8,+0] 84, Leather Shield [8,+0] 66, Gauntlets [3,+0] 24;
- **unidentified rings:** two, for 66 and 50 (they were Searching <+1> and Feather Falling);
- **survival:** 2 Cure Light Wounds 40, one Phase Door 15;
- **food:** rations 15.

It deposited its starting Word of Recall at Home (831).

## Timeline

| Turn | Native fact |
| --- | --- |
| 3964 → 3974 | Teleport trap: (67,16) → (7,32), beside a crow, three wild dogs and a giant yellow centipede. |
| 3981 | Crow bite, 12 → 10. |
| 3983 | Danger 18 → only Phase Door → (15,28). |
| 3993, 4003 | Two Nether Bolts use all of its 2 SP: the centipede flees, a wild dog dies. |
| 4013 | Danger 21 > 2 × HP → "Fleeing (excessive danger)", which sets `goal.fleeing`. Melee routs the crow. |
| 4023–4131 | Melee kills or routs the dogs. It retreats a few squares and picks up an unknown potion. |
| 4162 | Rests at 11/12; a wounded crow returns. Up stairs are known only at flow cost 61–63. |
| 4182–4199 | Melee against the crow (speed +20, 9 HP, two 1d3 bites): hit, then miss. HP 11 → 7 → 5 → −1. |

At death it carried 2 known Cure Light Wounds, an unknown potion, 3 unknown scrolls, 7 rations
and two copies of its spellbook.

## Findings

- **The trap was bad luck,** but the response to it was mostly sound. The Phase Door and both
  bolts were used, and the pack was routed.
- **Healing (#34).** Once fleeing is set, caution never calls `borg_heal`. Independently, at
  danger 18 ≥ HP 5 the inverted −75 chance adjustment makes the healing skip certain
  ([neo-angband-tracker#364](https://github.com/diegoami/neo-angband-tracker/issues/364)). So
  two known CLW went unused at 5/12 HP.
- **Inventory (#41).** About 290 of 360 gold went on armour and unknown rings, with one escape
  and two cures bought, and the Recall was left at Home. This repeats D0015, and the
  Recall-at-Home pattern now spans D0006, D0009, D0015 and D0016.

## Owner's manual retry from 3983 (verified, survived)

The owner requested 3974; the actual start was the Borg's decision at 3983, at 10/12 HP with
the crow adjacent. The identity is in [owner-manual-branch.json](owner-manual-branch.json), and
a complete rerun is **VERIFIED**. What happened:

1. **First action: a Nether Bolt killed the crow,** the speed +20 threat. The Borg had spent
   that same action, and its only Phase Door, escaping from it.
2. A bolt and melee killed the wild dogs.
3. Out of mana, the owner used the Phase Door and a CLW against the centipede. The low point
   was 6 HP.
4. At CL3 the owner learned Create Darkness and Bat Form, and used the last CLW in a small
   kobold fight.
5. Back in town at 10653 with 225 gold, the owner bought a Cure Serious Wounds and saved alive
   at 10743 (CL3, HP 19).

One manual line, not a population claim. Owner: "it was close but I got away."

## Limits

- Whether drinking at 5 HP survives a speed +20 crow is untested; each CLW heals 20 HP.
- The owner's retry from 3974 starts at the 3983 decision. `--from-turn 3973` starts before the
  trap step. A retry will be verified and recorded separately.

No gameplay change, new run or survival claim is made here.
