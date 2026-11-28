# Investigating a game

*Part of the [contributor guide](README.md).*

**Where the findings go.** Gameplay evidence (an investigation of a death,
a saved or constructed scenario, a trial or a counterfactual) belongs in
[borg-cemetery](https://github.com/diegoami/borg-cemetery). This repository
keeps the code, the tests and short summaries that link to the Cemetery
case (AGENTS.md, *Gameplay reports and scenarios*).

## Replaying a game by seed

A headless game is deterministic: the same build, seed and settings give
the same keys. To look at a game again, play it again:

```bash
harness/run.sh /tmp/case 1 0 0 5000 menu
```

The arguments are the output directory, the seed, the race, the class, the
number of turns and the flags. Useful flags:

| Flag | What |
| --- | --- |
| `exact` | stop exactly at that turn |
| `finalsave` | save at the stop |
| `save=NAME` | start from a saved game |
| `start-depth=N`, `start-monsters=...` | start at a depth, with monsters around |

To try a setting, pass it for that run with `PBORG_SET="name=value;..."` or
`BORG_PARAM_<NAME>=value`. Every flag and variable is listed at the top of
[harness/run.sh](../../harness/run.sh).

## The decision trace

Every think of the borg-rs core writes one line of `trace-core.jsonl`:
- the turn and the context (town or dungeon, depth, HP, SP);
- the danger at the Borg's position;
- the priority check that acted (`borg_caution`, `borg_recover`,
  `borg_attack`, ...);
- its goals and the keys it queued;
- each attack ranking, with its candidates and their parameters.

The schema is in [docs/CORE-TRACE.md](../CORE-TRACE.md). Other
run outputs:
- `messages.log`, `keys.log` and `state.log`;
- `settings-resolved.json`, which says which value of each setting the run
  played with, and where it came from.

## Death dossiers

A headless run that dies exports a **dossier** to borg-cemetery: its
settings, its keys, the trace's tail. The tools are borg-cemetery's, run
through the pointers in `tools/` after checking `death-tools-pin.txt`
([tools/README.md](../../tools/README.md)). To rerun a death and verify
that it plays again key for key:

```bash
tools/rerun-death.py /path/to/borg-cemetery/deaths/DXXXX-... /tmp/rerun --checkpoint-every 500
```

**Checkpoints.** The rerun saves a checkpoint every 500 turns, so a later
look can start just before the death.

**Rebuilding.** If the dossier was made by another build, check out its
`git_rev` (in `record.json`) and rebuild first.

**Recording the death.** A death that plays differently from every recorded
one needs a new Cemetery package. An unchanged control game needs none
(AGENTS.md, *Retained evidence*).

## The recorder

For a game played by hand or watched in a window, the optional recorder
captures the game's inputs and replays them, checking that the Borg chooses
the same keys again. It is built with `-DSUPPORT_BORG_RECORDING=ON`, and
its recipe and checks are in [docs/ANGBAND-RECORDER.md](../ANGBAND-RECORDER.md).
`harness/check-recorder.py` exercises it.

## Guided debugging

The owner's workflow for a recorded game with a problem is in
[docs/GUIDED-DEBUGGING.md](../GUIDED-DEBUGGING.md):
- preparing before play;
- inspecting recent saves;
- what to keep when something looks wrong;
- how a session reproduces and repairs the problem from the preserved
  evidence (RULES 26).
