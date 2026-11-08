# D0010: retreat pre-empts available archery

[Owner feedback](https://github.com/diegoami/better-borg-c/issues/163#issuecomment-6718075726) · [Focused investigation239](https://github.com/diegoami/better-borg-c/issues/239)

## Overall picture

A normal-mode High-Elf Ranger, level1, died to a kobold on dungeon level1 at
turn6309. Seed2060687716, source `1cd426af4848f36d8f027bfae5114c7efcd572f6`,
binary `8f507b27432083659e88d495806ab89598d44dae9818e27318e3fd0dd2781827`.
The original complete recording is VERIFIED, as is a separate historical native
combat observer against every original recorded channel, frame and native save.
These are the actual historical identities, not today's changed Borg.

The owner saw Phase Door at6255, a return toward the stairs and no fighting back
against the kobold. Native evidence supports this sequence. The character had a
Short Bow(x2)(+0,+0), eleven Arrows(1d4)(+0,+0), a Main Gauche(1d5), armour and
food. No known cure or Recall was carried; the one unknown white potion is not a
known usable heal. At level1 the ranger had zero maximum mana, so the book did
not supply an available spell. This is not an ammunition shortage. Earlier bow
shots and melee prove the attack mechanisms were working in this same life.

The first Phase Door is plausible: fifteen HP against potential two eight-damage
kobold blows makes walking away hazardous. The consequential unresolved choice
is the later repeated desperate-stair flow instead of an attack or another
escape direction. At6295 the staircase is geometrically two grids away, but its
accepted flow cost is25. Geometric proximity does not establish a clear legal
route. The native call order admits this retreat before ordinary attack; it does
not prove the bow's exact legality/score at each missed opportunity. Three actual
kobold hits killed the ranger:8,6,2 damage, HP15→7→1→-1. Doubled Borg HIT_BY
notes are not additional attacks.

## Decision timeline

| Turn | Known resources/situation | Chosen action | Alternative/evidence |
| --- | --- | --- | --- |
|5867/5887/5907|Bow/ammo and melee available|Shoots white icky thing/blue yeek; attacks in melee|Earlier native actions rule out a global inability to fight|
|6255|HP15, sole known Phase; kobold adjacent diagonally; danger16|Flee/leave and Phase Door|Walking costs possible hits; first phase not established as wrong|
|6265/6275|Phase exhausted|Upstairs flow cost6→5|Committed walking retreat is actually attempted|
|6285|Player(22,48), kobold(20,50)|Desperate upstairs flow, cost26|Inspect legal bow/escape choices and route obstruction, not just distance|
|6291/6292|Kobold hits8; next frame shows HP7|Continues retreat sequence|Native hit timestamp6291; first frame containing result is6292|
|6295|Player(21,49), stair(19,48), geometric distance2, flow25|Desperate stairs again|Attack priority and path acceptance under239; safe direct route unproved|
|6300/6301|Kobold hits6; HP1|No bow or melee in fatal tail|Compare actual legal alternatives, retaining costs and RNG divergence|
|6305|Kobold adjacent, blue yeek also visible, HP1|Reads unknown scroll, actually Blessing|A panic unidentified scroll, not a known healing/teleport resource|
|6309|Kobold hits2|Dies HP-1|Three actual HP-loss events in review-combat.jsonl|

World ticks are shared engine time, not one player decision each. Ranger speed110
(normal +0) gains10energy/tick, with a standard move costing100energy: roughly
ten ticks per move. Kobold speed111 gains11, roughly9.1ticks per standard action;
its observed hits are nine ticks apart. Some actions have different costs. Frames,
Borg input keys and think passes are not interchangeable with actions. The
observer exposes events only at their next recorded frame, avoiding future
same-tick information in the viewer.

## Owner's saved alternative

The owner took over at6255, tried for stairs, encountered the obstruction, took
a kobold hit, phased, hid/rested and used archery while returning. The complete
saved manual session is now VERIFIED with the separately bounded wide-terminal
reader. It ended at7303, level2, HP12, dungeon1, with both enemies killed.
[Branch identity and hashes](owner-manual-branch.json) retain the original link.

Native messages refine the owner's account: six confirmed arrow hits (6375,
6575,6595,7257,7283,7293); an additional arrow was spent7267 without a hit message.
The blue yeek was finished in melee7277; the kobold was shot dead7293. This does
not support a universal claim that every arrow hits. Rest between contacts
allowed recovery. Initial kobold damage6263 was7; later yeek hits dealt3 and1,
and a later kobold hit dealt2. See owner-combat.jsonl.

