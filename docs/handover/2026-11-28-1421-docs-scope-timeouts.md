## Skip engine builds for docs-only pull requests (#1421)

The scope job checks changed paths once. Pull requests confined to root
Markdown and `docs/` skip both core jobs and recorder build steps while
keeping the required checks. `docs/deaths/` changes still run the core
checks and recorder. The full suite keeps its existing reborn scope.
The Rust job and push coverage stay in place. Core jobs, full-suite
shards and their required check jobs have 30-minute timeouts.
