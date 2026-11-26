# Independent M1 review prompt

Review **M1 — Recorder extraction and recorded-play delivery** of
`diegoami/angband-recorder`. The owner has selected you and launches this review
separately from the implementation session.

Read the package at the immutable package commit linked on
https://github.com/diegoami/angband-recorder/issues/2:

- docs/milestones/M1.md
- docs/milestones/M1-inputs.json
- docs/milestones/M1-evidence.json
- docs/review/M1-REPRODUCE.md
- README.md, AGENTS.md, spec/streams-v1.md and CREDITS.md

**Treat all implementer statements, green CI and local receipts as claims to
verify, not as an independent review already completed.** Review the exact
listed commits/ranges, not current main. The C window uses its merge's first
parent; later v13R, reborn redesign/calibration and live cemetery data are out
of scope. The existing v0.1.0 tag is not an acceptance verdict.

Audit the Rust core and C ABI/hooks; completeness/order of external inputs;
RNG/bot-decision separation; forked saver/stream/clock behavior; observation
side effects; malformed/exhausted/unconsumed inputs and truthful finalization;
actual build identity, compatibility and pin enforcement; single-Rust-runtime
linking and recorder-off controls. Review source/startup/load/continuation
boundaries and licences/provenance, not just successful fixtures.

Also audit the delivered consumer delta: human start/resume/backup and optional
Borg behavior; bounded random runs; permanent player-cemetery resolution;
opt-in publication and its actual Git scope; dirty/concurrent/unpushed state;
archive path/resource/mode/binary handling; restore/verify requirements; C
recorder retirement preserving normal tools; reborn pristine-engine and default
reference behavior. Check the earlier failure fixes and their negative controls.

Use isolated worktrees, private profiles/displays and temporary bare Git remotes.
Never push a review recording to a real cemetery, touch active player data,
reset/clean active components, kill another validation holder or post upstream.
Do not edit implementation source, start another feature/strategy/carry or
change assertions/seeds/caps/timeouts. If you need a diagnostic overlay, keep it
isolated, describe it and preserve the unmodified control. Component rules and
normal death evidence obligations still apply.

Return:

1. **approve / approve with corrections / do not approve** for this scoped M1,
   with a short rationale. Do not certify unimplemented future features.
2. Findings **M1-F1..n**, each with severity, exact repository/commit/file:line,
   concrete failure or misleading claim, reproduced vs read-only status,
   evidence/command, acceptance impact and a proposed correction direction.
3. Checks actually run, exact inputs/toolchains, failures/skips and unavailable
   evidence; distinguish these from existing author/hosted results.
4. Remaining limitations and whether the documented scope/claims are honest.
   Known gaps do not excuse actual bugs in implemented paths.

Post the report on issue #2, or link a report at an immutable commit there.
`docs/review/M1-REVIEW.md` is reserved for your report if you write one locally;
the author has not created a verdict file. Findings/corrections and the owner's
acceptance decision are tracked on issue #2. Do not move existing tags or create
an acceptance tag yourself. No next milestone starts automatically.
