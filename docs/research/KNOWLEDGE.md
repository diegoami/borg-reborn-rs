# Knowledge ledger

*What this repository has read from its moving sources, what it means for
the tasks ahead, and how a session knows what is new to read. Update it
whenever you read (see [The loop](#the-loop)). The sources themselves are
described in [docs/SOURCES.md](../SOURCES.md); live state (open bugs,
carries, experiments) stays on the issue boards, which this file only
indexes.*

## The loop

Every session starts with `python3 scripts/refresh-sources.py` (RULES 21).
That report is the delta engine; this file is the state:

| The report says | Read |
| --- | --- |
| **new death cases** (better-borg-c, borg-rs) | the new dossiers' `report.md` first, then `overall-assessment.md` and their issue if one is named |
| a copy changed (`manuals`, `knowledge`, `records`, `ACCUMULATED_KNOWLEDGE.md`, `REDESIGN-LESSONS.md`, the zoo's catalogue) | the diff of that copy (`git diff <path>`): the pages are append-heavy, so the diff *is* the reading list |
| issues updated since `<read_on>` | the listed issues: death analyses and `Analyse …` issues are design input; `Adopt sourceNN` issues are C-Borg carries, ours only if they cite functions we lift |
| new tags `4.2.6-impact-v*` | the milestone summary issue: what the runtime units were and what was rejected |
| cited C Borg functions changed | re-check the citation where we lift it |

Then update this file's reading state and, if the reading changed a plan
or a design, the handover. Two rules that keep it honest:

- **A FACT is not ours until it is checked and cited** against
  `engine/angband/src/` at `e473720` (RULES 16). This file points; code
  cites.
- **Nothing is a claim until the zoo measures it** (RULES 14). Candidate
  rules below are hypotheses with evidence, not results.

## Reading state

| Source | Last read | What was covered | Re-read when |
| --- | --- | --- | --- |
| borg-runner (S15) | 2026-11-10, pin `883c474` | everything we copy: `INSTRUCTIONS.md` (all 469 lines), volume 2, the jev manual, every notebook and report including `minimaxflash-3`, `OVERVIEW.md`, `ACCUMULATED_KNOWLEDGE.md` **edition 3** in full (edition 2 before it, and the diff between them) | the report's copy lines: a changed `ACCUMULATED_KNOWLEDGE.md` means a new edition (read its own changelog paragraph, then the diff); new runs arrive as `knowledge/` + `records/` additions |
| better-borg-c deaths (S4, S9, S11) | 2026-11-10, pin `06f5789` | every copied dossier D0001–D0019 (`report.md`; `overall-assessment.md` for D0017–D0019), `REDESIGN-LESSONS.md` covering D0001–D0019 | new death cases; a changed `REDESIGN-LESSONS.md` |
| better-borg-c issues (S8) | 2026-11-10 | the analysis set: #34, #41, #154, #158, #159, #180, #183, #319–#341 (death analyses and analyses), and on this pass #400–#412, #420 (D0017–D0019 analyses, the five `Analyse`/`Evaluate` issues, the milestone-10 review); `docs/research/DEATHS-AND-ISSUES.md` indexes the rest by subsystem | the report's `issues updated since` list |
| neo-angband-tracker (S2, S14) | 2026-11-10 | the C Borg spec and the Tunable Plan at their pins; the strategy manual's chapters as tasks need them (ch. 15 for M3) | nothing scheduled: the pins do not move (the manual's chapters are read per task, below) |
| borg-zoo (S6) | 2026-11-10, pin `a092d1e` | the DESIGN and the catalogue diff of this refresh (bbc-m8/m9 specimens, `classes-1800-200k`) | changed catalogue/DESIGN copies; our specimens' issues |

## Distilled for the tasks ahead

Pointers, not copies: the line lives in the source, named beside it.

### T3.3 — the first strategy (modes, goals, budgets)

- **Depth is not the objective** (S15, INSTRUCTIONS *Depth is not the
  objective*; ACC-KNOWLEDGE §1): nine of ten runs ended at depth 0–2
  alive but stuck; both deaths were CL1 characters hunting a staircase;
  the deepest run arrived at depth 7 **at CL8**. The first strategy's
  mode set should treat *explore this level* and *town* as plans, not
  failures — our GOALS principle 6 budgets fit this directly.
- **A level has three or four down staircases and one or two up**
  (ACC-KNOWLEDGE §2, edition 3, `gen-cave.c:1291`), each on a dead-end
  square, ≥16 tiles apart. More targets per level than the C Borg's flow
  assumed; `path.rs` already floods to *all* known stair grids.
- **Explore-or-reroll is open** (ACC-KNOWLEDGE §8.3): a fresh level via
  `<` `>` costs 2–3 decisions against a coverage sweep's ~100; uniform
  placement makes the fresh level exactly as promising. A candidate
  parameter for the explore budget's failure path (GOALS principle 6's
  blocked-goal cooldown).
- **Constitution sets the pace of descent** (S15, INSTRUCTIONS *Constitution
  sets how fast you may dive*; ACC-KNOWLEDGE §4): HP per level varies
  −2.5…+12.5 by CON (`adj_con_mhp`, `player-calcs.c`), so a gate written
  in hit points fits nobody; gate on the depth band's worst round (a
  soldier's 2×1d7=14 at depth ≤2). Input for M3's simple descend gate and
  M5's readiness policy.
- **Budgets in game turns, and no batched waits with a monster
  unaccounted for** (S15, the `minimaxflash-3` death: Word of Recall read
  then forty standing turns; ACC-KNOWLEDGE §6). Our executor sends one
  action per think, which is the structural answer; the lesson for T3.3
  is that *waiting* is an action with a cost (150–340 game turns for
  Recall).

### T3.4 — melee, pickup, stairs, rest

- **Armour class decides whether anything connects at all**:
  `hit_chance = 12% + 83% × (to_hit − 2AC/3) / to_hit`
  (`player-attack.c:196`; ACC-KNOWLEDGE §4). Below `2AC/3` to-hit is the
  flat 12% floor. Depth-1 AC anchors: kobold/ant 24, worm mass/mold 1–8.
  Both directions of this formula feed M4's threat model.
- **Thrown oil is the early ranged attack for every class** (1d4×3×3
  ≈ 22, range 10; S15 INSTRUCTIONS §5); one flask killed each depth-6
  `p` human. The throw list has its own letters (`ui-object.c:855`) — a
  future executor `Throw` action must read them like the pack's.
- **A monster's dice are not its damage**: armour removes
  `damage × min(AC,240)/400` of a landed blow (2.5% at AC 10 — the hit
  roll is where AC pays; `mon-attack.c adjust_dam_armor`). Calibration
  input for M4.
- **Pickup and piles**: `g` on a pile opens a menu (T3.1's module docs);
  the C Borg instead walks with `pickup_always`. T3.4 decides the pickup
  option and records it in `docs/ENGINE.md`.
- **Rest policy** (c15 REST-01/02 via S15): rest when no awake monster
  can reach you in ~5 turns; rest on a stair when one is within ~10
  steps. Poison, stun or cuts stop rest — resting beside a spore monster
  never clears the confusion (S15, `minimaxflash-1.3`).

### T3.5 — town: sell, buy reserves

- **The reference loadout** (better-borg-c #410, from the owner's
  verified D0015 re-shop): 4 Phase Door, 3 CLW, single cheap utility
  scrolls, a flask of oil, ~20 gold of armour — against every recorded
  C-Borg death's pattern (300 of 360 gold on one armour, one Phase, no
  CLW, Recall left at Home: D0013, D0015, D0016, D0018, D0019).
  Reserves-are-invariants (GOALS principle 7) is what the owner keeps
  verifying manually.
- **The store identifies for free** (ACC-KNOWLEDGE §3, edition 3):
  `do_cmd_sell()` learns the flavour *before* the item changes hands
  (`store.c`), so with `birth_no_selling` a give-away identifies the kind
  at the cost of the item. Our `Sell` action doubles as identification;
  matches better-borg-c #412 (buy-to-identify) at 0 gold.
- **Shops restock once per game day and the keeper rotates**
  (`store_maint`, `store.c:1425`): "not stocked" is a statement about a
  day, not a seed (S15, `minimaxflash-1-again#399`).
- **Buy out the last row first** (S15, `minimaxflash-2#24`): the store
  list renumbers below a sold-out row; our one-buy-per-visit pattern
  (T3.1) avoids it, but a multi-buy visit must re-read letters.
- **Gold is the only loot** (S15 §6b): selling pays 0; an object is worth
  carrying only to use or wear; every harmful consumable has `cost:0` in
  `object.txt` — a kind's shop price is its triage once identified.
- **T3.3/T3.5's town model**: the Borg starts on the town's down
  staircase (`town_gen_layout`, S15 §"Where you are") — the first `>` is
  free of any search.

### M4 — danger and survival

- The five analysis issues read on this pass are the C Borg's failure
  catalogue for exactly this milestone: **#408** (`goal.fleeing`
  disables *all* healing and defence for the rest of the level —
  `borg-caution.c:1001`; REDESIGN-LESSONS L2's requirement is ours),
  **#409** (the walking retreat runs before any escape item and models
  no pursuer speed — `borg-caution.c:1430`; D0017/D0018), **#411** (the
  named scary-monster list sets whole-level flight — `borg-flow-kill.c`;
  D0019), **#340** (Speed never drunk), **#319/#320** (healing tiers,
  weapon valuation). Our design answers each by construction (one scale,
  modes re-evaluated, no name lists) — the issues are the *evidence* to
  cite in M4's review.
