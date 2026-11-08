# Handover

*Where the work stands, for the next session. Update it in every PR.*

## 2026-11-09: T1.0 source refresh routine (#3)

- `scripts/refresh-sources.py` reports, per pin: commits since the pin, new
  and changed death cases, new tags (new control arms), changed C Borg
  functions we cite, and issues updated since the pin was read. `--update`
  moves the pins, refreshes `reference/` (copy rules in `pins.json`) and
  regenerates `reference/deaths/INDEX.md`. Tests on synthetic repositories
  cover the report, update, dirty vendor and offline cases; CI runs them.
- RULES 21–23: every session starts with the report, every milestone with an
  `--update` PR; baselines are exact; lifted knowledge is watched.
- **First `--update` (PR closing #3):** borg-rs `6b58a46`→`e5cb263` (impact v6; its C Borg now differs from upstream in 26 files, issue #4 updated), better-borg-c `63fbc1f`→`8b26042` (deaths D0011–D0013 copied), borg-zoo `ffe5a05`→`9cd7423`, borg-agent `697abde`→`8f1dd7f`. The new deaths are read into SOURCES/GOALS/PLAN in a separate PR (RULES 21).
- **First real report (2026-11-09):** better-borg-c has three new deaths
  (D0011–D0013), 25 commits and 75 updated issues; borg-rs moved 7 commits,
  borg-zoo 3, borg-agent 9. The `--update` PR that brings them in is the
  last step of #3.

## 2026-11-09: M0 reviewed, M1 started, backlog moved to issues

- The owner merged the M0 review (#1); tag `m0` is on the merge
  (`ae64f12`). D1–D11 are accepted (GOALS.md).
- The backlog is now GitHub issues (owner request): GitHub milestones
  M1–M7+, labels `task`, `process`, `in progress` and `owner`, and issues
  #3–#24 for T1.0–T3.8, each with a **Needs** line linking its
  dependencies. PLAN.md keeps only milestones, exits and method.
- **Next:** #3 (T1.0, the source-refresh routine over every pin, death
  reports included; owner request), then #4 (T1.1, import the engine tree
  at refreshed pins).

## 2026-11-08: M0 review findings addressed (T0.4)

The owner's review of PR #1 had six findings, all verified and addressed
in the T0.4 PR:

| Finding | Change |
| --- | --- |
| Blocking: the primary endpoint has no tooling that computes it. borgh overshoots the cap, the adapter leaves `first_turn_at_depth` empty, and zoo stats exclude operational outcomes | GOALS *Observation and scoring rule*; PLAN T1.4 (exact boundary, depth stream), T1.7 (`tools/endpoint.py`), T1.8 (zoo change request); G1 prerequisites stated |
| Blocking: the imported `BORG_IMPL=c` is borg-rs's adopted C Borg (22 files differ from upstream), and observer-harness zoo cells cannot be paired with borgh | PLAN T1.1 corrected; new T1.3: `c-pristine` and `c-bbc-m7` controls built here on the same engine and `borgh`, `c-pristine` checked against the zoo's `4.2.6-rs-m0-c`; GOALS D2/D8/G1/G2 |
| setup `--strict` did not check that checkouts were at the pins | `scripts/setup.sh` checks out pins, reports WRONG REVISION, never moves dirty checkouts, `--checkout` moves clean ones (all three paths tested) |
| Death copies missed `overall-assessment.md` and other small evidence files | All root-level `.md`/`.json` of each case copied; external links listed in `reference/README.md`; a test fails on dangling internal links (checked that it fails when an assessment is removed) |
| Death export came after the first playing run | Minimal death export is part of T1.2, before its first smoke run; RULES 12 applies from the first game. Follow-up review: T1.4 checks only the C arms, and the every-arm check (`rust` included) is in T1.8 and the M1 exit |
| Nit: `BORG_IMPL=reborn` | `rust` everywhere, as borg-rs |

Follow-up review on #2: borg-rs is reclassified everywhere as engine and
tooling source and perception oracle, not a control. Its zoo results are
context (GOALS recommendation, non-goals and D8; AGENTS).

Still open for the owner: D1–D11, and routing the zoo change request once
T1.8 writes it.

## 2026-11-08: milestone 0 (foundation), ready for review

**Done**
- T0.1: sources read and evaluated ([SOURCES.md](SOURCES.md)). Specification,
  plan, curated death dossiers and zoo catalogue copied to `reference/`
  with provenance. Prior art outside Angband surveyed (AutoAscend, BotHack,
  TAEB/Saiph, qw, Rog-O-Matic, NetPlay). A digest of deaths and issues
  by subsystem is in [research/DEATHS-AND-ISSUES.md](research/DEATHS-AND-ISSUES.md).
- T0.2: goals, principles, staged targets and eleven owner decisions,
  all *proposed* ([GOALS.md](GOALS.md)). The design direction is in
  [ARCHITECTURE.md](ARCHITECTURE.md).
- T0.3: [PLAN.md](../PLAN.md) (M0–M7, tasks for M1–M3),
  [RULES.md](../RULES.md), [AGENTS.md](../AGENTS.md) (model-agnostic entry;
  `CLAUDE.md` only includes it), `pins.json`, `scripts/setup.sh`, the
  empty crate `borg-reborn-rs/` (`staticlib` + `rlib`, as borg-rs),
  repository consistency tests, CI.

**For the owner to decide (milestone 0 review)**
- D1–D11 in [GOALS.md](GOALS.md#owner-decisions). The ones that shape M1
  most: D2 (engine tree from borg-rs), D3 (information level), D4 (reuse
  of borg-rs code), D9 (workflow).
- Whether the `reviewed` branch starts at the root commit (so the M0 review
  PR shows everything), as set up here.

**Next**
- Done on 2026-11-09: see the newest entry above.

**Known limits**
- Nothing is built against the engine yet. The CI checks only the empty
  crate and the documents.
- The death digest was produced with a model's help from the issue bodies
  and dossiers at the pinned commits. Its classifications are leads, not
  findings. SOURCES.md's table of death patterns was checked by hand
  against the dossiers' `assessment.json`.
