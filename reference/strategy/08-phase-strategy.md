# Chapter 8 — The Game Plan: From Birth to Morgoth

> Scope: a practical, phase-by-phase plan for winning Angband 4.2.6 — what
> to aim for, what to carry, what will kill you, and when to go home — plus
> the cross-cutting skills (escape decisions, scumming, ignore/inscription
> setup, the no-selling economy) that make the plan work.
>
> Ground rules for this chapter:
> * Every number is taken from `lib/gamedata/*.txt` or from `src/`, and is
>   cited.
> * A large part of the depth-readiness advice is drawn from the
>   **APWBorg**, the automatic player that ships in `src/borg/`. The borg's
>   rules — especially `borg_prepared_aux()` and `borg_restock()` in
>   `src/borg/borg-prepared.c`, the escape ladder in
>   `src/borg/borg-escape.c:borg_escape()`, the healing ladder in
>   `src/borg/borg-caution.c:borg_heal()`, and the "scary guy" list in
>   `src/borg/borg-flow-kill.c` — are a heavily-tuned, decades-old
>   encoding of "what you need before going to depth X". When a number
>   comes from the borg it is labelled **(borg)**. The borg is a
>   conservative bot that plays with perfect attention and no intuition;
>   treat its gates as a *safe floor*, not a speed record.
> * Dungeon mechanics (profiles, feelings, recall, Deep Descent) are
>   explained in detail in Chapter 7; this chapter only uses them.

---

## 8.1 Two philosophies: diving vs. clearing

Two facts from the code shape everything:

1. **Depth is what makes loot and experience.** Object level is the
   dungeon level (`make_object()` in `src/obj-make.c`; 1 in 20 rolls get a
   further random boost, `great-obj:20`), and monster experience scales
   with monster level. The official guide puts it bluntly: "if you want to
   find better things, you need to go deeper" (`docs/guide.rst`).
2. **Every turn spent on a level is another roll of the dice.** New
   monsters arrive at 1 in 500 per 10 game turns (`mon-gen:chance:500`),
   and each new monster has a 1 in 25 chance to be out of depth
   (`ood-chance:25`). The guide: "messing around long enough at any one
   depth guarantees that something bad will eventually happen".

So the winning style for most characters is **diving**: go down as fast as
your *defensive* checklist allows, take what is easy, skip what is risky,
and leave any level whose feeling or occupants you don't like. "Clearing"
(full exploration of each level) is slower, generates more risk per unit
of reward, and only makes sense in specific cases:

* very early (DL1–5) to get the first few character levels and basic kit;
* on a level with a high object feeling (LF ?-7, 8, 9 or `$`) and a low
  danger feeling;
* when you are deliberately farming something (a pit theme you can
  dominate, a vault you can open safely).

Diving is gated by **defence** (escapes, curing, resists, Free Action, See
Invisible, speed, HP), not by offence. The borg's gate table (8.10) is
essentially a defence checklist with a character-level floor.

---

## 8.2 The no-selling economy

`birth_no_selling` is on by default (`src/list-options.h`). Under it:

* Shops pay **0 gold** for anything (`price_item()` in `src/store.c`), but
  they still take unknown items and identify them for you.
* **Gold found in the dungeon is ×5** (`make_gold()` in `src/obj-make.c`).
  So your income is gold piles, treasure veins and monster gold drops, all
  of which scale with depth.
* Consequence: do not carry loot home to "sell". Carry home only what you
  will **use** or **store**. Everything else: ignore it (`k`) or leave it.
* Spend gold on, in order: escapes and cures; the stat potions and other
  big-ticket items in the Black Market (BM prices are 3× base, Chapter 7.2);
  wands/staves/rods that replace consumables (Staff of Teleportation in the
  Magic shop, Rod of Treasure Location/Illumination/Curing/Detection from
  the BM or dungeon).
* Recharge empty wands/staves for free-ish: "Giving … a useful wand or
  staff with no charges to the Magic User's Shop is a convenient way to
  recharge the item, if you have enough gold to buy it back"
  (`docs/dungeon.rst`).

What the town *always* has (`store.txt` `always:` lines): Word of Recall,
Phase Door, Remove Curse and Cure Light Wounds (Alchemist); food, torches,
oil, shovel, pick, cloak, basic ammo (General Store); the first book of
each realm (Bookseller). Cure Serious/Critical Wounds and Magic Mapping
are only "normal" Alchemist stock (may be out); **Scrolls of Teleportation,
Teleport Level, Deep Descent and Potions of Speed/Healing are never
regular town stock** — you find them in the dungeon or in the Black Market.
The Staff of Teleportation is normal Magic-shop stock.

---

## 8.3 Setting up the interface

### 8.3.1 Options worth reviewing (`=`)

From `src/list-options.h` (defaults in brackets):

| Option | Recommendation | Why |
|---|---|---|
| `birth_connect_stairs` [yes] | keep on | arriving on a staircase back is your #1 escape (Ch. 7.8) |
| `birth_feelings` [yes] | keep on | danger feeling is free information |
| `birth_no_selling` [yes] | keep on | 5× gold; simplifies town trips |
| `birth_force_descend` [no] | off unless you want a challenge | disables up stairs, recall from quest levels |
| `birth_lose_arts` [no] | off | when on, gives `$` feeling but artifacts left behind are gone |
| `disturb_near` [yes] | keep on | interrupts runs/rests when a visible monster moves |
| `hp_changes_color` [yes] | keep on | |
| `auto_more` [no] | keep **off** while learning | with it on, messages like "You feel yourself yanked…" or breath damage can scroll by unseen |
| `autoexplore_commands` [no] | optional | `<`/`>` walk to the nearest known stair; `p` explores (Ch. 7.8) |
| `show_damage` [no] | turn on | see damage dealt |
| `purple_uniques` [no] | turn on | spot uniques at a glance |
| `use_old_target` [no] | your choice | helps keymaps that fire at the current target |

Also set the **hitpoint warning** in the options menu
(`hints.txt`: "Be sure to set your hitpoint warning in the options menu").

### 8.3.2 Information commands (original keyset, `docs/playing.rst`)

* `[` visible monster list, `]` visible object list — use them every time
  you enter a level and after every detection.
* `l` look, `/` identify a symbol, `*` / `'` target.
* `~` knowledge menus: monster recall (what it casts, breathes, how fast,
  how many HP), object knowledge, *and* the place to set automatic
  inscriptions and ignore settings for each item kind.
* `^F` repeat level feeling; the status bar shows `LF:danger-loot`
  (Chapter 7.9).
* `M` full map, `L` locate, `C` character sheet (check resist grid before
  every dive).
* Use subwindows (`=` then `w`) for the monster list, messages and
  inventory — the guide's "Information Awareness" section credits this with
  a large survival improvement.

### 8.3.3 Ignoring ("squelch")

`k` (or `^d`) ignores an item or a whole category; `K` toggles ignoring
(`docs/customize.rst`). Quality ignoring for wearables (`=` → `i`) has
levels **bad / average / good / non-artifact**. Practical schedule:

* From the start: ignore *bad* weapons and armour.
* Once you have decent gear (~DL15–20): ignore *average*.
* Mid-game (DL30+): ignore *good* (keeps egos and artifacts).
* Unwanted flavours (e.g. Potion of Salt Water, low-level scrolls you
  never use) as soon as they are known.
* Ammo you can't use, spellbooks of other realms.

Remember that ignored items are still there; `K` shows them again.

### 8.3.4 Inscriptions

From `docs/customize.rst`:

| Inscription | Effect | Typical use |
|---|---|---|
| `@q1`, `@r1`, `@u1`, `@a1`, `@z1`, `@m1` … | select this item with a digit for that command | `@r1` Phase Door, `@r2` Teleportation, `@r3` Word of Recall, `@q1` CLW/CSW/CCW, `@q2` Healing, `@m1`/`@m2` spellbooks, `@u1` Staff of Teleportation |
| `!*`, `!d`, `!k`, `!v`, `!q`, `!r` | ask before (any / drop / ignore / throw / quaff / read) | `!k!d!v` on artifacts and key consumables; `!r` on Deep Descent and Recall so you don't read them by accident |
| `=g` (`=g4`) | always pick up (up to N) | ammo `=g`, spare spellbooks `=g2` |
| `^>` / `^*` | prompt when doing that action while wearing it | e.g. `^>` on a swap item you must remove first |
| `!!` | notify when recharged | rods, activatable artifacts |
| `@v0` | quiver slot for throwing weapons | throwing weapons, flasks of oil |

Inscribe spellbooks with `@m1`, `@m2` … — the docs explain that if you
lose book 1, `maa` would otherwise cast from whatever book is now first.

Inscriptions can be set to apply automatically per item kind from the `~`
knowledge menu, so set them once per game.

### 8.3.5 Keymaps

