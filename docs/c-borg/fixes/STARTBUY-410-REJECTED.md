# Improvement I-003: REJECTED (negative result)

Unit: starting purchases - reserve gold for 4 Phase Door + 3 cure potions
before gear during the first-visit window
([#410](https://github.com/diegoami/better-borg-c/issues/410), contract and
evidence there; owner direction 2026-11-10 "do 410"). Branch
`task/imp-003-startbuy` (commits `a1a5a948a`, `c02a34253`, `49cb377bd`);
**not merged** - main is untouched. Local improvement unit, never proposed
upstream.

Status: **rejected** per the contract's own declared rule ("if cohort 2
still fails the progress margins, the unit is rejected and recorded as a
negative result"). The analysis findings (Q1 valuation mechanics, Q2 no
early floor) stand; the strict reserve + suppression policy is the tested
and rejected part.

## What was tested

Two policy revisions, both implemented on the branch:

- **v1** (`a1a5a948a`): reserve pass in `borg_think_shop_buy_useful()` -
  while `BI_MAXCLEVEL <= 7` and not in an urgent state, buy the cheapest
  affordable deficit toward 4 phase doors + 3 cure potions and suppress
  every other purchase in that function until the reserve is met.
- **v2** (`c02a34253`): v1 plus a home-sell guard - while the reserve is
  unmet, `borg_think_home_sell_useful()` leaves carried phase doors and
  cure potions in the pack (constants shared via `BORG_RESERVE_*` in
  `borg-store.h`, condition in `borg_startbuy_reserve_unmet()`).

Native gate green on both revisions (965/965 + 5/5). Scenario fixture
`tools/startbuy-scenario.c` (group `startbuy-410`, before-logs
`artifacts/startbuy-410/`): fail-before/pass-after on `pd-second` and
`clw-deficit` (the before build's valuation picks stocked rations because
later phase doors and low-level cures cannot win), byte-identical
`reserve-complete` and `urgent-poisoned` controls; all four modes
byte-identical between the v1 and v2 after-builds.

## Cohort 1 (v1 policy): FAILED

Seeds 3001-3012 (frozen per the contract), Human Warrior fixed birth, cap
12000, old `25d1ce6a7` vs v1. Survival losses 1/12 (within the <=2/12
margin) but **median max-depth 5 -> 3** - outside the declared "no median
max-depth loss >1". Diagnosis (traces): buy/sell deadlock - the reserve
bought phase doors and the home-sell pass immediately stored the `{unseen}`
scrolls back, so the borg looped between shops and home in town (seeds
3004/3009/3011 sat at CL1 for the whole run). v2's guard removes the loop
(those seeds then reach max-depth 6/5/6, at or above the old builds' 5/5/5).

## Cohort 2 (v2 policy): FAILED - decision is REJECT

Seeds 4001-4020 (frozen, declared before running), same pairing, cap 12000.
Declared margins: <=2/20 paired alive->dead losses; median max-depth loss
<=1; <=6/20 seeds with a max-depth loss >=2.

| measure | old `25d1ce6a7` | new v2 `c02a34253` | margin | verdict |
| --- | --- | --- | --- | --- |
| alive->dead losses | - | **4/20** | <=2/20 | **fail** |
| median max-depth | 6 | 6 | loss <=1 | pass |
| seeds with max-depth loss >=2 | - | **7/20** | <=6/20 | **fail** |

Per-seed finals (outcome depth/max-depth/CL/HP):

| seed | old | new | max-depth delta |
| --- | --- | --- | --- |
| 4001 | DEAD d1/2/2/-1 | DEAD d6/6/5/-3 | -4 |
| 4002 | alive 5/6/5/81 | alive 2/2/2/35 | +4 |
| 4003 | alive 5/6/5/64 | DEAD d1/1/1/-1 | +5 |
| 4004 | alive 0/2/3/42 | alive 4/6/8/91 | -4 |
| 4005 | alive 5/6/7/69 | alive 4/6/7/69 | 0 |
| 4006 | alive 6/6/6/60 | DEAD d1/1/5/-1 | +5 |
| 4007 | alive 0/6/7/96 | alive 5/6/6/88 | 0 |
| 4008 | alive 5/6/8/96 | alive 4/4/2/28 | +2 |
| 4009 | alive 4/6/9/132 | alive 5/6/7/100 | 0 |
| 4010 | alive 0/1/2/27 | alive 5/6/7/90 | -5 |
| 4011 | alive 1/1/2/40 | alive 4/5/6/95 | -4 |
| 4012 | alive 5/6/7/82 | alive 0/6/7/82 | 0 |
| 4013 | alive 5/5/6/98 | alive 5/5/6/98 | 0 |
| 4014 | alive 3/3/4/44 | DEAD d3/6/6/-1 | -3 |
| 4015 | alive 7/7/7/86 | alive 0/1/1/20 | +6 |
| 4016 | alive 5/6/6/89 | DEAD d1/1/1/-1 | +5 |
| 4017 | alive 3/3/4/46 | alive 4/6/8/113 | -3 |
| 4018 | DEAD d5/6/6/-4 | alive 0/1/3/34 | +5 |
| 4019 | alive 6/6/7/61 | alive 4/6/5/51 | 0 |
| 4020 | alive 6/6/8/118 | alive 6/6/7/107 | 0 |

Direction is mixed (several seeds improve) but the declared margins fail on
both survival and the max-depth-loss count. The three new CL1 deaths
(4003, 4006, 4016) are the predicted early-AC cost: the strict reserve
sends starting gold to consumables, so the character fights at CL1 with the
starting dagger and no armour upgrade and dies to trivial monsters
(kobold shaman, giant white mouse, grey mushroom patch). Both
distributions are noisy at n=20; the declared rule does not require
significance - the margins were the acceptance, and they failed.

Frozen cohort-v1 (seeds 11/29/47, 20k) on v2: 6/6 alive at cap (finals
depth 4/5/5, CL 5/6/6) - no survival regression there, but it cannot
offset the declared-margin failure above.

## Exact commands

Tools: `tools/measure-native.py`, `tools/measure-cohort.py`,
`tools/regression-matrix.py` under `flock /tmp/angband-validation.lock`.
Observer builds: `/tmp/opencode/imp003-obs-{old,v2}` (prepare `--ref
25d1ce6a7` / `--ref c02a34253 --observe`); runs at cap 12000
(`--seed S --cap 12000 --timeout 120`), cohort via `measure-cohort.py
--phase baseline`; fixture via the regression-matrix group `startbuy-410`
(baseline before-logs committed under `artifacts/startbuy-410/`,
gitignored evidence). Raw run trees (gitignored, ephemeral):
`/tmp/opencode/imp003-c2-{old,new}`, `/tmp/opencode/imp003-early-*`,
`/tmp/opencode/imp003-cohort-*`.

## Decision and follow-ups

Decision: **reject** this policy shape. Rollback route: nothing merged;
`main` never carried the change. The rejected branch is retained as
evidence.

What remains true from the analysis (issue410): the valuation gives later
phase doors +500 (suppressed under munchkin start), no cure-potion reward
below CL10, no carried-recall reward below CL8 - so the recorded
first-visit pattern is the valuation's optimum, and no early purchase-side
floor exists (`borg_prepared()` is happy at depth <= 5). Candidate shapes
for a future contract (none scheduled; each needs a fresh declaration):
a softer reserve that buys consumables first without suppressing gear;
a purchase-side floor tied to leaving town only (e.g. buy the reserve or
evidence it is out of stock, then leave); the dive-side floor of #154;
Recall retention (#183).
