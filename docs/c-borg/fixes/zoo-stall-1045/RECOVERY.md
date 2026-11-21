# Evidence recovery

The adjacent archive-receipt.json identifies the persistent local ignored raw
archive and its full member manifest in artifacts/zoo1045/manifest.json.
SHA256 and all227 member hashes were verified by reading the completed gzip tar.
This archive is not claimed uploaded to GitHub.

`runs/` contains every raw result/trace/config/native save, including the invalid
zero-decision UI activation attempt. `builds/` retains immutable binaries,
build manifests, original source tarballs and diagnostic observer source.
`session/` retains the frozen save, exact experiment drivers and logs.

Use `sha256sum artifacts/zoo1045/evidence.tar.gz` and compare the receipt before
extracting into a fresh directory. Rebuild through the report's pinned harness
recipe for a fresh independent reproduction; the archived binaries/data are
historical evidence, with the original Linux build/runtime requirements.
Reconstruct a source/data export from its source.tar when inspecting a binary;
reproduce diagnostic instrumentation with diagnose.py rather than treating the
unmodified source archive as the complete instrumented compilation tree.

The frozen d0303959... save is a naturally reached M4 native state. Loaded-state
controls reset Borg memory and use native named-play activation; they are not
faithful full-memory continuations or new independent games. Named play redirects
autosaves to preserve the frozen input. Keep invalid activation separate.

No product fix is in this report. Follow source403 for correction, then consumer204
for a separately verified carry. Existing death records, old baselines and
borg-zoo results/catalogues remain unchanged.