Keymaps (`=` → `e`, `docs/customize.rst`) map one key to a sequence. Put
`(` … `)` around the action to suppress `-more-` prompts inside it, and use
`\` to bypass a keymap. Combined with `@` inscriptions, they make escapes
one keystroke:

| Key | Action | Requires |
|---|---|---|
| F1 | `r1` | Phase Door inscribed `@r1` |
| F2 | `r2` | ?Teleportation inscribed `@r2` (or `u1` for a staff `@u1`) |
| F3 | `q1` | best cure potion inscribed `@q1` |
| F4 | `maa'` / `m1a'` | caster's bolt at nearest target |
| F5 | `h` | fire default ammo at nearest |

Save keymaps to a pref file (e.g. `Mage.prf`, `<name>.prf`) or they are
lost at exit.

---

## 8.4 Consumables reference

Values from `lib/gamedata/object.txt` (healing amounts via
`effect_handler_HEAL_HP` in `src/effect-handler-attack.c`: heals
`max(base, m% of missing HP)`). "Town" = sold by a regular store per
`store.txt`.

| Item | Effect | Base cost | Native level | Town? |
|---|---|---|---|---|
| !Cure Light Wounds | heal 20; cure blind; −20 cut, −20 confusion | 20 | 0 | always (Alchemist) |
| !Cure Serious Wounds | heal 40; cure blind, cut, confusion | 70 | 3 | normal stock |
| !Cure Critical Wounds | heal 60; cure blind, cut, **poison, stun, amnesia**, confusion | 200 | 5 | normal stock |
| !Healing | heal max(300, 35% of missing); cures all of the above | 800 | 15 (alloc 30+) | BM / dungeon |
| !\*Healing\* | heal 1200; cures | 1500 | 40 | BM / dungeon |
| !Life | heal 5000, restore all stats and XP, cures | 5000 | 60 | dungeon |
| !Speed | +10 speed for 20+2d10 turns | 75 | 1 | BM / dungeon |
| !Heroism | heal 10, cure fear, hero 25+1d25 | 25 | 1 | — |
| !Berserk Strength | heal 30, cure fear, berserk 25+1d25 | 80 | 3 | normal stock |
| !Resist Poison | temp rPois 50+1d10, cure poison | 30 | 20 | — |
| !Resist Heat / Cold | temp resist | 30 | 30 | normal stock |
| !Restore Mana | restore SP | 350 | 25 | — |
| ?Phase Door | short teleport (range 10) | 15 | 1 | always |
| ?Teleportation | long teleport | 40 | 10 | BM / dungeon |
| ?Teleport Level | up or down one level, instant | 800 | 20 | BM / dungeon |
| ?Word of Recall | town ↔ max depth after 15–34 turns | 125 | 5 | always |
| ?Deep Descent | max depth + 5 after 4–7 turns | 250 | 10 | BM / dungeon |
| ?Magic Mapping | map local area | 65 | 5 | normal stock |
| ?Remove Hunger | food to 50% | 10 | 5 | normal stock |
| ?Banishment / ?Mass Banishment | remove a monster race / everything nearby | 2500 / 3000 | 40 / 50 | dungeon |
| ?Rune of Protection | glyph of warding | 500 | 60 | dungeon |
| Staff of Teleportation | long teleport; **usable blind or confused** | 2000 | 20 | normal stock (Magic shop) |
| Staff of Detect Evil / Mapping | detection | 350 / 600 | 20 | normal stock |
| Wand of Stone to Mud | dig | 300 | 10 | normal stock |
| Rod of Treasure Location / Illumination / Curing / Detection / Recall / Healing | reusable versions | 1000 / 1000 / 1000 / 3000 / 4000 / 20000 | 10–80 | BM / dungeon |

Mushrooms worth knowing (`object.txt`): **Terror** (TERROR: you are afraid,
but **+10 speed** that stacks with !Speed — `calc_bonuses()` in
`src/player-calcs.c`), **Emergency** (heal 200, temp rFire/rCold, but
hallucinate), **Clear Mind** (cure confusion/fear, temp rConf, some mana),
**Fast Recovery** (heal 30, cure stun/cut/poison/blind), **Stoneskin**
(+40 AC, −5 speed; General Store), **Sprinting** (+10 speed for 100
turns, then you are slowed). Note that !Speed and Sprinting do *not*
stack with each other (same `TMD_FAST || TMD_SPRINT` check).

Remember the elemental hazards to inventory (`hints.txt`): fire burns
scrolls, books and staves; cold shatters potions; electricity destroys
wands, rods and rings; acid damages armour. Carry spares of anything vital.

---

## 8.5 The escape ladder

This is the decision procedure to run every turn something dangerous is in
view. It is distilled from the borg's `borg_escape()` and `borg_heal()`
(`src/borg/borg-escape.c`, `src/borg/borg-caution.c`) and from the game
mechanics in Chapter 7.

### 8.5.1 Estimate "danger"

The borg's core input is **danger**: the damage the visible monsters could
do to you before your next move (`borg_danger()`), compared with your
**current HP** (`borg.avoidance` is set to current HP in normal play,
`src/borg/borg-update.c`). You should do the same rough sum: the biggest
breath or spell each monster in line of sight could use, plus melee from
adjacent ones. Use monster recall (`/`, `l`, or `~`) to know what they do.

### 8.5.2 The borg's thresholds (normal play, no `plays_risky`)

From `borg_escape()` (risky borgs add 0.3 to every multiplier):

| Tier | Trigger (danger vs. current HP) | Action order |
|---|---|---|
| 1 — "about to die" | > **1.5×** HP with no unique near; > 1.7× with a unique near (above DL95); always if > 4.5×; or heavily stunned | stairs if standing on them; teleport (spell, ?Teleportation, staff, activation); **Teleport Level / Deep Descent**; then a desperation Phase Door |
| 2 — "a bit more scary" | > **1.1×** HP (no unique) or > 1.3× (unique near) | stairs; long teleport; else Phase Door if the landing area looks safer |
| 3 — "not too bad" | > **1.0×** HP (no unique); > 1.3× with 2+ uniques; afraid warrior with no missiles | stairs, or Phase Door |
| 4 — "compromised" | > **0.8×** HP when clvl < 35 or HP ≤ ⅓; > **0.6×** when clvl ≤ 35 | Phase Door |

Plus three level-leaving rules from `borg_caution()`:
* more than **3 escapes on one level** (outside unique fights): leave the
  level;
* danger > **2×** HP: flee the level;
* a "scary guy" (8.5.5) anywhere on the level: leave.

### 8.5.3 The borg's healing ladder

From `borg_heal()` — when danger is high enough that healing matters:

| HP lost | Use | Condition |
|---|---|---|
| 30–40% (any % below clvl 10) | CLW (or the Minor Healing spell) | heal would cover the danger and is > ⅓ of it ("no rope-a-doping") |
| 40–50% (any below clvl 20) | CSW | same |
| 50–55% | CCW | same |
| ≥ 55% | Healing-tier (rod/staff/spell/activation) | danger < HP + heal |
| HP < 10% max, or danger > HP and ≤ 2 escapes and HP < 25% | reserve \*Healing\*/Healing/Life | "Using reserve EZ_Heal" |

And the two "this is how you die" rules:
* **Confused**: cure it first (CCW/CSW/Clear Mind, Curing staff) if the
  danger allows; if you have a Staff of Teleportation and danger is extreme,
  use the staff (staffs work while confused and blind — `hints.txt`).
* **Blind**: if danger > 2.5× HP, teleport with the staff first and fix
  blindness later; otherwise cure (CLW/CSW/CCW/Fast Recovery).
* Out of combat: the borg flees the level when HP < ⅓ max (or < ½ and <
  3×clvl) and it holds fewer than 3 CCW and no !Healing; and when bleeding
  or poisoned below ½ HP.

### 8.5.4 The ladder in plain words

1. **Am I standing on stairs?** Take them. (With connected stairs you
   arrived on one — that is why you should fight near your arrival point
   whenever possible.)
2. **Could the visible monsters kill me before my next turn (danger ≥ my
   HP)?** Escape *now*, long-range: Teleport (spell/scroll/staff). If
   teleport is impossible or you are about to die anyway: **Teleport
   Level** (instant). Deep Descent takes 4–7 turns — it is *not* an
   instant escape (Chapter 7.10.2); read it only when you can survive that
   long and want to go max + 5.
3. **Is it close (danger 60–100% of my HP)?** Phase Door to break line of
   sight or melee contact, then re-assess. In open rooms Phase Door may
   leave you in the line of fire (`hints.txt`).
4. **Am I hurt but not in immediate danger?** Quaff the cheapest cure that
   heals more than one round of incoming damage: CLW/CSW/CCW/Healing by
   the table above. Do not "rope-a-dope" (heal for less than you take).
5. **Am I blind/confused/paralysable?** Fix status first: CCW cures blind,
   confusion, poison, stun; a Staff of Teleportation works blind and
   confused.
6. **Has this level already cost me 3 escapes, or is there something on it
   I can't beat?** Leave the level: stairs, or read Recall/Teleport Level.
7. **Word of Recall is a planned exit, not an escape.** Read it *early*
   (15–34 turns) and spend the delay somewhere safe.
8. **After teleporting away from a tough monster, beware of teleporting
   again — you may land next to it** (`hints.txt`).

### 8.5.5 The borg's "scary guy" list — leave the level

From `borg-flow-kill.c` (the borg leaves any level containing these):

| clvl condition | Monsters |
|---|---|
| ≤ 5 | squint-eyed rogue; Grip, Fang; small kobold |
| ≤ 6 (mages/priests) | squint-eyed rogue |
| ≤ 8 | soldier, cutpurse, acolyte, apprentice, kobold, shrieker, Farmer Maggot, street urchin, battle-scarred veteran, mean-looking mercenary |
| ≤ 15 | Bullroarer; white mice / white & green worm masses once their count ≥ clvl |
| ≤ 20 | cave spider, red naga, giant red frog, radiation eye; yellow worm masses ≥ clvl |
| < 45 | **gravity hound, inertia hound, any ancient dragon, Beorn, dread** |
| no nether resist | **Ossë, dracolich, dracolisk** |
| no blindness resist | light hounds (without rLight), dark hounds (without rDark) |
| no rChaos and no rConf | **anything that breathes chaos** |
| no rConf | Pukel-man, night mare |
| no basic poison resist | **drolem** |

---

## 8.6 Pre-dive checklist

Run this in town before every trip (and on the stairs before descending
past a new gate). Numbers are the borg's minimums for the target depth
(`borg_prepared_aux()`); 8.10 has the full table.

1. **Light**: radius ≥ 2 from DL10 (necromancers excepted); spare fuel or
   torches unless the light is permanent. Wooden torches are `LIGHT_2` and
   burn out; lanterns are `LIGHT_3` and take oil (`object.txt`).
2. **Food**: ≥ 5 rations (borg), or ?Remove Hunger / a hunger spell.
3. **Recall**: ≥ 1 ?Word of Recall from DL5 (borg). Carry 2–3 (borg
   rewards up to 3, up to 7 with STR ≥ 15, `borg-power.c`); a Rod of
   Recall from DL50 (borg rewards it there).
4. **Phase Door**: ≥ 1 is essential from DL6 (borg restock); the borg
   rewards carrying up to 8 (15 with STR ≥ 15).
5. **Teleport**: ≥ 2 (scrolls + staff charges + spell) from DL10, ≥ 4 to
   stay in DL20–35, ≥ 6 to *descend* to DL26+; the borg rewards up to 10
   scrolls at clvl 30+.
6. **Teleport Level**: the borg wants ≥ 2 of (?Teleportation + ?Teleport
   Level) to stay at DL36–45 and rewards up to 5 ?Teleport Level from clvl
   15.
7. **Cures**: ≥ 2 CLW/CSW/CCW from DL3; ≥ 3 CCW from DL10; ≥ 10 CSW+CCW
   for DL26+ (all only while clvl < 30); the borg rewards up to 10 CCW
   (15 with STR > 15).
8. **Healing**: ≥ 1 !Healing (or \*Healing\*) from DL46; ≥ 2 from DL56;
   restock trigger "1 heal" from DL65.
9. **Speed**: +5 from DL46, +10 from DL61, +20 from DL81 (borg). Carry
   !Speed (borg rewards up to 20).
10. **Resists & abilities**: See Invisible or telepathy (DL10), Free Action
    (DL20), rFire + 2 basic (DL21), all 4 basic (DL26), rPois + rConf
    (DL40), Hold Life (DL46, unless clvl 50), rBlind + ESP (DL56), rChaos +
    rDisen (DL61).
11. **Character level**: ≥ the target depth (borg, until clvl 50), plus
    class-specific margins (8.10).
12. **Inventory sanity**: spare spellbook(s) for casters, inscriptions set,
    no unknown cursed item on, nothing important unprotected from fire /
    cold / electricity damage.

---

## 8.7 The phases

Each phase lists: goals; clvl/HP/speed targets; mandatory and desirable
defences; a consumables kit; specific threats (from `monster.txt`, see
Chapter 7.14); how to farm or dive; when to go home. "Borg gate" means the
condition in `borg_prepared_aux()` the borg needs before *descending* into
that range; "borg restock" means the condition in `borg_restock()` that
sends it home if it falls below.

### Phase 0 — Birth and the first town visit

**Start**: 600 gold (`player:start-gold:600`) minus the start kit
(`birth_start_kit`). Start kits (`class.txt`):

| Class | Kit |
|---|---|
| Warrior | Dagger, Soft Leather Armour, !Berserk Strength, 1–3 rations, 1–3 torches, ?Recall |
| Mage | Rapier, [Magic book 1], rations, torches, ?Recall |
| Druid | Whip, [Nature book 1], rations, torches, ?Recall |
| Priest | Mace, [Prayer book 1], **2 !CSW**, rations, torches, ?Recall |
| Necromancer | Main Gauche, [Shadow book 1], rations, ?Recall (**no torch** — necromancers use unlight) |
| Paladin | Main Gauche, [Prayer book 1], ?Protection from Evil, rations, torches, ?Recall |
| Rogue | Dagger, Soft Leather Armour, [Magic book 1], rations, torches, ?Recall |
| Ranger | Main Gauche, Short Bow, 15–20 arrows, [Nature book 1], rations, torches, ?Recall |
| Blackguard | Tulwar, Leather Shield, [Shadow book 1], !CLW, rations, torches, ?Recall |

**Shopping list** (from `docs/guide.rst` "Plan to be deep", adjusted for
the no-selling economy):

* 3+ ?Phase Door (15 gold each).
* 1–2 !CLW (20 each); a !CSW if affordable.
* ~5 Flasks of Oil to throw (`v`) at dangerous early uniques — the guide's
  recommended "ranged attack to soften up a (single) unique".
* Iron Shots or arrows if you have a launcher (a Sling is cheap at the
  Weaponsmith).
* Melee classes: the lightest weapon that maximises blows (guide: Rapier,
  Main Gauche or Dagger).
* Don't buy armour yet ("very expensive in comparison to AC").

### Phase 1 — Town & DL1–5

| | |
|---|---|
| Goals | clvl 5–10; a launcher/ammo or throwing kit; first ego/magic items; learn your class's first spells |
| clvl | borg: DL3–4 needs warrior/paladin/ranger/blackguard clvl 4, rogue 8, priest/druid 9, mage/necro 11; DL5–9 needs 6 / 10 / 15 / 15 (warrior-paladin-ranger / rogue / priest-druid / mage-necro) |
| HP | borg: 30 HP for DL2; 50 (warrior/rogue/paladin/ranger), 40 (priest/druid), 60 (mage/necro) for DL3–4; 60/60/60/80 for DL5–9 |
| Speed | none needed |
| Must have | light; food; 2 cure potions for DL3+; ?Recall for DL5+ |
| Kit | 3–5 ?Phase, 2+ CLW, 1 ?Recall, oil flasks, food 5, spare torches/oil |

Threats: Grip and Fang (DL2, +10 speed), cave spider swarms (DL2, +10),
Bullroarer (DL5, +10), Sméagol (DL3, +20, steals), lice/worm breeders,
townspeople at clvl 1–5 (the borg leaves town if a squint-eyed rogue,
mercenary or veteran threatens a low-level character). Trap doors from
DL2.

How to play it:
* DL1–2 can be cleared. From DL3 start skipping anything that takes
  consumables to kill.
* Fight in corridors. Fast packs (dogs, jackals, cave spiders) are deadly
  in open rooms at clvl 1–5.
* Close doors on breeders and leave the level if they get out of hand
  (borg: flee breeder levels below clvl 15).
* Identify by use; read unknown scrolls standing on stairs (guide).

When to go home: out of Phase Door or cure potions; inventory full of
things you will actually use; or after ~DL5 to restock and buy a better
weapon/launcher with the (×5) gold.

### Phase 2 — Early game (DL5–15)

| | |
|---|---|
| Goals | clvl 15–25; Free Action and See Invisible sourced; first speed item if lucky; a reliable ranged attack |
| clvl | borg: never below clvl = depth; DL10–19: warriors/blackguards clvl ≥ depth − 4, rogues/priests/druids/paladins/rangers clvl ≥ depth, mages/necros clvl ≥ depth + 5 (until clvl 28) |
| HP | no explicit borg HP gate here; be able to survive a bolt from the deepest caster you meet |
| Speed | none needed; anything helps |
| Must have (borg gate DL10) | light radius 2; **2 teleports** (scrolls + staff charges, or a teleport spell); **3 CCW** (clvl < 30); **See Invisible or telepathy** |
| Borg restock DL6–9 | 1 ?Phase, 1 cure; **DL10–19**: light 2, 2 cures, 2 teleports |
| Kit | 5–8 ?Phase, 3–5 CCW + some CSW/CLW, 2–3 ?Teleportation or a Staff of Teleportation, 2 ?Recall, ?Magic Mapping or a mapping staff, food |

Threats: Wormtongue (DL8), the orc uniques (Grishnákh, Golfimbul, Ufthak,
Lagduf, Mughash, Brodda), wolf/warg packs (DL10/14, +10), orc archers
(DL13), lurkers (DL14), moaning spirits (DL12, invisible), disenchanter
eyes and molds, **Zephyr hounds** from DL15. **Moria levels** (DL10–39,
2.5%) are packed with orcs/trolls/ogres/giants. **Labyrinths** begin at
DL13.

How to play it:
* This is where "dive" starts paying off. Take stairs as soon as a level
  shows LF danger ≥ 5–6 *for your clvl*, or nothing useful.
* Kill uniques that drop well and you can handle (guide: Bullroarer,
  Brodda, Wormtongue); throw oil at them.
* Pick up every potion and scroll: early unknown flavours are cheap
  identification and many are escapes.
* Hunt for: Free Action (needed at DL20), See Invisible, a light radius 2
  source, a launcher, rFire/rCold items.

When to go home: below 2 teleports or 2 cures (borg restock); ~every
10–15 levels of descent to spend gold.

### Phase 3 — Early-mid game (DL15–30)

| | |
|---|---|
| Goals | clvl 25–35; basic resists; Free Action; first +speed; bigger HP pool; first Teleport Level scrolls |
| clvl | borg: clvl ≥ depth; for DL21+: warriors/blackguards clvl ≥ depth + 5 (until 38), rogues depth + 10 (until 43), priests/druids depth + 13 (until 46), paladins depth + 7 (until 40), rangers depth + 8 (for clvl 29–41), mages/necros depth + 8 (until 38) |
| Speed | none required by the borg yet; any + speed greatly improves survival |
| Must have | **Free Action by DL20** (borg gate); **rFire + 2 of rCold/rElec/rAcid by DL21**; STR, DEX, CON (and spell stat) ≥ 7; **all four basic resists by DL26** |
| Borg restock DL20–35 | 4 CSW+CCW, 4 teleport (scroll/staff) |
| Borg gate DL26 | 6 teleport, 10 CSW+CCW (while clvl < 30) |
| Kit | 8+ ?Phase, 10 CCW/CSW, 4–6 ?Teleportation (or staff + scrolls), 1–2 ?Teleport Level, 2–3 ?Recall, !Resist Poison, !Speed if found, Magic Mapping / Detect Evil staff |

Threats (Chapter 7.14): Zephyr hounds (light/dark hounds **blind**;
fire/cold/energy/air/earth/water), **carrion crawlers (DL25) and ghouls
(DL26) — paralysis**, imps (DL17, invisible), Old Man Willow (DL19),
Bolg (DL20, +10), Azog (DL23, +10), wererats/werewolves, vortices (DL21),
wyverns, young dragons (DL29–32), stone trolls, Mîm (DL27, +10),
Draebor (DL28, +10, summons), Beorn (DL28), nexus and vibration hounds
(DL27), mind flayers (DL28), the Stone Troll trio (DL30, +10, 1100 HP
each). From DL20: **lairs and gauntlets** (0.5% each); never use teleport
on a gauntlet's arrival side.

How to play it:
* The first great killer of characters is **paralysis without Free
  Action**. Do not go below DL20 without it — this is the borg's hardest
  early rule.
* Light/dark hounds come in packs and blind you; carry CLW/CSW/CCW (all
  cure blindness) and keep a Staff of Teleportation (usable blind).
* From DL26 rooms are **never lit** (`depth <= randint1(25)`); detection
  and a mapping source become standard equipment.
* Ranged attackers (orc archers, Easterling champions, spellcasters) start
  to matter; fight around corners.

When to go home: below the restock line (4 cures, 4 teleports); when an
important resist item is found and you want to rebuild your equipment
set; when your home holds stat potions to buy from the BM.

### Phase 4 — Mid game (DL30–45): "the 30s wall"

| | |
|---|---|
| Goals | clvl 35–40+; all basic resists; rPois and rConf by DL40; +speed; Teleport Level supply; first Healing potions |
| clvl | borg: **clvl 40 required for DL34+** (`"level 40"` gate), plus the class margins above |
| Must have | DL33-: all basic resists, 6 teleports, 10 CSW+CCW (clvl < 30); **DL40–45: resist poison, resist confusion, STR/DEX/CON (and spell stat) ≥ 16** |
| Borg restock DL36–45 | 2 of (?Teleportation + ?Teleport Level) |
| Kit | 10 CCW, 1–3 !Healing, 6–10 ?Teleportation (or staff + scrolls), 3–5 ?Teleport Level, 2–3 ?Recall, 5–10 !Speed, !Resist Poison spares, detection (rod/staff/spell) |

Threats: **gravity, impact and inertia hounds (DL35)** — the borg leaves
any level with gravity or inertia hounds until clvl 45; young/mature
dragons (DL29–43), basilisks, liches (DL34), master liches (DL41),
**white wraiths** (DL38), time vortices (DL38, +20), abyss spiders,
**beholders** (DL40), xaren, troll chieftains, **hellhound packs** (DL42,
+10), **multi-hued hounds** (DL43), **dread packs** (DL44, +10, pass walls
— borg-scary until clvl 45), greater basilisk (DL45), the elemental
uniques (DL43), The Queen Ant (DL42), Kavlax (DL41).

How to play it:
* Every monster type now has a "kill you in one turn if unprepared" form.
  Check `/` recall on anything new before engaging (`hints.txt`: "Never
  underestimate monsters you haven't fought before").
* Stop relying on Phase Door against packs of hounds; teleport, or fight in
  a corridor where only one can reach you.
* Confusion is the silent killer here: without rConf you can't read
  scrolls while confused, so the Staff of Teleportation (and CCW) are your
  only reliable outs.
* This is where speed starts to separate living characters from dead ones,
  even though the borg only *requires* +5 from DL46.

When to go home: below 2 teleport/teleport-level scrolls (borg restock), or
low on CCW/Healing.

### Phase 5 — Late game (DL45–70)

| | |
|---|---|
| Goals | +10 speed by DL60; Hold Life; rBlind; telepathy; rNether/rChaos/rDisen; 500+ HP; stockpile !Healing |
| clvl | 40–50 (the borg's clvl ≥ depth rule stops applying at clvl 50) |
| HP | **≥ 500 HP for DL46+** (borg) |
| Speed | **+5 for DL46–60**, **+10 for DL61+** (borg) |
| Must have | DL46: +5 speed, 1 !Healing (or \*Healing\*), 500 HP, Hold Life (unless clvl 50), STR ≥ 18/40, DEX and CON ≥ 18/60, spell stat 18/100; DL56: 2 heal, **rBlind**, **ESP**; DL61: +10 speed, **rChaos**, **rDisen** |
| Borg power rewards | rNether from max depth 49 (and again at 59), rSound from 49 (`borg.txt` power formula) |
| Kit | 10–15 CCW, 3–5 !Healing, 1–2 \*Healing\*, 10 ?Teleportation (or staff), 3–5 ?Teleport Level, 2 ?Recall or Rod of Recall, 10–20 !Speed, !Restore Mana (casters), ?Deep Descent for fast diving |

Threats: ancient dragons (DL50–53, +10; borg-scary below clvl 45),
**time hounds** (DL51, +20) and nether/plasma hounds (DL51),
**drolem** (DL54, poison breath), **dracolich / dracolisk** (DL55, nether /
nexus), dreadmasters, demiliches, 9-headed hydras, The Balrog of Moria
(DL50, +20), Harowen (DL51, +30), Shelob, Ar-Pharazôn, Baphomet, Fundin,
Saruman (DL60), Smaug (DL62), The Mouth of Sauron (DL65), chaos hounds
(DL64), dreadlords (DL62), greater vaults (1.8–3.8% per level here,
0.5% hard-centre levels from DL50).

How to play it:
* With +10 speed and the core resists you can start using **Deep
  Descent** as a diving tool: it takes you to max depth + 5 in 4–7 turns.
  Never read it as a panic button when you are shallower than your max.
* !Healing is now your real HP bar. The borg's warrior/rogue/blackguard
  power function rewards carrying up to 15 !Healing; casters up to 10;
  plus 1–2 \*Healing\* in reserve from max depth 46 (`borg-power.c`).
* Stat drain and experience drain (time, nether, Black Breath) are common:
  Hold Life and sustains matter; !Restore Life Levels is town stock.
* Greater vaults begin to appear. Only open one with detection, an escape
  that works inside it (teleport always takes you *outside* a vault) and
  a plan for its `8`/`0` monsters (Chapter 7.5.5).

When to go home: out of heals (borg restock from DL65: 1 heal in some
form); after collecting a stat-gain or speed item; to check the BM for
stat potions and !Healing.

### Phase 6 — Deep game (DL70–98)

| | |
|---|---|
| Goals | +20 speed by DL81; everything resisted; enough \*Healing\* for the endgame |
| Speed | **+20 for DL81+** (borg); the borg's power function also rewards +30 for warriors from max depth 79 |
| Must have | all of Phase 5, plus +20 speed for DL81+ |
| Borg extra | from DL82 the borg refuses to go deeper until it holds **10 \*Healing\*/Life** (counting home); once it has seen DL97+ it scums DL80–81 for **30** before continuing |
| Kit | 15 CCW, 5+ !Healing, 5–10 \*Healing\*, 10+ ?Teleportation, 5 ?Teleport Level, Rod of Recall, 20 !Speed, !Restore Mana stack (casters), ?Banishment / ?Mass Banishment |

Threats: great wyrms (DL73–82), hounds of Tindalos (DL69, +20), aether
hounds (DL74, +20), greater quylthulgs (DL71, summon greater demons /
dragons / undead), pit fiends (DL77, +20), greater Balrogs (DL85, +20),
the Nazgûl (DL77–85), Ungoliant (DL75, 13 000 HP), Glaurung (DL75),
Ancalagon (DL85, 10 000 HP), The Witch-King (DL85), Tarrasque (DL86),
Carcharoth and Huan (DL90), Vecna (DL92), Gothmog (DL95, +20).
Greater vaults now appear on 10–18% of levels.

How to play it:
* Summoners are the main killers. Kill them first, at range, or leave.
  ?Banishment / ?Mass Banishment remove whole races or everything nearby.
* Teleport-to and teleport-level from monsters: Nexus resistance
  blocks monster-cast teleport level (`effect_handler_TELEPORT_LEVEL`).
* Glyphs of warding (?Rune of Protection, priest spell) hold most monsters:
  breaking chance per attempt is `monster level / 550`
  (`mon-play:break-glyph:550`, `src/mon-move.c`).
* Farm \*Healing\* and !Speed, the endgame currency: the borg literally
  stops diving at DL82 to stockpile them.

When to go home: whenever the potion stockpile has grown enough to store
(the borg carries 1–2 \*Healing\* and stores the rest at home until it
prepares for the big fights).

### Phase 7 — Endgame (Sauron DL99, Morgoth DL100)

Facts (Chapter 7.12): Sauron (DL99, 8000 HP, +20) and his three shapes
(+20/+20/+30); Morgoth (DL100, 20 000 HP, +30, AC 180, never sleeps,
SHATTER blows 20d10). DL100 is unreachable while Sauron lives; Deep
Descent and Teleport Level stop at DL99; quest levels have no down stairs
until the quest monster dies; winning requires both kills.

The borg's preparation (`borg-prepared.c`, `borg-power.c`,
`borg-caution.c`):

* **Before DL100** (`borg_prepared_aux()`): 5 !Healing, **15 \*Healing\* +
  Life** combined, **10 !Speed**, and (if max SP > 100) a stack of
  !Restore Mana (the check is `< 15`).
* Restock at DL100: if it arrives with fewer than 15 \*Healing\* and has
  been on the level < 10 turns, it leaves at once.
* **Uniques first**: once its max depth is past 98, the borg will not go
  to DL98+ (and so not to DL100) while **any non-quest unique is still
  alive** (`borg_numb_live_unique`, which skips the quest monsters and
  seasonal uniques, `borg-update.c`); it may dive to DL99 and use Teleport
  Level to get back to DL98 if needed. (With `borg_kills_uniques` it also
  refuses, at any depth, to go deeper than the native depth of the third
  shallowest living unique: "Unique Monsters who normally reside on
  levels 12, 15, and 18 are still alive so the borg will not dive deeper
  than level 18" — `borg.txt`.)
* In questor fights the borg drinks Life when 5+ stat "votes" need fixing,
  otherwise \*Healing\*, Healing, Holy Word, etc. when HP ≤ 700
  ("Healing in Questor Combat").
* Its escape thresholds loosen near questors: e.g. tier 1 fires at > 4.0×
  HP on DL100 when HP < 600, > 3.0× on DL99.
* For Morgoth the borg builds a **"sea of runes"** — a pattern of glyphs of
  warding in a room at least 7×7 (`borg-flow-glyph.c`) — and stands in it,
  and will also fight from an "anti-summon corridor". It carries ?Mass
  Banishment once Sauron is dead.

Human plan:
1. Clear (or skip) the other deep uniques you can; they will otherwise join
   the endgame fights.
2. Stockpile: the borg's list above is a good floor (15+ \*Healing\*/Life,
   5+ !Healing, 10+ !Speed, Restore Mana for casters, Mass Banishment).
3. Sauron: kill him on DL99 (quest level, classic layout). Read Recall
   right after the kill or take the magical staircase.
4. Morgoth: he comes to you. Fight on prepared ground (glyphs, corridor),
   with speed potions running, healing on keymaps. Do not wander DL100
   unprepared (`docs/dungeon.rst`).
5. After the "*** CONGRATULATIONS ***" message, retire with `Q` when you
   are done — until you do, you can still die.

---

## 8.8 Diving technique: stair-scumming and feeling-scumming

With connected stairs (Chapter 7.8) every staircase you take leaves you
standing on one going back. That enables:

**Stair-scumming down (fast dive)**
1. Arrive on DL d (you are on `<`).
2. Read the feeling. If danger is high, press `<` immediately; you arrive
   on DL d−1 standing on `>`; press `>` again for a fresh DL d.
3. If the level looks fine, find the nearest `>` (detect/map, or
   `autoexplore_commands` `>`), descend, repeat.

**Feeling-scumming (looking for loot)**: as above, but also wait for the
object feeling (see ~10 of the 100 hidden feeling squares — roughly a
tenth of the open floor, Chapter 7.9.2) and stay only on `LF:low-high`
levels.

**Unique scumming**: alternate `<`/`>` at the native depth of a unique you
want (its drop is better deeper — guide); the danger feeling jumps when a
deep unique is present (the OOD term in the monster rating is cubic).

The borg's own variants (`borg.txt`): *munchkin start* stair-scums DL5–6
for items early on; *lunal mode* "bounces between levels to take nearby
down stairs" to dive while still low level. The borg's docs warn of the
cost: "Often he will come down the stairs, and find himself right next to
a monster … Even +3 speed would help to avoid this type of instadeath."

Costs of scumming: game turns pass (shops restock while you are in the
dungeon — Chapter 7.2.4 — which is a *benefit* for BM hunting), and every
new arrival is a fresh exposure. You always get the first move on arrival
(`hints.txt`), so the safe habit is: arrive → look (`[`, feeling) →
decide → act.

---

## 8.9 When to go back to town

Go home (read Recall early, then survive the 15–34 turns) when any of
these is true:

* You are below the **borg restock line** for your depth (8.10.2).
* You have used **3 escapes on this level** or a "scary guy" is present
  and you can't leave by stairs.
* HP < ⅓ and fewer than 3 CCW and no !Healing (borg).
* A big upgrade is waiting: new resist item to slot in, stat potions to
  buy, a book you need.
* The BM should be checked: shops restock once per 10 000 game turns spent
  in the dungeon (Chapter 7.2.4).

The borg also never restocks within the first 200 turns after leaving town
(no ping-ponging) and spends at least 100 turns on a level before deciding
it must return (`borg_must_return_to_town()`), except on DL100.

Use the recall-depth reset (Chapter 7.10.1) if you want to come back to a
shallower depth than your max: read Recall on the shallower level and
answer "yes" to "Set recall depth to current depth?". That also resets the
target of Deep Descent.

---

## 8.10 The borg-derived depth readiness tables

### 8.10.1 Prepared to go to depth D (`borg_prepared()` / `borg_prepared_aux()`)

Default configuration (`borg_plays_risky = FALSE`, `borg.txt`). Conditions
are cumulative: to go to DL30 you need everything in the rows above it
too. "Cures" = CLW + CSW + CCW potions; "teleports" = ?Teleportation +
Staff of Teleportation (+charges if usable) + teleport activations; a
reliable teleport *spell* counts as 1000 (`borg-trait.c`).

| Depth | Requirement (borg) |
|---|---|
| any > 1 | **clvl ≥ depth** (until clvl 50) — `"Clevel < depth"` |
| 1 | light radius ≥ 1; food ≥ 5 |
| 2 | 5 fuel if no light; **30 max HP** |
| 3–4 | Warrior/Blackguard: 50 HP, clvl 4. Rogue: 50 HP, clvl 8. Priest/Druid: 40 HP, clvl 9. Paladin: 50 HP, clvl 4. Ranger: 50 HP, clvl 4. Mage/Necromancer: 60 HP, clvl 11. **2 cures** (if clvl < 30) |
| 5–9 | Warrior/Blackguard 60 HP, clvl 6; Rogue 60, 10; Priest/Druid 60, 15; Paladin 60, 6; Ranger 60, 6; Mage/Necro 80, 15. 2 cures; **1 ?Recall** |
| 10–19 | **light radius 2** (not necromancers); **2 teleports**; Warrior/Blackguard clvl ≥ D − 4; Rogue/Priest/Druid/Paladin/Ranger clvl ≥ D; Mage/Necro clvl ≥ D + 5 (until clvl 28); **3 CCW** (clvl < 30); **See Invisible, Detect Invisible or ESP** |
| 20 | **Free Action** |
| 21–25 | **rFire**; **2 of rAcid/rCold/rElec**; STR, DEX, CON and spell stat ≥ 7; class margin: Warrior/Blackguard clvl ≥ D + 5 (until 38); Rogue D + 10 (until 43); Priest/Druid D + 13 (until 46); Paladin D + 7 (until 40); Ranger D + 8 (for clvl 29–41); Mage/Necro D + 8 (until 38), then (clvl − 38)·2 + 30 ≥ D for clvl 39–44 |
| 26–33 | **all four basic resists**; **6 teleports**; **10 CSW+CCW** (clvl < 30) |
| 34–39 | **clvl 40** |
| 40–45 | **rPois**, **rConf**; STR, DEX, CON, spell stat ≥ **16** |
| 46–55 | **speed ≥ +5**; **1 !Healing or \*Healing\***; **500 max HP**; STR ≥ 18/40; spell stat 18/100; DEX, CON ≥ 18/60; **Hold Life** (unless clvl 50) |
| 56–59 | **2 heals** (or 1 \*Healing\*); **rBlind**; **ESP** |
| 60–80 | **speed ≥ +10**; **rChaos**; **rDisen** |
| 81–85 | **speed ≥ +20** |
| 86–99 | (nothing new) |
| 82+ (extra) | ≥ 10 \*Healing\* + Life (carried or at home); ≥ 30 once max depth ≥ 97 |
| 98+ (extra, max depth ≥ 99) | every non-quest unique dead |
| 100 | 5 !Healing; 15 \*Healing\* + Life; 10 !Speed; !Restore Mana stack if max SP > 100 |

(The alternative "dynamic" formula set in `borg.txt` `Depth Requirement`
encodes the same gates, with resists allowed to come from swap items —
"resist cold with swap" etc.)

### 8.10.2 Restock: go home if below (`borg_restock()`)

| At depth | Must still have |
|---|---|
| 1+ | light radius ≥ 1; fuel ≥ 1 if no light |
| 2+ | fuel ≥ 2 (no light); food ≥ 3 |
| 6+ | 1 ?Phase Door; 1 cure (clvl < 30) |
| 10+ | light radius 2 (not necros); 2 cures (clvl < 30); 2 teleports |
| 20+ | 4 CSW+CCW (clvl < 30); 4 teleports |
| 36+ | 2 of (?Teleportation + ?Teleport Level) |
| 65+ | 1 heal (!Healing, Rod of Healing, or \*Healing\*) |
| 100 | 15 \*Healing\* (if just arrived) |

### 8.10.3 What the borg rewards carrying (`borg_power()` in `borg-power.c`)

These caps show where the borg stops valuing extra copies — a good guide to
pack sizes:

| Item | Rewarded up to |
|---|---|
| Fuel | 6 (10 with STR ≥ 15) |
| Food | 7 (10 with STR ≥ 15; 15 if Regeneration and clvl ≤ 15) |
| ?Word of Recall | 3 (7 with STR ≥ 15), from clvl 8; Rod of Recall bonus from max depth 50 |
| ?Phase Door | first one heavily, then up to 8 (15 with STR ≥ 15) |
| Staff of Teleportation / teleport activation | 2 (3 more from max depth 71+) |
| ?Teleportation | 1 from clvl 3; 3 from clvl 7; 10 from clvl 30 |
| ?Teleport Level | 5, from clvl 15 |
| !CCW | 10 (15 with STR > 15), clvl 11+ |
| !CSW | 7 (10) only while short of CCW |
| !Healing | warriors/rogues/blackguards 15; rangers/paladins/necros/mages 10 (paladins also +3); priests/druids 5 |
| \*Healing\* | 2 carried from max depth 46 (1 while preparing a big fight); unlimited at max depth 99 |
| !Speed (and speed staff charges) | 20 (unlimited at max depth 99) |
| !Restore Mana | 10, if max SP > 100 |
| Ammo | warriors/rangers 40 (80 / 180 with high STR); others 20 (50) |

### 8.10.4 What the borg flees (`borg-flow-kill.c`)

See 8.5.5.

---

## 8.11 "Never do this": the classic deaths

Each item below is either a mechanic in the code or a borg rule designed to
prevent it.

1. **Going below DL20 without Free Action.** Carrion crawlers (DL25) and
   ghouls (DL26) come in groups; paralysis chains are fatal. (Borg gate.)
2. **Fighting a pack in an open room.** Hounds, wolves, wargs, hellhounds:
   get into a corridor.
3. **Reading Deep Descent as an emergency escape.** It takes 4–7 turns and
   sends you to *max depth + 5*, not current + 5 (`list-effects.h`). From
   max depth 96 it drops you on Sauron's level.
4. **Reading Word of Recall as an emergency escape.** 15–34 turns. Read it
   *before* you need it.
5. **Relying on scrolls while blind or confused.** You can't read. Carry a
   Staff of Teleportation and cure potions (`hints.txt`: "Always carry
   around an escape that you can use while blind and confused").
6. **Teleporting on the arrival side of a gauntlet.** `NO_TELEPORT` —
   "Teleportation forbidden!". Take the stairs back up.
7. **Walking into lava.** 100 + 1d100 fire damage per step before
   resistance (Chapter 7.6).
8. **Exploring a level that "feels murderous".** LF 8–9 means a seriously
   out-of-depth monster set (Chapter 7.9.1). Leave via the connected stair.
9. **Ignoring the "scary guy" list.** Gravity/inertia hounds, dreads,
   ancient dragons and Beorn before clvl 45; drolem without rPois;
   dracolich/dracolisk/Ossë without rNether; chaos breathers without
   rChaos/rConf; light/dark hounds without rBlind.
10. **Letting breeders explode.** Close the door, leave the level (borg:
    flee breeder levels before clvl 15).
11. **Teleporting twice away from the same big monster.** You may land next
    to it (`hints.txt`).
12. **Opening a pit or vault blind.** Detect first. Pit/nest monsters are
    generated *awake* behind a single door (Chapter 7.5.6); vault `8`s can
    be 40 levels out of depth.
13. **Carrying one copy of something vital.** Fire burns scrolls/books,
    cold shatters potions; casters should carry spare books.
14. **Going deeper because you can, not because you're ready.** clvl ≥ depth
    is the borg's minimum, not its target.
15. **Fighting on after three escapes on the same level.** The borg leaves.
16. **Playing tired.** The guide's first "face-palm" tip. Save, take a
    break.
17. **Running out of light.** "About to run out of light is extremely
    nasty" (`borg_caution()`); the dungeon is pitch black below DL25.
18. **Starving.** The borg treats Weak as an emergency and flees the level
    for food.
19. **Wandering DL100 before you are ready.** Morgoth comes to you.
20. **Forgetting to retire after winning.** You can still die until `Q`.

---

## 8.12 Class game plans (summary)

Another chapter covers classes in depth; these are the strategic outlines,
with the borg's class-specific depth margins from 8.10.1. Spell levels are
from `lib/gamedata/class.txt`.

### Warrior and Blackguard (melee)

* Fastest depth progression in the borg (clvl ≥ D − 4 for DL10–19;
  D + 5 from DL21). Hit dice 9 (warrior) / 8 (blackguard), up to 6 / 5
  blows.
* No escape spells: you live on scrolls, staffs and potions. Carry the
  biggest stacks of Phase, Teleport, CCW and later !Healing (the borg
  rewards warriors up to 15 !Healing and up to 6 Rods of Healing).
* Ranged backup: the borg rewards warriors up to 40–180 missiles; throw oil
  early, use a launcher always.
* Blackguard spells (`class.txt`): Seek Battle (1), Berserk Strength (3),
  Whirlwind Attack (5), Shatter Stone (7), Leap into Battle (10),
  Grim Purpose (13), Maim Foe (15), Howl of the Damned (18),
  Relentless Taunting (24), Venom (28), Bloodlust (30), Werewolf Form (32),
  Unholy Reprieve (34), Forceful Blow (38), Quake (44).
* Warriors are scared without missiles when afraid (borg tier-3 rule):
  keep !Heroism/!Berserk or rFear.

### Mage and Necromancer (casters)

* Slowest borg progression: DL3–4 needs clvl 11 and 60 HP; DL5–9 clvl 15
  and 80 HP; DL10–19 clvl ≥ D + 5; DL21+ clvl ≥ D + 8 (until clvl 38).
* Mage escapes come early: Phase Door (clvl 2), Teleport Self (7),
  Teleport Level (28), Dimension Door (35); detection: Find Traps/Doors/
  Stairs (1), Detect Monsters (3), Reveal Monsters (15), Detection (30);
  Deep-game tools: Banishment (35), Mass Banishment (40), Mana Storm (45).
* Necromancers see without light (unlight) and dislike it; the borg waives
  the light-radius-2 rule for them. Key spells: Nether Bolt (1), Bat Form
  (3), Read Minds (4), Shadow Shift (15), Dark Spear (25), Warg Form (28),
  Annihilate (30), Zone of Unmagic (32), Vampire Form (37), Unleash Chaos
  (38), Fume of Mordor (40), Storm of Darkness (43), Command (45).
* Low HP means the escape tiers fire earlier for you; the borg mage even
  escapes at > 0.3× HP when it has ≤ 20 SP at clvl 45+.
* Carry !Restore Mana from max SP > 100 (borg rewards 10).

### Priest and Paladin

* Priest/druid borg margins: clvl 9 for DL3–4, clvl 15 for DL5–9, clvl ≥ D
  for DL10–19, clvl ≥ D + 13 for DL21+ (until 46). Paladins: clvl 4 / 6,
  then D, then D + 7 (until 40).
* Priest tools: Minor Healing (1), Bless (1), Portal (10), Remove Curse
  (16), Glyph of Warding (20), Word of Recall (25), Banish Evil (25),
  Healing (30), Word of Destruction (35), Holy Word (39). Glyph + Healing
  + Holy Word make priests the best endgame survivors — the borg lets
  priests keep healing even when it starts conserving potions at max
  depth 98+.
* Paladin tools: Bless, Minor Healing (7), Heroism (12), Protection from
  Evil (15), Word of Recall (25), Healing (30), Single Combat (40 — the
  arena, where recall and teleport level do not work).

### Rogue and Ranger (archers / skirmishers)

* Rogue: clvl 8 for DL3–4, 10 for DL5–9, D for DL10–19, D + 10 for DL21+.
  Spells: Detect Monsters (5), Phase Door (7), Object Detection (10),
  Detect Stairs (12), Teleport Self (17), Recharging (20), Hit and Run
  (23), Teleport Level (35). Best disarm skill (45 base,
  `skill-disarm-phys:45:20`): locked doors and traps are not an issue.
* Ranger: clvl 4 / 6, then D, then D + 8 for clvl 29–41. Starts with a
  Short Bow and 15–20 arrows. Spells: Remove Hunger (3), Detect Life (5),
  Herbal Curing (9), Resist Poison (12), Stone to Mud (15), Cover Tracks
  and Sense Surroundings (20), Create Arrows (22), Haste Self (25), Decoy
  (30), Brand Ammunition (40). "Rangers have a bow … Don't try to play a
  ranger like a warrior" (`docs/guide.rst`). The borg rewards rangers up to
  180 missiles.

### Druid

* Same borg margins as priests (clvl 9 / 15, then D, then D + 13).
* No weapon focus; starts with a Whip. Key spells: Stinking Cloud (3),
  Confuse/Slow Monster (5/6), Cure/Resist Poison (4/7), Stone to Mud (8),
  Sense Surroundings (9), Lightning Strike (12), Earth Rising (14), Trance
  (20), Haste Self (25), Mass Sleep (25), Meteor Swarm (30), Rift (35),
  Eagle's Flight (35), Ice Storm (37), Rapid Regeneration (37), Herbal
  Curing (40), Volcanic Eruption (40), River of Lightning (43).
* Haste Self at clvl 25 solves the early speed problem; Herbal Curing
  replaces CCW in the late game.

---

## 8.13 One-page summary

| Phase | Depth | Borg clvl floor | Key gates | Kit (minimum → comfortable) |
|---|---|---|---|---|
| 1 | DL1–5 | class: 4–11 for DL3, 6–15 for DL5 | light, food, 2 cures, Recall | 3–5 Phase, 2 CLW, 1 Recall, oil |
| 2 | DL5–15 | ≥ D (warrior D−4, mage D+5) | light 2, 2 teleports, 3 CCW, SI/ESP | 5–8 Phase, 3–5 CCW, 2–3 teleport, 2 Recall |
| 3 | DL15–30 | ≥ D, then D+5 … D+13 | FA (20), rFire+2 (21), 4 basics + 6 tele + 10 cures (26) | 8 Phase, 10 CCW/CSW, 4–6 teleport, Teleport Level |
| 4 | DL30–45 | **40 for DL34+** | rPois, rConf, stats 16 (40) | 10 CCW, 1–3 !Heal, 6–10 tele, 3–5 TL, !Speed |
| 5 | DL45–70 | 40–50 | +5 speed, 500 HP, !Heal, Hold Life (46); rBlind, ESP (56); +10, rChaos, rDisen (61) | 3–5 !Heal, 1–2 \*Heal\*, 10 tele, 10–20 !Speed |
| 6 | DL70–98 | 50 | +20 speed (81); 10 → 30 \*Heal\* stockpile (82+) | 5+ !Heal, 5–10 \*Heal\*, 20 !Speed, Banishment |
| 7 | DL99–100 | 50 | all other uniques dead; 5 !Heal, 15 \*Heal\*/Life, 10 !Speed, Restore Mana | as much as you can carry |

---

## 8.14 Where the gates come from: an acquisition roadmap

The depth gates in 8.10 are only useful if you know where each property
can come from and how deep it starts appearing. Native levels and
allocation ranges below are `level` / `alloc:commonness:min to max` from
`object.txt`, and `alloc` from `ego_item.txt` (an ego type can be rolled
only within its depth range; see `obj-make.c`, which also lets an ego
appear early with chance `1 in max(2, (alloc_min − level)/3)`).

### 8.14.1 Race (free from birth, `p_race.txt`)

| Race | Relevant intrinsic |
|---|---|
| Gnome | **Free Action** |
| High-Elf | **See Invisible**, rLight |
| Hobbit | **Hold Life** |
| Dwarf | **protection from blindness** |
| Kobold | **rPoison** |
| Elf | rLight, sustain DEX |
| Half-Orc | rDark |
| Half-Troll | Regeneration, sustain STR |
| Dunadan | sustain CON |
| Half-Elf | sustain DEX |

A Gnome never has to worry about the DL20 Free Action gate; a High-Elf
clears the DL10 See Invisible gate; a Kobold gets the DL40 rPois gate for
free; a Dwarf has the blindness half of the DL56 gate.

### 8.14.2 Free Action (borg gate DL20)

| Source | Depth window |
|---|---|
| Ring of Free Action | level 20, alloc 20–100 |
| Gloves of Free Action | 1–60 (commonness 100) |
| Boots of Free Action | 1–40 |
| (Defender) weapons | 10–80 (also SI, basic resists, Feather, Regen) |
| Weapons of Westernesse | 10–70 (also SI, STR/DEX/CON) |
| Weapons of Gondolin | 20–127 (also SI, rDark) |
| (Dwarven) armour / shields / DSM | 20+, 40+ |
| Crown of Might | 40+ |
| Gloves of Thievery | 40+ |
| Bows of Lothlórien | 20+ |
| Amulet of Inertia | 15–40 (but −speed) |
| Amulet of Weaponmastery, of the Magi | 70+ |

Gloves of Free Action are the most common route (commonness 100 over
DL1–60, the highest of any Free Action source).

### 8.14.3 See Invisible / telepathy (borg gate DL10; ESP DL56)

| Source | Depth |
|---|---|
| Ring of See Invisible | level 30, 30–100 |
| Helm of Seeing | 10–60 (also protection from blindness) |
| Crown of Night and Day | 10–80 (SI, pBlind, rLight, rDark) |
| Lantern of True Sight | 20–127 (SI, pBlind) |
| (Holy Avenger), (Defender), Westernesse, Gondolin weapons | 10–127 |
| Potion of True Seeing (temporary) | 3–40 |
| Helm of Telepathy | 40–127 |
| Amulet of ESP | level 50, 60–100 |
| Mushroom of Second Sight (temporary ESP) | |
| spells: Priest Sense Invisible (3), Paladin (8), Necromancer (2); Mage Reveal Monsters (15); Rogue Reveal Monsters (25) | |

The borg accepts **Detect Invisible** (the scroll/staff, "DINV") in place
of See Invisible for the DL10 gate.

### 8.14.4 Basic resists (borg: rFire + 2 by DL21, all four by DL26)

| Source | Depth |
|---|---|
| Soft armour / shields "of Resist Fire/Cold/Acid/Lightning" | 1–30 (commonness 100 — very common early) |
| Weapons of Flame/Frost/Acid/Lightning | 1–50 / 1–60 |
| Ring of Resist Fire and Cold | level 10, 10–50 |
| Amulets of Resist Acid / Resist Lightning | 10–50 (Resist Acid is Magic-shop stock) |
| Rings of Flames / Ice / Acid / Lightning | 20–100 |
| Armour / shields "of Resistance" (all four) | 10–100 |
| "of Elvenkind" armour/shields (all four + a random high resist) | 30–127 |
| Robe of Permanence (all four, sustains, Hold Life, random high resist) | 30–127 |
| Crown of the Magi (all four, INT) | 20–127 |
| Amulet of Resistance | level 60, 60–100 |
| Dragon Scale Mail: White (rCold, 30+), Blue (rElec, 25+), Black (rAcid, 35+), Red (rFire, 40+), Multi-Hued (all five, 75+) | |
| temporary: !Resist Heat/Cold (Alchemist), Mushroom of Emergency | |

Resistances reduce the four base elements to 1/3 (`hints.txt`), and
identical resistances do not stack, so the aim is *coverage*.

### 8.14.5 Poison and confusion (borg gate DL40)

| Source | Depth |
|---|---|
| Ring of Resist Poison | level 40, 40–100 |
| Green DSM (rPois) | 60+; Multi-Hued DSM 75+; Power DSM 100 |
| Amulet of Trickery (rPois, rNexus, speed) | 70+ |
| random high resists on Elvenkind / Permanence / Preservation egos | 30+ / 40+ |
| temporary: !Resist Poison (20+), Druid/Ranger Resist Poison spells (7 / 12) | |
| Confusion: Crown of Serenity (pConf, pFear, pStun, rSound) | 20–127 |
| temporary confusion protection: Mushroom of Clear Mind | |

Many artifacts carry these too (`artifact.txt` has 138 artifacts).

### 8.14.6 Hold Life, blindness, chaos, disenchantment (DL46–61 gates)

| Property | Sources |
|---|---|
| Hold Life | Hobbit; Robe of Permanence (30+); Shield of Preservation (40+); Amulet of Sustenance (60+); Amulet of Devotion (70+); weapons of Morgul (but they drain XP and aggravate — avoid) |
| Protection from blindness | Dwarf; Helm of Seeing (10–60); Crown of Night and Day; Lantern of True Sight; Amulet of the Magi (70+) |
| rChaos | Chaos DSM (85+), Balance DSM (95+), Power DSM (100); random high resists; artifacts |
| rDisen | Shield of Preservation; Chaos/Balance/Power DSM; Amulet of Weaponmastery (70+); artifacts |
| rNether / rSound / rShards / rNexus | random high resists; Gold DSM (rSound, 70+), Law DSM (rSound, rShard, 80+); Boots of Stability (rNexus, 15–100); Cloak of Protection (rShard, 1–40); Crown of Serenity (rSound) |

### 8.14.7 Speed (borg: +5 by DL46, +10 by DL61, +20 by DL81)

| Source | Depth / amount |
|---|---|
| Boots of Speed | 20–127, `SPEED[2+M8]` |
| Boots of Elvenkind | 60–127, `SPEED[4+M5]` |
| Dragon armour of Speed | 40–127, `SPEED[d5]` |
| Ring of Speed | level 75, 75–100, `SPEED[4+M6]` |
| Amulet of Trickery | 70+, `SPEED[1+M2]` |
| Rings of Teleportation / Escaping | +2 speed, but random teleports / permanent fear |
| Druid and Ranger Haste Self | clvl 25 |
| Potion of Speed / Staff / Rod of Speed | temporary +10 |
| Mushroom of Terror | temporary +10, stacks with !Speed, but you are afraid |

Rings of Speed only start at DL75; for most characters the realistic
early sources are Boots of Speed and artifacts, which is why the borg
treats +5 at DL46 as the first hard speed gate.

---

## 8.15 Threat classes and their counters

| Threat | First appears (native DL) | Counter |
|---|---|---|
| Fast early packs (Grip/Fang, jackals, cave spiders, wolves, wargs) | 2–14 | corridors; don't fight in rooms; leave if you can't |
| Breeders (lice, worm masses, mice) | 1–20 | close doors, leave; borg flees breeder levels below clvl 15 |
| Thieves (Sméagol, cutpurses, squint-eyed rogue) | 0–3+ | kill at range or avoid; carry little gold early |
| Invisible monsters (ghosts, imps, moaning spirits, poltergeists) | 3–31 | See Invisible / ESP (borg gate DL10) |
| Paralysis (carrion crawlers, ghouls, floating eyes, basilisks) | 25–45 | **Free Action** (borg gate DL20) |
| Blindness (light/dark hounds, shimmering vortex) | 15+ | CLW/CSW/CCW cure it; Staff of Teleportation; pBlind (borg gate DL56) |
| Confusion (chaos breathers, Pukel-man, night mare, many casters) | ~20+ | CCW, Clear Mind, rConf (borg gate DL40), staffs |
| Basic-element breath (Zephyr hounds, vortices, young/mature dragons, giants) | 18–43 | four basic resists (borg DL21/DL26) |
| Poison breath (drolem, green/MH dragons, basilisks) | 29–54 | rPois (borg DL40), !Resist Poison, CCW |
| Gravity / inertia / impact / nexus hounds | 27–35 | speed, corridors; borg leaves gravity/inertia levels below clvl 45 |
| Dreads (pass walls, packs) | 44 | ESP to see them in walls; borg-scary below clvl 45 |
| Nether (dracolich, dracolisk, Ossë, nether hounds) | 51–72 | rNether, Hold Life; borg flees without rNether |
| Time (time hounds, time vortices) | 38 / 51 | speed; !Restore Life Levels; there is no time resistance for players (`hints.txt`: time attacks are not mitigated by sustains) |
| Chaos / disenchantment | 33+ | rChaos, rDisen (borg gate DL61) |
| Summoners (quylthulgs, many uniques, greater quylthulgs) | 20 / 45 / 71 | kill first at range, Banishment, leave |
| Exp drain (wights, wraiths, vampires, Black Breath) | 24+ | Hold Life (borg DL46), !Restore Life Levels (Alchemist) |
| Stat drain (darts, time, nexus swaps) | 6+ | sustains, !Life, Mushroom of Vigor, Mushroom of Purging (restores STR and CON) |
| Inventory destruction (fire/cold/elec/acid) | early | immunities only fully protect (`hints.txt`); carry spares |
| Trapdoors | 2+ | Feather Fall only removes the damage, not the fall |

---

## 8.16 Routines

### 8.16.1 Arriving on a level

1. You get the first move (`hints.txt`). Do not move yet.
2. Read the danger feeling (status bar `LF:`). 7–9 → consider `<`/`>`
   immediately.
3. Check `[` (visible monster list). Use detection (spell, rod, staff,
   scroll) if you have it: monsters first, then map/stairs.
4. Decide: stay, or take the staircase you are standing on.
5. If staying, note the nearest escape route back to the stairs and the
   nearest corridor.
6. After ~a tenth of the floor has been seen, re-check `LF:` for the loot
   digit (Chapter 7.9.2) and decide whether exploring is worth it.

### 8.16.2 Fighting a unique

1. Look it up (`/`, `l`, `~` monster knowledge): speed, HP, spells,
   breaths, summons, escort.
2. Compare its best attack with your HP and resists (8.5). If it can kill
   you in one or two turns, don't fight it today.
3. Clear or avoid the escort first (the borg uses Teleport Other on
   non-uniques to get "mano-e-mano", `borg-fight-defend.c`).
4. Pre-buff out of sight: !Speed, Hero/Berserk, Bless, Protection from
   Evil, temporary resists — the borg's "setup moves" list
   (`borg-fight-defend.c`).
5. Fight from a corridor or a doorway, ideally near stairs.
6. Keep the escape ladder in mind; uniques make the borg tolerate a little
   more danger (tier thresholds rise to 1.3×–1.7×), not a lot.

### 8.16.3 Town trip

1. Read Recall early (15–34 turns), survive the delay.
2. In town: drop/store at Home, buy cures and escapes first, check the
   Alchemist for CCW, the Magic shop for Staff of Teleportation charges,
   the BM for stat potions/Speed/Healing.
3. Re-inscribe new stacks if needed; set keymaps for new spells.
4. Check the `C` sheet against the next gate in 8.10.1.
5. Read Recall back (to max depth), or take the stairs and dive.

---

## 8.17 Frequently asked strategic questions

**Should I use Deep Descent to dive?**
Only from your max depth, and only when you are ready for max + 5
(Chapter 7.10.2). It is a superb diving tool for a character who is
*ahead* of its gates — five levels in 4–7 turns, without searching for
stairs — and a trap for one that is not. If you have retreated to a
shallower recall depth, reset max depth first (Recall prompt) or the
scroll will overshoot.

**Is Teleport Level a safe escape?**
It is the fastest one (instant) but it is a coin flip between up and down
(`effect_handler_TELEPORT_LEVEL`). Going down one level from a bad
situation is usually still better than staying. It does not work on
`NO_TELEPORT` squares (gauntlets) or with a `NO_TELEPORT` curse, and it
never takes you below a living quest monster's level.

**Do I need to kill the uniques?**
Not to progress — except that both quest monsters must die to win
(`quest_check()`). The borg's optional `borg_kills_uniques` mode refuses
to go deeper than the native depth of the third-shallowest living unique,
and once past DL98 it refuses DL98+ while any non-quest unique lives
(`borg_prepared()`). Uniques drop better items deeper
(`docs/guide.rst`), so it is reasonable to postpone dangerous ones.

**How fast should I dive?**
The borg's floor is clvl ≥ depth, with class margins (8.10.1); its hard
plateaus are DL20 (Free Action), DL26 (four resists), DL34 (clvl 40),
DL40 (rPois, rConf), DL46 (+5 speed, 500 HP), DL61 (+10 speed, rChaos,
rDisen) and DL81 (+20 speed). Human players routinely dive faster than the
borg by being better at avoiding fights; the gates on *defences* are the
part not to skip.

**Is it worth opening pits and nests?**
Pit monsters are drawn 10 levels deeper than the current depth and are not
generated asleep (Chapter 7.5.6). A pit is worth opening when its theme is
something you dominate (e.g. orcs or trolls for a strong melee character,
ideally from a corridor), and not otherwise. Some themes leave floor
items (Undead nests 20% per square, Animals 10%).

**When do I start ignoring "good" items?**
When ordinary `+x,+y` items are no longer upgrades — usually somewhere in
Phase 4 — so that your item list (`]`) shows only egos and artifacts.
Under no-selling, there is no reason to pick up items you won't use.

**What do I do on a level whose feeling is `LF:9-9`?**
Something very out of depth is there (danger 9 is x > 7000 per level of
depth) along with superb loot. Detect monsters first. If the danger is a
single slow monster you can avoid, loot carefully; otherwise leave — the
loot level of the next levels will be just as good on average.

**Can I go back up to farm?**
Yes. With connected stairs `<` works any time (unless you chose
`birth_force_descend`), and the recall-depth reset lets you make a
shallower level your new recall target. The guide's "Level 1 is still
there" section recommends it whenever you need to rebuild.

**How much is "a lot" of \*Healing\*?**
The borg's numbers: 10 before going below DL82, 30 once it has been to
DL97, 15 carried into DL100 alongside 5 !Healing and 10 !Speed
(`borg_prepared()`, `borg_prepared_aux()`).

**What is the single most important habit?**
Run the escape ladder (8.5) *before* the turn in which you would need it.
The borg leaves a level after three escapes; you should too.

---

## 8.18 Source index for this chapter

* Borg readiness and restock: `src/borg/borg-prepared.c`
  (`borg_prepared_aux`, `borg_prepared`, `borg_restock`,
  `borg_must_return_to_town`); dynamic formulas: `src/borg/borg.txt`
  ("Depth Requirement", "Restock Requirement", power formula)
* Borg escapes and healing: `src/borg/borg-escape.c` (`borg_escape`),
  `src/borg/borg-caution.c` (`borg_heal`, `borg_caution`)
* Borg scary monsters: `src/borg/borg-flow-kill.c`
* Borg inventory valuation: `src/borg/borg-power.c`; item counting:
  `src/borg/borg-trait.c`
* Borg endgame: `src/borg/borg-flow-glyph.c` ("sea of runes"),
  `src/borg/borg-fight-defend.c`
* Items: `lib/gamedata/object.txt`, `store.txt`, `class.txt` (start kits,
  spells), `hints.txt`
* Mechanics: `src/effect-handler-general.c` (Recall, Deep Descent,
  Teleport Level), `src/effect-handler-attack.c` (HEAL_HP),
  `src/player-calcs.c` (timed speed bonuses), `src/mon-move.c` (glyph
  breaking), `src/store.c`, `src/obj-make.c`, `src/list-options.h`,
  `lib/gamedata/constants.txt`
* Official docs: `docs/guide.rst`, `docs/customize.rst`,
  `docs/playing.rst`, `docs/option.rst`, `docs/dungeon.rst`
