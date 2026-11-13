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
| borg-runner (S15) | 2026-11-13, pin `e55ac6f` | everything we copy through `ACCUMULATED_KNOWLEDGE.md` **edition 7** (editions 5–7: their changelog paragraphs and the diff from edition 4), including the new `mmf-mage.3` (crow death), `mmx-pri-1`/`mmx-pri-1.2` (Dwarf Priest, two lives) and `mmx-ran-1` (first Ranger) knowledge and reports, `records/characters/`, and the `INSTRUCTIONS.md` diff ("Experience is the objective, not depth") | the report's copy lines: a changed `ACCUMULATED_KNOWLEDGE.md` means a new edition (read its own changelog paragraph, then the diff); new runs arrive as `knowledge/` + `records/` additions |
| better-borg-c deaths (S4, S9, S11) | 2026-11-13, pin `4603703` | every copied dossier D0001–D0019, `REDESIGN-LESSONS.md`; no new or changed dossiers between `a4cc2d4` and `4603703` (INDEX.md's additions are our own cases) | new death cases; a changed `REDESIGN-LESSONS.md` |
| better-borg-c issues (S8) | 2026-11-13 (commits only: the issue list is not reachable from cloud sessions) | the earlier analysis set and the M4 carries; the 10 commits `a4cc2d4..4603703`: M11 review corrections F1–F5 (`e8942e44`, docs and a patch file, no `src/` change), the v11 tag record (`368efd4e`), the newkill-95 carry (`d4175367`, `borg-flow-kill.c:borg_new_kill`, neo-angband-tracker#95); tag `4.2.6-impact-v9.1` (`618f1554`, release packaging) | the report's `issues updated since` list; any commit touching a `borg-*.c` function we cite |
| neo-angband-tracker (S2, S14) | 2026-11-13, pin `c2ddf2c` (no copied path changed since `777eea1`) | the C Borg spec and the Tunable Plan at their pins; the strategy manual's chapters and appendices, and the official user manual (`reference/c-manual/`, `user-manual/`), as tasks need them (ch. 15 for M3) | when the pin moves (`--update`); otherwise per task, at the chapter or manual page the task cites. The strategy manual is the first reference for game behaviour; every number is checked against `engine/angband/` |
| borg-zoo (S6) | 2026-11-13, pin `c5f1b04` | the DESIGN and catalogue diff: two dashboard tabs with one mode policy each (plain = `normal` post-cutoff, a build's own `default` pre-cutoff; a point needs a full class run, `DASHBOARD_MIN_CELLS = 1500`); new specimens `4.2.6-reborn-rust-e` (ours, tag `e`), `4.2.6-bbc-m11`, `4.2.6-rs-m11`, `rs-family-buying`, `rs-town`, `angband-42cd156-original`, `neo-765-original`; buying, switch-screen and town-negative profiles; the adapter's `clevel` stop mapping (`2416bb9`, borg-zoo#48) | changed catalogue/DESIGN copies; our specimens' issues; any `classes-1800` control rerun |
| borg-rs | 2026-11-13, pin `4ed3263` | the v11R carry (PR #30, 18 units of better-borg-c M11, owner-accepted 2026-11-11, no cohort or benefit claim); buying family #26/#28 merged (opt-in, no tag); defaults audit: `borg_munchkin_by_class` built-in default back to FALSE (TRUE only via borg.txt), FORK-CORRECTIONS amendment | family changes that affect control interpretation; buying-strategy measurements |
| angband | 2026-11-10, pin `42cd156` | the 12-commit log since our engine pin: rest interruption/completion, command flushing and Borg reaction changes noted for M4's executor work; our engine remains `e473720` byte for byte | only when a lifted executor or combat rule needs its citation re-checked |

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
- **M4 source refresh, 2026-11-10 (better-borg-c `a4cc2d4`, M11 #471):**
  the five `borg-danger.c` corrections read directly. Partial melee divides
  armour by the monster's to-hit (source #63) and caps the luck-adjusted
  chance at 100%; constitution drain tests CON, not STR (#61); Crush clears
  the *ranged* value (#66); passable rubble is not granite in the ranged
  step (#68). Shoot'N'Scoot also passed the wrong monster index (#100).
  These are inputs to our expected-damage tests, not a licence to copy the
  old full-damage approximation. M11 has **no cohort/per-mode measurement**
  and its tag waits for review; none of its 18 units proves a survival gain.
- **M4 source refresh, borg-zoo `415183c`:** the mode cutoff is real:
  `default` is not comparable across it. Use the explicit `normal`
  profile (all munchkin/Lunal flags off) for our control comparisons.
  v8–v10 specimens are present; v11 is unmeasured and untagged. The
  reserve410 buying pilot is mixed (10/12 vs 11/12 alive, with one paired
  loss and two gains), not evidence for our reserve defaults.
- **M4 source refresh, borg-runner `1440f70`, edition 4:** new fixtures:
  fear is −20 to-hit, +8 AC and **no attacks**, cured only by the timer
  (two crows in `minimaxflash-1.5`); the shrieker wakes everything in line
  of sight (it does not summon new monsters). The M map covers the whole
  level scaled, and stair priority 25 means a remembered staircase cannot
  be hidden by scaling. Monsters are sparse, asleep and deliberately
  distant, so detection's door map and the experience cost of searching
  matter as much as combat. `minimaxflash-2.2` adds Fang AC36: oil is a
  12% lottery where Magic Missile ignores armour, and Phase came too late.
  Its mana-penalty paragraph repeats the factor-of-ten error; our ledger's
  engine-checked rule remains one SP per **pound**, worn armour only.
  `mmx-war-1` adds the batching/breeder death: a 60-key chain kills 35
  silver mice and the multiplied pack kills the character; a visible pack
  means one action per think, exactly our executor's discipline. Magic
  Mapping's window found no staircase and is a purchase to measure, not
  an automatic answer.
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

- **M5 source refresh, 2026-11-13 (borg-runner `e55ac6f` edition 7,
  better-borg-c `4603703`, borg-zoo `c5f1b04`, borg-rs `4ed3263`).** All
  single-run evidence; nothing here is measured by borg-zoo (RULES 14).
  - **Readiness is the destination's worst round, not its experience**
    (`mmf-mage.3`): a CL1 Mage (8 HP) went to DL2 and died to a crow
    (`sleepiness:0`, placed awake, `mon-make.c:1141`; speed 120, two 1d3
    bites) with the up staircase five tiles away. INSTRUCTIONS now says
    to stay until the deeper level's worst round is survivable and to
    arrive with full SP. For T5.1: evaluate awake and fast monsters at
    the destination, and keep the arrival staircase as the retreat.
  - **Experience, not depth, is now the corpus's stated objective**
    (edition 7 §1, an editorial change: 21 runs, 7 deaths, all at DL ≤ 2
    and CL ≤ 4). Consistent with GOALS principle 6; no threshold follows.
  - **Deep Descent is `max_depth` + 5**
    (`effect-handler-general.c:1166-1168`; blocked only when the target
    is not deeper). An unknown scroll at DL1 is Deep Descent 6.9% of the
    time (`mmx-pri-1.2#176`), so reading unknown scrolls is a readiness
    decision.
  - **Class kit:** a Priest gets one blow with any weapon; thrown oil
    hits at `35 − distance`. A Ranger's CL1 Short Bow (1d4×2) is no
    better than the dagger (`player-calcs.c:2281-2282`: shots in tenths,
    `lev/3`); oil is the early weapon; arrows cost 1 gold at the General
    Store (`mmx-ran-1#19`). Soft Leather Armour costs 84, so purchase
    order decides whether AC is bought. A Dwarf's digging turns veins
    into income. The Mage's cast menu lists studied spells only, and a
    letter moves the cursor rather than casting (an executor fact for
    T5.3).
  - **Magic Mapping coverage disagrees** between ACC §8.4 (16%, "a
    convenience") and `mmx-pri-1.2#184` (27% window, ~65% staircase
    chance): a purchase to measure, not a rule.
  - **For the M4 threat model:** breeders are a per-decision HP tax (22 HP
    in one 38-tile walk, `mmx-pri-1#81-82`); the silver mouse appears on
    DL2 in packs of 11; `sleepiness:0` races start awake, a prior our
    awake/asleep split could use.
  - **G1:** the zoo catalogues our milestone-E specimen
    (`4.2.6-reborn-rust-e`, before M3/M4) and reran `angband-e473-original`
    on `classes-1800` on 2026-11-11 (`94f54bb`, descriptive): a candidate
    control for T5.4. Its numbers are not read yet.
  - **Citations:** better-borg-c `d4175367` moved
    `borg-flow-kill.c:borg_guess_race` up 18 lines (now 874-889; body
    unchanged); `observation.rs` cites the new lines. The carry itself
    (`borg_new_kill` no longer forces PASS_WALL grids to floor) has no
    counterpart here.

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
- **`tools/screen/`** — the runs' shared screen-parsing library
  (`screen.py` and look/aim/map/diff/explore over it). Read 2026-11-10;
  assessment: its *purpose* (giving a text-screen reader eyes) is not
  ours — our observation reads the engine, not the text — but three
  things in it are ours to use:
  - **`diff.py`'s principle** ("the screen is identical and the turn has
    not moved is the tell of a key that did nothing") is the same
    invariant as our executor's expected-effect check, at screen
    granularity — independent validation of the design;
  - **its `PROMPTS` inventory** is a second list of the game's prompt
    texts, cross-checked against `ui-input.c` on 2026-11-10: the
    engine's direction prompts all start "Direction"
    (`ui-input.c:1512,1632-1634`), exactly what our catcher matches;
  - **the status line it parses exposes a fair read we do not take**:
    the level feeling `LF:n-m` (the printed danger digit is 10 − the
    internal one, S15). The C Borg reads the feeling, so it is inside
    D3; our observation does not have it. Candidate for T3.4/T3.5 as a
    leave-level signal (c15 STAIR-01) — needs a `docs/ENGINE-READS.md`
    row if taken.
  The KEYPAD-transposition war story in its header ("the first tool had
  the table transposed, so every route walked the wrong way") is the
  argument for our DIRS unit tests.
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
