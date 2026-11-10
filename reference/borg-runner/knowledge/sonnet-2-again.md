# Notes from sonnet-2-again

Written by the player during one game, in its own words, and promoted
here unedited. Every line carries the ask that taught it; the run
replays from its seed and keys, so any line can be checked.
This run was handed notebook.md (32 lines) and those lines are **not** repeated here: what follows is only what this run added.


- seed: 7
- character: sonnet-2
- game: angband/angband e473720ed88a
- patch: 54a4cc8b04a2
- reached: depth 7 in 291 asks, 1107 keypresses, turn 10416
- outcome: stopped
- keys: b43d1a10c309
- notebook: 40aec126d71d

## The notes
- PATHFINDING WORKS (asks 154-230). Look mode: 'l' then '>' jumps the cursor to the nearest KNOWN down staircase (message 'You recall a down staircase, 1 S, 11 E'), then 'g' walks the whole path in one ask (110 game turns, 11 steps) and the status line then reads 'Down staircase'; with no known '>' the key does nothing (bell, cursor stays 'You are on an open floor'). 'l' then 'x' jumps the cursor to the nearest square next to unexplored space, 'g' walks there: 'lxg' is a free autoexplore hop, 1 ask each (a 200-turn walk once). CAVEAT: any key typed after 'g' in the same ask interrupts the walk after ONE step, so send 'lxg' alone (chaining 'lxglxg...' only makes 1-square hops, and in rooms with ragged walls x keeps picking the adjacent square). Runs ('.4') also die after one step in twisty cave corridors. The '>' I found on DL6 was the dead-end I had stood ON: 'There is a wall in the way!' hid the status line and I did not notice the Down staircase terrain: read the bottom-right terrain after every dead end. '[' lists visible monsters with asleep/awake and offset ('2 large kobolds (2 asleep)'), unlike 'l' which only said own square (asks 201-208). Large kobold ~65hp: 4-5 rounds at 16 dmg/round (1.3 blows, Cutlass), cost me ~50 hp of 98 incl. an apprentice; sleeping ones wake. (ask 230)
- DL6->DL7 deliberately (ask 230), run ends alive at DL7 CL8 127 max hp, ~137 asks. STAIR-SCUM: connected stairs mean '<' from a bad DL7 (LF:6 'anxious', or caves) lands on a '>' in DL6 and '>' again gives a fresh DL7: 2 asks per reroll ('< ' then '> '); LF danger digit shows on arrival, so reroll when it is 5+. 'l<g' also works (walks to the nearest known up staircase). '[' on screen is the way to identify monsters. UNKNOWN WAND/STAFF: aiming an unknown wand at a sleeping monster was Haste Monster (the tamer woke hasted, then slowed me and confused me via its spells, took ~15 asks and ~50 hp to drive off, it fled at 30% health and was not killed); the Golden Staff used once did nothing visible and stayed unknown. Do not aim unknown wands at things you do not want hasted; tamers (p) cast slow and confuse: confusion makes 1/3 of moves go astray, so every attack costs several asks. Night lizard here has 18 hp, 2x1d6: easy (35 exp at level 7). Fang (DL7 corridor) dies in two rounds at CL7, CL7->8. Stepping on a floor Cutlass auto-wields the better one, a surprise. (ask 289)
