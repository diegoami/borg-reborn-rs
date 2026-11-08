# D0011 overall assessment: griffon at turn 119600

[Investigation #322](https://github.com/diegoami/better-borg-c/issues/322) · [Feedback process](../FEEDBACK.md)

Actual life: "Measure", Half-Elf Warrior, normal mode (`borg_munchkin_start=0`), seed
1907608236, source/harness `867c777845fd` (65 carried patches), upstream baseline `e473720`.
Died at engine turn 119600 at CL11 (reached at 119587), DL18 (max 18), HP −2/111.
Sources are the native `trace.txt` of the original run, this dossier's `events.log`, and
the T−5…T−105 snapshots. Snapshot `details` are diagnostic actual state and were never
Borg inputs.

## Owner feedback

Received in chat on 2026-11-08 and recorded verbatim on #322: the floor opens at 118711;
wolves and fruit flies; rest does not recover fully; griffon at 119278; no healing
potions; Phase at 119308; leaves a safe spot at 119422/119433; illusionist and Phase at
119466; the griffon returns and kills. Also: no Word of Recall, many useless objects, and
potions that would help but are not used.

## Timeline

| Turn | Native fact |
| --- | --- |
| 54903 | In town, CL4, HP 59/60, two filthy street urchins, "Scary guy", danger 377. Reads Deep Descent → DL7 (max 2 + 5). |
| 55624–118552 | DL4–7, max 8, about 63k turns without a town visit. 430 "restock phase door" and 82 "1 recall" preparation notes, 563 "no up stairs found, going down" fallbacks, 10 up-stair departures. |
| 100129 | Wields an unidentified Maul (4d4) (+7,+6) {??}. |
| 110374 | CL10. |
| 118552 | DL4, HP 102/102, an invisible caster and two scouts, danger 313, "Fleeing (excessive danger)", no teleport. Reads Deep Descent. |
| 118611 | Arrives on DL13 (max 8 + 5). Wields Dagger (1d4) (+0,+0) in place of the Maul. |
| 118641 | DL13, HP 98/102, ochre jelly adjacent, danger 705. Reads its last Deep Descent, then a "Critical Phase". |
| 118711 | Arrives on DL18 (13 + 5). |
| 118711–119258 | Corridor beside a seven-wolf pack and fruit flies. Fight/retreat/rest; rests are repeatedly cut short as wolves enter view. HP about 86/102. |
| 119268–119307 | Griffon melee with the dagger: HP 86 → 42, griffon 152 → 110. Phase Door at 119307. |
| 119427 | "not prep: restock tele", "no up stairs found, going down", bravery 36 → 72, flows into unexplored dark. |
| 119467 | Logged warrior and two wolves, danger 64 at HP 38. Last Phase Door. |
| 119547 | "Fleeing (failed to teleport)". HP 38 → 24 against wolves and a centipede. |
| 119587–119600 | Griffon returns. HP 24/111 → 11/111, still in melee. Dies. |

Pack at 119495 (T−105):
- **Healing and escape:** 8 known Cure Light Wounds; 9 Hazy Potions (actually Cure Serious Wounds, unaware); 2 Speed; 3 Neutralize Poison.
- **Unaware:** True Seeing ×4, Infravision ×4, a Monster Confusion scroll, and an Azure Amulet <+1,+5> {??} (Trickery), not worn.
- **Junk:** 4 Darkness, 4 Rings of Searching, 9 spare torches, 2 sandals, and the Maul.
- **Missing:** no Phase Door, Recall or teleport. Gold 1769; last purchase at 29180.

## Findings against the owner's points

**Healing (#319).** Pristine `borg_heal` drinks CLW only when
`pct_down >= 30 && (pct_down <= 40 || CLEVEL < 10)`. This warrior was CL10/11 and 58–90% down
at every fatal decision, so no CLW rung could fire. The CSW rung needs a known CSW, and its
nine CSW were unaware. It had no CCW, Healing or healing device. The `borg_recover` healing
step excludes CLW. The last-resort unknown-potion branch needs danger > 4 × HP, and the
logged danger was 25 against HP 24 and 11. This is a source-derived explanation consistent
with the trace; the runtime rung rejection has not been observed. A CLW heals 20 HP, about
one griffon turn, so no rescue is claimed.

**Weapon (#320).** At 118611 the Borg swapped the Maul for a Dagger. The likely cause, from
the source: `borg-power.c` gives +1,000,000 to an unidentified wielded weapon only below DL10,
and +45,000 per blow at CL ≤ 10. The 4.2 blow calculation gives this warrior about 3.0 blows
with the dagger and 1.3 with the maul, so about 7.6 against 21 average damage per turn. When
the identification bonus lapsed at DL13, the blows bonus picked the dagger. The power values
at that moment are not yet observed.

**Rest and the "safe place".** The incomplete rests are visible wolf interruptions. Leaving
the hidden spot at 119427 is the climb goal falling back to exploration because no up
staircase was known. It is not a decision to fight.

**Deep Descent (#183).** All three reads come from `borg_teleport_off_level` under
excessive-danger flight. Deep Descent goes to max depth + 5, so the DL4 read was a nine-level
jump from the current depth at full HP. The town read was triggered by 3-HP street urchins;
whether theft weighting inflated that danger is the #158 question. Whether staying, walking
or fighting would have survived is untested.

**Return to town (#321).** For 63k turns the Borg wanted to restock. The pristine fallback
"no up stairs found, going down" plus connected stairs can cycle without making progress.
That this fallback dominates is not yet shown.

**Inventory (#180, #41).** All unknown-item tests in this life were emergency uses. Nine
CSW stayed unidentified for about 47k turns, and the trip that might have identified them
by selling never happened. Junk and missing escapes repeat the survival-stock question.

## Competing explanations and limits

- The DL4 danger of 313 may be real: an invisible caster plus archers, and no teleport.
  Deep Descent may have been the only exit the Borg knew. The problem may be the absence of
  stock rather than the choice of escape.
- With the maul, the griffon (152 HP) fight is still not shown to be survivable. Wolves at
  speed +10 and no escapes remain.
- Speed potions compete with attack value in the defence/attack simulation. The trace shows
  attack value 38; the speed score is unobserved.
- The owner's illusionist is in the diagnostic monster list, but the Borg's own creation
  log at 119467 names a warrior and two wolves.

## Next discriminating evidence

A diagnostic-only observer on the original start/build:
- at 118611, `borg_power` and BI_BLOWS for the Dagger and the Maul;
- at 119268–119596, `borg_heal` `pct_down`, `chance`, the fleeing guard and the rejected rungs, plus the speed defence score;
- a cause classification of the level changes in 55624–118552.

No gameplay change, new run or survival claim is made here. A reusable original-C defect,
if proven, goes to neo-angband-tracker/source207 before a separate consumer carry.
