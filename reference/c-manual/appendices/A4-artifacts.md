# Appendix A4. Standard artifacts

> **Generated file - do not edit by hand.** Produced by `docs/c-manual/tools/gen_appendices.py` from `lib/gamedata/artifact.txt`, `lib/gamedata/activation.txt`, `lib/gamedata/object.txt`, `lib/gamedata/slay.txt`, `lib/gamedata/brand.txt` of the Angband source tree at commit `e473720ed`. Regenerate with `python3 docs/c-manual/tools/gen_appendices.py --src <angband-source-root>` (run from the repository root).

All 138 standard artifacts (not used with the `birth_randarts` option), grouped by item type and sorted by level: base item, level and allocation, combat values, modifiers, slays/brands, resistances and flags, curses, and activation resolved through `activation.txt`. How to judge and use them: [Chapter 6](../06-equipment.md).

## Contents

- [Legend](#legend)
- [Index](#index)
- [Sword (30)](#sword-30)
- [Polearm (19)](#polearm-19)
- [Hafted (14)](#hafted-14)
- [Digger (2)](#digger-2)
- [Bow (6)](#bow-6)
- [Soft Armor (3)](#soft-armor-3)
- [Hard Armor (7)](#hard-armor-7)
- [Dragon Armor (3)](#dragon-armor-3)
- [Shield (7)](#shield-7)
- [Cloak (7)](#cloak-7)
- [Gloves (9)](#gloves-9)
- [Boots (6)](#boots-6)
- [Helm (7)](#helm-7)
- [Crown (4)](#crown-4)
- [Light (3)](#light-3)
- [Ring (6)](#ring-6)
- [Amulet (5)](#amulet-5)

## Legend

- **Level** is the artifact level (also the activation difficulty). **Alloc** is `min-max depth (xcommonness)`. When an object of the artifact's exact base kind is allowed to become an artifact (`make_artifact()`; special one-of-a-kind items such as the Phial use `make_artifact_special()`, both in `src/obj-make.c`), the artifact is picked if `randint1(100) <= commonness`; shallower than its minimum depth it also needs a 1 in `2 * (min - depth)` roll; deeper than its maximum depth it is never generated. Each artifact exists once per game.
- **Combat**: weapons `dice (+hit, +dam)`, armour `[base AC, +AC]`, launchers `xmultiplier` (base item).
- **Mods**: stat and other modifiers; `resist`/`immune`/`vulnerable` are elemental resistance levels from `values:` (`RES_*`).
- **Activation**: activation name, description from `activation.txt` (or the effect list if no description), activation level; **recharge** in player turns (`time:`).
- **Modifier units** (`calc_bonuses()` / `calc_blows()` in `src/player-calcs.c`): `BLOWS` = whole extra blows; `SHOTS` = tenths of a shot per turn (`SHOTS +10` = one extra shot); `MIGHT` adds to the launcher multiplier; `SPEED` in speed points (+10 = twice as fast); `INFRA` in 10-foot steps; `LIGHT` = light radius; `TUNNEL`, `STEALTH`, `SEARCH` and the five stats add directly. `M8` in a value = a level-dependent bonus of 0..8 (see the `object.txt` header).

## Index

| Artifact | Type | Level | Alloc |
|---|---|---:|---|
| [Dagger 'Dethanc'](#dagger-dethanc) | Sword | 1 | 1-50 (x40) |
| [Long Sword 'Elvagil'](#long-sword-elvagil) | Sword | 1 | 10-50 (x50) |
| [Rapier 'Forasgil'](#rapier-forasgil) | Sword | 1 | 5-50 (x38) |
| [Dagger 'Narthanc'](#dagger-narthanc) | Sword | 1 | 1-50 (x40) |
| [Dagger 'Nimthanc'](#dagger-nimthanc) | Sword | 1 | 1-50 (x40) |
| [The Dagger of Rilia](#the-dagger-of-rilia) | Sword | 5 | 5-75 (x5) |
| [Dagger 'Belangil'](#dagger-belangil) | Sword | 10 | 10-80 (x2) |
| [Rapier 'Careth Asdriag'](#rapier-careth-asdriag) | Sword | 15 | 15-80 (x10) |
| [The Main Gauche of Maedhros](#the-main-gauche-of-maedhros) | Sword | 15 | 15-65 (x3) |
| [The Main Gauche of Azaghâl](#the-main-gauche-of-azaghâl) | Sword | 18 | 18-100 (x3) |
| [Broad Sword 'Aeglin'](#broad-sword-aeglin) | Sword | 20 | 20-100 (x3) |
| [Dagger 'Angrist'](#dagger-angrist) | Sword | 20 | 60-100 (x2) |
| [Long Sword 'Anguirel'](#long-sword-anguirel) | Sword | 20 | 20-127 (x3) |
| [Broad Sword 'Aranrúth'](#broad-sword-aranrúth) | Sword | 20 | 20-100 (x2) |
| [Short Sword 'Dagmor'](#short-sword-dagmor) | Sword | 20 | 20-80 (x12) |
| [Broad Sword 'Glamdring'](#broad-sword-glamdring) | Sword | 20 | 20-80 (x5) |
| [Cutlass 'Gondricam'](#cutlass-gondricam) | Sword | 20 | 20-65 (x12) |
| [Scimitar 'Haradekket'](#scimitar-haradekket) | Sword | 20 | 20-80 (x7) |
| [Broad Sword 'Orcrist'](#broad-sword-orcrist) | Sword | 20 | 20-80 (x5) |
| [Short Sword 'Sting'](#short-sword-sting) | Sword | 20 | 20-100 (x6) |
| [Katana 'Aglarang'](#katana-aglarang) | Sword | 30 | 30-100 (x4) |
| [Bastard Sword 'Calris'](#bastard-sword-calris) | Sword | 30 | 30-127 (x7) |
| [Zweihander 'Gurthang'](#zweihander-gurthang) | Sword | 30 | 30-100 (x3) |
| [Zweihander 'Mormegil'](#zweihander-mormegil) | Sword | 30 | 30-100 (x7) |
| [Zweihander 'Zarcuthra'](#zweihander-zarcuthra) | Sword | 30 | 30-127 (x1) |
| [The Bastard Sword of Éowyn](#the-bastard-sword-of-éowyn) | Sword | 30 | 30-127 (x3) |
| [Long Sword 'Andúril'](#long-sword-andúril) | Sword | 35 | 20-127 (x3) |
| [Executioner's Sword 'Crisdurian'](#executioners-sword-crisdurian) | Sword | 40 | 40-127 (x4) |
| [Long Sword 'Ringil'](#long-sword-ringil) | Sword | 40 | 20-127 (x1) |
| [Blade of Chaos 'Doomcaller'](#blade-of-chaos-doomcaller) | Sword | 70 | 70-127 (x4) |
| [The Spear of Oromë](#the-spear-of-oromë) | Polearm | 5 | 15-127 (x2) |
| [Spear 'Nimloth'](#spear-nimloth) | Polearm | 15 | 15-65 (x8) |
| [The Trident of Wrath](#the-trident-of-wrath) | Polearm | 15 | 15-100 (x3) |
| [Scythe 'Avavir'](#scythe-avavir) | Polearm | 20 | 45-100 (x12) |
| [Broad Axe 'Barukkheled'](#broad-axe-barukkheled) | Polearm | 20 | 20-75 (x12) |
| [Halberd 'Osondir'](#halberd-osondir) | Polearm | 20 | 25-80 (x12) |
| [Pike 'Til-i-arc'](#pike-til-i-arc) | Polearm | 20 | 20-80 (x7) |
| [The Beaked Axe of Húrin](#the-beaked-axe-of-húrin) | Polearm | 20 | 20-100 (x7) |
| [The Lance of the Eorlingas](#the-lance-of-the-eorlingas) | Polearm | 20 | 20-100 (x4) |
| [The Trident of Ulmo](#the-trident-of-ulmo) | Polearm | 27 | 30-127 (x1) |
| [Battle Axe 'Lotharang'](#battle-axe-lotharang) | Polearm | 30 | 30-100 (x7) |
| [Lochaber Axe 'Mundwine'](#lochaber-axe-mundwine) | Polearm | 30 | 30-100 (x12) |
| [The Battle Axe of Balli Stonehand](#the-battle-axe-of-balli-stonehand) | Polearm | 30 | 30-100 (x7) |
| [The Great Axe of Durin](#the-great-axe-of-durin) | Polearm | 30 | 40-127 (x1) |
| [The Great Axe of Eönwë](#the-great-axe-of-eönwë) | Polearm | 30 | 40-127 (x1) |
| [The Glaive of Pain](#the-glaive-of-pain) | Polearm | 30 | 30-127 (x4) |
| [The Beaked Axe of Théoden](#the-beaked-axe-of-théoden) | Polearm | 30 | 20-100 (x7) |
| [Spear 'Aiglos'](#spear-aiglos) | Polearm | 40 | 15-100 (x2) |
| [The Spear of Melkor](#the-spear-of-melkor) | Polearm | 65 | 65-127 (x2) |
| [Quarterstaff 'Eriril'](#quarterstaff-eriril) | Hafted | 15 | 20-100 (x5) |
| [Morning Star 'Bloodspike'](#morning-star-bloodspike) | Hafted | 20 | 20-75 (x3) |
| [Quarterstaff 'Nar-i-vagil'](#quarterstaff-nar-i-vagil) | Hafted | 20 | 20-80 (x5) |
| [Flail 'Totila'](#flail-totila) | Hafted | 20 | 20-65 (x12) |
| [Lead-Filled Mace 'Turmil'](#lead-filled-mace-turmil) | Hafted | 20 | 20-100 (x7) |
| [The Quarterstaff of Olórin](#the-quarterstaff-of-olórin) | Hafted | 20 | 30-100 (x1) |
| [Morning Star 'Firestar'](#morning-star-firestar) | Hafted | 25 | 20-100 (x7) |
| [Mace 'Taratol'](#mace-taratol) | Hafted | 35 | 20-100 (x7) |
| [The Great Hammer of Aulë](#the-great-hammer-of-aulë) | Hafted | 40 | 40-127 (x1) |
| [Two-Handed Great Flail 'Thunderfist'](#two-handed-great-flail-thunderfist) | Hafted | 45 | 45-127 (x3) |
| [The Whip of Gothmog](#the-whip-of-gothmog) | Hafted | 60 | 60-127 (x4) |
| [The Ball-and-Chain of Fundin Bluecloak](#the-ball-and-chain-of-fundin-bluecloak) | Hafted | 65 | 65-127 (x2) |
| [Mace of Disruption 'Deathwreaker'](#mace-of-disruption-deathwreaker) | Hafted | 80 | 80-127 (x3) |
| [Mighty Hammer 'Grond'](#mighty-hammer-grond) | Hafted | 100 | 100-100 (x0) |
| [The Mattock of Náin](#the-mattock-of-náin) | Digger | 10 | 60-100 (x12) |
| [The Pick of Erebor](#the-pick-of-erebor) | Digger | 55 | 55-100 (x20) |
| [The Short Bow of Amras](#the-short-bow-of-amras) | Bow | 25 | 30-80 (x10) |
| [The Short Bow of Amrod](#the-short-bow-of-amrod) | Bow | 25 | 30-100 (x10) |
| [The Long Bow of Bard](#the-long-bow-of-bard) | Bow | 30 | 40-127 (x5) |
| [Long Bow 'Belthronding'](#long-bow-belthronding) | Bow | 40 | 40-127 (x5) |
| [The Heavy Crossbow of Umbar](#the-heavy-crossbow-of-umbar) | Bow | 45 | 60-127 (x5) |
| [Light Crossbow 'Cubragol'](#light-crossbow-cubragol) | Bow | 50 | 50-127 (x4) |
| [Soft Leather Armour 'Hithlomir'](#soft-leather-armour-hithlomir) | Soft Armor | 20 | 20-80 (x20) |
| [Leather Scale Mail 'Thalkettoth'](#leather-scale-mail-thalkettoth) | Soft Armor | 20 | 30-100 (x20) |
| [The Hard Leather Armour of Himring](#the-hard-leather-armour-of-himring) | Soft Armor | 30 | 25-90 (x20) |
| [Mithril Chain Mail 'Belegennon'](#mithril-chain-mail-belegennon) | Hard Armor | 20 | 55-127 (x30) |
| [The Chain Mail of Arvedui](#the-chain-mail-of-arvedui) | Hard Armor | 20 | 20-75 (x30) |
| [The Augmented Chain Mail of Caspanion](#the-augmented-chain-mail-of-caspanion) | Hard Armor | 25 | 35-100 (x11) |
| [The Full Plate Armour of Isildur](#the-full-plate-armour-of-isildur) | Hard Armor | 30 | 55-127 (x30) |
| [The Metal Brigandine Armour of the Rohirrim](#the-metal-brigandine-armour-of-the-rohirrim) | Hard Armor | 30 | 45-100 (x30) |
| [The Mithril Plate Mail of Celeborn](#the-mithril-plate-mail-of-celeborn) | Hard Armor | 40 | 65-127 (x30) |
| [Adamantite Plate Mail 'Soulkeeper'](#adamantite-plate-mail-soulkeeper) | Hard Armor | 75 | 75-127 (x11) |
| [Power Dragon Scale Mail 'Bladeturner'](#power-dragon-scale-mail-bladeturner) | Dragon Armor | 55 | 100-127 (x6) |
| [Balance Dragon Scale Mail 'Mediator'](#balance-dragon-scale-mail-mediator) | Dragon Armor | 60 | 95-127 (x8) |
| [Multi-Hued Dragon Scale Mail 'Razorback'](#multi-hued-dragon-scale-mail-razorback) | Dragon Armor | 60 | 90-127 (x11) |
| [The Leather Shield of Celegorm](#the-leather-shield-of-celegorm) | Shield | 30 | 30-100 (x28) |
| [The Mithril Shield of Gil-galad](#the-mithril-shield-of-gil-galad) | Shield | 30 | 70-127 (x25) |
| [The Knight's Shield of Eärendil](#the-knights-shield-of-eärendil) | Shield | 32 | 65-127 (x6) |
| [The Leather Shield of the Haradrim](#the-leather-shield-of-the-haradrim) | Shield | 35 | 35-127 (x10) |
| [The Large Metal Shield of Anárion](#the-large-metal-shield-of-anárion) | Shield | 40 | 40-100 (x11) |
| [The Wicker Shield of Elros](#the-wicker-shield-of-elros) | Shield | 40 | 50-127 (x10) |
| [The Small Metal Shield of Thorin](#the-small-metal-shield-of-thorin) | Shield | 40 | 40-127 (x14) |
| [The Cloak of Thorongil](#the-cloak-of-thorongil) | Cloak | 5 | 5-75 (x10) |
| [Cloak 'Colluin'](#cloak-colluin) | Cloak | 12 | 5-100 (x2) |
| [Cloak 'Holcolleth'](#cloak-holcolleth) | Cloak | 13 | 5-100 (x5) |
| [Cloak 'Colannon'](#cloak-colannon) | Cloak | 26 | 5-100 (x5) |
| [The Elven Cloak of Lúthien](#the-elven-cloak-of-lúthien) | Cloak | 40 | 40-127 (x2) |
| [The Ethereal Cloak of Tuor](#the-ethereal-cloak-of-tuor) | Cloak | 40 | 70-127 (x2) |
| [The Cloak of Thingol](#the-cloak-of-thingol) | Cloak | 62 | 5-100 (x5) |
| [Set of Gauntlets 'Pauraegen'](#set-of-gauntlets-pauraegen) | Gloves | 1 | 10-75 (x23) |
| [Set of Gauntlets 'Paurhach'](#set-of-gauntlets-paurhach) | Gloves | 1 | 10-60 (x40) |
| [Set of Gauntlets 'Paurnen'](#set-of-gauntlets-paurnen) | Gloves | 1 | 10-60 (x30) |
| [Set of Gauntlets 'Paurnimmen'](#set-of-gauntlets-paurnimmen) | Gloves | 1 | 50-127 (x5) |
| [Set of Leather Gloves 'Cambeleg'](#set-of-leather-gloves-cambeleg) | Gloves | 10 | 10-100 (x3) |
| [Set of Gauntlets 'Camlost'](#set-of-gauntlets-camlost) | Gloves | 10 | 10-100 (x5) |
| [Set of Leather Gloves 'Cammithrim'](#set-of-leather-gloves-cammithrim) | Gloves | 10 | 10-80 (x18) |
| [The Set of Caestus of Fingolfin](#the-set-of-caestus-of-fingolfin) | Gloves | 40 | 40-127 (x3) |
| [The Set of Gauntlets of Eöl](#the-set-of-gauntlets-of-eöl) | Gloves | 55 | 55-127 (x3) |
| [Pair of Leather Boots 'Dal-i-thalion'](#pair-of-leather-boots-dal-i-thalion) | Boots | 10 | 50-100 (x4) |
| [The Pair of Leather Boots of Radagast](#the-pair-of-leather-boots-of-radagast) | Boots | 10 | 15-70 (x8) |
| [The Pair of Leather Boots of Wormtongue](#the-pair-of-leather-boots-of-wormtongue) | Boots | 10 | 15-80 (x8) |
| [Pair of Mithril Shod Boots 'Stormwalker'](#pair-of-mithril-shod-boots-stormwalker) | Boots | 30 | 60-127 (x3) |
| [The Pair of Leather Boots of Fëanor](#the-pair-of-leather-boots-of-fëanor) | Boots | 30 | 40-127 (x1) |
| [The Pair of Steel Shod Boots of Thrór](#the-pair-of-steel-shod-boots-of-thrór) | Boots | 30 | 30-100 (x5) |
| [The Metal Cap of Thengel](#the-metal-cap-of-thengel) | Helm | 10 | 10-100 (x18) |
| [Iron Helm 'Holhenneth'](#iron-helm-holhenneth) | Helm | 20 | 20-100 (x18) |
| [The Iron Helm of Gorlim](#the-iron-helm-of-gorlim) | Helm | 20 | 20-75 (x18) |
| [The Steel Helm of Hammerhand](#the-steel-helm-of-hammerhand) | Helm | 20 | 40-127 (x11) |
| [The Iron Helm of Dor-Lómin](#the-iron-helm-of-dor-lómin) | Helm | 40 | 40-127 (x5) |
| [The Hard Leather Cap of Thranduil](#the-hard-leather-cap-of-thranduil) | Helm | 50 | 40-100 (x14) |
| [The Metal Cap of Celebrimbor](#the-metal-cap-of-celebrimbor) | Helm | 55 | 55-100 (x8) |
| [The Iron Crown of Berúthiel](#the-iron-crown-of-berúthiel) | Crown | 40 | 45-100 (x8) |
| [The Golden Crown of Gondor](#the-golden-crown-of-gondor) | Crown | 40 | 45-127 (x5) |
| [The Jewel Encrusted Crown of Númenor](#the-jewel-encrusted-crown-of-númenor) | Crown | 60 | 60-127 (x5) |
| [The Massive Iron Crown of Morgoth](#the-massive-iron-crown-of-morgoth) | Crown | 100 | 100-100 (x0) |
| [The Phial of Galadriel](#the-phial-of-galadriel) | Light | 5 | 5-100 (x40) |
| [The Star of Elendil](#the-star-of-elendil) | Light | 30 | 30-100 (x4) |
| [The Arkenstone of Thráin](#the-arkenstone-of-thráin) | Light | 50 | 50-127 (x2) |
| [Ring of Fire 'Narya'](#ring-of-fire-narya) | Ring | 30 | 70-127 (x2) |
| [Ring of Adamant 'Nenya'](#ring-of-adamant-nenya) | Ring | 40 | 80-127 (x2) |
| [The Ring of Barahir](#the-ring-of-barahir) | Ring | 50 | 50-100 (x4) |
| [The Band of Tulkas](#the-band-of-tulkas) | Ring | 60 | 70-127 (x2) |
| [Ring of Power 'The One Ring'](#ring-of-power-the-one-ring) | Ring | 75 | 100-127 (x1) |
| [Ring of Firmament 'Vilya'](#ring-of-firmament-vilya) | Ring | 80 | 90-127 (x1) |
| [The Amulet of Carlammas](#the-amulet-of-carlammas) | Amulet | 30 | 50-100 (x10) |
| [Jewel 'Evenstar'](#jewel-evenstar) | Amulet | 40 | 30-70 (x2) |
| [The Necklace of the Dwarves](#the-necklace-of-the-dwarves) | Amulet | 50 | 50-127 (x2) |
| [Elfstone 'Elessar'](#elfstone-elessar) | Amulet | 60 | 60-127 (x2) |
| [The Pendant of Ingwë](#the-pendant-of-ingwë) | Amulet | 65 | 65-127 (x3) |

## Sword (30)

### Dagger 'Dethanc'

- **Base**: Dagger; **level** 1; **alloc** 1-50 (x40); **weight** 1.2 lb; **cost** 13000
- **Combat**: 1d4 (+9, +12) \[0, +10\]
- **Slays/brands**: brand lightning x3
- **Resists**: resist lightning
- **Activation**: ELEC\_BOLT: beam of lightning for 6d6 (level 1, aimed); recharge 6+d6 turns

> In this dagger is bound the very force of lightning.

### Long Sword 'Elvagil'

- **Base**: Long Sword; **level** 1; **alloc** 10-50 (x50); **weight** 13.0 lb; **cost** 30000
- **Combat**: 2d5 (+12, +12)
- **Mods**: DEX +2, STEALTH +2
- **Slays/brands**: slay trolls x3, slay orcs x3
- **Flags**: FEATHER, SEE\_INVIS

> The "Singing Blade", whose wearer can slay Orcs and Trolls in the hidden and secret places of the earth.

### Rapier 'Forasgil'

- **Base**: Rapier; **level** 1; **alloc** 5-50 (x38); **weight** 4.0 lb; **cost** 15000
- **Combat**: 1d12 (+12, +19)
- **Mods**: LIGHT +1
- **Slays/brands**: slay animals x2, brand cold x3
- **Resists**: resist cold, resist light

> A slender, tapered needle of ice that coldly grips its impaled prey.

### Dagger 'Narthanc'

- **Base**: Dagger; **level** 1; **alloc** 1-50 (x40); **weight** 1.2 lb; **cost** 12000
- **Combat**: 1d4 (+9, +12) \[0, +10\]
- **Slays/brands**: brand fire x3
- **Resists**: resist fire
- **Activation**: FIRE\_BOLT: bolt of fire for 9d8 (level 1, aimed); recharge 8+d8 turns

> Instead of a blade, a flame seems to emerge from this dagger's hilt.

### Dagger 'Nimthanc'

- **Base**: Dagger; **level** 1; **alloc** 1-50 (x40); **weight** 1.2 lb; **cost** 11000
- **Combat**: 1d4 (+9, +12) \[0, +10\]
- **Slays/brands**: brand cold x3
- **Resists**: resist cold
- **Activation**: COLD\_BOLT: bolt or beam of cold for 6d8 (level 1, aimed); recharge 7+d8 turns

> From the hilt of this dagger springs not so much a blade as an icicle.

### The Dagger of Rilia

- **Base**: Dagger; **level** 5; **alloc** 5-75 (x5); **weight** 1.2 lb; **cost** 15000
- **Combat**: 2d4 (+14, +13)
- **Slays/brands**: slay orcs x3, slay trolls x3, brand poison x3
- **Resists**: resist poison, resist disenchantment
- **Activation**: STINKING\_CLOUD: ball of poison (radius 3) for 12 (level 3, aimed); recharge 4+d4 turns

> A large stiletto dagger that glistens with odorless poison, to which the wearer seems oddly immune.

### Dagger 'Belangil'

- **Base**: Dagger; **level** 10; **alloc** 10-80 (x2); **weight** 1.2 lb; **cost** 40000
- **Combat**: 3d4 (+16, +14)
- **Mods**: DEX +2
- **Slays/brands**: brand cold x3
- **Resists**: resist cold
- **Flags**: SEE\_INVIS, SLOW\_DIGEST, REGEN
- **Activation**: COLD\_BALL50: ball of cold (radius 2) for 50 (level 10, aimed); recharge 5+d5 turns

> A frosty dagger wreathed in a nimbus of ice with a hilt of elk horn and an edge to wound the wind.

### Rapier 'Careth Asdriag'

- **Base**: Rapier; **level** 15; **alloc** 15-80 (x10); **weight** 4.0 lb; **cost** 25000
- **Combat**: 2d6 (+16, +12)
- **Mods**: BLOWS +1, STR +2, DEX +2, CON +2
- **Slays/brands**: slay dragons x3, slay animals x2, slay trolls x3, slay giants x3, slay orcs x3

> An heirloom of the dauntless Lords of Rhûn far to the east, and a name of dismay to creatures natural and unnatural.

### The Main Gauche of Maedhros

- **Base**: Main Gauche; **level** 15; **alloc** 15-65 (x3); **weight** 3.0 lb; **cost** 20000
- **Combat**: 2d6 (+12, +15)
- **Mods**: INT +3, DEX +3
- **Slays/brands**: slay trolls x3, slay giants x3
- **Flags**: FREE\_ACT, SEE\_INVIS

> A short thrusting blade with a large guard worn by the eldest son of Fëanor, Maedhros the Tall, who lost his right hand while escaping from Angband.

### The Main Gauche of Azaghâl

- **Base**: Main Gauche; **level** 18; **alloc** 18-100 (x3); **weight** 3.0 lb; **cost** 50000
- **Combat**: 2d5 (+12, +14)
- **Slays/brands**: slay dragons x5
- **Resists**: immune fire, resist acid
- **Flags**: PROT\_FEAR

> Azaghâl, Dwarf-king of Belegost, fell before Glaurung in the Battle of Unnumbered Tears. With his final breath, Azaghâl plunged this blade into Glaurung's belly with such a blow that the Father of Dragons fled the field with all his kin, and was at first believed to be mortally wounded.

### Broad Sword 'Aeglin'

- **Base**: Broad Sword; **level** 20; **alloc** 20-100 (x3); **weight** 15.0 lb; **cost** 45000
- **Combat**: 2d5 (+12, +16)
- **Mods**: SEARCH +1, LIGHT +1
- **Slays/brands**: slay orcs x3, slay trolls x3, slay giants x3, brand lightning x3
- **Resists**: resist lightning
- **Flags**: BLESSED, PROT\_BLIND, SLOW\_DIGEST

> Companion to Orcrist and Glamdring, and like them long lost, this sword is continually covered in tiny arcs of captive lightning that flash and dance eerily in the globes of light they create.

### Dagger 'Angrist'

- **Base**: Dagger; **level** 20; **alloc** 60-100 (x2); **weight** 1.2 lb; **cost** 100000
- **Combat**: 2d7 (+18, +16) \[0, +5\]
- **Mods**: DEX +4
- **Slays/brands**: slay evil creatures x2, slay trolls x3, slay orcs x3, brand acid x3
- **Resists**: resist acid, resist dark
- **Flags**: FREE\_ACT, SUST\_DEX

> Forged by Telchar, greatest of Dwarven smiths, and used by Beren to gouge a Silmaril out of Morgoth's crown, this long chopping dagger slices through ordinary metal as easily as its name, "Iron Cleaver", suggests.

### Long Sword 'Anguirel'

- **Base**: Long Sword; **level** 20; **alloc** 20-127 (x3); **weight** 13.0 lb; **cost** 40000
- **Combat**: 2d5 (+10, +12)
- **Mods**: SPEED +3, STR +2, CON +2, BLOWS +1, LIGHT +1
- **Slays/brands**: slay evil creatures x2, slay demons x3, brand poison x3
- **Resists**: resist lightning, resist light, resist dark
- **Flags**: AGGRAVATE, FREE\_ACT, SEE\_INVIS

> This black blade was forged of meteoric iron by Eöl the Dark Elf. It is imbued with the malice of its maker; as dangerous to its wielder as it is deadly to his foes.

### Broad Sword 'Aranrúth'

- **Base**: Broad Sword; **level** 20; **alloc** 20-100 (x2); **weight** 15.0 lb; **cost** 50000
- **Combat**: 3d5 (+20, +12)
- **Mods**: DEX +4
- **Slays/brands**: slay demons x3, slay orcs x3
- **Resists**: resist cold
- **Flags**: FREE\_ACT, FEATHER, SLOW\_DIGEST
- **Activation**: COLD\_BOLT2: bolt of cold for 12d8 (level 20, aimed); recharge 50 turns

> The beautiful sword of Thingol, justly named "King's Ire". It glistens icy enough to freeze the hearts of demons, and you feel supple and lightfooted as you clasp its hilt of gold and silver inlay.

### Short Sword 'Dagmor'

- **Base**: Short Sword; **level** 20; **alloc** 20-80 (x12); **weight** 8.0 lb; **cost** 15000
- **Combat**: 1d7 (+13, +11)
- **Mods**: BLOWS +2
- **Slays/brands**: slay animals x2, brand poison x3
- **Flags**: SLOW\_DIGEST, REGEN

> A stubby blade worn by Beren, whose horn sounded of old in the glades of Brethil.

### Broad Sword 'Glamdring'

- **Base**: Broad Sword; **level** 20; **alloc** 20-80 (x5); **weight** 15.0 lb; **cost** 40000
- **Combat**: 2d5 (+10, +15)
- **Mods**: SEARCH +1, LIGHT +1
- **Slays/brands**: slay orcs x3, slay demons x3, slay evil creatures x2, brand fire x3
- **Resists**: resist fire, resist light
- **Flags**: BLESSED, SLOW\_DIGEST

> This fiery, shining blade, mate to Orcrist, earned its sobriquet "Beater" from dying orcs who dared to behold hidden Gondolin.

### Cutlass 'Gondricam'

- **Base**: Cutlass; **level** 20; **alloc** 20-65 (x12); **weight** 11.0 lb; **cost** 28000
- **Combat**: 1d8 (+10, +11) \[0, +50\]
- **Mods**: DEX +4, STEALTH +2
- **Resists**: resist acid, resist lightning, resist fire, resist cold
- **Flags**: FEATHER, SEE\_INVIS, REGEN, FREE\_ACT

> Famed sea-defender of Lebennin. A short, slightly curved chopping blade with a perfect edge shining cleanly in the sun, an object of hate to the men of Umbar who met it in combat.

### Scimitar 'Haradekket'

- **Base**: Scimitar; **level** 20; **alloc** 20-80 (x7); **weight** 13.0 lb; **cost** 30000
- **Combat**: 4d2 (+9, +11)
- **Mods**: DEX +2, BLOWS +2
- **Slays/brands**: slay animals x2, slay evil creatures x2, slay undead x3
- **Flags**: SEE\_INVIS

> A damascened scimitar that hefts with wondrous ease. Famed in song as the "Sickle of Harad", and a deadly foe to the undead.

### Broad Sword 'Orcrist'

- **Base**: Broad Sword; **level** 20; **alloc** 20-80 (x5); **weight** 15.0 lb; **cost** 40000
- **Combat**: 2d5 (+10, +15)
- **Mods**: SEARCH +1, LIGHT +1
- **Slays/brands**: slay evil creatures x2, slay orcs x3, slay dragons x3, brand cold x3
- **Resists**: resist cold, resist dark
- **Flags**: BLESSED, SLOW\_DIGEST

> This coldly gleaming blade, mate to Glamdring, is called simply "Biter" by orcs who came to know its power all too well.

### Short Sword 'Sting'

- **Base**: Short Sword; **level** 20; **alloc** 20-100 (x6); **weight** 7.5 lb; **cost** 100000
- **Combat**: 1d7 (+7, +8)
- **Mods**: SPEED +3, BLOWS +2, STR +1, DEX +1, CON +1, LIGHT +1
- **Slays/brands**: slay evil creatures x2, slay undead x3, slay orcs x3, slay animals x2
- **Resists**: resist light
- **Flags**: PROT\_FEAR, FREE\_ACT, SEE\_INVIS

> "I will give you a name, and I shall call you Sting." The perfect size for Bilbo, and stamped forever by the courage he found in Mirkwood, this sturdy little blade grants the wearer combat prowess and survival abilities he did not know he had.

### Katana 'Aglarang'

- **Base**: Katana; **level** 30; **alloc** 30-100 (x4); **weight** 5.0 lb; **cost** 40000
- **Combat**: 6d6 (+10, +10)
- **Mods**: SPEED +5, DEX +5, BLOWS +1
- **Flags**: SUST\_DEX

> An utterly perfect, cleanly chiseled sword, with an edge that effortlessly slices rock and bone, and spells to render the wearer lithe and nimble. It is combat incarnate.

### Bastard Sword 'Calris'

- **Base**: Bastard Sword; **level** 30; **alloc** 30-127 (x7); **weight** 14.0 lb; **cost** 100000
- **Combat**: 3d7 (+0, +20)
- **Mods**: CON +5
- **Slays/brands**: slay dragons x5, slay evil creatures x2, slay demons x3, slay trolls x3
- **Resists**: resist disenchantment
- **Flags**: AGGRAVATE
- **Curses**: air swing (power 30)

> This sword has runes of power incused on its ornate hilt, and a single blood channel that gleams coldly blue as you grasp this mighty weapon of peril.

### Zweihander 'Gurthang'

- **Base**: Zweihander; **level** 30; **alloc** 30-100 (x3); **weight** 20.0 lb; **cost** 100000
- **Combat**: 3d6 (+13, +17)
- **Mods**: STR +2
- **Slays/brands**: slay dragons x5, brand poison x3, brand fire x3
- **Resists**: resist fire, resist poison
- **Flags**: FREE\_ACT, SLOW\_DIGEST, REGEN

> A giant sword once wielded by mighty Túrin, and an even greater dragonbane; for what worm could stand against the blade that bathed in Glaurung's blood?

### Zweihander 'Mormegil'

- **Base**: Zweihander; **level** 30; **alloc** 30-100 (x7); **weight** 25.0 lb; **cost** 10000
- **Combat**: 3d6 (-15, -15) \[0, -10\]
- **Mods**: SPEED -10
- **Slays/brands**: slay dragons x5, slay undead x3, brand poison x3
- **Flags**: SEE\_INVIS, HOLD\_LIFE, AGGRAVATE, DRAIN\_EXP

> A foul, twisted sword with blackened spines and knobs, its very name is a curse upon the lips of Elves and Men. It is said that it will drink the very lifeblood of its wielder.

### Zweihander 'Zarcuthra'

- **Base**: Zweihander; **level** 30; **alloc** 30-127 (x1); **weight** 24.0 lb; **cost** 200000
- **Combat**: 3d8 (+19, +21)
- **Mods**: STR +4, INFRA +2
- **Slays/brands**: slay dragons x5, slay animals x2, slay evil creatures x2, slay undead x3, slay demons x3, slay trolls x3, slay giants x3, slay orcs x3, brand fire x3
- **Resists**: resist fire, resist chaos
- **Flags**: FREE\_ACT, SEE\_INVIS, AGGRAVATE

> Runes dark and deadly stand stark against the naked steel of this awesome weapon, and you feel a wrathful power of slaying and rending as you slowly approach.

### The Bastard Sword of Éowyn

- **Base**: Bastard Sword; **level** 30; **alloc** 30-127 (x3); **weight** 14.0 lb; **cost** 120000
- **Combat**: 4d4 (+12, +16)
- **Mods**: STR +4, STEALTH +2
- **Slays/brands**: slay evil creatures x2, slay undead x5, slay giants x3, slay animals x2
- **Resists**: resist nether, resist dark, resist cold
- **Flags**: PROT\_FEAR

> The blade of the legendary Shieldmaiden of Rohan, who slew the foul steed of the Morgul-lord before striking down the mighty Witch-king himself, whom no mortal man could hinder.

### Long Sword 'Andúril'

- **Base**: Long Sword; **level** 35; **alloc** 20-127 (x3); **weight** 13.0 lb; **cost** 80000
- **Combat**: 3d5 (+10, +15) \[0, +10\]
- **Mods**: STR +4, DEX +4
- **Slays/brands**: slay evil creatures x2, slay trolls x3, slay orcs x3, slay undead x3, brand fire x3
- **Resists**: resist fire, resist disenchantment
- **Flags**: PROT\_FEAR, FREE\_ACT, BLESSED, SUST\_STR, SUST\_DEX, SEE\_INVIS
- **Activation**: FIRE\_BALL72: ball of fire (radius 2) for 72 (level 30, aimed); recharge 40 turns

> The famed "Flame of the West", the Sword that was Broken and is forged again. It glows with the essence of fire, its wearer is mighty in combat, and no creature of Sauron can withstand it.

### Executioner's Sword 'Crisdurian'

- **Base**: Executioner's Sword; **level** 40; **alloc** 40-127 (x4); **weight** 26.0 lb; **cost** 100000
- **Combat**: 4d6 (+18, +19)
- **Slays/brands**: slay dragons x3, slay evil creatures x2, slay undead x3, slay trolls x3, slay giants x3, slay orcs x3
- **Flags**: SEE\_INVIS

> A giant's weapon, with a long blade tall and straight thrusting out from a massive double-pronged hilt. On its blade are written doomspells against both the living and undead.

### Long Sword 'Ringil'

- **Base**: Long Sword; **level** 40; **alloc** 20-127 (x1); **weight** 13.0 lb; **cost** 300000
- **Combat**: 4d5 (+22, +25)
- **Mods**: SPEED +10, LIGHT +1
- **Slays/brands**: slay evil creatures x2, slay undead x3, slay trolls x3, slay demons x5, brand cold x3
- **Resists**: resist cold, resist light
- **Flags**: PROT\_FEAR, BLESSED, FREE\_ACT, SEE\_INVIS, SLOW\_DIGEST, REGEN
- **Activation**: COLD\_BALL100: ball of cold (radius 2) for 100 (level 35, aimed); recharge 40 turns

> The weapon of Fingolfin, High King of the Noldor; it shines like a column of ice lit by light unquenchable. Morgoth came but unwillingly to meet it of old; his lame foot will remind him of its might should he face it again.

### Blade of Chaos 'Doomcaller'

- **Base**: Blade of Chaos; **level** 70; **alloc** 70-127 (x4); **weight** 18.0 lb; **cost** 200000
- **Combat**: 4d7 (+18, +28) \[0, -50\]
- **Slays/brands**: slay animals x2, slay dragons x5, slay evil creatures x2, slay trolls x3, slay demons x3, brand cold x3
- **Resists**: resist acid, resist lightning, resist fire, resist cold, resist chaos
- **Flags**: FREE\_ACT, SEE\_INVIS, TELEPATHY, AGGRAVATE

> This weapon of wrath, wrought by a desperate berserker gang, dives hungrily into the yielding flesh of your enemies. You yourself fall under the shadow of death even as you inflict wounds that will not heal.

## Polearm (19)

### The Spear of Oromë

- **Base**: Spear; **level** 5; **alloc** 15-127 (x2); **weight** 5.0 lb; **cost** 60000
- **Combat**: 4d6 (+15, +15)
- **Mods**: INT +4, SPEED +4, INFRA +4, LIGHT +1
- **Slays/brands**: slay giants x3, slay animals x2, brand fire x3
- **Resists**: resist fire, resist light
- **Flags**: FEATHER, SEE\_INVIS, BLESSED
- **Activation**: STONE\_TO\_MUD: line of stone to mud for 20+1d30 (level 5, aimed); recharge 5 turns

> The thrusting spear of wise Oromë the Vala, strong against giants of frost, and able to pierce rock or flesh with ease.

### Spear 'Nimloth'

- **Base**: Spear; **level** 15; **alloc** 15-65 (x8); **weight** 5.0 lb; **cost** 30000
- **Combat**: 1d7 (+11, +13)
- **Mods**: STEALTH +3, SPEED +3
- **Slays/brands**: slay undead x3, brand cold x3
- **Resists**: resist cold
- **Flags**: BLESSED, SEE\_INVIS

> A thin spike of thrice-forged steel caps a straight silvan shaft cut from a legendary tree, and spells to break the wills of the undead and strike cold into the breasts of your enemies lie upon this perfectly balanced spear.

### The Trident of Wrath

- **Base**: Trident; **level** 15; **alloc** 15-100 (x3); **weight** 7.0 lb; **cost** 90000
- **Combat**: 3d10 (+16, +18)
- **Mods**: STR +2, DEX +2
- **Slays/brands**: slay evil creatures x2, slay undead x5, brand poison x3
- **Resists**: resist light, resist dark
- **Flags**: SEE\_INVIS, BLESSED

> A massive triple-pronged spear, so great it normally requires two hands to wield, evoking the spirit of Ossë who with it pierced legions of evil and undead.

### Scythe 'Avavir'

- **Base**: Scythe; **level** 20; **alloc** 45-100 (x12); **weight** 18.0 lb; **cost** 18000
- **Combat**: 5d3 (+18, +12) \[0, +30\]
- **Mods**: DEX +3, SPEED +3, LIGHT +1
- **Slays/brands**: brand fire x3, brand cold x3
- **Resists**: resist fire, resist cold, resist light
- **Flags**: FREE\_ACT, SEE\_INVIS
- **Activation**: RECALL: returns you from the dungeon or takes you to the dungeon after a short delay (level 20); recharge 200 turns

> Holding elemental powers whose struggles turn this weapon red and purest white, this shining reaper bears within it a power of going forth and returning.

### Broad Axe 'Barukkheled'

- **Base**: Broad Axe; **level** 20; **alloc** 20-75 (x12); **weight** 16.0 lb; **cost** 50000
- **Combat**: 2d6 (+13, +19)
- **Mods**: CON +3
- **Slays/brands**: slay evil creatures x2, slay trolls x3, slay giants x3, slay orcs x3
- **Flags**: SEE\_INVIS

> A royal heirloom of the southern coast, strong in combat against the evil creatures of earth.

### Halberd 'Osondir'

- **Base**: Halberd; **level** 20; **alloc** 25-80 (x12); **weight** 14.0 lb; **cost** 22000
- **Combat**: 3d7 (+16, +12)
- **Mods**: STR +3, WIS +3
- **Slays/brands**: slay undead x3, slay giants x3, brand fire x3
- **Resists**: resist fire, resist sound
- **Flags**: BLESSED, FEATHER, SEE\_INVIS

> Lordly and tall did Osondir stand against the wrath of giants, and clear-eyed in barrows fell, wielding a halberd glowing ruby red.

### Pike 'Til-i-arc'

- **Base**: Pike; **level** 20; **alloc** 20-80 (x7); **weight** 16.0 lb; **cost** 32000
- **Combat**: 1d15 (+10, +12) \[0, +10\]
- **Mods**: STR +3, INT +3, SPEED +2
- **Slays/brands**: slay demons x3, slay trolls x3, slay giants x3, brand fire x3, brand cold x3
- **Resists**: resist fire, resist cold
- **Flags**: SUST\_INT, SLOW\_DIGEST

> Within this long two-handed spear dwell the spirits of frost giants and fire demons, who war forever, fettered by sorcerous binds.

### The Beaked Axe of Húrin

- **Base**: Beaked Axe; **level** 20; **alloc** 20-100 (x7); **weight** 18.0 lb; **cost** 90000
- **Combat**: 3d6 (+12, +15)
- **Mods**: STR +2, CON +2, LIGHT +1
- **Slays/brands**: slay dragons x3, slay trolls x3, slay demons x5, brand acid x3
- **Resists**: resist acid, resist dark, resist fire
- **Activation**: BERSERKER: puts you in a berserker rage for d50+50 turns \[+50+1d50 turns of berserk rage\] (level 20); recharge 80+d80 turns

> Wielded by Húrin Thalion, last lord of Dor-Lómin at the Battle of Tears Unnumbered. He stood alone, his friends and kinsmen dead about him, his axe smoking in the black blood of Gothmog's troll-guard. No less than seventy times, as he slew his foes, did he utter his mighty war-cry: "Aurë entuluva!" - "Day shall come again!"

### The Lance of the Eorlingas

- **Base**: Lance; **level** 20; **alloc** 20-100 (x4); **weight** 36.0 lb; **cost** 55000
- **Combat**: 2d17 (+3, +21)
- **Mods**: STR +2, DEX +2, SPEED +2
- **Slays/brands**: slay evil creatures x2, slay trolls x3, slay orcs x3
- **Flags**: PROT\_FEAR, SEE\_INVIS

> "Forth Eorlingas!" To the field of Celebrant, Eorl the Young came beyond hope to save beleaguered Gondor, and from his lance fled massive trolls and dire wolves.

### The Trident of Ulmo

- **Base**: Trident; **level** 27; **alloc** 30-127 (x1); **weight** 7.0 lb; **cost** 120000
- **Combat**: 4d10 (+15, +19)
- **Mods**: DEX +4
- **Slays/brands**: slay dragons x3, slay animals x2
- **Resists**: immune acid, resist nether
- **Flags**: FREE\_ACT, HOLD\_LIFE, SEE\_INVIS, SLOW\_DIGEST, REGEN, BLESSED
- **Activation**: TELE\_OTHER: teleports a target monster away \[bolt that teleports monsters away (power (sight\*5))\] (level 25, aimed); recharge 50 turns

> The awesome weapon of the Vala Ulmo, Lord of Waters. Mightiest of all the powers of good save Manwë himself, Ulmo laughs to scorn the dread powers of the undead, and is utterly in command of the Element of water.

### Battle Axe 'Lotharang'

- **Base**: Battle Axe; **level** 30; **alloc** 30-100 (x7); **weight** 17.0 lb; **cost** 21000
- **Combat**: 2d8 (+14, +13)
- **Mods**: STR +3, DEX +3, BLOWS +1
- **Slays/brands**: slay trolls x3, slay orcs x3
- **Resists**: resist disenchantment
- **Activation**: CURE\_SERIOUS: heals 40 hitpoints, cut damage, and cures blindness and confusion \[heal 40 HP; cure blindness; cure wounds; cure confusion\] (level 25); recharge 3+d3 turns

> A superbly crafted double-bladed axe once borne by Ceruthain, commander of the Rangers of Arnor.

### Lochaber Axe 'Mundwine'

- **Base**: Lochaber Axe; **level** 30; **alloc** 30-100 (x12); **weight** 24.0 lb; **cost** 30000
- **Combat**: 3d10 (+12, +17)
- **Slays/brands**: slay evil creatures x2, slay animals x2, slay demons x3
- **Resists**: resist acid, resist lightning, resist fire, resist cold

> A massive axe with twin razor-sharp heads, so large that it usually requires two hands to wield, with spells to ward off the elements intricately engraved upon its surface in filleted gold.

### The Battle Axe of Balli Stonehand

- **Base**: Battle Axe; **level** 30; **alloc** 30-100 (x7); **weight** 17.0 lb; **cost** 90000
- **Combat**: 3d8 (+8, +11) \[0, +5\]
- **Mods**: STR +3, CON +3, STEALTH +1
- **Slays/brands**: slay demons x3, slay trolls x3, slay orcs x3
- **Resists**: resist acid, resist lightning, resist fire, resist cold
- **Flags**: FREE\_ACT, PROT\_BLIND, FEATHER, SEE\_INVIS, REGEN

> The twin blades of this weapon were forged in Belegost, and powerful forces to resist and endure lie ready for its next owner.

### The Great Axe of Durin

- **Base**: Great Axe; **level** 30; **alloc** 40-127 (x1); **weight** 23.0 lb; **cost** 150000
- **Combat**: 4d4 (+10, +20) \[0, +15\]
- **Mods**: TUNNEL +5, STR +3, CON +3, STEALTH -1
- **Slays/brands**: slay dragons x5, slay demons x3, slay trolls x3, slay orcs x3, brand acid x3, brand fire x3
- **Resists**: resist acid, resist fire, resist light, resist dark, resist chaos
- **Flags**: FREE\_ACT, PROT\_FEAR

> The twin massive axe heads of this ancient demon's dread gleam with mithril inlay telling sagas of endurance. The powers of Khazad-dûm protect its wielder and slay all evils found underground.

### The Great Axe of Eönwë

- **Base**: Great Axe; **level** 30; **alloc** 40-127 (x1); **weight** 23.0 lb; **cost** 200000
- **Combat**: 5d4 (+15, +18) \[0, +8\]
- **Mods**: STR +2, INT +2, WIS +2, DEX +2, CON +2
- **Slays/brands**: slay evil creatures x2, slay undead x3, slay demons x5, slay orcs x3, brand cold x3
- **Resists**: immune cold
- **Flags**: PROT\_FEAR, FREE\_ACT, SEE\_INVIS, BLESSED
- **Activation**: LOSKILL: removes all non-unique monsters within 20 squares, dealing you damage in the process \[removes all nearby creatures\] (level 40); recharge 1000 turns

> The axe of Eönwë, leader of the Hosts of the West in the War of Wrath, strikes with icy wrath at the undead, disperses hosts of evil at a word, and grants Maia-like powers of body and mind.

### The Glaive of Pain

- **Base**: Glaive; **level** 30; **alloc** 30-127 (x4); **weight** 30.0 lb; **cost** 50000
- **Combat**: 6d9 (+0, +30)

> The massive cleaver that crowns this glaive glows blood-red and black; fell spells of annihilation swirl and dance as you swing death's myrmidon down.

### The Beaked Axe of Théoden

- **Base**: Beaked Axe; **level** 30; **alloc** 20-100 (x7); **weight** 18.0 lb; **cost** 40000
- **Combat**: 2d6 (+8, +10)
- **Mods**: WIS +3, CON +3
- **Slays/brands**: slay dragons x3
- **Flags**: TELEPATHY, SLOW\_DIGEST
- **Activation**: DRAIN\_LIFE2: drains up to 120 hit points of life from a target creature. Does not affect undead, demons, golems, elementals or vortices \[bolt that damages living monsters for 120\] (level 30, aimed); recharge 40 turns

> The narrow axe head of this weapon would pierce the armour of a very Dragon, and the designs of your enemies stand naked and revealed.

### Spear 'Aiglos'

- **Base**: Spear; **level** 40; **alloc** 15-100 (x2); **weight** 5.0 lb; **cost** 140000
- **Combat**: 3d6 (+15, +25) \[0, +5\]
- **Mods**: DEX +4, WIS +2
- **Slays/brands**: slay evil creatures x2, slay trolls x3, slay orcs x3, slay undead x5, brand cold x3
- **Resists**: resist cold
- **Flags**: PROT\_FEAR, FREE\_ACT, SLOW\_DIGEST, BLESSED
- **Activation**: COLD\_BALL100: ball of cold (radius 2) for 100 (level 35, aimed); recharge 35 turns

> The mighty spear of Gil-galad, famed as "Snow-point" in the songs of Elves. Against it, all the foul corruptions of Sauron dashed in vain.

### The Spear of Melkor

- **Base**: Spear; **level** 65; **alloc** 65-127 (x2); **weight** 20.0 lb; **cost** 100000
- **Combat**: 4d6 (-12, +20)
- **Mods**: STEALTH -4, WIS -4
- **Slays/brands**: brand poison x3
- **Resists**: resist dark, resist light, resist nether
- **Flags**: DRAIN\_EXP, AGGRAVATE, PROT\_BLIND

> A mighty, heavy black spear, once wielded by the Lord of Darkness himself as he came to Valinor in wrath. Its envenomed bite pierced the Two Trees with wounds that would not heal.

## Hafted (14)

### Quarterstaff 'Eriril'

- **Base**: Quarterstaff; **level** 15; **alloc** 20-100 (x5); **weight** 15.0 lb; **cost** 20000
- **Combat**: 1d9 (+13, +15)
- **Mods**: INT +4, WIS +4, LIGHT +1
- **Slays/brands**: slay evil creatures x2
- **Resists**: resist light
- **Flags**: SEE\_INVIS
- **Activation**: DETECT\_OBJECTS: detects all objects in the immediate area \[detects objects nearby (area 22 x 40)\] (level 15); recharge 50+d50 turns

> The radiant golden staff of an Istar of legend, this wizard's companion grants keen sight and clear knowledge of many hidden things.

### Morning Star 'Bloodspike'

- **Base**: Morning Star; **level** 20; **alloc** 20-75 (x3); **weight** 15.0 lb; **cost** 30000
- **Combat**: 2d6 (+8, +22)
- **Mods**: STR +4
- **Slays/brands**: slay animals x2, slay trolls x3, slay orcs x3, brand poison x3
- **Resists**: resist nexus
- **Flags**: SEE\_INVIS

> You feel strong and firm of foot as you whip its spiked orb around and bathe it in the blood of your foes.

### Quarterstaff 'Nar-i-vagil'

- **Base**: Quarterstaff; **level** 20; **alloc** 20-80 (x5); **weight** 15.0 lb; **cost** 70000
- **Combat**: 1d9 (+10, +20)
- **Mods**: INT +4
- **Slays/brands**: slay animals x2, brand fire x3
- **Resists**: resist fire, resist poison

> Named for a fiery star and set with priceless gems bearing mystic auras of protection and thought.

### Flail 'Totila'

- **Base**: Flail; **level** 20; **alloc** 20-65 (x12); **weight** 15.0 lb; **cost** 55000
- **Combat**: 3d6 (+16, +12)
- **Mods**: STEALTH +2, SPEED +2
- **Slays/brands**: slay evil creatures x2, brand fire x3
- **Resists**: resist fire
- **Flags**: PROT\_CONF
- **Activation**: CONFUSE2: confuses a target monster \[bolt that attempts to confuse monsters (power 20)\] (level 20, aimed); recharge 15 turns

> A flail whose head befuddles those who stare as you whirl it round and becomes a fiery comet as you bring it down.

### Lead-Filled Mace 'Turmil'

- **Base**: Lead-Filled Mace; **level** 20; **alloc** 20-100 (x7); **weight** 12.0 lb; **cost** 30000
- **Combat**: 4d4 (+10, +11) \[0, +8\]
- **Mods**: WIS +4, INFRA +4, LIGHT +1
- **Slays/brands**: slay orcs x3, brand cold x3
- **Resists**: immune cold, resist light
- **Flags**: BLESSED, REGEN
- **Activation**: DRAIN\_LIFE1: drains up to 90 hit points of life from a target creature. Does not affect undead, demons, golems, elementals or vortices \[bolt that damages living monsters for 90\] (level 20, aimed); recharge 40 turns

> Once wielded by the High Priest of Meneltarma, this great mace gleams coldly as though moonlit, and can strike equally mighty spiritual and physical blows.

### The Quarterstaff of Olórin

- **Base**: Quarterstaff; **level** 20; **alloc** 30-100 (x1); **weight** 15.0 lb; **cost** 130000
- **Combat**: 2d9 (+10, +13)
- **Mods**: INT +4, WIS +4
- **Slays/brands**: slay demons x5, slay evil creatures x2, slay trolls x3, slay orcs x3, brand fire x3
- **Resists**: resist fire, resist nether
- **Flags**: HOLD\_LIFE, SEE\_INVIS
- **Activation**: PROBING: gives you information on the health and abilities of monsters you can see (level 20); recharge 20 turns

> A staff tall and sturdy, borne of old by Gandalf. Rough-hewn runes that invoke the element of Earth stand bold upon its gnarled surface, and no creature who lives in the shadow of mountains can withstand the blows it deals.

### Morning Star 'Firestar'

- **Base**: Morning Star; **level** 25; **alloc** 20-100 (x7); **weight** 15.0 lb; **cost** 35000
- **Combat**: 2d6 (+15, +17) \[0, +2\]
- **Slays/brands**: brand fire x3
- **Resists**: immune fire
- **Activation**: FIRE\_BALL72: ball of fire (radius 2) for 72 (level 30, aimed); recharge 20 turns

> A famed battle-lord of old with a head as ruddy as embers that can yet rise up in wrath.

### Mace 'Taratol'

- **Base**: Mace; **level** 35; **alloc** 20-100 (x7); **weight** 20.0 lb; **cost** 50000
- **Combat**: 3d6 (+12, +12)
- **Slays/brands**: slay dragons x5, brand lightning x3
- **Resists**: immune lightning
- **Activation**: HASTE1: hastens you for d20+20 turns \[+20+1d20 turns of haste\] (level 40); recharge 100+d100 turns

> A great ridged mace that surrounds you with a nimbus of living lightning. You remain utterly untouched, even as fat sparks crackle around your fingers and eyebrows.

### The Great Hammer of Aulë

- **Base**: Great Hammer; **level** 40; **alloc** 40-127 (x1); **weight** 12.0 lb; **cost** 250000
- **Combat**: 18d1 (+19, +21) \[0, +5\]
- **Mods**: TUNNEL +8, WIS +4
- **Slays/brands**: slay dragons x5, slay evil creatures x2, slay undead x3, slay demons x3, brand acid x3
- **Resists**: resist acid, resist lightning, resist fire, resist cold, resist nexus
- **Flags**: PROT\_FEAR, FREE\_ACT, SEE\_INVIS

> The wondrous hammer of Aulë, creator of the wise Dwarven lords of old. It bears demolishing magics that no serpent or demon can withstand, and invokes the strength of mountains to ward off the tumult of the elements.

### Two-Handed Great Flail 'Thunderfist'

- **Base**: Two-Handed Great Flail; **level** 45; **alloc** 45-127 (x3); **weight** 30.0 lb; **cost** 160000
- **Combat**: 3d9 (+5, +18)
- **Mods**: STR +4, CON +3
- **Slays/brands**: slay animals x2, slay trolls x3, slay orcs x3, brand lightning x3, brand fire x3
- **Resists**: resist lightning, resist fire, resist dark
- **Flags**: PROT\_FEAR

> The long-lost weapon of Kzurin, Dwarven champion of ancient Belegost. Runes of strength adorn its handle, and flames and sparks roar and crackle around its massive head.

### The Whip of Gothmog

- **Base**: Whip; **level** 60; **alloc** 60-127 (x4); **weight** 9.0 lb; **cost** 25000
- **Combat**: 6d3 (+13, +15)
- **Mods**: STR +3, INT -3, DEX -3, WIS -3, LIGHT +1
- **Slays/brands**: slay animals x2, slay dragons x5, slay trolls x3, slay giants x3, brand fire x3
- **Resists**: immune fire, resist lightning, resist dark
- **Flags**: AGGRAVATE
- **Activation**: FIRE\_BALL: ball of fire (radius 2) for 144 (level 50, aimed); recharge 15 turns

> A great crackling Balrog's whip of flame, this weapon could bring even Ungoliant to heel.

### The Ball-and-Chain of Fundin Bluecloak

- **Base**: Ball-and-Chain; **level** 65; **alloc** 65-127 (x2); **weight** 13.0 lb; **cost** 60000
- **Combat**: 4d4 (+13, +17) \[0, +10\]
- **Mods**: STR +4, WIS +4, LIGHT +1
- **Slays/brands**: slay evil creatures x2, slay undead x3
- **Resists**: resist fire, resist lightning, resist nether
- **Flags**: HOLD\_LIFE
- **Activation**: DISPEL\_EVIL: deals five times your level's damage to all evil creatures that you can see \[damages evil monsters in line of sight (power (plev\*5))\] (level 65); recharge 100+d100 turns

> The weapon of a mighty dwarven priest, whose shade is rumoured to haunt the halls of Angband still.

### Mace of Disruption 'Deathwreaker'

- **Base**: Mace of Disruption; **level** 80; **alloc** 80-127 (x3); **weight** 40.0 lb; **cost** 400000
- **Combat**: 7d8 (+18, +18)
- **Mods**: STR +6, TUNNEL +4
- **Slays/brands**: slay dragons x3, slay animals x2, slay evil creatures x2, slay undead x5, brand fire x3
- **Resists**: immune fire, resist dark, resist chaos, resist disenchantment
- **Flags**: AGGRAVATE

> A weapon so massive it seems beyond the strength of mortals, yet you feel the might of giants within you as you heft it. As you grip the handle of ebony and steel, coronas of fire blaze, and mighty spells to preserve magic surge around you. You wield the Fear of Dragons and the Despair of the Undead!

### Mighty Hammer 'Grond'

- **Base**: Mighty Hammer; **level** 100; **alloc** 100-100 (x0); **weight** 100.0 lb; **cost** 500000
- **Combat**: 9d9 (+5, +25) \[0, +10\]
- **Slays/brands**: slay dragons x5, slay animals x2, slay evil creatures x2, slay undead x5, slay demons x5, slay trolls x3, slay orcs x3
- **Flags**: IMPACT, SEE\_INVIS, TELEPATHY, AGGRAVATE

> The mighty Hammer of the Underworld, blackened by doomspells of shattering. Its wielder holds the lives of all Morgoth's servants in his hand.

## Digger (2)

### The Mattock of Náin

- **Base**: Mattock; **level** 10; **alloc** 60-100 (x12); **weight** 24.0 lb; **cost** 30000
- **Combat**: 2d8 (+12, +18)
- **Mods**: TUNNEL +6, SEARCH +4, INFRA +4, STR +4
- **Slays/brands**: slay orcs x3, slay trolls x3, slay giants x3, slay dragons x3
- **Resists**: resist dark, resist disenchantment
- **Activation**: STONE\_TO\_MUD: line of stone to mud for 20+1d30 (level 5, aimed); recharge 2 turns

> A mighty Dwarven mattock, as good for hewing orc-necks as for hewing stone. Náin, King of the Iron Hills, brought his mattock-wielding warriors to the Battle of Azanulbizar, and no Orc could withstand them, although Náin himself fell before their lord Azog.

### The Pick of Erebor

- **Base**: Pick; **level** 55; **alloc** 55-100 (x20); **weight** 18.0 lb; **cost** 30000
- **Combat**: 4d3 (+5, +20)
- **Mods**: TUNNEL +5, STR +3, CON +3, LIGHT +1
- **Slays/brands**: slay orcs x3, slay trolls x3, slay demons x3, brand acid x3
- **Resists**: resist chaos, resist light, resist dark
- **Flags**: SUST\_STR

> A fine dwarven pick, good for a miner, and suitable as a weapon should the wielder ever tunnel through into a foe's stronghold.

## Bow (6)

### The Short Bow of Amras

- **Base**: Short Bow; **level** 25; **alloc** 30-80 (x10); **weight** 3.0 lb; **cost** 9000
- **Combat**: x2 (+12, +15)
- **Mods**: INT +2, WIS +2, DEX +2, SHOTS +5, MIGHT +1, SPEED +1
- **Resists**: resist fire, resist lightning, resist cold
- **Flags**: SLOW\_DIGEST

> One of a pair of hunting bows once belonging to Fëanor's two youngest sons. The bows are like and yet unlike, much as the twin brothers were.

### The Short Bow of Amrod

- **Base**: Short Bow; **level** 25; **alloc** 30-100 (x10); **weight** 3.0 lb; **cost** 9000
- **Combat**: x2 (+12, +15)
- **Mods**: STR +2, CON +2, MIGHT +2
- **Resists**: resist fire, resist lightning, resist cold
- **Flags**: REGEN

> One of a pair of hunting bows once belonging to Fëanor's two youngest sons. The bows are like and yet unlike, much as the twin brothers were.

### The Long Bow of Bard

- **Base**: Long Bow; **level** 30; **alloc** 40-127 (x5); **weight** 4.0 lb; **cost** 20000
- **Combat**: x3 (+17, +19)
- **Mods**: MIGHT +2, DEX +1, SPEED +1
- **Flags**: FREE\_ACT

> The great yew bow of grim-faced Bard, who shot the mightiest arrow that songs record.

### Long Bow 'Belthronding'

- **Base**: Long Bow; **level** 40; **alloc** 40-127 (x5); **weight** 4.0 lb; **cost** 35000
- **Combat**: x3 (+20, +22)
- **Mods**: DEX +3, SPEED +1, STEALTH +1, SHOTS +10
- **Resists**: resist disenchantment

> The great bow of Beleg Cúthalion, the most famous archer of the Elves. Its backbone of black yew holds strings of bear sinew, and of old, only Beleg himself could draw them.

### The Heavy Crossbow of Umbar

- **Base**: Heavy Crossbow; **level** 45; **alloc** 60-127 (x5); **weight** 20.0 lb; **cost** 35000
- **Combat**: x4 (+18, +18)
- **Mods**: MIGHT +2, STR +1, CON +1
- **Resists**: resist light, resist dark, resist lightning
- **Flags**: AGGRAVATE, PROT\_BLIND
- **Activation**: ARROW: bolt of arrows for 150 (level 38, aimed); recharge 20+d20 turns

> A great brazen arbalest with arms of gleaming steel. It shoots quarrels with mighty power for those brave enough to risk betrayal.

### Light Crossbow 'Cubragol'

- **Base**: Light Crossbow; **level** 50; **alloc** 50-127 (x4); **weight** 11.0 lb; **cost** 50000
- **Combat**: x3 (+10, +14)
- **Mods**: SPEED +10
- **Slays/brands**: brand fire x3
- **Resists**: resist fire

> A crossbow that grants fiery speed to its finder, and imbues its bolts with flame unquenchable.

## Soft Armor (3)

### Soft Leather Armour 'Hithlomir'

- **Base**: Soft Leather Armour; **level** 20; **alloc** 20-80 (x20); **weight** 8.0 lb; **cost** 45000
- **Combat**: \[10, +20\]
- **Mods**: STEALTH +3
- **Resists**: resist dark, resist nether, resist acid, resist lightning, resist fire, resist cold

> Familiar with the secret ways hidden in darkness, this leather cuirass is truly more than it appears.

### Leather Scale Mail 'Thalkettoth'

- **Base**: Leather Scale Mail; **level** 20; **alloc** 30-100 (x20); **weight** 6.0 lb; **cost** 25000
- **Combat**: (-1, +0) \[20, +25\]
- **Mods**: DEX +3, SPEED +3
- **Resists**: resist acid, resist shards
- **Flags**: FREE\_ACT

> An amazingly light tunic and skirt sewn with thick, overlapping scales of hardened leather. Those who wear it move with agility and assurance.

### The Hard Leather Armour of Himring

- **Base**: Hard Leather Armour; **level** 30; **alloc** 25-90 (x20); **weight** 10.0 lb; **cost** 35000
- **Combat**: \[20, +15\]
- **Resists**: resist chaos, resist nether, resist poison
- **Activation**: PROTEVIL: grants you protection from evil for 1d25 plus 3 times your character level turns \[+(plev\*3)+1d25 turns of protection from evil\] (level 40); recharge 100+d100 turns

> In this studded cuirass of pliable leather lives the memory of unvanquished Himring, defiant fortress surrounded by the legions of Morgoth.

## Hard Armor (7)

### Mithril Chain Mail 'Belegennon'

- **Base**: Mithril Chain Mail; **level** 20; **alloc** 55-127 (x30); **weight** 15.0 lb; **cost** 105000
- **Combat**: 1d4 (-1, +0) \[35, +20\]
- **Mods**: DAM\_RED +5
- **Resists**: resist acid, resist lightning, resist fire, resist cold, resist poison
- **Activation**: TELE\_PHASE: teleports you randomly up to 10 squares away \[teleport self up to 10 grids\] (level 8); recharge 2 turns

> This wondrous shirt of fine-linked chain shimmers before your eyes like light upon a rippling sea, the gems on it glittering like stars, the sound of the rings when shaken like the tinkle of rain in a pool. It stands untouched amidst the fury of the elements, and blows seem strangely less powerful.

### The Chain Mail of Arvedui

- **Base**: Chain Mail; **level** 20; **alloc** 20-75 (x30); **weight** 22.0 lb; **cost** 32000
- **Combat**: 1d4 (-2, +0) \[40, +15\]
- **Mods**: STR +2
- **Resists**: resist shards, resist nexus, resist acid, resist lightning, resist fire, resist cold

> A hauberk, leggings, and sleeves of interlocking steel rings, well padded with leather. You feel as strong and tall as Arvedui, last king of Arnor, as you put it on.

### The Augmented Chain Mail of Caspanion

- **Base**: Augmented Chain Mail; **level** 25; **alloc** 35-100 (x11); **weight** 27.0 lb; **cost** 40000
- **Combat**: 1d4 (-2, +0) \[45, +20\]
- **Mods**: INT +2, WIS +2, CON +3
- **Resists**: resist acid, resist poison
- **Flags**: PROT\_CONF
- **Activation**: DESTROY\_DOORS: destroys all doors surrounding you \[destroys doors on adjacent grids (radius 1)\] (level 5); recharge 10 turns

> A hauberk, leggings, and sleeves of interlocking steel rings, strategically reinforced at vital locations with a second layer of chain. Magics to enhance body and mind lie within, and no door can bar the wearer's path.

### The Full Plate Armour of Isildur

- **Base**: Full Plate Armour; **level** 30; **alloc** 55-127 (x30); **weight** 30.0 lb; **cost** 50000
- **Combat**: 2d4 (+0, +0) \[75, +25\]
- **Mods**: CON +1
- **Resists**: resist acid, resist lightning, resist fire, resist cold, resist sound, resist nexus
- **Flags**: PROT\_CONF

> A gleaming steel suit covering the wearer from neck to foot, with runes of warding and stability deeply engraved into its surface.

### The Metal Brigandine Armour of the Rohirrim

- **Base**: Metal Brigandine Armour; **level** 30; **alloc** 45-100 (x30); **weight** 20.0 lb; **cost** 30000
- **Combat**: 1d4 (+0, +0) \[50, +15\]
- **Mods**: STR +2, DEX +2
- **Resists**: resist acid, resist lightning, resist fire, resist cold, resist sound
- **Flags**: PROT\_FEAR, PROT\_CONF

> Small steel plates cover an inner layer of sturdy canvas, and both metal and cloth bear scenes of hunting and war. As you don the armour of Eorl the Young, matchless in combat, you feel his spirit surround you.

### The Mithril Plate Mail of Celeborn

- **Base**: Mithril Plate Mail; **level** 40; **alloc** 65-127 (x30); **weight** 25.0 lb; **cost** 150000
- **Combat**: 2d4 (-3, +0) \[65, +25\]
- **Mods**: STR +3
- **Resists**: resist acid, resist lightning, resist fire, resist cold, resist dark, resist disenchantment
- **Activation**: BANISHMENT: removes all non-unique monsters represented by a chosen symbol from the level, dealing you damage in the process \[removes all of a given creature type from the level\] (level 50); recharge 500 turns

> A shimmering suit of true-silver, forged long ago by dwarven smiths of legend. It gleams with purest white as you gaze upon it, and mighty are its powers to protect and banish.

### Adamantite Plate Mail 'Soulkeeper'

- **Base**: Adamantite Plate Mail; **level** 75; **alloc** 75-127 (x11); **weight** 42.0 lb; **cost** 300000
- **Combat**: 2d4 (-4, +0) \[90, +20\]
- **Mods**: CON +2
- **Resists**: resist acid, resist cold, resist dark, resist nether, resist nexus, resist chaos
- **Flags**: HOLD\_LIFE, SUST\_CON, PROT\_FEAR
- **Activation**: HEAL2: heals 1000 hit points \[heal 1000 HP; cure wounds\] (level 80); recharge 444 turns

> A suit of imperishable adamant with unconquerable strength to endure evil and disruptive magics. It protects the life force of its wearer as nothing else can.

## Dragon Armor (3)

### Power Dragon Scale Mail 'Bladeturner'

- **Base**: Power Dragon Scale Mail; **level** 55; **alloc** 100-127 (x6); **weight** 25.0 lb; **cost** 500000
- **Combat**: 2d4 (-8, +0) \[80, +35\]
- **Resists**: resist acid, resist lightning, resist fire, resist cold, resist poison, resist light, resist dark, resist sound, resist shards, resist nether, resist nexus, resist chaos, resist disenchantment
- **Flags**: HOLD\_LIFE, REGEN, PROT\_FEAR, PROT\_BLIND, PROT\_CONF
- **Activation**: RAGE\_BLESS\_RESIST: bestows upon you berserk rage, bless, and resistance \[heal 30 HP; cure fear; +50+1d50 turns of berserk rage; +50+1d50 turns of your AC and to-hit bonus; +50+1d50 turns of acid resistance; +50+1d50 turns of electricity resistance; +50+1d50 turns of fire resistance; +50+1d50 turns of cold resistance; +50+1d50 turns of poison resistance\] (level 50); recharge 50 turns

> An adamant suit with scales of every color. Untrammelled powers elemental and ethereal swirl around it in an inextricably intermingled nimbus.

### Balance Dragon Scale Mail 'Mediator'

- **Base**: Balance Dragon Scale Mail; **level** 60; **alloc** 95-127 (x8); **weight** 25.0 lb; **cost** 400000
- **Combat**: 2d4 (-4, +0) \[60, +25\]
- **Resists**: resist chaos, resist disenchantment, resist shards, resist sound, resist nexus
- **Flags**: FREE\_ACT, SLOW\_DIGEST, REGEN
- **Activation**: STAR\_BALL: balls of lightning (radius 3) in all 8 directions for 150 (level 60); recharge 50 turns

> A great suit of dragon hide, set with scales of many hues. Even the mightiest wyrms of Law and Chaos fear the judgement of its wearer.

### Multi-Hued Dragon Scale Mail 'Razorback'

- **Base**: Multi-Hued Dragon Scale Mail; **level** 60; **alloc** 90-127 (x11); **weight** 20.0 lb; **cost** 400000
- **Combat**: 2d4 (-4, +0) \[50, +25\]
- **Mods**: LIGHT +1
- **Resists**: immune lightning, resist disenchantment, resist acid, resist fire, resist cold, resist poison, resist light, resist dark
- **Flags**: FREE\_ACT, SEE\_INVIS, AGGRAVATE
- **Activation**: STAR\_BALL: balls of lightning (radius 3) in all 8 directions for 150 (level 60); recharge 50 turns

> A massive suit of heavy dragon scales deeply saturated with many colors. It throbs with angry energies, and you feel the raw elemental might of untamed Lightning as you put it on.

## Shield (7)

### The Leather Shield of Celegorm

- **Base**: Leather Shield; **level** 30; **alloc** 30-100 (x28); **weight** 6.0 lb; **cost** 12000
- **Combat**: 1d3 (+0, +0) \[8, +20\]
- **Resists**: resist acid, resist lightning, resist fire, resist cold, resist light, resist dark
- **Flags**: PROT\_BLIND

> This shield emblazoned with a multitude of creatures unseen for ages once protected Celegorm, Lord of Himlad; around it, a mystic balance lies, containing the conflicts of the elements.

### The Mithril Shield of Gil-galad

- **Base**: Mithril Shield; **level** 30; **alloc** 70-127 (x25); **weight** 8.0 lb; **cost** 65000
- **Combat**: 1d4 (+0, +0) \[20, +20\]
- **Mods**: WIS +5, LIGHT +1
- **Resists**: resist lightning, resist acid, resist disenchantment, resist dark
- **Flags**: SUST\_WIS, SUST\_DEX
- **Activation**: STARLIGHT2: beams of light in all 8 directions for 10d8 (level 30); recharge 100 turns

> The legendary shield of Ereinion Gil-galad, who fought his way to the gates of the Dark Tower, and with whom came light even to Gorgoroth.

### The Knight's Shield of Eärendil

- **Base**: Knight's Shield; **level** 32; **alloc** 65-127 (x6); **weight** 16.0 lb; **cost** 90000
- **Combat**: 1d6 (+0, +0) \[10, +15\]
- **Mods**: LIGHT +1
- **Resists**: resist dark, resist nether, resist lightning, resist fire
- **Flags**: PROT\_BLIND
- **Activation**: CURE\_TEMP: heals cut damage, and cures all stunning, poison, blindness and confusion \[cure blindness; cure wounds; cure poisoning; cure stunning; cure confusion\] (level 27); recharge 100 turns

> A shining shield wrought for Eärendil Half-Elven, son of Tuor and Idril, "scored with runes to keep all wounds and harm from him".

### The Leather Shield of the Haradrim

- **Base**: Leather Shield; **level** 35; **alloc** 35-127 (x10); **weight** 9.0 lb; **cost** 25000
- **Combat**: 1d3 (+5, +5) \[9, +15\]
- **Mods**: STR +2, CON +2, STEALTH -2
- **Resists**: resist poison
- **Flags**: SUST\_STR, SUST\_CON, PROT\_FEAR, PROT\_BLIND
- **Activation**: BERSERKER: puts you in a berserker rage for d50+50 turns \[+50+1d50 turns of berserk rage\] (level 20); recharge 50 turns

> The painted shield of a chieftain of Far Harad, gaudy and barbaric. Its wielder will fear nothing and fight with unnatural strength, but also draw attention to himself.

### The Large Metal Shield of Anárion

- **Base**: Large Metal Shield; **level** 40; **alloc** 40-100 (x11); **weight** 12.0 lb; **cost** 160000
- **Combat**: 1d5 (+0, +0) \[12, +20\]
- **Resists**: resist acid, resist lightning, resist fire, resist cold
- **Flags**: SUST\_STR, SUST\_INT, SUST\_WIS, SUST\_DEX, SUST\_CON

> The great metal-bound shield of Elendil's younger son Anárion, whom even Sauron could not corrupt.

### The Wicker Shield of Elros

- **Base**: Wicker Shield; **level** 40; **alloc** 50-127 (x10); **weight** 3.0 lb; **cost** 60000
- **Combat**: \[2, +20\]
- **Mods**: CON +4, WIS +1, INT +1
- **Resists**: resist lightning, resist shards
- **Flags**: PROT\_CONF, HOLD\_LIFE

> The shield of Elros Tar-Minyatur, founder of Númenor. Granted long life by the Valar, even his shield became imbued with power.

### The Small Metal Shield of Thorin

- **Base**: Small Metal Shield; **level** 40; **alloc** 40-127 (x14); **weight** 6.5 lb; **cost** 60000
- **Combat**: 1d2 (+0, +0) \[5, +25\]
- **Mods**: CON +4, STR +3, STEALTH -1
- **Resists**: resist sound, immune acid, resist chaos
- **Flags**: PROT\_FEAR, FREE\_ACT

> Invoking the strength and endurance of Thorin, King under the Mountain, this little metal shield is proof against the Element of Earth.

## Cloak (7)

### The Cloak of Thorongil

- **Base**: Cloak; **level** 5; **alloc** 5-75 (x10); **weight** 1.0 lb; **cost** 8000
- **Combat**: \[1, +10\]
- **Resists**: resist acid
- **Flags**: FREE\_ACT, PROT\_FEAR, SEE\_INVIS

> This shimmering cloak of greens and browns was once worn by the young Aragorn, a keen-eyed captain of Rohan and Gondor, valiant on both land and sea.

### Cloak 'Colluin'

- **Base**: Cloak; **level** 12; **alloc** 5-100 (x2); **weight** 1.0 lb; **cost** 50000
- **Combat**: \[1, +15\]
- **Resists**: resist acid, resist lightning, resist fire, resist cold, resist poison
- **Activation**: RESIST\_ALL: grants temporary resistance to acid, electricity, fire, cold and poison for 1d20+20 turns \[+20+1d20 turns of acid resistance; +20+1d20 turns of electricity resistance; +20+1d20 turns of fire resistance; +20+1d20 turns of cold resistance; +20+1d20 turns of poison resistance\] (level 15); recharge 111 turns

> A cape worn by a hero from Valinor, a land utterly beyond the strife of Elements.

### Cloak 'Holcolleth'

- **Base**: Cloak; **level** 13; **alloc** 5-100 (x5); **weight** 1.0 lb; **cost** 18000
- **Combat**: \[1, +4\]
- **Mods**: STEALTH +3, INT +2, WIS +2
- **Resists**: resist acid
- **Activation**: SLEEPII: puts to sleep the monsters around you \[attempts to put all monsters to sleep on adjacent grids (radius 1) (power (plev\*10+500))\] (level 12); recharge 55 turns

> This elven-grey mantle possesses great powers of tranquility and concealment, and grants the wearer the knowledge and understanding of the Sindar.

### Cloak 'Colannon'

- **Base**: Cloak; **level** 26; **alloc** 5-100 (x5); **weight** 1.0 lb; **cost** 20000
- **Combat**: \[1, +15\]
- **Mods**: STEALTH +2, SPEED +2
- **Resists**: resist nexus
- **Activation**: TELE\_LONG: teleports you randomly about 60 per cent of the largest distance you could go \[teleport self 60% of the level's larger dimension\] (level 25); recharge 45 turns

> A crystal-blue cape of fine silk worn by a silent messenger of the forces of Law. Seldom will its wearer be diverted from his mission.

### The Elven Cloak of Lúthien

- **Base**: Elven Cloak; **level** 40; **alloc** 40-127 (x2); **weight** 0.5 lb; **cost** 45000
- **Combat**: \[6, +20\]
- **Mods**: INT +4, WIS +4, STEALTH +3, SPEED +2
- **Resists**: resist acid, resist fire, resist cold
- **Activation**: RESTORE\_LIFE: restores your experience to full \[restores your experience\] (level 40); recharge 250 turns

> The opaque midnight folds of this cloak, inlaid with a multitude of tiny diamonds, swirl around you, and you feel a hint, a fragment of the restorative magics of Lúthien, the most beautiful being who ever knew death.

### The Ethereal Cloak of Tuor

- **Base**: Ethereal Cloak; **level** 40; **alloc** 70-127 (x2); **weight** 0.0 lb; **cost** 35000
- **Combat**: \[6, +12\]
- **Mods**: STEALTH +4, DEX +4
- **Resists**: immune acid
- **Flags**: FREE\_ACT, SEE\_INVIS

> The great cloak given to Tuor by the Dweller in the Deep. As you wrap yourself from head to foot the words of Ulmo echo: "Thus thou shalt walk under my shadow".

### The Cloak of Thingol

- **Base**: Cloak; **level** 62; **alloc** 5-100 (x5); **weight** 1.0 lb; **cost** 35000
- **Combat**: \[1, +18\]
- **Mods**: DEX +3
- **Resists**: resist acid, resist fire, resist cold
- **Flags**: FREE\_ACT
- **Activation**: RECHARGE: tries to recharge a wand or staff, destroying the wand or staff on failure \[recharge a wand/staff (power 10)\] (level 60); recharge 70 turns

> A translucent cloak of pearly grey with glowing, energized Elven runes showing calm and clear as moonlight on still water.

## Gloves (9)

### Set of Gauntlets 'Pauraegen'

- **Base**: Set of Gauntlets; **level** 1; **alloc** 10-75 (x23); **weight** 2.5 lb; **cost** 11000
- **Combat**: \[3, +14\]
- **Mods**: LIGHT +1
- **Resists**: resist lightning, resist light
- **Activation**: ELEC\_BOLT: beam of lightning for 6d6 (level 1, aimed); recharge 6+d6 turns

> A set of handgear with sparks that crackle across its knuckleguards.

### Set of Gauntlets 'Paurhach'

- **Base**: Set of Gauntlets; **level** 1; **alloc** 10-60 (x40); **weight** 2.5 lb; **cost** 15000
- **Combat**: \[3, +14\]
- **Resists**: resist fire
- **Flags**: REGEN
- **Activation**: FIRE\_BOLT: bolt of fire for 9d8 (level 1, aimed); recharge 8+d8 turns

> A set of gauntlets that smoulder with an unnatural heat.

### Set of Gauntlets 'Paurnen'

- **Base**: Set of Gauntlets; **level** 1; **alloc** 10-60 (x30); **weight** 2.5 lb; **cost** 12000
- **Combat**: (+3, +3) \[3, +14\]
- **Resists**: resist acid
- **Flags**: FEATHER
- **Activation**: ACID\_BOLT: bolt of acid for 5d8 (level 1, aimed); recharge 5+d5 turns

> A set of gauntlets that give off a foul, acrid odour yet remain untarnished.

### Set of Gauntlets 'Paurnimmen'

- **Base**: Set of Gauntlets; **level** 1; **alloc** 50-127 (x5); **weight** 2.5 lb; **cost** 13000
- **Combat**: \[3, +14\]
- **Slays/brands**: brand cold x2
- **Resists**: resist cold
- **Flags**: SLOW\_DIGEST
- **Activation**: COLD\_BOLT: bolt or beam of cold for 6d8 (level 1, aimed); recharge 7+d7 turns

> A set of handgear, freezing with unnatural cold.

### Set of Leather Gloves 'Cambeleg'

- **Base**: Set of Leather Gloves; **level** 10; **alloc** 10-100 (x3); **weight** 0.5 lb; **cost** 36000
- **Combat**: (+8, +8) \[1, +15\]
- **Mods**: STR +2, CON +2
- **Flags**: FREE\_ACT

> A hero's handgear that lends great prowess in battle.

### Set of Gauntlets 'Camlost'

- **Base**: Set of Gauntlets; **level** 10; **alloc** 10-100 (x5); **weight** 2.5 lb; **cost** 0
- **Combat**: (-12, -12) \[3, +0\]
- **Mods**: STR -3, DEX -3
- **Resists**: resist fire, resist disenchantment
- **Flags**: FREE\_ACT, DRAIN\_EXP, AGGRAVATE

> A pair of weakening gauntlets, named after Beren, who returned to Thingol having lost to Carcharoth the hand that clasped a Silmaril.

### Set of Leather Gloves 'Cammithrim'

- **Base**: Set of Leather Gloves; **level** 10; **alloc** 10-80 (x18); **weight** 0.5 lb; **cost** 30000
- **Combat**: (+5, +5) \[1, +10\]
- **Mods**: LIGHT +1
- **Resists**: resist light
- **Flags**: FREE\_ACT, SUST\_CON
- **Activation**: MISSILE: bolt or beam of magic missiles for 3d4 (level 5, aimed); recharge 2 turns

> These gloves glow so brightly as to light the way for the wearer and cast magical bolts with great frequency.

### The Set of Caestus of Fingolfin

- **Base**: Set of Caestus; **level** 40; **alloc** 40-127 (x3); **weight** 4.0 lb; **cost** 110000
- **Combat**: (+10, +10) \[5, +20\]
- **Mods**: DEX +4
- **Resists**: resist acid
- **Flags**: FREE\_ACT
- **Activation**: ARROW: bolt of arrows for 150 (level 38, aimed); recharge 30+d30 turns

> The hand-sheathing of Fingolfin, the Elven warrior-king, who dealt Morgoth seven mighty wounds and pain that will last until the end of Arda.

### The Set of Gauntlets of Eöl

- **Base**: Set of Gauntlets; **level** 55; **alloc** 55-127 (x3); **weight** 2.5 lb; **cost** 40000
- **Combat**: \[3, +14\]
- **Mods**: INT +3
- **Resists**: resist lightning, resist dark, resist poison, immune fire
- **Flags**: FREE\_ACT, FEATHER, AGGRAVATE
- **Activation**: MANA\_BOLT: bolt of mana for 12d8 (level 52, aimed); recharge 30+d30 turns

> The iron-shod gauntlets of the Dark Elven smith Eöl, who imbued them with magics he could channel in battle.

## Boots (6)

### Pair of Leather Boots 'Dal-i-thalion'

- **Base**: Pair of Leather Boots; **level** 10; **alloc** 50-100 (x4); **weight** 2.0 lb; **cost** 40000
- **Combat**: \[2, +15\]
- **Mods**: DEX +5, SPEED +5
- **Resists**: resist nether
- **Flags**: FREE\_ACT, PROT\_CONF, SUST\_CON
- **Activation**: REM\_FEAR\_POIS: cures you of fear and poison \[cure fear; cure poisoning\] (level 10); recharge 5 turns

> A pair of high-laced shoes, bestowing extraordinary agility and proof against the powers of corruption and withering.

### The Pair of Leather Boots of Radagast

- **Base**: Pair of Leather Boots; **level** 10; **alloc** 15-70 (x8); **weight** 2.0 lb; **cost** 7000
- **Combat**: \[2, +12\]
- **Mods**: INFRA +5, STEALTH +3, WIS +1
- **Resists**: resist cold
- **Flags**: FREE\_ACT, FEATHER

> Made of brown leather, covered with scuff marks and dust, still these boots possess the magic of Radagast, friend of birds and nature.

### The Pair of Leather Boots of Wormtongue

- **Base**: Pair of Leather Boots; **level** 10; **alloc** 15-80 (x8); **weight** 2.0 lb; **cost** 17000
- **Combat**: (-1, -1) \[2, +0\]
- **Mods**: MOVES +1, INT +2, DEX +2, STEALTH +2
- **Flags**: FEATHER, TRAP\_IMMUNE
- **Activation**: TELE\_PHASE: teleports you randomly up to 10 squares away \[teleport self up to 10 grids\] (level 8); recharge 20 turns

> A pair of running shoes once used by the treacherous Gríma son of Gálmód, the Wormtongue of Edoras. They are aptly suited for a messenger, a servant, a spy... or a coward.

### Pair of Mithril Shod Boots 'Stormwalker'

- **Base**: Pair of Mithril Shod Boots; **level** 30; **alloc** 60-127 (x3); **weight** 4.0 lb; **cost** 40000
- **Combat**: \[8, +24\]
- **Mods**: SPEED +6
- **Resists**: immune lightning, resist sound
- **Activation**: BERSERKER: puts you in a berserker rage for d50+50 turns \[+50+1d50 turns of berserk rage\] (level 20); recharge 100 turns

> And they hewed off Gelmir's hands and feet, and his head last, within sight of the Elves, and left him. By ill chance there stood Gwindor of Nargothrond, the brother of Gelmir. Now his wrath was kindled to madness, and he leapt forth on horseback, and many riders with him; and they pursued the heralds and slew them, and drove on deep into the main host.

### The Pair of Leather Boots of Fëanor

- **Base**: Pair of Leather Boots; **level** 30; **alloc** 40-127 (x1); **weight** 2.0 lb; **cost** 300000
- **Combat**: \[2, +20\]
- **Mods**: SPEED +15
- **Resists**: resist nexus
- **Activation**: HASTE1: hastens you for d20+20 turns \[+20+1d20 turns of haste\] (level 40); recharge 200 turns

> This wondrous pair of leather boots once sped Fëanor, creator of the Silmarils and the mightiest of the Eldar, to fulfill his hero's challenge and do battle for a Middle-Earth held in thrall. "Dear-bought those songs shall be accounted, and yet shall be well-bought. For the price could be no other."

### The Pair of Steel Shod Boots of Thrór

- **Base**: Pair of Steel Shod Boots; **level** 30; **alloc** 30-100 (x5); **weight** 6.0 lb; **cost** 12000
- **Combat**: \[7, +20\]
- **Mods**: STR +3, CON +3, SPEED +3, STEALTH -1
- **Flags**: PROT\_FEAR

> Sturdy footwear of leather and steel as enduring as the long-suffering Dwarven king-in-exile who wore them.

## Helm (7)

### The Metal Cap of Thengel

- **Base**: Metal Cap; **level** 10; **alloc** 10-100 (x18); **weight** 2.0 lb; **cost** 22000
- **Combat**: \[3, +12\]
- **Mods**: WIS +3
- **Flags**: PROT\_CONF

> A ridged helmet made of steel and embossed with scenes of valor in fine-engraved silver. It grants the wearer nobility and understanding.

### Iron Helm 'Holhenneth'

- **Base**: Iron Helm; **level** 20; **alloc** 20-100 (x18); **weight** 5.0 lb; **cost** 100000
- **Combat**: 1d3 (+0, +0) \[7, +10\]
- **Mods**: INT +2, WIS +2, SEARCH +2
- **Flags**: PROT\_BLIND, PROT\_CONF, SEE\_INVIS
- **Activation**: DETECT\_ALL: detects treasure and all creatures and maps an area nearby \[detects gold nearby (area 22 x 40); senses objects nearby (area 22 x 40); detects invisible creatures nearby (area 22 x 40); detects visible creatures nearby (area 22 x 40); maps the area around you (area 11 x 11)\] (level 20); recharge 55+d55 turns

> A famous forged iron helm granting extraordinary powers of mind and awareness.

### The Iron Helm of Gorlim

- **Base**: Iron Helm; **level** 20; **alloc** 20-75 (x18); **weight** 7.5 lb; **cost** 1
- **Combat**: 1d3 (+8, +8) \[7, +10\]
- **Mods**: INT -3, WIS -3, SEARCH -5
- **Flags**: PROT\_FEAR, SEE\_INVIS, FREE\_ACT, AGGRAVATE

> A headpiece, gaudy and barbaric, that betrayed a warrior when he most needed succor.

### The Steel Helm of Hammerhand

- **Base**: Steel Helm; **level** 20; **alloc** 40-127 (x11); **weight** 6.0 lb; **cost** 45000
- **Combat**: 1d3 (+0, +0) \[9, +20\]
- **Mods**: STR +3, DEX +3, CON +3, STEALTH -2
- **Resists**: resist acid, resist nexus, resist cold, resist dark
- **Flags**: FREE\_ACT

> A great helm, as steady and as distinctive as the hero of the Westdike. Many long months was Helm Hammerhand besieged in the Hornburg, yet he strode out time and again among his foes and slew many. Although they knew of his coming, none could stand against him.

### The Iron Helm of Dor-Lómin

- **Base**: Iron Helm; **level** 40; **alloc** 40-127 (x5); **weight** 5.0 lb; **cost** 300000
- **Combat**: 1d3 (+0, +0) \[7, +20\]
- **Mods**: STR +4, CON +4, DEX +2, STEALTH -1
- **Resists**: resist acid, resist lightning, resist fire, resist cold
- **Flags**: PROT\_FEAR, SEE\_INVIS, TELEPATHY

> The legendary Dragon-helm, forged by one of the legendary Dwarven smiths of the First Age. The visage of mighty Glaurung forms its crest, and the mere sight of the helm, borne by Túrin Turambar, struck dread into the hearts of the servants of Morgoth.

### The Hard Leather Cap of Thranduil

- **Base**: Hard Leather Cap; **level** 50; **alloc** 40-100 (x14); **weight** 1.5 lb; **cost** 50000
- **Combat**: \[2, +10\]
- **Mods**: INT +2, WIS +2
- **Flags**: PROT\_BLIND, TELEPATHY

> The hunting cap of King Thranduil, to whose ears come all the secrets of his forest domain.

### The Metal Cap of Celebrimbor

- **Base**: Metal Cap; **level** 55; **alloc** 55-100 (x8); **weight** 2.0 lb; **cost** 45000
- **Combat**: \[3, +18\]
- **Mods**: INT +2, DEX +2, SEARCH +3
- **Resists**: resist fire, resist acid, resist disenchantment, resist shards

> A metal cap forged by the greatest Noldorin smith of the Second Age, this helm serves equally well in battle or at the forge, and its enchantment will never be diminished.

## Crown (4)

### The Iron Crown of Berúthiel

- **Base**: Iron Crown; **level** 40; **alloc** 45-100 (x8); **weight** 2.0 lb; **cost** 1
- **Combat**: \[0, +20\]
- **Mods**: CON -5, STR -3, DEX -3
- **Flags**: FREE\_ACT, SEE\_INVIS, TELEPATHY

> The midnight-hued steel circlet of the sorceress-queen Berúthiel. It grants extraordinary powers of sight and awareness at a terrible physical cost.

### The Golden Crown of Gondor

- **Base**: Golden Crown; **level** 40; **alloc** 45-127 (x5); **weight** 3.0 lb; **cost** 100000
- **Combat**: \[0, +15\]
- **Mods**: STR +3, CON +3, WIS +2, LIGHT +1
- **Resists**: resist cold, resist fire, resist light
- **Flags**: PROT\_BLIND, PROT\_CONF, PROT\_STUN, SEE\_INVIS, REGEN
- **Activation**: HEAL1: heals 500 hit points \[heal 500 HP; cure wounds\] (level 60); recharge 250 turns

> From dying Númenor, Elendil brought this shining winged circlet, emblem of Gondor through an age of the world.

### The Jewel Encrusted Crown of Númenor

- **Base**: Jewel Encrusted Crown; **level** 60; **alloc** 60-127 (x5); **weight** 4.0 lb; **cost** 50000
- **Combat**: \[0, +18\]
- **Mods**: INT +3, DEX +2, SEARCH +3, LIGHT +1
- **Resists**: resist shards, resist cold, resist light, resist dark
- **Flags**: SEE\_INVIS, FREE\_ACT, PROT\_STUN, PROT\_BLIND

> A massive golden crown, set with wondrous jewels of thought and warding, once worn by the kings of ancient Númenor.

### The Massive Iron Crown of Morgoth

- **Base**: Massive Iron Crown; **level** 100; **alloc** 100-100 (x0); **weight** 40.0 lb; **cost** 10000000
- **Mods**: STR +50, INT +50, WIS +50, DEX +50, CON +50, INFRA +50, LIGHT +1
- **Resists**: resist acid, resist lightning, resist fire, resist cold, resist poison, resist light, resist dark, resist nexus, resist nether
- **Flags**: PROT\_CONF, SEE\_INVIS, TELEPATHY, PROT\_FEAR, STICKY

> Containing much of the power of him who once was mightiest among the Ainur, this plain iron crown has mounted upon it the two remaining Silmarils, greatest treasures of Middle-Earth.

## Light (3)

### The Phial of Galadriel

- **Base**: Phial (unique item kind made for this artifact); **level** 5; **alloc** 5-100 (x40); **weight** 1.0 lb; **cost** 10000
- **Mods**: LIGHT +4
- **Flags**: NO\_FUEL
- **Activation**: ILLUMINATION: lights up the surrounding area; sphere of weak light around you (radius 3) for 2d15 (level 12); recharge 10+d10 turns

> A small crystal phial containing the light of Eärendil's Star. Its light is imperishable, and near it darkness cannot endure.

### The Star of Elendil

- **Base**: Star (unique item kind made for this artifact); **level** 30; **alloc** 30-100 (x4); **weight** 0.5 lb; **cost** 30000
- **Mods**: LIGHT +4
- **Flags**: SEE\_INVIS, NO\_FUEL
- **Activation**: MAPPING: maps the area around you \[maps the area around you (area 22 x 40)\] (level 30); recharge 50+d50 turns

> The shining Star of the North, a treasured heirloom of Elendil's house.

### The Arkenstone of Thráin

- **Base**: Arkenstone (unique item kind made for this artifact); **level** 50; **alloc** 50-127 (x2); **weight** 0.5 lb; **cost** 50000
- **Mods**: LIGHT +4
- **Resists**: resist light, resist dark
- **Flags**: SEE\_INVIS, HOLD\_LIFE, NO\_FUEL
- **Activation**: CLAIRVOYANCE: lights and maps the entire level, sensing objects \[completely lights up and magically maps the level\] (level 50); recharge 50+d50 turns

> A great globe seemingly filled with moonlight, the famed Heart of the Mountain. It splinters the light that falls upon it into ten thousand sparks of white radiance shot with glints of the rainbow.

## Ring (6)

### Ring of Fire 'Narya'

- **Base**: Ring of Fire (unique item kind made for this artifact); **level** 30; **alloc** 70-127 (x2); **weight** 0.2 lb; **cost** 100000
- **Combat**: (+2, +2)
- **Mods**: STR +1, INT +1, WIS +1, DEX +1, CON +1, SPEED +5
- **Resists**: immune fire
- **Flags**: FREE\_ACT, SEE\_INVIS, SLOW\_DIGEST, REGEN, TELEPATHY, SUST\_STR, SUST\_CON, PROT\_FEAR
- **Activation**: PROTEVIL: grants you protection from evil for 1d25 plus 3 times your character level turns \[+(plev\*3)+1d25 turns of protection from evil\] (level 40); recharge 200+4d50 turns

> The Ring of Fire, made of gold and set with a ruby that glows like flame. Narya is one of the three Rings of Power created by Celebrimbor and hidden by the Elves from Sauron.

### Ring of Adamant 'Nenya'

- **Base**: Ring of Adamant (unique item kind made for this artifact); **level** 40; **alloc** 80-127 (x2); **weight** 0.2 lb; **cost** 200000
- **Combat**: (+3, +3)
- **Mods**: STR +2, INT +2, WIS +2, DEX +2, CON +2, SPEED +5
- **Resists**: immune cold
- **Flags**: HOLD\_LIFE, FREE\_ACT, SEE\_INVIS, FEATHER, REGEN, SUST\_INT, SUST\_WIS, SUST\_CON, PROT\_BLIND, TELEPATHY
- **Activation**: RESTORE\_LIFE: restores your experience to full \[restores your experience\] (level 40); recharge 200+2d50 turns

> The Ring of Adamant, made of mithril with a pure white stone as centerpiece. Nenya is one of the three Rings of Power created by Celebrimbor and hidden by the Elves from Sauron.

### The Ring of Barahir

- **Base**: Ring (unique item kind made for this artifact); **level** 50; **alloc** 50-100 (x4); **weight** 0.2 lb; **cost** 65000
- **Mods**: STR +1, INT +1, WIS +1, DEX +1, CON +1, STEALTH +1
- **Resists**: resist poison, resist dark

> Twinned serpents with eyes of emerald meet beneath a crown of flowers to form this ring, an ancient treasure of Isildur's house.

### The Band of Tulkas

- **Base**: Band (unique item kind made for this artifact); **level** 60; **alloc** 70-127 (x2); **weight** 0.2 lb; **cost** 150000
- **Mods**: STR +4, DEX +4, CON +4
- **Flags**: PROT\_FEAR
- **Activation**: HASTE2: hastens you for d75+75 turns \[+75+1d75 turns of haste\] (level 60); recharge 150+d150 turns

> The treasure of Tulkas, most fleet and wrathful of the Valar.

### Ring of Power 'The One Ring'

- **Base**: Ring of Power (unique item kind made for this artifact); **level** 75; **alloc** 100-127 (x1); **weight** 0.2 lb; **cost** 5000000
- **Combat**: (+15, +15)
- **Mods**: STR +5, INT +5, WIS +5, DEX +5, CON +5, SPEED +5
- **Resists**: immune fire, immune cold, immune lightning, immune acid, resist dark, resist disenchantment, resist poison, resist nether
- **Flags**: AGGRAVATE, DRAIN\_EXP, SEE\_INVIS, REGEN, TELEPATHY, STICKY, SUST\_STR, SUST\_DEX, SUST\_CON, SUST\_INT, SUST\_WIS
- **Activation**: BIZARRE: does bizarre things (level 80, aimed); recharge 200+9d50 turns

> "One Ring to rule them all, One Ring to find them, One Ring to bring them all and in the darkness bind them." Made of massive gold, and inscribed with hidden runes in the foul speech of Mordor, Isildur's Bane possesses powers so great that it inevitably twists and masters any mortal being who wears it.

### Ring of Firmament 'Vilya'

- **Base**: Ring of Firmament (unique item kind made for this artifact); **level** 80; **alloc** 90-127 (x1); **weight** 0.2 lb; **cost** 300000
- **Combat**: (+5, +5)
- **Mods**: STR +3, INT +3, WIS +3, DEX +3, CON +3, SPEED +5
- **Resists**: immune lightning, resist poison, resist dark
- **Flags**: HOLD\_LIFE, FREE\_ACT, SEE\_INVIS, FEATHER, SLOW\_DIGEST, REGEN, TELEPATHY, SUST\_DEX, SUST\_INT, SUST\_CON
- **Activation**: HEAL2: heals 1000 hit points \[heal 1000 HP; cure wounds\] (level 80); recharge 200+2d50 turns

> The Ring of Sapphire, made of gold with a brilliant blue gem that shines like stars. It glitters untouchable despite all that Morgoth ever wrought. Vilya is one of the three Rings of Power created by Celebrimbor and hidden by the Elves from Sauron.

## Amulet (5)

### The Amulet of Carlammas

- **Base**: Amulet (unique item kind made for this artifact); **level** 30; **alloc** 50-100 (x10); **weight** 0.3 lb; **cost** 60000
- **Mods**: CON +3, STR +1
- **Resists**: resist fire
- **Activation**: PROTEVIL: grants you protection from evil for 1d25 plus 3 times your character level turns \[+(plev\*3)+1d25 turns of protection from evil\] (level 40); recharge 225+d225 turns

> A fiery circle of bronze, with mighty spells to ward off and banish evil.

### Jewel 'Evenstar'

- **Base**: Jewel (unique item kind made for this artifact); **level** 40; **alloc** 30-70 (x2); **weight** 0.3 lb; **cost** 25000
- **Mods**: WIS +2
- **Resists**: resist dark, resist cold, resist nether
- **Flags**: HOLD\_LIFE, SUST\_CON, SUST\_WIS, REGEN
- **Activation**: RESTORE\_LIFE: restores your experience to full \[restores your experience\] (level 40); recharge 150 turns

> A plain white jewel, given by Queen Arwen to Frodo Baggins before his return to the Shire.

### The Necklace of the Dwarves

- **Base**: Necklace (unique item kind made for this artifact); **level** 50; **alloc** 50-127 (x2); **weight** 0.3 lb; **cost** 75000
- **Mods**: STR +3, CON +3, INFRA +5, LIGHT +1
- **Flags**: SEE\_INVIS, FREE\_ACT, REGEN, PROT\_FEAR

> The Nauglamír, a carcanet of gold set with a multitude of shining gems of Valinor. The radiant Silmaril of Fëanor hangs in its midst as its crowning glory. The sturdy spirits of Dwarvish craftsmen who labored long in mountain smithies lie within it still, and as gossamer it rests upon the bearer.

### Elfstone 'Elessar'

- **Base**: Elfstone (unique item kind made for this artifact); **level** 60; **alloc** 60-127 (x2); **weight** 0.3 lb; **cost** 40000
- **Combat**: \[0, +10\]
- **Mods**: STR +2, WIS +2, SPEED +2
- **Resists**: resist fire, resist poison
- **Flags**: PROT\_FEAR
- **Activation**: HEAL1: heals 500 hit points \[heal 500 HP; cure wounds\] (level 60); recharge 200 turns

> A green stone, imbued with the power of Elvendom and the light of the Sun. Those who gaze through it see the aged and infirm as young again, and its wearer brings healing after victory in battle.

### The Pendant of Ingwë

- **Base**: Pendant (unique item kind made for this artifact); **level** 65; **alloc** 65-127 (x3); **weight** 0.3 lb; **cost** 90000
- **Mods**: INT +3, WIS +3, INFRA +4
- **Resists**: resist acid, resist cold, resist lightning
- **Flags**: SEE\_INVIS, FREE\_ACT, PROT\_CONF
- **Activation**: DISPEL\_EVIL: deals five times your level's damage to all evil creatures that you can see \[damages evil monsters in line of sight (power (plev\*5))\] (level 65); recharge 50+d50 turns

> The ancient heirloom of Ingwë, high lord of the Vanyar, against whom nothing of evil could stand.
