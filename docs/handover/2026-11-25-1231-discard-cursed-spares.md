## 2026-11-25: Discard known cursed spare equipment (#1231)

The Borg now drops known cursed pack equipment at a safe opportunity before ordinary junk disposal and town shopping. Unknown runes do not protect equipment that already has a known curse. Drops use the existing ignore inscription. A cursed light remains available when there is no clean light, and worn equipment still follows the existing replacement and curse-removal rules.

The watched Mage kept two cursed torches despite having clean light and six clean spares. With the fix, a bounded native run from the captured save loaded at turn 1,714,848 and dropped the two cursed torches at 1,715,728 and 1,715,733. The run stayed alive through 150 keys. The case is recorded on [#1231](https://github.com/diegoami/borg-reborn-rs/issues/1231).

The core regression, release core suite, workspace tests, formatting, Clippy, and parameter citations passed. All four default borg-rs smoke runs matched their key, message, and state references. The watched session was not replayed key for key.

After review of #1232, the owner confirmed that prompt disposal takes priority over keeping a potentially valuable cursed swap or waiting to remove a pack item's curse. The early disposal check requires an actual known curse. An intrinsic drawback such as Aggravation is insufficient. A working clean light takes priority, and town still replenishes its normal light reserve. In the dungeon, if a cursed lantern supplies the only reserve fuel, the Borg can transfer that fuel to its clean equipped lantern before dropping the empty donor.

The updated regression checks the recorded torch mix, an intrinsic drawback, an otherwise valuable removable curse, the One Ring, and a consumable. A second regression checks the only-fuel case and the alternatives that allow immediate disposal. The watched Mage run above used the first PR revision. The smoke cases check existing play against the retained references. No new calibration setting is required.
