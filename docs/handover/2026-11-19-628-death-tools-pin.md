## 2026-11-19: the death-tools pin moves to the current Cemetery tools (#628)

- **`death-tools-pin.txt`:** `09225018` → borg-cemetery `a4c76ac5` (main at
  the merge of borg-cemetery#73). `scripts/native` is identical from
  `7095958d` to `a4c76ac5`; `API_VERSION` is still 1. The old pin is an
  ancestor, so this is a fast-forward of 11 native commits (canonical
  `D-<SHA-256>` IDs, zoo death bundles, the scary list in a dossier, zoo
  reruns).
- **What the move changed here:**
  - dossiers are now named `D-<SHA-256>` with a `death-id.json`: CI's five
    `D*-rust-*` globs are `D-*`;
  - `tools/tests/test_export_death.py` takes the Cemetery's own test edits
    (hash IDs instead of `D0001-c-seed101`);
  - `tools/import-deaths.py` keeps an exported `death-id.json` as
    `.source-death-id.json` in a `D0000-q` package. The Cemetery's index
    verifies `death-id.json` against `record.json`, and the import rewrites
    `record.json`'s `id`, so a verbatim copy failed the index. The receipt's
    `original_id` already names the same canonical ID. Its tests follow;
    `docs/DOSSIER-IMPORT.md` says so.
- **Checked locally:** `tools/tests` 96/96, `scripts/tests` 26/26,
  `import-deaths.py --check` (2572 dossiers); CI's core-death steps (seed
  1016: D-6d99a7ec…, reconcile KNOWN) and the reborn death, its files and
  `rerun-death.py` (seed 1020: D-9728091a…, KNOWN, keys reproduced), against
  a known-deaths registry built as CI builds it. A local `harness/run.sh`
  that dies now exits 0 and writes a `D-<hash>` dossier (issue step 3).
- **Not checked:** the attack-preference registry steps; CI's engine job
  stops earlier at the smoke diff while main's smoke reference is being
  re-recorded after `e0c4a527`, so CI does not reach the death steps until
  that fix is in.
- **Left:** `scripts/refresh-sources.py` indexes source registries with
  `D\d+`, so a hash-named dossier in a source would be skipped (not used for
  the Cemetery pin; for the next refresh). The shared sibling
  `../borg-cemetery` is switched by other sessions (it was on `review/pr62`
  mid-check): local checks need a checkout at the pin, via
  `BORG_CEMETERY_DIR`.
