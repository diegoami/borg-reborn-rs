# Appendix A3. Monster spells, breaths and threats

> **Generated file - do not edit by hand.** Produced by `docs/c-manual/tools/gen_appendices.py` from `lib/gamedata/monster.txt`, `lib/gamedata/monster_base.txt`, `lib/gamedata/monster_spell.txt`, `lib/gamedata/projection.txt`, `src/list-mon-spells.h`, `lib/gamedata/player_timed.txt`, `src/project-player.c` of the Angband source tree at commit `e473720ed`. Regenerate with `python3 docs/c-manual/tools/gen_appendices.py --src <angband-source-root>` (run from the repository root).

Every monster spell with its effects, damage formula, lore text and saving throw; the breath rules per element and the deadliest breather of each element; and the shallowest native depth at which each major threat appears. Per-monster numbers are in [A1](A1-monsters-by-depth.md) and [A2](A2-uniques.md); how to resist is in [Chapter 5](../05-elements-and-status.md).

## Contents

- [How monster spell damage works](#how-monster-spell-damage-works)
- [Breath table](#breath-table)
- [Every monster spell](#every-monster-spell)
- [Deadliest breathers by element](#deadliest-breathers-by-element)
  - [Acid (BR_ACID)](#acid-br_acid)
  - [Lightning (BR_ELEC)](#lightning-br_elec)
  - [Fire (BR_FIRE)](#fire-br_fire)
  - [Cold (BR_COLD)](#cold-br_cold)
  - [Poison (BR_POIS)](#poison-br_pois)
  - [Nether (BR_NETH)](#nether-br_neth)
  - [Light (BR_LIGHT)](#light-br_light)
  - [Dark (BR_DARK)](#dark-br_dark)
  - [Sound (BR_SOUN)](#sound-br_soun)
  - [Chaos (BR_CHAO)](#chaos-br_chao)
  - [Disenchantment (BR_DISE)](#disenchantment-br_dise)
  - [Nexus (BR_NEXU)](#nexus-br_nexu)
  - [Time (BR_TIME)](#time-br_time)
  - [Inertia (BR_INER)](#inertia-br_iner)
  - [Gravity (BR_GRAV)](#gravity-br_grav)
  - [Shards (BR_SHAR)](#shards-br_shar)
  - [Plasma (BR_PLAS)](#plasma-br_plas)
  - [Force (BR_WALL)](#force-br_wall)
  - [Mana (BR_MANA)](#mana-br_mana)
- [Shallowest appearance of major threats](#shallowest-appearance-of-major-threats)

## How monster spell damage works

- **Spell power** is the monster's native depth unless `spell-power:` overrides it (`parse_monster_depth()` / `parse_monster_spell_power()` in `src/mon-init.c`). Dice expressions use `power`; `sight` is `player:max-sight` = 20 (`lib/gamedata/constants.txt`).
- **Breaths** ignore spell power: damage = `min(current HP / divisor, cap)` (`breath_dam()` in `src/mon-spell.c`). A wounded breather is weaker.
- **Lore** text changes at `power-cutoff` values: the text of the last cutoff that is `<=` the monster's power is used (`mon_spell_lore_description()`).
- **Save**: spells with a `message-save` line allow a saving throw: if `randint0(100) < skill_save` the spell does nothing (`do_mon_spell()`).
- **Hit**: base chance for the spell to hit (100 = always; archery 50-60).
- **Frequencies**: innate spells (breaths, arrows, boulders, spit, whip, shriek, weave) use `innate-freq`, all others `spell-freq`.

## Breath table

From `projection.txt`. *Divisor* divides the breather's current HP; *Cap* is the maximum before resistance; *Resist* is the fraction let through per level of resistance (`numerator / denominator`, applied once per resistance level by `adjust_dam()` in `src/project-player.c`; immunity = 0, vulnerability = x4/3). Base elements: one permanent resist = 1/3, plus a temporary resist = 1/9. High elements use a random denominator, so one resist leaves 6/12 to 6/9 of the damage. *HP for cap* is the HP at which the cap is reached. Only elements with a monster breath spell are listed.

| Element | Spell | Divisor | Cap | Resist | HP for cap | Side effects (unresisted) |
|---|---|---:|---:|---|---:|---|
| acid | `BR_ACID` | 3 | 1600 | 1/3 | 4800 | Damages inventory (unless immune). Hitting armour halves the damage (`adjust_dam()` / `minus_ac()`). |
| lightning | `BR_ELEC` | 3 | 1600 | 1/3 | 4800 | Damages inventory (unless immune). |
| fire | `BR_FIRE` | 3 | 1600 | 1/3 | 4800 | Damages inventory (unless immune). Caster power \>= 80: chance (`randint0(dam) > 500`) each of STR drain, blindness, poison. |
| cold | `BR_COLD` | 3 | 1600 | 1/3 | 4800 | Damages inventory (unless immune). Caster power \>= 80: chance (`randint0(dam) > 500`) each of DEX drain and experience drain (blocked by hold life). |
| poison | `BR_POIS` | 3 | 800 | 1/3 | 2400 | Poisons for 10+1d(dam) turns unless resisted. Power \>= 60: chance (`randint0(dam) > 200`) of extra acid damage and of CON drain. |
| nether | `BR_NETH` | 6 | 600 | 6/(8+1d4) | 3600 | Unless resisted or hold life: drains 200 + exp/100\*life-drain% experience. Power \>= 80: chances of mana loss and losing a turn. |
| light | `BR_LIGHT` | 6 | 500 | 6/(8+1d4) | 3000 | Unresisted: blind 2+1d5; if dam \> 300 also confused. |
| dark | `BR_DARK` | 6 | 500 | 6/(8+1d4) | 3000 | Unresisted: blind 2+1d5. Power \>= 70: chances of experience drain, slowness and amnesia. |
| sound | `BR_SOUN` | 6 | 500 | 6/(8+1d4) | 3000 | Unresisted: stun 5+1d(dam/3) (max 35) unless protected from stun; if dam \> 300 also confused. |
| chaos | `BR_CHAO` | 6 | 600 | 6/(8+1d4) | 3600 | Unresisted: hallucination, confusion 10+1d20, and experience drain (blocked by hold life). |
| disenchantment | `BR_DISE` | 6 | 500 | 6/(8+1d4) | 3000 | Unresisted: disenchants an equipped item. |
| nexus | `BR_NEXU` | 6 | 400 | 6/(8+1d4) | 2400 | Unresisted: stat scramble (saving throw) and one of teleport-to (1/3), teleport level (1/4 of the rest, saving throw) or teleport 200. |
| time | `BR_TIME` | 3 | 150 | cannot be resisted | 450 | Experience drain (1/2), else drain two stats (4/5) or all stats. Cannot be resisted. |
| inertia | `BR_INER` | 6 | 200 | cannot be resisted | 1200 | Slow 4+1d4. Cannot be resisted. |
| gravity | `BR_GRAV` | 3 | 200 | cannot be resisted | 600 | Blink unless 1d127 \<= player level, slow 4+1d4, stun (unless protected). Cannot be resisted. |
| shards | `BR_SHAR` | 6 | 500 | 6/(8+1d4) | 3000 | Unresisted: cuts of 1d(dam). |
| plasma | `BR_PLAS` | 6 | 150 | cannot be resisted | 900 | Stun 5+1d(3\*dam/4) (max 35) unless protected from stun. Cannot be resisted. |
| force | `BR_WALL` | 6 | 200 | cannot be resisted | 1200 | Stun 1d20 and knock-back. Cannot be resisted. |
| mana | `BR_MANA` | 3 | 1600 | cannot be resisted | 4800 | No side effect. Cannot be resisted. |

Other projections (no breath): **water**: Confusion 5+1d5 and stun 1d40 (no resistance reduces water).; **ice**: Cold inventory damage (unless immune to cold), cuts 5d8 unless shard resistant, stun 1d15. Damage is reduced by cold resistance.; **meteors**: No side effect.; **magic missiles**: No side effect.; **holy power**: No side effect.; **arrows**: No side effect.

## Every monster spell

*Type* is from `src/list-mon-spells.h`. *Damage* gives the formula (`power` = spell power); for breaths the HP rule. *Lore by power* lists each lore text with the spell-power range it applies to. *Protection* lists what blocks the timed effects (from `fail:` lines in `player_timed.txt`).

| Spell | Type | Hit | Save | Effects | Damage | Lore by power | Protection |
|---|---|---:|:-:|---|---|---|---|
| `SHRIEK` | annoy, innate | 100 | - | wake up nearby monsters; hastes monsters in line of sight (power 25) | - | shriek for help | - |
| `WHIP` | bolt, innate | 100 | - | lash (length 2); element = `lash-type` of the first blow's effect in `blow_effects.txt`; damage = first blow + half of each other blow (`effect_handler_LASH()`) | first blow + half of each other blow (max dice) | lash you if nearby | - |
| `SPIT` | bolt, innate | 100 | - | lash (length 3); element = `lash-type` of the first blow's effect in `blow_effects.txt`; damage = first blow + half of each other blow (`effect_handler_LASH()`) | first blow + half of each other blow (max dice) | spit at you from a distance | - |
| `SHOT` | bolt, innate, archery | 50 | - | bolt of arrows for (power/8+1)d5 | (power/8+1)d5 | power 0-24: fling pebbles; power 25-49: sling lead shots; power 50+: sling seeker shots | - |
| `ARROW` | bolt, innate, archery | 50 | - | bolt of arrows for (power/8+1)d6 | (power/8+1)d6 | power 0-15: fire small arrows; power 16-31: fire arrows; power 32-47: fire large arrows; power 48+: fire seeker arrows | - |
| `BOLT` | bolt, innate, archery | 50 | - | bolt of arrows for (power/8+1)d7 | (power/8+1)d7 | power 0-24: fire small bolts; power 25-49: fire bolts; power 50+: fire seeker bolts | - |
| `BR_ACID` | breath, innate | 100 | - | breath of acid (30-degree cone) | min(HP/3, 1600) | acid | - |
| `BR_ELEC` | breath, innate | 100 | - | breath of lightning (30-degree cone) | min(HP/3, 1600) | lightning | - |
| `BR_FIRE` | breath, innate | 100 | - | breath of fire (30-degree cone) | min(HP/3, 1600) | fire | - |
| `BR_COLD` | breath, innate | 100 | - | breath of cold (30-degree cone) | min(HP/3, 1600) | frost | - |
| `BR_POIS` | breath, innate | 100 | - | breath of poison (30-degree cone) | min(HP/3, 800) | poison | - |
| `BR_NETH` | breath, innate | 100 | - | breath of nether (30-degree cone) | min(HP/6, 600) | nether | - |
| `BR_LIGHT` | breath, innate | 100 | - | breath of light (30-degree cone) | min(HP/6, 500) | light | - |
| `BR_DARK` | breath, innate | 100 | - | breath of dark (30-degree cone) | min(HP/6, 500) | darkness | - |
| `BR_SOUN` | breath, innate | 100 | - | breath of sound (30-degree cone) | min(HP/6, 500) | sound | - |
| `BR_CHAO` | breath, innate | 100 | - | breath of chaos (30-degree cone) | min(HP/6, 600) | chaos | - |
| `BR_DISE` | breath, innate | 100 | - | breath of disenchantment (30-degree cone) | min(HP/6, 500) | disenchantment | - |
| `BR_NEXU` | breath, innate | 100 | - | breath of nexus (30-degree cone) | min(HP/6, 400) | nexus | - |
| `BR_TIME` | breath, innate | 100 | - | breath of time (30-degree cone) | min(HP/3, 150) | time | - |
| `BR_INER` | breath, innate | 100 | - | breath of inertia (30-degree cone) | min(HP/6, 200) | inertia | - |
| `BR_GRAV` | breath, innate | 100 | - | breath of gravity (30-degree cone) | min(HP/3, 200) | gravity | - |
| `BR_SHAR` | breath, innate | 100 | - | breath of shards (30-degree cone) | min(HP/6, 500) | shards | - |
| `BR_PLAS` | breath, innate | 100 | - | breath of plasma (30-degree cone) | min(HP/6, 150) | plasma | - |
| `BR_WALL` | breath, innate | 100 | - | breath of force (30-degree cone) | min(HP/6, 200) | force | - |
| `BR_MANA` | breath, innate | 100 | - | breath of mana (30-degree cone) | min(HP/3, 1600) | mana | - |
| `BOULDER` | bolt, innate | 60 | - | bolt of arrows for (power/7+1)d12 | (power/7+1)d12 | throw boulders | - |
| `WEAVE` | annoy, innate | 100 | - | creates a web | - | weave webs | - |
| `BA_ACID` | ball | 100 | - | ball of acid (radius 2) for 15+1d(power\*3) | 15+1d(power\*3) | produce acid balls | - |
| `BA_ELEC` | ball | 100 | - | ball of lightning (radius 2) for 8+1d(power\*3/2) | 8+1d(power\*3/2) | produce lightning balls | - |
| `BA_FIRE` | ball | 100 | - | ball of fire (radius 2) for 10+1d(power\*7/2) | 10+1d(power\*7/2) | power 0-79: produce fire balls; power 80+: produce firestorms | - |
| `BA_COLD` | ball | 100 | - | ball of cold (radius 2) for 10+1d(power\*3/2) | 10+1d(power\*3/2) | power 0-79: produce balls of cold; power 80+: produce blizzards | - |
| `BA_POIS` | ball | 100 | - | ball of poison (radius 2) for (power/2+3)d4 | (power/2+3)d4 | power 0-59: produce poison balls; power 60+: produce storms of poison | - |
| `BA_SHAR` | ball | 100 | - | ball of shards (radius 2) for 10+1d(power\*3/2) | 10+1d(power\*3/2) | power 0-69: produce shardstorms; power 70+: invoke storms of knives | - |
| `BA_NETH` | ball | 100 | - | ball of nether (radius 2) for (power\*4)+10d10 | (power\*4)+10d10 | power 0-79: produce nether balls; power 80+: produce nether storms | - |
| `BA_WATE` | ball | 100 | - | ball of water (radius 2) for 50+1d(power\*5/2) | 50+1d(power\*5/2) | produce water balls | - |
| `BA_MANA` | ball | 100 | - | ball of mana (radius 2) for (power\*5)+10d10 | (power\*5)+10d10 | invoke mana storms | - |
| `BA_HOLY` | ball | 100 | - | ball of holy power (radius 2) for 10+1d(power\*3/2) | 10+1d(power\*3/2) | cast orbs of draining | - |
| `BA_DARK` | ball | 100 | - | ball of dark (radius 2) for (power\*4)+10d10 | (power\*4)+10d10 | power 0-69: produce balls of darkness; power 70+: invoke darkness storms | - |
| `BA_LIGHT` | ball | 100 | - | ball of light (radius 2) for 10+1d(power\*3/2) | 10+1d(power\*3/2) | power 0-69: produce balls of light; power 70+: invoke starbursts | - |
| `STORM` | ball | 100 | - | ball of water (radius 3) for 30+(power/3)d5; ball of lightning (radius 3) for 20+(power/3)d5; ball of ice (radius 3) for 20+(power/3)d5 | 30+(power/3)d5 + 20+(power/3)d5 + 20+(power/3)d5 | power 0-79: create storms; power 80+: create tempests | - |
| `DRAIN_MANA` | annoy | 100 | - | drain d(power/2+1) mana | - | drain mana | - |
| `MIND_BLAST` | direct, annoy | 100 | yes | 8d8 damage to you; +3+1d4 turns of confusion | 8d8 | cause mind blasting | PROT\_CONF |
| `BRAIN_SMASH` | direct, annoy | 100 | yes | 12d15 damage to you; +3+1d4 turns of slowness; +3+1d4 turns of confusion; +3+1d4 turns of paralysis; +7+1d8 turns of blindness | 12d15 | cause brain smashing | FREE\_ACT, PROT\_CONF, PROT\_BLIND |
| `WOUND` | direct | 100 | yes | (power/3\*2)d5 damage to you; +(power/5-10)d10 turns of wounds | (power/3\*2)d5 | power 0-15: cause light wounds; power 16-31: cause medium wounds; power 32-49: cause serious wounds; power 50-79: cause critical wounds; power 80+: cause mortal wounds | player flag ROCK |
| `BO_ACID` | bolt | 100 | - | bolt of acid for (power/3)+7d8 | (power/3)+7d8 | produce acid bolts | - |
| `BO_ELEC` | bolt | 100 | - | bolt of lightning for (power/3)+4d8 | (power/3)+4d8 | produce lightning bolts | - |
| `BO_FIRE` | bolt | 100 | - | bolt of fire for (power/3)+9d8 | (power/3)+9d8 | produce fire bolts | - |
| `BO_COLD` | bolt | 100 | - | bolt of cold for (power/3)+6d8 | (power/3)+6d8 | produce frost bolts | - |
| `BO_POIS` | bolt | 100 | - | bolt of poison for (power/3)+9d8 | (power/3)+9d8 | produce poison bolts | - |
| `BO_NETH` | bolt | 100 | - | bolt of nether for (power\*3/2+30)+5d5 | (power\*3/2+30)+5d5 | produce nether bolts | - |
| `BO_WATE` | bolt | 100 | - | bolt of water for power+10d10 | power+10d10 | produce water bolts | - |
| `BO_MANA` | bolt | 100 | - | bolt of mana for 50+1d(power\*5/2) | 50+1d(power\*5/2) | produce mana bolts | - |
| `BO_PLAS` | bolt | 100 | - | bolt of plasma for (power+10)+8d7 | (power+10)+8d7 | produce plasma bolts | - |
| `BO_ICE` | bolt | 100 | - | bolt of ice for power+6d6 | power+6d6 | produce ice bolts | - |
| `MISSILE` | bolt | 100 | - | bolt of magic missiles for (power/3)+2d6 | (power/3)+2d6 | produce magic missiles | - |
| `BE_ELEC` | ball | 100 | - | short beam of lightning (length 5) for (power\*2+30)+5d5 | (power\*2+30)+5d5 | shoot sparks of lightning | - |
| `BE_NETH` | ball | 100 | - | short beam of nether (length 10) for (power\*2+30)+5d5 | (power\*2+30)+5d5 | power 0-79: hurl lances of nether; power 80+: shoot rays of death | - |
| `SCARE` | annoy | 100 | yes | +3+1d4 turns of fear | - | terrify | PROT\_FEAR |
| `BLIND` | annoy | 100 | yes | +11+1d4 turns of blindness | - | blind | PROT\_BLIND |
| `CONF` | annoy | 100 | yes | +3+1d4 turns of confusion | - | confuse | PROT\_CONF |
| `SLOW` | annoy, haste | 100 | yes | +3+1d4 turns of slowness | - | slow | FREE\_ACT |
| `HOLD` | annoy, haste | 100 | yes | +3+1d4 turns of paralysis | - | paralyze | FREE\_ACT |
| `HASTE` | haste | 100 | - | monster gains 50 turns of fast | - | haste-self | - |
| `HEAL` | heal | 100 | - | monster heals (power\*6) HP | - | heal-self | - |
| `HEAL_KIN` | heal\_other | 100 | - | heals a nearby kin monster by (power\*6) HP | - | heal kin | - |
| `BLINK` | tactic, escape | 100 | - | teleport self up to 10 grids | - | blink-self | - |
| `TPORT` | escape | 100 | - | teleport self up to (sight\*2+5) grids | - | teleport-self | - |
| `TELE_TO` | annoy | 100 | - | teleport the target next to the caster | - | teleport to | - |
| `TELE_SELF_TO` | annoy | 100 | - | caster teleports next to the target | - | teleport toward | - |
| `TELE_AWAY` | escape | 100 | - | teleport the target away up to 100 grids | - | teleport away | - |
| `TELE_LEVEL` | escape | 100 | yes | teleport level (up or down) | - | teleport level | - |
| `DARKNESS` | annoy | 100 | - | darken the area; ball of weak darkness (radius 3) for 10 | - | create darkness | - |
| `TRAPS` | annoy | 100 | - | creates traps on adjacent grids (radius 3) | - | create traps | - |
| `FORGET` | annoy | 100 | yes | +3 turns of amnesia | - | cause amnesia | - |
| `SHAPECHANGE` | tactic | 100 | - | monster gains 5+d10 turns of changed | - | change shape | - |
| `S_KIN` | summon | 100 | - | summon 8 x similar monsters | - | summon similar monsters | - |
| `S_MONSTER` | summon | 100 | - | summon 1 x a monster | - | summon a monster | - |
| `S_MONSTERS` | summon | 100 | - | summon 8 x monsters | - | summon monsters | - |
| `S_ANIMAL` | summon | 100 | - | summon 12 x animals | - | summon animals | - |
| `S_SPIDER` | summon | 100 | - | summon 12 x spiders | - | summon spiders | - |
| `S_HOUND` | summon | 100 | - | summon 12 x hounds | - | summon hounds | - |
| `S_HYDRA` | summon | 100 | - | summon 6 x hydras | - | summon hydras | - |
| `S_AINU` | summon | 100 | - | summon 1 x ainur | - | summon an ainu | - |
| `S_DEMON` | summon | 100 | - | summon 1 x demons | - | summon a demon | - |
| `S_UNDEAD` | summon | 100 | - | summon 1 x undead | - | summon an undead | - |
| `S_DRAGON` | summon | 100 | - | summon 1 x dragons | - | summon a dragon | - |
| `S_HI_DEMON` | summon | 100 | - | summon 8 x greater demons | - | summon greater demons | - |
| `S_HI_UNDEAD` | summon | 100 | - | summon 8 x greater undead | - | summon greater undead | - |
| `S_HI_DRAGON` | summon | 100 | - | summon 8 x ancient dragons | - | summon ancient dragons | - |
| `S_WRAITH` | summon | 100 | - | summon 8 x ringwraiths | - | summon ringwraiths | - |
| `S_UNIQUE` | summon | 100 | - | summon 8 x unique monsters | - | summon uniques | - |

## Deadliest breathers by element

For each breath, the monsters with the largest breath at average HP (top 10, uniques marked U), the shallowest breather, and the number of breathers (shape-change forms without their own HP are not counted). *Max-HP breath* uses the largest possible HP roll (`avg + 4*sd`, `mon_hp(MAXIMISE)`) for non-uniques; uniques have fixed HP.

### Acid (BR_ACID)

22 breathers. Shallowest: **giant black dragon fly** (level 6, 4 damage). Cap 1600 reached at 4800 HP.

| Monster | DL | Spd | Avg HP | Breath | Max-HP breath |
|---|---:|---:|---:|---:|---:|
| great wyrm of many colours | 82 | +10 | 5200 | 1600 | 1600 |
| Ancalagon the Black (U) | 85 | +20 | 10000 | 1600 | 1600 |
| great bile wyrm | 73 | +10 | 4500 | 1500 | 1600 |
| bile demon | 61 | +10 | 2464 | 821 | 1233 |
| ancient multi-hued dragon | 53 | +10 | 2400 | 800 | 1201 |
| hezrou | 53 | -10 | 1850 | 616 | 926 |
| ancient black dragon | 51 | +10 | 1560 | 520 | 781 |
| Kavlax the Many-Headed (U) | 41 | +10 | 1300 | 433 | 433 |
| aether hound | 74 | +20 | 1230 | 410 | 616 |
| 9-headed hydra | 57 | +10 | 1200 | 400 | 601 |

### Lightning (BR_ELEC)

19 breathers. Shallowest: **baby blue dragon** (level 9, 29 damage). Cap 1600 reached at 4800 HP.

| Monster | DL | Spd | Avg HP | Breath | Max-HP breath |
|---|---:|---:|---:|---:|---:|
| sky dragon | 80 | +10 | 5000 | 1600 | 1600 |
| great wyrm of many colours | 82 | +10 | 5200 | 1600 | 1600 |
| Ancalagon the Black (U) | 85 | +20 | 10000 | 1600 | 1600 |
| great storm wyrm | 70 | +10 | 4200 | 1400 | 1600 |
| ancient multi-hued dragon | 53 | +10 | 2400 | 800 | 1201 |
| ancient blue dragon | 50 | +10 | 1500 | 500 | 752 |
| Kavlax the Many-Headed (U) | 41 | +10 | 1300 | 433 | 433 |
| aether hound | 74 | +20 | 1230 | 410 | 616 |
| mature multi-hued dragon | 43 | +5 | 1000 | 333 | 501 |
| mature blue dragon | 39 | +5 | 700 | 233 | 352 |

### Fire (BR_FIRE)

41 breathers. Shallowest: **giant salamander** (level 8, 8 damage). Cap 1600 reached at 4800 HP.

| Monster | DL | Spd | Avg HP | Breath | Max-HP breath |
|---|---:|---:|---:|---:|---:|
| Glaurung, Father of the Dragons (U) | 75 | +20 | 7500 | 1600 | 1600 |
| great wyrm of many colours | 82 | +10 | 5200 | 1600 | 1600 |
| Ancalagon the Black (U) | 85 | +20 | 10000 | 1600 | 1600 |
| The Tarrasque (U) | 86 | +20 | 8500 | 1600 | 1600 |
| Lungorthin, the Balrog of White Fire (U) | 88 | +20 | 7000 | 1600 | 1600 |
| Carcharoth, the Jaws of Thirst (U) | 90 | +20 | 8000 | 1600 | 1600 |
| Gothmog, the High Captain of Balrogs (U) | 95 | +20 | 8000 | 1600 | 1600 |
| great hell wyrm | 73 | +10 | 4500 | 1500 | 1600 |
| greater Balrog | 85 | +20 | 4500 | 1500 | 1600 |
| Smaug the Golden (U) | 62 | +10 | 4200 | 1400 | 1400 |

### Cold (BR_COLD)

24 breathers. Shallowest: **giant white dragon fly** (level 5, 4 damage). Cap 1600 reached at 4800 HP.

| Monster | DL | Spd | Avg HP | Breath | Max-HP breath |
|---|---:|---:|---:|---:|---:|
| great wyrm of many colours | 82 | +10 | 5200 | 1600 | 1600 |
| Ancalagon the Black (U) | 85 | +20 | 10000 | 1600 | 1600 |
| The Tarrasque (U) | 86 | +20 | 8500 | 1600 | 1600 |
| Huan, Wolfhound of the Valar (U) | 90 | +20 | 8000 | 1600 | 1600 |
| great ice wyrm | 70 | +10 | 4200 | 1400 | 1600 |
| Wiruin, the Maelstrom (U) | 80 | +30 | 3500 | 1166 | 1166 |
| dracolich | 55 | +10 | 3080 | 1026 | 1541 |
| gelugon | 69 | +20 | 3080 | 1026 | 1541 |
| Scatha the Worm (U) | 53 | +10 | 2500 | 833 | 833 |
| ancient multi-hued dragon | 53 | +10 | 2400 | 800 | 1201 |

### Poison (BR_POIS)

34 breathers. Shallowest: **giant green dragon fly** (level 5, 4 damage). Cap 800 reached at 2400 HP.

| Monster | DL | Spd | Avg HP | Breath | Max-HP breath |
|---|---:|---:|---:|---:|---:|
| ancient multi-hued dragon | 53 | +10 | 2400 | 800 | 800 |
| Shelob, Spider of Darkness (U) | 55 | +10 | 3500 | 800 | 800 |
| bile demon | 61 | +10 | 2464 | 800 | 800 |
| spider of Gorgoroth | 63 | +10 | 4000 | 800 | 800 |
| great swamp wyrm | 70 | +10 | 4200 | 800 | 800 |
| Glaurung, Father of the Dragons (U) | 75 | +20 | 7500 | 800 | 800 |
| Ungoliant, the Unlight (U) | 75 | +20 | 13000 | 800 | 800 |
| pit fiend | 77 | +20 | 3520 | 800 | 800 |
| great wyrm of annihilation | 80 | +10 | 5000 | 800 | 800 |
| great wyrm of many colours | 82 | +10 | 5200 | 800 | 800 |

### Nether (BR_NETH)

13 breathers. Shallowest: **shadow drake** (level 33, 44 damage). Cap 600 reached at 3600 HP.

| Monster | DL | Spd | Avg HP | Breath | Max-HP breath |
|---|---:|---:|---:|---:|---:|
| great wyrm of annihilation | 80 | +10 | 5000 | 600 | 600 |
| Carcharoth, the Jaws of Thirst (U) | 90 | +20 | 8000 | 600 | 600 |
| dracolich | 55 | +10 | 3080 | 513 | 600 |
| nightcrawler | 69 | +10 | 2440 | 406 | 600 |
| werewolf of Sauron | 63 | +10 | 2200 | 366 | 550 |
| death drake | 50 | +10 | 1950 | 325 | 488 |
| aether hound | 74 | +20 | 1230 | 205 | 308 |
| 9-headed hydra | 57 | +10 | 1200 | 200 | 300 |
| winged horror | 48 | +10 | 1013 | 168 | 254 |
| hound of Tindalos | 69 | +20 | 480 | 80 | 120 |

### Light (BR_LIGHT)

10 breathers. Shallowest: **pseudo-dragon** (level 10, 29 damage). Cap 500 reached at 3000 HP.

| Monster | DL | Spd | Avg HP | Breath | Max-HP breath |
|---|---:|---:|---:|---:|---:|
| The Phoenix (U) | 54 | +10 | 3600 | 500 | 500 |
| sky dragon | 80 | +10 | 5000 | 500 | 500 |
| Huan, Wolfhound of the Valar (U) | 90 | +20 | 8000 | 500 | 500 |
| ethereal dragon | 50 | +10 | 1950 | 325 | 488 |
| aether hound | 74 | +20 | 1230 | 205 | 308 |
| aether vortex | 60 | +20 | 420 | 70 | 106 |
| ethereal drake | 33 | +0 | 352 | 58 | 88 |
| pseudo-dragon | 10 | +0 | 176 | 29 | 44 |
| shimmering vortex | 38 | +30 | 176 | 29 | 44 |
| light hound | 15 | +0 | 21 | 3 | 6 |

### Dark (BR_DARK)

17 breathers. Shallowest: **pseudo-dragon** (level 10, 29 damage). Cap 500 reached at 3000 HP.

| Monster | DL | Spd | Avg HP | Breath | Max-HP breath |
|---|---:|---:|---:|---:|---:|
| Shelob, Spider of Darkness (U) | 55 | +10 | 3500 | 500 | 500 |
| spider of Gorgoroth | 63 | +10 | 4000 | 500 | 500 |
| Ungoliant, the Unlight (U) | 75 | +20 | 13000 | 500 | 500 |
| great wyrm of annihilation | 80 | +10 | 5000 | 500 | 500 |
| Carcharoth, the Jaws of Thirst (U) | 90 | +20 | 8000 | 500 | 500 |
| werewolf of Sauron | 63 | +10 | 2200 | 366 | 500 |
| ethereal dragon | 50 | +10 | 1950 | 325 | 488 |
| aether hound | 74 | +20 | 1230 | 205 | 308 |
| ancient spider | 48 | +10 | 1050 | 175 | 263 |
| greater basilisk | 45 | +10 | 1010 | 168 | 253 |

### Sound (BR_SOUN)

17 breathers. Shallowest: **giant gold dragon fly** (level 6, 2 damage). Cap 500 reached at 3000 HP.

| Monster | DL | Spd | Avg HP | Breath | Max-HP breath |
|---|---:|---:|---:|---:|---:|
| great wyrm of thunder | 73 | +10 | 4500 | 500 | 500 |
| great wyrm of law | 75 | +10 | 3960 | 500 | 500 |
| Wiruin, the Maelstrom (U) | 80 | +30 | 3500 | 500 | 500 |
| great wyrm of balance | 82 | +10 | 5200 | 500 | 500 |
| Meássë, the Bloody (U) | 87 | +10 | 7000 | 500 | 500 |
| Huan, Wolfhound of the Valar (U) | 90 | +20 | 8000 | 500 | 500 |
| ancient gold dragon | 51 | +10 | 1560 | 260 | 390 |
| Kavlax the Many-Headed (U) | 41 | +10 | 1300 | 216 | 216 |
| aether hound | 74 | +20 | 1230 | 205 | 308 |
| mature gold dragon | 41 | +5 | 740 | 123 | 186 |

### Chaos (BR_CHAO)

14 breathers. Shallowest: **balance drake** (level 33, 88 damage). Cap 600 reached at 3600 HP.

| Monster | DL | Spd | Avg HP | Breath | Max-HP breath |
|---|---:|---:|---:|---:|---:|
| great wyrm of chaos | 75 | +10 | 3960 | 600 | 600 |
| great wyrm of annihilation | 80 | +10 | 5000 | 600 | 600 |
| great wyrm of balance | 82 | +10 | 5200 | 600 | 600 |
| pit fiend | 77 | +20 | 3520 | 586 | 600 |
| jabberwock | 68 | +20 | 2816 | 469 | 600 |
| Kavlax the Many-Headed (U) | 41 | +10 | 1300 | 216 | 216 |
| aether hound | 74 | +20 | 1230 | 205 | 308 |
| chaos hound | 64 | +10 | 930 | 155 | 233 |
| serpent of chaos | 78 | +20 | 900 | 150 | 226 |
| storm of Unmagic | 95 | +30 | 600 | 100 | 150 |

### Disenchantment (BR_DISE)

9 breathers. Shallowest: **balance drake** (level 33, 88 damage). Cap 500 reached at 3000 HP.

| Monster | DL | Spd | Avg HP | Breath | Max-HP breath |
|---|---:|---:|---:|---:|---:|
| Ossë, Herald of Ulmo (U) | 72 | +20 | 6000 | 500 | 500 |
| great wyrm of chaos | 75 | +10 | 3960 | 500 | 500 |
| great wyrm of balance | 82 | +10 | 5200 | 500 | 500 |
| The Tarrasque (U) | 86 | +20 | 8500 | 500 | 500 |
| aether hound | 74 | +20 | 1230 | 205 | 308 |
| storm of Unmagic | 95 | +30 | 600 | 100 | 150 |
| balance drake | 33 | +0 | 528 | 88 | 132 |
| hound of Tindalos | 69 | +20 | 480 | 80 | 120 |
| chaos drake | 33 | +0 | 440 | 73 | 110 |

### Nexus (BR_NEXU)

7 breathers. Shallowest: **nexus hound** (level 27, 23 damage). Cap 400 reached at 2400 HP.

| Monster | DL | Spd | Avg HP | Breath | Max-HP breath |
|---|---:|---:|---:|---:|---:|
| dracolisk | 55 | +10 | 3080 | 400 | 400 |
| Kavlax the Many-Headed (U) | 41 | +10 | 1300 | 216 | 216 |
| aether hound | 74 | +20 | 1230 | 205 | 308 |
| greater basilisk | 45 | +10 | 1010 | 168 | 253 |
| aether vortex | 60 | +20 | 420 | 70 | 106 |
| nexus vortex | 37 | +10 | 176 | 29 | 44 |
| nexus hound | 27 | +0 | 138 | 23 | 35 |

### Time (BR_TIME)

7 breathers. Shallowest: **time vortex** (level 38, 58 damage). Cap 150 reached at 450 HP.

| Monster | DL | Spd | Avg HP | Breath | Max-HP breath |
|---|---:|---:|---:|---:|---:|
| hound of Tindalos | 69 | +20 | 480 | 150 | 150 |
| aether hound | 74 | +20 | 1230 | 150 | 150 |
| great wyrm of annihilation | 80 | +10 | 5000 | 150 | 150 |
| storm of Unmagic | 95 | +30 | 600 | 150 | 150 |
| aether vortex | 60 | +20 | 420 | 140 | 150 |
| time hound | 51 | +20 | 330 | 110 | 150 |
| time vortex | 38 | +20 | 176 | 58 | 89 |

### Inertia (BR_INER)

3 breathers. Shallowest: **inertia hound** (level 35, 32 damage). Cap 200 reached at 1200 HP.

| Monster | DL | Spd | Avg HP | Breath | Max-HP breath |
|---|---:|---:|---:|---:|---:|
| aether hound | 74 | +20 | 1230 | 200 | 200 |
| aether vortex | 60 | +20 | 420 | 70 | 106 |
| inertia hound | 35 | +0 | 193 | 32 | 48 |

### Gravity (BR_GRAV)

5 breathers. Shallowest: **gravity hound** (level 35, 64 damage). Cap 200 reached at 600 HP.

| Monster | DL | Spd | Avg HP | Breath | Max-HP breath |
|---|---:|---:|---:|---:|---:|
| Kavlax the Many-Headed (U) | 41 | +10 | 1300 | 200 | 200 |
| aether hound | 74 | +20 | 1230 | 200 | 200 |
| sky dragon | 80 | +10 | 5000 | 200 | 200 |
| aether vortex | 60 | +20 | 420 | 140 | 200 |
| gravity hound | 35 | +0 | 193 | 64 | 97 |

### Shards (BR_SHAR)

13 breathers. Shallowest: **earth hound** (level 20, 11 damage). Cap 500 reached at 3000 HP.

| Monster | DL | Spd | Avg HP | Breath | Max-HP breath |
|---|---:|---:|---:|---:|---:|
| gelugon | 69 | +20 | 3080 | 500 | 500 |
| great wyrm of law | 75 | +10 | 3960 | 500 | 500 |
| great wyrm of balance | 82 | +10 | 5200 | 500 | 500 |
| Huan, Wolfhound of the Valar (U) | 90 | +20 | 8000 | 500 | 500 |
| great crystal drake | 50 | +10 | 1950 | 325 | 488 |
| Kavlax the Many-Headed (U) | 41 | +10 | 1300 | 216 | 216 |
| aether hound | 74 | +20 | 1230 | 205 | 308 |
| balance drake | 33 | +0 | 528 | 88 | 132 |
| law drake | 33 | +0 | 440 | 73 | 110 |
| aether vortex | 60 | +20 | 420 | 70 | 106 |

### Plasma (BR_PLAS)

9 breathers. Shallowest: **plasma vortex** (level 37, 29 damage). Cap 150 reached at 900 HP.

| Monster | DL | Spd | Avg HP | Breath | Max-HP breath |
|---|---:|---:|---:|---:|---:|
| The Phoenix (U) | 54 | +10 | 3600 | 150 | 150 |
| aether hound | 74 | +20 | 1230 | 150 | 150 |
| sky dragon | 80 | +10 | 5000 | 150 | 150 |
| greater Balrog | 85 | +20 | 4500 | 150 | 150 |
| Lungorthin, the Balrog of White Fire (U) | 88 | +20 | 7000 | 150 | 150 |
| Gothmog, the High Captain of Balrogs (U) | 95 | +20 | 8000 | 150 | 150 |
| aether vortex | 60 | +20 | 420 | 70 | 106 |
| plasma hound | 51 | +10 | 330 | 55 | 83 |
| plasma vortex | 37 | +10 | 176 | 29 | 44 |

### Force (BR_WALL)

5 breathers. Shallowest: **impact hound** (level 35, 32 damage). Cap 200 reached at 1200 HP.

| Monster | DL | Spd | Avg HP | Breath | Max-HP breath |
|---|---:|---:|---:|---:|---:|
| Baphomet the Minotaur Lord (U) | 58 | +20 | 4700 | 200 | 200 |
| aether hound | 74 | +20 | 1230 | 200 | 200 |
| sky dragon | 80 | +10 | 5000 | 200 | 200 |
| aether vortex | 60 | +20 | 420 | 70 | 106 |
| impact hound | 35 | +0 | 193 | 32 | 48 |

### Mana (BR_MANA)

No monster has this breath.

## Shallowest appearance of major threats

The shallowest native depth of any monster (non-unique and unique separately) with each ability. Monsters can appear out of depth, so treat these as the point where the threat becomes *routine*, not the first possible encounter.

| Threat | DL | Shallowest non-unique | DL | Shallowest unique | # monsters |
|---|---:|---|---:|---|---:|
| breathes acid (BR\_ACID) | 6 | giant black dragon fly | 41 | Kavlax the Many-Headed | 23 |
| breathes lightning (BR\_ELEC) | 9 | baby blue dragon | 41 | Kavlax the Many-Headed | 19 |
| breathes fire (BR\_FIRE) | 8 | giant salamander | 41 | Kavlax the Many-Headed | 42 |
| breathes cold (BR\_COLD) | 5 | giant white dragon fly | 41 | Kavlax the Many-Headed | 24 |
| breathes poison (BR\_POIS) | 5 | giant green dragon fly | 55 | Shelob, Spider of Darkness | 35 |
| breathes nether (BR\_NETH) | 33 | shadow drake | 90 | Carcharoth, the Jaws of Thirst | 14 |
| breathes light (BR\_LIGHT) | 10 | pseudo-dragon | 54 | The Phoenix | 10 |
| breathes dark (BR\_DARK) | 10 | pseudo-dragon | 55 | Shelob, Spider of Darkness | 17 |
| breathes sound (BR\_SOUN) | 6 | giant gold dragon fly | 41 | Kavlax the Many-Headed | 17 |
| breathes chaos (BR\_CHAO) | 33 | balance drake | 41 | Kavlax the Many-Headed | 15 |
| breathes disenchantment (BR\_DISE) | 33 | balance drake | 72 | Ossë, Herald of Ulmo | 9 |
| breathes nexus (BR\_NEXU) | 27 | nexus hound | 41 | Kavlax the Many-Headed | 7 |
| breathes time (BR\_TIME) | 38 | time vortex | - | - | 7 |
| breathes inertia (BR\_INER) | 35 | inertia hound | - | - | 3 |
| breathes gravity (BR\_GRAV) | 35 | gravity hound | 41 | Kavlax the Many-Headed | 5 |
| breathes shards (BR\_SHAR) | 20 | earth hound | 41 | Kavlax the Many-Headed | 13 |
| breathes plasma (BR\_PLAS) | 37 | plasma vortex | 54 | The Phoenix | 9 |
| breathes force (BR\_WALL) | 35 | impact hound | 58 | Baphomet the Minotaur Lord | 5 |
| breathes mana (BR\_MANA) | - | - | - | - | 0 |
| casts HOLD | 13 | druid | 19 | Old Man Willow | 54 |
| casts BLIND | 2 | apprentice | 13 | Boldor, King of the Yeeks | 111 |
| casts CONF | 2 | apprentice | 10 | Orfax, Son of Boldor | 102 |
| casts SCARE | 2 | acolyte | 27 | Mîm, Betrayer of Turin | 140 |
| casts SLOW | 4 | tamer | 8 | Wormtongue, Agent of Saruman | 37 |
| casts FORGET | 28 | mind flayer | 60 | Saruman of Many Colours | 18 |
| casts DRAIN\_MANA | 3 | radiation eye | 67 | Thuringwethil, the Vampire Messenger | 35 |
| casts MIND\_BLAST | 27 | vampire | 17 | Nár, the Dwarf | 15 |
| casts BRAIN\_SMASH | 28 | mind flayer | 59 | Fundin Bluecloak | 30 |
| casts WOUND | 3 | kobold shaman | 17 | Nár, the Dwarf | 80 |
| casts TELE\_TO | 10 | tengu | 10 | Orfax, Son of Boldor | 40 |
| casts TELE\_SELF\_TO | 23 | phase spider | 51 | Harowen the Black Hand | 7 |
| casts TELE\_AWAY | 28 | mage | 28 | Draebor, the Imp | 9 |
| casts TELE\_LEVEL | 16 | quasit | 28 | Draebor, the Imp | 9 |
| casts DARKNESS | 7 | eastern dwarf | 49 | Eöl, the Dark Elf | 38 |
| casts TRAPS | 27 | ogre shaman | 8 | Wormtongue, Agent of Saruman | 11 |
| casts HEAL | 2 | acolyte | 0 | Father Christmas | 42 |
| casts HASTE | 4 | scout | 60 | Saruman of Many Colours | 18 |
| casts BLINK | 2 | apprentice | 0 | Father Christmas | 49 |
| casts TPORT | 7 | lost soul | 13 | Boldor, King of the Yeeks | 26 |
| casts SHAPECHANGE | 13 | druid | 28 | Beorn, the Shape-Changer | 13 |
| casts BO\_MANA | 43 | Maia of Nienna | 41 | Gorlim, Betrayer of Barahir | 23 |
| casts BA\_MANA | 74 | black reaver | 65 | The Mouth of Sauron | 5 |
| casts BA\_NETH | 61 | nightwing | 49 | Eöl, the Dark Elf | 24 |
| casts BA\_DARK | 67 | beholder hive-mother | 65 | The Mouth of Sauron | 9 |
| casts BA\_WATE | 65 | Maia of Ulmo | 43 | Waldern, King of Water | 7 |
| casts BE\_NETH | 54 | nether wraith | 83 | Hoarmurath of Dir | 3 |
| casts STORM | 49 | storm giant | 60 | Gilim, the Giant of Eruman | 5 |
| casts BO\_PLAS | 37 | magma elemental | 43 | Vargo, Tyrant of Fire | 12 |
| casts BO\_ICE | 37 | ice elemental | 38 | Castamir the Usurper | 10 |
| summons (S\_KIN) | 12 | priest | 13 | Boldor, King of the Yeeks | 27 |
| summons (S\_MONSTER) | 11 | drúadan mage | 10 | Orfax, Son of Boldor | 19 |
| summons (S\_MONSTERS) | 35 | silent watcher | 36 | Lorgan, Chief of the Easterlings | 21 |
| summons (S\_ANIMAL) | 13 | druid | 59 | Fundin Bluecloak | 13 |
| summons (S\_SPIDER) | 33 | drúadan druid | 79 | Qlzqqlzuup, the Emperor Quylthulg | 5 |
| summons (S\_HOUND) | 52 | Maia of Oromë | 79 | Qlzqqlzuup, the Emperor Quylthulg | 8 |
| summons (S\_HYDRA) | 48 | gorgon | 79 | Qlzqqlzuup, the Emperor Quylthulg | 3 |
| summons (S\_AINU) | 60 | Maia of Yavanna | 72 | Ossë, Herald of Ulmo | 6 |
| summons (S\_DEMON) | 36 | bodak | 49 | Eöl, the Dark Elf | 19 |
| summons (S\_UNDEAD) | 25 | necromancer | 49 | Eöl, the Dark Elf | 29 |
| summons (S\_DRAGON) | 40 | enchantress | 49 | Eöl, the Dark Elf | 13 |
| summons (S\_HI\_DEMON) | 69 | gelugon | 65 | The Mouth of Sauron | 20 |
| summons (S\_HI\_UNDEAD) | 64 | archlich | 65 | The Mouth of Sauron | 18 |
| summons (S\_HI\_DRAGON) | 71 | greater draconic quylthulg | 60 | Saruman of Many Colours | 17 |
| summons (S\_WRAITH) | - | - | 77 | Uvatha the Horseman | 10 |
| summons (S\_UNIQUE) | - | - | 79 | Qlzqqlzuup, the Emperor Quylthulg | 4 |
| melee ACID | 2 | green worm mass | 42 | Rogrog the Black Troll | 33 |
| melee BLACK\_BREATH | - | - | 77 | Uvatha the Horseman | 9 |
| melee BLIND | 7 | bloodshot eye | 51 | Harowen the Black Hand | 15 |
| melee COLD | 1 | giant white centipede | 53 | Scatha the Worm | 19 |
| melee CONFUSE | 1 | grey mushroom patch | 43 | Ariel, Queen of Air | 30 |
| melee DISENCHANT | 5 | disenchanter eye | 24 | Ibun, Son of Mîm | 20 |
| melee DRAIN\_CHARGES | 34 | lich | 50 | The Balrog of Moria | 22 |
| melee EAT\_FOOD | 2 | blubbering icky thing | - | - | 4 |
| melee EAT\_GOLD | 0 | filthy street urchin | 3 | Sméagol | 8 |
| melee EAT\_ITEM | 0 | squint-eyed rogue | 51 | Harowen the Black Hand | 5 |
| melee EAT\_LIGHT | 3 | silver jelly | - | - | 2 |
| melee ELEC | 3 | metallic blue centipede | 82 | Pazuzu, Lord of Air | 17 |
| melee EXP\_10 | 12 | grape jelly | - | - | 7 |
| melee EXP\_20 | 24 | banshee | - | - | 11 |
| melee EXP\_40 | 24 | vampire bat | 81 | Ji Indur Dawndeath | 17 |
| melee EXP\_80 | 37 | emperor wight | 67 | Thuringwethil, the Vampire Messenger | 28 |
| melee FIRE | 2 | salamander | 43 | Vargo, Tyrant of Fire | 44 |
| melee HALLU | 15 | magic mushroom patch | - | - | 5 |
| melee LOSE\_ALL | 95 | storm of Unmagic | 100 | Morgoth, Lord of Darkness | 2 |
| melee LOSE\_CON | 6 | purple mushroom patch | - | - | 7 |
| melee LOSE\_DEX | 3 | yellow worm mass | 66 | Tevildo, Prince of Cats | 11 |
| melee LOSE\_INT | 28 | mind flayer | 73 | Omarax, the Eye Tyrant | 9 |
| melee LOSE\_STR | 3 | radiation eye | 55 | Shelob, Spider of Darkness | 20 |
| melee LOSE\_WIS | 5 | rot jelly | - | - | 8 |
| melee PARALYZE | 1 | floating eye | 19 | Old Man Willow | 26 |
| melee POISON | 1 | white worm mass | 28 | Draebor, the Imp | 74 |
| melee SHATTER | 60 | Maia of Aulë | 43 | Quaker, Master of Earth | 4 |
| melee TERRIFY | 2 | yellow mushroom patch | 79 | Akhorahil the Blind | 21 |
| flag INVISIBLE | 1 | clear icky thing | 3 | Sméagol | 62 |
| flag PASS\_WALL | 3 | poltergeist | 43 | Quaker, Master of Earth | 33 |
| flag KILL\_WALL | 16 | umber hulk | 76 | Makar, the Warrior | 7 |
| flag MULTIPLY | 1 | giant white mouse | - | - | 23 |
| flag EMPTY\_MIND | 1 | clear icky thing | 19 | Old Man Willow | 117 |
| flag WEIRD\_MIND | 1 | giant white centipede | 42 | The Queen Ant | 63 |
| flag COLD\_BLOOD | 3 | poltergeist | 19 | Old Man Willow | 122 |
| flag TAKE\_ITEM | 0 | aimless-looking merchant | 3 | Sméagol | 80 |
| flag KILL\_ITEM | 33 | fire elemental | 43 | Ariel, Queen of Air | 15 |
| flag IM\_NETHER | 12 | nether worm mass | 90 | Carcharoth, the Jaws of Thirst | 14 |
| flag IM\_DISEN | 5 | disenchanter eye | 72 | Ossë, Herald of Ulmo | 13 |
| flag IM\_NEXUS | 27 | nexus hound | 41 | Kavlax the Many-Headed | 8 |
| speed +10 or more | 0 | village idiot | 2 | Fang, Farmer Maggot's Dog | 305 |
| speed +20 or more | 3 | poltergeist | 3 | Sméagol | 79 |
| speed +30 or more | 15 | magic mushroom patch | 51 | Harowen the Black Hand | 11 |
