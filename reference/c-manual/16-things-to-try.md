# 16. Things to try

A laboratory of strategy ideas that nobody has measured. Each idea says what to try, why it might work
(with the mechanic cited from the game source), what can go wrong, and a concrete protocol for testing
it, by hand or with an automatic player. The chapter opens with a short testing methodology and a
template for recording what happened.

> **Everything in this chapter is UNTESTED.** The mechanics cited are facts from the source; the claim
> that exploiting them helps is a hypothesis. Do not quote an idea from here as established advice
> until a test result has been recorded with the template in Section 16.3.

Contents

- [16.1 Methodology](#161-methodology)
- [16.2 How the ideas are written](#162-how-the-ideas-are-written)
- [16.3 Template for recording a result](#163-template-for-recording-a-result)
- [16.4 Character builds](#164-character-builds)
- [16.5 Early game](#165-early-game)
- [16.6 Diving](#166-diving)
- [16.7 Combat tricks](#167-combat-tricks)
- [16.8 Spell use](#168-spell-use)
- [16.9 Item use](#169-item-use)
- [16.10 Shopping and money](#1610-shopping-and-money)
- [16.11 Exploration](#1611-exploration)
- [16.12 Uniques](#1612-uniques)
- [16.13 Endgame](#1613-endgame)
- [16.14 Unusual and fun](#1614-unusual-and-fun)
- [16.15 Index of ideas](#1615-index-of-ideas)
- [Things to try (untested)](#things-to-try-untested)

---

## 16.1 Methodology

### Two kinds of test

| Kind | Use for | Tooling |
|---|---|---|
| **Population runs** (borg-zoo style) | ideas a borg can execute: dive policies, heal thresholds, valuation, exploration | the tracker's borg harnesses: the native measurement runner (`tools/measure-native.py`), `borgh` population runs and borg-zoo suites, described in [TUNABLE-BORG-PLAN](../c-borg/TUNABLE-BORG-PLAN.md) |
| **Controlled encounters** | single-mechanic questions: does range change cast rate, does a glyph hold Morgoth, does Single Combat work on a questor | debug mode (the game's wizard commands include jumping to a level and summoning a named monster: `CMD_WIZ_JUMP_LEVEL`, `CMD_WIZ_SUMMON_NAMED` in `src/cmd-core.h`); a saved character copied before each trial |

Debug-mode characters are marked as cheaters; that is fine for experiments, not for score.

### Controls

1. **Paired seeds.** Run control and variant on the *same* seeds; compare per seed. The tracker's plan
   uses frozen seed sets (identity, development, sealed test) and paired comparisons
   ([TUNABLE-BORG-PLAN](../c-borg/TUNABLE-BORG-PLAN.md), "Tune by racing on a frozen seed population").
2. **Fix birth options.** Record every `birth_*` option (`src/list-options.h`): connected stairs,
   force descend, no recall, no selling, start kit, AI learn, feelings, know runes/flavors, persistent
   levels, percent damage. One changed option invalidates a comparison.
3. **Fix race/class**, or stratify by class: on the C borg, death rates range from 19% (Paladin) to 70%
   (Druid), and the same setting can help one class and hurt another (munchkin start: warriors 32% → 7%
   deaths, mages 31% → 45%) ([TUNABLE-BORG-PLAN](../c-borg/TUNABLE-BORG-PLAN.md)).
4. **Watch hidden inputs.** The plan notes that "seasonal monster selection still depends on the
   calendar": fix the system date or record it.
5. **Prespecify the metric** before running (below), and do not change it after seeing results.

### Metrics

| Metric | When to use |
|---|---|
| **Alive at turn T having reached DL D** (the tracker's primary objective, e.g. DL10 at 400,000 game turns) | any policy change; rewards neither stalling nor suicidal diving |
| Deaths by cause (killer, depth, CL, what was unused in the pack) | to find which mechanic an idea affects |
| Max depth and CL distributions at T | progress |
| Game turns to CL20 / to DL20 / to DL40 | speed of progress |
| Consumables used per 10,000 game turns | cost of a policy |
| Operational outcomes: stall, wall timeout, crash | liveness; never relabel as deaths, never silently drop |
| For encounters: damage taken per monster turn, spells cast per 100 monster turns, kill/death counts | single-mechanic tests |

### Sample size

Death-rate differences need many runs. Paired binary outcomes (alive/dead on the same seed) are best
analysed with McNemar's test on the discordant pairs (seeds where only one arm died). As a rough guide:
detecting a drop from 30% to 20% deaths with paired seeds typically needs a few hundred seeds per arm;
detecting 30% → 15% needs fewer. The tracker's plan sizes its comparisons from a pilot that measures
the discordant-pair rate; do the same. For human play, a handful of characters can only reveal very
large effects; record them anyway, marked "anecdotal".

### Recording

Every trial gets a record (Section 16.3), including failed and inconclusive ones. Record the source
commit of the game (`e473720ed` for this manual) and of the borg.

---

## 16.2 How the ideas are written

Each idea has:

- **Idea** — what to do.
- **Why it might work** — the mechanic, cited.
- **Risks** — what could go wrong, including how it could kill.
- **Test** — a protocol: population or encounter, arms, N, metric.

IDs are `T-<topic>-<n>` so results can refer to them.

---

## 16.3 Template for recording a result

Copy this block into a results log (one per trial):

```markdown
### Result: T-XXX-n <short name>
- Date / tester:
- Game commit: e473720ed   Borg commit (if any):
- Birth options: connect_stairs=Y force_descend=N no_recall=N no_selling=Y start_kit=Y ai_learn=Y
  feelings=Y know_runes=N know_flavors=N levels_persist=N percent_damage=N
- Race/class (or stratification):
- Kind: population | encounter | anecdotal
- Arms: control = ..., variant = ...
- Seeds / N per arm:
- Prespecified metric:
- Result (numbers, with intervals):
- Deaths by cause (control vs variant):
- Operational outcomes (stalls, timeouts, crashes):
- Verdict: supported | not supported | inconclusive | harmful
- Notes / follow-ups:
```

Keep the verdict honest: "inconclusive" is the normal outcome of a small test.

---

## 16.4 Character builds

### T-BUILD-1 Druid fox form for Free Action at CL1

- **Idea.** A druid casts *Fox Form* (level 1, 2 mana, `class.txt`) when it must melee a paralyser
  (floating eye, carrion crawler, ghoul) before finding Free Action.
- **Why.** The fox shape grants `FREE_ACT` and `FEATHER`, +5 stealth, +1 blow and +1 moves
  (`lib/gamedata/shape.txt`); `MOVES[1]` halves the energy of a step
  (`src/player-util.c:energy_per_move()`: `100 × (1+|n|−n)/(1+|n|)` = 50).
- **Risks.** Item commands ask you to resume normal shape first (`player_get_resume_normal_shape()` in
  `src/cmd-obj.c`), costing time in an emergency; STR −3.
- **Test.** Druid population, two arms (fox-form-for-paralysers vs never), 200 paired seeds, metric:
  paralysis-related deaths and alive at T=400k having reached DL15.

### T-BUILD-2 Pukel-man as a poison shield for DL40-55

- **Idea.** A druid of CL30+ uses *Become Pukel-man* (`class.txt`: level 30, 20 mana) when meeting
  poison breathers (hydras, drolems, AMHD) before having permanent rPois.
- **Why.** The Pukel-man shape gives `RES_POIS[3]` (immunity level), `RES_SHARD`, `DAM_RED[10]`,
  `PROT_STUN`, `REGEN`, `HOLD_LIFE`, +20 saving throw, but −5 speed (`shape.txt`).
- **Risks.** −5 speed raises every monster's actions per turn; no item use without reverting.
- **Test.** Encounter: debug-summon an ancient multi-hued dragon at DL53 against a CL35 druid, 30 trials
  per arm (Pukel-man vs normal shape + potion of Resist Poison); metric: damage taken per breath,
  survival to kill.

### T-BUILD-3 Eagle's Flight as an escape walker

- **Idea.** A druid of CL35 escapes on foot in eagle form instead of reading Teleport.
- **Why.** Eagle shape: `MOVES[3]` (a step costs 25 energy, four steps per turn), `FREE_ACT`,
  `SEE_INVIS`, `PROT_CONF`, `PROT_BLIND`, `PROT_FEAR`, `TRAP_IMMUNE` (`shape.txt`).
- **Risks.** Spells and breaths still reach you while you walk; no item use in shape.
- **Test.** Encounter: flee from a +10 pack (hellhounds DL42) to a known staircase 20 grids away;
  30 trials, eagle walk vs Phase+walk; metric: damage taken before the stairs.

### T-BUILD-4 Saving-throw build for the endgame

- **Idea.** Choose races/classes and gear that maximise saving throw for the Sauron fight.
- **Why.** A successful save negates the whole spell for spells with a save message: WOUND (430 max at
  power 130), BRAIN_SMASH, TELE_LEVEL (`src/mon-spell.c`: `randint0(100) < skill_sav`). Class
  saving-throw bases and per-level increments are in `class.txt` (`skill-save`).
- **Risks.** Sauron's biggest spells (BA_NETH, BA_DARK, BO_MANA) have no save.
- **Test.** Encounter: Sauron in debug mode, characters differing only in saving throw (e.g. 60% vs
  85%); 20 trials each; metric: damage taken per 100 Sauron turns.

### T-BUILD-5 Blackguard: fight instead of rest

- **Idea.** A blackguard never rests to heal mid-level; it relies on combat regeneration.
- **Why.** Blackguards have `COMBAT_REGEN` (`class.txt` `player-flags`). Resting costs game time during
  which monsters regenerate and wander.
- **Risks.** Entering fights at low hit points.
- **Test.** Blackguard population, 200 paired seeds; arms: rest-when-below-70% vs fight-when-safe-by-
  FIGHT-02 ([Chapter 15](15-notes-for-the-borg.md)); metric: game turns to DL20, deaths.

### T-BUILD-6 Necromancer that never lights a torch

- **Idea.** Play a necromancer entirely without a light source.
- **Why.** Necromancers have `UNLIGHT` (`class.txt`); light gear is adjusted for UNLIGHT players
  (`src/player-calcs.c:calc_light()`), and the C borg exempts necromancers from its light rule
  (`borg_prepared_aux()`). See [Chapter 14](14-class-playbooks.md) for what UNLIGHT gives.
- **Risks.** Reading scrolls needs light (`player_can_read()` fails with "You have no light to read by").
- **Test.** Necromancer population, arms light/no-light, 200 seeds; metric: alive at T having reached
  DL20; deaths where an escape scroll could not be read.

---

## 16.5 Early game

### T-EARLY-1 The oil-flask opener

- **Idea.** At CL1 spend most starting gold on Flasks of Oil (always stocked in the General Store) and
  throw them at every dangerous early monster (Fang, Grip, Bullroarer, Wormtongue).
- **Why.** Flasks are `EXPLODE | THROWING`; thrown, they are "considered lit, so it does more than
  nominal damage (which occurs even against creatures resistant to fire)" (`object.txt`).
- **Risks.** Less armour; flasks run out mid-fight.
- **Test.** Warrior and mage populations, 200 paired seeds each; arms 20 flasks vs default start
  shopping; metric: deaths before CL10, turns to CL10.

### T-EARLY-2 Corridor-only rule to CL10

- **Idea.** Never fight in a room before CL10; retreat to a corridor first.
- **Why.** In a corridor at most one or two monsters reach you; packs of +10 animals (cave spiders DL2,
  wolves DL10) lose their numbers advantage.
- **Risks.** Retreating costs turns; archers keep shooting.
- **Test.** Borg variant with FIGHT-03 ([Chapter 15](15-notes-for-the-borg.md)) forced on, 200 seeds;
  metric: deaths to packs before CL10.

### T-EARLY-3 Level-feeling filter for weak characters

- **Idea.** Below CL10, leave any level whose danger feeling is 1-4 immediately.
- **Why.** The danger digit is `mon_rating / depth` thresholded (`src/generate.c:calc_mon_feeling()`):
  1 = "Omens of death", 4 = "You feel anxious". It is shown on arrival.
- **Risks.** Level bouncing (cap it with STAIR-05).
- **Test.** 200 paired seeds; arms filter on/off; metric: deaths before CL10; level changes per kill.

### T-EARLY-4 Ranged-only warrior until CL15

- **Idea.** A warrior uses only a sling/bow and thrown weapons until CL15, meleeing only weak monsters.
- **Why.** Paralysers that punish melee (floating eye DL1, homunculus DL15) and jellies that damage
  gear are harmless at range. Throwing weapons can sit in the quiver, each counting as 5 missiles toward
  a slot's capacity of 40 (`carry-cap:thrown-quiver-mult:5`, `quiver-slot-size:40`, `constants.txt`;
  `src/obj-gear.c`).
- **Risks.** Ammunition cost; slower kills.
- **Test.** Warrior population, 200 seeds; metric: deaths before CL15, turns to CL15.

### T-EARLY-5 Early Free Action at any price

- **Idea.** Save every gold piece from CL1 for Free Action from the Black Market.
- **Why.** Paralysis without Free Action is lethal from DL13 (druid, illusionist HOLD) to DL26 (ghouls);
  `PARALYZED` fails against `FREE_ACT` (`player_timed.txt`).
- **Risks.** Weaker gear elsewhere; the Black Market may never stock it.
- **Test.** Population, 200 seeds; metric: paralysis-related deaths before DL30; turns to DL30.

---

## 16.6 Diving

### T-DIVE-1 Deep Descent from town as a fast "recall down"

- **Idea.** Read Deep Descent in town instead of Word of Recall when you want to return deeper than your
  max depth.
- **Why.** The handler computes the target from `player->max_depth + 5` and only checks
  `target_depth > player->depth` (`effect_handler_DEEP_DESCENT()`); there is no town check in that
  handler. Its delay is 4-7 world ticks against recall's 15-34 (`src/game-world.c`).
- **Risks.** You arrive five levels below your max depth, off-stairs. Whether it works from town must
  be verified first (encounter test).
- **Test.** Encounter: debug character in town with max depth 20 reads Deep Descent; record arrival
  depth. Then population: arms "recall" vs "deep descent when go-deeper test passes for max+5".

### T-DIVE-2 Deep Descent only when current depth = max depth

- **Idea.** Use Deep Descent as an exact +5 skip, never from shallower levels.
- **Why.** From max depth it is exactly +5 (stopping at quest levels, `dungeon_get_next_level()`).
- **Risks.** Five levels of threat growth at once.
- **Test.** Population, 200 seeds, scrolls supplied by a debug start kit in both arms (one arm never
  reads them); metric: turns to DL30, deaths DL20-35.

### T-DIVE-3 Kit-gated vs CL-gated depth

- **Idea.** Replace "CL ≥ depth" with DEPTH-01/02 ([Chapter 15](15-notes-for-the-borg.md)).
- **Why.** XP per kill is `mexp × mlevel / plevel` (`src/mon-util.c`); deep kills level you faster.
- **Risks.** Underestimating unknown monsters.
- **Test.** Borg population, 300 paired seeds, all classes stratified; metric: alive at T=400k having
  reached DL10 and DL20; deaths by cause.

### T-DIVE-4 Deliberate recall-depth reset

- **Idea.** After an over-deep dive, read recall on a shallower level and answer yes to "Set recall
  depth to current depth?", so later recalls land shallower.
- **Why.** `effect_handler_RECALL()` sets `recall_depth = max_depth = depth` on yes.
- **Risks.** Lowering max depth also lowers Deep Descent targets (useful) and loses nothing else.
- **Test.** Population: borg variant that resets when DEPTH-01 fails at max depth vs one that walks up;
  metric: deaths on the first level after a recall.

### T-DIVE-5 Object-feeling time budget

- **Idea.** Explore only until the object feeling appears; stay for full exploration only if it is 1-5
  ("wondrous" to "good treasures"), else head for the stairs.
- **Why.** The object feeling needs some exploration (`world:feeling-need:10` in `constants.txt`;
  `display_feeling()`); its scale is fixed in `obj_feeling_text[]` (`src/cmd-cave.c`).
- **Risks.** Missing unseen good items on "boring" levels.
- **Test.** Population, 200 seeds; metric: item value found per 10,000 turns, deaths, turns per level.

### T-DIVE-6 Force-descend training

- **Idea.** Play with `birth_force_descend` to learn how deep a character can survive without returning.
- **Why.** Up stairs are never made and recall cannot be used on quest levels
  (`effect_handler_RECALL()`, `do_cmd_go_down()`); the official guide recommends trying "as though the
  iron-man stairs setting is in effect".
- **Risks.** Much higher death rate; it is a training regime, not a winning one.
- **Test.** Human: five characters per class; record max depth and cause of death; compare with your
  normal results.

---

## 16.7 Combat tricks

### T-CMB-1 Fight casters adjacent, not at range 4

- **Idea.** Close to melee range with frequent casters, or break line of sight; never trade at range 4.
- **Why.** Cast chance doubles at the monster's `best_range`, which for casters with frequency > 24% is
  its minimum range + 3 (`get_move_find_range()`, `monster_can_cast()`).
- **Risks.** Melee damage from casters that also hit hard.
- **Test.** Encounter: a sorcerer (DL40, 1-in-2) at distance 1 vs 4, 30 trials each; metric: spells per
  100 monster turns, damage taken.

### T-CMB-2 Long-range breath duels

- **Idea.** Fight non-powerful breathers from 7+ grids.
- **Why.** Breath damage at distance *d* is `dam × 4/(d+1)` (source diameter 4, 6 if `POWERFUL`):
  half at 7, a third at 11 (`effect_handler_BREATH()`, `project()`).
- **Risks.** Frequent breathers prefer point blank and will close in; archers and casters among them.
- **Test.** Encounter: mature dragons (DL39-43), shooting from 8 vs 2; metric: damage taken per breath.

### T-CMB-3 Pillar dancing

- **Idea.** Against a same-speed melee-only monster, step around a pillar so it never gets adjacent at
  the start of its turn, and shoot when it lines up.
- **Why.** With equal energy, turn order alternates; a monster following you around a 1×1 pillar can be
  kept one step behind.
- **Risks.** Monster speed variance for non-uniques (±energy/10, `src/mon-make.c`) breaks parity;
  spellcasters ignore it.
- **Test.** Encounter: a +0 melee monster, 30 trials; metric: hits taken per kill vs straight melee.

### T-CMB-4 Doors against non-door monsters

- **Idea.** Close doors behind you against monsters lacking `OPEN_DOOR` and `BASH_DOOR`.
- **Why.** Door movement depends on those flags (`monster.txt`, `monster_base.txt`); hounds and many
  animals lack them.
- **Risks.** Pass-wall and kill-wall monsters ignore doors.
- **Test.** Population: borg variant that closes doors after passing when hounds are detected; metric:
  hound damage per encounter.

### T-CMB-5 Teach smart casters your resists

- **Idea.** Let a smart caster see you resist an element before the real fight, so it stops using it.
- **Why.** With `birth_ai_learn` (default on), monsters learn your resists and flags
  (`src/mon-util.c:update_smart_learn()`) and filter spells by that knowledge (`src/mon-attack.c`),
  but forget with a 1-in-20 chance per check.
- **Risks.** The remaining spells may be worse (mana, nether).
- **Test.** Encounter: a smart caster with fire and nether options; count fire casts over the first and
  last 50 casts of long fights.

### T-CMB-6 Taunt casters into melee

- **Idea.** Blackguards taunt frequent casters.
- **Why.** Under `TMD_TAUNT` monsters halve their cast chance and stop keeping their distance
  (`monster_can_cast()`, `get_move_find_range()`).
- **Risks.** Every monster in view comes to you.
- **Test.** Encounter: dúnadan of Angmar group (DL36), taunt vs no taunt, 30 trials; metric: spells per
  100 monster turns, damage taken.

### T-CMB-7 Stone to Mud on golems

- **Idea.** Use the Wand of Stone to Mud (magic shop stock) as a weapon against rock-hurt monsters.
- **Why.** Stone-to-mud projections damage monsters vulnerable to rock (`HURT_ROCK` flag in
  `monster.txt`; see [Chapter 7](07-consumables-and-devices.md) for the damage).
- **Risks.** Low charges; misidentifying which golems are affected.
- **Test.** Encounter on several golems; metric: turns to kill vs best other attack.

---

## 16.8 Spell use

### T-SPELL-1 Detection on every arrival

- **Idea.** Spend the first action on every level on monster detection, always.
- **Why.** Most early deaths are monsters the character did not know were near; detection spells exist
  in every caster book 1 (*Detect Monsters*, *Detect Evil*, *Detect Life*, *Read Minds*; `class.txt`).
- **Risks.** Mana spent; slightly slower play.
- **Test.** Caster populations, 200 seeds; metric: deaths within 100 game turns of arrival.

### T-SPELL-2 Tap Magical Energy as a mana battery

- **Idea.** A mage carries spare wands to drain with *Tap Magical Energy* (level 22) during long fights.
- **Why.** The spell converts device charges to mana (`class.txt`, book *Magical Defences*).
- **Risks.** Pack weight; losing useful wands.
- **Test.** Mage population DL25-40, arms with/without; metric: deaths with mana empty.

### T-SPELL-3 Glyph fortress for priests

- **Idea.** A priest fights tough melee monsters from a glyph (*Glyph of Warding*, level 20).
- **Why.** A monster breaks a glyph with probability `(level − 1)/550` per attempt (`mon-play:break-
  glyph:550`; `src/mon-move.c`, `randint1(glyph_hardness) < level`): about 4% for a DL20 monster,
  18% for Morgoth.
- **Risks.** Ranged attackers ignore glyphs.
- **Test.** Encounter: trolls (DL25-35) against a CL25 priest; metric: melee damage taken per kill.

### T-SPELL-4 Ranger decoys against breathers

- **Idea.** Place a *Decoy* (ranger level 30) before engaging a dragon.
- **Why.** Monsters that are decoyed target the decoy with breaths (`effect_handler_BREATH()`:
  `monster_is_decoyed(mon)` → `cave_find_decoy()`).
- **Risks.** The arc may still cover you; the decoy's lifetime.
- **Test.** Encounter: ancient dragons (DL50), 20 trials each arm; metric: breath damage taken.

### T-SPELL-5 Single Combat for isolating a unique

- **Idea.** A paladin (level 40 spell *Single Combat*) isolates a dangerous unique away from its escort.
- **Why.** `effect_handler_SINGLE_COMBAT()` moves you and the target into an arena; a monster resists if
  `randint0(spell_power) > player level` (Morgoth at CL50: about 49% resist; Sauron at power 130: about
  61%).
- **Risks.** No recall from the arena (`effect_handler_RECALL()`), no *Destruction*
  (`effect_handler_DESTRUCTION()`), and Teleport, Teleport To and Teleport Level return without effect
  inside an arena (`arena_level` checks in `src/effect-handler-general.c`); a lost duel is death.
- **Test.** Encounter first: does the arena allow summons; does a questor kill inside complete the quest
  (`quest_check()` tests `cave->depth`). Then a handful of controlled unique fights.

---

## 16.9 Item use

### T-ITEM-1 Speed potion before every unique fight

- **Idea.** Quaff a Potion of Speed at the start of every unique fight from DL20 on.
- **Why.** `TIMED_INC:FAST` for 20+2d10 turns (`object.txt`) gives +10 speed: a +10 monster drops from
  2 actions per player turn to 1.
- **Risks.** Running out for emergencies; potions cost.
- **Test.** Population, 200 seeds; metric: deaths to uniques; potions used.

### T-ITEM-2 Double resist before dragons

- **Idea.** Always drink the matching temporary resist potion before engaging a dragon, even with the
  item resist.
- **Why.** Timed resist adds a level up to 2 (`src/player-calcs.c`), and base-element damage is
  divided by 3 per level: 1600 → 533 → 177.
- **Risks.** Potion supply.
- **Test.** Population DL30-60; metric: breath-caused deaths.

### T-ITEM-3 *Destruction* as a panic button

- **Idea.** Use *Destruction* when surrounded by a crowd you cannot beat.
- **Why.** `effect_handler_DESTRUCTION()` deletes every monster within radius 15 except at your own
  grid (no unique exception in the loop); it does not work in town or arenas. Deleted monsters are not
  killed (no experience, no drop; uniques can reappear).
- **Risks.** Destroys items on the floor; blinding light (`DESTRUCTION:LIGHT:15`) without blindness
  protection.
- **Test.** Encounter: pit or nest at DL40; metric: survival vs Teleport-out.

### T-ITEM-4 Recharge vs replace the Staff of Teleportation

- **Idea.** Keep one Staff of Teleportation and recharge it rather than buying new ones.
- **Why.** Recharging scrolls are alchemist stock; the staff is magic-shop "normal" stock and may be
  absent. See [Chapter 7](07-consumables-and-devices.md) for recharge failure (it can destroy the staff).
- **Risks.** Losing the only escape device at a bad time.
- **Test.** Population, metric: gold spent on escapes and escape-unavailable situations.

### T-ITEM-5 Throwing-weapon quiver for non-archers

- **Idea.** Priests and mages carry throwing weapons in the quiver as their ranged fallback.
- **Why.** Throwing weapons (`THROWING` flag) can be kept in the quiver, each counting as 5 missiles of
  a slot's 40 (`thrown-quiver-mult:5`), so they do not fill pack slots; flasks of oil are always in
  stock.
- **Risks.** Weight for low-STR casters.
- **Test.** Caster population, arms with/without; metric: deaths with mana empty before CL20.

### T-ITEM-6 Stone-to-Mud escape tunnels

- **Idea.** When cornered by slow or non-tunnelling monsters, dig with Stone to Mud instead of
  teleporting.
- **Why.** Monsters without `KILL_WALL`/`PASS_WALL` must path around; the tunnel opens a new route.
- **Risks.** Tunnels open to new monsters; fast monsters still follow.
- **Test.** Encounter: pack of +0 monsters in a dead end; metric: damage taken before escaping.

---

## 16.10 Shopping and money

### T-SHOP-1 Gold is worthless after the Black Market

- **Idea.** After CL25, ignore gold on the floor unless it is on the way.
- **Why.** With `birth_no_selling` (default) gold drops are increased and selling is disabled
  (`src/list-options.h`); town stock that matters is mostly consumables.
- **Risks.** Missing an expensive Black Market speed item.
- **Test.** Population, metric: turns to DL40, gold at DL40, BM purchases.

### T-SHOP-2 CCW stockpile

- **Idea.** Carry 20+ Cure Critical Wounds from DL15.
- **Why.** CCW heals 60 and cures blindness, cuts, poison, stun, amnesia and confusion (`object.txt`):
  the statuses that block reading and regeneration.
- **Risks.** Weight; gold.
- **Test.** Population, arms 5 vs 20 CCW; metric: deaths with an unreadable escape scroll.

### T-SHOP-3 Store turnover timing

- **Idea.** Time town visits so stores have turned over.
- **Why.** Store stock changes with game time (`store:turns:1000`, `store:shuffle:25`,
  `constants.txt`).
- **Risks.** None beyond complexity.
- **Test.** Count BM items seen per visit as a function of game turns since the last visit.

### T-SHOP-4 Staff of Teleportation before armour

- **Idea.** The first 2000 gold goes to a Staff of Teleportation (magic shop "normal" stock, cost 2000 in
  `object.txt`) rather than armour.
- **Why.** It is the only long-range escape in town stock lists.
- **Risks.** Low AC early.
- **Test.** Population, metric: deaths DL10-25.

---

## 16.11 Exploration

### T-EXPL-1 The game's autoexplore as a baseline

- **Idea.** Use the built-in `autoexplore_commands` option as the exploration step for humans and borgs.
- **Why.** `do_cmd_explore()` paths to the nearest unknown grid and stops when a monster is in view or
  you are confused (`src/cmd-cave.c`, `path_nearest_unknown()`).
- **Risks.** It does not weigh danger or value.
- **Test.** Borg variant using it vs the borg's own flow; metric: turns per level, deaths during
  exploration.

### T-EXPL-2 Map, then stairs

- **Idea.** On each level read Magic Mapping (alchemist stock) or use the staff, go to the nearest '>'.
- **Why.** Minimises moves per level (the guide's risk-per-move argument).
- **Risks.** Less experience and loot; under-levelled characters.
- **Test.** Population, metric: alive at T having reached D, CL at T.

### T-EXPL-3 Light rooms on entry

- **Idea.** Always light a dark room before entering it.
- **Why.** Reveals sleeping monsters and items; some monsters are hurt by light (see
  [Chapter 8](08-monsters.md)).
- **Risks.** Mana/charges; noise is not involved but time is.
- **Test.** Population, metric: deaths in rooms.

### T-EXPL-4 No vaults before CL40

- **Idea.** Treat vaults as walls until CL40.
- **Why.** Vault contents are out of depth by design ([Chapter 9](09-dungeon.md)).
- **Risks.** Missing the best items of the early game.
- **Test.** Population, metric: deaths in vaults, item value found.

---

## 16.12 Uniques

### T-UNIQ-1 Kill Sauron's escort early

- **Idea.** Hunt Khamûl (DL84) and the Witch-King (DL85) before going to DL99.
- **Why.** Sauron is generated with both as servants (`friends:100:1d1:…:servant`, `monster.txt`).
- **Risks.** Each is a strong unique (the Witch-King casts BRAIN_SMASH, BO_MANA, summons at 1-in-2).
- **Test.** Encounter: Sauron fights with and without the escort (debug), 10 trials each.

### T-UNIQ-2 Ignore optional uniques deep

- **Idea.** After DL60, avoid every unique except Sauron's escort and the questors.
- **Why.** Only Sauron and Morgoth are required (`quest.txt`); the C borg's opposite rule (all uniques
  dead before DL98) adds risky fights ([Chapter 12](12-game-plan-by-phase.md#1214-the-c-borgs-readiness-rules-and-where-the-data-disagrees)).
- **Risks.** Missing great drops.
- **Test.** Deep borg suite; arms kill-all vs avoid; metric: survival DL60-99.

### T-UNIQ-3 Teleport Other liberally

- **Idea.** Teleport Other any unique that fails FIGHT-01 instead of fleeing the level.
- **Why.** It removes the threat while keeping the level ([Chapter 13](13-escapes-and-emergencies.md)).
- **Risks.** The unique comes back; the C borg zeroes such monsters' danger, which a rebuild should not
  copy (ANTI-06 in [Chapter 15](15-notes-for-the-borg.md)).
- **Test.** Population DL20-50; metric: deaths to uniques, level changes per unique met.

### T-UNIQ-4 Bullroarer at CL5 with oil

- **Idea.** Kill Bullroarer (DL5, +20, 60 hp) at CL5-6 with 10 oil flasks for his drop.
- **Why.** His melee is 2×1d8 per turn but he has only 60 hp.
- **Risks.** +20 speed means two turns per player turn: 32 damage per player turn.
- **Test.** Encounter: 30 trials at CL5 with 10 flasks; metric: kill rate, deaths.

---

## 16.13 Endgame

### T-END-1 Morgoth in the open

- **Idea.** Fight Morgoth in a large room rather than a corridor.
- **Why.** His `SHATTER` blows cause earthquakes of radius damage/12 when damage after AC exceeds 23
  (`src/mon-blows.c:melee_effect_handler_SHATTER()`), destroying corridor geometry anyway.
- **Risks.** Summons surround you in the open.
- **Test.** Encounter: debug Morgoth, 10 fights each; metric: damage taken per 100 Morgoth turns,
  summon count reaching you.

### T-END-2 Glyph line against Morgoth

- **Idea.** Casters fight Morgoth from behind glyphs.
- **Why.** He breaks a glyph about 18% of the time per attempt (`(100−1)/550`).
- **Risks.** His spells (BA_MANA up to 600) ignore glyphs; `SMASH_WALL` and `MOVE_BODY`.
- **Test.** Encounter: melee damage taken with and without glyphs.

### T-END-3 Keep Sauron low before he shapechanges

- **Idea.** Burst Sauron's hit points down quickly; his shapes breathe from current hit points.
- **Why.** Breath = `hp / divisor`, capped (`breath_dam()`): Wolf-Sauron at 8000 hp breathes fire at the
  1600 cap; at 3000 hp, 1000; at 1500 hp, 500 (before resistance).
- **Risks.** Burst consumes potions and mana early.
- **Test.** Encounter: measure breath damage taken vs Sauron's hp at shapechange.

### T-END-4 Life vs *Healing* allocation

- **Idea.** Save Life potions for drained states only; use `*Healing*` for hit points.
- **Why.** Life heals 5000 and restores experience and all stats; `*Healing*` heals 1200
  (`object.txt`). LOSE_ALL (Morgoth) drains five stats unless sustained.
- **Risks.** Running out of `*Healing*`.
- **Test.** Encounter: Morgoth fights with the two policies; metric: potions used per kill.

### T-END-5 Teleport Level as a reset

- **Idea.** When Morgoth's summons pile up, Teleport Level to DL99 and come back.
- **Why.** Teleport Level up is allowed from DL100 (`effect_handler_TELEPORT_LEVEL()` forbids only going
  down from a quest level); Morgoth regenerates 400 hp per 100 game turns (`regen_monster()`).
- **Risks.** He regenerates; a new DL100 regenerates him on arrival as the questor.
- **Test.** Encounter: fight outcome with and without one reset.

---

## 16.14 Unusual and fun

### T-FUN-1 Pacifist dive

- **Idea.** Avoid every fight that is not blocking the path; measure how deep stealth and escapes alone
  can go.
- **Why.** Experience comes only from kills (`src/mon-util.c`), but depth is gated by kit, not
  experience, in this experiment.
- **Risks.** Low hit points at depth.
- **Test.** Human or borg variant; metric: max depth before death.

### T-FUN-2 Fox-form stair hunting

- **Idea.** Druids walk levels in fox form to find stairs at double movement speed.
- **Why.** `MOVES[1]` halves step energy (`energy_per_move()`).
- **Risks.** Item use requires reverting.
- **Test.** Turns to find '>' per level, fox vs normal.

### T-FUN-3 No-recall ironman

- **Idea.** Play with `birth_no_recall` and connected stairs.
- **Why.** Recall does nothing until you win (`effect_handler_RECALL()`); everything must be found in
  the dungeon.
- **Risks.** Very hard; no town after the first trip.
- **Test.** Human: record max depth and causes of death for 5 characters.

### T-FUN-4 Know everything at birth

- **Idea.** Turn on `birth_know_runes` and `birth_know_flavors`; does an automatic player improve?
- **Why.** Removes identification uncertainty (`src/list-options.h`).
- **Risks.** None for testing.
- **Test.** Borg population with both on vs off; metric: alive at T having reached D; unknown-item
  deaths.

### T-FUN-5 Percent damage mode

- **Idea.** Try `birth_percent_damage` ("To-damage is a percentage of dice (experimental)").
- **Why.** Changes the value of to-dam bonuses vs dice; strategy around weapon choice changes.
- **Risks.** Experimental option.
- **Test.** Population per class; metric: turns to kill per depth band.

### T-FUN-6 Persistent levels

- **Idea.** Play with `birth_levels_persist` (experimental) and use known levels as safe bases.
- **Why.** Levels are kept; recall lets you choose a persistent level
  (`player_get_recall_depth()`); some stair creation is disabled.
- **Risks.** Experimental; monsters persist too.
- **Test.** Human: compare death causes with normal play.

---

## 16.15 Index of ideas

| ID | Topic | One-line idea |
|---|---|---|
| T-BUILD-1 | builds | druid fox form for Free Action at CL1 |
| T-BUILD-2 | builds | Pukel-man as poison immunity DL40-55 |
| T-BUILD-3 | builds | eagle form as an escape walker |
| T-BUILD-4 | builds | saving-throw build for Sauron |
| T-BUILD-5 | builds | blackguard fights instead of resting |
| T-BUILD-6 | builds | necromancer without light |
| T-EARLY-1 | early | oil-flask opener |
| T-EARLY-2 | early | corridor-only fighting to CL10 |
| T-EARLY-3 | early | level-feeling filter below CL10 |
| T-EARLY-4 | early | ranged-only warrior to CL15 |
| T-EARLY-5 | early | Free Action first |
| T-DIVE-1 | diving | Deep Descent from town |
| T-DIVE-2 | diving | Deep Descent only at max depth |
| T-DIVE-3 | diving | kit-gated vs CL-gated depth |
| T-DIVE-4 | diving | deliberate recall-depth reset |
| T-DIVE-5 | diving | object-feeling time budget |
| T-DIVE-6 | diving | force-descend training |
| T-CMB-1 | combat | casters adjacent, not at range 4 |
| T-CMB-2 | combat | long-range breath duels |
| T-CMB-3 | combat | pillar dancing |
| T-CMB-4 | combat | doors vs non-door monsters |
| T-CMB-5 | combat | teach casters your resists |
| T-CMB-6 | combat | taunt casters into melee |
| T-CMB-7 | combat | Stone to Mud on golems |
| T-SPELL-1 | spells | detection on every arrival |
| T-SPELL-2 | spells | Tap Magical Energy battery |
| T-SPELL-3 | spells | glyph fortress |
| T-SPELL-4 | spells | ranger decoys vs breathers |
| T-SPELL-5 | spells | Single Combat isolation |
| T-ITEM-1 | items | speed potion for every unique |
| T-ITEM-2 | items | double resist before dragons |
| T-ITEM-3 | items | *Destruction* panic button |
| T-ITEM-4 | items | recharge the teleport staff |
| T-ITEM-5 | items | throwing quiver for casters |
| T-ITEM-6 | items | Stone-to-Mud escape tunnels |
| T-SHOP-1 | money | ignore gold after CL25 |
| T-SHOP-2 | money | CCW stockpile |
| T-SHOP-3 | money | store turnover timing |
| T-SHOP-4 | money | teleport staff before armour |
| T-EXPL-1 | exploration | built-in autoexplore |
| T-EXPL-2 | exploration | map, then stairs |
| T-EXPL-3 | exploration | light rooms on entry |
| T-EXPL-4 | exploration | no vaults before CL40 |
| T-UNIQ-1 | uniques | kill Sauron's escort early |
| T-UNIQ-2 | uniques | ignore optional uniques deep |
| T-UNIQ-3 | uniques | Teleport Other liberally |
| T-UNIQ-4 | uniques | Bullroarer at CL5 with oil |
| T-END-1 | endgame | Morgoth in the open |
| T-END-2 | endgame | glyph line vs Morgoth |
| T-END-3 | endgame | keep Sauron low before shapechange |
| T-END-4 | endgame | Life vs *Healing* allocation |
| T-END-5 | endgame | Teleport Level reset on DL100 |
| T-FUN-1 | fun | pacifist dive |
| T-FUN-2 | fun | fox-form stair hunting |
| T-FUN-3 | fun | no-recall ironman |
| T-FUN-4 | fun | know runes and flavors at birth |
| T-FUN-5 | fun | percent damage mode |
| T-FUN-6 | fun | persistent levels |

---

## Things to try (untested)

Ideas about the testing process itself:

1. **A shared results log.** Keep every Section 16.3 record in one file (or the tracker's issue board)
   so that the same idea is not tested twice. *Test:* none; it is infrastructure.
2. **Encounter fixtures.** Save a small library of debug characters (CL20/30/40/50 per class) and
   scripted encounters (Bolg, AMHD, Sauron, Morgoth) so that single-mechanic ideas can be compared
   quickly. *Test:* repeatability of the same fixture across runs.
3. **Kill-cam dossiers.** For every borg death record the 20 turns before death (the tracker's death
   dossiers do "T-minus observations from replays"); classify which idea in this chapter would have
   prevented it. *Test:* inter-rater agreement on the classification.
4. **Idea interaction.** Ideas that each help may conflict (fox form vs item use; diving vs vaults).
   Test promising pairs factorially (2×2) rather than one at a time.
5. **Per-class verdicts.** Report every result by class: the same setting helps some classes and hurts
   others on the C borg ([TUNABLE-BORG-PLAN](../c-borg/TUNABLE-BORG-PLAN.md)).
