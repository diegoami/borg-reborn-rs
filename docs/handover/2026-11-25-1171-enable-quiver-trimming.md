## 2026-11-25 — Enable town quiver trimming (#1171)

The shipped borg-rs settings enable the plain-quiver trim from #1171 for all
nine classes. The per-class reserves are Mage 30, Necromancer 35, Druid and
Blackguard 40, Priest 45, Paladin 50, Warrior 55, Rogue 60, and Ranger 70
shots. These values lower each reserve in the #1199 starting profile by ten
shots. The common setting is 0 for configurations without an override.

Nine short headless runs confirmed the effective class reserves. All four
shipped borg-rs smoke runs matched the existing keys, messages, and state
references. The class-file check, parameter citations, Rust formatting,
Clippy, and both Rust test suites passed.

[Class and level calibration (#1199)](https://github.com/diegoami/borg-reborn-rs/issues/1199)
and the [Zoo comparison (#781)](https://github.com/diegoami/borg-zoo/issues/781)
remain open.
