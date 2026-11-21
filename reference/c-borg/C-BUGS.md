# Angband C findings and candidates

## Easy method: open a PR from an existing C proposal branch

[Compare the map-dump correction with Angband master](https://github.com/angband/angband/compare/master...diegoami:neo-angband-tracker:pr/c-borg-map-dump-cancel)

Review the diff and choose **Create pull request** if your account has the required permissions. Opening the comparison submits nothing. [Copy/patch alternatives and follow-up details](c-fix-handoff.md).

## The issue list is authoritative

[Browse all C findings and candidates](https://github.com/diegoami/neo-angband-tracker/issues?q=is%3Aissue+label%3Aupstream-c+label%3Afinding). Every finding has its own issue with source lines, discovery credit, evidence confidence and proposal links. This page does not duplicate that list or its live state.

- [Open C findings](https://github.com/diegoami/neo-angband-tracker/issues?q=is%3Aissue+label%3Aupstream-c+label%3Afinding+is%3Aopen)
- [Reported or unproven candidates](https://github.com/diegoami/neo-angband-tracker/issues?q=is%3Aissue+label%3Aupstream-c+label%3Afinding+label%3Acandidate)
- [Native C-game reproductions](https://github.com/diegoami/neo-angband-tracker/issues?q=is%3Aissue+label%3Aupstream-c+label%3Afinding+label%3Aevidence%3Ac-game-reproduced)

Reported candidates are not silently promoted to verified defects. Source issue open/closed state concerns the source correction; downstream carried/fixed state belongs only in linked Mandos adoption issues.

## Scope and supporting records

This tracker covers original Angband, especially its C Borg, alongside Neo Angband engine repairs. Neo Borg port defects belong in [our mod tracker](https://github.com/diegoami/neo-angband-borg-mod-tracker/issues).

[Per-finding import/provenance records](c-bugs/findings), [source snapshot](audit/c-reports/borg-rs-c-bugs.md), native scenario reports and [proposal patches](../patches) preserve evidence. They are historical import/audit material, not a second editable issue-state ledger.

Use the issue to update investigation, evidence conclusions, current proposals and resolution. The split JSON records preserve the imported source correspondence; regenerate this landing page and the data-only watch index with `node tools/c-bugs.mjs --write`, verify with `--check`.

No angband/angband issues, PRs or comments are submitted by us under the standing owner pause. See [AGENTS](../AGENTS.md), [RULES](../RULES.md) and the [session map](SESSION.md).
