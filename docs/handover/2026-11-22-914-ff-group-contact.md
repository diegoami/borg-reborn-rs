# 2026-11-22 — #914: fast-fragile groups in contact go to caution

**Issue:** diegoami/borg-reborn-rs#914. **Branch:** `task/T914-ff-group-contact` (from `track-p`
`86843a79`, `track-p` merged in again, last at `f8725bc0`). Track P, P8: this PR merges into
`track-p` once the owner says its review is done (RULES 1a).

Evidence: [borg-cemetery study](https://github.com/diegoami/borg-cemetery/blob/claude/happy-brahmagupta-2jcsa3/studies/fastfrag-cave-spiders-2008390085/README.md) (seed 2008390085, borg-zoo#596).

## What was done

- New parameter `fast_fragile_contact_max` (compiled 0 = off, **shipped 1**, P8; owner
  2026-11-22: each P8 fix ships on once the evidence supports it). Above 0, when more than that
  many awake fast-fragile monsters are next to the Borg: `attack_on_sight`'s fast-fragile branch
  skips the attack and caution decides
  (`# Fast-fragile: N marked attackers in contact, caution decides (#914)`), and `heal_banned`
  lifts the ban (`# Healing allowed: ...`).
- `track-p` merged in. Conflicts in `params.rs` (track-p had moved the fast-fragile rows; its
  side kept, the new row added after `fast_fragile_count_dot`), `docs/CALIBRATION.md` and
  `harness/borg.txt` (both sides kept). `check-param-citations.py --fix` re-pointed
  `fast_fragile_contact_max` (`borg_fast_fragile.rs:614`, the read in `crowded`) and
  `attack_speed_fight_danger_tenths` (`borg_fight_attack.rs:5911`, shifted by this PR's six lines):
  both checked to be the line that reads the param.

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

- **On the shipped settings the switch can act, but rarely.** With `fast_fragile_by_stats = 0`,
  `classify` marks only the scary list's `fast-fragile` entries, the crow and the wild cat
  (`harness/scary_monsters.txt:130-131`). Neither has `friends`
  (`engine/angband/lib/gamedata/monster.txt:1133-1166`; the cave spider has `friends:100:2d8:Same`,
  `:1130`). So `contact_max = 1` acts only when two of them are awake next to the Borg at once.
  No replayed game met that. borg-zoo#652's pilot (classes-18) played `stats-contact` identical
  to `stats` in 18 of 18: it never acted there either, even with `by_stats = 1`.
- Shipping 1 is therefore close to inert in today's play. It is the fix ready for a group of
  marked attackers, and it matters once `fast_fragile_by_stats` ships.
- `fast_fragile_group_max` (in view) is unchanged; counting the race's expected group (FRIENDS)
  is not done.

## What is left / owner

- **Owner:** 1 is shipped from one case. In shipped play it acts only against two crows or wild
  cats in contact, where the heal ban (#842, the owner's "death sentence") then lifts and
  caution decides. No game measured that; if the owner wants the ban kept against two crows,
  ship 0 until `by_stats` ships.
- borg-zoo#652's small and medium runs (short list) have no `stats-contact` arm; their `all` arm
  has `contact_max = 1` with `by_stats = 0`, mixed with four other switches. The zoo still has to
  measure the switch alone (RULES 14, 20).
- Not checked: the reborn core's smoke (`smoke-rust`; that core reads neither `borg.txt` nor
  `borg-core/`).
