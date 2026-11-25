## Exploration reserves and full packs (#1264)

The Warrior case had 23 occupied pack slots, eight stacks of spare gear,
four Potions of Sleep and four Scrolls of Word of Recall. The inherited
full-pack valuation dropped the Recall stack. Its nominal reserve was two.
[#1263](https://github.com/diegoami/borg-reborn-rs/issues/1263)

The opt-in policy tries known harmful items and plainly outclassed spare
gear before ranking other full-pack candidates. It uses the properties
the Borg knows and compares the worn and alternate kits after removing the
other item. It keeps visible or planned swaps and gives the Borg its normal
rune-identification opportunity. The existing Home retention handles gear
kept for later use.

Both full-pack passes and weight-driven disposal check the same trip
reserves. In the arrival-unload town mode, a shop purchase or Home
withdrawal can exceed the normal weight allowance one needed copy at a
time. The travel weight target still guides optional purchases. The
reserve quantities come from the class's existing town settings and trip
forecast. [Calibration](../CALIBRATION.md#exploration-reserves-and-full-packs-1264)

Native fixtures exercise the harmful-potion and outclassed-weapon decisions
in a full pack, preserve the Recall stack, and replenish Recall from Home
while the Priest stays above the ordinary weight limit.

The flag defaults to zero pending the paired Zoo evaluation. The comparison
request calls for the control to include the separate #1265 change.
[#804](https://github.com/diegoami/borg-zoo/issues/804)
