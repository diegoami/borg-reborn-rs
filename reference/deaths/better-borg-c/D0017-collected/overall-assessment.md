# D0017 overall assessment: poison at turn 1094860

[Investigation #403](https://github.com/diegoami/better-borg-c/issues/403) · [Feedback process](../FEEDBACK.md)

Actual life: "Measure", Dwarf Ranger, normal mode, seed 2993630922, source/harness `5cd2258dd78d`
(67 carried patches), upstream baseline `e473720`. Same collection as D0016 and D0018 (game 3).
Died of poison at engine turn 1094860 at CL25, DL21 (max 22), HP −1/279, after 1.09M turns and
90,701 decisions, with 2,734 kills and 5,740 gold. Sources are the original `trace.txt`,
per-frame HP/position/inventory and labelled diagnostic state (monsters, timed effects, Borg
`fleeing`/`avoidance`) from `frames.jsonl.gz`, and this dossier's snapshots. The raw trace, frames
and manifest hashes match record.json. Positions are (x, y).

## Owner feedback

Received in chat on 2026-11-10 and recorded verbatim on #403: a ranger dies of poison to spiders;
"he get swarmed by spiders but having so many potins there is no way he should have died to
them". No manual retry has been reported yet.

## What it carried

- **Cures and escapes:** 9 Potions of Cure Critical Wounds (heal 60, cure poison), 20 Potions of
  Speed, 13 Scrolls of Phase Door, 7 Word of Recall, a Staff of Teleportation (3 charges).
- **Offence:** 7 Wands of Magic Missile, a Long Bow (x3) with 88 arrows, a Cutlass (+13,+14).
- **Spells:** Herbal Curing (6 SP; cures poison, cuts, stun) and Resist Poison. SP 18–20/42
  through the fight.

At death everything was still in the pack except one staff charge and one Recall.

## The threat

Mirkwood spiders in this build have speed **+10**, 41 HP and come in groups of 2d7. Each has a
1d8 bite and **two 1d6 poison bites**, plus the innate WEAVE spell, which the Borg logs as
`SPELL_027`. The Ranger moves at normal speed (10 energy per tick).

## Timeline

| Turn | Native fact |
| --- | --- |
| 1091224 | Arrives on DL21. From 1094198 it rests at (77,56), full HP. |
| 1094618 | Spiders arrive along the corridor from the west. It shoots at 279/279 and kills one at 1094646. |
| 1094662, 1094672 | Two Herbal Curing casts ("Cure poison – danger 26/57"), 12 SP. With 1–2 spiders adjacent it is re-poisoned at once. |
| 1094682 | Shoots, with three adjacent. HP 232. |
| 1094688–1094778 | Ten-turn walk toward (83,53) ("Retreating to 53,83 … (87 > 0)"). 3–4 spiders adjacent most turns. HP 201 → 171 → 149 → 143 → 124 → 75 → 47 → 32 → 26 → 9; danger 76–116. Nothing is used. |
| 1094758 | "Fleeing (excessive danger)" at HP 32 sets `goal.fleeing` (frames confirm 0 → 1). |
| 1094788 | At HP 7, with no retreat step left, it uses the Staff of Teleportation. "Danger Level 1": **escaped**. |
| 1094798 | Reads Word of Recall. Poison about 485 turns. |
| 1094808–1094858 | Seven "Resting for Recall" decisions with no monster visible. HP 7 → −1 at 1 per player turn. |
| 1094860 | Dies of poison, holding 9 CCW and 20 SP. |

## Findings

- **Healing after the escape was gated off (#34).** `borg_caution()` calls `borg_heal()`, which
  holds both the HP tiers and the cure-poison logic, only inside `if (!borg.goal.fleeing)`
  (`borg-caution.c:1001`). With fleeing set, caution went straight to "waiting for recall &
  safe, stay put" (`:1033`).
  - `borg_check_rest()` (`borg-flow-misc.c:1219`) has no poison or cut test.
  - The recall "buy time" block that does drink cures needs danger > 2 × avoidance (`:1984`),
    which never holds once safe.
  - Recall takes 15–34 player turns (`effect-handler-general.c:1145`), so at 6 HP losing 1 per
    turn it could not arrive in time.
  - This is the D0016 fleeing gate, here killing a character that had already escaped.
- **Retreat pre-empted escape (#159).** `borg_escape()` (Phase Door, Teleportation;
  `borg-caution.c:1654`) runs only when the strategic-retreat block (`:1420–1650`) finds no step.
  A step existed on every turn from 1094688 to 1094778. Neither the 13 Phase Doors nor the staff
  were evaluated until HP 7.
- **The walk itself is the pursued-retreat pattern from D0018.** Speed +10 pursuers stay adjacent
  and attack as it walks; here 3–4 of them at once.
- **CCW tier window (#319).** `fleeing` was 0 during the walk, so `borg_heal()` ran, and poison
  waives its random skip. But CCW is offered only at **50–55% down** (Level 3) or at HP < 50
  with danger < ccw_heal (Level 5):
  - 1094728 (HP 124, 55% down, danger 116): Level 3's "no rope-a-doping" test
    `ccw_heal > danger/3` compares 38 with 38 and fails on the tie.
  - 1094738 (HP 75, 73% down): past the window, and no later tier uses CCW.
  - 1094748 (HP 47): Level 5 needs danger 85 < ccw_heal 58, which fails.
  - It carries no CLW or CSW for the lower tiers.
  - The inverted chance adjustment
    ([neo-angband-tracker#364](https://github.com/diegoami/neo-angband-tracker/issues/364)) does
    not apply, because poison bypasses that skip.
- **Speed (#340).** `borg_defend()` ran every turn of the walk and declined. Potions-only Speed
  needs the hasted danger p2 ≤ avoidance/3; p2 is not logged, so the decline is untested.
- **Herbal Curing in contact.** Two casts with spiders adjacent cost 12 SP and two turns; the
  poison returned at once. This is an observation; that shooting instead would have mattered is
  untested.

## Owner's manual retry

None yet. Takeover commands, both qualifying the 1.09M-turn prefix:
- `./play-from-replay.py docs/deaths/D0017-collected --from-turn 1094618 --timeout 3000` starts at
  first spider contact, 279/279 HP.
- `--from-turn 1094789` starts after the teleport: alone, 7 HP, poisoned, 9 CCW.

A retry will be verified and recorded separately.

## Limits

- Speed p2 at 1094688–1094748 needs a diagnostic observer.
- Whether a Phase Door at 1094688 (HP 201) out-runs speed +10 spiders in this corridor is untested.
- The post-teleport cure is a single legal action with no visible monster; no rescue is claimed
  until a branch is played.

No gameplay change, new run or survival claim is made here.
