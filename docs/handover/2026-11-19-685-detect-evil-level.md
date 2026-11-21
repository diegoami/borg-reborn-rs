## 2026-11-19: no detection prayer by panel at low level (#685)

Owner: "do 685". The values are the owner's: 10 for the casters.

- **Found:**
  - On main (`a87b4c96`), low-level Priests and Paladins spend 40% and 46%
    of game time resting at full HP for mana. C's `borg_check_light` casts
    Detect Evil on every new panel, which empties 2–6 SP, and
    `recover_mana_rest_*` then rests.
  - Detect Evil misses the monsters that kill Priests on DL1: crows, wild
    cats, Farmer Maggot's dogs, soldiers. None of them is evil.
- **Code** (`borg-core/src/borg_light.rs`, declared in
  `scripts/borg-core-vendor.py`): `light_detect_evil_spell_min_level`
  (0 = C). Below that level the panel check casts no Detect Evil, Detect
  Monsters, Read Minds or Seek Battle. Items and rods still detect.
- **Shipped** (class files): 10 for Priest, Paladin, Mage, Necromancer and
  Druid; 0 for Warrior, Rogue, Ranger and Blackguard.
- **Local check**, 100,000 turns:
  - The 34 Priest and Paladin starts that died in borg-zoo's calmed
    confirmation (`reborn-neutral38-calmed`), with the switch at 10 for
    Priest and Paladin only, against main:

    | | Priest, main | Priest, switch | Paladin, main | Paladin, switch |
    |---|---|---|---|---|
    | Mean max CL | 4.72 | 5.96 | 4.67 | 5.44 |
    | Mana-rest time | 40% | 18% | 46% | 25% |
    | Deaths | 1 | 1 | 0 | 0 |

  - classes-18 with the shipped values, against main: Mage CL 4.5 → 6.0
    and mana rest 60% → 27%; Necromancer CL 5.5 → 4.5 and mana rest
    49% → 38% (n = 2); other classes the same; no deaths in either arm.
  - The 34 starts were chosen by an earlier death, and 18 games measure
    nothing. A zoo run decides (RULES 14).
- **The issue's other questions:**
  - Priests learn a random prayer from the book (`do_cmd_study_book`,
    `src/cmd-obj.c:1200`), so a missing Minor Healing is the engine's luck.
  - Minor Healing is cast 0–7 times a game when known.
  - Bless (10–70 casts) comes mostly from `borg_caution`. The cast before
    the stairs needs 60% SP. Bless and Call Light are most of the mana rest
    that is left.
- **smoke-borg-rs re-recorded** (`scripts/rerecord-smoke-borg-rs.sh`): 2/3
  games identical to the old reference.
- **Checked:** fmt, clippy `-D warnings`, both test commands (core 472),
  the class files check, `borg-core-vendor.py check` (against
  `~/projects/angband/borg-rs`), citations (2200, 0 wrong).

**Left:** a zoo measurement against main (classes-1800, per class).
Possibly a follow-up on Minor Healing being used so little.
