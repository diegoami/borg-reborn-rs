# Painmsg-197 carry: verified product scenario

Consumer [287](https://github.com/diegoami/better-borg-c/issues/287),
canonical source
[197](https://github.com/diegoami/neo-angband-tracker/issues/197)
(painmsg-197; source PR458 `8a0c13f7`, evidence PR457). Fix applied as
`1e0031455` (`cherry-pick -x` of source
`8a0c13f7127642da8222c84380b4e6b8f9f96eff`, patch
`patches/0071-pain-message-allocation.patch`).

## Defect

`borg_get_parsed_pain()` allocated `strlen(pain) + 1` bytes for an entry
built as `' ' + text + '\0'`, so the parser ran out of room one byte short
and several monster status endings parsed without their last character,
never matching the game's message: "The kobold can move again.", "falls
under your spell!", "is no longer under your control." and "shimmers for a
moment." queued no Borg reaction. The correction allocates `strlen(pain) +
2`. Recognition and monster matching only; the Borg does not estimate
monster health from these lines.

## Pins and provenance

Before: merged main `53c17b06c` (snapshotted build `build/native-before`,
gate result `artifacts/native-before/result.json`, 965/965 + 5/5 at that
commit). The verifier proves that the source diff applied to the before
`borg-messages.c` equals the product module and that the applied commit is
the only `src`/`lib` difference from the before commit. Fixture: kit
`ba2167d5` at head `ffb74e54`, asserted byte-identical to the run pin.

## Verified results (`artifacts/pm197/scenarios`, 2026-11-08)

- `held`, `command`, `uncommand`, `shapefail` (the four status endings) —
  fail before, pass after: before 0/1 `recognised=0 matched_to_kobold=0`
  (the reaction queue stays empty), after 1/1 `recognised=1
  matched_to_kobold=1` (`PAIN:` reactions matched to the kobold, use=2).
- `stun`, `hurt` controls: 1/1 in both phases, logs byte-identical
  before/after (`f621a97d…`, `70ec0bdf…`).
- `puff` (outside this fix, no monster name): unrecognised in both phases —
  0/1 both, log byte-identical (`35457c9f…`).
- `result.json` records `verified: true` with all commands, pins and
  hashes.

Native gate at the applied tip: **965/965 unit + 5/5 frontend** (before
snapshot gate and product gate); tooling **129/129**. Prior-fixture matrix
not rerun (uncommitted runner; known gap recorded on checkpoint 1 and the
milestone 6 review). A harness correction is recorded in the verifier
commit: the first generated assertion expected the `puff` mode to pass;
like the source kit's contract it must keep failing (unrecognised in both
phases) — no other assertion changed.

## Scope and limits

Bounded message-reaction scenario driven through the real message API. It
proves recognition and monster matching for the four endings and the
unchanged controls; it does not measure a combat or survival effect.

## Exact next work

1. PR from this branch, agent evidence review plus required hosted checks,
   then normal merge.
2. Frozen v1 cohort at the merged tip against the 0070-after cohort at
   `53c17b06c`.
3. Receipts on issues 287, source 197 and checkpoint 1.

Spasticscore-147 was prepared for the same PR and deferred before the PR
opened: the source kit's fixture choreography was designed for the pristine
base where the #146 reverse flow is broken (the kit receipt states the
relation), and this product carries the 0064 reverse-flow correction in the
same function, so the unchanged fixture exercises nothing on the product
(zero searches in every mode, before logs byte-identical to after). The
consumer carry needs a product-side fixture that accounts for 0064; the
fix cherry-pick was reverted from this branch and consumer 288 records the
reason.
