# source111: remove the dead mana-cost test from the Venom preparation (patch 0125)

Consumer adoption of [source 111](https://github.com/diegoami/neo-angband-tracker/issues/111),
fix `65aa5a6d4` (branch `task/venomcost-111`), kit `docs/audit/c-reports/venomcost-111` from the evidence branch `task/venomcost-111-evidence`.

## Change

Remove the dead mana-cost test from the venom preparation.

borg_perma_aux_venom() was copied from the Smite Evil routine and still
read the cost of SMITE_EVIL.  Only the Blackguard has Venom and the
Blackguard has no Smite Evil, so the cost came back as -1.  The test
"cost >= current spell points / 10 (or / 7)" could then only refuse the
cast with negative spell points, while borg_spell_okay_fail(VENOM) just
above already requires the spell points to cover Venom's cost of 18.
The test always passed.

Remove the cost lookup and the test, keeping the current behaviour: the
Blackguard prepares Venom without a mana-cost limit.  No behaviour
change.

Source `65aa5a6d4` -> applied `acfa2ea4b` (`cherry-pick -x`), patch `patches/0125-venom-cost-dead-test.patch`, scope `src/borg/borg-fight-perm.c` only.

## Verification

Before commit `20e52b600` (the kit commit, `src/` identical to `origin/main`), tip `acfa2ea4b`, kit
unchanged (`tools/verify-venomcost111.py`). Native gates green at both; byte-wise provenance over the source diff; the
`src`/`lib`/build scope is the one file. This is a behaviour-preserving change, so the kit has no
fail-before mode: every mode passes in both phases with the recorded outcome.

| Mode | Before | After |
| --- | --- | --- |
| `unique` | passes, `{"cast": 1, "keys": 3, "no_rest_prep": 19000}` | passes, same outcome, **byte-identical** |
| `unique50` | passes, `{"cast": 1, "keys": 3, "no_rest_prep": 19000}` | passes, same outcome, **byte-identical** |
| `nounique` | passes, `{"cast": 0, "keys": 0, "no_rest_prep": 0}` | passes, same outcome, **byte-identical** |
| `poisoned` | passes, `{"cast": 0, "keys": 0, "no_rest_prep": 0}` | passes, same outcome, **byte-identical** |

The committed `before-*.log` / `after-*.log` are the source lane's captures. All product logs are byte-identical to the source lane's captures. The
source's own rows and hashes are in `receipt.json`, carried unchanged.

## Mode attribution

Per the 2026-11-10 owner rule, each configuration is stated. This is a behaviour-preserving cleanup
(removes code the source shows is unreachable or a no-op), so no configuration (unrestricted, munchkin/Lunal-only, normal-only) can observe a
difference. That rests on the source's reachability argument above and on the kit's identical
before/after outcomes; it is **not** a cohort measurement.

Matrix group `venomcost-111` (4 cases) uses the `after-*.log` rows as its baseline.
