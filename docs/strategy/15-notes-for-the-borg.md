# 15. Notes for the borg

A machine-oriented distillation of the manual: decision rules and numeric thresholds that a rebuilt
automatic player could implement. Every rule has an ID, a condition, an action, a rationale, a source,
a confidence level and the inputs it needs, classified by whether a human player could know them.
Treat the rules as a starting policy to be measured, not as truths; the constants are tunables.

Contents

- [15.1 Format and conventions](#151-format-and-conventions)
- [15.2 Observability: what a fair borg may know](#152-observability-what-a-fair-borg-may-know)
- [15.3 Core quantities (formulas)](#153-core-quantities-formulas)
- [15.4 Danger estimation rules (DANGER)](#154-danger-estimation-rules-danger)
- [15.5 Fight-or-flee rules (FIGHT)](#155-fight-or-flee-rules-fight)
- [15.6 Healing, cures and rest (HEAL, CURE, REST)](#156-healing-cures-and-rest-heal-cure-rest)
- [15.7 Escape rules (ESC)](#157-escape-rules-esc)
- [15.8 Depth gates and stair policy (DEPTH, STAIR)](#158-depth-gates-and-stair-policy-depth-stair)
- [15.9 Exploration procedure (EXPL)](#159-exploration-procedure-expl)
- [15.10 Item valuation (ITEM)](#1510-item-valuation-item)
- [15.11 Spell and device choice (SPELL)](#1511-spell-and-device-choice-spell)
- [15.12 Town routine (TOWN)](#1512-town-routine-town)
- [15.13 Endgame rules (END)](#1513-endgame-rules-end)
- [15.14 Anti-patterns seen in the C borg (ANTI)](#1514-anti-patterns-seen-in-the-c-borg-anti)
- [15.15 Tunable parameter table](#1515-tunable-parameter-table)
- [15.16 Suggested decision loop](#1516-suggested-decision-loop)
- [Things to try (untested)](#things-to-try-untested)

---

## 15.1 Format and conventions

Each rule is written as:

> **ID — short name.**
> *Condition* → *Action*.
> Rationale. Source. Confidence. Inputs.

- **Confidence**: `fact` (the mechanic is in the source, cited); `heuristic` (judgement built on facts,
  plausible, unmeasured); `untested` (an idea; see [Chapter 16](16-things-to-try.md) for test
  protocols).
- **Inputs** use the observability codes of Section 15.2: `O`, `L`, `S`, `H`.
- Constants written like `K_HEAL_FRAC = 0.5` are tunables, collected in Section 15.15 with their
  defaults.
- "Per player turn" means one action by the player at the player's current speed.
- Source paths are relative to the game source root (`src/`, `lib/gamedata/`); borg sources are
  `src/borg/*.c`; the borg specification is [BORG_SPEC](../c-borg/BORG_SPEC.md) and the tuning plan
  [TUNABLE-BORG-PLAN](../c-borg/TUNABLE-BORG-PLAN.md).

---

## 15.2 Observability: what a fair borg may know

A fair automatic player should decide from what a human could know. Four classes:

| Code | Class | Examples | Fair? |
|---|---|---|---|
| `O` | **Observed state** — on screen or on the character sheet | your hp/sp/max, speed, status effects, AC, known equipment properties, inventory, map you have seen, visible monsters and their names, the tracked monster's health bar (percentage), level feeling (`LF:`), messages | yes |
| `L` | **Lore** — what the in-game monster recall shows *for this character* | a monster's speed and native depth (always in the recall's movement paragraph, `src/mon-lore.c:lore_append_movement()`); blows once seen (`blow_known`/`times_seen`); spells once seen cast; spell frequency after more than 50 casts were observed; hit points ("life rating") and armour after one kill (`armour_known` when `tkills > 0`) | yes |
| `S` | **Spoilers** — public game data (`lib/gamedata/*.txt`, this manual's appendices) | full blow dice, spell lists, spell power, flags and immunities of every race; object allocation depths; store stock lists | arguably fair (a human can read spoilers). Make it a switch: `lore-only` vs `spoiler` mode |
| `H` | **Hidden state** — not visible to any player | a monster's exact current hit points, its energy counter, its individual speed variation, awake/asleep for unseen monsters, unexplored map, unknown runes, RNG state | **not fair**; do not use |

Notes (FACT):

- Non-unique monsters get a random individual speed adjustment of up to ±(energy/10) around their
  race speed (`src/mon-make.c`: `i = turn_energy(race->speed) / 10; mon->mspeed += rand_spread(0, i)`),
  so a +10 race (energy 20) may be +8 to +12. This value is hidden (`H`); use the race speed (`L`) and
  allow a margin.
- Breath damage depends on the breather's *current* hit points (`src/mon-spell.c:breath_dam()`). The
  exact value is hidden; a fair estimate is `health_bar_fraction × life_rating` (`O` × `L`), or the race
  average from spoilers (`S`) before the first kill.
- The C borg reads monster race data directly (`r_info[]` fields such as `freq_spell`, flags and blows
  in `src/borg/borg-danger.c:borg_danger_one_kill()`); that is spoiler-level (`S`) knowledge regardless
  of what the character's lore says.

---

## 15.3 Core quantities (formulas)

All are FACT unless marked.

| Symbol | Meaning | Formula | Source |
|---|---|---|---|
| `E(s)` | energy per game turn at speed `s` | table `extract_energy[]`: 110→10, 115→15, 120→20, 125→25, 130→30, 135→35, 140→38, 145→40, 150→42, 160→45 | `src/game-world.c` |
| `A(m)` | monster actions per player turn | `E(speed_m) / E(speed_p)`; monster haste +10 speed, monster slow −2 per slow level (max 5 levels) | `src/mon-move.c:process_monsters()` |
| `P_cast(m)` | chance a monster casts in a turn | `(100/spell_freq)%`, ×2 at its preferred range (`best_range`), ÷2 if you are taunting; needs a projectable path and range ≤ 20 | `src/mon-attack.c:monster_can_cast()` |
| `best_range` | preferred range | `min_range` (usually 1), +3 for archers, +3 for spellcasters with freq > 24%; breathers with innate freq > 24% and hp > 50% prefer 1 | `src/mon-move.c:get_move_find_range()` |
| `Melee(m)` | max melee per monster turn | sum of blow dice maxima; `HURT`/`SHATTER` reduced by `min(AC,240)/400` | `src/mon-attack.c:adjust_dam_armor()`, `src/mon-blows.c` |
| `Breath(m,e)` | breath damage | `min(hp_m / divisor_e, cap_e)` | `src/mon-spell.c:breath_dam()`, `lib/gamedata/projection.txt` |
| `Falloff(d)` | breath strength at distance `d` | `min(1, D/(d+1))`, `D = 4`, or 6 for `POWERFUL` | `effect_handler_BREATH()`, `src/project.c:project()` |
| `Spell(m,s)` | spell damage | dice in `monster_spell.txt` with `SPELL_POWER` = `spell-power:` or the monster's depth | `lib/gamedata/monster_spell.txt` |
| `Res(e, r)` | resisted damage | base elements: × (1/3)^r, r ≤ 2 with a timed resist; high elements: × 6/(8+1d4) per level (worst case 6/9); immune 0; vulnerable × 4/3 | `src/project-player.c:adjust_dam()`, `src/player-calcs.c` |
| `Save` | saving throw | `skill_sav`%; on a save the *whole* spell is negated (damage too), for spells with a save message (WOUND, BRAIN_SMASH, TELE_LEVEL, FORGET, …) | `src/mon-spell.c` (`randint0(100) < player->state.skills[SKILL_SAVE]`) |
| `XP(m)` | experience for a kill | `mexp × mlevel / plevel` | `src/mon-util.c` |
| `Regen` | player hp regeneration | proportional to max hp; ×2 when resting, ×2 with `REGEN`; **zero** while poisoned, stunned, cut or paralysed | `src/player-util.c:player_regen_hp()` |

Breath divisors and caps (FACT, `projection.txt`):

| Element | Divisor | Cap | Resist factor |
|---|---|---|---|
| acid, elec, fire, cold | 3 | 1600 | 1/3, 1/9 with timed resist |
| poison | 3 | 800 | 1/3, 1/9 |
| light, dark, sound, shards, disenchantment | 6 | 500 | 6/(8+1d4) |
| nexus | 6 | 400 | 6/(8+1d4) |
| nether, chaos | 6 | 600 | 6/(8+1d4) |
| water | 6 | none | — |
| gravity | 3 | 200 | none |
| inertia, force | 6 | 200 | none |
| time | 3 | 150 | none |
| plasma | 6 | 150 | none |
| mana | 3 | 1600 | none |

Useful monster spell maxima (FACT, at spell power `P`):

| Spell | Max damage | P=50 | P=100 | P=130 |
|---|---|---|---|---|
| BA_MANA | 5P + 100 | 350 | 600 | 750 |
| BA_NETH, BA_DARK | 4P + 100 | 300 | 500 | 620 |
| BA_FIRE | 10 + 3.5P | 185 | 360 | 465 |
| BA_ACID | 15 + 3P | 165 | 315 | 405 |
| BA_COLD, BA_ELEC | ~10 + 1.5P | 85 | 160 | 205 |
| BO_MANA | 50 + 2.5P | 175 | 300 | 375 |
| BO_PLAS | P + 66 | 116 | 166 | 196 |
| BO_ICE | P + 36 | 86 | 136 | 166 |
| BO_NETH | 1.5P + 55 | 130 | 205 | 250 |
| WOUND | 5 × (2·⌊P/3⌋) | 160 | 330 | 430 |
| BRAIN_SMASH | 180 (+ slow/confuse/paralyse/blind) | 180 | 180 | 180 |

---

## 15.4 Danger estimation rules (DANGER)

> **DANGER-01 — Per-monster threat per player turn.**
> For every monster `m` that is visible or remembered within 20 grids → compute
> `T(m) = A(m) × max(Tmelee(m), Tranged(m))`, where `Tmelee` = `Melee(m)` if `m` can reach you this
> turn (adjacent, or `A(m) ≥ 2` and one step away, or it passes walls), else 0; and `Tranged` = the
> largest resisted spell/breath damage among its known spells if it has a projectable path, else 0.
> Rationale: the worst single action is what kills; the borg's own model uses the same structure
> (`borg_danger_one_kill()`: physical term vs spell term, scaled by monster moves).
> Source: Section 15.3. Confidence: fact (inputs) + heuristic (worst-case choice). Inputs: `L`/`S`
> (blows, spells, speed), `O` (distance, LOS, your resists and AC).

> **DANGER-02 — Expected vs worst case.**
> Keep two numbers per monster: `T_max` (as DANGER-01) and
> `T_exp = A(m) × [(1−P_cast)·E[melee] + P_cast·E[spell]]`.
> Use `T_max` for life-or-death decisions (FIGHT-02, ESC-01) and `T_exp` for time-to-kill comparisons
> (FIGHT-01). Rationale: the C borg forces full damage everywhere (`borg_danger()` sets
> `full_damage = true` regardless of its argument), which makes it flee fights it would win and gives
> it no basis for comparing resource cost. Confidence: heuristic. Inputs: `L`/`S`, `O`.

> **DANGER-03 — Sum, then cap per source type.**
> Total danger `D = Σ T_max(m)` over monsters that can act on you this turn. Do not cap it at an
> arbitrary 2000 (the C borg's cap, `borg_danger()`); compare it with hit points directly.
> Confidence: heuristic. Inputs: as above.

> **DANGER-04 — Breath uses current hp and distance.**
> `T_breath(m) = Res(e, Breath(hp_est(m), e)) × Falloff(distance)`.
> `hp_est = health_bar × life_rating` once the race is known from a kill, else race average (`S`) or
> a pessimistic default of `2 × depth × 10`. Confidence: fact (formula), heuristic (estimate).
> Inputs: `O` health bar, `L` life rating, `O` distance.

> **DANGER-05 — Status attacks count as lethal when unresisted.**
> If `m` has an unresisted disabling attack — PARALYZE blows or HOLD without Free Action; BRAIN_SMASH
> or CONF/BLIND without protection and with `Save < 50%`; TELE_TO next to a worse monster — then set
> `T_max(m) = max(T_max(m), current_hp)`. Rationale: paralysis is `NONSTACKING` but repeated hits
> chain it; Free Action blocks it entirely (`player_timed.txt`: `fail:1:FREE_ACT`). Confidence: fact
> (mechanics), heuristic (lethality rule). Inputs: `L`/`S`, `O`.

> **DANGER-06 — Invisible and unseen monsters.**
> If a message reports an unseen attacker, or a known invisible race is on the level and you lack See
> Invisible/telepathy → add `K_UNSEEN_DANGER × depth` to `D` until it is located. Confidence:
> heuristic. Inputs: `O` (messages), `L`.

> **DANGER-07 — Summoners multiply.**
> For a monster with summon spells (`S_*`): `T_max += P_cast × A(m) × K_SUMMON × depth`. Rationale: a
> summon adds monsters native to the depth. Confidence: heuristic. Inputs: `L`/`S`.

> **DANGER-08 — Groups and breeders.**
> A pack (`friends:` in the race, or more than 3 of a race visible) → count members that can reach
> you in the next 2 player turns, not just the adjacent ones. Breeders (`MULTIPLY`) → treat the level as
> deteriorating with time: their reproduction rate is gated by `mon-play:mult-rate:8` and
> `mon-gen:repro-max:100` (`constants.txt`). Confidence: heuristic. Inputs: `O`, `L`.

> **DANGER-09 — Sleeping monsters are a deferred threat, not a reduced one.**
> A sleeping monster contributes 0 to *this turn's* danger but its full `T_max` to any plan that will
> make noise near it (fighting, digging, reading unknown scrolls). Rationale: the C borg's code under
> the comment "Reduce danger from sleeping monsters" actually *increases* danger by `sleep + 5` percent
> (`borg_danger_one_kill()`), a contradiction between intent and code. Confidence: heuristic. Inputs:
> `O` (monster list shows asleep/awake for visible monsters).

> **DANGER-10 — No danger discounts for "boredom".**
> Never scale danger down because time on level is high. Rationale: the C borg divides danger by 5
> when `antibounce_count > 1200` or level time > 25000 to escape stalls (`borg_danger_one_kill()`);
> that converts a liveness problem into a death risk. Fix the stall (STAIR-05) instead. Confidence:
> heuristic.

> **DANGER-11 — Use true speed.**
> Never cap your own speed in the danger model. Rationale: the C borg pretends a character with speed
> ≥ 135 has 120-125 (`fake_speed` in `borg_danger_one_kill()`). If the model mis-predicts at high speed,
> correct the model, not the input. Confidence: heuristic.

> **DANGER-12 — Ball spells do not fall off on you.**
> Monster balls are centred on the target; full damage at the centre (`project()`: `dam_temp = dam` at
> distance 0). Only breaths (arcs from the monster) fall off. Confidence: fact.

---

## 15.5 Fight-or-flee rules (FIGHT)

> **FIGHT-01 — Win estimate.**
> Compute `turns_to_kill = hp_est(m) / max(1, my_exp_damage_per_turn(m))` and
> `turns_to_die = (current_hp − K_HP_RESERVE) / max(1, D_exp)`.
> Fight if `turns_to_kill ≤ K_WIN_MARGIN × turns_to_die` **and** FIGHT-02 passes; else consider ESC or
> repositioning (FIGHT-04). `my_exp_damage_per_turn` uses your blows × hit chance × average damage
> (with slays/brands from known runes), or your best spell/device damage after resistances (`L`/`S`
> immunities). See [Chapter 4](04-combat.md). Confidence: heuristic. Inputs: `O`, `L`/`S`.

> **FIGHT-02 — One-turn survival test.**
> If `D_max ≥ current_hp − K_HP_RESERVE` → do not take an action that leaves you in reach; heal (HEAL-01)
> or escape (ESC-01) first. Rationale: you can die between your turns; with `A(m) > 1` a monster can act
> twice. Confidence: fact (mechanics) + heuristic (threshold). Inputs: `O`, `L`/`S`.

> **FIGHT-03 — Prefer corridors against groups.**
> If more than 2 monsters can reach you in open floor and a corridor or doorway is within
> `K_CORRIDOR_STEPS` steps that are safe by FIGHT-02 → move there before fighting. Confidence:
> heuristic (standard play; the official guide recommends it). Inputs: `O`.

> **FIGHT-04 — Fight spellcasters off their preferred range.**
> For a frequent caster (freq > 24%), its cast chance doubles at `best_range` (≈ 4 when not afraid).
> Close to melee range or break line of sight; do not stand at range 4. For a frequent breather with
> hp > 50%, point blank is its preferred range: fight it from range ≥ 7 if you have ranged damage
> (breath halves at 7 for non-powerful breathers). Confidence: fact (mechanics), untested (net effect).
> Inputs: `O`, `L`/`S`.

> **FIGHT-05 — Kill priority.**
> Order targets by `T_max(m) / turns_to_kill(m)` (threat removed per turn spent), highest first.
> Glass cannons (blood falcon: +30 speed, 6 hp, 64 melee) top the list. Confidence: heuristic.

> **FIGHT-06 — Don't fight for nothing.**
> If `XP(m) + expected_drop_value(m)` is below `K_PROFIT_MIN` per turn of fighting and `m` is not
> blocking the path → avoid it. Rationale: the official guide's "What to ignore" list (jellies that
> damage gear, monsters that waste consumables, runaway breeders, uniques with escorts you can't
> dominate). Confidence: heuristic.

> **FIGHT-07 — Uniques: only with a margin.**
> A unique adds `K_UNIQUE_MARGIN` to the win margin: fight only if FIGHT-01 passes with
> `K_WIN_MARGIN × K_UNIQUE_MARGIN`. Uniques have fixed full hit points (no variance) and often escorts
> or summons. Confidence: heuristic.

> **FIGHT-08 — Taunting and decoys change targeting.**
> Taunt (blackguard *Relentless Taunting*) halves monster cast chance and removes their preference for
> range (`monster_can_cast()`, `get_move_find_range()`); a ranger's *Decoy* redirects breaths and
> spells to the decoy (`effect_handler_BREATH()`: `monster_is_decoyed`). Use them against casters and
> breathers respectively. Confidence: fact (mechanics), untested (value).

---

## 15.6 Healing, cures and rest (HEAL, CURE, REST)

> **HEAL-01 — Heal on predicted danger, not on a random draw.**
> If `current_hp − D_max < K_HP_RESERVE` and you will remain exposed → use the smallest heal that
> restores `current_hp > D_max + K_HP_RESERVE`; if no heal achieves it in one action → escape instead
> (ESC-01). Rationale: the C borg mixes deterministic thresholds with random ones (for example
> bleeding cures when `hp < max/3 || randint0(100) < 20`, `borg-caution.c:borg_heal()`). Randomness
> adds variance without information. Confidence: heuristic.

> **HEAL-02 — Heal amounts (FACT).**
> CLW 20, CSW 40, CCW 60, Healing max(300, 35% of missing hp), `*Healing*` 1200, Life 5000 (plus
> experience and stat restore) — `lib/gamedata/object.txt`, `src/effect-handler-general.c:
> effect_handler_HEAL_HP()`. CLW also cures blindness and reduces cuts/confusion; CSW cures blindness,
> cuts and confusion; CCW also cures poison, stun and amnesia. Confidence: fact.

> **HEAL-03 — A potion unused at death is wasted.**
> Never hold back the best heal when HEAL-01 says no smaller heal suffices. The C borg saves
> `*Healing*` and Life "for Morgoth and emergencies" (`borg_heal()` comments) and only uses them after
> other checks fail. Keep an endgame reserve in the *home*, not in an unused pack slot during a fight.
> Confidence: heuristic.

> **HEAL-04 — Healing vs escaping when outdamaged.**
> If `D_exp > heal_amount_per_action` for every heal you have → healing cannot keep up; escape.
> Confidence: fact (arithmetic).

> **CURE-01 — Cure what stops regeneration first.**
> Poison, stun, cuts and paralysis set hp regeneration to zero (`player_regen_hp()`). Before resting
> → cure poison/cut/stun (CCW cures all three). Confidence: fact.

> **CURE-02 — Blindness, confusion, amnesia and darkness block reading.**
> Blind, confused, amnesiac, or without light → you cannot read scrolls (`src/player-util.c:
> player_can_read()`). CLW and CSW cure blindness (CSW also confusion); CCW cures blindness, confusion
> and amnesia (`object.txt`). When afflicted, escape with a potion, staff, rod or activation, or cure
> first. Confidence: fact.

> **CURE-03 — Experience and stat drain are not emergencies.**
> Drains do not kill; restore in town (Restore Life Levels, stat restoration) or with Life. Do not
> spend a fight action on them. Confidence: heuristic.

> **REST-01 — Rest only when safe.**
> Rest when no awake monster has a path to you within `K_REST_SAFE_TURNS` player turns, and hp or sp
> below `K_REST_HP`; rest is interrupted by disturbance (`src/player-util.c`). Resting doubles
> regeneration. Confidence: fact (regen), heuristic (thresholds).

> **REST-02 — Rest on a staircase when possible.**
> If a staircase is within `K_STAIR_REST_DIST` steps → rest on it. Rationale: an instant exit if
> something arrives. Confidence: heuristic.

> **REST-03 — Recall-wait rest.**
> After reading Word of Recall (15-34 world ticks), rest or move to a safe corner rather than explore.
> Confidence: fact (delay), heuristic (policy).

---

## 15.7 Escape rules (ESC)

> **ESC-01 — Escape ladder.**
> When FIGHT-02 fails and HEAL-01 cannot fix it, choose the first that applies:
> 1. standing on a staircase → take it (one action, guaranteed);
> 2. a single dangerous monster in reach and others far → Phase Door if the expected landing (random
>    grid up to 10 away) reduces `D_max` below `current_hp`;
> 3. otherwise → Teleport (scroll, staff, spell; the scroll and staff use `dice:M60`, i.e. a distance of
>    60% of the largest distance from you to a level edge, slightly randomised:
>    `effect_handler_TELEPORT()`; Phase Door is `dice:10`);
> 4. on a level you must leave → Teleport Level (up or down at random unless at a boundary or quest
>    level; `effect_handler_TELEPORT_LEVEL()`);
> 5. no escape and dying → the best heal, then any unknown potion/staff (last resort only).
> Rationale: standard escape order by cost and reliability; C borg `borg_escape()` uses danger bands.
> Source: [Chapter 13](13-escapes-and-emergencies.md). Confidence: heuristic.

> **ESC-02 — Teleport failure modes.**
> Teleportation is blocked by a no-teleport curse (`OF_NO_TELEPORT`) and in single-combat arenas; a
> hostile teleport (TELE_TO, TELE_LEVEL) is resisted with nexus resistance
> (`effect_handler_TELEPORT_LEVEL()`). Check before relying on it. Confidence: fact.

> **ESC-03 — Device failure is a risk term.**
> Escape value = success probability × benefit. Use the displayed failure rate for spells and devices
> (`O`); prefer scrolls (no failure roll, but unusable when blind, confused, amnesiac or without light:
> CURE-02) when the failure rate of the alternative exceeds `K_ESC_FAIL_MAX`. Confidence: heuristic (see
> [Chapter 1](01-core-mechanics.md) for device and spell failure).

> **ESC-03b — After escaping, leave.**
> After a Teleport away from a monster that failed FIGHT-01, mark the level "leave soon" and head for
> stairs or read recall. Rationale: the monster will usually find you again. Confidence: heuristic.

> **ESC-04 — Never rely on recall or Deep Descent as an emergency escape.**
> Delays: recall 15-34 world ticks; Deep Descent 4-7 ticks and lands at **max depth + 5**
> (`effect_handler_DEEP_DESCENT()`). Deep Descent from a shallow level when max depth is high is a
> dangerous "escape". Confidence: fact.

---

## 15.8 Depth gates and stair policy (DEPTH, STAIR)

> **DEPTH-01 — Gate depth on threat, not character level.**
> Allowed depth `d` iff, for the set `M(d)` of monsters native to depths `d−2 … d+5` that you know of
> (`L`) or that spoilers list (`S`): `max_m T_max(m, d) < K_DEPTH_HP_FRAC × max_hp`, with resists as
> currently worn. Rationale: the dominant failure deep is one big hit you do not resist, which this test
> measures directly; the C borg's "CL ≥ depth until CL50" (`borg_prepared()`) is the tuning plan's
> "biggest single lever" and probably too slow. Confidence: untested.

> **DEPTH-02 — Mandatory kit gates (minimum).**
> Do not go below these depths without the item (from [Chapter 12](12-game-plan-by-phase.md); each gate
> is anchored to specific monsters in `monster.txt`):
>
> | Depth | Required | Anchor monsters |
> |---|---|---|
> | 5 | Word of Recall, 3+ cure potions, ranged attack | Bullroarer (D5, +20) |
> | 15 | 100 max hp, Phase ×10 | Wormtongue (D8), Boldor (D13), druid (D13) |
> | 20 | Free Action | illusionist (D13), Old Man Willow (D19), carrion crawler (D25), ghoul (D26) |
> | 25 | See Invisible or ESP; Teleport ×5 | banshee (D24), ghost (D31) |
> | 30 | rFire, rCold | young red/white dragons (D29-31), fire/frost giants (D28-30) |
> | 35 | all four base resists | young MHD (D32), mature dragons (D39) |
> | 40 | rPois *or* temporary rPois potions ×5 | 6/7-headed hydras (D39-40), basilisk (D28) |
> | 45 | rPois permanent, pConf, pBlind | AMHD (D53, 800 poison), drolem (D54) |
> | 55 | +10 speed, Hold Life or rNether | dracolich (D55), time hounds (D51, +20) |
> | 70 | rNether, rChaos, +15 speed, 600 max hp | gelugon (D69), great wyrms (D70+) |
> | 85 | +20 speed, rDisen, 700 max hp | Ancalagon, Tarrasque, greater Balrog (D85-86, +20) |
> | 99 | the Sauron list in END-02 | Sauron |
> | 100 | the Morgoth list in END-03 | Morgoth |
>
> Confidence: heuristic (gates) on fact (monster data). Inputs: `O`, `S`/`L`.

> **DEPTH-03 — Restock gates (carry-to-stay).**
> Leave the dungeon (recall) when below: cures `K_CURE_MIN(phase)`, escapes `K_ESC_MIN(phase)`, light
> fuel, food (see the consumables table in [Chapter 12](12-game-plan-by-phase.md#1213-summary-tables-kit-by-depth)).
> The C borg's `borg_restock()` is a reasonable template (Section 12.14) but should not double as a
> depth gate. Confidence: heuristic.

> **DEPTH-04 — Dive rate.**
> If DEPTH-01 and DEPTH-02 pass for `current + 5` → prefer down stairs over exploring (dive). If they
> pass only for `current + 1` → explore for profit, then descend. If they fail for `current` → go up or
> recall and fix the failing item. Rationale: mirrors the C borg's "power dive" when
> `borg_prepared(depth + 5)` is null, with a better gate. Confidence: heuristic.

> **STAIR-01 — Leave on arrival if the feeling and detection are bad.**
> On arrival: if the monster feeling digit ≤ `K_LF_LEAVE` (1 = "Omens of death" … 9 = "quiet,
> peaceful"; `src/generate.c:calc_mon_feeling()`) and DEPTH-01 is marginal → take the connected
> staircase immediately. Else detect, then decide. Confidence: fact (scale), heuristic (rule).

> **STAIR-02 — Connected stairs give a first escape.**
> With `birth_connect_stairs` (default on), arriving by '>' puts you on '<' and vice versa
> (`do_cmd_go_down()`/`do_cmd_go_up()` set `create_up_stair`/`create_down_stair`). Treat the arrival
> staircase as a known exit until you move away from it. Confidence: fact.

> **STAIR-03 — Stair-scumming is out of scope for a fair, general borg.**
> Repeatedly taking stairs to regenerate levels without playing them (the C borg's munchkin and lunal
> modes) is excluded by the tracker owner's decision; it also produced liveness loops (152 of 200
> Blackguard runs looping on "Fleeing level. Munchkin Mode" without game time passing,
> [TUNABLE-BORG-PLAN](../c-borg/TUNABLE-BORG-PLAN.md)). Use STAIR-01 sparingly, with a budget
> (STAIR-05). Confidence: fact (measurement), policy.

> **STAIR-04 — Quest levels have no down stairs.**
> DL99 and DL100 before their questor dies: no '>' is generated (`alloc_stairs()` returns for
> `FEAT_MORE` on quest levels); Teleport Level only goes up; Deep Descent stops at DL99. Plan the exit
> upward. Confidence: fact.

> **STAIR-05 — Anti-bounce budget.**
> Count consecutive level changes without a kill, an item pickup or a minimum of `K_MIN_LEVEL_TURNS`
> game turns. If the count exceeds `K_BOUNCE_MAX` → stop leaving on feeling alone; play the level with
> extra caution or recall to town. Rationale: the C borg's level-leaving cycles (seed 1045 in issue
> #403, the munchkin loop) are liveness failures. Confidence: heuristic.

---

## 15.9 Exploration procedure (EXPL)

> **EXPL-01 — On arrival.**
> 1. Note the arrival staircase. 2. Read `LF:`. 3. Detect monsters (spell, rod, scroll) if available;
> map/stairs detection next. 4. Apply STAIR-01. 5. Mark the nearest down staircase as the "exit
> target". Confidence: heuristic.

> **EXPL-02 — Exploration order.**
> Explore toward (a) known items worth more than `K_ITEM_DETOUR` per step, (b) unexplored rooms
> nearest by path, (c) the exit. Stop exploring when the expected value of the next step falls below
> `K_EXPLORE_MIN` or DEPTH-04 says dive. The game itself offers `autoexplore_commands` (option,
> `src/list-options.h`): `do_cmd_explore()` walks to the nearest unknown grid
> (`path_nearest_unknown()`) and refuses while a monster is in view or while confused
> (`src/cmd-cave.c`). A fair borg may use the same primitive. Confidence: fact (feature), heuristic
> (policy).

> **EXPL-03 — Light and darkness.**
> Carry light of intensity ≥ 3 once affordable (torch `LIGHT_2` lights adjacent grids only, lantern `LIGHT_3`
> lights radius 2, `object.txt`; intensity computed in `src/player-calcs.c:calc_light()`, grids at distance
> `< intensity` lit in `src/cave-view.c:update_view_one()`); necromancers (`UNLIGHT`) are the exception. Light rooms on entry
> when a room is dark and not known safe. Confidence: fact (radii), heuristic.

> **EXPL-04 — Doors as tools.**
> Close doors behind you against breeders and slow monsters; monsters that cannot open or bash doors
> (`OPEN_DOOR`/`BASH_DOOR` absent in `L`/`S`) are stopped. Confidence: fact (flags), heuristic.

> **EXPL-05 — Avoid vaults unless strong.**
> Enter a vault only if DEPTH-01 passes for `depth + 10`. Vault monsters are out of depth by design.
> Confidence: heuristic.

> **EXPL-06 — Ignore junk automatically.**
> Use the game's ignore settings for item qualities below what you would ever use; time spent picking up
> and dropping is exposure. Confidence: heuristic.

---

## 15.10 Item valuation (ITEM)

> **ITEM-01 — Value = survival first, then offense.**
> `value(item) = W_SURV × ΔSurvival + W_OFF × ΔDamage + W_UTIL × ΔUtility`, where `ΔSurvival` is the
> reduction in `max_m T_max(m)` over `M(current … current+10)` (DANGER-01 with the item worn) and the
> number of DEPTH-02 gates the item satisfies. Rationale: the C borg values the same thing indirectly
> (`borg_power()` adds 40,000 per prepared depth, `borg-power.c`), but its gates are CL- and stat-heavy
> (Section 12.14). Confidence: heuristic.

> **ITEM-02 — Gate items outrank stats.**
> Free Action (before DL20), See Invisible (before DL25), base resists (before DL35), rPois (before
> DL45), speed (always) are worth more than any amount of to-hit/to-dam of similar slot cost.
> Confidence: heuristic based on DEPTH-02.

> **ITEM-03 — Speed is the universal multiplier.**
> +10 speed halves `A(m)` for every normal-speed monster (energy 20 vs 10) and therefore halves every
> `T_max`; no other single property does that. Rank speed above everything except a missing DEPTH-02
> gate. Confidence: fact (energy table), heuristic (ranking).

> **ITEM-04 — Swap items.**
> Keep, in pack or home, items that supply a resist you lack in your main kit; equip on detection of the
> matching breather (DANGER-04). The C borg has `weapon_swap`/`armour_swap` for this. Confidence:
> heuristic.

> **ITEM-05 — Selling is usually off.**
> With `birth_no_selling` (default on) stores pay no gold for items; gold comes from the floor and
> drops instead (`src/list-options.h`: "Increase gold drops but disable selling"). Do not walk to town
> to sell. The C borg's "go up to sell when ≥ 13 sellable items, gold < 25,000, CL < 25" rule
> (`borg-think-dungeon.c`) should be disabled under no-selling (verify how `borg_count_sell()` values
> items under that option). Confidence: fact (option), heuristic.

> **ITEM-06 — Consumable stocks have diminishing returns.**
> Value of the n-th copy of a consumable = `base × r^(n−1)` with `r ≈ 0.8` until the phase maximum in
> [Chapter 12](12-game-plan-by-phase.md#1213-summary-tables-kit-by-depth), then near zero. The C borg's
> inventory scoring has the same shape (`borg_power_inventory()`). Confidence: heuristic.

> **ITEM-07 — Identify by use when safe.**
> Unknown wearables: wear when no monster is in view; runes are learned on use/notice. Unknown scrolls:
> read on a staircase with full hp (official guide advice). Unknown potions: quaff at full hp when
> nothing is near. Confidence: heuristic (from the official guide).

---

## 15.11 Spell and device choice (SPELL)

> **SPELL-01 — Learn order: escape, detect, attack, utility.**
> When choosing spells (classes with `CHOOSE_SPELLS`, `class.txt`), prefer in order: (1) escapes
> (*Phase Door*, *Teleport Self*, *Portal*, *Shadow Shift*, *Teleport Level*), (2) monster detection
> (*Detect Monsters*, *Detect Evil*, *Detect Life*, *Read Minds*, *Reveal Monsters*), (3) the cheapest
> reliable attack, (4) healing/cures, (5) utility. Examples from `class.txt` (level:mana): Mage *Phase
> Door* 2:2, *Teleport Self* 7:6, *Teleport Level* 28:17; Rogue *Phase Door* 7:2, *Teleport Self* 17:6;
> Priest *Portal* 10:4; Necromancer *Shadow Shift* 15:10. See [Chapter 3](03-magic-and-spells.md).
> Confidence: heuristic.

> **SPELL-02 — Damage spell choice by efficiency.**
> For each attack spell: `eff = expected damage after resist/immunity × (1 − fail) / mana`. Cast the
> highest-`eff` spell that kills within `turns_to_die`; if two kill in the same number of turns, take
> the cheaper one. Use beams/balls when ≥ 2 targets are in the area. Confidence: heuristic.

> **SPELL-03 — Fail rate cap.**
> Do not cast an escape with failure > `K_ESC_FAIL_MAX` when a scroll exists; do not cast an attack
> with failure > `K_ATK_FAIL_MAX` in a FIGHT-02-marginal situation. Confidence: heuristic.

> **SPELL-04 — Mana reserve.**
> Keep mana for one escape spell (`K_MANA_RESERVE = mana cost of best escape`) unless a scroll/staff
> escape is in pack. Confidence: heuristic.

> **SPELL-05 — Immunities from lore.**
> Never cast an element a target is known (`L`) or listed (`S`) to be immune to: e.g. Morgoth `IM_ACID
> | IM_COLD | IM_ELEC | IM_FIRE | IM_POIS`; Sauron `IM_ELEC` plus `ainu` base cold, fire, poison
> (`monster.txt`, `monster_base.txt`). Confidence: fact.

---

## 15.12 Town routine (TOWN)

> **TOWN-01 — Shopping order.**
> Alchemist (5) → General (1) → Magic (6) → Black Market (7) → Home (8) → Armoury/Weaponsmith.
> Rationale: consumables first (they are the survival budget), then rare items. Confidence: heuristic.

> **TOWN-02 — Always-stocked items to rely on (FACT, `store.txt`).**
> Alchemist always: Word of Recall, Phase Door, Remove Curse, Cure Light Wounds. General always: Ration
> of Food, Wooden Torch, Flask of Oil, Cloak, Shovel, Pick, shots, arrows, bolts. Everything else is
> "normal" (may be absent). Notably absent from town stock lists: Scrolls of Teleportation, Teleport
> Level, Deep Descent, Potions of Healing+, Free Action, See Invisible (Black Market only, random).
> Confidence: fact.

> **TOWN-03 — Buy list by phase.**
> Fill to the consumables table of [Chapter 12](12-game-plan-by-phase.md#1213-summary-tables-kit-by-depth),
> then spend remaining gold by ITEM-01 value per gold. Keep `K_GOLD_RESERVE` for the next Word of Recall.
> Confidence: heuristic.

> **TOWN-04 — Night town.**
> Town monsters are more numerous at night (`mon-gen:town-day:4`, `town-night:8` in `constants.txt`).
> Shop quickly at night; do not hunt townspeople for gold (the C borg's money-scum option produced 49/50
> characters still at CL1 at the turn cap, [TUNABLE-BORG-PLAN](../c-borg/TUNABLE-BORG-PLAN.md)).
> Confidence: fact (constants, measurement).

> **TOWN-05 — Return.**
> Read Word of Recall in town after shopping; it returns you to your recall depth (normally max depth).
> To reset max depth upward-safe, read recall in the dungeon on a shallower level and answer "yes" to
> "Set recall depth to current depth?" (`effect_handler_RECALL()`). Confidence: fact.

---

## 15.13 Endgame rules (END)

> **END-01 — Kill order.**
> Kill Khamûl (D84) and the Witch-King (D85) before Sauron if feasible: Sauron is generated with both as
> servants (`friends:100:1d1:… :servant`, `monster.txt`). Do not hunt other uniques for the endgame;
> the only required kills are Sauron and Morgoth (`quest.txt`). Confidence: fact (data), heuristic
> (policy).

> **END-02 — Sauron readiness.**
> max_hp ≥ 700; speed ≥ +20 with ≥ 5 Speed potions; rFire, rCold, rAcid, rElec, rPois, rNether, rDark,
> rChaos, rNexus; FA, pConf, pBlind, SI, Hold Life; ≥ 15 `*Healing*`/Healing; Banishment or Mass
> Banishment. Numbers: BA_NETH/BA_DARK max 620 (413 resisted), BO_MANA 375 (unresistable), WOUND 430
> (save negates), BA_FIRE 465; Wolf-Sauron +30 with fire breath up to 1600 at full hp. Confidence: fact
> (numbers), heuristic (list).

> **END-03 — Morgoth readiness.**
> max_hp ≥ 800; speed +20 permanent with ≥ 10 Speed potions; FA, all sustains, pConf, pBlind, rNether,
> SI; 15-30 `*Healing*`/Life; Restore Mana ×15 for casters; summon control. Numbers: BA_MANA max 600
> (unresistable), BA_NETH 500 (333 resisted), BO_MANA 300, BRAIN_SMASH 180; melee 2× SHATTER 20d10
> (AC-reduced) + LOSE_ALL 10d12 (not AC-reduced); speed +30. Confidence: fact (numbers), heuristic.

> **END-04 — Heal thresholds in the endgame.**
> Against Sauron: heal when `hp < 650`. Against Morgoth: heal when `hp < 700` (and earlier if he gets two
> actions in a row: at +20 your speed, he has 1.27 actions per yours). Confidence: heuristic on fact.

> **END-05 — First action is free from spells.**
> Questors are generated `FORCE_SLEEP` → "nice": no monster can cast while nice (`src/mon-make.c`,
> `monster_can_cast()`), and the flag is cleared after your first energy-using action
> (`src/game-world.c:process_player_cleanup()`). Make that first action a buff. Confidence: fact.

> **END-06 — Exit plan.**
> On DL99/100 the only ways out are up (stairs up, Teleport Level, recall). Keep one Teleport Level and
> one Word of Recall at all times. Confidence: fact.

---

## 15.14 Anti-patterns seen in the C borg (ANTI)

These are taken from [BORG_SPEC](../c-borg/BORG_SPEC.md), [TUNABLE-BORG-PLAN](../c-borg/TUNABLE-BORG-PLAN.md)
and the borg source. They are ideas for what a rebuild might do differently, not directives; each
would need measurement before a change is adopted.

| ID | Observed in the C borg | Why it may be wrong | Alternative |
|---|---|---|---|
| ANTI-01 | Depth gate "CL ≥ depth until CL50" (`borg_prepared()`) | slows experience gain; threat is driven by kit; the plan calls depth preparation the biggest lever | DEPTH-01/02 |
| ANTI-02 | Danger capped at 2000 and forced to full damage in every call (`borg_danger()`) | no expected-damage estimate for resource decisions; the cap hides the scale of deep threats | DANGER-02/03 |
| ANTI-03 | "Reduce danger from sleeping monsters" code increases it by `sleep + 5`% | intent and code disagree | DANGER-09 |
| ANTI-04 | Danger ÷ 5 after long level time or bouncing | converts a liveness fix into a risk | DANGER-10, STAIR-05 |
| ANTI-05 | Own speed capped at 120-125 in the danger model when ≥ 135 | systematic misprediction at high speed | DANGER-11 |
| ANTI-06 | Monsters in the Teleport Other list score zero danger (`borg_tp_other_index[]`) | a teleported monster can come back | decay their danger with time instead |
| ANTI-07 | Random healing thresholds (`randint0(100) < 20` etc. in `borg_heal()`) | variance without information | HEAL-01 |
| ANTI-08 | `*Healing*`/Life hoarded for Morgoth | potions wasted on death | HEAL-03 |
| ANTI-09 | Quaff unknown potions / use unknown staffs in an emergency | a last resort that may be worse than an escape | ESC-01 step 5 only |
| ANTI-10 | Munchkin and lunal modes (stair-diving without fighting) | stair-scumming; liveness loop; deaths *rise* for mages and priests under munchkin (31→45%, 24→37%) while falling for warriors (32→7%) | excluded (owner decision); per-class dive policy instead |
| ANTI-11 | Money scum (killing townspeople for gold) | 49/50 characters stay CL1 | TOWN-04 |
| ANTI-12 | "Leave level that feels dangerous" without a budget | cycles between levels without game time | STAIR-05 |
| ANTI-13 | Leave-to-sell rule (≥ 13 sellable items, gold < 25,000, CL < 25) does not itself test `birth_no_selling` | trips to town that give no gold under the default birth option (to verify) | ITEM-05 |
| ANTI-14 | All uniques except Morgoth must be dead before DL98+ | not a game requirement; adds many risky fights | END-01 |
| ANTI-15 | Stat minimums (7, 16, 18/40-18/100) as depth gates | stats are proxies; breath damage ignores them | gate on hp, blows, fail rates |
| ANTI-16 | No rNether/rNexus gates; no hp gate past DL55 | the dominant late threats are nether/mana and 1600-cap breaths | DEPTH-02 rows 55-100 |
| ANTI-17 | Defect shapes listed in BORG_SPEC §11: stale state between calls, mutation during a read-only pass, simulations that do not restore state, double counting, wrong comparisons | engineering, not strategy, but each changes decisions | write pure evaluation functions; test them on fixtures |
| ANTI-18 | `borg_worships_*` switches as separate weight branches | AC, speed, HP, damage, mana gave near-identical outcomes on 50 seeds | one valuation with explicit weights (ITEM-01) |

---

## 15.15 Tunable parameter table

Defaults are starting points (heuristic); measure them with the method of
[Chapter 16](16-things-to-try.md#161-methodology).

| Parameter | Default | Used by | Meaning |
|---|---|---|---|
| `K_HP_RESERVE` | max(20, 10% max hp) | FIGHT-01/02, HEAL-01 | hit points you never plan to spend |
| `K_WIN_MARGIN` | 0.5 | FIGHT-01 | kill in at most half the turns it takes to die |
| `K_UNIQUE_MARGIN` | 0.6 | FIGHT-07 | extra factor for uniques |
| `K_DEPTH_HP_FRAC` | 0.5 | DEPTH-01 | largest single-turn threat of the depth band vs max hp |
| `K_LF_LEAVE` | 2 | STAIR-01 | leave on arrival at feeling ≤ this when marginal |
| `K_BOUNCE_MAX` | 5 | STAIR-05 | consecutive "empty" level changes allowed |
| `K_MIN_LEVEL_TURNS` | 500 game turns | STAIR-05 | minimum time that counts as "played" |
| `K_UNSEEN_DANGER` | 5 | DANGER-06 | danger per depth for an unlocated invisible attacker |
| `K_SUMMON` | 3 | DANGER-07 | summon danger multiplier per depth |
| `K_CORRIDOR_STEPS` | 3 | FIGHT-03 | max steps to reach a corridor |
| `K_PROFIT_MIN` | depth-scaled | FIGHT-06 | minimum value per fight turn |
| `K_REST_SAFE_TURNS` | 5 | REST-01 | no monster may arrive within this many turns |
| `K_REST_HP` | 70% | REST-01 | rest below this fraction of max hp |
| `K_STAIR_REST_DIST` | 10 | REST-02 | walk to stairs to rest if within this many steps |
| `K_ESC_FAIL_MAX` | 5% | ESC-03, SPELL-03 | escape fail cap |
| `K_ATK_FAIL_MAX` | 20% | SPELL-03 | attack fail cap when marginal |
| `K_ITEM_DETOUR` | depth-scaled | EXPL-02 | value per step for a detour |
| `K_EXPLORE_MIN` | depth-scaled | EXPL-02 | stop exploring below this value per step |
| `K_GOLD_RESERVE` | price of 2 Word of Recall | TOWN-03 | gold kept after shopping |
| `W_SURV`, `W_OFF`, `W_UTIL` | 10, 3, 1 | ITEM-01 | valuation weights |

---

## 15.16 Suggested decision loop

A compact ordering of the rules, close to the C borg's ladder (BORG_SPEC §2.8) but driven by explicit
estimates:

```
each turn:
  update observations (O) and lore (L)
  compute T_max, T_exp per monster; D_max, D_exp          # DANGER-01..12
  if D_max >= hp - K_HP_RESERVE:                          # FIGHT-02
      if a heal fixes it in one action: heal              # HEAL-01
      else: escape by ESC-01 ladder
  elif a disabling status is active (blind, confused, poisoned in a fight): cure (CURE-*)
  elif a fight is profitable and winnable: attack best target  # FIGHT-01,05,06
  elif restock gate fails: read recall / head to stairs   # DEPTH-03
  elif dive gate passes for depth+5: go to down stairs    # DEPTH-04
  elif hp/sp low and safe: rest                           # REST-01..03
  else: explore                                           # EXPL-01..06
```

---

## Things to try (untested)

1. **Lore-only vs spoiler mode.** Run the same policy with `L`-only monster knowledge and with `S`
   knowledge. *Why:* measures how much the C borg's direct `r_info` access buys. *Test:* paired seeds,
   death rate and depth at turn T.
2. **Expected-damage healing (HEAL-01) vs the C borg's thresholds.** *Test:* potions used per 10,000
   turns and deaths with unused healing in pack.
3. **Breath falloff positioning (FIGHT-04).** Prefer range ≥ 7 against non-powerful breathers.
   *Test:* breath damage taken per encounter with mature/ancient dragons.
4. **Taunt against casters** (blackguards). *Test:* spells cast per 100 monster turns with and without
   *Relentless Taunting*.
5. **K_DEPTH_HP_FRAC sweep** (0.3, 0.5, 0.7). *Test:* alive at T having reached D.
6. **Speed-first valuation (ITEM-03).** Rank +speed above everything except missing gates. *Test:*
   compare with the C borg's `borg_power()` weights on a fixed item stream.
7. **Anti-bounce budget (STAIR-05)** on the known looping seeds (seed 1045 in #403). *Test:* stall
   count, game turns per level.
8. **Kill-priority rule (FIGHT-05)** vs nearest-first. *Test:* damage taken per kill in pack fights.
9. **Autoexplore primitive.** Use the game's `do_cmd_explore()` as the exploration step. *Test:* turns
   per level explored and deaths during exploration, vs the borg's own flow.
10. **Decoy-assisted dragon fights** (rangers at CL30+). *Test:* breath damage taken with and without a
    decoy placed between you and the dragon.
