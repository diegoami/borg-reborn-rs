# Native C impact v1 — accumulated external-review package

[Milestone1](https://github.com/diegoami/better-borg-c/milestone/1) collects this
batch; [review issue11](https://github.com/diegoami/better-borg-c/issues/11) owns
external findings. The owner directed merge, measure and continue, with external
review at an accumulated milestone. No external verdict has been supplied.
This milestone does not reinstate a per-PR independent-review gate.

## Integrated changes and exact pins

| Change | PR | Merge commit | Measured benefit |
| --- | --- | --- | --- |
| Native harness/frozen baseline | [5](https://github.com/diegoami/better-borg-c/pull/5) | b2c4aeb7a14dc77232ab71dad74df4c071b095c0 | Repeated starts/traces, failure accounting and explicit execution contract |
| Safer available emergency swap | [6](https://github.com/diegoami/better-borg-c/pull/6) | e4932f358beffcf9956b4e4b8d9e719c5086f664 | Two-threat native damage113→59HP; five affected cases corrected |
| Trace/watchdog safety | [9](https://github.com/diegoami/better-borg-c/pull/9) | 401583ac5258356f07ea61e85eeaa7387d898b05 | Partial records no longer abort supervision; reader failure reaps owned child |
| Knockout reaction reset | [10](https://github.com/diegoami/better-borg-c/pull/10) | d4bfdce9d4782cf83994ad52de707a52d1f31018 | Post-recovery accepted kill notification0→1; three identical controls |

Review the executable snapshot at **d4bfdce9d4782cf83994ad52de707a52d1f31018**.
Later milestone documentation/session links do not change runtime source.
Compared with pristine **e473720ed88a0f0c7ca66815cef3000eb4a38bc3**, runtime
changes are only the guarded swap comparison/comment in borg-item-wear.c and
one reset assignment in borg.c. Source proposals ebcd255b and6f463d01 are kept
as separate original patch bundles/applied commits in patches/series.json.
Both bundles round-tripped with git am onto pristine files and matched the
reviewed final blobs. Credits/licences/history remain intact.

Consumer4/source82 and consumer7/source14 stay **carried/open +patch-active**;
external review is separate from upstream resolution/incorporation/closure.
Official6809/6808 remain owner-submitted proposals. No official Angband message,
release/tag, deployment, new strategy, source-lane worktree edit or private-data
operation was performed.

## Measured impact and limits

Controlled native source82 comparison: **5 failures before/pass after,9/9 after
passes,4 byte-identical controls**. Damage79→26HP in both frost directions,
88→29 fire,79→26 electricity and113→59 with two threats. Items, monsters,
seed1000, assertions, thresholds and attacks unchanged. Synthetic construction
and scheduled attacks isolate this emergency decision; no encounter-frequency
or autonomous mortality claim. See SAFER-SWAP.md.

Controlled source14 comparison adds the reset to an already swap-corrected
consumer. Knockout fails before/pass after, **4/4 after pass,3 byte-identical
controls**. Recovery at turn511/20HP, same kobold killed at519, accepted kill
notification0→1. This restores lost combat information; no worse autonomous
move or player-death consequence is established. Fixture deaths=1 counts a
monster, and the player-alive assertion passes. See KNOCKOUT.md.

Frozen independent **n=3**, seeds11/29/47, ordinary Human Warrior/default point
buy, engine turn20000, wall120seconds/stall30seconds. Baseline repetitions are
reproducibility controls. Actual merged swap-only and cumulative builds were
rebuilt and run after integration with the exact same cohort. All starts and
complete decision/state traces match pristine baseline. No knockout occurred;
max depth3 cannot exercise depth50 emergency swaps. **No gameplay survival
improvement or runtime speedup is demonstrated by this small shallow cohort.**

| Seed | Baseline wall seconds (repeat1/2) | Actual merged swap wall seconds | Actual cumulative wall seconds | Outcome in all variants | Max depth | Level |
| --- | --- | --- | --- | --- | --- | --- |
| 11 | 15.268 /15.673 | 7.637 | 7.437 | Turn20000 cap, alive | 1 | 3 |
| 29 | 11.929 /11.428 | 6.430 | 6.232 | Turn20000 cap, alive | 3 | 3 |
| 47 | 8.779 /8.149 | 5.428 | 5.226 | Turn20000 cap, alive | 2 | 4 |

Each post-merge variant has3cap survivors,0deaths,0completions,0stalls,0crashes,
0harness failures. These repetitions are the same three starting conditions,
not extra independent samples. All observations are right-censored at the cap;
wall differences are not attributed to fixes. Turns, spent energy and Borg calls
remain distinct. Raw final RNG/character state and every result/trace/save hash:
[original baseline receipt](https://github.com/diegoami/better-borg-c/issues/3#issuecomment-6682135753)
and [actual post-merge JSON](https://github.com/diegoami/better-borg-c/issues/11#issuecomment-6682638064).

## Verification and retained failures

Actual cumulative main [hosted42201885117](https://github.com/diegoami/better-borg-c/actions/runs/42201885117)
at d4bfdce9 passed **16 tooling tests,965/965 unit +5/5 frontend**, numerical
logs inspected. Source82 main42201213151, watchdog main42201501479 and PR
heads42200616909/42201431003/42201832068 also completed SUCCESS. Harness PR5
42198817837 passed its then-current14 controls +965/965 +5/5. Harness main
42200462354 was cancelled during dependency installation by normal concurrency;
its native tests never ran. Earlier pending-check receipts were snapshots, not
retroactive green claims. Local product gates39986cc52 and7da46d00 independently
passed965/965 +5/5; no redundant full local gate was added after applicable green.

Original compile/path/startup errors and stalls are preserved. The first6+3
direct-save runs are **observer-affected diagnostics**, excluded from accepted
performance statistics: native saving compacts monsters and changed later play.
Forked snapshot children isolate that mutation; a complete20000-turn no-save
control matches after checkpoint-only records are removed. Birth and recent
living saves loaded; checkpoint19319 advanced normally to19320. Separate native
retirement tests dead/end capture but is not a combat death/cohort member.
Native dead saves omit dungeon records; no reloadable post-death encounter or
exact mid-run Borg-memory continuation is claimed. The new real-child watchdog
controls fail before (1error +1failure) and pass after; no orphan was left by
the negative tests. Seeds, caps, timeouts, conditions and assertions unchanged.

## Copyable reproduction

Use Git, Python3.12+, CMake/C toolchain, X11 development files, Perl and flock
on Linux/POSIX. The observed toolchain was GCC15.2.0/x86_64; binary hashes name
actual builds, not a cross-toolchain bit-for-bit promise. All outputs below are
new ignored disposable directories. From a fresh consumer clone:

```sh
git switch --detach d4bfdce9d4782cf83994ad52de707a52d1f31018
git remote add source-tracker https://github.com/diegoami/neo-angband-tracker.git
git remote set-url --push source-tracker DISABLED
git fetch source-tracker task/cbug-0066-evidence task/cbug-0002-evidence
flock /tmp/angband-validation.lock python3 tools/measure-native.py prepare --ref e473720ed88a0f0c7ca66815cef3000eb4a38bc3 --observe --output artifacts/review/baseline
flock /tmp/angband-validation.lock python3 tools/measure-cohort.py --build artifacts/review/baseline --output artifacts/review/baseline-runs --phase baseline
flock /tmp/angband-validation.lock python3 tools/measure-native.py prepare --ref e4932f358beffcf9956b4e4b8d9e719c5086f664 --observe --output artifacts/review/swap
flock /tmp/angband-validation.lock python3 tools/measure-cohort.py --build artifacts/review/swap --output artifacts/review/swap-runs --phase fixed
flock /tmp/angband-validation.lock python3 tools/measure-native.py prepare --ref d4bfdce9d4782cf83994ad52de707a52d1f31018 --observe --output artifacts/review/cumulative
flock /tmp/angband-validation.lock python3 tools/measure-cohort.py --build artifacts/review/cumulative --output artifacts/review/cumulative-runs --phase fixed
```

For actual-core controlled scenarios, retain a normal swap-only build and run
the cumulative native gate. The fixture runners use immutable source commits,
not FETCH_HEAD; only absolute output/profile paths change.

```sh
git worktree add --detach .worktrees/review-swap e4932f358beffcf9956b4e4b8d9e719c5086f664
(cd .worktrees/review-swap && flock /tmp/angband-validation.lock python3 tools/verify-native.py)
flock /tmp/angband-validation.lock python3 tools/verify-native.py
flock /tmp/angband-validation.lock python3 tools/measure-native.py prepare --ref e473720ed88a0f0c7ca66815cef3000eb4a38bc3 --output artifacts/review/pristine-plain
flock /tmp/angband-validation.lock python3 tools/verify-swap.py --baseline-build artifacts/review/pristine-plain/build --product-build .worktrees/review-swap/build/native --output artifacts/review/swap-scenarios
flock /tmp/angband-validation.lock python3 tools/verify-knockout.py --before-build .worktrees/review-swap/build/native --output artifacts/review/knockout-scenarios
python3 -m unittest discover -s tools -p 'test_*.py'
```

## Evidence preservation and external review

Generated runs/saves/builds are outside Git. Primary-checkout persistent archives:

- artifacts/measurement-v1/evidence.tar.gz:382verified members,
  SHA256 b8299f28db916ff99fc50c0f9e4dfde59593d45b5fe5e42920802115a9bc0987.
  Original full baseline, qualifications and failures; pristine source/binary
  retained separately read-only. Unmodified source archive SHA256
  6bdd13921470ea541f240456c5c98fc972970aa70d22478c040901bdcc589a5d.
- artifacts/impact-milestone1/evidence.tar.gz:101verified members,7,883,719bytes,
  SHA256 6ab845a6099bb43bd26577bc0bad22ceeaaead6a90d1f90efeba29940b8ad54c.
  Actual post-merge raw runs/saves/maps/input, observer binaries/build identities,
  knockout native outputs/gate, watchdog red/green controls and hosted main logs.

Each archive has a per-member SHA256 manifest. Verify its adjacent .sha256 file,
then extract into a new directory; preserve active games/profiles. These are
persistent local archives, not claimed hosted uploads. Public every-run JSON
and committed pinned reproduction above survive independently; no workstation
/tmp file is needed. Source/build trees are not archived as whole worktrees.

External review should check guarded candidate/tie/availability selection,
message-reset timing and paralysis control, declared observer neutrality,
outcome and child-lifetime accounting, attribution/removal units and statistical
limits. Record actual findings on issue11, and carry corrections through normal
separate PRs and unchanged controls. The next cycle should select a source-owned
verified measurable defect, not promote a parked lead or invent new strategies.
