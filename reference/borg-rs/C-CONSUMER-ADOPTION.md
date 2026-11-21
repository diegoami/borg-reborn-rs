# Historical: follow verified better-borg-c milestones after M5

> **Superseded for new development after impact v10 (2026-11-10).** Starting
> from the accepted `4.2.6-impact-v10` snapshot, borg-rs diverges from BBC and
> develops a family of Borgs under the root `CLAUDE.md` rules and
> [borg-rs#25](https://github.com/diegoami/borg-rs/issues/25). Later BBC
> milestones are not an automatic queue, and new family variants need not match
> the C keypress stream. This contract remains the record and acceptance rule
> for the completed M2-M10 carry lane, including its `current`-control heritage;
> it does not schedule post-M10 family work.

Owner-approved direction, 2026-11-07: M5 freezes the completed port plus
post-M4 corrections/tooling; subsequent Rust bug-fix work tracks the maintained
C consumer's verified milestones. The current M5 candidate and review are in
[M5-PLAN.md](M5-PLAN.md) and [review#2](https://github.com/diegoami/borg-rs/issues/2).
M5 has independent pass and [owner acceptance](https://github.com/diegoami/borg-rs/issues/2#issuecomment-6712169628); tag m5 points
to `0203227` and review#2 is closed. On2026-11-08 the owner selected C
milestone2, runtime2675d198e4c3fcb1c69ee63ef25a2b6ac27ab96f,
[independently approved](https://github.com/diegoami/better-borg-c/issues/38#issuecomment-6705894080).
The first bounded batch is tracked on [Rust#6](https://github.com/diegoami/borg-rs/issues/6)
and in [adoption/c-m2](adoption/c-m2/README.md). C approval identifies the
target; Rust implementation/review state is recorded separately.
This contract supersedes old direct neo-angband-tracker queue scheduling for
the post-M5 lane; historical reports and their ownership remain preserved.

Owner update (2026-11-10): the consumer's milestone 11
(`4.2.6-impact-v11`) is carried as **v11R** under the same pin-and-carry
rules; record [adoption/c-v11r](adoption/c-v11r/README.md). It is an explicit
carry, not an automatic queue for later targets.

Owner update (2026-11-13): the consumer's milestone 12
(`4.2.6-impact-v12`, `4e1b912cb`) is carried as **v12R** under the same
pin-and-carry rules; record [adoption/c-v12r](adoption/c-v12r/README.md). It
carries the consumer's 0118 NEWKILL-95 and the official angband import since
impact v11, and retires the consumer's carries 0001 and 0002 because upstream
now carries that logic. It is an explicit carry, not an automatic queue for
later targets.

## Ownership and source of work

- better-borg-c owns accepted C bug-fixing checkpoints, native controls,
  engine integration and its source-tracker relationships. Rust adopts a
  named accepted milestone at an exact commit, with linked local adoption
  issues. The native C repository remains independently owned.
- borg-rs owns Rust translation, engine/FFI compatibility, C/Rust parity,
  regressions and its own death/measurement evidence. A C-only patch does
  not update Rust when its C Borg files are excluded from the Rust link.
- neo-angband-tracker links remain canonical historical/source provenance.
  Its issue list is not a second Rust adoption queue after the cutover.
  Do not independently race a source proposal ahead of C milestone adoption.
- New suspected C defects: record the actual pin/evidence, deduplicate C
  consumer issues and existing source links, and coordinate through the C
  consumer's ownership. A confirmed original-C report follows that lane's
  existing source route; do not duplicate or rewrite past findings. Rust-only
  defects stay in borg-rs. No automatic cross-repository issue creation is
  introduced. Never post issues/PRs/comments on angband/angband.

## Pin the accepted C target

Each adoption record includes the consumer milestone/review acceptance link,
full source SHA, previous accepted C/Rust baseline, official engine base,
engine/data/content/options/ABI hashes or immutable manifests, patch-ledger
snapshot and relevant before/after/negative-control evidence. Include effective
C/Rust configurations and seed/save/calendar/input origin as applicable.

A merged PR, source proposal, tool import pin, release name or moving main
head alone does not establish accepted C milestone status. Record any approved
subset and excluded fixes/dependencies explicitly. No automatic version chase,
merge, release or wholesale donor-tree replacement is part of this contract.

Tool sources08a5cd019 and f3ced6219768 in IMPORTED.json remain tooling provenance.
They are not by themselves accepted gameplay targets. The engine base moves
through a deliberate, qualified adoption: it is currently the official
import `42cd156ac` (v12R, owner-accepted 2026-11-13) plus the documented
local fork corrections and harness hooks. The base is not itself a
constraint - the constraint is the path: verified milestone, review,
owner acceptance and tag.

## Reconcile and implement a bounded batch

Before code changes, map the C patch ledger against existing local #287/#290/
#322/#295/#345 corrections, their C/Rust implementation and tests. Classify
already present, equivalent, superseded, new, engine-only, Rust translation
required, deferred or incompatible. Preserve source/consumer issue and commit
links, credit and licence; never count an already carried correction twice.

Pin interacting fixes/dependencies together when needed, while keeping each
correction/removal unit reviewable. Engine changes require compatible Rust
headers/FFI and identical engine/data for both implementations. Verify the
actual source and link inputs; C Borg files must stay excluded from production
Rust. Do not replace original references to make a changed target pass.

For each adopted correction, carry the native scenario and meaningful controls,
show its expected result and the old failure where applicable, translate the
actual semantics to Rust and check noninterference for observers/harnesses.
Source reading alone or equality of two wrong implementations is insufficient.
Broader new strategies remain separately proposed; an approved bug-fix milestone
does not authorize unrelated policy redesign.

## Validation and checkpoint

1. Build C and Rust against the same pinned engine/data/options. Record full
   source, binary, configuration and actual implementation/link identities.
2. Run the batch's controlled before/after/negative cases; obtain two identical
   corrected-C passes before accepting target-specific oracles. Compare Rust
   keys/messages/player state and complete native saves where the case declares
   them. Keep budgets, input origin and observable stop boundaries explicit.
3. Run appropriate existing smoke/class/config/engine/kernel regressions and
   no-C-Borg/FFI checks; retain every failure and declared exception. Native
   death-log date-line qualification does not relax gameplay matching.
4. Store new oracles under a distinct C-milestone/commit identity, for example
   harness/reference/better-borg-c/<milestone-or-full-sha>/. Preserve M4/M5 and
   previous target references and frozen death identities. A new target does
   not inherit old coverage percentages or universal-correctness claims.
5. Publish a bounded adoption receipt and checkpoint naming the accepted C
   milestone, effective Rust source, each fix/disposition, executed checks,
   failures and limits. Update docs/adoption/status.json only with actual pins
   and state. Milestone review is explicit; do not add an unrequested external
   review gate to every individual fix or infer acceptance without evidence.

Crash and blatant-play-blocker corrections remain allowed under the owner
policy. A new urgent defect may justify a narrow isolated fix with evidence
and coordination while a C milestone is pending; it does not authorize a
speculative backlog import. Keep such exceptions explicit and reconcile them
with the next accepted C milestone. Preserve original historical evidence.
