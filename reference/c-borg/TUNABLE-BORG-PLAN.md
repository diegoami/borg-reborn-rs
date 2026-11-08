*better-borg-c and borg-rs · evaluation and plan · revision 3, 2026-11-07*

> This file is the reviewable source of the plan; a rendered copy is published as a private page for the owner. Revision 2 answered the review in [#360](https://github.com/diegoami/neo-angband-tracker/issues/360) of revision 1 (`7e008cc93`); revision 3 answers the re-review of revision 2 (`e3c4fefae`) on [PR #361](https://github.com/diegoami/neo-angband-tracker/pull/361). Live state (issues, labels, carries) belongs to the boards of neo-angband-tracker, better-borg-c and borg-rs, not to this file.

# Tunable Borg Plan

How to get a Borg that we can tune and improve by experiment, without throwing away what the C Borg knows and without being bound to every decision it made. Two repositories share that goal: better-borg-c (native C) and borg-rs (Rust).

> **Recommendation.** Keep the C Borg as the working engine and grow a policy layer on top of it, in three steps that each pay off on their own:
>
> 1. **Expose parameters with baseline defaults.** Move the decision constants that matter for survival into configuration. With every parameter at its default, the build must reproduce the trace of the frozen *carry baseline* it was built from (see Baselines below), byte for byte, on the identity and development seeds (never the sealed test set).
> 2. **Tune by racing on a frozen seed population,** with a prespecified objective, survival statistics and held-out seeds.
> 3. **Replace modules one at a time** (caution, dive policy, fight-or-flee) behind switches. A replacement is kept only when it beats the original on the same harness. The original module stays as the reference.
>
> Do not rewrite the Borg from scratch, and do not start with machine learning. Both throw away tested behaviour before we can measure what it is worth.
>
> **First step: bring the fixes into borg-rs.** Freeze an ordered manifest of better-borg-c's carries and port it to borg-rs, one commit per fix. Accept it when the borg-rs suites produce the same key logs as a C oracle built from the same manifest, with a short translation audit for every fix (R0–R1 below). From then on both repositories start from the same baseline.
>
> **Two repositories, one contract.** Both use the same parameter names, profiles, seed sets, calendar policy and outcome definitions. Key-stream identity between C and Rust at the same profile is evidence that the *tested executions* agree; it does not prove every fix or parameter correct, so unreached changes keep a source review. Before building full parameter layers in both, run a small pilot in each (Phase 2a) and decide from it where new policy work lives.

## What we already have

### neo-angband-tracker and better-borg-c

- **A specification of the C Borg** (`docs/c-borg/BORG_SPEC.md` in neo-angband-tracker, merged with PR #317 as `8300a8521`), cited at the upstream pin `e473720`. It covers perception, the dungeon decision ladder, flow, danger, caution, depth preparation and modes: the map of where the decisions live.
- **A bug-fixed C product.** better-borg-c carries **34** verified fixes at `702f091` (`patches/series.json`; 31 at `8a7eba1ad`), each with its source and applied commits, consumer issue, verification document and removal unit.
- **A native measurement runner** (`tools/measure-native.py`): ordinary birth, the production game loop, both RNG states logged, an explicitly instrumented observer build. It classifies runs as death, successful completion, turn-cap survival, wall timeout, stall, crash or harness failure. Qualified repeats under recorded conditions are byte-identical; seasonal monster selection still depends on the calendar, and not every complete trajectory has been repeated. Recorded run times: about 95–113 s for 400,000 engine turns and 8–15 s for 20,000 (particular trajectories and builds, run one at a time).
- **Death dossiers** (`docs/deaths/`) with T-minus observations from replays, plus a death-analysis plan.
- **An external tuning language.** `borg.txt` has a formula section (lines 340–1129) with *Power*, *Depth Requirement* and *Restock Requirement* blocks. With `borg_uses_dynamic_calcs` set, `borg_power_dynamic()` replaces the **equipment and inventory** contributions to `borg_power()`, `borg_prepared_dynamic()` replaces the **static per-depth branch** of `borg_prepared()`, and `borg_restock()` dispatches to `borg_restock_dynamic()`. What stays in code: the outer `borg_power()` (the 40,000-per-prepared-depth bonus and the swap values) and the later preparation gates (king, town, the *Healing* stock, the unique gates). A formula change cannot move those.

What is missing is scale, and a tuning surface for everything else. So far the cohorts reuse three starting seeds (11, 29, 47). Danger, caution, escape, healing, flow fear and the ladder order are hard-coded: a lexical count finds 4,594 decimal literals of two or more digits in the Borg's `.c` files (comments, strings and character literals removed), 713 of them in `borg-power.c`, 376 in `borg-danger.c` and 179 in `borg-caution.c`. That is an inventory, not a count of independent decisions.

### borg-rs

- **A faithful Rust port of all 59 modules** (73,079 lines across `src/**/*.rs`, declarations included; M1 deterministic C harness, M2 c2rust transpile and M3 safe idiomatic Rust done). `unsafe` is confined to the engine boundary (`engine.rs` and the engine declarations). It links behind the same `inkey_hack` seam and drives the unmodified C engine.
- **Equivalence by key stream.** For the same seed the Rust Borg's key log is byte-identical to the C Borg's on all **80** suite runs: smoke 3, full 45, deep 9, classes 16, config 7. `harness/diff` reports the first divergence by seed, turn and key; each build records a `BUILD_ID`.
- **A headless harness**: the `borgh` frontend (`-DSUPPORT_BORG_HARNESS=ON`, hooks under `#ifdef BORG_HARNESS`) with scripted birth, a fixed seed, a fixed holiday date and a fixed 80×24 terminal. The full suite took 137.85 s with four jobs on its four-core machine.
- **Borg state centralised.** Former C globals live in `BorgState`, one sub-struct per module. That makes serialising Borg memory and swapping modules much more practical than in C, though not automatic (see R4).
- **Narrower scope today.** C bugs are kept on purpose (`docs/c-bugs.md`), none of the 34 carries is ported, verification reaches no deeper than dungeon level 27 (no Morgoth, no self-lunal thinker; several suites use cheat death), and "faithful port, no new tactics" is the rule.

### How the two compare for this goal

|  | better-borg-c | borg-rs |
| --- | --- | --- |
| Bug fixes | 34 verified carries | none yet (faithful to the pin) |
| Tracks upstream C | directly (same tree) | by porting each upstream Borg change |
| Measurement | native runner, outcome classes, death dossiers, frozen cohorts | seed suites, key-stream identity, build identity |
| Adding parameters | C edits across many files | fields on `BorgState` sub-structs |
| Replacing a module | preprocessor or function-pointer switches | Rust traits; the compiler checks the interface |
| Saving Borg memory | hard (scattered globals) | more practical (one struct), still real work |
| Correctness oracle | pristine C trace, then its own carry baseline | the C key log, then the C carry baseline at the same profile |

> **Scope and ownership decisions are needed first** (listed in full under Decisions). Each repository's rules currently exclude part of this plan, and each track needs an owning session.

## Approaches compared

| Approach | Keeps C Borg knowledge | Freedom to change decisions | Cost and risk |
| --- | --- | --- | --- |
| **A. Tune existing knobs only** (35 `borg.txt` settings plus the formula section) | all | Low: the formula-controlled parts of power, preparation and restock only | Cheapest; can start at once. Hits a ceiling quickly. |
| **B. Parameterise in place** (constants to config, defaults = baseline) | all | Medium: thresholds and weights move, the structure stays | Moderate work. Identity at defaults makes regressions detectable on the tested runs. |
| **C. Module replacement on B** | as reference | High within a module, including rung order and new rules | Each module is its own project; measured and reversible. |
| **C′. Policy development in the Rust port** | all (port is key-identical) | As C, with safe refactoring and traits | Fixes must be ported first; every upstream change is ported by hand. |
| **D. Rewrite from the specification** | mostly lost | Total | Very high; years of edge-case handling would be rediscovered. |
| **E. Learned policy** | indirect | Total in principle | Long horizons, sparse death signal, huge state; at most later, for one narrow decision. |

The plan is **A, then B, then C or C′**: a gradual replacement in which the existing Borg keeps doing perception, flow, items, stores and key generation, and decisions are taken over one surface at a time. The strongest alternative is to do B and C only in Rust from the start, which avoids building two parameter layers. The plan does not settle that by argument: Phase 2a pilots both and decides.

## Baselines and identity

The plan uses three kinds of identity, and keeps them apart:

- **P0, the pristine reference:** upstream `e473720`, no fixes. Used to show what a fix changed. Fixes are *meant* to break identity with P0.
- **Bn, a carry baseline:** an immutable, ordered manifest of carries (patch IDs and their applied commits) frozen at a named better-borg-c commit. B1 is the manifest R1 starts from. A later carry does not change B1; it goes into B2, measured as a new baseline. Results stay attached to the baseline they were measured on and are never relabelled.
- **Profile defaults:** the profile `baseline-Bn` sets every parameter to its default. Its trace must equal the Bn build's trace, byte for byte, on the identity and development seeds (never the sealed test set). That is tested equivalence on those executions, not zero risk. A parameterisation must keep integer widths, signedness, evaluation order, rounding, short-circuiting and the Borg's private-RNG draw order exactly.

## Architecture

- **Substrate (the existing Borg).** Perception (`borg_update()`, the direct reads), memory, flow (`borg_flow_*`), item handling, stores, the key queue and prompt answering. Bug carries go here, as today.
- **Parameter layer.** One table of named parameters with a default equal to the baseline constant, a valid range, a source location and a unit (engine turns or Borg-clock ticks, which differ). It is read from `borg.txt` (or a sibling file) at start and dumped into every run record. The formula language's `CONFIG` value type can read the same table.
- **Module switches.** A few seams where a variant can replace the original decision: `borg_caution()` steps, `borg_escape()`, `borg_heal()`, the dive policy around `borg_prepared()`, and the rung order in `borg_think_dungeon()`. Variant 0 is always the original code path.
- **Profiles.** A named profile is a complete parameter set plus module choices. `baseline-Bn` is the control in every comparison on Bn.

### The shared contract between the two repositories

- **One parameter table:** the same names, units, ranges and defaults in both, read from the same `borg.txt` syntax, so a profile file works unchanged in C and in Rust (for the parameters both implement).
- **One run contract:** the same seed sets, birth script, terminal size, options, data files, **calendar policy** (a fixed date, as `borgh` uses), engine-turn caps and logging semantics. Without all of these, "same seed and profile" does not mean the same game.
- **One outcome schema:** death (with cause), successful completion (victory), turn-cap survival, wall timeout, stall, crash and harness failure. Crashes, stalls and timeouts are operational outcomes counted per profile; they are never silently right-censored as if unrelated to the policy.
- **Differential identity:** at a profile both repositories implement, the key streams must match on the identity and development seeds. A match is evidence for those executions only. Changes those runs do not execute keep a source review.
- **One results store,** keyed by repository, baseline, build ID, profile hash and seed.
- **Two frontends, kept apart in purpose:** `borgh` for population and identity runs, better-borg-c's observer build for causal evidence and death dossiers. Neither validates the other automatically.

## The first tuning surface

These are the decision points the specification identifies as the main levers for survival and progress. Start with a small, justified subset; about 40–60 parameters is a ceiling, not a first batch.

| Group | Where (pin) | Current values | What it trades |
| --- | --- | --- | --- |
| Flow fear thresholds | `borg_flow_spread()`, `borg_flow_enqueue_grid()` | sequential overrides of `avoidance`: × 3/10; × 5/10 at **maximum** character level 50; × 2 scary; × 3 (two cases); × 5 ignoring; × 25 after 5000 **Borg-clock** ticks on the level; × 100 starving; × 3/10 at character level 0 | Routes through danger vs. detours and stalling |
| Bravery boosts | `borg_think_dungeon()` | 2×`BI_CURHP`, 4×`BI_MAXHP`, 30000 | Getting unstuck vs. walking into lethal grids |
| Retreat and flight | `borg_caution()` | retreat at danger > avoidance/3; leave after more than 3 escapes (with exceptions); desperate stair dashes at distance `< 3`, `<= 2`, `< 20` (with extra conditions) | Early disengagement vs. lost experience and loot |
| Escape choice | `borg_escape()` | allowed failure 10 / 25 / 35 %, plus other hard-coded allowances; `risky_boost` 0 or 3 | Teleporting too late vs. wasting escapes |
| Healing | `borg_heal()` (in `borg-caution.c`) | hit-point-loss bands vs. random chance thresholds; +5 when risky | Potions spent vs. deaths with unused cures |
| Depth preparation | `borg_prepared_aux()`, Depth section | per-band requirements; "maximum character level (`BI_MAXCLEVEL`) < depth" rule (dynamic mode changes which branches apply) | Dive speed vs. readiness. Probably the biggest single lever. |
| Restock | `borg_restock()`, Restock section | food, light, cures, escapes per depth | Trips to town vs. running dry |
| Time on level | boredom (10000), `borg_time_to_stay_on_level()` | as coded; Borg-clock ticks, not engine turns | Farming experience vs. exposure |
| Stair leash | `borg_think_dungeon()` | weak characters return when the nearest up stair is more than `BI_CLEVEL`×3+14 **grids** away (a distance, not a time) | Exploration range vs. a safe exit |
| Anti-bounce | `borg_check_bounciness()`, `borg_danger()` | 200 / 300 / 500 / 700 selected; 1200 and others remain | Loop breaking vs. discarding danger knowledge |
| Power weights | Power section | weights as in `borg.txt` (formula part only, see above) | What kit the Borg values |
| Mana reserve | `borg_spell_okay()` | per class from character level 35, with spell exceptions | Escape mana vs. offence (late game only) |
| Existing switches | `borg.txt` | `borg_plays_risky`, `borg_kills_uniques`, `borg_worships_*` (weight-branch selectors, not multipliers), `borg_uses_swaps`, munchkin, self-lunal | Style presets |

## What "best" means

The primary objective is **prespecified before any search**: the share of runs **alive at turn T having reached dungeon level D** (for example dlvl 10 at 400,000 engine turns). This rewards neither a Borg that never dives nor one that dives and dies at once. Report alongside it:

- milestone attainment (reached D by T) with death as a competing outcome;
- restricted mean survival time in engine turns, with Kaplan–Meier curves; turn-cap survival is censored;
- distributions of maximum depth and character level;
- operational outcomes (stalls, timeouts, crashes) per profile.

Before selection, fix a **progress floor** and a **survival tolerance** relative to the baseline profile. A survival/progress Pareto front is useful for exploration, but it is not an acceptance rule: a never-diving profile can stay on it. The endpoint is a bounded milestone: it accepts a Borg that reaches D and then shelters until T, so later campaigns add deeper milestones and eventually victory. Choose D and T from the Phase 0 pilot.

### Observation and scoring rule

Fixed before the Phase 0 cohort, and the same in both repositories:

- **Observe at the exact boundary.** Alive and depth status are taken at engine turn T itself, as better-borg-c's native observer does after each world-turn increment. `borgh` today checks its cap only after a returned key, and checks the cap before death, so a run can overshoot: a 100,000-turn cap has ended at 101,191. The population runner must stop exactly at T, or every result must name the common boundary rule it used instead.
- **Record attainment time:** the engine turn at which the run first reached D.
- **Score over all attempted runs.** Success is a verified victory before T, or verified alive at T having reached D. Every other run is a non-success in the primary score. Operational outcomes (stall, wall timeout, crash, harness failure) are non-successes there, are reported separately per profile, are never relabelled as deaths, and are never silently censored in the supporting survival curves.
- **Powering:** the sample-size table above is for death rates. The composite endpoint's rate can differ substantially, so the Phase 0 pilot measures it and sizes comparisons from it.
- **Suites keep their semantics.** R0/R1 keep the existing borgh suite semantics, so the borg-rs references stay valid. The exact-boundary observation is the part of R2 that is a prerequisite for shared Phase 0 measurement.

### How many runs a decision needs

With a two-sided test at α = 0.05 and 80 % power (normal approximation for two independent proportions):

| Death rate, control → variant | Runs per profile | Core-hours, both profiles, 110 s/run | Ideal wall time on 20 cores |
| --- | --- | --- | --- |
| 50 % → 40 % | 388 | 23.7 | 71 min |
| 30 % → 20 % | 294 | 18.0 | 54 min |
| 30 % → 25 % | 1,251 | 76.5 | 3.8 h |

"Ideal" excludes overhead and contention; on four dedicated cores the 30 % → 20 % comparison is about 4.5 hours. Single-run times do not establish parallel throughput, so Phase 0 measures it. Paired seeds with McNemar's test can reduce the runs needed; how much depends on the number of discordant pairs, which a pilot measures. The three-seed cohorts used so far cannot detect any realistic difference. Costs at longer caps are measured, not extrapolated.

## Test harness

1. **Identity test.** `baseline-Bn` reproduces the Bn trace byte for byte on the identity seeds and the development seeds, never on the sealed test set. Runs on every change to the parameter layer.
2. **Decision fixtures.** Hand-built boards that call one decision function and check its output, as the existing native scenarios do. Every module variant comes with fixtures showing where it decides differently.
3. **Situation suite — diagnostic until qualified.** It needs a small, qualified inventory of **loadable** positions: the dossiers' T-minus snapshots are textual observations from replays, not necessarily loadable saves at each point, while the existing recent living saves are loadable. A fresh Borg started from a save tests recovery from incomplete memory, not the original Borg's situation. Use it for diagnosis; do not base acceptance on it until R4 provides paired checkpoints.
4. **Population runs.** Full games from birth on four frozen seed sets: *identity/regression* seeds for mechanical checks; *training* for search; *validation* for choosing (repeated validation choices count as part of the search); and a **sealed** *test* set. Test seeds run only after a candidate and its evaluation protocol are frozen, candidate and matching baseline together. No earlier identity check, diagnostic or cohort exposes their per-seed traces or outcomes to policy development. Extend to more races and classes once Warrior results stabilise.

**Runner changes needed:** one lock-owning batch coordinator with a bounded worker count (workers do not reacquire the shared lock, which would serialise or deadlock the coordinator), isolated run directories, parameter injection through `borg.txt` (no rebuild per profile), the shared outcome schema, an append-only results store, and a report with survival curves and paired comparisons. Generated runs stay out of Git.

## Search strategy

- **Screening first,** on short caps, to find the parameters that move the outcome. Group parameters by when they can act (activation-aware): short caps cannot exercise late-game parameters such as the level-35 mana reserve, so a parameter whose code never ran is untested, not dropped. Keep interactions in the screening design, and add occasional longer evaluations.
- **Racing for tuning** (irace-style racing or successive halving): many candidates on common seed prefixes, drop the clearly worse, promote the rest to more seeds and longer caps. Rankings can reverse as caps grow, so keep occasional longer evaluations. CMA-ES or Bayesian optimisation only within genuinely continuous subspaces, not across integer thresholds and mode switches.
- **Confirm on held-out seeds** against `baseline-Bn` with the prespecified objective.
- **Death-driven module work:** rank causes of death, pick the dominant one, design a variant, and prove it on fixtures, then on the population (the situation suite is a diagnostic aid).

## Plan

**Order:** R0 and R1 come first. Phase 0 can be prepared meanwhile, but its baseline cohort runs on B1 only after R1, so that both repositories measure the same baseline.

- **R0 · first.** **A C oracle for borg-rs.** Freeze the B1 manifest (ordered patch IDs and applied commits at a named better-borg-c commit; 34 carries at `702f091`). Build a scratch C oracle: the pinned engine with the B1 carries plus the `borgh` harness, wired completely (build registration, input hooks, seed and calendar handling, summaries, build identity). Adopting `borgh` permanently in better-borg-c is separate and later. Exit: the oracle reproduces borg-rs's existing pristine references when built without carries, and records its identity with carries.
- **R1 · first.** **Carry B1 into borg-rs.** Port the manifest in order, one commit per fix, each linking its better-borg-c adoption issue. For every patch, write a short **applicability and translation audit**. Three patches fix C memory handling that Rust already owns structurally (0026 calculation reload array, 0033 operator-block arrays, 0034 calculation-token arrays): record an applicability decision instead of an artificial change. Check once, at the end: the 80 suite runs against the B1 oracle. A divergence is investigated at its first divergent key; if bisection is needed, compare each Rust prefix with the **matching C prefix**, never with the final carry set. Optionally collect lightweight counters on the changed branches in that same batch. No per-fix native scenarios. Exit: "identical on these 80 runs", plus the audit, plus the list of fixes the runs did not exercise. Expected outside suite coverage on present evidence (review #360): 0001, 0003, 0005, 0007, 0010, 0019, 0023, 0025, 0028, 0029, 0032; the "possible/unconfirmed" rows also rest on the audit until a distinguishing execution is shown.
- **Phase 0.** **Decide and prepare.** Owner decisions (below). Qualify the observation and scoring rule (above) in the population runner. Freeze the objective, progress floor, survival tolerance and four seed sets. Build the coordinator and results store; measure real parallel throughput and the discordant-pair rate in a pilot. Baseline cohort of about 300 seeds at 400k turns on B1 (about 9 core-hours for one profile). Exit: B1 survival curve, death-cause table, measured throughput.
- **Phase 1.** **Existing knobs.** Sweep the formula section and the `borg.txt` switches (approach A) on the training seeds, remembering which gates formulas cannot reach. Exit: a first measured profile, or a documented null result.
- **Phase 2a.** **Pilot both implementations.** Implement the same five parameters in C and in Rust, with identity at defaults and differential identity at two non-default profiles. Record engineering time and defects found. Exit: an owner decision on where the full parameter layer and module work live. After the pilot, duplicated policy implementation stops unless the defects it caught justify the effort.
- **Phase 2.** **Parameter layer.** In the chosen repository (or both, if Phase 2a says so), the first justified parameters with Bn defaults. Exit: `baseline-Bn` identity on the identity and development seeds; parameters dumped in every run record.
- **Phase 3.** **Screen and race.** Activation-aware screening on short caps (size and duration from the Phase 0 pilot), then racing at 400k turns, then validation. Exit: profiles that meet the prespecified objective against `baseline-Bn` with stated confidence.
- **Phase 4.** **Module variants.** Where the death analysis points, each behind a switch with its own fixtures and population evidence. Exit: each kept variant has that evidence.
- **Phase 5.** **Generalise and go deeper.** More races and classes, longer caps (measured costs), re-tuning of combined profiles. Open the test seed set once. Exit: a milestone review package in the style of better-borg-c's existing ones.

### borg-rs track (after R1)

- **R1b.** **Next baselines, not continuous catch-up.** The carry set is frozen for a measurement campaign. Later carries queue for the next baseline (B2, …), which is ported and checked the same way as R1 before the next campaign starts. A campaign is never invalidated by a carry landing mid-way.
- **R2.** **Shared run contract and deeper identity.** Adopt the shared seed sets, calendar policy, outcome schema and the exact-boundary observation (a prerequisite for Phase 0). Extend identity checks beyond dungeon level 27 with long caps. Exit: identity on the identity and development seeds, never on the sealed test set.
- **R3.** **Same parameter table** (if Phase 2a keeps both). Exit: differential identity at default and non-default profiles.
- **R4.** **Serialisable Borg memory, for new checkpoints.** Save and restore `BorgState` together with a game save, accounting for native save compaction, engine-side references, both RNG states and pending interaction state. Historical C dossiers cannot be retrofitted; this makes **new** paired game-and-Borg checkpoints. Exit: a resumed run's keys equal the uninterrupted run's.
- **R5.** **Module variants behind traits,** if Phase 2a places module work in Rust. Experimental variants do not need immediate C duplication; only profiles both repositories claim to implement must match.
- **Decide.** At Phase 2a, and again after the first module variants: which repository carries new policy work, and whether the C/Rust cross-check still pays for its maintenance, judged by engineering time, defects caught and upstream-tracking cost.

## Risks

- **Gaming the metric:** survival bought by stalling, or progress bought by reckless dives. Guard with the prespecified objective, the progress floor and the operational outcomes.
- **Overfitting:** to Warrior, the training seeds or the turn cap. Guard with held-out seeds, more classes and occasional longer caps.
- **Moving baselines:** carries landing during a campaign. Guard with frozen manifests and B1/B2 versioning.
- **Upstream drift:** keep the parameter layer mechanical, carry it as its own removal unit, re-run the identity test after each upstream import.
- **Mixing provenance:** strategy changes never enter bug carries or upstream proposals. Separate labels, patches and issues.
- **Compute contention:** one lock-owning coordinator, bounded workers, scheduled batches or a separate runner machine. borg-rs reports a 4-core machine, which suits identity suites but not tuning cohorts.
- **Two codebases drifting apart:** guard with differential identity at shared profiles, ledgers of what exists in one only, and the Phase 2a decision.

## What a remake leaves out

*Added 2026-11-08, after revision 3.* The plan above recommends against a rewrite (approach D). The owner is also considering a remake: a new Borg designed from scratch, keeping the C Borg's concepts (danger above all) but not its code. This section records what such a remake would not carry over, and the measurements behind each item. It does not change the plan for better-borg-c or borg-rs.

The measurements come from [borg-zoo](https://github.com/diegoami/borg-zoo), which runs seed suites across Borg versions and `borg.txt` profiles ([results](https://diegoami.github.io/borg-zoo/)). Unless stated otherwise:
- the original C Borg (`e473720`) on 50 Human Warrior seeds;
- the first 100,000 engine turns (experiments `switches-early` and `switches-early-2`);
- "no effect" means no effect in the early game, not proof that a setting is useless later.

| Item | Status | Why | Evidence |
| --- | --- | --- | --- |
| **Munchkin start** (`borg_munchkin_start`, §13.1 of the specification) | **Left out: owner decision, 2026-11-08** | It survives by stair-diving to a target depth without fighting, a scumming style of play. Its effect depends on the class, and it has a liveness defect. | `classes-munchkin`: 1,800 seeds, 9 classes × 200, races cycled. Deaths fall for warriors (32 % → 7 %, 60 seeds rescued against 10 newly killed), druids and rangers, but **rise** for mages (31 % → 45 %, 26 against 54) and priests (24 % → 37 %, 29 against 55). 152 of 200 Blackguard runs never finish: they loop on "Fleeing level. Munchkin Mode" without game time passing ([#403](https://github.com/diegoami/neo-angband-tracker/issues/403#issuecomment-6717932947)). These per-class figures are post hoc. |
| **Money scum** (`borg_money_scum_amount`) | Proposed | It rests in town killing townspeople for gold until a target is reached: survival by not playing. | 0 deaths in 50 runs at 5,000 and at 20,000 gold, because it hardly plays: 49 of 50 characters are still at character level 1 at the cap, never below dungeon level 1. All 16 of the default's deaths are avoided, and progress collapses with them (alive at the cap having reached dungeon level 5: 18 % → 2 %). |
| **Self scum** (`borg_self_scum`, default **on**) | Proposed | It sets a money-scum target when the Borg wants a shop item it cannot afford, and keeps playing once the target is reached. It is gold farming by another route. | Turning it off gave the same outcome on all 50 seeds; its conditions (character level 10+) are rarely met that early. |
| **Lunal mode** (`borg_self_lunal`, §13.1 of the specification) | **Left out: owner decision, 2026-11-08** | Stair-dives and climbs without fighting, as munchkin does, but deeper: the same scumming style. | Not measured; left out on the same grounds as munchkin. |
| **The formula language** (`borg_uses_dynamic_calcs` and the formula section of `borg.txt`) | Proposed | A second, interpreted valuation beside the code. A remake has one valuation model in code. | With it on, the Borg's start-up warning consumes a runner key and the Borg aborts, so the zoo cannot run it. The shipped `borg.txt`, formula section included, played identically to the built-in defaults on all 50 seeds. |
| **The `borg_worships_*` switches** (speed, HP, damage, mana, AC, gold) | Proposed: replace | Per-stat selectors of different weight branches in `borg_power()`; `worships_gold` changes selling instead. A remake would have one valuation with explicit weights. | Speed, HP and damage gave the same deaths as default on all 50 seeds (damage lost 2 runs on the progress measure); mana played identically; AC moved 9 seeds (5 rescued, 4 newly killed); gold changed 7 of 50 games without changing the death count. |
| **Chest tolerance** (`borg_chest_fail_tolerance`) | To evaluate | A single threshold in the flow. | Identical to default on all 50 seeds. |
| **Cheat death and respawn** (`borg_cheat_death`, `borg_respawn_*`) | Proposed: move | Test conveniences. They belong in the test runner, not in the Borg. | – |

**Kept as concepts, redesigned:**
- **The danger model.**
- **Leaving a level that feels dangerous.** Human players use it too, but the remake needs a rule that stops it from cycling between levels without game time passing. Munchkin's Blackguard loop and seed 1045 ([#403](https://github.com/diegoami/neo-angband-tracker/issues/403)) are both in that family.
- **Swaps, `borg_kills_uniques` and `borg_plays_risky`.** No clear early effect, but they are risk and equipment policies that may matter deeper in the game.

**Before striking more:** the items marked "Proposed" or "To evaluate" on early-game evidence alone should first be run on borg-zoo's deep suite (400,000 engine turns) across classes. Money scum and the formula language do not need it. Munchkin start and lunal mode are already decided.

### Strategy by class (owner direction, 2026-11-08)

A remake has different strategies per class, and possibly different Borgs. A shared base (perception, map memory, flow, item knowledge and the danger model) carries a strategy layer per class: fight or flee thresholds, dive pacing, resting, spell use, mana reserve and equipment valuation.

borg-zoo's class suite (`classes-1800`: 9 classes × 200 seeds, races cycled, 100,000 engine turns) shows why:

- **Class matters more than version.** On the original C Borg, deaths range from 19 % (Paladin) to 70 % (Druid). Every better-borg-c milestone so far moves the overall rate by about a point.
- **The same setting splits by class.** Munchkin-d6, against default on the same seeds:

  | Class | Default | Munchkin-d6 | Rescued / newly killed |
  | --- | --- | --- | --- |
  | Warrior | 32 % | 7 % | 60 / 10 |
  | Druid | 70 % | 49 % | 58 / 16 |
  | Ranger | 43 % | 29 % | 54 / 26 |
  | Paladin | 19 % | 18 % | 25 / 24 |
  | Mage | 31 % | 45 % | 26 / 54 |
  | Priest | 24 % | 37 % | 29 / 55 |
  | Blackguard | 20 % | — | 152 of 200 runs never finish |

  These splits are post hoc, from one run of 1,800 seeds.
- **Fixes split by class too.** Milestone 5 cut Warrior deaths from 32 % to 17 %, and milestone 6 brought them back to 28 %. A prespecified confirmation on fresh Warrior seeds and a bisect of the M5 to M6 carries are running.

**Measured before it is built.** Per-class strategies are compared per class. 200 seeds per class detect differences of about 12 points; about 1,000 seeds per class are needed for 5 points.

**Diving without scumming.** The early gain munchkin start buys for some classes comes from getting deeper faster. The remake should get it through fair play: a depth-preparation rule per class. In the C Borg that rule is `borg_prepared()`, including "maximum character level < depth", the plan's biggest single lever (see the first tuning surface). Stair-scumming to a target depth stays out. No existing setting does this: `borg_plays_risky` made no early difference on 50 Warrior seeds.

## Decisions for the owner

1. **better-borg-c scope:** a strategy track conflicts with RULES 12 (no new gameplay strategies), RULES 10 (no Rust work), RULES 14 and PLAN (bugs before discretionary batches), and the cross-repository limits in AGENTS and PLAN. Decide whether tuning gets protected capacity alongside the bug queue.
2. **borg-rs scope:** R0/R1 change its rule of keeping C bugs on purpose; R3–R5 change "faithful port, no new tactics".
3. **neo-angband-tracker:** its RULES 4, 13 and 15 exclude managed Rust adoption and parked strategy work here. This plan lives here as a document only; say whether that stands.
4. **Owning sessions:** name the session that owns each track (R0–R1, Phase 0–5, R2–R5) and the shared contract.
5. **The objective:** D, T, the progress floor and the survival tolerance.
6. **Compute:** the budget, and whether a dedicated runner machine is available.
7. **Classes:** whether tuned profiles may differ by class. For a remake the owner has answered yes (*Strategy by class*, above); for the existing Borgs it is still open.

Sources: `BORG_SPEC.md` at `684321538` (merged with PR #317); borg-rs at `74fc879` (`README.md`, `CLAUDE.md`, `docs/REVIEW.md`, `docs/STATUS.md`, `docs/harness.md`, `harness/suites/`, `borg-rs/src/state.rs`; local checkout, read only); better-borg-c at `702f091` (`patches/series.json`) and `8a7eba1ad` (`README.md`, `PLAN.md`, `RULES.md`, `docs/measurement/`); `src/borg/borg.txt` and the literal count from `src/borg/*.c` at `e473720`; reviews [#360](https://github.com/diegoami/neo-angband-tracker/issues/360) and [PR #361](https://github.com/diegoami/neo-angband-tracker/pull/361); `borgh_key()` in borg-rs `angband/src/main-borgh.c` at `74fc879`.
