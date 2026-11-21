# source52: type the birth menu's own letters when respawning (patch 0123)

Consumer adoption of [source 52](https://github.com/diegoami/neo-angband-tracker/issues/52)
(CBUG-0036), fix `85e7ce6c6` (branch `task/rebirthkey-52`), kit
`docs/audit/c-reports/rebirthkey-52` from the evidence branch `task/rebirthkey-52-evidence`.

## Defect

When the Borg respawns it picks its new race and class by typing the menu letter for each choice.
It used `all_letters`, but the birth menus label their entries with `all_letters_nohjkl`, which skips
h, j, k and l:

```c
borg_keypress(all_letters[race]);   /* and the same for the class */
```

From the eighth entry on the typed letter was wrong. Half-Troll (h), High-Elf (j) and Kobold (k) typed
a letter the menu does not have, so the highlighted race (Human) was kept; Dunadan (i) selected
Half-Troll. For classes, Ranger (h) kept the highlighted class and Blackguard (i) selected Ranger. The
random name was still made for the race wanted, so a Half-Troll could get a human name.

## Fix

`all_letters_nohjkl[race]` and `all_letters_nohjkl[class]` in `borg_reincarnate_start()`: the same
letter table as the menus. Source `85e7ce6c6` -> applied `7a76e7a20` (`cherry-pick -x`), patch
`patches/0123-rebirth-menu-letters.patch`, scope `src/borg/borg-reincarnate.c` only (two lines).

## Verification

Before commit `374cea77b` (the kit commit, `src/` identical to `origin/main`), tip `7a76e7a20`, kit
unchanged (`tools/verify-rebirthkey52.py`). Native gates green at both (965/965 unit, 5/5 frontend);
byte-wise provenance over the source diff; the `src`/`lib`/build scope is the one file.

| Mode | Before | After |
| --- | --- | --- |
| `race8` (wanted Dunadan Warrior, previous Human Warrior; Human Warrior fixture) | **fails** 0/1: a Half-Troll is created (`race=7`, `right_character=0`) | passes 1/1: a Dunadan is created (`race=8`, `right_character=1`) |
| `class8` (wanted Human Blackguard) | **fails** 0/1: a Ranger is created (`class=7`) | passes 1/1: a Blackguard (`class=8`) |
| `race7` (wanted Half-Troll Warrior) | **fails** 0/1: the highlighted Human is kept (`race=0`) | passes 1/1: a Half-Troll (`race=7`) |
| `dwarfpriest` (race 5, class 3), `humanwarrior` (race 0, class 0), controls | pass | pass, **byte-identical** |

The committed `before-*.log` / `after-*.log` are the source lane's captures; the product runs of this
tree (`product-result.json`) are byte-identical to all ten, so the kit reproduces unchanged. The
source's own rows and hashes are in `receipt.json`, carried unchanged.

## Mode attribution

Per the 2026-11-10 owner rule, each configuration is stated, and "neutral" is claimed only where the
changed branch executes. `borg_reincarnate_start()` runs when the Borg dies with the game's
`cheat_live` option on (and `borg_cheat_death` off), or wins with `borg_respawn_winners` on. It has no
Lunal or munchkin gate, so the change is mode-independent: it executes in unrestricted,
munchkin/Lunal-only and normal-only configurations alike whenever a respawn happens, and in none of
them while respawn is off, which is the default. The zoo's class grid ends a run at death and never
respawns, so no zoo arm reaches it. The effect on a respawning Borg's later play is **not measured**
in any configuration; the fix only changes which race and class the new character gets, and the fixture
covers that. Owed to the batch with the zoo's explicit arms (borg-zoo issue 54).

Matrix group `rebirthkey-52` (5 cases) uses the `after-*.log` rows as its baseline.

Full matrix at this tree: 101 groups / 312 cases, `ok: true`, no failed group, 259/272
byte-identical; the 13 byte differences are in the same six groups as before this carry (`dispel`,
`dispel259`, `fleeheal-408`, `knockout`, `preparation`, `vampire`), so no new difference.
