# 2026-11-22 — #914: fast-fragile groups in contact go to caution

**Issue:** diegoami/borg-reborn-rs#914. **Branch:** `task/T914-ff-group-contact` (from `track-p`
`86843a79`; owner, 2026-11-22: track P, switches shipped 0). Track P, P8 (P7 is reviewed on `promote/P7`, #939; `track-p` is open for P8):
this PR merges into `track-p` once the owner says its review is done.

Evidence: [borg-cemetery study](https://github.com/diegoami/borg-cemetery/blob/claude/happy-brahmagupta-2jcsa3/studies/fastfrag-cave-spiders-2008390085/README.md) (seed 2008390085, borg-zoo#596).

## What was done

New parameter `fast_fragile_contact_max` (0 = off, shipped 0). Above 0, when
more than that many awake fast-fragile monsters are next to the Borg:
`attack_on_sight`'s fast-fragile branch skips the attack and caution decides
(`# Fast-fragile: N marked attackers in contact, caution decides (#914)`), and
`heal_banned` lifts the ban (`# Healing allowed: ...`).

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

- On the zoo's game, `fast_fragile_contact_max = 1` alone saves the rogue: it reaches the
  100,000-turn cap (CL6, DL2). The fork at 64,454 now goes to caution, as in the control.
- `fast_fragile_group_max` (in view) is unchanged; counting the race's expected group
  (FRIENDS) is not done.
- The suggested 1 is chosen from one case (RULES 14).

## What is left / owner

- Measure against stats-ban (borg-zoo request).
- Thresholds are hypotheses until borg-zoo measures them (RULES 14, 20).
