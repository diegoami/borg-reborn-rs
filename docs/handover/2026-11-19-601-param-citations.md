## 2026-11-19: parameter citations checked in CI (#601)

From the review of #600: many `params.rs` citations, and the
`docs/CALIBRATION.md` rows that repeat them, pointed at unrelated lines.
Merges inserted code above them, and later PRs copied the wrong numbers.

- **Done:** `scripts/check-param-citations.py` checks every parameter's
  cited line. It must mention `Param::<name>`, or sit in a function that
  does: a table read in one place, such as the attack preferences, which
  `borg_attack` reaches through `attack_preference::evaluate` (the one
  indirection the script knows). It also checks that every CALIBRATION
  row that cites `` `file:line` `` in the parameter's own file equals the
  registry. The P2 tables' bare "Source line" column is left out: those
  tables cite the `5d9abb8f` snapshot their headers name (review of #610). CI's rust job runs it,
  and `--fix` repairs both files. Tests: `scripts/tests/test_check_param_citations.py`.
- **Fixed:** 221 of 2,153 citations in `params.rs` (91 in `borg_caution.rs`,
  47 in `borg_fight_attack.rs`, 39 in `borg_escape.rs`), each now at its
  first read, and 72 CALIBRATION rows that disagreed with the registry. After
  merging main `2026-11-19` (#607 and others), `--fix` found 91 citations
  and 88 rows wrong again on the merged tree, which is the drift the check
  now stops. Against main `4acf071f` the net change is 88 citations and 18
  `` `file:line` `` rows; no bare row changes.
- **Brace matching** skips `/* */` comments across lines and raw strings
  (`r#"…"#`), as well as `//` comments and plain strings.
- **Not changed:** the historical archives (`docs/deaths/*/params-core.json`
  and `docs/HANDOVER.md`) keep the citations of their time.
- **For later PRs:** after inserting code above a read, run
  `python3 scripts/check-param-citations.py --fix`.
