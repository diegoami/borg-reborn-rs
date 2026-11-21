# B2: the kind-only alternatives, and what they leave the solver

Status: **accepted** (owner, 2026-11-13: the three recommendations below
are accepted as stated - Q1 the multiple-choice group per equipment slot,
Q2 the caution per group, Q3 exact for the separable surrogate). Phase 3's
entry criterion, now met. Drafted 2026-11-12. Epic borg-rs#36; the blocker and its answer are on PR #42
(review comment of 2026-11-11, "B2. The solver assumes groups do not
interact, but phase 2 builds alternatives"; owner answer the same night,
"B2 — alternatives are kind-only"). Where this document and the epic's
decision log disagree, the log wins and this document is wrong.

## The decision being written up

> P5 is resolved to **kind-only** alternatives before phase 3 starts. The
> same kind at two shops is priced as the cumulative cheapest-n across
> shops (1x then 3x at the Black Market), so the DP never carries one price
> for two prices. Grouping by need is rejected for this design; phase 3's
> exactness claim stands on the kind-only basis, and the write-up of B2 is
> a phase-3 entry criterion.

The blocker named two interactions that break the group DP's
independence: the **same kind at two prices**, and **different kinds
closing the same need**. The decision answers the first and rejects the
second as a grouping basis. This write-up states precisely what the
kind-only basis makes exact, what it does not, and what the phase-3 solver
must therefore do. Three points need an owner answer; they are collected
at the end.

## 1. The group is the stack class, a refinement of the kind

Phase 2 (`borg_town_stock.rs`) gives every ware a **stack class**: wares
of one kind that stack with each other share a class. The solver's group
should be the class, not the bare kind:

- for food, potions and scrolls - everything the floors and most needs
  buy - `borg_object_similar` stacks every copy of a kind, so **class =
  kind** and nothing changes;
- for weapons, armour, rings and lights, copies of one kind stack only
  with identical bonuses. A Ring of Protection [+5] and one [+10] are one
  kind and two items with two values; pricing them as one kind with a
  cheapest-n curve would buy the cheaper ring and credit it with the
  dearer one's power. Within a class the copies are identical, so their
  value is identical too.

So the decided "the same kind in another shop is the alternative" holds
exactly *within a class*, which is where it can hold at all. This is a
refinement, not a change: it never splits a class of consumables.

## 2. The cost of n copies: cumulative cheapest-n

Within a group the copies are interchangeable, so the value of the plan
depends on **how many** copies it holds, never on **which shop** sold
them. Then, for any n, buying the n cheapest copies is optimal: swapping a
dearer copy for a cheaper unbought one keeps the value and lowers the
cost (an exchange argument). So

```
cost_G(n) = the sum of the n cheapest per-copy prices across the wares of G
```

is the exact cost curve, and it has two properties the solver uses:

