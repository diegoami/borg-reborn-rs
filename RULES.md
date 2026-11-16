# Rules

These hold for every session working on this repository, local or cloud,
whatever the model.

## Owner and scope

1. **Track P integrates on `track-p`, never directly on `main`** (owner,
   2026-11-15; branch spelling confirmed as `track-p`). A session creates P
   task branches from `track-p`, targets task PRs at `track-p`, and merges
   them there after CI is green. CI success is not review approval. At each
   milestone, stop and open the review/promotion PR **`track-p` → `main`**,
   pinned to an exact commit. The owner chooses/runs the review; findings
   are fixed through PRs into `track-p`. Only after the milestone review is
   accepted and CI is green does the owner merge that promotion, or
   explicitly delegate that reviewed merge. A session never promotes P to
   `main` under the generic task-merge permission. Do not integrate the
   next P milestone before this review gate, unless the owner explicitly
   changes the gate. Close completed P task issues explicitly after their
   `track-p` merge, linking the PR/commit (GitHub closing keywords normally
   wait for the default branch). Other tracks must not use a closed P task
   to bypass its review gate by carrying unreviewed P changes onto `main`.
   **Other tracks retain their own authorized workflow**; absent a track
   exception, a session merges task PRs into `main` after green CI, and
   leaves milestone review PRs (`main` → `reviewed`) to the owner.
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

Pre-rule dependency exception: P1–P3 were already on `main` at `8351f0a3`.
For those milestones, owner acceptance of the retrospective review #259
satisfies the P review dependency; a second runtime promotion is unnecessary.
A reviewer recommendation alone is not owner acceptance. New P work still
requires the reviewed `track-p` → `main` promotion. This clarification does
not undo other-track work already authorized before the gate.

## Git and GitHub

4. No AI attribution trailers in commits or PR bodies: no `Co-Authored-By`
   lines for a model and no "Generated with" footers.
5. One task issue per PR, on a branch named `task/Tn.m-short-name`; the PR
   says `Closes #N`. A branch may start from an unmerged one; the PR then
   says "stacked on #n". Each PR updates the handover. Mark the issue
   `in progress` when you start, so two sessions never take the same task. Never amend
   published commits, force-push `main`, `track-p` or `reviewed`, or move tags.
6. **Reviews happen at milestones, not per task.** When a milestone's last
   task is merged, the session stops and opens the review PR (PLAN.md).
   For P, the task merges into `track-p` and the review precedes promotion
   to `main` (RULES 1); the older `main` → `reviewed` flow does not authorize
   direct P task merges into `main`.
   **The owner chooses the reviewer** (owner, 2026-11-09). For every review
   (a milestone review, a re-review, a review of a spec or a plan) the
   session writes the review prompt, pinned to an exact commit, and gives it
   to the owner. The owner chooses the reviewer and runs it. A session never
   picks a reviewer or launches one itself. When the review arrives, the
   session processes its findings.
7. Other repositories (angband/angband, neo-angband-tracker, better-borg-c,
   borg-rs, borg-zoo) are **read only**, at pinned commits
   ([pins.json](pins.json)). Nothing is pushed or merged there. A session
   may open **issues**, and keep its own issues up to date, on borg-zoo,
   borg-rs, better-borg-c and neo-angband-tracker, when the
   owner asks or when this repository needs a change there. It follows
   that repository's own conventions and links the issue in the handover.
   Anything else needed there, or a C Borg defect found here, is described
   in the handover for the owner to route. angband/angband stays off limits
   (RULES 8). (Owner decision, 2026-11-09, applied after the M1 review.)
8. No posts to angband/angband or any other upstream project or forum.

## Play and evidence

