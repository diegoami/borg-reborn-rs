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
- Opened before play: docs/BRIEF.md, knowledge/MANUAL-GAPS.md, LEARNED.md, sonnet-1.md, guide.rst (1-150), command.rst (run/rest/throw), strategy-manual README + ch10 exploration. (ask 1)
- Contradiction: store-bought scrolls/potions ARE identified in the pack after purchase (shop showed Phase Door {unseen} before, plain after; pack listed '6 Scrolls of Phase Door'), though the purchase message said 'Scrolls titled ...'. Selling gives 0 au for everything here (ask 26: 'Give which item? ... 0 au'), so Word of Recall in the starting kit cannot be sold for gold. Mage seed 11 starts with 334 gold, not 146 (Human Mage). Study: command G then book then spell letter (asks 10-12); mac casts spell c; first cast of a spell gave 2 XP (NXT 10 -> 8, ask 14). 'pj20\ny' bought 20 oil in ONE ask (quantity then any-key confirm) - chained purchases work when the confirm is the last key, 'pn\ny' too for qty 1. (ask 26)
- ',' (comma) = hold/stay one turn; 's' prompts for a direction here (ask 46). Gas trap at DL1 gives Confused ~10+ turns (ask 45). (ask 47)
- DEATH at ask 48 (turn 1348), DL1, CL1: killed by the small kobold I had hit with one oil flask at ask 42-43. It left view (health bar went to [----------]), I assumed it was gone/dead (NXT still 8 = no kill xp), walked on, hit a gas trap (confused, ask 45), then sent ',,,,,,,,,,,,,,,,' (16 holds) blind in ONE ask (ask 47) and the kobold walked up and killed a 10-HP mage during those holds. Rule: when a monster that is not confirmed dead is around and you are disabled, send ONE or TWO keys per ask, never a batch of waits; a batch cannot be interrupted by what the screen would have shown. A hit that leaves the health bar unchanged/dashed is not a kill: no XP gain = no kill. (asks 42, 43, 47, 48)
- Mage (Human, seed 11): 10 HP at CL1 is one kobold's worth. Cast Find Traps/Doors/Stairs (`mac`, 1 mana, first cast +2 xp) on DL1 arrival showed a `>` ~13 cols west and 6 rows south of the `<` (ask 33-34) - detection works, but it shows no floor: path to the > still unknown after 12 asks. A `-more-` swallows the first key of the next string: ' mac' became ' ' + stray 'm' (dismissing) then 'a','c' ran aim/close (ask 32); send `\e` or ' ' alone, then the command, or prefix with `\e`.
- `v` shows only THROWING items with their own letters (oil = a), not pack letters (ask 41), so `va'` / `va5`.
