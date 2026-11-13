# The town plan: decisions from data, judged by the zoo

*Status: draft for the owner's review (2026-11-13). No code lands before it
is accepted. Source: borg-rs's town inventory strategy (borg-rs#36, PR #42,
branches `feature/town-*`), read at their heads of 2026-11-13; S5 (docs/SOURCES.md):
a reference to copy from, recorded in docs/PROVENANCE.md, never a dependency.*

## Why

The Borg buys five hand-written reserves (Ration, Torch, CLW, Phase Door,
Recall) and uses a short list of item kinds by hand-written rules. The zoo's
arm A (borg-reborn-rs#128, #183) shows where that leads: 86% survive,
0% reach DL5 in any class, mean level 1.3. More rules, one at a time, each
tuned on 45 runs, is not the way out (owner, 2026-11-13). borg-rs replaced
the same kind of rules with:

- **floors**, never traded: food, light, a cure for the condition, a
  poison-cure capability;
- a **usefulness function** over eight needs (light, food, bleeding,
  poison, escape, knowledge, power, trip): an item is worth the fraction of
  a deficit it closes, the deficit measured against a **trip model**; the
  value curve is flat then capped. borg-rs's trip model is provisional
  constants (depth target `max_depth + 5`, food per trip, one escape per 10
  levels, power `2000 x depth`): **we do not adopt those** (below);
- an **exact solver**: a group knapsack over gold, a Lagrangian penalty on
  pack slots, a weight repair; one plan per visit, re-solved after every
  action, deciding keep / store / sell / drop and what to buy;
- **one weight vector per zoo arm**: the weights are the free parameters
  the zoo sweeps;
- **two kinds of evidence** next to the zoo: the control's choices (the
  current rules, recorded headless) and a human's (town sessions where the
  owner takes the controls), compared to the plan in **shadow mode**, the
  disagreements read by the owner. End-of-run corrections are labels for
  the usefulness table of states and expected values.

Buying is useless unless the Borg uses what it buys (owner, 2026-11-13), so
the plan comes with **item-use mechanics**: how to use every usable item,
read from the engine, with no judgement in it.

## No lists: every target is a function of the state

A game visits the town hundreds of times, and every visit is a different
problem: the gold, the depth the Borg goes to next, the monsters that live
there, what it carries and can already use (owner, 2026-11-13). What is
right on the third visit (rations, torches, a few CLW) is wrong on the
hundred-and-fiftieth (Teleport, CCW, Speed, resistances). So the plan has
**no fixed counts**, ours (the five reserves, "20 shots") or borg-rs's
provisional constants. Every target is recomputed at every visit from the
state, through the threat model the Borg already has:

- **the trip**: the depth the Borg will play next (readiness for each
  depth, `readiness.rs`; the maximum depth; the trend of gold and level)
  and its expected length;
- **cures**: the expected damage of the fights at that depth (the natives
  of its band and their worst turns, `danger.rs:worst_turn`) against the
  character's maximum hit points;
- **escapes**: the natives the Borg cannot beat (worst turn above the HP
  fraction) times the expected encounters with them;
- **resistances, See Invisible, Free Action**: from the elements,
  invisibility and paralysis of the natives at the target depth (the manual's
  DEPTH-02 rows, derived from `monster.txt` rather than a table of depths);
- **food and light**: from the trip's expected length;
- **ammunition and missiles**: the share of the damage the trip needs dealt
  at range that melee and spells do not already cover;
- **power** (equipment): the try-on of every wearable against the effects the
  Borg already values (melee, AC, mana, ranged damage), not a target.

What is measured, then, is the trip model and the weights: the zoo chooses
between arms, the human sessions show how a player's choices move across a
whole game (one game is hundreds of visits, so a few sessions cover many
states), and the trip model's predictions are checked against what the
runs actually consumed.

## The record schema: shared with borg-rs

One session format for both Borgs (owner, 2026-11-13). We write borg-rs's
integer shapes where our state has the field, under the same names and
version numbers, and version our own additions:

