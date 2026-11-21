# Game knowledge: what is used, and by whom

There are two readers of game knowledge here, and they read different
things:

- **the Borg while it plays**: it reads only the game itself and its own
  settings files; it uses no manual, no model and no network;
- **the people and agents who analyse and change it**: they read the
  engine source, the manuals and guides in this directory, and the evidence
  of recorded play.

As of 2026-11-21 (#816). The copies in this directory, their sources and
pins are listed in [README.md](README.md) (since 2026-11-21 the manuals, the C Borg documents and the core's design documents are originals kept here); where every Borg repository's
documentation is, in [DOCUMENTATION-MAP.md](DOCUMENTATION-MAP.md).

## 1. What the shipped Borg reads at run time

The default core (borg-rs's, `borg-core/`; [architecture](../docs/architecture/borg-core.md)):

| Input | Where it comes from | What the Borg takes from it |
| --- | --- | --- |
| The screen and the game's memory | the running engine (`engine/angband/`, upstream `574b618c`), through `borg-core/src/engine.rs` | what the C Borg reads: the map as displayed, messages, the character sheet, inventory and stores, and the engine's in-memory data tables (monster races, object kinds, classes and spells, loaded by the engine from [`lib/gamedata/`](../engine/angband/lib/gamedata/)) |
| `borg.txt` | `harness/borg.txt`, copied into the game's user directory (`user/borg.txt` of a run; `.angband-reborn/user/` when watching) | the C Borg's settings, the `borg_param_*` thresholds ([CALIBRATION.md](../docs/CALIBRATION.md)), per-class values (folded from `harness/borg_<class>.txt`), the core selection and errand (`borg_reborn_*`) |
| The scary list | `harness/scary_monsters.txt`, named by `borg_scary_list` | which known monsters make a level too dangerous for which characters (replaces the C Borg's compiled list) |
| Ladders | `borg-core/ladders.txt` (or `BORG_LADDERS`) | the per-class depth ladder of the `/ladder` errand |
| A plan | `harness/plans/*.txt`, named by `borg_plan` or `BORG_ERRANDS_PLAN` | depth caps by class and character level ([GAME-PLAN-LANGUAGE.md](../docs/GAME-PLAN-LANGUAGE.md)) |

The knowledge itself (which monster is dangerous, which item is worth
buying, when to flee) is in the core's code and in those settings: the C
Borg's rules as translated, with the thresholds track P has named. The
frozen reborn core (`BORG_CORE=reborn`) reads the engine directly through
its own perception and its profile `harness/profiles/reborn-default.txt`
instead ([reborn-layers.md](../docs/architecture/reborn-layers.md)).

## 2. What analysis reads, in order of authority

| # | Source | Where | Use |
| --- | --- | --- | --- |
| 1 | **The engine** | [`engine/angband/src/`](../engine/angband/src/), [`engine/angband/lib/gamedata/`](../engine/angband/lib/gamedata/) | The only authority: every formula, table, prompt or key sequence a rule relies on is checked and cited here (RULES 16) |
| 2 | **The Angband Strategy Manual** | [`c-manual/`](c-manual/README.md) | A source-grounded guide to how 4.2 works underneath and how to survive it: mechanics (ch. 1–9), exploration, town, the game plan by phase, escapes (ch. 10–13), **class playbooks (ch. 14)**, *notes for the borg* (ch. 15) and experiments (ch. 16), appendices A1–A10. Read the chapter before designing a rule, then check its numbers in the engine |
| 3 | **The official user manual** of this version | [`c-manual/user-manual/`](c-manual/user-manual/), also [`engine/angband/docs/`](../engine/angband/docs/) | Commands, screens, options, birth, and upstream's own *Players' Guide* (`guide.rst`, written for 3.5 and marked by upstream as a little out of date) |
| 4 | **The C Borg's own documentation** | [`engine/angband/docs/hacking/borg.rst`](../engine/angband/docs/hacking/borg.rst) (running it, its commands), [`vendor/angband` `src/borg/borg.txt`](https://github.com/angband/angband/blob/574b618c3bfec595f3503a58f8e488b1c4ae3a46/src/borg/borg.txt) (its annotated settings); [`c-borg/BORG_SPEC.md`](c-borg/BORG_SPEC.md) (a specification of how it plays) and [`c-borg/TUNABLE-BORG-PLAN.md`](c-borg/TUNABLE-BORG-PLAN.md) | What the C Borg intends; the architecture of [original-c-borg.md](../docs/architecture/original-c-borg.md) |
| 4b | **How the C Borg went wrong, and how it was fixed** | [`c-borg/c-bugs/`](c-borg/c-bugs/README.md) (the defect catalogue), [`better-borg-c/measurement/`](better-borg-c/measurement/README.md) (one page per verified fix), [`borg-core/c-bugs.md`](borg-core/c-bugs.md) | Why a rule of the shipped core is written as it is, and which of the C Borg's behaviours are defects rather than strategy |
| 4c | **The shipped core's own design** | [`borg-core/`](borg-core/) (town planner, trip policy, class family, shop shortcuts) | The intent behind borg-core's town and trip code, which borg-rs added to the C Borg |
| 5 | **What agent players learned** | [`borg-runner/`](borg-runner/) (S15), and the retired [`borg-agent/knowledge/`](borg-agent/knowledge/README.md) | Language-model agents playing from the screen; every line traced to the run that found it. Leads, not rules |
| 6 | **Evidence from recorded play** | [`deaths/`](deaths/INDEX.md) (the C Borg's and borg-rs's death dossiers), [borg-cemetery](https://github.com/diegoami/borg-cemetery) (ours), each run's `trace-core.jsonl` | What actually killed a Borg. A lead until the zoo measures it (RULES 14) |

## 3. Strategy guides: what exists, and what may be copied

Surveyed 2026-11-21 for #816. A guide is copied here only when its licence
allows redistribution in a GPLv2 repository, with its source and date, as
the other copies are.

| Guide | Version | Licence | Here |
| --- | --- | --- | --- |
| Angband Strategy Manual (neo-angband-tracker) | 4.2.x, written against this engine | the owner's | **copied**, `c-manual/` |
| Official user manual and in-game help, including *A Players' Guide to Angband* | 4.2 (guide: 3.5) | GPLv2 or Angband licence (`docs/copying.rst`) | **copied**, `c-manual/user-manual/`; also in-tree |
| `docs/hacking/borg.rst`, the C Borg's annotated `borg.txt` | 4.2.3+ | same | **in-tree** in `engine/angband/` (the engine is upstream byte for byte): no second copy |
| APW Borg readme `borgread.txt` and its `borg.txt` (Andrew P. White) | 3.0.x–3.5.1 | shipped under the 3.5.1 release's GPLv2 / Angband licence; no notice in the file itself | **not copied**: historical, mechanics changed since; readable with `git -C vendor/angband show 3.5.1:src/borg/borgread.txt` (owner may decide to copy) |
| StrategyWiki *Angband/Gameplay* | unclear, 3.x–4.1 | CC BY-SA (not GPLv2-compatible) | not copied: low value, would need a separate attributed file |
| angband.live forum threads on the 4.2 classes (blackguard, ranger and spell reviews, diving strategy) | 4.2.x | no licence stated; terms not checked | not copied (copyright applies); the Strategy Manual's ch. 14 covers the 4.2 classes from the source |
| Older community guides and FAQs (rgra FAQ, 3.x guides) | 2.x–3.x | none stated | not copied: obsolete for 4.2 |

No dedicated 4.2 guide for a single class was found; the 4.2 class advice
outside this directory is in forum threads and upstream's `birth.rst`.
