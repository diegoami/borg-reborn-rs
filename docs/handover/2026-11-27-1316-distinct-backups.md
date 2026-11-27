# Protection backups do not count against the Home backup limit

Issue #1316. Written on #1314 (#1310), which another session owned. That
stack landed in `track-p` through #1321, and this branch now merges
`track-p` and targets it.

The owner did not pick between the issue's options before the work
started. This implements option (c), the one the session recommended,
behind a flag: `town_home_backup_protections` (compiled default 0,
shipped 1).

## Done

- With the flag on, `backup_counts` keeps a further backup, beyond the
  regular `junk_gear_alternatives` count, when its known properties add a
  protection that neither its slot's kept gear nor the worn kit gives.
  The worn kit means every equipped slot, combined by
  `combined_properties`. A protection is any element's resistance, Free
  Action, See Invisible, Hold Life, protection from blindness or
  confusion, or telepathy. Such a backup uses no count. It stays only at
  Home, or in the pack while the remembered Home has a free stack: #1314's
  rule, unchanged.
- Other distinct properties (a slay, a stat, stealth) still need a count,
  as in #1314. With the flag 0, play is #1314's.
- The flag is read through `borg_town_live::home_backup_protections`,
  which also requires `gear_home_backups()`.
- Review of #1325: the worn kit is built from the equipment snapshot
  `gear_plan` takes before its trials. The trials leave the last candidate
  in the slot, so a trailing rNexus piece looked worn and covered itself.
  Native case `protection-backup` (`test_exploration_pack.py`), at one
  backup per slot:
  - the worn Free Action robe stays;
  - the rFire robe at Home is the regular backup;
  - the carried rNexus robe, the last candidate and the only rNexus
    source, is kept.

  Against the earlier line the case fails: the rNexus robe is released.
- Unit test `protection_backups_do_not_count_against_the_limit`:
  - at one backup per slot, rNexus and rPois boots both stay at Home
    beside Free Action boots, and plain boots leave;
  - rPois worn in another slot covers the rPois boots;
  - a carried extra still needs Home room.

  #1314's tests pass unchanged (they pass no kit).
- Two `borg_town_live.rs` citations move with the helper
  (`town_gear_home_backups`, `town_shop_overloaded`).

## Checks

See the PR. They are the CI feature build's gear and town native modules,
the CI cargo commands, the citation check, and the shipped smoke against
`smoke-borg-rs`.

## For the owner

- Option (c) is a choice made for you. Turn
  `borg_param_town_home_backup_protections` to 0 for #1314's rule.
- borg-zoo#816 compares 1 and 2 backups per slot on #1314's head. With
  this flag shipped, the `gear1` arm on this head is no longer plain
  #1314. Pin #816 on #1314's head, or add an arm for this flag.
- No zoo evidence for any value (RULES 14).
