## 2026-11-19: a script to re-record smoke-borg-rs (#636)

Owner: changing a shipped value in `harness/borg.txt` should be "edit,
commit, run one script".

- **`scripts/rerecord-smoke-borg-rs.sh`:** on a clean, committed tree it
  builds, plays the smoke suite on the default core, compares with the old
  reference, re-records through `harness/make-reference.sh` and commits the
  reference on its own (`--no-commit` stages it). It refuses a dirty tree or
  a `BORG_CORE` other than borg-rs, stops when the games are unchanged, and
  never pushes. AGENTS.md names it under "Which core a run plays".
- **Checked:** with `--no-commit` on main `4acf071f` (stale reference): 2/3
  before, 3/3 after, the same reference as #631 (#635) byte for byte.
- **Review of #633:** the script also refuses `BORG_CORE_OVERRIDES` (which
  would play `borg-core/borg.txt` and record another game), and builds the
  new reference beside the old one, swapping it in only when complete. The
  review ran the commit mode and the "unchanged" exit.
- **Left:** The owner's worktree `.worktrees/borgtxt-values` (branch
  `task/borgtxt-values`) starts from this branch.
