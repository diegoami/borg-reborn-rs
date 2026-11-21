# source61: test constitution for the constitution-drain blow fear (patch 0107)

Consumer adoption of [source 61](https://github.com/diegoami/neo-angband-tracker/issues/61)
(CBUG-0045), fix `0ea55f39b` (source PR 469, evidence PR 468), kit
`docs/audit/c-reports/losecon-61` at fixture commit `3a886f40c`.

## Defect

In `borg_danger_physical()` a constitution-drain blow adds extra fear when the Borg would be left
weakened, and the below-8 test read the wrong trait:

```c
/* extra scary to have con drain below 8 */
if (borg.trait[BI_CSTR] < 8)
    z += 100;
```

It tests strength. A low-constitution Borg therefore does not get the extra fear from a
con-drain monster, and a low-strength Borg gets it wrongly. The value feeds `borg_danger()`, so
it can change fight, flee and target choice against any drainer.

## Fix

Read `borg.trait[BI_CCON]`. Source `0ea55f39b` -> applied `2abb17c35` (`cherry-pick -x`), patch
`patches/0107-losecon-con-fear.patch`, scope `src/borg/borg-danger.c` only.

## Verification

Before commit `293e153ec`, tip `2abb17c35`, kit unchanged (`tools/verify-losecon61.py`). Native
gates green at both (965/965 unit, 5/5 frontend); byte-wise provenance over the source diff; the
`src`/`lib`/build scope is the one file.

| Mode | Before | After |
| --- | --- | --- |
| `lowcon` (str 17, con 7) | **fails** 0/1: danger 456, no escape | passes 1/1: danger 756, escape taken |
| `lowstr` (str 7, con 16) | **fails** 0/1: danger 756, escape taken | passes 1/1: danger 456, no escape |
| `neither` (str 17, con 16) | passes, control: danger 456 | passes, **byte-identical** |
| `bothlow` (str 7, con 7) | passes, control: danger 756 | passes, **byte-identical** |

The two flips are symmetric: the +100 fear moves from the low-strength case to the
low-constitution case, and both controls keep the same danger and the same log.

The committed `before-*.log` / `after-*.log` and `result.json` are the product runs; the source
kit's own rows and file hashes are in `receipt.json`, carried unchanged.

## Mode attribution

Unconditional change in the shared danger model, so there is no mode gate to prove: the function
runs in every thinker and every mode. The fixture drives `borg_danger_physical()` directly and
shows the flip, so the branch executes rather than being disabled. The per-configuration cohort
effect (which of unrestricted, munchkin/Lunal and normal play actually meets a con-drain monster
at low constitution) is recorded as **not measured**, not neutral, and belongs to the batch with
the zoo's explicit arms (borg-zoo issue 54). This is the first of the three `borg-danger.c`
fixes and the batch notes they compound, so measuring them one at a time on the same arm is the
only way to attribute an effect.

Matrix group `losecon-61` (4 cases) uses the `after-*.log` rows as its baseline. Full matrix
at this tree: 85 groups / 261 cases, `ok: true`, no failed group, 208/221 byte-identical; the
13 byte differences are in the same six groups as before this carry, so no new difference.

## Ledger and credit

`patches/series.json` `LOSECON-61`, patch-active; consumer issue #448. Source defect, fix and
native fixture by Diego Amicabile (native proposal/evidence); carried unchanged.
