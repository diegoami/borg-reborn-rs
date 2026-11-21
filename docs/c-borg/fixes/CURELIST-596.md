# source596: do not try cures that do not cure the condition (patch 0134)

Consumer adoption of [source 596](https://github.com/diegoami/neo-angband-tracker/issues/596),
source pair [#613](https://github.com/diegoami/neo-angband-tracker/issues/613),
fix `ac214e76b` (branch `task/curelist-596`), kit `docs/audit/c-reports/curelist-596` from the
evidence branch `task/curelist-596-evidence` (generators byte-identical to the fixtures run-state
pin `c74322650`; the captured artifacts were added by later evidence-branch commits).

## Change

`borg_recover()` ate Elvish Waybread (or activated it) to cure blindness, and activated HEAL1 or
HEAL2 to cure stun. Waybread cures only poison (object.txt), and the HEAL1 and HEAL2 activations
heal and cure cuts but not stun (activation.txt), so the Borg spent the turn and the item and
stayed blind or stunned, before trying a cure that works. Drop those entries from the blindness
and stun lists.

Source `ac214e76b` -> applied `b533e8d65` (`cherry-pick -x`), patch `patches/0134-cure-lists.patch`,
scope `src/borg/borg-recover.c` only (5 insertions, 7 deletions; the module carried no earlier
consumer patches, so the source hunks apply at their recorded lines).

## Verification

Before commit `f1ff6c801` (the kit commit, `src/` identical to `origin/main`), tip `08859cc36`, kit
unchanged (`tools/verify-curelist596.py`). Native gates green at both (965/965 unit checks and 5/5
frontend tests at the before commit and at the tip). Byte-wise provenance: the source diff applied
to the before module equals the product module, and the `src`/`lib`/build scope of the fix and of
before..tip is the one file.

| Mode | Before | After |
| --- | --- | --- |
| `blind` | fails: the blind Borg eats the Waybread 25/100 times and never reaches the Cure Light Wounds potion (`eats_waybread=25`, `quaffs_clw=0`) | passes: the Waybread entry is gone and CLW is reached 25/100 (`eats_waybread=0`, `quaffs_clw=25`) |
| `control` | passes: not blind, CLW 25/100 | passes, same outcome, **byte-identical** |

The committed `before-*.log` / `after-*.log` are the source lane's captures. All four product
logs are **byte-identical** to them (both phases, both modes) and the OUTCOME rows match exactly;
the per-case comparison is recorded in `product-result.json`, and the source's own rows and hashes
are in `receipt.json`, carried unchanged.

## Mode attribution

Per the 2026-11-10 owner rule, each configuration is stated. `borg_recover()` runs in every
configuration — unrestricted (all modes), munchkin/Lunal-only and normal-only — whenever the Borg
is blind, stunned, confused, frightened or poisoned, so the changed branches execute in all of
them; the kit shows the changed branch executing (the blind mode's Waybread entry drops out and
the corrected list reaches CLW) and records the control where the corrected list is unchanged.
The per-mode cohort effect of not wasting turns and items on ineffective cures is **not measured**,
not neutral; the batch measurement with explicit arms is owed to borg-zoo issue 54. Not exercised
by the kit: the stun half (the same list pattern; HEAL1/HEAL2 removal).

Matrix group `curelist-596` (2 cases) uses the `after-*.log` rows as its baseline and is
byte-identical in both cases. Full matrix at this tree: 112 groups / 350 cases, `ok: true`, zero
failed cases, 286/350 byte-identical; the 24 byte differences are the same known ones as before
this carry (13 in the six pre-existing groups, dangerfull-69, teleotier-176, wandnames-38,
healamount-592) — no new difference.
