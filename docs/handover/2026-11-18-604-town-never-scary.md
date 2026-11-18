## 2026-11-18: the town is never scary, a switch (#604)

Owner: "who are the scary monsters in town anyway? Could we try a setting
where we always fight monsters in town, I have never got killed by monsters
in town."

- **Who:** five town residents on `harness/scary_monsters.txt` (scary up to
  CL8-11): the battle-scarred veteran, the mean-looking mercenary, the filthy
  street urchin, the squint-eyed rogue and Farmer Maggot. They made the town
  scary 2,017 times in 18 games.
- **`caution_town_never_scary`** (compiled 0, written out at 0 in
  `harness/borg.txt`): in town no monster makes the level scary.
- **On 18 starts** (9 classes x seeds 2001/2002, race 0, the borg-rs core): 2 deaths against 3, sum of CL 105 against
  93, no death in town, town <-> DL1 moves 2,831 to 1,286; 19 more hits from
  town monsters. At 0, identical to main. Details in `docs/CALIBRATION.md`.
- **Left in #604:** the remaining 1,286 moves are another loop: "Nothing to
  sell in town (leaving)" down, then on DL1 a flight or a leave takes the
  nearest up staircase back to town. The hold and return limits proposed in
  #604 target that. Whether to ship the switch at 1 is the owner's call, after
  a zoo pilot.
- **Review of #605:** borg-zoo's classes-18 starts with `BORG_CORE=borg-rs`
  give the same directions but 0 deaths on main against 2 with the switch
  (sum of CL 99 against 97). So the deaths and levels are not settled;
  the town scares and the "finish shopping" loop are (both to 0). The door
  scare is not gated by the switch; the town has no doors.
