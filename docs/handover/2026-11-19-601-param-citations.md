## 2026-11-19: parameter citations checked in CI (#601)

From the review of #600: many `params.rs` citations, and the
`docs/CALIBRATION.md` rows that repeat them, pointed at unrelated lines.
Merges inserted code above them, and later PRs copied the wrong numbers.

- **Done:** `scripts/check-param-citations.py` checks every parameter's
  cited line. It must mention `Param::<name>`, or sit in a function that
  does: a table read in one place, such as the attack preferences, which
  `borg_attack` reaches through `attack_preference::evaluate` (the one
  indirection the script knows). It also checks that every CALIBRATION
  row that carries a citation, as `` `file:line` `` or in the P2 tables'
  bare "Source line" column, equals the registry. CI's rust job runs it,
  and `--fix` repairs both files. Tests: `scripts/tests/test_check_param_citations.py`.
- **Fixed:** 221 of 2,153 citations in `params.rs` (91 in `borg_caution.rs`,
  47 in `borg_fight_attack.rs`, 39 in `borg_escape.rs`), each now at its
  first read, and 72 CALIBRATION rows that disagreed with the registry. After
  merging main `2026-11-19` (#607 and others), `--fix` found 91 citations
  and 88 rows wrong again on the merged tree, which is the drift the check
  now stops.
- **Not changed:** the historical archives (`docs/deaths/*/params-core.json`
  and `docs/HANDOVER.md`) keep the citations of their time.
- **For later PRs:** after inserting code above a read, run
  `python3 scripts/check-param-citations.py --fix`.