- **better-borg-c milestone 10** (#420, post-pin): I-001 (one-blow kill
  on a lone adjacent monster — the D0012 mold lesson) and I-002 (heal
  while fleeing at survivable danger — the D0017 lesson) went in;
  I-003 (starting purchases) was **rejected by its margins** and recorded
  as a negative result. When the zoo runs a v10 specimen, those are
  measured outcomes for the same decisions our tactical layer makes.
- Breath damage is current-HP/divisor with no save and no AC (acid
  excepted), a 30° cone, full to 7 squares then 8/(d+1); summoning caps
  by depth×level; +10 speed is double moves (S15 volume 2 §4–5 — all
  FACT with sources; the whole volume is the M4 checklist).
- **Escapes bought are escapes unused** (ACC-KNOWLEDGE §5): every run
  that bought Phase Door and CLW died or ended still holding them —
  the corpus's own validation of GOALS principle 5.
- **Unknown pools by depth** (ACC-KNOWLEDGE §4): at DL2 an unknown scroll
  is one of 8 kinds (17% Phase Door, 7% harmful); Magic Mapping cannot
  appear before DL5; one object in twenty is generated "great" from a
  deeper level (`obj-make.c`). The consumable-value model's priors.
- **Confusion randomises directions 75%** and blocks casting and running
  (`player-util.c:1353`): a confused Borg must not batch keys — the
  executor's per-action discipline covers it, but M4's status handling
  must treat a queued Rest during confusion as unsafe.

