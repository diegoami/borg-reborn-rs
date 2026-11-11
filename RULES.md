# Rules

These hold for every session working on this repository, local or cloud,
whatever the model.

## Owner and scope

1. **A session merges its own task PRs** into `main` once CI is green, with
   no approval needed. The one merge a session never makes is a milestone's
   **review PR** (`main` → `reviewed`). That one is the owner's.
2. Scope comes from [docs/GOALS.md](docs/GOALS.md), [PLAN.md](PLAN.md) and
   the task issues of the current GitHub milestone.
   **Work only inside the current milestone.** Work outside them waits
   for the owner. The secondary track Q (GOALS D12) has its own
   milestones; a session works on one only when the owner opens it, and
   never in the same PR as primary work. A session that finds a goal or
   a task wrong says so in its PR or in [docs/HANDOVER.md](docs/HANDOVER.md),
   and goes on with the rest.
3. The owner's decisions (GOALS.md, *Owner decisions*, once confirmed) are
   not reopened by a session. A session may report evidence against one.

## Git and GitHub

4. No AI attribution trailers in commits or PR bodies: no `Co-Authored-By`
   lines for a model and no "Generated with" footers.
5. One task issue per PR, on a branch named `task/Tn.m-short-name`; the PR
   says `Closes #N`. A branch may start from an unmerged one; the PR then
   says "stacked on #n". Each PR updates the handover. Mark the issue
   `in progress` when you start, so two sessions never take the same task. Never amend
   published commits, force-push `main` or `reviewed`, or move tags.
6. **Reviews happen at milestones, not per task.** When a milestone's last
   task is merged, the session stops and opens the review PR (PLAN.md).
   **The owner chooses the reviewer** (owner, 2026-11-09). For every review
   (a milestone review, a re-review, a review of a spec or a plan) the
   session writes the review prompt, pinned to an exact commit, and gives it
   to the owner. The owner chooses the reviewer and runs it. A session never
   picks a reviewer or launches one itself. When the review arrives, the
   session processes its findings.
7. Other repositories (angband/angband, neo-angband-tracker, better-borg-c,
   borg-rs, borg-zoo, borg-agent) are **read only**, at pinned commits
   ([pins.json](pins.json)). Nothing is pushed or merged there. A session
   may open **issues**, and keep its own issues up to date, on borg-zoo,
   borg-rs, better-borg-c, neo-angband-tracker and borg-agent, when the
   owner asks or when this repository needs a change there. It follows
   that repository's own conventions and links the issue in the handover.
   Anything else needed there, or a C Borg defect found here, is described
   in the handover for the owner to route. angband/angband stays off limits
   (RULES 8). (Owner decision, 2026-11-09, applied after the M1 review.)
8. No posts to angband/angband or any other upstream project or forum.

## Play and evidence

9. **The game is not changed.** The engine tree, `engine/angband/`, is
   upstream `e473720` byte for byte (a test checks it). Our only changes are
   the declared patch series of `engine/patches/` (docs/ENGINE.md): harness
   hooks under `#ifdef BORG_HARNESS` or `USE_BORGH`, applied to a copy at
   build time, each shown not to change the game with the guards off. Birth
   options are the game's defaults. No C Borg is built here (docs/SPEC.md
   §6).
10. **Determinism.** Seed, build and profile reproduce a game's key stream.
    The Borg thinks with its own RNG and never moves the game's. Wall time
    never affects play. Decisions never iterate over unordered containers.
11. **Fair information** (GOALS D3): the Borg reads only what the C Borg
    reads. Every engine read goes through the engine boundary and is listed
    there.
