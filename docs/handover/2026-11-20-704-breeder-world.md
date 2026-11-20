## 2026-11-20: breeder worlds (#704)

Owner, 2026-11-19: "we could think of a way we could test breeders, like
using a engine where breeders are more frequent, it should be driveable by
configuration file"; 2026-11-20: "do 704".

**What was done:**
- **`scripts/make-world.py --config FILE --lib LIB --out DIR`** copies the
  build's `lib/` (`gamedata`, `customize`, `help`, `screens`) and applies
  the config to `monster.txt` and `constants.txt`:
  - `breeder_weight`: every race without `MULTIPLY` gets its rarity times N.
    The weight is `100 / rarity` (`mon-make.c:113`), so breeders become N
    times as likely and the level's monster count is unchanged. A rarity
    over 100 would never be generated, so such a weight is refused.
  - `breeder_depth_shift`: breeders N levels shallower (at least 1).
  - `breeders_only` / `breeders_except`: choose the breeders by name.
  - `repro_max`: `mon-gen:repro-max` (100).
  - Prints the SHA-256 of the generated `gamedata/`.
  - Tests: `scripts/tests/test_make_world.py`.
- **`harness/run.sh`:** `BORG_WORLD=FILE`, or `borg_world = NAME` in the
  run's `borg.txt` (a zoo profile can write it; `harness/worlds/NAME.txt`).
  - It generates the world into the run's `world/` and passes it with
    `-dgamedata` and the others.
  - It replaces the key line with a comment carrying both hashes, so the
    Borg never reads the key and the world is part of `borg_txt_sha256`.
  - `config.txt` records `borg_world`, `borg_world_sha256` and
    `borg_world_data_sha256`.
- **`harness/borg.txt`** ships `borg_world = none`. **`harness/worlds/`**
  ships `breeders-x4` (×4, 2 shallower) and `breeders-x8` (×8, 3
  shallower).
- **The zoo contract** (`docs/ZOO-INTERFACE.md`) has the key.

**Checked:**
- The engine loads the changed data cleanly: every world game ran to the
  cap or a death, with no parse error.
- The Borg's knowledge comes from the loaded races: its notes name the
  favoured breeders as they appear. The scary list matches by name, and
  names are unchanged.
- smoke-borg-rs 3/3 identical with `borg_world = none`.
- classes-18 through `harness/run.sh … 100000 exact,menu` (Human seeds 1-9,
  Half-Elf 10-18), main's shipped settings (`caution_breeder_stoppable = 1`
  and `recover_rest_breeders = 1`, owner). Local leads (RULES 14):

| world | breeders seen (`# Creating a monster`) | breeders killed | games with 10+ breeders seen | deaths | sum max CL / DL | "Breeders cannot be stopped" |
|---|---:|---:|---:|---:|---:|---:|
| none | 399 | 96 | 14 | 0 | 104 / 37 | 0 |
| breeders-x4 | 1,492 | 264 | 17 | 2 (cave spider, poison) | 98 / 39 | 4 |
| breeders-x8 | 2,771 | 411 | 18 | 1 (a blue worm mass) | 104 / 42 | 8 |

- x4 gives about 3.7 times the breeders and x8 about 7 times. Progress is
  about the same: x8 goes deeper, because breeders are easy experience.

**Left / for the owner:**
- The zoo experiment: the breeder switches' arms in a breeder world.
- Other rare situations (uniques, out-of-depth monsters, invisible ones)
  could use the same mechanism with new config keys, later.
