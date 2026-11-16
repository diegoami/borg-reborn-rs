# Recorder v0.2.0 adoption validation (#351)

Canonical evidence: [current-source report](https://github.com/diegoami/borg-cemetery/blob/07ac44f9db3cbe280c3e802b76c7f8c29f525ce7/validation/borg-reborn-rs/recorder-v020/P351-VALIDATION.md),
[machine-readable receipts](https://github.com/diegoami/borg-cemetery/blob/07ac44f9db3cbe280c3e802b76c7f8c29f525ce7/validation/borg-reborn-rs/recorder-v020/P351-VALIDATION.json),
[first-epoch report](https://github.com/diegoami/borg-cemetery/blob/07ac44f9db3cbe280c3e802b76c7f8c29f525ce7/validation/borg-reborn-rs/recorder-v020/P351-PRE-REVERT-VALIDATION.md) and
[byte-exact migration manifest](https://github.com/diegoami/borg-cemetery/blob/07ac44f9db3cbe280c3e802b76c7f8c29f525ce7/validation/borg-reborn-rs/recorder-v020/migration.json).
Cemetery evidence commit: `07ac44f9db3cbe280c3e802b76c7f8c29f525ce7` (Cemetery PR11).
The manifest resolves historical source paths in the unchanged reports/maps.

The current recorder-off source hash is
`7d2af92011c8d5fa7d73cb2df1e5460048f2dd1f0987cc0143df26406a5a94f7`.
All 45 before/after play triples and deterministic summaries match; 242,641
trace rows are classified. Fresh/loaded v2 native capture/replay, final saves,
checkpoints, bot streams, timezone restoration and CLI verification pass.
Required Rust, native, tool and script checks pass; binding regeneration is a
CI check. The final ownership/base synchronization leaves gameplay source
identical to that measured build. CI reconciles against historical source
plus the pinned canonical Cemetery registry, never a temporary self-baseline.

All 54 new validation packets and eight reports were transferred byte-for-byte
from published source b5cbff1f to Cemetery (602 audited files). The first
#355-era epoch remains separately labelled; it is not restored-policy proof.
Raw recordings, logs and frozen executables remain outside Git as documented.
No strategy or population-performance improvement is claimed.

Schema v2 is incompatible with v1. Preserve old sessions and their matching
executables; no migration or GUI/save-menu coverage is claimed. This task
integrates into `track-p`; owner-selected milestone review gates `main`.
