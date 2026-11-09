# Chapter 4 — Resistances, Afflictions, Curses and Traps

*Angband 4.2.6 (C). Every number in this chapter was read from the game data
files in `lib/gamedata/` or the C source in `src/`. Where the official manual
(`docs/attack.rst`, `docs/playing.rst`) disagrees with the code, the code is
quoted and the disagreement is flagged.*

Primary sources used:

| Topic | Data file | Code |
|---|---|---|
| Elements, resist fractions, breath divisors and caps | `projection.txt` | `project-player.c` (`adjust_dam`, per-element handlers), `mon-spell.c` (`breath_dam`) |
| Breath geometry | `monster_spell.txt` (`BREATH:X:0:30`) | `effect-handler-attack.c` (`effect_handler_BREATH`), `project.c` (damage falloff) |
| Monster spells | `monster_spell.txt` | `mon-spell.c` (`do_mon_spell`, saving throws) |
| Timed effects | `player_timed.txt` | `player-timed.c`, `player-calcs.c`, `game-world.c` |
| Melee blow effects | `blow_effects.txt`, `blow_methods.txt` | `mon-blows.c`, `mon-attack.c` |
| Curses | `curse.txt` | `obj-curse.c`, `obj-make.c`, `effect-handler-general.c` (`uncurse_object`) |
| Traps | `trap.txt` | `trap.c`, `cmd-cave.c` (disarming) |
| Constants | `constants.txt` (`life-drain:2`, `food-value:100`) | |
| Monsters cited as evidence | `monster.txt` | `mon-make.c` (`mon_hp`) |

Throughout, "turn" means a **player turn at normal speed** (10 game ticks). All
timed effects tick down once per 10 game ticks regardless of your own speed
(`process_world()` runs `if (!(turn % 10))`), so being Hasted does not shorten
paralysis or poison in real terms — it only gives you more actions while the
clock runs. "DL" = dungeon level, "clvl" = character level, "HP" = hit points.

---

## Part 1 — How resistance works (the engine)

### 1.1 The single number: `res_level`

Every element the player can be hit by has a per-character `res_level`
(`player->state.el_info[elem].res_level`). It is an integer:

| `res_level` | Meaning | How you get it |
|---|---|---|
| **3** | Immune | An item with `IM_ACID/IM_ELEC/IM_FIRE/IM_COLD` (only these four immunities exist) |
| **2** | Double resist | One permanent resist (equipment, race, shape) **plus** the matching temporary resist (`OPP_ACID/ELEC/FIRE/COLD/POIS`) |
| **1** | Resist | Any permanent resist, *or* a temporary resist alone |
| **0** | Normal | — |
| **−1** | Vulnerable | An item with a negative resist value (e.g. the curses *burning up*, *chilled to the bone*, *steelskin*), a race/shape vulnerability; Necromancers (`PF_EVIL`) are vulnerable to holy orbs |

How the number is built (`player-calcs.c::calc_bonuses`):

1. Race and shape set a starting value.
2. Each worn item raises it to the **maximum** of what the items offer — two
   items with rFire are no better than one (`if (obj->el_info[j].res_level > state->el_info[j].res_level) ...`).
3. Any item/race/shape vulnerability then subtracts 1, **unless** you are
   already immune (`if (vuln[i] && res_level < 3) res_level--`).
4. Finally each active timed resistance adds 1, but only while the value is
   below 2 (`if (... res_level < 2) res_level++`).

Consequences you can exploit:

* **Permanent + temporary = double resist** for acid, lightning, fire, cold and
  poison (the only elements that have timed resist effects: `OPP_ACID`,
  `OPP_ELEC`, `OPP_FIRE`, `OPP_COLD`, `OPP_POIS`).
* Two temporary sources do not stack (the potion only extends the duration).
* **Temporary resist cannot make you immune**, the loop stops at 2.
* A vulnerability is cancelled by a permanent resist (net 0), and a
  temporary resist then lifts you to 1. *Manual discrepancy:* `attack.rst`
  claims "a character can not gain a temporary resistance to an element when a
  vulnerability to that element is present". In 4.2.6 code a temporary resist
  **does** raise −1 to 0, or 0 to 1.
* The high elements (light, dark, sound, shards, nexus, nether, chaos,
  disenchantment) can never go above 1: there are no timed effects for them.
  The one partial exception is `OPP_CONF` ("RConf"), which is a timed version
  of *protection from confusion* (a flag, not an element).
* Race/class freebies: Elf and High-Elf `RES_LIGHT`, Half-Orc `RES_DARK`,
  Kobold `RES_POIS`. Necromancers have the player flags `UNLIGHT` and `EVIL`:
  in the dungeon their dark res_level is forced to 1, nether to 1 and holy orb
  to −1 (set *after* equipment is counted, so gear can't
  raise these further).

### 1.2 `adjust_dam`: the damage formula

The full player-side resistance code (`project-player.c::adjust_dam`), in
order:

```
if resist == 3          -> damage 0                     (immune)
if type == ACID and minus_ac() hits an armour piece with AC>0 -> dam = (dam+1)/2
if resist == -1         -> return dam * 4 / 3           (vulnerable)
denom = roll(projection.denominator)
repeat resist times:    dam = dam * numerator / denom
```

The numerator/denominator come from `projection.txt`:

| Group | Elements | `numerator` / `denominator` | One resist | Two resists |
|---|---|---|---|---|
| Base five | acid, elec, fire, cold, poison | 1 / 3 | **×1/3** | **×1/9** |
| Ice | ice (uses your **cold** resist level) | 1 / 3 | ×1/3 | ×1/9 |
| High elements | light, dark, sound, shards, nexus, nether, chaos, disenchantment | 6 / (8+1d4) | **×6/9 to ×6/12** (66.7% → 50%; mean ≈ 57.8%) | n/a (cannot reach 2) |
| Unresistable | water, gravity, inertia, force, time, plasma, meteor, missile, mana, holy orb, arrow | none (denominator 0) | — | — |

Notes:

* The high-element denominator is re-rolled for every hit. The worst case for
  you is 6/9 = **two thirds** of the damage, the best is one half. Plan around
  two thirds.
* The `denom` roll applies once per resist level, so double resist on base
  elements really is 1/9 (`dam*1/3*1/3`, integer division each time).
* **Acid is special**: if the randomly chosen armour slot holds an item with
  `ac + to_a > 0`, the acid damage is halved *before* resistance, and that piece
  loses 1 point of `to_a` unless it ignores acid. With a full armour set this
  happens almost every time, so acid is effectively the weakest base element.
* Vulnerability: ×4/3, and it *replaces* resistance math (a vulnerable
  character has `res_level` −1 and so no resist division is applied).
* Unresistable elements can still be **vulnerable** in principle (the
  vulnerability check is generic) — in practice only Necromancers vs. holy orbs.
* After `adjust_dam`, `player_apply_damage_reduction` subtracts the flat
  `DAM_RED` modifier and percentage reduction, and the rare `INVULN` timed
  effect (`TMD_INVULN`) zeroes any hit under 9000.
  Damage reduction does **not** reduce side effects — those are computed from
  the post-resist, pre-reduction damage.

### 1.3 Side effects use the *resisted* damage

`project_p()` calls `adjust_dam()` first, stores the result in `context.dam`,
applies it, and **then** runs the element handler with that already-reduced
number. So every damage-scaled side effect (cut length, stun length, poison
length, "if randint0(dam) > 500" checks) is computed from resisted damage.
Resistance therefore protects twice: less HP loss and weaker side effects.
For the high elements the handler additionally returns early with "You resist
the effect!" when you resist — no side effect at all.

### 1.4 Spell power and "powerful" side effects

`project()` passes the caster's `spell_power` into the handler (for monsters,
`race->spell_power`, which defaults to the monster's depth). Monsters with the
`POWERFUL` flag have their power raised to at least 80 for this purpose
(`project.c`: `if (monster_is_powerful(mon)) power = MAX(power, 80)`). Several
elements have extra effects only above a power threshold (fire/cold ≥80,
poison ≥60, dark ≥70, nether ≥80). Traps pass power 0 and never trigger these.

### 1.5 Distance falloff

* **Breaths** (`effect_handler_BREATH`): monster breaths are 30° arcs. The
  energy-source diameter starts at 4, ×1.5 for `POWERFUL` monsters, then
  ×60/30 for the narrow arc: 8 (normal) or 12 (powerful). Damage at distance
  *d* is `min(dam, diameter × dam / (d+1))`. So **a normal breath is full
  strength out to 7 squares, a powerful breath out to 11**. At distance 15 a
  normal breath does 50%; at 19 (near max range 20) 40%.
* **Balls** centred on you deal full damage; a ball centred one square away
  deals `(dam+1)/2`.
* **Breath uses the monster's *current* HP** (`breath_dam(type, mon->hp)`).
  Wounding a dragon before it breathes is a real defence.

### 1.6 Inventory damage

`inven_damage(p, type, MIN(dam*5, 300))` — the chance is in hundredths of a
percent, so the cap is **3% per item per hit**, reached at **60 post-resist
damage**. Implications:

* Resistance barely protects your pack against big breaths: a resisted
  1600-point fire breath still deals 533 and still rolls the 3% maximum.
  Only **immunity** (handler returns before calling `inven_damage`) protects
  the pack completely.
* Each item in a stack rolls separately (a stack of 20 potions loses on
  average 0.6 potions per cold hit at the cap).
* Equipped items and artifacts are never destroyed by this routine.
* Weapons and armour *in the pack* that hate the element are not destroyed;
  they lose 1 to-hit and 1 to-dam (weapons) or 1 to-AC (armour). *Manual
  discrepancy:* `attack.rst` says pack weapons/armour "cannot be harmed".
* Rods get a quarter of the chance.
* Which items hate what (`object_base.txt`):

| Element that damages the pack | Item classes affected |
|---|---|
| Acid | chests, arrows, bolts, bows, hafted, polearms, swords, boots, gloves, helms, crowns, shields, cloaks, soft & hard armour, staves, scrolls |
| Fire | chests, arrows, bows, hafted, polearms, boots, gloves, cloaks, soft armour, light sources, staves, scrolls, **all spellbooks** |
| Lightning | rings, wands, rods |
| Cold (and ice) | **potions, flasks of oil** |

Potions also "hate" sound, shards and force, but the player handlers for those
elements never call `inven_damage` — those only destroy potions lying on the
floor. Amulets hate nothing.

### 1.7 Learning runes

Every time `adjust_dam` runs with `actual = true` the game calls
`equip_learn_element`, so taking a hit of an element teaches you whether your
gear resists it. Taking a fire bolt is the cheapest way to identify an
unknown rFire rune.

---

## Part 2 — Element-by-element reference

### 2.1 Master table

| Element | Resist / protection | Resist factor | Breath HP divisor | Breath cap | Unresisted side effects (player) |
|---|---|---|---|---|---|
| Acid | rAcid, imAcid | 1/3, double 1/9; halved by armour first | 3 | 1600 | pack damage; armour −1 AC |
| Lightning | rElec, imElec | 1/3, 1/9 | 3 | 1600 | pack damage (rings, wands, rods) |
| Fire | rFire, imFire | 1/3, 1/9 | 3 | 1600 | pack damage; power ≥80: STR drain, blind, poison |
| Cold | rCold, imCold | 1/3, 1/9 | 3 | 1600 | pack damage (potions!); power ≥80: DEX drain, exp drain |
| Poison | rPois (no immunity) | 1/3, 1/9 | 3 | **800** | poisoned 10+1d(dam) turns; power ≥60: acid sting, CON drain |
| Light | rLight | 6/(8+1d4) | 6 | 500 | blind 2+1d5; >300 dam: confused |
| Dark | rDark | 6/(8+1d4) | 6 | 500 | blind 2+1d5; power ≥70: exp drain, slow, amnesia |
| Sound | rSound | 6/(8+1d4) | 6 | 500 | stun 5+1d(dam/3) (max 35) unless pStun; >300 dam: confused |
| Shards | rShards | 6/(8+1d4) | 6 | 500 | cut 1d(dam) |
| Nexus | rNexus | 6/(8+1d4) | 6 | **400** | stat scramble (save), teleport-to / teleport-level (save) / teleport 200 |
| Nether | rNether (Hold Life blocks the drain) | 6/(8+1d4) | 6 | 600 | exp drain 200 + 2% of exp; power ≥80: mana loss, energy loss |
| Chaos | rChaos | 6/(8+1d4) | 6 | 600 | hallucinate 1d10, confused 10+1d20−1, exp drain 3% of exp |
| Disenchantment | rDisen | 6/(8+1d4) | 6 | 500 | one equipped item loses +hit/+dam or +AC |
| Water | none | — | 6 | 0 (no breath) | confused 5+1d5, stun 1d40 |
| Ice | uses cold resist | 1/3, 1/9 | 6 | 0 (no breath) | pack cold damage, cut 5d8 unless rShards, stun 1d15 |
| Gravity | none | — | 3 | **200** | blink (unless 1d127 ≤ clvl), slow 4–7 (ignores FA), stun unless pStun |
| Inertia | none | — | 6 | 200 | slow 4–7 (ignores Free Action) |
| Force | none | — | 6 | 200 | stun 1d20, knocked back 3+dam/20 squares |
| Time | none | — | 3 | **150** | 50% exp drain (ignores Hold Life), 40% two random stats, 10% all stats (ignores sustains) |
| Plasma | none | — | 6 | 150 | stun 5+1d(3·dam/4) (max 35) unless pStun |
| Meteor | none | — | 6 | 0 | none |
| Missile | none | — | 6 | 0 | none |
| Mana | none | — | 3 | 1600 | none (no monster in 4.2.6 has `BR_MANA`) |
| Holy orb | none (Necromancer vulnerable) | — | 6 | 0 | none |
| Arrow | none | — | 6 | 0 | none |

A cap of 0 means no monster breathes that element (it is only used for bolts,
balls or traps). Water, ice, meteor, missile, holy orb and arrow damage comes
from spells, not breath.

### 2.2 Acid

