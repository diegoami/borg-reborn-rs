# 2026-11-22 — #914: fast-fragile groups in contact skip the fast-fragile attack (shipped 0)

**Issue:** diegoami/borg-reborn-rs#914. **Branch:** `task/T914-ff-group-contact` (from `track-p`
`86843a79`, `track-p` merged in again, last at `182ca8b2`). Track P, P8: this PR merges into
`track-p` once the owner says its review is done (RULES 1a).

Evidence: [borg-cemetery study](https://github.com/diegoami/borg-cemetery/blob/claude/happy-brahmagupta-2jcsa3/studies/fastfrag-cave-spiders-2008390085/README.md) (seed 2008390085, borg-zoo#596).

## What was done

- New parameter `fast_fragile_contact_max` (compiled 0 = off, **shipped 0**; owner, 2026-11-23:
  of the cave-spider switches only #941's speed flag ships on; 1 is the value to try, a
  hypothesis). Above 0, when more than that many awake fast-fragile monsters are next to the
  Borg (`crowded`, read at `borg_fast_fragile.rs:675`):
  - the `continue` in `attack_on_sight`'s fast-fragile branch (`borg_fight_attack.rs:6197`)
    skips the fast-fragile attack on every marked target in that think, one outside contact
    included (`# Fast-fragile: N marked attackers in contact, caution decides (#914)`);
  - the early return in `heal_banned` (`borg_fast_fragile.rs:629-632`) allows healing against
    the group whether or not an escape is available (`# Healing allowed: ...`): a group
    exception to #842's owner rule.

  No hand-off to caution is guaranteed, despite the note's wording: other attack-on-sight
  entries still run, and `fight_before_flight` (`borg_caution.rs:4139-4156`) can take a
  lone-target or winning-fight action before `borg_caution`. Once caution is reached, it tries
  `borg_heal` (`borg_caution.rs:4380-4401`) before `borg_escape` (`:4711`), so its Phase Door is
  conditional. At 0, `crowded` returns before counting (`borg_fast_fragile.rs:676-678`), so both
  legacy paths are unchanged.
- `track-p` merged in four times, last at `182ca8b2` (`1f39aba7`). Conflicts in `params.rs`
  (track-p had moved the fast-fragile rows; its side kept, the new row added after
  `fast_fragile_count_dot`), `docs/CALIBRATION.md` and `harness/borg.txt` (both sides kept).
  `check-param-citations.py --fix` re-pointed `fast_fragile_contact_max`
  (`borg_fast_fragile.rs:675`, the read in `crowded`) and `attack_speed_fight_danger_tenths`
  (`borg_fight_attack.rs:6282`, shifted by this PR's six lines): both checked to be the line
  that reads the param. The registry holds 2506 parameters (0 errors).
- Review findings (GPT-6 in Codex, on `e5c91d72`), wording only: the handover, PR body,
  CALIBRATION section, param description, `borg.txt` comment and the code comments now say
  shipped 0 and describe the branch as above. No behaviour or shipped value changed.

## How it was checked

- Named cases on the merged head (`harness/run.sh`, `menu,exact` for 2008390085, `menu` for 224;
  `BORG_TXT_SET` for each arm):

  | case | contact_max 0 | contact_max 1 |
  |---|---|---|
  | zoo game 2008390085 (`7f8c6b80` + `9753a721`, `borg-stats-ban.txt`) | keys `1ef124668f17` (the zoo's), dies to a cave spider at 64,483 | keys `cb3fb751e8af`, alive at the cap, CL6, DL2: at 64,454 "2 marked attackers in contact, caution decides", then the control's emergency Phase Door and a CLW at danger 5 |
  | 2008390085, merged head, shipped | keys `8bd183c22e40`, alive at the cap, CL6, DL2 | same keys; no #914 note |
  | 2008390085, merged head, stats-ban (`fast_fragile_by_stats=1`) | same keys as shipped | same keys; no #914 note |
  | 224 Human Warrior, 144,000 turns, shipped | keys `eef241f5aa0b`, alive at the cap, CL11, DL6 | same keys; no #914 note |
  | smoke (default core) | `smoke-borg-rs` | 3/3 identical to `smoke-borg-rs`: nothing re-recorded |

  These replays ran on the merged tree at track-p `f8725bc0`; at `e5c91d72` (track-p
  `182ca8b2`, shipped 0) smoke was 4/4 identical to track-p's reference and
  `test_stair_handoff` passed.

  On the merged head 2008390085 never meets the cave spiders, and `by_stats = 1` changes no key
  there either. Every fast-fragile note in these games (8 crows, 4 wild cats) reads "0 others in view". The
  same keys at `b784409e` and `f8725bc0`; on the first merge (`e190e913`) the same holds with
  other keys.
- `cargo fmt --check -p borg-reborn-rs`, `cargo clippy --workspace --all-targets -- -D warnings`,
  `cargo test --workspace --exclude borg-core`, `cargo test --release -p borg-core`,
  `scripts/tests`, `tools/tests`, `check-param-citations.py` (0 errors), `borg-core-vendor.py
  check`, `engine/tests` (93 of 97 OK, `test_stair_handoff` included and unpinned for this switch; the
  four `test_shopping` errors are environmental: they clear `BORG_CEMETERY_DIR` and fall back to
  the sibling borg-cemetery, which is on another branch).
- The harness ran with `BORG_CEMETERY_DIR` at a borg-cemetery clone at `death-tools-pin.txt`.

## What was found

- **Shipped 0, it never acts. At 1 with the other shipped settings it can act, but rarely.**
  With `fast_fragile_by_stats = 0`,
  `classify` marks only the scary list's `fast-fragile` entries, the crow and the wild cat
  (`harness/scary_monsters.txt:130-131`). Neither declares `friends`
  (`engine/angband/lib/gamedata/monster.txt:1133-1164`; the cave spider has
  `friends:100:2d8:Same`, `:1130`). So `contact_max = 1` acts only when two of them are awake
  next to the Borg at once; independent spawns, a mixed crow and wild cat pair included, can
  still be adjacent. No replayed game met that. In borg-zoo#652's pilot (classes-18)
  `stats-contact`, `stats` and `ctl` played identically in 18 of 18 games, even with
  `by_stats = 1`; the small and medium series have no isolated contact arm.
- The 2008390085 case shows only that the switch can change that game (on the incident build
  with stats-ban settings), not a survival effect for crows or wild cats. The value 1 is a
  hypothesis.
- `fast_fragile_group_max` (in view) is unchanged; counting the race's expected group (FRIENDS)
  is not done.

## What is left / owner

- **Owner:** 0 is shipped (2026-11-23); 1 is the value to try. Before enabling 1:
  - the heal-ban lift is a group exception to #842's owner rule (healing allowed against two
    crows or wild cats in contact, escape or not);
  - no test covers the combined policy (the fast-fragile skip, the later attack entries,
    `fight_before_flight`, caution's heal-before-escape order); the unit test checks only
    `crowded_by`'s strict "more than". A path-level case would show which action actually
    follows. Not done: a code change, for the owner.
- borg-zoo#652's small and medium runs (short list) have no `stats-contact` arm; their `all` arm
  has `contact_max = 1` with `by_stats = 0`, mixed with four other switches. The zoo still has to
  measure the switch alone (RULES 14, 20).
- Not checked: the reborn core's smoke (`smoke-rust`; that core reads neither `borg.txt` nor
  `borg-core/`).
