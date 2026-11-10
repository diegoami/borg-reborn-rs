# Notes from minimaxflash-2

Written by the player during one game, in its own words, and promoted
here unedited. Every line carries the ask that taught it; the run
replays from its seed and keys, so any line can be checked.

- seed: 41
- character: minimaxflash-2
- game: angband/angband e473720ed88a
- patch: 54a4cc8b04a2
- reached: depth 1 in 158 asks, 1163 keypresses, turn 8001
- outcome: ended
- keys: a28936db338f
- notebook: 9c9c854e1968

## The notes
- Files I actually opened before my first keypress: manuals/INSTRUCTIONS.md (all 232 lines), agents/minimax/COMMON.md (all 380), agents/minimax/DRUID.md (all 315), docs/BRIEF.md, records/OVERVIEW.md (to pick an unused seed), vendor/angband/lib/gamedata/class.txt (Mage block) and p_race.txt (Gnome block). I did NOT read knowledge/*.md or any other run's decisions. Seed chosen: 41 (seeds already in use: 7, 11, 23, 103). (ask 1)
- Mage's CL1 spell Find Traps, Doors & Stairs = DETECT_TRAPS/DOORS/STAIRS with effect-yx:22:40; src/effect-handler-general.c:1467 DETECT_STAIRS scans the real grid (not the remembered map) and calls square_memorize + square_light_spot on every stair square in the window, so it reveals the down staircase even if never seen. The mage's answer to the staircase hunt that cost every earlier run a hundred asks. (ask 10)
- The town of Angband 4.2.6 has a down staircase on the north wall and the player is PLACED ON IT: src/gen-cave.c town_gen_layout ends with square_set_feat(c, pgrid, FEAT_MORE) then player_place(c, p, pgrid). The status line already says 'Down staircase' in town, so '>' should go straight to depth 1 - no staircase hunt for the first level. (ask 11)
- Buying in Angband 4.2.6 is ONE ask: 'p' then the row letter, then the quantity digits, then Enter, then ANY key to accept. src/ui-store.c store_purchase: after get_quantity it prints 'Price: N' at row 1 and asks 'Buy <item>? [ESC, any other key to accept]' via store_get_check. The Enter is consumed by the quantity string prompt, so a confirmation key is still needed - that is why 'p' + 'j' + '20' + Enter alone leaves gold unchanged. (ask 14)
- Armour display in Angband 4.2.6 shops is [AC,to-armour]: Metal Scale Mail [38,+0] matches the classic AC 38, Soft Leather Armour [8,+1] is AC 8. So the first number is real armour class and the sheet line 'Armor [%d,%+d]' is [protection, AC] (src/ui-player.c:735). For a Gnome Mage with 8 HP, AC is the best gold there is: Soft Leather 104g + Metal Cap 24g + Cloak 6g = AC 12. (ask 17)
- TRAP: when a store item sells out, the whole list RENUMBERS and every letter below it changes (Soft Leather was f, so the Wicker Shields moved from m to j, Metal Caps from r to q). Read the letter off the screen again after every purchase - the letter is a label, and buying shifts the labels. (ask 18)
- The hit formula in 4.2.6 (src/player-attack.c:196 hit_chance) is hit% = 12 + 83*(to_hit - 2*AC/3)/to_hit with a 5% always-miss. So AC 12 needs to_hit>=8 and AC 20 needs to_hit>=13: a monster below to_hit 13 cannot touch me at all. With 8 HP that is the difference between one fight and three. (ask 19)
- Equip in REVERSE pack-letter order (n, m, i, g): removing an item renumbers the pack letters, but only the letters AFTER the removed one move, so taking the last one first leaves every earlier letter valid. The pack skips h, j, k, l, so 10 items are a b c d e f g i m n. (ask 24)
- Where the down staircase is: src/gen-util.c alloc_stairs() picks a random EMPTY floor square with exactly 3 adjacent walls (a dead-end corridor), reducing to 2,1,0 walls only if none exists; src/gen-cave.c handle_level_stairs() sets minsep = MIN(width,height)/4 = 16, so the down staircase is more than 16 tiles away from the up staircase in x or y. The level is the full 198x66 (classic_gen does cave_new(dungeon_hgt, dungeon_wid)). (ask 46)
- HARNESS TRAP (mine, cost 5 asks): the frontend (patch/src/main-agent.c) drops a TRAILING LITERAL NEWLINE in the keys file, so a last key sent as a real newline byte is silently lost. Write the two-character escape \n instead of a real newline - in bash that means double quotes "\n", NOT $'\n'. With a dropped Enter the rest prompt sat open and every key after it was eaten by the prompt. Also: the game can be left sitting in the LOOK/TARGETING mode ('Press '?' for help.' at the bottom of the screen); press Escape before any other command. (ask 72)
- Aiming the cursor at an UNKNOWN square gives a bogus short path (find_path with only_known=false assumes unknown squares are passable and the reconstructed path can be nonsense - aim-20 gave 0 steps, aim-3 gave 3). Aiming at a KNOWN square gives a proper known-only path and a long walk. Use known landmarks (stairs, doors you have seen) as aiming points. (ask 90)
- OBSERVED (minimaxflash-2, asks 72-109): the Find Traps/Doors/Doors & Stairs spell only reports NEW squares - casting it again from within 10 tiles of the previous cast says 'You sense no traps. You sense no doors.' even though the window is the same. So the spell is a scanner of UNSCANNED area, and to widen the search the mage must MOVE between casts (the DETECT effects memorise what they find, so a second cast over the same window finds nothing new). Also: casting Find Traps from a spot where no stair is in the 81x45 window (the code scans player->grid.y +/- 22 and player->grid.x +/- 40) prints NO message at all about stairs - the DETECT_STAIRS handler only says 'You sense no stairs.' when context->aware is set, so silence means 'no stairs here' but you cannot tell it apart from 'the message was overwritten'. (ask 109)
