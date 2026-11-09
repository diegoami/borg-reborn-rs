# tools/

*Ours. The tools copied from borg-rs and better-borg-c in T1.6 (GUI recording,
replay viewer, human takeover, native recording, the death workflow and the
root launchers) were dropped in TE.6 (#76; docs/SPEC.md §8, §11): they were
one toolkit built on borg-rs's recording format and its own binary. What they
gave is covered by determinism and the Borg's own logs.*

| Tool | What |
| --- | --- |
| `export-death.py` | Run by `harness/run.sh` after every run: a natural death gets a dossier in `docs/deaths/` (RULES 12), with the trace tail and the parameters |
| `death_registry.py` | Regenerates `docs/deaths/index.json` and `README.md` (`python3 tools/death_registry.py`) |
| `rerun-death.py` | Reruns a dossier's game from its build and seed with the Borg's logs on (observations, world digests, knowledge; checkpoint saves with `--checkpoint-every N`) and checks that the keys are the recorded ones |
| `reconcile-deaths.py` | CI: every death of a CI run is in `docs/deaths/` |
| `depth-stream.py` | Run by `harness/run.sh`: `depths.json`, the first turn at each depth |
| `endpoint.py` | The primary endpoint over local outputs and zoo cells; goes in TE.7 (the zoo computes every statistic) |
| `tests/` | Unit tests of the above (`python3 -m unittest discover -s tools/tests`) |

To watch the Borg play: `./watch-borg.sh` at the repository's root.

## Investigating a death

1. Check out the dossier's `git_rev` (in `record.json`) if the current
   build is another source, and run `BORG_IMPL=rust harness/build.sh`.
2. `tools/rerun-death.py docs/deaths/DXXXX-... OUT --checkpoint-every 500`.
3. Read the rerun's trace and observations up to the death, or load a
   checkpoint before it.
4. Write the findings into the dossier's `assessment.json` and `report.md`,
   then regenerate the index.
