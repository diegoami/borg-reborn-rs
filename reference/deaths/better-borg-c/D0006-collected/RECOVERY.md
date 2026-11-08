# D0006 evidence recovery

Actual original raw bundle:
`artifacts/death-workflow/collections/20261107T172008.791632Z/game-001-seed-1344203209/`.
Its recording manifest SHA256 is
`19667165b22d255c281adf27da902212d7c6e4c49833f480c4fb7d4aba8d83cc`.
It retains the original engine/data/initial user state/input/clock/preference
transcripts, start/recent/native fingerprint saves and complete original frames.
A seed alone is insufficient; keep this bundle and its actual C40 identity.

The durable feedback archive is primary
`artifacts/d0006-feedback-evidence.tar.gz`; its readback-verified SHA/member receipt
is `artifacts/d0006-feedback-archive.json` and is posted on investigation177 and
checkpoint1. Extract at the repository root to restore the relative artifacts
and curated dossier paths. The archive includes the original complete movie,
selected native inventory qualification, feedback/HP-viewer PTY evidence,
curated anchors and historical curation scripts. Those scripts are provenance;
do not rerun them over an already curated dossier as if creating a new death.

The already-qualified native inventory cache is under
`artifacts/death-workflow/inventory/f1bbd722aa6d5485d0bb85309cff3293ca46e908f83a03ec21ee08b67b4bc477/`.
Its full frames/observer provenance are also in the combined UI evidence archive
`artifacts/replay-ui-20261107-evidence.tar.gz`, SHA256
`c55c3a6abcf459aadd39297cabad581440be978ccebfa1cc3c61ab15dd3f1c33`.
The original record stays a C40 death even after observer/UI improvements.

```sh
./replay-borg.py docs/deaths/D0006-collected --view-only --from-turn 71135
./replay-borg.py docs/deaths/D0006-collected --verify-only
```

View-only reads original/qualified cached observations and never runs an engine.
Verify-only repeats the original native engine/start and compares the captured
streams/frames/native saves under the shared validation lock. It is not a new
independent life, and this dossier does not count diagnostic overlay runs as
unchanged-binary process attempts. See review-anchors.json and the qualified
inventory certificate for that deliberately separate observation extension.
