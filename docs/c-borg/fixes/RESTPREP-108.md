# source108: widen the no-rest-after-prep timer so it cannot wrap (patch 0105)

Consumer adoption of [source 108](https://github.com/diegoami/neo-angband-tracker/issues/108)
(CBUG-0092), fix `9d1a7f1d2` (source PR 509, evidence PR 508), kit
`docs/audit/c-reports/restprep-108` at fixture commit `175792dd8`.

## Defect

`borg.no_rest_prep` keeps the Borg from resting right after a preparatory spell so the spell is
not wasted. The field was an `int16_t` (maximum 32767) while the code stores
`borg.trait[BI_CLEVEL] * 1000`, so from character level 33 the value wrapped negative: level 35
gave -30536. The checks that should see a recent prep spell (`borg_check_rest()`, the timer
countdown and the weak-caster stair check) read the timer as already expired, and the Borg
rested straight after casting. The same assignment appears after Protection from Evil and in
several fight and escape paths.

## Fix

Make the field an `int`. Source `9d1a7f1d2` -> applied `8dfb1cba8` (`cherry-pick -x`), patch
`patches/0105-restprep-timer-width.patch`, scope `src/borg/borg.h` only.

## Verification

Before commit `dcaf8dbe1`, tip `8dfb1cba8`, kit unchanged (`tools/verify-restprep108.py`).
Native gates green at both (965/965 unit, 5/5 frontend); byte-wise provenance over the source
diff; the `src`/`lib`/build scope is the one header.

The change is a structure field width, so the fixture is compiled against each phase's own
headers and linked against that phase's objects: a product-header fixture against before
objects would mix two layouts.

| Mode | Before | After |
| --- | --- | --- |
| `pfe` (level-35 Priest, Protection from Evil) | **fails** 0/1: timer -30536, `borg_check_rest()` allows resting, `rests_after_prep=1` | passes 1/1: timer 35000, resting refused, `rests_after_prep=0` |
| `clevel30` (level-30 control) | passes: timer 30000 fits an `int16_t` | passes, **byte-identical** |

The committed `before-*.log` / `after-*.log` and `result.json` are the product runs; the
source kit's own rows and file hashes are in `receipt.json`, carried unchanged.

The wrap is also visible in an independent prior fixture: the `preparation` group's
`speed-wins` and `speed-only` cases now print `no_rest_prep=50000` where their committed
baselines hold `-15536`, with identical decisions (`result=1 keys=3 sequence=mbb`). That is
the negative wrapped timer that this carry removes; the full matrix reports it as a byte
difference with matching rows, not a failure.

## Mode attribution

This is a field-width change in a structure every mode uses: the level × 1000 assignments run
in the preparatory, fight and escape paths, so the changed store executes in unrestricted,
munchkin/Lunal and normal play alike (no mode gate). What changes is only the value the store
can hold, and only from character level 33; the fixture proves the before/after behaviour at
level 35 with the level-30 control unchanged. Which runtime configuration reaches level 33+ in
a measured cohort is a cohort question, recorded as **not measured** rather than neutral and
owed to the batch with the zoo's explicit arms (borg-zoo issue 54).

Matrix group `restprep-108` (2 cases) uses the `after-*.log` rows as its baseline. Full matrix
at this tree: 83 groups / 254 cases, `ok: true`, no failed group, 201/214 byte-identical; the
13 byte differences are in `dispel`, `dispel259`, `fleeheal-408`, `knockout`, `vampire` (the
same groups as v10) plus `preparation`, explained above.

## Ledger and credit

`patches/series.json` `RESTPREP-108`, patch-active; consumer issue #443. Source defect, fix and
native fixture by Diego Amicabile (native proposal/evidence); carried unchanged.
