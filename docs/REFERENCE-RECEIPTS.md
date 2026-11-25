# Reference receipts for repeated evidence (#375 / P5)

## Decision

Keep full, self-contained qualified packages. Do not implement reference-only
dossiers in the historical importer or introduce a second registrar here.
The pinned native-tool API has no payload-reference resolver or verified
dependency-closure contract. A package link is useful provenance, but cannot
replace payload bytes under the current import/verification contract.

For future unchanged controls, use the already accepted `KNOWN` reconciliation
rule (#464). Routine `KNOWN` controls need no new record/package or additional
receipt-retention/publication step beyond the existing CI workflow. When a
specific validation or investigation claim depends on retaining a run, retain
its actual receipts and the payloads needed for that claim with its case's
Cemetery evidence bundle. The existing
[inventory-parameters-433 bundle](https://github.com/diegoami/borg-cemetery/tree/a71dba05bc9867ddac3f566cd8eb8afcadbbfe2b/validation/borg-reborn-rs/inventory-parameters-433)
is an example location, not a requirement to publish every control. Reusing a
verified case never makes another build's metadata describe the new run.
Existing packages, original records, maps and legacy inventory stay unchanged.
No format, pin, index, exporter or runtime changes are delivered by this design.

## Contracts inspected

The source snapshot inspected for this design was
`f357404ea30177887f020a42b0faeeb794c61413`, after #579. At that historical
snapshot, native tools used Cemetery
`09225018dd5f2d4336a16b29478793d902c70dc3` (API `1`). That is inspection
provenance, not the current execution pin. The authoritative current commit
is recorded once in [death-tools-pin.txt](../death-tools-pin.txt); subsequent
adoptions do not relabel this historical snapshot.

| Contract | Inspected behavior and consequence |
|---|---|
| `tools/import-deaths.py:package` | Requires local regular files, rejects symlinks, checks required payloads and declared file hashes. A qualified ID hashes the original record plus the full payload manifest. Re-import verifies exact inventory and bytes. An omitted payload/reference manifest cannot stand in for the original bytes. |
| `.source-record.json` and `import.json` | Preserve the original record and the complete fingerprint. Different source/build/config/provenance or payload bytes intentionally produce another qualified package; family/result equality is not package equality. |
| `import_packages` / `check_registry` | Never overwrite differing evidence; publish under the owned lock; delegate index generation. The checker protects the legacy inventory and verifies every qualified package. Canonical Cemetery allocation is refused at this boundary. |
| Cemetery `death_registry.py:aggregate` | Reads materialized records/assessments. Counts package directories separately from distinct starts and deduplicates retained death attempts by canonical encounter family plus result/key hash. Reference links must not redefine those keys or inflate counts. |
| Cemetery `reconcile-deaths.py` | Default `KNOWN` matching uses implementation, seed, race, class, starting-save hash and key hash, excluding build, borg.txt and flags. Missing start information falls back to exact matching; `--exact` retains family/key matching. This is a coverage rule, not a package or full-stream equivalence test. |
| Cemetery `export-death.py:export` | A complete existing family/key case can return `repeat`. That return value alone does not capture a new run's source/build/configuration or process history. For a claim-dependent retained run, preserve its own receipt; never relabel the old record. |

For source details see [the importer](../tools/import-deaths.py) and
[its contract](DOSSIER-IMPORT.md). The owned tools above are inspected at the
exact pin, not inferred from Cemetery's moving default branch.

## Why repeated results do not make interchangeable packages

Read-only inspection of [the P341 map](validation/P341-DOSSIER-MAP.json) verifies all 25
mapped packages and finds earlier qualified family/key matches for each of
its 13 receiving packages. The selected earlier matches all exist in source
`2f74e1c5`, before the fresh P341 evidence was added.

Across those 13 pairs, all build, summary and report files differ, as do 12
configuration files. Twelve `params-core.json` payloads are byte-identical to
the earlier matched payloads. Those twelve copies total 5,942,064 logical bytes
and share one Git blob, `ab757b31d52ebada5a3a0435f48888ac773f2244`.
The remaining [receiving package ending `6b4bb62c77f0`](deaths/D0000-q4a22a9426e911a40b94827ecdbeb7811683c3dfeb4a2ab10f2cc6b4bb62c77f0/record.json)
has no `params-core.json`: its config selects `borg_core=reborn` and it retains
`params.json` instead. Thus there are twelve core-parameter files, not thirteen.
All thirteen receiving directories together contain 7,113,140 logical bytes,
including their import metadata. These are uncompressed file-byte totals,
**not** Git object-store size or a measured storage-saving estimate: Git already
stores identical blob content once. Working-tree duplication still exists.

This is a package-format inspection, not new gameplay evidence or a new
independence count. Dropping a whole package because its key stream matches
would discard its distinct provenance and changed files. No retained artifact
is removed, merged, rewritten or reclassified by this assessment.

## Use existing coverage without losing the new attempt

| New run | Treatment with current contracts |
|---|---|
| Same start and keys; another source/build/config | Reconcile as `KNOWN` if the owner tool confirms it; no new record or extra retention is required for an ordinary control. If an explicit claim depends on retaining this run, retain its actual receipt/needed payloads with that case evidence and link the verified death package. Do not create a partial dossier. |
| Another key stream, or the same keys from another start | `UNRECORDED`: use the Cemetery-owned full evidence delivery/registration workflow. |
| Only family/result matches, or start/provenance is missing | Do not infer full package identity. Preserve the source evidence; let the pinned exact/fallback checks decide coverage. Mark unknown receipt fields explicitly. |
| Same keys but messages/state/trace/config/summary differ | A `KNOWN` result does not establish those channels' equality. If a claim relies on them, retain the required differing bytes and report the actual comparison; never substitute the linked package's payloads. This adds no blanket retention rule for routine controls. |
| Failed run/export, retry, replay or branch | Keep its process receipt and lineage/status. A reference cannot turn failure into success, supply missing evidence or create another independent life. |

For a run selected for claim-dependent retention, its receipt must retain the
actual source commit/hash/dirty state and diff,
build identity/binary/toolchain, start/save identity, argv/environment/profile/
parameters/flags and configuration hashes, per-channel hashes and explicit
missing channels, process exit/completion status and attempt/replay lineage.
Store exact config/build/summary/trace bytes wherever they differ, with the
run's original record if one was produced. Preserve all captured attempt
metadata even when the catalogue deduplicates its family/key accounting.
Use [the capture contract](VALIDATION-RECEIPTS.md); distinguish actual capture
from reconstruction. These are provenance requirements for retained evidence,
not instructions to publish receipts for all `KNOWN` controls. A metadata link
is not a new death-record schema.

## Requirements for any future payload-reference format

The following is a compatibility design for the Cemetery owner, **not an
implemented format**. Until an owned versioned contract and its validation
exist, retain full packages. This task changes no external repository.

1. **Target identity:** reference an already verified full qualified package
   by repository, immutable revision, full qualified ID/path, original-record
   SHA-256 and import-receipt/complete-payload-manifest hash. A numeric prefix,
   family/result pair, filename, URL to `main` or a shortened digest is not a
   sufficient target identity. The target retains its original provenance.
2. **New provenance:** retain the complete new run/attempt receipt above,
   independently of the referenced case. Its source/build/config/result hashes
   must not be copied from the target unless verified equal. Distinct physical
   attempts keep their actual receipts; existing catalogue counting stays
   unchanged and is never inferred from receipt count.
3. **Per-file resolution:** state every expected logical filename and SHA-256.
   Reference a target file only after verifying identical bytes. Keep any new
   or differing payload in full, including parameters, traces, config/build
   metadata and original records. Same keys do not authorize reuse of other
   channels. The resolved inventory must be complete; do not add symlinks or
   stubs to today's qualified package format.
4. **Closed retention:** for an initial design, allow references only to full
   packages, not other references. Verify the target and every resolved byte
   before publication and before successful validation. Missing/unreachable
   targets, wrong hashes, mutable locators, unknown formats, extra/missing
   files or cycles fail validation; never silently follow a newer target,
   skip a field or report success with an unresolved dependency. Retain the
   dependency closure under the owner's custody or materialize a full package
   before publication. An immutable URL alone is not a retention guarantee.
5. **Owner integration:** the Cemetery owner defines version/capability
   negotiation, resolver, publication lock/atomicity, verification, replay,
   index and reconciliation behavior together. Consumers see a verified
   materialized view; generic index/reconciliation code must not opportunistically
   dereference network URLs or count a link as another independent death.
   Do not reinterpret API `1` or `dossier-import@1` in place. Existing readers
   either receive a complete materialization or explicitly refuse the new
   format. Existing full packages remain valid and immutable.
6. **Access and rollback:** preserve payload ownership and access limits;
   referencing must not widen access to a private payload. Define garbage
   collection/pinning so targets cannot disappear while referenced. A failed
   publication exposes no incomplete case. A failed or unsupported reference
   path uses the existing full-package workflow before publication, rather
   than weakening an already published reference or validation result.

An owner API change would require a separately routed proposal and review in
the owning repository before implementation. No such API change is needed
for the current decision to keep full packages and use existing `KNOWN`
coverage. This source repository neither creates that resolver nor changes
canonical registration, allocation, counts or delivery.

## Validation and exit

The existing importer regressions cover exact re-import idempotence, distinct
provenance with unchanged independence counts, corruption/missing cases,
symlink rejection, no-overwrite/concurrency and refusal of the canonical root.
The pinned reconciliation regressions distinguish another build, another key
stream, another start, changed borg.txt and missing start information.

Any future owner implementation additionally needs fixtures for a valid
reference with a changed local payload, unchanged keys with a different
message/state/trace, missing/corrupt target, unknown version, mutable locator,
reference cycle, offline closure, denied access, concurrent publication and
target-retention failure. Validation must verify the complete new receipt and
resolved bytes without modifying the target. These are future acceptance
vectors, not claims that today's reader supports references.

For #375, the concrete outcome is **no reference-only implementation under
the present contract**. Existing full packages and all original bytes remain;
the owner can evaluate the bounded future design separately. P5's task ends
with this decision and proceeds through review before integration/promotion.
