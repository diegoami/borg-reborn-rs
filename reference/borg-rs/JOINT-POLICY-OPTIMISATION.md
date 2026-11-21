# Choosing trip and shopping settings from full-suite results

This experiment tests **borg-rs on the accepted milestone-12 engine**.
Every configuration plays the same 45 starting cases: nine actual classes,
five cases per class, with a 100,000-turn limit. Character-menu selection is
explicit so Ranger and Blackguard cases really use those classes.

The owner's ranking is **fewer deaths first**. Ties are resolved by mean
final character level, then mean deepest level, then the number reaching
dungeon level 3 or deeper. Means include all 45 characters, including those
that died. A failed, timed-out or key-limited case makes its configuration
ineligible for ranking; surviving because execution stalled does not win.

## Result: a candidate that repeated on fresh seeds

The best tested candidate is **provision for twice the forecast risk**.
The inventory planner uses `borg_town_risk=2000` instead of 1000. Both new
policies are enabled, with the normal return thresholds and other shopping
priorities retained.

This tells the shopper to budget for a more demanding trip. It does not
make combat decisions more reckless. The observed tradeoff is fewer deaths
with lower average level/depth on the fresh cases. Fewer deaths is the
priority you selected.

### Check on 45 fresh cases

| Settings | Deaths | Mean character level | Mean deepest level | Reached DL3+ |
|---|---:|---:|---:|---:|
| Provision for twice the forecast risk | 9 | 4.04 | 2.04 | 6 |
| Existing policies | 11 | 4.22 | 2.40 | 11 |
| Combine all compatible search improvements | 12 | 3.78 | 2.31 | 6 |

The complicated combination that looked best during the search did not
repeat its result. It is not the selected candidate. The simpler risk
setting had 8 deaths against the control's 12 in the search, then 9 against
11 on fresh cases.

The fresh-case comparison is mixed at the individual-character level:
nine paired cases survived only with the candidate, while seven survived
only with the control. The net difference is two fewer deaths. This is the
best of the tested settings, not proof of a universally safer policy.

### The full search: 45 cases for every row

| Settings | Deaths | Mean character level | Mean deepest level | Reached DL3+ |
|---|---:|---:|---:|---:|
| Combine all compatible search improvements | 7 | 4.24 | 2.16 | 4 |
| Provision for twice the forecast risk | 8 | 4.18 | 1.98 | 6 |
| Spend more readily | 10 | 4.20 | 1.93 | 7 |
| Provision for a 3,000-action trip | 12 | 4.69 | 2.38 | 8 |
| No purchases solely for learning | 12 | 4.49 | 2.67 | 12 |
| Combine the two best individual changes | 12 | 4.24 | 2.13 | 11 |
| Existing policies | 12 | 4.04 | 2.13 | 9 |
| Plan two levels shallower | 12 | 3.67 | 1.64 | 2 |
| Both new policies, initial values | 13 | 4.18 | 2.40 | 7 |
| Come home earlier | 14 | 4.11 | 2.36 | 6 |
| Double healing priority | 15 | 3.98 | 2.04 | 7 |
| Keep more gold | 16 | 3.91 | 2.02 | 7 |

There were **675 complete games**: twelve configurations on the first
45 cases, then the control and two selected candidates on another 45.
Every case ended at its turn bound or death; none was ranked from a partial,
stalled or timed-out game. The control matched C on keys, messages and state
in all 45 original cases.

## Exact candidate settings

The essential settings are:

```text
borg_trip_policy = 1
borg_town_plan = 1
borg_town_risk = 2000
```

The trip still uses a depth margin of 0, returns with 2 free spaces or
4 unknown gear stacks, and has the purse-return trigger disabled. Gold
retention stays at weight 25, healing at 100 and learning at 25. Trip
length is estimated automatically. The
[complete candidate dictionary](validation/2026-11-14-joint-optimisation/candidate-settings.json)
records every declared setting.

Both policies remain off in the shipped defaults. No default was changed
by this search. These results apply to this milestone-12 experiment; they
are not measurements of the earlier pre-milestone-12 trip branch.

## What was searched



The new trip and inventory policies act in every candidate. The control
keeps both off. Settings not named in a row remain at the reference values.

| Choice | Reference | Alternative tested |
|---|---:|---:|
| Return with this many free pack spaces | 2 | 4 |
| Return with this many unknown gear stacks | 4 | 2, paired with the earlier pack return |
| Stay this many levels above prepared depth | 0 | 2 |
| Priority for retaining gold | 25 | 10 and 50 |
| Priority for healing supplies | 100 | 200 |
| Priority for learning through purchases | 25 | 0 |
| Forecast risk multiplier | 1 | 2 |
| Expected trip length | automatic | 3,000 player actions |

The first round contains the old-policy control, the live reference and
eight changes. The earlier-return change moves the two return thresholds
together; other probes change one parameter each.

Next, changes that improve the reference are combined, avoiding conflicting
values of the same parameter. At most two combinations are tested on the
entire 45-case suite. Interactions are measured; their benefits are not
assumed to add together.

The best two complete live configurations and the control are then tested
on 45 different starting seeds. Those confirmation results are reported
separately. A search result is not automatically a new default.

## Reproduce the search

```bash
python3 tools/optimise_joint_policy.py --out /tmp/joint-m12-search \
  --jobs 8 --c-baseline /tmp/m12-c-control
```

Build both backends first. The optional C baseline is produced with
`BORG_IMPL=c bash harness/suite.sh full-policy /tmp/m12-c-control`.
The optimiser checks the control's keys, messages and state against it
before testing the live candidates. Each game also has a 50,000-key and
180-second bound. Raw runs stay outside Git.

The experiment is isolated on `experiment/joint-policy-m12`, based on the
trip/inventory branch plus the accepted milestone-12 snapshot. The earlier
published trip branch is left unchanged. The [complete results](validation/2026-11-14-joint-optimisation/results.json)
include each case, its settings, actual class/race, source/build identity
and channel hashes.


## Limits

This is a bounded search over the listed values, not an exhaustive search
of every trip/inventory parameter or a separate optimisation for each class.
Each phase has only five cases per class. Longer games were not evaluated.
The normal full-suite stop is checked at key boundaries, so a single action
can carry a run beyond the nominal 100,000-turn limit; all rows use the same
contract. The published trip branch was left unchanged while this
experiment was run separately.

The milestone-12 merged build passes 255 release tests (four ignored) and
clippy. Three optimiser tests check survival-first ranking, inclusion of
dead characters in means, and rejection of incomplete suites. Raw runs are
outside Git under `/tmp/borg-joint-m12-search/`; no saves or binaries are
included in the report.
