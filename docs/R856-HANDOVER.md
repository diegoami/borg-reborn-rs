# R856: R4.5 fixed-main catch-up

Owner accepted R4 (#1093) and named receiving main
24be98732748230062882fe2b7d9601dba5e2cc0. Task branch
`task/T856-r45-main-carry` starts at accepted R4 40cbaf27, targets
feature/track-r and does not consume unpromoted track-p commits.

R45-PLAN.md and R45-PREFLIGHT.json record the fixed scope, source counts,
changed engine/donor/settings and required staged checks. The read-only merge
preview has 35 conflicts; no runtime or receiving merge is implemented yet.
Resolve provenance/parameter/native-shadow/tooling/archive conflicts without
discarding either track. Historical R4 receivers and all original evidence
stay fixed. The source carry is not a generic upstream re-vendor operation.

R5 stays paused. This task and the eventual R4.5 milestone need their own
review/CI gates. Do not confuse R4 acceptance, future receiving parity and
review-baseline integration. The current R4 review checkpoint CI was still
running when acceptance was recorded; preserve that distinction in metadata.
