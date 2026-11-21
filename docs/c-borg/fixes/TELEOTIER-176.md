# source176: remove the unreachable fail-rate tier from the Teleport Other defence (patch 0129)

Consumer adoption of [source 176](https://github.com/diegoami/neo-angband-tracker/issues/176),
fix `841cd99ee` (branch `task/teleotier-176`), kit `docs/audit/c-reports/teleotier-176` from the evidence branch `task/teleotier-176-evidence`.

## Change

Remove the unreachable fail-rate tier from the teleport other defence.

borg_defend_aux_tele_away() lowers the spell failure it accepts as the
danger rises: by 10 above three times the avoidance and by 5 above
twice the avoidance.  A third tier, reached through the else of the
"twice" test, raised the allowance by 5 when the danger was above five
halves of the avoidance.  A danger that is not above twice the
avoidance is never above five halves of it (the avoidance is never
negative), so that tier could never run.

Remove it.  No behaviour change.

Source `841cd99ee` -> applied `4ef1afdff` (`cherry-pick -x`), patch `patches/0129-tele-other-tier.patch`, scope `src/borg/borg-fight-defend.c` only.

## Verification

Before commit `672f9e05e` (the kit commit, `src/` identical to `origin/main`), tip `4ef1afdff`, kit
unchanged (`tools/verify-teleotier176.py`). Native gates green at both; byte-wise provenance over the source diff; the
`src`/`lib`/build scope is the one file. This is a behaviour-preserving change, so the kit has no
fail-before mode: every mode passes in both phases with the recorded outcome.

| Mode | Before | After |
| --- | --- | --- |
| `over3` | passes, `{"cast": 0, "keys": 0}` | passes, same outcome, **byte-identical** |
| `upper` | passes, `{"cast": 1, "keys": 10}` | passes, same outcome, **byte-identical** |
| `lower` | passes, `{"cast": 1, "keys": 10}` | passes, same outcome, **byte-identical** |
| `under2` | passes, `{"cast": 1, "keys": 10}` | passes, same outcome, **byte-identical** |

The committed `before-*.log` / `after-*.log` are the source lane's captures. 0 of 8 product logs are byte-identical to the source lane's captures; the others differ in  line (diagnostic values that other local carries change; the before and after product logs still match each other where the source's do). The
source's own rows and hashes are in `receipt.json`, carried unchanged.

## Mode attribution

Per the 2026-11-10 owner rule, each configuration is stated. This is a behaviour-preserving cleanup
(removes code the source shows is unreachable or a no-op), so no configuration (unrestricted, munchkin/Lunal-only, normal-only) can observe a
difference. That rests on the source's reachability argument above and on the kit's identical
before/after outcomes; it is **not** a cohort measurement.

Matrix group `teleotier-176` (4 cases) uses the `after-*.log` rows as its baseline.
