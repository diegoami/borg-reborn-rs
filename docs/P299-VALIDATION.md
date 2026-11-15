# Sweep provenance and fragment assembly (#299)

The handoff names runtime commit `2ec1f2507ec81f375a7967b2876597865458aa76`
but previously stated the earlier build source `1f5eb966…`. Computing
`harness/identity.sh source-hash rust` in a clean detached worktree of that
exact commit gives:

`cb48d94396894881d4ae001b2a49b994e86da5f675e8f2444a7c9979c85e5b90`.

CORE-SWEEP.md now reports that identity, matching P269-VALIDATION.md.
No specimen ref, policy, suite assignment, endpoint or sample size changes.

Review #292 F5 needs qualification: the prompt already named the profile
file explicitly. The remaining assembly ambiguity is clarified by naming
the two catalogue fragments and the separate experiment manifest, their
load order and the need to keep all three at the same reviewed handoff
revision. The handoff commit and runtime specimen commit are distinct pins.
The catalogue note repeats that instruction.

The actual zoo catalogue at pin c5f1b042 validates the supplied fragments
and expands the unchanged experiment to384 cells; development128 and
confirmation900 seed assignments remain disjoint. These static checks
launch no games or population experiment. Global existing-suite collision
checks and owner review/promotion/dispatch remain the zoo catalogue PR's
gates. Gameplay inputs are identical to the stacked #296 source; no new
build or measured run is needed for this documentation-only change.
