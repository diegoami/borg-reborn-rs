# D0015 overall assessment: Fang at turn 4158

[Investigation #348](https://github.com/diegoami/better-borg-c/issues/348) · [Feedback process](../FEEDBACK.md)

Actual life: "Measure", Dunadan Necromancer, normal mode, seed 1907608243, source/harness
`867c777845fd`, upstream baseline `e473720`. Killed by Fang, Farmer Maggot's Dog at engine turn
4158 at CL1, DL1, HP −1/13. Sources are the original `trace.txt`, per-frame positions/HP and the
diagnostic monster list from `frames.jsonl.gz` (diagnostic only, never a Borg input), and this
dossier's snapshots. Positions are (x, y).

## Owner feedback

Received in chat on 2026-11-09 and recorded verbatim on #348:
- At 4119 Fang surprises it, while the escape route is still open.
- It goes the opposite way, into a dead end.
- At 4124 it is not cornered, yet it neither runs nor fights, and doesn't use its spells.
- It then starts hitting a dog it cannot defeat, and dies.
- Escaping is a no-brainer: the dog moves erratically and does not block the way out.
- The owner will retry.

## Geometry

The corridor runs along row 44: up staircase at (100,44), dead end at (108,44). Openings to the
north at x=104 and x=106 lead to the area Fang came from.

## Timeline

| Turn | Native fact |
| --- | --- |
| 4099–4114 | Explores east to (108,44). Fang approaches (109,42) → (106,43), marked not visible. No Borg note of it. |
| 4119 | At (107,44): first bite, 13 → 10. Fang visible for the first time. |
| 4124 | Second bite, → 8. No Borg decision between 4118 and 4128. |
| 4128 | "Scary guy on level", leaving and fleeing, then melee (value 26) with a Main Gauche (1d5). |
| 4138, 4148 | Melee again from the same square. HP 8 → 5 → 2; Fang 25 → 20. |
| 4157 | Nether Bolt (its only castable spell, 1 of 2 SP, 14% fail): fails. |
| 4158 | Bitten, dies. It never left (107,44) after noticing Fang. |

Fang in this version: speed +20, 25 HP, one 1d4 bite, RAND_25 (erratic movement). The observed
rate was about one damaging bite per player action.

Stock: starting gold went on a Robe, Iron Shod Boots, a Cloak and a Ring of Protection [+6],
leaving 6 gold. The single Phase Door was read at 3511, and it had no cures.

## Findings

- **The dead end was not a flight choice.** The Borg was exploring and did not know about Fang,
  which approached unseen in the dark. A Necromancer carries no light (L11: information and
  sight).
- **Standing and fighting is deliberate pristine code.** `borg-caution.c` turns off the
  desperate stair dash with *"Don't run from Grip or Fang … try to take them on, you cant
  outrun them"*. It applies to any unique near on DL1–5, or any adjacent monster faster than the
  player. `borg_attack` explicitly permits attacking Grip and Fang. But the scary scan lists both
  as reasons to flee the level at CL ≤ 5 (#327). The rule assumes that being faster means you
  can't escape. It ignores Fang's erratic movement, its weak bite, the open corridor and the
  hopelessness of a 1d5 melee against 25 HP.
- **Spells.** It had one Nether Bolt worth of mana. Melee scored higher until the end, and the
  final cast failed.
- **Stock (#41).** All gold went on armour; no cures or extra escapes.

## Owner's manual retry from 4118 (verified, died)

The owner requested 4110; the actual start was the Borg's decision at 4118, at 13/13 HP in the
dead end. The identity and hashes are in [owner-manual-branch.json](owner-manual-branch.json),
and a complete rerun is **VERIFIED**. What happened:

1. Two bites before the first step (13 → 10 → 8).
2. Fang blocked the corridor, forcing a detour north.
3. A Nether Bolt only made Fang yelp in pain.
4. A graze left the character bleeding, with no potion to quaff.
5. Death at (103,44), three squares from the up staircase, at 4190.

By 4118 neither fighting nor the dash survived. Preparation decided this death.

## Was it broke? No: it bought armour

Native shop messages show starting gold of **357**:
- **AC:** Ring of Protection [+6] 150, Robe 84, Iron Shod Boots 84, Cloak 6 (330 in all);
- **survival:** one Phase Door 15;
- **food:** 12.

That left 6 gold, and it deposited its starting Word of Recall at Home (1891). The ring alone
would have bought ten Phase Doors. Its brief town visits at 3181 and 4028 came with 6 gold.
This is the strongest survival-stock case so far (#41).

## Owner's re-shop from turn 1 (verified, survived)

The owner replayed from turn 1, the second entry in
[owner-manual-branch.json](owner-manual-branch.json); a complete rerun is **VERIFIED**. With
the same 357 gold he bought:
- **ways home:** a second Word of Recall (125), keeping the starting one;
- **escapes:** 4 Phase Door (60);
- **cures:** 3 Cure Light Wounds (60);
- **other scrolls:** Monster Confusion (30), Detect Invisible (15), Remove Hunger (10);
- **other:** a flask of oil (3);
- **armour:** only 20 gold (Robe [2,+0], Sandals).

He toured DL1 from 2689 to 6779, found 910 gold and a Hazy Potion, and saved in town at 6889
with 926 gold, still CL1. Fang was never met, because the RNG and path diverge from turn 1, so
this does not test the Fang fight. The owner's conclusion: buying the right gear at the start
is half the battle.

The owner later clarified two points. The second Recall was bought by accident, so it is not
part of the intended loadout. The single Monster Confusion, Detect Invisible and Remove Hunger
scrolls were bought to learn their flavours, so that dungeon finds would already be identified.
The pristine Borg's buying code has no awareness or flavour logic at all (#180).

## Limits

- Whether a dash of seven squares to the stair survives at 8–10 HP is untested; this is what
  the owner's retry will show. `--from-turn 4118` starts at 13/13 HP in the dead end, while 4119
  resolves to the 4128 decision at 8 HP.
- The diagnostic visibility flag is game-side state; the Borg's own memory of Fang before 4128
  is unobserved beyond the absence of any note.

No gameplay change, new run or survival claim is made here.
