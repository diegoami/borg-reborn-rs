## 2026-11-22: kill-on-sight HP floor for the grey mushroom patch (#969)

**Done.** `attack_kill_on_sight_min_hp_percent` (compiled 0, shipped 50 in
`harness/borg.txt`): below this percent of max HP a `kill-on-sight` entry
(#869: the grey mushroom patch) is not attacked on sight when a Cure Light,
Serious or Critical Wounds potion is in the pack or an escape is at hand
(stairs underfoot, a readable Phase Door: `borg_fast_fragile::has_cure`
and `escape_available`, now `pub(crate)`). The ordinary attack and caution
decide then. With neither, or above the floor, the attack still comes first,
confused or not (#869, owner). `harness/scary_monsters.txt`'s comment no
longer says "speed +10" (110 is normal speed). Stacked on #970 (#971).

**Found.**
- 11 of 28 deaths of the track-p baseline (zoo run 45028029063) were the
  patch: attack on sight before the cures and Phase Door, with no HP test.
- The 13% reach rate in those deaths is a selection effect of looking at
  deaths only, not an engine anomaly (engine: 11/32; simulation conditioned on
  death: 13-15%). Details in #969.
- Replays at track-p `4b2c2fcf` of the 13 patch-related deaths (11 patch, 2
  Grip): floor 0 reproduces all 13; floor 33 leaves 2 (a 13-HP Ranger yields
  only at HP 4); 50 and 66 leave 1, the Grip game (#974). At 50 each of the
  12 patch games diverges inside the fatal encounter (same game until then).
- A first version yielded with nothing at hand: the Ranger survived by a lucky
  confused step, and the Druid that arrived beside the patch (1062599390)
  lived only by taking the stairs it stood on. Hence the cure-or-escape test.
- Smoke (`smoke-borg-rs`): 3/3 identical, nothing re-recorded.

**Left.**
- #974: seed 453254028, Grip while confused by a patch; the floor never applies.
- #975: after one hit the Borg rates a 3-HP patch at 0 HP while it lives.
- Below the floor, with a cure, a confused Borg may still start a "Desperate
  for Stairs" walk instead of drinking (Ranger 1723941400 at 50%):
  `caution_confused_cure_before_walk` and `caution_confused_clw_cure` ship 0
  (#837); borg-zoo#665's `rangedclw` arm measures the CLW one.
- borg-zoo#665: the patch-world pilots (x4, x8) and then the normal world.

**Owner.** The shipped value 50 rests on 13 named replays (RULES 14: a
direction, not an effect); borg-zoo#665 measures it. Whether to keep #869's
kill-on-sight entry at all (removing it gave the same 1 of 13) is yours.
