# Sources

*Step 1 of the initialisation: what exists, what it teaches, what we take,
what we leave. Read on 2026-11-08 at the commits in [`pins.json`](../pins.json).*

Live state (open bugs, carries, experiments in flight) belongs to the issue
boards of the repositories below, not to this file. This file says what each
source is **for** in this project.

## Summary

| # | Source | What it is | What we take | What we leave |
| --- | --- | --- | --- | --- |
| S1 | Original C Borg, `angband/angband` `src/borg/` at `e473720` | 59 `.c` files, 64,147 lines: the original autoplayer, revived in 4.2.5 | Game knowledge: danger formulas per monster blow and spell, item values, the prompt answers, the option set, the store routines, the flow algorithms | The decision ladder, the global state, the scumming modes, the formula language, the cheat-death path |
| S2 | [BORG_SPEC.md](../reference/c-borg/BORG_SPEC.md) (neo-angband-tracker) | The specification of S1, cited `file:function:lines` | The map of where every decision lives; §16 (what a rebuild must preserve) as the list of contracts we keep *on purpose or not*; §11 and §17 (defect shapes a rebuild must not inherit) as design constraints | Its rule that §16 wins: we are not the C Borg and do not claim parity |
| S3 | [TUNABLE-BORG-PLAN.md](../reference/c-borg/TUNABLE-BORG-PLAN.md) (neo-angband-tracker) | Evaluation of how to improve the Borg by experiment; its last section, *What a remake leaves out*, is written for this repository | The objective and its statistics, the seed-set discipline, the first tuning surface (which levers matter), the per-class direction, the list of what the remake drops | Its main recommendation (do not rewrite): the owner has chosen a remake, and the plan still holds for better-borg-c and borg-rs |
| S4 | [better-borg-c](https://github.com/diegoami/better-borg-c) | The C Borg with verified fixes (76 carries at milestone 7, tag `4.2.6-impact-v7`), the native measurement runner, ten death dossiers | The death-dossier format and process; the death evidence; the list of real defects (each one is a test the reborn Borg must pass); the measurement runner as the zoo's engine | Its scope rule "no new gameplay strategies": that is exactly what we do here |
| S5 | [borg-rs](https://github.com/diegoami/borg-rs) | The faithful Rust port: 59 modules, ~73,000 lines, key-stream identical to C on 80 suite runs; M5 accepted (tag `m5`) | Ideas and one-time copies, recorded in docs/PROVENANCE.md and ours from then on: the harness hooks and the headless frontend, the `staticlib` behind `inkey_hack`, the M1 engine boundary (`engine.rs`, since re-based on bindgen), the death-dossier format. Nothing of it is used at build, test or run time | **A reference to copy from, never a dependency** (docs/SPEC.md §2, owner 2026-11-09). Its harness hooks and headless frontend were copied once and are ours (docs/PROVENANCE.md); its engine tree, shim, tools and references were removed in milestone E |
| S6 | [borg-zoo](https://github.com/diegoami/borg-zoo) | Seed suites × specimens × profiles, run on GitHub Actions, append-only results, [site](https://diegoami.github.io/borg-zoo/) | The yardstick: suites, outcome vocabulary (death, turn-cap survival, stall, timeout, crash, harness failure, config error), paired comparison; borg-rs specimens are already run through `harness/build.sh` and `borgh`, so the reborn Borg can be measured with the same adapter | Nothing: the zoo measures, it does not change Borgs (its RULES 1) |
| S7 | [borg-agent](https://github.com/diegoami/borg-agent) | A language-model player behind the Borg's connection, producing *claims with evidence*; started 2026-11-08 | Its claims, once reviewed, as candidate rules and parameters; its milestone/review workflow (we use the same); its pinning pattern (`pins.json`, `scripts/setup.sh`) | A model in the play loop: the reborn Borg is deterministic and runs without a model |
| S8 | Issue boards: [neo-angband-tracker](https://github.com/diegoami/neo-angband-tracker/issues) (291 issues, 263 `upstream-c`), [better-borg-c](https://github.com/diegoami/better-borg-c/issues), [borg-rs](https://github.com/diegoami/borg-rs/issues) | C Borg findings, adoption issues, death investigations, proposals | The defect shapes; the owner's stated preferences; the proposals that are strategy changes (now in scope here) | Their routing rules: findings about the C Borg still go to neo-angband-tracker, not here |
| S9 | Death dossiers: [better-borg-c](../reference/deaths/better-borg-c/), [borg-rs](../reference/deaths/borg-rs/) | 16 cases (10 + 6), with replay-verified observations before death | What actually kills Borgs (below) | — |
| S11 | [REDESIGN-LESSONS.md](../reference/deaths/better-borg-c/REDESIGN-LESSONS.md) (better-borg-c, 2026-11-09) | Requirements for this repository distilled from deaths D0001–D0014 and four owner manual takeovers, each lesson with its evidence level | All thirteen lessons; traced to our design in [LESSONS-TRACE.md](LESSONS-TRACE.md) | — |
| S12 | The Angband Strategy Manual, draft (neo-angband-tracker `docs/strategy/`) | **Retired 2026-11-09** (owner): superseded by S14, its complete version. The pin, its copy and `reference/strategy/` were removed; it is in git history |
| S14 | [The Angband Strategy Manual and the official user manual, complete](../reference/c-manual/README.md) (neo-angband-tracker, default branch, [`docs/c-manual/`](https://github.com/diegoami/neo-angband-tracker/blob/master/docs/c-manual/README.md)) | The finished version of S12's draft, for upstream Angband at `e473720` (our engine pin), every number cited to `lib/gamedata/` or `src/`. Seventeen chapters (core mechanics, character creation, magic and spells, combat, elements and status, equipment, consumables, monsters, dungeon, exploration, town, game plan by phase, escapes and emergencies, class playbooks, **notes for the borg**, things to try), appendices A1–A10 generated from the data files, and a copy of the official user manual and help files at the same commit. Added 2026-11-10 (owner) | **The game knowledge for M2–M6**: the official user manual for commands, screens and options; the strategy manual as the first reference for: L3 knowledge tables (A1–A10); the threat model (ch. 1, 4, 5, 8, A3: speed and energy, blows, breath from current HP); escapes and reserves (ch. 7, 13: escape budgets, what works blind or confused); readiness by depth and class (ch. 5 resistance checklist, 12, 14); exploration and leaving a level (ch. 10); town and stock (ch. 11). Chapter 15 gives decision rules and thresholds for an automatic player, as candidate parameters and defaults; chapter 16, as experiments. Check every claim against `engine/angband/src/` and cite the engine (RULES 16) | The same as S12: advice relying on information the Borg does not read (GOALS D3); any rule or threshold the zoo has not measured is a hypothesis (RULES 14). It supersedes S12, retired 2026-11-09 |
| S15 | [borg-runner](https://github.com/diegoami/borg-runner) ([site](https://diegoami.github.io/borg-runner/)) | Language-model agents (Claude, MiniMax, Codex, Qwen, GPT, and one owner run) play upstream 4.2.6 at `e473720`, our engine, **from the screen, one key at a time**. Every key is recorded, each run has a replay and a write-up, and what the players learned accumulates in one page they all read, [`manuals/INSTRUCTIONS.md`](../reference/borg-runner/manuals/INSTRUCTIONS.md). Each line found by a run is traced to its `run#ask` and labelled **FACT** (game data or source), **POLICY** (a strategy-manual rule) or **UNTESTED**. Volume 2 covers depth 5–20 and is untested. Fourteen runs at the pin, four deaths, deepest depth 7. Added 2026-11-10 (owner) | **Knowledge of agent players**, accumulated:<br>- what a player that reads the manual finds out at the screen: costs, prompts, early dangers, what killed it;<br>- the gaps between the manuals and the game;<br>- [`ACCUMULATED_KNOWLEDGE.md`](../reference/borg-runner/ACCUMULATED_KNOWLEDGE.md): the export **written for this repository** (its own header names borg-reborn-rs as the consumer), rewritten from time to time — only what no manual or data file says: measurements, corrections (including of itself: edition 3 corrected edition 2's "one down staircase per level" to three or four), numbers and the open questions;<br>- each run's notebook (`knowledge/`) and write-up (`records/`), with the screens and keys at the pin.<br>Candidate rules and parameters for the strategies and their profiles (M3–M6), and early-game situations for tests. Copied in `reference/borg-runner/` | **A model in the play loop**: the Borg plays without one (GOALS D11). **Claims as facts:** a FACT line is checked against `engine/angband/src/` before use and cited to the engine (RULES 16), and an OBSERVED line rests on a few runs, some of which inherited each other's notebooks. A rule that helps the reborn Borg is measured by the zoo before any claim (RULES 14). Screen-only readings (the level feeling) don't carry over to the Borg's direct reads (GOALS D3) |
| S13 | [Angband command descriptions](https://angband.readthedocs.io/en/latest/command.html) (angband.readthedocs.io, built from `angband/angband` `docs/command.rst`) | The official list of every command with its original and roguelike keys, prompts, repeat counts and energy use. On 2026-11-09 upstream `master` is at our engine pin `e473720`, so the page is exactly [`engine/angband/docs/command.rst`](../engine/angband/docs/command.rst) in our engine tree | **The key vocabulary of the Borg's actions:** what each key does, which prompts follow, which commands take energy. The Borg plays the original keyset (the game's default, RULES 9); key sequences are checked against the engine and cited there | The roguelike keyset; anything on a later `latest` page that differs from `e473720` (the page tracks upstream `master`; the `angband` pin's refresh report shows when master moves) |
| S10 | Prior art outside Angband (below) | Bots for NetHack, DCSS and Rogue | Architecture lessons | — |

A digest of the deaths and of the issue boards by subsystem is in
[research/DEATHS-AND-ISSUES.md](research/DEATHS-AND-ISSUES.md).

## S1–S2: what the C Borg is, in one page

From the specification (§ numbers refer to it):

- **A decision loop driven by the engine's key requests** (§2.1–2.2). The
  engine calls `inkey_hack`; the Borg perceives, thinks only when its key
  queue is empty, queues a complete key sequence, and returns keys one at a
  time. Rest and repeat are passed through as `EVT_NONE` (§2.7). A think that
  queues nothing stops the Borg. **We keep this mechanism unchanged**: it is
  the seam every Borg in the family uses, and the harness depends on it.
- **Perception reads game state directly** (§1.1), well beyond the screen:
  its own gear and status, `map_info()` for the visible panel, the real race
  of each newly seen monster, the real hit points and timers of tracked
  monsters, full monster race data. It does not see unseen monsters or the
  map outside what it observed. **Information level is an owner decision**
  ([GOALS.md](GOALS.md), D3); the recommendation is "the same as the C Borg,
  no more", so comparisons in the zoo are fair.
- **The intelligence is a fixed priority ladder** of about 60 rungs
  (`borg_think_dungeon()`, §2.8), each with its own guards. Caution comes
  before combat, combat before pickup and exploration, and when nothing
  applies three "bravery boosts" make danger look smaller until something
  does. **This is the part we redesign.**
- **Danger** (§8) is a scalar 0–2000 per grid over `c` turns: the sum over
  tracked monsters of physical and spell damage, scaled by speed, distance
  and wakefulness, plus regional and per-grid "fear". `borg.avoidance`
  (normally the current HP) turns danger into impassable grids for the flow.
  **We keep danger as the central concept** (owner direction) and redesign
  its form.
- **Power** (§5) is the utility of a kit; **preparation** (`borg_prepared()`,
  §5.5) gates every dive. The TUNABLE plan calls the preparation rule the
  biggest single lever.
- **State is global and patched between thinks**; simulations mutate the
  live item array and are meant to be undone (§5.4, §16.2.6). §11 lists
  the defect shapes this produces: stale state between calls, the wrong
  variable, mutation during a read-only pass, a mode flag set by one
  routine and cleared by another, shared running bests, double counting.
- **Anti-bounce counter and Borg clock** (§2.4–2.5): progress is measured
  in thinks, not game turns, and thresholds forget goals, forget monsters,
  or force leaving. Loops in which no game time passes are possible (#403).

## S3: the measurement doctrine we inherit

From the Tunable Borg Plan, adopted here without change:

- **Primary objective, prespecified:** the share of runs alive at engine
  turn T having reached dungeon level D. It rewards neither the Borg that
  never dives nor the one that dives and dies.
- **Every run counts:** stalls, timeouts, crashes and harness failures are
  non-successes, reported per profile, never relabelled as deaths.
- **Seed sets:** identity/regression, training, validation, and a sealed
  test set opened once.
- **Sample sizes:** 50 % → 40 % death rate needs ~390 runs per arm; 30 % →
  25 % needs ~1,250. Three-seed cohorts cannot detect any realistic
  difference. borg-zoo's 200 seeds per class detect about 12 points.
- **Class matters more than version:** on the original C Borg, deaths in
  the first 100,000 turns range from 19 % (Paladin) to 70 % (Druid); every
  better-borg-c milestone moved the overall rate by about a point. The same
  setting helps one class and hurts another.

And from its section *What a remake leaves out* (owner decisions of
2026-11-08 marked as such):

| Left out | Status |
| --- | --- |
| Munchkin start, lunal mode | **Owner decision** |
| Money scum, self scum | Proposed |
| The formula language | Proposed |
| `borg_worships_*` switches | Proposed: one valuation with explicit weights instead |
| Cheat death, respawn | Proposed: move to the test runner |
| Danger model; leaving a dangerous level (with a no-cycling rule); swaps, killing uniques, playing risky | **Kept as concepts, redesigned** |
| Strategy by class | **Owner direction**: a shared base and a strategy layer per class, possibly different Borgs |
| Diving without scumming | **Owner direction**: per-class depth preparation, no stair-scumming |

## S4–S5, S9: what kills Borgs

Twenty dossiers, fourteen from better-borg-c and six from borg-rs (D0011–D0014
added by the source refreshes of 2026-11-09; `reference/deaths/INDEX.md`
lists them all, ours included). better-borg-c's
[REDESIGN-LESSONS.md](../reference/deaths/better-borg-c/REDESIGN-LESSONS.md)
distils D0001–D0014 into requirements for this repository (S11). Not all are
independent: better-borg-c D0003–D0005 are verified replays of D0002, and
borg-rs D0001–D0004 are a C/Rust parity cohort. Read them for their *shape*,
not their statistics: they are not a sample.

| Pattern | Cases | What it says about the design |
| --- | --- | --- |
| **Resources unused at death.** Cure Light Wounds carried and not drunk; Speed potions unused; Phase Door left at home; eight known CLW excluded by the CL10 healing tier; 21 known CLW and 5 Speed carried into death | bbc D0001, D0002, D0011, D0014; rs D0005 | Healing and escape are separate rungs with their own guards (`!goal.fleeing`, danger-versus-heal tests); no step compares *all* options against the threat. |
| **Oscillation.** Retreat to the stair, step back into the dark next to a mold, repeat until dead (a 2-HP mold the Borg never attacks) | bbc D0008, D0012; rs D0006 | First-match rules with no commitment and no memory of what was just tried; a cached danger reused. |
| **Item use against readiness.** Deep Descent read at depth 4, arriving at depth 14 at 37 HP; three reads taking a CL10 Warrior to depth 18, and a CL9 Rogue to depth 20 | bbc D0006, D0011, D0014 | Item use is not checked against the dive policy. |
| **Liveness.** Starvation while repeatedly failing to pick up into a full quiver; 63k turns wanting town with 563 "no up stairs, going down" fallbacks | bbc D0009, D0011 | No budget on a goal that does not progress; food is not a hard invariant. |
| **The wrong alternative.** A Ranger with a bow flees by stairs and dies instead of shooting; the only Phase Door read where one step back would do; retreat chosen over killing a 2-HP mold | bbc D0010, D0012, D0013 | Fight-or-flee is not an evaluation of alternatives. |
| **Escapes spent until none are left** (the last Phase Door read at full HP) | bbc D0002, D0007, D0013, D0014 | No reserve policy; no account of what the escape buys. |
| **Fear by list, not by risk.** One small kobold, or two white worm masses, set "scary guy on level" and whole-level flight | bbc D0012, D0013 | Hard-coded scary lists stand in for the danger model; leaving the level is not weighed against what the threat can actually do. |
| **Unknown items left unknown.** Nine unidentified Cure Serious Wounds carried for ~47k turns; eight unidentified CLW at CL2; rings bought instead of cures | bbc D0011, D0012, D0013 | Identification and stock are not part of the survival policy (owner direction, better-borg-c#180, #41). |
| **Wrong kit valuation.** A Dagger wielded over a better unidentified Maul on arrival at depth 13 | bbc D0011 | Power has depth-tied bonuses and per-blow weights instead of expected damage per turn of the actual kit. |

The better-borg-c investigations also record two owner priorities:
survival stock over spare gear
([better-borg-c#41](https://github.com/diegoami/better-borg-c/issues/41))
and premature exposure to crowds
([better-borg-c#34](https://github.com/diegoami/better-borg-c/issues/34)).
The quick-gain proposals on neo-angband-tracker,
[#370](https://github.com/diegoami/neo-angband-tracker/issues/370) (Phase
Door and a cure before the first dives) and
[#371](https://github.com/diegoami/neo-angband-tracker/issues/371) (a tunable
clevel-versus-depth gate), are strategy changes the C repositories cannot
make and this one can.

*Historical (M1): since milestone E nothing below is a build input; see docs/SPEC.md §2 and §8.*

## S5: what borg-rs gives us concretely

| Piece | Where in borg-rs | Use here |
| --- | --- | --- |
| Engine tree with harness hooks | `angband/` (subtree of upstream `e473720`, hooks under `#ifdef BORG_HARNESS`, `BORG_IMPL` in CMake) | Imported in M1 with history, unchanged. `BORG_IMPL=rust` builds the reborn crate, as it builds borg-rs's. At the pin the tree's C Borg carries borg-rs's adopted fixes (impact v6 at `e5cb263`: 26 files under `src/borg/` differ from upstream), so the controls are separate arms (`c-pristine`, `c-bbc-m7`, T1.3, #6) |
| Headless frontend `borgh` | `angband/src/main-borgh.c` | Seeds, scripted birth, fixed date, 80×24, `keys.log`, `summary.json`: the zoo's interface |
| Harness scripts | `harness/build.sh`, `run.sh`, `suite.sh`, `diff`, `suites/` | Same names and semantics; suites for the reborn Borg |
| Engine boundary | `borg-rs/src/engine.rs` (~3,300 lines, the only `unsafe`) | Reused as the substrate's foundation |
| Launchers and replay | `run-rust-borg.sh`, `start-borg.sh`, `run-borg-headless.py`, `replay-borg.py`, `produce_death.py`, `replay_next_death.py`, `tools/` | Same commands and options, pointed at the reborn Borg (M1–M3) |
| Death records | `docs/deaths/`, `tools/record-death.py`, `aggregate-deaths.py` | Same format from the first natural death |
| `BorgState` | `borg-rs/src/state.rs` | A reference for what the C Borg remembers; not our state model |

## S10: prior art outside Angband

| Bot | Game | What it teaches |
| --- | --- | --- |
| [AutoAscend](https://arxiv.org/abs/2305.19240) | NetHack | Winner of the NeurIPS 2021 NetHack Challenge; symbolic agents took the top three places over neural ones. A parsed internal state plus a hierarchy of strategies chosen by a controller with predicates, close to a behaviour tree. Strong class dependence (Valkyrie). The follow-up ([Piterbarg et al., NeurIPS 2023](https://arxiv.org/abs/2305.19240)) logged *which strategy produced each action* and trained on it: a decision trace is valuable later. |
| [BotHack](https://github.com/krajj7/BotHack) | NetHack | First bot to ascend (2015). Framework and bot separated; A\* navigation; logic programming for item identification. Its win relied on an exploit later patched: a reminder of why we exclude scumming. |
| TAEB, Saiph | NetHack | Frameworks that separate perception, knowledge and personalities (strategies). The *framework + many personalities* shape matches *substrate + strategies per class*. |
| [qw](https://github.com/crawl/qw) | DCSS | The only bot to win DCSS unassisted, including 15-rune wins; rule-based, now maintained by the DCSS team. Explicit goals per game stage, configurable per character type. |
| Rog-O-Matic (Mauldin et al., 1984) | Rogue | The first expert-system roguelike player: separate knowledge base, layered behaviours. The design space is old; what is new here is measurement at scale. |
| [NetPlay](https://arxiv.org/abs/2403.00690) | NetHack | Language models as zero-shot players: good at describing, poor at long play. Supports keeping models out of the play loop (borg-agent's knowledge role is the right use). |

Two conclusions we draw from it:

1. **Symbolic, deterministic, hierarchical** is the state of the art for
   roguelike bots that actually win. Learning helps later, from logged
   decisions, for narrow choices.
2. **Exploits are a trap.** BotHack's ascension and the C Borg's munchkin
   and scum modes buy results the game's designers did not intend.
   The owner has excluded them.

## Sources added to this repository

Copied under [`reference/`](../reference/) so that every session can read
them without network access. Each is a copy at a pinned commit; the
original repository stays canonical.

Commits are in `pins.json` and `reference/README.md`, which
`scripts/refresh-sources.py --update` keeps current.

| Path | From | Selection |
| --- | --- | --- |
| `reference/c-borg/` | neo-angband-tracker `docs/c-borg/` | spec, plan, index |
| `reference/deaths/better-borg-c/` | better-borg-c `docs/deaths/` | every top-level file, and every `.md`/`.json` file at the root of each case (reports, assessments, overall assessments, owner feedback, verification records); raw logs, snapshots and saves left out |
| `reference/deaths/borg-rs/` | borg-rs `docs/deaths/` | same selection |
| `reference/zoo/` | borg-zoo | `docs/DESIGN.md` and `catalogue/` |
| `reference/borg-rs/` | borg-rs `docs/` | `FORK-CORRECTIONS.md`, `adoption/status.json`: what the imported engine tree carries |
| `reference/c-manual/` | neo-angband-tracker `docs/c-manual/` on the default branch (pin `neo-angband-tracker`) | the whole directory: 17 chapters, appendices A1–A10, the generator, and `user-manual/` (the official manual and help files at `e473720`) (S14) |
| `reference/borg-runner/` | borg-runner (S15) | `manuals/` (the knowledge page, volume 2 and the jev manual), `knowledge/` (each run's notebook), every `.md` file of `records/` (each run's write-up and notebook, and the overview), and `ACCUMULATED_KNOWLEDGE.md` at the root (the export written for this repository, rewritten from time to time); screens, keys, decisions and replay pages are left out (read them at the pin) |
| `reference/deaths/INDEX.md` | generated | every death of every registry, ours included |

The official game manual (`docs/*.rst`, published at
[angband.readthedocs.io](https://angband.readthedocs.io/en/latest/), S13) is
not copied: it is in the engine tree at `angband/docs/`, at the version we play.

The engine and the C Borg source are in `engine/angband/`, upstream's bytes (until milestone E: `angband/`, imported from borg-rs)
with history (T1.1, #4; replaced in milestone E, see [ENGINE.md](ENGINE.md)). Larger trees (full repositories) are cloned into `vendor/`
by `scripts/setup.sh` at the commits in `pins.json`; `vendor/` is not
committed.
