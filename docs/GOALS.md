# Goals

*Step 2 of the initialisation: what we build, what "better" means, and the
decisions the owner confirms at the milestone 0 review. Evidence is in
[SOURCES.md](SOURCES.md).*

## The question

The owner asked which of these the Borg reborn should be:

| Option | What it means | For | Against |
| --- | --- | --- | --- |
| **A. One Borg reborn** | One new design, hand-built, replacing the C Borg's ladder | Simple to explain; one thing to improve | Class matters more than version (19 % to 70 % early deaths by class on the C Borg); one design fits some classes and hurts others (munchkin start: Warrior deaths 32 % → 7 %, Mage deaths 31 % → 45 %) |
| **B. A family of competing Borgs** | Several Borgs, different designs, the zoo picks | Matches the per-class evidence; competition finds what works | Each Borg rebuilds perception, map memory, flow and items, the expensive part that differences in play do not come from |
| **C. A Borg with optimisable parameters** | One design whose decision constants are named, ranged and tuned by experiment | Measurable; reversible; the TUNABLE plan's whole method | Tuning moves thresholds, not structure; several C Borg deaths come from structure (rules that never compare alternatives) |
| **D. A learned policy** | Reinforcement or imitation learning at the core | Unbounded in principle | Long horizons, sparse death signal; symbolic bots still beat neural ones on NetHack; loses determinism and explainability; the TUNABLE plan advises against starting here |
| **E. A model in the loop** | A language model chooses actions | Uses general knowledge | That is borg-agent's job, for *knowledge*, not play; slow, costly, non-deterministic |

## Recommendation: one substrate, a family of strategies, all of them tunable

**Build one shared substrate and a family of strategies on it. Every
strategy is parameterised, and the zoo decides between them.** This is
A, B and C together, each where it pays:

1. **One substrate** (built once, shared by every strategy): the engine
   seam, perception, the world model (map, monsters, objects, the
   character), game knowledge, path-finding, the **danger model**, the
   action executor (key sequences and prompts) and the decision trace.
   Differences in play do not come from these. Building them twice would
   only add places for bugs. This is where borg-rs's code and the C Borg's
   game knowledge are reused.
2. **A family of strategies** behind one Rust trait: what to do now, given
   the world model and the danger assessment. The first strategy is
   generic. It splits by class only when a class's evidence shows it needs
   a different *structure*, not just different numbers. Casters (spells,
   mana reserve) and archers are the likely first splits. Rival designs of
   one decision (for example two fight-or-flee evaluators) are strategies
   too, and compete on the same seeds.
3. **Every strategy is tunable**: its decision constants are named
   parameters with a unit, a range and a default, read from a profile file
   and written into every run record. A **profile** is a strategy plus its
   parameter values; profiles per class are expected.