- **v2** (decision dump, frozen in borg-rs): `v, seq, turn, depth, level,
  chp, mhp, gold, race, class, traits, action, shop, ware, item, price`;
  pack/equipment/home entries `[slot, kind, subkind, qty, value, flags]`,
  shop entries `[shop, [[slot, kind, subkind, qty, price, flags]...]]`;
  -1 for a missing value; flags bit 0 known, bit 1 not identified. `traits`
  are C-Borg trait indexes, which we do not have: we write `[]` and add
  our own derived fields under a versioned extension (`x` object), never
  reusing a borg-rs field name with another meaning.
- **v3** (the handover's state record): v2's shape without the decision, one
  per distinct town state, deduplicated from `"turn"` onward.
- **v4** (the plan record): v2 plus the plan's fields (`mode` shadow/live,
  `paction`, the runner-up, `wares`), when the plan exists.
- **objects.log**: `NNNN turn T depth D <kind> ...` lines for buy (with cost),
  take, acquire, quaff, read, use, zap, aim, activate, throw, fire.
- **feedback.md / corrections.jsonl**: `{"line": N | null, "text": "..."}`.

Kind and subkind indexes are the engine's (`k_info`, upstream e473720 for
us); a cross-Borg comparison is valid only on the same object tables.

## The steps

Each step is one issue and one or more PRs, measured where it changes play
(the baseline and the travel-guard arm, #183), merged on green.

1. **Town handover and recording** (#191) (engine hooks and tools, no judgement).
   `BORG_TOWN_PLAYER=1`: in town the Borg yields the key hook so the owner
   plays through the X11 launcher (`watch-borg.sh`); on leaving town the
   Borg takes over. The v3 state writer, the object log, `harness/play.sh`
   and `harness/feedback.sh`. With the variable unset the key stream is
   byte-identical (a test). *Copies: borg-rs `borg.rs:407-462`,
   `borg_town_dump.rs`, `borg_object_log.rs`, `harness/play.sh`,
   `harness/feedback.sh`, adapted to our engine boundary.*
2. **Curation** (#192): sessions to a purchase dataset (`tools/curate-purchases.py`
   with its tests), **plus the state-purchase pairing borg-rs has not
   written**: each purchase joined to the v3 state it was made in.
3. **The control baseline** (#193): the current reserve rules' town choices on the
   full suite, in the v2 shape (`tools/town_baseline.py`,
   `tools/town_dump_report.py`).
4. **Item-use mechanics** (#194) (in parallel with 2-3): every usable kind usable,
   from engine facts: the command and its prompts (aim `a`, use `u`, zap
   `z`, activate `A`, throw `v`, quaff, read, eat, fire), the failure chance
   (`get_use_device_chance`), charges and recharge, and the effect with its
   dice evaluated without a roll (the spells' route). When to use an item
   stays with the tactical layer's one scale (risk removed against cost),
   not with per-item rules.
5. **The trip model** (#195): the state-derived targets above, written as pure
   functions of the World and the knowledge, with tests on fixed states.
   **Validated before any solver uses it**: on the full suite, its predicted
   needs per trip (cures, escapes, food, light, missiles) against what the
   runs actually consumed between two town visits; the gaps go to the
   owner.
6. **The model in shadow mode** (#196): the usefulness function over those
   targets, the stock, the budget and the solver (borg-rs phases 1-3,
   copied and adapted) on our `World`, computing a plan at every town visit
   next to the rules' choice, writing v4. The owner reads the disagreements
   (and the human sessions') before it acts.
7. **The model live** (#197): the plan replaces the reserve rules
   (keep/store/sell/drop/buy, the Home as a floor source); the arms are
   weight vectors on classes-1800.
8. **The depth gate** (#198) recalibrated with the kit and its use in place
   (readiness.rs, c15 DEPTH-01/02), as its own arm against the step-7
   baseline.

## Not decided here

- The trip model's exact derivations (step 5 proposes them; the owner
  accepts them with the validation).
- Whether dungeon-side item use also gets usefulness weights (step 4 keeps
  the tactical layer's scale).
- The travel guard's release (#183) and G1's D and margin (#128): the
  owner's, from the zoo.
- `store_buy_direct` (a fork-side store option borg-rs proposes): not
  adopted; our engine is upstream byte for byte (RULES 9).
