# source101: seed the dead-unique table with uniques at start-up (patch 0121)

Consumer adoption of [source 101](https://github.com/diegoami/neo-angband-tracker/issues/101),
fix `35d4ff3f4` (branch `task/deaduniq-101`), kit `docs/audit/c-reports/deaduniq-101` from the
evidence branch `task/deaduniq-101-evidence`.

## Defect

`borg_init_flow_kill()` fills `borg_race_death[]` from `r_info[].max_num`, which the game sets to 0
for a unique the player has killed. The test is inverted against its own comment:

```c
/* Skip non-uniques */
if (rf_has(r_ptr->flags, RF_UNIQUE))
    continue;
```

It skips the uniques and checks the non-uniques, whose `max_num` is never 0, so the table starts
empty even when the savefile records dead uniques. The first `borg_update()` pass fills it on its
new-level branch, but `borg_think()` calls `borg_notice()` before `borg_update()` and computes
`borg.power` from that notice. On the first pass the reference power therefore misses the
dead-unique terms (Sauron's death is worth 2,500 per Scroll of Mass Banishment in `borg_power()`)
while every candidate the same pass evaluates is scored with the table filled. A candidate that is
no better, or worse, can outscore the stale baseline: the Borg swaps in a weaker item or drops and
ignores a useful one.

## Fix

`if (!rf_has(r_ptr->flags, RF_UNIQUE)) continue;`: skip non-uniques, as the comment and the
matching scan in `borg_update()` do. Source `35d4ff3f4` -> applied `1b2ec94df` (`cherry-pick -x`),
patch `patches/0121-deaduniq-seed-start.patch`, scope `src/borg/borg-flow-kill.c` only (one line).

## Verification

Before commit `6b50eeb08` (the kit commit, `src/` identical to `origin/main`), tip `1b2ec94df`, kit
unchanged (`tools/verify-deaduniq101.py`). Native gates green at both (965/965 unit, 5/5 frontend);
byte-wise provenance over the source diff; the `src`/`lib`/build scope is the one file.

| Mode | Before | After |
| --- | --- | --- |
| `wear` (Sauron dead, 5 Mass Banishment, spare Hard Leather Cap; Human Warrior, seed 101, level 41, depth 98) | **fails** 0/1: the Borg swaps the Metal Cap for the weaker cap (`acts=1`, `baseline_stale=1`) | passes 1/1: no swap (`0`, `0`) |
| `junk` (Sauron dead, 5 Mass Banishment, junk decision) | **fails** 0/1: drops and ignores a Potion of Berserk Strength (`1`, `1`) | passes 1/1: nothing dropped (`0`, `0`) |
| `junk-nokitpotion` (same without the birth-kit potions) | **fails** 0/1: drops and ignores a Mass Banishment scroll (`1`, `1`) | passes 1/1 (`0`, `0`) |
| `alive-wear`, `alive-junk` (Sauron alive), controls | pass (`0`, `0`) | pass, **byte-identical** |
| `wear-noscrolls` (Sauron dead, no Mass Banishment) | passes (`0`, `0`) | passes; **not byte-identical** (the trait line differs), shown to bound the effect |

The committed `before-*.log` / `after-*.log` are the source lane's captures; the product runs of this
tree (`product-result.json`) are byte-identical to all twelve of them, so the kit reproduces
unchanged. The source's own rows and hashes are in `receipt.json`, carried unchanged.

## Mode attribution

Per the 2026-11-10 owner rule, each configuration is stated, and "neutral" is claimed only where the
changed branch executes. `borg_init_flow_kill()` runs at start-up in every configuration, so the
corrected loop executes in unrestricted, munchkin/Lunal-only and normal-only play alike. It changes a
decision only for a character whose savefile records a dead unique at start-up (the fixture records
Sauron's death as the game does, lore kill count 1 and `max_num` 0). A fresh character has none, so a
fresh-start cohort such as the zoo's class grid cannot move on this carry; it is not a measurement of
neutrality either. The effect for a Borg started from an existing savefile with dead uniques is
**not measured** in any configuration and belongs to the batch with the zoo's explicit arms
(borg-zoo issue 54).

Matrix group `deaduniq-101` (6 cases) uses the `after-*.log` rows as its baseline.

Full matrix at this tree: 99 groups / 305 cases, `ok: true`, no failed group, 252/265
byte-identical; the 13 byte differences are in the same six groups as before this carry (`dispel`,
`dispel259`, `fleeheal-408`, `knockout`, `preparation`, `vampire`), so no new difference.
