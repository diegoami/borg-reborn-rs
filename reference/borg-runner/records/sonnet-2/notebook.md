# Notes from sonnet-1

Written by the player during one game, in its own words, and promoted
here unedited. Every line carries the ask that taught it; the run
replays from its seed and keys, so any line can be checked.

- seed: 7
- character: sonnet-1
- game: angband/angband e473720ed88a
- patch: 5c02c7cbbcdd
- reached: depth 1 in 125 asks, 644 keypresses, turn 4659
- outcome: ended
- keys: 3258a3a6f41f
- notebook: 50b57559bebc

## The notes
- Store purchase: `p` + item letter, then quantity prompt has default 1 shown: type the digits then Enter, then a confirm 'ESC, any other key' (asks 14-24; sending 'ph' '15\n' worked, but chaining extra keys after one purchase got eaten). Store item letters skip (a,c,f,h,j,m,n,o,q,r,u,v,y,z) so read the letter from the list, never count. (ask 25)
- Inventory letters can skip too: with 8 items the list read a-g then i (no h) (ask 57). Always read the letter off the screen before q/r/v/w. (ask 57)
- Rerolling a bad level: take < (connected stairs put you on a staircase) to town, you land on the town > , press > for a fresh DL1; costs ~3 asks (asks 67-69). Arriving by > puts you on an UP staircase (ask 29), arriving by < in town put me on > (ask 69). (ask 69)
- Throw sequence: v, a (the oil), then ' (apostrophe) = throw at the closest visible target (ask 82-83 prompt: Direction ('*' or click to target, ' for closest)). (ask 83)
- Fang (unique dog, DL1) killed at CL2 in a corridor: 2 oil throws (v a ') then melee, cost ~11 HP of 31 (asks 84-90). Reward: CL2->3. Corridor fights vs a fast unique worked. (ask 90)
- After a first throw, the direction prompt offers '5' for the current target: v a 5 re-throws at it (ask 104). Enter in the dungeon opens a command menu (ask 103), so dismiss -more- with space/escape, not Enter, when nothing else is pending. (ask 104)

- CORRECTION to the reroll notes: after < then > the new DL1 started with the same kind of corridor (same spot, '<' at east end) but the layout was NOT the same: it had a door south and a worm north, the old one had neither (asks 71-78). Rerolling gives a different level whose start looks alike; it did not help find stairs faster (ask 78+).
- Runs (.4 etc.) stop at every side opening, treasure vein (*) and door; in caves/rooms a plain run of 8-10 digit steps ('4444444444') moves further per ask than '.4' (asks 62-66, 96, 114).
- Unknown scrolls on DL1 were harmless (Blessing ask 45, Light ask 118); an unknown potion was Boldness and gave a level (ask 58) because first-use ID xp counts.
- By ask 124 no '>' found on DL1 in ~2 levels explored (~100 asks, mostly walking). Levels are big; a Deep Descent/Magic Mapping (65g) scroll or just more starting gold is what would have paid. 10 gold left unspent was wasted: Magic Mapping needs 65.
- Files actually opened before ask 1: docs/BRIEF.md, knowledge/MANUAL-GAPS.md, knowledge/LEARNED.md, the sonnet-1 notebook (printed), vendor/angband docs guide.rst (first 150 lines) and command.rst (Run section), strategy-manual ch15 first 120 lines and the grep of ch14 for Warrior (only headings, not the full playbook). Plan: shop incl. Magic Mapping, then dive. (ask 1)
- Throw menu (v) lists only throwing items with its own letters: flasks of oil were 'a' there though b in the pack (ask 40), so read the v prompt, not the pack letter. Also a whole purchase 'pa\ny ' (p, letter, Enter, accept, space for -more-) works in ONE ask (asks 19-21) - contradicts inherited 'chained keys eaten'; what is eaten is anything typed after a level change ('<>' left me in town, ask 25) and keys after a 'wall in the way' message. (ask 40)
- Unknown scroll 'ilicus aur' picked up on DL1 at ask 76 was Deep Descent: 'The air around you starts to swirl...' and status bar showed 'Descent' (ask 77). Read at full HP 68/68, CL4. Reading unknown scrolls early is how I got below depth 1 for 0 gold. (ask 78)
- Level-change flush: any keys typed after '<' / '>' / Deep Descent arrival are discarded and a '-more-' follows ('You enter a maze of down staircases'), so a reroll is 3 asks ('<', '>', ' '), not 1 (ask 25-27). Also ask 59/70: several trailing spaces after a throw ("va'   ") clear the 2-3 -more- prompts of one throw in the SAME ask; a throw is 1 ask, not 3. Walls in the way ('There is a wall in the way!') cancel the rest of a digit string, so long strings in caves are wasted after the first wall. Cutpurse p touched me and 'puff of smoke' stole ~35 gold at ask 143 (126->91). (ask 143)
- Result sonnet-2: DL1->DL6 at ask 79 with a Deep Descent scroll found on DL1 (read unknown scrolls at once: ask 76-79; 5 levels below MAX depth, arrived with 'The floor opens beneath you', NOT on a staircase). Magic Mapping on DL1 (ask 47-50) showed no stairs in radius, 65 gold for nothing; sonnet-1's idea that MM is the fix was NOT confirmed. L (locate) scrolls the map by sector: used at ask 48-50 to read the mapped area off-screen. 'l' look on a visible-looking C/y/p returned only 'You are on open floor' (ask 58,112): not usable to identify monsters; use the throw prompt or just attack. DL6 CL4: wild dogs pack+Grip at DL1 gave CL1->4 in one fight (ask 63-70); apprentice/gallant/tamer p-group at DL6 each ~1 round in a doorway at CL5 with 89HP; oil one-shots p's (ask 131,145). (ask 151)