### M5 — readiness and progression

- The kit-gates table (S15 volume 2 §2) with its anchors, and the
  **Free Action gate sits seven levels after its first caster**
  (illusionist/druid at DL13, the gate at DL20) — a readiness policy
  must look at the band's casters, not the checklist's order.
- Gear is priced by **power** (`obj-power.c`), not `cost:`; armour buys
  misses, not padding; Berserk (+75 to-hit, fear immunity, −10 AC) is a
  bargain for the classes that can pay the rage. Equipment valuation by
  effect (GOALS principle 14) has its first data points here.
- The mana penalty is one spell point per **pound** of worn armour over
  the allowance (edition 3 re-corrected the correction: tenths cancel,
  `player-calcs.c:1505`); **pack weight does not count**.
- Mage: Magic Missile first (3d4 CL1–5, the only damage that ignores
  AC), study is free, one slot per level. Priest: prayers are random, no
  detection before CL5. Warrior: **can never dig** (`skill-dig:0:0`) —
  veins and their treasure are scenery for it.
- Locked doors: one in four, strength `randint1(7)`; a **failed pick
  prints `Cancelled` and eats the rest of the decision's keys**
  (`minimaxflash-1.4#616`) — the executor's expected-effect check will
  catch this if we ever batch door-picking; a picked lock leaves you
  outside.

### Cross-cutting

- **Detection marks its whole 81×45 window `SQUARE_DTRAP`, which stops
  the game's own run/`g` travel at every seam** (ACC-KNOWLEDGE §2,
  `effect-handler-general.c:1376`, `cmd-cave.c:1146`). Our Borg walks
  plain digits on its own flow, so it is immune — but this is why any
  future use of the game's travel commands after casting detection is
  dangerous.
- **A level left is regenerated**: map memory has no second use, both
  stair directions are exits, `<` `>` rerolls. The world model already
  forgets on level change; the *strategy* must not plan returns.
- ACC-KNOWLEDGE §8's open questions (explore-or-roll measured properly,
  coverage share after N decisions, Magic Mapping's yield, Deep
  Descent's depth field) are candidate experiments for our own trace
  data once T3.3 plays.

## What borg-runner can additionally provide

Copied now: the manuals, the notebooks, the write-ups, the overview,
`ACCUMULATED_KNOWLEDGE.md` (the export for us). Still at the pin, by
S15's selection rule:

- **`records/*/decisions.jsonl`** — every ask's screen, keys, turn and
  the model's stated reason: scenario material for M3's situation tests
  and a "what a manual-reader did" trace to compare decisions against.
  Large; read at the pin (`git -C vendor/borg-runner show HEAD:records/<run>/decisions.jsonl`).
- **The replay site** (diegoami.github.io/borg-runner) — step-through
  views of every run; useful when a notebook line is unclear.
- **`agents/<family>/`** — model families' notes to themselves about
  *playing*, not about Angband; indexed here only so nobody copies them
  as game knowledge.
- **Their `src/`, `prompts/`, `patch/`** — the runner tooling; not
  knowledge, and not ours to need.

If a future export edition or a new manual volume matters to a task,
extend the copy rule in a sources PR (RULES 21) rather than copying by
hand.
