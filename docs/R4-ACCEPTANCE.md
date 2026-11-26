# Owner acceptance of R4

On 2026-11-23 the owner accepted corrected immutable R4 source
`40cbaf272193abb44dd80fad2cfeec24b627be22` in review #1093, combining the
full Claude milestone review #1031 and the successful correction re-review
#1041. The exact source tree is `32f7572a5fda6dac351d4bd746b0dde1199b6eda`.
Acceptance was recorded publicly on #1093; its CI/review-baseline integration
status is a separate gate and must not be inferred from the owner decision.

The owner reaffirmed the previously planned R4.5 catch-up (#856) before R5,
then selected main `24be98732748230062882fe2b7d9601dba5e2cc0` as its fixed
receiving pin. See R45-PLAN.md. The old reviewed snapshots and evidence remain
immutable. This accepts R4's stated scope and limitations; it does not expand
natural-branch coverage, deliver #293, promote R into main/global reviewed or
accept the unfinished R4.5 carry. Nonblocking audit residuals remain #1092.
