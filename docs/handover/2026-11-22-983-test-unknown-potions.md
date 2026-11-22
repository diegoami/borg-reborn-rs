## 2026-11-22: test an unknown potion when safe (#983)

**Found.** #983's Priest (seed 17626119, zoo run 45028029063) did not skip a
cure: its "4 CLW" were an unlearned flavour (Copper Speckled Potions, found
on the floor), and #470 forbids every unidentified item, so it had no
confusion cure or heal it could know of. `caution_confused_clw_cure` and
`caution_confused_cure_before_walk` therefore play the same keys (death at
8,105); the trace shows `borg_caution` reaching its cures each confused turn
and finding nothing. Details on #983.

**Done** (owner, 2026-11-22: "test when safe").
`item_test_unknown_potions` (compiled 0, shipped 1) and
`item_test_unknown_min_hp_percent` (50): `borg_quaff_unknown_when_safe`
(`borg_item_use.rs`), called right after `borg_test_stuff` in
`borg_think_dungeon`, drinks one unknown potion when in the dungeon, danger 0
next turn, at least 50% of max HP, no harmful status, food in the pack. The
harmful unknown kinds at DL1-15 (Sleep, Blindness, Confusion, Poison,
Slowness, Surprise, Salt Water: object.txt) do not kill by themselves. #470
still forbids unknown scrolls, mushrooms and devices, and any unknown item in
an emergency. Unit test `unknown_potion_tests`.

**Checked.** Test off: the same keys as `promote/P7` (+#976). Test on:
17626119 learns Cloudy (2,053) and CLW (20,909) and reaches the cap; the 13
patch-related replays: no new death (games that tested a potion diverge, the
others are unchanged). Smoke: 0/3 identical (the games now test potions),
re-recorded, all three alive at their cap. One case, divergent from 2,053: a
direction, not an effect (RULES 14).

**Where this meets other work.**
- #969/#976 (kill-on-sight HP floor): its `has_cure` counts only known
  potions, so a learned CLW also lets the floor yield more often. Independent
  code; the two PRs do not touch the same lines.
- #974 (Grip while confused by a patch): 453254028 has no potion to learn.
- #975 (another session: the patch's HP estimate after a hit): no overlap.
- The town sale of one of each unknown kind (#470) is unchanged.

**Left / owner.** The shipped 1 and the 50% floor rest on one case; borg-zoo
should measure the switch (0 against 1) in the normal world. The branch name
says "confused-scary-flight" (the first lead); the fix is the potion test.
