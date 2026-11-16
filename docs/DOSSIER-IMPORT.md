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
owned toolkit. Existing differing data is never overwritten. Staging stays
private until all files are written. At publication, a new directory keeps
only the source directory's ordinary access bits allowed by directory creation
under the importing process's umask (no special permission bits). For example,
a 0755 source publishes as 0700 under umask 077; a 0777 source under umask
022 publishes as 0755. An explicitly permissive umask 000 still permits 0777
from a 0777 source. Files are also limited by source permissions and umask.
Generated metadata uses the source record's access limits. Re-import leaves
existing package permissions unchanged. This does not grant access to a private source.
A killed process can leave a `.import-*` directory; Git ignores historical
staging paths. Do not remove another process's active staging directory.

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
The inventory protects the complete record, including identity fields, first
result hashes and file-hash declarations. CI verifies the record's declared
payload files. Legacy analysis files such as `assessment.json` and `report.md`
are generally not declared: editing/deleting those files or adding extra
files is outside that legacy guarantee. Qualified packages, by contrast,
are immutable in every file. Keep new investigations in the cemetery-owned
workflow rather than treating undeclared legacy files as verified evidence.
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
