# #1078: ship cave_light_engine_radius = 1

**Done.** `harness/borg.txt` ships `borg_param_cave_light_engine_radius = 1`
(owner, 2026-11-23): the Borg's light is the engine's (#877, #884, #958).
`engine/tests/test_stair_handoff.py` pins the incident replays to 0.
`smoke-borg-rs` is re-recorded: 0/3 plays identical.

**Found (smoke, default core borg-rs; one game each, a lead only).**
- Warrior: CL 3 -> 4, max depth 2 -> 3.
- Mage: CL 3 -> 4, max depth 2 -> 1.
- Priest: CL 2 -> 1, keys 2,011 -> 1,050, kills 15 -> 5. It casts Call Light
  34 times against 18 and rests on a staircase for mana 150 times against 2.
  With a torch's engine radius (distance 1) more grids read as dark, and
  the Priest spends its mana lighting them.

**Left.** Not measured on the current code (borg-zoo#625 was an old build:
9 deaths against 8). borg-zoo#702 (running, at 206cf4e7) and borg-zoo PR #709
(at f8725bc0, with unseen_belief) measure it. If the zoo confirms that casters
spend their mana on light, the light-casting rules (Call Light, the dark flow)
may need their own look.

**For the owner.** The Priest's mana spent on Call Light (above).
