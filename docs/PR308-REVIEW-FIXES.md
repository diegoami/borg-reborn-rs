# PR #308 review disposition

Review: https://github.com/diegoami/borg-reborn-rs/pull/308#issuecomment-6899904146
Reviewed head: `19f2025bb7e2f0a41481e20bc3a072897170ca28`.
Reviewer: Claude Opus 5.5 (Anthropic); implementation: OpenAI Codex family.
This is a task-level experimental POC review, not milestone acceptance.

| Finding | Disposition |
| --- | --- |
| 1. Concurrent dossier IDs | POC cases moved from D2446–D2448 to D2476–D2478 using PR308-DOSSIER-MAP.json. Source/build/family/attempt identities and recorded evidence hashes are unchanged and audited against the reviewed head. The bounded D2476–D2481 reservation was insufficient: concurrent task branches had already used those provisional IDs. The re-review recovery below qualifies all six packages without overwriting another lineage; the numeric map remains historical only. |
| 2. Target and sequencing | Owner explicitly chose main first, with P/R synchronization through their own PRs afterward. The decision is recorded on #308. Accepted current main (including #307) was merged. R implementation was not imported/reworked; future track sync conflicts are resolved in their own reviews. |
| 3. Shipped-file parse | A release-core test calls Plan::parse(include_str!(...)) on harness/plans/errands_plan.txt and checks every class/default at levels 1–50. It covers BOM/CRLF too. The independent Python grammar approximation was removed; a native check also loads the real shipped file. |
| 4. Command spelling | Require exact lowercase loop-levels before delegating argument checks. Slash, uppercase and colon forms are rejected by tests. Both endpoint orders are deliberately supported and documented. |
| 5. Missing class timing | Resolve the actual class during Borg initialization, before writing the preserved-plan acknowledgement, and repeat validation on core reinitialization. Native missing-class input exits 2 with an empty Borg-key log and no acknowledgement. This moves refusal before policy execution; it is not a claim of unsaved interactive-game recovery. |
| 6. Bad borg.txt | New strict POC reads treat only NotFound as absence; decoding/I/O errors carry the path and stop selection. The core-selection plan guard uses it too, so a broken file does not silently disable a file-selected plan. Pure and native tests cover absence, valid settings, invalid UTF-8 and directory/unreadable paths. Legacy callers and full startup/re-entry behaviour remain #317, not silently marked complete. |
| 7. Small items | Accept leading plan BOM/CRLF; check size before decoding a bounded read; match accepted main's zero/none budget semantics without trimming whitespace; format our repository-owned new module; move plans/README to harness/plans, separate from BORG_PROFILE settings. Donor files were not rustfmt-rewritten. |

## Validation and limits

- Formatting and Clippy pass. Workspace tests: 156 + 4 + 5; release core:
  150 passed, one pre-existing ignore. Vendoring check passes.
- Engine suite: 62 checks pass with one environment skip. All seven focused
  native POC checks pass, including actual class selection/level-up,
  initialization refusal and invalid-configuration no-fallback paths.
- Scripts/tools: 19/78 pass. Disabled-plan keys/messages/states match the
  control over 3,000 turns (seed 1010 Human Warrior); this is a bounded
  regression check, not a population improvement claim.
- Revised native source: `8b9642d658f1ae059fa9de1049cff1a6fa9480ded5b77fff65b0d5717799e0e8`,
  built at `e900d67013b863d32ddfde8802adb23d622b624f`. Standard no-plan CI
  controls are retained as D2479–D2481; raw output directories are recorded
  in their attempts. No missing-death guard was weakened.
- No new GUI drive, long cohort, recorder change or exact plan-checkpoint
  qualification. Configuration errors still terminate with exit 2; use the
  bounded POC driver before enabling edited settings in an unsaved live game.
- The diagnostic-only measured-harness refusal remains. Plan ending on a
  terminating outcome, first-endpoint restart on phase changes and current
  level (including drain) selection remain deliberate; #320 evaluates
  longer-run policies. #317/#318 are implemented on track-p and await owner-accepted promotion #331; #320 remains a broader follow-up.

The amended PR stays draft for owner testing/focused re-review. The owner
chooses/runs any additional reviewer; no reviewer was launched by this work.

## Re-review of 0030b101

Report: https://github.com/diegoami/borg-reborn-rs/pull/308#issuecomment-6901041092
Reviewer: Claude Opus 5.5 (Anthropic), reviewing the exact implementation pin.
The reviewer accepts the POC on its own and independently confirms its parser,
initialization, evidence preservation and green CI. No GUI or long-run test
was performed, and the disabled-plan comparison was not repeated by review.

1. **Dossier compatibility:** all six POC packages from 0030b101 are now
   content-qualified. PR308-DOSSIER-MAP.json records the full source revision,
   source path, qualified path and original-record hash for every package.
   The importer at track-p pin 69b99484 was run as a temporary external tool;
   no unaccepted Track P implementation is added to this PR. Each original
   record is preserved byte for byte in .source-record.json, every other
   payload is unchanged, and the current record differs only in its ID.
   The original 2455-case accepted legacy inventory remains untouched.
   #335 tracks this recovery; #331 must also account for newly accepted #332
   evidence before enabling its frozen-inventory gate on current main.
2. **Config-reader compatibility:** owner sequencing remains #308 first,
   followed by synchronization through Track P's own PRs. Do not merge the
   reviewer’s auto-merged shell.rs result unchanged. The #331 integration
   must remove this POC's read_borg_txt_setting/checked_borg_txt_setting,
   use its shared strict reader with lazy environment precedence, update
   the obsolete legacy-reader comment, and adjust focused tests to retain
   both the POC's no-silent-fallback contract and original-core environment
   isolation. Run both native configuration suites and importer checks on
   the combined source before the second promotion. #331 remains subject
   to its owner-acceptance gate; this review does not authorize promoting it.
3. **Reservation correction:** the earlier branch-max scan missed concurrent
   unintegrated task branches. The reservation is historical context, not
   an allocation guarantee. Qualified imports supersede it for these six
   packages; original records and earlier rename history remain auditable.

The rereview requires reconciliation before the second PR merges, not an
unreviewed Track P import into this main-targeted POC. The PR remains draft
for owner testing; no new reviewer is launched and no automatic merge occurs.
