# 2026-11-22 — #913: emergency Speed against faster monsters near

**Issue:** diegoami/borg-reborn-rs#913 (ship-blocker). **Branch:** `task/T913-emergency-speed`
(from `track-p` `86843a79`, `track-p` merged in at `d8416afe`). Track P, P8: this PR merges into
`track-p` once the owner says its review is done (RULES 1a).

Evidence: [borg-cemetery study](https://github.com/diegoami/borg-cemetery/blob/claude/happy-brahmagupta-2jcsa3/studies/fastfrag-cave-spiders-2008390085/README.md) (seed 2008390085, borg-zoo#596).

## What was done

New parameters `caution_emergency_speed_danger_percent` (compiled 0 = off, **shipped 50**, P8),
`caution_emergency_speed_radius` (2) and `caution_emergency_speed_margin` (5).
Above 0, `borg_caution` drinks a Potion of Speed (or casts Haste Self, at most
`caution_cornered_spell_fail` failure) right after its healing step, before the
swaps, the stair walks, the retreat and the escapes, fleeing or not. It does so
when the summed `borg_danger_one_kill` of the awake known monsters within the
radius that are faster than the Borg by the margin reaches that percent of the HP.
The faster test uses an observed haste (`forecast_monster_speed`). Note:
`# Emergency Speed: N faster monsters near, danger D at HP H (#913)`.

Shipped 50 (owner, 2026-11-22: each P8 fix ships on once the evidence supports it). 50 is the
share #747's stair test ships (`caution_desperate_fast_danger_percent = 50`). No measurement chose
it. 50, 2 and 5 are hypotheses until borg-zoo measures them (RULES 14).

**#913 depends on #941 (PR #947, `trait_timed_sync`, not merged into `track-p`).** With the old
stuck speed flag the Borg believes it is fast for the rest of the game after its first Potion of
Speed, and the rule's "already fast" test returns. Before a game's first Speed, the rule can act
on `track-p` alone. None of the named games below drinks that way.

## How it was checked

- Merge of `origin/track-p` (`d8416afe`): conflicts in `params.rs` and `docs/CALIBRATION.md`
  only (citation lines). Took `track-p`'s files, re-added this PR's three rows and its section,
  then `check-param-citations.py --fix` (65 CALIBRATION rows re-pointed; spot-checked) and the
  check again: 2,408 parameters, 0 errors.
- CI's commands on `3d853a03`: `cargo fmt --check -p borg-reborn-rs`, clippy `-D warnings`,
  `cargo test --workspace --exclude borg-core`, `cargo test --release -p borg-core` (692 passed),
  `scripts/tests` (61), `tools/tests` (96), `check-param-citations.py`,
  `borg-core-vendor.py check` (ok against borg-rs `8c5d000a`), `engine/tests` (97 run: only the
  four environmental `test_shopping` errors; they clear `BORG_CEMETERY_DIR` and fall back to the
  sibling checkout, which is on another branch), `test_stair_handoff` OK with no new pin.
- `scripts/rerecord-smoke-borg-rs.sh` at shipped 50: 3/3 identical to `smoke-borg-rs`, nothing
  re-recorded.
- Named games, #913 off (0) against on (50). Local builds: the merged head; a throwaway detached
  merge of the merged head with `origin/task/T941-fast-expiry` (`920f36a3`, ships
  `trait_timed_sync = 1`), never pushed; the zoo game on `7f8c6b80` + `b67a2db8` (#941) +
  `8c9f5b7a` (#913), the study's `runcell.sh` and `borg-stats-ban.txt` (`58b6d86c…`).

| game | #913 off | #913 on (50) |
|---|---|---|
| zoo game, all off / #913 only | keys `1ef124668f17`, dies at 64,483 (the zoo's) | same keys: the stuck flag stops it |
| zoo game, #941 on | keys `37fba328d0e5`, alive at the cap, CL6, DL2 | keys `b571aea7bb47`, alive at the cap, CL6, DL2; fires once |
| 2008390085, merged head + #947, shipped | keys `65578788480c`, alive, CL6, DL3 | same keys; never fires |
| 2008390085, merged head + #947, stats-ban | keys `65578788480c` (the ban never acts) | same keys |
| 224 Human Warrior, merged head + #947, 144,000 turns | keys `7d86e9aeac6c`, alive, CL10, DL5 | same keys |
| merged head alone: 2008390085 shipped / stats-ban, 224 | `a5ad4668ef99` / `a5ad4668ef99`, `d62e4c2e01a1`, all alive | same keys |

## What was found

- On the zoo game with #941 the rule fires at 76,970: a metallic green centipede (speed 120,
  `monster.txt:827-830`) appears adjacent, "1 faster monsters near, danger 24 at HP 39". The
  Borg drinks Speed, then reads Phase Door at 76,980. The #941-only game does the same two
  things in the other order: Phase Door at 76,970 (danger 877), then the phase rule's Speed
  boost at 76,980. Both live to the cap at HP 39/39. One change of order in one game; not a rescue.
- PR #947's seed 224 death (poison at 62,200 with `trait_timed_sync = 1`) does not happen on this
  merged tree: `track-p` has moved since. It lives to the cap, #913 off or on.
- borg-zoo#652's pilot (run 45033305701, classes-18): `sync-speed` (#941 + #913) is identical to
  ctl in 10 of 18, as `sync` alone. One death in every arm, the same start. The counts do not say
  whether `sync-speed` differs from `sync` in the other 8. Small and medium runs (45065386860,
  45065415272) are pending.

## What is left / owner

- borg-zoo#652's small and medium gates (sync against sync-speed is the #913 measure).
- If #947 is not merged, the shipped 50 still acts before a game's first Speed; no named game
  showed it.
- Owner: whether Speed before a Desperate stair walk should also need the walk to be long.
