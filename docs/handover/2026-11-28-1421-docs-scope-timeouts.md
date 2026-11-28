## 2026-11-28: Skip engine builds for docs-only pull requests (#1421)

The scope job checks changed paths once. Pull requests confined to root
Markdown and `docs/` skip both core jobs and recorder build steps while
keeping the required checks. `docs/deaths/` changes still run the core
checks and recorder. Changes to `docs/ENGINE-READS.md` or `docs/examples/`
also run the core checks because engine tests read those files. The recorder
can skip those changes. The full suite keeps its existing reborn scope.
The Rust job keeps its existing coverage and timeout policy. Pushes run
the full workflow. Core jobs, full-suite
shards and their required check jobs have 30-minute timeouts.
