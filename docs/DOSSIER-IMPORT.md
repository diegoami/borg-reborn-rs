# Importing historical validation evidence

The native exporter and catalogue remain owned by borg-cemetery. Continue to
use its `tools/export-death.py` compatibility command and the documented
`BORG_DEATH_RECORDS_DIR` override for isolated CI or local validation runs.
Normal canonical exports still go through the cemetery's own registrar.

Numeric `D####` IDs from an isolated registry are provisional. They must not
be committed as new historical archive IDs: another track, task branch or CI
runner can independently allocate the same number. Per-track ranges alone do
not solve concurrent branches of the same track.

## Retain a completed validation package

Import each completed native dossier with:

```sh
python3 tools/import-deaths.py /path/to/isolated/D0001-rust-seed1011
python3 tools/import-deaths.py --check
```

The integration tool assigns `D0000-q` followed by a full SHA256 digest of the
original record bytes and the hashes of every payload file. Use the **full
ID or directory path**, not the `D0000` prefix, in links and discussions.
The namespace is deterministic: identical evidence gives identical IDs;
different payloads or provenance produce different IDs. No branch label,
clock, random number or caller-supplied counter is used.

The original `record.json` is retained byte for byte in `.source-record.json`.
The working record differs only in its display/storage `id`; source/build,
encounter family, attempts and file hashes remain unchanged. All other files,
including the original report and assessment, retain their exact bytes.
`import.json` records the original ID and complete package fingerprint. An
already imported package can be imported again without adding an attempt.
Concurrent import processes use the cemetery-owned registry lock, publish
complete directories atomically, and delegate index generation to that same
owned toolkit. Existing differing data is never overwritten.

These are immutable **historical validation packages**, not a new death
export implementation or a replacement for cemetery investigations. Do not
edit an imported package's analysis in place: investigate or annotate through
the cemetery-owned workflow while retaining the original evidence package.
The original ID in its preserved report is intentional; the receipt maps it
to its qualified storage ID. A corrected evidence package has a new fingerprint.

A package count is not an independent-run count. Two exports of the same
encounter/result can carry different provenance and both remain retained;
the owned index still groups independence and attempts by encounter family
and result hash. The tests check this explicitly.

## Legacy archive and CI

`docs/deaths/legacy-cases.json` freezes the pre-adoption archive from the
accepted #307 head. Those IDs and source identities are not rewritten.
The inventory protects identity fields, first result hashes and recorded
file-hash declarations, and CI verifies the retained file bytes too.
New evidence must have a verified qualified ID and receipt. CI rejects
unknown numeric IDs, missing legacy cases, identity/hash changes or damaged
qualified packages. A deliberate historical migration requires an explicit
reviewed inventory/map change, never a silent overwrite.

The ten #317 validation dossiers are qualified at adoption; their original
IDs and full qualified paths are listed in `P318-DOSSIER-MAP.json`. No legacy
P dossier or R implementation is imported or renamed. The same command can
qualify isolated R validation evidence when that track integrates it; the
current overlapping R numbers do not need a special range.

This tool refuses the canonical cemetery root. It is the local historical/CI
import boundary only. Any change to canonical allocation or delivery belongs
to the cemetery owner and its API, not to another exporter in this repository.
No external repository write or historical bulk migration is part of #318.
