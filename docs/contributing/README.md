# Contributor guide

*Layer 2 of the documentation (#817): for people who want to help, as
developers, analysts or reviewers. It explains how the work is organised and
the tools, each with what it is for and an example. To play the Borg, read
the [play guide](../user/GUIDE.md). The rules themselves are in
[RULES.md](../../RULES.md) and [AGENTS.md](../../AGENTS.md), which stay the
authority: this guide summarises them and links to them.*

| Page | What |
| --- | --- |
| [How we work](#how-we-work) (below) | tracks and milestones, issues, worktrees, pull requests, reviews |
| [Building and testing](building-and-testing.md) | CI's checks, the engine build, the smoke suite and its references, the shared lock |
| [Changing how the Borg plays](changing-play.md) | settings, parameters and their citations, the vendored core and its hooks |
| [Investigating a game](investigating.md) | replaying by seed, the decision trace, death dossiers, the recorder, guided debugging |
| [Measuring with the zoo](measuring.md) | specimens, arms, profiles and tiers; writing a request; reading a result |
| [The related repositories](repositories.md) | every project we use, what for, and its pin; keeping the sources current |

How the Borg works is in [docs/architecture/](../architecture/README.md):
- how the original C Borg plays;
- the borg-core we ship;
- the reborn plan, with its tracks P, R and Q.

What it and its analysts know of the game is in the
[Angband Strategy Manual](../strategy/README.md) and
[docs/GAME-KNOWLEDGE.md](../GAME-KNOWLEDGE.md).

## How we work

**The owner decides the scope** ([docs/GOALS.md](../GOALS.md)). Work
happens in **tracks**, each with its milestones listed in
[PLAN.md](../../PLAN.md):

| Track | Label | What | Integration branch |
| --- | --- | --- | --- |
| **P** | `track:P` | calibration of borg-rs's core: named parameters, a decision trace, fixes behind switches | `track-p` |
| **R** | `track:R` | the redesign: borg-rs's decisions re-expressed in our own domain, behind a shadow check | its own (PLAN.md) |
| Q | `track:Q` | the query surface for language models (only when the owner opens it) | |

A session works on one track, the one the owner names. Inside it, a session
works only in the milestone PLAN.md marks *in progress*.

**Issues.** Each task is an issue labelled `task`, with its track and
milestone labels (`P9`, `R5`, ...). Take the lowest-numbered open one whose
*Needs* are closed. Mark it **`in progress`** and comment with your branch
name before you start, so that two people never take the same task:

```bash
gh issue list --label task --label "track:P" --label P9 --state open
```

**Worktrees** (RULES 5a). Each task works in a git worktree of its own,
never in the shared checkout. The branch is `task/T<issue>-<short-name>`,
from `origin/track-p` for track P, otherwise from `origin/main`:

```bash
git worktree add .worktrees/t1234 -b task/T1234-short-name origin/track-p
```

**A pull request** has one task issue (RULES 5):
- it says what changed, how it was checked and what was not checked;
- a section *For the reviewer* says where to look;
- it ends with `Closes #N`;
- it adds a **handover note**, a new file
  `docs/handover/YYYY-MM-DD-<issue>-<short-name>.md` (RULES 5b,
  [docs/handover/README.md](../handover/README.md)). The note says what was
  done, what was found and what is left, and what is left becomes new
  issues.

Commits and PR bodies carry no AI attribution trailers (RULES 4).

**Reviews and merging.**
- **The reviewer.** The owner chooses the reviewer (RULES 6). A review is
  posted on the PR as a conversation comment, never as an *approve* or
  *request changes* review.
- **A task PR** is merged into its integration branch only after CI is
  green **and** the owner says the review is done (RULES 1a). Green CI alone
  is not enough.
- **A milestone.** At each milestone, `track-p` is promoted to `main`
  through a pinned review PR, which the owner accepts and merges.

**Evidence and claims.**
- **The engine is never changed** (RULES 9): our hooks are patches applied
  to a copy at build time.
- **No improvement is claimed** without a zoo measurement (RULES 14,
  [measuring.md](measuring.md)).
- **A fix for an observed defect ships switched on** in the PR that makes
  it, with its switch at 0 keeping the earlier play (RULES 28).
- **Gameplay evidence goes to borg-cemetery** (AGENTS.md, *Gameplay reports
  and scenarios*): death investigations, scenarios and trial reports, not
  this repository's `docs/`.