4. **The zoo decides.** A strategy or profile is kept when it beats the
   current best on the prespecified objective, on seeds it was not
   developed on. The **controls** in every comparison are C Borgs run under
   our exact run contract (D8): the original C Borg (`c-pristine`) and the
   latest better-borg-c milestone (`c-bbc-m7` today). borg-rs is not a
   control: it is the source of our engine tree and tooling, and the
   perception oracle (T2.5, #16). Its zoo results, like the observer
   harness's, are context.

Later, and only for narrow decisions: the decision trace (which strategy
and which rule chose each action, with the alternatives' scores) is the
dataset a learned component would need. Nothing in the substrate prevents
it, and nothing in the plan depends on it.

## What "better" means

### The objective

We adopt the Tunable Borg Plan's doctrine ([SOURCES.md](SOURCES.md#s3-the-measurement-doctrine-we-inherit)):

- **Primary endpoint, per class:** the share of runs **alive at engine turn
  T having reached dungeon level D**. Every attempted run counts. Stalls,
  timeouts, crashes and harness failures are non-successes, reported
  separately, never relabelled as deaths.
- **Reported alongside:** deaths with cause, survival curves by engine turn,
  maximum depth and character level, operational outcomes.
- **Comparison:** per class, against the control C Borgs (D8), on frozen suites, with
  paired seeds where the harnesses allow it. A claim of improvement needs
  a prespecified endpoint, a sized comparison and a confirmation suite
  (borg-zoo RULES 8).

Survival is the main goal but not the only one. A Borg that stays in town
never dies, which is why the endpoint requires depth. The progress floor
(D) rises milestone by milestone.

### Observation and scoring rule

Fixed now, before any comparison, following the Tunable Borg Plan's rule
of the same name:

- **Observed at the exact boundary.** Alive, depth and maximum depth are
  taken at engine turn T itself, by our `borgh` (T1.4, #7), not after the
  first key at or past T as borg-rs's `borgh` does today.
- **Success:** a verified victory before T, or alive at T with maximum
  depth ≥ D. The first engine turn at each depth is recorded too.
- **Denominator: every attempted run** of the arm, per class. Stalls,
  timeouts, crashes, harness failures and configuration errors are
  non-successes, reported per arm and never relabelled as deaths.
- **Arms:** the reborn Borg and the control C Borgs, all built from this
  repository on the same engine tree, run through the same `borgh` and the
  same run contract (D8, T1.3, #6). Results from the better-borg-c
  observer harness are context, not controls: same-seed equivalence
  between the two harnesses is unverified.
- **Calculated by** `tools/endpoint.py` (T1.7, #10) from our outputs or
  from borg-zoo result cells. borg-zoo's own `goal_rate` excludes
  operational outcomes from its denominator and is reported per arm rather
  than per class, so it is **not** this endpoint.

**Prerequisites of G1:** T1.3 (controls), T1.4 (run contract), T1.7
(calculator), and the control and reborn specimens in borg-zoo's
catalogue (T1.8, routed by the owner). Until then, numbers are descriptive.

### Staged targets

| Stage | Target | Suite (borg-zoo) | Milestone |
| --- | --- | --- | --- |
| G0 | **Plays.** Starts, explores, fights, uses stairs and town. No stalls, no crashes, no harness failures on the smoke suites; same seed twice gives the same keys | `smoke`, borg-rs `smoke` | M3 |
| G1 | **Not worse early.** Per class, primary endpoint at T = 100,000, D = 5 is not worse than the `c-pristine` control (non-inferiority, margin agreed in advance) | `classes-1800` (9 classes × 200) | M5 |
| G2 | **Better early.** Per class, better than the latest better-borg-c control (`c-bbc-m7` today) at T = 100,000 with D raised (D = 10 proposed), confirmed on fresh seeds | `classes-1800`, confirmed on `classes-1800-b` | M6 |
| G3 | **Better deep.** T = 400,000, D = 20 or more, per class | `deep-1000` and new per-class deep suites | M7+ |
| G4 | **Wins.** Kills Morgoth without the excluded modes | long runs | long term |

D, T and the margins in G1–G2 are proposals; the owner fixes them before
the comparison that uses them (D7 below).

## Principles

These follow from the sources. Each cites the evidence behind it.

1. **Same seam, same harness, same commands as borg-rs** (owner
   requirement). The reborn Borg is `BORG_IMPL=rust` behind `inkey_hack`,
   as borg-rs's Rust Borg is. `borgh`, the launchers, replay and death
   dossiers work the same way, and the zoo can measure it with the borg-rs
   adapter. Our harness adds the exact-boundary observation and the depth
   stream (T1.4, #7) as harness-only hooks.
2. **Deterministic.** Same seed, build and profile, same key stream. The
   Borg thinks with its own RNG and never moves the game's. Wall time
   never affects play (spec §1.2, §16.1.4).
3. **Fair information.** The reborn Borg perceives what the C Borg
   perceives, no more (spec §1.1, §16.1.3). Otherwise the zoo compares
   different games.
4. **Danger is the central concept** (owner direction), redesigned. Its
   unit is something an evaluation can compare (expected and worst-case
   damage over a horizon, and the chance of dying), not a capped scalar
   (spec §8).
5. **Alternatives are compared on one scale.** When the character is
   threatened, every available response (attack, shoot, cast, quaff,
   read, step, stairs, escape) is scored against the same risk model.
   The first applicable rule does not decide. *Evidence:* unused Cure Light
   Wounds (bbc D0002, rs D0005), the Ranger who fled instead of shooting
   (bbc D0010).
6. **Commitment and progress.** A goal, once chosen, keeps priority
   (hysteresis) until it succeeds, becomes unsafe, or exhausts a budget in
   *game* turns. A goal that does not progress is blocked for a while, and
   loops in which no game time passes are detected. *Evidence:* mold
   oscillation (bbc D0008, rs D0006), starvation by pickup loop (bbc D0009),
   munchkin loop ([#403](https://github.com/diegoami/neo-angband-tracker/issues/403)).
7. **Survival invariants are explicit.** Light, food, a cure and an escape
   are reserves with declared minimums per depth and class, checked every
   think. They are not side effects of the shopping list. *Evidence:* bbc
   D0001 (Phase Door at home), bbc D0009 (food), owner priority
   [better-borg-c#41](https://github.com/diegoami/better-borg-c/issues/41).
8. **Depth readiness is one policy, per class.** Stairs, Deep Descent,
   trapdoors and recall depth all go through it. *Evidence:* bbc D0006 (Deep
   Descent to depth 14); TUNABLE plan (the biggest single lever).
9. **No exploits** (owner decision). No munchkin start, no lunal mode, no
   money or self scum, no stair-scumming to a target depth, no cheat death
   inside the Borg (it is a runner setting).
10. **One state, never mutated by a read.** The world model is updated once
    per think from perception. Decisions read it. Simulations work on
    copies. Rust's ownership rules enforce this (spec §11.1, §11.6, §17).
11. **Explainable.** Every think writes a decision trace: mode, goal,
    candidate actions with their scores, choice, parameter-profile hash.
    Death dossiers read it; no death should need a source-level guess to
    explain.
12. **Changes are judged on populations, not on the case that motivated
    them.** Fixes interact: a better-borg-c carry that rescued seed 29
    killed seed 47 (better-borg-c#249, #255). A death fixture shows that a
    rule fires. Only a paired population comparison shows that it helps.
13. **Knowledge is carried over, code is not.** The C Borg's game knowledge
    (blow and spell damage, item values, prompt answers, store routines)
    is lifted with a `file:function:lines` citation at the pin. Its
    decision code is not translated.

## Non-goals

- Parity with the C Borg or borg-rs. Neither is an oracle for our
  decisions. The C Borgs are controls for our results (D8); borg-rs is
  an oracle only for perception (T2.5, #16).
- A model in the play loop, or machine learning at the core.
- Other engines (Neo Angband, variants) before the C 4.2.6 game is beaten.
- Fixing C Borg bugs. A C Borg defect we find is reported to the owner for
  routing to neo-angband-tracker.

## Owner decisions

**Accepted** (2026-11-09): the owner merged the milestone 0 review (#1)
with D2, D7 and D8 as amended by the review fixes (#2), and no other
change. A session does not reopen them (RULES 3); the owner may amend
them.

| # | Decision | Accepted |
| --- | --- | --- |
| D1 | What the reborn Borg is | **One substrate, a family of strategies, all tunable; the zoo decides** (above) |
| D2 | Engine and game | **Angband 4.2.6 at `e473720`**, the `angband/` tree of borg-rs (harness hooks, its declared fork corrections and its adopted C Borg carries) imported with history. Birth options at the game's defaults. The tree's own C Borg is not a control (D8) |
| D3 | Information level | **The C Borg's two channels, no more**: screen (message line, store header) plus the direct reads of spec §1.1. Revisit only with a separate "fair-play" profile, measured separately |
| D4 | Code reuse | **The substrate may reuse borg-rs code** (engine boundary first; perception, item analysis and flow where they survive review), with attribution. Decision code is new. C Borg knowledge is lifted with citations |
| D5 | Excluded | Munchkin start and lunal mode (already decided); money scum, self scum, the formula language, `borg_worships_*`, cheat death and respawn inside the Borg (proposed) |
| D6 | Strategies by class | **One generic strategy with class profiles first**; a class gets its own strategy when evidence shows it needs a different structure |
| D7 | Objective | Primary endpoint and **observation and scoring rule** as above (exact boundary, all attempted runs, per class, computed by `tools/endpoint.py`); **G1 at T = 100,000, D = 5**; D, T and margins fixed before each comparison |
| D8 | Controls | **Same-frontend controls built here** (T1.3, #6): `c-pristine` (upstream `e473720` C Borg, checked against the zoo's `4.2.6-rs-m0-c`) and `c-bbc-m7` (better-borg-c tag `4.2.6-impact-v7`), on our engine tree through our `borgh`. A later better-borg-c milestone is a new arm. **borg-rs is not a control**: it is the source of the engine tree and tooling and the perception oracle. Its zoo specimens and the observer-harness zoo results are context only |
| D9 | Workflow | **Milestones with a review PR** (borg-agent's method): task PRs merged by the session once CI is green; at a milestone's end the session stops and opens a PR from `main` to `reviewed`; the owner merges it |
| D10 | Compute | Identity and regression suites locally under `/tmp/angband-validation.lock`; population comparisons through borg-zoo on GitHub Actions, started by the owner |
| D11 | borg-agent | Its reviewed claims are inputs to strategy and parameter design. No dependency on it in code or in the plan's critical path |
