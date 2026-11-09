# Appendix A2. Uniques

> **Generated file - do not edit by hand.** Produced by `docs/c-manual/tools/gen_appendices.py` from `lib/gamedata/monster.txt`, `lib/gamedata/monster_base.txt`, `lib/gamedata/monster_spell.txt`, `lib/gamedata/projection.txt`, `src/list-mon-spells.h` of the Angband source tree at commit `e473720ed`. Regenerate with `python3 docs/c-manual/tools/gen_appendices.py --src <angband-source-root>` (run from the repository root).

One card per unique monster (101 in total), in order of native depth: statistics, melee, every spell with its maximum damage, defences and flags, escorts, drops and the in-game description. Uniques always have exactly their listed HP (`src/mon-make.c`: "Uniques get a fixed amount of HP"), so their breath damage is exact at full health and drops as they are wounded. Tactics are in [Chapter 8](../08-monsters.md).

## Contents

- [Legend](#legend)
- [Quick index](#quick-index)
- [Cards](#cards)

## Legend

- **Exp** is per monster level: the kill is worth `exp * depth / player level`.
- **Sleep**: initial sleepiness (0 = always awake); **Hearing**/**Smell**: detection ranges; **R**: rarity (0 = only appears via shape change or quest).
- **Spells** list each spell with the lore text at this unique's spell power, its maximum damage (breaths at full HP) and, for saving-throw spells, *save* (a successful saving throw negates it, `do_mon_spell()` in `src/mon-spell.c`).
- **Drops** (`mon_create_drop_count()` in `src/mon-make.c`): `DROP_1` = 1 object, `DROP_2` = 1-3, `DROP_3` = 2-4, `DROP_4` = 2-6, and each of `DROP_20/40/60` adds one object with that % chance (they add up); objects are made at level `max((monster level + depth) / 2, monster level)`; `DROP_GOOD`/`DROP_GREAT` = good/great objects, `ONLY_ITEM`/`ONLY_GOLD` restrict to items or gold; `drop:` lines are specific items with % chance and count range; `any <tval> N%` is a `drop-base:` line.
- **Biggest hit** in the index is the larger of the strongest single spell and the total of one full melee round (max dice).

## Quick index

| DL | Unique | Sym | Spd | HP | AC | Biggest hit |
|---:|---|:-:|---:|---:|---:|---|
| 0 | [Farmer Maggot](#farmer-maggot) | `h` | +0 | 350 | 12 | - |
| 0 | [Father Christmas](#father-christmas) | `h` | -10 | 1000 | 12 | - |
| 2 | [Fang, Farmer Maggot's Dog](#fang-farmer-maggots-dog) | `C` | +10 | 25 | 36 | melee round 4 |
| 2 | [Grip, Farmer Maggot's Dog](#grip-farmer-maggots-dog) | `C` | +10 | 25 | 36 | melee round 4 |
| 3 | [Sméagol](#sméagol) | `h` | +20 | 400 | 14 | melee round 4 |
| 5 | [Bullroarer the Hobbit](#bullroarer-the-hobbit) | `h` | +10 | 60 | 12 | melee round 16 |
| 7 | [Mughash the Kobold Lord](#mughash-the-kobold-lord) | `k` | +0 | 150 | 30 | melee round 30 |
| 8 | [Lagduf, the Snaga](#lagduf-the-snaga) | `o` | +0 | 190 | 48 | melee round 38 |
| 8 | [Wormtongue, Agent of Saruman](#wormtongue-agent-of-saruman) | `p` | +0 | 250 | 45 | BO\_COLD 50 |
| 9 | [Brodda, the Easterling](#brodda-the-easterling) | `p` | +0 | 210 | 37 | melee round 48 |
| 10 | [Grishnákh, the Hill Orc](#grishnákh-the-hill-orc) | `o` | +0 | 230 | 30 | melee round 44 |
| 10 | [Orfax, Son of Boldor](#orfax-son-of-boldor) | `y` | +10 | 120 | 24 | melee round 17 |
| 12 | [Golfimbul, the Hill Orc Chief](#golfimbul-the-hill-orc-chief) | `o` | +0 | 240 | 90 | melee round 44 |
| 13 | [Boldor, King of the Yeeks](#boldor-king-of-the-yeeks) | `y` | +10 | 180 | 28 | melee round 26 |
| 14 | [Ufthak of Cirith Ungol](#ufthak-of-cirith-ungol) | `o` | +0 | 320 | 75 | melee round 48 |
| 16 | [Ulfast, Son of Ulfang](#ulfast-son-of-ulfang) | `p` | +0 | 340 | 48 | melee round 60 |
| 17 | [Nár, the Dwarf](#nár-the-dwarf) | `h` | +0 | 450 | 84 | MIND\_BLAST 64 |
| 19 | [Gorbag, the Orc Captain](#gorbag-the-orc-captain) | `o` | +0 | 400 | 72 | melee round 78 |
| 19 | [Old Man Willow](#old-man-willow) | `l` | +0 | 1000 | 100 | melee round 52 |
| 19 | [Shagrat, the Orc Captain](#shagrat-the-orc-captain) | `o` | +0 | 400 | 72 | melee round 78 |
| 20 | [Bolg, Son of Azog](#bolg-son-of-azog) | `o` | +10 | 500 | 60 | melee round 72 |
| 21 | [Lugdush, the Uruk](#lugdush-the-uruk) | `o` | +0 | 640 | 108 | melee round 72 |
| 22 | [Uglúk, the Uruk](#uglúk-the-uruk) | `o` | +0 | 720 | 114 | melee round 78 |
| 23 | [Azog, Enemy of the Dwarves](#azog-enemy-of-the-dwarves) | `o` | +10 | 900 | 96 | melee round 75 |
| 24 | [Angamaitë of Umbar](#angamaitë-of-umbar) | `p` | +0 | 800 | 96 | melee round 96 |
| 24 | [Ibun, Son of Mîm](#ibun-son-of-mîm) | `h` | +0 | 820 | 96 | BO\_FIRE 80 |
| 24 | [Khîm, Son of Mîm](#khîm-son-of-mîm) | `h` | +0 | 820 | 96 | BO\_FIRE 80 |
| 24 | [Sangahyando of Umbar](#sangahyando-of-umbar) | `p` | +0 | 800 | 96 | melee round 96 |
| 26 | [Ulwarth, Son of Ulfang](#ulwarth-son-of-ulfang) | `p` | +0 | 850 | 48 | melee round 72 |
| 27 | [Mîm, Betrayer of Turin](#mîm-betrayer-of-turin) | `h` | +10 | 1100 | 96 | BA\_ACID 96 |
| 28 | [Beorn, the Mountain Bear](#beorn-the-mountain-bear) | `q` | +10 | shape | 72 | melee round 90 |
| 28 | [Beorn, the Shape-Changer](#beorn-the-shape-changer) | `p` | +0 | 1400 | 72 | melee round 56 |
| 28 | [Draebor, the Imp](#draebor-the-imp) | `u` | +10 | 520 | 75 | melee round 36 |
| 28 | [Uldor the Accursed](#uldor-the-accursed) | `p` | +0 | 1000 | 84 | melee round 96 |
| 30 | [Bert the Stone Troll](#bert-the-stone-troll) | `T` | +10 | 1100 | 84 | melee round 102 |
| 30 | [Bill the Stone Troll](#bill-the-stone-troll) | `T` | +10 | 1100 | 84 | melee round 102 |
| 30 | [Tom the Stone Troll](#tom-the-stone-troll) | `T` | +10 | 1100 | 84 | melee round 102 |
| 34 | [Ulfang the Black](#ulfang-the-black) | `p` | +10 | 1000 | 108 | melee round 100 |
| 35 | [Lokkak, the Ogre Chieftain](#lokkak-the-ogre-chieftain) | `O` | +10 | 1500 | 120 | melee round 108 |
| 36 | [Lorgan, Chief of the Easterlings](#lorgan-chief-of-the-easterlings) | `p` | +10 | 1800 | 120 | melee round 120 |
| 38 | [Castamir the Usurper](#castamir-the-usurper) | `p` | +10 | 880 | 108 | melee round 100 |
| 41 | [Gorlim, Betrayer of Barahir](#gorlim-betrayer-of-barahir) | `p` | +10 | 1600 | 144 | melee round 192 |
| 41 | [Kavlax the Many-Headed](#kavlax-the-many-headed) | `d` | +10 | 1300 | 102 | BR\_ACID 433 |
| 42 | [Rogrog the Black Troll](#rogrog-the-black-troll) | `T` | +10 | 2000 | 105 | melee round 156 |
| 42 | [The Queen Ant](#the-queen-ant) | `a` | +10 | 1500 | 120 | melee round 80 |
| 43 | [Ariel, Queen of Air](#ariel-queen-of-air) | `E` | +20 | 2700 | 75 | melee round 80 |
| 43 | [Quaker, Master of Earth](#quaker-master-of-earth) | `E` | +0 | 2800 | 145 | melee round 208 |
| 43 | [Vargo, Tyrant of Fire](#vargo-tyrant-of-fire) | `E` | +10 | 2400 | 75 | BA\_FIRE 160 |
| 43 | [Waldern, King of Water](#waldern-king-of-water) | `E` | +10 | 2500 | 120 | BA\_WATE 157 |
| 49 | [Eöl, the Dark Elf](#eöl-the-dark-elf) | `h` | +20 | 2400 | 150 | BA\_NETH 296 |
| 50 | [The Balrog of Moria](#the-balrog-of-moria) | `U` | +20 | 3000 | 150 | BR\_FIRE 1000 |
| 51 | [Harowen the Black Hand](#harowen-the-black-hand) | `p` | +30 | 2500 | 108 | melee round 140 |
| 52 | [Itangast the Fire Drake](#itangast-the-fire-drake) | `D` | +10 | 2200 | 144 | BR\_FIRE 733 |
| 53 | [Scatha the Worm](#scatha-the-worm) | `D` | +10 | 2500 | 156 | BR\_COLD 833 |
| 54 | [The Phoenix](#the-phoenix) | `B` | +10 | 3600 | 156 | BR\_FIRE 1200 |
| 55 | [Ar-Pharazôn the Golden](#ar-pharazôn-the-golden) | `p` | +20 | 4000 | 54 | melee round 256 |
| 55 | [Shelob, Spider of Darkness](#shelob-spider-of-darkness) | `S` | +10 | 3500 | 180 | BR\_POIS 800 |
| 58 | [Baphomet the Minotaur Lord](#baphomet-the-minotaur-lord) | `H` | +20 | 4700 | 144 | melee round 512 |
| 59 | [Fundin Bluecloak](#fundin-bluecloak) | `h` | +20 | 5000 | 234 | melee round 256 |
| 60 | [Gilim, the Giant of Eruman](#gilim-the-giant-of-eruman) | `P` | +10 | 4000 | 96 | STORM 370 |
| 60 | [Saruman of Many Colours](#saruman-of-many-colours) | `A` | +10 | 5000 | 120 | BA\_FIRE 220 |
| 62 | [Smaug the Golden](#smaug-the-golden) | `D` | +10 | 4200 | 195 | BR\_FIRE 1400 |
| 65 | [The Mouth of Sauron](#the-mouth-of-sauron) | `p` | +20 | 7000 | 120 | BA\_MANA 425 |
| 66 | [Tevildo, Prince of Cats](#tevildo-prince-of-cats) | `f` | +20 | 4800 | 240 | melee round 233 |
| 67 | [Thuringwethil, the Vampire Messenger](#thuringwethil-the-vampire-messenger) | `V` | +20 | 5000 | 174 | BA\_NETH 368 |
| 68 | [Tselakus, the Dreadlord](#tselakus-the-dreadlord) | `G` | +20 | 6500 | 180 | BA\_DARK 372 |
| 69 | [Nan, the Giant](#nan-the-giant) | `P` | +10 | 3344 | 150 | melee round 576 |
| 72 | [Ossë, Herald of Ulmo](#ossë-herald-of-ulmo) | `A` | +20 | 6000 | 204 | BR\_DISE 500 |
| 73 | [Omarax, the Eye Tyrant](#omarax-the-eye-tyrant) | `e` | +20 | 6500 | 96 | BA\_DARK 392 |
| 75 | [Glaurung, Father of the Dragons](#glaurung-father-of-the-dragons) | `D` | +20 | 7500 | 210 | BR\_FIRE 1600 |
| 75 | [Ungoliant, the Unlight](#ungoliant-the-unlight) | `S` | +20 | 13000 | 240 | BR\_POIS 800 |
| 76 | [Makar, the Warrior](#makar-the-warrior) | `A` | +10 | 6000 | 192 | melee round 676 |
| 77 | [Feagwath, the Undead Sorcerer](#feagwath-the-undead-sorcerer) | `L` | +20 | 5000 | 127 | BA\_MANA 485 |
| 77 | [Uvatha the Horseman](#uvatha-the-horseman) | `W` | +10 | 4200 | 72 | WOUND 250 |
| 78 | [Adunaphel the Quiet](#adunaphel-the-quiet) | `W` | +10 | 4300 | 72 | WOUND 260 |
| 79 | [Akhorahil the Blind](#akhorahil-the-blind) | `W` | +10 | 4400 | 84 | WOUND 260 |
| 79 | [Qlzqqlzuup, the Emperor Quylthulg](#qlzqqlzuup-the-emperor-quylthulg) | `Q` | +20 | 5000 | 1 | - |
| 80 | [Ren the Unclean](#ren-the-unclean) | `W` | +10 | 4500 | 84 | BA\_FIRE 290 |
| 80 | [Wiruin, the Maelstrom](#wiruin-the-maelstrom) | `v` | +30 | 3500 | 10 | BR\_COLD 1166 |
| 81 | [Ji Indur Dawndeath](#ji-indur-dawndeath) | `W` | +10 | 4600 | 84 | BA\_NETH 424 |
| 81 | [Maeglin, the Traitor of Gondolin](#maeglin-the-traitor-of-gondolin) | `h` | +20 | 6000 | 144 | melee round 256 |
| 82 | [Dwar, Dog Lord of Waw](#dwar-dog-lord-of-waw) | `W` | +10 | 4700 | 108 | BA\_NETH 428 |
| 82 | [Pazuzu, Lord of Air](#pazuzu-lord-of-air) | `U` | +30 | 5500 | 150 | melee round 576 |
| 83 | [Draugluin, Sire of All Werewolves](#draugluin-sire-of-all-werewolves) | `C` | +20 | 7000 | 135 | melee round 168 |
| 83 | [Hoarmurath of Dir](#hoarmurath-of-dir) | `W` | +10 | 4800 | 120 | BA\_NETH 432 |
| 84 | [Cantoras, the Skeletal Lord](#cantoras-the-skeletal-lord) | `s` | +30 | 7500 | 180 | BA\_NETH 540 |
| 84 | [Khamûl, the Black Easterling](#khamûl-the-black-easterling) | `W` | +10 | 4900 | 120 | BA\_NETH 436 |
| 85 | [Ancalagon the Black](#ancalagon-the-black) | `D` | +20 | 10000 | 255 | BR\_ACID 1600 |
| 85 | [The Witch-King of Angmar](#the-witch-king-of-angmar) | `W` | +20 | 5000 | 144 | BA\_NETH 440 |
| 86 | [The Tarrasque](#the-tarrasque) | `R` | +20 | 8500 | 222 | BR\_COLD 1600 |
| 87 | [Meássë, the Bloody](#meássë-the-bloody) | `A` | +10 | 7000 | 180 | melee round 576 |
| 88 | [Lungorthin, the Balrog of White Fire](#lungorthin-the-balrog-of-white-fire) | `U` | +20 | 7000 | 187 | BR\_FIRE 1600 |
| 90 | [Carcharoth, the Jaws of Thirst](#carcharoth-the-jaws-of-thirst) | `C` | +20 | 8000 | 192 | BR\_FIRE 1600 |
| 90 | [Huan, Wolfhound of the Valar](#huan-wolfhound-of-the-valar) | `C` | +20 | 8000 | 192 | BR\_COLD 1600 |
| 92 | [Vecna, the Emperor Lich](#vecna-the-emperor-lich) | `L` | +20 | 6000 | 120 | BA\_MANA 560 |
| 95 | [Gothmog, the High Captain of Balrogs](#gothmog-the-high-captain-of-balrogs) | `U` | +20 | 8000 | 168 | BR\_FIRE 1600 |
| 99 | [Sauron, the Sorcerer](#sauron-the-sorcerer) | `A` | +20 | 8000 | 192 | BA\_DARK 620 |
| 99 | [Serpent-Sauron](#serpent-sauron) | `J` | +20 | shape | 164 | WOUND 360 |
| 99 | [Vampire-Sauron](#vampire-sauron) | `V` | +20 | shape | 164 | STORM 715 |
| 99 | [Wolf-Sauron](#wolf-sauron) | `C` | +30 | shape | 164 | BA\_DARK 520 |
| 100 | [Morgoth, Lord of Darkness](#morgoth-lord-of-darkness) | `P` | +30 | 20000 | 180 | BA\_MANA 600 |

## Cards

### Farmer Maggot

- **Depth** 0, **speed** +0, **HP** 350, **AC** 12, **exp** 0/level, **rarity** 4, **sleep** 3, **hearing** 40, **light** 2
- **Melee**: moan, moan
- **Spells**: none
- **Defences**: no-conf, no-sleep
- **Flags**: spirit, opens doors, bashes doors
- **Other flags**: CLEAR\_WEB, UNIQUE, MALE
- **Drops**: DROP\_1, DROP\_GOOD, ONLY\_ITEM, any mushroom 100% x1-2

> He's lost his dogs. He's had his mushrooms stolen. He's not a happy hobbit!

### Father Christmas

- **Depth** 0, **speed** -10, **HP** 1000, **AC** 12, **exp** 0/level, **rarity** 1, **sleep** 3, **hearing** 80, **light** 2
- **Melee**: -
- **Spells** (spells 1/4, spell power 0):
  - `BLINK`: blink-self
  - `HEAL`: heal-self
- **Defences**: no-conf, no-sleep
- **Flags**: spirit, opens doors, bashes doors
- **Other flags**: CLEAR\_WEB, SEASONAL, UNIQUE
- **Drops**: DROP\_1, DROP\_GOOD, ONLY\_ITEM

> It's Saint Nick, out delivering presents to the town urchins.

### Fang, Farmer Maggot's Dog

- **Depth** 2, **speed** +10, **HP** 25, **AC** 36, **exp** 30/level, **rarity** 1, **sleep** 0, **hearing** 30, **smell** 50
- **Melee**: bite 1d4
- **Spells**: none
- **Defences**: no-conf, no-sleep
- **Flags**: erratic 25%, bashes doors, animal
- **Other flags**: CLEAR\_WEB, UNIQUE, NAME\_COMMA
- **Drops**: none

> A rather vicious dog belonging to Farmer Maggot. It thinks you are stealing mushrooms.

### Grip, Farmer Maggot's Dog

- **Depth** 2, **speed** +10, **HP** 25, **AC** 36, **exp** 30/level, **rarity** 1, **sleep** 0, **hearing** 30, **smell** 50
- **Melee**: bite 1d4
- **Spells**: none
- **Defences**: no-conf, no-sleep
- **Flags**: erratic 25%, bashes doors, animal
- **Other flags**: CLEAR\_WEB, UNIQUE, NAME\_COMMA
- **Drops**: none

> A rather vicious dog belonging to Farmer Maggot. It thinks you are stealing mushrooms.

### Sméagol

- **Depth** 3, **speed** +20, **HP** 400, **AC** 14, **exp** 50/level, **rarity** 2, **sleep** 5, **hearing** 20
- **Melee**: touch eat\_gold 1d4
- **Spells**: none
- **Defences**: hurt-light
- **Flags**: INVISIBLE, erratic 25%, erratic 50%, spirit, takes items, opens doors, bashes doors, evil
- **Other flags**: CLEAR\_WEB, UNIQUE, MALE
- **Drops**: DROP\_1, DROP\_GOOD, DROP\_GREAT

> Usually known as Gollum. He's been sneaking, and he wants his 'precious.'

### Bullroarer the Hobbit

- **Depth** 5, **speed** +10, **HP** 60, **AC** 12, **exp** 90/level, **rarity** 3, **sleep** 30, **hearing** 16
- **Melee**: hit 1d8, hit 1d8
- **Spells**: none
- **Defences**: -
- **Flags**: spirit, opens doors, bashes doors
- **Other flags**: CLEAR\_WEB, UNIQUE, MALE
- **Escorts / friends**: 80% 1d7 Scruffy looking hobbit (servant)
- **Drops**: DROP\_2, DROP\_GOOD, ONLY\_ITEM

> He is a sturdy hobbit who is renowned for his unusual strength and vigour. He can prove a troublesome opponent.

### Mughash the Kobold Lord

- **Depth** 7, **speed** +0, **HP** 150, **AC** 30, **exp** 100/level, **rarity** 3, **sleep** 40, **hearing** 20, **smell** 20
- **Melee**: hit 1d10, hit 1d10, hit 1d10
- **Spells**: none
- **Defences**: im-pois
- **Flags**: spirit, move-body, opens doors, bashes doors, evil
- **Other flags**: CLEAR\_WEB, UNIQUE, MALE
- **Escorts / friends**: 100% 1d10 Small kobold (servant); 100% 1d10 Kobold (servant); 100% 1d3 Kobold shaman (servant); 100% 1d5 Kobold archer (servant); 100% 1d5 Large kobold (bodyguard)
- **Drops**: DROP\_1, DROP\_GOOD, ONLY\_ITEM

> Strong and powerful, for a kobold.

### Lagduf, the Snaga

- **Depth** 8, **speed** +0, **HP** 190, **AC** 48, **exp** 80/level, **rarity** 2, **sleep** 50, **hearing** 20, **smell** 20
- **Melee**: hit 1d10, hit 1d10, hit 1d9, hit 1d9
- **Spells**: none
- **Defences**: -
- **Flags**: spirit, opens doors, bashes doors, evil, orc
- **Other flags**: MALE, CLEAR\_WEB, UNIQUE, NAME\_COMMA
- **Escorts / friends**: 80% 2d7 Wolf; 100% 5d4 snaga (servant)
- **Drops**: DROP\_1, DROP\_GOOD, ONLY\_ITEM

> A captain of a regiment of weaker orcs, Lagduf keeps his troop in order with displays of excessive violence.

### Wormtongue, Agent of Saruman

- **Depth** 8, **speed** +0, **HP** 250, **AC** 45, **exp** 150/level, **rarity** 1, **sleep** 10, **hearing** 20
- **Melee**: hit 1d5, hit 1d5, touch eat\_gold, insult
- **Spells** (spells 1/5, spell power 8):
  - `HEAL`: heal-self
  - `SLOW`: slow (save)
  - `TRAPS`: create traps
  - `BA_POIS`: produce poison balls, max 28 damage
  - `BO_COLD`: produce frost bolts, max 50 damage
- **Defences**: -
- **Flags**: spirit, takes items, opens doors, bashes doors, evil
- **Other flags**: CLEAR\_WEB, UNIQUE, MALE, NAME\_COMMA, FORCE\_SLEEP
- **Drops**: DROP\_1, DROP\_GOOD, DROP\_GREAT, ONLY\_ITEM

> He's been spying for Saruman. He is a snivelling wretch with no morals and disgusting habits.

### Brodda, the Easterling

- **Depth** 9, **speed** +0, **HP** 210, **AC** 37, **exp** 100/level, **rarity** 2, **sleep** 40, **hearing** 20, **light** 2
- **Melee**: hit 1d12, hit 1d12, hit 1d12, hit 1d12
- **Spells**: none
- **Defences**: -
- **Flags**: spirit, opens doors, bashes doors
- **Other flags**: CLEAR\_WEB, UNIQUE, MALE, NAME\_COMMA
- **Drops**: DROP\_1, DROP\_GOOD, ONLY\_ITEM

> A nasty piece of work, Brodda picks on defenseless women and children.

### Grishnákh, the Hill Orc

- **Depth** 10, **speed** +0, **HP** 230, **AC** 30, **exp** 160/level, **rarity** 3, **sleep** 50, **hearing** 20, **smell** 20
- **Melee**: hit 1d12, hit 1d10, hit 1d12, hit 1d10
- **Spells**: none
- **Defences**: -
- **Flags**: spirit, opens doors, bashes doors, evil, orc
- **Other flags**: MALE, CLEAR\_WEB, UNIQUE, NAME\_COMMA
- **Escorts / friends**: 50% 2d5 snaga (servant); 80% 2d7 Wolf; 100% 5d4 cave orc (servant)
- **Drops**: DROP\_1, DROP\_GOOD, ONLY\_ITEM

> He is a cunning and devious orc with a chaotic nature.

### Orfax, Son of Boldor

- **Depth** 10, **speed** +10, **HP** 120, **AC** 24, **exp** 80/level, **rarity** 3, **sleep** 10, **hearing** 18, **smell** 20
- **Melee**: hit 1d9, hit 1d8, insult, insult
- **Spells** (spells 1/4, spell power 10):
  - `BLINK`: blink-self
  - `CONF`: confuse (save)
  - `HEAL`: heal-self
  - `SLOW`: slow (save)
  - `TELE_TO`: teleport to
  - `S_MONSTER`: summon a monster
- **Defences**: im-acid
- **Flags**: smart, spirit, opens doors, bashes doors, evil, animal
- **Other flags**: CLEAR\_WEB, UNIQUE, MALE, NAME\_COMMA, FORCE\_SLEEP
- **Escorts / friends**: 50% 2d5 terrified yeek (servant); 100% 2d5 Blue yeek (servant); 50% 1d3 Master yeek (servant)
- **Drops**: DROP\_1, DROP\_GOOD, ONLY\_ITEM

> He's just like his daddy! He knows mighty spells, but fortunately he is a yeek.

### Golfimbul, the Hill Orc Chief

- **Depth** 12, **speed** +0, **HP** 240, **AC** 90, **exp** 230/level, **rarity** 3, **sleep** 40, **hearing** 20, **smell** 20
- **Melee**: hit 1d12, hit 1d12, hit 1d10, hit 1d10
- **Spells**: none
- **Defences**: im-elec, im-fire, im-cold, im-pois
- **Flags**: spirit, opens doors, bashes doors, evil, orc
- **Other flags**: MALE, CLEAR\_WEB, UNIQUE, NAME\_COMMA
- **Escorts / friends**: 50% 4d4 snaga (servant); 80% 2d7 Warg; 100% 5d4 cave orc (servant); 100% 1d1 Bullroarer the Hobbit; 100% 1d1 Grishnákh, the Hill Orc
- **Drops**: DROP\_2, DROP\_GOOD, ONLY\_ITEM

> A leader of a band of raiding orcs, he picks on hobbits.

### Boldor, King of the Yeeks

- **Depth** 13, **speed** +10, **HP** 180, **AC** 28, **exp** 200/level, **rarity** 3, **sleep** 10, **hearing** 18, **smell** 20
- **Melee**: hit 1d9, hit 1d9, hit 1d8
- **Spells** (spells 1/3, spell power 13):
  - `BLIND`: blind (save)
  - `BLINK`: blink-self
  - `HEAL`: heal-self
  - `SLOW`: slow (save)
  - `TPORT`: teleport-self
  - `S_KIN`: summon similar monsters
  - `S_MONSTER`: summon a monster
- **Defences**: im-acid
- **Flags**: smart, spirit, opens doors, bashes doors, evil, animal
- **Other flags**: CLEAR\_WEB, UNIQUE, MALE, NAME\_COMMA, FORCE\_SLEEP
- **Escorts / friends**: 50% 2d6 terrified yeek (servant); 100% 2d6 Blue yeek (servant); 100% 2d4 Master yeek (servant); 100% 1d1 Orfax, Son of Boldor
- **Drops**: DROP\_60, DROP\_1, DROP\_GOOD, ONLY\_ITEM

> A great yeek, powerful in magic and sorcery, but a yeek all the same.

### Ufthak of Cirith Ungol

- **Depth** 14, **speed** +0, **HP** 320, **AC** 75, **exp** 250/level, **rarity** 3, **sleep** 40, **hearing** 20, **smell** 20
- **Melee**: hit 3d4, hit 3d4, hit 3d4, hit 3d4
- **Spells**: none
- **Defences**: im-cold
- **Flags**: spirit, opens doors, bashes doors, evil, orc
- **Other flags**: MALE, CLEAR\_WEB, UNIQUE
- **Escorts / friends**: 50% 2d2 orc archer; 100% 4d4 snaga; 100% 3d4 cave orc
- **Drops**: DROP\_1, DROP\_GOOD, ONLY\_ITEM

> A strong orc guarding the pass of Cirith Ungol. He is mortally afraid of spiders.

### Ulfast, Son of Ulfang

- **Depth** 16, **speed** +0, **HP** 340, **AC** 48, **exp** 200/level, **rarity** 3, **sleep** 40, **hearing** 20, **smell** 20, **light** 2
- **Melee**: hit 3d5, hit 3d5, hit 3d5, hit 3d5
- **Spells**: none
- **Defences**: -
- **Flags**: spirit, takes items, opens doors, bashes doors, evil
- **Other flags**: CLEAR\_WEB, UNIQUE, MALE, NAME\_COMMA
- **Escorts / friends**: 60% 1d3 warrior (servant)
- **Drops**: DROP\_60, DROP\_GOOD, ONLY\_ITEM

> A short and swarthy Easterling.

### Nár, the Dwarf

- **Depth** 17, **speed** +0, **HP** 450, **AC** 84, **exp** 250/level, **rarity** 2, **sleep** 25, **hearing** 25
- **Melee**: hit 3d5, hit 3d5, hit 3d5, hit 3d5
- **Spells** (spells 1/6, spell power 17):
  - `BLIND`: blind (save)
  - `WOUND`: cause medium wounds, max 50 damage (save)
  - `CONF`: confuse (save)
  - `HEAL`: heal-self
  - `MIND_BLAST`: cause mind blasting, max 64 damage (save)
- **Defences**: im-fire, im-cold, im-pois, no-conf, no-sleep, no-hold
- **Flags**: spirit, opens doors, bashes doors
- **Other flags**: CLEAR\_WEB, UNIQUE, MALE, NAME\_COMMA, FORCE\_SLEEP
- **Drops**: DROP\_1, DROP\_GOOD, ONLY\_ITEM

> The friend and companion of the dwarven king Thrór, he went mad with grief after Thrór's death at the hands of Azog the Orc. With torn beard and ragged clothes, he seems to have fixed on you as a convenient target to vent his anger.

### Gorbag, the Orc Captain

- **Depth** 19, **speed** +0, **HP** 400, **AC** 72, **exp** 400/level, **rarity** 2, **sleep** 20, **hearing** 20, **smell** 20
- **Melee**: hit 3d8, hit 3d8, hit 3d5, hit 3d5
- **Spells**: none
- **Defences**: -
- **Flags**: spirit, opens doors, bashes doors, evil, orc
- **Other flags**: MALE, CLEAR\_WEB, UNIQUE, NAME\_COMMA
- **Escorts / friends**: 50% 3d4 snaga (servant); 80% 2d4 cave orc (servant); 80% 1d2 orc tracker (servant); 100% 1d2 orc archer (servant); 100% 2d4 uruk (servant)
- **Drops**: DROP\_1, DROP\_GOOD, ONLY\_ITEM

> He is an orc of power and great cunning, leader of the garrison at Minas Morgul.

### Old Man Willow

- **Depth** 19, **speed** +0, **HP** 1000, **AC** 100, **exp** 3000/level, **rarity** 3, **sleep** 20, **hearing** 20
- **Melee**: touch paralyze 1d14, touch paralyze 1d14, crush 2d12
- **Spells** (spells 1/10, spell power 19):
  - `TELE_TO`: teleport to
  - `HOLD`: paralyze (save)
- **Defences**: im-acid, im-pois, im-water, hurt-fire, no-fear, no-conf
- **Flags**: cold-blooded, empty-mind (no ESP), regenerates, never-moves
- **Other flags**: CLEAR\_WEB, UNIQUE, FORCE\_SLEEP
- **Drops**: DROP\_1, DROP\_GOOD, ONLY\_ITEM

> The ancient grey willow tree, ruler of the Old Forest. He despises trespassers in his territory. "...a huge willow-tree, old and hoary. Enormous it looked, its sprawling branches going up like reaching arms with many long-fingered hands, its knotted and twisted trunk gaping in wide fissures that creaked faintly as the boughs moved."

### Shagrat, the Orc Captain

- **Depth** 19, **speed** +0, **HP** 400, **AC** 72, **exp** 400/level, **rarity** 2, **sleep** 20, **hearing** 20, **smell** 20
- **Melee**: hit 3d8, hit 3d8, hit 3d5, hit 3d5
- **Spells**: none
- **Defences**: -
- **Flags**: spirit, opens doors, bashes doors, evil, orc
- **Other flags**: MALE, CLEAR\_WEB, UNIQUE, NAME\_COMMA
- **Escorts / friends**: 50% 3d4 snaga (servant); 80% 2d4 cave orc (servant); 80% 1d2 orc tracker (servant); 100% 1d2 orc archer (servant); 100% 2d4 uruk (servant)
- **Drops**: DROP\_1, DROP\_GOOD, ONLY\_ITEM

> He is an orc of power and great cunning, leader of the garrison at Cirith Ungol.

### Bolg, Son of Azog

- **Depth** 20, **speed** +10, **HP** 500, **AC** 60, **exp** 800/level, **rarity** 4, **sleep** 20, **hearing** 20, **smell** 20
- **Melee**: hit 3d6, hit 3d6, hit 3d6, hit 3d6
- **Spells**: none
- **Defences**: -
- **Flags**: spirit, move-body, opens doors, bashes doors, evil, orc
- **Other flags**: MALE, CLEAR\_WEB, UNIQUE, NAME\_COMMA
- **Escorts / friends**: 50% 3d4 cave orc (servant); 80% 2d7 Warg; 100% 2d2 orc tracker; 100% 4d2 orc archer; 100% 4d4 Uruk (bodyguard)
- **Drops**: DROP\_2, DROP\_GOOD, ONLY\_ITEM

> A large and powerful orc, he looks just like his father. He is tall and fast, but fortunately blessed with orcish brains.

### Lugdush, the Uruk

- **Depth** 21, **speed** +0, **HP** 640, **AC** 108, **exp** 550/level, **rarity** 4, **sleep** 20, **hearing** 20, **smell** 20
- **Melee**: hit 3d7, hit 3d7, hit 3d5, hit 3d5
- **Spells**: none
- **Defences**: -
- **Flags**: spirit, opens doors, bashes doors, evil, orc
- **Other flags**: MALE, CLEAR\_WEB, UNIQUE, NAME\_COMMA
- **Escorts / friends**: 50% 3d4 Half-orc (servant); 80% 2d7 Warg; 50% 1d3 Orc captain; 100% 4d4 Uruk (servant)
- **Drops**: DROP\_1, DROP\_GOOD, ONLY\_ITEM

> A large and powerful orc, captain of one of Saruman's orcish regiments.

### Uglúk, the Uruk

- **Depth** 22, **speed** +0, **HP** 720, **AC** 114, **exp** 600/level, **rarity** 3, **sleep** 20, **hearing** 20, **smell** 20
- **Melee**: hit 3d8, hit 3d8, hit 3d5, hit 3d5
- **Spells**: none
- **Defences**: im-fire, im-cold, im-pois, no-conf, no-sleep, no-hold
- **Flags**: spirit, move-body, opens doors, bashes doors, evil, orc
- **Other flags**: MALE, CLEAR\_WEB, UNIQUE, NAME\_COMMA
- **Escorts / friends**: 50% 2d4 Half-orc (servant); 80% 2d5 Warg; 100% 4d4 Uruk (bodyguard); 80% 1d3 Orc captain (servant); 100% 1d1 Lugdush, the Uruk (servant)
- **Drops**: DROP\_1, DROP\_GOOD, ONLY\_ITEM

> A strong and cunning orc warrior, the commander of Saruman's orcish horde.

### Azog, Enemy of the Dwarves

- **Depth** 23, **speed** +10, **HP** 900, **AC** 96, **exp** 1111/level, **rarity** 5, **sleep** 20, **hearing** 20, **smell** 20
- **Melee**: hit 5d5, hit 5d5, hit 5d5
- **Spells**: none
- **Defences**: -
- **Flags**: spirit, move-body, opens doors, bashes doors, evil, orc
- **Other flags**: MALE, CLEAR\_WEB, UNIQUE, NAME\_COMMA
- **Escorts / friends**: 50% 3d4 cave orc (servant); 80% 2d7 Warg; 100% 2d2 orc tracker; 100% 4d2 orc archer; 100% 4d4 Uruk (bodyguard); 50% 1d3 Orc captain (servant); 100% 1d1 Bolg, son of Azog
- **Drops**: DROP\_2, DROP\_GOOD, ONLY\_ITEM

> A great orc with a huge iron-clad head, agile and strong. He is notorious for starting the War of the Dwarves and Orcs by killing Durin's heir, Thrór.

### Angamaitë of Umbar

- **Depth** 24, **speed** +0, **HP** 800, **AC** 96, **exp** 400/level, **rarity** 2, **sleep** 25, **hearing** 25, **light** 2
- **Melee**: hit 4d6, hit 4d6, hit 4d6, hit 4d6
- **Spells** (spells 1/4, spell power 24):
  - `CONF`: confuse (save)
  - `SLOW`: slow (save)
- **Defences**: im-elec, im-fire, no-conf, no-sleep, no-hold
- **Flags**: spirit, opens doors, bashes doors, evil
- **Other flags**: CLEAR\_WEB, UNIQUE, MALE, FORCE\_SLEEP
- **Escorts / friends**: 100% 1d1 Sangahyando of Umbar
- **Drops**: DROP\_60, DROP\_1, DROP\_GOOD, ONLY\_ITEM

> A Black Númenórean who hates the men of the west.

### Ibun, Son of Mîm

- **Depth** 24, **speed** +0, **HP** 820, **AC** 96, **exp** 300/level, **rarity** 2, **sleep** 10, **hearing** 20
- **Melee**: hit 3d6, hit 3d6, hit 3d6, hit disenchant
- **Spells** (spells 1/8, spell power 24):
  - `HEAL`: heal-self
  - `SLOW`: slow (save)
  - `BO_FIRE`: produce fire bolts, max 80 damage
- **Defences**: im-fire, im-cold, no-conf, no-sleep, no-hold
- **Flags**: spirit, opens doors, bashes doors
- **Other flags**: CLEAR\_WEB, UNIQUE, MALE, NAME\_COMMA, FORCE\_SLEEP
- **Drops**: DROP\_1, DROP\_GOOD, ONLY\_ITEM

> One of the last of the Petty-Dwarves. Ibun is a tricky sorcerous little being, full of mischief.

### Khîm, Son of Mîm

- **Depth** 24, **speed** +0, **HP** 820, **AC** 96, **exp** 300/level, **rarity** 2, **sleep** 10, **hearing** 20
- **Melee**: hit 3d6, hit 3d6, hit 3d6, hit disenchant
- **Spells** (spells 1/8, spell power 24):
  - `HEAL`: heal-self
  - `SLOW`: slow (save)
  - `BO_FIRE`: produce fire bolts, max 80 damage
- **Defences**: im-fire, im-cold, no-conf, no-sleep, no-hold
- **Flags**: spirit, opens doors, bashes doors
- **Other flags**: CLEAR\_WEB, UNIQUE, MALE, NAME\_COMMA, FORCE\_SLEEP
- **Drops**: DROP\_1, DROP\_GOOD, ONLY\_ITEM

> One of the last of the Petty-Dwarves. Khîm is a tricky sorcerous little being, full of mischief.

### Sangahyando of Umbar

- **Depth** 24, **speed** +0, **HP** 800, **AC** 96, **exp** 400/level, **rarity** 2, **sleep** 25, **hearing** 25, **light** 2
- **Melee**: hit 4d6, hit 4d6, hit 4d6, hit 4d6
- **Spells** (spells 1/4, spell power 24):
  - `CONF`: confuse (save)
  - `SLOW`: slow (save)
- **Defences**: im-elec, im-fire, no-conf, no-sleep, no-hold
- **Flags**: spirit, opens doors, bashes doors, evil
- **Other flags**: CLEAR\_WEB, UNIQUE, MALE, FORCE\_SLEEP
- **Escorts / friends**: 100% 1d1 Angamaitë of Umbar
- **Drops**: DROP\_60, DROP\_1, DROP\_GOOD, ONLY\_ITEM

> A Black Númenórean with a blacker heart.

### Ulwarth, Son of Ulfang

- **Depth** 26, **speed** +0, **HP** 850, **AC** 48, **exp** 500/level, **rarity** 4, **sleep** 40, **hearing** 20, **light** 2
- **Melee**: hit 4d6, hit 4d6, hit 4d6
- **Spells**: none
- **Defences**: -
- **Flags**: spirit, takes items, opens doors, bashes doors, evil
- **Other flags**: CLEAR\_WEB, UNIQUE, MALE, NAME\_COMMA
- **Escorts / friends**: 60% 2d3 Easterling champion (servant); 100% 1d1 Ulfast, Son of Ulfang
- **Drops**: DROP\_60, DROP\_GOOD, ONLY\_ITEM

> A short and swarthy Easterling.

### Mîm, Betrayer of Turin

- **Depth** 27, **speed** +10, **HP** 1100, **AC** 96, **exp** 1000/level, **rarity** 4, **sleep** 20, **hearing** 20
- **Melee**: hit 3d8, hit 3d8, hit 3d8, hit disenchant
- **Spells** (spells 1/6, spell power 27):
  - `HEAL`: heal-self
  - `SCARE`: terrify (save)
  - `BA_ACID`: produce acid balls, max 96 damage
  - `BO_ACID`: produce acid bolts, max 65 damage
- **Defences**: im-acid, im-elec, im-fire, im-cold, im-pois, no-conf, no-sleep, no-hold
- **Flags**: spirit, opens doors, bashes doors, evil
- **Other flags**: CLEAR\_WEB, UNIQUE, MALE, NAME\_COMMA, FORCE\_SLEEP
- **Escorts / friends**: 100% 1d1 Ibun, Son of Mîm; 100% 1d1 Khîm, Son of Mîm
- **Drops**: DROP\_2, DROP\_GOOD, DROP\_GREAT, ONLY\_ITEM

> The last of his race, Mîm is a Petty-Dwarf. Petty-Dwarves are strange creatures, powerful in sorcery and originating in the East. They have been hunted nearly to extinction by the High Elves.

### Beorn, the Mountain Bear

- **Depth** 28, **speed** +10, **HP** as the form it changed from, **AC** 72, **exp** 1000/level, **rarity** 0, **sleep** 25, **hearing** 25, **smell** 20
- **Melee**: claw 2d10, claw 2d10, bite 4d8, crush 3d6
- **Spells**: none
- **Defences**: -
- **Flags**: smart, spirit, move-body, bashes doors, animal
- **Other flags**: CLEAR\_WEB, UNIQUE, MALE, NAME\_COMMA
- **Drops**: none

> Beorn in his form as a great black bear of the mountains, grown to giant-size in wrath. The roar of his voice is like drums, and no weapon seems to bite upon him.

### Beorn, the Shape-Changer

- **Depth** 28, **speed** +0, **HP** 1400, **AC** 72, **exp** 1000/level, **rarity** 3, **sleep** 25, **hearing** 25, **smell** 20
- **Melee**: punch 2d8, punch 2d8, hit 2d6, hit 2d6
- **Spells** (spells 1/2, spell power 28):
  - `SHAPECHANGE`: change shape
- **Defences**: -
- **Flags**: smart, spirit, bashes doors
- **Other flags**: CLEAR\_WEB, UNIQUE, MALE, NAME\_COMMA, FORCE\_SLEEP
- **Escorts / friends**: 100% 2d3 Cave bear; 100% 2d3 Grizzly bear; 100% 2d3 Werebear
- **Can change into**: Beorn, the Mountain Bear
- **Drops**: DROP\_2, DROP\_GOOD, ONLY\_ITEM

> Beorn is only occasionally seen in human form these days, preferring to appear in the shape of a giant black bear: he also prefers the company of beasts to that of humans. He has never taken kindly to strangers, even in human form.

### Draebor, the Imp

- **Depth** 28, **speed** +10, **HP** 520, **AC** 75, **exp** 750/level, **rarity** 5, **sleep** 10, **hearing** 20
- **Melee**: hit poison 3d4, hit poison 3d4, hit 3d4, insult
- **Spells** (spells 1/5, spell power 28):
  - `BLIND`: blind (save)
  - `BLINK`: blink-self
  - `CONF`: confuse (save)
  - `SCARE`: terrify (save)
  - `TELE_AWAY`: teleport away
  - `TELE_LEVEL`: teleport level (save)
  - `TELE_TO`: teleport to
  - `TPORT`: teleport-self
  - `S_KIN`: summon similar monsters
- **Defences**: im-fire
- **Flags**: INVISIBLE, erratic 25%, smart, spirit, bashes doors, evil, demon
- **Other flags**: CLEAR\_WEB, UNIQUE, NAME\_COMMA, FORCE\_SLEEP
- **Escorts / friends**: 100% 2d4 Imp (servant); 50% 1d3 any minor demon (servant); 50% 1d3 any minor demon (servant); 50% 1d3 any minor demon (servant)
- **Drops**: DROP\_4, DROP\_GOOD, ONLY\_ITEM

> An intensely irritating git of a monster.

### Uldor the Accursed

- **Depth** 28, **speed** +0, **HP** 1000, **AC** 84, **exp** 600/level, **rarity** 4, **sleep** 40, **hearing** 20, **light** 2
- **Melee**: hit 4d6, hit 4d6, hit 4d6, hit 4d6
- **Spells** (spells 1/10, spell power 28):
  - `S_KIN`: summon similar monsters
- **Defences**: -
- **Flags**: spirit, takes items, opens doors, bashes doors, evil
- **Other flags**: CLEAR\_WEB, UNIQUE, MALE
- **Drops**: DROP\_1, DROP\_GOOD, ONLY\_ITEM

> An evil and cunning man from the East. Having once sworn allegiance to the sons of Fëanor, it was Uldor's treachery that turned the tide of the Battle of Unnumbered Tears in Morgoth's favour.

### Bert the Stone Troll

- **Depth** 30, **speed** +10, **HP** 1100, **AC** 84, **exp** 2000/level, **rarity** 7, **sleep** 50, **hearing** 20, **smell** 20
- **Melee**: hit 6d6, hit 6d6, bite 3d10
- **Spells** (innate 1/7, spell power 30):
  - `BOULDER`: throw boulders, max 60 damage
- **Defences**: im-cold, hurt-light, hurt-rock
- **Flags**: spirit, takes items, move-body, opens doors, bashes doors, evil, troll
- **Other flags**: CLEAR\_WEB, UNIQUE, MALE
- **Escorts / friends**: 100% 1d1 Tom the Stone Troll; 100% 1d1 Bill the Stone Troll
- **Drops**: DROP\_1, DROP\_GOOD, ONLY\_ITEM

> Big, brawny, powerful and with a taste for hobbit. He has friends called Bill and Tom.

### Bill the Stone Troll

- **Depth** 30, **speed** +10, **HP** 1100, **AC** 84, **exp** 2000/level, **rarity** 7, **sleep** 50, **hearing** 20, **smell** 20
- **Melee**: hit 6d6, hit 6d6, bite 3d10
- **Spells** (innate 1/7, spell power 30):
  - `BOULDER`: throw boulders, max 60 damage
- **Defences**: im-cold, hurt-light, hurt-rock
- **Flags**: spirit, takes items, move-body, opens doors, bashes doors, evil, troll
- **Other flags**: CLEAR\_WEB, UNIQUE, MALE
- **Escorts / friends**: 100% 1d1 Bert the Stone Troll; 100% 1d1 Tom the Stone Troll
- **Drops**: DROP\_1, DROP\_GOOD, ONLY\_ITEM

> Big, brawny, powerful and with a taste for hobbit. He has friends called Bert and Tom.

### Tom the Stone Troll

- **Depth** 30, **speed** +10, **HP** 1100, **AC** 84, **exp** 2000/level, **rarity** 7, **sleep** 50, **hearing** 20, **smell** 20
- **Melee**: hit 6d6, hit 6d6, bite 3d10
- **Spells** (innate 1/7, spell power 30):
  - `BOULDER`: throw boulders, max 60 damage
- **Defences**: im-cold, hurt-light, hurt-rock
- **Flags**: spirit, takes items, move-body, opens doors, bashes doors, evil, troll
- **Other flags**: CLEAR\_WEB, UNIQUE, MALE
- **Escorts / friends**: 100% 1d1 Bert the Stone Troll; 100% 1d1 Bill the Stone Troll
- **Drops**: DROP\_1, DROP\_GOOD, ONLY\_ITEM

> Big, brawny, powerful and with a taste for hobbit. He has friends called Bert and Bill.

### Ulfang the Black

- **Depth** 34, **speed** +10, **HP** 1000, **AC** 108, **exp** 1200/level, **rarity** 5, **sleep** 40, **hearing** 20, **light** 2
- **Melee**: hit 5d5, hit 5d5, hit 5d5, hit 5d5
- **Spells** (spells 1/10, spell power 34):
  - `S_KIN`: summon similar monsters
- **Defences**: -
- **Flags**: spirit, takes items, opens doors, bashes doors, evil
- **Other flags**: CLEAR\_WEB, UNIQUE, MALE
- **Escorts / friends**: 60% 1d3 Mûmak; 100% 3d3 Easterling champion (bodyguard); 100% 1d1 Ulfast, Son of Ulfang; 100% 1d1 Ulwarth, Son of Ulfang
- **Drops**: DROP\_2, DROP\_GOOD, ONLY\_ITEM

> A short and swarthy Easterling dressed in black. He and his three sons once openly swore allegiance to the High Elves, but were secretly in the pay of Morgoth.

### Lokkak, the Ogre Chieftain

- **Depth** 35, **speed** +10, **HP** 1500, **AC** 120, **exp** 1500/level, **rarity** 2, **sleep** 20, **hearing** 20, **smell** 20
- **Melee**: hit 6d6, hit 6d6, hit 6d6
- **Spells**: none
- **Defences**: -
- **Flags**: spirit, move-body, opens doors, bashes doors, evil, giant
- **Other flags**: CLEAR\_WEB, UNIQUE, MALE, NAME\_COMMA
- **Escorts / friends**: 100% 3d3 Ogre (servant); 100% 3d3 Black ogre (servant); 100% 3d3 Cave ogre (bodyguard); 80% 1d2 Ogre mage (servant); 80% 1d2 Ogre shaman (servant); 50% 1d2 Ogre chieftain (servant)
- **Drops**: DROP\_2, DROP\_GOOD, ONLY\_ITEM

> An ogre renowned for acts of surpassing cruelty, Lokkak quickly became the leader of a large band of violent ogres.

### Lorgan, Chief of the Easterlings

- **Depth** 36, **speed** +10, **HP** 1800, **AC** 120, **exp** 1200/level, **rarity** 2, **sleep** 10, **hearing** 25, **light** 2
- **Melee**: hit 6d6, hit 6d6, hit 3d8, hit 3d8
- **Spells** (spells 1/4, spell power 36):
  - `TELE_TO`: teleport to
  - `S_MONSTERS`: summon monsters
- **Defences**: im-acid, im-elec, im-fire, im-cold, im-pois, no-conf, no-sleep, no-hold
- **Flags**: spirit, move-body, opens doors, bashes doors, evil
- **Other flags**: CLEAR\_WEB, UNIQUE, MALE, NAME\_COMMA, FORCE\_SLEEP
- **Escorts / friends**: 50% 1d3 Mûmak; 100% 2d5 Easterling champion (bodyguard)
- **Drops**: DROP\_2, DROP\_GOOD, ONLY\_ITEM

> A mighty warrior from the east, Lorgan hates everything that he cannot control.

### Castamir the Usurper

- **Depth** 38, **speed** +10, **HP** 880, **AC** 108, **exp** 1600/level, **rarity** 5, **sleep** 40, **hearing** 20, **light** 2
- **Melee**: hit 5d5, hit 5d5, hit 5d5, hit 5d5
- **Spells** (spells 1/2, spell power 38):
  - `HEAL`: heal-self
  - `TRAPS`: create traps
  - `BO_COLD`: produce frost bolts, max 60 damage
  - `BO_ELEC`: produce lightning bolts, max 44 damage
  - `BO_FIRE`: produce fire bolts, max 84 damage
  - `BO_ICE`: produce ice bolts, max 74 damage
- **Defences**: -
- **Flags**: smart, spirit, takes items, opens doors, bashes doors, evil
- **Other flags**: CLEAR\_WEB, UNIQUE, MALE, FORCE\_SLEEP
- **Drops**: DROP\_2, DROP\_GOOD, ONLY\_ITEM

> A Black Númenórean who usurped the throne of Gondor, he is treacherous and evil.

### Gorlim, Betrayer of Barahir

- **Depth** 41, **speed** +10, **HP** 1600, **AC** 144, **exp** 7000/level, **rarity** 3, **sleep** 40, **hearing** 20, **light** 2
- **Melee**: hit 8d6, hit 8d6, hit disenchant 6d8, hit disenchant 6d8
- **Spells** (spells 1/2, spell power 41):
  - `WOUND`: cause serious wounds, max 130 damage (save)
  - `BO_MANA`: produce mana bolts, max 152 damage
  - `BO_WATE`: produce water bolts, max 141 damage
- **Defences**: im-acid, im-elec, im-cold, im-pois, no-conf, no-sleep, no-hold
- **Flags**: smart, spirit, opens doors, bashes doors
- **Other flags**: CLEAR\_WEB, UNIQUE, MALE, NAME\_COMMA, FORCE\_SLEEP
- **Drops**: DROP\_2, DROP\_GOOD, ONLY\_ITEM

> This sad creature - once a mighty warrior - betrayed his former friends to Morgoth's army in return for, he thought, safety for himself and his wife. And so he fell under Morgoth's power and became little more than a mindless servant of evil, even though the other side of his "bargain" was not kept.

### Kavlax the Many-Headed

- **Depth** 41, **speed** +10, **HP** 1300, **AC** 102, **exp** 3000/level, **rarity** 3, **sleep** 30, **hearing** 20
- **Melee**: bite 2d12, bite 2d12, bite 2d12, bite 2d12
- **Spells** (innate 1/4, spell power 41):
  - `BR_ACID`: acid, max 433 damage
  - `BR_COLD`: frost, max 433 damage
  - `BR_CHAO`: chaos, max 216 damage
  - `BR_ELEC`: lightning, max 433 damage
  - `BR_FIRE`: fire, max 433 damage
  - `BR_GRAV`: gravity, max 200 damage
  - `BR_NEXU`: nexus, max 216 damage
  - `BR_SHAR`: shards, max 216 damage
  - `BR_SOUN`: sound, max 216 damage
- **Defences**: im-acid, im-elec, im-fire, im-cold, im-nexus, no-stun, no-conf, no-sleep, no-hold
- **Flags**: spirit, powerful, bashes doors, evil, dragon
- **Other flags**: FORCE\_SLEEP, CLEAR\_WEB, UNIQUE, MALE
- **Drops**: DROP\_4, DROP\_GOOD, ONLY\_ITEM

> A large dragon with a selection of heads, all shouting and arguing as they look for prey, but each with its own deadly breath weapon.

### Rogrog the Black Troll

- **Depth** 42, **speed** +10, **HP** 2000, **AC** 105, **exp** 5000/level, **rarity** 3, **sleep** 30, **hearing** 20, **smell** 20
- **Melee**: hit 6d7, hit 6d7, bite 4d10, spit acid 4d8
- **Spells** (innate 1/7, spell power 42):
  - `BOULDER`: throw boulders, max 84 damage
- **Defences**: im-cold
- **Flags**: regenerates, spirit, takes items, move-body, opens doors, bashes doors, evil, troll
- **Other flags**: CLEAR\_WEB, UNIQUE, MALE
- **Escorts / friends**: 60% 2d3 troll scavenger (servant); 60% 2d3 cave troll (servant); 40% 2d3 hill troll (servant); 30% 1d2 mountain troll (servant); 100% 4d4 olog (bodyguard)
- **Drops**: DROP\_3, DROP\_GOOD, ONLY\_ITEM

> A massive and cruel troll of great power, drool slides caustically down his muscular frame. Despite his bulk, he strikes with stunning speed.

### The Queen Ant

- **Depth** 42, **speed** +10, **HP** 1500, **AC** 120, **exp** 2000/level, **rarity** 2, **sleep** 10, **hearing** 30, **smell** 20
- **Melee**: bite 2d12, bite 2d12, bite 2d8, bite 2d8
- **Spells** (spells 1/2, spell power 42):
  - `S_KIN`: summon similar monsters
- **Defences**: no-conf, no-sleep
- **Flags**: weird-mind (ESP 1 in 10), group-AI, opens doors, bashes doors, animal
- **Other flags**: UNIQUE, FEMALE, FORCE\_SLEEP
- **Escorts / friends**: 100% 2d6 Soldier ant (servant); 60% 2d3 Giant silver ant (servant); 60% 2d3 Giant grey ant (servant); 60% 2d3 Giant blue ant (servant); 60% 2d3 Giant fire ant (servant); 100% 3d3 Giant army ant (bodyguard)
- **Drops**: DROP\_2, DROP\_GOOD, ONLY\_ITEM

> She's upset because you hurt her children.

### Ariel, Queen of Air

- **Depth** 43, **speed** +20, **HP** 2700, **AC** 75, **exp** 4750/level, **rarity** 3, **sleep** 30, **hearing** 12
- **Melee**: hit 4d6, hit 4d6, hit confuse 4d4, hit confuse 4d4
- **Spells** (spells 1/5, spell power 43):
  - `BA_COLD`: produce balls of cold, max 74 damage
  - `BA_ELEC`: produce lightning balls, max 72 damage
  - `BO_ELEC`: produce lightning bolts, max 46 damage
- **Defences**: im-acid, im-elec, im-cold, im-pois, no-fear, no-conf, no-sleep
- **Flags**: cold-blooded, empty-mind (no ESP), erratic 25%, powerful, destroys items, kill-body, bashes doors, nonliving
- **Other flags**: CLEAR\_WEB, UNIQUE, FEMALE, NAME\_COMMA, FORCE\_SLEEP
- **Drops**: none

> A towering air elemental, Ariel, the sorceress, avoids your blows with her extreme speed.

### Quaker, Master of Earth

- **Depth** 43, **speed** +0, **HP** 2800, **AC** 145, **exp** 4500/level, **rarity** 3, **sleep** 50, **hearing** 10
- **Melee**: hit 6d6, hit 6d6, hit 6d6, hit shatter 10d10
- **Spells** (spells 1/6, spell power 43):
  - `BA_ACID`: produce acid balls, max 144 damage
  - `BA_SHAR`: produce shardstorms, max 74 damage
  - `BO_ACID`: produce acid bolts, max 70 damage
- **Defences**: im-elec, im-fire, im-cold, im-pois, hurt-rock, no-fear, no-conf, no-sleep
- **Flags**: cold-blooded, empty-mind (no ESP), pass-wall, powerful, destroys items, kill-body, nonliving
- **Other flags**: CLEAR\_WEB, UNIQUE, MALE, NAME\_COMMA, FORCE\_SLEEP
- **Drops**: none

> A towering stone elemental stands before you. The walls and ceiling are reduced to rubble as Quaker advances.

### Vargo, Tyrant of Fire

- **Depth** 43, **speed** +10, **HP** 2400, **AC** 75, **exp** 4000/level, **rarity** 3, **sleep** 30, **hearing** 12, **light** 2
- **Melee**: hit fire 4d6, hit fire 4d6, hit fire 4d6, hit fire 4d6
- **Spells** (spells 1/4, spell power 43):
  - `BA_FIRE`: produce fire balls, max 160 damage
  - `BO_PLAS`: produce plasma bolts, max 109 damage
- **Defences**: im-acid, im-elec, im-fire, im-pois, hurt-cold, no-fear, no-conf, no-sleep
- **Flags**: empty-mind (no ESP), erratic 25%, powerful, destroys items, kill-body, bashes doors, nonliving
- **Other flags**: CLEAR\_WEB, UNIQUE, NAME\_COMMA, FORCE\_SLEEP
- **Drops**: none

> A towering fire elemental, Vargo burns everything beyond recognition.

### Waldern, King of Water

- **Depth** 43, **speed** +10, **HP** 2500, **AC** 120, **exp** 4250/level, **rarity** 3, **sleep** 30, **hearing** 12
- **Melee**: hit 5d5, hit 5d5, hit 5d5, hit 5d5
- **Spells** (spells 1/4, spell power 43):
  - `BA_COLD`: produce balls of cold, max 74 damage
  - `BA_WATE`: produce water balls, max 157 damage
  - `BO_ICE`: produce ice bolts, max 79 damage
  - `BO_WATE`: produce water bolts, max 143 damage
- **Defences**: im-acid, im-fire, im-cold, im-pois, im-water, no-fear, no-conf, no-sleep
- **Flags**: cold-blooded, empty-mind (no ESP), erratic 25%, powerful, destroys items, kill-body, bashes doors, nonliving
- **Other flags**: CLEAR\_WEB, UNIQUE, NAME\_COMMA, FORCE\_SLEEP
- **Drops**: none

> A towering water elemental, Waldern is master of all things liquid. Wave after wave drowns your frail body.

### Eöl, the Dark Elf

- **Depth** 49, **speed** +20, **HP** 2400, **AC** 150, **exp** 25000/level, **rarity** 2, **sleep** 40, **hearing** 20
- **Melee**: hit 3d8, hit 3d8, hit 3d8
- **Spells** (spells 1/2, spell power 49):
  - `BLIND`: blind (save)
  - `BLINK`: blink-self
  - `WOUND`: cause serious wounds, max 160 damage (save)
  - `CONF`: confuse (save)
  - `DARKNESS`: create darkness
  - `HEAL`: heal-self
  - `TELE_TO`: teleport to
  - `BA_ACID`: produce acid balls, max 162 damage
  - `BA_COLD`: produce balls of cold, max 83 damage
  - `BA_ELEC`: produce lightning balls, max 81 damage
  - `BA_FIRE`: produce fire balls, max 181 damage
  - `BA_NETH`: produce nether balls, max 296 damage
  - `BO_MANA`: produce mana bolts, max 172 damage
  - `S_DEMON`: summon a demon
  - `S_DRAGON`: summon a dragon
  - `S_MONSTERS`: summon monsters
  - `S_UNDEAD`: summon an undead
- **Defences**: im-fire, im-cold, im-pois, hurt-light, no-conf, no-sleep, no-hold
- **Flags**: smart, spirit, opens doors, bashes doors, evil
- **Other flags**: CLEAR\_WEB, UNIQUE, MALE, NAME\_COMMA, FORCE\_SLEEP
- **Drops**: DROP\_4, DROP\_GOOD, ONLY\_ITEM

> A lord of the Teleri, Eöl is a mighty metalsmith, the only one to ever forge weapons of meteorite iron. The mortal enemy of the Eldar, he is currently fleeing their wrath, following the murder of Princess Aredhel of Gondolin.

### The Balrog of Moria

- **Depth** 50, **speed** +20, **HP** 3000, **AC** 150, **exp** 30000/level, **rarity** 3, **sleep** 20, **hearing** 20, **light** 2
- **Melee**: hit fire 6d12, hit fire 6d12, crush 5d12, touch drain\_charges
- **Spells** (innate 1/15, spells 1/5, spell power 50):
  - `CONF`: confuse (save)
  - `SCARE`: terrify (save)
  - `BR_FIRE`: fire, max 1000 damage
  - `S_DEMON`: summon a demon
  - `S_UNDEAD`: summon an undead
- **Defences**: im-fire, no-conf, no-sleep, no-hold
- **Flags**: spirit, powerful, move-body, opens doors, bashes doors, evil, demon
- **Other flags**: CLEAR\_WEB, FORCE\_SLEEP, UNIQUE, MALE
- **Escorts / friends**: 100% 4d4 cave orc (servant); 100% 3d4 cave troll (servant)
- **Drops**: DROP\_4, DROP\_GOOD, ONLY\_ITEM

> A huge Balrog surrounded by raging pillars of fire, this is indeed a terrible opponent. Wielding a great whip of fire and a blazing sword, his fury blisters your skin and melts your flesh!

### Harowen the Black Hand

- **Depth** 51, **speed** +30, **HP** 2500, **AC** 108, **exp** 20000/level, **rarity** 3, **sleep** 0, **hearing** 40
- **Melee**: touch eat\_gold 5d5, touch eat\_item 5d5, hit blind 10d5, hit poison 8d5
- **Spells** (spells 1/4, spell power 51):
  - `TELE_TO`: teleport to
  - `TELE_SELF_TO`: teleport toward
  - `BLINK`: blink-self
  - `TRAPS`: create traps
  - `DARKNESS`: create darkness
  - `BLIND`: blind (save)
  - `HOLD`: paralyze (save)
- **Defences**: im-pois, no-conf, no-sleep, no-hold
- **Flags**: spirit, takes items, opens doors, bashes doors
- **Other flags**: CLEAR\_WEB, UNIQUE, MALE, FORCE\_SLEEP
- **Drops**: DROP\_4, DROP\_GOOD, ONLY\_ITEM

> He is a master of disguise, an expert of stealth, a genius at traps, and moves with blinding speed. Check your pockets!

### Itangast the Fire Drake

- **Depth** 52, **speed** +10, **HP** 2200, **AC** 144, **exp** 20000/level, **rarity** 2, **sleep** 70, **hearing** 30, **light** 2
- **Melee**: claw 4d11, claw 4d11, bite fire 4d15, bite fire 4d15
- **Spells** (innate 1/8, spells 1/8, spell power 52):
  - `WOUND`: cause critical wounds, max 170 damage (save)
  - `CONF`: confuse (save)
  - `BR_FIRE`: fire, max 733 damage
- **Defences**: im-fire, no-conf, no-sleep, no-hold
- **Flags**: smart, spirit, powerful, move-body, bashes doors, evil, dragon
- **Other flags**: CLEAR\_WEB, FORCE\_SLEEP, UNIQUE, MALE
- **Drops**: DROP\_4, DROP\_GOOD, ONLY\_ITEM

> A mighty ancient dragon, Itangast's form scorches your flesh. Wisps of smoke curl up from his nostrils as he regards you with disdain.

### Scatha the Worm

- **Depth** 53, **speed** +10, **HP** 2500, **AC** 156, **exp** 18000/level, **rarity** 2, **sleep** 70, **hearing** 30
- **Melee**: claw 4d11, claw 4d11, claw cold 4d15, bite cold 4d15
- **Spells** (innate 1/8, spells 1/8, spell power 53):
  - `WOUND`: cause critical wounds, max 170 damage (save)
  - `CONF`: confuse (save)
  - `BR_COLD`: frost, max 833 damage
- **Defences**: im-cold, no-conf, no-sleep, no-hold
- **Flags**: smart, spirit, powerful, move-body, bashes doors, evil, dragon
- **Other flags**: CLEAR\_WEB, FORCE\_SLEEP, UNIQUE, MALE
- **Drops**: DROP\_4, DROP\_GOOD, ONLY\_ITEM

> An ancient and wise Dragon. Scatha has grown clever over the long years. His scales are covered with frost, and his breath sends a shower of ice into the air.

### The Phoenix

- **Depth** 54, **speed** +10, **HP** 3600, **AC** 156, **exp** 40000/level, **rarity** 3, **sleep** 0, **hearing** 60, **light** 2
- **Melee**: bite fire 12d6, bite fire 12d6, hit fire 9d12, hit fire 9d12
- **Spells** (innate 1/6, spells 1/5, spell power 54):
  - `BA_FIRE`: produce fire balls, max 199 damage
  - `BO_FIRE`: produce fire bolts, max 90 damage
  - `BO_PLAS`: produce plasma bolts, max 120 damage
  - `BR_FIRE`: fire, max 1200 damage
  - `BR_LIGHT`: light, max 500 damage
  - `BR_PLAS`: plasma, max 150 damage
  - `S_KIN`: summon similar monsters
- **Defences**: im-acid, im-elec, im-fire, im-pois, im-plasma, no-conf, no-sleep, no-hold
- **Flags**: opens doors, bashes doors, animal
- **Other flags**: CLEAR\_WEB, UNIQUE, FORCE\_SLEEP
- **Drops**: DROP\_2, DROP\_GOOD, ONLY\_ITEM

> A massive glowing eagle bathed in flames. The searing heat chars your skin and melts your armour.

### Ar-Pharazôn the Golden

- **Depth** 55, **speed** +20, **HP** 4000, **AC** 54, **exp** 32500/level, **rarity** 1, **sleep** 255, **hearing** 30, **light** 2
- **Melee**: hit 8d8, hit 8d8, hit 8d8, hit 8d8
- **Spells** (spells 1/6, spell power 55):
  - `HEAL`: heal-self
  - `TELE_AWAY`: teleport away
  - `S_KIN`: summon similar monsters
  - `S_MONSTERS`: summon monsters
- **Defences**: im-elec, im-fire, im-pois
- **Flags**: smart, spirit, takes items, move-body, opens doors, bashes doors, evil
- **Other flags**: CLEAR\_WEB, UNIQUE, MALE, ATTR\_FLICKER, FORCE\_SLEEP
- **Drops**: DROP\_3, DROP\_GOOD, DROP\_GREAT, ONLY\_ITEM

> Last and proudest king of ancient Númenor. Corrupted by power and avarice, he fell victim to Sauron's wiles, tried to fight the Valar themselves, and condemned Númenor to oblivion.

### Shelob, Spider of Darkness

- **Depth** 55, **speed** +10, **HP** 3500, **AC** 180, **exp** 27000/level, **rarity** 3, **sleep** 20, **hearing** 30, **light** -3
- **Melee**: claw poison 5d6, claw poison 5d6, bite paralyze 5d10, sting lose\_str 5d4
- **Spells** (innate 1/20, spells 1/4, spell power 55):
  - `BLIND`: blind (save)
  - `WOUND`: cause critical wounds, max 180 damage (save)
  - `CONF`: confuse (save)
  - `HEAL`: heal-self
  - `SCARE`: terrify (save)
  - `SLOW`: slow (save)
  - `WEAVE`: weave webs
  - `BR_DARK`: darkness, max 500 damage
  - `BR_POIS`: poison, max 800 damage
  - `S_KIN`: summon similar monsters
- **Defences**: im-acid, im-pois, hurt-light, no-conf, no-sleep, no-hold
- **Flags**: smart, spirit, bashes doors, evil, animal
- **Other flags**: PASS\_WEB, UNIQUE, FEMALE, NAME\_COMMA, FORCE\_SLEEP
- **Escorts / friends**: 80% 4d4 uruk; 100% 3d7 mirkwood spider
- **Drops**: DROP\_2, DROP\_GOOD, ONLY\_ITEM

> Shelob is an enormous bloated spider, the last daughter of Ungoliant the Unlight. Her poison is legendary, as is her ego. She normally guards the pass through Cirith Ungol, but occasionally goes out foraging for food to feed her voracious appetite.

### Baphomet the Minotaur Lord

- **Depth** 58, **speed** +20, **HP** 4700, **AC** 144, **exp** 24000/level, **rarity** 4, **sleep** 30, **hearing** 30, **smell** 20
- **Melee**: butt 12d13, butt 12d13, hit 10d10, hit 10d10
- **Spells** (innate 1/10, spells 1/8, spell power 58):
  - `ARROW`: fire seeker arrows, max 48 damage
  - `SLOW`: slow (save)
  - `BA_ELEC`: produce lightning balls, max 95 damage
  - `BO_MANA`: produce mana bolts, max 195 damage
  - `BO_PLAS`: produce plasma bolts, max 124 damage
  - `BR_WALL`: force, max 200 damage
- **Defences**: im-fire, no-stun, no-conf, no-sleep, no-hold
- **Flags**: bashes doors, evil
- **Other flags**: CLEAR\_WEB, UNIQUE, MALE, FORCE\_SLEEP
- **Drops**: DROP\_4, DROP\_GOOD, ONLY\_ITEM

> A fearsome bull-headed monster, Baphomet swings a mighty axe as he curses all that defy him.

### Fundin Bluecloak

- **Depth** 59, **speed** +20, **HP** 5000, **AC** 234, **exp** 20000/level, **rarity** 2, **sleep** 10, **hearing** 25
- **Melee**: hit 8d8, hit 8d8, hit 8d8, hit 8d8
- **Spells** (spells 1/4, spell power 59):
  - `BLIND`: blind (save)
  - `BRAIN_SMASH`: cause brain smashing, max 180 damage (save)
  - `WOUND`: cause critical wounds, max 190 damage (save)
  - `CONF`: confuse (save)
  - `HEAL`: heal-self
  - `SCARE`: terrify (save)
  - `S_ANIMAL`: summon animals
  - `S_MONSTERS`: summon monsters
- **Defences**: im-acid, im-elec, im-fire, im-cold, im-pois, no-conf, no-sleep, no-hold
- **Flags**: spirit, opens doors, bashes doors
- **Other flags**: CLEAR\_WEB, UNIQUE, MALE, FORCE\_SLEEP
- **Drops**: DROP\_4, DROP\_GOOD, ONLY\_ITEM

> He is one of the greatest dwarven priests to walk the earth. Fundin has earned a high position in the church, and his skill with both weapon and spell only justifies his position further. His combination of both dwarven strength and priestly wisdom are a true match for any adventurer.

### Gilim, the Giant of Eruman

- **Depth** 60, **speed** +10, **HP** 4000, **AC** 96, **exp** 6000/level, **rarity** 3, **sleep** 15, **hearing** 30
- **Melee**: hit cold 9d10, hit cold 9d10, hit 10d9, hit confuse 10d9
- **Spells** (innate 1/6, spells 1/3, spell power 60):
  - `BOULDER`: throw boulders, max 108 damage
  - `BO_ICE`: produce ice bolts, max 96 damage
  - `STORM`: create storms, max 370 damage
  - `S_MONSTERS`: summon monsters
- **Defences**: im-cold
- **Flags**: smart, spirit, takes items, opens doors, bashes doors, evil, giant
- **Other flags**: CLEAR\_WEB, UNIQUE, MALE, NAME\_COMMA, ATTR\_FLICKER, FORCE\_SLEEP
- **Drops**: DROP\_4, DROP\_GOOD, ONLY\_ITEM

> Known as the giant of winter and the northern reaches, he is tall even among giants, and his name is cursed in many lands.

### Saruman of Many Colours

- **Depth** 60, **speed** +10, **HP** 5000, **AC** 120, **exp** 35000/level, **rarity** 1, **sleep** 0, **hearing** 100, **light** 2
- **Melee**: hit disenchant 6d8, hit disenchant 6d8, hit 5d5, hit 5d5
- **Spells** (spells 1/2, spell power 60):
  - `BLIND`: blind (save)
  - `WOUND`: cause critical wounds, max 200 damage (save)
  - `FORGET`: cause amnesia (save)
  - `HASTE`: haste-self
  - `HEAL`: heal-self
  - `MIND_BLAST`: cause mind blasting, max 64 damage (save)
  - `SCARE`: terrify (save)
  - `TELE_AWAY`: teleport away
  - `TPORT`: teleport-self
  - `TRAPS`: create traps
  - `BA_ACID`: produce acid balls, max 195 damage
  - `BA_COLD`: produce balls of cold, max 100 damage
  - `BA_FIRE`: produce fire balls, max 220 damage
  - `BA_WATE`: produce water balls, max 200 damage
  - `BO_ICE`: produce ice bolts, max 96 damage
  - `S_ANIMAL`: summon animals
  - `S_DEMON`: summon a demon
  - `S_HI_DRAGON`: summon ancient dragons
  - `S_MONSTERS`: summon monsters
  - `S_UNDEAD`: summon an undead
- **Defences**: im-elec, im-fire, im-cold, im-pois, no-conf, no-sleep, no-hold
- **Flags**: smart, spirit, powerful, takes items, opens doors, bashes doors, evil
- **Other flags**: CLEAR\_WEB, FORCE\_SLEEP, UNIQUE, MALE, ATTR\_MULTI
- **Escorts / friends**: 100% 1d1 Wormtongue, Agent of Saruman (servant)
- **Drops**: DROP\_4, DROP\_GOOD, DROP\_GREAT, ONLY\_ITEM

> Originally known as the White, Saruman fell prey to Sauron's wiles. He seeks to emulate him and breeds orcs and trolls to fight for him. He searches forever for the One Ring, to become a mighty Sorcerer-King of the world.

### Smaug the Golden

- **Depth** 62, **speed** +10, **HP** 4200, **AC** 195, **exp** 30000/level, **rarity** 2, **sleep** 120, **hearing** 30, **light** 2
- **Melee**: claw 6d12, claw 6d12, bite fire 7d14, bite fire 7d14
- **Spells** (innate 1/5, spells 1/8, spell power 62):
  - `WOUND`: cause critical wounds, max 200 damage (save)
  - `CONF`: confuse (save)
  - `BR_FIRE`: fire, max 1400 damage
- **Defences**: im-fire, no-conf, no-sleep, no-hold
- **Flags**: smart, spirit, powerful, move-body, bashes doors, evil, dragon
- **Other flags**: CLEAR\_WEB, FORCE\_SLEEP, UNIQUE, MALE
- **Drops**: DROP\_4, DROP\_GOOD, ONLY\_ITEM

> Smaug is one of the Urulóki that still survive, a fire-drake of immense cunning and intelligence. His speed through air is matched by few other dragons and his dragonfire is what legends are made of; he is believed to be the greatest dragon still surviving into the Third Age.

### The Mouth of Sauron

- **Depth** 65, **speed** +20, **HP** 7000, **AC** 120, **exp** 38000/level, **rarity** 3, **sleep** 10, **hearing** 60
- **Melee**: hit disenchant 6d8, hit disenchant 6d8, hit drain\_charges 5d5, hit drain\_charges 5d5
- **Spells** (spells 1/2, spell power 65):
  - `WOUND`: cause critical wounds, max 210 damage (save)
  - `HOLD`: paralyze (save)
  - `TELE_TO`: teleport to
  - `TRAPS`: create traps
  - `BA_DARK`: produce balls of darkness, max 360 damage
  - `BA_FIRE`: produce fire balls, max 237 damage
  - `BA_MANA`: invoke mana storms, max 425 damage
  - `BA_NETH`: produce nether balls, max 360 damage
  - `BA_WATE`: produce water balls, max 212 damage
  - `BO_PLAS`: produce plasma bolts, max 131 damage
  - `S_HI_DEMON`: summon greater demons
  - `S_HI_UNDEAD`: summon greater undead
- **Defences**: im-elec, im-fire, im-cold, no-conf, no-sleep, no-hold
- **Flags**: INVISIBLE, smart, spirit, opens doors, bashes doors, evil
- **Other flags**: CLEAR\_WEB, UNIQUE, MALE, FORCE\_SLEEP
- **Drops**: DROP\_4, DROP\_GOOD, ONLY\_ITEM

> The Mouth of Sauron is a mighty spell caster. So old that even he cannot remember his own name, his power and evil are undeniable. He believes unshakably that he is unbeatable and laughs as he weaves his awesome spells.

### Tevildo, Prince of Cats

- **Depth** 66, **speed** +20, **HP** 4800, **AC** 240, **exp** 30000/level, **rarity** 3, **sleep** 0, **hearing** 100, **smell** 30
- **Melee**: claw confuse 12d12, claw lose\_dex 2d12, claw blind 10d5, bite paralyze 15d1
- **Spells** (spells 1/3, spell power 66):
  - `TELE_TO`: teleport to
  - `S_KIN`: summon similar monsters
- **Defences**: im-fire, im-cold, no-conf, no-sleep, no-hold
- **Flags**: INVISIBLE, spirit, opens doors, bashes doors, animal
- **Other flags**: CLEAR\_WEB, UNIQUE, MALE, NAME\_COMMA, FORCE\_SLEEP
- **Escorts / friends**: 100% 2d4 Tiger (servant); 100% 2d4 Sabre-tooth tiger (servant); 100% 2d4 Displacer beast (servant)
- **Drops**: DROP\_4, DROP\_GOOD, ONLY\_ITEM

> Master of all things feline, Tevildo serves his master Morgoth well. He delights in small torments and cunning stratagems, and laughs merrily as he bats you about.

### Thuringwethil, the Vampire Messenger

- **Depth** 67, **speed** +20, **HP** 5000, **AC** 174, **exp** 23000/level, **rarity** 4, **sleep** 10, **hearing** 20, **smell** 20
- **Melee**: bite exp\_80 6d6, bite exp\_80 6d6, hit confuse 6d6, hit confuse 6d6
- **Spells** (spells 1/3, spell power 67):
  - `BLIND`: blind (save)
  - `BRAIN_SMASH`: cause brain smashing, max 180 damage (save)
  - `WOUND`: cause critical wounds, max 220 damage (save)
  - `DRAIN_MANA`: drain mana
  - `BA_NETH`: produce nether balls, max 368 damage
  - `S_HI_UNDEAD`: summon greater undead
  - `S_KIN`: summon similar monsters
  - `HOLD`: paralyze (save)
  - `SCARE`: terrify (save)
- **Defences**: im-cold, im-pois, hurt-light, no-stun, no-conf, no-sleep
- **Flags**: cold-blooded, regenerates, smart, spirit, opens doors, bashes doors, evil, undead
- **Other flags**: CLEAR\_WEB, FORCE\_SLEEP, UNIQUE, FEMALE, NAME\_COMMA
- **Drops**: DROP\_4, DROP\_GOOD, ONLY\_ITEM

> Chief messenger between Sauron and Morgoth, she is surely the most deadly of her vampire race. At first she is charming to meet, but her wings and eyes give away her true form.

### Tselakus, the Dreadlord

- **Depth** 68, **speed** +20, **HP** 6500, **AC** 180, **exp** 35000/level, **rarity** 2, **sleep** 10, **hearing** 20
- **Melee**: hit 10d10, hit 10d10, hit lose\_str 4d6, hit lose\_str 4d6
- **Spells** (spells 1/3, spell power 68):
  - `BLIND`: blind (save)
  - `CONF`: confuse (save)
  - `HOLD`: paralyze (save)
  - `BA_DARK`: produce balls of darkness, max 372 damage
  - `BA_NETH`: produce nether balls, max 372 damage
  - `S_HI_UNDEAD`: summon greater undead
  - `S_KIN`: summon similar monsters
- **Defences**: im-cold, im-pois, no-stun, no-conf, no-sleep, no-hold
- **Flags**: INVISIBLE, cold-blooded, pass-wall, spirit, evil, undead
- **Other flags**: UNIQUE, MALE, NAME\_COMMA, FORCE\_SLEEP
- **Drops**: DROP\_4, DROP\_GOOD, ONLY\_ITEM

> This huge affront to existence twists and tears at the fabric of space. A master of mighty magic, Tselakus hungers for your tender flesh. Darkness itself recoils from the touch of Tselakus as he leaves a trail of death and destruction. Tselakus is a being of sneering contempt, laughing at your pitiful efforts to defy him. Mighty claws rend reality as he annihilates all in his path to your soul!

### Nan, the Giant

- **Depth** 69, **speed** +10, **HP** 3344, **AC** 150, **exp** 23500/level, **rarity** 3, **sleep** 15, **hearing** 30
- **Melee**: hit 12d12, hit 12d12, hit confuse 12d12, hit confuse 12d12
- **Spells** (innate 1/6, spells 1/4, spell power 85):
  - `BOULDER`: throw boulders, max 156 damage
  - `BO_PLAS`: produce plasma bolts, max 151 damage
  - `BA_FIRE`: produce firestorms, max 307 damage
  - `S_MONSTERS`: summon monsters
- **Defences**: im-fire
- **Flags**: smart, spirit, takes items, bashes doors, evil, giant
- **Other flags**: CLEAR\_WEB, UNIQUE, MALE, NAME\_COMMA, ATTR\_FLICKER, FORCE\_SLEEP
- **Drops**: DROP\_4, DROP\_GOOD, ONLY\_ITEM

> Known as the giant of summer and the south, he towers above you like a great elm. In his right hand, he holds the longest sword you have ever beheld.

### Ossë, Herald of Ulmo

- **Depth** 72, **speed** +20, **HP** 6000, **AC** 204, **exp** 30000/level, **rarity** 3, **sleep** 10, **hearing** 40
- **Melee**: hit disenchant 10d5, hit disenchant 10d5, hit 10d10, hit 10d10
- **Spells** (innate 1/4, spells 1/2, spell power 72):
  - `BLIND`: blind (save)
  - `TELE_TO`: teleport to
  - `WHIP`: lash you if nearby, max 175 damage
  - `BA_WATE`: produce water balls, max 230 damage
  - `STORM`: create storms, max 430 damage
  - `BO_MANA`: produce mana bolts, max 230 damage
  - `BO_WATE`: produce water bolts, max 172 damage
  - `BR_DISE`: disenchantment, max 500 damage
  - `S_AINU`: summon an ainu
- **Defences**: im-acid, im-elec, im-fire, im-cold, im-pois, im-water, im-disen, no-conf, no-sleep, no-hold
- **Flags**: smart, spirit, powerful, takes items, move-body, opens doors, bashes doors
- **Other flags**: CLEAR\_WEB, FORCE\_SLEEP, UNIQUE, MALE, NAME\_COMMA
- **Drops**: DROP\_4, DROP\_GOOD, ONLY\_ITEM

> Encrusted with barnacles, slimy and dripping, the Maia of the untamed sea has dragged himself down into Angband to send you to a watery grave. Ossë is the most powerful and heartless of Ulmo's servants and embodies the untamed power of the ocean. Terror grows in your heart with each squelching step of his approach.

### Omarax, the Eye Tyrant

- **Depth** 73, **speed** +20, **HP** 6500, **AC** 96, **exp** 25000/level, **rarity** 3, **sleep** 10, **hearing** 30
- **Melee**: gaze disenchant 6d6, gaze drain\_charges 6d6, gaze lose\_int 6d6, bite exp\_80 8d8
- **Spells** (spells 1/2, spell power 73):
  - `BLIND`: blind (save)
  - `BRAIN_SMASH`: cause brain smashing, max 180 damage (save)
  - `CONF`: confuse (save)
  - `DRAIN_MANA`: drain mana
  - `FORGET`: cause amnesia (save)
  - `SCARE`: terrify (save)
  - `BA_ACID`: produce acid balls, max 234 damage
  - `BA_COLD`: produce balls of cold, max 119 damage
  - `BA_DARK`: invoke darkness storms, max 392 damage
  - `BA_FIRE`: produce fire balls, max 265 damage
  - `BA_NETH`: produce nether balls, max 392 damage
  - `BO_MANA`: produce mana bolts, max 232 damage
  - `S_KIN`: summon similar monsters
- **Defences**: im-acid, im-fire, im-cold, im-pois, no-conf, no-sleep, no-hold
- **Flags**: smart, bashes doors, evil
- **Other flags**: CLEAR\_WEB, UNIQUE, MALE, NAME\_COMMA, FORCE\_SLEEP
- **Drops**: none

> A beholder of great size and age, floating in the air. His gaze seems to shred your soul and his spells crush your will. He is ancient, his history steeped in forgotten evils, his atrocities numerous and sickening.

### Glaurung, Father of the Dragons

- **Depth** 75, **speed** +20, **HP** 7500, **AC** 210, **exp** 55000/level, **rarity** 3, **sleep** 20, **hearing** 30, **light** 2
- **Melee**: claw 6d12, claw 6d12, bite poison 8d14, bite fire 8d14
- **Spells** (innate 1/5, spells 1/3, spell power 75):
  - `BRAIN_SMASH`: cause brain smashing, max 180 damage (save)
  - `FORGET`: cause amnesia (save)
  - `SCARE`: terrify (save)
  - `BR_FIRE`: fire, max 1600 damage
  - `BR_POIS`: poison, max 800 damage
  - `S_HI_DRAGON`: summon ancient dragons
- **Defences**: im-fire, im-pois, no-conf, no-sleep, no-hold
- **Flags**: smart, spirit, powerful, move-body, bashes doors, evil, dragon
- **Other flags**: CLEAR\_WEB, FORCE\_SLEEP, UNIQUE, MALE, NAME\_COMMA
- **Drops**: DROP\_4, DROP\_GOOD, DROP\_GREAT, ONLY\_ITEM

> Glaurung is the father of all dragons, and was for a long time the most powerful. Though this is no longer so, he still has full command over his brood and can command them to appear whenever he so wishes. He is the definition of dragonfire.

### Ungoliant, the Unlight

- **Depth** 75, **speed** +20, **HP** 13000, **AC** 240, **exp** 35000/level, **rarity** 1, **sleep** 10, **hearing** 40, **light** -4
- **Melee**: claw poison 8d6, claw poison 8d6, bite paralyze 8d10, sting lose\_str 8d4
- **Spells** (innate 1/11, spells 1/4, spell power 75):
  - `BLIND`: blind (save)
  - `CONF`: confuse (save)
  - `HEAL`: heal-self
  - `SCARE`: terrify (save)
  - `WEAVE`: weave webs
  - `BA_DARK`: invoke darkness storms, max 400 damage
  - `BR_DARK`: darkness, max 500 damage
  - `BR_POIS`: poison, max 800 damage
  - `S_KIN`: summon similar monsters
- **Defences**: im-acid, im-pois, hurt-light, no-conf, no-sleep, no-hold
- **Flags**: smart, spirit, bashes doors, evil, animal
- **Other flags**: PASS\_WEB, UNIQUE, FEMALE, NAME\_COMMA, FORCE\_SLEEP
- **Drops**: DROP\_4, DROP\_GOOD, ONLY\_ITEM

> This enormous, hideous spirit of void is in the form of a spider of immense proportions. She is surrounded by a cloud of Unlight as she sucks in all living light into her bloated body, and breathes out the blackest of darkness. She is always ravenously hungry and would even eat herself to avoid starvation.

### Makar, the Warrior

- **Depth** 76, **speed** +10, **HP** 6000, **AC** 192, **exp** 37000/level, **rarity** 5, **sleep** 15, **hearing** 30
- **Melee**: hit shatter 13d13, hit confuse 13d13, hit shatter 13d13, hit confuse 13d13
- **Spells**: none
- **Defences**: im-acid, im-elec, im-fire, im-cold, im-pois, no-conf, no-sleep, no-hold
- **Flags**: kill-wall, smart, spirit, powerful, takes items, kill-body, opens doors, bashes doors
- **Other flags**: CLEAR\_WEB, FORCE\_SLEEP, UNIQUE, MALE, NAME\_COMMA
- **Drops**: DROP\_4, DROP\_GOOD, ONLY\_ITEM

> A battle-hungry spirit from amongst the Ainur, who built a grim hall in the early days of Arda to host endless battles. When not at home, he and his sister Meássë hunt and seek out the turmoils caused by Morgoth.

### Feagwath, the Undead Sorcerer

- **Depth** 77, **speed** +20, **HP** 5000, **AC** 127, **exp** 30000/level, **rarity** 2, **sleep** 10, **hearing** 20
- **Melee**: touch exp\_80 6d12, touch drain\_charges 6d12, touch lose\_dex 6d12, touch lose\_dex 6d12
- **Spells** (spells 1/2, spell power 77):
  - `BLIND`: blind (save)
  - `BLINK`: blink-self
  - `WOUND`: cause critical wounds, max 250 damage (save)
  - `HOLD`: paralyze (save)
  - `SCARE`: terrify (save)
  - `BRAIN_SMASH`: cause brain smashing, max 180 damage (save)
  - `TELE_TO`: teleport to
  - `TRAPS`: create traps
  - `BA_MANA`: invoke mana storms, max 485 damage
  - `BA_NETH`: produce nether balls, max 408 damage
  - `BO_MANA`: produce mana bolts, max 242 damage
  - `S_KIN`: summon similar monsters
  - `S_MONSTERS`: summon monsters
  - `S_UNDEAD`: summon an undead
- **Defences**: im-cold, im-pois, no-stun, no-conf, no-sleep, no-hold
- **Flags**: cold-blooded, smart, spirit, opens doors, bashes doors, evil, undead
- **Other flags**: CLEAR\_WEB, FORCE\_SLEEP, UNIQUE, MALE, NAME\_COMMA
- **Escorts / friends**: 100% 2d4 Dread (servant); 40% 1d2 Dreadmaster (servant); 10% 1d2 Dreadlord (servant); 100% 1d4 Lich (servant); 40% 1d2 Master lich (servant); 10% 1d2 Archlich (servant)
- **Drops**: DROP\_4, DROP\_GOOD, DROP\_GREAT, ONLY\_ITEM

> A stench of corruption and decay surrounds this sorcerer, who has clearly risen from the dead to continue his foul plots and schemes.

### Uvatha the Horseman

- **Depth** 77, **speed** +10, **HP** 4200, **AC** 72, **exp** 33000/level, **rarity** 3, **sleep** 10, **hearing** 90, **smell** 50
- **Melee**: hit 6d9, hit black\_breath 6d9, hit exp\_80 4d6, hit exp\_80 4d6
- **Spells** (spells 1/3, spell power 77):
  - `BLIND`: blind (save)
  - `WOUND`: cause critical wounds, max 250 damage (save)
  - `FORGET`: cause amnesia (save)
  - `HOLD`: paralyze (save)
  - `SCARE`: terrify (save)
  - `BO_COLD`: produce frost bolts, max 73 damage
  - `BO_NETH`: produce nether bolts, max 170 damage
  - `S_MONSTER`: summon a monster
  - `S_WRAITH`: summon ringwraiths
- **Defences**: im-cold, im-pois, hurt-light, no-stun, no-conf, no-sleep, no-hold
- **Flags**: cold-blooded, spirit, move-body, opens doors, bashes doors, evil, undead
- **Other flags**: CLEAR\_WEB, UNIQUE, MALE
- **Drops**: DROP\_2, DROP\_GOOD, ONLY\_ITEM

> A tall black-cloaked Ringwraith, he is a master of horsemanship. He longs to taste your blood.

### Adunaphel the Quiet

- **Depth** 78, **speed** +10, **HP** 4300, **AC** 72, **exp** 34000/level, **rarity** 3, **sleep** 10, **hearing** 90, **smell** 50
- **Melee**: hit 6d9, hit black\_breath 6d9, touch exp\_80 5d6, touch exp\_80 5d6
- **Spells** (spells 1/3, spell power 78):
  - `BLIND`: blind (save)
  - `WOUND`: cause critical wounds, max 260 damage (save)
  - `FORGET`: cause amnesia (save)
  - `HOLD`: paralyze (save)
  - `SCARE`: terrify (save)
  - `BO_ACID`: produce acid bolts, max 82 damage
  - `BO_COLD`: produce frost bolts, max 74 damage
  - `BO_FIRE`: produce fire bolts, max 98 damage
  - `BO_NETH`: produce nether bolts, max 172 damage
  - `S_MONSTER`: summon a monster
  - `S_WRAITH`: summon ringwraiths
- **Defences**: im-cold, im-pois, hurt-light, no-stun, no-conf, no-sleep, no-hold
- **Flags**: cold-blooded, spirit, move-body, opens doors, bashes doors, evil, undead
- **Other flags**: CLEAR\_WEB, UNIQUE, FEMALE, FORCE\_SLEEP
- **Drops**: DROP\_4, DROP\_GOOD, ONLY\_ITEM

> A sorceress in life, Adunaphel quickly fell under Sauron's sway and the power of the rings.

### Akhorahil the Blind

- **Depth** 79, **speed** +10, **HP** 4400, **AC** 84, **exp** 35000/level, **rarity** 3, **sleep** 10, **hearing** 90, **smell** 50
- **Melee**: hit 7d9, hit black\_breath 7d9, gaze exp\_80 6d6, wail terrify 6d6
- **Spells** (spells 1/3, spell power 79):
  - `BLIND`: blind (save)
  - `WOUND`: cause critical wounds, max 260 damage (save)
  - `DARKNESS`: create darkness
  - `HOLD`: paralyze (save)
  - `SCARE`: terrify (save)
  - `BO_COLD`: produce frost bolts, max 74 damage
  - `BO_FIRE`: produce fire bolts, max 98 damage
  - `BO_NETH`: produce nether bolts, max 173 damage
  - `S_MONSTERS`: summon monsters
  - `S_WRAITH`: summon ringwraiths
- **Defences**: im-cold, im-pois, hurt-light, no-stun, no-conf, no-sleep, no-hold
- **Flags**: cold-blooded, spirit, move-body, opens doors, bashes doors, evil, undead
- **Other flags**: CLEAR\_WEB, UNIQUE, MALE, FORCE\_SLEEP
- **Drops**: DROP\_4, DROP\_GOOD, ONLY\_ITEM

> A mighty sorcerer King, Akhorahil was blind in life. With powerful enchantments, he created jewelled eyes that enabled him to see better than any ordinary man ever could.

### Qlzqqlzuup, the Emperor Quylthulg

- **Depth** 79, **speed** +20, **HP** 5000, **AC** 1, **exp** 20000/level, **rarity** 3, **sleep** 0, **hearing** 30
- **Melee**: -
- **Spells** (spells 1/1, spell power 79):
  - `S_AINU`: summon an ainu
  - `S_ANIMAL`: summon animals
  - `S_HI_DEMON`: summon greater demons
  - `S_HI_DRAGON`: summon ancient dragons
  - `S_HI_UNDEAD`: summon greater undead
  - `S_HOUND`: summon hounds
  - `S_HYDRA`: summon hydras
  - `S_KIN`: summon similar monsters
  - `S_MONSTERS`: summon monsters
  - `S_SPIDER`: summon spiders
  - `S_UNIQUE`: summon uniques
- **Defences**: no-fear, no-conf, no-sleep, no-hold
- **Flags**: INVISIBLE, empty-mind (no ESP), never-moves, no melee
- **Other flags**: FORCE\_SLEEP, UNIQUE, NAME\_COMMA, ATTR\_MULTI
- **Drops**: DROP\_4, DROP\_GOOD, ONLY\_ITEM

> A gigantic seething mass of flesh, Qlzqqlzuup changes colours in front of your eyes. Pulsating first one colour then the next, it knows only it must bring help to protect itself.

### Ren the Unclean

- **Depth** 80, **speed** +10, **HP** 4500, **AC** 84, **exp** 36000/level, **rarity** 3, **sleep** 10, **hearing** 90, **smell** 50
- **Melee**: hit 7d9, hit black\_breath 7d9, touch exp\_80 6d7, wail terrify 6d7
- **Spells** (spells 1/3, spell power 80):
  - `BLIND`: blind (save)
  - `WOUND`: cause mortal wounds, max 260 damage (save)
  - `HOLD`: paralyze (save)
  - `SCARE`: terrify (save)
  - `BA_FIRE`: produce firestorms, max 290 damage
  - `BO_FIRE`: produce fire bolts, max 98 damage
  - `BO_NETH`: produce nether bolts, max 175 damage
  - `S_MONSTER`: summon a monster
  - `S_WRAITH`: summon ringwraiths
- **Defences**: im-fire, im-cold, im-pois, hurt-light, no-stun, no-conf, no-sleep, no-hold
- **Flags**: INVISIBLE, cold-blooded, spirit, move-body, opens doors, bashes doors, evil, undead
- **Other flags**: CLEAR\_WEB, UNIQUE, MALE, FORCE\_SLEEP
- **Drops**: DROP\_4, DROP\_GOOD, ONLY\_ITEM

> Ren was an insane eastern king who believed himself to be the son of a volcano god. At an early age his sanity was destroyed by a plague that wiped out his family, and he never recovered.

### Wiruin, the Maelstrom

- **Depth** 80, **speed** +30, **HP** 3500, **AC** 10, **exp** 18000/level, **rarity** 2, **sleep** 10, **hearing** 50
- **Melee**: engulf cold 3d6, engulf confuse 3d6, engulf 3d6
- **Spells** (innate 1/3, spells 1/5, spell power 80):
  - `BR_COLD`: frost, max 1166 damage
  - `BR_SOUN`: sound, max 500 damage
  - `S_KIN`: summon similar monsters
- **Defences**: im-acid, im-fire, im-cold, im-pois, no-fear, no-conf, no-sleep, no-hold
- **Flags**: empty-mind (no ESP), never-moves, powerful, nonliving
- **Other flags**: FORCE\_SLEEP, UNIQUE, NAME\_COMMA
- **Drops**: none

> A violent icy whirlpool, brought from the frozen wastes of the Helcaraxë by Morgoth as guard and entertainment. Smaller vortices arise on its edges, and you feel as if you will never be free of its deafening roar.

### Ji Indur Dawndeath

- **Depth** 81, **speed** +10, **HP** 4600, **AC** 84, **exp** 37000/level, **rarity** 3, **sleep** 10, **hearing** 90, **smell** 50
- **Melee**: hit 8d9, hit black\_breath 8d9, touch exp\_40 6d7, touch exp\_40 6d7
- **Spells** (spells 1/3, spell power 81):
  - `BLIND`: blind (save)
  - `WOUND`: cause mortal wounds, max 270 damage (save)
  - `HOLD`: paralyze (save)
  - `SCARE`: terrify (save)
  - `BA_ACID`: produce acid balls, max 258 damage
  - `BA_COLD`: produce blizzards, max 131 damage
  - `BA_ELEC`: produce lightning balls, max 129 damage
  - `BA_FIRE`: produce firestorms, max 293 damage
  - `BA_NETH`: produce nether storms, max 424 damage
  - `S_UNDEAD`: summon an undead
  - `S_WRAITH`: summon ringwraiths
- **Defences**: im-fire, im-cold, im-pois, hurt-light, no-stun, no-conf, no-sleep, no-hold
- **Flags**: INVISIBLE, cold-blooded, spirit, move-body, opens doors, bashes doors, evil, undead
- **Other flags**: CLEAR\_WEB, UNIQUE, MALE, FORCE\_SLEEP
- **Drops**: DROP\_4, DROP\_GOOD, ONLY\_ITEM

> This Ringwraith was a weak-minded sorcerer-king who fell easily under Sauron's power.

### Maeglin, the Traitor of Gondolin

- **Depth** 81, **speed** +20, **HP** 6000, **AC** 144, **exp** 35000/level, **rarity** 2, **sleep** 20, **hearing** 70
- **Melee**: hit 8d8, hit 8d8, hit 8d8, hit 8d8
- **Spells** (spells 1/6, spell power 81):
  - `S_ANIMAL`: summon animals
  - `S_HI_DEMON`: summon greater demons
  - `S_HI_DRAGON`: summon ancient dragons
  - `S_HI_UNDEAD`: summon greater undead
  - `S_MONSTERS`: summon monsters
  - `S_UNIQUE`: summon uniques
- **Defences**: im-fire, im-cold, im-pois, no-conf, no-sleep, no-hold
- **Flags**: kill-wall, spirit, move-body, opens doors, bashes doors, evil
- **Other flags**: CLEAR\_WEB, UNIQUE, MALE, NAME\_COMMA, FORCE\_SLEEP
- **Drops**: DROP\_4, DROP\_GOOD, DROP\_GREAT, ONLY\_ITEM

> The son of Eöl the Dark Elf, Maeglin is every bit as evil as his father and more. His greed for gold led him to betray the Hidden Kingdom of Gondolin to Morgoth's forces. He is a mighty warrior himself, and some of Morgoth's greatest servants answer to his call.

### Dwar, Dog Lord of Waw

- **Depth** 82, **speed** +10, **HP** 4700, **AC** 108, **exp** 38000/level, **rarity** 3, **sleep** 10, **hearing** 90, **smell** 50
- **Melee**: hit 9d9, hit black\_breath 8d9, bite exp\_40 6d7, wail terrify 6d7
- **Spells** (spells 1/3, spell power 82):
  - `BLIND`: blind (save)
  - `WOUND`: cause mortal wounds, max 270 damage (save)
  - `HOLD`: paralyze (save)
  - `SCARE`: terrify (save)
  - `BA_FIRE`: produce firestorms, max 297 damage
  - `BA_NETH`: produce nether storms, max 428 damage
  - `S_HOUND`: summon hounds
  - `S_UNDEAD`: summon an undead
  - `S_WRAITH`: summon ringwraiths
- **Defences**: im-cold, im-pois, hurt-light, no-stun, no-conf, no-sleep, no-hold
- **Flags**: cold-blooded, smart, spirit, move-body, opens doors, bashes doors, evil, undead
- **Other flags**: CLEAR\_WEB, UNIQUE, MALE, NAME\_COMMA, FORCE\_SLEEP
- **Drops**: DROP\_4, DROP\_GOOD, ONLY\_ITEM

> Dwar had a special affinity for dogs in life, and can still command them at will. He howls maniacally as he reaches out to destroy you.

### Pazuzu, Lord of Air

- **Depth** 82, **speed** +30, **HP** 5500, **AC** 150, **exp** 30000/level, **rarity** 2, **sleep** 10, **hearing** 40
- **Melee**: hit elec 12d12, hit elec 12d12, hit elec 12d12, hit elec 12d12
- **Spells** (spells 1/3, spell power 82):
  - `MIND_BLAST`: cause mind blasting, max 64 damage (save)
  - `BA_ELEC`: produce lightning balls, max 131 damage
  - `BO_ELEC`: produce lightning bolts, max 59 damage
  - `BO_MANA`: produce mana bolts, max 255 damage
  - `S_HI_DEMON`: summon greater demons
- **Defences**: im-acid, im-elec, im-fire, im-cold, im-pois, no-conf, no-sleep, no-hold
- **Flags**: INVISIBLE, spirit, opens doors, bashes doors, evil, demon
- **Other flags**: CLEAR\_WEB, FORCE\_SLEEP, UNIQUE, MALE, NAME\_COMMA
- **Escorts / friends**: 100% 2d3 Giant roc (servant)
- **Drops**: DROP\_4, DROP\_GOOD, ONLY\_ITEM

> A winged humanoid demon, Pazuzu grins inhumanely at you as he decides your fate.

### Draugluin, Sire of All Werewolves

- **Depth** 83, **speed** +20, **HP** 7000, **AC** 135, **exp** 40000/level, **rarity** 2, **sleep** 10, **hearing** 80, **smell** 50
- **Melee**: claw 6d8, claw 6d8, bite poison 6d6, bite poison 6d6
- **Spells** (spells 1/3, spell power 83):
  - `SCARE`: terrify (save)
  - `S_HOUND`: summon hounds
  - `S_MONSTERS`: summon monsters
- **Defences**: im-pois
- **Flags**: erratic 25%, takes items, move-body, opens doors, bashes doors, evil, animal
- **Other flags**: CLEAR\_WEB, UNIQUE, MALE, NAME\_COMMA, FORCE\_SLEEP
- **Escorts / friends**: 100% 3d5 Werewolf (servant); 100% 2d4 Wolf Chieftain (servant)
- **Drops**: DROP\_1, DROP\_GOOD, ONLY\_ITEM

> Draugluin provides Sauron with a fearsome personal guard. He is an enormous wolf inhabited by a human spirit. He is chief of all his kind.

### Hoarmurath of Dir

- **Depth** 83, **speed** +10, **HP** 4800, **AC** 120, **exp** 39000/level, **rarity** 3, **sleep** 10, **hearing** 90, **smell** 50
- **Melee**: hit 9d9, hit black\_breath 9d9, touch exp\_80 6d7, wail terrify 6d7
- **Spells** (spells 1/3, spell power 83):
  - `BLIND`: blind (save)
  - `WOUND`: cause mortal wounds, max 270 damage (save)
  - `HOLD`: paralyze (save)
  - `MIND_BLAST`: cause mind blasting, max 64 damage (save)
  - `SCARE`: terrify (save)
  - `BA_COLD`: produce blizzards, max 134 damage
  - `BA_NETH`: produce nether storms, max 432 damage
  - `BE_NETH`: shoot rays of death, max 221 damage
  - `S_UNDEAD`: summon an undead
  - `S_WRAITH`: summon ringwraiths
- **Defences**: im-cold, im-pois, hurt-light, no-stun, no-conf, no-sleep, no-hold
- **Flags**: cold-blooded, smart, spirit, move-body, opens doors, bashes doors, evil, undead
- **Other flags**: CLEAR\_WEB, UNIQUE, MALE, FORCE\_SLEEP
- **Drops**: DROP\_4, DROP\_GOOD, ONLY\_ITEM

> A Ringwraith powerful in fell sorcery, he yearns for the life he has lost for an unlife of everlasting torment.

### Cantoras, the Skeletal Lord

- **Depth** 84, **speed** +30, **HP** 7500, **AC** 180, **exp** 45000/level, **rarity** 2, **sleep** 40, **hearing** 20
- **Melee**: gaze exp\_80 5d5, gaze exp\_80 5d5, touch poison 5d5, touch poison 5d5
- **Spells** (spells 1/2, spell power 110):
  - `BRAIN_SMASH`: cause brain smashing, max 180 damage (save)
  - `WOUND`: cause mortal wounds, max 360 damage (save)
  - `SCARE`: terrify (save)
  - `TELE_TO`: teleport to
  - `BA_NETH`: produce nether storms, max 540 damage
  - `BA_WATE`: produce water balls, max 325 damage
  - `BO_ICE`: produce ice bolts, max 146 damage
  - `BO_MANA`: produce mana bolts, max 325 damage
  - `S_HI_UNDEAD`: summon greater undead
- **Defences**: im-fire, im-cold, im-pois, no-fear, no-stun, no-conf, no-sleep
- **Flags**: cold-blooded, smart, spirit, opens doors, bashes doors, evil, undead
- **Other flags**: CLEAR\_WEB, UNIQUE, MALE, NAME\_COMMA, ATTR\_FLICKER, FORCE\_SLEEP
- **Drops**: DROP\_4, DROP\_GOOD, DROP\_GREAT, ONLY\_ITEM

> A legion of evil undead druj animating the skeleton of a once mighty sorcerer. His power is devastating and his speed unmatched in the underworld. Flee his wrath!

### Khamûl, the Black Easterling

- **Depth** 84, **speed** +10, **HP** 4900, **AC** 120, **exp** 40000/level, **rarity** 3, **sleep** 10, **hearing** 90, **smell** 50
- **Melee**: hit 9d10, hit black\_breath 9d10, touch exp\_40 7d7, touch exp\_40 7d7
- **Spells** (spells 1/2, spell power 84):
  - `BLIND`: blind (save)
  - `WOUND`: cause mortal wounds, max 280 damage (save)
  - `HOLD`: paralyze (save)
  - `SCARE`: terrify (save)
  - `TELE_LEVEL`: teleport level (save)
  - `BA_COLD`: produce blizzards, max 136 damage
  - `BA_FIRE`: produce firestorms, max 304 damage
  - `BA_NETH`: produce nether storms, max 436 damage
  - `BO_MANA`: produce mana bolts, max 260 damage
  - `S_HI_UNDEAD`: summon greater undead
  - `S_WRAITH`: summon ringwraiths
- **Defences**: im-acid, im-fire, im-cold, im-pois, hurt-light, no-stun, no-conf, no-sleep, no-hold
- **Flags**: cold-blooded, smart, spirit, move-body, opens doors, bashes doors, evil, undead
- **Other flags**: CLEAR\_WEB, UNIQUE, MALE, NAME\_COMMA, FORCE\_SLEEP
- **Drops**: DROP\_4, DROP\_GOOD, ONLY\_ITEM

> A warrior-king of the East. Khamûl is a powerful opponent, his skill in combat awesome and his form twisted by evil cunning.

### Ancalagon the Black

- **Depth** 85, **speed** +20, **HP** 10000, **AC** 255, **exp** 45000/level, **rarity** 3, **sleep** 20, **hearing** 40
- **Melee**: claw 8d12, claw 8d12, bite 10d14, bite 10d14
- **Spells** (innate 1/4, spells 1/10, spell power 85):
  - `BR_ACID`: acid, max 1600 damage
  - `BR_COLD`: frost, max 1600 damage
  - `BR_ELEC`: lightning, max 1600 damage
  - `BR_FIRE`: fire, max 1600 damage
  - `BR_POIS`: poison, max 800 damage
  - `S_HI_DRAGON`: summon ancient dragons
  - `S_KIN`: summon similar monsters
- **Defences**: im-acid, im-elec, im-fire, im-cold, im-pois, no-conf, no-sleep, no-hold
- **Flags**: smart, spirit, powerful, move-body, bashes doors, evil, dragon
- **Other flags**: CLEAR\_WEB, FORCE\_SLEEP, UNIQUE, MALE
- **Escorts / friends**: 100% 1d1 Kavlax the Many-Headed; 100% 1d1 Scatha the Worm; 100% 1d1 Itangast the Fire Drake; 100% 1d1 Smaug the Golden; 100% 1d1 Glaurung, Father of the Dragons
- **Drops**: DROP\_4, DROP\_GOOD, DROP\_GREAT, ONLY\_ITEM

> "Rushing Jaws" is his name, and death is his game; the greatest and most terrible of all dragonkind, his power dismayed even the Valar for a time.

### The Witch-King of Angmar

- **Depth** 85, **speed** +20, **HP** 5000, **AC** 144, **exp** 60000/level, **rarity** 3, **sleep** 10, **hearing** 90, **smell** 50
- **Melee**: hit 10d10, hit black\_breath 10d10, hit exp\_80 7d7, hit exp\_80 7d7
- **Spells** (spells 1/2, spell power 85):
  - `BLIND`: blind (save)
  - `BRAIN_SMASH`: cause brain smashing, max 180 damage (save)
  - `WOUND`: cause mortal wounds, max 280 damage (save)
  - `HOLD`: paralyze (save)
  - `SCARE`: terrify (save)
  - `TELE_AWAY`: teleport away
  - `BA_NETH`: produce nether storms, max 440 damage
  - `BO_MANA`: produce mana bolts, max 262 damage
  - `S_HI_DEMON`: summon greater demons
  - `S_HI_DRAGON`: summon ancient dragons
  - `S_HI_UNDEAD`: summon greater undead
  - `S_KIN`: summon similar monsters
  - `S_MONSTERS`: summon monsters
  - `S_WRAITH`: summon ringwraiths
- **Defences**: im-cold, im-pois, hurt-light, no-stun, no-conf, no-sleep, no-hold
- **Flags**: cold-blooded, smart, spirit, move-body, opens doors, bashes doors, evil, undead
- **Other flags**: CLEAR\_WEB, UNIQUE, MALE, FORCE\_SLEEP
- **Escorts / friends**: 50% 1d1 Winged Horror
- **Drops**: DROP\_4, DROP\_GOOD, DROP\_GREAT, ONLY\_ITEM

> The most fell of all Sauron's captains: King of Angmar, Sorcerer, Ringwraith, Lord of the Nazgûl, a spear of terror in the hand of Sauron, shadow of despair. His spells are lethal and his combat blows crushingly hard. He moves at speed, and commands legions of evil to do his bidding. It is said that he is fated never to die by the hand of mortal man.

### The Tarrasque

- **Depth** 86, **speed** +20, **HP** 8500, **AC** 222, **exp** 35000/level, **rarity** 2, **sleep** 20, **hearing** 50, **smell** 20
- **Melee**: hit 10d10, hit 10d10, touch drain\_charges, touch drain\_charges
- **Spells** (innate 1/2, spell power 86):
  - `BR_COLD`: frost, max 1600 damage
  - `BR_DISE`: disenchantment, max 500 damage
  - `BR_FIRE`: fire, max 1600 damage
- **Defences**: im-fire, im-cold, im-disen, no-conf, no-sleep, no-hold
- **Flags**: powerful, move-body, opens doors, bashes doors, animal
- **Other flags**: CLEAR\_WEB, UNIQUE, ATTR\_MULTI, FORCE\_SLEEP
- **Drops**: DROP\_4, DROP\_GOOD, ONLY\_ITEM

> The Tarrasque is a massive reptile of legend, rumoured to be unkillable and immune to magic. Fear its anger, for its devastation is unmatched!

### Meássë, the Bloody

- **Depth** 87, **speed** +10, **HP** 7000, **AC** 180, **exp** 42000/level, **rarity** 3, **sleep** 15, **hearing** 30
- **Melee**: hit confuse 12d12, hit confuse 12d12, hit confuse 12d12, hit confuse 12d12
- **Spells** (innate 1/6, spells 1/5, spell power 87):
  - `HEAL`: heal-self
  - `HEAL_KIN`: heal kin
  - `BOULDER`: throw boulders, max 156 damage
  - `BA_SHAR`: invoke storms of knives, max 140 damage
  - `BR_SOUN`: sound, max 500 damage
  - `S_KIN`: summon similar monsters
  - `S_MONSTERS`: summon monsters
- **Defences**: im-acid, im-elec, im-fire, im-cold, im-pois, no-stun, no-conf, no-sleep, no-hold
- **Flags**: smart, spirit, powerful, takes items, move-body, opens doors, bashes doors
- **Other flags**: CLEAR\_WEB, FORCE\_SLEEP, UNIQUE, FEMALE, NAME\_COMMA
- **Escorts / friends**: 50% 1d4 Maia of Oromë (servant); 50% 1d4 Maia of Aulë (servant); 50% 1d3 Lesser titan (servant); 50% 1d2 Greater titan (servant); 100% 1d1 Makar, the Warrior
- **Drops**: DROP\_4, DROP\_GOOD, ONLY\_ITEM

> Sister of Makar the Warrior, she would shout encouragement to the warriors of their hall and heal their wounds so they could battle on, until her arms were reddened to the elbow. Meássë is delighted by all violence, and recruits any beings of like abilities to her service.

### Lungorthin, the Balrog of White Fire

- **Depth** 88, **speed** +20, **HP** 7000, **AC** 187, **exp** 37000/level, **rarity** 2, **sleep** 10, **hearing** 20, **light** 2
- **Melee**: hit fire 8d12, hit fire 8d12, crush 8d12, touch drain\_charges
- **Spells** (innate 1/12, spells 1/6, spell power 88):
  - `BLIND`: blind (save)
  - `CONF`: confuse (save)
  - `SCARE`: terrify (save)
  - `BR_FIRE`: fire, max 1600 damage
  - `BR_PLAS`: plasma, max 150 damage
  - `S_HI_DEMON`: summon greater demons
  - `S_UNDEAD`: summon an undead
- **Defences**: im-fire, im-plasma, no-conf, no-sleep, no-hold
- **Flags**: spirit, powerful, move-body, opens doors, bashes doors, evil, demon
- **Other flags**: CLEAR\_WEB, FORCE\_SLEEP, UNIQUE, MALE, NAME\_COMMA
- **Escorts / friends**: 100% 1d3 Lesser Balrog (servant); 50% 1d2 Greater Balrog (servant); 100% 1d1 The Balrog of Moria
- **Drops**: DROP\_4, DROP\_GOOD, ONLY\_ITEM

> A massive form cloaked in flame. Lungorthin stares balefully at you with eyes that smoulder red. The dungeon floor where he stands is scorched by the heat of his body.

### Carcharoth, the Jaws of Thirst

- **Depth** 90, **speed** +20, **HP** 8000, **AC** 192, **exp** 40000/level, **rarity** 1, **sleep** 10, **hearing** 50, **smell** 50, **light** 2
- **Melee**: claw fire 9d12, claw fire 9d12, bite fire 9d12, bite fire 9d12
- **Spells** (innate 1/3, spells 1/15, spell power 90):
  - `BR_DARK`: darkness, max 500 damage
  - `BR_FIRE`: fire, max 1600 damage
  - `BR_NETH`: nether, max 600 damage
  - `BR_POIS`: poison, max 800 damage
  - `S_HOUND`: summon hounds
- **Defences**: im-fire, im-pois, im-nether, no-conf, no-sleep, no-hold
- **Flags**: smart, move-body, opens doors, bashes doors, evil, animal
- **Other flags**: CLEAR\_WEB, UNIQUE, MALE, NAME\_COMMA, ATTR\_FLICKER, FORCE\_SLEEP
- **Drops**: DROP\_4, DROP\_GOOD, ONLY\_ITEM

> The first guard of Angband, Carcharoth, also known as 'The Red Maw', is the greatest wolf to ever walk the earth. He is highly intelligent and a deadly opponent in combat.

### Huan, Wolfhound of the Valar

- **Depth** 90, **speed** +20, **HP** 8000, **AC** 192, **exp** 40000/level, **rarity** 1, **sleep** 10, **hearing** 50, **smell** 50
- **Melee**: claw cold 9d12, claw cold 9d12, bite cold 9d12, bite cold 9d12
- **Spells** (innate 1/5, spell power 90):
  - `BR_COLD`: frost, max 1600 damage
  - `BR_LIGHT`: light, max 500 damage
  - `BR_SHAR`: shards, max 500 damage
  - `BR_SOUN`: sound, max 500 damage
- **Defences**: im-acid, im-elec, im-cold, no-fear, no-stun, no-conf, no-sleep, no-hold
- **Flags**: smart, kill-body, opens doors, bashes doors, animal
- **Other flags**: CLEAR\_WEB, UNIQUE, MALE, NAME\_COMMA, ATTR\_FLICKER, FORCE\_SLEEP
- **Drops**: DROP\_4, DROP\_GOOD, ONLY\_ITEM

> The wolfhound of the Valar, Huan has served many masters in his time, from Celegorm son of Fëanor to Beren son of Barahir: but now he runs wild and acknowledges no master save himself, as he hunts alone for his nemesis - Carcharoth, the terrible wolf of Angband.

### Vecna, the Emperor Lich

- **Depth** 92, **speed** +20, **HP** 6000, **AC** 120, **exp** 45000/level, **rarity** 3, **sleep** 0, **hearing** 100
- **Melee**: hit exp\_80 7d12, hit lose\_dex 7d12, hit drain\_charges 7d12, hit drain\_charges 7d12
- **Spells** (spells 1/2, spell power 92):
  - `BLIND`: blind (save)
  - `WOUND`: cause mortal wounds, max 300 damage (save)
  - `SCARE`: terrify (save)
  - `TPORT`: teleport-self
  - `BRAIN_SMASH`: cause brain smashing, max 180 damage (save)
  - `BA_FIRE`: produce firestorms, max 332 damage
  - `BA_MANA`: invoke mana storms, max 560 damage
  - `BA_NETH`: produce nether storms, max 468 damage
  - `BO_MANA`: produce mana bolts, max 280 damage
  - `S_HI_DEMON`: summon greater demons
  - `S_HI_UNDEAD`: summon greater undead
  - `S_KIN`: summon similar monsters
  - `S_MONSTERS`: summon monsters
- **Defences**: im-elec, im-fire, im-cold, im-pois, no-stun, no-conf, no-sleep, no-hold
- **Flags**: cold-blooded, smart, spirit, opens doors, bashes doors, evil, undead
- **Other flags**: CLEAR\_WEB, FORCE\_SLEEP, UNIQUE, MALE, NAME\_COMMA
- **Escorts / friends**: 50% 1d3 mummified chieftain (servant); 50% 1d3 Skeleton etten (servant); 50% 1d3 Night mare (servant); 30% 1d2 Dreadlord (servant); 40% 1d2 Archlich (servant); 30% 1d2 Nightwing (servant); 30% 1d2 Nightcrawler (servant); 20% 1d2 Nightwalker (servant); 20% 1d2 Bone golem (servant); 20% 1d2 Black reaver (servant)
- **Drops**: DROP\_4, DROP\_GOOD, DROP\_GREAT, ONLY\_ITEM

> The greatest of all undead sorcerers, even the gods once feared him. This ancient shadow of death wilts every living thing it passes.

### Gothmog, the High Captain of Balrogs

- **Depth** 95, **speed** +20, **HP** 8000, **AC** 168, **exp** 43000/level, **rarity** 1, **sleep** 0, **hearing** 100, **light** 2
- **Melee**: hit fire 9d12, hit fire 9d12, crush 8d12, touch drain\_charges
- **Spells** (innate 1/8, spells 1/4, spell power 95):
  - `BLIND`: blind (save)
  - `CONF`: confuse (save)
  - `SCARE`: terrify (save)
  - `BR_FIRE`: fire, max 1600 damage
  - `BR_PLAS`: plasma, max 150 damage
  - `S_HI_DEMON`: summon greater demons
  - `S_HI_UNDEAD`: summon greater undead
- **Defences**: im-elec, im-fire, im-plasma, no-conf, no-sleep, no-hold
- **Flags**: spirit, powerful, move-body, opens doors, bashes doors, evil, demon
- **Other flags**: CLEAR\_WEB, FORCE\_SLEEP, UNIQUE, MALE, NAME\_COMMA
- **Escorts / friends**: 100% 1d5 Lesser Balrog (servant); 30% 1d3 Osyluth (servant); 30% 1d3 Gelugon (servant); 30% 1d3 Pit fiend (servant); 100% 1d2 Greater Balrog (bodyguard); 100% 1d1 The Balrog of Moria (servant); 100% 1d1 Lungorthin, the Balrog of White Fire (servant)
- **Drops**: DROP\_4, DROP\_GOOD, DROP\_GREAT, ONLY\_ITEM

> Gothmog is the Chief Balrog in Morgoth's personal guard. He is renowned for slaying Fëanor and Fingon, two of the High Kings of the Noldor, and has never been defeated in combat. With his whip of flame and awesome fiery breath he saved his master from Ungoliant's rage.

### Sauron, the Sorcerer

- **Depth** 99, **speed** +20, **HP** 8000, **AC** 192, **exp** 50000/level, **rarity** 1, **sleep** 0, **hearing** 100
- **Melee**: hit disenchant 10d12, hit disenchant 10d12, hit drain\_charges 8d12, hit drain\_charges 8d12
- **Spells** (spells 1/2, spell power 130):
  - `BRAIN_SMASH`: cause brain smashing, max 180 damage (save)
  - `WOUND`: cause mortal wounds, max 430 damage (save)
  - `TELE_LEVEL`: teleport level (save)
  - `TPORT`: teleport-self
  - `BA_DARK`: invoke darkness storms, max 620 damage
  - `BA_FIRE`: produce firestorms, max 465 damage
  - `BA_NETH`: produce nether storms, max 620 damage
  - `BO_ICE`: produce ice bolts, max 166 damage
  - `BO_MANA`: produce mana bolts, max 375 damage
  - `BO_PLAS`: produce plasma bolts, max 196 damage
  - `SHAPECHANGE`: change shape
  - `S_HI_DEMON`: summon greater demons
  - `S_HI_DRAGON`: summon ancient dragons
  - `S_HI_UNDEAD`: summon greater undead
  - `S_UNIQUE`: summon uniques
  - `S_WRAITH`: summon ringwraiths
- **Defences**: im-elec, im-fire, im-cold, im-pois, no-fear, no-conf, no-sleep, no-hold
- **Flags**: regenerates, smart, spirit, powerful, takes items, move-body, opens doors, bashes doors, evil
- **Other flags**: CLEAR\_WEB, FORCE\_SLEEP, UNIQUE, QUESTOR, MALE, NAME\_COMMA, FORCE\_DEPTH
- **Escorts / friends**: 100% 1d1 Khamûl, the Black Easterling (servant); 100% 1d1 The Witch-King of Angmar (servant)
- **Can change into**: Wolf-Sauron, Serpent-Sauron, Vampire-Sauron
- **Drops**: DROP\_4, DROP\_GOOD, DROP\_GREAT, ONLY\_ITEM
- **Quest monster**: must be killed to win (`lib/gamedata/quest.txt`).

> Greatest and most terrible of the servants of Morgoth, creator of the One Ring. A sorcerer of dreadful powers, master of shadows and of phantoms, foul in wisdom, cruel in strength, misshaping what he rules, lord of werewolves; his dominion is torment. He has many servants, and rarely fights without them.

### Serpent-Sauron

- **Depth** 99, **speed** +20, **HP** as the form it changed from, **AC** 164, **exp** 0/level, **rarity** 0, **sleep** 0, **hearing** 100
- **Melee**: bite poison 7d12, bite poison 7d12, crush 8d12, spit acid 4d12
- **Spells** (innate 1/4, spells 1/4, spell power 108):
  - `BLINK`: blink-self
  - `TPORT`: teleport-self
  - `TELE_LEVEL`: teleport level (save)
  - `TELE_SELF_TO`: teleport toward
  - `BLIND`: blind (save)
  - `CONF`: confuse (save)
  - `SCARE`: terrify (save)
  - `WOUND`: cause mortal wounds, max 360 damage (save)
  - `BRAIN_SMASH`: cause brain smashing, max 180 damage (save)
  - `FORGET`: cause amnesia (save)
  - `BR_POIS`: poison
  - `BR_CHAO`: chaos
  - `BR_ACID`: acid
  - `S_DRAGON`: summon a dragon
  - `S_HI_DEMON`: summon greater demons
  - `S_HI_UNDEAD`: summon greater undead
- **Defences**: im-acid, im-elec, im-cold, im-pois, hurt-cold, no-fear, no-conf, no-sleep
- **Flags**: regenerates, smart, move-body, opens doors, bashes doors, evil, animal
- **Other flags**: CLEAR\_WEB, UNIQUE, MALE, FORCE\_SLEEP, FORCE\_DEPTH
- **Drops**: none

> Sauron's form as a serpent: poisonous, biting, spitting, and threatening to crush you in its mighty coils.

### Vampire-Sauron

- **Depth** 99, **speed** +20, **HP** as the form it changed from, **AC** 164, **exp** 0/level, **rarity** 0, **sleep** 0, **hearing** 100
- **Melee**: bite exp\_80 6d12, claw terrify 8d12, hit drain\_charges 8d12, hit drain\_charges 8d12
- **Spells** (spells 1/2, spell power 130):
  - `BLINK`: blink-self
  - `TPORT`: teleport-self
  - `TELE_LEVEL`: teleport level (save)
  - `BLIND`: blind (save)
  - `CONF`: confuse (save)
  - `BRAIN_SMASH`: cause brain smashing, max 180 damage (save)
  - `FORGET`: cause amnesia (save)
  - `BO_ICE`: produce ice bolts, max 166 damage
  - `BO_PLAS`: produce plasma bolts, max 196 damage
  - `BE_NETH`: shoot rays of death, max 315 damage
  - `BA_NETH`: produce nether storms, max 620 damage
  - `STORM`: create tempests, max 715 damage
  - `BA_DARK`: invoke darkness storms, max 620 damage
  - `BA_POIS`: produce storms of poison, max 272 damage
  - `BA_ACID`: produce acid balls, max 405 damage
  - `S_DRAGON`: summon a dragon
  - `S_HI_DEMON`: summon greater demons
  - `S_HI_UNDEAD`: summon greater undead
- **Defences**: im-elec, im-cold, im-pois, hurt-light, hurt-fire, no-fear, no-stun, no-conf, no-sleep
- **Flags**: cold-blooded, regenerates, spirit, move-body, opens doors, bashes doors, evil, undead
- **Other flags**: CLEAR\_WEB, FORCE\_SLEEP, UNIQUE, MALE, FORCE\_DEPTH
- **Drops**: none

> Sauron's form as a vampire, great as a dark cloud across the moon.

### Wolf-Sauron

- **Depth** 99, **speed** +30, **HP** as the form it changed from, **AC** 164, **exp** 0/level, **rarity** 0, **sleep** 0, **hearing** 100
- **Melee**: bite poison 8d10, bite poison 8d10, claw confuse 6d8, claw paralyze 6d8
- **Spells** (innate 1/3, spells 1/3, spell power 105):
  - `BLINK`: blink-self
  - `TPORT`: teleport-self
  - `TELE_LEVEL`: teleport level (save)
  - `TELE_SELF_TO`: teleport toward
  - `BLIND`: blind (save)
  - `CONF`: confuse (save)
  - `SCARE`: terrify (save)
  - `WOUND`: cause mortal wounds, max 350 damage (save)
  - `BR_FIRE`: fire
  - `BR_NETH`: nether
  - `BA_DARK`: invoke darkness storms, max 520 damage
  - `S_HOUND`: summon hounds
  - `S_DRAGON`: summon a dragon
  - `S_HI_DEMON`: summon greater demons
  - `S_HI_UNDEAD`: summon greater undead
- **Defences**: im-elec, im-fire, im-cold, im-pois, im-nether, no-fear, no-conf, no-sleep
- **Flags**: regenerates, smart, move-body, opens doors, bashes doors, evil, animal
- **Other flags**: CLEAR\_WEB, UNIQUE, MALE, FORCE\_SLEEP, FORCE\_DEPTH
- **Drops**: none

> Sauron's form as the mightiest wolf to walk the world.

### Morgoth, Lord of Darkness

- **Depth** 100, **speed** +30, **HP** 20000, **AC** 180, **exp** 60000/level, **rarity** 1, **sleep** 0, **hearing** 100
- **Melee**: hit shatter 20d10, hit shatter 20d10, hit lose\_all 10d12, touch drain\_charges
- **Spells** (spells 1/3, spell power 100):
  - `BRAIN_SMASH`: cause brain smashing, max 180 damage (save)
  - `BA_MANA`: invoke mana storms, max 600 damage
  - `BA_NETH`: produce nether storms, max 500 damage
  - `BO_MANA`: produce mana bolts, max 300 damage
  - `S_HI_DEMON`: summon greater demons
  - `S_HI_DRAGON`: summon ancient dragons
  - `S_HI_UNDEAD`: summon greater undead
  - `S_MONSTERS`: summon monsters
  - `S_UNIQUE`: summon uniques
- **Defences**: im-acid, im-elec, im-fire, im-cold, im-pois, no-fear, no-conf, no-sleep, no-hold
- **Flags**: regenerates, smash-wall, smart, spirit, move-body, evil
- **Other flags**: UNIQUE, QUESTOR, MALE, NAME\_COMMA, ATTR\_FLICKER, FORCE\_DEPTH, FORCE\_SLEEP
- **Drops**: DROP\_4, DROP\_GOOD, DROP\_GREAT, ONLY\_ITEM; always the quest artifacts Mighty Hammer 'Grond', The Massive Iron Crown of Morgoth (`QUEST_ART` kinds, `mon_create_drop()` in `src/mon-make.c`)
- **Quest monster**: must be killed to win (`lib/gamedata/quest.txt`).

> He is the Master of the Pits of Angband. His figure is like a black mountain crowned with lightning. He rages with everlasting anger, his body scarred by Fingolfin's eight mighty wounds. He can never rest from his pain, but seeks forever to dominate all that is light and good in the world. He is the origin of man's fear of darkness and created many foul creatures with his evil powers. Orcs, Dragons, and Trolls are his most foul corruptions, causing much pain and suffering in the world to please him. His disgusting visage, twisted with evil, is crowned with iron, the two remaining Silmarils forever burning him. Grond, the mighty Hammer of the Underworld, cries defiance as he strides towards you to crush you to a pulp!
