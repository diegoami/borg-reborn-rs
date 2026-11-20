## 2026-11-20: revert #685's low-level detection limit

Owner, 2026-11-20: "turning on call light and detect monsters on casters low
level seems a bad call, I would revert it and file an experiment in the zoo".

**What was done:**
- `light_detect_evil_spell_min_level` back to 0 (C) for every class: the
  global line in `harness/borg.txt` (10 since #723) and the class files of
  Mage, Druid, Priest, Necromancer and Paladin (10 since #685), refolded with
  `scripts/borg-class-files.py fold`.
- `smoke-borg-rs` re-recorded (`f389216d`); 2 of 3 games change.

**Why:**
- In borg-zoo#460's pilot, a Human Necromancer (300036923) at CL1 walked
  toward a dark grid on a new DL1 level, next to a floating eye it never saw.
  It was paralysed by the gaze before it could act, and worn down one HP per
  gaze. The engine gives an already paralysed player 1 damage per paralysing
  blow (`mon-blows.c:1019-1021`), and the eye (speed 110) gazes again as the
  paralysis ends. Replayed on `0a76dc19` with `caution_breeder_stoppable = 1`:
  death at game turn 5,910, as in the zoo.
- At CL1, unlight (`2 + lev/6`) and a Wooden Torch (`LIGHT_2`, radius 1)
  both show only the 8 adjacent grids in a dark room (`cave-view.c:658,777`),
  so every class meets the eye only when adjacent. Floating eyes: 165 of
  8,702 distinct zoo deaths, 117 at CL1 and 38 at CL2, every class with
  spells.
- Detection shows the eye first. #685 had turned detection off below CL10
  to stop low-level Priests and Paladins from spending their few SP on every
  panel and then resting for mana.

**Left:** the zoo experiment (0 against #685's caster 10 against 10 for
every class) decides. #685's resting cost comes back with this revert.
