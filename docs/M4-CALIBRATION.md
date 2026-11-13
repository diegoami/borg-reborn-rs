# M4: threat calibration and the survival gate

*T4.7 (#124). The M4 calibration deliverable (ARCHITECTURE L4,
*Calibration*) and the M4 exit evidence (PLAN.md). Date: 2026-11-12.*

**Build and data.** Every figure below comes from the smoke and full suites
played by the build at commit `7570d10` (`source_sha256 f70934d39dd8…`,
`git_dirty=no`), default profile (`372785a3dfab`). That build adds only trace
fields to the M4 baseline (T4.6, references at `da71d05`). It reproduces all
three references byte for byte: smoke 3/3, smoke-exact 3/3, full 45/45. So
these are the M4 baseline's games, with a richer trace. The tables are
`tools/calibrate.py harness/out/full harness/out/smoke` (smoke-exact is left
out because it repeats smoke). Rerun them with that command after any
regeneration of the references.

## Summary

- **The threat model over-predicts damage by about ten times, and the error is
  in blow frequency, not blow size.** Over the 11,519 windows with a
  prediction, the Borg took 432 hit points against 4,954 expected: a ratio of
  0.087 (90% interval 0.07–0.10). Split by blow:
  - **The per-blow model is calibrated.** Where one awake monster stood
    adjacent and in view and the Borg stayed on its grid, the model expected
    1.70 HP per predicted blow (623 HP over 367 blows). The engine's messages
    show 1.77 HP per blow that came (189 HP over 107 blows): hit rate 0.92,
    1.93 HP per hit. Hit chance, dice and armour are right.
  - **The missing hit points are blows that never came.** In 123 of those
    256 windows the Borg's own blow killed the monster or made it flee before
    it acted. In the rest, the engine printed 0.58 of the predicted blows.
    The assessment asks what standing here would cost if the Borg did
    nothing; the Borg always does something.
- **Worst case is an honest upper bound for what the model covers.** It was
  exceeded in 16 of 11,519 windows (0.14%). Every exceedance comes from
  something outside the model: poison ticks, a second unseen attacker, hunger
  fainting, or a Walk that ended beside a monster other than the one
  assessed.
- **p_death separates fatal windows.** 11 of the 16 deaths happened in the
  64 windows (0.02% of all windows) with p_death ≥ 0.2, and none in the
  10 windows between 0.001 and 0.2. In those 64 windows the model says 0.80
  and the observed rate is 0.17: it over-states death as it over-states
  damage.
