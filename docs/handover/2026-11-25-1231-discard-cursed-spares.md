## 2026-11-25: Discard known cursed spare equipment (#1231)

The Borg now drops known cursed pack equipment at a safe opportunity before ordinary junk disposal and town shopping. Unknown runes do not protect equipment that already has a known curse. Drops use the existing ignore inscription. A cursed light remains available when there is no clean light, and worn equipment still follows the existing replacement and curse-removal rules.

The watched Mage kept two cursed torches despite having clean light and six clean spares. With the fix, a bounded native run from the captured save loaded at turn 1,714,848 and dropped the two cursed torches at 1,715,728 and 1,715,733. The run stayed alive through 150 keys. The case is recorded on [#1231](https://github.com/diegoami/borg-reborn-rs/issues/1231).

The core regression, release core suite, workspace tests, formatting, Clippy, and parameter citations passed. All four default borg-rs smoke runs matched their key, message, and state references. The watched session was not replayed key for key.

Ready for independent review on Track P. No new calibration setting is required.
