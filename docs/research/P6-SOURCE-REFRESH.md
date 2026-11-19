# P6 startup source inventory (#706)

Read-only all-six report: `python3 scripts/refresh-sources.py`, 2026-11-19,
at accepted main `324697eda0c74ac0f462cfa78053d66dc97b66dc`.
Dedicated update: `python3 scripts/refresh-sources.py --no-fetch --update borg-rs`.
The latter used a task-private donor clone, so shared vendor checkouts were not moved.

| Source | Retained pin | Observed head | Disposition |
| --- | --- | --- | --- |
| angband | `42cd156ac9f95ddf4311e88dd7ad462292d6d1d6` | `574b618c3bfec595f3503a58f8e488b1c4ae3a46` | 10 commits. Retain the game contract; an engine update is a separate baseline change. |
| borg-rs | `8c5d000a58e9ddca1f1954ab9552e505856b313f` | `8c5d000a58e9ddca1f1954ab9552e505856b313f` | No new commits, changed copies or updated issues. Ran --no-fetch --update borg-rs after the fetched all-source report; only read_on changes to 2026-11-19. Copies and index remain identical. |
| better-borg-c | `46037038cb0ce3a248ea22c06f02454818dbe4b4` | `450f55fcef75090ad6afb259e0cbb5c7983d4dec` | 197 commits; 44 added and 2 changed copied files, D0020–D0030; tags v8 through v15 (including v9.1). Read all 11 new report.md files: all explicitly unassessed. Retain historical copies; no inferred enchant/decurse defect or control-policy adoption. |
| neo-angband-tracker | `c2ddf2ca045d0ceade7a915a8f7a0ab2a427d4d3` | `dbde80762dd45857c7840f1af9df1bd39930d4ab` | 21 commits; one C-Borg document changed. Retain the existing manuals/specification contract for neutral extraction; no new mechanics adoption. |
| borg-zoo | `7b373a702e7b9bb491ba468d97d34cf99ec3df2d` | `3561bc567cb89799d5201703271c202acf0d71a0` | 374 commits; one catalogue addition and four changes. Preserve accepted measurement inputs; experiments, catalogue refresh and specimen changes are separate work. |
| borg-runner | `e55ac6f384f2e19c0b148c2b1d2ac932fbec3d83` | `e55ac6f384f2e19c0b148c2b1d2ac932fbec3d83` | No new commits or updated issues. Existing pin remains current. |

The report also flags changed cited C functions `borg_danger`,
`borg_danger_one_kill`, `borg_flow_dark_interesting` and
`borg_update_kill_old`; none is part of the enchant/decurse extraction.
Updated-issue counts were borg-rs 0, better-borg-c 40, tracker 47, zoo 94
and runner 0. These are report-time inventories, not independent reviews
of every issue or claims of source correctness.

The donor itself has no new core source. P6 #680 classifies the receiving
`borg_item_enchant.rs` and `borg_item_decurse.rs` at the accepted source,
preserving existing configuration and separating policy from mechanics/UI.
The report is a reading inventory, not a gameplay experiment or an improvement claim.

Synchronization fast-forwards the task from track-p `9e0402d4` to the
owner-accepted P5 promotion `324697ed`. All intervening main changes are
preserved as accepted upstream work, including its current runtime settings,
smoke references and other-track changes. No implementation, engine, harness,
tool/fixture pin, reference-copy byte or retained-evidence byte is edited here.
P6 kickoff and implementation remain separate reviewed PRs into track-p.
