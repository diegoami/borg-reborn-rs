# Rolling native save points and fast inspection

Owner-requested incident tooling after the PlanMage wild-cat death in
borg-reborn-rs#350. This is save retention and read-only navigation, not the
future branch-tree menu and not an assertion that milestone M1 is accepted.

## Capture

Opt in before starting a recorder-enabled process:

```sh
ANGBAND_SAVE_WINDOW=128 ANGBAND_SAVE_EVERY=1000 \
  angband-recorder record /path/to/player-cemetery/recordings/SESSION/recording \
  -- /path/to/frozen/angband -mx11 ...
```

The last128 living command-boundary saves are kept. In addition, the first
save in each1000-game-turn bucket is retained for the older history. Supply
zero for either component to disable it; both zero disable save retention.
When only one setting is supplied, the other defaults to128/1000. With both
variables absent, existing capture behavior is unchanged. Window limit10000;
interval limit1000000000. Malformed/non-UTF8/out-of-range values fail visibly.

Native snapshots already used to fingerprint command frames are copied;
no additional save or RNG call touches the live game. Death and non-command
frames cannot replace the latest living point. Ordinary UI commands with no
world-turn cost are still command boundaries. Saving errors fail the capture
rather than silently promising a checkpoint that was not written.

`RECORDING/checkpoints/index.json` lists retained filenames, command frame,
game turn, ordered-input position, SHA256 and world fingerprint. Individual
save files are immutable; a new index is atomically published before expired
non-periodic files are pruned. This is live single-process retention, not a
concurrent registrar or crash-recovery repair tool. A crash may leave an
extra unindexed file; integrity validation rejects it, preserving evidence.

Completion validation checks the file inventory, hashes and correspondence
to actual recorded command frames. Existing recordings without checkpoints
remain readable. Retention can be enabled on a verifying replay too; optional
checkpoint scheduling is not another game input or an engine observation.
Long-run I/O/storage costs are not calibrated by this initial bounded proof.

## Inspect without running the game

```sh
angband-recorder view RECORDING --turn 411638
angband-recorder view RECORDING 26812
```

The turn form reconstructs the first command frame at or after that game
turn in the selected segment, using recorded screen deltas. It does not run
the Borg, load a game or replay the preceding actions. A frame number selects
a specific boundary when several frames share a game turn. This also reads
existing v1 frame streams. This is a CLI text viewer, not an in-game menu.

## Resume from a retained point

Select the desired entry and copy its native `.save` to a new private game
profile. Preserve the original save, configuration, plan, executable and all
streams. Load the copy in a fresh game process, with its save directory
selected before `-uNAME`, and record the new continuation from startup.
Consumer launchers own character names, Borg activation and publication.

**The save preserves game state, not Borg process memory.** Loading may
compact monster lists and changes the execution boundary; do not claim that
an arbitrary suffix of the parent recording will reproduce. A restarted
Borg begins with fresh perception/flows/goals unless that consumer explicitly
restores them. Prove a saved-position reproduction independently. A new
continuation can then verify against its own recording. Replays and resumed
segments of the same life are not additional population samples.

A consumer must adopt a tagged release and update its external pin to use
these hooks. Setting these variables does not upgrade an old executable.
Executables remain local; curated streams/save exports go to player-cemetery.

## Validation

Rust checks cover configuration, rolling/coarse retention, exact saved bytes,
expired-file cleanup, rejected changed/missing snapshots, frame/input binding,
and read-only turn navigation. The vanilla native round trip runs with window3
and interval20, keeps actual forked engine saves, verifies replay and identical
final native saves. A fresh X11 process loads a retained turn41 save, advances
to81, and its continuation verifies independently. The stock test frontend
clears savefile names on purpose, so the load proof uses the real X11 route.
CI runs it under its own virtual display. Failed local setup attempts remain
retained in scratch rather than being counted as successful game runs.
