# PR #308 review disposition

Review: https://github.com/diegoami/borg-reborn-rs/pull/308#issuecomment-6899904146
Reviewed head: `19f2025bb7e2f0a41481e20bc3a072897170ca28`.
Reviewer: Claude Opus 5.5 (Anthropic); implementation: OpenAI Codex family.
This is a task-level experimental POC review, not milestone acceptance.

| Finding | Disposition |
| --- | --- |
| 1. Concurrent dossier IDs | POC cases moved from D2446–D2448 to D2476–D2478 using PR308-DOSSIER-MAP.json. Source/build/family/attempt identities and recorded evidence hashes are unchanged and audited against the reviewed head. A bounded D2476–D2481 reservation is recorded on #318, above main/P/R maxima at reservation time. Revised-source controls occupy D2479–D2481. No other track's evidence was overwritten. The global cemetery-owned allocation mechanism remains #318. |
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
  longer-run policies. #317/#318 remain broader follow-ups.

The amended PR stays draft for owner testing/focused re-review. The owner
chooses/runs any additional reviewer; no reviewer was launched by this work.
