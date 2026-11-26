# source69: state that borg_danger() always rates the full damage on purpose (patch 0124)

Consumer adoption of [source 69](https://github.com/diegoami/neo-angband-tracker/issues/69),
fix `8af235045` (branch `task/dangerfull-69`), kit `docs/audit/c-reports/dangerfull-69` from the evidence branch `task/dangerfull-69-evidence`.

## Change

Say that borg_danger() always rates the full damage on purpose.

borg_danger() sets its full_damage argument to true before adding up the
danger of each monster, so every caller gets the worst-case estimate
whatever it passes.  The reduced estimate in borg_danger_one_kill() is
still used, but only by callers that invoke it directly.

Add a comment saying the override is deliberate.  Comment only; no code
change.  Nothing becomes dead because of the override: the reduced paths
in borg_danger_one_kill() and borg_danger_physical() stay reachable
through those direct calls.

Source `8af235045` -> applied `565c7aa9f` (`cherry-pick -x`), patch `patches/0124-danger-full-comment.patch`, scope `src/borg/borg-danger.c` only.

## Verification

Before commit `c45a4b011` (the kit commit, `src/` identical to `origin/main`), tip `565c7aa9f`, kit
unchanged (`tools/verify-dangerfull69.py`). Native gates green at both; byte-wise provenance over the source diff; the
`src`/`lib`/build scope is the one file. This is a behaviour-preserving change, so the kit has no
fail-before mode: every mode passes in both phases with the recorded outcome.

| Mode | Before | After |
| --- | --- | --- |
| `bear` | passes, `{"grid": 0, "adj": 20, "same": 1}` | passes, same outcome, **byte-identical** |
| `wolf` | passes, `{"grid": 16, "adj": 48, "same": 1}` | passes, same outcome, **byte-identical** |
| `orc` | passes, `{"grid": 13, "adj": 22, "same": 1}` | passes, same outcome, **byte-identical** |
| `shaman` | passes, `{"grid": 22, "adj": 22, "same": 1}` | passes, same outcome, **byte-identical** |

The committed `before-*.log` / `after-*.log` are the source lane's captures. 0 of 8 product logs are byte-identical to the source lane's captures; the others differ in onekill line (diagnostic values that other local carries change; the before and after product logs still match each other where the source's do). The
source's own rows and hashes are in `receipt.json`, carried unchanged.

## Mode attribution

Per the 2026-11-10 owner rule, each configuration is stated. This is a behaviour-preserving cleanup
(comment-only; compiled code unchanged), so no configuration (unrestricted, munchkin/Lunal-only, normal-only) can observe a
difference. That rests on the source's reachability argument above and on the kit's identical
before/after outcomes; it is **not** a cohort measurement.

Matrix group `dangerfull-69` (4 cases) uses the `after-*.log` rows as its baseline.
