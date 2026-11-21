# source234: keep the current depth's preparedness reason from being overwritten (patch 0106)

Consumer adoption of [source 234](https://github.com/diegoami/neo-angband-tracker/issues/234),
fix `52792d3e5` (source PR 577, evidence PR 576), kit
`docs/audit/c-reports/prepreason-234` at fixture commit `ffd6eaef5`.

## Defect

`borg_prepared()` formats every "why not ready for this depth" reason - "No deeper 100.",
"Collect from house (20 potions).", "Must kill kobold." - into one static buffer and returns a
pointer into it. `borg_leave_level()` held the current depth's pointer across three later
`borg_prepared()` calls (depth + 1, depth + 5, `max_depth * 5 / 10`), each of which overwrote
the text. The "heading up, not prep for current level", "Returning to town (too deep)" and
"Climbing (too deep)" notes printed a later depth's reason, and the "Collect from house"
substring test read a later depth's reason, so the return-to-town-to-collect decision could be
missed or taken for the wrong reason.

## Fix

Copy the current depth's reason into a local buffer right after the first call and use the copy
for the notes and the substring test. Source `52792d3e5` -> applied `6cf8b4f82`
(`cherry-pick -x`), patch `patches/0106-prepreason-copy.patch`, scope
`src/borg/borg-think-dungeon-util.c` only.

## Verification

Before commit `779b4799e`, tip `6cf8b4f82`, kit unchanged (`tools/verify-prepreason234.py`).
Native gates green at both (965/965 unit, 5/5 frontend); byte-wise provenance over the source
diff; the `src`/`lib`/build scope is the one file.

| Mode | Before | After |
| --- | --- | --- |
| `nodeeper100` (level-50 Warrior, depth 99, 20 Healing at home, `borg_no_deeper = 100`) | **fails** 0/1: notes "No deeper 100." (the next depth's reason), no collect, walks to a staircase | passes 1/1: real reason noted, returns to town to collect, reads Recall |
| `default127` (default `borg_no_deeper`) | passes, control | passes, **byte-identical** |
| `prepared97` (current depth prepared) | passes, control | passes, **byte-identical** |

The committed `before-*.log` / `after-*.log` and `result.json` are the product runs; the
source kit's own rows and file hashes are in `receipt.json`, carried unchanged.

## Mode attribution

The fix is in the leave-level decision code every configuration runs; there is no mode gate, and
the reason buffer is overwritten identically in unrestricted, munchkin/Lunal and normal play.
What the fix changes is which text the notes and the "Collect from house" test see; it only
changes a decision when the current and next depths format different reasons. The fixture proves
the before/after flip in that configuration and two controls. Which runtime configuration
reaches that combination in a measured cohort is recorded as **not measured** rather than
neutral and is owed to the batch with the zoo's explicit arms (borg-zoo issue 54).

Matrix group `prepreason-234` (3 cases) uses the `after-*.log` rows as its baseline. Full matrix
at this tree: 84 groups / 257 cases, `ok: true`, no failed group, 204/217 byte-identical; the 13
byte differences are in the same six groups as before this carry (`dispel`, `dispel259`,
`fleeheal-408`, `knockout`, `preparation`, `vampire`), so no new difference.

## Ledger and credit

`patches/series.json` `PREPREASON-234`, patch-active; consumer issue #446. Source defect, fix and
native fixture by Diego Amicabile (native proposal/evidence); carried unchanged.