This is a saved/loaded manual diagnostic alternative, not an independent Borg
life or a fair population comparison. Native load compacts monster IDs, private
Borg memory is not restored, different decisions change RNG, and the owner's
terminal was424×104 rather than80×24. The original reader rejected that valid
geometry; its failed receipt is retained in manual-original-reader-failure.json.
The separate compatibility observer verifies the complete movie without changing
its original files or engine. Zero width, width4097 and excessive cells remain
rejected. [Product correction238](https://github.com/diegoami/better-borg-c/issues/238)
and [delivery240](https://github.com/diegoami/better-borg-c/pull/240) cover this
recorder issue independently of gameplay.

## Brainstorming and candidate fit

| Candidate | Support | Uncertainty / competing explanation | Costs / possible harm | Action |
| --- | --- | --- | --- | --- |
|Desperate route pre-empts usable archery|11arrows, previous working attacks, long-cost near-stair retreat and no fatal-tail attack|Exact legal attack/score and remaining route constraints need native decision diagnostics|Fighting can waste escape windows or expose the ranger to multiple enemies|239: compare legal actions and call-order admission|
|Geometric staircase test admits poor path|distance2 versus accepted cost25|Walls, danger or remembered blocked grids may explain the detour|Rejecting uncertain paths can remove the only escape|239/shared159: inspect path and alternatives, not an arbitrary threshold|
|First Phase too early|Only escape scroll consumed6255|Two potential kobold hits exceed15HP; owner also needed Phase|Saving the scroll can kill before using it|Not proved wrong; evaluate independently|
|Preparedness/cure reserve|No known cure or Recall; several town purchases before dive|Unknown potion cannot be treated as a known cure; baseline shopping priorities unresolved|More town time/cost, delayed progress|Shared41/154; do not invent a missing stock bug|
|Blind panic Blessing|Unknown scroll read at1HP with no Phase|Attack or move may already be too late|Earlier identification has economic/action costs|Shared180; secondary, not the first missed opportunity|
|Combat viewer hides attacks|Old display used Borg notes/coalesced top line|Those notes cannot yield exact native hit counts|Historical helper build/verification costs time|240 adds qualified native messages and HP loss|

## Priorities and next evidence

First,239 should inspect legal bow shots, retreat route and action scores at6285
and6295 on this exact historical state; compare a single scoped decision change
with unchanged controls. Check which subsequent carries already alter those
sites before proposing a reusable source defect. Current consumer main contains
later flee/stair/back-away changes absent from this death. Do not relabel this
historical case as a failure of the new build. A proved original-C defect would
need source-tracker ownership and a separate correction/carry.

Second, retain shared159 walking-retreat and41/154 stock investigations without
combining policy changes. Identification is secondary here: known working bow
and ammo were already available. There is no evidence of pack pressure, a
missing mana spell, or a guaranteed safe original-route escape. Ordinary bad
luck and the enemy's slight speed advantage contribute to the outcome.

## Outcome and limits

Original death, actual three-hit sequence and saved manual survival are verified.
A successful human alternative motivates investigation; it does not prove a
Borg strategy improves survival or long-term progress. No gameplay policy is
implemented from this review. Any implementation experiment follows the separate
[improvement workflow](../../measurement/DEATH-IMPROVEMENT-WORKFLOW.md). Owner
review completion is not inferred from the instruction to save. Future collection
uses normal mode unless explicitly re-enabled; historical mode records stay intact.
