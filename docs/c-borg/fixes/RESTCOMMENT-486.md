# source486: explain why borg_check_rest() rates a monster at its own grid (patch 0130)

Consumer adoption of [source 486](https://github.com/diegoami/neo-angband-tracker/issues/486),
fix `622f62ddf` (branch `task/restcomment-486`), kit `docs/audit/c-reports/restcomment-486` from the evidence branch `task/restcomment-486-evidence`.

## Change

Explain why borg_check_rest() rates a monster at its own grid.

Before resting, borg_check_rest() refuses when a monster within four
grids is too dangerous.  It asks borg_danger_one_kill() for the danger
at the monster's own grid, not at the resting grid.  That is deliberate:
at the resting grid a melee monster three or four grids away scores
almost nothing, while at its own grid it is scored as if adjacent, which
is the danger it poses once it closes in.

Replace the "one call for dangers" comment with one that says so.
Comment only; no code change.

Source `622f62ddf` -> applied `08d83d341` (`cherry-pick -x`), patch `patches/0130-rest-check-comment.patch`, scope `src/borg/borg-flow-misc.c` only.

## Verification

Before commit `c962206e0` (the kit commit, `src/` identical to `origin/main`), tip `08d83d341`, kit
unchanged (`tools/verify-restcomment486.py`). Native gates green at both; byte-wise provenance over the source diff; the
`src`/`lib`/build scope is the one file. This is a behaviour-preserving change, so the kit has no
fail-before mode: every mode passes in both phases with the recorded outcome.

| Mode | Before | After |
| --- | --- | --- |
| `bear` | passes, `{"rest": 0}` | passes, same outcome, **byte-identical** |
| `bear-l20` | passes, `{"rest": 1}` | passes, same outcome, **byte-identical** |
| `asleep` | passes, `{"rest": 0}` | passes, same outcome, **byte-identical** |
| `wolf` | passes, `{"rest": 0}` | passes, same outcome, **byte-identical** |
| `orc` | passes, `{"rest": 0}` | passes, same outcome, **byte-identical** |

The committed `before-*.log` / `after-*.log` are the source lane's captures. All product logs are byte-identical to the source lane's captures. The
source's own rows and hashes are in `receipt.json`, carried unchanged.

## Mode attribution

Per the 2026-11-10 owner rule, each configuration is stated. This is a behaviour-preserving cleanup
(comment-only; compiled code unchanged), so no configuration (unrestricted, munchkin/Lunal-only, normal-only) can observe a
difference. That rests on the source's reachability argument above and on the kit's identical
before/after outcomes; it is **not** a cohort measurement.

Matrix group `restcomment-486` (5 cases) uses the `after-*.log` rows as its baseline.
