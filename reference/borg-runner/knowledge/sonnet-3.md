# Notes from sonnet-3

Written by the player during one game, in its own words, and promoted
here unedited. Every line carries the ask that taught it; the run
replays from its seed and keys, so any line can be checked.
This run was handed sonnet-1.md (27 lines) and those lines are **not** repeated here: what follows is only what this run added.


- seed: 11
- character: sonnet-3
- game: angband/angband e473720ed88a
- patch: 5c02c7cbbcdd
- reached: depth 1 in 47 asks, 222 keypresses, turn 1348
- outcome: died
- keys: fc5002023b66
- notebook: b5349650fb58

## The notes
- Opened before play: docs/BRIEF.md, knowledge/MANUAL-GAPS.md, LEARNED.md, sonnet-1.md, guide.rst (1-150), command.rst (run/rest/throw), strategy-manual README + ch10 exploration. (ask 1)
- Contradiction: store-bought scrolls/potions ARE identified in the pack after purchase (shop showed Phase Door {unseen} before, plain after; pack listed '6 Scrolls of Phase Door'), though the purchase message said 'Scrolls titled ...'. Selling gives 0 au for everything here (ask 26: 'Give which item? ... 0 au'), so Word of Recall in the starting kit cannot be sold for gold. Mage seed 11 starts with 334 gold, not 146 (Human Mage). Study: command G then book then spell letter (asks 10-12); mac casts spell c; first cast of a spell gave 2 XP (NXT 10 -> 8, ask 14). 'pj20\ny' bought 20 oil in ONE ask (quantity then any-key confirm) - chained purchases work when the confirm is the last key, 'pn\ny' too for qty 1. (ask 26)
- ',' (comma) = hold/stay one turn; 's' prompts for a direction here (ask 46). Gas trap at DL1 gives Confused ~10+ turns (ask 45). (ask 47)
- DEATH at ask 48 (turn 1348), DL1, CL1: killed by the small kobold I had hit with one oil flask at ask 42-43. It left view (health bar went to [----------]), I assumed it was gone/dead (NXT still 8 = no kill xp), walked on, hit a gas trap (confused, ask 45), then sent ',,,,,,,,,,,,,,,,' (16 holds) blind in ONE ask (ask 47) and the kobold walked up and killed a 10-HP mage during those holds. Rule: when a monster that is not confirmed dead is around and you are disabled, send ONE or TWO keys per ask, never a batch of waits; a batch cannot be interrupted by what the screen would have shown. A hit that leaves the health bar unchanged/dashed is not a kill: no XP gain = no kill. (asks 42, 43, 47, 48)
- Mage (Human, seed 11): 10 HP at CL1 is one kobold's worth. Cast Find Traps/Doors/Stairs (`mac`, 1 mana, first cast +2 xp) on DL1 arrival showed a `>` ~13 cols west and 6 rows south of the `<` (ask 33-34) - detection works, but it shows no floor: path to the > still unknown after 12 asks. A `-more-` swallows the first key of the next string: ' mac' became ' ' + stray 'm' (dismissing) then 'a','c' ran aim/close (ask 32); send `\e` or ' ' alone, then the command, or prefix with `\e`.
- `v` shows only THROWING items with their own letters (oil = a), not pack letters (ask 41), so `va'` / `va5`.
