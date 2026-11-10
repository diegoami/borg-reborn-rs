# D0018 overall assessment: wild cat at turn 33471

[Investigation #401](https://github.com/diegoami/better-borg-c/issues/401) · [Feedback process](../FEEDBACK.md)

Actual life: "Measure", Dunadan Druid, normal mode, seed 2993630923, source/harness `5cd2258dd78d`
(67 carried patches), upstream baseline `e473720`. Same collection as D0016 (game 4). Killed by a
wild cat at engine turn 33471 at CL2, DL1, HP −1/19. Sources are the original `trace.txt`,
per-frame positions/HP/inventory and labelled diagnostic state from `frames.jsonl.gz`, and this
dossier's snapshots; the raw trace, frames and manifest hashes match record.json. Positions are
(x, y).

## Owner feedback

Received in chat on 2026-11-10 and recorded verbatim on #401:
- A Druid killed by a wild cat; "as usual, inventory is a problem".
- "No health potion, no phase door", "a stupid melee death".
- The owner wanted to play from the turn the cat was met (named as 33347/33356), and asked for a
  death report.
- After the retry: "it could not be saved, the cat killed me too".

Neither 33347 nor 33356 appears in the recording. The cat comes adjacent at 33363 (HP 19/19); the
first claw lands at 33372.

## Starting purchases and town visits

Starting gold was 373. It deposited its starting **Word of Recall at Home** (352), then spent:
- food and torches 36, **one** Phase Door 15;
- **Studded Leather Armour (-1) [12,+2] 266**, Small Metal Shield 36, torches 12.

8 gold left and no Cure Light Wounds.

Second visit (25229–25659), with 373 gold found in the dungeon:
- Leather Sandals [1,+5] **126**, Potion of Speed **75**;
- two unidentified jewellery items **116** (Feather Falling, Slow Digestion);
- cap, gloves and torches 32, **one** Phase Door 15.

9 gold left and again no CLW. The Speed potion was drunk at 25839, at full HP 19/19, against a 2-HP
giant white mouse (danger 12).

At CL2 this build's Druid has no attack spell (Stinking Cloud is CL3 in its class.txt). Its spells
were Detect Life, Fox Form and Remove Hunger, so the full 4 SP during the fight was not a missed
bolt.

## Timeline

| Turn | Native fact |
| --- | --- |
| 26219 | Picks up 2 Ultramarine Potions. Diagnostic state (not Borg knowledge): **Cure Light Wounds**, flavour unknown. Never tried in about 7,000 turns. |
| 31722 | Takes the stairs to DL1 and arrives beside a blue yeek and a wild cat (speed +8, 8 HP). Melees the yeek, HP 19 → 14. |
| 31751 | Yeek at 0 HP, danger p=5, HP 14/19. "Boosting bravery (1)", then "Considering Phase (twitchy)": reads its **only Phase Door**. |
| 33113–33154 | "Spent too long on level"; fleeing is set; it cannot flow to up stairs. |
| 33164 | "Scary guy on level": whole-level flight. The tracked monster that set it is not identified by this recording. |
| 33363 | Flowing toward stairs, it meets the wild cat adjacent at (104,37); HP 19/19, danger p=6. "Adjacent to monster on Scaryguy Level": one whip hit, cat 8 → 5. |
| 33372–33452 | **Walks away for 8 steps** ("Retreating to 41,93 … (6 > 0)"). The cat follows and claws it every step: HP 16 → 12 → 10 → 7 → 5 → 3 → 1. The cat regenerates to 6. |
| 33462 | At HP 1 it attacks again: cat 6 → 2, "flees in terror!", but claws once more. Death at 33471. |

At death it carried the 2 unidentified CLW, 2 unidentified scrolls (Blessing, Treasure Detection),
2 Potions of Confusion, 5 Scrolls of Light, 2 Scrolls of Darkness, 10 spare torches, 7 rations, an
apple, a spare Wicker Shield, spare sandals and a duplicate spellbook. Recall was at Home.

## Findings

- **Walking away from a faster monster (#159).** It swung only twice. At 19 HP the strategic
  retreat test (`borg-caution.c:1430`, danger 6 > avoidance/3 = 6) failed, so it attacked. After
  the first claw (16 HP, 6 > 5) the test passed, and it passed again every step as HP fell. Each
  step compares the danger here with the destination using the cat's current square; nothing
  models a faster pursuer closing in. A cat already at 5 of 8 HP got free attacks for eight steps.
- **The last escape went on a twitch (#154, #41).** The only Phase Door was read at 31751 by the
  bored/twitchy fallback (`borg-think-dungeon.c:2149`, up to 3 Phases per level to get unstuck), at
  danger 5 against a dying yeek. That is not a threat response.
- **Unidentified cures (#180).** It had two CLW, unidentified. No unknown-potion path applies in this
  fight: two require being poisoned or cut, and the last-ditch one (`borg-caution.c:2023-2027`, "if I
  am gonna die next round, and I have no way to escape") requires `!borg.trait[BI_MAXSP]`, which
  excludes any character with mana. This Druid had 4 SP and no escape spell. Whether that block
  would otherwise have been reached at 33462 is untested.
- **Inventory (#41, #183, #340).** 266 gold on body armour in the first visit; 126 on sandals, 75 on
  a Speed potion drunk at full HP against a mouse, and 116 on unknown jewellery in the second. One
  Phase Door per visit, no CLW, Recall at Home, and 10 spare torches plus spare armour in the pack.
  This repeats D0015 and D0016.

## Owner's manual retry from 33363 (verified, died)

The start was 33363 with the cat adjacent at 8/8, HP 19/19 and the Phase Door already gone. The
identity is in [owner-manual-branch.json](owner-manual-branch.json), and a complete rerun is
**VERIFIED**. What happened:

1. The owner stepped back one square to (102,37). The cat followed.
2. Fox Form failed once ("You failed to concentrate hard enough!"), then worked. All 4 SP were gone
   by 33394, and HP fell 16 → 6 during the two casts.
3. In fox form the bites and claws did nothing: "You fail to harm the wild cat." In this build fox
   form is −3 to-hit and −3 damage, with the weapon merged away.
4. Fox form gives +1 move, so the owner walked about 17 squares west, (101,37) → (84,38), with the
   cat one or two squares behind and HP at 3–4.
5. It caught up: "You fail to harm something. You miss something. The wild cat claws you. You
   die." Death at 33505, 34 turns after the Borg's.

The cat was never damaged. The unknown potions (actually CLW) and scrolls were not tried. One
manual line, not a population claim.

From 33363 on, a CL2 Druid with no escape, no known cure and no attack spell has few good options
against a speed +8 cat, even in human hands. That puts the weight on the earlier decisions: the
Phase Door spent at 31751, no CLW bought in two town visits, and the unknown potions never tried.

## Limits

- What set `borg.mon.scary` at 33164 is unresolved. The actual level holds an apprentice and a
  cutpurse (both on the CL≤8 scary list) at (143,12)/(147,12), but the trace has no Borg sighting
  note for either. Settling it needs a diagnostic observer replay that logs `borg_kills` at
  33154–33164.
- Whether the Borg's own line, standing and fighting from 33372 with the cat at 5/8, survives is
  untested; the owner's retry took a different line.
- `--from-turn 31722` (arrival beside the yeek and cat, Phase Door unused) would test the earlier
  position.
- Source lead [neo-angband-tracker#154](https://github.com/diegoami/neo-angband-tracker/issues/154)
  (CBUG-0138) concerns the same retreat block, but this case does not exercise it: the first
  retreat step (102,37) was two squares from the cat.

These are observations with source references. Under the review rule for non-carried findings, no
disposition, source report or policy change follows until a review package is reviewed. No
gameplay change, new run or survival claim is made here.
