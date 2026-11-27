# Keep Home loadout retention local to Home stock

Issue #1346. A Mage on Track P stored an ordinary Main Gauche and Rapier
during loadout exchanges. Later pickups of the same variants stayed in the
pack through another town visit. The live map at turn 1,481,422 shows the
three carried copies and two matching pieces at Home. Reloading the
checkpoint with fresh Borg memory dropped all three carried weapons in two
decisions. The matching save, map, settings, and trace excerpts are in the
[Mage evidence bundle](https://github.com/diegoami/angband-workspace/releases/download/borg-mage-staff-teleports/borg-mage-live-spare-weapons.tar.gz).

The retention query now accepts an actual Home slot. Pack comparisons and
disposal use their normal equipment and swap checks. The loadout transaction
still records the displaced gear after the deposit and can restore it when
the character can carry it.

The native town-loadout regression exercises a real Home deposit, inserts
a matching two-copy pack stack into the Borg's observed inventory, and
checks the gear plan before continuing the existing restoration test. The
plan can release carried copies and protects the deposited stock. The
teleport behavior is tracked separately in #1342.
