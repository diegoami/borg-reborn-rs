## 2026-11-20: adopt the final reviewed Cemetery replay fix (#793)

Moves only death-tools-pin.txt from `46978936` to the final reviewed Cemetery
commit `0308f48d558be43a7dbb1e3219dce75d67df3907`, merged by Cemetery PR #91
as `918933a8`. Source PR #777 adopted the earlier implementation before the
review's required controller-inference fix was applied.

A hash-verified native parameter receipt now selects borg-rs even when an old
dossier has no borg_core entry. An explicit contradictory reborn setting still
fails closed. The no-receipt historical fallback remains reborn. The replay's
own native receipt is used for its effective-controller check. Two regression
tests accompany the implementation in Cemetery; source code remains a pointer.

The canonical review and response are on
https://github.com/diegoami/borg-cemetery/pull/91 . Its original birth evidence
and source/build/stream identities remain unchanged. The four historical native
dossiers now resolve correctly, but their missing external profile files remain
an explicit replay-input limitation, not permission to substitute defaults.
Pre-execution receipt coverage and the remaining Zoo-input-check limitation are
nonblocking findings recorded in that review, not claimed fixed by this pin.

API 1, game code, harness defaults, policy and references are unchanged. The
accepted Cemetery commit is reachable from main; it no longer depends on the
task branch remaining alive. No gameplay evidence or reports are duplicated here.

Validation uses BORG_CEMETERY_DIR pointing at the exact pinned native subtree:
the guard, all 96 consumer tool tests, 39 script tests, 81 native tests and the
four affected dossier plans pass. Results are recorded in the PR. No engine rebuild
or new gameplay-policy qualification is implied by this tooling-only adoption.

Left: owner review-complete/merge decision for this PR (#793), under RULES 1a.
No next milestone or population experiment is selected.
