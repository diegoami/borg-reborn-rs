## 2026-11-22: Reviewers post their own review on the PR (#948)

**Done.** RULES 6, AGENTS.md (P gate and Models) and PLAN.md now put the
posting of a review on the reviewer the owner named, not on the session
that launched it. The reviewer posts a conversation comment on the PR (or
the issue) through any write path it has (`gh` with `--repo`, a connected
GitHub App or connector, an MCP GitHub server), headed with the reviewer,
where the owner named it, the pinned commit and the head at review time,
and answers with the comment's URL. A reviewer with no write path says so
first and gives the full review; only then does the session post it,
unedited. The session never posts a second copy. Every review prompt says
this in its first lines, with the exact command, and repeats it at its
end. A GitHub review body (type *comment*) is no longer allowed: it does
not show in the PR's comment list.

**Found.** The first version of #942 defined "cannot post" as "no `gh`".
Its reviewer (MiniMax-M3, through a GitHub connector) had no `gh` and
posted anyway; the rule would have told it to answer in chat. The review
also found the older sentences that still gave the posting to the
session. Both are fixed in #942.

**Left.** `track-p` gets the rule with the next sync of `main`.
