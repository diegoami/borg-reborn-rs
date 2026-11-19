# P5 source-reference refresh inventory (#554)

Report command: `python3 scripts/refresh-sources.py`, run across all six
sources at accepted source `c9e58ca0` on 2026-11-18, before the borg-rs
update. Follow-up command: `python3 scripts/refresh-sources.py --update borg-rs`.
This is a source-reading inventory, not a gameplay validation report.

| Source | Retained/refreshed pin | Observed head | Report delta and disposition |
|---|---|---|---|
| angband | `42cd156ac9f95ddf4311e88dd7ad462292d6d1d6` | `574b618c3bfec595f3503a58f8e488b1c4ae3a46` | 10 commits; retain the engine contract at 42cd156ac. An engine upgrade is a separate baseline change, unrelated to historical receipt metadata. |
| borg-rs | `8c5d000a58e9ddca1f1954ab9552e505856b313f` | `8c5d000a58e9ddca1f1954ab9552e505856b313f` | 59 commits from the old reference pin 4ed3263b; two copied documentation files changed. Refresh to 8c5d000a, already used by the vendored core; no core-source update. |
| better-borg-c | `46037038cb0ce3a248ea22c06f02454818dbe4b4` | `b0bacee0697b5ca5ca2706d9335e24454bbdb4e9` | 178 commits; 44 added and 2 changed copied death files, cases D0020–D0030; tags v8/v9/v9.1/v10–v14. Retain the reference pin and immutable historical case inventory; no control-policy carry or dossier import in P5. |
| neo-angband-tracker | `c2ddf2ca045d0ceade7a915a8f7a0ab2a427d4d3` | `d649969591c9b9cd3ad423a49f44c142a05339dc` | 19 commits; one copied C-Borg document changed. Retain the manual/specification pin; receipt work does not adopt new engine or C-Borg rules. |
| borg-zoo | `7b373a702e7b9bb491ba468d97d34cf99ec3df2d` | `945126b319b96aa630d63675f135359aa903c61c` | 12 commits since main’s already accepted 7b373a70 pin; two catalogue files changed. Preserve main’s pin and source-qualified suite identities; no pilot execution or newer measurement baseline in P5. |
| borg-runner | `e55ac6f384f2e19c0b148c2b1d2ac932fbec3d83` | `e55ac6f384f2e19c0b148c2b1d2ac932fbec3d83` | No new commit or copied material. Existing pin remains current. |

Only the borg-rs read/reference pin moves in this dedicated PR. Its copied
FORK-CORRECTIONS and adoption/status blobs are exact donor bytes; docs/deaths
did not change across that donor interval and the generated death index stays
byte-identical. The refreshed ledger distinguishes an unaccepted, default-off
direct-buy proposal from the empty-light-slot Recall correction and donor
adoption history. Copying those descriptions authorizes no engine change.

The C report flags changed cited functions: `borg_danger`,
`borg_danger_one_kill`, `borg_flow_dark_interesting`, `borg_update_kill_old`.
They remain future citation/policy-review inputs; P5 does not change their
implementations or claim adoption. The report also lists 43 borg-rs, 37
better-borg-c, 43 tracker and 6 zoo issue updates; borg-runner has none.
Those counts are report-time inventory, not independent verification of each
issue or proof that a source has no other pending work.

This intentionally scoped refresh retains all other pins rather than moving
every source to its head. It records the previously missing all-source
report and the reason for each deferral for owner review. A later policy,
engine, reference-dossier or measurement refresh must use its own reviewed
PR; the P5 kickoff must not imply that the deferred updates were completed.
