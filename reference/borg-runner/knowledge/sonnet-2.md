# Notes from sonnet-2

Written by the player during one game, in its own words, and promoted
here unedited. Every line carries the ask that taught it; the run
replays from its seed and keys, so any line can be checked.
This run was handed sonnet-1.md (27 lines) and those lines are **not** repeated here: what follows is only what this run added.


- seed: 7
- character: sonnet-2
- game: angband/angband e473720ed88a
- patch: 5c02c7cbbcdd
- reached: depth 6 in 153 asks, 637 keypresses, turn 6056
- outcome: ended
- keys: 03a19a1ba55b
- notebook: ac1ca8c66ecd

## The notes
- Files actually opened before ask 1: docs/BRIEF.md, knowledge/MANUAL-GAPS.md, knowledge/LEARNED.md, the sonnet-1 notebook (printed), vendor/angband docs guide.rst (first 150 lines) and command.rst (Run section), strategy-manual ch15 first 120 lines and the grep of ch14 for Warrior (only headings, not the full playbook). Plan: shop incl. Magic Mapping, then dive. (ask 1)
- Throw menu (v) lists only throwing items with its own letters: flasks of oil were 'a' there though b in the pack (ask 40), so read the v prompt, not the pack letter. Also a whole purchase 'pa\ny ' (p, letter, Enter, accept, space for -more-) works in ONE ask (asks 19-21) - contradicts inherited 'chained keys eaten'; what is eaten is anything typed after a level change ('<>' left me in town, ask 25) and keys after a 'wall in the way' message. (ask 40)
- Unknown scroll 'ilicus aur' picked up on DL1 at ask 76 was Deep Descent: 'The air around you starts to swirl...' and status bar showed 'Descent' (ask 77). Read at full HP 68/68, CL4. Reading unknown scrolls early is how I got below depth 1 for 0 gold. (ask 78)
- Level-change flush: any keys typed after '<' / '>' / Deep Descent arrival are discarded and a '-more-' follows ('You enter a maze of down staircases'), so a reroll is 3 asks ('<', '>', ' '), not 1 (ask 25-27). Also ask 59/70: several trailing spaces after a throw ("va'   ") clear the 2-3 -more- prompts of one throw in the SAME ask; a throw is 1 ask, not 3. Walls in the way ('There is a wall in the way!') cancel the rest of a digit string, so long strings in caves are wasted after the first wall. Cutpurse p touched me and 'puff of smoke' stole ~35 gold at ask 143 (126->91). (ask 143)
- Result sonnet-2: DL1->DL6 at ask 79 with a Deep Descent scroll found on DL1 (read unknown scrolls at once: ask 76-79; 5 levels below MAX depth, arrived with 'The floor opens beneath you', NOT on a staircase). Magic Mapping on DL1 (ask 47-50) showed no stairs in radius, 65 gold for nothing; sonnet-1's idea that MM is the fix was NOT confirmed. L (locate) scrolls the map by sector: used at ask 48-50 to read the mapped area off-screen. 'l' look on a visible-looking C/y/p returned only 'You are on open floor' (ask 58,112): not usable to identify monsters; use the throw prompt or just attack. DL6 CL4: wild dogs pack+Grip at DL1 gave CL1->4 in one fight (ask 63-70); apprentice/gallant/tamer p-group at DL6 each ~1 round in a doorway at CL5 with 89HP; oil one-shots p's (ask 131,145). (ask 151)
