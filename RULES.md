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
   for the owner. A session that finds a goal or a task wrong says so in its
   PR or in [docs/HANDOVER.md](docs/HANDOVER.md), and goes on with the rest.
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
7. Other repositories (angband/angband, neo-angband-tracker, better-borg-c,
   borg-rs, borg-zoo, borg-agent) are **read only**, at pinned commits
   ([pins.json](pins.json)). Nothing is posted, filed or pushed there. A
   needed change there, or a C Borg defect found here, is described in
   the handover for the owner to route.
8. No posts to angband/angband or any other upstream project or forum.

## Play and evidence

9. **The game is not changed.** The engine tree is upstream `e473720` with
   the harness hooks, the declared fork corrections and the adopted C Borg
   carries inherited from borg-rs. Nothing else, except **harness-only
   hooks** under `#ifdef BORG_HARNESS` (such as the exact-boundary
   observation, T1.4, #7), each shown not to change the key stream before
   it acts. Control C Borgs are compiled from pinned sources overlaid in the
   build directory; the tree is not edited. Birth options are the game's
   defaults.
10. **Determinism.** Seed, build and profile reproduce a game's key stream.
    The Borg thinks with its own RNG and never moves the game's. Wall time
    never affects play. Decisions never iterate over unordered containers.
11. **Fair information** (GOALS D3): the Borg reads only what the C Borg
    reads. Every engine read goes through the engine boundary and is listed
    there.
12. **Every natural death is recorded** as a dossier under `docs/deaths/`
    (borg-rs format; the decision trace from M3), with the actual source,
    build, implementation, profile and seed. This applies from the first
    game played (T1.2, #5), whichever arm died. Repeats and replays never
    inflate independent counts.
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
    `file:function:lines` at `e473720`. Code taken from borg-rs keeps its
    attribution. The decision layer (strategy) is new code.

## Files and machines

17. Heavy local runs (builds of the engine, suites, cohorts) take the shared
    lock: `flock /tmp/angband-validation.lock <command>`. Watch and log long
    runs, and never kill another session's job.
18. Build output, saves, raw run directories and large logs are not
    committed. Curated evidence (dossier files, reports) is.
19. No secrets in the repository or in run records.
20. Population experiments run through borg-zoo and are started by the
    owner (they hold the account's Actions capacity for hours).