* Handler: if not immune, `inven_damage(ACID, min(dam*5,300))`.
* Damage is halved by armour (see 1.2) and the struck armour loses 1 point of
  enchantment unless it has `IGNORE_ACID`.
* Defence: rAcid, imAcid; temporary resist (Ring of Acid activation, Mage
  *Resistance*). In practice acid is the least dangerous base element; even a
  1600 great-wyrm breath is 800 after armour, 266 resisted, 88 double-resisted.

### 2.3 Lightning

* Handler: pack damage only. Destroys **rings, wands and rods** in the pack.
* There is no armour halving, so 1600 elec from a great storm wyrm or sky
  dragon is 1600 unresisted, 533 resisted, 177 double.
* The curse *steelskin* (+20 AC, rShards) gives `RES_ELEC[-1]`, i.e. elec
  vulnerability: ×4/3.

### 2.4 Fire

* Pack damage: scrolls, staves, books, arrows, light sources, many armours.
* If the caster's power is ≥80, each of the following happens independently
  when `randint0(dam) > 500` — only possible when the **post-resist** damage
  exceeds 500, so essentially only for unresisted big breaths:
  * "The intense heat saps you": STR drain (blocked by Sustain STR).
  * "Your eyes fill with smoke!": blind for 1d(dam/100) (blocked by pBlind).
  * "You are assailed by poisonous fumes!": poisoned 1d(dam/10) (blocked by rPois).
* A resisted 1600 breath is 533: `randint0(533) > 500` happens ~6% of the time
  per side effect. Double resist (177) makes them impossible.

### 2.5 Cold

* Pack damage: **potions and flasks of oil**. This is the element that
  destroys your Cure Critical Wounds stack.
* Power ≥80 and `randint0(dam) > 500`: DEX drain (Sustain DEX blocks) and
  "The cold withers your life force!" — experience loss equal to the damage
  (Hold Life blocks).

### 2.6 Poison

* Damage resistible by rPois / `OPP_POIS`; there is **no poison immunity** for
  the player and the breath cap is **800**.
* Status: poisoned for `10 + 1d(dam)` turns unless the `POISONED` timed effect
  fails (fails on rPois or while `OPP_POIS` is active). An unresisted 800-point
  breath can poison you for up to 810 turns: 1 HP per turn and no natural
  regeneration.
* Power ≥60, each when `randint0(dam) > 200`:
  * "The venom stings your skin!": extra acid damage of dam/5 (acid resist
    applies; also rolls pack acid damage).
  * "The stench sickens you.": CON drain (Sustain CON blocks).
  * **These run even if you resist poison** (the handler only checks the
    timed effect). A resisted 800 breath (266) still has ~25% chance of each.
* Monster melee `POISON` blows also inflict 5+1d(monster level) turns of
  poison.

### 2.7 Light

* rLight → "You resist the effect!", no side effect.
* Otherwise blind 2+1d5 (pBlind prevents) and, if the damage exceeds 300,
  confused 2+1d(dam/100) (pConf prevents). Only an unresisted breath can
  exceed 300 (cap is 500).
* Also: blinding-flash traps and the player's own `*Destruction*`-style light
  blasts blind for 10+1d10 unless rLight.

### 2.8 Darkness

* rDark → no side effects. Otherwise blind 2+1d5 (pBlind prevents).
* If caster power ≥70, each independently:
  * `randint0(dam) > 100`: exp drain equal to the damage (Hold Life blocks).
  * `randint0(dam) > 200`: slowed dam/100 turns — applied with `check=false`,
    so **Free Action does not stop it**.
  * `randint0(dam) > 300`: amnesia dam/100 turns (nothing prevents amnesia).
* Unresisted darkness from Ungoliant, Carcharoth, the greater Balrogs and
  Sauron's `BA_DARK` (power 70+) carries all three.
* Monster `DARKNESS` spell and `DARK_WEAK`: blind 3+1d5 unless rDark.

### 2.9 Sound

* rSound → nothing. Otherwise stun for `5 + 1d(dam/3)` turns, **capped at 35
  per hit**, unless pStun. Stun accumulates across hits; see Part 5 for the
  stun grades (Heavy Stun at 51, **Knocked Out above 150**).
* Damage > 300 also confuses 2+1d(dam/100) (pConf prevents).
* Five capped sound breaths (5×35 = 175) knock you out. Vibration hounds
  (DL27, packs of 2d7) are the first serious source.

### 2.10 Shards

* rShards → nothing. Otherwise cut for 1d(dam) — a 500 breath can inflict up
  to a 500-point cut ("Deep Gash", 3 HP/turn).
* Ice bolts also cut (5d8) unless rShards.

### 2.11 Nexus

* rNexus → nothing at all (no scramble, no teleport).
* Without it:
  1. Stat scramble: saving throw (`randint0(100) < skill_save`) avoids it;
     otherwise `SCRAMBLE` for 20+1d20−1 turns (your stat values are shuffled,
     restored when it ends).
  2. Then one of: 1/3 teleport-to the breathing monster; else 1/4
     teleport-level (saving throw avoids); else a normal teleport of 200
     squares.
* rNexus is also the only item defence against the `TELE_LEVEL` monster spell
  (`effect_handler_TELEPORT_LEVEL`: "You resist the effect!" when the source is
  a monster and you resist nexus).

### 2.12 Nether

* Blocked completely (damage aside) by **either** rNether **or** Hold Life.
* Otherwise: exp drain of `200 + (exp/100) × 2` — i.e. 200 + 2% of current
  experience (`life-drain:2` in `constants.txt`).
* Power ≥80 (and only reached if not blocked): `randint0(dam) > 100` drains
  dam/10 mana; `randint0(dam) > 200` sets your energy to 0 (you lose your next
  move).
* Nether is a damage problem more than a drain problem late: Sauron's
  `BA_NETH` is 530–620 and Morgoth's 410–500. rNether reduces those to two
  thirds or less.

### 2.13 Chaos

* rChaos → nothing.
* Otherwise: hallucination 1d10 (the `IMAGE` effect itself only fails on
  rChaos), confusion 10+1d20−1 (pConf prevents), and exp drain of 3% of
  current experience (Hold Life prevents).
* Chaos is the classic "nasty" unresisted breath: confusion prevents reading
  scrolls and casting, hallucination hides what is attacking you.

### 2.14 Disenchantment

* rDisen → nothing.
* Otherwise `EF_DISENCHANT`: one random equipment slot that is not ring,
  amulet or light is picked. Weapons/bows lose 1 to-hit and 1 to-dam (plus a
  20% chance of another point if above +5), armour loses 1 to-AC (same 20%
  rule). **Artifacts resist 60% of the time.**
* Melee `DISENCHANT` blows apply the same effect after damage.

### 2.15 Water (spells only)

* No resistance. Confused 5+1d5 (pConf prevents) and stunned 1d40 (pStun
  prevents — the call uses `check=true`).
* `BA_WATE` (50+1d(5·power/2)) from Ossë, Maia of Ulmo, Mouth of Sauron;
  `BO_WATE` (power + 10d10); `STORM`'s water component.

### 2.16 Ice (spells only)

* Uses your **cold** resistance (×1/3, ×1/9), immunity to cold protects the pack.
* Cut 5d8 unless rShards; stun 1d15 unless pStun.
* `BO_ICE` = power + 6d6 (Sauron: 136–166).

### 2.17 Gravity

* No resist, cap 200 (divisor 3). Gravity hounds (DL35) breathe up to 64.
* Blink (teleport 5) unless `randint1(127) ≤ clvl` — at clvl 50 you still get
  blinked 77/127 ≈ 61% of the time.
* Slowed 4+1d4−1 turns with `check=false` → **ignores Free Action**.
* Stun `5 + 1d(dam/3)` capped at 35 unless pStun.

### 2.18 Inertia

* No resist, cap 200. Slowed 4–7 turns, `check=false` → **ignores Free
  Action**. Slow is −10 speed and stacks in duration.
* Inertia hounds (DL35, packs) make you slow for long stretches. The manual is
  right that FA does not help.

### 2.19 Force

* No resist, cap 200. Stun 1d20 (pStun prevents), thrown 3+dam/20 squares
  away from the centre.

### 2.20 Time

* No resist, cap 150 (divisor 3: a 330-HP time hound breathes 110).
* Each hit, exactly one of:
  * 50%: exp drain `100 + 2% of exp` — **Hold Life does not help**.
  * 40% (4/5 of the remaining half): two random stats drained (STR/INT/WIS/
    DEX/CON chosen uniformly) — calls `player_stat_dec` directly, so
    **sustains do not help**.
  * 10%: every stat drained by one step — sustains do not help.
* Stat drain here is the temporary kind (current value only); it is restored
  by Restore/Life potions, Mushroom of Vigor, Rod of Restoration, or by
  gaining a character level (`adjust_level` restores all five stats on each
  level gained).

### 2.21 Plasma

* No resist, cap 150. Stun `5 + 1d(3·dam/4)` capped at 35 unless pStun.
* Breathers: plasma vortex (DL37), plasma hound (DL51), the Phoenix, sky
  dragon, greater Balrogs, Gothmog, Lungorthin. `BO_PLAS` = power+10 + 8d7.

### 2.22 Mana, meteor, missile, holy orb, arrows

* Pure damage, no resist, no side effects. Mana is the end-game killer: mana
  storms (`BA_MANA`) are `5×power + 10d10` — 510–600 from Morgoth, 470–560
  from Vecna (power 92), 335–425 from the Mouth of Sauron (power 65).
* No monster in 4.2.6 has `BR_MANA` (verified by grepping `monster.txt`).
* Holy orb (`BA_HOLY`, only priests and patriarchs) deals 4/3 damage to
  Necromancers.

---

## Part 3 — Breath damage

### 3.1 Formula

```
breath damage = min( monster_current_HP / divisor , damage_cap )
then: distance falloff (1.5), then adjust_dam (resist), then damage reduction
```

`divisor` and `damage_cap` are per element (table 2.1). The breath spell lines
in `monster_spell.txt` are all `effect:BREATH:<ELEM>:0:30` — radius 0 (full
range, 20) and a 30° arc.

Monster HP at creation (`mon-make.c::mon_hp`): **uniques always have exactly
their listed HP**; other monsters roll `Rand_normal(avg, sd)` with
`sd ≈ avg/8 + 1`, so roughly 95% fall within ±25% of the listed value, and the
theoretical maximum is +50%. A "lucky" ancient red dragon (listed 1560) can
have ~1950 HP and breathe 650.

Breath HP needed to reach each cap:

| Element group | Divisor | Cap | HP needed to hit the cap |
|---|---|---|---|
| Acid / elec / fire / cold | 3 | 1600 | 4800 |
| Poison | 3 | 800 | 2400 |
| Nether / chaos | 6 | 600 | 3600 |
| Light / dark / sound / shards / disen | 6 | 500 | 3000 |
| Nexus | 6 | 400 | 2400 |
| Gravity | 3 | 200 | 600 |
| Inertia / force | 6 | 200 | 1200 |
| Time | 3 | 150 | 450 |
| Plasma | 6 | 150 | 900 |

### 3.2 Worst-case breath vs. resist level

"Worst case" uses the cap and, for high elements, the 6/9 denominator roll.

| Element | Cap | Unresisted | Resisted | Double resisted | Immune |
|---|---|---|---|---|---|
| Fire / cold / elec | 1600 | 1600 | 533 | 177 | 0 |
| Acid (armour halves) | 1600 | 800 | 266 | 88 | 0 |
| Poison | 800 | 800 | 266 | 88 | n/a |
| Nether, chaos | 600 | 600 | 400 (300 best) | n/a | n/a |
| Light, dark, sound, shards, disen | 500 | 500 | 333 (250 best) | n/a | n/a |
| Nexus | 400 | 400 | 266 (200 best) | n/a | n/a |
| Gravity / inertia / force | 200 | 200 | — | — | — |
| Time / plasma | 150 | 150 | — | — | — |

Vulnerable: ×4/3 (a 1600 fire breath becomes 2133).

### 3.3 Breathers by depth with their full-HP breath damage

Breath values below are listed-HP / divisor, capped (what a freshly generated,
average-HP monster breathes). Speed is relative to normal (+10 = twice your
speed at +0). Hounds come in packs (`friends:100:2d7:Same`).