12. **Every natural death of a measured run is recorded** as a dossier
    under `docs/deaths/` (borg-rs format; the decision trace from M3), with
    the actual source, build, implementation, profile and seed. A measured
    run is one played through `harness/run.sh`: a suite, a single run, CI
    or the zoo's cells. This applies from the first game played (T1.2, #5),
    whichever arm died. Games watched with `watch-borg.sh` are not measured
    runs and are not recorded (owner, 2026-11-10, E review finding 3): they
    have no seed, key log or summary; a death seen there that matters is
    reproduced as a measured run and recorded then. Repeats and replays
    never inflate independent counts. CI writes its dossiers to a temporary
    registry, uploads it, and fails if a death is not in `docs/deaths/` or
    a run that died has no dossier (`tools/reconcile-deaths.py --runs`);
    deaths of test-feature builds are not natural deaths of an arm and are
    kept apart.
13. **Every run counts.** Stalls, timeouts, crashes and harness failures are
    reported, never dropped and never relabelled as deaths. Never weaken a
    seed list, a cap, a timeout or an assertion to get a green result.
14. **No claim of improvement** without a prespecified endpoint, a sized
    comparison and a confirmation suite (borg-zoo's rule). Sealed seed sets
    stay sealed until the owner opens them.
15. **No exploits** (GOALS D5): no munchkin start, lunal mode, money or self
    scum, stair-scumming, or cheat death inside the Borg.
16. **Knowledge is lifted, decision code is not.** Formulas, tables, key
    sequences and prompt answers taken from the C Borg cite
    `file:function:lines` at `e473720`. Code copied from another project is
    recorded once in docs/PROVENANCE.md and is ours from then on
    (docs/SPEC.md §2). The decision layer (strategy) is new code.

## Files and machines

17. Heavy local runs (builds of the engine, suites, cohorts) take the shared
    lock: `flock /tmp/angband-validation.lock <command>`. Watch and log long
    runs, and never kill another session's job.
18. Build output, saves, raw run directories and large logs are not
    committed. Curated evidence (dossier files, reports) is.
19. No secrets in the repository or in run records.
20. Population experiments run through borg-zoo and are started by the
    owner (they hold the account's Actions capacity for hours).

## Moving sources

21. **Every source moves** (owner, 2026-11-08): angband, the C Borgs,
    borg-rs, borg-agent, the specification, the zoo and the death reports.
    Every session starts with `python3 scripts/refresh-sources.py` and reads
    what it reports, new deaths first. Each milestone starts with a
    `--update` PR that moves the pins and refreshes `reference/` and the
    death index; a task may also do one when it needs a newer source. Pin
    moves are their own PRs, never mixed with other work.
22. **Baselines are exact.** A comparison names exact zoo specimens and
    builds, never "latest"; comparisons are borg-zoo's (docs/SPEC.md §6). A
    change to the engine's patch series, the frontend or the build is a
    dedicated PR, reproduces the references or explains every difference
    (docs/SPEC.md §10), and starts a new measurement baseline; results stay
    with the build they ran on.
23. **Lifted knowledge is watched.** Formulas, tables and key sequences
    lifted from a C Borg cite `file:function:lines` in the form the
    refresh report recognises, so a later fix to that function in
    better-borg-c or borg-rs is reported and reviewed.

## The query surface (secondary goal Q, owner 2026-11-11)

24. **The measured Borg plays without a model.** The Borg's code never
    calls a model, and the zoo specimen is always the model-free Borg. The
    core never depends on `borg-mcp` (docs/QUERY-SURFACE.md), and nothing of
    it is linked into the zoo's build or reaches the zoo contract. Query
    mode never sends keys, and a live query never changes `keys.log`. A
    game played by a model through play mode is a **separate arm**: never
    named or reported as the Borg, with the model, its effort and every
    tool call in the run record, and replayable key for key.
25. **The server has no judgement of its own.** Every answer comes from a
    function the Borg plays with, through the versioned `query` facade.
    A new computation goes into the core first, with its own task and
    tests. A change to a report type or a tool schema bumps
    `query::VERSION`. In play mode the model chooses only among executor
    actions; keys and prompts are the executor's, and rules 11 (fair
    information) and 15 (no exploits) bind the model as they bind the
    Borg. A model-played death is recorded like any other (rule 12),
    under its own arm.
