# source126: remove an unreachable door-count test from borg_play_step() (patch 0128)

Consumer adoption of [source 126](https://github.com/diegoami/neo-angband-tracker/issues/126),
fix `f6f0d37cd` (branch `task/doorcap-126`), kit `docs/audit/c-reports/doorcap-126` from the evidence branch `task/doorcap-126-evidence`.

## Change

Remove an unreachable door-count test from borg_play_step().

On breeder levels borg_play_step() closes an adjacent open door before
moving.  Its scan skipped every door when track_door.num >= 255, under
the comment "Skip repeatedly closed doors".  track_door holds 100
entries and its count only grows while it is below that size, so the
test is never true; it is also a count of the whole list, not a test of
the door being looked at.

Remove the test and its comment.  No behaviour change.

Source `f6f0d37cd` -> applied `e99bf2a85` (`cherry-pick -x`), patch `patches/0128-door-count-test.patch`, scope `src/borg/borg-flow.c` only.

## Verification

Before commit `9dfd445af` (the kit commit, `src/` identical to `origin/main`), tip `e99bf2a85`, kit
unchanged (`tools/verify-doorcap126.py`). Native gates green at both; byte-wise provenance over the source diff; the
`src`/`lib`/build scope is the one file. This is a behaviour-preserving change, so the kit has no
fail-before mode: every mode passes in both phases with the recorded outcome.

| Mode | Before | After |
| --- | --- | --- |
| `onedoor` | passes, `{"close": 1, "door_x": 10, "door_y": 11, "noted": 1}` | passes, same outcome, **byte-identical** |
| `twodoors` | passes, `{"close": 1, "door_x": 10, "door_y": 13, "noted": 1}` | passes, same outcome, **byte-identical** |
| `full` | passes, `{"close": 1, "door_x": 10, "door_y": 11, "noted": 0}` | passes, same outcome, **byte-identical** |
| `nobreeder` | passes, `{"close": 0, "door_x": -1, "door_y": -1, "noted": 0}` | passes, same outcome, **byte-identical** |

The committed `before-*.log` / `after-*.log` are the source lane's captures. All product logs are byte-identical to the source lane's captures. The
source's own rows and hashes are in `receipt.json`, carried unchanged.

## Mode attribution

Per the 2026-11-10 owner rule, each configuration is stated. This is a behaviour-preserving cleanup
(removes code the source shows is unreachable or a no-op), so no configuration (unrestricted, munchkin/Lunal-only, normal-only) can observe a
difference. That rests on the source's reachability argument above and on the kit's identical
before/after outcomes; it is **not** a cohort measurement.

Matrix group `doorcap-126` (4 cases) uses the `after-*.log` rows as its baseline.