| Monster | DL | HP | Speed | Breaths (damage at listed HP) |
|---|---|---|---|---|
| baby blue dragon | 9 | 88 | +0 | ELEC 29 |
| baby multi-hued dragon | 11 | 114 | +0 | ACID/COLD/ELEC/FIRE/POIS 38 |
| energy / fire hound | 18 | 35 | +0 | ELEC / FIRE 11 |
| chimaera | 20 | 160 | +0 | FIRE 53 |
| air hound | 20 | 68 | +0 | POIS 22 |
| earth hound | 20 | 68 | +0 | SHARDS 11 |
| vibration hound | 27 | 138 | +0 | SOUND 23 |
| nexus hound | 27 | 138 | +0 | NEXUS 23 |
| basilisk | 28 | 310 | +10 | POIS 103 |
| young multi-hued dragon | 32 | 281 | +0 | five base 93 |
| shadow drake | 33 | 264 | +0 | NETHER 44 |
| crystal drake | 33 | 396 | +0 | SHARDS 66 |
| chaos drake | 33 | 440 | +0 | CHAOS 73, DISEN 73 |
| law drake | 33 | 440 | +0 | SHARDS 73, SOUND 73 |
| balance drake | 33 | 528 | +0 | CHAOS/DISEN/SHARDS/SOUND 88 |
| gravity hound | 35 | 193 | +0 | GRAVITY 64 |
| inertia / impact hound | 35 | 193 | +0 | INERTIA / FORCE 32 |
| mature red dragon | 41 | 740 | +5 | FIRE 246 |
| Kavlax the Many-Headed (U) | 41 | 1300 | +10 | ACID/COLD/ELEC/FIRE 433, CHAOS/NEXUS/SHARDS/SOUND 216, GRAVITY 200 |
| hellhound | 42 | 352 | +10 | FIRE 117 |
| mature multi-hued dragon | 43 | 1000 | +5 | five base 333 |
| multi-hued hound | 43 | 220 | +0 | five base 73 |
| greater basilisk | 45 | 1010 | +10 | POIS 336, DARK 168, NEXUS 168 |
| winged horror | 48 | 1013 | +10 | POIS 337, DARK 168, NETHER 168 |
| The Balrog of Moria (U) | 50 | 3000 | +20 | **FIRE 1000** |
| death drake | 50 | 1950 | +10 | NETHER 325 |
| great crystal drake | 50 | 1950 | +10 | SHARDS 325 |
| ethereal dragon | 50 | 1950 | +10 | LIGHT 325, DARK 325 |
| ancient red dragon | 51 | 1560 | +10 | FIRE 520 |
| nether hound | 51 | 330 | +10 | NETHER 55 |
| plasma hound | 51 | 330 | +10 | PLASMA 55 |
| time hound | 51 | 330 | **+20** | TIME 110 |
| Itangast the Fire Drake (U) | 52 | 2200 | +10 | FIRE 733 |
| ancient multi-hued dragon | 53 | 2400 | +10 | ACID/COLD/ELEC/FIRE **800**, POIS **800** |
| Scatha the Worm (U) | 53 | 2500 | +10 | COLD 833 |
| drolem | 54 | 2200 | +10 | **POIS 733** |
| The Phoenix (U) | 54 | 3600 | +10 | FIRE 1200, LIGHT 500, PLASMA 150 |
| dracolich | 55 | 3080 | +10 | COLD 1026, NETHER 513 |
| dracolisk | 55 | 3080 | +10 | FIRE 1026, NEXUS 400 |
| Shelob, Spider of Darkness (U) | 55 | 3500 | +10 | POIS 800, DARK 500 |
| 9-headed hydra | 57 | 1200 | +10 | FIRE/POIS/ACID 400, NETHER 200 |
| Baphomet the Minotaur Lord (U) | 58 | 4700 | +20 | FORCE 200 |
| aether vortex | 60 | 420 | +20 | 17 elements; base 140, time 140, gravity 140, rest 70 |
| bile demon | 61 | 2464 | +10 | ACID 821, POIS 800 |
| Smaug the Golden (U) | 62 | 4200 | +10 | FIRE 1400 |
| werewolf of Sauron | 63 | 2200 | +10 | NETHER 366, DARK 366 |
| spider of Gorgoroth | 63 | 4000 | +10 | POIS 800, DARK 500 |
| chaos hound | 64 | 930 | +10 | CHAOS 155 |
| jabberwock | 68 | 2816 | +20 | CHAOS 469 |
| hound of Tindalos | 69 | 480 | +20 | TIME 150, NETHER 80, DISEN 80 |
| gelugon | 69 | 3080 | +20 | COLD 1026, SHARDS 500 |
| nightcrawler | 69 | 2440 | +10 | NETHER 406 |
| great storm / ice wyrm | 70 | 4200 | +10 | ELEC / COLD 1400 |
| great swamp wyrm | 70 | 4200 | +10 | POIS 800 |
| Ossë, Herald of Ulmo (U) | 72 | 6000 | +20 | DISEN 500 |
| great hell / bile wyrm | 73 | 4500 | +10 | FIRE / ACID 1500 |
| great wyrm of thunder | 73 | 4500 | +10 | SOUND 500 |
| aether hound | 74 | 1230 | +20 | 18 elements: base 410, chaos/dark/disen/light/nether/nexus/shards/sound 205, grav/inertia/force 200, time/plasma 150 |
| great wyrm of chaos | 75 | 3960 | +10 | CHAOS 600, DISEN 500 |
| great wyrm of law | 75 | 3960 | +10 | SHARDS 500, SOUND 500 |
| Glaurung (U) | 75 | 7500 | +20 | FIRE 1600, POIS 800 |
| Ungoliant, the Unlight (U) | 75 | 13000 | +20 | DARK 500, POIS 800 |
| pit fiend | 77 | 3520 | +20 | FIRE 1173, POIS 800, CHAOS 586 |
| serpent of chaos | 78 | 900 | +20 | CHAOS 150 |
| great wyrm of annihilation | 80 | 5000 | +10 | POIS 800, NETHER 600, CHAOS 600, DARK 500, TIME 150 |
| sky dragon | 80 | 5000 | +10 | ELEC 1600, LIGHT 500, GRAVITY 200, FORCE 200, PLASMA 150 |
| Wiruin, the Maelstrom (U) | 80 | 3500 | +30 | COLD 1166, SOUND 500 |
| great wyrm of balance | 82 | 5200 | +10 | CHAOS 600, DISEN/SHARDS/SOUND 500 |
| great wyrm of many colours | 82 | 5200 | +10 | ACID/COLD/ELEC/FIRE **1600**, POIS 800 |
| greater Balrog | 85 | 4500 | +20 | FIRE 1500, PLASMA 150 |
| Ancalagon the Black (U) | 85 | 10000 | +20 | ACID/COLD/ELEC/FIRE 1600, POIS 800 |
| The Tarrasque (U) | 86 | 8500 | +20 | COLD/FIRE 1600, DISEN 500 |
| Meássë, the Bloody (U) | 87 | 7000 | +10 | SOUND 500 |
| Lungorthin (U) | 88 | 7000 | +20 | FIRE 1600, PLASMA 150 |
| Carcharoth (U) | 90 | 8000 | +20 | FIRE 1600, POIS 800, NETHER 600, DARK 500 |
| Huan (U) | 90 | 8000 | +20 | COLD 1600, LIGHT/SHARDS/SOUND 500 |
| storm of Unmagic | 95 | 600 | +30 | TIME 150, CHAOS 100, DISEN 100 |
| Gothmog (U) | 95 | 8000 | +20 | FIRE 1600, PLASMA 150 |
| Wolf-Sauron (U) | 99 | (Sauron's) | +30 | FIRE, NETHER |
| Serpent-Sauron (U) | 99 | (Sauron's) | +20 | POIS, CHAOS, ACID |

Sauron's shapes are listed with 0 HP in `monster.txt` because
`monster_change_shape` swaps the race but leaves `mon->hp` alone: Sauron in
wolf or serpent form breathes from **Sauron's own current HP (8000 max)**, i.e.
up to 1600 fire/acid, 800 poison, 600 nether/chaos.

### 3.4 Worked examples

**Ancient red dragon (DL51), listed 1560 HP, power 51.**

| Situation | Dragon HP | Breath | No rFire | rFire | rFire + Resist Heat | Immune |
|---|---|---|---|---|---|---|
| Average spawn, unhurt | 1560 | 520 | 520 | 173 | 57 | 0 |
| Lucky spawn (+2 s.d.) | ~1952 | 650 | 650 | 216 | 72 | 0 |
| You have wounded it to half | 780 | 260 | 260 | 86 | 28 | 0 |
| You wear a *burning up* item (vulnerable) | 1560 | 520 | 693 | 520 (vuln+res = 0) | 173 | 0 |

Power 51 < 80, so no STR-drain/blind/poison fire side effects. Every hit above
60 post-resist rolls 3% per scroll/book/staff/arrow in the pack.

**Great wyrm of many colours (DL82), 5200 HP.** Any breath at ≥4800 HP hits
the 1600 cap. Unresisted fire or cold = 1600 (fatal for nearly every
character), resisted 533, double-resisted 177. Its poison breath is capped at
800: 266 resisted, 88 with a Potion of Resist Poison on top. Acid with armour:
800 / 266 / 88. Power 82 ≥ 80 means the fire/cold side effects are live, but
they need >500 post-resist damage, so a resisted character only rarely sees
them and a double-resisted one never.

**Drolem (DL54), listed 2200 HP, +10 speed, breath 1-in-8 innate frequency.**
2200/3 = 733 poison. About a quarter of drolems spawn with ≥2400 HP and breathe
the full 800. Without rPois that is death for most characters around DL50–55;
with rPois it is 244–266; with rPois + `OPP_POIS` 81–88. Unresisted, the poison
status alone can last 10+1d733 turns.

**Time hound pack (DL51).** Each hound has ~330 HP → 110 time per breath, cap
150 is reached only by hounds with ≥450 HP. A pack averages 8 hounds (2d7) at
+20 speed: 16 hound-turns per normal-speed player turn; with innate frequency 5
that is about 3 breaths per player turn: ~330 damage, plus a mixture of
unblockable exp drains and stat drains every turn.

---

## Part 4 — Monster spell damage and saving throws

### 4.1 How damage spells scale

Spell damage uses the monster's `spell_power` (default = monster depth; a few
monsters override it, e.g. Sauron 130, Serpent-Sauron 108, Wolf-Sauron 105,
Cantoras 110, Nan 85, priest 6, abyss spider 10). The dice expressions in
`monster_spell.txt` are evaluated left to right (`* 3 / 2 + 30` means
`p*3/2+30`). Integer division throughout.

| Spell | Formula (p = spell power) | Element |
|---|---|---|
| BO_ACID | p/3 + 7d8 | acid |
| BO_ELEC | p/3 + 4d8 | lightning |
| BO_FIRE | p/3 + 9d8 | fire |
| BO_COLD | p/3 + 6d8 | cold |
| BO_POIS | p/3 + 9d8 | poison |
| BO_NETH | p×3/2 + 30 + 5d5 | nether |
| BO_WATE | p + 10d10 | water |
| BO_MANA | 50 + 1d(p×5/2) | mana |
| BO_PLAS | p + 10 + 8d7 | plasma |
| BO_ICE | p + 6d6 | ice |
| MISSILE | p/3 + 2d6 | missile |
| BE_ELEC / BE_NETH | p×2 + 30 + 5d5 (short beam) | lightning / nether |
| BA_ACID | 15 + 1d(p×3) | acid |
| BA_ELEC | 8 + 1d(p×3/2) | lightning |
| BA_FIRE | 10 + 1d(p×7/2) | fire |
| BA_COLD | 10 + 1d(p×3/2) | cold |
| BA_POIS | (p/2 + 3)d4 | poison |
| BA_SHAR | 10 + 1d(p×3/2) | shards |
| BA_NETH | p×4 + 10d10 | nether |
| BA_WATE | 50 + 1d(p×5/2) | water |
| BA_MANA | p×5 + 10d10 | mana |
| BA_HOLY | 10 + 1d(p×3/2) | holy orb |
| BA_DARK | p×4 + 10d10 | dark |
| BA_LIGHT | 10 + 1d(p×3/2) | light |
| STORM | three radius-3 balls: water 30+(p/3)d5, elec 20+(p/3)d5, ice 20+(p/3)d5 | water, elec, ice |
| WOUND | damage (p/3×2)d5; cut (p/5−10)d10 (none below p=55) | — (save negates) |
| MIND_BLAST | 8d8 + confused 3+1d4 | — (save negates) |
| BRAIN_SMASH | 12d15 + slow 3+1d4, confused 3+1d4, paralyzed 3+1d4, blind 7+1d8 | — (save negates) |
| DRAIN_MANA | removes 1d(p/2+1) mana, heals caster 6× that | — |
| SHOT / ARROW / BOLT | (p/8+1)d5 / d6 / d7, hit roll vs AC (hit 50) | arrow |
| BOULDER | (p/7+1)d12, hit roll vs AC (hit 60) | arrow |

### 4.2 Damage ranges by spell power

Min–max raw damage (before resistance):

| Spell | p=10 | p=20 | p=30 | p=40 | p=50 | p=60 | p=70 | p=80 | p=100 | p=130 |
|---|---|---|---|---|---|---|---|---|---|---|
| BO_ACID | 10–59 | 13–62 | 17–66 | 20–69 | 23–72 | 27–76 | 30–79 | 33–82 | 40–89 | 50–99 |
| BO_ELEC | 7–35 | 10–38 | 14–42 | 17–45 | 20–48 | 24–52 | 27–55 | 30–58 | 37–65 | 47–75 |
| BO_FIRE / BO_POIS | 12–75 | 15–78 | 19–82 | 22–85 | 25–88 | 29–92 | 32–95 | 35–98 | 42–105 | 52–115 |
| BO_COLD | 9–51 | 12–54 | 16–58 | 19–61 | 22–64 | 26–68 | 29–71 | 32–74 | 39–81 | 49–91 |
| BO_NETH | 50–70 | 65–85 | 80–100 | 95–115 | 110–130 | 125–145 | 140–160 | 155–175 | 185–205 | 230–250 |
| BO_WATE | 20–110 | 30–120 | 40–130 | 50–140 | 60–150 | 70–160 | 80–170 | 90–180 | 110–200 | 140–230 |
| BO_MANA | 51–75 | 51–100 | 51–125 | 51–150 | 51–175 | 51–200 | 51–225 | 51–250 | 51–300 | 51–375 |
| BO_PLAS | 28–76 | 38–86 | 48–96 | 58–106 | 68–116 | 78–126 | 88–136 | 98–146 | 118–166 | 148–196 |
| BO_ICE | 16–46 | 26–56 | 36–66 | 46–76 | 56–86 | 66–96 | 76–106 | 86–116 | 106–136 | 136–166 |
| MISSILE | 5–15 | 8–18 | 12–22 | 15–25 | 18–28 | 22–32 | 25–35 | 28–38 | 35–45 | 45–55 |
| BE_ELEC / BE_NETH | 55–75 | 75–95 | 95–115 | 115–135 | 135–155 | 155–175 | 175–195 | 195–215 | 235–255 | 295–315 |
| BA_ACID | 16–45 | 16–75 | 16–105 | 16–135 | 16–165 | 16–195 | 16–225 | 16–255 | 16–315 | 16–405 |
| BA_ELEC | 9–23 | 9–38 | 9–53 | 9–68 | 9–83 | 9–98 | 9–113 | 9–128 | 9–158 | 9–203 |
| BA_FIRE | 11–45 | 11–80 | 11–115 | 11–150 | 11–185 | 11–220 | 11–255 | 11–290 | 11–360 | 11–465 |
| BA_COLD / BA_SHAR / BA_HOLY / BA_LIGHT | 11–25 | 11–40 | 11–55 | 11–70 | 11–85 | 11–100 | 11–115 | 11–130 | 11–160 | 11–205 |
| BA_POIS | 8–32 | 13–52 | 18–72 | 23–92 | 28–112 | 33–132 | 38–152 | 43–172 | 53–212 | 68–272 |
| BA_NETH / BA_DARK | 50–140 | 90–180 | 130–220 | 170–260 | 210–300 | 250–340 | 290–380 | 330–420 | 410–500 | 530–620 |
| BA_WATE | 51–75 | 51–100 | 51–125 | 51–150 | 51–175 | 51–200 | 51–225 | 51–250 | 51–300 | 51–375 |
| BA_MANA | 60–150 | 110–200 | 160–250 | 210–300 | 260–350 | 310–400 | 360–450 | 410–500 | 510–600 | 660–750 |
| WOUND | 6–30 | 12–60 | 20–100 | 26–130 | 32–160 | 40–200 | 46–230 | 52–260 | 66–330 | 86–430 |

Key end-game numbers:

| Caster | Power | Most dangerous spells (raw) |
|---|---|---|
| The Mouth of Sauron (DL65) | 65 | BA_MANA 335–425, BA_NETH/BA_DARK 270–360, BA_WATE 51–212, BO_PLAS 83–131, HOLD |
| Vecna (DL92) | 92 | BA_MANA 470–560, BA_NETH 378–468, BO_MANA 51–280, BRAIN_SMASH |
| The Witch-King (DL85) | 85 | BA_NETH 350–440, BO_MANA 51–262, HOLD, BRAIN_SMASH |
| Sauron (DL99) | 130 | BA_NETH/BA_DARK 530–620, BA_FIRE 11–465, BO_MANA 51–375, BO_PLAS 148–196, BO_ICE 136–166, WOUND 86–430 |
| Vampire-Sauron | 130 | BE_NETH 295–315, BA_NETH 530–620, STORM, BRAIN_SMASH, FORGET |
| Morgoth (DL100) | 100 | **BA_MANA 510–600 (unresistable)**, BA_NETH 410–500, BO_MANA 51–300, BRAIN_SMASH |

### 4.3 The saving throw

`do_mon_spell()`: if the spell level has a `message-save` line and the target is
the player, then `randint0(100) < skill_save` cancels the **whole spell**. The
spells with a save line are:

`BLIND, CONF, SCARE, SLOW, HOLD, FORGET, MIND_BLAST, BRAIN_SMASH, WOUND, TELE_LEVEL`.

Bolts, balls, breaths, `DRAIN_MANA`, `TELE_TO`, `TELE_AWAY`, `DARKNESS` and
summons get **no** saving throw. If you save, the game still checks which item
flag *would* have stopped it and teaches you that rune.

Even when the save fails, the timed effects then go through
`player_inc_check` — so pBlind, pConf, pFear, FA still block their effect (the
damage of MIND_BLAST/BRAIN_SMASH/WOUND still lands).

`skill_save` = race base + class base + `adj_wis_sav[WIS]` + class increment ×
clvl/10 (no cap; at ≥100 every save succeeds):

| Class | Base | Per 10 clvl | Race | Base |
|---|---|---|---|---|
| Warrior | 18 | +10 | Human | 0 |
| Mage | 30 | +9 | Half-Elf | 3 |
| Druid | 30 | +10 | Elf | 6 |
| Priest | 32 | +12 | Hobbit | 18 |
| Necromancer | 30 | +9 | Gnome | 12 |
| Paladin | 25 | +11 | Dwarf | 9 |
| Rogue | 28 | +10 | Half-Orc | −3 |
| Ranger | 32 | +10 | Half-Troll | −8 |
| Blackguard | 18 | +10 | Dúnadan | 5 |
| | | | High-Elf | 20 |
| | | | Kobold | 0 |

`adj_wis_sav` goes from 0 (WIS ≤7) to 3 at 18/00, 7 at 18/100, 12 at 18/150 and
18 at 18/210+. Example: a clvl-50 Human Warrior with 18/100 WIS has
18+0+7+50 = 75% save; a clvl-50 High-Elf Priest with 18/150 WIS has
32+20+12+60 = 124% (never fails). Severe hunger (the last tenth of the hunger range, around "Faint") multiplies skill_save
by 0.9.

### 4.4 Non-damage spells

| Spell | Effect on player | Save? | Item defence |
|---|---|---|---|
| SCARE | afraid 3+1d4 | yes | pFear (also Hero, Berserk, Bold) |
| BLIND | blind 11+1d4 | yes | pBlind |
| CONF | confused 3+1d4 | yes | pConf (also `OPP_CONF`) |
| SLOW | slowed 3+1d4 (−10 speed) | yes | **Free Action** |
| HOLD | paralyzed 3+1d4 | yes | **Free Action** |
| FORGET | amnesia 3 | yes | none |
| TELE_LEVEL | up/down a level | yes | **rNexus** (also `NO_TELEPORT` curse) |
| TELE_TO | pulled next to the caster | no | none (`NO_TELEPORT` curse blocks all teleports) |
| TELE_AWAY | teleported 100 squares | no | none |
| DRAIN_MANA | lose 1d(p/2+1) SP | no | none |
| DARKNESS | room unlit, `DARK_WEAK` ball blinds 3+1d5 | no | rDark |
| TRAPS | creates traps around you | no | trap immunity |
| SHRIEK | wakes and hastes monsters in LOS | no | — |

---
## Part 5 — Afflictions and timed effects

### 5.1 How timed effects are applied

* `player_inc_timed(p, idx, amount, ..., check)` adds `amount` to the current
  duration. If `check` is true the effect first runs `player_inc_check`, which
  walks the `fail:` lines of `player_timed.txt`:

  | `fail` code | Meaning | Example |
  |---|---|---|
  | 1 | object flag on you | `fail:1:FREE_ACT` on PARALYZED and SLOW |
  | 2 | you resist an element | `fail:2:CHAOS` on IMAGE; `fail:2:POIS` on POISONED; `fail:2:NEXUS` on SCRAMBLE |
  | 3 | you are vulnerable to an element | `fail:3:FIRE` on OPP_FIRE (*vulnerable characters cannot start the timed effect, but if it is already running the res_level math in 1.1 still applies*) |
  | 4 | player flag | `fail:4:ROCK` on CUT (no 4.2.6 race/class has ROCK) |
  | 5 | another timed effect is active | `fail:5:OPP_POIS` on POISONED |

* Several damaging sources call with `check=false` and **bypass protection**:
  slowing from inertia, gravity and darkness; Black Breath; the Potion of Salt
  Water and fainting from hunger (paralysis through Free Action); the slow after
  Sprinting; the stun of rock-fall traps (`TIMED_INC_NO_RES`).
* `PARALYZED` is the only `NONSTACKING` effect: while it is active, further
  paralysis attempts are refused (they cannot extend it).
* Durations of most effects drop by 1 per turn; **cuts, stun and poison drop by
  `adj_con_fix[CON] + 1`** per turn (1 at CON ≤13, 2 at 14–17, 3 at 18/00–18/49,
  4 at 18/50–18/99, 5 at 18/100–18/109, up to 10 at 18/220+). A "Mortal Wound"
  never drops on its own.
* Natural HP regeneration is **zero** while paralyzed, poisoned, stunned or
  cut (`player_regen_hp`).

### 5.2 Harmful effects — mechanics and protection

| Effect | Grades / limits | What it does (code) | Prevented by | Cured by |
|---|---|---|---|---|
| **Blind** | — | Cannot read scrolls, cast spells, see monsters or the map; disarm skill /10; monsters unseen | pBlind (Dwarf innate); rLight/rDark vs light/dark effects | CLW, CSW, CCW, Healing, *Healing*, Life, True Seeing, Mushroom of Second Sight / Fast Recovery, Staff/Rod of Curing, Staff of Holiness |
| **Confused** | — | 75% of moves/aims go in a random direction; cannot read or cast; device skill −25%; disarm /10 | pConf, `OPP_CONF` (Mushroom of Clear Mind, Blackguard *Grim Purpose*) | CLW (−20 turns), CSW, CCW, Healing, *Healing*, Life, Mushroom of Clear Mind, Staff/Rod of Curing, Staff of Holiness |
| **Poisoned** | — | 1 HP/turn (after damage reduction); no regeneration; device skill −5% | rPois, `OPP_POIS` | CCW, Healing, *Healing*, Life, Neutralize Poison, Resist Poison, Mushroom of Fast Recovery / Purging, Elvish Waybread, Staff/Rod of Curing, Staff of Holiness, Druid *Cure Poison*, Herbal Curing, Priest/Paladin *Healing*, *Holy Word* |
| **Cut** | Graze ≤10, Light Cut ≤25, Bad Cut ≤50, Nasty Cut ≤100, Severe Cut ≤200, Deep Gash ≤1000, **Mortal Wound** >1000 | 1 HP/turn up to Nasty, 2 for Severe, 3 for Deep Gash and Mortal; Mortal never heals by itself; no regeneration | rShards (vs shards/ice); nothing vs melee crits, WOUND spells, traps | CLW (−20), CSW, CCW, Healing, *Healing*, Life, Mushroom of Fast Recovery, Staff/Rod of Curing, Rod of Healing, Herbal Curing, *Healing*/*Holy Word* spells |
| **Stun** | Stun 1–50, Heavy Stun 51–150, **Knocked Out** >150 | Stun: −5 to-hit/to-dam, device −10%, spells +15% fail. Heavy: −20/−20, device −20%, spells +25% fail. Knocked Out: **you get no turns at all**. No regeneration; cancels Fast-cast | pStun; rSound vs sound | CCW, Healing, *Healing*, Life, Mushroom of Fast Recovery, Staff/Rod of Curing, Rod of Healing, Herbal Curing, *Healing* spells |
| **Paralyzed** | non-stacking | No turns (forced `CMD_SLEEP`); thieves' touch always succeeds | **Free Action** (Gnome innate); saving throw vs HOLD and melee PARALYZE | Nothing — wait it out (you cannot act) |
| **Afraid** | — | Cannot melee ("You are too afraid to attack"); −20 to-hit, **+8 AC**; device −5%; spells +20% fail | pFear, Hero, Berserk, Bold; Warriors gain pFear at clvl 30 (`BRAVERY_30`) | Heroism, Berserk Strength, Boldness, Mushroom of Clear Mind, Staff of Holiness, Fine Wine (Bold) |
| **Terror** | — | As afraid **plus +10 speed** (from Mushroom of Terror) | — | Staff of Holiness |
| **Hallucination** (`IMAGE`) | — | Monsters and objects display as random things; device −20%; disarm /10 | rChaos only | Life, Mushroom of Clear Mind, Staff of Holiness |
| **Slow** | stacks in duration | −10 speed | Free Action vs SLOW spell, BRAIN_SMASH and Potion of Slowness; **not** vs slow darts (`TIMED_INC_NO_RES`), inertia, gravity, dark side effects or post-Sprint slow | Staff of Holiness (only `CURE:SLOW` item) |
| **Amnesia** | — | Cannot read scrolls; spell fail becomes 50 + fail/2; device −20% | nothing (saving throw vs FORGET) | CCW, Healing, *Healing*, Life, Staff of Healing, Staff of Holiness, Priest/Paladin *Healing* |
| **Scrambled** | 20–39 turns | Your five stats are shuffled; restored at end | rNexus; saving throw vs nexus | wait it out |
| **Black Breath** | — | Each turn: 50% CON drain, 50% STR drain, 50% exp drain 100+2% exp — **ignores sustains and Hold Life** | nothing | Druid *Herbal Curing* (clvl 40), Ranger *Herbal Curing* (clvl 9); otherwise wait |
| **Hunger** | Starving ≤1%, Faint ≤4%, Weak ≤8%, Hungry ≤15%, Fed ≤90%, Full >90% (of the food bar; grade maxima in `player_timed.txt`) | Hungry/Weak/Faint: progressive to-hit/to-dam penalties, device/disarm/save penalties; Faint: 1-in-10 per turn of **paralysis 1–5 turns ignoring Free Action**; Starving: damage each turn. Full (gorged): up to −10 speed | Slow Digestion halves food use; Regeneration doubles it | Eat |

### 5.3 Details worth knowing

**Stun accumulation.** Every source *adds* to the counter. Sources and amounts:

| Source | Stun added | Blocked by |
|---|---|---|
| Sound | 5+1d(dam/3), max 35 per hit | rSound or pStun |
| Gravity | 5+1d(dam/3), max 35 | pStun |
| Plasma | 5+1d(3·dam/4), max 35 | pStun |
| Force | 1d20 | pStun |
| Water | 1d40 | pStun |
| Ice | 1d15 | pStun |
| Melee critical (HIT, PUNCH, KICK, BUTT, CRUSH) | 1d5 / 10+1d10 / 20+1d20 / 30+1d30 / 40+1d40 / **100** / **200** by crit tier | pStun |
| Rock-fall trap | 2d20 | nothing (`TIMED_INC_NO_RES`) |
| Petrifying trap | 20+1d20 | pStun |
| Orcish Liquor | set to 2+1d3 | — |

Melee crits (`monster_critical`): a blow must roll ≥95% of its maximum damage;
blows under 20 damage only qualify with probability dam%. Tier = 1 + (perfect
roll) + (2% chained "super-charge" rolls), plus 1 per damage band >11, >18,
>25, >33, >45. A >45-damage perfect hit is tier 7 → **200 stun = instant
knock-out**. Big crushers (Morgoth's `HIT:SHATTER:20d10`, giants, golems)
make **pStun mandatory** in the end game.

**Cut sources.** Shards 1d(dam); ice 5d8; melee crits from HIT/CLAW/BITE
(1d5, 5+1d5, 20+1d20, 50+1d50, 100+1d100, 300, 500); WOUND at p ≥55 adds
(p/5−10)d10 (Sauron p130: 16d10); knife trap 150; spiked/poison pits 4d6.

**Hold Life, sustains and drains** are covered in Part 7.

### 5.4 Beneficial timed effects (exact numbers)

| Effect | Mechanical effect (`player-calcs.c`) | Sources (duration) |
|---|---|---|
| Haste (`FAST`) | +10 speed | Potion of Speed 20+2d10 (if already hasted, only +5 turns), Rod/Staff of Speed (same), Druid/Ranger *Haste Self* |
| Sprint | +10 speed, **then 100 turns of Slow that ignores FA** | Mushroom of Sprinting (100) |
| Heroism (`HERO`) | +12 to-hit, device +5%, immune to fear | Potion (heal 10, 25+1d25), Priest/Paladin *Heroism* |
| Berserk (`SHERO`) | +75 melee skill, −10 AC, device −10%, immune to fear | Potion (heal 30, 25+1d25), Blackguard *Berserk Strength* |
| Bold | immune to fear | Pint of Fine Wine 79+1d41 |
| Blessed | +5 AC, +10 to-hit, device +5% | Scroll of Blessing 6+1d12, Holy Chant 12+1d24, Holy Prayer 24+1d48, Priest/Paladin *Bless* |
| Shield | +50 AC | spells/activations |
| Stone skin | +40 AC, **−5 speed** | Mushroom of Stoneskin 50+7d7; *stone* curse; petrifying trap |
| Protection from Evil | each blow from an EVIL monster with level ≤ your clvl is repelled if `randint0(100)+clvl > 50` | Scroll (3×clvl + 1d25), Staff of Holiness, Priest/Paladin spell |
| Invulnerability | +100 AC, all damage <9000 ignored | rare activations |
| Resist Acid/Elec/Fire/Cold/Poison | +1 res_level (max 2) | Potions of Resist Heat / Cold / Poison (50+1d10), ring activations, Mage *Resistance* (all four base), Druid/Ranger *Resist Poison*, Mushroom of Emergency (fire+cold) |
| RConf (`OPP_CONF`) | protection from confusion | Mushroom of Clear Mind 50+7d7, Blackguard *Grim Purpose* |
| Free Action (timed) | Free Action | Blackguard *Grim Purpose* |
| See Invisible | see invisible | Potion of True Seeing 50+2d6, *Sense Invisible* spells |
| Telepathy | ESP | Mushroom of Second Sight 50+7d7 |
| Infravision | +5 infravision | Potion of Infravision |
| Trap safety (`TRAPSAFE`) | trap immunity | (no 4.2.6 item; flag version from the eagle shape and *of Wormtongue*) |
| Stealth | +10 stealth | Mushroom of Shadows 10+1d10 |
| Heal (`TMD_HEAL`) | heals 30 HP every turn; burns food | some class abilities |
| Bloodlust | +to-dam = counter/2, +1 blow per 20 points of counter; HP/cut/slow exertion side effects as it fades | Blackguard |

### 5.5 Cure matrix for consumables

| Item | HP healed | Blind | Conf | Poison | Stun | Cut | Amnesia | Halluc | Other |
|---|---|---|---|---|---|---|---|---|---|
| Potion of Cure Light Wounds | 20 | cure | −20 | — | — | −20 | — | — | |
| Potion of Cure Serious Wounds | 40 | cure | cure | — | — | cure | — | — | |
| Potion of Cure Critical Wounds | 60 | cure | cure | cure | cure | cure | cure | — | |
| Potion of Healing | max(300, 35% of missing) | cure | cure | cure | cure | cure | cure | — | |
| Potion of *Healing* | 1200 | cure | cure | cure | cure | cure | cure | — | |
| Potion of Life | 5000 | cure | cure | cure | cure | cure | cure | cure | restore exp and all stats |
| Potion of Neutralize Poison | — | — | — | cure | — | — | — | — | |
| Potion of Resist Poison | — | — | — | cure | — | — | — | — | rPois 50+1d10 |
| Mushroom of Fast Recovery | 30 | cure | — | cure | cure | cure | — | — | |
| Mushroom of Clear Mind | — | — | cure | — | — | — | — | cure | 20 SP, cure fear, RConf |
| Mushroom of Emergency | 200 | — | — | — | — | — | — | **causes** | rFire+rCold 25+5d5 |
| Mushroom of Vigor | — | — | — | — | — | — | — | — | restore all stats |
| Staff / Rod of Curing | — | cure | cure | cure | cure | cure | — | — | |
| Staff of Healing | max(300, 35%) | cure | cure | cure | cure | cure | cure | — | |
| Rod of Healing | 500 | — | — | — | cure | cure | — | — | |
| Staff of Holiness | 50 | cure | cure | cure | cure | cure | cure | cure | cures fear, terror, **slow**; Prot Evil; dispel evil 120 |

Things nothing in the potion line cures: paralysis (you can't act anyway),
scrambled stats, Black Breath (Herbal Curing only), slow (only the Staff of
Holiness).

---

## Part 6 — Monster melee blow effects

### 6.1 To-hit and armour

* Monster hit chance (`chance_of_monster_hit`): `to_hit = 3 × monster level +
  effect power` (power from `blow_effects.txt`). Then `hit_chance`: always hit
  12%, always miss 5%, otherwise the hit rate is `(to_hit − 2/3·AC) / to_hit`
  scaled into the remaining 83%. Stunned monsters lose to-hit.
* `HURT` and `SHATTER` damage is reduced by armour:
  `dam − dam × min(AC, 240) / 400` (up to 60% reduction at AC 240).
* Elemental blows (ACID/ELEC/FIRE/COLD/POISON) take the **larger** of the
  armour-reduced physical damage (with AC+50) and the resist-adjusted
  elemental damage — resistance matters a lot for these.
* Paralysis-type blows have power 0, so a floating eye (level 1) has
  `to_hit` 3 (floored to 9) and hits an AC-14 character only the guaranteed
  12% of the time; a ghoul (level 26) has to_hit 78.

### 6.2 Effect table

| Effect | Power | What it does (`mon-blows.c`) | Defence | Early examples (DL) |
|---|---|---|---|---|
| HURT | 40 | physical, AC-reduced; can cut/stun via crits | AC, pStun | everything |
| POISON | 20 | elemental poison damage + poisoned 5+1d(rlev) | rPois | white worm mass 1, drolem bites 5d10 |
| DISENCHANT | 10 | damage, then disenchant one equipped item (artifacts 60% resist) | rDisen | disenchanter eye 5, disenchanter mold 10, Mîm's sons 24, Mouth of Sauron, Sauron |
| DRAIN_CHARGES | 10 | drains charges of one wand/staff in pack, heals monster `rlev × charges` | none (keep devices at home / kill fast) | lich 34, gauth 36, beholder 40, Balrogs, Vecna, Morgoth |
| EAT_GOLD | 0 | steals gold (10% + 1d25, more above 5000) then teleports | save `adj_dex_safe[DEX] + clvl` % (not while paralyzed) | street urchin 0, cutpurse 2, Sméagol 3 |
| EAT_ITEM | 0 | steals an item then teleports | same DEX+level save | squint-eyed rogue 0, rogue 16, master thief 34, Harowen 51 |
| EAT_FOOD | 0 | eats one food item | none | icky things |
| EAT_LIGHT | 0 | drains 250+1d250 turns of fuel | non-fuel lights | silver jelly 3, silver mouse 4 |
| ACID / ELEC / FIRE / COLD | 20/40/40/40 | elemental damage (max of physical and elemental), pack damage | resist/immunity | many |
| BLIND | 0 | blind 10+1d(rlev) | pBlind (no save) | bloodshot eye 7, king cobra 9 |
| CONFUSE | 20 | confused 3+1d(rlev) | pConf (no save) | grey mushroom patch 1, umber hulk 16 |
| TERRIFY | 0 | afraid 3+1d(rlev) | saving throw, then pFear | poltergeist 3, banshee 24 |
| PARALYZE | 0 | paralyzed 3+1d(rlev); always ≥1 damage while you are paralyzed | saving throw, then **Free Action** | floating eye 1, carrion crawler 25, ghoul 26 |
| LOSE_STR/INT/WIS/DEX/CON | 0 | damage then drain that stat | matching Sustain | giant red frog 7 (STR), mind flayer 28 (INT/WIS), ghast 30 (CON) |
| LOSE_ALL | 0 | drains all five (each sustain protects its stat) | all sustains | storm of Unmagic 95, Morgoth |
| SHATTER | 60 | AC-reduced damage; >23 damage causes an **earthquake** of radius dam/12 around the monster; >100 damage can knock you back | AC | Quaker 43, great earth elemental 60, Makar 76, Morgoth (20d10 ×2) |
| EXP_10/20/40/80 | 20 | lose `NdD + 2% exp` (10d6, 20d6, 40d6, 80d6) | Hold Life: 95/90/75/50% to keep it all; otherwise lose only 1/10 | grape jelly 12 (EXP_10), forest wight 24 (20), barrow wight 33 (40), emperor wight 37 (80) |
| HALLU | 0 | hallucinate 3+1d(rlev/2) | rChaos | magic mushroom patch 15, will o' the wisp 38 |
| BLACK_BREATH | 0 | 1-in-5 per hit: Black Breath for dam/10 turns | none | the Nazgûl (DL77–85) |

### 6.3 Melee cuts and stuns

`blow_methods.txt`: HIT can cut **or** stun, CLAW and BITE cut, PUNCH, KICK,
BUTT and CRUSH stun; TOUCH, GAZE, WAIL, ENGULF, SPIT and other "soft" methods
do neither. If a method could do both, one is cancelled at random. See the
tier tables in 5.3.

---

## Part 7 — Free Action, paralysis, See Invisible, Hold Life, sustains

### 7.1 Why Free Action is the first "must have"

Paralysis is the only affliction that removes *all* of your actions while the
monster keeps acting. Mechanics:

1. A paralysis blow first asks for a saving throw
   (`melee_effect_timed(..., save=true)`): `randint0(100) < skill_save`.
2. If that fails, `player_inc_timed(PARALYZED, 3 + 1d(rlev))`; this is refused
   by Free Action (`fail:1:FREE_ACT`).
3. Paralysis is `NONSTACKING`, and every paralysis blow does at least 1 damage
   while you are paralyzed — so you cannot be extended indefinitely by *one*
   blow, but nothing stops the next blow landing the instant the counter
   reaches 0.
4. The counter drops by 1 per normal-speed turn. A monster at your speed gets
   one full round of blows per turn of paralysis; a +10 monster gets two.

The chain: a pack of **ghouls** (DL26, groups of 4d4, two `CLAW:PARALYZE:1d4`
each plus the HOLD spell) or **carrion crawlers** (DL25, groups of 2d4, two
`STING:PARALYZE:2d6` each). A ghoul's paralysis lasts 3+1d26 turns. During that
window every other ghoul in contact gets free rounds; when it ends, any one of
them can re-paralyze before your energy comes back. A low-save character
(Warrior, Half-Troll, Blackguard) fails most saves: clvl 25 Human Warrior ≈
18+25+3 = 46% save, so each paralysis blow has ~54% × hit chance to succeed.

Paralysis sources and first depths:

| Source | DL | Notes |
|---|---|---|
| floating eye (`GAZE:PARALYZE`) | 1 | never moves; only dangerous if you melee it. Killed many new players |
| illusionist, druid (`HOLD` spell) | 13 | save negates |
| homunculus (`HIT:PARALYZE`) | 15 | |
| evil eye (`HOLD`) | 18 | |
| Old Man Willow (U) | 19 | two `TOUCH:PARALYZE:1d14` + HOLD + TELE_TO |
| carrion crawler | 25 | groups |
| ghoul | 26 | groups of 4d4, HOLD + paralyze claws |
| vampire (`HOLD`) | 27 | |
| gorgimaera, basilisk (+10), spectator | 27–28 | gaze |
| ghast (+10) | 30 | |
| trapper (+10, `HIT:PARALYZE:15d1` ×2) | 36 | invisible-ish lurker |
| greater basilisk (+10) | 45 | `GAZE:PARALYZE:3d12` ×2 |
| master mystic / grand master mystic (+20) | 50 / 57 | |
| Shelob (U), spider of Gorgoroth | 55 / 63 | `BITE:PARALYZE:5d10` |
| Wolf-Sauron | 99 | `CLAW:PARALYZE:6d8` |
| BRAIN_SMASH (mind flayer 28, lich 34, master lich 41 …) | 28+ | paralyzed 3+1d4 (save, then FA) |
| Potion of Sleep, sleep gas trap, *paralysis* curse | any | FA protects |
| Fainting from hunger, Potion of Salt Water | any | **FA does not protect** |

**Recommendation:** have Free Action before DL ~20 and absolutely before you
meet ghouls (DL26). Gnomes start with it. Sources from `ego_item.txt` /
`object.txt`: Gloves *of Free Action* (alloc 100, DL1–60 — the most common
source; 4.2.6 has no spell-casting penalty for gloves, so any class can wear
them), Boots *of Free Action* (alloc 20, DL1–40), Ring of Free Action
(DL20–100), Amulet of Inertia (FA but slows you, DL15–40), and egos such as
*(Defender)*, *of Gondolin*, *of Westernesse* weapons, *of Lothlórien* bows,
*of Might* crowns, *(Dwarven)* armour/gloves, gloves *of Thievery*. Free
Action also blocks the
SLOW spell and slow potions, but **not** inertia, gravity, darkness side
effects or the post-Sprint slow.

### 7.2 Knock-out is the "second paralysis"

Stun above 150 also denies all turns (`player_timed_grade_eq(STUN, "Knocked
Out")`), and Free Action does nothing about it. Only pStun (or rSound vs.
sound) prevents it. See 5.3.

### 7.3 See Invisible

`SEE_INVIS` (High-Elf innate, potions of True Seeing, Priest/Necromancer/
Paladin *Sense Invisible*). 35 monsters are `INVISIBLE`, plus every ghost
(`monster_base.txt` gives the ghost base `INVISIBLE`):

| DL | Invisible threat |
|---|---|
| 3 | poltergeist (fast, terrify), Sméagol (steals gold) |
| 7 | lost soul (LOSE_WIS) |
| 12 | moaning spirit (LOSE_DEX, terrify), air spirit |
| 15 | clear hound packs |
| 16–17 | quasit (LOSE_DEX, TELE_LEVEL, TELE_TO), imp (TELE_LEVEL, fire bolts) |
| 24 | banshee (EXP_20) |
| 26 | displacer beast |
| 31–40 | ghost (EXP_20, LOSE_INT/WIS), shade (EXP_40), spectre (EXP_40), phantom (EXP_80+EXP_40), invisible stalker (35) |
| 50+ | shadow (EXP_80), death drake (invisible nether breather), dreadmaster/dreadlord packs, nether wraith |
| 65 | The Mouth of Sauron (invisible, BA_MANA) |

Telepathy shows them too but does not let you target with bolts as reliably;
See Invisible is the clean answer from ~DL25.

### 7.4 Hold Life

* Nether (breath, bolt, ball): Hold Life alone blocks the drain (as does rNether).
* Cold/dark side-effect drains and chaos drain: blocked.
* Melee EXP_x: 95/90/75/50% chance to keep everything, else lose 1/10.
* **Not** blocked: time breath drain, Black Breath drain.
* Hobbits have it innately.

### 7.5 Sustains

* `SUST_x` blocks `EF_DRAIN_STAT` for that stat: melee LOSE_x, LOSE_ALL (per
  stat), stat-drain darts, fire/cold/poison side-effect drains, Potion of
  Surprise / Mushroom of Debility.
* **Not** blocked: time breath, Black Breath, nexus stat scramble (rNexus
  blocks that).
* Drained stats (current value) are restored by: gaining a character level,
  Potions of the stat / Augmentation / Life, Mushroom of Vigor (all), Mushroom
  of Purging (STR, CON), Miruvor and Draught of the Ents (STR, CON), Rod of
  Restoration.
* Innate sustains: Half-Elf and Elf SUST_DEX, Half-Troll SUST_STR, Dúnadan
  SUST_CON.

---
## Part 8 — Curses

### 8.1 How items get cursed

* `obj-make.c::apply_magic`: every **wearable** item has a **1 in 20** chance
  to go through `apply_curse` (items with the `BLESSED` flag are exempt).
* `apply_curse` tries `randint1(4)` times to add a random curse valid for the
  item's type, each with power `randint1(9) + 10 × m_bonus(9, level)`. Each
  successful curse *raises the item's generation level* by
  `randint1(1 + power/10)`, which then feeds the normal enchantment roll — a
  cursed item is often better than an uncursed one of the same kind.
* `m_bonus(9, level)` is a normal roll centred on `9 × level / 128` with
  standard deviation ~2, clamped to 0–9. Typical curse power by item depth:

  | Item level | Typical power (mean ± ~20) | Scroll of Remove Curse (21–40) | *Remove Curse* (51–100) |
  |---|---|---|---|
  | 10 | ~12 | almost always | always |
  | 30 | ~26 | usually | always |
  | 50 | ~40 | coin-flip | always |
  | 70 | ~54 | rarely | usually |
  | 100 | ~75 | no | usually |

* A curse is refused if it would conflict with another curse on the item
  (e.g. *teleportation* vs *anti-teleportation*), with an item flag listed in
  `conflict-flags`, or if its timed effect would be blocked by the item itself
  (no *paralysis* curse on Free Action gear, no *poison* curse on rPois gear,
  no *hallucination* curse on rChaos gear).
* Fixed curses: Ring and Amulet of Teleportation carry *teleportation* at power
  **100 (permanent)**; the artifact 'Calris' carries *air swing* at power 30.
* In 4.2.6 curses do **not** stop you removing an item. Only `STICKY` items
  can't be taken off: 'The One Ring' and the Massive Iron Crown *of Morgoth*.

### 8.2 Every runic curse (`curse.txt`)

"Every N turns" = the timeout is re-rolled from the `time:` dice each time it
fires; the counter only runs while the item is **equipped**.

| Curse | Item types | Effect | Fires | Danger / counter |
|---|---|---|---|---|
| vulnerability | cloak, soft/hard/dragon armour, shield | −50 AC and **aggravation** | permanent | High: `AGGRAVATE` instantly wakes every monster that processes its turn near you (`mon-move.c`), defeating stealth |
| teleportation | helm, crown, amulet, ring | random teleport (range 40) | every 1d100 | Annoying; deadly near a dangerous unique if it drops you next to one; conflicts with anti-teleportation |
| dullness | helm, crown | INT −5, WIS −5 | permanent | Casters |
| sickliness | helm, crown | STR −5, DEX −5, CON −5 | permanent | HP loss via CON |
| enveloping | cloak | −5 to-hit, −5 to-dam, **+20 AC** | permanent | Mild — sometimes worth keeping |
| irritation | cloak | −15 to-hit, −15 to-dam, aggravation | permanent | High |
| weakness | gloves | STR −10 | permanent | |
| clumsiness | gloves | DEX −10 | permanent | |
| slowness | boots | speed −5 | permanent | High |
| annoyance | boots | speed −10, stealth −10, aggravation | permanent | Very high |
| poison | any wearable | poisoned 10+1d10 | every 1d500 | Low with rPois (the timed effect fails) |
| siren | any wearable | wakes all monsters | every 100+1d50 | Medium |
| hallucination | any wearable | hallucinate 10+1d10 | every 1d500 | Low with rChaos |
| paralysis | any wearable | paralyzed 5+1d5 ("Your equipment grabs you!") | every 1d1000 | **Lethal without Free Action**; harmless with it |
| dragon summon | dragon armour | summons a dragon | every 1d500 | High |
| demon summon | weapons, diggers | summons a demon | every 1d500 | High |
| undead summon | weapons, diggers | summons an undead | every 1d500 | High |
| impair mana recovery | helm, crown | `IMPAIR_MANA` (slower SP regen) | permanent | Casters |
| impair hitpoint recovery | boots, gloves, shield, cloak, armours | `IMPAIR_HP` (HP regen halved) | permanent | Medium |
| cowardice | amulet, ring | permanent fear (`AFRAID`): cannot melee | permanent | Fatal for melee builds; pFear on the item conflicts, other pFear sources do **not** cancel the flag |
| stone | amulet, ring | Stone skin 20+1d20 (+40 AC, −5 speed) | every 1d1000 | Mild/mixed |
| anti-teleportation | boots, cloak, armours, amulet, ring | `NO_TELEPORT`: all teleports **including your own escapes** fail | permanent | **Lethal** — never wear unless you know what you're doing |
| treacherous weapon | weapons, diggers | you take your own weapon's damage | every 1d200 | Medium |
| burning up | boots, gloves, helm, crown, shield, cloak, armours | fire **vulnerability** (×4/3), +rCold | permanent | Dangerous vs fire breathers; cancelled by an rFire item |
| chilled to the bone | same | cold vulnerability, +rFire | permanent | Dangerous vs cold |
| steelskin | weapons, helm, crown, shield, hard/dragon armour, light, amulet, ring | +20 AC, elec vulnerability, +rShards | permanent | Often a net bonus with an rElec item |
| air swing | weapons | −20 to-hit | permanent | |

Note on *cowardice*: `player_calcs` collects `AFRAID` from equipment as a
flag; `HERO`/`BOLD` provide `PROT_FEAR`, which stops the *timed* fear effect but
the equipment flag remains. Treat a cowardice item as unwearable for melee.

### 8.3 Removing curses (`effect-handler-general.c::uncurse_object`)

You choose one item and one curse on it. Then a strength is rolled:

| Source | Strength |
|---|---|
| Scroll of Remove Curse; `REMOVE_CURSE` activation | 20 + 1d20 (21–40) |
| Scroll of *Remove Curse*; `REMOVE_CURSE2` activation | 50 + 1d50 (51–100) |
| Priest *Remove Curse* (clvl 16), Paladin *Remove Curse* (clvl 20) | clvl + 1d(clvl) |

Outcome:

1. Curse power ≥ 100 → impossible (permanent).
2. Strength ≥ power → the curse is removed.
3. Otherwise, if the item is not yet `FRAGILE` → it **becomes fragile**
   ("The spell fails; your … is now fragile.").
4. Otherwise (already fragile) → **1 in 4: the item is destroyed** with "a bang
   and a flash" and you take 5d5 damage (artifacts are destroyed too and
   recorded as lost); 3 in 4: "The removal fails."

Practical rules:

* The first failure is free (it only makes the item fragile); every failure
  after that is a 25% gamble on the item.
* Only use weak scrolls on items you'd be happy to lose. For an artifact or
  end-game ego, wait for *Remove Curse* or a high-level priest.
* Each curse is separate; an item with three curses needs three successes.
* You can learn a curse's existence (rune) when it fires; the curse's power
  is not displayed, so treat unknown powers as "level-appropriate" per 8.1.

---

## Part 9 — Traps

### 9.1 Generation, detection and avoidance

* Traps are picked with weight `100 / rarity` among kinds whose minimum depth
  is ≤ the trap level (`trap.c`). Rarity 1 = common, 2 = half as common.
* **Noticing:** whenever a square is in view and lit (`cave-view.c`), each
  trap on it is revealed if your **Searching skill ≥ the trap's visibility
  power** (`trap.c::square_reveal_trap`). Visibility 0 traps (runes, acid
  traps, blast traps) are always seen. Blindness hides everything.
* **Trap immunity**: the `TRAP_IMMUNE` flag (the artifact Leather Boots
  *of Wormtongue*, which also give Feather Fall; the eagle shapechange) or the
  `TRAPSAFE` timed effect lets you walk over traps unharmed; the trap just
  becomes visible.
* **Saving throws per trap** (`hit_trap`):
  * `save:FEATHER` (trap door, pits): Feather Falling negates the damage
    (but you **still fall** through a trap door — "You float gently down to
    the next level").
  * `SAVE_ARMOR` (darts): the dart is avoided if a monster-style hit roll
    with `to_hit` 125 misses your AC: avoid chance ≈ 5% + 83% × (2/3·AC)/125.
    AC 100 → ~49% avoided; AC 150 → ~71%.
  * `SAVE_THROW` (mind-blast and brain-smash traps): your saving throw.
* **Disarming** (`cmd-cave.c::do_cmd_disarm_aux`): skill = disarm-physical or
  disarm-magic (for `MAGICAL` traps), divided by 10 if blind, without light,
  confused or hallucinating. Chance = skill − DL/5 (minimum 2%). Success
  removes it and gives `1 + DL/5` exp; failure has the same chance again to be
  a harmless fail, otherwise you **set it off**.
* Detection: Mage *Find Traps, Doors and Stairs* (clvl 1), Rogue-line
  *Detection*, Wand/Rod of Disable Traps (removes them), and walking with
  the disarm/"walk into" safety commands.
* Any trap that isn't `ONETIME` has a 1-in-3 chance to vanish after firing;
  otherwise it becomes visible.

### 9.2 Every trap (`trap.txt`)

| Trap | Min DL | Rarity | Visibility | Effect | Avoid / mitigate |
|---|---|---|---|---|---|
| trap door | 2 | 1 | 2d10 + m_bonus(50) | 2d8 damage, **drop one level** | Feather Fall (damage only) |
| pit | 2 | 1 | 90 | 2d6 | Feather Fall |
| spiked pit | 2 | 1 | 80 | 2d6, then 50%: +2d6 and cut 4d6 | Feather Fall |
| poison pit | 2 | 1 | 70 | 2d6, then 50%: cut 4d6 + poison 8d6 | Feather Fall; rPois vs poison |
| rune of summon foe | 3 | 1 | 0 | summons 1 monster | always visible — walk around |
| rune of summoning | 3 | 2 | 0 | summons 2+1d3 monsters | always visible |
| rune of necromancy | 20 | 1 | 0 | summons undead | always visible |
| rune of dragonsong | 20 | 1 | 0 | summons 1d3 dragons | always visible |
| hellhole | 20 | 1 | 0 | summons 1+1d3 demons | always visible |
| teleport rune | 1 | 1 | 30+1d30 | teleport (range m_bonus 80) | `NO_TELEPORT` blocks; sometimes useful |
| fire trap | 2 | 1 | 20 | fire spot 4d(DL/2) | rFire |
| acid trap | 2 | 1 | 0 | acid spot 4d(DL/2) | rAcid |
| slow dart | 2 | 1 | m_bonus(50) | 1d4 + slow 20+1d20 **(ignores FA)** | AC |
| strength loss dart | 6 | 2 | 30 | 1d4 + STR drain | AC, Sustain STR |
| dexterity loss dart | 6 | 2 | 30 | 1d4 + DEX drain | AC, Sustain DEX |
| constitution loss dart | 6 | 2 | 30 | 1d4 + CON drain | AC, Sustain CON |
| blinding gas trap | 2 | 1 | 20+1d5 | blind 25+1d50 | pBlind |
| confusion gas trap | 1 | 1 | 20+1d5 | confused 10+1d20 | pConf |
| poison gas trap | 2 | 1 | 20+1d5 | poisoned 10+1d20 | rPois |
| sleep gas trap | 2 | 1 | 20+1d5 | paralyzed 5+1d10 | **Free Action** |
| aggravation trap | 5 | 1 | m_bonus(50) | wakes everything, hastes monsters in LOS (+25) | stealth won't save you; leave |
| siren | 1 | 1 | 60+1d10 | wakes everything | — |
| mine trap | 15 | 1 | 10+1d5 | shard explosion radius 2, damage 2×DL | rShards; one-time |
| blast trap | 25 | 2 | 0 | light 2d6, sound DL/2, fire DL, force DL (radius 1–2) | rLight/rSound/rFire, pStun |
| mind blasting trap | 20 | 1 | 30+1d10 | 8d(DL/10) + confused 3+1d4 | saving throw, pConf |
| brain smashing trap | 40 | 2 | 30+1d10 | 10d(DL/5) + slow, confused, paralyzed 3+1d4, blind 7+1d8 | saving throw; FA, pConf, pBlind |
| rock fall trap | 4 | 1 | 40+1d5 | (DL/10+1)d5, **stun 2d20 (no protection)**, rubble | — |
| earthquake trap | 30 | 1 | 30+1d5 | (DL/10+1)d5 + earthquake radius 5 | — |
| block fall trap | 4 | 1 | 40+1d5 | granite falls *behind* you as you leave the square | carry a digger / Stone to Mud |
| area blast trap | 40 | 2 | 20+1d5 | clears walls radius 2, force ball (damage DL), rubble | pStun |
| blinding flash trap | 5 | 1 | 30+1d5 | light ball (DL/10+2)d8 radius 4 | rLight / pBlind |
| blinding trap | 5 | 1 | 20+1d5 | dark ball (DL/10+2)d8 radius 4, drains 100+1d100 light fuel | rDark / pBlind |
| mana drain trap | 5 | 1 | 50+1d10 | drains 1d(DL/2+1) SP | — |
| knife trap | 20 | 1 | 20+1d5 | **cut 150** ("Severe Cut") + DL/2 damage | CSW/CCW; nothing prevents |
| petrifying trap | 5 | 1 | 20+1d5 | Stone skin 20+1d20 and stun 20+1d20 | pStun vs the stun |

Key trap dangers: sleep gas without FA next to monsters; trap doors as a
low-HP emergency drop (the next level is unknown); summon runes at DL20+
(dragonsong/hellhole); aggravation in a vault; a knife trap at low HP (150
cut bleeds 2/turn). Runes are always visible — just don't step on them.

---
## Part 10 — THE KEY TABLE: resistance checklist by depth

Each band lists (a) the defences you should have **before** entering it,
(b) the monsters that justify them, with depth and the damage they can do,
and (c) an HP floor. "HP floor" = enough to survive the worst *single*
attack you are likely to meet there with the recommended defences, plus a
margin for a second hit before you can react. Monster numbers are from
`monster.txt` (listed HP, speed relative to +0, breath = HP/divisor capped).

### 10.1 Summary table

| Band | Must have | Strongly recommended | Nice to have | HP floor |
|---|---|---|---|---|
| **DL 1–10** | Light source, Phase Door, CLW potions (they cure blindness), Flasks of oil | don't melee floating eyes; some ranged attack | rFire/rCold (protect scrolls/potions) | 60–100 |
| **DL 10–20** | **Free Action** (by DL 20), See Invisible *or* a plan for ghosts/imps | rFire, rCold, rElec, rAcid starting; CSW/CCW potions | pConf, pBlind | 120–200 |
| **DL 20–30** | **Free Action**, all four base resists, See Invisible | rPois ideally by DL 30, pConf, pBlind | Hold Life, rSound/pStun, rNexus | 250–350 |
| **DL 30–40** | FA, SI, base four, **rPois by DL ~35–40**, pConf or rChaos, pBlind (or rLight+rDark) | rNexus, rSound/pStun, rShards, Hold Life, +10 speed | rNether, rDisen | 400–500 |
| **DL 40–60** | everything above + **rPois (mandatory)**, rNether or Hold Life, pConf, pBlind, rNexus | rChaos, rSound **and** pStun, rShards, rDark, +15–20 speed; Potions of Resist Heat/Cold/Poison | rDisen, rLight | 600–900 |
| **DL 60–80** | all base + rPois + rNether + rChaos + rNexus + rSound/pStun + rShards + rDark + rLight + pConf + pBlind + FA + SI + HL | rDisen; temp resists for double resist vs great wyrms; +20 speed | immunity to the commonest breath you face | 900–1200 |
| **DL 80–99** | full set incl. rDisen, pStun; temp resists on demand | +25–30 speed; several *Healing*/Life potions | immunity (fire is the most common 1600 breath) | 1200–1500 |
| **DL 99–100** (Sauron, Morgoth) | full set, pStun, FA, SI, HL, rChaos, rNether, pConf, pBlind, rNexus | +30 speed or more | double resist fire/poison when fighting Wolf/Serpent-Sauron | 1500+ |

### 10.2 DL 1–10 — "the floating eye zone"

| Threat | DL | What it does | Defence |
|---|---|---|---|
| floating eye | 1 | `GAZE:PARALYZE` (11 HP, never moves) | Don't melee it without FA; shoot or ignore |
| grey mushroom patch, brown mold | 1, 6 | melee `CONFUSE` | don't stand next to them |
| poltergeist | 3 | invisible, +20 speed, `TOUCH:TERRIFY` | ignore / SI |
| Grip, Fang | 2 | +20 speed dogs | ranged attacks, doors |
| Bullroarer | 5 | 60 HP, +10 speed melee | |
| giant white/green dragon fly | 5 | cold/poison breath ~4 | trivial |
| disenchanter eye | 5 | gaze disenchant | rDisen not needed: just kill it from range |
| Wormtongue (U) | 8 | BA_POIS 7–28, BO_COLD 8–50, SLOW, TRAPS, EAT_GOLD | FA (vs SLOW), HP > 60 |
| baby dragons | 9–11 | breath 29–38 (88–114 HP) | 80+ HP is enough |
| Orfax, Son of Boldor (U) | 10 | CONF, SLOW, TELE_TO, summons | |

Verdict: no resistances are *required*. The killers are paralysis (floating
eye), being surrounded while confused or blind (carry CLW: cures blindness and
reduces confusion), and running out of Phase Door.

### 10.3 DL 10–20 — "hold casters and invisible imps"

| Threat | DL | What it does | Defence |
|---|---|---|---|
| illusionist, druid | 13 | **HOLD**, BLIND, CONF, SLOW; druid also BO_FIRE/BO_POIS | saving throw, **Free Action** |
| moaning spirit | 12 | invisible ghost, terrify, LOSE_DEX | See Invisible, Sustain DEX |
| homunculus | 15 | `HIT:PARALYZE` | FA |
| light / dark hounds | 15 | packs, blind | pBlind or rLight/rDark |
| quasit, imp | 16–17 | invisible; TELE_LEVEL, TELE_TO, BLIND, fire bolts | SI, rFire, rNexus vs tele-level |
| Nár, the Dwarf (U) | 17 | MIND_BLAST 8d8, BLIND, CONF, WOUND 10–50 | save, pConf, pBlind |
| fire / cold / energy hounds | 18 | packs, breath 11 each, pack item damage | base resists protect HP; only immunity protects the pack |
| evil eye | 18 | **HOLD**, `EXP_10` gaze, TELE_TO | FA, Hold Life |
| Old Man Willow (U) | 19 | 1000 HP, `TOUCH:PARALYZE:1d14` ×2, HOLD, TELE_TO | **FA mandatory** |
| air hounds, earth hounds, chimaera | 20 | poison 22, shards 11, fire 53 | |

Verdict: **get Free Action before DL 20.** Start collecting base resists to
stop pack damage to scrolls/potions.

### 10.4 DL 20–30 — "ghouls, crawlers and the first real breathers"

| Threat | DL | What it does | Defence |
|---|---|---|---|
| forest wight, banshee | 24 | `EXP_20` | Hold Life (optional) |
| Ibun / Khîm (U) | 24 | `HIT:DISENCHANT`, BO_FIRE | rDisen or don't melee with your best weapon |
| necromancer | 25 | **BO_NETH 72–92**, S_UNDEAD | HP; rNether later |
| **carrion crawler** | 25 | groups of 2d4, two paralyzing stings | **FA** |
| **ghoul** | 26 | groups of 4d4, paralyzing claws + HOLD | **FA** |
| disenchanter bat | 26 | +20 speed, disenchant | |
| vampire | 27 | `EXP_20`, HOLD, MIND_BLAST | FA, HL |
| vibration hound | 27 | packs, **sound → stun** 23 per breath | rSound or pStun |
| nexus hound | 27 | packs, nexus → teleport/scramble | rNexus |
| Mîm (U) | 27 | BA_ACID 16–96, disenchant melee | rAcid |
| basilisk | 28 | +10 speed, **poison breath 103**, paralyzing gaze | rPois, FA |
| mind flayer | 28 | **BRAIN_SMASH 12d15** + paralyze/slow/conf/blind, FORGET, LOSE_INT/WIS | save, FA, pConf, pBlind, sustains |
| Draebor (U) | 28 | TELE_LEVEL, TELE_TO, TELE_AWAY | rNexus |
| young blue/white/green dragons | 29 | breath ~79 | base resists |
| young multi-hued dragon | 32 | five base breaths 93 | all base + rPois |

Verdict: **Free Action is non-negotiable at DL 25+** (ghoul packs and carrion
crawlers). Base four resists by DL 30. The basilisk (103 poison, +10 speed)
is the first monster that makes rPois worth a slot.

### 10.5 DL 30–40 — "the drakes and the exotic hounds"

| Threat | DL | What it does | Defence |
|---|---|---|---|
| shadow / crystal / chaos / law / balance drakes | 33 | nether 44, shards 66, chaos/disen 73, sound/shards 73, all four 88 | rChaos or pConf, rSound/pStun, rShards |
| ethereal drake | 33 | light/dark 58; invisible, passes walls | pBlind, SI |
| lich | 34 | BRAIN_SMASH, DRAIN_MANA, `DRAIN_CHARGES`, `LOSE_DEX`, `EXP_40` | save, FA, HL |
| master thief | 34 | +20 speed, `EAT_ITEM` | DEX + level |
| gravity hound | 35 | gravity 64: blink, slow (ignores FA), stun | pStun; HP |
| inertia hound | 35 | inertia 32: slow (ignores FA) | kill fast; speed |
| impact hound | 35 | force 32 + stun 1d20 + knockback | pStun |
| master vampire | 36 | BO_NETH, `EXP_40`, MIND_BLAST | HL, rNether |
| plasma / nexus vortex, shardstorm | 37 | plasma 29 + stun; nexus 29; shards 29 | pStun, rNexus, rShards |
| emperor wight | 37 | `EXP_80` melee, BO_NETH | Hold Life |
| time vortex | 38 | **time 58** (unresistable drains), +20 speed | kill it from range |
| death knight | 38 | BO_NETH 92–112 | HP / rNether |
| abyss spider | 38 | poison + dark breath (spell power only 10), CONF, SLOW, BLIND | rPois, pBlind, FA |
| mature white/blue/green dragons | 39 | breath ~233 | base + rPois |
| sorcerer | 40 | +20 speed, BA_FIRE 11–150, BA_COLD, TELE_TO, summons dragons/undead | resists, HP |

Verdict: by **DL 35–40 you want rPois** (mature green dragon 233, mature
multi-hued 333 at DL43), confusion protection (chaos drakes, umber hulks,
casters), and blindness protection. rNexus starts paying for itself (nexus
hounds/vortices, Draebor). HP 400+: a mature multi-hued dragon's poison is 333
unresisted.

### 10.6 DL 40–60 — "the poison wall"

| Threat | DL | What it does | Defence |
|---|---|---|---|
| Gorlim (U) | 41 | **BO_MANA 51–152**, BO_WATE | HP (unresistable) |
| master lich | 41 | BRAIN_SMASH, HOLD, `EXP_80`, DRAIN_MANA, drains charges | save, FA, HL |
| Kavlax (U) | 41 | nine breaths: 433 base, 216 chaos/nexus/shards/sound, 200 gravity | all base, rChaos, rNexus, rSound |
| mature multi-hued dragon | 43 | **five breaths 333** | base + rPois |
| Quaker, Master of Earth (U) | 43 | `SHATTER` (earthquakes), BA_ACID, BA_SHAR | rShards |
| greater basilisk | 45 | poison 336, dark/nexus 168, `GAZE:PARALYZE:3d12` ×2, POWERFUL | rPois, FA, rNexus, pBlind |
| winged horror | 48 | poison 337, nether 168, dark 168 | rPois, rNether |
| Eöl (U) | 49 | **BA_NETH 206–296**, BO_MANA 51–172, BA_ELEC/ACID | rNether, HP |
| **The Balrog of Moria (U)** | 50 | **fire breath 1000**, +20 speed | rFire (333) — plus Resist Heat (111) |
| death drake / great crystal drake / ethereal dragon | 50 | nether / shards / light+dark 325 | rNether, rShards, rLight/rDark |
| ancient dragons | 50–51 | base breath 500–520 | base resists (173) |
| time / nether / plasma hounds | 51 | packs; time 110 at +20 speed, nether 55, plasma 55 | speed, pStun, rNether |
| ancient multi-hued dragon | 53 | **five breaths 800** (poison capped 800) | **rPois mandatory** (266) |
| drolem | 54 | **poison 733–800** | **rPois mandatory** |
| dreadmaster | 54 | TELE_LEVEL, BO_NETH 116–136, HOLD, WOUND; a ghost that passes walls | rNexus, FA, rNether, SI |
| The Phoenix (U) | 54 | fire 1200, light 500, plasma 150 | rFire + temp, rLight, pStun |
| dracolich / dracolisk | 55 | cold or fire 1026; nether 513 / nexus 400 | base + temp, rNether, rNexus |
| Shelob (U) | 55 | poison 800, dark 500, `BITE:PARALYZE:5d10` | rPois, rDark, FA |

Verdict: **rPois is mandatory before DL ~50.** An unresisted ancient
multi-hued dragon or drolem breath (733–800) kills nearly any character at
these levels; resisted it is 244–266. rNether (or at least Hold Life) for
the nether casters/breathers; rNexus vs tele-level spam. Keep Potions of
Resist Heat/Cold/Poison for the Balrog of Moria and dracolich/dracolisk.

### 10.7 DL 60–80 — "great wyrms, chaos and mana storms"

| Threat | DL | What it does | Defence |
|---|---|---|---|
| aether vortex | 60 | 17 breath types, base/time/gravity 140 | everything |
| bile demon | 61 | acid 821, poison 800 | rAcid, rPois |
| dreadlord | 62 | BA_NETH 258–348, `EXP_40` melee | rNether, HL |
| Smaug (U) | 62 | fire 1400 | rFire + temp (155) |
| werewolf of Sauron, spider of Gorgoroth | 63 | nether/dark 366; poison 800/dark 500 | rNether, rDark, rPois |
| chaos hound | 64 | packs, **chaos 155 each** | **rChaos** |
| archlich | 64 | BA_NETH 266–356, S_HI_UNDEAD | rNether |
| **The Mouth of Sauron (U)** | 65 | invisible; **BA_MANA 335–425**, BA_NETH/BA_DARK 270–360, BA_WATE, HOLD, disenchant + drain-charges melee | SI, FA, rDisen, HP |
| jabberwock | 68 | chaos 469 at +20 speed | rChaos (≤312) |
| hound of Tindalos | 69 | +20 speed packs: time 150, nether 80, disen 80 | rNether, rDisen; HP for time |
| gelugon | 69 | cold 1026, shards 500, `HIT:PARALYZE` | rCold+temp, rShards, FA |
| great storm / ice / swamp wyrms | 70 | elec / cold 1400, poison 800 | resist + temp (155 / 88) |
| Ossë (U) | 72 | disen 500, BA_WATE, STORM | rDisen, pConf, pStun |
| great hell / bile wyrm, wyrm of thunder | 73 | fire / acid 1500, sound 500 | resist + temp; rSound |
| aether hound | 74 | 18 breath types, base 410, +20 speed, packs | everything |
| black reaver | 74 | BA_MANA 380–470 | HP |
| great wyrms of chaos / law | 75 | chaos 600 + disen 500 / shards 500 + sound 500 | rChaos, rDisen, rShards, rSound |
| Glaurung (U), Ungoliant (U) | 75 | fire 1600 + poison 800; dark 500 + poison 800 | double resists, rDark |
| pit fiend | 77 | fire 1173, poison 800, chaos 586 at +20 | rFire+temp, rPois, rChaos |
| Feagwath (U) | 77 | BA_MANA 395–485 | HP |
| Nazgûl (U) | 77–84 | `BLACK_BREATH`, `EXP_80`/`EXP_40`, BA_NETH, BO_MANA, HOLD | FA, HL, rNether; Herbal Curing |
| great wyrm of annihilation | 80 | poison 800, nether/chaos 600, dark 500, time 150 | rPois, rNether, rChaos, rDark |
| sky dragon | 80 | elec 1600, light 500, gravity/force 200, plasma 150 | rElec+temp, rLight, pStun |

Verdict: every high resist now matters; the unresistable damage
(mana storms ~400–485, time 150, plasma 150, gravity 200) sets the HP floor.
Carry temporary resists for great wyrms: resisted 1400–1600 is 466–533,
double-resisted 155–177.

### 10.8 DL 80–99 — "everything at the cap"

| Threat | DL | What it does | Defence |
|---|---|---|---|
| great wyrm of many colours | 82 | acid/cold/elec/fire 1600, poison 800 | resist all + temp (177/88) |
| great wyrm of balance | 82 | chaos 600, disen/shards/sound 500 | high resists |
| Khamûl (U) | 84 | Black Breath, `EXP_40`, TELE_LEVEL, BA_NETH, BO_MANA | rNexus, HL, rNether |
| The Witch-King (U) | 85 | BA_NETH 350–440, BO_MANA 51–262, HOLD, BRAIN_SMASH, `EXP_80` ×2, Black Breath | FA, rNether, HL, pConf, pBlind |
| greater Balrog | 85 | fire 1500, plasma 150, BA_DARK, POWERFUL (fire side effects live) | rFire + temp, pStun |
| Ancalagon the Black (U) | 85 | 1600 ×4 base, poison 800, +20 speed | double resists |
| The Tarrasque (U) | 86 | cold/fire 1600, disen 500 | double resists, rDisen |
| Lungorthin (U) | 88 | fire 1600, plasma 150 | double rFire, pStun |
| Carcharoth / Huan (U) | 90 | fire or cold 1600 plus poison 800 / nether 600 / dark or light, shards, sound 500 | double resists, full high resists |
| Vecna (U) | 92 | **BA_MANA 470–560**, BA_NETH 378–468, BRAIN_SMASH, EXP_80 melee | HP, rNether, FA |
| storm of Unmagic | 95 | time 150 + chaos/disen 100 at +30 speed, `LOSE_ALL` melee | sustains, rChaos, rDisen; speed |
| Gothmog (U) | 95 | fire 1600, plasma 150, `HIT:FIRE:9d12` ×2 | double rFire, pStun |

### 10.9 DL 99–100 — Sauron and Morgoth

| Foe | Attacks (raw damage, before resistance) | With full resists |
|---|---|---|
| Sauron (power 130, 8000 HP, +20) | BA_NETH 530–620, BA_DARK 530–620, BA_FIRE 11–465, BO_MANA 51–375, BO_PLAS 148–196, BO_ICE 136–166, WOUND 86–430 + cut 16d10 (save negates), BRAIN_SMASH, TELE_LEVEL; melee `DISENCHANT` 10d12 ×2 | nether/dark ≤ 413; fire ≤ 155 (≤51 double); mana and plasma unchanged |
| Wolf-Sauron (+30) | breathes fire and nether from Sauron's HP: **up to 1600 fire, 600 nether**; BA_DARK; `CLAW:PARALYZE:6d8`, `CLAW:CONFUSE` | fire 533 (177 double), nether 400 |
| Serpent-Sauron | poison 800, chaos 600, acid 1600 (800 after armour); BRAIN_SMASH, FORGET | poison 266 (88), chaos 400, acid 266 (88) |
| Vampire-Sauron (power 130) | BE_NETH 295–315, BA_NETH 530–620, STORM, BO_PLAS, BA_POIS, BA_ACID, BRAIN_SMASH, FORGET; `BITE:EXP_80` | nether ≤ 413 |
| Morgoth (power 100, 20000 HP, +30) | **BA_MANA 510–600 (no resist)**, BA_NETH 410–500, BO_MANA 51–300, BRAIN_SMASH; melee `SHATTER` 20d10 ×2 (earthquakes, big stun crits), `LOSE_ALL` 10d12 | mana 600 unchanged; nether ≤ 333 |

HP floor reasoning for the endgame: Morgoth's worst mana storm is 600 and
cannot be reduced except by `DAM_RED` items; with +30 speed you get 1 action
per Morgoth action; a mana storm + a SHATTER round between your turns can
exceed 800. Wolf-Sauron resisted fire (533) + a resisted nether (400) is 933.
Hence **1500+ HP** (or very high speed) before engaging.

### 10.10 HP thresholds — what one hit can do

Worst single hit for each attack, by your defence level:

| Attack | Unresisted | Resisted | Double | First appears (DL) |
|---|---|---|---|---|
| Fire/cold/elec breath at cap | 1600 | 533 | 177 | GW of many colours 82 (Balrog of Moria 1000 at 50) |
| Acid breath at cap (armour on) | 800 | 266 | 88 | great bile wyrm 73 |
| Poison breath at cap | 800 | 266 | 88 | AMHD 53, drolem 54 |
| Ancient dragon base breath | ~520 (up to ~780 for a +4σ spawn) | 173–260 | 57–86 | DL 50–51 |
| Chaos / nether breath at cap | 600 | 400 | — | GW chaos 75 / GW annihilation 80 |
| Sound/shards/disen/light/dark at cap | 500 | 333 | — | Phoenix 54 (light), Shelob 55 (dark) |
| Nexus at cap | 400 | 266 | — | dracolisk 55 |
| Gravity / inertia / force | 200 | — | — | Kavlax 41, aether hounds 74, sky dragon 80 |
| Time / plasma | 150 | — | — | hound of Tindalos 69 (time 150), Phoenix 54 (plasma 150) |
| Mana storm | 600 (Morgoth) | — | — | Mouth of Sauron 65 (425) |
| Nether ball | 620 (Sauron) | 413 | — | Eöl 49 (296) |

Rule of thumb: **keep max HP above twice the worst resisted hit of the deepest
monster you expect to meet**, because breaths come from fast monsters and
packs. At DL 50 that is 2 × 266 (resisted AMHD/drolem poison) ≈ 550 — but an
*unresisted* poison breath at 800 makes the real answer "have rPois".

---

## Part 11 — Instant-death scenarios and their counters

| # | Scenario | Mechanism (code) | Counter |
|---|---|---|---|
| 1 | Poison breath without rPois at DL 45–60 | AMHD/drolem 733–800 unresisted, 10+1d(dam) turns of poison and no regen afterwards | rPois before DL ~45; Potions of Resist Poison stack to 1/9 |
| 2 | Fire breath of the Balrog of Moria (DL50, 3000 HP, +20 speed) | 1000 fire, unresisted; full strength to 7 squares | rFire + Resist Heat (111); fight at range; wound it first (breath ∝ current HP) |
| 3 | Paralysis chain | NONSTACKING but re-applied the turn it lapses; ghoul packs 4d4, carrion crawlers, basilisk gaze, Shelob, HOLD casters | **Free Action**; avoid melee range of gazers |
| 4 | Knocked out by stun | Stun >150 → no turns; sound/plasma/gravity add up to 35 each; melee crits add 100–200 | **pStun**, rSound; CCW potions cure stun if you still get a turn |
| 5 | Hound-pack breath swarms | 2d7 hounds, +10/+20 speed, ~1/5 innate frequency → several breaths per player turn (time hounds ≈ 3 × 110 per turn) | fight in corridors so one hound has LOS; speed; resist the element; leave the level |
| 6 | Mana storms | BA_MANA 5×power + 10d10, no resist: Mouth of Sauron 425, black reaver 470, Feagwath 485, Vecna 560, Morgoth 600 | HP above the max; `DAM_RED`; kill or teleport them first; never fight with < max-storm HP |
| 7 | Confused + blind lock-out | Cannot read Phase/Teleport/Recall, cannot cast; random movement | pConf, pBlind; quaff CSW/CCW (cures both) — potions still work when confused/blind; staff of Teleportation still usable |
| 8 | Amnesia / FORGET | Cannot read scrolls; spells 50%+ fail | CCW/Healing cure amnesia; carry potions not scrolls as the backup escape |
| 9 | Anti-teleportation curse | `NO_TELEPORT`: every teleport, including Teleport Level/Deep Descent-type and Phase, says "Teleportation forbidden!" | Never wear an item with that rune in the dungeon |
| 10 | Teleport-to into a pack | TELE_TO has no save and no resist | Escapes ready at all times; know the caster list (Orfax 10, tengu 10, quasit/imp 16–17, evil eye 18, Lorgan 36, master lich 41, …) |
| 11 | Teleport-level / trap door at low HP | lands you on an unknown level | rNexus blocks the monster spell; Feather Fall only avoids the 2d8 |
| 12 | Unresistable drains (time, Black Breath) | Ignore Hold Life and sustains | Kill time breathers from range; Herbal Curing for Black Breath; Potion of Life / gaining a level restores stats |
| 13 | Losing your potions | Cold hits destroy potions: 3% per potion per hit at ≥60 damage, resistance doesn't lower the cap | Cold **immunity** or keep emergency potions spread out; flee cold-breathing packs early |
| 14 | Starvation/fainting next to enemies | Faint → 1-in-10 per turn paralysis that ignores FA | Always carry food; Slow Digestion |
| 15 | Mortal Wound | Cut >1000 never heals by itself, 3 HP/turn | CSW or better cures all cuts |
| 16 | Disenchant/drain-charges attrition | Mouth of Sauron, Sauron, liches, Balrogs eat wand/staff charges and enchantment | rDisen; kill quickly; don't melee disenchanters with an unprotected artifact |
| 17 | Cowardice curse on a melee character | permanent `AFRAID` flag ⇒ cannot attack | Remove the item; HERO/BOLD do not override the equipment flag |
| 18 | Earthquake from SHATTER blows | radius dam/12 earthquake around the monster; can bury you or separate you from escapes | AC (reduces SHATTER damage); fight Morgoth/Quaker in open areas with escapes ready |

---

## Part 12 — One-page cheat sheet

**Resistance math:** base five ×1/3 (resist), ×1/9 (resist + timed); high
elements ×2/3 worst case; vulnerable ×4/3; immune 0 (only acid/elec/fire/cold);
acid halved by armour first.

**Breath = current HP / divisor, capped:**
base 4 → /3, cap 1600 · poison /3, 800 · nether, chaos /6, 600 ·
light, dark, sound, shards, disen /6, 500 · nexus /6, 400 · gravity /3, 200 ·
inertia, force /6, 200 · time /3, 150 · plasma /6, 150.

**What stops what:**

| Problem | Stopped by | Not stopped by |
|---|---|---|
| Paralysis | Free Action; saving throw (HOLD, melee) | — (except hunger fainting, Salt Water) |
| Slow | Free Action (spells, potion) | inertia, gravity, dark, slow darts, post-Sprint |
| Confusion | pConf, `OPP_CONF` | — |
| Blindness | pBlind; rLight/rDark vs those elements | — |
| Fear | pFear, Hero, Berserk, Bold | cowardice-curse flag |
| Stun | pStun; rSound vs sound | rock-fall trap |
| Cuts | rShards vs shards/ice | melee crits, WOUND, knife trap |
| Hallucination | rChaos | — |
| Exp drain | Hold Life (nether, chaos, cold, dark, melee 50–95%); rNether vs nether | time, Black Breath |
| Stat drain | sustains | time, Black Breath, nexus scramble (rNexus) |
| Teleport-level (monster) | rNexus, saving throw | — |
| Teleport-to / away | `NO_TELEPORT` curse only | rNexus |
| Item theft | DEX + clvl save | paralysis (auto-fail) |
| Pack destruction | immunity | resistance (cap 3%/item stays) |

**Order of acquisition** (from the depth table): Free Action → See Invisible →
rFire/rCold/rElec/rAcid → rPois → pConf/pBlind → rNexus → rSound/pStun →
rNether/Hold Life → rChaos → rShards/rDark/rLight → rDisen → temporary
resists for double resistance → immunity if lucky.
