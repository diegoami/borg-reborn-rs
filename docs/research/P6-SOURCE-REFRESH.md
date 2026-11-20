# P6 startup source inventory (#706)

## Corrected observation: 2026-11-20

This inventory supersedes the inaccurate 2026-11-19 observation at task
head `071b9c88`, identified by #708 review 7030550773. In particular,
the earlier “borg-rs has no newer commit” claim was false: the reviewer
identified commits already available before that task commit. The old zoo
head was also behind. Those are not merely later source advances.

Reran `python3 scripts/refresh-sources.py` across all six sources, then
repeated `python3 scripts/refresh-sources.py --no-fetch` after fetching
borg-rs directly from `https://github.com/diegoami/borg-rs.git` into a fresh
task-private clone. The first retry reported a borg-rs unpack failure and
was not used as proof of freshness. The final report has no problem lines.
Other sources were fetched by the all-source report. Observed refs below
are fixed report-time snapshots, not a promise that remote heads stay fixed.

Accepted task baseline: `324697eda0c74ac0f462cfa78053d66dc97b66dc`.
The original `--no-fetch --update borg-rs` refreshed only its reading date
to 2026-11-19. No new --update is claimed for this corrected observation:
all commit pins and copied bytes are intentionally retained.

| Source | Retained pin | Observed head | Disposition |
| --- | --- | --- | --- |
| angband | `42cd156ac9f95ddf4311e88dd7ad462292d6d1d6` | `574b618c3bfec595f3503a58f8e488b1c4ae3a46` | 10 commits. Retain this task’s accepted engine baseline; adopting a new engine is separate work. |
| borg-rs | `8c5d000a58e9ddca1f1954ab9552e505856b313f` | `d75e17a5ae743114c9b745934d36721800216f45` | 7 commits; 2 changed copied files (FORK-CORRECTIONS.md and adoption/status.json). Retain 8c5d000a and its exact historical copies deliberately; newer adoption documentation and core changes are not adopted in this correction. See explicit core deferral below. |
| better-borg-c | `46037038cb0ce3a248ea22c06f02454818dbe4b4` | `cc4c3e43aa0a863ad2fc4adab8f930277d181900` | 201 commits; 44 added and 2 changed copied files, D0020–D0030; tags v8 through v15 including v9.1. All 11 new report.md files were read and say unassessed. Retain historical copies; no inferred enchant/decurse defect or control-policy adoption. |
| neo-angband-tracker | `c2ddf2ca045d0ceade7a915a8f7a0ab2a427d4d3` | `dbde80762dd45857c7840f1af9df1bd39930d4ab` | 21 commits; BORG_SPEC.md changed among the three copied C-Borg documents. Retain existing manuals/specification for neutral extraction; no new mechanics adoption. |
| borg-zoo | `7b373a702e7b9bb491ba468d97d34cf99ec3df2d` | `b4d9d6d309c81643f09de1379ceb5ffef89c8ca0` | 417 commits; one catalogue addition and four changes. Retain accepted catalogue/measurement inputs; experiments and specimen/catalogue updates are separate work. |
| borg-runner | `e55ac6f384f2e19c0b148c2b1d2ac932fbec3d83` | `e55ac6f384f2e19c0b148c2b1d2ac932fbec3d83` | No new commits or updated issues. Existing pin remains current at this observation. |

## Explicit deferrals and scope

| Deferred adoption | Observed change | Reason and next gate |
| --- | --- | --- |
| borg-core re-vendor | One donor implementation commit, `5e6cd07565d145d8d5044d4c31536318ba38f785`, changes six mapped Rust files: borg_caution, borg_danger, borg_escape, borg_fight_defend, borg_messages and borg_trait. Upstream also advances its engine to 574b618c. | Separate behavior-changing carry with provenance/conflict audit and exact-source compatibility evidence. Do not silently mix these changes into the neutral #680 extraction. #680 must start from the owner-accepted receiving source and separately declare any required main synchronization. |
| borg-rs read/reference refresh | New adoption/status.json c-v15r entry and phase; FORK-CORRECTIONS.md adds rubble-boundary carry documentation. | Retain reference copies as exact bytes of retained 8c5d000a. A future dedicated pin/copy PR must adopt the newer documents together and describe its relationship to the core/engine carry. |
| Other moving sources | C reports/tags, tracker specification and zoo catalogue advances listed above. | No bulk historical-dossier rewrite, new control-policy adoption, engine adoption or population run in this task. |

Compared `borg-rs/src/borg_item_enchant.rs` and `borg_item_decurse.rs`
between retained 8c5d000a and observed d75e17a5: both blobs are unchanged.
Thus #680’s classification scope remains applicable; this is not a claim
that the rest of the donor core is unchanged or already adopted.

“Copies/index unchanged” means the actual receiving files remain byte-identical
to accepted task baseline 324697ed. It does **not** mean upstream has no
changed copies: both tracked donor documents changed, and are deferred.

The report also flags changed cited C functions `borg_danger`,
`borg_danger_one_kill`, `borg_flow_dark_interesting` and
`borg_update_kill_old`; none is part of the enchant/decurse extraction.
Updated-issue counts in the corrected report: borg-rs 2 (since retained
read_on 2026-11-19), better-borg-c 40, tracker 47, zoo 109 and runner 0.
These are report-time inventories, not independent reviews of every issue.
Upstream borg-rs #110 (two further C fixes) and #111 (Necromancer parity
divergence) are additional carry-review inputs, not results adopted here.

Synchronization fast-forwards this task from track-p 9e0402d4 to accepted
P5 promotion 324697ed. All intervening main work is preserved as accepted
upstream work. No implementation, engine, harness, tool/fixture pin,
reference-copy byte or retained-evidence byte is edited by this task.
This historical task baseline is not a claim to include later main changes.
P6 kickoff and implementation remain separate reviewed PRs into track-p.
No gameplay experiment or improvement claim is made.
