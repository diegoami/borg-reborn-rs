# Carry-pipeline lessons learned

Date: 2026-11-07, after the twenty-five corrections merged as consumer issues
48 through 124 (patches 0010-0034). This document records operational lessons
from that work. It changes no gate, fixture, assertion, baseline or gameplay
behavior; each linked report remains the authoritative evidence for its carry.

## Treat verification tooling like the fix itself

The most repeated lesson of this wave is that a verifier is part of the
evidence, not a neutral observer. A verifier bug can reject a correct product
(opblock-324) or, worse, make an incorrect product look verified. Verifier
failures are therefore retained and diagnosed like native failures; they are
never "fixed" by weakening a fixture or assertion.

* [OPBLOCK-LEAK.md](OPBLOCK-LEAK.md): the first verifier expected two
  LeakSanitizer report blocks where one block grouped two allocations. The
  failed verifier run is preserved under `artifacts/o324/verifier-group-count-error`;
  only its grouping interpretation was corrected.

## Pristine expectations do not survive layered carries

When earlier corrections legitimately change a module, a later source
fixture's absolute numbers may describe a build that no longer exists here.
Assert the defect contract relationally or with stack-specific attribution,
and state the layered interaction instead of copying the source number.

* [BLIND-FRIGHTEN.md](BLIND-FRIGHTEN.md): the pinned control value `-5` was a
  pristine value. After the losdispel correction removed a leaked `-1`, the
  layered product measured `-4`; the adapted control derives its expectation
  from the production launch call, and the original red attempt is retained.
* [CALFREE-TOKENS.md](CALFREE-TOKENS.md): pristine evidence reported
  `160 bytes / 2 allocations` before and `80 / 1` after. Because opblock-324
  was already carried, the consumer before-build baseline is `80 / 1` and the
  fixed build reports no leaks. That interaction is asserted explicitly.
* [GLYPH-TRACKER.md](GLYPH-TRACKER.md): an unchanged source fixture can stop
  reproducing its original failure after an earlier fix changes eligibility.
  The adapted tracked scenario documents why and keeps its provenance header;
  the historical evidence remains valid at its recorded commits.

## Separate functional identity from ownership and memory safety

Several memory corrections leave functional logs byte-identical by design. A
passing functional fixture does not prove memory ownership.

* opblock-324 and calfree-323 both evaluate their formulas identically before
  and after; only stack-attributed LeakSanitizer allocations distinguish them.
* The glow overflow's normal fixture detects the count, but AddressSanitizer
  provides the actual out-of-bounds WRITE proof ([GLOW-OVERFLOW.md](GLOW-OVERFLOW.md)).
* Keep both kinds of assertion, and keep sanitizer exits (including nonzero
  leak exits) as evidence rather than suppressing them.

## Read sanitizer output in its own units

A "Direct leak of 160 byte(s) in 2 allocation(s)" line can be one report block
describing two allocations. Count what the assertion names: allocations,
report blocks, or matching stacks. Stack-specific attribution
(`in tokenize_math`, `in adjust_order_ops_depth_block`) proved more stable
than absolute totals as carries layered.

## Assert cross-carry interactions instead of silently accepting drift

The regression matrix keeps all prior fixture cases and explicitly asserts the
two known losdispel-326 transitions rather than either discarding those cases
or demanding their historical logs. This is the pattern for future
interactions: name the transition, its cause and the affected evidence, then
assert it. It keeps prior carries' evidence valid at their recorded commits
without freezing incorrect absolutes.

## Make merge and measurement fail together

During stale-target, `gh pr merge` returned HTTP 502 but the shell chain
continued, so the first measurement ran on the unmerged parent. That attempt
is preserved as `invalid-premerge-*` evidence and was never used as merge
evidence. Standing practice is now:

* chain merge, head fetch and a head-change check with `&&`;
* only prepare and measure after proving the expected merge head moved;
* retain any accidental parent-build measurement rather than deleting it
  ([STALE-TARGET.md](STALE-TARGET.md)).

## Recheck the source board between carries

New demonstrated corrections repeatedly preempted the planned low-priority
queue (text-287, stale-target-322, glow-311, light-238, surrounded-312,
home-blows-313, lunal-314). The queue is severity- and readiness-driven, not
first-in-first-out. A durable checkpoint with the exact next action prevents
long-postponed low-priority items (brand-240) from being lost.

## Retained worktrees need explicit baseline paths

A regression matrix generated in one task worktree broke when reused from the
next worktree because a baseline path pointed at `task/artifacts/...` rather
than the previous worktree's retained tree. Baseline locations should be
explicit (normally the prior task worktree or the primary checkout) and are
part of the matrix's reproducibility, not incidental state.

## Stage long builds and tolerate background limits

A background build wrapper can time out after the native gate but before ASan
configuration completes. Use separate logged stages, check retained logs
before resuming, and never report a wrapper timeout as a test result. Resume
only the unfinished build step under the shared validation lock.

## Expect the baseline itself to move

Independently merged work can change coverage: PR116 raised tooling tests from
29 to 41 without touching this queue. Investigate baseline-count changes
before quoting them; the stronger number is not automatically wrong, and the
old number is not a contract.

## Keep failure evidence and state its limits

Retained red attempts, invalid measurements and verifier failures are part of
the record. Conversely, identical frozen-seed traces are a narrow regression
result: they show no divergence on those starts and fixes, not a general
proof of unchanged gameplay, improved survival or a performance claim.

## Summary table

| Lesson | Evidence | Standing practice |
| --- | --- | --- |
| Verifiers need verification | OPBLOCK-LEAK | retain and diagnose verifier failures |
| Layered baselines shift absolutes | BLIND-FRIGHTEN, CALFREE-TOKENS | assert relational/stack contracts and interactions |
| Functional equality is not ownership | OPBLOCK-LEAK, CALFREE-TOKENS, GLOW-OVERFLOW | keep functional, count and sanitizer assertions separate |
| Sanitizer units matter | OPBLOCK-LEAK | count allocations vs report blocks vs stacks |
| Interactions must be explicit | LOSDISPEL, regression matrices | assert known transitions with causes |
| Merge and measurement must be atomic | STALE-TARGET | `&&` chain plus head-change check; retain accidents |
| Recheck source readiness | checkpoint1 queue | severity/readiness order over FIFO |
| Baselines are paths, not assumptions | regression matrices | name retained baseline trees explicitly |
| Long builds are staged | gate/ASan logs | resume unfinished stages; never treat timeouts as results |
| Baselines move independently | PR116 tooling 29 -> 41 | investigate count changes before quoting them |
| Failure evidence has value | all carry archives | retain red/invalid attempts and state limits |
