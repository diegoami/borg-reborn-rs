# #1261 Inferior sandal margin by class

The shipped class files now set the unknown-rune Leather Sandals margin to 3 for Mage and Necromancer, 4 for Druid, 5 for Priest, 6 for Rogue and Ranger, 7 for Paladin, 8 for Blackguard, and 9 for Warrior. The values follow the class Strength modifiers in `engine/angband/lib/gamedata/class.txt`.

The default Borg core reads these overrides from the folded `harness/borg.txt`. To test a flat threshold on this build, set that threshold on each class-qualified key. [Zoo #804](https://github.com/diegoami/borg-zoo/issues/804) tracks measurement and broader armor and weapon coverage.