- **convex**: the marginal price is non-decreasing (1x copies first, the
  Black Market's 3x last). The Black Market's price binds only once the
  1x copies run out - which is what makes the 3x meaningful rather than
  arbitrary;
- **slot and weight are simple in n**: all copies of a class share one
  stack, so the slots the group opens are `s_G = [n > 0] * slots_cost`
  (0 when a carried stack absorbs it, else 1), and the weight is
  `n * weight`, linear.

Phase 2 implements this as `KindOffer::cost/split/weight/slots_cost`.
With the group refined to the class, `slots_cost(n)` is 0 or 1 and the
per-class bookkeeping in `KindOffer` becomes the trivial case.

A worked instance. Cure Light Wounds, 2 copies at the Alchemist for 20
each and 3 at the Black Market for 60 each:

| n | 0 | 1 | 2 | 3 | 4 | 5 |
|---|---|---|---|---|---|---|
| cost_G(n) | 0 | 20 | 40 | 100 | 160 | 220 |

With the value flat-then-capped (B1) and a gap of 2, `U_G(n)` is
`w/2, w, w, w, w` for n = 1..5: the DP never buys a third copy, so the 3x
price is never paid. With a gap of 4, the third and fourth copies come
from the Black Market at 60 each - and only then does the caution cap
(section 5) have anything to say.

## 3. What "exact" means: exact for the separable objective

The group DP over the gold

```
dp_new[b] = max over n of  dp_old[b - cost_G(n)] + U_G(n)
```

is exact for any value table and any cost table, provided **each group's
value depends on its own count alone**. Section 2 makes the cost
separable. The value is separable by construction - and that is the part
to state honestly: each group's `U_G(n)` is computed against the deficit
**as the carried kit stands when the plan is solved**, not after the
plan's other purchases.

So the DP is exact for the **separable surrogate**

```
maximise  sum over G of  U_G(n_G)      with every U_G priced against the current deficit
```

and the surrogate differs from the true objective of #39 in exactly the
interaction the blocker named second - **different groups closing the
same need**:

- Cure Light Wounds and Cure Serious Wounds both close the bleeding gap
  (`borg_trait` counts both as cut cures); Cure Critical Wounds and
  Neutralize Poison both close the poison gap (#33's evidence: they are
  the only two that cure it). Each is priced against the whole gap, so
  within one solve the surrogate can credit the gap twice and choose
  both;
- the **re-solve after every action** is what corrects it. The plan
  executes one action, the state changes, the deficit of the need shrinks,
  and the next solve prices the second kind against what is left. The
  surrogate is never trusted beyond the first action, which is the
  architecture's rule anyway ("the solver is re-run after every action,
  so it never has to be trusted with a plan it computed before the state
  changed").

What the re-solve guarantees, and what it does not. Every executed
purchase is chosen while its need's deficit is open, so no purchase is
ever worthless at the moment it is made. What the surrogate can get wrong
is the **mix**: priced twice, a need looks worth more than it is, so the
first action may favour it over a need the true objective would serve
first, and the gold spent there is not available to the rest. This
argument gives no bound on that loss. It is observable instead: the
record (P13) shows, per action, the need it served; a solve that chose
two kinds of one need, followed by a re-solve that drops one, is visible
in the log, so the zoo can measure how often it happens rather than
argue it.

Grouping by need - one group per need with the kinds as options - was
the alternative and is **rejected** by the decision: it makes the DP's
group a choice among kinds with different costs and reductions per copy,
which is a different and larger problem (a group's table becomes a
knapsack of its own). The surrogate plus re-solve is the decided form.

## 4. One interaction the decision does not cover: the equipment slot

The power need is not a deficit that copies fill; it is a **try-on**:
`power_with(w) - power()` (`borg_store_buy.rs:457-696`). Two body armours
are two classes (section 1) and each is priced against the *current*
armour. Both can be in the surrogate's optimal set, but the character
wears one: the second is not a redundant copy of a consumable that the
re-solve trims, it is a purchase whose value under the surrogate is
simply false.

The DP stays exact with one change: **wearables are a multiple-choice
group per equipment slot** - the options of the group are "buy none" and
"buy ware w" for each ware that fits the slot, at most one chosen. The
group DP already enumerates options per group (`max over n` becomes
`max over the options`), so the form, the cost and the exactness claim
are unchanged; only the group key differs for wearables. This is a
grouping by *equipment slot*, not by need - the decision's rejection of
need-grouping does not reach it - but it is a choice the decision did not
make, so it is listed for the owner (Q1).

## 5. The Black Market's caution in the cost table (P6)

The caution is `cost <= purse / k` for a shop-6 candidate (decided
2026-11-11; its exact shape is P6, open). In the cost table it is a
**truncation**: `cost_G(n)` is admissible only while the Black Market
copies in its split satisfy the cap. The two readings P6 leaves open:

- **per transaction**: the copies bought from one Black Market ware in
  one purchase, `copies * price * k <= purse`;
- **per group**: all Black Market copies of the group in the plan,
  `sum * k <= purse`.

Because the Black Market copies come last in the split (section 2), both
readings truncate the table at a single n, so either keeps the curve
convex and the DP exact. Which one is P6's decision (Q2). The floors are
not priced and are not capped (phase 2, `borg_town_budget.rs`).

## 6. What the phase-3 solver must do, from the above

1. build one group per **stack class** for consumables, and one
   **multiple-choice group per equipment slot** for wearables if Q1 is
   accepted;
2. remove the floors' copies from the wares' quantities **before** the
   cost tables are built, and solve on the budget after the floors
   (`Budget::gold`, `pack_slots`, `weight`);
3. bound each group's table at the cap - the first n at which `U_G(n)`
   reaches its weight - since copies past the cap add cost and no value;
   and at the copies available;
4. truncate Black Market tables by the caution (Q2);
5. run the exact gold DP; treat the pack slots by the Lagrangian sweep over
   `lambda * s_G` (integer lambda, binary search until the kinds fit);
   repair the weight afterwards (P15's recommendation, unchanged);
6. break ties deterministically: the larger usefulness, then the less
   gold, then the fewer slots, then the earlier group (class order);
7. execute only the first action of the chosen set, then re-solve.

## 7. What phase 3's suite must assert, from the above

- the P15 instance (greedy 12, optimum 18), so the greedy cannot pass by
  accident;
- a cross-shop instance where the 3x price is **not** paid because the
  cap closes first, and one where it **is**, within the caution;
- a split-class instance: two rings of one kind with different bonuses
  are two groups, never one cheapest-n curve;
- the documented surrogate case: two kinds of one need both chosen in one
  solve, and the re-solve after the first purchase pricing the second
  against what is left - pinned as the intended behaviour, not as a
  bug;
- if Q1 is accepted: two wearables for one slot never both chosen.

## Questions for the owner

**Decided 2026-11-13: all three recommendations accepted** (Q1 accept,
Q2 per group, Q3 accept). The questions are kept as they were asked.

- **Q1 - the equipment slot.** Wearables as one multiple-choice group per
  equipment slot (at most one purchase per slot per solve), section 4.
  *Recommendation: accept; without it the surrogate's power values are
  false rather than merely redundant.*
- **Q2 - the caution's reading (P6).** Per transaction or per group,
  section 5. *Recommendation: per group, so splitting a purchase cannot
  evade the cap.*
- **Q3 - the surrogate.** Accept that "exact" means exact for the
  separable surrogate, with the re-solve correcting cross-kind
  redundancy, and the redundant purchase recorded as an observable,
  section 3. *Recommendation: accept; it is what the kind-only decision
  already implies, stated rather than assumed.*