- **The other 5 deaths were invisible to the model by construction.** Two
  were starvation, one a poison tick, one an unseen biter ("Something bites
  you") while resting, and one a floating eye. The eye's gaze has no damage
  dice in `monster.txt`, but the engine deals at least 1 HP per paralysing
  blow to a character already paralysed (`mon-blows.c:1019-1021`, "Prevent
  perma-paralysis via damage").
- **In 11 of 16 deaths the model gave the warning and the Borg fought.**
  The fatal think had p_death between 0.38 and 1.0, and the choice was Attack
  (10) or Quaff (1). That is the tactical layer's decision with the means it
  had (#127 owns class means; #137 owns refused Engage actions), not the
  model's blindness.
- **Survival gate (descriptive).** 30/45 full runs survive the cap (67%).
  The zoo's C Borgs survive 66% (4.2.6) and 69% (better-borg-c m7) of
  `classes-1800`. Our Borg reaches that rate by not progressing: median max
  depth 1 and character level 1, where the C Borgs reach depth 1–4 and level
  1.5–7 by class. That is M5's ground (T5.1 readiness). Different seeds and
  suite sizes make this a description, not a G1 comparison.

## Method

A **window** is one think that took game time, paired with what happened
before the next think at a later game turn. Windows are cut on the engine's
key stream, not on turns: a monster with more energy than the character acts
at the next turn *before* the next key (`game-world.c:run_game_loop`,
"Process monster with even more energy first"), and the message log stamps
each message with the key whose processing printed it. A window holds:

- the **prediction**: the last trace line at the window's turn, i.e. the
  assessment at the character's grid over `threat_horizon` (1) player turns:
  `expected`, `worst`, `p_death`, `disabling`, and (new in this task)
  `sources`, one line per contributing monster (race, distance, speed,
  sleep, confusion/stun, in view, same-race group, rounds, expected, worst,
  disabling, ranged);
- the **outcome**: every hit-point drop between the window's first key and
  the next window's first key (healing is not netted against damage), plus
  the drop to the final hit points for the window that died; the window's
  messages; whether a disabling effect began (the onset texts of
  `player_timed.txt`), with hunger's faint counted apart.

**What a window does not control.** The prediction is for standing on the
grid. The Borg acts in every window, so the outcome depends on its choice
(reported by choice below, and pooled for choices that keep the character on
the grid). A Walk may end beside a different monster, so Walk windows cannot
calibrate a single source. At other speeds the window is not the horizon.
Damage from no monster (hunger, poison and cut ticks, traps) lands in windows
with no source and is reported as unattributed. **Uncertainty** is a
percentile bootstrap over windows (1,000 resamples, seed 0). It treats
windows as independent, and windows of one fight are not, so the intervals
are narrower than the truth.

## Results

Windows: 352417; threatened (expected or worst > 0): 11519; deaths: 16.

| pool | windows | predicted | actual | actual/predicted (90% CI) | hit rate | over worst |
| --- | ---: | ---: | ---: | --- | ---: | ---: |
| all threatened | 11519 | 4954.4 | 432 | 0.087 (0.07–0.10) | 0.016 | 16 |
| one source | 1980 | 1467.5 | 211 | 0.144 (0.11–0.17) | 0.045 | 11 |
| standing still (no Walk, stairs, Alter) | 1151 | 3070.2 | 383 | 0.125 (0.11–0.14) | 0.142 | 7 |

Quiet windows (no prediction): 340898, 61 with damage, 215 HP in all, 4 deaths (unattributed).

**By predicted expected damage**

| expected bin | windows | mean expected | mean worst | mean actual | actual/predicted (90% CI) | hit rate | over worst |
| --- | ---: | ---: | ---: | ---: | --- | ---: | ---: |
| [0, 0.5) | 9748 | 0.05 | 0.07 | 0.00 | 0.006 (0.00–0.01) | 0 | 3 |
| [0.5, 1) | 504 | 0.72 | 1.07 | 0.03 | 0.041 (0.02–0.06) | 0.022 | 4 |
| [1, 2) | 492 | 1.41 | 2.00 | 0.16 | 0.116 (0.09–0.14) | 0.126 | 9 |
| [2, 4) | 370 | 2.95 | 4.88 | 0.14 | 0.049 (0.03–0.07) | 0.068 | 0 |
| [4, 8) | 374 | 5.42 | 10.96 | 0.61 | 0.113 (0.09–0.14) | 0.179 | 0 |
| [8, inf) | 31 | 10.17 | 17.76 | 1.65 | 0.162 (0.12–0.20) | 0.677 | 0 |

**p_death against deaths in the window** (Brier score 0.000121)

| p_death bin | windows | mean p_death | deaths |
| --- | ---: | ---: | ---: |
| [0, 0.001) | 352343 | 0.0000 | 5 |
| [0.001, 0.01) | 1 | 0.0062 | 0 |
| [0.01, 0.05) | 3 | 0.0399 | 0 |
| [0.05, 0.2) | 6 | 0.1444 | 0 |
| [0.2, 1) | 64 | 0.7981 | 11 |

**Disabling (fear, blindness, confusion, paralysis) against onset messages**

| disabling bin | windows | mean predicted | windows with an onset | windows with a hunger faint (apart) |
| --- | ---: | ---: | ---: | ---: |
| [0, 0.01) | 352119 | 0.0000 | 6 | 321 |
| [0.05, 0.2) | 239 | 0.1365 | 1 | 0 |
| [0.2, 0.5) | 49 | 0.2927 | 2 | 0 |
| [0.5, 1) | 10 | 0.9636 | 3 | 0 |

**Blows: one awake adjacent source in view, the character standing still** (blows from `monster.txt`; hits and misses from the engine's messages)

| monster | windows | blows/round | predicted blows | observed blows | observed/predicted | hit rate | damage per hit (dice mean) | predicted HP | actual HP |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | --- | ---: | ---: |
| scrawny cat | 55 | 1 | 59.9 | 38 | 0.634 | 0.921 | 0.94 (1) | 56.8 | 33 |
| white worm mass | 28 | 1 | 14.0 | 4 | 0.286 | 1 | 3.75 (1.5) | 26.8 | 15 |
| giant white mouse | 18 | 1 | 20.8 | 0 | 0 | – | – (1.5) | 43.1 | 0 |
| fruit bat | 15 | 1 | 34.6 | 5 | 0.145 | 0.8 | 1 (1) | 30.2 | 4 |
| Fang, Farmer Maggot's Dog | 14 | 1 | 30.4 | 16 | 0.526 | 0.938 | 2.2 (2.5) | 67.1 | 33 |
| soldier ant | 12 | 1 | 15.3 | 2 | 0.131 | 0.5 | 2 (1.5) | 21.2 | 2 |
| white icky thing | 12 | 1 | 14.9 | 4 | 0.268 | 0.75 | 0.67 (1.5) | 20.2 | 2 |
| small kobold | 9 | 1 | 11.5 | 1 | 0.087 | 1 | 5 (3) | 31.5 | 5 |
| battle-scarred veteran | 8 | 1 | 8.0 | 5 | 0.625 | 1 | 4.2 (7) | 53.2 | 21 |
| giant yellow centipede | 7 | 2 | 17.8 | 4 | 0.225 | 1 | 1.75 (2) | 31.9 | 7 |
| rock lizard | 7 | 1 | 8.3 | 0 | 0 | – | – (1) | 7.4 | 1 |
| gallant | 6 | 2 | 10.8 | 4 | 0.37 | 1 | 4.75 (4) | 36.9 | 19 |
| large white snake | 6 | 2 | 6.0 | 0 | 0 | – | – (1) | 5.3 | 0 |
| wild cat | 6 | 2 | 25.6 | 8 | 0.312 | 1 | 2.12 (2) | 44.0 | 17 |
| clear icky thing | 5 | 1 | 5.7 | 0 | 0 | – | – (1.5) | 7.6 | 0 |
| giant white centipede | 5 | 2 | 11.4 | 0 | 0 | – | – (1.5) | 15.6 | 0 |
| green worm mass | 5 | 1 | 2.5 | 1 | 0.4 | 1 | 1 (2) | 7.1 | 1 |
| salamander | 5 | 1 | 6.0 | 1 | 0.167 | 1 | 3 (2) | 11.2 | 3 |
| squint-eyed rogue | 5 | 3 | 15.9 | 3 | 0.189 | 0.667 | 0 (1.17) | 16.8 | 0 |
| wild dog | 5 | 1 | 5.9 | 0 | 0 | – | – (1) | 5.3 | 0 |
| acolyte | 4 | 1 | 5.2 | 1 | 0.192 | 1 | 4 (3) | 13.1 | 4 |
| cave spider | 4 | 1 | 7.9 | 4 | 0.506 | 1 | 2.75 (2.5) | 17.2 | 11 |
| scruffy little dog | 4 | 1 | 4.0 | 1 | 0.25 | 0 | – (1) | 3.5 | 0 |
| Grip, Farmer Maggot's Dog | 3 | 1 | 6.9 | 1 | 0.145 | 1 | 4 (2.5) | 14.8 | 4 |
| **all** | 256 |  | 366.9 | 107 | 0.292 | 0.916 | 1.93 | 623.1 | 189 |

In 123 of these 256 windows the monster was slain or fled in the window. In the other windows the engine printed 103 blows against 177.7 predicted (ratio 0.58).

| distance (one source) | windows | predicted | actual | actual/predicted (90% CI) | hit rate | over worst |
| --- | ---: | ---: | ---: | --- | ---: | ---: |
| 1 | 724 | 1075.5 | 196 | 0.182 (0.14–0.22) | 0.112 | 5 |
| 2-3 | 1253 | 384.8 | 15 | 0.039 (0.02–0.07) | 0.006 | 6 |
| 4-7 | 1 | 2.4 | 0 | 0 | 0 | 0 |
| 8+ | 2 | 4.8 | 0 | 0 (0.00–0.00) | 0 | 0 |

| monster speed (one source) | windows | predicted | actual | actual/predicted (90% CI) | hit rate | over worst |
| --- | ---: | ---: | ---: | --- | ---: | ---: |
| fast (>110) | 754 | 773.8 | 124 | 0.16 (0.12–0.20) | 0.081 | 4 |
| normal (110) | 876 | 373.4 | 43 | 0.115 (0.06–0.18) | 0.013 | 1 |
| slow (<110) | 350 | 320.4 | 44 | 0.137 (0.07–0.22) | 0.049 | 6 |

| monster status (one source) | windows | predicted | actual | actual/predicted (90% CI) | hit rate | over worst |
| --- | ---: | ---: | ---: | --- | ---: | ---: |
| asleep | 80 | 47.7 | 0 | 0 (0.00–0.00) | 0 | 0 |
| awake, in view | 995 | 1224.1 | 208 | 0.17 (0.14–0.21) | 0.086 | 9 |
| remembered, out of view | 905 | 195.7 | 3 | 0.015 (0.01–0.03) | 0.003 | 2 |

| attack form (one source) | windows | predicted | actual | actual/predicted (90% CI) | hit rate | over worst |
| --- | ---: | ---: | ---: | --- | ---: | ---: |
| melee only | 1959 | 1423.2 | 202 | 0.142 (0.11–0.17) | 0.044 | 11 |
| ranged estimate | 21 | 44.3 | 9 | 0.203 (0.00–0.46) | 0.095 | 0 |

| sources in the window | windows | predicted | actual | actual/predicted (90% CI) | hit rate | over worst |
| --- | ---: | ---: | ---: | --- | ---: | ---: |
| 1 | 1980 | 1467.5 | 211 | 0.144 (0.11–0.17) | 0.045 | 11 |
| 2 | 393 | 610.2 | 66 | 0.108 (0.07–0.15) | 0.061 | 3 |
| 3+ | 9146 | 2876.7 | 155 | 0.054 (0.04–0.07) | 0.008 | 2 |

| same-race group | windows | predicted | actual | actual/predicted (90% CI) | hit rate | over worst |
| --- | ---: | ---: | ---: | --- | ---: | ---: |
| alone | 2127 | 1732.7 | 272 | 0.157 (0.13–0.19) | 0.051 | 14 |
| same-race group | 9392 | 3221.6 | 160 | 0.05 (0.04–0.06) | 0.009 | 2 |

| the Borg's choice | windows | predicted | actual | actual/predicted (90% CI) | hit rate | over worst |
| --- | ---: | ---: | ---: | --- | ---: | ---: |
| Alter | 342 | 9.7 | 0 | 0 (0.00–0.00) | 0 | 0 |
| Attack | 544 | 2032.2 | 350 | 0.172 (0.15–0.20) | 0.274 | 4 |
| Eat | 3 | 0.0 | 0 | 0 (0.00–0.00) | 0 | 0 |
| Get | 3 | 1.3 | 0 | 0 (0.00–0.00) | 0 | 0 |
| Hold | 529 | 822.6 | 20 | 0.024 (0.01–0.04) | 0.021 | 3 |
| Quaff | 66 | 207.9 | 9 | 0.043 (0.01–0.08) | 0.045 | 0 |
| Read | 124 | 377.4 | 2 | 0.005 (0.00–0.01) | 0.016 | 0 |
| Rest | 6 | 6.2 | 4 | 0.65 (0.00–1.00) | 0.167 | 0 |
| Walk | 9902 | 1497.0 | 47 | 0.031 (0.02–0.04) | 0.002 | 9 |

| monster (one source, ≥5 windows) | windows | predicted | actual | actual/predicted (90% CI) | hit rate | over worst |
| --- | ---: | ---: | ---: | --- | ---: | ---: |
| wild dog | 384 | 68.0 | 0 | 0 (0.00–0.00) | 0 | 0 |
| white icky thing | 268 | 37.6 | 3 | 0.08 (0.00–0.18) | 0.007 | 1 |
| scrawny cat | 180 | 66.7 | 33 | 0.495 (0.40–0.59) | 0.183 | 0 |
| giant yellow centipede | 154 | 135.9 | 7 | 0.051 (0.00–0.11) | 0.013 | 0 |
| giant white mouse | 142 | 117.9 | 1 | 0.008 (0.00–0.03) | 0.007 | 0 |
| fruit bat | 140 | 63.1 | 6 | 0.095 (0.02–0.18) | 0.029 | 1 |
| rock lizard | 85 | 35.4 | 2 | 0.056 (0.00–0.12) | 0.024 | 1 |
| soldier ant | 76 | 57.2 | 3 | 0.052 (0.00–0.12) | 0.026 | 1 |
| small kobold | 73 | 122.1 | 7 | 0.057 (0.00–0.14) | 0.027 | 1 |
| giant white centipede | 59 | 51.0 | 0 | 0 (0.00–0.00) | 0 | 0 |
| clear icky thing | 52 | 27.2 | 0 | 0 (0.00–0.00) | 0 | 0 |
| white worm mass | 43 | 36.5 | 15 | 0.411 (0.22–0.63) | 0.233 | 4 |
| Fang, Farmer Maggot's Dog | 39 | 119.6 | 33 | 0.276 (0.15–0.41) | 0.231 | 0 |
| giant black ant | 36 | 22.2 | 1 | 0.045 (0.00–0.14) | 0.028 | 1 |
| squint-eyed rogue | 28 | 32.7 | 0 | 0 (0.00–0.00) | 0 | 0 |
| Grip, Farmer Maggot's Dog | 21 | 49.5 | 4 | 0.081 (0.00–0.22) | 0.048 | 0 |
| blue yeek | 21 | 18.2 | 0 | 0 (0.00–0.00) | 0 | 0 |
| wild cat | 20 | 64.1 | 20 | 0.312 (0.05–0.60) | 0.15 | 1 |
| salamander | 18 | 19.7 | 3 | 0.152 (0.00–0.40) | 0.056 | 0 |
| scruffy little dog | 18 | 7.0 | 0 | 0 (0.00–0.00) | 0 | 0 |
| acolyte | 16 | 27.2 | 9 | 0.331 (0.00–0.69) | 0.125 | 0 |
| kobold | 15 | 23.4 | 0 | 0 (0.00–0.00) | 0 | 0 |
| battle-scarred veteran | 14 | 61.3 | 21 | 0.343 (0.11–0.61) | 0.286 | 0 |
| crow | 13 | 62.4 | 8 | 0.128 (0.00–0.29) | 0.154 | 0 |
| large white snake | 13 | 10.1 | 0 | 0 (0.00–0.00) | 0 | 0 |
| cave spider | 10 | 25.1 | 11 | 0.438 (0.10–0.71) | 0.3 | 0 |
| gallant | 9 | 40.7 | 19 | 0.467 (0.00–1.01) | 0.222 | 0 |
| aimless-looking merchant | 8 | 15.2 | 0 | 0 (0.00–0.00) | 0 | 0 |
| cutpurse | 7 | 10.2 | 3 | 0.295 (0.00–0.59) | 0.143 | 0 |
| apprentice | 6 | 17.5 | 0 | 0 (0.00–0.00) | 0 | 0 |
| green worm mass | 6 | 8.1 | 1 | 0.124 (0.00–0.35) | 0.167 | 0 |

## Reading the breakdowns

- **By range:** adjacent single sources run at 0.18 of the prediction; at
  distance 2–3 the ratio is 0.04. A monster two steps away is predicted to
  spend part of the turn attacking (`rounds = slots - (d - 1)`). In practice
  it spends the turn closing in, or does not come at all.
- **By status:** sleeping monsters got 48 HP predicted (the declared half
  share) and dealt 0 in 80 windows. Remembered monsters out of view dealt 3
  HP against 196. `Track::confidence` decays too slowly for one-turn damage,
  though it may still be right for paths, which look further ahead.
- **By speed:** fast, normal and slow sources all land between 0.12 and
  0.16. Speed is not where the error is. The `+3` speed the C Borg adds for
  characters under 20 maximum hit points (`borg-danger.c:2357-2359`) raises
  every prediction by about a third. Together with the erratic movers
  (`RAND_25`, `RAND_50` in `monster.txt`: bats, worm masses, mice), it
  explains much of the 0.58 blow rate among monsters that survived.
- **By group:** same-race groups run at 0.05, lone monsters at 0.16. The sum
  over a group (38 giant white mice in `full-2-druid-0`) counts every member
  as attacking, while only the few adjacent can.
- **By monster:** the closest calibrated sources are the scrawny cat (0.50;
  1d1, it rarely misses and rarely dies first), white worm mass (0.41,
  poison excluded) and cave spider (0.44). The most over-predicted are the
  wild dog (0 HP of 68 predicted, mostly at range 2–3), giant white mouse
  (0.008, killed first) and small kobold (0.057, killed first).

## Every natural death, with its trace

17 natural deaths in the runs: the 15 in full, the smoke mage, and the
smoke-exact mage, which repeats the smoke mage's death and has no dossier of
its own. `tools/reconcile-deaths.py --runs` finds 0 without a dossier. The
dossiers for this build are D1654–D1669. Their identity differs from the
T4.6 dossiers D1638–D1653 only by the source hash (the trace fields), so
they are the same games under a new build. **p_death** is at the fatal
think. **Warned** counts windows with p_death ≥ 0.2 among the last 15
windows.

| Dossier | Run | Cause | HP at the fatal think | p_death | warned | Choice | Reading |
| --- | --- | --- | ---: | ---: | ---: | --- | --- |
| D1654 | smoke-mage-2 | Fang | 1 | 1.00 | 4 | Attack | Warned; fought with no escape left (CLW drunk at 18355) |
| D1655 | full-1-mage-0 | a kobold | 3 | 0.71 | 3 | Attack | Warned; fought after two Quaffs |
| D1656 | full-0-warrior-0 | a crow | 7 | 0.00 | 0 | Rest | **Unseen attacker**: "Something bites you" across three rests; no source, no prediction |
| D1657 | full-1-mage-4 | poison | 2 | 0.00 | 0 | Walk | **Status damage**: a white worm mass's poison; the model's 1.16 HP excludes poison ticks |
| D1658 | full-2-druid-0 | a giant white mouse | 0 | 1.00 | 15 | Attack | Warned for 15 windows; a breeder swarm of 38 |
| D1659 | full-2-druid-1 | a small kobold | 4 | 0.38 | 2 | Attack | Warned |
| D1660 | full-2-druid-2 | a crow | 4 | 0.99 | 4 | Quaff | Warned; crow, Grip and an icky thing together |
| D1661 | full-3-priest-0 | a giant white mouse | 1 | 1.00 | 2 | Attack | Warned; breeder swarm of 29 |
| D1662 | full-3-priest-4 | a battle-scarred veteran | 6 | 0.69 | 7 | Attack | Warned; rested beside it between attacks |
| D1663 | full-4-necro-4 | a fatal wound | 0 | 1.00 | 1 | Attack | Warned; the cut (status damage) finished it |
| D1664 | full-5-paladin-2 | starvation | 1 | 0.00 | 0 | Rest | **Starvation**, no monster (T4.6's residue) |
| D1665 | full-4-necro-3 | a scruffy little dog | 0 | 1.00 | 2 | Attack | Warned; fainting from hunger beside two town animals |
| D1666 | full-5-paladin-3 | a scrawny cat | 0 | 0.95 | 2 | Attack | Warned; in town at 0 HP |
| D1667 | full-6-rogue-3 | a cutpurse | 3 | 0.89 | 1 | Attack | Warned one window ahead; two apprentices, two cutpurses |
| D1668 | full-7-ranger-0 | starvation | 1 | 0.00 | 0 | Rest | **Starvation**, no monster (T4.6's residue) |
| D1669 | full-7-ranger-4 | a floating eye | 1 | 0.00 | 0 | Rest | **Paralysis floor**: disabling 0.97 recorded; rested at 1/15 beside the eye; each gaze on a paralysed character deals ≥ 1 HP (`mon-blows.c:1019-1021`) |

**The first exhibit, revisited.** The smoke mage's grey-mold death at 9/9 HP
(D1602, `bd249a2`) belonged to T4.4's baseline. In the M4 baseline that seed
lives to turn 18,376 and dies to Fang (D1638/D1654). The exhibit's last 12
thinks all record `expected 0` before the death at turn 451, so it is the
"damage with no prediction" class this report counts. In the M4 baseline
that class is the four quiet deaths above, plus 61 quiet windows that took
215 HP, among them a grey mold's spores in `full-8-blackguard-0` at turn
8,559 (12 HP taken after a Walk; the mold is not in the walking think's
sources).

## The survival gate

| Criterion | Evidence | Result |
| --- | --- | --- |
| T4.1 death fixtures and owner-manual alternatives pass | `cargo test --workspace`: 21 active decision fixtures pass. The 3 gap fixtures (`bbc-d0010`, `bbc-d0011`, `bbc-d0014`) are pinned and owned by M5's #126/#127. No gap is owned by #124 | PASS |
| Deterministic replay | Two builds (`f1986c2`, `7570d10`) each reproduce smoke 3/3, smoke-exact 3/3 and full 45/45 byte for byte against the T4.6 references; the trace fields change no key | PASS |
| G0: no stall, crash or harness failure | smoke and smoke-exact with `STALL_TIMEOUT_SECONDS=10`: 6/6 processes `success`, none stalled or timed out; the 51 suite processes of the measured run all `success`; `tools/check-no-time.py`: 0 no-time budget violations in 51 traces | PASS |
| Every natural death recorded with its trace | 17 natural deaths, 0 without a dossier (`reconcile-deaths.py --runs`); 16 new dossiers D1654–D1669 with 200-line trace tails | PASS |
| Calibration report on recorded runs | This document; `tools/calibrate.py` and its tests | DONE |
| Early-suite deaths beside the zoo's C Borg results | Table below; descriptive only | DONE |

**Survival and progress together.** Full suite (9 classes × 5 seeds, cap
100,000 game turns) beside the zoo's existing results for the original C
Borg (`4.2.6`) and better-borg-c m7 (`4.2.6-bbc-m7`) on `classes-1800` (200
runs per class, same cap, profile `default`). Sources: borg-zoo
`reports/originals-20261108-classes-1800/42534228784-1/REPORT.md`
(4.2.6 deaths by class) and the `results` branch at `383d948`,
`results/classes-m7/42526892708-1/cells.jsonl` (both arms; medians computed
here from `final.max_depth` and `final.max_level`). The seeds and the
race/class assignment differ from ours, our sample is 5 per class, and
better-borg-c after `fd683ced` changes its default profile, so only m7 is
shown. **This is description, not G1** (GOALS: G1 is non-inferiority on
`classes-1800`, T5.4).

| Class | ours: alive at cap / 5 | ours: median max depth, clevel | C 4.2.6: deaths / 200 | C: median depth, clevel | bbc-m7: deaths / 200 | bbc-m7: median depth, clevel |
| --- | ---: | --- | ---: | --- | ---: | --- |
| Warrior | 4 | 1, 1 | 64 | 4, 7 | 50 | 3, 6 |
| Mage | 3 | 1, 1 | 62 | 1, 4 | 67 | 1, 4 |
| Druid | 2 | 1, 1 | 141 | 1, 1.5 | 128 | 2, 2 |
| Priest | 3 | 1, 1 | 48 | 2, 3 | 49 | 2, 3 |
| Necromancer | 3 | 1, 1 | 66 | 1, 3 | 62 | 1, 3 |
| Paladin | 3 | 1, 1 | 37 | 2, 4 | 38 | 2, 4 |
| Rogue | 4 | 2, 2 | 76 | 2, 5 | 64 | 2, 5 |
| Ranger | 3 | 1, 1 | 86 | 2, 5 | 67 | 2, 5 |
| Blackguard | 5 | 2, 1 | 39 | 2, 5 | 39 | 2, 4 |
| **all** | **30/45** (67%) | 1, 1 | **619/1800** (survive 66%) | | **564/1800** (survive 69%) | |

The rate is similar; the game behind it is not. Our survivors spend most of
the 100,000 turns at depth 1 and character level 1 (5 of 30 survivors
reached depth 2 or more). The C Borgs' survivors and deaths alike are deeper
and stronger. Our deaths are as early-game as theirs: all 16 at depth 0–2
and character level 1–3. The cause mix overlaps the C Borg's DL1–5 killers
(Maggot's dogs, crows, giant white mice, small kobolds, poison, cutpurses,
a fatal wound).

## Thresholds still declared, not tuned

The calibration measures the threat model. It changes no parameter (RULES 14:
no claim without a prespecified endpoint and a confirmation suite). Every
value below is still a declaration.

| Name | Value | What this report says about it |
| --- | --- | --- |
| `risk_aversion_pct` | 50 | Expected damage is already 8–12 times the realised damage on average, so a midpoint toward worst adds caution to an over-cautious number. Not changed: deaths come from windows the model flagged, so lowering aversion has no measured benefit yet |
| `threat_horizon` | 1 player turn | Windows are one think; a longer horizon would compound the frequency error found here |
| sleeping share (`danger.rs:melee_threat`) | 0.5 | 0 HP observed in 80 windows (48 predicted); small sample |
| `+3` monster speed below 20 max HP (`borg-danger.c:2357-2359`) | +3 | Inflates every early prediction; candidate first fix for the 0.58 blow rate |
| confusion ×0.5, stun ×0.75 | lifted | not separable in this sample (no confused/stunned single-source windows) |
| `Track::confidence` decay | 1/(1+moves) | 0.015 of predicted for remembered out-of-view monsters |
| breeder and summoner growth (1.5; 1.8 / 1.5) | declared (#163: the C Borg's code at `borg-danger.c:borg_danger_one_kill:2420-2436, 2700-2710` computes ×2.5, ×2.8/×2.3 and ×2.2; its comments say +50%, +80%/+30%; ours follow the comments, and the ≥CL15 summoner 1.5 is ours) | same-race groups at 0.05 |
| ranged floors (`borg-danger.c:619-760`), uncommon breaths `hp/6` | lifted / declared | 21 single-source ranged windows, ratio 0.20 (0.00–0.46); too few to separate |
| `escape_item_hp` 1, `last_escape_hp` 5, `arrival_risk_hp` 3 | declared (T4.4) | not measured: no window isolates them |
| `rest_below_hp_pct` 70, `eat_below_food` 300, reserves | declared | not threat parameters; two starvation deaths stand against `eat_below_food` with no food left |

## What was not measured

- Walk, stair and Alter windows cannot calibrate a source, because the
  assessment is for the grid the Borg leaves. Recording the destination's
  assessment for a Walk (as `travel_step` already computes it) would close
  that.
- Spells and breaths beyond the 21 ranged single-source windows; no breath
  was seen.
- Confused or stunned monsters; any depth below 3.
- Disabling: 6 onsets in 352,000 windows, 3 of them in the 10 windows
  predicted above 0.5. The sample is too small to calibrate.
- The economic term (theft): one theft in the deaths, none measured.
- Whether over-prediction costs survival or progress. That needs a sized
  comparison with a changed model (RULES 14), which is future work; this
  report only measures.
- Any classes-1800 run of our Borg. The zoo's last reborn-rust arm
  (`reborn-e-classes-1800`) predates M3/M4 and died in every game; a new
  specimen run is the owner's to start (RULES 20).

## What follows from it

These are candidates for tasks, not changes made here:

1. **Blow frequency.** Discount the standing assessment by what the Borg's
   own action does: a monster the exchange expects to kill or frighten first
   does not swing. Revisit the `+3` low-HP speed and add the engine's
   erratic movement (`RAND_25`/`RAND_50`). Measure against this report's
   standing-still pool.
2. **The paralysis floor.** A paralysing blow on a paralysed character deals
   at least 1 HP (`mon-blows.c:1019-1021`). Resting beside a never-moving
   paralyser at low HP is the death in D1669, and the largest unattributed
   window is `full-5-paladin-4` at turn 6,579: a Walk next to a floating
   eye, then 19 HP lost while paralysed. The model should
   price it, and Recover should not rest within reach of one.
3. **Unseen attackers.** Damage with no visible source ("Something bites
   you") should create a threat at the character's grid, so Rest and Recover
   do not resume (D1656).
4. **Status damage.** Poison and cut ticks are hit points the model never
   counts (D1657, D1663; most worst-case exceedances).
5. **Destination assessment in the trace,** so Walk windows can be
   calibrated (the trace already carries `sources`).
6. **Starvation** remains the late residue (D1664, D1668), as T4.6 reported.

Items 1–4 change play and need their own baselines. They are listed in the
handover for the owner to place (M4 review or M5).

## Addendum: the baseline after #137 (2026-11-12)

#137 changes play in two full runs; smoke and smoke-exact are unchanged.
The full reference was regenerated at `798239a`, and a fresh full run
reproduces it 45/45.

- `full-6-rogue-3` no longer re-reads a Phase Door the engine refuses while
  confused. It survives to the cap (was D1667, a cutpurse at 24,362).
  Its survival is not progress: it walks in a loop for about 75,000 turns
  around three apprentices remembered out of view. Their ranged estimate
  ignores the tracks' confidence (0.00015 at the end, still `expected
  1.8`, `worst 14.4` each). That is **#157**. Until it is fixed, the pooled
  "all threatened" ratio falls to 0.017, because this one run adds 20,663
  predicted HP. The standing-still pool (0.118) and the blow decomposition
  (per blow 1.71 predicted against 1.73 observed; 0.57 of predicted blows
  among monsters that survive) are essentially unchanged.
- `full-7-ranger-0` no longer tries to read Recall with no light. It still
  starves at turn 89,750 (D1683).
- **Gate numbers on this baseline:** 31/45 survive the cap (69%); 15
  natural deaths in full and smoke, all with dossiers (D1670–D1684);
  `check-no-time.py` finds 0 violations; the refused-read violations
  are gone (23 in the T4.7 run, 0 now).

## Addendum: after #139 and #157 (2026-11-12)

#139 changed three full runs through the Engage progress sentinel; full
survival stayed 31/45. #157 scales a remembered monster's ranged estimate,
and its chance of death, by the track's confidence, as the melee half
already was. It changes one run: `full-6-rogue-3` no longer walks in a loop
around three forgotten apprentices. It moves on, reaches character level 3,
and dies to Fang at turn 25,237. Its last four windows show p_death 0.00,
0.47, 1.00 and 0.93 at 4, 4, 1 and 1 HP: it killed a wild cat, then
walked, and Fang bit. The model warned and the Borg fought, the most
common shape in this report.

On this baseline (`tools/calibrate.py harness/out/full harness/out/smoke`):

| pool | windows | predicted | actual | actual/predicted (90% CI) |
| --- | ---: | ---: | ---: | --- |
| all threatened | 11,742 | 5,421.6 | 448 | 0.083 (0.07–0.10) |
| one source | 2,022 | 1,544.2 | 217 | 0.141 (0.11–0.17) |
| standing still | 1,253 | 3,545.7 | 400 | 0.113 (0.10–0.13) |
| remembered, out of view (one source) | 893 | 204.6 | 3 | 0.015 (0.004–0.03) |

The remembered-out-of-view row is back to its T4.7 shape (905 windows,
196 predicted). After #137 it had held 4,682 windows and 7,008 predicted
HP from the phantom apprentices. Out-of-view sources are still
over-predicted about 65 times, which says confidence decays too slowly for
a one-turn horizon. That is a calibration lead for the M4 review, not a
change made here.

**Gate numbers on this baseline:** 30/45 full runs survive the cap (67%,
the T4.7 rate: the extra survivor of #137 was the loop); 16 natural deaths
in full and smoke, all with dossiers (D1715–D1730); 0 no-time violations.
