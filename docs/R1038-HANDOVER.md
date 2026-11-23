# R1038: R4 milestone review corrections

Follow up Claude review #1031 at immutable source 0f33d653. The frozen branch
is unchanged. This task targets feature/track-r and changes source audits,
coverage reporting, tests and documentation only; gameplay/defaults/engine and
all measured raw evidence remain unchanged.

The item-use guard reverses the hash-pinned native seams to accepted R3 source,
then independently compares accepted/native/owned bodies. Its matching-mutant
test honestly refreshes the seam and still rejects native policy drift. CI now
runs the guard. Power normalization preserves compound-operator separation and
rejects unsupported static/impl/macro definitions rather than silently ignoring
them. The entry gate retains pooled strict requirements and separately reports
explicitly declared natural/fixture-enabled run counters; run classification is
not per-call synthetic tracking or branch coverage.

R4-EXIT-REPORT.md lists full45 fixture-only entries/roots and qualifies smoke,
focused-seed, logging and no-engine-consumption claims. log_buy process effects
remain shared and are now disclosed; its implementation and fixture bytes are
unchanged. Retained gates are rechecked, not rerun or relabelled as new games.

This task needs green CI and owner review completion before merge. After fixes
integrate, freeze a new exact R4 review snapshot and hand its delta to the owner;
never move the existing reviewed branch or infer milestone acceptance. No R5,
main or global-reviewed promotion. Original #1031 review and evidence remain
accessible at their exact pins.

Local verification: 62 source and 139 tooling tests pass; all four source
audits, P4 carry, metrics and the exact published evidence verifier pass.
The retained full45 check/switch gate rechecks each cover 45 natural runs and
one fixture-enabled run, with the same five entries and two roots supported
only by the latter. Combined strict coverage still passes with zero mismatches.
The runtime input hash remains equal to the accepted R4 runtime. No gameplay
source or original evidence bytes were changed. The initial regression for an
honestly refreshed matching mutant demonstrated that reversing a seam alone
is insufficient; the final guard also compares accepted-to-native bodies.
