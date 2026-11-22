# 2026-11-22 — #915: grouped monster messages read as their race

**Issue:** diegoami/borg-reborn-rs#915. **Branch:** `task/T915-grouped-messages` (from `track-p`
`86843a79`; owner, 2026-11-22: track P, switches shipped 0). Track P, P8 (P7 is reviewed on `promote/P7`, #939; `track-p` is open for P8):
this PR merges into `track-p` once the owner says its review is done.

Evidence: [borg-cemetery study](https://github.com/diegoami/borg-cemetery/blob/claude/happy-brahmagupta-2jcsa3/studies/fastfrag-cave-spiders-2008390085/README.md) (seed 2008390085, borg-zoo#596).

## What was done

New parameter `kill_grouped_names` (shipped 0). With it on, `guess_race_name`
reads "N <plural>" (`mon-msg.c:333-345`) as the race: the name as it stands
(monster.txt `plural:` is in the normal table), or `plural_aux` undone,
"-ses" → "-s" and then "-s" → "" (`mon-desc.c:27-36`). "N monsters" is an unseen
group. An unknown grouped name is noted (`# Grouped monster not known (...)`) and
still returns the ghost race, unused, as before. `best_normal` factors out the
existing scan, unchanged.

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

- On the zoo's game the switch alone changes no key: the lost messages decided nothing there.
- A grouped message is applied to one monster of the race near the spot, not to all N.

## What is left / owner

- Measure in the zoo; then decide whether to apply a grouped message to N monsters.
- Thresholds are hypotheses until borg-zoo measures them (RULES 14, 20).
