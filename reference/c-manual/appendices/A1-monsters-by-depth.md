# Appendix A1. Monsters by depth

> **Generated file - do not edit by hand.** Produced by `docs/c-manual/tools/gen_appendices.py` from `lib/gamedata/monster.txt`, `lib/gamedata/monster_base.txt`, `lib/gamedata/monster_spell.txt`, `lib/gamedata/projection.txt`, `src/list-mon-spells.h` of the Angband source tree at commit `e473720ed`. Regenerate with `python3 docs/c-manual/tools/gen_appendices.py --src <angband-source-root>` (run from the repository root).

Every **non-unique** monster race, grouped in 5-level depth bands by native depth, with its combat statistics, melee blows, spells (with computed maximum damage), defences and tactically relevant flags. Uniques are in [A2](A2-uniques.md); spell definitions and breath rules are in [A3](A3-monster-spells.md). Narrative advice is in [Chapter 8](../08-monsters.md).

## Contents

- [Legend](#legend)
- [Town (level 0)](#town-level-0)
- [Levels 1-5](#levels-1-5)
- [Levels 6-10](#levels-6-10)
- [Levels 11-15](#levels-11-15)
- [Levels 16-20](#levels-16-20)
- [Levels 21-25](#levels-21-25)
- [Levels 26-30](#levels-26-30)
- [Levels 31-35](#levels-31-35)
- [Levels 36-40](#levels-36-40)
- [Levels 41-45](#levels-41-45)
- [Levels 46-50](#levels-46-50)
- [Levels 51-55](#levels-51-55)
- [Levels 56-60](#levels-56-60)
- [Levels 61-65](#levels-61-65)
- [Levels 66-70](#levels-66-70)
- [Levels 71-75](#levels-71-75)
- [Levels 76-80](#levels-76-80)
- [Levels 81-85](#levels-81-85)
- [Levels 91-95](#levels-91-95)
- [Monster bases (families)](#monster-bases-families)

## Legend

- **DL**: native depth (`depth:`); monsters are generated mostly at or below it, occasionally out of depth.
- **Sym**: display glyph (from the monster or its base in `monster_base.txt`).
- **Spd**: speed relative to normal (+10 = twice as many turns as a speed-normal character).
- **HP**: *average* hit points (`hit-points:`). Non-uniques roll `Rand_normal(avg, sd)` with `sd = (avg*10/8+5)/10 (+1 if avg > 1)` and a maximum of `avg + 4*sd` (`mon_hp()` in `src/mon-make.c`). `shape` marks shape-change forms (rarity 0, never generated alone) that keep the HP of the monster that changed.
- **AC**: armour class. **Exp**: experience *per level of the monster*: killing it gives `exp * DL / player level` (`monster.txt` header).
- **R**: rarity; generation weight is `100 / rarity` (0 = never generated normally).
- **Blows**: `method effect dice` per blow; plain damage blows show no effect.
- **Spells**: `innate 1/N` is the innate-attack frequency (breaths, arrows, boulders, spit: chance 1 in N per monster turn, i.e. 100/N %); `spells 1/N` is the spell frequency (`parse_monster_innate_freq()` / `parse_monster_spell_freq()` in `src/mon-init.c`; a monster with spells but no frequency line gets 1/4). The number after a spell is its **maximum damage** for this monster, computed as monster recall does (`mon_spell_lore_damage()` in `src/mon-spell.c`): breaths are `min(HP / divisor, cap)` at *average* HP (`breath_dam()`; divisors and caps in [A3](A3-monster-spells.md#breath-table)); other attacks are the maximum roll of their dice at the monster's spell power (`nonhp_dam()`). Before resistance.
- **Defences** (effects in `src/project-mon.c`): `im-acid/elec/fire/cold/pois` divide that element's damage by 9; `im-nexus/disen/plasma` reduce it to `3/(1d6+6)`; `im-nether` likewise (undead take no nether damage at all and other evil monsters half); `im-water` negates water. A monster that breathes an element also resists it (`project_monster_breath()`). `hurt-fire`/`hurt-cold` double fire/cold damage, `hurt-light` doubles light damage and makes weak light hurt, `hurt-rock` means stone-to-mud damages it. `no-*`: immune to fear/stun/confusion/sleep/hold/slow.
- **Flags**: inherited from the base and own flags (`flags-off` removed): detection (`INVISIBLE`; `cold-blooded` = not seen by infravision; `empty-mind` = never seen by telepathy; `weird-mind` = seen by telepathy only 1 time in 10), movement, AI, race; `friends:` gives `chance% number name` of the group it is generated with.

## Town (level 0)

| DL | Monster | Sym | Spd | HP | AC | Exp | R | Blows | Spells | Defences | Flags |
|---:|---|:-:|---:|---:|---:|---:|---:|---|---|---|---|
| 0 | aimless-looking merchant | `t` | +0 | 6 | 1 | 0 | 1 | hit 1d3 | - | - | erratic 50%, spirit, takes items, opens doors, bashes doors, friends: 30% 2d2 Filthy street urchin |
| 0 | battle-scarred veteran | `t` | +0 | 32 | 36 | 0 | 1 | hit 2d6 | - | - | erratic 50%, spirit, takes items, opens doors, bashes doors |
| 0 | blubbering idiot | `t` | +0 | 2 | 1 | 0 | 1 | drool | - | - | erratic 25%, spirit, takes items |
| 0 | boil-covered wretch | `t` | +0 | 2 | 1 | 0 | 1 | drool | - | - | erratic 25%, spirit, takes items, opens doors, bashes doors |
| 0 | filthy street urchin | `t` | +0 | 3 | 1 | 0 | 2 | beg, touch eat\_gold | - | - | erratic 25%, spirit, takes items, opens doors, friends: 50% 2d1 Scrawny cat; 50% 2d1 Scruffy little dog; 100% 3d4 Same |
| 0 | mangy-looking leper | `t` | +0 | 1 | 1 | 0 | 1 | beg | - | - | erratic 25%, spirit, takes items, opens doors |
| 0 | mean-looking mercenary | `t` | +0 | 23 | 24 | 0 | 1 | hit 1d10 | - | - | erratic 50%, spirit, takes items, opens doors, bashes doors, evil |
| 0 | pitiful-looking beggar | `t` | +0 | 3 | 1 | 0 | 1 | beg | - | - | erratic 25%, spirit, takes items, opens doors |
| 0 | red-hatted elf | `h` | +0 | 100 | 12 | 0 | 1 | hit 1d1 | - | - | spirit, opens doors, bashes doors, friends: 30% 1d5 Same |
| 0 | scrawny cat | `f` | +0 | 2 | 1 | 0 | 3 | claw 1d1 | - | - | erratic 25%, animal |
| 0 | scruffy little dog | `C` | +0 | 2 | 1 | 0 | 3 | bite 1d1 | - | - | erratic 25%, animal |
| 0 | singing, happy drunk | `t` | +0 | 4 | 1 | 0 | 1 | beg | - | - | erratic 50%, spirit, takes items, opens doors, bashes doors |
| 0 | squint-eyed rogue | `t` | +0 | 9 | 9 | 0 | 1 | hit 1d6, touch eat\_item, insult | - | - | spirit, takes items, opens doors, bashes doors, evil |
| 0 | village idiot | `t` | +10 | 10 | 1 | 0 | 1 | drool | - | - | erratic 25%, spirit, takes items |

## Levels 1-5

| DL | Monster | Sym | Spd | HP | AC | Exp | R | Blows | Spells | Defences | Flags |
|---:|---|:-:|---:|---:|---:|---:|---:|---|---|---|---|
| 1 | clear icky thing | `i` | +0 | 6 | 7 | 2 | 1 | touch 1d2 | - | - | INVISIBLE, empty-mind (no ESP), erratic 25%, erratic 50% |
| 1 | floating eye | `e` | +0 | 11 | 7 | 1 | 1 | gaze paralyze | - | hurt-light, no-fear | never-moves |
| 1 | fruit bat | `b` | +10 | 4 | 3 | 1 | 1 | bite 1d1 | - | - | erratic 25%, animal |
| 1 | giant white centipede | `c` | +0 | 9 | 12 | 2 | 1 | crawl 1d2, sting cold 1d2 | - | - | weird-mind (ESP 1 in 10), erratic 50%, bashes doors, animal |
| 1 | giant white mouse | `r` | +0 | 2 | 4 | 1 | 1 | bite 1d2 | - | - | BREEDS, erratic 50%, animal |
| 1 | giant yellow centipede | `c` | +0 | 7 | 14 | 2 | 1 | bite 1d3, sting 1d3 | - | - | weird-mind (ESP 1 in 10), animal |
| 1 | grey mold | `m` | +0 | 2 | 1 | 3 | 1 | spore 1d4, spore 1d4 | - | im-pois, hurt-fire, no-fear, no-conf, no-sleep | empty-mind (no ESP), never-moves, stupid |
| 1 | grey mushroom patch | `,` | +0 | 2 | 1 | 1 | 1 | spore confuse 1d4 | - | im-pois, hurt-fire, no-fear, no-conf, no-sleep | empty-mind (no ESP), never-moves, stupid |
| 1 | large white snake | `J` | -10 | 11 | 36 | 2 | 1 | bite 1d1, crush 1d1 | - | hurt-cold | erratic 50%, bashes doors, animal |
| 1 | rock lizard | `R` | +0 | 8 | 4 | 2 | 1 | bite 1d1 | - | hurt-cold | animal |
| 1 | small kobold | `k` | +0 | 8 | 24 | 5 | 1 | hit 1d5 | - | im-pois | spirit, opens doors, bashes doors, evil |
| 1 | soldier ant | `a` | +0 | 6 | 4 | 3 | 1 | bite 1d2 | - | - | weird-mind (ESP 1 in 10), bashes doors, animal |
| 1 | white icky thing | `i` | +0 | 6 | 8 | 1 | 1 | touch 1d2 | - | - | empty-mind (no ESP), erratic 25%, erratic 50% |
| 1 | white worm mass | `w` | -10 | 10 | 1 | 2 | 1 | crawl poison 1d2 | - | im-pois, hurt-light, no-fear | weird-mind (ESP 1 in 10), BREEDS, erratic 25%, erratic 50%, stupid, animal |
| 1 | wild dog | `C` | +0 | 3 | 3 | 1 | 1 | bite 1d1 | - | - | group-AI, animal, friends: 100% 2d7 Same |
| 2 | acolyte | `p` | +0 | 18 | 15 | 6 | 1 | hit 1d5 | spells 1/12: HEAL, SCARE | - | spirit, opens doors, bashes doors, friends: 50% 1d2 Same; 20% 1d3 any person; 40% 1d3 any person; 80% 1d3 any person |
| 2 | apprentice | `p` | +0 | 15 | 9 | 6 | 1 | hit 1d4 | spells 1/12: BLIND, BLINK, CONF, MISSILE 12 | - | spirit, opens doors, bashes doors, friends: 50% 1d2 Same; 20% 1d3 any person; 40% 1d3 any person; 80% 1d3 any person |
| 2 | blubbering icky thing | `i` | +0 | 18 | 4 | 8 | 1 | crawl poison 1d4, crawl eat\_food | - | im-pois | empty-mind (no ESP), erratic 50%, takes items, kill-body |
| 2 | blue yeek | `y` | +0 | 7 | 16 | 4 | 1 | hit 1d5 | - | im-acid | spirit, opens doors, bashes doors, animal |
| 2 | cave spider | `S` | +10 | 7 | 19 | 7 | 1 | bite 1d4 | - | - | weird-mind (ESP 1 in 10), group-AI, bashes doors, animal, friends: 100% 2d8 Same |
| 2 | crow | `B` | +10 | 9 | 14 | 8 | 2 | bite 1d3, bite 1d3 | - | - | animal |
| 2 | cutpurse | `p` | +0 | 20 | 18 | 6 | 1 | hit 1d6, touch eat\_gold | - | - | spirit, takes items, opens doors, bashes doors, evil, friends: 50% 1d2 Same; 20% 1d3 any person; 40% 1d3 any person; 80% 1d3 any person |
| 2 | giant black ant | `a` | +0 | 11 | 24 | 8 | 1 | bite 1d4 | - | - | weird-mind (ESP 1 in 10), erratic 25%, bashes doors, animal |
| 2 | giant green frog | `R` | +0 | 9 | 9 | 6 | 1 | bite 1d3 | - | hurt-cold | erratic 25%, bashes doors, animal |
| 2 | green worm mass | `w` | -10 | 15 | 3 | 3 | 1 | crawl acid 1d3 | - | im-acid, hurt-light, no-fear | weird-mind (ESP 1 in 10), BREEDS, erratic 25%, erratic 50%, stupid, animal |
| 2 | kobold | `k` | +0 | 12 | 24 | 5 | 1 | hit 1d8 | - | im-pois | spirit, opens doors, bashes doors, evil |
| 2 | metallic green centipede | `c` | +10 | 10 | 4 | 3 | 1 | crawl 1d1, sting acid 1d1 | - | - | weird-mind (ESP 1 in 10), erratic 50%, bashes doors, animal |
| 2 | salamander | `R` | +0 | 14 | 24 | 10 | 1 | bite fire 1d3 | - | im-fire, hurt-cold | erratic 25%, animal |
| 2 | shrieker mushroom patch | `,` | +0 | 1 | 1 | 1 | 1 | - | innate 1/4: SHRIEK | im-pois, hurt-fire, no-fear, no-conf, no-sleep | empty-mind (no ESP), never-moves, no melee, stupid |
| 2 | soldier | `p` | +0 | 23 | 24 | 6 | 1 | hit 1d7, hit 1d7 | - | - | spirit, opens doors, bashes doors, friends: 50% 1d2 Same; 20% 1d3 any person; 40% 1d3 any person; 80% 1d3 any person |
| 2 | white harpy | `H` | +0 | 6 | 20 | 5 | 1 | claw 1d1, claw 1d1, bite 1d2 | - | - | erratic 50%, evil, animal |
| 2 | white jelly | `j` | +10 | 36 | 1 | 10 | 1 | touch poison 1d2 | - | im-pois, hurt-light, hurt-cold, no-fear, no-conf, no-sleep | empty-mind (no ESP), never-moves, stupid |
| 2 | wild cat | `f` | +10 | 9 | 14 | 8 | 2 | claw 1d3, claw 1d3 | - | - | bashes doors, animal |
| 2 | yellow mushroom patch | `,` | +0 | 1 | 1 | 2 | 1 | spore terrify 1d6 | - | im-pois, hurt-fire, no-fear, no-conf, no-sleep | empty-mind (no ESP), never-moves, stupid |
| 3 | black naga | `n` | +0 | 27 | 60 | 20 | 1 | crush 1d8 | - | hurt-cold | erratic 25%, spirit, bashes doors, evil |
| 3 | clear worm mass | `w` | -10 | 10 | 1 | 4 | 2 | crawl poison 1d2 | - | im-pois, hurt-light, no-fear | INVISIBLE, weird-mind (ESP 1 in 10), BREEDS, erratic 25%, erratic 50%, stupid, animal |
| 3 | giant white ant | `a` | +0 | 11 | 19 | 7 | 1 | bite 1d4 | - | - | weird-mind (ESP 1 in 10), bashes doors, animal, friends: 100% 4d4 same |
| 3 | giant white louse | `I` | +10 | 1 | 6 | 1 | 1 | bite 1d1 | - | - | weird-mind (ESP 1 in 10), BREEDS, erratic 25%, erratic 50%, animal |
| 3 | green ooze | `j` | +10 | 8 | 19 | 4 | 2 | crawl acid 1d3 | - | im-acid, im-pois, hurt-cold, no-fear, no-conf, no-sleep | empty-mind (no ESP), erratic 25%, erratic 50%, stupid |
| 3 | kobold shaman | `k` | +0 | 11 | 24 | 15 | 1 | hit 1d8 | spells 1/5: WOUND 10, CONF, HEAL, MISSILE 13 | im-pois | spirit, opens doors, bashes doors, evil, friends: 30% 1d2 any kobold; 70% 1d2 any kobold |
| 3 | metallic blue centipede | `c` | +10 | 12 | 7 | 7 | 1 | crawl 1d2, sting elec 1d2 | - | - | weird-mind (ESP 1 in 10), erratic 50%, bashes doors, animal |
| 3 | metallic red centipede | `c` | +10 | 18 | 10 | 12 | 1 | crawl 1d2, sting fire 1d2 | - | - | weird-mind (ESP 1 in 10), erratic 25%, bashes doors, animal |
| 3 | poltergeist | `G` | +20 | 6 | 18 | 8 | 1 | touch terrify | spells 1/15: BLINK | im-cold, im-pois, hurt-light, no-stun, no-conf, no-sleep, no-hold | INVISIBLE, cold-blooded, pass-wall, erratic 25%, erratic 50%, spirit, takes items, evil, undead |
| 3 | radiation eye | `e` | +0 | 11 | 7 | 6 | 1 | gaze lose\_str 1d6 | spells 1/11: DRAIN\_MANA | hurt-light, no-fear | never-moves |
| 3 | scruffy looking hobbit | `h` | +0 | 9 | 9 | 4 | 1 | hit 1d4, touch eat\_gold | - | - | spirit, takes items, opens doors, bashes doors, evil |
| 3 | silver jelly | `j` | +10 | 45 | 1 | 12 | 2 | touch eat\_light 1d3, touch eat\_light 1d3 | spells 1/15: DRAIN\_MANA | im-pois, hurt-cold, no-fear, no-conf, no-sleep | empty-mind (no ESP), never-moves, stupid |
| 3 | spotted mushroom patch | `,` | +0 | 1 | 1 | 3 | 1 | spore poison 2d4 | - | im-pois, hurt-fire, no-fear, no-conf, no-sleep | empty-mind (no ESP), never-moves, stupid |
| 3 | yellow jelly | `j` | +10 | 45 | 1 | 12 | 1 | touch poison 1d3 | spells 1/15: DRAIN\_MANA | im-pois, hurt-light, hurt-cold, no-fear, no-conf, no-sleep | empty-mind (no ESP), never-moves, stupid |
| 3 | yellow mold | `m` | +0 | 36 | 12 | 9 | 1 | spore 1d4 | - | im-pois, hurt-fire, no-fear, no-conf, no-sleep | empty-mind (no ESP), never-moves, stupid |
| 3 | yellow worm mass | `w` | -10 | 18 | 4 | 4 | 2 | crawl lose\_dex 1d3 | - | hurt-light, no-fear | weird-mind (ESP 1 in 10), BREEDS, erratic 25%, erratic 50%, stupid, animal |
| 4 | blue jelly | `j` | +0 | 54 | 1 | 14 | 1 | touch cold 1d6 | - | im-cold, hurt-light, no-fear, no-conf, no-sleep | cold-blooded, empty-mind (no ESP), never-moves, stupid |
| 4 | blue worm mass | `w` | -10 | 23 | 14 | 5 | 1 | crawl cold 1d4 | - | im-cold, hurt-light, no-fear | cold-blooded, weird-mind (ESP 1 in 10), BREEDS, erratic 25%, erratic 50%, stupid, animal |
| 4 | cave lizard | `R` | +0 | 11 | 19 | 8 | 1 | bite 1d5 | - | hurt-cold | animal |
| 4 | creeping copper coins | `$` | -10 | 32 | 28 | 9 | 3 | hit 1d4, touch poison 2d4 | - | im-elec, im-pois, no-conf, no-sleep | cold-blooded, empty-mind (no ESP), unaware (mimic), mimic-invisible, bashes doors, metal |
| 4 | gallant | `p` | +0 | 27 | 24 | 18 | 1 | hit 1d7, hit 1d7 | spells 1/9: HEAL | - | spirit, opens doors, bashes doors, friends: 50% 1d2 Same; 20% 1d3 any person; 40% 1d3 any person; 80% 1d3 any person |
| 4 | giant white rat | `r` | +0 | 3 | 8 | 1 | 1 | bite poison 1d3 | - | - | BREEDS, erratic 25%, animal |
| 4 | kobold archer | `k` | +0 | 24 | 24 | 20 | 1 | hit 1d9 | innate 1/2: ARROW 6 | im-pois | spirit, opens doors, bashes doors, evil, friends: 30% 1d2 any kobold; 70% 1d2 any kobold |
| 4 | large grey snake | `J` | -10 | 27 | 61 | 14 | 1 | bite 1d5, crush 1d8 | - | hurt-cold | erratic 25%, bashes doors, animal |
| 4 | scout | `p` | +0 | 27 | 12 | 18 | 1 | hit 1d5, hit 1d5 | innate 1/3: ARROW 6; spells 1/10: HASTE | - | spirit, opens doors, bashes doors, friends: 50% 1d2 Same; 20% 1d3 any person; 40% 1d3 any person; 80% 1d3 any person |
| 4 | silver mouse | `r` | +0 | 2 | 4 | 1 | 1 | bite eat\_light 1d2 | - | - | BREEDS, erratic 50%, animal |
| 4 | tamer | `p` | +0 | 20 | 20 | 13 | 1 | hit 1d2, hit 1d2 | spells 1/8: HASTE, SLOW, CONF | - | spirit, opens doors, bashes doors, friends: 50% 1d2 Same; 20% 1d3 any person; 40% 1d3 any person; 80% 1d3 any person |
| 5 | blue ooze | `j` | +0 | 8 | 19 | 7 | 1 | crawl cold 1d4 | - | im-cold, no-fear, no-conf, no-sleep | empty-mind (no ESP), erratic 25%, erratic 50%, stupid |
| 5 | copperhead snake | `J` | +0 | 14 | 30 | 15 | 1 | bite poison 2d4 | - | im-pois, hurt-cold | erratic 50%, bashes doors, animal |
| 5 | disenchanter eye | `e` | -10 | 32 | 7 | 20 | 2 | gaze disenchant | spells 1/9: DRAIN\_MANA | im-disen, hurt-light, no-fear | never-moves |
| 5 | giant green dragon fly | `F` | +0 | 14 | 30 | 16 | 2 | bite poison 1d6 | innate 1/10: BR\_POIS 4 | im-pois | weird-mind (ESP 1 in 10), erratic 25%, erratic 50%, bashes doors, animal |
| 5 | giant white dragon fly | `F` | +0 | 14 | 30 | 15 | 2 | bite cold 1d6 | innate 1/10: BR\_COLD 4 | im-cold | weird-mind (ESP 1 in 10), erratic 50%, bashes doors, animal |
| 5 | green glutton ghost | `G` | +20 | 8 | 24 | 15 | 1 | touch eat\_food 1d1 | - | im-pois, no-stun, no-conf, no-sleep, no-hold | INVISIBLE, cold-blooded, pass-wall, erratic 25%, erratic 50%, spirit, evil, undead |
| 5 | green jelly | `j` | +10 | 99 | 1 | 18 | 1 | touch acid 1d2 | - | im-acid, hurt-light, hurt-cold, no-fear, no-conf, no-sleep | empty-mind (no ESP), never-moves, stupid |
| 5 | green naga | `n` | +0 | 41 | 48 | 30 | 1 | crush 1d8, spit acid 2d6 | - | im-acid, hurt-cold | erratic 25%, spirit, takes items, bashes doors, evil |
| 5 | grey icky thing | `i` | +0 | 18 | 14 | 10 | 1 | touch 1d5 | - | - | empty-mind (no ESP), erratic 50% |
| 5 | large kobold | `k` | +0 | 65 | 48 | 25 | 1 | hit 1d10 | - | im-pois | spirit, opens doors, bashes doors, evil, friends: 50% 1d2 any kobold; 30% 1d2 any kobold |
| 5 | red worm mass | `w` | -10 | 23 | 14 | 6 | 1 | crawl fire 1d6 | - | im-fire, hurt-light, no-fear | weird-mind (ESP 1 in 10), BREEDS, erratic 25%, erratic 50%, stupid, bashes doors, animal |
| 5 | rot jelly | `j` | +10 | 90 | 36 | 15 | 1 | touch eat\_food 2d3, touch lose\_wis 2d3 | - | im-pois, hurt-light, hurt-cold, no-fear, no-conf, no-sleep | empty-mind (no ESP), never-moves, stupid |
| 5 | skeleton kobold | `s` | +0 | 23 | 39 | 12 | 1 | hit 1d6 | - | im-cold, im-pois, no-fear, no-stun, no-conf, no-sleep | cold-blooded, empty-mind (no ESP), opens doors, bashes doors, evil, undead |

## Levels 6-10

| DL | Monster | Sym | Spd | HP | AC | Exp | R | Blows | Spells | Defences | Flags |
|---:|---|:-:|---:|---:|---:|---:|---:|---|---|---|---|
| 6 | brown mold | `m` | +0 | 68 | 14 | 20 | 1 | spore confuse 1d4 | - | im-pois, hurt-fire, no-fear, no-conf, no-sleep | empty-mind (no ESP), never-moves, stupid |
| 6 | creeping silver coins | `$` | -10 | 54 | 36 | 18 | 3 | hit 1d6, touch poison 2d6 | - | im-elec, im-pois, no-conf, no-sleep | cold-blooded, empty-mind (no ESP), unaware (mimic), mimic-invisible, bashes doors, metal |
| 6 | giant black dragon fly | `F` | +10 | 14 | 30 | 18 | 2 | - | innate 1/10: BR\_ACID 4 | im-acid | weird-mind (ESP 1 in 10), no melee, erratic 25%, erratic 50%, bashes doors, animal |
| 6 | giant brown bat | `b` | +20 | 14 | 18 | 10 | 1 | bite 1d3 | - | - | erratic 50%, animal |
| 6 | giant gold dragon fly | `F` | +10 | 14 | 30 | 18 | 2 | bite 1d3 | innate 1/10: BR\_SOUN 2 | im-fire, no-stun | weird-mind (ESP 1 in 10), erratic 25%, erratic 50%, bashes doors, animal |
| 6 | purple mushroom patch | `,` | +0 | 1 | 1 | 15 | 2 | spore lose\_con 1d2, spore lose\_con 1d2, spore lose\_con 1d2 | - | hurt-fire, no-fear, no-conf, no-sleep | empty-mind (no ESP), never-moves, stupid |
| 6 | rattlesnake | `J` | +0 | 24 | 36 | 20 | 1 | bite poison 2d5 | - | im-pois, hurt-cold | erratic 50%, bashes doors, animal |
| 6 | snaga | `o` | +0 | 36 | 48 | 15 | 1 | hit 1d8 | innate 1/15: ARROW 6 | hurt-light | spirit, opens doors, bashes doors, evil, orc, friends: 30% 1d3 Crow; 100% 2d9 Same |
| 7 | bloodshot eye | `e` | +0 | 45 | 7 | 15 | 3 | gaze blind 2d6 | spells 1/7: DRAIN\_MANA | hurt-light, no-fear | never-moves |
| 7 | cave orc | `o` | +0 | 55 | 48 | 20 | 1 | hit 1d8 | innate 1/13: ARROW 6 | hurt-light | spirit, opens doors, bashes doors, evil, orc, friends: 50% 1d8 snaga; 100% 2d6 Same |
| 7 | crow of Durthang | `B` | +10 | 12 | 14 | 10 | 3 | bite 1d3, bite 1d2 | spells 1/9: WOUND 10 | - | animal |
| 7 | eastern dwarf | `h` | +0 | 39 | 24 | 25 | 2 | hit 1d6, hit 1d6 | spells 1/10: CONF, DARKNESS | - | spirit, opens doors, bashes doors, evil |
| 7 | giant red frog | `R` | +0 | 23 | 19 | 16 | 1 | bite lose\_str 2d4 | - | hurt-cold | erratic 50%, bashes doors, animal |
| 7 | green icky thing | `i` | +0 | 23 | 14 | 18 | 2 | touch acid 2d5 | - | im-acid | empty-mind (no ESP), erratic 50% |
| 7 | lost soul | `G` | +0 | 9 | 12 | 18 | 2 | hit 2d2, touch lose\_wis | spells 1/15: DRAIN\_MANA, TPORT | im-cold, im-pois, no-stun, no-conf, no-sleep, no-hold | INVISIBLE, cold-blooded, pass-wall, erratic 50%, spirit, takes items, evil, undead |
| 7 | night lizard | `R` | +0 | 18 | 19 | 35 | 2 | bite 1d6, bite 1d6 | - | hurt-cold | animal |
| 7 | red jelly | `j` | +0 | 117 | 1 | 26 | 1 | touch lose\_str 1d5 | - | hurt-light, hurt-cold, no-fear, no-conf, no-sleep | empty-mind (no ESP), never-moves, stupid |
| 7 | red naga | `n` | +0 | 50 | 48 | 40 | 2 | crush 1d10, bite lose\_str 1d4 | - | hurt-cold | erratic 25%, spirit, takes items, bashes doors, evil |
| 7 | wood spider | `S` | +10 | 11 | 19 | 15 | 3 | bite 1d3, sting poison 1d4 | - | im-pois | weird-mind (ESP 1 in 10), group-AI, bashes doors, animal, friends: 100% 2d7 Same |
| 7 | zombified kobold | `z` | +0 | 27 | 21 | 14 | 1 | hit 1d2, hit 1d2 | - | im-cold, im-pois, hurt-fire, no-fear, no-stun, no-conf, no-sleep | cold-blooded, empty-mind (no ESP), opens doors, bashes doors, evil, undead |
| 8 | giant salamander | `R` | +0 | 24 | 60 | 50 | 1 | bite fire 3d6 | innate 1/9: BR\_FIRE 8 | im-fire, hurt-cold | erratic 25%, animal |
| 8 | green mold | `m` | +0 | 95 | 16 | 28 | 1 | spore terrify 1d4 | - | im-acid, im-pois, hurt-fire, no-fear, no-conf, no-sleep | empty-mind (no ESP), never-moves, stupid |
| 8 | lemure | `u` | +0 | 65 | 48 | 16 | 3 | hit 1d8 | - | im-fire, no-fear | spirit, opens doors, bashes doors, evil, demon, friends: 100% 2d7 Same |
| 8 | orc tracker | `o` | +0 | 55 | 40 | 25 | 1 | hit 1d8 | innate 1/12: ARROW 12 | hurt-light | spirit, opens doors, bashes doors, evil, orc |
| 8 | ruffian | `p` | +0 | 36 | 28 | 26 | 2 | hit 2d4, hit 2d4 | innate 1/8: SHOT 10, WHIP 12 | - | spirit, takes items, opens doors, bashes doors, evil |
| 8 | skeleton orc | `s` | +0 | 45 | 54 | 26 | 1 | hit 2d5 | - | im-cold, im-pois, no-fear, no-stun, no-conf, no-sleep | cold-blooded, empty-mind (no ESP), opens doors, bashes doors, evil, undead, orc |
| 8 | terrified yeek | `y` | +0 | 18 | 21 | 11 | 1 | - | innate 1/9: SHRIEK | im-acid | no melee, spirit, frightened, opens doors, bashes doors, animal |
| 9 | baby black dragon | `d` | +0 | 88 | 36 | 35 | 2 | claw 1d3, claw 1d3, bite 1d5 | innate 1/12: BR\_ACID 29 | im-acid | spirit, opens doors, bashes doors, evil, dragon |
| 9 | baby blue dragon | `d` | +0 | 88 | 36 | 35 | 2 | claw 1d3, claw 1d3, bite 1d5 | innate 1/12: BR\_ELEC 29 | im-elec | spirit, opens doors, bashes doors, evil, dragon |
| 9 | baby gold dragon | `d` | +0 | 88 | 36 | 35 | 2 | claw 1d3, claw 1d3, bite 1d5 | innate 1/12: BR\_SOUN 14 | no-stun | spirit, opens doors, bashes doors, evil, dragon |
| 9 | baby green dragon | `d` | +0 | 88 | 36 | 35 | 2 | claw 1d3, claw 1d3, bite 1d5 | innate 1/12: BR\_POIS 29 | im-pois | spirit, opens doors, bashes doors, evil, dragon |
| 9 | baby red dragon | `d` | +0 | 88 | 36 | 35 | 2 | claw 1d3, claw 1d3, bite 1d5 | innate 1/12: BR\_FIRE 29 | im-fire | spirit, opens doors, bashes doors, evil, dragon |
| 9 | baby white dragon | `d` | +0 | 88 | 36 | 35 | 2 | claw 1d3, claw 1d3, bite 1d5 | innate 1/12: BR\_COLD 29 | im-cold | spirit, opens doors, bashes doors, evil, dragon |
| 9 | black harpy | `H` | +10 | 14 | 26 | 19 | 1 | claw 1d2, claw 1d2, bite 1d3 | - | - | erratic 25%, evil, animal |
| 9 | bloodshot icky thing | `i` | +0 | 32 | 21 | 24 | 3 | touch 1d4, crawl acid 2d4 | spells 1/11: DRAIN\_MANA | - | empty-mind (no ESP), erratic 50% |
| 9 | cave bear | `q` | +0 | 36 | 52 | 25 | 1 | claw 1d6, claw 1d6, bite 1d8 | - | - | erratic 25%, bashes doors, animal |
| 9 | giant black louse | `I` | +10 | 2 | 8 | 3 | 1 | bite 1d2 | - | - | weird-mind (ESP 1 in 10), BREEDS, erratic 25%, erratic 50%, animal |
| 9 | giant grey rat | `r` | +0 | 4 | 14 | 2 | 1 | bite poison 1d4 | - | - | BREEDS, erratic 25%, animal |
| 9 | giant red ant | `a` | +0 | 18 | 40 | 22 | 2 | bite 1d4, sting lose\_str 1d4 | - | - | weird-mind (ESP 1 in 10), bashes doors, animal |
| 9 | king cobra | `J` | +0 | 44 | 45 | 28 | 2 | spit blind 1d2, bite poison 3d4 | innate 1/4: SPIT 8 | im-pois, hurt-cold | erratic 50%, bashes doors, animal |
| 9 | orc shaman | `o` | +0 | 41 | 22 | 30 | 1 | hit 1d6, hit 1d6 | spells 1/8: BLINK, WOUND 30, MISSILE 15 | hurt-light | spirit, opens doors, bashes doors, evil, orc |
| 9 | yeti | `Y` | +0 | 55 | 36 | 30 | 3 | claw 1d3, claw 1d3, bite 1d4 | - | im-cold | opens doors, bashes doors, animal |
| 10 | blacklock mage | `h` | +10 | 39 | 24 | 50 | 1 | hit 1d6, hit 1d6 | spells 1/5: BLIND, CONF, DARKNESS, MISSILE 15, BA\_POIS 32 | - | spirit, opens doors, bashes doors, evil |
| 10 | clear mushroom patch | `,` | +10 | 1 | 1 | 3 | 2 | spore 1d1 | - | hurt-fire, no-fear, no-conf, no-sleep | INVISIBLE, cold-blooded, empty-mind (no ESP), BREEDS, never-moves, stupid |
| 10 | creeping gold coins | `$` | -10 | 81 | 43 | 32 | 3 | hit 2d5, touch poison 3d5 | - | im-acid, im-elec, im-pois, no-conf, no-sleep | cold-blooded, empty-mind (no ESP), unaware (mimic), mimic-invisible, bashes doors, metal |
| 10 | disenchanter mold | `m` | +0 | 72 | 30 | 40 | 1 | spore disenchant 1d6 | spells 1/11: DRAIN\_MANA | im-pois, im-disen, hurt-fire, no-fear, no-conf, no-sleep | empty-mind (no ESP), never-moves, stupid |
| 10 | giant fruit fly | `I` | +10 | 3 | 16 | 4 | 3 | bite 1d2 | - | - | weird-mind (ESP 1 in 10), BREEDS, erratic 25%, erratic 50%, animal |
| 10 | giant spider | `S` | +0 | 55 | 24 | 35 | 2 | bite 1d10, bite poison 1d6, bite poison 1d6, bite 1d10 | innate 1/10: WEAVE | im-pois | weird-mind (ESP 1 in 10), bashes doors, animal |
| 10 | giant white tick | `S` | -10 | 54 | 150 | 27 | 2 | bite poison 2d6 | - | im-pois | weird-mind (ESP 1 in 10), bashes doors, animal |
| 10 | hairy mold | `m` | +0 | 68 | 22 | 32 | 1 | spore poison 1d3 | - | im-pois, hurt-fire, no-fear, no-conf, no-sleep | empty-mind (no ESP), never-moves, stupid |
| 10 | panther | `f` | +10 | 45 | 36 | 25 | 2 | claw 1d8, claw 1d8 | - | - | bashes doors, animal |
| 10 | pseudo-dragon | `d` | +0 | 176 | 36 | 150 | 2 | claw 1d3, claw 1d3, bite 1d5 | innate 1/15: BR\_DARK 29, BR\_LIGHT 29; spells 1/20: CONF, SCARE | - | spirit, bashes doors, evil, dragon |
| 10 | stonefoot warrior | `h` | +0 | 60 | 24 | 50 | 1 | hit 1d8, hit 1d8 | - | - | spirit, opens doors, bashes doors, evil |
| 10 | tengu | `u` | +10 | 80 | 38 | 40 | 1 | hit 1d8 | spells 1/3: BLINK, TELE\_TO | im-fire, no-fear | spirit, opens doors, bashes doors, evil, demon |
| 10 | witch | `p` | +0 | 41 | 48 | 35 | 2 | hit 1d6 | innate 1/5: WHIP 6; spells 1/8: WOUND 30, SCARE, DARKNESS | - | spirit, takes items, opens doors, bashes doors, evil, friends: 50% 1d2 Illusionist; 50% 1d2 Priest; 50% 1d2 warrior; 50% 1d2 Same |
| 10 | wolf | `C` | +10 | 21 | 45 | 30 | 1 | bite 1d6 | - | - | erratic 25%, group-AI, bashes doors, animal, friends: 100% 2d10 Same |

## Levels 11-15

| DL | Monster | Sym | Spd | HP | AC | Exp | R | Blows | Spells | Defences | Flags |
|---:|---|:-:|---:|---:|---:|---:|---:|---|---|---|---|
| 11 | baby multi-hued dragon | `d` | +0 | 114 | 36 | 45 | 2 | claw 1d4, claw 1d4, bite 1d6 | innate 1/12: BR\_ACID 38, BR\_COLD 38, BR\_ELEC 38, BR\_FIRE 38, BR\_POIS 38 | im-acid, im-elec, im-fire, im-cold, im-pois | spirit, opens doors, bashes doors, evil, dragon |
| 11 | drúadan mage | `p` | +0 | 52 | 30 | 48 | 2 | hit 1d5 | spells 1/4: BLINK, DARKNESS, BO\_COLD 51, S\_MONSTER | - | spirit, bashes doors, friends: 100% 4d4 Same |
| 11 | hippogriff | `H` | +0 | 100 | 21 | 30 | 1 | hit 2d5, bite 2d5 | - | - | bashes doors, animal |
| 11 | old forest tree | `l` | -10 | 800 | 50 | 100 | 2 | crush 1d20 | spells 1/12: CONF | im-water, hurt-fire, no-fear, no-stun, no-conf, no-slow | cold-blooded, weird-mind (ESP 1 in 10), regenerates, never-moves, friends: 100% 2d4 same |
| 11 | zombified orc | `z` | +0 | 50 | 36 | 30 | 1 | hit 1d4, hit 1d4, hit 1d4 | - | im-cold, im-pois, hurt-fire, no-fear, no-stun, no-conf, no-sleep | cold-blooded, empty-mind (no ESP), opens doors, bashes doors, evil, undead, orc |
| 12 | air spirit | `E` | +20 | 36 | 48 | 40 | 2 | hit 1d3 | - | im-elec, im-pois, no-fear, no-conf, no-sleep | INVISIBLE, cold-blooded, empty-mind (no ESP), erratic 25%, erratic 50%, bashes doors, nonliving |
| 12 | black mamba | `J` | +10 | 45 | 48 | 40 | 3 | bite poison 4d6 | innate 1/4: SPIT 24 | im-pois, hurt-cold | erratic 50%, bashes doors, animal |
| 12 | grape jelly | `j` | +0 | 234 | 1 | 60 | 3 | touch exp\_10 | spells 1/11: DRAIN\_MANA | im-pois, hurt-light, hurt-cold, no-fear, no-conf, no-sleep | empty-mind (no ESP), never-moves, stupid |
| 12 | ironfist priest | `h` | +10 | 39 | 45 | 50 | 1 | hit 1d9, hit 1d10 | spells 1/5: BLIND, WOUND 40, CONF, DARKNESS, HEAL | - | smart, spirit, opens doors, bashes doors, evil |
| 12 | master yeek | `y` | +0 | 60 | 28 | 28 | 2 | hit 1d8 | spells 1/4: BLIND, BLINK, SLOW, TPORT, BA\_POIS 36, S\_MONSTER | im-acid | spirit, opens doors, bashes doors, evil, animal |
| 12 | moaning spirit | `G` | +10 | 23 | 24 | 44 | 2 | wail terrify, touch lose\_dex 1d8 | spells 1/15: SCARE, TPORT | im-cold, im-pois, no-stun, no-conf, no-sleep, no-hold | INVISIBLE, cold-blooded, pass-wall, erratic 25%, spirit, evil, undead |
| 12 | nether worm mass | `w` | -10 | 23 | 22 | 6 | 3 | touch exp\_10 | - | im-nether, hurt-light, no-fear | weird-mind (ESP 1 in 10), BREEDS, erratic 25%, erratic 50%, stupid, bashes doors, animal |
| 12 | priest | `p` | +0 | 54 | 26 | 36 | 1 | hit 2d3, hit 2d3 | spells 1/3: HEAL, SCARE, HEAL\_KIN, BA\_LIGHT 19, BA\_HOLY 19, S\_KIN | - | smart, spirit, opens doors, bashes doors, friends: 50% 1d2 Brigand; 50% 1d2 Illusionist; 50% 1d2 warrior; 50% 1d2 Same |
| 12 | skeleton human | `s` | +0 | 45 | 45 | 38 | 1 | hit 1d8 | - | im-cold, im-pois, no-fear, no-stun, no-conf, no-sleep | cold-blooded, empty-mind (no ESP), opens doors, bashes doors, evil, undead |
| 12 | spotted jelly | `j` | +10 | 59 | 27 | 33 | 3 | touch acid 1d10, touch acid 2d6, touch acid 2d6 | - | im-acid, im-pois, hurt-light, hurt-cold, no-fear, no-conf, no-sleep | cold-blooded, empty-mind (no ESP), never-moves, stupid |
| 12 | stegocentipede | `c` | +10 | 59 | 36 | 40 | 2 | bite 2d4, bite 2d4, sting 2d4 | - | - | weird-mind (ESP 1 in 10), bashes doors, animal |
| 12 | tiger | `f` | +10 | 66 | 48 | 40 | 2 | claw 1d8, claw 1d8, bite 1d6 | - | - | bashes doors, animal |
| 12 | warrior | `p` | +0 | 54 | 51 | 40 | 1 | hit 3d5, hit 3d5 | innate 1/10: BOLT 14 | - | spirit, opens doors, bashes doors, friends: 50% 1d2 Brigand; 50% 1d2 Priest; 50% 1d2 Illusionist; 50% 1d2 Same |
| 12 | white wolf | `C` | +10 | 28 | 45 | 30 | 1 | bite 1d3, bite 1d4 | - | im-cold | erratic 25%, group-AI, bashes doors, animal, friends: 100% 2d7 Same |
| 12 | zombified human | `z` | +0 | 54 | 36 | 34 | 1 | hit 1d4, hit 1d4 | - | im-cold, im-pois, hurt-fire, no-fear, no-stun, no-conf, no-sleep | cold-blooded, empty-mind (no ESP), opens doors, bashes doors, evil, undead |
| 13 | creeping mithril coins | `$` | +0 | 90 | 60 | 45 | 3 | hit 2d5, touch poison 3d5 | - | im-acid, im-elec, im-pois, no-conf, no-sleep | cold-blooded, empty-mind (no ESP), unaware (mimic), mimic-invisible, bashes doors, metal |
| 13 | druid | `p` | +0 | 78 | 15 | 50 | 2 | hit 2d4, hit 2d4 | spells 1/3: BLIND, HASTE, HOLD, SLOW, CONF, BO\_POIS 76, BO\_FIRE 76, SHAPECHANGE, S\_ANIMAL | - | smart, spirit, opens doors, bashes doors |
| 13 | illusionist | `p` | +10 | 54 | 15 | 50 | 2 | hit 2d2 | spells 1/3: BLIND, BLINK, CONF, DARKNESS, HASTE, HOLD, SLOW, TPORT | - | smart, spirit, opens doors, bashes doors, evil, friends: 50% 1d2 Brigand; 50% 1d2 Priest; 50% 1d2 warrior; 50% 1d2 Same |
| 13 | killer brown beetle | `K` | +0 | 59 | 72 | 45 | 1 | bite 4d4 | - | - | weird-mind (ESP 1 in 10), bashes doors, animal |
| 13 | ochre jelly | `j` | +10 | 59 | 21 | 40 | 3 | touch acid 1d10, touch acid 2d6, touch acid 2d6 | - | im-acid, im-pois, hurt-cold, no-fear, no-conf, no-sleep | cold-blooded, empty-mind (no ESP), stupid, takes items, opens doors, bashes doors |
| 13 | ogre | `O` | +0 | 65 | 49 | 50 | 2 | hit 2d8 | - | - | spirit, opens doors, bashes doors, evil, giant, friends: 100% 2d4 Same; 100% 2d4 any orc |
| 13 | orc archer | `o` | +0 | 66 | 54 | 45 | 1 | hit 2d4, hit 2d4 | innate 1/3: ARROW 18 | hurt-light | spirit, opens doors, bashes doors, evil, orc, friends: 100% 1d3 Same |
| 14 | blue icky thing | `i` | -10 | 35 | 30 | 20 | 4 | crawl poison 1d4, crawl eat\_food, hit 1d4, hit 1d4 | spells 1/8: BLIND, CONF, SCARE | im-pois | BREEDS, erratic 50%, opens doors, bashes doors, evil |
| 14 | flesh golem | `g` | +0 | 54 | 36 | 50 | 2 | hit 1d6, hit 1d6 | - | im-elec, no-fear, no-conf, no-sleep | empty-mind (no ESP), bashes doors, nonliving |
| 14 | giant flea | `I` | +10 | 3 | 30 | 4 | 3 | bite 1d2 | - | - | weird-mind (ESP 1 in 10), BREEDS, erratic 25%, erratic 50%, bashes doors, animal |
| 14 | lurker | `x` | +0 | 176 | 30 | 80 | 3 | hit 1d8, hit 1d8 | - | no-fear, no-conf, no-sleep | INVISIBLE, cold-blooded, empty-mind (no ESP), never-moves, unaware (mimic) |
| 14 | warg | `C` | +10 | 36 | 30 | 40 | 2 | bite 1d8 | - | - | erratic 25%, group-AI, bashes doors, evil, animal, friends: 100% 2d7 Same |
| 15 | black ogre | `O` | +0 | 100 | 49 | 75 | 2 | hit 2d8, hit 2d8 | - | - | erratic 25%, spirit, opens doors, bashes doors, evil, giant, friends: 60% 2d4 Ogre; 100% 3d4 Same |
| 15 | clay golem | `g` | +0 | 63 | 36 | 60 | 2 | hit 1d8, hit 1d8 | - | im-elec, im-fire, im-cold, im-pois, hurt-rock, no-fear, no-conf, no-sleep | cold-blooded, empty-mind (no ESP), bashes doors, nonliving |
| 15 | clear hound | `Z` | +0 | 21 | 36 | 50 | 3 | claw 1d6, claw 1d6, bite 1d8 | - | - | INVISIBLE, group-AI, bashes doors, animal, friends: 100% 2d7 Same |
| 15 | dark hound | `Z` | +0 | 21 | 36 | 50 | 3 | claw 1d4, claw 1d4, bite 1d6 | innate 1/5: BR\_DARK 3 | - | group-AI, bashes doors, animal, friends: 100% 2d7 Same |
| 15 | giant clear centipede | `c` | +0 | 23 | 36 | 30 | 2 | bite 2d4, sting 2d4 | - | - | INVISIBLE, weird-mind (ESP 1 in 10), bashes doors, animal |
| 15 | giant tan bat | `b` | +20 | 14 | 30 | 18 | 2 | bite terrify 1d3, claw 1d2, claw 1d2 | - | - | erratic 25%, animal, friends: 100% 2d6 Same |
| 15 | giant tarantula | `S` | +10 | 80 | 48 | 70 | 3 | bite poison 1d6, bite poison 1d6, bite poison 1d6 | - | im-pois | weird-mind (ESP 1 in 10), bashes doors, animal |
| 15 | griffon | `H` | +0 | 135 | 22 | 70 | 1 | hit 3d4, bite 2d6 | - | - | bashes doors, animal |
| 15 | guardian naga | `n` | +0 | 144 | 78 | 80 | 2 | crush 2d8, bite 1d8, bite 1d8 | - | hurt-cold | erratic 25%, spirit, opens doors, bashes doors, evil |
| 15 | half-orc | `o` | +0 | 88 | 60 | 50 | 2 | hit 3d4, hit 3d4 | innate 1/10: ARROW 12 | - | spirit, opens doors, bashes doors, evil, orc, friends: 50% 2d4 Warg; 50% 1d2 orc tracker; 100% 4d4 Same |
| 15 | homunculus | `u` | +0 | 36 | 48 | 40 | 3 | hit paralyze 1d2, hit 1d10 | - | im-fire, no-fear | spirit, opens doors, bashes doors, evil, demon |
| 15 | light hound | `Z` | +0 | 21 | 36 | 50 | 3 | claw 1d4, claw 1d4, bite 1d6 | innate 1/5: BR\_LIGHT 3 | - | group-AI, bashes doors, animal, friends: 100% 2d7 Same |
| 15 | magic mushroom patch | `,` | +30 | 1 | 12 | 10 | 2 | spore confuse, spore confuse, spore hallu, spore hallu | spells 1/2: BLINK, DARKNESS, SCARE, SLOW | hurt-fire, no-fear, no-conf, no-sleep | empty-mind (no ESP), never-moves, stupid |
| 15 | wererat | `r` | +0 | 90 | 15 | 45 | 2 | claw 1d8, claw 1d8, bite 2d6 | spells 1/9: BLINK, WOUND 50, BA\_POIS 40, BO\_COLD 53 | - | opens doors, bashes doors, evil, animal |

## Levels 16-20

| DL | Monster | Sym | Spd | HP | AC | Exp | R | Blows | Spells | Defences | Flags |
|---:|---|:-:|---:|---:|---:|---:|---:|---|---|---|---|
| 16 | craban | `B` | +10 | 9 | 14 | 20 | 4 | claw 1d4, claw 1d4 | innate 1/8: SHRIEK | - | group-AI, animal, friends: 100% 3d3 Same |
| 16 | gelatinous cube | `j` | +0 | 316 | 21 | 80 | 4 | touch acid 1d10, touch acid 1d10, touch acid 1d10 | - | im-acid, im-elec, im-fire, im-cold, im-pois, no-fear, no-conf, no-sleep | cold-blooded, empty-mind (no ESP), stupid, takes items, opens doors, bashes doors |
| 16 | grizzly bear | `q` | +0 | 78 | 52 | 55 | 2 | claw 1d8, claw 1d8, bite 1d12, crush 1d10 | - | - | bashes doors, animal |
| 16 | quasit | `u` | +0 | 27 | 36 | 50 | 2 | bite lose\_dex 1d6, claw 1d3, claw 1d3 | spells 1/10: BLIND, BLINK, CONF, SCARE, TELE\_LEVEL, TELE\_TO, TPORT | im-fire | INVISIBLE, erratic 25%, smart, spirit, bashes doors, evil, demon |
| 16 | rogue | `p` | +5 | 62 | 48 | 50 | 2 | hit 2d4, hit 2d4, touch eat\_item | innate 1/8: SHOT 15 | - | spirit, takes items, opens doors, bashes doors, evil, friends: 50% 1d2 Illusionist; 50% 1d2 Priest; 50% 1d2 warrior; 50% 1d2 Same |
| 16 | umber hulk | `X` | +0 | 110 | 75 | 75 | 1 | gaze confuse, hit 1d6, hit 1d6, bite 2d6 | - | im-pois, hurt-rock, no-conf, no-sleep | cold-blooded, empty-mind (no ESP), kill-wall, bashes doors, evil, animal |
| 16 | uruk | `o` | +0 | 70 | 75 | 60 | 1 | hit 3d5, hit 3d5 | innate 1/9: ARROW 18, WHIP 22 | - | spirit, opens doors, bashes doors, evil, orc, friends: 40% 1d4 Half-orc; 80% 2d5 Warg; 50% 1d2 orc tracker; 50% 1d2 orc archer; 100% 3d3 Same |
| 17 | 2-headed hydra | `M` | +0 | 200 | 90 | 80 | 2 | bite 2d6, bite 2d6 | spells 1/11: SCARE | hurt-cold | move-body, bashes doors, animal |
| 17 | earth spirit | `E` | +10 | 59 | 60 | 64 | 2 | hit 1d8, hit 1d8 | - | im-elec, im-fire, im-cold, im-pois, hurt-rock, no-fear, no-conf, no-sleep | cold-blooded, empty-mind (no ESP), pass-wall, erratic 25%, nonliving |
| 17 | forest troll | `T` | +0 | 110 | 75 | 70 | 1 | hit 1d4, hit 1d4, bite 1d6 | - | hurt-light | spirit, opens doors, bashes doors, evil, troll, friends: 100% 4d4 Same |
| 17 | giant red scorpion | `S` | +5 | 50 | 52 | 62 | 1 | claw 2d4, claw 2d4, sting lose\_str 1d7 | - | im-pois | weird-mind (ESP 1 in 10), bashes doors, animal |
| 17 | imp | `u` | +0 | 27 | 36 | 55 | 2 | hit poison 3d4, hit poison 3d4 | spells 1/10: BLIND, BLINK, CONF, SCARE, TELE\_LEVEL, TELE\_TO, TPORT | im-fire | INVISIBLE, cold-blooded, erratic 25%, smart, spirit, bashes doors, evil, demon |
| 17 | water spirit | `E` | +10 | 41 | 42 | 58 | 2 | hit 2d4, hit 2d4 | - | im-acid, im-pois, im-water, no-fear, no-conf, no-sleep | cold-blooded, empty-mind (no ESP), erratic 25%, bashes doors, nonliving |
| 18 | blackguard | `p` | +5 | 120 | 85 | 180 | 1 | hit 3d5, hit 3d5, hit 3d5 | innate 1/6: BOLT 21; spells 1/12: DARKNESS, SCARE | - | spirit, opens doors, bashes doors, evil, friends: 50% 1d1 rogue; 50% 1d1 ranger; 50% 1d1 paladin |
| 18 | blink dog | `C` | +10 | 36 | 24 | 50 | 2 | bite 1d8 | spells 1/4: BLINK, TELE\_TO | - | erratic 25%, group-AI, bashes doors, animal, friends: 100% 2d7 Same |
| 18 | cold hound | `Z` | +0 | 35 | 36 | 70 | 3 | claw 3d3, claw 3d3, bite cold 2d6 | innate 1/10: BR\_COLD 11 | im-cold, hurt-fire | group-AI, bashes doors, animal, friends: 100% 2d7 Same |
| 18 | energy hound | `Z` | +0 | 35 | 36 | 70 | 3 | claw 3d3, claw 3d3, bite elec 2d6 | innate 1/10: BR\_ELEC 11 | im-elec | group-AI, bashes doors, animal, friends: 100% 2d7 Same |
| 18 | evil eye | `e` | +0 | 68 | 7 | 80 | 3 | gaze exp\_10, gaze exp\_10 | spells 1/5: HOLD, TELE\_TO | hurt-light, no-fear | never-moves, evil |
| 18 | fire hound | `Z` | +0 | 35 | 36 | 70 | 3 | claw 3d3, claw 3d3, bite fire 2d6 | innate 1/10: BR\_FIRE 11 | im-fire, hurt-cold | group-AI, bashes doors, animal, friends: 100% 2d7 Same |
| 18 | fire spirit | `E` | +10 | 50 | 36 | 75 | 2 | hit fire 2d6, hit fire 2d6 | - | im-fire, im-pois, hurt-cold, no-fear, no-conf, no-sleep | empty-mind (no ESP), erratic 25%, bashes doors, nonliving |
| 18 | hummerhorn | `I` | +10 | 3 | 16 | 4 | 4 | bite confuse 2d2 | - | - | weird-mind (ESP 1 in 10), BREEDS, erratic 25%, erratic 50%, move-body, animal |
| 18 | orc captain | `o` | +0 | 110 | 88 | 80 | 3 | hit 3d5, hit 3d5, hit 3d5 | innate 1/9: ARROW 18 | - | spirit, opens doors, bashes doors, evil, orc, friends: 50% 1d5 Wolf; 50% 1d5 Warg; 80% 1d2 orc tracker (servant); 80% 1d4 orc archer (servant); 50% 1d5 any orc (servant); 50% 1d5 any orc (servant); 50% 1d5 any orc (servant) |
| 18 | potion mimic | `!` | +0 | 55 | 36 | 60 | 3 | hit poison 3d4, hit 2d3, hit 2d3 | spells 1/6: BO\_COLD 54, BLIND, WOUND 60, CONF, SCARE | hurt-cold, no-fear, no-conf, no-sleep | cold-blooded, empty-mind (no ESP), never-moves, unaware (mimic) |
| 18 | shambling mound | `,` | +0 | 70 | 19 | 75 | 2 | hit 1d8, hit 1d8 | innate 1/4: SHRIEK | hurt-fire, no-fear, no-conf, no-sleep | empty-mind (no ESP), stupid, opens doors, bashes doors, evil, friends: 60% 1d1 Giant fruit fly |
| 19 | red mold | `m` | +0 | 77 | 19 | 64 | 1 | spore fire 4d4 | - | im-fire, im-pois, no-fear, no-conf, no-sleep | empty-mind (no ESP), never-moves, stupid |
| 19 | stone golem | `g` | -10 | 126 | 90 | 100 | 2 | hit 1d10, hit 1d10 | - | im-elec, im-fire, im-cold, im-pois, hurt-rock, no-fear, no-conf, no-sleep | cold-blooded, empty-mind (no ESP), bashes doors, nonliving |
| 20 | 3-headed hydra | `M` | +10 | 300 | 97 | 350 | 2 | bite 2d6, bite 2d6, bite poison 2d6 | spells 1/9: SCARE | im-pois, hurt-cold | move-body, bashes doors, animal |
| 20 | air hound | `Z` | +0 | 68 | 36 | 200 | 3 | bite poison 1d8, bite poison 1d8, claw 3d3, claw 3d3 | innate 1/10: BR\_POIS 22 | im-pois | group-AI, bashes doors, animal, friends: 100% 2d7 Same |
| 20 | blood falcon | `B` | +30 | 6 | 2 | 50 | 2 | claw 2d10, claw 2d10, bite 3d8 | - | no-fear | animal |
| 20 | chimaera | `H` | +0 | 160 | 22 | 200 | 2 | butt 2d8, bite 2d10, bite fire 2d6 | innate 1/10: BR\_FIRE 53 | im-fire | bashes doors |
| 20 | dark dwarven lord | `h` | +10 | 144 | 48 | 500 | 2 | hit 3d8, hit 3d5 | spells 1/5: BLIND, CONF, DARKNESS, HASTE, BO\_COLD 54, BO\_FIRE 78 | - | spirit, opens doors, bashes doors, evil, friends: 80% 1d3 eastern dwarf (servant); 70% 1d2 stonefoot warrior (servant); 50% 1d1 blacklock mage (servant); 50% 1d1 ironfist priest (servant) |
| 20 | earth hound | `Z` | +0 | 68 | 36 | 200 | 3 | bite 1d8, bite 1d8, claw 3d3, claw 3d3 | innate 1/10: BR\_SHAR 11 | - | group-AI, bashes doors, animal, friends: 100% 2d7 Same |
| 20 | Mirkwood spider | `S` | +10 | 41 | 30 | 25 | 2 | bite 1d8, bite poison 1d6, bite poison 1d6 | innate 1/10: WEAVE | im-pois | weird-mind (ESP 1 in 10), group-AI, bashes doors, evil, animal, friends: 100% 2d7 Same |
| 20 | paladin | `p` | +0 | 90 | 48 | 55 | 1 | hit 4d6, hit 4d6 | spells 1/4: HEAL, SLOW | - | spirit, takes items, opens doors, bashes doors, friends: 50% 1d1 Easterling champion; 50% 1d1 rogue; 50% 1d2 Illusionist; 50% 1d2 Priest; 50% 1d1 Ranger |
| 20 | quylthulg | `Q` | +0 | 27 | 1 | 250 | 1 | - | spells 1/4: BLINK, S\_MONSTER | no-fear, no-conf, no-sleep, no-hold | INVISIBLE, empty-mind (no ESP), never-moves, no melee |
| 20 | ranger | `p` | +0 | 90 | 60 | 55 | 1 | hit 5d4, hit 5d4 | innate 1/4: ARROW 18; spells 1/5: BLINK, HASTE, BO\_COLD 54, S\_ANIMAL | - | spirit, takes items, opens doors, bashes doors, friends: 50% 1d1 Easterling champion; 50% 1d1 rogue; 50% 1d2 Illusionist; 50% 1d2 Priest; 50% 1d1 Paladin |
| 20 | sabre-tooth tiger | `f` | +10 | 150 | 60 | 120 | 2 | claw 1d10, claw 1d10, bite 1d10, bite 1d10 | - | - | bashes doors, animal |
| 20 | sasquatch | `Y` | +10 | 200 | 60 | 180 | 3 | claw 1d10, claw 1d10, bite 2d8 | - | im-cold | opens doors, bashes doors, animal |
| 20 | water hound | `Z` | +0 | 68 | 36 | 200 | 3 | bite acid 1d8, bite acid 1d8, claw 3d3, claw 3d3 | innate 1/10: BR\_ACID 22 | im-acid | group-AI, bashes doors, animal, friends: 100% 2d7 Same |
| 20 | werewolf | `C` | +0 | 230 | 36 | 150 | 1 | claw 2d6, claw 2d6, bite 2d10 | - | - | erratic 25%, takes items, opens doors, bashes doors, evil, animal |

## Levels 21-25

| DL | Monster | Sym | Spd | HP | AC | Exp | R | Blows | Spells | Defences | Flags |
|---:|---|:-:|---:|---:|---:|---:|---:|---|---|---|---|
| 21 | blue dragon bat | `b` | +20 | 10 | 39 | 54 | 1 | bite elec 1d3 | innate 1/4: BR\_ELEC 3 | im-elec | erratic 50%, bashes doors, animal |
| 21 | cold vortex | `v` | +0 | 45 | 36 | 100 | 1 | engulf cold 3d3 | innate 1/4: BR\_COLD 15 | im-cold, im-pois, hurt-fire, no-fear, no-conf, no-sleep, no-hold | empty-mind (no ESP), erratic 50%, powerful, bashes doors, nonliving |
| 21 | fire vortex | `v` | +0 | 45 | 36 | 100 | 1 | engulf fire 3d3 | innate 1/4: BR\_FIRE 15 | im-fire, im-pois, hurt-cold, no-fear, no-conf, no-sleep, no-hold | empty-mind (no ESP), erratic 50%, powerful, bashes doors, nonliving |
| 21 | mummified orc | `z` | +0 | 86 | 33 | 56 | 1 | hit 2d4, hit 2d4 | - | im-cold, im-pois, hurt-fire, no-fear, no-stun, no-conf, no-sleep | cold-blooded, empty-mind (no ESP), opens doors, bashes doors, evil, undead, orc |
| 21 | scroll mimic | `?` | +0 | 75 | 48 | 70 | 3 | hit poison 3d4, hit poison 3d4, hit 2d3, hit 2d3 | spells 1/5: BO\_FIRE 79, S\_MONSTER, BLIND, WOUND 70, CONF, SCARE | hurt-fire, no-fear, no-conf, no-sleep | cold-blooded, empty-mind (no ESP), never-moves, unaware (mimic) |
| 21 | water vortex | `v` | +0 | 45 | 36 | 100 | 1 | engulf acid 3d3 | innate 1/4: BR\_ACID 15 | im-acid, im-pois, im-water, no-fear, no-conf, no-sleep, no-hold | empty-mind (no ESP), erratic 50%, powerful, bashes doors, nonliving |
| 22 | giant yellow scorpion | `S` | +0 | 54 | 45 | 60 | 1 | claw 1d8, claw 1d8, sting poison 2d5 | - | im-pois | weird-mind (ESP 1 in 10), bashes doors, animal |
| 22 | iron golem | `g` | +0 | 520 | 120 | 160 | 2 | hit 1d12 | spells 1/7: SLOW | im-elec, im-fire, im-cold, im-pois, no-fear, no-conf, no-sleep | cold-blooded, empty-mind (no ESP), bashes doors, nonliving |
| 22 | killer stag beetle | `K` | +0 | 68 | 86 | 80 | 1 | claw 3d6, claw 3d6 | - | - | weird-mind (ESP 1 in 10), erratic 25%, bashes doors, animal |
| 22 | neekerbreeker | `I` | +10 | 5 | 21 | 4 | 4 | bite poison 2d2 | innate 1/12: SHRIEK | - | weird-mind (ESP 1 in 10), BREEDS, erratic 25%, erratic 50%, evil, animal |
| 22 | wyvern | `d` | +10 | 203 | 79 | 250 | 2 | bite 2d6, bite 2d6, sting poison 2d6 | - | im-pois | spirit, move-body, opens doors, bashes doors, evil, dragon, animal |
| 23 | black ooze | `j` | -20 | 27 | 7 | 7 | 1 | touch acid 2d6 | spells 1/11: DRAIN\_MANA | im-pois, hurt-cold, no-fear | empty-mind (no ESP), BREEDS, erratic 50%, stupid, takes items, kill-body, opens doors, bashes doors |
| 23 | brigand | `p` | +10 | 75 | 45 | 110 | 2 | hit 2d8, hit 2d8, hit eat\_gold 4d4, hit eat\_item 4d4 | innate 1/4: SHOT 15 | - | spirit, takes items, opens doors, bashes doors, evil, friends: 50% 1d1 Easterling champion; 50% 1d1 Ranger; 50% 1d1 Mage; 50% 1d1 Paladin |
| 23 | Easterling champion | `p` | +0 | 90 | 60 | 60 | 1 | hit 6d5, hit 6d5 | innate 1/6: BOLT 21 | - | spirit, takes items, opens doors, bashes doors, evil, friends: 50% 1d1 rogue; 50% 1d1 Ranger; 50% 1d1 Mage; 50% 1d1 Paladin |
| 23 | giant silver ant | `a` | +0 | 41 | 100 | 80 | 1 | bite acid 4d4, bite acid 4d4 | innate 1/6: SPIT 24 | - | weird-mind (ESP 1 in 10), erratic 25%, bashes doors, animal |
| 23 | phase spider | `S` | +10 | 27 | 30 | 60 | 2 | bite 1d8, bite poison 1d6, bite poison 1d6 | spells 1/5: BLINK, TELE\_TO, TELE\_SELF\_TO | im-pois | weird-mind (ESP 1 in 10), group-AI, bashes doors, animal, friends: 100% 2d7 Same |
| 23 | red dragon bat | `b` | +20 | 14 | 42 | 60 | 1 | bite fire 1d3 | innate 1/4: BR\_FIRE 4 | im-fire | erratic 50%, bashes doors, animal |
| 24 | 4-headed hydra | `M` | +10 | 350 | 105 | 450 | 2 | bite 2d6, bite 2d6, bite poison 2d6, bite fire 2d6 | spells 1/7: SCARE | im-fire, im-pois, hurt-cold | move-body, bashes doors, animal |
| 24 | banshee | `G` | +10 | 27 | 28 | 60 | 2 | wail terrify, touch exp\_20 | spells 1/15: DRAIN\_MANA, TPORT | im-cold, im-pois, no-stun, no-conf, no-sleep, no-hold | INVISIBLE, cold-blooded, pass-wall, erratic 50%, spirit, takes items, evil, undead |
| 24 | forest wight | `W` | +0 | 54 | 36 | 140 | 1 | hit 1d6, hit 1d6, touch exp\_20 | spells 1/10: DRAIN\_MANA, SCARE | im-cold, im-pois, hurt-light, no-stun, no-conf, no-sleep, no-hold | cold-blooded, erratic 25%, spirit, opens doors, bashes doors, evil, undead |
| 24 | mummified human | `z` | +0 | 105 | 51 | 70 | 1 | hit 2d4, hit 2d4 | - | im-cold, im-pois, hurt-fire, no-fear, no-stun, no-conf, no-sleep | cold-blooded, empty-mind (no ESP), opens doors, bashes doors, evil, undead |
| 24 | vampire bat | `b` | +10 | 50 | 60 | 150 | 2 | bite exp\_40 1d4, bite exp\_40 1d4 | - | im-cold, no-fear, no-stun, no-conf, no-sleep | cold-blooded, regenerates, erratic 50%, evil, undead, animal |
| 24 | werebear | `q` | +0 | 325 | 75 | 200 | 2 | claw 1d10, claw 1d10, bite 2d8, crush 2d6 | - | - | opens doors, bashes doors, evil, animal |
| 25 | carrion crawler | `c` | +0 | 130 | 60 | 60 | 2 | sting paralyze 2d6, sting paralyze 2d6 | - | im-pois | weird-mind (ESP 1 in 10), erratic 25%, bashes doors, animal, friends: 100% 2d4 Same |
| 25 | hill giant | `P` | +0 | 240 | 54 | 150 | 1 | hit 4d8, hit 4d8 | innate 1/10: BOULDER 48 | - | spirit, opens doors, bashes doors, evil, giant, friends: 60% 1d7 Wolf; 80% 1d3 Same |
| 25 | killer red beetle | `K` | +0 | 90 | 90 | 90 | 1 | bite lose\_str 4d4, bite 4d4 | - | - | weird-mind (ESP 1 in 10), erratic 25%, bashes doors, animal |
| 25 | necromancer | `p` | +0 | 164 | 55 | 330 | 2 | hit 2d4, hit 2d4 | spells 1/5: BLIND, WOUND 80, DARKNESS, SCARE, BO\_NETH 92, S\_UNDEAD | - | smart, spirit, opens doors, bashes doors, evil, friends: 80% 1d3 any skeleton |
| 25 | nruling | `u` | +10 | 50 | 16 | 75 | 2 | hit fire 1d8 | innate 1/3: WHIP 8; spells 1/2: BLINK, TELE\_SELF\_TO, BLIND, SLOW, WOUND 30, BO\_FIRE 75 | im-fire, no-fear | erratic 50%, spirit, opens doors, evil, demon, friends: 100% 2d7 Same |
| 25 | pukelman | `g` | +0 | 520 | 120 | 600 | 3 | hit 1d12, hit 3d6 | spells 1/4: CONF, SLOW, BO\_ACID 64 | im-elec, im-fire, im-cold, im-pois, hurt-rock, no-fear, no-conf, no-sleep | cold-blooded, empty-mind (no ESP), bashes doors, nonliving |
| 25 | stone troll | `T` | +0 | 127 | 60 | 85 | 1 | hit 1d6, hit 1d6, bite 3d4 | - | hurt-light, hurt-rock | spirit, opens doors, bashes doors, evil, troll, friends: 50% 1d5 Earth hound; 100% 3d3 Same |

## Levels 26-30

| DL | Monster | Sym | Spd | HP | AC | Exp | R | Blows | Spells | Defences | Flags |
|---:|---|:-:|---:|---:|---:|---:|---:|---|---|---|---|
| 26 | abyss worm mass | `w` | -10 | 35 | 25 | 6 | 5 | crawl exp\_10 1d5 | - | hurt-light, no-fear | INVISIBLE, cold-blooded, weird-mind (ESP 1 in 10), BREEDS, kill-wall, erratic 25%, erratic 50%, stupid, bashes doors, evil, animal |
| 26 | cave ogre | `O` | +0 | 150 | 49 | 80 | 2 | hit 3d8, hit 3d8 | - | - | spirit, opens doors, bashes doors, evil, giant, friends: 50% 2d4 Ogre; 50% 2d4 Black ogre; 100% 2d4 Same |
| 26 | disenchanter bat | `b` | +20 | 27 | 42 | 75 | 4 | hit disenchant | - | im-disen | erratic 50%, animal |
| 26 | displacer beast | `f` | +0 | 138 | 150 | 100 | 2 | bite 2d8, hit 1d10, hit 1d10, hit 1d10 | - | - | INVISIBLE, bashes doors, animal |
| 26 | ghoul | `z` | +0 | 83 | 36 | 95 | 2 | claw paralyze 1d4, claw paralyze 1d4, bite poison 1d6 | spells 1/9: HOLD, SCARE | im-cold, im-pois, hurt-light, hurt-fire, no-stun, no-conf, no-sleep | cold-blooded, opens doors, bashes doors, evil, undead, friends: 100% 4d4 Same |
| 26 | giant fire tick | `S` | +0 | 72 | 120 | 90 | 1 | bite fire 3d6 | innate 1/20: SPIT 18 | im-fire | weird-mind (ESP 1 in 10), erratic 25%, bashes doors, animal |
| 26 | giant firefly | `I` | +10 | 5 | 21 | 4 | 4 | bite blind 1d2 | - | - | weird-mind (ESP 1 in 10), BREEDS, erratic 25%, erratic 50%, animal |
| 26 | giant grey ant | `a` | +0 | 86 | 200 | 120 | 1 | bite 2d12, bite 2d12 | - | - | weird-mind (ESP 1 in 10), erratic 25%, kill-body, bashes doors, animal |
| 26 | wolf chieftain | `C` | +10 | 422 | 24 | 120 | 5 | claw 2d6, claw 2d6, bite 2d10, wail terrify | spells 1/8: DARKNESS | im-cold, no-fear | smart, opens doors, bashes doors, evil, animal, friends: 100% 2d4 Wolf (servant); 60% 2d4 White wolf (servant); 60% 2d4 Warg (servant); 60% 1d4 Werewolf (servant) |
| 27 | creeping adamantite coins | `$` | +10 | 260 | 60 | 60 | 3 | bite poison 3d4, touch poison 3d5, hit 1d12, hit 1d12 | - | im-acid, im-elec, im-pois, no-conf, no-sleep | cold-blooded, empty-mind (no ESP), unaware (mimic), mimic-invisible, bashes doors, metal |
| 27 | gorgimaera | `H` | +0 | 263 | 82 | 400 | 2 | butt 2d10, bite fire 2d10, gaze paralyze 2d6 | innate 1/8: BR\_FIRE 87 | im-fire | bashes doors |
| 27 | killer fire beetle | `K` | +0 | 99 | 64 | 95 | 1 | bite fire 3d4, spit fire 4d5 | innate 1/5: SPIT 22 | im-fire | weird-mind (ESP 1 in 10), bashes doors, animal |
| 27 | nexus hound | `Z` | +0 | 138 | 36 | 250 | 4 | bite 2d8, bite 2d8, claw 3d3, claw 3d3 | innate 1/5: BR\_NEXU 23 | im-nexus, no-conf, no-sleep | group-AI, bashes doors, animal, friends: 100% 2d7 Same |
| 27 | ogre shaman | `O` | +0 | 163 | 82 | 300 | 2 | hit 3d6, hit 3d6, hit 3d6 | spells 1/5: WOUND 90, HOLD, SCARE, TPORT, TRAPS, BO\_FIRE 81, S\_MONSTER | - | spirit, opens doors, bashes doors, evil, giant |
| 27 | shimmering mold | `m` | +0 | 144 | 36 | 140 | 1 | spore elec 5d4, spore elec 5d4 | - | im-elec, im-pois, hurt-fire, no-fear, no-conf, no-sleep | empty-mind (no ESP), never-moves, stupid |
| 27 | troll scavenger | `T` | +0 | 137 | 90 | 150 | 1 | claw poison 3d3, claw poison 3d3, bite 1d6 | - | im-pois, hurt-light | spirit, opens doors, bashes doors, evil, troll, friends: 30% 1d5 Forest Troll; 100% 3d3 Same |
| 27 | vampire | `V` | +0 | 163 | 67 | 175 | 1 | hit 1d6, hit 1d6, bite exp\_20 2d6, bite exp\_20 2d6 | spells 1/9: WOUND 90, DARKNESS, MIND\_BLAST 64, TELE\_TO, HOLD, SCARE | im-cold, im-pois, hurt-light, hurt-fire, no-stun, no-conf, no-sleep | cold-blooded, regenerates, spirit, opens doors, bashes doors, evil, undead |
| 27 | vibration hound | `Z` | +0 | 138 | 36 | 250 | 4 | bite 2d8, bite 2d8, claw 3d3, claw 3d3 | innate 1/5: BR\_SOUN 23 | no-stun, no-conf, no-sleep | group-AI, bashes doors, animal, friends: 100% 2d7 Same |
| 28 | 5-headed hydra | `M` | +10 | 450 | 120 | 650 | 2 | bite 2d6, bite 2d6, bite poison 2d6, bite fire 2d6, bite acid 2d6 | spells 1/5: SCARE | im-acid, im-fire, im-pois, hurt-cold | move-body, bashes doors, animal |
| 28 | basilisk | `R` | +10 | 310 | 108 | 300 | 3 | gaze paralyze, bite 2d12, bite 2d12, bite 2d12 | innate 1/8: BR\_POIS 103 | im-pois, hurt-cold, no-conf, no-sleep, no-hold | opens doors, bashes doors, animal |
| 28 | bat of Gorgoroth | `b` | +10 | 110 | 36 | 100 | 3 | bite poison 1d10, claw 1d4 | innate 1/12: BR\_DARK 18, BR\_POIS 36; spells 1/25: SCARE | im-pois | weird-mind (ESP 1 in 10), erratic 25%, move-body, bashes doors, animal, friends: 100% 2d7 Same |
| 28 | black knight | `p` | +10 | 165 | 105 | 240 | 1 | hit 5d5, hit 5d5, hit 5d5 | spells 1/8: BLIND, WOUND 90, DARKNESS, SCARE | - | spirit, opens doors, bashes doors, evil, friends: 50% 1d1 rogue; 50% 1d1 Ranger; 50% 1d1 Paladin; 50% 1d1 Mage; 50% 1d1 southron assassin |
| 28 | frost giant | `P` | +0 | 256 | 60 | 180 | 1 | hit cold 5d8, hit cold 5d8 | innate 1/10: BOULDER 60 | im-cold | spirit, opens doors, bashes doors, evil, giant, friends: 60% 1d7 Cold Hound; 80% 1d3 Same |
| 28 | mage | `p` | +0 | 68 | 48 | 150 | 1 | hit 2d5, hit 2d5 | spells 1/3: BLIND, CONF, TPORT, TELE\_AWAY, BO\_COLD 57, BO\_ELEC 41, BO\_FIRE 81, S\_MONSTER | - | smart, spirit, opens doors, bashes doors, friends: 100% 1d1 any golem |
| 28 | mind flayer | `h` | +0 | 132 | 72 | 200 | 1 | gaze lose\_int 2d6, gaze lose\_wis 2d6 | spells 1/8: BLIND, BRAIN\_SMASH 180, FORGET, HOLD, MIND\_BLAST 64, SCARE | no-conf, no-sleep | spirit, opens doors, bashes doors, evil |
| 28 | snow troll | `T` | +0 | 132 | 67 | 200 | 1 | claw 1d5, claw 1d5, bite cold 2d6, bite cold 2d6 | - | im-cold, hurt-light | spirit, opens doors, bashes doors, evil, troll, friends: 40% 1d5 Cold hound; 60% 2d7 White wolf; 100% 4d4 Same |
| 28 | spectator | `e` | +0 | 171 | 1 | 150 | 2 | gaze paralyze 1d4, gaze confuse 1d4, bite 1d8 | spells 1/6: WOUND 90, FORGET, HOLD, SLOW | no-fear, no-conf, no-sleep | spirit |
| 28 | spirit naga | `n` | +0 | 240 | 90 | 60 | 2 | crush 2d8, crush 2d8, bite 1d8, bite 1d8 | spells 1/4: BLIND, DARKNESS, HEAL, MIND\_BLAST 64 | im-pois, hurt-cold, no-conf, no-sleep | INVISIBLE, spirit, opens doors, bashes doors, evil |
| 29 | catoblepas | `q` | +0 | 165 | 66 | 400 | 2 | gaze terrify 2d4, gaze blind 2d4, butt 2d6, bite 2d12 | - | im-pois | bashes doors, animal |
| 29 | energy vortex | `v` | +0 | 65 | 36 | 140 | 1 | engulf elec 3d4 | innate 1/4: BR\_ELEC 21; spells 1/5: BE\_ELEC 113 | im-elec, im-pois, no-fear, no-conf, no-sleep, no-hold | empty-mind (no ESP), erratic 50%, powerful, bashes doors, nonliving |
| 29 | giant black scorpion | `S` | +10 | 189 | 60 | 425 | 3 | claw 1d11, claw 1d11, sting poison 3d4, sting blind 3d4 | - | im-pois | weird-mind (ESP 1 in 10), bashes doors, animal |
| 29 | green elf archer | `h` | +10 | 210 | 112 | 500 | 3 | hit 1d7, hit 1d7 | innate 1/3: ARROW 24; spells 1/12: CONF, HEAL, BLINK | im-pois, no-conf, no-sleep, no-hold | smart, spirit, opens doors, bashes doors |
| 29 | purple worm | `w` | +0 | 293 | 78 | 400 | 3 | hit 1d8, bite acid 2d8, sting poison 1d8 | - | im-acid, im-pois | bashes doors, animal |
| 29 | ring mimic | `=` | +10 | 180 | 72 | 200 | 3 | hit poison 3d4, hit poison 3d4, hit poison 3d4, hit poison 3d4 | spells 1/4: BO\_ACID 65, BO\_COLD 57, BO\_ELEC 41, BO\_FIRE 81, S\_MONSTER, BLIND, WOUND 90, CONF, SCARE | no-fear, no-conf, no-sleep | cold-blooded, empty-mind (no ESP), never-moves, unaware (mimic) |
| 29 | young blue dragon | `d` | +0 | 237 | 60 | 500 | 1 | claw 2d4, claw 2d4, bite 2d6 | innate 1/15: BR\_ELEC 79; spells 1/20: SCARE | im-elec | spirit, opens doors, bashes doors, evil, dragon |
| 29 | young green dragon | `d` | +0 | 237 | 60 | 500 | 1 | claw 2d4, claw 2d4, bite 2d6 | innate 1/15: BR\_POIS 79; spells 1/20: SCARE | im-pois | spirit, opens doors, bashes doors, evil, dragon |
| 29 | young white dragon | `d` | +0 | 237 | 60 | 500 | 1 | claw 2d4, claw 2d4, bite 2d6 | innate 1/15: BR\_COLD 79; spells 1/20: SCARE | im-cold | spirit, opens doors, bashes doors, evil, dragon |
| 30 | fire giant | `P` | +0 | 289 | 72 | 220 | 1 | hit fire 6d8, hit fire 6d8 | innate 1/8: BOULDER 60 | im-fire | spirit, opens doors, bashes doors, evil, giant, friends: 60% 1d7 Fire Hound; 80% 1d3 Same |
| 30 | ghast | `z` | +10 | 165 | 60 | 130 | 3 | claw paralyze 2d4, claw paralyze 2d4, bite lose\_con 2d4 | spells 1/7: HOLD, SCARE | im-cold, im-pois, hurt-light, hurt-fire, no-stun, no-conf, no-sleep | cold-blooded, opens doors, bashes doors, evil, undead, friends: 20% 1d3 Zombified kobold; 20% 1d3 Zombified orc; 20% 1d3 Zombified human; 40% 1d3 Mummified orc; 40% 1d3 Mummified human; 40% 1d3 Mummified troll; 100% 4d4 Ghoul |
| 30 | giant army ant | `a` | +10 | 67 | 60 | 90 | 3 | bite 2d12, bite 2d12 | - | - | weird-mind (ESP 1 in 10), erratic 25%, group-AI, kill-body, bashes doors, animal, friends: 100% 4d4 Same |
| 30 | giant blue ant | `a` | +0 | 36 | 75 | 80 | 2 | bite elec 5d5 | innate 1/6: SPIT 25 | im-elec | weird-mind (ESP 1 in 10), erratic 25%, group-AI, bashes doors, animal, friends: 100% 4d4 Same |
| 30 | grave wight | `W` | +0 | 66 | 60 | 325 | 1 | hit 1d7, hit 1d7, touch exp\_20 | spells 1/8: WOUND 100, DARKNESS, SCARE | im-cold, im-pois, hurt-light, no-stun, no-conf, no-sleep, no-hold | cold-blooded, erratic 25%, spirit, opens doors, bashes doors, evil, undead |
| 30 | killer slicer beetle | `K` | +0 | 138 | 92 | 250 | 2 | bite 7d8, bite 7d8 | - | - | weird-mind (ESP 1 in 10), bashes doors, animal |
| 30 | manticore | `H` | +10 | 220 | 22 | 300 | 2 | hit 3d4, hit 3d4, hit 3d4, hit 3d4 | innate 1/5: ARROW 24 | - | bashes doors, evil |
| 30 | mithril golem | `g` | +0 | 640 | 150 | 500 | 4 | hit 3d8, hit 3d8, hit 5d5, hit 5d5 | - | im-elec, im-fire, im-cold, im-pois, no-fear, no-conf, no-sleep | cold-blooded, empty-mind (no ESP), bashes doors, nonliving, metal |
| 30 | ogre chieftain | `O` | +10 | 240 | 66 | 600 | 5 | hit 3d8, hit 3d8, hit 3d8, hit 3d8 | - | - | spirit, move-body, opens doors, bashes doors, evil, giant, friends: 50% 2d4 Ogre (servant); 100% 2d4 Black ogre (servant); 100% 2d4 Cave ogre (servant); 50% 1d2 Ogre mage (servant); 50% 1d2 Ogre shaman (servant) |
| 30 | skeleton troll | `s` | +0 | 110 | 82 | 225 | 1 | hit 1d6, hit 1d6, bite 3d4 | - | im-cold, im-pois, no-fear, no-stun, no-conf, no-sleep | cold-blooded, empty-mind (no ESP), regenerates, opens doors, bashes doors, evil, undead, troll |

## Levels 31-35

| DL | Monster | Sym | Spd | HP | AC | Exp | R | Blows | Spells | Defences | Flags |
|---:|---|:-:|---:|---:|---:|---:|---:|---|---|---|---|
| 31 | death watch beetle | `K` | +0 | 163 | 72 | 300 | 3 | bite 5d6, bite 5d6, wail terrify 5d6 | - | - | weird-mind (ESP 1 in 10), bashes doors, animal |
| 31 | ghost | `G` | +10 | 59 | 36 | 350 | 1 | wail terrify, touch exp\_20, claw lose\_int 1d6, claw lose\_wis 1d6 | spells 1/15: BLIND, DRAIN\_MANA, HOLD | im-cold, im-pois, no-stun, no-conf, no-sleep, no-hold | INVISIBLE, cold-blooded, pass-wall, erratic 25%, spirit, takes items, evil, undead |
| 31 | huorn | `l` | +5 | 1000 | 80 | 1200 | 1 | crush 3d6, crush 3d6, crush 3d6, crush 3d6 | spells 1/9: HASTE, TELE\_TO, DARKNESS | im-water, hurt-fire, no-fear, no-stun, no-conf, no-sleep, no-slow | cold-blooded, weird-mind (ESP 1 in 10), regenerates, move-body |
| 31 | young black dragon | `d` | +0 | 264 | 72 | 700 | 1 | claw 2d6, claw 2d6, bite 2d8 | innate 1/15: BR\_ACID 88; spells 1/20: SCARE | im-acid | spirit, opens doors, bashes doors, evil, dragon |
| 31 | young gold dragon | `d` | +0 | 264 | 72 | 700 | 1 | claw 2d6, claw 2d6, bite 2d8 | innate 1/15: BR\_SOUN 44; spells 1/20: SCARE | no-stun | spirit, opens doors, bashes doors, evil, dragon |
| 31 | young red dragon | `d` | +0 | 264 | 72 | 700 | 1 | claw 2d6, claw 2d6, bite 2d8 | innate 1/15: BR\_FIRE 88; spells 1/20: SCARE | im-fire | spirit, opens doors, bashes doors, evil, dragon |
| 32 | doombat | `b` | +20 | 180 | 112 | 250 | 2 | bite fire 5d4, bite fire 5d4, bite fire 5d4 | - | im-fire | weird-mind (ESP 1 in 10), bashes doors, animal |
| 32 | memory moss | `,` | +0 | 2 | 1 | 150 | 3 | hit confuse 1d4, hit confuse 1d4 | spells 1/6: FORGET | hurt-fire, no-fear, no-conf, no-sleep | empty-mind (no ESP), never-moves, stupid |
| 32 | nexus quylthulg | `Q` | +0 | 65 | 1 | 300 | 1 | - | spells 1/1: BLINK, TELE\_AWAY | im-nexus, no-fear, no-conf, no-sleep, no-hold | INVISIBLE, empty-mind (no ESP), never-moves, no melee |
| 32 | ogre mage | `O` | +5 | 220 | 60 | 300 | 2 | hit 3d8, hit 3d8, hit 3d8, hit 3d8 | spells 1/4: HEAL, HOLD, TRAPS, BA\_COLD 58, S\_MONSTER | - | spirit, opens doors, bashes doors, evil, giant |
| 32 | serpent of the brownlands | `J` | +10 | 150 | 90 | 300 | 2 | spit blind 1d6, bite acid 2d12, bite poison 2d12, crush 4d4 | spells 1/8: HOLD, BLIND, CONF | hurt-cold | erratic 25%, kill-body, bashes doors, animal |
| 32 | southron assassin | `p` | +10 | 85 | 72 | 300 | 2 | hit poison 3d4, hit lose\_str 3d4, hit lose\_str 3d4 | - | no-conf, no-sleep | spirit, opens doors, bashes doors, evil, friends: 50% 1d1 Master thief; 50% 1d1 Mage; 50% 1d1 blackguard |
| 32 | young multi-hued dragon | `d` | +0 | 281 | 72 | 900 | 1 | claw 2d6, claw 2d6, bite 3d8 | innate 1/8: BR\_ACID 93, BR\_COLD 93, BR\_ELEC 93, BR\_FIRE 93, BR\_POIS 93; spells 1/50: SCARE | im-acid, im-elec, im-fire, im-cold, im-pois | spirit, opens doors, bashes doors, evil, dragon |
| 33 | balance drake | `d` | +0 | 528 | 84 | 1600 | 3 | claw 2d4, claw 2d4, bite 3d6 | innate 1/9: BR\_CHAO 88, BR\_DISE 88, BR\_SHAR 88, BR\_SOUN 88; spells 1/12: CONF, SCARE, SLOW | im-fire, im-cold, im-disen, no-stun, no-conf, no-sleep | spirit, opens doors, bashes doors, evil, dragon |
| 33 | barrow wight | `W` | +0 | 83 | 60 | 375 | 3 | hit 1d8, hit 1d8, touch exp\_40 | spells 1/8: WOUND 110, DARKNESS, HOLD, SCARE | im-cold, im-pois, hurt-light, no-stun, no-conf, no-sleep, no-hold | cold-blooded, spirit, opens doors, bashes doors, evil, undead, friends: 100% 4d4 Same |
| 33 | cave troll | `T` | +0 | 156 | 60 | 350 | 1 | hit 3d5, hit 3d5, bite 1d8, bite 1d8 | - | hurt-light | spirit, opens doors, bashes doors, evil, troll, friends: 30% 1d5 stone troll; 30% 1d5 troll scavenger; 100% 3d3 Same |
| 33 | chaos drake | `d` | +0 | 440 | 84 | 1400 | 3 | claw 2d4, claw 2d4, bite 3d6 | innate 1/12: BR\_CHAO 73, BR\_DISE 73; spells 1/13: CONF, SCARE, SLOW | im-fire, im-disen, no-conf, no-sleep | spirit, opens doors, bashes doors, evil, dragon |
| 33 | crystal drake | `d` | +0 | 396 | 84 | 1350 | 3 | claw 2d4, claw 2d4, bite 3d6 | innate 1/20: BR\_SHAR 66; spells 1/8: CONF, SCARE, SLOW | im-cold, no-conf, no-sleep | spirit, opens doors, bashes doors, evil, dragon |
| 33 | drúadan druid | `p` | +10 | 240 | 112 | 600 | 3 | punch 1d7, punch 1d7, kick 3d8 | spells 1/3: HASTE, CONF, DARKNESS, HEAL, SHAPECHANGE, S\_ANIMAL, S\_MONSTER, S\_SPIDER | im-pois, no-conf, no-sleep, no-hold | smart, spirit, bashes doors |
| 33 | ethereal drake | `d` | +0 | 352 | 84 | 1200 | 3 | claw 2d4, claw 2d4, bite 3d6 | innate 1/12: BR\_DARK 58, BR\_LIGHT 58; spells 1/13: CONF, SCARE, SLOW | no-conf, no-sleep | INVISIBLE, pass-wall, spirit, evil, dragon |
| 33 | fire elemental | `E` | +0 | 135 | 60 | 350 | 2 | hit fire 4d6, hit fire 4d6 | spells 1/6: BO\_FIRE 83 | im-acid, im-elec, im-fire, im-pois, hurt-cold, no-fear, no-conf, no-sleep | empty-mind (no ESP), erratic 25%, destroys items, kill-body, bashes doors, nonliving |
| 33 | law drake | `d` | +0 | 440 | 84 | 1400 | 3 | claw 2d4, claw 2d4, bite 3d6 | innate 1/12: BR\_SHAR 73, BR\_SOUN 73; spells 1/13: CONF, SCARE, SLOW | im-cold, no-stun, no-conf, no-sleep | spirit, opens doors, bashes doors, evil, dragon |
| 33 | mystic | `p` | +10 | 308 | 60 | 500 | 3 | kick 10d2, kick 10d2, kick 10d2, kick 10d2 | spells 1/6: HEAL, S\_ANIMAL, S\_SPIDER | im-acid, im-pois, no-conf, no-sleep, no-hold | INVISIBLE, spirit, opens doors, bashes doors, friends: 50% 2d4 any spider |
| 33 | shadow drake | `d` | +0 | 264 | 84 | 1100 | 3 | claw 2d4, claw 2d4, bite exp\_10 3d6 | innate 1/25: BR\_NETH 44; spells 1/10: CONF, DARKNESS, SCARE, SLOW | im-nether, no-conf, no-sleep | INVISIBLE, pass-wall, spirit, evil, dragon |
| 33 | skeleton etten | `s` | +0 | 396 | 75 | 325 | 1 | hit 1d9, hit 1d9, bite 1d5, bite 1d5 | - | im-cold, im-pois, no-fear, no-stun, no-conf, no-sleep | cold-blooded, empty-mind (no ESP), regenerates, opens doors, bashes doors, evil, undead, troll |
| 33 | stone giant | `P` | +0 | 333 | 90 | 250 | 1 | hit 7d8, hit 7d8 | innate 1/5: BOULDER 60 | - | spirit, takes items, opens doors, bashes doors, evil, giant, friends: 60% 1d7 Earth hound; 80% 1d3 Same |
| 33 | water elemental | `E` | +0 | 113 | 48 | 325 | 2 | hit 1d10, hit 1d10, hit 1d10 | spells 1/6: BO\_COLD 59 | im-acid, im-fire, im-cold, im-pois, im-water, no-fear, no-conf, no-sleep | cold-blooded, empty-mind (no ESP), erratic 25%, destroys items, kill-body, bashes doors, nonliving |
| 34 | air elemental | `E` | +10 | 90 | 60 | 390 | 2 | hit 1d10, hit confuse 1d4, hit 1d10 | spells 1/8: BO\_ELEC 43 | im-acid, im-elec, im-cold, im-pois, no-fear, no-conf, no-sleep | cold-blooded, empty-mind (no ESP), erratic 25%, destroys items, kill-body, bashes doors, nonliving |
| 34 | earth elemental | `E` | -10 | 165 | 90 | 375 | 2 | hit 4d6, hit 4d6, hit 4d6 | spells 1/8: BO\_ACID 67 | im-elec, im-fire, im-cold, im-pois, hurt-rock, no-fear, no-conf, no-sleep | cold-blooded, empty-mind (no ESP), pass-wall, destroys items, kill-body, nonliving |
| 34 | Eog golem | `g` | -10 | 1050 | 187 | 1200 | 4 | hit 8d6, hit 8d6, hit 6d6, hit 6d6 | - | im-elec, im-fire, im-cold, im-pois, no-fear, no-conf, no-sleep | cold-blooded, empty-mind (no ESP), bashes doors, nonliving |
| 34 | lich | `L` | +0 | 264 | 72 | 1000 | 2 | touch exp\_40, touch drain\_charges, touch lose\_dex 2d8, touch lose\_dex 2d8 | spells 1/4: BLIND, BLINK, WOUND 110, DRAIN\_MANA, HOLD, BRAIN\_SMASH 180, SCARE, SLOW, TELE\_AWAY, TELE\_TO | im-cold, im-pois, hurt-light, no-stun, no-conf, no-sleep, no-hold | cold-blooded, smart, spirit, opens doors, bashes doors, evil, undead, friends: 70% 1d3 any skeleton |
| 34 | Maia | `A` | +0 | 528 | 81 | 1000 | 4 | hit 3d5, hit 3d5, hit 3d5, hit 3d5 | spells 1/3: BLIND, CONF, HASTE, HEAL, SCARE | im-fire, im-cold, im-pois, no-conf, no-sleep, no-hold | smart, spirit, powerful, takes items, opens doors, bashes doors |
| 34 | master thief | `p` | +20 | 99 | 45 | 350 | 2 | hit 2d8, hit 3d4, hit eat\_gold 4d4, hit eat\_item 4d5 | innate 1/10: SHOT 25; spells 1/6: BLINK, TRAPS | - | spirit, takes items, opens doors, bashes doors, evil, friends: 50% 1d1 blackguard; 50% 1d1 Mage; 50% 1d1 southron assassin |
| 34 | shade | `G` | +10 | 147 | 36 | 350 | 3 | wail terrify, touch exp\_40, claw lose\_int 1d10 | spells 1/15: BLIND, DRAIN\_MANA, HOLD | im-cold, im-pois, no-stun, no-conf, no-sleep, no-hold | INVISIBLE, cold-blooded, pass-wall, erratic 25%, spirit, takes items, evil, undead |
| 34 | troll blackguard | `T` | +0 | 340 | 60 | 400 | 2 | claw poison 2d5, claw poison 2d5, bite 2d8, bite 2d8 | spells 1/6: HASTE, SCARE, DARKNESS | - | spirit, opens doors, bashes doors, evil, troll, friends: 100% 3d4 any troll |
| 34 | troll priest | `T` | +0 | 340 | 75 | 120 | 1 | hit 2d5, hit 2d5, bite 3d5, bite 3d5 | spells 1/5: BLINK, WOUND 110, DARKNESS, SCARE | hurt-light, no-conf, no-sleep | spirit, opens doors, bashes doors, evil, troll |
| 35 | chest mimic | `~` | +0 | 308 | 48 | 250 | 4 | bite poison 4d4, bite poison 4d4, butt confuse 4d4, spit blind 4d4 | spells 1/5: BA\_POIS 80, S\_MONSTER, BLIND, WOUND 110, CONF, SCARE | no-fear, no-conf, no-sleep | cold-blooded, empty-mind (no ESP), never-moves, unaware (mimic) |
| 35 | giant fire ant | `a` | +0 | 176 | 58 | 350 | 1 | bite fire 3d12, bite fire 3d12 | innate 1/6: SPIT 54 | im-fire | weird-mind (ESP 1 in 10), group-AI, kill-body, bashes doors, animal, friends: 100% 4d4 Same |
| 35 | gravity hound | `Z` | +0 | 193 | 36 | 500 | 4 | claw 2d8, claw 2d8, bite 2d8, bite 2d8 | innate 1/5: BR\_GRAV 64 | no-conf, no-sleep | group-AI, bashes doors, animal, friends: 100% 2d7 Same |
| 35 | hill troll | `T` | +0 | 316 | 60 | 420 | 1 | hit 2d9, hit 2d9, bite 2d4, bite 2d4 | - | - | spirit, opens doors, bashes doors, evil, troll, friends: 100% 4d4 Same |
| 35 | impact hound | `Z` | +0 | 193 | 36 | 500 | 4 | claw 2d8, claw 2d8, bite 2d8, bite 2d8 | innate 1/5: BR\_WALL 32 | no-stun, no-conf, no-sleep | group-AI, bashes doors, animal, friends: 100% 2d7 Same |
| 35 | inertia hound | `Z` | +0 | 193 | 36 | 500 | 4 | claw 2d8, claw 2d8, bite 2d8, bite 2d8 | innate 1/5: BR\_INER 32 | no-conf, no-sleep, no-slow | group-AI, bashes doors, animal, friends: 100% 2d7 Same |
| 35 | invisible stalker | `E` | +20 | 124 | 69 | 300 | 3 | hit 1d6, hit 1d6, hit 1d6 | - | im-elec, im-pois, no-fear, no-conf, no-sleep | INVISIBLE, cold-blooded, empty-mind (no ESP), erratic 50%, opens doors, bashes doors, evil, nonliving |
| 35 | mûmak | `q` | +0 | 495 | 82 | 2100 | 2 | butt 4d6, butt 4d6, crush 4d4 | - | - | bashes doors, animal, friends: 100% 4d4 Same |
| 35 | silent watcher | `g` | +0 | 1040 | 96 | 800 | 3 | gaze terrify, gaze paralyze, gaze lose\_str, gaze hallu | innate 1/10: SHRIEK; spells 1/6: CONF, DRAIN\_MANA, HOLD, MIND\_BLAST 64, S\_MONSTER, S\_MONSTERS | im-elec, im-fire, im-cold, im-pois, hurt-light, hurt-rock, no-fear, no-conf, no-sleep | cold-blooded, empty-mind (no ESP), never-moves, evil, nonliving |
| 35 | wereworm | `w` | -10 | 200 | 105 | 300 | 3 | gaze exp\_20, crawl acid 2d4, bite 1d10, bite poison 1d6 | - | im-acid | bashes doors, animal |

## Levels 36-40

| DL | Monster | Sym | Spd | HP | AC | Exp | R | Blows | Spells | Defences | Flags |
|---:|---|:-:|---:|---:|---:|---:|---:|---|---|---|---|
| 36 | bodak | `u` | +0 | 193 | 81 | 750 | 2 | hit fire 4d6, hit fire 4d6, gaze exp\_20 | spells 1/4: BA\_FIRE 136, BO\_FIRE 84, S\_DEMON | im-fire, im-pois, no-conf, no-sleep | spirit, takes items, opens doors, bashes doors, evil, demon |
| 36 | cloud giant | `P` | +0 | 368 | 90 | 500 | 1 | hit elec 8d8, hit elec 8d8 | innate 1/8: BOULDER 72 | im-elec | spirit, takes items, opens doors, bashes doors, evil, giant, friends: 60% 1d7 Energy hound; 80% 1d3 Same |
| 36 | colbran | `g` | +10 | 520 | 120 | 900 | 2 | hit elec 3d8, hit elec 3d8 | spells 1/3: BO\_ELEC 38, BE\_ELEC 95, TELE\_SELF\_TO | im-elec, im-pois, no-fear, no-conf, no-sleep | cold-blooded, empty-mind (no ESP), bashes doors, nonliving |
| 36 | colossus | `g` | -10 | 2640 | 180 | 850 | 4 | hit 10d10, hit 10d10, hit 6d6, hit 6d6 | - | im-elec, im-fire, im-cold, im-pois, no-fear, no-conf, no-sleep | cold-blooded, empty-mind (no ESP), bashes doors, nonliving |
| 36 | demonologist | `p` | +10 | 246 | 75 | 700 | 2 | hit 2d6, hit 2d6, hit 2d6 | spells 1/2: HOLD, TPORT, BO\_FIRE 84, S\_DEMON | - | smart, spirit, opens doors, bashes doors, evil, friends: 100% 1d1 Quasit (servant); 50% 1d3 any minor demon (servant) |
| 36 | dúnadan of Angmar | `p` | +10 | 246 | 75 | 630 | 2 | hit 2d6, hit 2d6 | spells 1/3: BLIND, WOUND 120, DARKNESS, HOLD, SCARE, TPORT, BO\_NETH 109, S\_UNDEAD | - | smart, spirit, opens doors, bashes doors, evil, friends: 80% 1d3 any skeleton |
| 36 | gauth | `e` | +0 | 264 | 75 | 600 | 2 | gaze disenchant 5d2, gaze disenchant 5d2, gaze drain\_charges 5d2, gaze drain\_charges 5d2 | spells 1/5: WOUND 120, DRAIN\_MANA, HOLD, TELE\_AWAY, BA\_COLD 64, BO\_ELEC 44 | - | spirit, bashes doors, evil |
| 36 | master vampire | `V` | +0 | 299 | 72 | 750 | 1 | hit 2d6, hit 2d6, bite exp\_40 3d6, bite exp\_40 3d6 | spells 1/6: WOUND 120, CONF, DARKNESS, MIND\_BLAST 64, TELE\_TO, BO\_NETH 109, HOLD, SCARE | im-cold, im-pois, hurt-light, hurt-fire, no-stun, no-conf, no-sleep | cold-blooded, regenerates, spirit, opens doors, bashes doors, evil, undead |
| 36 | olog | `T` | +5 | 440 | 60 | 450 | 1 | claw 2d9, claw 2d9, bite 2d3, bite 2d3 | - | - | smart, spirit, opens doors, bashes doors, evil, troll, friends: 100% 4d4 Olog |
| 36 | ooze elemental | `E` | +0 | 72 | 120 | 300 | 3 | touch acid 1d10, touch acid 1d10, touch acid 1d10 | spells 1/5: BA\_ACID 123, BO\_ACID 68 | im-acid, im-fire, im-cold, im-pois, no-fear, no-conf, no-sleep | cold-blooded, empty-mind (no ESP), powerful, destroys items, kill-body, bashes doors, nonliving |
| 36 | smoke elemental | `E` | +10 | 83 | 120 | 375 | 3 | engulf 2d6, engulf 2d6 | spells 1/5: DARKNESS, BO\_FIRE 84 | im-elec, im-fire, im-cold, im-pois, no-fear, no-conf, no-sleep | empty-mind (no ESP), destroys items, kill-body, bashes doors, nonliving |
| 36 | trapper | `x` | +10 | 528 | 90 | 580 | 3 | hit 3d8, hit 3d8, hit paralyze 15d1, hit paralyze 15d1 | - | no-fear, no-conf, no-sleep | INVISIBLE, cold-blooded, empty-mind (no ESP), never-moves, unaware (mimic) |
| 36 | xorn | `X` | +0 | 140 | 96 | 650 | 2 | hit 1d6, hit 1d6, hit 1d6, hit 1d6 | - | im-elec, im-fire, im-cold, im-pois, hurt-rock, no-conf, no-sleep | cold-blooded, empty-mind (no ESP), pass-wall, destroys items |
| 37 | emperor wight | `W` | +0 | 334 | 48 | 1600 | 2 | hit 1d12, hit 1d12, touch exp\_80, touch exp\_80 | spells 1/6: WOUND 120, HOLD, SCARE, BO\_NETH 110 | im-cold, im-pois, hurt-light, no-stun, no-conf, no-sleep, no-hold | cold-blooded, spirit, opens doors, bashes doors, evil, undead, friends: 50% 1d3 any wraith |
| 37 | ice elemental | `E` | +0 | 193 | 90 | 650 | 3 | touch cold 4d3, hit 4d6, touch cold 4d3 | spells 1/5: BA\_COLD 65, BO\_ICE 73 | im-acid, im-elec, im-cold, im-pois, hurt-fire, no-fear, no-conf, no-sleep | cold-blooded, empty-mind (no ESP), powerful, destroys items, kill-body, bashes doors, nonliving |
| 37 | killer iridescent beetle | `K` | +0 | 330 | 90 | 850 | 2 | claw elec 2d12, claw elec 2d12, gaze paralyze | innate 1/6: SPIT 36 | im-elec | weird-mind (ESP 1 in 10), bashes doors, animal |
| 37 | magma elemental | `E` | +0 | 193 | 105 | 950 | 3 | hit fire 3d7, hit 4d6, hit fire 3d7 | spells 1/7: BA\_FIRE 139, BO\_PLAS 103 | im-acid, im-elec, im-fire, im-pois, hurt-cold, no-fear, no-conf, no-sleep | empty-mind (no ESP), pass-wall, powerful, destroys items, kill-body, nonliving |
| 37 | mountain troll | `T` | +0 | 660 | 120 | 800 | 3 | punch 8d4, punch 8d4, kick 5d4, kick 5d4 | - | no-conf, no-sleep | spirit, takes items, opens doors, bashes doors, evil, troll |
| 37 | mummified troll | `z` | +0 | 207 | 75 | 420 | 1 | hit 2d6, hit 2d6 | - | im-cold, im-pois, hurt-fire, no-fear, no-stun, no-conf, no-sleep | cold-blooded, empty-mind (no ESP), opens doors, bashes doors, evil, undead, troll |
| 37 | nexus vortex | `v` | +10 | 176 | 48 | 800 | 1 | engulf 5d5 | innate 1/4: BR\_NEXU 29 | im-pois, im-nexus, no-fear, no-conf, no-sleep, no-hold | empty-mind (no ESP), erratic 25%, erratic 50%, powerful, bashes doors, nonliving |
| 37 | plasma vortex | `v` | +10 | 176 | 48 | 800 | 1 | engulf fire 4d8, engulf elec 4d8 | innate 1/4: BR\_PLAS 29 | im-elec, im-fire, im-pois, im-plasma, no-fear, no-conf, no-sleep, no-hold | empty-mind (no ESP), erratic 25%, erratic 50%, powerful, bashes doors, nonliving |
| 37 | shardstorm | `v` | +10 | 176 | 14 | 800 | 1 | engulf 6d6 | innate 1/4: BR\_SHAR 29 | im-pois, no-fear, no-conf, no-sleep, no-hold | empty-mind (no ESP), erratic 50%, powerful, bashes doors, nonliving |
| 37 | spectre | `G` | +10 | 167 | 36 | 380 | 3 | wail terrify, touch exp\_40, claw lose\_wis 5d5 | spells 1/15: BLIND, DRAIN\_MANA, HOLD | im-cold, im-pois, no-stun, no-conf, no-sleep, no-hold | INVISIBLE, cold-blooded, pass-wall, erratic 25%, spirit, takes items, evil, undead |
| 38 | abyss spider | `S` | +10 | 264 | 54 | 250 | 3 | claw 2d8, sting poison 2d6, sting poison 2d6, bite confuse 1d3 | innate 1/6: WEAVE, BR\_POIS 88, BR\_DARK 44; spells 1/5: CONF, SLOW, BLIND | im-pois, hurt-light | weird-mind (ESP 1 in 10), bashes doors, evil, animal |
| 38 | death knight | `p` | +10 | 528 | 120 | 1000 | 1 | hit 6d6, hit 5d5, hit 5d5 | spells 1/5: BLIND, WOUND 120, SCARE, BO\_NETH 112, S\_MONSTERS | im-cold | cold-blooded, smart, spirit, opens doors, bashes doors, evil, friends: 50% 1d1 Imp |
| 38 | etten | `T` | +0 | 1320 | 120 | 1000 | 3 | hit 4d6, hit 4d6, hit 4d6, hit 4d6 | - | no-conf, no-sleep | spirit, opens doors, bashes doors, evil, troll |
| 38 | mummified chieftain | `z` | +5 | 299 | 102 | 800 | 3 | claw lose\_con 3d6, claw lose\_con 3d6, gaze exp\_40 3d4, gaze terrify 3d4 | innate 1/9: SHRIEK; spells 1/3: BLIND, WOUND 120, DARKNESS, DRAIN\_MANA, FORGET, HASTE, HEAL, MIND\_BLAST 64, SCARE, BA\_COLD 67, BA\_POIS 88, BO\_NETH 112, S\_UNDEAD | im-acid, im-elec, im-cold, im-pois, hurt-fire, no-fear, no-stun, no-conf, no-sleep | cold-blooded, smart, takes items, move-body, opens doors, bashes doors, evil, undead, friends: 100% 2d3 Mummified orc (servant); 100% 2d3 Mummified human (servant); 100% 2d3 Mummified troll (servant) |
| 38 | shimmering vortex | `v` | +30 | 176 | 48 | 800 | 4 | engulf blind 4d4, engulf blind 4d4 | innate 1/4: SHRIEK, BR\_LIGHT 29 | im-pois, no-fear, no-conf, no-sleep, no-hold | empty-mind (no ESP), erratic 25%, erratic 50%, powerful, bashes doors, nonliving |
| 38 | time vortex | `v` | +20 | 176 | 48 | 800 | 4 | engulf exp\_20 5d5 | innate 1/4: BR\_TIME 58 | im-pois, no-fear, no-conf, no-sleep, no-hold | empty-mind (no ESP), erratic 25%, erratic 50%, powerful, bashes doors, nonliving |
| 38 | white wraith | `W` | +10 | 450 | 48 | 355 | 1 | hit 1d9, hit 1d9, touch exp\_20, touch exp\_20 | spells 1/8: WOUND 120, DARKNESS, SCARE | im-cold, im-pois, hurt-light, no-stun, no-conf, no-sleep, no-hold | cold-blooded, spirit, opens doors, bashes doors, evil, undead |
| 38 | will o' the wisp | `E` | +20 | 176 | 180 | 500 | 4 | hit 1d9, hit 1d9, hit hallu 1d9 | spells 1/2: BLINK, WOUND 120, CONF, TPORT | im-acid, im-elec, im-fire, im-cold, im-pois, no-fear, no-conf, no-sleep | INVISIBLE, empty-mind (no ESP), pass-wall, erratic 50%, smart, nonliving |
| 39 | 6-headed hydra | `M` | +10 | 550 | 135 | 2000 | 2 | bite 2d6, bite 2d6, bite poison 2d6, bite fire 2d6, bite acid 2d6, bite blind 1d2 | innate 1/10: BR\_POIS 183; spells 1/8: SCARE | im-acid, im-fire, im-pois, hurt-cold | move-body, bashes doors, animal |
| 39 | mature blue dragon | `d` | +5 | 700 | 84 | 1400 | 1 | claw 3d8, claw 3d8, bite 4d8 | innate 1/12: BR\_ELEC 233; spells 1/20: SCARE | im-elec, no-conf, no-sleep | spirit, bashes doors, evil, dragon, friends: 50% 1d4 Baby blue dragon; 50% 1d2 Young blue dragon |
| 39 | mature green dragon | `d` | +5 | 700 | 84 | 1400 | 1 | claw 3d8, claw 3d8, bite 4d8 | innate 1/12: BR\_POIS 233; spells 1/20: SCARE | im-pois, no-conf, no-sleep | spirit, bashes doors, evil, dragon, friends: 50% 1d4 Baby green dragon; 50% 1d2 Young green dragon |
| 39 | mature white dragon | `d` | +5 | 700 | 84 | 1400 | 1 | claw 3d8, claw 3d8, bite 4d8 | innate 1/12: BR\_COLD 233; spells 1/20: SCARE | im-cold, no-conf, no-sleep | spirit, bashes doors, evil, dragon, friends: 50% 1d4 Baby white dragon; 50% 1d2 Young white dragon |
| 39 | night mare | `q` | +10 | 1320 | 102 | 2900 | 3 | bite exp\_80 2d6, hit 3d8, hit 3d8, hit confuse 6d6 | - | im-cold, im-pois, no-stun, no-conf, no-sleep | cold-blooded, opens doors, bashes doors, evil, undead |
| 39 | southron archer | `p` | +10 | 209 | 114 | 700 | 4 | hit poison 3d4, hit lose\_str 3d4, hit lose\_str 3d4, hit poison 3d4 | innate 1/2: ARROW 30 | no-conf, no-sleep | spirit, opens doors, bashes doors, evil, friends: 80% 1d4 southron assassin |
| 40 | 7-headed hydra | `M` | +10 | 650 | 114 | 3000 | 2 | bite 2d6, bite 2d6, bite poison 2d6, bite fire 2d6, bite acid 2d6, bite blind 2d6, bite confuse 2d6 | innate 1/8: BR\_POIS 216, BR\_FIRE 216; spells 1/6: SCARE | im-acid, im-fire, im-pois, hurt-cold, no-conf | move-body, bashes doors, animal |
| 40 | beholder | `e` | +10 | 1400 | 96 | 6000 | 3 | gaze exp\_20 2d6, gaze drain\_charges 2d6, gaze lose\_int 2d6, bite 6d6 | spells 1/2: BLIND, CONF, DARKNESS, DRAIN\_MANA, FORGET, MIND\_BLAST 64, SCARE, SLOW, BO\_ACID 69, BO\_COLD 61, BO\_ELEC 45, BO\_FIRE 85 | no-conf, no-sleep | spirit, bashes doors, evil |
| 40 | death quasit | `u` | +20 | 387 | 96 | 1000 | 3 | bite lose\_dex 3d6, claw 3d3, claw 3d3 | spells 1/10: BLIND, WOUND 130, CONF, SCARE, S\_DEMON | im-fire, im-pois, no-fear, no-conf, no-sleep, no-hold | INVISIBLE, pass-wall, smart, spirit, evil, demon |
| 40 | disenchanter worm mass | `w` | -10 | 45 | 6 | 30 | 3 | crawl disenchant 1d4 | - | im-disen, hurt-light, no-fear | weird-mind (ESP 1 in 10), BREEDS, erratic 50%, stupid, bashes doors, animal |
| 40 | enchantress | `p` | +20 | 457 | 72 | 2100 | 4 | hit 2d6, hit 2d6, hit 2d8 | spells 1/2: BLIND, SHAPECHANGE, S\_DRAGON | no-conf, no-sleep, no-hold | spirit, opens doors, bashes doors, evil, friends: 30% 1d1 any dragon (servant); 60% 1d1 any dragon (servant) |
| 40 | giant roc | `B` | +0 | 560 | 84 | 1000 | 3 | crush 8d12, crush 8d12, hit elec 12d12 | - | im-elec | bashes doors, animal |
| 40 | minotaur | `H` | +20 | 550 | 30 | 2100 | 2 | butt 4d6, butt 4d6, butt 2d6, butt 2d6 | - | - | bashes doors, evil |
| 40 | patriarch | `p` | +10 | 457 | 90 | 1800 | 2 | hit 3d4, hit 3d4, hit 3d5 | spells 1/2: BLIND, HEAL, HEAL\_KIN, MIND\_BLAST 64, BA\_LIGHT 70, BA\_ELEC 68, BA\_HOLY 70, S\_KIN | no-conf, no-sleep, no-hold | smart, spirit, opens doors, bashes doors, friends: 30% 1d1 Master thief; 30% 1d1 Ranger chieftain; 30% 1d1 Sorcerer; 30% 1d1 Berserker |
| 40 | phantom | `G` | +10 | 260 | 36 | 400 | 3 | touch exp\_80, touch exp\_40, claw lose\_int 2d8, claw lose\_wis 2d8 | spells 1/5: FORGET | im-cold, im-pois, no-stun, no-conf, no-sleep, no-hold | INVISIBLE, cold-blooded, pass-wall, spirit, evil, undead |
| 40 | sorcerer | `p` | +20 | 457 | 72 | 2150 | 2 | hit 2d8, hit 2d8, hit 2d8 | spells 1/2: BLIND, BLINK, WOUND 130, CONF, TELE\_TO, TRAPS, BA\_COLD 70, BA\_FIRE 150, BO\_ACID 69, S\_DRAGON, S\_MONSTER, S\_UNDEAD | no-conf, no-sleep, no-hold | spirit, opens doors, bashes doors, evil, friends: 30% 1d1 Master thief; 30% 1d1 Ranger chieftain; 30% 1d1 Patriarch; 30% 1d1 Berserker |
| 40 | spirit troll | `G` | +0 | 880 | 108 | 900 | 3 | hit 3d6, hit 3d6, hit 3d6 | - | im-elec, im-cold, im-pois, no-stun, no-conf, no-sleep, no-hold | INVISIBLE, cold-blooded, regenerates, pass-wall, spirit, evil, undead, troll |
| 40 | troll chieftain | `T` | +10 | 792 | 60 | 3000 | 5 | hit 2d12, hit 2d12, bite 3d6, bite 3d6 | - | no-fear | smart, spirit, takes items, move-body, opens doors, bashes doors, evil, troll, friends: 60% 2d3 forest troll (servant); 60% 2d3 stone troll (servant); 60% 2d3 snow troll (servant); 60% 2d3 hill troll (servant); 60% 2d3 cave troll (servant); 60% 2d3 olog (servant) |
| 40 | vrock | `U` | +0 | 352 | 75 | 2000 | 2 | hit 3d4, crush 8d12, crush 8d12 | spells 1/8: BLIND, CONF | im-fire, no-conf, no-sleep, no-hold | spirit, opens doors, bashes doors, evil, demon, friends: 100% 4d4 Same |
| 40 | xaren | `X` | +10 | 280 | 96 | 1200 | 1 | hit 3d4, hit 3d4, hit 3d4, hit 3d4 | - | im-elec, im-fire, im-cold, im-pois, hurt-rock, no-conf, no-sleep | cold-blooded, empty-mind (no ESP), pass-wall, destroys items |

## Levels 41-45

| DL | Monster | Sym | Spd | HP | AC | Exp | R | Blows | Spells | Defences | Flags |
|---:|---|:-:|---:|---:|---:|---:|---:|---|---|---|---|
| 41 | master lich | `L` | +10 | 1584 | 96 | 10000 | 2 | touch exp\_80, touch drain\_charges, touch lose\_dex 2d12, touch lose\_dex 2d12 | spells 1/3: BLIND, BLINK, WOUND 130, CONF, BRAIN\_SMASH 180, DRAIN\_MANA, HOLD, SCARE, TELE\_TO, S\_UNDEAD | im-cold, im-pois, no-stun, no-conf, no-sleep, no-hold | cold-blooded, smart, spirit, opens doors, bashes doors, evil, undead, friends: 80% 1d3 any zombie (servant); 80% 1d3 any skeleton (servant) |
| 41 | mature black dragon | `d` | +5 | 740 | 96 | 1600 | 1 | claw 3d10, claw 3d10, bite 4d10 | innate 1/12: BR\_ACID 246; spells 1/20: SCARE | im-acid, no-conf, no-sleep | spirit, bashes doors, evil, dragon, friends: 50% 1d4 Baby black dragon; 50% 1d2 Young black dragon |
| 41 | mature gold dragon | `d` | +5 | 740 | 96 | 1600 | 1 | claw 3d10, claw 3d10, bite 4d10 | innate 1/12: BR\_SOUN 123; spells 1/20: CONF, SCARE | no-stun, no-conf, no-sleep | spirit, bashes doors, evil, dragon, friends: 50% 1d4 Baby gold dragon; 50% 1d2 Young gold dragon |
| 41 | mature red dragon | `d` | +5 | 740 | 96 | 1600 | 1 | claw 3d10, claw 3d10, bite 4d10 | innate 1/12: BR\_FIRE 246; spells 1/20: CONF, SCARE | im-fire, no-conf, no-sleep | spirit, bashes doors, evil, dragon, friends: 50% 1d4 Baby red dragon; 50% 1d2 Young red dragon |
| 41 | ranger chieftain | `p` | +10 | 880 | 72 | 1600 | 2 | hit 5d5, hit 5d5, hit 5d5 | innate 1/2: ARROW 36; spells 1/5: BLINK, HASTE, HEAL, BA\_COLD 71, S\_ANIMAL | im-elec, im-fire, im-cold, im-pois, no-conf, no-sleep, no-hold | INVISIBLE, spirit, takes items, opens doors, bashes doors, friends: 30% 1d1 Master thief; 30% 1d1 Sorcerer; 30% 1d1 Patriarch; 30% 1d1 Berserker |
| 41 | stiffbeard sorcerer | `h` | +20 | 700 | 84 | 3000 | 2 | hit 2d8, hit 2d8, hit 2d8 | spells 1/2: BLIND, BLINK, WOUND 130, CONF, DARKNESS, HEAL, TELE\_TO, BA\_COLD 71, BA\_FIRE 153, BO\_ACID 69, S\_DEMON, S\_MONSTER, S\_UNDEAD | no-conf, no-sleep, no-hold | smart, spirit, opens doors, bashes doors, evil |
| 42 | black-hearted huorn | `l` | +10 | 1200 | 120 | 2900 | 1 | crush 5d6, crush 5d6, crush 5d6, crush 5d6 | spells 1/7: HASTE, TELE\_TO, DARKNESS, SCARE, BLIND | im-water, hurt-fire, no-fear, no-stun, no-conf, no-sleep, no-slow | cold-blooded, weird-mind (ESP 1 in 10), regenerates, kill-body, bashes doors, evil |
| 42 | hellhound | `C` | +10 | 352 | 120 | 600 | 2 | bite fire 3d12, bite fire 3d12, bite fire 3d12 | innate 1/5: BR\_FIRE 117 | im-fire | erratic 25%, group-AI, move-body, bashes doors, evil, animal, friends: 100% 2d5 Same |
| 43 | glabrezu | `U` | +0 | 616 | 60 | 3000 | 2 | hit 3d4, hit 3d4 | spells 1/9: BO\_FIRE 86, S\_DEMON | im-fire, no-conf, no-sleep, no-hold | spirit, opens doors, bashes doors, evil, demon |
| 43 | Maia of Mandos | `A` | +10 | 1250 | 81 | 1900 | 4 | hit 4d5, hit 4d5, hit 4d5, hit 4d5 | spells 1/3: BLIND, HASTE, HOLD, CONF, SHAPECHANGE | im-fire, im-cold, im-pois, no-conf, no-sleep, no-hold | INVISIBLE, smart, spirit, powerful, takes items, opens doors, bashes doors |
| 43 | Maia of Nienna | `A` | +10 | 1250 | 81 | 1900 | 4 | hit 3d5, hit 3d5, hit 3d5, hit 3d5 | spells 1/3: BLIND, HASTE, HEAL, SCARE, BO\_MANA 157, SHAPECHANGE | im-fire, im-cold, im-pois, no-conf, no-sleep, no-hold | INVISIBLE, smart, spirit, powerful, takes items, opens doors, bashes doors |
| 43 | mature multi-hued dragon | `d` | +5 | 1000 | 96 | 1950 | 1 | claw 3d12, claw 3d12, bite 4d12 | innate 1/10: BR\_ACID 333, BR\_COLD 333, BR\_ELEC 333, BR\_FIRE 333, BR\_POIS 333; spells 1/15: BLIND, CONF, SCARE | im-acid, im-elec, im-fire, im-cold, im-pois, no-conf, no-sleep | spirit, bashes doors, evil, dragon, friends: 50% 1d4 Baby multi-hued dragon; 50% 1d2 Young multi-hued dragon |
| 43 | multi-hued hound | `Z` | +0 | 220 | 48 | 600 | 6 | claw 2d8, claw 2d8, bite 4d4, bite 4d4 | innate 1/5: BR\_ACID 73, BR\_COLD 73, BR\_ELEC 73, BR\_FIRE 73, BR\_POIS 73 | im-acid, im-elec, im-fire, im-cold, im-pois, no-conf, no-sleep | group-AI, bashes doors, animal, friends: 100% 2d7 Same |
| 44 | 8-headed hydra | `M` | +10 | 950 | 120 | 6000 | 2 | bite 2d6, bite 2d6, bite poison 2d6, bite fire 2d6, bite acid 2d6, bite blind 2d6, bite confuse 2d6, bite exp\_10 2d6 | innate 1/7: BR\_FIRE 316, BR\_POIS 316, BR\_ACID 316; spells 1/5: SCARE | im-acid, im-fire, im-pois, hurt-cold | move-body, bashes doors, animal |
| 44 | dread | `G` | +10 | 263 | 36 | 600 | 2 | hit 6d6, hit 6d6, hit lose\_str 3d4 | spells 1/15: BLIND, CONF, DRAIN\_MANA, HOLD, BO\_NETH 121 | im-cold, im-pois, no-stun, no-conf, no-sleep, no-hold | INVISIBLE, cold-blooded, pass-wall, erratic 25%, spirit, takes items, evil, undead, friends: 100% 2d5 Same |
| 45 | berserker | `p` | +10 | 1320 | 96 | 2500 | 2 | hit 6d6, hit 6d6, hit 6d6 | spells 1/4: HASTE, SCARE | no-fear, no-conf, no-sleep | spirit, kill-body, bashes doors, friends: 30% 1d1 Master thief; 30% 1d1 Ranger chieftain; 30% 1d1 Patriarch; 30% 1d1 Sorcerer |
| 45 | cyclops | `P` | +10 | 1050 | 144 | 1500 | 2 | hit 9d9, hit 9d9 | innate 1/8: BOULDER 84 | im-acid, im-fire, im-cold | spirit, takes items, move-body, opens doors, bashes doors, evil, giant |
| 45 | demonic quylthulg | `Q` | +10 | 420 | 1 | 3000 | 1 | - | spells 1/2: BLINK, TPORT, S\_DEMON | no-fear, no-conf, no-sleep, no-hold | INVISIBLE, empty-mind (no ESP), never-moves, no melee |
| 45 | draconic quylthulg | `Q` | +10 | 420 | 1 | 3000 | 1 | - | spells 1/2: BLINK, TPORT, S\_DRAGON | no-fear, no-conf, no-sleep, no-hold | INVISIBLE, empty-mind (no ESP), never-moves, no melee |
| 45 | greater basilisk | `R` | +10 | 1010 | 120 | 10000 | 2 | gaze paralyze 3d12, gaze paralyze 3d12, bite poison 2d12, bite poison 2d12 | innate 1/8: BR\_DARK 168, BR\_NEXU 168, BR\_POIS 336 | im-pois, im-nexus, hurt-cold, no-conf, no-sleep, no-hold | powerful, opens doors, bashes doors, evil, animal |
| 45 | grey wraith | `W` | +10 | 650 | 60 | 1700 | 1 | hit 1d10, hit 1d10, touch exp\_40 | spells 1/7: WOUND 150, DARKNESS, HOLD, SCARE, BO\_COLD 63 | im-cold, im-pois, hurt-light, no-stun, no-conf, no-sleep, no-hold | cold-blooded, spirit, opens doors, bashes doors, evil, undead |
| 45 | killer white beetle | `K` | +10 | 800 | 106 | 850 | 2 | bite cold 8d5, bite 8d5, spit cold 8d5 | innate 1/4: SPIT 80 | im-cold | weird-mind (ESP 1 in 10), bashes doors, animal |
| 45 | nalfeshnee | `U` | +0 | 792 | 60 | 5000 | 2 | hit 3d4, hit 3d4, hit 3d4 | innate 1/30: BR\_FIRE 264; spells 1/12: BLIND, CONF, S\_DEMON | im-fire, no-conf, no-sleep, no-hold | spirit, powerful, opens doors, bashes doors, evil, demon |
| 45 | rotting quylthulg | `Q` | +10 | 420 | 1 | 3000 | 1 | - | spells 1/2: BLINK, TPORT, S\_UNDEAD | no-fear, no-conf, no-sleep, no-hold | INVISIBLE, empty-mind (no ESP), never-moves, no melee |
| 45 | undead beholder | `e` | +10 | 2376 | 120 | 8000 | 3 | gaze exp\_40 3d6, gaze drain\_charges 3d6, gaze lose\_int 3d6, bite exp\_40 7d6 | spells 1/2: BRAIN\_SMASH 180, WOUND 150, CONF, DRAIN\_MANA, FORGET, MIND\_BLAST 64, SLOW, BA\_COLD 77, BA\_FIRE 167, BO\_ACID 71, BO\_MANA 162, BO\_NETH 122, S\_UNDEAD | im-acid, im-elec, im-fire, im-cold, im-pois, no-stun, no-conf, no-sleep | cold-blooded, spirit, bashes doors, evil, undead |

## Levels 46-50

| DL | Monster | Sym | Spd | HP | AC | Exp | R | Blows | Spells | Defences | Flags |
|---:|---|:-:|---:|---:|---:|---:|---:|---|---|---|---|
| 46 | lord of Carn Dûm | `p` | +10 | 557 | 90 | 2100 | 2 | hit 5d4, hit 5d4, hit 3d5 | spells 1/2: BLIND, WOUND 150, HEAL, HOLD, S\_MONSTER, S\_UNDEAD | no-conf, no-sleep, no-hold | smart, spirit, opens doors, bashes doors, evil, friends: 30% 1d1 Master thief; 30% 1d1 Ranger chieftain; 30% 1d1 Sorcerer; 30% 1d1 Berserker |
| 46 | vampire lord | `V` | +10 | 1400 | 84 | 1800 | 1 | hit 3d6, hit 3d6, bite exp\_80 4d6, bite exp\_80 4d6 | spells 1/7: BLIND, BRAIN\_SMASH 180, WOUND 150, DARKNESS, DRAIN\_MANA, BO\_NETH 124, HOLD, SCARE | im-cold, im-pois, hurt-light, hurt-fire, no-stun, no-conf, no-sleep | cold-blooded, regenerates, spirit, opens doors, bashes doors, evil, undead, friends: 40% 1d4 Vampire (servant) |
| 47 | death mold | `m` | +30 | 1050 | 72 | 1000 | 1 | spore disenchant 7d7, spore disenchant 7d7, spore disenchant 7d7, spore exp\_80 5d5 | - | im-acid, im-elec, im-fire, im-cold, im-pois, no-fear, no-conf, no-sleep | never-moves, evil |
| 47 | marilith | `U` | +10 | 1232 | 112 | 7000 | 2 | hit 4d6, hit 4d6, hit 4d6, hit 4d6 | spells 1/9: BLIND, WOUND 150, S\_DEMON | im-fire, no-conf, no-sleep, no-hold | spirit, opens doors, bashes doors, evil, demon |
| 48 | ancient spider | `S` | +10 | 1050 | 78 | 2500 | 5 | claw poison 5d8, claw poison 5d8, bite poison 5d6, bite poison 5d6 | innate 1/20: BR\_DARK 175; spells 1/6: WOUND 160, DRAIN\_MANA, HEAL, HOLD, SLOW, S\_KIN, S\_SPIDER | im-pois, hurt-light | smart, spirit, opens doors, bashes doors, evil, animal, friends: 100% 4d4 spider |
| 48 | gorgon | `n` | +10 | 1500 | 120 | 6000 | 2 | gaze exp\_80, gaze paralyze, hit 8d6, bite poison 8d6 | spells 1/4: WOUND 160, HOLD, SCARE, BA\_ACID 159, BO\_FIRE 88, BO\_PLAS 114, S\_HYDRA, S\_KIN | im-acid, im-fire, im-pois, no-conf, no-sleep, no-hold | smart, spirit, opens doors, bashes doors, evil |
| 48 | winged horror | `B` | +10 | 1013 | 96 | 4000 | 3 | claw 3d8, claw 3d8, bite exp\_40 4d6, bite exp\_40 4d6 | innate 1/6: BR\_DARK 168, BR\_NETH 168, BR\_POIS 337 | im-cold, im-pois, im-nether | animal |
| 49 | black wraith | `W` | +10 | 840 | 66 | 2700 | 2 | hit 1d12, hit 1d12, touch exp\_40, touch exp\_40 | spells 1/7: BLIND, WOUND 160, HOLD, SCARE, BO\_NETH 128 | im-cold, im-pois, hurt-light, no-stun, no-conf, no-sleep, no-hold | cold-blooded, spirit, opens doors, bashes doors, evil, undead |
| 49 | lesser Balrog | `U` | +10 | 1760 | 75 | 10000 | 2 | hit fire 4d12, hit fire 4d12, crush 3d12, touch drain\_charges | innate 1/14: BR\_FIRE 586; spells 1/5: BLIND, CONF, S\_DEMON | im-fire, no-conf, no-sleep, no-hold | spirit, powerful, move-body, opens doors, bashes doors, evil, demon |
| 49 | storm giant | `P` | +10 | 1200 | 110 | 1400 | 1 | hit elec 10d8, hit elec 10d8, hit 10d8 | innate 1/40: BOULDER 96; spells 1/8: BLINK, CONF, SCARE, TELE\_TO, BA\_ELEC 81, BO\_ELEC 48, STORM 310 | im-elec, im-cold | spirit, takes items, opens doors, bashes doors, evil, giant, friends: 60% 1d7 Energy hound; 80% 1d3 Same |
| 50 | ancient blue dragon | `D` | +10 | 1500 | 108 | 2500 | 1 | claw 6d8, claw 6d8, bite elec 7d8 | innate 1/12: BR\_ELEC 500; spells 1/8: BLIND, CONF, SCARE | im-elec, no-conf, no-sleep, no-hold | smart, spirit, powerful, move-body, bashes doors, evil, dragon |
| 50 | ancient green dragon | `D` | +10 | 1500 | 108 | 2500 | 1 | claw 6d8, claw 6d8, bite poison 7d8 | innate 1/12: BR\_POIS 500; spells 1/8: BLIND, CONF, SCARE | im-pois, no-conf, no-sleep, no-hold | smart, spirit, powerful, move-body, bashes doors, evil, dragon |
| 50 | ancient white dragon | `D` | +10 | 1500 | 108 | 2500 | 1 | claw 6d8, claw 6d8, bite cold 7d8 | innate 1/12: BR\_COLD 500; spells 1/8: BLIND, CONF, SCARE | im-cold, no-conf, no-sleep, no-hold | smart, spirit, powerful, move-body, bashes doors, evil, dragon |
| 50 | death drake | `D` | +10 | 1950 | 120 | 11000 | 2 | claw 6d10, claw 6d10, bite exp\_80 7d10 | innate 1/12: BR\_NETH 325; spells 1/6: CONF, DARKNESS, SCARE, SLOW | im-cold, im-nether, no-conf, no-sleep, no-hold | INVISIBLE, pass-wall, smart, spirit, powerful, takes items, move-body, evil, dragon |
| 50 | ethereal dragon | `D` | +10 | 1950 | 120 | 11000 | 2 | claw 6d10, claw 6d10, bite confuse 7d10 | innate 1/9: BR\_DARK 325, BR\_LIGHT 325; spells 1/10: BLIND, CONF | no-conf, no-sleep, no-hold | INVISIBLE, pass-wall, smart, spirit, powerful, move-body, evil, dragon |
| 50 | great crystal drake | `D` | +10 | 1950 | 120 | 11000 | 2 | claw 6d10, claw 6d10, bite 7d10 | innate 1/12: BR\_SHAR 325; spells 1/6: CONF, SCARE, SLOW | im-cold, no-conf, no-sleep, no-hold | smart, spirit, powerful, move-body, bashes doors, evil, dragon |
| 50 | master mystic | `p` | +20 | 968 | 72 | 6000 | 3 | kick 10d2, kick 10d2, hit paralyze 20d1, hit paralyze 15d1 | spells 1/3: HEAL, S\_ANIMAL | im-acid, im-elec, im-fire, im-cold, im-pois, no-conf, no-sleep, no-hold | INVISIBLE, spirit, opens doors, bashes doors, friends: 50% 2d4 any spider |
| 50 | shadow | `G` | +10 | 175 | 36 | 800 | 3 | touch exp\_80, touch exp\_40, claw lose\_int 3d10, claw lose\_wis 3d10 | spells 1/8: SLOW, TELE\_TO, BO\_NETH 130 | im-cold, im-pois, no-stun, no-conf, no-sleep, no-hold | INVISIBLE, cold-blooded, pass-wall, spirit, evil, undead |

## Levels 51-55

| DL | Monster | Sym | Spd | HP | AC | Exp | R | Blows | Spells | Defences | Flags |
|---:|---|:-:|---:|---:|---:|---:|---:|---|---|---|---|
| 51 | ancient black dragon | `D` | +10 | 1560 | 108 | 2800 | 1 | claw 6d9, claw 6d9, bite acid 7d9 | innate 1/12: BR\_ACID 520; spells 1/8: BLIND, CONF, SCARE | im-acid, no-conf, no-sleep, no-hold | smart, spirit, powerful, move-body, bashes doors, evil, dragon |
| 51 | ancient gold dragon | `D` | +10 | 1560 | 108 | 2800 | 1 | claw 6d9, claw 6d9, bite 7d9 | innate 1/12: BR\_SOUN 260; spells 1/8: BLIND, CONF, SCARE | no-stun, no-conf, no-sleep, no-hold | smart, spirit, powerful, move-body, bashes doors, evil, dragon |
| 51 | ancient red dragon | `D` | +10 | 1560 | 108 | 2800 | 1 | claw 6d9, claw 6d9, bite fire 7d9 | innate 1/12: BR\_FIRE 520; spells 1/8: BLIND, CONF, SCARE | im-fire, no-conf, no-sleep, no-hold | smart, spirit, powerful, move-body, bashes doors, evil, dragon |
| 51 | nether hound | `Z` | +10 | 330 | 120 | 5000 | 4 | claw 2d12, claw 2d12, bite 2d12, bite 2d12 | innate 1/5: BR\_NETH 55 | im-nether, no-conf, no-sleep | group-AI, opens doors, bashes doors, animal, friends: 100% 2d7 Same |
| 51 | plasma hound | `Z` | +10 | 330 | 120 | 5000 | 4 | claw 2d12, claw 2d12, bite 2d12, bite 2d12 | innate 1/5: BR\_PLAS 55 | im-elec, im-fire, im-plasma, no-conf, no-sleep, no-hold | group-AI, opens doors, bashes doors, animal, friends: 100% 2d7 Same |
| 51 | time hound | `Z` | +20 | 330 | 120 | 5000 | 4 | claw 2d12, claw 2d12, bite 2d12, bite 2d12 | innate 1/5: BR\_TIME 110 | no-conf, no-sleep, no-hold | group-AI, opens doors, bashes doors, animal, friends: 100% 2d7 Same |
| 52 | hasty ent | `l` | +10 | 3000 | 120 | 3500 | 3 | punch 10d5, punch 10d5, crush 3d10, crush 3d10 | - | im-acid, im-water, hurt-fire, no-fear, no-stun, no-conf, no-sleep, no-slow | cold-blooded, regenerates, kill-wall, destroys items, move-body, friends: 100% 3d4 huorn |
| 52 | Maia of Oromë | `A` | +10 | 2000 | 96 | 2500 | 4 | hit 4d6, hit 6d5, hit 4d6, hit 6d5 | innate 1/5: ARROW 42; spells 1/5: SHAPECHANGE, S\_HOUND | im-fire, im-cold, im-pois, no-conf, no-sleep, no-hold | smart, spirit, powerful, takes items, opens doors, bashes doors |
| 53 | ancient multi-hued dragon | `D` | +10 | 2400 | 120 | 14000 | 1 | claw 6d10, claw 6d10, bite 7d10 | innate 1/6: BR\_ACID 800, BR\_COLD 800, BR\_ELEC 800, BR\_FIRE 800, BR\_POIS 800; spells 1/13: BLIND, CONF, SCARE | im-acid, im-elec, im-fire, im-cold, im-pois, no-conf, no-sleep, no-hold | smart, spirit, powerful, move-body, bashes doors, evil, dragon |
| 53 | chaos vortex | `v` | +10 | 315 | 96 | 4000 | 3 | engulf confuse 5d5, engulf confuse 5d5, engulf confuse 5d5, engulf hallu 5d5 | innate 1/4: BR\_CHAO 52 | im-pois, no-fear, no-conf, no-sleep, no-hold | empty-mind (no ESP), erratic 25%, erratic 50%, powerful, bashes doors, nonliving |
| 53 | hezrou | `U` | -10 | 1850 | 60 | 7000 | 3 | claw 3d4, bite acid 3d10, bite blind 3d10 | innate 1/3: BR\_FIRE 616, BR\_ACID 616; spells 1/3: BO\_FIRE 89, BO\_ACID 73, S\_DEMON | im-acid, im-fire, no-fear, no-conf, no-sleep, no-hold | smart, spirit, opens doors, bashes doors, evil, demon, friends: 100% 1d2 Same |
| 54 | demilich | `L` | +10 | 2816 | 150 | 12500 | 2 | touch exp\_80, touch drain\_charges, touch lose\_dex 4d12, touch lose\_dex 4d12 | spells 1/3: BLIND, BLINK, WOUND 180, CONF, BRAIN\_SMASH 180, DRAIN\_MANA, FORGET, HEAL, HOLD, SCARE, TELE\_TO, TPORT, S\_DEMON, S\_UNDEAD | im-cold, im-pois, no-stun, no-conf, no-sleep, no-hold | cold-blooded, smart, spirit, opens doors, bashes doors, evil, undead |
| 54 | dreadmaster | `G` | +10 | 1056 | 120 | 8000 | 2 | hit 6d6, hit 6d6, hit lose\_str 3d4, hit lose\_str 3d4 | spells 1/9: BLIND, WOUND 180, CONF, DRAIN\_MANA, HOLD, TELE\_LEVEL, BO\_NETH 136, S\_UNDEAD | im-cold, im-pois, no-stun, no-conf, no-sleep, no-hold | INVISIBLE, cold-blooded, pass-wall, erratic 25%, smart, spirit, takes items, evil, undead |
| 54 | drolem | `g` | +10 | 2200 | 195 | 12000 | 3 | claw 3d10, claw 3d10, bite poison 5d10, bite poison 5d10 | innate 1/8: ARROW 42, BR\_POIS 733; spells 1/8: BLIND, CONF, SLOW | im-elec, im-fire, im-cold, im-pois, no-fear, no-conf, no-sleep | cold-blooded, empty-mind (no ESP), opens doors, bashes doors, dragon, nonliving |
| 54 | nether wraith | `W` | +10 | 1000 | 66 | 3900 | 2 | hit 1d12, hit 1d12, touch exp\_80, touch exp\_80 | spells 1/6: BLIND, WOUND 180, DARKNESS, MIND\_BLAST 64, SCARE, BO\_NETH 136, BE\_NETH 163 | im-cold, im-pois, hurt-light, no-stun, no-conf, no-sleep, no-hold | INVISIBLE, cold-blooded, pass-wall, spirit, evil, undead |
| 55 | barbazu | `U` | +10 | 700 | 90 | 3000 | 4 | hit 4d10, hit 4d10, hit lose\_con 10d2, sting poison 5d5 | spells 1/10: SCARE, S\_DEMON | im-pois, no-conf, no-sleep, no-hold | spirit, opens doors, bashes doors, evil, demon, friends: 100% 4d4 Same |
| 55 | dracolich | `D` | +10 | 3080 | 144 | 18000 | 2 | claw 6d12, claw 6d12, bite exp\_80 7d14 | innate 1/10: BR\_COLD 1026, BR\_NETH 513; spells 1/12: CONF, SCARE | im-cold, im-pois, im-nether, no-stun, no-conf, no-sleep, no-hold | cold-blooded, smart, spirit, powerful, takes items, move-body, bashes doors, evil, undead, dragon |
| 55 | dracolisk | `D` | +10 | 3080 | 144 | 18000 | 2 | bite 6d12, bite 6d12, bite fire 7d14, gaze paralyze | innate 1/10: BR\_FIRE 1026, BR\_NEXU 400; spells 1/12: HOLD, SCARE | im-acid, im-fire, im-nexus, no-conf, no-sleep, no-hold | smart, spirit, powerful, move-body, bashes doors, evil, dragon, animal |

## Levels 56-60

| DL | Monster | Sym | Spd | HP | AC | Exp | R | Blows | Spells | Defences | Flags |
|---:|---|:-:|---:|---:|---:|---:|---:|---|---|---|---|
| 56 | lesser titan | `P` | +10 | 2100 | 96 | 6000 | 3 | hit confuse 9d9, hit confuse 9d9, hit confuse 9d9, hit confuse 9d9 | innate 1/6: BOULDER 108; spells 1/4: HEAL, TELE\_TO, S\_MONSTERS | - | smart, spirit, takes items, opens doors, bashes doors, evil, giant |
| 57 | 9-headed hydra | `M` | +10 | 1200 | 168 | 7000 | 2 | bite 2d6, bite 2d6, bite poison 2d6, bite fire 2d6, bite acid 2d6, bite blind 2d6, bite confuse 2d6, bite exp\_10 2d6, bite 10d6 | innate 1/4: BR\_FIRE 400, BR\_POIS 400, BR\_ACID 400, BR\_NETH 200; spells 1/9: SCARE, S\_HYDRA, S\_KIN | im-acid, im-fire, im-pois, im-nether, hurt-cold, no-conf, no-sleep, no-hold | regenerates, smart, powerful, kill-body, bashes doors, animal |
| 57 | grand master mystic | `p` | +20 | 1936 | 96 | 15000 | 3 | kick 20d2, kick 10d2, hit paralyze 20d1, hit paralyze 15d1 | spells 1/3: HEAL, MIND\_BLAST 64, S\_ANIMAL, S\_HOUND, S\_SPIDER | im-acid, im-elec, im-fire, im-cold, im-pois, no-conf, no-sleep, no-hold | INVISIBLE, spirit, opens doors, bashes doors, friends: 80% 3d5 any spider |
| 57 | hand druj | `s` | +20 | 528 | 132 | 12000 | 2 | - | spells 1/1: BLIND, WOUND 190, CONF, DARKNESS, FORGET, SCARE, TELE\_AWAY, BO\_ACID 75, BO\_COLD 67, BO\_ELEC 51, BO\_FIRE 91 | im-cold, im-pois, no-fear, no-stun, no-conf, no-sleep | cold-blooded, never-moves, no melee, smart, spirit, evil, undead |
| 59 | elder vampire | `V` | +10 | 2640 | 108 | 4500 | 3 | hit 4d6, hit 4d6, bite exp\_80 5d6, bite exp\_80 5d6 | spells 1/5: BRAIN\_SMASH 180, WOUND 190, DARKNESS, DRAIN\_MANA, BO\_NETH 143, S\_UNDEAD, HOLD, SCARE | im-cold, im-pois, hurt-light, hurt-fire, no-stun, no-conf, no-sleep | INVISIBLE, cold-blooded, regenerates, pass-wall, spirit, opens doors, bashes doors, evil, undead, friends: 30% 1d3 any ghost |
| 60 | aether vortex | `v` | +20 | 420 | 48 | 5000 | 4 | engulf elec 5d5, engulf fire 5d5, engulf acid 5d5, engulf cold 5d5 | innate 1/4: BR\_ACID 140, BR\_CHAO 70, BR\_COLD 140, BR\_DARK 70, BR\_ELEC 140, BR\_FIRE 140, BR\_GRAV 140, BR\_INER 70, BR\_LIGHT 70, BR\_NETH 70, BR\_NEXU 70, BR\_PLAS 70, BR\_POIS 140, BR\_SHAR 70, BR\_SOUN 70, BR\_TIME 140, BR\_WALL 70 | im-acid, im-elec, im-fire, im-cold, im-pois, im-nether, im-plasma, im-nexus, no-fear, no-stun, no-conf, no-sleep, no-hold, no-slow | empty-mind (no ESP), erratic 25%, erratic 50%, powerful, bashes doors, nonliving |
| 60 | great earth elemental | `E` | +10 | shape | 145 | 0 | 0 | hit 6d6, hit 6d6, hit 6d6, hit shatter 10d10 | innate 1/6: BOULDER 108, BR\_SHAR; spells 1/6: BA\_SHAR 100 | im-elec, im-pois, hurt-rock, no-fear, no-conf, no-sleep | cold-blooded, empty-mind (no ESP), pass-wall, powerful, destroys items, kill-body, nonliving |
| 60 | Maia of Aulë | `A` | +10 | 3500 | 120 | 3600 | 4 | hit 6d6, hit 6d6, hit 6d6, hit shatter 10d10 | innate 1/5: BOULDER 108; spells 1/5: SHAPECHANGE | im-elec, im-fire, im-cold, im-pois, no-conf, no-sleep, no-hold | pass-wall, smart, spirit, powerful, takes items, opens doors, bashes doors |
| 60 | Maia of Yavanna | `A` | +10 | 3500 | 90 | 3600 | 4 | hit 4d6, hit 6d5, hit 4d6, hit 6d5 | spells 1/3: BO\_MANA 200, BO\_WATE 160, S\_AINU, S\_ANIMAL, SHAPECHANGE | im-acid, im-fire, im-cold, im-pois, no-conf, no-sleep, no-hold | smart, spirit, powerful, takes items, opens doors, bashes doors |

## Levels 61-65

| DL | Monster | Sym | Spd | HP | AC | Exp | R | Blows | Spells | Defences | Flags |
|---:|---|:-:|---:|---:|---:|---:|---:|---|---|---|---|
| 61 | bile demon | `U` | +10 | 2464 | 135 | 12000 | 2 | hit 8d8, hit 8d8, crush acid 9d9 | innate 1/6: ARROW 48, BR\_ACID 821, BR\_POIS 800; spells 1/10: BLIND, CONF, S\_DEMON | im-acid, im-pois, no-conf, no-sleep, no-hold | spirit, powerful, opens doors, bashes doors, evil, demon |
| 61 | nightwing | `W` | +10 | 1300 | 144 | 10000 | 3 | touch poison 6d5, touch poison 6d5, hit disenchant 6d8, hit disenchant 6d8 | spells 1/4: BLIND, BRAIN\_SMASH 180, WOUND 200, SCARE, BA\_NETH 344, BO\_MANA 202, BO\_NETH 146, S\_UNDEAD | im-cold, im-pois, hurt-light, no-stun, no-conf, no-sleep, no-hold | cold-blooded, smart, spirit, opens doors, bashes doors, evil, undead |
| 62 | dreadlord | `G` | +10 | 2640 | 180 | 20000 | 2 | hit exp\_40 6d6, hit exp\_40 6d6, hit lose\_str 4d6, hit lose\_str 4d6 | spells 1/4: BLIND, BLINK, BRAIN\_SMASH 180, CONF, DRAIN\_MANA, HOLD, TELE\_TO, TPORT, BA\_NETH 348, S\_UNDEAD | im-cold, im-pois, no-stun, no-conf, no-sleep, no-hold | INVISIBLE, cold-blooded, pass-wall, spirit, takes items, evil, undead, friends: 50% 2d3 Dread (servant) |
| 63 | spider of Gorgoroth | `S` | +10 | 4000 | 180 | 30000 | 3 | claw poison 6d6, claw poison 6d6, bite paralyze 5d10, sting lose\_str 5d4 | innate 1/20: WEAVE, BR\_DARK 500, BR\_POIS 800; spells 1/5: BLIND, WOUND 210, CONF, HEAL, SCARE, SLOW, S\_KIN | im-acid, im-pois, hurt-light, no-conf, no-sleep, no-hold | smart, spirit, bashes doors, evil, animal, friends: 100% 2d4 spider |
| 63 | werewolf of Sauron | `C` | +10 | 2200 | 88 | 4000 | 2 | bite exp\_20 2d4, claw poison 4d10, claw poison 4d10 | innate 1/6: BR\_NETH 366, BR\_DARK 366; spells 1/7: S\_HOUND | im-fire, im-cold, im-pois, im-nether, no-fear, no-conf, no-sleep | smart, move-body, opens doors, bashes doors, evil, animal, friends: 50% 2d4 warg (servant); 50% 2d3 hellhound (servant); 100% 2d3 werewolf (servant) |
| 64 | archlich | `L` | +10 | 3520 | 180 | 20000 | 2 | touch exp\_80, touch drain\_charges, touch lose\_dex 8d12, touch lose\_dex 8d12 | spells 1/3: BLINK, WOUND 210, DRAIN\_MANA, FORGET, HEAL, BRAIN\_SMASH 180, HOLD, SCARE, TELE\_TO, TPORT, BA\_NETH 356, S\_DEMON, S\_HI\_UNDEAD | im-cold, im-pois, no-stun, no-conf, no-sleep, no-hold | cold-blooded, pass-wall, smart, spirit, opens doors, bashes doors, evil, undead |
| 64 | chaos hound | `Z` | +10 | 930 | 120 | 10000 | 4 | claw 2d12, claw 2d12, bite 2d12, bite 2d12 | innate 1/5: BR\_CHAO 155 | no-conf, no-sleep, no-hold | group-AI, bashes doors, animal, friends: 100% 2d7 Same |
| 65 | Fëanorian raider | `h` | +10 | 1200 | 110 | 11000 | 2 | hit 7d12, hit 7d12, hit 7d12, hit 7d12 | innate 1/3: BOLT 63 | im-acid, im-elec, no-fear | spirit, move-body, opens doors, bashes doors |
| 65 | great water elemental | `E` | +10 | shape | 120 | 4250 | 0 | engulf terrify 6d5, engulf paralyze 6d5, engulf acid 6d10, engulf acid 6d10 | innate 1/5: BR\_COLD; spells 1/5: BA\_WATE 212, BO\_WATE 165 | im-acid, im-cold, im-pois, im-water, no-fear, no-conf, no-sleep | cold-blooded, empty-mind (no ESP), powerful, destroys items, kill-body, bashes doors, nonliving |
| 65 | Maia of Ulmo | `A` | +10 | 4700 | 90 | 5000 | 4 | touch terrify 6d5, touch paralyze 6d5, engulf acid 6d10, engulf acid 6d10 | spells 1/3: BA\_WATE 212, BO\_MANA 212, BO\_WATE 165, S\_AINU, SHAPECHANGE | im-acid, im-elec, im-fire, im-cold, im-pois, im-water, no-conf, no-sleep, no-hold | smart, spirit, powerful, takes items, opens doors, bashes doors |
| 65 | osyluth | `U` | +20 | 2288 | 112 | 13000 | 2 | hit lose\_wis 6d6, hit lose\_con 6d6, bite poison 8d8, sting lose\_str 5d5 | spells 1/6: SCARE, BA\_COLD 107, BA\_ELEC 105, BO\_ICE 101, S\_DEMON | im-acid, im-elec, im-fire, im-cold, im-pois, no-conf, no-sleep, no-hold | INVISIBLE, spirit, powerful, move-body, opens doors, bashes doors, evil, demon |

## Levels 66-70

| DL | Monster | Sym | Spd | HP | AC | Exp | R | Blows | Spells | Defences | Flags |
|---:|---|:-:|---:|---:|---:|---:|---:|---|---|---|---|
| 66 | eye druj | `s` | +20 | 1080 | 108 | 18000 | 2 | - | spells 1/1: BRAIN\_SMASH 180, BA\_NETH 364, BO\_MANA 215, BO\_NETH 154, S\_UNDEAD | im-fire, im-cold, im-pois, no-fear, no-stun, no-conf, no-sleep | cold-blooded, never-moves, no melee, smart, spirit, evil, undead |
| 66 | greater titan | `P` | +10 | 3344 | 150 | 23500 | 3 | hit confuse 12d12, hit confuse 12d12, hit confuse 12d12, hit confuse 12d12 | innate 1/6: BOULDER 120; spells 1/4: HEAL, TELE\_TO, S\_MONSTERS | - | smart, spirit, takes items, bashes doors, evil, giant |
| 67 | beholder hive-mother | `e` | +10 | 3080 | 96 | 17000 | 3 | bite exp\_80 6d6, gaze paralyze 5d5, gaze lose\_int 5d5, gaze drain\_charges 5d5 | spells 1/2: BLIND, BRAIN\_SMASH 180, CONF, DRAIN\_MANA, FORGET, SCARE, BA\_ACID 216, BA\_COLD 110, BA\_DARK 368, BA\_FIRE 244, BO\_MANA 217, BO\_NETH 155, S\_KIN | no-conf, no-sleep, no-hold | smart, spirit, bashes doors, evil |
| 68 | bone golem | `g` | +10 | 3080 | 255 | 23000 | 3 | hit disenchant 8d8, hit disenchant 8d8, hit lose\_str 6d6, hit lose\_str 6d6 | spells 1/8: BRAIN\_SMASH 180, WOUND 220, DRAIN\_MANA, TELE\_TO, BA\_NETH 372, S\_UNDEAD | im-elec, im-fire, im-cold, im-pois, no-fear, no-stun, no-conf, no-sleep | cold-blooded, empty-mind (no ESP), opens doors, bashes doors, undead, nonliving |
| 68 | jabberwock | `H` | +20 | 2816 | 187 | 19000 | 3 | claw 10d10, claw 10d10, bite 10d10, bite 10d10 | innate 1/9: BR\_CHAO 469; spells 1/10: WOUND 220 | no-conf | bashes doors, animal |
| 69 | gelugon | `U` | +20 | 3080 | 150 | 14000 | 3 | claw cold 6d8, claw cold 6d8, bite cold 9d9, hit paralyze 5d5 | innate 1/20: BR\_COLD 1026, BR\_SHAR 500; spells 1/8: HOLD, SCARE, SLOW, BA\_COLD 113, BO\_ICE 105, S\_HI\_DEMON | im-cold, no-conf, no-sleep, no-hold | regenerates, spirit, powerful, opens doors, bashes doors, evil, demon |
| 69 | hound of Tindalos | `Z` | +20 | 480 | 120 | 7000 | 6 | claw 2d12, claw 2d12, bite 2d12, bite 2d12 | innate 1/5: BR\_NETH 80, BR\_DISE 80, BR\_TIME 150 | im-nether, im-disen, no-conf, no-sleep, no-hold | INVISIBLE, pass-wall, group-AI, animal, friends: 100% 2d7 Same |
| 69 | nightcrawler | `J` | +10 | 2440 | 192 | 15000 | 3 | sting lose\_con 8d8, sting lose\_con 8d8, bite acid 10d10, bite acid 10d10 | innate 1/25: BR\_NETH 406; spells 1/4: BLIND, BRAIN\_SMASH 180, SCARE, BA\_NETH 376, BO\_MANA 222, BO\_NETH 158, S\_UNDEAD | im-fire, im-cold, im-pois, im-nether, hurt-light, no-stun, no-conf, no-sleep, no-hold | cold-blooded, smart, spirit, opens doors, bashes doors, evil, undead, animal |
| 70 | great ice wyrm | `D` | +10 | 4200 | 225 | 25000 | 2 | claw 6d12, claw 6d12, bite cold 6d14, bite cold 6d14 | innate 1/8: BR\_COLD 1400; spells 1/5: BLIND, CONF, SCARE | im-cold, no-conf, no-sleep, no-hold | smart, spirit, powerful, move-body, bashes doors, evil, dragon |
| 70 | great storm wyrm | `D` | +10 | 4200 | 225 | 25000 | 2 | claw 6d12, claw 6d12, bite elec 6d14, bite elec 6d14 | innate 1/8: BR\_ELEC 1400; spells 1/5: BLIND, CONF, SCARE | im-elec, no-conf, no-sleep, no-hold | smart, spirit, powerful, move-body, bashes doors, evil, dragon |
| 70 | great swamp wyrm | `D` | +10 | 4200 | 225 | 20000 | 2 | claw 6d12, claw 6d12, bite poison 6d14, bite poison 6d14 | innate 1/8: BR\_POIS 800; spells 1/5: BLIND, CONF, SCARE | im-pois, no-conf, no-sleep, no-hold | smart, spirit, powerful, move-body, bashes doors, evil, dragon |

## Levels 71-75

| DL | Monster | Sym | Spd | HP | AC | Exp | R | Blows | Spells | Defences | Flags |
|---:|---|:-:|---:|---:|---:|---:|---:|---|---|---|---|
| 71 | greater demonic quylthulg | `Q` | +10 | 1320 | 1 | 10500 | 2 | - | spells 1/2: BLINK, TELE\_TO, S\_HI\_DEMON | no-fear, no-conf, no-sleep, no-hold | INVISIBLE, empty-mind (no ESP), never-moves, no melee |
| 71 | greater draconic quylthulg | `Q` | +10 | 1320 | 1 | 10500 | 2 | - | spells 1/2: BLINK, TELE\_TO, S\_HI\_DRAGON | no-fear, no-conf, no-sleep, no-hold | INVISIBLE, empty-mind (no ESP), never-moves, no melee |
| 71 | greater rotting quylthulg | `Q` | +10 | 1320 | 1 | 10500 | 2 | - | spells 1/2: BLINK, TELE\_TO, S\_HI\_UNDEAD | no-fear, no-conf, no-sleep, no-hold | INVISIBLE, empty-mind (no ESP), never-moves, no melee |
| 71 | Maia of Manwë | `A` | +20 | 5000 | 120 | 7000 | 4 | hit elec 10d12, hit elec 10d12, engulf confuse 8d10, engulf confuse 8d10 | spells 1/3: STORM 415, BA\_ELEC 114, S\_AINU, SHAPECHANGE | im-acid, im-elec, im-fire, im-cold, im-pois, im-water, no-conf, no-sleep, no-hold | smart, spirit, powerful, takes items, opens doors, bashes doors |
| 71 | Maia of Varda | `A` | +20 | 5000 | 110 | 12000 | 4 | touch terrify 6d5, touch paralyze 6d5, engulf acid 6d10, engulf acid 6d10 | spells 1/3: BA\_LIGHT 116, BO\_MANA 227, S\_AINU, SHAPECHANGE | im-acid, im-elec, im-fire, im-cold, im-pois, no-conf, no-sleep, no-hold | smart, spirit, powerful, takes items, opens doors, bashes doors |
| 72 | horned reaper | `U` | +20 | 3344 | 180 | 18000 | 3 | hit 11d11, hit 11d11, hit 11d11, hit 11d11 | spells 1/5: HASTE, SCARE, SLOW, S\_HI\_DEMON | im-acid, im-elec, im-fire, im-cold, im-pois, no-conf, no-sleep, no-hold | regenerates, spirit, kill-body, opens doors, bashes doors, evil, demon |
| 73 | great bile wyrm | `D` | +10 | 4500 | 225 | 23000 | 2 | claw 6d12, claw 6d12, bite acid 6d14, bite acid 6d14 | innate 1/8: BR\_ACID 1500; spells 1/5: BLIND, CONF, SCARE | im-acid, no-conf, no-sleep, no-hold | smart, spirit, powerful, move-body, bashes doors, evil, dragon |
| 73 | great hell wyrm | `D` | +10 | 4500 | 225 | 25000 | 2 | claw 6d12, claw 6d12, bite fire 6d14, bite fire 6d14 | innate 1/8: BR\_FIRE 1500; spells 1/5: BLIND, CONF, SCARE | im-fire, no-conf, no-sleep, no-hold | smart, spirit, powerful, move-body, bashes doors, evil, dragon |
| 73 | great wyrm of thunder | `D` | +10 | 4500 | 225 | 23000 | 2 | claw 6d12, claw 6d12, bite 6d14, bite 6d14 | innate 1/8: BR\_SOUN 500; spells 1/5: BLIND, CONF, SCARE | no-stun, no-conf, no-sleep, no-hold | smart, spirit, powerful, move-body, bashes doors, evil, dragon |
| 73 | nightwalker | `W` | +20 | 1400 | 170 | 20000 | 3 | hit disenchant 10d10, hit disenchant 10d10, hit disenchant 8d8, hit disenchant 8d8 | spells 1/4: BLIND, BRAIN\_SMASH 180, SCARE, BA\_NETH 392, BO\_MANA 232, BO\_NETH 164, S\_UNDEAD | im-elec, im-fire, im-cold, im-pois, hurt-light, no-stun, no-conf, no-sleep, no-hold | cold-blooded, smart, spirit, opens doors, bashes doors, evil, undead |
| 74 | aether hound | `Z` | +20 | 1230 | 120 | 10000 | 5 | claw 3d12, claw 3d12, bite 3d12, bite 3d12 | innate 1/5: BR\_ACID 410, BR\_CHAO 205, BR\_COLD 410, BR\_DARK 205, BR\_DISE 205, BR\_ELEC 410, BR\_FIRE 410, BR\_GRAV 200, BR\_INER 200, BR\_LIGHT 205, BR\_NETH 205, BR\_NEXU 205, BR\_PLAS 150, BR\_POIS 410, BR\_SHAR 205, BR\_SOUN 205, BR\_TIME 150, BR\_WALL 200 | im-acid, im-elec, im-fire, im-cold, im-pois, im-nether, im-plasma, im-nexus, im-disen, no-stun, no-conf, no-sleep, no-hold, no-slow | group-AI, bashes doors, animal, friends: 100% 2d7 Same |
| 74 | black reaver | `L` | +10 | 3960 | 255 | 23000 | 3 | hit disenchant 6d8, hit disenchant 6d8, hit lose\_str 4d6, hit lose\_str 4d6 | spells 1/3: BLIND, WOUND 240, CONF, DRAIN\_MANA, BRAIN\_SMASH 180, HOLD, TELE\_TO, BA\_MANA 470, BA\_NETH 396, S\_UNDEAD | im-cold, im-pois, no-stun, no-conf, no-sleep, no-hold | cold-blooded, kill-wall, smart, spirit, opens doors, bashes doors, evil, undead |
| 74 | skull druj | `s` | +20 | 1540 | 144 | 23000 | 2 | - | spells 1/1: BRAIN\_SMASH 180, WOUND 240, TRAPS, BA\_WATE 235, BO\_NETH 166, BO\_PLAS 140, S\_UNDEAD | im-fire, im-cold, im-pois, no-fear, no-stun, no-conf, no-sleep | cold-blooded, never-moves, no melee, smart, spirit, evil, undead |
| 75 | bronze golem | `g` | +10 | 3520 | 255 | 26000 | 3 | hit 10d10, hit 10d10, hit 10d10, hit 10d10 | innate 1/40: BR\_FIRE 1173; spells 1/9: TELE\_TO, BA\_ELEC 120, BA\_FIRE 272, BO\_PLAS 141, S\_DEMON | im-acid, im-elec, im-fire, im-pois, no-fear, no-conf, no-sleep | cold-blooded, empty-mind (no ESP), opens doors, bashes doors, demon, nonliving |
| 75 | great wyrm of chaos | `D` | +10 | 3960 | 255 | 29000 | 2 | claw 5d12, claw 5d12, bite 7d14, bite 7d14 | innate 1/5: BR\_CHAO 600, BR\_DISE 500; spells 1/5: BLIND, CONF, SCARE, S\_DRAGON, S\_HI\_DRAGON | im-fire, im-disen, no-conf, no-sleep, no-hold | smart, spirit, powerful, move-body, bashes doors, evil, dragon |
| 75 | great wyrm of law | `D` | +10 | 3960 | 255 | 29000 | 2 | claw 5d12, claw 5d12, bite 7d14, bite 7d14 | innate 1/5: BR\_SHAR 500, BR\_SOUN 500; spells 1/5: BLIND, CONF, SCARE, S\_DRAGON, S\_HI\_DRAGON | im-cold, no-stun, no-conf, no-sleep, no-hold | smart, spirit, powerful, move-body, bashes doors, evil, dragon |

## Levels 76-80

| DL | Monster | Sym | Spd | HP | AC | Exp | R | Blows | Spells | Defences | Flags |
|---:|---|:-:|---:|---:|---:|---:|---:|---|---|---|---|
| 76 | master quylthulg | `Q` | +10 | 2640 | 1 | 15000 | 2 | - | spells 1/1: BLINK, TELE\_TO, S\_ANIMAL, S\_HI\_DEMON, S\_HI\_DRAGON, S\_HI\_UNDEAD, S\_MONSTERS | no-fear, no-conf, no-sleep, no-hold | INVISIBLE, empty-mind (no ESP), never-moves, no melee |
| 77 | pit fiend | `U` | +20 | 3520 | 180 | 22000 | 3 | claw fire 6d10, claw fire 6d10, bite poison 5d10, bite lose\_con 5d10 | innate 1/12: BR\_CHAO 586, BR\_FIRE 1173, BR\_POIS 800; spells 1/8: WOUND 250, SCARE, BA\_FIRE 279, S\_HI\_DEMON, S\_HI\_DRAGON | im-fire, im-pois, no-conf, no-sleep, no-hold | regenerates, spirit, powerful, move-body, opens doors, bashes doors, evil, demon |
| 78 | serpent of chaos | `J` | +20 | 900 | 145 | 17000 | 3 | crush 15d10, crush confuse 15d10 | innate 1/10: BR\_CHAO 150; spells 1/6: BLINK, TELE\_TO | hurt-cold | erratic 25%, bashes doors, animal |
| 80 | fury | `U` | +20 | 4000 | 120 | 23000 | 2 | claw 6d8, claw 6d8, bite confuse 5d10, engulf hallu 4d10 | spells 1/4: WOUND 260, SCARE, BRAIN\_SMASH 180, BLINK, TELE\_SELF\_TO, S\_HI\_DEMON | im-fire, im-cold, im-pois, no-conf, no-sleep, no-hold | pass-wall, spirit, opens doors, bashes doors, evil, demon |
| 80 | great wyrm of annihilation | `D` | +10 | 5000 | 255 | 31000 | 2 | claw 6d12, claw 6d12, bite 7d14, bite 7d14 | innate 1/5: BR\_POIS 800, BR\_DARK 500, BR\_NETH 600, BR\_CHAO 600, BR\_TIME 150; spells 1/5: BLIND, CONF, SCARE, S\_DRAGON, S\_HI\_DRAGON | im-fire, no-conf, no-sleep, no-hold | smart, spirit, powerful, move-body, bashes doors, evil, dragon |
| 80 | sky dragon | `D` | +10 | 5000 | 255 | 31000 | 2 | claw 6d12, claw 6d12, bite 7d14, bite 7d14 | innate 1/5: BR\_ELEC 1600, BR\_LIGHT 500, BR\_GRAV 200, BR\_PLAS 150, BR\_WALL 200; spells 1/5: BLIND, CONF, SCARE, S\_DRAGON, S\_HI\_DRAGON | im-cold, no-stun, no-conf, no-sleep, no-hold | smart, spirit, powerful, move-body, bashes doors, evil, dragon |

## Levels 81-85

| DL | Monster | Sym | Spd | HP | AC | Exp | R | Blows | Spells | Defences | Flags |
|---:|---|:-:|---:|---:|---:|---:|---:|---|---|---|---|
| 82 | great wyrm of balance | `D` | +10 | 5200 | 255 | 33000 | 2 | claw 6d12, claw 6d12, bite 7d14, bite 7d14 | innate 1/5: BR\_CHAO 600, BR\_DISE 500, BR\_SHAR 500, BR\_SOUN 500; spells 1/5: BLIND, CONF, SCARE, S\_DRAGON, S\_HI\_DRAGON | im-fire, im-cold, im-disen, no-stun, no-conf, no-sleep, no-hold | smart, spirit, powerful, move-body, bashes doors, evil, dragon |
| 82 | great wyrm of many colours | `D` | +10 | 5200 | 255 | 33000 | 2 | claw 6d12, claw 6d12, bite 7d14, bite 7d14 | innate 1/5: BR\_ACID 1600, BR\_COLD 1600, BR\_ELEC 1600, BR\_FIRE 1600, BR\_POIS 800; spells 1/5: BLIND, CONF, SCARE, S\_DRAGON, S\_HI\_DRAGON | im-acid, im-elec, im-fire, im-cold, im-pois, no-conf, no-sleep, no-hold | smart, spirit, powerful, move-body, bashes doors, evil, dragon |
| 85 | greater Balrog | `U` | +20 | 4500 | 210 | 25000 | 3 | hit fire 8d12, hit fire 8d12, crush 9d12, touch drain\_charges | innate 1/5: WHIP 198, BR\_FIRE 1500, BR\_PLAS 150; spells 1/4: CONF, BA\_FIRE 307, BO\_PLAS 151, BA\_DARK 440, S\_HI\_DEMON, S\_UNDEAD | im-elec, im-fire, im-pois, im-plasma, no-conf, no-sleep, no-hold | spirit, powerful, move-body, opens doors, bashes doors, evil, demon |

## Levels 91-95

| DL | Monster | Sym | Spd | HP | AC | Exp | R | Blows | Spells | Defences | Flags |
|---:|---|:-:|---:|---:|---:|---:|---:|---|---|---|---|
| 95 | storm of Unmagic | `v` | +30 | 600 | 48 | 17000 | 5 | engulf exp\_80 5d7, engulf drain\_charges 5d7, engulf disenchant 5d7, engulf lose\_all 5d7 | innate 1/3: BR\_CHAO 100, BR\_DISE 100, BR\_TIME 150 | im-pois, im-disen, no-fear, no-conf, no-sleep, no-hold | empty-mind (no ESP), kill-wall, erratic 50%, powerful, bashes doors, nonliving |

## Monster bases (families)

Flags listed here are inherited by every monster of the base (`monster_base.txt`), unless removed with `flags-off:`.

| Base | Sym | Family | Non-uniques | Uniques | Depths | Inherited flags |
|---|:-:|---|---:|---:|---|---|
| ancient dragon | `D` | Ancient Dragon/Wyrm | 24 | 5 | 50-85 | DRAGON, EVIL, POWERFUL, SMART, SPIRIT, DROP\_4, MOVE\_BODY, CLEAR\_WEB, NO\_CONF, NO\_SLEEP, NO\_HOLD, FORCE\_SLEEP |
| ainu | `A` | Ainu/Maia | 9 | 5 | 34-99 | POWERFUL, SPIRIT, SMART, ONLY\_ITEM, OPEN\_DOOR, BASH\_DOOR, TAKE\_ITEM, CLEAR\_WEB, IM\_COLD, IM\_FIRE, IM\_POIS, NO\_CONF, NO\_SLEEP, NO\_HOLD, FORCE\_SLEEP |
| ant | `a` | Ant | 9 | 1 | 1-42 | ANIMAL, WEIRD\_MIND, BASH\_DOOR |
| bat | `b` | Bat | 9 | 0 | 1-32 | ANIMAL |
| bird | `B` | Bird | 6 | 1 | 2-54 | ANIMAL, CLEAR\_WEB |
| canine | `C` | Canine (Dog) | 10 | 6 | 0-99 | ANIMAL, CLEAR\_WEB |
| centipede | `c` | Centipede | 8 | 0 | 1-25 | ANIMAL, WEIRD\_MIND |
| creeping coins | `$` | Creeping Coins | 5 | 0 | 4-27 | COLD\_BLOOD, METAL, UNAWARE, EMPTY\_MIND, ONLY\_GOLD, BASH\_DOOR, CLEAR\_WEB, IM\_ELEC, IM\_POIS, NO\_CONF, NO\_SLEEP, MIMIC\_INV |
| dragon | `d` | Dragon | 29 | 1 | 9-43 | DRAGON, EVIL, SPIRIT, FORCE\_SLEEP, CLEAR\_WEB |
| dragon fly | `F` | Dragon Fly | 4 | 0 | 5-6 | ANIMAL, WEIRD\_MIND, RAND\_50, BASH\_DOOR, FORCE\_SLEEP |
| eye | `e` | Floating Eye | 10 | 1 | 1-73 | CLEAR\_WEB |
| elemental | `E` | Elemental | 16 | 4 | 12-65 | EMPTY\_MIND, CLEAR\_WEB, IM\_POIS, NO\_FEAR, NO\_CONF, NO\_SLEEP, NONLIVING |
| feline | `f` | Feline (Cat) | 6 | 1 | 0-66 | ANIMAL, CLEAR\_WEB |
| ghost | `G` | Ghost | 14 | 1 | 3-68 | UNDEAD, EVIL, INVISIBLE, COLD\_BLOOD, SPIRIT, PASS\_WALL, IM\_COLD, IM\_POIS, NO\_CONF, NO\_SLEEP, NO\_STUN, NO\_HOLD |
| giant | `P` | Giant Humanoid | 9 | 2 | 25-69 | GIANT, EVIL, SPIRIT, BASH\_DOOR, CLEAR\_WEB |
| golem | `g` | Golem | 13 | 0 | 14-75 | EMPTY\_MIND, CLEAR\_WEB, IM\_ELEC, NO\_FEAR, NO\_CONF, NO\_SLEEP, NONLIVING |
| humanoid | `h` | Hobbit/Elf/Dwarf | 11 | 11 | 0-81 | OPEN\_DOOR, BASH\_DOOR, SPIRIT, CLEAR\_WEB |
| hybrid | `H` | Hybrid | 9 | 1 | 2-68 | CLEAR\_WEB |
| hydra | `M` | Hydra | 8 | 0 | 17-57 | ANIMAL, ONLY\_GOLD, BASH\_DOOR, CLEAR\_WEB, FORCE\_SLEEP, HURT\_COLD |
| icky thing | `i` | Icky Thing | 7 | 0 | 1-14 | RAND\_50 |
| insect | `I` | Insect | 7 | 0 | 3-26 | ANIMAL, WEIRD\_MIND, MULTIPLY, RAND\_50, RAND\_25 |
| jelly | `j` | Jelly | 14 | 0 | 2-23 | EMPTY\_MIND, STUPID, CLEAR\_WEB, NO\_FEAR |
| killer beetle | `K` | Killer Beetle | 8 | 0 | 13-45 | ANIMAL, WEIRD\_MIND, BASH\_DOOR |
| kobold | `k` | Kobold | 5 | 1 | 1-7 | EVIL, OPEN\_DOOR, BASH\_DOOR, SPIRIT, CLEAR\_WEB, IM\_POIS |
| lich | `L` | Lich | 5 | 2 | 34-92 | UNDEAD, EVIL, COLD\_BLOOD, SMART, SPIRIT, OPEN\_DOOR, BASH\_DOOR, CLEAR\_WEB, IM\_COLD, IM\_POIS, NO\_CONF, NO\_SLEEP, NO\_STUN, NO\_HOLD, FORCE\_SLEEP |
| lurker | `x` | Lurker | 2 | 0 | 14-36 | COLD\_BLOOD, EMPTY\_MIND, INVISIBLE, UNAWARE, NEVER\_MOVE, NO\_FEAR, NO\_CONF, NO\_SLEEP |
| major demon | `U` | Major Demon | 14 | 4 | 40-95 | DEMON, EVIL, SPIRIT, ONLY\_ITEM, OPEN\_DOOR, BASH\_DOOR, CLEAR\_WEB, NO\_CONF, NO\_SLEEP, NO\_HOLD, FORCE\_SLEEP |
| mimic | `?` | Mimic | 4 | 0 | 18-35 | COLD\_BLOOD, EMPTY\_MIND, UNAWARE, NEVER\_MOVE, NO\_FEAR, NO\_CONF, NO\_SLEEP, FORCE\_SLEEP, ATTR\_CLEAR |
| minor demon | `u` | Minor Demon | 8 | 1 | 8-40 | DEMON, EVIL, SPIRIT, CLEAR\_WEB, IM\_FIRE |
| mold | `m` | Mold | 9 | 0 | 1-47 | NEVER\_MOVE, IM\_POIS, HURT\_FIRE, NO\_FEAR, NO\_CONF, NO\_SLEEP |
| mushroom | `,` | Mushroom | 9 | 0 | 1-32 | EMPTY\_MIND, STUPID, HURT\_FIRE, NO\_FEAR, NO\_CONF, NO\_SLEEP |
| naga | `n` | Naga | 6 | 0 | 3-48 | FEMALE, EVIL, HURT\_COLD, SPIRIT, BASH\_DOOR, CLEAR\_WEB |
| ogre | `O` | Ogre | 6 | 1 | 13-35 | GIANT, EVIL, SPIRIT, OPEN\_DOOR, BASH\_DOOR, CLEAR\_WEB |
| orc | `o` | Orc | 8 | 10 | 6-23 | MALE, ORC, EVIL, SPIRIT, OPEN\_DOOR, BASH\_DOOR, CLEAR\_WEB |
| person | `p` | Person/Human | 39 | 15 | 2-65 | BASH\_DOOR, SPIRIT, CLEAR\_WEB |
| quadruped | `q` | Quadruped | 6 | 1 | 9-39 | BASH\_DOOR, CLEAR\_WEB |
| quylthulg | `Q` | Quylthulg | 9 | 1 | 20-79 | EMPTY\_MIND, INVISIBLE, NEVER\_BLOW, NEVER\_MOVE, NO\_FEAR, NO\_CONF, NO\_SLEEP, NO\_HOLD, FORCE\_SLEEP |
| reptile | `R` | Reptile/Amphibian | 9 | 1 | 1-86 | ANIMAL, HURT\_COLD, CLEAR\_WEB |
| rodent | `r` | Rodent | 5 | 0 | 1-15 | ANIMAL, CLEAR\_WEB |
| skeleton | `s` | Skeleton | 8 | 1 | 5-84 | UNDEAD, EVIL, COLD\_BLOOD, CLEAR\_WEB, IM\_COLD, IM\_POIS, NO\_FEAR, NO\_CONF, NO\_SLEEP, NO\_STUN |
| snake | `J` | Snake | 9 | 1 | 1-99 | ANIMAL, HURT\_COLD, BASH\_DOOR, CLEAR\_WEB |
| spider | `S` | Spider/Scorpion/Tick | 14 | 2 | 2-75 | BASH\_DOOR, PASS\_WEB |
| townsfolk | `t` | Townsfolk | 11 | 0 | 0-0 | SPIRIT, CLEAR\_WEB |
| tree | `l` | Tree/Ent | 4 | 1 | 11-52 | CLEAR\_WEB, COLD\_BLOOD, REGENERATE, NO\_FEAR, NO\_CONF, HURT\_FIRE, IM\_WATER |
| troll | `T` | Troll | 12 | 4 | 17-42 | TROLL, EVIL, SPIRIT, OPEN\_DOOR, BASH\_DOOR, CLEAR\_WEB |
| vampire | `V` | Vampire | 4 | 2 | 27-99 | UNDEAD, EVIL, COLD\_BLOOD, REGENERATE, SPIRIT, OPEN\_DOOR, BASH\_DOOR, CLEAR\_WEB, HURT\_LIGHT, HURT\_FIRE, IM\_COLD, IM\_POIS, NO\_CONF, NO\_SLEEP, NO\_STUN, FORCE\_SLEEP |
| vortex | `v` | Vortex | 12 | 1 | 21-95 | EMPTY\_MIND, POWERFUL, RAND\_50, BASH\_DOOR, CLEAR\_WEB, NO\_FEAR, NO\_CONF, NO\_SLEEP, NO\_HOLD, IM\_POIS, FORCE\_SLEEP, NONLIVING |
| worm | `w` | Worm/Worm Mass | 11 | 0 | 1-40 | ANIMAL |
| wraith | `W` | Wight/Wraith | 10 | 9 | 24-85 | UNDEAD, EVIL, COLD\_BLOOD, SPIRIT, CLEAR\_WEB, HURT\_LIGHT, IM\_COLD, IM\_POIS, NO\_CONF, NO\_SLEEP, NO\_STUN, NO\_HOLD |
| xorn | `X` | Xorn/Xaren | 3 | 0 | 16-40 | COLD\_BLOOD, EMPTY\_MIND, CLEAR\_WEB, HURT\_ROCK, IM\_POIS, NO\_CONF, NO\_SLEEP |
| yeek | `y` | Yeek | 3 | 2 | 2-13 | ANIMAL, SPIRIT, OPEN\_DOOR, BASH\_DOOR, CLEAR\_WEB, IM\_ACID |
| yeti | `Y` | Yeti | 2 | 0 | 9-20 | ANIMAL, OPEN\_DOOR, BASH\_DOOR, CLEAR\_WEB, IM\_COLD |
| zephyr hound | `Z` | Zephyr Hound | 21 | 0 | 15-74 | ANIMAL, GROUP\_AI, CLEAR\_WEB |
| zombie | `z` | Zombie/Mummy | 9 | 0 | 7-38 | UNDEAD, EVIL, COLD\_BLOOD, OPEN\_DOOR, BASH\_DOOR, CLEAR\_WEB, IM\_COLD, IM\_POIS, HURT\_FIRE, NO\_CONF, NO\_SLEEP, NO\_STUN |
| player | `@` | You | 1 | 0 | 0-0 | - |
| Morgoth | `P` | Morgoth | 0 | 1 | 100-100 | - |
