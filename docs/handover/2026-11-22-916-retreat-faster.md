# 2026-11-22 — #916: no strategic retreat from a faster monster in contact

**Issue:** diegoami/borg-reborn-rs#916. **Branch:** `task/T916-retreat-faster` (from `track-p`
`86843a79`; owner, 2026-11-22: track P, switches shipped 0). Track P, P8 (P7 is reviewed on `promote/P7`, #939; `track-p` is open for P8):
this PR merges into `track-p` once the owner says its review is done.

Evidence: [borg-cemetery study](https://github.com/diegoami/borg-cemetery/blob/claude/happy-brahmagupta-2jcsa3/studies/fastfrag-cave-spiders-2008390085/README.md) (seed 2008390085, borg-zoo#596).

## What was done

New parameters `caution_retreat_from_faster` (shipped 0) and
`caution_retreat_faster_margin` (5). With the switch on, `retreat` in
`borg_caution` is refused:
- while an awake monster next to the Borg is faster by the margin (observed haste
  counted): `# No retreat: <race> in contact is faster by N (#916)`;
- toward a destination whose danger is still at or above the HP.

C's own "adjacent to a faster monster" test inside the scan has no effect (its
`break` leaves only the inner loop) and is kept as in C.

## How it was checked

All on `track-p` `86843a79` plus this branch:

- `cargo fmt --check -p borg-reborn-rs`, `cargo clippy --workspace --all-targets -- -D warnings`,
  `cargo test --workspace --exclude borg-core` (166 passed) and `cargo test --release -p borg-core`
  (647 passed), plus the new unit tests; `scripts/check-param-citations.py` (citations shifted by
  the insertion re-pointed with `--fix`).
- Smoke on the default core (borg-rs) with the switch at its shipped 0: **3/3 identical** to
  `harness/reference/smoke-borg-rs`, so the shipped play is unchanged. The suite ran with
  `BORG_CEMETERY_DIR` at a borg-cemetery checkout at `death-tools-pin.txt` (`46978936`), because
  the sibling checkout is on another branch.
- Seed 2008390085 (High-Elf Rogue) with the stats-ban arm settings on `track-p`: this game no
  longer dies at all on `track-p` (its 78 commits since `7f8c6b80` change it), switch off or on.
  So the rescue was checked on the zoo's own game instead.
- The zoo's game, on a local branch never pushed: `7f8c6b80` plus the five commits
  (#913, #914, #915, #916, #941). With every switch off it plays the zoo game key for key
  (keys.log `1ef12466…`, death at 64,483).

## What was found

- On the zoo's game the switch alone saves the rogue: it reaches the cap (CL7, max DL3).

## What is left / owner

- Measure in the zoo; whether the back-away step (`back_away`) needs the same test is open.
- Thresholds are hypotheses until borg-zoo measures them (RULES 14, 20).
