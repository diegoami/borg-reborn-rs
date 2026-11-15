<!-- Cemetery ownership: normal death dossiers now live in diegoami/borg-cemetery. -->

# tools/

*Ours. The tools copied from borg-rs and better-borg-c in T1.6 (GUI recording,
replay viewer, human takeover, native recording, the death workflow and the
root launchers) were dropped in TE.6 (#76; docs/SPEC.md §8, §11): they were
one toolkit built on borg-rs's recording format and its own binary. What they
gave is covered by determinism and the Borg's own logs.*

| Tool | What |
| --- | --- |
| `export-death.py` | Run by `harness/run.sh` after every run: a natural death gets a dossier in borg-cemetery (RULES 12; explicit temporary destinations supported), with the trace tail and the parameters |
| `death_registry.py` | Forwards registry maintenance to cemetery ownership; explicit historical/temporary registries remain supported (`python3 tools/death_registry.py`) |
| `rerun-death.py` | Reruns a dossier's game from its build and seed with the Borg's logs on (observations, world digests, knowledge; checkpoint saves with `--checkpoint-every N`) and checks that the keys are the recorded ones and the logs were written. It always runs `build/rust` (an inherited `ANGBAND_BIN` is dropped), and refuses a death that loaded a save (`save=NAME` or an extra `-load`): the dossier alone cannot restart that game |
| `reconcile-deaths.py` | CI: every death of a CI run is in the registry; with `--runs DIR …`, every natural death in those run outputs has a dossier of the same encounter family and key stream (E review finding 3, M2 review finding 7). The comparison baseline is `--registry` (default the retained `docs/deaths/`) |
| `check-perception.py` | T2.5: over runs of a `test-perception` build, fails on any think whose observation or world disagrees with the engine (map memory, visible monsters, gear list, pack count) or whose status is in no `state.log` line of its turn; counts what the Borg does not see by design (off the map panel). Every run of a suite directory needs a truth log with one line per Borg decision (M2 review, finding 4) |
| `depth-stream.py` | Run by `harness/run.sh`: `depths.json`, the first turn at each depth |
| `tests/` | Unit tests of the above (`python3 -m unittest discover -s tools/tests`) |

To watch the Borg play: `./watch-borg.sh` at the repository's root.

## Investigating a death

1. Check out the dossier's `git_rev` (in `record.json`) if the current
   build is another source, and run `BORG_IMPL=rust harness/build.sh`.
2. `tools/rerun-death.py docs/deaths/DXXXX-... OUT --checkpoint-every 500`
   (the dossier's directory is `$BORG_DEATH_RECORDS_DIR`, default
   `docs/deaths/`; the canonical copy lives in
   [borg-cemetery](https://github.com/diegoami/borg-cemetery)).
3. Read the rerun's trace and observations up to the death, or load a
   checkpoint before it.
4. Write the findings into the dossier's `assessment.json` and `report.md`,
   then regenerate the index.

## Cemetery-owned death tools

The export, registry, source-checked rerun and reconciliation implementations now
live in [borg-cemetery](https://github.com/diegoami/borg-cemetery/tree/main/scripts/native),
with their provenance and original tests. The four old command paths are thin
API-1 pointers; there is no second implementation to maintain here.

Clone/update borg-cemetery beside the primary checkout, or set
`BORG_CEMETERY_DIR=/path/to/borg-cemetery`. Linked worktrees discover the sibling
of their primary checkout. New normal exports go to its `deaths/` and its own
registration hook builds readable pages/catalogue without overwriting historical
indexes. `BORG_DEATH_RECORDS_DIR` remains the explicit destination override.

This repository still owns the harness, build/source identity and observations.
Its historical `docs/deaths/` is preserved for existing evidence and CI baseline
checks. CI uses the pinned external tools and its existing temporary-registry
override/reconciliation guard; no assertion is weakened. To deliberately maintain
the historical registry, supply its path explicitly rather than making it the
default output again. The reborn strategy remains model-free; the diagnosis
pilot #153 and tactical counterfactual plan #212 are not implemented by this move.
