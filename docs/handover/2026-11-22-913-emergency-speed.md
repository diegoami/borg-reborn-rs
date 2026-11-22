# 2026-11-22 — #913: emergency Speed against faster monsters near

**Issue:** diegoami/borg-reborn-rs#913. **Branch:** `task/T913-emergency-speed` (from `track-p`
`86843a79`; owner, 2026-11-22: track P, switches shipped 0). Track P. P7 is closed for review
(#937): this PR targets `track-p` and merges after the promotion.

Evidence: [borg-cemetery study](https://github.com/diegoami/borg-cemetery/blob/claude/happy-brahmagupta-2jcsa3/studies/fastfrag-cave-spiders-2008390085/README.md) (seed 2008390085, borg-zoo#596).

## What was done

New parameters `caution_emergency_speed_danger_percent` (0 = off, shipped 0),
`caution_emergency_speed_radius` (2) and `caution_emergency_speed_margin` (5).
Above 0, `borg_caution` drinks a Potion of Speed (or casts Haste Self, at most
`caution_cornered_spell_fail` failure) right after its healing step, before the
swaps, the stair walks, the retreat and the escapes, fleeing or not. It does so
when the summed `borg_danger_one_kill` of the awake known monsters within the
radius that are faster than the Borg by the margin reaches that percent of the HP.
The faster test uses an observed haste (`forecast_monster_speed`). Note:
`# Emergency Speed: N faster monsters near, danger D at HP H (#913)`.

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

- **On the zoo's game the switch alone changes no key.** The rule returns at its "already
  fast" test from turn 56,229 to the end: the Borg's speed flag was stuck (#941). With #941's
  switch too, the rogue lives to the cap. But it is `borg_defend_aux_speed` that drinks first
  (64,344, against the shrieker); #913's rule fires later in that game (76,970: "1 faster
  monsters near, danger 24 at HP 39"). So #913 is only meaningful together with #941.
- 50, 2 and 5 are hypotheses (RULES 14).

## What is left / owner

- Measure with #941 on in both arms (borg-zoo request).
- Owner: whether Speed before a Desperate stair walk should also need the walk to be long.
- Thresholds are hypotheses until borg-zoo measures them (RULES 14, 20).
