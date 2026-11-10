# sonnet-1 report (seed 7, Half-Troll Warrior)

## 1. What happened
- Alive at ask 124, turn ~4660, **depth 1 (50 ft), character level 3** (52/52 HP), max depth 1. Never found a `>` on DL1.
- Town: 146 gold -> 15 Flasks of oil (45), Cloak (6), 3 Phase Door (45), 2 CLW (40). 10 gold left.
- DL1 first level (asks 29-67): killed a centipede, ant, fruit bat, worms; got to CL2 by drink-testing a Boldness potion. Level was twisty cave, found only a second `<`.
- Took `<` to town and `>` again (asks 67-70): a different DL1 with a similar start. There Fang, Farmer Maggot's Dog (asks 81-90) was killed at CL2 in a corridor with 2 thrown oil + melee: CL2 -> CL3. Then wild cat, 2 giant yellow centipedes (one by oil). No `>` found by ask 124 after exploring west rooms, a long E-W corridor, a lit room and an east door.
- No deaths, no near-death (lowest HP 20/31 vs Fang).

## 2. Decisions I would take back
- Ask 14-22 shopping: spent 10 gold nowhere; should have bought Flask count 12 and a 4th Phase. Minor.
- Ask 62-66: spent ~6 asks in a twisty cave; the `<` reroll at ask 67 should have been taken at ask ~35 once the first corridors showed cave.
- Ask 99 onward: walked into unlit rooms without looking; fine at DL1 but a wild cat took 13 HP unseen.
- The real cost was **walking asks**: ~100 asks on one level with no stairs found. I would buy Magic Mapping (65g) is not affordable; instead I would read every unknown scroll early (Deep Descent / Teleport / Magic Mapping would be found at DL1-2).

## 3. Where the manual was wrong or not enough
- jev-1 says TOWN-01 "buy escapes first" and OIL-01 "ten or more oil": 15 oil was used well (Fang). Phase/CLW were never needed on DL1, so this budget was not tested.
- Neither manual tells how to *find* stairs efficiently on a big level without Magic Mapping/Deep Descent; the brief says descend is the point but at 146 gold nothing buys it.
- Manual does not say that `<` then `>` yields a new level (it does, ask 70) nor that runs stop at every opening.

## 4. What I could not see / do
- Monster identity: `l`(look) showed nothing for sleeping out-of-LOS monsters (ask 40); I attacked by letter only. Wanted: recall without LOS.
- Full map: no way to see the level overview cheaply (didn't try `L`/`M`; did not budget it). Would have helped choose unexplored directions.
- Store/prompt chaining: extra keys after a purchase got eaten (ask 20-21).

## 5. Leads
1. *Stair-reroll gives a fresh level at 3 asks* (asks 67-70). Evidence: layout differed. Test: do it at turn ~800 when a level looks like a cave; compare asks-to-first-`>` over several seeds.
2. *Corridor + 2 oil + melee beats Fang at CL2 for ~11 HP* (asks 84-90). Test: takeover --at-turn 2618 and replay the same choice vs fighting in the open room.
3. *A pure walk-explore policy spends >100 asks per DL1 level.* Test: count asks to first `>` over several fresh seeds with and without reading unknown scrolls / using Deep Descent.
