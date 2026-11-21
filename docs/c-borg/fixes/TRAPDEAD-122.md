# source122: remove the unreachable visible-trap case from dark-flow exploration (patch 0127)

Consumer adoption of [source 122](https://github.com/diegoami/neo-angband-tracker/issues/122),
fix `b53fa471a` (branch `task/trapdead-122`), kit `docs/audit/c-reports/trapdead-122` from the evidence branch `task/trapdead-122-evidence`.

## Change

Remove the unreachable visible-trap case from dark-flow exploration.

borg_flow_dark_interesting() returns early for every grid below
FEAT_SECRET except closed doors.  Its "Explore visible traps" case tests
feat_is_trap_holding(ag->feat), and in the stock terrain data only the
open floor carries the TRAP flag.  The floor is below FEAT_SECRET, so the
test is never reached with a grid it could accept, and the borg keeps
known traps in ag->trap rather than in the feature anyway.

Delete the case and the "visible trap" wording in the function comment.
No behaviour change: traps lying on a chosen route are still checked by
the flow spread and disarmed by the step code.

Source `b53fa471a` -> applied `4a532c869` (`cherry-pick -x`), patch `patches/0127-dark-flow-trap-case.patch`, scope `src/borg/borg-flow-dark.c` only.

## Verification

Before commit `a5582047f` (the kit commit, `src/` identical to `origin/main`), tip `4a532c869`, kit
unchanged (`tools/verify-trapdead122.py`). Native gates green at both; byte-wise provenance over the source diff; the
`src`/`lib`/build scope is the one file. This is a behaviour-preserving change, so the kit has no
fail-before mode: every mode passes in both phases with the recorded outcome.

| Mode | Before | After |
| --- | --- | --- |
| `known` | passes, `{"flowed": 0, "key": 0, "step_x": -1, "step_y": -1, "step_is_trap": 0}` | passes, same outcome, **byte-identical** |
| `unknown` | passes, `{"flowed": 1, "key": 68, "step_x": 11, "step_y": 12, "step_is_trap": 1}` | passes, same outcome, **byte-identical** |
| `door` | passes, `{"flowed": 1, "key": 52, "step_x": 9, "step_y": 12, "step_is_trap": 0}` | passes, same outcome, **byte-identical** |

The committed `before-*.log` / `after-*.log` are the source lane's captures. All product logs are byte-identical to the source lane's captures. The
source's own rows and hashes are in `receipt.json`, carried unchanged.

## Mode attribution

Per the 2026-11-10 owner rule, each configuration is stated. This is a behaviour-preserving cleanup
(removes code the source shows is unreachable or a no-op), so no configuration (unrestricted, munchkin/Lunal-only, normal-only) can observe a
difference. That rests on the source's reachability argument above and on the kit's identical
before/after outcomes; it is **not** a cohort measurement.

Matrix group `trapdead-122` (3 cases) uses the `after-*.log` rows as its baseline.
