# 2026-11-22 — #913: emergency Speed against faster monsters near

**Issue:** diegoami/borg-reborn-rs#913 (ship-blocker). **Branch:** `task/T913-emergency-speed`
(from `track-p` `86843a79`; `track-p` merged in at `d8416afe`, `497cd344` (#943), `f77bbe1f`
(#890), `4f1bcae5` (#1146), `224e5272` (#1158) and `c6ed017b` (#944)). Track P, P8: this PR merges into `track-p` once
the owner says its review is done (RULES 1a).

Evidence: [borg-cemetery study](https://github.com/diegoami/borg-cemetery/blob/claude/happy-brahmagupta-2jcsa3/studies/fastfrag-cave-spiders-2008390085/README.md) (seed 2008390085, borg-zoo#596).

## What was done

New parameters `caution_emergency_speed_danger_percent` (compiled 0 = off, **shipped 0**),
`caution_emergency_speed_radius` (2) and `caution_emergency_speed_margin` (5).

**Shipped 0** (owner, 2026-11-23: "merge track-p here, put all inactive but the stale-speed flag";
of the four cave-spider PRs only #941's `trait_timed_sync` ships 1). Until then this PR shipped 50.
The candidate settings to try are **50 / 2 / 5**: 50 is the share #747's stair test ships
(`caution_desperate_fast_danger_percent = 50`), 2 and 5 are as compiled. No measurement chose them;
they are hypotheses until borg-zoo measures them (RULES 14).

Above 0, underground and unhasted, `borg_caution` drinks a Potion of Speed (or casts Haste Self, at
most `caution_cornered_spell_fail` failure). It does so when the summed one-action
`borg_danger_one_kill` at the Borg's tile of the awake known monsters within the radius that are
faster than the Borg by the margin reaches that percent of the HP
(`100 * danger >= max(HP, 1) * percent`). Note:
`# Emergency Speed: N faster monsters near, danger D at HP H (#913)`. At 0 the hook returns before
the radius and margin reads, the danger sum or any item or spell attempt.

- **Where it sits.** The hook comes before the equipment swap, the strategic stair walks, the
  retreat and the escapes, fleeing or not. It is not first in `borg_caution`: both healing paths
  can act before it; earlier in the same function the #1140 poison-race branch can step away (and
  #1140's carried poison cure be used) and `immobile_step_away` (#646, #713) can take the turn; and
  when not fleeing, `borg_defend` stays earlier where enabled. The code comment at the call now says
  so (comment only).
- **The speed test is a haste forecast.** `forecast_monster_speed`: `mspeed` plus 10 for an observed
  FAST. The engine's net speed also subtracts `2 * slow_level` for a slowed monster
  (`src/mon-move.c:1928-1938`); the rule omits that, so a slowed monster can count as faster.
- **Repeats.** A successful drink or cast sets the FAST timer, and the fast flag suppresses the next
  attempt while it lasts. A failed Haste Self cast sets nothing and leaves the Borg eligible again.
- **What depends on #941 (PR #947, `trait_timed_sync`).** The rule reads `st.borg.temp.fast` and
  never `trait_timed_sync`. With #941 off it can fire before the game's first Potion of Speed; after
  that Speed (from any rule) the stale flag stays set and the rule returns at its guard. Only a drink
  after a game's first Speed needs #941. Earlier text here said the rule "acts only with #941"; that
  was wrong and is corrected in the param description, `harness/borg.txt` and CALIBRATION.

## How it was checked

- Merges of `origin/track-p`. `d8416afe`: conflicts in `params.rs` and `docs/CALIBRATION.md`
  only (citation lines). Took `track-p`'s files, re-added this PR's three rows and its section,
  then `check-param-citations.py --fix` (65 CALIBRATION rows re-pointed; spot-checked). `497cd344`
  (#943): one `CALIBRATION.md` conflict, both sections kept. `f77bbe1f` (#890, #1030) and
  `4f1bcae5` (#1146): citation-only conflicts, `track-p`'s side taken, then `--fix`. `224e5272`
  (#1158, the loop detector): one `params.rs` block, line numbers only; `track-p`'s side taken,
  then `--fix` re-pointed `caution_flee_recall_danger_percent` to `borg_caution.rs:5054` and
  `caution_scary_ranged_no_stairs` to `:5007` (both read their param there). `c6ed017b` (#944,
  #916's retreat veto, which adds lines to `borg_caution.rs` but nothing between the healing paths
  and this hook): `params.rs` citation lines and one block where this PR's three rows were re-added
  to `track-p`'s side; `CALIBRATION.md` citation rows and the adjacent #913 and #916 sections, both
  kept; `--fix` re-pointed 71 rows, this PR's to `borg_caution.rs:6675`, `:6681` and `:6682`
  (each reads its param). The check after each: 0 errors (2,511 parameters at the end).
- On the final head: `cargo fmt --check -p borg-reborn-rs`, clippy `-D warnings`,
  `cargo test --workspace --exclude borg-core`, `cargo test --release -p borg-core`,
  `check-param-citations.py`, `scripts/tests`, the engine build and `test_stair_handoff`
  (numbers in the PR body).
- Smoke at the then-shipped 50: `scripts/rerecord-smoke-borg-rs.sh` on `3d853a03`, `497cd344` and
  `f77bbe1f`, 3/3 identical to `smoke-borg-rs` each time; at shipped 0 on `c214466c`, 4/4 identical
  to `track-p`'s reference. Nothing re-recorded.
- Named games, #913 off (0) against on (50). Local builds: the head `f77bbe1f`; a throwaway
  detached merge of it with `origin/task/T941-fast-expiry` (`920f36a3`, ships
  `trait_timed_sync = 1`), never pushed (the same runs on `d8416afe` and `497cd344` gave the same
  verdicts); the zoo game on `7f8c6b80` + `b67a2db8` (#941) + `8c9f5b7a` (#913), the study's
  `runcell.sh` and `borg-stats-ban.txt` (`58b6d86c…`).

| game | #913 off | #913 on (50) |
|---|---|---|
| zoo game, all off / #913 only | keys `1ef124668f17`, dies at 64,483 (the zoo's) | same keys: the stuck flag stops it |
| zoo game, #941 on | keys `37fba328d0e5`, alive at the cap, CL6, DL2 | keys `b571aea7bb47`, alive at the cap, CL6, DL2; fires once |
| 2008390085, head + #947, shipped | keys `c6fdf3cfe07c`, alive, CL6, DL2 | same keys; never fires |
| 2008390085, head + #947, stats-ban | keys `c6fdf3cfe07c` (the ban never acts) | same keys |
| 224 Human Warrior, head + #947, 144,000 turns | keys `67716d0d195d`, alive, CL11, DL5 | same keys |
| head alone: 2008390085 shipped / stats-ban, 224 | `8bd183c22e40` / `8bd183c22e40` (CL6, DL2), `eef241f5aa0b` (CL11, DL6), all alive | same keys |

## What was found

- On the zoo game with #941 the rule fires at 76,970: a metallic green centipede (speed 120,
  `monster.txt:827-830`) appears adjacent, "1 faster monsters near, danger 24 at HP 39". The
  Borg drinks Speed, then reads Phase Door at 76,980. The #941-only game does the same two
  things in the other order: Phase Door at 76,970 (danger 877), then the phase rule's Speed
  boost at 76,980. Both live to the cap at HP 39/39. One change of action order between surviving
  runs; not a rescue.
- PR #947's seed 224 death (poison at 62,200 with `trait_timed_sync = 1`) does not happen on this
  tree: `track-p` has moved since. It lives to the cap, #913 off or on.
- borg-zoo#652, `sync-speed` (#941 + #913 at 50/2/5) against `sync` (#941 alone): **0 of 18
  changed in the pilot, 1 of 90 in small, 21 of 270 in medium.** All 22 changed games reach the cap
  in both arms, and the death sets match. Max CL and max DL move either way, not always by one
  level: Elf Blackguard 2101309053 goes from max DL2 to DL4; Hobbit Blackguard 1981269294 from CL6
  to CL5. The zoo ran 50/2/5 together only: no comparison of thresholds, no measured change in
  mortality (RULES 14).
- Engine (574b618c): Potion of Speed `TIMED_INC:FAST`, 20+2d10 turns
  (`lib/gamedata/object.txt:3429-3439`); +10 player speed (`src/player-calcs.c:2176-2187`); a
  monster's FAST +10 and SLOW `-2 * level` (`src/mon-move.c:1928-1938`).

## What is left / owner

- A zoo run of other thresholds (and of radius and margin) if the switch is to ship on.
- The unit test (`speed_when_the_faster_monsters_reach_the_share_of_hp`) covers only the predicate's
  disabled case and basic arithmetic. It does not cover the monster filtering (awake, radius,
  margin), the call order in `borg_caution`, or exact equality at the HP threshold
  (`100 * danger == HP * percent`). For the owner: whether to add those tests.
- For the owner: whether the forecast should subtract a known SLOW, and whether a failed Haste Self
  cast should hold the rule back for a turn (both are behaviour changes, not made here).
- Owner: whether Speed before a Desperate stair walk should also need the walk to be long.