9. **The game is not changed.** The engine tree, `engine/angband/`, is
   upstream `42cd156ac` byte for byte (a test checks it). Our only changes are
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
12. **Every natural death of a measured run is recorded in borg-cemetery**,
    with the actual source, build, implementation, configuration, inputs and
    lineage. The Cemetery owns incident/death analysis, saved or constructed
    scenarios and behavioral/counterfactual trial reports. Source repositories
    keep code/tests, checksum-pinned fixture references and brief validation
    summaries with canonical links; do not publish duplicate gameplay reports
    or new validation dossiers here. Existing docs/deaths/ is historical, not
    a destination for new evidence. Changes to that retained archive require
    explicit source-qualified migration and integrity checks.
    A measured run includes harness runs, suites, CI and zoo cells. CI may use
    temporary registries/artifacts, but retained comparisons must point to
    canonical Cemetery evidence; never weaken missing-death checks or compare
    a temporary registry only against itself. BORG_DEATH_RECORDS_DIR selects
    isolated working storage, not a different owner for permanent reports.
    Watched/recorded incidents may have unknown birth seeds; preserve their
    recordings and saved states without inventing a measured seed identity.
    Original player/mixed recordings and working saves may stay in
    player-cemetery, linked by the canonical case. Case-specific alternative
    plays and scenario trials belong to Cemetery. Replays, reloads and branches
    of one life never inflate independent counts. Test-feature/fabricated
    scenarios are explicitly labelled and kept apart from natural measured
    deaths. Original published hashes and historical records are preserved.
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
    `file:function:lines` at `42cd156ac`. The **Angband Strategy Manual**
    (S14) and the **official user manual** under `reference/c-manual/` are
    the first reference for how the game behaves: read the relevant chapter
    before designing a rule, then check every number it gives in
    `engine/angband/` (`src/` or `lib/gamedata/`) and cite the engine there.
    Code copied from another project is recorded once in
    docs/PROVENANCE.md and is ours from then on (docs/SPEC.md §2). The
    decision layer (strategy) is new code.

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
    borg-rs, borg-runner, the specification, the zoo and the death reports.
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
    it is linked into a game build the harness or the zoo measures (ours or
    the zoo's) or reaches the zoo contract; Q5's live host, if built, is its
    own binary and never a measured build. Query
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

## Owner recorder integration, 2026-11-14

Task #210 permits the standalone angband-recorder crate and its versioned
BORG_RECORDING hooks on the generated engine copy. This supplements rule 9's
guard list for the optional recorder only; engine/angband remains pristine,
and the normal harness explicitly disables the recorder. The dependency is
sealed in recorder-pin.json and linked through our existing Rust library.
No strategy, default frontend flags or measured baseline changes are implied.

## Guided debugging sessions (owner, 2026-11-15)

26. **Debug reported play from preserved runtime evidence.** For an
    owner-requested guided session, the agent prepares and starts a
    recorder-enabled game with a named character, private session files,
    the exact executable/settings/starting save preserved, a core decision
    trace and errand outcomes. Follow
    [docs/GUIDED-DEBUGGING.md](docs/GUIDED-DEBUGGING.md). The owner reports
    the unexpected behaviour and saves if the game responds; the agent
    finds and verifies the save, preserves the incident, and attempts
    reproduction itself. A save alone does not preserve the Borg's
    in-memory state or the actions leading to the problem. Keep the
    recording and original evidence intact, reproduce in a separate
    directory with the original build first, and state whether replay
    actually verified. If saving fails or the game hangs, capture the live
    evidence before closing it; do not require a successful save to begin
    diagnosis. Never claim a character was recovered or a cause established
    from a filename, an old save, or static code inspection alone. Fixes
    get a bounded reproduction/regression check. Recorder defects are
    fixed through PRs in `diegoami/angband-recorder`, not by silently
    patching the vendored recorder or adding consumer engine workarounds;
    adopting a recorder update here is a separate pin-update change.
    For supported recorded builds, retain recent living native checkpoints
    and periodic older saves with the run in player-cemetery; record the
    actual retention settings/capability. Never claim an older recorder
    supports rolling saves just because environment variables were set.
    Use recorded-frame viewing and the nearest earlier copied checkpoint
    for fast incident work, proving original-build reproduction separately
    from a fresh-Borg counterfactual. A save does not restore Borg memory.
    Watched sessions retain rule 12's distinction from measured runs;
    publishing recordings is a
    separate owner request.
