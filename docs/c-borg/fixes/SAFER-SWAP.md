# Safer available emergency swap — consumer4

Canonical finding: [source82 / CBUG-0066](https://github.com/diegoami/neo-angband-tracker/issues/82).
Discovery credit borg-rs; native correction/evidence Diego Amicabile.
The separate source lane retains investigation ownership. Official6809 was
read-only checked open/unmerged; no official message or submission was made.

The cumulative C-only change is imported from reviewed head
`ebcd255ba9064fc44a35d693f42f9df3dc00bd88`, based on pristine
`e473720ed88a0f0c7ca66815cef3000eb4a38bc3`. Original commits66e10121,
017ba47e and ebcd255b are preserved with cherry-pick -x as9c10b9b6,
5aeae57a and39986cc5. The latter two clean the source comment. The final product
C file is byte-identical to submitted ebcd255b, SHA256
`7f6d4eb2e5e33139f23786a2cf51031418f5d3c214a78eb1723aa203c1acb750`.
The older96dd28c7 hash belongs to017ba47e's comment revision, not final ebcd255b.

The patch keeps the lower predicted danger among available backups, the existing
weapon preference on ties and the existing acceptance thresholds. Availability
guards matter: the source's tested simple reversal fails both single-backup
controls with an absent-item command. No broader equipment strategy is added.
The three-commit patch copy and source/applied pins are in patches/series.json.
It remains an active local carry; upstream incorporation/rebuild/verification is
required before fixed/closed. PR review/merge remains distinct from local proof.

## Verification

Unchanged source fixture:
`1b67370d0dc56d9b7ef34e5356e63d21a7095545`,
`docs/audit/c-reports/CBUG-0066/expanded/scenario.c`.
The runner relocates only absolute output/profile directory strings. Items,
monsters, seed1000, assertions, native Borg danger/selection, wield and native
spell damage are unchanged. It links the **actual product core object files**
from the normal native build, rather than a copied comparison helper. The
baseline uses pristine core objects and the same native test helpers/data.
Runtime src/lib/CMake diff was checked: only borg-item-wear.c differs.

| Scenario | Baseline HP lost | Fixed HP lost |
| --- | --- | --- |
| Frost armour better | 79 | 26 |
| Frost weapon better | 79 | 26 |
| Fire armour better | 88 | 29 |
| Electric weapon better | 79 | 26 |
| Two threats, both helpful | 113 | 59 |

Five protective-choice cases fail before/pass after; **9/9 pass after**. Armour
only, weapon only, ties and no benefit are byte-identical controls. The two-threat
case starts450HP and ends337 versus391: old selection chooses dagger danger174
instead of shield120 while current danger is239. These are captured native
results, not an invented damage model. The fixture constructs the character,
items and arena and schedules attacks; it isolates this emergency decision,
not the full caution/healing/escape loop or autonomous encounter frequency.

Normal native gate on applied39986cc52: **965/965 unit +5/5 frontend**, exit0.
A first scenario baseline attempt failed to locate constants.txt because its
working directory lacked data; that harness failure is retained. The corrected
18-case execution and four byte-identical controls all passed. No assertions,
seeds or thresholds were weakened. The gate's runtime C source is identical to
the final branch, so no redundant full local gate was added for ledger/docs.

The frozen three-seed ordinary-play comparison is in BASELINE-V1.md and
[raw receipt](https://github.com/diegoami/better-borg-c/issues/3#issuecomment-6682135753).
All three fixed starts/save hashes and entire traces match baseline; all reach
the20000-turn cap. Maximum depth3 cannot exercise the depth50 swap prerequisite.
No survival or wall-time improvement is demonstrated by this small batch. A
changed protective choice in a later encounter could change subsequent RNG
consumption; paired starts would not imply identical later encounters.

Copyable reproduction from this branch (native prerequisites in README):

```sh
git fetch source-tracker task/cbug-0066-evidence
# Runner reads immutable1b67370d, never mutable FETCH_HEAD.
flock /tmp/angband-validation.lock python3 tools/measure-native.py prepare --output artifacts/measurement/pristine
flock /tmp/angband-validation.lock python3 tools/verify-native.py
flock /tmp/angband-validation.lock python3 tools/verify-swap.py --baseline-build artifacts/measurement/pristine/build --output artifacts/swap
flock /tmp/angband-validation.lock python3 tools/measure-native.py prepare --ref HEAD --observe --output artifacts/measurement/fixed
flock /tmp/angband-validation.lock python3 tools/measure-cohort.py --build artifacts/measurement/fixed --output artifacts/measurement/cohort-fixed --phase fixed
```

If the named source remote is absent in a fresh clone, add it read-only first:
`git remote add source-tracker https://github.com/diegoami/neo-angband-tracker.git`
and `git remote set-url --push source-tracker DISABLED`.
The fixture is source-owned and fetched by immutable evidence pin. Generated
logs, machine results and saves stay ignored and are archived outside Git.
